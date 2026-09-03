/**
 * AO3-style symbol squares: pure helpers that derive the four AO3
 * "required tags" glyph/marker squares shown on Archive of Our Own work
 * blurbs.
 *
 * Each square corresponds to one of AO3's four required tag categories:
 *
 *  1. Rating    — a single-letter code (T / M / E / G)
 *  2. Category  — a short text code (M / F / FM / MM)
 *  3. Warning   — a glyph (e.g. "!", "✓", or a question mark)
 *  4. Status    — a completion glyph ("✓" complete / "✗" in-progress)
 *
 * Per FicHub project rules, the ONLY permitted emoji glyph is the
 * international-globe 🌐 (used for "no warnings / no category" placeholders).
 * No other emoji are used in these helpers or the UI they feed.
 */

// ── Rating ─────────────────────────────────────────────────────────────

/**
 * AO3 rating codes, in the order AO3 displays them.
 * These are the raw scraper values stored on `fic.rating`.
 */
export const AO3_RATINGS = ['general', 'teen', 'mature', 'explicit'] as const;
export type AO3Rating = (typeof AO3_RATINGS)[number];

/** Single-letter code displayed inside the rating square. */
export const AO3_RATING_LETTERS: Record<string, string> = {
  general: 'G',
  teen: 'T',
  mature: 'M',
  explicit: 'E',
};

/**
 * Map a raw rating code (or the human display name from `mapRating`)
 * to an AO3-style single-letter square value.
 *
 * - 'general' | 'General Audiences' → 'G'
 * - 'teen'    | 'Teen And Up Audiences' → 'T'
 * - 'mature'  | 'Mature' → 'M'
 * - 'explicit'| 'Explicit' → 'E'
 * - null / undefined / unknown → '-'
 */
export function ratingSquare(rating: string | null | undefined): string {
  if (!rating) return '-';
  const key = rating.toLowerCase();
  if (AO3_RATING_LETTERS[key]) return AO3_RATING_LETTERS[key];
  // Accept the human-readable AO3 names too (e.g. "General Audiences").
  for (const [code, letter] of Object.entries(AO3_RATING_LETTERS)) {
    if (key === code) return letter;
  }
  return '-';
}

// ── Category ───────────────────────────────────────────────────────────

/**
 * AO3 romance / category tags and the short text code each renders as
 * inside its square. AO3 work search uses these canonical tag names.
 */
export const AO3_CATEGORY_CODES: Record<string, string> = {
  // Male/Female pairings
  'F/M': 'F/M',
  'male/female': 'F/M',
  // Female/Female pairings
  'F/F': 'F/F',
  'female/female': 'F/F',
  // Male/Male pairings
  'M/M': 'M/M',
  'male/male': 'M/M',
  // Gen (no romantic pairing)
  'Gen': 'Gen',
  'gen': 'Gen',
  // Multi
  'Multi': 'Multi',
  'multi': 'Multi',
};

export const AO3_CATEGORY_ORDER = ['Gen', 'F/M', 'F/F', 'M/M', 'Multi'];

/**
 * Normalize a category value to one of the canonical AO3 category names
 * ('Gen' | 'F/M' | 'F/F' | 'M/M' | 'Multi').
 * Returns `null` when the input is empty / unrecognised.
 */
export function normalizeCategory(cat: string | null | undefined): string | null {
  if (!cat) return null;
  const key = cat.toLowerCase().trim();
  if (key === 'gen' || key === 'general') return 'Gen';
  if (key === 'f/m' || key === 'male/female') return 'F/M';
  if (key === 'f/f' || key === 'female/female') return 'F/F';
  if (key === 'm/m' || key === 'male/male') return 'M/M';
  if (key === 'multi') return 'Multi';
  // Already a canonical code
  if (AO3_CATEGORY_CODES[cat]) return AO3_CATEGORY_CODES[cat];
  return null;
}

/**
 * Pick the first (highest-priority) category code from a list of
 * categories and return the short square text for it.
 * AO3 displays a single multi-romance-category glyph, so we surface the
 * first recognised category. Returns the globe glyph 🌐 when there are
 * no categories.
 */
export function categorySquare(categories: string[] | null | undefined): string {
  if (!categories || categories.length === 0) return '🌐';
  for (const c of categories) {
    const norm = normalizeCategory(c);
    if (norm) return AO3_CATEGORY_CODES[norm] ?? norm;
  }
  return '🌐';
}

// ── Archive Warnings ───────────────────────────────────────────────────

/**
 * AO3 archive warnings as a set of canonical names. AO3 treats the absence
 * of warning tags with the special "No Archive Warnings" placeholder.
 */
export const AO3_WARNINGS = [
  'Creator Chose Not To Use Archive Warnings',
  'Graphic Descriptions of Sexual Violence',
  'Graphic Violence and Peril',
  'Major Character Death',
  'Non-Concensual Content',
  'Rape/Non-Con',
  'Sexual Content',
  'Underage',
  'Violence',
  'Major Death',
  'Rape/Non-Con',
  'Non-Con',
] as const;

/**
 * Derive the warning square glyph for a work's warning tags.
 *
 * AO3 convention (and the one we mirror):
 *  - No warnings → '✓'  (checkmark: "no major warnings")
 *  - Has warnings → '!' (exclamation: "warnings apply")
 *
 * If the list contains "Creator Chose Not To Use Archive Warnings"
 * specifically, we also surface '!' because the creator opted out.
 */
export function warningSquare(warnings: string[] | null | undefined): string {
  if (!warnings || warnings.length === 0) return '✓';
  const hasCreatorNote = warnings.some(
    (w) => w?.toLowerCase() === "creator chose not to use archive warnings"
  );
  if (hasCreatorNote) return '!';
  return '!';
}

// ── Completion / Status ────────────────────────────────────────────────

/**
 * AO3 statuses as a normalised lowercase set.
 */
export const AO3_STATUS_COMPLETE = ['complete', 'completed', 'finished'] as const;

/**
 * Derive the completion square glyph from a work's status string.
 *
 *  - Complete → '✓'
 *  - Everything else (in-progress, on-hold, abandoned, etc.) → '✗'
 */
export function statusSquare(status: string | null | undefined): string {
  if (!status) return '✗';
  const low = status.toLowerCase().trim();
  return AO3_STATUS_COMPLETE.includes(low as (typeof AO3_STATUS_COMPLETE)[number]) ? '✓' : '✗';
}

// ── Tooltip / guide text helper ───────────────────────────────────────

/**
 * Build a human-readable tooltip describing what each square means for a
 * given work. Used for the hover tooltip on SymbolSquares.
 * Values come from the i18n store (the `symbols.*` keys) so the guide is
 * localised; this helper simply joins them with a delimiter.
 */
export function symbolTooltip(parts: { rating?: string; category?: string; warning?: string; status?: string }): string {
  const bits: string[] = [];
  if (parts.rating) bits.push(parts.rating);
  if (parts.category) bits.push(parts.category);
  if (parts.warning) bits.push(parts.warning);
  if (parts.status) bits.push(parts.status);
  return bits.join(' · ');
}
