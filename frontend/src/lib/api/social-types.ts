// v2 API types for user accounts, bookmarks, ratings, comments, leaderboards

export interface User {
  id: number;
  username: string;
  role: number;
  reputation: number;
  email?: string | null;
  locale?: string;
  /** F7 site-wide level (0-100). Present on fresh auth responses; absent on
   *  cached/legacy users — derive from `role` until a fresh /me lands. */
  level?: number;
  /** F7 experience points (drives level). */
  exp?: number;
}

export interface AuthResponse {
  err: number;
  token?: string;
  user?: User;
  msg?: string;
}

export interface Bookmark {
  work_id: number;
  notes: string;
  is_private: boolean;
  created_at: string;
}

export interface CommentUser {
  id: number;
  username: string;
}

export interface Comment {
  id: number;
  work_id: number;
  url_id?: string;
  parent_id: number | null;
  body: string;
  user: CommentUser | null;
  created_at: string;
  updated_at: string | null;
  deleted: boolean;
  hidden: boolean;
  reply_count: number;
}

export interface RatingResponse {
  err: number;
  work_id?: number;
  /** Average star rating over the 1..=5 scale (0.0 when no star ratings). */
  avg_rating: number;
  /** Number of 1..=5 star ratings (excludes legacy internal -1 rows). */
  rating_count: number;
  /** Positive-only public signal: 5-star ratings + written reviews. */
  likes: number;
  /** Number of non-deleted written reviews (constructive only). */
  review_count: number;
  /** Per-star counts keyed "1".."5". */
  rating_distribution: Record<string, number>;
}

/** Kudos (AO3-style anonymous-appreciable likes) for a work.
 *  `kudos_count` = registered users who kudo'd; `guest_count` = 0 or 1
 *  anonymous guest kudo. `my_kudos` is false when the caller is anonymous. */
export interface KudosResponse {
  err: number;
  work_id: number;
  kudos_count: number;
  guest_count?: number;
  my_kudos: boolean;
}

// ── Reviews (feedback rework: in-depth written reviews) ───────────────

export interface Review {
  id: number;
  work_id: number;
  username: string;
  /** 1..=5 stars. */
  rating: number;
  title: string | null;
  body: string;
  constructive: boolean;
  created_at: string;
  updated_at: string;
}

export interface ReviewResponse {
  err: number;
  review: Review;
}

export interface ReviewsListResponse {
  err: number;
  work_id: number;
  total: number;
  reviews: Review[];
}

export interface LeaderboardEntry {
  id: number;
  username: string;
  reputation: number;
}

export interface UserProfile {
  err: number;
  user: {
    id: number;
    username: string;
    role: number;
    reputation: number;
    email: string | null;
    created_at: string;
    badges: number;
  };
  avatar_url?: string | null;
  bio?: string | null;
  badge_text?: string | null;
  is_owner?: boolean;
}

// ── Work types ──────────────────────────────────────────────────────────

// ── Feed / Following Feed ──────────────────────────────────────────────

export interface FeedItem {
  work_id: number;
  url_id: string;
  title: string;
  author: string;
  updated_at: string;
  format: string;
  url: string;
}

// ── Trending ──────────────────────────────────────────────────────────

export interface WorkSource {
  id: string;          // url_id hash
  source: string;      // e.g. "ao3", "ffn"
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  url: string;
}

export interface Work {
  id: number;
  canonical_title: string;
  canonical_author: string;
  description: string;
  default_source_id: string | null;
  created_at: string;
  updated_at: string;
  sources: WorkSource[];
  total_bookmarks: number;
  total_ratings: number;
  total_comments: number;
}

export interface WorkProposal {
  id: number;
  proposer_id: number;
  action_type: string;
  source_work_id: number | null;
  target_work_id: number | null;
  work_id: number | null;
  details: unknown;
  status: string;
  created_at: string;
  closed_at: string | null;
  vote_sum: number;
  voter_count: number;
}

// ── New feature types ─────────────────────────────────────────────────

export interface Notification {
  id: number;
  notification_type: string;
  title: string;
  body: string | null;
  link: string | null;
  reference_type: string | null;
  reference_id: string | null;
  is_read: boolean;
  created_at: string;
}

export interface NotificationPreferences {
  comment_reply: boolean;
  follow_update: boolean;
  work_update: boolean;
  badge_earned: boolean;
  curator_promotion: boolean;
  recommendation: boolean;
  email_digest: string;
  /** Notify when someone comments on a work I authored. */
  comments_on_work: boolean;
  /** Notify when someone replies to a comment I wrote. */
  replies_to_comments: boolean;
  /** Notify when someone kudos (likes) a work I authored. */
  kudos_on_work: boolean;
  /** Notify when someone bookmarks a work I authored. */
  bookmarks_on_work: boolean;
  /** Notify when someone follows me or my work. */
  follows: boolean;
  /** Notify when someone mentions me (@username). */
  mentions: boolean;
}

export interface Follow {
  id: number;
  followee_id: number | null;
  work_id: number | null;
  author_name: string | null;
  created_at: string;
  last_seen: string | null;
}

/**
 * One exclusion attached to a follow — a work, series, or fandom that should
 * be filtered out of the follow's updates feed. Exactly one target is set:
 * `work_id`, `series_id`, or `fandom`.
 */
export interface FollowExclusion {
  id: number;
  follow_id: number;
  /** 'work' | 'series' | 'fandom' */
  exclude_type: string;
  work_id: number | null;
  series_id: number | null;
  fandom: string | null;
  created_at: string;
}

// ── Blacklist & profile preferences ─────────────────────────────────
export interface BlockedTag {
  id?: number;
  name: string;
  /** 'fandom' | 'tag' | 'freeform' | 'character' | 'relationship' | 'warning' | 'category' */
  type: string;
}

export type VisibilityLevel = 'public' | 'followers' | 'private';

export interface ProfileVisibility {
  profile: VisibilityLevel;
  works: VisibilityLevel;
  reading_history: VisibilityLevel;
}

/** One row of the updates feed (GET /api/v1/updates). */
export interface FollowedWorkUpdate {
  follow_id: number;
  work_id: number;
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  fic_updated: string;
  updated_ago: string;
  is_new: boolean;
}

export interface BadgeDefinition {
  badge_type: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  threshold: number;
}

export interface UserBadge {
  id: number;
  badge_type: string;
  earned_at: string;
}

export interface TrendingItem {
  url_id: string;
  title: string;
  author: string;
  words: number;
  chapters: number;
  status: string;
  requests: number;
  downloads: number;
}

export interface TrendingTag {
  id: number;
  name: string;
  tag_type_id: number;
  type_name: string;
  fic_count: number;
}

export interface Locale {
  code: string;
  name: string;
  is_rtl: boolean;
}

export interface LoginStreak {
  current_streak: number;
  longest_streak: number;
  last_login_date: string;
}

// ── Shelf / Collection types ──────────────────────────────────────

export interface Shelf {
  id: number;
  name: string;
  description: string;
  is_public: boolean;
  sort_order: number;
  created_at: string;
}

export interface WorkShelfEntry {
  id: number;
  shelf_id: number;
  work_id: number;
  added_at: string;
}

// ── Chapter Translation types ──────────────────────────────────────

export interface ChapterTranslation {
  chapter_num: number;
  title: string;
  content_html: string;
}

export interface ChapterTranslationsResponse {
  err: number;
  work_id: number;
  locale_code: string;
  chapters: ChapterTranslation[];
  translated_by?: number;
  created_at?: string;
}

// ── Reading Status types ──────────────────────────────────────────

export type ReadingStatus = 'want_to_read' | 'reading' | 'completed' | 'dropped';

export interface ReadingListItem {
  id: number;
  work_id: number;
  words_read: number;
  read_count: number;
  status: ReadingStatus;
  current_chapter: number | null;
  last_read_at: string;
}

// ── Reading history types ──────────────────────────────────────────

export interface ReadingHistoryEntry {
  id: number;
  work_id: number;
  url_id: string;
  title: string;
  author: string;
  chapter_num: number | null;
  visited_at: string;
}

export interface ReadingHistoryResponse {
  err: number;
  history: ReadingHistoryEntry[];
  total: number;
  limit: number;
  offset: number;
}

export interface ReadingList {
  id: number;
  user_id: number;
  title: string;
  description: string;
  is_public: boolean;
  created_at: string;
  updated_at: string;
  item_count: number;
}

/** A work inside a reading list, with its position + optional blurb. */
export interface ReadingListItemView {
  id: number;
  work_id: number;
  position: number;
  blurb: string;
  title: string;
  author: string;
}

export interface ReadingListDetail {
  err: number;
  list: ReadingList;
  items: ReadingListItemView[];
  is_owner: boolean;
}

export interface WorkStats {
  work_id: number;
  total_bookmarks: number;
  total_ratings: number;
  total_comments: number;
  kudos_count: number;
  guest_count: number;
  avg_rating?: number;
  total_views?: number;
}
