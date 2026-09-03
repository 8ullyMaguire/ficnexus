import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

// Curator Consensus unified feed: loads all kinds, renders rows, filters.
const mockFetch = vi.fn();
vi.mock('$lib/api/admin', () => ({
  adminFetch: (...args: unknown[]) => mockFetch(...args),
}));

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    user: { id: 1, username: 'curator', role: 5 },
  },
}));

function okJson(body: unknown) {
  return { ok: true, status: 200, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

describe('/curator/consensus', () => {
  it('renders normalized items from all consensus surfaces', async () => {
    mockFetch.mockResolvedValue(okJson({
      err: 0,
      counts: { content: 1, roadmap: 1 },
      total: 2,
      items: [
        { kind: 'content', id: 3, title: 'Body fix proposal for abc123', detail: 'missing chapters', status: 'pending', score: 2, created_at: '2026-08-20T10:00:00Z', link: '/curator' },
        { kind: 'roadmap', id: 9, title: 'Roadmap cluster #9', detail: 'batch downloads', status: 'open', score: 1512, created_at: '2026-08-21T10:00:00Z', link: '/roadmap' },
      ],
    }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => {
      expect(screen.getByText('Body fix proposal for abc123')).toBeTruthy();
      expect(screen.getByText('Roadmap cluster #9')).toBeTruthy();
    });
    // Type badges rendered
    expect(screen.getAllByText('Body fixes').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Roadmap consensus').length).toBeGreaterThan(0);
    // Called the unified endpoint
    expect(mockFetch.mock.calls[0][0]).toBe('/api/curator/consensus');
  });

  it('passes kind filter to the endpoint', async () => {
    mockFetch.mockResolvedValue(okJson({ err: 0, items: [], total: 0, counts: {} }));
    const { default: Page } = await import('./+page.svelte');
    render(Page);
    await waitFor(() => expect(mockFetch).toHaveBeenCalled());
    // Simulate selecting roadmap in the dropdown.
    const select = document.querySelector('select') as HTMLSelectElement;
    await fireEvent.change(select, { target: { value: 'roadmap' } });
    await waitFor(() => {
      const calls = mockFetch.mock.calls.map((c) => String(c[0]));
      expect(calls.some((u) => u.includes('kind=roadmap'))).toBe(true);
    });
  });
});
