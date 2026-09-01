// Personal recommendations API client for GET /api/recommendations/personal
// Backend: src/recommender/routes.rs

import type { RecResult } from './types';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(options?.headers ?? {}),
    },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as T;
}

export interface BasedOnItem {
  title: string;
  url_id: string;
}

export interface PersonalRecsResponse {
  err: number;
  enough_data: boolean;
  recs: RecResult[];
  based_on: BasedOnItem[];
  /** Pluggable-engine diagnostics (REC_ENGINE_MODE=pluggable only). */
  strategies?: { name: string; contributed: boolean; count: number; duration_ms: number }[];
  /** Curator-prior weight used in the blend (0..1; present only when the
   *  curator prior is configured and active). */
  curator_alpha?: number;
}

/** GET /api/recommendations/personal — personalized recs from bookmarks+downloads.
 *  Returns enough_data:false (200) for anonymous users or < 3 signals. */
export async function fetchPersonalRecs(): Promise<PersonalRecsResponse> {
  return request<PersonalRecsResponse>('/recommendations/personal');
}

/**
 * Fetch the signed-in user's `recs.personalized` preference from
 * /api/me/prefs. Resolves to `true` when the pref is absent (default-on)
 * or explicitly `"true"`; only resolves to `false` when the stored value
 * is exactly `"false"`.
 *
 * Never throws — on network/auth failure it falls back to `true` so the
 * home dashboard still attempts the personal endpoint (which itself
 * returns `enough_data: false` for anonymous / low-signal users).
 */
export async function isPersonalizedRecsEnabled(): Promise<boolean> {
  try {
    const res = await fetch('/api/me/prefs', { credentials: 'include' });
    if (!res.ok) return true; // default-on on non-200
    const prefs = (await res.json()) as Array<{ key: string; value: string }>;
    const found = prefs.find((p) => p.key === 'recs.personalized');
    if (!found) return true; // key absent → default-on
    return found.value !== 'false';
  } catch {
    return true; // default-on on any fetch/network failure
  }
}

// ── Per-fic "readers also bookmarked" anchors ──────────────────────────────

export interface AlsoBookmarkedItem {
  url_id: string;
  work_id: number | null;
  title: string;
  author: string;
  words: number;
  status: string;
  site_domain: string;
  cooccur_count: number;
}

export interface AlsoBookmarkedResponse {
  err: number;
  url_id: string;
  items: AlsoBookmarkedItem[];
}

/** GET /api/v1/works/{url_id}/also-bookmarked — top co-bookmarked fics
 *  (aggregate co-occurrence counts; no per-user data). */
export async function fetchAlsoBookmarked(url_id: string): Promise<AlsoBookmarkedResponse> {
  return request<AlsoBookmarkedResponse>(
    `/v1/works/${encodeURIComponent(url_id)}/also-bookmarked`,
  );
}
