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

describe('shelves page', () => {
  it('renders the reading-status shelves grouped by status', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      // listShelves
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, shelves: [{ id: 1, name: 'Faves', description: '', is_public: false, sort_order: 0, created_at: '2026-01-01T00:00:00Z' }] }) })
      // getReadingList
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          reading_list: [
            { id: 1, work_id: 100, words_read: 0, read_count: 0, status: 'want_to_read', current_chapter: null, last_read_at: '2026-01-01T00:00:00Z' },
            { id: 2, work_id: 101, words_read: 5000, read_count: 1, status: 'reading', current_chapter: 3, last_read_at: '2026-01-02T00:00:00Z' },
          ],
        }),
      })
      // getWork for each item
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, work: { id: 100, canonical_title: 'Want Fic', canonical_author: 'Author W', description: '', default_source_id: null, created_at: '', updated_at: '', sources: [], total_bookmarks: 0, total_ratings: 0, total_comments: 0 } }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, work: { id: 101, canonical_title: 'Reading Fic', canonical_author: 'Author R', description: '', default_source_id: null, created_at: '', updated_at: '', sources: [], total_bookmarks: 0, total_ratings: 0, total_comments: 0 } }) });

    const { default: ShelvesPage } = await import('./+page.svelte');
    render(ShelvesPage);

    // Custom shelves section still renders (from the existing API).
    await waitFor(() => {
      expect(screen.getByText('Faves')).toBeTruthy();
    });
    // Reading-status shelves: both works appear in their status columns.
    await waitFor(() => {
      expect(screen.getByText('Want Fic')).toBeTruthy();
    });
    expect(screen.getByText('Reading Fic')).toBeTruthy();
    // Section headings.
    expect(screen.getByText('Reading Status')).toBeTruthy();
    expect(screen.getByText('Custom Shelves')).toBeTruthy();
  });

  it('moves a work between status shelves via the chip buttons', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, shelves: [] }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          reading_list: [{ id: 1, work_id: 100, words_read: 0, read_count: 0, status: 'want_to_read', current_chapter: null, last_read_at: '2026-01-01T00:00:00Z' }],
        }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, work: { id: 100, canonical_title: 'Want Fic', canonical_author: 'Author W', description: '', default_source_id: null, created_at: '', updated_at: '', sources: [], total_bookmarks: 0, total_ratings: 0, total_comments: 0 } }) })
      // updateReadingStatus POST
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, msg: 'Reading status updated' }) })
      // reload after status change: now in 'reading'
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, reading_list: [{ id: 1, work_id: 100, words_read: 0, read_count: 0, status: 'reading', current_chapter: null, last_read_at: '2026-01-01T00:00:00Z' }] }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, work: { id: 100, canonical_title: 'Want Fic', canonical_author: 'Author W', description: '', default_source_id: null, created_at: '', updated_at: '', sources: [], total_bookmarks: 0, total_ratings: 0, total_comments: 0 } }) });

    const { default: ShelvesPage } = await import('./+page.svelte');
    render(ShelvesPage);

    await waitFor(() => {
      expect(screen.getByText('Want Fic')).toBeTruthy();
    });

    // Click the "reading" chip on the work row (there is one row, one set of chips).
    const chips = screen.getAllByRole('button', { name: '📖' });
    await fireEvent.click(chips[0]);

    await waitFor(() => {
      // The POST to /api/reading/status carried status=reading.
      const postCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/reading/status') && c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
      expect(JSON.parse(postCall![1].body).status).toBe('reading');
    });
  });
});
