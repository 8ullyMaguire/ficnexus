# Extended Content: Additional Tutorial Material

---

# Rust Concurrency Patterns Complete Guide

## Thread-Based Concurrency

### Creating Threads

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("spawned thread: {}", i);
            thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    
    for i in 1..5 {
        println!("main thread: {}", i);
        thread::sleep(std::time::Duration::from_millis(1));
    }
    
    handle.join().unwrap();
}
```

### Message Passing

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let val = String::from("hello");
        tx.send(val).unwrap();
    });
    
    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

### Shared State

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Result: {}", *counter.lock().unwrap());
}
```

## Async Concurrency Patterns

### Task Spawning

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let handle = task::spawn(async {
        some_async_computation().await
    });
    
    let result = handle.await.unwrap();
    println!("Result: {}", result);
}
```

### JoinSet

```rust
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();
    
    for i in 0..10 {
        set.spawn(async move {
            println!("task {}", i);
            i * 2
        });
    }
    
    while let Some(result) = set.join_next().await {
        println!("Result: {}", result.unwrap());
    }
}
```

### Channels

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    let tx2 = tx.clone();
    
    tokio::spawn(async move {
        tx.send("hello from tx").await.unwrap();
    });
    
    tokio::spawn(async move {
        tx2.send("hello from tx2").await.unwrap();
    });
    
    while let Some(message) = rx.recv().await {
        println!("Received: {}", message);
    }
}
```

### Select

```rust
use tokio::select;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    select! {
        _ = sleep(Duration::from_secs(1)) => {
            println!("timeout");
        }
        result = async_operation() => {
            println!("result: {}", result);
        }
    }
}
```

### Broadcast

```rust
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(16);
    
    let tx2 = tx.clone();
    
    tokio::spawn(async move {
        tx.send("hello".to_string()).unwrap();
    });
    
    let mut rx = tx2.subscribe();
    
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            println!("received: {}", msg);
        }
    });
    
    sleep(Duration::from_millis(100)).await;
}
```

### RwLock

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let data = data.clone();
        handles.push(tokio::spawn(async move {
            let mut write = data.write().await;
            write.push(i);
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let read = data.read().await;
    println!("Data: {:?}", *read);
}
```

### Mutex

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let data = data.clone();
        handles.push(tokio::spawn(async move {
            let mut data = data.lock().await;
            data.push(i);
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let data = data.lock().await;
    println!("Data: {:?}", *data);
}
```

### Semaphore

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = vec![];
    
    for i in 0..10 {
        let semaphore = semaphore.clone();
        handles.push(tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            println!("task {} running", i);
            sleep(Duration::from_secs(1)).await;
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Advanced Patterns

### Work Stealing

```rust
use tokio::runtime::Builder;

fn main() {
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    
    rt.block_on(async {
        // Tasks are distributed across worker threads
        // Idle threads steal from busy threads
    });
}
```

### Task Affinity

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let handle = task::spawn_blocking(|| {
        // This runs on a dedicated blocking thread
        // Use for CPU-intensive work
        expensive_computation()
    });
    
    let result = handle.await.unwrap();
}
```

### Rate Limiting

```rust
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

async fn rate_limited_requests(urls: Vec<String>) {
    let semaphore = Arc::new(Semaphore::new(10));
    
    let handles: Vec<_> = urls.into_iter().map(|url| {
        let semaphore = semaphore.clone();
        async move {
            let _permit = semaphore.acquire().await.unwrap();
            sleep(Duration::from_millis(100)).await;
            reqwest::get(&url).await
        }
    }).collect();
    
    for handle in handles {
        let _ = handle.await;
    }
}
```

### Backpressure

```rust
use tokio::sync::mpsc;

async fn producer_consumer() {
    let (tx, mut rx) = mpsc::channel(100);
    
    // Producer
    tokio::spawn(async move {
        for i in 0..1000 {
            tx.send(i).await.unwrap(); // Blocks if buffer full
        }
    });
    
    // Consumer
    while let Some(item) = rx.recv().await {
        process(item).await;
    }
}
```

---

# Complete Error Handling Reference

## Error Types in Rust

### Standard Error Types

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct MyError {
    message: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {}
```

### Using thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("bad request: {0}")]
    BadRequest(String),
    
    #[error("internal error")]
    Internal(#[from] std::io::Error),
    
    #[error("database error")]
    Database(#[from] sqlx::Error),
    
    #[error("network error")]
    Network(#[from] reqwest::Error),
}
```

### Using anyhow

```rust
use anyhow::{Context, Result};

async fn process(url: &str) -> Result<String> {
    let response = reqwest::get(url).await
        .context(format!("failed to fetch {}", url))?;
    
    let text = response.text().await
        .context("failed to read response")?;
    
    Ok(text)
}
```

## Error Propagation Patterns

### The ? Operator

```rust
fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}
```

### From Implementations

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err)
    }
}
```

### Error Context

```rust
use anyhow::Context;

async fn fetch_data(url: &str) -> Result<Data> {
    let response = reqwest::get(url).await
        .context("failed to make HTTP request")?;
    
    let data: Data = response.json().await
        .context("failed to parse JSON")?;
    
    Ok(data)
}
```

## Common Error Patterns

### Custom Error Enums

```rust
#[derive(Debug)]
enum AppError {
    NotFound,
    Unauthorized,
    Forbidden,
    BadRequest(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::NotFound => write!(f, "not found"),
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::Forbidden => write!(f, "forbidden"),
            AppError::BadRequest(msg) => write!(f, "bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
```

### Result Type Aliases

```rust
type AppResult<T> = Result<T, AppError>;

fn process() -> AppResult<String> {
    Ok("success".to_string())
}
```

### Error Conversion at Boundaries

```rust
impl From<AppError> for axum::http::StatusCode {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

---

# Complete Testing Reference

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_addition() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_subtraction() {
        assert_eq!(subtract(5, 3), 2);
    }
    
    #[test]
    #[should_panic(expected = "divide by zero")]
    fn test_divide_by_zero() {
        divide(10, 0);
    }
    
    #[test]
    fn test_result() -> Result<(), Box<dyn std::error::Error>> {
        let result = parse_number("42")?;
        assert_eq!(result, 42);
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
    let response = reqwest::get("http://localhost:3000/api/v0/epub")
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}
```

## Test Fixtures

```rust
async fn setup_test_db() -> PgPool {
    let pool = PgPool::connect("postgres://localhost/test_db")
        .await
        .unwrap();
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    pool
}

async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("DELETE FROM stories").execute(pool).await.unwrap();
}
```

## Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_commutative(a in 0i32..1000, b in 0i32..1000) {
        prop_assert_eq!(add(a, b), add(b, a));
    }
    
    #[test]
    fn test_sort_sorts(v in prop::collection::vec(0i32..100, 0..100)) {
        let mut sorted = v.clone();
        sorted.sort();
        for i in 0..sorted.len()-1 {
            prop_assert!(sorted[i] <= sorted[i+1]);
        }
    }
}
```

## Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_add(c: &mut Criterion) {
    c.bench_function("add", |b| {
        b.iter(|| add(black_box(2), black_box(3)))
    });
}

criterion_group!(benches, benchmark_add);
criterion_main!(benches);
```

---

# Complete Deployment Reference

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.77-bookworm as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3
WORKDIR /app
COPY --from=builder /app/target/release/fichub .
EXPOSE 3000
CMD ["./fichub"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: fichub
      POSTGRES_USER: fichub
      POSTGRES_PASSWORD: fichub
    volumes:
      - pgdata:/var/lib/postgresql/data
  
  redis:
    image: redis:7-alpine
  
  fichub:
    build: .
    environment:
      DATABASE_URL: postgres://fichub:fichub@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
    depends_on:
      - postgres
      - redis

volumes:
  pgdata:
```

## Systemd Deployment

```ini
[Unit]
Description=FicHub Backend
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/opt/fichub
EnvironmentFile=/opt/fichub/.env
ExecStart=/opt/fichub/fichub
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Nginx Configuration

```nginx
server {
    listen 80;
    server_name fichub.net;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    location /static/ {
        alias /opt/fichub/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

---

# Complete Configuration Reference

## Environment Variables

```bash
# Required
DATABASE_URL=postgres://user:pass@host/db
REDIS_URL=redis://host:port

# Optional
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug

# Rate Limiting
DYNAMIC_RATE_LIMIT=true
TRUSTED_PROXIES=10.0.0.1,10.0.0.2

# Recommender
REC_DEFAULT_DELAY_SECS=5
REC_SITE_RATE_LIMITS={"ao3": 10, "ffn": 5}
REC_MAX_FAVOURITE_PAGES=3
REC_MAX_RECOMMENDATIONS=20
REC_MIN_FAVOURITERS_FOR_COLLAB=5
REC_VOTING_BOOST_GAMMA=0.2
REC_CACHE_TTL_HOURS=12
REC_SUGGEST_LIMIT_PER_HOUR=5
REC_VOTE_LIMIT_PER_HOUR=10

# Tagging
CURATOR_TOKEN=secret-token
TAG_HIDDEN_THRESHOLD=-3
TAG_AUTO_DELETE_THRESHOLD=-5
TAG_SUBMIT_LIMIT_PER_HOUR=10
TAG_VOTE_LIMIT_PER_HOUR=20

# Search
SEARCH_MAX_PER_PAGE=50

# OPDS
OPDS_SHELF_TOKEN=fichub
```

## Cargo.toml

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate", "tls-rustls-ring", "derive", "macros"] }
redis = { version = "1.4", features = ["aio", "tokio-comp"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
scraper = "0.27"
epub-builder = "0.8"
tera = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
md5 = "0.7"
sha2 = "0.10"
hex = "0.4"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
regex-lite = "0.1"
async-trait = "0.1"
zip = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
dotenvy = "0.15"
thiserror = "2"
ipnet = "2"
url = "2"
```

---

# Complete Troubleshooting Reference

## Build Errors

### Missing Lifetime

```rust
// Error
fn first_word(s: &str) -> &str { /* ... */ }

// Fix
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }
```

### Moved Value

```rust
// Error
let s = String::from("hello");
let s2 = s;
println!("{}", s);

// Fix
let s = String::from("hello");
let s2 = s.clone();
println!("{}", s);
```

### Type Mismatch

```rust
// Error
let x: i32 = "hello";

// Fix
let x: i32 = "hello".parse().unwrap();
```

## Runtime Errors

### Connection Refused

```bash
# Check if service is running
sudo systemctl status postgresql

# Check port
ss -tlnp | grep 5432

# Test connection
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

### Authentication Failed

```bash
# Check pg_hba.conf
sudo cat /etc/postgresql/16/main/pg_hba.conf

# Update auth method
sudo nano /etc/postgresql/16/main/pg_hba.conf
# Change "peer" to "md5"

# Restart
sudo systemctl restart postgresql
```

### Too Many Connections

```sql
-- Check connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub';

-- Kill idle connections
SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = 'fichub'
AND state = 'idle'
AND query_start < now() - interval '10 minutes';
```

## Performance Issues

### Slow Queries

```sql
-- Enable slow query logging
ALTER SYSTEM SET log_min_duration_statement = 1000;
SELECT pg_reload_conf();

-- Check slow queries
SELECT query, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

### Memory Leaks

```rust
// Use Arc::strong_count to check references
let data = Arc::new(vec![1, 2, 3]);
println!("References: {}", Arc::strong_count(&data));

// Use Weak to break circular references
use std::rc::Weak;
struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}
```

### Connection Pool Exhaustion

```rust
let stats = pool.stats();
tracing::info!(
    active = stats.active_connections(),
    idle = stats.idle_connections(),
    waiting = stats.waiting(),
    "Pool stats"
);
```

## Network Issues

### DNS Resolution

```bash
nslookup archiveofourown.org
cat /etc/hosts
```

### TLS Issues

```rust
let client = reqwest::Client::builder()
    .use_rustls_tls()
    .build()?;
```

### Timeout Issues

```rust
let client = reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(10))
    .timeout(Duration::from_secs(30))
    .build()?;
```

---

# Complete Security Reference

## Input Validation

```rust
fn validate_url(url: &str) -> Result<(), AppError> {
    if url.len() > 2048 {
        return Err(AppError::BadRequest("URL too long".into()));
    }
    
    let parsed = Url::parse(url)
        .map_err(|_| AppError::BadRequest("invalid URL".into()))?;
    
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(AppError::BadRequest("invalid scheme".into()));
    }
    
    Ok(())
}
```

## SQL Injection Prevention

```rust
// DANGEROUS
let query = format!("SELECT * FROM stories WHERE title LIKE '%{}%'", search);

// SAFE
let query = "SELECT * FROM stories WHERE title LIKE $1";
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

# Complete Monitoring Reference

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

## Prometheus Metrics

```rust
use prometheus::{IntCounter, Histogram};

lazy_static! {
    static ref REQUESTS: IntCounter = IntCounter::new(
        "fichub_requests_total", "Total requests"
    ).unwrap();
    
    static ref DURATION: Histogram = Histogram::new(
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

## Alerting Rules

```yaml
groups:
- name: fichub
  rules:
  - alert: HighErrorRate
    expr: rate(fichub_http_requests_total{status=~"5.."}[5m]) > 0.05
    for: 5m
    labels:
      severity: critical
  
  - alert: HighLatency
    expr: histogram_quantile(0.95, rate(fichub_http_request_duration_seconds_bucket[5m])) > 2
    for: 5m
    labels:
      severity: warning
```

---

# Complete Glossary

## Rust Terms

**Ownership** — Rust's memory management system where each value has exactly one owner.

**Borrowing** — Referencing data without taking ownership.

**Lifetime** — The scope during which a reference is valid.

**Trait** — A collection of methods that define shared behavior.

**Generic** — A parameterized type that works with multiple concrete types.

**Enum** — A type that can be one of several variants.

**Pattern Matching** — A mechanism for destructuring values and branching.

**Closure** — An anonymous function that can capture variables from its scope.

**Iterator** — A type that produces a sequence of values.

**Smart Pointer** — A pointer that provides additional functionality.

**Arc** — Atomic Reference Counted pointer for thread-safe shared ownership.

**Mutex** — Mutual exclusion lock for thread-safe data access.

**RwLock** — Read-write lock allowing multiple readers or single writer.

**Semaphore** — A counter that limits concurrent access to a resource.

**Channel** — A communication mechanism between concurrent tasks.

**Future** — A value representing a computation that will complete in the future.

**Async/Await** — Syntax for asynchronous programming.

**Executor** — The component that polls futures to make progress.

**Task** — A unit of work executed by the runtime.

**Spawn** — Create a new concurrent task.

**Join** — Wait for a task to complete.

**Select** — Wait for the first of multiple futures to complete.

## Web Terms

**HTTP** — HyperText Transfer Protocol for transferring web data.

**REST** — Representational State Transfer architectural style.

**API** — Application Programming Interface.

**JSON** — JavaScript Object Notation data format.

**CORS** — Cross-Origin Resource Sharing.

**Middleware** — Code that processes requests before/after handlers.

**Router** — Component that maps URLs to handlers.

**Handler** — Function that processes an HTTP request.

**Extractor** — Component that extracts data from requests.

**Response** — Data sent back to the client.

**Status Code** — Three-digit number indicating request result.

**Header** — Metadata in HTTP requests/responses.

**Body** — Payload of HTTP requests/responses.

**Endpoint** — A specific URL that handles requests.

**Route** — A URL pattern mapped to a handler.

## Database Terms

**SQL** — Structured Query Language for managing databases.

**PostgreSQL** — Open-source relational database.

**Table** — Collection of related data in rows and columns.

**Row** — A single record in a table.

**Column** — A field in a table.

**Index** — Data structure that speeds up queries.

**Primary Key** — Column that uniquely identifies each row.

**Foreign Key** — Column that references another table's primary key.

**Query** — Request for data from the database.

**Transaction** — Group of operations treated as a single unit.

**Migration** — Version-controlled schema change.

**Connection Pool** — Cache of reusable database connections.

**ORM** — Object-Relational Mapping library.

**Schema** — Structure of a database.

**Query Planner** — Component that optimizes query execution.

## DevOps Terms

**Docker** — Platform for containerized applications.

**Container** — Lightweight isolated application environment.

**Image** — Read-only template for containers.

**Dockerfile** — Script that builds Docker images.

**Docker Compose** — Tool for multi-container applications.

**Kubernetes** — Container orchestration platform.

**CI/CD** — Continuous Integration/Continuous Deployment.

**Git** — Version control system.

**Systemd** — Linux init system and service manager.

**Nginx** — Web server and reverse proxy.

**TLS** — Transport Layer Security for encrypted connections.

**SSL** — Secure Sockets Layer (deprecated, replaced by TLS).

**Certificate** — Digital document proving identity.

**Load Balancer** — Distributes traffic across servers.

**CDN** — Content Delivery Network for distributed content.

