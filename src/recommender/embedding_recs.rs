//! Embeddings-based recommendations endpoint (GET /api/recommendations/embeddings).
//!
//! Thin HTTP wrapper around the pgvector cosine-similarity machinery that the
//! `embeddings` strategy uses: build a recency-ordered seed set from the
//! user's reading history + bookmarks, average their `works.embedding`
//! vectors, and return the nearest neighbour works by cosine similarity.
//!
//! Degradation contract (frontend hides the section silently):
//! * anonymous → `{err:0, enough_data:false, recs:[], based_on:[]}`
//! * no seeds / no embedded works → same empty shape
//! * never 500s on missing data — embeddings being unpopulated is a NORMAL
//!   state of the archive (the training worker fills them over time).

use axum::{Json, extract::State};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Max seeds considered (most recent first across history + bookmarks).
const MAX_SEEDS: i64 = 20;
/// Cap on returned recommendations.
const MAX_RESULTS: i64 = 12;

#[derive(Debug, serde::Deserialize)]
pub struct EmbeddingRecsParams {
    /// How many recs to return (default 6, max 12).
    pub n: Option<i64>,
}

/// One seed: the works.embedding row id (works.id::text) + display info.
#[derive(Debug, sqlx::FromRow)]
struct SeedRow {
    work_key: String,
    title: Option<String>,
    url_id: Option<String>,
}

/// GET /api/recommendations/embeddings
///
/// Top-N works by cosine similarity between the average embedding of the
/// user's recently-read/bookmarked works and every other embedded work.
pub async fn embedding_recommendations_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<EmbeddingRecsParams>,
    auth: AuthUser,
) -> Json<Value> {
    let empty = || {
        Json(json!({
            "err": 0,
            "enough_data": false,
            "recs": [],
            "based_on": [],
            "source": "embeddings",
        }))
    };

    let Some(user_id) = auth.user_id else {
        return empty();
    };
    let n = params.n.unwrap_or(6).clamp(1, MAX_RESULTS);

    // ── Seeds: reading history (works.id) + bookmarks (url_id/work_id) ──
    // reading_history.work_id is an int FK to works.id; bookmarks carries
    // both url_id (text, always set) and work_id (int, nullable).
    let history_seeds: Vec<SeedRow> = sqlx::query_as(
        r#"SELECT DISTINCT w.id::text AS work_key,
                  COALESCE(w.canonical_title, '') AS title,
                  w.default_source_id AS url_id
           FROM reading_history rh
           JOIN works w ON w.id = rh.work_id
           WHERE rh.user_id = $1
           ORDER BY work_key
           LIMIT $2"#,
    )
    .bind(user_id)
    .bind(MAX_SEEDS)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let bookmark_seeds: Vec<SeedRow> = sqlx::query_as(
        r#"SELECT DISTINCT COALESCE(w.id::text, b.url_id) AS work_key,
                  COALESCE(w.canonical_title, '') AS title,
                  w.default_source_id AS url_id
           FROM bookmarks b
           LEFT JOIN works w
             ON (b.work_id IS NOT NULL AND w.id = b.work_id)
                OR (b.work_id IS NULL AND w.default_source_id = b.url_id)
           WHERE b.user_id = $1
           ORDER BY work_key
           LIMIT $2"#,
    )
    .bind(user_id)
    .bind(MAX_SEEDS)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    // Dedup seeds by work_key, keeping order (history first = most recent).
    let mut seed_keys: Vec<String> = Vec::new();
    let mut based_on: Vec<Value> = Vec::new();
    for s in history_seeds.into_iter().chain(bookmark_seeds.into_iter()) {
        if seed_keys.iter().any(|k| k == &s.work_key) {
            continue;
        }
        // Only seeds that actually have an embedding can anchor the query.
        let has_emb: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM works WHERE id::text = $1 AND embedding IS NOT NULL)",
        )
        .bind(&s.work_key)
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);
        if !has_emb {
            continue;
        }
        seed_keys.push(s.work_key.clone());
        if based_on.len() < 3 && !s.title.as_deref().unwrap_or("").is_empty() {
            based_on.push(json!({ "title": s.title, "url_id": s.url_id }));
        }
        if seed_keys.len() >= MAX_SEEDS as usize {
            break;
        }
    }

    if seed_keys.is_empty() {
        return empty();
    }

    // ── Nearest neighbours: avg cosine across seeds ────────────────────
    // One row per candidate work; AVG(1 - distance) over per-seed probes.
    // `$1::int[]` binds the seed works.id list (work_key is works.id::text).
    let seed_ids: Vec<i64> = seed_keys
        .iter()
        .filter_map(|k| k.parse::<i64>().ok())
        .collect();
    if seed_ids.is_empty() {
        return empty();
    }
    let rows: Vec<(String, String, Option<String>, Option<i64>, Option<i32>, Option<String>, Option<String>, Option<String>, f64)> =
        sqlx::query_as(
            r#"SELECT COALESCE(w.default_source_id, w.id::text) AS url_id,
                      COALESCE(NULLIF(w.canonical_title, ''), COALESCE(fi.title, 'Untitled')) AS title,
                      COALESCE(NULLIF(w.canonical_author, ''), COALESCE(fi.author, 'Anonymous')) AS author,
                      fi.words, fi.chapters, fi.status, fi.source, fi.description,
                      AVG(1.0 - (w.embedding <=> s.embedding))::float8 AS cosine
               FROM works s
               JOIN works w
                 ON w.embedding IS NOT NULL
                AND w.is_visible
                AND w.id <> s.id
               LEFT JOIN fic_info fi ON fi.id = w.default_source_id
               WHERE s.id = ANY($1)
               GROUP BY w.id, url_id, title, author, fi.words, fi.chapters, fi.status, fi.source, fi.description
               ORDER BY cosine DESC
               LIMIT $2"#,
        )
        .bind(&seed_ids)
        .bind(n)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

    if rows.is_empty() {
        return empty();
    }

    let recs: Vec<Value> = rows
        .into_iter()
        .map(
            |(url_id, title, author, words, chapters, status, source, description, cosine)| {
                json!({
                    "url_id": url_id,
                    "title": title,
                    "author": author,
                    "words": words.unwrap_or(0),
                    "chapters": chapters.unwrap_or(0),
                    "status": status.unwrap_or_else(|| "Unknown".into()),
                    "site_domain": source,
                    "summary": description.unwrap_or_default(),
                    "score": cosine,
                    "strategy": "embeddings",
                })
            },
        )
        .collect();

    Json(json!({
        "err": 0,
        "enough_data": true,
        "recs": recs,
        "based_on": based_on,
        "source": "embeddings",
    }))
}
