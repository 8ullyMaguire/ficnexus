import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

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

function mePayload() {
  return {
    err: 0,
    user: { id: 1, username: 'tester', role: 0, reputation: 0, email: null, locale: 'en' },
  };
}

function categoriesPayload() {
  return {
    err: 0,
    items: [
      { id: 1, slug: 'general', title: 'General', description: '', position: 0, is_mod_only: false, created_at: '2026-08-01T00:00:00Z', topic_count: 2, last_activity_at: '2026-08-13T00:00:00Z' },
      { id: 2, slug: 'recs', title: 'Recommendations', description: '', position: 1, is_mod_only: false, created_at: '2026-08-01T00:00:00Z', topic_count: 1, last_activity_at: '2026-08-12T00:00:00Z' },
    ],
  };
}

function searchPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    q: 'dragon',
    results: [
      {
        type: 'topic',
        topic_id: 5,
        post_id: null,
        title: 'Dragon lore',
        author_id: 1,
        author_username: 'tester',
        body: 'All about dragons',
        snippet: 'All about <mark>dragon</mark>s',
        category_slug: 'general',
        category_title: 'General',
        created_at: '2026-08-13T10:00:00Z',
      },
      {
        type: 'post',
        topic_id: 7,
        post_id: 52,
        title: 'Reading thread',
        author_id: 2,
        author_username: 'mod',
        body: 'A reply',
        snippet: 'A <mark>dragon</mark> reply in a post',
        category_slug: 'recs',
        category_title: 'Recommendations',
        created_at: '2026-08-13T11:00:00Z',
      },
    ],
    next_cursor: null,
    limit: 25,
    total: 2,
    ...overrides,
  };
}

function router(input: RequestInfo | URL) {
  const url = String(input);
  if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
  if (url.includes('/api/forum/categories')) return Promise.resolve(jsonResponse(categoriesPayload()));
  if (url.includes('/api/forum/search')) return Promise.resolve(jsonResponse(searchPayload()));
  return Promise.reject(new Error(`unexpected: ${url}`));
}

describe('forum search page', () => {
  it('shows a hint and does not search when the query is empty', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => router(input));
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: {} as never } });

    await waitFor(() => {
      expect(screen.getByText('Type a query to search topics and posts.')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Search' }));
    // Button disabled on empty query → no search call fired.
    expect(mockFetch.mock.calls.some((c) => String(c[0]).includes('/api/forum/search'))).toBe(false);
    expect(screen.getByText('Type a query to search topics and posts.')).toBeTruthy();
  });

  it('searches and renders results with badges, meta and highlighted snippets', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => router(input));
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: {} as never } });

    const input = await screen.findByPlaceholderText('Search the forum…');
    await fireEvent.input(input, { target: { value: 'dragon' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Search' }));

    await waitFor(() => {
      expect(screen.getByText('Dragon lore')).toBeTruthy();
    });
    expect(screen.getByText('2 results for “dragon”')).toBeTruthy();
    // Type badges.
    expect(screen.getByText('Topic')).toBeTruthy();
    expect(screen.getByText('Post')).toBeTruthy();
    // Author + category meta.
    expect(screen.getByText(/by tester/)).toBeTruthy();
    expect(screen.getByText(/by mod/)).toBeTruthy();
    expect(screen.getByText('General')).toBeTruthy();
    // Snippet rendered as HTML with <mark> highlights.
    const marks = document.querySelectorAll('.archive-snippet mark');
    expect(marks.length).toBeGreaterThanOrEqual(2);
    expect(document.querySelector('.archive-snippet mark')?.textContent).toBe('dragon');
    // Topic link points at the topic detail route.
    expect(document.querySelector('a[href="/forum/general/5"]')).toBeTruthy();
  });

  it('renders the empty state when a search returns no results', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/categories')) return Promise.resolve(jsonResponse(categoriesPayload()));
      if (url.includes('/api/forum/search')) {
        return Promise.resolve(jsonResponse({ err: 0, q: 'zzz', results: [], next_cursor: null, limit: 25, total: 0 }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: {} as never } });

    const input = await screen.findByPlaceholderText('Search the forum…');
    await fireEvent.input(input, { target: { value: 'zzz' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Search' }));

    await waitFor(() => {
      expect(screen.getByText('No results for “zzz”.')).toBeTruthy();
    });
  });

  it('sends the selected category with the search query', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => router(input));
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: {} as never } });

    const select = (await screen.findByLabelText('Category')) as HTMLSelectElement;
    // Wait for the category options to load before changing the selection —
    // the select starts disabled with a single "All categories" option.
    await waitFor(() => {
      expect(select.options.length).toBeGreaterThan(1);
    });
    await fireEvent.change(select, { target: { value: 'general' } });
    const input = screen.getByPlaceholderText('Search the forum…');
    await fireEvent.input(input, { target: { value: 'dragon' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Search' }));

    await waitFor(() => {
      expect(screen.getByText('Dragon lore')).toBeTruthy();
    });
    const searchCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/forum/search'));
    expect(String(searchCall?.[0])).toBe('/api/forum/search?q=dragon&category=general');
  });

  it('shows an error state when the search fails', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) return Promise.resolve(jsonResponse(mePayload()));
      if (url.includes('/api/forum/categories')) return Promise.resolve(jsonResponse(categoriesPayload()));
      if (url.includes('/api/forum/search')) return Promise.resolve(jsonResponse({ err: 500, msg: 'search boom' }));
      return Promise.reject(new Error(`unexpected: ${url}`));
    });
    const { default: Page } = await import('./+page.svelte');
    render(Page, { props: { data: {} as never } });

    const input = await screen.findByPlaceholderText('Search the forum…');
    await fireEvent.input(input, { target: { value: 'dragon' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Search' }));

    await waitFor(() => {
      expect(screen.getByText(/search boom/)).toBeTruthy();
    });
  });
});
