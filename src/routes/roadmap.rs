//! Roadmap Consensus Engine — semantic clustering + MaxDiff/Elo voting.
//!
//! Users submit feature ideas (`POST /api/roadmap/suggest`); the backend
//! embeds the text via Ollama (nomic-embed-text) and either joins the nearest
//! existing cluster (cosine distance < threshold) or spawns a new one.
//! The Arena (`GET /api/roadmap/arena`) serves 4 clusters; a MaxDiff vote
//! (`POST /api/roadmap/vote`, best+worst) is translated into 6 virtual 1v1
//! Elo matches to produce a global consensus ranking.
//!
//! Zero-PII: anonymous voters are keyed by client_id UUID, never IPs.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

// Consensus math from the extracted crate.
use fichub_consensus::{ConsensusConfig, VoterRef, maxdiff_elo_updates};
use std::collections::HashMap;

/// Resolve the effective user id for ownership/anti-spam (anonymous clients
/// have `user_id: None` — they're keyed by client_id only). Returns `None`
/// for anonymous so DB inserts use NULL (nullable FK column), avoiding a
/// FK violation for user_id = 0.
fn user_id_or(user: &AuthUser) -> Option<i32> {
    user.user_id
}

/// Clustering threshold: cosine distance below this means "same idea".
/// nomic-embed-text is well-suited to ~0.15-0.25 thresholds for short texts;
/// 0.22 is a reasonable middle ground (strict enough to avoid "dark mode" and
/// "change font to black" merging, loose enough to group paraphrases).
pub const CLUSTER_DISTANCE_THRESHOLD: f64 = 0.22;
/// Max submissions per day per identity (anti-spam).
pub const MAX_SUGGESTIONS_PER_DAY: i64 = 3;

// ─── Handlers ──────────────────────────────────────────────────────────────

// FichHub config: allow voting on all stages (not just ideas), require level >= 2 to vote.
// Built at runtime since ConsensusConfig::default_stages() is not const.
fn fichub_config() -> ConsensusConfig {
    ConsensusConfig::fichub()
}

// re-exported for future use when wiring Consensus engine directly.
#[allow(dead_code)]
fn voter_ref(user: &AuthUser) -> VoterRef {
    VoterRef {
        id: user.user_id.map(|u| u.to_string()),
        trust_level: user.level as u8,
    }
}

#[derive(Debug, Deserialize)]
pub struct SuggestRequest {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct FeaturesQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveFeatureRequest {
    pub status: String,
    pub category: Option<String>,
}

const VALID_STATUSES: &[&str] = &[
    "idea",
    "long_term",
    "medium_term",
    "up_next",
    "in_progress",
    "finished",
    "shipped",
    "rejected",
];

fn is_valid_status(s: &str) -> bool {
    VALID_STATUSES.contains(&s)
}

/// POST /api/roadmap/suggest — embed the text, cluster it, store it.
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<SuggestRequest>,
) -> Result<Json<Value>, AppError> {
    let text = req.text.trim().to_string();
    if text.is_empty() {
        return Err(AppError::BadRequest(
            "suggestion text is required".to_string(),
        ));
    }
    if text.len() > 1000 {
        return Err(AppError::BadRequest(
            "suggestion too long (max 1000 chars)".to_string(),
        ));
    }

    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Anti-spam: max 3 suggestions/day per identity.
    let since = chrono::Utc::now() - chrono::Duration::days(1);
    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM feature_suggestions WHERE (user_id = $1 OR client_id = $2) AND created_at >= $3",
    )
    .bind(user_id_or(&user))
    .bind(&client_id)
    .bind(since)
    .fetch_one(&state.db)
    .await?;
    if recent >= MAX_SUGGESTIONS_PER_DAY {
        return Err(AppError::BadRequest(
            "too many suggestions today (max 3)".to_string(),
        ));
    }

    // Embed via Ollama (best-effort — on failure store unclustered).
    let embedding: Option<Vec<f32>> = state
        .ollama
        .embed(&text)
        .await
        .map_err(|e| tracing::warn!("ollama embed failed: {e}"))
        .ok();

    let cluster_id: i32 = if let Some(emb) = embedding {
        let emb_sql = format!(
            "[{}]",
            emb.iter()
                .map(|f| format!("{f:.6}"))
                .collect::<Vec<_>>()
                .join(",")
        );
        // Nearest existing open cluster (now using 'idea' status post-Kanban)
        let nearest: Option<(i32, f64)> = sqlx::query_as(
            "SELECT id, (embedding <=> $1::vector) AS dist FROM feature_clusters WHERE status = 'idea' ORDER BY embedding <=> $1::vector LIMIT 1",
        )
        .bind(&emb_sql)
        .fetch_optional(&state.db)
        .await?;

        match nearest {
            Some((cid, dist)) if dist < CLUSTER_DISTANCE_THRESHOLD => {
                // Join the existing cluster (representative text unchanged)
                sqlx::query(
                    "UPDATE feature_clusters SET matches_played = matches_played WHERE id = $1",
                )
                .bind(cid)
                .execute(&state.db)
                .await?;
                cid
            }
            _ => {
                // Spawn a new cluster
                sqlx::query(
                    "INSERT INTO feature_clusters (representative_text, embedding, status) VALUES ($1, $2::vector, 'idea') RETURNING id",
                )
                .bind(&text)
                .bind(&emb_sql)
                .fetch_one(&state.db)
                .await
                .map(|row: sqlx::postgres::PgRow| row.get::<i32, _>(0))?
            }
        }
    } else {
        // Ollama down — no cluster yet; the raw suggestion is still saved.
        0
    };

    sqlx::query(
        "INSERT INTO feature_suggestions (user_id, client_id, raw_text, cluster_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id_or(&user))
    .bind(&client_id)
    .bind(&text)
    .bind(if cluster_id > 0 { Some(cluster_id) } else { None })
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "cluster_id": if cluster_id > 0 { Some(cluster_id) } else { None },
        "clustered": cluster_id > 0,
    })))
}

/// GET /api/roadmap/arena — 4 clusters, weighted inversely by matches_played
/// so fresh clusters get exposed (and their baseline Elo established).
pub async fn arena_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    // Weighted random-ish: pick from the least-played quartile when available,
    // otherwise any open cluster. (True weighted random in SQL is awkward; a
    // "least played first" draw is a fine approximation and cheap.)
    // Arena-eligible stages: FicNexus config allows voting on all non-rejected stages.
    let config = fichub_config();
    let eligible = config.arena_eligible_stage_ids();
    // Build the IN-list as a const string — eligible stages are bounded (max 8 strings).
    // Since FichHub allows all stages (allow_voting_on_non_idea_stages=true), this is all stages
    // except 'rejected' (frozen unconditionally in default_stages).
    let status_filter: String = eligible
        .iter()
        .map(|s| format!("'{}'", s))
        .collect::<Vec<_>>()
        .join(",");

    // Use query with leaked SQL (bounded, small, no user input).
    let sql: &'static str = Box::leak(
        format!(
            r#"
        SELECT id, representative_text, matches_played, elo_rating::float8
        FROM feature_clusters
        WHERE status IN ({status_filter})
        ORDER BY matches_played ASC, random()
        LIMIT 4
        "#
        )
        .into_boxed_str(),
    );
    let rows: Vec<(i32, String, i32, f64)> = sqlx::query_as(sql).fetch_all(&state.db).await?;

    if rows.is_empty() {
        return Ok(Json(
            json!({ "err": 0, "clusters": [], "message": "No features yet — be the first to suggest one!" }),
        ));
    }

    // Exclude sets the user already voted on (per-user uniqueness).
    let clusters = rows
        .into_iter()
        .map(|(id, text, played, elo)| {
            json!({
                "id": id,
                "text": text,
                "matches_played": played,
                "elo_rating": elo,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(
        json!({ "err": 0, "clusters": clusters, "user_id": user_id_or(&user).unwrap_or(0) }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub cluster_ids: Vec<i32>,
    pub best_cluster_id: i32,
    pub worst_cluster_id: i32,
}

/// POST /api/roadmap/vote — record a MaxDiff vote and run virtual Elo.
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    headers: HeaderMap,
    Json(req): Json<VoteRequest>,
) -> Result<Json<Value>, AppError> {
    if req.cluster_ids.len() != 4 {
        return Err(AppError::BadRequest(
            "arena must present exactly 4 clusters".to_string(),
        ));
    }
    if !req.cluster_ids.contains(&req.best_cluster_id)
        || !req.cluster_ids.contains(&req.worst_cluster_id)
    {
        return Err(AppError::BadRequest(
            "best/worst must be among the presented clusters".to_string(),
        ));
    }
    if req.best_cluster_id == req.worst_cluster_id {
        return Err(AppError::BadRequest(
            "best and worst must differ".to_string(),
        ));
    }

    // Trust gate: FichHub preset requires level >= 2 (min_trust_level_for_voting).
    if user.level < 2 {
        return Err(AppError::Forbidden("level 2+ required to vote".to_string()));
    }

    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut sorted_ids = req.cluster_ids.clone();
    sorted_ids.sort_unstable();

    // Uniqueness: no double-vote on the same set. Logged-in users are keyed
    // by user_id; anonymous clients (user_id None) by client_id (NULL user_id
    // rows don't collide on the UNIQUE (user_id, cluster_ids) index, so we
    // enforce per-client uniqueness here).
    let uid = user_id_or(&user);
    let exists: bool = if uid.is_some() {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE user_id = $1 AND cluster_ids = $2)",
        )
        .bind(uid)
        .bind(&sorted_ids)
        .fetch_one(&state.db)
        .await?
    } else {
        match &client_id {
            Some(cid) => {
                sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM arena_votes WHERE client_id = $1 AND cluster_ids = $2 AND user_id IS NULL)",
                )
                .bind(cid)
                .bind(&sorted_ids)
                .fetch_one(&state.db)
                .await?
            }
            None => false,
        }
    };
    if exists {
        return Err(AppError::BadRequest(
            "you already voted on this set".to_string(),
        ));
    }

    // Load current ratings for the 4 clusters (unknown -> 1500).
    let mut ratings_map: HashMap<i32, f64> = HashMap::new();
    for id in &req.cluster_ids {
        let rating: Option<f64> =
            sqlx::query_scalar("SELECT elo_rating::float8 FROM feature_clusters WHERE id = $1")
                .bind(id)
                .fetch_optional(&state.db)
                .await?;
        ratings_map.insert(*id, rating.unwrap_or(1500.0));
    }

    let new_ratings = maxdiff_elo_updates(
        &req.cluster_ids,
        &ratings_map,
        req.best_cluster_id,
        req.worst_cluster_id,
        32.0,
    );

    // Persist the vote + apply Elo + bump counters atomically.
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO arena_votes (user_id, client_id, cluster_ids, best_cluster_id, worst_cluster_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id_or(&user))
    .bind(&client_id)
    .bind(&sorted_ids)
    .bind(req.best_cluster_id)
    .bind(req.worst_cluster_id)
    .execute(&mut *tx)
    .await?;

    for (id, new_rating) in &new_ratings {
        let delta = new_rating - 1500.0; // only meaningful relative to baseline
        let _ = delta;
        sqlx::query(
            r#"
            UPDATE feature_clusters SET
                elo_rating = $1,
                matches_played = matches_played + 1,
                times_picked_best = times_picked_best + CASE WHEN $2 THEN 1 ELSE 0 END,
                times_picked_worst = times_picked_worst + CASE WHEN $3 THEN 1 ELSE 0 END
            WHERE id = $4
            "#,
        )
        .bind(new_rating)
        .bind(*id == req.best_cluster_id)
        .bind(*id == req.worst_cluster_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(Json(json!({
        "err": 0,
        "applied": new_ratings.iter().map(|(id, r)| json!({ "cluster_id": id, "new_elo": r })).collect::<Vec<_>>(),
    })))
}

/// GET /api/roadmap/consensus — public (no auth) leaderboard + controversy.
/// Same shape as the admin endpoint: `leaderboard` ranked by Elo DESC with
/// vote counters + suggestion count + status, and `controversy` (volume vs
/// best+worst picks) for the public Roadmap page's Consensus section.
/// Zero-PII: cluster texts + aggregates only (no user/client mapping).
pub async fn consensus_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    // Leaderboard: ranked by Elo, with vote counters + suggestion count.
    let leaderboard = sqlx::query_as::<_, (i32, String, f64, i32, i32, i32, i64, String)>(
        r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, COUNT(s.id)::bigint AS suggestions,
               c.status
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        GROUP BY c.id
        -- c.id breaks ties: SQL does not define the order of rows with equal
        -- elo_rating, so without this the admin leaderboard can reorder
        -- between equally-ranked items on identical queries.
        ORDER BY c.elo_rating DESC, c.id ASC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    // Controversy: volume (matches) vs controversy (best+worst picks).
    let controversy = sqlx::query_as::<_, (i32, String, i32, i32, i32, i32, f64)>(
        r#"
        SELECT id, representative_text, matches_played,
               times_picked_best, times_picked_worst,
               (times_picked_best + times_picked_worst) AS controversy,
               elo_rating::float8
        FROM feature_clusters
        WHERE status != 'rejected'
        ORDER BY matches_played DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "leaderboard": leaderboard.into_iter().map(|(id, text, elo, played, best, worst, sugg, status)| json!({
            "id": id, "text": text, "elo_rating": elo, "matches_played": played,
            "times_picked_best": best, "times_picked_worst": worst, "suggestions": sugg, "status": status,
        })).collect::<Vec<_>>(),
        "controversy": controversy.into_iter().map(|(id, text, played, best, worst, contr, elo)| json!({
            "id": id, "text": text, "matches_played": played,
            "times_picked_best": best, "times_picked_worst": worst,
            "controversy": contr, "elo_rating": elo,
        })).collect::<Vec<_>>(),
    })))
}

/// GET /api/roadmap/features — list clusters with status/category filters, sorted by Elo DESC.
/// Query: ?status=&category=&q=
pub async fn features_list_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<FeaturesQuery>,
) -> Result<Json<Value>, AppError> {
    let status = params.status.unwrap_or_default();
    let category = params.category.unwrap_or_default();
    let q = params.q.unwrap_or_default();

    type Row = (i32, String, f64, i32, i32, i32, String, String, i64);

    const SQL_ALL: &str = r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, c.status, c.category,
               COUNT(s.id)::bigint AS suggestions
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        GROUP BY c.id
        ORDER BY c.elo_rating DESC
        LIMIT 200
    "#;

    const SQL_BY_STATUS: &str = r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, c.status, c.category,
               COUNT(s.id)::bigint AS suggestions
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        WHERE c.status = $1
        GROUP BY c.id
        ORDER BY c.elo_rating DESC
        LIMIT 200
    "#;

    const SQL_BY_CATEGORY: &str = r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, c.status, c.category,
               COUNT(s.id)::bigint AS suggestions
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        WHERE c.category = $1
        GROUP BY c.id
        ORDER BY c.elo_rating DESC
        LIMIT 200
    "#;

    const SQL_BY_Q: &str = r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, c.status, c.category,
               COUNT(s.id)::bigint AS suggestions
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        WHERE c.representative_text ILIKE $1
        GROUP BY c.id
        ORDER BY c.elo_rating DESC
        LIMIT 200
    "#;

    let rows = if !status.is_empty() {
        sqlx::query_as::<_, Row>(SQL_BY_STATUS)
            .bind(&status)
            .fetch_all(&state.db)
            .await?
    } else if !category.is_empty() {
        sqlx::query_as::<_, Row>(SQL_BY_CATEGORY)
            .bind(&category)
            .fetch_all(&state.db)
            .await?
    } else if !q.is_empty() {
        let pattern = format!("%{}%", q);
        sqlx::query_as::<_, Row>(SQL_BY_Q)
            .bind(&pattern)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, Row>(SQL_ALL)
            .fetch_all(&state.db)
            .await?
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, text, elo, played, best, worst, status, category, sugg)| {
                json!({
                    "id": id, "text": text, "elo_rating": elo, "matches_played": played,
                    "times_picked_best": best, "times_picked_worst": worst,
                    "status": status, "category": category, "suggestions": sugg,
                })
            },
        )
        .collect();

    Ok(Json(json!({
        "err": 0,
        "features": items,
    })))
}

/// PATCH /api/roadmap/features/:id — curator move (status/category).
/// Guard: level >= 50 (curator).
pub async fn feature_move_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(req): Json<MoveFeatureRequest>,
) -> Result<Json<Value>, AppError> {
    // Require curator level (>= 50).
    if auth.level < 50 {
        return Err(AppError::Forbidden("curator trust level required".to_string()));
    }

    if !is_valid_status(&req.status) {
        return Err(AppError::BadRequest(format!(
            "invalid status: '{}'",
            req.status
        )));
    }

    let category = req.category.unwrap_or_else(|| "general".to_string());

    sqlx::query(r#"UPDATE feature_clusters SET status = $1, category = $2 WHERE id = $3"#)
        .bind(&req.status)
        .bind(&category)
        .bind(id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "id": id,
        "status": req.status,
        "category": category,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fichub_consensus::{elo_update, expected_score};

    #[test]
    fn expected_score_midpoint_is_half() {
        assert!((expected_score(1500.0, 1500.0) - 0.5).abs() < 1e-9);
        assert!(expected_score(1600.0, 1400.0) > 0.75);
    }

    #[test]
    fn elo_update_win_raises_loser_lowers() {
        let winner_new = elo_update(1500.0, 1500.0, 1.0, 32.0);
        let loser_new = elo_update(1500.0, 1500.0, 0.0, 32.0);
        assert!(winner_new > 1500.0);
        assert!(loser_new < 1500.0);
        // Symmetry: what winner gains, loser loses (equal ratings, K=32)
        assert!((winner_new - 1500.0 - (1500.0 - loser_new)).abs() < 1e-9);
    }

    #[test]
    fn maxdiff_best_soars_worst_tanks_neutrals_shift_little() {
        let ids = vec![1, 2, 3, 4];
        let ratings: HashMap<i32, f64> =
            [(1, 1500.0), (2, 1500.0), (3, 1500.0), (4, 1500.0)].into();
        let out = maxdiff_elo_updates(&ids, &ratings, 1, 4, 32.0);

        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        let r4 = out.iter().find(|(i, _)| *i == 4).unwrap().1;
        let r2 = out.iter().find(|(i, _)| *i == 2).unwrap().1;
        let r3 = out.iter().find(|(i, _)| *i == 3).unwrap().1;

        assert!(r1 > 1540.0, "best gains big: {r1}");
        assert!(r4 < 1460.0, "worst loses big: {r4}");
        assert!(r2 > r4 && r2 < r1, "neutral 2 between: {r2}");
        assert!(r3 > r4 && r3 < r1, "neutral 3 between: {r3}");
    }

    #[test]
    fn maxdiff_rating_gap_reduces_transfer() {
        // A 1800-rated best vs a 1200-rated worst: the win transfers less Elo
        // than an even match would.
        let ids = vec![1, 2, 3, 4];
        let ratings: HashMap<i32, f64> =
            [(1, 1800.0), (2, 1500.0), (3, 1500.0), (4, 1200.0)].into();
        let out = maxdiff_elo_updates(&ids, &ratings, 1, 4, 32.0);
        let r1 = out.iter().find(|(i, _)| *i == 1).unwrap().1;
        assert!(r1 > 1800.0, "still gains");
        assert!(r1 - 1800.0 < 32.0 * 3.0, "gain bounded by K*matches");
    }
}

// ─── Changelog ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChangelogQuery {
    pub kind: Option<String>, // new | improved | fixed
    pub feature_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ChangelogCreateRequest {
    pub feature_id: Option<i32>,
    pub title: String,
    pub body: String,
    pub kind: String, // new | improved | fixed
}

const VALID_CHANGELOG_KINDS: &[&str] = &["new", "improved", "fixed"];

fn is_valid_changelog_kind(k: &str) -> bool {
    VALID_CHANGELOG_KINDS.contains(&k)
}

/// GET /api/roadmap/changelog — public paginated list, filter by kind + feature.
pub async fn changelog_list_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<ChangelogQuery>,
) -> Result<Json<Value>, AppError> {
    let kind = params.kind.unwrap_or_default();
    let feature_id = params.feature_id;
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    type Row = (
        i64,
        Option<i32>,
        String,
        String,
        String,
        i32,
        chrono::DateTime<chrono::Utc>,
    );

    const SQL_ALL: &str = r#"
        SELECT id, feature_id, title, body, kind, author_id, published_at
        FROM roadmap_changelog
        WHERE is_published = TRUE
        ORDER BY published_at DESC
        LIMIT $1 OFFSET $2
    "#;

    const SQL_BY_KIND: &str = r#"
        SELECT id, feature_id, title, body, kind, author_id, published_at
        FROM roadmap_changelog
        WHERE is_published = TRUE AND kind = $1
        ORDER BY published_at DESC
        LIMIT $2 OFFSET $3
    "#;

    const SQL_BY_FEATURE: &str = r#"
        SELECT id, feature_id, title, body, kind, author_id, published_at
        FROM roadmap_changelog
        WHERE is_published = TRUE AND feature_id = $1
        ORDER BY published_at DESC
        LIMIT $2 OFFSET $3
    "#;

    const SQL_BY_BOTH: &str = r#"
        SELECT id, feature_id, title, body, kind, author_id, published_at
        FROM roadmap_changelog
        WHERE is_published = TRUE AND kind = $1 AND feature_id = $2
        ORDER BY published_at DESC
        LIMIT $3 OFFSET $4
    "#;

    let rows: Vec<Row> = if !kind.is_empty() {
        if !is_valid_changelog_kind(&kind) {
            return Err(AppError::BadRequest(format!("invalid kind: '{}'", kind)));
        }
        if feature_id.is_some() {
            sqlx::query_as::<_, Row>(SQL_BY_BOTH)
                .bind(&kind)
                .bind(feature_id.unwrap())
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.db)
                .await?
        } else {
            sqlx::query_as::<_, Row>(SQL_BY_KIND)
                .bind(&kind)
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.db)
                .await?
        }
    } else if feature_id.is_some() {
        sqlx::query_as::<_, Row>(SQL_BY_FEATURE)
            .bind(feature_id.unwrap())
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, Row>(SQL_ALL)
            .bind(limit)
            .bind(offset)
            .fetch_all(&state.db)
            .await?
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, feature_id, title, body, kind, author_id, published_at)| {
                json!({
                    "id": id, "feature_id": feature_id, "title": title, "body": body,
                    "kind": kind, "author_id": author_id,
                    "published_at": published_at.to_rfc3339(),
                })
            },
        )
        .collect();

    Ok(Json(json!({
        "err": 0,
        "changelog": items,
    })))
}

/// POST /api/roadmap/changelog — curator create entry (level >= 50).
pub async fn changelog_create_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<ChangelogCreateRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.level < 50 {
        return Err(AppError::Forbidden("curator trust level required".to_string()));
    }
    if !is_valid_changelog_kind(&req.kind) {
        return Err(AppError::BadRequest(format!(
            "invalid kind: '{}'",
            req.kind
        )));
    }

    let author_id = auth
        .user_id
        .ok_or_else(|| AppError::Forbidden("Not authenticated".to_string()))?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO roadmap_changelog (feature_id, title, body, kind, author_id, is_published)
        VALUES ($1, $2, $3, $4, $5, TRUE)
        RETURNING id
        "#,
    )
    .bind(req.feature_id)
    .bind(&req.title)
    .bind(&req.body)
    .bind(&req.kind)
    .bind(author_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "id": id,
    })))
}
