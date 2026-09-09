//! Achievement, feature-gating, and notification services.
//!
//! Handles achievement unlocking, feature access control, and user notifications.
//! Replaces the old XP-based progression system with reputation + trust-based gating.

use sqlx::PgPool;

use crate::error::AppError;
use crate::db::models::Feature;

/// Check and unlock achievements for a user based on their current stats.
/// Call this after reputation changes, post creation, reactions, etc.
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
        "reputation_milestone" => vec!["rep_100", "rep_500", "rep_1000", "rep_5000"],
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
            "rep_100" | "rep_500" | "rep_1000" | "rep_5000" => {
                let target: i64 = match slug {
                    "rep_100" => 100,
                    "rep_500" => 500,
                    "rep_1000" => 1000,
                    "rep_5000" => 5000,
                    _ => 0,
                };
                let rep: i64 = sqlx::query_scalar("SELECT reputation FROM users WHERE id = $1")
                    .bind(user_id).fetch_one(pool).await.unwrap_or(0);
                rep >= target
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
                // Fire notification for the newly unlocked achievement
                let notif_title = format!("Achievement Unlocked: {}", slug);
                let notif_body = format!("You've earned the '{}' achievement!", slug);
                let _ = crate::db::queries::social::create_notification(
                    pool,
                    user_id,
                    "achievement",
                    &notif_title,
                    Some(&notif_body),
                    Some("/achievements"),
                    Some("achievement"),
                    Some(slug),
                )
                .await;
            }
        }
    }

    Ok(newly_unlocked)
}

/// Recompute which features a user has unlocked based on their current
/// trust level. Returns the slugs of newly unlocked features.
pub async fn recompute_unlocks(pool: &PgPool, user_id: i32) -> Result<Vec<String>, AppError> {
    // Fetch user's trust level
    let user_row = sqlx::query_as::<_, (i16,)>(
        "SELECT COALESCE(trust_level, 0) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;

    let user_trust = user_row.0;
    let mut newly_unlocked: Vec<String> = Vec::new();

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
/// - The gate condition is met (trust/none)
/// - If `requires_feature` is set, that prerequisite feature is also enabled
pub async fn can_access(pool: &PgPool, user_id: i32, feature_slug: &str) -> Result<bool, AppError> {
    // Fetch the feature and user's relationship
    let row = sqlx::query_as::<_, (i16, Option<bool>, String, i32, Option<String>)>(
        "SELECT u.trust_level, uf.enabled, f.gate_type, f.gate_value, f.requires_feature
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

    let (user_trust, enabled, gate_type, gate_value, requires_feature) =
        match row {
            Some(r) => r,
            None => return Ok(false), // Feature not found
        };

    // Check enabled
    if enabled != Some(true) {
        return Ok(false);
    }

    // Check gate condition (only 'trust' and 'none' gate types supported)
    let gate_passed = match gate_type.as_str() {
        "trust" => user_trust as i32 >= gate_value,
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

#[cfg(test)]
mod tests {
    // Tests are in the integration test suite since they require a database.
    // Unit tests for pure logic would go here if any pure functions existed.
}
