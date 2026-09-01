# Supplementary Content: Practical Guides and Extended Concepts

---

# Extended Guide: Database Schema Design

## Why Schema Design Matters

The database schema is the foundation of any data-driven application. A well-designed schema ensures data integrity, query performance, and maintainability. A poorly designed schema leads to slow queries, data corruption, and frustrating development experiences.

FicHub's schema went through several iterations. Let's examine the design decisions behind each table.

## Core Tables

### fic_info — The Central Table

The `fic_info` table is the heart of FicHub. Every fic that's ever been scraped gets a row here:

```sql
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,           -- Deterministic hash-based ID
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,                   -- Site-specific story ID
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,       -- When the fic was published
    fic_updated TIMESTAMPTZ NOT NULL,       -- When the fic was last updated
    status TEXT NOT NULL,                   -- "ongoing", "complete", etc.
    source TEXT NOT NULL,                   -- Original URL
    extra_meta TEXT,                        -- Additional metadata (JSON)
    raw_extended_meta TEXT,                 -- Raw extended metadata
    source_id INT8,                         -- Site identifier (1=AO3, 2=FF.net)
    author_id INT8,                         -- Site-specific author ID
    content_hash VARCHAR(256)               -- Hash of story content
);
```

Design decisions:
1. **VARCHAR(128) for id** — The hash-based ID is 12 hex chars, but we use 128 to leave room for future changes
2. **TEXT for description** — Descriptions can be very long and are often HTML
3. **TIMESTAMPTZ** — Always store timestamps with timezone information
4. **content_hash** — Enables cache invalidation when stories are updated
5. **Separate created/updated from fic_created/fic_updated** — Our timestamps vs the story's timestamps

### export_log — The Cache Key Table

```sql
CREATE TABLE export_log (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    version INT NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, version, etype, input_hash)
);
```

The composite key `(url_id, version, etype, input_hash)` is the cache key:
- **url_id** — Which fic
- **version** — Export version (bumped on format changes)
- **etype** — Export type (epub, html, mobi, pdf)
- **input_hash** — Hash of the input data (changes when story is updated)

### Blacklist Tables

```sql
CREATE TABLE fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);

CREATE TABLE author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);
```

Blacklists use numeric reasons:
- **5** — DMCA/legal takedown
- **6** — Greylisted (metadata shown, no download)
- **7** — Content policy violation
- **8** — Other

The greylist (reason 6) is a middle ground — users can see the metadata but can't download the file.

## Recommender Tables

### fic_works

```sql
CREATE TABLE fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

This table extends `fic_info` with recommendation-specific data. The `ON DELETE CASCADE` ensures cleanup when a fic is deleted.

### fic_bookmarks

```sql
CREATE TABLE fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

User hashes are SHA-256 of profile URLs — one-way, anonymous, but deterministic.

### fic_bookmark_cooccur

```sql
CREATE TABLE fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

The `CHECK (work_a < work_b)` constraint canonicalizes the pair order, preventing duplicates.

### precomputed_recommendations

```sql
CREATE TABLE precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

This is a materialized cache — computed results stored for fast retrieval.

## Tagging Tables

### tags

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` makes comparisons case-sensitive. This is intentional — "Harry Potter" and "harry potter" are different tags.

### fic_tags

```sql
CREATE TABLE fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

The junction table connecting fics to tags. The `score` column is automatically updated by a PostgreSQL trigger.

### fic_tag_votes

```sql
CREATE TABLE fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INTEGER NOT NULL,
    voter_ip INET NOT NULL,
    value SMALLINT NOT NULL CHECK (value IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id, voter_ip),
    FOREIGN KEY (url_id, tag_id) REFERENCES fic_tags(url_id, tag_id) ON DELETE CASCADE
);
```

Each vote is either +1 (upvote) or -1 (downvote). The composite primary key ensures one vote per IP per tag per fic.

### The Score Trigger

```sql
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value
        WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

Three triggers fire on INSERT, UPDATE, and DELETE. This ensures the score is always accurate without application-level logic.

## OPDS Tables

### opds_shelves

```sql
CREATE TABLE opds_shelves (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

### opds_shelf_items

```sql
CREATE TABLE opds_shelf_items (
    shelf_id INTEGER NOT NULL REFERENCES opds_shelves(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, url_id)
);
```

## Schema Evolution

FicHub's schema evolved through four migrations:

1. **001_initial_schema** — Core tables (fic_info, request_log, export_log, blacklists)
2. **002_recommender** — Recommendation engine (fic_works, bookmarks, co-occurrence, suggestions, votes)
3. **003_tagging** — Tag system v3 (tags, aliases, fic_tags, votes, flags, full-text search)
4. **004_shelves** — OPDS shelves

Each migration is idempotent (`CREATE TABLE IF NOT EXISTS`) and can be run multiple times safely.

## Summary

FicHub's database schema is designed for:
- **Data integrity** — Foreign keys, unique constraints, check constraints
- **Performance** — Appropriate indexes, partial indexes, composite indexes
- **Maintainability** — Clear naming, consistent conventions, documented purpose
- **Evolution** — Idempotent migrations, backward-compatible changes

---

# Extended Guide: Error Handling Patterns

## Why Error Handling Matters

In a web server, errors are not exceptional — they're expected. Users send bad URLs, sites go down, databases lose connections. A well-designed error handling system ensures the server stays running and users get helpful error messages.

## FicHub's Error Architecture

### The Error Enum

```rust
#[derive(Debug)]
pub enum AppError {
    BadRequest(i32, String),
    RateLimited(u64),
    NotFound(String),
    Internal(String),
    ScrapeError(String),
    ExportError(String),
    Database(String),
    CacheError(String),
}
```

Each variant represents a different class of error. The associated data varies:
- **BadRequest** — Error code + message (code is used by the frontend)
- **RateLimited** — Retry-after seconds
- **NotFound** — What wasn't found
- **Internal** — Generic server error
- **ScrapeError** — What went wrong during scraping
- **ExportError** — What went wrong during export
- **Database** — What went wrong with the database
- **CacheError** — What went wrong with Redis

### Display Implementation

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

### IntoResponse Implementation

This is where errors become HTTP responses:

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
                    "retry_after": retry_after
                }))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "internal server error"
                }))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -5,
                    "msg": "export failed"
                }))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "database error"
                }))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "cache error"
                }))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

Key design decisions:
1. **BadRequest returns the error code** — The frontend can handle specific error types
2. **RateLimited includes retry_after** — The client knows when to retry
3. **ScrapeError returns 502** — The error originated from an upstream site
4. **Internal, Database, CacheError, ExportError return 500** — The client gets a generic message, but the server logs the details
5. **NotFound returns 404** — Standard HTTP semantics

### From Implementations

The `?` operator works because `AppError` implements `From` for various error types:

```rust
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

## Error Handling in Handlers

### Simple Pattern

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // ... more processing ...

    Ok(Json(json!({"err": 0, "data": result})))
}
```

### Pattern Matching Errors

```rust
match scraper.lookup(&state.http_client, query).await {
    Ok(meta) => {
        // Process metadata
    }
    Err(ScrapeError::NotFound) => {
        return Ok(Json(json!({"err": -5, "msg": "story not found"})));
    }
    Err(ScrapeError::Blocked) => {
        return Ok(Json(json!({"err": -6, "msg": "site is blocking requests"})));
    }
    Err(e) => {
        return Err(AppError::ScrapeError(e.to_string()));
    }
}
```

### Contextual Errors

Add context to errors for debugging:

```rust
let pool = db::init_pool(&config.database_url)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;
```

## Error Logging

FicHub logs errors at different levels:

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);  // Always logged
    // ...
}
AppError::ScrapeError(msg) => {
    // NOT logged here — logged at the handler level if needed
    // ...
}
AppError::BadRequest(code, msg) => {
    // NOT logged — client errors are expected
    // ...
}
```

The principle: log server errors (500s), don't log client errors (400s).

## Summary

FicHub's error handling uses a centralized enum with variants for each error class, `From` implementations for automatic conversion, `IntoResponse` for HTTP responses, and appropriate logging levels.

---

# Extended Guide: Configuration Management

## Environment Variables

FicHub uses environment variables for all configuration. This follows the twelve-factor app methodology.

### Loading Configuration

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        // ... 40+ more settings ...
    }
}
```

### Required vs Optional Settings

**Required** — Server can't start without them:
- `DATABASE_URL`
- `REDIS_URL`

**Optional with defaults** — Sensible defaults provided:
- `CACHE_DIR` → `./cache`
- `PORT` → `3000`
- `RUST_LOG` → `info,fichub=debug`

**Optional without defaults** — Feature-specific:
- `SECONDARY_CACHE_DIR` → `None`
- `CALIBRE_CONTAINER` → `""`
- `CURATOR_TOKEN` → `None`

### The dotenvy Crate

FicHub loads a `.env` file if present:

```rust
dotenvy::dotenv().ok();
```

The `.ok()` silently ignores missing files. The `.env` file is loaded before reading environment variables, so its values serve as defaults.

### The .env File

```bash
# Required
DATABASE_URL=postgres://fichub:fichub@localhost/fichub
REDIS_URL=redis://localhost/0

# Optional with defaults
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug

# Recommender settings
REC_DEFAULT_DELAY_SECS=5
REC_MAX_FAVOURITE_PAGES=3
REC_MAX_RECOMMENDATIONS=20
REC_VOTING_BOOST_GAMMA=0.2
REC_CACHE_TTL_HOURS=12

# Tagging settings
CURATOR_TOKEN=my-secret-token
TAG_HIDDEN_THRESHOLD=-3
TAG_SUBMIT_LIMIT_PER_HOUR=10
TAG_VOTE_LIMIT_PER_HOUR=20
SEARCH_MAX_PER_PAGE=50

# OPDS
OPDS_SHELF_TOKEN=fichub
```

### Parsing Configuration Values

Each setting is parsed from a string to its appropriate type:

```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

The pattern:
1. `std::env::var("PORT")` — Get the string value
2. `.unwrap_or_else(|_| "3000".to_string())` — Default if not set
3. `.parse()` — Convert to the target type
4. `.unwrap_or(3000)` — Default if parsing fails

### Complex Configuration

Some settings are more complex:

```rust
// JSON hash map
let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
    .unwrap_or_else(|_| "{}".to_string());
let rec_site_rate_limits: HashMap<String, u64> =
    serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();

// Comma-separated list
let trusted_proxies = std::env::var("TRUSTED_PROXIES")
    .unwrap_or_default()
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();

// Multi-line structured data
let ip_tag_sources = std::env::var("IP_TAG_SOURCES")
    .unwrap_or_default()
    .lines()
    .filter_map(|line| {
        let parts: Vec<&str> = line.splitn(3, ',').collect();
        if parts.len() == 3 {
            Some((parts[0].trim().to_string(), parts[1].trim().to_string(), parts[2].trim().to_string()))
        } else {
            None
        }
    })
    .collect();
```

## Configuration Testing

FicHub has thorough configuration tests:

```rust
#[test]
fn test_from_env_defaults() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test_db");
    guard.set("REDIS_URL", "redis://localhost/0");

    let config = Config::from_env();

    assert_eq!(config.database_url, "postgres://localhost/test_db");
    assert_eq!(config.cache_dir, PathBuf::from("./cache"));
    assert_eq!(config.app_port, 3000);
    assert_eq!(config.rec_default_delay_secs, 5);
    assert_eq!(config.rec_voting_boost_gamma, 0.2);
    assert!(config.rec_precompute_enabled);
}
```

The `EnvGuard` struct automatically cleans up environment variables when it goes out of scope, preventing test pollution.

## Summary

FicHub's configuration system uses environment variables with sensible defaults, supports complex data types (JSON, comma-separated lists), and has comprehensive tests. The `dotenvy` crate provides a convenient `.env` file for development.
