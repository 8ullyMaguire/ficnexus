import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@testing-library/svelte';
import { goto } from '$app/navigation';

// ---------------------------------------------------------------------------
// Navigation integration tests.
//
// These render the REAL root layout (+layout.svelte) with a REAL route page
// as its child, then simulate what a user does in the browser: click the nav
// link in the Discover dropdown → the routed page must actually render its
// visible content. No page component is mocked: we exercise the layout's
// routePages decision (isRoutePage), the dropdown, and the page's own mount
// logic together.
//
// IMPORTANT: /roadmap and /tropes currently FAIL on purpose — the layout's
// hardcoded routePages list omits them, so the layout renders the home
// dashboard instead of the routed page (the "links don't render anything"
// bug). These tests pin that regression; the fix (add the paths to
// routePages) flips them green. All other routes must pass.
// ---------------------------------------------------------------------------

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// ── SvelteKit mocks ────────────────────────────────────────────────────────
// A tiny reactive page store: tests setCurrentUrl() to simulate navigation.
// goto() is wired to it so clicking a nav link drives the SAME route change
// SvelteKit performs, and the harness re-renders the routed child.
let currentUrl = 'http://localhost/';
let pageSubscribers: Array<(v: unknown) => void> = [];

function emitPage() {
  const value = { url: new URL(currentUrl) };
  pageSubscribers.forEach((fn) => fn(value));
}

function setCurrentUrl(url: string) {
  currentUrl = url;
  emitPage();
}

vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: (v: unknown) => void) => {
      pageSubscribers.push(fn);
      fn({ url: new URL(currentUrl) });
      return () => {
        pageSubscribers = pageSubscribers.filter((f) => f !== fn);
      };
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn((url: string) => {
    // Mirror SvelteKit: navigate in-app (no full reload). Resolve relative
    // paths (e.g. "/trending") against the current origin for the page store.
    const resolved = url.startsWith('http') ? url : new URL(url, currentUrl).toString();
    setCurrentUrl(resolved);
  }),
}));

// ── HTTP stubs ─────────────────────────────────────────────────────────────
// A tiny fetch router: return realistic HTTP stubs per endpoint. Components
// mount and fetch real API paths (auth/me, trending, blind-date, search...).
function stubFetch() {
  mockFetch.mockImplementation((url: RequestInfo | URL, init?: RequestInit) => {
    const u = String(url);
    if (u.startsWith('/api/auth/me')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, user: null }),
      });
    }
    if (u.startsWith('/api/curator/authors/pending')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, proposals: [] }),
      });
    }
    if (u.startsWith('/api/curator/flags')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, flags: [] }),
      });
    }
    if (u.startsWith('/api/trending/tags')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, trending_tags: [{ id: 1, name: 'Angst', tag_type_id: 4, fic_count: 7 }] }),
      });
    }
    if (u.startsWith('/api/trending')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({
          err: 0,
          trending: [
            { url_id: 'trend1', title: 'Trending Fic A', author: 'Author A', words: 10000, chapters: 10, status: 'complete', downloads: 42, requests: 100 },
          ],
        }),
      });
    }
    if (u.startsWith('/api/blind-date')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({
          err: 0,
          fic: {
            url_id: 'blind1',
            description: 'A mystery story about a hidden title.',
            words: 5000,
            chapters: 3,
            status: 'complete',
            tropes: ['Dark Character A'],
          },
        }),
      });
    }
    if (u.startsWith('/api/recommendations/personal')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, enough_data: false, recs: [], based_on: [] }),
      });
    }
    if (u.startsWith('/api/locales')) {
      return Promise.resolve({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }] }) });
    }
    if (u.startsWith('/api/search')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ total: 0, page: 1, per_page: 20, results: [], facets: null }),
      });
    }
    if (u.startsWith('/api/search/ask')) {
      // POST Ask the Archive — the /ask route page fetches this on submit.
      return Promise.resolve({
        ok: true,
        json: async () => ({
          total: 0,
          page: 1,
          per_page: 20,
          results: [],
          facets: null,
          translated: true,
          nl_query: 'anything',
          applied_params: { q: 'anything', min_words: null, max_words: null, complete: null, source: null },
        }),
      });
    }
    if (u.startsWith('/api/leaderboard')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({ err: 0, leaderboard: [{ user_id: 1, username: 'curator1', score: 100, rank: 1 }] }),
      });
    }
    if (u.startsWith('/api/roadmap/arena')) {
      return Promise.resolve({
        ok: true,
        json: async () => ({
          err: 0,
          message: '',
          clusters: [
            { id: 1, text: 'Dark mode for night reading', matches_played: 14, elo_rating: 1600 },
            { id: 2, text: 'Send EPUBs to Kindle', matches_played: 9, elo_rating: 1540 },
          ],
        }),
      });
    }
    // Default: any other endpoint returns a harmless empty body.
    return Promise.resolve({ ok: true, json: async () => ({ err: 0 }) });
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  stubFetch();
  localStorage.clear();
  sessionStorage.clear();
  currentUrl = 'http://localhost/';
  pageSubscribers = [];
});

async function renderRoute(route: string) {
  setCurrentUrl(`http://localhost${route}`);
  const { default: RouteHarness } = await import('$lib/test/RouteHarness.svelte');
  return render(RouteHarness);
}

// Open the Discover dropdown (the real NavDropdown component) and click a
// link by its visible label. Scope to the open dropdown menu so we never
// match page content links (e.g. the dashboard's "Trending Fic A" links).
// jsdom cannot perform real navigation, so we simulate SvelteKit's client-
// side router: prevent the anchor's default full-page navigation, then drive
// the mocked goto() which updates the $page store (the layout + harness
// re-render the routed child). Returns the href so we can assert navigation.
async function clickDiscoverLink(label: string): Promise<string | null> {
  const discoverTrigger = screen.getByRole('button', { name: /discover/i });
  await fireEvent.click(discoverTrigger);
  const menu = await screen.findByRole('menu');
  const link = await within(menu).findByRole('link', { name: new RegExp(label, 'i') });
  const href = link.getAttribute('href');
  // Intercept the click like SvelteKit's router does for internal links.
  // A capture-phase listener calls preventDefault BEFORE jsdom's default
  // navigation action runs, avoiding the "Not implemented: navigation" noise.
  const cancel = (e: Event) => e.preventDefault();
  link.addEventListener('click', cancel, { capture: true });
  try {
    await fireEvent.click(link);
  } finally {
    link.removeEventListener('click', cancel, { capture: true });
  }
  if (href) await goto(href);
  return href;
}

describe('Navigation integration: Discover dropdown routes render their pages', () => {
  it('renders the layout with the Discover dropdown', async () => {
    await renderRoute('/');
    expect(screen.getByRole('button', { name: /discover/i })).toBeTruthy();
    expect(screen.getByRole('link', { name: /ficHub home/i })).toBeTruthy();
  });

  it('/trending renders the Trending page through the layout', async () => {
    await renderRoute('/trending');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '🔥 Trending' })).toBeTruthy();
    });
    // The page's own content (not the dashboard's trending section).
    await waitFor(() => {
      expect(screen.getByText('Trending Fic A')).toBeTruthy();
    });
  });

  it('/search renders the Search page through the layout', async () => {
    await renderRoute('/search');
    expect(screen.getByPlaceholderText(/search titles/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: /search/i })).toBeTruthy();
  });

  it('/ask renders the Ask the Archive page through the layout', async () => {
    await renderRoute('/ask');
    expect(screen.getByRole('heading', { name: 'Ask the Archive' })).toBeTruthy();
    expect(screen.getByLabelText(/natural language search query/i)).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Ask' })).toBeTruthy();
  });

  it('/leaderboard renders the Rankings page through the layout', async () => {
    await renderRoute('/leaderboard');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '🏆 Rankings' })).toBeTruthy();
    });
    await waitFor(() => {
      expect(screen.getByText('curator1')).toBeTruthy();
    });
  });

  it('/roadmap renders the Roadmap Arena page through the layout', async () => {
    await renderRoute('/roadmap');
    // The arena heading + a cluster from the stubbed API must be visible.
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /roadmap arena/i })).toBeTruthy();
    });
    // The arena fetch resolves after mount; wait for the cluster content
    // (not just the heading, which renders synchronously).
    await waitFor(() => {
      expect(screen.getByText('Dark mode for night reading')).toBeTruthy();
    });
    expect(screen.getByText('Send EPUBs to Kindle')).toBeTruthy();
  });

  it('/blind-date renders the Blind Date page through the layout', async () => {
    await renderRoute('/blind-date');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /blind date with a fic/i })).toBeTruthy();
    });
    // BlindDateCard does not auto-draw; the user clicks "Draw a fic".
    const drawBtn = screen.getByRole('button', { name: /draw a fic/i });
    await fireEvent.click(drawBtn);
    await waitFor(() => {
      expect(screen.getByText(/a mystery story about a hidden title/i)).toBeTruthy();
    });
  });
});

describe('Navigation integration: clicking the Discover nav link mounts the page', () => {
  it('clicking the /trending nav link renders the Trending page', async () => {
    await renderRoute('/');
    const href = await clickDiscoverLink('Trending');
    expect(href).toBe('/trending');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: '🔥 Trending' })).toBeTruthy();
    });
    await waitFor(() => {
      expect(screen.getByText('Trending Fic A')).toBeTruthy();
    });
  });

  it('clicking the /roadmap nav link renders the Roadmap Arena', async () => {
    await renderRoute('/');
    const href = await clickDiscoverLink('Roadmap');
    expect(href).toBe('/roadmap');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /roadmap arena/i })).toBeTruthy();
    });
  });

  it('clicking the /blind-date nav link renders the Blind Date page', async () => {
    await renderRoute('/');
    const href = await clickDiscoverLink('Blind Date');
    expect(href).toBe('/blind-date');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /blind date with a fic/i })).toBeTruthy();
    });
  });

  it('clicking the "Ask the Archive" nav link renders the Ask page', async () => {
    await renderRoute('/');
    const href = await clickDiscoverLink('Ask the Archive');
    expect(href).toBe('/ask');
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Ask the Archive' })).toBeTruthy();
    });
  });
});
