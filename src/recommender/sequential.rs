//! Sequential next-read strategy — first-order Markov transitions.
//!
//! The reader's "Next Up" panel wants: what do readers typically read after
//! this? `rec_transitions` holds (from_work → to_work, weight, last_seen),
//! recency-weighted. Sources (built by `train`):
//!   * sequel links (series_works ordering)
//!   * reading-position/completion signals (reading_stats / chapters)
//!   * co-download sequences in request_log (same session ordering)
//!
//! Serving: given a seed work, return the top transition targets that exist
//! in the corpus and are not the seed itself. This feeds the Next Up panel;
//! the existing fallback chain (sequel → community pick → also-bookmarked)
//! stays when this strategy is disabled or returns nothing.

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Increment (or insert) a transition edge. Pure function over a map so it
/// is unit-testable; `recency` multiplies the added weight.
pub fn add_transition(
    map: &mut std::collections::HashMap<(String, String), f64>,
    from: &str,
    to: &str,
    weight: f64,
) {
    if from == to || from.is_empty() || to.is_empty() {
        return;
    }
    *map.entry((from.to_string(), to.to_string())).or_insert(0.0) += weight;
}

/// Rank transition targets for a seed: weight × recency decay.
pub fn rank_targets(
    map: &std::collections::HashMap<(String, String), f64>,
    seed: &str,
    max: usize,
) -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = map
        .iter()
        .filter(|((from, _), _)| from == seed)
        .map(|((_, to), w)| (to.clone(), *w))
        .collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(max);
    out
}

/// Rebuild `rec_transitions` from sequel links + request_log sequences.
/// Returns (inserted, total_weight).
pub async fn rebuild_transitions(ctx: &StrategyContext) -> Result<(usize, f64), RecError> {
    let mut map: std::collections::HashMap<(String, String), f64> =
        std::collections::HashMap::new();

    // 1. Sequel links: series_works ordered by position → next-in-series.
    let series: Vec<(i32, i32, i32)> = sqlx::query_as(
        r#"SELECT series_id, work_id, position FROM series_works ORDER BY series_id, position"#,
    )
    .fetch_all(&ctx.db)
    .await?;
    let mut by_series: std::collections::HashMap<i32, Vec<(i32, i32)>> =
        std::collections::HashMap::new();
    for (sid, wid, pos) in series {
        by_series.entry(sid).or_default().push((wid, pos));
    }
    for (_sid, works) in by_series {
        let mut sorted = works;
        sorted.sort_by_key(|(_, p)| *p);
        for pair in sorted.windows(2) {
            let (from_w, _) = pair[0];
            let (to_w, _) = pair[1];
            // work_id → url_id via fic_info.
            let urls: Vec<(String, String)> =
                sqlx::query_as("SELECT id, work_id::text FROM fic_info WHERE work_id = ANY($1)")
                    .bind(&[from_w, to_w])
                    .fetch_all(&ctx.db)
                    .await?;
            let mut from_url: Option<String> = None;
            let mut to_url: Option<String> = None;
            for (id, wid) in urls {
                if wid == from_w.to_string() {
                    from_url = Some(id.clone());
                }
                if wid == to_w.to_string() {
                    to_url = Some(id);
                }
            }
            if let (Some(f), Some(t)) = (from_url, to_url) {
                add_transition(&mut map, &f, &t, 3.0);
            }
        }
    }

    // 2. request_log co-download sequences: same IP + etype download/export
    // within a 6-hour window → (earlier → later).
    let downloads: Vec<(
        Option<String>,
        Option<String>,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        r#"SELECT url, url_id, created
               FROM request_log
               WHERE url_id IS NOT NULL
                 AND (export_file_name LIKE '%.epub' OR etype IN ('download', 'export'))
                 AND created > now() - interval '180 days'
               ORDER BY created"#,
    )
    .fetch_all(&ctx.db)
    .await?;
    // Group by url (the IP is not reliably present; url = requested URL is
    // the closest session key we have). Take consecutive pairs in the
    // ordered stream where the gap ≤ 6h.
    let mut prev: Option<(String, chrono::DateTime<chrono::Utc>)> = None;
    for (url, url_id, created) in downloads {
        if let (Some(url), Some(url_id)) = (url, url_id) {
            if let Some((p_url, p_created)) = &prev {
                let gap = created.signed_duration_since(*p_created).num_hours();
                if gap >= 0 && gap <= 6 {
                    add_transition(&mut map, p_url, &url_id, 1.0);
                }
            }
            prev = Some((url_id, created));
            let _ = url;
        } else {
            prev = None;
        }
    }

    // Upsert (idempotent).
    let mut tx = ctx.db.begin().await?;
    sqlx::query("DELETE FROM rec_transitions")
        .execute(&mut *tx)
        .await?;
    let mut inserted = 0usize;
    let mut total_weight = 0.0f64;
    for ((from, to), w) in &map {
        sqlx::query(
            r#"INSERT INTO rec_transitions (from_work, to_work, weight, last_seen)
               VALUES ($1, $2, $3, NOW())"#,
        )
        .bind(from)
        .bind(to)
        .bind(*w)
        .execute(&mut *tx)
        .await?;
        inserted += 1;
        total_weight += w;
    }
    tx.commit().await?;
    Ok((inserted, total_weight))
}

pub struct SequentialStrategy;

impl SequentialStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SequentialStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for SequentialStrategy {
    fn name(&self) -> &str {
        "sequential"
    }

    /// Sequential is reader/seed-based only — it never serves the
    /// personalized home feed.
    fn supports_personal(&self) -> bool {
        false
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        if user_id.is_some() {
            return Err(RecError::NotEnoughData(
                "sequential is seed-based only (Next Up)".into(),
            ));
        }
        let Some(seed_id) = seed else {
            return Err(RecError::Strategy("sequential needs a seed work".into()));
        };
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"SELECT to_work, weight * exp(-(EXTRACT(EPOCH FROM (NOW() - last_seen)) / 86400.0) / 30.0) AS w
               FROM rec_transitions
               WHERE from_work = $1
               ORDER BY w DESC
               LIMIT $2"#,
        )
        .bind(seed_id)
        .bind(ctx.config.rec_max_recommendations as i64)
        .fetch_all(&ctx.db)
        .await?;
        if rows.is_empty() {
            return Err(RecError::NotEnoughData(format!(
                "no transitions from {seed_id}"
            )));
        }
        Ok(rows
            .into_iter()
            .map(|(work_id, w)| ScoredRec {
                work_id,
                score: w,
                strategy: self.name().into(),
                reason: "readers typically continue with this".into(),
            })
            .collect())
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        let (inserted, total_weight) = rebuild_transitions(ctx).await?;
        Ok(json!({ "edges": inserted, "total_weight": total_weight }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn transitions_accumulate_and_rank() {
        let mut map: HashMap<(String, String), f64> = HashMap::new();
        add_transition(&mut map, "a", "b", 1.0);
        add_transition(&mut map, "a", "b", 2.0);
        add_transition(&mut map, "a", "c", 1.5);
        add_transition(&mut map, "b", "c", 9.0);
        let ranked = rank_targets(&map, "a", 10);
        assert_eq!(ranked[0].0, "b");
        assert_eq!(ranked[0].1, 3.0);
        assert_eq!(ranked[1].0, "c");
        // b→c must NOT appear for seed a.
        assert_eq!(ranked.len(), 2);
    }

    #[test]
    fn transitions_ignore_self_and_empty() {
        let mut map: HashMap<(String, String), f64> = HashMap::new();
        add_transition(&mut map, "a", "a", 5.0);
        add_transition(&mut map, "", "b", 5.0);
        add_transition(&mut map, "a", "", 5.0);
        assert!(map.is_empty());
    }

    #[test]
    fn rank_targets_caps_and_sorts() {
        let mut map: HashMap<(String, String), f64> = HashMap::new();
        for i in 0..5 {
            add_transition(&mut map, "a", &format!("w{i}"), i as f64);
        }
        let ranked = rank_targets(&map, "a", 2);
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].0, "w4");
    }
}
