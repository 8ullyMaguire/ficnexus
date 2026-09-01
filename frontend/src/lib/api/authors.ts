// API client for author profiles, social links, and curator merge proposals.
// Backend: src/routes/authors.rs (routes registered in src/server.rs)

const BASE = '/api';
const TOKEN_KEY = 'fichub_token';

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

function authHeaders(): Record<string, string> {
  const token = getToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    credentials: 'include',
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

// ── Types (match src/routes/authors.rs JSON responses) ─────────────────────

/** A single row from GET /api/authors/search. */
export interface AuthorSummary {
  id: number;
  canonical_name: string;
  bio: string | null;
  linked_authors: number;
}

/** A social link attached to an author profile. */
export interface SocialLink {
  id: number;
  platform: string;
  url: string;
  label: string | null;
  is_visible: boolean;
}

/** A linked source author (AO3/FFN pseud etc.) pointing at a profile. */
export interface LinkedAuthor {
  id: number;
  source_author: string;
  source_url: string;
  source_id: number | null;
}

/** Full author profile returned by GET /api/authors/{id}. */
export interface AuthorProfile {
  id: number;
  canonical_name: string;
  bio: string | null;
  avatar_url: string | null;
  created_at: string;
  socials: SocialLink[];
  linked_authors: LinkedAuthor[];
}

/** A pending merge proposal from GET /api/curator/authors/pending. */
export interface MergeProposal {
  id: number;
  source_author: string;
  source_url: string;
  target_profile_id: number;
  target_name: string;
  proposed_by: string | null;
  created_at: string;
}

export interface AuthorSearchResponse {
  err: number;
  items: AuthorSummary[];
  page: number;
}

export interface AuthorProfileResponse {
  err: number;
  profile: AuthorProfile;
  linked_authors: LinkedAuthor[];
  socials: SocialLink[];
}

export interface PendingMergesResponse {
  err: number;
  items: MergeProposal[];
}

export interface SimpleResponse {
  err: number;
  msg: string;
}

// ── Public routes ───────────────────────────────────────────────────────────

/** Build the path for GET /api/authors/search (BASE is prepended by request()). */
export function buildAuthorSearchUrl(q: string): string {
  const params = new URLSearchParams();
  if (q) params.set('q', q);
  const qs = params.toString();
  return `/authors/search${qs ? `?${qs}` : ''}`;
}

/** GET /api/authors/search?q= — search author profiles by name/pseud. */
export async function searchAuthors(q: string): Promise<AuthorSummary[]> {
  const res = await request<AuthorSearchResponse>(buildAuthorSearchUrl(q));
  return res.items ?? [];
}

/** GET /api/authors/{id} — full profile with socials and linked accounts. */
export async function getAuthor(id: number): Promise<AuthorProfile> {
  const res = await request<AuthorProfileResponse>(`/authors/${id}`);
  return {
    ...res.profile,
    socials: res.socials ?? [],
    linked_authors: res.linked_authors ?? [],
  };
}

/** PUT /api/authors/{id} — update bio / avatar (curator+, role >= 5). */
export async function updateAuthor(
  id: number,
  payload: { bio?: string; avatar_url?: string },
): Promise<SimpleResponse> {
  return request<SimpleResponse>(`/authors/${id}`, {
    method: 'PUT',
    body: JSON.stringify(payload),
  });
}

/** POST /api/authors/{id}/socials — add a social link (curator+). */
export async function addSocial(
  id: number,
  payload: { platform: string; url: string; label?: string },
): Promise<SimpleResponse> {
  return request<SimpleResponse>(`/authors/${id}/socials`, {
    method: 'POST',
    body: JSON.stringify(payload),
  });
}

/** DELETE /api/authors/{id}/socials/{social_id} — remove a social link (curator+). */
export async function removeSocial(id: number, socialId: number): Promise<SimpleResponse> {
  return request<SimpleResponse>(`/authors/${id}/socials/${socialId}`, {
    method: 'DELETE',
  });
}

// ── Curator routes ──────────────────────────────────────────────────────────

/** POST /api/curator/authors/merge — propose (or admin-auto-approve) a merge. */
export async function proposeMerge(
  source_author: string,
  source_url: string,
  target_profile_id: number,
  options?: { auto_approve?: boolean; source_id?: number },
): Promise<SimpleResponse> {
  return request<SimpleResponse>('/curator/authors/merge', {
    method: 'POST',
    body: JSON.stringify({
      source_author,
      source_url,
      target_profile_id,
      ...(options?.auto_approve !== undefined ? { auto_approve: options.auto_approve } : {}),
      ...(options?.source_id !== undefined ? { source_id: options.source_id } : {}),
    }),
  });
}

/** GET /api/curator/authors/pending — list pending merge proposals. */
export async function getPendingMerges(): Promise<MergeProposal[]> {
  const res = await request<PendingMergesResponse>('/curator/authors/pending');
  return res.items ?? [];
}

/** POST /api/curator/authors/approve/{proposal_id} — approve a merge. */
export async function approveMerge(proposalId: number): Promise<SimpleResponse> {
  return request<SimpleResponse>(`/curator/authors/approve/${proposalId}`, {
    method: 'POST',
  });
}

/** POST /api/curator/authors/reject/{proposal_id} — reject a merge. */
export async function rejectMerge(proposalId: number): Promise<SimpleResponse> {
  return request<SimpleResponse>(`/curator/authors/reject/${proposalId}`, {
    method: 'POST',
  });
}
