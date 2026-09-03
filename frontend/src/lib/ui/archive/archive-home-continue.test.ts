// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const gotoMock = vi.fn();
vi.mock('$app/navigation', () => ({ goto: gotoMock }));

// Controllable auth-store fake — ArchiveHome only reads `isLoggedIn`
// (and awaits `init()`), so a plain object with runes-like getters works.
const authState = {
  isLoggedIn: false,
  user: null as unknown,
  level: 0,
};
const initMock = vi.fn(async () => {});

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    get isLoggedIn() {
      return authState.isLoggedIn;
    },
    get user() {
      return authState.user;
    },
    get level() {
      return authState.level;
    },
    init: (...args: unknown[]) => initMock(...(args as [])),
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

function historyBody() {
  return {
    err: 0,
    history: [
      { id: 11, work_id: 1, url_id: 'w-aaa', title: 'A Continued Story', author: 'Author A', chapter_num: 4, visited_at: new Date(Date.now() - 3600_000).toISOString() },
      { id: 12, work_id: 2, url_id: 'w-bbb', title: 'Another Tale', author: 'Author B', chapter_num: null, visited_at: new Date(Date.now() - 86400_000).toISOString() },
    ],
    total: 2,
    limit: 3,
    offset: 0,
  };
}

beforeEach(() => {
  mockFetch.mockReset();
  gotoMock.mockReset();
  initMock.mockClear();
  authState.isLoggedIn = false;
  authState.user = null;
  authState.level = 0;
  localStorage.clear();
  sessionStorage.clear();
});

function mockUrls(opts: { loggedIn: boolean; history?: unknown; embRecs?: boolean }) {
  mockFetch.mockImplementation(async (input: unknown) => {
    const u = String(input);
    if (u.startsWith('/api/auth/me')) {
      return opts.loggedIn
        ? jsonResponse({ err: 0, user: { id: 7, username: 'histuser', role: 0, reputation: 0, email: null } })
        : jsonResponse({ err: 401, msg: 'Login required' });
    }
    if (u.startsWith('/api/reading/history')) {
      if (!opts.loggedIn) return { ok: false, status: 401, json: async () => ({ err: 401 }) };
      return jsonResponse(opts.history ?? { err: 0, history: [], total: 0, limit: 3, offset: 0 });
    }
    if (u.startsWith('/api/search')) return jsonResponse({ results: [] });
    if (u.startsWith('/api/trending')) return jsonResponse({ err: 0, trending: [] });
    if (u.startsWith('/api/recommendations/personal')) {
      return jsonResponse({ err: 0, enough_data: false, recs: [], based_on: [] });
    }
    if (u.startsWith('/api/recommendations/embeddings')) {
      return opts.loggedIn && opts.embRecs
        ? jsonResponse({
            err: 0,
            enough_data: true,
            recs: [
              { url_id: 'emb-1', title: 'Embeddings Pick', author: 'Rec Author', words: 1200, chapters: 3, status: 'complete', site_domain: 'ao3', summary: 'A similar story.', score: 0.91, community_score: 0, download_urls: {} },
            ],
            based_on: [{ title: 'A Continued Story', url_id: 'w-aaa' }],
          })
        : jsonResponse({ err: 0, enough_data: false, recs: [], based_on: [] });
    }
    return jsonResponse({});
  });
}

async function loadHome() {
  return (await import('./ArchiveHome.svelte')).default;
}

describe('ArchiveHome continue-reading card', () => {
  it('shows recent history with Continue links when signed in', async () => {
    authState.isLoggedIn = true;
    mockUrls({ loggedIn: true, history: historyBody() });
    const Home = await loadHome();
    render(Home);

    expect(await screen.findByText('Continue Reading')).toBeTruthy();
    const link = screen.getByText('A Continued Story') as HTMLAnchorElement;
    expect(link.getAttribute('href')).toBe('/read/w-aaa');
    const cont = screen.getAllByText('Continue →')[0] as HTMLAnchorElement;
    expect(cont.getAttribute('href')).toBe('/read/w-aaa');
  });

  it('hides the section entirely when signed out (no history fetch)', async () => {
    mockUrls({ loggedIn: false });
    const Home = await loadHome();
    render(Home);

    await waitFor(() => {
      expect(mockFetch.mock.calls.some(([u]) => String(u).startsWith('/api/trending'))).toBe(true);
    });
    await new Promise((r) => setTimeout(r, 30));
    expect(screen.queryByText('Continue Reading')).toBeNull();
    expect(mockFetch.mock.calls.some(([u]) => String(u).startsWith('/api/reading/history'))).toBe(false);
  });

  it('hides the section when signed in but history is empty', async () => {
    authState.isLoggedIn = true;
    mockUrls({ loggedIn: true, history: { err: 0, history: [], total: 0, limit: 3, offset: 0 } });
    const Home = await loadHome();
    render(Home);

    await screen.findByText('Recent Works');
    await waitFor(() => {
      expect(mockFetch.mock.calls.some(([u]) => String(u).startsWith('/api/reading/history'))).toBe(true);
    });
    await new Promise((r) => setTimeout(r, 30));
    expect(screen.queryByText('Continue Reading')).toBeNull();
  });

  it('shows the embeddings Recommended-for-You section when backend has data', async () => {
    authState.isLoggedIn = true;
    mockUrls({ loggedIn: true, embRecs: true });
    const Home = await loadHome();
    render(Home);

    expect(await screen.findByRole('heading', { name: 'Recommended for You' })).toBeTruthy();
    const pick = (await screen.findByText('Embeddings Pick')) as HTMLAnchorElement;
    expect(pick.getAttribute('href')).toBe('/works/emb-1');
  });

  it('hides the embeddings section when not enough embedded data', async () => {
    authState.isLoggedIn = false;
    mockUrls({ loggedIn: false, embRecs: false });
    const Home = await loadHome();
    render(Home);

    await screen.findByText('Recent Works');
    await new Promise((r) => setTimeout(r, 30));
    expect(screen.queryByRole('heading', { name: 'Recommended for You' })).toBeNull();
  });
});
