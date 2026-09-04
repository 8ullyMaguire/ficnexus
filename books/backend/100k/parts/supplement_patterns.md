# Supplementary Content: Design Patterns and Architecture

---

# Design Patterns Used in FicHub

## The Strategy Pattern

FicHub uses the Strategy pattern extensively for its pluggable scraper system. The `SiteScraper` trait defines the interface, and each scraper provides a different implementation:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

This pattern makes it trivial to add new scrapers — just implement the trait and register it. The rest of the system doesn't need to change.

## The Registry Pattern

The `ScraperRegistry` is a classic Registry pattern — a central lookup table that maps identifiers (URLs) to implementations (scrapers):

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

impl ScraperRegistry {
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
}
```

This decouples the rest of the application from the specific scrapers. New scrapers are added to the registry without modifying any other code.

## The Repository Pattern

FicHub's database layer follows the Repository pattern. Each query function is a standalone function that takes a connection pool:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query("INSERT INTO fic_info ... ON CONFLICT ... DO UPDATE ...")
        .bind(&fic.id)
        // ...
        .execute(pool)
        .await?;
    Ok(())
}
```

This keeps database logic centralized and testable.

## The Builder Pattern

FicHub uses the Builder pattern for constructing complex objects:

### reqwest Client

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

### SQLx Query Builder

```rust
let mut qb = QueryBuilder::<Postgres>::new("SELECT fi.* FROM fic_info fi WHERE 1=1");

if let Some(ref q) = self.params.q {
    qb.push(" AND fi.text_search @@ plainto_tsquery('english', ");
    qb.push_bind(q);
    qb.push(")");
}

if let Some(min) = self.params.min_words {
    qb.push(" AND fi.words >= ");
    qb.push_bind(min);
}
```

### Axum Router

```rust
let app = Router::new()
    .route("/api/", get(api_docs_handler))
    .route("/api/v0/epub", get(epub_handler))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(state);
```

Each method call returns a modified builder, allowing fluent configuration.

## The Observer Pattern (via PostgreSQL Triggers)

FicHub uses PostgreSQL triggers as a form of the Observer pattern. When a vote is inserted, updated, or deleted, the trigger automatically updates the tag score:

```sql
CREATE TRIGGER trg_fic_tag_vote_insert
    AFTER INSERT ON fic_tag_votes
    FOR EACH ROW EXECUTE FUNCTION update_fic_tag_score();
```

The application doesn't need to manually update scores — the database handles it automatically.

## The Factory Pattern

The `ScraperRegistry::new()` method is a factory that creates all known scrapers:

```rust
pub fn new() -> Self {
    let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
    scrapers.push(Box::new(sites::ao3::Ao3Scraper));
    scrapers.push(Box::new(sites::ffnet::FfNetScraper));
    scrapers.push(Box::new(sites::xenforo::XenForoScraper));
    scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
    scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
    scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
    ScraperRegistry { scrapers }
}
```

## The Proxy Pattern

FicHub's caching system acts as a proxy — it intercepts requests and serves cached responses when available, only hitting the real data source (scraping) when the cache is empty:

```rust
// Check cache first
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Cache miss — do the real work
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await?;
```

## The Chain of Responsibility Pattern

The middleware stack in Axum is a Chain of Responsibility — each layer can handle the request or pass it to the next:

```rust
Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(CorsLayer::permissive())    // Layer 1: Handle CORS
    .layer(TraceLayer::new_for_http()) // Layer 2: Log request
```

Request flows: CorsLayer → TraceLayer → Router → handler

## The Template Method Pattern

The EPUB and HTML generation follow a template method pattern — the overall structure is fixed, but the content varies:

```rust
// Fixed structure
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
builder.metadata("title", &meta.title)?;
builder.stylesheet(css.as_bytes())?;
builder.add_content(introduction_page)?;

// Variable content
for chapter in chapters {
    builder.add_content(chapter_page(&chapter))?;
}

// Fixed finalization
builder.generate(file)?;
```

## The Double-Check Locking Pattern

FicHub uses double-check locking for cache validation:

```rust
// First check (no lock)
let cached = queries::find_export_log(&db, &url_id, version, "epub", &hash).await?;
if cached.is_some() { return Ok(cached); }

// Acquire lock
let sem = get_export_semaphore(&semaphores, &url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check (with lock)
let cached = queries::find_export_log(&db, &url_id, version, "epub", &hash).await?;
if cached.is_some() { return Ok(cached); }

// Do work
generate_epub().await?;
```

This pattern minimizes lock contention while preventing duplicate work.

## The Circuit Breaker Pattern

FicHub's rate limiter acts as a circuit breaker. When a site is blocking requests, the rate limiter returns `RateLimitResult::Blocked`:

```rust
if self.is_datacenter_ip(ip) {
    return RateLimitResult::Blocked;
}
```

This prevents the system from continuing to hammer a site that's rejecting requests.

## Summary

FicHub employs many classic design patterns: Strategy (scrapers), Registry (scraper lookup), Repository (database queries), Builder (configuration), Observer (triggers), Factory (registry creation), Proxy (caching), Chain of Responsibility (middleware), Template Method (export generation), Double-Check Locking (cache validation), and Circuit Breaker (rate limiting). Understanding these patterns makes the codebase easier to navigate and extend.

---

# Architectural Overview

## Layer Architecture

FicHub follows a layered architecture:

```
┌─────────────────────────────────────────┐
│           HTTP Layer (Axum)              │
│  Routes, Extractors, Middleware          │
├─────────────────────────────────────────┤
│         Application Layer               │
│  Export Flow, Recommendations, Search    │
├─────────────────────────────────────────┤
│          Domain Layer                   │
│  Scrapers, Exporters, Tag Resolution     │
├─────────────────────────────────────────┤
│         Data Access Layer               │
│  SQLx Queries, Redis Operations          │
├─────────────────────────────────────────┤
│        Infrastructure Layer             │
│  PostgreSQL, Redis, File System          │
└─────────────────────────────────────────┘
```

Each layer depends only on the layer below it. The HTTP layer doesn't know about PostgreSQL — it goes through the Application layer, which goes through the Data Access layer.

## Module Organization

```
src/
├── main.rs          → Entry point (infra layer)
├── server.rs        → HTTP setup (HTTP layer)
├── config.rs        → Configuration (infra layer)
├── error.rs         → Error types (cross-cutting)
├── db/              → Data access layer
│   ├── mod.rs       → Connection pool
│   ├── models.rs    → Data structures
│   └── queries.rs   → SQL queries
├── scrape/          → Domain layer (scraping)
│   ├── mod.rs       → Trait definitions
│   ├── registry.rs  → Scraper registry
│   └── sites/       → Site-specific scrapers
├── export/          → Domain layer (export)
│   ├── mod.rs       → Export types
│   ├── epub.rs      → EPUB generation
│   ├── html_bundle.rs → HTML generation
│   └── convert.rs   → Format conversion
├── cache/           → Domain layer (caching)
│   ├── mod.rs       → Cache types
│   └── disk.rs      → Disk operations
├── limiter/         → Domain layer (rate limiting)
│   ├── mod.rs       → Rate limiter trait
│   └── redis_bucket.rs → Token bucket impl
├── recommender/     → Application layer
│   ├── mod.rs       → Recommender types
│   ├── engine.rs    → Recommendation computation
│   ├── worker.rs    → Background collection
│   └── routes.rs    → API handlers
├── search/          → Application layer
│   ├── mod.rs       → Search modules
│   ├── builder.rs   → Query builder
│   └── routes.rs    → Search handler
├── tags/            → Application layer
│   ├── mod.rs       → Tag modules
│   ├── resolve.rs   → Tag resolution
│   ├── voting.rs    → Vote management
│   ├── routes.rs    → Tag handlers
│   └── curator.rs   → Curator tools
├── routes/          → HTTP layer
│   ├── mod.rs       → Route modules
│   ├── export.rs    → Export handler
│   ├── meta.rs      → Metadata handler
│   ├── cache_download.rs → Download handlers
│   ├── api_docs.rs  → API documentation
│   └── opds/        → OPDS catalog
└── frontend/        → Frontend module
    └── mod.rs       → Constants
```

## Data Flow

### Request Flow

```
HTTP Request
    ↓
Router (URL matching)
    ↓
Middleware (CORS, Logging)
    ↓
Handler (Parameter extraction)
    ↓
Domain Logic (Business rules)
    ↓
Data Access (Database/Redis)
    ↓
Response (JSON/XML)
```

### Scraping Flow

```
URL Input
    ↓
Scraper Selection (Registry lookup)
    ↓
HTTP Request (to fanfiction site)
    ↓
HTML Parsing (scraper crate)
    ↓
Data Extraction (CSS selectors)
    ↓
Metadata Assembly (FicMetadata)
    ↓
Database Storage (UPSERT)
```

### Export Flow

```
Metadata
    ↓
Cache Check (export_log table)
    ├── Hit → Return cached URLs
    └── Miss ↓
        ↓
Semaphore Acquisition (prevent duplicates)
        ↓
Double-Check Cache
        ├── Hit → Return cached URLs
        └── Miss ↓
            ↓
Chapter Fetching (from fanfiction site)
            ↓
EPUB Generation (epub-builder)
            ↓
HTML Generation (template)
            ↓
Cache Storage (move to cache dir)
            ↓
Database Recording (export_log)
            ↓
Response Building (JSON with URLs)
```

## Concurrency Model

FicHub's concurrency model is built on Tokio:

```
┌─────────────────────────────────────┐
│           Tokio Runtime             │
│  ┌─────────────────────────────┐    │
│  │     Worker Threads (N)      │    │
│  │  ┌───┐ ┌───┐ ┌───┐ ┌───┐  │    │
│  │  │ T │ │ T │ │ T │ │ T │  │    │
│  │  └───┘ └───┘ └───┘ └───┘  │    │
│  └─────────────────────────────┘    │
│                                     │
│  ┌─────────────────────────────┐    │
│  │     Blocking Thread Pool    │    │
│  │  ┌───┐ ┌───┐ ┌───┐        │    │
│  │  │ B │ │ B │ │ B │        │    │
│  │  └───┘ └───┘ └───┘        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

- **Async tasks** handle HTTP requests, database queries, and Redis operations
- **Blocking tasks** handle CPU-intensive work (EPUB generation, file I/O)
- **Shared state** is protected by `Arc` (read-only) or `Mutex`/`RwLock` (read-write)

## Summary

FicHub's architecture is clean, layered, and well-organized. Each module has a single responsibility, dependencies flow downward, and the concurrency model leverages Tokio's async runtime efficiently. Understanding this architecture makes it easy to navigate the codebase and add new features.
