import { describe, it, expect, vi, beforeEach } from 'vitest';
import { buildSearchQuery, defaultFilters, type SearchFilters } from './search';

// Test search API client
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

describe('search API', () => {
  it('buildSearchQuery includes q parameter', () => {
    const filters = defaultFilters();
    filters.q = 'greed';
    const qs = buildSearchQuery(filters);
    expect(qs).toContain('q=greed');
  });

  it('buildSearchQuery includes all new parameters', () => {
    const filters = defaultFilters();
    filters.primary_tag = '1:Harry Potter';
    filters.min_comments = 10;
    filters.min_kudos = 50;
    filters.no_warnings = true;
    filters.tag_ids = '1,2,3';
    const qs = buildSearchQuery(filters);
    expect(qs).toContain('primary_tag=1%3AHarry+Potter');
    expect(qs).toContain('min_comments=10');
    expect(qs).toContain('min_kudos=50');
    expect(qs).toContain('no_warnings=true');
    expect(qs).toContain('tag_ids=1%2C2%2C3');
  });

  it('buildSearchQuery omits empty parameters', () => {
    const filters = defaultFilters();
    const qs = buildSearchQuery(filters);
    expect(qs).not.toContain('primary_tag');
    expect(qs).not.toContain('min_comments');
    expect(qs).not.toContain('min_kudos');
    expect(qs).not.toContain('no_warnings');
    expect(qs).not.toContain('tag_ids');
  });

  it('search function calls correct endpoint', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ total: 0, page: 1, per_page: 20, results: [] }),
    });
    const { search } = await import('./search');
    const filters = defaultFilters();
    filters.q = 'test';
    await search(filters);
    const url = mockFetch.mock.calls[0][0];
    expect(url).toContain('/api/search');
    expect(url).toContain('q=test');
  });

  it('search function handles errors', async () => {
    mockFetch.mockResolvedValue({
      ok: false,
      text: async () => 'database error',
    });
    const { search } = await import('./search');
    const filters = defaultFilters();
    filters.q = 'greed';
    await expect(search(filters)).rejects.toThrow('Search failed');
  });

  it('defaultFilters has all new fields', () => {
    const filters = defaultFilters();
    expect(filters.primary_tag).toBe('');
    expect(filters.min_comments).toBeNull();
    expect(filters.min_kudos).toBeNull();
    expect(filters.no_warnings).toBeNull();
    expect(filters.tag_ids).toBe('');
  });
});
