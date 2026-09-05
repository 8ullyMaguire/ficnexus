//! fichub-consensus — Generic Elo/MaxDiff consensus engine.
//!
//! This crate provides a reusable consensus ranking system based on Elo ratings
//! derived from MaxDiff (best-worst) pairwise comparisons. It is designed to be
//! project-agnostic: no FicHub-specific types, configurable stages/categories,
//! and pluggable storage/voting policies.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod store;
pub use store::{MemoryStore, RatingStore, SqlxStore};

/// Error types for the consensus engine.
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("store error: {0}")]
    Store(#[from] store::StoreError),
    #[error("invalid status: {0}")]
    InvalidStatus(String),
    #[error("invalid category: {0}")]
    InvalidCategory(String),
    #[error("voter not eligible: {0}")]
    VoterNotEligible(String),
    #[error("invalid vote: {0}")]
    InvalidVote(String),
    #[error("not found: {0}")]
    NotFound(String),
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, ConsensusError>;

/// Elo configuration parameters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EloConfig {
    /// Starting Elo rating for new features.
    pub start: f64,
    /// K-factor for Elo updates.
    pub k: f64,
    /// Optional decay per day (not implemented yet).
    pub decay_per_day: Option<f64>,
    /// Optional max Elo cap (not implemented yet).
    pub cap: Option<f64>,
}

impl Default for EloConfig {
    fn default() -> Self {
        Self {
            start: 1500.0,
            k: 32.0,
            decay_per_day: None,
            cap: None,
        }
    }
}

/// Ranking mode: Elo (MaxDiff) or SimpleVote (up/down counts).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RankingMode {
    /// Elo rating from MaxDiff pairwise matches.
    Elo(EloConfig),
    /// Simple up/down vote counts; ranking by score = up - down.
    SimpleVote,
}

impl Default for RankingMode {
    fn default() -> Self {
        Self::Elo(EloConfig::default())
    }
}

/// Stage definition for the roadmap Kanban board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    /// Machine ID matching DB `status` enum value.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Hex color for UI.
    pub color: String,
    /// Sort order on board (lower = leftmost).
    pub roadmaps_position: f64,
    /// If true, this stage is NOT served in arena pairs (voting frozen).
    pub frozen_for_arena: bool,
}

/// Category definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub label: String,
}

/// Voter reference for policy checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoterRef {
    /// User ID if authenticated, None for anonymous.
    pub id: Option<String>,
    /// Trust level (0 = anonymous/guest).
    pub trust_level: u8,
}

/// Policy trait for voting eligibility.
#[async_trait]
pub trait VoterPolicy: Send + Sync {
    async fn can_vote(&self, voter: &VoterRef) -> bool;
}

/// Default policy: allow all (no trust gate).
pub struct AllowAllPolicy;

#[async_trait]
impl VoterPolicy for AllowAllPolicy {
    async fn can_vote(&self, _voter: &VoterRef) -> bool {
        true
    }
}

/// Trust-level policy: require minimum trust level.
pub struct TrustLevelPolicy {
    pub min_trust_level: u8,
}

#[async_trait]
impl VoterPolicy for TrustLevelPolicy {
    async fn can_vote(&self, voter: &VoterRef) -> bool {
        voter.trust_level >= self.min_trust_level
    }
}

/// Full consensus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Ordered stages (left-to-right on Kanban).
    pub stages: Vec<Stage>,
    /// Available categories.
    pub categories: Vec<Category>,
    /// Default category for new features.
    pub default_category: String,
    /// If true, arena serves all stages; if false, only non-frozen stages.
    pub allow_voting_on_non_idea_stages: bool,
    /// Minimum trust level required to vote (None = open to all).
    pub min_trust_level_for_voting: Option<u8>,
    /// Ranking algorithm.
    pub ranking: RankingMode,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            stages: Self::default_stages(),
            categories: Self::default_categories(),
            default_category: "general".to_string(),
            allow_voting_on_non_idea_stages: false,
            min_trust_level_for_voting: None,
            ranking: RankingMode::default(),
        }
    }
}

impl ConsensusConfig {
    /// Default stages matching StoryGraph order (generic preset: freeze non-idea).
    pub fn default_stages() -> Vec<Stage> {
        vec![
            Stage {
                id: "up_next".into(),
                label: "Up Next".into(),
                color: "#f59e0b".into(),
                roadmaps_position: 1.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "in_progress".into(),
                label: "In Progress".into(),
                color: "#f97316".into(),
                roadmaps_position: 2.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "finished".into(),
                label: "Finished (Not Shipped)".into(),
                color: "#22c55e".into(),
                roadmaps_position: 3.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "shipped".into(),
                label: "Shipped".into(),
                color: "#10b981".into(),
                roadmaps_position: 4.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "medium_term".into(),
                label: "Medium-term".into(),
                color: "#ec4899".into(),
                roadmaps_position: 5.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "long_term".into(),
                label: "Long-term".into(),
                color: "#7c3aed".into(),
                roadmaps_position: 6.0,
                frozen_for_arena: true,
            },
            Stage {
                id: "idea".into(),
                label: "Ideas".into(),
                color: "#3b82f6".into(),
                roadmaps_position: 7.0,
                frozen_for_arena: false,
            },
            Stage {
                id: "rejected".into(),
                label: "Rejected".into(),
                color: "#9ca3af".into(),
                roadmaps_position: 8.0,
                frozen_for_arena: true,
            },
        ]
    }

    /// Default categories.
    pub fn default_categories() -> Vec<Category> {
        vec![
            Category {
                id: "all".into(),
                label: "All".into(),
            },
            Category {
                id: "search".into(),
                label: "Search".into(),
            },
            Category {
                id: "scraper".into(),
                label: "Scrapers".into(),
            },
            Category {
                id: "social".into(),
                label: "Social".into(),
            },
            Category {
                id: "reader".into(),
                label: "Reader".into(),
            },
            Category {
                id: "admin".into(),
                label: "Admin".into(),
            },
            Category {
                id: "recs".into(),
                label: "Recs".into(),
            },
            Category {
                id: "general".into(),
                label: "General".into(),
            },
        ]
    }

    /// FicHub preset: votes on all stages, L2+ trust gate.
    pub fn fichub() -> Self {
        let mut config = Self::default();
        config.allow_voting_on_non_idea_stages = true;
        config.min_trust_level_for_voting = Some(2);
        config
    }

    /// Get stage by ID.
    pub fn stage(&self, id: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.id == id)
    }

    /// Get all stage IDs that are NOT frozen for arena.
    pub fn arena_eligible_stage_ids(&self) -> Vec<String> {
        self.stages
            .iter()
            .filter(|s| !s.frozen_for_arena || self.allow_voting_on_non_idea_stages)
            .map(|s| s.id.clone())
            .collect()
    }

    /// Validate a status value.
    pub fn is_valid_status(&self, status: &str) -> bool {
        self.stages.iter().any(|s| s.id == status)
    }

    /// Validate a category value.
    pub fn is_valid_category(&self, category: &str) -> bool {
        self.categories.iter().any(|c| c.id == category)
    }
}

/// Feature/cluster record from the store.
#[cfg(feature = "sqlx-store")]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Feature {
    pub id: i32,
    pub representative_text: String,
    pub elo_rating: f64,
    pub matches_played: i32,
    pub times_picked_best: i32,
    pub times_picked_worst: i32,
    pub status: String,
    pub category: String,
    pub suggestions: i64,
}

#[cfg(not(feature = "sqlx-store"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: i32,
    pub representative_text: String,
    pub elo_rating: f64,
    pub matches_played: i32,
    pub times_picked_best: i32,
    pub times_picked_worst: i32,
    pub status: String,
    pub category: String,
    pub suggestions: i64,
}

/// Consensus engine.
pub struct Consensus<S: RatingStore> {
    config: ConsensusConfig,
    store: Arc<S>,
    voter_policy: Arc<dyn VoterPolicy>,
}

impl<S: RatingStore> Consensus<S> {
    /// Create a new consensus engine.
    pub fn new(config: ConsensusConfig, store: S) -> Self {
        let voter_policy: Arc<dyn VoterPolicy> =
            if let Some(min) = config.min_trust_level_for_voting {
                Arc::new(TrustLevelPolicy {
                    min_trust_level: min,
                })
            } else {
                Arc::new(AllowAllPolicy)
            };
        Self {
            config,
            store: Arc::new(store),
            voter_policy,
        }
    }

    /// Create with a custom voter policy.
    pub fn with_policy(config: ConsensusConfig, store: S, policy: Arc<dyn VoterPolicy>) -> Self {
        Self {
            config,
            store: Arc::new(store),
            voter_policy: policy,
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &ConsensusConfig {
        &self.config
    }

    /// Get leaderboard: all features sorted by Elo DESC, optionally filtered by status/category.
    pub async fn leaderboard(
        &self,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<Feature>> {
        let features = self.store.list_features(status, category).await?;
        // Already sorted by Elo DESC from store
        Ok(features)
    }

    /// Get features grouped by status (for Kanban board).
    pub async fn board(&self, category: Option<&str>) -> Result<HashMap<String, Vec<Feature>>> {
        let all = self.store.list_features(None, category).await?;
        let mut grouped: HashMap<String, Vec<Feature>> = HashMap::new();
        for f in all {
            grouped.entry(f.status.clone()).or_default().push(f);
        }
        // Ensure each status list is sorted by Elo DESC
        for v in grouped.values_mut() {
            v.sort_by(|a, b| b.elo_rating.partial_cmp(&a.elo_rating).unwrap());
        }
        Ok(grouped)
    }

    /// Record a MaxDiff vote: best vs worst among the given cluster_ids.
    pub async fn record_maxdiff(
        &self,
        voter: &VoterRef,
        cluster_ids: &[i32],
        best_id: i32,
        worst_id: i32,
    ) -> Result<Vec<(i32, f64)>> {
        // Check voter eligibility
        if !self.voter_policy.can_vote(voter).await {
            return Err(ConsensusError::VoterNotEligible(format!(
                "trust level {} below minimum",
                voter.trust_level
            )));
        }

        // Validate cluster_ids
        if cluster_ids.len() != 4 {
            return Err(ConsensusError::InvalidVote(
                "expected exactly 4 cluster ids".into(),
            ));
        }
        if !cluster_ids.contains(&best_id) || !cluster_ids.contains(&worst_id) {
            return Err(ConsensusError::InvalidVote(
                "best and worst must be in cluster_ids".into(),
            ));
        }
        if best_id == worst_id {
            return Err(ConsensusError::InvalidVote(
                "best and worst must be distinct".into(),
            ));
        }

        // Load current ratings
        let ratings = self.store.get_ratings(cluster_ids).await?;
        let mut rating_map: HashMap<i32, f64> = ratings.into_iter().collect();

        // Ensure all have ratings (default to start Elo)
        let start_elo = match &self.config.ranking {
            RankingMode::Elo(cfg) => cfg.start,
            RankingMode::SimpleVote => 0.0, // not used
        };
        for id in cluster_ids {
            rating_map.entry(*id).or_insert(start_elo);
        }

        // Get K factor
        let k = match &self.config.ranking {
            RankingMode::Elo(cfg) => cfg.k,
            RankingMode::SimpleVote => 0.0,
        };

        // Apply MaxDiff → 6 virtual 1v1 matches
        let updates = maxdiff_elo_updates(cluster_ids, &rating_map, best_id, worst_id, k);

        // Persist new ratings
        for (id, new_elo) in &updates {
            self.store.update_rating(*id, *new_elo).await?;
        }

        // Increment match counters
        for id in cluster_ids {
            let is_best = *id == best_id;
            let is_worst = *id == worst_id;
            self.store
                .increment_counters(*id, is_best, is_worst)
                .await?;
        }

        Ok(updates)
    }

    /// Get arena-eligible features (for pairing).
    pub async fn arena_candidates(&self, limit: usize) -> Result<Vec<Feature>> {
        let stage_ids = self.config.arena_eligible_stage_ids();
        if stage_ids.is_empty() {
            return Ok(vec![]);
        }
        // Store handles the status filter
        self.store.list_arena_candidates(&stage_ids, limit).await
    }
}

/// Pure Elo math: expected score of A vs B.
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

/// Pure Elo math: new rating after a match.
pub fn elo_update(rating: f64, opponent_rating: f64, score: f64, k: f64) -> f64 {
    let expected = expected_score(rating, opponent_rating);
    rating + k * (score - expected)
}

/// Translate MaxDiff (best, worst, 2 neutrals) into 6 virtual 1v1 Elo updates.
/// Returns Vec of (cluster_id, new_elo).
pub fn maxdiff_elo_updates(
    cluster_ids: &[i32],
    ratings: &HashMap<i32, f64>,
    best_id: i32,
    worst_id: i32,
    k: f64,
) -> Vec<(i32, f64)> {
    let mut new_ratings: HashMap<i32, f64> = ratings.clone();

    let mut update = |id: i32, opponent: i32, score: f64| {
        let current = new_ratings[&id];
        let opp = new_ratings[&opponent];
        new_ratings.insert(id, elo_update(current, opp, score, k));
    };

    for &id in cluster_ids {
        if id == best_id {
            // Best: wins vs the other three
            for &opp in cluster_ids {
                if opp != best_id {
                    update(id, opp, 1.0);
                }
            }
        } else if id == worst_id {
            // Worst: loses vs the other three
            for &opp in cluster_ids {
                if opp != worst_id {
                    update(id, opp, 0.0);
                }
            }
        } else {
            // Neutral: beats worst, loses to best, draws the other neutral
            update(id, worst_id, 1.0);
            update(id, best_id, 0.0);
            for &opp in cluster_ids {
                if opp != best_id && opp != worst_id && opp != id {
                    update(id, opp, 0.5);
                }
            }
        }
    }

    new_ratings.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_score_midpoint_is_half() {
        assert!((expected_score(1500.0, 1500.0) - 0.5).abs() < 1e-9);
        assert!(expected_score(1600.0, 1400.0) > 0.75);
    }

    #[test]
    fn elo_update_win_raises_loser_lowers() {
        let winner_new = elo_update(1500.0, 1500.0, 1.0, 32.0);
        let loser_new = elo_update(1500.0, 1500.0, 0.0, 32.0);
        assert!(winner_new > 1500.0);
        assert!(loser_new < 1500.0);
        assert!((winner_new - 1500.0 - (1500.0 - loser_new)).abs() < 1e-9);
    }

    #[test]
    fn maxdiff_best_soars_worst_tanks_neutrals_shift_little() {
        let ids = vec![1, 2, 3, 4];
        let ratings: HashMap<i32, f64> =
            [(1, 1500.0), (2, 1500.0), (3, 1500.0), (4, 1500.0)].into();
        let out = maxdiff_elo_updates(&ids, &ratings, 1, 4, 32.0);
        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        let r4 = out.iter().find(|(i, _)| *i == 4).unwrap().1;
        let r2 = out.iter().find(|(i, _)| *i == 2).unwrap().1;
        let r3 = out.iter().find(|(i, _)| *i == 3).unwrap().1;
        assert!(r1 > 1540.0, "best gains big: {r1}");
        assert!(r4 < 1460.0, "worst loses big: {r4}");
        assert!(r2 > r4 && r2 < r1, "neutral 2 between: {r2}");
        assert!(r3 > r4 && r3 < r1, "neutral 3 between: {r3}");
    }

    #[test]
    fn maxdiff_rating_gap_reduces_transfer() {
        let ids = vec![1, 2, 3, 4];
        let ratings: HashMap<i32, f64> =
            [(1, 1800.0), (2, 1500.0), (3, 1500.0), (4, 1200.0)].into();
        let out = maxdiff_elo_updates(&ids, &ratings, 1, 4, 32.0);
        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        assert!(r1 > 1800.0, "still gains");
        assert!(r1 - 1800.0 < 32.0 * 3.0, "gain bounded by K*matches");
    }

    #[test]
    fn default_config_freezes_non_idea() {
        let config = ConsensusConfig::default();
        let arena_ids = config.arena_eligible_stage_ids();
        assert_eq!(arena_ids, vec!["idea"]);
    }

    #[test]
    fn fichub_config_allows_all_stages() {
        let config = ConsensusConfig::fichub();
        let arena_ids = config.arena_eligible_stage_ids();
        assert_eq!(arena_ids.len(), 8); // all stages
    }

    #[tokio::test]
    async fn trust_level_policy_rejects_low() {
        let policy = TrustLevelPolicy { min_trust_level: 2 };
        let voter = VoterRef {
            id: Some("u1".into()),
            trust_level: 1,
        };
        assert!(!policy.can_vote(&voter).await);
    }

    #[tokio::test]
    async fn trust_level_policy_allows_high() {
        let policy = TrustLevelPolicy { min_trust_level: 2 };
        let voter = VoterRef {
            id: Some("u2".into()),
            trust_level: 3,
        };
        assert!(policy.can_vote(&voter).await);
    }
}
