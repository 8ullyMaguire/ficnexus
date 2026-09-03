import { describe, it, expect, vi, beforeEach } from 'vitest';

// Auth store: login/register flows, failed-login handling, and the
// initialized short-circuit.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
  vi.resetModules();
});

function okJson(body: unknown) {
  return { ok: true, json: async () => body };
}

describe('auth store level/exp', () => {
  it('derives level from a fresh auth response (level+exp present)', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't1', user: { id: 1, username: 'alice', role: 5, reputation: 0, level: 62, exp: 25000 } }),
    );
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleLogin('alice', 'pw');
    expect(ok).toBe(true);
    expect(auth.level).toBe(62);
    expect(auth.exp).toBe(25000);
  });

  it('falls back to the role ladder for legacy users without level', async () => {
    const { auth, roleToLevel, userLevel } = await import('./auth.svelte');
    // admin(10) → 100
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't', user: { id: 1, username: 'admin', role: 10, reputation: 0 } }),
    );
    await auth.handleLogin('admin', 'pw');
    expect(auth.level).toBe(100);
    // curator(5) → 50
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't', user: { id: 2, username: 'curator', role: 5, reputation: 0 } }),
    );
    await auth.handleLogin('curator', 'pw');
    expect(auth.level).toBe(50);
    // trusted(1) → 1
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't', user: { id: 3, username: 'trusted', role: 1, reputation: 0 } }),
    );
    await auth.handleLogin('trusted', 'pw');
    expect(auth.level).toBe(1);
    // anonymous → 0
    expect(userLevel(null)).toBe(0);
    expect(roleToLevel(0)).toBe(0);
  });

  it('clamps a corrupt cached level to 0-100', async () => {
    localStorage.setItem('fichub_cached_user', JSON.stringify({ id: 9, username: 'ghost', role: 0, reputation: 0, level: 999 }));
    mockFetch.mockRejectedValue(new Error('offline'));
    const { auth } = await import('./auth.svelte');
    await auth.init();
    expect(auth.level).toBe(100);
  });
});

describe('auth store flows', () => {
  it('handleLogin stores the user and returns true', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't1', user: { id: 1, username: 'alice', role: 0, reputation: 0 } }),
    );
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleLogin('alice', 'pw');
    expect(ok).toBe(true);
    expect(auth.username).toBe('alice');
    expect(auth.isLoggedIn).toBe(true);
    // Token persisted for subsequent API calls.
    expect(localStorage.getItem('fichub_token')).toBe('t1');
  });

  it('handleLogin returns false on server rejection', async () => {
    mockFetch.mockResolvedValue(okJson({ err: -1, msg: 'bad credentials' }));
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleLogin('alice', 'wrong');
    expect(ok).toBe(false);
    expect(auth.isLoggedIn).toBe(false);
  });

  it('handleLogin returns false on network failure', async () => {
    mockFetch.mockRejectedValue(new Error('offline'));
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleLogin('alice', 'pw');
    expect(ok).toBe(false);
  });

  it('handleRegister stores the user and returns true', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, token: 't2', user: { id: 2, username: 'bob', role: 0, reputation: 0 } }),
    );
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleRegister('bob', 'pw', 'bob@example.com');
    expect(ok).toBe(true);
    expect(auth.username).toBe('bob');
  });

  it('handleRegister returns false on failure', async () => {
    mockFetch.mockResolvedValue(okJson({ err: -1, msg: 'username taken' }));
    const { auth } = await import('./auth.svelte');
    const ok = await auth.handleRegister('bob', 'pw');
    expect(ok).toBe(false);
  });

  it('clears the user when getMe returns err (invalidated session)', async () => {
    // A cached user exists, but the server says the session is gone.
    localStorage.setItem('fichub_cached_user', JSON.stringify({ id: 1, username: 'ghost', role: 0, reputation: 0 }));
    mockFetch.mockResolvedValue(okJson({ err: -2, msg: 'session expired' }));
    const { auth } = await import('./auth.svelte');
    await auth.init();
    expect(auth.isLoggedIn).toBe(false);
    expect(localStorage.getItem('fichub_cached_user')).toBeNull();
  });

  it('init short-circuits once initialized', async () => {
    mockFetch.mockResolvedValue(
      okJson({ err: 0, user: { id: 1, username: 'alice', role: 0, reputation: 0 } }),
    );
    const { auth } = await import('./auth.svelte');
    await auth.init();
    expect(auth.initialized).toBe(true);
    const callsBefore = mockFetch.mock.calls.length;
    await auth.init();
    expect(mockFetch.mock.calls.length).toBe(callsBefore);
  });
});
