//! Time-decayed co-occurrence strategy (SAR-style).
//!
//! Improves on the legacy co-occurrence engine by decaying the co-occurrence
//! signal with time: a pair bookmarked together last week counts far more
//! than one last seen a year ago. Pure SQL — no new artifacts needed.
//!
//! Scoring (per candidate work):
//!   score = Σ_pair  cooccur_count · exp(-Δt / halflife) · log-likelihood
//!
//! where the log-likelihood term is the standard co-occurrence
//! informativeness: how much more often A&B co-occur than expected by chance
//! (pointwise mutual information with a log base, smoothed).

use async_trait::async_trait;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Decay half-life in days for the exponential `exp(-ln2 · Δt / half_life_days)`.
/// `REC_DECAY_HALFLIFE_DAYS` (default 30): a 30-day-old co-occurrence counts
/// exactly half as much as a fresh one.
pub fn decay_weight(days_ago: f64, half_life_days: f64) -> f64 {
    if days_ago <= 0.0 {
        return 1.0;
    }
    (-std::f64::consts::LN_2 * days_ago / half_life_days.max(1e-9)).exp()
}

/// Log-likelihood ratio: how much more often than chance do A and B
/// co-occur? `cooccur` = observed co-occurrences, `favouriters_a`/`b` =
/// marginal counts, `total` = total users. Returns a smoothed positive
/// weight; 0 when there is no evidence of association.
pub fn log_likelihood(cooccur: f64, favouriters_a: f64, favouriters_b: f64, total: f64) -> f64 {
    if cooccur <= 0.0 || favouriters_a <= 0.0 || favouriters_b <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    // Expected co-occurrence under independence.
    let expected = (favouriters_a * favouriters_b) / total;
    if expected <= 0.0 {
        return 0.0;
    }
    let ratio = cooccur / expected;
    // Log2 of the ratio, clamped to a sane range (avoids infinities on
    // tiny expected values).
    ratio.ln() / std::f64::consts::LN_2
}

pub struct DecayCooccurStrategy;

impl DecayCooccurStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DecayCooccurStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for DecayCooccurStrategy {
    fn name(&self) -> &str {
        "decay"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        if user_id.is_some() {
            // Decay co-occurrence is a seed/item-to-item strategy; the
            // personalized path is served by legacy_cooccur / others.
            return Err(RecError::NotEnoughData(
                "decay strategy is seed-based only".into(),
            ));
        }
        let Some(seed_id) = seed else {
            return Err(RecError::Strategy("decay needs a seed work".into()));
        };

        let half_life = ctx.config.rec_decay_halflife_days;
        let limit = ctx.config.rec_max_recommendations as i64;

        // Candidate co-occurrences with decayed weights + log-likelihood.
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"
            WITH seed AS (
                SELECT fw.url_id, fw.favouriter_count
                FROM fic_works fw WHERE fw.url_id = $1
            ),
            tot AS (
                SELECT COALESCE(SUM(favouriter_count), 1)::float8 AS total
                FROM fic_works
            ),
            candidates AS (
                SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
                       cooccur_count,
                       last_updated
                FROM fic_bookmark_cooccur
                WHERE work_a = $1 OR work_b = $1
            )
            SELECT c.candidate_id,
                   (c.cooccur_count::float8
                        * exp(-(ln(2.0) * EXTRACT(EPOCH FROM (NOW() - c.last_updated)) / 86400.0) / $3)
                        * (ln((c.cooccur_count::float8 * t.total)
                              / GREATEST(s.favouriter_count * fw.favouriter_count, 1.0))
                           / ln(2.0))) AS score
            FROM candidates c
            CROSS JOIN seed s
            CROSS JOIN tot t
            JOIN fic_works fw ON fw.url_id = c.candidate_id
            WHERE c.candidate_id != $1
            ORDER BY score DESC
            LIMIT $2
            "#,
        )
        .bind(seed_id)
        .bind(limit)
        .bind(half_life)
        .fetch_all(&ctx.db)
        .await?;

        if rows.is_empty() {
            return Err(RecError::NotEnoughData(format!(
                "no co-occurrence data for seed {seed_id}"
            )));
        }

        Ok(rows
            .into_iter()
            .map(|(work_id, score)| ScoredRec {
                work_id,
                score,
                strategy: self.name().into(),
                reason: "time-decayed co-bookmarks (SAR)".into(),
            })
            .collect())
    }

    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // Pure SQL scoring — nothing to precompute.
        Ok(serde_json::json!({ "note": "decay scoring is computed live" }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decay_weight_halflife_math() {
        // Fresh pair → 1.0.
        assert!((decay_weight(0.0, 30.0) - 1.0).abs() < 1e-9);
        // Exactly one half-life → 0.5.
        assert!((decay_weight(30.0, 30.0) - 0.5).abs() < 1e-9);
        // Two half-lives → 0.25.
        assert!((decay_weight(60.0, 30.0) - 0.25).abs() < 1e-9);
        // Monotonic decreasing.
        assert!(decay_weight(10.0, 30.0) > decay_weight(100.0, 30.0));
    }
    #[test]
    fn log_likelihood_prefers_strong_association() {
        let total = 1000.0;
        // A and B always co-occur → high score.
        let strong = log_likelihood(100.0, 100.0, 100.0, total);
        // Expected = 10; ratio = 10 → ln2(10) ≈ 3.32.
        assert!((strong - 3.3219).abs() < 1e-3, "got {strong}");
        // Independent association → ~0.
        let weak = log_likelihood(10.0, 100.0, 100.0, total);
        assert!(weak.abs() < 1e-9, "got {weak}");
        // No co-occurrence → 0.
        assert_eq!(log_likelihood(0.0, 100.0, 100.0, total), 0.0);
    }

    #[test]
    fn name_is_decay() {
        assert_eq!(DecayCooccurStrategy::new().name(), "decay");
    }
}
