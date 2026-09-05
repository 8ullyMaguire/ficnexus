use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateShelfBody {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddToShelfBody {
    pub shelf_id: i32,
    pub work_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct RemoveFromShelfBody {
    pub work_id: i32,
}

/// POST /api/shelves — create a new shelf
pub async fn create_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateShelfBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Shelf name cannot be empty".to_string(),
        ));
    }
    if body.name.len() > 100 {
        return Err(AppError::BadRequest(
            "Shelf name too long (max 100 chars)".to_string(),
        ));
    }

    let shelf = queries::create_shelf(
        &state.db,
        user_id,
        &body.name,
        body.description.as_deref().unwrap_or(""),
        body.is_public.unwrap_or(false),
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "shelf": {
            "id": shelf.id,
            "name": shelf.name,
            "description": shelf.description,
            "is_public": shelf.is_public,
            "sort_order": shelf.sort_order,
            "created_at": shelf.created_at.to_rfc3339(),
        }
    })))
}

/// GET /api/shelves — list all shelves for the authenticated user
pub async fn list_shelves_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let shelves = queries::list_shelves(&state.db, user_id).await?;

    let items: Vec<Value> = shelves
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.name,
                "description": s.description,
                "is_public": s.is_public,
                "sort_order": s.sort_order,
                "created_at": s.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "shelves": items })))
}

/// DELETE /api/shelves/{id} — delete a shelf
pub async fn delete_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let removed = queries::delete_shelf(&state.db, shelf_id, user_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}

/// POST /api/shelves/add — add a work to a shelf
pub async fn add_work_to_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddToShelfBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, body.shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    queries::add_work_to_shelf(&state.db, body.shelf_id, body.work_id).await?;

    Ok(Json(json!({ "err": 0, "msg": "Work added to shelf" })))
}

/// DELETE /api/shelves/{shelf_id}/works/{work_id} — remove a work from a shelf
pub async fn remove_work_from_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((shelf_id, work_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    let removed = queries::remove_work_from_shelf(&state.db, shelf_id, work_id).await?;

    Ok(Json(json!({ "err": 0, "removed": removed })))
}

/// GET /api/shelves/{shelf_id}/works — list works in a shelf
pub async fn list_works_in_shelf_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Verify the shelf belongs to this user
    let shelf = queries::get_shelf(&state.db, shelf_id, user_id).await?;
    if shelf.is_none() {
        return Err(AppError::NotFound("Shelf not found".into()));
    }

    let entries = queries::list_works_in_shelf(&state.db, shelf_id).await?;

    let items: Vec<Value> = entries
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id,
                "shelf_id": e.shelf_id,
                "work_id": e.work_id,
                "added_at": e.added_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "works": items })))
}
