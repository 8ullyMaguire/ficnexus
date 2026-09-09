use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Fanfiction metadata as stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
    pub work_id: Option<i32>,
}

/// Request source tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestSource {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub is_automated: Option<bool>,
    pub route: Option<String>,
    pub description: Option<String>,
}

/// Request log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestLog {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub source_id: Option<i64>,
    pub etype: String,
    pub query: String,
    pub info_request_ms: i32,
    pub url_id: Option<String>,
    pub fic_info: Option<String>,
    pub export_ms: Option<i32>,
    pub export_file_name: Option<String>,
    pub export_file_hash: Option<String>,
    pub url: Option<String>,
}

/// Export log entry (cache tracking)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}

/// Fanfiction blacklist entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicBlacklist {
    pub url_id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}

/// Author blacklist entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthorBlacklist {
    pub source_id: i64,
    pub author_id: i64,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}

/// Version bump for cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicVersionBump {
    pub id: String,
    pub value: Option<i32>,
}

/// Canonical story entry (unified work)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkRow {
    pub id: i32,
    pub canonical_title: String,
    pub canonical_author: String,
    pub description: String,
    pub default_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader_id: Option<i32>,
    pub is_visible: Option<bool>,
}

/// Auto-merge log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AutoMergeLog {
    pub id: i32,
    pub source_url: String,
    pub matched_work_id: i32,
    pub confidence: f64,
    pub created_at: DateTime<Utc>,
}

/// Work proposal (merge or split)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkProposal {
    pub id: i32,
    pub proposer_id: i32,
    pub action_type: String,
    pub source_work_id: Option<i32>,
    pub target_work_id: Option<i32>,
    pub work_id: Option<i32>,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Vote on a work proposal
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkProposalVote {
    pub proposal_id: i32,
    pub user_id: i32,
    pub vote: i16,
    pub voted_at: DateTime<Utc>,
}

// ── Follows ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Follow {
    pub id: i64,
    pub follower_id: i32,
    pub followee_id: Option<i32>,
    pub work_id: Option<i32>,
    pub author_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

/// One exclusion attached to a follow (drops a work / series / fandom from
/// the follow's updates feed). Exactly one of the target fields is set.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FollowExclusion {
    pub id: i64,
    pub follow_id: i64,
    pub exclude_type: String,
    pub exclude_work_id: Option<i32>,
    pub exclude_series_id: Option<i32>,
    pub exclude_fandom: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ── Notifications ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: i64,
    pub user_id: i32,
    pub notification_type: String,
    pub title: String,
    pub body: Option<String>,
    pub link: Option<String>,
    pub reference_type: Option<String>,
    pub reference_id: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPreference {
    pub user_id: i32,
    pub comment_reply: bool,
    pub follow_update: bool,
    pub work_update: bool,
    pub badge_earned: bool,
    pub curator_promotion: bool,
    pub recommendation: bool,
    pub email_digest: String,
    pub updated_at: DateTime<Utc>,
    // ── Granular AO3-style toggles (migration 060) ─────────────────────
    /// Notify when someone comments on a work I authored.
    pub comments_on_work: bool,
    /// Notify when someone replies to a comment I wrote.
    pub replies_to_comments: bool,
    /// Notify when someone kudos (likes) a work I authored.
    pub kudos_on_work: bool,
    /// Notify when someone bookmarks a work I authored.
    pub bookmarks_on_work: bool,
    /// Notify when someone follows me or my work.
    pub follows: bool,
    /// Notify when someone mentions me (@username).
    pub mentions: bool,
}

// ── Badges ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BadgeDefinition {
    pub badge_type: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
    pub threshold: i32,
    pub event_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserBadge {
    pub id: i64,
    pub user_id: i32,
    pub badge_type: String,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginStreak {
    pub user_id: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_login_date: NaiveDate,
    pub updated_at: DateTime<Utc>,
}

// ── Reading Stats ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingStats {
    pub id: i64,
    pub user_id: i32,
    pub work_id: i32,
    pub words_read: i64,
    pub last_read_at: DateTime<Utc>,
    pub read_count: i32,
    pub status: String,
    pub current_chapter: Option<i32>,
}

// ── Shelves / Collections ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Shelf {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: String,
    pub is_public: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkShelf {
    pub id: i64,
    pub shelf_id: i32,
    pub work_id: i32,
    pub added_at: DateTime<Utc>,
}

// ── Reading lists ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingList {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub description: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A collection (AO3-style) — a reading_lists row enriched with the
/// collection-specific columns from migration 061 plus the resolved
/// owner username and live counters.
#[derive(Debug, Clone, FromRow)]
pub struct CollectionInfo {
    pub id: i32,
    pub user_id: i32,
    pub owner_username: String,
    pub title: String,
    pub description: String,
    pub visibility: String,
    pub item_selection: String,
    pub icon: String,
    pub slug: Option<String>,
    pub collection_kind: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub item_count: i64,
    pub bookmark_count: i64,
    pub is_bookmarked: bool,
}

/// A pending collection item request row.
/// Used by the moderated item-add workflow.
#[derive(Debug, Clone, FromRow)]
pub struct CollectionItemRequestRow {
    pub id: i64,
    pub list_id: i32,
    pub work_id: i32,
    pub requested_by: i32,
    pub requested_by_username: Option<String>,
    pub blurb: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub canonical_title: String,
    pub canonical_author: String,
}

/// A collection add-request joined with its collection title, work metadata,
/// and live community vote tallies for the viewing user. `my_vote` is the
/// viewer's own signal (-1/0/1); it is 0 when they have not voted yet.
#[derive(Debug, Clone, FromRow)]
pub struct CollectionItemRequestVoteRow {
    pub id: i64,
    pub list_id: i32,
    pub list_title: String,
    pub work_id: i32,
    pub requested_by: i32,
    pub requested_by_username: Option<String>,
    pub blurb: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub canonical_title: String,
    pub canonical_author: String,
    pub votes_for: i32,
    pub votes_against: i32,
    pub my_vote: i16,
}

/// A reading-list item joined with its work's title/author (for listing).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingListItemRow {
    pub id: i32,
    pub list_id: i32,
    pub work_id: i32,
    pub position: i32,
    pub blurb: String,
    pub created_at: DateTime<Utc>,
    pub canonical_title: String,
    pub canonical_author: String,
}

// ── Translations ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Locale {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub is_rtl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Translation {
    pub id: i64,
    pub locale_code: String,
    pub namespace: String,
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkTranslation {
    pub id: i64,
    pub work_id: i32,
    pub locale_code: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub translated_by: Option<i32>,
    pub translated_at: DateTime<Utc>,
}

// ── User site credentials ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSiteCredential {
    pub id: i64,
    pub user_id: i32,
    pub domain: String,
    pub username: String,
    pub password_enc: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// ── Leaderboard entries ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardWeekly {
    pub id: i64,
    pub user_id: i32,
    pub score: i32,
    pub rank: i16,
    pub week_start: NaiveDate,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeaderboardMonthly {
    pub id: i64,
    pub user_id: i32,
    pub score: i32,
    pub rank: i16,
    pub month_start: NaiveDate,
    pub computed_at: DateTime<Utc>,
}

/// A feature that can be gated, toggled, and pinned by users.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feature {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub long_help: String,
    pub icon: Option<String>,
    pub category: String,
    pub gate_type: String,
    pub gate_value: i32,
    pub requires_feature: Option<String>,
    pub is_default: bool,
    pub is_revocable: bool,
    pub admin_only: bool,
    pub widget_component: Option<String>,
    pub nav_target: Option<String>,
    pub sort_hint: i32,
    pub created_at: DateTime<Utc>,
}

/// A user's relationship to a feature: unlocked, enabled, pinned.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserFeature {
    pub user_id: i32,
    pub feature_id: i32,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub enabled_at: Option<DateTime<Utc>>,
    pub pinned: bool,
    pub sort_order: i32,
}

/// A single user preference stored as a JSONB value.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPref {
    pub user_id: i32,
    pub key: String,
    pub value: serde_json::Value,
}

/// A per-page dashboard layout stored as JSONB.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLayout {
    pub user_id: i32,
    pub page: String,
    pub layout: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// A saved search / filter view that the user can pin and switch between.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserView {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub query: serde_json::Value,
    pub pinned: bool,
}
