//! Anonymous visitor identification and state management.
//!
//! Issue #6: anonymous users should get trust-0 functionality, with their
//! data merging into their account on registration. A signed cookie
//! `fh_vis` (UUID v4) identifies the visitor across requests.

pub mod middleware;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export so callers can do `visitor::http` without depending on axum.
pub use ::http;

/// True iff the request carries `fh_consent=granted`.
///
/// The consent toast (`frontend/src/lib/stores/consent.svelte.ts`) syncs
/// user choice to a `fh_consent` cookie so the backend has a stable
/// signal. Absent cookie or any other value (`denied`, garbage) = no
/// consent. We do NOT try to read localStorage server-side.
pub fn has_cookie_consent(headers: &http::HeaderMap) -> bool {
    headers
        .get(http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|cookies| cookies.split(';').any(|c| c.trim() == "fh_consent=granted"))
        .unwrap_or(false)
}

/// Visitor identifier — a UUID v4 stored in a signed cookie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VisitorId(pub Uuid);

impl VisitorId {
    /// Generate a fresh random visitor ID.
    pub fn new() -> Self {
        VisitorId(Uuid::new_v4())
    }

    /// Parse from a cookie string.
    pub fn parse(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(VisitorId)
    }

    /// Serialize to string for cookie storage.
    pub fn to_cookie_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for VisitorId {
    fn default() -> Self {
        Self::new()
    }
}

/// Cookie name for the visitor ID.
pub const VISITOR_COOKIE_NAME: &str = "fh_vis";

/// Cookie lifetime in seconds (90 days).
pub const VISITOR_COOKIE_MAX_AGE: i64 = 90 * 24 * 60 * 60;

/// Visitor-scoped state that gets merged into a user account on registration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VisitorState {
    /// Work ratings: url_id → rating (1-5).
    pub ratings: serde_json::Value,
    /// Saved search queries.
    pub saved_searches: serde_json::Value,
    /// Bookmarked work IDs.
    pub bookmarks: serde_json::Value,
}

impl VisitorState {
    pub fn is_empty(&self) -> bool {
        let ratings_empty = match &self.ratings {
            serde_json::Value::Object(r) => r.is_empty(),
            serde_json::Value::Null => true,
            _ => false,
        };
        let searches_empty = match &self.saved_searches {
            serde_json::Value::Object(s) => s.is_empty(),
            serde_json::Value::Null => true,
            _ => false,
        };
        let bookmarks_empty = match &self.bookmarks {
            serde_json::Value::Object(b) => b.is_empty(),
            serde_json::Value::Null => true,
            _ => false,
        };
        ratings_empty && searches_empty && bookmarks_empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visitor_id_roundtrip() {
        let id = VisitorId::new();
        let s = id.to_cookie_string();
        let parsed = VisitorId::parse(&s).expect("parses");
        assert_eq!(id, parsed);
    }

    #[test]
    fn visitor_id_parse_rejects_garbage() {
        assert!(VisitorId::parse("not-a-uuid").is_none());
        assert!(VisitorId::parse("").is_none());
    }

    #[test]
    fn visitor_state_empty_by_default() {
        let state = VisitorState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn visitor_state_not_empty_with_ratings() {
        let state = VisitorState {
            ratings: serde_json::json!({"work_123": 5}),
            ..Default::default()
        };
        assert!(!state.is_empty());
    }

    #[test]
    fn cookie_name_is_expected() {
        assert_eq!(VISITOR_COOKIE_NAME, "fh_vis");
    }

    #[test]
    fn cookie_lifetime_is_90_days() {
        assert_eq!(VISITOR_COOKIE_MAX_AGE, 90 * 24 * 60 * 60);
    }

    // ── has_cookie_consent ────────────────────────────────────────

    fn header_map_from_cookie(cookie: &str) -> http::HeaderMap {
        let mut h = http::HeaderMap::new();
        h.insert(http::header::COOKIE, cookie.parse().expect("valid"));
        h
    }

    #[test]
    fn consent_granted_returns_true() {
        let h = header_map_from_cookie("fh_consent=granted");
        assert!(has_cookie_consent(&h));
    }

    #[test]
    fn consent_denied_returns_false() {
        let h = header_map_from_cookie("fh_consent=denied");
        assert!(!has_cookie_consent(&h));
    }

    #[test]
    fn no_cookie_header_returns_false() {
        let h = http::HeaderMap::new();
        assert!(!has_cookie_consent(&h));
    }

    #[test]
    fn consent_among_many_cookies() {
        let h = header_map_from_cookie("foo=bar; fh_consent=granted; baz=qux");
        assert!(has_cookie_consent(&h));
    }

    #[test]
    fn consent_with_spaces_around() {
        let h = header_map_from_cookie("  fh_consent=granted  ;  other=x");
        assert!(has_cookie_consent(&h));
    }

    #[test]
    fn garbage_consent_value_returns_false() {
        let h = header_map_from_cookie("fh_consent=garbage");
        assert!(!has_cookie_consent(&h));
    }
}
