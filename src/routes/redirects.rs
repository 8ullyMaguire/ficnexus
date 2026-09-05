//! Legacy work-URL redirects.
//!
//! The work detail page canonical form is `/works/{title_slug}.{work_id}`,
//! e.g. `/works/the-long-way-home.1234`. All legacy shapes
//! (`/works/{url_id}`, `/works/{numeric_id}`, `/work/{anything}`) 308-redirect
//! to that canonical URL so crawlers and existing inbound links keep working.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::server::AppState;

/// Parsed form of the trailing path segment after `/works/` or `/work/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkTail {
    /// Numeric work_id (e.g. `1234` from `/works/1234`).
    Id(i32),
    /// Legacy url_id (alphanumeric, e.g. `ao3-abc123`).
    UrlId(String),
}

/// Parse the final path segment under `/works/` or `/work/`.
///
/// Returns `None` for ambiguous/malformed input so the handler can 404.
///
/// Examples:
///   - `"1234"`           → Some(Id(1234))
///   - `"007"`            → Some(Id(7))
///   - `"some.title.99"`  → Some(Id(99))    (last `.id` wins)
///   - `"file.tar.gz.12"` → Some(Id(12))    (slugs may contain dots)
///   - `"ao3-abc"`        → Some(UrlId("ao3-abc"))
///   - `"a.b"`           → None              (looks like slug.id but id not numeric)
///   - `""`              → None
///   - `"12.abc"`        → None              (suffix after `.` is not numeric)
pub fn parse_work_tail(tail: &str) -> Option<WorkTail> {
    let tail = tail.trim();
    if tail.is_empty() {
        return None;
    }
    // All-digits → numeric Id.
    if tail.chars().all(|c| c.is_ascii_digit()) {
        return tail.parse::<i32>().ok().map(WorkTail::Id);
    }
    // Contains a `.` — split on LAST `.` and require numeric suffix.
    if let Some(dot) = tail.rfind('.') {
        let (left, suffix) = tail.split_at(dot);
        let suffix = &suffix[1..]; // skip the '.'
        if !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(id) = suffix.parse::<i32>() {
                let _ = left; // slug is ignored — server uses DB slug
                return Some(WorkTail::Id(id));
            }
        }
        return None;
    }
    // No dot, not all digits → legacy url_id.
    if !tail.is_empty() {
        return Some(WorkTail::UrlId(tail.to_string()));
    }
    None
}

/// Build the canonical URL from a (work_id, slug) pair.
pub fn canonical_url(slug: &str, work_id: i32) -> String {
    format!("/works/{}.{}", slug, work_id)
}

/// 308-redirect handler for `/works/{tail}` and `/work/{tail}`.
pub async fn work_redirect_handler(
    State(state): State<Arc<AppState>>,
    Path(tail): Path<String>,
) -> Result<Response, AppError> {
    let parsed = parse_work_tail(&tail).ok_or_else(|| {
        AppError::NotFound(format!("work not found: {tail}"))
    })?;
    // For Id, the DB lookup returns (url_id, slug) — we re-use the slug and
    // the input work_id. For UrlId, the DB lookup returns (work_id, slug).
    let (work_id, slug) = match parsed {
        WorkTail::Id(id) => {
            let (_uid, slug) = queries::get_work_canonical_url_id(&state.db, id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("work {id} not found")))?;
            (id, slug)
        }
        WorkTail::UrlId(uid) => {
            queries::get_work_slug_target(&state.db, &uid)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("work {uid} not found")))?
        }
    };
    let url = canonical_url(&slug, work_id);
    // 308 Permanent — preserves method + body, picked up by crawlers.
    Ok(Redirect::permanent(&url).into_response())
}

/// `GET /works` (no tail) → 308 to home.
pub async fn works_root_redirect() -> impl IntoResponse {
    Redirect::permanent("/")
}

/// Plain 404 (used by tests / future fall-throughs).
pub fn not_found_response() -> Response {
    (StatusCode::NOT_FOUND, "work not found").into_response()
}

/// `GET /api/works/resolve/{url_id}` — SPA helper: url_id → (work_id, slug).
pub async fn resolve_work(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    match queries::get_work_slug_target(&state.db, &url_id).await? {
        Some((work_id, slug)) => Ok(axum::Json(serde_json::json!({
            "err": 0,
            "work_id": work_id,
            "slug": slug,
            "url": canonical_url(&slug, work_id),
        }))),
        None => Err(AppError::NotFound(format!("work {url_id} not found"))),
    }
}

/// `GET /api/works/{id}/canonical` — SPA helper: numeric work_id → (url_id, slug).
pub async fn canonical_for_work(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    match queries::get_work_canonical_url_id(&state.db, work_id).await? {
        Some((url_id, slug)) => Ok(axum::Json(serde_json::json!({
            "err": 0,
            "url_id": url_id,
            "slug": slug,
            "url": canonical_url(&slug, work_id),
        }))),
        None => Err(AppError::NotFound(format!("work {work_id} not found"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_work_tail, canonical_url, WorkTail};

    #[test]
    fn parses_numeric_id() {
        assert_eq!(parse_work_tail("1234"), Some(WorkTail::Id(1234)));
    }

    #[test]
    fn parses_zero_padded_id() {
        assert_eq!(parse_work_tail("007"), Some(WorkTail::Id(7)));
    }

    #[test]
    fn parses_slug_with_numeric_suffix() {
        assert_eq!(parse_work_tail("some.title.99"), Some(WorkTail::Id(99)));
    }

    #[test]
    fn parses_filename_like_slug() {
        assert_eq!(parse_work_tail("file.tar.gz.12"), Some(WorkTail::Id(12)));
    }

    #[test]
    fn parses_legacy_url_id() {
        assert_eq!(
            parse_work_tail("ao3-abc"),
            Some(WorkTail::UrlId("ao3-abc".to_string()))
        );
    }

    #[test]
    fn rejects_dot_without_numeric_suffix() {
        assert_eq!(parse_work_tail("a.b"), None);
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse_work_tail(""), None);
    }

    #[test]
    fn rejects_non_numeric_suffix() {
        assert_eq!(parse_work_tail("12.abc"), None);
    }

    #[test]
    fn canonical_url_format() {
        assert_eq!(
            canonical_url("the-long-way-home", 1234),
            "/works/the-long-way-home.1234"
        );
    }

    #[test]
    fn canonical_url_with_default_slug() {
        assert_eq!(canonical_url("work", 0), "/works/work.0");
    }
}
