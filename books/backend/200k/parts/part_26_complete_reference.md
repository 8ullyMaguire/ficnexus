# Extended Content: Comprehensive Reference Manual

---

# Complete Rust Type System Reference

## Primitive Types

### Integers

Rust provides several integer types with different sizes:

```rust
// Signed integers
let a: i8 = 127;        // 8-bit, range: -128 to 127
let b: i16 = 32767;     // 16-bit, range: -32,768 to 32,767
let c: i32 = 2147483647; // 32-bit, range: -2.1B to 2.1B
let d: i64 = 9223372036854775807; // 64-bit

// Unsigned integers
let e: u8 = 255;         // 8-bit, range: 0 to 255
let f: u16 = 65535;      // 16-bit
let g: u32 = 4294967295; // 32-bit
let h: u64 = 18446744073709551615; // 64-bit

// Default type is i32
let x = 42; // i32

// Underscores for readability
let million = 1_000_000;
let binary = 0b1111_0000;
let hex = 0xFF;
let octal = 0o77;
```

### Floating-Point

```rust
let x = 2.0;      // f64 (default)
let y: f32 = 3.0;  // f32

// Scientific notation
let million = 1e6;
let tiny = 1.5e-10;

// Special values
let inf = f64::INFINITY;
let nan = f64::NAN;
let neg_inf = f64::NEG_INFINITY;
```

### Boolean

```rust
let t = true;
let f: bool = false;

// Logical operators
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

// Character is 4 bytes (Unicode)
println!("Size of char: {}", std::mem::size_of::<char>());  // 4

// Escape sequences
let newline = '\n';
let tab = '\t';
let null = '\0';
let backslash = '\\';
let quote = '"';
```

## Compound Types

### Tuple

```rust
let tup: (i32, f64, u8) = (500, 6.4, 1);
let (x, y, z) = tup;  // Destructuring
let five_hundred = tup.0;  // Index access
let six_point_four = tup.1;
let one = tup.2;

// Unit type (empty tuple)
let unit = ();
```

### Array

```rust
let a = [1, 2, 3, 4, 5];  // [i32; 5]
let a: [i32; 5] = [1, 2, 3, 4, 5];
let a = [3; 5];  // [3, 3, 3, 3, 3]

let first = a[0];
let second = a[1];

// Arrays are fixed-size, stack-allocated
// For dynamic size, use Vec<T>
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

### Dangling Reference

```rust
// This won't compile
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // s is dropped, reference is invalid
}

// Solution: return ownership
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // ownership is moved to caller
}
```

## Smart Pointers

### Box<T>

```rust
// Heap allocation
let b = Box::new(5);
println!("b = {}", b);

// Recursive types
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
let b = Rc::clone(&a);  // Increment reference count
let c = Rc::clone(&a);  // Increment reference count

println!("Reference count: {}", Rc::strong_count(&a));  // 3

// When a goes out of scope, count decrements
drop(a);
println!("Reference count: {}", Rc::strong_count(&b));  // 2
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

// Mutable borrow
data.borrow_mut().push(4);

// Immutable borrow
println!("{:?}", *data.borrow());
```

### Cell<T>

```rust
use std::cell::Cell;

let counter = Cell::new(0);
counter.set(counter.get() + 1);
println!("Counter: {}", counter.get());
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
enum Option<T> {
    None,
    Some(T),
}

let some_number: Option<i32> = Some(5);
let some_string: Option<String> = Some(String::from("hello"));
let absent_number: Option<i32> = None;

// Pattern matching
match some_number {
    Some(n) => println!("Number: {}", n),
    None => println!("No number"),
}
```

### Result<T, E>

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

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

### Trait with Default

```rust
trait Summary {
    fn summarize(&self) -> String;
    
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..50])
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

### Generic Methods

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
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

### Lifetime in Structs

```rust
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
    
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {}", announcement);
        self.part
    }
}
```

## Closure Types

### Fn Traits

```rust
// Fn: immutable borrow
let add_one = |x| x + 1;
let five = add_one(4);

// FnMut: mutable borrow
let mut list = vec![1, 2, 3];
let mut push_value = || list.push(4);
push_value();

// FnOnce: take ownership
let name = String::from("Alice");
let greet = || println!("Hello, {}!", name);
greet();
// name is now moved into the closure
```

### Closure as Parameter

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

let result = apply(|x| x * 2, 5);
println!("Result: {}", result);
```

### Closure as Return Value

```rust
fn make_adder(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

let add_five = make_adder(5);
println!("5 + 3 = {}", add_five(3));
```

## Iterator Types

### Iterator Trait

```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Counter {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

### Iterator Adaptors

```rust
let v = vec![1, 2, 3, 4, 5];

// Map
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();

// Filter
let evens: Vec<&i32> = v.iter().filter(|x| *x % 2 == 0).collect();

// Fold
let sum: i32 = v.iter().fold(0, |acc, x| acc + x);

// Any
let has_even = v.iter().any(|x| x % 2 == 0);

// Find
let first_even = v.iter().find(|x| *x % 2 == 0);

// Enumerate
let indexed: Vec<(usize, &i32)> = v.iter().enumerate().collect();

// Zip
let names = vec!["Alice", "Bob"];
let ages = vec![25, 30];
let people: Vec<(&str, &i32)> = names.iter().zip(ages.iter()).collect();

// Chain
let combined: Vec<i32> = v1.iter().chain(v2.iter()).cloned().collect();
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

async fn handler(Path((id, name)): Path<(String, String)>) -> Json<Value> {
    Json(json!({"id": id, "name": name}))
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
    // Use state.db
    Json(json!({"status": "ok"}))
}
```

### ConnectInfo

```rust
async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    Json(json!({"ip": addr.ip().to_string()}))
}
```

## Responses

### JSON

```rust
async fn handler() -> Json<Value> {
    Json(json!({"key": "value"}))
}
```

### Status Code

```rust
async fn handler() -> StatusCode {
    StatusCode::OK
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

### File Download

```rust
async fn handler() -> impl IntoResponse {
    let file = fs::read("file.epub").await.unwrap();
    (
        [
            (header::CONTENT_TYPE, "application/epub+zip"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"story.epub\""),
        ],
        file
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

### With State

```rust
let state = Arc::new(AppState { db });
let app = Router::new()
    .route("/", get(handler))
    .with_state(state);
```

### With Middleware

```rust
let app = Router::new()
    .route("/", get(handler))
    .layer(CorsLayer::permissive())
    .layer(TraceLayer::new_for_http());
```

## Middleware

### Tower Layer

```rust
use tower::Layer;

struct MyLayer;

impl<S> Layer<S> for MyLayer {
    type Service = MyMiddleware<S>;
    
    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware { inner }
    }
}
```

### Tower Service

```rust
use tower::Service;

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
        self.inner.call(req)
    }
}
```

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

### Fetch Optional

```rust
let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE id = $1")
    .bind(1)
    .fetch_optional(&pool)
    .await?;
```

### Fetch All

```rust
let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
    .fetch_all(&pool)
    .await?;
```

### Fetch Scalar

```rust
let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
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

if let Some(min_age) = params.min_age {
    qb.push(" AND age >= ");
    qb.push_bind(min_age);
}

let users: Vec<User> = qb
    .build_query_as()
    .fetch_all(&pool)
    .await?;
```

## Offline Mode

```bash
# Save database snapshot
cargo sqlx prepare

# Check queries offline
SQLX_OFFLINE=true cargo check
```

## Migrations

```rust
// Run migrations
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;

// Check migration status
let migrations = sqlx::migrate!("./migrations");
for migration in migrations.iter() {
    println!("Migration: {}", migration.description());
}
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
    .on_thread_start(|| {
        println!("Thread started");
    })
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

### Spawn Blocking

```rust
let result = tokio::task::spawn_blocking(|| {
    // CPU-intensive work
    expensive_computation()
}).await?;
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

### broadcast

```rust
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel(100);

let tx2 = tx.clone();
tokio::spawn(async move {
    tx.send("hello").unwrap();
});

let mut rx = tx2.subscribe();
while let Ok(msg) = rx.recv().await {
    println!("Received: {}", msg);
}
```

### oneshot

```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel();

tokio::spawn(async move {
    tx.send("hello").unwrap();
});

let msg = rx.await.unwrap();
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

let data3 = data.clone();
tokio::spawn(async move {
    let mut data = data3.lock().await;
    data.push(2);
});
```

### RwLock

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

let data = Arc::new(RwLock::new(vec![]));

// Multiple readers
let data2 = data.clone();
tokio::spawn(async move {
    let data = data2.read().await;
    println!("{:?}", *data);
});

// Single writer
let data3 = data.clone();
tokio::spawn(async move {
    let mut data = data3.write().await;
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
    async { work().await }
).await;

match result {
    Ok(result) => println!("Success: {:?}", result),
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

### File

```rust
use tokio::fs;

let content = fs::read_to_string("data.txt").await?;
fs::write("output.txt", "Hello, world!").await?;
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

## Attributes

```rust
// Default value
#[serde(default)]

// Skip if None
#[serde(skip_serializing_if = "Option::is_none")]

// Rename field
#[serde(rename = "camelCase")]

// Flatten nested struct
#[serde(flatten)]

// Custom serialization
#[serde(with = "module")]

// Skip entirely
#[serde(skip)]
```

## Custom Serialization

```rust
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
        i64::deserialize(deserializer)
    }
}

#[derive(Serialize, Deserialize)]
struct Story {
    #[serde(with = "timestamp_millis")]
    published: i64,
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
}
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
sudo systemctl status postgresql
ss -tlnp | grep 5432
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

### Authentication Failed

```bash
sudo cat /etc/postgresql/16/main/pg_hba.conf
sudo nano /etc/postgresql/16/main/pg_hba.conf
sudo systemctl restart postgresql
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

### Memory Leaks

```rust
let data = Arc::new(vec![1, 2, 3]);
println!("References: {}", Arc::strong_count(&data));
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

