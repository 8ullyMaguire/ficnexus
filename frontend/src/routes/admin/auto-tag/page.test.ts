import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin auto-tag page', () => {
  it('renders the pending queue with approve/dismiss actions', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        total: 2,
        items: [
          {
            url_id: 'https://archiveofourown.org/works/101',
            title: 'Alpha Fic',
            tag_id: 7,
            tag_name: 'Angst',
            score: 87.5,
            tag_type_id: 4,
            created: '2026-08-01T00:00:00Z',
          },
          {
            url_id: 'https://archiveofourown.org/works/202',
            title: 'Beta Fic',
            tag_id: 9,
            tag_name: 'Hurt/Comfort',
            score: 72,
            tag_type_id: 4,
            created: null,
          },
        ],
      }),
    });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    await waitFor(() => {
      expect(screen.getByText('Alpha Fic')).toBeTruthy();
    });
    expect(screen.getByText('Beta Fic')).toBeTruthy();
    expect(screen.getByText('Angst')).toBeTruthy();
    expect(screen.getByText('Hurt/Comfort')).toBeTruthy();
    // similarity rendered as score/100 with 2 decimals
    expect(screen.getByText('0.88')).toBeTruthy();
    expect(screen.getByText('0.72')).toBeTruthy();
    // count badge: "2 pending"
    expect(screen.getByText(/2 pending/)).toBeTruthy();
    expect(screen.getAllByRole('button', { name: /Approve/i }).length).toBe(2);
    expect(screen.getAllByRole('button', { name: /Dismiss/i }).length).toBe(2);
    // Backfill + run controls present
    expect(screen.getByRole('button', { name: /Backfill tag embeddings/i })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Auto-tag/i })).toBeTruthy();
  });

  it('approves a suggestion via POST and removes it from the queue', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          total: 1,
          items: [
            {
              url_id: 'https://archiveofourown.org/works/101',
              title: 'Alpha Fic',
              tag_id: 7,
              tag_name: 'Angst',
              score: 87.5,
              tag_type_id: 4,
              created: null,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'suggestion approved', url_id: 'https://archiveofourown.org/works/101', tag_id: 7 }),
      });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    const approveBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Approve/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(approveBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/auto-tag/approve/https://archiveofourown.org/works/101/7');
    expect(postCall[1].credentials).toBe('include');

    // Row removed without refetch; count badge drops to 0.
    await waitFor(() => {
      expect(screen.queryByText('Alpha Fic')).toBeNull();
    });
    expect(screen.getByText(/0 pending/)).toBeTruthy();
    expect(screen.getByText(/Approved tag #7/)).toBeTruthy();
  });

  it('dismisses a suggestion via POST and removes it from the queue', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          total: 1,
          items: [
            {
              url_id: 'https://archiveofourown.org/works/202',
              title: 'Beta Fic',
              tag_id: 9,
              tag_name: 'Hurt/Comfort',
              score: 72,
              tag_type_id: 4,
              created: null,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'suggestion dismissed', url_id: 'https://archiveofourown.org/works/202', tag_id: 9 }),
      });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    const dismissBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Dismiss/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(dismissBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/auto-tag/dismiss/https://archiveofourown.org/works/202/9');

    await waitFor(() => {
      expect(screen.queryByText('Beta Fic')).toBeNull();
    });
    expect(screen.getByText(/Dismissed tag #9/)).toBeTruthy();
  });

  it('runs backfill via POST /api/admin/auto-tag/backfill', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, total: 0, items: [] }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, embedded: 3, msg: 'embedded 3 tag(s); re-run to continue' }) });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    const backfillBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Backfill tag embeddings/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(backfillBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/auto-tag/backfill');

    await waitFor(() => {
      expect(screen.getByText(/embedded 3 tag/)).toBeTruthy();
    });
  });

  it('shows the empty state when nothing is pending', async () => {
    mockFetch.mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, total: 0, items: [] }) });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    await waitFor(() => {
      expect(screen.getByText(/No machine-suggested tags pending review/)).toBeTruthy();
    });
  });

  it('shows an error message when the queue API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: AutoTagPage } = await import('./+page.svelte');
    render(AutoTagPage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load queue/)).toBeTruthy();
    });
  });
});
