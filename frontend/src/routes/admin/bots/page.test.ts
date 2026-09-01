import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin/bots page', () => {
  it('renders flagged clients from the API with mirror/stuffing flags', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        err: 0,
        clients: [
          {
            client_id: 'botclient-mirror-1234',
            requests: 100,
            downloads: 99,
            export_ratio: 0.99,
            failed_auths: 8,
            windows: 1,
            bot_score: 0.99,
            flags: ['mirror', 'stuffing'],
          },
          {
            client_id: 'reader-abc',
            requests: 50,
            downloads: 1,
            export_ratio: 0.02,
            failed_auths: 0,
            windows: 2,
            bot_score: 0.02,
            flags: [],
          },
        ],
      }),
    });

    // Lazy-import so fetch is mocked before mount
    const { default: BotsPage } = await import('./+page.svelte');
    render(BotsPage);

    await waitFor(() => {
      expect(screen.getByText('mirror')).toBeTruthy();
      expect(screen.getByText('stuffing')).toBeTruthy();
    });
    expect(screen.getByText('99%')).toBeTruthy();
    expect(screen.getByText('99')).toBeTruthy();
    // Zero-PII hint is present
    expect(screen.getByText(/Zero-PII/)).toBeTruthy();
  });

  it('shows empty state when no flagged clients', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ err: 0, clients: [] }),
    });

    const { default: BotsPage } = await import('./+page.svelte');
    render(BotsPage);

    await waitFor(() => {
      expect(screen.getByText(/No flagged clients/)).toBeTruthy();
    });
  });

  it('shows an error message when the API fails', async () => {
    mockFetch.mockResolvedValueOnce({ ok: false, status: 500 });

    const { default: BotsPage } = await import('./+page.svelte');
    render(BotsPage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load bot data/)).toBeTruthy();
    });
  });

  it('shadowbans a client via POST and reflects the state in the row', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clients: [
            {
              client_id: 'botclient-act-1234',
              requests: 100,
              downloads: 99,
              export_ratio: 0.99,
              failed_auths: 8,
              windows: 1,
              bot_score: 0.99,
              flags: ['mirror'],
              shadowbanned: false,
            },
          ],
        }),
      })
      // The shadowban POST resolves {err:0, shadowbanned:true}.
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, client_id: 'botclient-act-1234', shadowbanned: true }),
      });

    const { default: BotsPage } = await import('./+page.svelte');
    render(BotsPage);

    const banBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Shadowban/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(banBtn);

    // POST went to the right URL with the right method.
    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/bots/botclient-act-1234/shadowban');
    expect(postCall[1].credentials).toBe('include');

    // Row now shows the shadowbanned state + Clear action + confirmation
    // (shortId truncates the 18-char id to `botclien…`).
    await waitFor(() => {
      expect(screen.getByText(/Shadowbanned botclien…/)).toBeTruthy();
    });
    expect(screen.getByRole('button', { name: /Clear/i })).toBeTruthy();
  });

  it('clears a shadowban via POST and reverts the row to the Shadowban action', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clients: [
            {
              client_id: 'botclient-clear-5678',
              requests: 80,
              downloads: 78,
              export_ratio: 0.97,
              failed_auths: 2,
              windows: 1,
              bot_score: 0.98,
              flags: ['mirror'],
              shadowbanned: true,
            },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, client_id: 'botclient-clear-5678', shadowbanned: false }),
      });

    const { default: BotsPage } = await import('./+page.svelte');
    render(BotsPage);

    const clearBtn = await waitFor(() => {
      const btn = screen.getByRole('button', { name: /Clear/i });
      expect(btn).toBeTruthy();
      return btn;
    });
    await fireEvent.click(clearBtn);

    await waitFor(() => {
      const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST');
      expect(postCall).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => c[1]?.method === 'POST')!;
    expect(postCall[0]).toBe('/api/admin/bots/botclient-clear-5678/unshadowban');
    expect(postCall[1].method).toBe('POST');

    await waitFor(() => {
      expect(screen.getByText(/Cleared shadowban for botclien…/)).toBeTruthy();
    });
    expect(screen.getByRole('button', { name: /Shadowban/i })).toBeTruthy();
  });
});
