# Part 6: Server Deployment

---

# Chapter 25: Full Router

## Every Route in FicHub

FicHub has 40+ routes organized by subsystem. This chapter catalogs every route, explains its purpose, and shows the handler signature.

## Core API Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/` | `api_docs_handler` | API documentation |
| GET | `/api/v0/epub` | `epub_handler` | Main export endpoint |
| GET | `/api/v0/meta` | `meta_handler` | Metadata-only endpoint |
| GET | `/api/v0/remote` | `remote_handler` | Client IP info |

### `/api/` — API Documentation

Returns a JSON object describing all available endpoints:

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links"
            },
            "/api/v0/meta": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata only"
            },
            "/api/v0/remote": {
                "method": "GET",
                "description": "Get request source information"
            },
            "/cache/:etype/:url_id": {
                "method": "GET",
                "params": { "h": "MD5 hash for validation" },
                "description": "Download cached export file"
            }
        }
    }))
}
```

### `/api/v0/epub` — Main Export

The most important endpoint. Accepts a fanfiction URL and returns metadata plus download links:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // 17-step export flow (see Chapter 20)
}
```

### `/api/v0/meta` — Metadata Only

Same as `/api/v0/epub` but without generating or serving files:

```rust
#[derive(Debug, Deserialize)]
pub struct MetaQuery {
    pub q: Option<String>,
}

pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "url_id": meta.url_id,
        "slug": generate_slug(&meta.title, &meta.url_id),
        "meta": build_meta_json(&meta),
        "hashes": {},
        "urls": {},
    })))
}
```

### `/api/v0/remote` — Client IP Info

Returns the client's IP address:

```rust
async fn remote_handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<serde_json::Value> {
    Json(json!({
        "ip": remote.ip().to_string(),
        "port": remote.port(),
        "is_automated": false,
    }))
}
```

## Cache Download Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/cache/{etype}/{url_id}/{fname}` | `download_with_hash` | Download with hash validation |
| GET | `/cache/{etype}/{url_id}` | `download_or_export` | Download or trigger export |

### Download with Hash Validation

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    let hash = match params.h {
        Some(h) => h,
        None => {
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash,
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
            };

            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

### Download or Export

```rust
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // If hash provided and file exists, serve directly
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash,
            );
            if cache_path.exists() {
                // ... serve file ...
            }
        }
    }

    // No cached file — redirect to frontend
    Redirect::to(&format!("/?id={}", url_id)).into_response()
}
```

## Recommender Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/v0/recommendations` | `recommendations_handler` | Get recommendations |
| POST | `/api/v0/recommendations/suggest` | `suggest_handler` | Submit a suggestion |
| POST | `/api/v0/recommendations/vote` | `vote_handler` | Vote on a suggestion |
| GET | `/api/v0/recommendations/votes` | `votes_handler` | List votes |

### Recommendations Handler

```rust
pub async fn recommendations_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecQueryParams>,
) -> Result<Json<Value>, AppError> {
    let (url_id, _seed_meta) = if let Some(q) = &params.q {
        let scraper = state.scraper_registry.find_scraper(q)
            .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", q)))?;
        let meta = scraper.lookup(&state.http_client, q).await
            .map_err(|e| AppError::ScrapeError(e.to_string()))?;
        (meta.url_id, Some(meta))
    } else if let Some(id) = &params.url_id {
        (id.clone(), None)
    } else {
        return Ok(Json(json!({"err": -1, "msg": "no query or url_id"})));
    };

    let n = params.n.unwrap_or(20).max(1).min(100) as usize;

    let rec_query = RecQuery {
        url_id: url_id.clone(),
        n,
        site_domain: params.site_domain.clone(),
    };

    let recommendations = state.recommender_engine
        .get_recommendations(&rec_query, &state.config).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "recommendations": recommendations,
    })))
}
```

## Tag Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/v0/tags/submit` | `submit_tag` | Submit a tag |
| POST | `/api/v0/tags/vote` | `vote_tag` | Vote on a tag |
| POST | `/api/v0/tags/flag` | `flag_tag` | Flag a tag |
| GET | `/api/v0/tags` | `get_tags` | Get tags for a fic |

### Submit Tag

```rust
pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    // Validate
    if body.url_id.is_empty() || body.tag_name.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id and tag_name required"})));
    }
    if !(1..=7).contains(&body.tag_type_id) {
        return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be 1-7"})));
    }

    // Rate limit
    check_tag_rate_limit(&mut redis, "submit", ip, state.config.tag_submit_limit_per_hour).await?;

    // Verify fic exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)"
    ).bind(&body.url_id).fetch_one(&state.db).await?;

    if !exists {
        return Ok(Json(json!({"err": -5, "msg": "fic not found"})));
    }

    // Resolve tag
    let resolution = resolve::resolve_tag(&state.db, &body.tag_name, body.tag_type_id).await?;

    // Attach tag (idempotent)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, 0)
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    ).bind(&body.url_id).bind(resolution.tag_id).bind(&ip.to_string())
     .execute(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "tag_id": resolution.tag_id,
        "tag_name": resolution.tag_name,
        "is_new": resolution.is_new,
    })))
}
```

## Curator Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/v0/curator/alias` | `create_alias` | Create a tag alias |
| POST | `/api/v0/curator/merge` | `merge_tags` | Merge two tags |
| DELETE | `/api/v0/curator/tags/{id}` | `delete_tag` | Delete a tag |
| GET | `/api/v0/curator/flags` | `list_flags` | List unresolved flags |
| POST | `/api/v0/curator/flags/{id}/resolve` | `resolve_flag` | Resolve a flag |

All curator endpoints require Bearer token authentication.

## Search Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/v0/search` | `search_handler` | Full-text search |

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let search_params = params.into_search_params()?;
    let builder = SearchQueryBuilder::new(capped_params, state.config.tag_hidden_threshold);

    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query.build_query_as().fetch_one(&state.db).await?;

    let mut data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query.build_query_as().fetch_all(&state.db).await?;

    // Batch-fetch tags
    let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let tag_rows = sqlx::query_as::<_, TagDbRow>(
        "SELECT ft.url_id, t.name, t.tag_type_id, tt.name AS type_name, ft.score
         FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id JOIN tag_types tt ON tt.id = t.tag_type_id
         WHERE ft.url_id = ANY($1) AND ft.score >= $2"
    ).bind(&url_ids).bind(state.config.tag_hidden_threshold)
     .fetch_all(&state.db).await?;

    // Build response
    Ok(Json(json!({
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": results,
    })))
}
```

## OPDS Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/opds` | `root_catalog` | Root catalog |
| GET | `/opds/new` | `recent_feed` | Recent fics |
| GET | `/opds/popular` | `popular_feed` | Popular fics |
| GET | `/opds/tags` | `tag_types` | Browse by tag type |
| GET | `/opds/tags/{type_id}` | `tags_by_type` | Tags in a type |
| GET | `/opds/tags/{type_id}/{tag_name}` | `fics_by_tag` | Fics with a tag |
| GET | `/opds/authors` | `author_list` | Browse by author |
| GET | `/opds/recommendations/popular` | `popular_recommendations` | Popular recs |
| GET | `/opds/recommendations` | `fic_recommendations` | Fic-specific recs |
| GET | `/opds/search` | `search_feed` | Search feed |
| GET | `/opds/shelves` | `shelf_list` | List shelves |
| GET | `/opds/shelf/{shelf_id}` | `shelf_contents` | Shelf contents |

All OPDS routes return Atom XML feeds for e-readers.

## Legacy Redirects

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/legacy/epub_export` | `redirect_to_root` | Redirect to / |
| GET | `/fic/{url_id}` | `redirect_to_root` | Redirect to / |
| GET | `/changes` | `redirect_to_root` | Redirect to / |
| GET | `/popular/` | `redirect_to_root` | Redirect to / |

## AppState

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

Every handler receives `State(state): State<Arc<AppState>>` and accesses shared resources through it.

## Summary

FicHub's router is well-organized with 40+ routes across 8 subsystems. The AppState struct centralizes shared state, and each handler follows a consistent pattern of extract → validate → process → respond.

---

# Chapter 26: Static Files

## Serving the Frontend

FicHub's frontend is a SvelteKit application built into static files. These are served by tower-http's `ServeDir`:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

## How ServeDir Works

`ServeDir` serves files from a directory based on the request path:

1. Request for `/` → serves `/index.html`
2. Request for `/assets/main.js` → serves the JS file
3. Request for `/unknown/path` → falls through to fallback (index.html)

## SPA Routing

SvelteKit builds to static files:

```
frontend/build/
├── index.html
├── _app/
│   └── immutable/
│       ├── assets/
│       │   ├── main-abc123.js
│       │   └── main-def456.css
│       └── chunks/
│           └── 0-ghi789.js
└── favicon.png
```

When a user visits `/fic/abc123`:
1. ServeDir looks for `frontend/build/fic/abc123` — not found
2. Falls through to `frontend/build/index.html`
3. SvelteKit boots and handles the route client-side

## Cache Headers

```rust
ServeDir::new(&frontend_dir)
    .precompressed_br()
    .precompressed_gzip()
    .append_index_html_on_directories(true)
```

For production, add cache headers via middleware:

```rust
.layer(CacheControl::new()
    .set_max_age(Duration::from_secs(3600))
    .set_public())
```

## MIME Types

| Extension | MIME Type |
|-----------|-----------|
| `.html` | `text/html` |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff2` | `font/woff2` |

## Watch Out!

**Order matters!** The fallback must come after all API routes.

**Asset paths must match!** The build output paths must match what the HTML expects.

## Summary

FicHub uses `ServeDir` with SPA fallback to serve the SvelteKit frontend. The configuration ensures proper routing and fallback behavior.

---

# Chapter 27: Docker

## Multi-Stage Build

```dockerfile
FROM rust:1.78-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/fichub .
COPY --from=builder /app/migrations migrations/
EXPOSE 3000
CMD ["./fichub"]
```

Stage 1 compiles, Stage 2 runs. The runtime image is ~50-100MB vs ~1GB+ for build.

## Docker Compose

```yaml
version: '3.8'
services:
  fichub:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://fichub:fichub@db/fichub
      - REDIS_URL=redis://redis:6379
      - CACHE_DIR=/data/cache
    volumes:
      - cache_data:/data/cache
    depends_on:
      - db
      - redis

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=fichub
      - POSTGRES_PASSWORD=fichub
      - POSTGRES_DB=fichub
    volumes:
      - pg_data:/var/lib/postgres/data

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  cache_data:
  pg_data:
  redis_data:
```

## Building and Running

```bash
docker compose build
docker compose up -d
docker compose logs -f fichub
docker compose down
```

## Environment Variables

Use a `.env` file:

```bash
DATABASE_URL=postgres://fichub:fichub@db/fichub
REDIS_URL=redis://redis:6379
CACHE_DIR=/data/cache
PORT=3000
RUST_LOG=info,fichub=debug
```

## Watch Out!

**Database readiness!** Make sure PostgreSQL is ready before FicHub starts. Use health checks or retry logic.

**Volume permissions!** The container runs as root. Ensure host volume permissions are correct.

**Redis persistence!** Without the `redis_data` volume, Redis data is lost on container stop.

## Summary

FicHub's Docker setup uses multi-stage builds, Docker Compose for orchestration, and volumes for persistence.

---

# Chapter 28: Cross-Compile

## Setting Up

```bash
rustup target add aarch64-unknown-linux-gnu
sudo pacman -S aarch64-linux-gnu-gcc  # Arch
# or: sudo apt install gcc-aarch64-linux-gnu  # Ubuntu
```

Create `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Building

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

## Deploying

```bash
rsync -avz target/aarch64-unknown-linux-gnu/release/fichub user@server:~/fichub/
rsync -avz migrations/ user@server:~/fichub/migrations/
rsync -avz frontend/build/ user@server:~/fichub/frontend/build/
```

## Systemd Service

```ini
[Unit]
Description=FicHub Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/home/fichub
Environment=DATABASE_URL=postgres://fichub:fichub@localhost/fichub
Environment=REDIS_URL=redis://localhost:6379
Environment=CACHE_DIR=/home/fichub/cache
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Nginx Reverse Proxy

```nginx
server {
    listen 80;
    server_name fichub.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /assets/ {
        proxy_pass http://127.0.0.1:3000;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    location /cache/ {
        proxy_pass http://127.0.0.1:3000;
        expires 30d;
        add_header Cache-Control "public";
    }
}
```

## TLS with Let's Encrypt

```bash
sudo certbot --nginx -d fichub.example.com
```

## Watch Out!

**Cross-compilation requires matching libraries!** FicHub uses `rustls` to avoid OpenSSL dependency issues.

**Test on the target device!** Always verify the binary works on the actual hardware.

## Summary

Cross-compilation enables deployment on ARM64 devices. Combined with rsync, systemd, and nginx, FicHub can run on anything from a Raspberry Pi to a cloud server.
