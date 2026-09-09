//! API handlers for Recipe Builder (P1 extension platform).
//!
//! Routes:
//!   GET    /api/recipes              — list user's recipes
//!   POST   /api/recipes              — create recipe
//!   PUT    /api/recipes/:id          — update recipe
//!   DELETE /api/recipes/:id          — delete recipe
//!   POST   /api/recipes/:id/activate — set as active recipe
//!   GET    /api/recipes/active       — get user's active recipe
//!   GET    /api/recipes/gallery      — browse public recipes
//!   POST   /api/recipes/:id/install  — install from gallery
//!   POST   /api/recipes/:id/publish  — make public

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::recipes::RecipeService;
use crate::services::trust;

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRecipeRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub blend: Value,
    #[serde(default)]
    pub filters: Value,
    #[serde(default)]
    pub boost: Value,
    #[serde(default = "default_curator_prior")]
    pub curator_prior: f32,
}

fn default_curator_prior() -> f32 {
    0.1
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecipeRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub blend: Option<Value>,
    pub filters: Option<Value>,
    pub boost: Option<Value>,
    pub curator_prior: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct GalleryQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub is_public: bool,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct RecipeResponse {
    pub err: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<crate::services::recipes::Recipe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipes: Option<Vec<crate::services::recipes::Recipe>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/recipes — list user's recipes
pub async fn list_recipes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let recipes = RecipeService::list_user(&state.db, user_id).await?;
    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: Some(recipes),
        message: None,
    }))
}

/// POST /api/recipes — create recipe
pub async fn create_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateRecipeRequest>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    if req.name.trim().is_empty() {
        return Err(AppError::BadRequest("Recipe name is required".into()));
    }

    let id = RecipeService::create(
        &state.db,
        user_id,
        &req.name,
        req.description.as_deref(),
        req.blend,
        req.filters,
        req.boost,
    )
    .await?;

    // Fetch and return the created recipe
    let recipes = RecipeService::list_user(&state.db, user_id).await?;
    let recipe = recipes.into_iter().find(|r| r.id == id);

    Ok(Json(RecipeResponse {
        err: 0,
        recipe,
        recipes: None,
        message: Some(format!("Recipe created with id {id}")),
    }))
}

/// PUT /api/recipes/:id — update recipe
pub async fn update_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateRecipeRequest>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Extract description strings before passing to service to avoid borrow issues
    let desc_opt = match req.description {
        Some(Some(s)) => Some(s),
        Some(None) => Some(String::new()),
        None => None,
    };

    RecipeService::update(
        &state.db,
        user_id,
        id,
        req.name.as_deref(),
        desc_opt.as_deref().map(Some),
        req.blend,
        req.filters,
        req.boost,
        req.curator_prior,
    )
    .await?;

    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: None,
        message: Some("Recipe updated".into()),
    }))
}

/// DELETE /api/recipes/:id — delete recipe
pub async fn delete_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    RecipeService::delete(&state.db, user_id, id).await?;

    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: None,
        message: Some("Recipe deleted".into()),
    }))
}

/// POST /api/recipes/:id/activate — set as active recipe
pub async fn activate_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    RecipeService::set_active(&state.db, user_id, id).await?;

    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: None,
        message: Some("Recipe activated".into()),
    }))
}

/// GET /api/recipes/active — get user's active recipe
pub async fn get_active_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let recipe = RecipeService::get_active(&state.db, user_id).await?;
    Ok(Json(RecipeResponse {
        err: 0,
        recipe,
        recipes: None,
        message: None,
    }))
}

/// GET /api/recipes/gallery — browse public recipes
pub async fn browse_gallery(
    State(state): State<Arc<AppState>>,
    Query(q): Query<GalleryQuery>,
) -> Result<Json<RecipeResponse>, AppError> {
    let limit = q.limit.clamp(1, 50);
    let recipes = RecipeService::browse_public(&state.db, limit, q.offset).await?;
    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: Some(recipes),
        message: None,
    }))
}

/// POST /api/recipes/:id/install — install from gallery
pub async fn install_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let new_id = RecipeService::install(&state.db, user_id, id).await?;

    // Return the newly installed recipe
    let recipes = RecipeService::list_user(&state.db, user_id).await?;
    let recipe = recipes.into_iter().find(|r| r.id == new_id);

    Ok(Json(RecipeResponse {
        err: 0,
        recipe,
        recipes: None,
        message: Some("Recipe installed".into()),
    }))
}

/// POST /api/recipes/:id/publish — make public
pub async fn publish_recipe(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(req): Json<PublishRequest>,
) -> Result<Json<RecipeResponse>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Publishing shares the recipe with the whole site, so it sits above the
    // bare-write tier (trust-sandboxed accounts may still draft privately).
    if req.is_public {
        trust::assert_staff_or_min_trust(
            &state.db,
            Some(user_id),
            auth.trust_level,
            trust::PUBLISH_MIN_TRUST,
            "Publishing recipes",
        )
        .await?;
    }

    RecipeService::set_public(&state.db, user_id, id, req.is_public).await?;

    let msg = if req.is_public {
        "Recipe published"
    } else {
        "Recipe unpublished"
    };

    Ok(Json(RecipeResponse {
        err: 0,
        recipe: None,
        recipes: None,
        message: Some(msg.into()),
    }))
}
