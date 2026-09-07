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
        let (status, body) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, json!({"err": -1, "msg": msg})),
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, json!({"err": 401, "msg": msg}))
            }
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, json!({"err": -403, "msg": msg})),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, json!({"err": -409, "msg": msg})),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg})),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "internal server error"}),
                )
            }
            AppError::ScrapeError(msg) => (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg})),
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -5, "msg": "export failed"}),
                )
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "database error"}),
                )
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "cache error"}),
                )
            }
            AppError::RateLimited(retry_after) => (
                StatusCode::TOO_MANY_REQUESTS,
                json!({"err": -429, "msg": "rate limited", "retry_after": retry_after}),
            ),
            AppError::RateLimitedJson(payload) => (StatusCode::TOO_MANY_REQUESTS, payload),
            AppError::PayloadTooLarge(msg) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                json!({"err": -413, "msg": msg}),
            ),
            AppError::Gone(msg) => (StatusCode::GONE, json!({"err": -410, "msg": msg})),
        };

        (status, Json(body)).into_response()
    }
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
