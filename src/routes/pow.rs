//! Proof-of-work (PoW) challenge endpoints.
//!
//! `GET /api/pow/challenge` issues a hashcash-style challenge to a
//! shadowbanned/flagged client. `POST /api/pow/solve` verifies the client's
//! solution and caches it in Redis (`fichub:pow:solved:<challenge>`, 10 min
//! TTL) so the client can reuse it across follow-up export requests.
//!
//! Non-shadowbanned clients get `{"err":0, "not_needed":true}` — normal users
//! never see any friction.

use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::services::pow;

/// `POST /api/pow/solve` body.
#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    /// The challenge string from `GET /api/pow/challenge`.
    pub challenge: String,
    /// The client's solution nonce (decimal digits).
    pub nonce: String,
}

/// `GET /api/pow/challenge` — issue a challenge to a shadowbanned client.
///
/// Requires the `X-Client-Id` header: the shadowban set is keyed by
/// client_id (see `src/limiter/`). When the client is NOT shadowbanned the
/// response is `{"err":0,"not_needed":true}` so normal traffic sees no
/// friction at all.
pub async fn challenge_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let Some(cid) = client_id else {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
            "message": "no client id",
        })));
    };

    // Fail-open: a Redis hiccup means is_shadowbanned() is false, so normal
    // traffic is never accidentally challenged.
    if !state.rate_limiter.is_shadowbanned(&cid) {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
        })));
    }

    let challenge = pow::generate_challenge(
        state.config.pow_difficulty,
        state.config.pow_ttl_secs,
    );
    Ok(Json(json!({
        "err": 0,
        "not_needed": false,
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}

/// `POST /api/pow/solve` — verify a solution and cache it in Redis.
///
/// Returns 429 until a valid solution is stored, then `{"err":0}`. A valid
/// solve is cached with a 10-minute TTL so subsequent export requests pass
/// the epub_handler's PoW gate without re-solving.
pub async fn solve_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SolveRequest>,
) -> Result<Json<Value>, AppError> {
    // Only shadowbanned clients are allowed to solve (the challenge is
    // meaningless for everyone else).
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref cid) = client_id {
        if !state.rate_limiter.is_shadowbanned(cid) {
            return Ok(Json(json!({
                "err": 0,
                "not_needed": true,
            })));
        }
    }

    // Basic sanity: the challenge must be a plausible 32-hex-char value.
    if body.challenge.len() != 32 || !body.challenge.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::RateLimited(0));
    }

    // Remember the challenge we are about to accept so the export gate can
    // tie the solve to the challenge it actually checks.
    if let Some(ref cid) = client_id {
        pow::set_latest_challenge_for_client(&state, cid, &body.challenge);
    }

    let difficulty = state.config.pow_difficulty;
    if pow::verify_solution(&body.challenge, &body.nonce, difficulty) {
        let _ = pow::store_solution(
            &mut state.redis.clone(),
            &body.challenge,
            state.config.pow_ttl_secs,
        )
        .await;
        return Ok(Json(json!({
            "err": 0,
            "solved": true,
        })));
    }

    // Invalid solution → 429 with a fresh challenge (friction, not a block).
    let challenge = pow::generate_challenge(difficulty, state.config.pow_ttl_secs);
    Err(AppError::RateLimitedJson(json!({
        "err": -429,
        "msg": "invalid proof of work",
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}
