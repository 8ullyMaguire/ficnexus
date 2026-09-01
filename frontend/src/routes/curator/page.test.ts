import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
  goto: (path: string) => mockGoto(path),
}));

// Admin-level user (role 10 → level 100 under the F7 fallback) so the hub
// renders its content instead of redirecting to '/' (the redirect target
// itself is a no-op via the mock).
// The object identity is fixed at mock-definition time; the test for the
// non-admin case re-imports the page after swapping the module-level flags
// this mock closes over. (Names must start with `mock` so vitest hoists them
// above the vi.mock call.)
let mockLoggedIn = true;
let mockRole = 10;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockImplementation(async () => {
      mockLoggedIn = mockRole >= 10;
    }),
    get isLoggedIn() {
      return mockLoggedIn;
    },
    get user() {
      return mockRole >= 10 ? { id: 1, username: 'curator', role: mockRole, reputation: 0, level: 100 } : null;
    },
    get level() {
      return mockLoggedIn && mockRole >= 10 ? 100 : 0;
    },
  },
}));
function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function queuePayloads() {
  return [
    { err: 0, items: [{ id: 1 }] }, // author merges: 1 pending
    { err: 0, flags: [{ id: 7 }, { id: 8 }] }, // tag flags: 2
    { err: 0, proposals: [{ id: 1 }] }, // work proposals: 1
    { err: 0, items: [], total: 3 }, // upload moderation: 3
    { err: 0, items: [{ comment_id: 11 }] }, // comment triage: 1
  ];
}

beforeEach(() => {
  vi.clearAllMocks();
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  mockGoto.mockReset();
});

describe('curator hub page', () => {
  it('renders links to every queue', async () => {
    mockFetch.mockImplementation(() => Promise.resolve(jsonResponse({ err: 0, items: [] })));

    const { default: HubPage } = await import('./+page.svelte');
    render(HubPage);

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Curator Hub/ })).toBeTruthy();
    });
    expect(screen.getByText('Author Merges')).toBeTruthy();
    expect(screen.getByText('Tag Flags')).toBeTruthy();
    expect(screen.getByText('Work Proposals')).toBeTruthy();
    expect(screen.getByText('Upload Moderation')).toBeTruthy();
    expect(screen.getByText('Comment Triage')).toBeTruthy();

    const hrefs = screen
      .getAllByRole('link')
      .map((a) => a.getAttribute('href'));
    for (const href of [
      '/curator/authors',
      '/curator/flags',
      '/work-proposals',
      '/admin/moderation',
      '/admin/comment-triage',
    ]) {
      expect(hrefs).toContain(href);
    }
  });

  it('fetches each queue API with credentials and shows pending counts', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      const idx = [
        '/api/curator/authors/pending',
        '/api/curator/flags',
        '/api/work-proposals',
        '/api/admin/moderation/queue',
        '/api/admin/moderation/comments',
      ].findIndex((prefix) => url.startsWith(prefix));
      if (idx === -1) return Promise.reject(new Error(`unexpected fetch: ${url}`));
      return Promise.resolve(jsonResponse(queuePayloads()[idx]));
    });

    const { default: HubPage } = await import('./+page.svelte');
    render(HubPage);

    // Author merges: 1 pending → count '1' rendered in the archive <dd>.
    await waitFor(() => {
      // Archive UI: label in <dt>, count in sibling <dd>.
      const row = screen.getByText('Author Merges').closest('div.dl-row');
      expect(row?.textContent).toContain('1');
    });
    // Every fetch went out with credentials: 'include'.
    for (const call of mockFetch.mock.calls) {
      expect(call[1]?.credentials).toBe('include');
    }
  });

  it('shows per-card pending counts from the API totals', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.startsWith('/api/curator/authors/pending'))
        return Promise.resolve(jsonResponse({ err: 0, items: [{ id: 1 }, { id: 2 }] }));
      if (url.startsWith('/api/curator/flags'))
        return Promise.resolve(jsonResponse({ err: 0, flags: [] }));
      if (url.startsWith('/api/work-proposals'))
        return Promise.resolve(jsonResponse({ err: 0, proposals: [] }));
      if (url.startsWith('/api/admin/moderation/queue'))
        return Promise.resolve(jsonResponse({ err: 0, items: [], total: 5 }));
      if (url.startsWith('/api/admin/moderation/comments'))
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: HubPage } = await import('./+page.svelte');
    render(HubPage);

    // 2 pending merges + moderation total of 5, both rendered as counts.
    await waitFor(() => {
      const mergesRow = screen.getByText('Author Merges').closest('div.dl-row');
      expect(mergesRow?.textContent).toContain('2');
      const modRow = screen.getByText('Upload Moderation').closest('div.dl-row');
      expect(modRow?.textContent).toContain('5');
    });
  });

  it('does not block the hub when a count API fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.startsWith('/api/curator/authors/pending')) {
        return Promise.resolve({ ok: false, status: 500 });
      }
      return Promise.resolve(jsonResponse({ err: 0, items: [], flags: [], proposals: [], total: 0 }));
    });

    const { default: HubPage } = await import('./+page.svelte');
    render(HubPage);

    // Page still renders its queue cards and an 'n/a' badge for the failed source.
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Curator Hub/ })).toBeTruthy();
      expect(screen.getAllByText('n/a').length).toBeGreaterThan(0);
    });
  });

  it('redirects non-curators to the home page', async () => {
    mockRole = 3;
    mockLoggedIn = true;

    const { default: HubPage } = await import('./+page.svelte');
    render(HubPage);

    await waitFor(() => {
      expect(mockGoto).toHaveBeenCalledWith('/');
    });
  });
});
