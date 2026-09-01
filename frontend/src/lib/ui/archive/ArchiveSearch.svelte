<script lang="ts">
  import type { SearchResult, SearchFilters } from '$lib/api/search';
  import ArchiveFilters from './ArchiveFilters.svelte';
  import WorkBlurb from './WorkBlurb.svelte';

  let {
    results = [],
    total = 0,
    page = 1,
    perPage = 20,
    query = '',
    filters,
    loading = false,
    onApplyFilters,
    onClearFilters,
    onNavigate,
  }: {
    results?: SearchResult[];
    total?: number;
    page?: number;
    perPage?: number;
    query?: string;
    filters: SearchFilters;
    loading?: boolean;
    onApplyFilters: () => void;
    onClearFilters: () => void;
    onNavigate: (page: number) => void;
  } = $props();

  const totalPages = $derived(Math.ceil(total / perPage) || 1);
  const startResult = $derived(total === 0 ? 0 : (page - 1) * perPage + 1);
  const endResult = $derived(Math.min(page * perPage, total));

  // Build page numbers to show (AO3 style: first few, current ± 2, last few)
  const visiblePages = $derived.by(() => {
    const pages: (number | '...')[] = [];
    const maxVisible = 7;
    if (totalPages <= maxVisible) {
      for (let i = 1; i <= totalPages; i++) pages.push(i);
    } else {
      pages.push(1);
      const rangeStart = Math.max(2, page - 2);
      const rangeEnd = Math.min(totalPages - 1, page + 2);
      if (rangeStart > 2) pages.push('...');
      for (let i = rangeStart; i <= rangeEnd; i++) pages.push(i);
      if (rangeEnd < totalPages - 1) pages.push('...');
      pages.push(totalPages);
    }
    return pages;
  });
</script>

<div class="archive-search">
  <!-- Left: Filters sidebar -->
  <ArchiveFilters
    {filters}
    {loading}
    onApply={onApplyFilters}
    onClear={onClearFilters}
  />

  <!-- Right: Results -->
  <div class="archive-results">
    <!-- Header -->
    <div class="results-header">
      <h2>
        {#if total > 0}
          Found <span class="count">{total.toLocaleString()}</span>
          {total === 1 ? 'work' : 'works'}
          {#if query}
            for <span class="query">'{query}'</span>
          {/if}
        {:else if query}
          No results found for <span class="query">'{query}'</span>
        {:else}
          Enter a search query
        {/if}
      </h2>
      {#if total > 0}
        <p class="results-meta">
          Showing {startResult.toLocaleString()}–{endResult.toLocaleString()} of {total.toLocaleString()}
        </p>
      {/if}
    </div>

    <!-- Loading -->
    {#if loading}
      <div class="loading-state">
        <span class="spinner"></span> Searching…
      </div>
    {/if}

    <!-- Results list -->
    {#if !loading && results.length > 0}
      <div class="results-list">
        {#each results as r (r.url_id)}
          <WorkBlurb fic={r as any} />
        {/each}
      </div>

      <!-- Pagination -->
      {#if totalPages > 1}
        <nav class="pagination" aria-label="Search results pages">
          <button
            class="page-btn"
            disabled={page <= 1}
            onclick={() => onNavigate(page - 1)}
          >
            ← Previous
          </button>

          {#each visiblePages as p}
            {#if p === '...'}
              <span class="page-ellipsis">…</span>
            {:else}
              <button
                class="page-num"
                class:active={p === page}
                onclick={() => onNavigate(p)}
                disabled={p === page}
              >
                {p}
              </button>
            {/if}
          {/each}

          <button
            class="page-btn"
            disabled={page >= totalPages}
            onclick={() => onNavigate(page + 1)}
          >
            Next →
          </button>
        </nav>
      {/if}
    {/if}

    <!-- No results -->
    {#if !loading && query && total === 0}
      <div class="no-results">
        <p>We couldn't find any works matching your search.</p>
        <p class="no-results-hint">Try different keywords or adjust your filters.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .archive-search {
    display: flex;
    gap: 1.5em;
    align-items: flex-start;
  }

  .archive-results {
    flex: 1;
    min-width: 0;
  }

  .results-header h2 {
    font-size: 1.15em;
    font-weight: 400;
    color: var(--archive-text, #2a2a2a);
    margin: 0 0 0.3em;
    line-height: 1.4;
    font-family: Georgia, 'Times New Roman', serif;
  }

  .results-header .count {
    font-weight: 700;
  }

  .results-header .query {
    color: var(--archive-link, #990000);
    font-style: italic;
  }

  .results-meta {
    font-size: 0.85em;
    color: var(--archive-muted, #666666);
    margin: 0 0 1em;
  }

  .loading-state {
    text-align: center;
    padding: 2em;
    color: var(--archive-muted, #666666);
    font-size: 0.95em;
  }

  .spinner {
    display: inline-block;
    width: 1em;
    height: 1em;
    border: 2px solid var(--archive-border, #dddddd);
    border-top-color: var(--archive-link, #990000);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
    vertical-align: middle;
    margin-right: 0.3em;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .results-list {
    display: flex;
    flex-direction: column;
  }

  /* ── AO3-style pagination ── */
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.2em;
    margin-top: 1.5em;
    padding: 0.75em 0;
    border-top: 1px solid var(--archive-border, #dddddd);
    font-size: 0.9em;
    font-family: Georgia, 'Times New Roman', serif;
  }

  .page-btn {
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
    padding: 0.25em 0.7em;
    font-family: inherit;
    font-size: 1em;
    color: var(--archive-link, #990000);
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  .page-btn:hover:not(:disabled) {
    color: #900;
    border-top-color: #999;
    border-left-color: #999;
    box-shadow: inset 2px 2px 2px #bbb;
  }

  .page-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    color: var(--archive-muted, #666666);
  }

  .page-num {
    display: inline-block;
    min-width: 2em;
    text-align: center;
    padding: 0.25em 0.5em;
    border: 1px solid transparent;
    background: none;
    font-family: inherit;
    font-size: 1em;
    color: #444;
    cursor: pointer;
    text-decoration: none;
    border-bottom: none;
    transition: background 0.15s, color 0.15s;
  }

  /* AO3 pagination = .action buttons: hover lifts ink red + inset shadow */
  .page-num:hover {
    color: #900;
    border-top: 1px solid #999;
    border-left: 1px solid #999;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    box-shadow: inset 2px 2px 2px #bbb;
  }

  /* AO3 .current: pressed-in look (#ccc box, inset shadow) */
  .page-num.active {
    font-weight: 700;
    color: #111;
    background: #ccc;
    border-color: #fff;
    box-shadow: inset 1px 1px 3px #333;
    text-decoration: none;
    cursor: default;
  }

  .page-ellipsis {
    padding: 0 0.3em;
    color: var(--archive-muted, #666666);
  }

  /* ── No results ── */
  .no-results {
    text-align: center;
    padding: 2em 1em;
    color: var(--archive-text, #2a2a2a);
    font-family: Georgia, 'Times New Roman', serif;
  }

  .no-results p {
    margin: 0.3em 0;
  }

  .no-results-hint {
    font-size: 0.9em;
    color: var(--archive-muted, #666666);
  }

  /* ── Responsive ── */
  @media (max-width: 700px) {
    .archive-search {
      flex-direction: column;
    }

    :global(.archive-filters) {
      width: 100% !important;
    }
  }
</style>
