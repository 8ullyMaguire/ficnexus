//! Inbox: verify HTTP Signature + Digest, store in ap_inbox_log.
use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::server::AppState;

pub fn canonical_origin(state: &AppState, headers: &HeaderMap) -> String {
    if let Some(d)=&state.config.activitypub_domain { let d=d.trim().trim_end_matches('/'); if d.starts_with("http://")||d.starts_with("https://"){return d.to_string();} return format!("https://{}", d); }
    if let Some(h)=headers.get("host").and_then(|v| v.to_str().ok()){ let host=h.trim().trim_end_matches('/'); if host.starts_with("localhost")||host.starts_with("127.")||host.contains(':'){return format!("http://{}", host);} return format!("https://{}", host); }
    "https://example.com".to_string()
}

fn is_allowed_host(state: &AppState, host: &str) -> bool {
    if state.config.activitypub_allow_loopback { return true; }
    let h=host.to_ascii_lowercase();
    !(h=="localhost"||h.starts_with("127.")||h.starts_with("10.")||h.starts_with("192.168.")||h=="::1"||h.starts_with("172."))
}

pub async fn inbox(State(state): State<Arc<AppState>>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    inbox_inner(state, headers, body, "/inbox").await
}
pub async fn actor_inbox(State(state): State<Arc<AppState>>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    inbox_inner(state, headers, body, "/actor/inbox").await
}

async fn inbox_inner(state: Arc<AppState>, headers: HeaderMap, body: Bytes, path: &str) -> axum::response::Response {
    if !state.config.activitypub_enabled { return (StatusCode::NOT_FOUND, Json(json!({"err":-5,"msg":"activitypub disabled"}))).into_response(); }
    // Digest check
    if let Some(digest_hdr)=headers.get("digest").and_then(|v| v.to_str().ok()) {
        let expected = crate::activitypub::signatures::digest_header(&body);
        if digest_hdr!=expected { return (StatusCode::BAD_REQUEST, Json(json!({"err":-1,"msg":"digest mismatch"}))).into_response(); }
    }
    // Signature required for POST per spec
    let sig_hdr = match headers.get("signature").and_then(|v| v.to_str().ok()) { Some(s)=>s.to_string(), None=> return (StatusCode::UNAUTHORIZED, Json(json!({"err":401,"msg":"missing signature"}))).into_response() };
    let Some((key_id, headers_list, sig_b64)) = crate::activitypub::signatures::parse_signature_header(&sig_hdr) else { return (StatusCode::BAD_REQUEST, Json(json!({"err":-1,"msg":"invalid signature header"}))).into_response(); };
    // Reconstruct signing string from the headers list declared in the signature
    let _method="post";
    let host=headers.get("host").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let date=headers.get("date").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let digest=headers.get("digest").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
    let mut parts: Vec<String>=Vec::new();
    for h in headers_list.split_whitespace() {
        match h {
            "(request-target)"=>parts.push(format!("(request-target): post {}", path)),
            "host"=>parts.push(format!("host: {}", host)),
            "date"=>parts.push(format!("date: {}", date)),
            "digest"=>{ if let Some(d)=&digest { parts.push(format!("digest: {}", d)); } }
            _=>{}
        }
    }
    let signing_string=parts.join("\n");
    // Fetch public key for keyId (DB or remote fetch stub)
    let origin=canonical_origin(&state, &headers);
    let public_pem=crate::activitypub::keys::public_pem_by_key_id(&state.db, &key_id, &origin).await;
    // Loopback: allow self-signed in tests when allow_loopback
    let verified = if let Some(pem)=public_pem {
        crate::activitypub::signatures::verify_signature(&pem, &signing_string, &sig_b64).unwrap_or(false)
    } else if state.config.activitypub_allow_loopback {
        true // dev: skip verify for loopback without key
    } else { false };
    if !verified { return (StatusCode::UNAUTHORIZED, Json(json!({"err":401,"msg":"signature verification failed"}))).into_response(); }
    // Parse activity JSON
    let activity: Value = match serde_json::from_slice(&body) { Ok(v)=>v, Err(_)=> return (StatusCode::BAD_REQUEST, Json(json!({"err":-1,"msg":"invalid json"}))).into_response() };
    let actor = activity.get("actor").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !actor.is_empty() {
        if let Ok(url)=url::Url::parse(&actor) { if let Some(h)=url.host_str() { if !is_allowed_host(&state, h) && !state.config.activitypub_allow_loopback { return (StatusCode::FORBIDDEN, Json(json!({"err":-403,"msg":"blocked host"}))).into_response(); } } }
    }
    // Store
    let activity_id = activity.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let _=sqlx::query("INSERT INTO ap_inbox_log (activity_id, actor, payload, created_at) VALUES ($1,$2,$3, now()) ON CONFLICT (activity_id) DO NOTHING")
        .bind(activity_id).bind(&actor).bind(&activity).execute(&state.db).await;
    // Best-effort: remote actors table
    if !actor.is_empty() { let _=sqlx::query("INSERT INTO ap_remote_actors (actor_uri, last_seen_at) VALUES ($1, now()) ON CONFLICT (actor_uri) DO UPDATE SET last_seen_at=now()")
        .bind(&actor).execute(&state.db).await; }
    (StatusCode::OK, Json(json!({"err":0,"msg":"accepted"}))).into_response()
}
