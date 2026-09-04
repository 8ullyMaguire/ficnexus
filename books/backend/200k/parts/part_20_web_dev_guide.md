# Extended Content: Web Development Comprehensive Guide

---

# Complete HTTP Reference for FicHub Developers

## HTTP Methods

### GET

The GET method retrieves data from the server. It should not have side effects — it should only read data, not modify it.

```rust
async fn get_story(
    Query(params): Query<GetStoryQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let story = queries::get_fic_info(&state.db, &params.url_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Story not found".into()))?;
    
    Ok(Json(json!({
        "err": 0,
        "story": story
    })))
}
```

### POST

The POST method creates a new resource or submits data for processing.

```rust
async fn create_tag(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTagRequest>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    let tag_id = tags::resolve::resolve_tag(
        &state.db, &payload.name, payload.tag_type_id
    ).await?.tag_id;
    
    tags::routes::associate_tag(
        &state.db, &payload.url_id, tag_id, &remote_addr.ip()
    ).await?;
    
    Ok(Json(json!({
        "err": 0,
        "tag_id": tag_id,
        "msg": "tag created"
    })))
}
```

### PUT

The PUT method replaces an existing resource entirely.

```rust
async fn update_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateStoryRequest>,
) -> Result<Json<Value>, AppError> {
    sqlx::query(
        "UPDATE fic_info SET title = $1, author = $2, updated = NOW() WHERE id = $3"
    )
    .bind(&payload.title)
    .bind(&payload.author)
    .bind(&id)
    .execute(&state.db)
    .await?;
    
    Ok(Json(json!({
        "err": 0,
        "msg": "story updated"
    })))
}
```

### DELETE

The DELETE method removes a resource.

```rust
async fn delete_tag(
    Path(id): Path<i32>,
    Query(params): Query<DeleteTagQuery>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    curator::verify_curator_token(&headers, &state.config)?;
    
    queries::delete_tag(&state.db, id, params.force.unwrap_or(false)).await?;
    
    Ok(Json(json!({
        "err": 0,
        "msg": "tag deleted"
    })))
}
```

### PATCH

The PATCH method applies partial modifications to a resource.

```rust
async fn patch_story(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PatchStoryRequest>,
) -> Result<Json<Value>, AppError> {
    let mut updates = Vec::new();
    let mut bind_values: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send>> = Vec::new();
    
    if let Some(title) = &payload.title {
        updates.push(format!("title = ${}", updates.len() + 1));
    }
    
    // ... build dynamic query ...
    
    Ok(Json(json!({"err": 0, "msg": "story patched"})))
}
```

## HTTP Status Codes

### Success Codes (2xx)

- **200 OK** — Request succeeded
- **201 Created** — Resource successfully created
- **204 No Content** — Request succeeded but no content to return

### Client Error Codes (4xx)

- **400 Bad Request** — Invalid request syntax or parameters
- **401 Unauthorized** — Authentication required
- **403 Forbidden** — Authenticated but not authorized
- **404 Not Found** — Resource doesn't exist
- **429 Too Many Requests** — Rate limited

### Server Error Codes (5xx)

- **500 Internal Server Error** — Unexpected server error
- **502 Bad Gateway** — Upstream server returned invalid response
- **503 Service Unavailable** — Server temporarily overloaded

## HTTP Headers

### Request Headers

```rust
async fn handler(
    headers: HeaderMap,
) -> Json<Value> {
    let user_agent = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    let accept = headers
        .get("Accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");
    
    let authorization = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    
    Json(json!({
        "user_agent": user_agent,
        "accept": accept,
        "has_auth": authorization.is_some()
    }))
}
```

### Response Headers

```rust
async fn handler() -> impl IntoResponse {
    let body = json!({"message": "hello"});
    
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
            (header::X_REQUEST_ID, Uuid::new_v4().to_string()),
        ],
        Json(body)
    )
}
```

## Content Negotiation

FicHub supports different response formats based on the Accept header:

```rust
async fn export_handler(
    headers: HeaderMap,
    Query(params): Query<ExportQuery>,
) -> Response {
    let accept = headers
        .get("Accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    
    if accept.contains("application/json") {
        // Return JSON metadata
        Json(json!({"err": 0, "url_id": "..."})).into_response()
    } else if accept.contains("application/epub+zip") {
        // Return EPUB file
        let file = fs::read(path).unwrap();
        ([(header::CONTENT_TYPE, "application/epub+zip")], file).into_response()
    } else {
        // Default to JSON
        Json(json!({"err": 0})).into_response()
    }
}
```

## CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any, Origin};

// Development: allow everything
let cors = CorsLayer::permissive();

// Production: restrict origins
let cors = CorsLayer::new()
    .allow_origin(Origin::list(vec![
        "https://fichub.net".parse().unwrap(),
        "https://www.fichub.net".parse().unwrap(),
    ]))
    .allow_methods(Any)
    .allow_headers(Any)
    .max_age(Duration::from_secs(3600));
```

## Request Validation

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct CreateStoryRequest {
    #[validate(length(min = 1, max = 500))]
    title: String,
    
    #[validate(length(min = 1, max = 200))]
    author: String,
    
    #[validate(url)]
    url: String,
    
    #[validate(range(min = 0, max = 10_000_000))]
    word_count: i64,
}

async fn create_story(
    Json(payload): Json<CreateStoryRequest>,
) -> Result<Json<Value>, AppError> {
    payload.validate()
        .map_err(|e| AppError::BadRequest(-1, e.to_string()))?;
    
    // Process valid request
    Ok(Json(json!({"err": 0, "msg": "story created"})))
}
```

## Rate Limiting Implementation

```rust
async fn check_rate_limit(
    redis: &mut MultiplexedConnection,
    key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<bool, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let window_start = now - window_seconds;
    
    // Use Redis sorted set for sliding window
    let mut pipe = redis::pipe();
    pipe.cmd("ZREMRANGEBYSCORE")
        .arg(key)
        .arg(0)
        .arg(window_start)
        .ignore()
        .cmd("ZADD")
        .arg(key)
        .arg(now)
        .arg(now)
        .ignore()
        .cmd("ZCARD")
        .arg(key)
        .ignore()
        .cmd("EXPIRE")
        .arg(key)
        .arg(window_seconds)
        .ignore();
    
    let results: ((), (), i64, ()) = pipe.query_async(redis).await?;
    let count = results.2;
    
    Ok(count <= max_requests as i64)
}
```

## Error Response Format

FicHub uses a consistent error response format:

```json
{
    "err": -1,
    "msg": "error message",
    "retry_after": 30
}
```

Error codes:
- `0` — Success
- `-1` — Internal error
- `-5` — Not found
- `-6` — Upstream error
- `-403` — Forbidden
- `-429` — Rate limited

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => (
                StatusCode::BAD_REQUEST,
                json!({"err": code, "msg": msg})
            ),
            AppError::RateLimited(retry_after) => (
                StatusCode::TOO_MANY_REQUESTS,
                json!({"err": -429, "msg": "rate limited", "retry_after": retry_after})
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                json!({"err": -5, "msg": msg})
            ),
            AppError::Internal(msg) => {
                tracing::error!(error = %msg, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "internal server error"})
                )
            }
            AppError::ScrapeError(msg) => (
                StatusCode::BAD_GATEWAY,
                json!({"err": -6, "msg": msg})
            ),
            AppError::ExportError(msg) => {
                tracing::error!(error = %msg, "Export error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -5, "msg": "export failed"})
                )
            }
            AppError::Database(msg) => {
                tracing::error!(error = %msg, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "database error"})
                )
            }
            AppError::CacheError(msg) => {
                tracing::error!(error = %msg, "Cache error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    json!({"err": -1, "msg": "cache error"})
                )
            }
        };
        
        (status, Json(body)).into_response()
    }
}
```

---

# Complete PostgreSQL Reference

## Data Types

### Numeric Types

| Type | Storage | Range |
|------|---------|-------|
| `SMALLINT` | 2 bytes | -32,768 to 32,767 |
| `INTEGER` | 4 bytes | -2,147,483,648 to 2,147,483,647 |
| `BIGINT` | 8 bytes | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 |
| `REAL` | 4 bytes | 6 decimal digits precision |
| `DOUBLE PRECISION` | 8 bytes | 15 decimal digits precision |
| `NUMERIC` | variable | Exact numeric, user-specified precision |

### String Types

| Type | Description |
|------|-------------|
| `CHAR(n)` | Fixed-length string, padded with spaces |
| `VARCHAR(n)` | Variable-length string with limit |
| `TEXT` | Variable-length string, no limit |

### Date/Time Types

| Type | Description |
|------|-------------|
| `DATE` | Calendar date |
| `TIME` | Time of day |
| `TIMESTAMP` | Date and time without timezone |
| `TIMESTAMPTZ` | Date and time with timezone |
| `INTERVAL` | Time span |

### Boolean Type

```sql
-- Boolean
CREATE TABLE example (
    is_active BOOLEAN DEFAULT TRUE
);

INSERT INTO example VALUES (TRUE);
INSERT INTO example VALUES (FALSE);
INSERT INTO example VALUES (NULL);
```

### JSON Types

```sql
-- JSONB (binary, indexed)
CREATE TABLE stories (
    id SERIAL PRIMARY KEY,
    metadata JSONB
);

INSERT INTO stories (metadata) VALUES (
    '{"title": "My Story", "tags": ["fantasy", "adventure"]}'
);

-- Query JSONB
SELECT metadata->>'title' FROM stories;
SELECT metadata @> '{"tags": ["fantasy"]}' FROM stories;
```

## SQL Commands

### DDL (Data Definition Language)

```sql
-- Create table
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    chapters INT4 DEFAULT 1,
    words INT8 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Alter table
ALTER TABLE fic_info ADD COLUMN description TEXT;
ALTER TABLE fic_info ALTER COLUMN chapters SET DEFAULT 1;
ALTER TABLE fic_info DROP COLUMN old_column;

-- Drop table
DROP TABLE IF EXISTS fic_info;

-- Create index
CREATE INDEX idx_fic_info_title ON fic_info(title);
CREATE INDEX CONCURRENTLY idx_fic_info_search ON fic_info
    USING gin(to_tsvector('english', title));
```

### DML (Data Manipulation Language)

```sql
-- Insert
INSERT INTO fic_info (id, title, author, chapters, words)
VALUES ('abc123', 'My Story', 'Author', 10, 50000);

-- Insert with conflict handling
INSERT INTO fic_info (id, title, author)
VALUES ('abc123', 'My Story', 'Author')
ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title;

-- Update
UPDATE fic_info SET chapters = 11 WHERE id = 'abc123';

-- Delete
DELETE FROM fic_info WHERE id = 'abc123';

-- Select
SELECT id, title, author FROM fic_info WHERE words > 10000;

-- Select with join
SELECT fi.title, ft.score
FROM fic_info fi
JOIN fic_tags ft ON fi.id = ft.url_id
WHERE ft.tag_id = 1;

-- Aggregate
SELECT author, COUNT(*) as story_count, SUM(words) as total_words
FROM fic_info
GROUP BY author
HAVING COUNT(*) > 5
ORDER BY total_words DESC;

-- Subquery
SELECT * FROM fic_info
WHERE id IN (
    SELECT url_id FROM fic_tags
    WHERE tag_id = 1
);

-- Full-text search
SELECT * FROM fic_info
WHERE to_tsvector('english', title) @@ plainto_tsquery('english', 'harry potter');
```

### DCL (Data Control Language)

```sql
-- Grant privileges
GRANT SELECT, INSERT, UPDATE ON fic_info TO fichub;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO fichub;

-- Revoke privileges
REVOKE DELETE ON fic_info FROM fichub;

-- Create role
CREATE ROLE readonly LOGIN PASSWORD 'password';
GRANT CONNECT ON DATABASE fichub TO readonly;
GRANT USAGE ON SCHEMA public TO readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO readonly;
```

## Indexes

### B-Tree Index

```sql
-- Default index type
CREATE INDEX idx_fic_info_words ON fic_info(words);

-- Unique index
CREATE UNIQUE INDEX idx_fic_info_id ON fic_info(id);

-- Composite index
CREATE INDEX idx_fic_info_status_words ON fic_info(status, words);
```

### GIN Index

```sql
-- For full-text search
CREATE INDEX idx_fic_info_search ON fic_info
    USING gin(to_tsvector('english', title));

-- For JSONB
CREATE INDEX idx_fic_info_metadata ON fic_info
    USING gin(extra_meta);

-- For arrays
CREATE INDEX idx_fic_info_tags ON fic_info
    USING gin(tags);
```

### GiST Index

```sql
-- For geometric data
CREATE INDEX idx_location ON places
    USING gist(location);

-- For full-text search
CREATE INDEX idx_fic_info_description ON fic_info
    USING gist(to_tsvector('english', description));
```

### Partial Index

```sql
-- Index only active stories
CREATE INDEX idx_fic_info_active ON fic_info(fic_updated)
WHERE status = 'ongoing';

-- Index only recent stories
CREATE INDEX idx_fic_info_recent ON fic_info(fic_updated)
WHERE fic_updated > NOW() - INTERVAL '30 days';
```

### Expression Index

```sql
-- Index on expression
CREATE INDEX idx_fic_info_lower_title ON fic_info(lower(title));

-- Index on function
CREATE INDEX idx_fic_info_word_count ON fic_info((words / 1000));
```

## Transactions

```sql
-- Begin transaction
BEGIN;

-- Operations
INSERT INTO fic_info (id, title) VALUES ('abc', 'My Story');
UPDATE fic_tags SET score = score + 1 WHERE url_id = 'abc';

-- Commit
COMMIT;

-- Or rollback
ROLLBACK;
```

### Isolation Levels

```sql
-- Read uncommitted
SET TRANSACTION ISOLATION LEVEL READ UNCOMMITTED;

-- Read committed (default)
SET TRANSACTION ISOLATION LEVEL READ COMMITTED;

-- Repeatable read
SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;

-- Serializable
SET TRANSACTION ISOLATION LEVEL SERIALIZABLE;
```

## Views

```sql
-- Create view
CREATE VIEW active_stories AS
SELECT id, title, author, words
FROM fic_info
WHERE status = 'complete'
AND words > 10000;

-- Use view
SELECT * FROM active_stories WHERE author = 'J.K. Rowling';

-- Materialized view
CREATE MATERIALIZED VIEW story_stats AS
SELECT author, COUNT(*) as story_count, SUM(words) as total_words
FROM fic_info
GROUP BY author;

-- Refresh materialized view
REFRESH MATERIALIZED VIEW story_stats;
```

## Stored Procedures

```sql
-- Create function
CREATE OR REPLACE FUNCTION update_story_count()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE author_stats
    SET story_count = story_count + 1
    WHERE author_id = NEW.author_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger
CREATE TRIGGER after_story_insert
AFTER INSERT ON fic_info
FOR EACH ROW
EXECUTE FUNCTION update_story_count();
```

## Performance Tuning

### EXPLAIN ANALYZE

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM fic_info WHERE words > 10000 ORDER BY fic_updated DESC LIMIT 20;
```

### Statistics

```sql
-- Update statistics
ANALYZE fic_info;

-- Check statistics
SELECT * FROM pg_stats WHERE tablename = 'fic_info';
```

### Configuration

```sql
-- Check current settings
SHOW shared_buffers;
SHOW work_mem;
SHOW effective_cache_size;

-- Change settings
SET work_mem = '256MB';
SET shared_buffers = '256MB';
SET effective_cache_size = '1GB';
```

---

# Complete Docker Reference

## Dockerfile Best Practices

### Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.77-bookworm as builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

# Build application
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN addgroup --system fichub && adduser --system --ingroup fichub fichub

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/fichub .

# Set ownership
RUN chown -R fichub:fichub /app

# Switch to non-root user
USER fichub

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run
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
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 10s
      timeout: 5s
      retries: 5
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
  
  fichub:
    build: .
    environment:
      DATABASE_URL: postgres://fichub:fichub@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      CACHE_DIR: /cache
      TMP_DIR: /tmp
      PORT: 3000
    volumes:
      - fichub-cache:/cache
      - fichub-tmp:/tmp
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    read_only: true
    tmpfs:
      - /tmp
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G

volumes:
  pgdata:
  fichub-cache:
  fichub-tmp:
```

## Docker Commands

```bash
# Build image
docker build -t fichub .

# Run container
docker run -d --name fichub -p 3000:3000 fichub

# View logs
docker logs -f fichub

# Execute command in container
docker exec -it fichub bash

# Stop container
docker stop fichub

# Remove container
docker rm fichub

# Remove image
docker rmi fichub

# List containers
docker ps -a

# List images
docker images

# Prune unused resources
docker system prune -a
```

## Docker Networking

```bash
# Create network
docker network create fichub-network

# Run container on network
docker run -d --name fichub --network fichub-network fichub

# Connect container to network
docker network connect fichub-network existing-container

# Inspect network
docker network inspect fichub-network
```

## Docker Volumes

```bash
# Create volume
docker volume create fichub-data

# Run container with volume
docker run -d -v fichub-data:/data fichub

# Bind mount
docker run -d -v /host/path:/container/path fichub

# List volumes
docker volume ls

# Inspect volume
docker volume inspect fichub-data

# Remove volume
docker volume rm fichub-data
```

## Docker Security

```bash
# Scan image for vulnerabilities
docker scan fichub

# Using Trivy
trivy image fichub

# Check for secrets in image
docker history fichub --no-trunc

# Run with limited capabilities
docker run --cap-drop=ALL --cap-add=NET_BIND_SERVICE fichub

# Read-only filesystem
docker run --read-only --tmpfs /tmp fichub

# No new privileges
docker run --security-opt=no-new-privileges fichub
```

## Docker Compose Commands

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Rebuild images
docker-compose build --no-cache

# Scale service
docker-compose up -d --scale fichub=3

# Execute command
docker-compose exec fichub bash

# View service status
docker-compose ps
```

---

# Complete Redis Reference

## Data Structures

### Strings

```bash
SET key value
GET key
SET key value EX 3600  # Set with expiration
INCR key
DECR key
APPEND key value
STRLEN key
```

### Hashes

```bash
HSET key field value
HGET key field
HGETALL key
HDEL key field
HINCRBY key field increment
HEXISTS key field
HLEN key
```

### Lists

```bash
LPUSH key value
RPUSH key value
LPOP key
RPOP key
LRANGE key start stop
LLEN key
LINDEX key index
LREM key count value
```

### Sets

```bash
SADD key member
SREM key member
SMEMBERS key
SISMEMBER key member
SCARD key
SINTER key1 key2
SUNION key1 key2
SDIFF key1 key2
```

### Sorted Sets

```bash
ZADD key score member
ZRANGE key start stop
ZREVRANGE key start stop
ZRANGEBYSCORE key min max
ZREM key member
ZSCORE key member
ZRANK key member
ZCARD key
```

## Common Patterns

### Rate Limiting

```bash
# Sliding window rate limiting
MULTI
ZREMRANGEBYSCORE rate_limit:ip:192.168.1.1 0 <timestamp - window>
ZADD rate_limit:ip:192.168.1.1 <timestamp> <timestamp>
ZCARD rate_limit:ip:192.168.1.1
EXPIRE rate_limit:ip:192.168.1.1 <window>
EXEC
```

### Caching

```bash
# Cache with expiration
SET cache:story:abc123 '{"title": "My Story"}' EX 3600
GET cache:story:abc123

# Cache invalidation
DEL cache:story:abc123
```

### Session Storage

```bash
# Store session
HSET session:session_id user_id 123 expires_at 1640000000
EXPIRE session:session_id 3600

# Retrieve session
HGETALL session:session_id
```

## Lua Scripts

```lua
-- Token bucket rate limiting
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= 1 then
    tokens = tokens - 1
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

## Redis Configuration

```bash
# Max memory
maxmemory 256mb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
save 60 10000

# Security
requirepass your_password
rename-command FLUSHALL ""
rename-command FLUSHDB ""
```

## Redis Monitoring

```bash
# View info
INFO
INFO memory
INFO clients
INFO stats

# Monitor commands
MONITOR

# Slow log
SLOWLOG GET 10
SLOWLOG LEN
SLOWLOG RESET
```

