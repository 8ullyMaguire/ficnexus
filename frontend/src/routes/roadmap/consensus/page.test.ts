import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('roadmap page', () => {
  it('renders the 2x2 arena from the API', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, clusters: [] }) }) // auth.init()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clusters: [
            { id: 1, text: 'Dark mode for night reading', matches_played: 14, elo_rating: 1600 },
            { id: 2, text: 'Send EPUBs to Kindle', matches_played: 9, elo_rating: 1540 },
            { id: 3, text: 'Trope browser', matches_played: 3, elo_rating: 1490 },
            { id: 4, text: 'Typo-tolerant search', matches_played: 1, elo_rating: 1450 },
          ],
        }),
      });

    const { default: RoadmapPage } = await import('./+page.svelte');
    render(RoadmapPage);

    await waitFor(() => {
      expect(screen.getByText('Dark mode for night reading')).toBeTruthy();
    });
    expect(screen.getByText('Send EPUBs to Kindle')).toBeTruthy();
    expect(screen.getByText('Trope browser')).toBeTruthy();
    expect(screen.getByText('Typo-tolerant search')).toBeTruthy();
  });

  it('shows the empty state when the arena has no clusters', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, clusters: [] }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, clusters: [], message: 'No features yet — be the first to suggest one!' }),
      });

    const { default: RoadmapPage } = await import('./+page.svelte');
    render(RoadmapPage);

    await waitFor(() => {
      expect(screen.getByText(/No features yet/)).toBeTruthy();
    });
  });

  it('loads the public consensus behind the toggle: leaderboard + most controversial', async () => {
    // auth store is a module singleton: init() already ran in the first test
    // of this file, so the FIRST fetch here is the arena, the SECOND is the
    // consensus endpoint (lazy, on toggle open).
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clusters: [
            { id: 1, text: 'Dark mode for night reading', matches_played: 14, elo_rating: 1600 },
            { id: 2, text: 'Send EPUBs to Kindle', matches_played: 9, elo_rating: 1540 },
            { id: 3, text: 'Trope browser', matches_played: 3, elo_rating: 1490 },
            { id: 4, text: 'Typo-tolerant search', matches_played: 1, elo_rating: 1450 },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          leaderboard: [
            { id: 1, text: 'Dark mode for night reading', elo_rating: 1600, matches_played: 14, times_picked_best: 9, times_picked_worst: 2, suggestions: 3, status: 'open' },
            { id: 2, text: 'Send EPUBs to Kindle', elo_rating: 1540, matches_played: 9, times_picked_best: 4, times_picked_worst: 1, suggestions: 1, status: 'implemented' },
            { id: 3, text: 'Trope browser', elo_rating: 1490, matches_played: 3, times_picked_best: 1, times_picked_worst: 0, suggestions: 1, status: 'archived' },
          ],
          controversy: [
            { id: 1, text: 'Dark mode for night reading', matches_played: 14, times_picked_best: 9, times_picked_worst: 2, controversy: 11, elo_rating: 1600 },
            { id: 2, text: 'Send EPUBs to Kindle', matches_played: 9, times_picked_best: 4, times_picked_worst: 1, controversy: 5, elo_rating: 1540 },
          ],
        }),
      });

    const { default: RoadmapPage } = await import('./+page.svelte');
    render(RoadmapPage);

    // Arena renders first; the consensus section stays hidden until the toggle.
    await waitFor(() => {
      expect(screen.getByText('Dark mode for night reading')).toBeTruthy();
    });
    expect(screen.queryByTestId('consensus-section')).toBeNull();

    // Open the toggle → lazy-fetches /api/roadmap/consensus (public fetch).
    const toggle = screen.getByTestId('consensus-toggle');
    toggle.click();

    await waitFor(() => {
      expect(screen.getByTestId('consensus-section')).toBeTruthy();
    });
    // Leaderboard row: feature text, rank, Elo, votes, best picks, status badge.
    expect(screen.getAllByText('Dark mode for night reading').length).toBeGreaterThan(0);
    expect(screen.getByText('1600')).toBeTruthy();
    expect(screen.getByText('14')).toBeTruthy();
    expect(screen.getAllByText('9').length).toBeGreaterThan(0); // best-picks + controversy votes
    expect(screen.getByText('shipped')).toBeTruthy(); // 'implemented' → shipped
    expect(screen.getByText('rejected')).toBeTruthy(); // 'archived' → rejected
    // Most controversial: text + controversy score (11), most-controversial first.
    expect(screen.getByText('11')).toBeTruthy();
    expect(screen.getByText('Most controversial')).toBeTruthy();

    // The consensus fetch was a plain public GET (no auth headers).
    const consensusCall = mockFetch.mock.calls.find((c) => c[0] === '/api/roadmap/consensus');
    expect(consensusCall).toBeTruthy();
    expect(consensusCall![1]?.headers?.Authorization).toBeUndefined();
  });
});
