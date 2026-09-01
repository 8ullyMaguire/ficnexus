# Part 7: Advanced Backend Features

---

# Chapter 29: The Search System

FicHub's search system provides full-text search with tag filters, word count ranges, and more. This chapter covers the search query builder, PostgreSQL full-text search, and the search API.

## Full-Text Search with PostgreSQL

PostgreSQL has built-in full-text search using `tsvector` and `tsquery`:

```sql
-- Convert text to tsvector
SELECT to_tsvector('english', title || ' ' || author) FROM fic_info;

-- Search using tsquery
SELECT * FROM fic_info
WHERE to_tsvector('english', title || ' ' || author) @@ plainto_tsquery('english', 'harry potter');
```

The `plainto_tsquery` function converts a search string into a query. The `@@` operator checks if a tsvector matches a tsquery.

### FicHub's Search Implementation

```rust
use sqlx::QueryBuilder;
use sqlx::Postgres;

pub struct SearchQueryBuilder {
    params: SearchParams,
    max_per_page: usize,
}

#[derive(Debug, Default)]
pub struct SearchParams {
    pub q: Option<String>,
    pub include_tags: Vec<TagFilter>,
    pub exclude_tags: Vec<TagFilter>,
    pub include_any_tags: Vec<TagFilter>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
}

impl SearchQueryBuilder {
    pub fn new(params: SearchParams, max_per_page: usize) -> Self {
        SearchQueryBuilder { params, max_per_page }
    }
    
    pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
        let mut qb = QueryBuilder::new(
            "SELECT fi.id, fi.title, fi.author, fi.chapters, fi.words, \
             fi.description, fi.status, fi.source, \
             fi.fic_created, fi.fic_updated, fi.extra_meta, \
             fi.raw_extended_meta, fi.source_id, fi.author_id, \
             fi.content_hash"
        );
        
        // Add rank if there's a search query
        if self.params.q.is_some() {
            qb.push(", ts_rank(to_tsvector('english', fi.title || ' ' || fi.author), \
                      plainto_tsquery('english', ");
            qb.push_bind(self.params.q.clone().unwrap_or_default());
            qb.push(")) AS rank");
        } else {
            qb.push(", NULL::real AS rank");
        }
        
        qb.push(" FROM fic_info fi WHERE 1=1");
        
        // Add search query filter
        if let Some(ref q) = self.params.q {
            qb.push(" AND to_tsvector('english', fi.title || ' ' || fi.author) \
                      @@ plainto_tsquery('english', ");
            qb.push_bind(q.clone());
            qb.push(")");
        }
        
        // Add tag filters
        self.add_tag_filters(&mut qb);
        
        // Add word count filters
        if let Some(min) = self.params.min_words {
            qb.push(" AND fi.words >= ");
            qb.push_bind(min);
        }
        if let Some(max) = self.params.max_words {
            qb.push(" AND fi.words <= ");
            qb.push(" AND fi.words <= ");
            qb.push_bind(max);
        }
        
        // Add completion filter
        if let Some(true) = self.params.complete {
            qb.push(" AND fi.status = 'complete'");
        }
        if let Some(false) = self.params.complete {
            qb.push(" AND fi.status != 'complete'");
        }
        
        // Add source filter
        if let Some(ref source) = self.params.source {
            qb.push(" AND fi.source = ");
            qb.push_bind(source);
        }
        
        // Add sorting
        match self.params.sort.as_deref() {
            Some("-relevance") if self.params.q.is_some() => {
                qb.push(" ORDER BY rank DESC");
            }
            Some("-words") => qb.push(" ORDER BY fi.words DESC"),
            Some("-chapters") => qb.push(" ORDER BY fi.chapters DESC"),
            Some("-title") => qb.push(" ORDER BY fi.title ASC"),
            _ => qb.push(" ORDER BY fi.fic_updated DESC"),
        }
        
        // Add pagination
        let limit = self.max_per_page.min(50);
        let offset = (self.page().saturating_sub(1)) as i64 * limit as i64;
        qb.push(" LIMIT ");
        qb.push_bind(limit as i64);
        qb.push(" OFFSET ");
        qb.push_bind(offset);
        
        qb
    }
    
    fn add_tag_filters(&self, qb: &mut QueryBuilder<Postgres>) {
        // Include tags (must have ALL)
        for tag in &self.params.include_tags {
            qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft \
                      JOIN tags t ON t.id = ft.tag_id \
                      WHERE ft.url_id = fi.id \
                      AND t.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(" AND t.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(" AND ft.score >= 0)");
        }
        
        // Exclude tags (must not have ANY)
        for tag in &self.params.exclude_tags {
            qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft \
                      JOIN tags t ON t.id = ft.tag_id \
                      WHERE ft.url_id = fi.id \
                      AND t.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(" AND t.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(")");
        }
        
        // Include any tags (must have at least ONE)
        if !self.params.include_any_tags.is_empty() {
            qb.push(" AND (");
            for (i, tag) in self.params.include_any_tags.iter().enumerate() {
                if i > 0 {
                    qb.push(" OR ");
                }
                qb.push("EXISTS (SELECT 1 FROM fic_tags ft \
                          JOIN tags t ON t.id = ft.tag_id \
                          WHERE ft.url_id = fi.id \
                          AND t.tag_type_id = ");
                qb.push_bind(tag.tag_type_id);
                qb.push(" AND t.name = ");
                qb.push_bind(&tag.tag_name);
                qb.push(" AND ft.score >= 0)");
            }
            qb.push(")");
        }
    }
}
```

## The Search API

```rust
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub tags: Option<String>,      // "1:Harry Potter,2:Hermione"
    pub exclude_tags: Option<String>,
    pub any_tags: Option<String>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
}

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Value>, AppError> {
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
    
    Ok(Json(json!({
        "err": 0,
        "total": count.0,
        "page": builder.page(),
        "per_page": state.config.search_max_per_page,
        "results": results
    })))
}
```

## 📝 Practice Exercises

1. **Search Autocomplete:** Implement a search autocomplete feature that suggests titles as the user types.

2. **Search Analytics:** Track popular search queries and zero-result searches.

3. **Advanced Filters:** Add date range filters, rating filters, and kudos count filters.

---

# Chapter 30: The Tag System

FicHub's tag system allows users to add, vote on, and moderate tags for stories. This chapter covers tag resolution, voting, and curator tools.

## Tag Resolution

When a user submits a tag, it needs to be resolved to a canonical tag:

```rust
pub struct TagResolution {
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_type_id: i16,
    pub is_new: bool,
}

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
        // Get the canonical tag details
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

## Tag Voting

```rust
pub struct VoteResult {
    pub new_score: i16,
    pub hidden: bool,
}

pub async fn record_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
    value: i16,
    hidden_threshold: i16,
) -> AppResult<VoteResult> {
    // Check for existing vote
    let existing = queries::get_existing_vote(pool, url_id, tag_id, voter_ip).await?;
    
    // Upsert vote
    queries::upsert_tag_vote(pool, url_id, tag_id, voter_ip, value).await?;
    
    // Compute new score
    let new_score = queries::get_tag_score(pool, url_id, tag_id).await?;
    
    // Update the fic_tags score
    sqlx::query(
        "UPDATE fic_tags SET score = $1 WHERE url_id = $2 AND tag_id = $3"
    )
    .bind(new_score)
    .bind(url_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    
    Ok(VoteResult {
        new_score,
        hidden: new_score < hidden_threshold,
    })
}
```

## Curator Tools

Curators can manage tags through admin endpoints:

```rust
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<CreateAliasRequest>,
) -> Result<Json<Value>, AppError> {
    // Verify curator token
    verify_curator_token(&headers, &state.config)?;
    
    // Resolve the canonical tag
    let resolution = resolve_tag(
        &state.db, &payload.canonical_name, payload.tag_type_id
    ).await?;
    
    // Create the alias
    queries::create_tag_alias(&state.db, &payload.alias_name, resolution.tag_id).await?;
    
    Ok(Json(json!({
        "err": 0,
        "alias": payload.alias_name,
        "canonical": resolution.tag_name,
        "msg": "alias created"
    })))
}

pub async fn merge_tags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<MergeTagsRequest>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config)?;
    
    queries::merge_tags(&state.db, payload.source_tag_id, payload.target_tag_id).await?;
    
    Ok(Json(json!({
        "err": 0,
        "msg": "tags merged"
    })))
}
```

## 📝 Practice Exercises

1. **Tag Statistics:** Build a function that returns tag usage statistics (most used tags, average score per tag type).

2. **Tag Suggestions:** Implement tag auto-suggestion based on story content.

3. **Bulk Tag Operations:** Build tools for bulk tag operations (rename, merge, delete).

---

# Chapter 31: The OPDS Catalog

OPDS (Open Publication Distribution System) is a standard for e-book catalogs. This chapter covers how FicHub publishes an OPDS catalog that e-reader apps can browse.

## OPDS Basics

OPDS feeds are Atom XML feeds with specific extensions for e-book metadata. An e-reader app can subscribe to the OPDS catalog and browse/download stories directly.

**Real-world analogy:** OPDS is like an RSS feed for books. Just as you can subscribe to a blog feed and read articles in your feed reader, you can subscribe to an OPDS catalog and browse books in your e-reader.

## OPDS Feed Generation

```rust
use axum::response::IntoResponse;

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
    
    <entry>
        <title>Search</title>
        <link href="/opds/search" type="application/opensearchdescription+xml"/>
        <id>urn:fichub:search</id>
        <updated>2024-01-01T00:00:00Z</updated>
    </entry>
</feed>"#;
    
    (
        [("Content-Type", "application/atom+xml;profile=opds-catalog")],
        xml
    )
}
```

### OPDS Entry for a Story

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
    <link rel="http://opds-spec.org/thumbnail" href="/api/v0/meta?url_id={}&amp;thumbnail=true" type="image/jpeg"/>
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
        story.id,
    )
}
```

## 📝 Practice Exercises

1. **OPDS Navigation:** Implement multi-level OPDS navigation (categories → subcategories → stories).

2. **OPDS Search:** Implement the OpenSearch description document for OPDS search.

3. **OPDS Authentication:** Add authentication support for private shelves.

---

# Chapter 32: The API Documentation

FicHub provides a self-documenting API. This chapter covers the API docs endpoint and OpenAPI integration.

## The API Docs Endpoint

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "description": "Export a story as EPUB",
                "params": {
                    "q": "URL of the story to export"
                },
                "response": {
                    "err": "0 for success",
                    "url_id": "Unique story identifier",
                    "meta": "Story metadata",
                    "urls": "Download URLs"
                }
            },
            "/api/v0/meta": {
                "method": "GET",
                "description": "Get story metadata",
                "params": {
                    "url_id": "Story identifier"
                }
            },
            "/api/v0/search": {
                "method": "GET",
                "description": "Search for stories",
                "params": {
                    "q": "Search query",
                    "tags": "Include tags (type:name,...)",
                    "exclude_tags": "Exclude tags",
                    "min_words": "Minimum word count",
                    "max_words": "Maximum word count",
                    "complete": "Filter by completion status",
                    "sort": "Sort order (-relevance, -words, -date)",
                    "page": "Page number"
                }
            }
        }
    }))
}
```

## 📝 Practice Exercises

1. **OpenAPI Spec:** Generate an OpenAPI 3.0 specification for FicHub's API.

2. **Interactive Docs:** Integrate Swagger UI or Redoc for interactive API documentation.

3. **API Changelog:** Add a changelog endpoint that documents API changes.

---

# Chapter 33: Performance and Optimization

Performance is critical for a web server. This chapter covers profiling, caching strategies, and optimization techniques.

## Profiling with cargo-flamegraph

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate a flamegraph
cargo flamegraph --bench my_benchmark
```

Flamegraphs visualize where your program spends time. Wide bars mean more time spent in that function.

## Database Query Optimization

### EXPLAIN ANALYZE

```sql
EXPLAIN ANALYZE
SELECT fi.id, fi.title, fi.author, fi.words
FROM fic_info fi
WHERE fi.words > 10000
AND fi.status = 'complete'
ORDER BY fi.fic_updated DESC
LIMIT 20;
```

This shows the query plan and actual execution time. Look for:
- **Seq Scan** — Full table scan (slow for large tables)
- **Index Scan** — Using an index (fast)
- **Nested Loop** — Joining tables (check if indexes are used)

### Index Strategy

```sql
-- Add indexes for common query patterns
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_fic_updated ON fic_info(fic_updated);
CREATE INDEX idx_fic_info_source ON fic_info(source);

-- Composite index for common filter combinations
CREATE INDEX idx_fic_info_status_words ON fic_info(status, words);

-- Partial index for active stories
CREATE INDEX idx_fic_info_active ON fic_info(fic_updated)
WHERE status = 'ongoing';
```

## Connection Pool Tuning

```rust
// Monitor pool usage
let stats = pool.stats();
tracing::info!(
    active = stats.active_connections(),
    idle = stats.idle_connections(),
    waiting = stats.waiting(),
    "Pool stats"
);
```

### Tuning Guidelines

| Metric | Small Deployment | Large Deployment |
|--------|-----------------|------------------|
| max_connections | 10-20 | 50-100 |
| min_connections | 2-5 | 10-20 |
| acquire_timeout | 30s | 10s |
| idle_timeout | 600s | 300s |
| max_lifetime | 1800s | 900s |

## Memory Management

Rust's ownership system prevents most memory issues, but you should still be mindful:

```rust
// Bad: allocating large buffers unnecessarily
let huge_buffer = vec![0u8; 1024 * 1024 * 100];  // 100MB!

// Good: streaming processing
let mut reader = BufReader::new(file);
let mut line = String::new();
while reader.read_line(&mut line)? > 0 {
    process_line(&line);
    line.clear();
}
```

## Async Performance

```rust
// Bad: sequential processing
for url in urls {
    let result = fetch_url(&client, &url).await?;
    results.push(result);
}

// Good: concurrent processing
let futures: Vec<_> = urls.iter()
    .map(|url| fetch_url(&client, url))
    .collect();
let results: Vec<_> = futures::future::join_all(futures).await;
```

## 📝 Practice Exercises

1. **Flamegraph:** Generate a flamegraph of FicHub handling 100 concurrent requests. Identify the bottleneck.

2. **Query Analysis:** Run EXPLAIN ANALYZE on 5 different queries and optimize the slowest one.

3. **Connection Pool:** Simulate 100 concurrent database queries and monitor pool usage.

4. **Memory Profiling:** Use `valgrind` or `heaptrack` to profile FicHub's memory usage during export generation.

