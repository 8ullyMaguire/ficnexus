# Part 20 — RSS and OPDS Feeds

> In this chapter you will learn how FicHub generates Atom and OPDS XML feeds that let users subscribe in feed readers (Feedly, NewsBlur) and e-book readers (KOReader, Calibre). We'll build the RSS feed handlers, the OPDS catalog, and the share links that connect readers to their libraries.

---

## Overview

FicHub generates XML feeds in two formats:

- **Atom/RSS** — for feed readers and web subscriptions
- **OPDS** — for e-book readers (KOReader, Calibre, Apple Books, etc.)

All RSS feeds live in `src/routes/rss.rs` (365 lines). The OPDS catalog lives in `src/routes/opds/` — `mod.rs` (250+ lines), `feeds.rs`, `authors.rs`, `tags.rs`, `search.rs`.

The RSS handlers:
- `GET /feed.xml` — new arrivals (recently scraped fics)
- `GET /feed/follows.xml` — updates from followed works/authors
- `GET /feed/works/{url_id}.xml` — per-fic update feed

The OPDS catalog:
- `GET /opds` — root catalog (new, popular, user's feed)
- `GET /opds/new` — new arrivals catalog
- `GET /opds/popular` — popular fics
- `GET /opds/tags` — browse by tag type
- `GET /opds/tags/{type_id}` — fics with a specific tag
- `GET /opds/tags/{type_id}/{tag_name}` — fics with a specific tag name
- `GET /opds/authors` — browse by author
- `GET /opds/recommendations/{url_id}` — recommendations for a fic
- `GET /opds/search` — search within OPDS
- `GET /opds/shelves` — user's shelves
- `GET /opds/shelf/{shelf_id}` — works in a shelf

---

## Chapter 20.1 — Atom/RSS Feeds

### Goal

Build the three RSS/Atom feed endpoints — new arrivals, follows feed, and per-fic feed.

### Actions

#### 1. The RSS module and build helpers

```rust
// src/routes/rss.rs (365 lines) — the RSS/Atom feed module
pub mod build;
pub use build::{atom_feed, entry_for_fic, html_escape, iso_now, FeedQuery};
```

```rust
// src/routes/rss/build.rs (300+ lines)
/// Build an Atom XML string from a list of fic entries.
pub fn atom_feed(entries: &[AtomEntry], updated: &str) -> String { ... }

/// Build a single Atom entry for a fic.
pub fn entry_for_fic(
    fic_info: &FicInfo,
    feed_query: &FeedQuery,
) -> AtomEntry { ... }

/// Escape HTML for safe embedding in XML/Atom.
pub fn html_escape(s: &str) -> String { ... }

/// Current time in ISO 8601 / RFC 3339 format.
pub fn iso_now() -> String { ... }

pub struct FeedQuery {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub source: String,
    pub created: String,
    pub description: String,
}
```

#### 2. New arrivals RSS — `GET /feed.xml`

```rust
// src/routes/rss.rs (365 lines total, handlers near the bottom)
pub async fn new_arrivals_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RssParams>,
) -> Result<(aiax::response::Response, axum::http::StatusCode), AppError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0);

    // Fetch recently added fics from fic_info, newest first
    let rows: Vec<(String, String, String, i64, i32, String, String, String, String)> =
        sqlx::query_as(
            r#"
            SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters,
                   fi.status, fi.source, fi.created, fi.description
            FROM fic_info fi
            ORDER BY fi.created DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let entries: Vec<AtomEntry> = rows
        .into_iter()
        .map(|(id, title, author, words, chapters, status, source, created, description)| {
            let func = FicInfo::from_row(&(
                id.clone(), title.clone(), author.clone(), words,
                chapters, status.clone(), source.clone(),
                created.clone(), description.clone(),
            ));
            entry_for_fic(&func, &FeedQuery {
                url_id: id, title: title.clone(), author: author.clone(),
                words, chapters, status: status.clone(), source: source.clone(),
                created: created.clone(), description: description.clone(),
            })
        })
        .collect();

    let feed_xml = atom_feed(&entries, &iso_now());
    let response = axum::response::Response::builder()
        .header("Content-Type", "application/atom+xml; charset=utf-8")
        .body(feed_xml.into())
        .build()?;

    Ok((response, axum::http::StatusCode::OK))
}
```

> **⚠️ Watch Out**: The `atom_feed` function builds the full XML string — the handler fetches the rows, converts each to an `AtomEntry` via `entry_for_fic`, then calls `atom_feed` to serialize. The response has `Content-Type: application/atom+xml; charset=utf-8`.

#### 3. Follows feed — `GET /feed/follows.xml`

```rust
// src/routes/rss.rs (365 lines total)
pub async fn follows_feed(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<RssParams>,
) -> Result<(aiax::response::Response, axum::http::StatusCode), AppError> {
    let limit = params.limit.unwrap_or(20).min(100).max(1);

    // Fetch updates from followed works/authors
    // The feed joins follows + updates + fic_info
    let rows = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (u.url_id)
            u.url_id, u.title, u.author, u.words, u.chapters,
            u.status, u.source, u.created, u.description
        FROM updates u
        JOIN follows f ON (
            (f.follow_type = 'work' AND f.work_id = u.work_id) OR
            (f.follow_type = 'author' AND f.author_name = u.author_name)
        )
        WHERE f.user_id = $1
        ORDER BY u.url_id, u.created_at DESC
        "#,
    )
    .bind(auth.user_id.unwrap())
    .fetch_all(&state.db)
    .await?;

    let entries: Vec<AtomEntry> = rows.iter().map(|row| {
        let func = FicInfo::from_row(row);
        entry_for_fic(&func, &FeedQuery { ... })
    }).collect();

    let feed_xml = atom_feed(&entries, &iso_now());
    // ... build response with Content-Type: application/atom+xml
}
```

The follows feed uses `DISTINCT ON (url_id)` — the same dedup logic as the updates feed in Part 13. If you follow both a work and its author, you only see each update once.

#### 4. Per-fic feed — `GET /feed/works/{url_id}.xml`

```rust
// src/routes/rss.rs — per-fic feed
pub async fn work_feed(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<(aiax::response::Response, axum::http::StatusCode), AppError> {
    let func = get_fic_info(&state.db, &url_id).await?;
    let Some(func) = func else {
        return Err(AppError::NotFound(format!("fic not found: {}", url_id)));
    };

    let entry = entry_for_fic(&func, &FeedQuery { ... });
    let feed_xml = atom_feed(&[entry], &iso_now());

    // ... build response with Content-Type: application/atom+xml
}
```

> **💡 Key Concept**: The per-fic feed returns a single-entry Atom feed — just the most recent update for that fic. This is what e-book readers use to check for new chapters.

### Try It Yourself

```bash
# Subscribe to new arrivals
curl /feed.xml

# Subscribe to your follows
curl /feed/follows.xml -H "Authorization: Bearer <token>"

# Check a specific fic for updates
curl /feed/works/abc123def456.xml
```

### Check

- ✅ All three feeds return `Content-Type: application/atom+xml; charset=utf-8`.
- ✅ `atom_feed` builds the XML, `entry_for_fic` converts a fic to an entry.
- ✅ Follows feed uses `DISTINCT ON (url_id)` — dedup across work + author follows.
- ✅ Per-fic feed: single-entry Atom feed, most recent update only.

### What you built

The three RSS/Atom feed endpoints — new arrivals (most recently scraped), follows (your followed works/authors, deduped), and per-fic feed (single-entry, most recent update).

---

## Chapter 20.2 — OPDS Catalog

### Goal

Build the OPDS (Open Publication Distribution System) catalog — an XML-based catalog that e-book readers use to browse, search, and acquire fics.

### Actions

#### 1. The OPDS module structure

```rust
// src/routes/opds/mod.rs (250+ lines) — shared helpers
pub mod feeds;
pub mod authors;
pub mod tags;
pub mod search;

pub fn html_escape(s: &str) -> String { ... }           // escape HTML for XML
pub fn iso_now() -> String { ... }                        // RFC 3339 timestamp
pub fn clean_rfc3339(s: &str) -> String { ... }          // strip timezone for ISO 8601
```

#### 2. Root catalog — `GET /opds`

```rust
// src/routes/opds/mod.rs — root catalog
pub async fn root_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    // Fetch new, popular, and user's feed entries
    let new_entries = fetch_new_arrivals(&state.db)?;
    let popular_entries = fetch_popular(&state.db)?;
    let user_feed_entries = fetch_user_feed(&state, auth)?;

    let catalog_xml = build_catalog(&new_entries, &popular_entries, &user_feed_entries);
    // ... build response with Content-Type: application/atom+xml
}
```

The root catalog has 3 `<entry>` groups: new arrivals, popular, and user's feed. Each entry links to the full catalog (`/opds/new`, `/opds/popular`) and to the acquisition links for each fic.

#### 3. New arrivals catalog — `GET /opds/new`

```rust
// src/routes/opds/feeds.rs — new arrivals catalog
pub async fn new_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    let rows = sqlx::query_as(
        r#"
        SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters,
               fi.status, fi.source, fi.created, fi.description
        FROM fic_info fi ORDER BY fi.created DESC LIMIT 50
        "#,
    ).fetch_all(&state.db).await?;

    let entries = rows.iter().map(|row| {
        let func = FicInfo::from_row(row);
        build_entry(&func)
    }).collect();

    let catalog_xml = build_catalog_with_entries(&entries);
    // ... response
}
```

#### 4. Popular catalog — `GET /opds/popular`

```rust
// src/routes/opds/feeds.rs — popular fics catalog
pub async fn popular_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    let rows = sqlx::query_as(
        r#"
        SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters,
               fi.status, fi.source, fi.created, fi.description
        FROM fic_info fi
        ORDER BY (fi.words + fi.downloads * 100) DESC LIMIT 50
        "#,
    ).fetch_all(&state.db).await?;

    // ... build catalog with entries
}
```

> **⚠️ Watch Out**: Popular is a simple heuristic — `words + downloads * 100`. This favors longer, more-downloaded fics. A more sophisticated recommender (like the one in Part 19) could power a better popular list, but the simple heuristic works well for a start.

#### 5. Acquisition links — the key OPDS concept

Each OPDS entry has `<link rel="http://opds-spec.org/acquisition" ...>` links that e-book readers use to download the fic:

```xml
<entry>
  <title>Harry Potter and the Methods of Rationality</title>
  <author><name>Eliezer Yudkowsky</name></author>
  <id>urn:fichub:abc123def456</id>
  <updated>2024-01-15T00:00:00Z</updated>
  <link rel="http://opds-spec.org/acquisition" type="application/epub+zip"
        href="/cache/epub/abc123def456?h=abc123def456"></link>
  <link rel="http://opds-spec.org/acquisition" type="text/html"
        href="/work/abc123def456"></link>
</entry>
```

> **💡 Key Concept**: OPDS acquisition links point to `/cache/{format}/{url_id}?h={export_hash}` — the same cache download endpoints from Part 7. The e-book reader follows the acquisition link to download the EPUB directly. The `?h=` parameter is the export hash for cache validation.

---

## Chapter 20.3 — OPDS Authors and Tags Browsing

### Goal

Build the author and tag browsing catalogs that let users browse fics by author or tag in OPDS readers.

### Actions

#### 1. Authors catalog — `GET /opds/authors`

```rust
// src/routes/opds/authors.rs (full file, 200+ lines)
pub async fn authors_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    // Fetch distinct authors from fic_info, grouped
    let rows = sqlx::query_as(
        r#"
        SELECT DISTINCT fi.author, COUNT(*) as fic_count
        FROM fic_info fi
        GROUP BY fi.author
        ORDER BY fi.author ASC
        LIMIT 200
        "#,
    ).fetch_all(&state.db).await?;

    let entries = rows.iter().map(|row| {
        // Each author entry links to /opds/tags/1/{author_name} (author tag listing)
        build_author_entry(row.0, row.1)
    }).collect();

    let catalog_xml = build_catalog_with_entries(&entries);
    // ... response
}
```

#### 2. Tags catalog — `GET /opds/tags`

```rust
// src/routes/opds/tags.rs — tags catalog
pub async fn tags_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    // Fetch all distinct tag types (Character, Relationship, Fandom, etc.)
    let type_rows = sqlx::query_as(
        r#"
        SELECT id, name FROM tag_types ORDER BY id ASC
        "#,
    ).fetch_all(&state.db).await?;

    // For each tag type, an entry linking to /opds/tags/{type_id}
    let type_entries: Vec<Value> = type_rows.iter().map(|t| {
        json!({
            "id": t.id,
            "name": t.name,
        })
    }.collect();

    // Build catalog with one entry per tag type
    // ... response
}
```

#### 3. Tag-type fics catalog — `GET /opds/tags/{type_id}`

```rust
// src/routes/opds/tags.rs — fics with a specific tag type
pub async fn fics_by_tag_type(
    State(state): State<Arc<AppState>>,
    Path(type_id): Path<i16>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    // Fetch all fics that have at least one tag of this type
    let rows = sqlx::query_as(
        r#"
        SELECT DISTINCT fi.id, fi.title, fi.author, fi.words, fi.chapters,
               fi.status, fi.source, fi.created, fi.description
        FROM fic_info fi
        JOIN fic_tags ft ON ft.url_id = fi.id
        JOIN tags t ON t.id = ft.tag_id
        WHERE t.tag_type_id = $1
        ORDER BY fi.created DESC
        LIMIT 200
        "#,
    ).bind(type_id).fetch_all(&state.db).await?;

    // Build catalog entries from each fic
    // ... response
}
```

> **⚠️ Watch Out**: The tag-type browse uses `DISTINCT` — a fic with multiple tags of the same type only appears once. The join is `fic_info JOIN fic_tags JOIN tags` — this is a 3-table join that can be slow for large catalogs. For production, consider caching the result.

#### 4. Specific tag fics — `GET /opds/tags/{type_id}/{tag_name}`

```rust
// src/routes/opds/tags.rs — fics with a specific tag
pub async fn fics_by_tag(
    State(state): State<Arc<AppState>>,
    Path((type_id, tag_name)): Path<(i16, String)>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    // Fetch fics with this specific tag
    let rows = sqlx::query_as(
        r#"
        SELECT DISTINCT fi.id, fi.title, fi.author, fi.words, fi.chapters,
               fi.status, fi.source, fi.created, fi.description
        FROM fic_info fi
        JOIN fic_tags ft ON ft.url_id = fi.id
        JOIN tags t ON t.id = ft.tag_id
        WHERE t.tag_type_id = $1 AND t.name = $2
        ORDER BY fi.created DESC
        LIMIT 200
        "#,
    ).bind(type_id).bind(&tag_name).fetch_all(&state.db).await?;

    // Build catalog entries
    // ... response
}
```

---

## Chapter 20.4 — OPDS Search

### Goal

Build the OPDS search endpoint that lets e-book readers search the catalog from within the reader.

### Actions

#### 1. Search handler

```rust
// src/routes/opds/search.rs — OPDS search
pub async fn search_catalog(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OPDSQueryParams>,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    let query = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20).min(100).max(1);
    let offset = params.offset.unwrap_or(0);

    if query.is_empty() {
        // Return empty catalog
        return Ok(build_empty_catalog());
    }

    // Full-text search on title + author
    let rows = sqlx::query_as(
        r#"
        SELECT DISTINCT fi.id, fi.title, fi.author, fi.words, fi.chapters,
               fi.status, fi.source, fi.created, fi.description
        FROM fic_info fi
        WHERE fi.title ILIKE $1 OR fi.author ILIKE $1
        ORDER BY fi.created DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(format!("%{}%", query))
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    // Build catalog entries from results
    // ... response
}
```

#### 2. Shelves catalog

```rust
// src/routes/opds/mod.rs — user's shelves in OPDS
pub async fn shelves_catalog(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    let shelves = queries::list_shelves(&state.db, user_id).await?;

    // Each shelf entry links to /opds/shelf/{shelf_id}
    // ... build catalog
}

pub async fn shelf_catalog(
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
    auth: AuthUser,
) -> Result<(axum::response::Response, axum::http::StatusCode), AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into()))?;

    // Verify shelf belongs to user
    let shelf = queries::get_shelf(&state.db, shelf_id, user_id).await?
        .ok_or_else(|| AppError::NotFound("Shelf not found".into()))?;

    // Fetch works in shelf
    let entries = queries::list_works_in_shelf(&state.db, shelf_id).await?;

    // Build catalog entries from shelf works
    // ... response
}
```

---

## Chapter 20.5 — Share Links and Integration

### Goal

Build the share links that connect RSS/OPDS to other tools — Add to Calibre, Add to KOReader, and the web share links.

### Actions

#### 1. Share links on the work page

```svelte
<!-- On the work page, under the download buttons -->
<div class="share-links">
    <h3>Subscribe & Share</h3>
    <ul class="share-list">
        <li><a href="/feed/works/{url_id}.xml">Subscribe in Feed Reader</a></li>
        <li><a href="/opds?url_id={url_id}">Add to KOReader</a></li>
        <li><a href="calibre://open?url={encodeURIComponent('/opds/' + url_id)}">Open in Calibre</a></li>
        <li><a href="https://www.feedly.com/#feed/https://fichub.net/feed/works/{url_id}.xml">Add to Feedly</a></li>
    </ul>
</div>
```

#### 2. The "Add to OPDS reader" share link

```typescript
// frontend/src/routes/work/[workId]/+page.svelte (simplified)
// The OPDS share link for KOReader and other OPDS-capable readers
const opdsUrl = `/opds?url_id=${url_id}`;
const opdsAcquisitionUrl = `/cache/epub/${url_id}?h=${exportHash}`;
```

The share links let users:
- **Subscribe in feed readers** — Point their Feedly/NewsBlur/Reader at `/feed/works/{url_id}.xml`
- **Add to KOReader** — KOReader supports OPDS. Point it at `/opds?url_id={url_id}`.
- **Open in Calibre** — Calibre's OPDS plugin can fetch the feed directly.
- **Share via URL** — The `/opds` URL is the canonical shareable OPDS catalog URL.

> **💡 Key Concept**: OPDS is the "USB cable" of the e-book world — it lets readers browse and acquire content directly from the server without a web browser. FicHub's OPDS catalog is what makes it work with KOReader, Calibre, and other OPDS-capable readers.

---

## Conclusion

You now understand FicHub's feed system:

1. **RSS/Atom feeds** — new arrivals, follows feed (deduped), per-fic feed (single-entry). All return `application/atom+xml`.
2. **OPDS catalog** — root catalog (new/popular/feed), new arrivals, popular, author browse, tag-type browse, specific tag fics, search, shelves.
3. **Acquisition links** — each entry links to `/cache/{format}/{url_id}?h={export_hash}` for direct download in e-book readers.
4. **Share links** — Feed Reader, KOReader, Calibre, Feedly integration via URL.
5. **OPDS as the "USB cable"** — e-book readers browse and acquire directly from the server via the OPDS catalog.

The feeds and OPDS catalog connect FicHub to the broader e-book ecosystem, making it work with the tools readers already use.
