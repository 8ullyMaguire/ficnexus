//! Auto-tag admin routes — trigger the auto-tagger and review its queue.
//!
//! - `POST /api/admin/auto-tag`  — run [`crate::services::auto_tagger::recommend_tags`]
//!   for a `url_id`; machine-suggested tags above the threshold are inserted
//!   with `is_machine_suggested = TRUE` and land in the review queue.
//! - `POST /api/admin/auto-tag/backfill` — embed the canonical freeform tag
//!   corpus into `tag_embeddings` (idempotent; call once after deploying).
//! - `GET  /api/admin/auto-tag/queue` — machine-suggested tags pending review
//!   (`is_machine_suggested = TRUE AND reviewed_at IS NULL`).
//! - `POST /api/admin/auto-tag/approve/{url_id}/{tag_id}` — promote a
//!   suggestion to a regular tag (flag off, bump score by the similarity).
//! - `POST /api/admin/auto-tag/dismiss/{url_id}/{tag_id}` — delete the
//!   suggestion row.
//!
//! All handlers gate on `user.role >= 10` (same guard as `src/routes/admin.rs`).

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::auto_tagger;

/// Request body for triggering the auto-tagger on a single fic.
#[derive(Debug, Deserialize)]
pub struct AutoTagRequest {
    pub url_id: String,
}

/// POST /api/admin/auto-tag — recommend (and insert) machine-suggested
/// tags for a fic, then report what was suggested.
pub async fn auto_tag_fic(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<AutoTagRequest>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let url_id = req.url_id.trim().to_string();
    if url_id.is_empty() {
        return Err(AppError::BadRequest("url_id is required".to_string()));
    }

    let suggestions = match auto_tagger::recommend_tags(&state.db, &state.ollama, &url_id).await {
        Ok(s) => s,
        Err(auto_tagger::AutoTaggerError::NoContent(msg)) => {
            return Err(AppError::BadRequest(msg));
        }
        Err(auto_tagger::AutoTaggerError::Embed(err)) => {
            tracing::warn!("auto-tag: embedding failed for {url_id}: {err}");
            return Err(AppError::Internal("embedding failed (is Ollama up?)".into()));
        }
        Err(auto_tagger::AutoTaggerError::Db(err)) => return Err(AppError::Database(err.to_string())),
    };

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "suggested": suggestions.iter().map(|s| json!({
            "tag_id": s.tag_id,
            "tag_name": s.tag_name,
            "tag_type_id": s.tag_type_id,
            "similarity": s.similarity,
            "machine_suggested": true,
        })).collect::<Vec<_>>(),
    })))
}

/// POST /api/admin/auto-tag/backfill — embed canonical freeform tags that
/// lack an embedding yet (idempotent). Call once after deploying 015.
pub async fn auto_tag_backfill(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let embedded = match auto_tagger::backfill_tag_embeddings(&state.db, &state.ollama).await {
        Ok(n) => n,
        Err(auto_tagger::AutoTaggerError::Embed(err)) => {
            tracing::warn!("auto-tag backfill: embedding failed: {err}");
            return Err(AppError::Internal("embedding failed (is Ollama up?)".into()));
        }
        Err(auto_tagger::AutoTaggerError::Db(err)) => return Err(AppError::Database(err.to_string())),
        Err(auto_tagger::AutoTaggerError::NoContent(_)) => unreachable!("backfill has no NoContent path"),
    };

    Ok(Json(json!({
        "err": 0,
        "embedded": embedded,
        "msg": format!("embedded {embedded} tag(s); re-run to continue"),
    })))
}

/// GET /api/admin/auto-tag/queue — machine-suggested tags pending review.
pub async fn auto_tag_queue(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let rows = sqlx::query_as::<_, (String, String, i32, String, i16, i16, Option<chrono::DateTime<chrono::Utc>>)>(
        r#"
        SELECT ft.url_id, fi.title, ft.tag_id, t.name, ft.score, t.tag_type_id, ft.created_at
        FROM fic_tags ft
        JOIN fic_info fi ON fi.id = ft.url_id
        JOIN tags t ON t.id = ft.tag_id
        WHERE ft.is_machine_suggested = TRUE AND ft.reviewed_at IS NULL
        ORDER BY ft.created_at DESC
        LIMIT 500
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_tags WHERE is_machine_suggested = TRUE AND reviewed_at IS NULL",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "total": total.0,
        "items": rows.into_iter().map(|(url_id, title, tag_id, tag_name, score, tag_type_id, created)| json!({
            "url_id": url_id,
            "title": title,
            "tag_id": tag_id,
            "tag_name": tag_name,
            "score": score,
            "tag_type_id": tag_type_id,
            "created": created.map(|c| c.to_rfc3339()),
        })).collect::<Vec<_>>(),
    })))
}

/// POST /api/admin/auto-tag/approve/{url_id}/{tag_id} — promote a machine
/// suggestion to a regular tag: clear the flag, mark it reviewed, and bump
/// its score by the original similarity (0-100) so it ranks like a scrape.
pub async fn auto_tag_approve(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((url_id, tag_id)): Path<(String, i32)>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let updated = sqlx::query(
        r#"
        UPDATE fic_tags
        SET is_machine_suggested = FALSE,
            reviewed_at = NOW(),
            score = score + GREATEST(score, 10)
        WHERE url_id = $1 AND tag_id = $2 AND is_machine_suggested = TRUE AND reviewed_at IS NULL
        "#,
    )
    .bind(&url_id)
    .bind(tag_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if updated == 0 {
        return Err(AppError::NotFound(
            "no pending machine-suggested tag with this url_id/tag_id".into(),
        ));
    }

    Ok(Json(json!({ "err": 0, "msg": "suggestion approved", "url_id": url_id, "tag_id": tag_id })))
}

/// POST /api/admin/auto-tag/dismiss/{url_id}/{tag_id} — delete a machine
/// suggestion from the fic (rejected by the reviewer).
pub async fn auto_tag_dismiss(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((url_id, tag_id)): Path<(String, i32)>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let deleted = sqlx::query(
        r#"
        DELETE FROM fic_tags
        WHERE url_id = $1 AND tag_id = $2 AND is_machine_suggested = TRUE AND reviewed_at IS NULL
        "#,
    )
    .bind(&url_id)
    .bind(tag_id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if deleted == 0 {
        return Err(AppError::NotFound(
            "no pending machine-suggested tag with this url_id/tag_id".into(),
        ));
    }

    Ok(Json(json!({ "err": 0, "msg": "suggestion dismissed", "url_id": url_id, "tag_id": tag_id })))
}
