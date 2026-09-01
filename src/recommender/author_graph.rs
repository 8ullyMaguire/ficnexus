//! Author graph strategy — author-level co-bookmark + shared-tag Jaccard.
//!
//! Readers who bookmark works by author A also tend to bookmark works by
//! author B. The `rec_author_graph` table (built by `train`) holds
//! (author_a, author_b, cooccur_count, tag_jaccard). Scoring a seed work:
//! rank its author's neighbours by combined co-bookmark × tag-Jaccard, then
//! surface the neighbour authors' best works.

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Rebuild the author graph from `fic_bookmark_cooccur` + `fic_tags`.
/// Co-occurrence: two authors co-occur when a reader bookmarks works by
/// both; counted via the work-level co-occurrence pairs. Tag Jaccard: shared
/// tag sets over their works (top 50 tags each).
pub async fn rebuild_author_graph(
    ctx: &StrategyContext,
) -> Result<(usize, usize), RecError> {
    // Map work → author (works may have multiple sources; use the work-level
    // canonical author when available, else fic_info.author).
    let work_authors: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT fi.id, COALESCE(w.canonical_author, fi.author) AS author
           FROM fic_info fi
           LEFT JOIN works w ON w.id = fi.work_id"#,
    )
    .fetch_all(&ctx.db)
    .await?;
    let author_of: std::collections::HashMap<String, String> = work_authors.into_iter().collect();

    // Co-occurrence edges: from fic_bookmark_cooccur (work_a, work_b) →
    // (author_a, author_b).
    let cooccur_edges: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT work_a, work_b, cooccur_count FROM fic_bookmark_cooccur",
    )
    .fetch_all(&ctx.db)
    .await?;

    // Shared-tag Jaccard per author pair (computed in Rust from per-author
    // tag sets — cheap at this scale).
    let author_tags: std::collections::HashMap<String, std::collections::HashSet<i32>> =
        fetch_author_tags(&ctx.db).await;

    let mut edges: std::collections::HashMap<(String, String), (i64, f64)> =
        std::collections::HashMap::new();
    for (wa, wb, count) in cooccur_edges {
        let (Some(aa), Some(ab)) = (author_of.get(&wa), author_of.get(&wb)) else {
            continue;
        };
        if aa == ab || aa.is_empty() || ab.is_empty() {
            continue;
        }
        let (a, b) = if aa < ab { (aa.clone(), ab.clone()) } else { (ab.clone(), aa.clone()) };
        let entry = edges.entry((a, b)).or_insert((0, 0.0));
        entry.0 += count;
    }
    // Tag Jaccard for each author pair that co-occurs.
    for ((a, b), (count, jac)) in edges.iter_mut() {
        let tags_a = author_tags.get(a);
        let tags_b = author_tags.get(b);
        *jac = match (tags_a, tags_b) {
            (Some(ta), Some(tb)) if !ta.is_empty() && !tb.is_empty() => {
                let inter = ta.intersection(tb).count();
                let union = ta.union(tb).count();
                if union > 0 {
                    inter as f64 / union as f64
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        let _ = count;
    }

    // Upsert into rec_author_graph (idempotent).
    let mut tx = ctx.db.begin().await?;
    sqlx::query("DELETE FROM rec_author_graph").execute(&mut *tx).await?;
    let mut inserted = 0usize;
    for ((a, b), (count, jac)) in edges.iter() {
        if *count == 0 {
            continue;
        }
        sqlx::query(
            r#"INSERT INTO rec_author_graph (author_a, author_b, cooccur_count, tag_jaccard)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(a)
        .bind(b)
        .bind(*count as i32)
        .bind(*jac)
        .execute(&mut *tx)
        .await?;
        inserted += 1;
    }
    tx.commit().await?;
    Ok((inserted, edges.len()))
}

async fn fetch_author_tags(
    db: &sqlx::PgPool,
) -> std::collections::HashMap<String, std::collections::HashSet<i32>> {
    let rows: Vec<(String, i32)> = sqlx::query_as(
        r#"SELECT COALESCE(w.canonical_author, fi.author) AS author, ft.tag_id
           FROM fic_tags ft
           JOIN fic_info fi ON fi.id = ft.url_id
           LEFT JOIN works w ON w.id = fi.work_id"#,
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    let mut map: std::collections::HashMap<String, std::collections::HashSet<i32>> =
        std::collections::HashMap::new();
    for (author, tag) in rows {
        map.entry(author).or_default().insert(tag);
    }
    map
}

pub struct AuthorGraphStrategy;

impl AuthorGraphStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AuthorGraphStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RecStrategy for AuthorGraphStrategy {
    fn name(&self) -> &str {
        "author_graph"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        if user_id.is_some() {
            return Err(RecError::NotEnoughData(
                "author_graph is seed-based only".into(),
            ));
        }
        let Some(seed_id) = seed else {
            return Err(RecError::Strategy("author_graph needs a seed work".into()));
        };

        // Seed author.
        let seed_author: Option<String> = sqlx::query_scalar(
            r#"SELECT COALESCE(w.canonical_author, fi.author)
               FROM fic_info fi
               LEFT JOIN works w ON w.id = fi.work_id
               WHERE fi.id = $1"#,
        )
        .bind(seed_id)
        .fetch_optional(&ctx.db)
        .await?;
        let Some(seed_author) = seed_author.filter(|a| !a.is_empty()) else {
            return Err(RecError::NotEnoughData("seed work has no author".into()));
        };

        // Neighbour authors via the graph, ranked by cooccur × (1+tag_jaccard).
        let neighbours: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT CASE WHEN author_a = $1 THEN author_b ELSE author_a END AS neighbour,
                   (cooccur_count::float8 * (1.0 + tag_jaccard)) AS score
            FROM rec_author_graph
            WHERE author_a = $1 OR author_b = $1
            ORDER BY score DESC
            LIMIT 20
            "#,
        )
        .bind(&seed_author)
        .fetch_all(&ctx.db)
        .await?;
        if neighbours.is_empty() {
            return Err(RecError::NotEnoughData(format!(
                "no author-graph neighbours for {seed_author}"
            )));
        }

        // Surface the neighbours' best works (exclude the seed itself).
        let mut out: Vec<ScoredRec> = Vec::new();
        let limit = ctx.config.rec_max_recommendations;
        for (author, author_score) in neighbours {
            if out.len() >= limit {
                break;
            }
            let works: Vec<(String, f64)> = sqlx::query_as(
                r#"SELECT fi.id, fi.words::float8
                   FROM fic_info fi
                   LEFT JOIN works w ON w.id = fi.work_id
                   WHERE COALESCE(w.canonical_author, fi.author) = $1
                     AND fi.id != $2
                   ORDER BY fi.updated DESC
                   LIMIT 5"#,
            )
            .bind(&author)
            .bind(seed_id)
            .fetch_all(&ctx.db)
            .await?;
            for (wid, _words) in works {
                out.push(ScoredRec {
                    work_id: wid,
                    score: author_score,
                    strategy: self.name().into(),
                    reason: format!("readers of {seed_author} also read {author}"),
                });
                if out.len() >= limit {
                    break;
                }
            }
        }

        if out.is_empty() {
            return Err(RecError::NotEnoughData("author graph produced no works".into()));
        }
        Ok(out)
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        let (inserted, edges) = rebuild_author_graph(ctx).await?;
        Ok(json!({ "edges": edges, "inserted": inserted }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_is_author_graph() {
        assert_eq!(AuthorGraphStrategy::new().name(), "author_graph");
    }
}
