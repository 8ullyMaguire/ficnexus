# Part 2: Axum Web Framework

---

# Chapter 5: Your First Web Server

Let's build our first web server with Axum. By the end of this chapter, you'll have a working HTTP server that responds to requests with JSON data. This is the foundation everything else builds on.

## Why Axum?

There are several excellent Rust web frameworks — Actix Web, Rocket, Warp, and Axum among them. We chose Axum for several reasons:

1. **Type-safe extractors** — Axum uses Rust's type system to extract request data. Instead of manually parsing query parameters, you just declare the types you want and Axum does the rest.

2. **Tower integration** — Axum is built on Tower, which provides a rich ecosystem of middleware. Need CORS? Compression? Rate limiting? Tower has you covered.

3. **Ergonomic error handling** — Axum's error handling is clean and composable. You define error types, and Axum converts them to HTTP responses automatically.

4. **Async-native** — Axum is built on Tokio and handles async code naturally. No need for special async frameworks or runtimes.

5. **Lightweight** — Axum is a thin layer over Tower and Hyper. There's very little magic — what you see is what you get.

**Real-world analogy:** If web frameworks were restaurants, Actix Web would be a fine dining establishment with its own unique recipes. Rocket would be a trendy bistro with a beautiful menu. Axum would be a professional kitchen with modular stations — each station (middleware) does one thing, and they work together seamlessly.

## Your First Handler

Let's start with the simplest possible Axum server:

```rust
use axum::{routing::get, Router, Json};
use serde_json::json;

async fn hello() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Hello from FicHub!",
        "version": "0.1.0"
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
```

Let's break this down line by line:

**Line 1:** Import the types we need from Axum. `routing::get` creates a route handler for GET requests. `Router` is the central routing table. `Json` wraps a response body in JSON format.

**Line 2:** Import `serde_json::json!` macro for creating JSON values.

**Lines 4-8:** Define an async function called `hello` that returns a `Json<serde_json::Value>`. The `async` keyword means this function can perform asynchronous operations (like database queries or HTTP requests) without blocking. The `Json` wrapper tells Axum to set the `Content-Type: application/json` header and serialize the value to JSON.

**Lines 10-11:** The `#[tokio::main]` attribute transforms your `main` function into a Tokio runtime entry point. It sets up the event loop, thread pool, and other async infrastructure.

**Lines 13-15:** Create a `Router` and add a route. `.route("/", get(hello))` means "when a GET request comes to `/`, call the `hello` function."

**Lines 17-19:** Create a TCP listener on port 3000 and start the server. The `0.0.0.0` address means "listen on all network interfaces."

**Line 21:** `axum::serve` starts the server and listens for incoming requests.

### How Axum Routes Requests

When a request arrives, Axum follows this process:

1. **Match the route** — Find the handler function for this URL and method
2. **Extract parameters** — Parse query parameters, path segments, headers, etc.
3. **Call the handler** — Pass the extracted data to the handler function
4. **Convert the response** — The handler's return type is converted to an HTTP response
5. **Send the response** — The response is sent back to the client

The key insight is that Axum uses Rust's type system for extraction. When you declare a parameter type, Axum knows how to extract it:

```rust
async fn get_story(
    Query(params): Query<StoryQuery>,  // Extract query parameters
    State(state): State<Arc<AppState>>,  // Extract shared state
) -> Json<serde_json::Value> {
    // params.url is now a String
    // state.db is now a PgPool
    todo!()
}

#[derive(serde::Deserialize)]
struct StoryQuery {
    url: String,
}
```

The `Query` extractor parses the URL query string into a `StoryQuery` struct. If the parsing fails (missing required field, wrong type), Axum automatically returns a 400 Bad Request error. You don't need to write any parsing code!

## Building a Multi-Route Server

Let's expand our server to handle multiple routes:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

// Our application state
struct AppState {
    db_url: String,
}

// Query parameters for the story endpoint
#[derive(Deserialize)]
struct StoryQuery {
    url: String,
}

// Response types
#[derive(Serialize)]
struct StoryResponse {
    err: i32,
    title: String,
    author: String,
    word_count: i64,
}

// Handler: GET /api/v0/stories
async fn list_stories(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    Json(json!({
        "err": 0,
        "stories": [],
        "count": 0
    }))
}

// Handler: GET /api/v0/stories/:id
async fn get_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<StoryResponse>, StatusCode> {
    // In a real app, you'd query the database here
    if id == "notfound" {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(Json(StoryResponse {
        err: 0,
        title: "Example Story".to_string(),
        author: "Example Author".to_string(),
        word_count: 50000,
    }))
}

// Handler: GET /api/v0/search
async fn search_stories(
    Query(params): Query<StoryQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    Json(json!({
        "err": 0,
        "query": params.url,
        "results": []
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        db_url: "postgres://localhost/fichub".to_string(),
    });
    
    let app = Router::new()
        .route("/api/v0/stories", get(list_stories))
        .route("/api/v0/stories/{id}", get(get_story))
        .route("/api/v0/search", get(search_stories))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Path Parameters

The `{id}` in the route pattern is a path parameter. Axum extracts it and passes it to the handler as a `Path<String>`:

```rust
async fn get_story(
    Path(id): Path<String>,  // Extracts "id" from the URL
) -> Json<Value> {
    println!("Requested story: {}", id);
    Json(json!({"id": id}))
}
```

You can have multiple path parameters:

```rust
async fn get_chapter(
    Path((story_id, chapter_id)): Path<(String, i32)>,
) -> Json<Value> {
    println!("Story {}, Chapter {}", story_id, chapter_id);
    Json(json!({"story": story_id, "chapter": chapter_id}))
}
```

And you can use struct types for complex extractions:

```rust
#[derive(Deserialize)]
struct StoryPath {
    story_id: String,
    chapter_id: i32,
}

async fn get_chapter(
    Path(path): Path<StoryPath>,
) -> Json<Value> {
    println!("Story {}, Chapter {}", path.story_id, path.chapter_id);
    Json(json!({"story": path.story_id, "chapter": path.chapter_id}))
}
```

### Query Parameters

Query parameters are extracted with `Query<T>`:

```rust
#[derive(Deserialize)]
struct SearchParams {
    q: String,                    // Required
    page: Option<u32>,            // Optional (defaults to None)
    #[serde(default = "default_limit")]
    limit: u32,                   // Optional with default
}

fn default_limit() -> u32 { 20 }

async fn search(
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit;
    
    Json(json!({
        "query": params.q,
        "page": page,
        "limit": limit
    }))
}
```

The `#[serde(default = "default_limit")]` attribute tells serde to use the `default_limit` function when the field is missing from the query string. This is cleaner than using `Option<u32>` and handling `None` everywhere.

### State Extraction

Shared state is extracted with `State<T>`:

```rust
use std::sync::Arc;

struct AppState {
    db: sqlx::PgPool,
    redis: redis::aio::MultiplexedConnection,
}

async fn handler(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    // Use state.db to query the database
    // Use state.redis to access Redis
    Json(json!({"status": "ok"}))
}

// In main:
let state = Arc::new(AppState { /* ... */ });
let app = Router::new()
    .route("/", get(handler))
    .with_state(state);
```

The `Arc<AppState>` is an atomic reference counter — it allows multiple handlers to share the same state without copying it. When the last `Arc` is dropped, the state is freed.

### Returning Different Response Types

Handlers can return different types depending on the situation:

```rust
async fn get_story(
    Path(id): Path<String>,
) -> Result<Json<StoryResponse>, StatusCode> {
    // Success: return the story
    // Error: return a status code
    
    match fetch_story(&id).await {
        Ok(story) => Ok(Json(story)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}
```

For more complex responses, you can use `IntoResponse`:

```rust
use axum::response::{IntoResponse, Response};

async fn get_story(
    Path(id): Path<String>,
) -> Response {
    match fetch_story(&id).await {
        Ok(story) => (
            StatusCode::OK,
            Json(json!({"err": 0, "story": story}))
        ).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({"err": -1, "msg": e.to_string()}))
        ).into_response(),
    }
}
```

### HTTP Methods

Axum supports all HTTP methods:

```rust
let app = Router::new()
    .route("/stories", get(list_stories).post(create_story))
    .route("/stories/{id}", get(get_story).put(update_story).delete(delete_story))
    .route("/stories/{id}/publish", post(publish_story));
```

You can chain multiple methods on the same route. The `.post(create_story)` part adds a POST handler to the `/stories` route.

### Headers and Request Body

To access headers, use the `HeaderMap` extractor:

```rust
use axum::http::HeaderMap;

async fn handler(
    headers: HeaderMap,
) -> Json<Value> {
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    Json(json!({"user_agent": user_agent}))
}
```

For POST requests with JSON bodies, use the `Json<T>` extractor:

```rust
#[derive(Deserialize)]
struct CreateStory {
    title: String,
    url: String,
}

async fn create_story(
    Json(payload): Json<CreateStory>,
) -> (StatusCode, Json<Value>) {
    // Create the story in the database
    let story_id = "abc123".to_string();
    
    (
        StatusCode::CREATED,
        Json(json!({
            "err": 0,
            "id": story_id,
            "title": payload.title
        }))
    )
}
```

⚠️ **Watch Out:** The order of extractors matters. The `State` extractor must come first, followed by other extractors. If you put `State` after `Json`, you'll get a compile error.

## 📝 Practice Exercises

1. **Build a Calculator API:** Create routes for `GET /add?a=1&b=2`, `GET /subtract?a=5&b=3`, `GET /multiply?a=4&b=6`. Each should return JSON with the result.

2. **Add POST Support:** Add a `POST /stories` endpoint that accepts JSON with `title`, `author`, and `content` fields. Return the created story with an ID.

3. **Path Parameters:** Create `GET /users/{user_id}/stories/{story_id}`. Extract both IDs and return them in the response.

4. **Error Handling:** Modify the story endpoint to return 404 when the story ID doesn't match any known story, and 400 when the ID is empty.

5. **Headers:** Create an endpoint that returns the client's User-Agent, Accept-Language, and X-Forwarded-For headers.

---

# Chapter 6: Configuration and Error Handling

Every real application needs configuration and error handling. In this chapter, we'll build FicHub's configuration system and custom error types. These are foundational patterns you'll use throughout the codebase.

## Configuration from Environment Variables

FicHub loads all configuration from environment variables. This is a common pattern for twelve-factor apps — applications that are designed to be deployed to cloud environments where configuration comes from the environment, not from config files.

### The Config Struct

Here's FicHub's configuration struct:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
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
    pub opds_shelf_token: String,
}
```

**Real-world analogy:** The Config struct is like a blueprint for a house. It defines every knob, switch, and setting that can be adjusted. The `from_env()` method reads these settings from the environment — like reading the switches on a control panel.

### Loading Configuration

The `from_env()` method reads each configuration value from the environment:

```rust
impl Config {
    pub fn from_env() -> Self {
        // Required: DATABASE_URL
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        // Required: REDIS_URL
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        // Optional with default: CACHE_DIR
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        // Optional with complex parsing: TRUSTED_PROXIES
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // Optional with JSON parsing: REC_SITE_RATE_LIMITS
        let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
            .unwrap_or_else(|_| "{}".to_string());
        let rec_site_rate_limits: HashMap<String, u64> =
            serde_json::from_str(&rec_site_rate_limits_str)
            .unwrap_or_default();
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            trusted_proxies,
            rec_site_rate_limits,
            // ... other fields
        }
    }
}
```

Let's examine the patterns used here:

**`expect("message")`** — Panics with a message if the environment variable is not set. This is for required configuration that the application cannot function without.

**`unwrap_or_else(|_| "default")`** — Returns a default value if the environment variable is not set. This is for optional configuration with sensible defaults.

**`unwrap_or_default()`** — Returns the default value for the type if parsing fails. For `HashMap`, the default is an empty map.

**Complex parsing** — The `TRUSTED_PROXIES` value is split by commas, trimmed, and collected into a `Vec<String>`. The `REC_SITE_RATE_LIMITS` value is parsed as JSON.

### Testing Configuration

Testing configuration is tricky because environment variables are global state. FicHub uses a mutex to serialize tests that modify environment variables:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    
    struct EnvGuard {
        keys: Vec<String>,
    }
    
    impl EnvGuard {
        fn new() -> Self {
            EnvGuard { keys: Vec::new() }
        }
        
        fn set(&mut self, key: &str, val: &str) {
            self.keys.push(key.to_string());
            // SAFETY: Test-only env var manipulation under ENV_LOCK.
            unsafe { std::env::set_var(key, val); }
        }
    }
    
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for key in &self.keys {
                // SAFETY: Test-only env var cleanup under ENV_LOCK.
                unsafe { std::env::remove_var(key); }
            }
        }
    }
    
    #[test]
    fn test_from_env_defaults() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test_db");
        guard.set("REDIS_URL", "redis://localhost/0");
        
        let config = Config::from_env();
        
        assert_eq!(config.database_url, "postgres://localhost/test_db");
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert_eq!(config.app_port, 3000);
    }
}
```

The `EnvGuard` struct saves which environment variables it sets and restores them when dropped. This ensures tests don't interfere with each other.

## Custom Error Types

FicHub defines a comprehensive error type that covers all the ways the application can fail:

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message
    BadRequest(i32, String),
    /// Rate limited - wait N seconds
    RateLimited(u64),
    /// Resource not found
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
}
```

### Display Implementation

The `Display` trait provides human-readable error messages:

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

### IntoResponse Implementation

The `IntoResponse` trait converts errors to HTTP responses. This is where the magic happens — each error variant maps to a specific HTTP status code and JSON response:

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
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
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

Key design decisions:
- **Internal errors log the full message** but return a generic error to the client. This prevents information leakage.
- **Scrape errors return 502 Bad Gateway** because they represent upstream failures.
- **Rate limiting returns 429** with a `retry_after` field so the client knows when to retry.
- **All responses include an `err` field** for consistent error handling on the client side.

### From Implementations for Error Conversion

FicHub implements `From` for common error types, allowing the `?` operator to automatically convert them:

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

Now you can write code like this:

```rust
async fn get_story(
    State(state): State<Arc<AppState>>,
    Query(params): Query<StoryQuery>,
) -> Result<Json<StoryResponse>, AppError> {
    // sqlx::Error automatically converts to AppError::Database
    let story = sqlx::query_as::<_, StoryInfo>(
        "SELECT * FROM fic_info WHERE id = $1"
    )
    .bind(&params.url_id)
    .fetch_optional(&state.db)
    .await?;  // The ? operator does the conversion
    
    // redis::RedisError automatically converts to AppError::CacheError
    let cached: Option<String> = redis::cmd("GET")
        .arg(&params.url_id)
        .query_async(&mut state.redis.clone())
        .await?;  // The ? operator does the conversion
    
    match story {
        Some(story) => Ok(Json(StoryResponse::from(story))),
        None => Err(AppError::NotFound("Story not found".into())),
    }
}
```

The `?` operator is Rust's error propagation shorthand. If the result is an `Err`, it converts it to `AppError` using the `From` implementation and returns early. If it's an `Ok`, it unwraps the value.

### The AppResult Type Alias

To reduce verbosity, FicHub defines a type alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Now you can write:

```rust
async fn get_story(id: &str) -> AppResult<StoryResponse> {
    // ...
    Ok(StoryResponse { /* ... */ })
}
```

Instead of:

```rust
async fn get_story(id: &str) -> Result<StoryResponse, AppError> {
    // ...
    Ok(StoryResponse { /* ... */ })
}
```

## Structured Logging with Tracing

FicHub uses the `tracing` crate for structured logging. Unlike traditional logging (which just writes text), structured logging creates machine-readable events with fields:

```rust
use tracing::{info, warn, error, debug};

async fn process_request(url: &str) -> Result<(), AppError> {
    info!(url = %url, "Processing request");
    
    // ... scraping logic ...
    
    if is_cached {
        debug!(url_id = %url_id, "Cache hit");
        return Ok(());
    }
    
    warn!(url = %url, "Cache miss, fetching from source");
    
    // ... export logic ...
    
    if let Err(e) = export_result {
        error!(url = %url, error = %e, "Export failed");
        return Err(AppError::ExportError(e.to_string()));
    }
    
    info!(url_id = %url_id, "Export complete");
    Ok(())
}
```

The `%` in `url = %url` means "format this field using the Display trait." Without it, tracing uses the Debug format, which is more verbose.

### Log Levels

- **ERROR** — Something went wrong and needs immediate attention
- **WARN** — Something unexpected happened but the application can continue
- **INFO** — Normal operation messages (requests received, exports completed)
- **DEBUG** — Detailed information for debugging (cache lookups, query parameters)
- **TRACE** — Very detailed information (every function call, every database query)

### Configuring Log Levels

In FicHub's `main.rs`, we configure logging from the `RUST_LOG` environment variable:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
    )
    .init();
```

This means:
- Default: `info` level for all modules, `debug` level for fichub
- Override: Set `RUST_LOG=debug` for verbose output, `RUST_LOG=error` for minimal output

## 📝 Practice Exercises

1. **Add Configuration:** Add a new configuration field `MAX_UPLOAD_SIZE` that reads from the environment variable `MAX_UPLOAD_SIZE_MB`. Default to 10 MB.

2. **Custom Error:** Create a new error variant `Authentication(String)` that returns 401 Unauthorized with the message.

3. **Error Conversion:** Implement `From<reqwest::Error>` for your custom error type. Then write a function that makes an HTTP request and uses `?` to propagate errors.

4. **Structured Logging:** Add tracing to a handler function. Log the request URL, response time, and any errors.

5. **Configuration Testing:** Write a test that verifies the default configuration values are correct.

---

# Chapter 7: Connecting to PostgreSQL

Now let's connect to PostgreSQL and start querying data. This chapter covers connection pooling, query execution, and working with the database in an async context.

## Why SQLx?

FicHub uses SQLx as its database library. Here's why:

1. **Compile-time checked queries** — SQLx verifies your SQL against the actual database schema at compile time. If you make a typo, your code won't compile.

2. **Async support** — SQLx is built on Tokio and handles async database operations natively.

3. **Type-safe** — SQLx maps SQL types to Rust types automatically. No manual conversion needed.

4. **Connection pooling** — SQLx manages a pool of database connections for you.

5. **Migrations** — SQLx includes a migration system for managing database schema changes.

**Real-world analogy:** SQLx is like a GPS navigation system for databases. It knows the road network (your schema), checks your route before you leave (compile-time validation), and handles the driving (async I/O) so you can focus on your destination (business logic).

## Connection Pool Setup

Here's how FicHub initializes its database connection pool:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    tracing::info!("Database pool initialized with {} connections", pool.options().max_connections);
    
    Ok(pool)
}
```

Let's break down each option:

**`max_connections(20)`** — The maximum number of connections in the pool. This is the most important setting. Too few connections and requests will wait for a connection to become available. Too many connections and PostgreSQL will run out of resources.

**Real-world analogy:** Think of the connection pool as a taxi stand. `max_connections` is the number of taxis at the stand. If all taxis are out serving passengers, new passengers have to wait. If there are too many taxis, they take up space and cost money.

**`min_connections(2)`** — The minimum number of connections to keep open. This ensures there are always connections ready to handle requests, even during startup.

**`acquire_timeout(Duration::from_secs(30))`** — How long to wait for a connection before timing out. If all connections are busy and this timeout expires, the request fails.

**`idle_timeout(Duration::from_secs(600))`** — How long a connection can sit idle before being closed. This prevents the pool from holding connections that aren't needed.

**`max_lifetime(Duration::from_secs(1800))`** — The maximum age of a connection. After this time, the connection is closed and replaced. This prevents issues with stale connections.

### Pool Sizing Guidelines

How many connections should you configure? Here are some guidelines:

- **Small deployment (1-2 instances, low traffic):** 10-20 connections
- **Medium deployment (2-5 instances, moderate traffic):** 20-50 connections
- **Large deployment (5+ instances, high traffic):** 50-100 connections

The formula is roughly: `connections_per_instance = (2 * num_cpu_cores) + num_disks`. For a 4-core server, that's about 10 connections. But since FicHub does a lot of I/O (scraping, file generation), you can go higher.

⚠️ **Watch Out:** Don't set `max_connections` too high. PostgreSQL has a `max_connections` setting (default 100). If all instances of your application try to use 100 connections each, PostgreSQL will refuse connections. Set `max_connections` in PostgreSQL higher than the sum of all application pools.

## Querying the Database

### Simple Queries

For simple queries, use `sqlx::query`:

```rust
use sqlx::PgPool;

async fn count_stories(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_info"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(row.0)
}
```

The `query_as` function maps the result to a tuple or struct. Here, `(i64,)` is a tuple with one element — the count.

### Parameterized Queries

Always use parameterized queries to prevent SQL injection:

```rust
async fn get_story(pool: &PgPool, id: &str) -> Result<Option<FicInfo>, sqlx::Error> {
    let fic = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1"
    )
    .bind(id)  // Bind the parameter
    .fetch_optional(pool)
    .await?;
    
    Ok(fic)
}
```

The `$1` is a placeholder for the first parameter. The `.bind(id)` call binds the `id` value to that placeholder. SQLx handles all the escaping and type conversion.

**Never concatenate strings into SQL queries:**

```rust
// DANGEROUS: SQL injection vulnerability!
let query = format!("SELECT * FROM fic_info WHERE id = '{}'", id);

// SAFE: Use parameterized queries
let query = "SELECT * FROM fic_info WHERE id = $1";
```

### Fetching Multiple Rows

For queries that return multiple rows:

```rust
async fn search_stories(pool: &PgPool, query: &str) -> Result<Vec<FicInfo>, sqlx::Error> {
    let fics = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE title ILIKE $1 LIMIT 10"
    )
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    
    Ok(fics)
}
```

`fetch_all` returns a `Vec` of all matching rows. `fetch_one` returns exactly one row (and fails if there are zero or more than one). `fetch_optional` returns zero or one row.

### INSERT with Returning

For inserts that return a value:

```rust
async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> Result<i32, sqlx::Error> {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id"
    )
    .bind(name)
    .bind(tag_type_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.0)
}
```

The `RETURNING id` clause tells PostgreSQL to return the generated ID after insertion.

### Upsert (INSERT ON CONFLICT)

FicHub uses upserts extensively — insert a row if it doesn't exist, update it if it does:

```rust
async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, chapters, words, status, source
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            status = EXCLUDED.status,
            updated = NOW()"#
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.status)
    .bind(&fic.source)
    .execute(pool)
    .await?;
    
    Ok(())
}
```

The `ON CONFLICT (id) DO UPDATE` clause means: if a row with this ID already exists, update the specified columns. The `EXCLUDED` keyword refers to the values that were going to be inserted.

**Real-world analogy:** Upsert is like checking if a book is already in the library catalog. If it is, you update the information (maybe the author changed their name). If it isn't, you add it as a new entry. Either way, the catalog is up to date.

### Transactions

When you need to perform multiple operations that must all succeed or all fail:

```rust
async fn transfer_tag(
    pool: &PgPool,
    from_url_id: &str,
    to_url_id: &str,
    tag_id: i32,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Remove tag from source
    sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(from_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    // Add tag to destination
    sqlx::query("INSERT INTO fic_tags (url_id, tag_id, score) VALUES ($1, $2, 0)")
        .bind(to_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    // Commit the transaction
    tx.commit().await?;
    
    Ok(())
}
```

The `begin()` method starts a transaction. All queries within the transaction are executed on the same connection. If any query fails, the transaction is rolled back and none of the changes take effect.

## Working with the FicInfo Model

Here's how FicHub's database models are defined:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

The `#[derive(FromRow)]` attribute tells SQLx how to map database columns to struct fields. SQLx matches columns by name — if the column is `title`, it looks for a field called `title` in the struct.

The `Option<T>` types represent nullable columns. If the database column is NULL, the field will be `None`. If it has a value, the field will be `Some(value)`.

### The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

This model tracks cached exports. When FicHub generates an EPUB, it records the export in this table so future requests can find the cached version.

## 📝 Practice Exercises

1. **Connection Pool:** Create a connection pool with 5 max connections and test it by running 10 concurrent queries. Observe how they're queued.

2. **Upsert Practice:** Write an upsert function for a `users` table that inserts a new user or updates the last_login timestamp if the user already exists.

3. **Transaction:** Write a function that moves a tag from one story to another within a transaction. Verify that if the second query fails, the first is rolled back.

4. **Model Mapping:** Create a struct `StoryStats` with fields for `total_stories`, `total_words`, and `average_words_per_story`. Write a query that returns this data.

5. **Error Handling:** Write a function that queries the database and handles three different error cases: connection failure, query error, and no results found.

---

# Chapter 8: Database Migrations and Models

Migrations are version-controlled changes to your database schema. They ensure that every developer and every deployment has the same database structure. This chapter covers how FicHub manages its database schema.

## Understanding Migrations

A migration is a SQL script that modifies the database schema. Migrations are numbered sequentially and are applied in order. Once applied, they're never reapplied.

**Real-world analogy:** Migrations are like version control for your database. Just as git tracks changes to your code, migrations track changes to your schema. If you need to go back to a previous state, you can reverse a migration (rollback).

### FicHub's Migration Files

FicHub has four migration files:

```
migrations/
├── 001_initial_schema.sql
├── 002_recommender.sql
├── 003_tagging.sql
└── 004_shelves.sql
```

Each migration is a complete SQL script that transforms the schema from one version to the next.

### Running Migrations

With the `sqlx-cli` tool:

```bash
# Apply all pending migrations
sqlx migrate run --source migrations

# Check which migrations have been applied
sqlx migrate info --source migrations

# Revert the last migration (use with caution!)
sqlx migrate revert --source migrations
```

### Creating New Migrations

When you need to change the schema, create a new migration:

```bash
sqlx migrate add --source migrations add_recommendation_tables
```

This creates a new file like `005_add_recommendation_tables.sql` with a timestamp. Edit the file with your SQL changes, then run `sqlx migrate run` to apply them.

## The Initial Schema

Let's look at the core tables FicHub uses:

```sql
-- 001_initial_schema.sql

-- Core fic metadata
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL DEFAULT 1,
    words INT8 NOT NULL DEFAULT 0,
    description TEXT DEFAULT '',
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    source_id INT8,
    author_id INT8,
    content_hash TEXT,
    extra_meta JSONB,
    raw_extended_meta JSONB
);

-- Export log (cache tracking)
CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

-- Request tracking
CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

-- Blacklists
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

-- Cache version bumps
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

### Key Design Decisions

**Primary Keys:** FicHub uses `VARCHAR(128)` for story IDs instead of auto-incrementing integers. This is because IDs are generated from the source site's IDs using a hash function. Using `VARCHAR` allows the same story to have the same ID across different FicHub instances.

**JSONB for Metadata:** The `extra_meta` and `raw_extended_meta` columns use PostgreSQL's `JSONB` type. This allows storing flexible, schema-less metadata without altering the table structure. Different fanfiction sites have different metadata fields, and JSONB handles this elegantly.

**Composite Primary Key:** The `export_log` table uses a composite primary key (`url_id`, `version`, `etype`, `input_hash`). This ensures that each unique combination of story, export version, export type, and input hash is stored exactly once.

**Indexes:** The indexes on `status`, `source`, `words`, `fic_updated`, and `url_id` are crucial for query performance. Without them, PostgreSQL would have to scan the entire table for every query.

## The Recommender Schema

```sql
-- 002_recommender.sql

-- Users who have been processed by the collection worker
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

-- User favourites (what stories a user has bookmarked)
CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

-- Co-occurrence counts (how often two stories are favourited together)
CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

-- Precomputed recommendations
CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

-- Suggestion queue
CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

-- Votes on recommendations
CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

### Understanding Co-occurrence

The co-occurrence table is the heart of the recommendation engine. When two stories appear in the same user's favourites, we increment the count for that pair. Stories that are favourited together frequently are likely similar.

**Real-world analogy:** Co-occurrence is like a bookstore's "customers who bought this also bought" feature. If many customers buy both "Book A" and "Book B", the store knows these books are related. Similarly, if many users favourite both "Story A" and "Story B", FicHub knows these stories appeal to the same audience.

## The Tagging Schema

```sql
-- 003_tagging.sql

-- Canonical tags
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Tag aliases (alternative names that map to canonical tags)
CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Story-tag associations
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

-- Votes on tags
CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

-- Flagged tags (for moderation)
CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

-- Tag type definitions
CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

### Understanding Tag Types

FicHub uses a type system for tags, similar to AO3's tag categories:

1. **Fandom** (tag_type_id = 1) — The source material (Harry Potter, Star Wars, etc.)
2. **Character** (tag_type_id = 2) — Individual characters (Harry Potter, Hermione Granger)
3. **Relationship** (tag_type_id = 3) — Romantic pairings (Harry/Hermione)
4. **Freeform** (tag_type_id = 4) — Any other tags (Angst, Humor, Slow Burn)
5. **Warning** (tag_type_id = 5) — Content warnings (Violence, Major Character Death)
6. **Category** (tag_type_id = 6) — Gen, Slash, F/M, Multi, Other

### Tag Voting System

Tags can be voted on by users. Each vote has a value:
- **+1** — "This tag is accurate"
- **0** — "This tag is okay"
- **-1** — "This tag is wrong"

The tag's score is the sum of all votes. Tags with low scores are hidden (below `tag_hidden_threshold`). This community-driven moderation ensures that tags remain accurate and useful.

## 📝 Practice Exercises

1. **Design a Migration:** Design a migration that adds a `reading_lists` table with columns for `id`, `name`, `description`, `created_at`, and `updated_at`. Write the SQL and run it.

2. **Add an Index:** Add an index on the `fic_tags` table for the `tag_id` column. Why is this index useful?

3. **JSONB Query:** Write a query that searches the `extra_meta` JSONB column for stories with a specific field value.

4. **Schema Documentation:** Document each table in the FicHub schema. What is each table used for? What are the key columns?

5. **Migration Rollback:** Write a rollback migration that drops the `reading_lists` table you created.

---

# Chapter 9: CRUD Operations

CRUD stands for Create, Read, Update, Delete — the four basic operations you can perform on data. This chapter covers how FicHub implements each of these operations.

## Create: Inserting Data

### Single Insert

```rust
async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id"
    )
    .bind(name)
    .bind(tag_type_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.0)
}
```

The `RETURNING id` clause tells PostgreSQL to return the auto-generated ID after insertion.

### Batch Insert

For inserting multiple rows at once:

```rust
async fn insert_fic_tags(
    pool: &PgPool,
    url_id: &str,
    tags: &[(i32, i16)],  // (tag_id, score)
) -> AppResult<()> {
    let mut query = String::from(
        "INSERT INTO fic_tags (url_id, tag_id, score) VALUES "
    );
    
    for (i, (tag_id, score)) in tags.iter().enumerate() {
        if i > 0 {
            query.push(',');
        }
        query.push_str(&format!("($1, ${}, ${})", i * 2 + 2, i * 2 + 3));
    }
    
    query.push_str(" ON CONFLICT DO NOTHING");
    
    let mut q = sqlx::query(&query).bind(url_id);
    for (tag_id, score) in tags {
        q = q.bind(tag_id).bind(score);
    }
    
    q.execute(pool).await?;
    Ok(())
}
```

⚠️ **Watch Out:** Building queries with string formatting is dangerous for user input. This example only works because `tag_id` and `score` are integers, not user-provided strings. For user input, always use parameterized queries.

## Read: Querying Data

### Fetch One

```rust
async fn get_tag_by_name(pool: &PgPool, name: &str) -> AppResult<Option<(i32, String, i16)>> {
    let row = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT id, name, tag_type_id FROM tags WHERE name = $1"
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    
    Ok(row)
}
```

`fetch_optional` returns `None` if no row matches, instead of failing.

### Fetch Many

```rust
async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#
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

### Join Queries

Joins combine data from multiple tables:

```rust
async fn get_recent_fics(pool: &PgPool, limit: i64) -> AppResult<Vec<FicInfo>> {
    let fics = sqlx::query_as::<_, FicInfo>(
        r#"SELECT fi.*
           FROM fic_info fi
           WHERE fi.status = 'complete'
           ORDER BY fi.fic_updated DESC
           LIMIT $1"#
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    Ok(fics)
}
```

## Update: Modifying Data

### Simple Update

```rust
async fn update_fic_status(
    pool: &PgPool,
    url_id: &str,
    status: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE fic_info SET status = $1, updated = NOW() WHERE id = $2"
    )
    .bind(status)
    .bind(url_id)
    .execute(pool)
    .await?;
    
    Ok(())
}
```

### Conditional Update

```rust
async fn update_tag_score(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    score_delta: i16,
) -> AppResult<i16> {
    let row: (i16,) = sqlx::query_as(
        r#"UPDATE fic_tags
           SET score = score + $1
           WHERE url_id = $2 AND tag_id = $3
           RETURNING score"#
    )
    .bind(score_delta)
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;
    
    Ok(row.0)
}
```

## Delete: Removing Data

### Simple Delete

```rust
async fn remove_tag_from_fic(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2"
    )
    .bind(url_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    
    Ok(())
}
```

### Conditional Delete

```rust
async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    if !force {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1"
        )
        .bind(tag_id)
        .fetch_one(pool)
        .await?;
        
        if count.0 > 0 {
            return Err(AppError::BadRequest(
                -5,
                format!("tag {} is used by {} fics, use ?force=true", tag_id, count.0)
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

This function demonstrates a safety check — it refuses to delete a tag that's in use unless `force` is true. This prevents accidental data loss.

## 📝 Practice Exercises

1. **CRUD for Books:** Create a complete CRUD implementation for a `books` table: insert a book, get a book by ID, update the title, and delete the book.

2. **Batch Operations:** Write a function that inserts 100 tags in a single query. Measure how much faster this is compared to inserting them one at a time.

3. **Soft Delete:** Implement a soft delete pattern where records are marked as deleted (with a `deleted_at` timestamp) instead of actually removing them.

4. **Audit Log:** Write a function that logs every update to the `fic_info` table in a separate `audit_log` table.

---

# Chapter 10: The Axum Router and Middleware

The Axum router is the central dispatch mechanism. It matches incoming requests to handler functions and applies middleware. This chapter covers routing patterns, middleware composition, and how FicHub wires everything together.

## Router Composition

Axum routers can be composed — you build smaller routers and combine them:

```rust
use axum::{routing::get, Router};
use std::sync::Arc;

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/epub", get(routes::export::epub_handler))
        .route("/meta", get(routes::meta::meta_handler))
        .route("/search", get(crate::search::routes::search_handler))
}

fn recommender_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
}

fn tag_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/tags", get(crate::tags::routes::get_tags))
}

fn opds_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(crate::routes::opds::feeds::root_catalog))
        .route("/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/popular", get(crate::routes::opds::feeds::popular_feed))
}

fn all_routes() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/api/v0", api_routes())
        .nest("/api/v0", recommender_routes())
        .nest("/api/v0", tag_routes())
        .nest("/opds", opds_routes())
}
```

The `nest` method combines routers under a common prefix. This keeps the code organized — each feature area has its own router, and they're combined at the top level.

## Middleware

Middleware is code that runs before or after every request. FicHub uses several middleware layers:

### CORS Middleware

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

CORS (Cross-Origin Resource Sharing) controls which websites can make requests to your API. The `allow_origin(Any)` setting allows requests from any origin — appropriate for a public API.

### Trace Middleware

```rust
use tower_http::trace::TraceLayer;

let trace = TraceLayer::new_for_http();
```

The trace middleware logs every request and response. It's essential for debugging and monitoring.

### Compression Middleware

```rust
use tower_http::compression::CompressionLayer;

let compression = CompressionLayer::new();
```

Compression middleware automatically compresses responses using gzip or brotli. This reduces bandwidth usage and improves response times for large responses.

## The Complete FicHub Router

Here's how FicHub builds its complete router:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
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
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        // Tag routes
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        // Curator routes
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        // Search route
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        // OPDS catalog routes
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
        
        // Legacy redirect routes
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        // Static frontend
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
}
```

## The AppState Construction

The `AppState` is constructed in the `run` function:

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine: RecommendationEngine::new(db_pool.clone()),
        collection_worker: CollectionWorker::new(
            db_pool.clone(),
            redis_conn.clone(),
            http_client.clone(),
            config.clone(),
            scraper_registry.clone(),
        ),
    });
    
    // Build and run server
    let app = build_router(state).await;
    let addr = format!("0.0.0.0:{}", config.app_port);
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

## Handler Patterns

Let's look at a real handler from FicHub — the export handler:

```rust
#[derive(Deserialize)]
pub struct ExportQuery {
    pub q: String,  // The URL to export
}

pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
    
    // 1. Find the right scraper
    let scraper = state.scraper_registry.find_scraper(&query.q)
        .ok_or_else(|| AppError::BadRequest(-1, "unsupported URL".into()))?;
    
    // 2. Rate limit check
    let client_ip = remote_addr.ip();
    match state.rate_limiter.check(client_ip, &query.q).await? {
        RateLimitResult::Allowed => {},
        RateLimitResult::Wait(secs) => return Err(AppError::RateLimited(secs)),
        RateLimitResult::Blocked => return Err(AppError::BadRequest(-403, "blocked".into())),
    }
    
    // 3. Lookup metadata
    let meta = scraper.lookup(&state.http_client, &query.q).await?;
    
    // 4. Upsert in database
    let fic_info = FicInfo::from_metadata(&meta);
    queries::upsert_fic_info(&state.db, &fic_info).await?;
    
    // 5. Check blacklist
    if !queries::check_fic_blacklist(&state.db, &meta.url_id).await?.is_empty() {
        return Err(AppError::BadRequest(-403, "story is blacklisted".into()));
    }
    
    // 6. Check cache
    let input_hash = compute_input_hash(&meta);
    if let Some(cached) = queries::find_export_log(&state.db, &meta.url_id, 1, "epub", &input_hash).await? {
        let download_url = format!("/cache/epub/{}?h={}", meta.url_id, cached.export_hash);
        return Ok(Json(json!({
            "err": 0,
            "url_id": meta.url_id,
            "meta": meta,
            "urls": {"epub": download_url}
        })));
    }
    
    // 7. Generate export
    let export_start = std::time::Instant::now();
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
    let (epub_path, epub_hash) = export::epub::generate_epub(&state.config.tmp_dir, &meta, &chapters)?;
    let export_ms = export_start.elapsed().as_millis() as i32;
    
    // 8. Move to cache
    let cache_path = cache::disk::cache_path(
        &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash
    );
    fs::rename(&epub_path, &cache_path)?;
    
    // 9. Record in export_log
    queries::insert_export_log(&state.db, &meta.url_id, 1, "epub", &input_hash, &epub_hash).await?;
    
    // 10. Log request
    let total_ms = start.elapsed().as_millis() as i32;
    queries::insert_request_log(
        &state.db, 1, "epub", &query.q, total_ms - export_ms,
        Some(&meta.url_id), None, Some(export_ms), None, Some(&epub_hash), None,
    ).await?;
    
    let download_url = format!("/cache/epub/{}?h={}", meta.url_id, epub_hash);
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "meta": meta,
        "urls": {"epub": download_url}
    })))
}
```

This handler demonstrates the complete request lifecycle:
1. Extract and validate input
2. Rate limit check
3. Scrape metadata
4. Database operations
5. Cache check
6. Export generation
7. File storage
8. Logging
9. Response

Each step is focused on one thing, and errors are propagated with the `?` operator.

## 📝 Practice Exercises

1. **Route Nesting:** Create a router with nested routes: `/api/v1/users`, `/api/v1/users/{id}`, `/api/v1/users/{id}/posts`. Use `nest` to organize the routes.

2. **Middleware:** Write a middleware that logs the request method, URL, and response time for every request.

3. **State Extraction:** Create a middleware that extracts the client's IP address from the `X-Forwarded-For` header (if present) or the connection info.

4. **Error Recovery:** Write a middleware that catches panics in handlers and returns a 500 error instead of crashing the server.

5. **Route Guards:** Implement a simple authentication middleware that checks for an API key in the `Authorization` header.

