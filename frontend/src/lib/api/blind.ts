// Blind Date with a Fic — API client for GET /api/blind-date
// Backend: src/routes/blind.rs
//
// Returns a random eligible fic with the title & fandom hidden: only a
// spoiler-free description, word/chapter count, status and top-3 core
// tropes. Pass `exclude` (url_ids seen already) so the UI avoids repeats.
// The `reveal` object carries a signed nonce used to fetch the title +
// fandom on demand from `/api/blind-date/reveal` — the discovery payload
// never contains the hidden fields.

export interface BlindDateReveal {
  nonce: string;
  sig: string;
}

export interface BlindDateFic {
  url_id: string;
  description: string;
  words: number;
  chapters: number;
  status: string;
  tropes: string[];
  reveal?: BlindDateReveal;
}

export interface BlindDateResponse {
  err: number;
  /** null when the archive has nothing eligible (all excluded / empty). */
  fic: BlindDateFic | null;
}

export interface BlindDateRevealResponse {
  err: number;
  fic: {
    url_id: string;
    title: string;
    author: string;
    source: string;
    fandom: string | null;
  };
}

/** GET /api/blind-date?exclude=a,b,c — a random fic, title hidden. */
export async function getBlindDate(exclude: string[] = []): Promise<BlindDateResponse> {
  const qs = exclude.length > 0 ? `?exclude=${encodeURIComponent(exclude.join(','))}` : '';
  const res = await fetch(`/api/blind-date${qs}`, { credentials: 'include' });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return (await res.json()) as BlindDateResponse;
}

/** GET /api/blind-date/reveal?url_id=...&nonce=...&sig=... — the hidden fields. */
export async function revealBlindDate(
  urlId: string,
  reveal: BlindDateReveal,
): Promise<BlindDateRevealResponse> {
  const qs = new URLSearchParams({
    url_id: urlId,
    nonce: reveal.nonce,
    sig: reveal.sig,
  }).toString();
  const res = await fetch(`/api/blind-date/reveal?${qs}`, { credentials: 'include' });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return (await res.json()) as BlindDateRevealResponse;
}
