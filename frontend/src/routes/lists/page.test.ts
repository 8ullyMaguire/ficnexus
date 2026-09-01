import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// Reset the auth store singleton between tests so auth.init() actually
// re-fetches /api/auth/me per test.
import { auth } from '$lib/stores/auth.svelte';
beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('lists page', () => {
  it('renders the user\'s reading lists from the API', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // auth.init()/me
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          lists: [
            { id: 1, user_id: 1, title: 'Best Time Travel', description: 'Curated picks', is_public: true, created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z', item_count: 3 },
            { id: 2, user_id: 1, title: 'Private Stash', description: '', is_public: false, created_at: '2026-01-02T00:00:00Z', updated_at: '2026-01-02T00:00:00Z', item_count: 0 },
          ],
        }),
      });

    const { default: ListsPage } = await import('./+page.svelte');
    render(ListsPage);

    await waitFor(() => {
      expect(screen.getByText('Best Time Travel')).toBeTruthy();
    });
    expect(screen.getByText('Private Stash')).toBeTruthy();
    expect(screen.getByText(/3 works/)).toBeTruthy();
    expect(screen.getByText(/Public · 3 works/)).toBeTruthy();
    expect(screen.getByText(/Private · 0 works/)).toBeTruthy();
    // Each list links to its detail page.
    const links = screen.getAllByRole('link');
    expect(links.some((a) => a.getAttribute('href') === '/lists/1')).toBe(true);
  });

  it('shows the empty state when the user has no lists', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, lists: [] }) });

    const { default: ListsPage } = await import('./+page.svelte');
    render(ListsPage);

    await waitFor(() => {
      expect(screen.getByText(/haven't created any reading lists yet/i)).toBeTruthy();
    });
  });

  it('creates a new list via the form', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, lists: [] }) }) // initial load
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          list: { id: 9, user_id: 1, title: 'My New List', description: '', is_public: false, created_at: '2026-01-03T00:00:00Z', updated_at: '2026-01-03T00:00:00Z', item_count: 0 },
        }),
      });

    const { default: ListsPage } = await import('./+page.svelte');
    render(ListsPage);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /\+ new list/i })).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /\+ new list/i }));
    await fireEvent.input(screen.getByPlaceholderText('List title'), { target: { value: 'My New List' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() => {
      expect(screen.getByText('My New List')).toBeTruthy();
    });
    // The POST to /api/lists carried the title.
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(postCall![1].body).title).toBe('My New List');
  });
});
