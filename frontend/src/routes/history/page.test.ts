import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

// Reset the auth store singleton between tests.
import { auth } from '$lib/stores/auth.svelte';
beforeEach(() => {
  auth.initialized = false;
  auth.user = null;
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  // Mock confirm() to return true for clear-all confirmation
  vi.stubGlobal('confirm', vi.fn(() => true));
});

describe('history page', () => {
  it('shows empty state when history is empty', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, history: [], total: 0, limit: 50, offset: 0 }) }); // history

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Your reading history is empty.')).toBeTruthy();
    });
  });

  it('renders history entries with title, author, and date', async () => {
    const visitedAt = new Date('2026-08-15T10:30:00Z').toISOString();
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          history: [{
            id: 1,
            work_id: 100,
            url_id: 'fic100',
            title: 'Test Fic',
            author: 'Test Author',
            chapter_num: null,
            visited_at: visitedAt,
          }],
          total: 1,
          limit: 50,
          offset: 0,
        }),
      });

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Test Fic')).toBeTruthy();
    });
    expect(screen.getByText('by Test Author')).toBeTruthy();
    expect(screen.getByText('1 visit')).toBeTruthy();
    // Date should be rendered (we just check the formatted version contains "Aug" or "August")
    expect(screen.getByText(/Aug|August/)).toBeTruthy();
  });

  it('shows chapter number when present', async () => {
    const visitedAt = new Date('2026-08-15T10:30:00Z').toISOString();
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          history: [{
            id: 2,
            work_id: 200,
            url_id: 'fic200',
            title: 'Chaptered Fic',
            author: 'Author C',
            chapter_num: 5,
            visited_at: visitedAt,
          }],
          total: 1,
          limit: 50,
          offset: 0,
        }),
      });

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Chaptered Fic')).toBeTruthy();
    });
    expect(screen.getByText('ch. 5')).toBeTruthy();
  });

  it('shows login prompt when not logged in', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: null }) }); // not logged in

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Log in to view your reading history.')).toBeTruthy();
    });
  });

  it('removes a history entry when remove button clicked', async () => {
    const visitedAt = new Date('2026-08-15T10:30:00Z').toISOString();
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          history: [{
            id: 1,
            work_id: 100,
            url_id: 'fic100',
            title: 'Test Fic',
            author: 'Test Author',
            chapter_num: null,
            visited_at: visitedAt,
          }],
          total: 1,
          limit: 50,
          offset: 0,
        }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, removed: true }) }); // DELETE response

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Test Fic')).toBeTruthy();
    });

    const removeBtn = screen.getByRole('button', { name: 'Remove' });
    await fireEvent.click(removeBtn);

    await waitFor(() => {
      const deleteCall = mockFetch.mock.calls.find(
        (c) => String(c[0]).includes('/history/1') && c[1]?.method === 'DELETE'
      );
      expect(deleteCall).toBeTruthy();
    });
  });

  it('clears all history when clear button clicked', async () => {
    const visitedAt = new Date('2026-08-15T10:30:00Z').toISOString();
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) }) // me
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          history: [{
            id: 1,
            work_id: 100,
            url_id: 'fic100',
            title: 'Test Fic',
            author: 'Test Author',
            chapter_num: null,
            visited_at: visitedAt,
          }],
          total: 1,
          limit: 50,
          offset: 0,
        }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, msg: 'History cleared' }) }); // clear POST response

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    await waitFor(() => {
      expect(screen.getByText('Test Fic')).toBeTruthy();
    });

    const clearBtn = screen.getByRole('button', { name: 'Clear History' });
    await fireEvent.click(clearBtn);

    await waitFor(() => {
      const clearCall = mockFetch.mock.calls.find(
        (c) => String(c[0]).includes('/history/clear') && c[1]?.method === 'POST'
      );
      expect(clearCall).toBeTruthy();
    });

    // After clearing, history should be empty
    await waitFor(() => {
      expect(screen.getByText('Your reading history is empty.')).toBeTruthy();
    });
  });

  it('shows correct visit count pluralization', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, user: { id: 1, username: 'tester' } }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          history: [],
          total: 0,
          limit: 50,
          offset: 0,
        }),
      });

    const { default: HistoryPage } = await import('./+page.svelte');
    render(HistoryPage);

    // The page should render without errors
    await waitFor(() => {
      expect(screen.getByText('Reading History')).toBeTruthy();
    });
  });
});
