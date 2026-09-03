import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

// Mock the API layer + auth store. The updates page calls getUpdates() on
// mount and markAllUpdatesSeen() once items arrive; both go through
// $lib/api/social which we mock entirely.
vi.mock('$lib/api/social', () => ({
  getUpdates: vi.fn(),
  markAllUpdatesSeen: vi.fn().mockResolvedValue(undefined),
}));

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
  goto: (path: string) => mockGoto(path),
}));

// Logged-in by default so the page loads its feed instead of redirecting.
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    isLoggedIn: true,
    user: { id: 1, username: 'tester', role: 0, reputation: 0 },
  },
}));

import { getUpdates, markAllUpdatesSeen } from '$lib/api/social';
import type { FollowedWorkUpdate } from '$lib/api/social-types';

const sampleItem = (over: Partial<FollowedWorkUpdate> = {}): FollowedWorkUpdate => ({
  follow_id: 11,
  work_id: 101,
  url_id: 'fic-alpha',
  title: 'Alpha Fic',
  author: 'Author A',
  words: 50000,
  chapters: 12,
  status: 'ongoing',
  fic_updated: new Date(Date.now() - 3600_000).toISOString(),
  updated_ago: '1 hour ago',
  is_new: true,
  ...over,
});
beforeEach(() => {
  vi.clearAllMocks();
  (getUpdates as ReturnType<typeof vi.fn>).mockReset();
  (markAllUpdatesSeen as ReturnType<typeof vi.fn>).mockReset();
  (markAllUpdatesSeen as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);
});

describe('updates feed page', () => {
  it('lists followed works with title, author and updated-ago', async () => {
    (getUpdates as ReturnType<typeof vi.fn>).mockResolvedValue({
      err: 0,
      unseen_count: 2,
      items: [
        sampleItem(),
        sampleItem({
          follow_id: 12,
          work_id: 102,
          url_id: 'fic-beta',
          title: 'Beta Fic',
          author: 'Author B',
          is_new: false,
          fic_updated: new Date(Date.now() - 3 * 24 * 3600_000).toISOString(),
        }),
      ],
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Alpha Fic')).toBeTruthy();
    });
    expect(screen.getByText('Beta Fic')).toBeTruthy();
    expect(screen.getAllByText(/1 hour ago/).length).toBeGreaterThan(0);
    expect(screen.getByText(/3 days ago/)).toBeTruthy();
    expect(screen.getByText(/by Author A/)).toBeTruthy();
  });

  it('shows a NEW badge only for unseen fics and links to the fic page', async () => {
    (getUpdates as ReturnType<typeof vi.fn>).mockResolvedValue({
      err: 0,
      unseen_count: 1,
      items: [sampleItem(), sampleItem({ follow_id: 12, url_id: 'fic-beta', title: 'Beta Fic', is_new: false })],
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('NEW')).toBeTruthy();
    });
    // Only one NEW badge for the unseen fic
    expect(screen.getAllByText('NEW')).toHaveLength(1);
    const link = screen.getByRole('link', { name: 'Alpha Fic' });
    expect(link.getAttribute('href')).toBe('/works/fic-alpha');
  });

  it('marks everything seen after loading (badges clear on next visit)', async () => {
    (getUpdates as ReturnType<typeof vi.fn>).mockResolvedValue({
      err: 0,
      unseen_count: 1,
      items: [sampleItem()],
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('Alpha Fic')).toBeTruthy();
    });
    await waitFor(() => {
      expect(markAllUpdatesSeen).toHaveBeenCalled();
    });
    const arg = (markAllUpdatesSeen as ReturnType<typeof vi.fn>).mock.calls[0][0] as { follow_id: number }[];
    expect(arg[0].follow_id).toBe(11);
  });

  it('shows the empty state when nothing is followed', async () => {
    (getUpdates as ReturnType<typeof vi.fn>).mockResolvedValue({ err: 0, unseen_count: 0, items: [] });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/nothing here yet/i)).toBeTruthy();
    });
  });
});
