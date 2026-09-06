//! Shared login helpers for site adapters that require authentication.
//!
//! Most login flows follow one of two shapes:
//! - simple form POST (username/password to a fixed URL)
//! - CSRF-token flow (GET a page, extract a hidden token, POST it with
//!   credentials)
//!
//! All flows require the host's `reqwest::Client` to have a cookie store
//! enabled so the session cookies set by the login response persist for
//! subsequent story fetches.

use crate::ScrapeError;
use scraper::{Html, Selector};

use super::http;

/// POST a login form and return the response body. `fields` are the form
/// params (username/password/tokens). The client's cookie jar picks up any
/// session cookie set by the response.
pub async fn post_login(
    client: &reqwest::Client,
    url: &str,
    fields: &[(&str, String)],
) -> Result<String, ScrapeError> {
    http::post_form(client, url, fields).await
}

/// Extract a hidden `<input name="X">` value from an HTML page (used by
/// CSRF-token login flows). Returns `None` when the input is missing.
pub fn hidden_input(html: &str, name: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse(&format!("input[name='{name}']")).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr("value").map(|v| v.to_string()))
}

/// Extract a `<meta name="csrf-token">` content value (used by Rails-style
/// sites).
pub fn csrf_meta(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("meta[name='csrf-token']").ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr("content").map(|v| v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_hidden_input() {
        let html = r#"<html><body><form><input type="hidden" name="token" value="abc123"></form></body></html>"#;
        assert_eq!(hidden_input(html, "token").as_deref(), Some("abc123"));
        assert_eq!(hidden_input(html, "missing"), None);
    }

    #[test]
    fn extracts_csrf_meta() {
        let html = r#"<html><head><meta name="csrf-token" content="xyz789"></head></html>"#;
        assert_eq!(csrf_meta(html).as_deref(), Some("xyz789"));
    }
}
