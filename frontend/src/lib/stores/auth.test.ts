import { describe, it, expect, vi, beforeEach } from 'vitest';

const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  vi.resetModules();
});

describe('auth store offline fallback', () => {
  it('falls back to the cached user when /api/auth/me fails (offline)', async () => {
    // Simulate a prior successful login: cached user present in localStorage.
    const cached = { id: 7, username: 'reader', role: 0, reputation: 3 };
    localStorage.setItem('fichub_cached_user', JSON.stringify(cached));
    // ... and the token exists (logged-in browser)
    localStorage.setItem('fichub_token', 'some-jwt');

    // getMe() throws because the network is down
    mockFetch.mockRejectedValue(new Error('NetworkError: offline'));

    const { auth } = await import('./auth.svelte');
    await auth.init();

    expect(auth.isLoggedIn).toBe(true);
    expect(auth.username).toBe('reader');
  });

  it('stays logged out when offline and no cached user exists', async () => {
    mockFetch.mockRejectedValue(new Error('NetworkError: offline'));

    const { auth } = await import('./auth.svelte');
    await auth.init();

    expect(auth.isLoggedIn).toBe(false);
  });

  it('caches the user on a successful getMe', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        err: 0,
        user: { id: 3, username: 'alice', role: 0, reputation: 0, level: 12, exp: 4800 },
      }),
    });

    const { auth } = await import('./auth.svelte');
    await auth.init();

    expect(auth.username).toBe('alice');
    expect(auth.level).toBe(12);
    expect(auth.exp).toBe(4800);
    const cached = JSON.parse(localStorage.getItem('fichub_cached_user') || 'null');
    expect(cached?.username).toBe('alice');
  });

  it('clears the cached user on logout', async () => {
    localStorage.setItem('fichub_cached_user', JSON.stringify({ id: 1, username: 'bob' }));
    const { auth } = await import('./auth.svelte');
    auth.handleLogout();
    expect(localStorage.getItem('fichub_cached_user')).toBeNull();
  });
});
