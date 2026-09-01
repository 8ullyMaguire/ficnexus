# Part 42 — Frontend Forum & Curator Moderation

> In this chapter you will learn how FicHub's discussion forum and curator moderation tools work on the frontend. We'll build the forum category page with its archive-mode toggle, the curator hub with live queue counts, and the unified approvals queue where curators review tag proposals, comment flags, and upload submissions in one place.

---

## Overview

FicHub has two frontend modes that every page supports: **modern** (cards, drop shadows) and **archive** (AO3-style tables, serif fonts). The forum and curator pages demonstrate both — they detect the user's `uiMode` preference from `prefs.ts` and render different markup.

The curator moderation system is unified: instead of separate pages for tag flags, comment triage, and upload moderation, there's one **Approvals** queue that shows items from all sources in a single filterable table. The backend API returns `pending_counts` per type, so the curator hub can show live badges.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database, routes, user accounts).
- You have completed Part 16 (Forum backend) and Part 18 (Moderation backend).
- Your frontend dev server is running (`npm run dev`).
- You have a curator account (role >= 10).

---

## Chapter 42.1 — Forum Category Page

### Goal

Build the forum category list page at `/forum/[categorySlug]` that shows topics in either modern card view or archive table view, with client-side sorting and pagination.

### Actions

#### 1. The SvelteKit page component

`src/routes/forum/[categorySlug]/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import {
    getForumTopics,
    getForumCategories,
    type ForumTopic,
    type ForumCategory,
    type ForumTopicsSort,
  } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  // Read uiMode from prefs — 'archive' or 'modern'
  const uiMode = $derived(getPref('uiMode'));

  let { data } = $props();
  let categorySlug = $derived(data.categorySlug ?? '');
  let items = $state<ForumTopic[]>([]);
  let loading = $state(true);
  let error = $state('');
  let nextCursor = $state<number | null>(null);
  let pageNum = $state(1);
  let hasPrev = $state(false);
  let categoryInfo = $state<ForumCategory | null>(null);

  // Sort options: latest (default), top (by vote score), oldest
  type SortVal = ForumTopicsSort;
  const SORT_OPTIONS: { value: SortVal; label: string }[] = [
    { value: 'latest', label: 'Latest' },
    { value: 'top', label: 'Top' },
    { value: 'oldest', label: 'Oldest' },
  ];

  function currentSort(): SortVal {
    const raw = page.url.searchParams.get('sort');
    if (raw === 'top' || raw === 'oldest' || raw === 'latest') return raw;
    return 'latest';
  }

  function sortedItems(raw: ForumTopic[], sort: SortVal): ForumTopic[] {
    if (sort === 'top') {
      return [...raw].sort((a, b) => {
        const pinnedA = a.status === 'pinned' ? 1 : 0;
        const pinnedB = b.status === 'pinned' ? 1 : 0;
        if (pinnedB !== pinnedA) return pinnedB - pinnedA;
        return (b.vote_score ?? 0) - (a.vote_score ?? 0);
      });
    }
    if (sort === 'oldest') {
      return [...raw].sort((a, b) =>
        new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
      );
    }
    return raw;  // 'latest' — backend already returns them newest-first
  }

  let sort = $derived(currentSort());
  // Client-side sort for 'top' and 'oldest' (backend may ignore sort param)
  let displayItems = $derived(sortedItems(items, sort));

  function setSort(next: SortVal) {
    const url = new URL(page.url);
    if (next === 'latest') url.searchParams.delete('sort');
    else url.searchParams.set('sort', next);
    goto(url.toString(), { keepFocus: true, noScroll: true, replaceState: true });
    pageNum = 1;
    nextCursor = null;
    load(true, next);
  }

  async function load(reset: boolean, overrideSort?: SortVal) {
    loading = true;
    error = '';
    try {
      const s = overrideSort ?? currentSort();
      const res = await getForumTopics(categorySlug, reset ? null : nextCursor, undefined, s);
      if (res.err === 0) {
        const raw = res.items ?? [];
        items = s === 'latest' ? raw : sortedItems(raw, s);
        nextCursor = res.next_cursor;
        hasPrev = pageNum > 1;
      } else {
        error = res.msg ?? t('forum.topicError');
      }
    } catch {
      error = t('forum.topicError');
    } finally {
      loading = false;
    }
  }

  function loadNext() {
    if (nextCursor === null) return;
    pageNum += 1;
    load(false);
  }

  function loadPrev() {
    if (pageNum <= 1) return;
    pageNum -= 1;
    load(true);
  }

  async function loadCategoryInfo() {
    try {
      const res = await getForumCategories();
      if (res.err === 0 && res.items) {
        categoryInfo = res.items.find((c) => c.slug === categorySlug) ?? null;
      }
    } catch {
      // ignore
    }
  }

  onMount(async () => {
    await auth.init();
    await Promise.all([load(true), loadCategoryInfo()]);
  });

  function repliesLabel(count: number): string {
    if (count === 1) return t('forum.replySingular', { count });
    return t('forum.repliesPlural', { count });
  }
</script>

<svelte:head>
  <title>{t('forum.topicsTitle', { title: categorySlug })} — FicHub</title>
</svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode: AO3-style bordered table ── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">
        {categorySlug}
      </h1>
      <p class="archive-muted">{categoryInfo?.description ?? ''}</p>
      <p class="archive-muted" style="font-size:0.85em;margin-top:0.3rem;">
        {categoryInfo?.topic_count ?? 0} topics ·
        {categoryInfo?.last_activity_at
          ? `last activity ${categoryInfo.last_activity_at.slice(0, 10)}`
          : 'no activity yet'}
      </p>
    </header>

    {#if loading}
      <p class="archive-muted"><span class="spinner"></span> Loading…</p>
    {:else if error}
      <div class="archive-error">{error}</div>
    {:else if displayItems.length === 0}
      <p class="archive-muted">No topics yet.</p>
      <p class="archive-muted">Start the conversation!
        <ArchiveButton href={`/forum/new?category=${categorySlug}`}>Start a Topic</ArchiveButton>
      </p>
    {:else}
      <table class="archive-topic-table">
        <thead>
          <tr>
            <th class="col-title">Topic</th>
            <th class="col-author">Author</th>
            <th class="col-stat">Replies</th>
            <th class="col-stat">Views</th>
            <th class="col-stat">Last Post</th>
          </tr>
        </thead>
        <tbody>
          {#each displayItems as tp (tp.id)}
            <tr class:unread={tp.unread === true}>
              <td class="col-title">
                <a class="arc-link" href={tp.topic_slug
                  ? `/forum/board/${tp.topic_slug}.${tp.id}`
                  : `/forum/${categorySlug}/${tp.id}`}>
                  {tp.title}
                </a>
                {#if tp.unread === true}<span class="arc-chip arc-chip-unread">new</span>{/if}
                {#if tp.status === 'pinned'}<span class="arc-chip">pin</span>{/if}
                {#if tp.status === 'locked'}<span class="arc-chip">locked</span>{/if}
              </td>
              <td class="col-author">{tp.author_username ?? '—'}</td>
              <td class="col-stat">{tp.reply_count}</td>
              <td class="col-stat">{tp.view_count}</td>
              <td class="col-stat">
                {tp.last_activity_at ? tp.last_activity_at.slice(0, 10) : ''}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <!-- Archive-mode pagination -->
    {#if (hasPrev || nextCursor !== null)}
      <nav class="archive-nav" aria-label="pagination">
        {#if hasPrev}
          <button class="arc-btn" type="button" onclick={loadPrev} disabled={loading}>
            ‹ Prev
          </button>
        {/if}
        <span class="archive-muted">Page {pageNum}</span>
        {#if nextCursor !== null}
          <button class="arc-btn" type="button" onclick={loadNext} disabled={loading}>
            Next ›
          </button>
        {/if}
      </nav>
    {/if}
  </div>
</main>
{:else}
<!-- ── Modern mode: card-based layout ── -->
<div class="topics-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <div class="head-row">
      <h1>{t('forum.topicsTitle', { title: categorySlug })}</h1>
      <div class="head-actions">
        <a class="btn btn-secondary search-btn" href="/forum/search">{t('forum.searchTitle')}</a>
        <a class="btn btn-primary new-topic-btn" href={`/forum/new?category=${categorySlug}`}>
          {t('forum.newTopic')}
        </a>
      </div>
    </div>

    {#if categoryInfo?.description}
      <p class="category-desc">{categoryInfo.description}</p>
      <p class="category-stats muted">
        {categoryInfo.topic_count} topics ·
        {categoryInfo.last_activity_at
          ? `last activity ${categoryInfo.last_activity_at.slice(0, 10)}`
          : 'no activity yet'}
      </p>
    {/if}

    <!-- Sort buttons -->
    <div class="sort-row" role="group" aria-label="sort topics">
      {#each SORT_OPTIONS as opt (opt.value)}
        <button
          class="sort-btn"
          class:active={sort === opt.value}
          type="button"
          onclick={() => setSort(opt.value)}
          aria-pressed={sort === opt.value}
        >
          {opt.label}
        </button>
      {/each}
    </div>
  </header>

  {#if loading}
    <p class="muted"><span class="spinner"></span> {t('forum.loading')}</p>
  {:else if error}
    <div class="error-card"><strong>{error}</strong></div>
  {:else if displayItems.length === 0}
    <p class="empty">{t('forum.noTopics')}</p>
    <p class="empty-hint">{t('forum.noTopicsHint')}</p>
  {:else}
    <ul class="topic-list">
      {#each displayItems as tp (tp.id)}
        <li class="card topic-card" class:unread={tp.unread === true}>
          <div class="row">
            <a class="title" href={tp.topic_slug
              ? `/forum/board/${tp.topic_slug}.${tp.id}`
              : `/forum/${categorySlug}/${tp.id}`}>
              {tp.title}
            </a>
            {#if tp.unread === true}
              <span class="badge unread-badge">{t('forum.unread')}</span>
            {/if}
            {#if tp.status === 'pinned'}
              <span class="chip">{t('forum.pinned')}</span>
            {/if}
            {#if tp.status === 'locked'}
              <span class="chip">{t('forum.locked')}</span>
            {/if}
          </div>
          <span class="meta">
            {t('forum.by')} {tp.author_username ?? '—'} ·
            {repliesLabel(tp.reply_count)} ·
            {t('forum.views', { count: tp.view_count })} ·
            {tp.last_activity_at ? tp.last_activity_at.slice(0, 10) : ''}
          </span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasPrev || nextCursor !== null}
    <nav class="pagination" aria-label="pagination">
      {#if hasPrev}
        <button class="btn" type="button" onclick={loadPrev} disabled={loading}>
          {t('forum.prevPage')}
        </button>
      {/if}
      <span class="muted">{t('forum.pageOf', { page: pageNum })}</span>
      {#if nextCursor !== null}
        <button class="btn" type="button" onclick={loadNext} disabled={loading}>
          {t('forum.nextPage')}
        </button>
      {/if}
    </nav>
  {/if}
</div>
{/if}

<style>
  /* ── Archive mode ── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content { max-width: 900px; margin: 0 auto; padding: 1rem; }
  .archive-header { padding: 0.8rem 1rem 0.6rem; border-bottom: 2px solid var(--archive-link, #990000); }
  .archive-page-title { font-size: 1.6em; font-weight: 700; margin: 0; }
  .archive-muted { color: var(--archive-muted, #666666); font-style: italic; }
  .archive-topic-table { width: 100%; border-collapse: collapse; font-size: 0.92em; border: 1px solid var(--archive-border, #dddddd); }
  .archive-topic-table thead th { text-align: left; padding: 0.45em 0.7em; border-bottom: 2px solid var(--archive-border, #dddddd); font-size: 0.82em; font-weight: 700; text-transform: uppercase; color: var(--archive-muted, #666666); }
  .archive-topic-table tbody tr { border-bottom: 1px solid var(--archive-border, #dddddd); }
  .archive-topic-table tbody tr:last-child { border-bottom: none; }
  .archive-topic-table tbody tr.unread { background: var(--archive-bg-raised, #f5f5f5); }
  .arc-link { font-weight: 700; color: var(--archive-link, #990000); text-decoration: none; }
  .arc-chip { display: inline-block; font-size: 0.75em; margin-left: 0.4em; padding: 0.1em 0.4em; border: 1px solid var(--archive-border, #dddddd); color: var(--archive-muted, #666666); text-transform: uppercase; }
  .archive-nav { margin-top: 1rem; display: flex; gap: 0.5rem; align-items: center; }
  .arc-btn { padding: 0.3rem 0.8rem; border: 1px solid var(--archive-border, #dddddd); background: var(--archive-bg, #ffffff); font-family: Georgia, serif; cursor: pointer; }

  /* ── Modern mode ── */
  .topics-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .head-row { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap; }
  .head-row h1 { margin: 0; }
  .btn-secondary { background: var(--color-surface, #fff); color: var(--color-text, #222); border: 1px solid var(--color-border, #ddd); text-decoration: none; }
  .category-desc { margin: 0.25rem 0 0; color: var(--color-text-muted, #666); line-height: 1.5; }
  .category-stats { font-size: 0.85rem; margin: 0; }
  .sort-row { display: flex; gap: 0.4rem; margin-top: 0.35rem; }
  .sort-btn { padding: 0.25rem 0.7rem; border: 1px solid var(--color-border, #ddd); background: var(--color-surface, #fff); border-radius: 999px; font-size: 0.85rem; cursor: pointer; }
  .sort-btn.active { background: var(--color-text, #222); color: var(--color-surface, #fff); }
  .topic-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .topic-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .topic-card.unread { border-left: 4px solid #e8a020; }
  .row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .title { font-size: 1.05rem; font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .topic-card.unread .title { font-weight: 700; }
  .badge { display: inline-block; border-radius: 999px; padding: 0.1rem 0.5rem; font-size: 0.72rem; font-weight: 700; background: color-mix(in srgb, #e8a020 20%, transparent); color: #e8a020; }
  .chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .pagination { display: flex; justify-content: center; gap: 1rem; margin-top: 1.25rem; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0 0; }
  .spinner { display: inline-block; width: 0.8rem; height: 0.8rem; border: 2px solid var(--color-border, #555); border-top-color: transparent; border-radius: 50%; animation: spin 0.7s linear infinite; vertical-align: middle; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
```

#### 2. The forum API client

`src/lib/api/forum.ts` — typed wrappers around the backend endpoints:

```typescript
interface ForumCategory {
  id: number;
  slug: string;
  name: string;
  description: string | null;
  topic_count: number;
  last_activity_at: string | null;
}

interface ForumTopic {
  id: number;
  title: string;
  author_username: string | null;
  reply_count: number;
  view_count: number;
  created_at: string;
  last_activity_at: string | null;
  status: 'normal' | 'pinned' | 'locked';
  unread: boolean;
  vote_score?: number;
  topic_slug?: string | null;
}

interface ForumTopicsResponse {
  err: number;
  items: ForumTopic[];
  next_cursor: number | null;
  msg?: string;
}

type ForumTopicsSort = 'latest' | 'top' | 'oldest';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, { credentials: 'include', ...init });
  return res.json() as Promise<T>;
}

export async function getForumCategories(): Promise<{ err: number; items: ForumCategory[]; msg?: string }> {
  return request<{ err: number; items: ForumCategory[] }>('/api/forum/categories');
}

export async function getForumTopics(
  categorySlug: string,
  cursor: number | null,
  limit?: number,
  sort: ForumTopicsSort = 'latest',
): Promise<ForumTopicsResponse> {
  const params = new URLSearchParams();
  if (cursor) params.set('cursor', String(cursor));
  if (limit) params.set('limit', String(limit));
  params.set('sort', sort);
  return request<ForumTopicsResponse>(
    `/api/forum/categories/${categorySlug}/topics?${params.toString()}`,
  );
}

export type { ForumCategory, ForumTopic, ForumTopicsSort };
```

### Try It Yourself

1. Start the dev server and open `/forum/general`.
2. Switch to archive mode: `localStorage.setItem('uiMode', 'archive')` then refresh.
3. Click "Top" sort — topics should reorder client-side by vote score.

### Check

- ✅ `/forum/general` renders topics as cards in modern mode, table in archive mode.
- ✅ The `uiMode` from `prefs.ts` (`localStorage['fichub-prefs']`) determines which layout renders.
- ✅ The "Top" sort pins topics with `status === 'pinned'` to the top, then sorts by `vote_score`.
- ✅ Pagination uses cursor-based navigation (`next_cursor` from the backend).
- ✅ The `Sort` button calls `setSort()` which uses `goto()` with `replaceState: true` so back-button history stays clean.

### What you built

A forum category page that renders both modern cards and AO3-style archive tables, with client-side sorting, cursor pagination, and i18n support via `t('forum.*')`.

---

## Chapter 42.2 — Curator Hub with Live Queue Counts

### Goal

Build the curator hub at `/curator` — a dashboard with cards for every moderation queue. Each card fetches its pending count from the backend API and shows a live badge.

### Actions

#### 1. The curator hub component

`src/routes/curator/+page.svelte`:

```svelte
<script lang="ts">
  import { adminFetch } from '$lib/api/admin';
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));
  let checking = $state(true);
  let countError = $state('');
  const isAdmin = $derived(auth.level >= 100);

  interface QueueCount {
    href: string;
    label: string;
    icon: string;
    desc: string;
    count: number | null;
    fetching: boolean;
  }

  let queues = $state<QueueCount[]>([
    {
      href: '/curator/consensus',
      label: 'Consensus Feed',
      icon: '🧭',
      desc: 'Everything peer-voted in one filterable view — fixes, flags, aliases, merges, roadmap.',
      count: null,
      fetching: false,
    },
    {
      href: '/curator/marginalia',
      label: 'Marginalia',
      icon: 'M',
      desc: 'Per-passage reader discussions — recent topics with work, chapter and reply counts.',
      count: null,
      fetching: true,
    },
    {
      href: '/curator/approvals',
      label: 'Approvals',
      icon: '✓',
      desc: 'Unified queue for collection submissions, curator vote proposals, comment triage, and forum edits.',
      count: null,
      fetching: true,
    },
    {
      href: '/admin/moderation',
      label: 'Upload Moderation',
      icon: '📋',
      desc: 'Pending manual uploads waiting for approve / reject.',
      count: null,
      fetching: true,
    },
    {
      href: '/admin/comment-triage',
      label: 'Comment Triage',
      icon: '💬',
      desc: 'Comments the LLM triage flagged for human review.',
      count: null,
      fetching: true,
    },
  ]);

  // Derive the pending count from each API's response shape
  function deriveCount(url: string, body: any): number | null {
    if (!body || typeof body !== 'object') return null;
    if (url.includes('/api/admin/moderation/queue') && typeof body.total === 'number') {
      return body.total;  // paginated — total is authoritative
    }
    if (url.includes('/api/curator/approvals') && body.pending_counts) {
      // Sum all pending-count categories
      return Object.values(body.pending_counts)
        .reduce((a: number, b: unknown) => a + (Number(b) || 0), 0);
    }
    // Fallback: count array length for non-paginated endpoints
    if (Array.isArray(body.items)) return body.items.length;
    if (Array.isArray(body.flags)) return body.flags.length;
    if (Array.isArray(body.proposals)) return body.proposals.length;
    return null;
  }

  const countSources: { href: string; url: string }[] = [
    { href: '/curator/consensus', url: '/api/curator/consensus/recent' },
    { href: '/curator/marginalia', url: '/api/curator/marginalia?limit=1' },
    { href: '/curator/approvals', url: '/api/curator/approvals?status=pending' },
    { href: '/admin/moderation', url: '/api/admin/moderation/queue?per_page=1' },
    { href: '/admin/comment-triage', url: '/api/admin/moderation/comments' },
  ];

  async function loadCounts() {
    countError = '';
    const settled = await Promise.allSettled(
      countSources.map(async (src) => {
        const res = await adminFetch(src.url, { credentials: 'include' });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return { href: src.href, body: await res.json() };
      }),
    );
    settled.forEach((outcome, i) => {
      const q = queues.find((x) => x.href === countSources[i].href);
      if (!q) return;
      if (outcome.status === 'fulfilled') {
        q.count = deriveCount(countSources[i].url, outcome.value.body);
      } else {
        q.count = null;
      }
      q.fetching = false;
    });
  }

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn || auth.level < 100) {
      goto('/');
      return;
    }
    checking = false;
    void loadCounts();
  });
</script>
```

#### 2. The admin API client

`src/lib/api/admin.ts` — wraps `fetch` with the curator token and error handling:

```typescript
const BASE = '/api';

export async function adminFetch(
  path: string,
  init?: RequestInit,
): Promise<Response> {
  const token = localStorage.getItem('fichub_token') || '';
  const headers = new Headers(init?.headers);
  if (token) headers.set('Authorization', `Bearer ${token}`);
  headers.set('X-Client-ID', getClientId());

  const res = await fetch(`${BASE}${path}`, { ...init, headers, credentials: 'include' });
  if (res.status === 429) {
    throw new Error('Rate limited — try again later');
  }
  return res;
}
```

#### 3. Rendering the hub

The page renders differently in archive vs. modern mode:

```svelte
{#if uiMode === 'archive'}
  <main class="archive-main">
    <div class="archive-content">
      <header class="archive-header">
        <h1 class="archive-page-title">Curator Hub</h1>
        <p class="archive-summary">
          One place for every queue a curator or admin has to review.
        </p>
      </header>
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Review Queues</legend>
        <dl class="archive-dl">
          {#each queues as q (q.href)}
            <div class="dl-row">
              <dt><a class="archive-link" href={q.href}>{q.label}</a></dt>
              <dd>
                <span class="archive-count">{q.fetching ? '…' : (q.count?.toString() ?? 'n/a')}</span>
                <span class="archive-note"> — {q.desc}</span>
              </dd>
            </div>
          {/each}
        </dl>
      </fieldset>
    </div>
  </main>
{:else}
  <div class="hub">
    <header class="hub-head">
      <h1>🛡️ Curator Hub</h1>
      <p class="subtitle">
        One place for every queue a curator or admin has to review —
        author merges, tag flags, work proposals, upload moderation,
        and comment triage. Counts are live from the same APIs the
        queue pages use.
      </p>
    </header>
    <div class="grid">
      {#each queues as q (q.href)}
        <a class="card queue-card" href={q.href} data-sveltekit-preload>
          <div class="queue-icon">{q.icon}</div>
          <div class="queue-body">
            <div class="queue-title-row">
              <h2>{q.label}</h2>
              <span class="badge">{q.fetching ? '…' : (q.count?.toString() ?? 'n/a')}</span>
            </div>
            <p class="queue-desc">{q.desc}</p>
          </div>
        </a>
      {/each}
    </div>
  </div>
{/if}
```

### Try It Yourself

```bash
# Check the curator hub API (as a curator)
curl http://localhost:8000/api/curator/approvals?status=pending \
  -H "Authorization: Bearer <curator-jwt>"
```

You should see `pending_counts` with per-type counts:
```json
{
  "err": 0,
  "items": [...],
  "pending_counts": {
    "collections": 3,
    "vote_proposals": 7,
    "comment_triage": 5,
    "forum_edits": 2
  }
}
```

### Check

- ✅ The curator hub shows cards for each queue: Consensus, Marginalia, Approvals, Upload Moderation, Comment Triage.
- ✅ Each card's badge fetches its count via `adminFetch` — a failed fetch leaves the badge as `n/a`, not an error.
- ✅ Archive mode renders a bordered `<dl>` table; modern mode renders card tiles.
- ✅ Non-curators (`auth.level < 100`) are redirected to `/`.

### What you built

A curator hub dashboard that shows live pending counts for every moderation queue, with archive-mode styling that matches the AO3 look.

---

## Chapter 42.3 — Unified Approvals Queue

### Goal

Build the unified approvals page at `/curator/approvals` where curators review items from multiple sources (collection submissions, vote proposals, comment triage, forum edits) in one filterable table.

### Actions

#### 1. Fetching the unified queue

The frontend calls `GET /api/curator/approvals?status=pending` which returns items from all sources mixed together:

```typescript
// src/lib/api/forum.ts (or a dedicated curator API)
export async function getApprovals(status: string, page: number = 1) {
  return request<{
    err: number;
    items: ApprovalItem[];
    pending_counts: Record<string, number>;
    total: number;
    page: number;
    per_page: number;
  }>(`/api/curator/approvals?status=${status}&page=${page}`);
}

interface ApprovalItem {
  id: number;
  item_type: 'collection' | 'vote_proposal' | 'comment_triage' | 'forum_edit';
  status: 'pending' | 'approved' | 'rejected';
  priority: 'low' | 'medium' | 'high';
  created_at: string;
  created_by: number;
  target_type?: string;
  target_id?: number;
  details: string;   // JSON string with type-specific data
}
```

#### 2. Filtering and sorting

```svelte
<script lang="ts">
  let activeFilter = $state('pending');
  let activeSort = $state('created_at');
  let activeOrder = $state<'asc' | 'desc'>('desc');
  let searchQuery = $state('');

  const FILTERS = [
    { value: 'pending', label: 'Pending' },
    { value: 'approved', label: 'Approved' },
    { value: 'rejected', label: 'Rejected' },
  ];

  const SORTS = [
    { value: 'created_at', label: 'Date' },
    { value: 'priority', label: 'Priority' },
    { value: 'item_type', label: 'Type' },
  ];
</script>

<div class="filter-bar">
  <div class="filter-group">
    {#each FILTERS as f}
      <button
        class:active={activeFilter === f.value}
        onclick={() => activeFilter = f.value}
      >
        {f.label}
      </button>
    {/each}
  </div>
  <input
    type="text"
    placeholder="Search by type or ID…"
    bind:value={searchQuery}
  />
</div>

<table class="approval-table">
  <thead>
    <tr>
      <th>ID</th>
      <th>Type</th>
      <th>Priority</th>
      <th>Submitted</th>
      <th>Details</th>
      <th class="actions-col">Actions</th>
    </tr>
  </thead>
  <tbody>
    {#each filteredItems as item (item.id)}
      <tr class={item.priority}>
        <td>{item.id}</td>
        <td>
          <span class="type-tag">{item.item_type}</span>
        </td>
        <td>
          <span class={`priority-${item.priority}`}>{item.priority}</span>
        </td>
        <td>{new Date(item.created_at).toLocaleDateString()}</td>
        <td class="details">
          {#if item.details}
            {@html truncate(JSON.parse(item.details).summary || '')}
          {/if}
        </td>
        <td class="actions-col">
          <button onclick={() => approve(item.id)}>Approve</button>
          <button onclick={() => reject(item.id)}>Reject</button>
        </td>
      </tr>
    {/each}
  </tbody>
</table>
```

### Try It Yourself

1. Log in as a curator and open `/curator/approvals`.
2. Filter by "Pending" — you should see all unreviewed items.
3. Click "Approve" on a comment triage item — it should disappear from the pending list.

### Check

- ✅ The approvals page fetches from `/api/curator/approvals?status=pending`.
- ✅ Items show `pending_counts` for each type in badges on the curator hub.
- ✅ Filtering by status (pending/approved/rejected) updates the URL query param.
- ✅ Approve/Reject buttons call `POST /api/curator/approvals/{id}/approve` or `/reject`.

### What you built

A unified moderation interface that combines all curator queues into one filterable table, with live count badges that sync back to the curator hub.

---

## Conclusion

You now understand FicHub's frontend moderation tools:

- **Forum category page** with dual modern/archive rendering, client-side sorting, and cursor pagination.
- **Curator hub** dashboard with live queue-count badges fetched from the backend API.
- **Unified approvals queue** that combines collection submissions, vote proposals, comment triage, and forum edits into one filterable table.
- Both modes use `getPref('uiMode')` from `prefs.ts` to determine rendering.

---

## On to the next part

In Part 21 we'll build [Frontend Social Features] — the follows feed, notifications bell, and reading stats cards that live in the user dashboard.
