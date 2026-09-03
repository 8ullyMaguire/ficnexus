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

function mePayload(role = 0) {
  return {
    err: 0,
    user: { id: 1, username: 'tester', role, reputation: 0, email: null, locale: 'en' },
  };
}

function categoriesPayload(overrides: Record<string, unknown> = {}) {
  return {
    err: 0,
    items: [
      { id: 1, slug: 'general', title: 'General Discussion', description: 'Anything goes', position: 0, is_mod_only: false, created_at: '2026-08-01T00:00:00Z', topic_count: 3, last_activity_at: '2026-08-13T10:00:00Z' },
      { id: 2, slug: 'fic-rec', title: 'Fic Recommendations', description: '', position: 1, is_mod_only: false, created_at: '2026-08-01T00:00:00Z', topic_count: 0, last_activity_at: null },
    ],
    ...overrides,
  };
}

describe('forum category list page', () => {
  it('renders categories from the API with topic counts', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/categories')) {
        return Promise.resolve(jsonResponse(categoriesPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('General Discussion')).toBeTruthy();
    });
    expect(screen.getByText('Fic Recommendations')).toBeTruthy();
    // Archive UI: category table shows topic_count in a Topics column.
    expect(screen.getByText('3')).toBeTruthy(); // General Discussion topic_count
    expect(screen.getByText(/Anything goes/)).toBeTruthy();
  });

  it('shows the empty state when there are no categories', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload()));
      }
      if (url.includes('/api/forum/categories')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('No categories yet.')).toBeTruthy();
    });
  });

  it('shows the new-category form for admins and creates a category', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload(10)));
      }
      if (url.includes('/api/forum/categories') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, id: 99 }));
      }
      if (url.includes('/api/forum/categories')) {
        // After creation the page reloads the list.
        return Promise.resolve(jsonResponse(categoriesPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('New Category')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: 'New Category' }));
    await fireEvent.input(screen.getByPlaceholderText(/e.g. general-discussion/), { target: { value: 'New-Cat' } });
    await fireEvent.input(screen.getByPlaceholderText(/e.g. General Discussion/), { target: { value: 'New Category' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Create category' }));

    await waitFor(() => {
      const call = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/forum/categories') && c[1]?.method === 'POST');
      expect(call).toBeTruthy();
      const body = JSON.parse(String(call![1]!.body));
      expect(body.slug).toBe('new-cat');
      expect(body.title).toBe('New Category');
    });
  });

  it('does not show the create button for non-admins', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/auth/me')) {
        return Promise.resolve(jsonResponse(mePayload(0)));
      }
      if (url.includes('/api/forum/categories')) {
        return Promise.resolve(jsonResponse(categoriesPayload()));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('General Discussion')).toBeTruthy();
    });
    expect(screen.queryByRole('button', { name: 'New Category' })).toBeNull();
  });
});
