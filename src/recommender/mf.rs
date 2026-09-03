//! Implicit matrix factorization (ALS) — Hu–Koren implicit-confidence ALS.
//!
//! Cargo feature `rec-mf` gates this module (default OFF so the binary
//! builds without the heavy linear-algebra dependency). When the feature is
//! off, the strategy registers a stub that returns "not enough data" — the
//! registry still lists it, so configs mentioning `mf` don't crash.
//!
//! Implementation: hand-rolled ALS over a sparse (user, work, confidence)
//! matrix built from `rec_user_signals`. No ndarray — dependency-light, ~150
//! lines. Factors are persisted to disk (params_path) and registered in
//! `rec_models`; serving is a dot product.


use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// One implicit-feedback observation: confidence = 1 + α·r_ui.
#[derive(Debug, Clone)]
pub struct MfObservation {
    pub user: usize,
    pub work: usize,
    pub confidence: f64,
}

/// Sparse implicit-confidence matrix.
#[derive(Debug, Clone)]
pub struct MfMatrix {
    pub n_users: usize,
    pub n_works: usize,
    /// user → (work, confidence)
    pub user_items: Vec<Vec<(usize, f64)>>,
    /// work → (user, confidence)
    pub item_users: Vec<Vec<(usize, f64)>>,
}

impl MfMatrix {
    pub fn new(n_users: usize, n_works: usize) -> Self {
        Self {
            n_users,
            n_works,
            user_items: vec![Vec::new(); n_users],
            item_users: vec![Vec::new(); n_works],
        }
    }

    pub fn push(&mut self, obs: MfObservation) {
        self.user_items[obs.user].push((obs.work, obs.confidence));
        self.item_users[obs.work].push((obs.user, obs.confidence));
    }

    pub fn is_empty(&self) -> bool {
        self.user_items.iter().all(|v| v.is_empty())
    }
}

/// Train implicit ALS. Returns (user_factors, item_factors), each
/// `[entity][factor]`.
///
/// Standard Hu–Koren alternating least squares with ridge regularization:
///   Xu = (Yᵀ·Cu·Y + λI)⁻¹ · Yᵀ·Cu·pu
/// solved per user with the "all items minus the user's" trick.
pub fn train_als(
    matrix: &MfMatrix,
    factors: usize,
    iterations: usize,
    _alpha: f64,
    lambda: f64,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut user_factors: Vec<Vec<f64>> = (0..matrix.n_users)
        .map(|i| {
            (0..factors)
                .map(|f| ((i * 7919 + f * 104729) % 1000) as f64 / 1000.0 - 0.5)
                .collect()
        })
        .collect();
    let mut item_factors: Vec<Vec<f64>> = (0..matrix.n_works)
        .map(|i| {
            (0..factors)
                .map(|f| ((i * 104729 + f * 7919) % 1000) as f64 / 1000.0 - 0.5)
                .collect()
        })
        .collect();

    // Precompute item-item Gram matrix: G = YᵀY.
    let mut gram = vec![vec![0.0f64; factors]; factors];
    for item in &item_factors {
        for i in 0..factors {
            for j in 0..factors {
                gram[i][j] += item[i] * item[j];
            }
        }
    }

    for _ in 0..iterations {
        // ── User step ──
        for (u, items) in matrix.user_items.iter().enumerate() {
            if items.is_empty() {
                continue;
            }
            // G_u = G + Σ_i (c_ui - 1) y_i y_iᵀ + λI
            let mut a = gram.clone();
            for (i, c) in items {
                let y = &item_factors[*i];
                for r in 0..factors {
                    for c2 in 0..factors {
                        a[r][c2] += (c - 1.0) * y[r] * y[c2];
                    }
                }
            }
            for d in 0..factors {
                a[d][d] += lambda;
            }
            // b = Σ_i c_ui y_i
            let mut b = vec![0.0f64; factors];
            for (i, c) in items {
                let y = &item_factors[*i];
                for r in 0..factors {
                    b[r] += c * y[r];
                }
            }
            // Solve a·x = b (Gaussian elimination with partial pivoting).
            if let Some(x) = solve_linear(&a, &b) {
                user_factors[u] = x;
            }
        }

        // ── Item step (symmetric) ──
        // Recompute the user Gram matrix.
        let mut ugram = vec![vec![0.0f64; factors]; factors];
        for user in &user_factors {
            for i in 0..factors {
                for j in 0..factors {
                    ugram[i][j] += user[i] * user[j];
                }
            }
        }
        for (i, users) in matrix.item_users.iter().enumerate() {
            if users.is_empty() {
                continue;
            }
            let mut a = ugram.clone();
            for (u, c) in users {
                let x = &user_factors[*u];
                for r in 0..factors {
                    for c2 in 0..factors {
                        a[r][c2] += (c - 1.0) * x[r] * x[c2];
                    }
                }
            }
            for d in 0..factors {
                a[d][d] += lambda;
            }
            let mut b = vec![0.0f64; factors];
            for (u, c) in users {
                let x = &user_factors[*u];
                for r in 0..factors {
                    b[r] += c * x[r];
                }
            }
            if let Some(x) = solve_linear(&a, &b) {
                item_factors[i] = x;
            }
        }
    }

    (user_factors, item_factors)
}

/// Solve a·x = b by Gaussian elimination with partial pivoting.
/// Returns None when singular (caller keeps the previous factors).
fn solve_linear(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = a.len();
    if n == 0 {
        return Some(Vec::new());
    }
    let mut m = a.to_vec();
    for (i, row) in m.iter_mut().enumerate() {
        row.push(b[i]);
    }
    for col in 0..n {
        // Partial pivot.
        let mut pivot = col;
        for r in (col + 1)..n {
            if m[r][col].abs() > m[pivot][col].abs() {
                pivot = r;
            }
        }
        if m[pivot][col].abs() < 1e-12 {
            return None;
        }
        m.swap(col, pivot);
        let div = m[col][col];
        for c in col..=n {
            m[col][c] /= div;
        }
        for r in 0..n {
            if r != col {
                let factor = m[r][col];
                if factor.abs() < 1e-15 {
                    continue;
                }
                for c in col..=n {
                    m[r][c] -= factor * m[col][c];
                }
            }
        }
    }
    Some((0..n).map(|i| m[i][n]).collect())
}

/// Dot product of two factor vectors.
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Strategy. When `rec-mf` is off this still compiles: `train` records a
/// no-op run and `score` returns NotEnoughData (configs mentioning `mf`
/// degrade gracefully).
pub struct MfStrategy;

impl MfStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MfStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Latest registered MF model (name, version, params_path, metrics).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MfModelRow {
    pub name: String,
    pub version: String,
    pub params_path: Option<String>,
    pub metrics: serde_json::Value,
}

/// Load the latest MF model factors from disk. Returns (user_factors,
/// item_factors, index maps) or None when no model exists.
pub fn load_factors(
    path: &str,
) -> Option<(
    Vec<Vec<f64>>,
    Vec<Vec<f64>>,
    Vec<String>,
    Vec<i32>,
)> {
    let data = std::fs::read_to_string(path).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&data).ok()?;
    let user_factors: Vec<Vec<f64>> =
        serde_json::from_value(parsed.get("user_factors")?.clone()).ok()?;
    let item_factors: Vec<Vec<f64>> =
        serde_json::from_value(parsed.get("item_factors")?.clone()).ok()?;
    let work_ids: Vec<String> = serde_json::from_value(parsed.get("work_ids")?.clone()).ok()?;
    let user_ids: Vec<i32> = serde_json::from_value(parsed.get("user_ids")?.clone()).ok()?;
    Some((user_factors, item_factors, work_ids, user_ids))
}

/// Persist factors + index maps to disk (JSON) and register in rec_models.
pub fn save_factors(
    path: &str,
    user_factors: &[Vec<f64>],
    item_factors: &[Vec<f64>],
    work_ids: &[String],
    user_ids: &[i32],
    metrics: serde_json::Value,
) -> std::io::Result<()> {
    let payload = json!({
        "user_factors": user_factors,
        "item_factors": item_factors,
        "work_ids": work_ids,
        "user_ids": user_ids,
        "metrics": metrics,
    });
    std::fs::write(path, serde_json::to_string_pretty(&payload)?)
}

#[cfg(feature = "rec-mf")]
#[async_trait]
impl RecStrategy for MfStrategy {
    fn name(&self) -> &str {
        "mf"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        _seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        let Some(uid) = user_id else {
            return Err(RecError::NotEnoughData("mf is personalized-only".into()));
        };
        let model: Option<MfModelRow> = sqlx::query_as(
            r#"SELECT name, version, params_path, metrics
               FROM rec_models WHERE name = 'mf-als' ORDER BY trained_at DESC LIMIT 1"#,
        )
        .fetch_optional(&ctx.db)
        .await?;
        let Some(model) = model else {
            return Err(RecError::NotEnoughData("no trained MF model".into()));
        };
        let Some(path) = model.params_path else {
            return Err(RecError::NotEnoughData("MF model has no factors file".into()));
        };
        let Some((_uf, item_factors, work_ids, user_ids)) = load_factors(&path) else {
            return Err(RecError::Strategy("MF factors unreadable".into()));
        };
        let Some(user_pos) = user_ids.iter().position(|u| *u == uid) else {
            return Err(RecError::NotEnoughData(format!(
                "user {uid} not in MF training set"
            )));
        };
        // Need user factors too: reload full file.
        let data = std::fs::read_to_string(&path).map_err(|e| RecError::Strategy(e.to_string()))?;
        let parsed: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| RecError::Strategy(e.to_string()))?;
        let user_factors: Vec<Vec<f64>> = serde_json::from_value(parsed["user_factors"].clone())
            .map_err(|e| RecError::Strategy(e.to_string()))?;
        let uf = &user_factors[user_pos];

        let mut scored: Vec<(String, f64)> = item_factors
            .iter()
            .zip(work_ids.iter())
            .map(|(item, wid)| (wid.clone(), dot(uf, item)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(ctx.config.rec_max_recommendations);

        if scored.is_empty() {
            return Err(RecError::NotEnoughData("MF produced no recs".into()));
        }
        Ok(scored
            .into_iter()
            .map(|(work_id, score)| ScoredRec {
                work_id,
                score,
                strategy: self.name().into(),
                reason: "collaborative taste match (MF)".into(),
            })
            .collect())
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // Load all signals, build the implicit-confidence matrix.
        let signals = crate::recommender::signals::all_signals(&ctx.db).await?;
        if signals.len() < ctx.config.rec_mf_train_min_signals {
            return Err(RecError::NotEnoughData(format!(
                "only {} signals — need {}",
                signals.len(),
                ctx.config.rec_mf_train_min_signals
            )));
        }
        let mut user_ids: Vec<i32> = Vec::new();
        let mut work_ids: Vec<String> = Vec::new();
        let mut user_index: HashMap<i32, usize> = HashMap::new();
        let mut work_index: HashMap<String, usize> = HashMap::new();
        for s in &signals {
            let u = *user_index.entry(s.user_id).or_insert_with(|| {
                user_ids.push(s.user_id);
                user_ids.len() - 1
            });
            let w = *work_index.entry(s.work_id.clone()).or_insert_with(|| {
                work_ids.push(s.work_id.clone());
                work_ids.len() - 1
            });
            let _ = (u, w);
        }
        let mut matrix = MfMatrix::new(user_ids.len(), work_ids.len());
        for s in &signals {
            let u = user_index[&s.user_id];
            let w = work_index[&s.work_id];
            matrix.push(MfObservation {
                user: u,
                work: w,
                confidence: 1.0 + 40.0 * s.signal_weight.max(0.0), // α = 40
            });
        }
        let (uf, wf) = train_als(
            &matrix,
            ctx.config.rec_mf_factors,
            ctx.config.rec_mf_iters,
            40.0,
            0.1,
        );
        let path = format!(
            "{}/rec-mf-{}.json",
            ctx.config.cache_dir.display(),
            ctx.now.format("%Y%m%d-%H%M%S")
        );
        let metrics = json!({
            "users": user_ids.len(),
            "works": work_ids.len(),
            "signals": signals.len(),
            "factors": ctx.config.rec_mf_factors,
            "iterations": ctx.config.rec_mf_iters,
        });
        save_factors(&path, &uf, &wf, &work_ids, &user_ids, metrics.clone())
            .map_err(|e| RecError::Strategy(format!("save factors: {e}")))?;
        sqlx::query(
            r#"INSERT INTO rec_models (name, version, params_path, trained_at, metrics)
               VALUES ('mf-als', '1', $1, NOW(), $2)
               ON CONFLICT (name, version) DO UPDATE SET
                 params_path = EXCLUDED.params_path,
                 trained_at = NOW(),
                 metrics = EXCLUDED.metrics"#,
        )
        .bind(&path)
        .bind(metrics.clone())
        .execute(&ctx.db)
        .await?;
        Ok(metrics)
    }
}

/// Stub implementation when `rec-mf` is off: always "not enough data", so
/// the registry still resolves `mf` but it contributes nothing.
#[cfg(not(feature = "rec-mf"))]
#[async_trait]
impl RecStrategy for MfStrategy {
    fn name(&self) -> &str {
        "mf"
    }

    async fn score(
        &self,
        _ctx: &StrategyContext,
        _seed: Option<&str>,
        _user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        Err(RecError::NotEnoughData(
            "MF strategy disabled — build with --features rec-mf".into(),
        ))
    }

    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        Err(RecError::NotEnoughData(
            "MF strategy disabled — build with --features rec-mf".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn als_recovers_block_structure() {
        // 3 users, 3 works: users 0/1 like works 0/1; user 2 likes work 2.
        // After ALS, the dot products for liked pairs must exceed non-liked.
        let mut matrix = MfMatrix::new(3, 3);
        for u in 0..2 {
            for w in 0..2 {
                matrix.push(MfObservation { user: u, work: w, confidence: 2.0 });
            }
        }
        matrix.push(MfObservation { user: 2, work: 2, confidence: 2.0 });
        let (uf, wf) = train_als(&matrix, 8, 20, 40.0, 0.1);
        let score = |u: usize, w: usize| dot(&uf[u], &wf[w]);
        // Liked pairs score higher than the user's non-liked pair.
        assert!(score(0, 0) > score(0, 2), "u0/w0 should beat u0/w2");
        assert!(score(1, 1) > score(1, 2), "u1/w1 should beat u1/w2");
        assert!(score(2, 2) > score(2, 0), "u2/w2 should beat u2/w0");
    }

    #[test]
    fn solve_linear_small_system() {
        // 2x + y = 5 ; x + 2y = 4  → x=2, y=1
        let a = vec![vec![2.0, 1.0], vec![1.0, 2.0]];
        let b = vec![5.0, 4.0];
        let x = solve_linear(&a, &b).expect("nonsingular");
        assert!((x[0] - 2.0).abs() < 1e-9, "x={x:?}");
        assert!((x[1] - 1.0).abs() < 1e-9, "x={x:?}");
    }

    #[test]
    fn solve_linear_singular_returns_none() {
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let b = vec![1.0, 2.0];
        assert!(solve_linear(&a, &b).is_none());
    }

    #[test]
    fn factors_roundtrip_disk() {
        let dir = std::env::temp_dir();
        let path = dir.join("rec-mf-test.json");
        let path_s = path.to_str().unwrap().to_string();
        save_factors(
            &path_s,
            &[vec![0.1, 0.2]],
            &[vec![0.3, 0.4]],
            &["w1".into()],
            &[7],
            json!({"users": 1}),
        )
        .expect("save");
        let loaded = load_factors(&path_s).expect("load");
        assert_eq!(loaded.2, vec!["w1".to_string()]);
        assert_eq!(loaded.3, vec![7]);
        let _ = std::fs::remove_file(&path);
    }
}
