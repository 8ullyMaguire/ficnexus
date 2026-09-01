<script lang="ts">
  import { onMount } from 'svelte';
  import { getFeed } from '$lib/api/social';
  import type { FeedItem } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let items = $state<FeedItem[]>([]);
  let loading = $state(true);
  let error = $state('');
  let page = $state(1);
  let total = $state(0);
  let perPage = $state(20);

  onMount(async () => {
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadFeed();
  });

  async function loadFeed() {
    loading = true; error = '';
    try {
      const res = await getFeed(page, perPage);
      if (res.err === 0) { items = res.items; total = res.total; }
      else error = 'Failed to load feed.';
    } catch { error = 'Network error.'; }
    finally { loading = false; }
  }

  $effect(() => { loadFeed(); });

  function relativeTime(dateStr: string): string {
    const now = Date.now();
    const d = new Date(dateStr).getTime();
    const diff = Math.floor((now - d) / 1000);
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)} minutes ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} hours ago`;
    if (diff < 2592000) return `${Math.floor(diff / 86400)} days ago`;
    return new Date(dateStr).toLocaleDateString();
  }

  function formatIcon(format: string): string {
    const icons: Record<string, string> = { epub: '📘', html: '🌐', pdf: '📄', mobi: '📱', txt: '📃', md: '📝', azw3: '📕' };
    return icons[format] || '📄';
  }

  const pageCount = $derived(Math.ceil(total / perPage));

  function goPage(p: number) {
    if (p < 1 || p > pageCount) return;
    page = p;
  }
</script>

<svelte:head><title>Following Feed | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Following Feed</h1>
      <p class="archive-muted archive-sub">Recently updated works from authors you follow.</p>
    </header>

    {#if loading}
      <p class="archive-loading">Loading...</p>
    {:else if error}
      <p class="archive-error-text">{error}</p>
    {:else if items.length === 0}
      <blockquote class="archive-summary">
        <p>No recent updates from followed works.</p>
        <p>Follow some works or authors to see their updates here.</p>
      </blockquote>
    {:else}
      <dl class="archive-dl feed-dl">
        {#each items as item (item.url_id)}
          <div class="dl-row">
            <dt>
              <a class="archive-link" href={`/works/${item.url_id}`}>{item.title}</a>
            </dt>
            <dd>
              <span class="archive-muted">by {item.author}</span>
              <span class="archive-feed-time">{relativeTime(item.updated_at)}</span>
              <span class="archive-format">{item.format.toUpperCase()}</span>
            </dd>
          </div>
        {/each}
      </dl>

      {#if pageCount > 1}
        <div class="archive-pagination">
          <button class="archive-btn" type="button" disabled={page <= 1} onclick={() => goPage(page - 1)}>Previous</button>
          <span class="page-info">Page {page} of {pageCount}</span>
          <button class="archive-btn" type="button" disabled={page >= pageCount} onclick={() => goPage(page + 1)}>Next</button>
        </div>
      {/if}
    {/if}
  </div>
</main>
{:else}
<div class="feed-page">
  <h1>📰 Following Feed</h1>
  <p class="muted">Recently updated works from authors you follow.</p>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <div class="card empty">
      <p class="muted">No recent updates from followed works.</p>
      <p class="muted" style="margin-top: 0.5rem;">Follow some works or authors to see their updates here.</p>
    </div>
  {:else}
    <div class="feed-list">
      {#each items as item (item.url_id)}
        <div class="card feed-item">
          <span class="feed-icon">{formatIcon(item.format)}</span>
          <div class="feed-content">
            <a class="feed-title" href="/works/{item.url_id}">{item.title}</a>
            <span class="feed-author muted">by {item.author}</span>
            <span class="feed-time muted">{relativeTime(item.updated_at)}</span>
          </div>
          <span class="feed-format muted">{item.format.toUpperCase()}</span>
        </div>
      {/each}
    </div>

    {#if pageCount > 1}
      <div class="pagination">
        <button class="btn-page" disabled={page <= 1} onclick={() => goPage(page - 1)}>← Prev</button>
        <span class="page-info">Page {page} of {pageCount}</span>
        <button class="btn-page" disabled={page >= pageCount} onclick={() => goPage(page + 1)}>Next →</button>
      </div>
    {/if}
  {/if}
</div>
{/if}

<style>
  /* ── Archive mode ─────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    padding-bottom: 0.6em;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    margin: 0;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-sub {
    margin: 0.4em 0 0;
    font-style: italic;
  }
  .archive-loading,
  .archive-muted,
  .archive-feed-time,
  .page-info {
    color: var(--archive-muted, #666666);
  }
  .archive-error-text {
    color: var(--color-error, #cc0000);
    font-weight: 600;
  }
  .archive-summary {
    border-left: 3px solid var(--archive-border, #dddddd);
    margin: 1rem 0;
    padding: 0.6rem 1rem;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-summary p { margin: 0 0 0.4em; }
  .archive-summary p:last-child { margin-bottom: 0; }
  .feed-dl { margin: 0; }
  .feed-dl .dl-row {
    padding: 0.55em 0.25em;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .feed-dl .dl-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .feed-dl .dl-row:last-child { border-bottom: none; }
  .feed-dl dt { font-weight: 700; font-size: 0.98em; }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover { text-decoration: underline; }
  .feed-dl dd {
    margin: 0.15em 0 0;
    display: flex;
    align-items: center;
    gap: 0.8em;
    flex-wrap: wrap;
    font-size: 0.88em;
  }
  .archive-feed-time { font-size: 0.92em; }
  .archive-format {
    font-size: 0.8em;
    font-weight: 700;
    letter-spacing: 0.04em;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg, #ffffff);
    padding: 0.05em 0.45em;
  }
  .archive-pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-top: 1.2em;
    font-size: 0.9em;
  }
  .archive-btn {
    padding: 0.28rem 0.75rem;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  /* ── Modern mode ── */
  .feed-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .feed-page .empty { text-align: center; padding: 2rem; }
  .feed-page .error-card { border-color: var(--color-error); }
  .feed-page .feed-list { display: flex; flex-direction: column; gap: 0.5rem; }
  .feed-page .feed-item { display: flex; align-items: center; gap: 0.8rem; padding: 0.7rem 1rem; }
  .feed-page .feed-icon { font-size: 1.3rem; }
  .feed-page .feed-content { flex: 1; display: flex; flex-direction: column; gap: 0.15rem; }
  .feed-page .feed-title { font-weight: 600; color: var(--color-primary); text-decoration: none; }
  .feed-page .feed-title:hover { text-decoration: underline; }
  .feed-page .feed-author { font-size: 0.85rem; }
  .feed-page .feed-time { font-size: 0.8rem; }
  .feed-page .feed-format { font-size: 0.75rem; font-weight: 700; padding: 0.2rem 0.5rem; border-radius: var(--radius-sm); background: var(--color-surface-2); }
  .feed-page .pagination { display: flex; align-items: center; justify-content: center; gap: 1rem; margin-top: 1rem; }
  .feed-page .btn-page { padding: 0.4rem 0.8rem; border-radius: var(--radius-sm); background: var(--color-surface-2); border: 1px solid var(--color-border); color: var(--color-text); cursor: pointer; font-weight: 600; }
  .feed-page .btn-page:disabled { opacity: 0.4; cursor: default; }
  .feed-page .btn-page:hover:not(:disabled) { background: var(--color-primary); color: white; }
  .feed-page .page-info { font-size: 0.85rem; color: var(--color-muted); }
</style>
