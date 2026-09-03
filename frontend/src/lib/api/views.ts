// API client for custom saved views (GET /api/me/views, POST, DELETE, toggle-pin).
// Backend: src/routes/progression.rs

import { authHeaders } from './social';

/** A saved search view. */
export interface SavedView {
  id: number;
  name: string;
  query: Record<string, unknown>;
  pinned: boolean;
}

/** Fetch all saved views for the current user. */
export async function fetchViews(): Promise<SavedView[]> {
  const res = await fetch('/api/me/views', {
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to fetch views (${res.status})`);
  return (await res.json()) as SavedView[];
}

/** Create a new saved view. */
export async function createView(
  name: string,
  query: Record<string, unknown>,
  pinned = false,
): Promise<SavedView> {
  const res = await fetch('/api/me/views', {
    method: 'POST',
    headers: { ...authHeaders(), 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ name, query, pinned }),
  });
  if (!res.ok) throw new Error(`Failed to create view (${res.status})`);
  return (await res.json()) as SavedView;
}

/** Delete a saved view by id. */
export async function deleteView(id: number): Promise<void> {
  const res = await fetch(`/api/me/views/${id}`, {
    method: 'DELETE',
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to delete view (${res.status})`);
}

/** Toggle the pinned state of a view. */
export async function togglePin(id: number): Promise<SavedView> {
  const res = await fetch(`/api/me/views/${id}/toggle-pin`, {
    method: 'PUT',
    headers: authHeaders(),
    credentials: 'include',
  });
  if (!res.ok) throw new Error(`Failed to toggle pin (${res.status})`);
  return (await res.json()) as SavedView;
}
