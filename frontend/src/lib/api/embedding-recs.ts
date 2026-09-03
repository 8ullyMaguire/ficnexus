// Embeddings-based recommendations API client.
// Backend: src/recommender/embedding_recs.rs — GET /api/recommendations/embeddings
//
// Auth: the backend reads the JWT from the `Authorization: Bearer` header
// only (AuthUser extractor), so authHeaders() is attached when a token is
// present. Anonymous callers get the same 200 empty shape (enough_data:
// false) and the section stays hidden.

import type { RecResult } from './types';
import { authHeaders } from './social';

const BASE = '/api';

export interface EmbeddingRecsResponse {
  err: number;
  enough_data: boolean;
  recs: RecResult[];
  based_on: Array<{ title: string; url_id: string | null }>;
  source?: string;
}

export async function fetchEmbeddingRecs(n = 6): Promise<EmbeddingRecsResponse> {
  const res = await fetch(`${BASE}/recommendations/embeddings?n=${n}`, {
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
    },
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return (await res.json()) as EmbeddingRecsResponse;
}
