# Part 6: Server and Deployment

---

# Chapter 25: The Full Axum Router

This chapter brings together all the routes we've built and shows how FicHub's complete router is structured.

## Router Architecture

FicHub's router is organized into logical groups:

```
/api/                          — API documentation
/api/v0/                       — Versioned API endpoints
├── /epub                      — Export endpoints
├── /meta                      — Metadata endpoints
├── /remote                    — Client info endpoint
├── /search                    — Search endpoint
├── /recommendations           — Recommendation endpoints
├── /tags                      — Tag endpoints
└── /curator                   — Curator (admin) endpoints
/cache/{type}/{url_id}         — Cached file downloads
/opds/                         — OPDS catalog feeds
├── /new                       — Recent stories
├── /popular                   — Popular stories
├── /tags                      — Browse by tags
├── /authors                   — Browse by authors
├── /recommendations           — Recommendation feeds
├── /search                    — OPDS search
└── /shelves                   — User shelves
/                              — Frontend SPA (fallback)
```

## Route Definitions

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        // API routes
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        // Cache routes
        .route("/cache/{etype}/{url_id}/{fname}", 
            get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", 
            get(routes::cache_download::download_or_export))
        
        // Recommender routes
        .route("/api/v0/recommendations", 
            get(recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", 
            post(recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", 
            post(recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", 
            get(recommender::routes::votes_handler))
        
        // Tag routes
        .route("/api/v0/tags/submit", post(tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(tags::routes::flag_tag))
        .route("/api/v0/tags", get(tags::routes::get_tags))
        
        // Curator routes
        .route("/api/v0/curator/alias", post(tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", 
            delete(tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", 
            post(tags::curator::resolve_flag))
        
        // Search route
        .route("/api/v0/search", get(search::routes::search_handler))
        
        // OPDS routes
        .nest("/opds", opds_routes())
        
        // Legacy redirects
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        // Static frontend
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        // Shared state
        .with_state(state)
}

fn opds_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(routes::opds::feeds::root_catalog))
        .route("/new", get(routes::opds::feeds::recent_feed))
        .route("/popular", get(routes::opds::feeds::popular_feed))
        .route("/tags", get(routes::opds::tags::tag_types))
        .route("/tags/{type_id}", get(routes::opds::tags::tags_by_type))
        .route("/tags/{type_id}/{tag_name}", 
            get(routes::opds::tags::fics_by_tag))
        .route("/authors", get(routes::opds::authors::author_list))
        .route("/recommendations/popular", 
            get(routes::opds::recommendations::popular_recommendations))
        .route("/recommendations", 
            get(routes::opds::recommendations::fic_recommendations))
        .route("/search", get(routes::opds::search::search_feed))
        .route("/shelves", get(routes::opds::shelves::shelf_list))
        .route("/shelf/{shelf_id}", get(routes::opds::shelves::shelf_contents))
}
```

## Route Matching Order

Axum matches routes in the order they're defined. More specific routes should come before less specific ones:

```rust
// Good: specific before general
.route("/api/v0/tags/submit", post(submit_tag))
.route("/api/v0/tags", get(get_tags))

// Bad: general before specific
.route("/api/v0/tags", get(get_tags))
.route("/api/v0/tags/submit", post(submit_tag))  // Never reached!
```

## 📝 Practice Exercises

1. **Route Audit:** List all routes in FicHub and categorize them by HTTP method and feature area.

2. **Route Testing:** Write integration tests for each route group.

3. **API Versioning:** Design a `/api/v1/` version of the API with breaking changes.

---

# Chapter 26: Serving Static Files

FicHub serves the SvelteKit frontend as static files. This chapter covers how static file serving works with Axum.

## Static File Serving

```rust
use tower_http::services::{ServeDir, ServeFile};

// In the router:
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

### How It Works

1. **ServeDir** — Tries to serve a file from the directory matching the URL path
2. **append_index_html_on_directories** — If a directory is requested, serve `index.html` from it
3. **fallback** — If no file matches, serve `index.html` (for SPA routing)

### SPA Routing

Single-page applications (SPAs) like SvelteKit use client-side routing. When a user navigates to `/stories/123`, the browser requests that URL from the server. If the server doesn't have a file at that path, it should return `index.html` so the SPA can handle the route.

The `.fallback(ServeFile::new(...))` handles this — any URL that doesn't match a static file returns `index.html`.

### Cache Headers

For production, you should set cache headers for static assets:

```rust
use tower_http::set_header::SetResponseHeaderLayer;

let cache_headers = SetResponseHeaderLayer::overriding(
    http::header::CACHE_CONTROL,
    http::HeaderValue::from_static("public, max-age=31536000"),
);

let app = Router::new()
    .nest_service("/assets", get(serve_assets).layer(cache_headers))
    .fallback_service(ServeDir::new(&frontend_dir));
```

## 📝 Practice Exercises

1. **Cache Headers:** Add appropriate cache headers for different file types (HTML, CSS, JS, images).

2. **Compression:** Enable gzip compression for static files.

3. **Security Headers:** Add security headers (X-Content-Type-Options, X-Frame-Options) to static file responses.

---

# Chapter 27: Docker and Docker Compose

Docker makes deployment reproducible and consistent. This chapter covers FicHub's Docker setup.

## The Multi-Stage Dockerfile

```dockerfile
# Stage 1: Build
FROM rust:1.77-bookworm as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fichub .

EXPOSE 3000
CMD ["./fichub"]
```

### Why Multi-Stage?

The build stage includes the entire Rust toolchain (hundreds of MB). The runtime stage only includes the binary and necessary libraries. This reduces the final image size from ~2GB to ~50MB.

**Real-world analogy:** A multi-stage build is like cooking a meal. The first stage is the kitchen where you prepare everything (chopping, mixing, cooking). The second stage is the serving plate — you only put the finished dish on it, not all the pots, pans, and ingredients.

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
  
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
  
  calibre:
    image: lscr.io/linuxserver/calibre-web:latest
    volumes:
      - calibre-config:/config
      - calibre-books:/books
    ports:
      - "8083:8083"
  
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
      - postgres
      - redis

volumes:
  pgdata:
  calibre-config:
  calibre-books:
  fichub-cache:
  fichub-tmp:
```

### Docker Networking

Docker Compose creates a network for all services. Services can communicate using their service names:
- `postgres` → PostgreSQL database
- `redis` → Redis server
- `calibre` → Calibre ebook management
- `fichub` → The FicHub application

The `depends_on` directive ensures PostgreSQL and Redis start before FicHub.

## 📝 Practice Exercises

1. **Docker Build:** Build the FicHub Docker image and verify it starts correctly.

2. **Health Checks:** Add health checks to each Docker service.

3. **Logging:** Configure Docker logging for all services.

---

# Chapter 28: Cross-Compilation and Deploy

FicHub runs on a remote server. This chapter covers cross-compilation for ARM64 and deployment.

## Cross-Compilation

```bash
# Install the ARM64 target
rustup target add aarch64-unknown-linux-gnu

# Install the cross-compilation linker
sudo pacman -S aarch64-linux-gnu-gcc

# Create a cargo config for cross-compilation
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-gnu
```

### Why Cross-Compile?

The development machine is typically x86_64 (Intel/AMD), but the deployment target might be ARM64 (Raspberry Pi, AWS Graviton). Cross-compilation lets you build for the target platform without needing the target hardware.

**Real-world analogy:** Cross-compilation is like having a translator at a business meeting. You speak English (x86_64), your partner speaks Japanese (ARM64), and the translator converts your words in real-time. You don't need to learn Japanese — the translator handles it.

### Deployment

```bash
# Copy the binary to the server
scp target/aarch64-unknown-linux-gnu/release/fichub user@server:/opt/fichub/

# SSH into the server
ssh user@server

# Run the binary
cd /opt/fichub
./fichub
```

### Systemd Service

For production, run FicHub as a systemd service:

```ini
[Unit]
Description=FicHub Backend
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/opt/fichub
EnvironmentFile=/opt/fichub/.env
ExecStart=/opt/fichub/fichub
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## 📝 Practice Exercises

1. **Cross-Compile:** Cross-compile FicHub for ARM64 and verify the binary runs on an ARM64 system.

2. **Systemd:** Create a systemd service file and verify FicHub starts automatically on boot.

3. **Monitoring:** Set up basic monitoring (CPU, memory, disk usage) for the deployed FicHub instance.

