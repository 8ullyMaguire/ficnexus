<!--
  Tier 2 (Guided): visual facet builder.

  Every control mutates `filters` (so the shared /api/search call + URL both
  stay in sync via the page's onFacetChange) and re-runs the search, which
  also refreshes the live match count (`total`). Facets that map to params the
  CURRENT backend honors work today (characters via include_tags, rating,
  status via complete, word count via min/max_words, updated-within via
  date_from). Attributes / relationships are written as v2 q= clauses so the
  URL stays shareable and they light up when the v2 backend lands.
-->
<script lang="ts">
  import { t } from '$lib/i18n/index.svelte';
  import type { SearchFilters, SearchFacets } from '$lib/api/search';

  let {
    filters = $bindable<SearchFilters>(),
    total,
    facets,
    onFacetChange,
  }: {
    filters: SearchFilters;
    total: number;
    facets: SearchFacets | null;
    onFacetChange: () => void;
  } = $props();

  const RATING_OPTIONS = [
    { value: 'General Audiences', key: 'search2.ratingGeneral' },
    { value: 'Teen And Up Audiences', key: 'search2.ratingTeen' },
    { value: 'Mature', key: 'search2.ratingMature' },
    { value: 'Explicit', key: 'search2.ratingExplicit' },
  ] as const;

  const STATUS_OPTIONS = [
    { value: 'complete', key: 'search2.statusComplete', complete: true },
    { value: 'ongoing', key: 'search2.statusOngoing', complete: false },
    { value: 'hiatus', key: 'search2.statusHiatus', complete: null },
    { value: 'cancelled', key: 'search2.statusCancelled', complete: null },
  ] as const;

  const UPDATED_OPTIONS = [
    { value: 'any', key: 'search2.anyTime', days: null },
    { value: 'day', key: 'search2.pastDay', days: 1 },
    { value: 'week', key: 'search2.pastWeek', days: 7 },
    { value: 'month', key: 'search2.pastMonth', days: 30 },
    { value: 'year', key: 'search2.pastYear', days: 365 },
  ] as const;

  let updatedWithin = $state('any');
  let guidedCharInput = $state('');
  let attr = $state('');
  let romship = $state('');
  let platship = $state('');

  // ── Characters (main_char → include_tags type 2 today) ───────────────

  function charSelected(name: string): boolean {
    return filters.include_tags
      .split(',')
      .map((s) => s.trim())
      .includes(`2:${name}`);
  }

  function characterOptions(): { name: string; checked: boolean }[] {
    const names = new Set<string>();
    for (const f of facets?.characters ?? []) names.add(f.name);
    for (const tok of filters.include_tags.split(',')) {
      const m = tok.trim().match(/^2:(.+)$/);
      if (m) names.add(m[1]);
    }
    return Array.from(names)
      .slice(0, 40)
      .map((name) => ({ name, checked: charSelected(name) }));
  }

  function toggleChar(name: string) {
    const entry = `2:${name}`;
    const list = filters.include_tags
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);
    const i = list.indexOf(entry);
    if (i >= 0) list.splice(i, 1);
    else list.push(entry);
    filters.include_tags = list.join(',');
    onFacetChange();
  }

  function addCharacter() {
    const name = guidedCharInput.trim();
    if (!name) return;
    toggleChar(name);
    guidedCharInput = '';
  }

  // ── Rating / Status (single-select chip groups) ──────────────────────

  function setRating(value: string) {
    filters.rating = filters.rating === value ? '' : value;
    onFacetChange();
  }

  function setStatus(opt: (typeof STATUS_OPTIONS)[number]) {
    if (filters.status === opt.value) {
      filters.status = '';
      filters.complete = null;
    } else {
      filters.status = opt.value;
      filters.complete = opt.complete;
    }
    onFacetChange();
  }

  // ── Attributes / Relationships → v2 q= clauses ───────────────────────

  function setClause(name: string, value: string) {
    const tokens = filters.q.split(/\s+/).filter(Boolean);
    const others = tokens.filter(
      (tok) => !tok.toLowerCase().startsWith(`${name.toLowerCase()}:`),
    );
    if (value) others.push(`${name}:"${value}"`);
    filters.q = others.join(' ');
    onFacetChange();
  }

  // ── Updated within ───────────────────────────────────────────────────

  function onUpdatedWithin(value: string) {
    updatedWithin = value;
    const opt = UPDATED_OPTIONS.find((o) => o.value === value);
    if (opt?.days) {
      filters.date_from = new Date(Date.now() - opt.days * 86_400_000).toISOString();
    } else {
      filters.date_from = '';
    }
    onFacetChange();
  }
</script>

<div class="guided-panel">
  <fieldset class="g-sec">
    <legend>{t('search2.characters')}</legend>
    {#if characterOptions().length > 0}
      <div class="char-grid">
        {#each characterOptions() as char (char.name)}
          <label class="g-check" class:checked={char.checked}>
            <input type="checkbox" checked={char.checked} onclick={() => toggleChar(char.name)} />
            <span class="g-check-name">{char.name}</span>
          </label>
        {/each}
      </div>
    {/if}
    <div class="g-add-row">
      <input
        class="g-add-input"
        type="text"
        aria-label={t('search2.addCharacter')}
        placeholder={t('search2.charPlaceholder')}
        bind:value={guidedCharInput}
        onkeydown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            addCharacter();
          }
        }}
      />
      <button class="chip" type="button" onclick={addCharacter}>{t('search2.addCharacter')}</button>
    </div>
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.attributes')}</legend>
    <input
      class="g-add-input"
      type="text"
      aria-label={t('search2.attributes')}
      placeholder={t('search2.attrPlaceholder')}
      value={attr}
      oninput={(e) => {
        attr = (e.currentTarget as HTMLInputElement).value;
        setClause('attr', attr);
      }}
    />
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.relationships')}</legend>
    <div class="g-col">
      <input
        class="g-add-input"
        type="text"
        aria-label={t('search2.romship')}
        placeholder={t('search2.romshipPlaceholder')}
        value={romship}
        oninput={(e) => {
          romship = (e.currentTarget as HTMLInputElement).value;
          setClause('romship', romship);
        }}
      />
      <input
        class="g-add-input"
        type="text"
        aria-label={t('search2.platship')}
        placeholder={t('search2.platshipPlaceholder')}
        value={platship}
        oninput={(e) => {
          platship = (e.currentTarget as HTMLInputElement).value;
          setClause('platship', platship);
        }}
      />
    </div>
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.rating')}</legend>
    <div class="chip-row">
      {#each RATING_OPTIONS as opt}
        <button
          class="chip"
          class:active={filters.rating === opt.value}
          type="button"
          aria-pressed={filters.rating === opt.value}
          onclick={() => setRating(opt.value)}
        >
          {t(opt.key)}
        </button>
      {/each}
    </div>
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.status')}</legend>
    <div class="chip-row">
      {#each STATUS_OPTIONS as opt}
        <button
          class="chip"
          class:active={filters.status === opt.value}
          type="button"
          aria-pressed={filters.status === opt.value}
          onclick={() => setStatus(opt)}
        >
          {t(opt.key)}
        </button>
      {/each}
    </div>
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.wordCount')}</legend>
    <div class="g-range">
      <label class="g-half">
        <span class="g-sub">{t('search2.minWords')}</span>
        <input type="number" min="0" bind:value={filters.min_words} oninput={onFacetChange} />
      </label>
      <label class="g-half">
        <span class="g-sub">{t('search2.maxWords')}</span>
        <input type="number" min="0" bind:value={filters.max_words} oninput={onFacetChange} />
      </label>
    </div>
  </fieldset>

  <fieldset class="g-sec">
    <legend>{t('search2.updatedWithin')}</legend>
    <select value={updatedWithin} onchange={(e) => onUpdatedWithin((e.currentTarget as HTMLSelectElement).value)}>
      {#each UPDATED_OPTIONS as opt}
        <option value={opt.value}>{t(opt.key)}</option>
      {/each}
    </select>
  </fieldset>

  <p class="g-count" aria-live="polite">{t('search2.matches', { count: total })}</p>
</div>

<style>
  .guided-panel {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 0.8rem;
    padding: 1rem;
    background: var(--color-surface-2);
    border: 1px solid var(--color-border);
    border-radius: var(--radius);
    margin-bottom: 1rem;
  }
  .g-sec {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: 0.6rem 0.7rem;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .g-sec legend {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--color-muted);
    padding: 0 0.3rem;
  }
  .g-check {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    cursor: pointer;
  }
  .g-check.checked .g-check-name {
    color: var(--color-primary);
  }
  .char-grid {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    max-height: 220px;
    overflow-y: auto;
  }
  .g-add-row {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }
  .g-add-input {
    flex: 1;
    min-width: 0;
  }
  .g-col {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }
  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }
  .g-range {
    display: flex;
    gap: 0.6rem;
  }
  .g-half {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    flex: 1;
  }
  .g-sub {
    font-size: 0.78rem;
    color: var(--color-muted);
  }
  .g-count {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-text);
    align-self: end;
    justify-self: end;
    margin: 0;
  }
</style>
