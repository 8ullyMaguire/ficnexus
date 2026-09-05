//! Work deletion branch + curator delete-proposal queue.
//!
//! `POST /api/works/{url_id}/delete` branches on the caller: a curator or
//! admin (role >= 10) or the work's own uploader deletes the work hard and
//! immediately; any other logged-in user files a `work_delete_proposals`
//! request that a curator or admin resolves via the `/api/curator`
//! work-deletions endpoints below.

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::routes::auth::AuthUser;
use crate::routes::curator_content::require_curator;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct DeleteRequestBody {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveBody {
    /// approve | reject
    pub action: String,
}

/// POST /api/works/{url_id}/delete — delete a work, or queue a deletion
/// request for curator review if the caller is neither a curator nor the
/// uploader.
pub async fn post_delete_request(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
    Json(body): Json<DeleteRequestBody>,
) -> AppResult<Json<Value>> {
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let row =
        sqlx::query("SELECT id, uploader_id FROM works WHERE url_id = $1 AND is_visible = TRUE")
            .bind(&url_id)
            .fetch_optional(&state.db)
            .await?;
    let Some(row) = row else {
        return Err(AppError::NotFound("Work not found".to_string().into()));
    };
    let work_id: i32 = row.get("id");
    let uploader_id: Option<i32> = row.get("uploader_id");

    // Curator or admin (role >= 10) or the work's own uploader deletes now.
    if user.role >= 10 || uploader_id == Some(user_id) {
        sqlx::query("DELETE FROM works WHERE id = $1")
            .bind(work_id)
            .execute(&state.db)
            .await?;

        crate::modlog::record(
            &state.db,
            user.user_id,
            user.username.clone(),
            "delete_work",
            "work",
            &url_id,
            serde_json::json!({
                "effected_immediately": true,
                "actor_level": user.level,
                "actor_role": user.role,
            }),
        )
        .await;

        return Ok(Json(json!({
            "err": 0,
            "msg": "Work deleted",
            "queued": false,
        })));
    }

    // Normal user: queue a proposal for curator review.
    let reason = body.reason.unwrap_or_default();
    let row = sqlx::query(
        "INSERT INTO work_delete_proposals (url_id, reason, proposed_by)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(&url_id)
    .bind(&reason)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to create deletion request: {e}")))?;
    let proposal_id: i64 = row.get("id");

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "propose_delete_work",
        "work",
        &url_id,
        vec![
            ("reason", json!(reason.as_str())),
            ("proposal_id", json!(proposal_id)),
        ],
    )
    .await;

    Ok(Json(json!({
        "err": 0,
        "msg": "Deletion requested for curator review",
        "queued": true,
    })))
}

/// GET /api/curator/work-deletions — list pending deletion requests.
pub async fn list_delete_requests(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;

    let rows = sqlx::query(
        "SELECT p.id, p.url_id, p.reason, p.proposed_by, p.status, p.created_at,
                u.username AS proposed_by_username
         FROM work_delete_proposals p
         LEFT JOIN users u ON u.id = p.proposed_by
         WHERE p.status = 'pending'
         ORDER BY p.created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::BadRequest(format!("failed to list deletion requests: {e}")))?;

    let proposals: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<i64, _>("id"),
                "url_id": r.get::<String, _>("url_id"),
                "reason": r.get::<String, _>("reason"),
                "proposed_by": r.get::<i32, _>("proposed_by"),
                "proposed_by_username": r.get::<Option<String>, _>("proposed_by_username"),
                "status": r.get::<String, _>("status"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "proposals": proposals })))
}

/// POST /api/curator/work-deletions/{id}/resolve — approve or reject a
/// deletion request as a curator or admin. Approving hard-deletes the work.
pub async fn resolve_delete_request(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<ResolveBody>,
) -> AppResult<Json<Value>> {
    require_curator(&auth)?;

    if body.action != "approve" && body.action != "reject" {
        return Err(AppError::BadRequest(
            "action must be approve or reject".to_string(),
        ));
    }

    let row = sqlx::query("SELECT url_id, status FROM work_delete_proposals WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::BadRequest(format!("failed to load deletion request: {e}")))?;
    let Some(row) = row else {
        return Err(AppError::NotFound(format!(
            "deletion request {id} not found"
        )));
    };
    let url_id: String = row.get("url_id");
    let status: String = row.get("status");

    if status != "pending" {
        return Err(AppError::BadRequest(format!(
            "deletion request already {status}"
        )));
    }

    match body.action.as_str() {
        "approve" => {
            sqlx::query("DELETE FROM works WHERE url_id = $1")
                .bind(&url_id)
                .execute(&state.db)
                .await?;
            sqlx::query("UPDATE work_delete_proposals SET status = 'approved' WHERE id = $1")
                .bind(id)
                .execute(&state.db)
                .await?;
            crate::modlog::record(
                &state.db,
                auth.user_id,
                auth.username.clone(),
                "approve_delete_request",
                "work",
                &url_id,
                serde_json::json!({}),
            )
            .await;
        }
        _ => {
            sqlx::query("UPDATE work_delete_proposals SET status = 'rejected' WHERE id = $1")
                .bind(id)
                .execute(&state.db)
                .await?;
            crate::modlog::record(
                &state.db,
                auth.user_id,
                auth.username.clone(),
                "reject_delete_request",
                "work",
                &url_id,
                serde_json::json!({}),
            )
            .await;
        }
    }

    Ok(Json(json!({
        "err": 0,
        "msg": format!("Deletion request {}", body.action),
        "action": body.action,
    })))
}
