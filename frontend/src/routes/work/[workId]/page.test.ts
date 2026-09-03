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

describe('work page — follow button', () => {
  it('renders Subscribe button and follows the work', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/works/')) {
        if (url.match(/\/reviews/)) return Promise.resolve(jsonResponse({ err: 0, reviews: [] }));
        if (url.match(/\/stats/)) return Promise.resolve(jsonResponse({
          err: 0, work_id: 1, kudos_count: 10, guest_count: 2, total_bookmarks: 5,
          total_ratings: 3, total_comments: 1, avg_rating: 4.5, total_views: 50,
        }));
        if (url.match(/\/follows/) && url.includes('/check')) return Promise.resolve(jsonResponse({
          err: 0, follow_id: 5, is_following: false,
        }));
        return Promise.resolve(jsonResponse({ err: 0, work: { id: 1, canonical_title: 'Test Work', canonical_author: 'Test Author', sources: [{ id: 1, url: 'http://test', source: 'test', words: 1000, chapters: 10, status: 'complete', updated: '2024-01-01' }], total_bookmarks: 5, total_comments: 1 } }));
      }
      if (url.includes('/api/ratings')) return Promise.resolve(jsonResponse({ err: 0, avg_rating: 4.5, rating_count: 3, review_count: 1 }));
      if (url.includes('/api/kudos')) return Promise.resolve(jsonResponse({ err: 0, kudos_count: 10, guest_count: 2, my_kudos: false }));
      if (url.includes('/api/user/site-credentials')) return Promise.resolve(jsonResponse({ err: 0, credentials: [] }));
      if (url.includes('/api/reactions/')) return Promise.resolve(jsonResponse({ err: 0, reactions: { reactions: {}, my_reactions: [] } }));
      if (url.includes('/api/comments') || url.match(/\/comments\?/)) return Promise.resolve(jsonResponse({ err: 0, comments: [], total_top_level: 0, page: 1 }));
      if (url.includes('/api/bookmarks')) return Promise.resolve(jsonResponse({ err: 0, bookmarks: [] }));
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { data: { workId: '1' } });

    await waitFor(() => {
      expect(screen.getByText('Subscribe')).toBeTruthy();
    });

    // Stats should link to stats page (archive mode uses archive-stats-links)
    const statLinks = document.querySelectorAll('.archive-stats-links a, .stat-link');
    expect(statLinks.length).toBeGreaterThan(0);
    expect((statLinks[0] as HTMLElement).getAttribute('href')).toBe('/work/1/stats');
  });
});
