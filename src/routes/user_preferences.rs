//! User preferences, dashboard layouts, and saved views.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct PrefUpdate {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, serde::Serialize)]
pub struct UserPrefResponse {
    pub key: String,
    pub value: Value,
}

/// GET /api/me/prefs — Return all user preferences.
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

/// PUT /api/me/prefs — Update user preferences.
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

/// GET /api/me/layout/{page} — Return the user's dashboard layout for a page.
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

/// PUT /api/me/layout/{page} — Save the user's dashboard layout for a page.
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

/// GET /api/me/views — Return all saved views for the current user.
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

/// POST /api/me/views — Create a new saved view.
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

/// GET /api/me/format-preferences — Return user format preferences.
pub async fn get_format_preferences(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let row = sqlx::query_as::<_, (Value,)>(
        "SELECT value FROM user_prefs WHERE user_id = $1 AND key = 'format'",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    match row {
        Some((value,)) => Ok(Json(value)),
        None => Ok(Json(json!({}))),
    }
}

/// PUT /api/me/format-preferences — Update user format preferences.
pub async fn update_format_preferences(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    sqlx::query("INSERT INTO user_prefs (user_id, key, value) VALUES ($1, 'format', $3) ON CONFLICT (user_id, key) DO UPDATE SET value = $3")
        .bind(user_id)
        .bind(&body)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({"ok": true})))
}
