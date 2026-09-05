//! ActivityPub federation for the forum — NodeBB-parity minimal slice.
//!
//! Exposes (when ACTIVITYPUB_ENABLED=true):
//!   GET  /.well-known/webfinger?resource=acct:{user}@{domain}
//!   GET  /actor                      — instance actor (uid 0 / cid 0)
//!   GET  /actor/{id} /uid/{id} /category/{id}  — user/category actors
//!   GET  /inbox /outbox              — stubs (202) for discovery
//!   POST /inbox /actor/inbox /uid/{id}/inbox — verified inbox
//!
//! Keys: RSA-2048 per-actor, stored in ap_keys (actor_type, actor_id).
//! Signatures: draft cavage + RFC 9421 style parsed from Signature / Signature-Input.
//! No background delivery yet — outbox enqueues in ap_outbox + logs in ap_inbox_log.

pub mod actors;
pub mod inbox;
pub mod keys;
pub mod outbox;
pub mod signatures;
pub mod webfinger;

use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

/// Health probe for AP feature flag.
pub async fn ap_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if !state.config.activitypub_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"err": -5, "msg": "activitypub disabled"})),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(json!({"err": 0, "enabled": true, "domain": state.config.activitypub_domain.clone()})),
    )
        .into_response()
}
