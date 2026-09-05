use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct UpdateReadingStatusBody {
    pub work_id: i32,
    pub status: String,
    pub current_chapter: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingListQuery {
    pub status: Option<String>,
}

/// GET /api/v1/users/{id}/reading-stats — reading stats
pub async fn get_reading_stats_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let stats = queries::get_user_reading_stats(&state.db, user_id).await?;
    let (total_words, total_works, streak) =
        queries::get_user_reading_aggregate(&state.db, user_id).await?;

    let items: Vec<Value> = stats
        .into_iter()
        .map(|s| {
            json!({
                "work_id": s.work_id,
                "words_read": s.words_read,
                "read_count": s.read_count,
                "last_read_at": s.last_read_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "total_words_read": total_words,
        "total_works_read": total_works,
        "login_streak": streak.unwrap_or(0),
        "recent": items,
    })))
}

/// POST /api/v1/reading/record — record reading progress
pub async fn record_read_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::routes::social::ReadingRecordBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    queries::record_work_read(&state.db, user_id, body.work_id, body.words_read).await?;

    Ok(Json(json!({ "err": 0, "msg": "Reading recorded" })))
}

/// GET /api/v1/users/{id}/streak — login streak
pub async fn get_streak_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let streak = queries::get_login_streak(&state.db, user_id).await?;

    match streak {
        Some(s) => Ok(Json(json!({
            "err": 0,
            "current_streak": s.current_streak,
            "longest_streak": s.longest_streak,
            "last_login_date": s.last_login_date.to_string(),
        }))),
        None => Ok(Json(json!({
            "err": 0,
            "current_streak": 0,
            "longest_streak": 0,
        }))),
    }
}

/// POST /api/reading/status — update reading status for a work
pub async fn update_reading_status_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateReadingStatusBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate status
    match body.status.as_str() {
        "want_to_read" | "reading" | "completed" | "dropped" => {}
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid status: '{}'. Must be one of: want_to_read, reading, completed, dropped",
                body.status
            )));
        }
    }

    queries::update_reading_status(
        &state.db,
        user_id,
        body.work_id,
        &body.status,
        body.current_chapter,
    )
    .await?;

    Ok(Json(json!({ "err": 0, "msg": "Reading status updated" })))
}

/// GET /api/reading/list — get reading list, optionally filtered by status
pub async fn get_reading_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReadingListQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate status filter if provided
    if let Some(ref status) = query.status {
        match status.as_str() {
            "want_to_read" | "reading" | "completed" | "dropped" => {}
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Invalid status filter: '{}'",
                    status
                )));
            }
        }
    }

    let list = queries::get_reading_stats_list(&state.db, user_id, query.status.as_deref()).await?;

    let items: Vec<Value> = list
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "work_id": r.work_id,
                "words_read": r.words_read,
                "read_count": r.read_count,
                "status": r.status,
                "current_chapter": r.current_chapter,
                "last_read_at": r.last_read_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "reading_list": items })))
}

/// Query params for GET /api/reading/history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Body for POST /api/reading/history (record a visit)
#[derive(Debug, Deserialize)]
pub struct HistoryRecordBody {
    pub work_id: i32,
    pub chapter_num: Option<i32>,
}

/// GET /api/reading/history — get the user's reading history (AO3-style per-visit log)
pub async fn get_reading_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let (items, total) = queries::get_reading_history(&state.db, user_id, limit, offset).await?;
    let entries: Vec<Value> = items
        .into_iter()
        .map(|h| {
            json!({
                "id": h.id,
                "work_id": h.work_id,
                "url_id": h.url_id,
                "title": h.title,
                "author": h.author,
                "chapter_num": h.chapter_num,
                "visited_at": h.visited_at.to_rfc3339(),
            })
        })
        .collect();
    Ok(Json(
        json!({ "err": 0, "history": entries, "total": total, "limit": limit, "offset": offset }),
    ))
}

/// POST /api/reading/history — record a visit to a work
pub async fn record_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<HistoryRecordBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    queries::record_read_history(&state.db, user_id, body.work_id, body.chapter_num).await?;
    Ok(Json(json!({ "err": 0, "msg": "Visit recorded" })))
}

/// DELETE /api/reading/history/{id} — delete a single history entry
pub async fn delete_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let removed = queries::delete_read_history(&state.db, user_id, id).await?;
    Ok(Json(json!({ "err": 0, "removed": removed })))
}

/// POST /api/reading/history/clear — clear all history entries for the user
pub async fn clear_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    queries::clear_read_history(&state.db, user_id).await?;
    Ok(Json(json!({ "err": 0, "msg": "History cleared" })))
}
