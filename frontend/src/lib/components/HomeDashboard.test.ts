import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const gotoMock = vi.fn();

// Controllable auth mock — HomeDashboard imports `auth` directly.
const authState = { isLoggedIn: false };
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: new Proxy({ init: async () => {} }, {
    get(_t, p) { return p === 'isLoggedIn' ? authState.isLoggedIn : undefined; },
  }),
}));
// SvelteKit navigation — mocked so the dashboard's goto import resolves.
vi.mock('$app/navigation', () => ({
  goto: gotoMock,
}));
// Mock only isPersonalizedRecsEnabled (pref-aware flag), keep fetchPersonalRecs
// real so it calls global fetch. prefsEnabled toggles the pref; logged-in state
// controls whether personal fetch happens at all (anon → no personal fetch).
let prefsEnabled = true;
vi.mock('$lib/api/recommendations', async (importOriginal) => {
  const actual = await importOriginal() as Record<string, unknown>;
  return {
    ...actual,
    isPersonalizedRecsEnabled: async () => prefsEnabled,
  };
});

beforeEach(() => {
  mockFetch.mockReset();
  gotoMock.mockReset();
  sessionStorage.clear();
  localStorage.clear();
  authState.isLoggedIn = false;
  prefsEnabled = true;
});

async function loadDashboard() {
  return await import('$lib/components/HomeDashboard.svelte');
}

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function trendResp(items: unknown[] = []) {
  return jsonResponse({ err: 0, trending: items });
}

const sampleRec = {
  url_id: 'r1',
  title: 'My Story',
  author: 'Me',
  words: 100,
  chapters: 1,
  status: 'complete',
  site_domain: 'ao3',
  summary: '',
  score: 1,
  download_urls: {},
};

describe('HomeDashboard', () => {
  // --- Anonymous users: no personal fetch, trending only (1 fetch) ---

  it('renders the compact download input', async () => {
    mockFetch.mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    expect(screen.getByPlaceholderText(/story url/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /download/i })).toBeTruthy();
  });

  it('does not render the Blind Date card or the Surprise me button', async () => {
    mockFetch.mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    expect(screen.queryByText(/blind date with a fic/i)).toBeNull();
    expect(screen.queryByRole('button', { name: /surprise me/i })).toBeNull();
  });

  it('hands the pasted URL to the /download page via sessionStorage on submit', async () => {
    mockFetch.mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    const input = screen.getByPlaceholderText(/story url/i);
    const url = 'https://www.royalroad.com/fiction/181303/gifted';
    await fireEvent.input(input, { target: { value: url } });
    await fireEvent.click(screen.getByRole('button', { name: /download/i }));

    expect(sessionStorage.getItem('fichub_dl_url')).toBe(url);
    expect(gotoMock).toHaveBeenCalledWith('/download');
  });

  it('shows the fallback hint when enough_data is false', async () => {
    // Anonymous + no recs → falls back to the "bookmark a few fics" hint.
    mockFetch.mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText(/bookmark a few fics/i)).toBeTruthy();
    });
  });

  it('renders trending independently when the recs endpoint rejects', async () => {
    // Personal rejected (anon path), trending available.
    mockFetch.mockResolvedValueOnce(
      trendResp([
        {
          url_id: 't1',
          title: 'Trending Fic',
          author: 'Author2',
          words: 10000,
          chapters: 5,
          status: 'ongoing',
          requests: 100,
          downloads: 50,
        },
      ]),
    );
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Trending Fic')).toBeTruthy();
    });
    // 4-level chain: personal (rejected) → trending → popular → recently added
    expect(screen.getAllByText('Trending Fic').length).toBeGreaterThan(0);
  });

  // --- Logged-in users: personal fetch present (2 fetches) ---

  it('shows "Curator\'s pick" when the pluggable engine reports curator influence', async () => {
    authState.isLoggedIn = true;
    mockFetch
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          enough_data: true,
          recs: [{ ...sampleRec, url_id: 'c1', title: 'Curated Story', author: 'Curator' }],
          based_on: [],
          curator_alpha: 0.9,
          strategies: [{ name: 'cooccur', contributed: true, count: 1, duration_ms: 2 }],
        }),
      )
      .mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText(/curator's pick/i)).toBeTruthy();
    });
  });

  it('keeps "Recommended for you" when no curator influence is reported', async () => {
    authState.isLoggedIn = true;
    mockFetch
      .mockResolvedValueOnce(jsonResponse({ err: 0, enough_data: true, recs: [sampleRec], based_on: [] }))
      .mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText(/recommended for you/i)).toBeTruthy();
    });
  });

  it('renders personalized recs with based_on when enough_data is true', async () => {
    authState.isLoggedIn = true;
    mockFetch
      .mockResolvedValueOnce(
        jsonResponse({
          err: 0,
          enough_data: true,
          recs: [
            {
              url_id: 'rec1',
              title: 'Recommended Story',
              author: 'Author1',
              words: 30000,
              chapters: 8,
              status: 'complete',
              site_domain: 'royalroad.com',
              summary: 'A similar story',
              score: 0.8,
              community_score: 3,
              download_urls: {},
            },
          ],
          based_on: [{ title: 'Bookmarked Fic', url_id: 'b1' }],
        }),
      )
      .mockResolvedValueOnce(trendResp());
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Recommended Story')).toBeTruthy();
    });
    expect(screen.getByText(/because you bookmarked/i)).toBeTruthy();
    expect(screen.getByText(/bookmarked: Bookmarked Fic/i)).toBeTruthy();
  });

  it('respects recs.personalized=false pref: skips personal fetch, falls back to trending', async () => {
    // Logged-in user opted OUT of personalized recs.
    authState.isLoggedIn = true;
    prefsEnabled = false;
    mockFetch.mockImplementation(async (input: unknown) => {
      const u = String(input);
      if (u.includes('/trending')) {
        return trendResp([
          {
            url_id: 't1',
            title: 'Trending Fic',
            author: 'Author2',
            words: 10000,
            chapters: 5,
            status: 'ongoing',
            requests: 100,
            downloads: 50,
          },
        ]);
      }
      return jsonResponse({});
    });
    const { default: Dashboard } = await loadDashboard();
    render(Dashboard);

    // The opted-out user must NOT hit the personal endpoint; trending renders.
    await waitFor(() => {
      expect(screen.getByText('Trending Fic')).toBeTruthy();
    });
    expect(screen.queryByText('Hidden')).toBeNull();
  });
});
