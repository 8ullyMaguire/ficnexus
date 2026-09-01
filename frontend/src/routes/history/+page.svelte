<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';
  import { getReadingHistory, deleteReadHistory, clearReadHistory } from '$lib/api/social';
import type { ReadingHistoryEntry } from '$lib/api/social-types';

  const uiMode = $derived(getPref('uiMode'));

  let history = $state<ReadingHistoryEntry[]>([]);
  let total = $state(0);
  let loading = $state(true);
  let error = $state('');
  let removingId = $state<number | null>(null);
  let clearing = $state(false);

  // Pagination
  let limit = 50;
  let offset = $state(0);
  let currentPage = $state(1);

  async function load() {
    // Ensure auth is initialized first
    if (!auth.initialized) {
      await auth.init();
    }
    if (!auth.isLoggedIn) {
      loading = false;
      return;
    }
    loading = true;
    error = '';
    try {
      const res = await getReadingHistory(limit, offset);
      if (res.err === 0) {
        history = res.history;
        total = res.total;
      } else {
        error = res.err.toString();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function removeEntry(id: number) {
    removingId = id;
    try {
      await deleteReadHistory(id);
      history = history.filter((h) => h.id !== id);
      total -= 1;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      removingId = null;
    }
  }

  async function clearAll() {
    if (!confirm(t('history.clearConfirm'))) return;
    clearing = true;
    try {
      await clearReadHistory();
      history = [];
      total = 0;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      clearing = false;
    }
  }

  function goToPage(page: number) {
    currentPage = page;
    offset = (page - 1) * limit;
    load();
  }

  onMount(load);
</script>

<svelte:head><title>{t('history.title')} — FicNexus</title></svelte:head>

{#if loading}
  <p class="loading-text">Loading...</p>
{:else if !auth.isLoggedIn}
  <div class="archive-empty">
    <p>{t('history.loginPrompt')}</p>
    <a href="/login">Log In</a>
  </div>
{:else if uiMode === 'archive'}
  <div class="archive-history-page">
    <header class="archive-header">
      <h1 class="archive-title">{t('history.title')}</h1>
      <p class="archive-count">{total} {total === 1 ? t('history.item') : t('history.items')}</p>
      <div class="history-toolbar">
        <button class="archive-btn" type="button" onclick={clearAll} disabled={clearing || history.length === 0}>
          {#if clearing}
            {t('history.clearing')}
          {:else}
            {t('history.clearButton')}
          {/if}
        </button>
      </div>
    </header>

    {#if error}
      <p class="error-text">{error}</p>
    {:else if history.length === 0}
      <p class="empty-text">{t('history.empty')}</p>
    {:else}
      <ul class="history-list">
        {#each history as entry (entry.id)}
          <li class="history-row">
            <span class="history-date">
              {new Date(entry.visited_at).toLocaleDateString(undefined, {
                year: 'numeric', month: 'short', day: 'numeric',
                hour: '2-digit', minute: '2-digit',
              })}
            </span>
            <a class="history-title" href={`/works/${entry.url_id}`}>{entry.title}</a>
            <span class="history-author">by {entry.author}</span>
            {#if entry.chapter_num}
              <span class="history-chapter">ch. {entry.chapter_num}</span>
            {/if}
            <span class="history-actions">
              <ArchiveButton href={`/works/${entry.url_id}`}>{t('history.readButton')}</ArchiveButton>
              <button class="archive-btn small" type="button" onclick={() => removeEntry(entry.id)} disabled={removingId === entry.id}>
                {#if removingId === entry.id}
                  {t('history.removing')}
                {:else}
                  {t('history.removeButton')}
                {/if}
              </button>
            </span>
          </li>
        {/each}
      </ul>

      <!-- Pagination -->
      {#if total > limit}
        <div class="history-pagination">
          {#each Array(Math.ceil(total / limit)).fill(0).map((_, i) => i + 1) as page}
            <button class="archive-btn small" class:active={currentPage === page} onclick={() => goToPage(page)}>
              {page}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
{:else}
  <!-- Modern mode -->
  <div class="history-page">
    <h1>{t('history.title')}</h1>
    <p class="stats-summary">{total} entries</p>
    {#if error}
      <p class="error-text">{error}</p>
    {:else if history.length === 0}
      <p class="empty">{t('history.empty')}</p>
    {:else}
      <ul class="history-list">
        {#each history as entry (entry.id)}
          <li>
            <span class="date">{new Date(entry.visited_at).toLocaleDateString()}</span>
            <a href={`/works/${entry.url_id}`}>{entry.title}</a>
            <span class="author">by {entry.author}</span>
            {#if entry.chapter_num}<span>ch. {entry.chapter_num}</span>{/if}
            <button onclick={() => removeEntry(entry.id)} disabled={removingId === entry.id}>Remove</button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .archive-history-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1em;
  }

  .archive-header {
    margin-bottom: 1em;
  }

  .archive-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 2em;
    color: var(--archive-heading, #171a23);
    margin: 0;
  }

  .archive-count {
    color: var(--archive-muted, #666);
    font-size: 0.9em;
    margin: 0 0 0.5em;
  }

  .history-toolbar {
    margin-top: 0.5em;
  }

  .loading-text, .error-text, .empty-text {
    padding: 1em;
    text-align: center;
  }

  .error-text {
    color: #c0392b;
  }

  .history-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .history-row {
    display: grid;
    grid-template-columns: 120px 1fr 120px auto;
    gap: 0.5em 1em;
    align-items: center;
    padding: 0.4em 0;
    border-bottom: 1px solid var(--archive-border, #eeeeee);
  }

  .history-row:hover {
    background: var(--archive-bg-hover, #f8f9fa);
  }

  .history-date {
    color: var(--archive-muted, #666);
    font-size: 0.85em;
    white-space: nowrap;
  }

  .history-title {
    text-decoration: none;
    color: var(--archive-link, #990000);
    font-weight: bold;
  }

  .history-title:hover {
    text-decoration: underline;
  }

  .history-author {
    color: var(--archive-muted, #666);
    font-size: 0.85em;
  }

  .history-chapter {
    color: var(--archive-muted, #666);
    font-size: 0.8em;
    font-style: italic;
  }

  .history-actions {
    display: flex;
    gap: 0.3em;
    align-items: center;
  }

  .archive-btn.small {
    font-size: 0.75em;
    padding: 0.2em 0.5em;
  }

  .archive-btn.small:active {
    background: var(--archive-btn-active, #e8dcc8);
  }

  .history-pagination {
    margin-top: 1em;
    display: flex;
    gap: 0.3em;
  }

  /* Modern mode minimal styles */
  .history-page {
    max-width: 800px;
    margin: 0 auto;
    padding: 2em;
  }

  .history-list li {
    display: flex;
    gap: 1em;
    align-items: center;
    padding: 0.4em 0;
    border-bottom: 1px solid #eee;
  }

  .history-list .date {
    color: #888;
    font-size: 0.85em;
    min-width: 120px;
  }

  .history-list a {
    color: #990000;
    text-decoration: none;
  }

  .history-list a:hover {
    text-decoration: underline;
  }

  @media (max-width: 600px) {
    .history-row {
      grid-template-columns: 1fr auto;
      grid-template-areas:
        "date actions"
        "title title"
        "author author"
        "chapter chapter";
      text-align: left;
    }
    .history-date { grid-area: date; }
    .history-actions { grid-area: actions; justify-self: end; }
    .history-title { grid-area: title; }
    .history-author { grid-area: author; }
    .history-chapter { grid-area: chapter; }
    .history-pagination {
      flex-wrap: wrap;
      gap: 0.2em;
    }
  }
</style>
