//! Config-driven strategy registry.
//!
//! Enabled strategies come from `REC_STRATEGIES` — a comma-separated list of
//! `name:weight` pairs, e.g. `cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1`.
//! The default is `["cooccur"]` (the legacy engine wrapped as a strategy),
//! which combined with `REC_ENGINE_MODE=legacy` (default) preserves today's
//! behavior exactly.
//!
//! Registry responsibilities:
//! * parse `REC_STRATEGIES` into ordered `(name, weight)` pairs
//! * resolve each name to a strategy instance, skipping unknown names with a
//!   warning (so a typo'd config degrades rather than crashes)
//! * expose `enabled()` for the ranker and `all()` for
//!   `GET /api/recommendations/strategies`

use std::collections::HashMap;
use std::sync::Arc;

use tracing::warn;

use super::strategy::{RecStrategy, StrategyContext};

/// A parsed `REC_STRATEGIES` entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StrategySpec {
    pub name: String,
    pub weight: f64,
}

/// Parse `REC_STRATEGIES` ("a:0.5,b:0.3" or plain "a,b"; missing weight → 1.0).
/// Unknown names survive parsing; they are dropped at resolution time.
pub fn parse_strategy_specs(raw: &str) -> Vec<StrategySpec> {
    raw.split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .filter_map(|part| {
            let (name, weight) = match part.split_once(':') {
                Some((n, w)) => (n.trim().to_string(), w.trim().parse::<f64>().unwrap_or(1.0)),
                None => (part.to_string(), 1.0),
            };
            if name.is_empty() {
                None
            } else {
                Some(StrategySpec {
                    name,
                    weight: if weight.is_finite() && weight > 0.0 { weight } else { 1.0 },
                })
            }
        })
        .collect()
}

/// The config-driven registry. Built once at server start from `Config`.
pub struct StrategyRegistry {
    /// Enabled strategy specs, in config order.
    specs: Vec<StrategySpec>,
    /// Resolved strategy instances keyed by name.
    strategies: HashMap<String, Arc<dyn RecStrategy>>,
}

impl StrategyRegistry {
    /// Build a registry from the given strategy instances (by name) and the
    /// parsed `REC_STRATEGIES` list. Unknown names in `specs` are dropped
    /// with a warning.
    pub fn new(
        available: Vec<Arc<dyn RecStrategy>>,
        raw_specs: &str,
    ) -> Self {
        let parsed = parse_strategy_specs(raw_specs);
        let mut by_name: HashMap<String, Arc<dyn RecStrategy>> = HashMap::new();
        for s in available {
            by_name.insert(s.name().to_string(), s);
        }

        let mut specs = Vec::new();
        for spec in parsed {
            if by_name.contains_key(&spec.name) {
                specs.push(spec);
            } else {
                warn!("rec strategy '{0}' in REC_STRATEGIES is unknown — skipping", spec.name);
            }
        }
        // Fall back to the legacy co-occurrence strategy when nothing valid
        // was configured (or the config was empty).
        if specs.is_empty() {
            if let Some(_legacy) = by_name.get("cooccur") {
                warn!("REC_STRATEGIES resolved to nothing — falling back to ['cooccur']");
                specs.push(StrategySpec {
                    name: "cooccur".into(),
                    weight: 1.0,
                });
            }
        }

        Self { specs, strategies: by_name }
    }

    /// Enabled specs in config order (used by the ranker for RRF weights).
    pub fn specs(&self) -> &[StrategySpec] {
        &self.specs
    }

    /// Look up a strategy instance by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn RecStrategy>> {
        self.strategies.get(name).cloned()
    }

    /// All registered strategies (enabled or not) — for the admin
    /// `/api/recommendations/strategies` listing.
    pub fn all(&self) -> Vec<Arc<dyn RecStrategy>> {
        self.strategies.values().cloned().collect()
    }

    /// All enabled strategy names, in config order.
    pub fn enabled_names(&self) -> Vec<String> {
        self.specs.iter().map(|s| s.name.clone()).collect()
    }

    /// Total weight of enabled specs (for RRF weight normalization).
    pub fn total_weight(&self) -> f64 {
        self.specs.iter().map(|s| s.weight).sum::<f64>().max(f64::EPSILON)
    }
}

impl std::fmt::Debug for StrategyRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrategyRegistry")
            .field("specs", &self.specs)
            .field("strategies", &self.enabled_names())
            .finish()
    }
}

// Convenience alias for tests and handlers.
pub type StrategyMap = HashMap<String, Arc<dyn RecStrategy>>;

/// Build a full strategy context from AppState pieces (used by handlers and
/// the pipeline worker).
pub fn build_context(
    db: sqlx::PgPool,
    config: Arc<crate::config::Config>,
    http_client: reqwest::Client,
    ollama: crate::services::ollama::OllamaClient,
) -> StrategyContext {
    StrategyContext::new(db, config, http_client, ollama)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_weighted_list() {
        let specs = parse_strategy_specs("cooccur:0.4,embeddings:0.3,mf:0.2,bandit:0.1");
        assert_eq!(specs.len(), 4);
        assert_eq!(specs[0], StrategySpec { name: "cooccur".into(), weight: 0.4 });
        assert_eq!(specs[3], StrategySpec { name: "bandit".into(), weight: 0.1 });
    }

    #[test]
    fn parses_unweighted_list_with_default_weight() {
        let specs = parse_strategy_specs("cooccur,embeddings");
        assert_eq!(
            specs,
            vec![
                StrategySpec { name: "cooccur".into(), weight: 1.0 },
                StrategySpec { name: "embeddings".into(), weight: 1.0 },
            ]
        );
    }

    #[test]
    fn tolerates_garbage_and_empty() {
        assert!(parse_strategy_specs("").is_empty());
        assert!(parse_strategy_specs(",,,  ,").is_empty());
        let specs = parse_strategy_specs("cooccur:notanumber");
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].weight, 1.0);
        // Negative/zero weights clamp to 1.0 (never disable via weight 0).
        let specs = parse_strategy_specs("mf:-2");
        assert_eq!(specs[0].weight, 1.0);
    }

    #[test]
    fn drops_unknown_names_and_falls_back_to_cooccur() {
        let legacy = super::super::legacy_cooccur::LegacyCooccurStrategy::new();
        let available: Vec<Arc<dyn RecStrategy>> = vec![Arc::new(legacy)];
        let reg = StrategyRegistry::new(available, "embeddings:0.5,cooccur:0.5,nope:0.1");
        assert_eq!(reg.enabled_names(), vec!["cooccur"]);
        assert_eq!(reg.specs()[0].weight, 0.5);

        // Empty config → fallback to ['cooccur'].
        let legacy = super::super::legacy_cooccur::LegacyCooccurStrategy::new();
        let available: Vec<Arc<dyn RecStrategy>> = vec![Arc::new(legacy)];
        let reg = StrategyRegistry::new(available, "");
        assert_eq!(reg.enabled_names(), vec!["cooccur"]);
    }
}
