use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Application-wide error type
#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message (HTTP 400)
    BadRequest(String),
    /// Unauthorized - authentication required (HTTP 401)
    Unauthorized(String),
    /// Forbidden - authenticated but not allowed (HTTP 403)
    Forbidden(String),
    /// Conflict - duplicate / already exists (HTTP 409)
    Conflict(String),
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Scraper error
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
    /// Rate limited - wait N seconds (HTTP 429)
    RateLimited(u64),
    /// Rate limited with a custom JSON body (e.g. a PoW challenge).
    /// The payload is served as-is with HTTP 429 so callers can hand the
    /// client the next challenge without inventing a new status code.
    RateLimitedJson(serde_json::Value),
    /// Upload too large (HTTP 413) — blocks oversized file uploads.
    PayloadTooLarge(String),
    /// Gone (HTTP 410) — retired endpoint with a migration pointer.
    Gone(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "BadRequest: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            AppError::ScrapeError(msg) => write!(f, "ScrapeError: {}", msg),
            AppError::ExportError(msg) => write!(f, "ExportError: {}", msg),
            AppError::Database(msg) => write!(f, "Database: {}", msg),
            AppError::CacheError(msg) => write!(f, "CacheError: {}", msg),
            AppError::RateLimited(retry_after) => {
                write!(f, "RateLimited: retry after {}s", retry_after)
            }
            AppError::RateLimitedJson(_) => write!(f, "RateLimitedJson"),
            AppError::PayloadTooLarge(msg) => write!(f, "PayloadTooLarge: {}", msg),
            AppError::Gone(msg) => write!(f, "Gone: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // ── the estate's envelope, and the shape these clients already parse ──────────────────
        //
        // Every exporter client on this machine reads `{"err": <int>, "msg": <string>}` — a numeric code
        // nobody can branch on without a table, and a field name that matches no other product in the
        // estate. The estate contract is `{"error": {"code", "message", "request_id"}}` (M72-04), so both
        // are emitted: `error` is the field to write new clients against, and the legacy pair stays until
        // the last caller moves (FICNEXUS_LEGACY_ERROR_ALIAS=off turns it off).
        let (status, code, message, legacy_err) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg, -1),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "AUTH_REQUIRED", msg, 401),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg, -403),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg, -409),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg, -5),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL",
                    "internal server error".to_owned(),
                    -1,
                )
            }
            AppError::ScrapeError(msg) => (StatusCode::BAD_GATEWAY, "UPSTREAM_FAILED", msg, -6),
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "EXPORT_FAILED",
                    "export failed".to_owned(),
                    -5,
                )
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "database error".to_owned(),
                    -1,
                )
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CACHE_ERROR",
                    "cache error".to_owned(),
                    -1,
                )
            }
            AppError::RateLimited(retry_after) => {
                let mut response = refusal(
                    StatusCode::TOO_MANY_REQUESTS,
                    "RATE_LIMITED",
                    "rate limited".to_owned(),
                    -429,
                );
                if let Ok(value) = axum::http::HeaderValue::from_str(&retry_after.to_string()) {
                    response.headers_mut().insert(
                        axum::http::header::RETRY_AFTER,
                        value,
                    );
                }
                return response;
            }
            // A PoW challenge is a payload, not a refusal: it carries the next challenge the client has to
            // answer. Reshaping it into an envelope would delete the only useful part, so it passes through
            // with the request id header and nothing else.
            AppError::RateLimitedJson(payload) => {
                let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(payload)).into_response();
                response
                    .headers_mut()
                    .insert("x-request-id", header_value(&request_id()));
                return response;
            }
            AppError::PayloadTooLarge(msg) => {
                (StatusCode::PAYLOAD_TOO_LARGE, "PAYLOAD_TOO_LARGE", msg, -413)
            }
            AppError::Gone(msg) => (StatusCode::GONE, "GONE", msg, -410),
        };

        refusal(status, code, message, legacy_err)
    }
}

/// A per-response identifier: the process and a monotonic counter, which is enough to join a client's
/// report to a log line and is not a secret. A request that never reached this process has no id, and
/// that is the honest answer for one.
fn request_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}

fn header_value(value: &str) -> axum::http::HeaderValue {
    axum::http::HeaderValue::from_str(value).unwrap_or(axum::http::HeaderValue::from_static("0"))
}

/// The legacy `err`/`msg` pair, on unless a caller has moved. Read once: an error path must not stat the
/// environment per refusal.
fn legacy_alias_enabled() -> bool {
    use std::sync::OnceLock;
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        !matches!(
            std::env::var("FICNEXUS_LEGACY_ERROR_ALIAS").as_deref(),
            Ok("off") | Ok("0") | Ok("false")
        )
    })
}

fn refusal(status: StatusCode, code: &str, message: String, legacy_err: i64) -> Response {
    let id = request_id();
    let mut body = json!({
        "error": { "code": code, "message": message.clone(), "request_id": id },
    });
    if legacy_alias_enabled() {
        // The same words in the shape these clients already parse. When the alias is off the legacy keys
        // are absent rather than null: a client branching on `err` must be able to tell the difference
        // between "not a refusal" and "the server stopped answering in this shape".
        body["err"] = json!(legacy_err);
        body["msg"] = json!(message);
    }
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert("x-request-id", header_value(&id));
    response
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Standard API result type
pub type AppResult<T> = Result<T, AppError>;
/// Shorter alias used across services
pub type Result<T, E = AppError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_bad_request() {
        let err = AppError::BadRequest("invalid input".to_string());
        let s = format!("{}", err);
        assert!(s.contains("BadRequest"));
        assert!(s.contains("invalid input"));
    }

    #[test]
    fn test_display_rate_limited() {
        let err = AppError::RateLimited(30);
        let s = format!("{}", err);
        assert!(s.contains("RateLimited"));
        assert!(s.contains("30"));
    }

    #[test]
    fn test_display_not_found() {
        let err = AppError::NotFound("story".into());
        let s = format!("{}", err);
        assert!(s.contains("NotFound"));
        assert!(s.contains("story"));
    }

    #[test]
    fn test_display_internal() {
        let err = AppError::Internal("oops".into());
        let s = format!("{}", err);
        assert!(s.contains("Internal"));
        assert!(s.contains("oops"));
    }

    #[test]
    fn test_display_scrape_error() {
        let err = AppError::ScrapeError("timeout".into());
        let s = format!("{}", err);
        assert!(s.contains("ScrapeError"));
        assert!(s.contains("timeout"));
    }

    #[test]
    fn test_display_export_error() {
        let err = AppError::ExportError("epub fail".into());
        let s = format!("{}", err);
        assert!(s.contains("ExportError"));
        assert!(s.contains("epub fail"));
    }

    #[test]
    fn test_display_database() {
        let err = AppError::Database("conn lost".into());
        let s = format!("{}", err);
        assert!(s.contains("Database"));
        assert!(s.contains("conn lost"));
    }

    #[test]
    fn test_display_cache_error() {
        let err = AppError::CacheError("cache miss".into());
        let s = format!("{}", err);
        assert!(s.contains("CacheError"));
        assert!(s.contains("cache miss"));
    }

    #[test]
    fn test_from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app: AppError = io.into();
        match app {
            AppError::Internal(msg) => assert!(msg.contains("file not found")),
            _ => panic!("expected Internal, got {:?}", app),
        }
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx = sqlx::Error::Protocol("bad query".into());
        let app: AppError = sqlx.into();
        match app {
            AppError::Database(msg) => assert!(msg.contains("bad query")),
            _ => panic!("expected Database, got {:?}", app),
        }
    }

    #[test]
    fn test_from_redis_error() {
        let err = redis::RedisError::from((redis::ErrorKind::Io, "connection refused"));
        let app: AppError = err.into();
        match app {
            AppError::CacheError(msg) => assert!(msg.contains("connection refused")),
            _ => panic!("expected CacheError, got {:?}", app),
        }
    }

    #[test]
    fn test_from_reqwest_error() {
        // Create a reqwest error by building a client that will produce an error
        // Use a builder that will fail to connect (no actual network call)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(10))
                .build()
                .unwrap()
                .get("http://127.0.0.1:1/")
                .send()
                .await
        });
        let app: AppError = match result {
            Err(e) => e.into(),
            Ok(_) => return, // skip if somehow succeeds on localhost
        };
        match app {
            AppError::ScrapeError(_) => {} // expected
            _ => panic!("expected ScrapeError, got {:?}", app),
        }
    }

    #[test]
    fn test_from_serde_json_error() {
        let json: Result<serde_json::Value, _> = serde_json::from_str("!invalid");
        let json_err = json.unwrap_err();
        let app: AppError = json_err.into();
        match app {
            AppError::Internal(_) => {} // expected
            _ => panic!("expected Internal, got {:?}", app),
        }
    }

    #[test]
    fn test_into_response_bad_request_status() {
        let err = AppError::BadRequest("bad".to_string());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_into_response_rate_limited_status() {
        let err = AppError::RateLimited(30);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_into_response_not_found_status() {
        let err = AppError::NotFound("gone".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_into_response_internal_status() {
        let err = AppError::Internal("oops".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_scrape_error_status() {
        let err = AppError::ScrapeError("timeout".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
    }

    #[test]
    fn test_into_response_export_error_status() {
        let err = AppError::ExportError("fail".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_database_error_status() {
        let err = AppError::Database("conn".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_into_response_cache_error_status() {
        let err = AppError::CacheError("miss".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

#[cfg(test)]
mod envelope_tests {
    use super::*;
    use axum::response::IntoResponse;

    async fn body_of(response: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("json")
    }

    #[tokio::test]
    async fn every_refusal_carries_code_message_and_request_id() {
        // Three arms with three different histories: a validation-shaped refusal, a missing resource, and a
        // masked internal fault. An envelope that only holds for the first is an envelope that holds for none.
        for (err, expected_code, expected_status) in [
            (AppError::BadRequest("missing title".into()), "BAD_REQUEST", 400),
            (AppError::NotFound("work 7 not found".into()), "NOT_FOUND", 404),
            (AppError::Internal("secret".into()), "INTERNAL", 500),
        ] {
            let response = err.into_response();
            assert_eq!(response.status().as_u16(), expected_status);
            let header = response
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_owned();
            assert!(!header.is_empty(), "no x-request-id header");

            let body = body_of(response).await;
            assert_eq!(body["error"]["code"], expected_code);
            assert!(body["error"]["message"].as_str().is_some_and(|m| !m.is_empty()));
            assert_eq!(body["error"]["request_id"], header);
            // The masked messages stay masked: an internal fault must not leak its cause to a client.
            if expected_status == 500 {
                assert_eq!(body["error"]["message"], "internal server error");
            }
            // And the legacy pair these clients parse is still there, by default.
            assert!(body["err"].is_number(), "legacy err missing: {body}");
            assert!(body["msg"].is_string(), "legacy msg missing: {body}");
        }
    }

    #[tokio::test]
    async fn request_ids_are_per_response() {
        let a = body_of(AppError::NotFound("x".into()).into_response()).await;
        let b = body_of(AppError::NotFound("x".into()).into_response()).await;
        assert_ne!(a["error"]["request_id"], b["error"]["request_id"]);
    }
}

/// Give a refusal the framework emits before a handler the estate's envelope (M72-04).
///
/// axum answers an unparseable body or a wrong content type from its own extractor rejections, in `text/plain`,
/// before any handler runs — so the handlers that render the envelope never see it. Measured on the sibling
/// products 2026-11-08 (ascend: `422 text/plain: missing field username`).
///
/// Only `/api` paths, only non-JSON answers, and the status is kept: the framework chose it, and this layer's job
/// is the body. A page or a file stream is not a refusal.
pub async fn envelope_refusals(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let is_api = request.uri().path().starts_with("/api/");
    let response = next.run(request).await;
    if !is_api || response.status().is_success() || response.status().is_redirection() {
        return response;
    }
    let already_json = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"));
    if already_json {
        return response;
    }

    let status = response.status();
    let code = match status.as_u16() {
        400 => "BAD_REQUEST",
        401 => "AUTH_REQUIRED",
        403 => "FORBIDDEN",
        404 => "NOT_FOUND",
        405 => "METHOD_NOT_ALLOWED",
        408 => "REQUEST_TIMEOUT",
        409 => "CONFLICT",
        413 => "PAYLOAD_TOO_LARGE",
        415 => "UNSUPPORTED_MEDIA_TYPE",
        422 => "UNPROCESSABLE_ENTITY",
        429 => "RATE_LIMITED",
        503 => "UNAVAILABLE",
        _ => "INTERNAL",
    };
    refusal(
        status,
        code,
        status.canonical_reason().unwrap_or("refused").to_owned(),
        -1,
    )
}

#[cfg(test)]
mod envelope_layer_tests {
    use super::*;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn a_text_plain_refusal_on_an_api_path_gets_the_envelope() {
        let app = Router::new()
            .route(
                "/api/v1/strict",
                get(|| async { (StatusCode::UNPROCESSABLE_ENTITY, "missing field `value`") }),
            )
            .layer(axum::middleware::from_fn(envelope_refusals));

        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/strict")
                    .body(axum::body::Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let content_type = resp
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        assert!(content_type.starts_with("application/json"), "got {content_type:?}");

        let bytes = axum::body::to_bytes(resp.into_body(), 64 * 1024).await.expect("body");
        let body: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(body["error"]["code"], "UNPROCESSABLE_ENTITY");
        assert!(body["error"]["request_id"].as_str().is_some_and(|id| !id.is_empty()));
    }

    #[tokio::test]
    async fn a_page_refusal_is_left_alone() {
        let app = Router::new()
            .route("/landing", get(|| async { (StatusCode::NOT_FOUND, "no such page") }))
            .layer(axum::middleware::from_fn(envelope_refusals));

        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/landing")
                    .body(axum::body::Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.expect("body");
        assert_eq!(&bytes[..], b"no such page");
    }
}
