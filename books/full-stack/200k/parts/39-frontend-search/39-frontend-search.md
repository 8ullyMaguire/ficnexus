# Part 39 — Frontend Search UI

> In this chapter you will learn how FicHub's three-tier search interface works — SimpleSearch for quick queries, GuidedSearch for facet-based filtering, and PowerSearch for raw query syntax. We'll build the search page, the tag autocomplete with debounce, the URL sync system, saved searches, and the Vitest test suite.

---

## Overview

FicHub's search page (`src/routes/search/+page.svelte`, 1839 lines) implements a **three-tier search UI**:

1. **Simple** — a single search box with quick-filter chips (No Warnings, Complete Only, 100k+ Words, etc.).
2. **Guided** — facet sections (fandoms, characters, relationships, warnings, time range, sort, source) with checkboxes.
3. **Power** — a raw query syntax textarea with live interpretation into editable chips.

All three tiers share the same `SearchFilters` state, serialized to the URL via `goto` so any search is shareable/bookmarkable. The URL uses `replaceState` to keep the back button clean.

---

## Chapter 39.1 — The Search API Client

### Goal

Understand the `SearchFilters` type and how filters serialize to query parameters.

### Actions

```typescript
// frontend/src/lib/api/search.ts (lines 1–225)

/// Tag types matching the database schema.
export const TAG_TYPES = { 1: 'Fandom', 2: 'Character', 3: 'Relationship', 4: 'Freeform' } as const;

/// A single search result item.
export interface SearchResult {
  url_id: string;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  updated: string | null;
  rank: number | null;
  snippet: string | null;
  tags: SearchTag[];
  total_freeform: number;
  comment_count: number;
  kudos_count: number;
}

/// All filter parameters for the search API.
export interface SearchFilters {
  q: string;               // free text query
  include_tags: string;     // comma-separated "type_id:name" (AND)
  exclude_tags: string;     // comma-separated (NOT)
  include_any_tags: string; // comma-separated (OR)
  exclude_tag_types: string; // "3" = no ships
  strict_gen: boolean;       // hide relationships + categories
  min_words: number | null;
  max_words: number | null;
  min_chapters: number | null;
  max_chapters: number | null;
  complete: boolean | null;
  source: string;
  date_from: string;
  date_to: string;
  sort: string;
  page: number;
  per_page: number;
  primary_tag: string;      // first tag to pin (AO3-style)
  relationship_characters: string;
  min_comments: number | null;
  min_kudos: number | null;
  no_warnings: boolean | null;
  hide_read: boolean;
  hide_bookmarked: boolean;
  library_only: boolean;
}

/// Default search filters — page 1, 20 per page, no query.
export function defaultFilters(): SearchFilters {
  return {
    q: '', include_tags: '', exclude_tags: '', include_any_tags: '',
    exclude_tag_types: '', strict_gen: false,
    min_words: null, max_words: null, min_chapters: null, max_chapters: null,
    complete: null, source: '', date_from: '', date_to: '',
    sort: '', page: 1, per_page: 20,
    primary_tag: '', relationship_characters: '',
    min_comments: null, min_kudos: null, no_warnings: null,
    hide_read: false, hide_bookmarked: false, library_only: false,
  };
}

/// Build query string from filters, omitting empty/null params.
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();
  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  // ... (same pattern for every field)
  return params.toString();
}

/// Execute a search request against GET /api/search.
export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const qs = buildSearchQuery(filters);
  const res = await fetch(`/api/search${qs ? '?' + qs : ''}`);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Search failed (${res.status}): ${text}`);
  }
  return (await res.json()) as SearchResponse;
}
```

### Try It Yourself

```typescript
// Add a new filter: max_words
// 1. Add to SearchFilters interface
// 2. Add to defaultFilters() (return null)
// 3. Add to buildSearchQuery() (if not null, params.set)
// 4. Add to activeChips() and isQuickActive() in +page.svelte
```

### Check

- ✅ `SearchFilters` has 23 fields — query, tags, words, chapters, complete, source, dates, sort, pagination, and personal toggles.
- ✅ `defaultFilters()` returns `per_page: 20`, `page: 1`, empty query.
- ✅ `buildSearchQuery` omits null/empty params — clean URLs.
- ✅ `search()` throws on non-OK response with status + body.

### What you built

The search API client — a typed `SearchFilters` interface with 23 fields, a `defaultFilters()` factory, a `buildSearchQuery()` serializer that omits empty params, and a `search()` function that throws on HTTP errors.

---

## Chapter 39.2 — SimpleSearch with Quick Filters

### Goal

Build the SimpleSearch tier: a search box with quick-filter chips that toggle common filters.

### Actions

#### 1. Quick filters

```typescript
// src/routes/search/+page.svelte (lines 317–371)
/// Preset quick filters (better than AO3) — labels resolved through i18n.
const QUICK_FILTERS = [
  { label: 'No Warnings', key: 'no_warnings' as const, value: true },
  { label: 'Complete Only', key: 'complete' as const, value: true },
  { label: '100k+ Words', key: 'min_words' as const, value: 100000 },
  { label: 'Short (<5k)', key: 'max_words' as const, value: 5000 },
  { label: 'Multi-chapter (20+)', key: 'min_chapters' as const, value: 20 },
  { label: 'Has Comments (10+)', key: 'min_comments' as const, value: 10 },
  { label: 'Popular (50+ kudos)', key: 'min_kudos' as const, value: 50 },
  { label: 'Strict Gen', key: 'strict_gen' as const, value: true },
  { label: 'No Ships', key: 'exclude_tag_types' as const, value: '3' },
];

function applyQuickFilter(label: string) {
  const f = QUICK_FILTERS.find(q => q.label === label);
  if (!f) return;
  // Toggle semantics: clicking an active chip clears the filter
  if (f.key === 'no_warnings') filters.no_warnings = filters.no_warnings === true ? null : true;
  else if (f.key === 'complete') filters.complete = filters.complete === true ? null : true;
  else if (f.key === 'strict_gen') filters.strict_gen = !filters.strict_gen;
  else if (f.key === 'exclude_tag_types')
    filters.exclude_tag_types = filters.exclude_tag_types.includes('3') ? '' : '3';
  else if (f.key === 'min_words') filters.min_words = filters.min_words === f.value ? null : f.value;
  // ... (same toggle pattern for all numeric filters)
  doSearch();  // debounced via queueSearch for numeric, immediate for boolean
}
```

#### 2. Active filter chips

```typescript
/// One removable chip per non-default filter.
function activeChips() {
  const chips: { label: string; value: string; clear: () => void }[] = [];
  const add = (label, value, clear) => chips.push({ label, value, clear });

  // Query chip
  if (filters.q) add('q', filters.q, () => (filters.q = ''));
  // Tag chips
  if (filters.include_tags) add('include_tags', filters.include_tags, () => (filters.include_tags = ''));
  // Boolean chips with localized labels
  if (filters.strict_gen) add('strict_gen', t('search.hideShips'), () => (filters.strict_gen = false));
  // Numeric chips
  if (filters.min_words !== null) add('min_words', String(filters.min_words), () => (filters.min_words = null));
  // ... (same for every field)
  return chips;
}
```

#### 3. The search box

```svelte
<!-- SimpleSearch renders the search input + applies Simple tier -->
<div class="simple-search">
  <input
    type="text"
    placeholder={t('search.placeholder')}
    bind:value={filters.q}
    on:input={() => handleSimpleChange()}  // syncUrl() + doSearch()
  />
  <button onclick={() => doSearch()}>{t('search.button')}</button>
</div>

<!-- Quick filter chips -->
<div class="quick-filters">
  {#each QUICK_FILTERS as { label, key }}
    <button
      class:active={isQuickActive(label)}
      onclick={() => applyQuickFilter(label)}
    >{label}</button>
  {/each}
</div>

<!-- Active filter chips: one removable chip per active filter -->
{#if activeChips().length > 0}
  <div class="active-chips">
    {#each activeChips() as chip}
      <span class="chip">{chip.label}: {chip.value}
        <button onclick={() => { clearChip(chip); queueSearch(); }}>✕</button>
      </span>
    {/each}
  </div>
{/if}
```

### Try It Yourself

```typescript
// Add a new quick filter: "Large (200k+ words)"
{ label: '200k+ Words', key: 'min_words' as const, value: 200000 },
```

### Check

- ✅ Quick filters toggle: clicking an active chip clears the filter.
- ✅ `"No Ships"` sets `exclude_tag_types` to `"3"` (relationship tags).
- ✅ `"Strict Gen"` hides both relationships and categories.
- ✅ Active chips render one per non-default filter, with a remove (✕) button.
- ✅ Labels are locale-resolved via `t()`.

### What you built

The SimpleSearch tier — a search box, quick-filter toggling chips with localized labels, and active filter chips with remove buttons. Each toggle immediately serializes to the URL for shareability.

---

## Chapter 39.3 — Tag Autocomplete with Debounce

### Goal

Build the tag autocomplete that fires only after 250ms of user inactivity, with type-prefixed tag IDs (e.g. `1:` for fandoms).

### Actions

```typescript
// src/routes/search/+page.svelte (lines 198–262)
let tagSearchSeq = 0;  // prevents stale responses

/// Debounced autocomplete: fire only after 250ms of inactivity.
function onTagInput(field: string) {
  const raw = filters[field] as string;
  const lastPart = raw.split(',').pop()?.trim() ?? '';
  const seq = ++tagSearchSeq;

  if (lastPart.length < 2) {
    tagSuggestions = []; tagLoading = false; return;
  }
  tagLoading = true;

  // Type prefix: "1:Harry" → tagType = 1 (Fandom)
  let tagType: number | undefined;
  const colon = lastPart.indexOf(':');
  if (colon > 0) {
    const t = Number(lastPart.slice(0, colon));
    if (Number.isInteger(t) && t >= 1 && t <= 7) tagType = t;
  }

  setTimeout(async () => {
    if (seq !== tagSearchSeq) return;  // superseded by newer keystrokes
    try {
      const hits = await fetchTagAutocomplete(lastPart.replace(/^\d+:/, ''), tagType);
      if (seq !== tagSearchSeq) return;
      tagSuggestions = hits;
    } catch {
      tagSuggestions = [];
    } finally {
      if (seq === tagSearchSeq) tagLoading = false;
    }
  }, 250);  // 250ms debounce
}

function applyTagSuggestion(field: string, hit: TagAutocompleteHit) {
  const raw = (filters[field] as string) || '';
  const parts = raw.split(',').map(p => p.trim()).filter(Boolean);
  parts[parts.length - 1] = `${hit.type_id}:${hit.name}`;  // "2:Hermione"
  filters[field] = parts.join(', ');
  tagSuggestions = []; activeTagField = null;
}
```

```typescript
// frontend/src/lib/api/search.ts (lines 256–281)
/// GET /api/tags/autocomplete?q=...&tag_type=2 — tag suggestions with usage counts.
export async function fetchTagAutocomplete(q: string, tagType?: number): Promise<TagAutocompleteHit[]> {
  if (q.trim().length < 2) return [];  // no single-char queries
  const params = new URLSearchParams({ q: q.trim() });
  if (tagType !== undefined) params.set('tag_type', String(tagType));
  const res = await fetch(`/api/tags/autocomplete?${params.toString()}`);
  if (!res.ok) throw new Error(`Tag autocomplete failed (${res.status})`);
  const data = (await res.json()) as { err: number; results?: TagAutocompleteHit[] };
  if (data.err !== 0) throw new Error(`Tag autocomplete err:${data.err}`);
  return data.results ?? [];
}
```

### Try It Yourself

```typescript
// Increase the debounce to 500ms for slower typists
setTimeout(async () => { /* ... */ }, 500);
```

### Check

- ✅ Autocomplete fires only after 250ms of inactivity (prevents API burst per keystroke).
- ✅ `tagSearchSeq` prevents stale responses from overwriting newer ones.
- ✅ Type prefix `"1:Harry"` is parsed to `tagType = 1` (Fandom).
- ✅ Minimum query length is 2 characters (no single-char API calls).
- ✅ Applied suggestions use the format `"type_id:name"` (e.g. `"2:Hermione"`).

### What you built

The tag autocomplete — a 250ms debounced lookup against `/api/tags/autocomplete`, with sequence-number anti-staleness (old responses discarded), type-prefixed tag IDs, and a 2-character minimum query length.

---

## Chapter 39.4 — PowerSearch with Chip Interpretation

### Goal

Build the PowerSearch tier: a raw query syntax textarea that interprets whitespace-separated tokens into editable, removable chips.

### Actions

#### 1. PowerSearch component

```svelte
<!-- SimpleSearch/GuidedSearch/PowerSearch components are imported at top of +page.svelte -->
<script lang="ts">
    // The Power tier shows a textarea with raw query syntax:
    // "Harry @char:Harry -romship:*Malfoy" → interpreted as 3 chips
    import PowerSearch from '$lib/components/search/PowerSearch.svelte';
    import GuidedSearch from '$lib/components/search/GuidedSearch.svelte';
    import SimpleSearch from '$lib/components/search/SimpleSearch.svelte';

    // Tier switcher — tab-style UI
    type SearchTier = 'simple' | 'guided' | 'power';
    let tier = $state<SearchTier>('simple');
</script>
```

#### 2. Chip interpretation

```typescript
// The PowerSearch textarea splits input on whitespace into tokens:
const tokens = filters.q.split(/\s+/).filter(Boolean);
// Each token becomes an editable chip:
// "Harry" → { text: "Harry", type: "term" }
// "@char:Harry" → { text: "@char:Harry", type: "tag" }
// "-romship:*Malfoy" → { text: "-romship:*Malfoy", type: "exclude" }

// Removing a chip updates filters.q by rejoining the remaining tokens
function removeChip(index: number) {
    const tokens = filters.q.split(/\s+/).filter(Boolean);
    tokens.splice(index, 1);
    filters.q = tokens.join(' ');
    doSearch();
}

// Editing a chip opens an inline input that updates the token in-place
```

#### 3. URL synchronization

```typescript
// src/routes/search/+page.svelte (lines 282–295)
/// Persist the current query + tier to the URL so any tier's state is shareable.
function syncUrl() {
    const params = new URLSearchParams(buildSearchQuery(filters));
    params.set('tier', tier);
    if (showAdvanced) params.set('advanced', '1');
    goto('/search?' + params.toString(), { replaceState: true, noScroll: true });
}

/// Switch tiers — the URL keeps the current q= (and structured params) so
/// state carries over; "Upgrade to Power" lands on the Power textarea seeded from q.
function setTier(next: SearchTier) {
    tier = next;
    syncUrl();
}
```

### Try It Yourself

```svelte
<!-- Add a "copy as URL" button that copies the current search URL to clipboard -->
<button onclick={() => {
    const url = window.location.href;
    navigator.clipboard.writeText(url);
}}>
    Copy URL
</button>
```

### Check

- ✅ Three tiers are tab-selectable: Simple (default), Guided, Power.
- ✅ `syncUrl()` uses `replaceState: true` (back button stays clean).
- ✅ URL includes `tier=guided|power|simple` so the selected tier is shareable.
- ✅ `setTier()` preserves `filters.q` — switching tiers doesn't lose the query.
- ✅ PowerSearch interprets `"@char:Harry"` as a typed tag token.
- ✅ Removing a chip rejoins tokens and re-searches.

### What you built

The PowerSearch tier — a raw query syntax textarea that interprets whitespace-separated tokens into editable/removable chips, with URL sync that preserves tier selection and search state across navigation.

---

## Chapter 39.5 — Search Page Tests

### Goal

Understand the Vitest test suite for the search page — 15 tests covering rendering, filtering, tier switching, chip management, and URL serialization.

### Actions

```typescript
// src/routes/search/page.test.ts (lines 1–313)
// Note: file is named page.test.ts (not +page.test.ts) — SvelteKit loads either,
// but the convention is +page.test.ts for newer projects. FicHub uses page.test.ts.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

beforeEach(() => {
    mockFetch.mockReset();
    localStorage.clear();
    // Default to modern UI in tests (archive is the app default)
    localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'modern' }));
});

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$app/stores', () => ({
    page: { subscribe: (fn) => { fn({ url: new URL('http://localhost/search') }); return () => {}; } },
}));
```

#### Test 1: Renders search bar and quick filters

```typescript
it('renders search bar and quick filters', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    expect(screen.getByPlaceholderText(/search titles/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /search/i })).toBeTruthy();
    expect(screen.getByText('No Warnings')).toBeTruthy();
    expect(screen.getByText('Complete Only')).toBeTruthy();
    expect(screen.getByText('100k+ Words')).toBeTruthy();
});
```

#### Test 2: Advanced filters toggle

```typescript
it('shows advanced filters when toggled', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const toggle = screen.getByText(/Show Advanced/i);
    await fireEvent.click(toggle);
    expect(screen.getByText('Must Have Tags (AND)')).toBeTruthy();
    expect(screen.getByText('Any Of These Tags (OR)')).toBeTruthy();
    expect(screen.getByText('Exclude Tags')).toBeTruthy();
});
```

#### Test 3: Strict Gen toggle

```typescript
it('Strict Gen quick filter toggles and persists into the search query', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ total: 0, results: [] }) });
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const strictGenBtn = screen.getByRole('button', { name: 'Strict Gen' });
    await fireEvent.click(strictGenBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(1); });
    expect(searchUrls()[0]).toContain('strict_gen=true');
    // Second click clears it
    await fireEvent.click(strictGenBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(2); });
    expect(searchUrls()[1]).not.toContain('strict_gen');
});
```

#### Test 4: No Ships toggle

```typescript
it('No Ships quick filter toggles exclude_tag_types', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ total: 0, results: [] }) });
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const noShipsBtn = screen.getByRole('button', { name: 'No Ships' });
    await fireEvent.click(noShipsBtn);
    await waitFor(() => { expect(searchUrls().length).toBe(1); });
    expect(searchUrls()[0]).toContain('exclude_tag_types=3');
});
```

#### Test 5: Three-tier rendering

```typescript
it('defaults to the Simple tier with a tier switcher', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const simpleTab = screen.getByRole('tab', { name: 'Simple' });
    expect(simpleTab.getAttribute('aria-selected')).toBe('true');
    expect(screen.getByRole('tab', { name: 'Guided' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Power' })).toBeTruthy();
});

it('switches to Guided and back to Simple', async () => {
    // Click Guided → facet sections appear, Simple input disappears
    // Click Simple → back to search box
});

it('switches to Power and shows the raw syntax textarea', async () => {
    await fireEvent.click(screen.getByRole('tab', { name: 'Power' }));
    expect(screen.getByPlaceholderText(/@char:"Harry"/)).toBeTruthy();
});
```

#### Test 6: Chip interpretation

```typescript
it('renders an Interpreted-as chip row from a typed q value', async () => {
    mockFetch.mockResolvedValue(okResponse());
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const input = screen.getByPlaceholderText(/search titles/i);
    await fireEvent.input(input, { target: { value: 'Harry @char:Harry -romship:*Malfoy' } });
    expect(screen.getByText('Interpreted as:')).toBeTruthy();
    expect(screen.getByLabelText('Edit Harry')).toBeTruthy();
    expect(screen.getByLabelText('Edit @char:Harry')).toBeTruthy();
    expect(screen.getByLabelText('Edit -romship:*Malfoy')).toBeTruthy();
    expect(screen.getByLabelText('Remove @char:Harry')).toBeTruthy();
});
```

#### Test 7: "Surprise me" link

```typescript
it('Surprise me links to the blind-date page', async () => {
    const { default: SearchPage } = await loadSearchPage();
    render(SearchPage);
    const link = screen.getByRole('link', { name: /surprise me/i });
    expect(link.getAttribute('href')).toBe('/blind-date');
});
```

### Try It Yourself

```bash
# Run the search page tests
cd frontend
npx vitest run src/routes/search/page.test.ts

# Run all frontend tests
npm test -- run
```

### Check

- ✅ Tests mock `$app/navigation` (goto) and `$app/stores` (page) for isolation.
- ✅ Default UI mode is 'modern' in tests (app default is 'archive').
- ✅ `searchUrls()` filters fetch calls to only `/api/search` requests.
- ✅ Tests verify chip toggle persistence (click active chip → clears filter).
- ✅ Tier switching preserves filters across tabs.
- ✅ Chip interpretation splits on whitespace, creates editable/removable chips.
- ✅ "Surprise me" links to `/blind-date`.

### What you built

The complete test suite for the search page — 15 Vitest cases covering rendering (all three tiers), quick filter toggling, strict_gen/No Ships toggle persistence, advanced filter visibility, search execution with mock API responses, error states, chip interpretation from typed queries, tier switching with URL sync, and the "Surprise me" link.

---

## Conclusion

You now understand FicHub's search UI frontend:

1. **Three-tier search** — SimpleSearch (box + quick chips), GuidedSearch (faceted filters), PowerSearch (raw syntax → interpreted chips), all sharing `SearchFilters` state.
2. **Tag autocomplete** — 250ms debounced, anti-stale via sequence numbers, type-prefixed IDs.
3. **URL sync** — `goto` with `replaceState`, `tier` parameter, shareable state across tier switches.
4. **Quick filters** — toggle semantics (click active → clear), localized labels, serialized to query params.
5. **Chip system** — editable + removable, whitespace-split interpretation, local-storage-persisted prefs.
6. **Testing** — 15 Vitest cases with mocked SvelteKit modules, fetch mocking, and `waitFor` assertions.

In Part 19 we'll build the work and fic page components — the `[workId]` and `[urlId]` routes, the `WorkBlurb` reusable component, and the bookmark/kudos/read-status widgets.
