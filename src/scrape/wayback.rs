//! Wayback Machine fallback — fetch a fic's last good snapshot from the
//! Internet Archive when the native scraper is blocked (403/404/transient).
//!
//! Design (per `docs/plans/wayback-fallback.md`):
//! * CDX API client (`get_latest_snapshot_url`) — 1 req/sec local rate limit.
//! * HTML cleaner (`clean_wayback_html`) — strips the `#wm-ipp` toolbar +
//!   archive.org analytics, optionally injects `<base href>`.
//! * `fetch_snapshot_html` — fetch + clean a snapshot for the heal path.
//!
//! The fallback is a FicNexus-level wrapper, NOT inside `fanfic-scrapers`:
//! the crate stays a pure site-agnostic adapter registry.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{Duration as ChronoDuration, Utc};
use serde_json::Value;

/// Default user agent for Wayback queries — identifies the bot.
pub const WAYBACK_USER_AGENT: &str =
    "FicNexus-Archive-Bot/1.0 (+https://ficnexus.polarisocial.xyz)";

/// Minimal rate limiter for the CDX API: one request per second.
/// Uses an atomic timestamp (monotonic) — process-local is fine because
/// FicNexus runs a single instance.
#[derive(Debug, Default)]
pub struct CdxRateLimiter {
    last_request_ms: AtomicU64,
}

impl CdxRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Wait until at least `min_interval` has elapsed since the last call.
    pub async fn wait(&self, min_interval: Duration) {
        let now = now_ms();
        let last = self.last_request_ms.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);
        if elapsed < min_interval.as_millis() as u64 {
            tokio::time::sleep(Duration::from_millis(
                min_interval.as_millis() as u64 - elapsed,
            ))
            .await;
        }
        self.last_request_ms.store(now_ms(), Ordering::Relaxed);
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Configuration for the Wayback fallback.
#[derive(Debug, Clone)]
pub struct WaybackConfig {
    pub enabled: bool,
    pub max_snapshot_age_days: i32,
    pub cdx_rate_limit_per_sec: u64,
}

impl WaybackConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_snapshot_age_days: 365,
            cdx_rate_limit_per_sec: 1,
        }
    }
}

/// Service exposed on `AppState`. Holds the CDX rate limiter so every
/// export path shares one global pace (1 req/sec default).
#[derive(Debug)]
pub struct WaybackService {
    pub config: WaybackConfig,
    pub limiter: CdxRateLimiter,
}

impl WaybackService {
    pub fn from_config(cfg: &crate::config::Config) -> Self {
        Self {
            config: WaybackConfig {
                enabled: cfg.wayback_fallback_enabled,
                max_snapshot_age_days: cfg.wayback_max_snapshot_age_days,
                cdx_rate_limit_per_sec: cfg.wayback_cdx_rate_limit_per_sec,
            },
            limiter: CdxRateLimiter::new(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            config: WaybackConfig::disabled(),
            limiter: CdxRateLimiter::new(),
        }
    }

    /// Best-effort fallback: fetch + clean the latest Wayback snapshot for
    /// `url`, or `None` when disabled / no snapshot / failure.
    pub async fn fetch_snapshot(&self, http: &reqwest::Client, url: &str) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        fetch_snapshot_html(http, &self.limiter, url, self.config.max_snapshot_age_days).await
    }
}

/// Look up the most recent Wayback snapshot URL for `original_url`.
///
/// Returns `Ok(None)` when no snapshot exists within the age window
/// (not an error — just no fallback available).
pub async fn get_latest_snapshot_url(
    http: &reqwest::Client,
    limiter: &CdxRateLimiter,
    original_url: &str,
    max_age_days: i32,
) -> Result<Option<String>, String> {
    limiter.wait(Duration::from_secs(1)).await;

    let from = (Utc::now() - ChronoDuration::days(max_age_days as i64))
        .format("%Y%m%d")
        .to_string();
    let encoded = urlencoding(&original_url);
    let cdx_url = format!(
        "https://web.archive.org/cdx/search/cdx?url={encoded}&output=json&collapse=urlkey&filter=statuscode:200&limit=1&from={from}"
    );

    let resp = http
        .get(&cdx_url)
        .header("User-Agent", WAYBACK_USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("CDX request failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("CDX returned {}", resp.status()));
    }
    let body = resp
        .text()
        .await
        .map_err(|e| format!("CDX body read failed: {e}"))?;

    parse_cdx_json(&body, original_url)
}

/// Parse the CDX `output=json` response and return the snapshot URL.
/// CDX JSON is `[["urlkey","timestamp","original","mimetype","statuscode",...], [row...]]`.
fn parse_cdx_json(body: &str, original_url: &str) -> Result<Option<String>, String> {
    let val: Value = serde_json::from_str(body).map_err(|e| format!("CDX JSON invalid: {e}"))?;
    let rows = val
        .as_array()
        .ok_or_else(|| "CDX JSON not an array".to_string())?;
    // rows[0] is the header row; data starts at rows[1].
    if rows.len() < 2 {
        return Ok(None);
    }
    // Find the timestamp column index from the header.
    let header = rows[0]
        .as_array()
        .ok_or_else(|| "CDX header not array".to_string())?;
    let ts_idx = header
        .iter()
        .position(|v| v.as_str() == Some("timestamp"))
        .unwrap_or(1);
    let row = rows[1]
        .as_array()
        .ok_or_else(|| "CDX row not array".to_string())?;
    let timestamp = row
        .get(ts_idx)
        .and_then(|v| v.as_str())
        .ok_or_else(|| "CDX row missing timestamp".to_string())?;
    // Ensure the timestamp looks like a 14-digit CDX timestamp.
    if timestamp.len() < 14 || !timestamp.chars().all(|c| c.is_ascii_digit()) {
        return Ok(None);
    }
    Ok(Some(format!(
        "https://web.archive.org/web/{timestamp}/{original_url}"
    )))
}

/// Fetch + clean a Wayback snapshot for `original_url`. Returns the cleaned
/// HTML or `None` on any failure (fallback must fail closed, never open).
/// Returns true when the fallback is enabled and the URL looks Wayback-eligible.
/// Currently gates on FICHUB_FALLBACK_ENABLED so operators can kill-switch
/// the entire fallback path without redeploying.
pub fn wayback_eligible(url: &url::Url) -> bool {
    // Only http/https are eligible.
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    // Archive.org and its mirrors are excluded (already archived).
    if url.host_str() == Some("web.archive.org") || url.host_str() == Some("archive.org") {
        return false;
    }
    // forum.questionablequesting.com is always eligible (primary source).
    if url.host_str() == Some("forum.questionablequesting.com") {
        return true;
    }
    // Other sites: only eligible if fallback is enabled.
    std::env::var("FICHUB_FALLBACK_ENABLED")
        .unwrap_or_default()
        .eq_ignore_ascii_case("false")
}

pub async fn fetch_snapshot_html(
    http: &reqwest::Client,
    limiter: &CdxRateLimiter,
    original_url: &str,
    max_age_days: i32,
) -> Option<String> {
    let snapshot_url = get_latest_snapshot_url(http, limiter, original_url, max_age_days)
        .await
        .ok()??;

    let resp = http
        .get(&snapshot_url)
        .header("User-Agent", WAYBACK_USER_AGENT)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    if body.is_empty() {
        return None;
    }
    Some(clean_wayback_html(&body, original_url))
}

/// Strip the Wayback toolbar + archive.org analytics and inject `<base href>`.
fn clean_wayback_html(html: &str, original_url: &str) -> String {
    let mut out = strip_element(html, "div", "wm-ipp");
    out = strip_element(&out, "div", "wm-ipp-print");
    // Remove archive.org analytics scripts.
    out = strip_script_by_src(&out, "archive.org");
    // Inject <base href> after <head> (or at the top if no <head>).
    inject_base_href(&out, original_url)
}

/// Remove the first element matching `tag` + `id` (naive, case-insensitive).
fn strip_element(html: &str, tag: &str, id: &str) -> String {
    let lower = html.to_lowercase();
    let open = format!("<{tag}");
    let id_attr = format!("id=\"{id}\"");
    let Some(open_idx) = lower.find(&open) else {
        return html.to_string();
    };
    let after_open = &lower[open_idx..];
    if after_open.find(&id_attr).is_none() {
        return html.to_string();
    }
    // The element starts at open_idx; find its matching close tag.
    let elem_start = open_idx;
    let elem_rest = &html[elem_start..];
    let close_tag = format!("</{tag}>");
    let close_idx = elem_rest
        .to_lowercase()
        .find(&close_tag)
        .map(|i| i + close_tag.len())
        .unwrap_or(elem_rest.len());
    format!("{}{}", &html[..elem_start], &elem_rest[close_idx..])
}

/// Remove `<script>` elements whose src points at the given domain.
fn strip_script_by_src(html: &str, domain: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    loop {
        let lower_rest = rest.to_lowercase();
        let Some(open_idx) = lower_rest.find("<script") else {
            out.push_str(rest);
            break;
        };
        // Find the end of the opening tag.
        let tag_end = rest[open_idx..]
            .find('>')
            .map(|i| open_idx + i + 1)
            .unwrap_or(rest.len());
        let open_tag = &rest[open_idx..tag_end];
        // Close tag for the whole element.
        let close_tag = "</script>";
        let after_open = &rest[tag_end..];
        let close_idx = after_open.to_lowercase().find(close_tag);
        let elem_end = close_idx
            .map(|i| tag_end + i + close_tag.len())
            .unwrap_or(rest.len());
        // Keep the element if it doesn't reference the domain.
        if !open_tag.to_lowercase().contains(domain) {
            out.push_str(&rest[..elem_end]);
        }
        rest = &rest[elem_end.min(rest.len())..];
    }
    out
}

/// Inject `<base href="{original_url}">` into `<head>` if present.
fn inject_base_href(html: &str, original_url: &str) -> String {
    let lower = html.to_lowercase();
    if let Some(head_end) = lower.find("</head>") {
        let base = format!("<base href=\"{}\">", original_url.trim_end_matches('/'));
        return format!("{}{}{}", &html[..head_end], base, &html[head_end..]);
    }
    // No head: prepend.
    format!(
        "<head><base href=\"{}\"></head>{}",
        original_url.trim_end_matches('/'),
        html
    )
}

/// Simple URL encoding for query parameters.
fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cdx_json() {
        let body = r#"[["urlkey","timestamp","original","mimetype","statuscode"],
["archive.org/example","20231015010203","https://example.com/story/123","text/html","200"]]"#;
        let url = get_latest_snapshot_url_for_test(body, "https://example.com/story/123");
        assert_eq!(
            url.unwrap(),
            Some(
                "https://web.archive.org/web/20231015010203/https://example.com/story/123"
                    .to_string()
            )
        );
    }

    #[test]
    fn parses_empty_cdx() {
        let body = r#"[["urlkey","timestamp","original","mimetype","statuscode"]]"#;
        assert_eq!(
            get_latest_snapshot_url_for_test(body, "https://example.com/story/123").unwrap(),
            None
        );
    }

    #[test]
    fn strips_wayback_toolbar() {
        let html = r#"<html><head><title>Fic</title></head><body>
<div id="wm-ipp"><div>Wayback toolbar</div></div>
<p>Actual content</p>
</body></html>"#;
        let cleaned = clean_wayback_html(html, "https://example.com/story/123");
        assert!(!cleaned.contains("wm-ipp"));
        assert!(cleaned.contains("Actual content"));
        assert!(cleaned.contains("<base href=\"https://example.com/story/123\">"));
    }

    #[test]
    fn strips_archive_analytics_scripts() {
        let html = r#"<html><head><script src="https://archive.org/includes/analytics.js"></script></head><body><p>ok</p></body></html>"#;
        let cleaned = clean_wayback_html(html, "https://example.com");
        assert!(!cleaned.contains("analytics.js"));
        assert!(cleaned.contains("<p>ok</p>"));
    }

    #[test]
    fn injects_base_href_without_head() {
        let html = "<html><body><p>hi</p></body></html>";
        let cleaned = clean_wayback_html(html, "https://example.com/fic");
        assert!(cleaned.contains("<base href=\"https://example.com/fic\">"));
    }

    // Test helper: call parse directly without the network.
    fn get_latest_snapshot_url_for_test(
        body: &str,
        original_url: &str,
    ) -> Result<Option<String>, String> {
        parse_cdx_json(body, original_url)
    }
}
