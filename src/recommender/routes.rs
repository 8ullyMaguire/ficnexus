use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::FromRow;
use std::collections::HashSet;
use std::sync::Arc;

use crate::config::Config;
use crate::db::queries;
use crate::error::AppError;
use crate::recommender::engine::{
    FicInfoRow, PERSONAL_RECS_MIN_SIGNAL, PERSONAL_RECS_N, cast_vote, get_community_suggestions,
    personal_score, popularity_norm, submit_suggestion, tag_overlap,
};
use crate::recommender::{RecQuery, RecResult};
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use sqlx::PgPool;
use std::collections::HashMap;

/// Query parameters for GET /api/v0/recommendations
#[derive(Debug, Deserialize)]
pub struct RecQueryParams {
    /// URL to get recommendations for (alternative to url_id)
    pub q: Option<String>,
    /// url_id to get recommendations for (alternative to q)
    pub url_id: Option<String>,
    /// Number of recommendations to return (default 20, max 100)
    pub n: Option<i32>,
    /// Optional domain filter (e.g. "fanfiction.net")
    pub site_domain: Option<String>,
}

/// Request body for POST /api/v0/recommendations/suggest
#[derive(Debug, Deserialize)]
pub struct SuggestBody {
    /// The url_id of the fic being suggested for
    pub url_id: String,
    /// The URL of the fic being suggested as a recommendation
    pub suggested_url: String,
    /// Optional comment explaining the suggestion
    pub comment: Option<String>,
}

/// Request body for POST /api/v0/recommendations/vote
#[derive(Debug, Deserialize)]
pub struct VoteBody {
    /// The suggestion ID to vote on
    pub suggestion_id: i64,
    /// Vote value: 1 for upvote, -1 for downvote
    pub vote: i32,
}

/// Query parameters for GET /api/v0/recommendations/votes
#[derive(Debug, Deserialize)]
pub struct VotesQuery {
    /// The url_id to get suggestions/votes for
    pub url_id: String,
}

/// Build a seed_meta JSON object from scraped FicMetadata
fn build_seed_meta_from_meta(meta: &crate::scrape::FicMetadata) -> Value {
    json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "chapters": meta.chapters,
        "words": meta.words,
        "description": meta.desc,
        "status": meta.status,
        "source": meta.source,
        "created": chrono::DateTime::from_timestamp_millis(meta.published)
            .map(|d| d.to_rfc3339()).unwrap_or_default(),
        "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.to_rfc3339()).unwrap_or_default(),
        "author_url": meta.author_url,
        "author_local_id": meta.author_local_id,
        "extra_meta": meta.extra_meta,
        "raw_extended_meta": meta.raw_extended_meta,
        "source_id": meta.source_id,
        "author_id": meta.author_id,
    })
}

/// Build a seed_meta JSON object from a DB FicInfo row
fn build_seed_meta_from_fic(fic: &crate::db::models::FicInfo) -> Value {
    json!({
        "id": fic.id,
        "title": fic.title,
        "author": fic.author,
        "chapters": fic.chapters,
        "words": fic.words,
        "description": fic.description,
        "status": fic.status,
        "source": fic.source,
        "created": fic.fic_created.to_rfc3339(),
        "updated": fic.fic_updated.to_rfc3339(),
        "author_url": fic.author_url,
        "author_local_id": fic.author_local_id,
        "extra_meta": fic.extra_meta,
        "raw_extended_meta": fic.raw_extended_meta,
        "source_id": fic.source_id,
        "author_id": fic.author_id,
    })
}

/// GET /api/v0/recommendations
///
/// Returns a list of recommended fanfictions similar to the given seed work.
/// The seed can be specified either by URL (`q`) or by `url_id`.
pub async fn recommendations_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecQueryParams>,
) -> Result<Json<Value>, AppError> {
    // --- Resolve url_id and build seed_meta ---
    let (url_id, _seed_meta) = if let Some(q) = &params.q {
        // Scrape metadata to resolve the URL to a url_id (same flow as v1 export/meta handlers)
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(q)
            .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {}", q)))?;
        let meta = scraper
            .lookup(&state.http_client, q)
            .await
            .map_err(|e| AppError::ScrapeError(e.to_string()))?;
        let seed = build_seed_meta_from_meta(&meta);
        (meta.url_id, seed)
    } else if let Some(id) = &params.url_id {
        // Look up seed_meta from the database
        let seed = match queries::get_fic_info(&state.db, id).await? {
            Some(fic) => build_seed_meta_from_fic(&fic),
            None => json!({ "id": id }),
        };
        (id.clone(), seed)
    } else {
        return Ok(Json(
            json!({"err": -1, "msg": "no query or url_id provided"}),
        ));
    };

    let n = params.n.unwrap_or(20).max(1).min(100) as usize;

    // --- Check if the fic exists in the database ---
    let exists = queries::get_fic_info(&state.db, &url_id).await?.is_some();

    if !exists {
        // Enqueue for background collection so it's available for future recommendations
        state.collection_worker.enqueue_fic(&url_id).await?;
    }

    // --- Build recommendation query ---
    let rec_query = RecQuery {
        url_id: url_id.clone(),
        n,
        site_domain: params.site_domain.clone(),
    };

    // --- Compute recommendations ---
    let recommendations = state
        .recommender_engine
        .get_recommendations(&rec_query, &state.config)
        .await?;

    // --- Build response ---
    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "site_domain": params.site_domain,
        "recommendations": recommendations,
        "generated_at": chrono::Utc::now().to_rfc3339(),
    })))
}

// =============================================================================
// Personalized recommendations
// =============================================================================

/// A candidate fic fetched by the SQL candidate query.
#[derive(Debug, FromRow)]
struct PersonalCandidateRow {
    id: String,
    title: String,
    author: String,
    updated: chrono::DateTime<Utc>,
}

/// A bookmarked fic (for the `based_on` list).
#[derive(Debug, FromRow)]
#[allow(dead_code)]
struct BookmarkTitleRow {
    url_id: String,
    title: String,
}

/// Empty personalized response (anonymous user or not enough data).
fn empty_personal_response() -> Json<Value> {
    Json(json!({
        "err": 0,
        "enough_data": false,
        "recs": [],
        "based_on": [],
    }))
}

/// Extract the tag ids of a user's bookmarks (their "profile tags").
/// Reuses the exact SQL from the personal-recs handler so the legacy path
/// and the pluggable path agree byte-for-byte.
pub async fn fetch_user_top_tags(db: &PgPool, user_id: i32) -> Result<Vec<i32>, AppError> {
    Ok(sqlx::query_scalar(
        r#"SELECT ft.tag_id
           FROM bookmarks b
           JOIN fic_tags ft ON ft.url_id = b.url_id
           WHERE b.user_id = $1
           GROUP BY ft.tag_id
           ORDER BY COUNT(*) DESC
           LIMIT 20"#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?)
}

/// The full personalized-recommendation computation, shared by the legacy
/// route handler and the `legacy_cooccur` strategy.
///
/// Returns `(recs, based_on_titles, enough_data)`.
pub async fn compute_personal_recommendations(
    db: &PgPool,
    user_id: i32,
    _config: &Config,
) -> Result<(Vec<RecResult>, Vec<(String, String)>, bool), AppError> {
    // --- 1. Bookmarks + their titles (titles feed `based_on`) ---
    let bookmarks: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT b.url_id, COALESCE(fi.title, '') AS title
           FROM bookmarks b
           LEFT JOIN fic_info fi ON fi.id = b.url_id
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC
           LIMIT 100"#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await?;

    let bookmarked_ids: Vec<String> = bookmarks.iter().map(|(id, _)| id.clone()).collect();

    // --- 2. Download/export signal: any url_id the user recently fetched
    // (anonymously logged, keyed by url_id — no user linkage) ---
    let download_ids: Vec<String> = sqlx::query_scalar(
        r#"SELECT DISTINCT url_id
           FROM request_log
           WHERE url_id IS NOT NULL
             AND (export_file_name LIKE '%.epub' OR etype IN ('download', 'export'))
             AND created > now() - interval '90 days'"#,
    )
    .fetch_all(db)
    .await?;

    // --- 3. Gate: not enough personal signal ---
    if bookmarked_ids.len() + download_ids.len() < PERSONAL_RECS_MIN_SIGNAL {
        return Ok((Vec::new(), Vec::new(), false));
    }

    // --- 4. User's top tags by bookmark frequency ---
    let top_tag_ids: Vec<i32> = fetch_user_top_tags(db, user_id).await?;

    if top_tag_ids.is_empty() {
        return Ok((Vec::new(), Vec::new(), false));
    }

    // Everything the user already knows (bookmarks + downloads) is excluded
    // from the candidate pool.
    let known_ids: Vec<String> = {
        let mut set: HashSet<String> = bookmarked_ids.iter().cloned().collect();
        set.extend(download_ids.iter().cloned());
        set.into_iter().collect()
    };

    // --- 5. SQL candidate generation: fics sharing any top tag ---
    let candidates: Vec<PersonalCandidateRow> = sqlx::query_as::<_, PersonalCandidateRow>(
        r#"SELECT f.id, f.title, f.author, f.updated
           FROM fic_info f
           JOIN fic_tags ft ON ft.url_id = f.id
           WHERE ft.tag_id = ANY($1) AND f.id <> ALL($2)
           GROUP BY f.id
           ORDER BY COUNT(DISTINCT ft.tag_id) DESC
           LIMIT 200"#,
    )
    .bind(&top_tag_ids)
    .bind(&known_ids)
    .fetch_all(db)
    .await?;

    // --- 6. Score in Rust: Jaccard tag overlap + recency popularity ---
    let now = Utc::now();
    let mut scored: Vec<(f64, PersonalCandidateRow)> = Vec::with_capacity(candidates.len());
    for c in candidates {
        let fic_tags: Vec<i32> =
            sqlx::query_scalar("SELECT tag_id FROM fic_tags WHERE url_id = $1")
                .bind(&c.id)
                .fetch_all(db)
                .await?;
        let overlap = tag_overlap(&top_tag_ids, &fic_tags);
        let popularity = popularity_norm(c.updated, now);
        scored.push((personal_score(overlap, popularity), c));
    }
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(PERSONAL_RECS_N);

    let recs: Vec<RecResult> = scored
        .into_iter()
        .map(|(score, c)| RecResult {
            url_id: c.id,
            title: c.title,
            author: c.author,
            words: 0,
            chapters: 0,
            status: String::new(),
            site_domain: String::new(),
            summary: String::new(),
            score,
            community_score: 0,
            download_urls: HashMap::new(),
        })
        .collect();

    // --- 7. `based_on`: up to 3 bookmarked fic titles ---
    let based_on: Vec<(String, String)> = bookmarks
        .iter()
        .take(3)
        .filter(|(_, title)| !title.is_empty())
        .map(|(url_id, title)| (title.clone(), url_id.clone()))
        .collect();

    Ok((recs, based_on, true))
}

/// GET /api/recommendations/personal
///
/// Personalized recommendations derived from the signed-in user's bookmarks
/// (and recent download/export activity). Anonymous users get a 200 with
/// `enough_data: false` so the frontend can render a hint card without an
/// auth round-trip.
///
/// Algorithm:
/// 1. Collect the user's bookmarked url_ids (up to 100) and their top 20
///    tags by frequency (JOIN bookmarks → fic_tags).
/// 2. Add recent download/export url_ids (last 90 days) as additional
///    "known" fics — they signal interest but are excluded from recs.
/// 3. Gate: fewer than `PERSONAL_RECS_MIN_SIGNAL` (3) total signals →
///    `enough_data: false`.
/// 4. SQL candidate generation (NOT whole-table Rust scoring): fics sharing
///    ≥1 of the user's top tags, excluding all known fics, ranked by shared
///    tag count, LIMIT 200.
/// 5. Rust scoring: for each candidate, Jaccard tag overlap with the user's
///    tags + recency popularity, sort desc, take `PERSONAL_RECS_N` (20).
pub async fn personal_recommendations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Ok(empty_personal_response());
    };

    // ── Check if user opted out of personalized recs ──────────────────
    let opted_out = sqlx::query_scalar::<_, Option<String>>(
        "SELECT value FROM user_prefs WHERE user_id = $1 AND key = 'recs.personalized'",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .flatten()
    .map(|v| v == "false")
    .unwrap_or(false);

    if opted_out {
        return Ok(empty_personal_response());
    }

    // ── Pluggable mode: registry + RRF ranker (REC_ENGINE_MODE=pluggable) ──
    if state.config.rec_engine_mode.is_pluggable() {
        return pluggable_personal_handler(state, user_id).await;
    }
    let _ = state; // legacy path keeps the borrow pattern below

    let db = &state.db;
    let (recs, based_on_titles, enough) =
        compute_personal_recommendations(db, user_id, &state.config).await?;

    if !enough {
        return Ok(empty_personal_response());
    }

    let recs: Vec<Value> = recs
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or(Value::Null))
        .collect();
    let based_on: Vec<Value> = based_on_titles
        .iter()
        .map(|(title, url_id)| json!({ "title": title, "url_id": url_id }))
        .collect();

    Ok(Json(json!({
        "err": 0,
        "enough_data": true,
        "recs": recs,
        "based_on": based_on,
    })))
}

/// Pluggable-mode personalized handler: runs the enabled strategies through
/// the RRF ranker, applies the curator prior, and shapes the response like
/// the legacy one (plus `strategies` and `curator_alpha` diagnostics).
///
/// If the user has an active recipe, its `blend` weights override the
/// default `REC_STRATEGY` weights for this request.
pub async fn pluggable_personal_handler(
    state: Arc<AppState>,
    user_id: i32,
) -> Result<Json<Value>, AppError> {
    use crate::recommender::ranker;
    use crate::services::recipes::RecipeService;

    let ctx = crate::recommender::registry::build_context(
        state.db.clone(),
        std::sync::Arc::new(state.config.clone()),
        state.http_client.clone(),
        state.ollama.clone(),
    );

    // Check for an active recipe — override strategy weights if found
    let recipe = RecipeService::get_active(&state.db, user_id)
        .await
        .ok()
        .flatten();
    let recipe_weights = recipe.as_ref().and_then(|r| {
        let w = RecipeService::to_strategy_weights(r);
        if w.is_empty() { None } else { Some(w) }
    });

    // If a recipe has custom blend weights, build a per-request registry;
    // otherwise fall back to the default config-driven registry.
    let recipe_registry = recipe_weights.map(|weights| {
        crate::recommender::registry::StrategyRegistry::new(
            crate::recommender::available_strategies(&state.config),
            &weights,
        )
    });
    let registry: &crate::recommender::registry::StrategyRegistry =
        recipe_registry.as_ref().unwrap_or(&state.strategy_registry);

    let curator_alpha = recipe.as_ref().map(|r| r.curator_prior as f64);

    let (blended, diagnostics) = ranker::blend(
        &ctx,
        &registry,
        None,
        Some(user_id),
        crate::recommender::engine::PERSONAL_RECS_N,
    )
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    // Enough-data semantics: the pluggable pipeline returns an empty list
    // when no strategy contributed (fallback chain exhausted).
    if blended.is_empty() {
        return Ok(empty_personal_response());
    }

    // Resolve metadata for blended recs.
    let mut recs: Vec<Value> = Vec::with_capacity(blended.len());
    for br in blended {
        let info = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info WHERE id = $1"#,
        )
        .bind(&br.work_id)
        .fetch_optional(&state.db)
        .await?;
        if let Some(info) = info {
            recs.push(
                serde_json::to_value(RecResult {
                    url_id: info.id,
                    title: info.title,
                    author: info.author,
                    words: info.words,
                    chapters: info.chapters,
                    status: info.status,
                    site_domain: info.source,
                    summary: info.description,
                    score: br.score,
                    community_score: 0,
                    download_urls: HashMap::new(),
                })
                .unwrap_or(Value::Null),
            );
        }
    }

    // `based_on`: legacy path is bookmark-title driven; the pluggable path
    // reports the user's top bookmarked fics the same way.
    let based_on: Vec<Value> = sqlx::query_as::<_, (String, String)>(
        r#"SELECT COALESCE(fi.title, ''), b.url_id
           FROM bookmarks b
           LEFT JOIN fic_info fi ON fi.id = b.url_id
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC
           LIMIT 3"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .filter(|(t, _)| !t.is_empty())
    .map(|(t, u)| json!({ "title": t, "url_id": u }))
    .collect();

    Ok(Json(json!({
        "err": 0,
        "enough_data": true,
        "recs": recs,
        "based_on": based_on,
        "strategies": diagnostics,
        "curator_alpha": curator_alpha,
    })))
}

/// GET /api/recommendations/strategies (admin role >= 10)
///
/// Lists every registered strategy with its enabled state + weight and the
/// last training run metrics from `rec_training_runs` (for QA / eval).
pub async fn strategies_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let enabled: std::collections::HashMap<String, f64> = state
        .strategy_registry
        .specs()
        .iter()
        .map(|s| (s.name.clone(), s.weight))
        .collect();

    // Last training run per strategy.
    let last_runs: Vec<(String, Option<serde_json::Value>, Option<bool>, Option<i64>)> =
        sqlx::query_as(
            r#"SELECT DISTINCT ON (strategy) strategy, metrics, ok, duration_ms
               FROM rec_training_runs
               ORDER BY strategy, started_at DESC"#,
        )
        .fetch_all(&state.db)
        .await?;
    let run_map: std::collections::HashMap<
        String,
        (Option<serde_json::Value>, Option<bool>, Option<i64>),
    > = last_runs
        .into_iter()
        .map(|(s, m, ok, d)| (s, (m, ok, d)))
        .collect();

    let strategies: Vec<Value> = state
        .strategy_registry
        .all()
        .into_iter()
        .map(|s| {
            let name = s.name().to_string();
            let (metrics, ok, duration_ms) =
                run_map.get(&name).cloned().unwrap_or((None, None, None));
            json!({
                "name": name,
                "enabled": enabled.contains_key(&name),
                "weight": enabled.get(&name).copied().unwrap_or(0.0),
                "supports_personal": s.supports_personal(),
                "last_run": {
                    "ok": ok,
                    "duration_ms": duration_ms,
                    "metrics": metrics.unwrap_or(serde_json::json!({})),
                },
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "mode": if state.config.rec_engine_mode.is_pluggable() { "pluggable" } else { "legacy" },
        "shadow_mode": state.config.rec_shadow_mode,
        "strategies": strategies,
    })))
}

/// POST /api/recommendations/train (admin role >= 10)
///
/// Manually trigger the batch training pipeline (same as the worker tick):
/// refresh signals, run every enabled strategy's train step, record
/// rec_training_runs. Returns per-strategy run summaries.
pub async fn train_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let results = crate::recommender::worker::run_training_pipeline(
        &state.db,
        &state.config,
        &state.http_client,
        &state.ollama,
        &state.strategy_registry,
    )
    .await;
    Ok(Json(json!({ "err": 0, "runs": results })))
}

/// POST /api/v0/recommendations/suggest
///
/// Submit a user suggestion linking a fic (`url_id`) to another fic (`suggested_url`).
/// The suggested URL is resolved via the scraper to obtain its url_id.
/// Both fics must exist in the database before the suggestion is accepted.
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    // Validate the body
    if body.url_id.is_empty() || body.suggested_url.is_empty() {
        return Ok(Json(
            json!({"err": -1, "msg": "url_id and suggested_url are required"}),
        ));
    }

    // Resolve the suggested_url to a url_id via the scraper
    let scraper = state
        .scraper_registry
        .find_specific_or_fff(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {}", body.suggested_url)))?;
    let meta = scraper
        .lookup(&state.http_client, &body.suggested_url)
        .await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let suggested_url_id = meta.url_id;

    // Validate that both url_ids exist in the DB
    let seed_exists = queries::get_fic_info(&state.db, &body.url_id)
        .await?
        .is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id)
        .await?
        .is_some();

    if !seed_exists {
        // Enqueue the seed for collection and return an error
        state.collection_worker.enqueue_fic(&body.url_id).await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "seed fic not found in database — enqueued for collection"
        })));
    }

    if !suggestion_exists {
        // Enqueue the suggestion for collection
        state
            .collection_worker
            .enqueue_fic(&suggested_url_id)
            .await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "suggested fic not yet collected — enqueued for processing"
        })));
    }

    // Submit the suggestion
    let suggestion_id = submit_suggestion(
        &state.db,
        &body.url_id,
        &suggested_url_id,
        "0.0.0.0",
        body.comment.as_deref(),
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "suggestion_id": suggestion_id,
    })))
}

/// POST /api/v0/recommendations/vote
///
/// Vote (upvote = 1, downvote = -1) on an existing suggestion.
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    // Validate vote value
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(
            json!({"err": -1, "msg": "vote must be 1 (upvote) or -1 (downvote)"}),
        ));
    }

    let new_score = cast_vote(&state.db, body.suggestion_id, "0.0.0.0", body.vote as i16).await?;

    Ok(Json(json!({
        "err": 0,
        "new_score": new_score,
    })))
}

/// GET /api/v0/recommendations/votes
///
/// List all suggestions and their vote scores for a given fic.
pub async fn votes_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VotesQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id is required"})));
    }

    let suggestions = get_community_suggestions(&state.db, &params.url_id).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "suggestions": suggestions,
    })))
}

/// GET /api/v0/recommendations/entities
///
/// Recommendations for non-work entities: similar tags/fandoms, authors,
/// collections (reading lists), and users. Pure query layer over the
/// materialized signal tables — no new training data required.
pub async fn entity_recs_handler(
    State(state): State<Arc<AppState>>,
    Query(q): Query<crate::recommender::entities::EntityRecQuery>,
) -> Result<Json<Value>, AppError> {
    let payload = crate::recommender::entities::entity_recs(&state.db, q).await?;
    Ok(Json(payload))
}

/// GET /api/users/{id}/taste-cluster
///
/// Returns the user's top taste cluster and adjacent cluster ids from the
/// nightly clusters strategy (rec_user_clusters). Logged-in users only;
/// logged-out → { "cluster": null, "err": 0 }.
///
/// Response: { "cluster": n, "affinity": 0.83, "adjacent": [1, 4, 7] }
pub async fn taste_cluster_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    // Load the user's top cluster by affinity.
    let top: Option<(i32, f64)> = sqlx::query_as(
        r#"SELECT cluster_id, affinity
           FROM rec_user_clusters
           WHERE user_id = $1
           ORDER BY affinity DESC
           LIMIT 1"#,
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (cluster, affinity) = match top {
        Some((c, a)) => (c, a),
        None => return Ok(Json(json!({ "cluster": serde_json::Value::Null, "affinity": serde_json::Value::Null, "adjacent": Vec::<i32>::new(), "err": 0 }))),
    };

    // Adjacent clusters: all other distinct cluster ids for this user.
    let adjacent: Vec<i32> = sqlx::query_scalar(
        r#"SELECT DISTINCT cluster_id FROM rec_user_clusters
           WHERE user_id = $1 AND cluster_id <> $2"#,
    )
    .bind(user_id)
    .bind(cluster)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(json!({
        "err": 0,
        "cluster": cluster,
        "affinity": affinity,
        "adjacent": adjacent,
    })))
}

