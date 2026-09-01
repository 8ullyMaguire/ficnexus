//! Curator-prior taste-shaping strategy.
//!
//! The user's requirement: a new/low-signal user's recommendations should be
//! shaped toward the curator's taste (REC_CURATOR_PRIOR), so the home
//! dashboard can label the panel "Curator's pick". The blend happens in the
//! RANKER (`ranker::blend` → `apply_curator_prior`) because it must apply
//! AFTER all strategies fuse; this module holds the pure math + the
//! alignment tracking helpers.
//!
//!    final_profile = α·user + (1−α)·curator
//!    α = floor + (1−floor)·exp(−n_signals/τ)
//!
//! with floor = REC_PRIOR_FLOOR (0.2) and τ = REC_CURATOR_TAU (25).
//! The curator's own interactions are weighted 5× in rec_user_signals
//! (see signals.rs), so their profile dominates for cold users.

use super::ranker::cosine;

#[cfg(test)]
use super::ranker::curator_alpha;

/// The blend used by the ranker (re-exported for docs/tests).
pub use super::ranker::apply_curator_prior as blend_profiles;

/// Alignment score between a user's and the curator's tag-affinity vectors.
/// Pure function: cosine over (tag_id, weight) pairs.
pub fn alignment(user: &[(i32, f64)], curator: &[(i32, f64)]) -> f64 {
    cosine(user, curator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_is_cosine() {
        let user = vec![(1, 4.0), (2, 3.0)];
        let curator = vec![(1, 8.0), (2, 6.0)];
        assert!((alignment(&user, &curator) - 1.0).abs() < 1e-9);
        let other = vec![(9, 1.0)];
        assert!(alignment(&user, &other) < 0.01);
    }

    #[test]
    fn alpha_respects_floor_for_cold_users() {
        // Re-check the exported math: 0 signals → α = 1.0 (pure curator).
        let a = curator_alpha(0, 0.2, 25.0);
        assert!((a - 1.0).abs() < 1e-9);
        // 500 signals → α ≈ floor (user taste dominates, curator floor kept).
        let a = curator_alpha(500, 0.2, 25.0);
        assert!((a - 0.2).abs() < 0.02);
    }
}
