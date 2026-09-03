// Per-fic "Similar fic suggestions" API client.
//
// These endpoints live under /api/fic-suggestions (distinct from the legacy
// /api/recommendations/suggest surface). Creating a suggestion requires auth;
// the panel attaches the JWT via authHeaders() from './social' so votes and
// `my_vote` resolve per-user. Anonymous users can still list and vote by IP.

import { authHeaders } from './social';
import type { FicSuggestion } from './types';

export type { FicSuggestion };
export type { FicSuggestion as FicSuggestionView };

export interface FicSuggestionsListResponse {
  err: number;
  url_id: string;
  suggestions: FicSuggestion[];
  msg?: string;
}

export interface FicSuggestionCreateResponse {
  err: number;
  suggestion_id?: number;
  msg?: string;
}

export interface FicSuggestionVoteResponse {
  err: number;
  suggestion_id?: number;
  new_score?: number;
  my_vote?: number;
  msg?: string;
}

export interface FicSuggestionRemoveResponse {
  err: number;
  suggestion_id?: number;
  removed?: boolean;
  msg?: string;
}

/** GET /api/fic-suggestions?url_id=... — list active suggestions for a fic. */
export async function listFicSuggestions(url_id: string): Promise<FicSuggestionsListResponse> {
  const res = await fetch(`/api/fic-suggestions?url_id=${encodeURIComponent(url_id)}`, {
    credentials: 'include',
  });
  if (!res.ok) {
    return { err: res.status, suggestions: [], url_id, msg: `HTTP ${res.status}` };
  }
  return (await res.json()) as FicSuggestionsListResponse;
}

/** POST /api/fic-suggestions — create a suggestion (auth required). */
export async function createFicSuggestion(
  url_id: string,
  payload: { suggested_url_id?: string; url?: string; comment?: string },
): Promise<FicSuggestionCreateResponse> {
  const res = await fetch('/api/fic-suggestions', {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ url_id, ...payload }),
  });
  if (!res.ok) {
    return { err: res.status, msg: `HTTP ${res.status}` };
  }
  return (await res.json()) as FicSuggestionCreateResponse;
}

/** POST /api/fic-suggestions/{id}/vote — 1 up, -1 down, 0 retract. */
export async function voteFicSuggestion(id: number, vote: 1 | -1 | 0): Promise<FicSuggestionVoteResponse> {
  const res = await fetch(`/api/fic-suggestions/${id}/vote`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ vote }),
  });
  if (!res.ok) {
    return { err: res.status, msg: `HTTP ${res.status}` };
  }
  return (await res.json()) as FicSuggestionVoteResponse;
}

/** POST /api/fic-suggestions/{id}/remove — owner or role>=10 soft-delete. */
export async function removeFicSuggestion(id: number): Promise<FicSuggestionRemoveResponse> {
  const res = await fetch(`/api/fic-suggestions/${id}/remove`, {
    method: 'POST',
    credentials: 'include',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
  });
  if (!res.ok) {
    return { err: res.status, msg: `HTTP ${res.status}` };
  }
  return (await res.json()) as FicSuggestionRemoveResponse;
}
