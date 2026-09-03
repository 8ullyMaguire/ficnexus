// Shared reader logic: state persistence, chapter splitting, Next Up assembly.
// Kept free of Svelte so it can be unit-tested directly with vitest.

export interface ReaderChapter {
  title: string;
  content: string;
  words: number;
}

export interface ReaderWork {
  url_id: string;
  title: string;
  author: string;
  work_id: number | null;
  words: number;
  chapters: ReaderChapter[];
}

export interface ReaderState {
  fontSize?: number;
  lineHeight?: number;
  maxWidth?: number;
  theme?: 'light' | 'sepia' | 'dark';
  indentParagraphs?: boolean;
  chapterIndex?: number;
  scrollPos?: number;
  finished?: boolean;
  lastReadAt?: number;
}

export const STATE_KEY_PREFIX = 'fichub:reader:state:';
export const HTML_CACHE_KEY_PREFIX = 'fichub:reader:html:';

export function stateKey(urlId: string): string {
  return `${STATE_KEY_PREFIX}${urlId}`;
}

/** localStorage key holding the fic's reader HTML bundle (offline fallback). */
export function htmlCacheKey(urlId: string): string {
  return `${HTML_CACHE_KEY_PREFIX}${urlId}`;
}

/** Store the raw reader API JSON so the fic re-renders offline. */
export function saveReaderHtml(urlId: string, payload: ReaderApiResponse): void {
  try {
    localStorage.setItem(htmlCacheKey(urlId), JSON.stringify(payload));
  } catch {
    /* storage full/blocked — the SW cache may still cover offline */
  }
}

/** Safely read the last-cached reader payload (null on any failure). */
export function tryCachedReaderHtml(urlId: string): ReaderApiResponse | null {
  try {
    const raw = localStorage.getItem(htmlCacheKey(urlId));
    if (!raw) return null;
    const parsed = JSON.parse(raw) as ReaderApiResponse;
    return parsed && typeof parsed === 'object' && typeof parsed.html === 'string' ? parsed : null;
  } catch {
    return null;
  }
}

/** Safely read a reader state blob from localStorage (returns {} on any failure). */
export function restoreReaderState(urlId: string): ReaderState {
  try {
    const raw = localStorage.getItem(stateKey(urlId));
    if (!raw) return {};
    const parsed = JSON.parse(raw) as ReaderState;
    return typeof parsed === 'object' && parsed !== null ? parsed : {};
  } catch {
    return {};
  }
}

/** Persist reader state (swallows quota/storage errors). */
export function saveReaderState(urlId: string, state: ReaderState): void {
  try {
    localStorage.setItem(stateKey(urlId), JSON.stringify(state));
  } catch {
    /* storage full/blocked — non-fatal */
  }
}

/** Response contract of GET /api/reader/{url_id} (backend: src/routes/reader.rs). */
export interface ReaderApiResponse {
  err: number;
  msg?: string;
  url_id: string;
  title: string;
  author: string;
  work_id: number | null;
  html: string;
  words: number;
  chapters: number;
  /** Best-effort EPUB download link (backend includes it when a cached EPUB exists). */
  epub_url?: string | null;
}

/** Result of loadReaderData when the reader API has no readable content. */
export class NoReaderContentError extends Error {
  epubUrl: string | null;
  constructor(message: string, epubUrl: string | null) {
    super(message);
    this.name = 'NoReaderContentError';
    this.epubUrl = epubUrl;
  }
}

export interface ReaderLoadResult {
  work: ReaderWork;
  /** true when rendered from the localStorage HTML cache (network failed). */
  fromCache: boolean;
}

/** Turn a raw reader API payload into a ReaderWork (shared by fetch + cache paths). */
export function readerWorkFromPayload(json: ReaderApiResponse, urlId: string): ReaderWork {
  const chapters = splitChapters(json.html);
  if (chapters.length === 0) {
    // No per-chapter headings — fall back to a single "chapter" with the
    // whole body so the reader still works.
    chapters.push({
      title: json.title,
      content: `<div class="chapter">${stripTags(json.html)}</div>`,
      words: countWords(json.html),
    });
  }
  return {
    url_id: json.url_id || urlId,
    title: json.title,
    author: json.author,
    work_id: json.work_id ?? null,
    words: json.words ?? 0,
    chapters,
  };
}

/**
 * Fetch the full fic HTML from the reader API and split it into chapters.
 * Throws on network failure, non-OK responses, and err != 0 — unless a
 * previously cached reader payload exists in localStorage, in which case the
 * cached HTML is returned so previously-opened fics keep reading offline.
 */
export async function loadReaderData(urlId: string): Promise<ReaderLoadResult> {
  let epubUrl: string | null = null;
  try {
    const res = await fetch(`/api/reader/${encodeURIComponent(urlId)}`);
    if (!res.ok) {
      // The error envelope may carry an EPUB fallback link (err:-5 responses
      // include epub_url when a cached EPUB exists).
      try {
        const errJson = (await res.json()) as { epub_url?: string | null };
        epubUrl = errJson.epub_url ?? null;
      } catch { /* no JSON body — ignore */ }
      throw new NoReaderContentError(`Failed to load reader (HTTP ${res.status})`, epubUrl);
    }
    const json = (await res.json()) as ReaderApiResponse;
    if (json.err !== 0) {
      throw new NoReaderContentError(json.msg || 'Failed to load reader', json.epub_url ?? null);
    }
    saveReaderHtml(urlId, json);
    return { work: readerWorkFromPayload(json, urlId), fromCache: false };
  } catch (err) {
    // Offline / server error — fall back to the last-cached reader HTML so
    // previously opened fics render without a network connection.
    const cached = tryCachedReaderHtml(urlId);
    if (cached) return { work: readerWorkFromPayload(cached, urlId), fromCache: true };
    throw err;
  }
}

export interface ChapterHeading {
  title: string;
  start: number; // index into the h2 headings (0-based)
}

/**
 * Split a fic's full HTML (as produced by the html bundle: one `<h2 id="chN">`
 * per chapter) into per-chapter fragments. Returns [] if no chapter headings
 * are found. The returned fragments are wrapped in <div class="chapter"> so
 * each can be rendered independently.
 */
export function splitChapters(html: string, titles: string[] = []): ReaderChapter[] {
  const h2Re = /<h2\b[^>]*id="ch(\d+)"[^>]*>(.*?)<\/h2>/gi;
  const headings: { id: number; title: string; h2Start: number; bodyStart: number }[] = [];
  let match: RegExpExecArray | null;
  while ((match = h2Re.exec(html)) !== null) {
    const id = parseInt(match[1], 10);
    const rawTitle = match[2].replace(/<[^>]*>/g, '').trim();
    const title = rawTitle || titles[id - 1] || `Chapter ${id}`;
    headings.push({
      id,
      title,
      h2Start: match.index,
      bodyStart: h2Re.lastIndex, // just past the closing </h2>
    });
  }
  if (headings.length === 0) return [];

  // Body of chapter N runs from the end of its </h2> to the start of the next
  // chapter's <h2> tag (or the end of the document for the last chapter).
  const chapters: ReaderChapter[] = [];
  for (let i = 0; i < headings.length; i++) {
    const start = headings[i].bodyStart;
    const end = i + 1 < headings.length ? headings[i + 1].h2Start : html.length;
    const body = html.slice(start, end).trim();
    const words = countWords(stripTags(body));
    chapters.push({
      title: headings[i].title,
      content: `<div class="chapter">${body}</div>`,
      words,
    });
  }
  return chapters;
}

export function stripTags(html: string): string {
  return html
    .replace(/<script[\s\S]*?<\/script>/gi, ' ')
    .replace(/<style[\s\S]*?<\/style>/gi, ' ')
    .replace(/<[^>]*>/g, ' ');
}

export function countWords(text: string): number {
  const t = text
    .replace(/<[^>]*>/g, ' ')
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&[a-z]+;/gi, ' ');
  const m = t.match(/[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*/gu);
  return m ? m.length : 0;
}

export interface NextUpCandidate {
  url_id: string;
  title: string;
  author: string;
  reason: string;
}

/**
 * Assemble the end-of-fic "Next Up" candidate list from raw sources.
 * Order: sequel (next in series) → top community suggestion → readers-also-bookmarked.
 * Dedupes by url_id, keeps at most `limit`.
 */
export function assembleNextUp(
  sequel: { url_id: string; title: string; author: string } | null,
  communityTop: { url_id: string; title: string; author: string } | null,
  alsoBookmarked: { url_id: string; title: string; author: string }[],
  limit = 3
): NextUpCandidate[] {
  const out: NextUpCandidate[] = [];
  const seen = new Set<string>();
  const push = (c: { url_id: string; title: string; author: string }, reason: string) => {
    if (!c?.url_id || seen.has(c.url_id)) return;
    seen.add(c.url_id);
    out.push({ ...c, reason });
  };
  if (sequel) push(sequel, 'Next in series');
  if (communityTop) push(communityTop, 'Top community suggestion');
  for (const c of alsoBookmarked ?? []) push(c, 'Readers also bookmarked');
  return out.slice(0, limit);
}

/** Heuristic sequel detection on two titles (same author assumed). */
export function looksLikeSequel(prevTitle: string, candidateTitle: string): boolean {
  const t1 = normalizeTitle(prevTitle);
  const t2 = normalizeTitle(candidateTitle);
  if (t1 === t2) return false;
  // Candidate contains a numeral, roman numeral, or ordinal that the previous lacks.
  const numeralRe = /(\b(?:2|3|4|5|6|7|8|9|10|11|12)\b|\b(?:II|III|IV|V|VI|VII|VIII|IX|X)\b|(?:2nd|3rd|4th|5th|6th|7th|8th|9th|10th)\b)/i;
  return numeralRe.test(t2) && !numeralRe.test(t1);
}

function normalizeTitle(title: string): string {
  return (title || '').toLowerCase().replace(/[^a-z0-9]+/g, ' ').trim();
}
