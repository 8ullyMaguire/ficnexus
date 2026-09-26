//! Ask-the-Docs — retrieve doc sections by embedding similarity.
//!
//! - POST /api/admin/docs/ingest — admin: read doc-sections.json (from the
//!   docs build), embed each section with nomic-embed-text, upsert into
//!   doc_sections (migration 036).
//! - GET /api/docs/ask?q=... — embed the question, cosine-search the top
//!   sections, return them with anchors (citations). Graceful fallback:
//!   when Ollama is down, fall back to a keyword (ILIKE) scan.
//!
//! Same pattern as roadmap consensus (embed + pgvector <=>), so no new
//! infrastructure.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// GET /api/docs/ask?q=... — Ask-the-Docs. Public.
#[derive(Deserialize)]
pub struct AskDocsParams {
    pub q: String,
    #[serde(default = "default_n")]
    pub n: usize,
}

fn default_n() -> usize {
    3
}

pub async fn ask_docs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AskDocsParams>,
) -> Result<Json<Value>, AppError> {
    let q = params.q.trim();
    if q.is_empty() {
        return Err(AppError::BadRequest("q is required".to_string()));
    }
    let n = params.n.clamp(1, 8);

    // Embed the question (best-effort; fall back to keyword scan).
    let embedding = state
        .ollama
        .embed(q)
        .await
        .map_err(|e| tracing::warn!("ask-docs embed failed: {e}"))
        .ok();

    let rows: Vec<(String, String, Option<String>, String, String, String, f64)> =
        if let Some(emb) = embedding {
            let emb_sql = format!(
                "[{}]",
                emb.iter()
                    .map(|f| format!("{f:.6}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            sqlx::query_as(
                r#"SELECT slug, page, anchor, title, feature,
                      LEFT(body, 400) AS snippet,
                      (embedding <=> $1::vector) AS dist
               FROM doc_sections
               WHERE embedding IS NOT NULL
               ORDER BY embedding <=> $1::vector
               LIMIT $2"#,
            )
            .bind(emb_sql)
            .bind(n as i64)
            .fetch_all(&state.db)
            .await?
        } else {
            // Fallback: keyword scan on title/body/keywords.
            let pat = format!("%{}%", q.to_lowercase());
            sqlx::query_as(
                r#"SELECT slug, page, anchor, title, feature,
                      LEFT(body, 400) AS snippet,
                      0.0 AS dist
               FROM doc_sections
               WHERE LOWER(title) LIKE $1 OR LOWER(body) LIKE $1 OR EXISTS (
                   SELECT 1 FROM unnest(keywords) k WHERE LOWER(k) LIKE $1
               )
               ORDER BY id
               LIMIT $2"#,
            )
            .bind(pat)
            .bind(n as i64)
            .fetch_all(&state.db)
            .await?
        };

    let hits: Vec<Value> = rows
        .into_iter()
        .map(|(slug, page, anchor, title, feature, snippet, dist)| {
            json!({
                "slug": slug, "page": page, "anchor": anchor,
                "title": title, "feature": feature, "snippet": snippet,
                "score": (1.0 - dist).max(0.0).min(1.0),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "query": q, "results": hits })))
}

/// POST /api/admin/docs/ingest — admin: (re)build the doc_sections table
/// from frontend/static/docs/doc-sections.json + Ollama embeddings.
/// Idempotent: upserts by slug. Safe to re-run after a docs rebuild.
pub async fn ingest_docs(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    crate::services::trust::require_admin_tier(&user, &state)?;

    let path = state
        .config
        .frontend_dir
        .join("docs")
        .join("doc-sections.json");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| AppError::BadRequest(format!("doc-sections.json not found: {e}")))?;
    let sections: Vec<Value> = serde_json::from_str(&raw)
        .map_err(|e| AppError::BadRequest(format!("doc-sections.json parse error: {e}")))?;

    let mut inserted = 0i64;
    let mut failed = 0i64;
    let mut total = sections.len() as i64;

    for sec in &sections {
        let slug = sec["slug"].as_str().unwrap_or_default();
        let page = sec["page"].as_str().unwrap_or_default();
        let anchor = sec["anchor"].as_str();
        let title = sec["title"].as_str().unwrap_or_default();
        let body = sec["body"].as_str().unwrap_or_default();
        let feature = sec["feature"].as_str().unwrap_or("home");
        let keywords: Vec<String> = sec["keywords"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        if slug.is_empty() || page.is_empty() {
            total -= 1;
            continue;
        }

        // Embed (best-effort; NULL embedding = keyword-fallback only).
        let embedding = state
            .ollama
            .embed(&format!("{title}\n{body}"))
            .await
            .map_err(|e| tracing::warn!("ingest embed failed for {slug}: {e}"))
            .ok();

        let res = if let Some(emb) = embedding {
            let emb_sql = format!(
                "[{}]",
                emb.iter()
                    .map(|f| format!("{f:.6}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            sqlx::query(
                r#"INSERT INTO doc_sections (slug, page, anchor, title, body, keywords, feature, embedding)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8::vector)
                   ON CONFLICT (slug) DO UPDATE SET
                     page = EXCLUDED.page, anchor = EXCLUDED.anchor,
                     title = EXCLUDED.title, body = EXCLUDED.body,
                     keywords = EXCLUDED.keywords, feature = EXCLUDED.feature,
                     embedding = EXCLUDED.embedding"#,
            )
            .bind(slug)
            .bind(page)
            .bind(anchor)
            .bind(title)
            .bind(body)
            .bind(&keywords)
            .bind(feature)
            .bind(emb_sql)
            .execute(&state.db)
            .await
        } else {
            sqlx::query(
                r#"INSERT INTO doc_sections (slug, page, anchor, title, body, keywords, feature, embedding)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, NULL)
                   ON CONFLICT (slug) DO UPDATE SET
                     page = EXCLUDED.page, anchor = EXCLUDED.anchor,
                     title = EXCLUDED.title, body = EXCLUDED.body,
                     keywords = EXCLUDED.keywords, feature = EXCLUDED.feature,
                     embedding = NULL"#,
            )
            .bind(slug)
            .bind(page)
            .bind(anchor)
            .bind(title)
            .bind(body)
            .bind(&keywords)
            .bind(feature)
            .execute(&state.db)
            .await
        };

        match res {
            Ok(r) => inserted += r.rows_affected() as i64,
            Err(e) => {
                failed += 1;
                tracing::warn!("ingest insert failed for {slug}: {e}");
            }
        }
    }

    Ok(Json(json!({
        "err": 0, "total": total, "inserted": inserted, "failed": failed,
        "msg": "Docs index rebuilt",
    })))
}
