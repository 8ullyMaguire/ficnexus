// API client for saved searches + daily alerts.
// Backend: src/routes/saved_search.rs  (tables from migration 003).

import { authHeaders } from './social';

/** A saved search (as returned by GET /api/search/saved). */
export interface SavedSearch {
  id: number;
  name: string;
  query_text: string;
  alert_mode: 'none' | 'rss';
  last_run_at: string | null;
  last_match_count: number | null;
  created_at: string;
}

/** The per-search public Atom feed URL (no auth required). */
export function savedSearchFeedUrl(userId: number, searchId: number): string {
  return `/feed/saved/${userId}/${searchId}.xml`;
}

/** Fetch all saved searches for the current user. */
export async function fetchSavedSearches(): Promise<SavedSearch[]> {
  const res = await fetch('/api/search/saved', {
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to fetch saved searches (${res.status})`);
  const data = (await res.json()) as { saved_searches?: SavedSearch[] };
  return data.saved_searches ?? [];
}

/** Create a new saved search from a name + query text. */
export async function createSavedSearch(
  name: string,
  queryText: string,
): Promise<SavedSearch> {
  const res = await fetch('/api/search/saved', {
    method: 'POST',
    headers: { ...authHeaders(), 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ name, query_text: queryText }),
  });
  if (!res.ok) {
    const data = (await res.json().catch(() => null)) as { msg?: string } | null;
    throw new Error(data?.msg ?? `Failed to save search (${res.status})`);
  }
  const data = (await res.json()) as { saved_search: SavedSearch };
  return data.saved_search;
}

/** Delete a saved search by id. */
export async function deleteSavedSearch(id: number): Promise<void> {
  const res = await fetch(`/api/search/saved/${id}`, {
    method: 'DELETE',
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to delete saved search (${res.status})`);
}

/** Set the alert mode ('none' | 'rss') for a saved search. */
export async function setSavedSearchAlert(
  id: number,
  alertMode: 'none' | 'rss',
): Promise<void> {
  const res = await fetch(`/api/search/saved/${id}/alert`, {
    method: 'PUT',
    headers: { ...authHeaders(), 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ alert_mode: alertMode }),
  });
  if (!res.ok) {
    const data = (await res.json().catch(() => null)) as { msg?: string } | null;
    throw new Error(data?.msg ?? `Failed to update alert mode (${res.status})`);
  }
}

/** The shape returned by POST /api/search/saved/{id}/run (matches /api/search). */
export interface SavedSearchRunResult {
  saved_search_id: number;
  name: string;
  total: number;
  page: number;
  per_page: number;
  results: unknown[];
}

/** Re-run a saved search and return its current results. */
export async function runSavedSearch(id: number): Promise<SavedSearchRunResult> {
  const res = await fetch(`/api/search/saved/${id}/run`, {
    method: 'POST',
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to run saved search (${res.status})`);
  return (await res.json()) as SavedSearchRunResult;
}
