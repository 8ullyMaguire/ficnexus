# Part 7 — EPUB Export

> In this chapter you will learn how FicHub generates downloadable EPUB, MOBI, PDF, and AZW3 files from scraped and uploaded fics. We'll build the export pipeline: the HTTP handler that receives a story URL, the EPUB generator, the Calibre-backed format converter, and the on-disk cache that makes repeat downloads instant.

---

## Overview

When a user clicks "Download EPUB" on FicHub, a chain of events fires behind the scenes:

1. The **frontend** calls `GET /api/epub?q=<url>` with the source URL.
2. The **backend handler** (`epub_handler` in `src/routes/export.rs`) checks the Redis and disk caches. If a fresh EPUB exists, it serves it immediately. If not, it scrapes the story, generates an EPUB, caches both the file and an `export_logs` row, and returns the download URL.
3. The **EPUB generator** (`src/export/epub.rs`) takes the parsed metadata + chapters and writes a valid `.epub` file using the `epub-builder` crate.
4. On-demand **format conversion** (`GET /api/epub/convert`) calls `ebook-convert` via Calibre to produce MOBI, PDF, AZW3, etc.
5. The `ebook-convert` binary must be installed (Part 32 covers Kindle delivery).

### Rate limiting

Export endpoints are on the **Download tier** — 10 burst / 60 per hour per IP (see Part 38). Shadowbanned clients get a PoW challenge.

---

## Prerequisites

- You have completed Parts 1–6 (booting the server, database, getting a single work, searching, user accounts, uploading).
- Calibre's `ebook-convert` is installed (the backend calls it as a subprocess for MOBI/PDF/AZW3).
- You have a test story URL from AO3, FanFiction.net, or a manually uploaded work.

---

## Chapter 7.1 — The Export HTTP Handler

### Goal

Understand `epub_handler` — the Axum route that turns a URL into a download link. It handles cache hits, scraping, EPUB generation, and format-specific logic.

### Actions

#### 1. Route registration

In `src/server.rs`, two routes handle exports:

```rust
// src/server.rs (lines 252–253)
.route("/api/epub", get(routes::export::epub_handler))
.route("/api/epub/convert", get(routes::export::convert_format_handler))
```

And the cache download route:

```rust
// Line 259
.route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
.route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
```

> **💡 Key Concept**: The real file route is `/cache/{etype}/{url_id}?h={export_hash}`. The `etype` is one of `epub`, `mobi`, `pdf`, `azw3`, `html`. The `export_hash` is a SHA-1 of the file content — it changes when the source story updates, so browsers can cache aggressively.

#### 2. The main `epub_handler`

Simplified skeleton of `src/routes/export.rs`:

```rust
use axum::{extract::State, http::HeaderMap, Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::error::AppError;
use crate::server::AppState;
use crate::cache::{EType, CacheSemaphores};
use crate::export::epub;
use crate::scrape::registry::ScraperRegistry;

pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    query: axum::extract::Query<EpubQuery>,
) -> Result<Json<Value>, AppError> {
    let q = query.q.as_deref().unwrap_or("");

    // ── Tiered rate limit (download tier) ───────────────────────────
    let (real_ip, client_id) = extract_rate_limit_params(&headers);
    let tier = state.rate_limiter.tier_for_path("/api/epub");
    use crate::limiter::TieredRateLimitResult;
    match state.rate_limiter.check(real_ip, client_id, tier).await {
        TieredRateLimitResult::Wait(secs) => {
            return Err(AppError::RateLimited(secs));
        }
        TieredRateLimitResult::Allowed => {}
    }

    if q.is_empty() {
        return Ok(Json(json!({ "err": -1, "msg": "no query", "q": "" })));
    }

    // ── Check disk cache ───────────────────────────────────────────
    let meta = scrape_or_cache_metadata(&state, q).await?;
    let version = state.config.export_version + get_fic_version_bump(&state, &meta.url_id).await?;
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());

    let cached = crate::db::queries::find_export_log(
        &state.db, &meta.url_id, version, "epub", &input_hash,
    ).await?;

    if let Some(export_log) = cached {
        let url = format!("/cache/epub/{}/{}?h={}", meta.url_id, export_log.export_hash, export_log.export_hash);
        return Ok(Json(json!({
            "err": 0,
            "q": q,
            "url_id": meta.url_id,
            "title": meta.title,
            "author": meta.author,
            "url": url,
            "hash": export_log.export_hash,
            "cached": true,
            "chapters": meta.chapters,
            "words": meta.words,
        })));
    }

    // ── Cache miss: scrape + generate ──────────────────────────────
    let scraper = state.scraper_registry.find_specific_or_fff(q)
        .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {q}")))?;
    let fetched = match scraper.lookup(&state.http_client, q).await {
        Ok(m) => m,
        Err(e) => return Err(AppError::ScrapeError(e.to_string())),
    };

    let cached_meta = state.db.fetch_or_store_metadata(&fetched).await?;
    let meta = cached_meta.to_fic_metadata(/* source: */ "scrape");

    // Acquire semaphore to prevent duplicate concurrent exports
    let sem = crate::cache::get_export_semaphore(
        &state.cache_semaphores, &meta.url_id, &EType::Epub,
    ).await;
    let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Re-check cache after semaphore (double-checked locking pattern)
    let cached_again = crate::db::queries::find_export_log(
        &state.db, &meta.url_id, version, "epub", &input_hash,
    ).await?;
    let epub_hash = if let Some(log) = cached_again {
        log.export_hash
    } else {
        let (epub_path, epub_hash) = epub::create_epub(
            &meta, &chapters, &state.config.tmp_dir
        ).map_err(|e| AppError::ExportError(e.to_string()))?;
        let cache_dest = crate::cache::disk::cache_path(
            &state.config.cache_dir, EType::Epub, &meta.url_id, &epub_hash,
        );
        crate::cache::disk::move_to_cache(&epub_path, &cache_dest)?;
        crate::db::queries::insert_export_log(
            &state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash,
        ).await?;
        epub_hash
    };

    Ok(Json(json!({
        "err": 0,
        "q": q,
        "url_id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "url": format!("/cache/epub/{}/{}?h={}", meta.url_id, epub_hash, epub_hash),
        "hash": epub_hash,
        "cached": false,
        "chapters": meta.chapters,
        "words": meta.words,
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct EpubQuery {
    pub q: Option<String>,
    pub format: Option<String>,
    pub refresh: Option<bool>,
    pub bypassCache: Option<bool>,
}
```

> **⚠️ Watch Out**: The handler uses a double-checked locking pattern with a semaphore. After acquiring the semaphore permit, it re-checks the cache. Without this, two concurrent requests for the same story would both scrape and generate EPUBs — doubled work and potential file collisions.

#### 3. Rate limit extraction

The handler extracts the client IP from `X-Forwarded-For` (set by Nginx) and the `X-Client-ID` from the frontend:

```rust
fn extract_rate_limit_params(headers: &HeaderMap) -> (std::net::IpAddr, Option<String>) {
    let real_ip = crate::limiter::client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    (real_ip, client_id)
}
```

### Try It Yourself

```bash
# Trigger a new export (cache miss)
curl "http://localhost:8000/api/epub?q=https://archiveofourown.org/works/12345678" | jq .

# Trigger a cached export (same URL, should be faster)
curl "http://localhost:8000/api/epub?q=https://archiveofourown.org/works/12345678" | jq '.cached'
```

### Check

- ✅ `GET /api/epub?q=<valid_url>` returns HTTP 200 with `"err": 0`, a `"url"` field, and `"cached": true/false`.
- ✅ A second request for the same URL returns `"cached": true` with the same hash.
- ✅ The rate limiter returns HTTP 429 when too many export requests come from one IP.

### What you built

The HTTP layer of the export pipeline — a cached, rate-limited handler that scrapes a story URL, generates an EPUB, stores it on disk, and returns the download URL.

---

## Chapter 7.2 — EPUB Generation

### Goal

Understand how `create_epub` in `src/export/epub.rs` turns parsed metadata + chapters into a valid `.epub` file.

### Actions

#### 1. The `create_epub` function

```rust
// src/export/epub.rs (simplified)
use epub_builder::{Epub, Language, Creator, Subject, EpubBuilder, Result as EpubResult, RefCell};
use std::path::Path;
use crate::models::{FicMetadata, Chapter, EType};

pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> EpubResult<(PathBuf, String)> {
    // Generate a hash of the content for cache-busting
    let content = generate_epub_content(meta, chapters);
    let hash = format!("{:x}", md5::compute(&content));
    let epub_path = tmp_dir.join(format!("{}_{}.epub", meta.url_id, &hash[..8]));
    std::fs::write(&epub_path, content)?;
    Ok((epub_path, hash))
}

fn generate_epub_content(meta: &FicMetadata, chapters: &[Chapter]) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut epub = EpubBuilder::new(RefCell::new(&mut buf))
            .title(&meta.title)
            .author(&meta.author)
            .publisher("FicHub")
            .language(Language::English)
            .rights("Licensed for personal use only")
            .css(&COVER_CSS)
            .unwrap();

        // Cover page
        epub.content_pdf_cover("/_coverpage.xhtml", &cover_page(meta));

        // Each chapter
        for (i, chapter) in chapters.iter().enumerate() {
            let filename = format!("/chapter_{}.xhtml", i + 1);
            let html = chapter_to_xhtml(chapter, i + 1);
            epub.content(&filename.as_str(), html.as_bytes());
        }

        // Table of contents
        let toc = generate_toc(&meta, chapters);
        epub.content_toc("/toc.ncx", toc);
    }
    buf
}
```

#### 2. The EPUB metadata

The `FicMetadata` struct carries everything the EPUB builder needs:

```rust
// src/models.rs
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub author_url: String,
    pub author_local_id: String,
    pub chapters: i32,
    pub words: i32,
    pub desc: String,
    pub status: String,
    pub published: i64,   // unix milliseconds
    pub updated: i64,
    pub source: String,
    pub extra_meta: serde_json::Value,
    pub raw_extended_meta: Option<serde_json::Value>,
    pub source_id: i32,
    pub author_id: i32,
    pub content_hash: Option<String>,
}
```

The `extra_meta` field holds source-specific fields (AO3 rating, warnings, series name) as JSON.

#### 3. Chapter to XHTML conversion

Each `Chapter` has a `body` field containing HTML from the scraper. The export function wraps it in a full XHTML document:

```rust
fn chapter_to_xhtml(chapter: &Chapter, index: usize) -> String {
    format!(
        r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8" />
  <title>{title}</title>
  <link rel="stylesheet" type="text/css" href="/_fichub.css" />
</head>
<body>
  <div class="chapter">
    <h2>{title}</h2>
    <div class="chapter-body">{body}</div>
  </div>
</body>
</html>"#,
        title = escape_html(&chapter.name),
        body = &chapter.body,
    )
}
```

#### 4. Cover page

The cover page includes the story title, author, word count, and a FicHub watermark:

```rust
fn cover_page(meta: &FicMetadata) -> String {
    format!(
        r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
  <meta charset="utf-8" />
  <title>{title}</title>
  <style>
    body {{ font-family: Georgia, serif; text-align: center; margin-top: 3rem; }}
    h1 {{ font-size: 1.8rem; margin-bottom: 0.5rem; }}
    .author {{ font-size: 1.2rem; color: #666; }}
    .stats {{ margin-top: 2rem; color: #888; font-size: 0.9rem; }}
    .fichub {{ margin-top: 3rem; color: #990000; }}
  </style>
</head>
<body>
  <h1>{title}</h1>
  <div class="author">by {author}</div>
  <div class="stats">{chapters} chapters · {words} words · {status}</div>
  <div class="fichub">Exported from FicHub</div>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        chapters = meta.chapters,
        words = meta.words,
        status = escape_html(&meta.status),
    )
}
```

### Try It Yourself

```rust
// Run the unit tests
cargo test --test epub_export -- --nocapture
```

### Check

- ✅ `create_epub` returns `(PathBuf, String)` — the temp file path and the content hash.
- ✅ The hash changes when the story content changes (because it's an MD5 of the file content).
- ✅ The generated EPUB passes `epubcheck` (the standard validation tool).

### What you built

The EPUB generator — a function that takes metadata + chapter HTML and writes a valid EPUB 3 file using the `epub-builder` crate.

---

## Chapter 7.3 — On-Demand Format Conversion

### Goal

Let users download MOBI, PDF, AZW3, and other formats by calling Calibre's `ebook-convert` binary on the cached EPUB.

### Actions

#### 1. The convert handler

`GET /api/epub/convert?q=<url>&format=mobi`:

```rust
// src/routes/export.rs (convert_format_handler, lines 750–900)
pub async fn convert_format_handler(
    State(state): State<Arc<AppState>>,
    query: Query<ConvertQuery>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let query = query.q.as_deref().unwrap_or("");
    let format = query.format.as_deref().unwrap_or("").to_lowercase();

    if !supported_convert_format(&format) {
        return Ok(Json(json!({
            "err": -1,
            "msg": "unsupported format (use mobi, pdf, or azw3)",
            "q": query,
        })));
    }

    // Rate limit check (same download tier as regular EPUB)
    let (real_ip, client_id) = extract_rate_limit_params(&headers);
    let tier = state.rate_limiter.tier_for_path("/api/epub");
    match state.rate_limiter.check(real_ip, client_id, tier).await {
        TieredRateLimitResult::Wait(secs) => {
            return Err(AppError::RateLimited(secs));
        }
        TieredRateLimitResult::Allowed => {}
    }

    // Find or generate the source EPUB
    let meta = scrape_or_cache_metadata(&state, query).await?;
    let epub_hash = ensure_epub_cached(&state, &meta).await?;

    // Check if the converted format is already cached
    let epub_input_hash = format!("epub:{epub_hash}");
    let cached = crate::db::queries::find_export_log(
        &state.db, &meta.url_id, version, &format, &epub_input_hash,
    ).await?;

    if let Some(log) = cached {
        let url = format!("/cache/{}/{}/{}?h={}", format, meta.url_id, log.export_hash, log.export_hash);
        return Ok(Json(json!({
            "err": 0, "q": query, "url_id": meta.url_id,
            "format": format, "hash": log.export_hash, "url": url, "cached": true,
        })));
    }

    // Convert using Calibre's ebook-convert
    let epub_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, EType::Epub, &meta.url_id, &epub_hash,
    );
    let (output_path, output_hash) = run_calibre_convert(
        &state.config.calibre_container, &epub_path, &format,
        &state.config.cache_dir,
    ).await?;

    crate::db::queries::insert_export_log(
        &state.db, &meta.url_id, version, &format, &epub_input_hash, &output_hash,
    ).await?;

    Ok(Json(json!({
        "err": 0, "q": query, "url_id": meta.url_id,
        "format": format, "hash": output_hash,
        "url": format!("/cache/{}/{}/{}?h={}", format, meta.url_id, output_hash, output_hash),
        "cached": false,
    })))
}

/// Whitelist of Calibre-backed formats
pub fn supported_convert_format(format: &str) -> bool {
    matches!(format, "mobi" | "pdf" | "azw3" | "docx" | "fb2" | "kepub" | "html" | "txt" | "md")
}
```

#### 2. Running `ebook-convert`

The `calibre_container` config field holds the path to `ebook-convert`:

```rust
use tokio::process::Command;
use tokio::io::AsyncReadExt;

async fn run_calibre_convert(
    calibre: &str,
    input: &Path,
    format: &str,
    cache_dir: &Path,
) -> Result<(PathBuf, String), AppError> {
    let output_path = cache_dir.join(format!!(
        "{}.{}", input.file_stem().unwrap(), format
    ));

    let result = Command::new(calibre)
        .arg(input)
        .arg(&output_path)
        .arg("--output-profile")
        .arg("tablet")  // good defaults for all target devices
        .status()
        .await?;

    if !result.success() {
        return Err(AppError::ExportError(format!("ebook-convert failed")));
    }

    let hash = file_hash(&output_path).await?;
    Ok((output_path, hash))
}

async fn file_hash(path: &Path) -> Result<String, AppError> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(format!("{:x}", md5::compute(&buf)))
}
```

> **⚠️ Watch Out**: The `ebook-convert` binary must be on the PATH (or `calibre_container` must point to it). On the production ThinkCentre server, it's at `/usr/bin/ebook-convert`. Without it, `convert_format_handler` returns HTTP 500 with `"export failed"`.

### Try It Yourself

```bash
# Convert an AO3 story to MOBI (first export the EPUB, then convert)
curl "http://localhost:8000/api/epub/convert?q=https://archiveofourown.org/works/12345678&format=mobi" | jq .
```

### Check

- ✅ `GET /api/epub/convert?q=<url>&format=mobi` returns a `/cache/mobi/{url_id}?h={hash}` URL.
- ✅ Converting the same URL + format twice returns `"cached": true` on the second call.
- ✅ `ebook-convert` is invoked with the input EPUB path and output format.

### What you built

A format conversion pipeline that takes the cached EPUB and produces MOBI, PDF, AZW3, or other formats on demand using Calibre, with its own on-disk cache.

---

## Chapter 7.4 — Cache Download Route

### Goal

Serve the cached files from disk. The `download_with_hash` handler verifies the `?h={hash}` parameter matches the file's content hash before serving.

### Actions

#### 1. Route registration

```rust
// src/server.rs (line 259)
.route(
    "/cache/{etype}/{url_id}/{fname}",
    get(routes::cache_download::download_with_hash),
)
.route(
    "/cache/{etype}/{url_id}",
    get(routes::cache_download::download_or_export),
)
```

#### 2. The handler

```rust
// src/routes/cache_download.rs (simplified)
use axum::response::IntoResponse;
use axum::body::Body;
use axum::http::header;

pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype, url_id, fname)): Path<(String, String, String)>,
    query: Query<HashMap<String, String>>,
) -> Result<Response<Body>, AppError> {
    let requested_hash = query.h.as_deref().ok_or_else(|| {
        AppError::BadRequest("missing ?h=hash parameter".into())
    })?;

    let etype = etype.parse::<EType>()
        .map_err(|_| AppError::BadRequest("invalid etype".into()))?;

    // Reconstruct the canonical disk path
    let path = crate::cache::disk::cache_path(
        &state.config.cache_dir, etype, &url_id, requested_hash,
    );

    // Verify the hash matches the actual file content
    let actual_hash = crate::cache::disk::file_hash(&path).await?;
    if actual_hash != requested_hash {
        return Err(AppError::NotFound("hash mismatch — file may have been updated".into()));
    }

    // Serve the file
    let file = tokio::fs::File::open(&path).await?;
    let stream = tokio_util::io::ReaderStream::new(file);
    let content_type = match etype {
        EType::Epub => "application/epub+zip",
        EType::Mobi => "application/x-mobipocket-ebook",
        EType::Pdf => "application/pdf",
        EType::Azw3 => "application/vnd.amz.tf8",
        EType::Html => "text/html",
        _ => "application/octet-stream",
    };

    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, content_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"fic_{}.{}\"", url_id, etype.extension),
        )
        .body(Body::from_stream(stream))?)
}
```

#### 3. The cache path layout

```
/var/cache/fichub/
  epub/
    ab/cd1234ef...  ← url_id/ab/cd/ is the sharding
      abcd1234abcd1234abcd1234abcd1234  ← the file (named by content hash)
  html/
    ab/cd1234ef...
  mobi/
    ab/cd1234ef...
```

The `cache_path` function shards by the first 4 characters of the URL ID to prevent any single directory from having too many files:

```rust
// src/cache/disk.rs (simplified)
pub fn cache_path(
    base: &Path, etype: &EType, url_id: &str, hash: &str,
) -> PathBuf {
    let shard = &url_id[..url_id.len().min(4)];
    base.join(etype.as_str())
        .join(&shard[..2])
        .join(&shard[2..])
        .join(hash)
}
```

### Try It Yourself

```bash
# Download a cached EPUB
curl -OJ "http://localhost:8000/cache/epub/abc123/abc?h=def456"

# Verify the hash parameter
curl -sI "http://localhost:8000/cache/epub/abc123/def?h=wrong"
# Should return 404
```

### Check

- ✅ `GET /cache/{etype}/{url_id}?h={hash}` returns the file with the correct `Content-Type`.
- ✅ A mismatched `?h=` parameter returns `404 Not Found`.
- ✅ Files are sharded into subdirectories of 2+2 characters of the url_id.

### What you built

The final piece of the export puzzle — a secure file-serving route that verifies the content hash before serving cached EPUBs, MOBIs, PDFs, and other formats.

---

## Conclusion

You now understand FicHub's complete export pipeline:

1. **`GET /api/epub?q=<url>`** — scrapes the story, generates an EPUB, caches it, returns a download link.
2. **`GET /api/epub/convert?format=mobi`** — calls `ebook-convert` to produce other formats on demand.
3. **`GET /cache/{etype}/{url_id}?h={hash}`** — serves cached files with hash verification.
4. **Rate limiting** — download tier (10/60hr per IP, 5/1hr for shadowbanned), with PoW challenge for flagged clients.

The cache uses a **content hash** as the filename — when the source story updates, the hash changes, the old file stays on disk (for existing links), and new requests get the fresh version.

---

## On to the next part

In Part 8 we'll build [User Accounts and Authentication] — JWT-based login, registration with invite codes, password reset, and the auth middleware that protects every write endpoint.
