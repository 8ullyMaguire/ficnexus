//! Database query functions — split into domain modules.
//!
//! All functions are re-exported at `crate::db::queries::*` so existing
//! call sites keep compiling unchanged.

use chrono::{DateTime, Utc};
use sqlx::Row;

// ── Shared types ─────────────────────────────────────────────────────────────

/// A stored site-credential row (no password — never surfaced).
#[derive(Debug, Clone)]
pub struct SiteCredRow {
    pub domain: String,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

/// Per-follow notification payload for followed-work updates.
#[derive(Debug, sqlx::FromRow)]
pub struct FollowedWorkUpdate {
    pub follow_id: i64,
    pub work_id: i32,
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub fic_updated: chrono::DateTime<chrono::Utc>,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
}

/// A single reading-history entry (for analytics / history page).
#[derive(Debug, sqlx::FromRow)]
pub struct ReadingHistoryEntry {
    pub id: i64,
    pub work_id: i32,
    /// Canonical fic id (works.default_source_id, i.e. fic_info.id); empty string if unset.
    pub url_id: String,
    /// Display title: works.canonical_title falling back to fic_info.title.
    pub title: String,
    /// Display author: works.canonical_author falling back to fic_info.author.
    pub author: String,
    pub visited_at: chrono::DateTime<chrono::Utc>,
    pub chapter_num: Option<i32>,
}

/// Anonymous visitor state row (before registration).
#[derive(Debug, sqlx::FromRow)]
pub struct VisitorStateRow {
    pub visitor_id: uuid::Uuid,
    pub ratings: serde_json::Value,
    pub saved_searches: serde_json::Value,
    pub bookmarks: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ── Submodules ───────────────────────────────────────────────────────────────

pub mod core;
pub mod prefs;
pub mod proposals;
pub mod reputation;
pub mod social;
pub mod tags;
pub mod visitor;
pub mod works;

// ── Re-exports (all existing call sites stay as crate::db::queries::fn) ────────

// core
pub use core::{
    check_author_blacklist, check_fic_blacklist, create_curator_quorum_proposal,
    delete_user_site_credential, find_export_log, get_fic_info, get_fic_version_bump,
    get_user_site_credential, get_user_site_credentials, hide_fic_info, insert_export_log,
    insert_request_log, insert_request_source, insert_search_query, latest_export_log,
    recent_search_chips, search_similar_fics, set_site_credentials, upsert_fic_info,
};

// tags (includes tag queries + follow queries, as per the original file structure)
pub use tags::{
    count_unseen_followed_updates, create_tag, find_follow_for_author, find_follow_for_work,
    follow_author, follow_user, follow_work, get_fic_tags, get_followers, is_following_user,
    is_following_work, list_followed_work_updates, list_follows, lookup_alias, lookup_tag_by_name,
    mark_follow_seen, unfollow, upsert_fic_tag,
};

// social (notifications, badges, reading stats, analytics, translations, leaderboard, flags)
pub use social::{
    award_badge, check_and_award_badges, check_rate_limit, clear_read_history,
    compute_monthly_leaderboard, compute_weekly_leaderboard, create_follow_exclusion,
    create_notification, create_tag_alias, delete_follow_exclusion, delete_read_history,
    delete_tag, get_author_analytics, get_badge_definitions, get_existing_vote, get_locales,
    get_login_streak, get_monthly_leaderboard, get_notification_preferences,
    get_personal_reading_analytics, get_reading_history, get_ui_translations,
    get_unread_notification_count, get_user_badges, get_user_reading_aggregate,
    get_user_reading_stats, get_weekly_leaderboard, get_work_translation, insert_tag_flag,
    list_follow_exclusions, list_notifications, list_unresolved_flags, mark_all_notifications_read,
    mark_notification_read, merge_tags, notify_comment_reply, notify_work_followers,
    record_login_streak, record_read_history, record_work_read, resolve_flag,
    update_notification_preferences, upsert_tag_vote, upsert_work_translation,
};

// works
pub use works::{
    create_work, find_work_by_source_url, find_work_by_title_author, get_work, get_work_by_source,
    get_work_canonical_url_id, get_work_slug_target, get_work_sources, link_source_to_work,
    list_my_works, log_auto_merge,
};

// proposals
pub use proposals::{
    cast_proposal_vote, create_proposal, execute_merge, execute_split, get_proposal,
    get_proposal_vote_sum, is_senior_curator, list_pending_proposals, update_proposal_status,
};

// reputation (analytics, leaderboard, shelves, reading lists, collections, reading status)
pub use reputation::{
    add_collection_item, add_reading_list_item, add_work_to_shelf, approve_collection_item,
    bookmark_collection, browse_public_collections, create_collection, create_reading_list,
    create_shelf, delete_collection, delete_reading_list, delete_shelf, endpoint_group,
    get_active_users, get_collection, get_collection_by_slug, get_daily_stats,
    get_daily_unique_visitors, get_endpoint_group_usage, get_endpoint_usage, get_format_breakdown,
    get_monthly_unique_visitors, get_popular_fics, get_reading_list, get_reading_stats_list,
    get_recent_events, get_return_visitor_rate, get_shelf, get_total_events,
    get_total_unique_visitors, get_user_stats, get_view_only_users, get_weekly_unique_visitors,
    insert_usage_event, list_collection_bookmarkers, list_collection_item_requests,
    list_collection_item_requests_with_votes, list_reading_list_items, list_reading_lists,
    list_shelves, list_works_in_shelf, reject_collection_item, remove_collection_item,
    remove_reading_list_item, remove_work_from_shelf, unbookmark_collection, update_collection,
    update_reading_list, update_reading_status, update_reputation_and_promote,
    upsert_collection_submission_vote,
};

// prefs
pub use prefs::{ALLOWED_FORMATS, get_user_format_preferences, save_user_format_preferences};

// visitor
pub use visitor::{
    cleanup_stale_visitor_state, delete_visitor_state, get_visitor_state,
    merge_visitor_state_into_user, upsert_visitor_state,
};
