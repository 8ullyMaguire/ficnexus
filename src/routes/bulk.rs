//! Bulk admin actions — re-scrape and auto-tag multiple fics at once.
//!
//! - `POST /api/admin/bulk/refresh`   — batch re-scrape (role ≥ 10)
//! - `POST /api/admin/bulk/auto-tag`  — batch auto-tag  (role ≥ 10)
//! - `GET  /api/admin/fics/search`    — fic picker search (role ≥ 10)

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::auto_tagger;

// ─── Request / Query types ────────────────────────────────────────────────

/// Shared request body for bulk operations (max 50 url_ids).
#[derive(Debug, Deserialize)]
pub struct BulkRequest {
    pub url_ids: Vec<String>,
}

/// Query params for the admin fic search.
#[derive(Debug, Deserialize)]
pub struct FicSearchQuery {
    pub q: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: Option<i64>,
}

fn default_limit() -> Option<i64> {
    Some(50)
}

// ─── Handlers ─────────────────────────────────────────────────────────────

/// `POST /api/admin/bulk/refresh` — sequentially re-scrape each fic (no
/// parallelism so we don't hammer source sites).
pub async fn bulk_refresh(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<BulkRequest>,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&user, &state)?;

    let url_ids: Vec<String> = req.url_ids.into_iter().take(50).collect();
    if url_ids.is_empty() {
        return Err(AppError::BadRequest("url_ids is required".to_string()));
    }

    let mut results = Vec::with_capacity(url_ids.len());

    for url_id in &url_ids {
        let result = refresh_single_fic(&state, url_id).await;
        results.push(result);
    }

    Ok(Json(json!({ "results": results })))
}

/// `POST /api/admin/bulk/auto-tag` — sequentially auto-tag each fic.
pub async fn bulk_auto_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<BulkRequest>,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&user, &state)?;

    let url_ids: Vec<String> = req.url_ids.into_iter().take(50).collect();
    if url_ids.is_empty() {
        return Err(AppError::BadRequest("url_ids is required".to_string()));
    }

    let mut results = Vec::with_capacity(url_ids.len());

    for url_id in &url_ids {
        let result = auto_tag_single_fic(&state, url_id).await;
        results.push(result);
    }

    Ok(Json(json!({ "results": results })))
}

/// `GET /api/admin/fics/search?q=...&limit=50` — search fics by title for
/// the admin picker.
pub async fn search_fics_for_admin(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<FicSearchQuery>,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&user, &state)?;

    let q = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(50).min(100);

    let pattern = format!("%{q}%");
    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT id, title, author, source
        FROM fic_info
        WHERE title ILIKE $1 OR author ILIKE $1
        ORDER BY title
        LIMIT $2
        "#,
    )
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let fics: Vec<Value> = rows
        .into_iter()
        .map(|(id, title, author, source)| {
            json!({
                "url_id": id,
                "title": title,
                "author": author,
                "source": source,
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "fics": fics })))
}

// ─── Private helpers ──────────────────────────────────────────────────────

/// Re-scrape a single fic, mirroring the logic from `refresh_fic_handler`.
async fn refresh_single_fic(state: &AppState, url_id: &str) -> Value {
    let url_id = url_id.trim();
    if url_id.is_empty() {
        return json!({ "url_id": "", "status": "error", "message": "empty url_id" });
    }

    // Look up existing fic metadata.
    let existing = match queries::get_fic_info(&state.db, url_id).await {
        Ok(Some(fi)) => fi,
        Ok(None) => {
            return json!({ "url_id": url_id, "status": "error", "message": "fic not found" });
        }
        Err(e) => {
            return json!({ "url_id": url_id, "status": "error", "message": e.to_string() });
        }
    };

    if existing.source.is_empty() {
        return json!({ "url_id": url_id, "status": "error", "message": "fic has no source URL" });
    }

    let source_url = existing.source.clone();

    // Find a scraper that can handle this URL.
    let scraper = match state.scraper_registry.find_specific_or_fff(&source_url) {
        Some(s) => s,
        None => {
            return json!({ "url_id": url_id, "status": "error", "message": format!("unsupported URL: {source_url}") });
        }
    };

    // Perform the scrape.
    let meta = match scraper.lookup(&state.http_client, &source_url).await {
        Ok(m) => m,
        Err(e) => {
            return json!({ "url_id": url_id, "status": "error", "message": e.to_string() });
        }
    };

    let changed = meta.updated > existing.fic_updated.timestamp_millis()
        || meta.chapters != existing.chapters
        || meta.words != existing.words;

    if !changed {
        return json!({ "url_id": url_id, "status": "ok", "message": "no change" });
    }

    // Persist the fresh metadata.
    let fic_info_row = crate::db::models::FicInfo {
        id: url_id.to_string(),
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
        raw_extended_meta: meta.raw_extended_meta.clone(),
        source_id: Some(meta.source_id),
        author_id: Some(meta.author_id),
        content_hash: meta.content_hash.clone(),
        work_id: existing.work_id,
    };

    if let Err(e) = queries::upsert_fic_info(&state.db, &fic_info_row).await {
        return json!({ "url_id": url_id, "status": "error", "message": format!("db error: {e}") });
    }

    // Reuse existing work or create one.
    let work_id = match existing.work_id {
        Some(wid) => wid,
        None => match crate::works::find_or_create_work(&state.db, &meta).await {
            Ok(w) => w.work_id,
            Err(e) => {
                return json!({ "url_id": url_id, "status": "error", "message": format!("work creation failed: {e}") });
            }
        },
    };

    if queries::get_work_by_source(&state.db, url_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        let _ = queries::link_source_to_work(&state.db, url_id, work_id).await;
    }

    // Bump version so export cache regenerates.
    let bump = queries::get_fic_version_bump(&state.db, url_id)
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
        + 1;

    let _ = sqlx::query(
        "INSERT INTO fic_version_bump (id, value) VALUES ($1, $2)
         ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(url_id)
    .bind(bump)
    .execute(&state.db)
    .await;

    json!({ "url_id": url_id, "status": "ok", "message": "updated" })
}

/// Auto-tag a single fic via the auto_tagger service.
async fn auto_tag_single_fic(state: &AppState, url_id: &str) -> Value {
    let url_id = url_id.trim();
    if url_id.is_empty() {
        return json!({ "url_id": "", "status": "error", "suggested_count": 0, "message": "empty url_id" });
    }

    match auto_tagger::recommend_tags(&state.db, &state.ollama, url_id).await {
        Ok(suggestions) => {
            let count = suggestions.len();
            json!({
                "url_id": url_id,
                "status": "ok",
                "suggested_count": count,
                "message": format!("{count} tag(s) suggested"),
            })
        }
        Err(auto_tagger::AutoTaggerError::NoContent(msg)) => {
            json!({ "url_id": url_id, "status": "error", "suggested_count": 0, "message": msg })
        }
        Err(auto_tagger::AutoTaggerError::Embed(err)) => {
            tracing::warn!("bulk auto-tag: embedding failed for {url_id}: {err}");
            json!({ "url_id": url_id, "status": "error", "suggested_count": 0, "message": "embedding failed (is Ollama up?)" })
        }
        Err(auto_tagger::AutoTaggerError::Db(err)) => {
            json!({ "url_id": url_id, "status": "error", "suggested_count": 0, "message": format!("db error: {err}") })
        }
    }
}
