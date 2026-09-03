import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

// SvelteKit's $app/state page store triggers notifiable_store which is
// unavailable in jsdom. Mock it with a minimal reactive store.
const mockPage = { url: new URL('http://localhost'), params: {} };
vi.mock('$app/state', () => ({
  page: mockPage,
}));

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

import { auth } from '$lib/stores/auth.svelte';

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

function mePayload(role = 0) {
  return {
    err: 0,
    user: { id: 1, username: 'tester', role, reputation: 0, email: null, locale: 'en' },
  };
}

function topicsPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    items: [
      { id: 11, title: 'First topic', author_id: 1, author_username: 'tester', reply_count: 2, vote_score: 5, view_count: 120, last_post_id: 30, status: 'open', unread: true, last_activity_at: '2026-08-13T10:00:00Z', created_at: '2026-08-10T00:00:00Z' },
      { id: 12, title: 'Pinned announcement', author_id: 2, author_username: 'mod', reply_count: 0, vote_score: 0, view_count: 400, last_post_id: null, status: 'pinned', unread: false, last_activity_at: '2026-08-12T10:00:00Z', created_at: '2026-08-09T00:00:00Z' },
    ],
    next_cursor: null,
    category: 'general',
    limit: 25,
    ...overrides,
  };
}

describe('forum topic list page', () => {
  it('renders topics from the API with author, replies, score and views', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/topics')) {
        return Promise.resolve(jsonResponse(topicsPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general' } } });

    await waitFor(() => {
      expect(screen.getByText('First topic')).toBeTruthy();
    });
    expect(screen.getByText('Pinned announcement')).toBeTruthy();
    // Archive UI: topic table with Author / Replies / Views columns.
    expect(screen.getByText('tester')).toBeTruthy(); // author column
    expect(screen.getByText('2')).toBeTruthy(); // replies
    expect(screen.getByText('120')).toBeTruthy(); // views
    expect(screen.getByText('pin')).toBeTruthy(); // pinned chip (lowercase)
    // F4 unread indicator: visible for the unread topic, hidden otherwise.
    expect(screen.getByText('new')).toBeTruthy(); // unread chip (lowercase)
    expect(document.querySelectorAll('.arc-chip-unread')).toHaveLength(1);
    expect(document.querySelectorAll('.archive-topic-table tr')).toHaveLength(3); // header + 2 rows
  });

  it('hides the unread indicator when every topic is read', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/topics')) {
        return Promise.resolve(jsonResponse(topicsPayload({ items: [
          { id: 11, title: 'First topic', author_id: 1, author_username: 'tester', reply_count: 2, vote_score: 5, view_count: 120, last_post_id: 30, status: 'open', unread: false, last_activity_at: '2026-08-13T10:00:00Z', created_at: '2026-08-10T00:00:00Z' },
          { id: 12, title: 'Pinned announcement', author_id: 2, author_username: 'mod', reply_count: 0, vote_score: 0, view_count: 400, last_post_id: null, status: 'pinned', last_activity_at: '2026-08-12T10:00:00Z', created_at: '2026-08-09T00:00:00Z' },
        ] })));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general' } } });

    await waitFor(() => {
      expect(screen.getByText('First topic')).toBeTruthy();
    });
    expect(screen.queryByText('new')).toBeNull();
    expect(document.querySelectorAll('.arc-chip-unread')).toHaveLength(0);
  });

  it('shows the empty state when a category has no topics', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/topics')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [], next_cursor: null, category: 'general', limit: 25 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general' } } });

    await waitFor(() => {
      expect(screen.getByText('No topics yet.')).toBeTruthy();
    });
  });

  it('loads the next page using the cursor', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/topics')) {
        if (url.includes('cursor=11')) {
          return Promise.resolve(
            jsonResponse({ err: 0, items: [{ id: 9, title: 'Older topic', author_id: 1, author_username: 'tester', reply_count: 0, vote_score: 0, view_count: 5, last_post_id: null, status: 'open', last_activity_at: '2026-08-05T00:00:00Z', created_at: '2026-08-05T00:00:00Z' }], next_cursor: null, category: 'general', limit: 25 }),
          );
        }
        return Promise.resolve(jsonResponse(topicsPayload({ next_cursor: 11 })));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general' } } });

    await waitFor(() => {
      expect(screen.getByText('First topic')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /Next page/ }));

    await waitFor(() => {
      expect(screen.getByText('Older topic')).toBeTruthy();
    });
    const cursorCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('cursor=11'));
    expect(cursorCall).toBeTruthy();
  });

  it('renders locked badge for locked topics', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/topics')) return Promise.resolve(jsonResponse({ err: 0, items: [{ id: 13, title: 'Locked topic', author_id: 1, author_username: 'tester', reply_count: 1, vote_score: 0, view_count: 10, last_post_id: 31, status: 'locked', last_activity_at: '2026-08-13T10:00:00Z', created_at: '2026-08-10T00:00:00Z' }], next_cursor: null, category: 'general', limit: 25 }));
      return Promise.reject(new Error(url));
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: { categorySlug: 'general' } } });
    await waitFor(() => { expect(screen.getByText('locked')).toBeTruthy(); });
  });

});
