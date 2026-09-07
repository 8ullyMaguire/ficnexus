//! Forum — community categories, topic listing (F2 READ side) + the F3 WRITE
//! path (topic detail/create/edit/delete, posts/replies, follow, notifications).
//!
//! Implements the forum API (see docs/FORUM-API-CONTRACT.md):
//!
//! * `GET   /api/forum/categories`          — visible categories + topic counts
//! * `GET   /api/forum/topics?category=&cursor=&limit=` — cursor-paginated topic list
//! * `POST  /api/forum/categories`          — create category (role ≥ 10)
//! * `PATCH /api/forum/categories/{id}`     — edit/reorder category (role ≥ 10)
//! * `GET   /api/forum/topics/{id}`         — topic detail (OP + posts, cursor, view_count)
//! * `POST  /api/forum/topics`              — create topic + OP post (one tx)
//! * `PATCH /api/forum/topics/{id}`         — edit title/body (author ≤ 15 min or mod)
//! * `DELETE /api/forum/topics/{id}`        — soft-delete (author or mod; mod → modlog)
//! * `POST  /api/forum/topics/{id}/posts`   — reply (+ follower/mention notifications)
//! * `PATCH /api/forum/posts/{id}`          — edit post body (author ≤ 15 min or mod)
//! * `DELETE /api/forum/posts/{id}`         — soft-delete (author or mod; mod → modlog)
//! * `POST  /api/forum/topics/{id}/follow`  — toggle follow (returns following + count)
//! * `GET   /api/forum/topics/{id}/follow`  — read follow state + follower count
//! * `POST  /api/forum/topics/{id}/read`    — mark topic read (upsert read state)
//! * `GET   /api/forum/search?q=&category=` — FTS over topics + posts (snippets)
//! * `GET   /api/forum/moderation/status` — my mod points / window / eligibility
//! * `GET   /api/forum/moderation/queue`  — posts needing moderation (F5)
//! * `POST  /api/forum/posts/{postId}/moderate` — spend 1 point, adjust score
//! * `GET   /api/forum/posts/{postId}/moderations` — public mod history
//! * `POST  /api/admin/forum/hide/{postId}` — fast-hide a post 72h (role ≥ 5)
//! * `POST  /api/admin/forum/topics/{topicId}/lock|pin` — status toggles (role ≥ 5)
//! * `POST  /api/admin/forum/bans` / `DELETE /api/admin/forum/bans/{id}` /
//!   `GET /api/admin/forum/bans` — ban management (role ≥ 5 / ≥ 10)
//!
//! Search vector maintenance: every write that changes searchable text sets
//! `search_vector = to_tsvector('english', ...)` inline (create/update topic,
//! create/update post); migration 042 backfills pre-existing NULL rows.
//! Config name must stay in sync with the GIN indexes from migration 041.
//!
//! The forum-core crate owns the domain model (`forum_core::model`); the
//! handlers here do thin sqlx query work against the forum tables (created by
//! migration 041).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Max topics returned per page (API contract: default 25, max 100).
const DEFAULT_LIMIT: i64 = 25;
const MAX_LIMIT: i64 = 100;

/// FicNexus roles (migration 007): 0 reader, 5 curator, 10 admin.
/// Legacy read-only after F7 — the ACTIVE gate is site-wide level:
/// curator = level ≥ 50, admin = level ≥ 100 (config, env-overridable).
const MOD_ROLE: i16 = 5;

/// Curator level threshold (F7). Env-overridable via FORUM_CURATOR_LEVEL.
fn curator_level() -> i16 {
    std::env::var("FORUM_CURATOR_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
}

/// Admin level threshold (F7). Env-overridable via FORUM_ADMIN_LEVEL.
fn admin_level() -> i16 {
    std::env::var("FORUM_ADMIN_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

/// Exp needed per level (F7). Env-overridable via FORUM_EXP_PER_LEVEL.
fn exp_per_level() -> i64 {
    std::env::var("FORUM_EXP_PER_LEVEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
}

/// Compute the site-wide level from experience (level = exp / per_level,
/// capped at 100). Never decreases below 0.
fn level_for_exp(exp: i64) -> i16 {
    let per = exp_per_level().max(1);
    ((exp / per).clamp(0, 100)) as i16
}

/// Award exp to a user with an idempotent audit trail (exp_events) and a
/// level-up notification when the level changes. Best-effort: failures are
/// logged, never fail the caller. The `dedupe` reference (reference_type +
/// reference_id) makes re-awards a no-op.
async fn award_exp(
    db: &sqlx::PgPool,
    user_id: i32,
    amount: i64,
    event_type: &str,
    reference_type: Option<&str>,
    reference_id: Option<i64>,
) {
    let res = sqlx::query_as::<_, (i64, i16)>(
        "WITH upd AS (
           UPDATE users SET exp = exp + $2, level = LEAST(100, (exp + $2) / $3::bigint)
           WHERE id = $1
           RETURNING exp, level
         )
         SELECT exp, level FROM upd",
    )
    .bind(user_id)
    .bind(amount)
    .bind(exp_per_level())
    .fetch_optional(db)
    .await;

    let Ok(Some((new_exp, new_level))) = res else {
        tracing::warn!("award_exp: no user {user_id} or db error");
        return;
    };

    if let Err(e) = sqlx::query(
        "INSERT INTO exp_events (user_id, amount, event_type, reference_type, reference_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(amount)
    .bind(event_type)
    .bind(reference_type)
    .bind(reference_id)
    .execute(db)
    .await
    {
        tracing::warn!("award_exp: exp_events insert failed: {e}");
    }

    // Level-up notification: compute the pre-award level from the new exp.
    let pre_level = level_for_exp(new_exp - amount);
    if new_level > pre_level && pre_level >= 0 {
        if let Err(e) = sqlx::query(
            "INSERT INTO notifications (user_id, notification_type, title, body, created_at)
             VALUES ($1, 'level_up', $2, $3, NOW())",
        )
        .bind(user_id)
        .bind(format!("Level {new_level}!"))
        .bind(format!("You reached level {new_level} ({new_exp} exp)."))
        .execute(db)
        .await
        {
            tracing::warn!("award_exp: level-up notification failed: {e}");
        }
    }
}

/// Exp awarded for creating a forum post or topic (SPEC §10): +2.
async fn award_post_exp(db: &sqlx::PgPool, user_id: i32, post_id: i64) {
    award_exp(
        db,
        user_id,
        2,
        "forum_post_create",
        Some("forum_post"),
        Some(post_id),
    )
    .await;
}

/// Exp awarded to a post author when a moderator action is positive (SPEC
/// §10): +1, capped at +3/day (mod-received).
async fn award_mod_received_exp(db: &sqlx::PgPool, user_id: i32, modlog_id: i64) {
    // Daily cap: count today's positive mod-received events for this user.
    let today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM exp_events
         WHERE user_id = $1 AND event_type = 'mod_received'
           AND created_at >= CURRENT_DATE",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .unwrap_or(0);
    if today >= 3 {
        return;
    }
    award_exp(
        db,
        user_id,
        1,
        "mod_received",
        Some("forum_moderation"),
        Some(modlog_id),
    )
    .await;
}

/// GET /api/users/me/level — leveling progress (F7). Requires login.
pub async fn my_level(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let row: Option<(i16, i64)> = sqlx::query_as("SELECT level, exp FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;
    let (level, exp) = row.ok_or_else(|| AppError::BadRequest("user not found".to_string()))?;
    let per = exp_per_level().max(1);
    let next_level_exp = i64::from(level.saturating_add(1)) * per;
    let cur_level_exp = i64::from(level) * per;
    let progress = if level >= 100 {
        1.0
    } else {
        (exp - cur_level_exp) as f64 / (next_level_exp - cur_level_exp).max(1) as f64
    };
    Ok(Json(json!({
        "err": 0,
        "level": level,
        "exp": exp,
        "exp_to_next": (next_level_exp - exp).max(0),
        "progress": progress.clamp(0.0, 1.0),
        "level_up": false,
    })))
}

// ── Request bodies ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCategoryBody {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryBody {
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListTopicsParams {
    pub category: String,
    #[serde(default)]
    pub cursor: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopicDetailParams {
    #[serde(default)]
    pub after: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default = "default_true")]
    pub inc_views: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct CreateTopicBody {
    pub title: String,
    pub category_slug: String,
    pub body: String,
    #[serde(default)]
    pub payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTopicBody {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostBody {
    pub body: String,
    #[serde(default)]
    pub payload: Option<Value>,
    #[serde(default)]
    pub quote_of: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostBody {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct MarkReadBody {
    #[serde(default)]
    pub last_read_post_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchForumParams {
    pub q: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ModeratePostBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBanBody {
    pub user_id: i32,
    /// "forum" (global, role ≥ 10) or "category" (role ≥ 5).
    pub scope: String,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub reason: Option<String>,
    /// RFC3339 string or null (null = permanent ban).
    #[serde(default)]
    pub expires_at: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Require a logged-in user; 401 otherwise (repo convention).
fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}

/// Public-read gate for forum content. When `FORUM_PUBLIC_READ=false` the
/// whole forum is login-only (private-instance hosting). When enabled (the
/// default), anonymous visitors may read public (non-mod-only) content and
/// this helper returns the caller's user id (0 for anon) — the caller must
/// then enforce per-category `is_mod_only` themselves.
fn public_reader_id(auth: &AuthUser, state: &AppState) -> Result<i32, AppError> {
    if !state.config.forum_public_read {
        return require_user(auth);
    }
    Ok(auth.user_id.unwrap_or(0))
}

/// Require admin (level ≥ FORUM_ADMIN_LEVEL, default 100); 403 otherwise.
/// Legacy role is ignored here — level is the active gate after F7.
fn require_admin(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.level < admin_level() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}

/// Validate a category slug: lowercase alphanumeric + hyphens, 3-60 chars.
fn validate_slug(slug: &str) -> Result<String, AppError> {
    let s = slug.trim().to_lowercase();
    let len = s.chars().count();
    if !(3..=60).contains(&len) {
        return Err(AppError::BadRequest(
            "slug must be 3-60 characters".to_string(),
        ));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(AppError::BadRequest(
            "slug must be lowercase alphanumeric + hyphens".to_string(),
        ));
    }
    Ok(s)
}

/// Trim + cap a title at 120 chars, description at 1000 chars.
fn validate_category_fields(title: &str, description: &str) -> Result<(String, String), AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_string()));
    }
    if title.chars().count() > 120 {
        return Err(AppError::BadRequest("title too long (max 120)".to_string()));
    }
    let description = description.trim();
    if description.chars().count() > 1000 {
        return Err(AppError::BadRequest(
            "description too long (max 1000)".to_string(),
        ));
    }
    Ok((title.to_string(), description.to_string()))
}

/// Validate a forum topic title: non-empty, ≤ 120 chars.
fn validate_topic_title(title: &str) -> Result<String, AppError> {
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".to_string()));
    }
    if t.chars().count() > 120 {
        return Err(AppError::BadRequest("title too long (max 120)".to_string()));
    }
    Ok(t.to_string())
}

/// Slugify a topic title into a URL-safe, unique-per-topic label used for the
/// human-readable part of board URLs (`/forum/board/{slug}.{id}`).
///
/// Rules (matching the category-slug alphabet so paths stay predictable):
///   * lowercase ASCII + digits + hyphens
///   * non-ASCII letters/digits → dropped; whitespace/punctuation → `-`
///   * hyphens collapsed, trimmed, capped at 60 chars
///
/// Uniqueness is NOT enforced here: `create_topic` appends the numeric id for
/// a guaranteed-unique slug (`{base}-{id}`), and the DB has a partial unique
/// index on `topic_slug`.
fn slugify_topic_title(title: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(title.len().min(64));
    let mut last_hyphen = false;
    for c in title.chars().flat_map(|c| c.to_lowercase()) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c as u8);
            last_hyphen = false;
        } else if !out.is_empty() && !last_hyphen {
            out.push(b'-');
            last_hyphen = true;
        }
    }
    while out.last() == Some(&b'-') {
        out.pop();
    }
    if out.is_empty() {
        out.push(b't');
    }
    let s = String::from_utf8(out).unwrap_or_else(|_| "topic".to_string());
    s.chars().take(60).collect()
}

/// Backfill `topic_slug` for legacy rows that predate migration 047. Runs at
/// startup; each row gets `{base}-{id}` so collisions are impossible even if
/// two topics share a title. Best-effort: failures are logged, never fatal.
pub async fn backfill_topic_slugs(db: &sqlx::PgPool) {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, title FROM forum_topics WHERE topic_slug IS NULL")
            .fetch_all(db)
            .await
            .unwrap_or_default();
    for (id, title) in rows {
        let slug = format!("{}-{id}", slugify_topic_title(&title));
        if let Err(e) = sqlx::query("UPDATE forum_topics SET topic_slug = $2 WHERE id = $1")
            .bind(id)
            .bind(&slug)
            .execute(db)
            .await
        {
            tracing::warn!("backfill_topic_slugs: topic {id}: {e}");
        }
    }
}

/// Look up a topic by its unique slug. Returns the topic id — 400 (repo
/// convention for missing resources) if absent or soft-deleted.
async fn topic_id_by_slug(state: &AppState, slug: &str) -> Result<i64, AppError> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM forum_topics WHERE topic_slug = $1 AND deleted_at IS NULL")
            .bind(slug)
            .fetch_optional(&state.db)
            .await?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::BadRequest("topic not found".to_string()))
}

/// Validate post/topic body markdown: non-empty, ≤ 20000 chars.
fn validate_forum_body(body: &str) -> Result<String, AppError> {
    let b = body.trim();
    if b.is_empty() {
        return Err(AppError::BadRequest("body must not be empty".to_string()));
    }
    if b.chars().count() > 20_000 {
        return Err(AppError::BadRequest(
            "body too long (max 20000)".to_string(),
        ));
    }
    Ok(b.to_string())
}

/// T003: unread helper — true if last_post_id is newer than last_read_post_id.
pub fn is_topic_unread(last_post_id: Option<i64>, last_read_post_id: Option<i64>) -> bool {
    match last_post_id {
        Some(lp) => lp > last_read_post_id.unwrap_or(0),
        None => false,
    }
}

/// T004: ordering helper — pinned topics first. Use as ORDER BY prefix.
pub const PINNED_FIRST_ORDER: &str = "t.status = 'pinned' DESC";

/// Mod-level check: level ≥ FORUM_CURATOR_LEVEL (50) counts as mod for the
/// forum (F7 leveling replaces the legacy role gate).
fn is_mod(auth: &AuthUser) -> bool {
    auth.level >= curator_level()
}

/// Require curator (level ≥ 50); 403 otherwise (also requires login → 401).
fn require_mod(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.level < curator_level() {
        return Err(AppError::Forbidden("Moderator access required".to_string()));
    }
    Ok(uid)
}

// ── Trust moderation (replaces points/metamod, 2026-09) ───────────────────
///
/// The platform trust ladder (TL0 New … TL6 Near-admin,
/// `src/services/trust.rs`) is the single source of "who may moderate".
/// Queue access needs TL4+ (Elder); resolving reports, fast-hide, lock/pin
/// need TL5+ (Community Moderator); bans stay admin. Staff (role ≥ 10)
/// always passes. A thin per-UTC-day action cap replaces the old
/// earned/spendable points currency — no grants, no refill windows.

/// Trust tier that may use the moderation queue (config FORUM_MOD_MIN_TRUST).
fn queue_min_trust(state: &AppState) -> i16 {
    state.config.forum_mod_min_trust.max(1)
}

/// Trust tier that may resolve/fast-hide/lock/pin (config FORUM_RESOLVE_MIN_TRUST).
fn resolve_min_trust(state: &AppState) -> i16 {
    state.config.forum_resolve_min_trust.max(1)
}

/// Require queue access: TL4+ (Elder) or staff. Returns the trust level.
async fn require_trust_queue(state: &AppState, auth: &AuthUser) -> Result<i16, AppError> {
    let uid = require_user(auth)?;
    let level = crate::services::trust::assert_staff_or_min_trust(
        &state.db,
        Some(uid),
        auth.role,
        queue_min_trust(state),
        "moderation queue access",
    )
    .await?;
    Ok(level)
}

/// Require resolve power: TL5+ (Community Moderator) or staff.
async fn require_trust_resolve(state: &AppState, auth: &AuthUser) -> Result<i16, AppError> {
    let uid = require_user(auth)?;
    let level = crate::services::trust::assert_staff_or_min_trust(
        &state.db,
        Some(uid),
        auth.role,
        resolve_min_trust(state),
        "report resolution",
    )
    .await?;
    Ok(level)
}

/// Daily action cap: how many moderation actions this user already logged
/// today (UTC) and the configured cap. Caps `forum_mod_actions` rows —
/// the audit trail doubles as the counter, so no new table.
async fn daily_mod_usage(state: &AppState, user_id: i32) -> Result<(i64, i32), AppError> {
    let used: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM forum_mod_actions
         WHERE moderator_id = $1 AND created_at >= date_trunc('day', NOW())",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    Ok((used, state.config.forum_mod_actions_per_day.max(1)))
}

/// Enforce the daily cap; 429-style Forbidden with a friendly message.
async fn check_daily_cap(state: &AppState, user_id: i32) -> Result<(), AppError> {
    let (used, cap) = daily_mod_usage(state, user_id).await?;
    if used >= i64::from(cap) {
        return Err(AppError::Forbidden(format!(
            "daily moderation limit reached ({used}/{cap}); back tomorrow"
        )));
    }
    Ok(())
}

// ── F5: moderation points + floor actions ─────────────────────────────────

/// Fixed reason → delta table (SPEC §8). Positive = rewards, negative = flags.
fn mod_delta(reason: &str) -> Option<i16> {
    match reason {
        "Insightful" => Some(2),
        "Informative" => Some(1),
        "Interesting" => Some(1),
        "Funny" => Some(1),
        "Off-Topic" => Some(-1),
        "Redundant" => Some(-1),
        "Flamebait" => Some(-2),
        "Troll" => Some(-2),
        "Abusive" => Some(-3),
        _ => None,
    }
}

/// Active ban check for a user writing into a category. A forum-scope ban
/// (category_id NULL) blocks every category; a category-scope ban blocks only
/// that category; expired bans (expires_at < NOW()) are ignored.
async fn is_banned(state: &AppState, user_id: i32, category_id: i64) -> Result<bool, AppError> {
    let banned: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM forum_bans
            WHERE user_id = $1
              AND (category_id IS NULL OR category_id = $2)
              AND (expires_at IS NULL OR expires_at > NOW())
         )",
    )
    .bind(user_id)
    .bind(category_id)
    .fetch_one(&state.db)
    .await?;
    Ok(banned.unwrap_or(false))
}

/// Check that a live (non-deleted, non-hidden) post exists; returns
/// (author_id, score, mod_count) — 400 if missing/deleted.
async fn load_moddable_post(db: &sqlx::PgPool, post_id: i64) -> Result<(i32, i32, i32), AppError> {
    let row: Option<(i32, i32, i32)> = sqlx::query_as(
        "SELECT author_id, score, mod_count FROM forum_posts
         WHERE id = $1 AND deleted_at IS NULL AND is_hidden = FALSE",
    )
    .bind(post_id)
    .fetch_optional(db)
    .await?;
    row.ok_or_else(|| AppError::BadRequest("post not found".to_string()))
}

/// Has this moderator already modded this post?
async fn already_modded(
    db: &sqlx::PgPool,
    post_id: i64,
    moderator_id: i32,
) -> Result<bool, AppError> {
    let hit: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_mod_actions WHERE post_id = $1 AND moderator_id = $2)",
    )
    .bind(post_id)
    .bind(moderator_id)
    .fetch_one(db)
    .await?;
    Ok(hit.unwrap_or(false))
}

/// GET /api/forum/moderation/status — trust tier + queue/resolve powers +
/// daily action usage. The old points-window keys (`points_left`,
/// `expires_at`) are kept as deprecated aliases so existing clients don't
/// break: points_left mirrors remaining daily actions, expires_at is end of
/// the UTC day.
pub async fn moderation_status(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let trust_level = crate::services::trust::fetch_trust_level(&state.db, Some(user_id)).await;
    let can_queue = trust_level >= queue_min_trust(&state) || auth.role >= 10;
    let can_resolve = trust_level >= resolve_min_trust(&state) || auth.role >= 10;
    let (used, cap) = daily_mod_usage(&state, user_id).await?;
    let day_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).map(|t| {
        chrono::DateTime::<Utc>::from_naive_utc_and_offset(t, Utc).to_rfc3339()
    });
    Ok(Json(json!({
        "err": 0,
        "trust_level": trust_level,
        "can_queue": can_queue,
        "can_resolve": can_resolve,
        "actions_today": used,
        "actions_cap": cap,
        // Deprecated aliases (points → daily budget).
        "eligible": can_queue,
        "points_left": (cap as i64 - used).max(0) as i32,
        "expires_at": day_end,
    })))
}

/// GET /api/forum/moderation/queue — posts needing moderation, lowest score
/// first, then oldest. Excludes: my own posts, posts I already modded, hidden
/// (is_hidden or hidden_until) posts, posts with mod_count ≥ 5. Requires
/// trust queue access (TL4+ / staff).
pub async fn moderation_queue(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    require_trust_queue(&state, &auth).await?;

    let rows: Vec<(i64, i64, String, String, i32, i32, String)> = sqlx::query_as(
        r#"SELECT p.id, p.topic_id, u.username,
                  LEFT(p.body, 300) || CASE WHEN LENGTH(p.body) > 300 THEN '…' ELSE '' END,
                  p.score, p.mod_count, p.created_at::text
           FROM forum_posts p
           JOIN users u ON u.id = p.author_id
           WHERE p.deleted_at IS NULL
             AND p.is_hidden = FALSE
             AND p.hidden_until IS NULL
             AND p.author_id <> $1
             AND p.mod_count < 5
             AND NOT EXISTS (SELECT 1 FROM forum_mod_actions ma
                             WHERE ma.post_id = p.id AND ma.moderator_id = $1)
           ORDER BY p.score ASC, p.created_at ASC, p.id ASC
           LIMIT 50"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(post_id, topic_id, author_username, body, score, mod_count, created_at)| {
                json!({
                    "post_id": post_id,
                    "topic_id": topic_id,
                    "author_username": author_username,
                    "body": body,
                    "score": score,
                    "mod_count": mod_count,
                    "reason": Value::Null,
                    "created_at": created_at,
                })
            },
        )
        .collect();
    let count = items.len() as i64;
    Ok(Json(json!({ "err": 0, "items": items, "count": count })))
}

/// POST /api/forum/posts/{postId}/moderate — apply the reason's delta to
/// the post score, bump mod_count. Requires trust queue access (TL4+) and
/// the daily action cap. Auto-collapse stays client-side; only admin
/// fast-hide sets hidden_until server-side.
pub async fn moderate_post(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    Json(body): Json<ModeratePostBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let reason = body.reason.trim().to_string();
    let Some(delta) = mod_delta(&reason) else {
        return Err(AppError::BadRequest(
            "invalid moderation reason".to_string(),
        ));
    };
    require_trust_queue(&state, &auth).await?;
    check_daily_cap(&state, user_id).await?;
    let (author_id, _score, mod_count) = load_moddable_post(&state.db, post_id).await?;
    if author_id == user_id {
        return Err(AppError::Forbidden(
            "Cannot moderate your own post".to_string(),
        ));
    }
    if mod_count >= 5 {
        return Err(AppError::BadRequest(
            "post has reached the moderation limit".to_string(),
        ));
    }
    if already_modded(&state.db, post_id, user_id).await? {
        return Err(AppError::Conflict(
            "You already moderated this post".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let score_after: i32 = sqlx::query_scalar(
        "UPDATE forum_posts SET score = score + $2, mod_count = mod_count + 1
         WHERE id = $1 RETURNING score",
    )
    .bind(post_id)
    .bind(i32::from(delta))
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO forum_mod_actions (post_id, moderator_id, reason, delta, score_after)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(post_id)
    .bind(user_id)
    .bind(&reason)
    .bind(delta)
    .bind(score_after)
    .execute(&mut *tx)
    .await?;
    // Auto-collapse is CLIENT-SIDE (SPEC §5: posts below threshold show a
    // collapsed toggle for readers). The post stays in the topic-detail API
    // with its negative score; the frontend renders it collapsed. Only the
    // admin fast-hide floor action sets hidden_until (server-side exclusion).
    let hidden_until: Option<String> = None;
    // No points spend: the daily cap (checked above) is the throttle, and
    // the forum_mod_actions row itself is the audit trail + cap counter.
    tx.commit().await?;

    crate::modlog::record_json(
        &state.db,
        Some(user_id),
        auth.username.clone(),
        "forum_moderate",
        "forum_post",
        &post_id.to_string(),
        vec![
            ("post_id", json!(post_id)),
            ("reason", json!(reason)),
            ("delta", json!(delta)),
            ("score_after", json!(score_after)),
        ],
    )
    .await;

    // F7: positive mod received → the post author gains +1 exp (capped +3/day).
    if delta > 0 {
        award_mod_received_exp(&state.db, author_id, post_id).await;
    }

    Ok(Json(json!({
        "err": 0,
        "delta": delta,
        "score_after": score_after,
        "hidden_until": hidden_until,
    })))
}

/// GET /api/forum/posts/{postId}/moderations — public mod history for a post.
/// Moderator identity stays hidden (moderator_id only — SPEC).
pub async fn post_moderations(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    require_user(&auth)?;
    let exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM forum_posts WHERE id = $1)")
            .bind(post_id)
            .fetch_one(&state.db)
            .await?;
    if !exists.unwrap_or(false) {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    let rows: Vec<(i32, String, i16, i32, String)> = sqlx::query_as(
        "SELECT moderator_id, reason, delta, score_after, created_at::text
         FROM forum_mod_actions WHERE post_id = $1
         ORDER BY created_at DESC, id DESC",
    )
    .bind(post_id)
    .fetch_all(&state.db)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(moderator_id, reason, delta, score_after, created_at)| {
            json!({
                "moderator_id": moderator_id,
                "reason": reason,
                "delta": delta,
                "score_after": score_after,
                "created_at": created_at,
            })
        })
        .collect();
    Ok(Json(
        json!({ "err": 0, "items": items, "count": items.len() }),
    ))
}

// ── F6: metamoderation (SPEC-COMMUNITY-PLATFORM §2 Layer 2 + §3 + §4) ─────

/// Verdict codes stored in forum_metamod_votes.verdict.
const META_FAIR: i16 = 0;
const META_UNFAIR: i16 = 1;
const META_UNSURE: i16 = 2;

#[derive(Debug, Deserialize)]
pub struct MetaVoteBody {
    pub verdict: String,
}

/// Query params for the retired metamod queue (kept so the stub compiles;
/// the endpoint always returns 410).
#[derive(Debug, Deserialize)]
pub struct MetamodQueueParams {
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub verdict: Option<String>,
    #[serde(default)]
    pub cursor: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// GET /api/forum/metamod/queue — RETIRED (2026-09 trust cutover).
/// Metamoderation voting is replaced by contested-resolution escalation via
/// reports. Returns HTTP 410 with a machine-readable pointer.
pub async fn metamod_queue(
    auth: AuthUser,
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<MetamodQueueParams>,
) -> Result<Json<Value>, AppError> {
    require_user(&auth)?;
    Err(AppError::Gone(
        "metamoderation queue retired; contested resolutions escalate via reports".to_string(),
    ))
}
/// GET /api/forum/metamod/grants/{id} — anonymized context (FR-009). Curator only.
pub async fn metamod_grant_detail(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(grant_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if auth.level < curator_level() {
        return Err(AppError::Forbidden("Moderator access required".to_string()));
    }
    require_user(&auth)?;
    let row: Option<(i64, i64, i64, String, String, i16, i32, String)> = sqlx::query_as(
        r#"SELECT ma.id, p.id, p.topic_id,
                  LEFT(p.body, 2000) || CASE WHEN LENGTH(p.body) > 2000 THEN '…' ELSE '' END,
                  ma.reason, ma.delta, ma.score_after, ma.created_at::text
           FROM forum_mod_actions ma
           JOIN forum_posts p ON p.id = ma.post_id
           WHERE ma.id = $1"#,
    )
    .bind(grant_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((action_id, post_id, topic_id, excerpt, reason, delta, score_after, created_at)) = row
    else {
        return Err(AppError::BadRequest("grant not found".to_string()));
    };
    Ok(Json(json!({
        "err": 0,
        "grant_id": action_id,
        "action_id": action_id,
        "post_id": post_id,
        "topic_id": topic_id,
        "excerpt": excerpt,
        "reason": reason,
        "delta": delta,
        "score_after": score_after,
        "created_at": created_at,
    })))
}

/// POST /api/forum/metamod/grants/{id}/verdict — RETIRED (2026-09 trust cutover).
pub async fn metamod_grant_verdict(
    auth: AuthUser,
    State(_state): State<Arc<AppState>>,
    Path(_grant_id): Path<i64>,
    Json(_body): Json<MetaVoteBody>,
) -> Result<Json<Value>, AppError> {
    require_user(&auth)?;
    Err(AppError::Gone(
        "metamoderation verdicts retired; contested resolutions escalate via reports".to_string(),
    ))
}
/// POST /api/forum/metamod/{actionId}/vote — RETIRED (2026-09 trust cutover).
pub async fn metamod_vote(
    auth: AuthUser,
    State(_state): State<Arc<AppState>>,
    Path(_action_id): Path<i64>,
    Json(_body): Json<MetaVoteBody>,
) -> Result<Json<Value>, AppError> {
    require_user(&auth)?;
    Err(AppError::Gone(
        "metamoderation votes retired; contested resolutions escalate via reports".to_string(),
    ))
}
pub async fn admin_hide_post(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    require_trust_resolve(&state, &auth).await?;
    let actor_id = require_user(&auth)?;
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_posts WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;
    if !exists.unwrap_or(false) {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    let until = Utc::now() + chrono::Duration::hours(72);
    sqlx::query("UPDATE forum_posts SET hidden_until = $2 WHERE id = $1")
        .bind(post_id)
        .bind(until)
        .execute(&state.db)
        .await?;
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        "forum_hide",
        "forum_post",
        &post_id.to_string(),
        vec![("hidden_until", json!(until.to_rfc3339()))],
    )
    .await;
    Ok(Json(
        json!({ "err": 0, "post_id": post_id, "hidden_until": until.to_rfc3339(), "msg": "Post hidden" }),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct LockTopicBody {
    #[serde(default)]
    pub locked: Option<bool>,
}

/// POST /api/admin/forum/topics/{topicId}/lock — open → locked (curator level ≥ 50).
pub async fn admin_lock_topic(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    body: Option<Json<LockTopicBody>>,
) -> Result<Json<Value>, AppError> {
    require_trust_resolve(&state, &auth).await?;
    let actor_id = require_user(&auth)?;
    let (_, _, _, _, status) = load_live_topic(&state.db, topic_id).await?;
    let new_status = match body.and_then(|b| b.locked) {
        Some(true) => "locked",
        Some(false) => "open",
        None => {
            if status == "locked" {
                "open"
            } else {
                "locked"
            }
        }
    };
    sqlx::query("UPDATE forum_topics SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(topic_id)
        .bind(&new_status)
        .execute(&state.db)
        .await?;
    let action = if new_status == "locked" {
        "forum_lock"
    } else {
        "forum_unlock"
    };
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        action,
        "forum_topic",
        &topic_id.to_string(),
        vec![("status", json!(new_status))],
    )
    .await;
    Ok(Json(
        json!({ "err": 0, "id": topic_id, "status": new_status, "msg": "Topic locked" }),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct PinTopicBody {
    #[serde(default)]
    pub pinned: Option<bool>,
}

/// POST /api/admin/forum/topics/{topicId}/pin — toggle pinned status (curator level ≥ 50).
pub async fn admin_pin_topic(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    body: Option<Json<PinTopicBody>>,
) -> Result<Json<Value>, AppError> {
    require_trust_resolve(&state, &auth).await?;
    let actor_id = require_user(&auth)?;
    let (_, _, _, _, status) = load_live_topic(&state.db, topic_id).await?;
    let new_status = match body.and_then(|b| b.pinned) {
        Some(true) => "pinned",
        Some(false) => "open",
        None => {
            if status == "pinned" {
                "open"
            } else {
                "pinned"
            }
        }
    };
    sqlx::query("UPDATE forum_topics SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(topic_id)
        .bind(&new_status)
        .execute(&state.db)
        .await?;
    let action = if new_status == "pinned" {
        "forum_pin"
    } else {
        "forum_unpin"
    };
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        action,
        "forum_topic",
        &topic_id.to_string(),
        vec![("status", json!(new_status))],
    )
    .await;
    Ok(Json(
        json!({ "err": 0, "id": topic_id, "status": new_status, "msg": "Topic pinned" }),
    ))
}

/// POST /api/admin/forum/bans — create a ban. Scope ≤ own level: forum-scope
/// (global) requires role ≥ 10; category-scope requires role ≥ 5. expires_at
/// present = timeout, null = permanent. Logs 'forum_ban' or 'forum_timeout'.
pub async fn admin_create_ban(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateBanBody>,
) -> Result<Json<Value>, AppError> {
    let actor_id = require_user(&auth)?;
    let scope = body.scope.trim().to_string();
    if scope != "forum" && scope != "category" {
        return Err(AppError::BadRequest(
            "scope must be 'forum' or 'category'".to_string(),
        ));
    }
    if scope == "forum" && auth.level < admin_level() {
        return Err(AppError::Forbidden(
            "Forum-scope bans require admin (level 100)".to_string(),
        ));
    }
    // Category-scope bans need resolve power (TL5+) — same tier as
    // fast-hide/lock/pin.
    if scope == "category" {
        require_trust_resolve(&state, &auth).await?;
    }
    let category_id = if scope == "category" {
        let cid = body.category_id.ok_or_else(|| {
            AppError::BadRequest("category_id required for category-scope bans".to_string())
        })?;
        let exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM forum_categories WHERE id = $1)")
                .bind(cid)
                .fetch_one(&state.db)
                .await?;
        if !exists.unwrap_or(false) {
            return Err(AppError::BadRequest("category not found".to_string()));
        }
        Some(cid)
    } else {
        None
    };
    // Parse expires_at (RFC3339) — null / absent = permanent.
    let expires_at: Option<DateTime<Utc>> = match &body.expires_at {
        Some(s) if !s.trim().is_empty() => Some(
            DateTime::parse_from_rfc3339(s.trim())
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| {
                    AppError::BadRequest("expires_at must be RFC3339 or null".to_string())
                })?,
        ),
        _ => None,
    };
    let reason = body.reason.unwrap_or_default();
    let ban_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_bans (user_id, category_id, reason, banned_by, expires_at)
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(body.user_id)
    .bind(category_id)
    .bind(&reason)
    .bind(actor_id)
    .bind(expires_at)
    .fetch_one(&state.db)
    .await?;

    let (action, action_target) = if expires_at.is_some() {
        ("forum_timeout", "forum")
    } else {
        ("forum_ban", "forum")
    };
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        action,
        action_target,
        &ban_id.to_string(),
        vec![
            ("user_id", json!(body.user_id)),
            ("scope", json!(scope)),
            (
                "category_id",
                category_id.map(|c| json!(c)).unwrap_or(Value::Null),
            ),
            ("reason", json!(reason)),
            (
                "expires_at",
                expires_at
                    .map(|e| json!(e.to_rfc3339()))
                    .unwrap_or(Value::Null),
            ),
        ],
    )
    .await;
    Ok(Json(
        json!({ "err": 0, "id": ban_id, "msg": "Ban created" }),
    ))
}

/// DELETE /api/admin/forum/bans/{id} — lift a ban (role ≥ 5). Logs 'forum_unban'.
pub async fn admin_delete_ban(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(ban_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let actor_id = require_admin(&auth)?;
    let deleted = sqlx::query("DELETE FROM forum_bans WHERE id = $1")
        .bind(ban_id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if deleted == 0 {
        return Err(AppError::BadRequest("ban not found".to_string()));
    }
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        "forum_unban",
        "forum",
        &ban_id.to_string(),
        vec![],
    )
    .await;
    Ok(Json(json!({ "err": 0, "id": ban_id, "msg": "Ban lifted" })))
}

/// GET /api/admin/forum/bans — all bans (active + expired) with user/category
/// slugs and the banning actor. Role ≥ 5.
pub async fn admin_list_bans(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    require_admin(&auth)?;
    let rows: Vec<(
        i64,
        i32,
        String,
        Option<String>,
        Option<i64>,
        Option<String>,
        String,
        String,
        Option<String>,
    )> = sqlx::query_as(
        r#"SELECT b.id, b.user_id, u.username, b.reason,
                      b.category_id, c.slug, bu.username,
                      b.created_at::text, b.expires_at::text
               FROM forum_bans b
               JOIN users u ON u.id = b.user_id
               LEFT JOIN forum_categories c ON c.id = b.category_id
               LEFT JOIN users bu ON bu.id = b.banned_by
               ORDER BY b.created_at DESC"#,
    )
    .fetch_all(&state.db)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                user_id,
                user_username,
                reason,
                category_id,
                category_slug,
                banned_by_username,
                created_at,
                expires_at,
            )| {
                json!({
                    "id": id,
                    "user_id": user_id,
                    "user_username": user_username,
                    "reason": reason,
                    "scope": if category_id.is_some() { "category" } else { "forum" },
                    "category_id": category_id,
                    "category_slug": category_slug,
                    "banned_by_username": banned_by_username,
                    "created_at": created_at,
                    "expires_at": expires_at,
                })
            },
        )
        .collect();
    Ok(Json(
        json!({ "err": 0, "items": items, "count": items.len() }),
    ))
}

/// T036: GET /api/forum/moderation/user/{userId}/grants — recent moderation grants for a user.
/// Returns the last N moderation actions targeting the user's posts (reason + delta).
/// Curator-only (level ≥ 50).
pub async fn moderation_user_grants(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    require_trust_resolve(&state, &auth).await?;
    let rows: Vec<(i64, i64, i64, String, i16, i32, String)> = sqlx::query_as(
        r#"SELECT ma.id, ma.post_id, p.topic_id, ma.reason, ma.delta, ma.score_after, ma.created_at::text
           FROM forum_mod_actions ma
           JOIN forum_posts p ON p.id = ma.post_id
           WHERE p.author_id = $1
           ORDER BY ma.created_at DESC, ma.id DESC
           LIMIT 20"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, post_id, topic_id, reason, delta, score_after, created_at)| {
            json!({ "id": id, "post_id": post_id, "topic_id": topic_id, "reason": reason, "delta": delta, "score_after": score_after, "created_at": created_at })
        })
        .collect();
    Ok(Json(
        json!({ "err": 0, "user_id": user_id, "items": items, "count": items.len() }),
    ))
}

/// Fetch a topic row (id, author_id, category_id, title, status) that is not
/// soft-deleted — 400 (repo convention for missing resources) if absent.
async fn load_live_topic(
    db: &sqlx::PgPool,
    topic_id: i64,
) -> Result<(i64, i32, i64, String, String), AppError> {
    let row: Option<(i64, i32, i64, String, String)> = sqlx::query_as(
        "SELECT id, author_id, category_id, title, status
         FROM forum_topics WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(topic_id)
    .fetch_optional(db)
    .await?;
    row.ok_or_else(|| AppError::BadRequest("topic not found".to_string()))
}

/// Map an AuthUser to (id, username) for modlog + notifications.
fn actor_parts(auth: &AuthUser) -> (Option<i32>, Option<String>) {
    (auth.user_id, auth.username.clone())
}

/// Postgres `::text` renders timezone offsets without a colon (`+02` or
/// `+0200`); chrono needs `+02:00`. Insert the colon when the trailing offset
/// is a bare ±HH or ±HHMM.
fn normalize_offset(s: &str) -> String {
    let bytes = s.as_bytes();
    let len = bytes.len();
    // Bare ±HH at the end (e.g. "+02"): position len-3 is the sign.
    if len >= 4
        && (bytes[len - 3] == b'+' || bytes[len - 3] == b'-')
        && bytes[len - 2].is_ascii_digit()
        && bytes[len - 1].is_ascii_digit()
    {
        // Keep everything before the sign, then re-emit sign + HH + ":00".
        let sign = if bytes[len - 3] == b'+' { "+" } else { "-" };
        let mut out = s[..len - 3].to_string();
        out.push_str(sign);
        out.push_str(&s[len - 2..]);
        out.push_str(":00");
        return out;
    }
    // ±HHMM without colon at the end (e.g. "+0200"): position len-5 is the sign.
    if len >= 6
        && (bytes[len - 5] == b'+' || bytes[len - 5] == b'-')
        && bytes[len - 4].is_ascii_digit()
        && bytes[len - 3].is_ascii_digit()
        && bytes[len - 2].is_ascii_digit()
        && bytes[len - 1].is_ascii_digit()
    {
        let mut out = s[..len - 2].to_string();
        out.insert(len - 2, ':');
        return out;
    }
    s.to_string()
}

#[cfg(test)]
mod mention_tests {
    use super::*;

    #[test]
    fn extracts_basic_mentions() {
        assert_eq!(
            extract_mentions("hi @alice and @bob!"),
            vec!["alice", "bob"]
        );
    }

    #[test]
    fn skips_email_and_inline_at() {
        // foo@bar is an email-ish token, not a mention.
        assert_eq!(
            extract_mentions("mail me at foo@bar.com"),
            Vec::<String>::new()
        );
        assert_eq!(extract_mentions("see @user in text"), vec!["user"]);
    }

    #[test]
    fn requires_word_boundary_before_at() {
        // underscore and alnum prefix must not count.
        assert_eq!(extract_mentions("_@user"), Vec::<String>::new());
        assert_eq!(extract_mentions("(@user)"), vec!["user"]);
        assert_eq!(extract_mentions(" @user"), vec!["user"]);
    }

    #[test]
    fn dedupes_preserving_order() {
        assert_eq!(extract_mentions("@aa @bb @aa"), vec!["aa", "bb"]);
    }
}

/// Parse `@username` mentions from a markdown body. Only matches valid
/// FicNexus usernames ([A-Za-z0-9_]{2,32}); skips tokens preceded by another
/// word char (email addresses etc.); dedupes preserving order.
fn extract_mentions(body: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for cap in crate::MENTION_RE.captures_iter(body) {
        let Some(m) = cap.get(1) else { continue };
        // Skip when the '@' is part of a longer word (e.g. foo@bar is not a
        // mention, but " @user" / "(@user" are).
        let at_start = m.start().saturating_sub(1);
        if at_start > 0 {
            let prev = body[..at_start].chars().next_back().unwrap_or(' ');
            if prev.is_alphanumeric() || prev == '_' {
                continue;
            }
        }
        let name = m.as_str().to_string();
        if seen.insert(name.clone()) {
            out.push(name);
        }
    }
    out
}

/// Fetch a category row by slug (visible to readers: not archived).
async fn category_id_by_slug(state: &AppState, slug: &str) -> Result<i64, AppError> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM forum_categories WHERE slug = $1")
        .bind(slug)
        .fetch_optional(&state.db)
        .await?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::BadRequest("category not found".to_string()))
}

// ── Handlers ───────────────────────────────────────────────────────────────

/// GET /api/forum/categories — list visible categories (non-archived, ordered
/// by position) with topic count + latest activity per category.
pub async fn list_categories(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    // Public read: anonymous visitors see non-mod-only categories; logged-in
    // users (incl. mods) see all. FORUM_PUBLIC_READ=false → whole forum
    // login-only.
    let user_id = public_reader_id(&auth, &state)?;
    let is_anon = user_id == 0;

    // All categories are visible (the DDL has no archive flag); the predicate
    // below is forward-compatible with a future `status` column (F7 archiving)
    // and matches every row today.
    let rows: Vec<(
        i64,
        String,
        String,
        String,
        i32,
        bool,
        String,
        i64,
        Option<String>,
    )> = sqlx::query_as(
        r#"SELECT c.id, c.slug, c.title, c.description, c.position, c.is_mod_only,
                      c.created_at::text,
                      (SELECT COUNT(*) FROM forum_topics t
                        WHERE t.category_id = c.id
                          AND t.deleted_at IS NULL AND t.is_hidden = FALSE)::bigint,
                      (SELECT MAX(t.last_activity_at)::text FROM forum_topics t
                        WHERE t.category_id = c.id
                          AND t.deleted_at IS NULL AND t.is_hidden = FALSE)
               FROM forum_categories c
               WHERE ($1::boolean OR c.is_mod_only = FALSE)
               ORDER BY c.position ASC, c.id ASC"#,
    )
    .bind(is_anon)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                slug,
                title,
                description,
                position,
                is_mod_only,
                created_at,
                topic_count,
                last_activity,
            )| {
                json!({
                    "id": id,
                    "slug": slug,
                    "title": title,
                    "description": description,
                    "position": position,
                    "is_mod_only": is_mod_only,
                    "created_at": created_at,
                    "topic_count": topic_count,
                    "last_activity_at": last_activity,
                })
            },
        )
        .collect();

    Ok(Json(json!({ "err": 0, "items": items })))
}

/// GET /api/forum/topics?category={slug}&cursor={id}&limit= — cursor-paginated
/// topic list for a category. Pinned first, then last_activity_at DESC.
/// Excludes deleted/hidden topics.
pub async fn list_topics(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTopicsParams>,
) -> Result<Json<Value>, AppError> {
    // Public read: anonymous visitors may list public categories. If the
    // category is mod-only and the reader is anonymous, refuse (403) — the
    // category itself is hidden from anon lists anyway.
    let user_id = public_reader_id(&auth, &state)?;
    let is_anon = user_id == 0;
    let category_id = category_id_by_slug(&state, params.category.trim()).await?;
    if is_anon {
        let mod_only: Option<bool> =
            sqlx::query_scalar("SELECT is_mod_only FROM forum_categories WHERE id = $1")
                .bind(category_id)
                .fetch_optional(&state.db)
                .await?;
        if mod_only.unwrap_or(true) {
            return Err(AppError::Forbidden("This board requires login".to_string()));
        }
    }
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    // Fetch limit+1 to detect whether a next page exists.
    let fetch_n = limit + 1;
    let cursor = params.cursor;

    // Sort extension (NodeBB parity): recently_replied (default), most_posts, most_views, most_votes, newest, oldest
    let sort = params.sort.as_deref().unwrap_or("recently_replied");
    let rows: Vec<(
        i64,
        String,
        Option<String>,
        i32,
        String,
        i64,
        i64,
        i64,
        i64,
        String,
        Option<String>,
        String,
        bool,
        Option<i64>,
    )> = match sort {
        "most_posts" | "posts" => sqlx::query_as(
        r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
                  (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id = pr.post_id WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
                  (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id, 0)) AS unread, rs.last_read_post_id
           FROM forum_topics t LEFT JOIN users u ON u.id = t.author_id LEFT JOIN forum_read_state rs ON rs.user_id = $4 AND rs.topic_id = t.id
           WHERE t.category_id = $1 AND t.deleted_at IS NULL AND t.is_hidden = FALSE AND ($2::bigint IS NULL OR t.id < $2)
           ORDER BY t.status = 'pinned' DESC, (SELECT COUNT(*) FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL) DESC, t.id DESC LIMIT $3"#,
        ).bind(category_id).bind(cursor).bind(fetch_n).bind(user_id).fetch_all(&state.db).await?,
        "most_views" | "views" => sqlx::query_as(
        r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
                  (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id = pr.post_id WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
                  (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id, 0)) AS unread, rs.last_read_post_id
           FROM forum_topics t LEFT JOIN users u ON u.id = t.author_id LEFT JOIN forum_read_state rs ON rs.user_id = $4 AND rs.topic_id = t.id
           WHERE t.category_id = $1 AND t.deleted_at IS NULL AND t.is_hidden = FALSE AND ($2::bigint IS NULL OR t.id < $2)
           ORDER BY t.status = 'pinned' DESC, t.view_count DESC, t.id DESC LIMIT $3"#,
        ).bind(category_id).bind(cursor).bind(fetch_n).bind(user_id).fetch_all(&state.db).await?,
        "most_votes" | "votes" => sqlx::query_as(
        r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
                  (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id = pr.post_id WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
                  (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id, 0)) AS unread, rs.last_read_post_id
           FROM forum_topics t LEFT JOIN users u ON u.id = t.author_id LEFT JOIN forum_read_state rs ON rs.user_id = $4 AND rs.topic_id = t.id
           WHERE t.category_id = $1 AND t.deleted_at IS NULL AND t.is_hidden = FALSE AND ($2::bigint IS NULL OR t.id < $2)
           ORDER BY t.status = 'pinned' DESC, (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id) DESC, t.id DESC LIMIT $3"#,
        ).bind(category_id).bind(cursor).bind(fetch_n).bind(user_id).fetch_all(&state.db).await?,
        "oldest" | "old" => sqlx::query_as(
        r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
                  (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id = pr.post_id WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
                  (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id, 0)) AS unread, rs.last_read_post_id
           FROM forum_topics t LEFT JOIN users u ON u.id = t.author_id LEFT JOIN forum_read_state rs ON rs.user_id = $4 AND rs.topic_id = t.id
           WHERE t.category_id = $1 AND t.deleted_at IS NULL AND t.is_hidden = FALSE AND ($2::bigint IS NULL OR t.id < $2)
           ORDER BY t.status = 'pinned' DESC, t.created_at ASC, t.id ASC LIMIT $3"#,
        ).bind(category_id).bind(cursor).bind(fetch_n).bind(user_id).fetch_all(&state.db).await?,
        _ => sqlx::query_as(
        r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
                  (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id = pr.post_id WHERE p.topic_id = t.id AND p.deleted_at IS NULL AND p.is_hidden = FALSE)::bigint,
                  t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
                  (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id, 0)) AS unread, rs.last_read_post_id
           FROM forum_topics t LEFT JOIN users u ON u.id = t.author_id LEFT JOIN forum_read_state rs ON rs.user_id = $4 AND rs.topic_id = t.id
           WHERE t.category_id = $1 AND t.deleted_at IS NULL AND t.is_hidden = FALSE AND ($2::bigint IS NULL OR t.id < $2)
           ORDER BY t.status = 'pinned' DESC, t.last_activity_at DESC, t.id DESC LIMIT $3"#,
        ).bind(category_id).bind(cursor).bind(fetch_n).bind(user_id).fetch_all(&state.db).await?,
    };

    let has_more = rows.len() as i64 > limit;
    let items: Vec<Value> = rows
        .into_iter()
        .take(limit as usize)
        .map(
            |(
                id,
                title,
                topic_slug,
                author_id,
                author_username,
                reply_count,
                vote_score,
                view_count,
                last_post_id,
                status,
                last_activity_at,
                created_at,
                unread,
                last_read_post_id,
            )| {
                let mut obj = json!({
                    "id": id,
                    "title": title,
                    "topic_slug": topic_slug,
                    "author_id": author_id,
                    "author_username": author_username,
                    "reply_count": reply_count,
                    "vote_score": vote_score,
                    "view_count": view_count,
                    "last_post_id": last_post_id,
                    "status": status,
                    "last_activity_at": last_activity_at,
                    "created_at": created_at,
                    "unread": unread,
                });
                if !is_anon {
                    obj["last_read_post_id"] = json!(last_read_post_id);
                }
                obj
            },
        )
        .collect();

    let next_cursor = if has_more {
        items.last().and_then(|i| i["id"].as_i64())
    } else {
        None
    };

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "next_cursor": next_cursor,
        "category": params.category.trim(),
        "limit": limit,
    })))
}

/// GET /api/forum/unread — cross-category unread topics for the caller (auth required).
/// Mirrors NodeBB /unread: topics where last_post_id > last_read_post_id.
/// Query: ?limit & ?cursor (id).
pub async fn unread_topics(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let limit: i64 = q
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(25)
        .clamp(1, 100);
    let cursor: Option<i64> = q.get("cursor").and_then(|s| s.parse().ok());
    let fetch_n = limit + 1;
    let rows: Vec<(i64,String,Option<String>,i32,String,i64,i64,i64,i64,String,Option<String>,String,i64,String)> = sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
        (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL AND p.is_hidden=FALSE)::bigint,
        (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id)::bigint,
        t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text, c.slug, c.title
        FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id LEFT JOIN users u ON u.id=t.author_id
        LEFT JOIN forum_read_state rs ON rs.user_id=$1 AND rs.topic_id=t.id
        WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE AND t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id,0)
        AND ($2::bigint IS NULL OR t.id < $2) ORDER BY t.last_activity_at DESC, t.id DESC LIMIT $3"#)
        .bind(user_id).bind(cursor).bind(fetch_n).fetch_all(&state.db).await?;
    let has_more = rows.len() as i64 > limit;
    let items: Vec<Value> = rows.into_iter().take(limit as usize).map(|(id,title,topic_slug,author_id,author_username,reply_count,vote_score,view_count,last_post_id,status,last_activity_at,created_at,category_slug,category_title)| json!({"id":id,"title":title,"topic_slug":topic_slug,"author_id":author_id,"author_username":author_username,"reply_count":reply_count,"vote_score":vote_score,"view_count":view_count,"last_post_id":last_post_id,"status":status,"last_activity_at":last_activity_at,"created_at":created_at,"category_slug":category_slug,"category_title":category_title,"unread":true})).collect();
    let next_cursor = if has_more {
        items.last().and_then(|i| i["id"].as_i64())
    } else {
        None
    };
    Ok(Json(
        json!({"err":0,"items":items,"next_cursor":next_cursor,"limit":limit}),
    ))
}

/// GET /api/forum/recent — recent topics across all categories (NodeBB /recent).
/// Query: ?limit & ?cursor & ?category (optional filter by slug).
pub async fn recent_topics(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let user_id = public_reader_id(&auth, &state)?;
    let _is_anon = user_id == 0;
    let limit: i64 = q
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(25)
        .clamp(1, 100);
    let cursor: Option<i64> = q.get("cursor").and_then(|s| s.parse().ok());
    let cat_filter = q
        .get("category")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let cat_id: Option<i64> = if let Some(slug) = &cat_filter {
        Some(category_id_by_slug(&state, slug).await?)
    } else {
        None
    };
    let fetch_n = limit + 1;
    let rows: Vec<(i64,String,Option<String>,i32,String,i64,i64,i64,i64,String,Option<String>,String,bool,String,String)> = sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
        (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL AND p.is_hidden=FALSE)::bigint,
        (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id)::bigint,
        t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text,
        (t.last_post_id IS NOT NULL AND t.last_post_id > COALESCE(rs.last_read_post_id,0)) AS unread, c.slug, c.title
        FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id LEFT JOIN users u ON u.id=t.author_id
        LEFT JOIN forum_read_state rs ON rs.user_id=$3 AND rs.topic_id=t.id
        WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE AND ($4::bigint IS NULL OR t.category_id=$4) AND ($1::bigint IS NULL OR t.id < $1)
        ORDER BY t.last_activity_at DESC, t.id DESC LIMIT $2"#)
        .bind(cursor).bind(fetch_n).bind(user_id).bind(cat_id).fetch_all(&state.db).await?;
    let has_more = rows.len() as i64 > limit;
    let items: Vec<Value> = rows.into_iter().take(limit as usize).map(|(id,title,topic_slug,author_id,author_username,reply_count,vote_score,view_count,last_post_id,status,last_activity_at,created_at,unread,category_slug,category_title)| json!({"id":id,"title":title,"topic_slug":topic_slug,"author_id":author_id,"author_username":author_username,"reply_count":reply_count,"vote_score":vote_score,"view_count":view_count,"last_post_id":last_post_id,"status":status,"last_activity_at":last_activity_at,"created_at":created_at,"unread":unread,"category_slug":category_slug,"category_title":category_title})).collect();
    let next_cursor = if has_more {
        items.last().and_then(|i| i["id"].as_i64())
    } else {
        None
    };
    Ok(Json(
        json!({"err":0,"items":items,"next_cursor":next_cursor,"limit":limit}),
    ))
}

/// GET /api/forum/popular — popular topics by view_count or reply_count (NodeBB /popular).
/// Query: ?limit & ?cursor & ?sort=views|posts|votes (default views).
pub async fn popular_topics(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    let user_id = public_reader_id(&auth, &state)?;
    let limit: i64 = q
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(25)
        .clamp(1, 100);
    let cursor: Option<i64> = q.get("cursor").and_then(|s| s.parse().ok());
    let sort = q.get("sort").map(|s| s.as_str()).unwrap_or("views");
    let order_sql = match sort {
        "posts" | "replies" => {
            "(SELECT COUNT(*) FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL) DESC"
        }
        "votes" => {
            "(SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id) DESC"
        }
        _ => "t.view_count DESC",
    };
    let fetch_n = limit + 1;
    // Use a single query with dynamic ORDER BY via CASE branching (avoid string interpolation)
    let rows: Vec<(i64,String,Option<String>,i32,String,i64,i64,i64,i64,String,Option<String>,String,String,String)> = match sort {
        "posts"|"replies" => sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
            (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL AND p.is_hidden=FALSE)::bigint,
            (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id)::bigint,
            t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text, c.slug, c.title
            FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id LEFT JOIN users u ON u.id=t.author_id
            WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE AND ($1::bigint IS NULL OR t.id < $1) ORDER BY (SELECT COUNT(*) FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL) DESC, t.id DESC LIMIT $2"#)
            .bind(cursor).bind(fetch_n).fetch_all(&state.db).await?,
        "votes" => sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
            (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL AND p.is_hidden=FALSE)::bigint,
            (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id)::bigint,
            t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text, c.slug, c.title
            FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id LEFT JOIN users u ON u.id=t.author_id
            WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE AND ($1::bigint IS NULL OR t.id < $1) ORDER BY (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id) DESC, t.id DESC LIMIT $2"#)
            .bind(cursor).bind(fetch_n).fetch_all(&state.db).await?,
        _ => sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.author_id, u.username,
            (SELECT COUNT(*) - 1 FROM forum_posts p WHERE p.topic_id=t.id AND p.deleted_at IS NULL AND p.is_hidden=FALSE)::bigint,
            (SELECT COUNT(*) FROM forum_post_reactions pr JOIN forum_posts p ON p.id=pr.post_id WHERE p.topic_id=t.id)::bigint,
            t.view_count, t.last_post_id, t.status, t.last_activity_at::text, t.created_at::text, c.slug, c.title
            FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id LEFT JOIN users u ON u.id=t.author_id
            WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE AND ($1::bigint IS NULL OR t.id < $1) ORDER BY t.view_count DESC, t.id DESC LIMIT $2"#)
            .bind(cursor).bind(fetch_n).fetch_all(&state.db).await?,
    };
    let _ = (user_id, order_sql);
    let has_more = rows.len() as i64 > limit;
    let items: Vec<Value> = rows.into_iter().take(limit as usize).map(|(id,title,topic_slug,author_id,author_username,reply_count,vote_score,view_count,last_post_id,status,last_activity_at,created_at,category_slug,category_title)| json!({"id":id,"title":title,"topic_slug":topic_slug,"author_id":author_id,"author_username":author_username,"reply_count":reply_count,"vote_score":vote_score,"view_count":view_count,"last_post_id":last_post_id,"status":status,"last_activity_at":last_activity_at,"created_at":created_at,"category_slug":category_slug,"category_title":category_title})).collect();
    let next_cursor = if has_more {
        items.last().and_then(|i| i["id"].as_i64())
    } else {
        None
    };
    Ok(Json(
        json!({"err":0,"items":items,"next_cursor":next_cursor,"limit":limit,"sort":sort}),
    ))
}

/// GET /api/forum/recent.rss & /api/forum/popular.rss — RSS feeds (NodeBB feeds.js parity, minimal).
pub async fn forum_rss(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Response, AppError> {
    let feed = q.get("feed").map(|s| s.as_str()).unwrap_or("recent");
    let limit: i64 = q
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .clamp(1, 50);
    let order = if feed == "popular" {
        "t.view_count DESC"
    } else {
        "t.last_activity_at DESC"
    };
    let _ = order;
    let rows: Vec<(i64, String, Option<String>, String, String)> = if feed == "popular" {
        sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.created_at::text, c.title FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE ORDER BY t.view_count DESC, t.id DESC LIMIT $1"#).bind(limit).fetch_all(&state.db).await?
    } else {
        sqlx::query_as(r#"SELECT t.id, t.title, t.topic_slug, t.created_at::text, c.title FROM forum_topics t JOIN forum_categories c ON c.id=t.category_id WHERE t.deleted_at IS NULL AND t.is_hidden=FALSE ORDER BY t.last_activity_at DESC, t.id DESC LIMIT $1"#).bind(limit).fetch_all(&state.db).await?
    };
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0"><channel><title>FicNexus Forum</title><link>/forum</link><description>Recent topics</description>"#,
    );
    for (id, title, slug, created, cat) in rows {
        let link = format!(
            "/forum/board/{}.{}",
            slug.unwrap_or_else(|| format!("topic-{}", id)),
            id
        );
        xml.push_str(&format!("<item><title>{}</title><link>{}</link><category>{}</category><pubDate>{}</pubDate><guid>{}</guid></item>", quick_xml::escape::escape(&title), quick_xml::escape::escape(&link), quick_xml::escape::escape(&cat), quick_xml::escape::escape(&created), id));
    }
    xml.push_str("</channel></rss>");
    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "application/rss+xml; charset=utf-8",
        )],
        xml,
    )
        .into_response())
}

/// POST /api/forum/topics/{id}/tags — attach tags to a topic (NodeBB topics/tags.js parity, minimal).
/// Body: { tags: string[] } — creates missing global tags then links.
/// Requires author or mod.
pub async fn set_topic_tags(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let tags: Vec<String> = body
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty() && s.len() <= 32)
                .collect()
        })
        .unwrap_or_default();
    if tags.len() > 5 {
        return Err(AppError::BadRequest("max 5 tags".to_string()));
    }
    let topic: Option<(i32,)> =
        sqlx::query_as("SELECT author_id FROM forum_topics WHERE id=$1 AND deleted_at IS NULL")
            .bind(topic_id)
            .fetch_optional(&state.db)
            .await?;
    let (author_id,) = topic.ok_or_else(|| AppError::NotFound("topic not found".to_string()))?;
    let is_author = author_id == user_id;
    let is_mod = auth.level >= 50;
    if !is_author && !is_mod {
        return Err(AppError::Forbidden("not allowed".to_string()));
    }
    // Upsert tags into global tags table then link via forum_topic_tags (create table if not exists via migration 073, but also ensure here)
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS forum_topic_tags (topic_id bigint NOT NULL REFERENCES forum_topics(id) ON DELETE CASCADE, tag text NOT NULL, PRIMARY KEY(topic_id, tag))").execute(&state.db).await;
    sqlx::query("DELETE FROM forum_topic_tags WHERE topic_id=$1")
        .bind(topic_id)
        .execute(&state.db)
        .await?;
    for tag in &tags {
        let _ = sqlx::query(
            "INSERT INTO forum_topic_tags (topic_id, tag) VALUES ($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(topic_id)
        .bind(tag)
        .execute(&state.db)
        .await;
    }
    Ok(Json(json!({"err":0,"tags":tags})))
}
pub async fn get_topic_tags(
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let tags: Vec<(String,)> =
        sqlx::query_as("SELECT tag FROM forum_topic_tags WHERE topic_id=$1 ORDER BY tag")
            .bind(topic_id)
            .fetch_all(&state.db)
            .await
            .unwrap_or_default();
    Ok(Json(
        json!({"err":0,"tags": tags.into_iter().map(|(t,)| t).collect::<Vec<_>>()}),
    ))
}

/// GET /api/forum/preferences — per-user pagination prefs (NodeBB user.getSettings parity, minimal).
/// Stored in user_preferences JSONB or a dedicated table; we use a tiny table.
pub async fn get_forum_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS forum_user_prefs (user_id integer PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE, posts_per_page integer NOT NULL DEFAULT 25, topic_sort text NOT NULL DEFAULT 'recently_replied')").execute(&state.db).await;
    let row: Option<(i32, String)> =
        sqlx::query_as("SELECT posts_per_page, topic_sort FROM forum_user_prefs WHERE user_id=$1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?;
    let (ppp, sort) = row.unwrap_or((25, "recently_replied".to_string()));
    Ok(Json(
        json!({"err":0,"posts_per_page":ppp,"topic_sort":sort}),
    ))
}
pub async fn set_forum_prefs(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let ppp: i32 = body
        .get("posts_per_page")
        .and_then(|v| v.as_i64())
        .map(|n| (n as i32).clamp(10, 100))
        .unwrap_or(25);
    let sort = body
        .get("topic_sort")
        .and_then(|v| v.as_str())
        .unwrap_or("recently_replied")
        .to_string();
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS forum_user_prefs (user_id integer PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE, posts_per_page integer NOT NULL DEFAULT 25, topic_sort text NOT NULL DEFAULT 'recently_replied')").execute(&state.db).await;
    sqlx::query("INSERT INTO forum_user_prefs (user_id, posts_per_page, topic_sort) VALUES ($1,$2,$3) ON CONFLICT (user_id) DO UPDATE SET posts_per_page=$2, topic_sort=$3").bind(user_id).bind(ppp).bind(&sort).execute(&state.db).await?;
    Ok(Json(
        json!({"err":0,"posts_per_page":ppp,"topic_sort":sort}),
    ))
}

/// POST /api/forum/categories — create category (admin only, role ≥ 10).
pub async fn create_category(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateCategoryBody>,
) -> Result<Json<Value>, AppError> {
    require_admin(&auth)?;

    let slug = validate_slug(&body.slug)?;
    let (title, description) = validate_category_fields(&body.title, &body.description)?;
    let position = body.position.unwrap_or(0);

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_categories (slug, title, description, position)
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&slug)
    .bind(&title)
    .bind(&description)
    .bind(position)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::BadRequest("category slug already exists".to_string())
        }
        _ => AppError::from(e),
    })?;

    Ok(Json(
        json!({ "err": 0, "id": id, "msg": "Category created" }),
    ))
}

/// PATCH /api/forum/categories/{id} — edit/reorder category (admin only).
pub async fn update_category(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateCategoryBody>,
) -> Result<Json<Value>, AppError> {
    require_admin(&auth)?;

    // Load the current row so absent fields keep their existing values.
    let current: Option<(String, String, String, i32)> = sqlx::query_as(
        "SELECT slug, title, description, position FROM forum_categories WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;
    let (cur_slug, cur_title, cur_desc, cur_pos) =
        current.ok_or_else(|| AppError::BadRequest("category not found".to_string()))?;

    let slug = match &body.slug {
        Some(s) => validate_slug(s)?,
        None => cur_slug,
    };
    let (title, description) = match (&body.title, &body.description) {
        (Some(t), Some(d)) => validate_category_fields(t, d)?,
        (Some(t), None) => {
            let (nt, _) = validate_category_fields(t, &cur_desc)?;
            (nt, cur_desc)
        }
        (None, Some(d)) => {
            let (_, nd) = validate_category_fields(&cur_title, d)?;
            (cur_title, nd)
        }
        (None, None) => (cur_title, cur_desc),
    };
    let position = body.position.unwrap_or(cur_pos);

    sqlx::query(
        "UPDATE forum_categories SET slug = $2, title = $3, description = $4, position = $5
         WHERE id = $1",
    )
    .bind(id)
    .bind(&slug)
    .bind(&title)
    .bind(&description)
    .bind(position)
    .execute(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::BadRequest("category slug already exists".to_string())
        }
        _ => AppError::from(e),
    })?;

    Ok(Json(
        json!({ "err": 0, "id": id, "msg": "Category updated" }),
    ))
}

// ── F3: topic write path ───────────────────────────────────────────────────

/// GET /api/forum/topics/{topicId} — topic detail: OP + posts
/// (`?after={postId}` cursor, default limit 25, max 100). Increments
/// view_count on fetch. Posts include the quoted-post preview resolved via
/// `quote_of` (best-effort).
pub async fn topic_detail(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    Query(params): Query<TopicDetailParams>,
) -> Result<Json<Value>, AppError> {
    // Public read (FORUM_PUBLIC_READ default true): anonymous visitors may
    // view topics in public categories. Mod-only category + anon → 403.
    let user_id = public_reader_id(&auth, &state)?;
    let is_anon = user_id == 0;
    let (id, author_id, category_id, title, status) = load_live_topic(&state.db, topic_id).await?;
    if is_anon {
        let mod_only: Option<bool> =
            sqlx::query_scalar("SELECT is_mod_only FROM forum_categories WHERE id = $1")
                .bind(category_id)
                .fetch_optional(&state.db)
                .await?;
        if mod_only.unwrap_or(true) {
            return Err(AppError::Forbidden("This board requires login".to_string()));
        }
    }

    let topic_row: Option<(
        String,
        String,
        String,
        String,
        Option<String>,
        i64,
        String,
        Option<String>,
        Option<String>,
    )> = sqlx::query_as(
        r#"SELECT u.username, c.slug, c.title, t.body, t.topic_slug, t.view_count,
                      t.created_at::text, t.updated_at::text, t.payload::text
               FROM forum_topics t
               JOIN users u ON u.id = t.author_id
               JOIN forum_categories c ON c.id = t.category_id
               WHERE t.id = $1 AND t.deleted_at IS NULL"#,
    )
    .bind(topic_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((
        author_username,
        category_slug,
        category_title,
        op_body,
        topic_slug,
        view_count,
        created_at,
        updated_at,
        payload,
    )) = topic_row
    else {
        return Err(AppError::BadRequest("topic not found".to_string()));
    };
    let payload_value = payload
        .as_deref()
        .and_then(|p| serde_json::from_str::<Value>(p).ok())
        .unwrap_or(Value::Null);
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let after = params.after;

    // Order: OP first (id = topic's first post), then replies by id ASC so
    // `after={postId}` cursor pagination works; the OP is re-included on the
    // first page (no cursor) and skipped once the cursor advances past it.
    // Hidden posts (is_hidden or hidden_until) are excluded — F5 auto-collapse
    // and admin fast-hide must hide posts from readers.
    let posts: Vec<(
        i64,
        i32,
        String,
        String,
        Option<i64>,
        String,
        Option<String>,
        Option<String>,
        i32,
    )> = sqlx::query_as(
        r#"SELECT p.id, p.author_id, u.username, p.body, p.quote_of,
                  p.created_at::text, p.edited_at::text, p.deleted_at::text, p.score
          FROM forum_posts p
          JOIN users u ON u.id = p.author_id
          WHERE p.topic_id = $1
            AND p.deleted_at IS NULL
            AND p.is_hidden = FALSE
            AND (p.hidden_until IS NULL OR p.hidden_until <= NOW())
            AND ($2::bigint IS NULL OR p.id > $2)
          ORDER BY p.id ASC
          LIMIT $3"#,
    )
    .bind(topic_id)
    .bind(after)
    .bind(limit + 1)
    .fetch_all(&state.db)
    .await?;
    let has_more = posts.len() as i64 > limit;
    let post_ids: Vec<i64> = posts.iter().map(|p| p.0).collect();
    let quoted_ids: Vec<i64> = posts
        .iter()
        .filter_map(|p| p.4)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let quoted_ids_all: Vec<i64> = post_ids
        .iter()
        .copied()
        .chain(quoted_ids.iter().copied())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    // Resolve quoted-post previews (id → "author: first 200 chars") in one
    // query. Missing/deleted quotes are omitted (quote_of stays raw).
    let mut quote_map: std::collections::HashMap<i64, Value> = std::collections::HashMap::new();
    if !quoted_ids_all.is_empty() {
        let qrows: Vec<(i64, String, String)> = sqlx::query_as(
            r#"SELECT p.id, u.username,
                      LEFT(p.body, 200) || CASE WHEN LENGTH(p.body) > 200 THEN '…' ELSE '' END
               FROM forum_posts p JOIN users u ON u.id = p.author_id
               WHERE p.id = ANY($1) AND p.deleted_at IS NULL"#,
        )
        .bind(&quoted_ids_all)
        .fetch_all(&state.db)
        .await?;
        for (qid, qauthor, preview) in qrows {
            quote_map.insert(
                qid,
                json!({ "author_username": qauthor, "preview": preview }),
            );
        }
    }

    // The OP is a real forum_posts row (create_topic inserts it as the first
    // post), so items are just the posts rows; the first (lowest-id) post is
    // flagged is_op: true. No separate synthesis — avoids double-counting.
    let first_post_id: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM forum_posts WHERE topic_id = $1 ORDER BY id ASC LIMIT 1",
    )
    .bind(topic_id)
    .fetch_optional(&state.db)
    .await?;
    let mut items: Vec<Value> = Vec::with_capacity(limit as usize + 1);
    for p in posts.into_iter().take(limit as usize) {
        let (
            pid,
            pauthor_id,
            pauthor_username,
            pbody,
            pquote_of,
            pcreated_at,
            pedited_at,
            pdeleted_at,
            pscore,
        ) = p;
        let is_op = first_post_id == Some(pid);
        let quote = pquote_of.and_then(|q| quote_map.get(&q).cloned());
        items.push(json!({
            "id": pid,
            "author_id": pauthor_id,
            "author_username": pauthor_username,
            "body": pbody,
            "quote_of": pquote_of,
            "quote": quote.unwrap_or(Value::Null),
            "edited_at": pedited_at,
            "deleted_at": pdeleted_at,
            "created_at": pcreated_at,
            "score": pscore,
            "is_op": is_op,
        }));
    }

    // Load reactions for all posts in this page
    let reaction_viewer: Option<i32> = auth.user_id;
    let reactions_map = load_reactions_batch(&state.db, &post_ids, reaction_viewer).await?;
    for item in items.iter_mut() {
        if let Some(pid) = item["id"].as_i64() {
            item["reactions"] = reactions_map
                .get(&pid)
                .cloned()
                .unwrap_or(json!({ "counts": {}, "my_reactions": [] }));
        }
    }

    let next_cursor = if has_more {
        items.last().and_then(|i| i["id"].as_i64())
    } else {
        None
    };
    // view_count increments on every detail fetch (contract: topic views).
    // Rate-limit: once per 60 minutes per user (via forum_topic_views).
    let new_view_count: i64 = if params.inc_views {
        let viewer_user_id: Option<i32> = auth.user_id;
        let can_count = if let Some(uid) = viewer_user_id {
            // Check if user viewed this topic in the last 60 minutes
            let recent: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
                "SELECT viewed_at FROM forum_topic_views WHERE topic_id = $1 AND user_id = $2",
            )
            .bind(topic_id)
            .bind(uid)
            .fetch_optional(&state.db)
            .await?;
            match recent {
                Some((viewed_at,)) => {
                    let elapsed = chrono::Utc::now()
                        .signed_duration_since(viewed_at)
                        .num_seconds();
                    elapsed >= 3600 // 60 minutes
                }
                None => true, // first view
            }
        } else {
            true // anonymous: always count (no user_id to rate-limit)
        };
        if can_count {
            // Rate-limit DB ops only for logged-in users (user_id is NOT NULL in PK)
            if let Some(uid) = viewer_user_id {
                sqlx::query(
                    "INSERT INTO forum_topic_views (topic_id, user_id, viewed_at)
                     VALUES ($1, $2, now())
                     ON CONFLICT (topic_id, user_id) DO UPDATE SET viewed_at = now()",
                )
                .bind(topic_id)
                .bind(uid)
                .execute(&state.db)
                .await?;
            }
            sqlx::query_scalar(
                "UPDATE forum_topics SET view_count = view_count + 1 WHERE id = $1 RETURNING view_count",
            )
            .bind(topic_id)
            .fetch_one(&state.db)
            .await?
        } else {
            sqlx::query_scalar("SELECT view_count FROM forum_topics WHERE id = $1")
                .bind(topic_id)
                .fetch_one(&state.db)
                .await?
        }
    } else {
        sqlx::query_scalar("SELECT view_count FROM forum_topics WHERE id = $1")
            .bind(topic_id)
            .fetch_one(&state.db)
            .await?
    };
    // Read-state for auth users
    let mut resp = json!({
        "err": 0,
        "id": id,
        "title": title,
        "topic_slug": topic_slug,
        "author_id": author_id,
        "author_username": author_username,
        "category_slug": category_slug,
        "category_title": category_title,
        "status": status,
        "body": op_body,
        "payload": payload_value,
        "view_count": new_view_count,
        "created_at": created_at,
        "updated_at": updated_at,
        "items": items,
        "next_cursor": next_cursor,
        "limit": limit,
        "view_count_before": view_count,
    });
    if !is_anon {
        let last_post_id: Option<i64> =
            sqlx::query_scalar("SELECT last_post_id FROM forum_topics WHERE id = $1")
                .bind(topic_id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten();
        let last_read: Option<i64> = sqlx::query_scalar(
            "SELECT last_read_post_id FROM forum_read_state WHERE user_id = $1 AND topic_id = $2",
        )
        .bind(user_id)
        .bind(topic_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten();
        let unread = last_post_id
            .map(|lp| lp > last_read.unwrap_or(0))
            .unwrap_or(false);
        resp["unread"] = json!(unread);
        resp["last_read_post_id"] = json!(last_read);
    }
    Ok(Json(resp))
}

/// GET /api/forum/topics/by-slug/{topicSlug} — topic detail resolved by the
/// unique topic slug (`{base}-{id}`). Thin wrapper over [`topic_detail`]:
/// maps the slug to the numeric id then delegates, so the detail response
/// (posts, view_count increment, follow state) is identical.
///
/// Registered BEFORE `/api/forum/topics/{topicId}` in the router — the
/// literal `by-slug` segment must win over the numeric param.
pub async fn topic_detail_by_slug(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_slug): Path<String>,
    Query(mut params): Query<TopicDetailParams>,
) -> Result<Json<Value>, AppError> {
    let topic_id = topic_id_by_slug(&state, &topic_slug).await?;
    // Slug endpoint is a resolver only — skip view increment.
    // The frontend calls this first, then fetches by ID which increments.
    params.inc_views = false;
    topic_detail(auth, State(state), Path(topic_id), Query(params)).await
}
/// POST /api/forum/topics — create topic + OP post in ONE transaction.
/// Validates title (≤120) + body (≤20000); no notifications for creation.
pub async fn create_topic(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTopicBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let title = validate_topic_title(&body.title)?;
    let op_body = validate_forum_body(&body.body)?;
    let category_id = category_id_by_slug(&state, body.category_slug.trim()).await?;
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }
    let payload = body.payload.unwrap_or(json!({}));

    // Marginalia gate: Level 5+ required
    let is_marginalia = payload.get("type").and_then(|v| v.as_str()) == Some("marginalia");
    if is_marginalia && auth.level < 5 {
        return Err(AppError::Forbidden(
            "Level 5 required for marginalia".to_string(),
        ));
    }
    // If marginalia, check for existing topic on same passage (one topic per passage)
    if is_marginalia {
        if let (Some(passage_hash), Some(chapter_index), Some(work_id)) = (
            payload.get("passage_hash").and_then(|v| v.as_str()),
            payload.get("chapter_index").and_then(|v| v.as_i64()),
            payload.get("work_id").and_then(|v| v.as_i64()),
        ) {
            if work_id > 0 && !passage_hash.is_empty() {
                let existing: Option<i64> = sqlx::query_scalar(
                    "SELECT topic_id FROM marginalia WHERE work_id = $1 AND chapter_index = $2 AND passage_hash = $3",
                )
                .bind(work_id as i32)
                .bind(chapter_index as i32)
                .bind(passage_hash)
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None);
                if let Some(existing_topic_id) = existing {
                    let slug: Option<String> =
                        sqlx::query_scalar("SELECT topic_slug FROM forum_topics WHERE id = $1")
                            .bind(existing_topic_id)
                            .fetch_optional(&state.db)
                            .await
                            .unwrap_or(None);
                    return Ok(Json(json!({
                        "err": 0,
                        "id": existing_topic_id,
                        "topic_slug": slug,
                        "msg": "Existing marginalia topic",
                        "existing": true
                    })));
                }
            }
        } else if let (Some(passage_hash), Some(chapter_index)) = (
            payload.get("passage_hash").and_then(|v| v.as_str()),
            payload.get("chapter_index").and_then(|v| v.as_i64()),
        ) {
            // fallback: resolve work_id via url_id if work_id missing
            if let Some(url_id) = payload.get("url_id").and_then(|v| v.as_str()) {
                if let Ok(Some(work)) = queries::get_work_by_source(&state.db, url_id).await {
                    let existing: Option<i64> = sqlx::query_scalar(
                        "SELECT topic_id FROM marginalia WHERE work_id = $1 AND chapter_index = $2 AND passage_hash = $3",
                    )
                    .bind(work.id)
                    .bind(chapter_index as i32)
                    .bind(passage_hash)
                    .fetch_optional(&state.db)
                    .await
                    .unwrap_or(None);
                    if let Some(existing_topic_id) = existing {
                        let slug: Option<String> =
                            sqlx::query_scalar("SELECT topic_slug FROM forum_topics WHERE id = $1")
                                .bind(existing_topic_id)
                                .fetch_optional(&state.db)
                                .await
                                .unwrap_or(None);
                        return Ok(Json(json!({
                            "err": 0,
                            "id": existing_topic_id,
                            "topic_slug": slug,
                            "msg": "Existing marginalia topic",
                            "existing": true
                        })));
                    }
                }
            }
        }
    }

    let mut tx = state.db.begin().await?;
    let topic_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_topics (category_id, author_id, title, body, payload, search_vector)
         VALUES ($1, $2, $3, $4, $5, to_tsvector('english', $3 || ' ' || $4))
         RETURNING id",
    )
    .bind(category_id)
    .bind(user_id)
    .bind(&title)
    .bind(&op_body)
    .bind(&payload)
    .fetch_one(&mut *tx)
    .await?;
    // Canonical slug: `{base}-{id}` — the id suffix guarantees uniqueness even
    // when two topics share a title (partial unique index on topic_slug).
    let topic_slug = format!("{}-{topic_id}", slugify_topic_title(&title));
    sqlx::query("UPDATE forum_topics SET topic_slug = $2 WHERE id = $1")
        .bind(topic_id)
        .bind(&topic_slug)
        .execute(&mut *tx)
        .await?;
    let post_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body, payload, search_vector)
         VALUES ($1, $2, $3, $4, to_tsvector('english', $3))
         RETURNING id",
    )
    .bind(topic_id)
    .bind(user_id)
    .bind(&op_body)
    .bind(&payload)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE forum_topics SET last_post_id = $2, last_activity_at = NOW() WHERE id = $1",
    )
    .bind(topic_id)
    .bind(post_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Insert marginalia link if this topic is a marginalia discussion
    if is_marginalia {
        let passage_hash = payload
            .get("passage_hash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let chapter_index = payload
            .get("chapter_index")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        // Resolve work_id: prefer payload.work_id, else via url_id
        let work_id_opt: Option<i32> =
            if let Some(wid) = payload.get("work_id").and_then(|v| v.as_i64()) {
                Some(wid as i32)
            } else if let Some(url_id) = payload.get("url_id").and_then(|v| v.as_str()) {
                queries::get_work_by_source(&state.db, url_id)
                    .await
                    .ok()
                    .flatten()
                    .map(|w| w.id)
            } else {
                None
            };
        let passage_text = payload
            .get("passage_text")
            .and_then(|v| v.as_str())
            .filter(|t| !t.trim().is_empty());
        if let Some(work_id) = work_id_opt {
            if !passage_hash.is_empty() {
                let _ = sqlx::query(
                    "INSERT INTO marginalia (work_id, chapter_index, passage_hash, topic_id, passage_text) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (work_id, chapter_index, passage_hash) DO UPDATE SET passage_text = COALESCE(marginalia.passage_text, EXCLUDED.passage_text)"
                )
                .bind(work_id)
                .bind(chapter_index)
                .bind(passage_hash)
                .bind(topic_id)
                .bind(passage_text)
                .execute(&state.db)
                .await;
            }
        }
    }

    award_post_exp(&state.db, user_id, post_id).await;

    Ok(Json(json!({
        "err": 0,
        "id": topic_id,
        "post_id": post_id,
        "topic_slug": topic_slug,
        "msg": "Topic created"
    })))
}

/// PATCH /api/forum/topics/{topicId} — authors edit directly; non-authors submit proposals.
pub async fn update_topic(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    Json(body): Json<UpdateTopicBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let (_, author_id, category_id, _, _) = load_live_topic(&state.db, topic_id).await?;
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }

    let (is_author, is_mod) = (author_id == user_id, is_mod(&auth));
    if !is_author && !is_mod {
        return Err(AppError::Forbidden("Not the topic author".to_string()));
    }

    let cur: (String, String, String) =
        sqlx::query_as("SELECT title, body, created_at::text FROM forum_topics WHERE id = $1")
            .bind(topic_id)
            .fetch_one(&state.db)
            .await?;

    let new_title = match &body.title {
        Some(t) => validate_topic_title(t)?,
        None => cur.0,
    };
    let new_body = match &body.body {
        Some(b) => validate_forum_body(b)?,
        None => cur.1,
    };

    if !is_author {
        let snapshot = json!({"title": new_title, "body": new_body});
        let id: i64 = sqlx::query_scalar("INSERT INTO forum_edit_proposals (target_type,target_id,author_id,snapshot) VALUES ('topic',$1,$2,$3) RETURNING id")
            .bind(topic_id).bind(user_id).bind(&snapshot).fetch_one(&state.db).await?;
        return Ok(Json(
            json!({"err":0,"id":id,"proposal_id":id,"status":"pending","msg":"Edit proposal submitted"}),
        ));
    }
    sqlx::query(
        "UPDATE forum_topics SET title = $2, body = $3,
                updated_at = NOW(), search_vector = to_tsvector('english', $2 || ' ' || $3)
         WHERE id = $1",
    )
    .bind(topic_id)
    .bind(&new_title)
    .bind(&new_body)
    .execute(&state.db)
    .await?;
    // Successful author edits are immutable history entries too.
    if is_author {
        sqlx::query("INSERT INTO forum_edit_proposals (target_type,target_id,author_id,snapshot,status) VALUES ('topic',$1,$2,$3,'approved')")
            .bind(topic_id).bind(user_id).bind(json!({"title": new_title, "body": new_body})).execute(&state.db).await?;
    }

    Ok(Json(
        json!({ "err": 0, "id": topic_id, "msg": "Topic updated" }),
    ))
}
/// Public immutable edit history. Every submitted snapshot is retained.
pub async fn forum_edit_history(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((target_type, target_id)): Path<(String, i64)>,
) -> Result<Json<Value>, AppError> {
    let _ = public_reader_id(&auth, &state)?;
    let rows: Vec<(i64, i32, Option<String>, Value, String, Option<String>, Option<String>, String)> = sqlx::query_as(
        "SELECT p.id,p.author_id,u.username,p.snapshot,p.status,r.username,p.review_note,p.created_at::text FROM forum_edit_proposals p LEFT JOIN users u ON u.id=p.author_id LEFT JOIN users r ON r.id=p.reviewed_by WHERE p.target_type=$1 AND p.target_id=$2 ORDER BY p.created_at ASC, p.id ASC")
        .bind(&target_type).bind(target_id).fetch_all(&state.db).await?;
    Ok(Json(
        json!({"err":0,"items":rows.into_iter().map(|(id,author_id,author_username,snapshot,status,reviewer,review_note,created_at)| json!({"id":id,"author_id":author_id,"author_username":author_username,"snapshot":snapshot,"status":status,"reviewer_username":reviewer,"review_note":review_note,"created_at":created_at})).collect::<Vec<_>>() }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct EditReviewBody {
    pub decision: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// Curator queue; approving applies exactly the retained snapshot atomically.
pub async fn forum_edit_queue(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    require_mod(&auth)?;
    let rows: Vec<(i64,String,i64,i32,Option<String>,Value,String)> = sqlx::query_as("SELECT p.id,p.target_type,p.target_id,p.author_id,u.username,p.snapshot,p.created_at::text FROM forum_edit_proposals p LEFT JOIN users u ON u.id=p.author_id WHERE p.status='pending' ORDER BY p.created_at,p.id").fetch_all(&state.db).await?;
    Ok(Json(
        json!({"err":0,"items":rows.into_iter().map(|(id,target_type,target_id,author_id,username,snapshot,created_at)| json!({"id":id,"target_type":target_type,"target_id":target_id,"author_id":author_id,"author_username":username,"snapshot":snapshot,"created_at":created_at})).collect::<Vec<_>>() }),
    ))
}

pub async fn review_forum_edit(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(proposal_id): Path<i64>,
    Json(body): Json<EditReviewBody>,
) -> Result<Json<Value>, AppError> {
    let reviewer = require_mod(&auth)?;
    if body.decision != "approve" && body.decision != "reject" {
        return Err(AppError::BadRequest(
            "decision must be approve or reject".into(),
        ));
    }
    let mut tx = state.db.begin().await?;
    let row: Option<(String,i64,Value)> = sqlx::query_as("SELECT target_type,target_id,snapshot FROM forum_edit_proposals WHERE id=$1 AND status='pending' FOR UPDATE").bind(proposal_id).fetch_optional(&mut *tx).await?;
    let Some((kind, target, snap)) = row else {
        return Err(AppError::BadRequest("pending proposal not found".into()));
    };
    if body.decision == "approve" {
        if kind == "topic" {
            sqlx::query("UPDATE forum_topics SET title=COALESCE($2->>'title',title),body=COALESCE($2->>'body',body),updated_at=NOW(),search_vector=to_tsvector('english',COALESCE($2->>'title',title)||' '||COALESCE($2->>'body',body)) WHERE id=$1").bind(target).bind(&snap).execute(&mut *tx).await?;
        } else {
            sqlx::query("UPDATE forum_posts SET body=COALESCE($2->>'body',body),edited_at=NOW(),search_vector=to_tsvector('english',COALESCE($2->>'body',body)) WHERE id=$1").bind(target).bind(&snap).execute(&mut *tx).await?;
        }
    }
    sqlx::query("UPDATE forum_edit_proposals SET status=$2,reviewed_by=$3,reviewed_at=NOW(),review_note=$4 WHERE id=$1").bind(proposal_id).bind(&body.decision).bind(reviewer).bind(&body.note).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(
        json!({"err":0,"id":proposal_id,"status":body.decision}),
    ))
}

/// DELETE /api/forum/topics/{topicId} — soft-delete (set deleted_at).
/// Author any time, or mod (role ≥ 5). Mod deletions are logged to modlog.
pub async fn delete_topic(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let (_, author_id, category_id, _, _) = load_live_topic(&state.db, topic_id).await?;
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }

    let is_author = author_id == user_id;
    let mod_delete = is_mod(&auth) && !is_author;
    if !is_author && !is_mod(&auth) {
        return Err(AppError::Forbidden("Not the topic author".to_string()));
    }

    sqlx::query("UPDATE forum_topics SET deleted_at = NOW() WHERE id = $1")
        .bind(topic_id)
        .execute(&state.db)
        .await?;

    if mod_delete {
        let (actor_id, actor_username) = actor_parts(&auth);
        crate::modlog::record_json(
            &state.db,
            actor_id,
            actor_username,
            "forum_topic_delete",
            "forum_topic",
            &topic_id.to_string(),
            vec![],
        )
        .await;
    }

    Ok(Json(
        json!({ "err": 0, "id": topic_id, "msg": "Topic deleted" }),
    ))
}
/// POST /api/forum/topics/{topicId}/posts — reply. Inserts the post, updates
/// topic last_post_id + last_activity_at, then notifies every follower
/// (forum_reply) and every mentioned user (forum_mention).
pub async fn create_post(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    Json(body): Json<CreatePostBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let (_, _, category_id, topic_title, status) = load_live_topic(&state.db, topic_id).await?;
    if status == "locked" && !is_mod(&auth) {
        return Err(AppError::Forbidden("topic_locked".to_string()));
    }
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }
    let post_body = validate_forum_body(&body.body)?;
    let quote_of = body.quote_of;
    let payload = body.payload.unwrap_or(json!({}));

    let mut tx = state.db.begin().await?;

    let post_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_posts (topic_id, author_id, body, payload, quote_of, search_vector)
         VALUES ($1, $2, $3, $4, $5, to_tsvector('english', $3))
         RETURNING id",
    )
    .bind(topic_id)
    .bind(user_id)
    .bind(&post_body)
    .bind(&payload)
    .bind(quote_of)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE forum_topics SET last_post_id = $2, last_activity_at = NOW() WHERE id = $1",
    )
    .bind(topic_id)
    .bind(post_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Notifications (outside the tx — best-effort per contract; each insert
    // is independent and failures must not fail the reply).
    notify_reply(
        &state,
        user_id,
        topic_id,
        category_id,
        &topic_title,
        &post_body,
    )
    .await;

    award_post_exp(&state.db, user_id, post_id).await;

    Ok(Json(
        json!({ "err": 0, "id": post_id, "msg": "Post created" }),
    ))
}

/// Notify followers (forum_reply) + mentioned users (forum_mention) of a new
/// post. Best-effort: never fails the reply itself.
async fn notify_reply(
    state: &AppState,
    author_id: i32,
    topic_id: i64,
    category_id: i64,
    topic_title: &str,
    post_body: &str,
) {
    let (category_slug, topic_slug): (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT c.slug, t.topic_slug FROM forum_categories c JOIN forum_topics t ON t.category_id = c.id WHERE c.id = $1",
    )
    .bind(category_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .unwrap_or((None, None));
    // Link prefers the human-readable board URL; falls back to the numeric
    // path when a legacy row has no slug yet.
    let link = match (category_slug, topic_slug) {
        (Some(_cat), Some(slug)) => Some(format!("/forum/board/{slug}.{topic_id}")),
        (Some(cat), None) => Some(format!("/forum/{cat}/{topic_id}")),
        _ => None,
    };

    // Follower set (excluding the reply author).
    let followers: Vec<i32> = sqlx::query_scalar(
        "SELECT user_id FROM forum_follows WHERE topic_id = $1 AND user_id <> $2",
    )
    .bind(topic_id)
    .bind(author_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let reply_title = format!(
        "{} replied to {}",
        auth_username(&state, author_id).await,
        topic_title
    );
    for follower_id in &followers {
        let _ = queries::create_notification(
            &state.db,
            *follower_id,
            "forum_reply",
            &reply_title,
            None,
            link.as_deref(),
            Some("forum_topic"),
            Some(&topic_id.to_string()),
        )
        .await;
    }

    // @username mentions — dedupe against followers already notified.
    let mut mentioned: std::collections::HashSet<i32> = std::collections::HashSet::new();
    for name in extract_mentions(post_body) {
        let id: Option<i32> = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
            .bind(&name)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
        if let Some(id) = id {
            if id != author_id && !followers.contains(&id) {
                mentioned.insert(id);
            }
        }
    }
    for mid in &mentioned {
        let _ = queries::create_notification(
            &state.db,
            *mid,
            "forum_mention",
            &reply_title,
            None,
            link.as_deref(),
            Some("forum_topic"),
            Some(&topic_id.to_string()),
        )
        .await;
    }
}

/// Resolve a user's username (fallback: "someone") for notification titles.
async fn auth_username(state: &AppState, user_id: i32) -> String {
    sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "someone".to_string())
}

/// PATCH /api/forum/posts/{postId} — authors submit immutable proposals; curators retain direct edits.
pub async fn update_post(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    Json(body): Json<UpdatePostBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let new_body = validate_forum_body(&body.body)?;

    let row: Option<(i32, String, Option<String>)> = sqlx::query_as(
        "SELECT author_id, created_at::text, deleted_at::text
         FROM forum_posts WHERE id = $1",
    )
    .bind(post_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((author_id, _created_at, deleted_at)) = row else {
        return Err(AppError::BadRequest("post not found".to_string()));
    };
    if deleted_at.is_some() {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    let category_id: i64 = sqlx::query_scalar(
        "SELECT t.category_id FROM forum_posts p JOIN forum_topics t ON t.id = p.topic_id WHERE p.id = $1",
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }

    let is_author = author_id == user_id;
    if !is_author && !is_mod(&auth) {
        return Err(AppError::Forbidden("Not the post author".to_string()));
    }
    if !is_author {
        let snapshot = json!({"body": new_body});
        let id: i64 = sqlx::query_scalar("INSERT INTO forum_edit_proposals (target_type,target_id,author_id,snapshot) VALUES ('post',$1,$2,$3) RETURNING id")
            .bind(post_id).bind(user_id).bind(&snapshot).fetch_one(&state.db).await?;
        return Ok(Json(
            json!({"err":0,"id":id,"proposal_id":id,"status":"pending","msg":"Edit proposal submitted"}),
        ));
    }

    sqlx::query(
        "UPDATE forum_posts SET body = $2, edited_at = NOW(),
                search_vector = to_tsvector('english', $2)
         WHERE id = $1",
    )
    .bind(post_id)
    .bind(&new_body)
    .execute(&state.db)
    .await?;
    if is_author {
        sqlx::query("INSERT INTO forum_edit_proposals (target_type,target_id,author_id,snapshot,status) VALUES ('post',$1,$2,$3,'approved')")
            .bind(post_id).bind(user_id).bind(json!({"body": new_body})).execute(&state.db).await?;
    }

    Ok(Json(
        json!({ "err": 0, "id": post_id, "msg": "Post updated" }),
    ))
}

/// DELETE /api/forum/posts/{postId} — soft-delete (author any time or mod).
/// Mod deletions are logged to modlog.
pub async fn delete_post(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let row: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT author_id, deleted_at::text FROM forum_posts WHERE id = $1")
            .bind(post_id)
            .fetch_optional(&state.db)
            .await?;
    let Some((author_id, deleted_at)) = row else {
        return Err(AppError::BadRequest("post not found".to_string()));
    };
    if deleted_at.is_some() {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    let category_id: i64 = sqlx::query_scalar(
        "SELECT t.category_id FROM forum_posts p JOIN forum_topics t ON t.id = p.topic_id WHERE p.id = $1",
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;
    if is_banned(&state, user_id, category_id).await? {
        return Err(AppError::Forbidden("Banned from the forum".to_string()));
    }

    let is_author = author_id == user_id;
    let mod_delete = is_mod(&auth) && !is_author;
    if !is_author && !is_mod(&auth) {
        return Err(AppError::Forbidden("Not the post author".to_string()));
    }

    sqlx::query("UPDATE forum_posts SET deleted_at = NOW() WHERE id = $1")
        .bind(post_id)
        .execute(&state.db)
        .await?;

    if mod_delete {
        let (actor_id, actor_username) = actor_parts(&auth);
        crate::modlog::record_json(
            &state.db,
            actor_id,
            actor_username,
            "forum_post_delete",
            "forum_post",
            &post_id.to_string(),
            vec![],
        )
        .await;
    }

    Ok(Json(
        json!({ "err": 0, "id": post_id, "msg": "Post deleted" }),
    ))
}

/// POST /api/forum/topics/{topicId}/follow — toggle follow. Returns
/// {following, follower_count}.
/// GET /api/forum/topics/{topicId}/follow — read the current user's follow
/// state for a topic + the follower count. Needed so the UI can render the
/// follow button correctly on load (the toggle alone doesn't reveal state).
pub async fn get_follow_state(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    load_live_topic(&state.db, topic_id).await?;

    let following: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_follows WHERE user_id = $1 AND topic_id = $2)",
    )
    .bind(user_id)
    .bind(topic_id)
    .fetch_one(&state.db)
    .await?;

    let follower_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM forum_follows WHERE topic_id = $1")
            .bind(topic_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(json!({
        "err": 0,
        "following": following,
        "follower_count": follower_count,
    })))
}

pub async fn toggle_follow(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    load_live_topic(&state.db, topic_id).await?;

    let deleted: i64 = sqlx::query("DELETE FROM forum_follows WHERE user_id = $1 AND topic_id = $2")
        .bind(user_id)
        .bind(topic_id)
        .execute(&state.db)
        .await?
        .rows_affected() as i64;

    let following = if deleted == 0 {
        sqlx::query("INSERT INTO forum_follows (user_id, topic_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(topic_id)
            .execute(&state.db)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(ref de) if de.is_unique_violation() => {
                    AppError::BadRequest("already following".to_string())
                }
                _ => AppError::from(e),
            })?;
        true
    } else {
        false
    };

    let follower_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM forum_follows WHERE topic_id = $1")
            .bind(topic_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(json!({
        "err": 0,
        "following": following,
        "follower_count": follower_count,
    })))
}

/// POST /api/forum/topics/{topicId}/read — mark a topic read for the current
/// user. Body: `{last_read_post_id: N}` (optional — defaults to the topic's
/// current `last_post_id`). Upserts `forum_read_state`; `last_read_post_id`
/// is monotonic (GREATEST of existing/new) so stale client payloads never
/// move the marker backwards. Requires auth (401 anonymous).
pub async fn mark_topic_read(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(topic_id): Path<i64>,
    Json(body): Json<MarkReadBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    // 400 on missing/deleted topics (repo convention, like other forum routes).
    load_live_topic(&state.db, topic_id).await?;

    let target: Option<i64> = match body.last_read_post_id {
        Some(n) => Some(n),
        None => {
            sqlx::query_scalar("SELECT last_post_id FROM forum_topics WHERE id = $1")
                .bind(topic_id)
                .fetch_one(&state.db)
                .await?
        }
    };

    let row: (i64, String) = sqlx::query_as(
        r#"INSERT INTO forum_read_state (user_id, topic_id, last_read_post_id)
           VALUES ($1, $2, $3)
           ON CONFLICT (user_id, topic_id) DO UPDATE SET
               last_read_post_id = GREATEST(forum_read_state.last_read_post_id, EXCLUDED.last_read_post_id),
               updated_at = NOW()
           RETURNING last_read_post_id, updated_at::text"#,
    )
    .bind(user_id)
    .bind(topic_id)
    .bind(target)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "last_read_post_id": row.0,
        "updated_at": row.1,
    })))
}

/// GET /api/forum/search?q=...&category={slug?}&limit=... — full-text search
/// over topic OPs (title + body) and post replies. Auth-gated (401
/// anonymous), rate-limited on the Default tier like the other forum routes.
///
/// Query syntax reuses the boolean parser (`src/search/parser.rs`); the
/// emitted tsquery string is safe to pass through `to_tsquery($1::text)`.
/// When parsing yields nothing usable (e.g. a bare exclusion) the handler
/// falls back to `plainto_tsquery('english', q)` so simple queries always
/// work. Matches rank by `ts_rank`; snippets are `ts_headline`-highlighted
/// (`<mark>` wraps hits). Pagination is plain limit/offset capped at 50
/// (deterministic ordering: rank DESC, post id DESC, topic id DESC) —
/// `next_cursor` is always null.
pub async fn search_forum(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchForumParams>,
) -> Result<Json<Value>, AppError> {
    // Public read: anonymous visitors may search public (non-mod-only)
    // categories. FORUM_PUBLIC_READ=false → login-only.
    let user_id = public_reader_id(&auth, &state)?;
    let is_anon = user_id == 0;

    let q = params.q.trim();
    if q.is_empty() {
        return Err(AppError::BadRequest("q must not be empty".to_string()));
    }
    let limit = params.limit.unwrap_or(20).clamp(1, 50);

    use crate::limiter::Tier;
    let ip = crate::limiter::client_ip_from_headers(
        None,
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    if let crate::limiter::TieredRateLimitResult::Wait(secs) =
        state.rate_limiter.check(ip, None, Tier::Default).await
    {
        return Err(AppError::RateLimited(secs));
    }

    // tsquery from the boolean parser (safe: parser only emits word/phrase/
    // exclusion tokens). Empty result → plainto fallback.
    let tsquery = crate::search::parser::expr_to_tsquery(&crate::search::parser::parse_query(q));
    let tsquery = if tsquery.trim().is_empty() {
        q.to_string()
    } else {
        tsquery
    };

    let category_filter = match &params.category {
        Some(slug) if !slug.trim().is_empty() => {
            Some(category_id_by_slug(&state, slug.trim()).await?)
        }
        _ => None,
    };
    // Anonymous searches exclude mod-only categories entirely.
    let anon_public_only = is_anon;

    let topic_hits: Vec<(
        i64,
        Option<String>,
        i64,
        Option<i64>,
        i32,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        f32,
    )> = sqlx::query_as(
        r#"SELECT t.id, t.topic_slug, t.id, NULL::bigint, t.author_id, u.username, t.title, t.body,
                  ts_headline('english', t.title || ' ' || t.body, to_tsquery($1::text),
                              'StartSel=<mark>, StopSel=</mark>, MaxWords=25, MinWords=5'),
                  c.slug, c.title, t.created_at::text, ts_rank(t.search_vector, to_tsquery($1::text))
           FROM forum_topics t
           JOIN users u ON u.id = t.author_id
           JOIN forum_categories c ON c.id = t.category_id
           WHERE t.search_vector @@ to_tsquery($1::text)
             AND t.deleted_at IS NULL AND t.is_hidden = FALSE
             AND ($2::bigint IS NULL OR t.category_id = $2)
             AND ($4::boolean OR c.is_mod_only = FALSE)
           ORDER BY ts_rank(t.search_vector, to_tsquery($1::text)) DESC, t.id DESC
           LIMIT $3"#,
    )
    .bind(&tsquery)
    .bind(category_filter)
    .bind(limit + 1)
    .bind(anon_public_only)
    .fetch_all(&state.db)
    .await?;

    let post_hits: Vec<(
        i64,
        Option<String>,
        i64,
        Option<i64>,
        i32,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        f32,
    )> = sqlx::query_as(
        r#"SELECT t.id, t.topic_slug, p.id, p.id, p.author_id, u.username, t.title, p.body,
                  ts_headline('english', p.body, to_tsquery($1::text),
                              'StartSel=<mark>, StopSel=</mark>, MaxWords=25, MinWords=5'),
                  c.slug, c.title, p.created_at::text, ts_rank(p.search_vector, to_tsquery($1::text))
           FROM forum_posts p
           JOIN forum_topics t ON t.id = p.topic_id
           JOIN users u ON u.id = p.author_id
           JOIN forum_categories c ON c.id = t.category_id
           WHERE p.search_vector @@ to_tsquery($1::text)
             AND p.deleted_at IS NULL AND p.is_hidden = FALSE
             AND t.deleted_at IS NULL AND t.is_hidden = FALSE
             AND ($2::bigint IS NULL OR t.category_id = $2)
             AND ($4::boolean OR c.is_mod_only = FALSE)
           ORDER BY ts_rank(p.search_vector, to_tsquery($1::text)) DESC, p.id DESC
           LIMIT $3"#,
    )
    .bind(&tsquery)
    .bind(category_filter)
    .bind(limit + 1)
    .bind(anon_public_only)
    .fetch_all(&state.db)
    .await?;

    // Merge + rank, then take `limit` (deterministic: rank DESC, then post id
    // DESC — topic hits have post_id NULL → -1 — then topic id DESC).
    let mut merged: Vec<(
        f32,
        i64,
        i64,
        i64,
        Option<i64>,
        i32,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
        String,
        String,
    )> = Vec::with_capacity(topic_hits.len() + post_hits.len());
    for (
        topic_id,
        topic_slug,
        _t,
        _p,
        author_id,
        author_username,
        title,
        body,
        snippet,
        category_slug,
        category_title,
        created_at,
        rank,
    ) in topic_hits
    {
        merged.push((
            rank,
            topic_id,
            -1,
            topic_id,
            None,
            author_id,
            author_username,
            title,
            body,
            topic_slug,
            snippet,
            category_slug,
            category_title,
            created_at,
        ));
    }
    for (
        topic_id,
        topic_slug,
        _t,
        post_id,
        author_id,
        author_username,
        title,
        body,
        snippet,
        category_slug,
        category_title,
        created_at,
        rank,
    ) in post_hits
    {
        let pid = post_id.unwrap_or(0);
        merged.push((
            rank,
            topic_id,
            pid,
            topic_id,
            post_id,
            author_id,
            author_username,
            title,
            body,
            topic_slug,
            snippet,
            category_slug,
            category_title,
            created_at,
        ));
    }
    merged.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| b.1.cmp(&a.1))
    });

    let total = merged.len() as i64;
    let results: Vec<Value> = merged
        .into_iter()
        .take(limit as usize)
        .map(
            |(
                _rank,
                _topic_id,
                _pid,
                topic_id,
                post_id,
                author_id,
                author_username,
                title,
                body,
                topic_slug,
                snippet,
                category_slug,
                category_title,
                created_at,
            )| {
                json!({
                    "type": if post_id.is_some() { "post" } else { "topic" },
                    "topic_id": topic_id,
                    "topic_slug": topic_slug,
                    "post_id": post_id,
                    "author_id": author_id,
                    "author_username": author_username,
                    "title": title,
                    "body": body,
                    "snippet": snippet,
                    "category_slug": category_slug,
                    "category_title": category_title,
                    "created_at": created_at,
                })
            },
        )
        .collect();

    Ok(Json(json!({
        "err": 0,
        "q": q,
        "results": results,
        "next_cursor": Value::Null,
        "limit": limit,
        "total": total,
    })))
}

// ── Emoji reactions ──────────────────────────────────────────────────

/// Allowed emoji reactions for forum posts (positive-only policy — no
/// downvote-style emojis). Keep this list short for UI ergonomics.
pub const ALLOWED_REACTIONS: &[&str] = &[
    "👍", // like / agree
    "❤️", // love / support
    "😂", // funny
    "🔥", // hot / exciting
    "👏", // applause / well said
    "🤔", // thinking / interesting
    "😢", // sad / sympathetic
    "😮", // surprised / wow
];

/// POST /api/forum/posts/{postId}/react — toggle an emoji reaction.
/// Body: { emoji: "👍" }.
/// If the user already reacted with that emoji, remove it (toggle off).
pub async fn react_to_post(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    auth: AuthUser,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let emoji = body
        .get("emoji")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !ALLOWED_REACTIONS.contains(&emoji.as_str()) {
        return Err(AppError::BadRequest("invalid emoji".to_string()));
    }
    // Check post exists and is not deleted
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_posts WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(post_id)
    .fetch_one(&state.db)
    .await?;
    if !exists {
        return Err(AppError::BadRequest("post not found".to_string()));
    }
    // Toggle: try delete first; if 0 rows deleted, insert
    let deleted = sqlx::query(
        "DELETE FROM forum_post_reactions WHERE post_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind(post_id)
    .bind(user_id)
    .bind(&emoji)
    .execute(&state.db)
    .await?
    .rows_affected();
    if deleted == 0 {
        sqlx::query(
            "INSERT INTO forum_post_reactions (post_id, user_id, emoji) VALUES ($1, $2, $3) \
             ON CONFLICT DO NOTHING",
        )
        .bind(post_id)
        .bind(user_id)
        .bind(&emoji)
        .execute(&state.db)
        .await?;
    }
    // Return updated reactions for this post
    let reactions = load_reactions(&state.db, post_id, Some(user_id)).await?;
    Ok(Json(json!({ "err": 0, "reactions": reactions })))
}

/// GET /api/forum/posts/{postId}/reactions — read reactions for a post.
/// Public-read friendly: anonymous callers get counts + empty `my_reactions`.
pub async fn get_post_reactions(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let reactions = load_reactions(&state.db, post_id, auth.user_id).await?;
    Ok(Json(json!({ "err": 0, "reactions": reactions })))
}

/// DELETE /api/forum/posts/{postId}/reactions?emoji=👍 — remove a reaction
/// (idempotent; succeeds even if the reaction was not present).
pub async fn remove_post_reaction(
    State(state): State<Arc<AppState>>,
    Path(post_id): Path<i64>,
    auth: AuthUser,
    Query(params): Query<ReactionQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;
    let emoji = params.emoji.trim().to_string();
    if !ALLOWED_REACTIONS.contains(&emoji.as_str()) {
        return Err(AppError::BadRequest("invalid emoji".to_string()));
    }
    sqlx::query(
        "DELETE FROM forum_post_reactions WHERE post_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind(post_id)
    .bind(user_id)
    .bind(&emoji)
    .execute(&state.db)
    .await?;
    let reactions = load_reactions(&state.db, post_id, Some(user_id)).await?;
    Ok(Json(json!({ "err": 0, "reactions": reactions })))
}

#[derive(Debug, Deserialize)]
pub struct ReactionQuery {
    pub emoji: String,
}

async fn load_reactions(
    db: &sqlx::PgPool,
    post_id: i64,
    viewer: Option<i32>,
) -> Result<Value, AppError> {
    // Each emoji → list of { user_id, username } who reacted
    let rows: Vec<(String, i32, String)> = sqlx::query_as(
        "SELECT r.emoji, r.user_id, u.username FROM forum_post_reactions r \
         JOIN users u ON u.id = r.user_id \
         WHERE r.post_id = $1 ORDER BY r.emoji, r.created_at",
    )
    .bind(post_id)
    .fetch_all(db)
    .await?;
    let mut grouped: std::collections::HashMap<String, Vec<Value>> =
        std::collections::HashMap::new();
    for (emoji, uid, username) in rows {
        grouped
            .entry(emoji)
            .or_default()
            .push(json!({ "user_id": uid, "username": username }));
    }
    let my_reactions: Vec<String> = if let Some(uid) = viewer {
        sqlx::query_scalar(
            "SELECT emoji FROM forum_post_reactions WHERE post_id = $1 AND user_id = $2",
        )
        .bind(post_id)
        .bind(uid)
        .fetch_all(db)
        .await?
    } else {
        vec![]
    };
    Ok(json!({ "reactions": grouped, "my_reactions": my_reactions }))
}

async fn load_reactions_batch(
    db: &sqlx::PgPool,
    post_ids: &[i64],
    viewer: Option<i32>,
) -> Result<std::collections::HashMap<i64, Value>, AppError> {
    if post_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    // Reaction details per post: emoji, user_id, username
    let rows: Vec<(i64, String, i32, String)> = sqlx::query_as(
        "SELECT r.post_id, r.emoji, r.user_id, u.username FROM forum_post_reactions r \
         JOIN users u ON u.id = r.user_id \
         WHERE r.post_id = ANY($1) ORDER BY r.post_id, r.emoji, r.created_at",
    )
    .bind(post_ids)
    .fetch_all(db)
    .await?;
    let mut map: std::collections::HashMap<i64, std::collections::HashMap<String, Vec<Value>>> =
        std::collections::HashMap::new();
    for (pid, emoji, uid, username) in rows {
        map.entry(pid)
            .or_default()
            .entry(emoji)
            .or_default()
            .push(json!({ "user_id": uid, "username": username }));
    }
    // My reactions per post
    let my_rows: Vec<(i64, String)> = if let Some(uid) = viewer {
        sqlx::query_as(
            "SELECT post_id, emoji FROM forum_post_reactions \
             WHERE post_id = ANY($1) AND user_id = $2",
        )
        .bind(post_ids)
        .bind(uid)
        .fetch_all(db)
        .await?
    } else {
        vec![]
    };
    let mut my_map: std::collections::HashMap<i64, Vec<String>> = std::collections::HashMap::new();
    for (pid, emoji) in my_rows {
        my_map.entry(pid).or_default().push(emoji);
    }
    // Merge
    let mut result = std::collections::HashMap::new();
    let all_ids: std::collections::HashSet<i64> = post_ids.iter().copied().collect();
    for pid in all_ids {
        let reactions = map.remove(&pid).unwrap_or_default();
        let my = my_map.remove(&pid).unwrap_or_default();
        result.insert(pid, json!({ "reactions": reactions, "my_reactions": my }));
    }
    Ok(result)
}
