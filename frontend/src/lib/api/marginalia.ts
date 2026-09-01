// Marginalia API — GET /api/reader/{urlId}/marginalia?chapter={n}
// Auth optional. Returns [{passage_hash, topic_id, post_count}]
import { authHeaders } from './social';

export interface MarginaliaEntry {
  passage_hash: string;
  topic_id: number;
  topic_slug?: string | null;
  post_count: number;
}

export interface MarginaliaResponse {
  err: number;
  marginalia?: MarginaliaEntry[];
  msg?: string;
}

export async function getMarginalia(urlId: string, chapter: number): Promise<MarginaliaResponse> {
  const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}/marginalia?chapter=${chapter}`, {
    credentials: 'include',
    headers: { ...authHeaders() },
  });
  if (!res.ok) {
    // Auth optional — treat 401/404 as empty
    if (res.status === 401 || res.status === 404) return { err: 0, marginalia: [] };
    throw new Error(`marginalia fetch failed: ${res.status}`);
  }
  return (await res.json()) as MarginaliaResponse;
}

/** Check show_marginalia pref via /api/me/prefs (defaults to true when absent). */
export async function getShowMarginalia(): Promise<boolean> {
  try {
    const res = await fetch('/api/me/prefs', { credentials: 'include', headers: { ...authHeaders() } });
    if (!res.ok) return true;
    const json = await res.json() as { prefs?: Array<{ key: string; value: string }> } & { err?: number };
    const prefs = (json as unknown as { prefs?: unknown[] })?.prefs;
    if (Array.isArray(prefs)) {
      const entry = prefs.find((p: unknown) => (p as { key: string }).key === 'show_marginalia') as { value: string } | undefined;
      if (entry) {
        try { return JSON.parse(entry.value) !== false; } catch { return entry.value !== 'false'; }
      }
    }
    // Alternative shape: { show_marginalia: true }
    const alt = json as unknown as Record<string, unknown>;
    if (typeof alt.show_marginalia === 'boolean') return alt.show_marginalia;
    if (typeof alt.show_marginalia === 'string') return alt.show_marginalia !== 'false';
    return true;
  } catch { return true; }
}

export interface MarginaliaPayload {
  type: 'marginalia';
  work_id: number | null;
  url_id: string;
  chapter_index: number;
  passage_hash: string;
  passage_text: string;
}
