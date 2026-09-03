import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
  goto: (path: string) => mockGoto(path),
}));

let mockLoggedIn = true;
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockImplementation(async () => {}),
    get isLoggedIn() {
      return mockLoggedIn;
    },
    get user() {
      return mockLoggedIn ? { id: 1, username: 'curator', role: 10, reputation: 0, level: 100 } : null;
    },
    get level() {
      return mockLoggedIn ? 100 : 0;
    },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function approvalsPayload() {
  return {
    err: 0,
    items: [
      {
        id: 10,
        type: 'collection_item_request',
        status: 'pending',
        created_at: '2026-08-25T00:00:00Z',
        proposer: 'alice',
        votes_for: 3,
        votes_against: 1,
        my_vote: 0,
        collection_id: 5,
        collection_title: 'My Coll',
        work_title: 'Wonder',
        work_author: 'Alice A',
        blurb: '',
      },
      {
        id: 20,
        type: 'curator_fix_proposal',
        status: 'pending',
        created_at: '2026-08-25T00:00:00Z',
        proposer: 'bob',
        votes_for: 1,
        votes_against: 0,
        my_vote: 1,
        url_id: 'ffn/123',
        reason: 'wrong scraped body',
      },
    ],
    pending_counts: {
      collection_item_request: 1,
      curator_fix_proposal: 1,
      metadata_proposal: 0,
      comment_triage: 0,
      forum_edit_proposal: 0,
    },
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockFetch.mockReset();
  mockGoto.mockReset();
  mockLoggedIn = true;
  localStorage.clear();
  localStorage.setItem('fichub_prefs_v1', JSON.stringify({ uiMode: 'archive' }));
});

describe('curator approvals page', () => {
  it('loads and groups approvals by type with vote tallies', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.startsWith('/api/curator/approvals')) {
        return Promise.resolve(jsonResponse(approvalsPayload()));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ApprovalsPage } = await import('./+page.svelte');
    render(ApprovalsPage);

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Approvals/ })).toBeTruthy();
    });

    // Wait for the async load to render group headings (h2 — distinct from the
    // <select> option labels which share the same translated string).
    await waitFor(() => {
      expect(screen.getByRole('heading', { level: 2, name: /Collection additions/ })).toBeTruthy();
    });
    expect(screen.getByRole('heading', { level: 2, name: /Content fixes/ })).toBeTruthy();

    // Item summaries present (function matcher handles em-dash text nodes).
    expect(screen.getByText((c) => c.includes('Wonder'))).toBeTruthy();
    expect(screen.getByText((c) => c.includes('ffn/123'))).toBeTruthy();

    // Vote tallies rendered for the collection request.
    expect(screen.getByText((c) => c.includes('Agree 3'))).toBeTruthy();
    expect(screen.getByText((c) => c.includes('Disagree 1'))).toBeTruthy();
  });

  it('calls the collection approve endpoint then reloads', async () => {
    let calls = 0;
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.startsWith('/api/collections/5/requests/10/approve') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, msg: 'approved' }));
      }
      if (url.startsWith('/api/curator/approvals')) {
        calls += 1;
        // First load lists the item; after approve it is gone (approved filter default is pending).
        if (calls > 1) {
          return Promise.resolve(jsonResponse({ err: 0, items: [], pending_counts: approvalsPayload().pending_counts }));
        }
        return Promise.resolve(jsonResponse(approvalsPayload()));
      }
      return Promise.reject(new Error(`unexpected fetch: ${url}`));
    });

    const { default: ApprovalsPage } = await import('./+page.svelte');
    render(ApprovalsPage);

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Approvals/ })).toBeTruthy();
    });

    await waitFor(() => {
      expect(screen.getAllByRole('button', { name: 'Approve' }).length).toBeGreaterThan(0);
    });
    const approveBtn = screen.getAllByRole('button', { name: 'Approve' })[0];
    fireEvent.click(approveBtn);

    await waitFor(() => {
      const postCalls = mockFetch.mock.calls.filter(
        (c) => String(c[0]).startsWith('/api/collections/5/requests/10/approve'),
      );
      expect(postCalls.length).toBeGreaterThan(0);
    });
  });

  it('redirects to /login when not authenticated', async () => {
    mockLoggedIn = false;
    const { default: ApprovalsPage } = await import('./+page.svelte');
    render(ApprovalsPage);
    await waitFor(() => {
      expect(mockGoto).toHaveBeenCalledWith('/login');
    });
  });
});
