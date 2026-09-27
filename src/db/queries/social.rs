use crate::db::models::*;
use crate::error::AppResult;
use chrono::Datelike;
use sqlx::{PgPool, Row};
use super::ReadingHistoryEntry;
use super::update_reputation_and_promote;

// ── Follow exclusions ─────────────────────────────────────────────────

/// Create a follow exclusion on a follow owned by `user_id`. Returns the new
/// exclusion's id, or `None` when the follow doesn't exist / isn't owned by
/// the caller (the INSERT realises zero rows in that case). The caller is
/// responsible for validating that exactly one target is supplied before the
/// DB CHECK constraint fires.
#[allow(clippy::too_many_arguments)]
pub async fn create_follow_exclusion(
    pool: &PgPool,
    user_id: i32,
    follow_id: i64,
    exclude_type: &str,
    work_id: Option<i32>,
    series_id: Option<i32>,
    fandom: Option<&str>,
) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"INSERT INTO follow_exclusions
             (follow_id, exclude_type, exclude_work_id, exclude_series_id, exclude_fandom)
           SELECT id, $3, $4, $5, $6 FROM follows WHERE id = $1 AND follower_id = $2
           RETURNING id"#,
    )
    .bind(follow_id)
    .bind(user_id)
    .bind(exclude_type)
    .bind(work_id)
    .bind(series_id)
    .bind(fandom)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// List a follow's exclusions, but only when the follow is owned by `user_id`.
pub async fn list_follow_exclusions(
    pool: &PgPool,
    user_id: i32,
    follow_id: i64,
) -> AppResult<Vec<FollowExclusion>> {
    let rows = sqlx::query_as::<_, FollowExclusion>(
        r#"SELECT fe.id, fe.follow_id, fe.exclude_type,
                  fe.exclude_work_id, fe.exclude_series_id, fe.exclude_fandom, fe.created_at
           FROM follow_exclusions fe
           JOIN follows f ON f.id = fe.follow_id
           WHERE fe.follow_id = $1 AND f.follower_id = $2
           ORDER BY fe.created_at ASC, fe.id ASC"#,
    )
    .bind(follow_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Delete a follow exclusion. Only succeeds when the exclusion lives on a
/// follow owned by `user_id`; returns true when a row was removed.
pub async fn delete_follow_exclusion(
    pool: &PgPool,
    user_id: i32,
    follow_id: i64,
    exclusion_id: i64,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"DELETE FROM follow_exclusions fe
           USING follows f
           WHERE fe.id = $1 AND fe.follow_id = $2
             AND f.id = $2 AND f.follower_id = $3"#,
    )
    .bind(exclusion_id)
    .bind(follow_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// ═══════════════════════════════════════════════════════════════════
// Notification queries
// ═══════════════════════════════════════════════════════════════════

/// Create a notification
pub async fn create_notification(
    pool: &PgPool,
    user_id: i32,
    notification_type: &str,
    title: &str,
    body: Option<&str>,
    link: Option<&str>,
    reference_type: Option<&str>,
    reference_id: Option<&str>,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO notifications (user_id, notification_type, title, body, link, reference_type, reference_id)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id"#,
    )
    .bind(user_id)
    .bind(notification_type)
    .bind(title)
    .bind(body)
    .bind(link)
    .bind(reference_type)
    .bind(reference_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Get unread notification count
pub async fn get_unread_notification_count(pool: &PgPool, user_id: i32) -> AppResult<i64> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// List notifications for a user
pub async fn list_notifications(
    pool: &PgPool,
    user_id: i32,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Notification>> {
    let rows = sqlx::query_as::<_, Notification>(
        r#"SELECT id, user_id, notification_type, title, body, link,
                  reference_type, reference_id, is_read, created_at
           FROM notifications
           WHERE user_id = $1
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Mark notification as read
pub async fn mark_notification_read(
    pool: &PgPool,
    user_id: i32,
    notification_id: i64,
) -> AppResult<bool> {
    let result =
        sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2")
            .bind(notification_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Mark all notifications as read for a user
pub async fn mark_all_notifications_read(pool: &PgPool, user_id: i32) -> AppResult<()> {
    sqlx::query("UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get or create notification preferences
pub async fn get_notification_preferences(
    pool: &PgPool,
    user_id: i32,
) -> AppResult<NotificationPreference> {
    let prefs = sqlx::query_as::<_, NotificationPreference>(
        r#"SELECT user_id, comment_reply, follow_update, work_update, badge_earned,
                  curator_promotion, recommendation, email_digest, updated_at,
                  comments_on_work, replies_to_comments, kudos_on_work,
                  bookmarks_on_work, follows, mentions
           FROM notification_preferences WHERE user_id = $1"#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    match prefs {
        Some(p) => Ok(p),
        None => {
            // Create default preferences
            sqlx::query(
                "INSERT INTO notification_preferences
                    (user_id, comment_reply, follow_update, work_update, badge_earned,
                     curator_promotion, recommendation, email_digest,
                     comments_on_work, replies_to_comments, kudos_on_work,
                     bookmarks_on_work, follows, mentions)
                 VALUES ($1, TRUE, TRUE, TRUE, TRUE, TRUE, FALSE, 'never',
                         TRUE, TRUE, TRUE, TRUE, TRUE, TRUE)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .execute(pool)
            .await?;

            Ok(NotificationPreference {
                user_id,
                comment_reply: true,
                follow_update: true,
                work_update: true,
                badge_earned: true,
                curator_promotion: true,
                recommendation: false,
                email_digest: "never".into(),
                updated_at: chrono::Utc::now(),
                comments_on_work: true,
                replies_to_comments: true,
                kudos_on_work: true,
                bookmarks_on_work: true,
                follows: true,
                mentions: true,
            })
        }
    }
}

/// Update notification preferences
pub async fn update_notification_preferences(
    pool: &PgPool,
    user_id: i32,
    prefs: &NotificationPreference,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO notification_preferences (
                  user_id, comment_reply, follow_update, work_update,
                  badge_earned, curator_promotion, recommendation, email_digest,
                  comments_on_work, replies_to_comments, kudos_on_work,
                  bookmarks_on_work, follows, mentions
               )
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
           ON CONFLICT (user_id) DO UPDATE SET
               comment_reply = $2, follow_update = $3, work_update = $4,
               badge_earned = $5, curator_promotion = $6, recommendation = $7,
               email_digest = $8,
               comments_on_work = $9, replies_to_comments = $10, kudos_on_work = $11,
               bookmarks_on_work = $12, follows = $13, mentions = $14,
               updated_at = NOW()"#,
    )
    .bind(user_id)
    .bind(prefs.comment_reply)
    .bind(prefs.follow_update)
    .bind(prefs.work_update)
    .bind(prefs.badge_earned)
    .bind(prefs.curator_promotion)
    .bind(prefs.recommendation)
    .bind(&prefs.email_digest)
    .bind(prefs.comments_on_work)
    .bind(prefs.replies_to_comments)
    .bind(prefs.kudos_on_work)
    .bind(prefs.bookmarks_on_work)
    .bind(prefs.follows)
    .bind(prefs.mentions)
    .execute(pool)
    .await?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Badge queries
// ═══════════════════════════════════════════════════════════════════

/// Get all badge definitions
pub async fn get_badge_definitions(pool: &PgPool) -> AppResult<Vec<BadgeDefinition>> {
    let rows = sqlx::query_as::<_, BadgeDefinition>(
        "SELECT badge_type, name, description, icon, category, threshold, event_type, created_at FROM badge_definitions ORDER BY category, threshold",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a user's badges
pub async fn get_user_badges(pool: &PgPool, user_id: i32) -> AppResult<Vec<UserBadge>> {
    let rows = sqlx::query_as::<_, UserBadge>(
        "SELECT id, user_id, badge_type, earned_at FROM user_badges WHERE user_id = $1 ORDER BY earned_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Award a badge and create notification
pub async fn award_badge(pool: &PgPool, user_id: i32, badge_type: &str) -> AppResult<bool> {
    // Insert badge (no-op if already earned)
    let result = sqlx::query(
        "INSERT INTO user_badges (user_id, badge_type) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(badge_type)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        // Look up badge name for notification
        let badge_def: Option<(String, String, String)> = sqlx::query_as(
            "SELECT name, description, icon FROM badge_definitions WHERE badge_type = $1",
        )
        .bind(badge_type)
        .fetch_optional(pool)
        .await?;

        if let Some((name, _, _)) = badge_def {
            // Create notification
            create_notification(
                pool,
                user_id,
                "badge_earned",
                &format!("Badge earned: {}", name),
                None,
                Some("/badges"),
                Some("badge"),
                Some(badge_type),
            )
            .await?;

            // Award reputation for badge
            update_reputation_and_promote(pool, user_id, 10, &format!("badge_{}", badge_type))
                .await?;
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Check and award badges based on event count
pub async fn check_and_award_badges(
    pool: &PgPool,
    user_id: i32,
    event_type: &str,
) -> AppResult<Vec<String>> {
    let mut awarded = Vec::new();

    // Get badge definitions that track this event type
    let badges = sqlx::query_as::<_, BadgeDefinition>(
        "SELECT badge_type, name, description, icon, category, threshold, event_type, created_at
         FROM badge_definitions WHERE event_type = $1 ORDER BY threshold",
    )
    .bind(event_type)
    .fetch_all(pool)
    .await?;

    if badges.is_empty() {
        return Ok(awarded);
    }

    // Count user's events of this type
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM reputation_events WHERE user_id = $1 AND event_type = $2",
    )
    .bind(user_id)
    .bind(event_type)
    .fetch_one(pool)
    .await?;

    for badge in &badges {
        if count.0 >= badge.threshold as i64 {
            if award_badge(pool, user_id, &badge.badge_type).await? {
                awarded.push(badge.badge_type.clone());
            }
        }
    }

    Ok(awarded)
}

// ═══════════════════════════════════════════════════════════════════
// Login streak queries
// ═══════════════════════════════════════════════════════════════════

/// Record daily login and update streak
pub async fn record_login_streak(pool: &PgPool, user_id: i32) -> AppResult<i32> {
    let today = chrono::Utc::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);

    let existing = sqlx::query_as::<_, LoginStreak>(
        "SELECT user_id, current_streak, longest_streak, last_login_date, updated_at FROM login_streaks WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (current_streak, _longest_streak) = match existing {
        Some(streak) if streak.last_login_date == today => {
            // Already logged in today
            return Ok(streak.current_streak);
        }
        Some(streak) if streak.last_login_date == yesterday => {
            // Consecutive day
            let new_streak = streak.current_streak + 1;
            let new_longest = new_streak.max(streak.longest_streak);
            sqlx::query(
                "UPDATE login_streaks SET current_streak = $1, longest_streak = $2, last_login_date = $3, updated_at = NOW() WHERE user_id = $4",
            )
            .bind(new_streak)
            .bind(new_longest)
            .bind(today)
            .bind(user_id)
            .execute(pool)
            .await?;
            (new_streak, new_longest)
        }
        _ => {
            // Streak broken or first login
            sqlx::query(
                "INSERT INTO login_streaks (user_id, current_streak, longest_streak, last_login_date)
                 VALUES ($1, 1, GREATEST(1, (SELECT COALESCE(longest_streak, 0) FROM login_streaks WHERE user_id = $1)), $2)
                 ON CONFLICT (user_id) DO UPDATE SET current_streak = 1, last_login_date = $2, updated_at = NOW()",
            )
            .bind(user_id)
            .bind(today)
            .execute(pool)
            .await?;
            (1, 1)
        }
    };

    // Check streak badges
    check_and_award_badges(pool, user_id, "login_streak").await?;

    // Award streak XP
    if current_streak >= 7 && current_streak % 7 == 0 {
        update_reputation_and_promote(pool, user_id, 10, "login_streak_milestone").await?;

        // Create streak milestone notification
        let _ = create_notification(
            pool,
            user_id,
            "streak_milestone",
            &format!("{}-day Login Streak!", current_streak),
            Some(&format!(
                "You've logged in for {} consecutive days! Keep it up!",
                current_streak
            )),
            Some("/reading/streak"),
            Some("streak"),
            Some(&current_streak.to_string()),
        )
        .await;
    }

    Ok(current_streak)
}

/// Get user's login streak
pub async fn get_login_streak(pool: &PgPool, user_id: i32) -> AppResult<Option<LoginStreak>> {
    let row = sqlx::query_as::<_, LoginStreak>(
        "SELECT user_id, current_streak, longest_streak, last_login_date, updated_at FROM login_streaks WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// ═══════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════
// Reading stats queries
// ═══════════════════════════════════════════════════════════════════

/// Record reading a work
pub async fn record_work_read(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    words_read: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_stats (user_id, work_id, words_read)
           VALUES ($1, $2, $3)
           ON CONFLICT (user_id, work_id) DO UPDATE SET
               words_read = reading_stats.words_read + $3,
               read_count = reading_stats.read_count + 1,
               last_read_at = NOW()"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(words_read)
    .execute(pool)
    .await?;

    // Update user aggregate stats
    sqlx::query(
        "UPDATE users SET total_words_read = total_words_read + $1, total_works_read = total_works_read + 1, last_active_at = NOW() WHERE id = $2",
    )
    .bind(words_read)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Check reading badges
    check_and_award_badges(pool, user_id, "work_read").await?;

    Ok(())
}

/// Get reading stats for a user
pub async fn get_user_reading_stats(pool: &PgPool, user_id: i32) -> AppResult<Vec<ReadingStats>> {
    let rows = sqlx::query_as::<_, ReadingStats>(
        "SELECT id, user_id, work_id, words_read, last_read_at, read_count
         FROM reading_stats WHERE user_id = $1 ORDER BY last_read_at DESC LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get aggregate user reading stats (zeros for unknown users — never 500)
pub async fn get_user_reading_aggregate(
    pool: &PgPool,
    user_id: i32,
) -> AppResult<(i64, i32, Option<i32>)> {
    let row = sqlx::query_as::<_, (i64, i32, Option<i32>)>(
        "SELECT total_words_read, total_works_read, COALESCE((SELECT current_streak FROM login_streaks WHERE user_id = $1), 0) FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.unwrap_or((0, 0, None)))
}

/// Record a visit to a work page or reader session.
pub async fn record_read_history(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    chapter_num: Option<i32>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_history (user_id, work_id, chapter_num, visited_at)
           VALUES ($1, $2, $3, NOW())"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(chapter_num)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get paginated reading history for a user.
/// Aggregate personal reading activity from the durable reading tables.
pub async fn get_personal_reading_analytics(
    pool: &PgPool,
    user_id: i32,
) -> AppResult<serde_json::Value> {
    let row = sqlx::query(
        r#"SELECT COUNT(*)::BIGINT AS visits, COUNT(DISTINCT work_id)::BIGINT AS works,
                  COUNT(DISTINCT visited_at::date)::BIGINT AS active_days,
                  COALESCE(SUM(rs.words_read), 0)::BIGINT AS words
           FROM reading_history h LEFT JOIN reading_stats rs ON rs.user_id=h.user_id AND rs.work_id=h.work_id
           WHERE h.user_id=$1"#)
        .bind(user_id).fetch_one(pool).await?;
    let recent = sqlx::query(
        r#"SELECT visited_at::date AS day, COUNT(*)::BIGINT AS visits,
                  COUNT(DISTINCT work_id)::BIGINT AS works
           FROM reading_history WHERE user_id=$1
           GROUP BY visited_at::date ORDER BY day DESC LIMIT 30"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    let days: Vec<_> = recent
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "date": r.get::<chrono::NaiveDate, _>("day").to_string(),
                "visits": r.get::<i64, _>("visits"), "works": r.get::<i64, _>("works")
            })
        })
        .collect();
    Ok(
        serde_json::json!({"visits": row.get::<i64,_>("visits"), "works": row.get::<i64,_>("works"),
        "active_days": row.get::<i64,_>("active_days"), "words_read": row.get::<i64,_>("words"), "daily": days}),
    )
}

/// Aggregate author performance and readership from real work/reading data.
pub async fn get_author_analytics(pool: &PgPool, profile_id: i32) -> AppResult<serde_json::Value> {
    let row = sqlx::query(r#"SELECT ap.canonical_name, COUNT(DISTINCT w.id)::BIGINT AS works,
        COALESCE(SUM(rs.read_count),0)::BIGINT AS reads, COALESCE(SUM(rs.words_read),0)::BIGINT AS words,
        COUNT(DISTINCT rh.user_id)::BIGINT AS readers
        FROM author_profiles ap LEFT JOIN author_profile_links l ON l.profile_id=ap.id
        LEFT JOIN fic_info fi ON fi.author=l.source_author LEFT JOIN works w ON w.default_source_id=fi.id
        LEFT JOIN reading_stats rs ON rs.work_id=w.id LEFT JOIN reading_history rh ON rh.work_id=w.id
        WHERE ap.id=$1 GROUP BY ap.id"#).bind(profile_id).fetch_optional(pool).await?;
    let row = row.ok_or_else(|| crate::error::AppError::NotFound("Author not found".into()))?;
    Ok(
        serde_json::json!({"author": row.get::<String,_>("canonical_name"), "works": row.get::<i64,_>("works"),
        "reads": row.get::<i64,_>("reads"), "words_read": row.get::<i64,_>("words"), "unique_readers": row.get::<i64,_>("readers")}),
    )
}

pub async fn get_reading_history(
    pool: &PgPool,
    user_id: i32,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<ReadingHistoryEntry>, i64)> {
    let rows = sqlx::query_as::<_, ReadingHistoryEntry>(
        r#"SELECT rh.id, rh.work_id, COALESCE(w.default_source_id,'') AS url_id, COALESCE(w.canonical_title, fi.title) AS title, COALESCE(w.canonical_author, fi.author) AS author, rh.visited_at, rh.chapter_num FROM reading_history rh JOIN works w ON w.id=rh.work_id LEFT JOIN fic_info fi ON fi.id=w.default_source_id WHERE rh.user_id=$1 ORDER BY rh.visited_at DESC LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar("SELECT COUNT(*) FROM reading_history WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    Ok((rows, total))
}

/// Delete a specific history entry.
pub async fn delete_read_history(pool: &PgPool, user_id: i32, id: i64) -> AppResult<bool> {
    let r = sqlx::query("DELETE FROM reading_history WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}

/// Clear all reading history for a user.
pub async fn clear_read_history(pool: &PgPool, user_id: i32) -> AppResult<()> {
    sqlx::query("DELETE FROM reading_history WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Locale & translation queries
// ═══════════════════════════════════════════════════════════════════

/// Get all supported locales
pub async fn get_locales(pool: &PgPool) -> AppResult<Vec<Locale>> {
    let rows =
        sqlx::query_as::<_, Locale>("SELECT id, code, name, is_rtl FROM locales ORDER BY id")
            .fetch_all(pool)
            .await?;
    Ok(rows)
}

/// Get UI translations for a locale
pub async fn get_ui_translations(pool: &PgPool, locale_code: &str) -> AppResult<Vec<Translation>> {
    let rows = sqlx::query_as::<_, Translation>(
        "SELECT id, locale_code, namespace, key, value, updated_at FROM translations WHERE locale_code = $1",
    )
    .bind(locale_code)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get work translation
pub async fn get_work_translation(
    pool: &PgPool,
    work_id: i32,
    locale_code: &str,
) -> AppResult<Option<WorkTranslation>> {
    let row = sqlx::query_as::<_, WorkTranslation>(
        "SELECT id, work_id, locale_code, title, summary, translated_by, translated_at
         FROM work_translations WHERE work_id = $1 AND locale_code = $2 AND status = 'approved'",
    )
    .bind(work_id)
    .bind(locale_code)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Submit a work translation
pub async fn upsert_work_translation(
    pool: &PgPool,
    work_id: i32,
    locale_code: &str,
    title: Option<&str>,
    summary: Option<&str>,
    translated_by: i32,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO work_translations (work_id, locale_code, title, summary, translated_by)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (work_id, locale_code) DO UPDATE SET
               title = COALESCE($3, work_translations.title),
               summary = COALESCE($4, work_translations.summary),
               translated_by = $5, translated_at = NOW(), status = 'draft', reviewed_by = NULL, reviewed_at = NULL"#,
    )
    .bind(work_id)
    .bind(locale_code)
    .bind(title)
    .bind(summary)
    .bind(translated_by)
    .execute(pool)
    .await?;

    // Track translation in reputation
    update_reputation_and_promote(pool, translated_by, 15, "translation_submit").await?;

    // Check translation badges
    check_and_award_badges(pool, translated_by, "translation_submit").await?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Enhanced leaderboard queries
// ═══════════════════════════════════════════════════════════════════

/// Get weekly leaderboard — LIVE from users.reputation (no materialized
/// dependency), so reloading always shows the current reputation. The
/// `leaderboard_weekly` materialized table remains for historical snapshots
/// but is NOT the source of truth for the live board.
pub async fn get_weekly_leaderboard(
    pool: &PgPool,
    limit: i64,
) -> AppResult<Vec<(i32, String, i32, i16)>> {
    let rows = sqlx::query_as::<_, (i32, String, i32, i16)>(
        r#"SELECT u.id, u.username, u.reputation::INT as score,
                  ROW_NUMBER() OVER (ORDER BY u.reputation DESC)::SMALLINT as rank
           FROM users u
           WHERE u.reputation > 0
           ORDER BY u.reputation DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get monthly leaderboard — LIVE from users.reputation (see weekly).
pub async fn get_monthly_leaderboard(
    pool: &PgPool,
    limit: i64,
) -> AppResult<Vec<(i32, String, i32, i16)>> {
    let rows = sqlx::query_as::<_, (i32, String, i32, i16)>(
        r#"SELECT u.id, u.username, u.reputation::INT as score,
                  ROW_NUMBER() OVER (ORDER BY u.reputation DESC)::SMALLINT as rank
           FROM users u
           WHERE u.reputation > 0
           ORDER BY u.reputation DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Compute weekly leaderboard from reputation_events
pub async fn compute_weekly_leaderboard(pool: &PgPool) -> AppResult<()> {
    let now = chrono::Utc::now().date_naive();
    let week_start = {
        let days_from_monday = now.weekday().num_days_from_monday();
        now - chrono::Duration::days(days_from_monday as i64)
    };

    // Clear old entries for this week
    sqlx::query("DELETE FROM leaderboard_weekly WHERE week_start = $1")
        .bind(week_start)
        .execute(pool)
        .await?;

    // Compute new rankings
    sqlx::query(
        r#"INSERT INTO leaderboard_weekly (user_id, score, rank, week_start)
           SELECT user_id, SUM(points) as score,
                  ROW_NUMBER() OVER (ORDER BY SUM(points) DESC)::SMALLINT as rank,
                  $1 as week_start
           FROM reputation_events
           WHERE created_at >= $1 AND created_at < $1 + INTERVAL '7 days'
           GROUP BY user_id
           ORDER BY score DESC
           LIMIT 100"#,
    )
    .bind(week_start)
    .execute(pool)
    .await?;

    Ok(())
}

/// Compute monthly leaderboard
pub async fn compute_monthly_leaderboard(pool: &PgPool) -> AppResult<()> {
    let now = chrono::Utc::now().date_naive();
    let month_start = { chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap() };

    sqlx::query("DELETE FROM leaderboard_monthly WHERE month_start = $1")
        .bind(month_start)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"INSERT INTO leaderboard_monthly (user_id, score, rank, month_start)
           SELECT user_id, SUM(points) as score,
                  ROW_NUMBER() OVER (ORDER BY SUM(points) DESC)::SMALLINT as rank,
                  $1 as month_start
           FROM reputation_events
           WHERE created_at >= $1 AND created_at < $1 + INTERVAL '1 month'
           GROUP BY user_id
           ORDER BY score DESC
           LIMIT 100"#,
    )
    .bind(month_start)
    .execute(pool)
    .await?;

    Ok(())
}

/// Notify user of a comment reply
pub async fn notify_comment_reply(
    pool: &PgPool,
    parent_comment_user_id: i32,
    replier_username: &str,
    work_id: i32,
    comment_id: i64,
) -> AppResult<()> {
    // Check if user has reply notifications enabled
    let prefs = get_notification_preferences(pool, parent_comment_user_id).await?;
    if !prefs.comment_reply {
        return Ok(());
    }

    create_notification(
        pool,
        parent_comment_user_id,
        "comment_reply",
        &format!("{} replied to your comment", replier_username),
        None,
        Some(&format!(
            "/api/works/{}/comments#comment-{}",
            work_id, comment_id
        )),
        Some("comment"),
        Some(&comment_id.to_string()),
    )
    .await?;

    Ok(())
}

/// Notify work followers of an update
pub async fn notify_work_followers(
    pool: &PgPool,
    work_id: i32,
    work_title: &str,
) -> AppResult<i64> {
    let followers =
        sqlx::query_scalar::<_, i32>("SELECT follower_id FROM follows WHERE work_id = $1")
            .bind(work_id)
            .fetch_all(pool)
            .await?;

    for follower_id in &followers {
        // Check preferences
        let prefs = get_notification_preferences(pool, *follower_id).await.ok();
        if prefs.map(|p| p.work_update).unwrap_or(true) {
            create_notification(
                pool,
                *follower_id,
                "work_update",
                &format!("\"{}\" has been updated", work_title),
                None,
                Some(&format!("/api/works/{}", work_id)),
                Some("work"),
                Some(&work_id.to_string()),
            )
            .await
            .ok();
        }
    }

    Ok(followers.len() as i64)
}

/// Record a vote (insert or update)
pub async fn upsert_tag_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
    value: i16,
) -> AppResult<()> {
    // Try update first, then insert
    let affected = sqlx::query(
        r#"UPDATE fic_tag_votes SET value = $1, created_at = NOW()
           WHERE url_id = $2 AND tag_id = $3 AND voter_ip = $4::inet"#,
    )
    .bind(value)
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        sqlx::query(
            r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
               VALUES ($1, $2, $3::inet, $4)"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .bind(value)
        .execute(pool)
        .await?;

        // Also ensure fic_tags row exists with starting score 0 + vote
        sqlx::query(
            r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
               VALUES ($1, $2, $3::inet, 0)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Check if a user has already voted on a tag
pub async fn get_existing_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
) -> AppResult<Option<i16>> {
    let row = sqlx::query_scalar::<_, i16>(
        r#"SELECT value FROM fic_tag_votes
           WHERE url_id = $1 AND tag_id = $2 AND voter_ip = $3::inet"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Flag a tag on a fic
pub async fn insert_tag_flag(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    flagged_by_ip: &std::net::IpAddr,
    reason: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, flagged_by_ip) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(flagged_by_ip.to_string())
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// List unresolved flags for curator review
pub async fn list_unresolved_flags(
    pool: &PgPool,
) -> AppResult<Vec<(i64, String, i32, String, Option<String>)>> {
    let rows = sqlx::query_as::<_, (i64, String, i32, String, Option<String>)>(
        r#"SELECT f.id, f.url_id, f.tag_id, t.name, f.reason
           FROM tag_flags f
           JOIN tags t ON t.id = f.tag_id
           WHERE f.resolved = FALSE
           ORDER BY f.created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Resolve a flag
pub async fn resolve_flag(pool: &PgPool, flag_id: i64) -> AppResult<()> {
    sqlx::query("UPDATE tag_flags SET resolved = TRUE WHERE id = $1")
        .bind(flag_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Check rate limit by counting actions in the last hour from Redis
pub async fn check_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
    max_per_hour: u32,
) -> AppResult<()> {
    use redis::AsyncCommands;
    let count: Option<u32> = redis.get(key).await.unwrap_or(None);
    if count.unwrap_or(0) >= max_per_hour {
        return Err(crate::error::AppError::RateLimited(3600));
    }
    // Increment and set TTL to 1 hour
    let _: () = redis.incr(key, 1).await.unwrap_or_default();
    let _: () = redis.expire(key, 3600).await.unwrap_or_default();
    Ok(())
}

/// Create a tag alias
pub async fn create_tag_alias(
    pool: &PgPool,
    alias_name: &str,
    canonical_tag_id: i32,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO tag_aliases (alias_name, canonical_tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(alias_name)
    .bind(canonical_tag_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Merge source tag into target tag: reassign all fic_tags, then delete source
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    // Reassign fic_tags from source to target (skip duplicates)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft
           WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO UPDATE SET score = GREATEST(fic_tags.score, EXCLUDED.score)"#,
    )
    .bind(source_tag_id)
    .bind(target_tag_id)
    .execute(pool)
    .await?;

    // Delete source fic_tags
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    // Delete source tag
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete a canonical tag (only if no fics use it)
pub async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    if !force {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1")
            .bind(tag_id)
            .fetch_one(pool)
            .await?;
        if count.0 > 0 {
            return Err(crate::error::AppError::BadRequest(format!(
                "tag {} is used by {} fics, use ?force=true",
                tag_id, count.0
            )));
        }
    }
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Work CRUD queries ──────────────────────────────────────────────

// ── Scheduled forum topics ───────────────────────────────────────���──────────

/// One forum topic that this call just published (was flipped out of the
/// scheduled queue).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PublishedTopic {
    pub id: i64,
    pub author_id: i32,
    pub title: String,
    pub topic_slug: Option<String>,
}

/// Publish every forum topic whose `scheduled_at` has come due, and notify each
/// author. Returns the rows this call flipped.
///
/// The `UPDATE … RETURNING` claims the rows, so a concurrent run cannot
/// double-notify: whichever run performs the flip owns the notification. This is
/// the single implementation — the `publish-scheduled` binary calls it, and so
/// does the scheduled-topics test, which previously pasted a copy of the flip
/// SQL inline and therefore asserted against a query that could drift from the
/// one cron actually runs.
pub async fn publish_due_topics(pool: &PgPool) -> AppResult<Vec<PublishedTopic>> {
    let due = sqlx::query_as::<_, PublishedTopic>(
        "UPDATE forum_topics SET scheduled_at = NULL
         WHERE scheduled_at IS NOT NULL AND scheduled_at <= NOW()
           AND deleted_at IS NULL
         RETURNING id, author_id, title, topic_slug",
    )
    .fetch_all(pool)
    .await?;

    for topic in &due {
        let link = format!("/forum/topic/{}", topic.id);
        let display = match &topic.topic_slug {
            Some(s) => format!("/forum/topic/{s}"),
            None => link.clone(),
        };
        let body = format!(
            "Your scheduled topic \"{}\" is now published.",
            topic.title
        );
        // A failed notification must not abort the run: the topic is already
        // published and un-claiming it would republish it on the next tick.
        let _ = create_notification(
            pool,
            topic.author_id,
            "topic_published",
            "Scheduled topic published",
            Some(&body),
            Some(&display),
            Some("forum_topic"),
            Some(&topic.id.to_string()),
        )
        .await;
    }

    Ok(due)
}
