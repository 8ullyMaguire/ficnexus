<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { getForumTopics, getForumCategories, type ForumTopic, type ForumCategory, type ForumTopicsSort } from '$lib/api/forum';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

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
      return [...raw].sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
    }
    return raw;
  }

  let sort = $derived(currentSort());
  let displayItems = $derived(sortedItems(items, sort));

  function setSort(next: SortVal) {
    const url = new URL(page.url);
    if (next === 'latest') url.searchParams.delete('sort');
    else url.searchParams.set('sort', next);
    goto(url.toString(), { keepFocus: true, noScroll: true, replaceState: true });
    // reset pagination and reload with new sort
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
        // backend may ignore sort param; we also client-side sort for 'top'/'oldest'
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

  // react to sort param changes via navigation (back/forward)
  $effect(() => {
    // track page.url.searchParams.get('sort') so effect reruns on url change
    const s = currentSort();
    // only reload if we are not already loading and items have been loaded once
    // The goto-triggered setSort already called load, so avoid double-fetch
    // by not auto-reloading here; this handles back/forward navigation
    void s;
  });

  function repliesLabel(count: number): string {
    if (count === 1) return t('forum.replySingular', { count });
    return t('forum.repliesPlural', { count });
  }
</script>

<svelte:head><title>{t('forum.topicsTitle', { title: categorySlug })} — FicNexus</title></svelte:head>

<div class="topics-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <div class="head-row">
      <h1>{uiMode === 'archive' ? categorySlug : t('forum.topicsTitle', { title: categorySlug })}</h1>
      <div class="head-actions">
        {#if uiMode !== 'archive'}
          <a class="btn btn-secondary search-btn" href="/forum/search">{t('forum.searchTitle')}</a>
        {/if}
        {#if uiMode === 'archive'}
          <ArchiveButton href={`/forum/new?category=${categorySlug}`}>Start a Topic</ArchiveButton>
        {:else}
          <a class="btn btn-primary new-topic-btn" href={`/forum/new?category=${categorySlug}`}>{t('forum.newTopic')}</a>
        {/if}
      </div>
    </div>
    {#if categoryInfo?.description}
      <p class="category-desc">{categoryInfo.description}</p>
      <p class="category-stats muted">{categoryInfo.topic_count} topics · {categoryInfo.last_activity_at ? `last activity ${categoryInfo.last_activity_at.slice(0, 10)}` : 'no activity yet'}</p>
    {/if}
    <div class="sort-row" role="group" aria-label="sort topics">
      {#each SORT_OPTIONS as opt (opt.value)}
        <button
          class="sort-btn"
          class:active={sort === opt.value}
          type="button"
          onclick={() => setSort(opt.value)}
          aria-pressed={sort === opt.value}
        >{opt.label}</button>
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
  {:else if uiMode === 'archive'}
    <!-- Archive: bordered topic table -->
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
              <a class="arc-link" href={tp.topic_slug ? `/forum/board/${tp.topic_slug}.${tp.id}` : `/forum/${categorySlug}/${tp.id}`}>{tp.title}</a>
              {#if tp.unread === true}<span class="arc-chip arc-chip-unread">new</span>{/if}
              {#if tp.status === 'pinned'}<span class="arc-chip">pin</span>{/if}
              {#if tp.status === 'locked'}<span class="arc-chip">locked</span>{/if}
            </td>
            <td class="col-author">{tp.author_username ?? '—'}</td>
            <td class="col-stat">{tp.reply_count}</td>
            <td class="col-stat">{tp.view_count}</td>
            <td class="col-stat">{tp.last_activity_at ? tp.last_activity_at.slice(0, 10) : ''}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {:else}
    <!-- Modern: card-based topic list -->
    <ul class="topic-list">
      {#each displayItems as tp (tp.id)}
        <li class="card topic-card" class:unread={tp.unread === true}>
          <div class="row">
            <a class="title" href={tp.topic_slug ? `/forum/board/${tp.topic_slug}.${tp.id}` : `/forum/${categorySlug}/${tp.id}`}>{tp.title}</a>
            {#if tp.unread === true}<span class="badge unread-badge">{t('forum.unread')}</span>{/if}
            {#if tp.status === 'pinned'}<span class="chip">{t('forum.pinned')}</span>{/if}
            {#if tp.status === 'locked'}<span class="chip">{t('forum.locked')}</span>{/if}
          </div>
          <span class="meta">
            {t('forum.by')} {tp.author_username ?? '—'} · {repliesLabel(tp.reply_count)} · {t('forum.views', { count: tp.view_count })} · {tp.last_activity_at ? tp.last_activity_at.slice(0, 10) : ''}
          </span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasPrev || nextCursor !== null}
    <nav class="pagination" aria-label="pagination">
      {#if hasPrev}
        <button class="btn" type="button" onclick={loadPrev} disabled={loading}>{t('forum.prevPage')}</button>
      {/if}
      <span class="muted">{t('forum.pageOf', { page: pageNum })}</span>
      {#if nextCursor !== null}
        <button class="btn" type="button" onclick={loadNext} disabled={loading}>{t('forum.nextPage')}</button>
      {/if}
    </nav>
  {/if}
</div>

<style>
  .topics-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .head-row { display: flex; align-items: center; justify-content: space-between; gap: 0.75rem; flex-wrap: wrap; }
  .head-row h1 { margin: 0; }
  .head-actions { display: flex; align-items: center; gap: 0.5rem; }
  .new-topic-btn, .search-btn { text-decoration: none; }
  .btn-secondary { background: var(--color-surface, #fff); color: var(--color-text, #222); border: 1px solid var(--color-border, #ddd); }
  .category-desc { margin: 0.25rem 0 0; color: var(--color-text-muted, #666); line-height: 1.5; }
  .category-stats { font-size: 0.85rem; margin: 0; }
  .sort-row { display: flex; gap: 0.4rem; margin-top: 0.35rem; }
  .sort-btn { padding: 0.25rem 0.7rem; border: 1px solid var(--color-border, #ddd); background: var(--color-surface, #fff); border-radius: 999px; font-size: 0.85rem; cursor: pointer; }
  .sort-btn.active { background: var(--color-text, #222); color: var(--color-surface, #fff); border-color: var(--color-text, #222); }
  .topic-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .topic-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .topic-card.unread { border-left: 4px solid #e8a020; }
  .row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .title { font-size: 1.05rem; font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .topic-card.unread .title { font-weight: 700; }
  .badge { display: inline-block; border-radius: 999px; padding: 0.1rem 0.5rem; font-size: 0.72rem; font-weight: 700; }
  .unread-badge { background: #e8a020; color: #fff; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; }
  .pagination { display: flex; align-items: center; justify-content: center; gap: 1rem; margin-top: 1.25rem; }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0 0; }
  .empty-hint { text-align: center; color: var(--color-text-muted, #888); }

  /* ── Archive table ─────────────────────────────────────────────── */
  .archive-topic-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.92em;
    border: 1px solid var(--archive-border, #dddddd);
  }
  .archive-topic-table thead th {
    text-align: left;
    padding: 0.45em 0.7em;
    border-bottom: 2px solid var(--archive-border, #dddddd);
    font-size: 0.82em;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--archive-muted, #666666);
  }
  .archive-topic-table tbody tr {
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-topic-table tbody tr:last-child {
    border-bottom: none;
  }
  .archive-topic-table tbody tr.unread {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-topic-table td {
    padding: 0.5em 0.7em;
    vertical-align: top;
  }
  .col-title { width: 100%; }
  .col-author { white-space: nowrap; }
  .col-stat { text-align: right; white-space: nowrap; }
  .arc-link {
    font-weight: 700;
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .arc-link:hover { color: #999; }
  .arc-chip {
    display: inline-block;
    font-size: 0.75em;
    margin-left: 0.4em;
    padding: 0.1em 0.4em;
    border: 1px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
</style>
