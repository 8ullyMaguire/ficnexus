# Part 2 — Hello Route: `/health` and `/api/remote`

You booted the server in Part 1. Now you look at two real endpoints that already exist in the codebase: `/api/health` and `/api/remote`. You will understand how they work, then write your own endpoint beside them.

---

## 2.1 The health endpoint: `GET /api/health`

Open `src/routes/health.rs`:

```rust
// src/routes/health.rs (excerpt)
use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::server::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub db: bool,
    pub redis: bool,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    #[serde(default)]
    pub skip_db: bool,
    #[serde(default)]
    pub skip_redis: bool,
}

pub async fn health_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> impl IntoResponse {
    let mut db_ok = true;
    let mut redis_ok = true;

    if !params.skip_db {
        db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&state.db)
            .await
            .is_ok();
    }

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

    let status = if db_ok && redis_ok {
        "ok".to_string()
    } else if !db_ok && !redis_ok {
        "error".to_string()
    } else {
        "degraded".to_string()
    };

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
```

### What the endpoint does

1. **Extracts the state**: `State(state): State<Arc<AppState>>` — pulls the shared `AppState` from the request. Every handler that needs the database, Redis, or config does this.

2. **Extracts query parameters**: `Query(params): Query<HealthQuery>` — parses the query string into a `HealthQuery` struct. If the URL is `/api/health?skip_db=true&skip_redis=false`, then `params.skip_db = true` and `params.skip_redis = false`.

3. **Checks PostgreSQL**: Runs `SELECT 1` against the database. If it succeeds, `db_ok = true`. If it fails (database down, connection error), `db_ok = false`.

4. **Checks Redis**: Sends `PING` to Redis on the **dedicated** `health_redis` connection (not the shared `state.redis`). The PING is wrapped in a 2-second timeout — if Redis is stuck, the health check degrades instead of hanging forever.

5. **Determines status**: 
   - Both ok → `"ok"`, HTTP 200
   - One failed → `"degraded"`, HTTP 503
   - Both failed → `"error"`, HTTP 503

6. **Returns the response**: A tuple of `(status_code, Json(response))`. Axum uses the status code for the HTTP status and serializes the struct to JSON.

### Why two Redis connections?

The server has two Redis connections: `state.redis` (shared, used by rate limiter, bookmark import worker, etc.) and `state.health_redis` (dedicated to health checks).

The bookmark-import worker parks an unbounded `BRPOP` on the shared connection. `BRPOP` blocks until there's an item in the queue. If the health check used the same connection and sent `PING` while `BRPOP` was blocking, the `PING` would queue behind the `BRPOP` and time out. Health would report `redis:false` even though Redis is perfectly healthy.

The dedicated connection avoids this. Health checks always use `health_redis`, which is never blocked.

### The `HealthQuery` struct

```rust
#[derive(Debug, Deserialize)]
pub struct HealthQuery {
    #[serde(default)]
    pub skip_db: bool,
    #[serde(default)]
    pub skip_redis: bool,
}
```

- `#[derive(Deserialize)]` — serde can parse this from query parameters.
- `#[serde(default)]` — if the parameter is missing, default to `false`. Without this, a missing parameter would cause a deserialization error.

So `/api/health` (no params) checks both. `/api/health?skip_db=true` skips the database check. `/api/health?skip_db=true&skip_redis=true` skips both and always returns `"ok"`.

### Try it

Start the server (with a database, or skip the DB check):

```bash
curl -s http://localhost:8000/api/health?skip_db=true\&skip_redis=true | python3 -m json.tool
```

**Expected output**:

```json
{
    "status": "ok",
    "db": true,
    "redis": true,
    "version": "0.2.0"
}
```

(The `db` and `redis` fields are `true` because we skipped the checks, not because they're actually healthy.)

Without skipping:

```bash
curl -s http://localhost:8000/api/health | python3 -m json.tool
```

**Expected output** (if database is running):

```json
{
    "status": "ok",
    "db": true,
    "redis": true,
    "version": "0.2.0"
}
```

If the database is down:

```json
{
    "status": "degraded",
    "db": false,
    "redis": true,
    "version": "0.2.0"
}
```

With HTTP status 503.

### The tests in health.rs

The file ends with tests:

```rust
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
    // ... more tests ...
}
```

These are unit tests. They test the serialization of `HealthResponse` without needing a real database or Redis. Run them with:

```bash
cargo test health_handler
```

---

## 2.2 The remote endpoint: `GET /api/remote`

Open `src/server.rs` again and find the `remote_handler` function at the bottom:

```rust
// src/server.rs (lines 936-945)
async fn remote_handler(
    axum::extract::ConnectInfo(remote_addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

This endpoint does not use `AppState`. It only uses `ConnectInfo`, which is extracted by Axum because the router was built with `into_make_service_with_connect_info::<std::net::SocketAddr>()`.

### How `ConnectInfo` works

When the server is built with `into_make_service_with_connect_info`, Axum wraps each incoming connection with its remote address. Handlers can extract it with `ConnectInfo<T>`, where `T` is the address type (here, `std::net::SocketAddr`).

The handler returns the IP and port of the client. This is useful for:
- Debugging: "who is hitting my server?"
- Rate limiting: "which IP is making too many requests?"
- Security: "is this request coming from a known bad IP?"

### Try it

```bash
curl -s http://localhost:8000/api/remote | python3 -m json.tool
```

**Expected output**:

```json
{
    "ip": "127.0.0.1",
    "port": 12345,
    "is_automated": false
}
```

The IP will be `127.0.0.1` if you're running curl on the same machine. The port will be a random ephemeral port.

---

## 2.3 The API docs endpoint: `GET /api/`

Open `src/routes/api_docs.rs`:

```rust
// src/routes/api_docs.rs
use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.2.0",
        "endpoints": {
            "/api/epub": { "method": "GET", "params": { "q": "URL of the fanfiction" }, "description": "..." },
            "/api/meta": { "method": "GET", "params": { "q": "URL of the fanfiction" }, "description": "..." },
            "/api/remote": { "method": "GET", "description": "Get request source information" },
            "/api/search": { "method": "GET", "params": { "q": "search query", ... }, "description": "..." },
            "/api/auth/register": { "method": "POST", "params": { "username": "...", "password": "..." }, "description": "..." },
            "/api/auth/login": { "method": "POST", "params": { "username": "...", "password": "..." }, "description": "..." },
            "/api/auth/me": { "method": "GET", "auth": "Bearer token", "description": "..." },
            "/api/bookmarks": { "method": "GET/POST/DELETE", "auth": "Bearer token", "description": "..." },
            "/api/ratings": { "method": "GET/POST", "auth": "Bearer token", "description": "..." },
            "/api/works/{url_id}/comments": { "method": "GET/POST", "description": "..." },
            "/api/tags": { "method": "GET", "params": { "url_id": "fic URL ID" }, "description": "..." },
            "/api/recommendations": { "method": "GET", "params": { "url_id": "fic URL ID", "n": "number of recommendations" }, "description": "..." },
            "/cache/{type}/{url_id}": { "method": "GET", "description": "Download cached export file" },
            "/opds": { "method": "GET", "description": "OPDS catalog root" }
        }
    }))
}
```

This is a hand-written API docs endpoint. It returns a JSON object listing all the endpoints, their methods, parameters, and descriptions. It's a simple way to discover the API without reading the source code.

**Try it**:

```bash
curl -s http://localhost:8000/api/ | python3 -m json.tool
```

---

## 2.4 Writing your own handler: the pattern

Every Axum handler follows the same pattern:

```rust
use axum::extract::{State, Query, Path};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

async fn my_handler(
    State(state): State<Arc<AppState>>,   // shared state (optional)
    Path(id): Path<u64>,                   // path parameter (optional)
    Query(params): Query<MyParams>,        // query parameters (optional)
    Json(body): Json<MyRequest>,           // JSON body (optional, for POST/PUT)
) -> impl IntoResponse {                    // return type
    // ... do work ...
    (StatusCode::OK, Json(my_response))
}
```

### Extractors

Axum uses "extractors" to pull data from the request. The extractors are:

- `State(state)` — the shared `AppState`.
- `Path(id)` — a path parameter like `{id}` in `/api/works/{id}`.
- `Query(params)` — query string parameters like `?q=hello`.
- `Json(body)` — a JSON request body (for POST/PUT).
- `ConnectInfo(addr)` — the remote address (only if the router was built with `into_make_service_with_connect_info`).
- `axum::extract::Multipart` — multipart form data (for file uploads).

Extractors are processed in order. If one fails (e.g., the JSON body is invalid), Axum returns an error before calling the handler.

### Return types

A handler can return:

- `impl IntoResponse` — anything that implements `IntoResponse`. This includes:
  - `(StatusCode, Json(value))` — status code + JSON body.
  - `Json(value)` — defaults to 200 OK.
  - `&'static str` — plain text body.
  - `Redirect` — HTTP redirect.
  - `(StatusCode, &'static str)` — status code + plain text.
  - `Result<Json<Value>, AppError>` — success or error (Axum knows how to convert `AppError` into an HTTP response).

### The `AppError` type

Open `src/error.rs`:

```rust
// src/error.rs (excerpt)
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("scrape error: {0}")]
    ScrapeError(String),
    #[error("internal: {0}")]
    Internal(String),
    // ... more variants ...
}
```

This is a `thiserror` enum. Each variant has a `#[error("...")]` attribute that defines the error message. Handlers return `Result<T, AppError>`, and Axum converts `AppError` into an HTTP response.

---

## 2.5 Try It Yourself: add an echo endpoint

Add a `POST /api/echo` endpoint that echoes back the JSON body with a timestamp.

### Step 1: Add the route

In `src/server.rs`, find the API routes section and add:

```rust
.route("/api/echo", axum::routing::post(echo_handler))
```

### Step 2: Add the handler

At the bottom of `src/server.rs`, before the `#[cfg(test)]` section:

```rust
use axum::Json;
use serde_json::Value;

async fn echo_handler(body: Json<Value>) -> Json<Value> {
    let mut response = body.0;
    response["echoed_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    Json(response)
}
```

### Step 3: Rebuild and test

```bash
cargo build
cargo run
```

In another terminal:

```bash
curl -s -X POST http://localhost:8000/api/echo \
  -H "Content-Type: application/json" \
  -d '{"message": "hello"}' | python3 -m json.tool
```

**Expected output**:

```json
{
    "message": "hello",
    "echoed_at": "2026-08-26T10:00:00+02:00"
}
```

### What you learned

- How to register a POST route.
- How to extract a JSON body with `Json<Value>`.
- How to modify the JSON and return it.
- How to use `chrono::Utc::now()` for timestamps.

---

## 2.6 What you have now

- You understand the health endpoint: query params, DB check, Redis check, status codes.
- You understand why there are two Redis connections.
- You understand the remote endpoint: `ConnectInfo`, client IP.
- You understand the API docs endpoint: hand-written JSON docs.
- You understand the handler pattern: extractors, return types, `AppError`.
- You added an echo endpoint and verified it.

Next: Part 3 — Database Foundations. You will connect to PostgreSQL, read the migration files, and write your first query.

---

*End of Part 2. On to [Part 3 — Database Foundations](./03-database-foundations/03-database-foundations.md).*
