// Fandom API helpers.

export interface Fandom {
  slug: string;
  name: string;
  fic_count: number;
  total_words: number;
}

export interface FandomDetail extends Fandom {
  active_authors: number;
}

export interface FandomTopFic {
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  score: number;
}

export interface FandomRecentFic {
  url_id: string;
  title: string;
  author: string;
  updated_at: string;
}

export interface FandomsResponse {
  err: number;
  fandoms: Fandom[];
}

export interface FandomDetailResponse {
  err: number;
  fandom: FandomDetail;
  top_fics: FandomTopFic[];
  recently_updated: FandomRecentFic[];
}

export async function getFandoms(): Promise<FandomsResponse> {
  const res = await fetch('/api/fandoms');
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json();
}

export async function getFandom(slug: string): Promise<FandomDetailResponse> {
  const res = await fetch(`/api/fandoms/${encodeURIComponent(slug)}`);
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json();
}
