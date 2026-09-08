// API client for site-wide drafts (Lane 1 — Phase 4).
//
// Drafts are keyed by (context, ref):
//   context=forum_post  ref={topicId}  → reply draft
//   context=forum_topic ref=new:{slug} → new-topic draft
//
// PUT /api/drafts/{context}/{ref}  → upsert (idempotent)
// DELETE /api/drafts/{context}/{ref} → delete
// GET /api/drafts?context=           → list own drafts

import { authHeaders } from './social';

export interface Draft {
  id: number;
  user_id: number;
  topic_id: number | null;
  category_id: number | null;
  title: string | null;
  body: string;
  payload: Record<string, unknown>;
  poll_data: Record<string, unknown> | null;
  created_at: string;
  updated_at: string;
}

/** Save (upsert) a draft. Returns `{err: 0}` on success. */
export async function saveDraft(
  context: string,
  ref: string,
  data: { title?: string; body?: string },
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/drafts/${context}/${ref}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
    },
    body: JSON.stringify(data),
  });
  return res.json();
}

/** Delete a draft. Returns `{err: 0}` on success. */
export async function deleteDraft(
  context: string,
  ref: string,
): Promise<{ err: number; msg?: string }> {
  const res = await fetch(`/api/drafts/${context}/${ref}`, {
    method: 'DELETE',
    headers: authHeaders(),
  });
  return res.json();
}

/** List own drafts, optionally filtered by context. */
export async function listDrafts(
  context?: string,
): Promise<{ err: number; items?: Draft[]; msg?: string }> {
  const q = context ? `?context=${encodeURIComponent(context)}` : '';
  const res = await fetch(`/api/drafts${q}`, {
    headers: authHeaders(),
  });
  return res.json();
}
