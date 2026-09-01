# Extended Content: Comprehensive Walkthroughs

---

# Complete Code Walkthrough: Export Handler

The export handler is the most complex handler in FicHub. Let's walk through every line of the actual implementation.

## The Export Query Structure

```rust
#[derive(Deserialize)]
pub struct ExportQuery {
    pub q: String,  // The URL to export
}
```

This struct defines the query parameters for the export endpoint. The `#[derive(Deserialize)]` attribute tells serde to automatically generate code for deserializing JSON or query parameters into this struct. The `q` field contains the URL of the story to export.

## The Export Handler Function

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
    
    tracing::info!(
        ip = %remote_addr.ip(),
        url = %query.q,
        "Export request received"
    );
```

**Line-by-line breakdown:**

1. `pub async fn epub_handler(` — This is an async function that handles HTTP requests. The `pub` keyword makes it accessible from other modules.

2. `State(state): State<Arc<AppState>>` — This is an Axum extractor that pulls the shared application state from the request. The `Arc<AppState>` type means the state is atomically reference-counted, allowing safe sharing across async tasks.

3. `Query(query): Query<ExportQuery>` — This extractor parses the URL query string into an `ExportQuery` struct. If the parsing fails (missing required field, wrong type), Axum automatically returns a 400 Bad Request error.

4. `ConnectInfo(remote_addr): ConnectInfo<SocketAddr>` — This extractor gets the client's IP address and port from the TCP connection info.

5. `-> Result<Json<Value>, AppError>` — The function returns a Result that either contains a JSON response or an AppError. Axum automatically converts this into an HTTP response.

6. `let start = std::time::Instant::now();` — We capture the current time to measure how long the request takes.

7. `tracing::info!(...)` — We log the request with the client's IP and the URL being exported. The `tracing` crate provides structured logging with key-value pairs.

## Rate Limiting

```rust
    // Rate limit check
    let client_ip = remote_addr.ip();
    match state.rate_limiter.check(client_ip, &query.q).await? {
        RateLimitResult::Allowed => {},
        RateLimitResult::Wait(secs) => {
            tracing::warn!(ip = %client_ip, wait = secs, "Rate limited");
            return Err(AppError::RateLimited(secs));
        }
        RateLimitResult::Blocked => {
            tracing::warn!(ip = %client_ip, "IP blocked");
            return Err(AppError::BadRequest(-403, "blocked".into()));
        }
    }
```

**Line-by-line breakdown:**

1. `let client_ip = remote_addr.ip();` — Extract just the IP address from the SocketAddr.

2. `match state.rate_limiter.check(client_ip, &query.q).await?` — Call the rate limiter's check method. The `.await` keyword suspends execution until the async operation completes. The `?` operator propagates any errors.

3. `RateLimitResult::Allowed => {}` — If the request is allowed, do nothing (empty block).

4. `RateLimitResult::Wait(secs) => { ... }` — If the client needs to wait, log a warning and return a 429 Too Many Requests error.

5. `RateLimitResult::Blocked => { ... }` — If the IP is blocked (e.g., datacenter IP), return a 403 Forbidden error.

## Scraper Lookup

```rust
    // Find the right scraper
    let scraper = state.scraper_registry.find_scraper(&query.q)
        .ok_or_else(|| {
            tracing::warn!(url = %query.q, "No scraper found");
            AppError::BadRequest(-1, "unsupported URL".into())
        })?;
    
    // Fetch metadata
    let meta = scraper.lookup(&state.http_client, &query.q).await?;
    tracing::info!(
        url_id = %meta.url_id,
        title = %meta.title,
        author = %meta.author,
        chapters = meta.chapters,
        words = meta.words,
        "Metadata fetched"
    );
```

**Line-by-line breakdown:**

1. `state.scraper_registry.find_scraper(&query.q)` — The registry iterates through all registered scrapers and calls `can_handle()` on each one. The first scraper that returns `true` for this URL is returned.

2. `.ok_or_else(|| { ... })?` — Convert the Option to a Result. If no scraper was found, create an error with a descriptive message. The `?` operator propagates the error if it occurs.

3. `scraper.lookup(&state.http_client, &query.q).await?` — Call the scraper's lookup method to fetch the story's metadata. This makes an HTTP request to the fanfiction site and parses the HTML.

4. `tracing::info!(...)` — Log the fetched metadata for debugging and analytics.

## Database Operations

```rust
    // Upsert in database
    let fic_info = FicInfo::from_metadata(&meta);
    queries::upsert_fic_info(&state.db, &fic_info).await?;
    
    // Check blacklist
    if !queries::check_fic_blacklist(&state.db, &meta.url_id).await?.is_empty() {
        tracing::warn!(url_id = %meta.url_id, "Story is blacklisted");
        return Err(AppError::BadRequest(-403, "story is blacklisted".into()));
    }
```

**Line-by-line breakdown:**

1. `FicInfo::from_metadata(&meta)` — Convert the scraper's FicMetadata into a database FicInfo model.

2. `queries::upsert_fic_info(&state.db, &fic_info).await?` — Insert or update the story in the database. If the story already exists (same ID), update the metadata.

3. `queries::check_fic_blacklist(&state.db, &meta.url_id).await?` — Check if this story or author is blacklisted.

4. `if !...is_empty()` — If the blacklist check returned any results, the story is blocked.

## Cache Check

```rust
    // Compute input hash
    let input_hash = compute_input_hash(&meta);
    
    // Check cache
    if let Some(cached) = queries::find_export_log(
        &state.db, &meta.url_id, 1, "epub", &input_hash
    ).await? {
        let elapsed = start.elapsed().as_millis();
        tracing::info!(
            url_id = %meta.url_id,
            elapsed_ms = elapsed,
            "Cache hit"
        );
        let download_url = format!(
            "/cache/epub/{}?h={}",
            meta.url_id, cached.export_hash
        );
        return Ok(Json(json!({
            "err": 0,
            "url_id": meta.url_id,
            "meta": meta,
            "urls": {"epub": download_url}
        })));
    }
```

**Line-by-line breakdown:**

1. `compute_input_hash(&meta)` — Compute an MD5 hash of the metadata. This hash changes if the story is updated, causing a cache miss.

2. `queries::find_export_log(...)` — Check the database for a cached export with matching url_id, version, etype, and input_hash.

3. `if let Some(cached) = ...` — If a cached export exists, return it immediately without regenerating.

4. `let download_url = format!("/cache/epub/{}?h={}", ...)` — Build the download URL with the content hash for cache validation.

5. `return Ok(Json(json!({...})))` — Return early with the cached result.

## Export Generation

```rust
    // Acquire semaphore (prevent duplicate exports for same story)
    let semaphore = get_export_semaphore(
        &state.cache_semaphores, &meta.url_id, EType::Epub
    ).await;
    let _permit = semaphore.acquire().await.unwrap();
    
    // Double-check cache (another request might have finished while we waited)
    if let Some(cached) = queries::find_export_log(
        &state.db, &meta.url_id, 1, "epub", &input_hash
    ).await? {
        let download_url = format!(
            "/cache/epub/{}?h={}",
            meta.url_id, cached.export_hash
        );
        return Ok(Json(json!({
            "err": 0,
            "url_id": meta.url_id,
            "meta": meta,
            "urls": {"epub": download_url}
        })));
    }
```

**Line-by-line breakdown:**

1. `get_export_semaphore(...)` — Get or create a semaphore for this (url_id, etype) pair. Semaphores prevent multiple concurrent exports for the same story.

2. `semaphore.acquire().await.unwrap()` — Wait for the semaphore to become available. Only one export can run at a time for each story.

3. `if let Some(cached) = ...` — After acquiring the semaphore, check the cache again. Another request might have completed the export while we were waiting.

## Chapter Fetching and Export

```rust
    // Fetch chapters
    let export_start = std::time::Instant::now();
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
    tracing::info!(
        url_id = %meta.url_id,
        chapter_count = chapters.len(),
        "Chapters fetched"
    );
    
    // Generate EPUB
    let (epub_path, epub_hash) = export::epub::generate_epub(
        &state.config.tmp_dir, &meta, &chapters
    )?;
    let export_ms = export_start.elapsed().as_millis() as i32;
    
    // Generate HTML bundle
    let (html_path, html_hash) = export::html_bundle::generate_html_bundle(
        &state.config.tmp_dir, &meta, &chapters
    )?;
```

**Line-by-line breakdown:**

1. `scraper.fetch_chapters(...)` — Fetch all chapter content from the fanfiction site. For AO3, this is a single request for the full work view. For FF.net, it's one request per chapter.

2. `export::epub::generate_epub(...)` — Generate an EPUB file from the metadata and chapters. Returns the file path and MD5 hash.

3. `export::html_bundle::generate_html_bundle(...)` — Generate an HTML bundle (ZIP with HTML file) from the same content.

## Cache Storage

```rust
    // Move to cache
    let epub_cache = cache::disk::cache_path(
        &state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash
    );
    let html_cache = cache::disk::cache_path(
        &state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash
    );
    
    fs::create_dir_all(epub_cache.parent().unwrap())?;
    fs::rename(&epub_path, &epub_cache)?;
    fs::create_dir_all(html_cache.parent().unwrap())?;
    fs::rename(&html_path, &html_cache)?;
```

**Line-by-line breakdown:**

1. `cache::disk::cache_path(...)` — Compute the cache file path based on the export type, url_id, and hash.

2. `fs::create_dir_all(...)` — Create the directory structure if it doesn't exist.

3. `fs::rename(...)` — Atomically move the generated file to the cache location. Atomic renames prevent partial files from being served.

## Database Recording

```rust
    // Record in database
    queries::insert_export_log(
        &state.db, &meta.url_id, 1, "epub", &input_hash, &epub_hash
    ).await?;
    queries::insert_export_log(
        &state.db, &meta.url_id, 1, "html", &input_hash, &html_hash
    ).await?;
    
    // Log request
    let total_ms = start.elapsed().as_millis() as i32;
    queries::insert_request_log(
        &state.db, 1, "epub", &query.q, total_ms - export_ms,
        Some(&meta.url_id), None, Some(export_ms),
        None, Some(&epub_hash), None,
    ).await?;
```

**Line-by-line breakdown:**

1. `queries::insert_export_log(...)` — Record the export in the database so future requests can find it in the cache.

2. `queries::insert_request_log(...)` — Log the request for analytics. This records the timing, URL, and export hash.

## Response

```rust
    tracing::info!(
        url_id = %meta.url_id,
        epub_hash = %epub_hash,
        html_hash = %html_hash,
        export_ms = export_ms,
        total_ms = total_ms,
        "Export complete"
    );
    
    let epub_url = format!("/cache/epub/{}?h={}", meta.url_id, epub_hash);
    let html_url = format!("/cache/html/{}?h={}", meta.url_id, html_hash);
    
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "meta": meta,
        "urls": {
            "epub": epub_url,
            "html": html_url
        }
    })))
}
```

**Line-by-line breakdown:**

1. `tracing::info!(...)` — Log the completed export with timing information.

2. Build the download URLs with content hashes for cache validation.

3. Return the JSON response with the story metadata and download URLs.

---

# Complete Code Walkthrough: Search Handler

The search handler provides full-text search with tag filters, word count ranges, and more.

## The Search Query Structure

```rust
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tags: Option<String>,
    pub exclude_tags: Option<String>,
    pub any_tags: Option<String>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
}
```

All fields are `Option<T>` because they're optional query parameters. Clients can search with any combination of filters.

## The Search Handler

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
    // Parse tag filters
    let include_tags = query.tags
        .map(|t| parse_tag_filters(&t))
        .transpose()
        .map_err(|e| AppError::BadRequest(-1, e))?;
    
    let exclude_tags = query.exclude_tags
        .map(|t| parse_tag_filters(&t))
        .transpose()
        .map_err(|e| AppError::BadRequest(-1, e))?;
    
    let include_any_tags = query.any_tags
        .map(|t| parse_tag_filters(&t))
        .transpose()
        .map_err(|e| AppError::BadRequest(-1, e))?;
```

**Line-by-line breakdown:**

1. `query.tags.map(|t| parse_tag_filters(&t))` — If tags were provided, parse them into a Vec<TagFilter>.

2. `.transpose()` — Convert Option<Result<Vec, Error>> to Result<Option<Vec>, Error>.

3. `.map_err(|e| AppError::BadRequest(-1, e))` — Convert parsing errors to AppError.

## Building the Query

```rust
    let params = SearchParams {
        q: query.q,
        include_tags: include_tags.unwrap_or_default(),
        exclude_tags: exclude_tags.unwrap_or_default(),
        include_any_tags: include_any_tags.unwrap_or_default(),
        min_words: query.min_words,
        max_words: query.max_words,
        complete: query.complete,
        source: query.source,
        sort: query.sort,
        page: query.page,
    };
    
    let builder = SearchQueryBuilder::new(params, state.config.search_max_per_page);
```

**Line-by-line breakdown:**

1. Build a `SearchParams` struct from the parsed query parameters.

2. Create a `SearchQueryBuilder` that will construct the SQL query.

## Executing the Query

```rust
    // Execute count query
    let mut count_qb = builder.build_count_query();
    let count: (i64,) = count_qb
        .build_query_as()
        .fetch_one(&state.db)
        .await?;
    
    // Execute data query
    let mut data_qb = builder.build_data_query();
    let rows = data_qb
        .build_query_as::<FicSearchRow>()
        .fetch_all(&state.db)
        .await?;
    
    let results: Vec<SearchResult> = rows.into_iter().map(|r| r.into()).collect();
```

**Line-by-line breakdown:**

1. `builder.build_count_query()` — Build a SQL query that counts matching results.

2. `builder.build_data_query()` — Build a SQL query that fetches matching stories.

3. Execute both queries against the database.

4. Convert the database rows into SearchResult structs.

## Returning Results

```rust
    Ok(Json(json!({
        "err": 0,
        "total": count.0,
        "page": builder.page(),
        "per_page": state.config.search_max_per_page,
        "results": results
    })))
}
```

Return the search results with pagination information.

---

# Complete Code Walkthrough: Tag Resolution

The tag resolution system maps user-submitted tag strings to canonical tags.

## The Resolution Process

```rust
pub async fn resolve_tag(
    pool: &PgPool,
    name: &str,
    tag_type_id: i16,
) -> AppResult<TagResolution> {
    // 1. Check for exact match
    if let Some((id, name, type_id)) = queries::lookup_tag_by_name(pool, name).await? {
        return Ok(TagResolution {
            tag_id: id,
            tag_name: name,
            tag_type_id: type_id,
            is_new: false,
        });
    }
    
    // 2. Check for alias
    if let Some(canonical_id) = queries::lookup_alias(pool, name).await? {
        let (id, name, type_id) = queries::lookup_tag_by_id(pool, canonical_id).await?;
        return Ok(TagResolution {
            tag_id: id,
            tag_name: name,
            tag_type_id: type_id,
            is_new: false,
        });
    }
    
    // 3. Create new tag
    let id = queries::create_tag(pool, name, tag_type_id).await?;
    Ok(TagResolution {
        tag_id: id,
        tag_name: name.to_string(),
        tag_type_id,
        is_new: true,
    })
}
```

**Line-by-line breakdown:**

1. **Exact match check** — Look for a tag with the exact name. If found, return it.

2. **Alias check** — Look for an alias that maps to a canonical tag. If found, return the canonical tag.

3. **Create new tag** — If no match or alias exists, create a new canonical tag.

This three-step process ensures that:
- Existing tags are reused (no duplicates)
- Aliases are resolved to canonical tags
- New tags are created when needed

---

# Complete Code Walkthrough: Recommendation Engine

The recommendation engine suggests similar stories based on collaborative filtering.

## Getting Recommendations

```rust
pub async fn get_recommendations(
    &self,
    url_id: &str,
    limit: usize,
) -> Result<Vec<Recommendation>, AppError> {
    // 1. Get co-occurring stories
    let cooccurrences = sqlx::query_as::<_, (String, i32)>(
        r#"SELECT story_b, count
           FROM rec_cooccurrence
           WHERE story_a = $1
           ORDER BY count DESC
           LIMIT $2"#
    )
    .bind(url_id)
    .bind((limit * 2) as i64)
    .fetch_all(&self.pool)
    .await?;
```

**Line-by-line breakdown:**

1. Query the co-occurrence table for stories that appear in the same users' favourites.

2. Order by co-occurrence count (most similar first).

3. Fetch twice as many results as needed for filtering.

## Computing Scores

```rust
    let mut recommendations = Vec::new();
    for (rec_url_id, count) in cooccurrences {
        // Get favouriter counts for Jaccard calculation
        let (a_count, b_count) = self.get_favouriter_counts(url_id, &rec_url_id).await?;
        
        let score = compute_score(count, a_count, b_count, 0.0);
        
        // Get story metadata
        if let Some(meta) = self.get_story_meta(&rec_url_id).await? {
            recommendations.push(Recommendation {
                url_id: rec_url_id,
                title: meta.title,
                author: meta.author,
                score,
                reason: format!("{} readers also favourited this", count),
            });
        }
    }
```

**Line-by-line breakdown:**

1. For each co-occurring story, get the number of users who favourited each story.

2. Compute the Jaccard similarity score.

3. Get the story's metadata for the response.

4. Build a Recommendation struct with the score and reason.

## Sorting and Filtering

```rust
    // Sort by score and take top N
    recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    recommendations.truncate(limit);
    
    Ok(recommendations)
}
```

Sort by score (highest first) and take the top N results.

---

# Complete Code Walkthrough: Rate Limiter

The rate limiter uses Redis and Lua scripts for atomic token bucket operations.

## The Rate Limiter Structure

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
}
```

**Field breakdown:**

- `redis` — A multiplexed Redis connection for atomic operations
- `lua_sha` — The SHA hash of the loaded Lua script
- `dynamic_rate_limit` — Whether to use dynamic rate limiting
- `static_delay_base` — Base delay for static rate limiting
- `datacenter_ips` — Set of IPs to block (datacenter IPs)

## The Check Method

```rust
#[async_trait]
impl RateLimiter for RedisBucketLimiter {
    async fn check(
        &self,
        ip: IpAddr,
        url: &str,
    ) -> Result<RateLimitResult, AppError> {
        // Check if IP is a datacenter IP (blocked)
        if self.datacenter_ips.read().await.contains(&ip) {
            return Ok(RateLimitResult::Blocked);
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        
        // Check global rate limit
        let global_key = format!("rate_limit:global");
        let allowed = self.check_bucket(
            &global_key, 60.0, 1.0, now, 1.0
        ).await?;
        if !allowed {
            return Ok(RateLimitResult::Wait(5));
        }
        
        // Check per-IP rate limit
        let ip_key = format!("rate_limit:ip:{}", ip);
        let allowed = self.check_bucket(
            &ip_key, 30.0, 0.5, now, 1.0
        ).await?;
        if !allowed {
            return Ok(RateLimitResult::Wait(10));
        }
        
        // Check per-site rate limit
        let site = self.extract_site(url);
        let site_key = format!("rate_limit:site:{}", site);
        let allowed = self.check_bucket(
            &site_key, 20.0, 0.33, now, 1.0
        ).await?;
        if !allowed {
            return Ok(RateLimitResult::Wait(15));
        }
        
        Ok(RateLimitResult::Allowed)
    }
}
```

**Line-by-line breakdown:**

1. **Datacenter IP check** — If the IP is in the datacenter list, block it immediately.

2. **Get current time** — Convert system time to seconds since epoch.

3. **Global rate limit** — Check the global bucket (60 tokens, 1 per second).

4. **Per-IP rate limit** — Check the per-IP bucket (30 tokens, 0.5 per second).

5. **Per-site rate limit** — Check the per-site bucket (20 tokens, 0.33 per second).

6. If all checks pass, the request is allowed.

## The Lua Script

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

-- Get current bucket state
local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

-- Refill tokens based on time elapsed
local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

-- Check if request is allowed
if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

**Line-by-line breakdown:**

1. Get the current bucket state (tokens and last refill time).

2. Calculate how many tokens to add based on time elapsed.

3. Cap tokens at the maximum.

4. If enough tokens, consume one and return 1 (allowed).

5. If not enough tokens, return 0 (denied).

6. Set an expiration key to automatically clean up old buckets.

---

# Complete Code Walkthrough: Disk Cache

The disk cache stores generated files in a hierarchical directory structure.

## Path Computation

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

**Line-by-line breakdown:**

1. Choose the type directory (epub or html).

2. Split the url_id into 3-character chunks for the directory structure.

3. Build the path: `cache_root/type/chunk1/chunk2/chunk3/url_id/hash.suffix`

4. This structure prevents any single directory from having too many files.

## Cache Semaphores

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: EType,
) -> Arc<Semaphore> {
    let mut map = semaphores.lock().await;
    let key = (url_id.to_string(), etype);
    
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

**Line-by-line breakdown:**

1. Lock the semaphore map.

2. Look up the semaphore for this (url_id, etype) pair.

3. If it doesn't exist, create a new semaphore with capacity 1.

4. Return a clone of the Arc<Semaphore>.

This ensures only one export runs at a time for each story, preventing duplicate work.

---

# Complete Code Walkthrough: OPDS Feed Generation

The OPDS catalog allows e-reader apps to browse and download stories.

## Root Catalog

```rust
pub async fn root_catalog() -> impl IntoResponse {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"
      xmlns:opds="http://opds-spec.org/2010/catalog">
    <id>urn:fichub:catalog</id>
    <title>FicHub Catalog</title>
    <updated>2024-01-01T00:00:00Z</updated>
    <link rel="start" href="/opds" type="application/atom+xml;profile=opds-catalog"/>
    
    <entry>
        <title>New Stories</title>
        <link href="/opds/new" type="application/atom+xml;profile=opds-catalog"/>
        <id>urn:fichub:new</id>
        <updated>2024-01-01T00:00:00Z</updated>
    </entry>
    
    <entry>
        <title>Popular Stories</title>
        <link href="/opds/popular" type="application/atom+xml;profile=opds-catalog"/>
        <id>urn:fichub:popular</id>
        <updated>2024-01-01T00:00:00Z</updated>
    </entry>
    
    <entry>
        <title>Browse by Tags</title>
        <link href="/opds/tags" type="application/atom+xml;profile=opds-catalog"/>
        <id>urn:fichub:tags</id>
        <updated>2024-01-01T00:00:00Z</updated>
    </entry>
</feed>"#;
    
    (
        [("Content-Type", "application/atom+xml;profile=opds-catalog")],
        xml
    )
}
```

**Line-by-line breakdown:**

1. Define the XML namespace for Atom and OPDS.

2. Add the root catalog metadata (id, title, updated time).

3. Add navigation entries for New, Popular, and Tags.

4. Set the Content-Type header to OPDS catalog format.

## Story Entry

```rust
fn story_to_opds_entry(story: &FicInfo, epub_url: &str) -> String {
    format!(r#"<entry>
    <title>{}</title>
    <author><name>{}</name></author>
    <id>urn:fichub:{}</id>
    <updated>{}</updated>
    <summary>{}</summary>
    <link rel="alternate" href="/api/v0/meta?url_id={}" type="application/json"/>
    <link rel="http://opds-spec.org/acquisition/open" href="{}" type="application/epub+zip"/>
    <dc:language xmlns:dc="http://purl.org/dc/elements/1.1/">en</dc:language>
    <dc:format xmlns:dc="http://purl.org/dc/elements/1.1/">application/epub+zip</dc:format>
</entry>"#,
        escape_xml(&story.title),
        escape_xml(&story.author),
        story.id,
        story.fic_updated.to_rfc3339(),
        escape_xml(&story.description),
        story.id,
        epub_url,
    )
}
```

**Line-by-line breakdown:**

1. Add the story title and author.

2. Add unique identifiers (URN and updated time).

3. Add a summary (description).

4. Add links to the metadata API and the EPUB download.

5. Add DC (Dublin Core) metadata for language and format.

---

# Complete Code Walkthrough: Database Queries

## Upsert Query

```rust
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated,
            status = EXCLUDED.status,
            extra_meta = EXCLUDED.extra_meta,
            raw_extended_meta = EXCLUDED.raw_extended_meta,
            content_hash = EXCLUDED.content_hash,
            updated = NOW()"#,
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(&fic.author_url)
    .bind(&fic.author_local_id)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.description)
    .bind(fic.fic_created)
    .bind(fic.fic_updated)
    .bind(&fic.status)
    .bind(&fic.source)
    .bind(&fic.extra_meta)
    .bind(&fic.raw_extended_meta)
    .bind(fic.source_id)
    .bind(fic.author_id)
    .bind(&fic.content_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

**Line-by-line breakdown:**

1. Build an INSERT query with all the fic_info columns.

2. Use `$1`, `$2`, etc. as parameter placeholders.

3. `ON CONFLICT (id) DO UPDATE SET ...` — If a row with this ID already exists, update the specified columns.

4. `EXCLUDED.title` refers to the value that was going to be inserted.

5. Bind each field value to the corresponding parameter.

6. Execute the query against the pool.

## Tag Query

```rust
pub async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    
    Ok(rows
        .into_iter()
        .map(|(id, name, type_id, score)| {
            let hidden = score < hidden_threshold;
            (id, name, type_id, score, hidden)
        })
        .collect())
}
```

**Line-by-line breakdown:**

1. Join fic_tags with tags to get the tag name and type.

2. Filter by url_id.

3. Order by tag type, then by score (highest first).

4. Map the results to include a `hidden` flag based on the threshold.

## Search Query Builder

```rust
impl SearchQueryBuilder {
    pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::new(
            "SELECT fi.id, fi.title, fi.author, fi.words FROM fic_info fi WHERE 1=1"
        );
        
        if let Some(ref q) = self.params.q {
            qb.push(" AND to_tsvector('english', fi.title) @@ plainto_tsquery('english', ");
            qb.push_bind(q.clone());
            qb.push(")");
        }
        
        for tag in &self.params.include_tags {
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id WHERE ft.url_id = fi.id AND t.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(" AND t.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(")");
        }
        
        if let Some(min) = self.params.min_words {
            qb.push(" AND fi.words >= ");
            qb.push_bind(min);
        }
        
        if let Some(max) = self.params.max_words {
            qb.push(" AND fi.words <= ");
            qb.push_bind(max);
        }
        
        match self.params.sort.as_deref() {
            Some("-words") => qb.push(" ORDER BY fi.words DESC"),
            Some("-date") => qb.push(" ORDER BY fi.fic_updated DESC"),
            _ => qb.push(" ORDER BY fi.fic_updated DESC"),
        }
        
        let limit = self.max_per_page.min(50) as i64;
        let offset = ((self.page().saturating_sub(1)) as i64) * limit;
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
        
        qb
    }
}
```

**Line-by-line breakdown:**

1. Start with a base SELECT query.

2. If a search query is provided, add full-text search using PostgreSQL's tsvector/tsquery.

3. For each include tag, add an EXISTS subquery.

4. Add word count filters if specified.

5. Add sorting based on the sort parameter.

6. Add pagination with LIMIT and OFFSET.

---

# Complete Code Walkthrough: Error Handling

## The AppError Enum

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

**Variant breakdown:**

- `BadRequest(code, msg)` — Client error with a specific error code
- `RateLimited(retry_after)` — Rate limited, wait N seconds
- `NotFound(msg)` — Resource not found
- `Internal(msg)` — Server error
- `ScrapeError(msg)` — Upstream scraping error
- `ExportError(msg)` — File generation error
- `Database(msg)` — Database error
- `CacheError(msg)` — Cache error

## The Display Implementation

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

## The IntoResponse Implementation

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

**Key design decisions:**

1. **Internal errors log the full message** but return a generic error to the client. This prevents information leakage.

2. **Scrape errors return 502 Bad Gateway** because they represent upstream failures.

3. **Rate limiting returns 429** with a `retry_after` field so the client knows when to retry.

4. **All responses include an `err` field** for consistent error handling on the client side.

## From Implementations

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

These implementations allow the `?` operator to automatically convert common error types to AppError.

---

# Complete Code Walkthrough: Server Startup

## The run Function

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
```

**Line-by-line breakdown:**

1. **Database connection** — Create a connection pool to PostgreSQL.

2. **Redis connection** — Create a multiplexed connection to Redis.

3. **HTTP client** — Build a shared HTTP client for making outgoing requests.

4. **Scraper registry** — Create the registry with all registered scrapers.

5. **Rate limiter** — Initialize the Redis-backed rate limiter.

## State Construction

```rust
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
```

**Line-by-line breakdown:**

1. Create the recommendation engine and collection worker.

2. Wrap everything in an Arc<AppState> for sharing across async tasks.

## Router Construction

```rust
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

**Line-by-line breakdown:**

1. Build the Axum router with all routes and middleware.

2. Bind to the configured address and port.

3. Start the server with `axum::serve`.

4. `into_make_service_with_connect_info` enables the `ConnectInfo` extractor for getting client IP addresses.

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

**Field breakdown:**

- `database_url` — PostgreSQL connection string
- `redis_url` — Redis connection string
- `cache_dir` — Directory for cached export files
- `secondary_cache_dir` — Optional secondary cache location
- `export_version` — Export format version (for cache invalidation)
- `dynamic_rate_limit` — Whether to use dynamic rate limiting
- `node_name` — Server node identifier
- `calibre_container` — Docker container for Calibre conversion
- `tmp_dir` — Temporary directory for export generation
- `app_port` — HTTP server port
- `frontend_dir` — Directory containing the SvelteKit build
- `trusted_proxies` — List of trusted proxy IPs
- `ip_tag_sources` — Sources for IP-based tagging
- `rec_*` — Recommender configuration
- `tag_*` — Tagging configuration
- `search_max_per_page` — Maximum search results per page
- `opds_shelf_token` — Token for OPDS shelf authentication

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

**Pattern breakdown:**

1. **Required variables** — Use `expect()` to panic if not set.

2. **Optional with default** — Use `unwrap_or_else()` to provide a default.

3. **Optional nullable** — Use `ok().filter()` to handle missing or empty values.

4. **Complex parsing** — Use `parse()` with `unwrap_or()` for type conversion.

5. **List parsing** — Use `split()` with `filter()` for comma-separated values.

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

**Derive macro breakdown:**

- `Debug` — Enables debug printing with `{:?}`
- `Clone` — Enables cloning with `.clone()`
- `Serialize` — Enables conversion to JSON
- `Deserialize` — Enables conversion from JSON
- `FromRow` — Enables conversion from database rows

**Field breakdown:**

- `id` — Unique story identifier (SHA-256 hash of source_id + story_id)
- `created` — When the record was created in the database
- `updated` — When the record was last updated
- `title` — Story title
- `author` — Author name
- `author_url` — Author's profile URL on the source site
- `author_local_id` — Author's ID on the source site
- `chapters` — Number of chapters
- `words` — Word count
- `description` — Story summary
- `fic_created` — When the story was published on the source site
- `fic_updated` — When the story was last updated on the source site
- `status` — Story status (ongoing, complete, hiatus, cancelled)
- `source` — Original URL
- `extra_meta` — Additional metadata (JSONB)
- `raw_extended_meta` — Raw extended metadata (JSONB)
- `source_id` — Source site identifier (1=AO3, 2=FF.net, etc.)
- `author_id` — Author ID on the source site
- `content_hash` — Hash of story content for change detection

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

**Field breakdown:**

- `url_id` — Story identifier
- `version` — Export version (for cache invalidation)
- `etype` — Export type (epub, html)
- `input_hash` — MD5 hash of the input metadata
- `export_hash` — MD5 hash of the generated file
- `created` — When the export was generated

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

**Field breakdown:**

- `id` — Tag identifier
- `name` — Tag name (unique)
- `tag_type_id` — Tag type (1=Fandom, 2=Character, 3=Relationship, 4=Freeform, 5=Warning, 6=Category)
- `created` — When the tag was created

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    
    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    
    /// Extract structured tags from a fic URL (optional, default empty)
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

**Method breakdown:**

- `can_handle` — Synchronous method that checks if this scraper handles the given URL. Fast because it just checks URL patterns.

- `lookup` — Async method that fetches the story page and extracts metadata. Makes HTTP requests and parses HTML.

- `fetch_chapters` — Async method that fetches all chapter content. May make multiple HTTP requests.

- `extract_tags` — Optional async method that extracts structured tags. Default implementation returns empty vector.

**Trait bounds:**

- `Send` — The scraper can be sent between threads
- `Sync` — The scraper can be shared between threads
- `async_trait` — Enables async methods in traits

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

**Convenience constructors for each tag type.**

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

**Field breakdown:**

- `url_id` — Unique identifier (SHA-256 hash)
- `title` — Story title
- `author` — Author name
- `chapters` — Number of chapters
- `words` — Word count
- `desc` — Story description
- `published` — Publication timestamp (unix millis)
- `updated` — Last update timestamp (unix millis)
- `status` — Story status
- `source` — Original URL
- `source_id` — Source site identifier
- `author_id` — Author ID on source site
- `author_url` — Author's profile URL
- `author_local_id` — Author's local ID on source site
- `content_hash` — Hash of story content
- `extra_meta` — Additional metadata
- `raw_extended_meta` — Raw extended metadata

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String, // HTML content
}
```

**Field breakdown:**

- `chapter_id` — Chapter number (1-based)
- `title` — Chapter title
- `content` — Chapter content as HTML

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

**Variant breakdown:**

- `NotFound` — Story doesn't exist or is private
- `Blocked` — Site blocked the request
- `Network(e)` — Network error (timeout, connection refused, etc.)
- `ParseError(e)` — HTML parsing error (site changed structure)

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

A simple vector of trait objects. Each scraper is boxed because trait objects have dynamic size.

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

**Method breakdown:**

- `new()` — Creates a new registry with all known scrapers.

- `find_scraper()` — Iterates through scrapers and returns the first one that can handle the URL.

- `lookup()` — Finds the appropriate scraper and calls its lookup method.

- `fetch_chapters()` — Finds the appropriate scraper and calls its fetch_chapters method.

- `scraper_count()` — Returns the number of registered scrapers.

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    // Load configuration
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    // Run the server
    server::run(config).await;
}
```

**Line-by-line breakdown:**

1. `dotenvy::dotenv().ok()` — Load environment variables from .env file if present.

2. `tracing_subscriber::fmt()...` — Initialize structured logging with environment-based log levels.

3. `config::Config::from_env()` — Load configuration from environment variables.

4. `server::run(config).await` — Start the server.

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

**Module breakdown:**

- `cache` — Disk caching and semaphores
- `config` — Configuration from environment variables
- `db` — Database connection, models, queries
- `error` — Application-wide error types
- `export` — EPUB, HTML, MOBI/PDF generation
- `frontend` — Static file serving
- `limiter` — Rate limiting with Redis
- `recommender` — Collaborative filtering engine
- `routes` — HTTP handlers for all API endpoints
- `scrape` — Web scraper trait and implementations
- `search` — Full-text search with dynamic query building
- `server` — AppState, router construction, server binding
- `tags` — Tag resolution, voting, curator tools

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

A unit struct (no fields) with a constant for the base URL.

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

**Line-by-line breakdown:**

1. Create a regex that matches `/works/DIGITS`.

2. Find the first capture group (the work ID).

3. Return the work ID as a String.

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

Simple string containment check.

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    // Extract title
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    // Extract author
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    // Extract word count
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    // Generate url_id
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        // ... other fields ...
    })
}
```

**Line-by-line breakdown:**

1. Extract the work ID from the URL.

2. Build the full work URL with `view_full_work=true`.

3. Make the HTTP request with a descriptive User-Agent.

4. Check the response status.

5. Parse the HTML document.

6. Extract metadata using CSS selectors.

7. Generate the url_id using the SHA-256 hash function.

8. Return the FicMetadata struct.

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    // Fallback for single-chapter works
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

**Line-by-line breakdown:**

1. Fetch the full work view.

2. Parse the HTML document.

3. Select all chapter divs.

4. For each chapter, extract the title and content.

5. If no chapters found (single-chapter work), extract the full work content.

6. Return the chapters vector.

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        // ... other fields ...
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

**Note:** FF.net requires chapter-by-chapter fetching (no full work view).

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    // Count pages for chapter estimation
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0, // XenForo doesn't show word count
        source: url.to_string(),
        source_id: 3,
        // ... other fields ...
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // Collect all post content on this page
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

**Note:** XenForo threads are multi-page, with one post per page. Each page becomes a chapter.

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    // 1. Create a temporary directory for this export
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    // 2. Build the EPUB
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    // 3. Add metadata
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    // 4. Add stylesheet
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    // 5. Add chapters
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    // 6. Generate the EPUB file
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    // 7. Compute MD5 hash for caching
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    // 8. Clean up the temporary directory
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

**Line-by-line breakdown:**

1. Create a temporary directory with a UUID name.

2. Initialize the EPUB builder with a ZIP library.

3. Set metadata (title, author, description).

4. Add the CSS stylesheet.

5. Add each chapter as an XHTML file.

6. Generate the EPUB file.

7. Compute the MD5 hash for caching.

8. Clean up and return the path and hash.

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: Georgia, serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
        }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

**Example:**

Given:
- `cache_root` = `/cache`
- `etype` = `EType::Epub`
- `url_id` = `abc123456def`
- `hash` = `a1b2c3d4e5f6`

The result is:
```
/cache/epub/abc/123/456/abc123456def/a1b2c3d4e5f6.epub
```

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

-- Get current bucket state
local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

-- Refill tokens based on time elapsed
local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

-- Check if request is allowed
if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

**Line-by-line breakdown:**

1. Get the bucket state (tokens and last refill time).

2. Calculate elapsed time since last refill.

3. Add tokens based on elapsed time and refill rate.

4. Cap tokens at maximum.

5. If enough tokens, consume one and return 1 (allowed).

6. If not enough, return 0 (denied).

7. Set expiration to clean up old buckets.

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
-- Core fic metadata
CREATE TABLE IF NOT EXISTS fic_info (
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

-- Export log (cache tracking)
CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

-- Request tracking
CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

-- Blacklists
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

-- Cache version bumps
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
-- Users processed by collection worker
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

-- User favourites
CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

-- Co-occurrence counts
CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

-- Precomputed recommendations
CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

-- Suggestion queue
CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

-- Votes on recommendations
CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
-- Canonical tags
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Tag aliases
CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

-- Story-tag associations
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

-- Votes on tags
CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

-- Flagged tags
CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

-- Tag type definitions
CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

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
        
        // Cache download routes
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        // Recommender routes
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        // Tag routes
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        // Curator routes
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        // Search route
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        // OPDS routes
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        // Legacy redirect routes
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
```

**Route breakdown:**

1. **API routes** — Documentation, export, metadata, client info
2. **Cache routes** — Download cached files
3. **Recommender routes** — Get recommendations, suggest stories, vote
4. **Tag routes** — Submit, vote, and flag tags
5. **Curator routes** — Admin tools for tag management
6. **Search route** — Full-text search
7. **OPDS routes** — E-reader catalog feeds
8. **Legacy routes** — Redirect old URLs
9. **Static frontend** — Serve the SvelteKit build
10. **Middleware** — Request tracing and CORS
11. **Shared state** — Application state for all handlers

---

# Complete Code Walkthrough: AppState Construction

## The run Function

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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
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

**Startup sequence:**

1. Initialize database connection pool
2. Initialize Redis connection
3. Build HTTP client
4. Create scraper registry
5. Initialize rate limiter
6. Load datacenter IPs
7. Create recommendation engine and collection worker
8. Wrap everything in AppState
9. Build the router
10. Bind to address and start serving

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

**Example:**

Given:
- `cache_root` = `/cache`
- `etype` = `EType::Epub`
- `url_id` = `abc123456def`
- `hash` = `a1b2c3d4e5f6`

The result is:
```
/cache/epub/abc/123/456/abc123456def/a1b2c3d4e5f6.epub
```

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
        .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "network error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| e.to_string())
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    score REAL NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: null,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.0.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.0.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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
    fic_updated TIMESTAMPTZ DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    source_id INT8,
    author_id INT8,
    content_hash TEXT,
    extra_meta JSONB,
    raw_extended_meta JSONB
);

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT EXISTS idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INT4 NOT NULL REFERENCES tags(id),
    created TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score INT2 DEFAULT 0,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);

CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);

CREATE TABLE IF NOT EXISTS tag_flags (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INT4 NOT NULL REFERENCES tags(id),
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, tag_id, flagged_by_ip)
);

CREATE TABLE IF NOT EXISTS tag_types (
    id INT2 PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'Fandom'),
    (2, 'Character'),
    (3, 'Relationship'),
    (4, 'Freeform'),
    (5, 'Warning'),
    (6, 'Category')
ON CONFLICT DO NOTHING;
```

---

# Complete Code Walkthrough: Server Router

## The build_router Function

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();
    
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    
    Router::new()
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))
        
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
        
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))
        
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))
        
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))
        
        .route("/api/v0/search", get(crate::search::routes::search_handler))
        
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))
        
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))
        
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )
        
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        
        .with_state(state)
}
```

---

# Complete Code Walkthrough: AppState Construction

## The run Function

```rust
pub async fn run(config: Config) {
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());
    
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
    
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
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
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    let app = build_router(state).await;
    
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

---

# Complete Code Walkthrough: Configuration Loading

## The Config Struct

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

## The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();
        
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            // ... other fields ...
        }
    }
}
```

---

# Complete Code Walkthrough: Database Models

## The FicInfo Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc],
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

## The ExportLog Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

## The Tag Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub tag_type_id: i16,
    pub created: Option<DateTime<Utc>>,
}
```

---

# Complete Code Walkthrough: Scraper Trait

## The SiteScraper Trait

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

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

## The FicMetadata Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,
    pub updated: i64,
    pub status: String,
    pub source: String,
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}
```

## The ScrapeError Enum

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
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

---

# Complete Code Walkthrough: Scraper Registry

## The Registry Structure

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

## The Registry Implementation

```rust
impl ScraperRegistry {
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
    
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
    
    pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        match self.find_scraper(&meta.source) {
            Some(scraper) => scraper.fetch_chapters(client, meta).await,
            None => Err(ScrapeError::NotFound),
        }
    }
    
    pub fn scraper_count(&self) -> usize {
        self.scrapers.len()
    }
}
```

---

# Complete Code Walkthrough: Main Entry Point

## The main Function

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    
    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    
    server::run(config).await;
}
```

## The lib.rs Module

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

---

# Complete Code Walkthrough: AO3 Scraper

## The Scraper Structure

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";
```

## URL Parsing

```rust
impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;
    
    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(1, &work_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 1,
        author_id: 0,
        author_url: String::new(),
        author_local_id: work_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client
        .get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: FF.net Scraper

## The Scraper Structure

```rust
pub struct FfNetScraper;
```

## URL Parsing

```rust
impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let story_id = Self::extract_story_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;
    
    let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.10")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let words = document
        .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);
    
    let url_id = crate::scrape::generate_url_id(2, &story_id);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        words,
        source: fic_url,
        source_id: 2,
        author_id: 0,
        author_url: String::new(),
        author_local_id: story_id,
        chapters: 0,
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: null,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;
    
    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.10")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let title = document
            .select(&Selector::parse("select#chap_select option[selected]").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {i}"));
        
        chapters.push(Chapter {
            chapter_id: i,
            title,
            content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: XenForo Scraper

## The Scraper Structure

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];
```

## The can_handle Method

```rust
fn can_handle(&self, url: &str) -> bool {
    XENFORO_DOMAINS.iter().any(|domain| url.contains(domain))
}
```

## The lookup Method

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = client
        .get(url)
        .header("User-Agent", "fichub.net/0.0.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let title = document
        .select(&Selector::parse("h1.p-title-value").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
    
    let author = document
        .select(&Selector::parse("article.message:first-child .username").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());
    
    let page_count = document
        .select(&Selector::parse("a.pageNav-page-jump").unwrap())
        .count()
        .max(1);
    
    let url_id = crate::scrape::generate_url_id(3, &url);
    let now = Utc::now().timestamp_millis();
    
    Ok(FicMetadata {
        url_id,
        title,
        author,
        chapters: page_count as i32,
        words: 0,
        source: url.to_string(),
        source_id: 3,
        author_id: 0,
        author_url: String::new(),
        author_local_id: String::new(),
        desc: String::new(),
        published: now,
        updated: now,
        status: String::new(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

## The fetch_chapters Method

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    
    for page in 1..=meta.chapters {
        let url = format!("{}?page={}", meta.source, page);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.0.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let mut page_content = String::new();
        for post in document.select(&Selector::parse("article.message .bbWrapper").unwrap()) {
            let post_html = post.inner_html();
            page_content.push_str(&post_html);
            page_content.push_str("<hr>");
        }
        
        chapters.push(Chapter {
            chapter_id: page,
            title: format!("Page {}", page),
            content: page_content,
        });
    }
    
    Ok(chapters)
}
```

---

# Complete Code Walkthrough: EPUB Generation

## The generate_epub Function

```rust
pub fn generate_epub(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let mut builder = EpubBuilder::new(ZipLibrary::new()
        .map_err(|e| ExportError::EpubError(e.to_string()))?)?;
    
    builder.set_title(&meta.title)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    builder.set_author(&meta.author)
        .map_err_err(e| ExportError::EpubError(e.to_string()))?;
    builder.set_description(&meta.desc)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let stylesheet = include_str!("epub_style.css");
    builder.add_resource("style.css", stylesheet.as_bytes(), "text/css")
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    for chapter in chapters {
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            &chapter.content
        );
        
        builder.add_content(
            EpubContent::new(format!("chapter_{}.xhtml", chapter.chapter_id))
                .title(&chapter.title)
                .content(xhtml.as_bytes()),
        )
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    }
    
    let epub_path = export_dir.join("export.epub");
    let mut epub_file = fs::File::create(&epub_path)?;
    builder.generate(&mut epub_file)
        .map_err(|e| ExportError::EpubError(e.to_string()))?;
    
    let epub_bytes = fs::read(&epub_path)?;
    let mut hasher = Md5::new();
    hasher.update(&epub_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((epub_path, hash))
}
```

---

# Complete Code Walkthrough: HTML Bundle Generation

## The generate_html_bundle Function

```rust
pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    let html_content = build_html(meta, chapters);
    
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    zip.finish()?;
    
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}
```

## The build_html Function

```rust
fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{ font-family: Georgia, serif; max-width: 800px; margin: 0 auto; padding: 20px; }}
        h1 {{ text-align: center; border-bottom: 2px solid #333; }}
        .author {{ text-align: center; color: #666; }}
        .toc {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .chapter {{ margin-top: 3em; border-top: 1px solid #ccc; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#,
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
    );
    
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str("        </ul>\n    </div>\n    <div class=\"content\">\n");
    
    for chapter in chapters {
        html.push_str(&format!(
            r#"        <div class="chapter" id="chapter-{}">
            <h2>{}</h2>
            {}
        </div>
"#,
            chapter.chapter_id,
            escape_html(&chapter.title),
            chapter.content
        ));
    }
    
    html.push_str("    </div>\n</body>\n</html>");
    html
}
```

---

# Complete Code Walkthrough: Cache Path Computation

## The cache_path Function

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

---

# Complete Code Walkthrough: Rate Limiter Lua Script

## The Token Bucket Algorithm

```lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0
end
```

---

# Complete Code Walkthrough: Database Migration

## Migration 001: Initial Schema

```sql
CREATE TABLE IF NOT EXISTS fic_info (
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

CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

CREATE TABLE IF NOT EXISTS request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE IF NOT EXISTS request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);

CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    reason INT4 NOT NULL DEFAULT 0,
    PRIMARY KEY (source_id, author_id)
);

CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT4
);

CREATE INDEX IF NOT EXISTS idx_fic_info_status ON fic_info(status);
CREATE INDEX IF NOT EXISTS idx_fic_info_source ON fic_info(source);
CREATE INDEX IF NOT EXISTS idx_fic_info_words ON fic_info(words);
CREATE INDEX IF NOT INDEX idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX IF NOT EXISTS idx_export_log_url_id ON export_log(url_id);
CREATE INDEX IF NOT EXISTS idx_request_log_created ON request_log(created);
```

## Migration 002: Recommender

```sql
CREATE TABLE IF NOT EXISTS rec_users (
    id VARCHAR(256) PRIMARY KEY,
    source_site VARCHAR(32) NOT NULL,
    last_collected TIMESTAMPTZ,
    favourite_count INT4 DEFAULT 0,
    processed BOOLEAN DEFAULT FALSE
);

CREATE TABLE IF NOT EXISTS rec_favourites (
    user_id VARCHAR(256) NOT NULL,
    url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_id, url_id)
);

CREATE TABLE IF NOT EXISTS rec_cooccurrence (
    story_a VARCHAR(128) NOT NULL,
    story_b VARCHAR(128) NOT NULL,
    count INT4 NOT NULL DEFAULT 1,
    PRIMARY KEY (story_a, story_b)
);

CREATE TABLE IF NOT EXISTS rec_results (
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, recommended_url_id)
);

CREATE TABLE IF NOT EXISTS rec_suggestions (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    source_url TEXT NOT NULL,
    submitted_ip INET,
    created TIMESTAMPTZ DEFAULT NOW(),
    status VARCHAR(32) DEFAULT 'pending'
);

CREATE TABLE IF NOT EXISTS rec_votes (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    recommended_url_id VARCHAR(128) NOT NULL,
    voter_ip INET NOT NULL,
    value INT2 NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, recommended_url_id, voter_ip)
);
```

## Migration 003: Tagging

```sql
CREATE TABLE IF NOT EXISTS tags (
    id INT4 GENERATED ALWAYS AS