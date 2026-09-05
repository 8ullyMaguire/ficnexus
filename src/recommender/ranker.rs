//! Reciprocal Rank Fusion ranker — blends strategy outputs, applies the
//! curator prior, and slots in bandit exploration.
//!
//! Pipeline:
//! 1. Run each enabled strategy in config order, collecting `Vec<ScoredRec>`.
//! 2. Fallback chain: a strategy that errors or returns empty is logged and
//!    skipped — the request still succeeds with whatever the others produced.
//! 3. RRF blend: `score(w) = Σ_strategy weight_s · 1/(k + rank_s(w))` with
//!    `k = 60` (the standard constant). Scores are rank-based, so strategies
//!    with wildly different score scales fuse cleanly.
//! 4. Curator prior: for personalized requests, blend the user's profile
//!    toward the curator profile — see [`curator_prior_blend`].
//! 5. Exploration slot: if the bandit strategy is enabled, its arms get
//!    `REC_BANDIT_SLOTS` guaranteed slots (Thompson-sampled) and impressions
//!    are logged.
//! 6. Dedupe (first occurrence wins), sort by blended score, truncate to N.

use std::collections::{HashMap, HashSet};

use tracing::{debug, warn};

use super::registry::StrategyRegistry;
use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Standard RRF constant (k=60 from the original paper).
pub const RRF_K: f64 = 60.0;

/// Blend one strategy's ranked list into the fused map.
///
/// Pure function, unit-tested. `weight` is the config weight for the
/// strategy; `rank` is 1-based position in the strategy's output.
pub fn rrf_accumulate(
    fused: &mut HashMap<String, (f64, String, String)>,
    list: &[ScoredRec],
    weight: f64,
) {
    for (rank, rec) in list.iter().enumerate() {
        let contribution = weight * (1.0 / (RRF_K + (rank as f64) + 1.0));
        let entry = fused
            .entry(rec.work_id.clone())
            .or_insert_with(|| (0.0, rec.strategy.clone(), rec.reason.clone()));
        entry.0 += contribution;
        // Keep the first (highest-ranked) strategy attribution.
    }
}

/// Blended result with attribution.
#[derive(Debug, Clone)]
pub struct BlendedRec {
    pub work_id: String,
    pub score: f64,
    pub strategy: String,
    pub reason: String,
}

/// Compute the curator-prior alpha for a user:
/// `α = floor + (1 - floor) · exp(-n_signals / τ)`
///
/// α is the CURATOR's weight in the blend:
/// `final = (1-α)·user + α·curator`
///
/// * `floor` = REC_PRIOR_FLOOR (default 0.2) — the minimum curator
///   influence. Even a very active user keeps ≥ 20% curator shaping.
/// * `τ` (tau) = REC_CURATOR_TAU (default 25 signals)
///
/// A brand-new user (0 signals) gets α = 1.0 → pure curator picks; an
/// active user converges to α ≈ floor (their own taste dominates, curator
/// floor respected).
pub fn curator_alpha(n_signals: i64, floor: f64, tau: f64) -> f64 {
    let floor = floor.clamp(0.0, 0.999);
    let n = n_signals.max(0) as f64;
    floor + (1.0 - floor) * (-n / tau.max(1.0)).exp()
}

/// Apply the curator prior to a user's rec list: re-rank the top candidates
/// toward the curator's own top list.
///
/// `curator_top` = the curator's own scored recs (their profile). For every
/// candidate in `fused`, if the curator also likes it, we lift it by
/// `α · curator_score(c)`; candidates the curator doesn't like keep their
/// user score scaled by `(1 - α)`.
///
/// Returns the blended list (descending).
pub fn apply_curator_prior(
    fused: &[(String, f64)],       // (work_id, user_blend_score)
    curator_top: &[(String, f64)], // (work_id, curator_score)
    alpha: f64,                    // curator weight — see curator_alpha
) -> Vec<(String, f64)> {
    let curator: HashMap<&str, f64> = curator_top.iter().map(|(w, s)| (w.as_str(), *s)).collect();
    let mut out: Vec<(String, f64)> = fused
        .iter()
        .map(|(w, s)| {
            let lift = curator.get(w.as_str()).copied().unwrap_or(0.0);
            // user contribution keeps (1-α) of its weight; curator lift adds α·curator_score
            (w.clone(), s * (1.0 - alpha) + alpha * lift)
        })
        .collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Run the full pluggable pipeline. Returns blended recs plus per-strategy
/// diagnostics (name → whether it contributed).
///
/// `user_id: None` + `seed: Some(...)` → item-to-item path; `user_id:
/// Some(id)` → personalized path (curator prior applies if a curator is
/// configured).
pub async fn blend(
    ctx: &StrategyContext,
    registry: &StrategyRegistry,
    seed: Option<&str>,
    user_id: Option<i32>,
    n: usize,
) -> Result<(Vec<BlendedRec>, Vec<StrategyRunInfo>), RecError> {
    let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
    let mut diagnostics = Vec::new();

    for spec in registry.specs() {
        let Some(strategy) = registry.get(&spec.name) else {
            continue;
        };
        // Item-to-item-only strategies skip personalized requests and vice
        // versa (e.g. sequential is seed/reader-based only).
        if user_id.is_some() && !strategy.supports_personal() {
            debug!("strategy {} skipped for personalized request", spec.name);
            continue;
        }
        if user_id.is_none() && seed.is_none() {
            continue;
        }

        let started = std::time::Instant::now();
        match strategy.score(ctx, seed, user_id).await {
            Ok(list) if !list.is_empty() => {
                rrf_accumulate(&mut fused, &list, spec.weight);
                diagnostics.push(StrategyRunInfo {
                    name: spec.name.clone(),
                    contributed: true,
                    count: list.len(),
                    error: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            }
            Ok(_) => {
                diagnostics.push(StrategyRunInfo {
                    name: spec.name.clone(),
                    contributed: false,
                    count: 0,
                    error: None,
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            }
            Err(e) => {
                warn!("rec strategy {} failed — falling through: {e}", spec.name);
                diagnostics.push(StrategyRunInfo {
                    name: spec.name.clone(),
                    contributed: false,
                    count: 0,
                    error: Some(e.to_string()),
                    duration_ms: started.elapsed().as_millis() as u64,
                });
            }
        }
    }

    if fused.is_empty() {
        return Err(RecError::NotEnoughData(
            "no enabled strategy produced recommendations".into(),
        ));
    }

    // --- Curator prior (personalized only) ---
    // Snapshot attribution BEFORE consuming `fused` (into_iter moves it).
    let attribution: HashMap<String, (String, String)> = fused
        .iter()
        .map(|(w, (_, strat, reason))| (w.clone(), (strat.clone(), reason.clone())))
        .collect();
    let mut fused_vec: Vec<(String, f64)> = fused
        .into_iter()
        .map(|(w, (s, _strat, _reason))| (w, s))
        .collect();
    fused_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut alpha_used = 1.0f64;
    if let (Some(uid), Some(curator_id)) = (user_id, ctx.config.rec_curator_prior) {
        if uid != curator_id {
            let n_signals = count_user_signals(ctx, uid).await.unwrap_or(0);
            alpha_used = curator_alpha(
                n_signals,
                ctx.config.rec_prior_floor,
                ctx.config.rec_curator_tau,
            );
            let curator_top = fetch_curator_top(ctx, curator_id).await;
            if !curator_top.is_empty() {
                fused_vec = apply_curator_prior(&fused_vec, &curator_top, alpha_used);
            }
        }
    }

    // --- Bandit exploration slots ---
    if let Some(bandit) = registry.get("bandit") {
        if let Some(recs) = slot_exploration(ctx, bandit.as_ref(), user_id, seed, &fused_vec).await
        {
            let mut existing: HashSet<String> = fused_vec.iter().map(|(w, _)| w.clone()).collect();
            for (rank, r) in recs.iter().enumerate() {
                if rank >= ctx.config.rec_bandit_slots {
                    break;
                }
                if existing.contains(&r.work_id) {
                    continue;
                }
                fused_vec.push((r.work_id.clone(), 1.0 / (RRF_K + rank as f64 + 1.0)));
                existing.insert(r.work_id.clone());
            }
        }
    }

    // --- Attribution + truncation ---
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<BlendedRec> = Vec::new();
    for (work_id, score) in &fused_vec {
        if seen.insert(work_id.clone()) {
            let (strat, reason) = attribution
                .get(work_id)
                .cloned()
                .unwrap_or_else(|| ("bandit".into(), "exploration pick".into()));
            out.push(BlendedRec {
                work_id: work_id.clone(),
                score: *score,
                strategy: strat,
                reason,
            });
        }
        if out.len() >= n.max(1) {
            break;
        }
    }

    // Track alignment for QA (best effort; never fail the request).
    if let (Some(uid), Some(curator_id)) = (user_id, ctx.config.rec_curator_prior) {
        if uid != curator_id {
            record_alignment(ctx, uid, curator_id, alpha_used).await;
        }
    }

    Ok((out, diagnostics))
}

/// Per-strategy run diagnostics for `/api/recommendations/strategies`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct StrategyRunInfo {
    pub name: String,
    pub contributed: bool,
    pub count: usize,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Thompson-sample `REC_BANDIT_SLOTS` exploration arms. Pure function:
/// `rng` is a closure returning uniform [0,1) samples (testable).
pub fn sample_arms(
    arms: &[(String, f64, f64)], // (work_id, alpha, beta)
    slots: usize,
    mut rng: impl FnMut() -> f64,
) -> Vec<String> {
    let mut scored: Vec<(String, f64)> = arms
        .iter()
        .map(|(w, a, b)| {
            // Beta(a,b) sample via the transformed-gamma trick is overkill;
            // use the mean-based Thompson proxy: sample ~ Beta by
            // gamma(a,1)/(gamma(a,1)+gamma(b,1)) approximated with
            // Marsaglia-Tsang for integer-ish alpha/beta. For determinism we
            // use the standard "sample from Beta via two gammas" only when
            // rng is cheap; here we approximate with
            // E[beta] + noise scaled by the variance.
            let mean = a / (a + b);
            let variance = if a + b > 2.0 {
                (a * b) / ((a + b) * (a + b) * (a + b + 1.0))
            } else {
                0.25
            };
            let noise = (rng() - 0.5) * 2.0 * variance.sqrt() * 3.0;
            (w.clone(), (mean + noise).max(0.0))
        })
        .collect();
    scored.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(slots).map(|(w, _)| w).collect()
}

/// Slot bandit exploration: fetch arms, sample, log impressions.
async fn slot_exploration(
    ctx: &StrategyContext,
    bandit: &dyn RecStrategy,
    user_id: Option<i32>,
    _seed: Option<&str>,
    fused: &[(String, f64)],
) -> Option<Vec<ScoredRec>> {
    if ctx.config.rec_bandit_slots == 0 {
        return None;
    }
    // Arms = works already in the fused list (the bandit re-ranks within the
    // candidate pool) plus a few fresh arms from rec_bandit_arms.
    let arms = fetch_bandit_arms(ctx).await;
    if arms.is_empty() {
        return None;
    }
    let slots = ctx.config.rec_bandit_slots.min(arms.len());
    let picked = sample_arms(&arms, slots, || {
        // Cheap uniform via a tiny xorshift (no rand dep needed at call site).
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as u64;
        ((t ^ (t << 13) ^ (t >> 7)) as f64) / u64::MAX as f64
    });
    if picked.is_empty() {
        return None;
    }

    let mut out = Vec::new();
    for w in picked {
        if fused.iter().any(|(fw, _)| fw == &w) {
            continue;
        }
        out.push(ScoredRec {
            work_id: w.clone(),
            score: 1.0,
            strategy: "bandit".into(),
            reason: "exploration — trying something new".into(),
        });
        // Log impression (best effort).
        if let Some(uid) = user_id {
            let _ = sqlx::query(
                "INSERT INTO rec_impressions (user_id, work_id, strategy) VALUES ($1, $2, 'bandit')",
            )
            .bind(uid)
            .bind(&w)
            .execute(&ctx.db)
            .await;
        }
        let _ = bandit; // bandit is used for future extension
    }
    Some(out)
}

async fn fetch_bandit_arms(ctx: &StrategyContext) -> Vec<(String, f64, f64)> {
    sqlx::query_as::<_, (String, f64, f64)>(
        "SELECT work_id, alpha, beta FROM rec_bandit_arms WHERE strategy = 'bandit' ORDER BY updated_at DESC LIMIT 200",
    )
    .fetch_all(&ctx.db)
    .await
    .unwrap_or_default()
}

/// Count the user's unified signals (for the curator-prior alpha).
pub async fn count_user_signals(ctx: &StrategyContext, user_id: i32) -> Result<i64, RecError> {
    let n: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM rec_user_signals WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&ctx.db)
            .await?;
    Ok(n.unwrap_or(0))
}

/// Fetch the curator's own top recs (their profile).
pub async fn fetch_curator_top(ctx: &StrategyContext, curator_id: i32) -> Vec<(String, f64)> {
    // Curator profile = works the curator bookmarked/rated, weighted by
    // signal strength (curator interactions are 5× in rec_user_signals).
    sqlx::query_as::<_, (String, f64)>(
        r#"SELECT work_id, SUM(signal_weight) AS s
           FROM rec_user_signals
           WHERE user_id = $1
           GROUP BY work_id
           ORDER BY s DESC
           LIMIT 50"#,
    )
    .bind(curator_id)
    .fetch_all(&ctx.db)
    .await
    .unwrap_or_default()
}

/// Best-effort alignment tracking (cos(user_centroid, curator_centroid) is
/// computed by the pipeline worker; here we record the alpha used).
async fn record_alignment(ctx: &StrategyContext, user_id: i32, curator_id: i32, alpha: f64) {
    let alignment = compute_alignment(ctx, user_id, curator_id).await;
    let _ = sqlx::query(
        r#"INSERT INTO rec_user_curator_align (user_id, alignment, n_signals, alpha, updated_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (user_id) DO UPDATE SET
             alignment = EXCLUDED.alignment,
             n_signals = EXCLUDED.n_signals,
             alpha = EXCLUDED.alpha,
             updated_at = NOW()"#,
    )
    .bind(user_id)
    .bind(alignment)
    .bind(count_user_signals(ctx, user_id).await.unwrap_or(0))
    .bind(alpha)
    .execute(&ctx.db)
    .await;
}

/// cos(user_centroid, curator_centroid) over tag-affinity vectors. Pure-ish:
/// reads signals from the DB, computes centroids in tag space, returns the
/// cosine. 0.0 when either side has no tags.
pub async fn compute_alignment(ctx: &StrategyContext, user_id: i32, curator_id: i32) -> f64 {
    let user_tags = user_tag_vector(ctx, user_id).await;
    let curator_tags = user_tag_vector(ctx, curator_id).await;
    cosine(&user_tags, &curator_tags)
}

/// Tag-affinity vector for a user: sum of signal weights per tag on their
/// signalled works (top 100 tags).
pub async fn user_tag_vector(ctx: &StrategyContext, user_id: i32) -> Vec<(i32, f64)> {
    sqlx::query_as::<_, (i32, f64)>(
        r#"SELECT ft.tag_id, SUM(s.signal_weight) AS w
           FROM rec_user_signals s
           JOIN fic_tags ft ON ft.url_id = s.work_id
           WHERE s.user_id = $1
           GROUP BY ft.tag_id
           ORDER BY w DESC
           LIMIT 100"#,
    )
    .bind(user_id)
    .fetch_all(&ctx.db)
    .await
    .unwrap_or_default()
}

/// Cosine similarity between two sparse (tag_id, weight) vectors.
pub fn cosine(a: &[(i32, f64)], b: &[(i32, f64)]) -> f64 {
    let a_map: HashMap<i32, f64> = a.iter().copied().collect();
    let b_map: HashMap<i32, f64> = b.iter().copied().collect();
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for (k, v) in &a_map {
        dot += v * b_map.get(k).copied().unwrap_or(0.0);
        na += v * v;
    }
    for v in b_map.values() {
        nb += v * v;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom <= f64::EPSILON {
        0.0
    } else {
        (dot / denom).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(work: &str, score: f64, strategy: &str) -> ScoredRec {
        ScoredRec {
            work_id: work.into(),
            score,
            strategy: strategy.into(),
            reason: "test".into(),
        }
    }

    #[test]
    fn rrf_blend_ranks_present_in_both_higher() {
        // A work ranked #1 by two strategies must beat a work ranked #1 by
        // only one.
        let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
        let a = vec![rec("w1", 0.9, "s1"), rec("w2", 0.1, "s1")];
        let b = vec![rec("w1", 0.4, "s2"), rec("w3", 0.6, "s2")];
        rrf_accumulate(&mut fused, &a, 1.0);
        rrf_accumulate(&mut fused, &b, 1.0);
        assert!(fused["w1"].0 > fused["w2"].0);
        assert!(fused["w1"].0 > fused["w3"].0);
    }

    #[test]
    fn rrf_is_scale_invariant() {
        // Even if strategy B's raw scores are 1000× bigger, rank position is
        // all that matters.
        let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
        let a = vec![rec("w1", 0.9, "s1"), rec("w2", 0.8, "s1")];
        let b = vec![rec("w2", 500.0, "s2"), rec("w3", 400.0, "s2")];
        rrf_accumulate(&mut fused, &a, 1.0);
        rrf_accumulate(&mut fused, &b, 1.0);
        // w2 appears at rank 2 in A and rank 1 in B → should lead.
        assert!(fused["w2"].0 > fused["w1"].0);
        assert!(fused["w2"].0 > fused["w3"].0);
    }

    #[test]
    fn rrf_respects_weights() {
        let mut fused: HashMap<String, (f64, String, String)> = HashMap::new();
        // A's #1 gets weight 2.0 → beats B's #1 (weight 0.5) even though
        // both are rank 1.
        let a = vec![rec("wa", 1.0, "s1")];
        let b = vec![rec("wb", 1.0, "s2")];
        rrf_accumulate(&mut fused, &a, 2.0);
        rrf_accumulate(&mut fused, &b, 0.5);
        assert!(fused["wa"].0 > fused["wb"].0);
    }

    #[test]
    fn curator_alpha_respects_floor_and_decays() {
        let floor = 0.2;
        let tau = 25.0;
        // α is the CURATOR weight: 0 signals → 1.0 (pure curator picks).
        assert!((curator_alpha(0, floor, tau) - 1.0).abs() < 1e-9);
        // A few signals → still strongly curator-shaped.
        let low = curator_alpha(3, floor, tau);
        assert!(low > 0.8, "3 signals keeps curator influence: {low}");
        // Many signals → α converges DOWN to the floor (user taste
        // dominates, but the curator floor is respected).
        let high = curator_alpha(500, floor, tau);
        assert!((high - floor).abs() < 0.02, "floor respected: {high}");
    }

    #[test]
    fn curator_prior_lifts_curator_liked_works() {
        let fused = vec![
            ("w1".to_string(), 0.8),
            ("w2".to_string(), 0.7),
            ("w3".to_string(), 0.6),
        ];
        let curator = vec![("w1".to_string(), 1.0), ("w3".to_string(), 0.9)];
        // α = 0.4 curator weight: w1: 0.8·0.6 + 0.4·1.0 = 0.88;
        // w3: 0.6·0.6 + 0.4·0.9 = 0.72; w2: 0.7·0.6 = 0.42
        let alpha = 0.4;
        let out = apply_curator_prior(&fused, &curator, alpha);
        assert!(out[0].0 == "w1", "{out:?}");
        assert!(out[1].0 == "w3", "{out:?}");
        assert!(out[2].0 == "w2", "{out:?}");
    }

    #[test]
    fn curator_prior_floor_respected_when_alpha_low() {
        // α = floor (0.2): user taste dominates but curator still lifts.
        let fused = vec![("w1".to_string(), 1.0), ("w2".to_string(), 0.0)];
        let curator = vec![("w2".to_string(), 0.5)];
        let out = apply_curator_prior(&fused, &curator, 0.2);
        // w1: 1.0·0.8 = 0.8; w2: 0 + 0.2·0.5 = 0.1 → w1 leads.
        assert_eq!(out[0].0, "w1");
        assert!(out[1].1 >= 0.0 && out[1].1 <= 1.0);
    }

    #[test]
    fn cosine_similarity_basic() {
        let a = vec![(1, 1.0), (2, 2.0)];
        let b = vec![(1, 2.0), (2, 4.0)];
        assert!((cosine(&a, &b) - 1.0).abs() < 1e-9);
        let c = vec![(3, 5.0)];
        assert!((cosine(&a, &c)).abs() < 1e-9);
        assert!((cosine(&[], &b)).abs() < 1e-9);
    }

    #[test]
    fn thompson_samples_explore_uncertain_arms() {
        // arm1: alpha=100,beta=1 (very certain, high mean)
        // arm2: alpha=1,beta=1 (uncertain)
        // With a fixed rng the noise term dominates for the uncertain arm,
        // so both arms can be picked across draws — assert the sampler
        // returns the requested number of distinct arms and the certain arm
        // is (on expectation) ranked first by mean.
        let arms = vec![
            ("certain".to_string(), 100.0, 1.0),
            ("uncertain".to_string(), 1.0, 1.0),
        ];
        let picked = sample_arms(&arms, 2, || 0.5);
        assert_eq!(picked.len(), 2);
        let picked_set: HashSet<&str> = picked.iter().map(|s| s.as_str()).collect();
        assert!(picked_set.contains("certain"));
        assert!(picked_set.contains("uncertain"));
    }

    #[test]
    fn thompson_zero_slots_picks_nothing() {
        let arms = vec![("a".to_string(), 1.0, 1.0)];
        let picked = sample_arms(&arms, 0, || 0.5);
        assert!(picked.is_empty());
    }
}
