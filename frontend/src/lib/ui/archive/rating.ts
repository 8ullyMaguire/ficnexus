/**
 * AO3-style rating and tag helpers for the archive UI.
 */

const RATING_MAP: Record<string, string> = {
  general: 'General Audiences',
  teen: 'Teen And Up Audiences',
  mature: 'Mature',
  explicit: 'Explicit',
};

const DEFAULT_RATING = 'Not Rated';

/**
 * Map a rating code to an AO3 display name.
 * Returns "Not Rated" for null, undefined, or unknown codes.
 */
export function mapRating(rating: string | null | undefined): string {
  if (!rating) return DEFAULT_RATING;
  return RATING_MAP[rating.toLowerCase()] ?? DEFAULT_RATING;
}

const CATEGORY_LABELS: Record<string, string> = {
  '1': 'Fandoms',
  '2': 'Characters',
  '3': 'Relationships',
  '4': 'Additional Tags',
  '5': 'Warnings',
  '6': 'Categories',
};

const CATEGORY_ORDER = ['1', '2', '6', '3', '4', '5'];

interface Tag {
  category?: number | string;
  name: string;
  [key: string]: unknown;
}

/**
 * Group tags by their numeric category into a Record.
 * Returns an object keyed by category number string, with a human-readable
 * label stored on each entry as a `__label` property for display.
 */
export function groupTags(tags: Tag[]): Record<string, Tag[]> {
  const result: Record<string, Tag[]> = {};

  for (const tag of tags) {
    const key = String(tag.category ?? '4');
    if (!result[key]) result[key] = [];
    result[key].push(tag);
  }

  // Ensure all known categories exist in output
  for (const cat of CATEGORY_ORDER) {
    if (!result[cat]) result[cat] = [];
  }

  return result;
}

/**
 * Return the display label for a tag category.
 */
export function categoryLabel(category: string): string {
  return CATEGORY_LABELS[category] ?? 'Tags';
}

/**
 * Return the ordered category keys for display (subset that actually have tags).
 */
export function orderedCategories(grouped: Record<string, unknown[]>): string[] {
  return CATEGORY_ORDER.filter((c) => grouped[c]?.length);
}

/**
 * Comma-format a number (e.g. 1204116 → "1,204,116").
 */
export function formatWords(words: number | null | undefined): string {
  if (words == null || isNaN(words)) return '0';
  return words.toLocaleString('en-US');
}

/**
 * Build a chapters display string: "n/n" if complete, "n/?" otherwise.
 */
export function chaptersDisplay(
  chapters: number | null | undefined,
  status: string | null | undefined
): string {
  const current = chapters ?? 0;
  const complete = status?.toLowerCase() === 'complete' || status?.toLowerCase() === 'completed';
  return complete ? `${current}/${current}` : `${current}/?`;
}

/**
 * Format an "updated" date string or timestamp into a relative time.
 * e.g. "2024-06-10T12:00:00Z" → "3d ago"
 */
export function formatUpdated(updated: string | null | undefined): string {
  if (!updated) return '';
  const date = new Date(updated);
  if (isNaN(date.getTime())) return '';
  const now = Date.now();
  const diff = now - date.getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  const years = Math.floor(months / 12);
  return `${years}y ago`;
}

/**
 * Format a "visited" date into a short relative time (e.g. "3d ago").
 * Uses the same cadence as formatUpdated so visited stamps are consistent
 * with the blurb's "Updated:" line.
 */
export function relativeTime(updated: string | null | undefined): string {
  if (!updated) return '';
  const date = new Date(updated);
  if (isNaN(date.getTime())) return '';
  const now = Date.now();
  const diff = now - date.getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  const months = Math.floor(days / 30);
  if (months < 12) return `${months}mo ago`;
  const years = Math.floor(months / 12);
  return `${years}y ago`;
}


/**
 * Format estimated reading time from the backend's dialogue-aware estimate.
 * Falls back to a word-count heuristic when minutes are absent or zero.
 * Rounds up to the nearest 5-minute increment for a clean display.
 */
export function formatReadingTime(estMinutes: number | undefined, words: number): string {
  let mins: number;
  if (estMinutes && estMinutes > 0) {
    mins = estMinutes;
  } else {
    // Fallback: 240 wpm (conservative dense-prose rate).
    mins = Math.ceil(words / 240);
  }
  // Round up to nearest 5 min for cleaner display.
  mins = Math.ceil(mins / 5) * 5;
  if (mins < 60) {
    return `${mins} min`;
  }
  const hours = Math.floor(mins / 60);
  const rem = mins % 60;
  return rem === 0 ? `${hours} hr` : `${hours} hr ${rem} min`;
}
