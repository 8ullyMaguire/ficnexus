//! Recipe service — CRUD, activation, gallery, and strategy-weight derivation
//! for the user-facing Recipe Builder (P1 extension platform).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Recipe {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub blend: serde_json::Value,
    pub filters: serde_json::Value,
    pub boost: serde_json::Value,
    pub curator_prior: f32,
    pub is_active: bool,
    pub is_public: bool,
    pub installs: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct RecipeService;

impl RecipeService {
    /// Create a new recipe for the given user.
    pub async fn create(
        pool: &PgPool,
        user_id: i32,
        name: &str,
        description: Option<&str>,
        blend: serde_json::Value,
        filters: serde_json::Value,
        boost: serde_json::Value,
    ) -> Result<i32, AppError> {
        let id: i32 = sqlx::query_scalar(
            r#"INSERT INTO user_recipes (user_id, name, description, blend, filters, boost)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(&blend)
        .bind(&filters)
        .bind(&boost)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create recipe: {e}")))?;

        Ok(id)
    }

    /// List all recipes owned by the user.
    pub async fn list_user(pool: &PgPool, user_id: i32) -> Result<Vec<Recipe>, AppError> {
        let rows: Vec<Recipe> = sqlx::query_as(
            "SELECT * FROM user_recipes WHERE user_id = $1 ORDER BY updated_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list recipes: {e}")))?;

        Ok(rows)
    }

    /// Get the user's currently active recipe (used by the rec engine).
    pub async fn get_active(pool: &PgPool, user_id: i32) -> Result<Option<Recipe>, AppError> {
        let row: Option<Recipe> =
            sqlx::query_as("SELECT * FROM user_recipes WHERE user_id = $1 AND is_active = true")
                .bind(user_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::Database(format!("Failed to get active recipe: {e}")))?;

        Ok(row)
    }

    /// Set a recipe as active (deactivates all others for the user first).
    pub async fn set_active(pool: &PgPool, user_id: i32, recipe_id: i32) -> Result<(), AppError> {
        // Verify the recipe belongs to this user
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_recipes WHERE id = $1 AND user_id = $2)",
        )
        .bind(recipe_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to verify recipe ownership: {e}")))?;

        if !owned {
            return Err(AppError::NotFound("Recipe not found".into()));
        }

        // Deactivate all then activate the chosen one
        sqlx::query("UPDATE user_recipes SET is_active = false WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to deactivate recipes: {e}")))?;

        sqlx::query("UPDATE user_recipes SET is_active = true WHERE id = $1 AND user_id = $2")
            .bind(recipe_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to activate recipe: {e}")))?;

        Ok(())
    }

    /// Delete a recipe owned by the user.
    pub async fn delete(pool: &PgPool, user_id: i32, recipe_id: i32) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM user_recipes WHERE id = $1 AND user_id = $2")
            .bind(recipe_id)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete recipe: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Recipe not found".into()));
        }

        Ok(())
    }

    /// Update a recipe owned by the user.
    pub async fn update(
        pool: &PgPool,
        user_id: i32,
        recipe_id: i32,
        name: Option<&str>,
        description: Option<Option<&str>>,
        blend: Option<serde_json::Value>,
        filters: Option<serde_json::Value>,
        boost: Option<serde_json::Value>,
        curator_prior: Option<f32>,
    ) -> Result<(), AppError> {
        // Verify ownership
        let owned: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_recipes WHERE id = $1 AND user_id = $2)",
        )
        .bind(recipe_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to verify recipe ownership: {e}")))?;

        if !owned {
            return Err(AppError::NotFound("Recipe not found".into()));
        }

        sqlx::query(
            r#"UPDATE user_recipes
               SET name = COALESCE($3, name),
                   description = $4,
                   blend = COALESCE($5, blend),
                   filters = COALESCE($6, filters),
                   boost = COALESCE($7, boost),
                   curator_prior = COALESCE($8, curator_prior),
                   updated_at = NOW()
               WHERE id = $1 AND user_id = $2"#,
        )
        .bind(recipe_id)
        .bind(user_id)
        .bind(name)
        .bind(description)
        .bind(blend)
        .bind(filters)
        .bind(boost)
        .bind(curator_prior)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update recipe: {e}")))?;

        Ok(())
    }

    /// Publish/unpublish a recipe (make it visible in the gallery).
    pub async fn set_public(
        pool: &PgPool,
        user_id: i32,
        recipe_id: i32,
        is_public: bool,
    ) -> Result<(), AppError> {
        let result = sqlx::query(
            "UPDATE user_recipes SET is_public = $3, updated_at = NOW() WHERE id = $1 AND user_id = $2",
        )
        .bind(recipe_id)
        .bind(user_id)
        .bind(is_public)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to publish recipe: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Recipe not found".into()));
        }

        Ok(())
    }

    /// Browse public recipes from the gallery.
    pub async fn browse_public(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Recipe>, AppError> {
        let rows: Vec<Recipe> = sqlx::query_as(
            r#"SELECT r.* FROM user_recipes r
               WHERE r.is_public = true
               ORDER BY r.installs DESC, r.updated_at DESC
               LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to browse public recipes: {e}")))?;

        Ok(rows)
    }

    /// Install (copy) a public recipe into the user's own collection.
    pub async fn install(pool: &PgPool, user_id: i32, recipe_id: i32) -> Result<i32, AppError> {
        // Fetch the public recipe
        let source: Recipe =
            sqlx::query_as("SELECT * FROM user_recipes WHERE id = $1 AND is_public = true")
                .bind(recipe_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::Database(format!("Failed to fetch public recipe: {e}")))?
                .ok_or_else(|| AppError::NotFound("Public recipe not found".into()))?;

        // Copy it into the user's collection
        let new_id: i32 = sqlx::query_scalar(
            r#"INSERT INTO user_recipes
               (user_id, name, description, blend, filters, boost, curator_prior)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id"#,
        )
        .bind(user_id)
        .bind(format!("{} (copy)", source.name))
        .bind(source.description)
        .bind(&source.blend)
        .bind(&source.filters)
        .bind(&source.boost)
        .bind(source.curator_prior)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to install recipe: {e}")))?;

        // Increment install count on the source
        sqlx::query("UPDATE user_recipes SET installs = installs + 1 WHERE id = $1")
            .bind(recipe_id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to increment installs: {e}")))?;

        Ok(new_id)
    }

    /// Convert a recipe's `blend` JSON into a strategy-weights string
    /// compatible with the rec engine (e.g. "cooccur:0.3,tag_graph:0.4").
    ///
    /// Falls back to `config::Config::rec_strategies` default when blend
    /// is empty or has no recognized strategy names.
    pub fn to_strategy_weights(recipe: &Recipe) -> String {
        if let Some(obj) = recipe.blend.as_object() {
            if !obj.is_empty() {
                let parts: Vec<String> = obj
                    .iter()
                    .filter(|(_, v)| v.as_f64().map_or(false, |w| w > 0.0))
                    .map(|(k, v)| {
                        let w = v.as_f64().unwrap_or(1.0);
                        format!("{k}:{w}")
                    })
                    .collect();
                if !parts.is_empty() {
                    return parts.join(",");
                }
            }
        }
        // Fallback: return empty string so caller uses default
        String::new()
    }
}
