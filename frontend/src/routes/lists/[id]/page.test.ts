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

// Mock SvelteKit modules: the detail page reads $page().params.id.
vi.mock('$app/stores', () => ({
  page: {
    subscribe: (fn: Function) => {
      fn({ url: new URL('http://localhost/lists/1'), params: { id: '1' } });
      return () => {};
    },
  },
}));

vi.mock('$app/navigation', () => ({
  goto: vi.fn(),
}));

describe('lists detail page', () => {
  it('renders the list with its ordered items and blurbs', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          list: { id: 1, user_id: 1, title: 'Best Time Travel', description: 'Curated', is_public: true, created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z', item_count: 2 },
          items: [
            { id: 10, work_id: 100, position: 1, blurb: 'Classic first', title: 'Steins Gate', author: 'Author A' },
            { id: 11, work_id: 101, position: 2, blurb: '', title: 'Erased', author: 'Author B' },
          ],
          is_owner: true,
        }),
      });

    const { default: DetailPage } = await import('./+page.svelte');
    render(DetailPage, { props: { data: { id: '1' } } });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Best Time Travel' })).toBeTruthy();
    });
    expect(screen.getByText('Steins Gate')).toBeTruthy();
    expect(screen.getByText('Erased')).toBeTruthy();
    expect(screen.getByText(/Classic first/)).toBeTruthy();
    expect(screen.getByText(/by Author A/)).toBeTruthy();
    // Ordered position markers.
    expect(screen.getAllByText(/1\./).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/2\./).length).toBeGreaterThan(0);
    // Owner sees the add form + remove buttons.
    expect(screen.getByRole('button', { name: /\+ add work/i })).toBeTruthy();
  });

  it('shows a private/not-found message when the list is inaccessible', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: -5, msg: 'Reading list not found' }) });

    const { default: DetailPage } = await import('./+page.svelte');
    render(DetailPage, { props: { data: { id: '1' } } });

    await waitFor(() => {
      expect(screen.getByText(/private or does not exist/i)).toBeTruthy();
    });
  });

  it('adds a work to the list via the form', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          list: { id: 1, user_id: 1, title: 'My List', description: '', is_public: false, created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z', item_count: 0 },
          items: [],
          is_owner: true,
        }),
      })
      // Add-item POST returns position, then the page reloads the detail:
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, msg: 'Work added to list', position: 1 }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          list: { id: 1, user_id: 1, title: 'My List', description: '', is_public: false, created_at: '2026-01-01T00:00:00Z', updated_at: '2026-01-01T00:00:00Z', item_count: 1 },
          items: [{ id: 10, work_id: 500, position: 1, blurb: 'nice', title: 'Added Work', author: 'Author Z' }],
          is_owner: true,
        }),
      });

    const { default: DetailPage } = await import('./+page.svelte');
    render(DetailPage, { props: { data: { id: '1' } } });

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /\+ add work/i })).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: /\+ add work/i }));
    await fireEvent.input(screen.getByPlaceholderText('Work id'), { target: { value: '500' } });
    await fireEvent.input(screen.getByPlaceholderText(/one-line blurb/i), { target: { value: 'nice' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Add to list' }));

    await waitFor(() => {
      expect(screen.getByText('Added Work')).toBeTruthy();
    });
  });
});
