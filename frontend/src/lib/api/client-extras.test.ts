import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ApiError, fetchExport, fetchMeta } from './client';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('client extras', () => {
  it('fetchMeta GETs /api/meta with the encoded URL', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, url_id: 'x', meta: { title: 'T', author: 'A', words: 1, chapters: 1, status: 'complete' } }),
    );
    const res = await fetchMeta('https://ao3.org/works/1');
    expect(String(mockFetch.mock.calls[0][0])).toBe('/api/meta?q=https%3A%2F%2Fao3.org%2Fworks%2F1');
    expect(res.url_id).toBe('x');
  });

  it('ApiError carries status and body', () => {
    const err = new ApiError(418, 'teapot');
    expect(err.status).toBe(418);
    expect(err.body).toBe('teapot');
    expect(err.message).toBe('API error 418: teapot');
    expect(err.name).toBe('ApiError');
  });

  it('sends X-Client-ID header on every request', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, url_id: 'x', meta: null }));
    await fetchExport('u');
    const headers = mockFetch.mock.calls[0][1].headers as Headers;
    const headerId = headers.get('X-Client-ID');
    expect(headerId).toBeTruthy();
    // The client ID is persisted so subsequent requests reuse it.
    const persisted = localStorage.getItem('fichub_client_id');
    expect(persisted).toBe(headerId);
  });

  it('rejects with ApiError on HTTP failure and empty body', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 500, text: async () => '' });
    await expect(fetchExport('u')).rejects.toBeInstanceOf(ApiError);
  });
});
