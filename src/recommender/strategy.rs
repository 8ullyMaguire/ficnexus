//! Pluggable recommendation strategies — the core trait and shared types.
//!
//! A [`RecStrategy`] turns a [`StrategyContext`] into a ranked list of
//! [`ScoredRec`]s for either a seed work (item-to-item) or a signed-in user
//! (personalized). Strategies are pure scorers: they never decide what gets
//! shown. The [`crate::recommender::registry::StrategyRegistry`] decides which
//! strategies are enabled (config-gated) and the
//! [`crate::recommender::ranker`] blends their outputs with Reciprocal Rank
//! Fusion, applies the curator prior, and slots in bandit exploration.
//!
//! The legacy engine is wrapped as the `legacy_cooccur` strategy so that
//! `REC_ENGINE_MODE=legacy` (the default) preserves today's behavior exactly.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::error::AppError;
use crate::services::ollama::OllamaClient;

/// A single scored recommendation produced by a strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredRec {
    /// url_id of the recommended work (fic_info.id).
    pub work_id: String,
    /// Raw strategy score (before RRF blending).
    pub score: f64,
    /// Strategy name that produced this rec (e.g. `legacy_cooccur`).
    pub strategy: String,
    /// Human-readable reason, surfaced in the API for QA ("because you
    /// bookmarked 3 fics in this fandom", "co-bookmarked by 12 readers", …).
    pub reason: String,
}

/// Context handed to every strategy invocation.
///
/// Cheap to clone (pools/clients are `Arc`-like); strategies build it once
/// per request via [`StrategyContext::new`].
#[derive(Debug, Clone)]
pub struct StrategyContext {
    pub db: PgPool,
    pub config: Arc<Config>,
    /// Shared HTTP client (scrapers, external sidecars, Ollama).
    pub http_client: reqwest::Client,
    /// Ollama embeddings client (nomic-embed-text). The embeddings strategy
    /// is the only consumer, but the context carries it so strategies don't
    /// construct their own clients.
    pub ollama: OllamaClient,
    /// Wall-clock "now" for decay math. Frozen per request so all strategies
    /// agree on time.
    pub now: DateTime<Utc>,
}

impl StrategyContext {
    pub fn new(
        db: PgPool,
        config: Arc<Config>,
        http_client: reqwest::Client,
        ollama: OllamaClient,
    ) -> Self {
        Self {
            db,
            config,
            http_client,
            ollama,
            now: Utc::now(),
        }
    }

    /// Per-strategy HTTP timeout for external calls (Ollama, sidecars).
    pub fn call_timeout(&self) -> Duration {
        Duration::from_secs(self.config.rec_strategy_timeout_secs)
    }
}

/// Errors that can abort a strategy run. Strategies are expected to degrade
/// gracefully: the ranker treats an `Err` as "skip this strategy and fall
/// through to the next" (the fallback chain), never as a hard failure of the
/// whole recommendation request.
#[derive(Debug, thiserror::Error)]
pub enum RecError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("strategy error: {0}")]
    Strategy(String),
    #[error("external sidecar error: {0}")]
    External(String),
    #[error("not enough data: {0}")]
    NotEnoughData(String),
    #[error("work {0} not found")]
    NotFound(String),
}

impl From<AppError> for RecError {
    fn from(e: AppError) -> Self {
        RecError::Strategy(e.to_string())
    }
}

/// A recommendation strategy.
///
/// Implementations MUST be cheap to construct and hold no per-request state;
/// the registry keeps one instance per strategy. `score` is called with a
/// `user_id` for personalized requests or `None` for item-to-item requests
/// (seed work in `ctx`-independent params).
#[async_trait]
pub trait RecStrategy: Send + Sync {
    /// Canonical strategy name (registry key, `rec_*` log/training key).
    fn name(&self) -> &str;

    /// Score candidate works.
    ///
    /// * `user_id: Some(id)` — personalized: score from the user's signals.
    /// * `user_id: None` — item-to-item: `seed` must be set and scoring is
    ///   relative to the seed work.
    ///
    /// Return an ordered (descending score) list, or `Err` to be skipped by
    /// the ranker's fallback chain.
    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError>;

    /// Run this strategy's training/precompute batch step (nightly/hourly
    /// pipeline). The default is a no-op; strategies that need precomputed
    /// artifacts (embeddings, MF factors, clusters, transitions, bandit
    /// decay) override it. Returns a JSON-ish metrics map for
    /// `rec_training_runs`.
    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        Ok(serde_json::json!({ "note": "no-op" }))
    }

    /// Whether this strategy serves personalized requests. Default true;
    /// item-to-item-only strategies (e.g. `sequential`) override.
    fn supports_personal(&self) -> bool {
        true
    }
}

/// A strategy that produced no output for a request (empty candidate pool).
/// Kept as a distinct error variant so the ranker can log it distinctly.
pub fn not_enough_data(why: impl Into<String>) -> RecError {
    RecError::NotEnoughData(why.into())
}
