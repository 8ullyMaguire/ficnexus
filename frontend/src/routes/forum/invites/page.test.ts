import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/svelte';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

vi.mock('$lib/stores/auth.svelte', () => ({
  auth: {
    init: vi.fn().mockResolvedValue(undefined),
    isLoggedIn: true,
    level: 80,
    exp: 0,
    user: { id: 1, username: 'curator', role: 5, reputation: 0, level: 80, email: null, locale: 'en' },
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

describe('forum invites page', () => {
  it('lists existing invites with usage', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/admin/invites')) {
        return Promise.resolve(
          jsonResponse({
            err: 0,
            items: [
              { id: 1, code: 'ABC123xyz789', created_by: 1, created_at: '2026-08-10T00:00:00Z', expires_at: null, used_by: null, used_username: null, used_at: null },
              { id: 2, code: 'DEF456uvw012', created_by: 1, created_at: '2026-08-11T00:00:00Z', expires_at: '2026-09-01T00:00:00Z', used_by: 9, used_username: 'bob', used_at: '2026-08-12T00:00:00Z' },
            ],
          }),
        );
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText('ABC123xyz789')).toBeTruthy();
    });
    expect(screen.getByText('DEF456uvw012')).toBeTruthy();
    expect(screen.getByText(/used by bob/)).toBeTruthy();
    expect(screen.getByText(/unused/)).toBeTruthy();
  });

  it('creates an invite via POST /api/admin/invites and shows the code', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith('/api/admin/invites') && init?.method === 'POST') {
        return Promise.resolve(jsonResponse({ err: 0, id: 3, code: 'NEWCODE0001', created_at: '2026-08-14T00:00:00Z', expires_at: null }));
      }
      if (url.endsWith('/api/admin/invites')) {
        // After creation the page reloads the list.
        return Promise.resolve(
          jsonResponse({
            err: 0,
            items: [{ id: 3, code: 'NEWCODE0001', created_by: 1, created_at: '2026-08-14T00:00:00Z', expires_at: null, used_by: null, used_username: null, used_at: null }],
          }),
        );
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByRole('button', { name: 'Generate invite' })).toBeTruthy();
    });
    await fireEvent.input(screen.getByPlaceholderText(/who this code is for/), { target: { value: 'for a friend' } });
    await fireEvent.click(screen.getByRole('button', { name: 'Generate invite' }));

    await waitFor(() => {
      expect(screen.getByText('NEWCODE0001')).toBeTruthy();
    });
    const postCall = mockFetch.mock.calls.find((c) => String(c[0]).endsWith('/api/admin/invites') && c[1]?.method === 'POST');
    expect(postCall).toBeTruthy();
    expect(JSON.parse(String(postCall![1].body)).note).toBe('for a friend');
  });

  it('shows an error when not permitted (level gate)', async () => {
    mockFetch.mockImplementation((input: RequestInfo | URL) => {
      const url = String(input);
      if (url.endsWith('/api/admin/invites')) {
        return Promise.resolve(jsonResponse({ err: 403, msg: 'forbidden' }));
      }
      return Promise.reject(new Error(`unexpected: ${url}`));
    });

    const { default: Page } = await import('./+page.svelte');
    render(Page);

    await waitFor(() => {
      expect(screen.getByText(/forbidden/)).toBeTruthy();
    });
  });
});
