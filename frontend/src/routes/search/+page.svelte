<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { search, fetchSearchSuggestions, type SearchResult, type SearchFacets, type SearchSuggestion } from '$lib/api/search';
  import ArchiveWorkSearchForm from '$lib/ui/archive/ArchiveWorkSearchForm.svelte';
  import WorkBlurb from '$lib/ui/archive/WorkBlurb.svelte';
  import {
    buildSearchQuery as buildArchiveQuery,
    formStateToUrl,
    urlToFormState,
    defaultFormState,
    SUPPORTED,
    type FormState,
  } from '$lib/ui/archive/searchForm';
  import { auth } from '$lib/stores/auth.svelte';
  import { t } from '$lib/i18n/index.svelte';

  let results = $state<SearchResult[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state('');
  let searched = $state(false);
  let facets = $state<SearchFacets | null>(null);
  let suggestions = $state<SearchSuggestion[]>([]);

  // ── Archive form state ────────────────────────────────────────────────
  let archiveFormState = $state<FormState>(defaultFormState());
  let archiveCurrentPage = $state(1);

  onMount(() => {
    // Parse archive form state from URL params
    archiveFormState = urlToFormState($page.url.searchParams);
    archiveCurrentPage = Number($page.url.searchParams.get('page')) || 1;

    // If the URL has any search params, show results
    if ($page.url.searchParams.toString()) {
      doArchiveSearch();
    }
  });

  // ── Archive helpers ───────────────────────────────────────────────────

  /** Map ArchiveWorkSearchForm FormState to SearchFilters for the API. */
  function formStateToFilters(state: FormState, page: number = archiveCurrentPage): any {
    return {
      q: state.q,
      include_tags: state.include_tags,
      exclude_tags: state.exclude_tags,
      include_any_tags: '',
      exclude_tag_types: '',
      strict_gen: false,
      min_words: state.min_words ? Number(state.min_words) || null : null,
      max_words: state.max_words ? Number(state.max_words) || null : null,
      min_chapters: null,
      max_chapters: state.single_chapter ? 1 : null,
      complete: state.complete === 'true' ? true : state.complete === 'false' ? false : null,
      source: '',
      date_from: state.date_from,
      date_to: '',
      sort: state.sort,
      page,
      per_page: 20,
      primary_tag: '',
      relationship_characters: '',
      min_comments: state.min_comments ? Number(state.min_comments) || null : null,
      min_kudos: state.min_kudos ? Number(state.min_kudos) || null : null,
      no_warnings: state.no_warnings || null,
      tag_ids: state.tag_ids,
      rating: '',
      status: '',
      hide_read: false,
      hide_bookmarked: false,
      library_only: false,
    };
  }

  /** Execute an archive search using the current form state. */
  async function doArchiveSearch() {
    loading = true;
    error = '';
    searched = true;
    try {
      const res = await search(formStateToFilters(archiveFormState));
      results = res.results;
      total = res.total;
      facets = res.facets;
      try {
        const isLoggedIn = auth.isLoggedIn;
        suggestions = await fetchSearchSuggestions(isLoggedIn);
      } catch {
        suggestions = [];
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed';
      results = [];
      total = 0;
      facets = null;
      suggestions = [];
    } finally {
      loading = false;
    }
  }

  /** Called by ArchiveWorkSearchForm onSubmit: build URL, navigate, search. */
  function handleArchiveFormSubmit() {
    archiveCurrentPage = 1;
    const { params } = buildArchiveQuery(archiveFormState, SUPPORTED);
    goto('/search?' + params.toString());
    doArchiveSearch();
  }

  /** Called by ArchiveWorkSearchForm onChange: update form state. */
  function handleArchiveFormChange(newState: FormState) {
    archiveFormState = newState;
  }
</script>

<div class="search-page">
  <!-- Results -->
    <!-- Archive mode: AO3 Work Search -->
    {#if !archiveFormState.q}
      <!-- No active query: full-page search form -->
      <ArchiveWorkSearchForm
        mode="page"
        formState={archiveFormState}
        onChange={handleArchiveFormChange}
        onSubmit={handleArchiveFormSubmit}
      />
    {:else}
      <!-- Active query: results + sidebar -->
      <div class="archive-results-layout">
        <div class="archive-results-main">
          <h2 class="archive-results-count">{total.toLocaleString()} Works</h2>
          <p class="archive-query-echo">Search: <strong>{archiveFormState.q}</strong></p>
          <p class="edit-search"><a href="/search">Edit Your Search</a></p>

          {#if loading}
            <div class="archive-loading">Loading...</div>
          {:else if error}
            <div class="archive-error">{error}</div>
          {:else}
            {#if suggestions.length > 0}
              <div class="archive-suggest-row">
                <span class="archive-suggest-label">Refine:</span>
                {#each suggestions.slice(0, 8) as s (s.id)}
                  <button
                    class="archive-chip"
                    type="button"
                    onclick={() => {
                      const next = { ...archiveFormState };
                      const entry = `${s.tag_type_id}:${s.name}`;
                      if (s.tag_type_id === 1) next.fandoms = next.fandoms ? `${next.fandoms}, ${s.name}` : s.name;
                      else if (s.tag_type_id === 2) next.characters = next.characters ? `${next.characters}, ${s.name}` : s.name;
                      else if (s.tag_type_id === 3) next.relationships = next.relationships ? `${next.relationships}, ${s.name}` : s.name;
                      else if (s.tag_type_id === 4) next.additional_tags = next.additional_tags ? `${next.additional_tags}, ${s.name}` : s.name;
                      else next.include_tags = next.include_tags ? `${next.include_tags},${entry}` : entry;
                      archiveFormState = next;
                      handleArchiveFormSubmit();
                    }}
                  >
                    {s.name}
                  </button>
                {/each}
              </div>
            {/if}

            {#each results as r}
              <WorkBlurb fic={r as any} />
            {/each}

            {#if results.length === 0}
              <p class="archive-empty">No works found.</p>
            {/if}

            <!-- AO3-style pagination -->
            {#if total > 20}
              {@const totalPages = Math.ceil(total / 20)}
              {@const currentPage = archiveCurrentPage}
              {@const start = Math.max(1, currentPage - 3)}
              {@const end = Math.min(totalPages, start + 6)}
              {@const adjustedStart = Math.max(1, end - 6)}
              <nav class="archive-pagination" aria-label="Search results pages">
                {#if currentPage > 1}
                  <button
                    class="archive-page-link"
                    type="button"
                    onclick={() => {
                      const newPage = currentPage - 1;
                      archiveCurrentPage = newPage;
                      goto('/search?' + new URLSearchParams({ ...Object.fromEntries(formStateToUrl(archiveFormState)), page: String(newPage) }).toString());
                      doArchiveSearch();
                    }}
                  >
                    ← Previous
                  </button>
                {:else}
                  <span class="archive-page-disabled">← Previous</span>
                {/if}
                {#each Array.from({ length: end - adjustedStart + 1 }, (_, i) => adjustedStart + i) as p}
                  <button
                    class="archive-page-num"
                    class:active={p === currentPage}
                    type="button"
                    onclick={() => {
                      archiveCurrentPage = p;
                      goto('/search?' + new URLSearchParams({ ...Object.fromEntries(formStateToUrl(archiveFormState)), page: String(p) }).toString());
                      doArchiveSearch();
                    }}
                  >
                    {p}
                  </button>
                {/each}
                {#if currentPage < totalPages}
                  <button
                    class="archive-page-link"
                    type="button"
                    onclick={() => {
                      const newPage = currentPage + 1;
                      archiveCurrentPage = newPage;
                      goto('/search?' + new URLSearchParams({ ...Object.fromEntries(formStateToUrl(archiveFormState)), page: String(newPage) }).toString());
                      doArchiveSearch();
                    }}
                  >
                    Next →
                  </button>
                {:else}
                  <span class="archive-page-disabled">Next →</span>
                {/if}
              </nav>
            {/if}
          {/if}
        </div>

        <aside class="archive-sidebar">
          <ArchiveWorkSearchForm
            mode="sidebar"
            formState={archiveFormState}
            onChange={handleArchiveFormChange}
            onSubmit={handleArchiveFormSubmit}
          />
        </aside>
      </div>
  {/if}
</div>

<style>
  .search-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 1rem;
  }
  .subtitle {
    margin-top: -0.5rem;
    margin-bottom: 1rem;
    font-size: 0.9rem;
  }
  /* Three-tier mode switcher */
  .tier-switch {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 1rem;
    border-bottom: 2px solid var(--color-border);
  }
  .tier-tab {
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    padding: 0.4rem 0.9rem;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-muted);
    cursor: pointer;
  }
  .tier-tab:hover {
    color: var(--color-text);
  }
  .tier-tab.active {
    color: var(--color-primary);
    border-bottom-color: var(--color-primary);
  }
  .quick-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
  }
  .personal-filters {
    padding: 0.4rem 0.6rem;
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius);
  }
  .saved-filters { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; margin: 0.75rem 0; }
  .saved-filters form { display: flex; gap: 0.4rem; }
  .saved-filters input, .saved-filters select { min-height: 2rem; padding: 0.3rem 0.5rem; }
  .tag-autocomplete {
    position: relative;
  }
  .tag-autocomplete input {
    width: 100%;
  }
  .tag-ac-loading {
    position: absolute;
    right: 0.6rem;
    top: 0.5rem;
  }
  .tag-ac-list {
    position: absolute;
    z-index: 20;
    top: 100%;
    left: 0;
    right: 0;
    list-style: none;
    margin: 0.15rem 0 0;
    padding: 0.2rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    max-height: 220px;
    overflow-y: auto;
    box-shadow: 0 4px 14px rgb(0 0 0 / 0.25);
  }
  .tag-ac-list li {
    margin: 0;
  }
  .tag-ac-list button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    background: none;
    border: none;
    color: var(--color-text);
    text-align: left;
    padding: 0.35rem 0.5rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .tag-ac-list button:hover {
    background: var(--color-surface-2);
    color: var(--color-primary);
  }
  .tag-ac-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tag-ac-count {
    font-size: 0.72rem;
    color: var(--color-muted);
    flex-shrink: 0;
    background: var(--color-surface-2);
    border-radius: 10px;
    padding: 0.05rem 0.45rem;
  }
  .chip {
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: 20px;
    padding: 0.3rem 0.8rem;
    font-size: 0.82rem;
    cursor: pointer;
    transition: all 0.15s;
    color: var(--color-text);
  }
  .chip:hover {
    border-color: var(--color-primary);
  }
  .chip.active {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }
  .suggest-filters {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
    padding: 0.5rem 0.7rem;
    background: var(--color-surface);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius);
  }
  .suggest-label {
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-muted);
    margin-right: 0.2rem;
  }
  .suggest-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }
  .suggest-badge {
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.05rem 0.4rem;
    border-radius: 10px;
    background: var(--color-surface-2);
    color: var(--color-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .suggest-badge.for-you {
    background: color-mix(in srgb, var(--color-primary) 18%, transparent);
    color: var(--color-primary);
  }
  .clear-all {
    margin-left: auto;
    background: none;
    border-color: var(--color-error);
    color: var(--color-error);
  }
  .adv-toggle {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 0.88rem;
    margin-bottom: 0.6rem;
    padding: 0;
  }
  .adv-toggle:hover {
    color: var(--color-text);
  }
  .advanced-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.8rem;
    padding: 1rem;
    background: var(--color-surface-2);
    border-radius: var(--radius);
    border: 1px solid var(--color-border);
    margin-bottom: 0.8rem;
  }
  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }
  .filter-group.half {
    flex: 1;
  }
  .filter-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--color-text);
  }
  .hint {
    font-size: 0.75rem;
    color: var(--color-muted);
  }
  .filter-row {
    grid-column: 1 / -1;
    display: flex;
    gap: 0.8rem;
  }
  .toggle-row {
    flex-direction: row;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }
  .toggle {
    position: relative;
    display: inline-flex;
    align-items: center;
    cursor: pointer;
  }
  .toggle input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .toggle-slider {
    display: inline-block;
    width: 2.2rem;
    height: 1.2rem;
    border-radius: 1rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    position: relative;
    transition: background 0.15s;
  }
  .toggle-slider::after {
    content: '';
    position: absolute;
    top: 0.12rem;
    left: 0.14rem;
    width: 0.85rem;
    height: 0.85rem;
    border-radius: 50%;
    background: var(--color-muted);
    transition: transform 0.15s, background 0.15s;
  }
  .toggle input:checked + .toggle-slider {
    background: var(--color-primary);
  }
  .toggle input:checked + .toggle-slider::after {
    transform: translateX(0.95rem);
    background: white;
  }
  .filter-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-bottom: 1rem;
  }
  .results-header {
    font-size: 0.88rem;
    color: var(--color-muted);
    margin-bottom: 0.8rem;
  }
  .active-chips {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.4rem;
    margin-bottom: 0.8rem;
  }
  .active-chips-label {
    font-size: 0.82rem;
    color: var(--color-muted);
  }
  .filter-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    cursor: default;
  }
  .filter-chip-name {
    font-weight: 600;
  }
  .chip-remove {
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.1rem;
  }
  .chip-remove:hover {
    color: var(--color-primary);
  }
  .results-layout {
    display: grid;
    grid-template-columns: 1fr 260px;
    gap: 1.2rem;
    align-items: start;
  }
  .facets-sidebar {
    position: sticky;
    top: 1rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 0.6rem;
  }
  .facet-group {
    border-bottom: 1px solid var(--color-border);
    padding: 0.4rem 0;
  }
  .facet-group:last-child {
    border-bottom: none;
  }
  .facet-group summary {
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--color-text);
    padding: 0.2rem 0;
  }
  .facet-group summary:hover {
    color: var(--color-primary);
  }
  .facet-list {
    list-style: none;
    margin: 0.3rem 0 0;
    padding: 0;
    max-height: 240px;
    overflow-y: auto;
  }
  .facet-value {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    background: none;
    border: none;
    color: var(--color-text);
    cursor: pointer;
    padding: 0.25rem 0.3rem;
    border-radius: 6px;
    font-size: 0.82rem;
    text-align: left;
  }
  .facet-value:hover {
    background: var(--color-surface-1);
    color: var(--color-primary);
  }
  .facet-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .facet-count {
    font-size: 0.75rem;
    color: var(--color-muted);
    flex-shrink: 0;
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }
  .result-card h3 {
    margin: 0 0 0.3rem;
  }
  .result-card h3 a {
    color: var(--color-text);
    text-decoration: none;
  }
  .result-card h3 a:hover {
    color: var(--color-primary);
  }
  .meta {
    font-size: 0.88rem;
    color: var(--color-muted);
    margin: 0.3rem 0;
  }
  .desc {
    font-size: 0.88rem;
    color: var(--color-muted);
    margin: 0.4rem 0;
  }
  .desc.snippet :global(b) {
    color: var(--color-primary);
    font-weight: 700;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-top: 0.5rem;
  }
  .tag-chip {
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: 12px;
    padding: 0.15rem 0.5rem;
    font-size: 0.78rem;
    color: var(--color-muted);
  }
  .tag-more {
    font-size: 0.78rem;
    color: var(--color-muted);
    font-style: italic;
  }
  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    margin-top: 1.5rem;
    font-size: 0.88rem;
    color: var(--color-muted);
  }
  .no-results {
    text-align: center;
    padding: 2rem;
    font-size: 1rem;
  }
  .loading {
    text-align: center;
    padding: 2rem;
    color: var(--color-muted);
  }

  /* ── Archive mode: AO3 Work Search ────────────────────────────────────── */

  .archive-results-layout {
    display: grid;
    grid-template-columns: 1fr 260px;
    gap: 1.2rem;
    align-items: start;
  }

  .archive-results-main {
    min-width: 0;
  }

  .archive-results-count {
    font-family: Georgia, 'Times New Roman', serif;
    color: #8b0000;
    font-size: 1.6rem;
    margin: 0 0 0.4rem;
    font-weight: 700;
  }

  .archive-query-echo {
    font-size: 0.88rem;
    color: var(--archive-muted, #666);
    margin-bottom: 1rem;
  }

  .archive-query-echo strong {
    color: var(--archive-text, #2a2a2a);
  }

  .archive-loading,
  .archive-error,
  .archive-empty {
    text-align: center;
    padding: 2rem 1rem;
    font-size: 0.95rem;
    color: var(--archive-muted, #666);
    font-style: italic;
  }

  .archive-error {
    color: #990000;
    font-weight: 600;
    font-style: normal;
  }

  .archive-suggest-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.35rem;
    margin-bottom: 1rem;
    padding: 0.4rem 0.6rem;
    border: 1px dashed var(--archive-border, #ddd);
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .archive-suggest-label {
    font-size: 0.78rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--archive-muted, #666);
    margin-right: 0.15rem;
  }

  .archive-chip {
    display: inline-block;
    padding: 0.2em 0.55em;
    font-size: 0.82em;
    font-family: inherit;
    background: var(--archive-bg, #fff);
    border: 1px solid var(--archive-border, #ddd);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    color: var(--archive-text, #2a2a2a);
    line-height: 1.4;
    text-decoration: none;
    white-space: nowrap;
  }

  .archive-chip:hover {
    border-color: var(--archive-link, #990000);
    background: var(--archive-bg-raised, #f5f5f5);
  }

  .archive-sidebar {
    position: sticky;
    top: 1rem;
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #ddd);
    padding: 0.6rem;
  }

  .archive-pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.15rem;
    margin-top: 1.2rem;
    font-size: 0.88rem;
    padding: 0.6rem 0;
    border-top: 1px solid var(--archive-border, #ddd);
  }

  .archive-page-link,
  .archive-page-num,
  .archive-page-disabled {
    padding: 0.2em 0.5em;
    font-size: 0.88em;
    font-family: inherit;
    color: var(--archive-link, #990000);
    background: none;
    border: none;
    cursor: pointer;
    text-decoration: none;
    line-height: 1.4;
  }

  .archive-page-link:hover {
    text-decoration: underline;
  }

  .archive-page-num.active {
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg-raised, #f5f5f5);
    border: 1px solid var(--archive-border, #ddd);
    cursor: default;
  }

  .archive-page-disabled {
    color: var(--archive-muted, #666);
    opacity: 0.5;
    cursor: default;
  }

  .edit-search {
    font-size: 0.85em;
    margin: 0 0 0.5em;
  }

  .edit-search a {
    color: var(--archive-link, #990000);
    text-decoration: none;
  }

  .edit-search a:hover {
    text-decoration: underline;
  }

  @media (max-width: 600px) {
    .advanced-grid {
      grid-template-columns: 1fr;
    }
    .filter-row {
      flex-direction: column;
    }
    .results-layout {
      grid-template-columns: 1fr;
    }
    .facets-sidebar {
      position: static;
      order: -1;
    }
    .archive-results-layout {
      grid-template-columns: 1fr;
    }
    .archive-sidebar {
      position: static;
      order: -1;
    }
  }
</style>
