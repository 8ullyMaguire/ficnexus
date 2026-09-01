/**
 * AO3-style search form logic — pure functions, no Svelte.
 *
 * Handles the Work Search form state, URL round-trip, range parsing,
 * and chip-merge behaviour for the archive UI.
 */

// ── Types ────────────────────────────────────────────────────────────────────

/** A numeric range with optional min/max. */
export interface Range {
  min?: number;
  max?: number;
}

/** Full state of the AO3-style work search form. */
export interface FormState {
  /** Free-text query (may include field prefixes like title:(...) appended). */
  q: string;
  /** Sort key — empty string = Best Match. */
  sort: string;
  /** Sort direction: '' = descending (default), 'asc' = ascending. */
  dir: string;
  /** Word count range */
  min_words: string;
  max_words: string;
  /** Completion status: '' = all, 'true' = complete, 'false' = in progress. */
  complete: string;
  /** Single chapter toggle (1 = max_chapters=1). */
  single_chapter: boolean;
  /** Date from (YYYY-MM-DD). */
  date_from: string;
  /** Kudos range */
  min_kudos: string;
  max_kudos: string;
  /** Comments range */
  min_comments: string;
  max_comments: string;
  /** Bookmarks range */
  min_bookmarks: string;
  max_bookmarks: string;
  /** Tag IDs (comma-separated). */
  tag_ids: string;
  /** Include tags (comma-separated, may be type-prefixed like "1:Harry Potter"). */
  include_tags: string;
  /** Exclude tags (comma-separated, may be type-prefixed). */
  exclude_tags: string;
  /** No warnings checkbox. */
  no_warnings: boolean;
  /** Rating filter value — mapped to include_tags by the component. */
  rating: string;
  /** Language (DISABLED/inert). */
  language: string;

  // ── Typed tag fields (AO3 separates these) ──
  /** Fandoms (tag type 1). */
  fandoms: string;
  /** Characters (tag type 2). */
  characters: string;
  /** Relationships (tag type 3). */
  relationships: string;
  /** Additional / freeform tags (tag type 4). */
  additional_tags: string;
  /** Warnings checkboxes (tag type 5): comma-separated warning names. */
  warnings: string[];
  /** Categories checkboxes (tag type 6): comma-separated category names. */
  categories: string[];
}

/** A map of which params the backend actually supports. */
export type SupportedMatrix = Record<string, boolean>;

/** Default empty form state. */
export function defaultFormState(): FormState {
  return {
    q: '',
    sort: '',
    dir: '',
    min_words: '',
    max_words: '',
    complete: '',
    single_chapter: false,
    date_from: '',
    min_kudos: '',
    max_kudos: '',
    min_comments: '',
    max_comments: '',
    min_bookmarks: '',
    max_bookmarks: '',
    tag_ids: '',
    include_tags: '',
    exclude_tags: '',
    no_warnings: false,
    rating: '',
    language: '',
    fandoms: '',
    characters: '',
    relationships: '',
    additional_tags: '',
    warnings: [],
    categories: [],
  };
}

// ── Range parsing ────────────────────────────────────────────────────────────

/** Regex for an AO3-style range string like "1000", ">1000", "<5000", "1000-5000", "≥1000", "≤5000". */
const RANGE_RE = /^\s*(?:([><≤≥])\s*)?(\d+)(?:\s*[-–—]\s*(\d+))?\s*$/;

/**
 * Parse an AO3-style range string into { min?, max? }.
 *
 * Supports: '1000', '>1000', '<5000', '1000-5000', '≥1000', '≤5000', ''.
 * Returns {} for empty or unrecognisable input.
 */
export function parseRange(input: string): Range {
  if (!input || !input.trim()) return {};

  const m = input.match(RANGE_RE);
  if (!m) return {};

  const [, op, single, rangeMax] = m;

  // Range form: "1000-5000"
  if (rangeMax !== undefined) {
    const lo = Number(single);
    const hi = Number(rangeMax);
    if (!Number.isFinite(lo) || !Number.isFinite(hi)) return {};
    return { min: lo, max: hi };
  }

  const n = Number(single);
  if (!Number.isFinite(n)) return {};

  if (!op) return { min: n };
  if (op === '>') return { min: n };
  if (op === '≥') return { min: n };
  if (op === '<') return { max: n };
  if (op === '≤') return { max: n };

  return {};
}

// ── Form → URLSearchParams ───────────────────────────────────────────────────

/**
 * Build a URLSearchParams from form state, using the supportedMatrix to
 * decide which params to include and which to skip (with notes).
 *
 * Returns { params, notes } where notes is an array of warning strings for
 * fields that were skipped because they are not supported.
 */
export function buildSearchQuery(
  state: FormState,
  matrix: SupportedMatrix,
): { params: URLSearchParams; notes: string[] } {
  const params = new URLSearchParams();
  const notes: string[] = [];

  function set(key: string, value: string | number | boolean) {
    if (!matrix[key]) {
      notes.push(`${key} is not supported by this archive`);
      return;
    }
    const s = String(value).trim();
    if (s === '' || s === 'false') return;
    params.set(key, s);
  }

  /**
   * Parse a range field as a minimum (for min_* fields): a bare number,
   * '>n'/'≥n' → min=n, or 'a-b' → min=a.
   */
  function parseMinField(raw: string): number | undefined {
    const r = parseRange(raw);
    return r.min ?? r.max;
  }

  /**
   * Parse a range field as a maximum (for max_* fields): a bare number,
   * '<n'/'≤n' → max=n, or 'a-b' → max=b.
   */
  function parseMaxField(raw: string): number | undefined {
    const r = parseRange(raw);
    return r.max ?? r.min;
  }

  // q — combine base q with field-prefixed terms
  let q = state.q.trim();
  if (state.fandoms.trim()) {
    q = q ? `${q} 1:${state.fandoms.trim()}` : `1:${state.fandoms.trim()}`;
  }
  if (state.characters.trim()) {
    q = q ? `${q} 2:${state.characters.trim()}` : `2:${state.characters.trim()}`;
  }
  if (state.relationships.trim()) {
    q = q ? `${q} 3:${state.relationships.trim()}` : `3:${state.relationships.trim()}`;
  }
  if (state.additional_tags.trim()) {
    q = q ? `${q} 4:${state.additional_tags.trim()}` : `4:${state.additional_tags.trim()}`;
  }
  // Warnings (type 5)
  if (state.warnings.length > 0) {
    const warnParts = state.warnings.map((w) => `5:${w}`);
    q = q ? `${q} ${warnParts.join(' ')}` : warnParts.join(' ');
  }
  // Categories (type 6)
  if (state.categories.length > 0) {
    const catParts = state.categories.map((c) => `6:${c}`);
    q = q ? `${q} ${catParts.join(' ')}` : catParts.join(' ');
  }
  if (q && matrix.q) params.set('q', q);
  else if (q && !matrix.q) notes.push('q is not supported by this archive');

  // Sort
  if (state.sort) set('sort', state.sort);
  if (state.dir) set('dir', state.dir);

  // Word count
  const wcMin = parseMinField(state.min_words);
  const wcMax = parseMaxField(state.max_words);
  if (wcMin !== undefined) set('min_words', wcMin);
  if (wcMax !== undefined) set('max_words', wcMax);

  // Completion
  if (state.complete) set('complete', state.complete);

  // Single chapter
  if (state.single_chapter) set('max_chapters', '1');

  // Date
  if (state.date_from.trim()) set('date_from', state.date_from.trim());

  // Rating
  if (state.rating) set('rating', state.rating);

  // Include / exclude tags (built from typed fields)
  const includeParts: string[] = [];
  if (state.include_tags.trim()) includeParts.push(state.include_tags.trim());
  if (includeParts.length > 0) {
    if (matrix.include_tags) {
      params.set('include_tags', includeParts.join(', '));
    } else {
      notes.push('include_tags is not supported by this archive');
    }
  }

  if (state.exclude_tags.trim()) {
    if (matrix.exclude_tags) {
      params.set('exclude_tags', state.exclude_tags.trim());
    } else {
      notes.push('exclude_tags is not supported by this archive');
    }
  }

  // Tag IDs
  if (state.tag_ids.trim()) {
    if (matrix.tag_ids) {
      params.set('tag_ids', state.tag_ids.trim());
    } else {
      notes.push('tag_ids is not supported by this archive');
    }
  }

  // No warnings
  if (state.no_warnings) set('no_warnings', 'true');

  // Stats ranges
  const kudosMin = parseMinField(state.min_kudos);
  if (kudosMin !== undefined) set('min_kudos', kudosMin);

  const kudosMax = parseMaxField(state.max_kudos);
  if (kudosMax !== undefined) {
    if (matrix.max_kudos) {
      params.set('max_kudos', String(kudosMax));
    } else {
      notes.push('max_kudos is not supported — only min_kudos is applied');
    }
  }

  const commentsMin = parseMinField(state.min_comments);
  if (commentsMin !== undefined) set('min_comments', commentsMin);

  const commentsMax = parseMaxField(state.max_comments);
  if (commentsMax !== undefined) {
    if (matrix.max_comments) {
      params.set('max_comments', String(commentsMax));
    } else {
      notes.push('max_comments is not supported — only min_comments is applied');
    }
  }

  const bkmMin = parseMinField(state.min_bookmarks);
  if (bkmMin !== undefined) {
    if (matrix.min_bookmarks) {
      params.set('min_bookmarks', String(bkmMin));
    } else {
      notes.push('min_bookmarks is not supported by this archive');
    }
  }

  const bkmMax = parseMaxField(state.max_bookmarks);
  if (bkmMax !== undefined) {
    if (matrix.max_bookmarks) {
      params.set('max_bookmarks', String(bkmMax));
    } else {
      notes.push('max_bookmarks is not supported — only min_bookmarks is applied');
    }
  }

  // Inert fields
  if (state.language) notes.push('language is not supported by this archive');

  return { params, notes };
}

// ── URL ↔ FormState round-trip ──────────────────────────────────────────────

/**
 * Serialize a FormState into URL search params (for pushState / shareable links).
 * Omits default values.
 */
export function formStateToUrl(state: FormState): URLSearchParams {
  const params = new URLSearchParams();

  function add(key: string, value: string | number | boolean, skipDefault?: string | boolean) {
    const s = String(value).trim();
    if (s === '' || s === String(skipDefault)) return;
    params.set(key, s);
  }

  add('q', state.q);
  add('sort', state.sort);
  add('dir', state.dir);
  add('min_words', state.min_words);
  add('max_words', state.max_words);
  add('complete', state.complete, '');
  if (state.single_chapter) add('single_chapter', '1');
  add('date_from', state.date_from);
  add('min_kudos', state.min_kudos);
  add('max_kudos', state.max_kudos);
  add('min_comments', state.min_comments);
  add('max_comments', state.max_comments);
  add('min_bookmarks', state.min_bookmarks);
  add('max_bookmarks', state.max_bookmarks);
  add('tag_ids', state.tag_ids);
  add('include_tags', state.include_tags);
  add('exclude_tags', state.exclude_tags);
  if (state.no_warnings) add('no_warnings', '1');
  add('rating', state.rating);
  add('fandoms', state.fandoms);
  add('characters', state.characters);
  add('relationships', state.relationships);
  add('additional_tags', state.additional_tags);
  if (state.warnings.length > 0) add('warnings', state.warnings.join(','));
  if (state.categories.length > 0) add('categories', state.categories.join(','));

  return params;
}

/**
 * Restore a FormState from URL search params (e.g. from window.location.search).
 */
export function urlToFormState(params: URLSearchParams | string): FormState {
  const sp = typeof params === 'string' ? new URLSearchParams(params) : params;
  const state = defaultFormState();

  state.q = sp.get('q') ?? '';
  state.sort = sp.get('sort') ?? '';
  state.dir = sp.get('dir') ?? '';
  state.min_words = sp.get('min_words') ?? '';
  state.max_words = sp.get('max_words') ?? '';
  state.complete = sp.get('complete') ?? '';
  state.single_chapter = sp.get('single_chapter') === '1';
  state.date_from = sp.get('date_from') ?? '';
  state.min_kudos = sp.get('min_kudos') ?? '';
  state.max_kudos = sp.get('max_kudos') ?? '';
  state.min_comments = sp.get('min_comments') ?? '';
  state.max_comments = sp.get('max_comments') ?? '';
  state.min_bookmarks = sp.get('min_bookmarks') ?? '';
  state.max_bookmarks = sp.get('max_bookmarks') ?? '';
  state.tag_ids = sp.get('tag_ids') ?? '';
  state.include_tags = sp.get('include_tags') ?? '';
  state.exclude_tags = sp.get('exclude_tags') ?? '';
  state.no_warnings = sp.get('no_warnings') === '1';
  state.rating = sp.get('rating') ?? '';
  state.fandoms = sp.get('fandoms') ?? '';
  state.characters = sp.get('characters') ?? '';
  state.relationships = sp.get('relationships') ?? '';
  state.additional_tags = sp.get('additional_tags') ?? '';

  const warningsRaw = sp.get('warnings');
  state.warnings = warningsRaw ? warningsRaw.split(',').map((w) => w.trim()).filter(Boolean) : [];

  const categoriesRaw = sp.get('categories');
  state.categories = categoriesRaw ? categoriesRaw.split(',').map((c) => c.trim()).filter(Boolean) : [];

  return state;
}

// ── Chip merge ───────────────────────────────────────────────────────────────

/**
 * Chip types returned by /api/search/suggest.
 * Each chip carries a tag_type_id and name; clicking one adds the tag to the
 * appropriate typed field in the form state.
 */
export interface Chip {
  tag_type_id: number;
  name: string;
}

/**
 * When a suggestion chip is clicked, add its tag to the matching form field
 * and return the updated state (original is not mutated).
 */
export function mergeChipIntoForm(state: FormState, chip: Chip): FormState {
  const next = { ...state };
  const tagStr = `${chip.name}`;

  switch (chip.tag_type_id) {
    case 1: // Fandom
      next.fandoms = next.fandoms ? `${next.fandoms}, ${tagStr}` : tagStr;
      break;
    case 2: // Character
      next.characters = next.characters ? `${next.characters}, ${tagStr}` : tagStr;
      break;
    case 3: // Relationship
      next.relationships = next.relationships ? `${next.relationships}, ${tagStr}` : tagStr;
      break;
    case 4: // Freeform / Additional
      next.additional_tags = next.additional_tags ? `${next.additional_tags}, ${tagStr}` : tagStr;
      break;
    case 5: // Warning
      next.warnings = next.warnings.includes(tagStr)
        ? next.warnings
        : [...next.warnings, tagStr];
      break;
    case 6: // Category
      next.categories = next.categories.includes(tagStr)
        ? next.categories
        : [...next.categories, tagStr];
      break;
    default:
      // Unknown type → generic include_tags
      next.include_tags = next.include_tags ? `${next.include_tags}, ${tagStr}` : tagStr;
      break;
  }

  return next;
}

// ── The canonical SUPPORTED matrix ──────────────────────────────────────────

/**
 * The default supported-param matrix. Values indicate whether the backend
 * handles a parameter; `false` means the UI shows it inert/disabled with a
 * footnote explaining why.
 */
export const SUPPORTED: SupportedMatrix = {
  q: true,
  sort: true,
  min_words: true,
  max_words: true,
  complete: true,
  max_chapters: true,
  date_from: true,
  min_kudos: true,
  min_comments: true,
  include_tags: true,
  exclude_tags: true,
  tag_ids: true,
  no_warnings: true,
  max_kudos: false,
  min_bookmarks: false,
  max_bookmarks: false,
  rating: false,
  language: false,
  dir: false,
};
