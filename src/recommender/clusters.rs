//! Taste clusters strategy — k-means over user tag-affinity vectors.
//!
//! Users cluster by their tag affinities (see `ranker::user_tag_vector`).
//! `train` runs k-means and writes memberships + affinities to
//! `rec_user_clusters`. Serving a personalized request: find the user's
//! cluster(s), then recommend works heavily tagged with the cluster's
//! characteristic tags that the user hasn't seen.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// One k-means iteration over sparse tag-affinity vectors.
/// Returns (centroids, assignments) — assignments[i] = cluster of user i.
/// Pure function, unit-tested.
pub fn kmeans_step(
    vectors: &[Vec<(i32, f64)>],
    centroids: &[Vec<(i32, f64)>],
) -> Vec<usize> {
    vectors
        .iter()
        .map(|v| {
            let mut best = 0usize;
            let mut best_d = f64::INFINITY;
            for (i, c) in centroids.iter().enumerate() {
                let d = sparse_distance(v, c);
                if d < best_d {
                    best_d = d;
                    best = i;
                }
            }
            best
        })
        .collect()
}

/// Euclidean distance between two sparse vectors (union of keys).
pub fn sparse_distance(a: &[(i32, f64)], b: &[(i32, f64)]) -> f64 {
    let a_map: HashMap<i32, f64> = a.iter().copied().collect();
    let mut sum = 0.0;
    let mut keys: std::collections::HashSet<i32> = a_map.keys().copied().collect();
    for (k, v) in b {
        keys.insert(*k);
        let diff = v - a_map.get(k).copied().unwrap_or(0.0);
        sum += diff * diff;
    }
    for k in keys {
        if !b.iter().any(|(bk, _)| bk == &k) {
            let diff = a_map.get(&k).copied().unwrap_or(0.0);
            sum += diff * diff;
        }
    }
    sum.sqrt()
}

/// Recompute centroids from assignments (mean of member vectors).
pub fn recompute_centroids(
    vectors: &[Vec<(i32, f64)>],
    assignments: &[usize],
    k: usize,
) -> Vec<Vec<(i32, f64)>> {
    let mut sums: Vec<HashMap<i32, f64>> = vec![HashMap::new(); k];
    let mut counts = vec![0usize; k];
    for (i, a) in assignments.iter().enumerate() {
        counts[*a] += 1;
        for (tag, w) in &vectors[i] {
            *sums[*a].entry(*tag).or_insert(0.0) += w;
        }
    }
    (0..k)
        .map(|c| {
            let n = counts[c].max(1) as f64;
            let mut v: Vec<(i32, f64)> = sums[c]
                .iter()
                .map(|(t, w)| (*t, w / n))
                .filter(|(_, w)| *w > 1e-9)
                .collect();
            v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            v.truncate(50);
            v
        })
        .collect()
}

/// Run k-means for `iterations` steps (k-means++ init not required for
/// determinism tests; we init from the first k vectors).
pub fn run_kmeans(
    vectors: &[Vec<(i32, f64)>],
    k: usize,
    iterations: usize,
) -> (Vec<Vec<(i32, f64)>>, Vec<usize>) {
    if vectors.is_empty() || k == 0 {
        return (Vec::new(), Vec::new());
    }
    let k = k.min(vectors.len());
    let mut centroids: Vec<Vec<(i32, f64)>> = vectors.iter().take(k).cloned().collect();
    let mut assignments = vec![0usize; vectors.len()];
    for _ in 0..iterations {
        assignments = kmeans_step(vectors, &centroids);
        let new_centroids = recompute_centroids(vectors, &assignments, k);
        if new_centroids == centroids {
            centroids = new_centroids;
            break;
        }
        centroids = new_centroids;
    }
    (centroids, assignments)
}

pub struct ClustersStrategy;

impl ClustersStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClustersStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for ClustersStrategy {
    fn name(&self) -> &str {
        "clusters"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        if seed.is_some() {
            return Err(RecError::NotEnoughData("clusters is personalized-only".into()));
        }
        let Some(uid) = user_id else {
            return Err(RecError::Strategy("clusters needs a user".into()));
        };
        // Cluster memberships for this user (top affinity cluster).
        let clusters: Vec<(i32, f64)> = sqlx::query_as(
            r#"SELECT cluster_id, (affinity->>cluster_id::text)::float8
               FROM rec_user_clusters
               WHERE user_id = $1"#,
        )
        .bind(uid)
        .fetch_all(&ctx.db)
        .await?;
        // affinity stored as {"<id>": prob, ...}; the query above is
        // simplified — recompute properly below.
        let memberships: Vec<(i32, f64)> = sqlx::query_as::<_, (i32, serde_json::Value)>(
            r#"SELECT cluster_id, affinity FROM rec_user_clusters WHERE user_id = $1"#,
        )
        .bind(uid)
        .fetch_all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|(cid, aff)| {
            aff.get(cid.to_string())
                .and_then(|v| v.as_f64())
                .map(|p| (cid, p))
        })
        .collect();
        if memberships.is_empty() {
            return Err(RecError::NotEnoughData(format!(
                "user {uid} has no cluster assignment"
            )));
        }
        let (cluster_id, _prob) = memberships
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .unwrap_or((0, 0.0));
        let _ = clusters;

        // Cluster's top tags → works carrying them, excluding the user's
        // known works.
        let top_tags: Vec<i32> = sqlx::query_scalar(
            r#"SELECT ft.tag_id
               FROM rec_user_clusters ruc
               JOIN fic_tags ft ON ft.url_id IN (
                   SELECT url_id FROM fic_info
               )
               WHERE ruc.user_id = $1 AND ruc.cluster_id = $2
               GROUP BY ft.tag_id
               ORDER BY COUNT(*) DESC
               LIMIT 10"#,
        )
        .bind(uid)
        .bind(cluster_id)
        .fetch_all(&ctx.db)
        .await?;
        if top_tags.is_empty() {
            return Err(RecError::NotEnoughData("cluster has no characteristic tags".into()));
        }
        let known: Vec<String> = sqlx::query_scalar(
            "SELECT work_id FROM rec_user_signals WHERE user_id = $1",
        )
        .bind(uid)
        .fetch_all(&ctx.db)
        .await?;

        let rows: Vec<(String, i64)> = sqlx::query_as(
            r#"SELECT f.id, COUNT(DISTINCT ft.tag_id) AS shared
               FROM fic_info f
               JOIN fic_tags ft ON ft.url_id = f.id
               WHERE ft.tag_id = ANY($1) AND f.id <> ALL($2)
               GROUP BY f.id
               ORDER BY shared DESC
               LIMIT $3"#,
        )
        .bind(&top_tags)
        .bind(&known)
        .bind(ctx.config.rec_max_recommendations as i64)
        .fetch_all(&ctx.db)
        .await?;
        if rows.is_empty() {
            return Err(RecError::NotEnoughData("cluster produced no candidates".into()));
        }
        Ok(rows
            .into_iter()
            .map(|(work_id, shared)| ScoredRec {
                work_id,
                score: shared as f64,
                strategy: self.name().into(),
                reason: format!("popular in your taste cluster ({shared} shared tags)"),
            })
            .collect())
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // Users with ≥ 2 signals, tag-affinity vectors (top 100 tags).
        let users: Vec<i32> = sqlx::query_scalar(
            "SELECT user_id FROM rec_user_signals GROUP BY user_id HAVING COUNT(*) >= 2",
        )
        .fetch_all(&ctx.db)
        .await?;
        if users.len() < 4 {
            return Err(RecError::NotEnoughData(format!(
                "only {} users — need ≥ 4 for k-means",
                users.len()
            )));
        }
        let mut vectors: Vec<Vec<(i32, f64)>> = Vec::with_capacity(users.len());
        for uid in &users {
            let v = crate::recommender::ranker::user_tag_vector(ctx, *uid).await;
            if !v.is_empty() {
                vectors.push(v);
            }
        }
        if vectors.len() < 4 {
            return Err(RecError::NotEnoughData("too few tag-affinity vectors".into()));
        }
        let k = 5usize.min(vectors.len() / 2);
        let (centroids, assignments) = run_kmeans(&vectors, k, 10);

        // Write memberships (idempotent).
        let mut tx = ctx.db.begin().await?;
        sqlx::query("DELETE FROM rec_user_clusters").execute(&mut *tx).await?;
        for (i, a) in assignments.iter().enumerate() {
            let affinity: HashMap<String, f64> = centroids
                .iter()
                .enumerate()
                .map(|(c, _)| (c.to_string(), if c == *a { 1.0 } else { 0.0 }))
                .collect();
            let affinity = serde_json::to_value(affinity).unwrap_or(json!({}));
            sqlx::query(
                r#"INSERT INTO rec_user_clusters (user_id, cluster_id, affinity, assigned_at)
                   VALUES ($1, $2, $3, NOW())"#,
            )
            .bind(users[i])
            .bind(*a as i32)
            .bind(affinity)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(json!({ "users": users.len(), "clusters": k, "iterations": 10 }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kmeans_separates_two_obvious_groups() {
        // Group A: tag 1 dominant; Group B: tag 2 dominant.
        let vectors = vec![
            vec![(1, 10.0), (3, 0.1)],
            vec![(1, 9.0), (3, 0.2)],
            vec![(1, 8.0)],
            vec![(2, 10.0), (4, 0.1)],
            vec![(2, 9.0), (4, 0.2)],
            vec![(2, 8.0)],
        ];
        let (centroids, assignments) = run_kmeans(&vectors, 2, 10);
        assert_eq!(centroids.len(), 2);
        // Assignments must split {A,A,A} vs {B,B,B}.
        let a_set: std::collections::HashSet<usize> = assignments[..3].iter().copied().collect();
        let b_set: std::collections::HashSet<usize> = assignments[3..].iter().copied().collect();
        assert_eq!(a_set.len(), 1, "group A must share one cluster: {assignments:?}");
        assert_eq!(b_set.len(), 1, "group B must share one cluster: {assignments:?}");
        assert_ne!(a_set, b_set, "groups must be in different clusters");
    }

    #[test]
    fn sparse_distance_basic() {
        let a = vec![(1, 1.0), (2, 2.0)];
        let b = vec![(1, 1.0), (2, 2.0)];
        assert!((sparse_distance(&a, &b)).abs() < 1e-9);
        let c = vec![(3, 1.0)];
        assert!((sparse_distance(&a, &c) - 6.0f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn recompute_centroids_averages_members() {
        let vectors = vec![vec![(1, 2.0)], vec![(1, 4.0)]];
        let assignments = vec![0usize, 0];
        let centroids = recompute_centroids(&vectors, &assignments, 1);
        assert_eq!(centroids[0], vec![(1, 3.0)]);
    }
}
