use crate::error::{AppError, AppResult};
use sqlx::PgPool;

/// Result of resolving a tag string to a canonical tag.
pub struct TagResolution {
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_type_id: i16,
    pub is_new: bool,
}

/// Resolve a tag string to a canonical tag ID.
///
/// 1. Look up the name in `tags` (case-sensitive COLLATE "C")
/// 2. If not found, look up in `tag_aliases`
/// 3. If still not found, create a new canonical tag
pub async fn resolve_tag(
    pool: &PgPool,
    tag_name: &str,
    tag_type_id: i16,
) -> AppResult<TagResolution> {
    // Step 1: Look up by exact name in tags table
    if let Some(row) = lookup_tag(pool, tag_name).await? {
        return Ok(TagResolution {
            tag_id: row.0,
            tag_name: row.1,
            tag_type_id: row.2,
            is_new: false,
        });
    }

    // Step 2: Look up in tag_aliases
    if let Some(canonical_id) = lookup_alias(pool, tag_name).await? {
        let tag = lookup_tag_by_id(pool, canonical_id).await?.ok_or_else(|| {
            AppError::Internal(format!(
                "alias '{}' points to non-existent tag id {}",
                tag_name, canonical_id
            ))
        })?;
        return Ok(TagResolution {
            tag_id: tag.0,
            tag_name: tag.1,
            tag_type_id: tag.2,
            is_new: false,
        });
    }

    // Step 3: Create a new canonical tag
    let new_id = create_tag(pool, tag_name, tag_type_id).await?;
    Ok(TagResolution {
        tag_id: new_id,
        tag_name: tag_name.to_string(),
        tag_type_id,
        is_new: true,
    })
}

/// Look up a tag by its exact name (COLLATE "C" case-sensitive).
async fn lookup_tag(pool: &PgPool, name: &str) -> Result<Option<(i32, String, i16)>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, tag_type_id FROM tags WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await
}

/// Look up a canonical tag id from an alias name.
async fn lookup_alias(pool: &PgPool, name: &str) -> Result<Option<i32>, sqlx::Error> {
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|r| r.0))
}

/// Look up a tag by its numeric id.
async fn lookup_tag_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<Option<(i32, String, i16)>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, tag_type_id FROM tags WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Create a new canonical tag and return its id.
async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> Result<i32, sqlx::Error> {
    let row: (i32,) =
        sqlx::query_as("INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id")
            .bind(name)
            .bind(tag_type_id)
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_resolution_struct_sizes() {
        // Verify the struct layout is reasonable
        assert_eq!(
            std::mem::size_of::<TagResolution>(),
            std::mem::size_of::<(i32, String, i16, bool)>()
        );
    }

    #[test]
    fn test_tag_resolution_new_tag() {
        let tag = TagResolution {
            tag_id: 1,
            tag_name: "Angst".into(),
            tag_type_id: 4,
            is_new: true,
        };
        assert_eq!(tag.tag_id, 1);
        assert_eq!(tag.tag_name, "Angst");
        assert_eq!(tag.tag_type_id, 4);
        assert!(tag.is_new);
    }

    #[test]
    fn test_tag_resolution_existing_tag() {
        let tag = TagResolution {
            tag_id: 42,
            tag_name: "Fluff".into(),
            tag_type_id: 4,
            is_new: false,
        };
        assert_eq!(tag.tag_id, 42);
        assert_eq!(tag.tag_name, "Fluff");
        assert_eq!(tag.tag_type_id, 4);
        assert!(!tag.is_new);
    }

    #[test]
    fn test_tag_resolution_different_types() {
        let character_tag = TagResolution {
            tag_id: 10,
            tag_name: "Character A".into(),
            tag_type_id: 1,
            is_new: false,
        };
        assert_eq!(character_tag.tag_type_id, 1);
        assert_eq!(character_tag.tag_name, "Character A");

        let rating_tag = TagResolution {
            tag_id: 20,
            tag_name: "Rating X".into(),
            tag_type_id: 7,
            is_new: true,
        };
        assert_eq!(rating_tag.tag_type_id, 7);
        assert_eq!(rating_tag.tag_name, "Rating X");
        assert!(rating_tag.is_new);
    }
}
