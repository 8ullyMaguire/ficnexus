//! API handlers for customization: theme, navigation, and widgets.

use std::sync::Arc;

use axum::{
    extract::State,
    Json,
};
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::customization::CustomizationService;

// ── GET /api/me/theme ────────────────────────────────────────────────────────

/// Return the current user's theme preferences, falling back to defaults
/// for any key the user has not yet set.
pub async fn get_theme(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let theme = CustomizationService::get_theme(&state.db, user_id).await?;
    Ok(Json(theme))
}

// ── PUT /api/me/theme ────────────────────────────────────────────────────────

/// Save theme preferences for the current user.  Accepts a JSON object
/// whose keys are bare theme-property names (e.g. `{"mode":"light"}`);
/// the handler namespaces them under `theme.*` in `user_prefs`.
pub async fn set_theme(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(theme): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    CustomizationService::set_theme(&state.db, user_id, theme).await?;
    Ok(Json(json!({ "ok": true })))
}

// ── GET /api/nav ─────────────────────────────────────────────────────────────

/// Return the navigation items for the current visitor.
///
/// Authenticated users see every feature with a `nav_target` that is
/// enabled for them (or is a default feature).  Anonymous users see
/// only default features.
pub async fn get_nav(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<crate::services::customization::NavItem>>, AppError> {
    let items = CustomizationService::get_nav_items(&state.db, auth.user_id).await?;
    Ok(Json(items))
}

// ── GET /api/widgets ─────────────────────────────────────────────────────────

/// Return the widget registry: every available widget, its metadata, and
/// the minimum rank required to use it.
pub async fn get_widgets(
) -> Result<Json<Vec<crate::services::customization::WidgetInfo>>, AppError> {
    Ok(Json(CustomizationService::available_widgets()))
}
