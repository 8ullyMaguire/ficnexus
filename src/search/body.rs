//! Full-text search over fic BODIES: "fics where X says Y".
//!
//! The metadata search (`/api/search`) matches the `text_search` generated
//! column (title + description). Body search matches the maintained
//! `body_text_search` tsvector column (populated at body-write time by
//! `body_cache::index_body_text` and by the `backfill_body_search` bin).
//!
//! Snippet strategy: the match+rank query is pure SQL (GIN index +
//! ts_rank). The highlighted excerpt is built in Rust — the body text
//! lives ON DISK (BODY_CACHE_DIR JSON blobs), so ts_headline cannot see
//! it; for the ≤50 rows on the page we load `body_plain_text` per row and
//! run a small highlighter that wraps matching terms in `<mark>`.

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::sync::Arc;

use crate::error::AppError;
use crate::search::parser::{expr_to_tsquery, parse_query};
use crate::server::AppState;

/// Query parameters for GET /api/search/body.
#[derive(Debug, Deserialize, Default)]
pub struct BodySearchParams {
    pub q: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

/// Row shape returned by the match query (fic_info columns + rank).
#[derive(Debug)]
struct BodyHit {
    url_id: String,
    work_id: Option<i32>,
    title: String,
    author: String,
    source: String,
    words: i64,
    chapters: i32,
    status: String,
    description: String,
    rank: f32,
}

const DEFAULT_PER_PAGE: usize = 20;
const MAX_PER_PAGE: usize = 50;
/// Max chars of plain body text scanned for highlighting (avoid feeding a
/// 2M-char fic through the word scanner on every page load).
const SNIPPET_TEXT_CAP: usize = 500_000;
const SNIPPET_WINDOW_WORDS: usize = 30;
const SNIPPET_MAX_WORDS: usize = 80;

/// GET /api/search/body?q=...&page=1&per_page=20
///
/// Response: `{ err: 0, total, page, per_page, results: [...] }` where
/// each result carries `body_snippet` — a `<mark>`-highlighted excerpt
/// from the fic body (null when the body cache was cleared).
pub async fn body_search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BodySearchParams>,
    _auth: crate::routes::auth::AuthUser,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    use crate::limiter::Tier;

    let q = params.q.as_deref().unwrap_or("").trim();
    if q.is_empty() {
        return Err(AppError::BadRequest("q must not be empty".to_string()));
    }

    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Rate limit (search tier — very high ceiling; fail-open: Redis errors
    // surface as Allowed, a hiccup never blocks the search itself).
    let ip = crate::limiter::client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    if let crate::limiter::TieredRateLimitResult::Wait(secs) = state
        .rate_limiter
        .check(ip, client_id.as_deref(), Tier::Search)
        .await
    {
        return Err(AppError::RateLimited(secs));
    }

    let tsquery = expr_to_tsquery(&parse_query(q));
    if tsquery.is_empty() {
        return Err(AppError::BadRequest("q must not be empty".to_string()));
    }

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(DEFAULT_PER_PAGE).clamp(1, MAX_PER_PAGE);
    let offset = (page - 1) * per_page;

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fic_info WHERE body_text_search @@ $1::tsquery",
    )
    .bind(&tsquery)
    .fetch_one(&state.db)
    .await?;

    // Match + rank. body_text_search is NULL for fics without a cached
    // body, so they can never match. ts_rank DESC breaks ties by id for
    // deterministic pagination.
    let rows = sqlx::query(
        "SELECT fi.id AS url_id, fi.work_id, fi.title, fi.author, fi.source,
                fi.words, fi.chapters, fi.status, fi.description,
                ts_rank(fi.body_text_search, $1::tsquery) AS rank
         FROM fic_info fi
         WHERE fi.body_text_search @@ $1::tsquery
         ORDER BY ts_rank(fi.body_text_search, $1::tsquery) DESC, fi.id
         LIMIT $2 OFFSET $3",
    )
    .bind(&tsquery)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let hit = BodyHit {
            url_id: row.get("url_id"),
            work_id: row.get("work_id"),
            title: row.get("title"),
            author: row.get("author"),
            source: row.get("source"),
            words: row.get("words"),
            chapters: row.get("chapters"),
            status: row.get("status"),
            description: row.get("description"),
            rank: row.get("rank"),
        };
        // Body text lives on disk — load it per row (only the page, ≤50)
        // and highlight client-side. Cache cleared → snippet null.
        let snippet = crate::body_cache::body_plain_text(&state.config, &hit.url_id)
            .map(|text| highlight_snippet(&text, &tsquery));
        results.push(json!({
            "url_id": hit.url_id,
            "work_id": hit.work_id,
            "title": hit.title,
            "author": hit.author,
            "source": hit.source,
            "words": hit.words,
            "chapters": hit.chapters,
            "status": hit.status,
            "description": hit.description,
            "rank": hit.rank,
            "body_snippet": snippet,
        }));
    }

    Ok(Json(json!({
        "err": 0,
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": results,
    })))
}

/// Build a `<mark>`-highlighted excerpt around the first match of any
/// tsquery term. Falls back to a plain excerpt when nothing matches (rank
/// > 0 but the term spelling differs from the stemmed tsvector form).
fn highlight_snippet(text: &str, tsquery: &str) -> String {
    let text = &text[..text.len().min(SNIPPET_TEXT_CAP)];
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let stems = tsquery_stems(tsquery);
    let mut best: Option<usize> = None; // word index of the first match
    for (i, w) in words.iter().enumerate() {
        if stems.iter().any(|s| w.to_lowercase().contains(s)) {
            best = Some(i);
            break;
        }
    }

    let start = best.unwrap_or(0).saturating_sub(SNIPPET_WINDOW_WORDS);
    let mut parts: Vec<String> = Vec::new();
    let mut count = 0usize;
    for (_i, w) in words.iter().enumerate().skip(start) {
        if count >= SNIPPET_MAX_WORDS {
            break;
        }
        let lower = w.to_lowercase();
        if stems.iter().any(|s| lower.contains(s)) {
            parts.push(format!("<mark>{}</mark>", html_escape(w)));
        } else {
            parts.push(html_escape(w));
        }
        count += 1;
    }

    let mut out = parts.join(" ");
    if start > 0 {
        out = format!("… {}", out);
    }
    if start + count < words.len() {
        out = format!("{} …", out);
    }
    out
}

/// Extract the bare stems a tsquery matches on: `harry:* & potter:*` →
/// ["harry", "potter"]; phrase operators `<->` and boolean `& | ! ()`
/// are ignored.
fn tsquery_stems(tsquery: &str) -> Vec<String> {
    tsquery
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|tok| !tok.is_empty() && tok.chars().any(|c| c.is_ascii_alphabetic()))
        .map(|tok| tok.to_ascii_lowercase())
        .collect()
}

/// Escape HTML metacharacters so body text cannot inject markup into the
/// snippet (only our own `<mark>` tags may pass through).
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tsquery_stems_extracts_terms() {
        let stems = tsquery_stems("harry:* & potter:*");
        assert_eq!(stems, vec!["harry", "potter"]);
        let stems = tsquery_stems("(fire:* <-> bolt:*) | !(snake:*)");
        assert_eq!(stems, vec!["fire", "bolt", "snake"]);
    }

    #[test]
    fn highlight_wraps_matches_and_ellipsizes() {
        // Long body (>80 words): the match sits far past the window start,
        // so the excerpt must open with "…" and still wrap the match.
        let mut text = String::new();
        for i in 0..40 {
            text.push_str(&format!("Padding word number {i}. "));
        }
        text.push_str("And then Harry Potter finally appeared in the story.");
        let out = highlight_snippet(&text, "harry:*");
        assert!(out.starts_with('…'), "snippet should open with ellipsis: {out}");
        assert!(out.contains("<mark>Harry</mark>"), "snippet: {out}");
    }

    #[test]
    fn highlight_escapes_html_in_body() {
        let text = "He said <b>no</b> to the monster & everyone cheered.";
        let out = highlight_snippet(text, "monster:*");
        assert!(out.contains("&lt;b&gt;"), "html must be escaped: {out}");
        assert!(out.contains("<mark>monster</mark>"), "snippet: {out}");
    }
}
