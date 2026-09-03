import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    user: { id: 1, username: 'alice', role: 0, reputation: 0, level: 12, exp: 4800, email: null, locale: 'en' },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

describe('collections page', () => {
  it('renders collection list and "New Collection" button', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/lists')) {
        return Promise.resolve(jsonResponse({
          err: 0,
          lists: [
            { id: 1, title: 'My Favorites', description: 'Top picks', item_count: 3, is_public: true },
            { id: 2, title: 'To Read Later', description: null, item_count: 5, is_public: false },
          ],
        }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { data: {} } as never);

    await waitFor(() => {
      expect(screen.getByText('My Favorites')).toBeTruthy();
      expect(screen.getByText('To Read Later')).toBeTruthy();
    });

    expect(screen.getByText('New Collection')).toBeTruthy();
  });
});
