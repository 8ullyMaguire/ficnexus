//! Trust API — per-user trust status + admin trust management + the weekly
//! moderation digest endpoint.

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::trust::{self, TRUST_NAMES, TlMetrics};

/// GET /api/me/trust — the caller's trust level, metrics, and what is needed
/// to reach the next level.
pub async fn my_trust(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let level = trust::fetch_trust_level(&state.db, Some(uid)).await;
    let metrics = trust::load_or_compute_metrics(&state.db, uid).await?;

    let hint = next_level(&metrics, level);
    Ok(Json(json!({
        "level": level,
        "level_name": TRUST_NAMES[level.clamp(0, 6) as usize],
        "metrics": metrics,
        "next_level": hint,
        "publish_allowed": level >= trust::PUBLISH_MIN_TRUST,
        "resolve_allowed": level >= trust::RESOLVE_MIN_TRUST,
    })))
}

fn next_level(_metrics: &TlMetrics, level: i16) -> Value {
    match level {
        0 => json!({ "target": 1, "hint": "read 5+ distinct works and 30k+ words" }),
        1 => {
            json!({ "target": 2, "hint": "be active 5+ days, read 100k+ words, and post once in the forum" })
        }
        2 => {
            json!({ "target": 3, "hint": "be consistently active (15+ days), 100+ works read, 10+ forum posts" })
        }
        3 => json!({ "target": 4, "hint": "400k+ words, 500+ works read, 50+ forum posts" }),
        4 => {
            json!({ "target": 5, "hint": "TL5 (Community Moderator) is granted from the weekly digest by an admin", "staff": true })
        }
        _ => json!({ "target": null, "hint": "Top trust level." }),
    }
}

/// GET /api/admin/trust — distribution + per-user snapshot (role >= 10).
pub async fn admin_trust(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin only".into()));
    }
    let distribution: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT trust_level, COUNT(*) FROM users GROUP BY trust_level ORDER BY trust_level",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(json!({ "distribution": distribution })))
}

#[derive(Deserialize)]
pub struct SetTrustBody {
    pub level: i16,
    pub reason: Option<String>,
}

/// PUT /api/admin/trust/{id} — set a user's trust level (role >= 10). The
/// weekly digest path nudges admins here for TL5 confirmations.
pub async fn admin_set_trust(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
    Json(body): Json<SetTrustBody>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin only".into()));
    }
    let reason = body.reason.unwrap_or_else(|| "admin override".into());
    let level = trust::set_trust_level(
        &state.db,
        user_id,
        body.level,
        &reason,
        auth.user_id,
        auth.username.clone(),
    )
    .await?;
    Ok(Json(
        json!({ "ok": true, "user_id": user_id, "level": level }),
    ))
}

/// GET /api/admin/digest — render the weekly moderation digest as JSON for
/// the in-app admin page (role >= 10).
pub async fn admin_digest(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin only".into()));
    }
    let contested: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, target_type, status, auto_status FROM user_reports
         WHERE auto_status IN ('auto_hidden','needs_admin') OR status = 'open'
         ORDER BY created_at DESC LIMIT 40",
    )
    .fetch_all(&state.db)
    .await?;

    let (spam_auto, spam_total): (i64, i64) = sqlx::query_as(
        "SELECT
           COUNT(*) FILTER (WHERE auto_status IN ('auto_hidden','needs_admin')),
           COUNT(*)
         FROM user_reports WHERE created_at > NOW() - interval '7 days'",
    )
    .fetch_one(&state.db)
    .await?;

    let events: Vec<(i32, i16, i16, String, String)> = sqlx::query_as(
        "SELECT te.user_id, te.from_level, te.to_level, te.reason, u.username
         FROM trust_events te JOIN users u ON u.id = te.user_id
         WHERE te.created_at > NOW() - interval '7 days'
         ORDER BY te.created_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "contested_reports": contested,
        "spam_auto_hidden_7d": spam_auto,
        "spam_total_7d": spam_total,
        "trust_events_7d": events,
    })))
}
