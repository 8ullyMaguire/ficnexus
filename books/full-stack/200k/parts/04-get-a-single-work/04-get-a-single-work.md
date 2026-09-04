# Part 4 — Get a Single Work

You have a database. Now you serve a work. This part builds the `GET /api/works/:id` endpoint and the frontend page that displays it. You see the full request/response cycle: browser → Axum route → SQL query → JSON → Svelte component.

---

## 4.1 Backend: `GET /api/works/:id` — `get_work_handler`

Open `src/routes/social.rs`. Search for `get_work_handler`:

```rust
// src/routes/social.rs (excerpt — find the get_work_handler function)
use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn get_work_handler(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    // Look up the work by ID
    let work = crate::works::get_work_by_id(&state.db, work_id).await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", work_id)))?;

    // Look up the fic_info for this work
    let fic = crate::db::queries::get_fic_info(&state.db, &work.default_source_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Fic info not found for work {}", work_id)))?;

    // Get rating stats
    let ratings = crate::routes::social::rating_aggregate_json(&state, work_id).await?;

    // Build the response
    Ok(Json(json!({
        "err": 0,
        "work": {
            "id": work.id,
            "title": work.canonical_title,
            "author": work.canonical_author,
            "description": work.description,
            "created_at": work.created_at.to_rfc3339(),
            "updated_at": work.updated_at.to_rfc3339(),
        },
        "fic": {
            "url_id": fic.id,
            "title": fic.title,
            "author": fic.author,
            "chapters": fic.chapters,
            "words": fic.words,
            "description": fic.description,
            "status": fic.status,
            "source": fic.source,
        },
        "ratings": ratings,
    })))
}
```

### Breakdown

**Extractors**:
- `State(state): State<Arc<AppState>>` — the shared state (database, config, etc.).
- `Path(work_id): Path<i32>` — the `:id` from the URL path, parsed as an `i32`.

**Look up the work**:
```rust
let work = crate::works::get_work_by_id(&state.db, work_id).await?
    .ok_or_else(|| AppError::NotFound(...))?;
```

This calls a function in `src/works/mod.rs` that queries the `works` table. If the work doesn't exist, it returns a 404 error.

**Look up the fic_info**:
```rust
let fic = crate::db::queries::get_fic_info(&state.db, &work.default_source_id).await?
    .ok_or_else(|| AppError::NotFound(...))?;
```

Every work has a `default_source_id` — the url_id of the primary fic. This query looks up the `fic_info` table for that url_id.

**Get rating stats**:
```rust
let ratings = crate::routes::social::rating_aggregate_json(&state, work_id).await?;
```

This calls the rating aggregation function (which you'll see in detail in Part 10). It returns the average rating, distribution, and count.

**Build the response**:
```rust
Ok(Json(json!({
    "err": 0,
    "work": { ... },
    "fic": { ... },
    "ratings": ratings,
})))
```

The response is a JSON object with `err: 0` (success), the work metadata, the fic metadata, and the ratings.

### The route registration

In `src/server.rs`, find the line:

```rust
.route("/api/works/{id}", get(crate::routes::social::get_work_handler))
```

This registers `GET /api/works/:id` → `get_work_handler`. The `{id}` is a path parameter that Axum extracts as an `i32` and passes to the handler.

---

## 4.2 SQL: the work lookup query

Open `src/works/mod.rs`. This is where the work lookup functions live.

```rust
// src/works/mod.rs (excerpt)
use sqlx::PgPool;
use crate::db::models::WorkRow;
use crate::error::AppResult;

pub async fn get_work_by_id(pool: &PgPool, id: i32) -> AppResult<Option<WorkRow>> {
    sqlx::query_as::<_, WorkRow>(
        "SELECT * FROM works WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
```

This is a simple query: select the work row by ID. The result is mapped to a `WorkRow` struct via `FromRow`.

### The `WorkRow` struct

From `src/db/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkRow {
    pub id: i32,
    pub canonical_title: String,
    pub canonical_author: String,
    pub description: String,
    pub default_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader_id: Option<i32>,
    pub is_visible: Option<bool>,
}
```

This is the canonical work entry. When multiple fic_info rows from different sources represent the same story, they're merged into one `works` row. The `default_source_id` points to the primary fic.

---

## 4.3 Frontend: SvelteKit page shell

Open `frontend/src/routes/work/[workId]/+page.svelte`. This is the work page.

SvelteKit uses file-based routing. The folder `src/routes/work/[workId]/` means:
- `/work/123` → this page, with `workId = "123"`.
- The `[workId]` part is a route parameter, like `:id` in Axum.

The `+page.svelte` file is the component that renders for this route.

### The script section

```svelte
<!-- frontend/src/routes/work/[workId]/+page.svelte (lines 1-60) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getWork, addBookmark, removeBookmark, listBookmarks, rateWork, getRatings, giveKudos, removeKudos, getKudos, follow, unfollow, checkWorkFollow } from '$lib/api/social';
  import type { Work, WorkSource } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import ReactionBar from '$lib/components/ReactionBar.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import CommentSection from '$lib/components/CommentSection.svelte';
  import StarRating from '$lib/components/StarRating.svelte';
  import ReviewsSection from '$lib/components/ReviewsSection.svelte';
  import { formatWords, detectSite, stripHtml } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  const workId = $derived(Number(data.workId));

  let work = $state<Work | null>(null);
  let loading = $state(true);
  let error = $state('');

  let selectedSource = $state<WorkSource | null>(null);

  // Bookmark state
  let isBookmarked = $state(false);
  let bookmarkNotes = $state('');
  let bookmarkPrivate = $state(false);
  let savingBookmark = $state(false);

  // Rating state
  let avgRating = $state(0);
  let ratingCount = $state(0);
  let reviewCount = $state(0);
  let myRating = $state(0);
  let ratingLoading = $state(false);

  // Kudos state
  let kudosCount = $state(0);
  let guestCount = $state(0);
  let myKudos = $state(false);
  let kudosLoading = $state(false);

  // Work-level follow state
  let isFollowingWork = $state(false);
  let workFollowId: number | null = $state(null);
  let followLoading = $state(false);

  // Download formats
  let downloads = $derived.by(() => {
    if (!selectedSource) return [];
    return [
      { label: 'EPUB', type: 'epub' },
      { label: 'HTML', type: 'html' },
      { label: 'TXT', type: 'txt' },
      { label: 'MD', type: 'md' },
      { label: 'MOBI', type: 'mobi' },
      { label: 'PDF', type: 'pdf' },
      { label: 'AZW3', type: 'azw3' },
    ];
  });
</script>
```

### Breakdown

**Imports**:
- `getWork` — API function to fetch a work by ID.
- `Work` — TypeScript type for the work response.
- `auth` — Svelte store for auth state.
- `ReactionBar`, `StarRating`, `ReviewsSection`, `CommentSection` — UI components.
- `formatWords`, `detectSite`, `stripHtml` — utility functions.

**Props**:
- `let { data } = $props()` — receives the data from the load function (`+page.ts`).
- `const workId = $derived(Number(data.workId))` — converts the workId string to a number.

**State**:
- `work` — the work data, initially null.
- `loading` — whether data is being fetched.
- `error` — error message.

Svelte 5 uses `$state` for reactive state and `$derived` for derived values. When `work` changes, anything that depends on it re-renders.

### The load function: `+page.ts`

SvelteKit pages can have a `+page.ts` file that runs on the server (or client) to load data before rendering. Let's look at it:

```typescript
// frontend/src/routes/work/[workId]/+page.ts
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ params }) => {
  const workId = params.workId;
  return { workId };
};
```

This is minimal — it just passes the `workId` to the page. The actual API call happens in the component's `onMount`.

### The onMount: fetching data

In the component, add an `onMount`:

```svelte
<script lang="ts">
  // ... imports and state ...

  onMount(async () => {
    try {
      loading = true;
      const workRes = await getWork(workId);
      if (workRes.err !== 0) throw new Error(workRes.msg || 'Failed to load work');
      work = workRes.work;

      const ratingsRes = await getRatings(workId);
      if (ratingsRes.err === 0) {
        avgRating = ratingsRes.avg_rating;
        ratingCount = ratingsRes.rating_count;
        reviewCount = ratingsRes.review_count;
      }

      const kudosRes = await getKudos(workId);
      if (kudosRes.err === 0) {
        kudosCount = kudosRes.kudos_count;
        guestCount = kudosRes.guest_count;
        myKudos = kudosRes.my_kudos;
      }

      // Check bookmark status
      const bookmarksRes = await listBookmarks();
      if (bookmarksRes.err === 0) {
        isBookmarked = bookmarksRes.bookmarks.some(b => b.work_id === workId);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'An error occurred';
    } finally {
      loading = false;
    }
  });
</script>
```

### Breakdown

**`onMount`** — runs when the component mounts in the browser. It's client-side only (not server-side).

**Fetch work**: calls `getWork(workId)`, which makes a `GET /api/works/${workId}` request.

**Fetch ratings**: calls `getRatings(workId)`, which makes a `GET /api/ratings/${workId}` request.

**Fetch kudos**: calls `getKudos(workId)`, which makes a `GET /api/kudos/${workId}` request.

**Check bookmark**: calls `listBookmarks()`, which makes a `GET /api/bookmarks` request, and checks if this work is in the list.

**Error handling**: if any call fails, sets `error` and the UI shows an error message.

**Finally**: sets `loading = false` so the UI shows the data (or error) instead of a spinner.

---

## 4.4 Frontend: rendering the work

Now the template section of the component:

```svelte
<!-- frontend/src/routes/work/[workId]/+page.svelte (template section) -->
<svelte:head>
  <title>{work?.canonical_title ?? 'Loading…'}</title>
</svelte:head>

<div class="work-page">
  {#if loading}
    <div class="loading">Loading…</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if work}
    <header class="work-header">
      <h1>{work.canonical_title}</h1>
      <p class="author">by {work.canonical_author}</p>
    </header>

    <section class="work-meta">
      <p>{formatWords(work.word_count)} words</p>
      <p>{work.chapter_count} chapters</p>
      <p>{work.status}</p>
    </section>

    <section class="work-description">
      {@html stripHtml(work.description)}
    </section>

    <section class="work-ratings">
      <StarRating value={myRating} readonly={true} />
      <p>{avgRating} / 5 ({ratingCount} ratings)</p>
    </section>

    <section class="work-actions">
      <button onclick={() => toggleBookmark()}>
        {isBookmarked ? 'Remove bookmark' : 'Bookmark'}
      </button>
      <ReactionBar workId={workId} />
    </section>

    <ReviewsSection workId={workId} />
    <CommentSection workId={workId} />
  {/if}
</div>
```

### Breakdown

**`<svelte:head>`** — sets the page title.

**Conditional rendering**:
- `{#if loading}` — show a loading message.
- `{:else if error}` — show the error.
- `{:else if work}` — show the work data.

**Work header**: title and author.

**Work meta**: word count (formatted with `formatWords`), chapter count, status.

**Work description**: the HTML description, stripped of dangerous tags with `stripHtml` (which uses DOMPurify under the hood).

**Work ratings**: a readonly star rating widget showing the user's rating (if any), plus the average rating and count.

**Work actions**: bookmark toggle button and a reaction bar.

**Reviews and comments**: sub-components that load and display reviews and comments.

---

## 4.5 API client: `src/lib/api/social.ts`

Open `frontend/src/lib/api/social.ts`. This is the frontend API client. Every function here makes an HTTP request to the backend.

```typescript
// frontend/src/lib/api/social.ts (excerpt)
import type { Work, WorkSource } from './social-types';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
  return res.json();
}

export async function getWork(workId: number): Promise<{ err: number; work?: Work; msg?: string }> {
  const data = await request<{ err: number; work?: Work; msg?: string }>(`/works/${workId}`);
  return data;
}
```

### Breakdown

**`BASE`** — the API base URL. In production, this is `/api`. In development with a separate frontend server, it might be `http://localhost:8000/api`.

**`request<T>`** — a generic function that makes a fetch request and parses the JSON response as type `T`.

**`getWork`** — calls `GET /api/works/${workId}` and returns the response.

Other functions in the file:
- `addBookmark(workId, notes, isPrivate)` — `POST /api/bookmarks`
- `removeBookmark(workId)` — `DELETE /api/bookmarks/${workId}`
- `listBookmarks()` — `GET /api/bookmarks`
- `rateWork(workId, rating)` — `POST /api/ratings`
- `getRatings(workId)` — `GET /api/ratings/${workId}`
- `giveKudos(workId)` — `POST /api/kudos/${workId}`
- `removeKudos(workId)` — `DELETE /api/kudos/${workId}`
- `getKudos(workId)` — `GET /api/kudos/${workId}`

---

## 4.6 Try It Yourself: add the work's synopsis to the page

The work page already shows the description. Now add the synopsis (a shorter summary) below the title.

### Step 1: Add synopsis state

In the component's script section, add:

```typescript
let synopsis = $state('');
```

### Step 2: Fetch the synopsis

In the `onMount`, after fetching the work:

```typescript
synopsis = work?.description ?? '';
```

### Step 3: Render the synopsis

In the template, below the title:

```svelte
<p class="synopsis">{synopsis}</p>
```

### Step 4: Test

Start the frontend dev server:

```bash
cd frontend
npm run dev
```

Open `http://localhost:5173/work/1` (or any work ID that exists in your database).

---

## 4.7 The work page in full

The real work page is more complex than the simplified version above. It has:
- A format selector for downloads (EPUB, HTML, TXT, MOBI, PDF, AZW3).
- A reading status widget.
- Follow/unfollow buttons.
- A kudos button with animation.
- A star rating widget (click to rate).
- A review form and review list.
- A comment thread.
- Download links from the `/api/epub` endpoint.

But the core pattern is the same: load data in `onMount`, render with conditionals, use components for sub-features.

---

## 4.8 What you have now

- You understand `get_work_handler`: extractors, work lookup, fic lookup, rating aggregation.
- You understand the SQL query: `SELECT * FROM works WHERE id = $1`.
- You understand the `WorkRow` struct and `FromRow`.
- You understand the SvelteKit page structure: `+page.svelte`, `+page.ts`, `onMount`.
- You understand the API client: `request<T>`, `getWork`, `getRatings`, `getKudos`.
- You understand the template: conditional rendering, components, data binding.
- You added a synopsis to the work page.

Next: Part 5 — Search Works. You will build the search endpoint and the search page.

---

*End of Part 4. On to [Part 5 — Search Works](./05-search-works/05-search-works.md).*
