use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::db::models::Feature;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

// ── Response types ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FeatureWithStatus {
    #[serde(flatten)]
    pub feature: Feature,
    pub unlocked: bool,
    pub enabled: bool,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// GET /api/features
/// List all features with the current user's unlock/enable status.
pub async fn list_features(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeatureWithStatus>>, AppError> {
    let user_id = auth.user_id.unwrap_or(0);

    // Fetch all features
    let features = sqlx::query_as::<_, Feature>("SELECT * FROM features ORDER BY sort_hint")
        .fetch_all(&state.db)
        .await?;

    // Fetch user's enabled features
    let enabled_ids: Vec<i32> = sqlx::query_scalar(
        "SELECT feature_id FROM user_features WHERE user_id = $1 AND enabled = true",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    // Fetch user's unlocked features
    let unlocked_ids: Vec<i32> =
        sqlx::query_scalar("SELECT feature_id FROM user_features WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;

    let features_with_status = features
        .into_iter()
        .map(|f| FeatureWithStatus {
            unlocked: unlocked_ids.contains(&f.id),
            enabled: enabled_ids.contains(&f.id),
            feature: f,
        })
        .collect();

    Ok(Json(features_with_status))
}

/// GET /api/features/available
/// Return features the user has unlocked but not enabled.
pub async fn available_features(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeatureWithStatus>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Fetch features where user has unlocked but not enabled
    let feature_ids: Vec<i32> = sqlx::query_scalar(
        "SELECT feature_id FROM user_features WHERE user_id = $1 AND enabled = false",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    if feature_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let features = sqlx::query_as::<_, Feature>(
        "SELECT * FROM features WHERE id = ANY($1) ORDER BY sort_hint",
    )
    .bind(&feature_ids)
    .fetch_all(&state.db)
    .await?;

    let features_with_status = features
        .into_iter()
        .map(|f| FeatureWithStatus {
            unlocked: true,
            enabled: false,
            feature: f,
        })
        .collect();

    Ok(Json(features_with_status))
}

/// POST /api/features/:slug/enable
/// Enable an unlocked feature for the current user.
pub async fn enable_feature(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Look up the feature
    let feature = sqlx::query_as::<_, Feature>("SELECT * FROM features WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Feature '{slug}' not found")))?;

    // Check if unlocked
    let uf = sqlx::query_as::<_, crate::db::models::UserFeature>(
        "SELECT * FROM user_features WHERE user_id = $1 AND feature_id = $2",
    )
    .bind(user_id)
    .bind(feature.id)
    .fetch_optional(&state.db)
    .await?;

    let _uf = uf.ok_or_else(|| AppError::Forbidden("Feature not unlocked".into()))?;

    // Enable it
    sqlx::query(
        "UPDATE user_features SET enabled = true, enabled_at = now() WHERE user_id = $1 AND feature_id = $2",
    )
    .bind(user_id)
    .bind(feature.id)
    .execute(&state.db)
    .await?;

    // Recompute unlocks to catch features gated by this one
    crate::services::achievements::recompute_unlocks(&state.db, user_id).await?;

    Ok(Json(json!({"ok": true, "slug": slug})))
}

/// POST /api/features/:slug/disable
/// Disable a feature if it is revocable.
pub async fn disable_feature(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Look up the feature
    let feature = sqlx::query_as::<_, Feature>("SELECT * FROM features WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Feature '{slug}' not found")))?;

    if !feature.is_revocable {
        return Err(AppError::Forbidden("Feature cannot be disabled".into()));
    }

    sqlx::query("UPDATE user_features SET enabled = false WHERE user_id = $1 AND feature_id = $2")
        .bind(user_id)
        .bind(feature.id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({"ok": true, "slug": slug})))
}

/// PUT /api/admin/features/:slug
/// Admin-only: update a feature's gate settings, description, etc.
pub async fn update_feature(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Check trust >= 4
    let trust: i16 = sqlx::query_scalar("SELECT COALESCE(trust, 0) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .unwrap_or(0);

    if trust < 4 {
        return Err(AppError::Forbidden(
            "Admin access required (trust >= 4)".into(),
        ));
    }

    // Look up the feature
    let feature = sqlx::query_as::<_, Feature>("SELECT * FROM features WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Feature '{slug}' not found")))?;

    // Build dynamic UPDATE — only touch fields present in the body
    let gate_type = body
        .get("gate_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&feature.gate_type);
    let gate_value = body
        .get("gate_value")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(feature.gate_value);
    let description = body
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(&feature.description);
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&feature.name);
    let is_default = body
        .get("is_default")
        .and_then(|v| v.as_bool())
        .unwrap_or(feature.is_default);
    let is_revocable = body
        .get("is_revocable")
        .and_then(|v| v.as_bool())
        .unwrap_or(feature.is_revocable);

    sqlx::query(
        "UPDATE features
         SET gate_type = $2, gate_value = $3, description = $4, name = $5,
             is_default = $6, is_revocable = $7
         WHERE id = $1",
    )
    .bind(feature.id)
    .bind(gate_type)
    .bind(gate_value)
    .bind(description)
    .bind(name)
    .bind(is_default)
    .bind(is_revocable)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(format!("Failed to update feature: {e}")))?;

    Ok(Json(json!({ "ok": true, "slug": slug })))
}

/// GET /api/admin/features/stats
/// Admin-only: count of users with each feature enabled=true.
pub async fn feature_stats(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Value>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let trust: i16 = sqlx::query_scalar("SELECT COALESCE(trust, 0) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .unwrap_or(0);

    if trust < 4 {
        return Err(AppError::Forbidden(
            "Admin access required (trust >= 4)".into(),
        ));
    }

    let stats = sqlx::query_as::<_, (String, i64)>(
        "SELECT f.slug, COUNT(uf.user_id) FILTER (WHERE uf.enabled = true)
         FROM features f
         LEFT JOIN user_features uf ON uf.feature_id = f.id
         GROUP BY f.slug
         ORDER BY f.slug",
    )
    .fetch_all(&state.db)
    .await?;

    let result = stats
        .into_iter()
        .map(|(slug, count)| {
            json!({
                "slug": slug,
                "enabled_count": count,
            })
        })
        .collect();

    Ok(Json(result))
}

/// DELETE /api/me/views/:id
/// Delete a saved view belonging to the current user.
pub async fn delete_user_view(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let deleted = sqlx::query("DELETE FROM user_views WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(format!("Failed to delete view: {e}")))?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound("View not found".into()));
    }

    Ok(Json(json!({ "ok": true })))
}

/// PUT /api/me/views/:id/toggle-pin
/// Toggle the pinned state of a saved view.
pub async fn toggle_view_pin(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let row = sqlx::query_as::<_, (bool,)>(
        "UPDATE user_views SET pinned = NOT pinned
         WHERE id = $1 AND user_id = $2
         RETURNING pinned",
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(format!("Failed to toggle pin: {e}")))?;

    match row {
        Some((pinned,)) => Ok(Json(json!({ "id": id, "pinned": pinned }))),
        None => Err(AppError::NotFound("View not found".into())),
    }
}

/// GET /api/admin/features
/// Admin-only: list all features with all users' statuses.
pub async fn admin_features(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeatureWithStatus>>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Check trust >= 4
    let trust: i16 = sqlx::query_scalar("SELECT COALESCE(trust, 0) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .unwrap_or(0);

    if trust < 4 {
        return Err(AppError::Forbidden(
            "Admin access required (trust >= 4)".into(),
        ));
    }

    let features = sqlx::query_as::<_, Feature>("SELECT * FROM features ORDER BY sort_hint")
        .fetch_all(&state.db)
        .await?;

    // For admin, return features without user-specific status
    let result = features
        .into_iter()
        .map(|f| FeatureWithStatus {
            unlocked: false,
            enabled: false,
            feature: f,
        })
        .collect();

    Ok(Json(result))
}
