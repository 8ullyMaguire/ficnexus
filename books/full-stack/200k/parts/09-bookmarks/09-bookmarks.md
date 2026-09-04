# Part 9 — Bookmarks

> In this chapter you will learn how FicHub's bookmark system works — saving works to read later, CSV import/export, private notes, and the dual UI mode (AO3-style archive + modern cards). We'll build the bookmark API handlers, the bookmarks page, and the background import queue.

---

## Overview

FicHub's bookmark system lives in two places:

- **Backend**: `src/routes/social.rs` — `add_bookmark_handler`, `list_bookmarks_handler`, `remove_bookmark_handler`, `export_bookmarks_csv`, `import_bookmarks_csv`
- **Frontend**: `frontend/src/routes/bookmarks/+page.svelte` — dual UI mode with parallel metadata fetching

Bookmarks are per-user, store a `work_id` + optional `notes` + `is_private` flag, and support CSV import (background queue) and CSV export (immediate download).

---

## Chapter 9.1 — The Bookmark Model and API

### Goal

Understand the Bookmark struct and the bookmark API contract.

### Actions

#### 1. The Bookmark model

```sql
-- From migrations/001_initial.sql (bookmarks table)
CREATE TABLE bookmarks (
    id          SERIAL PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    work_id     INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    notes       TEXT DEFAULT '',
    is_private  BOOLEAN DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, work_id)
);
```

```rust
// src/routes/social.rs
pub struct Bookmark {
    pub work_id: i32,
    pub notes: String,
    pub is_private: bool,
    pub created_at: String,
}
```

#### 2. Add a bookmark

```rust
// src/routes/social.rs — POST /api/bookmarks
pub async fn add_bookmark_handler(
    auth_user: AuthUser,
    Json(body): Json<AddBookmarkRequest>,
) -> AppResult<Json<ApiResponse>> {
    let work_id = sqlx::query_scalar!(
        "INSERT INTO bookmarks (user_id, work_id, notes, is_private) VALUES
         ($1::int4, $2::inet::int4, $3, $4)
         ON CONFLICT (user_id, work_id) DO UPDATE SET notes = EXCLUDED.notes, is_private = EXCLUDED.is_private
         RETURNING work_id",
        auth_user.id, body.work_id, &body.notes, body.is_private
    )
    .fetch_one(&auth_user.db)
    .await?;
    Ok(Json(ApiResponse::ok(json!({ "work_id": work_id }))))
}

pub struct AddBookmarkRequest {
    pub work_id: i32,
    pub notes: String,
    pub is_private: bool,
}
```

> **⚠️ Watch Out**: The INSERT uses `ON CONFLICT (user_id, work_id) DO UPDATE SET` — if a user bookmarks the same work twice, it's an UPSERT (updates notes/private, doesn't create a duplicate). The `work_id` is bound as `$2::inet::int4` because sqlx with postgres needs explicit type annotation for integer binds on some PostgreSQL versions.

#### 3. List bookmarks

```rust
// GET /api/bookmarks
pub async fn list_bookmarks_handler(
    auth_user: AuthUser,
) -> AppResult<Json<ApiResponse>> {
    let bookmarks = sqlx::query_as!(
        Bookmark,
        r#"SELECT work_id, notes, is_private, created_at FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC"#,
        auth_user.id
    )
    .fetch_all(&auth_user.db)
    .await?;
    Ok(Json(ApiResponse::ok(json!({ "bookmarks": bookmarks }))))
}
```

#### 4. Remove a bookmark

```rust
// DELETE /api/bookmarks/:work_id
pub async fn remove_bookmark_handler(
    auth_user: AuthUser,
    Path(work_id): Path<i32>,
) -> AppResult<Json<ApiResponse>> {
    sqlx::query!("DELETE FROM bookmarks WHERE user_id = $1::int4 AND work_id = $2::int4", auth_user.id, work_id)
        .execute(&auth_user.db)
        .await?;
    Ok(Json(ApiResponse::ok(json!({}))))
}
```

#### 5. CSV export

```rust
// GET /api/bookmarks/export
pub async fn export_bookmarks_csv(
    auth_user: AuthUser,
) -> AppResult<String> {
    let rows = sqlx::query!("SELECT work_id, notes, is_private, created_at FROM bookmarks WHERE user_id = $1 ORDER BY created_at DESC", auth_user.id)
        .fetch_all(&auth_user.db)
        .await?;
    
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["work_id", "notes", "is_private", "created_at"])?;
    for row in rows {
        wtr.write_record(&[row.work_id.to_string(), row.notes, row.is_private.to_string(), row.created_at])?;
    }
    let data = String::from_utf8(wtr.into_inner()?)?;
    Ok(data)  // Returned as plain text with Content-Disposition: attachment
}
```

> **💡 Key Concept**: CSV export is **immediate** — the server generates the CSV in-memory and returns it as a download. No background job needed because the user's bookmark set is small.

### Try It Yourself

```bash
# Test the bookmark API
curl -X POST /api/bookmarks \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"work_id": 123, "notes": "", "is_private": false}'

# Export your bookmarks as CSV
curl -X GET /api/bookmarks/export \
  -H "Authorization: Bearer <jwt_token>" \
  -o bookmarks.csv
```

### Check

- ✅ `bookmarks` table has `UNIQUE (user_id, work_id)` — prevents duplicates.
- ✅ `/api/bookmarks` POST does UPSERT via `ON CONFLICT`.
- ✅ `/api/bookmarks` GET returns all user's bookmarks, newest first.
- ✅ DELETE removes by `work_id` + `user_id` (authorization-safe).
- ✅ Export returns CSV with `work_id, notes, is_private, created_at` columns.

### What you built

The bookmark data model + API: PostgreSQL table with unique constraint, UPSERT insert, list-by-user, delete-by-work-id, and immediate CSV export.

---

## Chapter 9.2 — CSV Import and the Background Queue

### Goal

Understand how FicHub imports bookmarks from CSV without blocking the HTTP request — a background job pattern.

### Actions

#### 1. Import endpoint accepts CSV

```rust
// src/routes/social.rs — POST /api/bookmarks/import
pub async fn import_bookmarks_csv(
    auth_user: AuthUser,
    mut multipart: Multipart,
) -> AppResult<Json<ApiResponse>> {
    let mut csv_content = String::new();
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let data = field.bytes().await?;
            csv_content = String::from_utf8_lossy(&data).to_string();
        }
    }
    
    if csv_content.is_empty() {
        return Err(AppError::BadRequest("No file contents"));
    }
    
    // Queue the import as a background job (non-blocking)
    let job_id = format!("bookmark_import:{}", uuid::Uuid::new_v4());
    let user_id = auth_user.id;
    
    tokio::spawn(async move {
        // Parse and import CSV in background
        // ... csv crate reads rows, INSERTs into bookmarks table ...
    });
    
    // Return immediately with a job ID — the user is notified via the bell
    Ok(Json(ApiResponse::ok(json!({ "job_id": job_id }))))
}
```

#### 2. Frontend: triggering import

```typescript
// frontend/src/routes/bookmarks/+page.svelte (lines 126-168)
let importing = $state(false);
let importQueued = $state(false);
let importError = $state('');

async function handleImportFile(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    importing = true;
    importQueued = false;
    importError = '';
    try {
        const headers = { ...authHeaders() };
        const formData = new FormData();
        formData.append('file', file);

        const res = await fetch('/api/bookmarks/import', {
            method: 'POST',
            headers,
            body: formData,
        });
        if (!res.ok) { error = 'Import failed.'; return; }

        const data = await res.json();
        if (data.err !== 0) {
            importError = data.msg || 'Import failed.';
            return;
        }
        // Background import: server queues the job and notifies the user
        // when it finishes; show queued status instead of a final count.
        importQueued = true;
        // Refresh bookmarks after a short delay so the notification bell
        // picks up the queued state; the list itself updates on next visit.
        await loadBookmarks();
    } catch { error = 'Import failed.'; }
    finally { importing = false; input.value = ''; }
}
```

> **⚠️ Watch Out**: Import is a **background job**, not synchronous. The server returns immediately with a `job_id`, and the user is notified via the notification bell when it completes. The frontend shows "queued" status. The bookmarks list only refreshes on the next page visit (or when the notification arrives).

#### 3. Export vs Import architecture

| Feature | Export | Import |
|---|---|---|
| Response | Immediate CSV string | Immediate job_id |
| Processing | Synchronous (small data) | Async background job |
| User wait | None — download starts | None — "queued" message shown |
| Scale | Small (user has limited bookmarks) | Large (CSV could be 1000+ rows) |

### Try It Yourself

```typescript
// Add an import progress indicator
// 1. Store the job_id from the response
// 2. Poll /api/jobs/:job_id every 2s
// 3. Show progress bar, then "Import complete!"
const data = await res.json();
const jobId = data.job_id;
const timer = setInterval(async () => {
    const r = await fetch(`/api/jobs/${jobId}`, { headers: authHeaders() });
    const status = await r.json();
    if (status.done) clearInterval(timer);
}, 2000);
```

### Check

- ✅ Import uses `multipart/form-data` (file upload via `Multipart`).
- ✅ Server returns immediately with a `job_id` (background `tokio::spawn`).
- ✅ Frontend shows "queued" status, doesn't wait for completion.
- ✅ Export is synchronous (CSV returned immediately).
- ✅ Input file is reset after import (`input.value = ''`).

### What you built

The background import pattern — accept CSV via multipart, return immediately with a job_id, spawn a tokio task for the heavy work, and notify the user via the bell when done. Contrasted with the synchronous export.

---

## Chapter 9.3 — Bookmarks Page Frontend

### Goal

Build the bookmarks page with dual UI mode (archive + modern), parallel metadata fetching, and bookmark management.

### Actions

#### 1. Auth guard and data loading

```typescript
// frontend/src/routes/bookmarks/+page.svelte (lines 2-96)
import { onMount } from 'svelte';
import { listBookmarks, removeBookmark, getWork, authHeaders } from '$lib/api/social';
import type { Bookmark } from '$lib/api/social-types';
import { auth } from '$lib/stores/auth.svelte';
import { getPref } from '$lib/prefs';
import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

const uiMode = $derived(getPref('uiMode'));

let bookmarks = $state<Bookmark[]>([]);
let loading = $state(true);
let error = $state('');

onMount(async () => {
    const tab = $page.url.searchParams.get('tab');  // restore ?tab=list|search
    if (tab === 'list' || tab === 'search') showMode = tab;
    await auth.init();
    if (!auth.isLoggedIn) { loading = false; return; }
    await loadBookmarks();
});
```

#### 2. Parallel metadata fetching

```typescript
async function loadBookmarks() {
    loading = true; error = '';
    try {
        const res = await listBookmarks();
        if (res.err !== 0) { error = 'Failed to load bookmarks.'; return; }
        bookmarks = res.bookmarks;
        // Fetch metadata for each bookmark in parallel via /api/works/{id}.
        const entries = await Promise.allSettled(
            bookmarks.map(async (b) => {
                const data = await getWork(b.work_id);
                return { work_id: b.work_id, work: data.work };
            }),
        );
        for (const entry of entries) {
            if (entry.status === 'fulfilled' && entry.value.work) {
                const w = entry.value.work;
                metadata[entry.value.work_id] = {
                    title: w.canonical_title,
                    author: w.canonical_author,
                    words: w.sources[0]?.words ?? 0,
                    chapters: w.sources[0]?.chapters ?? 0,
                    source: w.sources[0]?.source ?? '',
                };
            }
        }
    } catch { error = 'Network error loading bookmarks.'; }
    finally { loading = false; }
}
```

> **💡 Key Concept**: `Promise.allSettled` is used instead of `Promise.all` because if one work's metadata fetch fails, it shouldn't prevent the others from loading. Each failed entry is simply skipped (no metadata shown), and the bookmark still renders with just the work_id.

#### 3. Archive mode (AO3-style)

```svelte
{#if uiMode === 'archive'}
<main class="archive-main">
    <div class="archive-content">
        <header class="archive-header">
            <h1 class="archive-title">Bookmarks</h1>
            <p class="bookmark-subnav">
                <a class="bookmark-tab" class:active={showMode === 'list'}
                   href="/bookmarks?tab=list"
                   onclick={(e) => { e.preventDefault(); switchTab('list'); }}>Bookmarks</a>
                <a class="bookmark-tab" class:active={showMode === 'search'}
                   href="/bookmarks?tab=search"
                   onclick={(e) => { e.preventDefault(); switchTab('search'); }}>Search Bookmarks</a>
            </p>
        </header>

        {#if !auth.isLoggedIn}
            <div class="archive-empty"><p>Log in to see your bookmarks.</p><a href="/login">Log In</a></div>
        {:else if showMode === 'search'}
            <!-- AO3-style bookmark search form: <dl> with dt labels -->
            <div class="bookmark-search-form">
                <h3 class="landmark heading">Bookmark Search</h3>
                <dl>
                    <div class="dl-row">
                        <dt><label for="bsearch-bookmark">Bookmark</label></dt>
                        <dd><input id="bsearch-bookmark" type="search" placeholder="Search your bookmarks" aria-label="Search bookmarks" /></dd>
                    </div>
                    <!-- ... more search fields ... -->
                </dl>
                <p class="submit actions"><ArchiveButton type="submit">Search</ArchiveButton></p>
            </div>
        {:else if loading}
            <div class="archive-skeleton-list">{#each Array(3) as _, i}<article class="skeleton-blurb">...</article>{/each}</div>
        {:else if bookmarks.length === 0}
            <div class="archive-empty"><p>No bookmarks yet.</p></div>
        {:else}
            <div class="archive-bookmark-list">
                {#each bookmarks as b (b.work_id)}
                    {@const meta = metadata[b.work_id]}
                    {#if meta}
                        <WorkBlurb fic={{ url_id: '', title: meta.title, author: meta.author, ... }} />
                    {:else}
                        <article class="archive-fallback-card">
                            <span class="fallback-id">Work #{b.work_id}</span>
                            <button class="archive-remove-btn" onclick={() => handleRemove(b.work_id)}>Remove</button>
                            {#if b.notes}<p class="fallback-notes">[{b.notes}]</p>{/if}
                            <p class="fallback-hint">Work metadata unavailable.</p>
                        </article>
                    {/if}
                {/each}
            </div>
        {/if}
    </div>
</main>
{/if}
```

#### 4. Optimistic removal

```typescript
let removing = $state<number | null>(null);

async function handleRemove(work_id: number) {
    removing = work_id;
    try {
        const res = await removeBookmark(work_id);
        if (res.err === 0) {
            bookmarks = bookmarks.filter((b) => b.work_id !== work_id);  // optimistic
        }
    } catch { error = 'Failed to remove bookmark.'; }
    finally { removing = null; }
}
```

### Try It Yourself

```typescript
// Add drag-and-drop reordering for bookmarks
// 1. Add a drag handle to each bookmark
// 2. On drop, reorder the local array
// 3. PATCH /api/bookmarks/order with the new positions
```

### Check

- ✅ Archive mode uses AO3-style `<dl>` with `<dt>`/`<dd>` rows for forms.
- ✅ Modern mode uses card-based layout with `.bookmark-list` flexbox.
- ✅ `Promise.allSettled` for parallel metadata — failed entries don't block others.
- ✅ Optimistic removal: bookmark disappears immediately on success.
- ✅ `showMode` (`list`/`search`) syncs to URL via `?tab=` parameter.
- ✅ Archive mode has a separate "Bookmark Search" form (AO3-style, not the global search).

### What you built

The bookmarks page — auth-guarded, dual UI mode (archive `<dl>` + modern cards), parallel metadata fetching with `Promise.allSettled`, optimistic removal, tab-based view switching with URL sync, and CSV import/export buttons.

---

## Conclusion

You now understand FicHub's complete bookmark system:

1. **API** — bookmark table (UPSERT via ON CONFLICT), list-by-user, DELETE by work_id, immediate CSV export.
2. **Import** — background `tokio::spawn` job, returns `job_id` immediately, user notified via bell.
3. **Frontend** — dual UI mode, `Promise.allSettled` for parallel metadata, optimistic remove, `?tab=` URL sync, WorkBlurb reuse.
4. **Design** — export is synchronous (small data), import is async (large CSV), `WorkBlurb` shared component across archive results.