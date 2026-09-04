# Extended Content: Complete Practice Exercises and Solutions

---

# Comprehensive Practice Exercises for Each Chapter

## Chapter 1 Exercises: What We're Building

### Exercise 1.1: System Architecture
Draw the complete architecture of FicHub including all modules, their dependencies, and the data flow between them. Include the database, Redis, and external services.

**Solution:**
```
┌─────────────────────────────────────────────────────────────┐
│                     Client (Browser/App)                    │
└──────────────────────────┬──────────────────────────────────┘
                           │ HTTP Request
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Router                              │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Rate    │  │ Scraper  │  │ Cache    │  │ Database │   │
│  │ Limiter │  │ Registry │  │ Check    │  │ Query    │   │
│  └────┬────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │            │             │              │           │
│       ▼            ▼             ▼              ▼           │
│  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ Redis   │  │ AO3/FFN  │  │ Disk     │  │PostgreSQL│   │
│  │         │  │ Scrapers │  │ Cache    │  │          │   │
│  └─────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Exercise 1.2: Feature Identification
List all features of FicHub and categorize them as:
- Core (must have)
- Important (should have)
- Nice to have

**Solution:**
Core: Scraping, EPUB generation, caching, rate limiting
Important: Search, tags, recommendations, OPDS
Nice to have: Multi-language, AI recommendations, mobile app

### Exercise 1.3: Technology Comparison
Compare Rust with Python and Node.js for building FicHub. What are the trade-offs?

**Solution:**
Rust: Fastest, safest, steepest learning curve
Python: Easiest to learn, slowest runtime, dynamic typing
Node.js: Good async support, npm ecosystem, JavaScript limitations

## Chapter 2 Exercises: How the Web Works

### Exercise 2.1: HTTP Request/Response
Write out a complete HTTP request and response for a FicHub export request.

**Solution:**
Request:
```
GET /api/v0/epub?q=https://archiveofourown.org/works/123456 HTTP/1.1
Host: fichub.net
User-Agent: Mozilla/5.0
Accept: application/json
```

Response:
```
HTTP/1.1 200 OK
Content-Type: application/json
X-Request-Id: abc-123

{"err": 0, "url_id": "abc123", "meta": {...}, "urls": {...}}
```

### Exercise 2.2: JSON Design
Design a JSON response format for a search endpoint that includes pagination, sorting, and filtering.

**Solution:**
```json
{
    "err": 0,
    "total": 1234,
    "page": 1,
    "per_page": 20,
    "sort": "-relevance",
    "filters": {
        "q": "harry potter",
        "tags": ["Harry Potter"],
        "min_words": 10000
    },
    "results": [...]
}
```

### Exercise 2.3: Error Handling
Design an error response format that includes error codes, messages, and retry information.

**Solution:**
```json
{
    "err": -429,
    "msg": "rate limited",
    "retry_after": 30,
    "documentation": "https://fichub.net/docs/rate-limiting"
}
```

## Chapter 3 Exercises: Setting Up Your Workshop

### Exercise 3.1: Environment Setup
Set up a complete development environment including Rust, PostgreSQL, Redis, and verify everything works.

**Solution:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL (Arch)
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl enable --now postgresql

# Install Redis (Arch)
sudo pacman -S redis
sudo systemctl enable --now redis

# Verify
rustc --version
psql --version
redis-cli ping
```

### Exercise 3.2: Project Setup
Clone FicHub, set up the environment, and run the first build.

**Solution:**
```bash
git clone https://github.com/your-username/fichub.git
cd fichub
cp .env.example .env
# Edit .env with your database credentials
cargo build
```

### Exercise 3.3: Database Setup
Create the database, run migrations, and verify the schema.

**Solution:**
```bash
sudo -u postgres psql
CREATE USER fichub WITH PASSWORD 'fichub';
CREATE DATABASE fichub OWNER fichub;
\q

sqlx migrate run --source migrations
psql -U fichub -d fichub -c "\dt"  # List tables
```

## Chapter 4 Exercises: Your First Rust Program

### Exercise 4.1: Ownership
Write code that demonstrates move semantics, borrowing, and mutable borrowing.

**Solution:**
```rust
fn main() {
    // Move semantics
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is moved
    println!("{}", s2);
    
    // Borrowing
    let s3 = String::from("world");
    let len = calculate_length(&s3);
    println!("{} has length {}", s3, len);
    
    // Mutable borrowing
    let mut s4 = String::from("hello");
    change(&mut s4);
    println!("{}", s4);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

### Exercise 4.2: Error Handling
Write a function that returns Result and handles different error cases.

**Solution:**
```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".into())
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    match divide(10.0, 0.0) {
        Ok(result) => println!("10 / 0 = {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Exercise 4.3: Traits
Define a trait and implement it for multiple types.

**Solution:**
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

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

fn main() {
    let article = Article { title: "My Article".into(), author: "John".into() };
    let tweet = Tweet { username: "alice".into(), content: "Hello!".into() };
    
    println!("Article: {}", article.summarize());
    println!("Tweet: {}", tweet.summarize());
}
```

## Chapter 5 Exercises: Your First Web Server

### Exercise 5.1: Multi-Route Server
Create a server with GET, POST, and DELETE routes.

**Solution:**
```rust
use axum::{routing::{get, post, delete}, Router, Json, extract::Path};
use serde_json::{json, Value};

async fn list_items() -> Json<Value> {
    Json(json!({"items": []}))
}

async fn create_item(Json(payload): Json<Value>) -> Json<Value> {
    Json(json!({"created": payload}))
}

async fn delete_item(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"deleted": id}))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/items", get(list_items).post(create_item))
        .route("/items/{id}", delete(delete_item));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Exercise 5.2: Query Parameters
Create an endpoint that accepts multiple query parameters and validates them.

**Solution:**
```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default)]
    offset: u32,
}

fn default_limit() -> u32 { 20 }

async fn search(Query(params): Query<SearchParams>) -> Json<Value> {
    Json(json!({
        "query": params.q,
        "limit": params.limit,
        "offset": params.offset
    }))
}
```

### Exercise 5.3: Error Handling
Create a handler that returns different error responses based on input.

**Solution:**
```rust
async fn handler(Query(params): Query<SearchParams>) -> Result<Json<Value>, StatusCode> {
    if params.q.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    if params.limit > 100 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    Ok(Json(json!({"results": []})))
}
```

## Chapter 6 Exercises: Configuration and Error Handling

### Exercise 6.1: Custom Config
Create a configuration struct that loads from environment variables with validation.

**Solution:**
```rust
struct Config {
    database_url: String,
    port: u16,
    max_connections: u32,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL must be set")?;
        
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse()
            .map_err(|_| "PORT must be a valid number")?;
        
        let max_connections = std::env::var("MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".into())
            .parse()
            .map_err(|_| "MAX_CONNECTIONS must be a valid number")?;
        
        Ok(Config { database_url, port, max_connections })
    }
}
```

### Exercise 6.2: Error Types
Create a custom error type with multiple variants and implement Display and From.

**Solution:**
```rust
#[derive(Debug)]
enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

### Exercise 6.3: Structured Logging
Add structured logging to a handler function with different log levels.

**Solution:**
```rust
use tracing::{info, warn, error, debug};

async fn handler(url: String) -> Result<Json<Value>, AppError> {
    info!(url = %url, "Processing request");
    
    if url.is_empty() {
        warn!("Empty URL provided");
        return Err(AppError::BadRequest("URL cannot be empty".into()));
    }
    
    debug!("Fetching metadata");
    let meta = fetch_metadata(&url).await?;
    
    info!(url_id = %meta.url_id, "Request complete");
    Ok(Json(json!({"meta": meta})))
}
```

## Chapter 7 Exercises: Connecting to PostgreSQL

### Exercise 7.1: Connection Pool
Set up a connection pool with proper configuration.

**Solution:**
```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

async fn setup_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

### Exercise 7.2: CRUD Operations
Implement create, read, update, and delete for a stories table.

**Solution:**
```rust
async fn create_story(pool: &PgPool, title: &str, author: &str) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "INSERT INTO stories (title, author) VALUES ($1, $2) RETURNING id"
    )
    .bind(title)
    .bind(author)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

async fn get_story(pool: &PgPool, id: i64) -> Result<Option<Story>, sqlx::Error> {
    sqlx::query_as::<_, Story>("SELECT * FROM stories WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

async fn update_story(pool: &PgPool, id: i64, title: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE stories SET title = $1 WHERE id = $2")
        .bind(title)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn delete_story(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM stories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

### Exercise 7.3: Transactions
Implement a function that moves a tag from one story to another using a transaction.

**Solution:**
```rust
async fn transfer_tag(
    pool: &PgPool,
    from_id: i64,
    to_id: i64,
    tag_id: i64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    sqlx::query("DELETE FROM story_tags WHERE story_id = $1 AND tag_id = $2")
        .bind(from_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    sqlx::query("INSERT INTO story_tags (story_id, tag_id) VALUES ($1, $2)")
        .bind(to_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

## Chapter 8 Exercises: Database Migrations and Models

### Exercise 8.1: Create Migration
Write a migration that adds a new table with indexes.

**Solution:**
```sql
-- 005_add_bookmarks.sql
CREATE TABLE bookmarks (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(128) NOT NULL,
    story_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, story_id)
);

CREATE INDEX idx_bookmarks_user ON bookmarks(user_id);
CREATE INDEX idx_bookmarks_story ON bookmarks(story_id);
```

### Exercise 8.2: Model Derivation
Create a model struct with derive macros for a database table.

**Solution:**
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bookmark {
    pub id: i64,
    pub user_id: String,
    pub story_id: String,
    pub created: Option<DateTime<Utc>>,
}
```

### Exercise 8.3: Schema Documentation
Document each table in the FicHub schema with columns, types, and constraints.

**Solution:**
See the complete schema reference in Part 23.

## Chapter 9 Exercises: CRUD Operations

### Exercise 9.1: Batch Insert
Implement a function that inserts multiple records in a single query.

**Solution:**
```rust
async fn batch_insert(pool: &PgPool, items: &[(String, String)]) -> Result<(), sqlx::Error> {
    let mut query = String::from("INSERT INTO stories (title, author) VALUES ");
    
    for (i, _) in items.iter().enumerate() {
        if i > 0 {
            query.push(',');
        }
        query.push_str(&format!("(${}, ${})", i * 2 + 1, i * 2 + 2));
    }
    
    query.push_str(" ON CONFLICT DO NOTHING");
    
    let mut q = sqlx::query(&query);
    for (title, author) in items {
        q = q.bind(title).bind(author);
    }
    
    q.execute(pool).await?;
    Ok(())
}
```

### Exercise 9.2: Upsert
Implement an upsert function that inserts or updates a record.

**Solution:**
```rust
async fn upsert_story(
    pool: &PgPool,
    id: &str,
    title: &str,
    author: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO stories (id, title, author)
           VALUES ($1, $2, $3)
           ON CONFLICT (id) DO UPDATE SET
               title = EXCLUDED.title,
               author = EXCLUDED.author,
               updated = NOW()"#
    )
    .bind(id)
    .bind(title)
    .bind(author)
    .execute(pool)
    .await?;
    Ok(())
}
```

### Exercise 9.3: Soft Delete
Implement soft delete by adding a deleted_at timestamp.

**Solution:**
```rust
async fn soft_delete(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE stories SET deleted_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

async fn get_active_stories(pool: &PgPool) -> Result<Vec<Story>, sqlx::Error> {
    sqlx::query_as::<_, Story>("SELECT * FROM stories WHERE deleted_at IS NULL")
        .fetch_all(pool)
        .await
}
```

## Chapter 10 Exercises: The Axum Router and Middleware

### Exercise 10.1: Route Composition
Create a router with nested routes and middleware.

**Solution:**
```rust
fn api_routes() -> Router {
    Router::new()
        .route("/stories", get(list_stories).post(create_story))
        .route("/stories/{id}", get(get_story).delete(delete_story))
}

fn admin_routes() -> Router {
    Router::new()
        .route("/users", get(list_users).delete(delete_user))
}

let app = Router::new()
    .nest("/api/v1", api_routes())
    .nest("/admin", admin_routes())
    .layer(CorsLayer::permissive());
```

### Exercise 10.2: Custom Middleware
Write middleware that adds a request ID to every response.

**Solution:**
```rust
use axum::middleware::Next;
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

### Exercise 10.3: State Extraction
Create a handler that extracts shared state and uses it.

**Solution:**
```rust
struct AppState {
    db: PgPool,
    redis: MultiplexedConnection,
}

async fn handler(
    State(state): State<Arc<AppState>>,
) -> Json<Value> {
    // Use state.db to query database
    // Use state.redis to access Redis
    Json(json!({"status": "ok"}))
}

let state = Arc::new(AppState { db, redis });
let app = Router::new()
    .route("/", get(handler))
    .with_state(state);
```

## Chapter 11 Exercises: Understanding Fanfiction Sites

### Exercise 11.1: Site Analysis
Visit AO3 and FF.net, inspect the HTML structure, and document the CSS selectors for key elements.

**Solution:**
AO3:
- Title: `h2.title.heading`
- Author: `a[rel='author']`
- Content: `div.userstuff`
- Words: `dd.words`

FF.net:
- Title: `#profile_top b.xcontrast_txt`
- Author: `#profile_top a.xcontrast_txt`
- Content: `div.storytext`
- Words: `span[data-xutitle='word count']`

### Exercise 11.2: URL Pattern Matching
Write a function that identifies which site a URL belongs to.

**Solution:**
```rust
fn identify_site(url: &str) -> Option<&str> {
    if url.contains("archiveofourown.org") {
        Some("ao3")
    } else if url.contains("fanfiction.net") {
        Some("ffnet")
    } else if url.contains("spacebattles.com") || url.contains("sufficientvelocity.com") {
        Some("xenforo")
    } else {
        None
    }
}
```

### Exercise 11.3: Ethical Considerations
Research and document FicHub's ethical scraping practices.

**Solution:**
1. Respect rate limits
2. Use descriptive User-Agent
3. Cache results
4. Handle errors gracefully
5. Don't scrape private content
6. Give credit to authors and sites

## Chapter 12 Exercises: Building Web Scrapers

### Exercise 12.1: HTML Parsing
Parse an HTML string and extract data using CSS selectors.

**Solution:**
```rust
use scraper::{Html, Selector;

fn parse_title(html: &str) -> String {
    let document = Html::parse_document(html);
    let selector = Selector::parse("h2.title").unwrap();
    
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
```

### Exercise 12.2: Error Handling
Implement error handling for network requests and HTML parsing.

**Solution:**
```rust
async fn fetch_and_parse(client: &Client, url: &str) -> Result<String, ScrapeError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))
}
```

### Exercise 12.3: Retry Logic
Implement retry logic with exponential backoff.

**Solution:**
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
            Ok(_) => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => {
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
        }
    }
    
    Err(ScrapeError::Network("max retries exceeded".into()))
}
```

## Chapter 13 Exercises: The Scraper Registry

### Exercise 13.1: New Scraper
Create a scraper for a hypothetical fanfiction site.

**Solution:**
```rust
pub struct StoryArchiveScraper;

impl SiteScraper for StoryArchiveScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("storyarchive.com")
    }
    
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Implementation
        todo!()
    }
    
    async fn fetch_chapters(&self, client: &Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Implementation
        todo!()
    }
}
```

### Exercise 13.2: Registry Extension
Add the new scraper to the registry.

**Solution:**
```rust
impl ScraperRegistry {
    pub fn new() -> Self {
        let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
        scrapers.push(Box::new(sites::ao3::Ao3Scraper));
        scrapers.push(Box::new(sites::ffnet::FfNetScraper));
        scrapers.push(Box::new(sites::storyarchive::StoryArchiveScraper));
        ScraperRegistry { scrapers }
    }
}
```

## Chapter 14 Exercises: AO3 Scraper Deep Dive

### Exercise 14.1: Metadata Extraction
Extract all metadata from an AO3 work page.

**Solution:**
```rust
fn extract_metadata(document: &Html) -> (String, String, i64, i32) {
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default();
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| el.text().collect::<String>().replace(',', "").parse().ok())
        .unwrap_or(0);
    
    let chapters = document
        .select(&Selector::parse("dd.chapters").unwrap())
        .next()
        .map(|el| {
            let text = el.text().collect::<String>();
            if let Some(pos) = text.find('/') {
                text[..pos].trim().parse().unwrap_or(1)
            } else {
                1
            }
        })
        .unwrap_or(1);
    
    (title, author, words, chapters)
}
```

### Exercise 14.2: Chapter Extraction
Extract chapter content from an AO3 work page.

**Solution:**
```rust
fn extract_chapters(document: &Html) -> Vec<Chapter> {
    let mut chapters = Vec::new();
    
    for (i, div) in document.select(&Selector::parse("div.chapter").unwrap()).enumerate() {
        let title = div
            .select(&Selector::parse("h3.title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title,
            content,
        });
    }
    
    chapters
}
```

### Exercise 14.3: Tag Extraction
Extract tags from an AO3 work page.

**Solution:**
```rust
fn extract_tags(document: &Html) -> Vec<ExtractedTag> {
    let mut tags = Vec::new();
    
    // Fandom tags
    for li in document.select(&Selector::parse("dd.fandoms ul li").unwrap()) {
        let name = li.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }
    
    // Character tags
    for li in document.select(&Selector::parse("dd.characters ul li").unwrap()) {
        let name = li.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::character(&name));
        }
    }
    
    tags
}
```

## Chapter 15 Exercises: Other Site Scrapers

### Exercise 15.1: FF.net Scraper
Implement a complete FF.net scraper.

**Solution:**
```rust
pub struct FfNetScraper;

impl SiteScraper for FfNetScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiction.net") || url.contains("fictionpress.com")
    }
    
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Extract story ID from URL
        let re = regex_lite::Regex::new(r"/s/(\d+)").unwrap();
        let story_id = re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| ScrapeError::ParseError("invalid URL".into()))?;
        
        // Fetch story page
        let fic_url = format!("https://www.fanfiction.net/s/{}/1/", story_id);
        let response = client.get(&fic_url).send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        // Parse metadata
        let document = Html::parse_document(&html);
        let title = document
            .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        // ... more extraction ...
        
        Ok(FicMetadata { /* ... */ })
    }
}
```

### Exercise 15.2: XenForo Scraper
Implement a XenForo scraper for SpaceBattles.

**Solution:**
```rust
pub struct XenForoScraper;

impl SiteScraper for XenForoScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("spacebattles.com") || url.contains("sufficientvelocity.com")
    }
    
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let response = client.get(url).send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let document = Html::parse_document(&html);
        
        let title = document
            .select(&Selector::parse("h1.p-title-value").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        // ... more extraction ...
        
        Ok(FicMetadata { /* ... */ })
    }
}
```

### Exercise 15.3: Multi-Site Support
Create a scraper that handles multiple sites with different logic.

**Solution:**
```rust
pub struct MultiSiteScraper {
    sites: Vec<Box<dyn SiteScraper>>,
}

impl MultiSiteScraper {
    pub fn new() -> Self {
        MultiSiteScraper {
            sites: vec![
                Box::new(Ao3Scraper),
                Box::new(FfNetScraper),
                Box::new(XenForoScraper),
            ],
        }
    }
}

impl SiteScraper for MultiSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        self.sites.iter().any(|s| s.can_handle(url))
    }
    
    async fn lookup(&self, client: &Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        for scraper in &self.sites {
            if scraper.can_handle(url) {
                return scraper.lookup(client, url).await;
            }
        }
        Err(ScrapeError::NotFound)
    }
}
```

## Chapter 16 Exercises: Generating EPUB Files

### Exercise 16.1: EPUB Metadata
Add complete metadata to an EPUB file.

**Solution:**
```rust
builder.set_title(&meta.title)?;
builder.set_author(&meta.author)?;
builder.set_description(&meta.desc)?;
builder.set_language("en")?;
builder.set_publisher("FicHub")?;
```

### Exercise 16.2: EPUB Styling
Create a custom CSS stylesheet for EPUB files.

**Solution:**
```css
body {
    font-family: Georgia, serif;
    line-height: 1.6;
    margin: 1em;
}

h1 {
    font-size: 1.5em;
    text-align: center;
    border-bottom: 1px solid #ccc;
}

p {
    margin-bottom: 0.8em;
    text-indent: 1.5em;
}

blockquote {
    margin: 1em 2em;
    border-left: 3px solid #ccc;
    font-style: italic;
}
```

### Exercise 16.3: EPUB Error Handling
Implement error handling for EPUB generation failures.

**Solution:**
```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    // ... generation logic ...
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)
        .map_err(|e| ExportError::IoError(e.to_string()))?;
    
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    // ... hash computation ...
    
    Ok((epub_path, hash))
}
```

## Chapter 17 Exercises: HTML Bundles

### Exercise 17.1: HTML Template
Create an HTML template for story bundles.

**Solution:**
```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; }}
        h1 {{ text-align: center; }}
        .chapter {{ margin-top: 2em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>by {}</p>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>{}</ul>
    </div>
    <div class="content">{}</div>
</body>
</html>"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
        build_toc(chapters),
        build_content(chapters),
    )
}
```

### Exercise 17.2: ZIP Creation
Create a ZIP file containing HTML and CSS.

**Solution:**
```rust
use zip::write::SimpleFileOptions;

let zip_file = fs::File::create(&zip_path)?;
let mut zip = ZipWriter::new(zip_file);
let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated);

zip.start_file("story.html", options)?;
zip.write_all(html_content.as_bytes())?;

zip.start_file("style.css", options)?;
zip.write_all(css_content.as_bytes())?;

zip.finish()?;
```

### Exercise 17.3: Navigation
Add chapter navigation links to the HTML bundle.

**Solution:**
```html
<div class="nav">
    <a href="#chapter-1">Chapter 1</a>
    <a href="#chapter-2">Chapter 2</a>
    <a href="#chapter-3">Chapter 3</a>
</div>

<div class="chapter" id="chapter-1">
    <h2>Chapter 1</h2>
    <p>Content...</p>
    <div class="nav">
        <a href="#chapter-2">Next Chapter</a>
    </div>
</div>
```

## Chapter 18 Exercises: The Disk Cache

### Exercise 18.1: Cache Path
Implement cache path computation with directory structure.

**Solution:**
```rust
pub fn cache_path(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    hash: &str,
) -> PathBuf {
    let type_dir = match etype {
        EType::Epub => "epub",
        EType::Html => "html",
    };
    
    let chunks: Vec<&str> = url_id
        .as_bytes()
        .chunks(3)
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect();
    
    let mut path = cache_root.join(type_dir);
    for chunk in &chunks {
        path = path.join(chunk);
    }
    path = path.join(url_id);
    
    let suffix = match etype {
        EType::Epub => ".epub",
        EType::Html => ".zip",
    };
    
    path.join(format!("{}{}", hash, suffix))
}
```

### Exercise 18.2: Cache Cleanup
Implement a function to clean up old cached files.

**Solution:**
```rust
pub fn cleanup_old_cache(
    cache_root: &Path,
    max_age_days: u64,
) -> Result<u64, std::io::Error> {
    let mut removed = 0;
    let cutoff = SystemTime::now() - Duration::from_secs(max_age_days * 86400);
    
    for entry in fs::read_dir(cache_root)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if metadata.modified()? < cutoff {
            fs::remove_file(entry.path())?;
            removed += 1;
        }
    }
    
    Ok(removed)
}
```

### Exercise 18.3: Cache Statistics
Implement a function to get cache statistics.

**Solution:**
```rust
pub fn cache_stats(cache_root: &Path) -> Result<CacheStats, std::io::Error> {
    let mut total_files = 0;
    let mut total_size = 0;
    
    for entry in fs::read_dir(cache_root)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        
        if metadata.is_file() {
            total_files += 1;
            total_size += metadata.len();
        }
    }
    
    Ok(CacheStats {
        total_files,
        total_size,
    })
}
```

## Chapter 19 Exercises: Rate Limiting with Redis

### Exercise 19.1: Token Bucket
Implement a token bucket rate limiter.

**Solution:**
```rust
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
    
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
    
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}
```

### Exercise 19.2: Rate Limit Headers
Add rate limit information to HTTP responses.

**Solution:**
```rust
fn rate_limit_headers(remaining: i64, limit: i64, reset_at: i64) -> Vec<(String, String)> {
    vec![
        ("X-RateLimit-Limit".into(), limit.to_string()),
        ("X-RateLimit-Remaining".into(), remaining.to_string()),
        ("X-RateLimit-Reset".into(), reset_at.to_string()),
    ]
}
```

### Exercise 19.3: Multi-Tier Rate Limiting
Implement rate limiting with multiple tiers (global, per-IP, per-site).

**Solution:**
```rust
async fn check_multi_tier_rate_limit(
    redis: &mut MultiplexedConnection,
    ip: IpAddr,
    site: &str,
) -> Result<RateLimitResult, AppError> {
    // Check global rate limit
    if !check_bucket(redis, "rate_limit:global", 60, 1.0).await? {
        return Ok(RateLimitResult::Wait(5));
    }
    
    // Check per-IP rate limit
    let ip_key = format!("rate_limit:ip:{}", ip);
    if !check_bucket(redis, &ip_key, 30, 0.5).await? {
        return Ok(RateLimitResult::Wait(10));
    }
    
    // Check per-site rate limit
    let site_key = format!("rate_limit:site:{}", site);
    if !check_bucket(redis, &site_key, 20, 0.33).await? {
        return Ok(RateLimitResult::Wait(15));
    }
    
    Ok(RateLimitResult::Allowed)
}
```

## Chapter 20 Exercises: The Export Flow

### Exercise 20.1: Complete Flow
Implement the complete export flow from request to response.

**Solution:**
```rust
async fn export_flow(
    state: &AppState,
    url: &str,
) -> Result<ExportResponse, AppError> {
    // 1. Rate limit check
    state.rate_limiter.check(client_ip, url).await?;
    
    // 2. Find scraper
    let scraper = state.scraper_registry.find_scraper(url)
        .ok_or_else(|| AppError::BadRequest(-1, "unsupported URL".into()))?;
    
    // 3. Fetch metadata
    let meta = scraper.lookup(&state.http_client, url).await?;
    
    // 4. Upsert in database
    queries::upsert_fic_info(&state.db, &FicInfo::from_metadata(&meta)).await?;
    
    // 5. Check cache
    let input_hash = compute_input_hash(&meta);
    if let Some(cached) = queries::find_export_log(&state.db, &meta.url_id, 1, "epub", &input_hash).await? {
        return Ok(ExportResponse::cached(&meta, &cached));
    }
    
    // 6. Generate export
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
    let (epub_path, epub_hash) = export::epub::generate_epub(&state.config.tmp_dir, &meta, &chapters)?;
    
    // 7. Store in cache
    let cache_path = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    fs::rename(&epub_path, &cache_path)?;
    
    // 8. Record in database
    queries::insert_export_log(&state.db, &meta.url_id, 1, "epub", &input_hash, &epub_hash).await?;
    
    Ok(ExportResponse::new(&meta, &epub_hash))
}
```

### Exercise 20.2: Error Recovery
Implement error recovery for partial export failures.

**Solution:**
```rust
async fn export_with_recovery(
    state: &AppState,
    url: &str,
) -> Result<ExportResponse, AppError> {
    match export_flow(state, url).await {
        Ok(response) => Ok(response),
        Err(AppError::ScrapeError(_)) => {
            // Retry with different scraper
            export_flow_with_fallback(state, url).await
        }
        Err(AppError::ExportError(_)) => {
            // Clean up partial files
            cleanup_partial_export(state, url).await?;
            // Retry
            export_flow(state, url).await
        }
        Err(e) => Err(e),
    }
}
```

### Exercise 20.3: Concurrent Exports
Implement concurrent export generation for multiple stories.

**Solution:**
```rust
async fn export_multiple(
    state: &AppState,
    urls: Vec<String>,
) -> Vec<Result<ExportResponse, AppError>> {
    let futures: Vec<_> = urls.iter()
        .map(|url| {
            let state = state.clone();
            let url = url.clone();
            async move {
                export_flow(&state, &url).await
            }
        })
        .collect();
    
    futures::future::join_all(futures).await
}
```

## Chapter 21 Exercises: Collaborative Filtering

### Exercise 21.1: Co-occurrence Matrix
Build a co-occurrence matrix from user favourites.

**Solution:**
```rust
fn build_cooccurrence(favourites: &[(String, Vec<String>)]) -> HashMap<(String, String), i32> {
    let mut cooccurrence = HashMap::new();
    
    for (_, favs) in favourites {
        for i in 0..favs.len() {
            for j in (i + 1)..favs.len() {
                let pair = if favs[i] < favs[j] {
                    (favs[i].clone(), favs[j].clone())
                } else {
                    (favs[j].clone(), favs[i].clone())
                };
                *cooccurrence.entry(pair).or_insert(0) += 1;
            }
        }
    }
    
    cooccurrence
}
```

### Exercise 21.2: Jaccard Similarity
Implement the Jaccard similarity coefficient.

**Solution:**
```rust
fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}
```

### Exercise 21.3: Score Computation
Compute recommendation scores using co-occurrence and similarity.

**Solution:**
```rust
fn compute_score(
    cooccurrence_count: i32,
    story_a_favouriters: i32,
    story_b_favouriters: i32,
    voting_boost: f64,
) -> f64 {
    let union = (story_a_favouriters + story_b_favouriters - cooccurrence_count) as f64;
    if union == 0.0 {
        return 0.0;
    }
    
    let jaccard = cooccurrence_count as f64 / union;
    let boosted = jaccard * (1.0 + voting_boost);
    boosted.min(1.0)
}
```

## Chapter 22 Exercises: The Collection Worker

### Exercise 22.1: Worker Scheduling
Implement a worker that runs at specific intervals.

**Solution:**
```rust
async fn run_worker(state: Arc<AppState>, interval_hours: u64) {
    loop {
        match process_batch(&state).await {
            Ok(count) => {
                tracing::info!(processed = count, "Batch complete");
            }
            Err(e) => {
                tracing::error!(error = %e, "Batch failed");
            }
        }
        
        tokio::time::sleep(Duration::from_secs(interval_hours * 3600)).await;
    }
}
```

### Exercise 22.2: Error Recovery
Implement error recovery for failed user processing.

**Solution:**
```rust
async fn process_user_with_retry(
    state: &AppState,
    user: &RecUser,
    max_retries: u32,
) -> Result<(), AppError> {
    let mut delay = Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match process_user(state, user).await {
            Ok(()) => return Ok(()),
            Err(e) if attempt < max_retries - 1 => {
                tracing::warn!(
                    user_id = %user.id,
                    attempt = attempt,
                    error = %e,
                    "Retrying"
                );
                tokio::time::sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(AppError::Internal("max retries exceeded".into()))
}
```

### Exercise 22.3: Metrics
Add metrics to track worker performance.

**Solution:**
```rust
use prometheus::{IntCounter, Histogram};

lazy_static! {
    static ref USERS_PROCESSED: IntCounter = IntCounter::new(
        "fichub_worker_users_processed", "Users processed"
    ).unwrap();
    
    static ref FAVOURITES_COLLECTED: IntCounter = IntCounter::new(
        "fichub_worker_favourites_collected", "Favourites collected"
    ).unwrap();
    
    static ref PROCESSING_DURATION: Histogram = Histogram::new(
        "fichub_worker_processing_duration_seconds", "Processing duration"
    ).unwrap();
}
```

## Chapter 23 Exercises: Community Suggestions

### Exercise 23.1: Suggestion Submission
Implement a suggestion submission endpoint.

**Solution:**
```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuggestRequest>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("suggest:{}", remote_addr.ip());
    queries::check_rate_limit(&mut state.redis.clone(), &key, 5).await?;
    
    // Validate URL
    let scraper = state.scraper_registry.find_scraper(&payload.url)
        .ok_or_else(|| AppError::BadRequest(-1, "unsupported URL".into()))?;
    
    // Lookup metadata
    let meta = scraper.lookup(&state.http_client, &payload.url).await?;
    
    // Store suggestion
    sqlx::query(
        "INSERT INTO rec_suggestions (url_id, source_url, submitted_ip) VALUES ($1, $2, $3::inet)"
    )
    .bind(&meta.url_id)
    .bind(&payload.url)
    .bind(remote_addr.ip().to_string())
    .execute(&state.db)
    .await?;
    
    Ok(Json(json!({"err": 0, "url_id": meta.url_id, "msg": "suggestion submitted"})))
}
```

### Exercise 23.2: Vote Recording
Implement voting on suggestions.

**Solution:**
```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VoteRequest>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("rec_vote:{}", remote_addr.ip());
    queries::check_rate_limit(&mut state.redis.clone(), &key, 10).await?;
    
    // Upsert vote
    sqlx::query(
        r#"INSERT INTO rec_votes (url_id, recommended_url_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, recommended_url_id, voter_ip)
           DO UPDATE SET value = $4, created_at = NOW()"#
    )
    .bind(&payload.url_id)
    .bind(&payload.recommended_url_id)
    .bind(remote_addr.ip().to_string())
    .bind(payload.value)
    .execute(&state.db)
    .await?;
    
    Ok(Json(json!({"err": 0, "msg": "vote recorded"})))
}
```

### Exercise 23.3: Spam Prevention
Implement spam prevention for the suggestion system.

**Solution:**
```rust
async fn check_suggestion_spam(
    pool: &PgPool,
    ip: &IpAddr,
    url_id: &str,
) -> Result<bool, AppError> {
    // Check if IP has submitted too many suggestions recently
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM rec_suggestions WHERE submitted_ip = $1::inet AND created > NOW() - INTERVAL '1 hour'"
    )
    .bind(ip.to_string())
    .fetch_one(pool)
    .await?;
    
    if count.0 > 5 {
        return Ok(true);  // Spam detected
    }
    
    // Check if URL has already been suggested
    let exists: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM rec_suggestions WHERE url_id = $1"
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(exists.is_some())
}
```

## Chapter 24 Exercises: Voting and Scoring

### Exercise 24.1: Vote Recording
Implement tag voting with score computation.

**Solution:**
```rust
pub async fn record_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &IpAddr,
    value: i16,
) -> Result<VoteResult, AppError> {
    // Upsert vote
    sqlx::query(
        r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, voter_ip)
           DO UPDATE SET value = $4, created_at = NOW()"#
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .bind(value)
    .execute(pool)
    .await?;
    
    // Compute new score
    let score: (i16,) = sqlx::query_as(
        "SELECT COALESCE(SUM(value), 0) FROM fic_tag_votes WHERE url_id = $1 AND tag_id = $2"
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;
    
    // Update tag score
    sqlx::query("UPDATE fic_tags SET score = $1 WHERE url_id = $2 AND tag_id = $3")
        .bind(score.0)
        .bind(url_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    
    Ok(VoteResult { new_score: score.0 })
}
```

### Exercise 24.2: Auto-Moderation
Implement auto-moderation for tags with low scores.

**Solution:**
```rust
pub async fn auto_moderate(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
    auto_delete_threshold: Option<i16>,
) -> Result<(), AppError> {
    let tags = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT t.id, t.name, ft.score FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id WHERE ft.url_id = $1"
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    
    for (tag_id, tag_name, score) in tags {
        if let Some(threshold) = auto_delete_threshold {
            if score <= threshold {
                sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
                    .bind(url_id)
                    .bind(tag_id)
                    .execute(pool)
                    .await?;
                
                tracing::info!(url_id = %url_id, tag = %tag_name, score = score, "Auto-deleted tag");
            }
        }
    }
    
    Ok(())
}
```

### Exercise 24.3: Vote History
Implement a function to get vote history for a tag.

**Solution:**
```rust
pub async fn get_vote_history(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
) -> Result<Vec<VoteHistory>, AppError> {
    let history = sqlx::query_as::<_, VoteHistory>(
        r#"SELECT voter_ip, value, created_at
           FROM fic_tag_votes
           WHERE url_id = $1 AND tag_id = $2
           ORDER BY created_at DESC"#
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_all(pool)
    .await?;
    
    Ok(history)
}
```

## Chapter 25 Exercises: The Full Axum Router

### Exercise 25.1: Route Audit
List all routes in FicHub and categorize them.

**Solution:**
See the complete route reference in Part 7.

### Exercise 25.2: Route Testing
Write integration tests for each route group.

**Solution:**
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

### Exercise 25.3: API Versioning
Design a v1 API with breaking changes.

**Solution:**
```rust
// v0: /api/v0/epub?q=URL
// v1: /api/v1/export?url=URL&type=epub

let app = Router::new()
    .nest("/api/v0", v0_routes())
    .nest("/api/v1", v1_routes());
```

## Chapter 26 Exercises: Serving Static Files

### Exercise 26.1: Cache Headers
Add appropriate cache headers for different file types.

**Solution:**
```rust
use tower_http::set_header::SetResponseHeaderLayer;

let html_cache = SetResponseHeaderLayer::overriding(
    header::CACHE_CONTROL,
    HeaderValue::from_static("no-cache"),
);

let asset_cache = SetResponseHeaderLayer::overriding(
    header::CACHE_CONTROL,
    HeaderValue::from_static("public, max-age=31536000"),
);
```

### Exercise 26.2: Compression
Enable gzip compression for static files.

**Solution:**
```rust
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .nest_service("/assets", get(serve_assets).layer(CompressionLayer::new()))
    .fallback_service(ServeDir::new(&frontend_dir));
```

### Exercise 26.3: Security Headers
Add security headers to static file responses.

**Solution:**
```rust
let security_headers = ServiceBuilder::new()
    .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    ));
```

## Chapter 27 Exercises: Docker and Docker Compose

### Exercise 27.1: Docker Build
Build the FicHub Docker image and verify it starts.

**Solution:**
```bash
docker build -t fichub .
docker run -d --name fichub -p 3000:3000 fichub
docker logs fichub
curl http://localhost:3000/health
```

### Exercise 27.2: Docker Compose
Set up a complete Docker Compose environment.

**Solution:**
```bash
docker-compose up -d
docker-compose ps
docker-compose logs -f fichub
```

### Exercise 27.3: Health Checks
Add health checks to each Docker service.

**Solution:**
```yaml
services:
  fichub:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
  
  postgres:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 10s
      timeout: 5s
      retries: 5
```

## Chapter 28 Exercises: Cross-Compilation and Deploy

### Exercise 28.1: Cross-Compile
Cross-compile FicHub for ARM64.

**Solution:**
```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

### Exercise 28.2: Systemd Service
Create a systemd service file for FicHub.

**Solution:**
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

### Exercise 28.3: Deployment Automation
Write a deployment script that handles building, copying, and restarting.

**Solution:**
```bash
#!/bin/bash
set -e

# Build
cargo build --release --target aarch64-unknown-linux-gnu

# Copy
scp target/aarch64-unknown-linux-gnu/release/fichub user@server:/opt/fichub/

# Restart
ssh user@server "sudo systemctl restart fichub"

echo "Deployment complete!"
```

## Chapter 29 Exercises: The Search System

### Exercise 29.1: Search Query
Implement a search query with multiple filters.

**Solution:**
```rust
fn build_search_query(params: &SearchParams) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new(
        "SELECT fi.id, fi.title, fi.author, fi.words FROM fic_info fi WHERE 1=1"
    );
    
    if let Some(ref q) = params.q {
        qb.push(" AND to_tsvector('english', fi.title) @@ plainto_tsquery('english', ");
        qb.push_bind(q.clone());
        qb.push(")");
    }
    
    for tag in &params.include_tags {
        qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id WHERE ft.url_id = fi.id AND t.tag_type_id = ");
        qb.push_bind(tag.tag_type_id);
        qb.push(" AND t.name = ");
        qb.push_bind(&tag.tag_name);
        qb.push(")");
    }
    
    if let Some(min) = params.min_words {
        qb.push(" AND fi.words >= ");
        qb.push_bind(min);
    }
    
    qb
}
```

### Exercise 29.2: Search Autocomplete
Implement search autocomplete.

**Solution:**
```rust
pub async fn autocomplete_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AutocompleteQuery>,
) -> Result<Json<Value>, AppError> {
    let suggestions = sqlx::query_as::<_, (String,)>(
        "SELECT DISTINCT title FROM fic_info WHERE title ILIKE $1 LIMIT 10"
    )
    .bind(format!("%{}%", params.q))
    .fetch_all(&state.db)
    .await?;
    
    Ok(Json(json!({
        "err": 0,
        "suggestions": suggestions.into_iter().map(|(t,)| t).collect::<Vec<_>>()
    })))
}
```

### Exercise 29.3: Search Analytics
Track popular search queries.

**Solution:**
```rust
pub async fn track_search(pool: &PgPool, query: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO search_log (query, created) VALUES ($1, NOW())"
    )
    .bind(query)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn popular_searches(pool: &PgPool, limit: i64) -> Result<Vec<String>, AppError> {
    let searches = sqlx::query_as::<_, (String,)>(
        "SELECT query, COUNT(*) as count FROM search_log GROUP BY query ORDER BY count DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    
    Ok(searches.into_iter().map(|(q,)| q).collect())
}
```

## Chapter 30 Exercises: The Tag System

### Exercise 30.1: Tag Resolution
Implement tag resolution with alias support.

**Solution:**
```rust
pub async fn resolve_tag(
    pool: &PgPool,
    name: &str,
    tag_type_id: i16,
) -> Result<TagResolution, AppError> {
    // Check exact match
    if let Some((id, name, type_id)) = queries::lookup_tag_by_name(pool, name).await? {
        return Ok(TagResolution { tag_id: id, tag_name: name, tag_type_id: type_id, is_new: false });
    }
    
    // Check alias
    if let Some(canonical_id) = queries::lookup_alias(pool, name).await? {
        let (id, name, type_id) = queries::lookup_tag_by_id(pool, canonical_id).await?;
        return Ok(TagResolution { tag_id: id, tag_name: name, tag_type_id: type_id, is_new: false });
    }
    
    // Create new tag
    let id = queries::create_tag(pool, name, tag_type_id).await?;
    Ok(TagResolution { tag_id: id, tag_name: name.to_string(), tag_type_id, is_new: true })
}
```

### Exercise 30.2: Tag Statistics
Get tag usage statistics.

**Solution:**
```rust
pub async fn tag_stats(pool: &PgPool) -> Result<Vec<TagStats>, AppError> {
    let stats = sqlx::query_as::<_, TagStats>(
        r#"SELECT t.name, t.tag_type_id, COUNT(ft.url_id) as usage_count
           FROM tags t
           LEFT JOIN fic_tags ft ON t.id = ft.tag_id
           GROUP BY t.id, t.name, t.tag_type_id
           ORDER BY usage_count DESC"#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(stats)
}
```

### Exercise 30.3: Bulk Tag Operations
Implement bulk tag operations.

**Solution:**
```rust
pub async fn bulk_add_tags(
    pool: &PgPool,
    url_id: &str,
    tag_names: &[(String, i16)],
) -> Result<(), AppError> {
    for (name, tag_type_id) in tag_names {
        let resolution = resolve_tag(pool, name, *tag_type_id).await?;
        queries::upsert_fic_tag(pool, url_id, resolution.tag_id, &IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))).await?;
    }
    Ok(())
}
```

## Chapter 31 Exercises: The OPDS Catalog

### Exercise 31.1: OPDS Feed
Generate an OPDS feed for recent stories.

**Solution:**
```rust
pub async fn recent_feed(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let stories = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info ORDER BY fic_updated DESC LIMIT 20"
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    
    let entries: String = stories.iter()
        .map(|s| story_to_opds_entry(s))
        .collect();
    
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <id>urn:fichub:recent</id>
    <title>Recent Stories</title>
    <updated>{}</updated>
    {}
</feed>"#,
        Utc::now().to_rfc3339(),
        entries
    );
    
    ([("Content-Type", "application/atom+xml;profile=opds-catalog")], xml)
}
```

### Exercise 31.2: OPDS Navigation
Implement multi-level OPDS navigation.

**Solution:**
```rust
pub async fn root_catalog() -> impl IntoResponse {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
    <id>urn:fichub:catalog</id>
    <title>FicHub Catalog</title>
    <entry>
        <title>New Stories</title>
        <link href="/opds/new" type="application/atom+xml;profile=opds-catalog"/>
    </entry>
    <entry>
        <title>Popular Stories</title>
        <link href="/opds/popular" type="application/atom+xml;profile=opds-catalog"/>
    </entry>
    <entry>
        <title>Browse by Tags</title>
        <link href="/opds/tags" type="application/atom+xml;profile=opds-catalog"/>
    </entry>
</feed>"#;
    
    ([("Content-Type", "application/atom+xml;profile=opds-catalog")], xml)
}
```

### Exercise 31.3: OPDS Authentication
Add authentication support for private shelves.

**Solution:**
```rust
pub async fn shelf_contents(
    Path(shelf_id): Path<String>,
    Query(params): Query<ShelfQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // Verify shelf token
    if params.token != state.config.opds_shelf_token {
        return Err(AppError::BadRequest(-403, "invalid token".into()));
    }
    
    // Fetch shelf contents
    let stories = queries::get_shelf_stories(&state.db, &shelf_id).await?;
    
    // Generate OPDS feed
    let xml = build_opds_feed(&stories);
    
    Ok(([("Content-Type", "application/atom+xml;profile=opds-catalog")], xml))
}
```

## Chapter 32 Exercises: The API Documentation

### Exercise 32.1: API Docs
Create a comprehensive API documentation endpoint.

**Solution:**
```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "description": "Export a story as EPUB",
                "params": {"q": "URL of the story"},
                "response": {"err": 0, "url_id": "...", "urls": {...}}
            }
        }
    }))
}
```

### Exercise 32.2: OpenAPI Spec
Generate an OpenAPI 3.0 specification.

**Solution:**
```yaml
openapi: 3.0.0
info:
  title: FicHub API
  version: 0.1.0
paths:
  /api/v0/epub:
    get:
      summary: Export a story
      parameters:
        - name: q
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Success
```

### Exercise 32.3: Interactive Docs
Integrate Swagger UI for interactive documentation.

**Solution:**
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(export_handler, search_handler),
    components(schemas(ExportResponse, SearchResponse))
)]
struct ApiDoc;

async fn swagger_ui() -> impl IntoResponse {
    let spec = ApiDoc::openapi().to_pretty_json().unwrap();
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({{
            spec: {},
            dom_id: '#swagger-ui'
        }});
    </script>
</body>
</html>"#,
        spec
    ))
}
```

## Chapter 33 Exercises: Performance and Optimization

### Exercise 33.1: Flamegraph
Generate a flamegraph of FicHub handling requests.

**Solution:**
```bash
cargo install flamegraph
cargo flamegraph --bench my_benchmark
# Open flamegraph.svg in browser
```

### Exercise 33.2: Query Analysis
Run EXPLAIN ANALYZE on slow queries and optimize them.

**Solution:**
```sql
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM fic_info WHERE words > 10000 ORDER BY fic_updated DESC LIMIT 20;
```

### Exercise 33.3: Connection Pool
Monitor and tune the connection pool.

**Solution:**
```rust
let stats = pool.stats();
tracing::info!(
    active = stats.active_connections(),
    idle = stats.idle_connections(),
    waiting = stats.waiting(),
    "Pool stats"
);
```

## Chapter 34 Exercises: Backend Testing

### Exercise 34.1: Unit Tests
Write unit tests for core functions.

**Solution:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_url_id() {
        let id = generate_url_id(1, "story_123");
        assert_eq!(id.len(), 12);
    }
    
    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<b>"), "&lt;b&gt;");
    }
}
```

### Exercise 34.2: Integration Tests
Write integration tests for API endpoints.

**Solution:**
```rust
#[tokio::test]
async fn test_search_endpoint() {
    let state = setup_test_state().await;
    let app = build_router(Arc::new(state));
    
    let response = axum_test::TestClient::new(app)
        .get("/api/v0/search")
        .query(&[("q", "test")])
        .await;
    
    assert_eq!(response.status_code(), 200);
}
```

### Exercise 34.3: Property Tests
Write property-based tests for URL ID generation.

**Solution:**
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

## Chapter 35 Exercises: Production Hardening

### Exercise 35.1: Security Audit
Review FicHub for security vulnerabilities.

**Solution:**
1. Input validation on all endpoints
2. SQL injection prevention (parameterized queries)
3. Rate limiting on all endpoints
4. CORS configuration
5. Security headers
6. Secrets management
7. Dependency auditing

### Exercise 35.2: Monitoring Dashboard
Set up a Grafana dashboard with FicHub metrics.

**Solution:**
- Request rate and latency
- Error rate
- Cache hit/miss ratio
- Database connection pool usage
- Export generation time
- Rate limit hits

### Exercise 35.3: Alerting
Configure alerts for error rate spikes and high latency.

**Solution:**
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

## Chapter 36 Exercises: What's Next?

### Exercise 36.1: Feature Proposal
Design a new feature for FicHub.

**Solution:**
Feature: User Reading Lists
- Database: reading_lists, reading_list_items tables
- API: GET/POST/DELETE /api/v0/lists
- UI: List management interface
- Integration: Export from lists

### Exercise 36.2: Code Review
Review a pull request in an open-source Rust project.

**Solution:**
1. Check for correctness
2. Check for performance
3. Check for security
4. Check for documentation
5. Check for tests

### Exercise 36.3: Build Something New
Build a new project using FicHub patterns.

**Solution:**
Build a recipe collection app with:
- Scraping recipes from websites
- Generating cookbook EPUBs
- Full-text search
- Tag system
- Rate limiting

