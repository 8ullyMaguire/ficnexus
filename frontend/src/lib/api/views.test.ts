import { describe, it, expect, vi, beforeEach } from 'vitest';
import { createView, fetchViews } from './views';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => mockFetch.mockReset());

describe('saved search views API', () => {
  it('creates a saved filter with an explicit query payload', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({ id: 4, name: 'Long gen', query: { min_words: 100000 }, pinned: false }) });
    const result = await createView('Long gen', { min_words: 100000 });
    expect(result.name).toBe('Long gen');
    expect(mockFetch).toHaveBeenCalledWith('/api/me/views', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ name: 'Long gen', query: { min_words: 100000 }, pinned: false }),
    }));
  });

  it('loads saved filters from the current user endpoint', async () => {
    mockFetch.mockResolvedValue({ ok: true, json: async () => [{ id: 1, name: 'Gen', query: {}, pinned: true }] });
    await expect(fetchViews()).resolves.toEqual([{ id: 1, name: 'Gen', query: {}, pinned: true }]);
    expect(mockFetch).toHaveBeenCalledWith('/api/me/views', expect.objectContaining({ credentials: 'include' }));
  });
});
