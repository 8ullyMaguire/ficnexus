# Extended Content: Final Comprehensive Expansion

---

# Complete Rust Type System Reference

## Primitive Types

### Integers

```rust
let a: i8 = 127;        // 8-bit signed
let b: u8 = 255;         // 8-bit unsigned
let c: i16 = 32767;      // 16-bit signed
let d: u16 = 65535;      // 16-bit unsigned
let e: i32 = 2147483647;  // 32-bit signed
let f: u32 = 4294967295;  // 32-bit unsigned
let g: i64 = 9223372036854775807; // 64-bit signed
let h: u64 = 18446744073709551615; // 64-bit unsigned

let x = 42; // i32 (default)

let million = 1_000_000;
let binary = 0b1111_0000;
let hex = 0xFF;
let octal = 0o77;
```

### Floating-Point

```rust
let x = 2.0;      // f64 (default)
let y: f32 = 3.0;  // f32

let million = 1e6;
let tiny = 1.5e-10;

let inf = f64::INFINITY;
let nan = f64::NAN;
```

### Boolean

```rust
let t = true;
let f: bool = false;

let and = true && false;   // false
let or = true || false;    // true
let not = !true;           // false
```

### Character

```rust
let c = 'z';
let z: char = 'ℤ';
let heart = '❤';
let chinese = '中';

println!("Size of char: {}", std::mem::size_of::<char>());  // 4
```

## Compound Types

### Tuple

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;
let five_hundred = tup.0;

let unit = ();
```

### Array

```rust
let a = [1, 2, 3, 4, 5];  // [i32; 5]
let a: [i32; 5] = [1, 2, 3, 4, 5];
let a = [3; 5];  // [3, 3, 3, 3, 3]

let first = a[0];
let second = a[1];
```

### Slice

```rust
let a = [1, 2, 3, 4, 5];
let slice = &a[1..3];  // [2, 3]
let slice = &a[..];    // entire array
let slice = &a[2..];   // [3, 4, 5]
let slice = &a[..3];   // [1, 2, 3]
```

## Reference Types

### Shared Reference

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    println!("The length of '{}' is {}.", s1, len);
}
```

### Mutable Reference

```rust
fn change(s: &mut String) {
    s.push_str(", world");
}

fn main() {
    let mut s = String::from("hello");
    change(&mut s);
    println!("{}", s);  // "hello, world"
}
```

## Smart Pointers

### Box<T>

```rust
let b = Box::new(5);
println!("b = {}", b);

enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Nil))));
```

### Rc<T>

```rust
use std::rc::Rc;

let a = Rc::new(vec![1, 2, 3]);
let b = Rc::clone(&a);
let c = Rc::clone(&a);

println!("Reference count: {}", Rc::strong_count(&a));  // 3
```

### Arc<T>

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3]);
let mut handles = vec![];

for _ in 0..10 {
    let data = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        println!("{:?}", *data);
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

### RefCell<T>

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

data.borrow_mut().push(4);

println!("{:?}", *data.borrow());
```

## Enum Types

### Basic Enum

```rust
enum IpAddrKind {
    V4,
    V6,
}

let home = IpAddrKind::V4;
let loopback = IpAddrKind::V6;
```

### Enum with Data

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

let msg = Message::Write(String::from("hello"));
```

### Option<T>

```rust
let some_number: Option<i32> = Some(5);
let some_string: Option<String> = Some(String::from("hello"));
let absent_number: Option<i32> = None;

match some_number {
    Some(n) => println!("Number: {}", n),
    None => println!("No number"),
}
```

### Result<T, E>

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".into())
    } else {
        Ok(a / b)
    }
}

match divide(10.0, 2.0) {
    Ok(result) => println!("10 / 2 = {}", result),
    Err(e) => println!("Error: {}", e),
}
```

## Trait Types

### Basic Trait

```rust
trait Summary {
    fn summarize(&self) -> String;
    
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..50])
    }
}

struct Article {
    title: String,
    author: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}, by {}", self.title, self.author)
    }
}
```

### Trait Objects

```rust
let articles: Vec<Box<dyn Summary>> = vec![
    Box::new(Article { title: "Hello".into(), author: "World".into() }),
];

for article in &articles {
    println!("{}", article.summarize());
}
```

## Generic Types

### Basic Generics

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100, 65];
    let result = largest(&numbers);
    println!("Largest number: {}", result);
}
```

### Generic Structs

```rust
struct Point<T> {
    x: T,
    y: T,
}

let integer_point = Point { x: 5, y: 10 };
let float_point = Point { x: 1.0, y: 4.0 };
```

## Lifetime Types

### Basic Lifetime

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("long string");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(&string1, &string2);
        println!("Longest: {}", result);
    }
}
```

## Closure Types

### Fn Traits

```rust
let add_one = |x| x + 1;
let five = add_one(4);

let mut list = vec![1, 2, 3];
let mut push_value = || list.push(4);
push_value();

let name = String::from("Alice");
let greet = || println!("Hello, {}!", name);
greet();
```

### Closure as Parameter

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

let result = apply(|x| x * 2, 5);
println!("Result: {}", result);
```

## Iterator Types

### Iterator Adaptors

```rust
let v = vec![1, 2, 3, 4, 5];

let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
let evens: Vec<&i32> = v.iter().filter(|x| *x % 2 == 0).collect();
let sum: i32 = v.iter().sum();
let has_even = v.iter().any(|x| x % 2 == 0);
let first_even = v.iter().find(|x| *x % 2 == 0);
let indexed: Vec<(usize, &i32)> = v.iter().enumerate().collect();
```

---

# Complete Axum Framework Reference

## Extractors

### Query

```rust
#[derive(Deserialize)]
struct Params {
    name: String,
    age: Option<u32>,
}

async fn handler(Query(params): Query<Params>) -> Json<Value> {
    Json(json!({"name": params.name, "age": params.age}))
}
```

### Path

```rust
async fn handler(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"id": id}))
}
```

### JSON Body

```rust
#[derive(Deserialize)]
struct CreateItem {
    name: String,
    description: String,
}

async fn handler(Json(payload): Json<CreateItem>) -> Json<Value> {
    Json(json!({"created": payload.name}))
}
```

### Headers

```rust
async fn handler(headers: HeaderMap) -> Json<Value> {
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    Json(json!({"user_agent": user_agent}))
}
```

### State

```rust
struct AppState {
    db: PgPool,
}

async fn handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({"status": "ok"}))
}
```

## Responses

### JSON

```rust
async fn handler() -> Json<Value> {
    Json(json!({"key": "value"}))
}
```

### Custom Response

```rust
async fn handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [("X-Custom", "value")],
        Json(json!({"key": "value"}))
    )
}
```

## Router

### Basic Routes

```rust
let app = Router::new()
    .route("/", get(index))
    .route("/users", get(list_users).post(create_user))
    .route("/users/{id}", get(get_user).put(update_user).delete(delete_user));
```

### Nested Routes

```rust
let api_routes = Router::new()
    .route("/users", get(list_users))
    .route("/posts", get(list_posts));

let app = Router::new()
    .nest("/api/v1", api_routes);
```

## Middleware

### From Fn Middleware

```rust
async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    println!("Request took: {:?}", duration);
    response
}

let app = Router::new()
    .route("/", get(handler))
    .layer(axum::middleware::from_fn(logging_middleware));
```

---

# Complete SQLx Reference

## Query Types

### Execute

```rust
sqlx::query("INSERT INTO users (name) VALUES ($1)")
    .bind("Alice")
    .execute(&pool)
    .await?;
```

### Fetch One

```rust
let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
    .bind(1)
    .fetch_one(&pool)
    .await?;
```

### Fetch All

```rust
let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
    .fetch_all(&pool)
    .await?;
```

## Transactions

```rust
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO users (name) VALUES ($1)")
    .bind("Alice")
    .execute(&mut *tx)
    .await?;

sqlx::query("INSERT INTO posts (user_id, title) VALUES ($1, $2)")
    .bind(1)
    .bind("Hello World")
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

## Query Builder

```rust
use sqlx::QueryBuilder;

let mut qb = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

if let Some(name) = &params.name {
    qb.push(" AND name = ");
    qb.push_bind(name.clone());
}

let users: Vec<User> = qb
    .build_query_as()
    .fetch_all(&pool)
    .await?;
```

---

# Complete Tokio Reference

## Runtime

### Builder

```rust
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .thread_name("my-worker")
    .build()
    .unwrap();
```

## Tasks

### Spawn

```rust
tokio::spawn(async {
    work().await;
});
```

### JoinSet

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

for i in 0..10 {
    set.spawn(async move {
        work(i).await
    });
}

while let Some(result) = set.join_next().await {
    println!("Result: {}", result?);
}
```

## Channels

### mpsc

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    tx.send("hello").await.unwrap();
});

while let Some(msg) = rx.recv().await {
    println!("Received: {}", msg);
}
```

## Synchronization

### Mutex

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(vec![]));

let data2 = data.clone();
tokio::spawn(async move {
    let mut data = data2.lock().await;
    data.push(1);
});
```

### Semaphore

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(3));

let mut handles = vec![];
for i in 0..10 {
    let sem = semaphore.clone();
    handles.push(tokio::spawn(async move {
        let _permit = sem.acquire().await.unwrap();
        println!("Task {} running", i);
        work().await;
    }));
}
```

## Time

### Sleep

```rust
tokio::time::sleep(Duration::from_secs(5)).await;
```

### Timeout

```rust
use tokio::time::timeout;

let result = timeout(
    Duration::from_secs(30),
    async { work().await }
).await;
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

---

# Complete Serde Reference

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

## Tagged Enums

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ExportResult {
    #[serde(rename = "epub")]
    Epub { hash: String },
    #[serde(rename = "html")]
    Html { hash: String },
    #[serde(rename = "error")]
    Error { message: String },
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

# Tagging
CURATOR_TOKEN=secret-token
TAG_HIDDEN_THRESHOLD=-3
TAG_SUBMIT_LIMIT_PER_HOUR=10
TAG_VOTE_LIMIT_PER_HOUR=20

# Search
SEARCH_MAX_PER_PAGE=50
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

# Complete Deployment Reference

## Docker

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

## Systemd

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

## Nginx

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
}
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

## Runtime Errors

### Connection Refused

```bash
sudo systemctl status postgresql
ss -tlnp | grep 5432
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

### Too Many Connections

```sql
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub';
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'fichub' AND state = 'idle';
```

## Performance Issues

### Slow Queries

```sql
ALTER SYSTEM SET log_min_duration_statement = 1000;
SELECT pg_reload_conf();
SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;
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
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let safe_url = database_url.replace(
    &database_url[database_url.find('@').unwrap()..database_url.find('/').unwrap()],
    ":***@"
);
tracing::info!("Connecting to {}", safe_url);
```

## Container Security

```dockerfile
RUN addgroup --system fichub && adduser --system --ingroup fichub fichub
USER fichub
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

