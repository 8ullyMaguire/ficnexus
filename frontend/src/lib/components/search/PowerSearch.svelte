<!--
  Tier 3 (Power): raw v2 syntax textarea fed straight to q=.

  Includes a field-name datalist for light autocomplete, a bracket/quote
  balance hint, an inline generic parse-error line, and a "Save this query"
  control that reuses the existing saved-views endpoint (there is no dedicated
  /api/search/saved endpoint, so the cross-link is omitted and this inline
  save is shown instead).
-->
<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import { onMount } from 'svelte';
  import type { SearchFilters } from '$lib/api/search';

  let {
    filters = $bindable<SearchFilters>(),
    onSearch,
    onFacetChange,
    error = '',
    canSave = false,
    savingView = false,
    onSave,
  }: {
    filters: SearchFilters;
    onSearch: () => void;
    onFacetChange: () => void;
    error?: string;
    canSave?: boolean;
    savingView?: boolean;
    onSave?: (name: string) => Promise<boolean>;
  } = $props();

  const POWER_FIELDS = [
    'title',
    'author',
    'fandom',
    'primary_fandom',
    'crossover',
    'char',
    'main_char',
    'ship',
    'romship',
    'platship',
    'attr',
    'rating',
    'status',
    'words',
    'chapters',
    'kudos',
    'comments',
    'bookmarks',
    'hits',
    'language',
    'sort',
  ];

  const DEBOUNCE_MS = 700;
  let debounce: ReturnType<typeof setTimeout> | null = null;
  let saveName = $state('');

  onMount(() => {
    return () => {
      if (debounce) clearTimeout(debounce);
    };
  });

  function onInput() {
    if (debounce) clearTimeout(debounce);
    debounce = setTimeout(() => onFacetChange(), DEBOUNCE_MS);
  }

  /** Best-effort balance check for quotes and [ ] / ( ) pairs. */
  function unbalanced(q: string): boolean {
    let depth = 0;
    let quote = false;
    let esc = false;
    for (let i = 0; i < q.length; i++) {
      const c = q[i];
      if (esc) {
        esc = false;
        continue;
      }
      if (c === '\\') {
        esc = true;
        continue;
      }
      if (quote) {
        if (c === '"') quote = false;
        continue;
      }
      if (c === '"') {
        quote = true;
        continue;
      }
      if (c === '[' || c === '(') depth++;
      if (c === ']' || c === ')') depth--;
    }
    return quote || depth !== 0;
  }

  const powerHasError = $derived(unbalanced(filters.q) || Boolean(error));

  async function handleSave() {
    const name = saveName.trim();
    if (!name || !onSave || savingView) return;
    const ok = await onSave(name);
    if (ok) saveName = '';
  }
</script>

<div class="power-panel">
  <label class="power-field">
    <span class="power-label">{t('search2.powerLabel')}</span>
    <textarea
      class="power-textarea"
      rows="4"
      spellcheck="false"
      placeholder={t('search2.powerPlaceholder')}
      bind:value={filters.q}
      oninput={onInput}
      onkeydown={(e) => {
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
          e.preventDefault();
          onSearch();
        }
      }}
    ></textarea>
  </label>

  <datalist id="search-field-names">
    {#each POWER_FIELDS as field}
      <option value="{field}:"></option>
    {/each}
  </datalist>

  <label class="power-field">
    <span class="power-label">{t('search2.fieldAutocomplete')}</span>
    <input class="g-add-input" list="search-field-names" type="text" placeholder={t('search2.powerFieldPlaceholder')} />
  </label>

  <p class="power-hint">{t('search2.powerHint')}</p>

  {#if powerHasError}
    <p class="power-error-line" role="alert">
      {unbalanced(filters.q) ? t('search2.bracketHint') : error}
    </p>
    {#if unbalanced(filters.q)}
      <p class="power-error-line" role="alert">{t('search2.parseError')}</p>
    {/if}
  {/if}

  {#if canSave}
    <form class="power-save" onsubmit={(e) => { e.preventDefault(); void handleSave(); }}>
      <input
        type="text"
        aria-label={t('search2.saveThisSearch')}
        placeholder={t('search2.savedNamePlaceholder')}
        bind:value={saveName}
      />
      <button class="chip" type="submit" disabled={!saveName.trim() || savingView}>
        {t('search2.saveThisSearch')}
      </button>
    </form>
  {/if}
</div>

<style>
  .power-panel {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 1rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    margin-bottom: 1rem;
  }
  .power-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
  }
  .power-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--color-text);
  }
  .power-textarea {
    width: 100%;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.88rem;
    resize: vertical;
  }
  .power-hint {
    font-size: 0.78rem;
    color: var(--color-muted);
    margin: 0;
  }
  .power-error-line {
    font-size: 0.85rem;
    color: var(--color-error);
    margin: 0;
  }
  .power-save {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .power-save input {
    min-height: 2rem;
    padding: 0.3rem 0.5rem;
  }
</style>
