//! Embedding-based dedupe: find works with cosine similarity > threshold.
//! Secondary check after title+author fast path catches cross-site duplicates
//! that the exact matcher misses (different titles, different authors).
//!
//! Auto-merges high-confidence matches (≥0.98) since split is available.
//! Creates proposals for lower confidence matches.

use crate::db::queries;
use serde::{Deserialize, Serialize};

/// One potential duplicate pair found by embedding similarity.
///
/// `*_url_id` is the stable identity: `fic_info.id`, a slug such as
/// `adminit_autotag_fic`. `*_work_id` is the actionable one: `fic_info.work_id`,
/// which is `integer REFERENCES works(id)`. The two are bridged by
/// `fic_info.work_id` because the merge path (`execute_merge`,
/// `work_proposals`) operates on `works.id`.
///
/// `*_work_id` is `Option` because `fic_info.work_id` is nullable: an unlinked
/// source-site entry has no `works` row to merge into. The query already filters
/// those out, so this is belt-and-braces for the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCandidate {
    pub source_url_id: String,
    pub target_url_id: String,
    pub source_work_id: Option<i32>,
    pub target_work_id: Option<i32>,
    pub source_title: String,
    pub target_title: String,
    pub similarity: f64,
}

/// Find works with embedding similarity above threshold.
/// Uses pgvector cosine distance on rec_embeddings table.
pub async fn candidate_pairs(
    db: &sqlx::PgPool,
    threshold: f64,
    limit: usize,
) -> Result<Vec<EmbeddingCandidate>, sqlx::Error> {
    // `rec_embeddings.work_id` is `varchar(128)` with a foreign key onto
    // `fic_info(id)`, NOT `works(id)`, so every join here goes through
    // `fic_info`. Titles live there too, and `fic_info.work_id` bridges to the
    // `works.id` that the merge path needs.
    let rows = sqlx::query_as::<_, (String, String, Option<i32>, String, String, Option<i32>, f64)>(
        r#"SELECT
            a.work_id                        AS source_url_id,
            fa.title                         AS source_title,
            fa.work_id                       AS source_work_id,
            b.work_id                        AS target_url_id,
            fb.title                         AS target_title,
            fb.work_id                       AS target_work_id,
            1 - (a.embedding <=> b.embedding) AS similarity
        FROM rec_embeddings a
        JOIN rec_embeddings b ON a.model = b.model AND a.work_id < b.work_id
        JOIN fic_info fa ON fa.id = a.work_id
        JOIN fic_info fb ON fb.id = b.work_id
        WHERE fa.work_id IS NOT NULL
          AND fb.work_id IS NOT NULL
          AND 1 - (a.embedding <=> b.embedding) > $1
          AND NOT EXISTS (
            SELECT 1 FROM work_proposals wp
            WHERE wp.action_type = 'merge'
              AND wp.status = 'pending'
              AND ((wp.source_work_id = fa.work_id AND wp.target_work_id = fb.work_id)
                OR (wp.source_work_id = fb.work_id AND wp.target_work_id = fa.work_id))
          )
        ORDER BY similarity DESC
        LIMIT $2"#,
    )
    .bind(threshold)
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(src_url, src_title, src_work, tgt_url, tgt_title, tgt_work, sim)| {
                EmbeddingCandidate {
                    source_url_id: src_url,
                    target_url_id: tgt_url,
                    source_work_id: src_work,
                    target_work_id: tgt_work,
                    source_title: src_title,
                    target_title: tgt_title,
                    similarity: sim,
                }
            },
        )
        .collect())
}

/// Confidence threshold for auto-merge (no proposal needed).
const AUTO_MERGE_THRESHOLD: f64 = 0.98;

/// Run embedding dedupe: find candidate pairs, auto-merge high-confidence,
/// create proposals for lower confidence.
/// Returns (auto_merged, proposed, examined).
pub async fn run_dedupe(
    db: &sqlx::PgPool,
    proposer_id: i32,
    threshold: f64,
    limit: usize,
) -> (usize, usize, usize) {
    // A failure here used to be swallowed into a silent (0, 0, 0), which made
    // this whole feature look like a healthy no-op: candidate_pairs could not
    // execute against the real schema and every run reported "no duplicates".
    // Keep the tolerant return for the caller, but never fail silently.
    let candidates = match candidate_pairs(db, threshold, limit).await {
        Ok(c) => c,
        Err(err) => {
            tracing::error!(
                "embedding dedupe: candidate_pairs failed, reporting no duplicates: {err}"
            );
            return (0, 0, 0);
        }
    };
    if candidates.is_empty() {
        return (0, 0, 0);
    }

    let mut auto_merged = 0usize;
    let mut proposed = 0usize;

    for c in &candidates {
        // `fic_info.work_id` is nullable; the query filters those rows out, so
        // this only trips if that filter is ever weakened.
        let (Some(source_work_id), Some(target_work_id)) =
            (c.source_work_id, c.target_work_id)
        else {
            tracing::debug!(
                "embedding dedupe: skipping unlinked pair {} / {}",
                c.source_url_id,
                c.target_url_id
            );
            continue;
        };

        if c.similarity >= AUTO_MERGE_THRESHOLD {
            // High confidence: auto-merge directly
            if let Ok(()) = queries::execute_merge(db, source_work_id, target_work_id).await {
                auto_merged += 1;
                tracing::info!(
                    "Auto-merged work {} into {} (similarity={:.3})",
                    source_work_id,
                    target_work_id,
                    c.similarity
                );
            }
        } else {
            // Lower confidence: create proposal for curator review
            let _ = queries::create_proposal(
                db,
                proposer_id,
                "merge",
                Some(source_work_id),
                Some(target_work_id),
                None,
                Some(serde_json::json!({
                    "reason": "embedding similarity detected",
                    "similarity": c.similarity,
                    "source_title": c.source_title,
                    "target_title": c.target_title,
                })),
            )
            .await;
            proposed += 1;
        }
    }

    (auto_merged, proposed, candidates.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_serializes() {
        let c = EmbeddingCandidate {
            source_url_id: "slug-1".into(),
            target_url_id: "slug-2".into(),
            source_work_id: Some(1),
            target_work_id: Some(2),
            source_title: "Title A".into(),
            target_title: "Title B".into(),
            similarity: 0.97,
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"similarity\":0.97"));
    }

    #[test]
    fn threshold_filters_correctly() {
        let candidates = vec![
            EmbeddingCandidate {
                source_url_id: "slug-1".into(),
                target_url_id: "slug-2".into(),
                source_work_id: Some(1),
                target_work_id: Some(2),
                source_title: "A".into(),
                target_title: "B".into(),
                similarity: 0.98,
            },
            EmbeddingCandidate {
                source_url_id: "slug-3".into(),
                target_url_id: "slug-4".into(),
                source_work_id: Some(3),
                target_work_id: Some(4),
                source_title: "C".into(),
                target_title: "D".into(),
                similarity: 0.92,
            },
        ];

        let threshold = 0.95;
        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|c| c.similarity > threshold)
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source_work_id, Some(1));
        assert_eq!(filtered[0].source_url_id, "slug-1");
    }

    #[test]
    fn auto_merge_threshold_works() {
        assert!(0.98 >= AUTO_MERGE_THRESHOLD);
        assert!(0.99 >= AUTO_MERGE_THRESHOLD);
        assert!(!(0.97 >= AUTO_MERGE_THRESHOLD));
        assert!(!(0.95 >= AUTO_MERGE_THRESHOLD));
    }
}
