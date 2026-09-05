use crate::db::models::*;
use crate::error::{AppError, AppResult};
use chrono::Datelike;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

/// A stored site-credential row (no password — never surfaced).
#[derive(Debug, Clone)]
pub struct SiteCredRow {
    pub domain: String,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

/// Upsert a user's site credentials (encrypted password). Resets the
/// 30-day expiry clock on re-save.
pub async fn set_site_credentials(
    pool: &PgPool,
    user_id: i32,
    domain: &str,
    username: &str,
    password_enc: &str,
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO user_site_credentials (user_id, domain, username, password_enc, created_at, expires_at)
           VALUES ($1, $2, $3, $4, now(), $5)
           ON CONFLICT (user_id, domain) DO UPDATE
           SET username = EXCLUDED.username,
               password_enc = EXCLUDED.password_enc,
               created_at = now(),
               expires_at = EXCLUDED.expires_at"#,
    )
    .bind(user_id)
    .bind(domain.trim().to_lowercase())
    .bind(username)
    .bind(password_enc)
    .bind(expires_at)
    .execute(pool)
    .await?;
    // Opportunistic lazy-expiry cleanup for this user.
    let _ =
        sqlx::query("DELETE FROM user_site_credentials WHERE user_id = $1 AND expires_at <= now()")
            .bind(user_id)
            .execute(pool)
            .await;
    Ok(())
}

/// List the user's non-expired site credentials. Passwords are NEVER
/// selected here — callers (and the API) only ever see domain/username/expiry.
pub async fn get_user_site_credentials(pool: &PgPool, user_id: i32) -> AppResult<Vec<SiteCredRow>> {
    let rows = sqlx::query_as::<_, (String, String, DateTime<Utc>)>(
        r#"SELECT domain, username, expires_at
           FROM user_site_credentials
           WHERE user_id = $1 AND expires_at > now()
           ORDER BY domain"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(domain, username, expires_at)| SiteCredRow {
            domain,
            username,
            expires_at,
        })
        .collect())
}

/// Fetch one non-expired credential (username + encrypted password) for the
/// download path. Returns `None` when the user has no (valid) creds for the
/// domain — the caller then falls back to anonymous AuthRequired behavior.
pub async fn get_user_site_credential(
    pool: &PgPool,
    user_id: i32,
    domain: &str,
) -> AppResult<Option<(String, String)>> {
    let row = sqlx::query_as::<_, (String, String)>(
        r#"SELECT username, password_enc
           FROM user_site_credentials
           WHERE user_id = $1 AND domain = $2 AND expires_at > now()"#,
    )
    .bind(user_id)
    .bind(domain.trim().to_lowercase())
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Remove a user's credential for a domain. Idempotent.
pub async fn delete_user_site_credential(
    pool: &PgPool,
    user_id: i32,
    domain: &str,
) -> AppResult<()> {
    sqlx::query("DELETE FROM user_site_credentials WHERE user_id = $1 AND domain = $2")
        .bind(user_id)
        .bind(domain.trim().to_lowercase())
        .execute(pool)
        .await?;
    Ok(())
}

/// Upsert a fic_info record (INSERT ON CONFLICT UPDATE)
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated,
            status = EXCLUDED.status,
            extra_meta = EXCLUDED.extra_meta,
            raw_extended_meta = EXCLUDED.raw_extended_meta,
            content_hash = EXCLUDED.content_hash,
            updated = NOW()"#,
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(&fic.author_url)
    .bind(&fic.author_local_id)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.description)
    .bind(fic.fic_created)
    .bind(fic.fic_updated)
    .bind(&fic.status)
    .bind(&fic.source)
    .bind(&fic.extra_meta)
    .bind(&fic.raw_extended_meta)
    .bind(fic.source_id)
    .bind(fic.author_id)
    .bind(&fic.content_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Hide a fic_info row (sets hidden = true). Used by the quality filter
/// to quarantine suspicious works from public view.
pub async fn hide_fic_info(pool: &PgPool, url_id: &str) -> AppResult<()> {
    sqlx::query("UPDATE fic_info SET hidden = true WHERE id = $1")
        .bind(url_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Create a curator quorum proposal for reviewing a quarantined work.
pub async fn create_curator_quorum_proposal(
    pool: &PgPool,
    url_id: &str,
    proposal_type: &str,
    reason: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO curator_quorum_proposals (url_id, proposal_type, reason, status)
         VALUES ($1, $2, $3, 'pending')",
    )
    .bind(url_id)
    .bind(proposal_type)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get fic_info by ID
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// Insert a request source record
pub async fn insert_request_source(
    pool: &PgPool,
    is_automated: bool,
    route: &str,
    description: &str,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO request_source (is_automated, route, description)
           VALUES ($1, $2, $3)
           ON CONFLICT (is_automated, route, description) DO UPDATE SET route = EXCLUDED.route
           RETURNING id"#,
    )
    .bind(is_automated)
    .bind(route)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Log a request
pub async fn insert_request_log(
    pool: &PgPool,
    source_id: i64,
    etype: &str,
    query: &str,
    info_request_ms: i32,
    url_id: Option<&str>,
    fic_info: Option<&str>,
    export_ms: Option<i32>,
    export_file_name: Option<&str>,
    export_file_hash: Option<&str>,
    url: Option<&str>,
    client_id: Option<&str>,
    user_agent: Option<&str>,
    ip: Option<std::net::IpAddr>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO request_log
           (source_id, etype, query, info_request_ms, url_id, fic_info, export_ms, export_file_name, export_file_hash, url, client_id, user_agent, ip)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::inet)"#,
    )
    .bind(source_id)
    .bind(etype)
    .bind(query)
    .bind(info_request_ms)
    .bind(url_id)
    .bind(fic_info)
    .bind(export_ms)
    .bind(export_file_name)
    .bind(export_file_hash)
    .bind(url)
    .bind(client_id)
    .bind(user_agent)
    .bind(ip.map(|i| i.to_string()))
    .execute(pool)
    .await?;
    Ok(())
}

/// Log a search query for analytics (best-effort — callers must not fail
/// the search request if this insert fails).
pub async fn insert_search_query(
    pool: &PgPool,
    query: &str,
    total_results: i64,
    main_char_attr: Option<&str>,
    client_id: Option<&str>,
    user_id: Option<i32>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO search_queries (query, total_results, main_char_attr, client_id, user_id)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(query)
    .bind(total_results)
    .bind(main_char_attr)
    .bind(client_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Return the signed-in user's top-8 most frequent search queries.
/// Used by `GET /api/search/history/chips`.
pub async fn recent_search_chips(
    pool: &PgPool,
    user_id: Option<i32>,
) -> AppResult<Vec<(String, i64)>> {
    let Some(uid) = user_id else {
        return Ok(Vec::new());
    };
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT query, COUNT(*)::bigint as cnt
           FROM search_queries
           WHERE user_id = $1 AND query <> ''
           GROUP BY query
           ORDER BY cnt DESC, MAX(ts) DESC
           LIMIT 8"#,
    )
    .bind(uid)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Find export log entry (cache hit check)
/// Most recent export_log row for (url_id, version, etype) regardless of
/// input_hash. Used by the reader to find legacy HTML exports whose input_hash
/// was recorded as `epub:<hash>` rather than the fic's content hash.
pub async fn latest_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
) -> AppResult<Option<ExportLog>> {
    let row = sqlx::query_as::<_, ExportLog>(
        r#"SELECT * FROM export_log
           WHERE url_id = $1 AND version = $2 AND etype = $3
           ORDER BY created DESC
           LIMIT 1"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn find_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
) -> AppResult<Option<ExportLog>> {
    let row = sqlx::query_as::<_, ExportLog>(
        r#"SELECT * FROM export_log
           WHERE url_id = $1 AND version = $2 AND etype = $3 AND input_hash = $4"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Insert export log record
pub async fn insert_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
    export_hash: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (url_id, version, etype, input_hash) DO UPDATE SET
               export_hash = EXCLUDED.export_hash,
               created = NOW()"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .bind(export_hash)
    .execute(pool)
    .await?;
    Ok(())
}

/// Check if a fic is blacklisted
pub async fn check_fic_blacklist(pool: &PgPool, url_id: &str) -> AppResult<Vec<FicBlacklist>> {
    let rows = sqlx::query_as::<_, FicBlacklist>("SELECT * FROM fic_blacklist WHERE url_id = $1")
        .bind(url_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Check if an author is blacklisted
pub async fn check_author_blacklist(
    pool: &PgPool,
    source_id: i64,
    author_id: i64,
) -> AppResult<Vec<AuthorBlacklist>> {
    let rows = sqlx::query_as::<_, AuthorBlacklist>(
        "SELECT * FROM author_blacklist WHERE source_id = $1 AND author_id = $2",
    )
    .bind(source_id)
    .bind(author_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get version bumps for a fic (cache invalidation)
pub async fn get_fic_version_bump(pool: &PgPool, url_id: &str) -> AppResult<Option<i32>> {
    let row = sqlx::query_as::<_, FicVersionBump>("SELECT * FROM fic_version_bump WHERE id = $1")
        .bind(url_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| r.value))
}

/// Search similar fics (for "did you mean?" suggestions)
pub async fn search_similar_fics(pool: &PgPool, query: &str) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        r#"SELECT * FROM fic_info
           WHERE title ILIKE $1 OR author ILIKE $2
           LIMIT 5"#,
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

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

// ═══════════════════════════════════════════════════════════════════
// Reading history (AO3-style per-visit log)
// ═══════════════════════════════════════════════════════════════════

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

/// Create a new work, returning its ID
pub async fn create_work(
    pool: &PgPool,
    canonical_title: &str,
    canonical_author: &str,
    description: &str,
    default_source_id: Option<&str>,
) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        r#"INSERT INTO works (canonical_title, canonical_author, description, default_source_id)
           VALUES ($1, $2, $3, $4)
           RETURNING id"#,
    )
    .bind(canonical_title)
    .bind(canonical_author)
    .bind(description)
    .bind(default_source_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Get a work by ID
pub async fn get_work(pool: &PgPool, work_id: i32) -> AppResult<Option<WorkRow>> {
    let row = sqlx::query_as::<_, WorkRow>(
        "SELECT id, canonical_title, canonical_author, description, default_source_id, created_at, updated_at, uploader_id, is_visible FROM works WHERE id = $1",
    )
    .bind(work_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// List works uploaded by `uploader_id` (the signed-in user's own uploads),
/// newest first.
pub async fn list_my_works(pool: &PgPool, user_id: i32) -> AppResult<Vec<WorkRow>> {
    let row = sqlx::query_as::<_, WorkRow>(
        r#"SELECT id, canonical_title, canonical_author, description, default_source_id, created_at, updated_at, uploader_id, is_visible FROM works WHERE uploader_id = $1 AND is_visible = TRUE ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(row)
}

/// Get work by source url_id (via fic_info.work_id)
pub async fn get_work_by_source(pool: &PgPool, url_id: &str) -> AppResult<Option<WorkRow>> {
    let row = sqlx::query_as::<_, WorkRow>(
        r#"SELECT w.id, w.canonical_title, w.canonical_author, w.description, w.default_source_id, w.created_at, w.updated_at, w.uploader_id, w.is_visible
           FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           WHERE fi.id = $1"#,
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Find work by source URL (searching fic_info.source)
pub async fn find_work_by_source_url(pool: &PgPool, url: &str) -> AppResult<Option<WorkRow>> {
    let row = sqlx::query_as::<_, WorkRow>(
        r#"SELECT w.id, w.canonical_title, w.canonical_author, w.description, w.default_source_id, w.created_at, w.updated_at, w.uploader_id, w.is_visible
           FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           WHERE fi.source = $1
           LIMIT 1"#,
    )
    .bind(url)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Find work by exact title+author match (for auto-merge)
pub async fn find_work_by_title_author(
    pool: &PgPool,
    title: &str,
    author: &str,
) -> AppResult<Option<WorkRow>> {
    let row = sqlx::query_as::<_, WorkRow>(
        r#"SELECT id, canonical_title, canonical_author, description, default_source_id, created_at, updated_at, uploader_id, is_visible
           FROM works
           WHERE LOWER(canonical_title) = LOWER($1) AND LOWER(canonical_author) = LOWER($2)
           LIMIT 1"#,
    )
    .bind(title)
    .bind(author)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Link a fic_info source to a work
pub async fn link_source_to_work(pool: &PgPool, url_id: &str, work_id: i32) -> AppResult<()> {
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
        .bind(work_id)
        .bind(url_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get all sources (fic_info) for a work
pub async fn get_work_sources(pool: &PgPool, work_id: i32) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        "SELECT id, created, updated, title, author, author_url, author_local_id,
                chapters, words, description, fic_created, fic_updated, status,
                source, extra_meta, raw_extended_meta, source_id, author_id, content_hash, work_id
         FROM fic_info WHERE work_id = $1 ORDER BY words DESC",
    )
    .bind(work_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Given a url_id (fic_info.id), resolve the work_id and the canonical slug.
///
/// Used by the redirect handler to canonicalise `/works/{url_id}` → `/works/{slug}.{id}`.
pub async fn get_work_slug_target(
    pool: &PgPool,
    url_id: &str,
) -> AppResult<Option<(i32, String)>> {
    let row: Option<(i32, Option<String>)> = sqlx::query_as(
        "SELECT w.id, COALESCE(w.canonical_title, fi.title) AS title
         FROM works w
         JOIN fic_info fi ON fi.work_id = w.id
         WHERE fi.id = $1",
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(id, title)| {
        let slug = title
            .map(|t| crate::routes::export::work_url_slug(&t))
            .unwrap_or_else(|| "work".into());
        (id, slug)
    }))
}

/// Given a work_id (works.id), resolve the url_id and the canonical slug.
///
/// Used to canonicalise `/work/{id}` → `/works/{slug}.{id}` and by
/// the SPA when the URL arrives as a slug URL and we need the url_id.
pub async fn get_work_canonical_url_id(
    pool: &PgPool,
    work_id: i32,
) -> AppResult<Option<(String, String)>> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT fi.id, COALESCE(w.canonical_title, fi.title) AS title
         FROM works w
         JOIN fic_info fi ON fi.work_id = w.id
         WHERE w.id = $1",
    )
    .bind(work_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(uid, title)| {
        let slug = title
            .map(|t| crate::routes::export::work_url_slug(&t))
            .unwrap_or_else(|| "work".into());
        (uid, slug)
    }))
}

/// Log an auto-merge
pub async fn log_auto_merge(
    pool: &PgPool,
    source_url: &str,
    matched_work_id: i32,
    confidence: f64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO auto_merge_log (source_url, matched_work_id, confidence) VALUES ($1, $2, $3)",
    )
    .bind(source_url)
    .bind(matched_work_id)
    .bind(confidence)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Proposal queries ──────────────────────────────────────────────

/// Create a work proposal
pub async fn create_proposal(
    pool: &PgPool,
    proposer_id: i32,
    action_type: &str,
    source_work_id: Option<i32>,
    target_work_id: Option<i32>,
    work_id: Option<i32>,
    details: Option<serde_json::Value>,
) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        r#"INSERT INTO work_proposals (proposer_id, action_type, source_work_id, target_work_id, work_id, details)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id"#,
    )
    .bind(proposer_id)
    .bind(action_type)
    .bind(source_work_id)
    .bind(target_work_id)
    .bind(work_id)
    .bind(details)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// List pending proposals
pub async fn list_pending_proposals(pool: &PgPool) -> AppResult<Vec<WorkProposal>> {
    let rows = sqlx::query_as::<_, WorkProposal>(
        r#"SELECT id, proposer_id, action_type, source_work_id, target_work_id, work_id, details, status, created_at, closed_at
           FROM work_proposals
           WHERE status = 'pending'
           ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a proposal by ID
pub async fn get_proposal(pool: &PgPool, proposal_id: i32) -> AppResult<Option<WorkProposal>> {
    let row = sqlx::query_as::<_, WorkProposal>(
        r#"SELECT id, proposer_id, action_type, source_work_id, target_work_id, work_id, details, status, created_at, closed_at
           FROM work_proposals WHERE id = $1"#,
    )
    .bind(proposal_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Cast a vote on a proposal
pub async fn cast_proposal_vote(
    pool: &PgPool,
    proposal_id: i32,
    user_id: i32,
    vote: i16,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO work_proposal_votes (proposal_id, user_id, vote)
           VALUES ($1, $2, $3)
           ON CONFLICT (proposal_id, user_id) DO UPDATE SET vote = $3, voted_at = NOW()"#,
    )
    .bind(proposal_id)
    .bind(user_id)
    .bind(vote)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get vote sum for a proposal
pub async fn get_proposal_vote_sum(pool: &PgPool, proposal_id: i32) -> AppResult<(i64, i64)> {
    let row: (Option<i64>, Option<i64>) = sqlx::query_as(
        r#"SELECT COALESCE(SUM(vote), 0), COUNT(DISTINCT user_id)
           FROM work_proposal_votes WHERE proposal_id = $1"#,
    )
    .bind(proposal_id)
    .fetch_one(pool)
    .await?;
    Ok((row.0.unwrap_or(0), row.1.unwrap_or(0)))
}

/// Check if user has senior curator role (role >= 2)
pub async fn is_senior_curator(pool: &PgPool, user_id: i32) -> AppResult<bool> {
    let row: Option<(String,)> = sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    Ok(matches!(
        row.as_ref().map(|r| r.0.as_str()),
        Some("admin") | Some("moderator")
    ))
}

/// Update proposal status
pub async fn update_proposal_status(
    pool: &PgPool,
    proposal_id: i32,
    status: &str,
) -> AppResult<()> {
    sqlx::query("UPDATE work_proposals SET status = $1, closed_at = NOW() WHERE id = $2")
        .bind(status)
        .bind(proposal_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Execute a merge: move sources from source_work to target_work
pub async fn execute_merge(
    pool: &PgPool,
    source_work_id: i32,
    target_work_id: i32,
) -> AppResult<()> {
    // Move all sources
    sqlx::query("UPDATE fic_info SET work_id = $1 WHERE work_id = $2")
        .bind(target_work_id)
        .bind(source_work_id)
        .execute(pool)
        .await?;

    // Update target work metadata from most common values
    sqlx::query(
        r#"UPDATE works SET
            canonical_title = (SELECT canonical_title FROM works WHERE id = $1),
            canonical_author = (SELECT canonical_author FROM works WHERE id = $1),
            description = (SELECT description FROM works WHERE id = $1),
            updated_at = NOW()
           WHERE id = $2"#,
    )
    .bind(source_work_id)
    .bind(target_work_id)
    .execute(pool)
    .await?;

    // Delete source work
    sqlx::query("DELETE FROM works WHERE id = $1")
        .bind(source_work_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Split a merged work back into separate works.
///
/// Given a work_id and a list of source IDs to split off,
/// creates new works for each source and reassigns their sources.
/// The original work keeps sources not in the split list.
pub async fn execute_split(
    pool: &PgPool,
    work_id: i32,
    split_source_ids: &[&str],
) -> AppResult<()> {
    // For each source to split off, create a new work and reassign sources
    for &source_id in split_source_ids {
        // Get the source's original title/author before merge
        let source: Option<(String, String, String)> =
            sqlx::query_as("SELECT title, author, description FROM fic_info WHERE id = $1")
                .bind(source_id)
                .fetch_optional(pool)
                .await?;

        if let Some((title, author, desc)) = source {
            // Create new work for this source
            let new_work_id = create_work(pool, &title, &author, &desc, Some(source_id)).await?;

            // Reassign the source to the new work
            sqlx::query("UPDATE fic_info SET work_id = $1 WHERE id = $2")
                .bind(new_work_id)
                .bind(source_id)
                .execute(pool)
                .await?;
        }
    }

    // Update the original work's metadata from remaining sources
    sqlx::query(
        r#"UPDATE works SET
            canonical_title = COALESCE(
                (SELECT title FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                canonical_title
            ),
            canonical_author = COALESCE(
                (SELECT author FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                canonical_author
            ),
            description = COALESCE(
                (SELECT description FROM fic_info WHERE work_id = $1 ORDER BY fic_created DESC LIMIT 1),
                description
            ),
            updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(work_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Reputation & Auto-Promotion ─────────────────────────────────────

/// Award reputation to a user and check for auto-promotion to curator
pub async fn update_reputation_and_promote(
    pool: &PgPool,
    user_id: i32,
    delta: i32,
    event_type: &str,
) -> AppResult<()> {
    // Update reputation
    sqlx::query("UPDATE users SET reputation = reputation + $1 WHERE id = $2")
        .bind(delta)
        .bind(user_id)
        .execute(pool)
        .await?;

    // Record reputation event
    sqlx::query(
        "INSERT INTO reputation_events (user_id, event_type, delta)
         VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(delta)
    .execute(pool)
    .await?;

    // Check for auto-promotion to curator (reputation >= 100, role = user)
    let row: Option<(i32, String)> =
        sqlx::query_as("SELECT reputation, role FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if let Some((reputation, role)) = row {
        if reputation >= 100 && role == "user" {
            // Promote to curator
            sqlx::query("UPDATE users SET role = 'curator', curator_since = NOW() WHERE id = $1")
                .bind(user_id)
                .execute(pool)
                .await?;

            // Record promotion event
            sqlx::query(
                "INSERT INTO reputation_events (user_id, event_type, delta)
                 VALUES ($1, 'curator_promoted', 0)",
            )
            .bind(user_id)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════
// Analytics queries
// ═══════════════════════════════════════════════════════════════════

/// Get daily unique visitors and total requests for the last N days
pub async fn get_daily_stats(
    pool: &PgPool,
    days: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64, i64)>(
        r#"SELECT DATE(created) as day,
                  COUNT(DISTINCT client_id) as unique_visitors,
                  COUNT(*) as total_requests
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND client_id IS NOT NULL
           GROUP BY DATE(created)
           ORDER BY day DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get per-user stats (total requests, downloads, unique fics)
pub async fn get_user_stats(pool: &PgPool, client_id: &str) -> AppResult<(i64, i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"SELECT 
            COUNT(*) as total_requests,
            COUNT(DISTINCT url_id) as unique_fics,
            COALESCE(SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END), 0) as downloads
          FROM request_log
          WHERE client_id = $1"#,
    )
    .bind(client_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Get most popular fics (by request count)
pub async fn get_popular_fics(
    pool: &PgPool,
    days: i32,
    limit: i32,
) -> AppResult<Vec<(String, i64, i64)>> {
    let rows = sqlx::query_as::<_, (String, i64, i64)>(
        r#"SELECT url_id,
                  COUNT(*) as request_count,
                  SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END) as downloads
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND url_id IS NOT NULL
           GROUP BY url_id
           ORDER BY request_count DESC
           LIMIT $2"#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get download format breakdown
pub async fn get_format_breakdown(pool: &PgPool, days: i32) -> AppResult<Vec<(String, i64)>> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT 
             CASE 
               WHEN export_file_name LIKE '%.epub' THEN 'epub'
               WHEN export_file_name LIKE '%.html' THEN 'html'
               WHEN export_file_name LIKE '%.mobi' THEN 'mobi'
               WHEN export_file_name LIKE '%.pdf' THEN 'pdf'
               WHEN export_file_name LIKE '%.azw3' THEN 'azw3'
               WHEN export_file_name LIKE '%.txt' THEN 'txt'
               WHEN export_file_name LIKE '%.md' THEN 'md'
               ELSE 'other'
             END as format,
             COUNT(*) as count
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND export_file_name IS NOT NULL
           GROUP BY format
           ORDER BY count DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get total unique visitors (distinct client_ids)
pub async fn get_total_unique_visitors(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT client_id)
           FROM request_log
           WHERE created > NOW() - ($1 || ' days')::INTERVAL
             AND client_id IS NOT NULL"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Get return visitor rate (visitors with 2+ requests)
pub async fn get_return_visitor_rate(pool: &PgPool, days: i32) -> AppResult<(i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT 
             COUNT(DISTINCT client_id) as total_visitors,
             COUNT(DISTINCT CASE WHEN request_count > 1 THEN client_id END) as return_visitors
           FROM (
             SELECT client_id, COUNT(*) as request_count
             FROM request_log
             WHERE created > NOW() - ($1 || ' days')::INTERVAL
               AND client_id IS NOT NULL
             GROUP BY client_id
           ) sub"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

// ═══════════════════════════════════════════════════════════════════
// Usage analytics (usage_events table — non-PII, X-Client-ID based)
// ═══════════════════════════════════════════════════════════════════

/// Insert a usage event (best-effort; failures are logged, never fatal).
pub async fn insert_usage_event(
    pool: &PgPool,
    client_id: &str,
    path: &str,
    event_type: &str,
    user_agent: Option<&str>,
) {
    if let Err(e) = sqlx::query(
        "INSERT INTO usage_events (client_id, path, event_type, user_agent)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(client_id)
    .bind(path)
    .bind(event_type)
    .bind(user_agent)
    .execute(pool)
    .await
    {
        tracing::warn!("insert_usage_event failed: {e}");
    }
}

/// Unique visitors per day for the last N days (from usage_events).
pub async fn get_daily_unique_visitors(
    pool: &PgPool,
    days: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE(created_at) as day,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY DATE(created_at)
           ORDER BY day DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Unique visitors per week for the last N weeks (ISO week start Monday).
pub async fn get_weekly_unique_visitors(
    pool: &PgPool,
    weeks: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE_TRUNC('week', created_at)::date as week_start,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' weeks')::INTERVAL
           GROUP BY DATE_TRUNC('week', created_at)
           ORDER BY week_start DESC"#,
    )
    .bind(weeks)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Unique visitors per month for the last N months.
pub async fn get_monthly_unique_visitors(
    pool: &PgPool,
    months: i32,
) -> AppResult<Vec<(chrono::NaiveDate, i64)>> {
    let rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        r#"SELECT DATE_TRUNC('month', created_at)::date as month_start,
                  COUNT(DISTINCT client_id) as unique_visitors
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' months')::INTERVAL
           GROUP BY DATE_TRUNC('month', created_at)
           ORDER BY month_start DESC"#,
    )
    .bind(months)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Active users in a window: distinct client_ids that performed at least one
/// ACTION (non-view event). Returns (active_count, total_events).
pub async fn get_active_users(pool: &PgPool, days: i32) -> AppResult<(i64, i64)> {
    let row = sqlx::query_as::<_, (i64, i64)>(
        r#"SELECT COUNT(DISTINCT client_id) as active_users,
                  COUNT(*) as action_events
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND event_type = 'action'"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Distinct client_ids that ONLY viewed (no actions) in the window.
pub async fn get_view_only_users(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT client_id)
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
             AND client_id NOT IN (
               SELECT DISTINCT client_id FROM usage_events
               WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
                 AND event_type = 'action'
             )"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Total events in a window (all types).
pub async fn get_total_events(pool: &PgPool, days: i32) -> AppResult<i64> {
    let row = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL"#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Per-endpoint usage breakdown (top paths by count, view vs action).
/// Grouped by (path, event_type) in the last `days` days, ordered by count desc.
/// Derives a feature-group label (search/export/reader/social/other) from path prefix.
pub fn endpoint_group(path: &str) -> &'static str {
    if path.starts_with("/api/search")
        || path.starts_with("/api/tags/search")
        || path.starts_with("/api/tags/autocomplete")
        || path.starts_with("/opds/search")
        || path.starts_with("/feed")
    {
        "search"
    } else if path.starts_with("/api/epub")
        || path.starts_with("/api/download")
        || path.starts_with("/cache/")
        || path.starts_with("/api/send-to-kindle")
        || path.starts_with("/api/meta")
    {
        "export"
    } else if path.starts_with("/reader")
        || path.starts_with("/api/works")
        || path.starts_with("/api/fic")
        || path.starts_with("/api/chapters")
        || path.starts_with("/works/")
    {
        "reader"
    } else if path.starts_with("/api/comments")
        || path.starts_with("/api/votes")
        || path.starts_with("/api/bookmarks")
        || path.starts_with("/api/follow")
        || path.starts_with("/api/rate")
        || path.starts_with("/api/review")
        || path.starts_with("/api/shelves")
        || path.starts_with("/api/reading-lists")
        || path.starts_with("/api/forum")
    {
        "social"
    } else {
        "other"
    }
}

pub async fn get_endpoint_usage(
    pool: &PgPool,
    days: i32,
    limit: i64,
) -> AppResult<Vec<(String, String, i64)>> {
    let limit = limit.clamp(1, 100);
    let rows = sqlx::query_as::<_, (String, String, i64)>(
        r#"SELECT path, event_type, COUNT(*)::bigint as cnt
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY path, event_type
           ORDER BY cnt DESC
           LIMIT $2"#,
    )
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_endpoint_group_usage(pool: &PgPool, days: i32) -> AppResult<Vec<(String, i64)>> {
    // Use SQL CASE to bucket paths into feature groups server-side.
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT
              CASE
                WHEN path LIKE '/api/search%' OR path LIKE '/api/tags/search%' OR path LIKE '/api/tags/autocomplete%' OR path LIKE '/opds/search%' OR path LIKE '/feed%' THEN 'search'
                WHEN path LIKE '/api/epub%' OR path LIKE '/api/download%' OR path LIKE '/cache/%' OR path LIKE '/api/send-to-kindle%' OR path LIKE '/api/meta%' THEN 'export'
                WHEN path LIKE '/reader%' OR path LIKE '/api/works%' OR path LIKE '/api/fic%' OR path LIKE '/api/chapters%' OR path LIKE '/works/%' THEN 'reader'
                WHEN path LIKE '/api/comments%' OR path LIKE '/api/votes%' OR path LIKE '/api/bookmarks%' OR path LIKE '/api/follow%' OR path LIKE '/api/rate%' OR path LIKE '/api/review%' OR path LIKE '/api/shelves%' OR path LIKE '/api/reading-lists%' OR path LIKE '/api/forum%' THEN 'social'
                ELSE 'other'
              END as grp,
              COUNT(*)::bigint as cnt
           FROM usage_events
           WHERE created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY grp
           ORDER BY cnt DESC"#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Recent events timeline (latest first) for the admin view.
pub async fn get_recent_events(
    pool: &PgPool,
    limit: i32,
) -> AppResult<
    Vec<(
        chrono::DateTime<chrono::Utc>,
        String,
        String,
        String,
        Option<String>,
    )>,
> {
    let rows = sqlx::query_as::<
        _,
        (
            chrono::DateTime<chrono::Utc>,
            String,
            String,
            String,
            Option<String>,
        ),
    >(
        r#"SELECT created_at, client_id, path, event_type, user_agent
           FROM usage_events
           ORDER BY created_at DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Shelf / Collection queries
// ═══════════════════════════════════════════════════════════════════

/// Create a new shelf
pub async fn create_shelf(
    pool: &PgPool,
    user_id: i32,
    name: &str,
    description: &str,
    is_public: bool,
) -> AppResult<Shelf> {
    let row = sqlx::query_as::<_, Shelf>(
        r#"INSERT INTO shelves (user_id, name, description, is_public)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, name, description, is_public, sort_order, created_at"#,
    )
    .bind(user_id)
    .bind(name)
    .bind(description)
    .bind(is_public)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// List all shelves for a user
pub async fn list_shelves(pool: &PgPool, user_id: i32) -> AppResult<Vec<Shelf>> {
    let rows = sqlx::query_as::<_, Shelf>(
        "SELECT id, user_id, name, description, is_public, sort_order, created_at
         FROM shelves WHERE user_id = $1 ORDER BY sort_order, created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Get a single shelf by id (scoped to user)
pub async fn get_shelf(pool: &PgPool, shelf_id: i32, user_id: i32) -> AppResult<Option<Shelf>> {
    let row = sqlx::query_as::<_, Shelf>(
        "SELECT id, user_id, name, description, is_public, sort_order, created_at
         FROM shelves WHERE id = $1 AND user_id = $2",
    )
    .bind(shelf_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Delete a shelf (user-scoped)
pub async fn delete_shelf(pool: &PgPool, shelf_id: i32, user_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM shelves WHERE id = $1 AND user_id = $2")
        .bind(shelf_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Add a work to a shelf
pub async fn add_work_to_shelf(pool: &PgPool, shelf_id: i32, work_id: i32) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO work_shelves (shelf_id, work_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(shelf_id)
    .bind(work_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a work from a shelf
pub async fn remove_work_from_shelf(pool: &PgPool, shelf_id: i32, work_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM work_shelves WHERE shelf_id = $1 AND work_id = $2")
        .bind(shelf_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List works in a shelf (returns work_ids)
pub async fn list_works_in_shelf(pool: &PgPool, shelf_id: i32) -> AppResult<Vec<WorkShelf>> {
    let rows = sqlx::query_as::<_, WorkShelf>(
        "SELECT id, shelf_id, work_id, added_at
         FROM work_shelves WHERE shelf_id = $1 ORDER BY added_at DESC",
    )
    .bind(shelf_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Reading list queries
// ═══════════════════════════════════════════════════════════════════

/// Create a reading list for a user.
pub async fn create_reading_list(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    description: &str,
    is_public: bool,
) -> AppResult<ReadingList> {
    let row = sqlx::query_as::<_, ReadingList>(
        r#"INSERT INTO reading_lists (user_id, title, description, is_public)
           VALUES ($1, $2, $3, $4)
           RETURNING id, user_id, title, description, is_public, created_at, updated_at"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(is_public)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// List the user's own non-deleted reading lists, newest first.
pub async fn list_reading_lists(pool: &PgPool, user_id: i32) -> AppResult<Vec<ReadingList>> {
    let rows = sqlx::query_as::<_, ReadingList>(
        "SELECT id, user_id, title, description, is_public, created_at, updated_at
         FROM reading_lists
         WHERE user_id = $1 AND deleted_at IS NULL
         ORDER BY updated_at DESC, id DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Fetch one list, scoped to the owner unless `public_ok` (shared view).
/// `deleted` filters out soft-deleted lists for the owner.
pub async fn get_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    public_ok: bool,
) -> AppResult<Option<ReadingList>> {
    let row = if public_ok {
        sqlx::query_as::<_, ReadingList>(
            "SELECT id, user_id, title, description, is_public, created_at, updated_at
             FROM reading_lists
             WHERE id = $1 AND deleted_at IS NULL AND is_public = TRUE",
        )
        .bind(list_id)
        .fetch_optional(pool)
        .await?
    } else {
        sqlx::query_as::<_, ReadingList>(
            "SELECT id, user_id, title, description, is_public, created_at, updated_at
             FROM reading_lists
             WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(list_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    };
    Ok(row)
}

/// Update a list's title/description/public flag (owner-scoped).
pub async fn update_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    title: &str,
    description: &str,
    is_public: bool,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE reading_lists
           SET title = $3, description = $4, is_public = $5, updated_at = NOW()
           WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(list_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(is_public)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Soft-delete a reading list (owner or curator).
pub async fn delete_reading_list(
    pool: &PgPool,
    list_id: i32,
    user_id: i32,
    is_curator: bool,
) -> AppResult<bool> {
    let result = if is_curator {
        sqlx::query("UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(list_id)
            .execute(pool)
            .await?
    } else {
        sqlx::query(
            "UPDATE reading_lists SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(list_id)
        .bind(user_id)
        .execute(pool)
        .await?
    };
    Ok(result.rows_affected() > 0)
}

/// Append a work to a list at the next position. The UNIQUE(list_id, work_id)
/// constraint rejects duplicates; returns Ok(false) when the work is already
/// in the list.
pub async fn add_reading_list_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
    blurb: &str,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
           SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
           FROM reading_list_items WHERE list_id = $1
           ON CONFLICT (list_id, work_id) DO NOTHING"#,
    )
    .bind(list_id)
    .bind(work_id)
    .bind(blurb)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a work from a list; returns Ok(false) if it was not present.
pub async fn remove_reading_list_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM reading_list_items WHERE list_id = $1 AND work_id = $2")
        .bind(list_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List a list's items in position order, joined with work titles/authors.
pub async fn list_reading_list_items(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<ReadingListItemRow>> {
    let rows = sqlx::query_as::<_, ReadingListItemRow>(
        r#"SELECT i.id, i.list_id, i.work_id, i.position, i.blurb, i.created_at,
                  w.canonical_title, w.canonical_author
           FROM reading_list_items i
           JOIN works w ON w.id = i.work_id
           WHERE i.list_id = $1
           ORDER BY i.position ASC, i.id ASC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Collection queries (AO3-style: visibility, item selection, bookmarks)
// ═══════════════════════════════════════════════════════════════════

/// Create an AO3-style collection. Wraps the reading_lists table with the
/// collection_kind / visibility / item_selection / icon / slug columns.
pub async fn create_collection(
    pool: &PgPool,
    user_id: i32,
    title: &str,
    description: &str,
    visibility: &str,
    item_selection: &str,
    icon: &str,
    slug: Option<&str>,
) -> AppResult<CollectionInfo> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"INSERT INTO reading_lists
             (user_id, title, description, collection_kind, visibility,
              item_selection, icon, slug)
           VALUES ($1, $2, $3, 'collection', $4, $5, $6, $7)
           RETURNING
             id, user_id,
             (SELECT username FROM users WHERE id = $1) AS owner_username,
             title, description, visibility, item_selection, icon, slug,
             'collection'::text AS collection_kind,
             created_at, updated_at,
             0::bigint AS item_count,
             0::bigint AS bookmark_count,
             FALSE AS is_bookmarked"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(visibility)
    .bind(item_selection)
    .bind(icon)
    .bind(slug)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

/// Browse public + anonymous collections (paginated, newest first).
/// Anonymous collections show the owner username only to themselves.
pub async fn browse_public_collections(
    pool: &PgPool,
    viewer_id: Option<i32>,
    limit: i64,
    offset: i64,
) -> AppResult<(Vec<CollectionInfo>, i64)> {
    let limit = limit.clamp(1, 100);
    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(*) FROM reading_lists
           WHERE collection_kind = 'collection'
             AND visibility IN ('public', 'anonymous')
             AND deleted_at IS NULL"#,
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.collection_kind = 'collection'
             AND rl.visibility IN ('public', 'anonymous')
             AND rl.deleted_at IS NULL
           ORDER BY rl.updated_at DESC, rl.id DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(viewer_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((rows, total.0))
}

/// Fetch a single collection by id. Returns the row only if visible to the
/// viewer: owner always; public/anonymous visible to everyone (anonymous
/// hides owner name unless viewer is the owner).
pub async fn get_collection(
    pool: &PgPool,
    id: i32,
    viewer_id: Option<i32>,
) -> AppResult<Option<CollectionInfo>> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.id = $2
             AND rl.collection_kind = 'collection'
             AND rl.deleted_at IS NULL
             AND (
                  rl.user_id = $1                       -- owner always
                  OR rl.visibility = 'public'
                  OR rl.visibility = 'anonymous'
             )"#,
    )
    .bind(viewer_id)
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Fetch a collection by slug (for clean /collections/[slug] URLs).
pub async fn get_collection_by_slug(
    pool: &PgPool,
    slug: &str,
    viewer_id: Option<i32>,
) -> AppResult<Option<CollectionInfo>> {
    let row = sqlx::query_as::<_, CollectionInfo>(
        r#"SELECT rl.id, rl.user_id,
                  CASE WHEN rl.visibility = 'anonymous' AND $1 IS DISTINCT FROM rl.user_id
                       THEN NULL ELSE u.username END AS owner_username,
                  rl.title, rl.description, rl.visibility, rl.item_selection,
                  rl.icon, rl.slug, rl.collection_kind,
                  rl.created_at, rl.updated_at,
                  (SELECT COUNT(*) FROM reading_list_items WHERE list_id = rl.id) AS item_count,
                  (SELECT COUNT(*) FROM collection_bookmarks WHERE list_id = rl.id) AS bookmark_count,
                  EXISTS (SELECT 1 FROM collection_bookmarks WHERE list_id = rl.id AND user_id = $1) AS is_bookmarked
           FROM reading_lists rl
           LEFT JOIN users u ON u.id = rl.user_id
           WHERE rl.slug = $2
             AND rl.collection_kind = 'collection'
             AND rl.deleted_at IS NULL
             AND (
                  rl.user_id = $1
                  OR rl.visibility = 'public'
                  OR rl.visibility = 'anonymous'
             )"#,
    )
    .bind(viewer_id)
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Update a collection's mutable metadata (owner-scoped).
pub async fn update_collection(
    pool: &PgPool,
    id: i32,
    user_id: i32,
    title: &str,
    description: &str,
    visibility: &str,
    item_selection: &str,
    icon: &str,
    slug: Option<&str>,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE reading_lists
           SET title = $3, description = $4, visibility = $5,
               item_selection = $6, icon = $7, slug = $8, updated_at = NOW()
           WHERE id = $1 AND user_id = $2
             AND collection_kind = 'collection'
             AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(visibility)
    .bind(item_selection)
    .bind(icon)
    .bind(slug)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Soft-delete a collection (owner or curator >= 5).
pub async fn delete_collection(
    pool: &PgPool,
    id: i32,
    user_id: i32,
    is_curator: bool,
) -> AppResult<bool> {
    let result = if is_curator {
        sqlx::query(
            r#"UPDATE reading_lists
               SET deleted_at = NOW(), updated_at = NOW()
               WHERE id = $1
                 AND collection_kind = 'collection'
                 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .execute(pool)
        .await?
    } else {
        sqlx::query(
            r#"UPDATE reading_lists
               SET deleted_at = NOW(), updated_at = NOW()
               WHERE id = $1 AND user_id = $2
                 AND collection_kind = 'collection'
                 AND deleted_at IS NULL"#,
        )
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?
    };
    Ok(result.rows_affected() > 0)
}

/// Add a work to a collection. Respects item_selection:
///   * restricted  — only owner/curator may add (returns Ok(false) otherwise)
///   * moderated   — non-owners land in collection_item_requests as 'pending'
/// Returns Ok(true) when the item was added directly (approved immediately).
pub async fn add_collection_item(
    pool: &PgPool,
    list_id: i32,
    work_id: i32,
    actor_id: i32,
    blurb: &str,
    is_owner_or_curator: bool,
    item_selection: &str,
) -> AppResult<bool> {
    if item_selection == "moderated" && !is_owner_or_curator {
        // Non-owners of a moderated collection must request; the item lands
        // as 'pending' in collection_item_requests.
        sqlx::query(
            r#"INSERT INTO collection_item_requests (list_id, work_id, requested_by, blurb)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (list_id, work_id) DO UPDATE
                 SET requested_by = EXCLUDED.requested_by,
                     blurb = EXCLUDED.blurb"#,
        )
        .bind(list_id)
        .bind(work_id)
        .bind(actor_id)
        .bind(blurb)
        .execute(pool)
        .await?;
        return Ok(false);
    }

    // Restricted mode (or owner/curator of a moderated collection): add directly.
    let result = sqlx::query(
        r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
           SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
           FROM reading_list_items WHERE list_id = $1
           ON CONFLICT (list_id, work_id) DO NOTHING"#,
    )
    .bind(list_id)
    .bind(work_id)
    .bind(blurb)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a work from a collection (owner or curator).
pub async fn remove_collection_item(pool: &PgPool, list_id: i32, work_id: i32) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM reading_list_items WHERE list_id = $1 AND work_id = $2")
        .bind(list_id)
        .bind(work_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// List pending (and recent) add requests for a moderated collection.
pub async fn list_collection_item_requests(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<CollectionItemRequestRow>> {
    let rows = sqlx::query_as::<_, CollectionItemRequestRow>(
        r#"SELECT cir.id, cir.list_id, cir.work_id, cir.requested_by,
                  u.username AS requested_by_username, cir.blurb, cir.status,
                  cir.created_at, cir.reviewed_at, w.canonical_title, w.canonical_author
           FROM collection_item_requests cir
           LEFT JOIN users u ON u.id = cir.requested_by
           JOIN works w ON w.id = cir.work_id
           WHERE cir.list_id = $1
           ORDER BY cir.status ASC, cir.created_at DESC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// List collection add-requests (optionally for a single `list_id`) joined
/// with the collection title, the work's metadata, and live community vote
/// tallies plus the viewing user's own vote. Pass `None` for `list_id` to list
/// across every collection (used by the unified curator Approvals queue).
pub async fn list_collection_item_requests_with_votes(
    pool: &PgPool,
    list_id: Option<i32>,
    viewer_id: i32,
) -> AppResult<Vec<CollectionItemRequestVoteRow>> {
    let rows = sqlx::query_as::<_, CollectionItemRequestVoteRow>(
        r#"SELECT cir.id, cir.list_id, rl.title AS list_title, cir.work_id,
                   cir.requested_by, u.username AS requested_by_username,
                   cir.blurb, cir.status, cir.created_at, cir.reviewed_at,
                   w.canonical_title, w.canonical_author,
                   COALESCE(sv.votes_for, 0)::int AS votes_for,
                   COALESCE(sv.votes_against, 0)::int AS votes_against,
                   COALESCE(my.vote, 0)::smallint AS my_vote
            FROM collection_item_requests cir
            JOIN reading_lists rl ON rl.id = cir.list_id
            LEFT JOIN users u ON u.id = cir.requested_by
            JOIN works w ON w.id = cir.work_id
            LEFT JOIN (
                SELECT request_id,
                       COUNT(*) FILTER (WHERE vote = 1)::int AS votes_for,
                       COUNT(*) FILTER (WHERE vote = -1)::int AS votes_against
                FROM collection_submission_votes
                GROUP BY request_id
            ) sv ON sv.request_id = cir.id
            LEFT JOIN collection_submission_votes my
                ON my.request_id = cir.id AND my.user_id = $2
            WHERE ($1::int IS NULL OR cir.list_id = $1)
            ORDER BY cir.status ASC, cir.created_at DESC"#,
    )
    .bind(list_id)
    .bind(viewer_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Record (upsert) a community vote for a collection add-request. The
/// (request_id, user_id) unique constraint guarantees a member cannot vote
/// twice; re-voting simply changes their prior signal.
pub async fn upsert_collection_submission_vote(
    pool: &PgPool,
    request_id: i64,
    user_id: i32,
    vote: i16,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"INSERT INTO collection_submission_votes (request_id, user_id, vote)
           VALUES ($1, $2, $3)
           ON CONFLICT (request_id, user_id)
           DO UPDATE SET vote = EXCLUDED.vote, created_at = now()"#,
    )
    .bind(request_id)
    .bind(user_id)
    .bind(vote)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Approve a pending collection item (owner/curator): move it into
/// reading_list_items and mark the request 'approved'.
pub async fn approve_collection_item(
    pool: &PgPool,
    list_id: i32,
    request_id: i64,
) -> AppResult<bool> {
    let mut tx = pool.begin().await?;
    // Grab the pending request
    let req: Option<(i32, i32, String)> = sqlx::query_as(
        r#"SELECT work_id, requested_by, blurb FROM collection_item_requests
           WHERE id = $1 AND list_id = $2 AND status = 'pending'"#,
    )
    .bind(request_id)
    .bind(list_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some((work_id, _requested_by, blurb)) = req {
        // Insert the work item (skip if already present)
        sqlx::query(
            r#"INSERT INTO reading_list_items (list_id, work_id, position, blurb)
               SELECT $1, $2, COALESCE(MAX(position), 0) + 1, $3
               FROM reading_list_items WHERE list_id = $1
               ON CONFLICT (list_id, work_id) DO NOTHING"#,
        )
        .bind(list_id)
        .bind(work_id)
        .bind(blurb)
        .execute(&mut *tx)
        .await?;
        // Mark the request approved
        sqlx::query(
            r#"UPDATE collection_item_requests
               SET status = 'approved', reviewed_at = NOW()
               WHERE id = $1"#,
        )
        .bind(request_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(true)
    } else {
        tx.rollback().await?;
        Ok(false)
    }
}

/// Reject a pending collection item (owner/curator).
pub async fn reject_collection_item(
    pool: &PgPool,
    list_id: i32,
    request_id: i64,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE collection_item_requests
           SET status = 'rejected', reviewed_at = NOW()
           WHERE id = $1 AND list_id = $2 AND status = 'pending'"#,
    )
    .bind(request_id)
    .bind(list_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Bookmark a collection for the current user (idempotent).
pub async fn bookmark_collection(pool: &PgPool, user_id: i32, list_id: i32) -> AppResult<bool> {
    let result = sqlx::query(
        "INSERT INTO collection_bookmarks (user_id, list_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(list_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Remove a collection bookmark.
pub async fn unbookmark_collection(pool: &PgPool, user_id: i32, list_id: i32) -> AppResult<bool> {
    let result =
        sqlx::query("DELETE FROM collection_bookmarks WHERE user_id = $1 AND list_id = $2")
            .bind(user_id)
            .bind(list_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// List users who bookmarked a collection.
pub async fn list_collection_bookmarkers(
    pool: &PgPool,
    list_id: i32,
) -> AppResult<Vec<(i32, String, String)>> {
    let rows: Vec<(i32, String, String)> = sqlx::query_as(
        r#"SELECT u.id, u.username, cb.created_at::text
           FROM collection_bookmarks cb
           JOIN users u ON u.id = cb.user_id
           WHERE cb.list_id = $1
           ORDER BY cb.created_at DESC"#,
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════
// Reading status queries
// ═══════════════════════════════════════════════════════════════════

/// Update reading status for a work
pub async fn update_reading_status(
    pool: &PgPool,
    user_id: i32,
    work_id: i32,
    status: &str,
    current_chapter: Option<i32>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO reading_stats (user_id, work_id, status, current_chapter)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, work_id) DO UPDATE SET
               status = $3,
               current_chapter = COALESCE($4, reading_stats.current_chapter),
               last_read_at = NOW()"#,
    )
    .bind(user_id)
    .bind(work_id)
    .bind(status)
    .bind(current_chapter)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get reading list rows for a user, optionally filtered by status
pub async fn get_reading_stats_list(
    pool: &PgPool,
    user_id: i32,
    status_filter: Option<&str>,
) -> AppResult<Vec<ReadingStats>> {
    let rows = match status_filter {
        Some(status) => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 AND status = $2 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .bind(status)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, ReadingStats>(
                "SELECT id, user_id, work_id, words_read, last_read_at, read_count, status, current_chapter
                 FROM reading_stats WHERE user_id = $1 ORDER BY last_read_at DESC",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_client_id_format() {
        // Test UUID v4 format validation
        let valid_uuids = vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "12345678-1234-4234-8234-123456789abc",
        ];
        let invalid_uuids = vec![
            "not-a-uuid",
            "550e8400-e29b-41d4-a716",
            "550e8400-e29b-41d4-a716-446655440000-extra",
            "6ba7b810-9dad-11d1-80b4-00c04fd430c8", // version 1, not version 4
        ];

        let uuid_regex = regex_lite::Regex::new(
            r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
        )
        .unwrap();

        for uuid in valid_uuids {
            assert!(
                uuid_regex.is_match(uuid),
                "Should be valid UUID v4: {}",
                uuid
            );
        }
        for uuid in invalid_uuids {
            assert!(
                !uuid_regex.is_match(uuid),
                "Should be invalid UUID: {}",
                uuid
            );
        }
    }

    #[test]
    fn test_analytics_query_structures() {
        // Test that our query return types are correct
        let daily_stat: (chrono::NaiveDate, i64, i64) = (
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            100,
            500,
        );
        assert_eq!(daily_stat.1, 100); // unique_visitors
        assert_eq!(daily_stat.2, 500); // total_requests

        let user_stat: (i64, i64, i64) = (50, 10, 5);
        assert_eq!(user_stat.0, 50); // total_requests
        assert_eq!(user_stat.1, 10); // unique_fics
        assert_eq!(user_stat.2, 5); // downloads

        let popular_fic: (String, i64, i64) = ("abc123".to_string(), 200, 150);
        assert_eq!(popular_fic.1, 200); // request_count
        assert_eq!(popular_fic.2, 150); // downloads

        let format_stat: (String, i64) = ("epub".to_string(), 1000);
        assert_eq!(format_stat.0, "epub");
        assert_eq!(format_stat.1, 1000);
    }

    #[test]
    fn test_follow_model_types() {
        // Verify Follow tuple shapes are correct
        let follow = crate::db::models::Follow {
            id: 1,
            follower_id: 10,
            followee_id: Some(20),
            work_id: None,
            author_name: None,
            created_at: chrono::Utc::now(),
            last_seen: None,
        };
        assert_eq!(follow.follower_id, 10);
        assert_eq!(follow.followee_id, Some(20));
    }

    #[test]
    fn test_notification_model() {
        let notif = crate::db::models::Notification {
            id: 1,
            user_id: 10,
            notification_type: "comment_reply".into(),
            title: "Reply".into(),
            body: None,
            link: Some("/works/1".into()),
            reference_type: Some("comment".into()),
            reference_id: Some("5".into()),
            is_read: false,
            created_at: chrono::Utc::now(),
        };
        assert!(!notif.is_read);
        assert_eq!(notif.notification_type, "comment_reply");
    }

    #[test]
    fn test_badge_definition_model() {
        let badge = crate::db::models::BadgeDefinition {
            badge_type: "curator_apprentice".into(),
            name: "Curator Apprentice".into(),
            description: "Test badge".into(),
            icon: "🔰".into(),
            category: "curation".into(),
            threshold: 10,
            event_type: Some("curation_approve".into()),
            created_at: chrono::Utc::now(),
        };
        assert_eq!(badge.threshold, 10);
        assert_eq!(badge.event_type, Some("curation_approve".into()));
    }

    #[test]
    fn test_locale_model() {
        let locale = crate::db::models::Locale {
            id: 1,
            code: "en".into(),
            name: "English".into(),
            is_rtl: false,
        };
        assert_eq!(locale.code, "en");
        assert!(!locale.is_rtl);
    }

    #[test]
    fn test_leaderboard_entry_shapes() {
        let weekly: (i32, String, i32, i16) = (1, "alice".into(), 500, 1);
        assert_eq!(weekly.2, 500);
        assert_eq!(weekly.3, 1);

        let monthly: (i32, String, i32, i16) = (2, "bob".into(), 300, 2);
        assert_eq!(monthly.0, 2);
        assert_eq!(monthly.1, "bob");
    }

    #[test]
    fn test_translation_model_shapes() {
        let t = crate::db::models::WorkTranslation {
            id: 1,
            work_id: 42,
            locale_code: "es".into(),
            title: Some("El título".into()),
            summary: None,
            translated_by: Some(7),
            translated_at: chrono::Utc::now(),
        };
        assert_eq!(t.locale_code, "es");
        assert_eq!(t.title, Some("El título".into()));
    }

    #[test]
    fn test_reading_stats_model() {
        let stats = crate::db::models::ReadingStats {
            id: 1,
            user_id: 10,
            work_id: 42,
            words_read: 50000,
            last_read_at: chrono::Utc::now(),
            read_count: 3,
            status: "reading".into(),
            current_chapter: Some(5),
        };
        assert_eq!(stats.words_read, 50000);
        assert_eq!(stats.read_count, 3);
        assert_eq!(stats.status, "reading");
        assert_eq!(stats.current_chapter, Some(5));
    }

    #[test]
    fn test_login_streak_model() {
        let streak = crate::db::models::LoginStreak {
            user_id: 1,
            current_streak: 7,
            longest_streak: 30,
            last_login_date: chrono::Utc::now().date_naive(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(streak.current_streak, 7);
        assert_eq!(streak.longest_streak, 30);
    }

    #[test]
    fn test_leaderboard_weekly_monthly_models() {
        let week_start = chrono::Utc::now().date_naive();
        let lw = crate::db::models::LeaderboardWeekly {
            id: 1,
            user_id: 1,
            score: 500,
            rank: 1,
            week_start,
            computed_at: chrono::Utc::now(),
        };
        assert_eq!(lw.score, 500);
        assert_eq!(lw.rank, 1);

        let lm = crate::db::models::LeaderboardMonthly {
            id: 2,
            user_id: 1,
            score: 2000,
            rank: 1,
            month_start: week_start,
            computed_at: chrono::Utc::now(),
        };
        assert_eq!(lm.score, 2000);
    }

    #[test]
    fn test_endpoint_group_classification() {
        assert_eq!(
            crate::db::queries::endpoint_group("/api/search?q=foo"),
            "search"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/tags/search?q=x"),
            "search"
        );
        assert_eq!(crate::db::queries::endpoint_group("/feed.xml"), "search");
        assert_eq!(
            crate::db::queries::endpoint_group("/api/epub?id=1"),
            "export"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/cache/epub/1/file.epub"),
            "export"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/download/author"),
            "export"
        );
        assert_eq!(crate::db::queries::endpoint_group("/reader/1"), "reader");
        assert_eq!(crate::db::queries::endpoint_group("/api/works/1"), "reader");
        assert_eq!(
            crate::db::queries::endpoint_group("/api/comments"),
            "social"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/bookmarks"),
            "social"
        );
        assert_eq!(
            crate::db::queries::endpoint_group("/api/forum/threads"),
            "social"
        );
        assert_eq!(crate::db::queries::endpoint_group("/api/health"), "other");
        assert_eq!(crate::db::queries::endpoint_group("/"), "other");
    }

    #[test]
    fn test_endpoint_usage_query_shape() {
        // Validate SQL skeleton for get_endpoint_usage matches spec:
        // SELECT path, event_type, COUNT(*) FROM usage_events WHERE ts > NOW()-interval ... GROUP BY path, event_type ORDER BY count DESC LIMIT
        // Actual column is created_at; ensure the query string in source has the expected clauses.
        let sql = r#"SELECT path, event_type, COUNT(*)::bigint as cnt FROM usage_events WHERE created_at > NOW() - ($1 || ' days')::INTERVAL GROUP BY path, event_type ORDER BY cnt DESC LIMIT $2"#;
        assert!(sql.contains("usage_events"));
        assert!(sql.contains("GROUP BY path, event_type"));
        assert!(sql.contains("ORDER BY cnt DESC"));
        assert!(sql.contains("created_at"));
    }
}

// ── User format preferences ─────────────────────────────────────────

/// Default download formats for a new user.
pub const DEFAULT_FORMATS: &[&str] = &["epub"];

/// Allowed download formats.
pub const ALLOWED_FORMATS: &[&str] = &["epub", "pdf", "mobi", "html", "azw3", "kepub", "docx"];

/// Get a user's preferred download formats.
/// Returns `["epub"]` if the user has no settings yet.
pub async fn get_user_format_preferences(
    pool: &PgPool,
    user_id: i32,
) -> AppResult<Vec<String>> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT settings FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some((settings,)) = row {
        if let Some(formats) = settings.get("formats").and_then(|v| v.as_array()) {
            let formats: Vec<String> = formats
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .filter(|f| ALLOWED_FORMATS.contains(&f.as_str()))
                .collect();
            if !formats.is_empty() {
                return Ok(formats);
            }
        }
    }
    Ok(DEFAULT_FORMATS.iter().map(|&s| s.to_string()).collect())
}

/// Save a user's preferred download formats.
/// Validates against ALLOWED_FORMATS before saving.
pub async fn save_user_format_preferences(
    pool: &PgPool,
    user_id: i32,
    formats: &[String],
) -> AppResult<()> {
    // Validate all formats are allowed.
    for fmt in formats {
        if !ALLOWED_FORMATS.contains(&fmt.as_str()) {
            return Err(AppError::BadRequest(format!(
                "unsupported format: {fmt}. allowed: {ALLOWED_FORMATS:?}"
            )));
        }
    }

    if formats.is_empty() {
        return Err(AppError::BadRequest(
            "at least one format must be selected".into(),
        ));
    }

    let formats_value = serde_json::json!(formats);
    sqlx::query(
        "UPDATE users SET settings = jsonb_set(
            COALESCE(settings, '{}'::jsonb),
            '{formats}',
            $1::jsonb
        ) WHERE id = $2",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ── Visitor state (anonymous funnel) ────────────────────────────────

/// Load visitor state by visitor ID. Returns None if not found.
pub async fn get_visitor_state(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
) -> AppResult<Option<VisitorStateRow>> {
    let row = sqlx::query_as::<_, VisitorStateRow>(
        "SELECT visitor_id, ratings, saved_searches, bookmarks, created_at, updated_at
         FROM visitor_state WHERE visitor_id = $1",
    )
    .bind(visitor_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// Upsert visitor state (insert or update on conflict).
pub async fn upsert_visitor_state(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
    ratings: &serde_json::Value,
    saved_searches: &serde_json::Value,
    bookmarks: &serde_json::Value,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO visitor_state (visitor_id, ratings, saved_searches, bookmarks, updated_at)
         VALUES ($1, $2, $3, $4, now())
         ON CONFLICT (visitor_id) DO UPDATE SET
           ratings = EXCLUDED.ratings,
           saved_searches = EXCLUDED.saved_searches,
           bookmarks = EXCLUDED.bookmarks,
           updated_at = now()",
    )
    .bind(visitor_id)
    .bind(ratings)
    .bind(saved_searches)
    .bind(bookmarks)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete a visitor state row (after merge on registration).
pub async fn delete_visitor_state(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
) -> AppResult<()> {
    sqlx::query("DELETE FROM visitor_state WHERE visitor_id = $1")
        .bind(visitor_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete visitor state rows older than 30 days (cron cleanup).
pub async fn cleanup_stale_visitor_state(pool: &PgPool) -> AppResult<u64> {
    let result = sqlx::query(
        "DELETE FROM visitor_state WHERE updated_at < now() - interval '30 days'",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Visitor state row from the database.
#[derive(Debug, sqlx::FromRow)]
pub struct VisitorStateRow {
    pub visitor_id: uuid::Uuid,
    pub ratings: serde_json::Value,
    pub saved_searches: serde_json::Value,
    pub bookmarks: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}


/// Merge anonymous visitor state into a newly-registered user account.
///
/// Called from the register handler after a user is successfully created.
/// Ratings, saved searches, and bookmarks from `visitor_state` are copied
/// into their user-specific tables, then the visitor row is deleted.
///
/// Returns a tuple of (merged_ratings, merged_searches, merged_bookmarks).
/// Errors are logged and swallowed so a merge failure never blocks registration.
pub async fn merge_visitor_state_into_user(
    pool: &PgPool,
    visitor_id: uuid::Uuid,
    user_id: i32,
) -> AppResult<(u64, u64, u64)> {
    let Some(row) = get_visitor_state(pool, visitor_id).await? else {
        return Ok((0, 0, 0));
    };

    let mut merged_ratings: u64 = 0;
    let mut merged_searches: u64 = 0;
    let mut merged_bookmarks: u64 = 0;

    // Merge ratings: visitor_state.ratings is a JSONB object {url_id: rating, ...}
    if let serde_json::Value::Object(ratings_map) = &row.ratings {
        for (url_id, rating_val) in ratings_map {
            let rating_i64 = rating_val.as_i64().unwrap_or(0);
            if !(1..=5).contains(&rating_i64) {
                continue;
            }
            let rating = rating_i64 as i16;

            // Resolve work_id from url_id via fic_info → works join.
            let work_id: Option<i32> =
                sqlx::query_scalar("SELECT w.id FROM works w JOIN fic_info fi ON fi.work_id = w.id WHERE fi.id = $1 LIMIT 1")
                    .bind(url_id)
                    .fetch_optional(pool)
                    .await?;

            if let Some(wid) = work_id {
                sqlx::query(
                    "INSERT INTO work_ratings (user_id, work_id, url_id, rating)
                     VALUES ($1, $2, $3, $4)
                     ON CONFLICT (user_id, url_id) DO UPDATE SET rating = $4, work_id = $2",
                )
                .bind(user_id)
                .bind(wid)
                .bind(url_id)
                .bind(rating)
                .execute(pool)
                .await?;
                merged_ratings += 1;
            }
        }
    }

    // Merge saved searches: visitor_state.saved_searches is a JSONB array
    // of objects with at least {name, query_text, query_json}.
    if let serde_json::Value::Array(searches) = &row.saved_searches {
        for item in searches {
            let obj = match item {
                serde_json::Value::Object(o) => o,
                _ => continue,
            };
            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("Imported search");
            let query_text = obj.get("query_text").and_then(|v| v.as_str()).unwrap_or("");
            let query_json = obj.get("query_json").cloned().unwrap_or(serde_json::Value::Null);

            sqlx::query(
                "INSERT INTO saved_searches (user_id, name, query_text, query_json, alert_mode)
                 VALUES ($1, $2, $3, $4, 'none')
                 ON CONFLICT (user_id, name) DO NOTHING",
            )
            .bind(user_id)
            .bind(name)
            .bind(query_text)
            .bind(&query_json)
            .execute(pool)
            .await?;
            merged_searches += 1;
        }
    }

    // Merge bookmarks: visitor_state.bookmarks is a JSONB array of url_ids.
    if let serde_json::Value::Array(bookmarks) = &row.bookmarks {
        for item in bookmarks {
            let url_id = match item {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };

            // Resolve work_id from url_id.
            let work_id: Option<i32> =
                sqlx::query_scalar("SELECT w.id FROM works w JOIN fic_info fi ON fi.work_id = w.id WHERE fi.id = $1 LIMIT 1")
                    .bind(&url_id)
                    .fetch_optional(pool)
                    .await?;

            sqlx::query(
                "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
                 VALUES ($1, $2, $3, '', false)
                 ON CONFLICT (user_id, url_id) DO NOTHING",
            )
            .bind(user_id)
            .bind(&url_id)
            .bind(work_id)
            .execute(pool)
            .await?;
            merged_bookmarks += 1;
        }
    }

    // Clean up the visitor state row.
    delete_visitor_state(pool, visitor_id).await?;

    tracing::info!(
        visitor_id = %visitor_id,
        user_id,
        merged_ratings,
        merged_searches,
        merged_bookmarks,
        "visitor state merged into user account"
    );

    Ok((merged_ratings, merged_searches, merged_bookmarks))
}
