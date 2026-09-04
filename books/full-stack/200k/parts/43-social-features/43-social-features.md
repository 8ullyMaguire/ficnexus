# Part 43 — Frontend Social Features

> In this chapter you will learn how FicHub's social features work on the frontend — the follows page with exclusion management, the Ask the Archive page with natural-language search, and the unified search experience.

---

## Overview

FicHub's social frontend lives in `frontend/src/routes/` and includes:

- **Follows page** (`/follows`) — lists all user's follows (users, works, authors) with per-author exclusion controls.
- **Ask page** (`/ask`) — natural-language search with Ollama, auto-persisting to fic requests, archivist LLM answers, and human-answers section.
- **Search page** (`/search`) — three-tier UI (Simple, Guided, Power) with tag autocomplete and URL sync.

All pages support dual UI mode — archive (AO3-style `<dl>` layout) and modern (cards) — controlled by the `uiMode` pref.

---

## Chapter 43.1 — The Follows Page

### Goal

Understand how the follows page displays different follow types (users, works, authors) with per-author exclusion controls, in both archive and modern UI modes.

### Actions

#### 1. The Follow type

```typescript
// frontend/src/lib/api/social-types.ts
export interface Follow {
    id: number;           // follow row ID
    user_id: number;     // who is following
    follow_type: 'user' | 'work' | 'author';
    followee_id?: number;  // for user follows
    work_id?: number;      // for work follows
    author_name?: string;  // for author follows (string-based)
    created_at: string;
}

export interface FollowExclusion {
    id: number;
    follow_id: number;
    exclude_type: 'work' | 'series' | 'fandom';
    work_id?: number;
    series_id?: number;
    fandom?: string;
}
```

#### 2. The follows page script

```typescript
// frontend/src/routes/follows/+page.svelte (lines 1–55)
import { listFollows, unfollow } from '$lib/api/social';
import { t } from '$lib/i18n/index.svelte';
import Exclusions from './Exclusions.svelte';

const uiMode = $derived(getPref('uiMode'));

let follows = $state<Follow[]>([]);

let loading = $state(true);
let error = $state('');

onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadFollows();
});

async function loadFollows() {
    loading = true; error = '';
    try {
        const res = await listFollows();
        if (res.err === 0) follows = res.follows;
        else error = 'Failed to load follows.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
}

async function handleUnfollow(id: number) {
    await unfollow(id);
    follows = follows.filter(f => f.id !== id);
}
```

> **💡 Key Concept**: The follows page uses **optimistic UI** — when you click unfollow, the row disappears immediately without waiting for the API response.

#### 3. Rendering different follow types

```typescript
function followLabel(f: Follow): string {
    if (f.followee_id) return `User #${f.followee_id}`;
    if (f.work_id) return `Work #${f.work_id}`;
    if (f.author_name) return `Author: ${f.author_name}`;
    return t('follows.unknown');
}

function isAuthorFollow(f: Follow): boolean {
    return !!f.author_name;
}
```

#### 4. Archive mode

```svelte
{#if uiMode === 'archive'}
<dl class="archive-dl follows-dl">
    {#each follows as f (f.id)}
        <div class="dl-row">
            <dt>{followLabel(f)}</dt>
            <dd>
                <button class="archive-btn archive-unfollow"
                    onclick={() => handleUnfollow(f.id)}>
                    {t('follows.unfollow')}
                </button>
                {#if isAuthorFollow(f)}
                    <div class="archive-exclusions"><Exclusions follow={f} /></div>
                {/if}
            </dd>
        </div>
    {/each}
</dl>
{/if}
```

### Check

- ✅ `listFollows()` guarded by `auth.isLoggedIn` — anonymous users redirect to `/`.
- ✅ `handleUnfollow` uses optimistic UI (filter immediately).
- ✅ Only author follows (`!!f.author_name`) render exclusion controls.
- ✅ Archive mode uses `<dl class="archive-dl">` with `<dt>`/`<dd>` rows.
- ✅ Follow labels distinguish users (#ID), works (#ID), and authors (name).

---

## Chapter 43.2 — The Ask Page

### Goal

Understand the Ask page's natural-language search with Ollama, auto-persist to community requests, and the "interpreted as" chip.

### Actions

#### 1. The ask API client

```typescript
// frontend/src/lib/api/ask.ts (51 lines)
export interface AskResponse {
    total: number;
    results: SearchResult[];
    translated: boolean;      // true when Ollama produced a v2 query
    nl_query: string;         // original NL request
    applied_params: string;   // v2 query actually run
}

export async function askArchive(nlQuery: string): Promise<AskResponse> {
    const res = await fetch('/api/search/ask', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ q: nlQuery }),
    });
    if (!res.ok) throw new Error(`Ask failed (${res.status})`);
    return (await res.json()) as AskResponse;
}
```

#### 2. Auto-persist flow

```typescript
// frontend/src/routes/ask/+page.svelte
let result: AskResponse | null = $state(null);
let persistedRequestId: number | null = $state(null);

async function persistAsRequest() {
    if (!result || persistedRequestId) return;
    const nl = result.nl_query || query.trim();
    try {
        const res = await createRequest({ title: nl, body: '' });
        if (res.err === 0 && res.id) {
            persistedRequestId = res.id;
            // URL becomes ?q=request:{id} — bookmarkable, back-button-safe
            goto(`?q=request:${res.id}`, { replaceState: true });
        }
    } catch { /* non-fatal */ }
}

// Auto-persist after successful ask
$effect(() => {
    if (searched && result && !persisted) {
        void persistAsRequest();
    }
});
```

> **💡 Key Concept**: The ask page auto-persists every successful ask as a community request. The LLM's translated search query is stored as an `"llm"` answer row, and community members can add human answers below it.

#### 3. Interpreted-as chip

```svelte
{#if result}
    <div class="interpreted-row">
        <span class="chip interpreted-chip">
            <span class="chip-label">Interpreted as:</span> {result.applied_params}
        </span>
        {#if !result.translated}
            <span class="chip plain-chip">
                plain search (no translation)
            </span>
        {/if}
    </div>
{/if}
```

> **⚠️ Watch Out**: When `translated === false`, Ollama was unavailable and the backend fell back to a plain keyword search on the raw NL query. The "plain search (no translation)" chip tells the user this isn't a true semantic search.

#### 4. Empty results → Turn into a request

```svelte
{#if result.total === 0}
    <div class="request-card">
        <p>No results found. Turn this ask into a fic request.</p>
        <a href={`/requests/new?q=${encodeURIComponent(query.trim())}`}>
            Turn this into a request
        </a>
    </div>
{/if}
```

### Check

- ✅ `askArchive()` sends `POST /api/search/ask` with `{ q: nlQuery }`.
- ✅ `translated: false` means Ollama was unavailable — plain keyword fallback.
- ✅ When `total === 0`, a "Turn into a request" link is shown.
- ✅ `$effect` auto-persists after successful ask.
- ✅ URL becomes `?q=request:{id}` — bookmarkable, back-button-safe.

### What you built

The Ask page flow — LLM translation with plain-keyword fallback, auto-persist to community requests, "interpreted as" chip, and empty-result → request conversion.

---

## Chapter 43.3 — Dual UI Mode

### Goal

Understand how all social pages render in both archive (AO3-style) and modern modes.

### Actions

Every social frontend page reads `uiMode` the same way:

```typescript
const uiMode = $derived(getPref('uiMode'));
```

```svelte
{#if uiMode === 'archive'}
    <!-- AO3-style: <dl> layout, serif font, red accent ArchiveButton -->
    <main class="archive-main">
        <header class="archive-header">...</header>
        <dl class="archive-dl">
            <div class="dl-row">
                <dt>Label</dt>
                <dd>Content</dd>
            </div>
        </dl>
    </main>
{:else}
    <!-- Modern: card-based, sans-serif, colored buttons -->
    <div class="card">...</div>
{/if}
```

Key archive-mode components:
- `ArchiveButton` — red-bordered button matching AO3 style (`<ArchiveButton>`)
- `ArchiveButton` uses class `.archive-btn` with `:hover` red border
- `ArchiveHeader` — consistent `<header class="archive-header">` with `<h1 class="archive-page-title">`
- `WorkBlurb` — reusable result card (shared across search, ask, bookmarks)
- Skeleton loaders: `<div class="archive-skeleton-list">` with `<article class="skeleton-blurb">`

> **⚠️ Watch Out**: The `getPref('uiMode')` call reads from `localStorage` synchronously at component init. If the user toggles UI mode, `getPref` must return the new value on the next component creation (full page reload or new Svelte component mount).

### Try It Yourself

```svelte
<!-- Add an archive-style breadcrumb to a new page -->
<nav class="archive-breadcrumb">
    <a href="/" class="archive-muted">Home</a>
    <span class="archive-muted">/</span>
    <span>{t('page.title')}</span>
</nav>
```

### Check

- ✅ `uiMode = $derived(getPref('uiMode'))` — reactive to pref changes.
- ✅ Both modes share the same data state — only the template differs.
- ✅ `ArchiveButton` component provides consistent AO3-style button.
- ✅ `ArchiveHeader` provides consistent page header with title + subtitle.
- ✅ Skeleton loaders shown while data is loading in archive mode.

### What you built

The dual UI rendering system — `$derived(getPref('uiMode'))` drives conditional `{#if}` blocks, shared data state, reusable archive-mode components (ArchiveButton, ArchiveHeader, WorkBlurb), and skeleton loaders.

---

## Conclusion

You now understand FicHub's frontend social features:

1. **Follows page** — discriminant union Follow type (user/work/author), optimistic unfollow, dual UI mode.
2. **Ask page** — natural-language search with Ollama, auto-persist to community requests, "interpreted as" chip, empty-result → request conversion.
3. **Dual UI** — archive mode (AO3-style `<dl>`, serif, red accents) and modern mode (cards), sharing data via `$derived`.
4. **Reusable components** — `WorkBlurb`, `ArchiveButton`, `ArchiveHeader` shared across pages.
