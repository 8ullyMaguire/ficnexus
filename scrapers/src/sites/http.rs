//! Shared HTTP helpers for site adapters.

use crate::ScrapeError;

/// Browser-ish User-Agent used for sites that block plain library agents.
pub const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0";

/// Extract the bare host from a URL (e.g. `https://www.fanficauthors.net/s/x`
/// → `fanficauthors.net`). Used for credential matching.
pub fn host_of(url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_lowercase()
}

/// GET a URL and return the body text. Returns `Network` on transport/HTTP
/// errors.
pub async fn fetch(client: &reqwest::Client, url: &str) -> Result<String, ScrapeError> {
    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ScrapeError::Network(format!("HTTP {}", resp.status())));
    }
    resp.text()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))
}

/// POST an application/x-www-form-urlencoded form and return the body text.
/// Used for site logins (the client must have a cookie store enabled so
/// session cookies set by the response persist).
pub async fn post_form(
    client: &reqwest::Client,
    url: &str,
    fields: &[(&str, String)],
) -> Result<String, ScrapeError> {
    let resp = client
        .post(url)
        .header("User-Agent", USER_AGENT)
        .form(
            &fields
                .iter()
                .cloned()
                .collect::<std::collections::HashMap<_, _>>(),
        )
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ScrapeError::Network(format!("HTTP {}", resp.status())));
    }
    resp.text()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))
}
