import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const mockAuth = {
  user: { level: 50, role: 5, user_id: 1, username: 'curator' },
  isLoggedIn: true,
  init: vi.fn().mockResolvedValue(undefined),
};
vi.mock('$lib/stores/auth.svelte', () => ({ auth: mockAuth }));
vi.mock('$lib/prefs', () => ({
  getPref: vi.fn().mockReturnValue('modern'),
}));

beforeEach(() => {
  mockFetch.mockReset();
});

describe('roadmap board page', () => {
  it('renders kanban columns and feature cards from API', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          features: [
            { id: 1, text: 'Dark mode for night reading', elo_rating: 1600, matches_played: 14, times_picked_best: 9, times_picked_worst: 2, status: 'shipped', category: 'reader', suggestions: 3 },
            { id: 2, text: 'Send EPUBs to Kindle', elo_rating: 1540, matches_played: 9, times_picked_best: 4, times_picked_worst: 1, status: 'up_next', category: 'recs', suggestions: 1 },
            { id: 3, text: 'Trope browser', elo_rating: 1490, matches_played: 3, times_picked_best: 1, times_picked_worst: 0, status: 'idea', category: 'search', suggestions: 1 },
          ],
        }),
      });

    const { default: BoardPage } = await import('./+page.svelte');
    render(BoardPage);

    await waitFor(() => {
      expect(screen.getAllByText('Dark mode for night reading').length).toBeGreaterThan(0);
    });
    expect(screen.getByText('Send EPUBs to Kindle')).toBeTruthy();
    expect(screen.getByText('Trope browser')).toBeTruthy();

    // Kanban columns render
    expect(screen.getByLabelText('Shipped')).toBeTruthy();
    expect(screen.getByLabelText('Up Next')).toBeTruthy();
    expect(screen.getByLabelText('Ideas')).toBeTruthy();
  });

  it('shows category filter chips', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ err: 0, features: [] }),
    });

    const { default: BoardPage } = await import('./+page.svelte');
    render(BoardPage);

    await waitFor(() => {
      expect(screen.getByText('All')).toBeTruthy();
    });
    expect(screen.getByText('Reader')).toBeTruthy();
    expect(screen.getByText('Search')).toBeTruthy();
  });

  it('shows arena CTA in Ideas column', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        features: [
          { id: 1, text: 'Some idea', elo_rating: 1500, matches_played: 2, times_picked_best: 1, times_picked_worst: 1, status: 'idea', category: 'general', suggestions: 1 },
        ],
      }),
    });

    const { default: BoardPage } = await import('./+page.svelte');
    render(BoardPage);

    await waitFor(() => {
      expect(screen.getByTestId('arena-cta')).toBeTruthy();
    });
    expect(screen.getByText(/features ranked by community/i)).toBeTruthy();
  });

  it('shows curator move dropdown for curator-level users', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        features: [
          { id: 1, text: 'Test feature', elo_rating: 1500, matches_played: 0, times_picked_best: 0, times_picked_worst: 0, status: 'idea', category: 'general', suggestions: 0 },
        ],
      }),
    });

    const { default: BoardPage } = await import('./+page.svelte');
    render(BoardPage);

    await waitFor(() => {
      expect(screen.getByDisplayValue('Ideas')).toBeTruthy();
    });
  });
});
