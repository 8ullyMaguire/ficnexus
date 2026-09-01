# Extended Content: Complete Application Walkthrough

---

# FicHub Application Architecture Deep Dive

## System Overview

FicHub is a multi-component system that handles the entire lifecycle of fanfiction stories, from discovery to delivery. Understanding the system architecture helps developers navigate the codebase and contribute effectively.

### Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Request                          │
│  GET /api/v0/epub?q=https://archiveofourown.org/works/123  │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Router                              │
│  Rate Limit Check → Scraper Lookup → Cache Check           │
└──────────────────────────┬──────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
         ┌────────┐  ┌──────────┐  ┌──────────┐
         │ Redis  │  │ Scraper  │  │PostgreSQL│
         │ (Rate  │  │ Registry │  │ (Cache   │
         │ Limit) │  │          │  │  Check)  │
         └────────┘  └─────┬────┘  └──────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ AO3 Scraper  │
                    │ (HTTP Fetch) │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ EPUB Builder │
                    │ (Export)     │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ Disk Cache   │
                    │ (File Store) │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ JSON Response│
                    │ (Client)     │
                    └──────────────┘
```

### Data Flow

When a client requests a story export, the following data flow occurs:

1. **Request Parsing** — The URL is extracted from the query parameters
2. **Rate Limiting** — Redis checks if the client has exceeded rate limits
3. **Scraper Selection** — The registry finds the appropriate scraper for the URL
4. **Metadata Fetching** — The scraper fetches the story page and extracts metadata
5. **Database Upsert** — The metadata is stored or updated in PostgreSQL
6. **Cache Check** — The system checks if a cached export already exists
7. **Export Generation** — If not cached, the story chapters are fetched and an EPUB/HTML bundle is generated
8. **Cache Storage** — The generated files are stored on disk and recorded in the database
9. **Response** — A JSON response with download URLs is returned to the client

### Module Responsibilities

Each module in FicHub has a single, well-defined responsibility:

**scrape/** — Responsible for communicating with fanfiction websites. Contains the `SiteScraper` trait, individual site implementations (AO3, FF.net, XenForo), and the `ScraperRegistry` that maps URLs to scrapers.

**export/** — Responsible for generating output files. Contains EPUB generation, HTML bundle generation, and format conversion (MOBI/PDF via Calibre).

**cache/** — Responsible for storing and retrieving cached files. Contains disk cache path computation, semaphore-based deduplication, and cache invalidation logic.

**limiter/** — Responsible for rate limiting. Contains the Redis-backed token bucket implementation and Lua scripts for atomic operations.

**recommender/** — Responsible for story recommendations. Contains the collaborative filtering engine, the collection worker that scrapes user favourites, and API endpoints for recommendations and voting.

**tags/** — Responsible for the tag system. Contains tag resolution (canonical lookup, alias fallback, creation), voting logic, and curator tools for moderation.

**search/** — Responsible for full-text search. Contains the dynamic query builder that constructs SQL queries based on search parameters.

**routes/** — Responsible for HTTP handlers. Contains all API endpoint implementations, OPDS feed generation, and cache download logic.

**db/** — Responsible for database access. Contains the connection pool setup, SQLx models, and query functions.

**server/** — Responsible for application startup. Contains the `AppState` struct, router construction, and server binding.

## Configuration System

FicHub loads all configuration from environment variables. This follows the twelve-factor app methodology.

### Configuration Loading

```rust
impl Config {
    pub fn from_env() -> Self {
        // Required variables
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        // Optional with defaults
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        // Complex parsing
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            port,
            trusted_proxies,
            // ... other fields
        }
    }
}
```

### Configuration Validation

FicHub validates configuration at startup:

```rust
impl Config {
    fn validate(&self) -> Result<(), String> {
        // Validate database URL
        if self.database_url.is_empty() {
            return Err("DATABASE_URL cannot be empty".into());
        }
        
        // Validate port
        if self.port == 0 {
            return Err("PORT cannot be 0".into());
        }
        
        // Validate cache directory
        if !self.cache_dir.exists() {
            std::fs::create_dir_all(&self.cache_dir)
                .map_err(|e| format!("Failed to create cache dir: {}", e))?;
        }
        
        Ok(())
    }
}
```

### Configuration Testing

FicHub uses a mutex to serialize tests that modify environment variables:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    
    struct EnvGuard {
        keys: Vec<String>,
    }
    
    impl EnvGuard {
        fn new() -> Self {
            EnvGuard { keys: Vec::new() }
        }
        
        fn set(&mut self, key: &str, val: &str) {
            self.keys.push(key.to_string());
            unsafe { std::env::set_var(key, val); }
        }
    }
    
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for key in &self.keys {
                unsafe { std::env::remove_var(key); }
            }
        }
    }
    
    #[test]
    fn test_from_env_defaults() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test_db");
        guard.set("REDIS_URL", "redis://localhost/0");
        
        let config = Config::from_env();
        
        assert_eq!(config.database_url, "postgres://localhost/test_db");
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert_eq!(config.port, 3000);
    }
}
```

## Error Handling System

FicHub uses a comprehensive error handling system that converts all errors to appropriate HTTP responses.

### Error Types

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

### Error Conversion

```rust
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
```

### Error Propagation

The `?` operator automatically converts errors:

```rust
async fn process(url: &str) -> Result<StoryMeta, AppError> {
    // reqwest::Error → AppError::ScrapeError
    let response = reqwest::get(url).await?;
    
    // reqwest::Error → AppError::ScrapeError
    let html = response.text().await?;
    
    // sqlx::Error → AppError::Database
    let existing = queries::get_fic_info(&pool, &url_id).await?;
    
    Ok(meta)
}
```

## Database Layer

### Connection Pool

```rust
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;
    
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    Ok(pool)
}
```

### Query Patterns

**Single Row:**
```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

**Multiple Rows:**
```rust
pub async fn search_stories(pool: &PgPool, query: &str) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE title ILIKE $1 LIMIT 10"
    )
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

**Upsert:**
```rust
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (id, title, author, chapters, words)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (id) DO UPDATE SET
               title = EXCLUDED.title,
               author = EXCLUDED.author,
               chapters = EXCLUDED.chapters,
               words = EXCLUDED.words,
               updated = NOW()"#
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(fic.chapters)
    .bind(fic.words)
    .execute(pool)
    .await?;
    Ok(())
}
```

**Transaction:**
```rust
pub async fn transfer_tag(
    pool: &PgPool,
    from_url_id: &str,
    to_url_id: &str,
    tag_id: i32,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    
    sqlx::query("DELETE FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
        .bind(from_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    sqlx::query("INSERT INTO fic_tags (url_id, tag_id, score) VALUES ($1, $2, 0)")
        .bind(to_url_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    Ok(())
}
```

## Scraping System

### The SiteScraper Trait

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) 
        -> Result<FicMetadata, ScrapeError>;
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError>;
    
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) 
        -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

### Scraper Registry

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

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
}
```

### AO3 Scraper

```rust
pub struct Ao3Scraper;

const BASE_URL: &str = "https://archiveofourown.org";

impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) 
        -> Result<FicMetadata, ScrapeError> 
    {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("invalid URL".into()))?;
        
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
            .and_then(|el| el.text().collect::<String>().replace(',', "").parse().ok())
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
            // ... other fields
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
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
        
        let mut chapters = Vec::new();
        for (i, chapter_div) in document.select(&Selector::parse("div.chapter").unwrap()).enumerate() {
            let title = chapter_div
                .select(&Selector::parse("h3.title").unwrap())
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
                title,
                content,
            });
        }
        
        Ok(chapters)
    }
}
```

## Export System

### EPUB Generation

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

### HTML Bundle Generation

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

## Caching System

### Cache Path Computation

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

### Cache Semaphores

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

## Rate Limiting System

### Token Bucket Implementation

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
}

impl RedisBucketLimiter {
    pub async fn new(
        redis: redis::aio::MultiplexedConnection,
        dynamic_rate_limit: bool,
    ) -> Result<Self, redis::RedisError> {
        let lua_script = include_str!("token_bucket.lua");
        let sha: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(lua_script)
            .query_async(&mut redis.clone())
            .await?;
        
        Ok(RedisBucketLimiter {
            redis,
            lua_sha: sha,
            dynamic_rate_limit,
            static_delay_base: 1.0,
            datacenter_ips: Arc::new(RwLock::new(HashSet::new())),
        })
    }
    
    async fn check_bucket(
        &self,
        key: &str,
        max_tokens: f64,
        refill_rate: f64,
        now: f64,
        cost: f64,
    ) -> Result<bool, AppError> {
        let result: i32 = redis::cmd("EVALSHA")
            .arg(&self.lua_sha)
            .arg(1)
            .arg(key)
            .arg(max_tokens)
            .arg(refill_rate)
            .arg(now)
            .arg(cost)
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| AppError::CacheError(e.to_string()))?;
        
        Ok(result == 1)
    }
}
```

## Recommendation System

### Collaborative Filtering Engine

```rust
pub struct RecommendationEngine {
    pool: PgPool,
}

impl RecommendationEngine {
    pub fn new(pool: PgPool) -> Self {
        RecommendationEngine { pool }
    }
    
    pub async fn get_recommendations(
        &self,
        url_id: &str,
        limit: usize,
    ) -> Result<Vec<Recommendation>, AppError> {
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
        
        let mut recommendations = Vec::new();
        for (rec_url_id, count) in cooccurrences {
            let (a_count, b_count) = self.get_favouriter_counts(url_id, &rec_url_id).await?;
            let score = compute_score(count, a_count, b_count, 0.0);
            
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
        
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(limit);
        
        Ok(recommendations)
    }
}
```

## Tag System

### Tag Resolution

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

## Search System

### Dynamic Query Builder

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

## Monitoring and Observability

### Structured Logging

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,fichub=debug".into()),
    )
    .with_target(false)
    .with_thread_ids(true)
    .with_file(true)
    .with_line_number(true)
    .init();
```

### Request Logging

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
    let request_id = Uuid::new_v4();
    
    tracing::info!(
        request_id = %request_id,
        ip = %remote_addr.ip(),
        url = %params.q,
        "Request received"
    );
    
    // Process request...
    
    let duration = start.elapsed();
    tracing::info!(
        request_id = %request_id,
        duration_ms = duration.as_millis(),
        "Request complete"
    );
    
    Ok(Json(response))
}
```

### Health Checks

```rust
async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let mut checks = HashMap::new();
    
    let db_ok = sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_ok();
    checks.insert("database", db_ok);
    
    let redis_ok = redis::cmd("PING")
        .query_async::<String>(&mut state.redis.clone())
        .await
        .is_ok();
    checks.insert("redis", redis_ok);
    
    let all_healthy = checks.values().all(|&v| v);
    
    Ok(Json(json!({
        "status": if all_healthy { "healthy" } else { "unhealthy" },
        "checks": checks
    })))
}
```

## Deployment

### Docker

```dockerfile
FROM rust:1.77-bookworm as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3
WORKDIR /app
COPY --from=builder /app/target/release/fichub .
EXPOSE 3000
CMD ["./fichub"]
```

### Systemd

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

## Testing

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
    fn test_escape_xml() {
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("<b>bold</b>"), "&lt;b&gt;bold&lt;/b&gt;");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_search_endpoint() {
    let state = setup_test_state().await;
    let app = build_router(Arc::new(state));
    
    let response = axum_test::TestClient::new(app)
        .get("/api/v0/search")
        .query(&[("q", "harry potter")])
        .await;
    
    assert_eq!(response.status_code(), 200);
    let body: serde_json::Value = response.json();
    assert_eq!(body["err"], 0);
}
```

