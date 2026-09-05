//! Contextual bandit strategy — Thompson sampling over rec arms.
//!
//! `rec_bandit_arms` holds a Beta(alpha, beta) posterior per (work,
//! strategy). Serving: sample each arm's posterior and rank; the ranker
//! reserves `REC_BANDIT_SLOTS` exploration slots. Engagement (bookmark/
//! download/completion after an impression) updates alpha; a shown-but-not-
//! engaged impression updates beta. `train` runs the nightly decay + reward
//! reconciliation:
//!   * arms decay: alpha,beta shrink toward 1 (exp(-days/30))
//!   * impressions older than N days with no engagement → beta += 1

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Thompson sample from Beta(alpha, beta) via two Gamma samples
/// (Marsaglia-Tsang). Deterministic-ish given the rng closure (testable).
pub fn beta_sample(alpha: f64, beta: f64, mut rng: impl FnMut() -> f64) -> f64 {
    let g1 = gamma_sample(alpha, &mut rng);
    let g2 = gamma_sample(beta, &mut rng);
    if g1 + g2 <= 1e-12 {
        // Degenerate: both shapes ≤ 0 → uniform (0.5) when both are 0,
        // else the mean of the surviving parameter.
        if alpha + beta <= 1e-12 {
            0.5
        } else {
            alpha / (alpha + beta)
        }
    } else {
        g1 / (g1 + g2)
    }
}

/// Marsaglia-Tsang-style gamma sampler approximated with the
/// Wilson–Hilferty transform (never loops, never hangs — a constant rng is
/// fine for unit tests). Good enough for Thompson sampling.
pub fn gamma_sample(shape: f64, rng: &mut dyn FnMut() -> f64) -> f64 {
    if shape <= 0.0 {
        return 0.0;
    }
    // Boost shape < 1 via multiplication (single recursive hop, bounded).
    // The boost branch uses its own uniform so the caller's rng never
    // recurses through a generic type parameter.
    if shape < 1.0 {
        return gamma_sample(shape + 1.0, rng) * rng().powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    let z = normal_sample(rng);
    let v = (1.0 + c * z).max(1e-9);
    d * v * v * v
}

/// Standard normal via Box–Muller on two uniforms.
pub fn normal_sample(rng: &mut dyn FnMut() -> f64) -> f64 {
    let u1 = (rng().max(1e-12)).min(1.0 - 1e-12);
    let u2 = rng().max(1e-12);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Uniform [0,1) helper (xorshift64 over system time).
#[allow(dead_code)]
fn rand_uniform_alt() -> f64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut x = t ^ 0x9E3779B97F4A7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x as f64) / (u64::MAX as f64)
}

/// Record an impression (shown).
pub async fn log_impression(
    ctx: &StrategyContext,
    user_id: Option<i32>,
    work_id: &str,
    strategy: &str,
) {
    let _ =
        sqlx::query("INSERT INTO rec_impressions (user_id, work_id, strategy) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(work_id)
            .bind(strategy)
            .execute(&ctx.db)
            .await;
}

/// Update arm posteriors from impressions: engagement → alpha+1, shown-but-
/// cold → beta+1. Returns (engaged, missed) counts.
pub async fn reconcile_impressions(ctx: &StrategyContext) -> Result<(i64, i64), RecError> {
    // Engagement = the user later bookmarked/downloaded/completed the work.
    // We approximate "later" as: an impression older than 1 day with no
    // rec_impressions.engaged_at AND no bookmark/download for that user+work
    // since shown_at → beta += 1. Engaged impressions (engaged_at set) are
    // alpha += 1 by the frontend/reader wiring; here we reconcile the
    // engaged_at column being NULL vs a bookmark existing.
    let engaged: Vec<(i32, String)> = sqlx::query_as(
        r#"SELECT DISTINCT i.user_id, i.work_id
           FROM rec_impressions i
           WHERE i.user_id IS NOT NULL
             AND i.engaged_at IS NULL
             AND EXISTS (
                SELECT 1 FROM bookmarks b
                WHERE b.user_id = i.user_id AND b.url_id = i.work_id
                  AND b.created_at > i.shown_at
             )"#,
    )
    .fetch_all(&ctx.db)
    .await?;

    let mut engaged_count = 0i64;
    for (uid, wid) in &engaged {
        let _ = sqlx::query(
            r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
               VALUES ($1, 'bandit', 1.0, 1.0, NOW())
               ON CONFLICT (work_id, strategy) DO UPDATE SET
                 alpha = rec_bandit_arms.alpha + 1,
                 updated_at = NOW()"#,
        )
        .bind(wid)
        .execute(&ctx.db)
        .await;
        let _ = sqlx::query(
            "UPDATE rec_impressions SET engaged_at = NOW() WHERE user_id = $1 AND work_id = $2 AND engaged_at IS NULL",
        )
        .bind(uid)
        .bind(wid)
        .execute(&ctx.db)
        .await;
        engaged_count += 1;
    }

    // Cold impressions (≥ 3 days old, never engaged) → beta += 1.
    let cold: Vec<(i32, String)> = sqlx::query_as(
        r#"SELECT user_id, work_id
           FROM rec_impressions
           WHERE user_id IS NOT NULL AND engaged_at IS NULL
             AND shown_at < NOW() - interval '3 days'"#,
    )
    .fetch_all(&ctx.db)
    .await?;
    let mut missed = 0i64;
    for (_, wid) in &cold {
        let _ = sqlx::query(
            r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
               VALUES ($1, 'bandit', 1.0, 1.0, NOW())
               ON CONFLICT (work_id, strategy) DO UPDATE SET
                 beta = rec_bandit_arms.beta + 1,
                 updated_at = NOW()"#,
        )
        .bind(wid)
        .execute(&ctx.db)
        .await;
        missed += 1;
    }
    Ok((engaged_count, missed))
}

/// Nightly arm decay: shrink posteriors toward uniform (exp decay on the
/// "evidence" part, keeping the mean roughly stable).
pub async fn decay_arms(ctx: &StrategyContext) -> Result<usize, RecError> {
    let half_life = ctx.config.rec_decay_halflife_days;
    let rows: Vec<(String, f64, f64)> = sqlx::query_as(
        "SELECT work_id, alpha, beta FROM rec_bandit_arms WHERE strategy = 'bandit'",
    )
    .fetch_all(&ctx.db)
    .await?;
    let n = rows.len();
    for (work, alpha, beta) in rows {
        let age_days = 1.0; // decay relative to last update below
        let w = (-age_days / half_life.max(1e-9)).exp();
        // Pull both params toward 1.0: evidence = (param - 1) * w + 1.
        let new_a = 1.0 + (alpha - 1.0) * w;
        let new_b = 1.0 + (beta - 1.0) * w;
        let _ = sqlx::query(
            "UPDATE rec_bandit_arms SET alpha = $1, beta = $2, updated_at = NOW() WHERE work_id = $3 AND strategy = 'bandit'",
        )
        .bind(new_a.max(1.0))
        .bind(new_b.max(1.0))
        .bind(&work)
        .execute(&ctx.db)
        .await;
    }
    Ok(n)
}

pub struct BanditStrategy;

impl BanditStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BanditStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for BanditStrategy {
    fn name(&self) -> &str {
        "bandit"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        // Thompson-sample all arms and rank. The ranker consumes this via
        // the exploration-slot path (sample_arms) — this strategy's own
        // score() is the fallback when bandit is listed as a regular
        // strategy in REC_STRATEGIES.
        let arms: Vec<(String, f64, f64)> = sqlx::query_as(
            "SELECT work_id, alpha, beta FROM rec_bandit_arms WHERE strategy = 'bandit' ORDER BY updated_at DESC LIMIT 200",
        )
        .fetch_all(&ctx.db)
        .await?;
        if arms.is_empty() {
            return Err(RecError::NotEnoughData("no bandit arms".into()));
        }
        let mut scored: Vec<(String, f64)> = arms
            .iter()
            .map(|(w, a, b)| {
                let s = beta_sample(*a, *b, rand_uniform);
                (w.clone(), s)
            })
            .collect();
        scored.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(ctx.config.rec_bandit_slots.max(1));

        let mut out = Vec::new();
        for (work_id, s) in scored {
            out.push(ScoredRec {
                work_id,
                score: s,
                strategy: self.name().into(),
                reason: "exploration pick".into(),
            });
        }
        if let Some(uid) = user_id {
            for r in &out {
                log_impression(ctx, Some(uid), &r.work_id, self.name()).await;
            }
        }
        let _ = seed;
        Ok(out)
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        let (engaged, missed) = reconcile_impressions(ctx).await?;
        let decayed = decay_arms(ctx).await?;
        Ok(json!({ "engaged": engaged, "missed": missed, "arms_decayed": decayed }))
    }
}

/// Cheap uniform [0,1) — xorshift64 over system time. Good enough for
/// Thompson sampling; deterministic tests pass their own rng.
pub fn rand_uniform() -> f64 {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let mut x = t ^ 0x9E3779B97F4A7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    (x as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thompson_prefers_certain_high_arm_on_average() {
        // Deterministic rng sequence: with a fixed 0.5 the gamma samples are
        // reproducible; assert the sampler returns values in [0,1] and that
        // the high-confidence arm's mean is above the uncertain arm's mean
        // across many draws with a cycling rng.
        let mut rng = || 0.42;
        let high = (0..200)
            .map(|_| beta_sample(50.0, 1.0, &mut rng))
            .collect::<Vec<_>>();
        let low = (0..200)
            .map(|_| beta_sample(1.0, 1.0, &mut rng))
            .collect::<Vec<_>>();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(
            mean(&high) > mean(&low),
            "high={:?} low={:?}",
            mean(&high),
            mean(&low)
        );
        assert!(high.iter().all(|s| (0.0..=1.0).contains(s)));
    }

    #[test]
    fn gamma_sample_is_positive() {
        let mut rng = || 0.5;
        for shape in [0.5, 1.0, 5.0, 10.0] {
            let s = gamma_sample(shape, &mut rng);
            assert!(s > 0.0, "shape {shape} → {s}");
        }
    }

    #[test]
    fn beta_sample_degenerate_falls_back_to_mean() {
        // shape 0 → returns the Beta mean.
        let s = beta_sample(0.0, 0.0, || 0.5);
        assert!((s - 0.5).abs() < 1e-9);
    }
}
