use axum::Json;
use axum::extract::{Path, State};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::server::AppState;

/// GET /api/v1/badges — list all badge definitions
pub async fn list_badge_definitions_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let badges = queries::get_badge_definitions(&state.db).await?;

    let items: Vec<Value> = badges
        .into_iter()
        .map(|b| {
            json!({
                "badge_type": b.badge_type,
                "name": b.name,
                "description": b.description,
                "icon": b.icon,
                "category": b.category,
                "threshold": b.threshold,
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "badges": items })))
}

/// GET /api/v1/users/{id}/badges — list a user's earned badges
pub async fn get_user_badges_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let badges = queries::get_user_badges(&state.db, user_id).await?;

    let items: Vec<Value> = badges
        .into_iter()
        .map(|b| {
            json!({
                "id": b.id,
                "badge_type": b.badge_type,
                "earned_at": b.earned_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "badges": items })))
}

/// GET /api/v1/leaderboard/curators/weekly
pub async fn leaderboard_weekly_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let entries = queries::get_weekly_leaderboard(&state.db, 50).await?;

    let items: Vec<Value> = entries
        .into_iter()
        .map(|(id, username, score, rank)| {
            json!({
                "user_id": id,
                "username": username,
                "score": score,
                "rank": rank,
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "leaderboard": items })))
}

/// GET /api/v1/leaderboard/curators/monthly
pub async fn leaderboard_monthly_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let entries = queries::get_monthly_leaderboard(&state.db, 50).await?;

    let items: Vec<Value> = entries
        .into_iter()
        .map(|(id, username, score, rank)| {
            json!({
                "user_id": id,
                "username": username,
                "score": score,
                "rank": rank,
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "leaderboard": items })))
}
