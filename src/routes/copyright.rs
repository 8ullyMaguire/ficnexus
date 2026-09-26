//! DMCA / copyright takedown endpoints.
//!
//! * `POST /api/copyright/notice`  — public: rights holder submits a notice
//! * `GET  /api/admin/copyright/notices`        — admin: list pending notices
//! * `POST /api/admin/copyright/notices/{id}/action` — admin: blacklist work
//! * `POST /api/admin/copyright/notices/{id}/reject` — admin: reject notice
//!
//! When a notice is actioned, the work's url_id is inserted into
//! `fic_blacklist` with reason `dmca`; the export handler already refuses
//! blacklisted works (see `src/routes/export.rs`), so the work becomes
//! undownloadable immediately.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct NoticeBody {
    pub work_url_id: String,
    #[serde(default)]
    pub work_title: String,
    pub claimant_name: String,
    pub claimant_email: String,
    #[serde(default)]
    pub reason: String,
}

/// POST /api/copyright/notice — submit a takedown notice (public).
pub async fn submit_notice(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NoticeBody>,
) -> Result<Json<Value>, AppError> {
    if body.work_url_id.trim().is_empty()
        || body.claimant_name.trim().is_empty()
        || body.claimant_email.trim().is_empty()
    {
        return Err(AppError::BadRequest(
            "work_url_id, claimant_name, and claimant_email are required".into(),
        ));
    }

    // Basic spam guard: notice body is small, so rate limit by IP is enough.
    // (The tiered limiter isn't wired into this handler; a simple daily cap
    // via the notices table itself prevents floods.)
    let recent: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM copyright_notices WHERE created_at > NOW() - INTERVAL '1 day'",
    )
    .fetch_one(&state.db)
    .await?;
    if recent.0 > 500 {
        return Err(AppError::RateLimited(3600));
    }

    sqlx::query(
        r#"INSERT INTO copyright_notices
               (work_url_id, work_title, claimant_name, claimant_email, reason)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(&body.work_url_id)
    .bind(&body.work_title)
    .bind(&body.claimant_name)
    .bind(&body.claimant_email)
    .bind(&body.reason)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "msg": "Notice received. We review takedown requests promptly.",
    })))
}

/// GET /api/admin/copyright/notices — list notices (admin only).
pub async fn list_notices(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&auth, &state)?;
    let rows: Vec<Value> = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        r#"SELECT id, work_url_id, work_title, claimant_name, claimant_email,
                   reason, status, admin_notes, created_at, resolved_at
           FROM copyright_notices
           ORDER BY created_at DESC
           LIMIT 200"#,
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(
        |(
            id,
            work_url_id,
            work_title,
            claimant_name,
            claimant_email,
            reason,
            status,
            admin_notes,
            created_at,
            resolved_at,
        )| {
            json!({
                "id": id,
                "work_url_id": work_url_id,
                "work_title": work_title,
                "claimant_name": claimant_name,
                "claimant_email": claimant_email,
                "reason": reason,
                "status": status,
                "admin_notes": admin_notes,
                "created_at": created_at.to_rfc3339(),
                "resolved_at": resolved_at.map(|d| d.to_rfc3339()),
            })
        },
    )
    .collect();

    Ok(Json(json!({ "err": 0, "notices": rows })))
}

/// POST /api/admin/copyright/notices/{id}/action — blacklist the work.
pub async fn action_notice(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&auth, &state)?;
    let work_url_id: Option<String> = sqlx::query_scalar(
        "SELECT work_url_id FROM copyright_notices WHERE id = $1 AND status = 'pending'",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let Some(work_url_id) = work_url_id else {
        return Err(AppError::NotFound(
            "Notice not found or already resolved".into(),
        ));
    };

    // Mark the notice actioned.
    sqlx::query(
        "UPDATE copyright_notices SET status = 'actioned', resolved_at = NOW() WHERE id = $1",
    )
    .bind(id)
    .execute(&state.db)
    .await?;

    // Blacklist the work (reason 4 = dmca; export handler blocks it).
    sqlx::query(
        r#"INSERT INTO fic_blacklist (url_id, reason)
           VALUES ($1, 4)
           ON CONFLICT (url_id) DO UPDATE SET reason = 4"#,
    )
    .bind(&work_url_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    crate::modlog::record(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "dmca_action",
        "fic",
        &work_url_id,
        json!({ "notice_id": id }),
    )
    .await;

    Ok(Json(
        json!({ "err": 0, "msg": "Work blacklisted for copyright", "work_url_id": work_url_id }),
    ))
}

/// POST /api/admin/copyright/notices/{id}/reject — reject the notice.
pub async fn reject_notice(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&auth, &state)?;
    let n = sqlx::query(
        "UPDATE copyright_notices SET status = 'rejected', resolved_at = NOW() WHERE id = $1 AND status = 'pending'",
    )
    .bind(id)
    .execute(&state.db)
    .await?;
    if n.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Notice not found or already resolved".into(),
        ));
    }
    Ok(Json(json!({ "err": 0, "msg": "Notice rejected" })))
}
