//! Curator proposals API — M2 of docs/plans/translation-everything.md
//!
//! Routes:
//! - POST /api/proposals                 submit (auth, honeypot, budget)
//! - GET  /api/proposals/{id}            detail + votes + competing
//! - POST /api/proposals/{id}/vote       curator quorum vote
//! - POST /api/proposals/{id}/decide     curator force-resolve
//! - GET  /api/curator/proposals         queue (curators only)
//! - POST /api/translate/flag            flag machine translation -> proposal

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::honeypot;
use crate::server::AppState;
use crate::services::proposals;

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[derive(Deserialize)]
pub struct CreateBody {
    kind: String,
    target_type: String,
    target_id: String,
    payload: serde_json::Value,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    form_opened_at: Option<String>,
}

/// POST /api/proposals
pub async fn create(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let Some(user_id) = user.user_id else {
        return Err(AppError::Unauthorized("login required".into()));
    };
    // honeypot: silent fake success
    if honeypot::inspect_submission(body.website.as_deref(), body.form_opened_at.as_deref(), now_ms())
        == honeypot::TrapVerdict::RejectSilently
    {
        return Ok(Json(json!({ "err": 0, "id": -1 })));
    }
    if !proposals::valid_kind(&body.kind) {
        return Err(AppError::BadRequest(format!(
            "unknown proposal kind: {}",
            body.kind
        )));
    }
    // translate payloads must carry field+locale+text
    if body.kind == "translate" {
        let p = &body.payload;
        let ok = ["field", "locale", "text"]
            .iter()
            .all(|k| p.get(*k).and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()));
        if !ok {
            return Err(AppError::BadRequest(
                "translate payload requires non-empty field, locale, text".into(),
            ));
        }
    }
    // per-user daily cap
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint FROM proposals
           WHERE proposer_id = $1 AND created_at > now() - interval '1 day'"#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    if count >= state.config.proposals_per_user_per_day as i64 {
        return Err(AppError::RateLimited(3600));
    }
    let id = proposals::submit(
        &state.db,
        &body.kind,
        &body.target_type,
        &body.target_id,
        body.payload,
        Some(user_id),
        "user",
    )
    .await?;
    Ok(Json(json!({ "err": 0, "id": id })))
}

/// GET /api/proposals/{id} — public detail with votes + competing proposals.
pub async fn get_one(
    State(state): State<std::sync::Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row: Option<(String, String, String, serde_json::Value, Option<i32>, Option<String>, String, Option<String>, serde_json::Value, String)> = sqlx::query_as(
        r#"SELECT p.kind, p.target_type, p.target_id, p.payload, p.proposer_id,
                  u.username, p.status, p.note, p.created_at::text AS created_at, p.source
           FROM proposals p LEFT JOIN users u ON u.id = p.proposer_id
           WHERE p.id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let Some(row) = row else {
        return Err(AppError::NotFound("proposal not found".into()));
    };
    let votes: Vec<(String, String, String)> = sqlx::query_as(
        r#"SELECT u.username, v.decision, v.created_at::text
           FROM proposal_votes v JOIN users u ON u.id = v.curator_id
           WHERE v.proposal_id = $1 ORDER BY v.created_at"#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let competing: Vec<(i64, String, serde_json::Value)> = sqlx::query_as(
        r#"SELECT id, status, payload FROM proposals
           WHERE kind = $1 AND target_type = $2 AND target_id = $3 AND id <> $4
             AND status IN ('pending','approved')
           ORDER BY created_at DESC LIMIT 10"#,
    )
    .bind(&row.0)
    .bind(&row.1)
    .bind(&row.2)
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(json!({
        "err": 0,
        "proposal": {
            "id": id,
            "kind": row.0,
            "target_type": row.1,
            "target_id": row.2,
            "payload": row.3,
            "proposer_id": row.4,
            "proposer": row.5,
            "status": row.6,
            "note": row.7,
            "created_at": row.8,
            "source": row.9,
            "votes": votes.iter().map(|(u, d, ts)| json!({"username": u, "decision": d, "at": ts})).collect::<Vec<_>>(),
            "competing": competing.iter().map(|(cid, cs, cp)| json!({"id": cid, "status": cs, "payload": cp})).collect::<Vec<_>>(),
        }
    })))
}

#[derive(Deserialize)]
pub struct VoteBody {
    decision: String,
}

/// POST /api/proposals/{id}/vote — curator quorum vote.
pub async fn vote(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let curator_id = user.user_id.ok_or_else(|| AppError::Unauthorized("login".into()))?;
    let name = user.username.clone().unwrap_or_else(|| "?".into());
    let out = proposals::record_vote(
        &state.db,
        &state.config,
        id,
        curator_id,
        user.role,
        &name,
        &body.decision,
    )
    .await?;
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct DecideBody {
    decision: String,
    #[serde(default)]
    note: Option<String>,
}

/// POST /api/proposals/{id}/decide — curator fast-path resolve.
pub async fn decide(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<DecideBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let curator_id = user.user_id.ok_or_else(|| AppError::Unauthorized("login".into()))?;
    let name = user.username.clone().unwrap_or_else(|| "?".into());
    let out = proposals::decide(
        &state.db,
        &state.config,
        id,
        curator_id,
        user.role,
        &name,
        &body.decision,
        body.note.as_deref(),
    )
    .await?;
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct QueueParams {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    locale: Option<String>,
    #[serde(default = "default_page")]
    page: i64,
}

fn default_page() -> i64 {
    0
}

/// GET /api/curator/proposals — queue for curators (role >= 5).
pub async fn queue(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Query(q): Query<QueueParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    if user.role < 5 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
    let status = q.status.unwrap_or_else(|| "pending".into());
    let page = q.page.clamp(0, 1000);
    let rows: Vec<(i64, String, String, String, serde_json::Value, Option<String>, Option<i32>, i64, i64, String, String)> = sqlx::query_as(
        r#"SELECT p.id, p.kind, p.target_type, p.target_id, p.payload, u.username,
                  p.proposer_id,
                  COUNT(v.*) FILTER (WHERE v.decision = 'approve')::bigint AS approves,
                  COUNT(v.*) FILTER (WHERE v.decision = 'dismiss')::bigint AS dismisses,
                  p.status, p.created_at::text
           FROM proposals p
           LEFT JOIN users u ON u.id = p.proposer_id
           LEFT JOIN proposal_votes v ON v.proposal_id = p.id
           WHERE p.status = $1
             AND ($2::text IS NULL OR p.kind = $2)
             AND ($3::text IS NULL OR p.payload->>'locale' = $3)
           GROUP BY p.id, u.username, v.proposal_id
           ORDER BY p.created_at DESC
           LIMIT 20 OFFSET $4"#,
    )
    .bind(&status)
    .bind(q.kind.as_deref())
    .bind(q.locale.as_deref())
    .bind(page * 20)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(id, kind, ttype, tid, payload, proposer, _pid, approves, dismisses, st, created)| {
            json!({
                "id": id, "kind": kind, "target_type": ttype, "target_id": tid,
                "payload": payload, "proposer": proposer, "approves": approves,
                "dismisses": dismisses, "quorum": proposals::quorum_for(&state.config, &kind),
                "status": st, "created_at": created,
            })
        })
        .collect();
    Ok(Json(json!({ "err": 0, "items": items, "page": page })))
}

#[derive(Deserialize)]
pub struct FlagBody {
    target_type: String,
    target_id: String,
    field: String,
    locale: String,
    #[serde(default)]
    reason: Option<String>,
}

/// POST /api/translate/flag — dismiss a machine row + queue a re-translation.
pub async fn flag_translation(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<FlagBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let Some(user_id) = user.user_id else {
        return Err(AppError::Unauthorized("login required".into()));
    };
    // dedupe against existing pending proposal for same tuple
    if proposals::pending_translate_exists(
        &state.db,
        &body.target_type,
        &body.target_id,
        &body.field,
        &body.locale,
    )
    .await?
    {
        return Ok(Json(json!({ "err": 0, "dup": true })));
    }
    // dismiss the machine row immediately (it's gone if it was user-approved;
    // approved rows are not dismissed by flags — curator vote decides those)
    let dismissed: u64 = sqlx::query(
        r#"UPDATE translation_strings SET status = 'dismissed', updated_at = NOW()
           WHERE target_type=$1 AND target_id=$2 AND field=$3 AND locale=$4
             AND status = 'machine'"#,
    )
    .bind(&body.target_type)
    .bind(&body.target_id)
    .bind(&body.field)
    .bind(&body.locale)
    .execute(&state.db)
    .await
    .map(|r| r.rows_affected())
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let _ = dismissed;

    // proposal records the flag
    let payload = json!({
        "field": body.field,
        "locale": body.locale,
        "reason": body.reason,
        "flag": true,
    });
    proposals::submit(
        &state.db,
        "translate",
        &body.target_type,
        &body.target_id,
        payload,
        Some(user_id),
        "user",
    )
    .await?;
    // small rep for flagging (capped in xp_source_defs)
    let _ = crate::services::progression::award_xp(
        &state.db,
        user_id,
        "translation_flagged",
        Some(&format!("{}:{}", body.target_type, body.target_id)),
    )
    .await;
    // re-queue machine translation for a fresh attempt
    if state.config.translate_llm_enabled {
        let mut conn = state.redis.clone();
        let job = json!({
            "type": body.target_type, "id": body.target_id.parse::<i64>().unwrap_or(0),
            "field": body.field, "locale": body.locale, "flag_requeue": true,
        });
        let _: Result<i64, _> = redis::cmd("RPUSH")
            .arg("translate_queue")
            .arg(job.to_string())
            .query_async(&mut conn)
            .await;
    }
    Ok(Json(json!({ "err": 0 })))
}

