# Extended Content: Comprehensive Reference Material

---

# Complete Rust Reference for FicHub Developers

## Rust Standard Library Quick Reference

### String Types

Rust has two main string types: `String` (owned) and `&str` (borrowed).

```rust
// String: owned, growable
let mut s = String::from("hello");
s.push_str(" world");
s.push('!');

// &str: borrowed, immutable slice
let s: &str = "hello world";

// Conversions
let s: String = "hello".to_string();
let s: String = String::from("hello");
let s: &str = &my_string;  // Deref coercion
let s: &str = my_string.as_str();
```

### Collection Types

```rust
// Vec<T>: dynamic array
let mut v = vec![1, 2, 3];
v.push(4);
let first = v[0];  // Panics if out of bounds
let first = v.get(0);  // Returns Option<&T>

// HashMap<K, V>: hash map
let mut map = HashMap::new();
map.insert("key".to_string(), 42);
let value = map.get("key");  // Returns Option<&V>
let value = map.remove("key");  // Returns Option<V>

// HashSet<T>: hash set
let mut set = HashSet::new();
set.insert(1);
set.insert(2);
let contains = set.contains(&1);  // true

// BTreeMap<K, V>: ordered map
let mut map = BTreeMap::new();
map.insert(3, "c");
map.insert(1, "a");
// Iterates in key order: (1, "a"), (3, "c")
```

### Option<T> and Result<T, E>

```rust
// Option<T>: nullable value
let some_value: Option<i32> = Some(42);
let no_value: Option<i32> = None;

// Pattern matching
match some_value {
    Some(v) => println!("Value: {}", v),
    None => println!("No value"),
}

// Unwrap methods
let v = some_value.unwrap();  // Panics if None
let v = some_value.unwrap_or(0);  // Default if None
let v = some_value.unwrap_or_else(|| compute_default());
let v = some_value.expect("value must exist");

// Result<T, E>: success or error
let ok: Result<i32, String> = Ok(42);
let err: Result<i32, String> = Err("error".into());

// ? operator for error propagation
fn process() -> Result<i32, String> {
    let value = some_operation()?;  // Returns Err if Err
    Ok(value)
}
```

### Iterators

```rust
let v = vec![1, 2, 3, 4, 5];

// Map: transform each element
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();

// Filter: keep matching elements
let evens: Vec<&i32> = v.iter().filter(|x| *x % 2 == 0).collect();

// Fold: combine all elements
let sum: i32 = v.iter().fold(0, |acc, x| acc + x);

// Any: check if any element matches
let has_even = v.iter().any(|x| x % 2 == 0);

// Find: find first matching element
let first_even = v.iter().find(|x| x % 2 == 0);

// Enumerate: add index
let indexed: Vec<(usize, &i32)> = v.iter().enumerate().collect();

// Zip: combine two iterators
let names = vec!["Alice", "Bob"];
let ages = vec![25, 30];
let people: Vec<(&str, &i32)> = names.iter().zip(ages.iter()).collect();

// Chain: combine iterators
let combined: Vec<i32> = v1.iter().chain(v2.iter()).cloned().collect();
```

## Axum Reference

### Request Extractors

```rust
// Query parameters
async fn handler(Query(params): Query<MyParams>) -> impl IntoResponse {}

// Path parameters
async fn handler(Path(id): Path<String>) -> impl IntoResponse {}

// JSON body
async fn handler(Json(payload): Json<MyPayload>) -> impl IntoResponse {}

// Headers
async fn handler(headers: HeaderMap) -> impl IntoResponse {}

// Connection info
async fn handler(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {}

// State
async fn handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {}
```

### Response Types

```rust
// JSON
async fn handler() -> Json<Value> {
    Json(json!({"key": "value"}))
}

// Status code
async fn handler() -> StatusCode {
    StatusCode::OK
}

// Custom response
async fn handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"key": "value"})))
}

// With headers
async fn handler() -> impl IntoResponse {
    (
        [("X-Custom", "value")],
        Json(json!({"key": "value"}))
    )
}
```

### Router Configuration

```rust
let app = Router::new()
    .route("/path", get(handler))
    .route("/path", post(handler))
    .nest("/prefix", sub_router)
    .layer(middleware)
    .with_state(state);
```

## SQLx Reference

### Query Types

```rust
// Execute (no result)
sqlx::query("INSERT INTO table VALUES ($1)")
    .bind(value)
    .execute(&pool)
    .await?;

// Fetch one row
let row: MyStruct = sqlx::query_as("SELECT * FROM table WHERE id = $1")
    .bind(id)
    .fetch_one(&pool)
    .await?;

// Fetch optional
let row: Option<MyStruct> = sqlx::query_as("SELECT * FROM table WHERE id = $1")
    .bind(id)
    .fetch_optional(&pool)
    .await?;

// Fetch all
let rows: Vec<MyStruct> = sqlx::query_as("SELECT * FROM table")
    .fetch_all(&pool)
    .await?;

// Fetch scalar
let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM table")
    .fetch_one(&pool)
    .await?;
```

### Transactions

```rust
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO table VALUES ($1)")
    .bind(value)
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

### Query Builder

```rust
use sqlx::QueryBuilder;

let mut qb = QueryBuilder::new("SELECT * FROM table WHERE 1=1");

if let Some(name) = &params.name {
    qb.push(" AND name = ");
    qb.push_bind(name.clone());
}

if let Some(min_age) = params.min_age {
    qb.push(" AND age >= ");
    qb.push_bind(min_age);
}

let results = qb
    .build_query_as::<MyStruct>()
    .fetch_all(&pool)
    .await?;
```

## Tokio Reference

### Task Spawning

```rust
// Fire and forget
tokio::spawn(async { work().await });

// Wait for result
let result = tokio::spawn(async { compute().await }).await?;

// Join multiple tasks
let (r1, r2) = tokio::join!(
    task1(),
    task2(),
);

// Select first completion
tokio::select! {
    r = task1() => handle_r1(r),
    r = task2() => handle_r2(r),
}
```

### Channels

```rust
// mpsc: multiple producers, single consumer
let (tx, mut rx) = mpsc::channel(100);
tx.send(value).await?;
let value = rx.recv().await?;

// broadcast: multiple producers, multiple consumers
let (tx, _) = broadcast::channel(100);
tx.send(value)?;
let value = rx.recv().await?;

// oneshot: single producer, single consumer
let (tx, rx) = oneshot::channel();
tx.send(value)?;
let value = rx.await?;
```

### Synchronization

```rust
// Mutex
let data = Arc::new(Mutex::new(vec![]));
let mut guard = data.lock().await;
guard.push(value);

// RwLock
let data = Arc::new(RwLock::new(vec![]));
let guard = data.read().await;  // Multiple readers
let mut guard = data.write().await;  // Single writer

// Semaphore
let sem = Arc::new(Semaphore::new(3));
let permit = sem.acquire().await?;
// ... do work ...
drop(permit);  // Release
```

### Time

```rust
// Sleep
tokio::time::sleep(Duration::from_secs(5)).await;

// Interval
let mut interval = interval(Duration::from_secs(1));
interval.tick().await;

// Timeout
let result = timeout(Duration::from_secs(30), async { work() }).await;
```

## Serde Reference

### Derive Attributes

```rust
#[derive(Serialize, Deserialize)]
struct MyStruct {
    #[serde(default)]                    // Use default if missing
    #[serde(skip_serializing_if = "Option::is_none")]  // Skip if None
    #[serde(rename = "camelCase")]       // Use different name
    #[serde(flatten)]                    // Flatten nested struct
    #[serde(with = "module")]            // Custom serialization
    #[serde(deserialize_with = "func")]  // Custom deserialization
    field: Type,
}
```

### Custom Serialize

```rust
use serde::{Serialize, Serializer};

fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(*value)
}
```

### Custom Deserialize

```rust
use serde::{Deserialize, Deserializer};

fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    String::deserialize(deserializer)
}
```

## Common Patterns

### Builder Pattern

```rust
struct ConfigBuilder {
    port: Option<u16>,
    host: Option<String>,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder { port: None, host: None }
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    
    fn build(self) -> Result<Config, String> {
        Ok(Config {
            port: self.port.ok_or("port required")?,
            host: self.host.ok_or("host required")?,
        })
    }
}
```

### Newtype Pattern

```rust
struct UrlId(String);

impl UrlId {
    fn new(source_id: i64, story_id: &str) -> Self {
        // ... generate ID ...
        UrlId(id)
    }
    
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UrlId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### Type-State Pattern

```rust
struct Unvalidated;
struct Validated;

struct Request<State> {
    url: String,
    _state: std::marker::PhantomData<State>,
}

impl Request<Unvalidated> {
    fn validate(self) -> Result<Request<Validated>, Error> {
        // ...
        Ok(Request { url: self.url, _state: std::marker::PhantomData })
    }
}
```

### Strategy Pattern

```rust
trait Strategy {
    fn execute(&self, data: &str) -> String;
}

struct UpperCase;
impl Strategy for UpperCase {
    fn execute(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

struct LowerCase;
impl Strategy for LowerCase {
    fn execute(&self, data: &str) -> String {
        data.to_lowercase()
    }
}

fn process(data: &str, strategy: &dyn Strategy) -> String {
    strategy.execute(data)
}
```

## Database Schema Reference

### FicHub Core Tables

```sql
-- Story metadata
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL DEFAULT 1,
    words INT8 NOT NULL DEFAULT 0,
    description TEXT DEFAULT '',
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    source_id INT8,
    author_id INT8,
    content_hash TEXT,
    extra_meta JSONB,
    raw_extended_meta JSONB
);

-- Export cache
CREATE TABLE export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

-- Tags
CREATE TABLE tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Story-tag associations
CREATE TABLE fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

-- Tag votes
CREATE TABLE fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

-- Recommendations
CREATE TABLE rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);
```

### Indexes

```sql
-- Core indexes
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_source ON fic_info(source);
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX idx_export_log_url_id ON export_log(url_id);

-- Full-text search
CREATE INDEX idx_fic_info_search ON fic_info
    USING gin(to_tsvector('english', title || ' ' || author));

-- Tag indexes
CREATE INDEX idx_fic_tags_url_id ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag_id ON fic_tags(tag_id);

-- Recommendation indexes
CREATE INDEX idx_rec_cooccurrence_a ON rec_cooccurrence(story_a);
CREATE INDEX idx_rec_cooccurrence_b ON rec_cooccurrence(story_b);
CREATE INDEX idx_rec_favourites_user ON rec_favourites(user_id);
```

## Environment Variables Reference

```bash
# Required
DATABASE_URL=postgres://user:pass@host/db
REDIS_URL=redis://host:port

# Optional with defaults
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug

# Rate limiting
DYNAMIC_RATE_LIMIT=true
TRUSTED_PROXIES=10.0.0.1,10.0.0.2
IP_TAG_SOURCES=/path/to/ips,cloudflare,cdn

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
REC_PRECOMPUTE_ENABLED=true
REC_PRECOMPUTE_INTERVAL_HOURS=6
REC_ENABLE_CROSS_SITE=true

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

## API Reference

### Export Endpoints

```
GET /api/v0/epub?q={url}
  Response: {"err": 0, "url_id": "...", "meta": {...}, "urls": {"epub": "...", "html": "..."}}

GET /api/v0/meta?url_id={id}
  Response: {"err": 0, "meta": {...}}
```

### Cache Download

```
GET /cache/{type}/{url_id}?h={hash}
  Response: Binary file (EPUB or ZIP)
```

### Search

```
GET /api/v0/search?q={query}&tags={tags}&min_words={min}&max_words={max}&complete={bool}&sort={sort}&page={page}
  Response: {"err": 0, "total": N, "page": N, "per_page": N, "results": [...]}
```

### Tags

```
POST /api/v0/tags/submit
  Body: {"url_id": "...", "name": "...", "tag_type_id": N}
  Response: {"err": 0, "tag_id": N, "is_new": bool}

POST /api/v0/tags/vote
  Body: {"url_id": "...", "tag_id": N, "value": 1|-1}
  Response: {"err": 0, "new_score": N, "hidden": bool}

GET /api/v0/tags?url_id={id}
  Response: {"err": 0, "tags": [...]}
```

### Recommendations

```
GET /api/v0/recommendations?url_id={id}&limit={N}
  Response: {"err": 0, "recommendations": [...]}

POST /api/v0/recommendations/suggest
  Body: {"url": "..."}
  Response: {"err": 0, "url_id": "...", "title": "..."}

POST /api/v0/recommendations/vote
  Body: {"url_id": "...", "recommended_url_id": "...", "value": 1|-1}
  Response: {"err": 0, "upvotes": N, "downvotes": N}
```

### Curator (Admin)

```
POST /api/v0/curator/alias
  Body: {"alias_name": "...", "canonical_name": "...", "tag_type_id": N}
  Response: {"err": 0, "alias": "...", "canonical": "..."}

POST /api/v0/curator/merge
  Body: {"source_tag_id": N, "target_tag_id": N}
  Response: {"err": 0, "msg": "tags merged"}

DELETE /api/v0/curator/tags/{id}?force={bool}
  Response: {"err": 0, "msg": "tag deleted"}

GET /api/v0/curator/flags
  Response: {"err": 0, "flags": [...]}

POST /api/v0/curator/flags/{id}/resolve
  Response: {"err": 0, "msg": "flag resolved"}
```

### OPDS

```
GET /opds
  Response: OPDS root catalog (Atom XML)

GET /opds/new
  Response: Recent stories (Atom XML)

GET /opds/popular
  Response: Popular stories (Atom XML)

GET /opds/tags
  Response: Tag types (Atom XML)

GET /opds/tags/{type_id}
  Response: Tags by type (Atom XML)

GET /opds/tags/{type_id}/{tag_name}
  Response: Stories by tag (Atom XML)

GET /opds/authors
  Response: Author list (Atom XML)

GET /opds/search?q={query}
  Response: Search results (Atom XML)
```

## Deployment Checklist

### Pre-Deployment

- [ ] Run all tests (`cargo test`)
- [ ] Run clippy (`cargo clippy`)
- [ ] Run formatter (`cargo fmt`)
- [ ] Audit dependencies (`cargo audit`)
- [ ] Build release binary (`cargo build --release`)
- [ ] Cross-compile for target platform
- [ ] Test on target platform

### Docker

- [ ] Multi-stage Dockerfile
- [ ] Non-root user
- [ ] Read-only filesystem
- [ ] Resource limits
- [ ] Health checks
- [ ] Logging configuration

### Database

- [ ] Run migrations
- [ ] Create indexes
- [ ] Configure connection pooling
- [ ] Set up backups
- [ ] Configure replication (if needed)

### Redis

- [ ] Configure persistence
- [ ] Set memory limits
- [ ] Configure eviction policy
- [ ] Set up replication (if needed)

### Monitoring

- [ ] Structured logging
- [ ] Prometheus metrics
- [ ] Health check endpoint
- [ ] Alerting rules
- [ ] Dashboard

### Security

- [ ] Input validation
- [ ] Rate limiting
- [ ] CORS configuration
- [ ] Security headers
- [ ] Secrets management
- [ ] Dependency auditing

### Performance

- [ ] Connection pool tuning
- [ ] Query optimization
- [ ] Index strategy
- [ ] Caching strategy
- [ ] Load testing

## Glossary of FicHub-Specific Terms

- **url_id** — Unique identifier for a story, generated from source_id + story_id using SHA-256
- **source_id** — Numeric identifier for a fanfiction site (1=AO3, 2=FF.net, 3=XenForo)
- **export_log** — Database table tracking cached exports
- **input_hash** — MD5 hash of the metadata used to generate an export
- **export_hash** — MD5 hash of the generated export file
- **EType** — Export type enum (Epub, Html)
- **ScraperRegistry** — Maps URLs to the appropriate scraper
- **CacheSemaphores** — Prevents duplicate concurrent exports
- **CollectionWorker** — Background task that scrapes user favourites
- **RecommendationEngine** — Computes story recommendations
- **TokenBucket** — Rate limiting algorithm implementation
- **TagResolution** — Process of mapping tag strings to canonical tags

