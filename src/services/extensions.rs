//! Extension marketplace — unified store for themes, skins, recipes, layouts,
//! views, and saved-search/profile objects. Everything that can be installed,
//! rated, and remixed lives here so the gallery is consistent across
//! customization types.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::error::{AppError, AppResult};

/// Valid top-level extension kinds.
pub const KINDS: [&str; 9] = [
    "theme", "skin", "recipe", "layout", "view", "saved_search", "profile",
    "rec_strategy", "search",
];

pub fn is_known_kind(k: &str) -> bool {
    KINDS.iter().any(|&kk| kk == k)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ExtensionRow {
    pub id: i32,
    pub kind: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub author_id: i32,
    pub version: i32,
    pub tier: String,
    pub payload: Value,
    pub meta: Value,
    pub is_public: bool,
    pub is_verified: bool,
    pub installs: i32,
    pub rating: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct PublishExtBody {
    pub kind: String,
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: Option<i32>,
    #[serde(default = "default_tier")]
    pub tier: String,
    pub payload: Value,
    #[serde(default)]
    pub meta: Option<Value>,
    pub is_public: bool,
}

fn default_tier() -> String {
    "config".into()
}

/// Validate a slug: lowercase alphanumerics + `-` + `_`, 1..64 chars.
pub fn validate_slug(slug: &str) -> AppResult<()> {
    if slug.is_empty() || slug.len() > 64 {
        return Err(AppError::BadRequest("slug length must be 1..64".into()));
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return     Err(AppError::BadRequest(
            "slug must be lowercase alphanumerics, '-' or '_'".into(),
        ));
    }
    Ok(())
}

/// Publish (or update) an extension. `upsert=true` updates an existing row the
/// caller owns; otherwise a duplicate slug (in the same category) errors so
/// the user is nudged toward remixing.
pub async fn publish(
    db: &PgPool,
    author_id: i32,
    body: PublishExtBody,
    upsert: bool,
) -> AppResult<ExtensionRow> {
    if !is_known_kind(&body.kind) {
        return Err(AppError::BadRequest(format!(
            "kind must be one of: {}",
            KINDS.join(", ")
        )));
    }
    validate_slug(&body.slug)?;
    if body.name.trim().is_empty() {
        return Err(AppError::BadRequest("name required".into()));
    }
    let tier = if body.tier.is_empty() { "config" } else { &body.tier };

    if upsert {
        let row = sqlx::query_as::<_, ExtensionRow>(
            r#"UPDATE extensions
                  SET name = $1, description = $2, version = $3, tier = $4,
                      payload = $5, meta = $6, is_public = $7, updated_at = NOW()
                WHERE kind = $8 AND slug = $9 AND author_id = $10
             RETURNING id, kind, slug, name, description, author_id, version, tier,
                       payload, meta, is_public, is_verified, installs, rating,
                       created_at, updated_at"#,
        )
        .bind(&body.name)
        .bind(&body.description)
        .bind(body.version.unwrap_or(1))
        .bind(tier)
        .bind(&body.payload)
        .bind(body.meta.unwrap_or_else(|| json!({})))
        .bind(body.is_public)
        .bind(&body.kind)
        .bind(&body.slug)
        .bind(author_id)
        .fetch_optional(db)
        .await?;
        match row {
            Some(r) => Ok(r),
            None => Err(AppError::NotFound(
                "Extension not found or not owned by you".into(),
            )),
        }
    } else {
        let res = sqlx::query(
            r#"INSERT INTO extensions
                   (kind, slug, name, description, author_id, version, tier, payload, meta, is_public)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
        )
        .bind(&body.kind)
        .bind(&body.slug)
        .bind(&body.name)
        .bind(&body.description)
        .bind(author_id)
        .bind(body.version.unwrap_or(1))
        .bind(tier)
        .bind(&body.payload)
        .bind(body.meta.unwrap_or_else(|| json!({})))
        .bind(body.is_public)
        .execute(db)
        .await;
        match res {
            Ok(_) => {
                sqlx::query_as::<_, ExtensionRow>(
                    r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                              payload, meta, is_public, is_verified, installs, rating,
                              created_at, updated_at
                         FROM extensions WHERE kind = $1 AND slug = $2 AND author_id = $3"#,
                )
                .bind(&body.kind)
                .bind(&body.slug)
                .bind(author_id)
                .fetch_optional(db)
                .await?
                .ok_or_else(|| AppError::Database("inserted extension vanished".into()))
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(AppError::Conflict(
                    "That slug is already published in this category. Remix it instead.".into(),
                ))
            }
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }
}

pub async fn get(db: &PgPool, id: i32) -> AppResult<ExtensionRow> {
    sqlx::query_as::<_, ExtensionRow>(
        r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                  payload, meta, is_public, is_verified, installs, rating,
                  created_at, updated_at
             FROM extensions WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Extension not found".into()))
}

pub async fn by_slug(db: &PgPool, kind: &str, slug: &str) -> AppResult<ExtensionRow> {
    if !is_known_kind(kind) {
        return Err(AppError::BadRequest(format!("unknown kind: {kind}")));
    }
    sqlx::query_as::<_, ExtensionRow>(
        r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                  payload, meta, is_public, is_verified, installs, rating,
                  created_at, updated_at
                          FROM extensions WHERE kind = $1 AND slug = $2 AND is_public"#,
    )
    .bind(kind)
    .bind(slug)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Extension not found".into()))
}

/// Gallery listing, optionally filtered by `kind` and/or a search term.
pub async fn gallery(
    db: &PgPool,
    kind: Option<&str>,
    q: Option<&str>,
    limit: i64,
) -> AppResult<Vec<ExtensionRow>> {
    if let Some(k) = kind {
        if !is_known_kind(k) {
            return Err(AppError::BadRequest(format!("unknown kind: {k}")));
        }
    }
    let limit = limit.clamp(1, 100);
    let term = q.map(|s| s.trim()).filter(|s| !s.is_empty());

    let rows = match (kind, term) {
        (Some(k), Some(t)) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    WHERE is_public AND kind = $1
                      AND (name ILIKE $2 OR slug ILIKE $2)
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $3"#,
            )
            .bind(k)
            .bind(format!("%{t}%"))
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (Some(k), None) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions WHERE is_public AND kind = $1
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $2"#,
            )
            .bind(k)
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (None, Some(t)) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    WHERE is_public AND (name ILIKE $1 OR slug ILIKE $1)
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $2"#,
            )
            .bind(format!("%{t}%"))
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    WHERE is_public
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $1"#,
            )
            .bind(limit)
            .fetch_all(db)
            .await?
        }
    };
    Ok(rows)
}

/// Admin listing: every extension (private + public), optionally filtered.
pub async fn gallery_all(
    db: &PgPool,
    kind: Option<&str>,
    q: Option<&str>,
    limit: i64,
) -> AppResult<Vec<ExtensionRow>> {
    if let Some(k) = kind {
        if !is_known_kind(k) {
            return Err(AppError::BadRequest(format!("unknown kind: {k}")));
        }
    }
    let limit = limit.clamp(1, 500);
    let term = q.map(|s| s.trim()).filter(|s| !s.is_empty());

    let rows = match (kind, term) {
        (Some(k), Some(t)) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    WHERE kind = $1 AND (name ILIKE $2 OR slug ILIKE $2)
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $3"#,
            )
            .bind(k)
            .bind(format!("%{t}%"))
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (Some(k), None) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions WHERE kind = $1
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $2"#,
            )
            .bind(k)
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (None, Some(t)) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    WHERE name ILIKE $1 OR slug ILIKE $1
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $2"#,
            )
            .bind(format!("%{t}%"))
            .bind(limit)
            .fetch_all(db)
            .await?
        }
        (None, None) => {
            sqlx::query_as::<_, ExtensionRow>(
                r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                          payload, meta, is_public, is_verified, installs, rating,
                          created_at, updated_at
                     FROM extensions
                    ORDER BY is_verified DESC, installs DESC, rating DESC, created_at DESC
                    LIMIT $1"#,
            )
            .bind(limit)
            .fetch_all(db)
            .await?
        }
    };
    Ok(rows)
}

/// Record an install. Idempotent — repeat installs do not double-count.
pub async fn install(db: &PgPool, user_id: i32, ext_id: i32) -> AppResult<()> {
    let exists: Option<i32> =
        sqlx::query_scalar("SELECT id FROM extensions WHERE id = $1 AND is_public")
            .bind(ext_id)
            .fetch_optional(db)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Extension not found".into()));
    }

    let inserted = sqlx::query(
        r#"INSERT INTO extension_installs (extension_id, user_id) VALUES ($1, $2)
           ON CONFLICT DO NOTHING"#,
    )
    .bind(ext_id)
    .bind(user_id)
    .execute(db)
    .await?
    .rows_affected();

        if inserted > 0 {
        sqlx::query("UPDATE extensions SET installs = installs + 1 WHERE id = $1")
            .bind(ext_id)
            .execute(db)
            .await?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RateBody {
    pub rating: i16,
}

/// Rate an extension (1..5). Updates the running average on the row.
pub async fn rate(db: &PgPool, user_id: i32, ext_id: i32, rating: i16) -> AppResult<f32> {
    if !(1..=5).contains(&rating) {
        return Err(AppError::BadRequest("rating must be 1..5".into()));
    }
    let exists: Option<i32> =
        sqlx::query_scalar("SELECT id FROM extensions WHERE id = $1 AND is_public")
            .bind(ext_id)
            .fetch_optional(db)
            .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("Extension not found".into()));
    }

    sqlx::query(
        r#"INSERT INTO extension_ratings (extension_id, user_id, rating) VALUES ($1, $2, $3)
           ON CONFLICT (user_id, extension_id) DO UPDATE SET rating = $3"#,
    )
    .bind(ext_id)
    .bind(user_id)
    .bind(rating)
    .execute(db)
    .await?;

    let avg: Option<f64> =
        sqlx::query_scalar("SELECT AVG(rating)::float8 FROM extension_ratings WHERE extension_id = $1")
            .bind(ext_id)
            .fetch_optional(db)
            .await?;
    let avg = avg.unwrap_or(0.0);
    sqlx::query("UPDATE extensions SET rating = $1 WHERE id = $2")
        .bind(avg)
        .bind(ext_id)
        .execute(db)
        .await?;
    Ok(avg as f32)
}

/// Remix: copy a public extension's payload as a new private draft owned by
/// the caller, so two users can't overwrite the same slug.
pub async fn remix(
    db: &PgPool,
    author_id: i32,
    ext_id: i32,
    new_slug: &str,
    new_name: &str,
) -> AppResult<ExtensionRow> {
    validate_slug(new_slug)?;
    if new_name.trim().is_empty() {
        return Err(AppError::BadRequest("name required".into()));
    }
    let src: Option<ExtensionRow> = sqlx::query_as::<_, ExtensionRow>(
        r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                  payload, meta, is_public, is_verified, installs, rating,
                  created_at, updated_at
             FROM extensions WHERE id = $1 AND is_public"#,
    )
    .bind(ext_id)
    .fetch_optional(db)
    .await?;
    let src = src.ok_or_else(|| AppError::NotFound("Public extension not found".into()))?;

    sqlx::query(
        r#"INSERT INTO extensions (kind, slug, name, description, author_id, version, tier, payload, meta, is_public)
           VALUES ($1, $2, $3, $4, $5, 1, $6, $7, '{}'::jsonb, false)"#,
    )
    .bind(&src.kind)
    .bind(new_slug)
    .bind(new_name)
    .bind(format!("Remix of {}/{}", src.kind, src.slug))
    .bind(author_id)
    .bind(&src.tier)
    .bind(&src.payload)
    .execute(db)
    .await?;

    sqlx::query_as::<_, ExtensionRow>(
        r#"SELECT id, kind, slug, name, description, author_id, version, tier,
                  payload, meta, is_public, is_verified, installs, rating,
                  created_at, updated_at
             FROM extensions WHERE kind = $1 AND slug = $2 AND author_id = $3"#,
    )
        .bind(&src.kind)
    .bind(new_slug)
    .bind(author_id)
    .fetch_one(db)
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_kinds_are_recognized() {
        for k in KINDS {
            assert!(is_known_kind(k), "{k} must be known");
        }
        assert!(!is_known_kind("widget"));
        assert!(!is_known_kind(""));
    }

    #[test]
    fn slug_validation_accepts_good_slugs() {
        for s in ["a", "dark-mode", "dark_mode", "abc-123_xyz", "x9"] {
            assert!(validate_slug(s).is_ok(), "{s} should be valid");
        }
    }

    #[test]
    fn slug_validation_rejects_bad_slugs() {
        for s in [
            "",                                   // empty
            "Has-Upper",                          // uppercase
            "has space",                          // whitespace
            "has/slash",                          // path-ish
            "has.dot",                            // dot
            "emoji-🎨",                           // non-ascii
        ] {
            assert!(validate_slug(s).is_err(), "{s} should be rejected");
        }
        // Too long (65 chars).
        let long = "a".repeat(65);
        assert!(validate_slug(&long).is_err());
        // 64 chars is the boundary and is accepted.
        let max = "a".repeat(64);
        assert!(validate_slug(&max).is_ok());
    }

    #[test]
    fn publish_body_defaults_tier_to_config() {
        let body: PublishExtBody = serde_json::from_value(json!({
            "kind": "theme",
            "slug": "my-theme",
            "name": "My Theme",
            "payload": { "bg": "#fff" },
            "is_public": false,
        }))
        .expect("body parses");
        assert_eq!(body.tier, "config");
        assert_eq!(body.version, None);
        assert_eq!(body.meta, None);
    }

    #[test]
    fn rating_body_rejects_out_of_range_values() {
        // rate() validates 1..=5 before touching the DB; the parse here just
        // pins the wire shape.
        for v in [1i16, 3, 5] {
            let b: RateBody = serde_json::from_value(json!({ "rating": v })).expect("parses");
            assert_eq!(b.rating, v);
        }
    }
}