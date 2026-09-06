use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::limiter::{Tier, TieredRateLimitResult, client_ip_from_headers};
use crate::search::builder::SearchParams;
use crate::search::routes::{run_search, SearchFacets, SearchResponseEnvelope};
use crate::server::AppState;
use crate::db::queries::works::get_work_sources;

/// Parsed representation of a find-fic query.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParsedFindQuery {
    pub title: Option<String>,
    pub author: Option<String>,
    pub site: Option<String>,  // canonical site key, e.g. "spacebattles"
    pub raw: String,           // original input for the /ask fallback
}

/// Form input for POST /api/find-fic.
#[derive(Debug, Deserialize)]
pub struct FindFicQuery {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_limit() -> usize { 5 }
fn default_per_page() -> usize { 20 }

/// Resolve the caller's real IP from X-Forwarded-For with the peer address
/// as fallback.
fn caller_ip(headers: &HeaderMap, remote: SocketAddr) -> std::net::IpAddr {
    client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        remote.ip(),
    )
}

/// Enforce the Search-tier rate limiter (very high: 1000/min per IP).
/// Fails open on limiter errors so a Redis hiccup never blocks searches.
async fn enforce_search_rate_limit(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    remote: SocketAddr,
) -> Result<(), AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    match state
        .rate_limiter
        .check(
            caller_ip(headers, remote),
            client_id.as_deref(),
            Tier::Search,
        )
        .await
    {
        TieredRateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        TieredRateLimitResult::Allowed => Ok(()),
    }
}

/// Handles POST /api/find-fic.
/// Accepts a free-text query and returns canonical works that match.
pub async fn find_fic(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(form): Json<FindFicQuery>,
) -> Result<Json<Value>, AppError> {
    // Apply the Search-tier rate limiter (very high: 1000/min per IP).
    enforce_search_rate_limit(&state, &headers, remote).await?;

    // Parse the query string into structured fields
    let parsed = parse_find_query(&form.query);

    // If the input contained a URL, don't parse further — let the client
    // decide whether to hit /api/search/ask instead.
    if parsed.raw.contains("://") {
        return Ok(Json(json!({
            "err": 0,
            "parsed": parsed,
            "results": [],
            "suggestions": [],
            "fallback": "ask",
        })));
    }

    // Build the v2 query string: title:"{title}" author:"{author}"
    // Include source:{site} when a site was specified.
    let mut q = String::new();
    if let Some(ref title) = parsed.title {
        q.push_str(&format!("title:\"{}\" ", title));
    }
    if let Some(ref author) = parsed.author {
        q.push_str(&format!("author:\"{}\" ", author));
    }
    if let Some(ref site) = parsed.site {
        q.push_str(&format!("source:{} ", site));
    }
    // Trim trailing space
    q = q.trim_end().to_string();

    // Build SearchParams and delegate to the existing search pipeline
    let search_params = SearchParams {
        q: Some(q.clone()),
        page: Some(1),
        per_page: Some(form.per_page),
        ..Default::default()
    };

    // Run the search through the shared pipeline
    let envelope: SearchResponseEnvelope = run_search(&state, search_params).await?;

    // Map results to canonical works: dedupe by work_id
    let mut seen_work_ids: std::collections::HashSet<i32> = std::collections::HashSet::new();
    let mut canonical_results: Vec<Value> = Vec::new();

    for result in &envelope.results {
        // Look up the work_id for this url_id
        let work_id: Option<i32> = sqlx::query_scalar(
            "SELECT work_id FROM fic_info WHERE id = $1",
        ).bind(&result.url_id)
        .fetch_optional(&state.db)
        .await?;

        if let Some(wid) = work_id {
            if seen_work_ids.contains(&wid) {
                continue;  // already have this canonical work
            }
            seen_work_ids.insert(wid);

            // Get all sources for this work
            let sources = get_work_sources(&state.db, wid)
                .await
                .unwrap_or_default()
                .iter()
                .map(|fi| fi.source.clone())
                .collect::<Vec<_>>();

            canonical_results.push(json!({
                "work_id": wid,
                "url_id": result.url_id,
                "title": result.title,
                "author": result.author,
                "words": result.words,
                "sources": sources,
                "score": result.rank.unwrap_or(0.0) as f64,
            }));
        }
    }

    // Limit to the requested number
    canonical_results.truncate(form.limit);

    // If no results, try fuzzy suggestions
    let suggestions: Vec<Value> = if canonical_results.is_empty() && parsed.title.is_some() {
        let fuzzy_params = SearchParams {
            q: Some(parsed.title.clone().unwrap_or_default()),
            fuzzy: true,
            page: Some(1),
            per_page: Some(5),
            ..Default::default()
        };
        let fuzzy_envelope: SearchResponseEnvelope = run_search(&state, fuzzy_params).await.unwrap_or_else(|_| SearchResponseEnvelope {
            total: 0,
            page: 1,
            per_page: 5,
            results: Vec::new(),
            facets: SearchFacets {
                fandoms: Vec::new(),
                characters: Vec::new(),
                relationships: Vec::new(),
                warnings: Vec::new(),
                categories: Vec::new(),
                freeforms: Vec::new(),
                statuses: Vec::new(),
            },
        });
        fuzzy_envelope.results.iter().map(|r| {
            json!({
                "title": r.title,
                "author": r.author,
            })
        }).collect()
    } else {
        Vec::new()
    };

    // Determine fallback
    let fallback: Option<String> = if canonical_results.is_empty() {
        if parsed.title.is_some() || parsed.author.is_some() {
            Some("request".to_string())
        } else {
            Some("ask".to_string())
        }
    } else {
        None
    };

    Ok(Json(json!({
        "err": 0,
        "parsed": parsed,
        "results": canonical_results,
        "suggestions": suggestions,
        "fallback": fallback,
    })))
}

/// Site alias -> canonical key mapping.
const SITE_ALIASES: &[(&str, &str)] = &[
    ("sb", "spacebattles"),
    ("spacebattles", "spacebattles"),
    ("sv", "sufficientvelocity"),
    ("sufficientvelocity", "sufficientvelocity"),
    ("qq", "questionablequesting"),
    ("questionablequesting", "questionablequesting"),
    ("ao3", "archiveofourown"),
    ("archiveofourown", "archiveofourown"),
    ("ffn", "fanfiction.net"),
    ("fanfiction.net", "fanfiction.net"),
    ("ff.net", "fanfiction.net"),
    ("royalroad", "royalroad"),
    ("rr", "royalroad"),
    ("scribblehub", "scribblehub"),
    ("sh", "scribblehub"),
];

/// Strip a trailing site mention from the end of the input.
/// Patterns: " on <site>", " from <site>", " (<site>)", " - <site>"
/// Returns the stripped string and the canonical site key if found.
fn strip_site_alias(input: &str) -> (&str, Option<String>) {
    let seps = [" on ", " from ", " (", " - "];
    for sep in &seps {
        if let Some(pos) = input.rfind(sep) {
            let site_part = &input[pos + sep.len()..];
            let site_part = site_part.trim_end_matches(')');
            for (alias, canonical_key) in SITE_ALIASES {
                if site_part.eq_ignore_ascii_case(alias) {
                    let stripped = &input[..pos];
                    return (stripped.trim_end(), Some(canonical_key.to_string()));
                }
            }
        }
    }
    (input, None)
}

/// Pure parser for a find-fic query string into ParsedFindQuery.
/// Rules:
/// - If input contains "://" -> return all None (URL already exists)
/// - Otherwise strip site aliases from the end first
/// - Then split on the LAST " by " only (not first)
/// - Strip surrounding quotes/asterisks (markdown italics) from both parts
/// - Empty title or author parts are omitted from the query
pub fn parse_find_query(raw: &str) -> ParsedFindQuery {
    // If the input contains a URL, don't parse further.
    if raw.contains("://") {
        return ParsedFindQuery {
            title: None,
            author: None,
            site: None,
            raw: raw.to_string(),
        };
    }

    // Step 1: Strip site alias from the end
    let (cleaned, site) = strip_site_alias(raw);

    // Step 2: Split on the LAST " by " only (not the first)
    let last_by_pos = cleaned.rfind(" by ");
    let title: Option<String>;
    let author: Option<String>;

    if let Some(pos) = last_by_pos {
        let before = &cleaned[..pos];
        let after = &cleaned[pos + 4..]; // skip " by "
        title = Some(strip_markdown(before.trim()).to_string());
        author = Some(strip_markdown(after.trim()).to_string());
    } else {
        title = Some(strip_markdown(cleaned.trim()).to_string());
        author = None;
    }

    ParsedFindQuery {
        title,
        author,
        site,
        raw: raw.to_string(),
    }
}

/// Strip surrounding quotes, asterisks, and apostrophes (markdown emphasis)
fn strip_markdown(s: &str) -> &str {
    s.trim_matches(|c| c == '"' || c == '*' || c == '\'')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_governors_gambit() {
        let parsed = parse_find_query("Governor's Gambit by Freefaller");
        assert_eq!(parsed.title.as_deref(), Some("Governor's Gambit"));
        assert_eq!(parsed.author.as_deref(), Some("Freefaller"));
    }

    #[test]
    fn test_parse_war_by_other_means() {
        // "War by Other Means by Author" should split at the LAST " by "
        // Left: "War by Other Means", Right: "Author"
        let parsed = parse_find_query("War by Other Means by Author");
        assert_eq!(parsed.title.as_deref(), Some("War by Other Means"));
        assert_eq!(parsed.author.as_deref(), Some("Author"));
    }

    #[test]
    fn test_parse_markdown_italics() {
        let parsed = parse_find_query("*Title* by Author (AO3)");
        assert_eq!(parsed.title.as_deref(), Some("Title"));
        assert_eq!(parsed.author.as_deref(), Some("Author"));
        assert_eq!(parsed.site.as_deref(), Some("archiveofourown"));
    }

    #[test]
    fn test_parse_title_only() {
        let parsed = parse_find_query("Just A Title");
        assert_eq!(parsed.title.as_deref(), Some("Just A Title"));
        assert_eq!(parsed.author, None);
    }

    #[test]
    fn test_parse_url_passthrough() {
        let parsed = parse_find_query("https://example.com/fic/123");
        assert_eq!(parsed.title, None);
        assert_eq!(parsed.author, None);
        assert_eq!(parsed.site, None);
        assert_eq!(parsed.raw, "https://example.com/fic/123");
    }

    #[test]
    fn test_parse_empty() {
        let parsed = parse_find_query("");
        assert_eq!(parsed.title.as_deref(), Some(""));
        assert_eq!(parsed.author, None);
    }

    #[test]
    fn test_parse_quotes() {
        let parsed = parse_find_query("\"My Title\" by Author");
        assert_eq!(parsed.title.as_deref(), Some("My Title"));
        assert_eq!(parsed.author.as_deref(), Some("Author"));
    }

    #[test]
    fn test_parse_single_by_author() {
        let parsed = parse_find_query("Something by Author");
        assert_eq!(parsed.title.as_deref(), Some("Something"));
        assert_eq!(parsed.author.as_deref(), Some("Author"));
    }

    #[test]
    fn test_parse_on_spacebattles() {
        let parsed = parse_find_query("Governor's Gambit by Freefaller on SpaceBattles");
        assert_eq!(parsed.title.as_deref(), Some("Governor's Gambit"));
        assert_eq!(parsed.author.as_deref(), Some("Freefaller"));
        assert_eq!(parsed.site.as_deref(), Some("spacebattles"));
    }

    #[test]
    fn test_parse_sb_alias() {
        let parsed = parse_find_query("Title by Author on sb");
        assert_eq!(parsed.title.as_deref(), Some("Title"));
        assert_eq!(parsed.author.as_deref(), Some("Author"));
        assert_eq!(parsed.site.as_deref(), Some("spacebattles"));
    }

    #[test]
    fn test_parse_from_royalroad() {
        let parsed = parse_find_query("Title by Author - RoyalRoad");
        assert_eq!(parsed.site.as_deref(), Some("royalroad"));
    }
}
