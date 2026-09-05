use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::AppError;
use crate::progression::{rank_title, xp_for_next_level};
use crate::routes::auth::AuthUser;
use crate::server::AppState;

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProgressionInfo {
    pub level: i32,
    pub rank: i32,
    pub rank_title: String,
    pub xp: i64,
    pub xp_to_next_level: i64,
    pub recent_events: Vec<RecentXpEvent>,
}

#[derive(Debug, Serialize)]
pub struct RecentXpEvent {
    pub event_type: String,
    pub xp: i32,
    pub source_ref: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct PrefUpdate {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Serialize)]
pub struct UserPrefResponse {
    pub key: String,
    pub value: Value,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/me/progression
/// Return the current user's level, rank, XP, and recent events.
pub async fn get_progression(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProgressionInfo>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Fetch user stats
    let user_row = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT COALESCE(level, 0), COALESCE(rank, 0), COALESCE(xp, 0) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let (level, rank, xp) = user_row;

    // Recent 20 XP events
    let events = sqlx::query_as::<_, (String, i32, Option<String>, String)>(
        "SELECT event_type, xp, source_ref, created_at::text
         FROM xp_events
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 20",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let recent_events = events
        .into_iter()
        .map(|(event_type, xp, source_ref, created_at)| RecentXpEvent {
            event_type,
            xp,
            source_ref,
            created_at,
        })
        .collect();

    Ok(Json(ProgressionInfo {
        level,
        rank,
        rank_title: rank_title(rank).to_string(),
        xp: xp as i64,
        xp_to_next_level: xp_for_next_level(level),
        recent_events,
    }))
}

/// GET /api/me/prefs
/// Return all user preferences.
pub async fn get_user_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserPrefResponse>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let prefs = sqlx::query_as::<_, (String, Value)>(
        "SELECT key, value FROM user_prefs WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let result = prefs
        .into_iter()
        .map(|(key, value)| UserPrefResponse { key, value })
        .collect();

    Ok(Json(result))
}

/// PUT /api/me/prefs
/// Update user preferences (accepts a JSON array of {key, value} pairs).
pub async fn set_user_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(updates): Json<Vec<PrefUpdate>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    for update in updates {
        sqlx::query(
            "INSERT INTO user_prefs (user_id, key, value) VALUES ($1, $2, $3)
             ON CONFLICT (user_id, key) DO UPDATE SET value = $3",
        )
        .bind(user_id)
        .bind(&update.key)
        .bind(&update.value)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update pref: {e}")))?;
    }

    Ok(Json(json!({"ok": true})))
}

/// GET /api/me/layout/:page
/// Return the user's dashboard layout for a specific page.
pub async fn get_user_layout(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(page): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let row = sqlx::query_as::<_, (Value,)>(
        "SELECT layout FROM user_layouts WHERE user_id = $1 AND page = $2",
    )
    .bind(user_id)
    .bind(&page)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some((layout,)) => Ok(Json(layout)),
        None => Ok(Json(json!({}))),
    }
}

/// PUT /api/me/layout/:page
/// Save the user's dashboard layout for a specific page.
pub async fn set_user_layout(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(page): Path<String>,
    Json(layout): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    sqlx::query(
        "INSERT INTO user_layouts (user_id, page, layout, updated_at)
         VALUES ($1, $2, $3, now())
         ON CONFLICT (user_id, page) DO UPDATE SET layout = $3, updated_at = now()",
    )
    .bind(user_id)
    .bind(&page)
    .bind(&layout)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(format!("Failed to save layout: {e}")))?;

    Ok(Json(json!({"ok": true})))
}

/// GET /api/me/views
/// Return all saved views for the current user.
pub async fn get_user_views(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Value>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let views = sqlx::query_as::<_, (i32, String, Value, bool)>(
        "SELECT id, name, query, pinned FROM user_views WHERE user_id = $1 ORDER BY name",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let result = views
        .into_iter()
        .map(|(id, name, query, pinned)| {
            json!({
                "id": id,
                "name": name,
                "query": query,
                "pinned": pinned
            })
        })
        .collect();

    Ok(Json(result))
}

/// POST /api/me/views
/// Create a new saved view.
pub async fn create_user_view(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing 'name' field".into()))?;

    let query = body
        .get("query")
        .cloned()
        .ok_or_else(|| AppError::BadRequest("Missing 'query' field".into()))?;

    let pinned = body
        .get("pinned")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let row = sqlx::query_as::<_, (i32,)>(
        "INSERT INTO user_views (user_id, name, query, pinned)
         VALUES ($1, $2, $3, $4)
         RETURNING id",
    )
    .bind(user_id)
    .bind(name)
    .bind(&query)
    .bind(pinned)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(format!("Failed to create view: {e}")))?;

    Ok(Json(json!({
        "id": row.0,
        "name": name,
        "query": query,
        "pinned": pinned
    })))
}
