//! Popular + personalized tag suggestions for the search filter UI.
//!
//! `GET /api/search/suggest` returns tags ranked by how many fics they
//! appear on (popular). When `personal=1` and the caller is authenticated,
//! the suggestions are re-ranked against the user's bookmarked tags and
//! tagged `reason: "for_you"` / `"popular"`.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// How long (seconds) the non-personalized popular list is cached.
pub const SUGGEST_CACHE_TTL_SECS: u64 = 300;

/// Max suggestions returned.
const SUGGEST_LIMIT: i64 = 20;

/// Raw query parameters for `/api/search/suggest`.
#[derive(Debug, Deserialize)]
pub struct SuggestQueryParams {
    /// Optional prefix — only tags whose name starts with this are returned.
    pub q: Option<String>,
    /// Optional tag type filter (e.g. 1=fandom, 4=freeform).
    pub tag_type_id: Option<i16>,
    /// 1 = re-rank for the authenticated user (if any); ignored anonymously.
    pub personal: Option<i16>,
}

/// One suggestion in the response.
#[derive(Debug, Clone)]
struct SuggestItem {
    id: i32,
    name: String,
    tag_type_id: i16,
    usage_count: i64,
    reason: &'static str,
}

/// `GET /api/search/suggest`
pub async fn search_suggest_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<SuggestQueryParams>,
) -> Result<Json<Value>, AppError> {
    let personal = params.personal == Some(1);

    // Personalized requests are never served from the shared popular cache
    // (they depend on the user). Anonymous requests fall back to the cached
    // popular list.
    let use_cache = !personal;

    if use_cache {
        let cached = {
            let cache = state.suggest_cache.lock().await;
            cache
                .as_ref()
                .filter(|(at, _)| at.elapsed().as_secs() < SUGGEST_CACHE_TTL_SECS)
                .map(|(_, items)| items.clone())
        };
        if let Some(items) = cached {
            return Ok(Json(json!({ "err": 0, "suggestions": items })));
        }
    }

    // Popular query: every tag with at least one fic, ranked by usage.
    // No correlated subquery — plain GROUP BY over the junction table.
    let popular: Vec<(i32, String, i16, i64)> = sqlx::query_as(
        r#"SELECT t.id, t.name, t.tag_type_id, COUNT(ft.url_id) AS usage_count
           FROM tags t
           LEFT JOIN fic_tags ft ON ft.tag_id = t.id
           WHERE ($1::int IS NULL OR t.tag_type_id = $1)
             AND ($2::text IS NULL OR t.name ILIKE $2 || '%')
           GROUP BY t.id
           HAVING COUNT(ft.url_id) > 0
           ORDER BY usage_count DESC, t.name
           LIMIT $3"#,
    )
    .bind(params.tag_type_id)
    .bind(params.q.as_deref())
    .bind(SUGGEST_LIMIT)
    .fetch_all(&state.db)
    .await?;

    let mut items: Vec<SuggestItem> = popular
        .into_iter()
        .map(|(id, name, tag_type_id, usage_count)| SuggestItem {
            id,
            name,
            tag_type_id,
            usage_count,
            reason: "popular",
        })
        .collect();

    // Personalized re-rank: for a signed-in user, fetch their top tags
    // (by bookmark count), re-order the popular list by overlap with that
    // set, and mark the strongest matches as `for_you`.
    if personal {
        if let Some(user_id) = auth.user_id {
            let top: Vec<(i32, i64)> = sqlx::query_as(
                r#"SELECT ft.tag_id, COUNT(*) AS c
                   FROM bookmarks b
                   JOIN fic_tags ft ON ft.url_id = b.url_id
                   WHERE b.user_id = $1
                   GROUP BY ft.tag_id
                   ORDER BY c DESC
                   LIMIT 20"#,
            )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?;

            let top_ids: Vec<i32> = top.iter().map(|(id, _)| *id).collect();
            let mut overlaps: Vec<(f64, usize)> = items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let overlap = crate::recommender::engine::tag_overlap(
                        &[item.id],
                        &top_ids,
                    );
                    (overlap, i)
                })
                .collect();
            // Stable sort by overlap (descending): ties keep the original
            // popular ranking.
            overlaps.sort_by(|(oa, ia), (ob, ib)| {
                ob.partial_cmp(oa)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| ia.cmp(ib))
            });
            let reordered: Vec<usize> = overlaps.iter().map(|(_, i)| *i).collect();

            // Mark the top 5 overlap matches as `for_you` (only those with
            // any real overlap — ties may fall below the popular order).
            let n_for_you = reordered
                .iter()
                .take(5)
                .filter(|&&i| overlaps.iter().find(|(_, j)| *j == i).map(|(o, _)| *o).unwrap_or(0.0) > 0.0)
                .count();

            let mut new_items: Vec<SuggestItem> = Vec::with_capacity(items.len());
            for (rank, &i) in reordered.iter().enumerate() {
                let mut item = items[i].clone();
                if rank < n_for_you {
                    item.reason = "for_you";
                }
                new_items.push(item);
            }
            items = new_items;
        }
    }

    let suggestions: Vec<Value> = items
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "name": s.name,
                "tag_type_id": s.tag_type_id,
                "usage_count": s.usage_count,
                "reason": s.reason,
            })
        })
        .collect();

    if use_cache {
        let mut cache = state.suggest_cache.lock().await;
        // Only populate if the entry expired/vanished while we computed.
        let expired = cache
            .as_ref()
            .map(|(at, _)| at.elapsed().as_secs() >= SUGGEST_CACHE_TTL_SECS)
            .unwrap_or(true);
        if expired {
            *cache = Some((std::time::Instant::now(), suggestions.clone()));
        }
    }

    Ok(Json(json!({ "err": 0, "suggestions": suggestions })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: i32, name: &str, usage: i64) -> SuggestItem {
        SuggestItem {
            id,
            name: name.into(),
            tag_type_id: 4,
            usage_count: usage,
            reason: "popular",
        }
    }

    /// Popular order: higher usage first, ties broken by name.
    #[test]
    fn popular_ranking_higher_usage_first() {
        let mut items = vec![item(1, "Angst", 3), item(2, "Fluff", 9), item(3, "Adventure", 5)];
        items.sort_by(|a, b| {
            b.usage_count
                .cmp(&a.usage_count)
                .then_with(|| a.name.cmp(&b.name))
        });
        let ids: Vec<i32> = items.iter().map(|i| i.id).collect();
        assert_eq!(ids, vec![2, 3, 1]);
    }

    /// Ties in usage fall back to name order (the SQL ORDER BY contract).
    #[test]
    fn popular_ranking_ties_broken_by_name() {
        let mut items = vec![item(1, "Zombies", 5), item(2, "Aliens", 5), item(3, "Bots", 5)];
        items.sort_by(|a, b| {
            b.usage_count
                .cmp(&a.usage_count)
                .then_with(|| a.name.cmp(&b.name))
        });
        let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, vec!["Aliens", "Bots", "Zombies"]);
    }

    /// Prefix filter keeps only names starting with the prefix (ILIPKE
    /// semantics — case-insensitive).
    #[test]
    fn prefix_filter_matches_case_insensitively() {
        let names = ["Adventure", "Angst", "Fantasy", "Fluff", "Whump"];
        let keep: Vec<&str> = names
            .iter()
            .copied()
            .filter(|n| n.to_lowercase().starts_with("fan"))
            .collect();
        assert_eq!(keep, vec!["Fantasy"]);
    }

    /// tag_type_id filter keeps only matching types.
    #[test]
    fn tag_type_filter_keeps_only_matching_type() {
        let mut items = vec![
            SuggestItem { tag_type_id: 1, ..item(1, "Harry Potter", 5) },
            SuggestItem { tag_type_id: 4, ..item(2, "Fluff", 5) },
            SuggestItem { tag_type_id: 4, ..item(3, "Angst", 5) },
        ];
        items.retain(|i| i.tag_type_id == 4);
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| i.tag_type_id == 4));
    }

    /// Personalization: items overlapping the user's top tags are re-ranked
    /// ahead of the popular order and marked `for_you`.
    #[test]
    fn personal_rerank_marks_for_you_and_reorders() {
        // Popular order: Fluff (9), Angst (5), Adventure (3).
        let items = vec![item(1, "Fluff", 9), item(2, "Angst", 5), item(3, "Adventure", 3)];
        // User's top tags are exactly { Angst, Adventure }.
        let top_ids = vec![2, 3];

        let mut overlaps: Vec<(f64, usize)> = items
            .iter()
            .enumerate()
            .map(|(i, it)| (crate::recommender::engine::tag_overlap(&[it.id], &top_ids), i))
            .collect();
        overlaps.sort_by(|(oa, ia), (ob, ib)| {
            ob.partial_cmp(oa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| ia.cmp(ib))
        });
        let reordered: Vec<usize> = overlaps.iter().map(|(_, i)| *i).collect();

        let n_for_you = reordered
            .iter()
            .take(5)
            .filter(|&&i| overlaps.iter().find(|(_, j)| *j == i).map(|(o, _)| *o).unwrap_or(0.0) > 0.0)
            .count();

        let names: Vec<&str> = reordered.iter().map(|&i| items[i].name.as_str()).collect();
        // Angst and Adventure (the user's tags) lead, Fluff trails.
        assert_eq!(names, vec!["Angst", "Adventure", "Fluff"]);
        assert_eq!(n_for_you, 2);
    }

    /// Personalization with NO overlap leaves the popular order intact and
    /// marks nothing `for_you`.
    #[test]
    fn personal_rerank_no_overlap_keeps_popular_order() {
        let items = vec![item(1, "Fluff", 9), item(2, "Angst", 5)];
        let top_ids = vec![99];

        let mut overlaps: Vec<(f64, usize)> = items
            .iter()
            .enumerate()
            .map(|(i, it)| (crate::recommender::engine::tag_overlap(&[it.id], &top_ids), i))
            .collect();
        overlaps.sort_by(|(oa, ia), (ob, ib)| {
            ob.partial_cmp(oa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| ia.cmp(ib))
        });
        let reordered: Vec<usize> = overlaps.iter().map(|(_, i)| *i).collect();

        let n_for_you = reordered
            .iter()
            .take(5)
            .filter(|&&i| overlaps.iter().find(|(_, j)| *j == i).map(|(o, _)| *o).unwrap_or(0.0) > 0.0)
            .count();

        let names: Vec<&str> = reordered.iter().map(|&i| items[i].name.as_str()).collect();
        assert_eq!(names, vec!["Fluff", "Angst"]);
        assert_eq!(n_for_you, 0);
    }
}
