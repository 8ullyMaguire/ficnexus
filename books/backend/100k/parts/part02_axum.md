# Part 2: Building the Axum Backend

---

# Chapter 5: First Axum Server

## Hello, Axum!

Now that you understand the basics of Rust, it's time to build something real. In this chapter, we'll create our first Axum web server — a minimal HTTP server that responds to requests. By the end of this chapter, you'll understand how Axum works and how FicHub uses it.

Axum is a web framework built on top of Tokio (the async runtime) and Tower (a middleware framework). It's designed to be ergonomic, type-safe, and composable. Unlike some web frameworks that rely heavily on macros or runtime reflection, Axum uses Rust's type system to ensure correctness at compile time.

Let's create a new project:

```bash
cargo new fichub-demo
cd fichub-demo
cargo add axum tokio --features tokio/full
```

Now edit `src/main.rs`:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, FicHub!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
```

Run it:

```bash
cargo run
```

In another terminal:

```bash
curl http://localhost:3000/
# Hello, FicHub!
```

That's a working web server in 12 lines of code. Let's break down what's happening:

1. `Router::new()` creates a new router — the central routing table that maps URLs to handlers
2. `.route("/", get(...))` adds a route: when a GET request comes in for `/`, call the given handler
3. `TcpListener::bind("0.0.0.0:3000")` binds to all network interfaces on port 3000
4. `axum::serve(listener, app)` starts the server, handling requests as they arrive

## Handlers

A **handler** is an async function that takes some **extractors** and returns a **response**:

```rust
async fn hello_handler() -> &'static str {
    "Hello, FicHub!"
}
```

The return type determines the response. A `&'static str` becomes a plain text response. An `axum::Json<Value>` becomes a JSON response. Let's make a JSON endpoint:

```rust
use axum::Json;
use serde_json::json;

async fn api_info() -> Json<serde_json::Value> {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": "Fetch metadata and download links",
            "/api/v0/meta": "Fetch metadata only",
        }
    }))
}
```

Add this route to your router:

```rust
let app = Router::new()
    .route("/", get(|| async { "Hello, FicHub!" }))
    .route("/api/", get(api_info));
```

Now `curl http://localhost:3000/api/` returns a JSON response with all the endpoint information.

This is exactly what FicHub does in `src/routes/api_docs.rs`:

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links"
            },
            // ...
        }
    }))
}
```

## Extractors

**Extractors** are how Axum pulls data out of the incoming request. They're the parameters to your handler functions. Axum figures out what to extract based on the type:

```rust
use axum::extract::{Path, Query, State};
use std::sync::Arc;

async fn handler(
    State(state): State<Arc<AppState>>,     // Shared application state
    Path(url_id): Path<String>,              // URL path parameter
    Query(params): Query<MyParams>,          // Query string parameters
) -> Json<serde_json::Value> {
    Json(json!({ "url_id": url_id }))
}
```

Let's look at each extractor type:

### Path Extractor

The `Path` extractor captures parts of the URL path:

```rust
.route("/cache/{etype}/{url_id}", get(download_handler))

async fn download_handler(
    Path((etype, url_id)): Path<(String, String)>,
) -> String {
    format!("Download {} in format {}", url_id, etype)
}
```

If the URL is `/cache/epub/abc123`, then `etype` is `"epub"` and `url_id` is `"abc123"`.

### Query Extractor

The `Query` extractor captures query string parameters:

```rust
.route("/api/v0/epub", get(epub_handler))

#[derive(Deserialize)]
struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

async fn epub_handler(
    Query(params): Query<ExportQuery>,
) -> Json<serde_json::Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({ "q": query }))
}
```

If the URL is `/api/v0/epub?q=https://ao3.org/works/123`, then `params.q` is `Some("https://ao3.org/works/123")`.

### State Extractor

The `State` extractor provides shared application state — the database pool, Redis connection, scraper registry, and everything else that needs to be shared across handlers:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    // ...
}

async fn handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    // Access the database
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db)
        .await
        .unwrap();
    
    Json(json!({ "fic_count": count.0 }))
}
```

We wrap `AppState` in `Arc` (Atomic Reference Counting) because it's shared across all handlers. `Arc` lets multiple tasks hold a reference to the same data without taking ownership.

### ConnectInfo Extractor

The `ConnectInfo` extractor gives you the client's IP address and port:

```rust
use axum::extract::ConnectInfo;
use std::net::SocketAddr;

async fn handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> String {
    format!("Your IP is {}", remote.ip())
}
```

This requires the server to be started with `into_make_service_with_connect_info`:

```rust
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
).await.unwrap();
```

FicHub uses this to track client IPs for rate limiting and tag voting.

## Building the Router

The router is the central mapping from URLs to handlers. You build it by chaining `.route()` calls:

```rust
let app = Router::new()
    .route("/", get(root_handler))
    .route("/api/", get(api_docs_handler))
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .route("/api/v0/search", get(search_handler))
    .route("/cache/{etype}/{url_id}", get(download_handler));
```

You can also use `nest` to group routes under a common prefix:

```rust
let app = Router::new()
    .nest("/api/v0", api_routes)
    .nest("/opds", opds_routes);
```

In FicHub, the router is built in `server.rs` and includes about 40 routes. The full route tree looks like this:

```rust
Router::new()
    // API routes
    .route("/api/", get(routes::api_docs::api_docs_handler))
    .route("/api/v0/epub", get(routes::export::epub_handler))
    .route("/api/v0/meta", get(routes::meta::meta_handler))
    .route("/api/v0/remote", get(remote_handler))

    // Cache download routes
    .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
    .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))

    // Recommender routes
    .route("/api/v0/recommendations", get(recommendations_handler))
    .route("/api/v0/recommendations/suggest", post(suggest_handler))
    .route("/api/v0/recommendations/vote", post(vote_handler))
    .route("/api/v0/recommendations/votes", get(votes_handler))

    // Tag routes
    .route("/api/v0/tags/submit", post(submit_tag))
    .route("/api/v0/tags/vote", post(vote_tag))
    .route("/api/v0/tags/flag", post(flag_tag))
    .route("/api/v0/tags", get(get_tags))

    // Curator routes
    .route("/api/v0/curator/alias", post(create_alias))
    .route("/api/v0/curator/merge", post(merge_tags))
    .route("/api/v0/curator/tags/{id}", delete(delete_tag))
    .route("/api/v0/curator/flags", get(list_flags))
    .route("/api/v0/curator/flags/{id}/resolve", post(resolve_flag))

    // Search
    .route("/api/v0/search", get(search_handler))

    // OPDS catalog routes
    .route("/opds", get(root_catalog))
    .route("/opds/new", get(recent_feed))
    .route("/opds/popular", get(popular_feed))
    .route("/opds/tags", get(tag_types))
    .route("/opds/tags/{type_id}", get(tags_by_type))
    .route("/opds/tags/{type_id}/{tag_name}", get(fics_by_tag))
    .route("/opds/authors", get(author_list))
    .route("/opds/recommendations/popular", get(popular_recommendations))
    .route("/opds/recommendations", get(fic_recommendations))
    .route("/opds/search", get(search_feed))
    .route("/opds/shelves", get(shelf_list))
    .route("/opds/shelf/{shelf_id}", get(shelf_contents))

    // Legacy redirects
    .route("/legacy/epub_export", get(redirect_to_root))
    .route("/fic/{url_id}", get(redirect_to_root))

    // Static frontend (SPA fallback)
    .fallback_service(
        ServeDir::new(&frontend_dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(frontend_dir.join("index.html"))),
    )

    // Middleware
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())

    // Shared state
    .with_state(state)
```

Notice the three different HTTP methods: `get()` for read-only endpoints, `post()` for creating/updating data, and `delete()` for removing data. The tags and curator routes use POST because they modify state. The search and OPDS routes use GET because they only read.

## Method Routing

Axum provides routing functions for each HTTP method:

```rust
use axum::routing::{get, post, put, delete};

Router::new()
    .route("/items", get(list_items).post(create_item))
    .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
```

You can chain multiple methods on the same route. In FicHub, most routes are GET (read-only), with POST for creating suggestions, voting, and submitting tags. The only DELETE route is for curator tag deletion.

## Middleware

**Middleware** is code that runs before or after your handler. FicHub uses two middleware layers:

**TraceLayer** logs every request:

```rust
.layer(TraceLayer::new_for_http())
```

This produces log lines like:

```
2024-01-15T10:30:00Z INFO request{method=GET uri=/api/v0/epub}: fichub::routes: 200 OK
```

**CorsLayer** handles Cross-Origin Resource Sharing:

```rust
.layer(CorsLayer::permissive())
```

This allows any origin to make requests to the API. In production, you'd restrict this to your frontend's domain, but `permissive()` is fine for development.

## Returning Different Response Types

Handlers can return different types depending on the situation:

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    // Do some work...
    let result = do_work(query).await?;
    
    Ok(Json(json!({"err": 0, "data": result})))
}
```

Notice the `Result<Json<Value>, AppError>` return type. If the handler returns `Ok(json)`, the client gets a JSON response. If it returns `Err(error)`, Axum calls `error.into_response()` to produce the appropriate HTTP error response.

This is FicHub's pattern — every handler returns `Result<Json<Value>, AppError>`, and the `AppError` type knows how to become an HTTP response.

## Try It Yourself

Let's build a complete mini-server that mimics a few FicHub endpoints:

```rust
use axum::{extract::{Query, State}, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

struct AppState {
    fic_count: i64,
}

#[derive(Deserialize)]
struct MetaQuery {
    q: Option<String>,
}

async fn api_docs() -> Json<Value> {
    Json(json!({
        "name": "FicHub Demo",
        "endpoints": ["/api/v0/meta", "/api/v0/epub"]
    }))
}

async fn meta_handler(
    Query(params): Query<MetaQuery>,
) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({
        "err": 0,
        "q": query,
        "meta": { "title": "Demo Fic", "author": "Demo Author" }
    }))
}

async fn epub_handler(
    Query(params): Query<MetaQuery>,
) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({
        "err": 0,
        "q": query,
        "epub_url": format!("/cache/epub/abc123"),
        "html_url": format!("/cache/html/abc123")
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState { fic_count: 42 });

    let app = Router::new()
        .route("/api/", get(api_docs))
        .route("/api/v0/meta", get(meta_handler))
        .route("/api/v0/epub", get(epub_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
```

Try these commands:

```bash
curl http://localhost:3000/api/
curl "http://localhost:3000/api/v0/meta?q=https://ao3.org/works/123"
curl "http://localhost:3000/api/v0/epub?q=https://ao3.org/works/123"
```

This is essentially a simplified version of what FicHub does. The real thing has more error handling, database queries, and scraping — but the structure is the same.

## Watch Out!

**Don't forget `with_state`!** If you use `State` in your handlers but forget to call `.with_state(state)` on the router, you'll get a confusing compile error.

**Async handlers must be `Send`!** Axum requires all handlers to be `Send`, which means they can't hold non-Send types across `.await` points. This is rarely an issue, but if you see errors about `Send` bounds, that's why.

**JSON responses require `serde_json::Value`!** You can't return raw strings as JSON — you need to wrap them in `Json(json!({...}))`. The `json!` macro creates a `serde_json::Value` from a JSON-like syntax.

## Summary

In this chapter, you learned:
- How to create a minimal Axum server
- What handlers are and how they work
- How extractors pull data from requests (Path, Query, State, ConnectInfo)
- How to build a router with multiple routes
- How middleware adds logging and CORS
- How handlers return JSON responses or errors
- How FicHub's route tree is organized

Next, we'll look at how FicHub handles configuration and errors in detail.

---

# Chapter 6: Config and Errors

## Configuration: One Place, Many Settings

Every web application needs configuration — database URLs, port numbers, feature flags, rate limits. The question is: where should this configuration come from?

FicHub follows the **twelve-factor app** methodology: configuration comes from environment variables. This is the standard for containerized and cloud-native applications. It means you can run the same binary in development, staging, and production — just change the environment variables.

The `Config` struct in `src/config.rs` holds all of FicHub's configuration:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // Infrastructure
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub app_port: u16,
    pub tmp_dir: PathBuf,
    pub frontend_dir: PathBuf,

    // Feature flags
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,

    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,

    // Tagging v3 settings
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,

    // OPDS
    pub opds_shelf_token: String,

    // Other
    pub calibre_container: String,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
}
```

That's a lot of settings! Each one serves a specific purpose. Let's look at the most important groups:

### Infrastructure Settings

These are required — the server can't start without them:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let redis_url = std::env::var("REDIS_URL")
    .expect("REDIS_URL must be set");
```

The `.expect()` call will panic (crash the server) if the variable isn't set. This is intentional — if you forget to set the database URL, you want to know immediately, not discover it 5 minutes later when a request fails.

### Optional Settings with Defaults

Most settings have sensible defaults:

```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());

let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

The `.unwrap_or_else()` pattern is more user-friendly — if the variable isn't set, use the default. The `.parse()` converts the string to the appropriate type (u16 for port).

### The from_env Method

The entire configuration is loaded in one place:

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        // ... all other settings ...
        
        Config {
            database_url,
            redis_url,
            // ...
        }
    }
}
```

This is called once at startup, in `main.rs`:

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    server::run(config).await;
}
```

The `.dotenv().ok()` call loads a `.env` file if it exists (and silently does nothing if it doesn't). The tracing subscriber is configured from the `RUST_LOG` environment variable.

### The Clone Derive

Notice that `Config` derives `Clone`:

```rust
#[derive(Debug, Clone)]
pub struct Config { ... }
```

This lets us clone the config and pass it to different parts of the application. Since `Config` contains only simple types (strings, numbers, booleans, PathBufs), cloning is cheap.

## Error Handling: The FicHub Way

Error handling is one of the most important aspects of a web application. FicHub has a clean, centralized error type that handles all error scenarios:

```rust
#[derive(Debug)]
pub enum AppError {
    BadRequest(i32, String),
    RateLimited(u64),
    NotFound(String),
    Internal(String),
    ScrapeError(String),
    ExportError(String),
    Database(String),
    CacheError(String),
}
```

Each variant represents a different class of error:

- **BadRequest** — The client sent invalid input (bad URL, missing parameters)
- **RateLimited** — Too many requests from this IP
- **NotFound** — The requested resource doesn't exist
- **Internal** — Something went wrong on the server (but not related to any specific subsystem)
- **ScrapeError** — The scraper failed (site is down, blocked, or HTML changed)
- **ExportError** — EPUB/HTML generation failed
- **Database** — A database query failed
- **CacheError** — A Redis operation failed

### Display Implementation

Every error needs a human-readable representation:

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(code, msg) => write!(f, "BadRequest({}): {}", code, msg),
            AppError::RateLimited(retry_after) => write!(f, "RateLimited: retry after {}s", retry_after),
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            AppError::ScrapeError(msg) => write!(f, "ScrapeError: {}", msg),
            AppError::ExportError(msg) => write!(f, "ExportError: {}", msg),
            AppError::Database(msg) => write!(f, "Database: {}", msg),
            AppError::CacheError(msg) => write!(f, "CacheError: {}", msg),
        }
    }
}
```

This is used for logging — when an error occurs, it's logged with its Display representation.

### IntoResponse Implementation

The key feature of `AppError` is that it implements `IntoResponse` — it knows how to become an HTTP response:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS, json!({
                    "err": -429,
                    "msg": "rate limited",
                    "retry_after": retry_after
                }))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "internal server error"
                }))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({
                    "err": -6,
                    "msg": msg
                }))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -5,
                    "msg": "export failed"
                }))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "database error"
                }))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "cache error"
                }))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

A few design choices here:

1. **BadRequest** returns the error code to the client, so the frontend can handle specific error types
2. **RateLimited** includes `retry_after` so the client knows when to retry
3. **ScrapeError** returns `502 Bad Gateway` because the error originated from an upstream site
4. **Internal, Database, CacheError, ExportError** all return `500 Internal Server Error` and log the error — the client gets a generic message, but the server logs the details
5. **NotFound** returns `404` with a descriptive message

### From Implementations

The `?` operator works because `AppError` implements `From` for various error types:

```rust
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
```

This means you can use `?` on any of these error types and they'll automatically be converted to the appropriate `AppError` variant:

```rust
// sqlx::Error → AppError::Database
let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

// redis::RedisError → AppError::CacheError
let count: Option<u32> = redis.get(key).await?;

// reqwest::Error → AppError::ScrapeError
let response = client.get(url).send().await?;
```

This is incredibly ergonomic — you never have to manually convert errors. The `?` operator does it for you.

### The AppResult Type

FicHub defines a convenience alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

This means handlers can return `AppResult<Json<Value>>` instead of the longer `Result<Json<Value>, AppError>`. Most query functions also return `AppResult<T>`:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
```

### Custom Error Codes

The `BadRequest` variant uses custom numeric error codes:

- `-1` — Generic bad request
- `-5` — Resource not found (used in JSON body, separate from HTTP 404)
- `-6` — Scraper error
- `-7` — Blacklisted
- `-10` — Automated request blocked
- `-429` — Rate limited

These codes are used by the frontend to determine how to display errors. They're a legacy convention from the original FicHub.

## Watch Out!

**Don't log sensitive information in errors!** The `Internal` variant logs the error message, but it should never contain passwords, tokens, or other secrets. Notice that the JSON response to the client says `"internal server error"` — it doesn't expose the actual error details.

**Be careful with `unwrap()`!** In production code, prefer `?` or `unwrap_or_else()` over `unwrap()`. A panic crashes the server and disconnects all connected clients. The only safe place for `unwrap()` is in initialization code (like config loading) where a panic is acceptable because the server hasn't started yet.

**Error conversion is lossy!** When you convert `sqlx::Error` to `AppError::Database`, you lose the original error type. The error message is preserved, but the structured information is gone. This is fine for logging and HTTP responses, but if you need to handle specific SQL errors differently, match on the `sqlx::Error` before converting.

## Summary

In this chapter, we explored:
- How FicHub loads configuration from environment variables
- The `Config` struct and its 40+ settings
- How the `AppError` enum centralizes all error types
- How `IntoResponse` converts errors to HTTP responses
- How `From` implementations make `?` error propagation work
- The `AppResult<T>` type alias for cleaner function signatures
- Custom error codes for the frontend

Next, we'll dive into PostgreSQL and SQLx — how FicHub connects to the database and runs queries.

---

# Chapter 7: PostgreSQL

## Your Data's Best Friend

Every web application needs somewhere to store data that persists across restarts. FicHub uses PostgreSQL — the world's most advanced open-source relational database. PostgreSQL is reliable, feature-rich, and handles everything from simple key-value lookups to complex full-text search queries.

In this chapter, we'll look at how FicHub connects to PostgreSQL, creates a connection pool, and runs queries using SQLx.

## What is SQLx?

SQLx is a Rust library for interacting with PostgreSQL (and other databases). What makes SQLx special is that it can **check your SQL queries at compile time**. If you have a typo in a column name or a type mismatch, the Rust compiler catches it before your code ever runs.

```rust
// This won't compile if the 'title' column doesn't exist in fic_info!
let title: (String,) = sqlx::query_as("SELECT title FROM fic_info WHERE id = $1")
    .bind("abc123")
    .fetch_one(&pool)
    .await?;
```

This is incredibly valuable for catching bugs early. You don't have to run the application and make a request to discover that your SQL is wrong — the compiler tells you immediately.

**Watch Out!** For compile-time SQL checking to work, you need a running database at compile time (or an offline query cache). If you're building in CI/CD or in a Docker container without a database, you'll need to use `sqlx prepare` to generate the cache. For development, just make sure your `DATABASE_URL` environment variable is set.

## Connection Pools

A **connection pool** is a cache of database connections that are reused across requests. Creating a new database connection for every request is expensive (TCP handshake, authentication, etc.), so connection pools maintain a set of open connections and hand them out as needed.

FicHub creates its connection pool in `src/db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Run migrations
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("migrations")
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
        }
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    };

    if migrations_path.exists() {
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;
        tracing::info!("Database migrations applied");
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }

    Ok(pool)
}
```

Let's break this down:

**`PgPoolOptions::new()`** creates a builder for the connection pool.

**`.max_connections(20)`** sets the maximum number of simultaneous connections. 20 is a reasonable default for a web server — enough to handle concurrent requests without overwhelming PostgreSQL.

**`.acquire_timeout(Duration::from_secs(10))`** sets how long to wait for a connection from the pool. If all 20 connections are busy and a new request needs one, it waits up to 10 seconds before failing.

**`.connect(database_url).await?`** actually connects to PostgreSQL and creates the pool.

The migrations section is interesting — it looks for the `migrations/` directory relative to the binary's location. This is because Rust binaries can be moved around, and we want the migrations to follow the binary.

### The Pool Type

`PgPool` is the type that represents a connection pool. It's cheaply clonable (it's an `Arc` internally) and can be shared across async tasks:

```rust
// Clone the pool — this doesn't create a new connection
let pool_clone = pool.clone();

// Use it in different tasks
tokio::spawn(async move {
    let result = sqlx::query("SELECT 1").fetch_one(&pool_clone).await;
});
```

In FicHub, the pool is stored in `AppState` and shared across all handlers:

```rust
pub struct AppState {
    pub db: sqlx::PgPool,
    // ...
}

// In server.rs
let db_pool = db::init_pool(&config.database_url)
    .await
    .expect("Failed to connect to database");

let state = Arc::new(AppState {
    db: db_pool,
    // ...
});
```

## SQL Basics

If you're new to SQL, here's a quick primer on the operations FicHub uses most:

### SELECT — Reading Data

```sql
-- Simple select
SELECT * FROM fic_info WHERE id = 'abc123';

-- Select specific columns
SELECT title, author, words FROM fic_info WHERE status = 'complete';

-- Aggregate functions
SELECT COUNT(*) FROM fic_info;
SELECT SUM(words) FROM fic_info WHERE author = 'SomeAuthor';

-- ORDER BY and LIMIT
SELECT * FROM fic_info ORDER BY words DESC LIMIT 10;

-- JOINs
SELECT fi.title, t.name 
FROM fic_info fi
JOIN fic_tags ft ON ft.url_id = fi.id
JOIN tags t ON t.id = ft.tag_id
WHERE fi.id = 'abc123';

-- Full-text search
SELECT * FROM fic_info 
WHERE text_search @@ plainto_tsquery('english', 'harry potter');
```

### INSERT — Creating Data

```sql
-- Simple insert
INSERT INTO fic_info (id, title, author, words, chapters, status, source, description, fic_created, fic_updated)
VALUES ('abc123', 'My Story', 'Author', 50000, 10, 'complete', 'https://...', 'A story...', NOW(), NOW());

-- Insert with conflict handling (upsert)
INSERT INTO fic_info (id, title, author, words)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    words = EXCLUDED.words;
```

The `ON CONFLICT DO UPDATE` clause is what makes this an "upsert" — if a row with the same `id` already exists, it updates instead of failing. This is crucial for FicHub's caching strategy.

### UPDATE — Modifying Data

```sql
-- Update specific columns
UPDATE fic_info SET words = 51000, chapters = 11 WHERE id = 'abc123';

-- Conditional update
UPDATE fic_tag_votes SET value = 1 WHERE url_id = $1 AND tag_id = $2 AND voter_ip = $3;
```

### DELETE — Removing Data

```sql
-- Delete specific rows
DELETE FROM fic_blacklist WHERE url_id = 'abc123';

-- Delete with cascade (handled by foreign keys)
DELETE FROM tags WHERE id = 1;  -- Also deletes fic_tags, tag_votes, etc.
```

## Querying with SQLx

SQLx provides several ways to run queries:

### `query` — For queries without structured return types

```rust
sqlx::query("INSERT INTO fic_info (id, title) VALUES ($1, $2)")
    .bind("abc123")
    .bind("My Story")
    .execute(&pool)
    .await?;
```

`.execute()` runs the query and returns a `QueryResult` with `rows_affected()`.

### `query_as` — For queries that return structured data

```rust
#[derive(Debug, FromRow)]
struct FicInfo {
    id: String,
    title: String,
    author: String,
    words: i64,
}

let fic: FicInfo = sqlx::query_as("SELECT id, title, author, words FROM fic_info WHERE id = $1")
    .bind("abc123")
    .fetch_one(&pool)
    .await?;
```

The `#[derive(FromRow)]` macro automatically implements the trait that converts database rows to your struct.

### Fetch Methods

- `.fetch_one(&pool)` — Returns exactly one row (errors if zero or multiple)
- `.fetch_optional(&pool)` — Returns `Option<Row>` (None if no rows)
- `.fetch_all(&pool)` — Returns `Vec<Row>` (all matching rows)

### Bind Parameters

SQLx uses PostgreSQL's parameterized queries to prevent SQL injection:

```rust
// SAFE — parameterized query
sqlx::query("SELECT * FROM fic_info WHERE title = $1")
    .bind(user_input)
    .fetch_all(&pool)
    .await?;

// DANGEROUS — string interpolation (NEVER do this!)
let query = format!("SELECT * FROM fic_info WHERE title = '{}'", user_input);
```

The `$1`, `$2`, etc. are parameter placeholders. SQLx sends the parameters separately from the SQL query, so PostgreSQL treats them as data, not code. Even if `user_input` contains SQL metacharacters, they're harmless.

## FicHub's Database Models

FicHub defines its data models in `src/db/models.rs`. The main model is `FicInfo`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

The `Option<T>` types represent nullable columns. A fic might not have an `author_url` or `content_hash`, so those are `Option<String>`.

Other models include:
- `RequestSource` — tracks where requests come from
- `RequestLog` — records every request
- `ExportLog` — tracks generated exports
- `FicBlacklist` — fics that shouldn't be exported
- `AuthorBlacklist` — authors whose fics shouldn't be exported
- `FicVersionBump` — manual cache invalidation

## Watch Out!

**Connection pool sizing matters!** Too few connections and requests queue up waiting. Too many and PostgreSQL runs out of memory or context-switches excessively. 20 is a good starting point for a single-server deployment. For high-traffic deployments, monitor `pg_stat_activity` and adjust accordingly.

**Always use parameterized queries!** Never interpolate user input into SQL strings. SQLx's bind parameters are your defense against SQL injection.

**Migrations run automatically on startup.** This is convenient for development but can be dangerous in production. If a migration fails, the server won't start. Consider running migrations separately in production (with backups!).

**Close connections gracefully.** SQLx's pool handles this automatically, but be aware that long-running queries can hold connections and cause pool exhaustion.

## Summary

In this chapter, you learned:
- What SQLx is and why it's great for Rust + PostgreSQL
- How to create a connection pool with `PgPoolOptions`
- SQL basics: SELECT, INSERT, UPDATE, DELETE
- How SQLx's query methods work (`query`, `query_as`, `fetch_one`, etc.)
- Why parameterized queries prevent SQL injection
- FicHub's database models and their column types
- How migrations run automatically on startup

Next, we'll look at the migration files in detail — the SQL that defines FicHub's database schema.

---

# Chapter 8: Migrations

## The Database Blueprint

Before FicHub can store any data, it needs tables. Tables are defined by the database schema — a blueprint that says "this table has these columns, with these types and constraints." FicHub defines its schema in SQL migration files.

Migrations are versioned SQL files that evolve the database schema over time. Each migration is a set of SQL statements that transform the database from one state to the next. They're designed to be idempotent (safe to run multiple times) and irreversible in practice.

FicHub has four migration files:

```
migrations/
├── 001_initial_schema.sql      # Core tables
├── 002_recommender.sql         # Recommendation engine
├── 003_tagging.sql             # Tag system v3
└── 004_shelves.sql             # OPDS shelves
```

Let's examine each one.

## Migration 001: Initial Schema

This is the foundation — the core tables that everything else builds on.

### request_source

```sql
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);
```

This table tracks where requests come from. Each unique combination of `is_automated`, `route`, and `description` gets its own row. The `UNIQUE` constraint ensures we don't create duplicate entries.

- `id` — auto-incrementing primary key (BIGSERIAL is PostgreSQL's 64-bit auto-increment)
- `created` — when the record was created
- `is_automated` — whether the request came from a bot
- `route` — which API route was hit (e.g., "/api/v0/epub")
- `description` — human-readable description (e.g., "web request")

### request_log

```sql
CREATE TABLE IF NOT EXISTS request_log (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    source_id BIGINT REFERENCES request_source(id),
    etype TEXT NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4 NOT NULL,
    url_id TEXT,
    fic_info TEXT,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);
```

Every API request is logged here. This table is useful for analytics, debugging, and monitoring:

- `source_id` — links to `request_source`
- `etype` — the export type ("epub", "html", "mobi", "pdf")
- `query` — the original URL the user submitted
- `info_request_ms` — how long the metadata lookup took (in milliseconds)
- `url_id` — the FicHub-internal ID for this fic
- `fic_info` — the scraped metadata (stored as JSON text)
- `export_ms` — how long the export generation took
- `export_file_name` — the filename of the generated file
- `export_file_hash` — the MD5 hash of the generated file

The indexes on this table are carefully chosen:

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);

CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

The first index supports lookups by `url_id` and `etype` (used for cache checking). The second is a **partial index** — it only indexes rows where an EPUB was actually exported. This is more efficient than indexing all rows.

### fic_info

```sql
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id INT8,
    author_id INT8,
    content_hash VARCHAR(256)
);
```

This is the most important table — it stores metadata for every fic FicHub has ever seen. The `id` is a 12-character hex string generated by SHA-256 hashing the source site's story ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 6 bytes = 12 hex chars
}
```

This ensures the same story always gets the same ID, regardless of when it was first seen.

### export_log

```sql
CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    version INT NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, version, etype, input_hash)
);
```

This table is the heart of the caching system. For each fic, export type, and input hash, it records the output hash. When a request comes in, FicHub checks this table to see if the export already exists.

The `input_hash` is typically the `content_hash` from the scraper (which changes when the story is updated). The `export_hash` is the MD5 of the generated file.

### fic_blacklist and author_blacklist

```sql
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);
```

These tables allow blocking specific fics or authors from being exported. The `reason` column indicates why:
- `5` — DMCA or legal takedown
- `6` — Greylisted (metadata shown, no download)
- `7` — Content policy violation
- `8` — Other

### fic_version_bump

```sql
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT
);
```

This table allows manually invalidating cached exports for a specific fic. If the scraping format changes, you can bump the version to force regeneration.

## Migration 002: Recommender System

This migration adds the tables needed for collaborative filtering recommendations.

### fic_works

```sql
CREATE TABLE IF NOT EXISTS fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

This table tracks site-specific metadata for each fic, particularly the number of users who have favourited/bookmarked it. The `favouriter_count` is crucial for the recommendation engine — works with more favouriters provide better collaborative filtering signals.

### fic_bookmarks

```sql
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

This records which users have bookmarked which fics. Users are identified by a SHA-256 hash of their profile URL — this preserves anonymity while allowing us to track their preferences.

### fic_bookmark_cooccur

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

This is the co-occurrence matrix — it tracks how many users have bookmarked both `work_a` and `work_b`. The `CHECK (work_a < work_b)` constraint ensures we only store each pair once (avoiding duplicates like A-B and B-A).

The co-occurrence count is the foundation of the recommendation engine. Two works with a high co-occurrence count are frequently bookmarked together, which means they're likely similar.

### recommendation_suggestions and recommendation_votes

```sql
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);

CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

These tables support community-submitted recommendations. Users can suggest that fic A is similar to fic B, and other users can upvote or downvote that suggestion.

### precomputed_recommendations

```sql
CREATE TABLE IF NOT EXISTS precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

This is a materialized cache of computed recommendations. When the recommendation engine runs, it stores its results here so that subsequent requests can be served instantly without recomputing.

## Migration 003: Tagging System v3

This is the most complex migration, adding the tag system with community voting.

### tag_types

```sql
CREATE TABLE IF NOT EXISTS tag_types (
    id SMALLINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'fandom'),
    (2, 'character'),
    (3, 'relationship'),
    (4, 'freeform'),
    (5, 'warning'),
    (6, 'category'),
    (7, 'other')
ON CONFLICT (id) DO NOTHING;
```

Tag types are a fixed enum — there are exactly 7 types, and they're inserted at migration time. The `ON CONFLICT DO NOTHING` makes this idempotent.

### tags

```sql
CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` is important — it makes tag name comparisons case-sensitive. This means "Harry Potter" and "harry potter" are different tags. Case-sensitive matching is the convention in fanfiction tagging.

### tag_aliases

```sql
CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT NOT NULL UNIQUE COLLATE "C",
    canonical_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

Aliases let curators redirect variant spellings to a canonical tag. For example, "HP" could be an alias for "Harry Potter". When a user submits a tag that matches an alias, it's automatically resolved to the canonical tag.

### fic_tags

```sql
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

This is the junction table connecting fics to tags. The `score` column tracks the net votes (upvotes minus downvotes). Tags with scores below the hidden threshold are not shown to users.

### The Vote Trigger

This is the most sophisticated part of the migration — a PostgreSQL trigger that automatically updates `fic_tags.score` when votes are inserted, updated, or deleted:

```sql
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value
        WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

This trigger runs automatically whenever a row in `fic_tag_votes` is inserted, updated, or deleted. It adjusts the `score` on `fic_tags` to reflect the new vote total. This means the application code never needs to manually recalculate scores — the database handles it.

### Full-Text Search

The migration also adds a generated column for full-text search:

```sql
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX IF NOT EXISTS idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

The `text_search` column is automatically generated from the title (weight A, more important) and description (weight B, less important). The GIN index makes full-text search queries very fast.

## Migration 004: Shelves

The simplest migration — adding OPDS shelves:

```sql
CREATE TABLE IF NOT EXISTS opds_shelves (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS opds_shelf_items (
    shelf_id INTEGER NOT NULL REFERENCES opds_shelves(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, url_id)
);

INSERT INTO opds_shelves (id, name, description, token)
VALUES (1, 'Default Shelf', 'Shared reading list', 'fichub')
ON CONFLICT (id) DO NOTHING;
```

Shelves are shared reading lists that users can access via OPDS. The `token` field provides authentication — users need to know the token to access a shelf.

## How Migrations Run

FicHub runs migrations automatically at startup. SQLx tracks which migrations have been applied in a special `_sqlx_migrations` table. When the server starts, it checks this table and applies any new migrations.

```rust
sqlx::migrate::Migrator::new(migrations_path)
    .await?
    .run(&pool)
    .await?;
```

This is convenient for development — you just add a new migration file and restart the server. In production, you might want more control (e.g., running migrations during a maintenance window), but for most deployments, automatic migrations work fine.

## Watch Out!

**Never modify applied migrations!** Once a migration has been applied to production, you can't change it. If you need to fix a schema issue, create a new migration.

**Migrations are not reversible!** SQLx doesn't track down-migrations. If you need to roll back, you'll need to write a new migration that undoes the changes.

**Test migrations before deploying!** Run migrations against a copy of your production database to verify they work correctly.

## Summary

In this chapter, we examined all four of FicHub's migration files:
- **001:** Core tables (fic_info, request_log, export_log, blacklists)
- **002:** Recommender tables (fic_works, bookmarks, co-occurrence, suggestions, votes)
- **003:** Tag system (tag_types, tags, aliases, fic_tags, votes, flags, full-text search)
- **004:** OPDS shelves

We also learned about PostgreSQL triggers, generated columns, GIN indexes, and how SQLx manages migrations automatically.

Next, we'll look at how FicHub performs CRUD operations using these tables.

---

# Chapter 9: CRUD Operations

## Create, Read, Update, Delete

The four basic database operations — **CRUD** — are the building blocks of any data-driven application. In this chapter, we'll see how FicHub implements each one using SQLx.

FicHub's query functions live in `src/db/queries.rs`. Let's walk through the most important ones.

## CREATE: Upserting Fic Metadata

The most important "create" operation in FicHub is upserting fic metadata. This is called every time a story is scraped:

```rust
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated,
            status = EXCLUDED.status,
            extra_meta = EXCLUDED.extra_meta,
            raw_extended_meta = EXCLUDED.raw_extended_meta,
            content_hash = EXCLUDED.content_hash,
            updated = NOW()"#,
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(&fic.author_url)
    .bind(&fic.author_local_id)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.description)
    .bind(fic.fic_created)
    .bind(fic.fic_updated)
    .bind(&fic.status)
    .bind(&fic.source)
    .bind(&fic.extra_meta)
    .bind(&fic.raw_extended_meta)
    .bind(fic.source_id)
    .bind(fic.author_id)
    .bind(&fic.content_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

This is a classic upsert — if the fic already exists (same `id`), update its metadata. If it's new, insert it. The `EXCLUDED` keyword refers to the values that were proposed for insertion.

Notice how many fields there are! FicHub stores a lot of metadata about each fic. The `content_hash` is particularly important — it changes when the story's content changes, which triggers cache invalidation.

## READ: Getting Fic Info

The simplest read operation:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

Using `fetch_optional` instead of `fetch_one` is important — if the fic doesn't exist, we get `None` instead of an error. This lets the handler decide how to respond.

## READ: Searching Similar Fics

For the "did you mean?" feature:

```rust
pub async fn search_similar_fics(pool: &PgPool, query: &str) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        r#"SELECT * FROM fic_info
           WHERE title ILIKE $1 OR author ILIKE $2
           LIMIT 5"#,
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

The `ILIKE` operator is case-insensitive `LIKE`. The `%` wildcards match any characters before and after the query.

## CREATE: Logging Requests

Every API request is logged:

```rust
pub async fn insert_request_log(
    pool: &PgPool,
    source_id: i64,
    etype: &str,
    query: &str,
    info_request_ms: i32,
    url_id: Option<&str>,
    fic_info: Option<&str>,
    export_ms: Option<i32>,
    export_file_name: Option<&str>,
    export_file_hash: Option<&str>,
    url: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO request_log
           (source_id, etype, query, info_request_ms, url_id, fic_info,
            export_ms, export_file_name, export_file_hash, url)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
    )
    .bind(source_id)
    .bind(etype)
    .bind(query)
    .bind(info_request_ms)
    .bind(url_id)
    .bind(fic_info)
    .bind(export_ms)
    .bind(export_file_name)
    .bind(export_file_hash)
    .bind(url)
    .execute(pool)
    .await?;
    Ok(())
}
```

Notice the many `Option<&str>` parameters — not all fields are available for every request. The `export_ms` and `export_file_name` are only set when an export is actually generated.

## CREATE: Request Sources

Before logging a request, we need to know the source:

```rust
pub async fn insert_request_source(
    pool: &PgPool,
    is_automated: bool,
    route: &str,
    description: &str,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO request_source (is_automated, route, description)
           VALUES ($1, $2, $3)
           ON CONFLICT (is_automated, route, description) DO UPDATE SET route = EXCLUDED.route
           RETURNING id"#,
    )
    .bind(is_automated)
    .bind(route)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

The `RETURNING id` clause returns the auto-generated ID, which we need for the request log.

## READ: Cache Lookup

Before generating an export, FicHub checks if it's already cached:

```rust
pub async fn find_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
) -> AppResult<Option<ExportLog>> {
    let row = sqlx::query_as::<_, ExportLog>(
        r#"SELECT * FROM export_log
           WHERE url_id = $1 AND version = $2 AND etype = $3 AND input_hash = $4"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

The cache key is a composite of `(url_id, version, etype, input_hash)`. If any of these change, the cache is invalidated.

## CREATE: Recording Exports

After generating an export, FicHub records it:

```rust
pub async fn insert_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
    export_hash: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (url_id, version, etype, input_hash) DO UPDATE SET
               export_hash = EXCLUDED.export_hash,
               created = NOW()"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .bind(export_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

## READ: Blacklist Checks

Before exporting, FicHub checks blacklists:

```rust
pub async fn check_fic_blacklist(pool: &PgPool, url_id: &str) -> AppResult<Vec<FicBlacklist>> {
    let rows = sqlx::query_as::<_, FicBlacklist>(
        "SELECT * FROM fic_blacklist WHERE url_id = $1",
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn check_author_blacklist(
    pool: &PgPool,
    source_id: i64,
    author_id: i64,
) -> AppResult<Vec<AuthorBlacklist>> {
    let rows = sqlx::query_as::<_, AuthorBlacklist>(
        "SELECT * FROM author_blacklist WHERE source_id = $1 AND author_id = $2",
    )
    .bind(source_id)
    .bind(author_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

These return vectors because a fic or author can be blacklisted for multiple reasons.

## Tag Operations

FicHub's tag system has several CRUD operations:

### Lookup a Tag

```rust
pub async fn lookup_tag_by_name(pool: &PgPool, name: &str) -> AppResult<Option<(i32, String, i16)>> {
    let row = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT id, name, tag_type_id FROM tags WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

### Create a Tag

```rust
pub async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id",
    )
    .bind(name)
    .bind(tag_type_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

### Attach a Tag to a Fic

```rust
pub async fn upsert_fic_tag(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    ip: &std::net::IpAddr,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, 0)
           ON CONFLICT (url_id, tag_id) DO UPDATE SET added_by_ip = EXCLUDED.added_by_ip"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(ip.to_string())
    .execute(pool)
    .await?;
    Ok(())
}
```

### Get Tags for a Fic

```rust
pub async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#,
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, type_id, score)| {
            let hidden = score < hidden_threshold;
            (id, name, type_id, score, hidden)
        })
        .collect())
}
```

### Record a Vote

```rust
pub async fn upsert_tag_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
    value: i16,
) -> AppResult<()> {
    // Try update first, then insert
    let affected = sqlx::query(
        r#"UPDATE fic_tag_votes SET value = $1, created_at = NOW()
           WHERE url_id = $2 AND tag_id = $3 AND voter_ip = $4::inet"#,
    )
    .bind(value)
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        sqlx::query(
            r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
               VALUES ($1, $2, $3::inet, $4)"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .bind(value)
        .execute(pool)
        .await?;

        // Ensure fic_tags row exists
        sqlx::query(
            r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
               VALUES ($1, $2, $3::inet, 0)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .execute(pool)
        .await?;
    }
    Ok(())
}
```

This is a manual upsert — try UPDATE first, and if no rows were affected, INSERT. The trigger on `fic_tag_votes` automatically updates the score on `fic_tags`.

## Curator Operations

### Merge Tags

```rust
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    // Reassign fic_tags from source to target (skip duplicates)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft
           WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO UPDATE SET score = GREATEST(fic_tags.score, EXCLUDED.score)"#,
    )
    .bind(source_tag_id)
    .bind(target_tag_id)
    .execute(pool)
    .await?;

    // Delete source fic_tags
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    // Delete source tag
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    Ok(())
}
```

When merging tags, FicHub reassigns all fic-tag associations from the source to the target, keeping the higher score when there's a conflict.

### Delete a Tag

```rust
pub async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    if !force {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1",
        )
        .bind(tag_id)
        .fetch_one(pool)
        .await?;
        if count.0 > 0 {
            return Err(AppError::BadRequest(
                -5,
                format!("tag {} is used by {} fics, use ?force=true", tag_id, count.0),
            ));
        }
    }
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

The `force` parameter prevents accidental deletion of tags that are in use. Without `force`, the function checks if any fics use the tag and returns an error if so.

## Watch Out!

**Watch for N+1 queries!** If you fetch a list of fics and then query tags for each one individually, you'll have N+1 database queries. FicHub's search handler avoids this by batching tag lookups:

```rust
let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
let tag_rows = sqlx::query_as::<_, TagDbRow>(
    "SELECT ... WHERE ft.url_id = ANY($1) ..."
)
.bind(&url_ids)
.fetch_all(&pool)
.await?;
```

The `ANY($1)` operator lets you pass an array of IDs and get all matching rows in a single query.

**Always check rows_affected()** when doing updates or deletes. If you expected to update a row but `rows_affected()` is 0, something went wrong.

**Use transactions for multi-step operations!** The `merge_tags` function should really use a transaction to ensure all three operations succeed or none do. Without a transaction, a failure between steps could leave the database in an inconsistent state.

## Summary

In this chapter, we saw how FicHub implements CRUD operations:
- **Upsert** fic metadata with `ON CONFLICT DO UPDATE`
- **Read** fic info, blacklist entries, and export logs
- **Create** request sources and logs
- **Record** tag votes and suggestions
- **Merge** and **delete** tags with curator tools
- **Batch** queries to avoid N+1 problems

Next, we'll look at the router and middleware — how FicHub organizes its routes and adds cross-cutting concerns.

---

# Chapter 10: Router and Middleware

## The Full Route Tree

In Chapter 5, we built a simple router with a few routes. Now it's time to see FicHub's complete router — all 40+ routes organized by subsystem.

The router is built in `server.rs`'s `build_router` function. Let's walk through the entire thing:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }

    Router::new()
        // ── Core API ─────────────────────────────────────────────
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // ── Cache Downloads ──────────────────────────────────────
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))

        // ── Recommender ──────────────────────────────────────────
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))

        // ── Tags (v3) ───────────────────────────────────────────
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))

        // ── Curator ─────────────────────────────────────────────
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))

        // ── Search ──────────────────────────────────────────────
        .route("/api/v0/search", get(crate::search::routes::search_handler))

        // ── OPDS Catalog ────────────────────────────────────────
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))

        // ── Legacy Redirects ────────────────────────────────────
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))

        // ── Static Frontend (SPA) ───────────────────────────────
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )

        // ── Middleware ───────────────────────────────────────────
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())

        // ── State ───────────────────────────────────────────────
        .with_state(state)
}
```

## Route Organization

The routes are organized by subsystem:

### Core API Routes (GET only)
- `/api/` — API documentation
- `/api/v0/epub` — Export endpoint (the main feature)
- `/api/v0/meta` — Metadata-only endpoint
- `/api/v0/remote` — Client IP information

### Cache Download Routes (GET only)
- `/cache/{etype}/{url_id}/{fname}` — Download with hash validation
- `/cache/{etype}/{url_id}` — Download or trigger export

### Recommender Routes (GET + POST)
- `GET /api/v0/recommendations` — Get recommendations
- `POST /api/v0/recommendations/suggest` — Submit a suggestion
- `POST /api/v0/recommendations/vote` — Vote on a suggestion
- `GET /api/v0/recommendations/votes` — List votes for a fic

### Tag Routes (GET + POST)
- `POST /api/v0/tags/submit` — Submit a tag
- `POST /api/v0/tags/vote` — Vote on a tag
- `POST /api/v0/tags/flag` — Flag a tag
- `GET /api/v0/tags` — Get tags for a fic

### Curator Routes (POST + DELETE + GET)
- `POST /api/v0/curator/alias` — Create a tag alias
- `POST /api/v0/curator/merge` — Merge two tags
- `DELETE /api/v0/curator/tags/{id}` — Delete a tag
- `GET /api/v0/curator/flags` — List unresolved flags
- `POST /api/v0/curator/flags/{id}/resolve` — Resolve a flag

### Search Routes (GET only)
- `GET /api/v0/search` — Full-text search with filters

### OPDS Routes (GET only)
All OPDS routes return Atom XML feeds for e-readers.

## Middleware

FicHub uses two middleware layers:

### TraceLayer

```rust
.layer(TraceLayer::new_for_http())
```

This logs every HTTP request with its method, URI, status code, and response time. The log output looks like:

```
2024-01-15T10:30:00.123Z  INFO request{method=GET uri=/api/v0/epub version=HTTP/1.1}: fichub::routes::export: 200 response_time=1.234s
```

This is invaluable for debugging and monitoring. You can see at a glance which requests are slow, which are failing, and how often each endpoint is called.

### CorsLayer

```rust
.layer(CorsLayer::permissive())
```

CORS (Cross-Origin Resource Sharing) is a browser security feature that prevents web pages from making requests to a different domain. Since FicHub's frontend and backend are served from the same domain, this shouldn't be an issue. But `CorsLayer::permissive()` allows any origin, which is useful during development when the frontend might be running on a different port.

In production, you'd restrict CORS to your frontend's domain:

```rust
.layer(CorsLayer::new()
    .allow_origin("https://fichub.example.com".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any))
```

## The Fallback Service

The fallback service handles any request that doesn't match a route:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This serves the SvelteKit frontend's build output as static files. If a request doesn't match any API route, it tries to serve the corresponding file from the frontend directory. If the file doesn't exist, it serves `index.html` — this is the standard SPA (Single Page Application) fallback pattern.

For example:
- `GET /` → serves `frontend/build/index.html`
- `GET /assets/main.js` → serves `frontend/build/assets/main.js`
- `GET /some/random/path` → serves `frontend/build/index.html` (SPA fallback)

## The Server Startup Sequence

Let's look at how the server starts up in `server.rs`:

```rust
pub async fn run(config: Config) {
    // 1. Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // 2. Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // 3. Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // 4. Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // 5. Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // 6. Load datacenter IPs
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // 7. Create shared state
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });

    // 8. Build router
    let app = build_router(state).await;

    // 9. Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}
```

The startup sequence is:
1. Connect to PostgreSQL (with migrations)
2. Connect to Redis
3. Build the HTTP client (with user agent and timeout)
4. Initialize the scraper registry (register all scrapers)
5. Initialize the rate limiter (load the Lua script)
6. Load datacenter IPs (for blocking bot traffic)
7. Create the application state (the big `AppState` struct)
8. Build the router (all routes + middleware)
9. Bind to a port and start serving

Each step is logged, so if something fails during startup, you'll know exactly where the problem is.

## The AppState Struct

The `AppState` struct is the heart of FicHub — it holds everything that needs to be shared across handlers:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

Let's look at each field:

- **config** — All configuration values
- **db** — PostgreSQL connection pool
- **redis** — Redis multiplexed connection (supports concurrent operations)
- **http_client** — Shared HTTP client for scraping (reuses TCP connections)
- **scraper_registry** — All registered site scrapers
- **cache_semaphores** — Prevents duplicate concurrent exports
- **rate_limiter** — Token bucket rate limiter backed by Redis
- **recommender_engine** — Computes recommendations
- **collection_worker** — Background worker for scraping user favourites

The entire `AppState` is wrapped in `Arc` so it can be shared across all handlers without cloning.

## Watch Out!

**Route order matters!** Axum matches routes in the order they're defined. If you have both `/api/v0/tags` and `/api/v0/tags/submit`, the more specific route should come first. In FicHub, the routes are ordered correctly, but be careful when adding new ones.

**Middleware order matters!** Layers are applied in reverse order. The last `.layer()` added runs first. So `TraceLayer` runs before `CorsLayer`. This is usually what you want, but be aware of it.

**The fallback catches everything!** If you accidentally put your fallback before your API routes, all requests would be served from the frontend directory. Always put the fallback last.

**ConnectInfo requires special setup!** If you use `ConnectInfo` in a handler but forget `into_make_service_with_connect_info`, you'll get a runtime error. The `?` in `into_make_service_with_connect_info::<std::net::SocketAddr>()` is the turbofish syntax for specifying the type.

## Summary

In this chapter, we've seen:
- FicHub's complete route tree (40+ routes)
- How routes are organized by subsystem
- How middleware adds logging and CORS
- How the fallback service serves the SPA frontend
- The server startup sequence (9 steps)
- The `AppState` struct and its role

You now have a solid understanding of the Axum layer of FicHub. In the next part, we'll dive into the scraping system — the heart of FicHub's functionality.
