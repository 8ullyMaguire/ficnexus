//! Auto-tagger — zero-shot tag classification using embeddings.
//!
//! The fic's description (+ title, and first chapter text when it is
//! available in the DB) is embedded via Ollama (nomic-embed-text, 768-d)
//! and compared with cosine similarity against the embedded canonical
//! freeform tags in `tag_embeddings`. Tags above the similarity threshold
//! are attached to the fic with `is_machine_suggested = TRUE` and enter
//! the admin review queue; an admin approves (promotes to a regular tag)
//! or dismisses (deletes the row) each suggestion.
//!
//! Chapter text lives in the export cache on disk (zip bundles — see
//! `cache::disk`), not in the DB, so the classifier embeds
//! description + title only; the `first_chapter` field is kept on
//! `TagSuggestion` for future wiring once chapter text is indexed.
//!
//! The `tag_embeddings` cache is populated by the idempotent
//! [`backfill_tag_embeddings`] path (per-call embedding would re-embed the
//! whole tag corpus on every recommendation).

use sqlx::PgPool;

use crate::services::ollama::{OllamaClient, OllamaError};

/// Similarity threshold: suggestions with cosine similarity >= this are
/// attached as machine-suggested tags (0.75 ≈ a near-duplicate phrase; tags
/// that merely share a token land below it).
pub const SIMILARITY_THRESHOLD: f32 = 0.75;

/// Number of nearest tag embeddings to consider per recommendation call.
pub const TOP_N_TAGS: i64 = 10;

/// Dimension of the nomic-embed-text vectors (matches `VECTOR(768)`).
const EMBED_DIM: usize = 768;

/// A single machine-suggested tag.
#[derive(Debug, Clone)]
pub struct TagSuggestion {
    pub tag_id: i32,
    pub tag_name: String,
    pub tag_type_id: i16,
    pub similarity: f32,
    /// First chapter text — reserved for when chapter text is indexed; the
    /// classifier currently embeds description + title only.
    pub first_chapter: Option<String>,
}

/// Accept a suggestion when its cosine similarity meets the threshold.
/// Pure function (unit-tested): `similarity` is clamped to [0, 1] so a
/// degenerate/NaN embedding can never pass.
pub fn should_accept(similarity: f32, threshold: f32) -> bool {
    similarity.is_finite() && similarity.clamp(0.0, 1.0) >= threshold
}

/// Embed a vec of f32s as pgvector literal text: `[0.123,0.456,...]`.
/// Used to bind Rust-side vectors into `$1::vector` casts (same approach
/// as the Roadmap Consensus Engine).
pub fn vec_to_sql(emb: &[f32]) -> String {
    format!(
        "[{}]",
        emb.iter()
            .map(|f| format!("{f:.6}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// Recommend (and insert) machine-suggested tags for a fic.
///
/// Pipeline: load the fic's description + title (first chapter text is in
/// the on-disk cache, not the DB — skipped), embed it, find the nearest
/// embedded canonical freeform tags via pgvector cosine distance, keep
/// those at/above [`SIMILARITY_THRESHOLD`], and insert them into `fic_tags`
/// with `is_machine_suggested = TRUE` and the similarity as the score.
///
/// Inserts are idempotent (ON CONFLICT DO NOTHING): re-running a fic that
/// already has a machine suggestion is a no-op rather than a duplicate.
/// Existing human tags are never touched. Returns the suggestions that were
/// inserted (plus any that were already present, so the admin UI can show
/// the full picture on re-run).
pub async fn recommend_tags(
    db: &PgPool,
    ollama: &OllamaClient,
    url_id: &str,
) -> Result<Vec<TagSuggestion>, AutoTaggerError> {
    let (title, description) = fetch_fic_text(db, url_id).await?;

    let mut text = title.trim().to_string();
    let desc = description.trim();
    if !desc.is_empty() {
        if !text.is_empty() {
            text.push_str(". ");
        }
        text.push_str(desc);
    }
    if text.is_empty() {
        return Err(AutoTaggerError::NoContent(
            "fic has no title or description to embed".into(),
        ));
    }

    let embedding = ollama.embed(&text).await.map_err(AutoTaggerError::Embed)?;
    if embedding.len() != EMBED_DIM {
        return Err(AutoTaggerError::Embed(OllamaError(format!(
            "unexpected embedding dimension {} (expected {EMBED_DIM})",
            embedding.len()
        ))));
    }

    let emb_sql = vec_to_sql(&embedding);

    // Nearest embedded canonical freeform tags (tag_type_id = 4), by
    // cosine distance (<=>) — similarity = 1 - distance.
    let rows: Vec<(i32, String, i16, f64)> = sqlx::query_as(
        r#"
        SELECT t.id, t.name, t.tag_type_id,
               (1 - (te.embedding <=> $1::vector))::float8 AS similarity
        FROM tag_embeddings te
        JOIN tags t ON t.id = te.tag_id
        WHERE t.tag_type_id = 4
        ORDER BY te.embedding <=> $1::vector ASC
        LIMIT $2
        "#,
    )
    .bind(&emb_sql)
    .bind(TOP_N_TAGS)
    .fetch_all(db)
    .await
    .map_err(AutoTaggerError::Db)?;

    let mut suggestions: Vec<TagSuggestion> = Vec::new();
    for (tag_id, tag_name, tag_type_id, similarity) in rows {
        let similarity = similarity as f32;
        if !should_accept(similarity, SIMILARITY_THRESHOLD) {
            continue; // rows are distance-ordered → everything after also fails
        }
        sqlx::query(
            r#"
            INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score, is_machine_suggested)
            VALUES ($1, $2, '0.0.0.0', $3, TRUE)
            ON CONFLICT (url_id, tag_id) DO NOTHING
            "#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind((similarity * 100.0) as i16)
        .execute(db)
        .await
        .map_err(AutoTaggerError::Db)?;

        suggestions.push(TagSuggestion {
            tag_id,
            tag_name,
            tag_type_id,
            similarity,
            first_chapter: None,
        });
    }

    Ok(suggestions)
}

/// Load the fic's title + description. First chapter text lives in the
/// on-disk export cache (zip bundles), not the DB, so we embed title +
/// description only.
async fn fetch_fic_text(db: &PgPool, url_id: &str) -> Result<(String, String), AutoTaggerError> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT title, description FROM fic_info WHERE id = $1")
            .bind(url_id)
            .fetch_optional(db)
            .await
            .map_err(AutoTaggerError::Db)?;

    match row {
        Some((title, description)) => Ok((title, description)),
        None => Err(AutoTaggerError::NoContent(format!(
            "fic {url_id} not found"
        ))),
    }
}

/// Backfill `tag_embeddings` for canonical freeform tags that don't have
/// one yet (idempotent — re-running skips already-embedded tags).
///
/// Embeds only canonical freeform tags: freeform tropes (type 4) are the
/// vocabulary the classifier recommends against; fandom/character/
/// relationship/category/warning tags are scraped directly and excluded.
/// Returns the number of tags newly embedded.
pub async fn backfill_tag_embeddings(
    db: &PgPool,
    ollama: &OllamaClient,
) -> Result<usize, AutoTaggerError> {
    let rows: Vec<(i32, String)> = sqlx::query_as(
        r#"
        SELECT t.id, t.name
        FROM tags t
        WHERE t.tag_type_id = 4
          AND NOT EXISTS (SELECT 1 FROM tag_embeddings te WHERE te.tag_id = t.id)
        ORDER BY t.id
        "#,
    )
    .fetch_all(db)
    .await
    .map_err(AutoTaggerError::Db)?;

    let mut embedded = 0usize;
    for (tag_id, name) in rows {
        let emb = match ollama.embed(&name).await {
            Ok(e) => e,
            Err(err) => {
                // Best-effort: one failure shouldn't abort the whole corpus.
                tracing::warn!("auto-tagger backfill: embed failed for tag {tag_id}: {err}");
                continue;
            }
        };
        if emb.len() != EMBED_DIM {
            tracing::warn!(
                "auto-tagger backfill: tag {tag_id} embedded to {} dims (expected {EMBED_DIM})",
                emb.len()
            );
            continue;
        }
        let emb_sql = vec_to_sql(&emb);
        sqlx::query(
            "INSERT INTO tag_embeddings (tag_id, embedding) VALUES ($1, $2::vector) ON CONFLICT (tag_id) DO NOTHING",
        )
        .bind(tag_id)
        .bind(&emb_sql)
        .execute(db)
        .await
        .map_err(AutoTaggerError::Db)?;
        embedded += 1;
    }

    Ok(embedded)
}

/// Errors from the auto-tagger service. All are non-fatal at the handler
/// level (the admin gets a clear message instead of a 500 for expected
/// conditions like a missing fic or Ollama being down).
#[derive(Debug)]
pub enum AutoTaggerError {
    /// The fic doesn't exist, or has no title/description to embed.
    NoContent(String),
    /// Ollama embedding call failed (model down, bad response, …).
    Embed(OllamaError),
    /// Database error while reading fics/tags or inserting suggestions.
    Db(sqlx::Error),
}

impl std::fmt::Display for AutoTaggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoTaggerError::NoContent(msg) => write!(f, "{msg}"),
            AutoTaggerError::Embed(err) => write!(f, "embedding failed: {err}"),
            AutoTaggerError::Db(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for AutoTaggerError {}

impl From<sqlx::Error> for AutoTaggerError {
    fn from(err: sqlx::Error) -> Self {
        AutoTaggerError::Db(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_accept_at_and_above_threshold() {
        assert!(should_accept(0.75, 0.75), "exactly at threshold accepts");
        assert!(should_accept(0.9, 0.75), "above threshold accepts");
        assert!(should_accept(1.0, 0.75), "perfect match accepts");
    }

    #[test]
    fn should_reject_below_threshold() {
        assert!(!should_accept(0.749, 0.75), "just below threshold rejects");
        assert!(!should_accept(0.0, 0.75), "orthogonal rejects");
        assert!(!should_accept(0.5, 0.75), "half-similar rejects");
    }

    #[test]
    fn should_reject_degenerate_inputs() {
        // NaN similarity can never be a real match.
        assert!(!should_accept(f32::NAN, 0.75));
        // Negative similarity (anti-correlated) clamps to 0.0, so it
        // rejects at any threshold > 0.
        assert!(!should_accept(-0.9, 0.01));
        assert!(!should_accept(-0.9, 0.5));
        // Infinite similarity rejects.
        assert!(!should_accept(f32::INFINITY, 0.0));
    }

    #[test]
    fn should_clamp_similarity_over_one() {
        // Values above 1.0 (shouldn't happen for cosine, but be safe)
        // are clamped and still accepted at any threshold <= 1.0.
        assert!(should_accept(1.5, 0.75));
    }

    #[test]
    fn vec_to_sql_formats_pgvector_literal() {
        let sql = vec_to_sql(&[0.123456789, 1.0]);
        assert_eq!(sql, "[0.123457,1.000000]");
    }

    #[test]
    fn vec_to_sql_empty_is_empty_brackets() {
        assert_eq!(vec_to_sql(&[]), "[]");
    }

    #[test]
    fn vec_to_sql_round_trips_precision_and_negatives() {
        // 6-decimal formatting is stable for negatives and small values.
        assert_eq!(
            vec_to_sql(&[-0.5, 0.0, 0.1234567]),
            "[-0.500000,0.000000,0.123457]"
        );
        assert_eq!(vec_to_sql(&[0.1]), "[0.100000]");
    }

    #[test]
    fn should_accept_threshold_zero() {
        // Threshold 0: any finite clamped similarity >= 0 passes, even 0.0.
        assert!(should_accept(0.0, 0.0));
        assert!(should_accept(0.5, 0.0));
        // NaN still rejects.
        assert!(!should_accept(f32::NAN, 0.0));
    }
}
