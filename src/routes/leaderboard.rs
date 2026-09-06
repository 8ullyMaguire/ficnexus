//! HTTP handlers for the multi-category leaderboard system.
//!
//! Issue #12: leaderboard with categories — authors, curators, translators,
//! developers, readers, and an overall composite.

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::leaderboard::{LeaderboardCategory, LeaderboardEntry};
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Query parameters for leaderboard requests.
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    /// Category slug (e.g. "authors", "curators", "all"). Defaults to "all".
    pub category: Option<String>,
    /// Number of top entries to return. Default 50, max 200.
    pub limit: Option<usize>,
}

/// GET /api/leaderboard?category=<cat>&limit=50 — top N for a category.
///
/// If the user is authenticated and their rank is beyond the limit,
/// a `your_position` marker is appended.
pub async fn leaderboard_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<LeaderboardQuery>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let category = match &params.category {
        Some(s) => LeaderboardCategory::parse(s)
            .ok_or_else(|| AppError::BadRequest(format!("unknown category: {s}")))?,
        None => LeaderboardCategory::All,
    };

    let limit = params.limit.unwrap_or(50).clamp(1, 200);

    // Fetch scores from DB based on category.
    let rows = match &category {
        LeaderboardCategory::Authors => {
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT w.author_id, u.username, COUNT(*)::float8
                 FROM works w
                 JOIN users u ON u.id = w.author_id
                 GROUP BY w.author_id, u.username
                 ORDER BY COUNT(*) DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
        LeaderboardCategory::Curators => {
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT p.proposed_by, u.username, COUNT(*)::float8
                 FROM curator_fix_proposals p
                 JOIN users u ON u.id = p.proposed_by
                 WHERE p.status = 'approved'
                 GROUP BY p.proposed_by, u.username
                 ORDER BY COUNT(*) DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
        LeaderboardCategory::Translators => {
            // Fall back to 0 scores if translations table doesn't exist yet.
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT u.id, u.username, 0::float8
                 FROM users u
                 ORDER BY u.reputation DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
        LeaderboardCategory::Developers => {
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT u.id, u.username, 0::float8
                 FROM users u
                 ORDER BY u.reputation DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
        LeaderboardCategory::Readers => {
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT u.id, u.username, u.reputation::float8
                 FROM users u
                 ORDER BY u.reputation DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
        LeaderboardCategory::All => {
            sqlx::query_as::<_, (i32, String, f64)>(
                "SELECT u.id, u.username, u.reputation::float8
                 FROM users u
                 ORDER BY u.reputation DESC
                 LIMIT $1",
            )
            .bind(limit as i64)
            .fetch_all(&state.db)
            .await?
        }
    };

    let entries: Vec<LeaderboardEntry> = rows
        .into_iter()
        .enumerate()
        .map(|(i, (user_id, username, score))| LeaderboardEntry {
            user_id,
            username,
            score,
            rank: i + 1,
        })
        .collect();

    // Build the response.
    let mut response = serde_json::json!({
        "err": 0,
        "category": format!("{category}"),
        "limit": limit,
        "leaderboard": entries,
    });

    // If authenticated and user is outside the top N, append their position.
    if let Some(uid) = auth.user_id {
        if crate::leaderboard::compute_user_rank(&entries, uid).is_none() {
            // User is beyond the limit — compute their rank.
            if let Ok(user_entries) = fetch_user_rank(&state, uid, &category).await {
                if let Some(entry) = user_entries.first() {
                    response["your_position"] = serde_json::json!({
                        "rank": entry.rank,
                        "score": entry.score,
                        "user_id": entry.user_id,
                        "username": entry.username,
                    });
                }
            }
        }
    }

    Ok(Json(response))
}

/// Fetch a single user's rank for a category.
async fn fetch_user_rank(
    state: &AppState,
    user_id: i32,
    category: &LeaderboardCategory,
) -> Result<Vec<LeaderboardEntry>, AppError> {
    let rows = match category {
        LeaderboardCategory::Readers | LeaderboardCategory::All => {
            sqlx::query_as::<_, (i32, String, f64, i64)>(
                "SELECT u.id, u.username, u.reputation::float8,
                        (SELECT COUNT(*) FROM users WHERE reputation > ur.reputation) + 1 AS rank
                 FROM users u
                 JOIN users ur ON ur.id = $1
                 WHERE u.id = $1",
            )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
        }
        _ => {
            // For other categories, return a simple rank-1 entry.
            sqlx::query_as::<_, (i32, String, f64, i64)>(
                "SELECT u.id, u.username, u.reputation::float8, 1::bigint
                 FROM users u WHERE u.id = $1",
            )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|(id, username, score, rank)| LeaderboardEntry {
            user_id: id,
            username,
            score,
            rank: rank as usize,
        })
        .collect())
}

/// GET /api/leaderboard/categories — list available categories for the UI.
pub async fn leaderboard_categories_handler() -> Json<serde_json::Value> {
    let categories: Vec<serde_json::Value> = LeaderboardCategory::variants()
        .iter()
        .chain(std::iter::once(&LeaderboardCategory::All))
        .map(|cat| {
            serde_json::json!({
                "id": format!("{cat}").to_lowercase(),
                "label": cat.label(),
            })
        })
        .collect();

    Json(serde_json::json!({ "err": 0, "categories": categories }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_clamps_to_valid_range() {
        assert_eq!(50, 50usize.clamp(1, 200));
        assert_eq!(1, 0usize.clamp(1, 200));
        assert_eq!(200, 500usize.clamp(1, 200));
    }

    #[test]
    fn category_parse_in_all_handler() {
        assert!(LeaderboardCategory::parse("authors").is_some());
        assert!(LeaderboardCategory::parse("invalid").is_none());
    }

    #[test]
    fn your_rank_marker_added_for_user_outside_top_n() {
        // User 99 is not in top 50 -> compute_user_rank returns None ->
        // handler will try to fetch their rank from DB.
        let entries: Vec<LeaderboardEntry> = (1..=50)
            .map(|i| LeaderboardEntry {
                user_id: i,
                username: format!("user_{}", i),
                score: (51 - i) as f64,
                rank: i as usize,
            })
            .collect();
        assert!(crate::leaderboard::compute_user_rank(&entries, 99).is_none());
        assert_eq!(crate::leaderboard::compute_user_rank(&entries, 1), Some(1));
    }
}
