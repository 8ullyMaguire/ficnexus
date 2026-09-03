import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

const mockGoto = vi.fn();
vi.mock('$app/navigation', () => ({
  goto: (path: string) => mockGoto(path),
}));

// Admin-level user (role 10 → level 100 under the F7 fallback) so the page
// renders its content instead of redirecting to '/' (the redirect target
// itself is a no-op via the mock).
vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    user: { id: 1, username: 'curator', role: 10, reputation: 0, level: 100 },
    level: 100,
  },
}));

beforeEach(() => {
  vi.clearAllMocks();
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
  mockGoto.mockReset();
});

describe('curator flags page', () => {
  it('lists open tag flags with work, tag, reason and flagged-by', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        flags: [
          {
            id: 7,
            url_id: 'fic-alpha',
            tag_id: 42,
            flagged_by_ip: '203.0.113.9',
            reason: 'Wrong ship tag',
            resolved: false,
            created_at: '2026-08-09T12:00:00Z',
          },
        ],
      }),
    });

    const { default: FlagsPage } = await import('./+page.svelte');
    render(FlagsPage);

    await waitFor(() => {
      expect(screen.getByText('#7')).toBeTruthy();
    });
    expect(screen.getByText('fic-alpha')).toBeTruthy();
    expect(screen.getByText('#42')).toBeTruthy();
    expect(screen.getByText('Wrong ship tag')).toBeTruthy();
    expect(screen.getByText('203.0.113.9')).toBeTruthy();
  });

  it('resolves a flag via POST /api/curator/flags/{id}/resolve and removes it from the queue', async () => {
    // URL-routed mocks: onMount fires flags + aliases (in that order), and the
    // resolve POST must land on the third call — positional Once-queues break
    // when the alias list fetch is added.
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.includes('/api/curator/flags') && !url.includes('/resolve')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({
            err: 0,
            flags: [
              {
                id: 9,
                url_id: 'fic-beta',
                tag_id: 7,
                flagged_by_ip: '198.51.100.2',
                reason: 'Duplicate tag',
                resolved: false,
                created_at: '2026-08-09T15:00:00Z',
              },
            ],
          }),
        });
      }
      if (url.includes('/api/curator/aliases')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, aliases: [] }) });
      }
      if (url.includes('/api/curator/flags/9/resolve')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, msg: 'flag updated' }) });
      }
      return Promise.resolve({ ok: true, json: async () => ({ err: 0 }) });
    });

    const { default: FlagsPage } = await import('./+page.svelte');
    render(FlagsPage);

    const resolveBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Resolve/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(resolveBtn);

    // POST went to the right URL with the right method + body.
    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/curator/flags/9/resolve');
    expect(postCall[1].credentials).toBe('include');
    expect(JSON.parse(postCall[1].body as string)).toEqual({ resolved: true });

    // Queue now empty + confirmation shown.
    await waitFor(() => {
      expect(screen.getByText(/Flag #9 resolved/)).toBeTruthy();
    });
    expect(screen.getByText(/No open tag flags/)).toBeTruthy();
  });

  it('shows the empty state when the queue is empty', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ err: 0, flags: [] }),
    });

    const { default: FlagsPage } = await import('./+page.svelte');
    render(FlagsPage);

    await waitFor(() => {
      expect(screen.getByText(/No open tag flags/)).toBeTruthy();
    });
  });

  it('shows an error message when the API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: FlagsPage } = await import('./+page.svelte');
    render(FlagsPage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load tag flags/)).toBeTruthy();
    });
  });
});
