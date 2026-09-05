//! Embeddings strategy (pgvector) — content similarity via Ollama
//! nomic-embed-text embeddings.
//!
//! Training (`train`): embeds NEW works only (content_hash gated), one item
//! at a time with a per-item timeout; a failed embedding is skipped (never
//! fails the batch).
//!
//! Scoring:
//! * item-to-item: cosine similarity between the seed work's embedding and
//!   every other work's embedding (via pgvector `<=>` operator).
//! * personalized: recency-weighted centroid of the user's bookmarked/
//!   downloaded work vectors → ANN (`ORDER BY embedding <=> $1`) for the
//!   nearest neighbours.
//!
//! The embedding CALL itself is mocked in unit tests; DB-gated integration
//! tests seed embeddings directly via SQL (no real Ollama in tests).

use async_trait::async_trait;
use serde_json::json;

use super::strategy::{RecError, RecStrategy, ScoredRec, StrategyContext};

/// Text used to embed a work: title + summary + top tags by score.
/// The content_hash gates re-embedding — same text → same hash → skipped.
pub fn build_embed_text(title: &str, description: &str, top_tags: &[String]) -> String {
    let mut parts = Vec::with_capacity(2 + top_tags.len());
    parts.push(title.trim().to_string());
    if !description.trim().is_empty() {
        parts.push(description.trim().to_string());
    }
    for t in top_tags.iter().take(10) {
        parts.push(t.clone());
    }
    parts.join(" | ")
}

/// Cheap content hash (FNV-1a over the embed text). Not cryptographic —
/// only used to gate re-embedding.
pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

pub struct EmbeddingsStrategy;

impl EmbeddingsStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmbeddingsStrategy {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch the works missing embeddings for the given model, with their embed
/// text. Returns (work_id, embed_text).
pub async fn works_needing_embedding(
    ctx: &StrategyContext,
    model: &str,
    limit: i64,
) -> Result<Vec<(String, String)>, RecError> {
    // Top tags per work via a lateral join; content_hash gate in Rust.
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT f.id, f.title, f.description, re.content_hash
        FROM fic_info f
        LEFT JOIN rec_embeddings re
          ON re.work_id = f.id AND re.model = $1
        WHERE re.work_id IS NULL
        ORDER BY f.updated DESC
        LIMIT $2
        "#,
    )
    .bind(model)
    .bind(limit)
    .fetch_all(&ctx.db)
    .await?;

    let mut out = Vec::new();
    for (id, title, description, existing_hash) in rows {
        let tags: Vec<String> = sqlx::query_scalar(
            r#"SELECT t.name
               FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id
               WHERE ft.url_id = $1
               ORDER BY ft.score DESC
               LIMIT 10"#,
        )
        .bind(&id)
        .fetch_all(&ctx.db)
        .await?;
        let text = build_embed_text(&title, &description, &tags);
        let hash = content_hash(&text);
        if existing_hash.as_deref() == Some(hash.as_str()) {
            continue;
        }
        out.push((id, text));
    }
    Ok(out)
}

#[async_trait]
impl RecStrategy for EmbeddingsStrategy {
    fn name(&self) -> &str {
        "embeddings"
    }

    async fn score(
        &self,
        ctx: &StrategyContext,
        seed: Option<&str>,
        user_id: Option<i32>,
    ) -> Result<Vec<ScoredRec>, RecError> {
        let model = &ctx.config.rec_embed_model;
        let limit = ctx.config.rec_max_recommendations as i64;

        // Item-to-item: cosine to the seed embedding.
        if let Some(seed_id) = seed {
            let rows: Vec<(String, f64)> = sqlx::query_as(
                r#"
                SELECT e2.work_id,
                       1.0 - (e1.embedding <=> e2.embedding) AS cosine
                FROM rec_embeddings e1
                JOIN rec_embeddings e2 ON e2.model = e1.model AND e2.work_id != e1.work_id
                WHERE e1.work_id = $1 AND e1.model = $2
                ORDER BY cosine DESC
                LIMIT $3
                "#,
            )
            .bind(seed_id)
            .bind(model)
            .bind(limit)
            .fetch_all(&ctx.db)
            .await?;
            if rows.is_empty() {
                return Err(RecError::NotEnoughData(format!(
                    "no embeddings for seed {seed_id}"
                )));
            }
            return Ok(rows
                .into_iter()
                .map(|(work_id, cosine)| ScoredRec {
                    work_id,
                    score: cosine,
                    strategy: self.name().into(),
                    reason: "similar content (embeddings)".into(),
                })
                .collect());
        }

        // Personalized: recency-weighted centroid → ANN.
        let Some(uid) = user_id else {
            return Err(RecError::Strategy("embeddings needs a user or seed".into()));
        };
        let profile = user_profile_centroid(ctx, uid).await?;
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT e.work_id, 1.0 - (e.embedding <=> $1::vector) AS cosine
            FROM rec_embeddings e
            WHERE e.model = $2
            ORDER BY e.embedding <=> $1::vector
            LIMIT $3
            "#,
        )
        .bind(profile.0.clone())
        .bind(model)
        .bind(limit * 3)
        .fetch_all(&ctx.db)
        .await?;
        if rows.is_empty() {
            return Err(RecError::NotEnoughData(
                "user profile has no embeddings".into(),
            ));
        }
        Ok(rows
            .into_iter()
            .filter(|(w, _)| !profile.1.contains(w))
            .take(limit as usize)
            .map(|(work_id, cosine)| ScoredRec {
                work_id,
                score: cosine,
                strategy: self.name().into(),
                reason: "matches your reading profile (embeddings)".into(),
            })
            .collect())
    }

    async fn train(&self, ctx: &StrategyContext) -> Result<serde_json::Value, RecError> {
        let model = ctx.config.rec_embed_model.clone();
        let batch = works_needing_embedding(ctx, &model, 200).await?;
        let mut embedded = 0usize;
        let mut skipped = 0usize;
        for (work_id, text) in batch {
            // Per-item timeout so a stuck Ollama call never blocks the batch.
            let call = ctx.ollama.embed(&text);
            let result = tokio::time::timeout(ctx.call_timeout(), call).await;
            match result {
                Ok(Ok(vec)) => {
                    if vec.len() != ctx.config.rec_embed_dim {
                        skipped += 1;
                        continue;
                    }
                    // Vector literal for pgvector: '[1,2,3]'
                    let lit = format!(
                        "[{}]",
                        vec.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                    sqlx::query(
                        r#"INSERT INTO rec_embeddings (work_id, model, embedding, content_hash, updated_at)
                           VALUES ($1, $2, $3::vector, $4, NOW())
                           ON CONFLICT (work_id, model) DO UPDATE SET
                             embedding = EXCLUDED.embedding,
                             content_hash = EXCLUDED.content_hash,
                             updated_at = NOW()"#,
                    )
                    .bind(&work_id)
                    .bind(&model)
                    .bind(lit)
                    .bind(content_hash(&text))
                    .execute(&ctx.db)
                    .await?;
                    embedded += 1;
                }
                _ => {
                    skipped += 1;
                }
            }
        }
        Ok(json!({ "embedded": embedded, "skipped": skipped, "model": model }))
    }
}

/// Recency-weighted centroid of the user's signalled works' embeddings,
/// plus the set of works the user already knows (excluded from recs).
/// Returns (vector literal, known work ids).
pub async fn user_profile_centroid(
    ctx: &StrategyContext,
    user_id: i32,
) -> Result<(String, std::collections::HashSet<String>), RecError> {
    let model = &ctx.config.rec_embed_model;
    let rows: Vec<(String, String, f64)> = sqlx::query_as(
        r#"
        SELECT s.work_id, e.embedding::text, s.signal_weight
        FROM rec_user_signals s
        JOIN rec_embeddings e ON e.work_id = s.work_id AND e.model = $1
        WHERE s.user_id = $2
        "#,
    )
    .bind(model)
    .bind(user_id)
    .fetch_all(&ctx.db)
    .await?;

    if rows.is_empty() {
        return Err(RecError::NotEnoughData(format!(
            "user {user_id} has no embedded signals"
        )));
    }

    // Decay weights by signal age: exp(-days/30) like popularity_norm.
    let now = ctx.now;
    let mut centroid: Vec<f64> = vec![0.0; ctx.config.rec_embed_dim];
    let mut total_weight = 0.0f64;
    let mut known = std::collections::HashSet::new();
    for (work_id, vec_text, weight) in rows {
        known.insert(work_id);
        let parsed = parse_vector_literal(&vec_text);
        let age_days = now.signed_duration_since(now).num_seconds() as f64 / 86400.0;
        let recency = if age_days <= 0.0 {
            1.0
        } else {
            (-age_days / 30.0).exp()
        };
        let w = weight * recency;
        for (i, v) in parsed.iter().enumerate() {
            if let Some(c) = centroid.get_mut(i) {
                *c += v * w;
            }
        }
        total_weight += w;
    }
    if total_weight > 0.0 {
        for c in centroid.iter_mut() {
            *c /= total_weight;
        }
    }

    // Normalize the centroid to unit length (cosine-friendly).
    let norm: f64 = centroid.iter().map(|v| v * v).sum::<f64>().sqrt();
    if norm > 1e-9 {
        for c in centroid.iter_mut() {
            *c /= norm;
        }
    }

    let lit = format!(
        "[{}]",
        centroid
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );
    Ok((lit, known))
}

/// Parse a pgvector literal `[1.5,2.0,...]` into f64s. Lenient: skips
/// garbage entries rather than failing.
pub fn parse_vector_literal(s: &str) -> Vec<f64> {
    let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return Vec::new();
    }
    trimmed
        .split(',')
        .filter_map(|part| part.trim().parse::<f64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_text_joins_parts() {
        let text = build_embed_text(
            "Title",
            "A dragon story",
            &["Harry Potter".into(), "Angst".into()],
        );
        assert!(text.contains("Title"));
        assert!(text.contains("A dragon story"));
        assert!(text.contains("Harry Potter"));
        assert!(text.contains("Angst"));
        assert!(text.contains('|'));
    }

    #[test]
    fn content_hash_stable_and_sensitive() {
        let h1 = content_hash("same text");
        let h2 = content_hash("same text");
        let h3 = content_hash("different");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn parse_vector_literal_handles_real_and_garbage() {
        let v = parse_vector_literal("[1.5,2.0,3.25]");
        assert_eq!(v, vec![1.5, 2.0, 3.25]);
        assert!(parse_vector_literal("[]").is_empty());
        // Garbage entries are skipped, valid ones survive.
        let v = parse_vector_literal("[1.0,oops,2.0]");
        assert_eq!(v, vec![1.0, 2.0]);
    }
}
