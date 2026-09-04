// Auth store using Svelte 5 runes (.svelte.ts extension required)

import { getMe, login, logout, register, setToken } from '$lib/api/social';
import { mergeOnLogin } from '$lib/stores/deviceLibrary.svelte';
import type { User } from '$lib/api/social-types';

// Last-known signed-in user, persisted so the app can keep working (and keep
// showing the reader) when the network is unavailable. Only ever written when
// a real auth response succeeds; never used to fabricate a login for a browser
// that has no token.
const CACHED_USER_KEY = 'fichub_cached_user';

/**
 * Legacy role → level fallback (F7). Fresh auth responses carry `level`; a
 * cached/legacy user may not. Map the old role ladder onto the site-wide
 * level scale so role gates (curator ≥ 50, admin ≥ 100) keep working until a
 * fresh /api/auth/me lands: admin(10)→100, curator(5)→50, trusted(1)→1.
 */
export function roleToLevel(role: number | undefined): number {
  if (!role) return 0;
  if (role >= 10) return 100;
  if (role >= 5) return 50;
  if (role >= 1) return 1;
  return 0;
}

/** Resolve a user's effective site-wide level (0-100). */
export function userLevel(user: Pick<User, 'role'> & { level?: number } | null | undefined): number {
  if (!user) return 0;
  if (typeof user.level === 'number' && user.level > 0) return user.level;
  return roleToLevel(user.role);
}

function cacheUser(user: User): void {
  try {
    localStorage.setItem(CACHED_USER_KEY, JSON.stringify(user));
  } catch {
    /* storage full/blocked — non-fatal */
  }
}

function readCachedUser(): User | null {
  try {
    const raw = localStorage.getItem(CACHED_USER_KEY);
    return raw ? (JSON.parse(raw) as User) : null;
  } catch {
    return null;
  }
}

function clearCachedUser(): void {
  try {
    localStorage.removeItem(CACHED_USER_KEY);
  } catch {
    /* ignore */
  }
}

class AuthStore {
  user = $state<User | null>(null);
  loading = $state(false);
  initialized = $state(false);

  get isLoggedIn(): boolean {
    return this.user !== null;
  }

  get username(): string | null {
    return this.user?.username ?? null;
  }

  /**
   * Effective site-wide level (0-100). Reads `user.level` from fresh auth
   * responses and falls back to the legacy role ladder for cached users.
   * Never exceeds 100 — guards against a corrupt cached level.
   */
  get level(): number {
    const lvl = userLevel(this.user);
    return Math.min(100, Math.max(0, lvl));
  }

  /** Effective experience points (0 when the user object lacks exp). */
  get exp(): number {
    return Math.max(0, this.user?.exp ?? 0);
  }

  async init(): Promise<void> {
    if (this.initialized) return;
    this.loading = true;
    try {
      const res = await getMe();
      if (res.err === 0 && res.user) {
        this.user = res.user;
        cacheUser(res.user);
      } else {
        this.user = null;
        clearCachedUser();
      }
    } catch {
      // Network failure (e.g. offline). Fall back to the cached user so the
      // reader keeps working for previously opened fics. If there was never a
      // login, the cached user is null and we stay logged out.
      this.user = readCachedUser();
    }
    this.loading = false;
    this.initialized = true;
  }

  async handleLogin(username: string, password: string, rememberMe = false): Promise<boolean> {
    this.loading = true;
    try {
      const res = await login(username, password, rememberMe);
            if (res.err === 0 && res.user) {
        this.user = res.user;
        cacheUser(res.user);
        this.loading = false;
        // Fold any anonymous device-library bookmarks/follows into the account.
        void mergeOnLogin();
        return true;
      }
      this.loading = false;
      return false;
    } catch {
      this.loading = false;
      return false;
    }
  }

  async handleRegister(
    username: string,
    password: string,
    email?: string,
    inviteCode?: string,
  ): Promise<boolean> {
    this.loading = true;
    try {
      const res = await register(username, password, email, inviteCode);
            if (res.err === 0 && res.user) {
        this.user = res.user;
        cacheUser(res.user);
        this.loading = false;
        // Fold any anonymous device-library bookmarks/follows into the account.
        void mergeOnLogin();
        return true;
      }
      this.loading = false;
      return false;
    } catch {
      this.loading = false;
      return false;
    }
  }

  handleLogout(): void {
    logout();
    this.user = null;
    clearCachedUser();
  }
}

export const auth = new AuthStore();
