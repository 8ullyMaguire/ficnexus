//! In-depth written reviews (feedback rework, program item 1).
//!
//! `reviews` is a separate table from `comments` because a review is a
//! first-class signal: it carries a 1..=5 star rating, an optional title and
//! a longer body, is unique per (user, work) and feeds the rec-engine
//! aggregates (see `crate::db::reviews::work_feedback_signals`).
//!
//! Endpoints:
//!   POST   /api/reviews            — create or upsert (one per user per work)
//!   GET    /api/works/{id}/reviews — public list (non-deleted only, with
//!                                    username; reviews are always kept
//!                                    constructive — same heuristic as
//!                                    comments, see `comments.rs`)
//!   DELETE /api/reviews/{id}       — soft delete (owner or curator)
//!
//! Positive-feedback policy: review bodies that trip the negative heuristic
//! are still stored (soft-hidden via a `constructive`-style column is not
//! needed — we reuse the `deleted_at` soft-delete + the insert-time heuristic
//! returns `constructive` to the client) but the public list only ever shows
//! non-deleted rows, and the heuristic flags are applied on insert.

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// POST /api/reviews — create or update the caller's review for a work.
///
/// One review per user per work (UNIQUE(user_id, work_id)): a second POST
/// upserts (rating/title/body updated, `updated_at` bumped).
#[derive(Debug, Deserialize)]
pub struct ReviewBody {
    pub work_id: i32,
    pub rating: i16,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

pub async fn upsert_review_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if !(1..=5).contains(&body.rating) {
        return Err(AppError::BadRequest("Review rating must be 1..=5 stars".to_string()));
    }

    let title = body.title.as_deref().unwrap_or("").trim().to_string();
    if title.len() > 200 {
        return Err(AppError::BadRequest("Review title too long (max 200 chars)".to_string()));
    }

    let text = body.body.as_deref().unwrap_or("").trim().to_string();
    if text.is_empty() {
        return Err(AppError::BadRequest("Review body cannot be empty".to_string()));
    }
    if text.len() > 8000 {
        return Err(AppError::BadRequest("Review too long (max 8000 chars)".to_string()));
    }

    // Resolve url_id like the other work-scoped social endpoints.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    // Constructive heuristic (same as comments): negative-toned review bodies
    // are still stored but flagged, and never shown by the public list.
    let constructive = crate::routes::comments::constructive_score(&text);

    let (id, created_at, updated_at, constructive): (i64, String, String, bool) =
        sqlx::query_as(
            r#"INSERT INTO reviews (user_id, work_id, url_id, rating, title, body, constructive)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (user_id, work_id) DO UPDATE SET
                   rating = EXCLUDED.rating,
                   title = EXCLUDED.title,
                   body = EXCLUDED.body,
                   constructive = EXCLUDED.constructive,
                   updated_at = NOW()
               RETURNING id, created_at::text, COALESCE(updated_at::text, created_at::text), constructive"#,
        )
        .bind(user_id)
        .bind(body.work_id)
        .bind(&url_id)
        .bind(body.rating)
        .bind(title.as_str())
        .bind(text.as_str())
        .bind(constructive)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "review": {
            "id": id,
            "work_id": body.work_id,
            "rating": body.rating,
            "title": if title.is_empty() { Value::Null } else { Value::String(title) },
            "body": text,
            "constructive": constructive,
            "created_at": created_at,
            "updated_at": updated_at,
        }
    })))
}

/// GET /api/works/{id}/reviews — public review list for a work.
///
/// Only non-deleted reviews are returned, with username. Non-constructive
/// reviews (negative-toned at insert) are also filtered out of the public
/// view — the data stays in the DB for the rec engine.
pub async fn list_reviews_handler(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let rows: Vec<(i64, String, String, i16, Option<String>, String, String, bool)> =
        sqlx::query_as(
            r#"SELECT r.id, u.username, r.body, r.rating, r.title,
                      r.created_at::text, COALESCE(r.updated_at::text, r.created_at::text),
                      r.constructive
               FROM reviews r
               JOIN users u ON u.id = r.user_id
               WHERE r.work_id = $1
                 AND r.deleted_at IS NULL
                 AND r.constructive = TRUE
               ORDER BY r.created_at DESC"#,
        )
        .bind(work_id)
        .fetch_all(&state.db)
        .await?;

    let reviews: Vec<Value> = rows
        .into_iter()
        .map(|(id, username, body, rating, title, created_at, updated_at, constructive)| {
            json!({
                "id": id,
                "work_id": work_id,
                "username": username,
                "rating": rating,
                "title": title,
                "body": body,
                "constructive": constructive,
                "created_at": created_at,
                "updated_at": updated_at,
            })
        })
        .collect();

    let (total,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM reviews
         WHERE work_id = $1 AND deleted_at IS NULL AND constructive = TRUE",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "work_id": work_id,
        "total": total,
        "reviews": reviews,
    })))
}

/// DELETE /api/reviews/{id} — soft-delete a review (owner or curator).
pub async fn delete_review_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(review_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let row: Option<(Option<i32>, i16)> = sqlx::query_as(
        "SELECT r.user_id, u.role FROM reviews r
         LEFT JOIN users u ON u.id = $2
         WHERE r.id = $1",
    )
    .bind(review_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some((Some(uid), _)) if uid == user_id => {}
        Some((_, role)) if role >= 1 => {}
        _ => return Err(AppError::Forbidden("Not authorized".to_string())),
    }

    let updated = sqlx::query(
        "UPDATE reviews SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(review_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound("Review not found".into()));
    }

    Ok(Json(json!({ "err": 0, "msg": "Review deleted" })))
}

#[cfg(test)]
mod tests {
    use crate::routes::comments::{constructive_score, is_negative_comment};

    #[test]
    fn constructive_heuristic_positive() {
        assert!(constructive_score("Loved the pacing, great character work!"));
    }

    #[test]
    fn constructive_heuristic_negative() {
        assert!(!constructive_score("This fic is terrible, the worst thing I ever read"));
        assert!(is_negative_comment("garbage"));
    }

    #[test]
    fn constructive_heuristic_case_insensitive() {
        assert!(!constructive_score("TERRIBLE"));
        assert!(constructive_score("Wonderful"));
    }
}
