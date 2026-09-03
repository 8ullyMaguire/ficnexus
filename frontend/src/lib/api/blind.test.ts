import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getBlindDate, revealBlindDate } from './blind';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('getBlindDate', () => {
  it('GETs /api/blind-date with no exclude when the list is empty', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, fic: { url_id: 'f1', description: 'd', words: 1, chapters: 1, status: 'complete', tropes: [] } }),
    );
    const res = await getBlindDate();
    expect(mockFetch.mock.calls[0][0]).toBe('/api/blind-date');
    expect(res.fic?.url_id).toBe('f1');
  });

  it('encodes the exclude list as a comma-joined query param', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, fic: null }));
    const res = await getBlindDate(['fic 1', 'fic2']);
    expect(mockFetch.mock.calls[0][0]).toBe('/api/blind-date?exclude=fic%201%2Cfic2');
    expect(res.fic).toBeNull();
  });

  it('throws on non-OK responses', async () => {
    mockFetch.mockResolvedValue({ ok: false, status: 500, json: async () => ({}) });
    await expect(getBlindDate()).rejects.toThrow('API error 500');
  });
});

describe('revealBlindDate', () => {
  it('GETs the reveal endpoint with url_id + signed nonce/sig', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, fic: { url_id: 'f1', title: 'Secret', author: 'A', source: 's', fandom: null } }),
    );
    const res = await revealBlindDate('f1', { nonce: 'n1', sig: 's1' });
    const url = mockFetch.mock.calls[0][0] as string;
    expect(url).toContain('/api/blind-date/reveal?');
    expect(url).toContain('url_id=f1');
    expect(url).toContain('nonce=n1');
    expect(url).toContain('sig=s1');
    expect(res.fic.title).toBe('Secret');
  });
});
