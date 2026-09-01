<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getPref } from '$lib/prefs';
  import { searchAuthors, type AuthorSummary } from '$lib/api/authors';
  import ArchiveButton from '$lib/ui/archive/ArchiveButton.svelte';

  const uiMode = $derived(getPref('uiMode'));

  let query = $state('');
  let results = $state<AuthorSummary[]>([]);
  let loading = $state(false);
  let searched = $state(false);

  onMount(async () => {
    const q = $page.url.searchParams.get('q');
    if (q) {
      query = q;
      await doSearch();
    }
  });

  async function doSearch() {
    const q = query.trim();
    if (!q) return;
    loading = true;
    searched = true;
    try {
      results = await searchAuthors(q);
    } catch {
      results = [];
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') doSearch();
  }
</script>

{#if uiMode === 'archive'}
  <!-- AO3-style People Search -->
  <div class="archive-authors">
    <h2 class="archive-heading">People Search</h2>

    <div class="archive-search-row">
      <input
        type="search"
        placeholder="Search authors by name…"
        bind:value={query}
        onkeydown={onKeydown}
        aria-label="Search authors"
        class="archive-input"
      />
      <ArchiveButton onclick={doSearch} disabled={loading}>
        {#if loading}Searching…{:else}Search{/if}
      </ArchiveButton>
    </div>

    {#if loading}
      <ul class="archive-results">
        {#each Array(3) as _}
          <li class="archive-skeleton">&#8203;</li>
        {/each}
      </ul>
    {:else if searched && results.length === 0}
      <p class="archive-empty">No authors found.</p>
    {:else if results.length > 0}
      <p class="archive-count">{results.length} result{results.length !== 1 ? 's' : ''}</p>
      <ul class="archive-results">
        {#each results as a (a.id)}
          <li>
            <a class="archive-link" href={`/authors/${a.id}`}>{a.canonical_name}</a>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{:else}
  <!-- Modern UI -->
  <div class="authors-page">
    <h1>Authors</h1>
    <p class="muted">
      Search curated author profiles. Each profile links together the pseudonyms and
      accounts that belong to the same author.
    </p>

    <div class="search-row">
      <input
        type="search"
        placeholder="Search authors by name or pseud…"
        bind:value={query}
        onkeydown={onKeydown}
        aria-label="Search authors"
      />
      <button class="btn" onclick={doSearch} disabled={loading}>
        {#if loading}<span class="spinner"></span> Searching…{:else}Search{/if}
      </button>
    </div>

    {#if loading}
      <div class="card"><p class="muted"><span class="spinner"></span> Loading authors…</p></div>
    {:else if searched && results.length === 0}
      <div class="card"><p class="muted">No authors found.</p></div>
    {:else if searched && results.length > 0}
      <div class="results">
        {#each results as a (a.id)}
          <a class="card author-card" href={`/authors/${a.id}`}>
            <div class="author-main">
              <strong>{a.canonical_name}</strong>
              {#if a.bio}
                <p class="muted author-bio">{a.bio}</p>
              {/if}
            </div>
            <span class="linked-count">
              {a.linked_authors} linked {a.linked_authors === 1 ? 'account' : 'accounts'}
            </span>
          </a>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  /* ── Archive mode ──────────────────────────────────────────────── */
  .archive-authors {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
  }

  .archive-heading {
    font-family: Georgia, 'Times New Roman', serif;
    font-size: 1.6em;
    font-weight: 700;
    color: #990000;
    margin: 0 0 0.5em;
    padding-bottom: 0.3em;
    border-bottom: 2px solid #990000;
  }

  .archive-search-row {
    display: flex;
    gap: 0.5rem;
    margin: 1em 0;
  }

  .archive-input {
    flex: 1;
    padding: 0.35em 0.6em;
    font-size: 0.95em;
    font-family: inherit;
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    background: var(--archive-bg-raised, #f9f9f9);
    color: var(--archive-text, #2a2a2a);
  }

  .archive-input:focus {
    outline: 1px solid #990000;
    border-color: #990000;
  }

  .archive-count {
    font-size: 0.88em;
    color: var(--archive-muted, #666666);
    margin: 0 0 0.5em;
  }

  .archive-empty {
    font-size: 0.95em;
    color: var(--archive-muted, #666666);
    padding: 2em 0;
    text-align: center;
  }

  .archive-results {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--archive-border, #dddddd);
    background: var(--archive-bg-raised, #f9f9f9);
  }

  .archive-results li {
    padding: 0.4em 0.8em;
    border-bottom: 1px solid var(--archive-border, #dddddd);
  }

  .archive-results li:last-child {
    border-bottom: none;
  }

  .archive-link {
    color: #990000;
    text-decoration: none;
    font-size: 0.92em;
  }

  .archive-link:hover {
    text-decoration: underline;
  }

  .archive-skeleton {
    height: 1.6em;
    background: linear-gradient(90deg, #eee 25%, #e0e0e0 50%, #eee 75%);
    background-size: 200% 100%;
    animation: shimmer 1.2s infinite;
  }

  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  /* ── Modern mode ───────────────────────────────────────────────── */
  .authors-page {
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 1rem;
  }

  .search-row {
    display: flex;
    gap: 0.5rem;
    margin: 1rem 0;
  }

  .search-row input {
    flex: 1;
    padding: 0.5rem 0.7rem;
    font-size: 0.95rem;
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .author-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.8rem 1rem;
    text-decoration: none;
    color: var(--color-text);
  }

  .author-card:hover {
    text-decoration: none;
    background: var(--color-surface-2);
  }

  .author-bio {
    font-size: 0.85rem;
    margin: 0.25rem 0 0;
    max-width: 60ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .linked-count {
    font-size: 0.82rem;
    color: var(--color-muted);
    white-space: nowrap;
  }

  /* ── Mobile ────────────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .archive-authors {
      padding: 0.75rem;
    }

    .archive-search-row {
      flex-direction: column;
    }
  }
</style>
