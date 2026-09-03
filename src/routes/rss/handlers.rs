//! Feed route handlers.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

use super::build::{
    atom_feed, atom_response, entry_for_fic, fetch_fic, fetch_followed_updates,
    fetch_new_arrivals, iso_now, FeedQuery,
};

/// Resolve the authenticated user id from a feed request.
///
/// Feed readers can't send `Authorization` headers, so the JWT may arrive
/// as a `token` query param (same token the web frontend stores in
/// `localStorage['fichub_token']`). Falls back to the Bearer header.
fn authed_user_id(headers: &HeaderMap, token: Option<&str>) -> Option<i32> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    if let Some(t) = token {
        if let Ok(claims) = crate::routes::auth::verify_token(t, &secret) {
            return Some(claims.sub);
        }
    }
    if let Some(header_val) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(val) = header_val.to_str() {
            if let Some(t) = val.strip_prefix("Bearer ") {
                if let Ok(claims) = crate::routes::auth::verify_token(t, &secret) {
                    return Some(claims.sub);
                }
            }
        }
    }
    None
}

/// Query params for the follows feed (token may come from query string).
#[derive(Debug, serde::Deserialize)]
pub struct FollowsQuery {
    pub token: Option<String>,
    pub limit: Option<i64>,
}

/// GET /feed.xml — new arrivals (most recently created fics).
pub async fn new_arrivals_feed(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FeedQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.effective_limit();
    let rows = fetch_new_arrivals(&state, limit).await?;
    let now = iso_now();

    let entries: String = rows.iter().map(entry_for_fic).collect();
    let body = atom_feed(
        "urn:ficnexus:feed:new",
        "FicNexus — New Arrivals",
        "Recently added fanfiction",
        "/feed.xml",
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );
    Ok(atom_response(body))
}

/// GET /feed/follows.xml?token=<jwt> — updates for followed fics.
///
/// 401 (via AppError::BadRequest(401, ...), the project's auth convention)
/// when no valid token is supplied.
pub async fn follows_feed(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<FollowsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = authed_user_id(&headers, query.token.as_deref())
        .ok_or_else(|| AppError::Unauthorized("Feed token required (login)".to_string()))?;

    let limit = query.limit.unwrap_or(super::MAX_ENTRIES).clamp(1, super::MAX_ENTRIES);
    let rows = fetch_followed_updates(&state, user_id, limit).await?;
    let now = iso_now();

    let entries: String = rows.iter().map(entry_for_fic).collect();
    let body = atom_feed(
        "urn:ficnexus:feed:follows",
        "FicNexus — Followed Updates",
        "Updates for the fics and authors you follow",
        "/feed/follows.xml",
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );
    Ok(atom_response(body))
}

/// GET /feed/works/<url_id>.xml — updates for a single fic.
///
/// The route is registered as `/feed/works/{url_id}` (axum 0.8 forbids
/// mixed literal+param segments); a trailing `.xml` is stripped here so the
/// canonical `/feed/works/<url_id>.xml` URL works.
pub async fn work_feed(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let url_id = url_id.strip_suffix(".xml").unwrap_or(&url_id).to_string();
    let row = fetch_fic(&state, &url_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {url_id}")))?;

    let now = iso_now();
    let entries = entry_for_fic(&row);
    let body = atom_feed(
        &format!("urn:ficnexus:feed:work:{}", row.id),
        &format!("FicNexus — Updates: {}", row.title),
        "Updates for this fanfiction",
        &format!("/feed/works/{}.xml", row.id),
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );
    Ok(atom_response(body))
}
