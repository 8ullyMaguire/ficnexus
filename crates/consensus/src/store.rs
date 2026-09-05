//! Storage traits and implementations for the consensus engine.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::{Feature, Result};

/// Error types for storage operations.
#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlx error: {0}")]
    #[cfg(feature = "sqlx-store")]
    Sqlx(#[from] sqlx::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("other: {0}")]
    Other(String),
}

/// Trait for persisting and querying Elo ratings and feature metadata.
#[async_trait]
pub trait RatingStore: Send + Sync {
    /// List all features, optionally filtered by status and/or category.
    /// Results must be sorted by Elo DESC.
    async fn list_features(
        &self,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<Feature>>;

    /// List features eligible for arena (matching given status IDs).
    async fn list_arena_candidates(
        &self,
        stage_ids: &[String],
        limit: usize,
    ) -> Result<Vec<Feature>>;

    /// Get current ratings for a set of cluster IDs.
    async fn get_ratings(&self, cluster_ids: &[i32]) -> Result<HashMap<i32, f64>>;

    /// Update a single cluster's Elo rating.
    async fn update_rating(&self, cluster_id: i32, new_elo: f64) -> Result<()>;

    /// Increment match counters (played, best, worst).
    async fn increment_counters(
        &self,
        cluster_id: i32,
        is_best: bool,
        is_worst: bool,
    ) -> Result<()>;
}

// ─── SQLx-backed store ──────────────────────────────────────────────

#[cfg(feature = "sqlx-store")]
const SQL_FEATURES_ALL: &str = r#"
    SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
           c.times_picked_best, c.times_picked_worst, c.status, c.category,
           COUNT(s.id)::bigint AS suggestions
    FROM feature_clusters c
    LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
    GROUP BY c.id
    ORDER BY c.elo_rating DESC
    LIMIT 200
"#;

#[cfg(feature = "sqlx-store")]
const SQL_FEATURES_BY_STATUS: &str = r#"
    SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
           c.times_picked_best, c.times_picked_worst, c.status, c.category,
           COUNT(s.id)::bigint AS suggestions
    FROM feature_clusters c
    LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
    WHERE c.status = $1
    GROUP BY c.id
    ORDER BY c.elo_rating DESC
    LIMIT 200
"#;

#[cfg(feature = "sqlx-store")]
const SQL_FEATURES_BY_CATEGORY: &str = r#"
    SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
           c.times_picked_best, c.times_picked_worst, c.status, c.category,
           COUNT(s.id)::bigint AS suggestions
    FROM feature_clusters c
    LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
    WHERE c.category = $1
    GROUP BY c.id
    ORDER BY c.elo_rating DESC
    LIMIT 200
"#;

#[cfg(feature = "sqlx-store")]
const SQL_FEATURES_BY_STATUS_AND_CATEGORY: &str = r#"
    SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
           c.times_picked_best, c.times_picked_worst, c.status, c.category,
           COUNT(s.id)::bigint AS suggestions
    FROM feature_clusters c
    LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
    WHERE c.status = $1 AND c.category = $2
    GROUP BY c.id
    ORDER BY c.elo_rating DESC
    LIMIT 200
"#;

#[cfg(feature = "sqlx-store")]
const SQL_GET_RATING: &str = "SELECT elo_rating::float8 FROM feature_clusters WHERE id = $1";

#[cfg(feature = "sqlx-store")]
const SQL_UPDATE_RATING: &str = "UPDATE feature_clusters SET elo_rating = $1 WHERE id = $2";

#[cfg(feature = "sqlx-store")]
const SQL_INCREMENT_PLAYED: &str =
    "UPDATE feature_clusters SET matches_played = matches_played + 1 WHERE id = $1";

#[cfg(feature = "sqlx-store")]
const SQL_INCREMENT_BEST: &str =
    "UPDATE feature_clusters SET times_picked_best = times_picked_best + 1 WHERE id = $1";

#[cfg(feature = "sqlx-store")]
const SQL_INCREMENT_WORST: &str =
    "UPDATE feature_clusters SET times_picked_worst = times_picked_worst + 1 WHERE id = $1";

/// SQLx-backed store implementation (PostgreSQL).
#[cfg(feature = "sqlx-store")]
#[derive(Clone)]
pub struct SqlxStore {
    pool: sqlx::PgPool,
}

#[cfg(feature = "sqlx-store")]
impl SqlxStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "sqlx-store")]
#[async_trait]
impl RatingStore for SqlxStore {
    async fn list_features(
        &self,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<Feature>> {
        let rows = match (status, category) {
            (Some(s), Some(c)) => sqlx::query_as::<_, Feature>(SQL_FEATURES_BY_STATUS_AND_CATEGORY)
                .bind(s)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(StoreError::from)?,
            (Some(s), None) => sqlx::query_as::<_, Feature>(SQL_FEATURES_BY_STATUS)
                .bind(s)
                .fetch_all(&self.pool)
                .await
                .map_err(StoreError::from)?,
            (None, Some(c)) => sqlx::query_as::<_, Feature>(SQL_FEATURES_BY_CATEGORY)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(StoreError::from)?,
            (None, None) => sqlx::query_as::<_, Feature>(SQL_FEATURES_ALL)
                .fetch_all(&self.pool)
                .await
                .map_err(StoreError::from)?,
        };
        Ok(rows)
    }

    async fn list_arena_candidates(
        &self,
        stage_ids: &[String],
        limit: usize,
    ) -> Result<Vec<Feature>> {
        if stage_ids.is_empty() {
            return Ok(vec![]);
        }
        // Build placeholders for IN clause. stage_ids come from ConsensusConfig
        // (validated enum values), so SQL injection is not a concern — but we
        // still use bind params for the IN values.
        let placeholders: Vec<String> = (1..=stage_ids.len()).map(|i| format!("${i}")).collect();
        let in_clause = placeholders.join(", ");

        let sql = format!(
            r#"
            SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
                   c.times_picked_best, c.times_picked_worst, c.status, c.category,
                   COUNT(s.id)::bigint AS suggestions
            FROM feature_clusters c
            LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
            WHERE c.status IN ({in_clause})
            GROUP BY c.id
            ORDER BY c.elo_rating DESC
            LIMIT {limit}
            "#
        );

        let sql_str: &'static str = sql.leak(); // small, bounded strings; accept the leak
        let mut q = sqlx::query_as::<_, Feature>(sql_str);
        for sid in stage_ids {
            q = q.bind(sid);
        }
        let rows = q.fetch_all(&self.pool).await.map_err(StoreError::from)?;
        Ok(rows)
    }

    async fn get_ratings(&self, cluster_ids: &[i32]) -> Result<HashMap<i32, f64>> {
        if cluster_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Query each cluster's rating individually (arena = 4 clusters, cheap).
        let mut map = HashMap::new();
        for id in cluster_ids {
            let rating: f64 = sqlx::query_scalar(SQL_GET_RATING)
                .bind(id)
                .fetch_one(&self.pool)
                .await
                .map_err(StoreError::from)?;
            map.insert(*id, rating);
        }
        Ok(map)
    }

    async fn update_rating(&self, cluster_id: i32, new_elo: f64) -> Result<()> {
        sqlx::query(SQL_UPDATE_RATING)
            .bind(new_elo)
            .bind(cluster_id)
            .execute(&self.pool)
            .await
            .map_err(StoreError::from)?;
        Ok(())
    }

    async fn increment_counters(
        &self,
        cluster_id: i32,
        is_best: bool,
        is_worst: bool,
    ) -> Result<()> {
        if is_best {
            sqlx::query(SQL_INCREMENT_BEST)
                .bind(cluster_id)
                .execute(&self.pool)
                .await
                .map_err(StoreError::from)?;
        }
        if is_worst {
            sqlx::query(SQL_INCREMENT_WORST)
                .bind(cluster_id)
                .execute(&self.pool)
                .await
                .map_err(StoreError::from)?;
        }
        sqlx::query(SQL_INCREMENT_PLAYED)
            .bind(cluster_id)
            .execute(&self.pool)
            .await
            .map_err(StoreError::from)?;
        Ok(())
    }
}

// ─── In-memory store ────────────────────────────────────────────────

/// In-memory store for testing and standalone usage (no DB).
#[derive(Clone, Default)]
pub struct MemoryStore {
    features: Arc<tokio::sync::RwLock<HashMap<i32, Feature>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_features(features: Vec<Feature>) -> Self {
        let map: HashMap<i32, Feature> = features.into_iter().map(|f| (f.id, f)).collect();
        Self {
            features: Arc::new(tokio::sync::RwLock::new(map)),
        }
    }
}

#[async_trait]
impl RatingStore for MemoryStore {
    async fn list_features(
        &self,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<Feature>> {
        let features = self.features.read().await;
        let mut result: Vec<Feature> = features
            .values()
            .filter(|f| status.is_none_or(|s| f.status == s))
            .filter(|f| category.is_none_or(|c| f.category == c))
            .cloned()
            .collect();
        result.sort_by(|a, b| b.elo_rating.partial_cmp(&a.elo_rating).unwrap());
        Ok(result)
    }

    async fn list_arena_candidates(
        &self,
        stage_ids: &[String],
        limit: usize,
    ) -> Result<Vec<Feature>> {
        let features = self.features.read().await;
        let mut result: Vec<Feature> = features
            .values()
            .filter(|f| stage_ids.contains(&f.status))
            .cloned()
            .collect();
        result.sort_by(|a, b| b.elo_rating.partial_cmp(&a.elo_rating).unwrap());
        result.truncate(limit);
        Ok(result)
    }

    async fn get_ratings(&self, cluster_ids: &[i32]) -> Result<HashMap<i32, f64>> {
        let features = self.features.read().await;
        let mut map = HashMap::new();
        for id in cluster_ids {
            if let Some(f) = features.get(id) {
                map.insert(*id, f.elo_rating);
            }
        }
        Ok(map)
    }

    async fn update_rating(&self, cluster_id: i32, new_elo: f64) -> Result<()> {
        let mut features = self.features.write().await;
        if let Some(f) = features.get_mut(&cluster_id) {
            f.elo_rating = new_elo;
            Ok(())
        } else {
            Err(StoreError::NotFound(format!("cluster {cluster_id}")).into())
        }
    }

    async fn increment_counters(
        &self,
        cluster_id: i32,
        is_best: bool,
        is_worst: bool,
    ) -> Result<()> {
        let mut features = self.features.write().await;
        if let Some(f) = features.get_mut(&cluster_id) {
            f.matches_played += 1;
            if is_best {
                f.times_picked_best += 1;
            }
            if is_worst {
                f.times_picked_worst += 1;
            }
            Ok(())
        } else {
            Err(StoreError::NotFound(format!("cluster {cluster_id}")).into())
        }
    }
}
