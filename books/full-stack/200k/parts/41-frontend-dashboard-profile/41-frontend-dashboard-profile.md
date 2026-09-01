# Part 41 — Frontend Dashboard and Profile

Now we build the pages people actually land on after logging in: the **dashboard**, the **author profile page**, and the **updates feed**. These are the "lobby" views that greet a reader and help them see their own activity at a glance. FicHub is special here because it ships **two complete UIs in one codebase**: a "modern" default look and an **AO3-style archive mode** that looks and feels just like Archive of Our Own. You'll see the same `uiMode` switch driving both designs throughout this part.

---

## 41.1 `src/routes/dashboard/+page.svelte` — User Dashboard

The dashboard is your reading HQ. It pulls together several API calls — bookmarks, reading stats, login streak, reading lists, and the recent feed — all in parallel so the page paints fast. Let's unpack how it works.

### Goal

Build a dashboard page that:

- Shows a greeting / login prompt when logged out.
- Fetches seven different data sources in parallel (using `Promise.allSettled`).
- Renders **two entirely different layouts** based on `uiMode` (`archive` or `modern`).
- Uses the `WorkBlurb` and `ArchiveButton` components in archive mode.

### Actions

Open the file and look at the script block:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import { getReadingStats, getUnreadCount, getStreak, listBookmarks, listReadingLists, getFeed } from '$lib/api/social';
  import type { Bookmark, ReadingList, FeedItem } from '$lib/api/social-types';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import { formatWords, relativeTime } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let loading = $state(true);
  let error = $state('');

  let readingStats = $state<{ total_words_read: number; total_works_read: number; login_streak: number } | null>(null);
  let recentBookmarks: Bookmark[] = $state([]);
  let unreadCount = $state(0);
  let streak = $state<{ current_streak: number; longest_streak: number } | null>(null);
  let recentActivity: FeedItem[] = $state([]);
  let myLists: ReadingList[] = $state([]);
```

**Key ideas:**

1. **State is `$state()`** — Svelte 5 runes make every variable here automatically reactive. When `loading` flips to `false`, the template updates.
2. **`$derived`** reads the `uiMode` preference once at mount. The `$derived` rune recomputes if `getPref('uiMode')` changes, so toggling archive mode mid-session instantly re-renders.
3. **Types are imported** from `$lib/api/social-types` so the compiler catches mismatches between what the frontend expects and what the backend returns.

Now look at the `loadOverview` function:

```typescript
  async function loadOverview() {
    loading = true;
    error = '';
    const uid = auth.user?.id ?? 0;
    if (!uid) { loading = false; return; }
    try {
      await Promise.allSettled([
        (async () => { const r = await getReadingStats(uid); if ((r as any)?.err === 0) readingStats = r as any; else if (r && !('err' in r)) readingStats = r as any; })(),
        (async () => { const r = await getUnreadCount(); if ((r as any)?.err === 0) unreadCount = (r as any).unread_count; })(),
        (async () => { const r = await getStreak(uid); if ((r as any)?.err === 0) streak = r as any; })(),
        (async () => { const r = await listBookmarks(); if ((r as any)?.err === 0) recentBookmarks = (r as any).bookmarks.slice(0, 4); })(),
        (async () => { const r = await listReadingLists(); if ((r as any)?.err === 0) myLists = (r as any).lists; })(),
        (async () => { const r = await getFeed(1, 10); if ((r as any)?.err === 0) recentActivity = (r as any).items; })(),
      ]);
    } catch {
      error = 'Network error loading dashboard.';
    } finally {
      loading = false;
    }
  }
```

**What's happening here:**

- **`Promise.allSettled`** — Instead of `Promise.all` (which fails fast if one request errors), `allSettled` lets every request finish independently. If the streak API is down, the reading stats and bookmarks still load. This is the dashboard philosophy: **show me what you can, tell me what you can't**.
- **Six requests in parallel** — All fire at once. The browser handles them concurrently over separate HTTP/2 streams.
- **Defensive guards** — Every callback checks `r.err === 0` before assigning. If the backend returns an error, the state stays at its default (empty array or `null`), and the template renders the empty-state branch instead of crashing.
- **`auth.user?.id ?? 0`** — If the user isn't logged in, there's no `uid`, so we bail early. The `onMount` handler calls `auth.init()` first, which fetches `/api/auth/me` to hydrate the auth store.

The `onMount` hook:

```typescript
  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    await loadOverview();
  });
```

`auth.init()` (see `src/lib/stores/auth.svelte.ts`) calls `getMe()`, which hits `GET /api/auth/me`. If there's a cached user in localStorage, it falls back to that so the reader can keep reading offline. Only if `auth.isLoggedIn` is false do we skip loading entirely.

#### The template: two UIs in one

**Archive mode** (lines 69–197) renders an AO3-style page:

```svelte
{#if !auth.isLoggedIn}
  <!-- login prompt -->
{:else if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Dashboard</h1>
        <p class="archive-summary">Your reading stats, bookmarks, collections, and activity — AO3-style overview.</p>
      </header>

      {#if loading}
        <!-- skeleton loaders -->
      {:else if error}
        <div class="archive-error"><p>{error}</p></div>
      {:else}
        <!-- Reading stats (AO3-style dl.stats) -->
        {#if readingStats}
          <blockquote class="archive-summary">
            <dl class="stats">
              <div class="stat"><dt>Works</dt><dd>{readingStats.total_works_read}</dd></div>
              <div class="stat"><dt>Words</dt><dd>{formatWords(readingStats.total_words_read)}</dd></div>
              <div class="stat"><dt>Login streak</dt><dd>{readingStats.login_streak} day{readingStats.login_streak !== 1 ? 's' : ''}</dd></div>
            </dl>
          </blockquote>
        {/if}
```

The `dl.stats` pattern mirrors AO3's `<dl class="stats">` meta block. `formatWords` (from `$lib/util`) adds thousands separators: `1,234,567`.

**Modern mode** (lines 199–217) is a minimalist stub:

```svelte
{:else}
  <div class="dashboard-page">
    <h1>Dashboard</h1>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else if error}
      <div class="error-card"><p>{error}</p></div>
    {:else}
      <p class="muted">Switch to archive mode for the full AO3-style dashboard.</p>
      {#if readingStats}
        <div class="stats-grid">
          <div class="stat-card"><span class="stat-value">{readingStats.total_works_read}</span><span class="stat-label">Works</span></div>
          <div class="stat-card"><span class="stat-value">{formatWords(readingStats.total_words_read)}</span><span class="stat-label">Words</span></div>
        </div>
      {/if}
    {/if}
  </div>
{/if}
```

### Breakdown

| Section | What it shows | API calls |
|---|---|---|
| Login gate | Redirect to `/login` if not signed in | `auth.init()` → `/api/auth/me` |
| Reading stats | Total words, total works, login streak | `GET /api/users/{id}/reading-stats` |
| Streak | Current + longest consecutive days | `GET /api/users/{id}/streak` |
| Recent bookmarks | Last 4 bookmarked works | `GET /api/bookmarks` |
| Collections | Reading lists with item counts | `GET /api/lists` |
| Activity feed | 10 most recent exported works | `GET /api/feed` |
| Unread count | Badge count for nav bar | `GET /api/notifications/unread-count` |
| Quick links | Shortcuts to Bookmarks, Collections, etc. | (static links) |

**`WorkBlurb`** is the AO3-style work card. It takes a `fic` object and renders the title, byline, symbol squares (rating/category/warnings), fandom tags, summary, and stats block — all sanitized via DOMPurify. In the dashboard, bookmark objects are adapted to the `FicSearchResult` shape `WorkBlurb` expects:

```typescript
<WorkBlurb fic={{\n  url_id: String(b.work_id),\n  title: '',\n  author: '',\n  words: 0,\n  chapters: 0,\n  source: 'bookmark',\n  status: '',\n  description: b.notes || '',\n  updated: b.created_at,\n  tags: []
}} />
```

**`ArchiveButton`** is a styled link or button that renders with AO3's gray-bordered, hover-highlighted `.archive-btn` CSS class. It's used for "All Bookmarks", "All Collections", and "Create one".

**CSS variables** drive the theme. Look at how `--archive-link`, `--archive-text`, `--archive-border`, and `--archive-bg-raised` are used throughout the `<style>` block. These are set by the archive skin system (see `src/lib/prefs.ts` → `setArchiveSkin`) so switching skins recolors the entire page without touching the Svelte markup.

### Check

1. Start the dev server:
   ```bash
   cd frontend && npm run dev
   ```
2. Log in, then visit `http://localhost:5173/dashboard`.
3. You should see your stats, bookmarks, and collections.
4. Add `?ui=archive` to the URL and refresh — the page should switch to the AO3-style layout instantly (the `applyUiParam` function in `$lib/prefs` reads the URL param on every navigation).
5. Click "All Bookmarks" — it should take you to `/bookmarks`.

### Troubleshooting

- **"Log in to see your dashboard"** — `auth.init()` hasn't resolved yet. Check that your dev server is running and that `GET /api/auth/me` returns `{"err": 0, "user": {...}}` with a valid token cookie.
- **All sections show "No bookmarks yet"** — The API is returning an empty array. Make sure you've bookmarked at least one work first by visiting a work page and clicking "Save".
- **Skeleton loader spins forever** — One of the `Promise.allSettled` callbacks threw. Open the browser console and check for CORS errors or 500 responses on any of the six API endpoints.
- **Modern mode shows a stub message** — That's by design! The dashboard in modern mode is intentionally minimal right now. Switch to archive mode (`?ui=archive`) for the full experience.

---

## 41.2 `src/routes/authors/[name]/+page.svelte` — Author Page

This is the "profile" half of Part 21's title. When you click an author's name from a bookmark, work page, or search result, you land here. The page shows the author's avatar, badge, bio, top tags, and a grid of their works. The page owner gets an "Edit Profile" button.

### Goal

Build an author profile page that:

- Loads an author by name from `GET /api/authors/by-name/{name}`.
- Shows avatar, badge text, badges, stats, and favorite tags.
- Lets the owner edit their bio, avatar URL, and badge text.
- Renders the bio as HTML in archive mode but strips HTML in modern mode.
- Lists the author's canonical works and orphan (unmerged) sources in a responsive grid.

### Actions

The script block starts with SvelteKit's `$props` — this route uses a URL parameter:

```typescript
let { data } = $props();
const authorName = $derived(decodeURIComponent(data.name));
```

The `+page.svelte` file sits at `src/routes/authors/[name]/+page.svelte`, so SvelteKit passes `data.name` from the URL segment. `decodeURIComponent` handles authors with spaces or special characters (e.g., `%20` → space).

The component fetches author data on mount:

```typescript
onMount(async () => {
  await auth.init();
  await load();
});

async function load() {
  loading = true;
  error = '';
  try {
    const res = await getAuthorByName(authorName);
    if (res.err !== 0) {
      error = 'Could not load author.';
      return;
    }
    author = res.author;
    works = res.works ?? [];
    orphans = res.orphans ?? [];
    isOwner = auth.isLoggedIn && author?.name === auth.username;
    if (author?.id) authorId = author.id;
  } catch {
    error = 'Network error loading author.';
  } finally {
    loading = false;
  }
}
```

The `getAuthorByName` function lives in `src/lib/api/series.ts`:

```typescript
export async function getAuthorByName(name: string): Promise<AuthorResponse> {
  return request<AuthorResponse>(`/authors/by-name/${encodeURIComponent(name)}`);
}
```

Notice the `encodeURIComponent` — author names with spaces or special characters get safely URL-encoded. The backend route is `GET /api/authors/by-name/{name}`.

**Owner detection** happens with a simple username comparison:

```typescript
isOwner = auth.isLoggedIn && author?.name === auth.username;
```

If the logged-in user's username matches the author name in the URL, `isOwner` becomes `true` and the edit form appears.

**The edit profile flow:**

```typescript
async function saveProfile() {
  if (!authorId) return;
  const updates: { avatar_url?: string; bio?: string; badge_text?: string } = {};
  if (editAvatar.trim()) updates.avatar_url = editAvatar;
  if (editBio.trim()) updates.bio = editBio;
  if (editBadge.trim()) updates.badge_text = editBadge;
  try {
    const res = await updateAuthorProfile(authorId, updates);
    if (res.err === 0) {
      editingProfile = false;
      await load();
    }
  } catch {
    error = 'Failed to save profile.';
  }
}
```

`updateAuthorProfile` calls `PUT /api/authors/{id}`:

```typescript
export async function updateAuthorProfile(
  authorId: number,
  updates: { avatar_url?: string; bio?: string; badge_text?: string },
): Promise<{ err: number; msg?: string }> {
  return request<{ err: number; msg?: string }>(`/authors/${authorId}`, {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}
```

Only fields that the owner actually changed are sent — if `editAvatar` is empty, `avatar_url` isn't included in the update at all.

**Bio rendering** is mode-dependent:

```typescript
function renderBio(): string {
  if (!author?.bio) return '';
  return isArchive ? author.bio : stripHtml(author.bio);
}
```

In **archive mode**, the bio is injected as raw HTML (`{@html author.bio}`) because AO3 author bios are already sanitized server-side and may contain `<p>`, `<a>`, and `<br>` tags. In **modern mode**, `stripHtml` from `$lib/util` removes all tags so a plain-text bio is safe.

The template renders:

- The avatar (with a fallback to AO3's default `avatar.gif`: `https://archiveofourown.org/images/cons/avatar.gif`)
- The badge text as a colored span
- Earned badges as emoji icons (with a `title` tooltip)
- A stats row showing work count and total words
- Top tags as clickable chips that link to `/search?q=...`
- Favorite tags in a separate row with the `--color-warning` color

### Breakdown

The `AuthorDetail` type (from `src/lib/api/series.ts`) is the contract between frontend and backend:

```typescript
export interface AuthorDetail {
  name: string;
  id?: number;
  avatar_url?: string | null;
  bio?: string | null;
  badge_text?: string | null;
  badges?: { badge_type: string; name: string; icon: string; earned_at: string }[];
  favorite_tags?: { name: string; tag_type_id: number }[];
  work_count: number;
  total_words: number;
  top_tags: { name: string; tag_type_id: number; usage_count: number }[];
}
```

Every optional field (`id?`, `avatar_url?`, `bio?`, etc.) is because the API may not always return every field — for instance, anonymous authors or authors pulled from an external source may lack an `id` or `badge_text`. The template guards each one with `{#if author.badge_text}`.

The **works grid** uses CSS `grid-template-columns: repeat(auto-fill, minmax(240px, 1fr))` so cards wrap responsively. Each work card is an `<a>` link with the title, a 160-character snippet (truncated with `…`), and metadata (words, chapters, status).

### Check

1. Visit `http://localhost:5173/authors/Test%20Author` (URL-encoded name).
2. You should see the author's profile, works grid, and (if you're logged in as that author) an "Edit Profile" button.
3. Click "Edit Profile", change your bio, and click "Save".
4. Refresh the page — your new bio should appear.
5. Add `?ui=archive` to see the bio rendered as HTML (AO3-style) instead of plain text.

### Troubleshooting

- **404 or "Could not load author"** — The author name in the URL must match exactly what the backend has. Check `/api/authors/by-name/{name}` directly with curl.
- **"Edit Profile" doesn't appear** — `isOwner` requires `auth.isLoggedIn && author.name === auth.username`. Make sure you're logged in as the exact author you're viewing.
- **Avatar shows broken image** — If `avatar_url` is empty or invalid, the `DEFAULT_AVATAR` fallback should kick in. Check that the `{#if author.avatar_url}` / `{:else}` branches are both present.
- **Bio is blank after saving** — The `saveProfile` function only sends non-empty fields. If you cleared the bio to empty, it isn't included in the PUT request. Add a check: `if (typeof editBio !== 'undefined')` instead of `editBio.trim()`.

---

## 41.3 `src/routes/updates/+page.svelte` — Updates Feed

The updates page is the notification center for followed works. When someone you follow publishes a new chapter or updates their story, it shows up here with a bright **NEW** badge. Opening the page marks everything as seen.

### Goal

Build an updates feed that:

- Fetches `GET /api/v1/updates` on mount.
- Shows each update with title, author, stats, and relative timestamp.
- Automatically marks all items as seen when the page loads (so the nav-bar counter drops).
- Renders both archive mode and modern mode.

### Actions

The script imports the key functions:

```typescript
import { getUpdates, markAllUpdatesSeen } from '$lib/api/social';
import type { FollowedWorkUpdate } from '$lib/api/social-types';
```

These live in `src/lib/api/social.ts` (lines 642–653):

```typescript
export async function getUpdates(): Promise<{
  err: number;
  items: import('./social-types').FollowedWorkUpdate[];
  unseen_count: number;
}> {
  return request('/v1/updates');
}

export async function markAllUpdatesSeen(items: { follow_id: number }[]): Promise<void> {
  await Promise.allSettled(items.map((it) => markFollowSeen(it.follow_id)));
}
```

The `getUpdates` call returns a list of `FollowedWorkUpdate` objects, each with:

```typescript
export interface FollowedWorkUpdate {
  follow_id: number;
  work_id: number;
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  fic_updated: string;
  updated_ago: string;
  is_new: boolean;
}
```

The `loadUpdates` function:

```typescript
async function loadUpdates() {
  loading = true; error = '';
  try {
    const res = await getUpdates();
    if (res.err === 0) {
      items = res.items;
      // Mark everything as seen once so the NEW badges clear (and the nav
      // count drops). Fire-and-forget: the feed itself is the "read" action.
      if (!seenMarked && items.length > 0) {
        seenMarked = true;
        markAllUpdatesSeen(items).catch(() => {});
      }
    } else {
      error = 'Failed to load updates.';
    }
  } catch {
    error = 'Network error loading updates.';
  } finally {
    loading = false;
  }
}
```

**The auto-mark-seen pattern:** When the updates page mounts, the component calls `markAllUpdatesSeen(items)`. This fires `markFollowSeen(follow_id)` for each item in parallel via `Promise.allSettled`. The backend's `markFollowSeen` calls `PUT /api/v1/follows/{id}/seen`, which sets `last_seen = now()` on the follow row. After that, the `is_new` flag for each item flips to `false`, and the nav-bar badge count (`getUnreadCount`) drops to zero.

The `seenMarked` boolean prevents double-calling — if `loadUpdates` runs again, we don't mark everything as seen a second time.

### The two UIs

**Archive mode** uses an AO3-style definition list (`<dl class="archive-dl updates-dl">`):

```svelte
<dl class="archive-dl updates-dl">
  {#each items as item (item.follow_id)}
    <div class="dl-row">
      <dt>
        {#if item.is_new}
          <span class="new-badge">NEW</span>
        {/if}
        <a class="archive-link update-title" href={`/works/${encodeURIComponent(item.url_id)}`}>{item.title}</a>
      </dt>
      <dd>
        <span class="archive-muted">by {item.author}</span>
        <span class="update-meta">
          {formatWords(item.words)} words · {item.chapters} chapters · {item.status} ·
          updated {relativeTime(item.fic_updated)}
        </span>
        <a class="archive-btn read-link" href={`/read/${encodeURIComponent(item.url_id)}`}>Read</a>
      </dd>
    </div>
  {/each}
</dl>
```

The **NEW badge** renders inline before the title link, styled red with white bold caps text (matching AO3's convention). The `read-link` is an AO3-style button-styled anchor (using `ArchiveButton`'s visual style) that links to the reader.

**Modern mode** is a card-based list:

```svelte
<div class="update-list">
  {#each items as item (item.follow_id)}
    <div class="card update-item">
      {#if item.is_new}
        <span class="new-badge">NEW</span>
      {/if}
      <a class="update-title" href={`/works/${encodeURIComponent(item.url_id)}`}>{item.title}</a>
      <span class="update-author muted">by {item.author}</span>
      <span class="update-meta muted">
        {formatWords(item.words)} words · {item.chapters} chapters · {item.status} ·
        updated {relativeTime(item.fic_updated)}
      </span>
      <a class="read-link" href={`/read/${encodeURIComponent(item.url_id)}`}>Read →</a>
    </div>
  {/each}
</div>
```

### Breakdown

| Piece | Detail |
|---|---|
| **Auth guard** | `onMount` checks `auth.isLoggedIn`; if not logged in, redirects to `/` via `goto`. |
| **Parallel marking** | `markAllUpdatesSeen` uses `Promise.allSettled` so a single failed `markFollowSeen` doesn't block the others. |
| **`is_new` flag** | The backend computes this by comparing `fic_updated > last_seen` (or `last_seen IS NULL`). |
| **`relativeTime`** | From `$lib/util`, formats "3 hours ago" / "2 days ago" for the `fic_updated` timestamp. |
| **`encodeURIComponent(item.url_id)`** | URL IDs can contain spaces or special characters; always encode before building hrefs. |

### Check

1. Follow at least one work (visit a work page and click the 🔔 follow button).
2. Visit `http://localhost:5173/updates`.
3. If the followed work has been updated since you followed it, you'll see a "NEW" badge.
4. Refresh — the badges should clear (because `markAllUpdatesSeen` ran on first load).
5. Toggle `?ui=archive` to see the AO3-style `<dl>` layout.

### Troubleshooting

- **No updates show** — You may not be following any works, or no followed works have new updates. Check the backend: `curl -H "Authorization: Bearer ***" http://localhost:8000/api/v1/updates`.
- **NEW badge doesn't clear** — The `markAllUpdatesSeen` call might be failing silently. Open DevTools → Network tab, watch the `POST /api/v1/follows/{id}/seen` requests. If they 401, your token has expired.
- **Redirects to home** — The page guards with `if (!auth.isLoggedIn) { goto('/'); return; }`. Make sure you're logged in.
- **Works don't link** — If `url_id` contains characters like `?` or `#`, they must be `encodeURIComponent`'d. The template does this correctly, but check your data if links break.

---

## 41.4 Frontend: Progression Widget, Recent Activity, and the Full Picture

This chapter zooms out to see how the dashboard pieces connect to the broader **progression system** (XP, levels, badges) and how the "Recent Activity" section ties into the social feed.

### The Progression Widget

FicHub has a gamification layer called **"F7"** (Features, Levels, Leaderboards). Every user has a site-wide level (0–100), experience points, and earned badges. The progression API client lives in `src/lib/api/features.ts`:

```typescript
// src/lib/api/features.ts (lines 21-39, 86-88)
export interface ProgressionInfo {
  level: number;
  rank: number;
  rank_title: string;
  xp: number;
  xp_to_next_level: number;
  recent_events: {
    event_type: string;
    xp: number;
    created_at: string;
  }[];
}

export async function fetchProgression(): Promise<ProgressionInfo> {
  return request<ProgressionInfo>('/me/progression');
}
```

The `fetchProgression` function calls `GET /api/me/progression`, which returns the user's current level, XP, XP needed for the next level, rank, rank title (like "Curator Apprentice" or "Archivist Sage"), and a list of recent XP-earning events.

**The progression widget** (shown in the Settings page, not the dashboard yet) would look like this:

```svelte
<script lang="ts">
  import { fetchProgression } from '$lib/api/features';
  let prog: ProgressionInfo | null = $state(null);
  let pct = $derived(prog ? (prog.xp / (prog.xp + prog.xp_to_next_level)) * 100 : 0);

  onMount(async () => {
    prog = await fetchProgression();
  });
</script>

{#if prog}
  <section class="progression-widget">
    <div class="level-info">
      <span class="level-badge">Level {prog.level}</span>
      <span class="rank-title">{prog.rank_title}</span>
      <span class="rank"># {prog.rank}</span>
    </div>
    <div class="xp-bar" role="progressbar" aria-valuenow={pct} aria-valuemin="0" aria-valuemax="100">
      <div class="xp-fill" style="width: {pct}%"></div>
    </div>
    <p class="xp-text">{prog.xp} EXP · {prog.xp_to_next_level} to next level</p>
    {#if prog.recent_events.length > 0}
      <ul class="event-list">
        {#each prog.recent_events as ev (ev.event_type + ev.created_at)}
          <li>{ev.event_type}: +{ev.xp} EXP</li>
        {/each}
      </ul>
    {/if}
  </section>
{/if}
```

The progress percentage is calculated as `xp / (xp + xp_to_next_level)`. When you hit the next level, `xp_to_next_level` resets to a higher threshold (a classic XP curve, usually doubling each level).

### Recent Activity Feed

The dashboard's "Recent Activity" section reuses the **social feed**:

```typescript
// In loadOverview():
(async () => { const r = await getFeed(1, 10); if ((r as any)?.err === 0) recentActivity = (r as any).items; })(),
```

`getFeed` calls `GET /api/feed?page=1&per_page=10` and returns `FeedItem` objects:

```typescript
export interface FeedItem {
  work_id: number;
  url_id: string;
  title: string;
  author: string;
  updated_at: string;
  format: string;
  url: string;
}
```

These are the 10 most recently exported (downloaded/cached) works from authors you follow. The template renders them as a simple list:

```svelte
{#each recentActivity as item (item.work_id)}
  <li class="archive-blurb">
    <span class="archive-blurb-title">
      <a class="archive-link" href={`/works/${item.work_id}`}>{item.title}</a>
    </span>
    <span class="archive-meta"> by {item.author} · {relativeTime(item.updated_at)}</span>
  </li>
{/each}
```

The difference between **feed** and **updates**:

- **Feed** (`GET /api/feed`): Most recent *export* per followed work. Shows "someone exported this fic."
- **Updates** (`GET /api/v1/updates`): Works where the *source fic* was updated (new chapter). Shows "this fic has new content."

### The Auth Store: Your Session Passport

Both the dashboard and author page call `auth.init()` from `src/lib/stores/auth.svelte.ts`. This store is the single source of truth for whether you're logged in:

```typescript
export const auth = new AuthStore();
```

The `AuthStore` class has:

- `user` — the current `User` object (or `null`)
- `isLoggedIn` — derived from `user !== null`
- `username` — convenience getter
- `level` — your site-wide level (0–100), with legacy `role` fallback
- `exp` — your XP total
- `init()` — calls `getMe()` to hydrate from the token

The `roleToLevel` helper maps old role numbers to the new level scale:

```typescript
export function roleToLevel(role: number | undefined): number {
  if (!role) return 0;
  if (role >= 10) return 100;  // admin → level 100
  if (role >= 5) return 50;    // curator → level 50
  if (role >= 1) return 1;     // trusted → level 1
  return 0;                    // anon/guest
}
```

This backward-compatibility shim means old browser sessions with cached `role` data still work for feature gates (curator-only features require level ≥ 50).

### The `uiMode` Switch: Two Sites in One

Every page that checks `getPref('uiMode')` is participating in FicHub's "archive mode" feature. The preference is stored in localStorage and can also be set via the `?ui=archive` URL parameter:

```typescript
// src/lib/prefs.ts (lines 91-128)
export function applyUiParam(url: URL): void {
  const ui = url.searchParams.get('ui');
  if (ui === 'archive' || ui === 'modern') {
    setPref('uiMode', ui);
  }
}
```

The root layout calls `applyUiParam` on every navigation, so users can deep-link to archive mode. The CSS uses custom properties (`--archive-link`, `--archive-text`, `--archive-border`, `--archive-bg`) so the archive skin system can swap palettes without reloading.

### Check

1. Visit `http://localhost:5173/dashboard?ui=archive`.
2. You should see the AO3-style dashboard with `<dl class="stats">` blocks.
3. Visit `http://localhost:5173/dashboard?ui=modern` — you'll see the minimal modern stub.
4. In archive mode, verify the CSS variables are active: inspect the page and check that `.archive-link` has `color: var(--archive-link, #990000)`.
5. Open DevTools, go to the Application tab, find `localStorage['fichub_prefs_v1']`, and confirm `uiMode` is set to your choice.

### Troubleshooting

- **Dashboard shows "Switch to archive mode" in modern mode** — This is intentional. The modern dashboard UI isn't fully built yet; archive mode has the complete feature set. To see all dashboard sections, use `?ui=archive`.
- **Progression widget shows "Could not load your level"** — You must be logged in. The `/me/progression` endpoint requires authentication.
- **`auth.init()` hangs** — Check that `GET /api/auth/me` returns a valid response. If your token expired, the `request()` helper in `social.ts` auto-redirects to `/login` on HTTP 401:

```typescript
if (res.status === 401) {
  setToken(null);
  try { localStorage.removeItem('fichub_cached_user'); } catch { /* ignore */ }
  if (typeof location !== 'undefined' && !location.pathname.startsWith('/login')) {
    location.href = '/login';
  }
  const text = await res.text().catch(() => '');
  throw new Error(`API error 401: ${text}`);
}
```

- **Recent Activity is empty** — The social feed (`GET /api/feed`) only shows works *you follow*. Follow a work first (Part 13), then the dashboard will populate.
- **CSS variables aren't styling** — Make sure the root layout sets them. They're defined in the global CSS, not in component-scoped `<style>` blocks. The fallback values (`#990000`, `#2a2a2a`, etc.) kick in if the variable isn't defined.

---

## What you have now

- **Dashboard page** (`/dashboard`): Fetches 7 data sources in parallel, renders both archive and modern UIs, with login gating, skeleton loaders, and error handling.
- **Author profile page** (`/authors/[name]`): Loads author by name, shows avatar/badge/bio/works, supports owner-only profile editing, and renders bio as HTML (archive) or plain text (modern).
- **Updates feed** (`/updates`): Lists followed-work updates with NEW badges, auto-marks all as seen on open, and renders both UI modes.
- **Progression system**: The `ProgressionInfo` type and `fetchProgression` API client (in `features.ts`) ready for a widget showing XP, level, rank, and recent events.
- **Auth store** (`auth.svelte.ts`): Single source of truth for login state, with role→level backward compatibility and offline caching.
- **Style system**: `uiMode` preference drives the archive/modern split, with CSS custom properties for theming.

---

On to [Part 42 — Frontend Forum and Curator](./42-frontend-forum-curator/19-frontend-forum-curator.md)
