/**
 * Admin/curator API helper.
 *
 * The backend's AuthUser extractor reads the JWT ONLY from the
 * `Authorization: Bearer *** header — not from cookies. Raw
 * `fetch(path, { credentials: 'include' })` (the old admin-page pattern)
 * therefore authenticates as role 0 → every admin endpoint 403s.
 *
 * Use `adminFetch` (or `adminJson`) in admin/curator pages so the token
 * is attached via the shared auth-headers helper (single source of truth).
 */

import { authHeaders } from './social';

/** fetch() that attaches the JWT. Returns the Response — callers check res.ok
 *  and read body.msg on error (admin endpoints return {err, msg}). */
export async function adminFetch(path: string, init?: RequestInit): Promise<Response> {
  const headers = new Headers(init?.headers);
  const auth = authHeaders();
  for (const [k, v] of Object.entries(auth)) headers.set(k, v);
  return fetch(path, { ...init, headers, credentials: 'include' });
}

/** adminFetch that throws on non-2xx (with the server msg when available). */
export async function adminJson<T = unknown>(path: string, init?: RequestInit): Promise<T> {
  const res = await adminFetch(path, init);
  if (!res.ok) {
    let msg = `HTTP ${res.status}`;
    try {
      const body = (await res.json()) as { msg?: string };
      if (body?.msg) msg = body.msg;
    } catch { /* keep HTTP status */ }
    throw new Error(msg);
  }
  return (await res.json()) as T;
}
