//! Cloudflare-aware fetch helper for sites behind CF's "Just a moment" wall
//! (e.g. fanfiction.net). Primary path: TLS fingerprint impersonation via
//! [`primp`] — a fresh client per call, no cookies, no persistence. Fallback:
//! a one-shot headless chromium subprocess (`--dump-dom`) that exits after
//! dumping the page — again no persistence, no cookies carried.
//!
//! Why not the shared `reqwest::Client`? CF binds its challenge to the TLS
//! fingerprint of the client that earned it. reqwest's default rustls
//! fingerprint is rejected; `primp` mimics a real Chrome TLS/HTTP2 fingerprint
//! and passes without any browser. Verified live against FFN (2026-08-15).

use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Minimum spacing between FFN requests. Rapid consecutive hits trigger CF
/// escalation (verified: back-to-back chapter fetches 403 even with primp).
const FFN_MIN_INTERVAL_MS: u64 = 8_000;

static LAST_FFN_FETCH: AtomicU64 = AtomicU64::new(0);

/// The Chrome fingerprint we impersonate. ChromeV144 passed FFN live in tests.
fn impersonate_variant() -> primp::Impersonate {
    primp::Impersonate::ChromeV144
}

/// Detect a genuine Cloudflare interstitial. A real CF challenge page has
/// `<title>Just a moment...</title>` and a challenge form; the `challenge-platform`
/// script src alone appears on EVERY page CF serves (inert, not a wall), so it
/// must NOT be treated as a challenge.
pub fn is_cf_challenge(body: &str) -> bool {
    let title = body
        .split("<title>")
        .nth(1)
        .and_then(|s| s.split("</title>").next())
        .unwrap_or("")
        .to_lowercase();
    title.contains("just a moment")
        || (title.contains("attention required") && body.contains("challenge"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cf_challenge_bodies() {
        // Real interstitial: title "Just a moment" + challenge form.
        assert!(is_cf_challenge(
            "<html><head><title>Just a moment...</title></head><body><form class=\"challenge-form\">...</form></body></html>"
        ));
        // Real page WITH inert challenge-platform script src must NOT be flagged.
        assert!(!is_cf_challenge(
            "<html><head><title>Jillian Holtzmann: Ace Attorney Chapter 1</title><script src=\"/cdn-cgi/challenge-platform/scripts/jsd/main.js\"></script></head><body><div class=\"storytext\">story</div></body></html>"
        ));
        assert!(!is_cf_challenge(""));
        // Attention-required title with challenge body.
        assert!(is_cf_challenge(
            "<html><head><title>Attention Required! | Cloudflare</title></head><body>challenge</body></html>"
        ));
    }
}

/// Enforce the per-domain minimum interval (best-effort; async-friendly via
/// the atomic + sleep since FFN fetches are rare and this only spaces them).
async fn rate_limit_spacing() {
    let now = std::time::Instant::now();
    let last = LAST_FFN_FETCH.load(Ordering::Relaxed);
    let elapsed_ms = now.elapsed().as_millis() as u64;
    let wait = FFN_MIN_INTERVAL_MS.saturating_sub(elapsed_ms + last);
    if wait > 0 {
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }
    LAST_FFN_FETCH.store(now.elapsed().as_millis() as u64, Ordering::Relaxed);
}

/// Fetch `url` as HTML, passing Cloudflare via TLS impersonation first and a
/// one-shot headless browser as fallback. Returns the raw HTML body.
pub async fn fetch_with_cf_fallback(url: &str) -> Result<String, crate::ScrapeError> {
    // 1. Primary: TLS impersonation (primp).
    let impersonated = fetch_impersonated(url).await;
    match impersonated {
        Ok(body) if !is_cf_challenge(&body) => return Ok(body),
        Ok(_) => {
            // CF challenge slipped through; fall through to browser.
            tracing::warn!("primp returned CF challenge for {url}, falling back to chromium");
        }
        Err(e) => {
            tracing::warn!("primp failed for {url}: {e}, falling back to chromium");
        }
    }

    // 2. Fallback: one-shot headless chromium, no persistence. Space it too —
    //    running chromium immediately after a primp request can trip CF's
    //    rapid-request escalation (verified: chromium exits 13 with empty DOM).
    rate_limit_spacing().await;
    fetch_via_chromium(url).await
}

/// Primary: TLS impersonation via primp (fresh client, no cookies).
async fn fetch_impersonated(url: &str) -> Result<String, crate::ScrapeError> {
    rate_limit_spacing().await;

    let client = primp::Client::builder()
        .impersonate(impersonate_variant())
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| crate::ScrapeError::Network(e.to_string()))?;

    let resp = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| crate::ScrapeError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(crate::ScrapeError::NotFound);
    }
    resp.text()
        .await
        .map_err(|e| crate::ScrapeError::Network(e.to_string()))
}

/// Fallback: one-shot headless chromium, dump DOM, exit. No persistence.
async fn fetch_via_chromium(url: &str) -> Result<String, crate::ScrapeError> {
    // Need tokio for async; the command itself is sync/blocking but short-lived.
    // Use spawn_blocking to avoid stalling the async runtime on process wait.
    let url = url.to_string();
    tokio::task::spawn_blocking(move || {
        let ua = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
        let output = Command::new("chromium")
            .args([
                "--headless=new",
                "--no-sandbox",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--dump-dom",
                "--virtual-time-budget=20000",
                "--user-agent",
                ua,
            ])
            .arg(url.as_str())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| crate::ScrapeError::Network(format!("chromium spawn: {e}")))?;

        if !output.status.success() {
            return Err(crate::ScrapeError::Network(format!(
                "chromium exited {:?}",
                output.status.code()
            )));
        }
        String::from_utf8(output.stdout)
            .map_err(|e| crate::ScrapeError::Network(format!("chromium output: {e}")))
    })
    .await
    .map_err(|e| crate::ScrapeError::Network(format!("chromium task join: {e}")))?
}
