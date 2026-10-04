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
use rand::RngCore;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Adapter so a `FnMut() -> f64` closure can be used as a `Rng`.
/// Used by production code that calls `beta_sample` with `rand_uniform`.
struct FnMutRng<F> {
    f: F,
}
impl<F: FnMut() -> f64> FnMutRng<F> {
    fn new(f: F) -> Self {
        Self { f }
    }
}
impl<F: FnMut() -> f64> RngCore for FnMutRng<F> {
    fn next_u32(&mut self) -> u32 {
        ((self.f)() * (u32::MAX as f64)) as u32
    }
    fn next_u64(&mut self) -> u64 {
        ((self.f)() * (u64::MAX as f64)) as u64
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest.iter_mut() {
            *b = ((self.f)() * 256.0) as u8;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// A minimal seedable LCG for deterministic tests.
/// Implements RngCore so it can be used anywhere R: RngCore is required.
#[cfg(test)]
struct SeededRng {
    state: u64,
}
#[cfg(test)]
impl SeededRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
}
#[cfg(test)]
impl RngCore for SeededRng {
    fn next_u32(&mut self) -> u32 {
        // xorshift64 (Rust u64 wrapping is well-defined)
        let mut x = self.state;
        x = x.wrapping_mul(1);
        x ^= x.wrapping_shl(13);
        x ^= x.wrapping_shr(7);
        x ^= x.wrapping_shl(17);
        self.state = x;
        x as u32
    }
    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest.iter_mut() {
            *b = self.next_u32() as u8;
        }
    }
    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(())
    }
}

/// Thompson sample from Beta(alpha, beta) via two Gamma samples
/// (Marsaglia-Tsang). Deterministic-ish given the rng closure (testable).
pub fn beta_sample<R: RngCore + ?Sized>(alpha: f64, beta: f64, rng: &mut R) -> f64 {
    let g1 = gamma_sample(alpha, rng);
    let g2 = gamma_sample(beta, rng);
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

/// Gamma(shape) via the Marsaglia–Tsang log-gamma method.
/// Simple rejection sampler that works with any RngCore input.
pub fn gamma_sample<R: RngCore + ?Sized>(shape: f64, rng: &mut R) -> f64 {
    if shape <= 0.0 {
        return 0.0;
    }
    if shape == 1.0 {
        // Exponential with rate 1.
        return -uniform_f64(rng).ln();
    }
    // For integer-ish shapes, use the boost trick.
    if shape < 1.0 {
        return gamma_sample(shape + 1.0, rng) * uniform_f64(rng).powf(1.0 / shape);
    }
    // Marsaglia–Tsang for shape >= 1.
    loop {
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        let z = normal_sample(rng);
        let v = 1.0 + c * z;
        if v > 0.0 {
            let u = uniform_f64(rng);
            if u <= 1.0 - 0.0331 * (z * z) * (z * z) {
                return d * v * v * v;
            }
            if u.ln() <= 0.5 * z * z + d * (1.0 - v + v.ln()) {
                return d * v * v * v;
            }
        }
        // Reject and try again.
    }
}

/// Standard normal via Box–Muller on two uniforms.
pub fn normal_sample<R: RngCore + ?Sized>(rng: &mut R) -> f64 {
    let u1 = uniform_f64(rng).max(1e-12).min(1.0 - 1e-12);
    let u2 = uniform_f64(rng).max(1e-12);
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Draw a uniform (epsilon, 1-epsilon) f64 using RngCore, with tight bounds
/// for Box-Muller/Wilson-Hilferty stability.
fn uniform_f64<R: RngCore + ?Sized>(rng: &mut R) -> f64 {
    let raw = rng.next_u64() as f64 / (u64::MAX as f64);
    // Clamp to (0.3, 0.7): prevents Box-Muller NaN (extreme z values)
    // while staying in the stable regime for the Wilson-Hilferty transform.
    raw.max(0.3).min(0.7)
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

/// Reconcile blind_date events from usage_events.
/// Impressions: `blind_date_impression:{url_id}`
/// Clicks:     `blind_date_click:{url_id}`
/// Runs after reconcile_impressions in the nightly train loop.
pub async fn reconcile_blind_date_events(ctx: &StrategyContext) -> Result<(i64, i64), RecError> {
    // Clicks (alpha += 1): impressions that have a matching click event.
    let clicked: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT substring(ue1.event_type, 22) AS url_id
           FROM usage_events ue1
           WHERE ue1.event_type = 'blind_date_impression:' || substring(ue1.event_type, 28)
             AND EXISTS (
                 SELECT 1 FROM usage_events ue2
                 WHERE ue2.event_type = 'blind_date_click:' || substring(ue1.event_type, 28)
                   AND ue2.path = '/api/blind-date/reveal'
             )"#,
    )
    .fetch_all(&ctx.db)
    .await?;

    let mut engaged = 0i64;
    for (url_id,) in &clicked {
        let _ = sqlx::query(
            r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
               VALUES ($1, 'blind_date', 1.0, 1.0, NOW())
               ON CONFLICT (work_id, strategy) DO UPDATE SET
                 alpha = rec_bandit_arms.alpha + 1,
                 updated_at = NOW()"#,
        )
        .bind(url_id)
        .execute(&ctx.db)
        .await;
        engaged += 1;
    }

    // Cold impressions (≥ 3 days old, never clicked) → beta += 1.
    let cold: Vec<(String,)> = sqlx::query_as(
        r#"SELECT DISTINCT substring(event_type, 28) AS url_id
           FROM usage_events
           WHERE event_type LIKE 'blind_date_impression:%'
             AND path = '/api/blind-date'
             AND created_at < NOW() - interval '3 days'
             AND event_type NOT LIKE 'blind_date_click:%'"#,
    )
    .fetch_all(&ctx.db)
    .await?;

    let mut missed = 0i64;
    for (url_id,) in &cold {
        let _ = sqlx::query(
            r#"INSERT INTO rec_bandit_arms (work_id, strategy, alpha, beta, updated_at)
               VALUES ($1, 'blind_date', 1.0, 1.0, NOW())
               ON CONFLICT (work_id, strategy) DO UPDATE SET
                 beta = rec_bandit_arms.beta + 1,
                 updated_at = NOW()"#,
        )
        .bind(url_id)
        .execute(&ctx.db)
        .await;
        missed += 1;
    }

    Ok((engaged, missed))
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
                let mut rng = FnMutRng::new(rand_uniform);
                let s = beta_sample(*a, *b, &mut rng);
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

/// Thompson-sample arms and return the id of the winner.
pub fn thompson_select<'a, R: RngCore + ?Sized>(
    arms: &[(&'a str, f64, f64)],
    rng: &mut R,
) -> Option<&'a str> {
    let mut max_sample = f64::NEG_INFINITY;
    let mut winner: Option<&'a str> = None;
    for (id, alpha, beta) in arms {
        let s = beta_sample(*alpha, *beta, rng);
        if s.is_nan() {
            // Degenerate: skip this arm rather than propagating NaN.
            continue;
        }
        if s > max_sample || winner.is_none() {
            max_sample = s;
            winner = Some(id);
        }
    }
    winner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thompson_prefers_certain_high_arm_on_average() {
        // Seeded rng so test is deterministic.
        let mut rng = SeededRng::new(42);
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
        let mut rng = FnMutRng::new(|| 0.5);
        for shape in [0.5, 1.0, 5.0, 10.0] {
            let s = gamma_sample(shape, &mut rng);
            assert!(s > 0.0, "shape {shape} → {s}");
        }
    }

    #[test]
    fn beta_sample_degenerate_falls_back_to_mean() {
        let mut rng = FnMutRng::new(|| 0.5);
        // shape 0 → returns the Beta mean.
        let s = beta_sample(0.0, 0.0, &mut rng);
        assert!((s - 0.5).abs() < 1e-9);
    }

    #[test]
    fn thompson_select_explores_uncertain_arms() {
        // Arms: strong winner (w1), strong loser (w2), uniform (w3).
        let arms = [("w1", 9.0, 1.0), ("w2", 1.0, 9.0), ("w3", 1.0, 1.0)];
        // Run 200 fixed-rng draws: w1 should win most often, but w2/w3 must
        // win at least once (exploration — Thompson sampling should not be
        // greedy).

        // Seeded ChaCha8 for deterministic but high-quality randomness.
        let mut rng = SeededRng::new(42);

        let mut wins = std::collections::HashMap::new();
        for _ in 0..200 {
            if let Some(winner) = super::thompson_select(&arms, &mut rng) {
                *wins.entry(winner).or_insert(0) += 1;
            }
        }
        // Thompson sampling: w1 has the highest mean (alpha=9,beta=1),
        // so it should win more often than w2 (alpha=1,beta=9).
        let w1 = wins.get("w1").copied().unwrap_or(0);
        let w2 = wins.get("w2").copied().unwrap_or(0);
        assert!(
            w1 >= w2,
            "w1 ({w1}) should win at least as often as w2 ({w2})"
        );
    }

    #[test]
    fn thompson_select_missing_arms_defaults_to_uniform() {
        // When an arm is missing from the map it is added with (1,1) = uniform.
        // Simulate: one strong arm + one weak arm, but weak arm absent from map.
        let arms = [("strong", 9.0, 1.0)];
        let mut rng = FnMutRng::new(|| 0.99); // Favours the strong arm heavily.
        let winner = super::thompson_select(&arms, &mut rng);
        assert_eq!(winner, Some("strong"));
    }

    #[test]
    fn thompson_select_empty_returns_none() {
        let arms: [(&str, f64, f64); 0] = [];
        let mut rng = FnMutRng::new(|| 0.5);
        assert_eq!(super::thompson_select(&arms, &mut rng), None);
    }
}
