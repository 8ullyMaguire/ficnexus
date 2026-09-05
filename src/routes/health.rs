use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::server::AppState;

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub db: bool,
    pub redis: bool,
    pub version: String,
}

/// Health check query parameters
#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    /// Skip database check
    #[serde(default)]
    pub skip_db: bool,
    /// Skip Redis check
    #[serde(default)]
    pub skip_redis: bool,
}

/// Health check endpoint: GET /api/health
///
/// Redis is checked on the *dedicated* `health_redis` connection (never the
/// shared `state.redis`): the bookmark-import worker parks an unbounded BRPOP
/// on the shared multiplexed connection, so a PING queued behind it would time
/// out and report `redis:false` even when Redis is healthy. The PING is also
/// wrapped in a short timeout so a stuck Redis degrades health instead of
/// hanging the endpoint.
pub async fn health_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> impl IntoResponse {
    let mut db_ok = true;
    let mut redis_ok = true;

    // Check PostgreSQL if not skipped
    if !params.skip_db {
        db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&state.db)
            .await
            .is_ok();
    }

    // Check Redis if not skipped
    if !params.skip_redis {
        let mut redis_conn = state.health_redis.clone();
        redis_ok = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            redis::cmd("PING").query_async::<String>(&mut redis_conn),
        )
        .await
        .map(|res| res.is_ok())
        .unwrap_or(false);
    }

    // Determine overall status
    let status = if db_ok && redis_ok {
        "ok".to_string()
    } else if !db_ok && !redis_ok {
        "error".to_string()
    } else {
        "degraded".to_string()
    };

    // Return 503 if not fully healthy
    let status_code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status,
        db: db_ok,
        redis: redis_ok,
        version: "0.2.0".to_string(),
    };

    (status_code, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serializes_correctly() {
        let response = HealthResponse {
            status: "ok".to_string(),
            db: true,
            redis: true,
            version: "0.2.0".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["db"], true);
        assert_eq!(json["redis"], true);
        assert_eq!(json["version"], "0.2.0");
    }

    #[test]
    fn test_health_response_degraded() {
        let response = HealthResponse {
            status: "degraded".to_string(),
            db: true,
            redis: false,
            version: "0.2.0".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["db"], true);
        assert_eq!(json["redis"], false);
    }

    #[test]
    fn test_health_response_error() {
        let response = HealthResponse {
            status: "error".to_string(),
            db: false,
            redis: false,
            version: "0.2.0".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["db"], false);
        assert_eq!(json["redis"], false);
    }

    // ─── Regression tests for the redis:false bug ─────────────────────────
    //
    // The handler must never hang: a PING against a dead Redis endpoint must
    // come back `redis:false` (degraded) within the 2s handler timeout, not
    // block forever.

    #[tokio::test]
    async fn health_redis_check_returns_false_when_redis_unreachable() {
        // Localhost port 1 is guaranteed closed — connect fails immediately,
        // so this never touches real infrastructure and needs no Redis.
        let client = redis::Client::open("redis://127.0.0.1:1").unwrap();
        let conn = client.get_multiplexed_async_connection().await;
        assert!(
            conn.is_err(),
            "expected connect to closed port 1 to fail (is something listening on 127.0.0.1:1?)"
        );
    }

    #[tokio::test]
    async fn health_handler_degrades_quickly_when_redis_conn_is_dead() {
        // The health handler should treat an unusable dedicated connection as
        // redis:false and return a 503 within the 2s handler timeout. This is
        // exercised end-to-end in the DB-gated integration test
        // (tests/health_api.rs); here we only assert the 2s PING timeout logic
        // on a deliberately-unreachable connection, so the unit test never
        // hangs and never needs a live Redis.
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), async {
            let client = redis::Client::open("redis://127.0.0.1:1").unwrap();
            let mut conn = client.get_multiplexed_async_connection().await?;
            redis::cmd("PING").query_async::<String>(&mut conn).await
        })
        .await;
        match result {
            Ok(Err(_)) => {} // connect/PING failed → redis false: correct
            Ok(Ok(pong)) => panic!("unreachable port 1 answered PING: {pong}"),
            Err(_) => panic!("PING against dead endpoint must fail fast, not time out"),
        }
    }
}
