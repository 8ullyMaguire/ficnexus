use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Query parameters for the similar-by-vector endpoint.
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub limit: Option<usize>,
}

/// GET /api/search/similar/{work_id} — "More like this" vector search
///
/// Uses pgvector cosine distance on the `works.embedding` column (384-dim).
/// Falls back to tag-based similarity when the source work has no embedding.
pub async fn similar_by_vector(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).max(1).min(50);

    // Check if the source work has an embedding — fetch as text for pgvector compat
    let embedding_text: Option<String> = sqlx::query_scalar(
        "SELECT embedding::text FROM works WHERE id = $1 AND embedding IS NOT NULL",
    )
    .bind(work_id)
    .fetch_optional(&state.db)
    .await?;

    let results = if let Some(ref emb_text) = embedding_text {
        // Use pgvector cosine distance (<=> operator)
        let rows = sqlx::query_as::<_, (i32, String, Option<f64>)>(
            r#"SELECT w.id, w.canonical_title,
                      (w.embedding <=> $1::vector) AS distance
               FROM works w
               WHERE w.id != $2 AND w.embedding IS NOT NULL
               ORDER BY distance ASC
               LIMIT $3"#,
        )
        .bind(emb_text)
        .bind(work_id)
        .bind(limit as i64)
        .fetch_all(&state.db)
        .await?;

        rows.into_iter()
            .map(|(id, title, dist)| {
                json!({
                    "work_id": id,
                    "title": title,
                    "similarity": dist.map(|d| format!("{:.4}", 1.0 - d)),
                })
            })
            .collect::<Vec<_>>()
    } else {
        // Fallback: tag-based similarity if no embedding on the source work
        let rows = sqlx::query_as::<_, (i32, String, i64)>(
            r#"SELECT w2.id, w2.canonical_title,
                      COUNT(DISTINCT wt2.tag_id) AS common_tags
               FROM works w
               JOIN fic_info fi ON fi.work_id = w.id
               JOIN fic_tags wt1 ON wt1.url_id = fi.id
               JOIN fic_tags wt2 ON wt2.tag_id = wt1.tag_id AND wt2.url_id != wt1.url_id
               JOIN works w2 ON w2.id = fi.work_id
               WHERE w.id = $1 AND w2.embedding IS NULL
               GROUP BY w2.id, w2.canonical_title
               ORDER BY common_tags DESC
               LIMIT $2"#,
        )
        .bind(work_id)
        .bind(limit as i64)
        .fetch_all(&state.db)
        .await?;

        rows.into_iter()
            .map(|(id, title, common)| {
                json!({
                    "work_id": id,
                    "title": title,
                    "common_tags": common,
                })
            })
            .collect::<Vec<_>>()
    };

    Ok(Json(json!({"err": 0, "items": results})))
}

/// GET /api/v1/works/{id}/also-bookmarked — "readers also bookmarked" recs
/// from the precomputed fic_bookmark_cooccur table.
///
/// The co-occurrence table stores fic_info.id pairs as `work_a < work_b` with
/// a `cooccur_count`. For a given fic we must probe BOTH orientations
/// (work_a = $1 AND work_b = $1) — the pair ordering is normalized, not
/// anchored on the seed. We then look up the partner's fic_info row (title,
/// author, words, site) and rank by cooccur_count, top 5. Zero-PII: only
/// aggregate co-occurrence counts, never who bookmarked what.
pub async fn also_bookmarked(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    // Both orientations: the table normalizes pairs as work_a < work_b, so the
    // requested fic can appear in either column.
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT CASE
                   WHEN work_a = $1 THEN work_b
                   ELSE work_a
               END AS partner_id,
               cooccur_count
        FROM fic_bookmark_cooccur
        WHERE work_a = $1 OR work_b = $1
        ORDER BY cooccur_count DESC, partner_id
        LIMIT 5
        "#,
    )
    .bind(&url_id)
    .fetch_all(&state.db)
    .await?;

    let mut items: Vec<Value> = Vec::with_capacity(rows.len());
    for (partner_id, count) in rows {
        // The partner row may not exist locally (co-occurrence is populated
        // from remote bookmark exports) — degrade to id-only then.
        let meta: Option<(String, String, i64, String, String)> = sqlx::query_as(
            r#"SELECT title, author, words, source, status
               FROM fic_info WHERE id = $1"#,
        )
        .bind(&partner_id)
        .fetch_optional(&state.db)
        .await?;

        let (title, author, words, source, status) = meta.unwrap_or_else(|| {
            (
                partner_id.clone(),
                String::new(),
                0,
                String::new(),
                String::new(),
            )
        });

        // fic_info.id may not be linked to a work yet; the UI links via
        // /fic/{url_id} when present.
        let work_id: Option<i32> = sqlx::query_scalar("SELECT work_id FROM fic_info WHERE id = $1")
            .bind(&partner_id)
            .fetch_optional(&state.db)
            .await?;

        items.push(json!({
            "url_id": partner_id,
            "work_id": work_id,
            "title": title,
            "author": author,
            "words": words,
            "status": status,
            "site_domain": source,
            "cooccur_count": count,
        }));
    }

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "items": items,
    })))
}

/// GET /api/works/random — a random fic for "Surprise me" / Blind Date.
///
/// Unlike `/api/blind-date` (which hides the title behind a signed reveal),
/// this returns FULL metadata — the UI links straight to the fic page. The
/// pick is `ORDER BY random() LIMIT 1` with a `work_id`-optional shape so
/// the frontend can link via `/fic/{url_id}`. 200 with `fic: null` when the
/// archive is empty.
pub async fn random_work(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let row: Option<(
        String,
        Option<i32>,
        String,
        String,
        String,
        i64,
        i32,
        String,
        String,
    )> = sqlx::query_as(
        r#"
            SELECT id, work_id, title, author, source, words, chapters, status, description
            FROM fic_info
            WHERE btrim(description) <> ''
            ORDER BY random()
            LIMIT 1
            "#,
    )
    .fetch_optional(&state.db)
    .await?;

    let Some((id, work_id, title, author, source, words, chapters, status, description)) = row
    else {
        return Ok(Json(json!({ "err": 0, "fic": null })));
    };

    Ok(Json(json!({
        "err": 0,
        "fic": {
            "url_id": id,
            "work_id": work_id,
            "title": title,
            "author": author,
            "source": source,
            "words": words,
            "chapters": chapters,
            "status": status,
            "description": description,
        },
    })))
}

/// GET /api/works/{id}/similar — collaborative filtering via bookmarks
///
/// Finds users who bookmarked this work, then finds what else they bookmarked.
/// Results are ranked by co-occurrence (number of shared bookmarkers).
pub async fn similar_by_bookmarks(
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i32, String, i64)>(
        r#"SELECT w.id, w.canonical_title,
                  COUNT(DISTINCT b2.user_id) AS co_bookmarkers
           FROM bookmarks b1
           JOIN bookmarks b2 ON b2.user_id = b1.user_id AND b2.work_id != b1.work_id
           JOIN works w ON w.id = b2.work_id
           WHERE b1.work_id = $1
           GROUP BY w.id, w.canonical_title
           ORDER BY co_bookmarkers DESC
           LIMIT 20"#,
    )
    .bind(work_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": rows.into_iter().map(|(id, title, count)| json!({
            "work_id": id,
            "title": title,
            "co_bookmarkers": count,
        })).collect::<Vec<_>>(),
    })))
}

/// GET /api/search/chips — personalized "For You" search chips.
///
/// Auth-optional: logged-out users get static chips; logged-in users get chips
/// derived from their top bookmark tag affinities. The embedding centroid is
/// computed and logged (usage_events) even when chips are tag-derived so the
/// vector-path stays warm for future use.
pub async fn search_chips_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    // Static defaults for unauthenticated users.
    if auth.user_id.is_none() {
        return Ok(Json(json!({
            "chips": ["Popular", "Updated Today", "Completed", "One-Shot"],
            "personalized": false,
        })));
    };

    let user_id = match auth.user_id {
        Some(id) => id,
        None => {
            return Ok(Json(json!({
                "chips": ["Popular", "Updated Today", "Completed", "One-Shot"],
                "personalized": false,
            })));
        }
    };

    // 1. Compute the embedding centroid from recent bookmarks (for logging only).
    let embedding_texts: Vec<String> = sqlx::query_scalar(
        r#"SELECT re.embedding::text
           FROM rec_embeddings re
           JOIN bookmarks b ON b.work_id = re.work_id
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC
           LIMIT 20"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if !embedding_texts.is_empty() {
        let centroid_computed = compute_centroid(&embedding_texts);
        let _ = sqlx::query(
            r#"INSERT INTO usage_events (client_id, path, event_type, user_agent)
               VALUES (NULL, '/api/search/chips', 'chips_centroid_computed', '')"#,
        )
        .execute(&state.db)
        .await;
        let _ = centroid_computed;
    }

    // 2. Derive chips from the user's top tag affinities (cheaper + explainable).
    let top_tags: Vec<String> = sqlx::query_scalar(
        r#"SELECT t.name
           FROM bookmarks b
           JOIN work_tags wt ON wt.work_id = b.work_id
           JOIN tags t ON t.id = wt.tag_id
           WHERE b.user_id = $1
           GROUP BY t.name
           ORDER BY COUNT(*) DESC
           LIMIT 3"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let chips: Vec<String> = if top_tags.is_empty() {
        vec![
            "Popular".to_string(),
            "Updated Today".to_string(),
            "Completed".to_string(),
        ]
    } else {
        top_tags
            .into_iter()
            .map(|tag| format!("for-you:{}", tag))
            .collect()
    };

    Ok(Json(json!({
        "chips": chips,
        "personalized": true,
    })))
}

/// Parse a pgvector text literal `'[v1,v2,...]'` into a Vec<f32>.
fn parse_vector(text: &str) -> Vec<f32> {
    let text = text.trim();
    let text = text.strip_prefix('[').unwrap_or(text);
    let text = text.strip_suffix(']').unwrap_or(text);
    text.split(',')
        .filter_map(|s| s.trim().parse::<f32>().ok())
        .collect()
}

/// Compute the element-wise mean of N pgvector embedding strings.
fn compute_centroid(texts: &[String]) -> Vec<f32> {
    if texts.is_empty() {
        return vec![];
    }
    let first = parse_vector(&texts[0]);
    if first.is_empty() {
        return vec![];
    }
    let dim = first.len();
    let mut sum = first;
    let mut valid_count = 1usize; // first is always valid
    for text in &texts[1..] {
        let vec = parse_vector(text);
        if vec.len() != dim {
            continue;
        }
        valid_count += 1;
        for (i, v) in vec.into_iter().enumerate() {
            sum[i] += v;
        }
    }
    let n = valid_count as f32;
    for v in &mut sum {
        *v /= n;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::{compute_centroid, parse_vector};

    #[test]
    fn parse_vector_parses_pgvector_literal() {
        assert_eq!(parse_vector("[1.0, 2.0, 3.0]"), vec![1.0, 2.0, 3.0]);
        assert_eq!(parse_vector("[0.5,-0.5]"), vec![0.5, -0.5]);
    }

    #[test]
    fn parse_vector_handles_whitespace() {
        assert_eq!(parse_vector("  [ 1.0 , 2.0 ]  "), vec![1.0, 2.0]);
    }

    #[test]
    fn compute_centroid_returns_mean_of_vectors() {
        let texts = vec!["[1.0, 2.0]".to_string(), "[3.0, 4.0]".to_string()];
        let centroid = compute_centroid(&texts);
        assert_eq!(centroid, vec![2.0, 3.0]); // (1+3)/2, (2+4)/2
    }

    #[test]
    fn compute_centroid_empty_input_returns_empty() {
        let texts: Vec<String> = vec![];
        assert!(compute_centroid(&texts).is_empty());
    }

    #[test]
    fn compute_centroid_mismatched_dims_skipped() {
        // Second vector has wrong dim → skipped, only first is averaged.
        let texts = vec!["[1.0, 2.0]".to_string(), "[3.0, 4.0, 5.0]".to_string()];
        let centroid = compute_centroid(&texts);
        assert_eq!(centroid, vec![1.0, 2.0]); // only first
    }
}
