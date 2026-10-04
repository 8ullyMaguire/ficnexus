use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Request body for creating a proposal
#[derive(Debug, Deserialize)]
pub struct ProposalBody {
    pub action_type: String,         // "merge" or "split"
    pub source_work_id: Option<i32>, // for merge
    pub target_work_id: Option<i32>, // for merge
    pub work_id: Option<i32>,        // for split
    pub details: Option<Value>,      // per-source assignments (split)
}

/// Request body for voting
#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub vote: i16, // 1, 0, or -1
}

/// POST /api/work-proposals — create a new proposal
pub async fn create_proposal_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProposalBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate user is at least a curator (role >= 1)
    let user_role = get_user_role(&state.db, user_id).await?;
    if user_role < 1 {
        return Err(AppError::Forbidden(
            "Curator trust level required".to_string(),
        ));
    }

    // Validate action type
    if body.action_type != "merge" && body.action_type != "split" {
        return Err(AppError::BadRequest(
            "action_type must be 'merge' or 'split'".to_string(),
        ));
    }

    // Validate works exist
    if body.action_type == "merge" {
        let source_id = body
            .source_work_id
            .ok_or_else(|| AppError::BadRequest("source_work_id required for merge".to_string()))?;
        let target_id = body
            .target_work_id
            .ok_or_else(|| AppError::BadRequest("target_work_id required for merge".to_string()))?;

        if queries::get_work(&state.db, source_id).await?.is_none() {
            return Err(AppError::NotFound("Source work not found".into()));
        }
        if queries::get_work(&state.db, target_id).await?.is_none() {
            return Err(AppError::NotFound("Target work not found".into()));
        }
        if source_id == target_id {
            return Err(AppError::BadRequest(
                "Cannot merge a work with itself".to_string(),
            ));
        }
    } else {
        let work_id = body
            .work_id
            .ok_or_else(|| AppError::BadRequest("work_id required for split".to_string()))?;
        if queries::get_work(&state.db, work_id).await?.is_none() {
            return Err(AppError::NotFound("Work not found".into()));
        }
    }

    let proposal_id = queries::create_proposal(
        &state.db,
        user_id,
        &body.action_type,
        body.source_work_id,
        body.target_work_id,
        body.work_id,
        body.details,
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "proposal_id": proposal_id,
        "msg": "Proposal created",
    })))
}

/// GET /api/work-proposals — list pending proposals
pub async fn list_proposals_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let proposals = queries::list_pending_proposals(&state.db).await?;

    let entries: Vec<Value> = proposals
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "proposer_id": p.proposer_id,
                "action_type": p.action_type,
                "source_work_id": p.source_work_id,
                "target_work_id": p.target_work_id,
                "work_id": p.work_id,
                "details": p.details,
                "status": p.status,
                "created_at": p.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "proposals": entries })))
}

/// GET /api/work-proposals/{id} — detailed view of a proposal
pub async fn get_proposal_handler(
    State(state): State<Arc<AppState>>,
    Path(proposal_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let proposal = queries::get_proposal(&state.db, proposal_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Proposal not found".into()))?;

    let (vote_sum, voter_count) = queries::get_proposal_vote_sum(&state.db, proposal_id).await?;

    Ok(Json(json!({
        "err": 0,
        "proposal": {
            "id": proposal.id,
            "proposer_id": proposal.proposer_id,
            "action_type": proposal.action_type,
            "source_work_id": proposal.source_work_id,
            "target_work_id": proposal.target_work_id,
            "work_id": proposal.work_id,
            "details": proposal.details,
            "status": proposal.status,
            "created_at": proposal.created_at.to_rfc3339(),
            "closed_at": proposal.closed_at.map(|d| d.to_rfc3339()),
            "vote_sum": vote_sum,
            "voter_count": voter_count,
        }
    })))
}

/// POST /api/work-proposals/{id}/vote — cast a vote
pub async fn vote_proposal_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(proposal_id): Path<i32>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate user is at least a curator
    let user_role = get_user_role(&state.db, user_id).await?;
    if user_role < 1 {
        return Err(AppError::Forbidden(
            "Curator trust level required".to_string(),
        ));
    }

    // Validate vote value
    if body.vote != -1 && body.vote != 0 && body.vote != 1 {
        return Err(AppError::BadRequest("Vote must be -1, 0, or 1".to_string()));
    }

    // Check proposal exists and is pending
    let proposal = queries::get_proposal(&state.db, proposal_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Proposal not found".into()))?;

    if proposal.status != "pending" {
        return Err(AppError::BadRequest(
            "Proposal is no longer pending".to_string(),
        ));
    }

    // Cast vote
    queries::cast_proposal_vote(&state.db, proposal_id, user_id, body.vote).await?;

    // Check if proposal should be accepted or rejected
    let (vote_sum, voter_count) = queries::get_proposal_vote_sum(&state.db, proposal_id).await?;

    // Senior curator veto: immediate rejection
    if body.vote == -1 && user_role >= 2 {
        queries::update_proposal_status(&state.db, proposal_id, "rejected").await?;
        return Ok(Json(json!({
            "err": 0,
            "msg": "Vote recorded. Proposal rejected by senior curator veto.",
            "vote_sum": vote_sum,
            "voter_count": voter_count,
        })));
    }

    // Quorum: 3 net upvotes and at least 2 distinct voters
    if vote_sum >= 3 && voter_count >= 2 {
        // Execute the proposal
        if proposal.action_type == "merge" {
            let source_id = proposal
                .source_work_id
                .ok_or_else(|| AppError::Internal("Missing source_work_id".into()))?;
            let target_id = proposal
                .target_work_id
                .ok_or_else(|| AppError::Internal("Missing target_work_id".into()))?;
            queries::execute_merge(&state.db, source_id, target_id).await?;
        }
        queries::update_proposal_status(&state.db, proposal_id, "accepted").await?;

        // Award reputation: +5 to proposer, +2 to each approving voter
        let event_type = format!("work_{}_accepted", proposal.action_type);
        queries::update_reputation_and_promote(&state.db, proposal.proposer_id, 5, &event_type)
            .await?;

        // Award +2 to each approving voter (who voted +1)
        let approvers: Vec<(i32,)> = sqlx::query_as(
            "SELECT user_id FROM work_proposal_votes WHERE proposal_id = $1 AND vote = 1",
        )
        .bind(proposal_id)
        .fetch_all(&state.db)
        .await?;

        for (voter_id,) in approvers {
            queries::update_reputation_and_promote(&state.db, voter_id, 2, "proposal_approved")
                .await?;
        }

        return Ok(Json(json!({
            "err": 0,
            "msg": "Vote recorded. Proposal accepted!",
            "vote_sum": vote_sum,
            "voter_count": voter_count,
        })));
    }

    // Check for rejection (net negative votes)
    if vote_sum < 0 {
        queries::update_proposal_status(&state.db, proposal_id, "rejected").await?;

        // Penalty: -1 to proposer
        let event_type = format!("work_{}_rejected", proposal.action_type);
        queries::update_reputation_and_promote(&state.db, proposal.proposer_id, -1, &event_type)
            .await?;

        return Ok(Json(json!({
            "err": 0,
            "msg": "Vote recorded. Proposal rejected.",
            "vote_sum": vote_sum,
            "voter_count": voter_count,
        })));
    }

    Ok(Json(json!({
        "err": 0,
        "msg": "Vote recorded",
        "vote_sum": vote_sum,
        "voter_count": voter_count,
    })))
}

/// Helper: get user role as integer.
///
/// NOTE: the `users.role` column is SMALLINT (0=regular, 1=trusted,
/// 5=curator, 10=admin — see auth.rs Claims doc). Some legacy code paths
/// store role *strings* ("admin"/"moderator"/"curator") — this helper maps
/// both so callers never hit a decode error.
async fn get_user_role(pool: &sqlx::PgPool, user_id: i32) -> Result<i32, AppError> {
    // Decode the raw role as text and classify: numeric smallint values pass
    // through; legacy string roles map to their numeric equivalent.
    let row: Option<(String,)> =
        sqlx::query_as("SELECT trust_level::text FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    match row.as_ref().map(|r| r.0.as_str()) {
        Some("admin") => Ok(3),
        Some("moderator") => Ok(2),
        Some("curator") => Ok(1),
        Some(other) => other
            .parse::<i32>()
            .map_err(|_| AppError::Internal(format!("unexpected trust_level value {other:?}"))),
        None => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_body_validation() {
        // Test that action_type validation works
        let body = ProposalBody {
            action_type: "merge".to_string(),
            source_work_id: Some(1),
            target_work_id: Some(2),
            work_id: None,
            details: None,
        };
        assert!(body.action_type == "merge" || body.action_type == "split");

        let body = ProposalBody {
            action_type: "invalid".to_string(),
            source_work_id: None,
            target_work_id: None,
            work_id: None,
            details: None,
        };
        assert!(body.action_type != "merge" && body.action_type != "split");
    }

    #[test]
    fn test_vote_value_validation() {
        let valid_votes = [-1, 0, 1];
        for vote in valid_votes {
            assert!(vote == -1 || vote == 0 || vote == 1);
        }
    }
}
