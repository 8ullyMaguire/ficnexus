import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    level: 3,
    exp: 0,
    user: { id: 1, username: 'alice', role: 0, reputation: 0, level: 3, email: null, locale: 'en' },
  },
}));

function jsonResponse(body: unknown) {
  return { ok: true, json: async () => body };
}

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  localStorage.setItem("fichub_prefs_v1", JSON.stringify({ uiMode: "archive" }));
});

describe('forum blocks page', () => {
  it('lists blocked users from GET /api/blocks', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/blocks')) {
        return Promise.resolve(
          jsonResponse({
            err: 0,
            items: [
              { user_id: 7, username: 'troll1', created_at: '2026-08-12T10:00:00Z' },
              { user_id: 8, username: 'troll2', created_at: '2026-08-13T11:00:00Z' },
            ],
          }),
        );
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('troll1')).toBeTruthy();
    });
    expect(screen.getByText('troll2')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: 'Unblock' })).toHaveLength(2);
  });

  it('blocks a user by id via POST /api/blocks and refreshes the list', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/blocks') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, user_id: 42, blocked: true }));
      }
      if (url.endsWith('/api/blocks')) {
        return Promise.resolve(
          jsonResponse({
            err: 0,
            items: [{ user_id: 42, username: 'newblocked', created_at: '2026-08-14T00:00:00Z' }],
          }),
        );
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Block' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/42/), { target: { value: '42' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Block' }));

    await waitFor(() => {
      expect(screen.getByText('newblocked')).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/blocks') && c[1]?.method === 'POST');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(String(postCall![1].body)).user_id).toBe(42);
  });

  it('unblocks via DELETE /api/blocks/{id} and removes the row', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.includes('/api/blocks/') && init?.method === 'DELETE') {
        return Promise.resolve(jsonResponse({ err: 0, user_id: 7, blocked: false }));
      }
      if (url.endsWith('/api/blocks')) {
        return Promise.resolve(
          jsonResponse({
            err: 0,
            items: [{ user_id: 7, username: 'troll1', created_at: '2026-08-12T10:00:00Z' }],
          }),
        );
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('troll1')).toBeTruthy();
    });
    await fireEvent.click(screen.getByRole('button', { name: 'Unblock' }));

    await waitFor(() => {
      // The DELETE went out (mock matched) and the row dropped optimistically.
      const delCall = mockFetch.mock.calls.find((c) => String(c[0]).includes('/api/blocks/') && c[1]?.method === 'DELETE');
      expect(delCall).toBeTruthy();
    });
    expect(screen.queryByText('troll1')).toBeNull();
    // After the optimistic removal the empty state shows (list reload not
    // needed — the row is dropped client-side).
    expect(screen.getByText(/You have not blocked anyone/)).toBeTruthy();
  });

  it('surfaces the self-block error from the backend', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/blocks') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 400, msg: 'You cannot block yourself' }));
      }
      if (url.endsWith('/api/blocks')) {
        return Promise.resolve(jsonResponse({ err: 0, items: [] }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Block' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/42/), { target: { value: '1' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Block' }));

    await waitFor(() => {
      expect(screen.getByText(/You cannot block yourself/)).toBeTruthy();
    });
  });
});
