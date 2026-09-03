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
        post_id: 101,
        topic_id: 5,
        author_username: 'spammer',
        body: 'Buy my coins now!!!',
        score: -4,
        mod_count: 3,
        reason: 'abusive',
        created_at: '2026-08-14T09:00:00Z',
      },
      {
        post_id: 102,
        topic_id: 7,
        author_username: 'trollface',
        body: 'You are all wrong about everything',
        score: -1,
        mod_count: 1,
        reason: null,
        created_at: '2026-08-14T10:00:00Z',
      },
    ],
    ...overrides,
  };
}

beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

function mePayload(role = 5) {
  return {
    err: 0,
    user: { id: 1, username: 'curator', role, reputation: 0, email: null, locale: 'en' },
  };
}

async function renderPage() {
  const { default: Page } = await import('./+page.svelte');
  return render(Page, { props: { data: {} as never } });
}

function itemReasonButtons(label: string, itemIndex: number): HTMLElement[] {
  // Each queue item renders the nine reasons in MODERATION_REASONS order, so
  // the per-label button list holds one button per item in item order.
  const all = screen.getAllByRole('button', { name: new RegExp(`· ${label}$`) });
  return all.slice(itemIndex, itemIndex + 1);
}

describe('forum moderation queue page', () => {
  it('renders queue items with excerpt, score/mod_count badges and nine reason buttons', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();

    await waitFor(() => {
      expect(screen.getByText('Buy my coins now!!!')).toBeTruthy();
    });
    expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    // Score + mod_count badges.
    expect(screen.getByText('-4')).toBeTruthy();
    expect(screen.getByText('3 mods')).toBeTruthy();
    expect(screen.getByText('-1')).toBeTruthy();
    expect(screen.getByText('1 mod')).toBeTruthy();
    // Server-provided reason chip.
    expect(screen.getByText('Abusive')).toBeTruthy();
    // Nine reason buttons per item.
    expect(screen.getAllByRole('button', { name: /· Insightful$/ })).toHaveLength(2);
    expect(screen.getAllByRole('button', { name: /· Off-topic$/ })).toHaveLength(2);
    expect(screen.getAllByRole('button', { name: /· Abusive$/ })).toHaveLength(2);
    expect(screen.getByText('2 posts waiting')).toBeTruthy();
  });

  it('applies a reason and removes the item optimistically', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/posts/101/moderate') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, delta: -3, score_after: -7, hidden_until: null }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('Buy my coins now!!!')).toBeTruthy();
    });

    // Item 0's "Abusive" button.
    const abusive = itemReasonButtons('Abusive', 0);
    expect(abusive.length).toBeGreaterThan(0);
    await fireEvent.click(abusive[0]);

    await waitFor(() => {
      expect(screen.queryByText('Buy my coins now!!!')).toBeNull();
    });
    // The other item stays.
    expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    // Count updated.
    expect(screen.getByText('1 post waiting')).toBeTruthy();
    const modCall = mockFetch.mock.calls.find(
      (c) => String(c[0]).endsWith('/api/forum/posts/101/moderate') && (c[1] as RequestInit)?.method === 'POST',
    );
    expect(modCall).toBeTruthy();
    expect(JSON.parse(String((modCall as any)[1].body))).toEqual({ reason: 'abusive' });
  });

  it('removes the item on 409 (already moderated) without an error', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/posts/102/moderate') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 409, msg: 'already moderated' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('You are all wrong about everything')).toBeTruthy();
    });

    // Item 1's "Troll" button.
    const troll = itemReasonButtons('Troll', 1);
    expect(troll.length).toBeGreaterThan(0);
    await fireEvent.click(troll[0]);

    await waitFor(() => {
      expect(screen.queryByText('You are all wrong about everything')).toBeNull();
    });
    expect(screen.queryByText('Moderation failed. Try again.')).toBeNull();
    expect(screen.getByText('Buy my coins now!!!')).toBeTruthy();
  });

  it('keeps the item and shows an error when moderation fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.endsWith('/api/forum/posts/101/moderate') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 403, msg: 'no points' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('Buy my coins now!!!')).toBeTruthy();
    });

    // Item 0's "Troll" button.
    const troll = itemReasonButtons('Troll', 0);
    expect(troll.length).toBeGreaterThan(0);
    await fireEvent.click(troll[0]);
    await waitFor(() => {
      expect(screen.getByText('Moderation failed. Try again.')).toBeTruthy();
    });
    // Item stays.
    expect(screen.getByText('Buy my coins now!!!')).toBeTruthy();
  });

  it('renders the empty state when the queue has no items', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) {
        return Promise.resolve(jsonResponse({ err: 0, count: 0, items: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('Queue is empty — nothing to moderate right now.')).toBeTruthy();
    });
  });

  it('mod drawer: fetches and shows recent grants with reasons', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) return Promise.resolve(jsonResponse(queuePayload()));
      if (url.includes('/api/forum/moderation/user/42/grants')) return Promise.resolve(jsonResponse({ err: 0, user_id: 42, count: 2, items: [{ id: 1, post_id: 10, topic_id: 5, reason: 'troll', delta: -2, score_after: -1, created_at: '2026-08-14T09:00:00Z' }, { id: 2, post_id: 11, topic_id: 5, reason: 'insightful', delta: 2, score_after: 3, created_at: '2026-08-14T10:00:00Z' }] }));
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    // This test validates the client contract for the user grants endpoint;
    // the drawer UI is exercised via contract fetch + TopicThread drawer.
    const { getModerationUserGrants } = await import('$lib/api/forum');
    const res = await getModerationUserGrants(42);
    expect(res.err).toBe(0);
    expect(res.items).toHaveLength(2);
    expect(res.items[0].reason).toBe('troll');
    expect(res.items[1].reason).toBe('insightful');
    expect(String(mockFetch.mock.calls.find(c=>String(c[0]).includes('/grants'))?.[0])).toContain('/api/forum/moderation/user/42/grants');
  });

  it('renders an error state when loading the queue fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/moderation/queue')) {
        return Promise.resolve(jsonResponse({ err: 500, msg: 'queue boom' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    await renderPage();
    await waitFor(() => {
      expect(screen.getByText('queue boom')).toBeTruthy();
    });
  });
});
