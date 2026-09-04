# Part 9: Advanced Rust Patterns

---

# Chapter 37: The Builder Pattern in Practice

The builder pattern is a creational pattern that lets you construct complex objects step by step. In Rust, it's particularly useful for configuring structs with many optional fields.

## Why Use the Builder Pattern?

Consider a configuration struct with 20+ fields:

```rust
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub port: u16,
    pub max_connections: u32,
    pub timeout: Duration,
    pub log_level: String,
    // ... 15 more fields
}
```

Creating this with a struct literal is verbose and error-prone. The builder pattern provides a cleaner API:

```rust
let config = ConfigBuilder::new()
    .database_url("postgres://localhost/fichub")
    .redis_url("redis://localhost")
    .port(3000)
    .max_connections(20)
    .build()
    .unwrap();
```

## Implementing a Builder

```rust
use std::path::PathBuf;
use std::time::Duration;

pub struct ConfigBuilder {
    database_url: Option<String>,
    redis_url: Option<String>,
    cache_dir: PathBuf,
    port: u16,
    max_connections: u32,
    timeout: Duration,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            database_url: None,
            redis_url: None,
            cache_dir: PathBuf::from("./cache"),
            port: 3000,
            max_connections: 20,
            timeout: Duration::from_secs(30),
        }
    }
    
    pub fn database_url(mut self, url: &str) -> Self {
        self.database_url = Some(url.to_string());
        self
    }
    
    pub fn redis_url(mut self, url: &str) -> Self {
        self.redis_url = Some(url.to_string());
        self
    }
    
    pub fn cache_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = dir;
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
    
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn build(self) -> Result<Config, String> {
        let database_url = self.database_url
            .ok_or("database_url is required")?;
        let redis_url = self.redis_url
            .ok_or("redis_url is required")?;
        
        Ok(Config {
            database_url,
            redis_url,
            cache_dir: self.cache_dir,
            port: self.port,
            max_connections: self.max_connections,
            timeout: self.timeout,
        })
    }
}

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub port: u16,
    pub max_connections: u32,
    pub timeout: Duration,
}
```

### Key Design Decisions

1. **Ownership:** The builder takes ownership of `self` in each method and returns it. This enables method chaining.

2. **Option<T>:** Required fields are `Option<T>` in the builder, checked at build time.

3. **Defaults:** Optional fields have sensible defaults in `new()`.

4. **Error handling:** The `build()` method returns `Result<Config, String>` to report missing required fields.

## FicHub's Use of Builders

FicHub uses the builder pattern for the reqwest HTTP client:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(10))
    .pool_max_idle_per_host(10)
    .build()
    .expect("Failed to build HTTP client");
```

Each `.method()` call configures one aspect of the client. The `.build()` call creates the final client, validating that the configuration is valid.

## 📝 Practice Exercises

1. **Build a Scraper Config Builder:** Create a builder for scraper configuration with fields for timeout, retry count, and user agent.

2. **Validation:** Add validation to the builder. For example, ensure the port is between 1 and 65535.

3. **Default Values:** Add a `with_defaults()` method that returns a builder with production-ready defaults.

---

# Chapter 38: Traits and Dynamic Dispatch

Traits are Rust's mechanism for defining shared behavior. They're similar to interfaces in Java or type classes in Haskell.

## Trait Definitions

```rust
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

### Trait Bounds

Trait bounds specify that a type must implement certain traits:

```rust
// Generic function with trait bound
fn find_scraper<T: SiteScraper>(scrapers: &[T], url: &str) -> Option<&T> {
    scrapers.iter().find(|s| s.can_handle(url))
}

// Where clause (alternative syntax)
fn process_scraper<T>(scraper: &T, url: &str) -> bool
where
    T: SiteScraper + std::fmt::Debug,
{
    println!("Processing: {:?}", scraper);
    scraper.can_handle(url)
}
```

### Trait Objects

Trait objects enable dynamic dispatch — the method to call is determined at runtime:

```rust
// Static dispatch (compile-time)
fn process_static(scraper: &impl SiteScraper, url: &str) -> bool {
    scraper.can_handle(url)
}

// Dynamic dispatch (runtime)
fn process_dynamic(scraper: &dyn SiteScraper, url: &str) -> bool {
    scraper.can_handle(url)
}
```

The `dyn SiteScraper` is a "fat pointer" — it contains a pointer to the data and a pointer to the vtable (virtual method table) for the trait.

### When to Use Each

- **Static dispatch:** When you know the concrete type at compile time. Slightly faster (can be inlined).
- **Dynamic dispatch:** When you need heterogeneous collections or runtime polymorphism. More flexible but slightly slower.

FicHub uses dynamic dispatch for the scraper registry because it stores different scraper types in a single collection:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,  // Dynamic dispatch
}
```

## Default Implementations

Traits can provide default implementations:

```rust
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    
    // Default implementation
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

## 📝 Practice Exercises

1. **Trait Design:** Design a trait `Cacheable` with methods for computing cache keys and checking cache validity.

2. **Trait Objects:** Create a collection of trait objects and demonstrate dynamic dispatch.

3. **Blanket Implementations:** Implement `From<&str>` for a custom type and demonstrate automatic conversions.

---

# Chapter 39: Smart Pointers and Interior Mutability

Smart pointers provide ownership semantics beyond simple references. Interior mutability allows modifying data through shared references.

## Box<T>

`Box<T>` provides heap allocation:

```rust
// A recursive type that requires heap allocation
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Nil))));
```

## Arc<T>

`Arc<T>` (Atomic Reference Counted) enables shared ownership across threads:

```rust
use std::sync::Arc;

let state = Arc::new(AppState { /* ... */ });
let state_clone = state.clone();  // Increments reference count

// Both `state` and `state_clone` point to the same data
```

### Arc in FicHub

FicHub wraps `AppState` in `Arc` so it can be shared across async tasks:

```rust
let state = Arc::new(AppState {
    config,
    db: db_pool,
    redis: redis_conn,
    // ...
});

// Pass to router
let app = Router::new()
    .route("/", get(handler))
    .with_state(state);  // state is moved, but Arc clone is cheap
```

## Mutex<T> and RwLock<T>

Mutex and RwLock provide interior mutability with synchronization:

```rust
use std::sync::Mutex;

let data = Mutex::new(Vec::new());
{
    let mut guard = data.lock().unwrap();
    guard.push(42);
}  // Lock is released when guard is dropped

let data = tokio::sync::RwLock::new(Vec::new());
{
    let mut write_guard = data.write().await;
    write_guard.push(42);
}
{
    let read_guard = data.read().await;
    println!("{:?}", *read_guard);
}
```

### RwLock in FicHub

FicHub uses `RwLock` for the datacenter IP set:

```rust
pub struct RedisBucketLimiter {
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
}

// Multiple readers can access simultaneously
let contains = self.datacenter_ips.read().await.contains(&ip);

// Only one writer at a time
self.datacenter_ips.write().await.insert(new_ip);
```

### Mutex in FicHub

FicHub uses `Mutex` for the cache semaphores:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;
```

## Rc<T>

`Rc<T>` (Reference Counted) provides single-threaded shared ownership:

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let data_clone = data.clone();  // Increments reference count

// Both point to the same data, but not thread-safe
```

## Cell<T> and RefCell<T>

`Cell<T>` provides interior mutability for `Copy` types:

```rust
use std::cell::Cell;

let counter = Cell::new(0);
counter.set(counter.get() + 1);  // No borrow checker issues
```

`RefCell<T>` provides runtime borrow checking:

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);
{
    let mut borrowed = data.borrow_mut();
    borrowed.push(4);
}  // Mutable borrow released
{
    let borrowed = data.borrow();
    println!("{:?}", *borrowed);
}
```

## 📝 Practice Exercises

1. **Arc<Mutex<T>>:** Create a shared counter using `Arc<Mutex<T>>` and increment it from multiple threads.

2. **RwLock Performance:** Benchmark `RwLock` vs `Mutex` for read-heavy workloads.

3. **Interior Mutability:** Implement a cache using `RefCell<HashMap<K, V>>` that can be updated through shared references.

---

# Chapter 40: The Newtype Pattern

The newtype pattern wraps an existing type in a single-field tuple struct. It provides type safety and allows implementing traits on the wrapper.

## Why Newtype?

```rust
// Without newtype: easy to mix up
fn process_user_id(id: i64) { /* ... */ }
fn process_story_id(id: i64) { /* ... */ }

// Both take i64 — easy to mix up!
process_user_id(story_id);  // Bug! But compiles fine.

// With newtype: type-safe
struct UserId(i64);
struct StoryId(i64);

fn process_user_id(id: UserId) { /* ... */ }
fn process_story_id(id: StoryId) { /* ... */ }

process_user_id(StoryId(123));  // Compile error!
```

## Newtype in FicHub

FicHub could benefit from newtype for story IDs:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UrlId(String);

impl UrlId {
    pub fn new(source_id: i64, story_id: &str) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(source_id.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(story_id.as_bytes());
        let result = hasher.finalize();
        UrlId(hex::encode(&result[..6]))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UrlId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

## Newtype for Units of Measurement

```rust
#[derive(Debug, Clone, Copy)]
struct Meters(f64);

#[derive(Debug, Clone, Copy)]
struct Seconds(f64);

#[derive(Debug, Clone, Copy)]
struct MetersPerSecond(f64);

impl MetersPerSecond {
    fn from_distance_and_time(distance: Meters, time: Seconds) -> Self {
        MetersPerSecond(distance.0 / time.0)
    }
}

// Now you can't accidentally mix up units!
let distance = Meters(100.0);
let time = Seconds(10.0);
let speed = MetersPerSecond::from_distance_and_time(distance, time);
```

## 📝 Practice Exercises

1. **Newtype for IPs:** Create a newtype `ClientIp(std::net::IpAddr)` that implements `Display` and `FromStr`.

2. **Newtype for Rates:** Create newtypes `RequestsPerSecond(f64)` and `TokensPerSecond(f64)` for rate limiting.

3. **Derive Macros:** Use `#[derive(Debug, Clone, PartialEq)]` on your newtype and explain what each derive does.

---

# Chapter 41: Error Handling Patterns Beyond `?`

Rust's error handling is more nuanced than just `?`. This chapter covers advanced patterns.

## The thiserror Crate

`thiserror` simplifies creating custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("bad request ({0}): {1}")]
    BadRequest(i32, String),
    
    #[error("rate limited: retry after {0}s")]
    RateLimited(u64),
    
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("internal error")]
    Internal(#[from] std::io::Error),
    
    #[error("database error")]
    Database(#[from] sqlx::Error),
    
    #[error("scrape error: {0}")]
    ScrapeError(String),
    
    #[error("export error")]
    ExportError(#[from] ExportError),
}
```

The `#[from]` attribute automatically implements `From` for error conversion.

## The anyhow Crate

`anyhow` provides a flexible error type for applications:

```rust
use anyhow::{Context, Result, bail};

fn process_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config from {}", path))?;
    
    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config")?;
    
    if config.database_url.is_empty() {
        bail!("database_url is empty");
    }
    
    Ok(config)
}
```

`anyhow` is great for applications (where you just need to propagate errors) but not for libraries (where callers need to match on specific error variants).

## Error Context

Adding context to errors helps with debugging:

```rust
use anyhow::Context;

async fn fetch_and_parse(url: &str) -> Result<StoryMeta> {
    let response = client.get(url).send().await
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
    
    Ok(StoryMeta { title, /* ... */ })
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

## 📝 Practice Exercises

1. **thiserror:** Create a custom error type using `thiserror` that covers all FicHub's error cases.

2. **anyhow:** Refactor a function to use `anyhow::Result` and add context to each error.

3. **Error Matching:** Write code that catches specific error variants and handles them differently.

---

# Chapter 42: Serde: The Serialization Powerhouse

Serde is the de facto serialization framework for Rust. It handles converting between Rust types and formats like JSON, TOML, YAML, and more.

## Derive Macros

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Story {
    title: String,
    author: String,
    word_count: i64,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}
```

### Common Attributes

- `#[serde(default)]` — Use default value if field is missing
- `#[serde(skip_serializing_if = "Option::is_none")]` — Don't include `None` values
- `#[serde(rename = "wordCount")]` — Use different name in serialized format
- `#[serde(flatten)]` — Flatten nested structures
- `#[serde(tag = "type")]` — Tagged enum representation

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

## Serde with Axum

```rust
// Deserialize query parameters
#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
    #[serde(default = "default_page")]
    page: u32,
}

fn default_page() -> u32 { 1 }

// Serialize response
#[derive(Serialize)]
struct ApiResponse {
    err: i32,
    data: Vec<Story>,
    total: i64,
}

async fn handler(
    Query(params): Query<SearchParams>,
) -> Json<ApiResponse> {
    Json(ApiResponse {
        err: 0,
        data: vec![],
        total: 0,
    })
}
```

## 📝 Practice Exercises

1. **Custom Serialize:** Implement custom serialization for a type that stores timestamps as milliseconds.

2. **Flatten:** Use `#[serde(flatten)]` to combine two structs into one JSON object.

3. **Tagged Enums:** Serialize an enum with `#[serde(tag = "type")]` and deserialize it back.

---

# Chapter 43: Testing Patterns and Property-Based Testing

This chapter covers advanced testing patterns including mocking, fixtures, and property-based testing.

## Mocking with mockall

```rust
use mockall::automock;

#[automock]
trait Database {
    async fn get_user(&self, id: &str) -> Result<Option<User>, Error>;
    async fn create_user(&self, user: &User) -> Result<(), Error>;
}

#[tokio::test]
async fn test_get_user() {
    let mut mock_db = MockDatabase::new();
    mock_db
        .expect_get_user()
        .returning(|id| {
            Box::pin(async move {
                Ok(Some(User {
                    id: id.to_string(),
                    name: "Test User".to_string(),
                }))
            })
        });
    
    let result = mock_db.get_user("123").await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Test User");
}
```

## Test Fixtures with rstest

```rust
use rstest::rstest;

#[rstest]
#[case("https://archiveofourown.org/works/123456", true)]
#[case("https://fanfiction.net/s/123456/", true)]
#[case("https://google.com", false)]
fn test_can_handle(#[case] url: &str, #[case] expected: bool) {
    let scraper = Ao3Scraper;
    assert_eq!(scraper.can_handle(url), expected);
}
```

## Property-Based Testing with proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_escape_xml_preserves_text(s in "[a-zA-Z0-9 ]{1,100}") {
        let escaped = escape_xml(&s);
        // Escaped string should contain original characters
        // (minus any XML special chars)
        prop_assert!(escaped.len() >= s.len());
    }
    
    #[test]
    fn test_url_id_is_valid_hex(id in 0i64..100000, story in "[a-z0-9]{1,20}") {
        let url_id = generate_url_id(id, &story);
        prop_assert_eq!(url_id.len(), 12);
        prop_assert!(url_id.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_jaccard_symmetric(
        a in prop::collection::hash_set(0u32..100, 1..20),
        b in prop::collection::hash_set(0u32..100, 1..20),
    ) {
        let jaccard_ab = jaccard(&a, &b);
        let jaccard_ba = jaccard(&b, &a);
        prop_assert!((jaccard_ab - jaccard_ba).abs() < f64::EPSILON);
    }
}

fn jaccard(a: &HashSet<u32>, b: &HashSet<u32>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
```

## Benchmark Tests

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_generate_url_id(c: &mut Criterion) {
    c.bench_function("generate_url_id", |b| {
        b.iter(|| {
            generate_url_id(black_box(42), black_box("story_123"))
        })
    });
}

fn benchmark_escape_xml(c: &mut Criterion) {
    c.bench_function("escape_xml", |b| {
        b.iter(|| {
            escape_xml(black_box("<b>Hello & World</b>"))
        })
    });
}

criterion_group!(benches, benchmark_generate_url_id, benchmark_escape_xml);
criterion_main!(benches);
```

## 📝 Practice Exercises

1. **Mockall:** Create a mock HTTP client and test a scraper function without making real network requests.

2. **Property Tests:** Write property tests for the `parse_tag_filters` function.

3. **Benchmarks:** Benchmark the EPUB generation function with different chapter counts.

4. **Integration Test:** Write an integration test that starts the server, makes a request, and verifies the response.

