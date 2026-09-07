//! Site-wide drafts API — Lane 1 of Phase 4 integration-first rewrite.
//!
//! Live schema from migration 077 (verified against prod 2026-09-07):
//!   id (bigserial PK)
//!   user_id (int4 NOT NULL)
//!   topic_id (bigint NULL FK forum_topics, NULL = new-topic draft)
//!   category_id (bigint NULL FK forum_categories, NULL on delete)
//!   title (text NULL)
//!   body (text NOT NULL) — markdown
//!   payload (jsonb NOT NULL DEFAULT '{}')
//!   poll_data (jsonb NULL) — {question, options[], ...}
//!   updated_at / created_at (timestamptz)
//!
//! Keying: one draft per (user, topic) for replies (`topic_id` set,
//! `category_id` NULL), one draft per (user, category) for new topics
//! (`topic_id` NULL, `category_id` set). The `context`/`ref` URL segments
//! are a site-wide namespacing convenience decoded as:
//!   context=forum_post  ref={topicId}  → reply draft (topic_id)
//!   context=forum_topic ref=new:{slug} → new-topic draft (category_id)
//!   context=forum_topic ref={topicId}  → topic-edit draft (topic_id)
//! Unknown contexts → 400. Future contexts (e.g. `comment`) extend the
//! match without DDL changes.
//!
//! Routes are site-level, sibling of notifications.rs.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::require_user;
use crate::server::AppState;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DraftPayload {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub poll_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct DraftListQuery {
    #[serde(default)]
    pub context: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DraftRow {
    pub id: i64,
    pub user_id: i32,
    pub topic_id: Option<i64>,
    pub category_id: Option<i64>,
    pub title: Option<String>,
    pub body: String,
    pub payload: serde_json::Value,
    pub poll_data: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Decode `(context, ref)` URL segments into a `(topic_id, category_id)`
/// key. Returns the context back for re-encoding on the way out.
async fn decode_key(
    db: &sqlx::PgPool,
    context: &str,
    ref_: &str,
) -> Result<(Option<i64>, Option<i64>), AppError> {
    match context {
        "forum_post" => {
            let topic_id: i64 = ref_.parse().map_err(|_| {
                AppError::BadRequest("ref must be a topic id for forum_post".to_string())
            })?;
            Ok((Some(topic_id), None))
        }
        "forum_topic" => {
            if let Some(slug) = ref_.strip_prefix("new:") {
                let category_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM forum_categories WHERE slug = $1",
                )
                .bind(slug)
                .fetch_optional(db)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
                match category_id {
                    Some(id) => Ok((None, Some(id))),
                    None => Err(AppError::NotFound("Unknown category".to_string())),
                }
            } else {
                let topic_id: i64 = ref_.parse().map_err(|_| {
                    AppError::BadRequest(
                        "ref must be a topic id or new:{categorySlug}".to_string(),
                    )
                })?;
                Ok((Some(topic_id), None))
            }
        }
        _ => Err(AppError::BadRequest(
            "unknown draft context (want forum_post|forum_topic)".to_string(),
        )),
    }
}

fn draft_to_json(d: &DraftRow, context: &str, ref_: &str) -> serde_json::Value {
    json!({
        "id": d.id,
        "context": context,
        "ref": ref_,
        "topic_id": d.topic_id,
        "category_id": d.category_id,
        "title": d.title,
        "body": d.body,
        "payload": d.payload,
        "poll_data": d.poll_data,
        "updated_at": d.updated_at.to_rfc3339(),
        "created_at": d.created_at.to_rfc3339(),
    })
}

// ── GET /api/drafts?context= ───────────────────────────────────────────────

/// List own drafts, optionally filtered by context.
pub async fn list_drafts(
    Query(q): Query<DraftListQuery>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;

    if let Some(ctx) = q.context.as_deref() {
        if !matches!(ctx, "forum_post" | "forum_topic") {
            return Err(AppError::BadRequest(
                "unknown draft context (want forum_post|forum_topic)".to_string(),
            ));
        }
    }

    // forum_post + forum_topic drafts are both topic-keyed; new-topic
    // drafts are the category-keyed ones (topic_id IS NULL).
    let rows: Vec<DraftRow> = match q.context.as_deref() {
        Some("forum_topic") => sqlx::query_as::<_, DraftRow>(
            r#"SELECT id, user_id, topic_id, category_id, title, body, payload,
                      poll_data, created_at, updated_at
               FROM forum_drafts WHERE user_id = $1 AND topic_id IS NULL
               ORDER BY updated_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?,
        _ => sqlx::query_as::<_, DraftRow>(
            r#"SELECT id, user_id, topic_id, category_id, title, body, payload,
                      poll_data, created_at, updated_at
               FROM forum_drafts WHERE user_id = $1
               ORDER BY updated_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?,
    };

    // Re-encode keys: reply/edit drafts → their context by shape is
    // ambiguous from the row alone, so report both coordinates and let
    // the client re-address the draft it saved.
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|d| {
            let (ctx, r) = match (d.topic_id, d.category_id) {
                (Some(t), _) => ("forum_post", t.to_string()),
                (None, Some(_)) => ("forum_topic", "new".to_string()),
                (None, None) => ("forum_topic", "new".to_string()),
            };
            draft_to_json(d, ctx, &r)
        })
        .collect();
    Ok(Json(json!({ "err": 0, "drafts": items, "count": items.len() })))
}

// ── PUT /api/drafts/{context}/{ref} ────────────────────────────────────────

/// Upsert a single draft keyed by `(user_id, topic_id, category_id)`.
pub async fn upsert_draft(
    Path((context, ref_)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<DraftPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;
    let (topic_id, category_id) = decode_key(&state.db, &context, &ref_).await?;

    let body = payload.body.unwrap_or_default();
    if body.chars().count() > 100_000 {
        return Err(AppError::BadRequest(
            "draft body too long (max 100000 chars)".to_string(),
        ));
    }

    let mut tx = state.db.begin().await.map_err(|e| AppError::Database(e.to_string()))?;
    let updated = sqlx::query(
        r#"UPDATE forum_drafts
           SET title = $4, body = $5, payload = COALESCE($6, payload),
               poll_data = $7, updated_at = NOW()
           WHERE user_id = $1
             AND COALESCE(topic_id, -1) = COALESCE($2, -1)
             AND COALESCE(category_id, -1) = COALESCE($3, -1)"#,
    )
    .bind(user_id)
    .bind(topic_id)
    .bind(category_id)
    .bind(payload.title.as_deref())
    .bind(&body)
    .bind(payload.payload.clone())
    .bind(payload.poll_data.clone())
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    if updated.rows_affected() == 0 {
        sqlx::query(
            r#"INSERT INTO forum_drafts (user_id, topic_id, category_id, title, body, payload, poll_data, updated_at)
               VALUES ($1, $2, $3, $4, $5, COALESCE($6, '{}'::jsonb), $7, NOW())"#,
        )
        .bind(user_id)
        .bind(topic_id)
        .bind(category_id)
        .bind(payload.title.as_deref())
        .bind(&body)
        .bind(payload.payload)
        .bind(payload.poll_data)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    let draft = sqlx::query_as::<_, DraftRow>(
        r#"SELECT id, user_id, topic_id, category_id, title, body, payload,
                  poll_data, created_at, updated_at
           FROM forum_drafts
           WHERE user_id = $1
             AND COALESCE(topic_id, -1) = COALESCE($2, -1)
             AND COALESCE(category_id, -1) = COALESCE($3, -1)"#,
    )
    .bind(user_id)
    .bind(topic_id)
    .bind(category_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    tx.commit().await.map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(draft_to_json(&draft, &context, &ref_)))
}

// ── DELETE /api/drafts/{context}/{ref} ─────────────────────────────────────

/// Remove a specific draft. Idempotent — missing row returns `{err:0,
/// deleted:false}` (matches the site blocks unblock convention).
pub async fn delete_draft(
    Path((context, ref_)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;
    let (topic_id, category_id) = decode_key(&state.db, &context, &ref_).await?;

    let deleted = sqlx::query(
        r#"DELETE FROM forum_drafts
           WHERE user_id = $1
             AND COALESCE(topic_id, -1) = COALESCE($2, -1)
             AND COALESCE(category_id, -1) = COALESCE($3, -1)"#,
    )
    .bind(user_id)
    .bind(topic_id)
    .bind(category_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "err": 0, "deleted": deleted.rows_affected() > 0 })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draft_payload_defaults_empty() {
        let p: DraftPayload = serde_json::from_value(json!({})).expect("empty payload");
        assert!(p.title.is_none());
        assert!(p.body.is_none());
        assert!(p.payload.is_none());
        assert!(p.poll_data.is_none());
    }

    #[test]
    fn test_draft_to_json_round_trips_key() {
        let row = DraftRow {
            id: 1,
            user_id: 7,
            topic_id: Some(42),
            category_id: None,
            title: None,
            body: "hello".to_string(),
            payload: json!({}),
            poll_data: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let v = draft_to_json(&row, "forum_post", "42");
        assert_eq!(v["context"], "forum_post");
        assert_eq!(v["ref"], "42");
        assert_eq!(v["topic_id"], 42);
        assert_eq!(v["body"], "hello");
    }
}
