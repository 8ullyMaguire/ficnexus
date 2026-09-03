import { describe, it, expect, vi, beforeEach } from 'vitest';

// Extend the search API client tests: the `search` fetch path, autocomplete,
// and the personal-filter query params (hide_read / hide_bookmarked /
// library_only) that the existing suites don't cover.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function importSearch() {
  return await import('./search');
}

describe('search() fetch path', () => {
  it('fetches /api/search with the built query string', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ total: 0, page: 1, per_page: 20, results: [], facets: {} }),
    });
    const { search, defaultFilters } = await importSearch();
    const f = defaultFilters();
    f.q = 'Harry';
    f.hide_read = true;
    f.hide_bookmarked = true;
    f.library_only = true;
    await search(f);
    const url = String(mockFetch.mock.calls[0][0]);
    expect(url).toContain('/api/search?');
    expect(url).toContain('q=Harry');
    expect(url).toContain('hide_read=true');
    expect(url).toContain('hide_bookmarked=true');
    expect(url).toContain('library_only=true');
  });

  it('fetches /api/search with no query string for default filters', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ total: 0, page: 1, per_page: 20, results: [], facets: {} }),
    });
    const { search, defaultFilters } = await importSearch();
    await search(defaultFilters());
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/search');
  });
});

describe('fetchTagAutocomplete', () => {
  it('returns [] without fetching for short queries', async () => {
    const { fetchTagAutocomplete } = await importSearch();
    const res = await fetchTagAutocomplete('a');
    expect(res).toEqual([]);
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('GETs /api/tags/autocomplete?q= and parses results', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, results: [{ id: 1, name: 'Fluff', type: 'freeform', type_id: 4, usage_count: 5 }] }),
    });
    const { fetchTagAutocomplete } = await importSearch();
    const res = await fetchTagAutocomplete('flu');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/tags/autocomplete?q=flu');
    expect(res).toHaveLength(1);
    expect(res[0].name).toBe('Fluff');
  });

  it('adds tag_type when provided', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: 0, results: [] }) });
    const { fetchTagAutocomplete } = await importSearch();
    await fetchTagAutocomplete('flu', 4);
    expect(String(mockFetch.mock.calls[0][0])).toContain('tag_type=4');
  });

  it('throws on non-OK or err != 0', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 500, json: async () => ({}) });
    const { fetchTagAutocomplete } = await importSearch();
    await expect(fetchTagAutocomplete('flu')).rejects.toThrow('Tag autocomplete failed');
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ err: -1 }) });
    await expect(fetchTagAutocomplete('flu')).rejects.toThrow('Tag autocomplete err');
  });
});
