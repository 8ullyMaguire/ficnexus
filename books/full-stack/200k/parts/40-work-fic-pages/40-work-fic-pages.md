# Part 40 — Frontend Work and Fic Pages

> In this chapter you will learn how FicHub's work detail page works — the dual UI mode (archive + modern), the social widgets (bookmark, rating, kudos, follow), the source selector, and the download format dropdown. We'll build the StarRating widget, the /fic/[urlId] redirect, and the WorkBlurb reusable list item.

---

## Overview

FicHub's work page (`/work/[workId]/+page.svelte`, 938 lines) supports **two UI modes** — archive (AO3-style `<dl>` definition lists with `<dt>`/`<dd>` rows) and modern (card-based layout). Both share the same data: a `Work` object with sources, bookmarks, ratings, kudos, follows, and comments.

The `/fic/[urlId]` route is a 9-line redirect — it bounces to `/work/[workId]` via SvelteKit's `+page.ts` `redirect(308)`. The real work happens on the work page.

---

## Chapter 40.1 — The StarRating Widget

### Goal

Build a 5-star click-to-rate widget with hover preview, ARIA accessibility, and CSS animations.

### Actions

```svelte
<!-- frontend/src/lib/components/StarRating.svelte (83 lines) -->
<script lang="ts">
    export let value = $bindable(0);       // 0 = nothing selected, 1..=5 = rating
    export let readonly = false;
    export let size = '1.3rem';
    export let onrate: (rating: number) => void;

    let hoverValue = $state(0);  // transient preview
    const displayValue = $derived(hoverValue > 0 ? hoverValue : value);

    function setHover(n: number) { if (!readonly) hoverValue = n; }
    function clearHover() { hoverValue = 0; }
    function click(n: number) {
        if (readonly) return;
        value = n;          // two-way bindable prop
        onrate?.(n);       // parent's POST handler
    }
</script>

<div class="star-rating" role="group" aria-label="Star rating">
    {#each [1, 2, 3, 4, 5] as n (n)}
        <button
            type="button"
            class="star"
            class:filled={n <= displayValue}
            class:active={n <= value}
            aria-label={readonly ? `${n} star${n > 1 ? 's' : ''}` : `Rate ${n} star${n > 1 ? 's' : ''}`}
            aria-pressed={n <= value}
            disabled={readonly}
            onmouseenter={() => setHover(n)}
            onmouseleave={clearHover}
            onclick={() => click(n)}
            style={`font-size: ${size}`}
        >{n <= displayValue ? '★' : '☆'}</button>
    {/each}
</div>
```

> **💡 Key Concept**: `displayValue` uses `$derived` — it shows the hover preview if hovering (`hoverValue > 0`), otherwise falls back to the bound `value`. This gives the familiar "drag to preview" UX where moving your mouse over stars 1-4 shows 4 filled stars before you click.

#### CSS

```css
.star-rating {
    display: inline-flex;
    gap: 0.15rem;
    align-items: center;
}
.star {
    background: none;
    border: none;
    padding: 0 0.05rem;
    cursor: pointer;
    line-height: 1;
    color: var(--color-muted);     /* unfilled: gray ☆ */
    transition: color 0.12s, transform 0.12s;
}
.star:hover:not(:disabled) { transform: scale(1.15); }  /* pop on hover */
.star.filled { color: #f5b301; }    /* filled: warm gold ★ */
.star.active { color: #f5b301; }    /* clicked: same gold (persist) */
.star:disabled { cursor: default; } /* readonly: no pointer cursor */
```

### Try It Yourself

```svelte
<!-- Add a numeric rating label that updates as you hover -->
<span class="rating-value">{hoverValue || value}</span>
```

### Check

- ✅ Hover preview via `displayValue = $derived(hoverValue > 0 ? hoverValue : value)`.
- ✅ Click emits `onrate(n)` — the POST is handled by the parent (`work/[workId]/+page.svelte`).
- ✅ ARIA: `role="group"`, `aria-label="Star rating"`, `aria-pressed`, `aria-label` per star.
- ✅ Readonly mode: hover/click ignored, `cursor: default`.
- ✅ CSS `@star:hover` transforms to scale(1.15) — tactile feedback.

### What you built

A 5-star rating widget with live hover preview, click-to-rate via a callback prop, full ARIA accessibility, CSS hover animations, and two-way bindable value.

---

## Chapter 40.2 — The Work Page Data Model

### Goal

Understand the `Work` type and how the page loads all related data in parallel.

### Actions

#### 1. Loading work data

```typescript
// frontend/src/lib/api/social.ts (lines 300–1086)
// The API client exports functions for every social endpoint:
export async function getWork(workId: number): Promise<{ err: number; work: Work }> {
    const res = await fetch(`/api/works/${workId}`);
    return (await res.json()) as { err: number; work: Work };
}
export async function addBookmark(workId: number, notes?: string, private_?: boolean) { ... }
export async function removeBookmark(workId: number) { ... }
export async function listBookmarks() { ... }  // returns { bookmarks: [] }
export async function rateWork(workId: number, rating: 1 | 2 | 3 | 4 | 5) { ... }
export async function getRatings(workId: number) { ... }  // avg, count
export async function giveKudos(workId: number) { ... }
export async function removeKudos(workId: number) { ... }
export async function getKudos(workId: number) { ... }
export async function follow(targetType: string, targetId: number) { ... }
export async function unfollow(followId: number) { ... }
export async function checkWorkFollow(workId: number) { ... }

// The Work type (from social-types.ts):
export interface Work {
    id: number;
    url_id: string;
    canonical_title: string;
    canonical_author: string;
    description: string;
    words: number;
    chapters: number;
    status: string;
    rating: string;            // content rating (Not Rated, Teen, etc.)
    sources: WorkSource[];
    total_bookmarks: number;
    total_comments: number;
    total_kudos: number;
}

export interface WorkSource {
    id: number;
    source: string;    // full URL (e.g. https://archiveofourown.org/works/123)
    source_domain: string;
    words: number;
    chapters: number;
    url_id: string;
}
```

#### 2. Parallel data loading

```typescript
// /work/[workId]/+page.svelte (lines 65–91)
onMount(async () => {
    await auth.init();           // ensure JWT is loaded before any API calls
    await loadWork();
});

async function loadWork() {
    loading = true; error = '';
    try {
        const res = await getWork(workId);
        if (res.err !== 0) { error = 'Could not load work.'; return; }
        work = res.work;

        if (work.sources && work.sources.length > 0) {
            selectedSource = work.sources[0];  // default to first source
        }

        // Load secondary data in parallel — bookmarks, ratings, kudos, follow status
        await Promise.allSettled([
            loadBookmarkStatus(),
            loadRatings(),
            loadKudos(),
            loadFollowStatus(),
        ]);
    } catch {
        error = 'Network error loading work.';
    } finally {
        loading = false;
    }
}
```

> **💡 Key Concept**: `Promise.allSettled()` is used instead of `Promise.all()` because social data loads are **best-effort** — if kudos fails to load (e.g. DB hiccup), the work page still renders. Each loader catches its own errors independently.

#### 3. Two-way binding for state

```typescript
// Svelte 5 $state for reactive local state
let isBookmarked = $state(false);
let myRating = $state(0);
let kudosCount = $state(0);
let isFollowingWork = $state(false);

// StarRating uses $bindable for two-way binding:
<StarRating bind:value={myRating} onrate={handleRate} />
// When the user clicks a star, StarRating sets `value` (→ myRating updates),
// and calls onrate(n) which triggers the POST to the backend.
```

### Try It Yourself

```typescript
// Add a "last read" timestamp to the Work type
// 1. Add to social-types.ts interface Work
// 2. Add GET /api/bookmarks/:work_id/last-read endpoint
// 3. Call it in loadWork() inside Promise.allSettled
```

### Check

- ✅ `loadWork()` calls `auth.init()` first (no unauthenticated API calls).
- ✅ `Promise.allSettled` — social data failure doesn't block the work page.
- ✅ `selectedSource` defaults to `work.sources[0]`.
- ✅ `$bindable(0)` enables `<StarRating bind:value={myRating}>`.
- ✅ `getDownloadUrl` builds `/api/epub?q=<url>&format=<fmt>` links.

### What you built

The work page's data model — the `Work` and `WorkSource` types, the parallel loading pattern with `Promise.allSettled`, two-way binding for reactive state, and the API client functions for every social interaction.

---

## Chapter 40.3 — Dual UI Mode (Archive + Modern)

### Goal

Understand how the work page renders in two completely different UIs from the same data, controlled by `uiMode`.

### Actions

#### 1. Mode detection

```svelte
<script lang="ts">
    // uiMode is read from localStorage prefs (default: 'archive')
    const uiMode = $derived(getPref('uiMode'));
    // getPref reads 'fichub_prefs_v1' from localStorage
    // UI_MODE_KEY in prefs.ts defines: archive = AO3-style, modern = card-based
</script>
```

#### 2. Archive mode — AO3-style `<dl>` layout

```svelte
<!-- Archive mode uses <dl class="archive-dl"> with <dt>/<dd> rows -->
{#if uiMode === 'archive'}
<main class="archive-main">
    <div class="archive-content">
        <header class="archive-header">
            {#if loading}
                <p class="archive-muted"><span class="spinner"></span> {t('work.loading')}</p>
            {:else if error}
                <h1 class="archive-page-title">Work</h1>
                <p class="archive-error">⚠️ {error}</p>
            {:else if work}
                <h1 class="archive-page-title">{work.canonical_title}</h1>
                <p class="archive-byline">
                    by <a href={`/authors/${encodeURIComponent(work.canonical_author)}`}>
                        {work.canonical_author}
                    </a>
                </p>
            {/if}
        </header>

        {#if !loading && !error && work}
            <!-- Source selector as radio fieldset -->
            <fieldset class="archive-fieldset">
                <legend class="archive-legend">{t('work.selectSource')}</legend>
                <dl class="archive-dl">
                    {#each work.sources as source}
                        <div class="dl-row">
                            <dt>
                                <label class="archive-radio">
                                    <input type="radio" name="source"
                                        checked={selectedSource?.id === source.id}
                                        onchange={() => selectedSource = source} />
                                    {detectSite(source.source)}
                                </label>
                            </dt>
                            <dd>{formatWords(source.words)} · {source.chapters} ch</dd>
                        </div>
                    {/each}
                </dl>
            </fieldset>

            <!-- Download formats as buttons -->
            <fieldset class="archive-fieldset">
                <legend class="archive-legend">Download</legend>
                <div class="archive-actions">
                    {#each downloads as d}
                        <a class="archive-btn" href={getDownloadUrl(selectedSource, d.type)} download>
                            {d.label}
                        </a>
                    {/each}
                </div>
            </fieldset>
        {/if}
    </div>
</main>
{:else}
<!-- Modern mode (card-based, lines 408–938) -->
...
{/if}
```

#### 3. Download format list

```typescript
// Formats are derived from the selected source URL
let downloads = $derived.by(() => {
    if (!selectedSource) return [];
    // The source URL is used for export — formats are fetched via /api/epub?q=<url>
    return [
        { label: 'EPUB', type: 'epub' },
        { label: 'HTML', type: 'html' },
        { label: 'TXT', type: 'txt' },
        { label: 'MD', type: 'md' },
        { label: 'MOBI', type: 'mobi' },   // requires Calibre on server
        { label: 'PDF', type: 'pdf' },    // requires Calibre
        { label: 'AZW3', type: 'azw3' },  // requires Calibre
    ];
});
```

#### 4. The /fic/[urlId] redirect

```svelte
<!-- frontend/src/routes/fic/[urlId]/+page.svelte (9 lines) -->
<script lang="ts">
    // /fic/[urlId] redirects to /works/[urlId] via +page.ts (308). This
    // component only renders while the redirect is in flight.
</script>

<svelte:head><title>Redirecting…</title></svelte:head>
<main class="archive-main">
    <p>Redirecting to /works/…</p>
</main>
```

```typescript
// frontend/src/routes/fic/[urlId]/+page.ts
import { redirect } from '@sveltejs/kit';
export function load({ params }) {
    throw redirect(308, `/works/${params.urlId}`);  // 308 = permanent redirect
}
```

### Try It Yourself

```svelte
<!-- Add a "Related Works" section in archive mode -->
{#if work.related_works}
    <fieldset class="archive-fieldset">
        <legend class="archive-legend">Related</legend>
        <dl class="archive-dl">
            {#each work.related_works as rel}
                <div class="dl-row">
                    <dt><a class="archive-link" href={`/works/${rel.id}`}>{rel.title}</a></dt>
                </div>
            {/each}
        </dl>
    </fieldset>
{/if}
```

### Check

- ✅ `uiMode` is read from localStorage via `getPref('uiMode')`.
- ✅ Archive mode uses `<dl class="archive-dl">` with `<dt>`/`<dd>` rows (AO3-style).
- ✅ Modern mode uses `<div class="card">` and `<div class="actions-row">`.
- ✅ Download formats include EPUB, HTML, TXT, MD, MOBI, PDF, AZW3.
- ✅ `/fic/[urlId]` redirects 308 to `/works/[urlId]` in `+page.ts`.
- ✅ `detectSite()` converts a source URL to a human-readable site name.

### What you built

The dual UI architecture — the same `Work` data rendered through either an AO3-style `<dl>` definition list (archive mode) or card-based layout (modern mode), with the source selector, download format dropdown, and the 9-line `/fic/[urlId]` redirect stub.

---

## Chapter 40.4 — Social Widgets Integration

### Goal

Understand how the work page wires up bookmarks, kudos, follows, and comments as inline widgets.

### Actions

#### 1. Bookmark widget with notes

```svelte
<!-- In the work page, bookmark toggle button -->
<button class="archive-btn archive-btn-primary"
    onclick={toggleBookmark} disabled={savingBookmark}>
    {#if savingBookmark}
        <span class="spinner"></span>
    {:else if isBookmarked}
        {t('work.saved')}
    {:else}
        {t('work.save')}  <!-- terse "Save" -->
    {/if}
</button>

{#if !isBookmarked}
    <input type="text" class="archive-input"
        placeholder="Add a note…" bind:value={bookmarkNotes} />
    <label class="archive-checkbox-label">
        <input type="checkbox" bind:checked={bookmarkPrivate} />
        Private
    </label>
{/if}
```

#### 2. Kudos — AO3 anonymous-appreciable likes

```svelte
<button class="archive-btn"
    onclick={handleKudos} disabled={kudosLoading}
    aria-label={myKudos ? t('work.kudosRemove') : t('work.kudos')}>
    {#if kudosLoading}
        <span class="spinner"></span>
    {:else if myKudos}
        {t('work.kudosGiven')}  <!-- "Kudos Given" after clicking -->
    {:else}
        {t('work.kudos')}       <!-- terse "Kudos" -->
    {/if}
</button>

{#if kudosCount > 0 || guestCount > 0}
    <span class="rating-count">{t('work.kudosCount', { count: kudosCount })}</span>
    {#if guestCount > 0}
        <span class="archive-muted">{t('work.kudosGuests', { count: guestCount })}</span>
    {/if}
{/if}
```

> **💡 Key Concept**: Kudos supports **guest appreciation** — anonymous visitors can leave kudos (counted as `guest_count`). Logged-in users can give + remove. The count shows both: "12 kudos (3 guests)".

#### 3. Follow (subscribe to updates)

```svelte
<button class="archive-btn"
    onclick={toggleFollow} disabled={followLoading}
    aria-label={isFollowingWork ? 'Unsubscribe' : 'Subscribe to updates'}>
    {#if followLoading}
        <span class="spinner"></span>
    {:else if isFollowingWork}
        Subscribed
    {:else}
        Subscribe
    {/if}
</button>
```

#### 4. Stats links

```svelte
<dl class="archive-dl archive-stats-dl">
    <div class="dl-row">
        <dt>Stats</dt>
        <dd class="archive-stats-links">
            <a class="archive-link" href={`/work/${workId}/stats`}>
                {work.total_bookmarks} {t('work.bookmarks')}
            </a>
            <a class="archive-link" href={`/work/${workId}/stats`}>
                {avgRating > 0 ? `${avgRating.toFixed(1)}★` : '—'} {t('work.rating')}
            </a>
            <a class="archive-link" href={`/work/${workId}/stats`}>
                {work.total_comments} {t('work.comments')}
            </a>
            {#if kudosCount > 0 || guestCount > 0}
                <a class="archive-link" href={`/work/${workId}/stats`}>
                    {kudosCount + guestCount} {t('work.kudos')}
                </a>
            {/if}
        </dd>
    </div>
</dl>
```

#### 5. Comments and Reviews sections

```svelte
<!-- CommentSection and ReviewsSection are separate reusable components -->
<CommentSection workId={workId} />
<ReviewsSection workId={workId} />
```

> **⚠️ Watch Out**: The work page checks `auth.isLoggedIn` before rendering interactive widgets. Anonymous visitors see a "login to rate" link instead. This check happens at the template level (`{#if auth.isLoggedIn}`) — the data loaders (`loadBookmarkStatus`, `loadFollowStatus`) also early-return for anonymous users.

### Try It Yourself

```typescript
// Add a "reading status" dropdown to the actions
let readingStatus = $state<'unread' | 'reading' | 'read'>('unread');
// POST /api/reading/status when changed
```

### Check

- ✅ Bookmark button shows "Saved" when active, spinner while saving.
- ✅ Kudos has guest_count support (anonymous can kudo).
- ✅ Follow toggle shows "Subscribed" vs "Subscribe".
- ✅ All interactive widgets gated behind `auth.isLoggedIn`.
- ✅ Stats link to `/work/{workId}/stats` for detail page.
- ✅ Comments and Reviews are separate reusable components.

### What you built

The social widget integration — bookmark toggle with notes + private flag, AO3-style kudos with guest appreciation, follow/unsubscribe toggle, stats link cluster, and the auth-gated comment + review sections.

---

## Conclusion

You now understand FicHub's work/fic frontend:

1. **StarRating** — 5-star widget with hover preview, ARIA, CSS animations, `$bindable` two-way binding.
2. **Work page** — loads `Work` + sources + bookmarks + ratings + kudos + follows in parallel with `Promise.allSettled`.
3. **Dual UI** — AO3-style archive mode (`<dl>` definition lists) and card-based modern mode, switched via `uiMode` pref.
4. **Download formats** — EPUB, HTML, TXT, MD (built-in) + MOBI, PDF, AZW3 (via Calibre).
5. **Social widgets** — bookmark (notes + private), kudos (with guest count), follow (subscribe), stats link cluster, auth-gated comment + review sections.
6. **`/fic/[urlId]`** — 9-line redirect to `/works/[urlId]` via SvelteKit `redirect(308)`.

In Part 21 we'll build the dashboard and profile pages.
