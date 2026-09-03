// API client for the forum (F2 — categories + topic list, F3 — topic detail,
// write path: create/edit/delete topics & posts, follow toggle).

import { authHeaders } from './social';

export interface ApiResponse {
  err: number;
  msg?: string;
}

export interface ForumCategory {
  id: number;
  slug: string;
  title: string;
  description: string;
  position: number;
  is_mod_only: boolean;
  created_at: string;
  topic_count: number;
  last_activity_at: string | null;
}

export interface ForumTopic {
  id: number;
  title: string;
  /** Human-readable, unique slug (`{base}-{id}`). Drives `/forum/board/{slug}.{id}`. */
  topic_slug?: string | null;
  author_id: number;
  author_username: string | null;
  reply_count: number;
  vote_score: number;
  view_count: number;
  last_post_id: number | null;
  status: 'open' | 'locked' | 'pinned' | 'archived';
  last_activity_at: string;
  created_at: string;
  /** F4: true when the topic has posts newer than the user's last_read_post_id. */
  unread?: boolean;
  last_read_post_id?: number | null;
}

export interface ForumTopicsResponse extends ApiResponse {
  items: ForumTopic[];
  next_cursor: number | null;
  category: string;
  limit: number;
}

export interface ForumPost {
  id: number;
  author_id: number;
  author_username: string | null;
  body: string;
  quote_of: number | null;
  quote: { author_username: string | null; preview: string } | null;
  edited_at: string | null;
  deleted_at: string | null;
  created_at: string;
  score: number;
  is_op: boolean;
  reactions?: {
    reactions: Record<string, Array<{ user_id: number; username: string }>>;
    my_reactions: string[];
  };
}

export interface ForumTopicDetail extends ApiResponse {
  id: number;
  title: string;
  /** Canonical slug (`{base}-{id}`); null only for legacy rows pre-backfill. */
  topic_slug?: string | null;
  author_id: number;
  author_username: string | null;
  category_slug: string;
  category_title: string;
  status: 'open' | 'locked' | 'pinned' | 'archived';
  body: string;
  payload: unknown;
  view_count: number;
  created_at: string;
  updated_at: string;
  items: ForumPost[];
  next_cursor: number | null;
  limit: number;
  view_count_before: number;
  /** F4 read-state: set for auth users, absent for anon. */
  unread?: boolean;
  last_read_post_id?: number | null;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    credentials: 'include',
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...init?.headers,
    },
  });
  return (await res.json()) as T;
}

export async function getForumCategories(): Promise<ApiResponse & { items?: ForumCategory[] }> {
  return request('/api/forum/categories');
}

export type ForumTopicsSort = 'latest' | 'top' | 'oldest' | 'recently_replied' | 'most_posts' | 'most_views' | 'most_votes';

export async function getForumTopics(
  categorySlug: string,
  cursor?: number | null,
  limit?: number,
  sort?: ForumTopicsSort | null,
): Promise<ForumTopicsResponse> {
  const params = new URLSearchParams({ category: categorySlug });
  if (cursor) params.set('cursor', String(cursor));
  if (limit) params.set('limit', String(limit));
  if (sort && sort !== 'latest') params.set('sort', sort);
  return request(`/api/forum/topics?${params.toString()}`);
}

export async function createCategory(body: {
  slug: string;
  title: string;
  description?: string;
  position?: number;
}): Promise<ApiResponse & { id?: number }> {
  return request('/api/forum/categories', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function updateCategory(
  id: number,
  body: {
    slug?: string;
    title?: string;
    description?: string;
    position?: number;
  },
): Promise<ApiResponse> {
  return request(`/api/forum/categories/${id}`, {
    method: 'PATCH',
    body: JSON.stringify(body),
  });
}

// ── F3: topic detail ───────────────────────────────────────────────────────

export async function getForumTopic(
  topicId: number,
  after?: number | null,
  limit?: number,
): Promise<ForumTopicDetail> {
  const params = new URLSearchParams();
  if (after) params.set('after', String(after));
  if (limit) params.set('limit', String(limit));
  const qs = params.toString();
  return request(`/api/forum/topics/${topicId}${qs ? `?${qs}` : ''}`);
}

/** Resolve a topic by its unique slug (`{base}-{id}`). Same response as the
 * numeric detail endpoint. */
export async function getForumTopicBySlug(
  topicSlug: string,
  after?: number | null,
  limit?: number,
): Promise<ForumTopicDetail> {
  const params = new URLSearchParams();
  if (after) params.set('after', String(after));
  if (limit) params.set('limit', String(limit));
  const qs = params.toString();
  return request(`/api/forum/topics/by-slug/${encodeURIComponent(topicSlug)}${qs ? `?${qs}` : ''}`);
}

// ── F3: write path (topics) ────────────────────────────────────────────────

export async function createTopic(body: {
  title: string;
  category_slug: string;
  body: string;
  payload?: unknown;
}): Promise<ApiResponse & { id?: number; topic_slug?: string | null }> {
  return request('/api/forum/topics', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function updateTopic(
  topicId: number,
  body: { title?: string; body?: string },
): Promise<ApiResponse & { id?: number }> {
  return request(`/api/forum/topics/${topicId}`, {
    method: 'PATCH',
    body: JSON.stringify(body),
  });
}

export async function deleteTopic(topicId: number): Promise<ApiResponse & { id?: number }> {
  return request(`/api/forum/topics/${topicId}`, {
    method: 'DELETE',
  });
}

// ── F3: write path (posts) ─────────────────────────────────────────────────

export async function createPost(
  topicId: number,
  body: { body: string; quote_of?: number | null },
): Promise<ApiResponse & { id?: number }> {
  return request(`/api/forum/topics/${topicId}/posts`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

/** Submit an author edit proposal (curators apply it after review). */
export async function submitPostEditProposal(
  postId: number,
  body: { body: string },
): Promise<ApiResponse & { id?: number; proposal_id?: number; status?: string }> {
  return request(`/api/forum/posts/${postId}`, {
    method: 'PATCH',
    body: JSON.stringify(body),
  });
}

/** Legacy name retained for moderator callers. */
export const updatePost = submitPostEditProposal;

export async function deletePost(postId: number): Promise<ApiResponse & { id?: number }> {
  return request(`/api/forum/posts/${postId}`, {
    method: 'DELETE',
  });
}

// ── F3: follow toggle ──────────────────────────────────────────────────────

export type ForumFollowState = ApiResponse & { following?: boolean; follower_count?: number };

export async function getFollowState(topicId: number): Promise<ForumFollowState> {
  return request(`/api/forum/topics/${topicId}/follow`);
}

export async function toggleFollow(topicId: number): Promise<ForumFollowState> {
  return request(`/api/forum/topics/${topicId}/follow`, {
    method: 'POST',
    body: JSON.stringify({}),
  });
}

// ── F4: mark-read + full-text search ─────────────────────────────────────

export interface ForumReadState extends ApiResponse {
  last_read_post_id: number | null;
  updated_at: string;
}

/**
 * Mark a topic as read up to `lastReadPostId` (defaults to the topic's
 * last_post_id on the backend). Fire-and-forget is fine — callers do not
 * need to block on it.
 */
export async function markTopicRead(topicId: number, lastReadPostId?: number | null): Promise<ForumReadState> {
  const body: { last_read_post_id?: number } = {};
  if (lastReadPostId != null) body.last_read_post_id = lastReadPostId;
  return request(`/api/forum/topics/${topicId}/read`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export type ForumSearchResultType = 'topic' | 'post';

export interface ForumSearchResult {
  type: ForumSearchResultType;
  topic_id: number;
  /** Slug of the result's topic; drives the board URL link. */
  topic_slug?: string | null;
  post_id: number | null;
  title: string;
  author_id: number;
  author_username: string | null;
  body: string;
  /** Server-generated ts_headline output — <mark> tags around matches. */
  snippet: string;
  category_slug: string;
  category_title: string;
  created_at: string;
}

export interface ForumSearchResponse extends ApiResponse {
  q: string;
  results: ForumSearchResult[];
  next_cursor: null;
  limit: number;
  total: number;
}

export async function searchForum(
  q: string,
  categorySlug?: string | null,
  limit?: number,
): Promise<ForumSearchResponse> {
  const params = new URLSearchParams({ q });
  if (categorySlug) params.set('category', categorySlug);
  if (limit) params.set('limit', String(limit));
  return request(`/api/forum/search?${params.toString()}`);
}

// ── F5: moderation ─────────────────────────────────────────────────────

/**
 * The nine moderation reasons a curator can apply to a forum post, in the
 * canonical backend order. `delta` is the score change applied to the post;
 * `labelKey` is the i18n key (forum.reason.<key>) for the human label.
 */
export const MODERATION_REASONS: { key: string; delta: number; labelKey: string }[] = [
  { key: 'insightful', delta: 2, labelKey: 'forum.reason.insightful' },
  { key: 'informative', delta: 1, labelKey: 'forum.reason.informative' },
  { key: 'interesting', delta: 1, labelKey: 'forum.reason.interesting' },
  { key: 'funny', delta: 1, labelKey: 'forum.reason.funny' },
  { key: 'off-topic', delta: -1, labelKey: 'forum.reason.off-topic' },
  { key: 'redundant', delta: -1, labelKey: 'forum.reason.redundant' },
  { key: 'flamebait', delta: -2, labelKey: 'forum.reason.flamebait' },
  { key: 'troll', delta: -2, labelKey: 'forum.reason.troll' },
  { key: 'abusive', delta: -3, labelKey: 'forum.reason.abusive' },
];

/** GET /api/forum/moderation/status — the caller's moderation allowance. */
export interface ForumModStatus extends ApiResponse {
  points_left: number;
  expires_at: string | null;
  eligible: boolean;
  reason?: string;
  unfair_rate?: number;
  cooldown_until?: string | null;
}

/** GET /api/forum/moderation/queue — one queued post awaiting moderation. */
export interface ForumModQueueItem {
  post_id: number;
  topic_id: number;
  /** Canonical topic slug (`{base}-{id}`); drives board deep links. */
  topic_slug?: string | null;
  author_username: string;
  /** Truncated body (≤300 chars server-side). */
  body: string;
  score: number;
  mod_count: number;
  reason?: string;
  created_at: string;
  /** Present on some deployments for deep links back to the topic. */
  category_slug?: string;
}

export interface ForumModQueueResponse extends ApiResponse {
  items: ForumModQueueItem[];
  count: number;
  points_left?: number;
}

export interface ForumModerateResponse extends ApiResponse {
  delta: number;
  score_after: number;
  hidden_until?: string | null;
}

/** GET /api/forum/posts/{postId}/moderations — history for one post. */
export interface ForumModerationEntry {
  moderator_id: number | null;
  reason: string;
  delta: number;
  score_after: number;
  created_at: string;
}

export interface ForumModerationsResponse extends ApiResponse {
  items: ForumModerationEntry[];
}

export interface ForumBan {
  id: number;
  user_id: number;
  user_username: string;
  category_slug: string | null;
  category_title: string | null;
  reason: string;
  banned_by_username: string;
  expires_at: string | null;
  created_at: string;
}

export interface ForumBansResponse extends ApiResponse {
  items: ForumBan[];
}

export async function getModerationStatus(): Promise<ForumModStatus> {
  return request('/api/forum/moderation/status');
}

export async function getModerationQueue(): Promise<ForumModQueueResponse> {
  return request('/api/forum/moderation/queue');
}

/** Apply a moderation reason to a post. 409 → already moderated by the caller. */
export async function moderatePost(postId: number, reason: string): Promise<ForumModerateResponse> {
  return request(`/api/forum/posts/${postId}/moderate`, {
    method: 'POST',
    body: JSON.stringify({ reason }),
  });
}

export async function getPostModerations(postId: number): Promise<ForumModerationsResponse> {
  return request(`/api/forum/posts/${postId}/moderations`);
}

/** Hide a post from public view (curator, level ≥ 50). */
export async function hidePost(postId: number): Promise<ApiResponse> {
  return request(`/api/admin/forum/hide/${postId}`, {
    method: 'POST',
    body: JSON.stringify({}),
  });
}

/** Lock a topic against new replies (curator, level ≥ 50). Pass locked explicitly or omit to toggle. */
export async function lockTopic(topicId: number, locked?: boolean): Promise<ApiResponse & { status?: string }> {
  const body: Record<string, boolean> = {};
  if (locked !== undefined) body.locked = locked;
  return request(`/api/admin/forum/topics/${topicId}/lock`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

/** Pin a topic to the top of its category (curator, level ≥ 50). Pass pinned explicitly or omit to toggle. */
export async function pinTopic(topicId: number, pinned?: boolean): Promise<ApiResponse & { status?: string }> {
  const body: Record<string, boolean> = {};
  if (pinned !== undefined) body.pinned = pinned;
  return request(`/api/admin/forum/topics/${topicId}/pin`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export interface CreateBanBody {
  user_id: number;
  scope: 'forum' | 'category';
  category_id?: number;
  reason?: string;
  expires_at?: string | null;
}

/** Ban a user from the forum or a single category (curator, level ≥ 50). */
export async function createBan(body: CreateBanBody): Promise<ApiResponse & { id?: number }> {
  return request('/api/admin/forum/bans', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function listBans(): Promise<ForumBansResponse> {
  return request('/api/admin/forum/bans');
}

/** Lift a forum/category ban (curator, level ≥ 50). */
export async function liftBan(banId: number): Promise<ApiResponse> {
  return request(`/api/admin/forum/bans/${banId}`, {
    method: 'DELETE',
  });
}

export interface ForumReportResponse extends ApiResponse {
  report_id?: number;
}

/** Report a post to the moderation team (any logged-in user). */
export async function reportForumPost(postId: number, reason: string): Promise<ForumReportResponse> {
  return request('/api/reports', {
    method: 'POST',
    body: JSON.stringify({ target_type: 'forum_post', target_id: postId, reason }),
  });
}

export interface ForumReactionResponse {
  err: number;
  msg?: string;
  reactions: {
    reactions: Record<string, Array<{ user_id: number; username: string }>>;
    my_reactions: string[];
  };
}

/** Positive-only emoji set served by the backend (ALLOWED_REACTIONS). */
export const FORUM_REACTIONS = ['👍', '❤️', '😂', '🔥', '👏', '🤔', '😢', '😮'] as const;

export async function reactToPost(postId: number, emoji: string): Promise<ForumReactionResponse> {
  return request(`/api/forum/posts/${postId}/reactions`, {
    method: 'POST',
    body: JSON.stringify({ emoji }),
  });
}

/** GET /api/forum/posts/{postId}/reactions — counts grouped by emoji + viewer state. */
export async function getPostReactions(postId: number): Promise<ForumReactionResponse> {
  return request(`/api/forum/posts/${postId}/reactions`);
}

/** DELETE /api/forum/posts/{postId}/reactions?emoji=… — remove one of my reactions. */
export async function removePostReaction(postId: number, emoji: string): Promise<ForumReactionResponse> {
  return request(`/api/forum/posts/${postId}/reactions?emoji=${encodeURIComponent(emoji)}`, {
    method: 'DELETE',
  });
}

// ── F6: metamoderation ──────────────────────────────────────────────────

/** One sampled moderator action awaiting a metamod verdict. */
export interface ForumMetamodItem {
  action_id: number;
  post_id: number;
  topic_id: number;
  /** Truncated post body (≤200 chars server-side). */
  excerpt: string;
  /** The moderation reason the moderator applied (one of MODERATION_REASONS). */
  reason: string;
  /** Score change the action applied. */
  delta: number;
  /** Post score after the action. */
  score_after: number;
  created_at: string;
}

/** GET /api/forum/metamod/queue — random under-rated actions, moderator anonymized. */
export interface ForumMetamodQueueResponse extends ApiResponse {
  items: ForumMetamodItem[];
  count: number;
  /** True when the eligible pool is too small — metamoderation is dormant. */
  pool_too_small?: boolean;
  next_cursor?: number | null;
}

export type MetamodFilter = 'unreviewed' | 'reviewed' | 'all';
export type ForumMetamodVerdict = 'fair' | 'unfair' | 'unsure';

/** POST /api/forum/metamod/{actionId}/vote — `{err:0}` on success. */
export type ForumMetamodVote = ApiResponse;

/** GET /api/forum/metamod/queue — filterable (unreviewed/reviewed/all + verdict + cursor) */
export async function getMetamodQueue(params?: { filter?: MetamodFilter; verdict?: string | null; cursor?: number | null; limit?: number }): Promise<ForumMetamodQueueResponse> {
  const qs = new URLSearchParams();
  if (params?.filter) qs.set('filter', params.filter);
  if (params?.verdict) qs.set('verdict', params.verdict);
  if (params?.cursor) qs.set('cursor', String(params.cursor));
  if (params?.limit) qs.set('limit', String(params.limit));
  const q = qs.toString();
  return request(`/api/forum/metamod/queue${q ? `?${q}` : ''}`);
}

export async function getMetamodGrant(grantId: number): Promise<ApiResponse & ForumMetamodItem> {
  return request(`/api/forum/metamod/grants/${grantId}`);
}

export async function voteMetamodGrant(grantId: number, verdict: ForumMetamodVerdict): Promise<ApiResponse & { grant_id?: number; verdict?: string }> {
  return request(`/api/forum/metamod/grants/${grantId}/verdict`, { method: 'POST', body: JSON.stringify({ verdict }) });
}

/** Vote on a sampled mod action. 409 → already voted; 400 → invalid verdict; 403 → not eligible. */
export async function voteMetamod(actionId: number, verdict: ForumMetamodVerdict): Promise<ForumMetamodVote> {
  return request(`/api/forum/metamod/${actionId}/vote`, {
    method: 'POST',
    body: JSON.stringify({ verdict }),
  });
}

// ── F7: leveling, site info, invites, registration applications, blocks ──

export type RegistrationMode = 'open' | 'invite' | 'application';

export interface SiteInfo {
  err: number;
  site?: {
    name: string;
    description: string;
    registration_mode: RegistrationMode;
    version: string;
  };
}

/** GET /api/site — public site info (no auth). */
export async function getSiteInfo(): Promise<SiteInfo> {
  return request('/api/site');
}

/** GET /api/users/me/level — the caller's leveling progress (login required). */
export interface LevelProgress extends ApiResponse {
  level: number;
  exp: number;
  exp_to_next: number;
  /** 0..1 float — progress toward the next level. */
  progress: number;
  level_up: boolean;
}

export async function getMyLevel(): Promise<LevelProgress> {
  return request('/api/users/me/level');
}

export interface ForumInvite {
  id: number;
  code: string;
  created_by: number;
  created_at: string;
  expires_at: string | null;
  used_by: number | null;
  used_username: string | null;
  used_at: string | null;
}

export interface InvitesResponse extends ApiResponse {
  items?: ForumInvite[];
}

/** POST /api/admin/invites — generate a single-use invite code (level ≥ 50). */
export async function createInvite(body: { note?: string } = {}): Promise<ApiResponse & { id?: number; code?: string; expires_at?: string | null }> {
  return request('/api/admin/invites', {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

/** GET /api/admin/invites — list invites + usage (level ≥ 50). */
export async function listInvites(): Promise<InvitesResponse> {
  return request('/api/admin/invites');
}

export interface RegistrationApplication {
  id: number;
  user_id: number;
  username: string | null;
  reason: string;
  status: 'pending' | 'approved' | 'rejected';
  reviewed_by: number | null;
  reviewer_name: string | null;
  reviewed_at: string | null;
  created_at: string;
}

export interface ApplicationsResponse extends ApiResponse {
  status?: string;
  items?: RegistrationApplication[];
}

/** POST /api/registration-applications — apply to join (logged-in users). 409 → duplicate pending. */
export async function applyForRegistration(reason: string): Promise<ApiResponse & { id?: number; status?: string }> {
  return request('/api/registration-applications', {
    method: 'POST',
    body: JSON.stringify({ reason }),
  });
}

/** GET /api/admin/registration-applications?status=… (level ≥ 50). */
export async function listRegistrationApplications(status: 'pending' | 'approved' | 'rejected' | 'all' = 'pending'): Promise<ApplicationsResponse> {
  return request(`/api/admin/registration-applications?status=${status}`);
}

/** POST /api/admin/registration-applications/{id}/review (level ≥ 50).
 * Body: `{status: 'approved'|'rejected', reason?}` — approval flips the
 * applicant's role to 1 (trusted) server-side. */
export async function reviewRegistrationApplication(
  id: number,
  body: { status: 'approved' | 'rejected'; reason?: string },
): Promise<ApiResponse & { id?: number; status?: string }> {
  return request(`/api/admin/registration-applications/${id}/review`, {
    method: 'POST',
    body: JSON.stringify({ status: body.status, reason: body.reason || undefined }),
  });
}

export interface BlockedUser {
  user_id: number;
  username: string;
  created_at: string;
}

export interface BlocksResponse extends ApiResponse {
  items?: BlockedUser[];
}

/** POST /api/blocks — block a user by id. Self-block → 400; idempotent. */
export async function blockUser(user_id: number): Promise<ApiResponse & { user_id?: number; blocked?: boolean }> {
  return request('/api/blocks', {
    method: 'POST',
    body: JSON.stringify({ user_id }),
  });
}

/** DELETE /api/blocks/{user_id} — unblock. Idempotent. */
export async function unblockUser(user_id: number): Promise<ApiResponse & { user_id?: number; blocked?: boolean }> {
  return request(`/api/blocks/${user_id}`, {
    method: 'DELETE',
  });
}

/** GET /api/blocks — list the caller's blocked users. */
export async function listBlocks(): Promise<BlocksResponse> {
  return request('/api/blocks');
}

export interface ForumEditProposal { id: number; target_type: 'topic' | 'post'; target_id: number; author_id: number; author_username: string | null; snapshot: Record<string, unknown>; status: 'pending' | 'approved' | 'rejected'; reviewer_username?: string | null; review_note?: string | null; created_at: string; }

export async function getForumEditHistory(targetType: 'topic' | 'post', targetId: number): Promise<ApiResponse & { items: ForumEditProposal[] }> {
  return request(`/api/forum/edits/${targetType}/${targetId}`);
}

export async function getForumEditQueue(): Promise<ApiResponse & { items: ForumEditProposal[] }> {
  return request('/api/forum/edits/queue');
}

export async function reviewForumEdit(proposalId: number, decision: 'approve' | 'reject', note?: string): Promise<ApiResponse & { status?: string }> {
  return request(`/api/forum/edits/${proposalId}/review`, { method: 'POST', body: JSON.stringify({ decision, note }) });
}

export interface ForumUserGrant { id: number; post_id: number; topic_id: number; reason: string; delta: number; score_after: number; created_at: string; }
export interface ForumUserGrantsResponse extends ApiResponse { user_id: number; items: ForumUserGrant[]; count: number; }
export async function getModerationUserGrants(userId: number): Promise<ForumUserGrantsResponse> {
  return request(`/api/forum/moderation/user/${userId}/grants`);
}


// ── Parity: cross-category views + prefs + tags + RSS ─────────────────────
export interface ForumUnreadResponse extends ApiResponse { items: ForumTopic[]; next_cursor: number | null; limit: number; }
export async function getUnreadTopics(limit?: number, cursor?: number | null): Promise<ForumUnreadResponse> {
  const p=new URLSearchParams(); if(limit) p.set('limit', String(limit)); if(cursor) p.set('cursor', String(cursor));
  const qs=p.toString(); return request(`/api/forum/unread${qs?`?${qs}`:''}`);
}
export async function getRecentTopics(limit?: number, cursor?: number | null, category?: string | null): Promise<ForumTopicsResponse & { items: ForumTopic[] }> {
  const p=new URLSearchParams(); if(limit) p.set('limit', String(limit)); if(cursor) p.set('cursor', String(cursor)); if(category) p.set('category', category);
  const qs=p.toString(); return request(`/api/forum/recent${qs?`?${qs}`:''}`);
}
export async function getPopularTopics(limit?: number, cursor?: number | null, sort?: string | null): Promise<ForumTopicsResponse & { items: ForumTopic[]; sort?: string }> {
  const p=new URLSearchParams(); if(limit) p.set('limit', String(limit)); if(cursor) p.set('cursor', String(cursor)); if(sort) p.set('sort', sort);
  const qs=p.toString(); return request(`/api/forum/popular${qs?`?${qs}`:''}`);
}
export interface ForumPrefs extends ApiResponse { posts_per_page: number; topic_sort: string; }
export async function getForumPrefs(): Promise<ForumPrefs> { return request('/api/forum/preferences'); }
export async function setForumPrefs(body: { posts_per_page?: number; topic_sort?: string }): Promise<ForumPrefs> {
  return request('/api/forum/preferences', { method: 'POST', body: JSON.stringify(body) });
}
export async function getTopicTags(topicId: number): Promise<ApiResponse & { tags: string[] }> { return request(`/api/forum/topics/${topicId}/tags`); }
export async function setTopicTags(topicId: number, tags: string[]): Promise<ApiResponse & { tags: string[] }> {
  return request(`/api/forum/topics/${topicId}/tags`, { method: 'POST', body: JSON.stringify({ tags }) });
}
export function forumRssUrl(feed: 'recent'|'popular'='recent', limit=20): string { return `/api/forum/rss?feed=${feed}&limit=${limit}`; }
