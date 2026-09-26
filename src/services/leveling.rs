//! F7 — site-wide forum leveling.
//!
//! Spec: `docs/specs/f7-leveling.md`.
//!
//! ## Why this module exists rather than reusing `update_reputation_and_promote`
//!
//! `users` has both an `exp` column and a `reputation` column, and they are
//! not the same thing. F7 leveling is driven by `exp`; the reputation system
//! is a separate axis with its own thresholds (auto-promotion to curator at
//! reputation >= 100). `update_reputation_and_promote` moves `reputation`,
//! so calling it for a forum exp grant would move the wrong column and the F7
//! tests would still read 0.
//!
//! ## The curve
//!
//! All values come from `Config`, never from literals, so they are tunable per
//! deployment without a rebuild (`src/config.rs:1199-1217`):
//!
//! | field | env | default |
//! |---|---|---|
//! | `forum_exp_per_level` | `FORUM_EXP_PER_LEVEL` | 100 |
//! | `forum_exp_topic_create` | `FORUM_EXP_TOPIC_CREATE` | 2 |
//! | `forum_exp_post_create` | `FORUM_EXP_POST_CREATE` | 2 |
//! | `forum_exp_mod_received` | `FORUM_EXP_MOD_RECEIVED` | 1 |
//! | `forum_exp_mod_daily_cap` | `FORUM_EXP_MOD_DAILY_CAP` | 3 |

use crate::db::queries;
use crate::error::AppResult;
use sqlx::PgPool;

/// Award forum exp and promote through any level thresholds it crosses.
///
/// Returns the number of levels gained, which is 0 for the common case and >0
/// only when a grant crosses a boundary. Callers use it to decide whether to
/// emit a level-up notification, which is why the count is returned rather than
/// discarded.
///
/// Best-effort by contract: every caller in this codebase invokes it as
/// `let _ = ...`, because an exp grant must never fail the post that earned
/// it. That convention is why the `delta`-versus-`points` bug in
/// `update_reputation_and_promote` went unnoticed for so long - see that
/// function's comment.
pub async fn award_exp(pool: &PgPool, user_id: i32, delta: i32, reason: &str) -> AppResult<i32> {
    if delta <= 0 {
        return Ok(0);
    }

    // Read the level before, so a multi-level jump is detectable afterwards.
    // `users.level` is smallint, so it decodes as i16. Reading it as i32 fails
    // with a type mismatch, and because every caller uses `if let Ok(..)` that
    // failure was silent - the grant simply never happened.
    let before: Option<i16> = sqlx::query_scalar("SELECT level FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    let Some(level_before) = before else {
        // Unknown user: nothing to grant. Not an error - the caller has
        // already validated the user exists by this point in most paths.
        return Ok(0);
    };

    sqlx::query("UPDATE users SET exp = exp + $1 WHERE id = $2")
        .bind(delta)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Audit trail in `exp_events` - the table migrations/001_initial.sql already
    // defines for exactly this (user_id, amount, event_type) and which nothing
    // in src/ ever wrote to. `amount` is the exp delta. This is also what the
    // mod_received daily cap counts.
    sqlx::query(
        "INSERT INTO exp_events (user_id, amount, event_type)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(delta)
    .bind(reason)
    .execute(pool)
    .await?;

    let (exp_now, level_now): (i64, i16) =
        sqlx::query_as("SELECT exp, level FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    // `per_level` is read from Config by the caller and passed in, because the
    // curve is deployment configuration. A value of 0 would divide by zero, so
    // treat it as "no promotion" rather than panicking on a misconfiguration.
    let per_level = crate::config::Config::from_env().forum_exp_per_level.max(1);
    let level_from_exp = (exp_now / i64::from(per_level)) as i16;

    if level_from_exp > level_now {
        sqlx::query("UPDATE users SET level = $1 WHERE id = $2")
            .bind(level_from_exp)
            .bind(user_id)
            .execute(pool)
            .await?;
        return Ok(i32::from(level_from_exp) - i32::from(level_now));
    }

    Ok(0)
}

/// Emit a level-up notification, once per level crossed.
///
/// Separate from [`award_exp`] so a caller that only wants the exp does not
/// have to care about notifications, and so the notification shape can change
/// without touching the grant.
pub async fn notify_level_up(pool: &PgPool, user_id: i32, new_level: i16) -> AppResult<()> {
    let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let _ = queries::create_notification(
        pool,
        user_id,
        "level_up",
        &format!("Level {new_level} reached"),
        Some(&format!("You reached level {new_level}, {username}.")),
        Some(&format!("/u/{username}")),
        Some("user"),
        Some(&user_id.to_string()),
    )
    .await;
    Ok(())
}

/// Award `mod_received` exp, capped per day per author.
///
/// The cap is `forum_exp_mod_daily_cap` (default 3). It counts the
/// `mod_received` grants already recorded for this user since midnight rather
/// than the moderation events themselves, so a moderator acting on the same
/// post twice does not double-count toward the cap.
pub async fn award_mod_received_exp(pool: &PgPool, author_id: i32, delta: i32) -> AppResult<()> {
    let cfg = crate::config::Config::from_env();
    let cap = cfg.forum_exp_mod_daily_cap;
    if cap <= 0 || delta <= 0 {
        return Ok(());
    }

    let granted_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM exp_events
         WHERE user_id = $1 AND event_type = 'mod_received'
           AND created_at >= date_trunc('day', NOW())",
    )
    .bind(author_id)
    .fetch_one(pool)
    .await?;

    if granted_today >= i64::from(cap) {
        return Ok(());
    }
    let _ = award_exp(pool, author_id, delta, "mod_received").await;
    Ok(())
}
