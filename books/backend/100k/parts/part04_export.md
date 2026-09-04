# Part 4: Export and Caching

---

# Chapter 16: EPUB Generation

## What is EPUB?

EPUB (Electronic Publication) is the most widely supported e-book format, defined by the IDPF (International Digital Publishing Forum). It's essentially a ZIP file containing XHTML files, CSS stylesheets, images, and metadata described by XML files. The structure follows the Open Packaging Format (OPF).

EPUB files can be read on virtually any e-reader: Kindle (with conversion to AZW3), Kobo, Nook, Apple Books, Google Play Books, and many Android reading apps. They're also readable in web browsers using browser extensions or online readers.

FicHub generates EPUBs using the `epub-builder` Rust crate, which provides a clean API for constructing EPUB files without dealing with the underlying XML complexity.

## The EPUB File Structure

An EPUB file is a ZIP archive with a specific internal structure:

```
story.epub (ZIP file)
├── mimetype                    # Must be first, uncompressed
├── META-INF/
│   └── container.xml           # Points to the OPF file
└── OEBPS/
    ├── content.opf             # Package document (metadata, manifest, spine)
    ├── toc.ncx                 # Navigation control (for older readers)
    ├── stylesheet.css          # Shared CSS
    ├── introduction.xhtml      # Title page
    ├── chapter_1.xhtml         # Chapter 1
    ├── chapter_2.xhtml         # Chapter 2
    └── ...                     # More chapters
```

The `epub-builder` crate handles all of this complexity — you just provide the metadata, content, and styles, and it assembles the EPUB correctly.

## The EPUB Generation Process

When FicHub generates an EPUB, it follows these steps:

### Step 1: Create a Working Directory

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

Each EPUB is generated in a unique UUID-named directory. This prevents conflicts when multiple EPUBs are generated concurrently.

### Step 2: Initialize the Builder

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
```

The `ZipLibrary` provides the ZIP compression backend. The builder manages the internal EPUB structure.

### Step 3: Set Metadata

```rust
builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

Metadata is stored in the OPF file and is used by e-readers to display book information.

### Step 4: Add CSS Stylesheet

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

The stylesheet is inlined into the EPUB. The `epub-builder` crate writes it as `stylesheet.css` and links it from all content files.

### Step 5: Add Introduction Page

The introduction page displays the story's metadata:

```rust
let intro_html = format!(
    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
    title = escape_html(&meta.title),
    author = escape_html(&meta.author),
    words = meta.words,
    chapters = meta.chapters,
    status = escape_html(&meta.status),
    published = format_timestamp(meta.published),
    updated = format_timestamp(meta.updated),
    desc = escape_html(&meta.desc),
);

builder.add_content(
    EpubContent::new("introduction.xhtml", intro_html.as_bytes())
        .title("Introduction"),
)?;
```

The introduction gives readers context before they start reading.

### Step 6: Add Chapters

```rust
for chapter in chapters {
    let chapter_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
        title = escape_html(&chapter.title),
        content = chapter.content,
    );

    let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
    builder.add_content(
        EpubContent::new(filename.as_str(), chapter_html.as_bytes())
            .title(&chapter.title),
    )?;
}
```

Each chapter becomes a separate XHTML file in the EPUB. The `epub-builder` crate handles the internal linking and spine ordering.

### Step 7: Write the EPUB File

```rust
let epub_path = work_dir.join("output.epub");
let file = fs::File::create(&epub_path)?;
builder.generate(file)?;
```

The `generate` method writes the complete EPUB structure to the file.

### Step 8: Compute MD5 Hash

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();
```

The MD5 hash serves two purposes:
1. **Cache key** — Stored in the database to check if the same EPUB already exists
2. **Integrity check** — Verified when serving the file to detect corruption

## HTML Escaping

User-provided content must be escaped to prevent HTML injection:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

Without this, a title like `My <script>alert('xss')</script> Fic` would be rendered as HTML instead of displayed as text.

## Timestamp Formatting

Unix millisecond timestamps are converted to human-readable dates:

```rust
fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;
    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

## Watch Out!

**EPUB generation is synchronous!** The `epub-builder` crate blocks the current task. For most stories, this is fine (a few seconds). For very long stories, consider using `tokio::task::spawn_blocking`.

**Memory usage!** The entire EPUB is built in memory. For very long stories, this could use significant memory.

**XML compliance!** All content must be valid XHTML. The `epub-builder` crate handles this, but malformed HTML from scrapers can cause issues.

## Summary

FicHub's EPUB generation produces well-formatted e-books with proper metadata, chapter navigation, and CSS styling. The MD5 hash ensures cache validity and file integrity.

---

# Chapter 17: HTML Bundles

## Why HTML Bundles?

While EPUBs are great for e-readers, sometimes you just want a simple HTML file you can open in any browser. FicHub generates HTML bundles alongside EPUBs — self-contained ZIP files containing a single HTML page with all chapters, metadata, and navigation.

## The HTML Bundle Structure

The generated HTML has a clean, readable layout:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>My Amazing Fanfiction — SomeAuthor</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: Georgia, 'Times New Roman', serif;
            line-height: 1.7;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #fafafa;
        }
        h1 { text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }
        h2 {
            text-align: center;
            margin: 1.5em 0 0.5em;
            font-size: 1.4em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }
        .meta { text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }
        .meta td { padding: 2px 8px; }
        .desc {
            margin: 1em 0;
            padding: 1em;
            background: #fff;
            border-radius: 4px;
            border: 1px solid #eee;
        }
        .nav {
            background: #fff;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1em;
            margin: 1.5em 0;
        }
        .nav h3 { margin-bottom: 0.5em; }
        .nav ul { list-style: none; columns: 2; }
        .nav li { padding: 2px 0; }
        .nav a { color: #1a5276; text-decoration: none; }
        .nav a:hover { text-decoration: underline; }
        .content p { margin: 0.5em 0; text-indent: 1.5em; }
        .content p:first-of-type { text-indent: 0; }
        hr { border: none; border-top: 1px solid #ddd; margin: 2em 0; }
        .footer { text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }
    </style>
</head>
<body>
    <h1>My Amazing Fanfiction</h1>
    <div class="meta">
        <p>by <strong>SomeAuthor</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>125000</td></tr>
            <tr><td>Chapters:</td><td>10</td></tr>
            <tr><td>Status:</td><td>complete</td></tr>
            <tr><td>Published:</td><td>2023-01-15</td></tr>
            <tr><td>Updated:</td><td>2023-06-20</td></tr>
        </table>
    </div>
    <div class="desc">
        A story about things happening...
    </div>
    <hr/>
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>
            <li><a href="#ch1">Chapter 1</a></li>
            <li><a href="#ch2">Chapter 2</a></li>
            <li><a href="#ch3">Chapter 3</a></li>
        </ul>
    </div>
    <hr/>
    <div class="content">
        <h2 id="ch1">Chapter 1</h2>
        <p>Once upon a time in a land far away...</p>
        <h2 id="ch2">Chapter 2</h2>
        <p>The adventure continued as they...</p>
        <h2 id="ch3">Chapter 3</h2>
        <p>And so the journey reached its climax...</p>
    </div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — https://archiveofourown.org/works/12345678</p>
    </div>
</body>
</html>
```

Key features:
- **Responsive design** — Works on mobile and desktop
- **Chapter navigation** — Clickable links to jump between chapters
- **Clean typography** — Serif font, comfortable line height, proper spacing
- **Metadata header** — Title, author, word count, status, dates
- **Description** — Story summary in a highlighted box

## The Generation Process

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Create working directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // Build navigation and content
    let mut chapters_nav = String::new();
    let mut chapters_content = String::new();

    for chapter in chapters {
        chapters_nav.push_str(&format!(
            r##"<li><a href="#ch{ch}">{title}</a></li>"##,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
        ));

        chapters_content.push_str(&format!(
            r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
            content = chapter.content,
        ));
    }

    // Assemble the full HTML
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: Georgia, 'Times New Roman', serif; line-height: 1.7; color: #333; max-width: 800px; margin: 0 auto; padding: 20px; background: #fafafa; }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }}
        h2 {{ text-align: center; margin: 1.5em 0 0.5em; font-size: 1.4em; border-bottom: 1px solid #ddd; padding-bottom: 0.3em; }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }}
        .meta td {{ padding: 2px 8px; }}
        .desc {{ margin: 1em 0; padding: 1em; background: #fff; border-radius: 4px; border: 1px solid #eee; }}
        .nav {{ background: #fff; border: 1px solid #ddd; border-radius: 4px; padding: 1em; margin: 1.5em 0; }}
        .nav h3 {{ margin-bottom: 0.5em; }}
        .nav ul {{ list-style: none; columns: 2; }}
        .nav li {{ padding: 2px 0; }}
        .nav a {{ color: #1a5276; text-decoration: none; }}
        .nav a:hover {{ text-decoration: underline; }}
        .content p {{ margin: 0.5em 0; text-indent: 1.5em; }}
        .content p:first-of-type {{ text-indent: 0; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2em 0; }}
        .footer {{ text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <div class="meta">
        <p>by <strong>{author}</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>{words}</td></tr>
            <tr><td>Chapters:</td><td>{chapters}</td></tr>
            <tr><td>Status:</td><td>{status}</td></tr>
            <tr><td>Published:</td><td>{published}</td></tr>
            <tr><td>Updated:</td><td>{updated}</td></tr>
        </table>
    </div>
    <div class="desc">{desc_escaped}</div>
    <hr/>
    <div class="nav"><h3>Chapter Navigation</h3><ul>{nav}</ul></div>
    <hr/>
    <div class="content">{content}</div>
    <hr/>
    <div class="footer"><p>Generated by fICHub — {source}</p></div>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        words = meta.words,
        chapters = meta.chapters,
        status = escape_html(&meta.status),
        published = format_timestamp(meta.published),
        updated = format_timestamp(meta.updated),
        desc_escaped = escape_html(&meta.desc),
        nav = chapters_nav,
        content = chapters_content,
        source = escape_html(&meta.source),
    );

    // Write to file
    let html_path = work_dir.join("index.html");
    fs::write(&html_path, &html)?;

    // Bundle into ZIP
    let zip_path = work_dir.join("bundle.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip.start_file("index.html", options)?;
    zip.write_all(html.as_bytes())?;
    zip.finish()?;

    // Compute MD5
    let zip_data = fs::read(&zip_path)?;
    let md5_hex = Md5::digest(&zip_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((zip_path, md5_hex))
}
```

## ZIP Compression

The HTML is compressed into a ZIP file:

```rust
let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

HTML compresses well — typically achieving 70-80% reduction in file size.

## EPUB vs HTML Bundle

| Feature | EPUB | HTML Bundle |
|---------|------|-------------|
| Format | EPUB (ZIP with XHTML) | ZIP with single HTML |
| Reader support | E-readers, Apple Books | Any web browser |
| Chapter navigation | Built-in TOC | Anchor links |
| Styling | CSS (limited by reader) | Full CSS control |
| File size | Smaller (no duplication) | Larger (all content in one file) |
| Offline reading | Yes | Yes (if extracted) |
| Search | Reader's search | Browser's search |
| Bookmarking | Reader's bookmarking | Browser's bookmarking |

## Watch Out!

**Large files!** For very long stories (100+ chapters), the HTML bundle can be several megabytes. The ZIP compression helps, but the file is still larger than an EPUB.

**CSS compatibility!** The CSS is designed for modern browsers. Older browsers might not render it correctly.

**Encoding!** The HTML uses UTF-8 encoding. Non-ASCII characters (common in fanfiction) are handled correctly.

## Summary

HTML bundles provide a universal format that works in any browser. They offer clean typography, chapter navigation, and responsive design in a self-contained package.

---

# Chapter 18: Disk Cache

## The Caching Strategy

FicHub generates EPUBs and HTML bundles on-demand, which is expensive. To avoid regenerating the same file multiple times, it uses a multi-layered caching strategy.

## Hash-Based Directory Structure

Cached files are stored in a hash-based directory structure:

```
cache/
├── epub/
│   └── a1b/
│       └── def/
│           └── a1b2c3d4e5f6/
│               └── abc123def456.epub
├── html/
│   └── a1b/
│       └── def/
│           └── a1b2c3d4e5f6/
│               └── 789xyz012abc.zip
```

The first three directory levels are chunks of the `url_id` (3 characters each). This spreads files across directories to avoid having too many files in a single folder.

The `cache_path` function computes this:

```rust
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    path = path.join(url_id);
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

## EType Enum

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }

    pub fn version(&self) -> i32 {
        match self {
            EType::Epub => 1,
            EType::Html => 1,
            EType::Mobi => 0,
            EType::Pdf => 0,
        }
    }
}
```

## Cache Semaphores

When multiple requests arrive for the same story simultaneously, FicHub uses semaphores to prevent duplicate generation:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

Each `(url_id, etype)` pair gets its own semaphore with capacity 1.

## The Double-Check Pattern

```rust
// First check (fast — no semaphore needed)
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Acquire semaphore
let sem = cache::get_export_semaphore(&state.cache_semaphores, &url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Generate
let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await?;
```

## Moving Files to Cache

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

`fs::rename` is atomic on most filesystems.

## Cache Validation

```rust
match crate::cache::disk::file_md5(&cache_path) {
    Ok(actual_hash) if actual_hash == hash => {
        // Serve the file
        let data = tokio::fs::read(&cache_path).await?;
        // ...
    }
    _ => {
        // Hash mismatch
        Json(json!({"err": -5, "msg": "hash mismatch"})).into_response()
    }
}
```

## File MD5 Computation

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

## Clearing Stale Cache

```rust
pub fn clear_stale_cache(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    keep_hash: &str,
) -> AppResult<()> {
    let dir = cache_root.join(etype.as_str()).join(url_id);
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem != keep_hash {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
```

## Watch Out!

**Disk space management!** Cached files grow over time. Monitor disk usage and clean up old files if needed.

**Filesystem limits!** Some filesystems limit files per directory. The 3-level hash helps.

**Atomic moves!** If source and destination are on different filesystems, `fs::rename` falls back to copy-then-delete.

## Summary

FicHub's disk cache uses hash-based directories, semaphore-controlled generation, double-check locking, and MD5 validation for efficient, concurrent-safe caching.

---

# Chapter 19: Rate Limiting

## Why Rate Limit?

Rate limiting serves two purposes:
1. **Protect FicHub** — Prevent abuse and ensure fair usage
2. **Protect fanfiction sites** — Avoid overwhelming the sites we scrape

## Token Bucket Algorithm

A token bucket has two parameters:
- **Capacity** — Maximum tokens
- **Flow rate** — Tokens added per second

When a request comes in, it consumes one token. If no tokens are available, the request is delayed.

FicHub uses two buckets:
- **Global** — Capacity: 150, Flow: 30/sec
- **Per-IP** — Capacity: 30, Flow: 0.116/sec (~1 request every 8.6 seconds)

## The Redis Lua Script

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

Redis executes Lua scripts atomically, so there are no race conditions.

## The RedisBucketLimiter

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    global_capacity: f64,
    global_flow: f64,
    ip_capacity: f64,
    ip_flow: f64,
}
```

## Checking Rate Limits

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }

    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
        .await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
        .await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

## The RateLimiter Trait

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;
    async fn report_failure(&self, ip: IpAddr);
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}
```

## RateLimitResult Enum

```rust
pub enum RateLimitResult {
    Allowed,
    Wait(u64),   // Seconds to wait
    Blocked,     // Datacenter IP
}
```

## Penalties on Failure

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}
```

## Datacenter IP Blocking

```rust
pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
    for (file_path, _type, _tag) in sources {
        if let Ok(content) = tokio::fs::read_to_string(file_path).await {
            let mut ips = self.datacenter_ips.write().await;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Ok(ip) = line.parse::<IpAddr>() {
                        ips.insert(ip);
                    }
                }
            }
        }
    }
}
```

## Watch Out!

**Rate limits are per-server!** For shared rate limiting across instances, you'd need a distributed solution.

**Lua scripts are atomic!** Redis ensures no race conditions in the token bucket logic.

## Summary

FicHub's rate limiter uses a token bucket algorithm in Redis Lua scripts, providing global and per-IP rate limiting with datacenter IP blocking and failure penalties.

---

# Chapter 20: Export Flow

## The Complete Journey

Let's trace the complete 17-step export flow.

## Step 1: HTTP Request

User sends `GET /api/v0/epub?q=https://archiveofourown.org/works/12345678`

## Step 2: Validate Query

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}
```

## Step 3: Check Automated Flag

```rust
if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
}
```

## Step 4: Find Scraper

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

## Step 5: Lookup Metadata

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
let info_request_ms = start.elapsed().as_millis() as i32;
```

## Step 6: Upsert Fic Info

```rust
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

## Step 7: Auto-populate Tags

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(&state.db, &tag.name, tag.tag_type_id).await {
            let _ = queries::upsert_fic_tag(&state.db, &meta.url_id, resolution.tag_id, &ip).await;
        }
    }
}
```

## Step 8: Check Blacklists

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }
}
```

## Step 9: Compute Version

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
```

## Step 10: Check Cache (First)

```rust
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
```

## Step 11: Cache Hit Path

If cached, build response and return immediately.

## Step 12: Acquire Semaphore

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

## Step 13: Double-Check

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}
```

## Step 14: Fetch Chapters

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

## Step 15: Generate EPUB

```rust
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

## Step 16: Move to Cache

```rust
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

## Step 17: Generate HTML and Build Response

```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir).await?;
// ... move to cache, record ...

Ok(Json(json!({
    "err": 0,
    "q": query,
    "url_id": meta.url_id,
    "slug": generate_slug(&meta.title, &meta.url_id),
    "meta": build_meta_json(&meta),
    "epub_url": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
    "html_url": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
})))
```

## Performance Characteristics

- **Cache hit:** ~10ms
- **Concurrent generation:** ~100ms (waiting)
- **Full generation:** ~5-30 seconds

## Summary

The 17-step export flow handles caching, scraping, generation, and response building with efficient concurrency and cache validation.
