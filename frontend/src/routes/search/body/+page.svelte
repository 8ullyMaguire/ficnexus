<script lang="ts">
  import { onMount } from 'svelte';
  import { getPref } from '$lib/prefs';
  import { page } from '$app/stores';
  import { bodySearch, type BodySearchResult } from '$lib/api/search';
  import { formatWords, detectSite } from '$lib/util';

  const uiMode = $derived(getPref('uiMode'));

  let q = $state('');
  let results = $state<BodySearchResult[]>([]);
  let total = $state(0);
  let currentPage = $state(1);
  const perPage = 20;
  let loading = $state(false);
  let error = $state('');
  let searched = $state(false);

  // Read ?q= from the URL on mount and auto-search (mirrors /search).
  onMount(() => {
    const urlQ = $page.url.searchParams.get('q');
    if (urlQ) {
      q = urlQ;
      doSearch();
    }
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') doSearch();
  }

  async function doSearch() {
    const query = q.trim();
    if (!query) return;
    loading = true;
    error = '';
    searched = true;
    try {
      const res = await bodySearch(query, currentPage, perPage);
      results = res.results;
      total = res.total;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Body search failed';
      results = [];
      total = 0;
    } finally {
      loading = false;
    }
  }

  function nextPage() {
    currentPage++;
    doSearch();
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      doSearch();
    }
  }
</script>

{#if uiMode === 'archive'}
<!-- ── Archive mode ─────────────────────────────────────────── -->
<main class="archive-main">
  <div class="archive-content">
    <header class="archive-header">
      <h1 class="archive-page-title">Search inside fics</h1>
      <blockquote class="archive-summary">
        Find fics where the <em>body text</em> matches — “fics where X says Y”.
        Supports the same boolean syntax as the main search:
        <code>AND</code>/<code>OR</code>/<code>NOT</code>, <code>"quotes"</code>, <code>-exclude</code>.
      </blockquote>
    </header>

    <!-- Search form -->
    <form
      class="archive-form"
      onsubmit={(e) => { e.preventDefault(); doSearch(); }}
    >
      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Body text search</legend>
        <dl class="archive-dl">
          <div class="dl-row">
            <dt><label for="body-search-q">Query</label></dt>
            <dd><input id="body-search-q" class="archive-input" type="search" placeholder="e.g. Harry Potter AND wandless magic" aria-label="body search query" bind:value={q} /></dd>
          </div>
        </dl>
        <div class="archive-actions">
          <button class="archive-btn archive-btn-primary" type="submit" disabled={loading}>
            {#if loading}<span class="spinner"></span>{:else}Search{/if}
          </button>
          <a class="archive-btn" href="/search" title="Back to the metadata search">← Advanced search</a>
        </div>
      </fieldset>
    </form>

    {#if error}
      <p class="archive-error">{error}</p>
    {/if}

    {#if loading}
      <p class="archive-muted"><span class="spinner"></span> Searching…</p>
    {:else if results.length > 0}
      <p class="archive-muted count-line">{total} fic{total === 1 ? '' : 's'} matched</p>

      <fieldset class="archive-fieldset">
        <legend class="archive-legend">Results</legend>
        <ol class="archive-list">
          {#each results as r}
            <li class="archive-list-row result-row">
              <div class="result-main">
                <a class="archive-link archive-result-title" href="/works/{r.url_id}">{r.title}</a>
                <div class="archive-muted">
                  {r.author}
                  {#if detectSite(r.source)}
                    · {detectSite(r.source)}
                  {/if}
                </div>
                <div class="archive-meta-line">
                  {formatWords(r.words)} words · {r.chapters} ch · {r.status}
                </div>
                {#if r.body_snippet}
                  <!-- body_snippet is server-highlighted with <mark> tags -->
                  <div class="snippet">{@html r.body_snippet}</div>
                {:else if r.description}
                  <div class="snippet">{r.description}</div>
                {/if}
              </div>
            </li>
          {/each}
        </ol>
      </fieldset>

      <div class="archive-pagination">
        <button class="archive-btn" onclick={prevPage} disabled={currentPage <= 1}>← Previous</button>
        <span>Page {currentPage}</span>
        <button class="archive-btn" onclick={nextPage} disabled={results.length < perPage}>Next →</button>
      </div>
    {:else if searched}
      <p class="archive-muted no-results">No results found. Try different terms.</p>
    {/if}
  </div>
</main>
{:else}
<!-- Modern mode (unchanged) -->
<div class="body-search-page">
  <h1>Search inside fics</h1>
  <p class="muted subtitle">
    Find fics where the <em>body text</em> matches — “fics where X says Y”.
    Supports the same boolean syntax as the main search:
    <code>AND</code>/<code>OR</code>/<code>NOT</code>, <code>"quotes"</code>, <code>-exclude</code>.
  </p>

  <!-- Search bar (mirrors /search) -->
  <div class="search-bar">
    <input
      type="search"
      placeholder="e.g. Harry Potter AND wandless magic"
      aria-label="body search query"
      bind:value={q}
      onkeydown={onKeydown}
    />
    <button class="btn" onclick={doSearch} disabled={loading}>
      {#if loading}<span class="spinner"></span>{:else}Search{/if}
    </button>
    <a class="btn btn-secondary" href="/search" title="Back to the metadata search">← Advanced search</a>
  </div>

  {#if error}
    <div class="error-card">{error}</div>
  {/if}

  {#if loading}
    <div class="loading"><span class="spinner"></span> Searching…</div>
  {:else if results.length > 0}
    <p class="muted count-line">{total} fic{total === 1 ? '' : 's'} matched</p>
    <div class="results">
      {#each results as r}
        <div class="card result-card">
          <h3><a href="/works/{r.url_id}">{r.title}</a></h3>
          <p class="muted">{r.author} · <span class="tag">{detectSite(r.source)}</span></p>
          <p class="meta">
            {formatWords(r.words)} words · {r.chapters} ch · {r.status}
          </p>
          {#if r.body_snippet}
            <!-- body_snippet is server-highlighted with <mark> tags -->
            <p class="desc snippet">{@html r.body_snippet}</p>
          {:else}
            <p class="desc">{r.description}</p>
          {/if}
        </div>
      {/each}
    </div>

    <div class="pagination">
      <button class="btn btn-secondary" onclick={prevPage} disabled={currentPage <= 1}>← Previous</button>
      <span>Page {currentPage}</span>
      <button class="btn btn-secondary" onclick={nextPage} disabled={results.length < perPage}>Next →</button>
    </div>
  {:else if searched}
    <p class="muted no-results">No results found. Try different terms.</p>
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
    max-width: var(--archive-max-width, 900px);
    margin: 0 auto;
    padding: 1rem;
  }
  .archive-header {
    padding: 0 0 0.6rem;
    border-bottom: 2px solid var(--archive-link, #990000);
    margin-bottom: 1rem;
  }
  .archive-page-title {
    font-size: 1.6em;
    font-weight: 700;
    margin: 0 0 0.3rem;
  }
  .archive-summary {
    margin: 0.5rem 0 0;
    padding: 0.4rem 1rem;
    border-left: 3px solid var(--archive-border, #dddddd);
    color: var(--archive-muted, #666666);
    font-style: italic;
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-muted {
    color: var(--archive-muted, #666666);
    font-style: italic;
  }
  .archive-error {
    color: var(--archive-link, #990000);
    font-weight: 700;
    font-size: 0.9rem;
    margin-bottom: 0.8rem;
  }
  .archive-fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.6rem 0.9rem 0.75rem;
    margin: 0 0 1rem;
    background: var(--archive-bg, #ffffff);
  }
  .archive-fieldset legend,
  .archive-legend {
    font-weight: 700;
    font-size: 0.9em;
    color: var(--archive-link, #990000);
    padding: 0 0.4rem;
  }
  .archive-form { margin-bottom: 0.5rem; }
  .archive-dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.4rem 1rem;
    margin: 0.4rem 0 0.7rem;
  }
  .archive-dl dt {
    font-weight: 600;
    color: var(--archive-text, #2a2a2a);
  }
  .archive-dl dd { margin: 0; }
  .archive-input {
    padding: 0.35rem 0.55rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    font-family: inherit;
    font-size: 0.95rem;
    width: 100%;
    box-sizing: border-box;
    background: var(--archive-bg, #ffffff);
    color: var(--archive-text, #2a2a2a);
  }
  .archive-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .archive-btn {
    padding: 0.28rem 0.8rem;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-text, #2a2a2a);
    font-family: inherit;
    font-size: 0.88rem;
    text-decoration: none;
    cursor: pointer;
    display: inline-block;
  }
  .archive-btn:hover { background: var(--archive-border, #dddddd); }
  .archive-btn:disabled { opacity: 0.5; cursor: not-allowed; }
  .archive-btn-primary {
    color: var(--archive-bg, #ffffff);
    background: var(--archive-link, #990000);
    border-color: var(--archive-link, #990000);
  }
  .archive-btn-primary:hover { background: var(--archive-link-visited, #660066); border-color: var(--archive-link-visited, #660066); }
  .count-line { font-size: 0.88rem; margin: 0 0 0.75rem; }
  .archive-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border-top: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row.result-row {
    display: block;
    padding: 0.65rem 0.4rem;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }
  .archive-list-row.result-row:nth-child(odd) {
    background: var(--archive-bg-raised, #f5f5f5);
  }
  .archive-link {
    color: var(--archive-link, #990000);
    text-decoration: none;
    font-weight: 600;
  }
  .archive-link:hover { text-decoration: underline; }
  .archive-result-title {
    font-size: 1em;
    font-weight: 700;
  }
  .archive-meta-line {
    font-size: 0.82rem;
    color: var(--archive-muted, #666666);
    margin-top: 0.15rem;
  }
  .snippet {
    background: var(--archive-bg-raised, #faf7f0);
    border-left: 3px solid var(--archive-border, #dddddd);
    padding: 0.4rem 0.6rem;
    margin-top: 0.35rem;
    font-size: 0.88rem;
    line-height: 1.5;
  }
  .snippet :global(mark) {
    background: #ffe08a;
    color: inherit;
    padding: 0 1px;
  }
  .archive-pagination {
    display: flex;
    align-items: center;
    gap: 1rem;
    justify-content: center;
    margin-top: 1rem;
  }
  .no-results {
    text-align: center;
    padding: 2rem;
  }

  /* ── Modern mode (unchanged) ── */
  .body-search-page {
    max-width: 860px;
    margin: 0 auto;
    padding: 1rem;
  }
  .subtitle {
    margin-bottom: 1rem;
  }
  .search-bar {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }
  .search-bar input {
    flex: 1;
    min-width: 200px;
    padding: 0.5rem;
    font-size: 1rem;
    border: 1px solid var(--border, #ccc);
    border-radius: 6px;
    background: var(--bg-input, #fff);
    color: var(--fg, inherit);
  }
  .count-line {
    margin-bottom: 0.75rem;
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .card {
    border: 1px solid var(--border, #ddd);
    border-radius: 8px;
    padding: 0.9rem 1rem;
    background: var(--bg-card, #fff);
  }
  .card h3 {
    margin: 0 0 0.25rem;
  }
  .card h3 a {
    text-decoration: none;
  }
  .muted {
    opacity: 0.75;
    margin: 0.15rem 0;
  }
  .meta {
    font-size: 0.85rem;
    opacity: 0.7;
    margin: 0.25rem 0;
  }
  .desc {
    margin-top: 0.5rem;
  }
  .snippet {
    background: var(--bg-snippet, #faf7f0);
    border-radius: 6px;
    padding: 0.5rem 0.6rem;
    font-size: 0.9rem;
    line-height: 1.5;
  }
  .tag {
    background: var(--bg-tag, #eee);
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    font-size: 0.8rem;
  }
  .loading {
    padding: 1.5rem;
    text-align: center;
    opacity: 0.7;
  }
  .spinner {
    display: inline-block;
    width: 1em;
    height: 1em;
    border: 2px solid var(--border, #ccc);
    border-top-color: var(--fg, #333);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    vertical-align: -0.1em;
    margin-right: 0.4em;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .error-card {
    background: #fdecea;
    border: 1px solid #f5c6c6;
    color: #a33;
    border-radius: 8px;
    padding: 0.75rem 1rem;
    margin-bottom: 1rem;
  }
  .pagination {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-top: 1rem;
    justify-content: center;
  }
  .no-results {
    text-align: center;
    padding: 2rem;
  }
</style>
