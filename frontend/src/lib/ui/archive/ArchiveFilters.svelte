<script lang="ts">
  import type { SearchFilters, TagAutocompleteHit } from '$lib/api/search';
  import { fetchTagAutocomplete } from '$lib/api/search';

  let {
    filters,
    loading = false,
    onApply,
    onClear,
  }: {
    filters: SearchFilters;
    loading?: boolean;
    onApply: () => void;
    onClear: () => void;
  } = $props();

  // ── Local state copied from parent filters ──
  let sort = $state(filters.sort);
  let complete = $state(filters.complete);
  let minWords = $state(filters.min_words);
  let maxWords = $state(filters.max_words);
  let minKudos = $state(filters.min_kudos);

  // Rating checkboxes (AO3 has 5: Not Rated, General, Teen, Mature, Explicit)
  let ratings = $state<string[]>([]);
  const RATING_OPTIONS = ['Not Rated', 'General', 'Teen', 'Mature', 'Explicit'];

  // Tag autocomplete
  let includeTags = $state(filters.include_tags);
  let excludeTags = $state(filters.exclude_tags);
  let activeTagField = $state<string | null>(null);
  let tagSuggestions = $state<TagAutocompleteHit[]>([]);
  let tagLoading = $state(false);
  let tagSearchSeq = 0;

  function syncToFilters() {
    filters.sort = sort;
    filters.complete = complete;
    filters.min_words = minWords;
    filters.max_words = maxWords;
    filters.min_kudos = minKudos;
    filters.include_tags = includeTags;
    filters.exclude_tags = excludeTags;
  }

  function handleApply() {
    syncToFilters();
    onApply();
  }

  function handleClear() {
    sort = '';
    complete = null;
    minWords = null;
    maxWords = null;
    minKudos = null;
    ratings = [];
    includeTags = '';
    excludeTags = '';
    syncToFilters();
    onClear();
  }

  // ── Tag autocomplete ──
  function onTagInput(field: string) {
    const raw = field === 'include_tags' ? includeTags : excludeTags;
    const lastPart = raw.split(',').pop()?.trim() ?? '';
    const seq = ++tagSearchSeq;
    if (lastPart.length < 2) {
      tagSuggestions = [];
      tagLoading = false;
      return;
    }
    tagLoading = true;
    let tagType: number | undefined;
    const colon = lastPart.indexOf(':');
    if (colon > 0) {
      const t = Number(lastPart.slice(0, colon));
      if (Number.isInteger(t) && t >= 1 && t <= 7) tagType = t;
    }
    setTimeout(async () => {
      if (seq !== tagSearchSeq) return;
      try {
        const hits = await fetchTagAutocomplete(lastPart.replace(/^\d+:/, ''), tagType);
        if (seq !== tagSearchSeq) return;
        tagSuggestions = hits;
      } catch {
        tagSuggestions = [];
      } finally {
        if (seq === tagSearchSeq) tagLoading = false;
      }
    }, 250);
  }

  function applyTagSuggestion(field: string, hit: TagAutocompleteHit) {
    const raw = field === 'include_tags' ? includeTags : excludeTags;
    const parts = raw.split(',').map((p) => p.trim()).filter(Boolean);
    parts[parts.length - 1] = `${hit.type_id}:${hit.name}`;
    const joined = parts.join(', ');
    if (field === 'include_tags') {
      includeTags = joined;
    } else {
      excludeTags = joined;
    }
    tagSuggestions = [];
    activeTagField = null;
  }
</script>

<aside class="archive-filters">
  <fieldset>
    <legend>Sort & Filter</legend>

    <!-- Sort -->
    <label class="filter-group">
      <span class="filter-label">Sort By</span>
      <select bind:value={sort}>
        <option value="">Best Match</option>
        <option value="updated">Date Updated</option>
        <option value="words">Words</option>
        <option value="kudos">Kudos</option>
      </select>
    </label>

    <!-- Complete only -->
    <label class="filter-check">
      <input
        type="checkbox"
        checked={complete === true}
        onchange={(e) => {
          complete = e.currentTarget.checked ? true : null;
        }}
      />
      <span>Complete Only</span>
    </label>

    <!-- Word count range -->
    <div class="filter-range">
      <label class="filter-group half">
        <span class="filter-label">Min Words</span>
        <input type="number" bind:value={minWords} placeholder="0" min="0" />
      </label>
      <label class="filter-group half">
        <span class="filter-label">Max Words</span>
        <input type="number" bind:value={maxWords} placeholder="∞" min="0" />
      </label>
    </div>

    <!-- Kudos -->
    <label class="filter-group">
      <span class="filter-label">Min Kudos</span>
      <input type="number" bind:value={minKudos} placeholder="0" min="0" />
    </label>

    <!-- Rating checkboxes -->
    <div class="filter-group">
      <span class="filter-label">Rating</span>
      <div class="rating-options">
        {#each RATING_OPTIONS as rating}
          <label class="filter-check small">
            <input
              type="checkbox"
              checked={ratings.includes(rating)}
              onchange={(e) => {
                if (e.currentTarget.checked) {
                  ratings = [...ratings, rating];
                } else {
                  ratings = ratings.filter((r) => r !== rating);
                }
              }}
            />
            <span>{rating}</span>
          </label>
        {/each}
      </div>
    </div>

    <!-- Include tags -->
    <label class="filter-group">
      <span class="filter-label">Include Tags</span>
      <div class="tag-autocomplete" class:open={activeTagField === 'include_tags'}>
        <input
          type="text"
          placeholder="1:Fandom, 2:Character"
          bind:value={includeTags}
          oninput={() => onTagInput('include_tags')}
          onfocus={() => (activeTagField = 'include_tags')}
          onblur={() => setTimeout(() => (activeTagField = null), 150)}
        />
        {#if activeTagField === 'include_tags' && tagLoading}
          <span class="tag-ac-loading">⋯</span>
        {/if}
        {#if activeTagField === 'include_tags' && tagSuggestions.length > 0}
          <ul class="tag-ac-list" role="listbox">
            {#each tagSuggestions as hit (hit.id)}
              <li>
                <button type="button" role="option" aria-selected="false" onclick={() => applyTagSuggestion('include_tags', hit)}>
                  <span class="tag-ac-name">{hit.name}</span>
                  <span class="tag-ac-count">{hit.usage_count}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </label>

    <!-- Exclude tags -->
    <label class="filter-group">
      <span class="filter-label">Exclude Tags</span>
      <div class="tag-autocomplete" class:open={activeTagField === 'exclude_tags'}>
        <input
          type="text"
          placeholder="5:Graphic Depictions"
          bind:value={excludeTags}
          oninput={() => onTagInput('exclude_tags')}
          onfocus={() => (activeTagField = 'exclude_tags')}
          onblur={() => setTimeout(() => (activeTagField = null), 150)}
        />
        {#if activeTagField === 'exclude_tags' && tagLoading}
          <span class="tag-ac-loading">⋯</span>
        {/if}
        {#if activeTagField === 'exclude_tags' && tagSuggestions.length > 0}
          <ul class="tag-ac-list" role="listbox">
            {#each tagSuggestions as hit (hit.id)}
              <li>
                <button type="button" role="option" aria-selected="false" onclick={() => applyTagSuggestion('exclude_tags', hit)}>
                  <span class="tag-ac-name">{hit.name}</span>
                  <span class="tag-ac-count">{hit.usage_count}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </label>
  </fieldset>

  <!-- Actions -->
  <div class="filter-actions">
    <button class="archive-btn primary" onclick={handleApply} disabled={loading}>
      {loading ? 'Searching…' : 'Apply Filters'}
    </button>
    <button class="archive-btn clear" onclick={handleClear}>Clear All</button>
  </div>
</aside>

<style>
  .archive-filters {
    width: 240px;
    flex-shrink: 0;
    font-size: 0.9em;
  }

  fieldset {
    border: 1px solid var(--archive-border, #dddddd);
    padding: 0.75em 0.85em;
    margin: 0 0 0.75em;
    background: var(--archive-bg, #ffffff);
  }

  legend {
    font-weight: 700;
    font-size: 0.95em;
    color: var(--archive-text, #2a2a2a);
    padding: 0 0.3em;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.2em;
    margin-bottom: 0.75em;
  }

  .filter-label {
    font-size: 0.85em;
    font-weight: 700;
    color: var(--archive-text, #2a2a2a);
  }

  select,
  input[type='number'],
  input[type='text'] {
    width: 100%;
    padding: 0.3em 0.45em;
    font-family: inherit;
    font-size: 0.92em;
    color: var(--archive-text, #2a2a2a);
    background: var(--archive-bg, #ffffff);
    border: 1px solid var(--archive-border, #dddddd);
    border-radius: 0;
  }

  select:focus,
  input:focus {
    border-color: var(--archive-link, #990000);
    outline: none;
  }

  .filter-check {
    display: flex;
    align-items: center;
    gap: 0.4em;
    margin-bottom: 0.5em;
    font-size: 0.9em;
    cursor: pointer;
  }

  .filter-check.small {
    font-size: 0.85em;
    margin-bottom: 0.3em;
  }

  .filter-check input[type='checkbox'] {
    accent-color: var(--archive-link, #990000);
  }

  .rating-options {
    display: flex;
    flex-direction: column;
    gap: 0.15em;
  }

  .filter-range {
    display: flex;
    gap: 0.5em;
    margin-bottom: 0.75em;
  }

  .filter-range .half {
    flex: 1;
  }

  /* Tag autocomplete */
  .tag-autocomplete {
    position: relative;
  }

  .tag-ac-loading {
    position: absolute;
    right: 0.5em;
    top: 0.4em;
    color: var(--archive-muted, #666666);
  }

  .tag-ac-list {
    position: absolute;
    z-index: 20;
    top: 100%;
    left: 0;
    right: 0;
    list-style: none;
    margin: 0.15em 0 0;
    padding: 0.2em;
    background: var(--archive-bg, #ffffff);
    border: 1px solid var(--archive-border, #dddddd);
    max-height: 200px;
    overflow-y: auto;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .tag-ac-list li {
    margin: 0;
  }

  .tag-ac-list button {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5em;
    width: 100%;
    background: none;
    border: none;
    color: var(--archive-text, #2a2a2a);
    text-align: left;
    padding: 0.3em 0.4em;
    cursor: pointer;
    font-size: 0.88em;
    font-family: inherit;
  }

  .tag-ac-list button:hover {
    background: var(--archive-bg-raised, #f5f5f5);
    color: var(--archive-link, #990000);
  }

  .tag-ac-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tag-ac-count {
    font-size: 0.75em;
    color: var(--archive-muted, #666666);
    flex-shrink: 0;
  }

  /* Actions */
  .filter-actions {
    display: flex;
    flex-direction: column;
    gap: 0.4em;
  }

  .archive-btn {
    display: block;
    width: 100%;
    padding: 0.35em 0.7em;
    font-size: 0.9em;
    font-family: inherit;
    font-weight: 600;
    line-height: 1.6;
    color: #444;
    background: #eee;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #bbb 100%);
    border: 1px solid #bbb;
    border-bottom: 1px solid #aaa;
    border-radius: 0.25em;
    cursor: pointer;
    text-align: center;
    transition: color 0.15s, border-color 0.15s, box-shadow 0.15s;
  }

  .archive-btn:hover {
    color: #900;
    border-top-color: #999;
    border-left-color: #999;
    box-shadow: inset 2px 2px 2px #bbb;
  }

  .archive-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* AO3 has no red primary buttons — keep it in the .action family with a
     heavier border so it still reads as the main action. */
  .archive-btn.primary {
    color: #111;
    background: #e6e6e6;
    background-image: linear-gradient(#fff 2%, #ddd 95%, #ccc 100%);
    font-weight: 700;
    border-color: #999;
  }

  .archive-btn.primary:hover {
    color: #900;
    box-shadow: inset 2px 2px 2px #bbb;
  }

  .archive-btn.clear {
    background: none;
    border: none;
    color: var(--archive-muted, #666666);
    font-weight: 400;
    text-decoration: underline;
  }

  .archive-btn.clear:hover {
    color: #999;
  }
</style>
