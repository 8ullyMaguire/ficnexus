# Expansion Material: Extended Walkthroughs and Practice Projects

*Complete project walkthroughs, detailed code explanations, and hands-on exercises.*

---

## Complete Project: Building a FicHub-Inspired Search Interface

### Overview

In this walkthrough, we'll build a complete search interface from scratch. This combines everything we've learned: Svelte 5 runes, TypeScript, API communication, component composition, state management, and accessibility.

### Step 1: Define the Data Types

```typescript
// lib/types/search.ts
export interface SearchResult {
  id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: 'complete' | 'in-progress' | 'abandoned';
  site: string;
  tags: string[];
  summary: string;
  rating: 'general' | 'teen' | 'mature' | 'explicit';
  updated: string;
}

export interface SearchFilters {
  query: string;
  site: string;
  status: string;
  minWords: number | null;
  maxWords: number | null;
  rating: string;
  sortBy: 'relevance' | 'words' | 'updated' | 'title';
  page: number;
  perPage: number;
}

export interface SearchResponse {
  results: SearchResult[];
  total: number;
  page: number;
  perPage: number;
}
```

### Step 2: Build the API Client

```typescript
// lib/api/searchClient.ts
import type { SearchFilters, SearchResponse } from '$lib/types/search';

const BASE = '/api/v0';

export class SearchError extends Error {
  constructor(public status: number, message: string) {
    super(message);
    this.name = 'SearchError';
  }
}

export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const params = new URLSearchParams();
  
  if (filters.query) params.set('q', filters.query);
  if (filters.site) params.set('site', filters.site);
  if (filters.status) params.set('status', filters.status);
  if (filters.minWords) params.set('min_words', String(filters.minWords));
  if (filters.maxWords) params.set('max_words', String(filters.maxWords));
  if (filters.rating) params.set('rating', filters.rating);
  if (filters.sortBy !== 'relevance') params.set('sort', filters.sortBy);
  if (filters.page > 1) params.set('page', String(filters.page));
  if (filters.perPage !== 20) params.set('per_page', String(filters.perPage));

  const qs = params.toString();
  const res = await fetch(`${BASE}/search${qs ? '?' + qs : ''}`);
  
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new SearchError(res.status, text || `Search failed (${res.status})`);
  }
  
  return res.json();
}
```

### Step 3: Build the Search State

```typescript
// lib/state/searchState.ts
import type { SearchFilters, SearchResult } from '$lib/types/search';
import { search, SearchError } from '$lib/api/searchClient';

export function createSearchState() {
  let filters = $state<SearchFilters>({
    query: '',
    site: '',
    status: '',
    minWords: null,
    maxWords: null,
    rating: '',
    sortBy: 'relevance',
    page: 1,
    perPage: 20
  });

  let results = $state<SearchResult[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Derived values
  let totalPages = $derived(Math.ceil(total / filters.perPage));
  let hasResults = $derived(results.length > 0);
  let hasQuery = $derived(filters.query.trim().length > 0);
  let activeFilters = $derived(
    [filters.site, filters.status, filters.minWords, filters.maxWords, filters.rating]
      .filter(Boolean).length
  );

  async function executeSearch() {
    loading = true;
    error = null;
    
    try {
      const response = await search(filters);
      results = response.results;
      total = response.total;
    } catch (e) {
      if (e instanceof SearchError) {
        error = `Search failed: ${e.message}`;
      } else {
        error = 'Network error. Please try again.';
      }
      results = [];
      total = 0;
    } finally {
      loading = false;
    }
  }

  function updateFilter<K extends keyof SearchFilters>(
    key: K, 
    value: SearchFilters[K]
  ) {
    filters[key] = value;
    filters.page = 1; // Reset to first page on filter change
  }

  function nextPage() {
    if (filters.page < totalPages) {
      filters.page++;
      executeSearch();
    }
  }

  function prevPage() {
    if (filters.page > 1) {
      filters.page--;
      executeSearch();
    }
  }

  function resetFilters() {
    filters = {
      query: '',
      site: '',
      status: '',
      minWords: null,
      maxWords: null,
      rating: '',
      sortBy: 'relevance',
      page: 1,
      perPage: 20
    };
  }

  return {
    get filters() { return filters; },
    get results() { return results; },
    get total() { return total; },
    get loading() { return loading; },
    get error() { return error; },
    get totalPages() { return totalPages; },
    get hasResults() { return hasResults; },
    get hasQuery() { return hasQuery; },
    get activeFilters() { return activeFilters; },
    executeSearch,
    updateFilter,
    nextPage,
    prevPage,
    resetFilters
  };
}
```

### Step 4: Build the Search Components

```svelte
<!-- SearchBar.svelte -->
<script lang="ts">
  import { debounce } from '$lib/util/debounce';
  
  let { 
    value = $bindable(''), 
    onSearch,
    loading = false 
  } = $props();

  let localValue = $state(value);
  
  const debouncedSearch = debounce(() => {
    value = localValue;
    onSearch?.();
  }, 300);

  function handleInput(e: Event) {
    localValue = (e.target as HTMLInputElement).value;
    debouncedSearch();
  }

  function handleSubmit(e: Event) {
    e.preventDefault();
    value = localValue;
    onSearch?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      localValue = '';
      value = '';
      onSearch?.();
    }
  }
</script>

<form class="search-bar" onsubmit={handleSubmit} role="search">
  <div class="search-input-wrapper">
    <span class="search-icon" aria-hidden="true">🔍</span>
    <input
      type="search"
      value={localValue}
      oninput={handleInput}
      onkeydown={handleKeydown}
      placeholder="Search fanfiction... (e.g., fandom:Harry Potter words:>50000)"
      aria-label="Search fanfiction"
      disabled={loading}
    />
    {#if localValue}
      <button 
        type="button" 
        class="clear-btn" 
        onclick={() => { localValue = ''; value = ''; onSearch?.(); }}
        aria-label="Clear search"
      >
        ✕
      </button>
    {/if}
  </div>
  <button type="submit" disabled={loading || !localValue.trim()}>
    {loading ? 'Searching...' : 'Search'}
  </button>
</form>

<style>
  .search-bar {
    display: flex;
    gap: 0.5rem;
    width: 100%;
    max-width: 720px;
  }
  
  .search-input-wrapper {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }
  
  .search-icon {
    position: absolute;
    left: 0.75rem;
    font-size: 1.1rem;
    pointer-events: none;
  }
  
  input {
    width: 100%;
    padding: 0.75rem 2.5rem 0.75rem 2.5rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 1rem;
  }
  
  input:focus {
    outline: none;
    border-color: var(--color-primary);
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
  }
  
  .clear-btn {
    position: absolute;
    right: 0.5rem;
    background: none;
    border: none;
    color: var(--color-muted);
    cursor: pointer;
    padding: 0.25rem;
    font-size: 1rem;
  }
  
  .clear-btn:hover {
    color: var(--color-text);
  }
  
  button[type="submit"] {
    padding: 0.75rem 1.5rem;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: var(--radius);
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
  
  button[type="submit"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  @media (max-width: 600px) {
    .search-bar {
      flex-direction: column;
    }
  }
</style>
```

```svelte
<!-- SearchFilters.svelte -->
<script lang="ts">
  import type { SearchFilters } from '$lib/types/search';
  
  let { 
    filters, 
    onFilterChange, 
    onReset 
  } = $props();

  const sites = [
    { value: '', label: 'All Sites' },
    { value: 'archiveofourown.org', label: 'Archive of Our Own' },
    { value: 'fanfiction.net', label: 'FanFiction.net' },
    { value: 'fictionpress.com', label: 'FictionPress' },
    { value: 'forums.spacebattles.com', label: 'SpaceBattles' },
    { value: 'forums.sufficientvelocity.com', label: 'SufficientVelocity' }
  ];

  const statuses = [
    { value: '', label: 'All Statuses' },
    { value: 'complete', label: 'Complete' },
    { value: 'in-progress', label: 'In Progress' },
    { value: 'abandoned', label: 'Abandoned' }
  ];

  const ratings = [
    { value: '', label: 'All Ratings' },
    { value: 'general', label: 'General' },
    { value: 'teen', label: 'Teen' },
    { value: 'mature', label: 'Mature' },
    { value: 'explicit', label: 'Explicit' }
  ];

  const sortOptions = [
    { value: 'relevance', label: 'Relevance' },
    { value: 'words', label: 'Word Count' },
    { value: 'updated', label: 'Last Updated' },
    { value: 'title', label: 'Title' }
  ];

  function handleChange(key: string, value: any) {
    onFilterChange(key, value);
  }
</script>

<div class="filters">
  <div class="filter-row">
    <label>
      Site
      <select 
        value={filters.site} 
        onchange={(e) => handleChange('site', e.currentTarget.value)}
      >
        {#each sites as site}
          <option value={site.value}>{site.label}</option>
        {/each}
      </select>
    </label>

    <label>
      Status
      <select 
        value={filters.status} 
        onchange={(e) => handleChange('status', e.currentTarget.value)}
      >
        {#each statuses as status}
          <option value={status.value}>{status.label}</option>
        {/each}
      </select>
    </label>

    <label>
      Rating
      <select 
        value={filters.rating} 
        onchange={(e) => handleChange('rating', e.currentTarget.value)}
      >
        {#each ratings as rating}
          <option value={rating.value}>{rating.label}</option>
        {/each}
      </select>
    </label>

    <label>
      Sort By
      <select 
        value={filters.sortBy} 
        onchange={(e) => handleChange('sortBy', e.currentTarget.value)}
      >
        {#each sortOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>
  </div>

  <div class="filter-row">
    <label>
      Min Words
      <input 
        type="number" 
        value={filters.minWords ?? ''} 
        oninput={(e) => handleChange('minWords', e.currentTarget.value ? Number(e.currentTarget.value) : null)}
        placeholder="0"
        min="0"
      />
    </label>

    <label>
      Max Words
      <input 
        type="number" 
        value={filters.maxWords ?? ''} 
        oninput={(e) => handleChange('maxWords', e.currentTarget.value ? Number(e.currentTarget.value) : null)}
        placeholder="∞"
        min="0"
      />
    </label>

    <button class="reset-btn" onclick={onReset}>
      Reset Filters
    </button>
  </div>
</div>

<style>
  .filters {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    padding: 1rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
  }
  
  .filter-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
    align-items: end;
  }
  
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--color-muted);
  }
  
  select, input[type="number"] {
    padding: 0.5rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface-2);
    color: var(--color-text);
    min-width: 120px;
  }
  
  .reset-btn {
    padding: 0.5rem 1rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-muted);
    cursor: pointer;
    font-size: 0.85rem;
  }
  
  .reset-btn:hover {
    background: var(--color-surface-3);
    color: var(--color-text);
  }
</style>
```

```svelte
<!-- SearchResultCard.svelte -->
<script lang="ts">
  import type { SearchResult } from '$lib/types/search';
  import { formatWords, relativeTime, detectSite } from '$lib/util';
  
  let { result } = $props();
  
  let siteLabel = $derived(detectSite(result.site));
  
  const statusColors: Record<string, string> = {
    'complete': 'var(--color-success)',
    'in-progress': 'var(--color-warning)',
    'abandoned': 'var(--color-error)'
  };
  
  const ratingColors: Record<string, string> = {
    'general': '#10b981',
    'teen': '#f59e0b',
    'mature': '#ef4444',
    'explicit': '#dc2626'
  };
</script>

<article class="result-card">
  <div class="result-header">
    <h3>
      <a href={`/story/${result.id}`}>{result.title}</a>
    </h3>
    <span class="site-badge">{siteLabel}</span>
  </div>
  
  <p class="author">by {result.author}</p>
  
  <div class="meta">
    <span class="words">{formatWords(result.words)} words</span>
    <span class="separator">·</span>
    <span class="chapters">{result.chapters} chapters</span>
    <span class="separator">·</span>
    <span class="status" style="color: {statusColors[result.status]}">
      {result.status}
    </span>
    <span class="separator">·</span>
    <span class="rating" style="color: {ratingColors[result.rating]}">
      {result.rating}
    </span>
  </div>
  
  {#if result.summary}
    <p class="summary">{result.summary.slice(0, 250)}{result.summary.length > 250 ? '...' : ''}</p>
  {/if}
  
  {#if result.tags.length > 0}
    <div class="tags">
      {#each result.tags.slice(0, 5) as tag}
        <span class="tag">{tag}</span>
      {/each}
      {#if result.tags.length > 5}
        <span class="tag-more">+{result.tags.length - 5} more</span>
      {/if}
    </div>
  {/if}
  
  <div class="result-footer">
    <span class="updated">Updated {relativeTime(result.updated)}</span>
    <div class="actions">
      <button class="btn-secondary">Details</button>
      <button class="btn-primary">Download</button>
    </div>
  </div>
</article>

<style>
  .result-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    padding: 1.25rem;
    transition: border-color 0.2s ease, box-shadow 0.2s ease;
  }
  
  .result-card:hover {
    border-color: var(--color-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }
  
  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  
  .result-header h3 {
    margin: 0;
    font-size: 1.1rem;
  }
  
  .result-header h3 a {
    color: var(--color-text);
    text-decoration: none;
  }
  
  .result-header h3 a:hover {
    color: var(--color-primary);
  }
  
  .site-badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-sm);
    color: var(--color-muted);
    white-space: nowrap;
  }
  
  .author {
    color: var(--color-muted);
    font-size: 0.9rem;
    margin: 0 0 0.75rem;
  }
  
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    font-size: 0.85rem;
    color: var(--color-muted);
    margin-bottom: 0.75rem;
  }
  
  .separator {
    opacity: 0.5;
  }
  
  .summary {
    color: var(--color-muted);
    font-size: 0.9rem;
    line-height: 1.5;
    margin: 0 0 0.75rem;
  }
  
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    margin-bottom: 0.75rem;
  }
  
  .tag {
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    background: var(--color-surface-2);
    border-radius: var(--radius-full);
    color: var(--color-muted);
  }
  
  .tag-more {
    font-size: 0.75rem;
    color: var(--color-primary);
  }
  
  .result-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border);
  }
  
  .updated {
    font-size: 0.8rem;
    color: var(--color-muted);
  }
  
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .btn-secondary, .btn-primary {
    padding: 0.4rem 0.8rem;
    border-radius: var(--radius-sm);
    font-size: 0.85rem;
    cursor: pointer;
    border: none;
  }
  
  .btn-secondary {
    background: var(--color-surface-2);
    color: var(--color-text);
    border: 1px solid var(--color-border);
  }
  
  .btn-primary {
    background: var(--color-primary);
    color: white;
  }
</style>
```

```svelte
<!-- Pagination.svelte -->
<script lang="ts">
  let { 
    currentPage, 
    totalPages, 
    onPageChange 
  } = $props();

  let pages = $derived.by(() => {
    const pages: (number | '...')[] = [];
    const delta = 2;
    
    for (let i = 1; i <= totalPages; i++) {
      if (
        i === 1 || 
        i === totalPages || 
        (i >= currentPage - delta && i <= currentPage + delta)
      ) {
        pages.push(i);
      } else if (pages[pages.length - 1] !== '...') {
        pages.push('...');
      }
    }
    
    return pages;
  });
</script>

{#if totalPages > 1}
  <nav class="pagination" aria-label="Search results pages">
    <button 
      class="page-btn"
      disabled={currentPage === 1}
      onclick={() => onPageChange(currentPage - 1)}
      aria-label="Previous page"
    >
      ← Prev
    </button>
    
    {#each pages as page}
      {#if page === '...'}
        <span class="page-dots">...</span>
      {:else}
        <button
          class="page-btn"
          class:active={page === currentPage}
          onclick={() => onPageChange(page)}
          aria-label="Page {page}"
          aria-current={page === currentPage ? 'page' : undefined}
        >
          {page}
        </button>
      {/if}
    {/each}
    
    <button 
      class="page-btn"
      disabled={currentPage === totalPages}
      onclick={() => onPageChange(currentPage + 1)}
      aria-label="Next page"
    >
      Next →
    </button>
  </nav>
{/if}

<style>
  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.25rem;
    padding: 1rem;
  }
  
  .page-btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-surface);
    color: var(--color-text);
    cursor: pointer;
    font-size: 0.9rem;
  }
  
  .page-btn:hover:not(:disabled) {
    background: var(--color-surface-2);
  }
  
  .page-btn.active {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }
  
  .page-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  
  .page-dots {
    padding: 0.5rem;
    color: var(--color-muted);
  }
</style>
```

### Step 5: Assemble the Search Page

```svelte
<!-- routes/search/+page.svelte -->
<script lang="ts">
  import SearchBar from '$lib/components/SearchBar.svelte';
  import SearchFilters from '$lib/components/SearchFilters.svelte';
  import SearchResultCard from '$lib/components/SearchResultCard.svelte';
  import Pagination from '$lib/components/Pagination.svelte';
  import { createSearchState } from '$lib/state/searchState';

  const searchState = createSearchState();

  function handleSearch() {
    searchState.executeSearch();
  }

  function handleFilterChange(key: string, value: any) {
    searchState.updateFilter(key, value);
    searchState.executeSearch();
  }

  function handlePageChange(page: number) {
    searchState.filters.page = page;
    searchState.executeSearch();
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }
</script>

<div class="search-page">
  <header class="search-header">
    <h1>Search Fanfiction</h1>
    <p class="muted">
      Find stories across Archive of Our Own, FanFiction.net, and more.
    </p>
  </header>

  <SearchBar 
    bind:value={searchState.filters.query}
    onSearch={handleSearch}
    loading={searchState.loading}
  />

  <SearchFilters 
    filters={searchState.filters}
    onFilterChange={handleFilterChange}
    onReset={() => { searchState.resetFilters(); searchState.executeSearch(); }}
  />

  {#if searchState.error}
    <div class="error-card">
      <p>⚠️ {searchState.error}</p>
      <button onclick={handleSearch}>Try Again</button>
    </div>
  {/if}

  {#if searchState.loading}
    <div class="loading">
      <span class="spinner"></span>
      <p>Searching...</p>
    </div>
  {:else if searchState.hasResults}
    <div class="results-header">
      <p>{searchState.total.toLocaleString()} results found</p>
    </div>
    
    <div class="results-list">
      {#each searchState.results as result (result.id)}
        <SearchResultCard {result} />
      {/each}
    </div>
    
    <Pagination 
      currentPage={searchState.filters.page}
      totalPages={searchState.totalPages}
      onPageChange={handlePageChange}
    />
  {:else if searchState.hasQuery}
    <div class="empty-state">
      <p>No results found. Try different search terms or filters.</p>
    </div>
  {/if}
</div>

<style>
  .search-page {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .search-header {
    text-align: center;
  }
  
  .search-header h1 {
    margin: 0 0 0.5rem;
  }
  
  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem;
  }
  
  .spinner {
    width: 2rem;
    height: 2rem;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .results-header {
    color: var(--color-muted);
    font-size: 0.9rem;
  }
  
  .results-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  
  .error-card {
    padding: 1rem;
    background: var(--color-surface);
    border: 1px solid var(--color-error);
    border-radius: var(--radius);
    color: var(--color-error);
  }
  
  .empty-state {
    text-align: center;
    padding: 3rem;
    color: var(--color-muted);
  }
</style>
```

---

## Complete Project: Building a Rating Component

### Overview

Build a star rating component that supports:
- Display mode (read-only)
- Interactive mode (click to rate)
- Half-star ratings
- Keyboard navigation
- Accessible with screen readers

### The Rating Component

```svelte
<!-- Rating.svelte -->
<script lang="ts">
  interface Props {
    value?: number;
    maxStars?: number;
    size?: 'sm' | 'md' | 'lg';
    interactive?: boolean;
    onRate?: (value: number) => void;
  }

  let {
    value = $bindable(0),
    maxStars = 5,
    size = 'md',
    interactive = false,
    onRate
  }: Props = $props();

  let hoverValue = $state(0);
  let isHovering = $state(false);

  let displayValue = $derived(isHovering ? hoverValue : value);

  function getStarFill(starIndex: number): 'full' | 'half' | 'empty' {
    const starPosition = starIndex + 1;
    if (displayValue >= starPosition) return 'full';
    if (displayValue >= starPosition - 0.5) return 'half';
    return 'empty';
  }

  function handleClick(starIndex: number) {
    if (!interactive) return;
    
    const newValue = starIndex + 1;
    if (newValue === value) {
      // Clicking the same star resets to 0
      value = 0;
    } else {
      value = newValue;
    }
    onRate?.(value);
  }

  function handleMouseEnter(starIndex: number) {
    if (!interactive) return;
    hoverValue = starIndex + 1;
    isHovering = true;
  }

  function handleMouseLeave() {
    isHovering = false;
  }

  function handleKeydown(e: KeyboardEvent, starIndex: number) {
    if (!interactive) return;
    
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowUp':
        e.preventDefault();
        if (starIndex < maxStars - 1) {
          value = starIndex + 2;
          onRate?.(value);
        }
        break;
      case 'ArrowLeft':
      case 'ArrowDown':
        e.preventDefault();
        if (starIndex > 0) {
          value = starIndex;
          onRate?.(value);
        }
        break;
      case ' ':
      case 'Enter':
        e.preventDefault();
        handleClick(starIndex);
        break;
    }
  }
</script>

<div 
  class="rating rating-{size}"
  class:interactive
  role={interactive ? 'radiogroup' : 'img'}
  aria-label={interactive ? 'Rate this story' : `Rating: ${value} out of ${maxStars} stars`}
  on:mouseleave={handleMouseLeave}
>
  {#each Array(maxStars) as _, i}
    <button
      class="star"
      class:full={getStarFill(i) === 'full'}
      class:half={getStarFill(i) === 'half'}
      class:empty={getStarFill(i) === 'empty'}
      disabled={!interactive}
      onclick={() => handleClick(i)}
      onmouseenter={() => handleMouseEnter(i)}
      onkeydown={(e) => handleKeydown(e, i)}
      aria-label="{i + 1} star{i > 0 ? 's' : ''}"
      tabindex={interactive ? 0 : -1}
    >
      <svg viewBox="0 0 24 24" fill="currentColor">
        <defs>
          <linearGradient id="half-{i}">
            <stop offset="50%" stop-color="currentColor" />
            <stop offset="50%" stop-color="transparent" />
          </linearGradient>
        </defs>
        {#if getStarFill(i) === 'full'}
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
        {:else if getStarFill(i) === 'half'}
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" 
                fill="url(#half-{i})" />
        {:else}
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" 
                fill="none" stroke="currentColor" stroke-width="1.5" />
        {/if}
      </svg>
    </button>
  {/each}
  
  {#if value > 0}
    <span class="rating-value">{value.toFixed(1)}</span>
  {/if}
</div>

<style>
  .rating {
    display: inline-flex;
    align-items: center;
    gap: 0.1rem;
  }
  
  .rating.sm { font-size: 1rem; }
  .rating.md { font-size: 1.25rem; }
  .rating.lg { font-size: 1.5rem; }
  
  .star {
    background: none;
    border: none;
    padding: 0;
    cursor: default;
    color: var(--color-muted);
    transition: color 0.15s ease, transform 0.15s ease;
  }
  
  .interactive .star {
    cursor: pointer;
  }
  
  .interactive .star:hover {
    transform: scale(1.1);
  }
  
  .star.full {
    color: #fbbf24;
  }
  
  .star.half {
    color: #fbbf24;
  }
  
  .star.empty {
    color: var(--color-border);
  }
  
  .star:focus-visible {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
    border-radius: 2px;
  }
  
  .rating-value {
    margin-left: 0.5rem;
    font-size: 0.9rem;
    color: var(--color-muted);
    font-weight: 500;
  }
</style>
```

### Usage Examples

```svelte
<!-- Display mode -->
<Rating value={4.5} />

<!-- Interactive mode -->
<Rating bind:value={userRating} interactive onRate={handleRate} />

<!-- Different sizes -->
<Rating value={3} size="sm" />
<Rating value={4} size="md" />
<Rating value={5} size="lg" />
```

---

## Extended Practice Projects

### Project 1: Build a Complete Todo Application

Build a todo application with the following features:

**Core Features:**
- Add new todos
- Mark todos as complete
- Delete todos
- Edit todo text
- Filter by status (all, active, completed)

**Advanced Features:**
- Drag and drop to reorder
- Categories/tags
- Due dates
- Priority levels
- Search
- Keyboard shortcuts
- Undo/redo

**Technical Requirements:**
- Use Svelte 5 runes
- TypeScript for all types
- localStorage persistence
- Unit tests for utility functions
- Component tests for UI
- Accessible with keyboard and screen readers

### Project 2: Build a Bookmark Manager

Build a browser bookmark manager:

**Features:**
- Save bookmarks with title, URL, description, tags
- Organize into folders
- Search and filter
- Import/export bookmarks
- Sort by date, title, frequency
- Preview URLs

**Technical Requirements:**
- SvelteKit with file-based routing
- TypeScript interfaces for all data
- API client for backend communication
- Form validation
- Responsive design
- Dark/light theme

### Project 3: Build a Code Snippet Manager

Build a code snippet manager for developers:

**Features:**
- Save code snippets with syntax highlighting
- Organize by language and tags
- Search across all snippets
- Copy to clipboard
- Share snippets via URL
- Version history

**Technical Requirements:**
- Syntax highlighting (use Prism.js or highlight.js)
- Clipboard API integration
- URL-based sharing with encoded state
- Responsive layout
- Keyboard shortcuts

### Project 4: Build a Recipe App

Build a recipe management application:

**Features:**
- Add recipes with ingredients, steps, and photos
- Scale servings (auto-recalculate ingredients)
- Timer for cooking steps
- Shopping list generation
- Nutritional information
- Print-friendly view

**Technical Requirements:**
- Service worker for offline access
- Push notifications for timers
- Print CSS for recipe cards
- Responsive images
- Schema.org structured data

### Project 5: Build a Personal Finance Tracker

Build a personal finance application:

**Features:**
- Record income and expenses
- Categorize transactions
- Budget tracking
- Monthly/weekly reports
- Charts and graphs
- Export to CSV

**Technical Requirements:**
- Chart.js or D3.js for visualizations
- Date manipulation
- Currency formatting
- Data normalization
- Complex filtering and aggregation
- Responsive data tables

---

## Extended CSS Reference

### Flexbox Patterns

```css
/* Centering */
.center { display: flex; justify-content: center; align-items: center; }
.center-v { display: flex; align-items: center; }
.center-h { display: flex; justify-content: center; }

/* Spacing */
.between { display: flex; justify-content: space-between; }
.around { display: flex; justify-content: space-around; }
.evenly { display: flex; justify-content: space-evenly; }

/* Wrapping */
.wrap { display: flex; flex-wrap: wrap; gap: 1rem; }

/* Equal widths */
.equal { display: flex; gap: 1rem; }
.equal > * { flex: 1; }

/* Sidebar layout */
.layout { display: flex; gap: 1.5rem; }
.sidebar { flex: 0 0 250px; }
.content { flex: 1; }

/* Sticky footer */
body { display: flex; flex-direction: column; min-height: 100vh; }
main { flex: 1; }
```

### Grid Patterns

```css
/* Simple grid */
.grid { display: grid; gap: 1rem; }

/* Auto-fill responsive */
.auto-fill { 
  display: grid; 
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); 
  gap: 1rem;
}

/* Named areas */
.layout {
  display: grid;
  grid-template-areas:
    "header header"
    "sidebar main"
    "footer footer";
  grid-template-columns: 200px 1fr;
  grid-template-rows: auto 1fr auto;
}

/* Spanning columns */
.span-2 { grid-column: span 2; }
.span-full { grid-column: 1 / -1; }
```

### Responsive Patterns

```css
/* Mobile-first */
.container {
  width: 100%;
  padding: 1rem;
}

@media (min-width: 640px) {
  .container { max-width: 640px; margin: 0 auto; }
}

@media (min-width: 768px) {
  .container { max-width: 768px; }
}

@media (min-width: 1024px) {
  .container { max-width: 1024px; }
}

/* Fluid */
.fluid {
  width: min(100% - 2rem, 720px);
  margin-inline: auto;
  padding: clamp(1rem, 3vw, 2rem);
}
```

### Animation Patterns

```css
/* Fade in */
@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

/* Slide up */
@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Scale in */
@keyframes scaleIn {
  from { opacity: 0; transform: scale(0.9); }
  to { opacity: 1; transform: scale(1); }
}

/* Spin */
@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Pulse */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Reduced motion */
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## Extended TypeScript Reference

### Utility Types

```typescript
// Make all properties optional
Partial<T>

// Make all properties required
Required<T>

// Make all properties readonly
Readonly<T>

// Pick specific properties
Pick<T, 'key1' | 'key2'>

// Omit specific properties
Omit<T, 'key1' | 'key2'>

// Record of key-value pairs
Record<string, T>

// Extract union members
Extract<T, U>

// Exclude union members
Exclude<T, U>

// Get function return type
ReturnType<typeof fn>

// Get function parameter types
Parameters<typeof fn>

// Make properties nullable
Nullable<T> = { [K in keyof T]: T[K] | null }

// Deep partial
DeepPartial<T> = { [K in keyof T]?: DeepPartial<T[K]> }
```

### Type Guards

```typescript
// Instance of
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

// Discriminated union
function isSuccess<T>(state: State<T>): state is { status: 'success'; data: T } {
  return state.status === 'success';
}

// Custom type guard
function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError && 'status' in error;
}
```

### Generics

```typescript
// Generic function
function first<T>(arr: T[]): T | undefined {
  return arr[0];
}

// Generic interface
interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

// Generic constraint
function getProperty<T, K extends keyof T>(obj: T, key: K): T[K] {
  return obj[key];
}

// Conditional type
type IsString<T> = T extends string ? true : false;
```
