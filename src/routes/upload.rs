use axum::{
    Json,
    extract::{Multipart, Path, State},
};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::ingest::manual;
use crate::routes::auth::AuthUser;
use crate::routes::honeypot::{self, TrapVerdict};
use crate::server::AppState;
use std::time::{SystemTime, UNIX_EPOCH};

/// POST /api/upload — Upload a new fic manually
pub async fn handle_manual_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    // Require authentication
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // ── Trust gate: uploads require TL1+ (email-verified reader) ──────
    // Stops brand-new throwaway accounts from flooding the archive with
    // spam/copyrighted uploads.
    let trust_level = crate::services::trust::fetch_trust_level(&state.db, Some(user_id)).await;
    if trust_level < 1 {
        return Err(AppError::Forbidden(
            "Uploads require a verified account (Trust Level 1+). Verify your email first.".into(),
        ));
    }

    // ── Per-user rate limit (anti-script: blocks "1M × 1MB epubs" floods) ─
    // Keyed on the username (client_id) so one account can't flood the
    // archive. The Download tier is 60/hr per key by default.
    let client_id = user.username.as_deref();
    if let crate::limiter::TieredRateLimitResult::Wait(secs) = state
        .rate_limiter
        .check(
            std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            client_id,
            crate::limiter::Tier::Download,
        )
        .await
    {
        return Err(AppError::RateLimited(secs));
    }

    let mut title = String::new();
    let mut author = String::new();
    let mut summary = String::new();
    let mut tags = String::new();
    let mut status = String::new();
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    let mut website = None;
    let mut form_opened_at = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => title = field.text().await.unwrap_or_default(),
            "author" => author = field.text().await.unwrap_or_default(),
            "summary" => summary = field.text().await.unwrap_or_default(),
            "tags" => tags = field.text().await.unwrap_or_default(),
            "status" => status = field.text().await.unwrap_or_default(),
            "website" => website = Some(field.text().await.unwrap_or_default()),
            "form_opened_at" => form_opened_at = Some(field.text().await.unwrap_or_default()),
            "file" => {
                file_name = field.file_name().unwrap_or("upload.txt").to_string();
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    // ── Honeypot + form-timing trap (silent rejection) ──────────────────────
    // Mirrors the registration/comment handlers: a bot that fills the hidden
    // `website` field, or submits without the JS-set `form_opened_at` (or too
    // fast), gets a success-looking response but the upload is dropped. This
    // stops scripted API clients from spamming manual uploads.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict =
        honeypot::inspect_submission(website.as_deref(), form_opened_at.as_deref(), now_ms);
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("manual upload silently rejected (honeypot/timing trap)");
        return Ok(Json(json!(
            {
                "err": 0,
                "url_id": "",
                "work_id": 0,
                "title": title,
                "msg": "Fic uploaded successfully",
            }
        )));
    }

    if title.is_empty() || author.is_empty() || file_data.is_empty() {
        return Err(AppError::BadRequest(
            "title, author, and file are required".to_string(),
        ));
    }

    // ── Upload hardening ──────────────────────────────────────────────
    // Allowed extensions only (blocks .exe, .wasm, .zip bombs, scripts).
    let lower_name = file_name.to_lowercase();
    let allowed_ext = [".txt", ".epub", ".html", ".htm"];
    if !allowed_ext.iter().any(|ext| lower_name.ends_with(ext)) {
        return Err(AppError::BadRequest(format!(
            "Unsupported file type '{}'. Allowed: .txt, .epub, .html",
            std::path::Path::new(&file_name)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into())
        )));
    }

    // Hard size cap: MAX_UPLOAD_BYTES (default 20MB) — blocks the "1M × 1MB
    // epubs" script attack and accidental giant uploads. The body is read
    // fully into memory above, so this is the last line of defense after
    // the route-level DefaultBodyLimit.
    let max_upload_bytes = state.config.max_upload_bytes;
    if file_data.len() > max_upload_bytes {
        return Err(AppError::PayloadTooLarge(format!(
            "Uploaded file is {} bytes; maximum allowed is {} bytes ({} MB)",
            file_data.len(),
            max_upload_bytes,
            max_upload_bytes / (1024 * 1024),
        )));
    }

    let payload = manual::ManualUploadPayload {
        title: title.clone(),
        author: author.clone(),
        summary,
        tags: tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        status,
        file_data,
        file_name,
        user_id,
    };

    let (meta, chapters) = manual::parse_manual_upload(payload).await?;

    // Insert fic_info with source_type='manual_epub'
    let fic_info_row = crate::db::models::FicInfo {
        id: meta.url_id.clone(),
        created: None,
        updated: None,
        title: meta.title.clone(),
        author: meta.author.clone(),
        author_url: Some(meta.author_url.clone()),
        author_local_id: Some(meta.author_local_id.clone()),
        chapters: meta.chapters,
        words: meta.words,
        description: meta.desc.clone(),
        fic_created: chrono::DateTime::from_timestamp_millis(meta.published).unwrap_or_default(),
        fic_updated: chrono::DateTime::from_timestamp_millis(meta.updated).unwrap_or_default(),
        status: meta.status.clone(),
        source: meta.source.clone(),
        extra_meta: meta.extra_meta.clone(),
        raw_extended_meta: None,
        source_id: Some(meta.source_id),
        author_id: Some(meta.author_id),
        content_hash: None,
        work_id: None,
    };
    crate::db::queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // Update source_type to 'manual_epub'
    sqlx::query("UPDATE fic_info SET source_type = 'manual_epub' WHERE id = $1")
        .bind(&meta.url_id)
        .execute(&state.db)
        .await?;

    // The reader looks up the HTML cache by `content_hash` (falling back to
    // "upstream" when null). The export logs for manual uploads are keyed on
    // the url_id — so populate content_hash with the url_id to make the
    // reader's find_export_log match (fixes "No HTML cache" for uploads).
    sqlx::query("UPDATE fic_info SET content_hash = $1 WHERE id = $2")
        .bind(&meta.url_id)
        .bind(&meta.url_id)
        .execute(&state.db)
        .await?;

    // Auto-merge: find or create work
    let auto_merge_result = crate::works::find_or_create_work(&state.db, &meta).await?;
    let work_id = auto_merge_result.work_id;

    // Set uploader_id on the work
    sqlx::query("UPDATE works SET uploader_id = $1 WHERE id = $2")
        .bind(user_id)
        .bind(work_id)
        .execute(&state.db)
        .await?;

    // Generate EPUB and cache it
    let (epub_path, epub_hash) =
        crate::export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
            .await
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    let cache_dest = crate::cache::disk::cache_path(
        &state.config.cache_dir,
        &crate::cache::EType::Epub,
        &meta.url_id,
        &epub_hash,
    );
    crate::cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    let version = state.config.export_version;
    crate::db::queries::insert_export_log(
        &state.db,
        &meta.url_id,
        version,
        "epub",
        &meta.url_id,
        &epub_hash,
    )
    .await?;

    // Generate HTML bundle
    let (html_path, html_hash) =
        crate::export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir)
            .await
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    let html_cache_dest = crate::cache::disk::cache_path(
        &state.config.cache_dir,
        &crate::cache::EType::Html,
        &meta.url_id,
        &html_hash,
    );
    crate::cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
    crate::db::queries::insert_export_log(
        &state.db,
        &meta.url_id,
        version,
        "html",
        &meta.url_id,
        &html_hash,
    )
    .await?;

    // Award XP for manual upload
    let _ =
        crate::db::queries::update_reputation_and_promote(&state.db, user_id, 50, "manual_upload")
            .await;
    let _ = crate::db::queries::check_and_award_badges(&state.db, user_id, "manual_upload").await;

    // Fetch the user's current reputation so the frontend can show a toast
    // ("+50 reputation") and the value is immediately fresh.
    let reputation: i32 = sqlx::query_scalar("SELECT reputation FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "work_id": work_id,
        "title": meta.title,
        "msg": "Fic uploaded successfully",
        "rep_gained": 50,
        "reputation": reputation,
    })))
}

/// POST /api/upload/{url_id}/update — Update chapters on an existing manual upload
pub async fn handle_fic_update(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Verify ownership: only uploader or curator can update
    let work = crate::db::queries::get_work_by_source(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Work not found".into()))?;

    // Only allow if user is admin/curator (role >= 10) or the uploader
    if user.trust_level < 5 && work.uploader_id != Some(user_id) {
        return Err(AppError::Forbidden(
            "Only the uploader or a curator can update this fic".into(),
        ));
    }

    // Extract file from multipart
    let mut file_data = Vec::new();
    let mut file_name = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            file_name = field.file_name().unwrap_or("update.txt").to_string();
            file_data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if file_data.is_empty() {
        return Err(AppError::BadRequest(
            "file is required for update".to_string(),
        ));
    }

    // Get existing metadata
    let fic = crate::db::queries::get_fic_info(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Fic not found".into()))?;

    let payload = manual::ManualUploadPayload {
        title: fic.title,
        author: fic.author,
        summary: fic.description,
        tags: Vec::new(),
        status: fic.status,
        file_data,
        file_name,
        user_id,
    };

    let (new_meta, new_chapters) = manual::parse_manual_upload(payload).await?;

    // Increment version bump
    let version_bump = crate::db::queries::get_fic_version_bump(&state.db, &url_id)
        .await?
        .unwrap_or(0)
        + 1;
    sqlx::query(
        "INSERT INTO fic_version_bump (id, value) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(&url_id)
    .bind(version_bump)
    .execute(&state.db)
    .await?;

    // Regenerate EPUB
    let (epub_path, epub_hash) =
        crate::export::epub::create_epub(&new_meta, &new_chapters, &state.config.tmp_dir)
            .await
            .map_err(|e| AppError::ExportError(e.to_string()))?;
    let cache_dest = crate::cache::disk::cache_path(
        &state.config.cache_dir,
        &crate::cache::EType::Epub,
        &url_id,
        &epub_hash,
    );
    crate::cache::disk::move_to_cache(&epub_path, &cache_dest)?;
    let version = state.config.export_version + version_bump;
    crate::db::queries::insert_export_log(&state.db, &url_id, version, "epub", &url_id, &epub_hash)
        .await?;

    // Notify followers
    let _ = crate::db::queries::notify_work_followers(&state.db, work.id, &new_meta.title).await;

    // Award reputation for keeping content fresh (encourages updates).
    let _ = crate::db::queries::update_reputation_and_promote(&state.db, user_id, 10, "fic_update")
        .await;
    let reputation: i32 = sqlx::query_scalar("SELECT reputation FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "msg": "Fic updated successfully",
        "rep_gained": 10,
        "reputation": reputation,
    })))
}

/// DELETE /api/upload/{url_id} — Delete (hide) a manual upload
pub async fn handle_fic_delete(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let work = crate::db::queries::get_work_by_source(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Work not found".into()))?;

    // Only allow if user is admin/curator (role >= 10) or the uploader
    if user.trust_level < 5 && work.uploader_id != Some(user_id) {
        return Err(AppError::Forbidden(
            "Only the uploader or a curator can delete this fic".into(),
        ));
    }

    // Soft delete: set is_visible = FALSE
    sqlx::query("UPDATE works SET is_visible = FALSE WHERE id = $1")
        .bind(work.id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        json!({"err": 0, "msg": "Fic hidden from public view"}),
    ))
}
