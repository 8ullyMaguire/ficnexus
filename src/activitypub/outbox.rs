//! Minimal outbox: signed POST of Create(Note) to remote inboxes.
use crate::server::AppState;
use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

pub async fn outbox(State(state): State<Arc<AppState>>, headers: HeaderMap) -> impl IntoResponse {
    if !state.config.activitypub_enabled {
        return (
            axum::http::StatusCode::NOT_FOUND,
            Json(json!({"err":-5,"msg":"activitypub disabled"})),
        )
            .into_response();
    }
    let origin = crate::activitypub::inbox::canonical_origin(&state, &headers);
    let id = format!("{}/outbox", origin);
    (axum::http::StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "application/activity+json")], Json(json!({"@context":"https://www.w3.org/ns/activitystreams","id":id,"type":"OrderedCollection","totalItems":0,"orderedItems":[]}))).into_response()
}

pub async fn deliver_note(
    _state: &Arc<AppState>,
    _actor_type: &str,
    _actor_id: i64,
    _inbox_url: &str,
    _activity: &serde_json::Value,
) -> anyhow::Result<()> {
    // stub: real delivery is out-of-scope for slice 1; just log.
    tracing::info!(
        "AP deliver stub: inbox={} activity={}",
        _inbox_url,
        _activity
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
    );
    Ok(())
}
