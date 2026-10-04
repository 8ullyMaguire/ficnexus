use crate::db::models::*;
use crate::error::AppResult;
use sqlx::{PgPool, Row};

// ── Tag-related queries (v3) ─────────────────────────────────────────────

/// Look up a tag by exact name (COLLATE "C" case-sensitive)
pub async fn lookup_tag_by_name(
    pool: &PgPool,
    name: &str,
) -> AppResult<Option<(i32, String, i16)>> {
    let row = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT id, name, tag_type_id FROM tags WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Look up an alias, returning the canonical tag ID
pub async fn lookup_alias(pool: &PgPool, alias_name: &str) -> AppResult<Option<i32>> {
    let row = sqlx::query_scalar::<_, i32>(
        "SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = $1",
    )
    .bind(alias_name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Create a new canonical tag, returning its ID
pub async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> AppResult<i32> {
    let row: (i32,) =
        sqlx::query_as("INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(tag_type_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

/// Upsert a fic-tag association (no-op if exists)
pub async fn upsert_fic_tag(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    ip: &std::net::IpAddr,
    score: i16,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id) DO UPDATE SET
             added_by_ip = EXCLUDED.added_by_ip,
             score = GREATEST(EXCLUDED.score, fic_tags.score)"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(ip.to_string())
    .bind(score)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all tags for a fic, with scores and visibility
pub async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#,
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, type_id, score)| {
            let hidden = score < hidden_threshold;
            (id, name, type_id, score, hidden)
        })
        .collect())
}

// ═══════════════════════════════════════════════════════════════════
// Follow queries
// ═══════════════════════════════════════════════════════════════════

/// Follow a user
pub async fn follow_user(pool: &PgPool, follower_id: i32, followee_id: i32) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO follows (follower_id, followee_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(follower_id)
    .bind(followee_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Follow a work
pub async fn follow_work(pool: &PgPool, follower_id: i32, work_id: i32) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO follows (follower_id, work_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(follower_id)
    .bind(work_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Follow an author by name
pub async fn follow_author(pool: &PgPool, follower_id: i32, author_name: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO follows (follower_id, author_name) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(follower_id)
    .bind(author_name)
    .execute(pool)
    .await?;
    Ok(())
}

/// Unfollow
pub async fn unfollow(pool: &PgPool, follower_id: i32, follow_id: i64) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM follows WHERE id = $1 AND follower_id = $2")
        .bind(follow_id)
        .bind(follower_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List who a user follows
pub async fn list_follows(pool: &PgPool, user_id: i32) -> AppResult<Vec<Follow>> {
    let rows = sqlx::query_as::<_, Follow>(
        r#"SELECT id, follower_id, followee_id, work_id, author_name, created_at, last_seen
           FROM follows WHERE follower_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get followers of a user
pub async fn get_followers(pool: &PgPool, user_id: i32) -> AppResult<Vec<Follow>> {
    let rows = sqlx::query_as::<_, Follow>(
        r#"SELECT id, follower_id, followee_id, work_id, author_name, created_at, last_seen
           FROM follows WHERE followee_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Check if user follows something
pub async fn is_following_user(
    pool: &PgPool,
    follower_id: i32,
    followee_id: i32,
) -> AppResult<bool> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM follows WHERE follower_id = $1 AND followee_id = $2 LIMIT 1",
    )
    .bind(follower_id)
    .bind(followee_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

pub async fn is_following_work(pool: &PgPool, follower_id: i32, work_id: i32) -> AppResult<bool> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM follows WHERE follower_id = $1 AND work_id = $2 LIMIT 1")
            .bind(follower_id)
            .bind(work_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

/// Get the follow id for (user, work) — used by the fic page follow button.
pub async fn find_follow_for_work(
    pool: &PgPool,
    follower_id: i32,
    work_id: i32,
) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM follows WHERE follower_id = $1 AND work_id = $2 LIMIT 1")
            .bind(follower_id)
            .bind(work_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Get the follow id for (user, author name).
pub async fn find_follow_for_author(
    pool: &PgPool,
    follower_id: i32,
    author_name: &str,
) -> AppResult<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM follows WHERE follower_id = $1 AND author_name = $2 LIMIT 1",
    )
    .bind(follower_id)
    .bind(author_name)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.0))
}

/// Mark a follow's `last_seen` as now (idempotent; no row → no-op).
pub async fn mark_follow_seen(pool: &PgPool, follow_id: i64, user_id: i32) -> AppResult<bool> {
    let result =
        sqlx::query("UPDATE follows SET last_seen = NOW() WHERE id = $1 AND follower_id = $2")
            .bind(follow_id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Works the user follows (by work_id), ordered by fic_updated DESC. Each row
/// carries the fic_updated timestamp plus the follow's last_seen so the
/// updates feed can render "updated X ago" and a NEW badge.
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

pub async fn list_followed_work_updates(
    pool: &PgPool,
    user_id: i32,
) -> AppResult<Vec<FollowedWorkUpdate>> {
    let rows = sqlx::query_as::<_, FollowedWorkUpdate>(
        r#"SELECT f.id AS follow_id, f.work_id, fi.id AS url_id, fi.title, fi.author,
                  fi.words, fi.chapters, fi.status, fi.fic_updated, f.last_seen
           FROM follows f
           JOIN fic_info fi ON fi.work_id = f.work_id
           WHERE f.follower_id = $1 AND f.work_id IS NOT NULL
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions wx
                 WHERE wx.follow_id = f.id
                   AND wx.exclude_type = 'work'
                   AND wx.exclude_work_id = f.work_id
             )
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions sx
                 WHERE sx.follow_id = f.id
                   AND sx.exclude_type = 'series'
                   AND sx.exclude_series_id IN (
                       SELECT sw.series_id FROM series_works sw WHERE sw.work_id = f.work_id
                   )
             )
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions fx
                 WHERE fx.follow_id = f.id
                   AND fx.exclude_type = 'fandom'
                   AND fx.exclude_fandom IN (
                       SELECT t.name FROM tags t
                       JOIN fic_tags ft ON ft.tag_id = t.id
                       JOIN fic_info fl ON fl.id = ft.url_id
                       WHERE fl.work_id = f.work_id AND t.tag_type_id = 1
                   )
             )
           ORDER BY fi.fic_updated DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Number of followed works whose fic_updated is newer than last_seen
/// (used for the nav "Updates" badge count). Follow exclusions are applied
/// so excluded works/series/fandoms never count toward the unseen badge.
pub async fn count_unseen_followed_updates(pool: &PgPool, user_id: i32) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*)
           FROM follows f
           JOIN fic_info fi ON fi.work_id = f.work_id
           WHERE f.follower_id = $1
             AND f.work_id IS NOT NULL
             AND (f.last_seen IS NULL OR fi.fic_updated > f.last_seen)
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions wx
                 WHERE wx.follow_id = f.id
                   AND wx.exclude_type = 'work'
                   AND wx.exclude_work_id = f.work_id
             )
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions sx
                 WHERE sx.follow_id = f.id
                   AND sx.exclude_type = 'series'
                   AND sx.exclude_series_id IN (
                       SELECT sw.series_id FROM series_works sw WHERE sw.work_id = f.work_id
                   )
             )
             AND NOT EXISTS (
                 SELECT 1 FROM follow_exclusions fx
                 WHERE fx.follow_id = f.id
                   AND fx.exclude_type = 'fandom'
                   AND fx.exclude_fandom IN (
                       SELECT t.name FROM tags t
                       JOIN fic_tags ft ON ft.tag_id = t.id
                       JOIN fic_info fl ON fl.id = ft.url_id
                       WHERE fl.work_id = f.work_id AND t.tag_type_id = 1
                   )
             )"#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

// ── Follow exclusions ─────────────────────────────────────────────────
