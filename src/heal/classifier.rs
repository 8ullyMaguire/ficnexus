//! Scrape-failure classification for the self-healing loop.
//!
//! Milestone 1: pure-Rust classifier that turns a raw [`ScrapeError`] into an
//! [`ErrorKind`] (stored verbatim in `scrape_failures.error_kind`) and then
//! into a [`Class`] (transient / blocked / structural / systemic) that decides
//! whether a domain is worth an agent diagnose run.
//!
//! Fingerprint: stable sha256 over `domain|kind|normalized-message` so the
//! same failure signature recurring across many fics dedupes to one signature
//! (mirrors the `qa/fingerprint.js` idea from the bug queue).

use sha2::{Digest, Sha256};

use crate::scrape::ScrapeError;

/// Fine-grained error kind persisted in `scrape_failures.error_kind`.
/// Mirrors `ScrapeError` with `Network` split by timeout-ness and adds the
/// `export` and `unknown` kinds produced by the export pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Blocked,
    Timeout,
    Parse,
    NotFound,
    Export,
    Unknown,
}

impl ErrorKind {
    /// The literal stored in `scrape_failures.error_kind` (CHECK-constrained).
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::Blocked => "blocked",
            ErrorKind::Timeout => "timeout",
            ErrorKind::Parse => "parse",
            ErrorKind::NotFound => "not_found",
            ErrorKind::Export => "export",
            ErrorKind::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "blocked" => ErrorKind::Blocked,
            "timeout" => ErrorKind::Timeout,
            "parse" => ErrorKind::Parse,
            "not_found" => ErrorKind::NotFound,
            "export" => ErrorKind::Export,
            _ => ErrorKind::Unknown,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Network errors whose message hints at a timeout/refused/unreachable cause
/// are classified as [`ErrorKind::Timeout`]; everything else is Unknown.
impl From<&ScrapeError> for ErrorKind {
    fn from(e: &ScrapeError) -> Self {
        match e {
            ScrapeError::NotFound => ErrorKind::NotFound,
            ScrapeError::Blocked => ErrorKind::Blocked,
            ScrapeError::ParseError(_) => ErrorKind::Parse,
            ScrapeError::Unsupported(_) | ScrapeError::Internal(_) => ErrorKind::Unknown,
            ScrapeError::AuthRequired(_) => ErrorKind::Blocked,
            ScrapeError::RateLimited(_) => ErrorKind::Blocked,
            ScrapeError::Network(msg) => {
                let lower = msg.to_lowercase();
                if [
                    "timeout",
                    "timed out",
                    "connect",
                    "refused",
                    "unreachable",
                    "dns",
                    "reset",
                ]
                .iter()
                .any(|k| lower.contains(k))
                {
                    ErrorKind::Timeout
                } else {
                    ErrorKind::Unknown
                }
            }
        }
    }
}

/// Coarse healing class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// One-off flake (timeouts, not-found). No healing action.
    Transient,
    /// The site is refusing us (rate-limit, bot block, IP ban). Needs
    /// throttling/captcha handling, not a code fix.
    Blocked,
    /// The site changed its DOM / our parser no longer matches. A code fix
    /// (scraper selector update) is the right intervention.
    Structural,
    /// Our own pipeline failed (export layer), independent of the site.
    Systemic,
}

impl Class {
    pub fn as_str(&self) -> &'static str {
        match self {
            Class::Transient => "transient",
            Class::Blocked => "blocked",
            Class::Structural => "structural",
            Class::Systemic => "systemic",
        }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Domains where a 403/blocked signal is a *site-side* bot block rather than
/// a scraper bug (AO3/FFN actively block datacenter IPs and heavy scrapers).
fn is_known_anti_bot_domain(domain: &str) -> bool {
    let d = domain.to_lowercase();
    d.contains("archiveofourown.org")
        || d.contains("fanfiction.net")
        || d.contains("fictionpress")
        || d.contains("fimfiction")
        || d.contains("spacebattles")
        || d.contains("sufficientvelocity")
        || d.contains("questionablequesting")
        || d.contains("royalroad")
}

/// Map a persisted error kind + message to a healing class.
///
/// Base mapping: Timeout→Transient, NotFound→Transient, Blocked→Blocked,
/// Parse→Structural, Export→Systemic (unless the message hints at a network
/// timeout, which makes it Transient), Unknown→Transient (assume flake).
///
/// Domain hint: on anti-bot-heavy domains a Blocked error stays Blocked even
/// if the message does not literally say 403 — those sites block at the edge
/// with generic messages. On other domains a Blocked error whose message does
/// NOT hint at a block/403 is demoted to Structural (the "blocked" signal
/// usually means our parser hit an interstitial).
pub fn classify(kind: &ErrorKind, domain: &str, message: &str) -> Class {
    let lower = message.to_lowercase();
    let hints_block = [
        "403",
        "forbidden",
        "blocked",
        "cloudflare",
        "captcha",
        "rate limit",
        "too many requests",
        "429",
        "bot",
    ];
    let hints_timeout = [
        "timeout",
        "timed out",
        "connect",
        "refused",
        "unreachable",
        "dns",
        "reset",
    ];

    match kind {
        ErrorKind::Timeout => Class::Transient,
        ErrorKind::NotFound => Class::Transient,
        ErrorKind::Blocked => {
            if is_known_anti_bot_domain(domain) {
                Class::Blocked
            } else if hints_block.iter().any(|h| lower.contains(h)) {
                Class::Blocked
            } else {
                Class::Structural
            }
        }
        ErrorKind::Parse => Class::Structural,
        ErrorKind::Export => {
            if hints_timeout.iter().any(|h| lower.contains(h)) {
                Class::Transient
            } else {
                Class::Systemic
            }
        }
        ErrorKind::Unknown => Class::Transient,
    }
}

/// Normalize a message before hashing: collapse runs of digits/whitespace so
/// story ids, chapter numbers and timestamps don't fork the fingerprint.
fn normalize_message(msg: &str) -> String {
    let mut out = String::with_capacity(msg.len());
    let mut in_digit_run = false;
    for c in msg.chars() {
        if c.is_ascii_digit() {
            if !in_digit_run {
                out.push('#');
                in_digit_run = true;
            }
        } else {
            in_digit_run = false;
            if c.is_whitespace() {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
            } else {
                out.push(c.to_ascii_lowercase());
            }
        }
    }
    out.trim().to_string()
}

/// Stable 12-hex fingerprint of `domain|kind|normalized-message`.
/// The same failure signature across many fics yields the same fingerprint.
pub fn fingerprint(url: &str, kind: &ErrorKind, message: &str) -> String {
    let domain = url_domain(url).unwrap_or_else(|| url.to_string());
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update(b"|");
    hasher.update(kind.as_str().as_bytes());
    hasher.update(b"|");
    hasher.update(normalize_message(message).as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..6])
}

/// Extract the registrable-ish domain (host) from a URL string.
/// Best-effort: returns `None` for URLs without a `://` scheme + host.
pub fn url_domain(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host.is_empty() {
        return None;
    }
    // Strip userinfo (rare, but `http://user:pass@host/...` must not leak).
    let host = host.rsplit('@').next().unwrap_or(host);
    // Strip port.
    let host = host.split(':').next().unwrap_or(host);
    Some(host.to_lowercase())
}

/// Whether a URL is plausibly a fanfic URL worth recording when no scraper
/// handles it (registry miss). Keep it simple: host has a dot and the path
/// (after the host) is more than one character — this is exactly the gate the
/// plan asks for.
pub fn looks_like_fic_url(url: &str) -> bool {
    match url_domain(url) {
        Some(domain) => {
            let has_dot = domain.contains('.');
            let path_len = url
                .split("://")
                .nth(1)
                .map(|rest| {
                    let after_host = rest
                        .split(['/', '?', '#'])
                        .skip(1)
                        .collect::<Vec<_>>()
                        .join("/");
                    // Reconstruct: "/" + remaining segments (or the raw
                    // remainder if the URL had no slash, e.g. "host").
                    let full_path = if after_host.is_empty() {
                        // No path at all → path len 0.
                        String::new()
                    } else {
                        format!("/{after_host}")
                    };
                    full_path.len()
                })
                .unwrap_or(0);
            has_dot && path_len > 1
        }
        None => false,
    }
}

/// Debounce: true when `>= 3` STRUCTURAL-class failures for the same domain
/// occurred within `window_minutes`. The caller passes failures already
/// filtered to a single domain; rows are classified via [`classify`] and only
/// `Class::Structural` counts (parse errors, blocked-without-block-hint on
/// non-anti-bot domains). Per-fingerprint 24h dedup is enforced separately at
/// the agent-run level (one diagnose per signature per day).
pub fn should_heal(existing: &[crate::heal::store::ScrapeFailureRow], window_minutes: i64) -> bool {
    let now = chrono::Utc::now();
    let cutoff = now - chrono::Duration::minutes(window_minutes);
    let structural_count = existing
        .iter()
        .filter(|r| r.created_at >= cutoff)
        .filter(|r| {
            let kind = ErrorKind::from_str(&r.error_kind);
            classify(&kind, &r.domain, r.message.as_deref().unwrap_or("")) == Class::Structural
        })
        .count();
    structural_count >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_kind_maps_scrape_error() {
        assert_eq!(ErrorKind::from(&ScrapeError::Blocked), ErrorKind::Blocked);
        assert_eq!(ErrorKind::from(&ScrapeError::NotFound), ErrorKind::NotFound);
        assert_eq!(
            ErrorKind::from(&ScrapeError::ParseError("no h1".into())),
            ErrorKind::Parse
        );
        assert_eq!(
            ErrorKind::from(&ScrapeError::Network("connection timed out".into())),
            ErrorKind::Timeout
        );
        assert_eq!(
            ErrorKind::from(&ScrapeError::Network("connection refused".into())),
            ErrorKind::Timeout
        );
        assert_eq!(
            ErrorKind::from(&ScrapeError::Network("tls handshake failed".into())),
            ErrorKind::Unknown
        );
    }

    #[test]
    fn class_mapping_basics() {
        assert_eq!(
            classify(&ErrorKind::Timeout, "x.com", "timeout"),
            Class::Transient
        );
        assert_eq!(
            classify(&ErrorKind::NotFound, "x.com", "404"),
            Class::Transient
        );
        assert_eq!(
            classify(&ErrorKind::Parse, "x.com", "no h1"),
            Class::Structural
        );
        assert_eq!(
            classify(&ErrorKind::Export, "x.com", "epub failed"),
            Class::Systemic
        );
        assert_eq!(
            classify(&ErrorKind::Unknown, "x.com", "weird"),
            Class::Transient
        );
    }

    #[test]
    fn export_with_timeout_hint_is_transient() {
        assert_eq!(
            classify(
                &ErrorKind::Export,
                "x.com",
                "connection refused while converting"
            ),
            Class::Transient
        );
    }

    #[test]
    fn blocked_on_antibot_domain_stays_blocked() {
        assert_eq!(
            classify(
                &ErrorKind::Blocked,
                "archiveofourown.org",
                "got an error page"
            ),
            Class::Blocked
        );
        assert_eq!(
            classify(&ErrorKind::Blocked, "fanfiction.net", "empty body"),
            Class::Blocked
        );
    }

    #[test]
    fn blocked_elsewhere_without_hint_is_structural() {
        assert_eq!(
            classify(&ErrorKind::Blocked, "somesite.org", "empty body"),
            Class::Structural
        );
        assert_eq!(
            classify(&ErrorKind::Blocked, "somesite.org", "403 forbidden"),
            Class::Blocked
        );
    }

    #[test]
    fn fingerprint_is_stable_and_sensitive_to_kind() {
        let a = fingerprint(
            "https://example.com/story/123",
            &ErrorKind::Parse,
            "missing h1 on chapter 5",
        );
        let b = fingerprint(
            "https://example.com/story/456",
            &ErrorKind::Parse,
            "missing h1 on chapter 9",
        );
        // Story ids / chapter numbers normalize away: same signature.
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));

        // Different kind → different fingerprint.
        let c = fingerprint(
            "https://example.com/story/123",
            &ErrorKind::Timeout,
            "missing h1 on chapter 5",
        );
        assert_ne!(a, c);
    }

    #[test]
    fn fingerprint_ignores_digit_and_case_churn() {
        let a = fingerprint(
            "https://example.com/s/12345678/1/",
            &ErrorKind::Parse,
            "Chapter 42 exploded",
        );
        let b = fingerprint(
            "https://example.com/s/99999999/2/",
            &ErrorKind::Parse,
            "chapter 7 exploded",
        );
        assert_eq!(a, b);
    }

    #[test]
    fn fingerprint_domain_isolates_sites() {
        let a = fingerprint("https://site-a.example/story/1", &ErrorKind::Parse, "boom");
        let b = fingerprint("https://site-b.example/story/1", &ErrorKind::Parse, "boom");
        assert_ne!(a, b);
    }

    #[test]
    fn looks_like_fic_url_gate() {
        assert!(looks_like_fic_url("https://archiveofourown.org/works/123"));
        assert!(looks_like_fic_url(
            "https://www.fanfiction.net/s/123/1/Title"
        ));
        assert!(!looks_like_fic_url("https://example.com"));
        assert!(!looks_like_fic_url("not a url"));
        assert!(!looks_like_fic_url("https://localhost:8000/x"));
    }

    #[test]
    fn url_domain_parses_ports_and_userinfo() {
        assert_eq!(
            url_domain("https://www.fanfiction.net/s/1/").as_deref(),
            Some("www.fanfiction.net")
        );
        assert_eq!(
            url_domain("https://example.com:8443/path").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            url_domain("http://user:pass@host.example/x").as_deref(),
            Some("host.example")
        );
        assert_eq!(url_domain("nonsense"), None);
    }

    #[test]
    fn debounce_requires_three_same_fingerprint_in_window() {
        use crate::heal::store::ScrapeFailureRow;
        let now = chrono::Utc::now();
        let row = |fp: &str, mins_ago: i64| ScrapeFailureRow {
            id: 1,
            url: "https://example.com/s/1".into(),
            url_id: None,
            domain: "example.com".into(),
            error_kind: "parse".into(),
            message: None,
            html_snapshot_path: None,
            fingerprint: fp.into(),
            created_at: now - chrono::Duration::minutes(mins_ago),
            resolved_at: None,
            resolution: None,
        };
        // Two in window → no heal.
        let two = vec![row("fp-a", 1), row("fp-a", 2)];
        assert!(!should_heal(&two, 60));
        // Three in window → heal.
        let three = vec![row("fp-a", 1), row("fp-a", 2), row("fp-a", 3)];
        assert!(should_heal(&three, 60));
        // Same count but scattered across fingerprints → still heals: the
        // debounce is per-DOMAIN structural count, not per-fingerprint.
        let scattered = vec![row("fp-a", 1), row("fp-b", 2), row("fp-c", 3)];
        assert!(should_heal(&scattered, 60));
        // Old rows fall outside the window → no heal.
        let old = vec![row("fp-a", 61), row("fp-a", 62), row("fp-a", 63)];
        assert!(!should_heal(&old, 60));
    }
}
