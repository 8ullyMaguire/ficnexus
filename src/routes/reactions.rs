//! Emoji reactions for works, comments, and chapters.
//!
//! * `POST   /api/reactions/{target_type}/{target_id}/react` — toggle an emoji
//! * `GET    /api/reactions/{target_type}/{target_id}`    — load reactions + viewer state
//!
//! `target_type` ∈ {work, comment, chapter}.  Works use INT4 ids; comments use
//! BIGINT — both fit fine in the BIGINT `target_id` column.

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Allowed emoji reactions.
pub const ALLOWED_REACTIONS: &[&str] = &["👍", "❤️", "😂", "🔥", "👎"];

/// Validate that `target_type` is one of the allowed values.
fn validate_target_type(target_type: &str) -> Result<&str, AppError> {
    match target_type {
        "work" | "comment" | "chapter" => Ok(target_type),
        _ => Err(AppError::BadRequest(
            "target_type must be 'work', 'comment', or 'chapter'".to_string(),
        )),
    }
}

/// POST /api/reactions/{target_type}/{target_id}/react
/// Body: { emoji: "👍" }
/// Toggles the reaction: if the user already reacted with that emoji, removes it.
pub async fn react(
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i64)>,
    auth: AuthUser,
    Json(body): Json<ReactBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = match &auth.user_id {
        Some(uid) => *uid,
        None => return Err(AppError::Unauthorized("login required".to_string())),
    };
    let _tt = validate_target_type(&target_type)?;
    let emoji = body.emoji.trim();
    if !ALLOWED_REACTIONS.contains(&emoji) {
        return Err(AppError::BadRequest("invalid emoji".to_string()));
    }

    // Toggle: try delete first; if 0 rows, insert.
    let deleted = sqlx::query(
        "DELETE FROM reactions WHERE target_type = $1 AND target_id = $2 AND user_id = $3 AND emoji = $4",
    )
    .bind(&target_type)
    .bind(target_id)
    .bind(user_id)
    .bind(emoji)
    .execute(&state.db)
    .await?
    .rows_affected();

    if deleted == 0 {
        sqlx::query(
            "INSERT INTO reactions (target_type, target_id, user_id, emoji) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
        )
        .bind(&target_type)
        .bind(target_id)
        .bind(user_id)
        .bind(emoji)
        .execute(&state.db)
        .await?;
    }

    let reactions = load_reactions(&state.db, &target_type, target_id, Some(user_id)).await?;
    Ok(Json(json!({ "err": 0, "reactions": reactions })))
}

/// GET /api/reactions/{target_type}/{target_id}
pub async fn get_reactions(
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i64)>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let _tt = validate_target_type(&target_type)?;
    let viewer = auth.user_id;
    let reactions = load_reactions(&state.db, &target_type, target_id, viewer).await?;
    Ok(Json(json!({ "err": 0, "reactions": reactions })))
}

/// Load all reactions for a target, grouped by emoji, plus the viewer's reactions.
async fn load_reactions(
    db: &sqlx::PgPool,
    target_type: &str,
    target_id: i64,
    viewer: Option<i32>,
) -> Result<Value, AppError> {
    let rows: Vec<(String, i32, String)> = sqlx::query_as(
        "SELECT r.emoji, r.user_id, u.username FROM reactions r \
         JOIN users u ON u.id = r.user_id \
         WHERE r.target_type = $1 AND r.target_id = $2 \
         ORDER BY r.emoji, r.created_at",
    )
    .bind(target_type)
    .bind(target_id)
    .fetch_all(db)
    .await?;

    let mut grouped: std::collections::HashMap<String, Vec<Value>> =
        std::collections::HashMap::new();
    for (emoji, uid, username) in rows {
        grouped
            .entry(emoji)
            .or_default()
            .push(json!({ "user_id": uid, "username": username }));
    }

    let my_reactions: Vec<String> = if let Some(uid) = viewer {
        sqlx::query_scalar(
            "SELECT emoji FROM reactions WHERE target_type = $1 AND target_id = $2 AND user_id = $3",
        )
        .bind(target_type)
        .bind(target_id)
        .bind(uid)
        .fetch_all(db)
        .await?
    } else {
        vec![]
    };

    Ok(json!({ "reactions": grouped, "my_reactions": my_reactions }))
}

#[derive(Deserialize)]
pub struct ReactBody {
    emoji: String,
}
