# EPUB Codebase Documentation

Junior-developer onboarding guide covering the fullstack architecture of FicHub:
the Rust backend (Axum + SQLx + scraper crate), the SvelteKit frontend (runes,
ArchiveLayout, AO3-style archive parity), and how EPUBs are generated, cached, and
served.

No prior FicHub context required. Every example links to real source files.

---

## Table of contents

1. [Stack at a glance](#1-stack-at-a-glance)
2. [Backend: the Rust side](#2-backend-the-rust-side)
3. [Frontend: the SvelteKit side](#3-frontend-the-sveltekit-side)
4. [EPUB generation pipeline](#4-epub-generation-pipeline)
5. [Data flow: scrape to EPUB](#5-data-flow-scrape-to-epub)
6. [Frontend EPUB UI](#6-frontend-epub-ui)
7. [Key source files](#7-key-source-files)

---

## 1. Stack at a glance

| Layer        | Technology              | Notes                                             |
|--------------|-------------------------|---------------------------------------------------|
| Backend      | Rust 2024, Axum, SQLx   | `src/` — API, scraping, exports, social           |
| Scraping     | `fanfic-scrapers` crate | `scrapers/` — native adapters for 107 sites       |
| Frontend     | SvelteKit 5 (SPA)       | `frontend/` — built to `frontend/build/`          |
| Database     | PostgreSQL 16 + pgvector| Migrations in `migrations/`                      |
| Cache        | Redis                   | Rate limiter, shadowban, reading state           |
| ML/embeddings| Ollama (nomic-embed-text) | Recs, auto-tagger, roadmap consensus            |
| Docs         | mdBook (markdown)       | `docs/` — this site                              |

The frontend is a **SPA served from one origin**: Axum serves `frontend/build/`
as a fallback, so `/docs/` is static HTML inside the SPA.

---

## 2. Backend: the Rust side

### 2.1 Project structure

```
fichub/
├── src/
│   ├── server.rs          # Axum router — ALL routes registered here
│   ├── routes/            # one module per feature area
│   │   ├── download.rs    # /download, /api/download
│   │   ├── search.rs      # /api/search, /api/search/body
│   │   ├── auth.rs        # login, logout, registration
│   │   ├── social.rs      # bookmarks, ratings, reviews, comments
│   │   ├── forum.rs       # forum categories, topics, posts
│   │   ├── works.rs       # work metadata, series, authors
│   │   └── ...            # ~25 route modules
│   ├── scrape/            # scraper subsystem
│   │   ├── mod.rs         # FicMetadata, Chapter, ScrapeError
│   │   ├── registry.rs    # find_scraper, lookup, fetch_chapters
│   │   └── compat_fichub_net.rs  # fichub.net API fallback
│   ├── export/            # export format builders
│   │   ├── mod.rs         # ExportFormat enum, factory
│   │   ├── epub.rs        # EPUB generation (epub_builder)
│   │   ├── txt.rs         # plain text export
│   │   └── md.rs          # markdown export
│   ├── db/                # sqlx pool, queries module
│   ├── services/          # OCR, content scan, Ollama
│   └── bin/               # migration runner, backfill bins
├── scrapers/              # standalone crate (published to crates.io)
│   ├── src/lib.rs         # FicMetadata, Chapter, SiteScraper trait
│   ├── src/registry.rs    # ScraperRegistry: site → adapter
│   └── src/sites/         # one adapter per site (ao3.rs, ffnet.rs, …)
├── frontend/              # SvelteKit SPA
├── migrations/            # SQLx migrations 001…N
└── tests/                 # Rust integration tests (DB-gated)
```

### 2.2 The scraper contract

The `fanfic-scrapers` crate defines a trait every site adapter implements:

**Real source:** `scrapers/src/lib.rs` (lines 98–160)

```rust
// Every site adapter implements this trait
pub trait SiteScraper {
    /// Stable, site-local ID from a story URL.
    fn url_id(&self, url: &str) -> Result<String, ScrapeError>;

    /// Fetch the fic metadata (title, author, chapters, word count, …).
    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError>;

    /// Download all chapter content (HTML) for the resolved fic.
    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError>;
}

// Shared data model — site-agnostic
pub struct FicMetadata {
    pub url_id: String,        // e.g. "ao3_21845264"
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub status: String,        // "ongoing" | "complete" | "hiatus" | "cancelled"
    pub source: String,        // original fic URL
    // … author_id, author_url, published, updated, etc.
}

pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,       // HTML
}
```

The host (`src/scrape/registry.rs`) calls `find_specific_or_fallback()` to look
up a native adapter first; if no native adapter exists, the catch-all only runs
as a last resort.

### 2.3 Export pipeline

**Real source:** `src/export/mod.rs`, `src/export/epub.rs`

When a user downloads a fic:

1. The route handler calls `scrape_fic` → `lookup` metadata + `fetch_chapters`
2. The metadata and chapters are passed to the format-specific builder
3. For EPUB: `create_epub(meta, chapters, tmp_dir)` → returns `(path, md5)`
4. The file is served to the user as a download; the MD5 is returned for caching

```rust
// src/export/mod.rs — export factory
pub enum ExportFormat {
    Epub,
    Txt,
    Html,
    Markdown,
    Mobi,
    Pdf,
    Azw3,
}

// src/routes/download.rs — simplified
pub async fn download_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadParams>,
) -> Result<Response, AppError> {
    let (meta, chapters) = scrape_fic(&state, &params.url).await?;
    let (path, md5) = create_epub(&meta, &chapters, &state.tmp_dir).await?;
    Ok(send_file(path, &meta.title, "epub"))
}
```

---

## 3. Frontend: the SvelteKit side

### 3.1 Dual UI modes

The frontend supports two UI modes, switched via the user's preference
(`getPref('uiMode')`):

- **Archive mode** (`'archive'`): AO3-style archive look with serif fonts,
  `fieldset.dl` forms, `archive-main`/`archive-content` wrappers
- **Modern mode** (`'modern'`): default SvelteKit UI with cards, cards, buttons

The root layout (`/routes/+layout.svelte`) checks `uiMode` and renders
either `ArchiveLayout` or the modern `<div class="app">` shell:

```svelte
<!-- frontend/src/routes/+layout.svelte, line 150 -->
{#if uiMode === 'archive'}
  <ArchiveLayout>
    {@render children()}
  </ArchiveLayout>
{:else}
  <div class="app">
    <header class="topbar">…</header>
    <main class="container">{@render children()}</main>
    <footer class="footer">…</footer>
  </div>
{/if}
```

### 3.2 ArchiveLayout

**Real source:** `frontend/src/lib/ui/archive/ArchiveLayout.svelte`

```svelte
<script>
  import ArchiveHeader from './ArchiveHeader.svelte';
  import ArchiveFooter from './ArchiveFooter.svelte';
  import { getPref } from '$lib/prefs';

  // body gets skin-zerafina or skin-ao3 class for CSS targeting
  onMount(() => {
    const skin = getPref('archiveSkin');
    document.body.classList.add(`skin-${skin}`);
  });
</script>

<div class="archive-shell">
  <ArchiveHeader />
  <main class="archive-main">
    {@render children()}
  </main>
  <ArchiveFooter />
</div>
```

Every page that uses archive mode delegates its header/footer to `ArchiveLayout`
— per-page `ArchiveHeader`/`ArchiveFooter` components are **not** used
(they were removed to avoid duplication).

### 3.3 ArchiveHeader dropdowns

**Real source:** `frontend/src/lib/ui/archive/ArchiveHeader.svelte`

The header has dropdown menus for **Browse** and **Search**:

```svelte
<!-- Browse dropdown -->
<NavDropdown label="Browse">
  <a class="archive-dd-item" href="/trending">Trending</a>
  <a class="archive-dd-item" href="/bookmarks">Bookmarks</a>
  <a class="archive-dd-item" href="/fandoms">Fandoms</a>
  <a class="archive-dd-item" href="/lists">Lists</a>
  <a class="archive-dd-item" href="/series">Series</a>
  <a class="archive-dd-item" href="/tags">Tags</a>
</NavDropdown>

<!-- Search dropdown -->
<NavDropdown label="Search">
  <a class="archive-dd-item" href="/search">Work Search</a>
  <a class="archive-dd-item" href="/people">People Search</a>
  <a class="archive-dd-item" href="/tags/search">Tag Search</a>
  <a class="archive-dd-item" href="/bookmarks/search">Bookmark Search</a>
</NavDropdown>
```

### 3.4 Svelte 5 runes

The frontend uses Svelte 5 runes:

```svelte
<script lang="ts">
  // Reactive state — updates are fine-grained
  let items = $state<Work[]>([]);
  let loading = $state(true);

  // Derived value — recomputes when deps change
  const visibleItems = $derived(items.slice(0, 50));

  // Props (Svelte 5 syntax, no export default)
  let { data } = $props<{ categorySlug: string }>();

  // Effects — run when dependencies change
  $effect(() => {
    void load();  // data-fetching effect
  });

  async function load() {
    loading = true;
    const res = await fetch(`/api/forum/topics?category=${data.categorySlug}`);
    // …
  }
</script>
```

### 3.5 Route testing

Route tests live in `page.test.ts` (NOT `+page.test.ts`) in each route
directory. Example:

```
frontend/src/routes/search/tags/page.test.ts
```

---

## 4. EPUB generation pipeline

### 4.1 The EPUB builder

**Real source:** `src/export/epub.rs` (145 lines)

EPUBs are generated using the `epub_builder` crate with a `ZipLibrary` backend:

```rust
pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // 1. Create a UUID-named work directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // 2. Set up the epub_builder
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    // 3. Set metadata (title, author, lang, description)
    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;

    // 4. Add a stylesheet (inline CSS for book-style paragraphs)
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0 0 0.8em 0;text-indent:1.5em;}",
        "h1+p, h2+p, h3+p, h4+p, p:first-of-type{text-indent:0;}",
    );
    builder.stylesheet(css.as_bytes())?;

    // 5. Add an introduction page (metadata table)
    let intro_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>Introduction</title>
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

    // 6. Add each chapter as its own XHTML file
    for chapter in chapters {
        let chapter_html = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{title}</title>
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

    // 7. Write the final .epub file (a ZIP archive)
    let epub_path = work_dir.join("output.epub");
    let file = fs::File::create(&epub_path)?;
    builder.generate(file)?;

    // 8. Compute MD5 hash for cache keying
    let epub_data = fs::read(&epub_path)?;
    let md5_hex = Md5::digest(&epub_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((epub_path, md5_hex))
}
```

### 4.2 Body cache

**Real source:** `src/scrape/compat_fichub_net.rs`, `src/cache/`

Every scraped fic body is persisted as sharded versioned files under
`BODY_CACHE_DIR` (e.g. `/public/literature/fichub/bodies`):

```
bodies/
├── 42/
│   ├── 13/
│   │   ├── 4213f2f5_0/        # {url_id[0:2]}/{url_id[2:4]}/{url_id[5:]}_0/
│   │   │   ├── metadata.json  # FicMetadata snapshot
│   │   │   ├── chapter_1.html
│   │   │   ├── chapter_2.html
│   │   │   └── …
```

The path is derived from the last 8 characters of the `url_id` (hex), sharded
two characters at a time: `{url_id[0:2]}/{url_id[2:4]}/{url_id[5:]}_{version}/`.

On export, the builder checks the body cache first — if a cached version exists,
it reuses it instead of re-scraping. This is why repeat exports are instant.

### 4.3 Calibre sidecar

For **MOBI**, **PDF**, and **AZW3** formats, the backend shell-processes via
Calibre's `ebook-convert`:

```rust
// src/export/mod.rs — MOBI/PDF/AZW3 path
pub async fn convert_with_calibre(
    epub_path: &Path,
    format: &str,
    output_dir: &Path,
) -> Result<PathBuf, ExportError> {
    let output_path = output_dir.join(format!(
        "{}.{}",
        epub_path.file_stem().unwrap().to_str().unwrap(),
        format
    ));
    let status = Command::new("ebook-convert")
        .arg(epub_path)
        .arg(&output_path)
        .status()
        .await?;
    if !status.success() {
        return Err(ExportError::ConversionFailed(format));
    }
    Ok(output_path)
}
```

Calibre lives on the production machine (ThinkCentre) — it is NOT required for
EPUB/HTML/TXT/MD (those are pure Rust via `epub_builder`).

---

## 5. Data flow: scrape to EPUB

```
                    ┌─────────────────────────────────────────────┐
                    │         User clicks "Download"               │
                    └──────────────┬──────────────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────────────┐
                    │  /download route handler                    │
                    │  (src/routes/download.rs)                   │
                    └──────────────┬──────────────────────────────┘
                                   │  1. Build reqwest client (with cookie store)
                                   │
                    ┌──────────────▼──────────────────────────────┐
                    │  scrape/compat_fichub_net.rs                │
                    │  ─ checks body cache for url_id            │
                    │  ─ if cached: skip lookup, use cached      │
                    │  ─ if not: call registry.lookup()        │
                    └──────────────┬──────────────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────────────┐         ┌──────────────────┐
                    │  scrapers/src/registry.rs                   │         │  SiteScraper      │
                    │  find_specific_or_fallback()               │◄────────┤  trait impl      │
                    │  1. Native adapter (preferred)             │         │  (ao3.rs, ffnet, │
                    │  2. Catch-all fallback (FicFab)            │         │   etc.)           │
                    └──────────────┬──────────────────────────────┘         └──────┬───────────┘
                                   │                                                  │
                    ┌──────────────▼──────────────────────────────┐                 │
                    │  SiteScraper::lookup()                       │◄────────────────┘
                    │  → FicMetadata { url_id, title, author,     │
                    │    chapters, words, desc, status, … }      │
                    └──────────────┬──────────────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────────────┐
                    │  SiteScraper::fetch_chapters()              │
                    │  → Vec<Chapter> { chapter_id, title,       │
                    │    content: HTML }                         │
                    └──────────────┬──────────────────────────────┘
                                   │  2. Cache body to BODY_CACHE_DIR
                                   │
                    ┌──────────────▼──────────────────────────────┐
                    │  export/epub.rs: create_epub()              │
                    │  1. epub_builder::EpubBuilder (ZipLibrary)  │
                    │  2. add metadata (title, author, desc)      │
                    │  3. add stylesheet (CSS)                    │
                    │  4. add introduction.xhtml (metadata table) │
                    │  5. add chapter_N.xhtml for each chapter    │
                    │  6. .generate() → write .epub (ZIP)         │
                    │  7. compute MD5 hash                        │
                    └──────────────┬──────────────────────────────┘
                                   │  3. (Optional) Calibre for MOBI/PDF/AZW3
                                   │
                    ┌──────────────▼──────────────────────────────┐
                    │  Serve file to user                          │
                    │  Response headers:                          │
                    │  Content-Disposition: attachment;           │
                    │    filename="..."                           │
                    │                                             │
                    │  For MOBI/PDF/AZW3: Calibre runs           │
                    │  ebook-convert epub→mobi (or pdf/azw3)     │
                    └─────────────────────────────────────────────┘
```

---

## 6. Frontend EPUB UI

### 6.1 Download form

**Real source:** `frontend/src/routes/download/+page.svelte`

The download page has a URL input form. When submitted, it POSTs to the API:

```svelte
<!-- archive mode uses ArchiveWorkSearchForm-style fieldset.dl -->
{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content archive-page">
      <header class="archive-header">
        <h1 class="archive-page-title">Download</h1>
      </header>
      <form class="archive-form" onsubmit={handleDownload}>
        <fieldset>
          <legend>Story URL</legend>
          <input type="url" bind:value={urlInput}
                 placeholder="https://archiveofourown.org/works/…"
                 required />
        </fieldset>
        <div class="archive-actions">
          <ArchiveButton type="submit" disabled={downloading}>
            {downloading ? 'Processing…' : 'Download'}
          </ArchiveButton>
        </div>
      </form>
    </div>
  </main>
{/if}
```

### 6.2 EPUB in the web reader

**Real source:** `frontend/src/routes/read/[urlId]/+page.svelte`

The web reader renders chapter HTML inline in the browser. When a user
requests an EPUB download, the frontend calls:

```typescript
// frontend/src/lib/api/download.ts
export async function downloadEpub(url: string): Promise<Blob> {
  const res = await fetch('/api/download', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url, format: 'epub' }),
  });
  return res.blob();
}
```

---

## 7. Key source files

### Backend
- `src/server.rs` — Axum router, all route registration
- `src/scrape/registry.rs` — scraper lookup, `find_specific_or_fallback`
- `src/export/epub.rs` — EPUB generation (145 lines)
- `src/export/mod.rs` — export format factory, Calibre conversion
- `src/routes/download.rs` — `/download` + `/api/download` handlers
- `src/db/queries.rs` — all SQL queries (works, fics, bookmarks, tags)

### Frontend
- `frontend/src/routes/+layout.svelte` — root layout, UI mode switching
- `frontend/src/lib/ui/archive/ArchiveLayout.svelte` — archive wrapper
- `frontend/src/lib/ui/archive/ArchiveHeader.svelte` — archive nav dropdowns
- `frontend/src/lib/ui/archive/ArchiveButton.svelte` — AO3-style button
- `frontend/src/routes/download/+page.svelte` — download form
- `frontend/src/routes/search/tags/+page.svelte` — tag search (AO3-style)
- `frontend/src/routes/people/+page.svelte` — people search
- `frontend/src/lib/api/forum.ts` — forum API client
- `frontend/src/lib/prefs.ts` — localStorage preferences (uiMode, archiveSkin)

### Scrapers
- `scrapers/src/lib.rs` — `FicMetadata`, `Chapter`, `SiteScraper` trait
- `scrapers/src/registry.rs` — `ScraperRegistry` (site → adapter)
- `scrapers/src/sites/ao3.rs` — AO3 adapter (200 lines)
- `scrapers/src/sites/ffnet.rs` — FanFiction.net adapter
