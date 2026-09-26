//! Trust system — a Discourse-inspired 7-level axis separate from
//! rank/reputation.
//!
//! Trust measures *participation safety*: what a member may do and how much
//! their moderation actions weigh. Levels 0-4 are earned automatically from
//! reading + community signals via [`run_trust_promotion`]; levels 5-6 are
//! staff-designated (the weekly digest surfaces TL5 candidates for a quick
//! admin confirmation — the "<15 min/week" human layer).
//!
//! Design invariants:
//! - Trust is monotonic up to TL4 (once earned, never lost) matching
//!   Discourse, EXCEPT a revocation at TL3+ if a member accrues confirmed
//!   spam reports (the only automatic demotion).
//! - Trust level replaces legacy `role` for all access control.
//!   Mapping: role 0→TL0, role 1→TL1, role 5→TL3, role 10→TL5.
//!   Admin tools require TL5+ (Community Moderator), curator tools TL3+.
//!
//! Migration from `role`:
//! - `role < 5` (reader) → `trust_level < 3`
//! - `role >= 5` (curator) → `trust_level >= 3`
//! - `role >= 10` (admin) → `trust_level >= 5`

use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

/// Display names for each trust level (index = level).
pub const TRUST_NAMES: [&str; 7] = [
    "New",
    "Basic",
    "Member",
    "Regular",
    "Elder",
    "Community Moderator",
    "Near-admin",
];

/// The minimum trust level required to publish a shareable object
/// (skin / recipe / theme / layout). Publishing is a "social write": the
/// object becomes visible to everyone, so it is gated above the bare write
/// tier. TL0/1 can draft privately; TL2+ may publish.
pub const PUBLISH_MIN_TRUST: i16 = 2;

/// Minimum trust to resolve other users' reports (the Community Moderator
/// human layer).
pub const RESOLVE_MIN_TRUST: i16 = 5;

/// Minimum trust for admin tools (replaces legacy role >= 10).
pub const ADMIN_MIN_TRUST: i16 = 5;

/// Minimum trust for curator tools (replaces legacy role >= 5).
pub const CURATOR_MIN_TRUST: i16 = 3;

/// Flag weight granted per trust level (reporter trust → effective weight):
/// TL1=1, TL2=1, TL3=2, TL4=3, TL5=5, TL6=5. TL0 may not flag.
pub fn flag_weight(level: i16) -> i32 {
    match level {
        0 => 0,
        1 | 2 => 1,
        3 => 2,
        4 => 3,
        5 | 6 => 5,
        _ => 1,
    }
}

/// Aggregated participation metrics that drive trust promotions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TlMetrics {
    pub works_entered: i64,
    pub works_read: i64,
    pub words_read: i64,
    pub days_active_30: i64,
    pub forum_posts: i64,
    pub reviews: i64,
    pub reports_filed: i64,
    pub reports_received_30d: i64,
}

fn clamp(l: i16) -> usize {
    l.clamp(0, 6) as usize
}

/// Fetch a user's current trust level (0 for anonymous/missing).
pub async fn fetch_trust_level(db: &PgPool, user_id: Option<i32>) -> i16 {
    let Some(uid) = user_id else {
        return 0;
    };
    sqlx::query_scalar("SELECT trust_level FROM users WHERE id = $1")
        .bind(uid)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
}

/// Gate: require `min` trust or return `Forbidden`. Used at write boundaries
/// so new (TL0) accounts are sandboxed without blocking reads.
pub async fn assert_min_trust(
    db: &PgPool,
    user_id: Option<i32>,
    min: i16,
    what: &str,
) -> Result<i16, AppError> {
    let level = fetch_trust_level(db, user_id).await;
    if level < min {
        return Err(AppError::Forbidden(format!(
            "{what} requires trust level {min} ({}) or higher; you are at {level} ({}). \
             Keep reading and participating to rise.",
            TRUST_NAMES[clamp(min)],
            TRUST_NAMES[clamp(level)],
        )));
    }
    Ok(level)
}

/// Trust gate that never blocks staff: `trust_level >= 5` (Community Mod+)
/// is inherently trusted and always passes. Regular users must reach `min`.
/// Used by trust-gated write surfaces so a TL0 admin isn't locked out of
/// publishing/moderation they're authorised for.
pub async fn assert_staff_or_min_trust(
    db: &PgPool,
    user_id: Option<i32>,
    trust_level: i16,
    min: i16,
    what: &str,
) -> Result<i16, AppError> {
    if trust_level >= 5 {
        return Ok(fetch_trust_level(db, user_id).await.max(min));
    }
    assert_min_trust(db, user_id, min, what).await
}

/// Upsert a user's cached `tl_metrics` jsonb. Called after reads/engagement.
pub async fn refresh_metrics(db: &PgPool, user_id: i32) -> AppResult<()> {
    let m = compute_metrics(db, user_id).await?;
    let json = serde_json::to_value(&m)
        .map_err(|e| AppError::Internal(format!("tl_metrics serialize: {e}")))?;
    sqlx::query("UPDATE users SET tl_metrics = $1, tl_updated_at = NOW() WHERE id = $2")
        .bind(json)
        .bind(user_id)
        .execute(db)
        .await
        .map_err(|e| db_err("refresh tl_metrics", e))?;
    Ok(())
}

/// Compute the raw metrics that drive trust promotions from live tables.
pub async fn compute_metrics(db: &PgPool, user_id: i32) -> AppResult<TlMetrics> {
    let works_entered: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT work_id) FROM reading_history WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| db_err("works_entered", e))?;

    // total_works_read is INT4, total_words_read is INT8 — decode separately.
    let (works_read,): (i32,) = sqlx::query_as("SELECT total_works_read FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(|e| db_err("user works_read col", e))?
        .unwrap_or((0,));
    let words_read: i64 = sqlx::query_scalar("SELECT total_words_read FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(db)
        .await
        .map_err(|e| db_err("user words_read col", e))?
        .unwrap_or(0);
    let works_read = works_read as i64;

    let days_active_30: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT visited_at::date) FROM reading_history
         WHERE user_id = $1 AND visited_at > NOW() - interval '30 days'",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| db_err("days_active_30", e))?;

    let forum_posts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM forum_posts WHERE author_id = $1 AND deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| db_err("forum_posts", e))?;

    let reviews: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(db)
        .await
        .map_err(|e| db_err("reviews", e))?;

    let reports_filed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_reports WHERE reporter_id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await
            .map_err(|e| db_err("reports_filed", e))?;

    let reports_received_30d: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_reports
         WHERE target_type = 'user' AND target_id = $1
           AND created_at > NOW() - interval '30 days'",
    )
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| db_err("reports_received_30d", e))?;

    Ok(TlMetrics {
        works_entered,
        works_read,
        words_read,
        days_active_30,
        forum_posts,
        reviews,
        reports_filed,
        reports_received_30d,
    })
}

/// Load cached metrics, falling back to a live computation.
pub async fn load_or_compute_metrics(db: &PgPool, user_id: i32) -> AppResult<TlMetrics> {
    let cached: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT tl_metrics FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(db)
            .await
            .map_err(|e| db_err("load tl_metrics", e))?;
    if let Some(v) = cached {
        if !v.is_null() {
            if let Ok(m) = serde_json::from_value::<TlMetrics>(v) {
                return Ok(m);
            }
        }
    }
    compute_metrics(db, user_id).await
}

fn db_err(what: &str, e: sqlx::Error) -> AppError {
    AppError::Database(format!("{what}: {e}"))
}

/// Record a trust-level change in `trust_events` and update `users`.
/// `by_user_id = None` means automatic (system). The legacy `trust` int is
/// kept in sync so existing feature-gates (rank/trust) also pick up levels.
pub async fn set_trust_level(
    db: &PgPool,
    user_id: i32,
    to_level: i16,
    reason: &str,
    by_user_id: Option<i32>,
    actor_username: Option<String>,
) -> AppResult<i16> {
    let to_level = to_level.clamp(0, 6);
    let from_level = fetch_trust_level(db, Some(user_id)).await;
    if from_level == to_level {
        return Ok(to_level);
    }

    let mut tx = db.begin().await.map_err(|e| db_err("trust tx begin", e))?;

    // Increment trust_loss_count on demotion (for diminishing recovery)
    if to_level < from_level {
        sqlx::query(
            "UPDATE users SET trust_level = $1, trust = $1, tl_notes = $2, trust_loss_count = trust_loss_count + 1 WHERE id = $3",
        )
        .bind(to_level)
        .bind(reason)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| db_err("trust update", e))?;
    } else {
        sqlx::query("UPDATE users SET trust_level = $1, trust = $1, tl_notes = $2 WHERE id = $3")
            .bind(to_level)
            .bind(reason)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| db_err("trust update", e))?;
    }

    sqlx::query(
        "INSERT INTO trust_events (user_id, from_level, to_level, reason, by_user_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(from_level)
    .bind(to_level)
    .bind(reason)
    .bind(by_user_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| db_err("trust event insert", e))?;
    tx.commit()
        .await
        .map_err(|e| db_err("trust tx commit", e))?;

    if let Some(actor) = actor_username {
        crate::modlog::record(
            db,
            by_user_id,
            Some(actor),
            "trust_level_change",
            "user",
            &user_id.to_string(),
            json!({ "from": from_level, "to": to_level, "reason": reason }),
        )
        .await;
    }

    Ok(to_level)
}

/// Promotion threshold check. Higher levels supersede lower ones.
fn meets_level(m: &TlMetrics, target: i16) -> bool {
    match target {
        1 => m.works_entered >= 5 && m.words_read >= 30_000,
        2 => {
            m.days_active_30 >= 5
                && m.works_entered >= 25
                && m.words_read >= 100_000
                && m.forum_posts >= 1
        }
        3 => {
            m.days_active_30 >= 15
                && m.works_entered >= 100
                && m.words_read >= 400_000
                && m.forum_posts >= 10
                && m.reports_received_30d < 2
        }
        4 => {
            m.works_read >= 500
                && m.words_read >= 1_000_000
                && m.forum_posts >= 50
                && m.reports_received_30d < 2
        }
        _ => false,
    }
}

/// Run the automatic promotion pass over users below TL5 (levels 5-6 are
/// staff-designated via the digest). Refreshes each user's metrics, raises
/// TL0→TL4 where thresholds are met, and collects TL5 candidates (returned,
/// not auto-promoted) for the weekly digest.
///
/// Returns `(promotions, tl5_candidates)`.
pub async fn run_trust_promotion(
    db: &PgPool,
    config: &crate::config::Config,
) -> AppResult<(Vec<(i32, i16, i16, String)>, Vec<(i32, TlMetrics)>)> {
    let user_ids: Vec<i32> =
        sqlx::query_scalar("SELECT id FROM users WHERE is_banned = false AND trust_level < 5")
            .fetch_all(db)
            .await
            .map_err(|e| db_err("trust promote user list", e))?;

    let mut promotions = Vec::new();
    let mut tl5_candidates = Vec::new();

    for uid in user_ids {
        let current = fetch_trust_level(db, Some(uid)).await;
        if current >= 5 {
            continue;
        }
        let m = compute_metrics(db, uid).await?;
        refresh_metrics(db, uid).await?;

        // Apply diminishing recovery: reduce effective metrics based on loss count.
        // Each trust loss event reduces effective metrics by trust_recovery_decay^loss_count.
        // Example: decay=0.7, losses=1 → metrics * 0.7; losses=2 → metrics * 0.49.
        let loss_count: i32 =
            sqlx::query_scalar("SELECT COALESCE(trust_loss_count, 0) FROM users WHERE id = $1")
                .bind(uid)
                .fetch_one(db)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);

        let decay_factor = if loss_count > 0 {
            config.trust_recovery_decay.powi(loss_count)
        } else {
            1.0
        };

        let adjusted_m = TlMetrics {
            works_entered: (m.works_entered as f64 * decay_factor) as i64,
            works_read: (m.works_read as f64 * decay_factor) as i64,
            words_read: (m.words_read as f64 * decay_factor) as i64,
            days_active_30: (m.days_active_30 as f64 * decay_factor) as i64,
            forum_posts: (m.forum_posts as f64 * decay_factor) as i64,
            reports_received_30d: m.reports_received_30d, // spam penalties are absolute, not decayed
            ..m.clone()
        };

        let target = if meets_level(&adjusted_m, 4) {
            4
        } else if meets_level(&adjusted_m, 3) {
            3
        } else if meets_level(&adjusted_m, 2) {
            2
        } else if meets_level(&adjusted_m, 1) {
            1
        } else {
            0
        };

        if target > current {
            let reason = match target {
                1 => "read 5+ works and 30k+ words",
                2 => "active 5+ days, read 100k+ words, posted in forum",
                3 => "consistently active, 100+ works, 400k+ words, 10+ posts",
                4 => "500+ works read, 1M+ words, 50+ posts",
                _ => "onboarding read",
            };
            set_trust_level(db, uid, target, reason, None, None).await?;
            promotions.push((uid, current, target, reason.to_string()));
        } else if current == 4 {
            if m.days_active_30 >= 20
                && m.forum_posts >= 100
                && m.reports_received_30d < 2
                && m.reports_filed >= 5
            {
                tl5_candidates.push((uid, m));
            }
        }
    }

    Ok((promotions, tl5_candidates))
}

/// Revoke trust where a member accrues confirmed spam reports (the only
/// automatic demotion). Returns the demotions as `(user_id, from, to, reason)`.
pub async fn run_trust_revocation(db: &PgPool) -> AppResult<Vec<(i32, i16, i16, String)>> {
    let rows: Vec<(i32, i16, i64)> = sqlx::query_as(
        "SELECT u.id, u.trust_level,
                (SELECT COUNT(*) FROM user_reports r
                 WHERE r.target_type='user' AND r.target_id = u.id
                   AND r.created_at > NOW() - interval '90 days') AS spam
         FROM users u
         WHERE u.trust_level >= 3 AND u.is_banned = false",
    )
    .fetch_all(db)
    .await
    .map_err(|e| db_err("trust revocation list", e))?;

    let mut demotions = Vec::new();
    for (uid, current, spam) in rows {
        if spam >= 3 && current > 2 {
            let to = current - 1;
            let reason = format!("{spam} confirmed spam reports in the last 90 days");
            set_trust_level(db, uid, to, &reason, None, None).await?;
            demotions.push((uid, current, to, reason));
        }
    }
    Ok(demotions)
}

/// Grant a preference-similarity trust boost when a user interacts with a work
/// (bookmark, rate, kudos). Compares the user's tag preferences with the
/// author's most popular tags and grants a small boost if they align.
///
/// Returns the boost amount applied (in trust-level thousandths).
pub async fn preference_similarity_boost(
    db: &PgPool,
    config: &crate::config::Config,
    user_id: i32,
    work_url_id: &str,
) -> i64 {
    if !config.trust_preference_similarity_enabled {
        return 0;
    }

    let user_id = match user_id {
        id if id > 0 => id,
        _ => return 0,
    };

    // Get the user's most-voted tags (their preferences)
    let user_tags: Vec<(String,)> = match sqlx::query_as(
        r#"SELECT t.name
           FROM fic_tag_votes v
           JOIN tags t ON t.id = v.tag_id
           WHERE v.user_id = $1 AND v.value > 0
           ORDER BY v.value DESC, v.created_at DESC
           LIMIT 20"#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return 0,
    };

    if user_tags.is_empty() {
        return 0;
    }

    // Get the work's tags
    let work_tags: Vec<(String,)> = match sqlx::query_as(
        r#"SELECT t.name
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY ft.score DESC
           LIMIT 20"#,
    )
    .bind(work_url_id)
    .fetch_all(db)
    .await
    {
        Ok(rows) => rows,
        Err(_) => return 0,
    };

    if work_tags.is_empty() {
        return 0;
    }

    // Compute Jaccard similarity between user preferences and work tags
    let user_set: std::collections::HashSet<&str> =
        user_tags.iter().map(|(n,)| n.as_str()).collect();
    let work_set: std::collections::HashSet<&str> =
        work_tags.iter().map(|(n,)| n.as_str()).collect();

    let intersection = user_set.intersection(&work_set).count();
    let union = user_set.union(&work_set).count();

    if union == 0 {
        return 0;
    }

    let similarity = intersection as f64 / union as f64;

    // Only boost if similarity is meaningful (>= 0.2)
    if similarity < 0.2 {
        return 0;
    }

    let boost = (similarity * config.trust_similarity_boost as f64) as i64;
    if boost > 0 {
        tracing::debug!(
            "preference similarity boost for user {}: sim={:.2}, boost={}",
            user_id,
            similarity,
            boost
        );
    }
    boost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_names_cover_seven_levels() {
        assert_eq!(TRUST_NAMES.len(), 7);
        assert_eq!(TRUST_NAMES[0], "New");
        assert_eq!(TRUST_NAMES[6], "Near-admin");
    }

    #[test]
    fn flag_weight_scales_with_trust() {
        assert_eq!(flag_weight(0), 0);
        assert_eq!(flag_weight(1), 1);
        assert_eq!(flag_weight(3), 2);
        assert_eq!(flag_weight(4), 3);
        assert_eq!(flag_weight(5), 5);
        assert_eq!(flag_weight(6), 5);
    }

    fn m(entered: i64, wr: i64, words: i64, days: i64, posts: i64, spam: i64) -> TlMetrics {
        TlMetrics {
            works_entered: entered,
            works_read: wr,
            words_read: words,
            days_active_30: days,
            forum_posts: posts,
            reports_received_30d: spam,
            ..Default::default()
        }
    }

    #[test]
    fn promotion_is_monotonic() {
        // A brand-new reader does not qualify for any level.
        assert!(!meets_level(&m(0, 0, 0, 0, 0, 0), 1));
        // Meets TL1 but not TL2.
        assert!(meets_level(&m(10, 0, 50_000, 0, 0, 0), 1));
        assert!(!meets_level(&m(10, 0, 50_000, 0, 0, 0), 2));
        // Meets TL2.
        assert!(meets_level(&m(30, 0, 150_000, 6, 2, 0), 2));
        assert!(!meets_level(&m(30, 0, 150_000, 6, 2, 0), 3));
        // Meets TL4.
        assert!(meets_level(&m(600, 600, 1_200_000, 20, 60, 0), 4));
    }

    #[test]
    fn spam_flags_block_promotion() {
        // Even with huge volume, recent spam reports block TL3/TL4.
        assert!(!meets_level(&m(600, 600, 1_200_000, 20, 60, 3), 3));
        assert!(!meets_level(&m(600, 600, 1_200_000, 20, 60, 3), 4));
    }
}


/// Gate an administrator-tier route.
///
/// Reads `AuthUser.trust_level` — the **JWT claim** — not the database column.
/// This distinction is load-bearing: `assert_min_trust` above reads
/// `users.trust_level` from the database, but the highest value any account can
/// reach there in practice is 5, so a database-backed admin gate would lock out
/// every administrator. JWT issuance copies `users.trust_level` into the claim
/// at `src/routes/auth.rs:145-162`; the claim is what routes authorize on.
///
/// Gate an administrator-tier route on the `is_admin` flag.
///
/// **Why a flag and not a trust level.** `users.trust_level` is
/// `CHECK (trust_level BETWEEN 0 AND 6)` (migrations/013_trust_levels.sql:25) and
/// JWT issuance mints the claim from that column (`src/routes/auth.rs`), so the
/// highest claim a real login can hold is 6. A trust threshold of 10 - which is
/// what the tests asserted, and what this function briefly required - is
/// therefore unreachable in production: only hand-minted test tokens pass. The
/// owner chose a separate capability instead. See `docs/specs/admin-flag.md` and
/// `docs/specs/admin-tier-unreachable.md`.
///
/// The claim is baked in at token issue, so a demoted administrator's existing
/// token keeps working until it expires (30 days). `POST /api/auth/refresh`
/// re-reads the row, so a refresh is the way to cut that short. That is
/// pre-existing token behaviour, not something this change introduces.
///
/// This is *not* `users.role`, which is a separate legacy column holding a
/// second trust ladder (0/1/5/10) that no admin route consults. It is written
/// only by subsystem approval in `src/routes/subsystems.rs:526` and read only as
/// profile data for display.
pub fn require_admin_tier(
    auth: &crate::routes::auth::AuthUser,
    _state: &crate::server::AppState,
) -> Result<i32, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("login required".to_string()))?;
    if !auth.is_admin {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }
    Ok(uid)
}
