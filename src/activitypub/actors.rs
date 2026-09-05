//! ActivityStreams actor documents.
use crate::server::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn canonical_origin(state: &AppState, headers: &HeaderMap) -> String {
    if let Some(d) = &state.config.activitypub_domain {
        let d = d.trim().trim_end_matches('/');
        if d.starts_with("http://") || d.starts_with("https://") {
            return d.to_string();
        }
        return format!("https://{}", d);
    }
    if let Some(h) = headers.get("host").and_then(|v| v.to_str().ok()) {
        let host = h.trim().trim_end_matches('/');
        // dev: allow http
        if host.starts_with("localhost") || host.starts_with("127.") || host.contains(":") {
            return format!("http://{}", host);
        }
        return format!("https://{}", host);
    }
    "https://example.com".to_string()
}

async fn actor_doc(
    state: &Arc<AppState>,
    origin: &str,
    actor_type: &str,
    actor_id: i64,
    display_name: &str,
    username: &str,
) -> Value {
    let id = match actor_type {
        "instance" => format!("{}/actor", origin),
        "user" => format!("{}/uid/{}", origin, actor_id),
        "category" => format!("{}/category/{}", origin, actor_id),
        _ => format!("{}/actor", origin),
    };
    let public_pem = crate::activitypub::keys::public_pem(&state.db, actor_type, actor_id)
        .await
        .unwrap_or_default();
    // Public key PEM single-line for ActivityPub (NodeBB exposes via publicKey.publicKeyPem)
    let key_id = format!("{}#key", id);
    let inbox = format!("{}/inbox", id);
    let outbox = format!("{}/outbox", id);
    let followers = format!("{}/followers", id);
    let following = format!("{}/following", id);
    let ap_type = match actor_type {
        "category" => "Group",
        _ => "Person",
    };
    json!({
        "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
        "id": id,
        "type": ap_type,
        "preferredUsername": username,
        "name": display_name,
        "inbox": inbox,
        "outbox": outbox,
        "followers": followers,
        "following": following,
        "url": id,
        "publicKey": {
            "id": key_id,
            "owner": id,
            "publicKeyPem": public_pem
        }
    })
}

pub async fn instance_actor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !state.config.activitypub_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"err":-5,"msg":"activitypub disabled"})),
        )
            .into_response();
    }
    let origin = canonical_origin(&state, &headers);
    let doc = actor_doc(
        &state,
        &origin,
        "instance",
        0,
        &state.config.node_name,
        &state.config.node_name,
    )
    .await;
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/activity+json",
        )],
        Json(doc),
    )
        .into_response()
}

pub async fn user_actor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if !state.config.activitypub_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"err":-5,"msg":"activitypub disabled"})),
        )
            .into_response();
    }
    // Check user exists (best-effort, still serve actor for compat)
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT username, COALESCE(display_name, username) FROM users WHERE id = $1",
    )
    .bind(id as i32)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);
    let (username, display) = row.unwrap_or((format!("user{}", id), format!("User {}", id)));
    let origin = canonical_origin(&state, &headers);
    let doc = actor_doc(&state, &origin, "user", id, &display, &username).await;
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/activity+json",
        )],
        Json(doc),
    )
        .into_response()
}

pub async fn category_actor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    if !state.config.activitypub_enabled {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"err":-5,"msg":"activitypub disabled"})),
        )
            .into_response();
    }
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT slug, title FROM forum_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .unwrap_or(None);
    let (slug, title) = row.unwrap_or((format!("cat{}", id), format!("Category {}", id)));
    let origin = canonical_origin(&state, &headers);
    let doc = actor_doc(&state, &origin, "category", id, &title, &slug).await;
    (
        StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "application/activity+json",
        )],
        Json(doc),
    )
        .into_response()
}

/// Legacy alias: /actor/{id} -> user actor
pub async fn legacy_actor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    user_actor(State(state), headers, Path(id)).await
}
