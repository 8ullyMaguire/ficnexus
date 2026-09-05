//! User format preference endpoints.
//!
//! Issue #4: users can select preferred download formats.
//! - GET /api/user/preferences/formats → {"formats": [...], "available": [...]}
//! - PUT /api/user/preferences/formats → validates + saves

use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

use crate::db::queries::{save_user_format_preferences, get_user_format_preferences, ALLOWED_FORMATS};
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Request body for updating format preferences.
#[derive(Debug, Deserialize)]
pub struct FormatPreferencesBody {
    pub formats: Vec<String>,
}

/// GET /api/user/preferences/formats — get current user's preferred formats.
pub async fn get_format_preferences(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| {
        AppError::BadRequest("authentication required".into())
    })?;

    let formats = get_user_format_preferences(&state.db, user_id).await?;

    let available: Vec<&str> = ALLOWED_FORMATS.to_vec();

    Ok(Json(serde_json::json!({
        "err": 0,
        "formats": formats,
        "available": available,
    })))
}

/// PUT /api/user/preferences/formats — update format preferences.
pub async fn update_format_preferences(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<FormatPreferencesBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| {
        AppError::BadRequest("authentication required".into())
    })?;

    save_user_format_preferences(&state.db, user_id, &body.formats).await?;

    Ok(Json(serde_json::json!({
        "err": 0,
        "formats": body.formats,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_formats_contains_epub() {
        assert!(ALLOWED_FORMATS.contains(&"epub"));
    }

    #[test]
    fn allowed_formats_contains_common_formats() {
        for fmt in &["epub", "pdf", "mobi", "html"] {
            assert!(ALLOWED_FORMATS.contains(fmt), "missing {fmt}");
        }
    }
}
