import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function proposalsPayload() {
  return jsonResponse({
    err: 0,
    proposals: [
      {
        id: 1,
        proposer_id: 10,
        action_type: 'merge',
        source_work_id: 101,
        target_work_id: 202,
        work_id: null,
        details: null,
        status: 'pending',
        created_at: '2026-08-01T00:00:00Z',
        closed_at: null,
        vote_sum: 0,
        voter_count: 0,
      },
      {
        id: 2,
        proposer_id: 11,
        action_type: 'split',
        source_work_id: null,
        target_work_id: null,
        work_id: 303,
        details: { chapters: [1, 2] },
        status: 'pending',
        created_at: '2026-08-02T00:00:00Z',
        closed_at: null,
        vote_sum: 2,
        voter_count: 1,
      },
    ],
  });
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  localStorage.clear();
  // Reset the auth singleton so auth.init() re-fetches /api/auth/me and the
  // role-derived canVote flag reflects the user each test seeds.
  void import('$lib/stores/auth.svelte').then(({ auth }) => {
    auth.initialized = false;
    auth.user = null;
  });
});

async function loadPage() {
  return await import('./+page.svelte');
}

async function mountPage(role: number) {
  mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
    const url = String(input);
    if (url.includes('/auth/me')) {
      return Promise.resolve(
        jsonResponse({ err: 0, user: { id: 1, username: 'curator', role, reputation: 0 } }),
      );
    }
    if (url.includes('/api/work-proposals') && init?.method === 'POST') {
      return Promise.resolve(jsonResponse({ err: 0, msg: 'Vote recorded', vote_sum: 4, voter_count: 3 }));
    }
    if (url.includes('/api/work-proposals')) {
      return Promise.resolve(proposalsPayload());
    }
    return Promise.reject(new Error(`unexpected fetch: ${url}`));
  });
  const { default: Page } = await loadPage();
  render(Page);
}

describe('work proposals page', () => {
  it('renders pending proposals with type, status, and vote chips', async () => {
    await mountPage(10);

    await waitFor(() => {
      expect(screen.getByText('Merge work #101 into work #202')).toBeTruthy();
    });
    expect(screen.getByText('Split work #303')).toBeTruthy();
    expect(screen.getByText('merge')).toBeTruthy();
    expect(screen.getByText('split')).toBeTruthy();
    expect(screen.getByText('+0 / 0 voters')).toBeTruthy();
    expect(screen.getByText('+2 / 1 voter')).toBeTruthy();
  });

  it('shows the vote buttons for curator+ users', async () => {
    await mountPage(5);

    await waitFor(() => {
      expect(screen.getByText('Merge work #101 into work #202')).toBeTruthy();
    });
    expect(screen.getByRole('button', { name: /Approve proposal 1/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Abstain on proposal 1/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Reject proposal 1/ })).toBeTruthy();
  });

  it('hides vote buttons and shows a notice for non-curators', async () => {
    await mountPage(0);

    await waitFor(() => {
      expect(screen.getByText('Merge work #101 into work #202')).toBeTruthy();
    });
    expect(screen.queryByRole('button', { name: /Approve proposal 1/ })).toBeNull();
    expect(screen.getByText(/Only curators can vote/)).toBeTruthy();
  });

  it('POSTs a +1 vote and updates the displayed vote sum', async () => {
    await mountPage(10);

    await waitFor(() => {
      expect(screen.getByText('Merge work #101 into work #202')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /Approve proposal 1/ }));

    await waitFor(() => {
      const voteCall = mockFetch.mock.calls.find(
        (c) => String(c[0]).includes('/api/work-proposals/1/vote') && c[1]?.method === 'POST',
      );
      expect(voteCall).toBeTruthy();
      expect(JSON.parse(String(voteCall![1]!.body)).vote).toBe(1);
    });
    // The server response carries the fresh aggregate — the page reflects it.
    await waitFor(() => {
      expect(screen.getByText('+4 / 3 voters')).toBeTruthy();
    });
  });
});
