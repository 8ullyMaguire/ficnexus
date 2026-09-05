//! Visitor funnel: anonymous tracking via signed cookie.
//!
//! Issue #6: anonymous users get trust-0 functionality. Their state
//! (ratings, saved searches, bookmarks) is stored under a visitor ID
//! and merged into their account on registration.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use cookie::{Cookie, SameSite};

use crate::visitor::VisitorId;

/// Name of the signed visitor ID cookie.
pub const VISITOR_COOKIE_NAME: &str = "vh_vis";

/// Max age of the visitor cookie (30 days, in seconds).
pub const VISITOR_COOKIE_MAX_AGE_SECONDS: i64 = 60 * 60 * 24 * 30;

/// Extract a visitor ID from the request cookies.
pub fn extract_visitor_id(request: &Request) -> Option<VisitorId> {
    let cookies = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())?;

    for cookie_str in cookies.split(';') {
        let cookie_str = cookie_str.trim();
        if let Ok(cookie) = Cookie::parse_encoded(cookie_str) {
            if cookie.name() == VISITOR_COOKIE_NAME {
                return VisitorId::parse(cookie.value());
            }
        }
    }
    None
}

/// Create a visitor cookie for a new visitor.
pub fn create_visitor_cookie(visitor_id: &VisitorId) -> Cookie<'static> {
    let mut cookie = Cookie::new(VISITOR_COOKIE_NAME, visitor_id.to_cookie_string());
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie.set_max_age(time::Duration::seconds(VISITOR_COOKIE_MAX_AGE_SECONDS));
    cookie
}

/// Middleware that ensures a visitor ID cookie is present.
///
/// If the incoming request already has a `vh_vis` cookie, its `VisitorId`
/// is exposed to downstream handlers via `request.extensions()`. If not,
/// a fresh ID is minted and attached to the request extensions.
/// A `Set-Cookie` header is only added when `fh_consent=granted` is present
/// (GDPR/privacy compliance — no tracking cookies without consent).
pub async fn visitor_middleware(mut request: Request, next: Next) -> Response {
    // Extract headers before consuming the request.
    let existing = extract_visitor_id(&request);
    let has_cookie = existing.is_some();
    let consented = crate::visitor::has_cookie_consent(request.headers());

    // Always expose a VisitorId to handlers, even for the first request.
    let visitor_id = existing.unwrap_or_else(VisitorId::new);
    request.extensions_mut().insert(visitor_id);

    let mut response = next.run(request).await;

    // Only set the visitor cookie if the user has given cookie consent.
    if !has_cookie && consented {
        let cookie = create_visitor_cookie(&visitor_id);
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            response.headers_mut().append("set-cookie", header_value);
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_visitor_cookie_has_correct_name() {
        let id = VisitorId::new();
        let cookie = create_visitor_cookie(&id);
        assert_eq!(cookie.name(), VISITOR_COOKIE_NAME);
    }

    #[test]
    fn visitor_cookie_is_http_only() {
        let id = VisitorId::new();
        let cookie = create_visitor_cookie(&id);
        assert!(cookie.http_only().unwrap_or(false));
    }

    #[test]
    fn visitor_cookie_path_is_root() {
        let id = VisitorId::new();
        let cookie = create_visitor_cookie(&id);
        assert_eq!(cookie.path().unwrap_or(""), "/");
    }

    #[test]
    fn visitor_cookie_max_age_is_30_days() {
        let id = VisitorId::new();
        let cookie = create_visitor_cookie(&id);
        let max_age = cookie.max_age();
        assert!(max_age.is_some());
        let seconds = max_age.unwrap().whole_seconds();
        assert_eq!(seconds, 60 * 60 * 24 * 30);
    }

    #[test]
    fn extract_returns_none_when_cookie_absent() {
        // No Cookie header → no visitor id.
        let req = axum::extract::Request::new(axum::body::Body::empty());
        assert!(extract_visitor_id(&req).is_none());
    }

    #[test]
    fn extract_parses_present_cookie() {
        let id = VisitorId::new();
        let cookie_str = format!("{VISITOR_COOKIE_NAME}={}", id.to_cookie_string());
        let mut req = axum::extract::Request::new(axum::body::Body::empty());
        req.headers_mut().insert(
            "cookie",
            cookie_str.parse().expect("valid header value"),
        );
        let parsed = extract_visitor_id(&req).expect("present");
        assert_eq!(parsed, id);
    }

    #[test]
    fn extract_ignores_unrelated_cookies() {
        let mut req = axum::extract::Request::new(axum::body::Body::empty());
        req.headers_mut()
            .insert("cookie", "foo=bar; other=stuff".parse().unwrap());
        assert!(extract_visitor_id(&req).is_none());
    }
}
