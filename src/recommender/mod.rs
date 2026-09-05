pub mod author_graph;
pub mod bandit;
pub mod clusters;
pub mod curator;
pub mod decay;
pub mod embedding_recs;
pub mod embeddings;
pub mod engine;
pub mod entities;
pub mod external;
pub mod hybrid;
pub mod legacy_cooccur;
pub mod mf;
pub mod ranker;
pub mod registry;
pub mod routes;
pub mod sequential;
pub mod signals;
pub mod strategy;
pub mod tag_graph;
/// Recommender system — pluggable strategy platform.
///
/// Modules:
/// - `worker`: Background collection worker that scrapes user favourites/bookmarks
/// - `engine`: Legacy recommendation computation (co-occurrence, tag fallback, scoring)
/// - `routes`: API endpoints for recommendations, suggestions, and voting
/// - `embedding_recs`: GET /api/recommendations/embeddings (works.embedding cosine recs)
/// - `strategy`: `RecStrategy` trait + `ScoredRec`/`StrategyContext`/`RecError`
/// - `registry`: config-driven strategy registry (`REC_STRATEGIES`)
/// - `ranker`: Reciprocal Rank Fusion blending + curator prior + bandit slots
/// - `signals`: unified user-signal view (bookmarks/downloads/ratings/reviews)
/// - `legacy_cooccur`: the legacy engine wrapped as the default `cooccur` strategy
/// - `decay`: time-decayed co-occurrence (SAR-style)
/// - `embeddings`: pgvector embedding strategy (Ollama nomic-embed-text)
/// - `mf`: implicit matrix factorization (ALS) — Cargo feature `rec-mf`
/// - `hybrid`: learned-weight blend of MF + embeddings + tag Jaccard
/// - `author_graph`: author-level co-bookmark + shared-tag Jaccard
/// - `tag_graph`: meta-path tag knowledge-graph traversal
/// - `sequential`: first-order Markov next-read transitions
/// - `clusters`: taste-cluster k-means memberships
/// - `bandit`: Thompson-sampling contextual bandit exploration
/// - `external`: HTTP adapter for optional Python sidecars (`REC_EXTERNAL_URL`)
pub mod worker;

pub use engine::{RecQuery, RecResult, RecommendationEngine};
pub use strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};
pub use worker::CollectionWorker;

/// Assemble the full set of available strategies for the registry.
/// `external` is included only when `REC_EXTERNAL_URL` is set.
pub fn available_strategies(
    config: &crate::config::Config,
) -> Vec<std::sync::Arc<dyn RecStrategy>> {
    let mut strategies: Vec<std::sync::Arc<dyn RecStrategy>> = vec![
        std::sync::Arc::new(legacy_cooccur::LegacyCooccurStrategy::new()),
        std::sync::Arc::new(decay::DecayCooccurStrategy::new()),
        std::sync::Arc::new(embeddings::EmbeddingsStrategy::new()),
        std::sync::Arc::new(hybrid::HybridStrategy::new()),
        std::sync::Arc::new(author_graph::AuthorGraphStrategy::new()),
        std::sync::Arc::new(tag_graph::TagGraphStrategy::new()),
        std::sync::Arc::new(sequential::SequentialStrategy::new()),
        std::sync::Arc::new(clusters::ClustersStrategy::new()),
        std::sync::Arc::new(bandit::BanditStrategy::new()),
    ];
    #[cfg(feature = "rec-mf")]
    strategies.push(std::sync::Arc::new(mf::MfStrategy::new()));
    if config.rec_external_url.is_some() {
        strategies.push(std::sync::Arc::new(external::ExternalStrategy::new()));
    }
    strategies
}
