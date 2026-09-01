use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    pub days: Option<i32>,
    pub limit: Option<i64>,
}

/// GET /api/v1/trending/tag/{tag_type_id}/{tag_name}
///
/// Returns fics with a specific tag that are trending (most viewed / downloaded).
pub async fn trending_by_tag_handler(
    State(state): State<Arc<AppState>>,
    Path((tag_type_id, tag_name)): Path<(i16, String)>,
    Query(params): Query<TrendingQuery>,
) -> Result<Json<Value>, AppError> {
    let days = params.days.unwrap_or(7).max(1).min(365);
    let limit = params.limit.unwrap_or(20).min(50).max(1);

    // Find the tag first (resolve aliases)
    let tag = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT id, name, tag_type_id FROM tags WHERE LOWER(name) = LOWER($1) AND tag_type_id = $2",
    )
    .bind(&tag_name)
    .bind(tag_type_id)
    .fetch_optional(&state.db)
    .await?
    .or_else(|| {
        // Check aliases
        None // handled by second query below
    });

    // Try alias lookup
    let tag = match tag {
        Some(t) => t,
        None => {
            // Check tag_aliases
            let alias = sqlx::query_scalar::<_, i32>(
                "SELECT canonical_tag_id FROM tag_aliases WHERE LOWER(alias_name) = LOWER($1)",
            )
            .bind(&tag_name)
            .fetch_optional(&state.db)
            .await?;

            match alias {
                Some(canonical_id) => {
                    sqlx::query_as::<_, (i32, String, i16)>(
                        "SELECT id, name, tag_type_id FROM tags WHERE id = $1",
                    )
                    .bind(canonical_id)
                    .fetch_optional(&state.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("Tag not found".into()))?
                }
                None => return Err(AppError::NotFound(format!("Tag '{}' not found", tag_name))),
            }
        }
    };

    // Get trending fics for this tag
    let rows = sqlx::query_as::<_, (String, String, String, i64, i32, String, i64, i64)>(
        r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, fi.status,
                  COALESCE(rl.request_count, 0) as requests,
                  COALESCE(rl.download_count, 0) as downloads
           FROM fic_tags ft
           JOIN fic_info fi ON fi.id = ft.url_id
           LEFT JOIN LATERAL (
               SELECT
                   COUNT(*) as request_count,
                   SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END) as download_count
               FROM request_log
               WHERE url_id = fi.id
                 AND created > NOW() - ($1 || ' days')::INTERVAL
           ) rl ON TRUE
           WHERE ft.tag_id = $2
           ORDER BY rl.request_count DESC NULLS LAST, fi.title ASC
           LIMIT $3"#,
    )
    .bind(days.to_string())
    .bind(tag.0)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows.into_iter().map(|(id, title, author, words, chapters, status, requests, downloads)| {
        json!({
            "url_id": id,
            "title": title,
            "author": author,
            "words": words,
            "chapters": chapters,
            "status": status,
            "requests": requests,
            "downloads": downloads,
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "tag": {
            "id": tag.0,
            "name": tag.1,
            "tag_type_id": tag.2,
        },
        "trending": items,
        "days": days,
    })))
}

/// GET /api/v1/trending — general trending across all fics
///
/// Returns the most-viewed fics across all tags.
pub async fn trending_general_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TrendingQuery>,
) -> Result<Json<Value>, AppError> {
    let days = params.days.unwrap_or(7).max(1).min(365);
    let limit = params.limit.unwrap_or(20).min(50).max(1);

    let rows = sqlx::query_as::<_, (String, String, String, i64, i32, String, i64, i64)>(
        r#"SELECT fi.id, fi.title, fi.author, fi.words, fi.chapters, fi.status,
                  COALESCE(rl.request_count, 0) as requests,
                  COALESCE(rl.download_count, 0) as downloads
           FROM fic_info fi
           LEFT JOIN LATERAL (
               SELECT
                   COUNT(*) as request_count,
                   SUM(CASE WHEN export_file_name IS NOT NULL THEN 1 ELSE 0 END) as download_count
               FROM request_log
               WHERE url_id = fi.id
                 AND created > NOW() - ($1 || ' days')::INTERVAL
           ) rl ON TRUE
           ORDER BY rl.request_count DESC NULLS LAST, fi.title ASC
           LIMIT $2"#,
    )
    .bind(days.to_string())
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows.into_iter().map(|(id, title, author, words, chapters, status, requests, downloads)| {
        json!({
            "url_id": id,
            "title": title,
            "author": author,
            "words": words,
            "chapters": chapters,
            "status": status,
            "requests": requests,
            "downloads": downloads,
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "trending": items,
        "days": days,
    })))
}

/// GET /api/v1/trending/tags — trending tags (most popular tags by usage)
pub async fn trending_tags_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TrendingQuery>,
) -> Result<Json<Value>, AppError> {
    let days = params.days.unwrap_or(7).max(1).min(365);
    let limit = params.limit.unwrap_or(20).min(50).max(1);

    let rows = sqlx::query_as::<_, (i32, String, i16, String, i64)>(
        r#"SELECT t.id, t.name, t.tag_type_id, tt.name as type_name,
                  COUNT(DISTINCT ft.url_id) as fic_count
           FROM tags t
           JOIN tag_types tt ON tt.id = t.tag_type_id
           JOIN fic_tags ft ON ft.tag_id = t.id
           WHERE ft.created_at > NOW() - ($1 || ' days')::INTERVAL
           GROUP BY t.id, t.name, t.tag_type_id, tt.name
           ORDER BY fic_count DESC
           LIMIT $2"#,
    )
    .bind(days.to_string())
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows.into_iter().map(|(id, name, type_id, type_name, count)| {
        json!({
            "id": id,
            "name": name,
            "tag_type_id": type_id,
            "type_name": type_name,
            "fic_count": count,
        })
    }).collect();

    Ok(Json(json!({
        "err": 0,
        "trending_tags": items,
        "days": days,
    })))
}
