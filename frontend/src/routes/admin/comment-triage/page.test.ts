import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin comment triage page', () => {
  it('renders flagged comments with category badges, reason and confidence', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        items: [
          {
            comment_id: 11,
            body: 'This fic is garbage and the author should quit.',
            category: 'toxic',
            reason: 'personal attack on the author',
            confidence: 0.97,
            work_id: 101,
            title: 'Alpha Fic',
            user_id: 5,
          },
          {
            comment_id: 12,
            body: 'buy cheap epubs at scam.example',
            category: 'spam',
            reason: 'promotional link',
            confidence: 0.99,
            work_id: null,
            title: null,
            user_id: null,
          },
        ],
      }),
    });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    await waitFor(() => {
      expect(screen.getByText(/This fic is garbage/)).toBeTruthy();
    });
    expect(screen.getAllByText('toxic').length).toBeGreaterThan(0);
    expect(screen.getAllByText('spam').length).toBeGreaterThan(0);
    expect(screen.getByText(/personal attack on the author/)).toBeTruthy();
    expect(screen.getByText(/promotional link/)).toBeTruthy();
    expect(screen.getByText(/confidence 97%/)).toBeTruthy();
    expect(screen.getByText(/Alpha Fic/)).toBeTruthy();
    expect(screen.getByText(/#5/)).toBeTruthy();
  });

  it('shows the empty state when nothing is flagged', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ err: 0, items: [] }),
    });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    await waitFor(() => {
      expect(screen.getByText(/No flagged comments/)).toBeTruthy();
    });
  });

  it('shows an error message when the API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load comment triage/)).toBeTruthy();
    });
  });

  it('hides a comment via POST and removes it from the queue', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          items: [
            {
              comment_id: 11,
              body: 'This fic is garbage and the author should quit.',
              category: 'toxic',
              reason: 'personal attack on the author',
              confidence: 0.97,
              work_id: 101,
              title: 'Alpha Fic',
              user_id: 5,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'Comment hidden', comment_id: 11 }),
      });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    const hideBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Hide/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(hideBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/moderation/comments/11/hide');
    expect(postCall[1].credentials).toBe('include');

    // Card removed without refetch.
    await waitFor(() => {
      expect(screen.queryByText(/This fic is garbage/)).toBeNull();
    });
    expect(screen.getByText(/Hidden comment #11/)).toBeTruthy();
  });

  it('deletes a comment via POST and removes it from the queue', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          items: [
            {
              comment_id: 12,
              body: 'buy cheap epubs at scam.example',
              category: 'spam',
              reason: 'promotional link',
              confidence: 0.99,
              work_id: null,
              title: null,
              user_id: null,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, msg: 'Comment deleted', comment_id: 12 }),
      });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    const deleteBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Delete/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(deleteBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/moderation/comments/12/delete');
    expect(postCall[1].method).toBe('POST');

    await waitFor(() => {
      expect(screen.queryByText(/buy cheap epubs/)).toBeNull();
    });
    expect(screen.getByText(/Deleted comment #12/)).toBeTruthy();
  });

  it('keeps the comment in the queue and shows an error when hide fails', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          items: [
            {
              comment_id: 13,
              body: 'still here',
              category: 'spam',
              reason: 'promotional link',
              confidence: 0.9,
              work_id: null,
              title: null,
              user_id: null,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({ ok: false, status: 404, json: async () => ({ err: -1, msg: 'Comment not found' }) });

    const { default: TriagePage } = await import('./+page.svelte');
    render(TriagePage);

    const hideBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Hide/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(hideBtn);

    await waitFor(() => {
      expect(screen.getByText(/hide failed: Comment not found/)).toBeTruthy();
    });
    // Card stays in the queue.
    expect(screen.getByText(/still here/)).toBeTruthy();
  });
});
