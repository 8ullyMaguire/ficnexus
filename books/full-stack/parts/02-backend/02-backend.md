
# Part 2: The Rust Backend

---

*In Part 1, we set up our tools — Rust, Node.js, PostgreSQL, Redis — and ran our first Hello World. Now we build the real thing. Welcome to the engine room.*

*This part takes you from an empty project to a fully functional backend with database, configuration, error handling, and all the routes wired up. By the end, you'll understand every line of the Rust code that powers FicHub.*

*Let's fire up the editor and get cooking.* 🚀

---

# Chapter 5: Your First Rust Web Server (Axum)

## Why Axum?

There are several web frameworks for Rust — Actix Web, Rocket, Warp, and Axum among them. We chose Axum for FicHub because:

1. **It's built by the Tokio team** — the same people who built the async runtime. This means deep integration and fewer surprises.
2. **It's minimal and composable** — Axum doesn't force opinions on you. It gives you routing, extractors, and middleware, and lets you choose the rest.
3. **It's type-safe** — if you extract a parameter that doesn't exist, your code won't compile. Errors are caught at compile time, not in production.
4. **It's well-maintained** — active development, good documentation, and a growing ecosystem of middleware and extensions.
5. **It's fast** — Axum consistently ranks among the fastest Rust web frameworks in benchmarks.

For a self-hosted server that needs to handle concurrent requests, serve static files, connect to databases, and expose a clean API, Axum is an excellent choice.

## What Is Axum? (A Restaurant Analogy)

Imagine you're opening a restaurant. You need:

- **A building** — a place for customers to walk into (that's your web server)
- **A menu** — a list of what you serve (that's your routes)
- **Waiters** — people who take orders and bring food (those are your handler functions)
- **A kitchen** — where the actual cooking happens (that's your database, your cache, your scraping logic)
- **A reservation system** — so multiple customers can eat at the same time (that's the async runtime)

**Axum** is the whole restaurant package. It gives you the building, the menus, and the waiters. You just need to write the kitchen logic — the parts that actually *do* something.

Unlike some web frameworks that try to do everything (cook the food, wash the dishes, AND park the cars), Axum is focused on one thing: routing HTTP requests to the right handler function. It's like a small, efficient restaurant where every waiter knows exactly what to do and never gets in each other's way.

Here's what makes Axum special:

1. **It's built on Tokio** — the async runtime we learned about in Part 1. This means it can handle thousands of simultaneous requests without breaking a sweat.
2. **It's type-safe** — if you try to extract a parameter that doesn't exist, your code won't compile. The compiler catches mistakes before they reach production.
3. **It's composable** — middleware, extractors, and handlers plug together like LEGO bricks.
4. **It's minimal** — Axum doesn't have its own template engine, ORM, or session management. It expects you to use the best library for each job (SQLx for databases, Tera for templates, etc.).

The Axum framework was created by the Tokio team — the same people who built the async runtime underneath it. This means it's deeply integrated with the async ecosystem, and it handles edge cases that other frameworks might miss.

## Cargo.toml: Adding Dependencies

Let's look at our `Cargo.toml` to see what goes into building this restaurant:

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
description = "Self-hosted fanfiction download server (fichub.net replacement)"

[dependencies]
# Web framework & runtime
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }

# Database
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "migrate", "derive"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Config
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

Let's break this down line by line:

| Dependency | What It Does | Restaurant Analogy |
|------------|-------------|-------------------|
| `axum` | Web framework — routes, handlers, state | The restaurant building and staff |
| `tokio` | Async runtime — lets Rust do many things at once | The energy that powers everything |
| `tower` | Middleware — layers that wrap around handlers | The security guard at the door |
| `tower-http` | HTTP-specific middleware (CORS, logging, static files) | Extra security and comfort features |
| `sqlx` | PostgreSQL database driver | The filing cabinet where we store recipes |
| `redis` | Redis client for caching | The memory foam mattress — instant recall |
| `serde` | Serialize/deserialize data structures | The universal translator |
| `serde_json` | JSON-specific serialization | The JSON translation module |
| `dotenvy` | Load `.env` files | The settings clipboard |
| `tracing` | Structured logging | The security camera system |
| `tracing-subscriber` | Formatting and filtering logs | The camera monitor |

The `features` part is important — it's like ordering specific dishes rather than the whole menu:

- **`tokio` with `features = ["full"]`** means "give me everything." This includes timers, TCP/UDP networking, file I/O, channels, and synchronization primitives. You need the full feature set when building a web server.

- **`sqlx` features** are more selective:
  - `"runtime-tokio"` — Use Tokio as the async runtime
  - `"postgres"` — Include the PostgreSQL driver (we could also use MySQL or SQLite)
  - `"migrate"` — Migration support for database schema changes
  - `"derive"` — The `#[derive(FromRow)]` macro that auto-maps database rows to structs

- **`tower-http` features** — Each feature adds specific middleware:
  - `"cors"` — Cross-Origin Resource Sharing headers
  - `"compression-gzip"` — Gzip compression for responses
  - `"trace"` — Request/response logging
  - `"fs"` — Static file serving

You might wonder why we have both `tower` and `tower-http`. Tower is the general middleware framework; tower-http adds HTTP-specific middleware on top of it. Think of Tower as the security system and tower-http as the specific cameras and locks.

## The Main Function: `#[tokio::main]`

Here's the entry point of our entire application — the `main.rs` file. Every Rust program starts here:

```rust
// src/main.rs
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

Let's walk through this step by step, because every line does something important.

**The `pub mod` lines** at the top are like doors in a hallway. Each one opens into a different room of our application:

```rust
pub mod db;        // The database room
pub mod config;    // The settings room
pub mod routes;    // The handler room (where API endpoints live)
pub mod error;     // The error room
pub mod server;    // The server room
pub mod cache;     // The caching room
pub mod scrape;    // The web scraping room
pub mod export;    // The file generation room
pub mod frontend;  // The frontend serving room
pub mod limiter;   // The rate limiting room
pub mod recommender; // The recommendation engine room
pub mod search;    // The search room
pub mod tags;      // The tagging system room
```

The `pub` means these modules are public — other parts of the code can use them. If you left off `pub`, only code inside `main.rs` could access them. By making them `pub`, the entire application can import from any module.

**The `#[tokio::main]` attribute** is where the magic happens. When you write `async fn main()`, Rust needs someone to manage the async operations. Tokio volunteers to be that manager. It sets up:

1. A **thread pool** — multiple threads that can run tasks simultaneously
2. An **event loop** — a mechanism that watches for things to happen (data arriving on a socket, a timer firing, a file finishing reading)
3. A **task scheduler** — decides which async tasks to run when

```rust
#[tokio::main]  // ← This macro transforms your main function
async fn main() {
    // Tokio is now in charge of managing async tasks
    // You can use .await freely here
}
```

Think of `#[tokio::main]` as hiring a restaurant manager. Without it, the waiters (async tasks) would have no one coordinating them — they'd all try to serve the same table at once, or stand around waiting for orders that never come.

**The `.ok()` on `dotenvy::dotenv()`** means "try to load the `.env` file, but don't panic if it's missing." In Rust, `.ok()` converts a `Result<T, E>` into an `Option<T>` — if it was `Ok(value)`, you get `Some(value)`; if it was `Err(e)`, you get `None`. By calling `.ok()` and then discarding the result, we say "I don't care if this fails."

This is important because in production, you might not have a `.env` file — environment variables might be set directly in Docker, systemd, or your hosting platform.

**The `tracing_subscriber` block** sets up our logging system. Think of it as installing the security cameras:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
    )
    .init();
```

The `EnvFilter` reads the `RUST_LOG` environment variable to decide what to log. If `RUST_LOG` is not set, it defaults to `info,fichub=debug` — this means:
- Log messages at info level and above for everything
- Log messages at debug level and above for our `fichub` crate

If you wanted to see EVERYTHING (including debug messages from all libraries), you could set `RUST_LOG=debug` in your `.env` file. If you wanted to silence everything except errors, you'd set `RUST_LOG=error`.

**Finally**, `server::run(config).await` hands everything off to the server module. This is where the actual work begins. The `config` variable holds all our settings, and `.await` means "wait for the server to finish" (which in practice means "run forever").

🧪 **Try It Yourself**: Create a minimal `main.rs` that just prints "Hello from Tokio!" and runs a timer. Use `#[tokio::main]` and `tokio::time::sleep`. Example:

```rust
#[tokio::main]
async fn main() {
    println!("Hello from Tokio!");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("One second later!");
}
```

Run it with `cargo run` and watch the delay. That delay is Tokio's async runtime in action!

## Creating a Router with Routes

Now let's look at the heart of our application — the router. The router is like a switchboard operator — it looks at each incoming request and figures out which handler function should deal with it.

Here's a simplified version first:

```rust
use axum::{routing::get, Router};

// A simple handler function
async fn hello() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() {
    // Create a router with one route
    let app = Router::new()
        .route("/", get(hello));

    // Bind to a port and serve
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

The `Router::new()` creates an empty restaurant. `.route("/", get(hello))` adds a menu item: "When someone visits `/`, run the `hello` function." The `get` part means this only works for GET requests — if someone sends a POST to `/`, they'll get a 405 "Method Not Allowed" response.

You can add multiple routes:

```rust
let app = Router::new()
    .route("/", get(hello))
    .route("/about", get(about_page))
    .route("/api/data", get(get_data).post(create_data));
```

The `.route("/api/data", get(get_data).post(create_data))` line shows how to handle multiple HTTP methods on the same path. GET requests go to `get_data`, POST requests go to `create_data`.

### Understanding the `json!()` Macro

The `json!()` macro is one of the most useful tools in `serde_json`. It lets you build JSON values using Rust-like syntax:

```rust
use serde_json::json;

// Simple values
let j = json!("hello");           // "hello"
let j = json!(42);                // 42
let j = json!(true);              // true
let j = json!(null);              // null

// Objects
let j = json!({
    "name": "Alice",
    "age": 30,
    "active": true
});
// {"name": "Alice", "age": 30, "active": true}

// Arrays
let j = json!([1, 2, 3]);
// [1, 2, 3]

// Nested
let j = json!({
    "user": {
        "name": "Bob",
        "scores": [100, 95, 87]
    }
});

// Interpolating Rust variables
let title = "My Story";
let wordcount = 5000;
let j = json!({
    "title": title,
    "wordcount": wordcount,
    "status": "Complete"
});
// {"title": "My Story", "wordcount": 5000, "status": "Complete"}
```

The `json!()` macro is powerful because it:
- Handles escaping automatically (strings with quotes, newlines, etc.)
- Works with any Rust value that implements `Serialize`
- Produces a `serde_json::Value` that Axum can send as a response
- Is checked at compile time for syntax errors

### The `Json<T>` Extractor and Response

When a handler returns `Json<Value>`, Axum:
1. Serializes the value to JSON
2. Sets the `Content-Type: application/json` header
3. Sends the JSON body

When a handler takes `Json<T>` as a parameter, Axum:
1. Reads the request body
2. Parses it as JSON
3. Deserializes it into type `T`
4. If parsing fails, returns a 400 error automatically

```rust
// Sending JSON
async fn handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

// Receiving JSON
async fn create_fic(
    Json(payload): Json<CreateFicRequest>,
) -> Json<Value> {
    // payload is already parsed and type-checked
    Json(json!({"id": payload.id, "created": true}))
}
```

## GET /api/v0/epub Handler Returning JSON

Now let's look at a real handler from our codebase — the epub handler. This is the handler that powers the main API endpoint. Before we read it, let's understand the `serde` derive macros that make it work:

### The `serde` Derive Macros

Serde is Rust's serialization framework. It converts Rust data structures to and from other formats (JSON, YAML, TOML, etc.). The `#[derive(Serialize, Deserialize)]` macros do the heavy lifting:

```rust
use serde::{Deserialize, Serialize};

// This struct can be converted TO JSON (Serialize)
// and FROM JSON (Deserialize)
#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: Option<String>,
}

// Serialize: Rust struct → JSON
let user = User {
    name: "Alice".to_string(),
    age: 30,
    email: None,
};
let json = serde_json::to_string(&user).unwrap();
// {"name":"Alice","age":30,"email":null}

// Deserialize: JSON → Rust struct
let json = r#"{"name":"Bob","age":25}"#;
let user: User = serde_json::from_str(json).unwrap();
// User { name: "Bob", age: 25, email: None }
```

The `Option<String>` for `email` means it can be `null` in JSON. If the JSON doesn't include `email`, it becomes `None` in Rust.

### Query Parameter Structs

When you use `Query(params): Query<ExportQuery>`, Axum deserializes the URL query string into your struct:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,        // ?q=... → Some("...") or None
    pub automated: Option<String>, // ?automated=true → Some("true")
    pub format: Option<String>,    // ?format=epub → Some("epub")
}
```

Every field must be `Option<T>` for optional parameters. If someone visits `/api/v0/epub` without any query parameters, all fields are `None`. If they visit `?q=test&format=epub`, `q` is `Some("test")` and `format` is `Some("epub")`.

Axum handles the parsing automatically. If the query string is malformed (like `?q=hello&q=hello`), Axum returns a 400 error. You never have to parse query strings manually.

```rust
// src/routes/export.rs (simplified)

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,      // The URL to look up
    pub automated: Option<String>, // Whether it's a bot
    pub format: Option<String>,    // Export format
}

/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // Get the query parameter (URL to look up)
    let query = params.q.as_deref().unwrap_or("");

    if query.is_empty() {
        return Ok(Json(json!({
            "err": -1,
            "msg": "no query",
            "q": ""
        })));
    }

    // Find the right scraper for this URL
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(
            -5,
            format!("unsupported URL: {}", query)
        ))?;

    // Look up metadata from the source site
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Return the metadata as JSON
    Ok(Json(json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "wordcount": meta.words,
        "chapters": meta.chapters,
        "description": meta.desc,
    })))
}
```

Several important things to notice here:

**1. The function takes `State` as a parameter.** This is how Axum gives your handler access to shared data. The handler needs the database pool, the HTTP client, the config — all of that lives in `AppState`. We extract it with `State(state)`.

**2. The function takes `Query` as a parameter.** This tells Axum to parse the URL query string into our `ExportQuery` struct. If the parsing fails (like if someone sends `?q=hello&q=hello` with duplicate keys), Axum automatically returns a 400 error. You don't have to handle that case!

**3. The return type is `Result<Json<Value>, AppError>`.** This means the handler can either succeed (return JSON) or fail (return an AppError). The `?` operator handles the conversion automatically.

**4. `json!({...})`** is a macro from `serde_json` that creates JSON values. It's like building a dictionary, but it produces valid JSON. The syntax looks like Rust's struct initialization but produces a `serde_json::Value` at runtime.

**5. The `as_deref().unwrap_or("")` pattern.** `params.q` is an `Option<String>`. `as_deref()` converts it to `Option<&str>` (a reference to the string, without copying). `unwrap_or("")` provides a default empty string. This is a common Rust pattern for handling optional strings.

## Axum Extractors: The Magic Behind Handler Parameters

Before we dive into path and query parameters, let's understand **extractors** — the mechanism that makes Axum so elegant. An extractor is any type that implements the `FromRequestParts` or `FromRequest` trait. When you add a parameter to a handler, Axum tries to extract it from the request.

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,      // Extractor 1: shared state
    Query(params): Query<MyQuery>,            // Extractor 2: query string
    Path(id): Path<String>,                   // Extractor 3: path parameter
    Json(body): Json<MyBody>,                 // Extractor 4: request body
) -> impl IntoResponse {
    // All four values are extracted automatically
}
```

Axum processes extractors in order. If any extractor fails (bad query string, missing path segment, invalid JSON), the handler is never called — Axum returns an error response instead. This means your handler code can assume all data is valid.

The most common extractors:

| Extractor | What It Extracts | Example |
|-----------|-----------------|---------|
| `State(s)` | Shared application state | Database pool, config |
| `Query(q)` | URL query parameters | `?key=value` |
| `Path(p)` | URL path segments | `/users/{id}` |
| `Json(body)` | JSON request body | POST data |
| `ConnectInfo(addr)` | Client's IP address | Remote socket address |

You can use multiple extractors in any order — Axum figures out how to combine them. The only rule is that `Json<T>` must be the last extractor because it consumes the request body.

🧪 **Try It Yourself**: Write a handler that uses three extractors: `State`, `Query`, and `Path`. What happens if you send a request with a malformed query string? What if the path doesn't match?

## Path Parameters and Query Parameters

Axum has two main ways to get data from URLs:

### Query Parameters

These come after the `?` in a URL:
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345
```

You define a struct with `#[derive(Deserialize)]` and use `Query(params)`:

```rust
use serde::Deserialize;
use axum::extract::Query;

#[derive(Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,    // "q" from the URL
    pub format: Option<String>, // "format" from the URL
}

pub async fn epub_handler(
    Query(params): Query<ExportQuery>,
) -> ... {
    let url = params.q.unwrap_or_default();
    let format = params.format.unwrap_or_else(|| "epub".to_string());
}
```

Every field in the struct must be `Option<T>` if it might not be present in the URL. If someone visits `/api/v0/epub` without any query parameters, `params.q` will be `None`.

You can also make fields required by not using `Option`:

```rust
#[derive(Deserialize)]
pub struct RequiredQuery {
    pub q: String,  // REQUIRED — missing = 400 error
}
```

### Path Parameters

These are parts of the URL path itself:
```
GET /cache/epub/abc123/myfile.epub
```

You define them with `{name}` in the route and use `Path(...)`:

```rust
use axum::extract::Path;

pub async fn download_handler(
    Path((etype, url_id, fname)): Path<(String, String, String)>,
) -> ... {
    // etype = "epub", url_id = "abc123", fname = "myfile.epub"
}
```

The type annotation `Path<(String, String, String)>` tells Axum "I expect three path segments, and they should all be strings." If someone visits `/cache/epub/abc123` with only two segments, they get a 404.

You can also use structs for path parameters:

```rust
#[derive(Deserialize)]
struct DownloadParams {
    etype: String,
    url_id: String,
    fname: String,
}

async fn download_handler(
    Path(params): Path<DownloadParams>,
) -> ... {
    // params.etype, params.url_id, params.fname
}
```

The tuple version is more concise; the struct version is more readable. Pick whichever makes sense for your use case.

## State: Sharing Data Across Routes

In a real application, many routes need access to the same resources — the database pool, the HTTP client, the config. You don't want to create a new database connection for every request. That would be like hiring a new waiter for every customer — expensive and slow!

This is where **State** comes in. State is a single value that all handlers share:

```rust
use std::sync::Arc;

// Define what state looks like
struct AppState {
    db: sqlx::PgPool,           // Database connection pool
    redis: redis::aio::MultiplexedConnection,  // Redis connection
    http_client: reqwest::Client,               // HTTP client
    config: Config,                             // All settings
}

#[tokio::main]
async fn main() {
    // Create the state ONCE, at startup
    let state = Arc::new(AppState {
        db: db_pool,
        redis: redis_conn,
        http_client,
        config,
    });

    // Give it to the router
    let app = Router::new()
        .route("/api/v0/epub", get(epub_handler))
        .with_state(state);  // ← State is attached here!

    // Every handler can now access it via State(state)
}
```

The `Arc` (Atomic Reference Count) is Rust's way of sharing ownership safely. Multiple handlers can all have a reference to the same `AppState` without worrying about who "owns" it. When the last reference is dropped, the state is cleaned up. Think of it as giving every waiter a key to the same kitchen — they can all use it, and when the restaurant closes, the key is returned.

The `Arc` is needed because Axum requires state to be cloneable and shareable across threads. Without it, you'd have ownership conflicts — Rust's borrow checker would prevent you from passing the same state to multiple handlers.

## The AppState Struct

Here's our actual `AppState` from `server.rs`:

```rust
/// Shared application state accessible by all handlers
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

This is the "kitchen" of our restaurant. Every piece of state is something that multiple handlers might need:

| Field | Type | Purpose | Created Once |
|-------|------|---------|-------------|
| `config` | `Config` | All our settings (port, database URL, etc.) | ✓ |
| `db` | `sqlx::PgPool` | The database connection pool | ✓ |
| `redis` | `MultiplexedConnection` | Redis connection for caching | ✓ |
| `http_client` | `reqwest::Client` | For fetching web pages | ✓ |
| `scraper_registry` | `Arc<ScraperRegistry>` | Which sites we know how to scrape | ✓ |
| `cache_semaphores` | `CacheSemaphores` | Prevents duplicate exports | ✓ |
| `rate_limiter` | `Box<dyn RateLimiter>` | Limits how fast users can hit us | ✓ |
| `recommender_engine` | `RecommendationEngine` | The recommendation system | ✓ |
| `collection_worker` | `CollectionWorker` | Background data collection | ✓ |

The "Created Once" column is key — all of these are created at startup and shared across all requests. Creating them on every request would be incredibly wasteful.

The `Box<dyn limiter::RateLimiter>` is a **trait object** — it means the rate limiter can be any type that implements the `RateLimiter` trait. This lets us swap implementations (like switching from a Redis-based limiter to an in-memory one for testing) without changing the rest of the code.

## Running the Server and Testing with curl

Let's see how the server starts up. The `run()` function in `server.rs` does all the heavy lifting:

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

    // 4. Create shared state
    let state = Arc::new(AppState { /* ... */ });

    // 5. Build router
    let app = build_router(state).await;

    // 6. Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server error");
}
```

Notice the order: database first, then Redis, then HTTP client, then state, then router, then serve. Each step can fail, and we use `.expect()` to crash with a clear message if anything goes wrong. In production, you might want to handle these errors more gracefully (retry, log, send alerts), but for development, crashing loudly is the right choice — it tells you immediately what's broken.

The `reqwest::Client::builder()` creates a reusable HTTP client with custom settings. The `.user_agent("fichub.net/0.1.0")` sets a custom User-Agent header so web servers know who's making the request. The `.timeout(Duration::from_secs(30))` means we give up on any HTTP request that takes longer than 30 seconds.

Now let's test it! Start your server in one terminal:

```bash
cd ~/code/rust/fichub
cargo run
```

You should see output like:
```
INFO Starting fichub-rs server on port 3000
INFO Listening on 0.0.0.0:3000
```

In another terminal, use `curl` to talk to it:

```bash
# See the API docs (self-documenting!)
curl http://localhost:3000/api/

# Get metadata for a fanfic
curl "http://localhost:3000/api/v0/meta?q=https://archiveofourown.org/works/12345"

# Check remote info (who am I?)
curl http://localhost:3000/api/v0/remote

# Search for fanfics
curl "http://localhost:3000/api/v0/search?q=harry+potter"

# Try an error — empty query
curl "http://localhost:3000/api/v0/epub?q="
```

The `curl` command is your best friend for testing APIs. It sends HTTP requests from the command line and shows you the response. The `-v` flag (verbose) shows you all the headers and details:

```bash
curl -v "http://localhost:3000/api/v0/meta?q=test"
```

🧪 **Try It Yourself**: Start the server and hit each endpoint with curl. What do the JSON responses look like? Can you make the server return an error by sending an empty query? What happens if you try to access a route that doesn't exist?

## What Happens When a Request Arrives (The Flow)

Let's trace exactly what happens when someone visits `GET /api/v0/meta?q=some-url`. This is important to understand because it shows how all the pieces fit together:

```
Step 1: Browser sends: GET /api/v0/meta?q=some-url HTTP/1.1
        ↓
Step 2: Tokio receives the TCP connection on port 3000
        ↓
Step 3: Axum parses the HTTP request into parts:
        - Method: GET
        - Path: /api/v0/meta
        - Query: q=some-url
        ↓
Step 4: Router matches: /api/v0/meta → meta_handler
        ↓
Step 5: TraceLayer logs: "GET /api/v0/meta → 200 OK (45ms)"
        ↓
Step 6: Axum extracts from the request:
        - State(state) → from the router's shared state
        - Query(params) → parsed from "?q=some-url"
        ↓
Step 7: Axum calls meta_handler(state, params)
        ↓
Step 8: Handler uses state.http_client to fetch the web page
        (this is an async operation — other requests can proceed)
        ↓
Step 9: Handler uses state.db to store/retrieve data
        (another async operation)
        ↓
Step 10: Handler returns Ok(Json({...}))
        ↓
Step 11: Axum serializes the JSON and sends HTTP 200 OK
         Content-Type: application/json
        ↓
Step 12: Browser receives the response and renders it
```

This whole journey takes about 50-200 milliseconds, depending on network speed and database performance. And because of Tokio's async runtime, the server can handle thousands of these simultaneously — while one handler waits for a database query, other handlers are free to do their work.

The key insight is that **async doesn't mean parallel**. It means the server can do other work while waiting. When handler A calls `.await` on a database query, Tokio pauses handler A and runs handler B. When the database responds, Tokio resumes handler A. It's like a chef who starts cooking one dish, puts it in the oven, and while it bakes, starts preparing the next dish.

⚠️ **Watch Out**: If your handler blocks (does slow synchronous work), it blocks all other handlers too. This is called "blocking the runtime." Always use `.await` for async operations and avoid `.unwrap()` on network calls that might hang. If you need to do CPU-intensive work, use `tokio::task::spawn_blocking` to move it to a separate thread.

## Building a Complete Simple Server

Let's put everything together into a complete, runnable example. This isn't our full FicHub server — it's a simplified version that demonstrates all the key concepts:

```rust
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

// Our shared state
struct AppState {
    visitor_count: std::sync::atomic::AtomicU64,
}

// A query parameter struct
#[derive(Deserialize)]
struct GreetQuery {
    name: Option<String>,
}

// Handler 1: Root route
async fn index() -> Json<Value> {
    Json(json!({
        "message": "Welcome to our server!",
        "endpoints": ["/greet", "/hello/{name}", "/count"]
    }))
}

// Handler 2: Greet with query parameter
async fn greet(
    Query(params): Query<GreetQuery>,
) -> Json<Value> {
    let name = params.name.unwrap_or_else(|| "stranger".to_string());
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

// Handler 3: Greet with path parameter
async fn hello_name(
    Path(name): Path<String>,
) -> Json<Value> {
    Json(json!({
        "message": format!("Hello, {}!", name)
    }))
}

// Handler 4: Uses shared state
async fn count(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let count = state.visitor_count
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(json!({
        "visitor_number": count + 1,
        "message": format!("You are visitor #{}!", count + 1)
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        visitor_count: std::sync::atomic::AtomicU64::new(0),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/greet", get(greet))
        .route("/hello/{name}", get(hello_name))
        .route("/count", get(count))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}
```

Try running this and testing all the endpoints:

```bash
# Welcome message
curl http://localhost:3000/

# Greet with default name
curl http://localhost:3000/greet

# Greet with query parameter
curl "http://localhost:3000/greet?name=Alice"

# Greet with path parameter
curl http://localhost:3000/hello/Bob

# Visitor counter (run multiple times!)
curl http://localhost:3000/count
curl http://localhost:3000/count
curl http://localhost:3000/count
```

The visitor counter demonstrates shared state — the `AtomicU64` counter persists across requests because it lives in `AppState`, which is shared via `Arc`. Without state, each request would start with a fresh counter.

🧪 **Try It Yourself**: Modify the server above to add a `/time` endpoint that returns the current time as JSON. Hint: use `chrono::Utc::now()`.

---

# Chapter 6: Configuration and Error Handling

## Environment Variables: Settings for Your App

Every application needs settings. Where's the database? What port should we listen on? What's the Redis URL? These are things that change between your laptop, your test server, and your production server.

You *could* hardcode them:

```rust
let database_url = "postgres://user:password@localhost:5432/fichub";
```

But that's terrible for three reasons:

1. **You'd have to change the source code** every time you move to a different server
2. **You'd accidentally commit your password** to git (and everyone would see it)
3. **You can't have different settings** for development, testing, and production

Instead, we use **environment variables** — settings that live outside your code, in the environment where the program runs:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

This reads the `DATABASE_URL` environment variable. If it's not set, the program crashes with a clear error message: "DATABASE_URL must be set."

Environment variables are key-value pairs in your shell:

```bash
# Linux/Mac
export DATABASE_URL=postgres://user:password@localhost:5432/fichub

# Windows
set DATABASE_URL=postgres://user:password@localhost:5432/fichub
```

They're set before running your program and last until the terminal closes. Different terminals can have different values.

## The .env File: Keeping Secrets Safe

Setting environment variables every time you run the server is annoying. Imagine typing this every morning:

```bash
export DATABASE_URL=postgres://...
export REDIS_URL=redis://...
export CACHE_DIR=./cache
export PORT=3000
# ... 20 more variables ...
cargo run
```

That's where `.env` files come in. Create a file called `.env` in your project root:

```bash
# .env file in the project root
DATABASE_URL=postgres://user:password@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
PORT=3000
```

We load it at the very start of `main.rs`:

```rust
dotenvy::dotenv().ok();
```

The `dotenvy` crate reads the `.env` file and sets each variable in the environment — as if you had typed `export` for each one. The `.ok()` means "don't panic if the file doesn't exist."

Now when you run `cargo run`, the `.env` file is automatically loaded. No typing required!

⚠️ **Watch Out**: Never commit your `.env` file to git! It contains passwords and secrets. Add it to `.gitignore`:

```bash
echo ".env" >> .gitignore
```

In production, you'd set these variables directly (in Docker, systemd, or your hosting platform) rather than using a `.env` file. The `.env` file is just for development convenience.

You can also have different `.env` files for different environments:

```
.env              # Default (used when no other file is specified)
.env.development  # Development settings
.env.production   # Production settings
.env.testing      # Test settings
```

## Config Struct: Loading All Settings at Once

Instead of scattering `std::env::var()` calls throughout your code (which makes it hard to know what settings exist and what they do), collect everything into a single `Config` struct.

Here's our actual `Config` from `config.rs`:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration loaded from environment variables
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
    pub rec_max_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    // ... more fields ...
}
```

The `#[derive(Debug, Clone)]` is two derive macros in one line:
- `Debug` lets you print the struct with `{:?}` for debugging — essential when things go wrong
- `Clone` lets you copy the struct (Config is cheap to clone since it's mostly strings and numbers)

Notice the variety of types:
- `String` — for text values like URLs
- `PathBuf` — for file paths (like `./cache`)
- `i32`, `u16`, `u32`, `u64` — for numbers of different sizes
- `bool` — for true/false settings
- `Vec<String>` — for lists (like trusted proxy IPs)
- `Option<PathBuf>` — for optional settings
- `HashMap<String, u64>` — for key-value maps (like per-site rate limits)

Each type has a purpose. `PathBuf` is better than `String` for file paths because it handles OS-specific separators automatically. `u16` is the right size for a port number (0-65535).

## Config::from_env() Function Explained

The magic happens in the `from_env()` method. Let's read it carefully:

```rust
impl Config {
    pub fn from_env() -> Self {
        // REQUIRED: crash immediately if these are missing
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        // OPTIONAL with defaults: use fallback if missing
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());

        // OPTIONAL: might not be set
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);

        // ... load everything else ...

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            node_name,
            secondary_cache_dir,
            // ... other fields ...
        }
    }
}
```

There are three patterns here, and they're important:

**Pattern 1: `expect()` — Required variables**
```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```
If `DATABASE_URL` isn't set, this crashes with a clear message. Use this for things the app can't function without. The server would crash immediately on startup — and that's the right behavior! It's better to crash loudly than to silently fail later.

**Pattern 2: `unwrap_or_else()` — Optional with defaults**
```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());
```
If `CACHE_DIR` isn't set, use `"./cache"`. The server continues normally. Use this for settings that have reasonable defaults.

**Pattern 3: Chain with `.parse()` — String to number conversion**
```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())  // String
    .parse()                                   // Result<u16>
    .unwrap_or(3000);                          // u16
```
Let's trace through this chain:
```
std::env::var("PORT")                    → Result<String, VarError>
    .unwrap_or_else(|_| "3000".to_string()) → String (either from env or "3000")
    .parse()                               → Result<u16, ParseIntError>
    .unwrap_or(3000)                        → u16 (either parsed or 3000)
```

This chain is **safe** — it can never fail. If `PORT` isn't set, or if it's set to something that isn't a valid port number (like "banana"), we get 3000. This is defensive programming at its finest.

**Pattern 4: Optional with `.ok()` and `.filter()`**
```rust
let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
    .ok()                                    // Option<String>
    .filter(|s| !s.is_empty())              // Option<String> (None if empty)
    .map(PathBuf::from);                     // Option<PathBuf>
```
The `.ok()` converts `Result` to `Option`. The `.filter()` removes empty strings. The `.map()` converts the type. This chain handles three cases:
- Variable not set → `None`
- Variable set to empty string → `None`
- Variable set to a real path → `Some(PathBuf)`

🧪 **Try It Yourself**: Create a `.env` file with `DATABASE_URL`, `REDIS_URL`, and `PORT=8080`. Run the server and verify it starts on port 8080. Then change the port to `banana` and see what happens (it should default to 3000).

## Making Nice Error Pages

When something goes wrong, we don't want to dump a stack trace on the user. We don't want to show them our database password or the internal structure of our code. We want a nice, structured JSON error that our frontend can display nicely.

Here's what a successful response looks like:
```json
{
  "id": "12345",
  "title": "A Great Fanfic",
  "author": "Some Author",
  "wordcount": 15000
}
```

And here's what an error looks like:
```json
{
  "err": -5,
  "msg": "unsupported URL: not-a-real-site.com"
}
```

The `err` field is always a negative number that tells the frontend what kind of error occurred. The `msg` field is a human-readable description. This consistent format means our frontend can always handle errors the same way.

Why negative numbers? Because positive numbers could be confused with HTTP status codes (200, 404, 500, etc.). By using negatives, we make it clear these are application-specific codes, not HTTP codes.

## The AppError Enum: Different Kinds of Errors

Our `error.rs` file defines every possible error our app can produce:

```rust
/// Application-wide error type
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

Each variant represents a different *category* of failure. Let's use our restaurant analogy:

| Variant | HTTP Status | Meaning | Restaurant Analogy |
|---------|-------------|---------|-------------------|
| `BadRequest(code, msg)` | 400 | The user sent bad data | Customer ordered something not on the menu |
| `RateLimited(secs)` | 429 | Too many requests, wait N seconds | Customer tried to order 100 meals at once |
| `NotFound(msg)` | 404 | The resource doesn't exist | Customer asked for a dish we ran out of |
| `Internal(msg)` | 500 | Something broke on our end | The oven exploded |
| `ScrapeError(msg)` | 502 | Couldn't fetch from external site | The supplier didn't deliver |
| `ExportError(msg)` | 500 | Couldn't generate the file | The chef burned the food |
| `Database(msg)` | 500 | Database problem | The filing cabinet jammed |
| `CacheError(msg)` | 500 | Redis problem | The memory foam mattress deflated |

The `Display` implementation makes errors printable:

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(code, msg) => write!(f, "BadRequest({}): {}", code, msg),
            AppError::RateLimited(retry_after) => write!(f, "RateLimited: retry after {}s", retry_after),
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            // ... more variants ...
        }
    }
}
```

This is used when logging errors — the tracing system calls `Display` to get a human-readable message.

## IntoResponse: Converting Errors to JSON

The magic that makes errors into nice JSON is the `IntoResponse` implementation. Axum knows that if a handler returns a type that implements `IntoResponse`, it should call `into_response()` to get the HTTP response:

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

Each error variant maps to:
1. An **HTTP status code** (400, 404, 429, 500, 502) — tells the browser what happened
2. A **JSON body** with `err` and `msg` — tells our frontend how to display it

The `tracing::error!` call on `Internal` errors is crucial — it logs the *real* error message to our server logs (for debugging) but sends a *generic* message to the user (for security). You never want to leak internal details like "connection refused at 127.0.0.1:5432" to users.

## Error Codes: What -1, -5, -6, -7, -429 Mean

Our error codes are negative numbers that the frontend can use to show the right message:

| Code | HTTP Status | Meaning | What the Frontend Does |
|------|-------------|---------|----------------------|
| `-1` | 500 | Internal error | "Something went wrong on our end. Try again later." |
| `-5` | 404 / 400 | Not found or bad request | "We couldn't find that fanfic. Check the URL?" |
| `-6` | 502 | Scraper error | "We couldn't reach the source site. It might be down." |
| `-7` | 500 | Export error | "We couldn't create your download. Try again?" |
| `-10` | 400 | Automated request blocked | "Automated requests are not allowed." |
| `-429` | 429 | Rate limited | "Please slow down. Try again in {N} seconds." |
| `400` | 400 | Custom bad request | Varies by message |

The frontend can switch on these codes to show the right UI:
```javascript
if (response.err === -429) {
    showRateLimitMessage(response.retry_after);
} else if (response.err === -5) {
    showNotFoundMessage();
} else {
    showGenericError();
}
```

## Error Recovery Patterns

Sometimes you don't want an error to crash the whole request — you want to recover gracefully. Here are some patterns:

### Pattern 1: Log and Continue

```rust
// Try to log the request, but don't fail if logging breaks
if let Err(e) = queries::insert_request_log(&pool, ...).await {
    tracing::warn!("Failed to log request: {}", e);
    // Continue anyway — logging failure shouldn't break the API
}
```

The `if let Err(e)` pattern means "if this fails, do something with the error but don't propagate it."

### Pattern 2: Fallback Values

```rust
// Try to get from cache, fall back to database
let fic = cache::get(&redis, &url_id).await
    .unwrap_or_else(|_| {
        tracing::debug!("Cache miss for {}", url_id);
        None
    })
    .or_else(|| {
        // Try database as fallback
        block_on(queries::get_fic_info(&pool, &url_id)).ok().flatten()
    });
```

This tries Redis first, then the database. If both fail, `fic` is `None`.

### Pattern 3: Partial Results

```rust
// Collect results, ignoring failures
let mut results = Vec::new();
for url in urls {
    match queries::get_fic_info(&pool, &url).await {
        Ok(Some(fic)) => results.push(fic),
        Ok(None) => tracing::debug!("Not found: {}", url),
        Err(e) => tracing::warn!("Error fetching {}: {}", url, e),
    }
}
// Return whatever we got
Ok(results)
```

This collects successful results and skips failures, rather than failing the entire batch.

### Pattern 4: Retry on Transient Errors

```rust
use std::time::Duration;

async fn retry_query(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let mut last_err = None;

    for attempt in 0..3 {
        match queries::get_fic_info(pool, id).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!("Attempt {} failed: {}", attempt + 1, e);
                last_err = Some(e);
                tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
            }
        }
    }

    Err(last_err.unwrap())
}
```

This retries up to 3 times with increasing delays (100ms, 200ms, 300ms). Useful for transient network issues.

## The From Trait: Automatic Error Conversion

One of Rust's most powerful features is automatic error conversion with the `From` trait. This is what makes the `?` operator so convenient:

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

This means anywhere you use `?` in a function that returns `AppResult<T>`, the error is automatically converted:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Result<..., sqlx::Error>
    .await?;               // ← ? automatically converts sqlx::Error to AppError
    Ok(row)
}
```

Without the `From` implementations, you'd have to write this everywhere:
```rust
.await.map_err(|e| AppError::Database(e.to_string()))?;
```

With `From`, just `?` works. It's like having a universal adapter — any error type plugs into our `AppError` automatically. This is one of Rust's best features for error handling.

There's also a type alias that makes function signatures cleaner:

```rust
/// Standard API result type
pub type AppResult<T> = Result<T, AppError>;
```

Now instead of writing `Result<Option<FicInfo>, AppError>`, you write `AppResult<Option<FicInfo>>`. Shorter, clearer, and easier to read.

---

# Chapter 7: Connecting to PostgreSQL

## The Database URL

Before we connect to PostgreSQL, let's understand the connection URL format:

```
postgres://username:password@hostname:port/database_name
```

For example:
```
postgres://fichub:secretpass@localhost:5432/fichub
```

This tells SQLx:
- **Username**: `fichub`
- **Password**: `secretpass`
- **Host**: `localhost` (the same machine)
- **Port**: `5432` (PostgreSQL's default port)
- **Database**: `fichub` (the specific database to use)

You can also use environment variables in the URL:
```
postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}
```

The URL is stored in your `.env` file and read by `Config::from_env()`.

## What Is a Database? (A Giant Filing Cabinet)

Imagine you have a massive filing cabinet with thousands of folders. Each folder contains information about one fanfic — title, author, word count, when it was published. When you want to find a specific fanfic, you open the drawer, flip through the folders, and pull out the one you need.

A **database** is exactly that, except:
- It lives on a computer, not in a room
- It can hold millions of records
- It can find things in milliseconds (not hours of flipping)
- Multiple people can use it at the same time
- It keeps backups so nothing gets lost
- You can ask complex questions like "show me all fics with more than 10,000 words by authors whose names start with 'A'"

The filing cabinet we're using is called **PostgreSQL** (often shortened to "Postgres"). It's been around since 1996 and is one of the most reliable, feature-rich databases in the world. It's used by Apple, Instagram, Spotify, and thousands of other companies.

PostgreSQL speaks **SQL** (Structured Query Language) — a language specifically designed for working with databases. Here's a simple SQL query:

```sql
SELECT title, author FROM fic_info WHERE words > 10000;
```

This says "go to the fic_info table, find all rows where the words column is greater than 10,000, and return just the title and author columns."

## SQLx: A Rust Library for Talking to PostgreSQL

**SQLx** is how Rust talks to PostgreSQL. It's like a translator who speaks both "Rust" and "SQL" and can carry messages between them.

What makes SQLx special:

1. **Compile-time checked queries** — if your SQL has errors, Rust catches them when you compile. Not at runtime when a user hits the endpoint. At *compile time*, before you even deploy.

2. **Async** — it doesn't block while waiting for the database. Your server keeps handling other requests while SQLx waits for PostgreSQL to respond.

3. **Type-safe** — database rows automatically map to Rust structs. You get the benefits of Rust's type system even for database operations.

4. **Connection pooling** — built-in support for sharing connections efficiently.

We added it to our `Cargo.toml`:
```toml
sqlx = { version = "0.9", features = [
    "runtime-tokio",  # Use Tokio as the async runtime
    "postgres",       # PostgreSQL driver (could also be "mysql" or "sqlite")
    "migrate",        # Migration support
    "derive",         # Derive macros (FromRow for automatic row-to-struct mapping)
    "macros",         # Compile-time checked SQL
    "tls-rustls-ring", # TLS support for encrypted connections
]}
```

## Connection Pools: Sharing One Connection Among Many Users

Here's a problem: if you create a new database connection for every request, and you get 1,000 requests per second, you'd need 1,000 connections. PostgreSQL can handle that, but it's wasteful — each connection uses memory and takes time to set up (about 20-50 milliseconds).

Instead, we use a **connection pool** — a shared collection of connections. When a request comes in, we borrow a connection from the pool. When it's done, we return it. It's like a library checkout system — there are 20 books (connections), and 100 people (requests) take turns reading them.

Here's how we create our pool in `db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

/// Initialize the database connection pool and run migrations
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)                        // Up to 20 connections
        .acquire_timeout(Duration::from_secs(10))   // Wait up to 10 seconds
        .connect(database_url)                      // Connect to PostgreSQL
        .await?;

    // Run migrations from the migrations directory
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

Let's break down the pool configuration:

- **`max_connections(20)`** — We allow up to 20 simultaneous database connections. This is like having 20 clerks at the filing cabinet. More clerks = more people served simultaneously, but each clerk uses resources. For a typical web app, 10-20 is a good number.

- **`acquire_timeout(Duration::from_secs(10))`** — If all 20 connections are busy, wait up to 10 seconds for one to become available. If it takes longer, fail with an error. This prevents requests from hanging forever if the database is overloaded.

- **`connect(database_url)`** — Actually connect to PostgreSQL using the URL. The URL format is: `postgres://username:password@host:port/database_name`

The `PgPool` type is the pool itself. It's designed to be shared — you create it once and pass it to every handler. In our `AppState`, it's the `db` field. Handlers access it like this:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    // SQLx automatically borrows a connection from the pool
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // ← pool is passed here
    .await?;
    Ok(row)
}
```

You never manually open or close connections. SQLx handles it all through the pool.

🧪 **Try It Yourself**: What happens if you set `max_connections(1)` and try to handle two requests at the same time? Try it — start the server, and in two terminal windows, run `curl` commands simultaneously. The second request should wait until the first one finishes.

## Creating Tables with CREATE TABLE

PostgreSQL stores data in **tables** — like spreadsheets with rows and columns. Each table has a specific structure (which columns exist and what type of data they hold).

You create tables with SQL:

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,                    -- Auto-incrementing ID
    name TEXT NOT NULL,                       -- Text that can't be empty
    email TEXT UNIQUE NOT NULL,               -- Text that must be unique
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP  -- When it was created
);
```

Let's break down each part:

- **`CREATE TABLE users`** — "Create a new table called 'users'"
- **`id SERIAL PRIMARY KEY`** — An auto-incrementing integer that uniquely identifies each row. The first row gets id=1, the second gets id=2, etc. `PRIMARY KEY` means this column must be unique and non-null.
- **`name TEXT NOT NULL`** — A text column that cannot be empty
- **`email TEXT UNIQUE NOT NULL`** — A text column that must be unique across all rows AND cannot be empty
- **`created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP`** — A timestamp that automatically fills in with the current time when a row is inserted

Here are the SQL types you'll encounter most often:

| SQL Type | Rust Type | What It Means | Example |
|----------|-----------|---------------|---------|
| `SERIAL` | `i32` | Auto-incrementing integer | 1, 2, 3, ... |
| `BIGSERIAL` | `i64` | Like SERIAL but bigger | For tables with millions of rows |
| `INT4` / `INT8` | `i32` / `i64` | Regular integer | Fixed-size numbers |
| `SMALLINT` | `i16` | Small integer | For values 0-32767 |
| `TEXT` | `String` | Any text, any length | Titles, descriptions |
| `VARCHAR(128)` | `String` | Text with max length | URLs, IDs |
| `BOOLEAN` | `bool` | True or false | is_automated |
| `TIMESTAMPTZ` | `DateTime<Utc>` | Date + time + timezone | created, updated |
| `REAL` | `f32` | Floating-point number | Scores, ratings |
| `INET` | `IpAddr` | An IP address | 192.168.1.1 |

The difference between `INT4` and `INT8` matters. `INT4` (4 bytes) stores numbers up to about 2 billion. `INT8` (8 bytes) stores numbers up to about 9 quintillion. For a word count, `INT8` is safe — some fanfics are over 2 billion words... well, maybe not, but better safe than sorry!

## The fic_info Table: Our Main Data Store

Here's the heart of our database — the `fic_info` table. Every fanfic we know about gets a row here:

```sql
-- Fic metadata cache
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

Let's read this like a recipe:

- **`id VARCHAR(128) PRIMARY KEY`** — The unique identifier for each fic. For AO3 fics, this is the work ID (like "12345"). For FFN, it's the story ID. This is the primary key — every row must have a unique id.

- **`title TEXT NOT NULL`** — The fic's title. Cannot be empty.

- **`author TEXT NOT NULL`** — The author's name. Cannot be empty.

- **`author_url TEXT`** — The author's profile URL. Can be NULL (some fics don't have this).

- **`chapters INT4 NOT NULL`** — Number of chapters. A regular integer.

- **`words INT8 NOT NULL`** — Word count. A big integer to handle very long fics.

- **`description TEXT NOT NULL** — The fic's summary/description.

- **`fic_created TIMESTAMPTZ NOT NULL`** — When the fic was originally published on the source site.

- **`fic_updated TIMESTAMPTZ NOT NULL`** — When the fic was last updated on the source site.

- **`status TEXT NOT NULL`** — Completion status ("Complete", "In Progress", "Hiatus").

- **`source TEXT NOT NULL`** — Which site the fic is from ("ao3", "ffn", etc.).

- **`content_hash VARCHAR(256)`** — A hash of the fic's content. Used to detect when a fic has been updated. NULL if we haven't fetched the content yet.

The `IF NOT EXISTS` is important — it means this command is safe to run multiple times. If the table already exists, it does nothing. If it doesn't exist, it creates it. This is what makes migrations idempotent (safe to re-run).

Notice the difference between `created` (when we added it to our database) and `fic_created` (when the fic was published on the source site). These are often different — a fic published in 2015 might not have been added to our database until 2024.

## Creating a Database Migration File

Instead of running SQL manually (which is error-prone and hard to track), we use **migration files** — numbered SQL files that get applied in order. Our migrations live in `~/code/rust/fichub/migrations/`:

```
migrations/
├── 001_initial_schema.sql    ← Creates the basic tables
├── 002_recommender.sql       ← Adds recommendation engine tables
├── 003_tagging.sql           ← Adds tagging system tables
└── 004_shelves.sql           ← Adds OPDS shelf tables
```

The naming convention is `NNN_description.sql` where `NNN` is a three-digit number. This ensures they run in order (001 before 002, etc.).

Migration files are just SQL. They look like any other `.sql` file, but they're managed by SQLx's migration system. The key difference is that SQLx tracks which migrations have been applied, so running the server again won't re-apply old ones.

🧪 **Try It Yourself**: Create a new migration file `005_test.sql` with a simple `CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT);`. Restart the server and check if the table was created. You can check with `psql`:

```bash
psql -U your_user -d fichub -c "\dt test_table"
```

## Running Migrations with sqlx::migrate!

Here's how we run migrations on startup, from `db/mod.rs`:

```rust
// Find the migrations directory
let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
    if let Some(exe_dir) = exe_path.parent() {
        exe_dir.join("migrations")
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    }
} else {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
};

// Apply any new migrations
if migrations_path.exists() {
    sqlx::migrate::Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;
    tracing::info!("Database migrations applied");
}
```

The `sqlx::migrate!` macro (or `Migrator::new`) does something clever:

1. It creates a special table called `_sqlx_migrations` in your database (if it doesn't exist)
2. It reads all `.sql` files from the migrations directory, sorted by name
3. It checks which ones have already been applied (by looking at the tracking table)
4. It applies any new ones, in order
5. It marks them as done in the tracking table

So every time you start the server, it automatically applies any new migrations. You never have to run them manually!

The path resolution logic handles two cases:
- **Running from the compiled binary** — migrations are next to the executable
- **Running with `cargo run`** — migrations are in `CARGO_MANIFEST_DIR` (your project root)

## SQL Basics: The Language of Databases

Before we go further, let's make sure you understand the basic SQL commands. Think of SQL as a recipe language — you tell the database exactly what to do.

### SELECT: Reading Data

```sql
-- Get everything from a table
SELECT * FROM fic_info;

-- Get specific columns
SELECT title, author, words FROM fic_info;

-- Filter with WHERE
SELECT * FROM fic_info WHERE words > 10000;

-- Combine conditions
SELECT * FROM fic_info WHERE words > 10000 AND status = 'Complete';

-- Sort by a column
SELECT * FROM fic_info ORDER BY words DESC LIMIT 10;

-- Count rows
SELECT COUNT(*) FROM fic_info WHERE source = 'ao3';
```

The `*` means "all columns." The `WHERE` clause filters rows. `ORDER BY` sorts results. `LIMIT` caps the number of results.

### INSERT: Adding Data

```sql
-- Insert one row
INSERT INTO fic_info (id, title, author, words, status, source)
VALUES ('12345', 'A Great Story', 'SomeAuthor', 50000, 'Complete', 'ao3');

-- Insert multiple rows
INSERT INTO fic_info (id, title, author, words, status, source) VALUES
    ('11111', 'Story One', 'Author A', 10000, 'Complete', 'ao3'),
    ('22222', 'Story Two', 'Author B', 25000, 'In Progress', 'ffn');
```

### UPDATE: Changing Data

```sql
-- Update one row
UPDATE fic_info SET status = 'Complete' WHERE id = '12345';

-- Update multiple rows (CAREFUL!)
UPDATE fic_info SET status = 'Hiatus' WHERE source = 'ffn';

-- Update with a condition
UPDATE fic_info SET words = words + 1000 WHERE id = '12345';
```

⚠️ **Watch Out**: Never run UPDATE or DELETE without a WHERE clause! `UPDATE fic_info SET status = 'Hiatus'` (without WHERE) changes EVERY row in the table.

### DELETE: Removing Data

```sql
-- Delete one row
DELETE FROM fic_info WHERE id = '12345';

-- Delete rows matching a condition
DELETE FROM export_log WHERE created < NOW() - INTERVAL '30 days';
```

### JOIN: Combining Tables

```sql
-- Get fics with their tags
SELECT fic_info.title, tags.name
FROM fic_info
JOIN fic_tags ON fic_tags.url_id = fic_info.id
JOIN tags ON tags.id = fic_tags.tag_id
WHERE fic_info.id = '12345';
```

JOINs let you combine data from multiple tables. The `ON` clause specifies how the tables relate to each other.

## Connection Pool Tuning

The connection pool is one of the most important settings in your application. Too few connections and requests wait; too many and PostgreSQL runs out of resources.

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)                         // How many connections to create
    .min_connections(5)                          // Keep at least 5 warm connections
    .acquire_timeout(Duration::from_secs(10))    // How long to wait for a connection
    .idle_timeout(Duration::from_secs(300))      // Close idle connections after 5 minutes
    .max_lifetime(Duration::from_secs(1800))     // Close connections after 30 minutes
    .connect(database_url)
    .await?;
```

Here's what each setting means:

| Setting | Default | What It Does |
|---------|---------|-------------|
| `max_connections` | 10 | Max connections in the pool |
| `min_connections` | 0 | Min connections to keep warm |
| `acquire_timeout` | 30s | How long to wait for a free connection |
| `idle_timeout` | 10min | Close connections idle longer than this |
| `max_lifetime` | 30min | Close connections older than this |

**Rule of thumb**: Set `max_connections` to about 2-4x the number of CPU cores on your database server. For a small PostgreSQL instance with 2 cores, 10-20 connections is reasonable.

The `acquire_timeout` is critical — if all connections are busy and a new request comes in, it waits up to this duration. If the timeout expires, the request fails with an error. Set this to match your user's patience — 5-10 seconds is usually reasonable.

🧪 **Try It Yourself**: Set `max_connections(2)` and `acquire_timeout(Duration::from_secs(5))`. Then fire off 10 simultaneous curl requests. Watch the logs — you should see some requests waiting, and possibly timing out if they take too long.

## Common Query Patterns in FicHub

Let's look at some real query patterns from our codebase and understand why they're written the way they are:

### Pattern 1: Upsert (Insert or Update)

```sql
INSERT INTO fic_info (id, title, author, ...)
VALUES ($1, $2, $3, ...)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    author = EXCLUDED.author
```

This is the most common pattern in FicHub. When we scrape a fic, we either add it new or update it if it already exists. `EXCLUDED` refers to the values we tried to insert.

### Pattern 2: Optional Fetch

```rust
.fetch_optional(pool)  // Returns Option<T> — None if no rows match
```

This is safer than `fetch_one()` because it doesn't panic when there are no results. Use it when "not found" is a valid outcome.

### Pattern 3: Batch Operations

```sql
INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
VALUES ($1, $2, $3), ($1, $4, $3), ($1, $5, $3)
ON CONFLICT DO NOTHING
```

Instead of inserting one row at a time (which requires a round-trip to the database for each), batch them into a single query. This is much faster for bulk operations.

### Pattern 4: Conditional Queries

```sql
SELECT * FROM fic_info
WHERE ($1::text IS NULL OR title ILIKE '%' || $1 || '%')
AND ($2::int IS NULL OR words >= $2)
```

This lets you build dynamic filters without creating separate queries for each combination. If a parameter is NULL, it's ignored.

## Testing the Connection

Let's write a simple function to test that the database connection works:

```rust
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Simple query to verify the connection works
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    Ok(())
}
```

If this succeeds, the connection pool is working. If it fails, you'll get an error like "connection refused" (PostgreSQL isn't running) or "authentication failed" (wrong password).

You could call this in `main.rs` after creating the pool:

```rust
if let Err(e) = db::test_connection(&db_pool).await {
    tracing::error!("Database connection failed: {}", e);
    std::process::exit(1);
}
```

⚠️ **Watch Out**: The `init_pool` function runs migrations automatically. If you're setting up a fresh database, make sure the database user has permission to create tables and run migrations. On most PostgreSQL installations, the default user has these permissions.

---

# Chapter 8: Database Migrations and Models

## Why Migrations Matter

Your database schema will change over time — you'll add tables, modify columns, create indexes. Without migrations, you'd have to manually track and apply these changes on every server. Migrations automate this process, ensuring every database is in the same state.

## What Are Migrations? (Version Control for Your Database)

You know how Git tracks changes to your source code? Migrations are the same thing, but for your database.

Imagine you have a database with 10,000 rows of data. You can't just `DROP TABLE` and recreate it — you'd lose everything! Instead, you make **small, incremental changes**:

- Migration 1: Create the initial tables (001_initial_schema.sql)
- Migration 2: Add new tables for the recommender system (002_recommender.sql)
- Migration 3: Add tagging tables (003_tagging.sql)
- Migration 4: Add shelf tables (004_shelves.sql)

Each migration is a numbered SQL file. SQLx tracks which ones have been applied, so running the server again won't re-apply old migrations.

Why not just edit the SQL directly? Because:
1. **You can't go back** — if you drop a column, the data is gone
2. **You can't collaborate** — two people editing the same SQL is a nightmare
3. **You can't deploy safely** — you need to know exactly what changed between versions
4. **You can't test** — you can't easily run the exact same changes on a test database

Migrations solve all these problems. Each one is a self-contained, numbered, versioned change.

## The 001_initial_schema.sql File Explained Line by Line

Let's read our first migration like a recipe, understanding every line:

```sql
-- FicHub database schema
-- Migration 001: Initial schema
```

The `--` is a SQL comment. It's ignored by the database — just for humans to read. Always add comments at the top of your migrations explaining what they do.

```sql
-- Request source tracking
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);
```

This creates the `request_source` table — it tracks where requests come from (browser? bot? API?). Let's understand each line:

- **`id BIGSERIAL PRIMARY KEY`** — `BIGSERIAL` is like `SERIAL` but uses 8 bytes instead of 4, allowing for billions of rows. `PRIMARY KEY` means this uniquely identifies each row.

- **`created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP`** — When the record was created. `TIMESTAMPTZ` includes timezone info. `DEFAULT CURRENT_TIMESTAMP` means it auto-fills with the current time.

- **`is_automated BOOLEAN DEFAULT FALSE`** — Whether this is a bot. Default is `FALSE` (not a bot).

- **`route TEXT`** — The API route that was hit (like "/api/v0/epub"). Can be NULL.

- **`description TEXT`** — A description of the source. Can be NULL.

- **`UNIQUE(is_automated, route, description)`** — This means you can't have two rows with the exact same combination of these three columns. If someone tries to insert a duplicate, PostgreSQL raises an error.

```sql
-- Request log
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

The `request_log` table records every API request. Key points:

- **`source_id BIGINT REFERENCES request_source(id)`** — This is a **foreign key**. It links each log entry to a source. The `REFERENCES` clause means `source_id` must match an existing `id` in the `request_source` table. If you try to reference a source that doesn't exist, PostgreSQL rejects the insert.

- **`info_request_ms INT4 NOT NULL`** — How long the metadata lookup took (in milliseconds). This helps us track performance.

- **`url_id TEXT`** — The fic's ID, if we found one. NULL if the lookup failed.

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);
```

An **index** is like a table of contents — it makes certain queries much faster. Without this index, searching for all logs with a specific `url_id` would require scanning every row in the table (a "sequential scan"). With the index, PostgreSQL can jump straight to the right rows (an "index scan").

The index covers three columns (`url_id`, `etype`, `created`) because our most common query filters by all three. This is called a "composite index."

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

This is a **partial index** — it only indexes rows that match the WHERE clause. This is more efficient than indexing everything because:
1. The index is smaller (fewer rows)
2. Queries that match the WHERE clause are faster
3. Less disk space is used

Now the main table:

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

This is the core table. Every fanfic we know about gets a row here. Notice:

- The `id` is a `VARCHAR(128)`, not a `SERIAL`. That's because different sites use different ID formats — AO3 uses numeric IDs, but we might need room for other formats.

- `content_hash VARCHAR(256)` stores an MD5 or SHA-256 hash of the fic's content. When we check if a fic has been updated, we compare this hash to a new hash of the current content. If they differ, the fic has been modified.

- We have both `created` (when we added it to our database) and `fic_created` (when it was published on the source site). These are different dates!

The export log and blacklist tables follow similar patterns:

```sql
-- Export log (cache tracking)
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

The `UNIQUE(url_id, version, etype, input_hash)` constraint means you can't have two exports with the same fic, version, format, and input. This is what enables cache hits — if the same export exists, we serve the cached version instead of regenerating it.

## Creating the export_log Table

We already covered this above! The key insight is the unique constraint and the `REFERENCES fic_info(id)` foreign key. The foreign key ensures that `export_log` entries always reference a valid fic — you can't have an export for a fic that doesn't exist.

## The FicInfo Struct: A Rust Struct That Matches the Database Row

This is where SQLx gets magical. Our `FicInfo` struct in `db/models.rs` is a **mirror** of the database table:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Fanfiction metadata as stored in the database
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

Notice how every field maps to a database column:

| Rust Field | Rust Type | SQL Column | SQL Type | Nullable? |
|-----------|-----------|------------|----------|-----------|
| `id` | `String` | `id` | `VARCHAR(128)` | No (PRIMARY KEY) |
| `created` | `Option<DateTime<Utc>>` | `created` | `TIMESTAMPTZ` | Yes |
| `title` | `String` | `title` | `TEXT` | No |
| `author` | `String` | `author` | `TEXT` | No |
| `chapters` | `i32` | `chapters` | `INT4` | No |
| `words` | `i64` | `words` | `INT8` | No |
| `description` | `String` | `description` | `TEXT` | No |
| `fic_created` | `DateTime<Utc>` | `fic_created` | `TIMESTAMPTZ` | No |
| `status` | `String` | `status` | `TEXT` | No |
| `source` | `String` | `source` | `TEXT` | No |
| `extra_meta` | `Option<String>` | `extra_meta` | `TEXT` | Yes |
| `content_hash` | `Option<String>` | `content_hash` | `VARCHAR(256)` | Yes |

The `Option<...>` types correspond to nullable columns. In PostgreSQL, a column is nullable unless you say `NOT NULL`. In Rust, nullable means `Option<T>`. The mapping is automatic — SQLx's `FromRow` derive macro handles it.

The derive macros on the struct do the following:
- `Debug` — lets you print the struct with `{:?}` for debugging
- `Clone` — lets you make copies of the struct
- `Serialize` — lets you convert the struct to JSON (for API responses)
- `Deserialize` — lets you create the struct from JSON (for API requests)
- `FromRow` — lets SQLx convert a database row to this struct

## FromRow Derive: Automatically Mapping Rows to Structs

The `#[derive(FromRow)]` macro is the magic that connects Rust to PostgreSQL. It generates code that automatically converts a database row into a `FicInfo` struct:

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

Without `FromRow`, you'd have to manually extract each field from the row:

```rust
// WITHOUT FromRow (DON'T DO THIS!)
let row = sqlx::query("SELECT * FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

let fic = row.map(|r| FicInfo {
    id: r.get("id"),
    title: r.get("title"),
    author: r.get("author"),
    chapters: r.get("chapters"),
    words: r.get("words"),
    description: r.get("description"),
    // ... 15 more fields ...
});
```

That's tedious, error-prone, and hard to maintain. With `FromRow`:

```rust
// WITH FromRow (DO THIS!)
let fic: Option<FicInfo> = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;
```

One line versus twenty. And if you add a column to the database but forget to add it to the struct, SQLx will give you a compile-time error. That's type safety in action.

## Creating the Migration for Recommendations: 002_recommender.sql

Our second migration adds the recommendation engine — a system that suggests similar fics based on user favourites. This migration creates several related tables:

```sql
-- fic_works: site-specific work metadata and favourite counts
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

The `REFERENCES fic_info(id) ON DELETE CASCADE` is important — if a fanfic is deleted from `fic_info`, all related `fic_works` rows are automatically deleted too. This is called "cascade delete" and it keeps our data consistent.

The `UNIQUE(site_domain, site_work_id)` constraint ensures each work is only tracked once per site.

```sql
-- fic_bookmarks: tracks which user favourited which work
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

We store a `user_hash` (anonymized user ID) rather than actual user information. This is a privacy measure — we can track patterns without knowing who users are. The hash is a one-way transformation (like a fingerprint) that can't be reversed to find the original user.

```sql
-- fic_bookmark_cooccur: pairwise co-occurrence counts
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

This is the co-occurrence table — it counts how many users have favourited both Work A and Work B. The `CHECK (work_a < work_b)` constraint is clever — it ensures we always store the pair in alphabetical order. Without it, we might store both (A, B) and (B, A), which would double-count. With the constraint, only one ordering is allowed.

```sql
-- recommendation_suggestions: community-submitted recommendations
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);

-- recommendation_votes: upvotes/downvotes on suggestions
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

The voting system uses `CHECK (vote IN (-1, 1))` to ensure votes are either -1 (downvote) or +1 (upvote). No other values are allowed.

## The Tagging System Migration (003_tagging.sql)

Our third migration adds a sophisticated tagging system — users can tag fics with categories like "fandom", "character", "relationship", and "freeform". Let's look at the key parts:

```sql
-- Tag types (fixed enum)
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

This creates a "type" system for tags. Each tag belongs to a type (fandom, character, etc.). The `INSERT ... ON CONFLICT DO NOTHING` is important — it's idempotent, meaning running it multiple times doesn't create duplicates.

```sql
-- Canonical tags
CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` is interesting — it makes tag name comparison case-sensitive and fast. "Harry Potter" and "harry potter" are different tags. This is intentional for a tagging system.

```sql
-- Fic-tag junction (which tags are on which fics)
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

This is a **junction table** — it connects fics to tags. Many-to-many relationships in SQL always use junction tables. A fic can have many tags, and a tag can be on many fics.

```sql
-- Function: update fic_tags.score when a vote is inserted/updated/deleted
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

This is a PostgreSQL **trigger function** — it runs automatically whenever votes are inserted, updated, or deleted. It keeps the `score` column in `fic_tags` synchronized with the votes. This is a classic database pattern: store the detail data (votes) and maintain a summary (score) automatically.

```sql
-- Add full-text search to fic_info
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX IF NOT EXISTS idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

This adds PostgreSQL's built-in full-text search. The `tsvector` type stores pre-processed search terms. The `setweight` function assigns importance: title matches (weight 'A') rank higher than description matches (weight 'B'). The `GIN` index makes searches fast.

## Schema Design Principles

Looking at all our migrations together, some patterns emerge:

### 1. Use Appropriate Data Types
Don't use `TEXT` for everything. Use `INT4` for small numbers, `INT8` for large ones, `BOOLEAN` for true/false, `TIMESTAMPTZ` for dates. This saves space and catches bugs.

### 2. Add Constraints
`NOT NULL`, `UNIQUE`, `FOREIGN KEY`, `CHECK` — constraints prevent bad data from entering your database. It's easier to enforce rules in the schema than in application code.

### 3. Create Indexes for Common Queries
Look at your `WHERE` clauses and `JOIN` conditions. If you're filtering by a column frequently, add an index on it.

### 4. Use Foreign Keys with CASCADE
`REFERENCES fic_info(id) ON DELETE CASCADE` ensures that when a fic is deleted, all related records (tags, bookmarks, exports) are automatically cleaned up. Without CASCADE, you'd have orphaned records.

### 5. Make Migrations Idempotent
Always use `IF NOT EXISTS` and `ON CONFLICT`. This makes migrations safe to run multiple times, which is essential for automated deployment.

## Running Migrations on Startup

As we saw in Chapter 7, migrations run automatically when the server starts. This is handled in `db::init_pool()`:

```rust
sqlx::migrate::Migrator::new(migrations_path)
    .await?
    .run(&pool)
    .await?;
```

The `Migrator` reads all `.sql` files from the migrations directory, sorts them by name, and applies any that haven't been run yet. It tracks which ones have been applied in a special `_sqlx_migrations` table that looks like:

```sql
SELECT * FROM _sqlx_migrations;

--  version |          description          |          installed_on          | success
----------+-------------------------------+-------------------------------+---------
       1 | 001_initial_schema            | 2024-01-15 10:30:00+00        | t
       2 | 002_recommender               | 2024-01-15 10:30:01+00        | t
       3 | 003_tagging                   | 2024-01-15 10:30:02+00        | t
       4 | 004_shelves                   | 2024-01-15 10:30:03+00        | t
```

⚠️ **Watch Out**: Never manually edit a migration that's already been applied! If you need to change a schema, create a new migration (e.g., `005_fix_column.sql`). Changing an applied migration can cause the tracking table to get out of sync, leading to confusing errors.

⚠️ **Watch Out**: Always use `IF NOT EXISTS` in your migrations. This makes them idempotent — safe to run multiple times. Without it, re-running a migration would fail with "table already exists."

🧪 **Try It Yourself**: Look at the `_sqlx_migrations` table in your database. What columns does it have? How does SQLx track which migrations have been run? You can check with:

```bash
psql -U your_user -d fichub -c "SELECT * FROM _sqlx_migrations;"
```

---

# Chapter 9: CRUD Operations

## Why CRUD Matters

Every application — from a simple todo list to a massive social network — fundamentally does four things with data: creates it, reads it, updates it, and deletes it. Mastering CRUD is the foundation of database programming. Once you can do these four things safely and efficiently, you can build anything.

## What Is CRUD? (Create, Read, Update, Delete)

CRUD stands for the four basic things you can do with data. Every database interaction falls into one of these categories:

| Operation | SQL | What It Does | Restaurant Analogy |
|-----------|-----|-------------|-------------------|
| **Create** | INSERT | Add new data | A new customer sits down and orders |
| **Read** | SELECT | Look up data | Waiter reads the order |
| **Update** | UPDATE | Change existing data | Customer changes their order |
| **Delete** | DELETE | Remove data | Customer leaves and their order is removed |

Understanding CRUD is fundamental to building any application. Let's look at how FicHub does each one.

## INSERT: Saving a Fic to the Database

Here's how we save a fanfic to the database — the `upsert_fic_info` function:

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

This is an **upsert** (UPDATE + INSERT). Here's what happens:

1. **Try to INSERT** a new row with the fic's data
2. **If the id already exists** (`ON CONFLICT`), UPDATE the existing row instead
3. **Set `updated = NOW()`** to track when we last touched this record

The `EXCLUDED` keyword in the ON CONFLICT clause refers to the values you tried to insert. So `title = EXCLUDED.title` means "set title to the value we tried to insert."

The `$1, $2, $3...` are **placeholders** — they're replaced by the `.bind()` values. This is crucial for security! If you concatenated strings instead:

```rust
// DANGEROUS! NEVER DO THIS! SQL INJECTION VULNERABILITY!
let query = format!("INSERT INTO fic_info (id, title) VALUES ('{}', '{}')", id, title);
```

An attacker could inject SQL:
```
id = "'; DROP TABLE fic_info; --"
```

This would generate:
```sql
INSERT INTO fic_info (id, title) VALUES (''; DROP TABLE fic_info; --', 'some title')
```

Which would DROP your entire fic_info table! With `.bind()`, the database driver handles escaping. User input is always treated as data, never as SQL code.

🧪 **Try It Yourself**: Create a simple function that inserts a row into a test table. What happens if you try to insert a duplicate primary key without `ON CONFLICT`?

## SELECT: Finding a Fic by url_id

Reading data is the most common operation. Here's `get_fic_info`:

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

Let's break this down piece by piece:

- **`sqlx::query_as::<_, FicInfo>`** — "Run a SQL query and convert the result into a `FicInfo` struct." The `_,` is a turbofish that lets you specify the output type.

- **`"SELECT * FROM fic_info WHERE id = $1"`** — The SQL query. `$1` is the first placeholder (like a blank in a fill-in-the-blank test).

- **`.bind(id)`** — "Put the `id` value into `$1`." This safely escapes the value.

- **`.fetch_optional(pool)`** — "Run the query and return at most one result. If nothing matches, return `None`."

The return type is `Option<FicInfo>` — either `Some(fic)` if found, or `None` if not. This is much safer than `fetch_one()`, which would panic if no rows are returned.

SQLx offers several fetch methods:

| Method | Returns | Use When |
|--------|---------|----------|
| `fetch_optional(pool)` | `Option<Row>` | You expect 0 or 1 results |
| `fetch_one(pool)` | `Row` | You expect exactly 1 result (panics if 0) |
| `fetch_all(pool)` | `Vec<Row>` | You expect multiple results |
| `fetch_scalar(pool)` | `Option<T>` | You want a single value (like COUNT) |
| `execute(pool)` | `QueryResult` | You don't need data back (INSERT/UPDATE/DELETE) |

For searching similar fics, we use `fetch_all`:

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

The `ILIKE` operator is case-insensitive LIKE — it matches "Harry" and "harry" and "HARRY". The `%` wildcards mean "anything before and after." So `%harry%` matches any title containing "harry" anywhere.

## UPDATE: Changing a Fic's Status

Updating data uses the `UPDATE` SQL statement:

```sql
UPDATE fic_info SET status = $1, updated = NOW() WHERE id = $2
```

And in Rust:

```rust
pub async fn update_fic_status(pool: &PgPool, id: &str, status: &str) -> AppResult<()> {
    sqlx::query("UPDATE fic_info SET status = $1, updated = NOW() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

The `WHERE id = $2` clause is critical — without it, you'd update **every row** in the table! Always double-check your WHERE clause before running an UPDATE.

Our export log uses an upsert pattern for updates:

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

This is an upsert — if the export already exists (same fic, version, format, and input), we update the export hash and timestamp. If it's new, we insert it.

## DELETE: Removing Old Cache Entries

Deleting data uses the `DELETE` SQL statement. Our tag management functions show safe deletion patterns:

```rust
pub async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    // Safety check: don't delete if fics are using this tag
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

    // Delete in reverse dependency order (children first, then parent)
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id).execute(pool).await?;

    Ok(())
}
```

Notice three important patterns:

1. **Safety check first** — we check if any fics use this tag before deleting. This prevents accidental data loss.

2. **Reverse dependency order** — we delete `fic_tags` first (the children), then `tag_aliases`, then `tags` (the parent). If we deleted `tags` first, the foreign key constraint on `fic_tags` would prevent the deletion.

3. **The `force` parameter** — when `force=true`, we skip the safety check and delete everything. This is for admin use when you really need to clean up.

⚠️ **Watch Out**: Always use a WHERE clause with DELETE! `DELETE FROM table` (without WHERE) deletes **everything** in the table. That's usually catastrophic. Always test your DELETE queries with a SELECT first:

```sql
-- First, check what would be deleted:
SELECT * FROM fic_tags WHERE tag_id = 1;

-- Then, when you're sure:
DELETE FROM fic_tags WHERE tag_id = 1;
```

## The queries.rs File: Database Functions as Rust Functions

Our `queries.rs` file is organized as a collection of async functions — one per operation. This is a clean pattern that makes database code easy to find and maintain:

| Function | Operation | What It Does |
|----------|-----------|-------------|
| `upsert_fic_info` | INSERT/UPDATE | Save or update a fanfic's metadata |
| `get_fic_info` | SELECT | Look up a fic by its ID |
| `insert_request_source` | INSERT | Record where a request came from |
| `insert_request_log` | INSERT | Log an API request |
| `find_export_log` | SELECT | Check if we have a cached export |
| `insert_export_log` | INSERT/UPDATE | Record a new export in the cache |
| `check_fic_blacklist` | SELECT | Check if a fic is blocked |
| `check_author_blacklist` | SELECT | Check if an author is blocked |
| `get_fic_version_bump` | SELECT | Check for cache invalidation signals |
| `search_similar_fics` | SELECT | Find fics with similar titles |
| `lookup_tag_by_name` | SELECT | Find a tag by its exact name |
| `create_tag` | INSERT | Create a new canonical tag |
| `upsert_fic_tag` | INSERT/UPDATE | Tag a fanfic with a tag |
| `get_fic_tags` | SELECT | Get all tags for a fic with scores |
| `upsert_tag_vote` | INSERT/UPDATE | Record a vote on a tag |
| `get_existing_vote` | SELECT | Check if a user already voted |
| `insert_tag_flag` | INSERT | Flag a tag for review |
| `list_unresolved_flags` | SELECT | Get flags needing curator attention |
| `resolve_flag` | UPDATE | Mark a flag as resolved |
| `check_rate_limit` | Redis | Check and increment rate limit counter |
| `create_tag_alias` | INSERT | Create an alias for a tag |
| `merge_tags` | UPDATE/DELETE | Merge one tag into another |
| `delete_tag` | DELETE | Remove a tag and its associations |

Each function follows the same pattern:
1. Accept the `PgPool` (or `redis::aio::MultiplexedConnection`) and the data needed
2. Build a SQL query with `.bind()` for parameters
3. Execute with the appropriate fetch method
4. Return `AppResult<T>` (which automatically converts database errors to `AppError`)

The consistent pattern makes the code predictable. When you see `pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>>`, you know exactly what it does without reading the implementation.

## Using sqlx::query_as with PostgreSQL

The `query_as` function is the key to type-safe database access:

```rust
// query_as: converts rows to structs automatically (PREFERRED)
let fic: Option<FicInfo> = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;

// query: returns raw rows (less convenient, more manual work)
let row: Option<sqlx::postgres::PgRow> = sqlx::query(
    "SELECT * FROM fic_info WHERE id = $1",
)
.bind(id)
.fetch_optional(pool)
.await?;
```

The `<_, FicInfo>` part is a turbofish — it tells Rust "convert the result into `FicInfo`." The `_` means "figure out the row type automatically."

For simple queries where you just need one value, use `query_scalar`:

```rust
let count: Option<i64> = sqlx::query_scalar::<_, i64>(
    "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1",
)
.bind(tag_id)
.fetch_optional(pool)
.await?;
```

This is more efficient than `query_as` when you only need a single value — no need to create a whole struct.

For queries that return tuples of known types:

```rust
let row: (i32, String, i16) = sqlx::query_as(
    "SELECT id, name, tag_type_id FROM tags WHERE name = $1",
)
.bind(name)
.fetch_optional(pool)
.await?
.unwrap_or_default();
```

The tuple `(i32, String, i16)` matches the three columns selected. This is useful for quick lookups where you don't need a full struct.

## Understanding the `?` Operator and Error Propagation

The `?` operator is one of Rust's most elegant features for error handling. Let's understand it deeply because you'll use it everywhere in database code.

### How `?` Works

The `?` operator does two things:
1. If the value is `Ok(v)`, it unwraps to `v` and execution continues
2. If the value is `Err(e)`, it **returns early** from the function, converting the error

Here's a concrete example:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Result<Option<FicInfo>, sqlx::Error>
    .await?;               // ← ? converts sqlx::Error to AppError::Database
    Ok(row)
}
```

Let's trace what happens in two scenarios:

**Scenario 1: Success**
```
.query_as(...)
.bind(id)
.fetch_optional(pool)   → Ok(Some(fic))
.await                  → Ok(Some(fic))
?                       → Some(fic)  (unwrapped, execution continues)
Ok(row)                 → Ok(Some(fic))  (returned to caller)
```

**Scenario 2: Database error**
```
.query_as(...)
.bind(id)
.fetch_optional(pool)   → Err(sqlx::Error::ConnectionRefused)
.await                  → Err(sqlx::Error::ConnectionRefused)
?                       → EARLY RETURN: Err(AppError::Database("connection refused"))
                          (the Ok(row) line is never reached)
```

The `?` operator essentially says: "If this fails, convert the error and return it. Otherwise, give me the value."

### Without `?`

To understand why `?` is so valuable, look at what you'd write without it:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let result = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(row) => Ok(row),
        Err(e) => Err(AppError::Database(e.to_string())),
    }
}
```

That's 6 extra lines for every database call! With `?`, it's zero extra lines. The `From<sqlx::Error> for AppError` implementation handles the conversion.

### Chaining `?`

You can chain multiple `?` operators in a single function. Each one can return early:

```rust
pub async fn export_fic(pool: &PgPool, id: &str) -> AppResult<String> {
    // Step 1: Look up the fic (might fail)
    let fic = get_fic_info(pool, id).await?;   // ← Returns early if not found

    // Step 2: Check blacklist (might fail)
    let blacklist = check_fic_blacklist(pool, id).await?;  // ← Returns early on DB error

    if !blacklist.is_empty() {
        return Err(AppError::BadRequest(-7, "fic is blacklisted".into()));
    }

    // Step 3: Generate the file (might fail)
    let content = generate_epub(&fic).await?;  // ← Returns early on generation error

    Ok(content)
}
```

Each `?` is a potential early return point. If any step fails, the function returns immediately with the error. The caller never sees partial results.

### The `?` Operator in Closures

The `?` operator also works in closures and async blocks, but you need to be careful:

```rust
// This works:
let fic = sqlx::query_as::<_, FicInfo>("SELECT ...")
    .fetch_optional(pool)
    .await?;

// This also works in a closure:
let fics: Vec<FicInfo> = urls
    .iter()
    .filter_map(|url| {
        // ? works here because the closure returns Option
        let fic = lookup_fic(url).ok()?;
        Some(fic)
    })
    .collect();
```

## Error Handling in Database Operations

Every query can fail — the database might be down, the connection might time out, the query might have a syntax error. Our error handling uses the `?` operator with automatic conversion:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;  // ← If this fails, sqlx::Error is converted to AppError::Database
    Ok(row)
}
```

The `From<sqlx::Error> for AppError` implementation we saw in Chapter 6 makes this automatic. The `?` operator:
1. If the result is `Ok(value)`, unwrap it and continue
2. If the result is `Err(e)`, convert `e` to `AppError::Database` and return early from the function

This means every database function automatically handles errors cleanly — no `.unwrap()` that could panic in production. When a database error occurs, the handler returns a clean JSON response like `{"err": -1, "msg": "database error"}` instead of crashing.

For operations that need to handle "not found" gracefully:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)  // Returns Ok(None) if not found
    .await?;                // Converts sqlx::Error to AppError
    Ok(row)                 // Returns Ok(None) to the caller
}
```

The caller then decides what to do with `None`:
```rust
let fic = queries::get_fic_info(&state.db, &url_id).await?;

match fic {
    Some(fic) => Ok(Json(json!({ "title": fic.title, ... }))),
    None => Err(AppError::NotFound(format!("fic {} not found", url_id))),
}
```

🧪 **Try It Yourself**: Write a function that queries a nonexistent table. What error do you get? How does the error type get converted to `AppError`? Try adding a `.unwrap()` and see what happens when the query fails.

---

# Chapter 10: The Axum Router and Middleware

## Why Middleware Matters

Without middleware, every handler would need to log requests, check CORS headers, compress responses, and handle errors independently. That's a lot of duplicated code. Middleware solves this by wrapping handlers with reusable behavior — write once, apply everywhere. It's the difference between every waiter memorizing the health code versus having a health inspector check everyone uniformly.

## Building the Full Router with All Routes

Now let's see the complete router — the "menu" of our restaurant. This is where everything comes together: every endpoint, every handler, every piece of middleware.

Here's the full `build_router` function from `server.rs`:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // A helper function for legacy redirects
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }

    Router::new()
        // ── API routes ──────────────────────────────────────────────
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // ── Cache download routes ──────────────────────────────────
        .route("/cache/{etype}/{url_id}/{fname}",
               get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}",
               get(routes::cache_download::download_or_export))

        // ── Recommender routes ─────────────────────────────────────
        .route("/api/v0/recommendations",
               get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest",
               post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote",
               post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes",
               get(crate::recommender::routes::votes_handler))

        // ── Tag routes (v3) ────────────────────────────────────────
        .route("/api/v0/tags/submit",
               post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote",
               post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag",
               post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags",
               get(crate::tags::routes::get_tags))

        // ── Curator routes (v3) ────────────────────────────────────
        .route("/api/v0/curator/alias",
               post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge",
               post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}",
               delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags",
               get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve",
               post(crate::tags::curator::resolve_flag))

        // ── Search routes ──────────────────────────────────────────
        .route("/api/v0/search",
               get(crate::search::routes::search_handler))

        // ── OPDS catalog routes ────────────────────────────────────
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}",
               get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}",
               get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors",
               get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular",
               get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations",
               get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves",
               get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}",
               get(crate::routes::opds::shelves::shelf_contents))

        // ── Legacy redirect routes ─────────────────────────────────
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))

        // ── Static frontend files (catch-all fallback) ─────────────
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )

        // ── Middleware (applied to ALL routes above) ────────────────
        .layer(TraceLayer::new_for_http())    // Request/response logging
        .layer(CorsLayer::permissive())       // CORS headers

        // ── Shared state ───────────────────────────────────────────
        .with_state(state)
}
```

That's a lot of routes! Let's organize them by category and understand the patterns:

### API Routes (`/api/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/api/` | GET | `api_docs_handler` | Self-documenting API docs (like a restaurant's menu) |
| `/api/v0/epub` | GET | `epub_handler` | Get metadata and download links |
| `/api/v0/meta` | GET | `meta_handler` | Get metadata only (no downloads) |
| `/api/v0/remote` | GET | `remote_handler` | Get request source info (IP, port) |
| `/api/v0/search` | GET | `search_handler` | Search for fanfics |

### Cache Routes (`/cache/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/cache/{etype}/{url_id}/{fname}` | GET | `download_with_hash` | Direct download with hash validation |
| `/cache/{etype}/{url_id}` | GET | `download_or_export` | Download if cached, otherwise trigger export |

The `{etype}` can be "epub", "html", "mobi", or "pdf". The `{url_id}` is the fic's identifier. The `{fname}` is the filename (optional).

### Tag Routes (`/api/v0/tags/...`)

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/api/v0/tags` | GET | `get_tags` | Get all tags for a fic |
| `/api/v0/tags/submit` | POST | `submit_tag` | Submit a new tag for a fic |
| `/api/v0/tags/vote` | POST | `vote_tag` | Upvote or downvote a tag |
| `/api/v0/tags/flag` | POST | `flag_tag` | Flag a tag for review |

### OPDS Routes (`/opds/...`)

OPDS (Open Publication Distribution System) is a standard for ebook catalog browsing. Our OPDS routes let ebook readers browse and download fics:

| Route | Method | Handler | Purpose |
|-------|--------|---------|---------|
| `/opds` | GET | `root_catalog` | Root catalog (starting point) |
| `/opds/new` | GET | `recent_feed` | Recently added fics |
| `/opds/popular` | GET | `popular_feed` | Popular fics |
| `/opds/tags` | GET | `tag_types` | Browse by tag type |
| `/opds/authors` | GET | `author_list` | Browse by author |
| `/opds/search` | GET | `search_feed` | Search the catalog |
| `/opds/shelves` | GET | `shelf_list` | Shared reading lists |

### Legacy Routes

```rust
.route("/legacy/epub_export", get(redirect_to_root))
.route("/fic/{url_id}", get(redirect_to_root))
.route("/changes", get(redirect_to_root))
.route("/popular/", get(redirect_to_root))
```

These redirect old URLs to the new frontend. If someone has a bookmark to `/fic/12345`, they'll be sent to the homepage instead of getting a 404. This is a nice UX touch.

## The run() Function: Starting the Server

The `run()` function orchestrates the entire startup sequence. Let's look at it again with more commentary:

```rust
pub async fn run(config: Config) {
    // STEP 1: Connect to PostgreSQL
    // This creates the connection pool and runs any pending migrations
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    // If PostgreSQL isn't running, we crash here with a clear message

    // STEP 2: Connect to Redis
    // Redis is our caching layer — instant lookups for frequently accessed data
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // STEP 3: Build HTTP client
    // Reusable client for fetching web pages (fanfic metadata)
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")     // Identify ourselves
        .timeout(Duration::from_secs(30))   // Give up after 30 seconds
        .build()
        .expect("Failed to build HTTP client");

    // STEP 4: Initialize scraper registry
    // This knows how to extract metadata from AO3, FFN, etc.
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // STEP 5: Initialize rate limiter
    // Prevents any single user from overwhelming the server
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // Load datacenter IPs if configured
    // (bots from datacenters get different rate limits)
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // STEP 6: Create shared state
    // Everything handlers need, packed into one shared object
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

    // STEP 7: Build the router with all routes
    let app = build_router(state).await;

    // STEP 8: Bind to a port and start accepting connections
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

The `into_make_service_with_connect_info::<std::net::SocketAddr>()` part is important — it tells Axum to pass the client's IP address to handlers that need it. This is how our `remote_handler` knows who's making the request:

```rust
async fn remote_handler(
    ConnectInfo(remote_addr): ConnectInfo<std::net::SocketAddr>,
) -> Json<Value> {
    Json(json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

The `ConnectInfo` extractor pulls the client's socket address out of the request. Without `into_make_service_with_connect_info`, this information wouldn't be available.

The `.expect()` calls throughout are intentional — they cause the server to crash immediately if any dependency fails. In a real deployment, you'd want health checks and graceful shutdown, but for development, failing fast is the right behavior.

## Middleware: The Security Guards

**Middleware** is code that wraps around your handlers. It runs before (and sometimes after) the handler, adding extra behavior. Think of middleware as security guards at the door — they check your ID, log who came in, and maybe offer you a mint before you sit down.

Axum uses the **Layer** pattern for middleware:

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .layer(TraceLayer::new_for_http())  // ← Middleware 1
    .layer(CorsLayer::permissive())     // ← Middleware 2
```

Layers apply to **all routes above them**. So both `TraceLayer` and `CorsLayer` apply to every route in our router.

The order of layers matters! They wrap from outside in — the last layer added is the first to execute. So `CorsLayer` runs first (adds CORS headers), then `TraceLayer` runs (logs the request), then the actual handler runs.

You can also apply middleware to specific routes:

```rust
Router::new()
    .route("/public", get(public_handler))
    .route("/admin", get(admin_handler))
    .layer(auth_middleware)  // Only wraps these routes
```

This is how you protect certain routes (like admin pages) with authentication while leaving public routes open.

## TraceLayer: Logging Every Request

The `TraceLayer` logs every HTTP request that hits our server:

```rust
use tower_http::trace::TraceLayer;

.layer(TraceLayer::new_for_http())
```

When a request comes in, you'll see output like:

```
2024-01-15T10:30:00Z INFO request{method=GET uri=/api/v0/epub version=HTTP/1.1}: tower_http::trace::on_response: finished 45ms - 200 OK
```

This tells you:
- **When** the request happened (timestamp)
- **What** the request was (GET /api/v0/epub)
- **What version** of HTTP was used (HTTP/1.1)
- **How long** it took (45ms)
- **What the response was** (200 OK)

For debugging, this is invaluable. When something goes wrong, you can look at the logs and see exactly what happened. The timing information helps you spot slow queries or API calls.

For production monitoring, you might want more detailed logging:

```rust
.layer(
    TraceLayer::new_for_http()
        .on_request(|request: &Request<_>| {
            tracing::info!("Incoming: {} {}", request.method(), request.uri());
        })
        .on_response(|response: &Response<_>, latency: Duration| {
            tracing::info!("Response: {} in {:?}", response.status(), latency);
        })
)
```

## CorsLayer: Allowing Cross-Origin Requests

**CORS** (Cross-Origin Resource Sharing) is a browser security feature that prevents websites from making requests to other websites without permission.

By default, a website at `http://localhost:5173` (your SvelteKit frontend) can't make requests to `http://localhost:3000` (your Rust backend). The browser blocks it because they're different "origins" (different ports count as different origins!).

The `CorsLayer` fixes this:

```rust
use tower_http::cors::CorsLayer;

.layer(CorsLayer::permissive())
```

`CorsLayer::permissive()` allows all origins, all methods, and all headers. This is convenient for development but too permissive for production.

In production, you'd want to be more specific:

```rust
use tower_http::cors::{CorsLayer, Any, Method};
use axum::http::HeaderValue;

.layer(
    CorsLayer::new()
        .allow_origin("https://fichub.example.com".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600)),
)
```

This allows:
- Only requests from `https://fichub.example.com`
- Only GET and POST methods
- Any headers
- Cached for 1 hour (browsers won't re-check CORS for an hour)

⚠️ **Watch Out**: Never use `CorsLayer::permissive()` in production with sensitive data. It allows any website on the internet to make requests to your API, which could leak user data or allow unauthorized actions.

## Serving Static Files with ServeDir

Our SvelteKit frontend builds static files (HTML, CSS, JavaScript). Axum serves them with `ServeDir`:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

Here's how it works, step by step:

1. **`ServeDir::new(&frontend_dir)`** — Serve files from the `./frontend/build` directory. If someone requests `/style.css`, Axum looks for `./frontend/build/style.css`.

2. **`append_index_html_on_directories(true)`** — If someone visits `/` or `/about/`, look for `index.html` in that directory. Without this, visiting `/about/` would look for a directory listing instead of the app.

3. **`.fallback(ServeFile::new(...))`** — If the file isn't found in the directory, serve `index.html` from the frontend build directory. This is crucial for Single Page Applications (SPAs) — the frontend handles routing client-side, so all unmatched paths should serve the main HTML file.

The `fallback_service` is important — it's the **last** thing Axum checks. The priority is:
1. Check all explicit routes first
2. If no route matches, try static file serving
3. If no file matches, serve `index.html` (the SPA fallback)

This means your API routes (`/api/v0/...`) are checked before static files. If someone visits `/api/v0/epub`, they hit the handler, not a file.

## Middleware Composition: The Tower Ecosystem

Axum's middleware system is built on **Tower** — a library that provides a universal interface for services, layers, and middleware. Think of Tower as the "universal remote control" for async services.

The Tower ecosystem includes many useful middleware layers:

### Request Body Size Limiting

```rust
use tower_http::limit::RequestBodyLimitLayer;

// Limit request bodies to 1MB
.layer(RequestBodyLimitLayer::new(1024 * 1024))
```

This prevents someone from sending a 1GB POST request and crashing your server.

### Gzip Compression

```rust
use tower_http::compression::CompressionLayer;

.layer(CompressionLayer::new())
```

This automatically compresses responses with gzip. For JSON responses, this can reduce bandwidth by 70-90%.

### Request Timeout

```rust
use tower::timeout::TimeoutLayer;
use std::time::Duration;

.layer(TimeoutLayer::new(Duration::from_secs(30)))
```

If a handler takes longer than 30 seconds, the connection is terminated. This prevents slow handlers from holding connections forever.

### Custom Middleware

You can write your own middleware. Here's a simple logging middleware:

```rust
use axum::middleware::{self, Next};
use axum::extract::Request;

async fn log_request(request: Request, next: Next) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    tracing::info!("{} {} → {}", method, uri, response.status());
    response
}

// Usage:
.layer(middleware::from_fn(log_request))
```

This middleware logs every request and its status code. The `next.run(request)` call passes the request to the next middleware (or the handler if there are no more middleware).

### Stacking Middleware

The order of `.layer()` calls matters. Middleware wraps from outside in:

```rust
Router::new()
    .route("/api", get(handler))
    .layer(TimeoutLayer::new(Duration::from_secs(30)))  // 3rd: timeout
    .layer(CompressionLayer::new())                      // 2nd: compress
    .layer(TraceLayer::new_for_http())                   // 1st: log
```

Execution order:
1. `TraceLayer` logs the incoming request
2. `CompressionLayer` prepares to compress the response
3. `TimeoutLayer` sets a 30-second timeout
4. The handler runs
5. Response goes back through CompressionLayer (compressed)
6. Response goes back through TraceLayer (logs the response)

Think of it like layers of an onion — each layer wraps the one inside it.

🧪 **Try It Yourself**: Add a `TimeoutLayer` to your router and test what happens when a handler sleeps for longer than the timeout:

```rust
async fn slow_handler() -> &'static str {
    tokio::time::sleep(Duration::from_secs(10)).await;
    "Done!"
}

// Route with 5-second timeout
.layer(TimeoutLayer::new(Duration::from_secs(5)))
```

Visit the endpoint — you should get a timeout error after 5 seconds, not after 10.

## The Complete Router Tree

Let's visualize how everything fits together in one diagram:

```
Router::new()
│
├── EXPLICIT ROUTES (checked first, in order)
│   ├── /api/                              → api_docs_handler (GET)
│   ├── /api/v0/epub                       → epub_handler (GET)
│   ├── /api/v0/meta                       → meta_handler (GET)
│   ├── /api/v0/remote                     → remote_handler (GET)
│   ├── /api/v0/search                     → search_handler (GET)
│   ├── /api/v0/recommendations            → recommendations_handler (GET)
│   ├── /api/v0/recommendations/suggest    → suggest_handler (POST)
│   ├── /api/v0/recommendations/vote       → vote_handler (POST)
│   ├── /api/v0/recommendations/votes      → votes_handler (GET)
│   ├── /api/v0/tags                       → get_tags (GET)
│   ├── /api/v0/tags/submit                → submit_tag (POST)
│   ├── /api/v0/tags/vote                  → vote_tag (POST)
│   ├── /api/v0/tags/flag                  → flag_tag (POST)
│   ├── /api/v0/curator/*                  → curator handlers
│   ├── /cache/{etype}/{url_id}/{fname}    → download_with_hash (GET)
│   ├── /cache/{etype}/{url_id}            → download_or_export (GET)
│   ├── /opds                              → root_catalog (GET)
│   ├── /opds/new, /opds/popular, etc.     → OPDS feed handlers
│   ├── /legacy/epub_export                → redirect_to_root (GET)
│   └── /fic/{url_id}                      → redirect_to_root (GET)
│
├── FALLBACK SERVICE (if no route matched)
│   └── ServeDir(frontend_dir)
│       ├── Try: /style.css → frontend/build/style.css
│       ├── Try: /about → frontend/build/about/index.html
│       └── Fallback: frontend/build/index.html (SPA)
│
├── MIDDLEWARE (applied to ALL of the above)
│   ├── CorsLayer::permissive()   → adds CORS headers
│   └── TraceLayer::new_for_http() → logs requests
│
└── STATE (available to all handlers)
    └── Arc<AppState>
        ├── config: Config
        ├── db: PgPool
        ├── redis: MultiplexedConnection
        ├── http_client: reqwest::Client
        ├── scraper_registry: ScraperRegistry
        ├── cache_semaphores: CacheSemaphores
        ├── rate_limiter: RateLimiter
        ├── recommender_engine: RecommendationEngine
        └── collection_worker: CollectionWorker
```

## The Cache System and Export Types

FicHub supports multiple export formats — EPUB, HTML, MOBI, and PDF. Each format has its own file extension, MIME type, and version number. The `EType` enum in `cache/mod.rs` defines these:

```rust
/// The type of export format
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }
}

impl std::str::FromStr for EType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            _ => Err(()),
        }
    }
}
```

The `FromStr` implementation lets you parse strings into `EType` values:

```rust
let etype: EType = "epub".parse().unwrap();  // EType::Epub
let etype: EType = "invalid".parse();         // Err(())
```

This is used in the cache download route:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };
    // ... serve the file ...
}
```

The cache system uses a semaphore to prevent duplicate concurrent exports:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;
```

When two users request the same export simultaneously, the semaphore ensures only one generates the file while the other waits. This prevents wasted work and potential race conditions.

## How Routes Connect to Handler Functions

The connection between routes and handlers is explicit — you write it yourself. This is one of Axum's strengths: there's no magic, no convention-over-configuration, no hidden routing. Everything is visible in one place.

```rust
.route("/api/v0/epub", get(routes::export::epub_handler))
```

This single line says: "When someone sends a GET request to `/api/v0/epub`, call the `epub_handler` function from the `routes::export` module."

For POST routes:

```rust
.route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
```

For routes that handle multiple methods:

```rust
.route("/api/v0/tags",
    get(crate::tags::routes::get_tags)
     .post(crate::tags::routes::create_tag)
     .delete(crate::tags::routes::delete_tag)
)
```

The route matching is **first-match wins**. If you have overlapping routes:

```rust
.route("/api/v0/tags", get(get_tags))
.route("/api/v0/tags/submit", post(submit_tag))
```

When someone visits `/api/v0/tags/submit`, Axum first tries to match `/api/v0/tags` — but the method is POST and that route only handles GET, so it doesn't match. Then it tries `/api/v0/tags/submit` — method is POST and this route handles POST, so it matches.

⚠️ **Watch Out**: Route order matters! If you have a catch-all route, put it last. Otherwise, it will match everything and your specific routes will never be reached. The `.fallback_service()` in our router handles this correctly — it's at the very end.

⚠️ **Watch Out**: Path parameters like `{url_id}` must be at the end of a segment. You can't have `/api/{id}/epub` — Axum requires path segments to be at the end. Use `/api/v0/{id}` instead.

🧪 **Try It Yourself**: Add a new route that handles both GET and POST. GET should return `{"method": "GET"}` and POST should return `{"method": "POST"}`. Test with:

```bash
# GET request
curl http://localhost:3000/your-route

# POST request
curl -X POST http://localhost:3000/your-route
```

---

## What We Built

In Part 2, we built the entire Rust backend for FicHub. We went from an empty `main.rs` to a fully functional web server with database, configuration, error handling, middleware, and all the routes wired up. Let's recap what we accomplished:

**Chapter 5: Your First Rust Web Server (Axum)** — We learned what Axum is (a focused, composable web framework built on Tokio), how the `#[tokio::main]` macro sets up the async runtime, how to create a Router with routes, how handlers extract query parameters (`Query<...>`) and path parameters (`Path<...>`), how to share state across handlers using `Arc<AppState>`, and traced the complete 12-step journey of a request from TCP connection to JSON response. We built a complete simple server example and tested it with curl.

**Chapter 6: Configuration and Error Handling** — We built a configuration system using environment variables and `.env` files with `dotenvy`, created a comprehensive `Config` struct with `from_env()` that handles required settings (`.expect()`), optional settings with defaults (`.unwrap_or_else()`), and type conversion (`.parse()`). We built a robust `AppError` enum with eight error variants, each mapping to specific HTTP status codes and JSON error codes (-1, -5, -6, -7, -10, -429). We saw how the `From` trait enables automatic error conversion with the `?` operator, and learned four error recovery patterns: log-and-continue, fallback values, partial results, and retry on transient errors.

**Chapter 7: Connecting to PostgreSQL** — We connected to PostgreSQL using SQLx and connection pools (`PgPoolOptions`), learned how to create tables with CREATE TABLE, understood the fic_info table's schema with its 18 columns, created our first migration file, and learned how `sqlx::migrate!` runs migrations automatically on startup. We covered SQL basics (SELECT, INSERT, UPDATE, DELETE, JOIN), connection pool tuning (max_connections, acquire_timeout, idle_timeout), and common query patterns (upsert, optional fetch, batch operations, conditional queries).

**Chapter 8: Database Migrations and Models** — We understood migrations as version control for your database, read through our initial schema migration (`001_initial_schema.sql`) line by line, explored the recommender system tables (`002_recommender.sql`) with co-occurrence tracking and community voting, examined the tagging system (`003_tagging.sql`) with trigger functions and full-text search, saw how the `FicInfo` struct mirrors the database table using `#[derive(FromRow)]`, and learned about the `_sqlx_migrations` tracking table. We also covered schema design principles: appropriate data types, constraints, indexes, foreign keys with CASCADE, and idempotent migrations.

**Chapter 9: CRUD Operations** — We mastered INSERT (saving fics with upsert), SELECT (finding fics by ID, searching similar fics), UPDATE (changing statuses, recording exports), and DELETE (removing tags with safety checks). We deeply understood the `?` operator — how it unwraps `Ok` values and returns early on `Err`, how it converts between error types via `From` implementations, and how it chains across multiple fallible operations. We saw how `.bind()` prevents SQL injection, how `query_as` provides type-safe database access, and how `fetch_optional` safely handles "not found" cases.

**Chapter 10: The Axum Router and Middleware** — We assembled the complete router with all routes organized by category (API, cache, tags, OPDS, legacy), understood the `run()` function's eight-step startup sequence, learned about TraceLayer for request logging and CorsLayer for cross-origin requests, saw how ServeDir handles static files and SPA routing with fallbacks, explored the Tower middleware ecosystem (request body limiting, compression, timeouts, custom middleware), understood middleware composition (onion layers), visualized the complete router tree, and examined the cache system with the `EType` enum and semaphore-based deduplication.

### Skills You've Gained

By the end of Part 2, you can:

- **Build a web server** with Axum that handles GET and POST requests
- **Extract parameters** from URLs (path params) and query strings
- **Share state** across handlers using `Arc<AppState>`
- **Configure your app** with environment variables and `.env` files
- **Handle errors gracefully** with `AppError` and the `?` operator
- **Connect to PostgreSQL** with connection pools
- **Create and run migrations** to manage your database schema
- **Map database rows to Rust structs** with `#[derive(FromRow)]`
- **Perform CRUD operations** safely with parameterized queries
- **Add middleware** for logging, CORS, compression, and more
- **Serve static files** for a single-page application

These are fundamental skills for any Rust web developer. The patterns you've learned — extractors, state sharing, error handling, connection pooling — apply to any Axum project, not just FicHub.

### What's Coming in Part 3

In Part 3, we'll build the SvelteKit frontend that talks to this backend. You'll learn:

- How to create a SvelteKit project with TypeScript
- How to fetch data from our Rust API endpoints
- How to display fanfic metadata in a beautiful UI
- How to handle loading states, errors, and edge cases
- How to build a search interface
- How to connect the frontend and backend in development and production

The backend we built in this part is the foundation — Part 3 builds the house on top of it. See you there! 🏠

---
