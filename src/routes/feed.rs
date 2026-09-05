use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct FeedQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

/// GET /api/v1/feed — recent activity from works the user follows
pub async fn feed_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<FeedQuery>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 50);
    let offset = (page - 1) * per_page;

    // Get total count first
    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(DISTINCT f.work_id)
           FROM follows f
           JOIN fic_info fi ON fi.work_id = f.work_id
           JOIN export_log el ON el.url_id = fi.id
           WHERE f.follower_id = $1 AND f.work_id IS NOT NULL"#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Get most recent export per followed work, ordered by newest first
    let items: Vec<Value> = sqlx::query_as::<_, FeedRow>(
        r#"SELECT sub.work_id, sub.url_id, sub.title, sub.author,
                  sub.etype, sub.created, sub.export_hash
           FROM (
             SELECT DISTINCT ON (f.work_id)
               f.work_id,
               el.url_id,
               fi.title,
               fi.author,
               el.etype,
               el.created,
               el.export_hash
             FROM follows f
             JOIN fic_info fi ON fi.work_id = f.work_id
             JOIN export_log el ON el.url_id = fi.id
             WHERE f.follower_id = $1 AND f.work_id IS NOT NULL
             ORDER BY f.work_id, el.created DESC
           ) sub
           ORDER BY sub.created DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .into_iter()
    .map(|row| {
        let cache_url = format!("/cache/{}/{}?h={}", row.etype, row.url_id, row.export_hash);
        let updated = row.created.map(|c| c.to_rfc3339()).unwrap_or_default();
        json!({
            "work_id": row.work_id,
            "url_id": row.url_id,
            "title": row.title,
            "author": row.author,
            "updated_at": updated,
            "format": row.etype,
            "url": cache_url,
        })
    })
    .collect();

    Ok(Json(json!({
        "err": 0,
        "items": items,
        "page": page,
        "per_page": per_page,
        "total": total,
    })))
}

/// Row returned by the feed query
#[derive(Debug, sqlx::FromRow)]
struct FeedRow {
    work_id: i32,
    url_id: String,
    title: String,
    author: String,
    etype: String,
    created: Option<chrono::DateTime<chrono::Utc>>,
    export_hash: String,
}
