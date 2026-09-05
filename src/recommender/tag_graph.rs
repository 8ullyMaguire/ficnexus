//! Tag knowledge-graph strategy — meta-path traversal over the tag graph.
//!
//! Tags form a knowledge graph via their co-occurrence on works:
//! fandom → character → relationship → freeform, weighted by the tag score
//! on each work (main char = 10, primary ship = 5 — the fic_tags score).
//!
//! Scoring a seed work:
//!   1. Collect the seed's tags with their scores.
//!   2. Walk meta-paths of length ≤ 2 through the tag graph:
//!      seed-tag → shared-work-tag → candidate-work-tag.
//!   3. Score each candidate by the product of tag scores along the path,
//!      summed over all paths (a soft random-walk with restart flavor).
//!
//! Pure Rust over a tag→work→tag incidence fetched from SQL; unit-tested
//! with a small synthetic graph.

use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// A meta-path walk result: (work_id, accumulated_score, reason_tag).
#[derive(Debug, Clone, PartialEq)]
pub struct PathWalk {
    pub work_id: String,
    pub score: f64,
    pub via: String,
}

/// Score candidates by tag meta-path traversal.
///
/// `seed_tags`: (tag_id, score) of the seed work.
/// `tag_works`: tag_id → [(work_id, tag_score)] incidence.
/// `work_tags`: work_id → [(tag_id, tag_score)].
/// Returns candidates with scores, excluding the seed work, sorted desc.
pub fn meta_path_score(
    seed_tags: &[(i32, f64)],
    tag_works: &HashMap<i32, Vec<(String, f64)>>,
    work_tags: &HashMap<String, Vec<(i32, f64)>>,
    seed_work: &str,
    max_candidates: usize,
) -> Vec<PathWalk> {
    let mut scores: HashMap<String, (f64, String)> = HashMap::new();

    for (seed_tag, seed_score) in seed_tags {
        // Direct neighbours: works sharing this tag.
        if let Some(neighbours) = tag_works.get(seed_tag) {
            for (work, work_tag_score) in neighbours {
                if work == seed_work {
                    continue;
                }
                let s = seed_score * work_tag_score;
                let entry = scores
                    .entry(work.clone())
                    .or_insert((0.0, format!("tag {seed_tag}")));
                entry.0 += s;
            }
        }
        // Two-hop: works sharing a tag with a work that shares seed_tag.
        if let Some(neighbours) = tag_works.get(seed_tag) {
            for (intermediate, _) in neighbours {
                if intermediate == seed_work {
                    continue;
                }
                if let Some(inter_tags) = work_tags.get(intermediate) {
                    for (itag, _) in inter_tags {
                        if let Some(second_hop) = tag_works.get(itag) {
                            for (work, work_tag_score) in second_hop {
                                if work == seed_work || work == intermediate {
                                    continue;
                                }
                                let s = seed_score * work_tag_score * 0.5; // path-length discount
                                let entry = scores
                                    .entry(work.clone())
                                    .or_insert((0.0, format!("tag {itag}")));
                                entry.0 += s;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut out: Vec<PathWalk> = scores
        .into_iter()
        .map(|(work_id, (score, via))| PathWalk {
            work_id,
            score,
            via,
        })
        .collect();
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out.truncate(max_candidates);
    out
}

pub struct TagGraphStrategy;

impl TagGraphStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TagGraphStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Load the tag→work incidence and work→tag incidence from SQL.
pub async fn load_tag_graph(
    ctx: &StrategyContext,
) -> Result<
    (
        HashMap<i32, Vec<(String, f64)>>,
        HashMap<String, Vec<(i32, f64)>>,
    ),
    RecError,
> {
    let rows: Vec<(String, i32, f64)> = sqlx::query_as(
        r#"SELECT ft.url_id, ft.tag_id, ft.score::float8
           FROM fic_tags ft
           WHERE ft.score > 0"#,
    )
    .fetch_all(&ctx.db)
    .await?;
    let mut tag_works: HashMap<i32, Vec<(String, f64)>> = HashMap::new();
    let mut work_tags: HashMap<String, Vec<(i32, f64)>> = HashMap::new();
    for (work, tag, score) in rows {
        tag_works
            .entry(tag)
            .or_default()
            .push((work.clone(), score));
        work_tags.entry(work).or_default().push((tag, score));
    }
    Ok((tag_works, work_tags))
}

#[async_trait]
impl RecStrategy for TagGraphStrategy {
    fn name(&self) -> &str {
        "tag_graph"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        if user_id.is_some() {
            return Err(RecError::NotEnoughData(
                "tag_graph is seed-based only".into(),
            ));
        }
        let Some(seed_id) = seed else {
            return Err(RecError::Strategy("tag_graph needs a seed work".into()));
        };

        // Seed tags with scores.
        let seed_tags: Vec<(i32, f64)> = sqlx::query_as(
            "SELECT tag_id, score::float8 FROM fic_tags WHERE url_id = $1 AND score > 0",
        )
        .bind(seed_id)
        .fetch_all(&ctx.db)
        .await?;
        if seed_tags.is_empty() {
            return Err(RecError::NotEnoughData(format!(
                "seed {seed_id} has no scored tags"
            )));
        }

        let (tag_works, work_tags) = load_tag_graph(ctx).await?;
        let walks = meta_path_score(
            &seed_tags,
            &tag_works,
            &work_tags,
            seed_id,
            ctx.config.rec_max_recommendations,
        );
        if walks.is_empty() {
            return Err(RecError::NotEnoughData(
                "tag graph produced no candidates".into(),
            ));
        }
        Ok(walks
            .into_iter()
            .map(|w| ScoredRec {
                work_id: w.work_id,
                score: w.score,
                strategy: self.name().into(),
                reason: format!("tag path via {}", w.via),
            })
            .collect())
    }

    async fn train(&self, _ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        // Traversal is computed live from fic_tags; nothing to precompute.
        Ok(json!({ "note": "tag graph traversal is computed live" }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tiny synthetic graph:
    /// seed work S has tags [F(10), C(5)]
    /// work A shares F (score 10) — strong
    /// work B shares C (score 5) — weaker
    /// work D shares a two-hop tag with S (via A's extra tag X)
    #[test]
    fn meta_path_ranks_direct_above_two_hop() {
        let seed_tags = vec![(1, 10.0), (2, 5.0)];
        let mut tag_works: HashMap<i32, Vec<(String, f64)>> = HashMap::new();
        tag_works.insert(1, vec![("S".into(), 10.0), ("A".into(), 10.0)]);
        tag_works.insert(2, vec![("S".into(), 5.0), ("B".into(), 5.0)]);
        tag_works.insert(3, vec![("A".into(), 2.0), ("D".into(), 9.0)]);
        let mut work_tags: HashMap<String, Vec<(i32, f64)>> = HashMap::new();
        work_tags.insert("S".into(), vec![(1, 10.0), (2, 5.0)]);
        work_tags.insert("A".into(), vec![(1, 10.0), (3, 2.0)]);
        work_tags.insert("B".into(), vec![(2, 5.0)]);
        work_tags.insert("D".into(), vec![(3, 9.0)]);

        let walks = meta_path_score(&seed_tags, &tag_works, &work_tags, "S", 10);
        assert!(!walks.is_empty());
        // A (direct, 10·10=100) must beat B (direct, 5·5=25) and D (two-hop, 10·2·9·0.5=90).
        assert_eq!(walks[0].work_id, "A", "{walks:?}");
        assert!(walks.iter().any(|w| w.work_id == "B"));
        assert!(walks.iter().any(|w| w.work_id == "D"));
        let a = walks.iter().find(|w| w.work_id == "A").unwrap();
        let d = walks.iter().find(|w| w.work_id == "D").unwrap();
        assert!(a.score > d.score, "{walks:?}");
        // Seed never appears.
        assert!(!walks.iter().any(|w| w.work_id == "S"));
    }

    #[test]
    fn meta_path_excludes_seed_and_intermediate() {
        let seed_tags = vec![(1, 1.0)];
        let mut tag_works: HashMap<i32, Vec<(String, f64)>> = HashMap::new();
        tag_works.insert(
            1,
            vec![("S".into(), 1.0), ("I".into(), 1.0), ("C".into(), 1.0)],
        );
        let mut work_tags: HashMap<String, Vec<(i32, f64)>> = HashMap::new();
        work_tags.insert("S".into(), vec![(1, 1.0)]);
        work_tags.insert("I".into(), vec![(1, 1.0)]);
        work_tags.insert("C".into(), vec![(1, 1.0)]);
        let walks = meta_path_score(&seed_tags, &tag_works, &work_tags, "S", 10);
        let ids: Vec<&str> = walks.iter().map(|w| w.work_id.as_str()).collect();
        assert!(!ids.contains(&"S"), "seed must be excluded: {ids:?}");
        // C shares tag 1 directly (score 1.0) AND via the two-hop through I
        // (+0.5) → C must outrank I (which only shares directly, score 1.0).
        assert!(ids.contains(&"C"), "{ids:?}");
        assert!(
            ids.contains(&"I"),
            "I shares the tag directly, so it is a
legit candidate: {ids:?}"
        );
        let c = walks.iter().find(|w| w.work_id == "C").unwrap();
        let i = walks.iter().find(|w| w.work_id == "I").unwrap();
        // Both share tag 1 directly AND symmetrically via the two-hop
        // through each other, so their scores tie at 1.5 — assert presence
        // and non-negative, not a strict ordering (the graph is symmetric).
        assert!(c.score >= 1.0 && i.score >= 1.0, "{walks:?}");
    }

    #[test]
    fn name_is_tag_graph() {
        assert_eq!(TagGraphStrategy::new().name(), "tag_graph");
    }
}
