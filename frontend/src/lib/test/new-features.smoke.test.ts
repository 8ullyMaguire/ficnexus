import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';

// ---------------------------------------------------------------------------
// Smoke tests for recently-added features not covered by the navigation
// integration suite. Each asserts at least one visible/observable behavior so
// a missing route, blank page, or dead API path is caught in the future.
//
// 1. Send-to-Kindle — API client POSTs to /api/send-to-kindle with the auth
//    header, and refuses to run without a token (the fic-page button guards
//    on auth; the client is the second line of defense).
// 2. User data export — the "📦 Export My Data" button in the real Account
//    dropdown calls /api/user/export and triggers a browser download.
// ---------------------------------------------------------------------------

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

afterEach(() => {
  vi.restoreAllMocks();
});

// ── Send-to-Kindle ─────────────────────────────────────────────────────────

describe('Send-to-Kindle smoke', () => {
  it('POSTs to /api/send-to-kindle with the auth header when logged in', async () => {
    localStorage.setItem('fichub_token', 'kindle-token');
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, url_id: 'fic1', to: 'user@kindle.com' }),
    });

    const { sendToKindle } = await import('$lib/api/kindle');
    const res = await sendToKindle({ url_id: 'fic1' });

    expect(res.err).toBe(0);
    expect(res.to).toBe('user@kindle.com');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/send-to-kindle');
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe('Bearer kindle-token');
    expect(JSON.parse(init.body)).toEqual({ url_id: 'fic1' });
  });

  it('throws when the user is not logged in (no token)', async () => {
    localStorage.removeItem('fichub_token');
    const { sendToKindle } = await import('$lib/api/kindle');
    await expect(sendToKindle({ url_id: 'fic1' })).rejects.toThrow(
      /must be logged in/i,
    );
    // No request is made without a token.
    expect(mockFetch).not.toHaveBeenCalled();
  });
});

// ── User data export button ────────────────────────────────────────────────

describe('User data export button smoke', () => {
  // The layout renders the Account dropdown only when logged in. Seed the
  // auth store as logged-in, then render the real +layout.svelte (route '/')
  // and click through the dropdown to the export button.
  async function renderLayoutLoggedIn() {
    // Mock $app modules.
    vi.mock('$app/stores', () => ({
      page: {
        subscribe: (fn: (v: unknown) => void) => {
          fn({ url: new URL('http://localhost/') });
          return () => {};
        },
      },
    }));
    vi.mock('$app/navigation', () => ({
      goto: vi.fn(),
    }));

    // Seed a logged-in user in the auth store.
    localStorage.setItem('fichub_cached_user', JSON.stringify({ username: 'alice', role: 10 }));
    // getMe() returns the same user so auth.init() keeps them logged in.
    mockFetch.mockImplementation((url: RequestInfo | URL) => {
      const u = String(url);
      if (u.startsWith('/api/auth/me')) {
        return Promise.resolve({
          ok: true,
          json: async () => ({ err: 0, user: { username: 'alice', role: 10 } }),
        });
      }
      if (u.startsWith('/api/locales')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, locales: [{ code: 'en', name: 'English' }] }) });
      }
      if (u.startsWith('/api/recommendations/personal')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, enough_data: false, recs: [], based_on: [] }) });
      }
      if (u.startsWith('/api/trending')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, trending: [] }) });
      }
      if (u.startsWith('/api/blind-date')) {
        return Promise.resolve({ ok: true, json: async () => ({ err: 0, fic: null }) });
      }
      if (u.startsWith('/api/user/export')) {
        return Promise.resolve({
          ok: true,
          blob: async () => new Blob(['zip'], { type: 'application/zip' }),
        });
      }
      return Promise.resolve({ ok: true, json: async () => ({ err: 0 }) });
    });

    const { default: Layout } = await import('../../routes/+layout.svelte');
    render(Layout);
    // Wait for auth.init() to resolve (the account dropdown appears — its
    // trigger label is the logged-in username).
    await screen.findByRole('button', { name: /alice/i });
  }

  it('renders the Export My Data button in the Account dropdown', async () => {
    await renderLayoutLoggedIn();
    // The Account dropdown's trigger label is the logged-in username.
    const accountBtn = screen.getByRole('button', { name: /alice/i });
    await fireEvent.click(accountBtn);
    expect(await screen.findByText(/export my data/i)).toBeTruthy();
  });

  it('clicking Export My Data calls /api/user/export and triggers a download', async () => {
    // jsdom lacks URL.createObjectURL — define it (not spyOn: the method does
    // not exist on the URL constructor in jsdom).
    (URL as unknown as { createObjectURL: (b: Blob) => string }).createObjectURL = vi.fn(() => 'blob:mock');
    (URL as unknown as { revokeObjectURL: (u: string) => void }).revokeObjectURL = vi.fn();
    const anchorClickSpy = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    await renderLayoutLoggedIn();
    const accountBtn = screen.getByRole('button', { name: /alice/i });
    await fireEvent.click(accountBtn);
    const exportBtn = await screen.findByText(/export my data/i);
    await fireEvent.click(exportBtn);

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith('/api/user/export', expect.anything());
    });
    expect(anchorClickSpy).toHaveBeenCalled();
    expect(URL.createObjectURL).toHaveBeenCalled();

    anchorClickSpy.mockRestore();
  });
});
