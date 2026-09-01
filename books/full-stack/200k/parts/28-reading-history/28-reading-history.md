# Part 28 — Reading History

Reading history is the AO3-style per-visit log: every time you open a work, the server records it. You can browse your history, delete single entries, or clear the whole thing. This part builds all three backend endpoints and explains the frontend page that already consumes them.

---

## 28.1 Backend: `GET /api/reading/history` and `POST /api/reading/history` — per-visit history

Open `src/routes/quests.rs`. Near the top of the file (after the reading-status code), you'll find the reading-history types and handlers. Scroll to around lines 138-206.

```rust
// src/routes/quests.rs (lines 138-185, excerpt)

/// Query params for GET /api/reading/history
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Body for POST /api/reading/history (record a visit)
#[derive(Debug, Deserialize)]
pub struct HistoryRecordBody {
    pub work_id: i32,
    pub chapter_num: Option<i32>,
}

/// GET /api/reading/history — get the user's reading history (AO3-style per-visit log)
pub async fn get_reading_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);
    let (items, total) = queries::get_reading_history(&state.db, user_id, limit, offset).await?;
    let entries: Vec<Value> = items.into_iter().map(|h| {
        json!({
            "id": h.id,
            "work_id": h.work_id,
            "url_id": h.url_id,
            "title": h.title,
            "author": h.author,
            "chapter_num": h.chapter_num,
            "visited_at": h.visited_at.to_rfc3339(),
        })
    }).collect();
    Ok(Json(json!({ "err": 0, "history": entries, "total": total, "limit": limit, "offset": offset })))
}

/// POST /api/reading/history — record a visit to a work
pub async fn record_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<HistoryRecordBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    queries::record_read_history(&state.db, user_id, body.work_id, body.chapter_num).await?;
    Ok(Json(json!({ "err": 0, "msg": "Visit recorded" })))
}
```

### Breakdown

**Two endpoints, one path**: `/api/reading/history` handles both `GET` (list) and `POST` (record). Axum decides which handler to call based on the HTTP method — that's why you see two `.route(...)` calls in `server.rs`:

```rust
// src/server.rs (lines 696-700, excerpt)
.route("/api/reading/history", get(crate::routes::quests::get_reading_history_handler))
.route("/api/reading/history", axum::routing::post(crate::routes::quests::record_read_history_handler))
.route("/api/reading/history/{id}", delete(crate::routes::quests::delete_read_history_handler))
.route("/api/reading/history/clear", axum::routing::post(crate::routes::quests::clear_read_history_handler))
```

**HistoryQuery**: The GET handler accepts `limit` and `offset`. `limit` defaults to 50 and is clamped to [1, 100] — no one can grab your whole history in one request. `offset` defaults to 0 for pagination.

**HistoryRecordBody**: The POST body has `work_id` (required) and `chapter_num` (optional). When you open chapter 5 of a fic, the frontend sends `{ work_id: 123, chapter_num: 5 }`. When you open the fic's main page without a chapter, it sends `{ work_id: 123 }` (chapter_num is null).

**The GET response**: Each entry carries `id` (the database row ID), `work_id`, `url_id` (the source-specific identifier, e.g. an AO3 slug or FanFiction.net story ID), `title`, `author`, `chapter_num` (nullable), and `visited_at` — an RFC 3339 timestamp. The response also includes `total` (total count for pagination) and echoes back `limit` and `offset`.

**The POST response**: Just `{"err": 0, "msg": "Visit recorded"}`. It's fire-and-forget — the frontend calls it after the reader loads, and it doesn't wait for a response (see `.catch(() => { /* best-effort */ })` in the reader page).

The database query lives in `src/db/queries.rs` around lines 1288-1309. Here's the important SQL:

```rust
pub async fn get_reading_history(
    pool: &PgPool,
    user_id: i32,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<ReadingHistoryEntry>, i64)> {
    let rows = sqlx::query_as::<_, ReadingHistoryEntry>(
        r#"
        SELECT rh.id, rh.work_id, COALESCE(w.default_source_id,'') AS url_id,
               COALESCE(w.canonical_title, fi.title) AS title,
               COALESCE(w.canonical_author, fi.author) AS author,
               rh.visited_at, rh.chapter_num
        FROM reading_history rh
        JOIN works w ON w.id = rh.work_id
        LEFT JOIN fic_info fi ON fi.id = w.default_source_id
        WHERE rh.user_id = $1
        ORDER BY rh.visited_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar("SELECT COUNT(*) FROM reading_history WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}
```

**Why the JOINs**: The `reading_history` table only stores `user_id`, `work_id`, `chapter_num`, and `visited_at`. It doesn't store the title or author — that would be redundant. The query joins `works` (to get `default_source_id` for `url_id` and `canonical_title`/`canonical_author`) and left-joins `fic_info` (the source-specific metadata table) so that even if the canonical title is missing, the actual source title shows up.

**`COALESCE`**: If `w.canonical_title` is null, fall back to `fi.title`. If `w.canonical_author` is null, fall back to `fi.author`. The `url_id` comes from `w.default_source_id` — the primary source's identifier.

### The `reading_history` table

```sql
-- migrations/001_initial.sql (lines 2254-2260)
CREATE TABLE public.reading_history (
    id bigint NOT NULL,
    user_id integer NOT NULL,
    work_id integer NOT NULL,
    visited_at timestamp with time zone DEFAULT now() NOT NULL,
    chapter_num integer DEFAULT 1
);
```

Every visit gets a row. `visited_at` defaults to `NOW()` — the server stamps the visit time automatically. `chapter_num` defaults to 1 (the start), but when the reader loads a specific chapter the frontend sends the actual number.

### Recording a visit from the reader page

The reader page (`frontend/src/routes/read/[urlId]/+page.svelte`, around lines 494-498) calls `recordReadHistory` when the reader loads:

```typescript
// frontend/src/routes/read/[urlId]/+page.svelte (lines 494-498, excerpt)
// Record a reading history visit (AO3-style per-visit log)
if (auth.isLoggedIn && work?.work_id) {
  void recordReadHistory(work.work_id, saved.chapterIndex != null ? saved.chapterIndex + 1 : undefined)
    .catch(() => { /* best-effort */ });
}
```

This is best-effort: if the backend is down or the user is offline, the visit simply isn't recorded. The reader still works. The call is `void` — the page doesn't wait for it.

---

## 28.2 Backend: `DELETE /api/reading/history/:id` — delete entry

Open the same `quests.rs` file, around lines 187-196:

```rust
// src/routes/quests.rs (lines 187-196)
/// DELETE /api/reading/history/{id} — delete a single history entry
pub async fn delete_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let removed = queries::delete_read_history(&state.db, user_id, id).await?;
    Ok(Json(json!({ "err": 0, "removed": removed })))
}
```

### Breakdown

**Path parameter**: The entry's database `id` is passed as a path parameter (`{id}` in the route). The handler owns the `Path<i64>` extractor.

**Ownership check**: The query deletes only if both `id` AND `user_id` match:

```rust
// src/db/queries.rs (lines 1312-1319)
pub async fn delete_read_history(pool: &PgPool, user_id: i32, id: i64) -> AppResult<bool> {
    let r = sqlx::query("DELETE FROM reading_history WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
```

This prevents user A from deleting user B's history entries by guessing IDs. If the row doesn't belong to the requesting user, `rows_affected()` is 0 and the handler returns `{"removed": false}`.

**Response**: `{"err": 0, "removed": true}` when the entry existed and was deleted. `{"err": 0, "removed": false}` when it wasn't found or didn't belong to the user.

### Frontend call

The history page (`frontend/src/routes/history/+page.svelte`, around lines 50-61) calls `deleteReadHistory` from the API client:

```typescript
// frontend/src/routes/history/+page.svelte (lines 50-61)
async function removeEntry(id: number) {
  removingId = id;
  try {
    await deleteReadHistory(id);
    history = history.filter((h) => h.id !== id);
    total -= 1;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    removingId = null;
  }
}
```

The page optimistically removes the entry from its local `history` array and decrements `total`. If the API call fails, the `catch` block sets the error message but doesn't re-add the entry — the user would have to refresh to see it again.

The API function in `social.ts`:

```typescript
// frontend/src/lib/api/social.ts (lines 862-864)
export async function deleteReadHistory(id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/reading/history/${id}`, { method: 'DELETE' });
}
```

---

## 28.3 Backend: `POST /api/reading/history/clear` — clear all

Open `quests.rs` again, around lines 198-206:

```rust
// src/routes/quests.rs (lines 198-206)
/// POST /api/reading/history/clear — clear all history entries for the user
pub async fn clear_read_history_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    queries::clear_read_history(&state.db, user_id).await?;
    Ok(Json(json!({ "err": 0, "msg": "History cleared" })))
}
```

### Breakdown

**User-scoped delete**: The query deletes all rows for the current user — no other user's history is touched:

```rust
// src/db/queries.rs (lines 1321-1328)
pub async fn clear_read_history(pool: &PgPool, user_id: i32) -> AppResult<()> {
    sqlx::query("DELETE FROM reading_history WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

**Response**: `{"err": 0, "msg": "History cleared"}`. Simple, because there's nothing to return — the table is now empty for that user.

### Frontend call with confirmation

The history page wraps the clear call in a browser `confirm()` dialog:

```typescript
// frontend/src/routes/history/+page.svelte (lines 63-75)
async function clearAll() {
  if (!confirm(t('history.clearConfirm'))) return;
  clearing = true;
  try {
    await clearReadHistory();
    history = [];
    total = 0;
  } catch (e) {
    error = e instanceof Error ? e.message : String(e);
  } finally {
    clearing = false;
  }
}
```

The `t('history.clearConfirm')` string comes from the i18n dictionary:

```typescript
// frontend/src/lib/i18n/dictionaries/en.ts (line 818)
'history.clearConfirm': 'Are you sure you want to clear your entire reading history? This cannot be undone.',
```

After the API call succeeds, the page resets its local state (`history = []`, `total = 0`) and the empty-state message appears.

---

## How the frontend page works

The history page is at `frontend/src/routes/history/+page.svelte`. It uses Svelte 5 runes (`$state`, `$derived`) and the `auth` store. Here's the full script:

```typescript
// frontend/src/routes/history/+page.svelte (lines 1-84)
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import { getReadingHistory, deleteReadHistory, clearReadHistory } from '$lib/api/social';
  import type { ReadingHistoryEntry } from '$lib/api/social-types';

  const uiMode = $derived(getPref('uiMode'));

  let history = $state<ReadingHistoryEntry[]>([]);
  let total = $state(0);
  let loading = $state(true);
  let error = $state('');
  let removingId = $state<number | null>(null);
  let clearing = $state(false);

  // Pagination
  let limit = 50;
  let offset = $state(0);
  let currentPage = $state(1);

  async function load() {
    // Ensure auth is initialized first
    if (!auth.initialized) {
      await auth.init();
    }
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    loading = true;
    error = '';
    try {
      const res = await getReadingHistory(limit, offset);
      if (res.err === 0) {
        history = res.history;
        total = res.total;
      } else {
        error = res.err.toString();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function removeEntry(id: number) {
    removingId = id;
    try {
      await deleteReadHistory(id);
      history = history.filter((h) => h.id !== id);
      total -= 1;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      removingId = null;
    }
  }

  async function clearAll() {
    if (!confirm(t('history.clearConfirm'))) return;
    clearing = true;
    try {
      await clearReadHistory();
      history = [];
      total = 0;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      clearing = false;
    }
  }

  function goToPage(page: number) {
    currentPage = page;
    offset = (page - 1) * limit;
    load();
  }

  onMount(load);
</script>
```

### Key design decisions

**Auth gate**: If the user isn't logged in, the page shows `history.loginPrompt` ("Log in to view your reading history.") instead of the list. The backend also returns 401 if `auth.user_id` is missing, but the frontend checks first to avoid the fetch.

**Two UI modes**: The page renders differently depending on `uiMode` — `'archive'` (the AO3-style archive layout) vs the modern mode. The archive mode uses the `ArchiveButton` component, Georgia serif fonts, and a toolbar with the clear button. The modern mode is simpler.

**Pagination**: The page fetches `limit` entries per page (default 50) and renders page-number buttons when `total > limit`. Each button calls `goToPage(page)`, which recomputes `offset = (page - 1) * limit` and reloads.

**Optimistic updates**: For delete, the entry is removed from the local array before the API response arrives. For clear, the whole array is reset after the API call (not before, since it's a destructive action that needs confirmation).

---

## Try It Yourself

### Exercise 1: Fetch your own history

Start the server (or use the dev instance on localhost:8000). Log in, then:

```bash
curl -s "http://localhost:8000/api/reading/history?limit=5" \
  -H "Authorization: Bearer YOUR_TOKEN" | python3 -m json.tool
```

**Expected**: A JSON response with `err: 0`, `history: [...]`, `total`, `limit: 5`, and `offset: 0`. If you haven't visited any works yet, `history` will be an empty array and `total` will be 0.

### Exercise 2: Record a visit manually

Visit a work in the reader first (so you have a `work_id` to use). Then:

```bash
curl -s -X POST "http://localhost:8000/api/reading/history" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"work_id": 1, "chapter_num": 3}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "Visit recorded"}`. Then fetch the history again to see the new entry at the top.

### Exercise 3: Delete the entry you just created

```bash
curl -s -X DELETE "http://localhost:8000/api/reading/history/<ID>" \
  -H "Authorization: Bearer YOUR_TOKEN" | python3 -m json.tool
```

Replace `<ID>` with the `id` from the entry you recorded. **Expected**: `{"err": 0, "removed": true}`. If you use a wrong ID or someone else's ID, you'll get `{"err": 0, "removed": false}`.

### Exercise 4: Clear all history

```bash
curl -s -X POST "http://localhost:8000/api/reading/history/clear" \
  -H "Authorization: Bearer YOUR_TOKEN" | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "History cleared"}`. Then fetch the history again — `total` is now 0.

### Exercise 5: Read the history page test

Open `frontend/src/routes/history/page.test.ts`. The test file mocks `fetch` and renders the Svelte component. Trace through the `"removes a history entry when remove button clicked"` test to see how the frontend calls `DELETE /api/reading/history/1` and checks that the button was clicked.

---

## Troubleshooting

**"Login required" on every call**: Make sure you're passing the `Authorization: Bearer <token>` header. The `AuthUser` extractor returns `Unauthorized` if `user_id` is missing.

**History returns empty but you've been reading**: The reader page calls `recordReadHistory` in a `void` expression with a `.catch(() => {})`. If the backend was unreachable when you opened the fic, the visit wasn't recorded. There's no retry — visit the fic again to create a new entry.

**`chapter_num` is null**: The frontend only sends `chapter_num` when it has a saved chapter position. If you open a fic's main page (not a specific chapter), `chapter_num` is omitted and stored as `NULL` in the database (the column defaults to 1, but the INSERT uses the provided value or NULL).

**Pagination doesn't show buttons**: The pagination buttons render only when `total > limit`. If you have 50 or fewer entries, there's no pagination. Change `limit` to a smaller value (e.g. 5) to see the pagination UI.

**Clear doesn't work**: The clear endpoint is `POST`, not `DELETE`. You must use `-X POST` (or `--request POST`). A `DELETE` to `/api/reading/history/clear` returns 404.

**Forever scroll vs pagination**: The current history page uses page-numbered pagination, not infinite scroll. If you want to add infinite scroll later, replace the `goToPage` function with a "load more" button that increments `offset += limit` and appends to `history` instead of replacing it.

---

## What you have now

- You understand the reading-history table: `id`, `user_id`, `work_id`, `visited_at`, `chapter_num`.
- You understand `GET /api/reading/history`: pagination with `limit`/`offset`, JOINs to get title/author/url_id, RFC 3339 timestamps.
- You understand `POST /api/reading/history`: recording a visit with optional `chapter_num`, best-effort from the reader page.
- You understand `DELETE /api/reading/history/:id`: ownership check via `user_id`, `removed` boolean response.
- You understand `POST /api/reading/history/clear`: user-scoped mass delete, frontend confirmation dialog.
- You understand the frontend history page: two UI modes, pagination, optimistic delete, auth gate, empty states.
- You know how the reader page records visits automatically on load.
- You tested all three endpoints with curl.

Next: Part 29 — Translations. You will build the locale system, UI translations, and work/chapter translation endpoints.

---

*End of Part 28. On to [Part 29 — Translations](./29-translations/29-translations.md).*
