import { describe, it, expect, vi, beforeEach } from 'vitest';

// The new per-fic suggestions client (/api/fic-suggestions) attaches the JWT
// via authHeaders() from './social'. Mock './social' so tests don't touch the
// real localStorage token plumbing.
vi.mock('./social', () => ({
  authHeaders: () => ({ Authorization: 'Bearer test-token' }),
}));

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

async function load() {
  return await import('./ficSuggestions');
}

describe('ficSuggestions client', () => {
  it('listFicSuggestions GETs /api/fic-suggestions?url_id=', async () => {
    const { listFicSuggestions } = await load();
    mockFetch.mockResolvedValue(
      okJson({ err: 0, url_id: 'fic-1', suggestions: [{ id: 1, score: 2 }] }),
    );
    const res = await listFicSuggestions('fic-1');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/fic-suggestions?url_id=fic-1');
    expect(res.suggestions).toHaveLength(1);
  });

  it('listFicSuggestions surfaces non-OK as err payload', async () => {
    const { listFicSuggestions } = await load();
    mockFetch.mockResolvedValue({ ok: false, status: 500 });
    const res = await listFicSuggestions('fic-1');
    expect(res.err).toBe(500);
  });

  it('createFicSuggestion POSTs with auth header and suggested_url_id', async () => {
    const { createFicSuggestion } = await load();
    mockFetch.mockResolvedValue(okJson({ err: 0, suggestion_id: 9 }));
    const res = await createFicSuggestion('fic-1', { suggested_url_id: 'fic-2', comment: 'great' });
    expect(res.suggestion_id).toBe(9);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/fic-suggestions');
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe('Bearer test-token');
    const body = JSON.parse(init.body);
    expect(body.url_id).toBe('fic-1');
    expect(body.suggested_url_id).toBe('fic-2');
    expect(body.comment).toBe('great');
  });

  it('createFicSuggestion sends url (scrape) mode without suggested_url_id', async () => {
    const { createFicSuggestion } = await load();
    mockFetch.mockResolvedValue(okJson({ err: 0, suggestion_id: 10 }));
    await createFicSuggestion('fic-1', { url: 'https://ao3.org/works/123' });
    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.url).toBe('https://ao3.org/works/123');
    expect(body.suggested_url_id).toBeUndefined();
  });

  it('voteFicSuggestion POSTs vote to /api/fic-suggestions/{id}/vote', async () => {
    const { voteFicSuggestion } = await load();
    mockFetch.mockResolvedValue(okJson({ err: 0, new_score: 4, my_vote: 1 }));
    const res = await voteFicSuggestion(5, 1);
    expect(res.new_score).toBe(4);
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/fic-suggestions/5/vote');
    expect(JSON.parse(init.body).vote).toBe(1);
  });

  it('removeFicSuggestion POSTs to /api/fic-suggestions/{id}/remove', async () => {
    const { removeFicSuggestion } = await load();
    mockFetch.mockResolvedValue(okJson({ err: 0, removed: true }));
    const res = await removeFicSuggestion(5);
    expect(res.removed).toBe(true);
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/fic-suggestions/5/remove');
    expect(mockFetch.mock.calls[0][1].method).toBe('POST');
  });
});
