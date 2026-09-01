//! Embedding-based dedupe: find works with cosine similarity > threshold.
//! Secondary check after title+author fast path catches cross-site duplicates
//! that the exact matcher misses (different titles, different authors).
//!
//! Auto-merges high-confidence matches (≥0.98) since split is available.
//! Creates proposals for lower confidence matches.

use crate::db::queries;
use serde::{Deserialize, Serialize};

/// One potential duplicate pair found by embedding similarity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingCandidate {
    pub source_work_id: i32,
    pub target_work_id: i32,
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
    let rows = sqlx::query_as::<_, (i32, String, i32, String, f64)>(
        r#"SELECT 
            a.work_id as source_work_id,
            wa.canonical_title as source_title,
            b.work_id as target_work_id,
            wb.canonical_title as target_title,
            1 - (a.embedding <=> b.embedding) as similarity
        FROM rec_embeddings a
        JOIN rec_embeddings b ON a.model = b.model AND a.work_id < b.work_id
        JOIN works wa ON wa.id = a.work_id
        JOIN works wb ON wb.id = b.work_id
        WHERE 1 - (a.embedding <=> b.embedding) > $1
          AND NOT EXISTS (
            SELECT 1 FROM work_proposals wp
            WHERE wp.action_type = 'merge'
              AND wp.status = 'pending'
              AND ((wp.source_work_id = a.work_id AND wp.target_work_id = b.work_id)
                OR (wp.source_work_id = b.work_id AND wp.target_work_id = a.work_id))
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
        .map(|(src_id, src_title, tgt_id, tgt_title, sim)| EmbeddingCandidate {
            source_work_id: src_id,
            target_work_id: tgt_id,
            source_title: src_title,
            target_title: tgt_title,
            similarity: sim,
        })
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
    let Ok(candidates) = candidate_pairs(db, threshold, limit).await else {
        return (0, 0, 0);
    };
    if candidates.is_empty() {
        return (0, 0, 0);
    }

    let mut auto_merged = 0usize;
    let mut proposed = 0usize;

    for c in &candidates {
        if c.similarity >= AUTO_MERGE_THRESHOLD {
            // High confidence: auto-merge directly
            if let Ok(()) = queries::execute_merge(db, c.source_work_id, c.target_work_id).await {
                auto_merged += 1;
                tracing::info!(
                    "Auto-merged work {} into {} (similarity={:.3})",
                    c.source_work_id, c.target_work_id, c.similarity
                );
            }
        } else {
            // Lower confidence: create proposal for curator review
            let _ = queries::create_proposal(
                db,
                proposer_id,
                "merge",
                Some(c.source_work_id),
                Some(c.target_work_id),
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
            source_work_id: 1,
            target_work_id: 2,
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
                source_work_id: 1,
                target_work_id: 2,
                source_title: "A".into(),
                target_title: "B".into(),
                similarity: 0.98,
            },
            EmbeddingCandidate {
                source_work_id: 3,
                target_work_id: 4,
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
        assert_eq!(filtered[0].source_work_id, 1);
    }

    #[test]
    fn auto_merge_threshold_works() {
        assert!(0.98 >= AUTO_MERGE_THRESHOLD);
        assert!(0.99 >= AUTO_MERGE_THRESHOLD);
        assert!(!(0.97 >= AUTO_MERGE_THRESHOLD));
        assert!(!(0.95 >= AUTO_MERGE_THRESHOLD));
    }
}
