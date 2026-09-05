//! Bounty endpoints. Reputation (spendable) on stake; XP (level ladder) untouched.

use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::bounties;

#[derive(Debug, Deserialize)]
pub struct BountyListParams {
    pub target_type: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BountyRow {
    pub id: i32,
    pub creator_id: i32,
    pub target_type: String,
    pub goal_desc: String,
    pub amount: i32,
    pub created_at: String,
}

/// GET /api/bounties — open bounties, optionally filtered by target_type.
pub async fn list_bounties(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BountyListParams>,
) -> Result<Json<Vec<BountyRow>>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200).max(1);
    let rows =
        bounties::list_open_bounties(&state.db, params.target_type.as_deref(), limit).await?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, creator_id, tt, desc, amt, ts)| BountyRow {
                id,
                creator_id,
                target_type: tt,
                goal_desc: desc,
                amount: amt,
                created_at: ts,
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateBountyReq {
    pub target_type: String, // work | series | tag | meta
    pub target_ref: String,  // url_id / series id / tag slug / label
    pub amount: i32,         // rep to stake
    pub goal_desc: Option<String>,
    pub expiry_days: Option<i32>,
}

/// POST /api/bounties — create + stake a bounty (authenticated, rep-gated).
pub async fn create_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<CreateBountyReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let valid_targets = ["work", "series", "tag", "meta"];
    if !valid_targets.contains(&req.target_type.as_str()) {
        return Err(AppError::BadRequest("bad target_type".into()));
    }
    let days = req.expiry_days.unwrap_or(30).clamp(1, 90);
    let pot_id = bounties::create_bounty(
        &state.db,
        user_id,
        &req.target_type,
        &req.target_ref,
        req.amount,
        req.goal_desc.as_deref(),
        days,
    )
    .await?;
    Ok(Json(json!({ "ok": true, "bounty_id": pot_id })))
}

#[derive(Debug, Deserialize)]
pub struct ClaimBountyReq {
    pub claim_ref: String,
}

/// POST /api/bounties/{id}/claim — submit proof of completion.
pub async fn claim_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(pot_id): Path<i32>,
    Json(req): Json<ClaimBountyReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    // record the claimant on the pot so resolve can credit them.
    bounties::claim_bounty_with(&state.db, pot_id, user_id, &req.claim_ref).await?;
    Ok(Json(json!({ "ok": true })))
}

#[derive(Debug, Deserialize)]
pub struct ResolveBountyReq {
    pub action: String,
    pub note: Option<String>,
}

/// POST /api/bounties/{id}/resolve — admin resolves/slashes/cancels.
pub async fn resolve_bounty_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(pot_id): Path<i32>,
    Json(req): Json<ResolveBountyReq>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let resolver_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let valid_actions = ["resolved", "slashed", "cancelled"];
    if !valid_actions.contains(&req.action.as_str()) {
        return Err(AppError::BadRequest("bad action".into()));
    }
    let payout = bounties::resolve_bounty(
        &state.db,
        resolver_id,
        pot_id,
        &req.action,
        req.note.as_deref(),
    )
    .await?;
    Ok(Json(json!({ "ok": true, "payout": payout })))
}

/// POST /api/xp/idle-tick — tiny XP for authenticated presence (capped by engine).
pub async fn idle_tick_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let total = bounties::idle_tick(&state.db, user_id).await?;
    Ok(Json(json!({ "ok": true, "xp_now": total })))
}
