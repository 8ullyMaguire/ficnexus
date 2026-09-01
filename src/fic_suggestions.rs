use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use crate::error::AppError;
use crate::limiter::{client_ip_from_headers, Tier, TieredRateLimitResult};
use crate::server::AppState;

/// Query params for GET /api/fic-suggestions?url_id=...
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub url_id: String,
}

/// Body for POST /api/fic-suggestions — either an in-DB `suggested_url_id` or
/// a `url` that is resolved via the scraper registry (rejected when the host
/// is unsupported; enqueued for collection when the fic isn't collected yet).
#[derive(Debug, Deserialize)]
pub struct CreateBody {
    pub url_id: String,
    pub suggested_url_id: Option<String>,
    pub url: Option<String>,
    pub comment: Option<String>,
}

/// Body for POST /api/fic-suggestions/{id}/vote — 1 up, -1 down, 0 retract.
#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub vote: i32,
}

/// Resolve the caller's real IP from X-Forwarded-For (nginx sets and
/// overwrites it, so the first hop is the client) with the peer address as
/// fallback — the same convention as src/routes/export.rs.
fn caller_ip(headers: &HeaderMap, remote: SocketAddr) -> IpAddr {
    client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        remote.ip(),
    )
}

/// Enforce the Default-tier write limiter (moderate per-IP bucket). Fails
/// open on limiter errors so a Redis hiccup never blocks community features.
async fn enforce_write_rate_limit(state: &Arc<AppState>, headers: &HeaderMap, remote: SocketAddr) -> Result<(), AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    match state
        .rate_limiter
        .check(caller_ip(headers, remote), client_id.as_deref(), Tier::Default)
        .await
    {
        TieredRateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        TieredRateLimitResult::Allowed => Ok(()),
    }
}

/// GET /api/fic-suggestions?url_id=...
///
/// List active suggestions for a fic, ordered by vote score desc (ties: most
/// recent first). Each item carries the suggested fic's title/author (JOIN
/// fic_info), the net score, total vote count, and the caller's own vote
/// (`my_vote`: 1/-1/0) resolved from their user_id (authenticated) or IP
/// (anonymous).
pub async fn list_suggestions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListQuery>,
    auth: crate::routes::auth::AuthUser,
    headers: HeaderMap,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({ "err": -1, "msg": "url_id is required" })));
    }

    let ip = caller_ip(&headers, remote);
    let user_id = auth.user_id;

    // Vote aggregate + titles in one pass; the LEFT JOIN keeps suggestions
    // with zero votes. Only rows with status='active' are listed.
    let rows: Vec<(i64, String, String, String, Option<String>, Option<i32>, Option<i64>, Option<i16>, Option<i32>, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as(
            r#"
            SELECT s.id,
                   s.suggested_url_id,
                   COALESCE(fi.title, '')      AS suggested_title,
                   COALESCE(fi.author, '')     AS suggested_author,
                   s.comment,
                   COALESCE(v.net, 0)::INT     AS score,
                   COALESCE(v.cnt, 0)::BIGINT  AS total_votes,
                   my.vote                     AS my_vote,
                   s.user_id                   AS user_id,
                   s.created
              FROM recommendation_suggestions s
              JOIN fic_info fi ON fi.id = s.suggested_url_id
              LEFT JOIN (
                  SELECT suggestion_id, SUM(vote)::INT AS net, COUNT(*)::BIGINT AS cnt
                    FROM recommendation_votes
                   GROUP BY suggestion_id
              ) v ON v.suggestion_id = s.id
              LEFT JOIN LATERAL (
                  SELECT vote
                    FROM recommendation_votes rv
                   WHERE rv.suggestion_id = s.id
                     AND (
                         ($1::int IS NOT NULL AND rv.user_id = $1)
                         OR
                         ($1::int IS NULL AND rv.voter_ip = $2::inet)
                     )
                   LIMIT 1
              ) my ON TRUE
             WHERE s.url_id = $3 AND s.status = 'active'
             ORDER BY score DESC, s.created DESC, s.id DESC
            "#,
        )
        .bind(user_id)
        .bind(ip.to_string())
        .bind(&params.url_id)
        .fetch_all(&state.db)
        .await?;

    let suggestions: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, suggested_url_id, suggested_title, suggested_author, comment, score, total_votes, my_vote, user_id, created)| {
                json!({
                    "id": id,
                    "url_id": params.url_id,
                    "suggested_url_id": suggested_url_id,
                    "suggested_title": suggested_title,
                    "suggested_author": suggested_author,
                    "comment": comment,
                    "score": score.unwrap_or(0),
                    "total_votes": total_votes.unwrap_or(0),
                    "my_vote": my_vote.unwrap_or(0),
                    "user_id": user_id,
                    "created": created.map(|c| c.to_rfc3339()),
                })
            },
        )
        .collect();

    Ok(Json(json!({ "err": 0, "url_id": params.url_id, "suggestions": suggestions })))
}

/// POST /api/fic-suggestions
///
/// Create a per-fic suggestion. Auth required (HTTP 400 err 401 when no valid
/// token is sent — the project's 400-as-401 convention).
///
/// * `suggested_url_id` given → validated against fic_info.
/// * `url` given → resolved via the scraper registry; unsupported hosts get a
///   friendly BadRequest (err -5); a resolvable-but-uncollected fic is
///   enqueued for background collection and returns err -5
///   ('will be added when collected').
/// * Duplicate (url_id, suggested_url_id) → err -1 'already suggested'.
pub async fn create_suggestion(
    State(state): State<Arc<AppState>>,
    auth: crate::routes::auth::AuthUser,
    headers: HeaderMap,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<CreateBody>,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Err(AppError::Unauthorized("login required".to_string()));
    };

    if body.url_id.is_empty() {
        return Ok(Json(json!({ "err": -1, "msg": "url_id is required" })));
    }
    let suggested_url_id = match (&body.suggested_url_id, &body.url) {
        (Some(id), _) if !id.trim().is_empty() => id.trim().to_string(),
        (_, Some(u)) if !u.trim().is_empty() => {
            let url = u.trim().to_string();
            // Reject URLs with no host or a host no scraper knows, BEFORE
            // invoking the scraper. The registry's FanFicFare fallback claims
            // it can handle anything, so a bogus domain would otherwise burn
            // a subprocess round-trip and surface as a 502 instead of a
            // friendly "unsupported URL" 400.
            let host = url
                .split("://")
                .nth(1)
                .and_then(|rest| rest.split(['/', '?', '#']).next())
                .unwrap_or("")
                .to_ascii_lowercase();
            if host.is_empty() {
                return Err(AppError::BadRequest(format!("unsupported URL: {url}")));
            }
            let looks_supported = state
                .scraper_registry
                .has_specific_scraper(&format!("https://{host}/"))
                || state
                    .scraper_registry
                    .has_specific_scraper(&format!("https://{host}/works/1"));
            if !looks_supported {
                return Err(AppError::BadRequest(format!("unsupported URL: {url}")));
            }
            let scraper = state
                .scraper_registry
                .find_specific_or_fff(&url)
                .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {url}")))?;
            let meta = match scraper.lookup(&state.http_client, &url).await {
                Ok(m) => m,
                Err(e) => {
                    state
                        .heal
                        .record_failure(
                            &url,
                            None,
                            &crate::heal::classifier::ErrorKind::from(&e),
                            Some(&e.to_string()),
                            None,
                        )
                        .await;
                    return Err(AppError::ScrapeError(e.to_string()));
                }
            };
            meta.url_id
        }
        _ => {
            return Ok(Json(json!({
                "err": -1,
                "msg": "suggested_url_id or url is required"
            })));
        }
    };

    if suggested_url_id == body.url_id {
        return Ok(Json(json!({ "err": -1, "msg": "can't suggest a fic as similar to itself" })));
    }

    // Seed fic must exist (the per-fic panel only renders on existing fics).
    let seed_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)")
        .bind(&body.url_id)
        .fetch_one(&state.db)
        .await?;
    if !seed_exists {
        return Ok(Json(json!({ "err": -5, "msg": "seed fic not found in database" })));
    }

    // Suggested fic must exist; otherwise enqueue for collection.
    let suggested_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)")
            .bind(&suggested_url_id)
            .fetch_one(&state.db)
            .await?;
    if !suggested_exists {
        state
            .collection_worker
            .enqueue_fic(&suggested_url_id)
            .await?;
        return Ok(Json(json!({
            "err": -5,
            "msg": "that fic hasn't been collected yet — it will be added when the collector picks it up"
        })));
    }

    enforce_write_rate_limit(&state, &headers, remote).await?;

    let comment = body.comment.as_deref().map(str::trim).filter(|c| !c.is_empty());

    let result: Result<(i64,), sqlx::Error> = sqlx::query_as(
        r#"
        INSERT INTO recommendation_suggestions
            (url_id, suggested_url_id, user_id, scraped_url, comment, created_by)
        VALUES ($1, $2, $3, $4, $5, 'user')
        RETURNING id
        "#,
    )
    .bind(&body.url_id)
    .bind(&suggested_url_id)
    .bind(user_id)
    .bind(&body.url)
    .bind(comment)
    .fetch_one(&state.db)
    .await;

    let suggestion_id = match result {
        Ok((id,)) => id,
        Err(sqlx::Error::Database(ref de)) if de.is_unique_violation() => {
            return Ok(Json(json!({ "err": -1, "msg": "already suggested" })));
        }
        Err(e) => return Err(AppError::Database(e.to_string())),
    };

    Ok(Json(json!({
        "err": 0,
        "suggestion_id": suggestion_id,
        "msg": "suggestion added"
    })))
}

/// POST /api/fic-suggestions/{id}/vote
///
/// Vote on a suggestion: 1 up, -1 down, 0 retracts. Authenticated votes are
/// keyed by user_id (one vote per user per suggestion, upsert); anonymous
/// votes fall back to the IP key. Self-voting (the suggestion's author voting
/// on their own suggestion) is rejected with HTTP 400 err -1.
pub async fn vote_suggestion(
    State(state): State<Arc<AppState>>,
    auth: crate::routes::auth::AuthUser,
    headers: HeaderMap,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Path(suggestion_id): Path<i64>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if !(-1..=1).contains(&body.vote) {
        return Ok(Json(json!({ "err": -1, "msg": "vote must be -1, 0 or 1" })));
    }

    let ip = caller_ip(&headers, remote);

    // Load the suggestion; missing/removed → 404-style err.
    let row: Option<(Option<i32>,)> = sqlx::query_as(
        "SELECT user_id FROM recommendation_suggestions WHERE id = $1 AND status = 'active'",
    )
    .bind(suggestion_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((sug_user_id,)) = row else {
        return Ok(Json(json!({ "err": -5, "msg": "suggestion not found" })));
    };

    // No self-vote: the suggestion's author can't vote on their own
    // suggestion (verified for authenticated callers; anonymous callers have
    // no user_id so they can never match the owner).
    if auth.user_id.is_some() && auth.user_id == sug_user_id {
        return Err(AppError::BadRequest("you can't vote on your own suggestion".to_string()));
    }

    // 0 retracts the caller's vote entirely.
    if body.vote == 0 {
        if let Some(voter_id) = auth.user_id {
            sqlx::query("DELETE FROM recommendation_votes WHERE suggestion_id = $1 AND user_id = $2")
                .bind(suggestion_id)
                .bind(voter_id)
                .execute(&state.db)
                .await?;
        } else {
            sqlx::query("DELETE FROM recommendation_votes WHERE suggestion_id = $1 AND voter_ip = $2::inet")
                .bind(suggestion_id)
                .bind(ip.to_string())
                .execute(&state.db)
                .await?;
        }
    } else if let Some(voter_id) = auth.user_id {
        sqlx::query(
            r#"
            INSERT INTO recommendation_votes (suggestion_id, user_id, vote)
            VALUES ($1, $2, $3)
            ON CONFLICT (suggestion_id, user_id) WHERE user_id IS NOT NULL
            DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP
            "#,
        )
        .bind(suggestion_id)
        .bind(voter_id)
        .bind(body.vote as i16)
        .execute(&state.db)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
            VALUES ($1, $2::inet, $3)
            ON CONFLICT (suggestion_id, voter_ip)
            DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP
            "#,
        )
        .bind(suggestion_id)
        .bind(ip.to_string())
        .bind(body.vote as i16)
        .execute(&state.db)
        .await?;
    }

    let new_score: Option<i32> =
        sqlx::query_scalar("SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1")
            .bind(suggestion_id)
            .fetch_one(&state.db)
            .await?;

    Ok(Json(json!({
        "err": 0,
        "suggestion_id": suggestion_id,
        "new_score": new_score.unwrap_or(0),
        "my_vote": if body.vote == 0 { 0 } else { body.vote },
    })))
}

/// POST /api/fic-suggestions/{id}/remove
///
/// Soft-delete a suggestion (status='removed'). Allowed for the suggestion's
/// author or any role >= 10 user. Anonymous callers get 401.
pub async fn remove_suggestion(
    State(state): State<Arc<AppState>>,
    auth: crate::routes::auth::AuthUser,
    Path(suggestion_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let Some(user_id) = auth.user_id else {
        return Err(AppError::Unauthorized("login required".to_string()));
    };

    let row: Option<(Option<i32>,)> = sqlx::query_as(
        "SELECT user_id FROM recommendation_suggestions WHERE id = $1 AND status = 'active'",
    )
    .bind(suggestion_id)
    .fetch_optional(&state.db)
    .await?;
    let Some((sug_user_id,)) = row else {
        return Ok(Json(json!({ "err": -5, "msg": "suggestion not found" })));
    };

    if auth.role < 10 && Some(user_id) != sug_user_id {
        return Err(AppError::Forbidden("not your suggestion".into()));
    }

    sqlx::query("UPDATE recommendation_suggestions SET status = 'removed' WHERE id = $1")
        .bind(suggestion_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "suggestion_id": suggestion_id, "removed": true })))
}
