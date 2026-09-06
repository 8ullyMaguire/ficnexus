// Type definitions matching the FicNexus Rust backend (v0 API) response shapes.
// Backend: ~/code/rust/fichub/src/routes/{export,meta,recommender}/routes.rs

/** A single export's download URLs (keys: epub, html, txt, md, mobi, pdf, azw3, docx, fb2, kepub). */
export interface ExportUrls {
  epub?: string;
  html?: string;
  txt?: string;
  md?: string;
  mobi?: string;
  pdf?: string;
  azw3?: string;
  docx?: string;
  fb2?: string;
  kepub?: string;
}

/** Fic metadata object returned by /api/epub and /api/meta. */
export interface FicMeta {
  id: string;
  work_id?: number;
  title: string;
  author: string;
  chapters: number;
  words: number;
  estimated_reading_minutes?: number;
  description: string;
  status: string;
  source: string;
  created: string;
  updated: string;
  extra_meta: unknown | null;
  raw_extended_meta: unknown | null;
  author_url: string;
  author_local_id: string;
  source_id: number;
  author_id: number;
}

/** A tag associated with a fic (from fic_tags table). */
export interface FicTag {
  name: string;
  category: number;  // tag_type_id: 1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category
  score: number;
  id: number;
}

/** Tags grouped by type. */
export interface FicTags {
  fandom: FicTag[];
  character: FicTag[];
  relationship: FicTag[];
  freeform: FicTag[];
  warning: FicTag[];
  category: FicTag[];
}

/** A source (fic_info) entry for a merged work. */
export interface FicSource {
  url_id: string;
  site: string;
  source_url: string;
  words: number;
  chapters: number;
}

/** Response from GET /api/epub?q=<url> and GET /api/meta?q=<url>. */
export interface ExportResponse {
  err: number;
  q?: string;
  msg?: string;
  fixits?: unknown[];
  info?: string;
  url_id?: string;
  slug?: string;
  meta?: FicMeta;
  tags?: FicTags;
  sources?: FicSource[];
  hashes?: Record<string, string>;
  urls?: ExportUrls;
  epub_url?: string | null;
  html_url?: string | null;
  txt_url?: string | null;
  md_url?: string | null;
  mobi_url?: string | null;
  pdf_url?: string | null;
  azw3_url?: string | null;
  docx_url?: string | null;
  fb2_url?: string | null;
  kepub_url?: string | null;
  notes?: string[];
}

/** A single recommendation returned by GET /api/recommendations. */
export interface RecResult {
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  site_domain: string;
  summary: string;
  score: number;
  community_score: number;
  download_urls: Record<string, string>;
}

/** Response from GET /api/recommendations. */
export interface RecommendationsResponse {
  err: number;
  url_id: string;
  site_domain?: string | null;
  recommendations: RecResult[];
  generated_at: string;
}

/** A community suggestion returned by GET /api/recommendations/votes. */
export interface Suggestion {
  id: number;
  suggested_url_id: string;
  comment: string | null;
  net_votes: number;
  created: string | null;
}

/** Response from GET /api/recommendations/votes. */
export interface VotesResponse {
  err: number;
  url_id: string;
  suggestions: Suggestion[];
}

/** Response from POST /api/recommendations/suggest. */
export interface SuggestResponse {
  err: number;
  suggestion_id?: number;
  msg?: string;
}

/** Response from POST /api/recommendations/vote. */
export interface VoteResponse {
  err: number;
  new_score: number;
}

/** A per-fic suggestion returned by GET /api/fic-suggestions. */
export interface FicSuggestion {
  id: number;
  url_id: string;
  suggested_url_id: string;
  suggested_title: string;
  suggested_author: string;
  comment: string | null;
  score: number;
  total_votes: number;
  my_vote: number; // 1 | -1 | 0
  user_id?: number | null;
  created?: string | null;
}

/** Entity kinds for entity recommendations. */
export type EntityKind = 'tag' | 'fandom' | 'author' | 'collection' | 'user';

/** A single entity recommendation returned by GET /api/recommendations/entities. */
export interface EntityRec {
  id: unknown;
  name: string;
  score: number;
  reason: string;
}

/** Response from GET /api/recommendations/entities. */
export interface EntityRecsResponse {
  err: number;
  kind: EntityKind;
  items: EntityRec[];
}
