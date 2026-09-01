# Extended Content: Deep Dives and Additional Material

---

# Extended Chapter: Rust Ownership Deep Dive

## The Ownership System in Detail

Rust's ownership system is the language's most distinctive feature. It eliminates entire categories of bugs at compile time without the overhead of a garbage collector. Let's explore this system in depth with real-world examples from FicHub.

### Ownership Rules

Every value in Rust has exactly one owner. When the owner goes out of scope, the value is dropped (memory is freed). This is automatic — no manual memory management needed.

```rust
fn main() {
    let s1 = String::from("hello");  // s1 owns the String
    let s2 = s1;                      // ownership moves to s2
    // s1 is no longer valid
    println!("{}", s2);               // works fine
}
```

**Real-world analogy:** Ownership is like checking out a library book. Only one person can have the book checked out at a time. When you "give" the book to someone else (move), you no longer have it. When you're done with the book (scope ends), it goes back to the library (memory is freed).

### Move Semantics vs Copy Semantics

Types that implement the `Copy` trait are copied instead of moved:

```rust
// Copy types (stack-allocated, fixed size)
let x: i32 = 42;
let y = x;     // x is copied, both x and y are valid
println!("{}", x);  // works

// Non-Copy types (heap-allocated, variable size)
let s1 = String::from("hello");
let s2 = s1;     // s1 is moved
// println!("{}", s1);  // Error: s1 was moved
```

Common Copy types: `i32`, `f64`, `bool`, `char`, tuples of Copy types, arrays of Copy types.

Common non-Copy types: `String`, `Vec<T>`, `Box<T>`, `HashMap<K, V>`.

### Borrowing

Borrowing allows you to reference data without taking ownership:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope here, but since it doesn't own the value, nothing is dropped

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);  // borrow s1
    println!("{} has length {}", s1, len);  // s1 is still valid
}
```

**Real-world analogy:** Borrowing is like lending a book to a friend. They can read it (use the data), but they can't throw it away (drop the data) or give it to someone else (move the data). You can lend the same book to multiple people (multiple immutable borrows), but if someone wants to write notes in it (mutable borrow), you need to get all other copies back first.

### Mutable Borrowing

Mutable references allow modifying borrowed data:

```rust
fn add_world(s: &mut String) {
    s.push_str(", world!");
}

fn main() {
    let mut s = String::from("hello");
    add_world(&mut s);
    println!("{}", s);  // "hello, world!"
}
```

The rules for mutable references:
1. You can have either one mutable reference OR any number of immutable references
2. References must always be valid (no dangling references)

### Lifetimes

Lifetimes ensure that references don't outlive the data they point to:

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
        println!("Longest: {}", result);  // Works here
    }
    // Can't use result here because string2 was dropped
}
```

The `'a` lifetime parameter says "the returned reference lives as long as the shorter of the two input lifetimes."

**Real-world analogy:** Lifetimes are like expiration dates on food. A reference is only valid as long as the data it points to. The lifetime annotation tells the compiler the expiration date of the reference.

### Self and Lifetimes in Methods

```rust
struct Parser {
    input: String,
    position: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Parser {
            input: input.to_string(),
            position: 0,
        }
    }
    
    fn remaining(&self) -> &str {
        &self.input[self.position..]
    }
    
    fn advance(&mut self, n: usize) {
        self.position = (self.position + n).min(self.input.len());
    }
    
    fn parse_word(&mut self) -> &str {
        let start = self.position;
        while self.position < self.input.len() {
            let byte = self.input.as_bytes()[self.position];
            if byte.is_ascii_whitespace() {
                break;
            }
            self.position += 1;
        }
        &self.input[start..self.position]
    }
}
```

The `&self` parameter means "borrow self immutably." The `&mut self` parameter means "borrow self mutably." Rust automatically handles the lifetime parameters for methods.

## Interior Mutability Patterns

Interior mutability allows modifying data through shared references. This is essential for certain patterns in concurrent code.

### RefCell<T>

```rust
use std::cell::RefCell;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: RefCell::new(HashMap::new()),
        }
    }
    
    fn get(&self, key: &str) -> Option<String> {
        self.data.borrow().get(key).cloned()
    }
    
    fn insert(&self, key: String, value: String) {
        self.data.borrow_mut().insert(key, value);
    }
}
```

`RefCell` provides runtime borrow checking. If you try to borrow mutably while an immutable borrow exists, it panics at runtime (instead of compile time).

### Mutex<T> and RwLock<T>

```rust
use std::sync::{Arc, Mutex, RwLock};

// Mutex: one writer OR multiple readers (but not both)
let data = Arc::new(Mutex::new(Vec::new()));
let data_clone = data.clone();
tokio::spawn(async move {
    let mut guard = data_clone.lock().unwrap();
    guard.push(42);
});

// RwLock: one writer OR multiple readers
let data = Arc::new(RwLock::new(Vec::new()));
let data_clone = data.clone();
tokio::spawn(async move {
    let mut guard = data_clone.write().unwrap();
    guard.push(42);
});
let guard = data.read().unwrap();
println!("{:?}", *guard);
```

### Cell<T>

`Cell` provides interior mutability for `Copy` types:

```rust
use std::cell::Cell;

struct Counter {
    value: Cell<u32>,
}

impl Counter {
    fn increment(&self) {
        self.value.set(self.value.get() + 1);
    }
    
    fn get(&self) -> u32 {
        self.value.get()
    }
}
```

## Smart Pointers in Depth

### Box<T>

```rust
// Recursive type (requires heap allocation)
enum List {
    Cons(i32, Box<List>),
    Nil,
}

let list = Cons(1, Box::new(Cons(2, Box::new(Nil))));

// Trait objects
let scrapers: Vec<Box<dyn SiteScraper>> = vec![
    Box::new(Ao3Scraper),
    Box::new(FfNetScraper),
];
```

### Rc<T> and Arc<T>

```rust
use std::rc::Rc;
use std::sync::Arc;

// Rc: single-threaded reference counting
let data = Rc::new(vec![1, 2, 3]);
let data2 = data.clone();  // Increments reference count
println!("References: {}", Rc::strong_count(&data));  // 2

// Arc: thread-safe reference counting
let data = Arc::new(vec![1, 2, 3]);
let data2 = data.clone();
tokio::spawn(async move {
    println!("{:?}", *data2);
});
```

## Error Handling Patterns

### The ? Operator

```rust
use std::fs;
use std::io;

fn read_config() -> Result<String, io::Error> {
    let content = fs::read_to_string("config.toml")?;  // Propagates error
    Ok(content)
}

// Equivalent to:
fn read_config_verbose() -> Result<String, io::Error> {
    match fs::read_to_string("config.toml") {
        Ok(content) => Ok(content),
        Err(e) => Err(e),
    }
}
```

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("bad request ({0}): {1}")]
    BadRequest(i32, String),
    
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("internal error")]
    Internal(#[from] std::io::Error),
    
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg})),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg})),
            AppError::Internal(e) => {
                tracing::error!(error = %e, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "internal error"}))
            }
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "database error"}))
            }
        };
        (status, Json(body)).into_response()
    }
}
```

### Error Context with anyhow

```rust
use anyhow::{Context, Result};

async fn process_story(url: &str) -> Result<Story> {
    let response = client.get(url).send().await
        .context(format!("Failed to fetch {}", url))?;
    
    let html = response.text().await
        .context("Failed to read response body")?;
    
    let story = parse_story(&html)
        .context("Failed to parse story")?;
    
    Ok(story)
}
```

## Trait Objects and Dynamic Dispatch

### Object Safety

A trait is object-safe if it can be used as a trait object. Requirements:
- All methods must have `Self: Sized` or not use `Self` in the return type
- No generic type parameters
- No associated types

```rust
// Object-safe
trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError>;
}

// Not object-safe (has generic parameter)
trait Repository<T> {
    fn get(&self, id: &str) -> Result<T>;
}
```

### VTables

When you use a trait object, Rust creates a vtable (virtual method table) that maps method calls to the concrete implementation:

```rust
// Trait object layout
struct DynSiteScraper {
    data: *const (),           // Pointer to the actual data
    vtable: *const VTable,     // Pointer to the vtable
}

struct VTable {
    drop: fn(*const ()),       // Destructor
    size: usize,               // Size of the concrete type
    align: usize,              // Alignment of the concrete type
    can_handle: fn(*const (), &str) -> bool,  // Method pointer
    lookup: fn(*const (), &Client, &str) -> Pin<Box<dyn Future<Output = Result<FicMetadata>>>>,
}
```

## Type-State Pattern

The type-state pattern uses the type system to enforce state transitions at compile time:

```rust
// States
struct Unvalidated;
struct Validated;
struct Scraped;

// Request with phantom type
struct Request<State> {
    url: String,
    _state: std::marker::PhantomData<State>,
}

impl Request<Unvalidated> {
    fn new(url: String) -> Self {
        Request { url, _state: std::marker::PhantomData }
    }
    
    fn validate(self) -> Result<Request<Validated>, AppError> {
        validate_url(&self.url)?;
        Ok(Request { url: self.url, _state: std::marker::PhantomData })
    }
}

impl Request<Validated> {
    async fn scrape(self, client: &Client) -> Result<Request<Scraped>, AppError> {
        // ... scraping logic ...
        Ok(Request { url: self.url, _state: std::marker::PhantomData })
    }
}

// Usage:
let result = Request::new(url)
    .validate()?     // Returns Request<Validated>
    .scrape(&client).await?;  // Returns Request<Scraped>

// This won't compile:
let result = Request::new(url)
    .scrape(&client).await?;  // Error: Request<Unvalidated> has no scrape method
```

## Iterator Patterns

### Custom Iterators

```rust
struct ChapterRange {
    start: i32,
    end: i32,
}

impl Iterator for ChapterRange {
    type Item = Chapter;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let chapter = Chapter {
                chapter_id: self.start,
                title: format!("Chapter {}", self.start),
                content: String::new(),
            };
            self.start += 1;
            Some(chapter)
        } else {
            None
        }
    }
}

// Usage:
let chapters: Vec<Chapter> = ChapterRange { start: 1, end: 10 }
    .filter(|c| c.chapter_id % 2 == 0)
    .collect();
```

### Iterator Adaptors

```rust
// Map: transform each element
let titles: Vec<String> = chapters.iter()
    .map(|c| c.title.clone())
    .collect();

// Filter: keep elements matching predicate
let long_chapters: Vec<&Chapter> = chapters.iter()
    .filter(|c| c.content.len() > 1000)
    .collect();

// Fold: combine all elements
let total_words: usize = chapters.iter()
    .fold(0, |acc, c| acc + c.content.split_whitespace().count());

// Chain: combine iterators
let all: Vec<String> = titles1.into_iter()
    .chain(titles2.into_iter())
    .collect();

// Enumerate: add index
let indexed: Vec<(usize, &Chapter)> = chapters.iter()
    .enumerate()
    .collect();

// Zip: combine two iterators
let pairs: Vec<(&str, &str)> = titles.iter()
    .zip(authors.iter())
    .collect();
```

## Concurrency Patterns

### Channel-Based Communication

```rust
use tokio::sync::mpsc;

async fn producer_consumer() {
    let (tx, mut rx) = mpsc::channel::<Story>(100);
    
    // Producer
    tokio::spawn(async move {
        for url in urls {
            let story = fetch_story(&url).await;
            tx.send(story).await.unwrap();
        }
    });
    
    // Consumer
    while let Some(story) = rx.recv().await {
        process_story(story).await;
    }
}
```

### Shared State with Arc<Mutex<T>>

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn shared_counter() {
    let counter = Arc::new(Mutex::new(0));
    
    let mut handles = vec![];
    for _ in 0..10 {
        let counter = counter.clone();
        handles.push(tokio::spawn(async move {
            let mut count = counter.lock().await;
            *count += 1;
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    println!("Final count: {}", *counter.lock().await);  // 10
}
```

### JoinSet for Structured Concurrency

```rust
use tokio::task::JoinSet;

async fn process_all(urls: Vec<String>) -> Vec<Story> {
    let mut set = JoinSet::new();
    
    for url in urls {
        set.spawn(async move {
            fetch_story(&url).await
        });
    }
    
    let mut stories = Vec::new();
    while let Some(result) = set.join_next().await {
        if let Ok(story) = result {
            stories.push(story);
        }
    }
    
    stories
}
```

## Async Patterns in Depth

### Select

```rust
use tokio::select;

async fn fetch_with_timeout(url: &str) -> Result<String, AppError> {
    select! {
        result = async {
            reqwest::get(url).await?.text().await
        } => {
            result.map_err(|e| AppError::Network(e.to_string()))
        }
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            Err(AppError::Network("timeout".into()))
        }
    }
}
```

### Join

```rust
use tokio::join;

async fn fetch_parallel(urls: [&str; 3]) -> [String; 3] {
    let (r1, r2, r3) = join!(
        fetch_url(urls[0]),
        fetch_url(urls[1]),
        fetch_url(urls[2]),
    );
    [r1.unwrap(), r2.unwrap(), r3.unwrap()]
}
```

### Stream Processing

```rust
use tokio_stream::StreamExt;

async fn process_stream(urls: Vec<String>) {
    let mut stream = tokio_stream::iter(urls)
        .map(|url| async move {
            fetch_story(&url).await
        })
        .buffer_unordered(10);  // Process 10 at a time
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(story) => process(story),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
```

## Database Patterns

### Transaction Management

```rust
async fn transfer_tag(
    pool: &PgPool,
    from_url_id: &str,
    to_url_id: &str,
    tag_id: i32,
) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    
    // Remove from source
    sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(from_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    // Add to destination
    sqlx::query("INSERT INTO fic_tags (url_id, tag_id, score) VALUES ($1, $2, 0)")
        .bind(to_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    // Commit
    tx.commit().await?;
    Ok(())
}
```

### Bulk Operations

```rust
async fn bulk_insert(
    pool: &PgPool,
    items: &[FicInfo],
) -> Result<(), AppError> {
    let mut query = String::from(
        "INSERT INTO fic_info (id, title, author, words) VALUES "
    );
    
    for (i, _) in items.iter().enumerate() {
        if i > 0 {
            query.push(',');
        }
        query.push_str(&format!("(${}, ${}, ${}, ${})", i * 4 + 1, i * 4 + 2, i * 4 + 3, i * 4 + 4));
    }
    
    query.push_str(" ON CONFLICT DO NOTHING");
    
    let mut q = sqlx::query(&query);
    for item in items {
        q = q.bind(&item.id).bind(&item.title).bind(&item.author).bind(item.words);
    }
    
    q.execute(pool).await?;
    Ok(())
}
```

### Query Builder Pattern

```rust
use sqlx::QueryBuilder;

fn build_search_query(params: &SearchParams) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new(
        "SELECT fi.id, fi.title, fi.author, fi.words FROM fic_info fi WHERE 1=1"
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
    
    if let Some(max) = params.max_words {
        qb.push(" AND fi.words <= ");
        qb.push_bind(max);
    }
    
    if let Some(ref source) = params.source {
        qb.push(" AND fi.source = ");
        qb.push_bind(source.clone());
    }
    
    // Add sorting
    match params.sort.as_deref() {
        Some("-words") => qb.push(" ORDER BY fi.words DESC"),
        Some("-date") => qb.push(" ORDER BY fi.fic_updated DESC"),
        _ => qb.push(" ORDER BY fi.fic_updated DESC"),
    }
    
    // Add pagination
    let limit = params.per_page.unwrap_or(20) as i64;
    let offset = ((params.page.unwrap_or(1) - 1) as i64) * limit;
    qb.push(" LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    
    qb
}
```

## Web Scraping Patterns

### HTML Parsing with CSS Selectors

```rust
use scraper::{Html, Selector};

fn parse_story(html: &str) -> Result<StoryMeta, ScrapeError> {
    let document = Html::parse_document(html);
    
    // Title
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    // Author
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    // Word count
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    Ok(StoryMeta { title, author, words })
}
```

### Error Handling in Scrapers

```rust
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::NotFound => write!(f, "story not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

### Retry Logic

```rust
async fn fetch_with_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<String, ScrapeError> {
    let mut delay = Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                return response.text().await
                    .map_err(|e| ScrapeError::Network(e.to_string()));
            }
            Ok(response) if response.status().as_u16() == 429 => {
                // Rate limited
                tracing::warn!(url = %url, attempt = attempt, "Rate limited, waiting");
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Ok(response) => {
                return Err(ScrapeError::Network(
                    format!("HTTP {}", response.status())
                ));
            }
            Err(e) if e.is_timeout() || e.is_connect() => {
                // Transient error
                tracing::warn!(url = %url, attempt = attempt, error = %e, "Transient error, retrying");
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => {
                return Err(ScrapeError::Network(e.to_string()));
            }
        }
    }
    
    Err(ScrapeError::Network("max retries exceeded".into()))
}
```

## Configuration Patterns

### Environment Variable Loading

```rust
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .expect("REDIS_URL must be set"),
            cache_dir: PathBuf::from(
                env::var("CACHE_DIR").unwrap_or_else(|_| "./cache".into())
            ),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a valid number"),
        }
    }
}
```

### Builder Pattern

```rust
pub struct ServerBuilder {
    port: u16,
    host: String,
    max_connections: u32,
}

impl ServerBuilder {
    pub fn new() -> Self {
        ServerBuilder {
            port: 3000,
            host: "0.0.0.0".into(),
            max_connections: 20,
        }
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }
    
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
    
    pub fn build(self) -> Server {
        Server {
            port: self.port,
            host: self.host,
            max_connections: self.max_connections,
        }
    }
}
```

## Testing Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_url_id() {
        let id = generate_url_id(1, "story_123");
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_generate_url_id_deterministic() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(1, "story_123");
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_generate_url_id_different() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(2, "story_123");
        assert_ne!(id1, id2);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_export_endpoint() {
    let state = setup_test_state().await;
    let app = build_router(Arc::new(state));
    
    let response = axum_test::TestClient::new(app)
        .get("/api/v0/epub")
        .query(&[("q", "https://archiveofourown.org/works/123456")])
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["err"], 0);
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_url_id_always_hex(s in "[a-z0-9]+", id in 0i64..10000) {
        let url_id = generate_url_id(id, &s);
        prop_assert_eq!(url_id.len(), 12);
        prop_assert!(url_id.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[test]
    fn test_url_id_deterministic(s in "[a-z0-9]+", id in 0i64..10000) {
        let id1 = generate_url_id(id, &s);
        let id2 = generate_url_id(id, &s);
        prop_assert_eq!(id1, id2);
    }
}
```

## Performance Optimization

### Avoiding Allocations

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
    s.push_str("item_");
    s.push_str(&i.to_string());
    process(&s);
}

// BETTER: Use write! macro
let mut s = String::new();
for i in 0..10000 {
    s.clear();
    write!(&mut s, "item_{}", i).unwrap();
    process(&s);
}
```

### Using References Instead of Cloning

```rust
// BAD: Cloning large data
let data = vec![0u8; 1024 * 1024];
let data_clone = data.clone();  // 1MB copy

// GOOD: Use references
let data = vec![0u8; 1024 * 1024];
process(&data);  // Just a reference

// GOOD: Use Arc for shared data
let data = Arc::new(vec![0u8; 1024 * 1024]);
let data2 = data.clone();  // Arc clone is cheap
```

### Parallel Processing

```rust
use rayon::prelude::*;

// Sequential
let results: Vec<_> = items.iter()
    .map(|item| process(item))
    .collect();

// Parallel
let results: Vec<_> = items.par_iter()
    .map(|item| process(item))
    .collect();
```

## Security Patterns

### Input Validation

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
    
    let host = parsed.host_str()
        .ok_or_else(|| AppError::BadRequest(-1, "no host".into()))?;
    
    let allowed = ["archiveofourown.org", "fanfiction.net"];
    if !allowed.iter().any(|h| host.ends_with(h)) {
        return Err(AppError::BadRequest(-1, "unsupported domain".into()));
    }
    
    Ok(())
}
```

### SQL Injection Prevention

```rust
// DANGEROUS
let query = format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", search);

// SAFE
let query = "SELECT * FROM fic_info WHERE title LIKE $1";
let pattern = format!("%{}%", search);
sqlx::query(query)
    .bind(&pattern)
    .fetch_all(&pool)
    .await?;
```

### Rate Limiting

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

## Deployment Patterns

### Docker Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.77-bookworm as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
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

### Systemd Service

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

## Monitoring Patterns

### Structured Logging

```rust
use tracing::{info, warn, error, debug};

async fn process_request(url: &str) -> Result<(), AppError> {
    info!(url = %url, "Processing request");
    
    let meta = fetch_metadata(url).await?;
    debug!(url_id = %meta.url_id, title = %meta.title, "Metadata fetched");
    
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

### Prometheus Metrics

```rust
use prometheus::{IntCounter, Histogram, register_int_counter_with_registry, register_histogram_with_registry};

lazy_static! {
    static ref REQUESTS: IntCounter = register_int_counter_with_registry!(
        "fichub_requests_total", "Total requests"
    ).unwrap();
    
    static ref REQUEST_DURATION: Histogram = register_histogram_with_registry!(
        "fichub_request_duration_seconds", "Request duration"
    ).unwrap();
}
```

### Health Checks

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

## Troubleshooting Patterns

### Common Build Errors

```rust
// Error: missing lifetime specifier
// Fix: Add explicit lifetime
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }

// Error: use of moved value
// Fix: Clone or use reference
let s = String::from("hello");
let s2 = s.clone();
println!("{}", s);

// Error: the size for values of type cannot be known
// Fix: Use Box or reference
fn process(item: &dyn SiteScraper) { /* ... */ }
```

### Runtime Debugging

```rust
// Add debug logging
debug!(input = %input, "Processing");
let result = process(input);
debug!(result = ?result, "Result");

// Add request IDs
let request_id = Uuid::new_v4();
info!(request_id = %request_id, "Processing request");
```

### Memory Leak Detection

```rust
// Use Arc::strong_count to check reference counts
let data = Arc::new(vec![1, 2, 3]);
println!("References: {}", Arc::strong_count(&data));

// Use Weak to break circular references
use std::rc::Weak;
struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}
```

## Advanced Async Patterns

### Select with Multiple Branches

```rust
use tokio::select;

async fn fetch_with_fallback(url: &str, fallback: &str) -> Result<String, AppError> {
    select! {
        result = async { reqwest::get(url).await?.text().await } => {
            result.map_err(|e| AppError::Network(e.to_string()))
        }
        result = async { reqwest::get(fallback).await?.text().await } => {
            result.map_err(|e| AppError::Network(e.to_string()))
        }
    }
}
```

### Broadcast Channels

```rust
use tokio::sync::broadcast;

async fn event_broadcast() {
    let (tx, _) = broadcast::channel::<Event>(100);
    
    // Publisher
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            let event = Event::new();
            tx_clone.send(event).ok();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    
    // Subscriber 1
    let mut rx1 = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx1.recv().await {
            println!("Subscriber 1: {:?}", event);
        }
    });
    
    // Subscriber 2
    let mut rx2 = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = rx2.recv().await {
            println!("Subscriber 2: {:?}", event);
        }
    });
}
```

### Oneshot Channels

```rust
use tokio::sync::oneshot;

async fn request_response() {
    let (tx, rx) = oneshot::channel();
    
    tokio::spawn(async move {
        let result = compute_something().await;
        tx.send(result).ok();
    });
    
    let result = rx.await.unwrap();
    println!("Result: {:?}", result);
}
```

### Rate Limiting with Token Bucket

```rust
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        TokenBucket {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
    
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

async fn rate_limited_request(
    bucket: Arc<Mutex<TokenBucket>>,
    url: &str,
) -> Result<String, AppError> {
    loop {
        {
            let mut bucket = bucket.lock().await;
            if bucket.try_consume(1.0) {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    reqwest::get(url).await?.text().await
        .map_err(|e| AppError::Network(e.to_string()))
}
```

## Final Summary

This extended content covers the most important patterns and techniques used in Rust backend development. The key takeaways are:

1. **Ownership and borrowing** are fundamental to Rust's safety guarantees
2. **Traits and generics** provide polymorphism without runtime overhead
3. **Async/await** enables efficient concurrency with Tokio
4. **Pattern matching** makes code expressive and safe
5. **Error handling** with Result and ? is elegant and composable
6. **Testing** is essential for production software
7. **Monitoring** helps you understand and optimize your application
8. **Security** must be considered from the beginning

Remember: Rust's strictness is a feature, not a bug. The compiler catches entire categories of bugs at compile time, saving you from debugging them at 3 AM in production.

