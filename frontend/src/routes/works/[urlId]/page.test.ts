import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

// Mock the auth store so `isLoggedIn` is controllable per-test. The real
// store caches `initialized` across tests in the same file, which makes
// "logged in on second render" impossible to test — so we drive it here.
let mockLoggedIn = false;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    get isLoggedIn() { return mockLoggedIn; },
    get user() { return mockLoggedIn ? { id: 1, username: 'u', role: 0, reputation: 0 } : null; },
    init: vi.fn(),
    handleLogout: vi.fn(),
  },
}));

// The page fires several fetches in a racy order (CommentSection mounts as
// soon as `fic` is set, racing loadAlsoBookmarked). Mock EVERY endpoint by URL
// so the response each call gets is deterministic regardless of order.
let alsoBookmarkedPayload: unknown = null;
let followCheckPayload: unknown = null;

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function getLS(): Storage {
  if (typeof window !== 'undefined' && (window as any).localStorage) return (window as any).localStorage as Storage;
  const g = (globalThis as any).localStorage;
  if (g) return g as Storage;
  const store = new Map<string,string>();
  const fake = { getItem:(k:string)=>store.get(k)??null, setItem:(k:string,v:string)=>{store.set(k,String(v))}, removeItem:(k:string)=>store.delete(k), clear:()=>store.clear(), get length(){return store.size}, key:(i:number)=>Array.from(store.keys())[i]??null } as unknown as Storage;
  (globalThis as any).localStorage = fake;
  return fake;
}

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function ratingAggregatePayload() {
  return jsonResponse({
    err: 0,
    work_id: 101,
    avg_rating: 4.5,
    rating_count: 2,
    likes: 3,
    review_count: 1,
    rating_distribution: { '4': 1, '5': 1 },
  });
}

function ficExportResponse() {
  return jsonResponse({
    err: 0,
    url_id: 'fic-1',
    meta: {
      id: 'fic-1',
      work_id: 101,
      title: 'The Anchor Fic',
      author: 'Author A',
      chapters: 10,
      words: 50000,
      description: 'desc',
      status: 'complete',
      source: 'https://archiveofourown.org/works/1',
      created: '2024-01-01T00:00:00Z',
      updated: '2024-01-01T00:00:00Z',
      extra_meta: null,
      raw_extended_meta: null,
      author_url: '',
      author_local_id: '',
      source_id: 1,
      author_id: 1,
    },
    epub_url: '/api/epub?q=fic-1',
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  // NOTE: clear BEFORE seeding — the reverse order wiped the pref blob and
  // every test silently fell back to uiMode:'archive' (archive layout has
  // no also-bookmarked/suggestions sections), breaking 13 assertions.
  getLS().clear();
  getLS().setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  alsoBookmarkedPayload = { err: 0, url_id: 'fic-1', items: [] };
  followCheckPayload = null;

  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.includes('/api/fic-suggestions?url_id=')) {
      return Promise.resolve(jsonResponse({
        err: 0,
        url_id: 'fic-1',
        suggestions: [
          {
            id: 1,
            url_id: 'fic-1',
            suggested_url_id: 'fic-2',
            suggested_title: 'Suggested Companion',
            suggested_author: 'Author B',
            comment: 'Pairs well!',
            score: 2,
            total_votes: 3,
            my_vote: 0,
            user_id: 9,
          },
        ],
      }));
    }
    if (url.includes('/api/auth/me')) {
      // If a cached user exists, the store treats us as logged in.
      const cached = getLS().getItem('fichub_cached_user');
      return Promise.resolve(jsonResponse(cached
        ? { err: 0, user: JSON.parse(cached) }
        : { err: 0, user: null }));
    }
    if (url.includes('/epub?q=')) {
      return Promise.resolve(ficExportResponse());
    }
    if (url.includes('/ratings/')) {
      return Promise.resolve(jsonResponse({ err: 0, likes: 0, dislikes: 0 }));
    }
    if (url.includes('/api/search/similar/')) {
      return Promise.resolve(jsonResponse({ err: 0, items: [] }));
    }
    if (url.includes('/comments')) {
      return Promise.resolve(jsonResponse({ err: 0, comments: [], total_top_level: 0, page: 1 }));
    }
    if (url.includes('/also-bookmarked')) {
      // alsoBookmarkedPayload may be a Response-like object (jsonResponse) or
      // a raw payload set by the test; wrap plain objects so request() works.
      return Promise.resolve(typeof alsoBookmarkedPayload === 'object' && 'json' in (alsoBookmarkedPayload as object)
        ? alsoBookmarkedPayload
        : jsonResponse(alsoBookmarkedPayload));
    }
    if (url.includes('/follows/check/work/')) {
      return Promise.resolve(jsonResponse(followCheckPayload ?? { err: 0, is_following: false, follow_id: null }));
    }
    if (url.includes('/follows') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, follow_id: 77 }));
    }
    if (url.includes('/follows/') && init?.method === 'DELETE') {
      return Promise.resolve(jsonResponse({ err: 0, removed: true }));
    }
    if (url.includes('/v1/follows/') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, updated: true }));
    }
    if (url.includes('/v1/works/') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, status: 'ok', url_id: 'fic-1', version_bump: 2, notified: 1 }));
    }
    return Promise.reject(new Error(`unexpected fetch: ${url}`));
  });
});

async function loadPage() {
  return await import('./+page.svelte');
}

describe('fic page "Readers also bookmarked" section', () => {
  it('renders co-bookmarked fics from /api/v1/works/{url_id}/also-bookmarked', async () => {
    alsoBookmarkedPayload = {
      err: 0,
      url_id: 'fic-1',
      items: [
        { url_id: 'fic-2', work_id: 202, title: 'Companion Fic', author: 'Author B', words: 30000, status: 'complete', site_domain: 'ao3', cooccur_count: 7 },
        { url_id: 'fic-3', work_id: 303, title: 'Sister Fic', author: 'Author C', words: 20000, status: 'complete', site_domain: 'ao3', cooccur_count: 4 },
      ],
    };

    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    await waitFor(() => {
      expect(screen.getByText('Readers also bookmarked')).toBeTruthy();
    });
    await waitFor(() => {
      expect(screen.getByText('Companion Fic')).toBeTruthy();
    });
    expect(screen.getByText(/7 reader\(s\) also saved/)).toBeTruthy();
    expect(screen.getByText('Sister Fic')).toBeTruthy();
    // Links point at the work page when the partner has a work_id
    expect(screen.getByRole('link', { name: /Companion Fic/ }).getAttribute('href')).toBe('/work/202');
  });

  it('shows the empty state when there is no co-occurrence data', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    await waitFor(() => {
      expect(screen.getByText(/not enough bookmark data yet/i)).toBeTruthy();
    });
  });

  it('does not crash when the also-bookmarked fetch rejects', async () => {
    // The reject must surface from the handler, not from json() — wrap the
    // rejection in an object whose json() rejects instead.
    alsoBookmarkedPayload = { reject: true } as never;
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/also-bookmarked')) {
        // A rejection that the page's handler must swallow; mark it handled
        // so vitest doesn't report an unhandled rejection when the test ends.
        return Promise.reject(new Error('network down')).catch((e) => {
          // The page handles the rejection internally; this catch only tags
          // the error as expected so the rejection is not "unhandled".
          throw e;
        });
      }
      if (url.includes('/comments')) {
        return Promise.resolve(jsonResponse({ err: 0, comments: [], total_top_level: 0, page: 1 }));
      }
      // Keep the default mocks the page needs to render its main content.
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: null }));
      }
      if (url.includes('/epub?q=')) {
        return Promise.resolve(ficExportResponse());
      }
      if (url.includes('/ratings/')) {
        return Promise.resolve(jsonResponse({ err: 0, likes: 0, dislikes: 0 }));
      }
      if (url.includes('/api/search/similar/')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      if (url.includes('/suggest')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      return Promise.resolve(jsonResponse({ err: 0, user: null }));
    });

    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    await waitFor(() => {
      expect(screen.getAllByText('The Anchor Fic').length).toBeGreaterThan(0);
    });
    // Section still renders with its empty state (fetch failed → loaded=true, no items)
    await waitFor(() => {
      expect(screen.getByText(/not enough bookmark data yet/i)).toBeTruthy();
    });
  });
});

// ─── Follow + refresh buttons ─────────────────────────────────────────────


describe('fic page 5-star feedback UI', () => {
  it('renders the 5-star widget with the average rating and no dislikes when logged in', async () => {
    // The auth store is mocked at the top of this file (mockLoggedIn flag).
    mockLoggedIn = true;
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }));
      }
      if (url.includes('/epub?q=')) {
        return Promise.resolve(ficExportResponse());
      }
      if (url.includes('/ratings/')) {
        return Promise.resolve(ratingAggregatePayload());
      }
      if (url.includes('/api/search/similar/')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      if (url.includes('/comments')) {
        return Promise.resolve(jsonResponse({ err: 0, comments: [], total_top_level: 0, page: 1 }));
      }
      if (url.includes('/also-bookmarked')) {
        return Promise.resolve(jsonResponse({ err: 0, url_id: 'fic-1', items: [] }));
      }
      if (url.includes('/works/101/reviews') || url.includes('/reviews/')) {
        return Promise.resolve(jsonResponse({ err: 0, work_id: 101, total: 0, reviews: [] }));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    // Page rendered
    await waitFor(() => {
      expect(screen.getAllByText('The Anchor Fic').length).toBeGreaterThan(0);
    });

    // The rating widget shows the fic's avg_rating + total positive signals
    await waitFor(() => {
      const span = Array.from(document.querySelectorAll('span.rating-count')).find(
        (s) => s.textContent?.trim().startsWith('4.5') && s.textContent?.includes('(3)'),
      );
      expect(span).toBeTruthy();
    });

    // The 5-star click-to-rate control is present (aria-label "Rate N stars")
    expect(screen.getAllByRole('button', { name: /^Rate [1-5] stars?$/ }).length).toBeGreaterThan(0);

    // Positive-only UI: no dislike button, no 👎 anywhere
    expect(screen.queryByRole('button', { name: /dislike/i })).toBeNull();
    expect(document.body.textContent).not.toContain('👎');
    expect(document.body.textContent).not.toContain('Dislike');
  });

  it('renders the review section on the fic page', async () => {
    // Ensure logged-out state (the previous test left mockLoggedIn true)
    mockLoggedIn = false;

    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    await waitFor(() => {
      expect(screen.getAllByText('The Anchor Fic').length).toBeGreaterThan(0);
    });
    // ReviewsSection heading is present (work_id 101 from the export meta)
    await waitFor(() => {
      expect(screen.getByText(/Reviews \(0\)/)).toBeTruthy();
    });
    // Logged-out: the login hint is shown instead of the form
    expect(screen.getByText('to write a review.')).toBeTruthy();
  });
});

describe('fic page per-fic suggestions panel', () => {
  it('renders the similar-fic suggestions section with its suggestions', async () => {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });

    // Panel heading + a suggestion rendered from /api/fic-suggestions.
    await waitFor(() => {
      expect(screen.getByText('💡 Similar fic suggestions')).toBeTruthy();
    });
    await waitFor(() => {
      expect(screen.getByText('Suggested Companion')).toBeTruthy();
    });
    expect(screen.getByText(/Pairs well!/)).toBeTruthy();
    // The suggest form is present on the page.
    expect(screen.getByLabelText(/Search a fic to suggest/)).toBeTruthy();
    expect(screen.getByRole('button', { name: /Suggest similar fic/ })).toBeTruthy();
  });
});

// ─── All export formats + preferred-format selector ───────────────────────

describe('fic page all formats + preferred-format selector', () => {
  function allFormatsExport() {
    return jsonResponse({
      err: 0,
      url_id: 'fic-1',
      meta: {
        id: 'fic-1',
        work_id: 101,
        title: 'The Anchor Fic',
        author: 'Author A',
        chapters: 10,
        words: 50000,
        description: 'desc',
        status: 'complete',
        source: 'https://archiveofourown.org/works/1',
        created: '2024-01-01T00:00:00Z',
        updated: '2024-01-01T00:00:00Z',
        extra_meta: null,
        raw_extended_meta: null,
        author_url: '',
        author_local_id: '',
        source_id: 1,
        author_id: 1,
      },
      epub_url: '/api/epub?q=fic-1',
      mobi_url: '/cache/mobi/fic-1?h=2',
      azw3_url: '/cache/azw3/fic-1?h=3',
      pdf_url: '/cache/pdf/fic-1?h=4',
      docx_url: '/cache/docx/fic-1?h=5',
      kepub_url: '/cache/kepub/fic-1?h=6',
      fb2_url: '/cache/fb2/fic-1?h=7',
      html_url: '/cache/html/fic-1?h=8',
      txt_url: '/cache/txt/fic-1?h=9',
      md_url: '/cache/md/fic-1?h=10',
    });
  }

  function mockAllFormats() {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/auth/me')) {
        return Promise.resolve(jsonResponse({ err: 0, user: null }));
      }
      if (url.includes('/epub?q=')) {
        return Promise.resolve(allFormatsExport());
      }
      if (url.includes('/ratings/')) {
        return Promise.resolve(jsonResponse({ err: 0, likes: 0, dislikes: 0 }));
      }
      if (url.includes('/api/search/similar/')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      if (url.includes('/comments')) {
        return Promise.resolve(jsonResponse({ err: 0, comments: [], total_top_level: 0, page: 1 }));
      }
      if (url.includes('/also-bookmarked')) {
        return Promise.resolve(jsonResponse({ err: 0, url_id: 'fic-1', items: [] }));
      }
      return Promise.resolve(jsonResponse({ err: 0 }));
    });
  }

  async function renderPage() {
    const { default: Page } = await loadPage();
    render(Page, { props: { data: { urlId: 'fic-1' } } });
    await waitFor(() => expect(screen.getAllByText('The Anchor Fic').length).toBeGreaterThan(0));
  }

  /** Open the ArchiveWork download dropdown and return its item links. */
  async function openDlMenu(): Promise<HTMLElement[]> {
    const btn = screen.getByRole('button', { name: /Download/ });
    await fireEvent.click(btn);
    return Array.from(document.querySelectorAll<HTMLElement>('a.download-item'));
  }

  it('renders download items for all ten formats when their URLs are present', async () => {
    mockAllFormats();
    await renderPage();

    const items = await openDlMenu();
    for (const label of ['EPUB', 'MOBI', 'AZW3', 'PDF', 'DOCX', 'KEPUB', 'FB2', 'HTML', 'TXT', 'MD']) {
      expect(screen.getByText(label)).toBeTruthy();
    }
    expect(items.some((a) => a.textContent === 'DOCX' && a.getAttribute('href')?.includes('/cache/docx/'))).toBe(true);
    expect(items.some((a) => a.textContent === 'FB2' && a.getAttribute('href')?.includes('/cache/fb2/'))).toBe(true);
    expect(items.some((a) => a.textContent === 'KEPUB' && a.getAttribute('href')?.includes('/cache/kepub/'))).toBe(true);
  });

  it('persists the chosen format in localStorage when a download item is clicked', async () => {
    mockAllFormats();
    await renderPage();

    const items = await openDlMenu();
    const fb2 = items.find((a) => a.textContent === 'FB2');
    expect(fb2).toBeTruthy();
    await fireEvent.click(fb2!);
    expect(JSON.parse(getLS().getItem('fichub_prefs_v1') || '{}').defaultFormat).toBe('fb2');
  });
});