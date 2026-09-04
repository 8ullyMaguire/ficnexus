# Part 5 — Search Works

Now you build the search feature. Backend: parse query strings, build SQL, return results. Frontend: search box, results list, filter chips. This is the most complex single feature in FicHub — the search system has a full boolean query parser, tag resolution, faceted filtering, and sorting.

---

## 5.1 Backend: `GET /api/search` — `search_handler`

Open `src/search/routes.rs`. The handler is at the top:

```rust
// src/search/routes.rs (lines 1-20, excerpt)
use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::search::builder::{
    parse_tag_filters, FicSearchRow, SearchParams, SearchQueryBuilder, TagFilter,
};
use crate::search::parser::{
    extract_excluded_terms, extract_field_queries, extract_fielded_terms,
    extract_text_tsquery, parse_query,
};
use crate::search::tags::get_matching_tag_ids;
use crate::server::AppState;
```

### The route parameters

```rust
// src/search/routes.rs (lines 23-84)
#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    pub include_tags: Option<String>,
    pub exclude_tags: Option<String>,
    pub exclude_tag_types: Option<String>,
    pub strict_gen: Option<bool>,
    pub include_any_tags: Option<String>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub primary_tag: Option<String>,
    pub min_comments: Option<i64>,
    pub min_kudos: Option<i64>,
    pub max_kudos: Option<i64>,
    pub min_bookmarks: Option<i64>,
    pub max_bookmarks: Option<i64>,
    pub rating: Option<String>,
    pub language: Option<String>,
    pub main_char: Option<String>,
    pub status: Option<String>,
    pub beta_status: Option<String>,
    pub crossover: Option<bool>,
    pub min_hits: Option<i64>,
    pub max_hits: Option<i64>,
    pub no_warnings: Option<bool>,
    pub tag_ids: Option<String>,
    pub relationship_characters: Option<String>,
    pub hide_read: Option<bool>,
    pub hide_bookmarked: Option<bool>,
    pub library_only: Option<bool>,
}
```

This is the search API surface. Every parameter maps to a filter or sort option. The frontend can pass any combination.

### The handler function

```rust
// src/search/routes.rs (excerpt — find search_handler)
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let mut search_params = params.into_search_params()?;

    // Apply the boolean query parser to the q string
    crate::search::routes::apply_query_parse(&mut search_params);

    // Resolve tag names to IDs
    let tag_resolver = crate::search::tags::TagResolver::new(&state.db);
    tag_resolver.resolve(&mut search_params).await?;

    // Build and run the query
    let builder = SearchQueryBuilder::new(&state.db, &search_params);
    let results = builder.build().await?;

    Ok(Json(json!({
        "err": 0,
        "results": results,
        "total": builder.total_count,
        "page": search_params.page.unwrap_or(1),
        "per_page": search_params.per_page.unwrap_or(20),
    })))
}
```

### Breakdown

**Step 1: Convert raw params to typed params**:
```rust
let mut search_params = params.into_search_params()?;
```

This calls `SearchQueryParams::into_search_params()`, which parses dates, tag filters, and other complex fields. It returns a `SearchParams` struct.

**Step 2: Apply the query parser**:
```rust
crate::search::routes::apply_query_parse(&mut search_params);
```

If `q` is `"harry AND potter"`, this parses it into a PostgreSQL `tsquery` and fills in `search_params.parsed_tsquery`. It also extracts fielded terms like `fandom:harry` and `author:rowling`.

**Step 3: Resolve tag names to IDs**:
```rust
let tag_resolver = crate::search::tags::TagResolver::new(&state.db);
tag_resolver.resolve(&mut search_params).await?;
```

Tag names like `"Harry Potter"` are resolved to tag IDs. Synonyms are expanded — if "HP" is a synonym for "Harry Potter", both IDs are included.

**Step 4: Build and run the query**:
```rust
let builder = SearchQueryBuilder::new(&state.db, &search_params);
let results = builder.build().await?;
```

The builder constructs a SQL query with all the filters and executes it.

**Step 5: Return the response**:
```rust
Ok(Json(json!({
    "err": 0,
    "results": results,
    "total": builder.total_count,
    "page": ...,
    "per_page": ...,
})))
```

---

## 5.2 The query parser: `src/search/parser.rs`

Open `src/search/parser.rs`. This is the boolean query parser. It's 1631 lines because it handles a lot of syntax.

### What it parses

The parser supports:

- **Simple words**: `coffee` — full-text search for "coffee".
- **Quoted phrases**: `"enemies to lovers"` — exact phrase match.
- **AND**: `coffee AND angst` — both terms must match.
- **OR**: `coffee OR tea` — either term matches.
- **NOT**: `NOT major character death` or `-major character death` — exclude.
- **Fielded search**: `title:harry`, `author:jk`, `fandom:harry`.
- **Grouping**: `(fluff OR humor) AND -angst`.
- **Role modifier**: `@char:Harry` — require the character tag to be main/primary.
- **Fuzzy**: `title~harry` — fuzzy match on title.
- **Wildcards**: `attr:cozy*`, `attr:*burn`.
- **Ship polarity**: `romship:A/B` (romantic), `platship:A&B` (platonic).
- **Counters & ranges**: `words:>50k`, `words:10k-100k`, `kudos:>=100`, `chapters:5-50`.
- **Crossover**: `crossover:1` — fics with more than one fandom.

### The parsed representation

```rust
// src/search/parser.rs (lines 29-46)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum QueryTerm {
    Word(String),
    Phrase(String),
    Excluded(String),
    Fielded { field: String, value: String },
    FieldQuery(FieldQuery),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FieldQuery {
    pub field: Field,
    pub value: Option<String>,
    pub op: FieldOp,
    pub role: bool,       // @ modifier
    pub negated: bool,    // NOT/- prefix
}
```

The parser converts the query string into a list of `QueryTerm` enums. Each term represents one piece of the query.

### The `Field` enum

```rust
// src/search/parser.rs (lines 72-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Field {
    Title,
    Author,
    Description,
    Fandom,
    PrimaryFandom,
    Char,
    Ship,
    Romship,
    Platship,
    Attr,
    Warning,
    Category,
    Rating,
    Status,
    Words,
    // ... more fields ...
}
```

Each field knows how it maps to a database column or tag type. `Fandom` maps to tag type 1. `Char` maps to tag type 2. `Words` maps to the `fic_info.words` column.

### How parsing works (simplified)

The parser is a hand-rolled tokenizer + recursive descent parser. It does not use regex-lite backreferences (regex-lite doesn't support them). Instead, it scans the input character by character.

The high-level flow:

1. **Tokenize**: split the input into tokens (words, operators, punctuation).
2. **Parse**: build an expression tree from the tokens.
3. **Extract**: convert the tree into `QueryTerm` enums and a PostgreSQL `tsquery` string.

---

## 5.3 The query builder: `src/search/builder.rs`

Open `src/search/builder.rs`. This is where the SQL is constructed. It's 2025 lines.

### The `SearchParams` struct

```rust
// src/search/builder.rs (lines 22-119)
#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub q: Option<String>,
    pub parsed_tsquery: Option<String>,
    pub fuzzy: bool,
    pub fielded_terms: Vec<(String, String)>,
    pub field_queries: Vec<FieldQuery>,
    pub main_char: Option<String>,
    pub status: Option<String>,
    pub beta_status: Option<String>,
    pub crossover: Option<bool>,
    pub min_hits: Option<i64>,
    pub max_hits: Option<i64>,
    pub include_tags: Vec<TagFilter>,
    pub exclude_tags: Vec<TagFilter>,
    pub exclude_tag_types: Vec<i16>,
    pub strict_gen: bool,
    pub include_any_tags: Vec<TagFilter>,
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub primary_tag: Option<TagFilter>,
    pub min_comments: Option<i64>,
    pub min_kudos: Option<i64>,
    pub max_kudos: Option<i64>,
    pub min_bookmarks: Option<i64>,
    pub max_bookmarks: Option<i64>,
    pub rating: Option<String>,
    pub language: Option<String>,
    pub no_warnings: Option<bool>,
    pub tag_ids: Vec<i32>,
    pub relationship_characters: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub hide_read: bool,
    pub hide_bookmarked: bool,
    pub library_only: bool,
    pub user_id: Option<i32>,
}
```

This is the typed representation of all search parameters. The `SearchQueryParams::into_search_params()` method fills this from the HTTP query string.

### The `SearchResult` struct

```rust
// src/search/builder.rs (lines 122-137)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchResult {
    pub url_id: String,
    pub work_id: Option<i32>,
    pub title: String,
    pub author: String,
    pub source: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub description: String,
    pub updated: Option<DateTime<Utc>>,
    pub rank: Option<f32>,
    pub snippet: Option<String>,
}
```

This is what the frontend receives. Each result has the fic metadata, an optional relevance rank, and an optional ts_headline snippet.

### The builder: constructing SQL

The builder uses `sqlx::QueryBuilder` to construct a SQL query dynamically:

```rust
// src/search/builder.rs (excerpt — conceptual)
use sqlx::QueryBuilder;

pub async fn build(&self) -> Result<Vec<SearchResult>, AppError> {
    let mut query = QueryBuilder::new(
        "SELECT fi.id, fi.work_id, fi.title, fi.author, fi.source, "
        "fi.words, fi.chapters, fi.status, fi.description, fi.updated, "
        "COALESCE(rank.rank, 0) as rank, "
        "ts_headline('english', fi.description, q.query) as snippet "
        "FROM fic_info fi "
        "LEFT JOIN (SELECT url_id, ts_rank_cd(fts, q.query) as rank "
        "FROM fic_search, to_tsquery('english', $N) as q(query) "
        "WHERE fts @@ q) rank ON fi.id = rank.url_id "
        "WHERE 1=1"
    );

    // Add text search clause
    if let Some(ref tsquery) = self.params.parsed_tsquery {
        query.push(" AND fi.id IN (SELECT url_id FROM fic_search WHERE fts @@ to_tsquery('english', ");
        query.push_bind(tsquery);
        query.push("))");
    }

    // Add tag filters
    for tag in &self.params.include_tags {
        query.push(" AND fi.id IN (SELECT url_id FROM fic_tags WHERE tag_id = ANY(");
        query.push_bind(tag.tag_type_id);
        // ... more ...
    }

    // Add word count filter
    if let Some(min_words) = self.params.min_words {
        query.push(" AND fi.words >= ");
        query.push_bind(min_words);
    }

    // Add sorting
    query.push(" ORDER BY ");
    // ... sort logic ...

    // Add pagination
    query.push(" LIMIT ");
    query.push_bind(self.params.per_page.unwrap_or(20) as i64);
    query.push(" OFFSET ");
    query.push_bind((self.params.page.unwrap_or(1) - 1) * self.params.per_page.unwrap_or(20) as i64);

    // Execute
    let rows = query.build().fetch_all(&self.pool).await?;
    // ... map to SearchResult ...
}
```

### Key design decisions

- **`QueryBuilder`**: SQLx's query builder constructs SQL dynamically. It handles escaping and parameter binding.
- **`ts_headline`**: PostgreSQL function that returns a snippet with `<b>` tags around matching terms.
- **`ts_rank_cd`**: PostgreSQL function that ranks results by relevance.
- **` fic_search`**: a `tsvector` column on `fic_info` that stores the full-text search index.

---

## 5.4 Tag resolution: `src/search/tags/`

The tag resolver converts tag names to IDs. Open `src/search/tags/`:

```rust
// src/search/tags/mod.rs (conceptual)
pub struct TagResolver {
    pool: PgPool,
}

impl TagResolver {
    pub async fn resolve(&self, params: &mut SearchParams) -> Result<(), AppError> {
        // Resolve include_tags
        params.expanded_include_tag_ids = self.resolve_tags(&params.include_tags).await?;
        params.expanded_exclude_tag_ids = self.resolve_tags(&params.exclude_tags).await?;
        params.expanded_include_any_tag_ids = self.resolve_tags(&params.include_any_tags).await?;
    }

    async fn resolve_tags(&self, tags: &[TagFilter]) -> Result<Vec<Vec<i32>>, AppError> {
        // For each tag, look up canonical + synonym IDs
        // Returns a Vec<Vec<i32>> — one inner Vec per input tag
    }
}
```

Tag resolution handles synonyms. If "HP" is a synonym for "Harry Potter", searching for "HP" includes both the "HP" tag ID and the "Harry Potter" tag ID.

---

## 5.5 The suggest endpoint: `GET /api/search/suggest`

Open `src/search/suggest.rs`. This endpoint returns tag suggestions for the search filter UI.

```rust
// src/search/suggest.rs (excerpt)
pub async fn search_suggest_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<SuggestQueryParams>,
) -> Result<Json<Value>, AppError> {
    let personal = params.personal == Some(1);
    let use_cache = !personal;

    // Check cache for popular suggestions
    if use_cache {
        let cached = {
            let cache = state.suggest_cache.lock().await;
            cache
                .as_ref()
                .filter(|(at, _)| at.elapsed().as_secs() < 300)
                .map(|(_, items)| items.clone())
        };
        if let Some(items) = cached {
            return Ok(Json(json!({ "err": 0, "suggestions": items })));
        }
    }

    // Query popular tags
    let popular: Vec<(i32, String, i16, i64)> = sqlx::query_as(
        r#"SELECT t.id, t.name, t.tag_type_id, COUNT(ft.url_id) AS usage_count
           FROM tags t
           LEFT JOIN fic_tags ft ON ft.tag_id = t.id
           WHERE ($1::int IS NULL OR t.tag_type_id = $1)
             AND ($2::text IS NULL OR t.name ILIKE $2 || '%')
           GROUP BY t.id
           HAVING COUNT(ft.url_id) > 0
           ORDER BY usage_count DESC, t.name
           LIMIT 20"#,
    )
    .bind(params.tag_type_id)
    .bind(params.q.as_deref())
    .bind(20)
    .fetch_all(&state.db)
    .await?;

    // ... format and return ...
}
```

### Breakdown

- **Cache**: popular suggestions are cached for 300 seconds in `state.suggest_cache`.
- **Personalized**: if `personal=1` and the user is logged in, the suggestions are re-ranked based on the user's bookmarked tags.
- **Query**: every tag with at least one fic, ranked by usage count, limited to 20.
- **Prefix match**: `ILIKE $2 || '%'` matches tags whose name starts with the query string.

---

## 5.6 Frontend: `src/routes/search/+page.svelte`

Open `frontend/src/routes/search/+page.svelte`. This is the search page.

```svelte
<!-- frontend/src/routes/search/+page.svelte (lines 1-60, excerpt) -->
<script lang="ts">
  import { search, searchSuggest } from '$lib/api/search';
  import SimpleSearch from '$lib/components/search/SimpleSearch.svelte';
  import PowerSearch from '$lib/components/search/PowerSearch.svelte';
  import GuidedSearch from '$lib/components/search/GuidedSearch.svelte';
  import { getPref } from '$lib/prefs';

  let { data } = $props();

  let query = $state(data.query ?? '');
  let results = $state<SearchResult[]>([]);
  let loading = $state(false);
  let total = $state(0);
  let page = $state(1);
  let perPage = $state(20);
  let sort = $state(data.sort ?? 'updated');
  let activeMode = $state<'simple' | 'power' | 'guided'>('simple');
</script>
```

### The API client: `src/lib/api/search.ts`

```typescript
// frontend/src/lib/api/search.ts (excerpt)
export interface SearchResult {
  url_id: string;
  work_id: number | null;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  updated: string | null;
  rank: number | null;
  snippet: string | null;
}

export async function search(params: SearchParams): Promise<SearchResponse> {
  const qs = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') {
      qs.set(key, String(value));
    }
  }
  const data = await request<SearchResponse>(`/search?${qs.toString()}`);
  return data;
}

export async function searchSuggest(params: SuggestParams): Promise<SuggestResponse> {
  const qs = new URLSearchParams();
  if (params.q) qs.set('q', params.q);
  if (params.tag_type_id) qs.set('tag_type_id', String(params.tag_type_id));
  if (params.personal) qs.set('personal', '1');
  return request<SuggestResponse>(`/search/suggest?${qs.toString()}`);
}
```

---

## 5.7 Frontend: search components

There are three search modes, each a different component:

### SimpleSearch

A simple text box with a search button. Good for quick searches.

```svelte
<!-- frontend/src/lib/components/search/SimpleSearch.svelte (conceptual) -->
<script lang="ts">
  let { value = $bindable(''), onsearch } = $props();
</script>

<form onsubmit={(e) => { e.preventDefault(); onsearch?.(value); }}>
  <input type="text" bind:value placeholder="Search fanfiction…" />
  <button type="submit">Search</button>
</form>
```

### PowerSearch

A chip-based advanced search. Each chip is a filter (tag, word count range, sort, etc.).

```svelte
<!-- frontend/src/lib/components/search/PowerSearch.svelte (conceptual) -->
<script lang="ts">
  let { chips = $bindable([]) } = $props();

  function addChip(chip: Chip) {
    chips = [...chips, chip];
  }

  function removeChip(index: number) {
    chips = chips.filter((_, i) => i !== index);
  }
</script>

<div class="power-search">
  <!-- Tag filter chips -->
  {#each chips as chip, i}
    <span class="chip">
      {chip.label}
      <button onclick={() => removeChip(i)}>×</button>
    </span>
  {/each}
  <!-- Add chip buttons -->
</div>
```

### GuidedSearch

A form with dropdowns and inputs for each filter category.

```svelte
<!-- frontend/src/lib/components/search/GuidedSearch.svelte (conceptual) -->
<script lang="ts">
  let { form = $bindable({}) } = $props();
</script>

<form class="guided-search">
  <select bind:value={form.sort}>
    <option value="updated">Recently updated</option>
    <option value="kudos">Most kudos</option>
    <option value="comments">Most comments</option>
    <option value="words">Most words</option>
  </select>
  <select bind:value={form.include_tags}>
    <option value="">Any tag</option>
    <!-- populated from searchSuggest -->
  </select>
  <button type="submit">Search</button>
</form>
```

---

## 5.8 Try It Yourself: add a tag filter chip

Add a tag filter that toggles a URL parameter.

### Step 1: Add a chip state

In the search page component:

```typescript
let chips = $state<{ type: string; value: string; label: string }[]>([]);
```

### Step 2: Add a function to toggle a chip

```typescript
function toggleChip(type: string, value: string, label: string) {
  const existing = chips.find(c => c.type === type && c.value === value);
  if (existing) {
    chips = chips.filter(c => c !== existing);
  } else {
    chips = [...chips, { type, value, label }];
  }
}
```

### Step 3: Build the query string from chips

```typescript
function buildQueryString(): string {
  const qs = new URLSearchParams();
  qs.set('q', query);
  qs.set('sort', sort);

  for (const chip of chips) {
    if (chip.type === 'include_tag') {
      qs.set('include_tags', qs.has('include_tags') ? qs.get('include_tags')! + ',' + chip.value : chip.value);
    }
  }

  return qs.toString();
}
```

### Step 4: Render the chips

```svelte
<div class="chips">
  {#each chips as chip}
    <span class="chip">
      {chip.label}
      <button onclick={() => toggleChip(chip.type, chip.value, chip.label)}>×</button>
    </span>
  {/each}
</div>
```

### Step 5: Test

Start the frontend dev server and open the search page. Type a query, add a tag chip, and verify the URL updates and the results change.

---

## 5.9 What you have now

- You understand the search handler: parameters, query parsing, tag resolution, builder, response.
- You understand the query parser: tokens, `QueryTerm` enum, `FieldQuery`, `Field` enum.
- You understand the query builder: `QueryBuilder`, dynamic SQL, `ts_headline`, `ts_rank_cd`.
- You understand tag resolution: canonical + synonym IDs.
- You understand the suggest endpoint: cache, popular tags, prefix match.
- You understand the frontend search page: three modes, API client, state management.
- You understand the three search components: SimpleSearch, PowerSearch, GuidedSearch.
- You added a tag filter chip.

Next: Part 6 — Upload a Work. You will build the upload endpoint and the upload page.

---

*End of Part 5. On to [Part 6 — Upload a Work](./06-upload-work/06-upload-a-work.md).*
