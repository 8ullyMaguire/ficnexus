# Part 4: Export and Caching

---

# Chapter 16: Generating EPUB Files

EPUB is the standard format for e-books. This chapter covers how FicHub generates EPUB files from scraped story content, including metadata, table of contents, and formatting.

## Understanding EPUB Format

An EPUB file is actually a ZIP archive containing:
- `META-INF/container.xml` — Points to the content file
- `OEBPS/content.opf` — Package document (metadata, manifest, spine)
- `OEBPS/toc.ncx` — Navigation control (table of contents)
- `OEBPS/*.xhtml` — Chapter content files
- `OEBPS/*.css` — Stylesheets
- `mimetype` — Must be first file, uncompressed

**Real-world analogy:** An EPUB is like a well-organized binder. The `container.xml` is the table of contents that tells you where everything is. The `content.opf` is the metadata page (title, author, ISBN). The chapter XHTML files are the actual pages. The CSS is the formatting guide. The `mimetype` file is the label on the outside of the binder that says "this is an EPUB."

## The EPUB Generation Function

Here's FicHub's EPUB generation code:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;

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

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
```

### Line-by-Line Walkthrough

**Lines 1-3:** Import necessary types. `EpubBuilder` is the main struct for building EPUBs. `ZipLibrary` handles ZIP compression. `Md5` computes file hashes.

**Lines 6-9:** Create a temporary directory with a UUID name. This prevents conflicts when multiple exports run concurrently.

**Lines 12-15:** Initialize the EPUB builder with a ZIP library. The builder accumulates metadata and content, then generates the final EPUB in one step.

**Lines 18-22:** Set metadata on the EPUB. Title, author, and description are required for a valid EPUB.

**Lines 25-26:** Add a CSS stylesheet. The `include_str!` macro embeds the CSS file at compile time, so we don't need to ship a separate CSS file.

**Lines 29-43:** Add each chapter as an XHTML file. The chapter content is wrapped in a proper HTML structure with the stylesheet link. The `escape_xml` function ensures special characters don't break the XML.

**Lines 46-48:** Generate the EPUB file. The builder writes the ZIP archive with all the EPUB components.

**Lines 51-54:** Compute the MD5 hash of the generated file. This hash is used for cache lookup — if the same content is requested again, we can return the cached file.

**Lines 57-58:** Clean up the temporary directory. We only need the final EPUB file, not the intermediate files.

### EPUB Stylesheet

```css
/* epub_style.css */
body {
    font-family: Georgia, serif;
    line-height: 1.6;
    margin: 1em;
    text-align: justify;
}

h1 {
    font-size: 1.5em;
    margin-top: 2em;
    margin-bottom: 1em;
    text-align: center;
    border-bottom: 1px solid #ccc;
    padding-bottom: 0.5em;
}

p {
    margin-bottom: 0.8em;
    text-indent: 1.5em;
}

blockquote {
    margin: 1em 2em;
    padding: 0.5em 1em;
    border-left: 3px solid #ccc;
    font-style: italic;
}

/* Chapter navigation */
.nav {
    text-align: center;
    margin-top: 2em;
    padding-top: 1em;
    border-top: 1px solid #ccc;
}

.nav a {
    margin: 0 1em;
    color: #666;
    text-decoration: none;
}
```

## EPUB Best Practices

### Metadata Completeness

A good EPUB should include:
- **Title** — The story title
- **Author** — The author's name
- **Description** — The story summary
- **Language** — Usually "en" for English
- **Publisher** — "FicHub" for our generated EPUBs
- **Date** — When the EPUB was generated

### Chapter Structure

Each chapter should:
- Have a clear heading
- Be a separate XHTML file
- Include proper HTML structure
- Use the stylesheet for consistent formatting

### Error Handling

EPUB generation can fail for several reasons:
- Invalid characters in metadata
- File system errors
- ZIP compression errors
- Memory limits for very large stories

FicHub handles these with the `ExportError` enum:

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    TemplateError(String),
    EpubError(String),
    CalibreError(String),
    ZipError(String),
}
```

## 📝 Practice Exercises

1. **EPUB Metadata:** Generate an EPUB with complete metadata including title, author, description, language, and publisher. Verify the metadata is correct by opening the EPUB in an e-reader.

2. **Chapter Formatting:** Add support for bold, italic, and blockquote formatting in the EPUB output. How does the HTML need to be structured?

3. **Table of Contents:** Add a table of contents to the EPUB that links to each chapter. How does the NCX file need to be structured?

4. **Error Recovery:** What happens if a chapter has invalid HTML? Write a function that sanitizes HTML before adding it to the EPUB.

---

# Chapter 17: HTML Bundles

In addition to EPUB files, FicHub generates HTML bundles for browser-based reading. This chapter covers the HTML bundle generation process.

## What Are HTML Bundles?

An HTML bundle is a self-contained ZIP file containing:
- A single HTML file with all chapters
- Embedded CSS for styling
- A table of contents
- Chapter navigation

**Real-world analogy:** An HTML bundle is like a travel guide. Everything you need is in one package — the content, the formatting, the table of contents. You don't need internet access or any special software — just open it in a browser.

## The HTML Bundle Generator

```rust
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;
use md5::{Digest, Md5};
use uuid::Uuid;

pub fn generate_html_bundle(
    tmp_dir: &Path,
    meta: &FicMetadata,
    chapters: &[Chapter],
) -> Result<(PathBuf, String), ExportError> {
    // 1. Create temporary directory
    let export_id = Uuid::new_v4().to_string();
    let export_dir = tmp_dir.join(&export_id);
    fs::create_dir_all(&export_dir)?;
    
    // 2. Generate the HTML content
    let html_content = build_html(meta, chapters);
    
    // 3. Create the ZIP file
    let zip_path = export_dir.join("export.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);
    
    // 4. Add the HTML file to the ZIP
    zip.start_file("story.html", options)?;
    zip.write_all(html_content.as_bytes())?;
    
    // 5. Finalize the ZIP
    zip.finish()?;
    
    // 6. Compute hash
    let zip_bytes = fs::read(&zip_path)?;
    let mut hasher = Md5::new();
    hasher.update(&zip_bytes);
    let hash = hex::encode(hasher.finalize());
    
    // 7. Clean up
    fs::remove_dir_all(&export_dir)?;
    
    Ok((zip_path, hash))
}

fn build_html(meta: &FicMetadata, chapters: &[Chapter]) -> String {
    let mut html = format!(r#"<!DOCTYPE html>
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
            color: #333;
        }}
        h1 {{
            text-align: center;
            border-bottom: 2px solid #333;
            padding-bottom: 10px;
        }}
        .author {{
            text-align: center;
            color: #666;
            margin-bottom: 2em;
        }}
        .toc {{
            background: #f5f5f5;
            padding: 20px;
            border-radius: 5px;
            margin-bottom: 2em;
        }}
        .toc h2 {{
            margin-top: 0;
        }}
        .toc ul {{
            list-style: none;
            padding: 0;
        }}
        .toc li {{
            margin: 5px 0;
        }}
        .toc a {{
            color: #333;
            text-decoration: none;
        }}
        .toc a:hover {{
            text-decoration: underline;
        }}
        .chapter {{
            margin-top: 3em;
            padding-top: 1em;
            border-top: 1px solid #ccc;
        }}
        .chapter h2 {{
            text-align: center;
        }}
        p {{
            margin-bottom: 0.8em;
            text-indent: 1.5em;
        }}
        blockquote {{
            margin: 1em 2em;
            padding: 0.5em 1em;
            border-left: 3px solid #ccc;
            font-style: italic;
        }}
        .nav {{
            text-align: center;
            margin-top: 2em;
            padding: 1em;
            background: #f5f5f5;
            border-radius: 5px;
        }}
        .stats {{
            text-align: center;
            color: #666;
            margin-bottom: 2em;
        }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="author">by {}</div>
    <div class="stats">
        {} words · {} chapters · {}
    </div>
    
    <div class="toc">
        <h2>Table of Contents</h2>
        <ul>
"#, 
        escape_html(&meta.title),
        escape_html(&meta.title),
        escape_html(&meta.author),
        meta.words,
        meta.chapters,
        meta.status,
    );
    
    // Add table of contents entries
    for chapter in chapters {
        html.push_str(&format!(
            "            <li><a href=\"#chapter-{}\">{}</a></li>\n",
            chapter.chapter_id,
            escape_html(&chapter.title)
        ));
    }
    
    html.push_str(r#"        </ul>
    </div>
    
    <div class="content">
"#);
    
    // Add chapter content
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
    
    html.push_str(r#"    </div>
    
    <div class="nav">
        <a href="#">Back to Top</a>
    </div>
</body>
</html>"#);
    
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

## HTML Bundle vs EPUB

| Feature | HTML Bundle | EPUB |
|---------|------------|------|
| Format | ZIP with HTML | ZIP with XHTML |
| Reader | Any browser | E-reader apps |
| Navigation | Scroll + TOC | Chapter-based |
| Styling | Embedded CSS | Embedded CSS |
| Metadata | In HTML | In OPF/NCX |
| Offline | Yes | Yes |

## 📝 Practice Exercises

1. **Custom Styling:** Modify the HTML bundle generator to support custom color schemes (light mode, dark mode, sepia).

2. **Navigation:** Add "previous chapter" and "next chapter" links within each chapter.

3. **Search:** Add a simple search function that highlights matching text in the HTML.

---

# Chapter 18: The Disk Cache

The disk cache stores generated files on the filesystem. This chapter covers the cache directory structure, path computation, and cache management.

## Cache Directory Structure

FicHub uses a hierarchical directory structure to organize cached files:

```
cache/
├── epub/
│   ├── abc/
│   │   ├── 123/
│   │   │   ├── 456/
│   │   │   │   ├── abc123456def/
│   │   │   │   │   └── a1b2c3d4e5f6.epub
├── html/
│   ├── def/
│   │   ├── 789/
│   │   │   ├── 012/
│   │   │   │   ├── def789012abc/
│   │   │   │   │   └── a1b2c3d4e5f6.zip
```

The url_id is split into 3-character chunks to keep directory sizes manageable. For example, url_id `abc123456def` becomes:
- `abc/123/456/abc123456def/`

**Real-world analogy:** The cache directory is like a library's filing system. Instead of putting all books in one giant shelf (which would be slow to search), the library organizes them by subject, author, and title. Similarly, FicHub organizes cached files by type, url_id chunks, and content hash.

## Path Computation

```rust
use std::path::{Path, PathBuf};
use crate::cache::EType;

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
    
    // Split url_id into 3-character chunks
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_cache_path_epub() {
        let root = PathBuf::from("/cache");
        let path = cache_path(&root, &EType::Epub, "abc123456def", "a1b2c3d4");
        assert_eq!(
            path,
            PathBuf::from("/cache/epub/abc/123/456/abc123456def/a1b2c3d4.epub")
        );
    }
    
    #[test]
    fn test_cache_path_html() {
        let root = PathBuf::from("/cache");
        let path = cache_path(&root, &EType::Html, "abc123456def", "a1b2c3d4");
        assert_eq!(
            path,
            PathBuf::from("/cache/html/abc/123/456/abc123456def/a1b2c3d4.zip")
        );
    }
    
    #[test]
    fn test_cache_path_short_id() {
        let root = PathBuf::from("/cache");
        let path = cache_path(&root, &EType::Epub, "abc", "a1b2c3d4");
        assert_eq!(
            path,
            PathBuf::from("/cache/epub/abc/abc/a1b2c3d4.epub")
        );
    }
}
```

## Cache Semaphores

To prevent duplicate concurrent exports for the same story, FicHub uses semaphores:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

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

**Real-world analogy:** A semaphore is like a bathroom key at a gas station. Only one person can use the bathroom at a time. If someone else wants to use it, they have to wait for the key to become available. This prevents two people from trying to use the bathroom simultaneously.

The semaphore ensures that only one export runs at a time for each (url_id, etype) pair. If two users request the same story simultaneously, the second request waits for the first to complete, then finds the cached result.

## Cache Hit/Miss Flow

```rust
// In the export handler:

// 1. Compute input hash
let input_hash = compute_input_hash(&meta);

// 2. Check cache (first pass)
if let Some(cached) = queries::find_export_log(
    &state.db, &url_id, version, etype, &input_hash
).await? {
    // Cache hit — return immediately
    let download_url = format!("/cache/{}/{}/{}?h={}", 
        etype, url_id, cached.export_hash);
    return Ok(Json(json!({"err": 0, "url": download_url})));
}

// 3. Acquire semaphore (prevent duplicate exports)
let semaphore = get_export_semaphore(
    &state.cache_semaphores, &url_id, etype
).await;
let _permit = semaphore.acquire().await.unwrap();

// 4. Double-check cache (another request might have finished)
if let Some(cached) = queries::find_export_log(
    &state.db, &url_id, version, etype, &input_hash
).await? {
    // Cache hit after semaphore — another request finished
    let download_url = format!("/cache/{}/{}/{}?h={}", 
        etype, url_id, cached.export_hash);
    return Ok(Json(json!({"err": 0, "url": download_url})));
}

// 5. Generate export (cache miss)
let (file_path, hash) = generate_export(&meta, &chapters).await?;

// 6. Move to cache
let cache_path = cache::disk::cache_path(
    &state.config.cache_dir, &etype, &url_id, &hash
);
fs::rename(&file_path, &cache_path)?;

// 7. Record in cache
queries::insert_export_log(
    &state.db, &url_id, version, etype, &input_hash, &hash
).await?;
```

The double-check pattern is important: after acquiring the semaphore, we check the cache again because another request might have completed the export while we were waiting.

## 📝 Practice Exercises

1. **Cache Cleanup:** Write a function that finds and removes cached files older than 30 days.

2. **Cache Statistics:** Write a function that counts the number of cached files and their total size.

3. **Cache Invalidation:** Write a function that invalidates all cached files for a specific story.

---

# Chapter 19: Rate Limiting with Redis

Rate limiting prevents FicHub from overwhelming upstream sites. This chapter covers the token bucket algorithm, Redis-backed implementation, and Lua scripts for atomic operations.

## The Token Bucket Algorithm

The token bucket is a classic rate limiting algorithm:

1. A bucket holds up to `max_tokens` tokens
2. Tokens are added at a fixed rate (`refill_rate` tokens per second)
3. Each request consumes one token
4. If the bucket is empty, the request is denied

**Real-world analogy:** A token bucket is like a parking garage with a limited number of spaces. New spaces open up at regular intervals (when cars leave). If all spaces are full, new cars have to wait. The garage has a maximum capacity (`max_tokens`) and a turnover rate (`refill_rate`).

## The Lua Script

FicHub's rate limiting is implemented as a Lua script that runs atomically inside Redis:

```lua
-- token_bucket.lua
local key = KEYS[1]
local max_tokens = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local cost = tonumber(ARGV[4]) or 1

-- Get current state
local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1])
local last_refill = tonumber(bucket[2])

-- Initialize if new bucket
if tokens == nil then
    tokens = max_tokens
    last_refill = now
end

-- Refill tokens based on time elapsed
local elapsed = math.max(0, now - last_refill)
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

-- Check if request is allowed
if tokens >= cost then
    tokens = tokens - cost
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1  -- allowed
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0  -- denied
end
```

### Why Lua?

Redis executes Lua scripts atomically — no other command can run while the script is executing. This prevents race conditions where two requests check the token count simultaneously and both succeed.

**Real-world analogy:** The Lua script is like a bank teller processing a transaction. The teller checks your balance, withdraws the money, and updates the balance — all in one atomic operation. No one else can access your account while this is happening, so there's no risk of overdrawing.

## The RedisBucketLimiter

```rust
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

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
        // Load the Lua script into Redis
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
}

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
    
    async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
        // Load IP ranges from configuration
        // These are IPs that should be blocked (bots, scrapers)
        for (path, _type, _tag) in sources {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    if let Ok(ip) = line.trim().parse::<IpAddr>() {
                        self.datacenter_ips.write().await.insert(ip);
                    }
                }
            }
        }
    }
}
```

### Rate Limit Tiers

FicHub uses three tiers of rate limiting:

| Tier | Key Pattern | Max Tokens | Refill Rate | Purpose |
|------|------------|------------|-------------|---------|
| Global | `rate_limit:global` | 60 | 1.0/sec | Total system capacity |
| Per-IP | `rate_limit:ip:{ip}` | 30 | 0.5/sec | Per-user fairness |
| Per-site | `rate_limit:site:{site}` | 20 | 0.33/sec | Upstream site protection |

The per-site rate limit is the most important — it ensures FicHub doesn't overwhelm any single fanfiction site.

## 📝 Practice Exercises

1. **Rate Limit Testing:** Write a test that sends 100 requests in rapid succession and verifies that only 60 are allowed (global limit).

2. **Dynamic Rate Limiting:** Implement dynamic rate limiting that adjusts based on upstream site response times. If a site is slow, reduce the rate limit.

3. **Rate Limit Headers:** Add rate limit information to HTTP response headers (`X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`).

---

# Chapter 20: The Export Flow

This chapter traces the complete export flow from request to response, tying together all the pieces we've built.

## The Complete Flow

```
User Request → Rate Limiter → Scraper Registry → Database Check
    → Cache Check → [Cache Hit] → Return Cached File
    → [Cache Miss] → Semaphore → Double-Check Cache
    → Fetch Chapters → Generate EPUB → Generate HTML
    → Move to Cache → Record in DB → Return Response
```

## Step-by-Step Walkthrough

### Step 1: Request Arrives

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ExportQuery>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
    let client_ip = remote_addr.ip();
    
    tracing::info!(
        ip = %client_ip,
        url = %query.q,
        "Export request received"
    );
```

### Step 2: Rate Limit Check

```rust
    // Rate limit check
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

### Step 3: Scraper Lookup

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

### Step 4: Database Operations

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

### Step 5: Cache Check

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

### Step 6: Export Generation

```rust
    // Acquire semaphore
    let semaphore = get_export_semaphore(
        &state.cache_semaphores, &meta.url_id, EType::Epub
    ).await;
    let _permit = semaphore.acquire().await.unwrap();
    
    // Double-check cache
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

## Performance Considerations

The export flow involves several I/O operations:
- HTTP requests to fanfiction sites
- Database queries
- File system operations
- EPUB/HTML generation

Each of these can be slow. FicHub optimizes by:
1. **Caching aggressively** — Most requests hit the cache
2. **Using connection pools** — Database and HTTP connections are reused
3. **Running exports concurrently** — Different stories export in parallel
4. **Using semaphores** — Preventing duplicate exports for the same story

## 📝 Practice Exercises

1. **Tracing:** Add detailed tracing to each step of the export flow. What information should be logged at each step?

2. **Error Recovery:** What happens if the EPUB generation fails after the HTML bundle is generated? How should we handle partial failures?

3. **Metrics:** Add timing metrics for each step of the export flow. How would you use these metrics to identify bottlenecks?

