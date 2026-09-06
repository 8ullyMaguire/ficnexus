// Small formatting helpers shared across components.

/** Format a word count into a human string (1234567 -> "1,234,567"). */
export function formatWords(words: number): string {
  return words.toLocaleString('en-US');
}

/** Format an ISO timestamp into a short date (2024-01-15). */
export function formatDate(iso: string): string {
  if (!iso) return '';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return '';
  return d.toISOString().slice(0, 10);
}

/** Relative time from an ISO timestamp (e.g. "3 days ago"). */
export function relativeTime(iso: string): string {
  if (!iso) return '';
  const then = new Date(iso).getTime();
  if (isNaN(then)) return '';
  const diff = Date.now() - then;
  const sec = Math.floor(diff / 1000);
  if (sec < 60) return 'less than a minute ago';
  const min = Math.floor(sec / 60);
  if (min < 60) return `${min} minute${min > 1 ? 's' : ''} ago`;
  const hr = Math.floor(min / 60);
  if (hr < 24) return `${hr} hour${hr > 1 ? 's' : ''} ago`;
  const day = Math.floor(hr / 24);
  return `${day} day${day > 1 ? 's' : ''} ago`;
}

/** Detect the fanfiction site from a URL. */
export function detectSite(url: string): string {
  if (url.includes('archiveofourown.org')) return 'AO3';
  if (url.includes('fanfiction.net')) return 'FanFiction.net';
  if (url.includes('fictionpress.com')) return 'FictionPress';
  if (url.includes('forums.spacebattles.com')) return 'SpaceBattles';
  if (url.includes('forums.sufficientvelocity.com')) return 'SufficientVelocity';
  if (url.includes('forum.questionablequesting.com')) return 'QuestionableQuesting';
  if (url.includes('royalroad.com')) return 'RoyalRoad';
  return 'Unknown';
}

/** Strip HTML tags from a string (for summaries). */
export function stripHtml(html: string): string {
  if (!html) return '';
  return html
    .replace(/<br\s*\/?>(?=)/gi, ' ')
    .replace(/<\/(p|div)>/gi, ' ')
    .replace(/<[^>]+>/g, '')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, ' ')
    .trim();
}

/** Build a download URL that goes through the backend cache route. */
export function cacheUrl(etype: string, urlId: string, hash: string): string {
  return `/cache/${etype}/${urlId}?h=${hash}`;
}

/** Returns true if the input looks like a fanfiction URL (starts with http(s)://). */
export function isFicUrl(v: string): boolean {
  if (!v) return false;
  return /^https?:\/\//i.test(v.trim());
}
