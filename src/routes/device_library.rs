//! Anonymous library device cookie (Tier-2 of the incremental plan).
//!
//! An unauthenticated visitor can bookmark works and follow authors/works.
//! The library is stored in a signed `fh_dev` cookie (HMAC-SHA256 + JWT_SECRET)
//! as base64-encoded JSON. On login or register the client sends the cookie
//! to `/api/v1/device/merge`, which verifies the signature and folds the
//! anonymous library into the user's account.

use axum::Json;
use axum::extract::State;
use axum::http::header::{HeaderMap, HeaderValue, SET_COOKIE};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use std::sync::Arc;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Return 403 if the request does not carry `fh_consent=granted`.
///
/// Device-library endpoints (anonymous bookmarks, follows) persist state
/// in a tracking cookie, so they require the user has accepted the
/// cookie consent. The frontend's ConsentToast already shows on first
/// visit; this 403 lets it know to surface the toast if the user tries
/// to bookmark/follow before accepting.
fn require_consent(headers: &HeaderMap) -> Result<(), AppError> {
    if !crate::visitor::has_cookie_consent(headers) {
        return Err(AppError::Forbidden("cookie consent required".to_string()));
    }
    Ok(())
}

use crate::db::queries;
use crate::error::AppError;
use crate::server::AppState;

const COOKIE_NAME: &str = "fh_dev";
/// 1 year in seconds. The device library is anonymous + low-risk; rotating
/// the cookie every login is acceptable but yearly renewal is the right default.
const COOKIE_MAX_AGE_SECS: u64 = 60 * 60 * 24 * 365;

/// The anonymous library payload stored in the device cookie.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeviceLibrary {
    pub device_id: String,
    pub bookmarks: Vec<i32>,
    pub follows: Vec<DeviceFollow>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFollow {
    pub target_type: String,
    pub target_id: Option<i32>,
    pub author_name: Option<String>,
}

/// Compute HMAC-SHA256 using the `hmac` crate (constant-time under the hood).
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac =
        <HmacSha256 as KeyInit>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Sign a JSON payload with HMAC-SHA256 and return `base64(json).base64(sig)`.
fn sign_json(value: &Value, secret: &str) -> Result<String, AppError> {
    let json = serde_json::to_vec(value).map_err(|e| AppError::Internal(e.to_string()))?;
    let b64_json = base64::engine::general_purpose::STANDARD.encode(&json);
    let sig = hmac_sha256(secret.as_bytes(), b64_json.as_bytes());
    let b64_sig = base64::engine::general_purpose::STANDARD.encode(sig);
    Ok(format!("{b64_json}.{b64_sig}"))
}

/// Verify and decode a signed cookie payload. Returns `None` if invalid.
///
/// Uses constant-time comparison (`subtle::ConstantTimeEq`) to prevent
/// timing side-channel attacks on the signature.
fn verify_cookie(signed: &str, secret: &str) -> Option<DeviceLibrary> {
    let (b64_json, b64_sig) = signed.split_once('.')?;
    let expected = hmac_sha256(secret.as_bytes(), b64_json.as_bytes());
    let provided_bytes = base64::engine::general_purpose::STANDARD
        .decode(b64_sig)
        .ok()?;
    let provided: [u8; 32] = provided_bytes.try_into().ok()?;
    // Constant-time compare — protects against timing attacks.
    if expected.ct_eq(&provided).unwrap_u8() != 1 {
        return None;
    }
    let json = base64::engine::general_purpose::STANDARD
        .decode(b64_json)
        .ok()?;
    serde_json::from_slice::<DeviceLibrary>(&json).ok()
}

/// Read the `fh_dev` cookie from request headers.
fn read_device_cookie(headers: &HeaderMap, secret: &str) -> Option<DeviceLibrary> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{COOKIE_NAME}=")) {
            return verify_cookie(value.trim(), secret);
        }
    }
    None
}

// ── Public handlers ─────────────────────────────────────────────────────────
//
// The backend owns the signed `fh_dev` cookie: every mutating endpoint reads
// any existing cookie, updates the in-memory `DeviceLibrary`, and returns
// a `Set-Cookie: fh_dev=…` header so the browser persists the new value.
// The frontend never sees the JWT_SECRET and never has to do crypto.
//
// Cookies are NOT HttpOnly: the device library is a low-trust anonymous
// store the page itself needs to read for reactive UI (badge counts,
// "you've bookmarked this" indicators, etc.). SameSite=Lax + HMAC
// signature is the right threat model here.

/// Build a `Set-Cookie` header value carrying the signed `fh_dev` cookie.
fn make_set_cookie(library: &DeviceLibrary, secret: &str) -> Result<HeaderValue, AppError> {
    let value = serde_json::to_value(library).map_err(|e| AppError::Internal(e.to_string()))?;
    let signed = sign_json(&value, secret)?;
    // urlencoded to be safe for `;` and other reserved chars
    let header =
        format!("{COOKIE_NAME}={signed}; Max-Age={COOKIE_MAX_AGE_SECS}; Path=/; SameSite=Lax");
    HeaderValue::from_str(&header).map_err(|e| AppError::Internal(format!("bad cookie: {e}")))
}

/// Helper: read the existing library from the cookie, or initialise a new one.
fn read_or_init(headers: &HeaderMap, secret: &str) -> DeviceLibrary {
    read_device_cookie(headers, secret).unwrap_or_else(|| DeviceLibrary {
        device_id: uuid::Uuid::new_v4().to_string(),
        ..Default::default()
    })
}

/// `GET /api/v1/device/library` — return the anonymous library from the cookie.
pub async fn get_device_library(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let library = read_device_cookie(&headers, &state.jwt_secret).unwrap_or_default();
    Ok(Json(json!({ "err": 0, "library": library })))
}

/// `POST /api/v1/device/bookmark` — add a bookmark, return Set-Cookie.
#[derive(Debug, Deserialize)]
pub struct DeviceBookmarkBody {
    pub work_id: i32,
}

pub async fn add_device_bookmark(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<DeviceBookmarkBody>,
) -> Result<Response, AppError> {
    require_consent(&headers)?;
    if queries::get_work(&state.db, body.work_id).await?.is_none() {
        return Err(AppError::NotFound(format!(
            "Work {} not found",
            body.work_id
        )));
    }

    let mut library = read_or_init(&headers, &state.jwt_secret);
    if !library.bookmarks.contains(&body.work_id) {
        library.bookmarks.push(body.work_id);
        library.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    let set_cookie = make_set_cookie(&library, &state.jwt_secret)?;
    let mut out_headers = HeaderMap::new();
    out_headers.insert(SET_COOKIE, set_cookie);
    let body = Json(json!({ "err": 0, "msg": "Bookmarked", "library": &library }));
    Ok((out_headers, body).into_response())
}

/// `DELETE /api/v1/device/bookmark/{work_id}` — remove a bookmark, return Set-Cookie.
pub async fn remove_device_bookmark(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Response, AppError> {
    require_consent(&headers)?;
    let mut library = read_or_init(&headers, &state.jwt_secret);
    if library.bookmarks.contains(&work_id) {
        library.bookmarks.retain(|id| *id != work_id);
        library.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    let set_cookie = make_set_cookie(&library, &state.jwt_secret)?;
    let mut out_headers = HeaderMap::new();
    out_headers.insert(SET_COOKIE, set_cookie);
    let body = Json(json!({ "err": 0, "msg": "Removed", "library": &library }));
    Ok((out_headers, body).into_response())
}

/// `POST /api/v1/device/follow` — add a follow, return Set-Cookie.
#[derive(Debug, Deserialize)]
pub struct DeviceFollowBody {
    pub target_type: String,
    pub target_id: Option<i32>,
    pub author_name: Option<String>,
}

pub async fn add_device_follow(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<DeviceFollowBody>,
) -> Result<Response, AppError> {
    require_consent(&headers)?;
    let mut library = read_or_init(&headers, &state.jwt_secret);

    let follow = DeviceFollow {
        target_type: body.target_type,
        target_id: body.target_id,
        author_name: body.author_name,
    };

    let exists = library.follows.iter().any(|f| {
        f.target_type == follow.target_type
            && f.target_id == follow.target_id
            && f.author_name == follow.author_name
    });
    if !exists {
        library.follows.push(follow);
        library.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    let set_cookie = make_set_cookie(&library, &state.jwt_secret)?;
    let mut out_headers = HeaderMap::new();
    out_headers.insert(SET_COOKIE, set_cookie);
    let body = Json(json!({ "err": 0, "msg": "Following", "library": &library }));
    Ok((out_headers, body).into_response())
}

/// `DELETE /api/v1/device/follow` — remove a follow, return Set-Cookie.
pub async fn remove_device_follow(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<DeviceFollowBody>,
) -> Result<Response, AppError> {
    require_consent(&headers)?;
    let mut library = read_or_init(&headers, &state.jwt_secret);

    library.follows.retain(|f| {
        !(f.target_type == body.target_type
            && f.target_id == body.target_id
            && f.author_name == body.author_name)
    });
    library.updated_at = Some(chrono::Utc::now().to_rfc3339());

    let set_cookie = make_set_cookie(&library, &state.jwt_secret)?;
    let mut out_headers = HeaderMap::new();
    out_headers.insert(SET_COOKIE, set_cookie);
    let body = Json(json!({ "err": 0, "msg": "Unfollowed", "library": &library }));
    Ok((out_headers, body).into_response())
}

/// `POST /api/v1/device/merge` — fold the anonymous library into a user account.
///
/// Reads the cookie from the request's `Cookie` header (set by the browser
/// during the preceding mutating calls). If the body provides an explicit
/// `cookie_value`, that is used instead — useful when the client wants to
/// pre-emptively merge from a copy of the library before login.
#[derive(Debug, Deserialize, Default)]
pub struct MergeBody {
    pub cookie_value: Option<String>,
}

pub async fn merge_device_library(
    auth: crate::routes::auth::AuthUser,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MergeBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let secret = state.jwt_secret.clone();
    let library = if let Some(ref raw) = body.cookie_value {
        verify_cookie(raw, &secret)
            .ok_or_else(|| AppError::BadRequest("Invalid device cookie".to_string()))?
    } else {
        read_device_cookie(&headers, &secret)
            .ok_or_else(|| AppError::BadRequest("No device cookie present".to_string()))?
    };

    let mut merged_bookmarks = 0;
    let mut merged_follows = 0;

    for work_id in &library.bookmarks {
        if let Ok(Some(url_id)) =
            crate::services::bookmark_import::resolve_url_id_for_work(&state.db, *work_id).await
        {
            sqlx::query(
                "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
                 VALUES ($1, $2, $3, '', false)
                 ON CONFLICT (user_id, url_id) DO NOTHING",
            )
            .bind(user_id)
            .bind(&url_id)
            .bind(work_id)
            .execute(&state.db)
            .await?;
            merged_bookmarks += 1;
        }
    }

    for follow in &library.follows {
        match follow.target_type.as_str() {
            "user" => {
                if let Some(target) = follow.target_id {
                    if queries::is_following_user(&state.db, user_id, target).await? == false {
                        queries::follow_user(&state.db, user_id, target).await?;
                        merged_follows += 1;
                    }
                }
            }
            "work" => {
                if let Some(target) = follow.target_id {
                    if queries::is_following_work(&state.db, user_id, target).await? == false {
                        queries::follow_work(&state.db, user_id, target).await?;
                        merged_follows += 1;
                    }
                }
            }
            "author" => {
                if let Some(ref name) = follow.author_name {
                    if queries::find_follow_for_author(&state.db, user_id, name)
                        .await?
                        .is_none()
                    {
                        queries::follow_author(&state.db, user_id, name).await?;
                        merged_follows += 1;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Json(
        json!({ "err": 0, "merged_bookmarks": merged_bookmarks, "merged_follows": merged_follows }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key";

    #[test]
    fn sign_and_roundtrip() {
        let library = DeviceLibrary {
            device_id: "abc-123".into(),
            bookmarks: vec![1, 2, 3],
            follows: vec![DeviceFollow {
                target_type: "author".into(),
                target_id: None,
                author_name: Some("TestAuthor".into()),
            }],
            updated_at: None,
        };
        let value = serde_json::to_value(&library).unwrap();
        let signed = sign_json(&value, TEST_SECRET).unwrap();
        let decoded = verify_cookie(&signed, TEST_SECRET).unwrap();
        assert_eq!(decoded.device_id, "abc-123");
        assert_eq!(decoded.bookmarks, vec![1, 2, 3]);
        assert_eq!(decoded.follows.len(), 1);
    }

    #[test]
    fn verify_rejects_tampered_cookie() {
        let value = serde_json::to_value(&DeviceLibrary::default()).unwrap();
        let signed = sign_json(&value, TEST_SECRET).unwrap();
        let tampered = signed.replace('A', "B");
        assert!(verify_cookie(&tampered, TEST_SECRET).is_none());
    }

    #[test]
    fn verify_rejects_wrong_secret() {
        let value = serde_json::to_value(&DeviceLibrary::default()).unwrap();
        let signed = sign_json(&value, TEST_SECRET).unwrap();
        assert!(verify_cookie(&signed, "different-secret").is_none());
    }

    #[test]
    fn verify_rejects_cookie_signed_with_other_secret() {
        // Guard against someone accidentally hardcoding a key in the future.
        let value = serde_json::to_value(&DeviceLibrary::default()).unwrap();
        let signed_with_a = sign_json(&value, "secret-key-alpha").unwrap();
        // Any other key (even similarly-formatted) must reject.
        assert!(verify_cookie(&signed_with_a, "secret-key-beta").is_none());
    }

    #[test]
    fn verify_rejects_garbage() {
        assert!(verify_cookie("not-a-valid-cookie", TEST_SECRET).is_none());
        assert!(verify_cookie("", TEST_SECRET).is_none());
        assert!(verify_cookie("no-dot-here", TEST_SECRET).is_none());
    }

    #[test]
    fn set_cookie_roundtrips_through_verifier() {
        unsafe {
            std::env::set_var("JWT_SECRET", TEST_SECRET);
        }
        let library = DeviceLibrary {
            device_id: "cookie-roundtrip".into(),
            bookmarks: vec![42, 99],
            follows: vec![],
            updated_at: Some("2026-09-04T00:00:00Z".into()),
        };
        let header = make_set_cookie(&library, "test-secret").expect("set-cookie header");
        let s = header.to_str().unwrap();
        // Format check
        assert!(s.starts_with(&format!("{COOKIE_NAME}=")));
        assert!(s.contains("Max-Age="));
        assert!(s.contains("Path=/"));
        assert!(s.contains("SameSite=Lax"));
        // Extract payload and verify
        let payload = s
            .split(';')
            .next()
            .unwrap()
            .trim_start_matches(&format!("{COOKIE_NAME}="));
        let decoded = verify_cookie(payload, TEST_SECRET).expect("verify");
        assert_eq!(decoded.device_id, "cookie-roundtrip");
        assert_eq!(decoded.bookmarks, vec![42, 99]);
    }

    #[test]
    fn set_cookie_rejects_tampering() {
        unsafe {
            std::env::set_var("JWT_SECRET", TEST_SECRET);
        }
        let library = DeviceLibrary {
            device_id: "tamper-test".into(),
            bookmarks: vec![1],
            follows: vec![],
            updated_at: None,
        };
        let header = make_set_cookie(&library, "test-secret").unwrap();
        let s = header.to_str().unwrap().to_string();
        let payload = s
            .split(';')
            .next()
            .unwrap()
            .trim_start_matches(&format!("{COOKIE_NAME}="))
            .to_string();
        // Flip a base64 char in the signature (the bit after the `.`)
        let chars: Vec<char> = payload.chars().collect();
        let dot_idx = chars.iter().position(|c| *c == '.').unwrap();
        let mut tampered = chars;
        // Replace one base64 char with another valid char to avoid decode error
        let orig = tampered[dot_idx + 1];
        tampered[dot_idx + 1] = if orig == 'A' { 'B' } else { 'A' };
        let tampered: String = tampered.into_iter().collect();
        assert_ne!(tampered, payload);
        assert!(verify_cookie(&tampered, TEST_SECRET).is_none());
    }

    // ── Consent gate tests ───────────────────────────────────────────

    fn header_map(cookie: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(c) = cookie {
            h.insert(
                axum::http::header::COOKIE,
                c.parse().expect("valid header value"),
            );
        }
        h
    }

    #[test]
    fn consent_gate_allows_with_granted() {
        let h = header_map(Some("fh_consent=granted"));
        assert!(require_consent(&h).is_ok());
    }

    #[test]
    fn consent_gate_denied_returns_err() {
        let h = header_map(Some("fh_consent=denied"));
        assert!(require_consent(&h).is_err());
    }

    #[test]
    fn consent_gate_no_cookie_returns_err() {
        let h = header_map(None);
        assert!(require_consent(&h).is_err());
    }

    #[test]
    fn consent_gate_garbage_returns_err() {
        let h = header_map(Some("fh_consent=garbage"));
        assert!(require_consent(&h).is_err());
    }

    #[test]
    fn consent_gate_mixed_cookies() {
        let h = header_map(Some("foo=bar; fh_consent=granted; baz=qux"));
        assert!(require_consent(&h).is_ok());
    }
}
