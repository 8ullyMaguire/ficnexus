// Search API types and client for GET /api/search
// Backend: src/search/routes.rs + src/search/builder.rs

import type { FicMeta } from './types';

/** Tag types matching the database schema. */
export const TAG_TYPES = {
  1: 'Fandom',
  2: 'Character',
  3: 'Relationship',
  4: 'Freeform',
} as const;

export type TagTypeId = keyof typeof TAG_TYPES;

/** A tag attached to a search result. */
export interface SearchTag {
  name: string;
  type: string;
  type_id: number;
  score: number;
}

/** A single facet value with its result count. */
export interface SearchFacet {
  name: string;
  count: number;
}

/** Facet counts returned alongside search results. */
export interface SearchFacets {
  fandoms: SearchFacet[];
  characters: SearchFacet[];
  relationships: SearchFacet[];
  warnings: SearchFacet[];
  categories: SearchFacet[];
  freeforms: SearchFacet[];
  statuses: SearchFacet[];
}

/** A single search result item. */
export interface SearchResult {
  url_id: string;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  updated: string | null;
  rank: number | null;
  snippet: string | null;
  tags: SearchTag[];
  total_freeform: number;
  comment_count: number;
  kudos_count: number;
}

/** Response from GET /api/search. */
export interface SearchResponse {
  total: number;
  page: number;
  per_page: number;
  results: SearchResult[];
  facets: SearchFacets;
}

/** All filter parameters for the search API. */
export interface SearchFilters {
  q: string;
  include_tags: string;
  exclude_tags: string;
  include_any_tags: string;
  /** Comma-separated tag TYPE ids to exclude entirely (e.g. "3" = no ships). */
  exclude_tag_types: string;
  /** Strict Gen: hide relationship + category tagged fics. */
  strict_gen: boolean;
  min_words: number | null;
  max_words: number | null;
  min_chapters: number | null;
  max_chapters: number | null;
  complete: boolean | null;
  source: string;
  date_from: string;
  date_to: string;
  sort: string;
  page: number;
  per_page: number;
  primary_tag: string;
  relationship_characters: string;
  min_comments: number | null;
  min_kudos: number | null;
  no_warnings: boolean | null;
  tag_ids: string;
  /** Content rating filter (canonical AO3 tag name, e.g. "Explicit"). Honored by the backend.
   *  Optional so callers that build a SearchFilters literal don't need to declare it. */
  rating?: string;
  /** Completion status (complete|ongoing|hiatus|cancelled) — v2 spec. Backed by the
   *  `complete` boolean today and by the `status` param once the v2 backend lands;
   *  sending it is harmless (the handler ignores unknown params). Optional for the
   *  same reason as `rating`. */
  status?: string;
  /** Hide works the signed-in user marked read (status = 'completed'). */
  hide_read: boolean;
  /** Hide works the signed-in user bookmarked. */
  hide_bookmarked: boolean;
  /** Search only the signed-in user's bookmarked works ("My library"). */
  library_only: boolean;
}

/** Sort options matching the backend. */
export const SORT_OPTIONS = [
  { value: '', label: 'Relevance' },
  { value: 'updated', label: 'Date Updated' },
  { value: 'created', label: 'Date Published' },
  { value: 'words', label: 'Word Count' },
  { value: 'kudos', label: 'Kudos' },
] as const;

/** Completion status options. */
export const COMPLETE_OPTIONS = [
  { value: '', label: 'All Works' },
  { value: 'true', label: 'Complete Only' },
  { value: 'false', label: 'In Progress Only' },
] as const;

/** Source/site options. */
export const SOURCE_OPTIONS = [
  { value: '', label: 'All Sites' },
  { value: 'archiveofourown.org', label: 'Archive of Our Own' },
  { value: 'fanfiction.net', label: 'FanFiction.net' },
  { value: 'fictionpress.com', label: 'FictionPress' },
  { value: 'forums.spacebattles.com', label: 'SpaceBattles' },
  { value: 'forums.sufficientvelocity.com', label: 'SufficientVelocity' },
] as const;

/** Localized labels for the sort/completion/source option lists. */
export const SORT_OPTION_KEYS: Record<string, string> = {
  '': 'search.sortRelevance',
  updated: 'search.sortUpdated',
  created: 'search.sortCreated',
  words: 'search.sortWords',
  kudos: 'search.sortKudos',
};

export const COMPLETE_OPTION_KEYS: Record<string, string> = {
  '': 'search.completeAll',
  true: 'search.completeOnly',
  false: 'search.completeInProgress',
};

export const SOURCE_OPTION_KEYS: Record<string, string> = {
  '': 'search.sourceAll',
};

/** Default search filters. */
export function defaultFilters(): SearchFilters {
  return {
    q: '',
    include_tags: '',
    exclude_tags: '',
    include_any_tags: '',
    exclude_tag_types: '',
    strict_gen: false,
    min_words: null,
    max_words: null,
    min_chapters: null,
    max_chapters: null,
    complete: null,
    source: '',
    date_from: '',
    date_to: '',
    sort: '',
    page: 1,
    per_page: 20,
    primary_tag: '',
    relationship_characters: '',
    min_comments: null,
    min_kudos: null,
    no_warnings: null,
    tag_ids: '',
    rating: '',
    status: '',
    hide_read: false,
    hide_bookmarked: false,
    library_only: false,
  };
}

/** Build query string from filters, omitting empty/null params. */
export function buildSearchQuery(filters: SearchFilters): string {
  const params = new URLSearchParams();

  if (filters.q) params.set('q', filters.q);
  if (filters.include_tags) params.set('include_tags', filters.include_tags);
  if (filters.exclude_tags) params.set('exclude_tags', filters.exclude_tags);
  if (filters.include_any_tags) params.set('include_any_tags', filters.include_any_tags);
  if (filters.exclude_tag_types) params.set('exclude_tag_types', filters.exclude_tag_types);
  if (filters.strict_gen) params.set('strict_gen', 'true');
  if (filters.min_words !== null) params.set('min_words', String(filters.min_words));
  if (filters.max_words !== null) params.set('max_words', String(filters.max_words));
  if (filters.min_chapters !== null) params.set('min_chapters', String(filters.min_chapters));
  if (filters.max_chapters !== null) params.set('max_chapters', String(filters.max_chapters));
  if (filters.complete !== null) params.set('complete', String(filters.complete));
  if (filters.source) params.set('source', filters.source);
  if (filters.date_from) params.set('date_from', filters.date_from);
  if (filters.date_to) params.set('date_to', filters.date_to);
  if (filters.sort) params.set('sort', filters.sort);
  if (filters.page > 1) params.set('page', String(filters.page));
  if (filters.per_page !== 20) params.set('per_page', String(filters.per_page));
  if (filters.primary_tag) params.set('primary_tag', filters.primary_tag);
  if (filters.relationship_characters) params.set('relationship_characters', filters.relationship_characters);
  if (filters.min_comments !== null) params.set('min_comments', String(filters.min_comments));
  if (filters.min_kudos !== null) params.set('min_kudos', String(filters.min_kudos));
  if (filters.no_warnings !== null) params.set('no_warnings', String(filters.no_warnings));
  if (filters.tag_ids) params.set('tag_ids', filters.tag_ids);
  if (filters.rating) params.set('rating', filters.rating);
  if (filters.status) params.set('status', filters.status);
  if (filters.hide_read) params.set('hide_read', 'true');
  if (filters.hide_bookmarked) params.set('hide_bookmarked', 'true');
  if (filters.library_only) params.set('library_only', 'true');

  return params.toString();
}

/** Execute a search request. */
export async function search(filters: SearchFilters): Promise<SearchResponse> {
  const qs = buildSearchQuery(filters);
  const res = await fetch(`/api/search${qs ? '?' + qs : ''}`);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Search failed (${res.status}): ${text}`);
  }
  return (await res.json()) as SearchResponse;
}

/** A filter suggestion from GET /api/search/suggest. */
export interface SearchSuggestion {
  id: number;
  name: string;
  tag_type_id: number;
  usage_count: number;
  reason: 'popular' | 'for_you';
}

/** A tag autocomplete hit from GET /api/tags/autocomplete. */
export interface TagAutocompleteHit {
  id: number;
  name: string;
  type: string;
  type_id: number;
  usage_count: number;
}

/** GET /api/tags/autocomplete?q=... — tag suggestions with usage counts. */
export async function fetchTagAutocomplete(q: string, tagType?: number): Promise<TagAutocompleteHit[]> {
  if (q.trim().length < 2) return [];
  const params = new URLSearchParams({ q: q.trim() });
  if (tagType !== undefined) params.set('tag_type', String(tagType));
  const res = await fetch(`/api/tags/autocomplete?${params.toString()}`);
  if (!res.ok) throw new Error(`Tag autocomplete failed (${res.status})`);
  const data = (await res.json()) as { err: number; results?: TagAutocompleteHit[] };
  if (data.err !== 0) throw new Error(`Tag autocomplete err:${data.err}`);
  return data.results ?? [];
}

/**
 * GET /api/search/suggest — popular (or personalized) filter suggestions.
 * Throws on error so callers can silently hide the row.
 */
export async function fetchSearchSuggestions(personal: boolean): Promise<SearchSuggestion[]> {
  const qs = personal ? '?personal=1' : '';
  const res = await fetch(`/api/search/suggest${qs}`, { credentials: 'include' });
  if (!res.ok) {
    throw new Error(`Suggest failed (${res.status})`);
  }
  const data = (await res.json()) as { err: number; suggestions?: SearchSuggestion[] };
  if (data.err !== 0) throw new Error(`Suggest err:${data.err}`);
  return data.suggestions ?? [];
}

// ── Body full-text search (GET /api/search/body) ────────────────────────────

/** A single body-search result: fic metadata + highlighted body excerpt. */
export interface BodySearchResult {
  url_id: string;
  title: string;
  author: string;
  source: string;
  words: number;
  chapters: number;
  status: string;
  description: string;
  work_id: number | null;
  /** `<mark>`-highlighted excerpt from the fic body (null when the body cache was cleared). */
  body_snippet: string | null;
}

/** Response from GET /api/search/body. */
export interface BodySearchResponse {
  err: number;
  total: number;
  page: number;
  per_page: number;
  results: BodySearchResult[];
}

/**
 * Full-text search over fic bodies ("fics where X says Y").
 * Supports the same boolean syntax as /api/search (AND/OR/NOT, quotes, -exclude).
 */
export async function bodySearch(q: string, page = 1, perPage = 20): Promise<BodySearchResponse> {
  const params = new URLSearchParams({ q: q.trim(), page: String(page), per_page: String(perPage) });
  const res = await fetch(`/api/search/body?${params.toString()}`);
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Body search failed (${res.status}): ${text}`);
  }
  const data = (await res.json()) as BodySearchResponse;
  if (data.err !== 0) throw new Error(`Body search err:${data.err}`);
  return data;
}
