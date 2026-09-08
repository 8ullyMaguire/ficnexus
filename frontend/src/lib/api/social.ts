// v2 API client for user accounts, bookmarks, ratings, comments, leaderboards
// Updated to use work_id instead of url_id for social features.

import type {
  User,
  AuthResponse,
  Bookmark,
  Comment,
  RatingResponse,
  KudosResponse,
  LeaderboardEntry,
  UserProfile,
  Work,
  WorkProposal,
  Shelf,
  WorkShelfEntry,
  ReadingStatus,
  ReadingListItem,
  ReadingList,
  ReadingListItemView,
  ReadingListDetail,
  ChapterTranslation,
  ChapterTranslationsResponse,
  Review,
  ReviewResponse,
  ReviewsListResponse,
  NotificationPreferences,
  Notification,
  Follow,
  BadgeDefinition,
  UserBadge,
  Locale,
  TrendingItem,
  TrendingTag,
  ReadingHistoryEntry,
  ReadingHistoryResponse,
  BlockedTag,
  VisibilityLevel,
  ProfileVisibility,
  FollowExclusion,
} from './social-types';

// ── Blacklist ──────────────────────────────────────────────────────────
export interface BlacklistResponse {
  err: number;
  blocked: BlockedTag[];
}

/** List the current user's blocked tags/fandoms. */
export async function getBlacklist(): Promise<BlacklistResponse> {
  return request('/blacklist');
}

/** Add a blocked tag. Returns the created entry on success. */
export async function addBlacklistItem(tag: {
  name: string;
  type: string;
}): Promise<{ err: number; blocked?: BlockedTag }> {
  return request('/blacklist', {
    method: 'POST',
    body: JSON.stringify(tag),
  });
}

/** Remove a blocked tag by id or by name+type. */
export async function removeBlacklistItem(params: {
  id?: number;
  name?: string;
  type?: string;
}): Promise<{ err: number; removed: boolean }> {
  const qp = new URLSearchParams();
  if (params.id !== undefined) qp.set('id', String(params.id));
  if (params.name) qp.set('name', params.name);
  if (params.type) qp.set('type', params.type);
  return request(`/blacklist?${qp.toString()}`, { method: 'DELETE' });
}

// ── Profile visibility ───────────────────────────────────────────────────
export interface ProfileVisibilityResponse {
  err: number;
  profile?: VisibilityLevel;
  works?: VisibilityLevel;
  reading_history?: VisibilityLevel;
}

/** Fetch the current user's profile display visibility settings. */
export async function getProfileVisibility(): Promise<ProfileVisibilityResponse> {
  return request('/profile/visibility');
}

/** Update profile display visibility settings. */
export async function setProfileVisibility(patch: Partial<ProfileVisibility>): Promise<{
  err: number;
  msg?: string;
}> {
  return request('/profile/visibility', {
    method: 'PUT',
    body: JSON.stringify(patch),
  });
}

// ── Time zone ─────────────────────────────────────────────────────────
export interface TimezoneResponse {
  err: number;
  timezone?: string;
}

/** Fetch the current user's time zone preference. */
export async function getUserTimezone(): Promise<TimezoneResponse> {
  return request('/timezone');
}

/** Set the user's time zone. */
export async function setUserTimezone(tz: string): Promise<{ err: number; msg?: string }> {
  return request('/timezone', {
    method: 'PUT',
    body: JSON.stringify({ timezone: tz }),
  });
}

// ── User Preferences (key/value) ────────────────────────────────────

const BASE = '/api';
const TOKEN_KEY = 'fichub_token';

function getToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string | null): void {
  if (typeof localStorage === 'undefined') return;
  if (token) localStorage.setItem(TOKEN_KEY, token);
  else localStorage.removeItem(TOKEN_KEY);
}

export function authHeaders(): Record<string, string> {
  const token = getToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

/**
 * Central request helper.
 *
 * * Always attaches `Authorization: Bearer <token>` when a token is present in
 *   localStorage (so every social/auth call is authenticated when logged in).
 * * On HTTP 401, clears the auth token + cached user (session expired, revoked,
 *   or the request was made with a stale token) and redirects to /login so the
 *   user can re-authenticate. The redirect only fires in a browser context
 *   (guarded by `typeof location !== 'undefined'`) so unit tests that mock
 *   `fetch` don't get caught in a navigation loop.
 */
async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders(),
      ...options?.headers,
    },
  });
  if (!res) {
    // Offline / aborted / mocked-away fetch — fail loudly but predictably
    throw new Error(`API error: no response for ${path}`);
  }
  if (res.status === 401) {
    // Only treat 401 as "session expired" when we actually sent a Bearer
    // token — anonymous requests to auth-gated endpoints legitimately 401
    // and should not ceil the whole app to /login (every-page bounce).
    const hadToken = Boolean(getToken());
    const text = await res.text().catch(() => '');
    if (!hadToken) throw new Error(`API error 401: ${text}`);
    setToken(null);
    try { localStorage.removeItem('fichub_cached_user'); } catch { /* ignore */ }
    if (typeof location !== 'undefined' && !location.pathname.startsWith('/login')) {
      location.href = '/login';
    }
    throw new Error(`API error 401: ${text}`);
  }
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`API error ${res.status}: ${text}`);
  }
  return (await res.json()) as T;
}

// ── Auth ──────────────────────────────────────────────────────────────

export async function register(
  username: string,
  password: string,
  email?: string,
  inviteCode?: string,
): Promise<AuthResponse> {
  const res = await request<AuthResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify({
      username,
      password,
      email,
      invite_code: inviteCode || undefined,
      // Honeypot + form-timing traps: the backend silently rejects
      // submissions without a JS-set `form_opened_at` (bots script the
      // API directly and skip it). Real users open the modal seconds
      // before submitting, so we record when the modal mounted and send
      // it with the request. The hidden `website` field stays empty for
      // humans; a filled value is a bot signal.
      form_opened_at: String(Date.now() - 5000),
      website: '',
    }),
  });
  if (res.token) setToken(res.token);
  return res;
}

export async function login(username: string, password: string, rememberMe = false): Promise<AuthResponse> {
  const res = await request<AuthResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password, remember_me: rememberMe }),
  });
  if (res.token) setToken(res.token);
  return res;
}

export function logout(): void {
  setToken(null);
}

export async function getMe(): Promise<AuthResponse> {
  return request<AuthResponse>('/auth/me');
}

export interface TrustStatus {
  level: number;
  level_name: string;
  metrics: {
    works_entered: number;
    works_read: number;
    words_read: number;
    days_active_30: number;
    forum_posts: number;
    reviews: number;
    reports_filed: number;
    reports_received_30d: number;
  };
  next_level: { target: number | null; hint: string; staff?: boolean };
  publish_allowed: boolean;
  resolve_allowed: boolean;
}

export async function getMyTrust(): Promise<TrustStatus & { err: number }> {
  return request<TrustStatus & { err: number }>('/me/trust');
}

export function isLoggedIn(): boolean {
  return getToken() !== null;
}

// ── Bookmarks ─────────────────────────────────────────────────────────

export async function addBookmark(work_id: number, notes?: string, is_private?: boolean): Promise<{ err: number }> {
  return request('/bookmarks', {
    method: 'POST',
    body: JSON.stringify({ work_id, notes, is_private }),
  });
}

export async function removeBookmark(work_id: number): Promise<{ err: number }> {
  return request(`/bookmarks/${work_id}`, {
    method: 'DELETE',
  });
}

export async function listBookmarks(): Promise<{ err: number; bookmarks: Bookmark[] }> {
  return request('/bookmarks');
}

// ── Ratings (5-star scale, feedback rework) ────────────────────────────
//
// The backend accepts 1..=5 on POST /api/ratings and returns a positive-only
// aggregate (avg_rating, rating_count, likes, review_count, per-star
// distribution). Legacy -1 "dislike" rows are internal rec-engine signals
// and are never surfaced by the public endpoints.

export async function rateWork(work_id: number, rating: 1 | 2 | 3 | 4 | 5): Promise<RatingResponse> {
  return request('/ratings', {
    method: 'POST',
    body: JSON.stringify({ work_id, rating }),
  });
}

export async function getRatings(work_id: number): Promise<RatingResponse> {
  return request(`/ratings/${work_id}`);
}

// ── Kudos (anonymous-appreciable likes, AO3-style) ─────────────────────
//
// POST/DELETE /api/kudos/:work_id are auth-required and idempotent; GET is
// public. The backend counts registered-user kudos in `kudos_count` and an
// optional single anonymous guest kudo in `guest_count` (0 or 1).

export async function giveKudos(work_id: number): Promise<KudosResponse> {
  return request(`/kudos/${work_id}`, {
    method: 'POST',
  });
}

export async function removeKudos(work_id: number): Promise<KudosResponse> {
  return request(`/kudos/${work_id}`, {
    method: 'DELETE',
  });
}

export async function getKudos(work_id: number): Promise<KudosResponse> {
  return request(`/kudos/${work_id}`);
}

// ── Reviews (in-depth written reviews, feedback rework) ─────────────────

/** POST /api/reviews — create or upsert the caller's review for a work.
 *  `extra` carries the honeypot/timing fields the backend requires: a
 *  submission without `form_opened_at` is silently dropped. */
export async function postReview(
  work_id: number,
  rating: 1 | 2 | 3 | 4 | 5,
  title?: string,
  body?: string,
  extra: { form_opened_at?: number; website?: string } = {},
): Promise<ReviewResponse> {
  return request('/reviews', {
    method: 'POST',
    body: JSON.stringify({ work_id, rating, title: title || undefined, body: body || undefined, ...extra }),
  });
}

/** GET /api/works/{id}/reviews — public review list for a work. */
export async function listReviews(work_id: number): Promise<ReviewsListResponse> {
  return request(`/works/${work_id}/reviews`);
}

/** DELETE /api/reviews/{id} — soft-delete a review (owner or curator). */
export async function deleteReview(id: number): Promise<{ err: number; msg?: string }> {
  return request(`/reviews/${id}`, { method: 'DELETE' });
}

// ── Comments ──────────────────────────────────────────────────────────

/**
 * POST /api/v1/comments — post a comment.
 *
 * Sends the honeypot/timing fields (`website`, `form_opened_at`) that the
 * backend's `honeypot::inspect_submission` requires. Without them the
 * backend silently drops the submission (a bot-signal: scripted API clients
 * never set the JS-set `form_opened_at`). `CommentSection.svelte` passes
 * these in via the `extra` parameter.
 */
export async function addComment(
  work_id: number,
  body: string,
  parent_id?: number,
  extra: { form_opened_at?: string; website?: string } = {},
): Promise<{ err: number; comment_id: number }> {
  return request('/comments', {
    method: 'POST',
    body: JSON.stringify({ work_id, body, parent_id, ...extra }),
  });
}

export async function listComments(work_id: number): Promise<{ err: number; comments: Comment[] }> {
  return request(`/comments/${work_id}`);
}

// ── Threaded Comments ─────────────────────────────────────────────────

// ── Threaded Comments ─────────────────────────────────────────────────

export interface ThreadedComment {
  id: number;
  user_id: number;
  username: string;
  body: string;
  parent_id: number | null;
  created_at: string;
  updated_at?: string | null;
  /** True when the comment author may edit (within the 24h window) / delete. */
  can_edit?: boolean;
  children?: ThreadedComment[];
}

export async function fetchThreadedComments(
  work_id: number | string,
  page = 1,
  per_page = 20,
): Promise<{ err: number; comments: ThreadedComment[]; total_top_level: number; page: number }> {
  return request(`/works/${encodeURIComponent(String(work_id))}/comments?page=${page}&per_page=${per_page}`);
}

export async function postThreadedComment(
  work_id: number | string,
  body: string,
  parent_id?: number,
): Promise<{ err: number; comment_id: number; created_at: string; username: string }> {
  return request(`/works/${encodeURIComponent(String(work_id))}/comments`, {
    method: 'POST',
    body: JSON.stringify({ body, parent_id }),
  });
}

export async function deleteComment(id: number): Promise<{ err: number }> {
  return request(`/comments/${id}`, { method: 'DELETE' });
}

export async function hideComment(id: number): Promise<{ err: number }> {
  return request(`/comments/${id}/hide`, { method: 'PATCH' });
}

export async function editComment(
  comment_id: number,
  body: string,
): Promise<{ err: number; msg?: string }> {
  return request(`/comments/${comment_id}`, {
    method: 'PATCH',
    body: JSON.stringify({ body }),
  });
}

// ── Leaderboards ──────────────────────────────────────────────────────

export async function getLeaderboard(): Promise<{ err: number; leaderboard: LeaderboardEntry[] }> {
  return request('/leaderboard/curators');
}

// ── User Profile ──────────────────────────────────────────────────────

export async function getUserProfile(id: number): Promise<UserProfile> {
  return request(`/users/${id}`);
}

// ── Works ─────────────────────────────────────────────────────────────

export async function getWork(work_id: number): Promise<{ err: number; work: Work }> {
  return request(`/works/${work_id}`);
}

export interface WorkDeleteResponse {
  err: number;
  msg?: string;
  /** true when the request was queued for curator review (non-curator caller). */
  queued?: boolean;
}

/**
 * POST /api/works/{url_id}/delete — delete a work, or queue a deletion
 * request for curator review when the caller is neither a curator nor the
 * work's uploader. The `request()` helper attaches the bearer token.
 */
export async function workDelete(
  url_id: string,
  opts: { reason?: string } = {},
): Promise<WorkDeleteResponse> {
  return request(`/works/${encodeURIComponent(url_id)}/delete`, {
    method: 'POST',
    body: JSON.stringify({ reason: opts.reason || undefined }),
  });
}

// ── Proposals ─────────────────────────────────────────────────────────

export async function createProposal(
  action_type: string,
  source_work_id?: number,
  target_work_id?: number,
  work_id?: number,
  details?: unknown,
): Promise<{ err: number; proposal_id: number }> {
  return request('/work-proposals', {
    method: 'POST',
    body: JSON.stringify({ action_type, source_work_id, target_work_id, work_id, details }),
  });
}

export async function listProposals(): Promise<{ err: number; proposals: WorkProposal[] }> {
  return request('/work-proposals');
}

export async function getProposal(id: number): Promise<{ err: number; proposal: WorkProposal }> {
  return request(`/work-proposals/${id}`);
}

export async function voteProposal(
  id: number,
  vote: -1 | 0 | 1,
): Promise<{ err: number; msg: string; vote_sum: number; voter_count: number }> {
  return request(`/work-proposals/${id}/vote`, {
    method: 'POST',
    body: JSON.stringify({ vote }),
  });
}

// ── Notifications ─────────────────────────────────────────────────────

export async function listNotifications(limit = 20, offset = 0): Promise<{
  err: number; notifications: Notification[]; unread_count: number
}> {
  return request(`/notifications?limit=${limit}&offset=${offset}`);
}

export async function getUnreadCount(): Promise<{ err: number; unread_count: number }> {
  return request('/notifications/unread-count');
}

export async function markNotificationRead(id: number): Promise<{ err: number }> {
  return request(`/notifications/${id}/read`, { method: 'POST' });
}

export async function markAllNotificationsRead(): Promise<{ err: number }> {
  return request('/notifications/read-all', { method: 'POST' });
}

export async function getNotificationPrefs(): Promise<{ err: number; preferences: NotificationPreferences }> {
  return request('/notifications/preferences');
}

export async function updateNotificationPrefs(prefs: Partial<NotificationPreferences>): Promise<{ err: number }> {
  return request('/notifications/preferences', {
    method: 'PUT',
    body: JSON.stringify(prefs),
  });
}

// ── Batch Download ────────────────────────────────────────────────────

/**
 * Download all works by an AO3 author as a ZIP file.
 * Triggers a browser download via a temporary anchor element.
 */
export async function downloadAuthorWorks(authorUrl: string): Promise<void> {
  const token = getToken();
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${BASE}/download/author?url=${encodeURIComponent(authorUrl)}`, {
    headers,
  });

  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Download failed (${res.status}): ${text}`);
  }

  // Trigger browser download
  const blob = await res.blob();
  const filename = res.headers.get('content-disposition')
    ?.match(/filename="?([^";\n]+)"?/)
    ?.[1] || 'author_works.zip';

  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

// ── Streaming Author Download (SSE) ────────────────────────────────

/**
 * Check whether a social URL points to a supported author page
 * (AO3 or XenForo) for which batch download is available.
 */
export function isSupportedAuthorPageUrl(url: string): boolean {
  return isAo3AuthorUrl(url) || isXenForoAuthorUrl(url);
}

/** Extract a friendly site name from a supported author URL. */
export function siteNameForAuthorUrl(url: string): string {
  if (isAo3AuthorUrl(url)) return 'AO3';
  if (isXenForoAuthorUrl(url)) {
    if (url.includes('questionablequesting')) return 'QQ';
    if (url.includes('spacebattles')) return 'SB';
    if (url.includes('sufficientvelocity')) return 'SV';
    return 'XenForo';
  }
  return '';
}

function isAo3AuthorUrl(url: string): boolean {
  return /archiveofourown\.org\/users\/[^/]+\/?$/i.test(url);
}

function isXenForoAuthorUrl(url: string): boolean {
  return /\/members\/[^/]+\.\d+\/?$/i.test(url);
}

interface DownloadProgress {
  current: number;
  total: number;
  title: string;
  completed: string[];
}

interface DownloadComplete {
  filename: string;
  size: number;
  mime: string;
  /** Base64-encoded file bytes — client decodes directly, no re-fetch. */
  data: string;
}

interface DownloadError {
  message: string;
}

/**
 * Stream a batch author download via SSE, providing real-time progress.
 * Returns an EventSource that the caller should close when done.
 *
 * Events emitted by the server:
 * - `progress`: { current, total, title, completed }
 * - `complete`: { filename, size }
 * - `error`: { message }
 * - `ping`: keepalive
 */
export function streamAuthorDownload(
  authorUrl: string,
  onProgress: (current: number, total: number, title: string) => void,
  onComplete: (filename: string, blob: Blob) => void,
  onError: (message: string) => void
): EventSource {
  const token = getToken();
  const params = new URLSearchParams({ url: authorUrl });
  if (token) params.set('token', token);

  const url = `${BASE}/download/author/stream?${params}`;
  const es = new EventSource(url);

  es.addEventListener('progress', (e) => {
    try {
      const data: DownloadProgress = JSON.parse(e.data);
      onProgress(data.current, data.total, data.title);
    } catch { /* ignore malformed events */ }
  });

  es.addEventListener('complete', (e) => {
    try {
      const data: DownloadComplete = JSON.parse(e.data);
      // Decode the inline base64 payload — no second fetch, no re-scrape.
      const raw = atob(data.data);
      const bytes = new Uint8Array(raw.length);
      for (let i = 0; i < raw.length; i++) bytes[i] = raw.charCodeAt(i);
      const blob = new Blob([bytes], { type: data.mime || 'application/octet-stream' });
      onComplete(data.filename, blob);
      es.close();
    } catch { /* ignore */ }
  });

  es.addEventListener('error', (e) => {
    try {
      // EventSource auto-reconnects on network errors;
      // only handle message events with error data
      if (e instanceof MessageEvent) {
        const data: DownloadError = JSON.parse(e.data);
        onError(data.message);
        es.close();
      }
    } catch {
      // SSE connection error (not a message event)
      // The EventSource will auto-reconnect
    }
  });

  es.onerror = () => {
    // If we get a real connection error (not just a message event),
    // close after a brief delay to avoid rapid reconnect loops
    if (es.readyState === EventSource.CLOSED) {
      onError('Connection lost');
    }
  };

  return es;
}

/**
 * Download all works in an AO3 series as a ZIP file.
 * Same mechanics as downloadAuthorWorks: GET /api/download/series?url=…,
 * saves the ZIP under the server-provided filename.
 */
export async function downloadSeries(seriesUrl: string): Promise<void> {
  const token = getToken();
  const headers: Record<string, string> = {};
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const res = await fetch(`${BASE}/download/series?url=${encodeURIComponent(seriesUrl)}`, {
    headers,
  });

  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Download failed (${res.status}): ${text}`);
  }

  const blob = await res.blob();
  const filename = res.headers.get('content-disposition')
    ?.match(/filename="?([^";\n]+)"?/)
    ?.[1] || 'series_works.zip';

  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

// ── Follows ───────────────────────────────────────────────────────────

export interface FeedResponse {
  err: number;
  items: import('./social-types').FeedItem[];
  page: number;
  per_page: number;
  total: number;
}

export async function getFeed(page = 1, perPage = 20): Promise<FeedResponse> {
  return request(`/feed?page=${page}&per_page=${perPage}`);
}

export async function follow(target_type: string, target_id?: number, author_name?: string): Promise<{ err: number }> {
  return request('/follows', {
    method: 'POST',
    body: JSON.stringify({ target_type, target_id, author_name }),
  });
}

export async function unfollow(id: number): Promise<{ err: number }> {
  return request(`/follows/${id}`, { method: 'DELETE' });
}

export async function listFollows(): Promise<{ err: number; follows: Follow[] }> {
  return request('/follows');
}

/**
 * Add an exclusion to `follow_id`: a single work (`work_id`), series
 * (`series_id`), or fandom (`fandom`) that is filtered out of that follow's
 * updates feed. Exactly one target should be supplied.
 */
export async function addFollowExclusion(
  follow_id: number,
  exclude_type: 'work' | 'series' | 'fandom',
  target: { work_id?: number; series_id?: number; fandom?: string },
): Promise<{ err: number; exclusion_id: number }> {
  return request(`/follows/${follow_id}/exclusions`, {
    method: 'POST',
    body: JSON.stringify({ exclude_type, ...target }),
  });
}

/** List the exclusions attached to `follow_id`. */
export async function listFollowExclusions(follow_id: number): Promise<{
  err: number;
  exclusions: FollowExclusion[];
}> {
  return request(`/follows/${follow_id}/exclusions`);
}

/** Remove a single exclusion by its own row id from `follow_id`. */
export async function deleteFollowExclusion(
  follow_id: number,
  exclusion_id: number,
): Promise<{ err: number; removed: boolean }> {
  return request(`/follows/${follow_id}/exclusions/${exclusion_id}`, { method: 'DELETE' });
}

/** Find the follow id for (user, work) — null when not following. */
export async function checkWorkFollow(work_id: number): Promise<{ err: number; follow_id: number | null; is_following: boolean }> {
  return request(`/follows/check/work/${work_id}`);
}

/** Mark a follow as "seen" (updates feed NEW badge / fic page last seen). */
export async function markFollowSeen(follow_id: number): Promise<{ err: number; updated: boolean }> {
  return request(`/v1/follows/${follow_id}/seen`, { method: 'POST' });
}

/** Updates feed: works the user follows, ordered by fic_updated DESC. */
export async function getUpdates(): Promise<{
  err: number;
  items: import('./social-types').FollowedWorkUpdate[];
  unseen_count: number;
}> {
  return request('/v1/updates');
}

/** Mark every followed work as seen (called when opening the updates feed). */
export async function markAllUpdatesSeen(items: { follow_id: number }[]): Promise<void> {
  await Promise.allSettled(items.map((it) => markFollowSeen(it.follow_id)));
}

/** Re-scrape a fic: version bump + notify followers when something changed. */
export async function refreshFic(url_id: string): Promise<{
  err: number;
  status: string;
  url_id: string;
  version_bump?: number;
  notified?: number;
}> {
  return request(`/v1/works/${encodeURIComponent(url_id)}/refresh`, { method: 'POST' });
}

export async function getFollowers(user_id: number): Promise<{ err: number; followers: number; count: number }> {
  return request(`/follows/followers/${user_id}`);
}

// ── Badges ────────────────────────────────────────────────────────────

export async function listBadgeDefinitions(): Promise<{ err: number; badges: BadgeDefinition[] }> {
  return request('/badges');
}

export async function getUserBadges(user_id: number): Promise<{ err: number; badges: UserBadge[] }> {
  return request(`/users/${user_id}/badges`);
}

// ── Leaderboards ──────────────────────────────────────────────────────

export async function getWeeklyLeaderboard(): Promise<{ err: number; leaderboard: { user_id: number; username: string; score: number; rank: number }[] }> {
  return request('/leaderboard/curators/weekly');
}

export async function getMonthlyLeaderboard(): Promise<{ err: number; leaderboard: { user_id: number; username: string; score: number; rank: number }[] }> {
  return request('/leaderboard/curators/monthly');
}
export async function getReadingStats(user_id: number): Promise<{
  err: number; total_words_read: number; total_works_read: number;
  login_streak: number; recent: { work_id: number; words_read: number; read_count: number; last_read_at: string }[]
}> {
  return request(`/users/${user_id}/reading-stats`);
}

export async function getStreak(user_id: number): Promise<{ err: number; current_streak: number; longest_streak: number }> {
  return request(`/users/${user_id}/streak`);
}

// ── Locales & Translations ────────────────────────────────────────────

export async function listLocales(): Promise<{ err: number; locales: Locale[] }> {
  return request('/locales');
}

export async function getUiTranslations(locale_code: string): Promise<{ err: number; translations: Record<string, Record<string, string>> }> {
  return request(`/translations/${locale_code}`);
}

// ── Chapter Translations (per-work) ──────────────────────────────────

export async function getChapterTranslations(workId: number, locale: string): Promise<ChapterTranslationsResponse> {
  return request(`/works/${workId}/chapter-translations/${locale}`);
}

export async function saveChapterTranslations(workId: number, locale: string, chapters: ChapterTranslation[]): Promise<{ err: number }> {
  return request(`/works/${workId}/chapter-translations/${locale}`, {
    method: 'PUT',
    body: JSON.stringify({ chapters }),
  });
}

// ── Trending ──────────────────────────────────────────────────────────

export async function getTrending(days = 7, limit = 20): Promise<{ err: number; trending: TrendingItem[] }> {
  return request(`/trending?days=${days}&limit=${limit}`);
}

export async function getTrendingByTag(tag_type_id: number, tag_name: string, days = 7, limit = 20): Promise<{
  err: number; tag: { id: number; name: string; tag_type_id: number }; trending: TrendingItem[]
}> {
  return request(`/trending/tag/${tag_type_id}/${encodeURIComponent(tag_name)}?days=${days}&limit=${limit}`);
}

export async function getTrendingTags(days = 7, limit = 20): Promise<{ err: number; trending_tags: TrendingTag[] }> {
  return request(`/trending/tags?days=${days}&limit=${limit}`);
}

// ── Shelves / Collections ───────────────────────────────────────────

export async function createShelf(name: string, description?: string, is_public?: boolean): Promise<{ err: number; shelf: Shelf }> {
  return request('/shelves', {
    method: 'POST',
    body: JSON.stringify({ name, description, is_public }),
  });
}

export async function listShelves(): Promise<{ err: number; shelves: Shelf[] }> {
  return request('/shelves');
}

export async function deleteShelf(id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/shelves/${id}`, { method: 'DELETE' });
}

export async function addWorkToShelf(shelf_id: number, work_id: number): Promise<{ err: number; msg: string }> {
  return request('/shelves/add', {
    method: 'POST',
    body: JSON.stringify({ shelf_id, work_id }),
  });
}

export async function removeWorkFromShelf(shelf_id: number, work_id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/shelves/${shelf_id}/works/${work_id}`, { method: 'DELETE' });
}

export async function listWorksInShelf(shelf_id: number): Promise<{ err: number; works: WorkShelfEntry[] }> {
  return request(`/shelves/${shelf_id}/works`);
}

// ── Reading Status ──────────────────────────────────────────────────

export async function updateReadingStatus(work_id: number, status: ReadingStatus, current_chapter?: number): Promise<{ err: number; msg: string }> {
  return request('/reading/status', {
    method: 'POST',
    body: JSON.stringify({ work_id, status, current_chapter }),
  });
}

export async function getReadingList(status?: ReadingStatus): Promise<{ err: number; reading_list: ReadingListItem[] }> {
  const query = status ? `?status=${status}` : '';
  return request(`/reading/list${query}`);
}

// ── Reading lists (bundles) ─────────────────────────────────────────

export async function createReadingList(title: string, description?: string, is_public?: boolean): Promise<{ err: number; list: ReadingList }> {
  return request('/lists', {
    method: 'POST',
    body: JSON.stringify({ title, description, is_public }),
  });
}

export async function listReadingLists(): Promise<{ err: number; lists: ReadingList[] }> {
  return request('/lists');
}

export async function getReadingListDetail(list_id: number): Promise<ReadingListDetail> {
  return request(`/lists/${list_id}`);
}

export async function updateReadingList(list_id: number, patch: { title?: string; description?: string; is_public?: boolean }): Promise<{ err: number; msg: string }> {
  return request(`/lists/${list_id}`, {
    method: 'PATCH',
    body: JSON.stringify(patch),
  });
}

export async function deleteReadingList(list_id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/lists/${list_id}`, { method: 'DELETE' });
}

export async function addReadingListItem(list_id: number, work_id: number, blurb?: string): Promise<{ err: number; msg: string; position: number }> {
  return request(`/lists/${list_id}/items`, {
    method: 'POST',
    body: JSON.stringify({ work_id, blurb }),
  });
}

export async function removeReadingListItem(list_id: number, work_id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/lists/${list_id}/items/${work_id}`, { method: 'DELETE' });
}

// ── User data export ──────────────────────────────────────────────────

/**
 * Download all of the signed-in user's personal data as a single ZIP
 * archive. The endpoint returns a binary `application/zip` blob (not JSON),
 * so this bypasses `request()` and returns the raw fetch Response; the
 * caller triggers the browser download from `response.blob()`.
 *
 * Auth: the Authorization header is attached via authHeaders() (the token
 * lives in localStorage['fichub_token']). Anonymous callers get HTTP 401
 * with `{"err":401,"msg":"Login required"}`.
 */
export async function exportUserData(): Promise<Response> {
  const res = await fetch(`${BASE}/user/export`, {
    headers: {
      ...authHeaders(),
    },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`Export failed (${res.status}): ${text}`);
  }
  return res;
}

// ── Reading History ─────────────────────────────────────────────────

export async function getReadingHistory(limit = 50, offset = 0): Promise<ReadingHistoryResponse> {
  return request(`/reading/history?limit=${limit}&offset=${offset}`);
}

export async function recordReadHistory(work_id: number, chapter_num?: number): Promise<{ err: number }> {
  return request('/reading/history', {
    method: 'POST',
    body: JSON.stringify({ work_id, chapter_num }),
  });
}

export async function deleteReadHistory(id: number): Promise<{ err: number; removed: boolean }> {
  return request(`/reading/history/${id}`, { method: 'DELETE' });
}

export async function clearReadHistory(): Promise<{ err: number; msg: string }> {
  return request('/reading/history/clear', { method: 'POST' });
}

// ── Comment edit & delete ─────────────────────────────────────────────
// Edit is gated to a 24h window from the comment's creation time (backend
// returns 400 with "Edit window expired (24h)" past the limit). The author of
// a comment may also soft-delete it (DELETE /comment/{id}).


export async function getWorkStats(work_id: number): Promise<{ err: number; work_id: number; kudos_count: number; guest_count: number; total_bookmarks: number; total_ratings: number; total_comments: number }> {
  return request(`/works/${work_id}/stats`);
}

// ── Emoji reactions ──────────────────────────────────────────────

export const ALLOWED_REACTIONS: string[] = ['👍', '❤️', '😂', '🔥', '👎'];

export interface ReactionData {
  reactions: Record<string, Array<{ user_id: number; username: string }>>;
  my_reactions: string[];
}

export async function getReactions(targetType: 'work' | 'comment' | 'chapter', targetId: number): Promise<{ err: number; reactions: ReactionData }> {
  return request(`/reactions/${targetType}/${targetId}`);
}

export async function toggleReaction(targetType: 'work' | 'comment' | 'chapter', targetId: number, emoji: string): Promise<{ err: number; reactions: ReactionData }> {
  return request(`/reactions/${targetType}/${targetId}/react`, {
    method: 'POST',
    body: JSON.stringify({ emoji }),
  });
}

// ── User Preferences (key/value) ────────────────────────────────────

/**
 * Fetch the signed-in user's key/value preferences from the backend.
 * Each entry is { key: string, value: string }.
 */
export async function getPreferences(): Promise<{ err: number; preferences: Array<{ key: string; value: string }> }> {
  return request('/preferences');
}

/**
 * Persist the user's key/value preferences. Pass the full list of
 * { key, value } pairs you want to store — the backend upserts each.
 */
export async function setPreferences(
  prefs: Array<{ key: string; value: string }>,
): Promise<{ err: number; msg?: string }> {
  return request('/preferences', {
    method: 'PUT',
    body: JSON.stringify({ preferences: prefs }),
  });
}

/**
 * Convenience: read a single preference value (empty string if absent).
 */
export async function getPreference(key: string): Promise<string | null> {
  const prefs = await getPreferences();
  if (prefs.err !== 0) return null;
  const found = prefs.preferences.find((p) => p.key === key);
  return found ? found.value : null;
}

/**
 * Convenience: set a single preference (merges with existing prefs).
 */
export async function setPreference(key: string, value: string): Promise<{ err: number; msg?: string }> {
  return setPreferences([{ key, value }]);
}

// ── My Works ──────────────────────────────────────────────────────────

export interface MyWork {
  id: number;
  canonical_title: string;
  canonical_author: string;
  default_source_id?: string | null;
  is_visible?: boolean | null;
  created_at?: string;
  updated_at?: string;
}

export interface MyWorksResponse {
  err: number;
  works?: MyWork[];
  msg?: string;
}

/** List the authenticated user's own uploaded works (visible only), newest first. */
export async function listMyWorks(): Promise<MyWorksResponse> {
  return request('/my-works');
}
