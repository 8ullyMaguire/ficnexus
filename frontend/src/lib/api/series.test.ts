import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

async function importSeries() {
  return await import('./series');
}

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('getSeries', () => {
  it('fetches /api/series/{id} and returns the series response', async () => {
    const payload = {
      err: 0,
      series: { id: 7, name: 'Trilogy', description: 'd', created_at: 'c', updated_at: 'u', work_count: 3 },
      works: [
        {
          work_id: 1,
          url_id: 'a',
          canonical_title: 'A',
          canonical_author: 'Auth',
          title: 'A',
          author: 'Auth',
          words: 100,
          chapters: 2,
          status: 'complete',
          next_in_series: { work_id: 2, canonical_title: 'B', url_id: 'b' },
        },
      ],
    };
    mockFetch.mockResolvedValue(okJson(payload));

    const { getSeries } = await importSeries();
    const res = await getSeries(7);
    const called = mockFetch.mock.calls[0][0] as string;
    expect(called).toBe('/api/series/7');
    expect(res.series.name).toBe('Trilogy');
    expect(res.works[0].next_in_series?.work_id).toBe(2);
  });

  it('throws on non-OK responses', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 404, text: async () => '{"err":-5}' });
    const { getSeries } = await importSeries();
    await expect(getSeries(999)).rejects.toThrow('API error 404');
  });
});

describe('getAuthorByName', () => {
  it('URL-encodes the author name and returns the bibliography', async () => {
    const payload = {
      err: 0,
      author: { name: 'Test Author', work_count: 2, total_words: 3000, top_tags: [] },
      works: [{ work_id: 1, canonical_title: 'W', description: '', canonical_author: 'Test Author', url_id: 'u', words: 1000, chapters: 1, status: 'complete' }],
      orphans: [],
    };
    mockFetch.mockResolvedValue(okJson(payload));

    const { getAuthorByName } = await importSeries();
    const res = await getAuthorByName('Test Author');
    const called = mockFetch.mock.calls[0][0] as string;
    expect(called).toBe('/api/authors/by-name/Test%20Author');
    expect(res.author.work_count).toBe(2);
    expect(res.works[0].url_id).toBe('u');
  });
});
