use crate::db::models::*;
use super::SiteCredRow;
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;


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