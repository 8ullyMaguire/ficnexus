import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

function queuePayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    count: 2,
    items: [
      {
        action_id: 11,
        post_id: 101,
        topic_id: 5,
        excerpt: 'This is a well-reasoned correction of the earlier claim',
        reason: 'insightful',
        delta: 2,
        score_after: 4,
        created_at: '2026-08-14T09:00:00Z',
      },
      {
        action_id: 12,
        post_id: 102,
        topic_id: 7,
        excerpt: 'You are all wrong about everything',
        reason: 'troll',
        delta: -2,
        score_after: -2,
        created_at: '2026-08-14T10:00:00Z',
      },
    ],
    ...overrides,
  };
}

function mePayload(role = 1) {
  return {
    err: 0,
    user: { id: 1, username: 'veteran', role, reputation: 0, email: null, locale: 'en' },
  };
}

beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

async function renderPage() {
  const { default: Page } = await import('./+page.svelte');
  return render(Page, { props: { data: {} as never } });
}

function itemVerdictButtons(label: string, itemIndex: number): HTMLElement[] {
  const all = screen.getAllByRole('button', { name: label });
  return all.slice(itemIndex, itemIndex + 1);
}

describe('forum metamod page', () => {
  it('renders audit cards with excerpt, reason chip, delta and fair/unfair/unsure buttons', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();

    await waitFor(() => {
      expect(screen.getByText('This is a well-reasoned correction of the earlier claim')).toBeTruthy();
    });
    expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    // Reason chip + delta badges.
    expect(screen.getByText('Insightful')).toBeTruthy();
    expect(screen.getByText('Troll')).toBeTruthy();
    expect(screen.getByText('+2')).toBeTruthy();
    expect(screen.getByText('-2')).toBeTruthy();
    // No moderator identity leaks onto the cards.
    expect(screen.queryByText(/moderator|curator/i)).toBeNull();
    // Three verdict buttons per card, labelled.
    expect(screen.getAllByRole('button', { name: 'Fair' })).toHaveLength(2);
    expect(screen.getAllByRole('button', { name: 'Unfair' })).toHaveLength(2);
    expect(screen.getAllByRole('button', { name: 'Unsure' })).toHaveLength(2);
    expect(screen.getByText('2 actions to audit')).toBeTruthy();
  });

  it('votes and removes the card optimistically', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/metamod/11/vote') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('This is a well-reasoned correction of the earlier claim')).toBeTruthy();
    });

    // Item 0's "Unfair" button.
    const unfair = itemVerdictButtons('Unfair', 0);
    expect(unfair.length).toBeGreaterThan(0);
    await fireEvent.click(unfair[0]);

    await waitFor(() => {
      expect(screen.queryByText('This is a well-reasoned correction of the earlier claim')).toBeNull();
    });
    // The other card stays.
    expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    // Count updated.
    expect(screen.getByText('1 action to audit')).toBeTruthy();
    const voteCall = mockFetch.mock.calls.find(
      (c) => String(c[0]).endsWith('/api/forum/metamod/11/vote') && (c[1] as RequestInit)?.method === 'POST',
    );
    expect(voteCall).toBeTruthy();
    expect(JSON.parse(String((voteCall as any)[1].body))).toEqual({ verdict: 'unfair' });
  });

  it('removes the card on 409 (already voted) without an error', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/metamod/12/vote') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 409, msg: 'already voted' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    });

    const unsure = itemVerdictButtons('Unsure', 1);
    expect(unsure.length).toBeGreaterThan(0);
    await fireEvent.click(unsure[0]);

    await waitFor(() => {
      expect(screen.queryByText('You are all wrong about everything')).toBeNull();
    });
    expect(screen.queryByText('Vote failed. Try again.')).toBeNull();
    expect(screen.getByText('This is a well-reasoned correction of the earlier claim')).toBeTruthy();
  });

  it('keeps the card and shows an error when the vote fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/metamod/11/vote') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 403, msg: 'not eligible' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('This is a well-reasoned correction of the earlier claim')).toBeTruthy();
    });

    const fair = itemVerdictButtons('Fair', 0);
    expect(fair.length).toBeGreaterThan(0);
    await fireEvent.click(fair[0]);
    await waitFor(() => {
      expect(screen.getByText(/Vote failed/)).toBeTruthy();
    });
    // Card stays.
    expect(screen.getByText('This is a well-reasoned correction of the earlier claim')).toBeTruthy();
  });

  it('renders the empty state when there is nothing to audit', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) {
        return Promise.resolve(jsonResponse({ err: 0, count: 0, items: [], pool_too_small: false }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('Nothing to audit right now.')).toBeTruthy();
    });
    // No dormant note when the pool is healthy.
    expect(screen.queryByText(/dormant/i)).toBeNull();
  });

  it('renders the empty state with a dormancy note when pool_too_small', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) {
        return Promise.resolve(jsonResponse({ err: 0, count: 0, items: [], pool_too_small: true }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('Nothing to audit right now.')).toBeTruthy();
    });
    expect(
      screen.getByText(/Metamoderation is dormant while the eligible pool is small/i),
    ).toBeTruthy();
  });

  it('renders an error state when loading the queue fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/metamod/queue')) {
        return Promise.resolve(jsonResponse({ err: 500, msg: 'metamod boom' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText(/metamod boom/)).toBeTruthy();
    });
  });
});
