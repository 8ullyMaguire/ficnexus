//! Modlog routes: transparent moderation log, readable by ANY logged-in user.

use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// GET /api/modlog?limit=50&action=ban_user
/// Any logged-in user can read the moderation log (transparency).
pub async fn modlog_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<Value>,
) -> Result<Json<Value>, AppError> {
    crate::modlog::require_logged_in(&auth)?;

    let limit = params
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(50)
        .clamp(1, 200);
    let action_filter = params
        .get("action")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let entries = crate::modlog::list(&state.db, limit, action_filter.as_deref())
        .await
        .map_err(|e| AppError::Internal(format!("modlog query failed: {e}")))?;

    let items: Vec<Value> = entries
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id,
                "actor_id": e.actor_id,
                "actor_username": e.actor_username,
                "action": e.action,
                "target_type": e.target_type,
                "target_id": e.target_id,
                "details": e.details,
                "created_at": e.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "entries": items })))
}
