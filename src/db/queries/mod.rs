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
pub mod tags;
pub mod social;
pub mod works;
pub mod proposals;
pub mod reputation;
pub mod prefs;
pub mod visitor;

// ── Re-exports (all existing call sites stay as crate::db::queries::fn) ────────

// core
pub use core::{
    set_site_credentials, get_user_site_credentials, get_user_site_credential,
    delete_user_site_credential, upsert_fic_info, hide_fic_info,
    create_curator_quorum_proposal, get_fic_info, insert_request_source,
    insert_request_log, insert_search_query, recent_search_chips, latest_export_log,
    find_export_log, insert_export_log, check_fic_blacklist, check_author_blacklist,
    get_fic_version_bump, search_similar_fics,
};

// tags (includes tag queries + follow queries, as per the original file structure)
pub use tags::{
    lookup_tag_by_name, lookup_alias, create_tag, upsert_fic_tag, get_fic_tags,
    follow_user, follow_work, follow_author, unfollow, list_follows, get_followers,
    is_following_user, is_following_work, find_follow_for_work, find_follow_for_author,
    mark_follow_seen, list_followed_work_updates, count_unseen_followed_updates,
};

// social (notifications, badges, reading stats, analytics, translations, leaderboard, flags)
pub use social::{
    create_follow_exclusion, list_follow_exclusions, delete_follow_exclusion,
    create_notification, get_unread_notification_count, list_notifications,
    mark_notification_read, mark_all_notifications_read, get_notification_preferences,
    update_notification_preferences, get_badge_definitions, get_user_badges,
    award_badge, check_and_award_badges, record_login_streak, get_login_streak,
    record_work_read, get_user_reading_stats, get_user_reading_aggregate,
    record_read_history, get_personal_reading_analytics, get_author_analytics,
    get_reading_history, delete_read_history, clear_read_history,
    get_locales, get_ui_translations, get_work_translation, upsert_work_translation,
    get_weekly_leaderboard, get_monthly_leaderboard,
    compute_weekly_leaderboard, compute_monthly_leaderboard,
    notify_comment_reply, notify_work_followers,
    upsert_tag_vote, get_existing_vote, insert_tag_flag,
    list_unresolved_flags, resolve_flag, check_rate_limit,
    create_tag_alias, merge_tags, delete_tag,
};

// works
pub use works::{
    create_work, get_work, list_my_works, get_work_by_source,
    find_work_by_source_url, find_work_by_title_author, link_source_to_work,
    get_work_sources, get_work_slug_target, get_work_canonical_url_id, log_auto_merge,
};

// proposals
pub use proposals::{
    create_proposal, list_pending_proposals, get_proposal, cast_proposal_vote,
    get_proposal_vote_sum, is_senior_curator, update_proposal_status,
    execute_merge, execute_split,
};

// reputation (analytics, leaderboard, shelves, reading lists, collections, reading status)
pub use reputation::{
    update_reputation_and_promote,
    get_daily_stats, get_user_stats, get_popular_fics, get_format_breakdown,
    get_total_unique_visitors, get_return_visitor_rate,
    insert_usage_event, get_daily_unique_visitors, get_weekly_unique_visitors,
    get_monthly_unique_visitors, get_active_users, get_view_only_users, get_total_events,
    endpoint_group, get_endpoint_usage, get_endpoint_group_usage, get_recent_events,
    create_shelf, list_shelves, get_shelf, delete_shelf,
    add_work_to_shelf, remove_work_from_shelf, list_works_in_shelf,
    create_reading_list, list_reading_lists, get_reading_list,
    update_reading_list, delete_reading_list, add_reading_list_item,
    remove_reading_list_item, list_reading_list_items,
    create_collection, browse_public_collections, get_collection, get_collection_by_slug,
    update_collection, delete_collection, add_collection_item,
    remove_collection_item, list_collection_item_requests,
    list_collection_item_requests_with_votes, upsert_collection_submission_vote,
    approve_collection_item, reject_collection_item,
    bookmark_collection, unbookmark_collection, list_collection_bookmarkers,
    update_reading_status, get_reading_stats_list,
};

// prefs
pub use prefs::{
    ALLOWED_FORMATS, get_user_format_preferences, save_user_format_preferences,
};

// visitor
pub use visitor::{
    get_visitor_state, upsert_visitor_state, delete_visitor_state,
    cleanup_stale_visitor_state, merge_visitor_state_into_user,
};
