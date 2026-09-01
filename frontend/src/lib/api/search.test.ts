import { describe, it, expect, vi } from 'vitest';
import { buildSearchQuery, defaultFilters, fetchSearchSuggestions, SORT_OPTIONS, COMPLETE_OPTIONS, SOURCE_OPTIONS } from './search';

describe('defaultFilters', () => {
  it('returns empty/null defaults', () => {
    const f = defaultFilters();
    expect(f.q).toBe('');
    expect(f.include_tags).toBe('');
    expect(f.min_words).toBeNull();
    expect(f.complete).toBeNull();
    expect(f.page).toBe(1);
    expect(f.per_page).toBe(20);
  });
});

describe('buildSearchQuery', () => {
  it('omits empty params', () => {
    const qs = buildSearchQuery(defaultFilters());
    expect(qs).toBe('');
  });

  it('includes q when set', () => {
    const f = defaultFilters();
    f.q = 'Harry Potter';
    expect(buildSearchQuery(f)).toContain('q=Harry+Potter');
  });

  it('includes min_words when set', () => {
    const f = defaultFilters();
    f.min_words = 10000;
    expect(buildSearchQuery(f)).toContain('min_words=10000');
  });

  it('includes complete=true', () => {
    const f = defaultFilters();
    f.complete = true;
    expect(buildSearchQuery(f)).toContain('complete=true');
  });

  it('includes sort', () => {
    const f = defaultFilters();
    f.sort = 'updated';
    expect(buildSearchQuery(f)).toContain('sort=updated');
  });

  it('includes source', () => {
    const f = defaultFilters();
    f.source = 'archiveofourown.org';
    expect(buildSearchQuery(f)).toContain('source=archiveofourown.org');
  });

  it('includes date_from in ISO format', () => {
    const f = defaultFilters();
    f.date_from = '2024-01-01T00:00:00Z';
    expect(buildSearchQuery(f)).toContain('date_from=2024-01-01T00%3A00%3A00Z');
  });

  it('includes include_tags', () => {
    const f = defaultFilters();
    f.include_tags = '1:Harry Potter,4:Fluff';
    expect(buildSearchQuery(f)).toContain('include_tags=1%3AHarry+Potter%2C4%3AFluff');
  });

  it('includes primary_tag', () => {
    const f = defaultFilters();
    f.primary_tag = '1:Harry Potter';
    expect(buildSearchQuery(f)).toContain('primary_tag=1%3AHarry+Potter');
  });

  it('includes relationship_characters when set', () => {
    const f = defaultFilters();
    f.relationship_characters = 'Draco Malfoy, Hermione Granger';
    expect(buildSearchQuery(f)).toContain('relationship_characters=Draco+Malfoy%2C+Hermione+Granger');
  });

  it('omits relationship_characters when empty', () => {
    const f = defaultFilters();
    f.q = 'Harry Potter';
    f.relationship_characters = '';
    const qs = buildSearchQuery(f);
    expect(qs).not.toContain('relationship_characters');
    expect(qs).toContain('q=Harry+Potter');
  });

  it('omits exclude_tag_types and strict_gen by default', () => {
    const qs = buildSearchQuery(defaultFilters());
    expect(qs).not.toContain('exclude_tag_types');
    expect(qs).not.toContain('strict_gen');
  });

  it('includes exclude_tag_types when set', () => {
    const f = defaultFilters();
    f.exclude_tag_types = '3,6';
    const qs = buildSearchQuery(f);
    expect(qs).toContain('exclude_tag_types=3%2C6');
  });

  it('includes strict_gen=true when enabled', () => {
    const f = defaultFilters();
    f.strict_gen = true;
    const qs = buildSearchQuery(f);
    expect(qs).toContain('strict_gen=true');
  });

  it('does not emit strict_gen=false', () => {
    const f = defaultFilters();
    f.strict_gen = false;
    expect(buildSearchQuery(f)).not.toContain('strict_gen');
  });

  it('omits page when 1', () => {
    const f = defaultFilters();
    f.page = 1;
    expect(buildSearchQuery(f)).not.toContain('page=');
  });

  it('includes page when > 1', () => {
    const f = defaultFilters();
    f.page = 3;
    expect(buildSearchQuery(f)).toContain('page=3');
  });
});

describe('constants', () => {
  it('SORT_OPTIONS has entries', () => {
    expect(SORT_OPTIONS.length).toBeGreaterThan(0);
  });
  it('COMPLETE_OPTIONS has 3 entries', () => {
    expect(COMPLETE_OPTIONS).toHaveLength(3);
  });
  it('SOURCE_OPTIONS has entries', () => {
    expect(SOURCE_OPTIONS.length).toBeGreaterThan(0);
  });
});

describe('fetchSearchSuggestions', () => {
  it('requests the popular endpoint without personal param', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, suggestions: [] }),
    });
    vi.stubGlobal('fetch', fetchMock);

    await fetchSearchSuggestions(false);
    expect(fetchMock).toHaveBeenCalledWith('/api/search/suggest', { credentials: 'include' });
    vi.unstubAllGlobals();
  });

  it('requests personal=1 when personal is true', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, suggestions: [] }),
    });
    vi.stubGlobal('fetch', fetchMock);

    await fetchSearchSuggestions(true);
    expect(fetchMock).toHaveBeenCalledWith('/api/search/suggest?personal=1', { credentials: 'include' });
    vi.unstubAllGlobals();
  });

  it('throws when err !== 0 (caller hides the row)', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ err: -1, msg: 'boom' }),
    }));

    await expect(fetchSearchSuggestions(false)).rejects.toThrow();
    vi.unstubAllGlobals();
  });

  it('parses the suggestion list', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        suggestions: [{ id: 1, name: 'Adventure', tag_type_id: 4, usage_count: 5, reason: 'popular' }],
      }),
    }));

    const s = await fetchSearchSuggestions(false);
    expect(s).toHaveLength(1);
    expect(s[0].name).toBe('Adventure');
    expect(s[0].reason).toBe('popular');
    vi.unstubAllGlobals();
  });
});
