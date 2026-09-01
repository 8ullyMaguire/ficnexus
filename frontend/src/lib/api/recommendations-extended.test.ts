import { describe, it, expect, vi, beforeEach } from 'vitest';
import { fetchPersonalRecs, fetchAlsoBookmarked } from './recommendations';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('fetchPersonalRecs', () => {
  it('GETs /api/recommendations/personal with credentials', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, enough_data: false, recs: [], based_on: [] }),
    );
    const res = await fetchPersonalRecs();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/recommendations/personal');
    expect(mockFetch.mock.calls[0][1].credentials).toBe('include');
    expect(res.enough_data).toBe(false);
  });

  it('throws on non-OK responses', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 500, json: async () => ({}) });
    await expect(fetchPersonalRecs()).rejects.toThrow('HTTP 500');
  });
});

describe('fetchAlsoBookmarked', () => {
  it('returns the co-bookmarked items', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, url_id: 'x', items: [{ url_id: 'y', work_id: 1, title: 'Y', author: 'A', words: 1, status: 'c', site_domain: 'ao3', cooccur_count: 2 }] }),
    );
    const res = await fetchAlsoBookmarked('x');
    expect(res.items).toHaveLength(1);
  });
});

describe('isPersonalizedRecsEnabled', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  it('defaults to true when prefs fetch returns 401 (logged-out)', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 401, json: async () => ({ err: 401 }) });
    const { isPersonalizedRecsEnabled } = await import('./recommendations');
    expect(await isPersonalizedRecsEnabled()).toBe(true);
  });

  it('defaults to true when recs.personalized key is absent', async () => {
    mockFetch.mockResolvedValueOnce(okJson([{ key: 'theme.dark', value: 'true' }]));
    const { isPersonalizedRecsEnabled } = await import('./recommendations');
    expect(await isPersonalizedRecsEnabled()).toBe(true);
  });

  it('returns true when value is "true"', async () => {
    mockFetch.mockResolvedValueOnce(okJson([{ key: 'recs.personalized', value: 'true' }]));
    const { isPersonalizedRecsEnabled } = await import('./recommendations');
    expect(await isPersonalizedRecsEnabled()).toBe(true);
  });

  it('returns false only when value is exactly "false"', async () => {
    mockFetch.mockResolvedValueOnce(okJson([{ key: 'recs.personalized', value: 'false' }]));
    const { isPersonalizedRecsEnabled } = await import('./recommendations');
    expect(await isPersonalizedRecsEnabled()).toBe(false);
  });

  it('returns true on network failure (never blocks the personal endpoint)', async () => {
    mockFetch.mockRejectedValueOnce(new Error('offline'));
    const { isPersonalizedRecsEnabled } = await import('./recommendations');
    expect(await isPersonalizedRecsEnabled()).toBe(true);
  });
});
