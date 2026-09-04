# Extended Content: Additional Deep Dives

---

# Advanced Topic: Axum Middleware in Depth

Middleware in Axum is implemented using Tower's Service trait. Every request passes through a stack of middleware layers before reaching the handler.

## How Middleware Works

When a request arrives at the server, it passes through each middleware layer in order. Each layer can:
1. Inspect the request
2. Modify the request
3. Call the next layer
4. Inspect the response
5. Modify the response

**Real-world analogy:** Middleware is like a series of security checkpoints at an airport. Each checkpoint (middleware) checks your boarding pass (request), maybe stamps it (modifies), and then lets you through to the next checkpoint or to your gate (handler).

## Custom Middleware Example

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = status.as_u16(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );
    
    response
}

// Usage in router:
let app = Router::new()
    .route("/", get(handler))
    .layer(axum::middleware::from_fn(logging_middleware));
```

## Request ID Middleware

```rust
use axum::http::HeaderMap;
use uuid::Uuid;

async fn request_id_middleware(
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(request_id.clone());
    
    let mut response = next.run(req).await;
    response.headers_mut().insert(
        "X-Request-Id",
        request_id.parse().unwrap(),
    );
    
    response
}
```

## CORS Middleware Configuration

```rust
use tower_http::cors::{CorsLayer, Any, Method};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
    ])
    .allow_headers(Any)
    .max_age(Duration::from_secs(3600));

let app = Router::new()
    .route("/", get(handler))
    .layer(cors);
```

## Compression Middleware

```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(CompressionLayer::new());
```

This automatically compresses responses using the best available algorithm (brotli, gzip, deflate).

## Rate Limiting Middleware

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn rate_limit_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    
    match state.rate_limiter.check(ip, req.uri().path()).await? {
        RateLimitResult::Allowed => Ok(next.run(req).await),
        RateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        RateLimitResult::Blocked => Err(AppError::BadRequest(-403, "blocked".into())),
    }
}
```

## Middleware Ordering

The order of middleware matters. Layers are applied in reverse order:

```rust
let app = Router::new()
    .route("/", get(handler))
    .layer(A)  // Applied third (outermost)
    .layer(B)  // Applied second
    .layer(C); // Applied first (innermost)

// Request flow: C -> B -> A -> Handler -> A -> B -> C
```

## Tower Service Trait

Under the hood, middleware implements the Tower Service trait:

```rust
use tower::Service;
use std::task::{Context, Poll};

struct MyMiddleware<S> {
    inner: S,
}

impl<S, Request> Service<Request> for MyMiddleware<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;
    
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }
    
    fn call(&mut self, req: Request) -> Self::Future {
        // Do something before
        println!("Request: {:?}", req);
        
        // Call inner service
        self.inner.call(req)
    }
}
```

## Layer Trait

Layers create middleware from a service:

```rust
use tower::Layer;

struct MyLayer;

impl<S> Layer<S> for MyLayer {
    type Service = MyMiddleware<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}

// Usage:
let app = Router::new()
    .route("/", get(handler))
    .layer(MyLayer);
```

## Middleware Testing

```rust
#[tokio::test]
async fn test_logging_middleware() {
    let app = Router::new()
        .route("/", get(handler))
        .layer(axum::middleware::from_fn(logging_middleware));
    
    let client = axum_test::TestClient::new(app);
    let response = client.get("/").await;
    
    assert_eq!(response.status_code(), 200);
    // Check logs for request information
}
```

---

# Advanced Topic: SQLx Deep Dive

SQLx is more than just a database driver. This chapter covers advanced SQLx features.

## Compile-Time Query Checking

SQLx verifies queries against your database schema at compile time:

```rust
// This query is checked at compile time
let fic = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1"
)
.bind(url_id)
.fetch_optional(&pool)
.await?;
```

If the `fic_info` table doesn't exist or has different columns, the code won't compile.

## Type Mapping

SQLx maps Rust types to PostgreSQL types:

| Rust Type | PostgreSQL Type |
|-----------|----------------|
| `i32` | `INT4` |
| `i64` | `INT8` |
| `f64` | `FLOAT8` |
| `bool` | `BOOLEAN` |
| `String` | `TEXT` |
| `Vec<u8>` | `BYTEA` |
| `chrono::DateTime<Utc>` | `TIMESTAMPTZ` |
| `uuid::Uuid` | `UUID` |
| `serde_json::Value` | `JSONB` |
| `Option<T>` | Nullable `T` |

## Custom Type Mapping

```rust
use sqlx::Type;

#[derive(Type)]
#[sqlx(type_name = "status_type", rename_all = "lowercase")]
enum StoryStatus {
    Ongoing,
    Complete,
    Hiatus,
    Cancelled,
}
```

## Transaction Management

```rust
async fn transfer_tag(
    pool: &PgPool,
    from_url_id: &str,
    to_url_id: &str,
    tag_id: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    
    sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(from_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    sqlx::query("INSERT INTO fic_tags (url_id, tag_id, score) VALUES ($1, $2, 0)")
        .bind(to_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

## Connection Pool Monitoring

```rust
let stats = pool.stats();
tracing::info!(
    active = stats.active_connections(),
    idle = stats.idle_connections(),
    waiting = stats.waiting(),
    max = pool.options().max_connections(),
    "Database pool stats"
);
```

## Migrations

```rust
// Run migrations on startup
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;

// Check migration status
let migrations = sqlx::migrate!("./migrations");
for migration in migrations.iter() {
    println!("Migration: {}", migration.description());
}
```

## Raw SQL with Query Builder

```rust
use sqlx::QueryBuilder;

fn build_search_query(params: &SearchParams) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new(
        "SELECT fi.id, fi.title, fi.author FROM fic_info fi WHERE 1=1"
    );
    
    if let Some(ref q) = params.q {
        qb.push(" AND to_tsvector('english', fi.title) @@ plainto_tsquery('english', ");
        qb.push_bind(q.clone());
        qb.push(")");
    }
    
    if let Some(min) = params.min_words {
        qb.push(" AND fi.words >= ");
        qb.push_bind(min);
    }
    
    qb
}
```

## Offline Mode

SQLx can check queries offline using a saved database snapshot:

```bash
# Save database snapshot
cargo sqlx prepare

# Check queries offline
SQLX_OFFLINE=true cargo check
```

---

# Advanced Topic: Tokio Deep Dive

Tokio is the most widely used async runtime for Rust. This chapter covers advanced Tokio features.

## Task Spawning

```rust
// Spawn a fire-and-forget task
tokio::spawn(async {
    background_work().await;
});

// Spawn and wait for result
let result = tokio::spawn(async {
    compute_something().await
}).await?;

// Spawn on a specific runtime
let handle = tokio::runtime::Handle::current();
handle.spawn(async {
    work().await;
});
```

## Channels

### mpsc (Multi-Producer, Single-Consumer)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<String>(100);

// Multiple producers
let tx1 = tx.clone();
tokio::spawn(async move {
    tx1.send("hello".into()).await.unwrap();
});

let tx2 = tx.clone();
tokio::spawn(async move {
    tx2.send("world".into()).await.unwrap();
});

// Single consumer
while let Some(msg) = rx.recv().await {
    println!("Received: {}", msg);
}
```

### broadcast (Multi-Producer, Multi-Consumer)

```rust
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel::<Event>(100);

// Multiple subscribers
let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

tokio::spawn(async move {
    while let Ok(event) = rx1.recv().await {
        println!("Subscriber 1: {:?}", event);
    }
});

tokio::spawn(async move {
    while let Ok(event) = rx2.recv().await {
        println!("Subscriber 2: {:?}", event);
    }
});

// Publisher
tx.send(Event::new()).unwrap();
```

### oneshot (Single-Producer, Single-Consumer)

```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel();

tokio::spawn(async move {
    let result = compute().await;
    tx.send(result).unwrap();
});

let result = rx.await.unwrap();
```

## Synchronization Primitives

### Mutex

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(Vec::new()));

let data1 = data.clone();
tokio::spawn(async move {
    let mut guard = data1.lock().await;
    guard.push(1);
});

let data2 = data.clone();
tokio::spawn(async move {
    let mut guard = data2.lock().await;
    guard.push(2);
});

// Wait for all tasks to complete
tokio::time::sleep(Duration::from_millis(100)).await;
println!("{:?}", *data.lock().await);  // [1, 2] or [2, 1]
```

### RwLock

```rust
use tokio::sync::RwLock;

let data = Arc::new(RwLock::new(Vec::new()));

// Multiple readers
let data1 = data.clone();
tokio::spawn(async move {
    let guard = data1.read().await;
    println!("Reader 1: {:?}", *guard);
});

let data2 = data.clone();
tokio::spawn(async move {
    let guard = data2.read().await;
    println!("Reader 2: {:?}", *guard);
});

// Writer
let data3 = data.clone();
tokio::spawn(async move {
    let mut guard = data3.write().await;
    guard.push(42);
});
```

### Semaphore

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(3));  // Max 3 concurrent tasks

let mut handles = vec![];
for i in 0..10 {
    let sem = semaphore.clone();
    handles.push(tokio::spawn(async move {
        let _permit = sem.acquire().await.unwrap();
        println!("Task {} running", i);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }));
}

for handle in handles {
    handle.await.unwrap();
}
```

## Time

### Sleep

```rust
tokio::time::sleep(Duration::from_secs(5)).await;
```

### Interval

```rust
use tokio::time::{interval, Duration};

let mut interval = interval(Duration::from_secs(1));

loop {
    interval.tick().await;
    println!("Tick!");
}
```

### Timeout

```rust
use tokio::time::timeout;

let result = timeout(
    Duration::from_secs(30),
    async {
        reqwest::get("https://example.com").await
    }
).await;

match result {
    Ok(Ok(response)) => println!("Success"),
    Ok(Err(e)) => println!("Request error: {}", e),
    Err(_) => println!("Timeout"),
}
```

## I/O

### TCP

```rust
use tokio::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:8080").await?;

loop {
    let (socket, addr) = listener.accept().await?;
    tokio::spawn(async move {
        process_connection(socket).await;
    });
}
```

### File I/O

```rust
use tokio::fs;

let content = fs::read_to_string("data.txt").await?;
fs::write("output.txt", "Hello, world!").await?;
```

## Runtime Configuration

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .thread_name("fichub-worker")
    .on_thread_start(|| {
        tracing::debug!("Worker thread started");
    })
    .on_thread_stop(|| {
        tracing::debug!("Worker thread stopped");
    })
    .build()
    .unwrap();
```

---

# Advanced Topic: Serde Deep Dive

Serde is the serialization framework for Rust. This chapter covers advanced features.

## Derive Macros

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Story {
    title: String,
    author: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(rename = "wordCount")]
    word_count: i64,
}
```

## Custom Serialization

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};

mod timestamp_millis {
    use super::*;
    
    pub fn serialize<S>(timestamp: &i64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(*timestamp)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Ok(timestamp)
    }
}

#[derive(Serialize, Deserialize)]
struct FicMetadata {
    title: String,
    #[serde(with = "timestamp_millis")]
    published: i64,
}
```

## Flattening

```rust
#[derive(Serialize, Deserialize)]
struct Base {
    id: String,
    created: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct Story {
    #[serde(flatten)]
    base: Base,
    title: String,
    author: String,
}

// Serializes as:
// {"id": "123", "created": "...", "title": "...", "author": "..."}
```

## Tagged Enums

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ExportResult {
    #[serde(rename = "epub")]
    Epub { hash: String, size: u64 },
    #[serde(rename = "html")]
    Html { hash: String, size: u64 },
    #[serde(rename = "error")]
    Error { message: String },
}

// Serializes as:
// {"type": "epub", "hash": "abc123", "size": 12345}
```

## Conditional Serialization

```rust
#[derive(Serialize)]
struct Story {
    title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cover_url: Option<String>,
}
```

## Custom Deserialize

```rust
use serde::de::{self, Deserializer, Visitor};

struct StringOrNumber;

impl<'de> Visitor<'de> for StringOrNumber {
    type Value = String;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or number")
    }
    
    fn visit_str<E>(self, value: &str) -> Result<String, E> {
        Ok(value.to_string())
    }
    
    fn visit_i64<E>(self, value: i64) -> Result<String, E> {
        Ok(value.to_string())
    }
    
    fn visit_u64<E>(self, value: u64) -> Result<String, E> {
        Ok(value.to_string())
    }
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(StringOrNumber)
}

#[derive(Deserialize)]
struct Config {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    port: String,
}
```

## Serde with Axum

```rust
#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_per_page")]
    per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 20 }

#[derive(Serialize)]
struct SearchResponse {
    err: i32,
    total: i64,
    page: u32,
    per_page: u32,
    results: Vec<Story>,
}

async fn search_handler(
    Query(params): Query<SearchParams>,
) -> Json<SearchResponse> {
    Json(SearchResponse {
        err: 0,
        total: 0,
        page: params.page,
        per_page: params.per_page,
        results: vec![],
    })
}
```

---

# Advanced Topic: Error Handling Patterns

## The ? Operator in Depth

The ? operator automatically converts errors using From implementations:

```rust
async fn process(url: &str) -> Result<String, AppError> {
    // reqwest::Error -> AppError via From
    let response = reqwest::get(url).await?;
    
    // reqwest::Error -> AppError via From
    let text = response.text().await?;
    
    Ok(text)
}
```

## Error Conversion Chains

```rust
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::ScrapeError("request timed out".into())
        } else if err.is_connect() {
            AppError::ScrapeError("connection failed".into())
        } else {
            AppError::ScrapeError(err.to_string())
        }
    }
}
```

## Error Context

```rust
use anyhow::Context;

async fn fetch_and_parse(url: &str) -> Result<StoryMeta, anyhow::Error> {
    let response = reqwest::get(url).await
        .context(format!("Failed to fetch {}", url))?;
    
    let html = response.text().await
        .context("Failed to read response body")?;
    
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title").unwrap())
        .next()
        .context("Title element not found")?
        .text()
        .collect::<String>();
    
    Ok(StoryMeta { title })
}
```

## Error Handling Best Practices

1. **Use thiserror for libraries** — Define specific error types that callers can match on
2. **Use anyhow for applications** — Use flexible error types with context
3. **Don't ignore errors** — Always handle or propagate errors
4. **Add context** — Include relevant information in error messages
5. **Use ? for propagation** — Let the compiler handle error conversion
6. **Map errors at boundaries** — Convert library errors to application errors at module boundaries

---

# Advanced Topic: Testing Patterns

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        let input = "test";
        let result = my_function(input);
        assert_eq!(result, expected);
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_panic() {
        my_function_that_panics();
    }
    
    #[test]
    fn test_result() -> Result<(), Box<dyn std::error::Error>> {
        let result = my_function()?;
        assert_eq!(result, expected);
        Ok(())
    }
}
```

## Integration Tests

```rust
// tests/api_tests.rs
use reqwest;

#[tokio::test]
async fn test_endpoint() {
    let app = build_test_app().await;
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("http://{}/api/v0/epub", app.addr()))
        .query(&[("q", "https://example.com/story")])
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}
```

## Test Fixtures

```rust
async fn setup_test_db() -> PgPool {
    let pool = PgPool::connect("postgres://localhost/fichub_test")
        .await
        .unwrap();
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    pool
}

async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("DELETE FROM fic_info").execute(pool).await.unwrap();
}
```

## Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_url_id_always_hex(s in "[a-z0-9]+", id in 0i64..10000) {
        let url_id = generate_url_id(id, &s);
        prop_assert_eq!(url_id.len(), 12);
        prop_assert!(url_id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
```

## Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("generate_url_id", |b| {
        b.iter(|| generate_url_id(black_box(42), black_box("story_123")))
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

## Test Organization

```rust
#[cfg(test)]
mod tests {
    mod unit {
        use super::*;
        
        #[test]
        fn test_basic() { /* ... */ }
    }
    
    mod integration {
        use super::*;
        
        #[tokio::test]
        async fn test_endpoint() { /* ... */ }
    }
}
```

---

# Advanced Topic: Performance Optimization

## Avoiding Allocations

```rust
// BAD: Allocating in a loop
for i in 0..10000 {
    let s = format!("item_{}", i);
    process(&s);
}

// GOOD: Reuse buffer
let mut s = String::new();
for i in 0..10000 {
    s.clear();
    write!(&mut s, "item_{}", i).unwrap();
    process(&s);
}
```

## Using References

```rust
// BAD: Cloning
let data = vec![0u8; 1024 * 1024];
let data_clone = data.clone();

// GOOD: Reference
let data = vec![0u8; 1024 * 1024];
process(&data);
```

## Parallel Processing

```rust
use rayon::prelude::*;

// Sequential
let results: Vec<_> = items.iter().map(|i| process(i)).collect();

// Parallel
let results: Vec<_> = items.par_iter().map(|i| process(i)).collect();
```

## Connection Pool Sizing

```rust
// Formula: (2 * num_cpu_cores) + num_disks
let pool = PgPoolOptions::new()
    .max_connections((2 * num_cpus::get() + 1) as u32)
    .connect(database_url)
    .await?;
```

## Index Optimization

```sql
-- Composite index for common query pattern
CREATE INDEX idx_fic_info_status_words ON fic_info(status, words);

-- Partial index for active stories
CREATE INDEX idx_fic_info_active ON fic_info(fic_updated)
WHERE status = 'ongoing';

-- Covering index to avoid table lookups
CREATE INDEX idx_fic_info_covering ON fic_info(status, words)
INCLUDE (title, author, chapters);
```

---

# Advanced Topic: Security Hardening

## Input Validation

```rust
fn validate_url(url: &str) -> Result<(), AppError> {
    if url.len() > 2048 {
        return Err(AppError::BadRequest(-1, "URL too long".into()));
    }
    
    let parsed = Url::parse(url)
        .map_err(|_| AppError::BadRequest(-1, "invalid URL".into()))?;
    
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(AppError::BadRequest(-1, "invalid scheme".into()));
    }
    
    Ok(())
}
```

## SQL Injection Prevention

```rust
// DANGEROUS
let query = format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", search);

// SAFE
let query = "SELECT * FROM fic_info WHERE title LIKE $1";
let pattern = format!("%{}%", search);
sqlx::query(query).bind(&pattern).fetch_all(&pool).await?;
```

## Rate Limiting

```rust
async fn check_rate_limit(
    redis: &mut MultiplexedConnection,
    key: &str,
    max: u32,
) -> Result<(), AppError> {
    let count: Option<u32> = redis.get(key).await.unwrap_or(None);
    if count.unwrap_or(0) >= max {
        return Err(AppError::RateLimited(3600));
    }
    
    let _: () = redis.incr(key, 1).await.unwrap_or_default();
    let _: () = redis.expire(key, 3600).await.unwrap_or_default();
    Ok(())
}
```

## CORS Configuration

```rust
let cors = CorsLayer::new()
    .allow_origin(Origin::list(vec![
        "https://fichub.net".parse().unwrap(),
    ]))
    .allow_methods(Any)
    .allow_headers(Any);
```

## Secrets Management

```rust
// Load from environment
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

// Never log secrets
let safe_url = database_url.replace(
    &database_url[database_url.find('@').unwrap()..database_url.find('/').unwrap()],
    ":***@"
);
tracing::info!("Connecting to {}", safe_url);
```

## Container Security

```dockerfile
# Non-root user
RUN addgroup --system fichub && adduser --system --ingroup fichub fichub
USER fichub

# Read-only filesystem
# docker-compose.yml:
# read_only: true
# tmpfs:
#   - /tmp
```

---

# Advanced Topic: Monitoring and Observability

## Structured Logging

```rust
use tracing::{info, warn, error, debug};

async fn process_request(url: &str) -> Result<(), AppError> {
    info!(url = %url, "Processing request");
    
    let meta = fetch_metadata(url).await?;
    debug!(url_id = %meta.url_id, "Metadata fetched");
    
    if meta.words > 100_000 {
        warn!(url_id = %meta.url_id, words = meta.words, "Large story");
    }
    
    match export_story(&meta).await {
        Ok(hash) => {
            info!(url_id = %meta.url_id, hash = %hash, "Export complete");
            Ok(())
        }
        Err(e) => {
            error!(url_id = %meta.url_id, error = %e, "Export failed");
            Err(e)
        }
    }
}
```

## Spans

```rust
use tracing::{info_span, Instrument};

async fn process_request(url: &str) -> Result<(), AppError> {
    let span = info_span!("request", url = %url);
    let _guard = span.enter();
    
    async {
        info!("Processing started");
        let meta = fetch_metadata(url).await?;
        info!("Metadata fetched");
        export_story(&meta).await
    }.instrument(span).await
}
```

## Prometheus Metrics

```rust
use prometheus::{IntCounter, Histogram, register_int_counter_with_registry};

lazy_static! {
    static ref REQUESTS: IntCounter = register_int_counter_with_registry!(
        "fichub_requests_total", "Total requests"
    ).unwrap();
    
    static ref DURATION: Histogram = register_histogram_with_registry!(
        "fichub_request_duration_seconds", "Request duration"
    ).unwrap();
}

async fn handler() -> impl IntoResponse {
    let timer = DURATION.start_timer();
    let result = process().await;
    timer.observe_duration();
    REQUESTS.inc();
    result
}
```

## Health Checks

```rust
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();
    
    let redis_ok = redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .is_ok();
    
    Ok(Json(json!({
        "status": if db_ok && redis_ok { "healthy" } else { "unhealthy" },
        "database": db_ok,
        "redis": redis_ok
    })))
}
```

---

# Advanced Topic: Troubleshooting

## Common Build Errors

```rust
// Error: missing lifetime specifier
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }

// Error: use of moved value
let s = String::from("hello");
let s2 = s.clone();
println!("{}", s);

// Error: cannot borrow as mutable
let mut data = vec![1, 2, 3];
let first = data[0];
data.push(4);
```

## Runtime Debugging

```rust
// Add debug logging
debug!(input = %input, "Processing");
let result = process(input);
debug!(result = ?result, "Result");

// Add request IDs
let request_id = Uuid::new_v4();
info!(request_id = %request_id, "Processing request");
```

## Memory Leak Detection

```rust
// Use Arc::strong_count
let data = Arc::new(vec![1, 2, 3]);
println!("References: {}", Arc::strong_count(&data));

// Use Weak to break circular references
use std::rc::Weak;
struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}
```

## Database Troubleshooting

```sql
-- Check connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub';

-- Find slow queries
SELECT query, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- Kill idle connections
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'fichub'
AND state = 'idle'
AND query_start < now() - interval '10 minutes';
```

## Network Troubleshooting

```bash
# Test connectivity
ping archiveofourown.org

# Test DNS
nslookup archiveofourown.org

# Test port
nc -zv archiveofourown.org 443

# Test HTTP
curl -v https://archiveofourown.org
```

