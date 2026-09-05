//! Update feed + refresh-fic endpoints (program items 6 & 7).
//!
//! - `GET /api/v1/updates` — works the signed-in user follows, ordered by
//!   `fic_updated DESC`, each with `updated_ago` plus an `is_new` flag when
//!   the fic was updated after the user's last-seen on that follow.
//! - `POST /api/v1/follows/{id}/seen` — marks a follow as seen (called when
//!   the user opens the updates feed, and by the fic page after following).
//! - `POST /api/v1/works/{url_id}/refresh` — re-scrapes the fic from its
//!   source. If `fic_updated` advanced (or chapters/words changed), bumps
//!   `fic_version_bump` (invalidating the export cache) and notifies the
//!   work's followers via the notifications table. The scrape itself is
//!   synchronous: it reuses the same `scraper.lookup` + `find_or_create_work`
//!   ingest path as the export handler, then bumps the version (which forces
//!   the next export to regenerate) — we deliberately do *not* regenerate all
//!   formats here, so the endpoint stays fast.

use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// GET /api/v1/updates?user_id=me — followed works with update info.
pub async fn updates_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let updates = queries::list_followed_work_updates(&state.db, user_id).await?;
    let unseen = queries::count_unseen_followed_updates(&state.db, user_id).await?;

    let now = chrono::Utc::now();
    let items: Vec<Value> = updates
        .into_iter()
        .map(|u| {
            let is_new = match u.last_seen {
                Some(last) => u.fic_updated > last,
                None => true,
            };
            json!({
                "follow_id": u.follow_id,
                "work_id": u.work_id,
                "url_id": u.url_id,
                "title": u.title,
                "author": u.author,
                "words": u.words,
                "chapters": u.chapters,
                "status": u.status,
                "fic_updated": u.fic_updated.to_rfc3339(),
                "updated_ago": relative_ago(now, u.fic_updated),
                "is_new": is_new,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "unseen_count": unseen,
    })))
}

/// POST /api/v1/follows/{id}/seen — mark a follow's last_seen as now.
pub async fn mark_seen_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(follow_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let updated = queries::mark_follow_seen(&state.db, follow_id, user_id).await?;
    Ok(Json(json!({ "err": 0, "updated": updated })))
}

/// POST /api/v1/works/{url_id}/refresh — re-scrape + version bump + notify.
///
/// Returns `{ "err": 0, "status": "ok"|"no_change", "version_bump": N,
/// "notified": M }`. When the re-scrape found no change, status is
/// `no_change` and no bump/notification happens.
pub async fn refresh_fic_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let _user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // The fic must already be known to us (we need its source URL to re-scrape).
    let existing = queries::get_fic_info(&state.db, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {url_id}")))?;

    let source_url = if existing.source.is_empty() {
        return Err(AppError::BadRequest(
            "fic has no source URL to refresh from".to_string(),
        ));
    } else {
        existing.source.clone()
    };

    // Re-scrape via the same ingest path as the export handler. A network /
    // upstream failure is NOT fatal: the endpoint returns `status: "error"`
    // with the message so the UI can show "couldn't reach the source" while
    // keeping the HTTP contract (200 + {err:0}) used by the rest of the API.
    let scraper = match state.scraper_registry.find_specific_or_fff(&source_url) {
        Some(s) => s,
        None => {
            return Ok(Json(json!({
                "err": 0,
                "status": "error",
                "url_id": url_id,
                "msg": format!("unsupported URL: {source_url}"),
            })));
        }
    };

    let meta = match scraper.lookup(&state.http_client, &source_url).await {
        Ok(m) => m,
        Err(e) => {
            return Ok(Json(json!({
                "err": 0,
                "status": "error",
                "url_id": url_id,
                "msg": e.to_string(),
            })));
        }
    };

    let changed = meta.updated > existing.fic_updated.timestamp_millis()
        || meta.chapters != existing.chapters
        || meta.words != existing.words;

    if !changed {
        return Ok(Json(json!({
            "err": 0,
            "status": "no_change",
            "url_id": url_id,
        })));
    }

    // Persist the fresh metadata. The fic keeps its original url_id (the one
    // from the path) — a re-scrape must not create a second source row when
    // the upstream URL format differs.
    let fic_info_row = crate::db::models::FicInfo {
        id: url_id.clone(),
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
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // The work stays the same: reuse the existing work when the fic is already
    // linked; only fall back to auto-merge for fics that were never ingested.
    let work_id = match existing.work_id {
        Some(wid) => wid,
        None => {
            crate::works::find_or_create_work(&state.db, &meta)
                .await?
                .work_id
        }
    };
    // Make sure the source points at the work in either case.
    if queries::get_work_by_source(&state.db, &url_id)
        .await?
        .is_none()
    {
        queries::link_source_to_work(&state.db, &url_id, work_id).await?;
    }

    // Bump the version so the export cache regenerates on next request.
    let bump = queries::get_fic_version_bump(&state.db, &url_id)
        .await?
        .unwrap_or(0)
        + 1;
    sqlx::query(
        "INSERT INTO fic_version_bump (id, value) VALUES ($1, $2)
         ON CONFLICT (id) DO UPDATE SET value = EXCLUDED.value",
    )
    .bind(&url_id)
    .bind(bump)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Notify followers (respects their work_update preference).
    let notified = queries::notify_work_followers(&state.db, work_id, &meta.title).await?;

    Ok(Json(json!({
        "err": 0,
        "status": "ok",
        "url_id": url_id,
        "version_bump": bump,
        "notified": notified,
    })))
}

/// Human "updated X ago" string (mirrors the frontend util, server-side).
fn relative_ago(now: chrono::DateTime<chrono::Utc>, then: chrono::DateTime<chrono::Utc>) -> String {
    let diff = now.signed_duration_since(then);
    if diff.num_seconds() < 60 {
        "less than a minute ago".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{} hours ago", diff.num_hours())
    } else {
        format!("{} days ago", diff.num_days())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_ago_handles_units() {
        let now = chrono::Utc::now();
        assert!(relative_ago(now, now).contains("less than a minute"));
        assert!(relative_ago(now, now - chrono::Duration::minutes(5)).contains("5 minutes"));
        assert!(relative_ago(now, now - chrono::Duration::hours(3)).contains("3 hours"));
        assert!(relative_ago(now, now - chrono::Duration::days(2)).contains("2 days"));
    }
}
