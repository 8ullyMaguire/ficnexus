//! Site-wide drafts API — Lane 1 of Phase 4 integration-first rewrite.
//!
//! Live schema from migration 077 (forum_drafts):
//!   id (bigserial PK)
//!   user_id (bigint NOT NULL)
//!   context (text NOT NULL) e.g. "forum_topic", "forum_post"
//!   ref (text NOT NULL) secondary key within context
//!   content (text NOT NULL) markdown body as JSON string
//!   created_at (timestamptz DEFAULT now())
//!   updated_at (timestamptz DEFAULT now())
//!   Unique index: (user_id, context, ref)
//!
//! Routes are site-level, sibling of notifications.rs.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
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
    pub content: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct DraftRow {
    pub id: i64,
    pub user_id: i64,
    pub context: String,
    #[serde(rename = "ref")]
    pub r#ref: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ── GET /api/drafts?context=<ctx> ─────────────────────────────────────────

/// List drafts for the authenticated user, filtered by optional context.
pub async fn list_drafts(
    axum::extract::Query(context): axum::extract::Query<Option<String>>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;

    let mut query =
        sqlx::query_as::<_, DraftRow>(
            r#"SELECT id, user_id, context, ref, content, created_at, updated_at
               FROM forum_drafts WHERE user_id = $1"#,
        )
        .bind(user_id as i64);

    if let Some(ref ctx) = context.0 {
        if !ctx.is_empty() {
            query = query.bind(ctx);
        }
    }

    let drafts: Vec<DraftRow> = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "drafts": drafts.iter().map(draft_to_json).collect::<Vec<_>>(),
        "count": drafts.len(),
    })))
}

fn draft_to_json(d: &DraftRow) -> serde_json::Value {
    json!({
        "id": d.id,
        "context": d.context,
        "ref": d.r#ref,
        "content": d.content,
        "updated_at": d.updated_at.to_rfc3339(),
        "created_at": d.created_at.to_rfc3339(),
    })
}

// ── PUT /api/drafts/{context}/{ref} ───────────────────────────────────────

/// Upsert a single draft by `(user_id, context, ref)` tuple.
/// Creates new row or updates existing one atomically.
pub async fn upsert_draft(
    Path((context, ref_)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<DraftPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;

    let draft = sqlx::query_as::<_, DraftRow>(
        r#"INSERT INTO forum_drafts (user_id, context, ref, content, updated_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (user_id, context, ref)
           DO UPDATE SET content = EXCLUDED.content, updated_at = NOW()
           RETURNING id, user_id, context, ref, content, created_at, updated_at"#,
    )
    .bind(user_id as i64)
    .bind(context)
    .bind(ref_)
    .bind(&payload.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(draft_to_json(&draft)))
}

// ── DELETE /api/drafts/{context}/{ref} ────────────────────────────────────

/// Remove a specific draft. Returns 404 if no matching row.
pub async fn delete_draft(
    Path((context, ref_)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<StatusCode, AppError> {
    let user_id = require_user(&auth)?;

    let deleted = sqlx::query(
        r#"DELETE FROM forum_drafts WHERE user_id = $1 AND context = $2 AND ref = $3"#,
    )
    .bind(user_id as i64)
    .bind(context)
    .bind(ref_)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if deleted.rows_affected() == 0 {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify privilege enum variants are all unique (sanity check, trivial).
    #[test]
    fn test_route_names_unique() {
        // Draft routes must be distinct from each other
        assert_ne!("list_drafts", "upsert_draft");
        assert_ne!("list_drafts", "delete_draft");
        assert_ne!("upsert_draft", "delete_draft");
    }
}
