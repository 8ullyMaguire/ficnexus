//! WebFinger JRD for ActivityPub discovery.
use axum::{extract::{Query, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::server::AppState;

#[derive(Deserialize)]
pub struct WebfingerParams { pub resource: Option<String> }

fn canonical_origin(state: &AppState, headers: &HeaderMap) -> String {
    if let Some(d) = &state.config.activitypub_domain { let d=d.trim().trim_end_matches('/'); if d.starts_with("http://")||d.starts_with("https://"){return d.to_string();} return format!("https://{}", d); }
    if let Some(h)=headers.get("host").and_then(|v| v.to_str().ok()){ let host=h.trim().trim_end_matches('/'); if host.starts_with("localhost")||host.starts_with("127.")||host.contains(':'){return format!("http://{}", host);} return format!("https://{}", host); }
    "https://example.com".to_string()
}

pub async fn webfinger(State(state): State<Arc<AppState>>, headers: HeaderMap, Query(params): Query<WebfingerParams>) -> impl IntoResponse {
    if !state.config.activitypub_enabled { return (StatusCode::NOT_FOUND, Json(json!({"err":-5,"msg":"activitypub disabled"}))).into_response(); }
    let Some(resource) = params.resource else { return (StatusCode::BAD_REQUEST, Json(json!({"err":-1,"msg":"missing resource"}))).into_response(); };
    let origin = canonical_origin(&state, &headers);
    let domain = origin.trim_start_matches("https://").trim_start_matches("http://").trim_end_matches('/');
    // resource is acct:username@domain or actor URI
    let mut username: Option<String> = None;
    let mut actor_uri: Option<String> = None;
    if resource.starts_with("acct:") {
        let bare = resource.trim_start_matches("acct:");
        if let Some((u, d)) = bare.split_once('@') {
            if d.eq_ignore_ascii_case(domain) || state.config.activitypub_allow_loopback { username = Some(u.to_string()); }
            else { return (StatusCode::NOT_FOUND, Json(json!({"err":-5,"msg":"domain mismatch"}))).into_response(); }
        }
    } else if resource.starts_with("http://") || resource.starts_with("https://") {
        actor_uri = Some(resource.clone());
    } else if resource.contains('@') {
        let bare = resource.trim_start_matches('@');
        if let Some((u, d)) = bare.split_once('@') {
            if d.eq_ignore_ascii_case(domain) || state.config.activitypub_allow_loopback { username = Some(u.to_string()); }
        }
    }
    // Resolve username -> actor URI via users or node_name (instance actor)
    let subject;
    let href;
    if let Some(u) = username {
        // instance actor if matches node_name
        if u.eq_ignore_ascii_case(&state.config.node_name) {
            subject = format!("acct:{}@{}", u, domain);
            href = format!("{}/actor", origin);
        } else {
            // Look up by username
            let row: Option<(i32,)> = sqlx::query_as("SELECT id FROM users WHERE lower(username)=lower($1) LIMIT 1")
                .bind(&u).fetch_optional(&state.db).await.unwrap_or(None);
            if let Some((uid,)) = row {
                subject = format!("acct:{}@{}", u, domain);
                href = format!("{}/uid/{}", origin, uid);
            } else {
                // Try category slug
                let cat: Option<(i64,)> = sqlx::query_as("SELECT id FROM forum_categories WHERE lower(slug)=lower($1) LIMIT 1")
                    .bind(&u).fetch_optional(&state.db).await.unwrap_or(None);
                if let Some((cid,)) = cat {
                    subject = format!("acct:{}@{}", u, domain);
                    href = format!("{}/category/{}", origin, cid);
                } else {
                    return (StatusCode::NOT_FOUND, Json(json!({"err":-5,"msg":"actor not found"}))).into_response();
                }
            }
        }
    } else if let Some(uri) = actor_uri {
        // Already an actor URI — echo it
        let host_ok = uri.contains(domain) || state.config.activitypub_allow_loopback;
        if !host_ok { return (StatusCode::NOT_FOUND, Json(json!({"err":-5,"msg":"domain mismatch"}))).into_response(); }
        // Derive subject from path
        let path = uri.split('/').last().unwrap_or("actor");
        subject = format!("acct:{}@{}", path, domain);
        href = uri;
    } else {
        return (StatusCode::BAD_REQUEST, Json(json!({"err":-1,"msg":"invalid resource"}))).into_response();
    }
    let jrd = json!({
        "subject": subject,
        "aliases": [href],
        "links": [{"rel":"self","type":"application/activity+json","href": href}]
    });
    (StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "application/jrd+json")], Json(jrd)).into_response()
}
