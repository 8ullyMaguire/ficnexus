//! The legacy engine as a strategy — Stage 0 golden-test guarantee.
//!
//! `REC_ENGINE_MODE=legacy` (default) routes requests through the original
//! code path exactly as before; nothing here changes. In `pluggable` mode
//! this strategy is registered as `cooccur` and its `score()` calls the SAME
//! functions the legacy handlers call, so a registry containing only
//! `cooccur` produces identical output to the legacy mode (the golden test
//! in `tests/rec_strategies.rs` asserts this for a seeded user).

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};
use crate::error::AppError;

/// Wraps `RecommendationEngine` / the shared personal-recs computation.
pub struct LegacyCooccurStrategy;

impl LegacyCooccurStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LegacyCooccurStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for LegacyCooccurStrategy {
    fn name(&self) -> &str {
        "cooccur"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        match (user_id, seed) {
            // Personalized: the exact same computation the legacy handler
            // runs (tag Jaccard + bookmarks + downloads).
            (Some(uid), _) => {
                let (recs, _based_on, enough) =
                    crate::recommender::routes::compute_personal_recommendations(
                        &ctx.db,
                        uid,
                        &ctx.config,
                    )
                    .await?;
                if !enough {
                    return Err(RecError::NotEnoughData(
                        "fewer than 3 signals — legacy gate".into(),
                    ));
                }
                Ok(recs
                    .into_iter()
                    .map(|r| ScoredRec {
                        work_id: r.url_id,
                        score: r.score,
                        strategy: self.name().into(),
                        reason: "tag overlap with your bookmarks".into(),
                    })
                    .collect())
            }
            // Item-to-item: the legacy co-occurrence engine.
            (None, Some(seed_id)) => {
                let engine = crate::recommender::engine::RecommendationEngine::new(ctx.db.clone());
                let query = crate::recommender::engine::RecQuery {
                    url_id: seed_id.to_string(),
                    n: ctx.config.rec_max_recommendations,
                    site_domain: None,
                };
                let results = engine.get_recommendations(&query, &ctx.config).await?;
                Ok(results
                    .into_iter()
                    .map(|r| ScoredRec {
                        work_id: r.url_id,
                        score: r.score,
                        strategy: self.name().into(),
                        reason: "co-bookmarked by other readers".into(),
                    })
                    .collect())
            }
            (None, None) => Err(RecError::Strategy(
                "legacy_cooccur needs a user or a seed work".into(),
            )),
        }
    }

    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // The legacy engine has no batch step (precomputation happens in the
        // collection worker via compute_and_cache). Nothing to do here.
        Ok(json!({ "note": "legacy engine has no batch training step" }))
    }
}

// Helper so tests can map AppError → RecError cheaply.
#[allow(dead_code)]
pub(crate) fn map_app_error(e: AppError) -> RecError {
    RecError::Strategy(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_cooccur() {
        assert_eq!(LegacyCooccurStrategy::new().name(), "cooccur");
    }
}
