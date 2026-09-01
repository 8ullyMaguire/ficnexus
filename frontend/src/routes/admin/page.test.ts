import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('admin command center', () => {
  it('renders stat cards from realtime', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          exports_5m: 12,
          searches_5m: 34,
          flagged_clients_1h: 2,
          active_ips_1h: 9,
          redis: { ok: true, used_memory_bytes: 1048576 },
        }),
      })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, clients: [] }) })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, zero_result_queries: [], search_volume: [] }),
      });

    const { default: AdminPage } = await import('./+page.svelte');
    render(AdminPage);

    await waitFor(() => {
      expect(screen.getByText('12')).toBeTruthy();
      expect(screen.getByText('34')).toBeTruthy();
      expect(screen.getByText('2')).toBeTruthy();
    });
    expect(screen.getByText(/Redis/)).toBeTruthy();
  });

  it('shows top flags with badges', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          exports_5m: 0,
          searches_5m: 0,
          flagged_clients_1h: 1,
          active_ips_1h: 1,
          redis: { ok: true, used_memory_bytes: 0 },
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clients: [
            { client_id: 'abc123def456', bot_score: 0.95, export_ratio: 0.98, failed_auths: 5, flags: ['mirror', 'stuffing'], shadowbanned: true },
          ],
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ err: 0, zero_result_queries: [{ query: 'hary pottr', count: 3 }], search_volume: [] }),
      });

    const { default: AdminPage } = await import('./+page.svelte');
    render(AdminPage);

    await waitFor(() => {
      expect(screen.getByText('mirror')).toBeTruthy();
      expect(screen.getByText('stuffing')).toBeTruthy();
      expect(screen.getByText('hary pottr')).toBeTruthy();
    });
  });

  it('per-card fallback: a failing source does not blank the page', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: false, json: async () => ({}) }) // realtime fails
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, clients: [] }) })
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, zero_result_queries: [], search_volume: [] }) });

    const { default: AdminPage } = await import('./+page.svelte');
    render(AdminPage);

    // Page still renders (no crash); zero-results card shows its empty state.
    await waitFor(() => {
      expect(screen.getByText(/No zero-result queries/)).toBeTruthy();
    });
  });
});
