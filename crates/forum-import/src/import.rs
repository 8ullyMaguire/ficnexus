use anyhow::{Context, Result};
use sqlx::PgPool;

use crate::schema::{ImportFile, ms_to_datetime, ms_to_optional_datetime};

/// Import statistics.
#[derive(Debug, Default)]
pub struct ImportStats {
    pub users_inserted: u64,
    pub users_skipped: u64,
    pub categories_inserted: u64,
    pub categories_skipped: u64,
    pub topics_inserted: u64,
    pub topics_skipped: u64,
    pub posts_inserted: u64,
    pub posts_skipped: u64,
    pub notifications_created: u64,
}

/// Run the full import. If dry_run is true, validates without writing.
pub async fn run_import(pool: &PgPool, data: &ImportFile, dry_run: bool) -> Result<ImportStats> {
    let mut stats = ImportStats::default();

    // Phase 1: Users
    tracing::info!("Importing {} users...", data.users.len());
    for user in &data.users {
        if dry_run {
            tracing::debug!("[dry-run] Would import user {} ({})", user.uid, user.username);
            stats.users_skipped += 1;
            continue;
        }
        match import_user(pool, user).await {
            Ok(inserted) => {
                if inserted {
                    stats.users_inserted += 1;
                } else {
                    stats.users_skipped += 1;
                }
            }
            Err(e) => {
                tracing::warn!("Skipping user {} ({}): {}", user.uid, user.username, e);
                stats.users_skipped += 1;
            }
        }
    }

    // Phase 2: Categories
    tracing::info!("Importing {} categories...", data.categories.len());
    for cat in &data.categories {
        if dry_run {
            tracing::debug!("[dry-run] Would import category {} ({})", cat.cid, cat.name);
            stats.categories_skipped += 1;
            continue;
        }
        match import_category(pool, cat).await {
            Ok(inserted) => {
                if inserted {
                    stats.categories_inserted += 1;
                } else {
                    stats.categories_skipped += 1;
                }
            }
            Err(e) => {
                tracing::warn!("Skipping category {} ({}): {}", cat.cid, cat.name, e);
                stats.categories_skipped += 1;
            }
        }
    }

    // Phase 3: Topics
    tracing::info!("Importing {} topics...", data.topics.len());
    for topic in &data.topics {
        if dry_run {
            tracing::debug!("[dry-run] Would import topic {}", topic.tid);
            stats.topics_skipped += 1;
            continue;
        }
        match import_topic(pool, topic).await {
            Ok(inserted) => {
                if inserted {
                    stats.topics_inserted += 1;
                } else {
                    stats.topics_skipped += 1;
                }
            }
            Err(e) => {
                tracing::warn!("Skipping topic {}: {}", topic.tid, e);
                stats.topics_skipped += 1;
            }
        }
    }

    // Phase 4: Posts
    tracing::info!("Importing {} posts...", data.posts.len());
    for post in &data.posts {
        if dry_run {
            tracing::debug!("[dry-run] Would import post {}", post.pid);
            stats.posts_skipped += 1;
            continue;
        }
        match import_post(pool, post).await {
            Ok(inserted) => {
                if inserted {
                    stats.posts_inserted += 1;
                } else {
                    stats.posts_skipped += 1;
                }
            }
            Err(e) => {
                tracing::warn!("Skipping post {}: {}", post.pid, e);
                stats.posts_skipped += 1;
            }
        }
    }

    // Phase 5: Notifications (one per unique user)
    if !dry_run {
        let mut imported_users: Vec<i64> = data.users.iter().map(|u| u.uid).collect();
        imported_users.dedup();
        tracing::info!("Creating import notifications for {} users...", imported_users.len());
        for uid in &imported_users {
            match create_import_notification(pool, *uid).await {
                Ok(()) => stats.notifications_created += 1,
                Err(e) => tracing::warn!("Notification for user {}: {}", uid, e),
            }
        }
    }

    Ok(stats)
}

/// Import a single user. Returns true if inserted, false if skipped (duplicate).
async fn import_user(pool: &PgPool, user: &crate::schema::ImportUser) -> Result<bool> {
    let row = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)",
    )
    .bind(user.uid)
    .fetch_one(pool)
    .await
    .context("check user exists")?;

    if row {
        return Ok(false);
    }

    let created_at = ms_to_datetime(user.joindate);
    let last_active = ms_to_optional_datetime(user.lastonline);
    let is_banned = user.banned.unwrap_or(0) != 0;
    let reputation = user.reputation.unwrap_or(0) as i32;

    sqlx::query(
        "INSERT INTO users (id, username, password_hash, email, role, reputation, created_at, last_active_at, is_banned)
         VALUES ($1, $2, '', $3, 0, $4, $5, $6, $7)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(user.uid)
    .bind(&user.username)
    .bind(&user.email)
    .bind(reputation)
    .bind(created_at)
    .bind(last_active)
    .bind(is_banned)
    .execute(pool)
    .await
    .context("insert user")?;

    Ok(true)
}

/// Import a single category. Returns true if inserted, false if skipped.
async fn import_category(pool: &PgPool, cat: &crate::schema::ImportCategory) -> Result<bool> {
    let row = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM forum_categories WHERE id = $1)",
    )
    .bind(cat.cid)
    .fetch_one(pool)
    .await
    .context("check category exists")?;

    if row {
        return Ok(false);
    }

    let slug = cat.slug.clone().unwrap_or_else(|| {
        cat.name.to_lowercase().replace(' ', "-")
    });
    let position = cat.order.unwrap_or(0) as i32;
    let is_mod_only = cat.is_private.unwrap_or(0) != 0;

    sqlx::query(
        "INSERT INTO forum_categories (id, slug, title, description, \"position\", is_mod_only, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW())
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(cat.cid)
    .bind(&slug)
    .bind(&cat.name)
    .bind(cat.description.as_deref().unwrap_or(""))
    .bind(position)
    .bind(is_mod_only)
    .execute(pool)
    .await
    .context("insert category")?;

    Ok(true)
}

/// Import a single topic. Returns true if inserted, false if skipped.
async fn import_topic(pool: &PgPool, topic: &crate::schema::ImportTopic) -> Result<bool> {
    let row = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM forum_topics WHERE id = $1)",
    )
    .bind(topic.tid)
    .fetch_one(pool)
    .await
    .context("check topic exists")?;

    if row {
        return Ok(false);
    }

    let created_at = ms_to_datetime(topic.timestamp);
    let last_activity = ms_to_optional_datetime(topic.lastposttimestamp)
        .unwrap_or(created_at);
    let updated_at = ms_to_optional_datetime(topic.lastposttimestamp);
    let status = if topic.locked.unwrap_or(0) != 0 {
        "locked"
    } else if topic.pinned.unwrap_or(0) != 0 {
        "pinned"
    } else {
        "open"
    };
    let is_hidden = topic.deleted.unwrap_or(0) != 0;
    let view_count = topic.viewcount.unwrap_or(0);
    let slug = topic.slug.as_deref().unwrap_or("");

    sqlx::query(
        "INSERT INTO forum_topics (id, category_id, author_id, title, body, status, view_count,
         last_activity_at, created_at, updated_at, is_hidden, topic_slug)
         VALUES ($1, $2, $3, $4, '', $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(topic.tid)
    .bind(topic.cid)
    .bind(topic.uid)
    .bind(&topic.title)
    .bind(status)
    .bind(view_count)
    .bind(last_activity)
    .bind(created_at)
    .bind(updated_at)
    .bind(is_hidden)
    .bind(slug)
    .execute(pool)
    .await
    .context("insert topic")?;

    Ok(true)
}

/// Import a single post. Returns true if inserted, false if skipped.
async fn import_post(pool: &PgPool, post: &crate::schema::ImportPost) -> Result<bool> {
    let row = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM forum_posts WHERE id = $1)",
    )
    .bind(post.pid)
    .fetch_one(pool)
    .await
    .context("check post exists")?;

    if row {
        return Ok(false);
    }

    let created_at = ms_to_datetime(post.timestamp);
    let edited_at = ms_to_optional_datetime(post.edited);
    let is_deleted = post.deleted.unwrap_or(0) != 0;
    let score = (post.upvotes.unwrap_or(0) - post.downvotes.unwrap_or(0)) as i32;
    let body = post.content.as_deref().unwrap_or("");

    sqlx::query(
        "INSERT INTO forum_posts (id, topic_id, author_id, body, score, edited_at, deleted_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(post.pid)
    .bind(post.tid)
    .bind(post.uid)
    .bind(body)
    .bind(score)
    .bind(edited_at)
    .bind(is_deleted.then(|| created_at))
    .bind(created_at)
    .execute(pool)
    .await
    .context("insert post")?;

    Ok(true)
}

/// Create an import notification for a user. Uses `notifications` table (NOT `forum_notifications`).
async fn create_import_notification(pool: &PgPool, user_id: i64) -> Result<()> {
    sqlx::query(
        "INSERT INTO notifications (user_id, notification_type, title, body, link, is_read, created_at)
         VALUES ($1, 'forum_import', 'Forum Import Complete', 'Your NodeBB content has been imported.', '/forum', true, NOW())
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .context("insert import notification")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        ms_to_datetime, ImportCategory, ImportFile, ImportPost, ImportTopic, ImportUser,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_ms_to_datetime() {
        // 2023-01-01 00:00:00 UTC = 1672531200000 ms
        let dt = ms_to_datetime(Some(1672531200000));
        assert_eq!(dt, Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn test_ms_to_datetime_none() {
        let dt = ms_to_datetime(None);
        // Should return current time (approximately)
        assert!(dt.signed_duration_since(Utc::now()).num_seconds().abs() < 5);
    }

    #[test]
    fn test_parse_import_file() {
        let json = r#"{
            "users": [{"uid": 1, "username": "alice", "email": "a@b.com", "joindate": 1672531200000}],
            "categories": [{"cid": 1, "name": "General", "slug": "general"}],
            "topics": [{"tid": 100, "cid": 1, "uid": 1, "title": "Hello", "timestamp": 1672531200000}],
            "posts": [{"pid": 1000, "tid": 100, "uid": 1, "content": "Hi!", "timestamp": 1672531200000}]
        }"#;

        let data: ImportFile = serde_json::from_str(json).unwrap();
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.categories.len(), 1);
        assert_eq!(data.topics.len(), 1);
        assert_eq!(data.posts.len(), 1);
        assert_eq!(data.users[0].username, "alice");
        assert_eq!(data.topics[0].title, "Hello");
    }

    #[test]
    fn test_parse_empty_file() {
        let json = r#"{}"#;
        let data: ImportFile = serde_json::from_str(json).unwrap();
        assert!(data.users.is_empty());
        assert!(data.categories.is_empty());
        assert!(data.topics.is_empty());
        assert!(data.posts.is_empty());
    }

    #[test]
    fn test_user_defaults() {
        let json = r#"{"uid": 1, "username": "bob"}"#;
        let user: ImportUser = serde_json::from_str(json).unwrap();
        assert_eq!(user.uid, 1);
        assert_eq!(user.username, "bob");
        assert!(user.email.is_none());
        assert!(user.joindate.is_none());
        assert!(user.banned.is_none());
    }

    #[test]
    fn test_category_slug_default() {
        let json = r#"{"cid": 1, "name": "My Category"}"#;
        let cat: ImportCategory = serde_json::from_str(json).unwrap();
        assert!(cat.slug.is_none());
    }

    #[test]
    fn test_topic_status_fields() {
        let json = r#"{"tid": 1, "cid": 1, "uid": 1, "title": "T", "locked": 1, "pinned": 1}"#;
        let topic: ImportTopic = serde_json::from_str(json).unwrap();
        assert_eq!(topic.locked, Some(1));
        assert_eq!(topic.pinned, Some(1));
    }

    #[test]
    fn test_post_score_calculation() {
        let upvotes = 10i64;
        let downvotes = 3i64;
        let score = (upvotes - downvotes) as i32;
        assert_eq!(score, 7);
    }

    #[test]
    fn test_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.users_inserted, 0);
        assert_eq!(stats.posts_skipped, 0);
    }

    #[test]
    fn test_import_file_partial() {
        // Only users and categories, no topics/posts
        let json = r#"{
            "users": [{"uid": 1, "username": "alice"}],
            "categories": [{"cid": 1, "name": "General"}]
        }"#;
        let data: ImportFile = serde_json::from_str(json).unwrap();
        assert_eq!(data.users.len(), 1);
        assert_eq!(data.categories.len(), 1);
        assert!(data.topics.is_empty());
        assert!(data.posts.is_empty());
    }
}
