<!--
  Tier 1 (Simple) search: one free-text box fed to q= plus the
  "Interpreted as" chip row. Also carries the surprise-me + body-search
  links that were historically part of the search bar.
-->
<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { onMount } from 'svelte';
  import type { SearchFilters } from '$lib/api/search';
  import InterpretedChips from './InterpretedChips.svelte';

  let {
    filters = $bindable<SearchFilters>(),
    onSearch,
    onFacetChange,
  }: {
    filters: SearchFilters;
    onSearch: () => void;
    onFacetChange: () => void;
  } = $props();

  const SEARCH_DEBOUNCE_MS = 700;
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

  onMount(() => {
    return () => {
      if (searchDebounce) clearTimeout(searchDebounce);
    };
  });

  function onSearchInput() {
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => onFacetChange(), SEARCH_DEBOUNCE_MS);
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      if (searchDebounce) clearTimeout(searchDebounce);
      searchDebounce = null;
      onSearch();
    }
  }

  function handleChipsChange(q: string) {
    filters.q = q;
    onFacetChange();
  }
</script>

<div class="search-bar">
  <input
    type="search"
    placeholder={t('search.placeholder')}
    bind:value={filters.q}
    onkeydown={onKeydown}
    oninput={onSearchInput}
  />
  <button class="btn" onclick={onSearch}>
    {t('search.button')}
  </button>
  <a class="btn btn-secondary surprise-btn" href="/blind-date" title={t('search.surpriseTitle')}>
    {t('search.surpriseMe')}
  </a>
  <a class="btn btn-secondary body-search-btn" href="/search/body" title={t('search2.bodySearchTitle')}>
    {t('search2.bodySearch')}
  </a>
</div>

<InterpretedChips q={filters.q} onChange={handleChipsChange} />

<style>
  .search-bar {
    display: flex;
    gap: 0.6rem;
    margin-bottom: 0.8rem;
  }
  .search-bar input {
    flex: 1;
  }
  .surprise-btn {
    text-decoration: none;
    white-space: nowrap;
    align-self: center;
  }
  .body-search-btn {
    text-decoration: none;
    white-space: nowrap;
    align-self: center;
  }
</style>
