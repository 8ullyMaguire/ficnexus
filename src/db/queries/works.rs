use crate::db::models::*;
use crate::error::AppResult;
use sqlx::PgPool;

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
pub async fn get_work_slug_target(pool: &PgPool, url_id: &str) -> AppResult<Option<(i32, String)>> {
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
