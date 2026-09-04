# Part 2: The Axum Web Server

---

Error handling is the foundation of reliable Rust code. Without it, programs crash. With it, they recover gracefully. The `AppError` enum is FicHub's central error type. The `From` trait lets us convert between error types automatically. The `?` operator propagates errors up the call stack. And `tracing` gives us visibility into what went wrong.

Now we take everything we've learned and build the actual web server — routes, handlers, extractors, and middleware. This is where FicHub comes alive. Every concept from Part 1 — ownership, error handling, traits, async — now has a purpose. We're not learning Rust in the abstract anymore. We're building something real.

---

# Chapter 6: Hello Axum

## What Is a Web Server?

A web server is a program that listens for HTTP requests and sends back HTTP responses. That's the simplest definition, and it's accurate. When you type `https://example.com` into your browser, your browser sends an HTTP GET request to the server at example.com. The server sends back an HTTP response containing the HTML of the homepage. Your browser renders it. That's the web.

HTTP — Hypertext Transfer Protocol — is the language of the web. Every HTTP request has:

- A **method** (GET, POST, PUT, DELETE) — what you want to do
- A **path** (like `/api/v0/epub`) — which resource you want
- **Headers** — metadata about the request (content type, authentication, etc.)
- A **body** (optional) — data you're sending (like a JSON payload)

Every HTTP response has:

- A **status code** (200 OK, 404 Not Found, 500 Internal Server Error)
- **Headers** — metadata about the response (content type, cache control, etc.)
- A **body** — the actual content (HTML, JSON, a file, etc.)

FicHub is a web server. When someone pastes a fanfiction URL into the FicHub interface, the browser sends an HTTP request to FicHub. FicHub scrapes the story, generates an EPUB, and sends back a JSON response with download links. The "web server" part is the piece that receives the request and sends back the response.

There are hundreds of web frameworks in different languages — Django and Flask in Python, Express in JavaScript, Gin in Go. In Rust, the most popular framework for building APIs is **Axum**.

## Why Axum?

Axum is a web framework built by the same team that created Tokio (the async runtime) and Tower (the middleware library). This means it fits perfectly into Rust's async ecosystem. Here's what makes it special:

- **Type-safe extractors** — instead of manually parsing request data, Axum uses Rust's type system to extract parameters automatically. If the data is missing or malformed, you get a 400 Bad Request without writing any validation code.
- **Composable middleware** — thanks to Tower, adding logging, CORS, or rate limiting is as simple as calling `.layer()`. No configuration objects, no decorator syntax, just function calls.
- **No magic** — Axum doesn't use macros or attribute decorators to define routes. It's plain Rust function calls. What you see is what you get.
- **Async-native** — built on Tokio from the ground up, so everything is non-blocking. When a handler waits for a database query, other requests keep being processed.

FicHub uses Axum 0.8, which is the current stable version. Let's get it running.

## Adding Axum to Cargo.toml

Open your `Cargo.toml` and add these dependencies:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Three crates, three purposes:

- **axum** — the web framework itself. It handles HTTP parsing, routing, and response generation.
- **tokio** — the async runtime. Axum requires it. The `full` feature enables everything: timers, networking, filesystem access, synchronization primitives.
- **serde** + **serde_json** — serialization and deserialization of JSON. You'll use these constantly for API responses.

After adding these, run `cargo build` to download and compile everything. The first build will take a few minutes (Rust is downloading and compiling hundreds of dependencies). Subsequent builds will be fast thanks to incremental compilation.

💡 **Key Concept: Why "Async"?**

An async web server can handle thousands of requests simultaneously. When a handler waits for a database query to complete (which takes milliseconds), the server doesn't sit idle — it processes other requests. This is like a chef who starts cooking the next dish while waiting for the oven to finish with the current one. Without async, the chef would stare at the oven doing nothing.

## Your First Axum Server

Create a new Rust project:

```bash
cargo new fichub-mini
cd fichub-mini
```

Replace the contents of `src/main.rs` with:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Build our application with routes
    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/health", get(health_handler));

    // Bind and serve
    let addr = "0.0.0.0:3000";
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn hello_handler() -> &'static str {
    "Hello, FicHub!"
}

async fn health_handler() -> &'static str {
    "OK"
}
```

To see the `tracing::info!` output, add `tracing` and `tracing-subscriber` to your dependencies:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

And initialize the subscriber in main (we'll cover this properly later):

```rust
tracing_subscriber::fmt()
    .with_env_filter("info")
    .init();
```

Run it:

```bash
cargo run
```

You should see:

```
2024-01-15T10:30:00Z  INFO fichub_mini: Listening on 0.0.0.0:3000
```

In another terminal, test it:

```bash
curl http://localhost:3000/hello
# Hello, FicHub!

curl http://localhost:3000/health
# OK
```

🎉 It works! You have a web server running. Type Ctrl+C in the terminal running the server to stop it.

## Understanding the Code

Let's break down what's happening line by line.

**1. The `#[tokio::main]` attribute:**

```rust
#[tokio::main]
async fn main() {
```

This is an attribute macro that transforms your `async fn main()` into a regular `fn main()` that creates a Tokio runtime and runs your async code inside it. Without Tokio, you can't use `async`/`await` for networking. Tokio is the engine that makes async Rust actually work. It manages a pool of threads and schedules async tasks across them.

**2. Creating the router:**

```rust
let app = Router::new()
    .route("/hello", get(hello_handler))
    .route("/health", get(health_handler));
```

`Router::new()` creates an empty router. Each `.route()` call adds a mapping: "when someone makes a GET request to this path, call this function." The `get()` function specifies that this route only handles GET requests (not POST, PUT, DELETE, etc.). You can also use `post()`, `put()`, `delete()`, and others.

The router is built by chaining method calls — each call returns the modified router, so you can keep adding routes.

**3. The handler functions:**

```rust
async fn hello_handler() -> &'static str {
    "Hello, FicHub!"
}
```

A handler is just an async function. Axum figures out what to do with the return value. Here we return `&'static str` (a string that lives for the entire program), and Axum converts it into a plain text HTTP response with status 200 and `Content-Type: text/plain`.

The function signature is simple because Axum uses the type system to understand what to do. Return `String`? Plain text. Return `Json<Value>`? JSON response. Return `(StatusCode, Json<Value>)`? Custom status code with JSON body.

**4. Starting the server:**

```rust
let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("Failed to bind to address");

axum::serve(listener, app)
    .await
    .expect("Server error");
```

We bind a TCP listener to port 3000 on all network interfaces (`0.0.0.0`). Then we pass it to `axum::serve` along with our router. From this point on, Axum handles incoming connections, parses HTTP requests, routes them to the right handler, and sends back responses. The `.await` calls are essential — they tell Tokio to wait for the operation to complete without blocking the thread.

⚠️ **Watch Out: Port Already in Use**

If you see `Error: Address already in use`, something else is running on port 3000. Either stop that process or change the port number:

```bash
# Find what's using port 3000
lsof -i :3000

# Kill it (be careful!)
kill -9 <PID>
```

Or just use a different port:

```rust
let addr = "0.0.0.0:3001";
```

## Adding JSON Responses

Plain text is nice, but FicHub deals in JSON. Let's update our handler to return JSON:

```rust
use axum::{routing::get, Json, Router};
use serde_json::json;

async fn hello_handler() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Hello, FicHub!",
        "version": "0.1.0",
        "status": "running"
    }))
}
```

Now `curl http://localhost:3000/hello` returns:

```json
{
    "message": "Hello, FicHub!",
    "version": "0.1.0",
    "status": "running"
}
```

The `Json()` wrapper tells Axum to set the `Content-Type: application/json` header. The `json!()` macro from `serde_json` creates a JSON value inline — it looks like a JavaScript object but it's real Rust that gets serialized to JSON at runtime.

Let's add more routes that return JSON, similar to how FicHub's API looks:

```rust
async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "uptime": "just started"
    }))
}

async fn info_handler() -> Json<serde_json::Value> {
    Json(json!({
        "name": "FicHub",
        "description": "Self-hosted fanfiction download server",
        "supported_sites": ["AO3", "FF.net", "XenForo"],
        "formats": ["EPUB", "HTML", "MOBI", "PDF"]
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/health", get(health_handler))
        .route("/api/info", get(info_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");

    tracing::info!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
```

Test each endpoint:

```bash
curl http://localhost:3000/hello | jq .
curl http://localhost:3000/health | jq .
curl http://localhost:3000/api/info | jq .
```

The `jq` command formats JSON nicely in the terminal.

💡 **Key Concept: `Json<T>` Wrapper**

`Json<T>` is an Axum type that does two things depending on context: when used as a return type (like `Json<Value>`), it serializes `T` into JSON for the response body and sets the `Content-Type: application/json` header. When used as a function parameter (which we'll see in the next chapter), it deserializes JSON from the request body into `T`. Same type, two uses — very Rust.

## Handling Different HTTP Methods

So far we've only used GET. But FicHub also needs POST for things like submitting tags and voting on recommendations. Let's add a POST endpoint:

```rust
use axum::{extract::Json, routing::get, Router};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct VoteRequest {
    fic_id: String,
    vote: i32,
}

async fn vote_handler(
    Json(payload): Json<VoteRequest>,
) -> Json<Value> {
    Json(json!({
        "err": 0,
        "msg": format!("Voted {} on fic {}", payload.vote, payload.fic_id),
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/api/vote", axum::routing::post(vote_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");

    tracing::info!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.expect("Server error");
}
```

Test with curl:

```bash
curl -X POST http://localhost:3000/api/vote \
  -H "Content-Type: application/json" \
  -d '{"fic_id": "abc123", "vote": 1}'
# {"err":0,"msg":"Voted +1 on fic abc123"}
```

Notice we use `axum::routing::post` instead of `get`. POST requests carry data in the body, not the URL. The `-H "Content-Type: application/json"` header tells the server we're sending JSON. The `-d` flag provides the JSON body.

## How HTTP Works Under the Hood

When you run `curl http://localhost:3000/hello`, here's what actually happens:

1. Your computer opens a TCP connection to localhost on port 3000
2. curl sends an HTTP request over that connection:
   ```
   GET /hello HTTP/1.1
   Host: localhost:3000
   Accept: */*
   ```
3. The server's TCP listener accepts the connection
4. Axum reads and parses the HTTP request
5. Axum matches the path `/hello` to the `hello_handler` function
6. The handler runs and returns `"Hello, FicHub!"`
7. Axum wraps it in an HTTP response:
   ```
   HTTP/1.1 200 OK
   Content-Type: text/plain

   Hello, FicHub!
   ```
8. The response is sent back over the TCP connection
9. curl displays the response body

This all happens in milliseconds. The beauty of Axum is that you don't need to think about steps 2-4 and 6-8. You just write the handler function and Axum handles everything else.

🧪 **Try It Yourself: Build a Mini API**

Create a new project with three endpoints:

```rust
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

async fn root() -> Json<Value> {
    json!({"message": "Welcome to FicHub!"})
}

async fn health() -> Json<Value> {
    json!({"status": "ok"})
}

async fn version() -> Json<Value> {
    json!({"version": "0.1.0", "rust": "2024 edition"})
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/version", get(version));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

Run it and test each endpoint with curl. Then try adding a 404 handler by returning a different status code.

## Summary

In this chapter, you learned:

- What a web server does: listen for HTTP requests, parse them, route them to handlers, send back responses
- How to add Axum, Tokio, serde, and tracing to a Rust project
- How to create a router with `Router::new()` and `.route()`
- How to write handler functions that return text or JSON
- How to start the server with `TcpListener` and `axum::serve`
- How `#[tokio::main]` enables async code by creating a Tokio runtime
- How HTTP works under the hood (request → parse → route → respond)

This is the skeleton of FicHub. In the next chapter, we'll learn how to extract parameters from requests — path segments, query strings, and JSON bodies. The routes will start to feel dynamic.

---

# Chapter 7: Request Parameters

## The Problem with Static Routes

In the last chapter, our routes looked like this:

```rust
.route("/hello", get(hello_handler))
.route("/health", get(health_handler))
```

These are **static routes** — they match exactly one URL. But FicHub needs dynamic routes. When someone requests a story, the URL looks like `/api/v0/epub?q=https://archiveofourown.org/works/12345`. The `q` parameter changes with every request.

Even more dynamic: the download route is `/cache/epub/abc123` — where `epub` is the export type and `abc123` is the story's unique ID. We can't create a separate route for every possible story ID. We need a way to extract parts of the URL.

Axum calls these **extractors**. They pull data out of requests — path segments, query parameters, JSON bodies, headers, and more. Extractors are Axum's killer feature. Instead of manually parsing URLs and request bodies, you describe what data you want in the type system, and Axum handles the rest.

## Path Parameters

Path parameters are placeholders in the URL. In Axum, you use curly braces `{name}` to mark them:

```rust
use axum::{
    extract::Path,
    routing::get,
    Json,
    Router,
};

async fn get_fic(Path(id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "fic_id": id,
        "title": "Example Story",
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/fics/{id}", get(get_fic));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Test it:

```bash
curl http://localhost:3000/fics/abc123
# {"fic_id":"abc123","title":"Example Story"}

curl http://localhost:3000/fics/xyz789
# {"fic_id":"xyz789","title":"Example Story"}
```

The `{id}` in the route pattern captures whatever segment appears at that position. Axum passes it to the handler as a `Path<String>` extractor. The `Path(id): Path<String>` syntax destructures it — `id` becomes a `String` containing the captured value.

What if someone requests `/fics/` (empty) or `/fics/a/b` (too many segments)? Axum handles this gracefully — it only matches when exactly one segment is present. No extra validation needed.

### Multiple Path Parameters

You can have multiple parameters. FicHub's download route needs both the export type and the story ID:

```rust
async fn download_fic(
    Path((etype, url_id)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "format": etype,
        "story_id": url_id,
        "message": "Download ready",
    }))
}

let app = Router::new()
    .route("/cache/{etype}/{url_id}", get(download_fic));
```

```bash
curl http://localhost:3000/cache/epub/abc123
# {"format":"epub","story_id":"abc123","message":"Download ready"}
```

When you have multiple path parameters, the `Path` extractor takes a tuple. The order matches the order in the route pattern — first `{etype}`, then `{url_id}`.

This is exactly how FicHub's cache download route works. The real code looks like this:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // Three path parameters: type, story ID, filename
}
```

### Typed Path Parameters

Path parameters don't have to be strings. Axum can parse them into numbers, and it will automatically return a 400 Bad Request if the conversion fails:

```rust
async fn get_chapter(Path(chapter_num): Path<u32>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "chapter": chapter_num,
        "content": format!("This is chapter {}", chapter_num),
    }))
}

let app = Router::new()
    .route("/chapters/{chapter_num}", get(get_chapter));
```

```bash
curl http://localhost:3000/chapters/5
# {"chapter":5,"content":"This is chapter 5"}

curl http://localhost:3000/chapters/not-a-number
# 400 Bad Request (automatic!)
```

This is the power of Axum's type-safe extractors. You don't write parsing code. You don't write validation code. You just declare the type you want, and Axum handles the rest. If the data is there and valid, you get a typed Rust value. If not, you get an error response.

## Query Parameters

Query parameters come after the `?` in a URL. They look like `?key=value&key2=value2`. In FicHub, the main export endpoint uses query parameters:

```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345
```

To extract query parameters, you define a struct with `#[derive(Deserialize)]` and use the `Query` extractor:

```rust
use axum::{
    extract::{Query, State},
    Json,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct ExportQuery {
    q: Option<String>,
    automated: Option<String>,
    format: Option<String>,
}

async fn epub_handler(
    Query(params): Query<ExportQuery>,
) -> Json<serde_json::Value> {
    let query = params.q.unwrap_or_default();
    if query.is_empty() {
        return Json(serde_json::json!({
            "err": -1,
            "msg": "no query provided"
        }));
    }

    Json(serde_json::json!({
        "err": 0,
        "q": query,
        "format": params.format.unwrap_or_else(|| "epub".to_string()),
    }))
}

let app = Router::new()
    .route("/api/v0/epub", get(epub_handler));
```

Test it:

```bash
# Good request
curl "http://localhost:3000/api/v0/epub?q=https://example.com/story"
# {"err":0,"q":"https://example.com/story","format":"epub"}

# With format parameter
curl "http://localhost:3000/api/v0/epub?q=https://example.com/story&format=html"
# {"err":0,"q":"https://example.com/story","format":"html"}

# Missing query
curl "http://localhost:3000/api/v0/epub"
# {"err":-1,"msg":"no query provided"}
```

Notice that all fields in `ExportQuery` are `Option<String>`. This means they're optional — the client doesn't have to provide them. If a field is missing from the query string, Axum sets it to `None`.

The real FicHub export handler follows this exact pattern:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Check for automated flag (block if present)
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // ... find scraper, look up metadata, etc.
}
```

### Required vs Optional Query Parameters

If you make a field non-optional, it becomes required. Axum will return a 400 Bad Request automatically if the field is missing:

```rust
#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,           // Required!
    page: Option<u32>,   // Optional
    limit: Option<u32>,  // Optional
}
```

If someone requests `/search` without the `q` parameter, Axum returns a 400 Bad Request automatically. No manual validation needed. The `serde` deserialization (which Axum uses internally) handles this.

### Query Parameters with Numbers

You can use typed fields in your query struct:

```rust
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_handler(
    Query(params): Query<PaginationQuery>,
) -> Json<serde_json::Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100); // Cap at 100

    Json(serde_json::json!({
        "page": page,
        "limit": limit,
        "results": []
    }))
}
```

```bash
curl "http://localhost:3000/list?page=2&limit=50"
# {"page":2,"limit":50,"results":[]}

curl "http://localhost:3000/list?page=abc"
# 400 Bad Request (page isn't a valid u32)
```

💡 **Key Concept: `Option<T>` Means Optional**

In Rust, `Option<T>` means "maybe present, maybe not." When you use `Option<String>` as a query parameter field, Axum parses it as `Some(value)` if present, or `None` if absent. Your handler code can then use patterns like `.unwrap_or_default()`, `.unwrap_or_else(|| ...)`, or `.as_deref()` to handle both cases gracefully. This is much better than checking for empty strings or null values.

## JSON Request Bodies

Some endpoints receive data in the request body as JSON. This is common for POST requests. FicHub uses JSON bodies for tag submissions and recommendation votes.

To extract a JSON body, use the `Json` extractor as a function parameter:

```rust
use axum::{
    extract::State,
    Json,
    Router,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct VoteRequest {
    fic_id: String,
    vote: i32,  // +1 or -1
}

async fn vote_handler(
    Json(payload): Json<VoteRequest>,
) -> Json<serde_json::Value> {
    // Validate the vote
    if payload.vote != 1 && payload.vote != -1 {
        return Json(serde_json::json!({
            "err": -1,
            "msg": "vote must be +1 or -1"
        }));
    }

    Json(serde_json::json!({
        "err": 0,
        "msg": format!("Voted {} on fic {}", payload.vote, payload.fic_id),
    }))
}

let app = Router::new()
    .route("/api/v0/recommendations/vote", axum::routing::post(vote_handler));
```

Test it:

```bash
curl -X POST http://localhost:3000/api/v0/recommendations/vote \
  -H "Content-Type: application/json" \
  -d '{"fic_id": "abc123", "vote": 1}'
# {"err":0,"msg":"Voted +1 on fic abc123"}
```

Notice we use `axum::routing::post` instead of `get` — POST requests carry data in the body, not the URL.

The JSON body is parsed automatically by Axum. If the JSON doesn't match the struct (wrong field types, missing required fields), Axum returns a 400 Bad Request. No manual parsing needed.

### Nested JSON Bodies

JSON bodies can be nested. Here's a tag submission example:

```rust
#[derive(Debug, Deserialize)]
struct TagSubmission {
    url_id: String,
    tags: Vec<TagEntry>,
}

#[derive(Debug, Deserialize)]
struct TagEntry {
    name: String,
    tag_type_id: i32,
}

async fn submit_tags(
    Json(payload): Json<TagSubmission>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "err": 0,
        "msg": format!("Submitted {} tags for {}", payload.tags.len(), payload.url_id),
    }))
}
```

The request body would be:

```json
{
    "url_id": "abc123",
    "tags": [
        {"name": "Harry Potter", "tag_type_id": 1},
        {"name": "Dramione", "tag_type_id": 3}
    ]
}
```

Serde handles all the nested parsing automatically.

⚠️ **Watch Out: Content-Type Header**

When sending JSON in a POST request, you **must** set the `Content-Type: application/json` header. Without it, Axum doesn't know the body is JSON and returns a 415 Unsupported Media Type error. In `curl`, use `-H "Content-Type: application/json"`. In JavaScript, use `fetch` with `headers: {'Content-Type': 'application/json'}`.

## Combining Multiple Extractors

Handlers can take multiple extractors. Axum extracts them in the order they appear as function parameters:

```rust
use axum::{
    extract::{Path, Query, State},
    Json,
    Router,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct DownloadQuery {
    h: Option<String>,  // optional hash
}

async fn download_handler(
    State(state): State<Arc<AppState>>,
    Path((etype, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "format": etype,
        "story_id": url_id,
        "hash": params.h,
    }))
}
```

Here we have three extractors:
1. `State` — the shared application state (we'll cover this fully in Chapter 10)
2. `Path` — captures `{etype}` and `{url_id}` from the URL
3. `Query` — captures the optional `h` parameter from the query string

Each extractor pulls its data from a different part of the HTTP request. Axum runs them all in parallel for performance.

The real FicHub download handler follows this exact pattern:

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

    let hash = match params.h {
        Some(h) => h,
        None => {
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    // ... serve the file or return error ...
}
```

### The Order of Extractors

In Axum 0.8, `State` can go anywhere in the parameter list. The key rule is that the `State` you pass to `Router::with_state()` must match the type in the handler. Don't overthink the ordering — just put extractors in a logical order that makes the code readable.

A common convention is:

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,    // 1. Shared state
    Path(id): Path<String>,                // 2. Path parameters
    Query(params): Query<MyQuery>,         // 3. Query parameters
    Json(body): Json<MyBody>,              // 4. Request body
) -> Result<Json<Value>, AppError> {
    // ...
}
```

But this is a convention, not a requirement. Put them in whatever order makes your handler most readable.

### Using Extractors with Different Combinations

Not every handler needs every extractor. Some handlers only need a query:

```rust
async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    // Just state and query
}
```

Some only need a path:

```rust
async fn get_tag(
    State(state): State<Arc<AppState>>,
    Path(tag_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    // Just state and path
}
```

Some don't need any extractors at all:

```rust
async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "FicHub API",
        "version": "0.1.0",
    }))
}
```

The flexibility is part of Axum's design. You declare exactly what data you need, and nothing more.

🧪 **Try It Yourself: Build a Search Endpoint**

Create a search endpoint that combines path and query parameters:

```rust
use axum::{extract::{Path, Query}, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,                    // Required: search term
    page: Option<u32>,            // Optional: page number
    limit: Option<u32>,           // Optional: results per page
}

async fn search_handler(
    Path(fandom): Path<String>,
    Query(params): Query<SearchQuery>,
) -> Json<Value> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20);

    Json(json!({
        "fandom": fandom,
        "query": params.q,
        "page": page,
        "limit": limit,
        "results": []
    }))
}

let app = Router::new()
    .route("/api/v0/fandoms/{fandom}/search", get(search_handler));
```

Test it:

```bash
curl "http://localhost:3000/api/v0/fandoms/harry-potter/search?q=draco&page=2&limit=10"
# {"fandom":"harry-potter","query":"draco","page":2,"limit":10,"results":[]}
```

Now try making `q` optional and handling the case where it's missing.

## The Request Lifecycle with Extractors

Let's trace what happens when a request arrives:

```
1. Client sends: GET /cache/epub/abc123?h=md5hash

2. Axum parses the HTTP request

3. Router matches: /cache/{etype}/{url_id}

4. Axum runs extractors:
   a. State → Arc<AppState> (shared state from router)
   b. Path → ("epub", "abc123") (from URL segments)
   c. Query → DownloadQuery { h: Some("md5hash") } (from ?h=...)

5. Handler receives typed Rust values:
   - state: &Arc<AppState>
   - etype: "epub" (String)
   - url_id: "abc123" (String)
   - h: Some("md5hash") (Option<String>)

6. Handler runs its logic and returns a response
```

This is the magic of Axum. The handler receives fully typed, validated data. It doesn't parse strings. It doesn't check for None. It just works with Rust values.

## Summary

In this chapter, you learned:

- **Path parameters** — `{name}` in route patterns, extracted with `Path<T>`. Supports single values, tuples for multiple params, and typed values like `Path<u32>`.
- **Query parameters** — `?key=value`, extracted with `Query<T>` from a `#[derive(Deserialize)]` struct. Use `Option<T>` for optional fields.
- **JSON bodies** — extracted with `Json<T>` from a `#[derive(Deserialize)]` struct. Supports nested structures and automatic validation.
- **Combining extractors** — multiple extractors in one handler, each pulling data from a different part of the request.
- **Automatic validation** — if data doesn't match the expected type, Axum returns a 400 Bad Request. No manual validation needed.

Extractors are the heart of Axum's design. Instead of manually parsing URLs and request bodies, you describe what data you want in the type system, and Axum handles the rest. If the data is missing or malformed, you get a 400 Bad Request automatically. If it's there, you get typed Rust values ready to use.

Next, we'll tackle configuration — how to load settings from environment variables so our server works in development and production.

---

# Chapter 8: Configuration

## Why Configuration Matters

So far, our server has hardcoded values everywhere. The port number is `3000`. The database URL would be `localhost`. The cache directory would be `./cache`. That's fine for a quick demo, but real applications need configuration.

Think about it: when you run FicHub on your laptop, the database is at `localhost:5432`. When you deploy it to a server, the database might be at `db.internal.fichub.net:5432`. When you deploy to production, it's at `fichub-db.rds.amazonaws.com:5432` with a password you definitely don't want in your source code.

You don't want to change the source code every time you move servers. That's fragile, error-prone, and a security disaster.

The solution is **configuration** — loading settings from the environment instead of hardcoding them. The Twelve-Factor App methodology (a set of best practices for modern software, written in 2011 and still relevant) says: store configuration in the environment. This means your application reads settings from environment variables, not from files or constants.

Why? Because the same binary can run anywhere — your laptop, a staging server, production — just by changing the environment variables. The code stays the same. Only the configuration changes.

## Loading Environment Variables with dotenvy

Environment variables are key-value pairs set in your shell. They're the standard way to configure applications:

```bash
export DATABASE_URL="postgres://localhost/fichub"
export REDIS_URL="redis://localhost/0"
export PORT=3000
```

Rust reads them with `std::env::var()`:

```rust
let port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string());
```

But managing environment variables in your shell is tedious. You have to type `export` for every variable, every time you open a new terminal. And if you forget one, your application crashes with a confusing error.

That's where **dotenvy** comes in. dotenvy (a maintained fork of the `dotenv` crate) loads a `.env` file and sets those values as environment variables. No more `export` commands. Just put your configuration in a file and dotenvy does the rest.

Add dotenvy to `Cargo.toml`:

```toml
[dependencies]
dotenvy = "0.15"
```

Create a `.env` file in your project root:

```env
DATABASE_URL=postgres://localhost/fichub
REDIS_URL=redis://localhost/0
PORT=3000
CACHE_DIR=./cache
```

In `main.rs`, load it before anything else:

```rust
fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Now environment variables are set
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    println!("Port: {}", port);
}
```

The `.ok()` at the end is important. `dotenvy::dotenv()` returns a `Result`. If there's no `.env` file (like on a production server where you use real environment variables), it would return `Err`. By calling `.ok()`, we ignore the error — the `.env` file is optional.

This is exactly how FicHub does it in `main.rs`:

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();
    // ...
}
```

⚠️ **Watch Out: .env Files and Security**

Never commit your `.env` file to git! It might contain database passwords, API keys, or other secrets. Add it to `.gitignore`:

```bash
echo ".env" >> .gitignore
```

FicHub ships a `.env.example` file with placeholder values. Developers copy it to `.env` and fill in their own settings:

```bash
cp .env.example .env
# Edit .env with your actual settings
```

The `.env.example` file looks like:

```env
DATABASE_URL=postgres://fichub:fichub@localhost/fichub
REDIS_URL=redis://localhost/0
PORT=3000
CACHE_DIR=./cache
FRONTEND_DIR=./frontend/build
NODE_NAME=dev
```

## Creating a Config Struct

Reading individual environment variables everywhere is messy. If you have 30 environment variables and you read them in 10 different files, that's 300 calls to `std::env::var()`. When you want to rename a variable, you have to find and replace in 300 places. That's a nightmare.

Instead, load them once into a `Config` struct. This is the single source of truth for all configuration:

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
}
```

Then implement a `from_env()` method that reads everything from the environment:

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
        }
    }
}
```

The struct has `#[derive(Debug, Clone)]` so we can print it for debugging and clone it when needed. The `Clone` derive is important because we'll pass the config through `Arc`, and `Arc` needs its inner type to be accessible from multiple places.

### Required vs Optional Variables

Look at the different patterns for reading environment variables:

**Required variables** use `.expect()`:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

If `DATABASE_URL` isn't set, the program panics immediately with a clear message: `"DATABASE_URL must be set"`. This is correct for settings that **must** be configured — like database credentials. There's no sensible default for a database URL. If you can't connect to the database, the server can't function, so panicking early is the right behavior.

**Optional variables with defaults** use `.unwrap_or_else()`:

```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());
```

If `CACHE_DIR` isn't set, it falls back to `"./cache"`. This is good for settings that have a reasonable default. Most developers are fine with the cache being in `./cache` during development.

**Truly optional variables** use `.ok()`:

```rust
let secondary_cache = std::env::var("SECONDARY_CACHE_DIR")
    .ok()
    .filter(|s| !s.is_empty())
    .map(PathBuf::from);
```

This returns `Option<PathBuf>` — `Some(path)` if the env var is set and non-empty, `None` otherwise. The `.filter(|s| !s.is_empty())` catches the case where someone sets the variable to an empty string. This is useful for features that aren't always enabled.

### Parsing Numbers

Environment variables are always strings. When you need a number, use `.parse()`:

```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()
    .unwrap_or(3000);
```

The `.parse()` method converts a string to any type that implements `FromStr`. If the string isn't a valid number (like `PORT=abc`), it returns `Err`, and `.unwrap_or(3000)` falls back to the default.

### Parsing Booleans

Same idea for booleans:

```rust
let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
    .unwrap_or_else(|_| "true".to_string())
    .parse::<bool>()
    .unwrap_or(true);
```

Now `DYNAMIC_RATE_LIMIT=false` disables dynamic rate limiting, and anything else (or unset) enables it.

### Parsing Complex Types

For more complex types, you can parse JSON:

```rust
let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
    .unwrap_or_else(|_| "{}".to_string());
let rec_site_rate_limits: HashMap<String, u64> =
    serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();
```

This lets you set `REC_SITE_RATE_LIMITS={"ao3": 10, "ffn": 5}` in your `.env` file. The value is parsed as a JSON string and deserialized into a `HashMap`. If the JSON is invalid, it falls back to an empty map.

### Parsing Comma-Separated Lists

For lists of values:

```rust
let trusted_proxies = std::env::var("TRUSTED_PROXIES")
    .unwrap_or_default()
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
```

Set `TRUSTED_PROXIES=10.0.0.1,10.0.0.2` and you get a `Vec<String>` with two elements.

## The Real FicHub Config

FicHub's `Config` struct is much bigger than our simplified version. It has settings for the database, Redis, caching, rate limiting, recommendations, tagging, and more. Let's look at the real thing:

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

That's a lot of fields! Each one has a corresponding environment variable. Required settings use `.expect()`. Optional settings use `.unwrap_or_else()` with sensible defaults. This struct is the single source of truth for all configuration — every part of FicHub reads from it.

The `.env` file for FicHub might look like:

```env
DATABASE_URL=postgres://fichub:fichub@localhost/fichub
REDIS_URL=redis://localhost/0
CACHE_DIR=/var/cache/fichub
PORT=3000
NODE_NAME=orion
EXPORT_VERSION=1
DYNAMIC_RATE_LIMIT=true
FRONTEND_DIR=./frontend/build
REC_DEFAULT_DELAY_SECS=5
REC_MAX_RECOMMENDATIONS=20
REC_PRECOMPUTE_ENABLED=true
TAG_HIDDEN_THRESHOLD=-3
SEARCH_MAX_PER_PAGE=50
```

Each line is a key-value pair. dotenvy loads them all, and `Config::from_env()` reads them into the struct.

## Using the Config

Once loaded, the `Config` is passed through the application. In `main.rs`:

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

    tracing::info!("Starting fichub server on port {}", config.app_port);

    server::run(config).await;
}
```

The config flows into `server::run()`, which uses it to:
- Connect to PostgreSQL with `config.database_url`
- Connect to Redis with `config.redis_url`
- Store cached files in `config.cache_dir`
- Bind the TCP listener to `config.app_port`
- Serve frontend files from `config.frontend_dir`

Every part of the application reads from the same `Config` instance. No magic globals, no hidden state. Just a struct passed through the call chain.

🧪 **Try It Yourself: Add a Custom Config**

Add an `APP_NAME` environment variable to your config:

```rust
pub struct Config {
    pub database_url: String,
    pub app_port: u16,
    pub app_name: String,  // NEW
}

impl Config {
    pub fn from_env() -> Self {
        // ... other fields ...

        let app_name = std::env::var("APP_NAME")
            .unwrap_or_else(|_| "FicHub".to_string());

        Config {
            // ... other fields ...
            app_name,
        }
    }
}
```

Add it to your `.env` file:

```env
APP_NAME=My Custom FicHub
```

Use it in a handler:

```rust
async fn info_handler(State(state): State<Arc<Config>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": state.app_name,
        "port": state.app_port,
    }))
}
```

## Testing Configuration

FicHub has tests for its configuration. This is important because if the config is wrong, the whole server breaks. Here's how to test env var loading:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_defaults() {
        // Set required env vars
        unsafe { std::env::set_var("DATABASE_URL", "postgres://localhost/test_db"); }
        unsafe { std::env::set_var("REDIS_URL", "redis://localhost/0"); }

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://localhost/test_db");
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert_eq!(config.app_port, 3000);

        // Clean up
        unsafe { std::env::remove_var("DATABASE_URL"); }
        unsafe { std::env::remove_var("REDIS_URL"); }
    }

    #[test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    fn test_panics_without_database_url() {
        unsafe { std::env::remove_var("DATABASE_URL"); }
        unsafe { std::env::set_var("REDIS_URL", "redis://localhost"); }
        let _ = Config::from_env();
    }
}
```

The `#[should_panic]` attribute verifies that the function panics with the expected message when `DATABASE_URL` is missing. This ensures required variables are actually required.

## Summary

In this chapter, you learned:

- **Environment variables** are the standard way to configure applications — they separate config from code
- **dotenvy** loads a `.env` file so you don't have to `export` everything manually
- A **Config struct** centralizes all settings in one place — single source of truth
- **Required variables** use `.expect()` to panic with a clear message if missing
- **Optional variables** use `.unwrap_or_else()` or `.ok()` for graceful defaults
- **.parse()** converts strings to numbers, booleans, and other types
- **Complex types** can be parsed from JSON or comma-separated strings
- **Testing** config loading ensures required vars are enforced and defaults work

Configuration is boring but essential. Get it right early and you'll save yourself hours of debugging later. In the next chapter, we'll revisit error handling — this time in the context of Axum, where errors need to become HTTP responses.

---

# Chapter 9: Error Handling in Axum

## From Errors to HTTP Responses

In Part 1, we built an `AppError` enum that represents all the ways FicHub can fail. But errors in a library and errors in a web server are different things. In a library, an error might print to stderr or propagate to the caller. In a web server, an error must become an HTTP response — with a status code, a JSON body, and the right headers.

The user can't see your terminal. They can't read your Rust error messages. When something goes wrong, they need a clear, structured JSON response that tells them what happened and what to do about it. And the HTTP status code needs to be correct so that browsers, API clients, and monitoring tools can understand what happened.

Axum's rule is simple: a handler must return something that implements `IntoResponse`. That could be:
- A `String` → plain text response
- A `Json<T>` → JSON response with 200 status
- A `(StatusCode, Json<T>)` → custom status code + JSON
- A `Response` → full control over the response
- A custom type that implements `IntoResponse`

Since our handlers return `Result<Json<Value>, AppError>`, we need to teach Axum how to turn an `AppError` into a response. This is done by implementing the `IntoResponse` trait.

## Implementing IntoResponse

Here's the `AppError` enum from FicHub:

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

Each variant represents a different category of failure. `BadRequest` means the client sent something invalid. `NotFound` means the requested resource doesn't exist. `Internal` means something went wrong on the server side. And so on.

Now implement `IntoResponse`:

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
                    "retry_after": retry_after,
                }))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "internal server error",
                }))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -5,
                    "msg": "export failed",
                }))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "database error",
                }))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "cache error",
                }))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

This is the magic bridge between Rust errors and HTTP responses. When a handler returns `Err(AppError::NotFound("story not found".into()))`, Axum calls this `into_response` method, which turns it into an HTTP 404 response with a JSON body. The client never sees Rust error types — they see a clean JSON response.

## Understanding the Pattern

Let's break down what's happening in detail.

**1. Each error variant maps to an HTTP status code:**

| Error Variant | HTTP Status | Error Code | Why |
|---|---|---|---|
| `BadRequest` | 400 Bad Request | caller-specified | Client sent invalid input |
| `RateLimited` | 429 Too Many Requests | -429 | Too many requests, slow down |
| `NotFound` | 404 Not Found | -5 | Resource doesn't exist |
| `Internal` | 500 Internal Server Error | -1 | Something broke on our side |
| `ScrapeError` | 502 Bad Gateway | -6 | Upstream site returned an error |
| `ExportError` | 500 Internal Server Error | -5 | EPUB generation failed |
| `Database` | 500 Internal Server Error | -1 | Database query failed |
| `CacheError` | 500 Internal Server Error | -1 | Redis operation failed |

The status codes follow HTTP conventions:
- **4xx** = client error (the client did something wrong)
- **5xx** = server error (something broke on our side)
- **429** = rate limiting (special case of client error)

**2. Internal errors are logged but not exposed:**

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);
    (StatusCode::INTERNAL_SERVER_ERROR, json!({
        "err": -1,
        "msg": "internal server error",
    }))
}
```

The `msg` is logged with `tracing::error!` for debugging — the developer can see what went wrong in the logs. But the client only sees a generic "internal server error." You never want to leak internal details (like database connection strings, file paths, or stack traces) to the user. That's a security risk.

**3. Client-facing errors include useful messages:**

```rust
AppError::BadRequest(code, msg) => {
    (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
}
```

For `BadRequest`, both the error code and message are sent to the client — because the client needs to know what they did wrong to fix it. If someone sends an unsupported URL, the error message tells them exactly what's wrong.

**4. Scrape errors use 502 Bad Gateway:**

```rust
AppError::ScrapeError(msg) => {
    (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
}
```

502 means "the server received an invalid response from an upstream server." When FicHub tries to scrape AO3 and AO3 returns a 403 or times out, that's a scrape error. The problem isn't with FicHub or the client — it's with the upstream site.

## Using AppError in Handlers

Now handlers can use `AppError` naturally with the `?` operator:

```rust
async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    // This might return AppError::BadRequest
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    // This might return AppError::ScrapeError
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // This might return AppError::Database (via From impl)
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;

    if !fic_blacklist.is_empty() {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }

    Ok(Json(json!({
        "err": 0,
        "info": format!("{} by {}", meta.title, meta.author),
    })))
}
```

The flow is clean and readable:

1. Check for empty query → return `Ok` with an error JSON (handled gracefully, not a true error)
2. Find scraper → if not found, return `Err(BadRequest)` with a helpful message
3. Lookup metadata → if it fails, return `Err(ScrapeError)` with the error details
4. Check database → if it fails, return `Err(Database)` via the `From` impl
5. Check blacklists → if blacklisted, return `Ok` with error info (not a true error)
6. Return success

The `?` operator is doing the heavy lifting. When a database query fails with `sqlx::Error`, the `?` operator automatically converts it to `AppError::Database(...)` using the `From` trait. No manual mapping needed.

## The From Trait: Automatic Error Conversion

The `From` implementations are what make the `?` operator work:

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

Each `From` implementation maps a library-specific error type to the appropriate `AppError` variant. When you write `queries::check_fic_blacklist(&state.db, &url).await?`, the `?` operator:
1. Checks if the result is `Err`
2. If so, calls `AppError::from(err)` using the `From` implementation
3. Returns the `AppError` from the handler

This is the Rust error handling pattern at its finest. Error propagation is automatic, type-safe, and explicit. You can see every possible error in the function signature.

💡 **Key Concept: `Result<T, AppError>` as a Return Type**

When a handler returns `Result<Json<Value>, AppError>`, Axum knows:
- On `Ok(Json(value))` → send a 200 OK with the JSON body
- On `Err(app_error)` → call `app_error.into_response()` to get the status code and error body

This is the standard pattern for Axum error handling. Every FicHub handler uses it. The `Result` type makes errors explicit in the function signature — you can see at a glance that the handler can fail.

## The Standard API Pattern

FicHub follows a consistent pattern for API responses. Every response has an `err` field:

```json
{
    "err": 0,     // 0 = success, negative = error code
    "msg": "...", // human-readable message (optional on success)
    // ... additional data
}
```

Success responses:

```json
{"err": 0, "info": "My Story by Author", "url_id": "abc123"}
```

Error responses:

```json
{"err": -5, "msg": "unsupported URL: not-a-real-site.com"}
```

Rate limit responses include a retry hint:

```json
{"err": -429, "msg": "rate limited", "retry_after": 30}
```

This convention makes it easy for clients to check for errors — just look at the `err` field. The HTTP status code provides the machine-readable classification, and the `err` field provides the application-specific code. Both are useful for different purposes.

## Type Alias for Convenience

To avoid typing `Result<Json<Value>, AppError>` everywhere, FicHub defines a type alias:

```rust
/// Standard API result type
pub type AppResult<T> = Result<T, AppError>;
```

Now handlers can write:

```rust
async fn epub_handler(...) -> AppResult<Json<Value>> {
    // ...
}
```

Much cleaner. The type alias also communicates intent — this is a "standard result" for the FicHub API.

## Display for Debugging

The `AppError` also implements `Display` for human-readable output:

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

This is used by `tracing::error!("Error: {}", err)` and for debugging output. It gives a human-readable representation of the error.

🧪 **Try It Yourself: Add a Custom Error**

Add a new error variant to `AppError`:

```rust
#[derive(Debug)]
pub enum AppError {
    // ... existing variants ...
    Unauthorized(String),
}
```

Implement its `IntoResponse`:

```rust
AppError::Unauthorized(msg) => {
    (StatusCode::UNAUTHORIZED, json!({"err": -3, "msg": msg}))
}
```

And its `Display`:

```rust
AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
```

Use it in a handler:

```rust
async fn admin_handler(
    Query(params): Query<AdminQuery>,
) -> Result<Json<Value>, AppError> {
    let token = params.token.as_deref()
        .ok_or_else(|| AppError::Unauthorized("no token provided".into()))?;

    if token != "secret" {
        return Err(AppError::Unauthorized("invalid token".into()));
    }

    Ok(Json(json!({"err": 0, "msg": "welcome, admin"})))
}
```

Test it:

```bash
curl "http://localhost:3000/admin"
# {"err":-3,"msg":"no token provided"}  (401)

curl "http://localhost:3000/admin?token=wrong"
# {"err":-3,"msg":"invalid token"}  (401)

curl "http://localhost:3000/admin?token=secret"
# {"err":0,"msg":"welcome, admin"}  (200)
```

## Error Handling Best Practices

Based on the FicHub codebase, here are the patterns to follow:

**1. Always log internal errors:**

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);  // Log for debugging
    (StatusCode::INTERNAL_SERVER_ERROR, json!({
        "err": -1,
        "msg": "internal server error",  // Generic message for client
    }))
}
```

**2. Don't leak internal details:**

```rust
// BAD - exposes database error to client
AppError::Database(msg) => {
    (StatusCode::INTERNAL_SERVER_ERROR, json!({"msg": msg}))
}

// GOOD - generic message, full error in logs
AppError::Database(msg) => {
    tracing::error!("Database error: {}", msg);
    (StatusCode::INTERNAL_SERVER_ERROR, json!({
        "err": -1,
        "msg": "database error",
    }))
}
```

**3. Use specific error codes for client errors:**

```rust
// Client needs to know what they did wrong
AppError::BadRequest(-5, "unsupported URL: ...".into())
AppError::BadRequest(-10, "automated requests blocked".into())
```

**4. Return `Ok` for expected "errors":**

Sometimes a "bad" situation isn't really an error — it's just the normal flow. For example, a blacklisted fic isn't a server error:

```rust
// This is OK - not a true error
if !fic_blacklist.is_empty() {
    return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
}
```

The client can check the `err` field and handle it accordingly.

## Summary

In this chapter, you learned:

- Handlers must return types that implement `IntoResponse`
- `IntoResponse for AppError` maps each error variant to an HTTP status code + JSON body
- Internal errors are logged but not exposed to clients (security)
- Client-facing errors include helpful messages so the client can fix the problem
- The `?` operator + `From` trait lets errors flow naturally through the call stack
- The `Result<T, AppError>` pattern is the standard way to handle errors in Axum
- The `AppResult<T>` type alias makes handler signatures cleaner
- Consistent error responses (with `err` and `msg` fields) make client code simpler

Error handling is the safety net that keeps FicHub running smoothly. With `AppError`, every failure is caught, logged, and returned as a meaningful HTTP response. No panics in production, no mysterious 500 errors, no silent failures.

In the next chapter, we'll tackle shared state — how to give all your handlers access to the database, HTTP client, and configuration.

---

# Chapter 10: Shared State

## The Problem

Our handlers need access to things that don't come from the HTTP request: the database pool, the HTTP client, the scraper registry, the configuration. These are **shared resources** — the same database connection pool is used by every handler for every request.

But there's a catch. Axum runs handlers on Tokio's async runtime, which might execute them on different threads. If two handlers try to access the same `Config` at the same time, we need to make sure that's safe. In Rust, sharing mutable data across threads requires special handling — this is the ownership system at work.

You might think: "Just make a global variable." But Rust doesn't have global mutable state for good reasons — it's a recipe for race conditions, data corruption, and bugs that are nearly impossible to reproduce. Instead, Rust gives us `Arc<T>` — a safe, thread-friendly way to share data.

## Arc<T>: Thread-Safe Sharing

The solution is `Arc<T>` — **Atomically Reference Counted**. Think of it as a shared pointer that multiple owners can hold simultaneously. When the last owner is dropped, the data is cleaned up automatically.

```rust
use std::sync::Arc;

let config = Arc::new(Config::from_env());

// Clone doesn't copy the data — it creates a new pointer to the same data
let config_clone = Arc::clone(&config);
```

`Arc::clone()` is cheap — it just increments an atomic counter (the reference count). The actual `Config` data is shared, not copied. When the last `Arc` pointing to the data is dropped, the data is freed.

This is how FicHub shares its configuration, database pool, and HTTP client across all handlers. Every handler gets its own `Arc` pointer to the same underlying data.

💡 **Key Concept: `Arc<T>` vs `Rc<T>`**

There's also `Rc<T>` (Reference Counted), which does the same thing but without thread safety. Since Axum handlers run on Tokio's multi-threaded runtime, you **must** use `Arc<T>`, not `Rc<T>`. Using `Rc` in async code causes a compile error — Rust's type system catches the mistake before it ever reaches production.

The `A` in `Arc` stands for "atomic" — it uses atomic operations for the reference count, which are safe to use across threads.

## Creating AppState

FicHub defines an `AppState` struct that holds all shared resources:

```rust
use std::sync::Arc;

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

Each field is a shared resource:
- **config** — all the settings we loaded from environment variables
- **db** — a PostgreSQL connection pool (manages multiple connections automatically)
- **redis** — an async Redis connection for caching and rate limiting
- **http_client** — a reusable HTTP client for scraping fanfiction sites
- **scraper_registry** — the map of URL patterns to scrapers
- **cache_semaphores** — prevents duplicate concurrent exports of the same story
- **rate_limiter** — per-IP rate limiting with Redis
- **recommender_engine** — the recommendation algorithm
- **collection_worker** — background worker for collecting user favourites

The `AppState` is created once at startup and wrapped in `Arc`:

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

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new()
        )),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });

    // Build and run the router
    let app = build_router(state).await;
    // ...
}
```

Notice how many resources are created at startup:
1. Database pool with 20 max connections
2. Redis multiplexed connection
3. HTTP client with user agent and timeout
4. Scraper registry (auto-detects which sites are supported)
5. Rate limiter (connected to Redis)
6. Cache semaphores (prevents duplicate exports)

All of these are created once and shared across all requests. This is the "initialize once, use everywhere" pattern.

## The State Extractor

To use `AppState` in a handler, add `State(state): State<Arc<AppState>>` as a parameter:

```rust
async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // Access the config
    tracing::info!("Export version: {}", state.config.export_version);

    // Access the database
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;

    // Access the HTTP client
    let response = state.http_client.get(&url).send().await?;

    // Access the scraper registry
    let scraper = state.scraper_registry.find_scraper(&query)
        .ok_or_else(|| AppError::BadRequest(-5, "unsupported URL".into()))?;

    Ok(Json(json!({"err": 0})))
}
```

The `State` extractor pulls the `Arc<AppState>` from the router and passes it to the handler. From there, the handler can access any shared resource through `state.config`, `state.db`, `state.http_client`, etc.

The `State` extractor is special — it doesn't come from the HTTP request. It comes from the router's configuration. This is dependency injection: the handler declares what it needs, and the router provides it.

### Connecting State to the Router

The state is passed to the router with `.with_state()`:

```rust
let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .route("/api/v0/remote", get(remote_handler))
    .route("/api/v0/search", get(search_handler))
    .with_state(state);
```

Every handler registered on this router can now use the `State` extractor. The state is shared — all handlers see the same `AppState`. If one handler updates the database, all subsequent handlers see the update.

## Connecting to a Database Pool

SQLx's `PgPool` is already thread-safe and cloneable (it wraps an `Arc` internally). This makes it perfect for shared state:

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
}
```

The pool manages a set of database connections. When a handler needs to run a query, it gets a connection from the pool. When the query finishes, the connection goes back to the pool. This is much more efficient than creating a new connection for every request.

The `max_connections(20)` setting means at most 20 queries can run simultaneously. If more come in, they wait for a connection to become available. This prevents the database from being overwhelmed.

The `acquire_timeout(Duration::from_secs(10))` means if all connections are busy, the handler waits up to 10 seconds before failing. This is better than failing immediately — most of the time, a connection becomes available within milliseconds.

## Why Arc<AppState> and Not Just AppState?

You might wonder why we wrap `AppState` in `Arc` when it already contains owned data. The reason is that Axum clones the state for each request. Without `Arc`, each clone would be a full copy of all the data — copying the database pool, the HTTP client, the config, etc. That's expensive and wasteful.

With `Arc`, each clone is just a pointer increment — nanoseconds instead of milliseconds. The `AppState` struct itself doesn't need to be `Clone` because `Arc` handles the sharing. But individual fields like `Config`, `PgPool`, and `reqwest::Client` do need to be cheap to clone (which they are — they're internally `Arc`-wrapped).

This is a common pattern in Rust async applications: wrap your shared state in `Arc` and pass it through the call chain.

## The Cloning Chain

Let's trace how the state flows through the application:

```
1. main() creates Config from env vars
2. server::run() creates AppState with Config + db + redis + ...
3. Arc::new(AppState { ... }) wraps it
4. build_router(state) receives Arc<AppState>
5. Router::with_state(state) stores it in the router
6. For each request:
   a. Axum clones the Arc (pointer increment)
   b. Passes it to the handler via State extractor
   c. Handler accesses state.db, state.config, etc.
   d. Arc is dropped when the request completes
```

The `AppState` is never copied. Only `Arc` pointers are cloned. The actual data lives in a single allocation on the heap, shared by all handlers.

🧪 **Try It Yourself: Add a Counter to State**

Create a simple shared counter using `AtomicUsize`:

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct AppState {
    pub request_count: AtomicUsize,
}

// In your main function:
let state = Arc::new(AppState {
    request_count: AtomicUsize::new(0),
});

// In a handler:
async fn counter_handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let count = state.request_count.fetch_add(1, Ordering::Relaxed);
    Json(serde_json::json!({
        "requests_served": count + 1,
    }))
}
```

Each request increments the counter atomically. Visit the endpoint a few times and watch the count go up. This demonstrates how shared state works across concurrent requests.

## Shared State in Practice

Here's how FicHub handlers use the shared state in practice:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    // 1. Use config
    let query = params.q.as_deref().unwrap_or("");

    // 2. Use scraper registry + HTTP client
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // 3. Use database
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;

    // 4. Use config again
    let version = state.config.export_version;

    // 5. Use database for cache lookup
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;

    Ok(Json(json!({"err": 0})))
}
```

One handler, four different shared resources, all accessed through `state`. Clean, explicit, and efficient.

## Summary

In this chapter, you learned:

- **Arc<T>** provides thread-safe shared ownership of data — multiple handlers can read the same data without copying it
- **AppState** is a struct that holds all shared resources (config, database, HTTP client, scraper registry, etc.)
- The **State extractor** passes `Arc<AppState>` to handlers automatically
- **`.with_state()`** connects the state to the router
- **PgPool** manages database connections efficiently — connection pooling prevents exhausting the database
- **Arc::clone()** is cheap — it's just a pointer increment, not a data copy
- All handlers share the same state — they see the same database, the same config, the same HTTP client

Shared state is how FicHub connects everything together. The database pool, the scraper registry, the rate limiter — they all live in `AppState` and are accessible to every handler. This is the "dependency injection" pattern in Rust: handlers declare what they need via the `State` extractor, and the router provides it.

In the next chapter, we'll add middleware — the invisible layer that logs requests, adds CORS headers, and handles cross-cutting concerns.

---

# Chapter 11: Middleware

## What Is Middleware?

Imagine you're running a pizza shop. Every order goes through the same process: the cashier takes the order, gives you a receipt, and the kitchen makes the pizza. The "receipt" step happens for *every* order, regardless of what pizza you ordered. You wouldn't write the receipt logic inside each pizza recipe — that would be duplicated and error-prone.

Middleware is like that receipt step in web development. It's code that runs **before** (or after) every request, regardless of which route handles it. Common middleware tasks include:

- **Logging** — record every request and response for monitoring
- **CORS** — add headers so browsers allow cross-origin requests
- **Compression** — gzip responses to save bandwidth
- **Authentication** — check for valid tokens before reaching handlers
- **Rate limiting** — prevent abuse by limiting requests per IP

The beauty of middleware is that it's reusable. Write the logging middleware once, and it applies to every route. No copy-pasting. No forgetting to add it to new routes. It's just there, working, for everything.

In Axum, middleware comes from **Tower** and **tower-http**. It's applied using the `.layer()` method on the router. Think of layers as transparent sheets stacked on top of each other — each one can see and modify the request as it passes through.

## TraceLayer: Request Logging

The most useful middleware for a web server is request logging. Tower provides `TraceLayer` which logs every incoming request and outgoing response:

```rust
use tower_http::trace::TraceLayer;

let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .layer(TraceLayer::new_for_http());
```

Now every request gets logged automatically:

```
2024-01-15T10:30:00Z  INFO request{method=GET uri=/api/v0/epub}: tower_http::trace::on_request: started processing request
2024-01-15T10:30:00Z  INFO request{method=GET uri=/api/v0/epub}: tower_http::trace::on_response: finished processing request latency=42 ms status=200
```

You can see the HTTP method, the URL path, how long it took, and the status code. This is invaluable for debugging ("why is this endpoint slow?") and monitoring ("how many requests are we getting?").

The `TraceLayer` uses `tracing` under the hood, so it integrates with the same logging system we set up in `main.rs`. If you configured a JSON log format, these traces become structured log entries that you can query in your monitoring tool.

## CorsLayer: Cross-Origin Requests

CORS (Cross-Origin Resource Sharing) is a browser security feature. When your frontend (running at `https://fichub.example.com`) makes an API request to your backend (running at `https://api.fichub.example.com`), the browser checks if the backend allows it. Without CORS headers, the browser blocks the request with a cryptic error in the console.

```rust
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(CorsLayer::permissive());
```

`CorsLayer::permissive()` allows all origins, methods, and headers. It's the simplest setup for development. In production, you'd restrict it:

```rust
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any);
```

This allows only GET and POST methods, which is what FicHub's API uses.

⚠️ **Watch Out: Permissive CORS in Production**

`CorsLayer::permissive()` is fine for development, but in production you should restrict `allow_origin` to your actual frontend domain. Otherwise, any website could make requests to your API — a potential security issue. For example:

```rust
let cors = CorsLayer::new()
    .allow_origin("https://fichub.example.com".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any);
```

Only requests from `fichub.example.com` will be allowed.

## Layer Ordering Matters

Here's something subtle but important: **the order you add layers matters**. Layers are applied in reverse order — the last layer added runs first.

```rust
let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(TraceLayer::new_for_http())   // Applied second (runs second)
    .layer(CorsLayer::permissive());     // Applied first (runs first)
```

Wait — that's the opposite of what you might expect. Let me clarify:

The `.layer()` method adds layers to the **outside** of the stack. The last `.layer()` call adds the outermost layer. When a request comes in, it goes through layers from outermost to innermost.

So in the code above:
1. CORS runs first (adds CORS headers to the response)
2. Trace runs second (logs the request and response)

In FicHub, the order is:

```rust
Router::new()
    // ... routes ...
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(state)
```

This means CORS runs first, then tracing. Both are applied to all routes.

💡 **Key Concept: The Middleware Stack**

Think of middleware as layers of an onion. A request enters from the outside (first layer) and works its way in toward the handler (innermost layer). The response then travels back out, passing through the layers in reverse order. Each layer can modify the request or response, or just observe it.

```
Request
  │
  ▼
┌─────────────────┐
│   CorsLayer     │  ← Adds CORS headers
└─────────────────┘
  │
  ▼
┌─────────────────┐
│  TraceLayer     │  ← Logs the request
└─────────────────┘
  │
  ▼
┌─────────────────┐
│    Handler      │  ← Your actual code
└─────────────────┘
  │
  ▼
Response (back through the layers)
```

If the handler returns an error, the error travels back through the layers too. TraceLayer logs it, CorsLayer adds headers to the error response, and the client gets a proper HTTP response.

## Combining Multiple Layers

FicHub uses several layers. Here's how they fit together:

```rust
Router::new()
    // ... all routes ...
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
```

You can also use `.layer()` with other Tower middleware:

- **CompressionLayer** — compresses responses with gzip (saves bandwidth)
- **TimeoutLayer** — adds a timeout to request processing (prevents hung requests)
- **LimitBodySizeLayer** — limits the size of request bodies (prevents abuse)
- **SetResponseHeaderLayer** — adds custom headers to all responses

Each layer is independent and composable. Mix and match as needed. The power of Tower is that all middleware follows the same interface, so they compose seamlessly.

## Custom Middleware

Sometimes you need middleware that tower-http doesn't provide. Axum lets you write custom middleware as async functions:

```rust
use axum::{
    body::Body,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};

async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    tracing::info!("Incoming: {} {}", method, path);

    let response = next.run(req).await;

    tracing::info!("Response: {} {} → {}", method, path, response.status());

    response
}

let app = Router::new()
    .route("/hello", get(hello_handler))
    .layer(middleware::from_fn(log_request));
```

The `next.run(req)` call passes the request through to the next middleware (or the handler if there are no more layers). You can run code before and after — this is the "before/after" pattern.

FicHub doesn't use custom middleware for logging (TraceLayer handles it), but it's useful for things like:
- Adding request IDs for tracing
- Measuring request duration
- Custom authentication checks
- Rate limiting

🧪 **Try It Yourself: Add Timing Middleware**

Create middleware that measures how long each request takes:

```rust
use axum::{body::Body, extract::Request, middleware::{self, Next}, response::Response};
use std::time::Instant;

async fn measure_duration(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();

    let response = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!(
        "{} {} completed in {:?} (status: {})",
        method,
        path,
        duration,
        response.status(),
    );

    response
}

let app = Router::new()
    .route("/hello", get(hello_handler))
    .layer(middleware::from_fn(measure_duration));
```

Now every request logs its duration. This is useful for finding slow endpoints.

## Summary

In this chapter, you learned:

- **Middleware** is code that runs before/after every request — like layers of an onion
- **TraceLayer** logs requests and responses automatically — essential for debugging and monitoring
- **CorsLayer** adds headers for cross-origin browser requests — required when frontend and backend are on different domains
- **Layer ordering** matters — last added runs first (outermost layer)
- **`middleware::from_fn`** lets you write custom middleware as simple async functions
- Each layer is independent and composable — mix and match as needed

Middleware handles the cross-cutting concerns that every request needs. Instead of adding CORS headers and logging to every handler, you add them once as middleware and they apply everywhere. Clean, DRY, maintainable.

In the next chapter, we'll put it all together — organizing routes into modules, building the complete router, and creating the fallback for FicHub's frontend.

---

# Chapter 12: Building the Router

## Organizing Routes into Modules

So far, we've built handlers as standalone functions. But FicHub has dozens of endpoints — exports, metadata, search, tags, recommendations, OPDS catalog, cache downloads. Putting them all in one file would be a nightmare. You'd scroll for minutes to find a specific handler.

The solution is **modules**. Each group of related routes gets its own file:

```
src/
├── main.rs              ← entry point
├── server.rs            ← AppState + build_router
├── config.rs            ← Config struct
├── error.rs             ← AppError enum
├── db/
│   ├── mod.rs           ← init_pool
│   ├── models.rs        ← database models
│   └── queries.rs       ← SQL queries
├── scrape/
│   ├── mod.rs           ← scraper trait
│   ├── registry.rs      ← ScraperRegistry
│   └── ao3.rs           ← AO3 scraper
├── routes/
│   ├── mod.rs           ← re-exports
│   ├── export.rs        ← EPUB export handler
│   ├── meta.rs          ← metadata handler
│   ├── api_docs.rs      ← API documentation
│   ├── cache_download.rs← cache download handler
│   └── opds/
│       ├── mod.rs
│       ├── feeds.rs
│       ├── tags.rs
│       └── shelves.rs
├── recommender/
│   ├── mod.rs
│   ├── engine.rs
│   ├── routes.rs
│   └── worker.rs
├── tags/
│   ├── mod.rs
│   ├── routes.rs
│   ├── resolve.rs
│   └── curator.rs
└── search/
    ├── mod.rs
    └── routes.rs
```

Each route file defines its handlers:

```rust
// routes/export.rs
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Check for automated flag
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // Find the appropriate scraper
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    // Lookup metadata
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // ... continue with export logic ...
}
```

```rust
// routes/meta.rs
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct MetaQuery {
    pub q: Option<String>,
}

/// Metadata-only handler: GET /api/v0/meta?q=<url>
pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "info": format!("{} by {} - {} words, {} chapters",
            meta.title, meta.author, meta.words, meta.chapters),
    })))
}
```

```rust
// routes/mod.rs
pub mod api_docs;
pub mod cache_download;
pub mod export;
pub mod meta;
pub mod opds;
```

Now each handler lives in its own file, logically grouped, easy to find. When you need to modify the export handler, you know exactly where to look.

## The build_router Function

FicHub's router is built in a dedicated function. This is the central nervous system of the web server:

```rust
/// Build the Axum router with all routes
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // Redirect handler (for legacy URLs)
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }

    Router::new()
        // API documentation
        .route("/api/", get(routes::api_docs::api_docs_handler))

        // Core API routes
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // Cache download routes
        .route("/cache/{etype}/{url_id}/{fname}",
            get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}",
            get(routes::cache_download::download_or_export))

        // Recommender routes
        .route("/api/v0/recommendations",
            get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest",
            axum::routing::post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote",
            axum::routing::post(crate::recommender::routes::vote_handler))

        // Tag routes
        .route("/api/v0/tags",
            get(crate::tags::routes::get_tags))
        .route("/api/v0/tags/submit",
            axum::routing::post(crate::tags::routes::submit_tag))

        // Search route
        .route("/api/v0/search",
            get(crate::search::routes::search_handler))

        // Legacy redirect routes
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))

        // Static frontend (MUST come after API routes)
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

Each `.route()` call maps a URL pattern to a handler function. The full URL becomes visible by looking at the route chain — you can see at a glance what endpoints exist.

## Route Groups: /api/v0/...

FicHub organizes its API under a versioned prefix: `/api/v0/`. This is a common and important practice. When you make breaking changes to the API, you can add `/api/v1/` while keeping `/api/v0/` working for existing clients. Your users don't break when you upgrade.

The routes are flat (not explicitly nested in Axum) but the naming convention groups them logically:

```
API Documentation:
  /api/                              → API docs

Core Export:
  /api/v0/epub                       → EPUB export (main endpoint)
  /api/v0/meta                       → Metadata only
  /api/v0/remote                     → Remote client info

Cache Downloads:
  /cache/{etype}/{url_id}/{fname}    → Download with hash validation
  /cache/{etype}/{url_id}            → Export and download

Search:
  /api/v0/search                     → Advanced search

Tags:
  /api/v0/tags                       → Get tags
  /api/v0/tags/submit                → Submit a new tag
  /api/v0/tags/vote                  → Vote on a tag
  /api/v0/tags/flag                  → Flag a tag

Curator (admin):
  /api/v0/curator/alias              → Create tag alias
  /api/v0/curator/merge              → Merge tags
  /api/v0/curator/tags/{id}          → Delete a tag
  /api/v0/curator/flags              → List flagged tags

Recommendations:
  /api/v0/recommendations            → Get recommendations
  /api/v0/recommendations/suggest    → Suggest a recommendation
  /api/v0/recommendations/vote       → Vote on a recommendation

OPDS Catalog (for e-readers):
  /opds                              → Root catalog
  /opds/new                          → Recent fics
  /opds/popular                      → Popular fics
  /opds/tags                         → Tag categories
  /opds/tags/{type_id}               → Tags by type
  /opds/tags/{type_id}/{tag_name}    → Fics by tag
  /opds/authors                      → Author list
  /opds/search                       → Search feed
  /opds/shelves                      → Shelf list
```

Each group shares a common prefix but doesn't need explicit nesting in Axum. The route strings just need to match.

### GET vs POST Routes

Most routes are GET (read-only data). Some use POST for mutations:

```rust
// GET: read data (no side effects)
.route("/api/v0/epub", get(routes::export::epub_handler))
.route("/api/v0/meta", get(routes::meta::meta_handler))

// POST: write data or trigger side effects
.route("/api/v0/recommendations/vote",
    axum::routing::post(crate::recommender::routes::vote_handler))
.route("/api/v0/tags/submit",
    axum::routing::post(crate::tags::routes::submit_tag))
```

The convention:
- **GET** for reading — no changes to server state
- **POST** for creating or triggering — changes server state
- **PUT** for updating — replace a resource
- **DELETE** for removing — delete a resource

FicHub mostly uses GET and POST because its operations are simple: read fics, submit tags, vote on things.

## The Fallback: SPA Routing

FicHub has a web frontend — a single-page application (SPA) that runs in the browser. When someone visits `https://fichub.net/`, they see the web interface. But SPA routing is tricky.

In a traditional website, every URL maps to a different HTML file:
- `/about` → `about.html`
- `/contact` → `contact.html`

In an SPA, there's only one `index.html`. The JavaScript framework (React, Svelte, etc.) handles routing client-side. But if someone navigates directly to `https://fichub.net/fic/abc123` or refreshes the page, the browser requests that URL from the server. If the server doesn't have a file at `/fic/abc123`, it returns 404. The user sees a blank page.

The solution is a **fallback**: if no API route matches, serve `index.html` and let the frontend handle routing.

```rust
use tower_http::services::{ServeDir, ServeFile};

Router::new()
    // API routes...
    .fallback_service(
        ServeDir::new(&frontend_dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(frontend_dir.join("index.html"))),
    )
```

This does two things:

1. **`ServeDir`** — serves static files from the frontend build directory. If someone requests `/assets/style.css`, it serves the actual file. If they request `/favicon.ico`, it serves the favicon.

2. **`.fallback(ServeFile::new(...))`** — if the file doesn't exist in the directory, serve `index.html` instead. This way, `/fic/abc123` returns `index.html`, and the JavaScript router takes over.

The `.append_index_html_on_directories(true)` part means that requesting `/` or `/some-dir/` automatically serves `index.html` — the standard behavior for web servers.

⚠️ **Watch Out: API Routes Before Fallback**

The fallback must come **after** all API routes. Axum tries routes in order. If the fallback came first, it would catch API requests before they reach the handlers. This is the single most common mistake when setting up an SPA backend.

```rust
// CORRECT: API routes first, then fallback
Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .fallback_service(ServeDir::new(...))

// WRONG: fallback would catch /api/v0/epub and return index.html
Router::new()
    .fallback_service(ServeDir::new(...))
    .route("/api/v0/epub", get(epub_handler))
```

## The Complete Router

Let's see how FicHub's full router looks in `server.rs`. This is the real code, simplified for clarity:

```rust
use std::sync::Arc;
use axum::{routing::get, Router};
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer, cors::CorsLayer};
use axum::response::Redirect;

use crate::config::Config;
use crate::routes;

/// Shared application state
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<crate::scrape::registry::ScraperRegistry>,
}

/// Build and run the HTTP server
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = crate::db::init_pool(&config.database_url)
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
    let scraper_registry = Arc::new(crate::scrape::registry::ScraperRegistry::new());

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
    });

    // Build router
    let app = build_router(state).await;

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

/// Build the Axum router
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

        // Cache routes
        .route("/cache/{etype}/{url_id}/{fname}",
            get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}",
            get(routes::cache_download::download_or_export))

        // Legacy redirects
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))

        // Static frontend (fallback)
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )

        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())

        // State
        .with_state(state)
}

/// Remote info handler
async fn remote_handler(
    axum::extract::ConnectInfo(remote_addr):
        axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
    }))
}
```

This is the heart of FicHub. Every HTTP request to the server flows through this router. It matches the URL to a route, extracts parameters, calls the handler, and sends back the response.

## The Entry Point: main.rs

The entry point ties everything together:

```rust
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
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(
                    "info,fichub=debug"
                )),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

The flow is:

1. **`pub mod ...`** — declare all modules. Each module is a file or directory.
2. **`dotenvy::dotenv().ok()`** — load `.env` file (if it exists)
3. **`tracing_subscriber::fmt()...`** — initialize logging with environment-based filtering
4. **`config::Config::from_env()`** — load all settings from environment variables
5. **`tracing::info!`** — log that we're starting (useful for knowing when the server is up)
6. **`server::run(config).await`** — connect to databases, build the router, start listening

Clean, minimal, no surprises. Each line does one thing. The `server::run()` function handles all the heavy lifting.

## The Request Lifecycle

Now that we've built the complete server, let's trace a request from start to finish:

```
1. Client sends: GET /api/v0/epub?q=https://ao3.org/works/12345

2. TcpListener accepts the TCP connection

3. Axum reads the raw bytes and parses the HTTP request:
   - Method: GET
   - Path: /api/v0/epub
   - Query: q=https://ao3.org/works/12345
   - Headers: Host, Accept, etc.

4. CorsLayer runs:
   - Checks Origin header
   - Adds Access-Control-Allow-Origin header to response

5. TraceLayer runs:
   - Logs: "GET /api/v0/epub started"
   - Starts a timer

6. Router matches the path to /api/v0/epub → epub_handler

7. Extractors run:
   - State → Arc<AppState> (shared state from router)
   - Query → ExportQuery { q: Some("https://ao3.org/works/12345") }

8. epub_handler runs:
   a. Validates query parameter (not empty, not automated)
   b. Finds the right scraper for the URL
   c. Calls scraper.lookup() with the HTTP client
   d. Checks blacklists in the database
   e. Checks cache for existing export
   f. If cache miss: fetches chapters, generates EPUB
   g. Returns Ok(Json({ "err": 0, "url_id": "...", "epub_url": "..." }))

9. TraceLayer logs: "GET /api/v0/epub completed, status=200, latency=42ms"

10. CorsLayer adds final CORS headers

11. Response sent to client:
    HTTP/1.1 200 OK
    Content-Type: application/json
    Access-Control-Allow-Origin: *

    {"err": 0, "url_id": "abc123", "epub_url": "/cache/epub/abc123?h=..."}
```

Every piece we built — the router, extractors, error handling, shared state, middleware — plays a role in this flow. It's a beautiful system where each component has a clear responsibility.

## What We Built

Let's zoom out and appreciate what we've constructed in Part 2:

1. **An Axum web server** that listens for HTTP requests
2. **A routing system** that maps URLs to handler functions
3. **Extractors** that pull data from requests automatically (path params, query strings, JSON bodies)
4. **A configuration system** that loads settings from environment variables
5. **An error handling system** that converts Rust errors into HTTP responses
6. **A shared state system** that gives all handlers access to the database, HTTP client, and config
7. **Middleware** that adds logging and CORS headers to every request
8. **A module structure** that organizes dozens of routes into logical groups
9. **A SPA fallback** that serves the frontend for any non-API URL

This is a production-grade web server architecture. The same patterns you see in FicHub are used by thousands of Rust web applications.

🧪 **Try It Yourself: Build a Complete Mini-Server**

Combine everything from this part into one server:

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

// Error type
#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            AppError::Internal(m) => {
                tracing::error!("Internal: {}", m);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into())
            }
        };
        (status, Json(json!({"err": -1, "msg": msg}))).into_response()
    }
}

// Config
#[derive(Clone)]
struct Config {
    port: u16,
}

// State
struct AppState {
    config: Config,
    request_count: std::sync::atomic::AtomicUsize,
}

// Query
#[derive(Deserialize)]
struct GreetingQuery {
    name: Option<String>,
}

// Handlers
async fn hello() -> Json<Value> {
    Json(json!({"message": "Hello, FicHub!"}))
}

async fn greet(
    Path(name): Path<String>,
) -> Json<Value> {
    Json(json!({"greeting": format!("Hello, {}!", name)}))
}

async fn health(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    let count = state.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Json(json!({
        "status": "ok",
        "port": state.config.port,
        "requests": count + 1,
    }))
}

async fn search(
    Query(params): Query<GreetingQuery>,
) -> Result<Json<Value>, AppError> {
    let name = params.name
        .ok_or_else(|| AppError::BadRequest("name is required".into()))?;
    Ok(Json(json!({"results": [format!("Found: {}", name)]})))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let state = Arc::new(AppState {
        config: Config { port: 3000 },
        request_count: std::sync::atomic::AtomicUsize::new(0),
    });

    let app = Router::new()
        .route("/hello", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/health", get(health))
        .route("/search", get(search))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::info!("Listening on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

Test every endpoint:

```bash
curl http://localhost:3000/hello
curl http://localhost:3000/greet/world
curl http://localhost:3000/health
curl "http://localhost:3000/search?name=test"
```

This mini-server uses every concept from Part 2: routes, extractors, shared state, error handling, and async handlers. It's the same architecture as FicHub, just smaller.

## What's Ahead

You now have a complete web server. Routes handle URLs. Extractors pull data from requests. Error handling catches failures. Shared state gives handlers access to resources. Middleware adds cross-cutting concerns. Modules keep the code organized.

But we're missing something big: the database. Right now, our handlers don't persist anything. When FicHub scrapes a story, it needs to store the metadata so it doesn't scrape the same story twice. It needs to track which stories are cached, which are blacklisted, and which have been requested before.

In Part 3, we'll dive into PostgreSQL — the database that stores all of FicHub's data. You'll learn SQL, database design, migrations, and how to use SQLx to talk to PostgreSQL from Rust with compile-time safety.

The foundation is solid. Time to build the data layer.

---

# Part 2 Summary

Congratulations! You've completed Part 2 of the FicHub Backend book. Here's what you learned:

- **Chapter 6**: What a web server is, how HTTP works, how to set up Axum with Tokio, create routes and handlers, return text and JSON responses, and start a server with `TcpListener` and `axum::serve`.

- **Chapter 7**: How to extract data from HTTP requests — path parameters with `Path<T>`, query parameters with `Query<T>`, JSON bodies with `Json<T>`, and combining multiple extractors in one handler. How `Option<T>` makes fields optional and typed extractors provide automatic validation.

- **Chapter 8**: How to load configuration from environment variables with `dotenvy`, create a `Config` struct as a single source of truth, distinguish required vs optional variables using `.expect()` and `.unwrap_or_else()`, and parse strings into numbers, booleans, and complex types.

- **Chapter 9**: How to implement `IntoResponse` for custom error types, map error variants to HTTP status codes, log internal errors while keeping client messages generic, use the `?` operator with `From` trait conversions, and follow the `Result<T, AppError>` pattern.

- **Chapter 10**: How to share state across handlers with `Arc<T>`, create an `AppState` struct holding database pools, HTTP clients, and config, use the `State` extractor, connect to PostgreSQL with `PgPool`, and pass state to the router with `.with_state()`.

- **Chapter 11**: How to add middleware with `.layer()`, use `TraceLayer` for request logging, `CorsLayer` for cross-origin requests, understand layer ordering (last added = outermost = runs first), and write custom middleware with `middleware::from_fn`.

- **Chapter 12**: How to organize routes into modules, build a complete router with `build_router`, handle API versioning with `/api/v0/` prefixes, add a SPA fallback for frontend routing, and trace the full request lifecycle from TCP connection to HTTP response.

You now have a working Axum web server with proper configuration, error handling, shared state, middleware, and organized routing. This is the web layer that FicHub runs on. Every concept connects to the real codebase.

In Part 3, we'll dive into PostgreSQL — the database that stores all of FicHub's data. You'll learn SQL, database design, migrations, and how to use SQLx to talk to PostgreSQL from Rust. The foundation is solid. Time to build the data layer!

Ready for the next chapter? Let's keep building!
