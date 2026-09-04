# Part 7: Advanced Features

---

# Chapter 29: Search System

## Full-Text Search

FicHub's search system goes beyond simple string matching. It uses PostgreSQL's full-text search capabilities with weighted ranking, complex filters, and tag-based queries.

## The text_search Column

The `text_search` column is a generated tsvector:

```sql
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

**Weights:**
- A (title) — Higher relevance
- B (description) — Lower relevance

The `to_tsvector` function tokenizes text and applies English stemming ("running" → "run"). The GIN index makes queries fast.

## Search Parameters

```rust
pub struct SearchParams {
    pub q: Option<String>,              // Full-text search query
    pub include_tags: Vec<TagFilter>,   // ALL must match
    pub exclude_tags: Vec<TagFilter>,   // NONE must match
    pub include_any_tags: Vec<TagFilter>, // At least ONE
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
```

## The SearchQueryBuilder

```rust
pub struct SearchQueryBuilder {
    pub params: SearchParams,
    hidden_threshold: i16,
}
```

### Building the Count Query

```rust
pub fn build_count_query(&self) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM fic_info fi WHERE 1=1");
    self.push_where_clauses(&mut qb);
    qb
}
```

### Building the Data Query

```rust
pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
    let has_q = self.params.q.is_some();
    let mut qb = QueryBuilder::new("SELECT fi.*, ");

    if let Some(ref q) = self.params.q {
        qb.push("ts_rank(fi.text_search, plainto_tsquery('english', ");
        qb.push_bind(q);
        qb.push(")) AS rank FROM fic_info fi WHERE 1=1");
    } else {
        qb.push("NULL::real AS rank FROM fic_info fi WHERE 1=1");
    }

    self.push_where_clauses(&mut qb);

    // ORDER BY
    let sort = self.params.sort.as_deref().unwrap_or(
        if has_q { "-relevance" } else { "-date" }
    );
    qb.push(" ORDER BY ");
    match sort {
        "-relevance" if has_q => qb.push("rank DESC"),
        "-date" => qb.push("fi.fic_updated DESC"),
        "-words" => qb.push("fi.words DESC"),
        "-chapters" => qb.push("fi.chapters DESC"),
        "-title" => qb.push("fi.title ASC"),
        _ => qb.push("fi.fic_updated DESC"),
    }

    // LIMIT / OFFSET
    let page = self.params.page.unwrap_or(1).max(1);
    let per_page = self.params.per_page.unwrap_or(20).max(1);
    let offset = (page - 1) * per_page;
    qb.push(" LIMIT ");
    qb.push_bind(per_page as i64);
    qb.push(" OFFSET ");
    qb.push_bind(offset as i64);

    qb
}
```

### WHERE Clauses

```rust
fn push_where_clauses(&self, qb: &mut QueryBuilder<Postgres>) {
    // Full-text search
    if let Some(ref q) = self.params.q {
        qb.push(" AND fi.text_search @@ plainto_tsquery('english', ");
        qb.push_bind(q);
        qb.push(")");
    }

    // include_tags — ALL must match
    for tag in &self.params.include_tags {
        qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND t.name = ");
        qb.push_bind(&tag.tag_name);
        qb.push(" AND t.tag_type_id = ");
        qb.push_bind(tag.tag_type_id);
        qb.push(" AND ft.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(")");
    }

    // exclude_tags — NONE must match
    for tag in &self.params.exclude_tags {
        qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND t.name = ");
        qb.push_bind(&tag.tag_name);
        qb.push(" AND t.tag_type_id = ");
        qb.push_bind(tag.tag_type_id);
        qb.push(")");
    }

    // include_any_tags — at least ONE must match
    if !self.params.include_any_tags.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND ft.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(" AND (");
        let mut first = true;
        for tag in &self.params.include_any_tags {
            if !first { qb.push(" OR"); }
            first = false;
            qb.push(" (t.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(" AND t.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(")");
        }
        qb.push("))");
    }

    // Word count range
    if let Some(min) = self.params.min_words {
        qb.push(" AND fi.words >= ");
        qb.push_bind(min);
    }
    if let Some(max) = self.params.max_words {
        qb.push(" AND fi.words <= ");
        qb.push_bind(max);
    }

    // Chapter count range
    if let Some(min) = self.params.min_chapters {
        qb.push(" AND fi.chapters >= ");
        qb.push_bind(min);
    }
    if let Some(max) = self.params.max_chapters {
        qb.push(" AND fi.chapters <= ");
        qb.push_bind(max);
    }

    // Completion status
    if let Some(true) = self.params.complete {
        qb.push(" AND fi.status = 'complete'");
    }
    if let Some(false) = self.params.complete {
        qb.push(" AND fi.status != 'complete'");
    }

    // Source filter
    if let Some(ref source) = self.params.source {
        qb.push(" AND fi.source = ");
        qb.push_bind(source);
    }

    // Date range
    if let Some(ref dt) = self.params.date_from {
        qb.push(" AND fi.fic_updated >= ");
        qb.push_bind(dt);
    }
    if let Some(ref dt) = self.params.date_to {
        qb.push(" AND fi.fic_updated <= ");
        qb.push_bind(dt);
    }
}
```

## Tag Filter Parsing

```rust
pub fn parse_tag_filters(input: &str) -> Result<Vec<TagFilter>, String> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let mut filters = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        let colon_pos = part.find(':').ok_or_else(|| {
            format!("Invalid format '{}': expected 'type_id:name'", part)
        })?;
        let type_id: i16 = part[..colon_pos].parse()
            .map_err(|e| format!("Invalid type_id in '{}': {}", part, e))?;
        let name = part[colon_pos + 1..].to_string();
        filters.push(TagFilter { tag_type_id: type_id, tag_name: name });
    }
    Ok(filters)
}
```

Format: `1:Harry Potter,2:Hermione Granger,4:Angst`

## The Search Handler

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let search_params = params.into_search_params()?;
    let per_page = search_params.per_page.unwrap_or(20)
        .min(state.config.search_max_per_page).max(1);

    let builder = SearchQueryBuilder::new(capped_params, state.config.tag_hidden_threshold);

    // Count query
    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query.build_query_as().fetch_one(&state.db).await?;

    // Data query
    let mut data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query.build_query_as().fetch_all(&state.db).await?;

    // Batch-fetch tags
    let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let tag_rows = sqlx::query_as::<_, TagDbRow>(
        r#"SELECT ft.url_id, t.name, t.tag_type_id, tt.name AS type_name, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           JOIN tag_types tt ON tt.id = t.tag_type_id
           WHERE ft.url_id = ANY($1) AND ft.score >= $2
           ORDER BY ft.url_id, ft.score DESC"#,
    ).bind(&url_ids).bind(state.config.tag_hidden_threshold)
     .fetch_all(&state.db).await?;

    // Group tags by url_id
    let mut tags_by_fic: HashMap<String, Vec<TagDbRow>> = HashMap::new();
    for tag_row in tag_rows {
        tags_by_fic.entry(tag_row.url_id.clone()).or_default().push(tag_row);
    }

    // Build results
    let results: Vec<SearchResultData> = rows.into_iter().map(|row| {
        let fic_tags = tags_by_fic.remove(&row.id).unwrap_or_default();
        let response_tags: Vec<Value> = fic_tags.into_iter().map(|t| {
            json!({
                "name": t.name,
                "type": t.type_name,
                "type_id": t.tag_type_id,
                "score": t.score,
            })
        }).collect();
        SearchResultData {
            url_id: row.id,
            title: row.title,
            author: row.author,
            words: row.words,
            chapters: row.chapters,
            status: row.status,
            description: row.description,
            rank: row.rank,
            tags: response_tags,
            // ...
        }
    }).collect();

    Ok(Json(json!({
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": results,
    })))
}
```

## Watch Out!

**Search is case-insensitive!** `plainto_tsquery` handles case automatically.

**Tag filters use the hidden threshold!** Tags below the threshold are excluded.

**Pagination is essential!** Without LIMIT/OFFSET, large result sets would be slow.

## Summary

FicHub's search uses PostgreSQL full-text search with weighted ranking, dynamic query building with tag filters, batch tag fetching, and proper pagination.

---

# Chapter 30: Tag System

## Tag Types

| ID | Name | Example |
|----|------|---------|
| 1 | fandom | Harry Potter |
| 2 | character | Hermione Granger |
| 3 | relationship | Harry/Hermione |
| 4 | freeform | Angst, Fluff |
| 5 | warning | Major Character Death |
| 6 | category | M/M, F/M |
| 7 | other | Alternate Universe |

## Tag Resolution

```rust
pub async fn resolve_tag(pool: &PgPool, tag_name: &str, tag_type_id: i16) -> AppResult<TagResolution> {
    // 1. Exact match
    if let Some(row) = lookup_tag(pool, tag_name).await? {
        return Ok(TagResolution { tag_id: row.0, tag_name: row.1, tag_type_id: row.2, is_new: false });
    }

    // 2. Alias match
    if let Some(canonical_id) = lookup_alias(pool, tag_name).await? {
        let tag = lookup_tag_by_id(pool, canonical_id).await?;
        return Ok(TagResolution { tag_id: canonical_id, tag_name: tag.1, tag_type_id: tag.2, is_new: false });
    }

    // 3. Create new
    let new_id = create_tag(pool, tag_name, tag_type_id).await?;
    Ok(TagResolution { tag_id: new_id, tag_name: tag_name.to_string(), tag_type_id, is_new: true })
}
```

## Score Management via Triggers

```sql
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

Three triggers fire on INSERT, UPDATE, and DELETE on `fic_tag_votes`.

## Visibility

```rust
pub fn is_hidden(score: i16, threshold: i16) -> bool {
    score <= threshold
}
```

Default threshold: -3. Tags need 3+ net downvotes to be hidden.

## Curator Tools

### Create Alias

```rust
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateAliasBody>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    sqlx::query(
        "INSERT INTO tag_aliases (alias_name, canonical_tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    ).bind(&body.alias_name).bind(body.canonical_tag_id)
     .execute(&state.db).await?;

    Ok(Json(json!({"err": 0, "msg": "alias created"})))
}
```

### Merge Tags

```rust
pub async fn merge_tags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MergeTagsBody>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    if body.source_tag_id == body.target_tag_id {
        return Ok(Json(json!({"err": -1, "msg": "cannot merge tag into itself"})));
    }

    // Migrate fic_tags
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    ).bind(body.source_tag_id).bind(body.target_tag_id)
     .execute(&state.db).await?;

    // Delete source
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(body.source_tag_id).execute(&state.db).await?;
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(body.source_tag_id).execute(&state.db).await?;

    Ok(Json(json!({"err": 0, "msg": "tags merged"})))
}
```

### Delete Tag

```rust
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id).execute(&state.db).await?;

    if result.rows_affected() == 0 {
        return Ok(Json(json!({"err": -5, "msg": "tag not found"})));
    }

    Ok(Json(json!({"err": 0, "msg": "tag deleted"})))
}
```

### Flag Management

```rust
pub async fn list_flags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<FlagListQuery>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    let resolved_filter = params.resolved.unwrap_or(false);

    let rows: Vec<FlagRow> = sqlx::query_as::<_, FlagRow>(
        r#"SELECT id, url_id, tag_id, flagged_by_ip::text, reason, resolved, created_at
           FROM tag_flags WHERE resolved = $1 ORDER BY created_at DESC"#,
    ).bind(resolved_filter).fetch_all(&state.db).await?;

    let flags: Vec<Value> = rows.into_iter().map(|r| {
        json!({
            "id": r.id,
            "url_id": r.url_id,
            "tag_id": r.tag_id,
            "reason": r.reason,
            "resolved": r.resolved,
        })
    }).collect();

    Ok(Json(json!({"err": 0, "flags": flags})))
}
```

## Watch Out!

**Tag names are case-sensitive!** `COLLATE "C"` means "Harry Potter" ≠ "harry potter".

**Merging is destructive!** Always double-check before merging.

**The trigger handles scores!** Don't manually update `fic_tags.score`.

## Summary

FicHub's tag system supports community-submitted tags with voting, visibility thresholds, curator tools, and PostgreSQL triggers for automatic score management.

---

# Chapter 31: OPDS Catalog

## What is OPDS?

OPDS (Open Publication Distribution System) is a standard for distributing e-books using Atom XML feeds. E-readers like Kindle, Kobo, and Calibre can subscribe to OPDS catalogs.

## The Root Catalog

```xml
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <id>urn:fichub:catalog</id>
  <title>FicHub</title>
  <updated>2024-01-15T10:30:00Z</updated>
  <entry>
    <title>Recent Fics</title>
    <link href="/opds/new" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:new</id>
    <summary>Recently added and updated fanfiction</summary>
  </entry>
  <entry>
    <title>Popular Fics</title>
    <link href="/opds/popular" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:popular</id>
  </entry>
  <entry>
    <title>Tags</title>
    <link href="/opds/tags" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
    <id>urn:fichub:catalog:tags</id>
  </entry>
  <entry>
    <title>Authors</title>
    <link href="/opds/authors" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
    <id>urn:fichub:catalog:authors</id>
  </entry>
  <entry>
    <title>Recommendations</title>
    <link href="/opds/recommendations/popular" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:recommendations</id>
  </entry>
  <entry>
    <title>Search</title>
    <link href="/opds/search?q=" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:search</id>
  </entry>
</feed>
```

## Feed Types

**Navigation feeds** list categories:
```xml
<link href="/opds/new" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
```

**Acquisition feeds** list downloadable books with download links.

## Fic Entry

```rust
pub fn fic_entry(
    url_id: &str, title: &str, author: &str, summary: &str,
    updated: &str, words: i64, chapters: i32, status: &str,
) -> String {
    format!(
        r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>urn:fichub:fic:{url_id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <dc:extent>{words}</dc:extent>
    <dc:format>application/epub+zip</dc:format>
    <category term="{status}" label="{status}"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/epub/{url_id}/{url_id}.epub" type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/pdf/{url_id}/{url_id}.pdf" type="application/pdf"/>
  </entry>"#,
        title = html_escape(title),
        author = html_escape(author),
        summary = html_escape(summary),
        // ...
    )
}
```

## Pagination

```rust
pub fn build_feed(
    title: &str, feed_id: &str, entries: &str, updated: &str,
    self_link: Option<&str>, feed_kind: FeedKind,
    pagination: Option<&PaginationInfo>,
) -> String {
    let mut links = String::new();

    if let Some(ref pagi) = pagination {
        let total_pages = (pagi.total as f64 / pagi.per_page as f64).ceil() as usize;
        if pagi.page > 1 {
            links.push_str(&format!(
                r#"  <link href="{base}?page={prev}" rel="previous"/>"#,
                base = pagi.base_path, prev = pagi.page - 1
            ));
        }
        if pagi.page < total_pages {
            links.push_str(&format!(
                r#"  <link href="{base}?page={next}" rel="next"/>"#,
                base = pagi.base_path, next = pagi.page + 1
            ));
        }
    }

    format!(
        r#"{header}<id>{id}</id>
  <title>{title}</title>
  <updated>{updated}</updated>
  <author><name>FicHub</name></author>
{links}{entries}</feed>"#,
        // ...
    )
}
```

## Shelf Authentication

```rust
pub async fn shelf_contents(
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
    Query(params): Query<ShelfQuery>,
) -> Result<impl IntoResponse, AppError> {
    let token = params.token.as_deref().unwrap_or("");
    if token != state.config.opds_shelf_token {
        return Err(AppError::BadRequest(-1, "invalid token".into()));
    }
    // ... return shelf contents ...
}
```

## Watch Out!

**Content-Type headers are critical!** Wrong headers confuse e-readers.

**XML escaping is mandatory!** All user content must be XML-escaped.

## Summary

FicHub's OPDS catalog provides a standard interface for e-readers with navigation feeds, acquisition feeds, pagination, and token-based shelf authentication.

---

# Chapter 32: API Documentation

## Self-Documenting API

FicHub provides API documentation at `/api/`:

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
            // ...
        }
    }))
}
```

## Response Format

```json
{
    "err": 0,
    "msg": "...",
    "data": { ... }
}
```

| Code | Meaning |
|------|---------|
| 0 | Success |
| -1 | Bad request |
| -5 | Not found |
| -6 | Scraper error |
| -7 | Blacklisted |
| -10 | Automated blocked |
| -429 | Rate limited |

## Summary

FicHub's API documentation is self-hosted and follows a consistent format with error codes.

---

# Chapter 33: Performance

## Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(10))
    .connect(database_url)
    .await?;
```

## Caching Layers

1. **Database cache** — `export_log` table
2. **Disk cache** — EPUB/HTML files on disk
3. **Redis cache** — Rate limiter state
4. **Precomputed recommendations** — Cached results

## Profiling

```bash
cargo install flamegraph
cargo flamegraph -- -c ./fichub
```

## Database Query Performance

```sql
EXPLAIN ANALYZE SELECT ... FROM fic_info WHERE text_search @@ plainto_tsquery('english', 'harry potter');
```

## Async Performance

- **Connection reuse** — `reqwest::Client` reuses TCP connections
- **Semaphore control** — Prevents duplicate work
- **Redis multiplexing** — Concurrent Redis operations

## Watch Out!

**Don't block the async runtime!** Use `spawn_blocking` for sync operations.

**Monitor memory usage!** Use `valgrind` or `heaptrack` for leak detection.

**Connection pool exhaustion!** Keep database operations short.

## Summary

FicHub's performance relies on connection pooling, multi-layered caching, async I/O, and careful resource management.
