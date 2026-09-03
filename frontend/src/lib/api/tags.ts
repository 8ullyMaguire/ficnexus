// Tags API helpers — browse all tags by type.

export interface TagItem {
  id: number;
  name: string;
  tag_type_id: number;
  description: string | null;
  usage_count: number;
  is_alias: boolean;
}

export interface TagSearchResponse {
  err: number;
  total: number;
  tags: TagItem[];
}

/**
 * GET /api/tags/search?type=&limit=&sort= — search/list tags.
 * type: 1=fandom, 2=character, 3=relationship, 4=freeform, 5=warning, 6=category
 */
export async function fetchTagsByType(
  typeId: number,
  limit = 200,
  sort = 'usage',
): Promise<TagSearchResponse> {
  const params = new URLSearchParams({
    type: String(typeId),
    limit: String(limit),
    sort,
  });
  const res = await fetch(`/api/tags/search?${params.toString()}`);
  if (!res.ok) throw new Error(`Tags API error ${res.status}`);
  return res.json();
}

/**
 * Advanced tag search — mirrors the AO3 tag search form fields.
 * GET /api/tags/search?name=&fandom=&tag_type=&wrangling_status=&sort_by=&sort_direction=
 */
export interface TagSearchAdvancedParams {
  q?: string;
  tag_type?: number;
  canonical?: boolean;
  fandom?: string;
  page?: number;
  limit?: number;
  sort?: 'name' | 'usage' | 'created';
  sort_direction?: 'asc' | 'desc';
}

export interface TagSearchAdvancedResponse {
  err: number;
  msg?: string;
  tags?: TagItem[];
  total?: number;
}

export async function searchTagsAdvanced(
  params: TagSearchAdvancedParams,
): Promise<TagSearchAdvancedResponse> {
  const p: Record<string, string> = {};
  if (params.q) p.name = params.q;
  if (params.tag_type != null) p.tag_type = String(params.tag_type);
  if (params.canonical != null) p.wrangle_status = params.canonical ? 'canonical' : 'non-canonical';
  if (params.fandom) p.fandom = params.fandom;
  if (params.page) p.page = String(params.page);
  if (params.limit) p.limit = String(params.limit);
  if (params.sort) p.sort_by = params.sort;
  if (params.sort_direction) p.sort_direction = params.sort_direction;
  const qs = new URLSearchParams(p).toString();
  const res = await fetch(`/api/tags/search${qs ? `?${qs}` : ''}`);
  if (!res.ok) throw new Error(`Tags API error ${res.status}`);
  return res.json();
}

/** Tag type IDs matching the database schema. */
export const TAG_TYPE_LABELS: Record<number, string> = {
  1: 'Fandoms',
  2: 'Characters',
  3: 'Relationships',
  4: 'Additional Tags',
  5: 'Warnings',
  6: 'Categories',
};

/** Tag type name values used in search URLs (e.g. "1:Harry Potter"). */
export const TAG_TYPE_NAMES: Record<number, string> = {
  1: 'Fandom',
  2: 'Character',
  3: 'Relationship',
  4: 'Freeform',
  5: 'Warning',
  6: 'Category',
};

/** Ordered list of tag types to display. */
export const TAG_TYPE_ORDER = [1, 2, 3, 4, 5, 6] as const;
