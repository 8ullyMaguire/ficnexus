//! Hybrid blend strategy — learned-weight mix of MF + embedding cosine +
//! tag Jaccard.
//!
//! The hybrid is itself a strategy: it combines the *signals* of the
//! component strategies (mf, embeddings, tag Jaccard) with learned weights,
//! producing a single ranked list. Weights default to the RRF-style
//! `(0.4, 0.4, 0.2)`; they are overridden in this order by
//! `REC_HYBRID_WEIGHTS` (JSON `[mf, embedding, tag]`), then by the
//! engagement-learned weights persisted by `train` (smoothed CTR over
//! `rec_impressions`), see `ctr_weights` below.
//!
//! Note: when the component strategies are also enabled individually, the
//! ranker's RRF will include the hybrid's output alongside theirs — the
//! recommended config is to enable EITHER `hybrid` OR the individual
//! components, not both.

use async_trait::async_trait;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Default hybrid weights: (mf, embedding, tag).
pub const DEFAULT_HYBRID_WEIGHTS: (f64, f64, f64) = (0.4, 0.4, 0.2);

pub struct HybridStrategy;

impl HybridStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HybridStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse `REC_HYBRID_WEIGHTS` ("0.4,0.4,0.2") with fallback to defaults.
pub fn parse_hybrid_weights(raw: Option<&str>) -> (f64, f64, f64) {
    let Some(raw) = raw else {
        return DEFAULT_HYBRID_WEIGHTS;
    };
    let parts: Vec<f64> = raw
        .split(',')
        .filter_map(|p| p.trim().parse::<f64>().ok())
        .collect();
    if parts.len() != 3 {
        return DEFAULT_HYBRID_WEIGHTS;
    }
    (parts[0], parts[1], parts[2])
}

/// Component strategies in the (mf, embedding, tag) order used throughout.
const COMPONENT_STRATEGIES: [&str; 3] = ["mf", "embeddings", "tag_graph"];

/// Beta(2, 20) prior mean ≈ 0.09 — keeps CTRs of tiny samples well-behaved.
const CTR_PRIOR_SHOWN: f64 = 22.0;
const CTR_PRIOR_ENGAGED: f64 = 2.0;

/// Compute hybrid weights from per-component (shown, engaged) counters:
/// weight ∝ smoothed CTR for components with data. A component with no
/// impressions is assumed to perform at the observed system average CTR, so
/// an unmeasured strategy can never outrank one with real engagement data.
/// The result is normalized to sum 1; with no impressions anywhere the
/// configured default blend is returned.
pub fn ctr_weights(shown: &[i64; 3], engaged: &[i64; 3]) -> (f64, f64, f64) {
    let defaults = [
        DEFAULT_HYBRID_WEIGHTS.0,
        DEFAULT_HYBRID_WEIGHTS.1,
        DEFAULT_HYBRID_WEIGHTS.2,
    ];

    let mut ctrs = [0.0f64; 3];
    let mut measured = [false; 3];
    let mut n_measured = 0usize;
    for i in 0..3 {
        if shown[i] > 0 {
            ctrs[i] = (engaged[i] as f64 + CTR_PRIOR_ENGAGED) / (shown[i] as f64 + CTR_PRIOR_SHOWN);
            measured[i] = true;
            n_measured += 1;
        }
    }

    // Nothing measured anywhere: fall back to the configured default blend.
    if n_measured == 0 {
        let sum: f64 = defaults.iter().sum();
        if sum <= f64::EPSILON {
            return DEFAULT_HYBRID_WEIGHTS;
        }
        return (defaults[0] / sum, defaults[1] / sum, defaults[2] / sum);
    }

    // Unmeasured components are imputed with the mean observed CTR.
    let mean_ctr = ctrs
        .iter()
        .zip(measured.iter())
        .filter(|(_, m)| **m)
        .map(|(c, _)| *c)
        .sum::<f64>()
        / n_measured as f64;

    let mut raw = [0.0f64; 3];
    for i in 0..3 {
        raw[i] = if measured[i] { ctrs[i] } else { mean_ctr };
    }

    let sum: f64 = raw.iter().sum();
    if sum <= f64::EPSILON {
        return DEFAULT_HYBRID_WEIGHTS;
    }
    (raw[0] / sum, raw[1] / sum, raw[2] / sum)
}

/// Learn component weights from observed engagement in `rec_impressions`
/// (shown → engaged). Used by the hybrid `train` pass.
pub async fn learn_weights_from_impressions(
    db: &sqlx::PgPool,
) -> Result<(f64, f64, f64), RecError> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        r#"SELECT strategy, COUNT(*) AS shown, COUNT(engaged_at) AS engaged
           FROM rec_impressions
           WHERE strategy = ANY($1)
           GROUP BY strategy"#,
    )
    .bind(&COMPONENT_STRATEGIES[..])
    .fetch_all(db)
    .await
    .map_err(|e| RecError::External(format!("load impressions for hybrid: {e}")))?;

    let mut shown = [0i64; 3];
    let mut engaged = [0i64; 3];
    for (strategy, s, e) in rows {
        if let Some(i) = COMPONENT_STRATEGIES.iter().position(|n| *n == strategy) {
            shown[i] = s;
            engaged[i] = e;
        }
    }
    Ok(ctr_weights(&shown, &engaged))
}

/// Load learned weights persisted by `train` in `rec_models`.
async fn load_stored_weights(db: &sqlx::PgPool) -> Option<(f64, f64, f64)> {
    let metrics: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT metrics FROM rec_models WHERE name = 'hybrid-weights'
         ORDER BY trained_at DESC LIMIT 1",
    )
    .fetch_optional(db)
    .await
    .ok()
    .flatten()?;

    let metrics = match metrics {
        Some(m) => m,
        None => return None,
    };
    let w = match metrics.get("weights") {
        Some(w) => w,
        None => return None,
    };
    let mf = w.get("mf")?.as_f64()?;
    let emb = w.get("embeddings")?.as_f64()?;
    let tag = w.get("tags")?.as_f64()?;
    if mf.is_finite() && emb.is_finite() && tag.is_finite() {
        Some((mf, emb, tag))
    } else {
        None
    }
}

/// Weight resolution order: `REC_HYBRID_WEIGHTS` env override, then learned
/// weights persisted by `train`, then static defaults.
async fn resolve_weights(ctx: &StrategyContext) -> (f64, f64, f64) {
    match std::env::var("REC_HYBRID_WEIGHTS") {
        Ok(raw) => parse_hybrid_weights(Some(&raw)),
        Err(_) => load_stored_weights(&ctx.db)
            .await
            .unwrap_or(DEFAULT_HYBRID_WEIGHTS),
    }
}

#[async_trait]
impl RecStrategy for HybridStrategy {
    fn name(&self) -> &str {
        "hybrid"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        let weights = resolve_weights(ctx).await;

        // Component 1: MF scores (requires trained factors).
        let mf_scores = mf_component_scores(ctx, user_id).await;
        // Component 2: embedding cosine vs user profile / seed.
        let emb_scores = embedding_component_scores(ctx, seed, user_id).await;
        // Component 3: tag Jaccard vs user's top tags / seed tags.
        let tag_scores = tag_component_scores(ctx, seed, user_id).await;

        if mf_scores.is_empty() && emb_scores.is_empty() && tag_scores.is_empty() {
            return Err(RecError::NotEnoughData(
                "hybrid has no component signals".into(),
            ));
        }

        // Normalize each component to [0,1] (min-max), then blend.
        let mf_norm = normalize(&mf_scores);
        let emb_norm = normalize(&emb_scores);
        let tag_norm = normalize(&tag_scores);

        let mut blended: Vec<ScoredRec> = Vec::new();
        for (work, s) in mf_norm {
            blended.push(ScoredRec {
                work_id: work,
                score: s * weights.0,
                strategy: self.name().into(),
                reason: "hybrid (mf)".into(),
            });
        }
        for (work, s) in emb_norm {
            let entry = blended.iter_mut().find(|r| r.work_id == work);
            match entry {
                Some(e) => e.score += s * weights.1,
                None => blended.push(ScoredRec {
                    work_id: work,
                    score: s * weights.1,
                    strategy: self.name().into(),
                    reason: "hybrid (embeddings)".into(),
                }),
            }
        }
        for (work, s) in tag_norm {
            let entry = blended.iter_mut().find(|r| r.work_id == work);
            match entry {
                Some(e) => e.score += s * weights.2,
                None => blended.push(ScoredRec {
                    work_id: work,
                    score: s * weights.2,
                    strategy: self.name().into(),
                    reason: "hybrid (tags)".into(),
                }),
            }
        }

        blended.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        blended.truncate(ctx.config.rec_max_recommendations);
        if blended.is_empty() {
            return Err(RecError::NotEnoughData("hybrid produced nothing".into()));
        }
        Ok(blended)
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // Learn component weights from observed engagement: for each component
        // strategy, compute its smoothed click-through rate over
        // `rec_impressions` (shown → engaged). Weight ∝ CTR; when a component
        // has no impressions yet it keeps a neutral prior weight. The learned
        // weights are persisted to `rec_models` (name='hybrid-weights') and
        // picked up by `score` on the next request; `REC_HYBRID_WEIGHTS`
        // always overrides them.
        let learned = learn_weights_from_impressions(&ctx.db).await?;
        let (mf_w, emb_w, tag_w) = learned;

        // Persist so `score` can serve the learned blend without re-learning.
        let metrics = serde_json::json!({
            "weights": { "mf": mf_w, "embeddings": emb_w, "tags": tag_w },
            "source": "rec_impressions ctr",
        });
        sqlx::query(
            r#"INSERT INTO rec_models (name, version, trained_at, metrics)
               VALUES ('hybrid-weights', '1', NOW(), $1)
               ON CONFLICT (name, version) DO UPDATE SET
                   trained_at = NOW(),
                   metrics = EXCLUDED.metrics"#,
        )
        .bind(&metrics)
        .execute(&ctx.db)
        .await
        .map_err(|e| RecError::External(format!("persist hybrid weights: {e}")))?;

        Ok(metrics)
    }
}

/// Min-max normalize a scored list into [0,1]; empty stays empty.
pub fn normalize(scores: &[(String, f64)]) -> Vec<(String, f64)> {
    if scores.is_empty() {
        return Vec::new();
    }
    let max = scores
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);
    let min = scores.iter().map(|(_, s)| *s).fold(f64::INFINITY, f64::min);
    let range = max - min;
    if range <= 1e-12 {
        return scores.iter().map(|(w, _)| (w.clone(), 1.0)).collect();
    }
    scores
        .iter()
        .map(|(w, s)| (w.clone(), (s - min) / range))
        .collect()
}

async fn mf_component_scores(ctx: &StrategyContext, user_id: Option<i32>) -> Vec<(String, f64)> {
    let Some(uid) = user_id else {
        return Vec::new();
    };
    // Reuse the MF strategy's scoring (works with or without rec-mf).
    let mf = super::mf::MfStrategy::new();
    match mf.score(ctx, None, Some(uid)).await {
        Ok(list) => list.into_iter().map(|r| (r.work_id, r.score)).collect(),
        Err(_) => Vec::new(),
    }
}

async fn embedding_component_scores(
    ctx: &StrategyContext,
    seed: Option<&str>,
    user_id: Option<i32>,
) -> Vec<(String, f64)> {
    let emb = super::embeddings::EmbeddingsStrategy::new();
    match emb.score(ctx, seed, user_id).await {
        Ok(list) => list.into_iter().map(|r| (r.work_id, r.score)).collect(),
        Err(_) => Vec::new(),
    }
}

async fn tag_component_scores(
    ctx: &StrategyContext,
    seed: Option<&str>,
    user_id: Option<i32>,
) -> Vec<(String, f64)> {
    let tag = super::tag_graph::TagGraphStrategy::new();
    match tag.score(ctx, seed, user_id).await {
        Ok(list) => list.into_iter().map(|r| (r.work_id, r.score)).collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_maps_to_unit_range() {
        let scores = vec![("a".to_string(), 2.0), ("b".to_string(), 4.0)];
        let n = normalize(&scores);
        assert!((n[0].1 - 0.0).abs() < 1e-9);
        assert!((n[1].1 - 1.0).abs() < 1e-9);
        assert!(normalize(&[]).is_empty());
        // Flat list → all 1.0.
        let flat = normalize(&[("a".to_string(), 3.0), ("b".to_string(), 3.0)]);
        assert!((flat[0].1 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn hybrid_weights_parse_with_fallback() {
        assert_eq!(parse_hybrid_weights(None), (0.4, 0.4, 0.2));
        assert_eq!(parse_hybrid_weights(Some("0.5,0.3,0.2")), (0.5, 0.3, 0.2));
        assert_eq!(parse_hybrid_weights(Some("garbage")), (0.4, 0.4, 0.2));
        assert_eq!(parse_hybrid_weights(Some("0.5,0.3")), (0.4, 0.4, 0.2));
    }

    #[test]
    fn ctr_weights_with_no_data_keeps_defaults() {
        let shown = [0, 0, 0];
        let engaged = [0, 0, 0];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        let defaults = [
            DEFAULT_HYBRID_WEIGHTS.0,
            DEFAULT_HYBRID_WEIGHTS.1,
            DEFAULT_HYBRID_WEIGHTS.2,
        ];
        let sum = defaults.iter().sum::<f64>();
        assert!((mf - defaults[0] / sum).abs() < 1e-9);
        assert!((emb - defaults[1] / sum).abs() < 1e-9);
        assert!((tag - defaults[2] / sum).abs() < 1e-9);
    }

    #[test]
    fn ctr_weights_prefers_component_with_better_engagement() {
        // embeddings: 40/100 engaged; tag: 5/100 engaged; mf: no data.
        let shown = [0, 100, 100];
        let engaged = [0, 40, 5];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        assert!(emb > tag, "higher CTR must win: emb={emb} tag={tag}");
        assert!(emb > mf, "40% CTR beats a default no-data share");
        assert!((mf + emb + tag - 1.0).abs() < 1e-9, "weights normalized");
    }

    #[test]
    fn ctr_weights_never_degenerate() {
        // All-engaged components still produce a valid normalized blend.
        let shown = [10, 10, 10];
        let engaged = [10, 10, 10];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        assert!((mf + emb + tag - 1.0).abs() < 1e-9);
        assert!(mf.is_finite() && emb.is_finite() && tag.is_finite());
    }

    #[test]
    fn ctr_weights_unmeasured_never_outranks_measured() {
        // A weak measured component must still beat the imputed no-data share.
        let shown = [100, 100, 0];
        let engaged = [2, 0, 0];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        assert!(
            mf > emb,
            "2% CTR beats the no-data imputation: mf={mf} emb={emb}"
        );
        assert!(mf > tag);
        assert!((mf + emb + tag - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ctr_weights_partial_data_is_normalized() {
        // Two measured components; the unmeasured one is imputed at their
        // mean CTR and must therefore lose to the stronger measured one.
        let shown = [0, 50, 30];
        let engaged = [0, 20, 1];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        // emb ctr = 22/72 ≈ 0.306, tag ctr = 3/52 ≈ 0.058,
        // mf imputed at the mean ≈ 0.182 → emb must win.
        assert!(
            emb > mf && emb > tag,
            "strongest measured component wins: emb={emb} mf={mf} tag={tag}"
        );
        assert!((mf + emb + tag - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ctr_weights_single_measured_component_ties_with_imputed() {
        // With only one measured component there is no "better" signal: the
        // imputed share equals the measured one, so all three normalize to 1/3.
        let shown = [0, 50, 0];
        let engaged = [0, 20, 0];
        let (mf, emb, tag) = ctr_weights(&shown, &engaged);
        assert!((mf - 1.0 / 3.0).abs() < 1e-9 && (emb - 1.0 / 3.0).abs() < 1e-9);
        assert!((mf + emb + tag - 1.0).abs() < 1e-9);
    }
}
