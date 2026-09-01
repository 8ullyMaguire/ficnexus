# Part 6: Server and Deployment

---

# Chapter 25: The Full Axum Router

We've spent five parts of this book building the individual pieces of FicHub — scrapers, exporters, caches, a recommendation engine, a tagging system. Now it's time to stitch them all together. The place where everything connects is the Axum router, and it lives in a single file: `src/server.rs`.

If you open that file, you'll see a `build_router` function that returns a `Router`. That's it. That's the heart of the application. Every HTTP request that comes in gets routed through this tree, matched to a handler, and processed with shared application state.

Let's walk through every piece.

## AppState: The Backpack of Shared Resources

Before we look at routes, we need to understand what `AppState` is. In Axum, you can pass a piece of state to all your handlers through a technique called "shared state." FicHub wraps everything it needs in an `Arc<AppState>` and hands it to every route.

Here's the struct straight from the source:

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

Each field deserves a moment:

- **`config`** — The `Config` struct loaded from environment variables. It has everything: database URLs, Redis URLs, cache directories, port numbers, rate limit settings, recommender parameters, tagging thresholds. We looked at this in detail in earlier chapters, and it's all pulled from the environment in `Config::from_env()`. The config contains over 30 configuration options, from the core `DATABASE_URL` and `REDIS_URL` to fine-tuned recommender parameters like `REC_VOTING_BOOST_GAMMA` and `REC_CACHE_TTL_HOURS`. Every one of these can be overridden by setting the corresponding environment variable, making FicHub configurable without any code changes.

- **`db`** — A SQLx connection pool for PostgreSQL. This is the persistent storage layer where fic metadata, export logs, recommendations, tags, and all the other data lives. SQLx manages the pool automatically — it creates connections as needed, reuses them across requests, and handles connection timeouts and reconnection. The pool size is tunable, but the defaults work well for most deployments.

- **`redis`** — A multiplexed async Redis connection. Redis serves double duty here: it's used for the token-bucket rate limiter and for caching recommendation results. The "multiplexed" part means multiple logical streams share a single TCP connection — more efficient than opening a new connection for every request.

- **`http_client`** — A `reqwest::Client` with a custom user agent (`fichub.net/0.1.0`) and a 30-second timeout. This is what the scrapers use to fetch pages from AO3, FanFiction.net, and all the other supported sites. The 30-second timeout is generous enough for slow sites but prevents the server from hanging indefinitely on unresponsive upstreams.

- **`scraper_registry`** — An `Arc<ScraperRegistry>` that holds all the registered scrapers (AO3, FFN, FictionPress, HPFanFicArchive, AdultFanFiction, XenForo forums, etc.). When a URL comes in, the registry finds the right scraper for it by matching the URL pattern. The registry is created once at startup and shared across all requests — no need to recreate it for every request.

- **`cache_semaphores`** — A `HashMap` behind a `Mutex` that holds per-fic export semaphores. These prevent the same fic from being exported ten times concurrently — if two people request the same URL at the same time, only one export runs and the other waits for the result. This is a classic "thundering herd" prevention pattern. Without it, if a popular fic gets ten simultaneous requests, you'd make ten identical requests to AO3, generate ten identical EPUBs, and waste a lot of time and bandwidth.

- **`rate_limiter`** — A trait object for the token-bucket rate limiter backed by Redis. It limits how many requests each IP can make. The rate limiter uses a Redis-backed token bucket algorithm, which is distributed across all instances of the application. It can also load datacenter IP ranges from external sources and apply different rate limits to datacenter IPs versus residential IPs.

- **`recommender_engine`** — The recommendation engine we built in Part 5. It computes recommendations based on collaboratively-filtered favourites. The engine is stateless — it takes a database pool reference and computes recommendations on the fly — but having it in the shared state avoids recreating it per request.

- **`collection_worker`** — A background worker that can fetch and process fics from a collection or author page. When someone submits a collection URL (like an AO3 collection or an author's entire works page), the worker processes it in the background, fetching metadata and generating exports for each fic in the collection.

All of this gets created in the `run()` function, wrapped in `Arc::new()`, and passed to the router. The `Arc` (Atomic Reference Counted) pointer allows multiple handlers to share ownership of the same state without violating Rust's ownership rules. Since `Arc` uses atomic operations for reference counting, it's safe to share across async tasks.

## The run() Function: Bringing It to Life

The `run()` function is the entry point for the server. Here's what it does, step by step:

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // Load datacenter IPs if configured
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // Create shared state
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new()
        )),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });

    // Build router
    let app = build_router(state).await;

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
        .await
        .expect("Server error");
}
```

First, it connects to PostgreSQL using SQLx's `init_pool`, which runs any pending migrations and creates a connection pool. The `init_pool` function is smart — it checks the database schema version and applies any SQL migration files that haven't been run yet. This means you can just point FicHub at an empty database and it'll set itself up.

Then it connects to Redis, builds the HTTP client, initializes the scraper registry and rate limiter, loads datacenter IP lists if configured, creates the recommendation engine and collection worker, and wraps everything in `AppState`.

Notice the use of `.clone()` throughout. When building the state, we need to clone the database pool, Redis connection, HTTP client, and scraper registry because they're all shared. Cloning a connection pool doesn't create a new pool — it just creates another handle to the same pool. This is the beauty of Rust's `Arc` and reference-counted types.

The rate limiter initialization is particularly interesting. It creates a new `RedisBucketLimiter` with the Redis connection and the `dynamic_rate_limit` config flag. If dynamic rate limiting is enabled, the limiter adjusts its thresholds based on server load. The `load_datacenter_ips` call fetches IP ranges from external sources (like Cloudflare's published ranges) so the limiter can apply different limits to datacenter IPs versus residential users.

Then it builds the router and binds to a TCP listener:

```rust
let addr = format!("0.0.0.0:{}", config.app_port);
let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .expect("Failed to bind to address");

axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
)
.await
.expect("Server error");
```

Notice the `into_make_service_with_connect_info::<SocketAddr>()` call. This tells Axum to extract the client's IP address and make it available through the `ConnectInfo` extractor. That's how the `/api/v0/remote` endpoint knows your IP address. Without this call, the `ConnectInfo` extractor would panic at runtime.

The server binds to `0.0.0.0:{port}`, which means it listens on all network interfaces. By default, the port is 3000 (set via the `PORT` environment variable). Binding to `0.0.0.0` instead of `127.0.0.1` means the server is accessible from other machines on the network, not just localhost. This is necessary for Docker (where the container needs to be reachable from outside) and for bare-metal deployment (where other machines need to reach the server).

If the port is already in use, `TcpListener::bind` will fail with an error. The `.expect("Failed to bind to address")` call will panic with a clear message. In production, you'd want to handle this more gracefully — perhaps logging the error and suggesting a different port.

## The Router Tree: Every Route Explained

Now let's look at the actual router. The `build_router` function constructs it by chaining route definitions. Here's the complete tree, explained route by route.

### API Documentation

```rust
.route("/api/", get(routes::api_docs::api_docs_handler))
```

The root `/api/` endpoint returns a JSON document describing all available API endpoints. It's like a built-in Swagger page, except it's just hand-written JSON. If you curl `http://localhost:3000/api/`, you'll get back a list of endpoints with their methods, parameters, and descriptions:

```json
{
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
}
```

This is handy for debugging and for anyone who wants to integrate with the API. It doesn't cover every endpoint (the OPDS, recommendation, and tag routes aren't listed), but it covers the core ones that most people use.

### Core Export Endpoint

```rust
.route("/api/v0/epub", get(routes::export::epub_handler))
```

This is the main workhorse endpoint. It takes a query parameter `q` containing a fanfiction URL, looks up metadata, checks blacklists, generates EPUB and HTML files (or serves them from cache), and returns JSON with metadata, download URLs, and hashes.

The handler signature is:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError>
```

The `State(state)` extractor pulls the shared `AppState` from the router. The `Query(params)` extractor deserializes the query string into the `ExportQuery` struct. And the return type `Result<Json<Value>, AppError>` means the handler either returns a JSON response or an error.

The flow inside `epub_handler` is:

1. **Parse the `q` parameter** — If it's empty, return an error with code `-1` and the message "no query".

2. **Block automated requests** — If the `automated=true` parameter is present, return an error with code `-10` and the message "automated requests blocked". This prevents bots from hammering the service.

3. **Find the right scraper** — Call `state.scraper_registry.find_scraper(query)` to match the URL to a scraper (AO3, FFN, etc.). If no scraper matches, return error `-5` with "unsupported URL".

4. **Look up metadata** — Call `scraper.lookup(&state.http_client, query)` to fetch the fic's title, author, word count, chapter count, description, and other metadata from the upstream site.

5. **Upsert fic info** — Call `queries::upsert_fic_info` to store or update the fic's metadata in PostgreSQL. This uses an "upsert" (INSERT ... ON CONFLICT DO UPDATE) so it works whether the fic is new or already exists.

6. **Auto-populate tags** — Call `scraper.extract_tags` to pull genre/rating/relationship tags from the upstream site, then resolve and store them in the tagging system.

7. **Check blacklists** — Query the database for fic and author blacklists. If the fic is greylisted (reason 6), return metadata but no download links. If it's hard-blacklisted (reason 5, 7, or 8), return error `-7` with "fic is blacklisted".

8. **Check cache** — Look up the export log in the database. If a cached EPUB exists for the current version and content hash, build download URLs from the cached hash and return them immediately.

9. **If cache miss** — Acquire the export semaphore (preventing concurrent duplicate exports), recheck the cache (double-check pattern — another concurrent request might have generated it while we were waiting), fetch chapters, generate the EPUB, move it to the cache directory, record it in the export log, generate the HTML bundle, record that too, and return the URLs.

10. **Log the request** — Insert a record into `request_log` with timing information, the fic metadata, and the export hash.

The handler returns JSON shaped like:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345",
    "fixits": [],
    "info": "The Story by Author\n50000 words in 10 chapters\nStatus: complete\nUpdated: 2024-01-15 12:30:00 - 30 days ago\n",
    "url_id": "abc123",
    "slug": "The_Story-abc123",
    "meta": {
        "id": "abc123",
        "title": "The Story",
        "author": "Author",
        "chapters": 10,
        "words": 50000,
        "description": "<p>A great story</p>",
        "status": "complete",
        "source": "https://archiveofourown.org/works/12345",
        "created": "2023-12-01T00:00:00Z",
        "updated": "2024-01-15T12:30:00Z"
    },
    "hashes": {
        "epub": "a1b2c3d4e5f6",
        "html": "f6e5d4c3b2a1"
    },
    "urls": {
        "epub": "/cache/epub/abc123?h=a1b2c3d4e5f6",
        "html": "/cache/html/abc123?h=f6e5d4c3b2a1"
    },
    "epub_url": "/cache/epub/abc123?h=a1b2c3d4e5f6",
    "html_url": "/cache/html/abc123?h=f6e5d4c3b2a1",
    "mobi_url": null,
    "pdf_url": null,
    "notes": []
}
```

The error code (`err`) tells the caller what happened: `0` means success, negative numbers mean various errors. The `fixits` array is reserved for future use (potential corrections to the fic). The `info` string is a human-readable summary. The `slug` is a URL-safe version of the title combined with the URL ID.

### Metadata-Only Endpoint

```rust
.route("/api/v0/meta", get(routes::meta::meta_handler))
```

This endpoint is like the epub handler but lighter. It takes the same `q` parameter, finds the scraper, looks up metadata, and returns it without generating any files. This is useful when you just want to display fic information without waiting for an export to complete — for example, when the user is browsing and wants to see a preview before deciding to download.

The handler does the same scraper lookup, blacklist check, and slug generation, but skips all the caching and export logic. The response has the same shape as the epub endpoint, but `hashes`, `urls`, and `epub_url` are all empty/null:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345",
    "info": "The Story by Author - 50000 words, 10 chapters",
    "url_id": "abc123",
    "slug": "The_Story-abc123",
    "meta": { ... },
    "hashes": {},
    "urls": {},
    "epub_url": null,
    "html_url": null,
    "mobi_url": null,
    "pdf_url": null,
    "notes": []
}
```

The `info` format is slightly different here — it's a single-line summary instead of the multi-line format used in the epub handler.

### Remote Info

```rust
.route("/api/v0/remote", get(remote_handler))
```

A simple diagnostic endpoint that returns your IP address and port:

```rust
async fn remote_handler(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Json<serde_json::Value> {
    Json(json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

This uses the `ConnectInfo` extractor, which only works because the server was set up with `into_make_service_with_connect_info::<std::net::SocketAddr>()`. If you forget that call in `run()`, this handler will panic.

The `is_automated` field is always `false` in the response — it's a field the client can set in requests to indicate automated access, and this endpoint echoes back the default. It's useful for debugging proxy configurations, verifying that the client IP is being correctly extracted, and testing whether Docker networking is set up properly.

### Cache Download Routes

```rust
.route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
.route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
```

These routes serve cached export files directly. There are two variants, and understanding the difference is important.

**`/cache/{etype}/{url_id}/{fname}`** — The direct download route. The `fname` parameter includes the hash embedded in the filename (like `a1b2c3d4e5f6.epub`). The handler extracts the hash, validates it against the file's actual MD5 hash, and serves the file with the correct MIME type and Content-Disposition header.

The validation step is crucial. Without it, someone could request `/cache/epub/abc123/anything.epub` and potentially read arbitrary files. By requiring a valid MD5 hash, the server ensures that only correctly-hashed filenames can access cached files.

The MIME types are determined by the `etype` parameter:

| Format | Suffix | MIME Type |
|--------|--------|-----------|
| epub | `.epub` | `application/epub+zip` |
| html | `.zip` | `application/zip` |
| mobi | `.mobi` | `application/x-mobipocket-ebook` |
| pdf | `.pdf` | `application/pdf` |

**`/cache/{etype}/{url_id}`** — The smart download route. This is what the API returns in its `urls` field. If a hash is provided via query parameter (`?h=a1b2c3d4e5f6`) and the cached file exists and validates, it's served directly. Otherwise, the handler redirects to `/?id={url_id}`, which opens the SvelteKit frontend with that fic pre-loaded — the frontend then triggers the export via the API.

This redirect behavior is smart: if the cache has been evicted or the server was restarted, the user doesn't get a confusing error. Instead, they're gently redirected to the web UI, which will re-export the fic on demand.

### Recommendation Routes

```rust
.route("/api/v0/recommendations",
    get(crate::recommender::routes::recommendations_handler))
.route("/api/v0/recommendations/suggest",
    post(crate::recommender::routes::suggest_handler))
.route("/api/v0/recommendations/vote",
    post(crate::recommender::routes::vote_handler))
.route("/api/v0/recommendations/votes",
    get(crate::recommender::routes::votes_handler))
```

These four routes power the recommendation engine:

- **GET `/api/v0/recommendations`** — Fetch recommendations for a given fic or author. The engine uses collaboratively-filtered favourites to find similar stories. Pass a `url_id` or `author` parameter to get recommendations. The response includes a list of recommended fics with similarity scores.

- **POST `/api/v0/recommendations/suggest`** — Submit a new recommendation (suggest that fic A is similar to fic B). This is an authenticated endpoint that requires a user identifier. Suggestions feed into the collaborative filtering algorithm — if enough users suggest that two fics are similar, the system learns to recommend them together.

- **POST `/api/v0/recommendations/vote`** — Vote on an existing recommendation (agree or disagree). Voting strengthens or weakens the recommendation connection between two fics. The voting system uses a gamma parameter (`REC_VOTING_BOOST_GAMMA`) to control how much votes affect recommendation scores.

- **GET `/api/v0/recommendations/votes`** — Fetch vote counts for recommendations on a given fic. This lets the UI display how many people agree or disagree with each recommendation.

### Tag Routes

```rust
.route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
.route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
.route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
.route("/api/v0/tags", get(crate::tags::routes::get_tags))
```

The tagging system (v3) is a community-driven categorization system:

- **POST `/api/v0/tags/submit`** — Submit a new tag for a fic. Tags go through a resolution process that handles duplicates and synonyms — if someone submits "Romance" and another person submits "romance", they're merged.

- **POST `/api/v0/tags/vote`** — Vote on an existing tag (upvote or downvote). Tags with votes below `TAG_HIDDEN_THRESHOLD` (default -3) are hidden from the public view. This is the community's way of curating tag quality.

- **POST `/api/v0/tags/flag`** — Flag a tag for moderator review. If someone sees a tag that's inappropriate, misleading, or spam, they can flag it for the curators to handle.

- **GET `/api/v0/tags`** — Fetch existing tags for a fic. Returns all tags with their vote counts and types.

All tag endpoints are rate-limited: `TAG_SUBMIT_LIMIT_PER_HOUR` (default 10) for submissions and `TAG_VOTE_LIMIT_PER_HOUR` (default 20) for votes.

### Curator Routes

```rust
.route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
.route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
.route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
.route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
.route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
```

These routes are behind a curator token (set via `CURATOR_TOKEN` environment variable). They provide moderation tools:

- **POST `/api/v0/curator/alias`** — Create a tag alias. For example, making "fluff" an alias of "Humor/Fluff" ensures that both terms map to the same tag.

- **POST `/api/v0/curator/merge`** — Merge two tags into one. All fics tagged with the source tag get re-tagged with the destination tag, and the source tag is deleted.

- **DELETE `/api/v0/curator/tags/{id}`** — Delete a tag entirely. This removes it from all fics.

- **GET `/api/v0/curator/flags`** — List all flagged tags that need moderator attention. This is the curator's dashboard.

- **POST `/api/v0/curator/flags/{id}/resolve`** — Resolve a flag (mark it as handled). The curator can choose to dismiss the flag (tag is fine) or act on it (delete the tag, merge it, etc.).

The curator token is checked in the handler code, not in middleware. This means if someone sends a request without the token, they get a 403 Forbidden response, not a 401 Unauthorized (which would suggest they need to authenticate differently).

### Search Route

```rust
.route("/api/v0/search", get(crate::search::routes::search_handler))
```

The advanced search endpoint lets users search across the fic database by title, author, tags, word count, status, and other criteria. It returns paginated results and uses the `SEARCH_MAX_PER_PAGE` config option to limit results (default 50).

The search is powered by SQLx queries against PostgreSQL. For simple title searches, it uses `ILIKE` (case-insensitive LIKE). For more complex queries involving multiple criteria, it builds dynamic SQL with appropriate parameter binding to prevent SQL injection.

### OPDS Catalog Routes

```rust
.route("/opds", get(crate::routes::opds::feeds::root_catalog))
.route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
.route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
.route("/opds/tags", get(crate::routes::opds::tags::tag_types))
.route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
.route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
.route("/opds/authors", get(crate::routes::opds::authors::author_list))
.route("/opds/recommendations/popular",
    get(crate::routes::opds::recommendations::popular_recommendations))
.route("/opds/recommendations",
    get(crate::routes::opds::recommendations::fic_recommendations))
.route("/opds/search", get(crate::routes::opds::search::search_feed))
.route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
.route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
```

OPDS (Open Publication Distribution System) is a standard that e-readers like KOReader, Calibre, Thorium Reader, and Moon+ Reader use to browse and download books. FicHub implements a full OPDS catalog, so you can point your e-reader at `http://your-server:3000/opds` and browse fics right from your device — no app installation needed.

The routes cover every aspect of a book catalog:

- **Root catalog** — The top-level navigation feed. When you first point your e-reader at FicHub, this is what it loads. It contains links to all the sub-feeds.

- **New fics** (`/opds/new`) — Recently added stories, sorted by when they were last updated. Great for finding new things to read.

- **Popular fics** (`/opds/popular`) — Stories ranked by popularity. The popularity score considers word count, chapter count, and how many times the fic has been downloaded.

- **Tags** — A three-level hierarchy. `/opds/tags` lists tag types (genre, rating, relationship, etc.). `/opds/tags/{type_id}` lists tags of that type. `/opds/tags/{type_id}/{tag_name}` lists fics with that specific tag.

- **Authors** (`/opds/authors`) — Browse by author. The feed lists all authors who have fics in the system, with links to each author's fics.

- **Recommendations** — Both popular recommendations (`/opds/recommendations/popular`) and per-fic recommendations (`/opds/recommendations?url_id=abc123`). This brings the recommendation engine to your e-reader.

- **Search** (`/opds/search?q=keyword`) — Full-text search via the OPDS protocol. Your e-reader's built-in search will use this endpoint.

- **Shelves** — User-created reading lists, authenticated with `OPDS_SHELF_TOKEN`. `/opds/shelves` lists all shelves, and `/opds/shelf/{shelf_id}` shows the fics on a specific shelf.

This is one of FicHub's killer features — it turns your fanfiction library into a personal OPDS server. You can browse, search, and download fics from any OPDS-compatible reader.

### Legacy Redirect Routes

```rust
.route("/legacy/epub_export", get(redirect_to_root))
.route("/fic/{url_id}", get(redirect_to_root))
.route("/changes", get(redirect_to_root))
.route("/popular/", get(redirect_to_root))
```

These four routes handle old URLs from the Python-based fichub.net. If someone has a bookmark from the old site — `https://fichub.net/fic/abc123` or `https://fichub.net/changes` — they'll get redirected to the root page where the SvelteKit frontend can handle it.

The redirect handler is defined inside `build_router` as a simple async function:

```rust
async fn redirect_to_root() -> Redirect {
    Redirect::to("/")
}
```

It's defined inside the function rather than as a closure because Axum's routing works better with named functions. Closures with async blocks can cause lifetime issues in the router construction.

This is a nice touch for backward compatibility. Old links don't break — they just land on the modern UI, where the frontend can handle the URL.

### The Fallback: Serving the Frontend

After all the API routes, there's a crucial piece:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This is how FicHub serves its SvelteKit frontend. The `fallback_service` is called last, so it only handles requests that didn't match any of the explicit routes above. Let's break it down:

1. **`ServeDir::new(&frontend_dir)`** — Serve static files from the frontend build directory (default: `./frontend/build`). When someone requests `/style.css`, it serves `./frontend/build/style.css`. When someone requests `/app.js`, it serves `./frontend/build/app.js`. The directory path comes from `state.config.frontend_dir`.

2. **`append_index_html_on_directories(true)`** — If someone requests `/` or `/about/`, serve the `index.html` from that directory. This handles the basic SPA routing case where the user navigates to a directory-like path.

3. **`fallback(ServeFile::new(frontend_dir.join("index.html")))`** — If `ServeDir` can't find a matching file, serve `index.html` anyway. This is the SPA (Single Page Application) catch-all. The SvelteKit frontend uses client-side routing, so routes like `/fic/abc123` or `/settings` need to serve `index.html` and let the JavaScript router take over.

### Middleware: Logging and CORS

At the bottom of the router construction:

```rust
.layer(TraceLayer::new_for_http())
.layer(CorsLayer::permissive())
```

Two middleware layers wrap the entire application:

- **`TraceLayer`** — Logs every incoming request and outgoing response with timing information. This is essential for debugging production issues. The trace includes the HTTP method, path, status code, and response time.

- **`CorsLayer::permissive()`** — Allows all CORS requests. This is important for development (where the frontend might be running on port 5173 and the API on port 3000) and for any API consumers. In a production environment, you'd typically tighten this down to only allow requests from your specific domain.

The middleware is applied in reverse order — `CorsLayer` runs first (closest to the application), then `TraceLayer` (closest to the network). So a request flows through: TraceLayer → CorsLayer → Route matching → Handler.

### Injecting State

The final line is:

```rust
.with_state(state)
```

This injects the shared `AppState` into the router. Without this line, every handler that tries to extract `State<Arc<AppState>>` will fail at runtime with a confusing error message. The `with_state` call is what makes the state available to all handlers through Axum's extractor system.

## Connecting to the System: A Data Flow Diagram

To really understand how the router connects to the rest of FicHub, let's trace data as it flows through the system:

```
User's Browser / E-Reader
         │
         ▼
    TCP Connection (tokio::net::TcpListener)
         │
         ▼
    Axum Server (axum::serve)
         │
         ├──▶ TraceLayer (logs request)
         ├──▶ CorsLayer (handles CORS)
         │
         ▼
    Router Tree (build_router)
         │
         ├──▶ /api/v0/epub ──▶ epub_handler ──▶ ScraperRegistry ──▶ AO3/FFN/etc.
         │                              │──▶ Database (upsert fic_info)
         │                              │──▶ Cache (check/generate EPUB)
         │                              │──▶ Response (JSON with URLs)
         │
         ├──▶ /api/v0/meta ──▶ meta_handler ──▶ ScraperRegistry ──▶ Database
         │
         ├──▶ /cache/{etype}/{url_id} ──▶ cache_download ──▶ Disk (serve file)
         │
         ├──▶ /api/v0/recommendations ──▶ RecommendationEngine ──▶ Database + Redis
         │
         ├──▶ /api/v0/tags ──▶ Tag routes ──▶ Database
         │
         ├──▶ /api/v0/search ──▶ Search handler ──▶ Database
         │
         ├──▶ /opds/* ──▶ OPDS feeds ──▶ Database ──▶ XML response
         │
         └──▶ fallback_service ──▶ ServeDir (static files) ──▶ SvelteKit frontend
```

This diagram shows the complete picture. Every request enters through the same door (the TCP listener), passes through the same middleware (logging and CORS), and gets routed to the appropriate handler. Each handler reaches into the shared `AppState` to access databases, caches, scrapers, and other resources.

The beauty of this architecture is that each piece is independent and testable. You can test the scraper registry without the router. You can test the router without the database (using mock state). You can test the cache download handler without the scraper. The router is the integration point, but the individual pieces are loosely coupled.

## Connecting the Dots

The `run()` function is called from `main()`:

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(
                    "info,fichub=debug"
                )),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

It loads environment variables from `.env` (if present), sets up structured logging with `tracing_subscriber`, loads the configuration, and runs the server. The `RUST_LOG` environment variable controls log verbosity — `info,fichub=debug` is a good default, giving you info-level logs for everything except FicHub's own code, which gets debug-level logging.

The `.ok()` on `dotenvy::dotenv()` means if there's no `.env` file, it silently continues. This is important for Docker deployments where environment variables are passed through Docker Compose instead of a file.

## Request Flow: End to End

Let's trace a complete request to understand how everything connects:

1. User sends `GET /api/v0/epub?q=https://archiveofourown.org/works/12345`
2. Tokio receives the TCP connection
3. Axum parses the HTTP request
4. `TraceLayer` logs the incoming request
5. `CorsLayer` checks for CORS headers
6. The router matches `/api/v0/epub` → `epub_handler`
7. `State(state)` extractor pulls the shared `AppState`
8. `Query(params)` extractor parses `q=https://...`
9. The handler runs through its 10-step flow
10. Returns `Json(value)`
11. `CorsLayer` adds CORS headers to the response
12. `TraceLayer` logs the response with timing
13. Tokio sends the HTTP response back

The whole thing takes milliseconds for cached requests and a few seconds for fresh exports. The async runtime handles thousands of concurrent requests without breaking a sweat.

## 🧪 Try It Yourself

1. Start FicHub locally with `cargo run`
2. Open a browser and navigate to `http://localhost:3000/api/`
3. You should see the API documentation JSON
4. Try `http://localhost:3000/api/v0/remote` — it shows your IP
5. Try `http://localhost:3000/` — you should see the SvelteKit frontend (if the build is in place)
6. Run `curl -v http://localhost:3000/api/v0/epub` to see the empty query error
7. Check the terminal output — you'll see TraceLayer logging each request

## ⚠️ Watch Out

The router construction order matters. In Axum, more specific routes take precedence over less specific ones. The `/api/` routes are registered first, so they always win. If you added `.fallback_service()` before your API routes, the static file server would swallow everything. Always put your fallback last.

Also note that the `with_state(state)` call at the very end injects the shared state into all handlers. If you forget this line, every handler that tries to extract `State<Arc<AppState>>` will fail at runtime with a cryptic error. The error message usually mentions something about "missing state" but it's not always clear what went wrong.

One more subtlety: the `remote_handler` function uses `ConnectInfo<SocketAddr>`, which requires the server to be configured with `into_make_service_with_connect_info`. If you change how the server is started (for example, using a different serve method), this handler will break silently — it'll compile fine but panic at runtime.

---

# Chapter 26: Serving Static Files

We've alluded to it a few times, but now let's really dig into how FicHub serves its SvelteKit frontend. The frontend is a separate application — a SvelteKit SPA that builds into static HTML, CSS, and JavaScript files. FicHub's Rust server doesn't just run an API — it also serves those static files, acting as both the backend API server and the frontend web server.

This is actually a pretty elegant setup. You get a single server, a single port, and a single Docker container that handles everything.

## The Frontend Module

Let's look at `src/frontend/mod.rs`:

```rust
/// Frontend module - serves the SvelteKit built SPA
/// The SvelteKit build output is served as static files by
/// tower-http ServeDir in server.rs

/// Static file serving is handled in server.rs via:
/// `.fallback_service(ServeDir::new(&frontend_dir)
///     .append_index_html_on_directories(true))`
///
/// This module only provides constants and helpers.

/// Default location for the SvelteKit build output
pub const DEFAULT_FRONTEND_DIR: &str = "./frontend/build";
```

This module is intentionally minimal. It defines a single constant — the default path to the SvelteKit build output — and that's it. All the actual file serving happens in `server.rs` through `tower-http`'s `ServeDir` and `ServeFile`.

The `FRONTEND_DIR` environment variable lets you override this path. In Docker, for example, it's set to `/app/frontend`, while locally it defaults to `./frontend/build`. This flexibility means the same binary works in development, Docker, and bare-metal deployments without any code changes.

Why is the module so small? Because `tower-http` already does everything we need. There's no point writing custom file-serving logic when a well-tested, battle-hardened library handles it. The module exists as a central place for the constant and as documentation about how the frontend is served.

## ServeDir: The Static File Engine

The magic happens in the `fallback_service` call:

```rust
ServeDir::new(&frontend_dir)
    .append_index_html_on_directories(true)
    .fallback(ServeFile::new(frontend_dir.join("index.html")))
```

Let's unpack each piece.

### ServeDir

`ServeDir` is a middleware from `tower-http` that serves files from a directory. When a request comes in, it maps the URL path to a file path within the directory and serves it. It handles:

- **MIME type detection** — Based on file extension. `.css` → `text/css`, `.js` → `application/javascript`, `.json` → `application/json`, `.svg` → `image/svg+xml`, etc. It uses the `mime_guess` crate under the hood for reliable detection.

- **404 responses** — If the file doesn't exist, it returns a 404 status code. Without the fallback, this is what happens for all SPA routes.

- **Efficient streaming** — Files are streamed to the client using `tokio::fs::File`, which means the entire file doesn't need to be loaded into memory. For large JavaScript bundles (which can be megabytes), this matters.

- **Conditional requests** — `ServeDir` supports `If-Modified-Since` and `If-None-Match` headers. If the client sends one of these and the file hasn't changed, the server returns 304 Not Modified instead of re-sending the entire file.

- **Content-Length headers** — The file size is determined upfront and sent in the response headers, so the client can show a progress bar during download.

The directory is determined by `state.config.frontend_dir`, which defaults to `./frontend/build`. That's where the SvelteKit build puts its output.

### append_index_html_on_directories

When you set `append_index_html_on_directories(true)`, requesting a directory (like `/` or `/about/`) will serve the `index.html` from that directory. This is important for SvelteKit's routing — when you navigate to `/`, the server needs to serve `frontend/build/index.html`.

Without this option, requesting `/` would return a 404 or a directory listing, neither of which is useful for an SPA. With it, `/` maps to `./frontend/build/index.html`, which is exactly what we want.

This option is particularly important for the root path `/`. When someone visits `http://your-server.com/`, the server receives a request for `/`. Without `append_index_html_on_directories`, ServeDir would look for a file named `` (empty string) and fail. With it, ServeDir recognizes `/` as a directory and serves `index.html`.

### The Fallback: SPA Catch-All

The most important part is the `.fallback(ServeFile::new(frontend_dir.join("index.html")))` call. This is the SPA catch-all.

Consider this scenario: a user navigates directly to `http://your-server.com/fic/abc123`. The server receives a request for `/fic/abc123`. `ServeDir` looks for `frontend/build/fic/abc123` — that file doesn't exist. Without the fallback, the user would get a 404.

With the fallback, `ServeDir` gives up and passes the request to `ServeFile`, which serves `index.html` regardless of the path. The SvelteKit JavaScript then boots up, sees the URL path `/fic/abc123`, and renders the appropriate page. This is the standard SPA routing pattern.

The fallback chain works like this:

1. Request for `/fic/abc123` arrives
2. `ServeDir` tries `./frontend/build/fic/abc123` — not found
3. `ServeDir` tries `./frontend/build/fic/abc123/index.html` — not found
4. `ServeDir` gives up, invokes the fallback
5. `ServeFile` serves `./frontend/build/index.html`
6. SvelteKit's JavaScript boots, reads the URL, renders the fic page

This is invisible to the user — they just see the page they asked for. And it works for every possible path: `/settings`, `/search?q=hello`, `/recommendations/abc123`, anything. They all serve `index.html`, and the JavaScript handles the rest.

## Build Directory Structure

When you build the SvelteKit frontend (`npm run build` or `vite build`), it produces a `build/` directory with a structure like this:

```
frontend/build/
├── _app/
│   ├── immutable/
│   │   ├── assets/
│   │   │   ├── index-Bx3d5j1L.css      (hashed CSS bundle)
│   │   │   └── index-C7H5dK3x.js       (hashed JS bundle)
│   │   └── chunks/
│   │       ├── index-Bk2eP1aR.js       (code-split chunk)
│   │       ├── error-CjH5fG2x.js       (error page chunk)
│   │       └── _layout-Dm4eN8wQ.js     (layout chunk)
│   └── version.json                     (build version info)
├── favicon.png
├── index.html                           (the entry point)
└── manifest.json                        (PWA manifest)
```

Key things to notice:

- **`index.html`** — The entry point. This is what the SPA fallback serves for any unmatched route. It contains `<script>` tags that load the JavaScript bundles, and `<link>` tags that load the CSS.

- **`_app/immutable/assets/`** — Hashed asset filenames. These are the "immutable" assets — once built, they never change. The hash in the filename (like `index-Bx3d5j1L.css`) ensures that if the content changes, the filename changes too. This is the key to cache busting — browsers can safely cache these files forever because the filename will change when the content changes.

- **`_app/immutable/chunks/`** — Code-split chunks that are loaded on demand. SvelteKit automatically splits code so that only the JavaScript needed for the current page is loaded initially. If you navigate to a different page, the corresponding chunk is fetched on demand.

- **`version.json`** — Contains the build timestamp and a hash of the build. The SvelteKit service worker uses this to detect when a new version is available and prompt the user to refresh.

The entire build directory is typically 500KB to 2MB, depending on how many components and pages the frontend has.

## Cache Headers

One thing that SvelteKit handles beautifully is cache headers. The immutable assets in `_app/immutable/` are meant to be cached forever. Their filenames contain content hashes, so when the code changes, the filename changes too. Browsers can safely cache `index-Bx3d5j1L.css` forever because if the CSS changes, it'll be `index-Kq9mN2wQ.css` next time.

The `index.html` file, on the other hand, should never be cached (or cached very briefly). It's the entry point, and it references the hashed asset filenames. If `index.html` is cached, users might load stale JavaScript that references assets that no longer exist.

`tower-http`'s `ServeDir` doesn't set cache headers by default. For FicHub's production deployment, you'd typically put an nginx reverse proxy in front that sets appropriate cache headers:

```nginx
# Immutable assets — cache for a year
location /_app/immutable/ {
    add_header Cache-Control "public, max-age=31536000, immutable";
}

# Everything else — don't cache
location / {
    add_header Cache-Control "no-cache, must-revalidate";
}
```

If you're not using nginx, you can add cache header middleware in the Axum router itself. You'd create a custom middleware layer that inspects the request path and adds appropriate `Cache-Control` headers to the response.

### Adding Cache Headers in Axum

For deployments without nginx, here's how you'd add cache headers directly in the Axum router:

```rust
use tower_http::set_header::SetResponseHeaderLayer;
use axum::http::header::CACHE_CONTROL;

// Before the fallback_service:
.layer(SetResponseHeaderLayer::overriding(
    CACHE_CONTROL,
    HeaderValue::from_static("no-cache"),
))
```

This sets a default `Cache-Control: no-cache` header on all responses. You'd then need a more targeted middleware to set longer cache times for the immutable assets. The `tower-http` ecosystem has the tools for this, but nginx is generally the easier choice.

## How the Rust Server Serves the SvelteKit Frontend

The full flow works like this:

**First visit (no cache):**

1. User visits `http://your-server.com/`
2. Request hits Axum
3. No API route matches (`/api/`, `/api/v0/...`, etc.)
4. `ServeDir` looks for `./frontend/build/` → finds `index.html` → serves it
5. Browser downloads `index.html`, which loads `index-C7H5dK3x.js`
6. Request for `index-C7H5dK3x.js` hits Axum
7. `ServeDir` finds the file in `./frontend/build/_app/immutable/assets/` → serves it
8. SvelteKit JavaScript boots up, takes over client-side routing
9. User sees the FicHub web interface

**Navigating within the app:**

10. User clicks a link to `/fic/abc123`
11. Client-side router intercepts the click (prevents a full page reload)
12. Router renders the fic page component with the `abc123` parameter
13. No request to the server (unless the component fetches data)

**Direct URL access (refresh on a sub-page):**

14. User refreshes the page on `/fic/abc123`
15. Browser requests `http://your-server.com/fic/abc123`
16. No API route matches
17. `ServeDir` can't find `./frontend/build/fic/abc123`
18. The fallback `ServeFile` kicks in → serves `index.html`
19. SvelteKit boots up, sees the URL, renders the fic page

**Loading a hashed asset:**

20. `index.html` contains `<link rel="stylesheet" href="/_app/immutable/assets/index-Bx3d5j1L.css">`
21. Browser requests `/_app/immutable/assets/index-Bx3d5j1L.css`
22. `ServeDir` maps this to `./frontend/build/_app/immutable/assets/index-Bx3d5j1L.css`
23. File exists → served with correct MIME type (`text/css`)
24. Browser caches it forever (because of the content hash in the filename)

This is the beauty of SPA routing with a Rust backend. The server is simple — it just serves files — and the client handles the routing intelligence.

## Production Considerations

In development, you'd typically run the SvelteKit dev server on one port (like 5173) and the Rust server on another (3000), with the SvelteKit dev server proxying API requests to the Rust server. This gives you hot module replacement and fast iteration. The Vite dev server is configured with a proxy:

```javascript
// vite.config.js
export default {
    server: {
        proxy: {
            '/api': 'http://localhost:3000',
            '/cache': 'http://localhost:3000',
            '/opds': 'http://localhost:3000',
        }
    }
};
```

For production, you build the SvelteKit app (`npm run build`) and copy the `build/` output into the path where FicHub expects it (default: `./frontend/build`). Then the Rust server serves everything.

### Building the Frontend

```bash
cd frontend
npm install
npm run build
# Output goes to frontend/build/
```

The build process:
1. Compiles all Svelte components
2. Tree-shakes unused code (removes anything not imported)
3. Code-splits into chunks for lazy loading
4. Hashes all asset filenames
5. Generates `index.html` with correct script/link tags
6. Copies static assets (favicon, manifest, etc.)

### Deploying the Build

If you're running FicHub directly (not in Docker), the build output needs to be in `./frontend/build` relative to where you run the FicHub binary:

```bash
# Build the frontend
cd frontend && npm run build && cd ..

# The build output is already in frontend/build/
# Just make sure FRONTEND_DIR points to the right place
# Default: ./frontend/build
export FRONTEND_DIR=./frontend/build

# Start the server
cargo run --release
```

In Docker, the Dockerfile copies the frontend build into the container (or the frontend is mounted as a volume). The `FRONTEND_DIR` environment variable in `docker-compose.yml` is set to `/app/frontend`.

### Without the Frontend

If you don't have the frontend build in place, the SPA fallback still works — `index.html` won't exist, so every non-API request will return a 404 from `ServeFile`. The API endpoints will still work perfectly, though. You can use FicHub purely as an API server without the web UI.

This is useful for headless deployments where another frontend (a mobile app, a command-line tool, or a different web framework) consumes the API directly. The OPDS catalog also works without the web UI — e-readers only need the OPDS endpoints.

### Performance

Static file serving is incredibly efficient. `tower-http`'s `ServeDir` uses `tokio::fs::File` for async I/O, so file reads don't block the runtime. On a modern SSD, FicHub can serve thousands of static file requests per second. The main bottleneck is network bandwidth, not the file serving itself.

If you're serving a lot of static files, you might want to add gzip compression. The `tower-http` crate includes `CompressionLayer` that can compress responses on the fly. For text-based assets (HTML, CSS, JS), gzip compression typically reduces file sizes by 60-80%, significantly improving load times.

## 🧪 Try It Yourself

1. Build the SvelteKit frontend: `cd frontend && npm install && npm run build`
2. Start the FicHub server: `cargo run`
3. Visit `http://localhost:3000/` — you should see the frontend
4. Open your browser's developer tools, go to the Network tab
5. Navigate to a non-existent page like `http://localhost:3000/nonexistent`
6. You'll see the request for `/nonexistent` return `index.html` (status 200, type document)
7. The SvelteKit router handles the 404 page client-side
8. Check the response headers — you should see `Content-Type: text/html` and the file size
9. Try clearing the browser cache and refreshing — watch the network requests to see the full loading sequence

## ⚠️ Watch Out

The most common deployment mistake is forgetting to build the frontend. If `./frontend/build/index.html` doesn't exist, the SPA fallback returns a 404, and users see a blank page or an error. The API still works, but the web UI is dead. Always verify that `index.html` exists before deploying.

Another gotcha: `ServeDir` serves files from the filesystem at request time. It doesn't cache them in memory. If you're serving thousands of files, the filesystem performance matters. On a fast SSD, this is fine. On a slow spinning disk, you might want to add a caching layer.

A third issue: symlinks. If your `frontend/build` directory contains symbolic links (which can happen if you use a build tool that creates them), `ServeDir` follows them by default. This is usually fine, but it can be a security concern if the symlinks point outside the expected directory.

And one more: the trailing slash. `ServeDir` handles paths with and without trailing slashes differently. `/style.css` works fine, but `/about` vs `/about/` might behave differently depending on the `append_index_html_on_directories` setting. SvelteKit generates links with trailing slashes for directory-like routes, so this usually works out of the box.

---

# Chapter 27: Docker and Docker Compose

If you've made it this far, you have a working FicHub server with PostgreSQL, Redis, scrapers, exporters, a recommendation engine, and a frontend. Running it all locally with `cargo run` works for development, but for production — and especially for deployment on a remote server — Docker is your best friend.

## What Is Docker? A Lunchbox for Your App

Imagine you've made a delicious sandwich. Now you want to take it to work. You put it in a lunchbox so it stays fresh and doesn't make a mess. Docker is the lunchbox for your application.

When you "containerize" an application with Docker, you pack up:
- The application code
- The compiled binary
- All system libraries it needs
- A minimal operating system to run it

The result is a self-contained unit (a "container") that runs the same way on your laptop, on a cloud server, on your friend's machine, or on a Raspberry Pi. No "works on my machine" problems.

Docker images are like recipes — they describe exactly how to build the container. Containers are like instances — they're the running version of the recipe.

Let's clarify some terminology:

- **Dockerfile** — A text file with instructions for building a Docker image. Think of it as a recipe.
- **Image** — The compiled result of a Dockerfile. A read-only snapshot of everything your app needs.
- **Container** — A running instance of an image. Multiple containers can run from the same image.
- **Volume** — A persistent storage area that survives container restarts and removals.
- **Docker Compose** — A tool for defining and running multi-container applications. You describe all your services in a YAML file and start them with one command.

## The Dockerfile: A Recipe for FicHub

FicHub's Dockerfile uses a multi-stage build. This is a clever trick that keeps the final image small. Here it is:

```dockerfile
# syntax=docker/dockerfile:1

# Stage 1: Build the Rust binary
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Stage 2: Create the slim runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fichub /app/fichub
COPY --from=builder /app/migrations /app/migrations
COPY .env.example /app/.env

EXPOSE 3000

CMD ["/app/fichub"]
```

### The Builder Stage

The first stage (`FROM rust:slim-bookworm AS builder`) is the build environment. It's a complete development environment with the Rust compiler and all necessary tools.

**Base image:** `rust:slim-bookworm` is a Debian Bookworm image with Rust pre-installed. The "slim" variant strips out documentation, man pages, and other non-essential files, keeping the image smaller than the full Rust image. At around 200-300 MB, it's still much larger than the final runtime image — but that's OK because we throw it away after building.

**Build dependencies:** The `apt-get install` line installs three packages needed at compile time:

- `pkg-config` — Helps Cargo find system libraries (like OpenSSL and libpq) during compilation
- `libssl-dev` — OpenSSL development headers (needed by crates that use TLS)
- `libpq-dev` — PostgreSQL client library development headers (needed by SQLx)

The `&& rm -rf /var/lib/apt/lists/*` line cleans up the apt cache to keep the layer smaller.

**Source code:** The `COPY` commands bring in the source files:

```dockerfile
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
```

Notice that `Cargo.toml` and `Cargo.lock` are copied first. This is a Docker best practice — Docker caches layers, and if `Cargo.toml` hasn't changed, Docker can skip the dependency resolution step. Only `src/` and `migrations/` trigger a full rebuild.

**Build command:** `cargo build --release` compiles the binary in release mode. Release mode enables optimizations (`-O2` or `-O3`), which makes the binary much faster but takes longer to compile. In a Docker build, this typically takes 5-10 minutes.

### The Runtime Stage

The second stage (`FROM debian:bookworm-slim`) is the actual runtime environment. It's much smaller:

**Base image:** `debian:bookworm-slim` is a minimal Debian image without Rust, without compilers, without build tools. Just the essentials — a basic shell, glibc, and a package manager. It's typically around 70-80 MB.

**Runtime dependencies:** Only two packages are needed:

- `ca-certificates` — Root certificates for HTTPS connections (needed by `reqwest` when scraping sites over HTTPS)
- `libpq-dev` — The PostgreSQL client library that the compiled binary links against at runtime

**Copy the binary:** The key line is:

```dockerfile
COPY --from=builder /app/target/release/fichub /app/fichub
```

This pulls just the compiled binary from the builder stage. No source code, no build artifacts, no compiler. Just the single executable file.

**Copy migrations:** The SQL migration files are copied too, since FicHub runs migrations on startup:

```dockerfile
COPY --from=builder /app/migrations /app/migrations
```

**Copy default config:** `COPY .env.example /app/.env` provides a default environment configuration. In production, you'd override this with Docker Compose environment variables.

**Expose and run:**

```dockerfile
EXPOSE 3000
CMD ["/app/fichub"]
```

`EXPOSE 3000` is documentation — it tells Docker users that the container listens on port 3000. It doesn't actually publish the port (that's done with `-p 3000:3000` or in Docker Compose). `CMD` specifies the default command to run when the container starts.

### Why Multi-Stage?

The final image only contains the runtime binary and its minimal dependencies. It does NOT contain:
- The Rust compiler (hundreds of MB)
- All the source code
- Build dependencies like `libssl-dev` and `pkg-config`
- Cargo cache, intermediate build artifacts
- Any of the `target/debug/` or `target/release/` intermediate files

This makes the final image much smaller — typically 50-100 MB instead of 1+ GB. Smaller images mean faster pulls, less disk usage, and a smaller attack surface. They also start faster and use less memory.

### Docker Build Cache

Docker caches each layer of a Dockerfile. If a layer hasn't changed since the last build, Docker reuses the cached version. This means subsequent builds are much faster.

For example, if you only change source code (not `Cargo.toml`), Docker will:
1. Reuse the cached `rust:slim-bookworm` base image ✓
2. Reuse the cached `apt-get install` layer ✓
3. Reuse the cached `COPY Cargo.toml Cargo.lock` layer ✓
4. Rebuild `COPY src ./src` (changed) ✗
5. Rebuild `cargo build --release` (because src changed) ✗

This can save significant time during development iterations.

## Docker Compose: Orchestrating the Stack

FicHub needs three services to run: PostgreSQL, Redis, and the application itself. Docker Compose makes it easy to define and run all of them together.

Here's `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: fichub
      POSTGRES_PASSWORD: fichub
      POSTGRES_USER: fichub
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  calibre:
    build:
      context: .
      dockerfile: docker/calibre.Dockerfile
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp

  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      CALIBRE_CONTAINER: calibre
      PORT: 3000
      FRONTEND_DIR: /app/frontend
      RUST_LOG: info,fichub=debug
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

volumes:
  pgdata:
  cache:
  tmp:
```

### PostgreSQL

```yaml
postgres:
  image: postgres:16-alpine
  environment:
    POSTGRES_DB: fichub
    POSTGRES_PASSWORD: fichub
    POSTGRES_USER: fichub
  volumes:
    - pgdata:/var/lib/postgresql/data
```

FicHub uses PostgreSQL 16 (the Alpine variant, which is smaller — about 80 MB instead of 350 MB for the full image). The environment variables configure PostgreSQL:

- `POSTGRES_DB` — Creates a database called `fichub`
- `POSTGRES_USER` — Creates a user called `fichub`
- `POSTGRES_PASSWORD` — Sets the password for that user

The `pgdata` named volume persists the database across container restarts — you won't lose your data when you run `docker compose down` and `docker compose up` again.

The healthcheck ensures that the app service doesn't start until PostgreSQL is actually ready to accept connections:

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U fichub"]
  interval: 5s
  timeout: 5s
  retries: 5
```

This runs `pg_isready` every 5 seconds. If it fails 5 times in a row (25 seconds), PostgreSQL is marked as unhealthy. The `depends_on: condition: service_healthy` on the app service means FicHub won't start until PostgreSQL passes its healthcheck.

### Redis

```yaml
redis:
  image: redis:7-alpine
  ports:
    - "6379:6379"
```

Redis 7 (also Alpine) handles rate limiting and recommendation caching. The healthcheck pings Redis to confirm it's alive:

```yaml
healthcheck:
  test: ["CMD", "redis-cli", "ping"]
  interval: 5s
  timeout: 5s
  retries: 5
```

Unlike PostgreSQL, Redis doesn't need a persistent volume — the rate limit data and recommendation cache are ephemeral and can be lost without issues. If Redis restarts, the rate limit counters reset (which is actually fine — it gives everyone a fresh start).

### Calibre

```yaml
calibre:
  build:
    context: .
    dockerfile: docker/calibre.Dockerfile
  volumes:
    - cache:/app/cache
    - tmp:/app/tmp
```

This is the sidecar container that handles MOBI and PDF conversion. When FicHub needs to convert an EPUB to MOBI or PDF, it talks to the Calibre container. The Calibre container shares the `cache` and `tmp` volumes with the main app, so it can read EPUBs from the cache and write converted files back.

FicHub communicates with the Calibre container using Docker's networking — it calls the Calibre container by its service name (`calibre`), which Docker Compose resolves automatically via its built-in DNS server.

The Calibre container is built from a separate Dockerfile (`docker/calibre.Dockerfile`) that installs Calibre and runs a conversion server. Calibre is a large package (hundreds of MB), so isolating it in a separate container keeps the main app image small.

### The Application

```yaml
app:
  build: .
  ports:
    - "3000:3000"
  environment:
    DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
    REDIS_URL: redis://redis:6379
    CACHE_DIR: /app/cache
    TMP_DIR: /app/tmp
    CALIBRE_CONTAINER: calibre
    PORT: 3000
    FRONTEND_DIR: /app/frontend
    RUST_LOG: info,fichub=debug
  depends_on:
    postgres:
      condition: service_healthy
    redis:
      condition: service_healthy
```

The app service:

1. **Builds from the Dockerfile** in the root directory (`.`)
2. **Maps port 3000** to the host (`host:container`)
3. **Sets environment variables** that the Rust app reads via `Config::from_env()`
4. **Shares volumes** for cache and temp files (same volumes as Calibre)
5. **Depends on** PostgreSQL and Redis, waiting for their healthchecks to pass

Notice that `DATABASE_URL` uses the Docker network hostname `postgres` and `REDIS_URL` uses `redis`. Inside Docker Compose, containers can reach each other by service name. Docker Compose creates a virtual network and registers each service's name in its built-in DNS. This is different from local development where you'd use `localhost`.

## Environment Variables in Docker

The environment variables in `docker-compose.yml` correspond directly to the `Config::from_env()` method we looked at in earlier chapters:

| Variable | Value | Purpose |
|----------|-------|---------|
| `DATABASE_URL` | `postgres://fichub:***@postgres:5432/fichub` | PostgreSQL connection string |
| `REDIS_URL` | `redis://redis:6379` | Redis connection string |
| `CACHE_DIR` | `/app/cache` | Where cached exports are stored |
| `TMP_DIR` | `/app/tmp` | Temporary files during export |
| `CALIBRE_CONTAINER` | `calibre` | Docker service name for Calibre |
| `PORT` | `3000` | HTTP server port |
| `FRONTEND_DIR` | `/app/frontend` | Path to SvelteKit build output |
| `RUST_LOG` | `info,fichub=debug` | Log levels |

In production, you'd override these with a `.env` file or Docker secrets. The `docker-compose.yml` shown here uses sensible defaults for getting started.

### Security Note

The `DATABASE_URL` in the compose file contains a password (`***`). In a real deployment, you'd want to:

1. Use a `.env` file for secrets (add it to `.gitignore`)
2. Or use Docker secrets for more secure secret management
3. Or use an environment-specific compose override file

Never commit real passwords to version control.

## Volumes: Persistence Across Restarts

Docker Compose defines three named volumes:

```yaml
volumes:
  pgdata:
  cache:
  tmp:
```

- **`pgdata`** — PostgreSQL's data directory. This is the most critical volume — it contains your database. Without it, you'd lose all fic metadata, export logs, recommendations, and tags every time you restart. Docker stores this in its own storage area (usually `/var/lib/docker/volumes/fichub_pgdata/_data/`).

- **`cache`** — Exported EPUBs, HTML bundles, MOBIs, and PDFs. If this is lost, FicHub can regenerate exports on demand, but it means extra scraping and processing time. For a busy server, regenerating all cached exports could take hours.

- **`tmp`** — Temporary files during export. This can be lost without issues. It's only used as a staging area during EPUB generation.

Named volumes are managed by Docker and persist across container restarts. They're stored in Docker's own storage area and are independent of the container's lifecycle. When you run `docker compose down`, containers are stopped and removed, but volumes persist. Only `docker compose down -v` removes volumes.

## Building and Running

### First-Time Setup

```bash
# Clone the repository
git clone https://github.com/youruser/fichub.git
cd fichub

# Build and start all services
docker compose up -d

# Watch the logs
docker compose logs -f app
```

The first build takes a while — Docker needs to download base images and compile the Rust binary. Subsequent builds are faster because Docker caches layers.

### Running in the Foreground

For development, you might want to see the logs:

```bash
docker compose up
```

This runs all services in the foreground with interleaved log output. You'll see PostgreSQL initializing, Redis starting, Calibre loading, and FicHub compiling and starting. Press Ctrl+C to stop all services.

### Running in the Background

For production:

```bash
docker compose up -d
```

The `-d` flag runs containers in detached mode. They'll keep running even after you close the terminal. You can check their status with `docker compose ps`.

### Checking Status

```bash
docker compose ps
```

This shows which containers are running, their ports, their health status, and their uptime. You should see all four services (postgres, redis, calibre, app) in a "running" state.

### Viewing Logs

```bash
# All services
docker compose logs

# Just the app
docker compose logs app

# Follow mode (like tail -f)
docker compose logs -f app

# Last 100 lines
docker compose logs --tail 100 app

# Since a specific time
docker compose logs --since 10m app
```

### Stopping

```bash
# Stop all containers (preserves volumes)
docker compose down

# Stop AND remove volumes (destroys data!)
docker compose down -v
```

⚠️ **Watch Out:** Never run `docker compose down -v` in production. The `-v` flag removes volumes, which deletes your PostgreSQL database and cached exports. Use `docker compose down` (without `-v`) to preserve data.

### Rebuilding After Code Changes

When you change the Rust code, you need to rebuild the Docker image:

```bash
# Rebuild just the app service
docker compose build app

# Or rebuild everything
docker compose build

# Then restart
docker compose up -d
```

Docker Compose will use cached layers for unchanged parts, so rebuilds are faster than fresh builds.

## The Calibre Sidecar for MOBI/PDF Conversion

FicHub generates EPUB files natively in Rust. But MOBI and PDF formats require Calibre — a powerful ebook management tool with conversion capabilities. Rather than installing Calibre in the main container (which would bloat the image enormously — Calibre is over 500 MB), FicHub uses a "sidecar" pattern:

1. The main app generates an EPUB
2. If MOBI or PDF is requested, it communicates with the Calibre container
3. Calibre reads the EPUB from the shared cache volume
4. Calibre converts it and writes the result back to the cache volume
5. The main app serves the converted file

This keeps the main container lean while still supporting all export formats. The Calibre container is built from a separate Dockerfile (`docker/calibre.Dockerfile`) that installs Calibre and runs a conversion server.

The `CALIBRE_CONTAINER` environment variable tells the main app what Docker service name to use when communicating with Calibre. If you don't need MOBI/PDF conversion, you can leave `CALIBRE_CONTAINER` empty and FicHub will skip the Calibre integration. The epub and html exports still work — only mobi and pdf are affected.

## Docker Networking

Docker Compose creates a default network for all services. Containers on this network can reach each other by service name:

- `postgres:5432` — PostgreSQL from the app
- `redis:6379` — Redis from the app
- `calibre` — Calibre from the app

The `ports` mapping in the compose file publishes ports to the host machine:

- `5432:5432` — PostgreSQL is accessible from the host (useful for debugging)
- `6379:6379` — Redis is accessible from the host
- `3000:3000` — FicHub is accessible from the host

If you don't need external access to PostgreSQL or Redis, remove their port mappings. This is actually more secure — it prevents other processes on the host from connecting to your database.

## Docker Build Optimization

Building Rust in Docker can be slow because of dependency compilation. Here are some optimization techniques:

### Cache Cargo Dependencies

The key insight is that Cargo dependencies change much less frequently than source code. By copying `Cargo.toml` and `Cargo.lock` first, Docker can cache the dependency compilation:

```dockerfile
# Copy manifests first (for dependency caching)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source and build
COPY src ./src
RUN cargo build --release
```

This creates a dummy build that compiles all dependencies, then reuses that cache when building with the real source code. For FicHub, which has many dependencies (axum, sqlx, reqwest, serde, etc.), this can save 5+ minutes on subsequent builds.

### Use cargo-chef

For even better caching, use `cargo-chef`:

```dockerfile
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release
```

`cargo-chef` separates dependency compilation from source compilation, giving Docker more opportunities to cache layers. It analyzes your `Cargo.toml` and `Cargo.lock` to create a "recipe" of just the dependency compilation steps, then cooks that recipe in a separate layer. When you change source code, only the final `cargo build` layer needs to rebuild — all dependencies are already compiled and cached.

### Build the Frontend in Docker

If you want to build the SvelteKit frontend inside Docker (to avoid installing Node.js on your dev machine), you can add another build stage:

```dockerfile
# Stage 1: Build the frontend
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build the Rust binary (existing builder stage)
FROM rust:slim-bookworm AS builder
# ... existing build steps ...

# Stage 3: Runtime
FROM debian:bookworm-slim
# ... existing runtime setup ...
COPY --from=frontend-builder /app/frontend/build /app/frontend
```

This adds a Node.js build stage that compiles the SvelteKit frontend, and the runtime stage copies the build output into the container. The result is a fully self-contained Docker image with both the backend API and the frontend web UI.

### Docker Security Best Practices

For production deployments, consider these security additions to your Dockerfile:

```dockerfile
# Don't run as root
RUN useradd -r -s /bin/false fichub
USER fichub

# Use a non-root base image
# (debian:bookworm-slim already runs as root, but you can add USER)

# Don't install unnecessary packages
# (already handled by using slim images)

# Scan for vulnerabilities
# docker scout cves fichub:latest
```

Running as a non-root user inside the container is a defense-in-depth measure. Even if an attacker breaks out of the container, they'd only have the permissions of the `fichub` user, not root.

## Production Hardening

Whether you're running Docker or bare-metal, there are some production considerations that apply to both:

### TLS/HTTPS

FicHub doesn't handle TLS directly. In production, you'd put an nginx or Caddy reverse proxy in front that terminates TLS:

```nginx
server {
    listen 443 ssl http2;
    server_name fichub.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/fichub.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

The proxy also handles:
- Request logging
- Rate limiting (at the edge, before FicHub even sees the request)
- Gzip compression for API responses
- Static asset caching
- Security headers (HSTS, CSP, etc.)

### Log Rotation

FicHub uses `tracing` for structured logging. In production, logs can grow quickly. Set up log rotation:

```bash
# /etc/logrotate.d/fichub
/home/fichub/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    copytruncate
}
```

Or if running in Docker, use the `json-file` log driver with max-size:

```yaml
logging:
  driver: json-file
  options:
    max-size: "10m"
    max-file: "3"
```

### Backup Strategy

For a production FicHub deployment, back up these things:

1. **PostgreSQL database** — `pg_dump fichub > backup.sql`
2. **Cache directory** — `tar czf cache-backup.tar.gz /home/fichub/cache/`
3. **Configuration** — The `.env` file and systemd service file
4. **Migrations** — Already in version control, so just `git clone` on a new machine

A simple backup script:

```bash
#!/bin/bash
BACKUP_DIR="/backup/fichub/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Database
sudo -u fichub pg_dump fichub > "$BACKUP_DIR/fichub.sql"

# Cache (just the manifest, not all files)
ls /home/fichub/cache/ > "$BACKUP_DIR/cache-manifest.txt"

# Config
cp /home/fichub/.env "$BACKUP_DIR/"
cp /etc/systemd/system/fichub.service "$BACKUP_DIR/"

# Cleanup old backups (keep 30 days)
find /backup/fichub/ -maxdepth 1 -mtime +30 -exec rm -rf {} +
```

Run this daily via cron:

```bash
0 2 * * * /home/fichub/backup.sh >> /var/log/fichub-backup.log 2>&1
```

The 2 AM timing ensures backups happen during low-traffic hours.

## 🧪 Try It Yourself

1. Install Docker and Docker Compose if you haven't already
2. Run `docker compose up` in the FicHub directory
3. Wait for all services to start (watch the logs for "Listening on 0.0.0.0:3000")
4. Open `http://localhost:3000/` in your browser
5. Try `docker compose ps` to see all running containers
6. Try `docker compose exec postgres psql -U fichub fichub` to explore the database
7. Try stopping and restarting: `docker compose down && docker compose up -d`
8. Notice that your data persists across restarts (thanks to named volumes)

## ⚠️ Watch Out

The biggest pitfall with Docker is the database. When you run `docker compose down -v`, the PostgreSQL volume is destroyed. All your data — fic metadata, export logs, recommendations, tags — is gone. If you're testing, this is fine. If you're in production, always use named volumes and never use `-v` unless you really mean it.

Another common issue: the first `cargo build --release` inside Docker takes 5-10 minutes. If you're making code changes and rebuilding frequently, consider using `cargo-watch` locally and only building Docker images for deployment. Alternatively, use Docker build caching as described above.

A third issue: the `ports` mapping exposes PostgreSQL to the host machine. In production, remove the PostgreSQL port mapping to prevent unauthorized access. The app can reach PostgreSQL through Docker's internal network, and that's all you need.

---

# Chapter 28: Cross-Compilation and Orange Pi Deploy

Docker is great for deployment, but sometimes you want to run your application directly on a machine without the overhead of containers. Maybe you have a small ARM board like an Orange Pi. Maybe you want the simplest possible deployment: copy a binary, run it, done.

That's where cross-compilation comes in.

## What Is Cross-Compilation?

Normally, when you run `cargo build --release`, the compiler builds a binary for your current machine. If you're on an x86_64 Linux machine, you get an x86_64 Linux binary. That binary won't run on an ARM machine like a Raspberry Pi or Orange Pi.

Cross-compilation means building a binary on one machine that runs on a different machine. You're still sitting at your x86_64 desktop, but you're telling the Rust compiler: "Please make an ARM64 binary instead."

This requires two things:
1. **A target toolchain** — The Rust compiler needs to know how to generate ARM64 machine code
2. **A cross-linker** — The final linking step needs a linker that can create ARM64 executables

Rust's cross-compilation support is excellent. The compiler itself handles most architectures natively. You just need to install the target and provide a linker. This is one of Rust's strengths over languages like C/C++ where cross-compilation can be a nightmare of configuring sysroots, compilers, and toolchains.

## The Orange Pi 5: A Small ARM Server

The Orange Pi 5 is a small single-board computer based on the Rockchip RK3588S processor. It has:
- An ARM64 (AArch64) processor — 4 performance cores + 4 efficiency cores
- 4GB, 8GB, or 16GB of LPDDR4/5 RAM
- Gigabit Ethernet
- eMMC and SD card storage
- USB 3.0, PCIe, HDMI
- Very low power consumption (5-15 watts)

It's a popular choice for self-hosting because it's powerful enough to run services like FicHub but small enough to fit on a shelf and use minimal power. Think of it as a Raspberry Pi's bigger, faster cousin — roughly 3-5x the CPU performance of a Raspberry Pi 4.

We're going to cross-compile FicHub on an x86_64 machine and deploy it to an Orange Pi 5 running an ARM64 Linux distribution (like Armbian or Manjaro ARM).

### Why Not Just Build on the Pi?

You absolutely could install the Rust toolchain on the Pi and build there. But:

- The Pi's CPU is slower than a desktop, so compilation takes 5-10x longer
- Building on the Pi consumes resources that could be serving requests
- Cross-compilation lets you build in a CI/CD pipeline on faster hardware
- You can test the binary on your desktop (via QEMU) before deploying

Cross-compilation is a skill that pays off whenever you deploy to embedded or ARM devices.

## Installing the AArch64 Target

First, you need to tell Rust that you want to target ARM64 Linux. This is a one-time setup step on your development machine:

```bash
rustup target add aarch64-unknown-linux-gnu
```

This downloads the Rust standard library pre-compiled for ARM64 Linux. The standard library includes all the fundamental types and functions that Rust programs need — things like string handling, collections, I/O, networking, etc.

You can verify it's installed:

```bash
rustup target list --installed
# Should show: aarch64-unknown-linux-gnu (and your native target)
```

The target name follows a triple format: `{arch}-{vendor}-{os}-{env}`. For ARM64 Linux, it's `aarch64-unknown-linux-gnu`. The `unknown` means we don't care about the vendor, and `gnu` means we're linking against glibc (as opposed to `musl` for a fully static binary).

### Alternative: musl for Fully Static Binaries

If you want a binary that doesn't depend on glibc at all (which means it'll run on ANY Linux, regardless of the system's glibc version), you can use musl:

```bash
rustup target add aarch64-unknown-linux-musl
```

Musl-based binaries are fully static — they don't link against any system libraries. This is useful for deployment to minimal Linux distributions or containers without glibc. However, musl has some limitations (less mature threading support, different DNS resolution behavior), so glibc is usually the safer choice for a server application.

## The Cross-Compiler

Rust can generate ARM64 machine code, but it needs help with linking. The linker is the program that takes all the compiled object files and combines them into a single executable. For cross-compilation, you need a linker that can create ARM64 executables.

On Arch Linux or similar distributions:

```bash
sudo pacman -S aarch64-linux-gnu-gcc
```

On Ubuntu/Debian:

```bash
sudo apt install gcc-aarch64-linux-gnu
```

On Fedora:

```bash
sudo dnf install gcc-aarch64-linux-gnu
```

On macOS (via Homebrew):

```bash
brew install aarch64-elf-gcc
# Note: macOS cross-compilation to Linux requires more setup
```

This installs a full cross-compilation toolchain including:
- `aarch64-linux-gnu-gcc` — The cross-linker (and C compiler)
- `aarch64-linux-gnu-ld` — The cross-assembler/linker
- `aarch64-linux-gnu-ar` — The cross-archiver
- Standard ARM64 libraries (like `libgcc`)

You can verify the installation:

```bash
aarch64-linux-gnu-gcc --version
# Should show: aarch64-linux-gnu-gcc (GCC) ...
```

## Setting the Linker in .cargo/config.toml

Now you need to tell Cargo to use the cross-linker when building for the `aarch64-unknown-linux-gnu` target. Create (or edit) `.cargo/config.toml` in your project root:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

This configuration tells Cargo: "When I'm building for `aarch64-unknown-linux-gnu`, use `aarch64-linux-gnu-gcc` as the linker."

You can put this in the project's `.cargo/config.toml` (so it's checked into git and shared with the team) or in `~/.cargo/config.toml` (so it applies to all your projects). For FicHub, the project-level config is better because it ensures everyone on the team uses the same cross-compilation setup.

Without this configuration, Cargo would try to use the system's default linker (usually `cc`), which creates x86_64 binaries. The cross-linker is essential for producing ARM64 executables.

## Building the Binary

Now the fun part. One command to build for ARM64:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

This:
1. Compiles all Rust code targeting ARM64 architecture
2. Links using the `aarch64-linux-gnu-gcc` cross-linker
3. Produces an executable at `target/aarch64-unknown-linux-gnu/release/fichub`

The build takes a few minutes (cross-compilation can be slightly slower than native compilation because the cross-linker has more work to do). When it's done, you can verify the binary:

```bash
file target/aarch64-unknown-linux-gnu/release/fichub
# Should show: ELF 64-bit LSB executable, ARM aarch64, ...
```

If you see "ARM aarch64" in the output, you've successfully cross-compiled!

You can also check the binary size:

```bash
ls -lh target/aarch64-unknown-linux-gnu/release/fichub
# Typically 15-30 MB for a release build with full optimizations
```

The binary is much larger than the debug build because release mode includes all optimizations and debug symbols. You can strip debug symbols to make it smaller:

```bash
aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/fichub
ls -lh target/aarch64-unknown-linux-gnu/release/fichub
# Now typically 10-15 MB
```

Stripping removes debug information that's only useful for debugging, not for running the application. It's a safe operation that doesn't affect functionality.

## Shipping the Binary: rsync to the Pi

Now that you have an ARM64 binary, you need to get it to the Orange Pi. The simplest way is `rsync` over SSH:

```bash
# First, make sure SSH is set up on the Pi
ssh-copy-id user@192.168.1.100

# Copy the binary
rsync -avz target/aarch64-unknown-linux-gnu/release/fichub \
    user@192.168.1.100:~/

# Copy the migrations
rsync -avz migrations/ user@192.168.1.100:~/migrations/

# Copy the frontend build (if serving from the Pi)
rsync -avz frontend/build/ user@192.168.1.100:~/frontend/build/

# Copy the .env file
rsync -avz .env user@192.168.1.100:~/
```

`rsync` is smart about transfers — it only sends the parts of files that have changed, so subsequent deploys are fast. The `-a` flag preserves permissions and timestamps, `-v` shows what's being transferred, and `-z` compresses the data during transfer.

You can also create a simple deploy script:

```bash
#!/bin/bash
# deploy.sh — cross-compile and ship to the Pi

set -e

PI_HOST="192.168.1.100"
PI_USER="user"

echo "Building for aarch64..."
cargo build --release --target aarch64-unknown-linux-gnu

echo "Deploying binary..."
rsync -avz --progress \
    target/aarch64-unknown-linux-gnu/release/fichub \
    ${PI_USER}@${PI_HOST}:~/

echo "Deploying migrations..."
rsync -avz --progress \
    migrations/ \
    ${PI_USER}@${PI_HOST}:~/migrations/

echo "Deploying frontend..."
rsync -avz --progress \
    frontend/build/ \
    ${PI_USER}@${PI_HOST}:~/frontend/build/

echo "Deploying .env..."
rsync -avz --progress \
    .env \
    ${PI_USER}@${PI_HOST}:~/

echo "Restarting service..."
ssh ${PI_USER}@${PI_HOST} "sudo systemctl restart fichub"

echo "Done! Verify with: curl http://${PI_HOST}:3000/api/"
```

Save this as `deploy.sh`, make it executable (`chmod +x deploy.sh`), and you have a one-command deployment pipeline.

## Creating the Systemd Service File

On the Orange Pi, you want FicHub to start automatically when the machine boots and restart if it crashes. That's what systemd is for.

Create the service file on the Pi:

```bash
sudo nano /etc/systemd/system/fichub.service
```

Here's the service file:

```ini
[Unit]
Description=FicHub - Fanfiction Download Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/home/fichub
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5

# Environment
EnvironmentFile=/home/fichub/.env

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/fichub/cache /home/fichub/tmp
PrivateTmp=true

# Resource limits
LimitNOFILE=65536
MemoryMax=512M

[Install]
WantedBy=multi-user.target
```

Let's break this down section by section.

### [Unit] Section

```ini
[Unit]
Description=FicHub - Fanfiction Download Server
After=network.target postgresql.service redis.service
Wants=postgresql.service redis.service
```

The `Description` is what shows up in `systemctl status` and journal logs. The `After` line tells systemd to start FicHub after the network is up and after PostgreSQL and Redis are running. The `Wants` line means PostgreSQL and Redis are "wanted" (preferred) but not strictly required — if they're not installed, FicHub still starts (it'll fail to connect to the database, but at least systemd doesn't complain about missing dependencies).

This is important for boot ordering. Without `After=network.target`, FicHub might try to start before the network interface is configured, which would cause the bind to fail.

### [Service] Section

```ini
[Service]
Type=simple
User=fichub
Group=fichub
WorkingDirectory=/home/fichub
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5
```

- **`Type=simple`** — The process runs in the foreground (which is exactly what Axum does with `axum::serve().await`). systemd tracks the process by its PID.

- **`User=fichub`** and **`Group=fichub`** — Run as a dedicated `fichub` user, not root. This is a security best practice. If FicHub is compromised, the attacker only gets the permissions of the `fichub` user, not the entire system.

- **`WorkingDirectory=/home/fichub`** — Sets the working directory to where the binary and data live. This is important because FicHub's default paths (like `./cache` and `./frontend/build`) are relative to the working directory.

- **`ExecStart=/home/fichub/fichub`** — The full path to the FicHub binary.

- **`Restart=always`** — If the process crashes, systemd restarts it. This is crucial for a server that should be always-on.

- **`RestartSec=5`** — Wait 5 seconds before restarting. This prevents rapid restart loops — if FicHub keeps crashing immediately (say, because the database is down), it won't restart more than once every 5 seconds.

### Environment File

```ini
EnvironmentFile=/home/fichub/.env
```

This loads environment variables from a file, just like `dotenvy::dotenv()` does in the Rust code. The `.env` file on the Pi might look like:

```bash
DATABASE_URL=postgres://fichub:secretpassword@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=/home/fichub/cache
TMP_DIR=/home/fichub/tmp
PORT=3000
FRONTEND_DIR=/home/fichub/frontend/build
RUST_LOG=info,fichub=debug
```

Notice the `DATABASE_URL` uses `localhost` instead of `postgres` — this is a bare-metal deployment, not Docker, so PostgreSQL runs on the same machine.

### Security Hardening

```ini
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/fichub/cache /home/fichub/tmp
PrivateTmp=true
```

These lines limit what FicHub can do if it's compromised:

- **`NoNewPrivileges`** — Prevents the process from gaining new privileges (like setuid). This is a defense-in-depth measure.

- **`ProtectSystem=strict`** — Makes the entire filesystem read-only except explicitly listed paths. FicHub can't write to `/etc`, `/var`, or any other system directory.

- **`ProtectHome=read-only`** — Makes `/home` read-only. FicHub can read its own home directory but not write to it directly — except for the explicitly listed paths.

- **`ReadWritePaths`** — Explicitly allows writing to cache and tmp directories. These are the only writable locations outside of tmpfs.

- **`PrivateTmp=true`** — Gives the process its own `/tmp` (a tmpfs mount). This prevents `/tmp`-based attacks — if an attacker creates a file in `/tmp`, FicHub won't see it.

### Resource Limits

```ini
LimitNOFILE=65536
MemoryMax=512M
```

- **`LimitNOFILE`** — Sets the maximum number of open file descriptors to 65536. This is important for handling many concurrent connections — each open socket, database connection, and file handle counts as an open file descriptor. The default (usually 1024) is too low for a server.

- **`MemoryMax`** — Caps memory usage at 512MB. If FicHub exceeds this (due to a memory leak or unexpected load), systemd kills it and restarts it. This prevents a runaway process from crashing the entire Pi by exhausting memory.

### [Install] Section

```ini
[Install]
WantedBy=multi-user.target
```

This means FicHub starts when the system reaches multi-user mode (the normal boot target for servers). You could use `graphical.target` instead if the Pi has a desktop environment, but for a headless server, `multi-user.target` is correct.

## Enabling and Starting the Service

On the Pi, after creating the service file:

```bash
# Create a dedicated user
sudo useradd -r -s /bin/false fichub
sudo mkdir -p /home/fichub/cache /home/fichub/tmp /home/fichub/frontend/build
sudo chown -R fichub:fichub /home/fichub

# Copy files (from the deploy script or manually)
sudo cp /home/user/fichub /home/fichub/
sudo cp /home/user/.env /home/fichub/
sudo cp -r /home/user/migrations /home/fichub/
sudo cp -r /home/user/frontend/build/* /home/fichub/frontend/build/
sudo chown -R fichub:fichub /home/fichub

# Reload systemd (after creating/editing service files)
sudo systemctl daemon-reload

# Enable (start on boot)
sudo systemctl enable fichub

# Start immediately
sudo systemctl start fichub

# Check status
sudo systemctl status fichub
```

The `daemon-reload` command tells systemd to re-read its configuration files. This is necessary after creating or editing any service file. Without it, systemd continues using the old configuration.

The `enable` command creates symlinks in the systemd directories so FicHub starts automatically on boot. It doesn't start the service — it just ensures it will be started in the future.

The `start` command starts FicHub immediately. You should see output like:

```
● fichub.service - FicHub - Fanfiction Download Server
     Loaded: loaded (/etc/systemd/system/fichub.service; enabled; ...)
     Active: active (running) since ...
   Main PID: 12345 (fichub)
      Tasks: 8 (limit: 4589)
     Memory: 45.2M
        CPU: 1.234s
     CGroup: /system.slice/fichub.service
             └─12345 /home/fichub/fichub
```

The `active (running)` status confirms that FicHub started successfully. The PID and memory usage are shown.

## Verifying with curl

After the service is running, verify it works:

```bash
# From the Pi itself
curl http://localhost:3000/api/

# From another machine on the network
curl http://192.168.1.100:3000/api/

# Check the API documentation
curl http://192.168.1.100:3000/api/ | python3 -m json.tool

# Try the remote endpoint
curl http://192.168.1.100:3000/api/v0/remote

# Check the frontend
curl -I http://192.168.1.100:3000/
```

If you get JSON responses back, FicHub is running and accepting requests. The `-I` flag on the frontend request shows just the headers, confirming that `Content-Type: text/html` is returned.

For more detailed verification:

```bash
# Check logs
sudo journalctl -u fichub -f

# Check the service is healthy
sudo systemctl is-active fichub

# Check PostgreSQL connection
sudo -u fichub psql -h localhost -U fichub -d fichub -c "SELECT count(*) FROM fic_info;"

# Test an export
curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/12345"

# Check disk usage
du -sh /home/fichub/cache/

# Check memory usage
ps aux | grep fichub

# Check open file descriptors
ls /proc/$(pgrep fichub)/fd | wc -l
```

## The Deployment Checklist

Here's a complete checklist for deploying FicHub to an Orange Pi (or any ARM64 Linux machine):

### One-Time Setup (on the Pi)

- [ ] Install an ARM64 Linux distribution (Armbian, Manjaro ARM, Ubuntu ARM)
- [ ] Update the system: `sudo apt update && sudo apt upgrade`
- [ ] Install PostgreSQL: `sudo apt install postgresql`
- [ ] Install Redis: `sudo apt install redis-server`
- [ ] Create the database: `sudo -u postgres createdb fichub`
- [ ] Create database user: `sudo -u postgres createuser -P fichub`
- [ ] Grant permissions: `sudo -u postgres psql -c "GRANT ALL ON DATABASE fichub TO fichub;"`
- [ ] Create the fichub user: `sudo useradd -r -s /bin/false fichub`
- [ ] Create directories: `sudo mkdir -p /home/fichub/{cache,tmp,frontend/build,migrations}`
- [ ] Set permissions: `sudo chown -R fichub:fichub /home/fichub`
- [ ] Install `libpq` and `ca-certificates`: `sudo apt install libpq-dev ca-certificates`

### Cross-Compile (on your dev machine)

- [ ] Install the aarch64 target: `rustup target add aarch64-unknown-linux-gnu`
- [ ] Install the cross-linker: `sudo pacman -S aarch64-linux-gnu-gcc` (or equivalent)
- [ ] Configure `.cargo/config.toml` with the linker path
- [ ] Build: `cargo build --release --target aarch64-unknown-linux-gnu`
- [ ] Verify: `file target/aarch64-unknown-linux-gnu/release/fichub`
- [ ] Strip debug symbols: `aarch64-linux-gnu-strip target/aarch64-unknown-linux-gnu/release/fichub`

### Deploy (rsync)

- [ ] Copy the binary: `rsync target/aarch64-unknown-linux-gnu/release/fichub user@pi:~/`
- [ ] Copy migrations: `rsync -r migrations/ user@pi:~/migrations/`
- [ ] Copy frontend build: `rsync -r frontend/build/ user@pi:~/frontend/build/`
- [ ] Copy `.env` file: `rsync .env user@pi:~/`
- [ ] Set permissions: `ssh user@pi "sudo chown -R fichub:fichub /home/fichub"`

### Configure (on the Pi)

- [ ] Create systemd service file at `/etc/systemd/system/fichub.service`
- [ ] `sudo systemctl daemon-reload`
- [ ] `sudo systemctl enable fichub`
- [ ] `sudo systemctl start fichub`

### Verify

- [ ] `curl http://pi-ip:3000/api/` returns JSON
- [ ] `curl http://pi-ip:3000/` returns the frontend
- [ ] `sudo systemctl status fichub` shows `active (running)`
- [ ] `sudo journalctl -u fichub` shows no errors
- [ ] Test a fic export end-to-end

## Updating: Rebuild, Rsync, Restart

Once FicHub is deployed, updating is straightforward:

1. Pull latest code: `git pull`
2. Build: `cargo build --release --target aarch64-unknown-linux-gnu`
3. Deploy: `./deploy.sh`
4. The systemd service restarts automatically

Or if you wrote the `deploy.sh` script from earlier, it's literally one command:

```bash
./deploy.sh
```

The script builds, copies files, and restarts the service. From code change to deployed update, it takes about 30 seconds (assuming no dependency changes).

### Zero-Downtime Updates

For zero-downtime updates, you could run two instances of FicHub on different ports and use nginx to switch between them:

```bash
# Build the new binary
cargo build --release --target aarch64-unknown-linux-gnu

# Copy it with a different name
rsync target/aarch64-unknown-linux-gnu/release/fichub user@pi:~/fichub-new

# Start the new instance on a different port
ssh user@pi "PORT=3001 /home/fichub/fichub-new &"

# Switch nginx to the new port
ssh user@pi "sudo sed -i 's/3000/3001/' /etc/nginx/sites-enabled/fichub"
ssh user@pi "sudo nginx -s reload"

# Stop the old instance
ssh user@pi "sudo systemctl stop fichub"

# Move the new binary into place
ssh user@pi "mv /home/fichub/fichub-new /home/fichub/fichub"

# Update the service file to use port 3001 (or revert to 3000)
ssh user@pi "sudo systemctl start fichub"
```

This is more complex but ensures users never see a downtime blip.

### Health Check Script

For monitoring, you can set up a simple health check:

```bash
#!/bin/bash
# healthcheck.sh — check if FicHub is running

RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/)

if [ "$RESPONSE" != "200" ]; then
    echo "FicHub is down! Response code: $RESPONSE"
    sudo systemctl restart fichub
    echo "Restarted fichub"
fi
```

Run this from a cron job every 5 minutes:

```bash
crontab -e
*/5 * * * * /home/fichub/healthcheck.sh >> /var/log/fichub-health.log 2>&1
```

## Performance on the Orange Pi 5

The Orange Pi 5 with its RK3588S processor handles FicHub surprisingly well:

- **API responses** are sub-100ms for cached requests
- **First-time exports** take 2-5 seconds depending on fic length
- **Concurrent requests** are handled well thanks to Tokio's async runtime
- **Memory usage** typically stays under 100MB
- **CPU usage** is minimal at idle (a few percent)
- **Disk usage** depends on cached exports (each EPUB is 1-10MB)

For a personal fanfiction server, the Orange Pi 5 is more than enough. You could serve dozens of concurrent users without breaking a sweat. The RK3588S's 8 cores handle Tokio's thread pool easily, and the 8-16GB of RAM provides plenty of headroom.

### Monitoring Resource Usage

Keep an eye on resource usage with these commands:

```bash
# Memory
free -h

# Disk
df -h /home/fichub/cache/

# CPU
top -p $(pgrep fichub)

# Network connections
ss -tlnp | grep 3000

# Database size
sudo -u fichub psql -h localhost -U fichub -d fichub \
    -c "SELECT pg_size_pretty(pg_database_size('fichub'));"
```

## 🧪 Try It Yourself

1. Install the cross-compilation toolchain on your dev machine
2. Build for ARM64: `cargo build --release --target aarch64-unknown-linux-gnu`
3. Verify the binary: `file target/aarch64-unknown-linux-gnu/release/fichub`
4. If you have access to an ARM64 machine, copy the binary and run it
5. Try the deploy script workflow end-to-end
6. Monitor the service with `journalctl -u fichub -f` and watch the logs

## ⚠️ Watch Out

The most common cross-compilation issue is missing system libraries. If you see errors like "cannot find -lpq" or "cannot find -lssl", you need to install the ARM64 versions of those libraries on your cross-compilation machine. On Arch Linux: `sudo pacman -S aarch64-linux-gnu-postgresql-libs aarch64-linux-gnu-openssl`. On Ubuntu: `sudo apt install libpq-dev:arm64`.

Another pitfall: forgetting to install the runtime dependencies on the target machine. The cross-compiled binary is linked against `libpq` and `libssl`, so the Orange Pi needs those libraries installed. Run `sudo apt install libpq-dev ca-certificates` on the Pi before deploying.

A third issue: the `.env` file on the Pi uses `localhost` in the `DATABASE_URL`, not `postgres` as in Docker. The Docker network hostname `postgres` only works inside Docker Compose. On a bare metal deployment, it's `localhost` or the actual IP address of the database server.

A fourth issue: file permissions. If the `fichub` user can't read the binary, the `.env` file, or the frontend build, the service will fail silently. Always check `ls -la /home/fichub/` to verify permissions.

A fifth issue: the `ExecStart` path in the service file must be the full path to the binary. A relative path won't work because systemd changes to the `WorkingDirectory` before starting the process, but `ExecStart` is evaluated before that change takes effect.

---

# Summary

Part 6 brought everything together — from the internal architecture of the Axum router to Docker containers to cross-compiled deployments on ARM hardware.

## Chapter 25 Recap

The Axum router is the central nervous system of FicHub. The `build_router` function in `server.rs` creates a tree of routes that covers everything: API endpoints for export and metadata, cache download routes, recommendation and tagging APIs, curatorial tools, search, OPDS catalog feeds, and legacy redirects. The `AppState` struct bundles all shared resources (database pools, Redis connections, scraper registries, rate limiters) into a single `Arc` that's accessible from every handler. The `run()` function ties it all together — connecting to services, building state, constructing the router, binding to a port, and serving requests.

The key design decisions in the router are worth noting:
- Routes are organized by API version (`/api/v0/`) for future-proofing
- The OPDS routes provide e-reader integration without any additional setup
- Legacy redirects maintain backward compatibility with old bookmarks
- The SPA fallback ensures the SvelteKit frontend works for any URL
- Middleware layers (logging and CORS) are applied to all requests uniformly

## Chapter 26 Recap

FicHub serves its SvelteKit frontend directly from the Rust server using `tower-http`'s `ServeDir` and `ServeFile`. The SPA routing pattern — serve `index.html` for any path that doesn't match a real file — lets the SvelteKit JavaScript router handle client-side navigation. Immutable assets get long cache lifetimes thanks to content-hashed filenames, while `index.html` should never be cached.

The frontend module is intentionally minimal — just a constant for the default build directory. All the heavy lifting is done by `tower-http`, which provides efficient, async file serving with proper MIME type detection and conditional request support.

## Chapter 27 Recap

Docker makes deployment reproducible and portable. The multi-stage Dockerfile compiles Rust in a full build image and copies just the binary into a slim runtime image — reducing the final image from 1+ GB to 50-100 MB. Docker Compose orchestrates PostgreSQL, Redis, Calibre, and the FicHub app, with named volumes for persistence and healthchecks for proper startup ordering.

The Calibre sidecar pattern is a great example of the microservice approach — isolate heavy dependencies in separate containers and communicate through shared volumes. This keeps the main image lean while still supporting all export formats.

## Chapter 28 Recap

Cross-compilation lets you build ARM64 binaries on x86_64 machines with just a target add and a linker configuration. Deploying to an Orange Pi 5 via rsync and systemd gives you a lean, efficient personal server that starts on boot, restarts on crash, and handles security hardening out of the box. The deploy script reduces the update workflow to a single command.

The systemd service file demonstrates production-grade process management: dedicated users, security hardening, resource limits, automatic restarts, and dependency ordering. Combined with the health check script and cron job, you get a self-healing deployment that monitors itself and recovers from failures.

---

In the next part, we'll look at monitoring, testing, and operational concerns — how to keep FicHub running smoothly in production. We'll cover structured logging, Prometheus metrics, integration testing, and operational runbooks for common issues like database migrations, cache eviction, and rate limit tuning.
