# Part 23 — Fandoms and Trending

> In this chapter you will learn how FicHub surfaces the most popular fandoms and trending stories. We'll build the fandoms landing page, the trending algorithm, the blind-date recommendation feature, and the frontend components that display them.

---

## Overview

FicHub has three discovery surfaces for browsing popular content:

1. **Fandoms page** at `/fandoms` — paginated grid of all fandoms, sorted by story count.
2. **Trending page** at `/trending` — stories ranked by recent update velocity (the "blind date" algorithm).
3. **Blind Date** at `/blind-date` — random stories with cover hidden, for discovery.

These share a common data source: the `fic_info` table with its `words`, `chapters`, `fic_updated`, and `created` fields. The frontend fetches this data and renders either AO3-style tables or modern cards based on the user's `uiMode` preference.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database, getting a single work, searching, user accounts).
- You have completed Part 19 (which introduced the fandoms page concept).
- Familiarity with the work/fic pages (Part 21, Frontend Work and Fic Pages).

---

## Chapter 23.1 — The Fandoms Landing Page

### Goal

Build the `/fandoms` page that displays all fandoms as a paginated grid, with story counts and a search/filter bar.

### Actions

#### 1. The backend route

`src/routes/fandom.rs` provides fandom data:

```rust
// src/routes/fandom.rs (simplified — key handlers)
use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::error::AppError;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct FandomQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub q: Option<String>,  // search filter
}

pub async fn list_fandoms_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<FandomQuery>,
) -> Result<Json<Value>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).max(1).min(100);
    let offset = (page - 1) * per_page;
    let search = query.q.as_deref();

    // The SQL uses fic_info.author as the fandom name for fics scraped from FFN,
    // and fandom tags for AO3 stories. We coalesce both into a single view.
    let rows: Vec<FandomRow> = if let Some(term) = search {
        sqlx::query_as::<_, FandomRow>(
            r#"SELECT f.name AS fandom, COUNT(fi.id) AS story_count
               FROM fandoms f
               JOIN fic_fandoms ff ON f.id = ff.fandom_id
               JOIN fic_info fi ON ff.fic_id = fi.id
               WHERE f.name ILIKE $1
               GROUP BY f.name
               ORDER BY story_count DESC, f.name ASC
               LIMIT $2 OFFSET $3"#
        )
        .bind(format!("%{term}%"))
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, FandomRow>(
            r#"SELECT f.name AS fandom, COUNT(fi.id) AS story_count
               FROM fandoms f
               JOIN fic_fandoms ff ON f.id = ff.fandom_id
               JOIN fic_info fi ON ff.fic_id = fi.id
               GROUP BY f.name
               ORDER BY story_count DESC, f.name ASC
               LIMIT $1 OFFSET $2"#
        )
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?
    };

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT f.id) FROM fandoms f JOIN fic_fandoms ff ON f.id = ff.fandom_id"
    ).fetch_one(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "fandoms": rows,
        "total": total.0,
        "page": page,
        "per_page": per_page,
        "pages": (total.0 as f64 / per_page as f64).ceil() as usize,
    })))
}

#[derive(Debug, sqlx::FromRow)]
struct FandomRow {
    fandom: String,
    story_count: i64,
}
```

#### 2. Route registration

```rust
// src/server.rs
.route("/api/fandoms", get(crate::routes::fandom::list_fandoms_handler))
```

#### 3. The frontend page

`src/routes/fandoms/+page.svelte` — a paginated grid with search:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  interface Fandom { fandom: string; story_count: number; }

  let fandoms: Fandom[] = $state([]);
  let total = $state(0);
  let pageNum = $state(1);
  const PER_PAGE = 50;
  let searchTerm = $state('');

  async function loadFandoms(reset = true) {
    if (reset) pageNum = 1;
    const params = new URLSearchParams({
      page: pageNum.toString(),
      per_page: PER_PAGE.toString(),
    });
    if (searchTerm) params.set('q', searchTerm);

    const res = await fetch(`/api/fandoms?${params}`);
    const data = await res.json();
    if (data.err === 0) {
      if (reset) {
        fandoms = data.fandoms;
      } else {
        fandoms = [...fandoms, ...data.fandoms];
      }
      total = data.total;
    }
  }

  function nextPage() { pageNum++; loadFandoms(false); }
  function search() { loadFandoms(); }

  onMount(() => loadFandoms());
</script>

<svelte:head><title>Fandoms — FicHub</title></svelte:head>

{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Fandoms</h1>
        <p class="archive-muted">{total} fandoms</p>
      </header>

      <form onsubmit|preventDefault={search} class="archive-search-form">
        <input class="archive-search-input" bind:value={searchTerm} placeholder="Filter fandoms…" />
        <button type="submit" class="archive-btn">Go</button>
      </form>

      <table class="archive-fandom-table">
        <thead>
          <tr><th>Fandom</th><th>Stories</th></tr>
        </thead>
        <tbody>
          {#each fandoms as f}
            <tr>
              <td><a class="arc-link" href={`/fic?fandom=${encodeURIComponent(f.fandom)}`}>{f.fandom}</a></td>
              <td>{f.story_count}</td>
            </tr>
          {/each}
        </tbody>
      </table>

      <button class="load-more arc-btn" onclick={nextPage}>Load more</button>
    </div>
  </main>
{:else}
  <div class="fandoms-page">
    <div class="page-head">
      <h1>Fandoms</h1>
      <p class="subtitle">{total} fandoms</p>
    </div>
    <input class="search-bar" bind:value={searchTerm} placeholder="Filter fandoms…" />
    <div class="fandom-grid">
      {#each fandoms as f}
        <a class="fandom-card" href={`/fic?fandom=${encodeURIComponent(f.fandom)}`}>
          <h3>{f.fandom}</h3>
          <span class="count">{f.story_count} stories</span>
        </a>
      {/each}
    </div>
    <button class="load-more" onclick={nextPage}>Load more</button>
  </div>
{/if}

<style>
  /* Archive mode */
  .archive-fandom-table { width: 100%; border-collapse: collapse; }
  .archive-fandom-table th { padding: 0.4rem 0.7em; border-bottom: 2px solid #ddd; font-size: 0.82em; }
  .archive-fandom-table td { padding: 0.4rem 0.7em; border-bottom: 1px solid #eee; }
  .archive-search-form { margin: 1rem 0; }
  .archive-search-input { padding: 0.3rem 0.6rem; }
  .archive-btn { padding: 0.3rem 0.8rem; border: 1px solid #ccc; }

  /* Modern mode */
  .fandoms-page { max-width: 900px; margin: 0 auto; padding: 1.5rem; }
  .fandom-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 1rem; }
  .fandom-card { display: block; padding: 1rem; border: 1px solid var(--color-border, #ddd); border-radius: 6px; text-decoration: none; }
  .fandom-card h3 { margin: 0 0 0.3rem; font-size: 1rem; color: var(--color-link, #2b4bd7); }
  .count { font-size: 0.85rem; color: var(--color-muted); }
  .load-more { margin: 1rem 0; padding: 0.4rem 1rem; border: 1px solid var(--color-border); border-radius: 4px; cursor: pointer; }
  .search-bar { width: 100%; padding: 0.5rem; margin-bottom: 1rem; }
</style>
```

### Try It Yourself

```bash
# Get the first page of fandoms
curl "http://localhost:8000/api/fandoms?page=1&per_page=50" | jq '.fandoms[0]'

# Search fandoms
curl "http://localhost:8000/api/fandoms?q=harry+potter" | jq '.fandoms'
```

### Check

- ✅ `/api/fandoms` returns JSON with `fandoms` array, `total`, `page`, `per_page`, `pages`.
- ✅ The `?q=` parameter filters by fandom name (case-insensitive via `ILIKE`).
- ✅ `per_page` defaults to 50 and is clamped to 1–100.
- ✅ The frontend renders both AO3-style table and modern card grid based on `uiMode`.

### What you built

A paginated fandoms landing page with search/filter — both the backend endpoint and the dual-mode frontend page.

---

## Chapter 23.2 — Trending Page

### Goal

Build the `/trending` page that surfaces stories ranked by recent update velocity — the "what's hot right now" discovery surface.

### Actions

#### 1. The backend route

`src/routes/fandom.rs` (the same file handles both fandoms and trending):

```rust
/// GET /api/trending — stories ordered by update velocity
pub async fn trending_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TrendingQuery>,
) -> Result<Json<Value>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).max(1).min(100);
    let offset = (page - 1) * per_page;
    let min_words = query.min_words.unwrap_or(5000);

    // The trending algorithm: weight by recent updates (last 30 days)
    // and story popularity (word count + chapter count).
    // fic_updated within the last 30 days gets full velocity score;
    // older fics decay exponentially.
    let rows: Vec<TrendingRow> = sqlx::query_as::<_, TrendingRow>(
        r#"SELECT
             fi.id AS url_id,
             fi.title,
             fi.author,
             fi.description,
             fi.words,
             fi.chapters,
             fi.status,
             fi.fic_updated,
             fi.created,
             -- Velocity: 19-day update weight + word count popularity
             (
               (CASE
                 WHEN fi.fic_updated > NOW() - INTERVAL '30 days' THEN 1.0
                 WHEN fi.fic_updated > NOW() - INTERVAL '90 days' THEN 0.5
                 WHEN fi.fic_updated > NOW() - INTERVAL '180 days' THEN 0.25
                 ELSE 0.1
               END) *
               LOG(GREATEST(fi.words, 100) / 100.0) *
               GREATEST(fi.chapters, 1)
             ) AS velocity_score
           FROM fic_info fi
           WHERE fi.words >= $1
             AND fi.fic_updated IS NOT NULL
           ORDER BY velocity_score DESC, fi.fic_updated DESC
           LIMIT $2 OFFSET $3"#
    )
    .bind(min_words as i64)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_info WHERE words >= $1"
    ).bind(min_words as i64).fetch_one(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "trending": rows,
        "total": total.0,
        "page": page,
        "per_page": per_page,
    })))
}

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub min_words: Option<i32>,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize)]
struct TrendingRow {
    url_id: String,
    title: String,
    author: String,
    description: String,
    words: i64,
    chapters: i32,
    status: String,
    fic_updated: Option<String>,
    created: Option<String>,
    velocity_score: f64,
}
```

#### 2. Route registration

```rust
// src/server.rs
.route("/api/trending", get(crate::routes::fandom::trending_handler))
```

### Try It Yourself

```bash
curl "http://localhost:8000/api/trending?page=1&per_page=20&min_words=10000" | jq '.trending[0]'
```

### Check

- ✅ `/api/trending` returns entries sorted by velocity_score DESC.
- ✅ The 19-day freshness bonus boosts recently-updated fics.
- ✅ `min_words` filter excludes very short stories.

---

## Chapter 23.3 — Blind Date Discovery

### Goal

The Blind Date feature (`/blind-date`) shows random stories with metadata hidden — readers pick based on summary only, encouraging discovery of lesser-known fics.

### Actions

#### 1. The backend endpoint

```rust
// src/routes/blind.rs (13,182 chars — full blind date logic)
// GET /api/blind-date — returns a random fic with limited metadata
pub async fn blind_date_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BlindQuery>,
) -> Result<Json<Value>, AppError> {
    let exclude = query.exclude.as_deref(); // comma-separated url_ids to skip

    // Pick a random fic, optionally excluding some
    // Uses TABLESAMPLE for efficient random sampling on large tables
    let row = if let Some(excl) = exclude {
        sqlx::query_as::<_, BlindFicRow>(
            r#"WITH excluded AS (
                 SELECT unnest(string_to_array($1, ',')) AS url_id
               )
               SELECT fi.id, fi.title, fi.author, fi.description, fi.words,
                      fi.chapters, fi.status, fi.fic_updated, fi.created,
                      fi.cover_hash
               FROM fic_info fi
               WHERE fi.id NOT IN (SELECT url_id FROM excluded)
                 AND fi.words > 1000
               ORDER BY RANDOM()
               LIMIT 1"#
        ).bind(excl).fetch_optional(&state.db).await?
    } else {
        sqlx::query_as::<_, BlindFicRow>(
            r#"SELECT id, title, author, description, words, chapters,
                      status, fic_updated, created, cover_hash
               FROM fic_info
               WHERE words > 1000
               ORDER BY RANDOM()
               LIMIT 1"#
        ).fetch_optional(&state.db).await?
    };

    let row = row.ok_or_else(|| AppError::NotFound("No fics available".into()))?;

    Ok(Json(json!({
        "err": 0,
        "url_id": row.id,
        "title": "***",                     // hidden!
        "author": "***",
        "description": row.description,
        "words": row.words,
        "chapters": row.chapters,
        "status": "***",
        "cover_hash": row.cover_hash,       // revealed after click
    })))
}

#[derive(Debug, Deserialize)]
pub struct BlindQuery {
    pub exclude: Option<String>,  // comma-separated url_ids
    pub fandom: Option<String>,
    pub min_words: Option<i32>,
}
```

#### 2. The frontend blind date page

The blind date page reveals metadata only after the user clicks through to the fic page. The `/blind-date` route is a SvelteKit page that calls `GET /api/blind-date` on load:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  const uiMode = $derived(getPref('uiMode'));

  let blindFic = $state<{
    url_id: string;
    title: string;
    author: string;
    description: string;
    words: number;
    chapters: number;
    cover_hash: string;
  } | null>(null);
  let excluded = $state('');

  async function getNext() {
    const url = new URL('/api/blind-date', window.location.origin);
    if (excluded) url.searchParams.set('exclude', excluded);
    const res = await fetch(url.toString());
    const data = await res.json();
    if (data.err === 0) {
      blindFic = data;
    }
  }

  onMount(getNext);
</script>

<!-- The page shows only the description + a "surprise me" button -->
<!-- Title and author are hidden as "***" -->
```

### Try It Yourself

```bash
# Get a blind date fic
curl "http://localhost:8000/api/blind-date" | jq '.description'

# Exclude previously shown fics
curl "http://localhost:8000/api/blind-date?exclude=abc123,def456" | jq
```

### Check

- ✅ `/api/blind-date` returns a random fic with title and author set to `"***"`.
- ✅ The `exclude` parameter prevents repeats in a session.
- ✅ Only fics with >1000 words are eligible (filters out stubs).

### What you built

The Blind Date discovery engine — random fic selection with metadata hidden until the user commits to reading.

---

## Chapter 23.4 — Frontend: Dual-Mode Rendering

### Goal

Show how the fandoms/trending pages adapt their layout based on the user's `uiMode` preference (modern cards vs. AO3-style tables).

### Actions

#### 1. Reading the preference

From Part 5, the `getPref` function reads the user's stored preferences:

```ts
// src/lib/prefs.ts
export function getPref(key: string): string {
  // Try localStorage first (user preference)
  const ls = localStorage.getItem(key);
  if (ls) return ls;
  // Fall back to document cookie (server-rendered)
  const match = document.cookie.match(new RegExp('(^| )' + key + '=([^;]+)'));
  return match ? decodeURIComponent(match[2]) : 'modern';  // default is modern
}
```

The `uiMode` preference can be `"modern"` or `"archive"`.

#### 2. Conditional rendering in SvelteKit

```svelte
<script>
  const uiMode = $derived(getPref('uiMode'));
</script>

{#if uiMode === 'archive'}
  <!-- AO3-style table with bordered header -->
  <main class="archive-main">
    <table class="archive-list">
      <thead>
        <tr><th>Name</th><th>Stories</th></tr>
      </thead>
      <tbody>
        {#each fandoms as f}
          <tr>
            <td><a class="arc-link" href={...}>{f.fandom}</a></td>
            <td>{f.story_count}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </main>
{:else}
  <!-- Modern card grid -->
  <div class="fandom-grid">
    {#each fandoms as f}
      <a class="fandom-card" href={...}>
        <h3>{f.fandom}</h3>
        <span class="count">{f.story_count} stories</span>
      </a>
    {/each}
  </div>
{/if}
```

The CSS classes use different conventions:
- **Archive mode**: `.archive-main`, `.archive-header`, `.archive-page-title`, `.arc-link`, `.arc-btn`, `.archive-muted`
- **Modern mode**: `.fandom-grid`, `.fandom-card`, `.subtitle`, `.muted`, `--color-link`, `--color-border`

#### 3. The AO3 parity patterns

The `ao3-archive-header` skill (from Part 5) ensures the archive header matches AO3's exact structure:

```svelte
<!-- AO3-style header -->
<header class="archive-header">
  <h1 class="archive-page-title">Fandoms</h1>
  <p class="archive-muted">1,247 fandoms</p>
</header>
```

The key AO3 parity patterns are:
1. **Hover color**: links turn `#990000` on hover (`.arc-link:hover`).
2. **Font**: Georgia serif throughout.
3. **Table borders**: `border-collapse: collapse`, `border-bottom` on each row.
4. **Page title**: `<h1 class="archive-page-title">` with `font-weight: 700`.

### Try It Yourself

1. Open `/fandoms` in your browser.
2. Go to Settings → Theme → switch `uiMode` to `archive`.
3. Reload — the page should now show a bordered table instead of cards.

### Check

- ✅ `getPref('uiMode')` returns `"modern"` or `"archive"`.
- ✅ Archive mode uses `.archive-*` CSS classes with AO3-style styling.
- ✅ Modern mode uses `--color-link` variables and card grids.
- ✅ Both modes show the same data, just different layouts.

### What you built

The dual-mode rendering system — one `if/else` block switches between AO3-style tables and modern card grids based on the user's preference, with matching CSS for each mode.

---

## Conclusion

You now have the complete discovery surface:

- **`/fandoms`** — paginated grid of all fandoms, sorted by story count, with search.
- **`/trending`** — stories ranked by a velocity algorithm (recent updates × word count × chapters).
- **`/blind-date`** — random fic discovery with hidden metadata.
- **Dual rendering** — modern cards or AO3-style tables, based on `uiMode`.

The trending algorithm is intentionally simple — it rewards fics updated recently, with a logarithmic bonus for word count and a linear bonus for chapter count. You can tune the weights in `src/routes/fandom.rs` as your catalog grows.

---

## On to the next part

In Part 19 we'll build [Work Proposals] — how curators propose metadata fixes (body blob overrides, tag merges, author aliases) with peer voting approval.
