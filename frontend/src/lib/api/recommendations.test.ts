import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchAlsoBookmarked } from './recommendations';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('fetchAlsoBookmarked', () => {
  it('GETs /api/v1/works/{url_id}/also-bookmarked with the encoded url_id', async () => {
    mockFetch.mockResolvedValueOnce(
      jsonResponse({
        err: 0,
        url_id: 'a fic with spaces',
        items: [{ url_id: 'b', work_id: 2, title: 'B', author: 'A', words: 1, status: 'complete', site_domain: 'ao3', cooccur_count: 5 }],
      }),
    );

    const res = await fetchAlsoBookmarked('a fic with spaces');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('/v1/works/a%20fic%20with%20spaces/also-bookmarked'),
      expect.objectContaining({ credentials: 'include' }),
    );
    expect(res.err).toBe(0);
    expect(res.items).toHaveLength(1);
    expect(res.items[0].cooccur_count).toBe(5);
  });

  it('returns an empty items list when there is no co-occurrence data', async () => {
    mockFetch.mockResolvedValueOnce(jsonResponse({ err: 0, url_id: 'x', items: [] }));

    const res = await fetchAlsoBookmarked('x');
    expect(res.items).toEqual([]);
  });

  it('throws on a non-OK response', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500, json: async () => ({}) });
    await expect(fetchAlsoBookmarked('x')).rejects.toThrow(/HTTP 500/);
  });
});
