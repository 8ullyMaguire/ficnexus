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

vi.mock('$lib/util', () => ({
  formatWords: (n: number) => `${n} words`,
  relativeTime: () => 'just now',
}));

// NOTE: no component mocks here. `{ default: {} }` stubs are not valid
// Svelte 5 components (crash: "default is not a function"); the real
// WorkBlurb/ArchiveButton render fine against the empty fixtures below.
// The page itself is a sections layout now — the old tab strip
// (Overview/Bookmarks/Collections/Activity/Quests) was removed in the
// 2026-08 dashboard redesign.

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

describe('dashboard page', () => {
  it('renders the dashboard sections after loading', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/reading-stats')) {
        return Promise.resolve(jsonResponse({ err: 0, total_works_read: 10, total_words_read: 50000, login_streak: 4 }));
      }
      if (url.includes('/unread-count')) {
        return Promise.resolve(jsonResponse({ err: 0, unread_count: 2 }));
      }
      if (url.includes('/streak')) {
        return Promise.resolve(jsonResponse({ err: 0, current_streak: 3, longest_streak: 7 }));
      }
      if (url.includes('/bookmarks')) {
        return Promise.resolve(jsonResponse({ err: 0, bookmarks: [] }));
      }
      if (url.includes('/lists')) {
        return Promise.resolve(jsonResponse({ err: 0, lists: [] }));
      }
      if (url.includes('/quests')) {
        return Promise.resolve(jsonResponse({ err: 0, quests: [] }));
      }
      if (url.includes('/feed')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/Dashboard/i)).toBeTruthy();
    });

    // AO3-style reading-stats block (dl.stats)
    await waitFor(() => {
      expect(screen.getByText('Works')).toBeTruthy();
    });
    expect(screen.getByText('Words')).toBeTruthy();
    expect(screen.getByText('50000 words')).toBeTruthy();

    // Section headings of the redesigned layout
    expect(screen.getByText('Reading Streak')).toBeTruthy();
    expect(screen.getByText('Recent Bookmarks')).toBeTruthy();
    expect(screen.getByText('Your Library')).toBeTruthy();
    expect(screen.getByText('Your Collections')).toBeTruthy();
    expect(screen.getByText('Recent Activity')).toBeTruthy();
    expect(screen.getByText('Current Quests')).toBeTruthy();
  });
});
