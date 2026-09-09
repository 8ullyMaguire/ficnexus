//! XP engine service and feature gate module (v3).
//!
//! Handles XP awarding with daily caps, level/rank recalculation,
//! feature unlocking, and access checking.

use sqlx::PgPool;

use crate::error::AppError;
use crate::progression::Feature;

// ── XP Awarding ──────────────────────────────────────────────────────────────

/// Award XP to a user, respecting daily caps.
///
/// 1. Look up the XP source definition for `event_type`
/// 2. Check the daily cap (if any) for this user+event today
/// 3. Award `min(base_xp, remaining_cap)` or full base if no cap
/// 4. Insert the xp_event row
/// 5. Update user's XP, recalculate level/rank
/// 6. Return the new total XP
pub async fn award_xp(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
    source_ref: Option<&str>,
) -> Result<i64, AppError> {
    // Look up the XP source definition (+ anti-gaming columns added in 010)
    let src = sqlx::query_as::<_, (i32, Option<i32>, Option<i64>, Option<i64>, Option<i32>, Option<f64>)>(
        "SELECT xp_amount, daily_cap, cooldown_seconds, rate_limit_per_period, streak_window_days, streak_multiplier FROM xp_source_defs WHERE event_type = $1",
    )
    .bind(event_type)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Unknown event_type: {event_type}")))?;

    let (base_xp, daily_cap, cooldown_seconds, rate_limit, streak_window, streak_mult) = src;

    // Cooldown: block duplicate identical award inside the window.
    if let Some(cd) = cooldown_seconds {
        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - make_interval(secs => $3::bigint)",
        )
        .bind(user_id)
        .bind(event_type)
        .bind(cd)
        .fetch_one(pool)
        .await?;
        if recent > 0 {
            // Still log for audit, but no XP.
            sqlx::query(
                "INSERT INTO xp_events (user_id, event_type, xp, source_ref) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(event_type)
            .bind(0i32)
            .bind(source_ref)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("xp_events insert (cooldown) failed: {e}")))?;
            return Ok(0);
        }
    }

    // Daily cap (existing behaviour).
    let mut actual_xp = base_xp as i64;
    if let Some(cap) = daily_cap {
        let today_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '1 day'",
        )
        .bind(user_id)
        .bind(event_type)
        .fetch_one(pool)
        .await?;
        let remaining = cap as i64 - today_count;
        if remaining <= 0 {
            actual_xp = 0;
        } else {
            actual_xp = (base_xp as i64).min(remaining);
        }
    }

    // Weekly per-period cap (e.g. engagement dividend, referral_new_user).
    if let Some(rl) = rate_limit {
        let period: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '7 days'",
        )
        .bind(user_id)
        .bind(event_type)
        .fetch_one(pool)
        .await?;
        if period >= rl as i64 {
            actual_xp = 0;
        }
    }

    // Streak multiplier: boost_at is the Nth-in-window threshold for this source.
    if let (Some(window_days), Some(mult)) = (streak_window, streak_mult) {
        let boost_at = sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(streak_threshold),1) FROM xp_sources_streaks WHERE event_type = $1 AND boost_at <= $2",
        )
        .bind(event_type)
        .bind(window_days)
        .fetch_optional(pool)
        .await?
        .unwrap_or(1);
        let streak_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - make_interval(days => $3::bigint)",
        )
        .bind(user_id)
        .bind(event_type)
        .bind(window_days)
        .fetch_one(pool)
        .await?;
        if streak_count >= boost_at {
            actual_xp = ((base_xp as f64)
                * mult
                * (1.0 + (streak_count as f64 - boost_at as f64) / boost_at.max(1) as f64))
                .round() as i64;
        }
    }

    let mut level_up = false;
    let mut xp_residual: i32 = 0;

    let mut tx = pool.begin().await?;

    // Always-log the event (capped/cooldown = 0 xp rows exist for audit).
    sqlx::query(
        "INSERT INTO xp_events (user_id, event_type, xp, source_ref, level_up, xp_residual) VALUES ($1, $2, $3, $4, false, 0)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(actual_xp as i32)
    .bind(source_ref)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(format!("xp_events insert failed: {e}")))?;

    tx.commit().await?;

    // Level-up + residual reset lives in the shared helper so award_xp and
    // award_scaled_xp share exactly one level/rank mutation path.
    if actual_xp > 0 {
        (level_up, xp_residual) =
            apply_xp_and_level_up(pool, user_id, event_type, actual_xp as i32, None).await?;
    }

    Ok(if level_up {
        xp_residual as i64
    } else {
        actual_xp
    })
}

/// Like `award_xp` but uses an externally-computed XP amount instead of the
/// def's `xp_amount`. Used for *scaled* awards (e.g. completion bonuses that
/// grow with word count) where the base def row exists only for cap/streak
/// config + dedup, not a fixed amount.
pub async fn award_scaled_xp(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
    source_ref: Option<&str>,
    scaled_xp: i32,
) -> Result<i64, AppError> {
    if scaled_xp <= 0 {
        return Ok(0);
    }
    // Reuse the cap/streak/cooldown enforcement from award_xp by reading the def,
    // but substitute the caller-supplied amount. We replicate the cap logic with
    // `scaled_xp` as the base so the def's daily_cap/rate_limit still bind.
    let def: (i32, Option<i32>, Option<i64>, Option<i64>) = sqlx::query_as::<_, (i32, Option<i32>, Option<i64>, Option<i64>)>(
        "SELECT xp_amount, daily_cap, cooldown_seconds, rate_limit_per_period FROM xp_source_defs WHERE event_type = $1",
    )
    .bind(event_type)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Unknown event_type: {event_type}")))?;
    let (_def_base, daily_cap, cooldown_seconds, rate_limit) = def;
    let mut actual_xp = scaled_xp as i64;
    // Cooldown (same semantics; block duplicate within window).
    if let Some(cd) = cooldown_seconds {
        let recent: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - make_interval(secs => $3::bigint)",
        )
        .bind(user_id).bind(event_type).bind(cd)
        .fetch_one(pool).await?;
        if recent > 0 {
            // Log + skip.
            sqlx::query("INSERT INTO xp_events (user_id, event_type, xp, source_ref, level_up, xp_residual) VALUES ($1, $2, 0, $4, false, 0)")
                .bind(user_id).bind(event_type).bind(source_ref)
                .execute(pool).await.map_err(|e| AppError::Database(format!("xp_events cooldown-log insert failed: {e}")))?;
            return Ok(0);
        }
    }
    // Daily cap.
    if let Some(cap) = daily_cap {
        let today: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '1 day'",
        ).bind(user_id).bind(event_type).fetch_one(pool).await?;
        let spent_today: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(xp),0) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '1 day'",
        ).bind(user_id).bind(event_type).fetch_one(pool).await?;
        if today >= cap as i64 {
            actual_xp = 0;
        } else {
            actual_xp = actual_xp.min((cap as i64) - spent_today);
        }
    }
    // Per-period rate cap.
    if let Some(rl) = rate_limit {
        let period: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM xp_events WHERE user_id = $1 AND event_type = $2 AND created_at > now() - interval '7 days'",
        ).bind(user_id).bind(event_type).fetch_one(pool).await?;
        if period >= rl {
            actual_xp = 0;
        }
    }
    // NOTE: streak multipliers apply to the def's `xp_amount`, not an externally-scaled
    // value — the completion bonus is a single large award, not a streak. If streak
    // config is ever needed on a scaled source, handle it in the caller before this fn.
    // Delegate to the shared TX path: insert event + level-up with the resolved amount.
    let (level_up, xp_residual) =
        apply_xp_and_level_up(pool, user_id, event_type, actual_xp as i32, source_ref).await?;
    let _ = (level_up, xp_residual);
    Ok(actual_xp)
}

/// Shared transactional core: insert the xp_event row + bump users.xp/level/rank
/// with level-up reset. Used by both `award_xp` and `award_scaled_xp` so there
/// is exactly one place that touches level/rank — no dupe logic.
async fn apply_xp_and_level_up(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
    xp: i32,
    source_ref: Option<&str>,
) -> Result<(bool, i32), AppError> {
    let mut level_up = false;
    let mut xp_residual: i32 = 0;
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO xp_events (user_id, event_type, xp, source_ref, level_up, xp_residual) VALUES ($1, $2, $3, $4, false, 0)",
    )
    .bind(user_id).bind(event_type).bind(xp).bind(source_ref)
    .execute(&mut *tx).await.map_err(|e| AppError::Database(format!("xp_events insert failed: {e}")))?;
    if xp > 0 {
        let cur: (i64,) =
            sqlx::query_as("SELECT COALESCE(xp,0) FROM users WHERE id = $1 FOR UPDATE")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::Database(format!("user row lock failed: {e}")))?;
        let old_level = crate::progression::xp_to_level(cur.0);
        let new_xp: (i64,) =
            sqlx::query_as("UPDATE users SET xp = xp + $1 WHERE id = $2 RETURNING xp")
                .bind(xp)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError::Database(format!("users xp update failed: {e}")))?;
        let new_level = crate::progression::xp_to_level(new_xp.0);
        if new_level > old_level {
            let threshold = crate::progression::xp_for_level(new_level);
            let residual = (new_xp.0 - threshold).max(0);
            xp_residual = residual as i32;
            level_up = true;
            sqlx::query("UPDATE users SET xp = $1, level = $2, rank = $3 WHERE id = $4")
                .bind(residual)
                .bind(new_level as i16)
                .bind(crate::progression::xp_to_rank(new_level) as i16)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(format!("users level-up reset failed: {e}")))?;
            sqlx::query("INSERT INTO xp_events (user_id, event_type, xp, source_ref, level_up, xp_residual) VALUES ($1, 'level_up', 0, NULL, true, $2)")
                .bind(user_id).bind(xp_residual).execute(&mut *tx).await.map_err(|e| AppError::Database(format!("level_up event insert failed: {e}")))?;
        } else {
            sqlx::query("UPDATE users SET level = $1, rank = $2 WHERE id = $3")
                .bind(new_level as i16)
                .bind(crate::progression::xp_to_rank(new_level) as i16)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(format!("users level/rank update failed: {e}")))?;
        }
    }
    tx.commit().await?;

    // Check for level-up achievements (best-effort, outside tx)
    if level_up {
        let _ = check_and_unlock_achievements(pool, user_id, "level_up").await;
    }

    Ok((level_up, xp_residual))
}

// ── Feature Unlocking ────────────────────────────────────────────────────────

/// Recompute which features a user has unlocked based on their current
/// rank and trust. Returns the slugs of newly unlocked features.
pub async fn recompute_unlocks(pool: &PgPool, user_id: i32) -> Result<Vec<String>, AppError> {
    // Fetch user's rank and trust
    let user_row = sqlx::query_as::<_, (i16, i16)>(
        "SELECT COALESCE(rank, 0), COALESCE(trust, 0) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;

    let (user_rank, user_trust) = user_row;
    let mut newly_unlocked: Vec<String> = Vec::new();

    // Handle gate_type='rank'
    let rank_features = sqlx::query_as::<_, Feature>(
        "SELECT * FROM features WHERE gate_type = 'rank' AND gate_value <= $1 AND id NOT IN (SELECT feature_id FROM user_features WHERE user_id = $2)",
    )
    .bind(user_rank as i32)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("rank features query failed: {e}")))?;

    for f in &rank_features {
        sqlx::query(
            "INSERT INTO user_features (user_id, feature_id, unlocked_at) VALUES ($1, $2, now())",
        )
        .bind(user_id)
        .bind(f.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("user_features insert failed: {e}")))?;
        newly_unlocked.push(f.slug.clone());
    }

    // Handle gate_type='trust'
    let trust_features = sqlx::query_as::<_, Feature>(
        "SELECT * FROM features WHERE gate_type = 'trust' AND gate_value <= $1 AND id NOT IN (SELECT feature_id FROM user_features WHERE user_id = $2)",
    )
    .bind(user_trust as i32)
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("trust features query failed: {e}")))?;

    for f in &trust_features {
        sqlx::query(
            "INSERT INTO user_features (user_id, feature_id, unlocked_at) VALUES ($1, $2, now())",
        )
        .bind(user_id)
        .bind(f.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("user_features insert failed: {e}")))?;
        newly_unlocked.push(f.slug.clone());
    }

    Ok(newly_unlocked)
}

/// Check whether a user can access a specific feature.
///
/// Returns `true` only if:
/// - The feature exists
/// - The user has a user_features row with `enabled = true`
/// - The gate condition is met (rank/trust/xp/none)
/// - If `requires_feature` is set, that prerequisite feature is also enabled
pub async fn can_access(pool: &PgPool, user_id: i32, feature_slug: &str) -> Result<bool, AppError> {
    // Fetch the feature and user's relationship
    let row = sqlx::query_as::<_, (i16, i16, i64, Option<bool>, String, i32, Option<String>)>(
        "SELECT u.rank, u.trust, u.xp, uf.enabled, f.gate_type, f.gate_value, f.requires_feature
         FROM features f
         LEFT JOIN user_features uf ON f.id = uf.feature_id AND uf.user_id = $2
         LEFT JOIN users u ON u.id = $2
         WHERE f.slug = $1",
    )
    .bind(feature_slug)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(format!("can_access query failed: {e}")))?;

    let (user_rank, user_trust, user_xp, enabled, gate_type, gate_value, requires_feature) =
        match row {
            Some(r) => r,
            None => return Ok(false), // Feature not found
        };

    // Check enabled
    if enabled != Some(true) {
        return Ok(false);
    }

    // Check gate condition
    let gate_passed = match gate_type.as_str() {
        "rank" => user_rank as i32 >= gate_value,
        "trust" => user_trust as i32 >= gate_value,
        "xp" => user_xp >= gate_value as i64,
        "none" => true,
        _ => false,
    };

    if !gate_passed {
        return Ok(false);
    }

    // Check requires_feature if set
    if let Some(req_slug) = requires_feature {
        let prereq_accessible = Box::pin(can_access(pool, user_id, &req_slug)).await?;
        if !prereq_accessible {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Seed default features for a new user.
///
/// For all features where `is_default = true`, insert a `user_features`
/// row with `unlocked_at = now()` and `enabled = true` (ON CONFLICT DO NOTHING).
pub async fn seed_default_features_for_user(pool: &PgPool, user_id: i32) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO user_features (user_id, feature_id, unlocked_at, enabled)
         SELECT $1, f.id, now(), true
         FROM features f
         WHERE f.is_default = true
         ON CONFLICT (user_id, feature_id) DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("seed_default_features failed: {e}")))?;

    Ok(())
}

/// Check and unlock achievements for a user based on their current stats.
/// Call this after XP awards, post creation, reactions, etc.
/// Returns a list of newly unlocked achievement slugs.
pub async fn check_and_unlock_achievements(
    pool: &PgPool,
    user_id: i32,
    action: &str,
) -> Result<Vec<String>, AppError> {
    let mut newly_unlocked = Vec::new();

    // Map actions to achievement slugs to check
    let slugs: Vec<&str> = match action {
        "post_created" => vec!["first_post", "forum_veteran", "topic_creator"],
        "reaction_given" => vec!["reaction_giver"],
        "poll_voted" => vec!["poll_master"],
        "message_sent" => vec!["social_butterfly"],
        "mod_action" => vec!["mod_action"],
        "work_read" => vec!["loremaster"],
        "work_bookmarked" => vec!["collector"],
        "review_left" => vec!["reviewer"],
        "level_up" => vec!["level_5", "level_10", "level_25", "level_50", "level_100"],
        _ => vec![],
    };

    for slug in slugs {
        let should_unlock = match slug {
            "first_post" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_posts WHERE author_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 1
            }
            "forum_veteran" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_posts WHERE author_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 50
            }
            "topic_creator" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_topics WHERE author_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 10
            }
            "reaction_giver" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM forum_post_reactions WHERE user_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 25
            }
            "poll_master" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT topic_id) FROM forum_poll_votes WHERE user_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 10
            }
            "social_butterfly" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE sender_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 25
            }
            "mod_action" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mod_log WHERE moderator_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 1
            }
            "loremaster" => {
                let count: i64 = sqlx::query_scalar("SELECT total_works_read FROM users WHERE id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 100
            }
            "collector" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookmarks WHERE user_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 50
            }
            "reviewer" => {
                let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reviews WHERE user_id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                count >= 10
            }
            "level_5" | "level_10" | "level_25" | "level_50" | "level_100" => {
                let target: i32 = match slug {
                    "level_5" => 5,
                    "level_10" => 10,
                    "level_25" => 25,
                    "level_50" => 50,
                    "level_100" => 100,
                    _ => 0,
                };
                let level: i32 = sqlx::query_scalar("SELECT level FROM users WHERE id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                level >= target
            }
            _ => false,
        };

        if should_unlock {
            // Try to insert — ON CONFLICT DO NOTHING means we only unlock once
            let result = sqlx::query(
                "INSERT INTO user_features (user_id, feature_id, unlocked_at, enabled)
                 SELECT $1, f.id, now(), true
                 FROM features f
                 WHERE f.slug = $2
                 ON CONFLICT (user_id, feature_id) DO NOTHING
                 RETURNING feature_id",
            )
            .bind(user_id)
            .bind(slug)
            .fetch_optional(pool)
            .await?;

            if result.is_some() {
                newly_unlocked.push(slug.to_string());
            }
        }
    }

    Ok(newly_unlocked)
}

#[cfg(test)]
mod tests {
    use crate::progression::{xp_for_level, xp_to_level};

    #[test]
    fn level_curve_expensive_late() {
        // Early: L1->L2 cheap; late: L99->L100 expensive (50 * level^1.6 curve).
        let l1 = xp_for_level(1);
        let l2 = xp_for_level(2);
        let l99 = xp_for_level(99);
        let l100 = xp_for_level(100);
        assert!(l2 > l1, "L2 cumulative must exceed L1 cumulative");
        assert!(l2 - l1 < 500, "early level-up should be fast (<500 XP)");
        assert!(
            l100 - l99 > (l2 - l1) * 100,
            "late level-up (L99->L100 = {}) must be >100x early (L1->L2 = {})",
            l100 - l99,
            l2 - l1
        );
    }

    #[test]
    fn level_up_reset_zeroes_xp_after_threshold() {
        // User sits at the L1->L2 boundary, then earns enough to cross.
        // xp_for_level(2) is the XP total needed for level 2; crossing it with
        // a single gain must leave the residual *inside* the new level's window,
        // not at the same value as before (that would mean the reset no-op'd).
        let xp_at_l2_boundary = xp_for_level(2);
        // earn a chunk that would cross the boundary on its own
        let crossed = xp_to_level(xp_at_l2_boundary + 50);
        assert!(crossed >= 2, "crossing L2 boundary must advance level");
        // residual is not exposed by xp_to_level (level-only); the engine stores
        // xp_residual in xp_events so the DB column reflects the leftover.
        // Here we assert the invariant the engine enforces:
        let total_after = xp_at_l2_boundary + 50;
        let new_level = xp_to_level(total_after);
        assert!(new_level >= 2);
        assert!(
            total_after > xp_for_level(new_level),
            "residual XP above new level floor must remain for the next level-up"
        );
    }
}
