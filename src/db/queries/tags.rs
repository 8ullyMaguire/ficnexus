use crate::error::AppResult;
use sqlx::PgPool;

// ── Tag-related queries (v3) ─────────────────────────────────────────────

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
