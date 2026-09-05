//! Reading lists (bundles): user-curated ordered lists of works with
//! per-item blurbs. Lists are private by default; the owner can flip
//! is_public to share a read-only view.
//!
//! Endpoints:
//! * POST /api/lists          — create a list (auth)
//! * GET  /api/lists          — list my lists (auth)
//! * GET  /api/lists/{id}     — shared public view, or the owner's view
//! * PATCH /api/lists/{id}    — update title/description/is_public (owner)
//! * DELETE /api/lists/{id}   — soft delete (owner or curator >=5)
//! * POST /api/lists/{id}/items — append {work_id, blurb?} at next position
//! * DELETE /api/lists/{id}/items/{work_id} — remove an item (owner/curator)

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
pub struct CreateListBody {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddItemBody {
    pub work_id: i32,
    #[serde(default)]
    pub blurb: Option<String>,
}

/// Require a logged-in user id (project convention: HTTP 400 {"err":401}).
fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}

fn sanitize_title(title: &str) -> Result<String, AppError> {
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::BadRequest(
            "List title cannot be empty".to_string(),
        ));
    }
    if t.chars().count() > 200 {
        return Err(AppError::BadRequest(
            "List title too long (max 200 chars)".to_string(),
        ));
    }
    Ok(t.to_string())
}

/// POST /api/lists — create a reading list
pub async fn create_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateListBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let title = sanitize_title(&body.title)?;
    let description = body.description.unwrap_or_default();
    if description.chars().count() > 2000 {
        return Err(AppError::BadRequest(
            "Description too long (max 2000 chars)".to_string(),
        ));
    }

    let list = queries::create_reading_list(
        &state.db,
        user_id,
        &title,
        &description,
        body.is_public.unwrap_or(false),
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "list": {
            "id": list.id,
            "user_id": list.user_id,
            "title": list.title,
            "description": list.description,
            "is_public": list.is_public,
            "created_at": list.created_at.to_rfc3339(),
            "updated_at": list.updated_at.to_rfc3339(),
            "item_count": 0,
        }
    })))
}

/// GET /api/lists — list the user's own lists (with item counts)
pub async fn list_lists_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let lists = queries::list_reading_lists(&state.db, user_id).await?;

    let items: Vec<Value> = {
        let mut out = Vec::with_capacity(lists.len());
        for l in lists {
            let count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM reading_list_items WHERE list_id = $1")
                    .bind(l.id)
                    .fetch_one(&state.db)
                    .await?;
            out.push(json!({
                "id": l.id,
                "user_id": l.user_id,
                "title": l.title,
                "description": l.description,
                "is_public": l.is_public,
                "created_at": l.created_at.to_rfc3339(),
                "updated_at": l.updated_at.to_rfc3339(),
                "item_count": count.0,
            }));
        }
        out
    };

    Ok(Json(json!({ "err": 0, "lists": items })))
}

/// GET /api/lists/{id} — shared public view, or the owner's private view.
pub async fn get_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let owner_id = auth.user_id;

    // Owner view when authenticated and the list belongs to them, else the
    // public view (is_public = TRUE, not deleted).
    let list = match owner_id {
        Some(uid) => match queries::get_reading_list(&state.db, list_id, uid, false).await? {
            Some(l) => Some(l),
            None => queries::get_reading_list(&state.db, list_id, uid, true).await?,
        },
        None => queries::get_reading_list(&state.db, list_id, 0, true).await?,
    };

    let list = match list {
        Some(l) => l,
        None => return Err(AppError::NotFound("Reading list not found".into())),
    };

    let items = queries::list_reading_list_items(&state.db, list_id).await?;
    let item_values: Vec<Value> = items
        .into_iter()
        .map(|i| {
            json!({
                "id": i.id,
                "work_id": i.work_id,
                "position": i.position,
                "blurb": i.blurb,
                "title": i.canonical_title,
                "author": i.canonical_author,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "list": {
            "id": list.id,
            "user_id": list.user_id,
            "title": list.title,
            "description": list.description,
            "is_public": list.is_public,
            "created_at": list.created_at.to_rfc3339(),
            "updated_at": list.updated_at.to_rfc3339(),
            "item_count": item_values.len(),
        },
        "items": item_values,
        "is_owner": owner_id == Some(list.user_id),
    })))
}

/// PATCH /api/lists/{id} — update title/description/is_public (owner only)
pub async fn update_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
    Json(body): Json<UpdateListBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let current = queries::get_reading_list(&state.db, list_id, user_id, false).await?;
    let current = match current {
        Some(l) => l,
        None => return Err(AppError::NotFound("Reading list not found".into())),
    };

    let title = match &body.title {
        Some(t) => sanitize_title(t)?,
        None => current.title,
    };
    let description = match &body.description {
        Some(d) => {
            if d.chars().count() > 2000 {
                return Err(AppError::BadRequest(
                    "Description too long (max 2000 chars)".to_string(),
                ));
            }
            d.clone()
        }
        None => current.description,
    };
    let is_public = body.is_public.unwrap_or(current.is_public);

    let updated =
        queries::update_reading_list(&state.db, list_id, user_id, &title, &description, is_public)
            .await?;
    if !updated {
        return Err(AppError::NotFound("Reading list not found".into()));
    }

    Ok(Json(json!({ "err": 0, "msg": "List updated" })))
}

/// DELETE /api/lists/{id} — soft delete (owner or curator >= 5)
pub async fn delete_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let is_curator = auth.role >= 5;

    let removed = queries::delete_reading_list(&state.db, list_id, user_id, is_curator).await?;
    if !removed {
        return Err(AppError::NotFound("Reading list not found".into()));
    }
    Ok(Json(json!({ "err": 0, "removed": true })))
}

/// POST /api/lists/{id}/items — append a work at the next position
pub async fn add_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(list_id): Path<i32>,
    Json(body): Json<AddItemBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    // Owner or curator may add items.
    let list = queries::get_reading_list(&state.db, list_id, user_id, false).await?;
    let is_owner = list.is_some();
    let is_curator = auth.role >= 5;
    if !is_owner && !is_curator {
        return Err(AppError::NotFound("Reading list not found".into()));
    }

    // The work must exist.
    let work = queries::get_work(&state.db, body.work_id).await?;
    if work.is_none() {
        return Err(AppError::NotFound(format!(
            "Work {} not found",
            body.work_id
        )));
    }

    let blurb = body.blurb.unwrap_or_default();
    if blurb.chars().count() > 500 {
        return Err(AppError::BadRequest(
            "Blurb too long (max 500 chars)".to_string(),
        ));
    }

    let added = queries::add_reading_list_item(&state.db, list_id, body.work_id, &blurb).await?;
    if !added {
        return Err(AppError::BadRequest(
            "Work is already in this list".to_string(),
        ));
    }

    let position: (i32,) = sqlx::query_as(
        "SELECT position FROM reading_list_items WHERE list_id = $1 AND work_id = $2",
    )
    .bind(list_id)
    .bind(body.work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "msg": "Work added to list", "position": position.0 }),
    ))
}

/// DELETE /api/lists/{id}/items/{work_id} — remove a work (owner/curator)
pub async fn remove_item_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((list_id, work_id)): Path<(i32, i32)>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let list = queries::get_reading_list(&state.db, list_id, user_id, false).await?;
    let is_owner = list.is_some();
    let is_curator = auth.role >= 5;
    if !is_owner && !is_curator {
        return Err(AppError::NotFound("Reading list not found".into()));
    }

    let removed = queries::remove_reading_list_item(&state.db, list_id, work_id).await?;
    if !removed {
        return Err(AppError::NotFound("Item not in list".into()));
    }
    Ok(Json(json!({ "err": 0, "removed": true })))
}
