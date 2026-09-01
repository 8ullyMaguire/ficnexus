# Part 38 — Frontend Foundations

> In this chapter you will learn how the FicHub SvelteKit frontend is structured — the API client, localization system with six languages, shared UI components, and the routing conventions that keep everything consistent. We'll build the client, the i18n store, and an AO3-style archive header component.

---

## Overview

The FicHub frontend lives in `frontend/` and is a SvelteKit 5 app (Vite bundler, static adapter). It has:

- **API client** (`src/lib/api/client.ts`) — centralized `fetch` wrapper with base URL, error handling, and JWT Bearer auth.
- **Localization** (`src/lib/i18n/`) — six locale dictionaries (en, de, es, fr, pt-BR, zh), a Svelte 5 store that handles locale switching, and an archive UI layer with AO3-style components.
- **Shared components** (`src/lib/ui/`) — `ArchiveHeader`, `WorkBlurb`, `StarRating`, `ReadingProgressBar`.
- **Routing conventions** — every list page uses `+page.test.ts` for Vitest route tests.

The frontend is designed to match AO3's archive style — same heading hierarchy, same navigation patterns, same label vocabulary (e.g. "Kudos" not "Give Kudos"). This is covered in the `ao3-archive-header` and `sveltekit-ao3-archive-parity` skills.

---

## Chapter 38.1 — The API Client

### Goal

Build a centralized `fetch` wrapper that handles base URLs, JWT auth, error parsing, and consistent response shapes.

### Actions

#### 1. The client module

```typescript
// frontend/src/lib/api/client.ts (lines 1–80)
// v2 API client for user accounts, bookmarks, ratings, comments, leaderboards

import type { /* ... */ } from './social-types';

/// Base URL — read from Vite env, fallback to same origin (production deploy)
const BASE_URL =
    import.meta.env.VITE_API_BASE_URL ||
    (import.meta.env.PROD ? '' : 'http://localhost:8000');

/// Current JWT token + auto-refresh logic (managed by auth.svelte.ts store)
let token: string | null = null;
export function setToken(t: string | null) { token = t; }

/// Core fetch wrapper
export async function apiFetch(
    url: string,
    options: RequestInit = {},
): Promise<Response> {
    const headers = new Headers(options.headers);
    if (token) { headers.set('Authorization', `Bearer ${token}`); }
    headers.set('Content-Type', 'application/json');

    const res = await fetch(`${BASE_URL}${url}`, { ...options, headers });

    // 429: rate limited — expose Retry-After header to caller
    if (res.status === 429) {
        const retryAfter = res.headers.get('Retry-After');
        throw new RateLimitError(retryAfter ? parseInt(retryAfter) : undefined);
    }

    // 401: token expired/invalid — caller should redirect to login
    if (res.status === 401) {
        throw new UnauthorizedError();
    }

    return res;
}

export class RateLimitError extends Error {
    constructor(public retryAfter?: number) { super('Rate limited'); }
}
export class UnauthorizedError extends Error {
    constructor() { super('Unauthorized'); }
}

/// Convenience wrappers
export async function apiGet(url: string): Promise<any> {
    const res = await apiFetch(url, { method: 'GET' });
    return res.json();
}

export async function apiPost(url: string, body: any): Promise<any> {
    const res = await apiFetch(url, {
        method: 'POST',
        body: JSON.stringify(body),
    });
    return res.json();
}
```

> **💡 Key Concept**: The client never stores the JWT in `localStorage` — only the `refresh_token` is persisted. The JWT lives in the `auth.svelte.ts` store's memory. This mitigates XSS: even if JS is injected, the attacker can't read the in-memory token (Svelte 5 runes are not accessible via `window.__data`).

#### 2. Typed response wrapper

```typescript
// frontend/src/lib/api/social.ts (lines 30–45)
export interface ApiResponse<T = unknown> {
    err: number;           // 0 = success, non-zero = error code
    msg?: string;          // error message (when err != 0)
    data?: T;              // response body (when err == 0)
}

/// Generic request helper that enforces the { err, ... } contract
export async function request<T = any>(
    method: string,
    url: string,
    body?: any,
): Promise<ApiResponse<T>> {
    const res = await apiFetch(url, {
        method,
        body: body ? JSON.stringify(body) : undefined,
    });
    const data = await res.json().catch(() => ({}));
    return { err: data.err ?? res.status, ...data };
}
```

### Try It Yourself

```typescript
// Add a timeout wrapper to apiFetch
export async function apiFetch(url: string, options: RequestInit = {}): Promise<Response> {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 30000); // 30s
    const res = await fetch(`${BASE_URL}${url}`, {
        ...options,
        signal: controller.signal,
    });
    clearTimeout(timeout);
    return res;
}
```

### Check

- ✅ `BASE_URL` falls back to `""` in production (same origin) and `http://localhost:8000` in dev.
- ✅ `429` responses throw `RateLimitError` with `Retry-After`.
- ✅ `401` responses throw `UnauthorizedError`.
- ✅ JWT is stored in memory (not localStorage).
- ✅ Response wrapper enforces `{ err, ... }` contract.

### What you built

A centralized API client with base URL resolution, JWT Bearer header injection, 429/401 error classes, and a typed response wrapper that enforces FicHub's `{ err: 0 }` success contract.

---

## Chapter 38.2 — Localization System

### Goal

Build a six-language i18n system with hot locale switching, using Svelte 5 runes and a dictionary-per-language pattern.

### Actions

#### 1. Locale dictionaries

```typescript
// frontend/src/lib/i18n/dictionaries/en.ts (lines 1–31)
export const en = {
    // Navigation
    nav: {
        home: 'Home',
        search: 'Search',
        work: 'Work',
        fandoms: 'Fandoms',
        forum: 'Forum',
        curator: 'Curator',
        leaderboard: 'Leaderboard',    // ← literal label per FicHub convention
        kudos: 'Kudos',               // ← terse verb, not "Give kudos"
    },
    reader: {
        toc: 'Table of Contents',
        next: 'Next',
        prev: 'Previous',
        font_size: 'Font Size',
    },
    // ... 923 lines of strings
};
```

> **⚠️ Watch Out**: FicHub's nav uses **literal feature names** — the label for the leaderboard page is "Leaderboard" (not "Rankings"), and the kudos action is "Kudos" (not "Give kudos"). This matches AO3's terse vocabulary. The `ao3-archive-header` skill enforces this pattern across all locales.

```typescript
// frontend/src/lib/i18n/dictionaries/de.ts, es.ts, fr.ts, pt-BR.ts, zh.ts
// Each follows the same structure as en.ts but translated.
```

#### 2. The i18n store

```typescript
// frontend/src/lib/i18n/index.svelte.ts (lines 1–60)
import { en } from './dictionaries/en';
import { de } from './dictionaries/de';
import { es } from './dictionaries/es';
import { fr } from './dictionaries/fr';
import { ptBR } from './dictionaries/pt-BR';
import { zh } from './dictionaries/zh';

export type Locale = 'en' | 'de' | 'es' | 'fr' | 'pt-BR' | 'zh';
export const SUPPORTED_LOCALES: Locale[] = ['en', 'de', 'es', 'fr', 'pt-BR', 'zh'];

const dictionaries: Record<Locale, Record<string, any>> = { en, de, es, fr, 'pt-BR': ptBR, zh };

/// Svelte 5 rune store — re-renders on locale change
export const locale = $state<{ value: Locale }>({ value: 'en' });

/// Set locale + persist to localStorage
export function setLocale(newLocale: Locale) {
    locale.value = newLocale;
    try { localStorage.setItem('fichub_locale', newLocale); } catch { /* non-fatal */ }
}

/// Initialize from localStorage or browser navigator
export function initI18n() {
    const stored = localStorage.getItem('fichub_locale');
    if (stored && SUPPORTED_LOCALES.includes(stored as Locale)) {
        locale.value = stored as Locale;
        return;
    }
    const nav = navigator.language;
    const match = SUPPORTED_LOCALES.find(l => nav.startsWith(l.split('-')[0]));
    locale.value = match ?? 'en';
}

/// Translation function
export function t(path: string): string {
    const parts = path.split('.');
    let obj: any = dictionaries[locale.value];
    for (const p of parts) { obj = obj?.[p]; if (obj === undefined) return path; }
    return typeof obj === 'string' ? obj : path;
}
```

#### 3. Using translations in components

```svelte
<!-- frontend/src/routes/+layout.svelte -->
<script lang="ts">
    import { t, initI18n, SUPPORTED_LOCALES, setLocale } from '$lib/i18n';
    initI18n();
</script>

<a href="/">{t('nav.home')}</a>
<a href="/fandoms">{t('nav.fandoms')}</a>
<a href="/forum">{t('nav.forum')}</a>
<a href="/leaderboard">{t('nav.leaderboard')}</a>  <!-- literal "Leaderboard" -->

<!-- Locale switcher -->
{#each SUPPORTED_LOCALES as loc}
    <button onclick={() => setLocale(loc)}>{loc}</button>
{/each}
```

### Try It Yourself

```bash
# Add a new key to en.ts
# frontend/src/lib/i18n/dictionaries/en.ts
export const en = {
    ...en,
    search: {
        ...(en.search ?? {}),
        no_results: 'No fics found. Try different keywords.',
    },
};
```

### Check

- ✅ `SUPPORTED_LOCALES` has exactly 6 entries: `en, de, es, fr, pt-BR, zh`.
- ✅ `initI18n()` checks localStorage first, then falls back to `navigator.language`.
- ✅ `t('nav.kudos')` returns the terse "Kudos" label (not "Give kudos").
- ✅ Locale is persisted to localStorage and survives page reload.
- ✅ `$state` rune ensures components re-render when `locale.value` changes.

### What you built

The localization system — six locale dictionaries, a Svelte 5 store with `$state` reactivity, locale persistence, and a `t()` translator function that supports dotted-path lookups (e.g., `t('nav.home')`).

---

## Chapter 38.3 — Archive-Style UI Components

### Goal

Build AO3-compatible archive components — the `ArchiveHeader`, `WorkBlurb`, and `StarRating` — that match AO3's heading hierarchy and styling.

### Actions

#### 1. The ArchiveHeader

```svelte
<!-- frontend/src/lib/ui/archive/ArchiveHeader.svelte -->
<script lang="ts">
    export let work: {
        title: string;
        authors: string[];
        fandoms: string[];
        rating: string;
        warnings: string[];
        categories: string[];
        language: string;
        chapters: string;
        words: number;
        summary: string;
        url_id: string;
    };
    import { t } from '$lib/i18n';
</script>

<div class="work-header">
    <h2 class="heading">{work.title}</h2>
    <h3 class="byline">
        by {#each work.authors as author, i}
            {author}{#if i < work.authors.length - 1}, {/if}
        {/each}
    </h3>

    <!-- AO3-style metadata row -->
    <div class="meta">
        <span class="rating">{work.rating}</span>
        {#if work.warnings.length > 0}
            <span class="warnings">
                {#each work.warnings as warning, i}
                    {warning}{#if i < work.warnings.length - 1}, {/if}
                {/each}
            </span>
        {/if}
        <span class="category">
            {#each work.categories as cat, i}
                {cat}{#if i < work.categories.length - 1}, {/if}
            {/each}
        </span>
        <span class="language">{work.language}</span>
        <span class="stats">{work.chapters} chapters · {work.words.toLocaleString()} words</span>
    </div>

    {#if work.summary}
        <blockquote class="summary">{work.summary}</blockquote>
    {/if}
</div>
```

> **💡 Key Concept**: The `ArchiveHeader` mirrors AO3's HTML structure exactly — `<h2 class="heading">` for the work title, `<h3 class="byline">` for authors, a `.meta` div with rating/warnings/category/language/stats. This isn't just visual — it's semantic. AO3 users expect this structure.

#### 2. The WorkBlurb (list item)

```svelte
<!-- frontend/src/lib/ui/archive/WorkBlurb.svelte -->
<script lang="ts">
    export let work: {
        url_id: string;
        title: string;
        author: string;
        fandoms: string[];
        words: number;
        chapters: number;
        status: string;
        rating: string;
        updated: string;
        bookmarked?: boolean;
        rating_score?: number;  // 1..=5
    };
    import { StarRating } from '$lib/components/StarRating.svelte';
    import { t } from '$lib/i18n';
</script>

<article class="blurb">
    <h4 class="title"><a href="/fic/{work.url_id}">{work.title}</a></h4>
    <p class="byline">by {work.author}</p>
    <div class="fandoms">
        {#each work.fandoms as fandom}
            <a class="fandom" href="/fandoms/{fandom}">{fandom}</a>
        {/each}
    </div>
    <div class="blurb-stats">
        <span>{work.words.toLocaleString()} words</span>
        <span>{work.chapters} chapters</span>
        <span class="status">{work.status}</span>
        <StarRating {rating_score} readonly />
        <button class="kudos-btn">{t('nav.kudos')}</button>  <!-- literal "Kudos" -->
    </div>
</article>
```

#### 3. The StarRating

```svelte
<!-- frontend/src/lib/components/StarRating.svelte (2430 chars) -->
<script lang="ts">
    export let rating: number = 0;       // 1..=5
    export let readonly: boolean = false;
    export let showCount: boolean = false;
    export let reviewCount: number = 0;

    function handleRate(star: number) {
        if (!readonly) {
            // dispatch('rate', star)
        }
    }
</script>

<div class="star-rating" class:readonly>
    {#for i in [1, 2, 3, 4, 5]}
        <span class:active={i <= rating} onclick={() => handleRate(i)}>★</span>
    {/for}
    {#if showCount && reviewCount > 0}
        <span class="count">({reviewCount})</span>
    {/if}
</div>
```

### Try It Yourself

```svelte
<!-- Add a new ArchiveHeader field: completion status badge -->
<div class="completion-badge" class:complete={work.status === 'Complete'}>
    {work.status === 'Complete' ? '✓ Complete' : '✓ Ongoing'}
</div>
```

### Check

- ✅ `ArchiveHeader` uses `<h2 class="heading">` and `<h3 class="byline">` (AO3 structure).
- ✅ `WorkBlurb` uses `<h4 class="title">` with an `<a>` link to the fic page.
- ✅ `StarRating` renders 5 `★` characters, fills `i <= rating`.
- ✅ Kudos button uses the terse label `{t('nav.kudos')}` → "Kudos".
- ✅ Fandoms render as `<a>` tags linking to `/fandoms/{slug}`.

### What you built

Three AO3-compatible archive components — `ArchiveHeader` (work detail header), `WorkBlurb` (list item card), and `StarRating` (5-star widget) — all matching AO3's heading structure and terse label vocabulary.

---

## Chapter 38.4 — Routing Conventions and Tests

### Goal

Understand the SvelteKit routing conventions — `+page.svelte` for routes, `+page.test.ts` for tests, and the static adapter for production.

### Actions

#### 1. SvelteKit file-based routing

```
frontend/src/routes/
├── +layout.svelte          ← root layout (wraps all pages)
├── +page.svelte            ← homepage (/)
├── fic/
│   └── [urlId]/
│       └── +page.svelte    ← /fic/{urlId} (individual fic)
├── work/
│   └── [workId]/
│       └── +page.svelte    ← /work/{workId} (canonical work)
├── forum/
│   ├── +page.svelte        ← /forum (index)
│   ├── [categorySlug]/
│   │   └── +page.svelte    ← /forum/{category}
│   └── [topicSlug].[topicId]/
│       └── +page.svelte    ← /forum/{slug}.{id}
├── curator/
│   ├── +page.svelte        ← /curator (dashboard)
│   └── approvals/
│       └── +page.svelte    ← /curator/approvals
└── fandoms/
    └── +page.svelte        ← /fandoms
```

> **💡 Key Concept**: FicHub uses `[slug].[id]` in the URL (e.g., `/forum/my-topic.42`) — the slug is for SEO/human readability, the ID is what the backend uses. The `+` prefix files are SvelteKit's layout and page conventions.

#### 2. Page tests

```typescript
// frontend/src/routes/search/+page.test.ts
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/svelte';
import SearchPage from './+page.svelte';

describe('Search page', () => {
    it('renders search box', () => {
        const { getByPlaceholderText } = render(SearchPage);
        expect(getByPlaceholderText('Search fics...')).toBeInTheDocument();
    });

    it('shows zero state when no results', async () => {
        const { findByText } = render(SearchPage);
        expect(await findByText('No fics found')).toBeInTheDocument();
    });
});
```

#### 3. Static adapter

```typescript
// frontend/svelte.config.js (excerpt)
import adapter from '@sveltejs/adapter-static';
const config = {
    kit: {
        adapter: adapter({
            pages: '../dist/frontend',
            assets: '../dist/frontend',
            fallback: 'index.html',   // SPA fallback for client-side routes
        }),
    },
};
```

> **⚠️ Watch Out**: After frontend deploy, if you see "no UI changes," it's browser cache. FicHub's production uses **immutable hashed assets cached for 1 year** (`?[hash]` in Vite build). The CSS/JS bundles have content hashes, so a new deploy generates new filenames. If the page looks stale, do a hard refresh (Ctrl+Shift+R) or clear the cache for `localhost`.

### Try It Yourself

```bash
# Run frontend tests
cd frontend
npm test -- run src/routes/search/+page.test.ts

# Build for production
npm run build   # outputs to ../dist/frontend
```

### Check

- ✅ Tests are named `+page.test.ts` (not `page.test.ts`) — SvelteKit convention.
- ✅ `fallback: 'index.html'` enables SPA routing for client-side routes.
- ✅ Static adapter outputs to `../dist/frontend` (relative to `frontend/`).
- ✅ The `[slug].[topicId]` pattern uses a literal `.` separator in filenames.

### What you built

Understanding of SvelteKit's file-based routing, the `+page.test.ts` test convention, the static adapter config, and the content-hash cache-busting strategy that requires hard refresh after deploy.

---

## Conclusion

You now understand the FicHub frontend's foundations:

1. **API client** — centralized fetch wrapper with base URL resolution, JWT Bearer injection, 429/401 error classes, and `{ err }` response contract.
2. **Localization** — six-language dictionaries, Svelte 5 `$state` store, locale persistence, dotted-path `t()` translator.
3. **Archive components** — AO3-compatible `ArchiveHeader` (h2/h3 structure), `WorkBlurb` (list card), `StarRating` (5-star widget), all with literal terse labels.
4. **Routing** — file-based `+page.svelte` convention, `[slug].[id]` URL pattern, `+page.test.ts` test files, static adapter with content-hash caching.

In Part 21 we'll build the search UI — the SimpleSearch box, PowerSearch with chips, GuidedSearch form, and the search results page with zero-state handling.
