use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

/// A resolved tag — always points to the canonical version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTag {
    pub input: String,
    pub canonical_id: i32,
    pub canonical_name: String,
    pub tag_type_id: i16,
    pub tag_type_name: String,
    pub matched_as: String, // "exact", "synonym", "fuzzy"
    pub is_canonical: bool,
    pub synonyms: Vec<String>,
    pub usage_count: i64,
}

/// Resolve a tag query to its canonical version.
///
/// 1. Try exact match on canonical tag name
/// 2. Try alias lookup
/// 3. Try fuzzy match (ILIKE)
pub async fn resolve_tag(pool: &PgPool, query: &str) -> Result<Option<ResolvedTag>, sqlx::Error> {
    // 1. Try exact match on canonical tag name
    if let Some(tag) = get_canonical_tag_by_name(pool, query).await? {
        let synonyms = get_synonyms(pool, tag.id).await?;
        let usage = get_usage_count(pool, tag.id).await?;
        return Ok(Some(ResolvedTag {
            input: query.to_string(),
            canonical_id: tag.id,
            canonical_name: tag.name,
            tag_type_id: tag.tag_type_id,
            tag_type_name: get_type_name(tag.tag_type_id).to_string(),
            matched_as: "exact".into(),
            is_canonical: true,
            synonyms,
            usage_count: usage,
        }));
    }

    // 2. Try alias lookup
    if let Some(canonical_id) = resolve_alias(pool, query).await? {
        if let Some(tag) = get_tag_by_id(pool, canonical_id).await? {
            let synonyms = get_synonyms(pool, tag.id).await?;
            let usage = get_usage_count(pool, tag.id).await?;
            return Ok(Some(ResolvedTag {
                input: query.to_string(),
                canonical_id: tag.id,
                canonical_name: tag.name,
                tag_type_id: tag.tag_type_id,
                tag_type_name: get_type_name(tag.tag_type_id).to_string(),
                matched_as: "synonym".into(),
                is_canonical: false,
                synonyms,
                usage_count: usage,
            }));
        }
    }

    // 3. Try fuzzy match (ILIKE) — only match tags that are canonical or have aliases
    let fuzzy = sqlx::query_as::<_, TagRow>(
        r#"SELECT t.id, t.name, t.tag_type_id
           FROM tags t
           WHERE t.name ILIKE $1
             AND (t.id IN (SELECT canonical_tag_id FROM tag_aliases)
                  OR t.id NOT IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name))
           LIMIT 1"#,
    )
    .bind(format!("%{}%", query))
    .fetch_optional(pool)
    .await?;

    if let Some(tag) = fuzzy {
        let synonyms = get_synonyms(pool, tag.id).await?;
        let usage = get_usage_count(pool, tag.id).await?;
        return Ok(Some(ResolvedTag {
            input: query.to_string(),
            canonical_id: tag.id,
            canonical_name: tag.name,
            tag_type_id: tag.tag_type_id,
            tag_type_name: get_type_name(tag.tag_type_id).to_string(),
            matched_as: "fuzzy".into(),
            is_canonical: true,
            synonyms,
            usage_count: usage,
        }));
    }

    Ok(None)
}

/// Resolve multiple tags at once, returning map of input → resolved
pub async fn resolve_tags_batch(
    pool: &PgPool,
    queries: &[String],
) -> Result<HashMap<String, ResolvedTag>, sqlx::Error> {
    let mut results = HashMap::new();
    for q in queries {
        if let Some(resolved) = resolve_tag(pool, q).await? {
            results.insert(q.clone(), resolved);
        }
    }
    Ok(results)
}

/// Get all synonym names for a canonical tag (from tag_aliases)
async fn get_synonyms(pool: &PgPool, tag_id: i32) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT alias_name FROM tag_aliases WHERE canonical_tag_id = $1",
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Get public usage count for a tag (number of fics tagged with it)
async fn get_usage_count(pool: &PgPool, tag_id: i32) -> Result<i64, sqlx::Error> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = $1",
    )
    .bind(tag_id)
    .fetch_one(pool)
    .await?;
    Ok(count.0)
}

/// Get all tag IDs that should match when a user selects a tag
/// (canonical tag + any tags that have this tag as their alias target)
pub async fn get_expanded_tag_ids(pool: &PgPool, tag_id: i32) -> Result<Vec<i32>, sqlx::Error> {
    let mut ids = vec![tag_id];
    // Also include any tags whose name matches an alias pointing to this canonical tag
    let synonym_rows: Vec<(i32,)> = sqlx::query_as(
        r#"SELECT t.id FROM tags t
           WHERE t.name IN (SELECT alias_name FROM tag_aliases WHERE canonical_tag_id = $1)"#,
    )
    .bind(tag_id)
    .fetch_all(pool)
    .await?;
    ids.extend(synonym_rows.into_iter().map(|r| r.0));
    Ok(ids)
}

/// Given a tag name and type_id, return all tag IDs that should be matched
/// (the canonical tag itself, plus any synonym tags via aliases).
///
/// This is the core function for synonym-aware search filtering.
pub async fn get_matching_tag_ids(
    pool: &PgPool,
    name: &str,
    type_id: i16,
) -> Result<Vec<i32>, sqlx::Error> {
    // type_id 0 means "any type" (used by excluded terms from -term / NOT term).
    // Resolve the canonical tag by name regardless of type in that case.
    if type_id == 0 {
        return get_matching_tag_ids_any_type(pool, name).await;
    }
    let ids: Vec<i32> = sqlx::query_scalar(
        r#"WITH canonical AS (
            SELECT id FROM tags WHERE LOWER(name) = LOWER($1) AND tag_type_id = $2
        )
        SELECT id FROM tags WHERE id IN (
            SELECT id FROM canonical
            UNION
            SELECT id FROM tags WHERE LOWER(name) IN (
                SELECT LOWER(alias_name) FROM tag_aliases WHERE canonical_tag_id IN (SELECT id FROM canonical)
            )
            UNION
            SELECT canonical_tag_id FROM tag_aliases WHERE LOWER(alias_name) = LOWER($1)
        )
        AND tag_type_id = $2"#,
    )
    .bind(name)
    .bind(type_id)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

/// Resolve a tag name to matching IDs across all tag types (type_id 0 / any).
/// Used for excluded terms from `-term` / `NOT term` so exclusions work on
/// tags of any type (fandom, character, freeform, warning, etc.).
async fn get_matching_tag_ids_any_type(
    pool: &PgPool,
    name: &str,
) -> Result<Vec<i32>, sqlx::Error> {
    let ids: Vec<i32> = sqlx::query_scalar(
        r#"WITH canonical AS (
            SELECT id FROM tags WHERE LOWER(name) = LOWER($1)
        )
        SELECT id FROM tags WHERE id IN (
            SELECT id FROM canonical
            UNION
            SELECT id FROM tags WHERE LOWER(name) IN (
                SELECT LOWER(alias_name) FROM tag_aliases WHERE canonical_tag_id IN (SELECT id FROM canonical)
            )
            UNION
            SELECT canonical_tag_id FROM tag_aliases WHERE LOWER(alias_name) = LOWER($1)
        )"#,
    )
    .bind(name)
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

// ---- Helper structs and functions ----

#[derive(Debug, Clone, sqlx::FromRow)]
struct TagRow {
    id: i32,
    name: String,
    tag_type_id: i16,
}

async fn get_canonical_tag_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        "SELECT id, name, tag_type_id, description FROM tags WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

async fn resolve_alias(pool: &PgPool, alias: &str) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar("SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = $1")
        .bind(alias)
        .fetch_optional(pool)
        .await
}

async fn get_tag_by_id(pool: &PgPool, id: i32) -> Result<Option<TagRow>, sqlx::Error> {
    sqlx::query_as::<_, TagRow>(
        "SELECT id, name, tag_type_id, description FROM tags WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Map tag type_id to a human-readable name
pub fn get_type_name(type_id: i16) -> &'static str {
    match type_id {
        1 => "fandom",
        2 => "character",
        3 => "relationship",
        4 => "freeform",
        5 => "warning",
        6 => "category",
        7 => "other",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_type_name_known() {
        assert_eq!(get_type_name(1), "fandom");
        assert_eq!(get_type_name(2), "character");
        assert_eq!(get_type_name(3), "relationship");
        assert_eq!(get_type_name(4), "freeform");
        assert_eq!(get_type_name(5), "warning");
        assert_eq!(get_type_name(6), "category");
        assert_eq!(get_type_name(7), "other");
    }

    #[test]
    fn test_get_type_name_unknown() {
        assert_eq!(get_type_name(0), "unknown");
        assert_eq!(get_type_name(99), "unknown");
    }

    #[test]
    fn test_resolved_tag_serialization() {
        let tag = ResolvedTag {
            input: "angst".into(),
            canonical_id: 42,
            canonical_name: "Angst".into(),
            tag_type_id: 4,
            tag_type_name: "freeform".into(),
            matched_as: "exact".into(),
            is_canonical: true,
            synonyms: vec!["Angst (Light)".into(), "Implied Angst".into()],
            usage_count: 1500,
        };
        let json = serde_json::to_value(&tag).unwrap();
        assert_eq!(json["input"], "angst");
        assert_eq!(json["canonical_name"], "Angst");
        assert_eq!(json["matched_as"], "exact");
        assert_eq!(json["usage_count"], 1500);
        assert_eq!(json["synonyms"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_resolved_tag_synonym_variant() {
        let tag = ResolvedTag {
            input: "Cafe".into(),
            canonical_id: 99,
            canonical_name: "Coffee Shops".into(),
            tag_type_id: 4,
            tag_type_name: "freeform".into(),
            matched_as: "synonym".into(),
            is_canonical: false,
            synonyms: vec!["Coffee Shops".into(), "Cafe".into()],
            usage_count: 500,
        };
        assert!(!tag.is_canonical);
        assert_eq!(tag.matched_as, "synonym");
    }
}
