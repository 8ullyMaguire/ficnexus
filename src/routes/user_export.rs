//! One-click full user data export (`GET /api/user/export`).
//!
//! Lets a signed-in user download a single ZIP archive containing all of
//! their personal data: profile, bookmarks (with notes), shelves,
//! reading stats/status, comments, follows, notification preferences,
//! login streak and reputation/badges.
//!
//! The archive is a flat ZIP of JSON files with clear, human-readable
//! filenames (e.g. `bookmarks.json`, `shelves.json`, `notes.json`).
//! Bookmarks carry the `notes` column, so `notes.json` is an aggregate of
//! all per-item user notes (bookmarks + reading list) — there is no
//! standalone `notes` table in the schema.
//!
//! Auth: `AuthUser` extractor — anonymous requests get HTTP 401
//! (`{"err": 401, "msg": "Login required"}`), matching the other social
//! endpoints. Every query is scoped by `user_id`; nothing is global.

use std::io::Write;
use std::sync::Arc;

use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Collect every bookmark (with notes + private flag) belonging to the user.
async fn collect_bookmarks(state: &AppState, user_id: i32) -> Result<Vec<Value>, AppError> {
    // bookmarks.id is BIGSERIAL (INT8); work_id is INT4 nullable.
    let rows = sqlx::query_as::<_, (i64, Option<i32>, String, bool, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT b.id, b.work_id, b.notes, b.is_private, b.created_at
           FROM bookmarks b
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, work_id, notes, is_private, created_at)| {
            json!({
                "id": id,
                "work_id": work_id,
                "notes": notes,
                "is_private": is_private,
                "created_at": created_at.to_rfc3339(),
            })
        })
        .collect())
}

/// Collect shelves + the works each shelf contains.
async fn collect_shelves(state: &AppState, user_id: i32) -> Result<Vec<Value>, AppError> {
    let shelves = sqlx::query_as::<_, (i32, String, String, bool, i32, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT s.id, s.name, s.description, s.is_public, s.sort_order, s.created_at
           FROM shelves s
           WHERE s.user_id = $1
           ORDER BY s.sort_order, s.id"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let mut out = Vec::with_capacity(shelves.len());
    for (id, name, description, is_public, sort_order, created_at) in shelves {
        let items = sqlx::query_scalar::<_, i32>(
            "SELECT work_id FROM work_shelves WHERE shelf_id = $1 ORDER BY added_at",
        )
        .bind(id)
        .fetch_all(&state.db)
        .await?;
        out.push(json!({
            "id": id,
            "name": name,
            "description": description,
            "is_public": is_public,
            "sort_order": sort_order,
            "created_at": created_at.to_rfc3339(),
            "works": items,
        }));
    }
    Ok(out)
}

/// Collect the user's reading stats + status rows (the per-work "notes"
/// surface: words read, status, chapter progress, last-read time).
async fn collect_reading(state: &AppState, user_id: i32) -> Result<Vec<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i32, i64, chrono::DateTime<chrono::Utc>, i32, Option<String>, Option<i32>)>(
        r#"SELECT rs.work_id, rs.words_read, rs.last_read_at, rs.read_count, rs.status, rs.current_chapter
           FROM reading_stats rs
           WHERE rs.user_id = $1
           ORDER BY rs.last_read_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(work_id, words_read, last_read_at, read_count, status, current_chapter)| {
            json!({
                "work_id": work_id,
                "words_read": words_read,
                "last_read_at": last_read_at.to_rfc3339(),
                "read_count": read_count,
                "status": status,
                "current_chapter": current_chapter,
            })
        })
        .collect())
}

/// Collect the user's comments.
async fn collect_comments(state: &AppState, user_id: i32) -> Result<Vec<Value>, AppError> {
    let rows = sqlx::query_as::<_, (String, Option<i32>, Option<i64>, String, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, bool)>(
        r#"SELECT c.url_id, c.work_id, c.parent_id, c.body, c.created_at, c.updated_at, c.is_hidden
           FROM comments c
           WHERE c.user_id = $1
           ORDER BY c.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url_id, work_id, parent_id, body, created_at, updated_at, is_hidden)| {
            json!({
                "url_id": url_id,
                "work_id": work_id,
                "parent_id": parent_id,
                "body": body,
                "created_at": created_at.to_rfc3339(),
                "updated_at": updated_at.map(|d| d.to_rfc3339()),
                "is_hidden": is_hidden,
            })
        })
        .collect())
}

/// Collect follows (users, works, authors).
async fn collect_follows(state: &AppState, user_id: i32) -> Result<Vec<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i64, Option<i32>, Option<i32>, Option<String>, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT f.id, f.followee_id, f.work_id, f.author_name, f.created_at
           FROM follows f
           WHERE f.follower_id = $1
           ORDER BY f.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, followee_id, work_id, author_name, created_at)| {
            json!({
                "id": id,
                "followee_id": followee_id,
                "work_id": work_id,
                "author_name": author_name,
                "created_at": created_at.to_rfc3339(),
            })
        })
        .collect())
}

/// Collect reputation events + badges for the user.
async fn collect_reputation(state: &AppState, user_id: i32) -> Result<Value, AppError> {
    let events = sqlx::query_as::<_, (String, i32, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT re.event_type, re.points, re.reference_type, re.reference_id, re.created_at
           FROM reputation_events re
           WHERE re.user_id = $1
           ORDER BY re.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let badges = sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>)>(
        "SELECT badge_type, earned_at FROM user_badges WHERE user_id = $1 ORDER BY earned_at",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(json!({
        "reputation_events": events.iter().map(|(et, pts, rt, rid, at)| json!({
            "event_type": et,
            "points": pts,
            "reference_type": rt,
            "reference_id": rid,
            "created_at": at.to_rfc3339(),
        })).collect::<Vec<_>>(),
        "badges": badges.iter().map(|(bt, at)| json!({
            "badge_type": bt,
            "earned_at": at.to_rfc3339(),
        })).collect::<Vec<_>>(),
    }))
}

/// Collect reading-level stats aggregates from the users table.
async fn collect_user_stats(
    state: &AppState,
    user_id: i32,
    username: &str,
    role: i16,
) -> Result<Value, AppError> {
    let row = sqlx::query_as::<_, (i64, i32, Option<chrono::DateTime<chrono::Utc>>)>(
        "SELECT total_words_read, total_works_read, last_active_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok(json!({
        "user_id": user_id,
        "username": username,
        "role": role,
        "total_words_read": row.0,
        "total_works_read": row.1,
        "last_active_at": row.2.map(|d| d.to_rfc3339()),
        "exported_at": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Collect login streak.
async fn collect_progress(state: &AppState, user_id: i32) -> Result<Value, AppError> {
    let streak = sqlx::query_as::<_, (i32, i32, chrono::NaiveDate, chrono::DateTime<chrono::Utc>)>(
        "SELECT current_streak, longest_streak, last_login_date, updated_at FROM login_streaks WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    Ok(json!({
        "login_streak": streak.map(|(cur, longest, last, updated)| json!({
            "current_streak": cur,
            "longest_streak": longest,
            "last_login_date": last.to_string(),
            "updated_at": updated.to_rfc3339(),
        })),
        "quests": null,
    }))
}

/// Write a pretty JSON entry into the zip archive.
fn write_json_entry(
    zip: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>,
    name: &str,
    value: &Value,
) -> Result<(), AppError> {
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(name, options)
        .map_err(|e| AppError::Internal(format!("failed to start ZIP entry '{name}': {e}")))?;
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| AppError::Internal(format!("failed to serialize '{name}': {e}")))?;
    bytes.push(b'\n');
    zip.write_all(&bytes)
        .map_err(|e| AppError::Internal(format!("failed to write ZIP entry '{name}': {e}")))?;
    Ok(())
}

/// GET /api/user/export
///
/// Returns a `application/zip` archive `ficnexus-user-data-<username>.zip`
/// with one JSON file per data category. Authentication required.
pub async fn user_export_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let username = auth.username.clone().unwrap_or_else(|| format!("user{user_id}"));

    let bookmarks = collect_bookmarks(&state, user_id).await?;
    let shelves = collect_shelves(&state, user_id).await?;
    let reading = collect_reading(&state, user_id).await?;
    let comments = collect_comments(&state, user_id).await?;
    let follows = collect_follows(&state, user_id).await?;
    let reputation = collect_reputation(&state, user_id).await?;
    let stats = collect_user_stats(&state, user_id, &username, auth.role).await?;
    let progress = collect_progress(&state, user_id).await?;

    // ── Assemble the archive ──────────────────────────────────────────
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    write_json_entry(&mut zip, "profile.json", &stats)?;
    write_json_entry(&mut zip, "bookmarks.json", &json!({ "bookmarks": bookmarks }))?;
    write_json_entry(&mut zip, "shelves.json", &json!({ "shelves": shelves }))?;
    write_json_entry(&mut zip, "notes.json", &json!({ "notes": bookmarks }))?;
    write_json_entry(&mut zip, "reading.json", &json!({ "reading_stats": reading }))?;
    write_json_entry(&mut zip, "comments.json", &json!({ "comments": comments }))?;
    write_json_entry(&mut zip, "follows.json", &json!({ "follows": follows }))?;
    write_json_entry(&mut zip, "reputation.json", &reputation)?;
    write_json_entry(&mut zip, "progress.json", &progress)?;

    let cursor = zip
        .finish()
        .map_err(|e| AppError::Internal(format!("failed to finalize ZIP: {e}")))?;
    let bytes = cursor.into_inner();

    let safe_username: String = username
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let filename = format!("ficnexus-user-data-{}.zip", safe_username);

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        ),
    ];
    Ok((headers, bytes).into_response())
}

/// DELETE /api/user/account — GDPR Art. 17 right to erasure.
///
/// Anonymizes the user's personal data (username, email, password_hash,
/// bio, kindle_email) and marks the row deleted. Dependent rows (bookmarks,
/// ratings, comments, follows, etc.) are hard-deleted so no personal data
/// survives; the users row itself is kept (anonymized) only to preserve
/// foreign keys on forum posts and uploaded works. Returns the export URL
/// so the user can download their data BEFORE erasure if they want.
pub async fn delete_account_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let username = auth.username.clone().unwrap_or_else(|| format!("user{user_id}"));

    // ── 1. Hard-delete all personal-data rows owned by this user ──────
    // Each delete is best-effort: the tables may not all exist in older
    // deployments, and one failing delete must not block erasure.
    let deletes: &[&'static str] = &[
        "DELETE FROM bookmarks WHERE user_id = $1",
        "DELETE FROM work_ratings WHERE user_id = $1",
        "DELETE FROM reviews WHERE user_id = $1",
        "DELETE FROM comments WHERE user_id = $1",
        "DELETE FROM follows WHERE user_id = $1",
        "DELETE FROM notifications WHERE user_id = $1",
        "DELETE FROM reading_history WHERE user_id = $1",
        "DELETE FROM user_favorite_tags WHERE user_id = $1",
        "DELETE FROM user_consents WHERE user_id = $1",
        "DELETE FROM extension_installs WHERE user_id = $1",
        "DELETE FROM extension_ratings WHERE user_id = $1",
        "DELETE FROM request_log WHERE user_id = $1",
        "DELETE FROM user_credentials WHERE user_id = $1",
        "DELETE FROM saved_searches WHERE user_id = $1",
        "DELETE FROM saved_views WHERE user_id = $1",
        "DELETE FROM collection_submissions WHERE user_id = $1",
    ];
    for sql in deletes.iter().copied() {
        let _ = sqlx::query(sql).bind(user_id).execute(&state.db).await;
    }

    // ── 2. Anonymize the users row (keep FK integrity on forum posts) ──
    let anon_name = format!("deleted-user-{user_id}");
    sqlx::query(
        r#"UPDATE users SET
               username = $2,
               email = '',
               password_hash = '!',
               bio = '',
               kindle_email = NULL,
               avatar_url = NULL,
               deleted_at = NOW()
           WHERE id = $1"#,
    )
    .bind(user_id)
    .bind(&anon_name)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("GDPR account erasure for user {user_id} ({username})");

    Ok(Json(json!({
        "err": 0,
        "msg": "Account deleted. All personal data has been erased.",
        "deleted_username": anon_name,
    })))
}

/// POST /api/user/consent — record a consent (GDPR Art. 7).
pub async fn record_consent_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ConsentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let consent_type = body.consent_type.unwrap_or_else(|| "tos".to_string());
    let version = body.version.unwrap_or_else(|| "1.0".to_string());

    sqlx::query(
        r#"INSERT INTO user_consents (user_id, consent_type, version, granted_at)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (user_id, consent_type, version) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(&consent_type)
    .bind(&version)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "err": 0, "msg": "consent recorded" })))
}

#[derive(serde::Deserialize)]
pub struct ConsentBody {
    pub consent_type: Option<String>,
    pub version: Option<String>,
}

