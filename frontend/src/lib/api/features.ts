// API client for the Ability Tree / features system.
// Uses the same auth + request pattern as social.ts.

import { authHeaders } from './social';

const BASE = '/api';

// ── Types ──────────────────────────────────────────────────────────────

export interface Feature {
  id: number;
  slug: string;
  name: string;
  description: string;
  icon: string | null;
  category: string;
  gate_type: string;
  gate_value: number;
  requires_feature: string | null;
  nav_target: string | null;
  widget_component: string | null;
  is_default: boolean;
  is_revocable: boolean;
  sort_hint: number;
  unlocked: boolean;
  enabled: boolean;
}

export interface ProgressionInfo {
  level: number;
  rank: number;
  rank_title: string;
  xp: number;
  xp_to_next_level: number;
  recent_events: {
    event_type: string;
    xp: number;
    created_at: string;
  }[];
}

// ── Request helper ─────────────────────────────────────────────────────

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as T;
}

// ── Endpoints ──────────────────────────────────────────────────────────

/** GET /api/features — all features with user unlock/enable state. */
export async function fetchFeatures(): Promise<Feature[]> {
  return request<Feature[]>('/features');
}

/** GET /api/features/available — features the user has unlocked. */
export async function fetchAvailable(): Promise<Feature[]> {
  return request<Feature[]>('/features/available');
}

/** POST /api/features/{slug}/enable — enable a feature. */
export async function enableFeature(slug: string): Promise<void> {
  await request(`/features/${encodeURIComponent(slug)}/enable`, {
    method: 'POST',
  });
}

/** POST /api/features/{slug}/disable — disable (revoke) a feature. */
export async function disableFeature(slug: string): Promise<void> {
  await request(`/features/${encodeURIComponent(slug)}/disable`, {
    method: 'POST',
  });
}

/** GET /api/me/progression — user's XP, level, rank, recent events. */
export async function fetchProgression(): Promise<ProgressionInfo> {
  return request<ProgressionInfo>('/me/progression');
}
