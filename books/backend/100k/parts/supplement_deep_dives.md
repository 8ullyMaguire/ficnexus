# Supplementary Content: Deep Dives and Extended Explanations

---

# Extended Chapter: Understanding Axum's Architecture

## How Axum Processes Requests

When a request arrives at FicHub's server, it goes through several layers before reaching your handler code. Understanding this pipeline helps you write better handlers and debug issues.

### The Tower Service Stack

Axum is built on Tower, a library for building modular middleware. Every Axum application is a Tower service — a function that takes a request and returns a response. Middleware layers wrap this service, adding behavior before and after the handler runs.

When you write:

```rust
let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(state);
```

You're building a stack that looks like this:

1. **CorsLayer** — Handles CORS headers (outermost)
2. **TraceLayer** — Logs the request and response
3. **Router** — Matches the URL to a handler
4. **epub_handler** — Your handler code (innermost)

When a request arrives, it flows from the outermost layer inward. Each layer can:
- Inspect or modify the request
- Pass it to the next layer
- Short-circuit and return a response immediately

### Request Flow in Detail

Here's what happens when you send `GET /api/v0/epub?q=https://ao3.org/works/123`:

1. **TCP connection** — The OS accepts the connection on port 3000
2. **HTTP parsing** — hyper (Axum's HTTP library) parses the request bytes into a structured request
3. **CorsLayer** — Checks if the request origin is allowed; adds CORS headers to the response
4. **TraceLayer** — Records the start time, method, and URI; will log the response when it comes back
5. **Router** — Matches `/api/v0/epub` to `epub_handler`
6. **Extractor resolution** — Axum looks at the handler's parameters and extracts them from the request:
   - `State(state): State<Arc<AppState>>` — Clones the Arc from the router's state
   - `Query(params): Query<ExportQuery>` — Parses the query string into `ExportQuery`
7. **Handler execution** — Your async function runs
8. **Response serialization** — The return value is converted to an HTTP response
9. **TraceLayer** — Logs the response status and timing
10. **CorsLayer** — Adds CORS headers if needed
11. **TCP send** — The response bytes are sent back to the client

### Why This Matters

Understanding the pipeline helps you:
- **Debug middleware issues** — If a request returns unexpected headers, check the middleware layers
- **Write efficient handlers** — Know what work the framework does vs. what you do
- **Understand error propagation** — Errors flow back through the same layers
- **Optimize performance** — Know where time is spent (parsing, extraction, handler, serialization)

## Extractors in Depth

Extractors are one of Axum's most powerful features. They let you declare what data you need from the request, and Axum figures out how to get it.

### How Extractors Work

When you write:

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Query(params): Query<ExportQuery>,
    Json(body): Json<SubmitBody>,
) -> Json<Value> {
    // ...
}
```

Axum resolves extractors in left-to-right order. For each parameter, it calls the corresponding `FromRequest` implementation, which either:
- Succeeds and provides the extracted data
- Fails and returns an error response (the handler is never called)

### Custom Extractors

You can create your own extractors by implementing `FromRequest`:

```rust
struct ClientIp(IpAddr);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ConnectInfo(addr) = ConnectInfo::<SocketAddr>::from_request_parts(parts, _state).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(ClientIp(addr.ip()))
    }
}
```

Now you can use `ClientIp(ip): ClientIp` in your handlers.

### extractor Combinations

Some extractors can be combined:

```rust
// Both work — Axum resolves them independently
async fn handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    // Access both state and client IP
}
```

But some extractors consume the request body, so they can't be combined:

```rust
// This WON'T work — both try to read the body
async fn handler(
    Json(body1): Json<FirstType>,
    Json(body2): Json<SecondType>,  // Error! Body already consumed
) -> Json<Value> {
    unreachable!()
}
```

## Router Patterns

### Nested Routes

Group related routes under a common prefix:

```rust
let api_routes = Router::new()
    .route("/epub", get(epub_handler))
    .route("/meta", get(meta_handler))
    .route("/search", get(search_handler));

let app = Router::new()
    .nest("/api/v0", api_routes)
    .nest("/opds", opds_routes);
```

### Method Routing

Chain multiple HTTP methods on the same path:

```rust
Router::new()
    .route("/items", get(list_items).post(create_item))
    .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
```

### Route Merging

Combine routers from different modules:

```rust
let app = Router::new()
    .merge(core_routes)
    .merge(recommender_routes)
    .merge(tag_routes)
    .merge(opds_routes);
```

## Error Handling Patterns

### The Result Return Type

Handlers return `Result<T, E>` where `E: IntoResponse`:

```rust
async fn handler() -> Result<Json<Value>, AppError> {
    let data = fetch_data().await?;  // ? converts errors
    Ok(Json(json!({ "data": data })))
}
```

### The ? Operator

The `?` operator is Rust's error propagation shorthand. When you write `fetch_data().await?`, it either:
- Returns the value on success
- Converts the error via `From` and returns it from the function

FicHub's `AppError` implements `From` for all common error types:

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
```

### Custom Error Responses

The `IntoResponse` implementation on `AppError` controls the HTTP response:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            // ... other variants ...
        };
        (status, Json(body)).into_response()
    }
}
```

## Summary

Axum's architecture is built on Tower services and middleware layers. Understanding the request pipeline, extractor system, and error handling patterns is essential for building effective handlers.

---

# Extended Chapter: PostgreSQL Deep Dive

## Connection Pool Internals

When you create a `PgPool`, you're creating a pool of connections to PostgreSQL. Here's how it works:

### Pool Lifecycle

1. **Creation** — `PgPoolOptions::new().connect(url).await` creates the pool with zero connections
2. **Lazy connection** — Connections are created on-demand as handlers request them
3. **Maximum** — Up to `max_connections` (default 20) connections are created
4. **Idle** — Unused connections are kept alive with periodic pings
5. **Timeout** — If all connections are busy, new requests wait up to `acquire_timeout`
6. **Release** — After a query completes, the connection is returned to the pool

### Pool Configuration

```rust
PgPoolOptions::new()
    .max_connections(20)          // Maximum concurrent connections
    .min_connections(5)           // Minimum idle connections
    .max_lifetime(Duration::from_secs(1800))  // Recycle connections after 30 min
    .idle_timeout(Duration::from_secs(600))    // Close idle connections after 10 min
    .acquire_timeout(Duration::from_secs(10))  // Wait 10s for a connection
    .test_before_acquire(true)    // Test connections before handing them out
    .connect(url).await?
```

### Monitoring Pool Health

```sql
-- Active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub' AND state = 'active';

-- Idle connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub' AND state = 'idle';

-- Connection states
SELECT state, count(*) FROM pg_stat_activity WHERE datname = 'fichub' GROUP BY state;
```

If you see many idle connections, reduce `max_connections`. If you see many waiting requests, increase it.

## Query Performance

### Understanding EXPLAIN ANALYZE

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM fic_info WHERE text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY ts_rank(text_search, plainto_tsquery('english', 'harry potter')) DESC
LIMIT 20;
```

This shows:
- **Seq Scan vs Index Scan** — Whether PostgreSQL uses the GIN index
- **Cost** — Estimated and actual cost of each operation
- **Rows** — Estimated vs actual number of rows
- **Time** — Actual execution time

### Index Strategies

FicHub uses several index types:

**B-tree indexes** (default) — For equality and range queries:
```sql
CREATE INDEX idx_fic_info_id ON fic_info(id);
```

**GIN indexes** — For full-text search:
```sql
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

**Partial indexes** — For common filtered queries:
```sql
CREATE INDEX idx_request_log_date_export ON request_log(created)
WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

**Composite indexes** — For multi-column queries:
```sql
CREATE INDEX idx_request_log_url_id_etype_created ON request_log(url_id, etype, created);
```

## JSON and JSONB

PostgreSQL supports JSON natively. FicHub stores some metadata as JSON text:

```sql
extra_meta TEXT  -- JSON text
raw_extended_meta TEXT  -- Raw JSON from the source
```

For complex JSON queries, you'd use JSONB:

```sql
-- Extract a field
SELECT extra_meta->>'rating' FROM fic_info WHERE id = 'abc123';

-- Filter by JSON field
SELECT * FROM fic_info WHERE (extra_meta::jsonb)->>'rating' = 'T';
```

## Transactions

For multi-step operations that must be atomic:

```rust
let mut tx = pool.begin().await?;

sqlx::query("UPDATE fic_tags SET score = score + 1 WHERE tag_id = $1")
    .bind(tag_id).execute(&mut *tx).await?;

sqlx::query("UPDATE fic_tag_votes SET value = value + 1 WHERE tag_id = $1")
    .bind(tag_id).execute(&mut *tx).await?;

tx.commit().await?;
```

If any step fails, the transaction is rolled back.

## Connection String Formats

```bash
# Simple
postgres://user:password@localhost/database

# With port
postgres://user:password@localhost:5432/database

# With options
postgres://user:password@localhost/database?sslmode=require

# Unix socket
postgres://user:password@/database?host=/var/run/postgresql
```

## Summary

Understanding connection pool internals, query performance, index strategies, and transaction handling helps you build reliable, fast database-backed applications.

---

# Extended Chapter: Async Rust Explained

## What is Async?

In synchronous code, operations execute one at a time. If your program needs to read a file, it blocks until the file is read. If it needs to make a network request, it blocks until the response arrives.

In asynchronous code, operations can be suspended while waiting for I/O. When a handler needs to wait for a database query, it yields control to the runtime, which can then handle other requests. When the query completes, the handler resumes.

## The async/await Pattern

```rust
async fn fetch_metadata(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;  // Suspends until response arrives
    let body = response.text().await?;         // Suspends until body is read
    Ok(body)
}
```

The `async` keyword marks a function as asynchronous. Inside, `.await` pauses execution until the future completes.

### What Happens at .await

When you write `reqwest::get(url).await`:

1. The function starts the HTTP request
2. It returns a future to the runtime
3. The runtime polls the future periodically
4. When the response arrives, the runtime resumes the function
5. The function continues with the response data

### Futures vs Promises

Rust futures are **lazy** — they don't do anything until polled. JavaScript promises are **eager** — they start executing immediately. This is a key difference.

```rust
// This does nothing — the future is just created
let future = async { 1 + 1 };

// This polls the future
let result = future.await;  // result = 2
```

## The Tokio Runtime

Tokio is the most popular async runtime for Rust. It provides:

- **Multi-threaded executor** — Runs tasks on a thread pool
- **Timer** — Sleeps and timeouts
- **I/O** — TCP, UDP, Unix sockets
- **Channels** — mpsc, oneshot, broadcast
- **Sync primitives** — Mutex, RwLock, Semaphore
- **Task spawning** — `tokio::spawn`

### The #[tokio::main] Macro

```rust
#[tokio::main]
async fn main() {
    // This runs on the Tokio runtime
    let result = fetch_metadata("https://example.com").await;
}
```

The macro expands to:

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let result = fetch_metadata("https://example.com").await;
        });
}
```

## Common Async Patterns

### Join — Run Tasks Concurrently

```rust
let (epub, html) = tokio::join!(
    create_epub(&meta, &chapters, &tmp_dir),
    create_html_bundle(&meta, &chapters, &tmp_dir),
);
```

Both tasks run concurrently. `join!` waits for both to complete.

### Select — Race Multiple Tasks

```rust
tokio::select! {
    result = fetch_metadata(url) => { /* metadata arrived */ }
    _ = tokio::time::sleep(Duration::from_secs(30)) => { /* timeout */ }
}
```

Whichever future completes first wins. The other is cancelled.

### Spawn — Independent Tasks

```rust
tokio::spawn(async {
    // This runs independently
    process_fic(url).await;
});
// Main task continues without waiting
```

### Mutex — Shared Mutable State

```rust
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(HashMap::new()));

// In a handler
let mut map = data.lock().await;
map.insert("key".to_string(), "value".to_string());
// Lock is released when `map` goes out of scope
```

### Semaphore — Rate Limiting

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10));  // Max 10 concurrent

let permit = semaphore.acquire().await?;
// Do work
drop(permit);  // Release the permit
```

## Send and Sync

Rust's ownership system extends to async code. Futures must be `Send` to be moved between threads:

```rust
// This future is Send — it can run on any thread
async fn handler() -> String {
    let data = fetch_data().await;
    process(data).await
}

// This future is NOT Send — it holds a non-Send type
async fn bad_handler() {
    let rc = std::rc::Rc::new(5);  // Rc is not Send
    // Error if this future needs to be Send
}
```

**Rule of thumb:** Don't hold `Rc`, `RefCell`, or raw pointers across `.await` points.

## Blocking Operations

Some operations can't be async (like file I/O or CPU-intensive work). Use `spawn_blocking`:

```rust
let result = tokio::task::spawn_blocking(move || {
    // This runs on a dedicated blocking thread pool
    std::fs::read_to_string(path)
}).await?;
```

## Watch Out!

**Don't block the runtime!** Long-running sync operations in async code block all other tasks.

**Avoid unwrap() in async code!** A panic in an async task kills only that task, but the error message is unhelpful. Use `?` or `unwrap_or_else`.

**Clone Arc, don't clone data!** `Arc::clone` is cheap (atomic increment). Cloning the data inside is expensive.

## Summary

Async Rust enables efficient I/O by suspending tasks while waiting. The Tokio runtime manages task scheduling, timers, and I/O. Understanding `Send`/`Sync`, `spawn_blocking`, and common patterns like `join!` and `select!` is essential for building concurrent applications.

---

# Extended Chapter: The Fanfiction Scraping Pipeline

## Complete Scraping Workflow

When FicHub processes a request for a fanfiction URL, it goes through a complex pipeline. Let's trace every step in detail.

### Phase 1: URL Resolution (Step 1-4)

**Step 1: HTTP Request Arrives**

A user sends:
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345678 HTTP/1.1
Host: fichub.example.com
```

The Axum router matches this to `epub_handler`.

**Step 2: Parameter Extraction**

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}
```

Axum parses the query string into this struct. `q` is the fanfiction URL.

**Step 3: Validation**

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}
if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
}
```

**Step 4: Scraper Lookup**

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

The registry iterates through all scrapers and calls `can_handle` on each. For an AO3 URL, `Ao3Scraper::can_handle` returns true.

### Phase 2: Metadata Extraction (Step 5-6)

**Step 5: HTTP Request to AO3**

```rust
let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

This makes an HTTP GET request to AO3's servers. The `?view_full_work=true` parameter tells AO3 to render all chapters on one page.

**Step 6: HTML Parsing**

```rust
let html = response.text().await?;
let document = Html::parse_document(&html);

let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

The `scraper` crate parses the HTML into a DOM tree, then CSS selectors extract specific elements.

### Phase 3: Database Operations (Step 6-8)

**Step 7: Upsert Fic Info**

```rust
sqlx::query(
    r#"INSERT INTO fic_info (id, title, author, words, chapters, status, source, ...)
       VALUES ($1, $2, $3, $4, $5, $6, $7, ...)
       ON CONFLICT (id) DO UPDATE SET
           title = EXCLUDED.title,
           words = EXCLUDED.words,
           ..."#,
)
.bind(&meta.url_id)
.bind(&meta.title)
// ... more binds ...
.execute(&state.db)
.await?;
```

The `ON CONFLICT DO UPDATE` clause ensures the record is created or updated.

**Step 8: Auto-populate Tags**

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(
            &state.db, &tag.name, tag.tag_type_id,
        ).await {
            let _ = queries::upsert_fic_tag(
                &state.db, &meta.url_id, resolution.tag_id,
                &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            ).await;
        }
    }
}
```

Tags extracted from AO3's HTML are automatically added to the database.

### Phase 4: Cache Check (Step 9-13)

**Step 9: Version Computation**

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
```

**Step 10-11: Cache Lookup**

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Cache hit — build response and return
    return Ok(build_response(&export_log, &meta));
}
```

The cache lookup checks if the same EPUB was already generated with the same input hash.

**Step 12: Semaphore Acquisition**

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

This prevents duplicate concurrent generation of the same EPUB.

**Step 13: Double-Check**

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Another request generated it while we were waiting
    return Ok(build_response(&export_log, &meta));
}
```

### Phase 5: Content Fetching (Step 14)

**Step 14: Fetch Chapters**

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

For AO3, this fetches the full work page and extracts each chapter's content.

For FF.net, this loops through each chapter URL individually:
```rust
for i in 1..=meta.chapters {
    let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
    let response = client.get(&url).send().await?;
    // ... extract content ...
}
```

### Phase 6: File Generation (Step 15-17)

**Step 15: EPUB Generation**

```rust
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

This creates a well-formatted EPUB file with metadata, CSS, introduction page, and chapters.

**Step 16: Move to Cache**

```rust
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

**Step 17: Generate HTML Bundle**

```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir).await?;
let html_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
```

### Phase 7: Response Building

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    "url_id": meta.url_id,
    "slug": generate_slug(&meta.title, &meta.url_id),
    "meta": build_meta_json(&meta),
    "hashes": {
        "epub": epub_hash,
        "html": html_hash,
    },
    "urls": {
        "epub": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
        "html": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
    },
    "epub_url": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
    "html_url": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
    "info": build_info_string(&meta).0,
    "notes": Vec::<String>::new(),
})))
```

## Error Scenarios

### Network Error

If AO3 is unreachable:
```
ScrapeError::Network("connection refused") → AppError::ScrapeError → 502 Bad Gateway
```

### Parse Error

If AO3's HTML structure changed:
```
ScrapeError::ParseError("could not extract title") → AppError::ScrapeError → 502 Bad Gateway
```

### Blacklisted Fic

If the fic is on the blacklist:
```
AppError::BadRequest(-7, "fic is blacklisted") → 400 Bad Request
```

### Rate Limited

If the user has made too many requests:
```
AppError::RateLimited(3600) → 429 Too Many Requests
```

## Performance Characteristics

| Phase | Time | Notes |
|-------|------|-------|
| URL resolution | ~1ms | Regex + HashMap lookup |
| Metadata extraction | ~500ms-2s | HTTP request to fanfiction site |
| Database upsert | ~5ms | Single INSERT/UPDATE |
| Cache check | ~2ms | Single SELECT |
| Chapter fetching | ~2-30s | Depends on chapter count |
| EPUB generation | ~1-5s | Depends on story length |
| HTML generation | ~1-3s | Depends on story length |
| **Total (cache hit)** | **~10ms** | Database lookups only |
| **Total (cache miss)** | **~10-60s** | Full pipeline |

## Summary

The scraping pipeline is a carefully orchestrated sequence of network requests, HTML parsing, database operations, and file generation. Understanding each phase helps you optimize performance and debug issues.
