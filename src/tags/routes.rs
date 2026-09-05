use axum::{
    Json,
    extract::{ConnectInfo, Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::server::AppState;
use crate::tags::{resolve, voting};

// ---- Request structures ----

#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    pub url_id: String,
    pub tag_name: String,
    pub tag_type_id: i16,
}

#[derive(Debug, Deserialize)]
pub struct VoteBody {
    pub url_id: String,
    pub tag_id: i32,
    pub value: i16,
}

#[derive(Debug, Deserialize)]
pub struct FlagBody {
    pub url_id: String,
    pub tag_id: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TagsQuery {
    pub url_id: String,
}

// ---- Handlers ----

/// POST /api/v0/tags/submit
///
/// Submit a tag on a fic. The tag string is resolved to a canonical tag
/// (creating a new one if necessary) and attached to the fic.
pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    // Validate input
    if body.url_id.is_empty() || body.tag_name.is_empty() {
        return Ok(Json(
            json!({"err": -1, "msg": "url_id and tag_name are required"}),
        ));
    }
    if !(1..=7).contains(&body.tag_type_id) {
        return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be 1-7"})));
    }

    // Rate limit check (Redis-backed)
    let mut redis = state.redis.clone();
    check_tag_rate_limit(
        &mut redis,
        "submit",
        ip,
        state.config.tag_submit_limit_per_hour,
    )
    .await?;

    // Verify the fic exists in the database
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)")
        .bind(&body.url_id)
        .fetch_one(&state.db)
        .await?;

    if !exists {
        return Ok(Json(json!({"err": -5, "msg": "fic not found"})));
    }

    // Resolve the tag string to a canonical tag
    let resolution = resolve::resolve_tag(&state.db, &body.tag_name, body.tag_type_id).await?;

    // Attach the tag to the fic (idempotent)
    let ip_str = ip.to_string();
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, 0)
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    )
    .bind(&body.url_id)
    .bind(resolution.tag_id)
    .bind(&ip_str)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "tag_id": resolution.tag_id,
        "tag_name": resolution.tag_name,
        "tag_type_id": resolution.tag_type_id,
        "is_new": resolution.is_new,
    })))
}

/// POST /api/v0/tags/vote
///
/// Vote (upvote = 1, downvote = -1) on a tag attached to a fic.
/// The trigger on `fic_tag_votes` automatically adjusts the aggregate score.
pub async fn vote_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    if body.url_id.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id is required"})));
    }
    if body.value != 1 && body.value != -1 {
        return Ok(Json(json!({"err": -1, "msg": "value must be 1 or -1"})));
    }

    // Rate limit check
    let mut redis = state.redis.clone();
    check_tag_rate_limit(&mut redis, "vote", ip, state.config.tag_vote_limit_per_hour).await?;

    // Record the vote
    let result = voting::record_vote(
        &state.db,
        &body.url_id,
        body.tag_id,
        ip,
        body.value,
        state.config.tag_hidden_threshold,
    )
    .await?;

    Ok(Json(json!({
        "err": 0,
        "new_score": result.new_score,
        "hidden": result.hidden,
    })))
}

/// POST /api/v0/tags/flag
///
/// Flag a tag on a fic for curator review.
/// Duplicate flags from the same IP update the reason and reset resolved status.
pub async fn flag_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<FlagBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    if body.url_id.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id is required"})));
    }

    let ip_str = ip.to_string();
    sqlx::query(
        r#"INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, flagged_by_ip) DO UPDATE SET
               reason = EXCLUDED.reason,
               resolved = FALSE"#,
    )
    .bind(&body.url_id)
    .bind(body.tag_id)
    .bind(&ip_str)
    .bind(&body.reason)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({"err": 0, "msg": "flag submitted"})))
}

/// GET /api/v0/tags?url_id=<id>
///
/// Return all tags attached to a fic, with their scores and visibility status.
pub async fn get_tags(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TagsQuery>,
) -> Result<Json<Value>, AppError> {
    if params.url_id.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id is required"})));
    }

    let rows: Vec<(i32, String, i16, i16)> = sqlx::query_as(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, t.name"#,
    )
    .bind(&params.url_id)
    .fetch_all(&state.db)
    .await?;

    let threshold = state.config.tag_hidden_threshold;
    let tags: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, tag_type_id, score)| {
            json!({
                "id": id,
                "name": name,
                "tag_type_id": tag_type_id,
                "score": score,
                "hidden": voting::is_hidden(score, threshold),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "url_id": params.url_id,
        "tags": tags,
    })))
}

// ---- Rate limiting (Redis-backed) ----

/// Check and record a rate-limited action for the given IP.
///
/// Tracks counts in Redis with a 1-hour TTL using key pattern:
/// `ratelimit:tag:{action}:{ip}`. Returns `AppError::RateLimited` when the
/// limit is exceeded so the handler returns HTTP 429.
async fn check_tag_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    action: &str,
    ip: std::net::IpAddr,
    limit: u32,
) -> AppResult<()> {
    let key = format!("ratelimit:tag:{}:{}", action, ip);

    // Read current count
    let count: Option<u32> = redis::cmd("GET")
        .arg(&key)
        .query_async(redis)
        .await
        .unwrap_or(None);

    if let Some(c) = count {
        if c >= limit {
            let ttl: u64 = redis::cmd("TTL")
                .arg(&key)
                .query_async(redis)
                .await
                .unwrap_or(3600);
            return Err(AppError::RateLimited(ttl));
        }
    }

    // Increment and set TTL on first visit
    let new_count: u32 = redis::cmd("INCR").arg(&key).query_async(redis).await?;

    if new_count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600i64)
            .query_async(redis)
            .await?;
    }

    Ok(())
}

// ── Tag Health & Status Endpoints ──────────────────────────────────────

/// GET /api/tags/resolve?q= — resolve a tag query to canonical form
pub async fn resolve_tag_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ResolveParams>,
) -> Result<Json<Value>, AppError> {
    let resolved = if let Some(ref q) = params.q {
        if q.is_empty() {
            None
        } else {
            crate::search::tags::resolve_tag(&state.db, q).await?
        }
    } else {
        None
    };

    Ok(Json(json!({"err": 0, "result": resolved})))
}

#[derive(Debug, Deserialize)]
pub struct ResolveParams {
    pub q: Option<String>,
}

/// GET /api/tags/search?q=&type=&canonical=&fandom=&page=&limit=&sort=
pub async fn search_tags(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TagSearchParams>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * limit;

    // Build dynamic query
    let _limit_val = limit as i64;
    let _offset_val = offset as i64;

    // Count total matching tags using QueryBuilder
    let mut count_qb =
        sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT COUNT(*) FROM tags t WHERE 1=1");

    if let Some(ref q) = params.q {
        if !q.is_empty() {
            let like_val = format!("%{}%", q);
            count_qb.push(" AND t.name ILIKE ");
            count_qb.push_bind(like_val.clone());
        }
    }

    if let Some(type_id) = params.tag_type {
        count_qb.push(" AND t.tag_type_id = ");
        count_qb.push_bind(type_id);
    }

    if let Some(is_canonical) = params.canonical {
        if is_canonical {
            count_qb.push(" AND t.id NOT IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)");
        } else {
            count_qb.push(" AND t.id IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)");
        }
    }

    let total: (i64,) = count_qb.build_query_as().fetch_one(&state.db).await?;

    let sort_clause_str = match params.sort.as_deref() {
        Some("usage") => "ORDER BY usage_count DESC",
        Some("created") => "ORDER BY t.created_at DESC NULLS LAST",
        _ => "ORDER BY t.name ASC",
    };

    // Fetch page of tags with usage counts using QueryBuilder
    let mut data_qb = sqlx::QueryBuilder::<sqlx::Postgres>::new(
        r#"SELECT t.id, t.name, t.tag_type_id, t.description,
                  (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count,
                  EXISTS(SELECT 1 FROM tag_aliases ta WHERE ta.alias_name = t.name) as is_alias
           FROM tags t WHERE 1=1"#,
    );

    if let Some(ref q) = params.q {
        if !q.is_empty() {
            let like_val = format!("%{}%", q);
            data_qb.push(" AND t.name ILIKE ");
            data_qb.push_bind(like_val);
        }
    }

    if let Some(type_id) = params.tag_type {
        data_qb.push(" AND t.tag_type_id = ");
        data_qb.push_bind(type_id);
    }

    if let Some(is_canonical) = params.canonical {
        if is_canonical {
            data_qb.push(" AND t.id NOT IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)");
        } else {
            data_qb.push(" AND t.id IN (SELECT DISTINCT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)");
        }
    }

    data_qb.push(" ");
    data_qb.push(sort_clause_str);
    data_qb.push(" LIMIT ");
    data_qb.push_bind(limit as i64);
    data_qb.push(" OFFSET ");
    data_qb.push_bind(offset as i64);

    #[derive(sqlx::FromRow)]
    struct TagSearchRow {
        id: i32,
        name: String,
        tag_type_id: i16,
        description: Option<String>,
        usage_count: i64,
        is_alias: bool,
    }

    let rows: Vec<TagSearchRow> = data_qb.build_query_as().fetch_all(&state.db).await?;

    let tags: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "type_id": r.tag_type_id,
                "type_name": crate::search::tags::get_type_name(r.tag_type_id),
                "description": r.description,
                "usage_count": r.usage_count,
                "is_canonical": !r.is_alias,
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "total": total.0,
        "page": params.page.unwrap_or(1),
        "limit": limit,
        "tags": tags,
    })))
}

#[derive(Debug, Deserialize)]
pub struct TagSearchParams {
    pub q: Option<String>,
    pub tag_type: Option<i16>,
    pub canonical: Option<bool>,
    pub fandom: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub sort: Option<String>,
}

/// GET /api/tags/{id} — get single tag with full info
pub async fn get_tag_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    #[derive(sqlx::FromRow)]
    struct TagDetailRow {
        id: i32,
        name: String,
        tag_type_id: i16,
        description: Option<String>,
        created_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let tag: Option<TagDetailRow> = sqlx::query_as::<_, TagDetailRow>(
        r#"SELECT id, name, tag_type_id, description, created_at
           FROM tags WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    let tag = match tag {
        Some(t) => t,
        None => {
            return Ok(Json(json!({"err": -5, "msg": "tag not found"})));
        }
    };

    // Get synonyms (aliases pointing to this canonical tag)
    let synonyms: Vec<String> = {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT alias_name FROM tag_aliases WHERE canonical_tag_id = $1")
                .bind(id)
                .fetch_all(&state.db)
                .await?;
        rows.into_iter().map(|r| r.0).collect()
    };

    // Get usage count
    let usage_count: i64 = {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = $1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;
        count.0
    };

    // Check if this tag is itself an alias target
    let is_alias: bool = {
        let res: Option<(i32,)> = sqlx::query_as(
            "SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = (SELECT name FROM tags WHERE id = $1)",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
        res.is_some()
    };

    let is_canonical = !is_alias;

    Ok(Json(json!({
        "err": 0,
        "tag": {
            "id": tag.id,
            "name": tag.name,
            "type_id": tag.tag_type_id,
            "type_name": crate::search::tags::get_type_name(tag.tag_type_id),
            "description": tag.description,
            "created_at": tag.created_at,
            "is_canonical": is_canonical,
            "synonyms": synonyms,
            "usage_count": usage_count,
        }
    })))
}

/// GET /api/tags/autocomplete?q=&type=&limit= — autocomplete for search UI
pub async fn tag_autocomplete(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TagAutocompleteParams>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).min(25);
    let query = params.q.unwrap_or_default();

    if query.len() < 2 {
        return Ok(Json(json!({"err": 0, "results": []})));
    }

    let like_pattern = format!("{}%", query);

    let rows = if let Some(type_id) = params.tag_type {
        sqlx::query_as::<_, (i32, String, i16, i64)>(
            r#"SELECT t.id, t.name, t.tag_type_id,
                      (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count
               FROM tags t
               WHERE t.name ILIKE $1 AND t.tag_type_id = $2
                  AND t.id NOT IN (SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)
               ORDER BY usage_count DESC
               LIMIT $3"#,
        )
        .bind(&like_pattern)
        .bind(type_id)
        .bind(limit as i64)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, (i32, String, i16, i64)>(
            r#"SELECT t.id, t.name, t.tag_type_id,
                      (SELECT COUNT(*) FROM fic_tags ft WHERE ft.tag_id = t.id) as usage_count
               FROM tags t
               WHERE t.name ILIKE $1
                  AND t.id NOT IN (SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = t.name)
               ORDER BY t.tag_type_id, usage_count DESC
               LIMIT $2"#,
        )
        .bind(&like_pattern)
        .bind(limit as i64)
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(json!({
        "err": 0,
        "results": rows.into_iter().map(|(id, name, type_id, count)| json!({
            "id": id,
            "name": name,
            "type": crate::search::tags::get_type_name(type_id),
            "type_id": type_id,
            "usage_count": count,
        })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
pub struct TagAutocompleteParams {
    pub q: Option<String>,
    pub tag_type: Option<i16>,
    pub limit: Option<usize>,
}

/// Escape a string literal for safe use in SQL (used only for ILIKE patterns
/// where we cannot use parameterized bindings in dynamic query builder).
/// This is sufficient for tag names which have limited character sets.
#[allow(dead_code)]
fn escape_sql_literal(s: &str) -> String {
    let escaped = s.replace('\'', "''");
    format!("'{}'", escaped)
}
