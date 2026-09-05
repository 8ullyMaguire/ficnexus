use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::trending::Timeframe;

#[derive(Debug, Deserialize)]
pub struct TrendingQuery {
    /// Backward-compat: integer day count (e.g. `?days=7`).
    pub days: Option<i32>,
    /// Flexible timeframe string (e.g. `?timeframe=2w`, `?timeframe=3m`,
    /// `?timeframe=1y`). Overrides `days` when both are set.
    pub timeframe: Option<String>,
    pub limit: Option<i64>,
}

/// Resolve a [`TrendingQuery`] into a concrete [`Timeframe`].
///
/// `?timeframe=` takes precedence; falls back to `?days=`; defaults to
/// [`Timeframe::DEFAULT`] (7 days). Returns just the `days` field so
/// handlers stay focused on the SQL.
fn resolve_timeframe(q: &TrendingQuery) -> i32 {
    if let Some(tf) = &q.timeframe {
        if let Some(parsed) = Timeframe::parse(tf) {
            return parsed.days;
        }
    }
    if let Some(d) = q.days {
        return Timeframe::from_days(d).days;
    }
    Timeframe::DEFAULT.days
}

/// GET /api/v1/trending/tag/{tag_type_id}/{tag_name}
///
/// Returns fics with a specific tag that are trending (most viewed / downloaded).
pub async fn trending_by_tag_handler(
    State(state): State<Arc<AppState>>,
    Path((tag_type_id, tag_name)): Path<(i16, String)>,
    Query(params): Query<TrendingQuery>,
) -> Result<Json<Value>, AppError> {
    let days = resolve_timeframe(&params);
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
                Some(canonical_id) => sqlx::query_as::<_, (i32, String, i16)>(
                    "SELECT id, name, tag_type_id FROM tags WHERE id = $1",
                )
                .bind(canonical_id)
                .fetch_optional(&state.db)
                .await?
                .ok_or_else(|| AppError::NotFound("Tag not found".into()))?,
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

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, title, author, words, chapters, status, requests, downloads)| {
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
            },
        )
        .collect();

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
    let days = resolve_timeframe(&params);
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

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, title, author, words, chapters, status, requests, downloads)| {
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
            },
        )
        .collect();

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
    let days = resolve_timeframe(&params);
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

    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, type_id, type_name, count)| {
            json!({
                "id": id,
                "name": name,
                "tag_type_id": type_id,
                "type_name": type_name,
                "fic_count": count,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "trending_tags": items,
        "days": days,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(days: Option<i32>, tf: Option<&str>) -> TrendingQuery {
        TrendingQuery {
            days,
            timeframe: tf.map(String::from),
            limit: None,
        }
    }

    #[test]
    fn resolve_timeframe_prefers_timeframe_param() {
        assert_eq!(resolve_timeframe(&q(Some(7), Some("2w"))), 14);
        assert_eq!(resolve_timeframe(&q(Some(7), Some("3m"))), 90);
    }

    #[test]
    fn resolve_timeframe_falls_back_to_days() {
        assert_eq!(resolve_timeframe(&q(Some(30), None)), 30);
        // Out-of-range days value gets clamped to MAX_DAYS
        let huge = 5 * 366 + 1000;
        assert_eq!(resolve_timeframe(&q(Some(huge), None)), 5 * 366);
    }

    #[test]
    fn resolve_timeframe_uses_default() {
        assert_eq!(resolve_timeframe(&q(None, None)), 7);
    }

    #[test]
    fn resolve_timeframe_ignores_garbage_timeframe() {
        // Bad timeframe string → fall through to days
        assert_eq!(resolve_timeframe(&q(Some(5), Some("garbage"))), 5);
    }
}
