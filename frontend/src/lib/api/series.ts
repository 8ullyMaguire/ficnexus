// API client for the series & author bibliography pages.
// Backend: src/routes/series.rs (routes registered in src/server.rs).

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    credentials: 'include',
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as T;
}

// ── Types (match src/routes/series.rs JSON responses) ─────────────────────

/** A work inside a series, with its position and next-in-series link. */
export interface SeriesWork {
  work_id: number;
  url_id: string;
  canonical_title: string;
  canonical_author: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  next_in_series: {
    work_id: number;
    canonical_title: string;
    url_id: string;
  } | null;
}

export interface SeriesDetail {
  id: number;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
  work_count: number;
}

export interface SeriesResponse {
  err: number;
  series: SeriesDetail;
  works: SeriesWork[];
}

/** A canonical work in an author's bibliography. */
export interface AuthorWork {
  work_id: number;
  canonical_title: string;
  description: string;
  canonical_author: string;
  url_id: string;
  words: number;
  chapters: number;
  status: string;
}

/** An orphan source (fic_info with no canonical work yet) by the author. */
export interface AuthorOrphan {
  work_id: null;
  url_id: string;
  canonical_title: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
}

export interface AuthorDetail {
  name: string;
  id?: number;
  avatar_url?: string | null;
  bio?: string | null;
  badge_text?: string | null;
  /** Earned badges (badge_type, name, icon, earned_at). May be empty on
   *  bibliography endpoints that don't surface badge data. */
  badges?: {
    badge_type: string;
    name: string;
    icon: string;
    earned_at: string;
  }[];
  /** Tags this author/user has favorited (AO3-style favourite tags). */
  favorite_tags?: { name: string; tag_type_id: number }[];
  /** Social links (AO3-style external links: Twitter, Tumblr, personal site, etc.). */
  socials?: { id: number; platform: string; url: string; label?: string }[];
  work_count: number;
  total_words: number;
  top_tags: { name: string; tag_type_id: number; usage_count: number }[];
}

export interface AuthorResponse {
  err: number;
  author: AuthorDetail;
  works: AuthorWork[];
  orphans: AuthorOrphan[];
}

// ── Public routes ──────────────────────────────────────────────────────────

/** GET /api/series/{id} — series detail with ordered works. */
export async function getSeries(id: number): Promise<SeriesResponse> {
  return request<SeriesResponse>(`/series/${id}`);
}

/** GET /api/authors/by-name/{name} — author bibliography (canonical name keyed). */
export async function getAuthorByName(name: string): Promise<AuthorResponse> {
  return request<AuthorResponse>(`/authors/by-name/${encodeURIComponent(name)}`);
}

/** PUT /api/authors/{id} — update bio, avatar_url, badge_text (curator+). */
export async function updateAuthorProfile(
  authorId: number,
  updates: { avatar_url?: string; bio?: string; badge_text?: string },
): Promise<{ err: number; msg?: string }> {
  return request<{ err: number; msg?: string }>(`/authors/${authorId}`, {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}
