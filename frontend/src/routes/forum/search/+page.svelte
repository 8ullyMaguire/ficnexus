<script lang="ts">
  // Forum full-text search (F4): q input + optional category filter, POSTs
  // to /api/forum/search (auth-gated). Results carry server-generated
  // ts_headline snippets with <mark> around matches — rendered via {@html}
  // after DOMPurify sanitization (belt-and-braces on server output).
  import { onMount } from 'svelte';
  import { getForumCategories, searchForum, type ForumCategory, type ForumSearchResult } from '$lib/api/forum';
  import DOMPurify from 'dompurify';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getPref } from '$lib/prefs';

  const uiMode = $derived(getPref('uiMode'));

  let q = $state('');
  let categorySlug = $state('');
  let categories = $state<ForumCategory[]>([]);
  let loadingCategories = $state(true);
  let searching = $state(false);
  let searched = $state(false);
  let error = $state('');
  let results = $state<ForumSearchResult[]>([]);
  let total = $state(0);
  let lastQuery = $state('');

  function sanitizeSnippet(snippet: string): string {
    return DOMPurify.sanitize(snippet, { USE_PROFILES: { html: true } });
  }

  async function loadCategories() {
    loadingCategories = true;
    try {
      const res = await getForumCategories();
      if (res.err === 0) {
        categories = res.items ?? [];
      }
    } catch {
      // Non-fatal: search still works without the category dropdown.
    } finally {
      loadingCategories = false;
    }
  }

  async function doSearch() {
    const query = q.trim();
    if (!query) return;
    searching = true;
    error = '';
    try {
      const res = await searchForum(query, categorySlug || null);
      if (res.err === 0) {
        results = res.results ?? [];
        total = res.total ?? res.results?.length ?? 0;
        lastQuery = query;
        searched = true;
      } else {
        error = res.msg ?? t('forum.searchError');
      }
    } catch {
      error = t('forum.searchError');
    } finally {
      searching = false;
    }
  }

  onMount(async () => {
    await auth.init();
    await loadCategories();
  });

  function resultLabel(count: number): string {
    return count === 1 ? t('forum.searchResult', { count, q: lastQuery }) : t('forum.searchResults', { count, q: lastQuery });
  }

  function typeLabel(type: ForumSearchResult['type']): string {
    return type === 'topic' ? t('forum.searchTopic') : t('forum.searchPost');
  }
</script>

<svelte:head><title>{t('forum.searchTitle')} — FicNexus</title></svelte:head>

{#if uiMode === 'archive'}
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">{t('forum.searchTitle')}</h1>
      <p class="archive-summary">{t('forum.searchSubtitle')}</p>
      <p class="archive-backlink"><a class="archive-link" href="/forum">&larr; {t('forum.back')}</a></p>
    </header>

    <form class="archive-search-form" onsubmit={(e) => { e.preventDefault(); doSearch(); }}>
      <fieldset class="archive-fieldset">
        <legend>{t('forum.searchTitle')}</legend>
        <dl class="archive-dl">
          <dt><label for="forum-search-q">{t('forum.searchPlaceholder')}</label></dt>
          <dd><input id="forum-search-q" class="archive-input" type="search" name="q" bind:value={q} placeholder={t('forum.searchPlaceholder')} aria-label={t('forum.searchPlaceholder')} /></dd>
          <dt><label for="forum-search-cat">{t('forum.searchCategory')}</label></dt>
          <dd>
            <select id="forum-search-cat" class="archive-select" bind:value={categorySlug} aria-label={t('forum.searchCategory')} disabled={loadingCategories || categories.length === 0}>
              <option value="">{t('forum.searchAllCategories')}</option>
              {#each categories as c (c.id)}
                <option value={c.slug}>{c.title}</option>
              {/each}
            </select>
          </dd>
        </dl>
        <div class="archive-form-actions">
          <button class="archive-btn" type="submit" disabled={searching || !q.trim()}>
            {searching ? t('forum.searching') : t('forum.searchButton')}
          </button>
        </div>
      </fieldset>
    </form>

    {#if error}
      <div class="archive-error"><strong>&#9888; {error}</strong></div>
    {:else if searched && results.length === 0}
      <blockquote class="archive-empty">
        <p>{t('forum.searchNoResults', { q: lastQuery })}</p>
        <p>{t('forum.searchNoResultsHint')}</p>
      </blockquote>
    {:else if results.length > 0}
      <p class="archive-result-count">{resultLabel(total)}</p>
      <ul class="archive-results">
        {#each results as r (r.post_id ? `post-${r.post_id}` : `topic-${r.topic_id}`)}
          <li class="archive-result">
            <span class="archive-result-type">{typeLabel(r.type)}</span>
            <a class="archive-link archive-result-title" href={r.topic_slug ? `/forum/board/${r.topic_slug}.${r.topic_id}` : `/forum/${r.category_slug}/${r.topic_id}`}>{r.title}</a>
            <span class="archive-muted archive-result-meta">
              {t('forum.searchBy', { author: r.author_username ?? '&mdash;' })} &middot; {r.category_title} &middot; {(r.created_at ?? '').slice(0, 10)}
            </span>
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            <div class="archive-snippet">{@html sanitizeSnippet(r.snippet)}</div>
          </li>
        {/each}
      </ul>
    {:else if !searched}
      <blockquote class="archive-empty"><p>{t('forum.searchHint')}</p></blockquote>
    {/if}
  </div>
</main>
{:else}
<div class="search-page">
  <header class="page-head">
    <a class="back" href="/forum">{t('forum.back')}</a>
    <h1>{t('forum.searchTitle')}</h1>
    <p class="subtitle">{t('forum.searchSubtitle')}</p>
  </header>

  <form class="search-form" onsubmit={(e) => { e.preventDefault(); doSearch(); }}>
    <div class="search-row">
      <input
        type="search"
        name="q"
        bind:value={q}
        placeholder={t('forum.searchPlaceholder')}
        aria-label={t('forum.searchPlaceholder')}
      />
      <select bind:value={categorySlug} aria-label={t('forum.searchCategory')} disabled={loadingCategories || categories.length === 0}>
        <option value="">{t('forum.searchAllCategories')}</option>
        {#each categories as c (c.id)}
          <option value={c.slug}>{c.title}</option>
        {/each}
      </select>
      <button class="btn btn-primary" type="submit" disabled={searching || !q.trim()}>
        {searching ? t('forum.searching') : t('forum.searchButton')}
      </button>
    </div>
  </form>

  {#if error}
    <div class="error-card"><strong>⚠️ {error}</strong></div>
  {:else if searched && results.length === 0}
    <p class="empty">{t('forum.searchNoResults', { q: lastQuery })}</p>
    <p class="empty-hint">{t('forum.searchNoResultsHint')}</p>
  {:else if results.length > 0}
    <p class="result-count">{resultLabel(total)}</p>
    <ul class="result-list">
      {#each results as r (r.post_id ? `post-${r.post_id}` : `topic-${r.topic_id}`)}
        <li class="card result-card">
          <div class="row">
            <span class="type-chip">{typeLabel(r.type)}</span>
            <a class="title" href={r.topic_slug ? `/forum/board/${r.topic_slug}.${r.topic_id}` : `/forum/${r.category_slug}/${r.topic_id}`}>{r.title}</a>
          </div>
          <span class="meta">
            {t('forum.searchBy', { author: r.author_username ?? '—' })} · {r.category_title} · {(r.created_at ?? '').slice(0, 10)}
          </span>
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="snippet">{@html sanitizeSnippet(r.snippet)}</div>
        </li>
      {/each}
    </ul>
  {:else if !searched}
    <p class="hint">{t('forum.searchHint')}</p>
  {/if}
</div>
{/if}

<style>

  /* ── Archive mode ─────────────────────────────────────────────── */
  .archive-main {
    max-width: var(--archive-max-width, 1100px);
    margin: 0 auto;
    padding: 0;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-content {
    max-width: 820px;
    margin: 0 auto;
    padding: 1rem;
    font-family: Georgia, 'Times New Roman', serif;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-header {
    margin-bottom: 1.25rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
    padding-bottom: 0.6em;
  }
  .archive-page-title {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.8em;
    font-weight: 700;
    color: var(--archive-heading, #990000);
    margin: 0 0 0.15em;
  }
  .archive-summary {
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
    margin: 0.2em 0 0;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 1em 1.25em;
    margin: 0 0 1.25rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend {
    font-family: Georgia, 'Times New Roman', serif;
    font-weight: 700;
    font-size: 1.05em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.4em;
  }
  .archive-dl { margin: 0; padding: 0; }
  .archive-dl dt {
    font-weight: 700;
    font-size: 0.9em;
    margin: 0.85em 0 0.2em;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dt:first-child { margin-top: 0; }
  .archive-dl dd { margin: 0; }
  .archive-input, .archive-select, .archive-textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 0.5em 0.6em;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95em;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-textarea { resize: vertical; }
  .archive-btn {
    display: inline-block;
    padding: 0.25em 0.8em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    cursor: pointer;
    text-decoration: none;
  }
  .archive-btn:hover:not(:disabled) {
    background: var(--archive-border, #dddddd);
    border-color: var(--archive-muted, #666666);
  }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-form-actions { margin-top: 0.75em; display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
  .archive-link { color: var(--archive-link, #990000); }
  .archive-muted { color: var(--archive-muted, #666666); }
  .archive-error {
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
    padding: 0.8em 1em;
    font-weight: 700;
    margin: 0.5rem 0;
  }
  .archive-ok { color: #2e7d32; font-size: 0.92em; margin: 0.4em 0; }
  .archive-empty {
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.6em 1em;
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
    margin: 1rem 0;
  }
  .archive-empty p { margin: 0.2em 0; }
  .archive-backlink { font-size: 0.9em; margin: 0.4em 0 0; }

  .archive-result-count { color: var(--archive-muted, #666666); font-size: 0.9em; margin: 0 0 0.75rem; }
  .archive-results { list-style: none; margin: 0; padding: 0; border-top: 1px solid var(--archive-border, #dddddd); }
  .archive-result { padding: 0.6rem 0.4rem; border-bottom: 1px solid var(--archive-border, #dddddd); display: flex; flex-direction: column; gap: 0.25rem; }
  .archive-result:nth-child(odd) { background: var(--archive-bg-raised, #f5f5f5); }
  .archive-result-type { font-size: 0.78em; text-transform: uppercase; letter-spacing: 0.04em; color: var(--archive-muted, #666666); border: 1px solid var(--archive-border, #dddddd); padding: 0.05em 0.4em; align-self: flex-start; }
  .archive-result-title { font-weight: 700; }
  .archive-result-meta { font-size: 0.85em; }
  .archive-snippet { line-height: 1.5; overflow-wrap: break-word; font-size: 0.92em; }
  .archive-snippet :global(mark) { background: #ffe58a; padding: 0 0.1rem; }
  .search-page { max-width: 820px; margin: 0 auto; padding: 1.5rem; }
  .page-head { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1.25rem; }
  .back { color: var(--color-link, #2b4bd7); text-decoration: none; font-size: 0.9rem; }
  .subtitle { color: var(--color-text-muted, #888); }
  .search-form { margin-bottom: 1.25rem; }
  .search-row { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  .search-row input { flex: 1 1 240px; padding: 0.5rem 0.75rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px; font: inherit; }
  .search-row select { padding: 0.5rem 0.6rem; border: 1px solid var(--color-border, #ddd); border-radius: 8px; font: inherit; background: var(--color-surface, #fff); }
  .result-count { color: var(--color-text-muted, #888); font-size: 0.9rem; margin: 0 0 0.75rem; }
  .result-list { list-style: none; display: flex; flex-direction: column; gap: 0.75rem; padding: 0; }
  .result-card { padding: 1rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .row { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .title { font-size: 1.05rem; font-weight: 600; color: var(--color-link, #2b4bd7); text-decoration: none; }
  .meta { color: var(--color-text-muted, #888); font-size: 0.85rem; }
  .type-chip { background: var(--color-chip-bg, #eef); padding: 0.15rem 0.5rem; border-radius: 999px; font-size: 0.75rem; white-space: nowrap; }
  .snippet { line-height: 1.5; overflow-wrap: break-word; }
  .snippet :global(mark) { background: #ffe58a; padding: 0 0.1rem; border-radius: 2px; }
  .hint { color: var(--color-text-muted, #888); }
  .empty { text-align: center; color: var(--color-text-muted, #888); padding: 3rem 0 0; }
  .empty-hint { text-align: center; color: var(--color-text-muted, #888); }
</style>
