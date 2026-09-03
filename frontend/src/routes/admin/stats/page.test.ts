import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin stats page', () => {
  it('renders live totals and the daily rollup rows', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        totals: { users: 1234, works: 5678, bookmarks: 90, requests: 123456 },
        daily: [
          {
            date: '2026-08-09',
            total_users: 1230,
            new_users: 4,
            total_works: 5670,
            new_works: 8,
            manual_uploads: 2,
            epubs_downloaded: 310,
            words_read: 2500000,
          },
        ],
      }),
    });

    const { default: StatsPage } = await import('./+page.svelte');
    render(StatsPage);

    await waitFor(() => {
      expect(screen.getByText('1234')).toBeTruthy();
    });
    expect(screen.getByText('5678')).toBeTruthy();
    expect(screen.getByText('2026-08-09')).toBeTruthy();
    expect(screen.getByText('2.5M')).toBeTruthy();
  });

  it('shows the empty state when no daily stats are recorded', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ err: 0, totals: null, daily: [] }),
    });

    const { default: StatsPage } = await import('./+page.svelte');
    render(StatsPage);

    await waitFor(() => {
      expect(screen.getByText(/No daily stats recorded/)).toBeTruthy();
    });
  });

  it('shows an error message when the API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: StatsPage } = await import('./+page.svelte');
    render(StatsPage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load stats/)).toBeTruthy();
    });
  });
});
