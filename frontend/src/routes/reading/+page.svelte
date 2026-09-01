<script lang="ts">
  import { onMount } from 'svelte';
  import { getReadingList, getWork } from '$lib/api/social';
  import type { ReadingListItem, ReadingStatus, Work } from '$lib/api/social-types';
  import { auth } from '$lib/stores/auth.svelte';
  import { goto } from '$app/navigation';
  import { formatWords, relativeTime } from '$lib/util';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

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

  let items = $state<ReadingListItem[]>([]);
  let works = $state<Record<number, Work | null>>({});
  let loading = $state(true);
  let error = $state('');
  let activeTab = $state<ReadingStatus | ''>('');

  onMount(async () => {
    await auth.init();
    if (!auth.isLoggedIn) { goto('/'); return; }
    await loadList();
  });

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

  function switchTab(tab: ReadingStatus | '') {
    activeTab = tab;
    loadList();
  }

  function statusBadge(status: string): string {
    switch (status) {
      case 'want_to_read': return '📋';
      case 'reading': return '📖';
      case 'completed': return '✅';
      case 'dropped': return '❌';
      default: return '❓';
    }
  }

  function statusLabel(status: string): string {
    switch (status) {
      case 'want_to_read': return 'Want to Read';
      case 'reading': return 'Reading';
      case 'completed': return 'Completed';
      case 'dropped': return 'Dropped';
      default: return status;
    }
  }

  function workMeta(workId: number): { title: string; author: string; words: number } | null {
    const w = works[workId];
    if (!w) return null;
    return {
      title: w.canonical_title,
      author: w.canonical_author,
      words: w.sources[0]?.words ?? 0,
    };
  }
</script>

<svelte:head><title>My Reading List | FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">My Reading List</h1>
      <p class="archive-muted archive-sub">Track what you're reading, want to read, or have finished.</p>
    </header>

    <!-- Status filter fieldset — AO3 archive style -->
    <fieldset class="archive-fieldset">
      <legend>Status</legend>
      <div class="archive-filter-options">
        {#each ARCHIVE_TABS as tab}
          <button
            class="archive-btn"
            class:current={activeTab === tab.value}
            type="button"
            onclick={() => switchTab(tab.value)}
            aria-pressed={activeTab === tab.value}
          >{tab.label}</button>
        {/each}
      </div>
    </fieldset>

    {#if loading}
      <div class="archive-loading"><p>Loading...</p></div>
    {:else if error}
      <p class="archive-error-text">{error}</p>
    {:else if items.length === 0}
      <blockquote class="archive-summary">
        <p>
          {#if activeTab}
            No works with status "{statusLabel(activeTab)}" yet.
          {:else}
            Your reading list is empty. Set a reading status from a work's page to start tracking.
          {/if}
        </p>
      </blockquote>
    {:else}
      <p class="archive-muted archive-count">{items.length} work{items.length !== 1 ? 's' : ''}</p>

      <dl class="archive-dl reading-dl">
        {#each items as r (r.id)}
          {@const meta = workMeta(r.work_id)}
          <div class="dl-row">
            <dt>
              <span class="archive-status-label">{statusLabel(r.status)}</span>
              {#if meta}
                <a class="archive-link" href={`/work/${r.work_id}`}>{meta.title}</a>
              {:else}
                Work #{r.work_id}
              {/if}
            </dt>
            <dd>
              {#if meta}<span class="archive-muted">by {meta.author}</span>{/if}
              <span class="archive-meta-line">
                {#if r.status === 'reading'}
                  {formatWords(r.words_read)} words read
                  {#if r.current_chapter} · Chapter {r.current_chapter}{/if}
                {:else if r.status === 'completed'}
                  {formatWords(r.words_read)} words · {r.read_count} read{r.read_count !== 1 ? 's' : ''}
                {:else}
                  {formatWords(r.words_read)} words
                {/if}
                · Last read {relativeTime(r.last_read_at)}
              </span>
            </dd>
          </div>
        {/each}
      </dl>
    {/if}
  </div>
</main>
{:else}
<div class="reading-page">
  <h1>📚 My Reading List</h1>
  <p class="muted">Track what you're reading, want to read, or have finished.</p>

  <div class="tabs">
    {#each TABS as tab}
      <button
        class="tab"
        class:active={activeTab === tab.value}
        onclick={() => switchTab(tab.value)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if loading}
    <div class="card"><p class="muted"><span class="spinner"></span> Loading...</p></div>
  {:else if error}
    <div class="card error-card"><strong class="error-text">⚠️ {error}</strong></div>
  {:else if items.length === 0}
    <div class="card empty">
      <p class="muted">
        {#if activeTab}
          No works with status "{activeTab}" yet.
        {:else}
          Your reading list is empty. Set a reading status from a work's page to start tracking.
        {/if}
      </p>
    </div>
  {:else}
    <p class="muted count">{items.length} work{items.length !== 1 ? 's' : ''}</p>
    <div class="reading-list">
      {#each items as r (r.id)}
        {@const meta = workMeta(r.work_id)}
        <div class="card reading-item">
          <div class="reading-status-icon">{statusBadge(r.status)}</div>
          <div class="reading-info">
            {#if meta}
              <h3><a href="/work/{r.work_id}">{meta.title}</a></h3>
              <p class="muted">by {meta.author}</p>
            {:else}
              <h3>Work #{r.work_id}</h3>
            {/if}
            <p class="meta-line muted">
              {#if r.status === 'reading'}
                {formatWords(r.words_read)} words read
                {#if r.current_chapter} · Chapter {r.current_chapter}{/if}
              {:else if r.status === 'completed'}
                {formatWords(r.words_read)} words · {r.read_count} read{r.read_count !== 1 ? 's' : ''}
              {:else}
                {formatWords(r.words_read)} words
              {/if}
              · Last read {relativeTime(r.last_read_at)}
            </p>
          </div>
        </div>
      {/each}
    </div>
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
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-filter-options {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }
  .archive-btn {
    padding: 0.28rem 0.75rem;
    font-size: 0.85em;
    font-family: inherit;
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    line-height: 1.5;
  }
  .archive-btn:hover {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn.current {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-loading,
  .archive-count {
    color: var(--archive-muted, #666666);
    font-size: 0.9em;
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
  .archive-summary p { margin: 0; }
  .reading-dl { margin: 0; }
  .reading-dl .dl-row {
    display: grid;
    grid-template-columns: minmax(140px, 220px) 1fr;
    gap: 0.3em 1em;
    padding: 0.55em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }
  .reading-dl .dl-row:last-child { border-bottom: none; }
  .reading-dl dt { font-weight: 700; font-size: 0.95em; }
  .reading-dl dt .archive-status-label {
    display: block;
    font-weight: 400;
    font-size: 0.8em;
    color: var(--archive-muted, #666666);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }
  .archive-link:hover { text-decoration: underline; }
  .reading-dl dd { margin: 0; font-size: 0.9em; }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-meta-line {
    display: block;
    margin-top: 0.15em;
    color: var(--archive-muted, #666666);
  }

  @media (max-width: 600px) {
    .reading-dl .dl-row { grid-template-columns: 1fr; }
  }

  /* ── Modern mode ── */
  .reading-page { max-width: var(--max-width); margin: 0 auto; padding: 1rem; }
  .reading-page .tabs { display: flex; flex-wrap: wrap; gap: 0.4rem; margin: 1rem 0; }
  .reading-page .tab { padding: 0.4rem 0.8rem; border: 1px solid var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface); color: var(--color-text); cursor: pointer; font-size: 0.85rem; }
  .reading-page .tab:hover { background: var(--color-surface-2); }
  .reading-page .tab.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }
  .reading-page .empty { text-align: center; padding: 2rem; }
  .reading-page .error-card { border-color: var(--color-error); }
  .reading-page .count { font-size: 0.9rem; margin-bottom: 0.8rem; }
  .reading-page .reading-list { display: flex; flex-direction: column; gap: 0.6rem; }
  .reading-page .reading-item { display: flex; align-items: flex-start; gap: 0.8rem; padding: 0.8rem 1rem; }
  .reading-page .reading-status-icon { font-size: 1.4rem; flex-shrink: 0; padding-top: 0.2rem; }
  .reading-page .reading-info { flex: 1; }
  .reading-page .reading-info h3 { margin: 0 0 0.2rem; font-size: 1.05rem; }
  .reading-page .reading-info h3 a { color: var(--color-text); text-decoration: none; }
  .reading-page .reading-info h3 a:hover { color: var(--color-primary); }
  .reading-page .meta-line { margin: 0.2rem 0 0; font-size: 0.85rem; }
</style>
