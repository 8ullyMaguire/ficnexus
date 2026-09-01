# Part 27 — Reading Quests and Stats

Every time you read a fic, FicHub remembers what you did. It tracks your **reading status** (want to read, reading, completed, dropped), your **word count**, your **login streak**, and even your recent activity. All of this lives in one backend file — `src/routes/quests.rs` — and shows up on two frontend pages: `/reading` and `/stats`.

This part walks through all four backend endpoints and shows how the frontend pages call them. You'll also write the exact queries that power them.

---

## 27.1 Backend: `POST /api/reading/status` — update reading status

Open `src/routes/quests.rs`. This handler is the one that lets a user say *"I want to read this"* or *"I'm reading it now"* or *"I finished it!"* — just like AO3's reading status dropdowns.

```rust
// src/routes/quests.rs (lines 12-17, 86-103)
#[derive(Debug, Deserialize)]
pub struct UpdateReadingStatusBody {
    pub work_id: i32,
    pub status: String,
    pub current_chapter: Option<i32>,
}

/// POST /api/reading/status — update reading status for a work
pub async fn update_reading_status_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateReadingStatusBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate status
    match body.status.as_str() {
        "want_to_read" | "reading" | "completed" | "dropped" => {}
        _ => return Err(AppError::BadRequest(format!("Invalid status: '{}'. Must be one of: want_to_read, reading, completed, dropped", body.status))),
    }

    queries::update_reading_status(&state.db, user_id, body.work_id, &body.status, body.current_chapter).await?;

    Ok(Json(json!({ "err": 0, "msg": "Reading status updated" })))
}
```

### Breakdown

**The `AuthUser` extractor**: You've seen this before in Part 8. It reads the `Authorization: Bearer *** header, verifies the JWT, and gives you `auth.user_id`. If there's no token, `user_id` is `None` and the handler returns 401.

**Body validation in two layers**: First, Rust checks that the JSON body matches the struct — `work_id` must be an integer, `status` must be a string, and `current_chapter` is optional (nullable). Then the handler manually validates the `status` string against the four allowed values: `want_to_read`, `reading`, `completed`, and `dropped`. Any other value returns a `BadRequest` error with a helpful message.

**The upsert pattern**: The handler calls `queries::update_reading_status`, which uses a PostgreSQL `INSERT ... ON CONFLICT DO UPDATE` — also called an "upsert." Here's the query in `src/db/queries.rs` (lines 3313-3335):

```rust
// src/db/queries.rs (lines 3313-3335)
/// Update reading status for a work
pub async fn update_reading_status(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    status: &str,
    current_chapter: Option<i32>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_stats (user_id, work_id, status, current_chapter)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, work_id) DO UPDATE SET
               status = $3,
               current_chapter = COALESCE($4, reading_stats.current_chapter),
               last_read_at = NOW()"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(status)
    .bind(current_chapter)
    .execute(pool)
    .await?;
    Ok(())
}
```

**Why `ON CONFLICT`**: The `reading_stats` table has a unique constraint on `(user_id, work_id)`. If the user has never set a status for this work, the INSERT creates a new row. If they already have a row, PostgreSQL detects the conflict and runs the UPDATE instead — setting the new status, optionally updating the chapter (or keeping the old one if `current_chapter` is `None`), and stamping `last_read_at` to `NOW()`.

**`COALESCE($4, reading_stats.current_chapter)`**: This is a neat trick. If the caller passes `current_chapter: None` (meaning "I don't want to change the chapter"), the update keeps the existing chapter number. If the caller passes a number, it overwrites it.

### Where it's wired up

In `src/server.rs` (lines 688-689):

```rust
// src/server.rs (lines 688-689, excerpt)
.route("/api/reading/status", axum::routing::post(crate::routes::quests::update_reading_status_handler))
.route("/api/reading/list", get(crate::routes::quests::get_reading_list_handler))
```

The route is registered with `post(...)` because it only accepts POST requests. A GET to this URL would return a 405 Method Not Allowed.

### The `reading_stats` table

```sql
-- The table that holds per-user, per-work reading state
CREATE TABLE reading_stats (
    id              bigserial PRIMARY KEY,
    user_id         integer NOT NULL REFERENCES users(id),
    work_id         integer NOT NULL REFERENCES works(id),
    words_read      bigint DEFAULT 0,
    read_count      integer DEFAULT 0,
    status          text DEFAULT 'want_to_read',
    current_chapter integer,
    last_read_at    timestamptz DEFAULT NOW(),
    created_at      timestamptz DEFAULT NOW(),
    UNIQUE (user_id, work_id)
);
```

Every user-work pair gets exactly one row. The `status` column defaults to `want_to_read` — so as soon as you set a status, you get a row with that status. The `words_read` and `read_count` columns are updated by a *different* endpoint (the one in section 29.3).

---

## 27.2 Backend: `GET /api/reading/list` — reading list

Once a user has set statuses on some works, they need a way to see all of them at once. That's the reading list endpoint:

```rust
// src/routes/quests.rs (lines 19-22, 105-136)
#[derive(Debug, Deserialize)]
pub struct ReadingListQuery {
    pub status: Option<String>,
}

/// GET /api/reading/list — get reading list, optionally filtered by status
pub async fn get_reading_list_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReadingListQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Validate status filter if provided
    if let Some(ref status) = query.status {
        match status.as_str() {
            "want_to_read" | "reading" | "completed" | "dropped" => {}
            _ => return Err(AppError::BadRequest(format!("Invalid status filter: '{}'", status))),
        }
    }

    let list = queries::get_reading_stats_list(&state.db, user_id, query.status.as_deref()).await?;

    let items: Vec<Value> = list.into_iter().map(|r| {
        json!({
            "id": r.id,
            "work_id": r.work_id,
            "words_read": r.words_read,
            "read_count": r.read_count,
            "status": r.status,
            "current_chapter": r.current_chapter,
            "last_read_at": r.last_read_at.to_rfc3339(),
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "reading_list": items })))
}
```

### Breakdown

**Query parameters**: The `ReadingListQuery` struct uses `Option<String>` for `status`. When the client calls `/api/reading/list?status=reading`, Axum parses `reading` into `query.status` as `Some("reading")`. When no query parameter is present, it's `None` and the endpoint returns the full list.

**Status filter validation**: Same four valid values as the status-update endpoint. If the client sends `?status=bogus`, the handler returns a `BadRequest` error before touching the database — no SQL injection risk from unvalidated input.

**The database query** in `src/db/queries.rs` (lines 3338-3365) branches on whether a status filter was provided:

```rust
// src/db/queries.rs (lines 3338-3365)
/// Get reading list rows for a user, optionally filtered by status
pub async fn get_reading_stats_list(
    pool: &PgPool,
    user_id: i32,
    status_filter: Option<&str>,
) -> AppResult<Vec<ReadingStats>> {
    let rows = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 AND status = $2 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .bind(status)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows)
}
```

**Two queries, one branch**: The `match` either produces a two-parameter query (with status filter) or a one-parameter query (without). Both select the same columns. The results are mapped into the `ReadingStats` struct:

```rust
// src/db/models.rs (lines 244-253)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingStats {
    pub id: i64,
    pub user_id: i32,
    pub work_id: i32,
    pub words_read: i64,
    pub last_read_at: DateTime<Utc>,
    pub read_count: i32,
    pub status: String,
    pub current_chapter: Option<i32>,
}
```

**The response shape**: Each item in `reading_list` carries all eight fields. The `last_read_at` is converted to an RFC 3339 string (e.g. `"2024-08-21T14:30:00+00:00"`) so the frontend can parse it with `new Date()` natively.

**Route registration**: Also in `src/server.rs` (line 689):

```rust
.route("/api/reading/list", get(crate::routes::quests::get_reading_list_handler))
```

Uses `get(...)` — only GET requests are accepted. No status filter means "all statuses."

---

## 27.3 Backend: `POST /api/reading/record` — record a read

Setting a status is about *what you plan to do*. Recording a read is about *what you actually did*. When you finish reading a chapter, the reader page calls this endpoint to increment your word count and read count:

```rust
// src/routes/quests.rs (lines 51-62)
/// POST /api/reading/record — record reading progress
pub async fn record_read_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<crate::routes::social::ReadingRecordBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    queries::record_work_read(&state.db, user_id, body.work_id, body.words_read).await?;

    Ok(Json(json!({ "err": 0, "msg": "Reading recorded" })))
}
```

### Breakdown

**Body reuse**: The `ReadingRecordBody` struct is actually defined in `src/routes/social.rs` (line 654), not in `quests.rs`:

```rust
// src/routes/social.rs (lines 654-658)
#[derive(Debug, Deserialize)]
pub struct ReadingRecordBody {
    pub work_id: i32,
    pub words_read: i64,
}
```

This reuse is intentional — both the quests module (recording reading progress) and the social module (which originally owned the type) reference the same struct. It's a small DRY pattern that keeps the API body shape consistent.

**Auth check**: `auth.user_id.ok_or_else(...)` — same pattern as the other handlers. No token, no recording.

**The database transaction** (in `src/db/queries.rs`, lines 1155-1188):

```rust
// src/db/queries.rs (lines 1155-1188)
/// Record reading a work
pub async fn record_work_read(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    words_read: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_stats (user_id, work_id, words_read)
           VALUES ($1, $2, $3)
           ON CONFLICT (user_id, work_id) DO UPDATE SET
               words_read = reading_stats.words_read + $3,
               read_count = reading_stats.read_count + 1,
               last_read_at = NOW()"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(words_read)
    .execute(pool)
    .await?;

    // Update user aggregate stats
    sqlx::query(
        "UPDATE users SET total_words_read = total_words_read + $1, total_works_read = total_works_read + 1, last_active_at = NOW() WHERE id = $2",
    )
    .bind(words_read)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Check reading badges
    check_and_award_badges(pool, user_id, "work_read").await?;

    Ok(())
}
```

**Three things happen in sequence**:

1. **Upsert into `reading_stats`**: If the user has a row for this work, `words_read` is *added* (not replaced — that's the `+ $3` part). `read_count` goes up by 1. `last_read_at` is stamped. If no row exists, a new one is created with the given `words_read` and `read_count` starts at 1.

2. **Update the `users` table**: The user's `total_words_read` and `total_works_read` columns are incremented. `last_active_at` is stamped to `NOW()`. This is the aggregate data that shows up on the stats page — it avoids expensive `SUM()` queries every time stats are requested.

3. **Badge check**: After the read is recorded, `check_and_award_badges` is called with the trigger `"work_read"`. This is the quest system — certain badges unlock after reading N works or N words. The badge check is fire-and-forget; if it fails, the read is still recorded.

**Route registration** (in `src/server.rs`, line 693):

```rust
.route("/api/reading/record", axum::routing::post(crate::routes::quests::record_read_handler))
```

Like the status endpoint, this is POST-only. The reader page calls it after you finish a chapter.

### What the reader page does

The reader page at `frontend/src/routes/read/[urlId]/+page.svelte` imports several API functions on line 7:

```typescript
// frontend/src/routes/read/[urlId]/+page.svelte (line 7, excerpt)
import { updateReadingStatus, getChapterTranslations, listLocales, recordReadHistory, giveKudos, removeKudos, getKudos, addBookmark, removeBookmark, listBookmarks, addComment, fetchThreadedComments } from '$lib/api/social';
```

And later, when the reader loads, it records a visit:

```typescript
// frontend/src/routes/read/[urlId]/+page.svelte (lines 494-498)
// Record a reading history visit (AO3-style per-visit log)
if (auth.isLoggedIn && work?.work_id) {
  void recordReadHistory(work.work_id, saved.chapterIndex != null ? saved.chapterIndex + 1 : undefined)
    .catch(() => { /* best-effort */ });
}
```

This is the `recordReadHistory` function (not `recordWorkRead` — note the distinction). History recording is best-effort: it uses `void` and `.catch()` so a failed API call never breaks the reading experience.

---

## 27.4 Backend: `GET /api/users/:id/streak` — reading streak

A streak is how many days in a row you've logged in. The stats page shows both your current streak and your longest-ever streak:

```rust
// src/routes/quests.rs (lines 64-84)
/// GET /api/v1/users/{id}/streak — login streak
pub async fn get_streak_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let streak = queries::get_login_streak(&state.db, user_id).await?;

    match streak {
        Some(s) => Ok(Json(json!({
            "err": 0,
            "current_streak": s.current_streak,
            "longest_streak": s.longest_streak,
            "last_login_date": s.last_login_date.to_string(),
        }))),
        None => Ok(Json(json!({
            "err": 0,
            "current_streak": 0,
            "longest_streak": 0,
        }))),
    }
}
```

### Breakdown

**Path parameter**: `axum::extract::Path(user_id): axum::extract::Path<i32>` extracts the user ID from the URL path. When the frontend calls `/api/users/42/streak`, `user_id` is `42`.

**Public endpoint**: Unlike the other quests handlers, this one does **not** require auth — it doesn't take an `AuthUser` parameter. You can look up anyone's streak. The streak is stored in the `login_streaks` table, which is updated on login (a separate system in the auth flow).

**The database query** (in `src/db/queries.rs`, line 1139):

```rust
// src/db/queries.rs (lines 1139-1147)
pub async fn get_login_streak(pool: &PgPool, user_id: i32) -> AppResult<Option<LoginStreak>> {
    let row = sqlx::query_as::<_, LoginStreak>(
        "SELECT user_id, current_streak, longest_streak, last_login_date, updated_at FROM login_streaks WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

**The `LoginStreak` struct** (in `src/db/models.rs`, lines 233-239):

```rust
// src/db/models.rs (lines 233-239)
pub struct LoginStreak {
    pub user_id: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_login_date: NaiveDate,
    pub updated_at: DateTime<Utc>,
}
```

**`fetch_optional`**: Returns `None` if no row exists (the user has never logged in, or their streak row hasn't been created yet). The handler handles this gracefully — returning zeros instead of an error. This means the API never returns a 500 for a missing streak; it just says "0 days."

**The `last_login_date`**: This is a `NaiveDate` (no timezone). The handler calls `.to_string()` which produces a simple `"2024-08-21"` string.

**Graceful degradation**: The `None` branch returns `current_streak: 0, longest_streak: 0` — no `last_login_date` field. The stats page checks for this (see section 29.5).

**Route registration** (in `src/server.rs`, line 694):

```rust
.route("/api/users/{id}/streak", get(crate::routes::quests::get_streak_handler))
```

The `{id}` path parameter is Axum's syntax. It matches any integer in that position.

---

## 27.5 Frontend: reading stats page, quest progress

The stats page lives at `frontend/src/routes/stats/+page.svelte`. It pulls data from **four** different endpoints and stitches them together into a single view:

1. `GET /api/users/:id/reading-stats` — your personal totals
2. `GET /api/users/:id/streak` — your login streak
3. `GET /api/reading/list?status=completed` — your recently completed reads
4. `GET /api/site/stats` — site-wide totals

Here's the script block that loads all this data:

```typescript
// frontend/src/routes/stats/+page.svelte (lines 1-115, excerpt)
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { getPref } from '$lib/prefs';
  import { t } from '$lib/i18n/index.svelte';
  import { getReadingStats, getStreak } from '$lib/api/social';
  import { relativeTime } from '$lib/util';

  interface RecentItem {
    work_id: number;
    words_read: number;
    read_count: number;
    last_read_at: string;
    title?: string;
  }

  let stats = $state<{
    total_words_read: number;
    total_works_read: number;
    login_streak: number;
    recent: RecentItem[];
  } | null>(null);

  let streak = $state<{
    current_streak: number;
    longest_streak: number;
    last_login_date: string;
  } | null>(null);

  let readingList = $state<any[]>([]);
  let siteStats = $state<SiteStatsResponse | null>(null);
  let personalAnalytics = $state<any>(null);
  let loading = $state(true);
  let error = $state('');

  const uiMode = $derived(getPref('uiMode'));

  onMount(async () => {
    await auth.init();
    const promises: Promise<any>[] = [];

    if (auth.isLoggedIn && auth.user) {
      promises.push(
        Promise.all([
          getReadingStats(auth.user.id),
          fetch('/api/reading/analytics', { credentials: 'include' }).then(r => r.ok ? r.json() : null),
          getStreak(auth.user.id),
          fetch('/api/reading/list?status=completed&per_page=10', { credentials: 'include' }).then(r => r.ok ? r.json() : { reading_list: [] }),
        ]).then(([statsRes, analyticsRes, streakRes, readingRes]) => {
          personalAnalytics = analyticsRes;
          if (statsRes && (statsRes as any).err === 0) stats = statsRes as any;
          else if (statsRes && !(statsRes as any).err) stats = statsRes as any;
          if (streakRes && (streakRes as any).err === 0) streak = streakRes as any;
          readingList = (readingRes as any).reading_list || (readingRes as any).items || [];
        }).catch(() => {})
      );
    }

    promises.push(
      fetch('/api/site/stats', { credentials: 'include' })
        .then(r => r.ok ? r.json() : null)
        .then(data => { if (data && data.err === 0 && Array.isArray(data.periods)) siteStats = data as SiteStatsResponse; })
        .catch(() => null)
    );

    try {
      await Promise.all(promises);
    } catch {
      error = 'Failed to load statistics.';
    } finally {
      loading = false;
    }
  });
</script>
```

### Breakdown

**Parallel loading with `Promise.all`**: The stats page doesn't wait for each API call to finish before starting the next. It fires all four requests simultaneously and waits for them all with `Promise.all`. This makes the page load faster — instead of 4 sequential round-trips, it's just 1 (plus whatever the slowest response takes).

**Auth gate**: `auth.init()` runs first to restore the cached user from localStorage. If `auth.isLoggedIn` is false, the personal stats section shows a "Log in to see your reading totals" prompt instead.

**The `getReadingStats` function** (in `frontend/src/lib/api/social.ts`, lines 689-693):

```typescript
// frontend/src/lib/api/social.ts (lines 689-693)
export async function getReadingStats(user_id: number): Promise<{
  err: number; total_words_read: number; total_works_read: number;
  login_streak: number; recent: { work_id: number; words_read: number; read_count: number; last_read_at: string }[]
}> {
  return request(`/users/${user_id}/reading-stats`);
}
```

**The `getStreak` function** (lines 696-697):

```typescript
export async function getStreak(user_id: number): Promise<{ err: number; current_streak: number; longest_streak: number }> {
  return request(`/users/${user_id}/streak`);
}
```

Both use the central `request()` helper, which automatically attaches the `Authorization: Bearer *** header from localStorage.

**Two UI modes**: The page renders differently based on `uiMode` — `'archive'` (the AO3-style layout) or `'modern'` (the default FicHub look). The archive mode uses Georgia serif fonts, a border-left summary block, and tables for data. The modern mode uses card-based stat blocks with large numbers.

**Rendering the personal stats** (in archive mode):

```svelte
<!-- frontend/src/routes/stats/+page.svelte (lines 188-203, excerpt) -->
{#if stats}
  <section class="archive-section">
    <h2 class="archive-subhead">Your reading</h2>
    <blockquote class="archive-summary">
      <dl class="stats">
        <div class="stat"><dt>Words read</dt><dd>{(stats.total_words_read ?? 0).toLocaleString()}</dd></div>
        <div class="stat"><dt>Works read</dt><dd>{stats.total_works_read ?? 0}</dd></div>
        {#if streak}
          <div class="stat"><dt>Current streak</dt><dd>{streak.current_streak ?? 0} day{(streak.current_streak ?? 0) === 1 ? '' : 's'}</dd></div>
          <div class="stat"><dt>Longest streak</dt><dd>{streak.longest_streak ?? 0} day{(streak.longest_streak ?? 0) === 1 ? '' : 's'}</dd></div>
        {:else if stats.login_streak !== undefined}
          <div class="stat"><dt>Login streak</dt><dd>{stats.login_streak ?? 0} day{(stats.login_streak ?? 0) === 1 ? '' : 's'}</dd></div>
        {/if}
      </dl>
    </blockquote>
  </section>
{:else}
  <section class="archive-section">
    <h2 class="archive-subhead">Your reading</h2>
    <p class="archive-muted">No personal totals yet — start reading and they will appear here.</p>
  </section>
{/if}
```

**Fallback logic**: If the `streak` endpoint hasn't loaded yet (but `stats` has), the page falls back to `stats.login_streak` — which is the streak value returned by the `reading-stats` endpoint. This is a redundancy: the `reading-stats` handler calls `get_user_reading_aggregate` which does a subquery into `login_streaks`, so the streak is available even if the separate `get_streak` call hasn't resolved yet.

**The `reading_stats` and `login_streaks` tables**:

```sql
-- reading_stats: per-user, per-work state (from section 29.1)
-- login_streaks: per-user daily login chain
CREATE TABLE login_streaks (
    user_id         integer PRIMARY KEY REFERENCES users(id),
    current_streak  integer DEFAULT 0,
    longest_streak  integer DEFAULT 0,
    last_login_date date,
    updated_at      timestamptz DEFAULT NOW()
);
```

The streak is updated by the auth/login system whenever you log in or visit the site with a valid session. The `current_streak` goes up by 1 each consecutive day; if you miss a day, it resets to 0. `longest_streak` keeps the all-time high.

**The `/reading` page**: The sibling page at `frontend/src/routes/reading/+page.svelte` shows your full reading list with tab filtering. It imports `getReadingList` and `getWork`:

```typescript
// frontend/src/routes/reading/+page.svelte (lines 1-8, excerpt)
import { onMount } from 'svelte';
import { getReadingList, getWork } from '$lib/api/social';
import type { ReadingListItem, ReadingStatus, Work } from '$lib/api/social-types';
import { auth } from '$lib/stores/auth.svelte';
import { goto } from '$app/navigation';
import { formatWords, relativeTime } from '$lib/util';
import { getPref } from '$lib/pref';
```

The page defines tab options for switching between statuses:

```typescript
// frontend/src/routes/reading/+page.svelte (lines 12-27, excerpt)
const TABS: { label: string; value: ReadingStatus | '' }[] = [
  { label: 'All', value: '' },
  { label: '📋 Want to Read', value: 'want_to_read' },
  { label: '📖 Reading', value: 'reading' },
  { label: '✅ Completed', value: 'completed' },
  { label: '❌ Dropped', value: 'dropped' },
];

// Archive-mode tab labels (no emoji — AO3 archive style).
const ARCHIVE_TABS: { label: string; value: ReadingStatus | '' }[] = [
  { label: 'All', value: '' },
  { label: 'Want to Read', value: 'want_to_read' },
  { label: 'Reading', value: 'reading' },
  { label: 'Completed', value: 'completed' },
  { label: 'Dropped', value: 'dropped' },
];
```

When a tab is clicked, `switchTab(tab.value)` updates `activeTab` and calls `loadList()`, which fetches the filtered reading list:

```typescript
// frontend/src/routes/reading/+page.svelte (lines 21-65, excerpt)
async function loadList() {
  loading = true; error = '';
  try {
    const res = activeTab
      ? await getReadingList(activeTab as ReadingStatus)
      : await getReadingList();
    if (res.err === 0) {
      items = res.reading_list;
      // Fetch work metadata
      await Promise.allSettled(
        items.map(async (r) => {
          try {
            const w = await getWork(r.work_id);
            works[r.work_id] = w.work;
          } catch {
            works[r.work_id] = null;
          }
        })
      );
    } else {
      error = 'Failed to load reading list.';
    }
  } catch { error = 'Network error.'; }
  finally { loading = false; }
}
```

**The types** (in `frontend/src/lib/api/social-types.ts`, lines 357-367):

```typescript
// frontend/src/lib/api/social-types.ts (lines 357-367)
export type ReadingStatus = 'want_to_read' | 'reading' | 'completed' | 'dropped';

export interface ReadingListItem {
  id: number;
  work_id: number;
  words_read: number;
  read_count: number;
  status: ReadingStatus;
  current_chapter: number | null;
  last_read_at: string;
}
```

The `ReadingStatus` type is a union of four string literals. TypeScript uses this to enforce that only valid status values are passed to `updateReadingStatus` and `getReadingList`. If you try to pass `'done'` instead of `'completed'`, TypeScript will error at build time.

**The `getReadingList` function** (in `social.ts`, lines 780-783):

```typescript
export async function getReadingList(status?: ReadingStatus): Promise<{ err: number; reading_list: ReadingListItem[] }> {
  const query = status ? `?status=${status}` : '';
  return request(`/reading/list${query}`);
}
```

When `status` is provided, it appends `?status=reading` (or whatever) to the URL. When it's not, it fetches the full list.

**The `updateReadingStatus` function** (lines 773-778):

```typescript
export async function updateReadingStatus(work_id: number, status: ReadingStatus, current_chapter?: number): Promise<{ err: number; msg: string }> {
  return request('/reading/status', {
    method: 'POST',
    body: JSON.stringify({ work_id, status, current_chapter }),
  });
}
```

This is the function the work page calls when you click a status button. It POSTs to `/reading/status` with the work ID, the new status, and an optional chapter number.

---

## Try It Yourself

### Exercise 1: Set a reading status

Start the server and log in to get a token. Then update a work's reading status:

```bash
curl -s -X POST http://localhost:8000/api/reading/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"work_id": 1, "status": "want_to_read"}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "Reading status updated"}`

### Exercise 2: Try an invalid status

```bash
curl -s -X POST http://localhost:8000/api/reading/status \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"work_id": 1, "status": "done"}' | python3 -m json.tool
```

**Expected**: An error message like `Invalid status: 'done'. Must be one of: want_to_read, reading, completed, dropped`. The handler returns a 400 Bad Request.

### Exercise 3: List your reading list

```bash
curl -s http://localhost:8000/api/reading/list \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: `{"err": 0, "reading_list": [...]}`. After Exercise 1, the list will contain at least one entry with `status: "want_to_read"`.

### Exercise 4: Filter by status

```bash
curl -s "http://localhost:8000/api/reading/list?status=want_to_read" \
  -H "Authorization: Bearer ***" | python3 -m json.tool
```

**Expected**: Same as Exercise 3, but only entries with `status: "want_to_read"`. Try `?status=reading` or `?status=completed` to see different results.

### Exercise 5: Record a read

```bash
curl -s -X POST http://localhost:8000/api/reading/record \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ***" \
  -d '{"work_id": 1, "words_read": 2500}' | python3 -m json.tool
```

**Expected**: `{"err": 0, "msg": "Reading recorded"}`. Now check the reading list again — the entry for work 1 will have `words_read: 2500` and `read_count: 1`.

### Exercise 6: Check your streak

First, find your user ID (it's in your JWT token or from `/api/auth/me`). Then:

```bash
curl -s http://localhost:8000/api/users/42/streak | python3 -m json.tool
```

**Expected**: `{"err": 0, "current_streak": 3, "longest_streak": 12, "last_login_date": "2024-08-21"}`. If you've never logged in on consecutive days, `current_streak` will be 0 or 1.

### Exercise 7: View the stats page

Open `http://localhost:5173/stats` in your browser (the SvelteKit dev server). You should see:

- **Site-wide** table: views, kudos, bookmarks, works read, reviews per time period
- **Your reading** block: total words read, works read, current streak, longest streak
- **Recently read** table: the last 10 completed works with word counts and read dates

---

## Troubleshooting

**"Login required" on POST /api/reading/status**: Make sure you're passing the `Authorization: Bearer *** header. The `AuthUser` extractor returns 401 if `user_id` is missing. Get a token from `/api/auth/login` first.

**"Invalid status" error**: The status must be exactly one of: `want_to_read`, `reading`, `completed`, `dropped`. No abbreviations, no aliases. Check your spelling.

**Reading list is empty after setting a status**: The `updateReadingStatus` handler creates a row in `reading_stats` with the given status. The `get_reading_list_handler` reads from the same table. If you're logged in as different users for each call, you won't see the data. Use the same token.

**Streak is always 0**: The `login_streaks` table is populated by the login system, not the reading endpoints. If you've never logged in on consecutive days, your streak is 0. Log in each day to build it up. Also check that the `login_streaks` table exists in your database migrations.

**Stats page shows "Log in to see personal totals"**: The page checks `auth.isLoggedIn` before making personal API calls. If you're not logged in, it shows a prompt instead. Make sure you've logged in and your token is in localStorage.

**`words_read` doesn't match what you expect**: The `record_work_read` function *adds* words to the existing count (`words_read = reading_stats.words_read + $3`). If you call it twice with `words_read: 2500`, the total becomes 5000. To reset, you'd need to delete the row in `reading_stats` for that user-work pair first.

**Site stats table is empty on the stats page**: The `/api/site/stats` endpoint aggregates analytics data. If your database has no `request_log` entries, the analytics will be empty. Visit a few pages in the reader to generate activity, then refresh.

**The `reading/analytics` fetch fails silently**: The stats page fetches `/api/reading/analytics` with `fetch` (not the `request()` helper), so 401s don't trigger the redirect. If it fails, `personalAnalytics` stays null and the page doesn't show analytics — but the rest of the page still works. This is intentional: analytics are a bonus, not a requirement.

---

## What you have now

- You understand reading status: `POST /api/reading/status` validates the status string, uses an upsert into `reading_stats`, and handles optional chapter updates with `COALESCE`.
- You understand the reading list: `GET /api/reading/list` accepts an optional `?status=` query parameter, validates it, and branches between two SQL queries (filtered vs. unfiltered).
- You understand recording a read: `POST /api/reading/record` increments `words_read` and `read_count` via an upsert, updates the user's aggregate totals, and triggers badge checks.
- You understand login streaks: `GET /api/users/:id/streak` reads from the `login_streaks` table, returns zeros for unknown users, and is a public endpoint (no auth required).
- You understand the stats page: it fires four API calls in parallel with `Promise.all`, handles two UI modes (archive vs. modern), and has fallback logic when the streak endpoint hasn't loaded yet.
- You understand the reading list page: tabbed filtering by status, emoji vs. archive labels, parallel work metadata fetching, and both UI modes.
- You know the difference between reading stats (words/counts) and reading history (AO3-style per-visit log).

Next: Part 28 — Reading History. You will build the AO3-style per-visit log that records every time you open a fic.

---

*End of Part 27. On to [Part 28 — Reading History](./28-reading-history/28-reading-history.md).*
