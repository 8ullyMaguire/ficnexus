// Forum user profile + XP history API client.

import { authHeaders } from '$lib/api/social';

const BASE = '/api/forum';

async function request<T>(path: string): Promise<T> {
  const headers = { 'Content-Type': 'application/json', ...authHeaders() };
  const res = await fetch(`${BASE}${path}`, { headers });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json() as Promise<T>;
}

export interface UserProfile {
  id: number;
  username: string;
  email: string | null;
  level: number;
  exp: number;
  xp: number;
  rank: number;
  trust: number;
  reputation: number;
  created_at: string;
  last_active_at: string | null;
  total_words_read: number;
  total_works_read: number;
  next_level_exp: number;
  level_start_exp: number;
  progress_percent: number;
}

export interface XpEvent {
  event_type: string;
  xp: number;
  created_at: string;
}

export interface XpHistory {
  current_xp: number;
  current_level: number;
  xp_in_level: number;
  next_level_xp: number;
  level_start_xp: number;
  progress_percent: number;
  recent_events: XpEvent[];
  today_by_type: { event_type: string; total_xp: number }[];
}

export function fetchUserProfile(userId: number): Promise<UserProfile> {
  return request(`/users/${userId}/profile`);
}

export function fetchXpHistory(userId: number): Promise<XpHistory> {
  return request(`/users/${userId}/xp-history`);
}
