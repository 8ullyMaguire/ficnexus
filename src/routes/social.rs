use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::limiter::Tier;
use crate::routes::auth::{self, AuthUser, LoginRequest, RefreshRequest, RegisterRequest};
use crate::routes::honeypot::{self, TrapVerdict};
use crate::server::AppState;
use std::time::{SystemTime, UNIX_EPOCH};

/// POST /api/v1/auth/register
pub async fn register_handler(
    State(state): State<Arc<AppState>>,
    visitor: Option<axum::Extension<crate::visitor::VisitorId>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, AppError> {
    // ── Tiered rate limit (auth tier: 10/min per IP) ──────────────────
    enforce_auth_rate_limit(&state).await?;

    // ── Honeypot + timing trap (silent rejection) ──────────────────────
    // A bot that fills the hidden `website` field, or submits without the
    // JS-set `form_opened_at` (or impossibly fast), gets a success-looking
    // response and no account. See `crate::routes::honeypot`.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("registration silently rejected (honeypot/timing trap)");
        // Success-looking response with no token — the bot "registered"
        // without creating anything.
        return Ok(Json(json!({
            "err": 0,
            "token": "",
            "user": {
                "id": 0,
                "username": body.username,
                "trust_level": 0,
                "reputation": 0,
                "email": null,
            },
        })));
    }

    let secret = state.jwt_secret.clone();

    // ── F7: registration mode gate ────────────────────────────────────
    // * invite: an unused, unexpired invite code is REQUIRED (403 without).
    // * application: open signup is disabled — users apply via
    //   POST /api/registration-applications (403 with a pointer).
    // * open (default): no gate; an optional valid code is still consumed.
    let invite_code = body.invite_code.clone().unwrap_or_default();
    match crate::routes::subsystems::registration_mode() {
        "invite" => {
            crate::routes::subsystems::validate_invite_code(&state.db, &invite_code).await?;
        }
        "application" => {
            return Err(AppError::Forbidden(
                "Registration by application only — apply via /api/registration-applications"
                    .to_string(),
            ));
        }
        _ => {}
    }

    let result = auth::register_user(&state.db, body, &secret).await?;

    // Merge anonymous visitor state into the new account (best-effort; never
    // blocks registration success).
    if let Some(axum::Extension(visitor_id)) = visitor {
        if let Err(e) = crate::db::queries::merge_visitor_state_into_user(
            &state.db,
            visitor_id.0,
            result.user.id,
        )
        .await
        {
            tracing::warn!(
                visitor_id = %visitor_id.0,
                user_id = result.user.id,
                error = %e,
                "failed to merge visitor state on registration"
            );
        }
    }

    let refresh = auth::create_refresh_token(result.user.id, &secret)?;

    // Consume the invite code (any mode) once the account exists. In invite
    // mode this was validated above; the UPDATE only touches unused codes so
    // a racing second registration cannot double-consume.
    if !invite_code.trim().is_empty() {
        crate::routes::subsystems::consume_invite_code(&state.db, &invite_code, result.user.id)
            .await;
    }

    Ok(Json(json!({
        "err": 0,
        "token": result.token,
        "refresh_token": refresh,
        "user": result.user,
    })))
}

/// POST /api/v1/auth/login
pub async fn login_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    // ── Tiered rate limit (auth tier: 10/min per IP) ──────────────────
    enforce_auth_rate_limit(&state).await?;

    let secret = state.jwt_secret.clone();
    match auth::login_user(&state.db, body, &secret).await {
        Ok(result) => {
            let refresh = auth::create_refresh_token(result.user.id, &secret)?;
            Ok(Json(json!({
                "err": 0,
                "token": result.token,
                "refresh_token": refresh,
                "user": result.user,
            })))
        }
        Err(e) => {
            // Log the failed auth (bot stuffing signal) — best-effort, never
            // masks the real error. etype='auth_failed' is aggregated by
            // bot-scorer into bot_scores.failed_auths.
            let client_id = headers
                .get("x-client-id")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let user_agent = headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let ip = crate::limiter::client_ip_from_headers(
                headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
                "127.0.0.1".parse().expect("static ip"),
            );
            // request_log.source_id has an FK to request_source; ensure the
            // 'auth' source row exists (idempotent) before logging.
            let auth_source_id: i64 = match sqlx::query_scalar::<_, i64>(
                r#"INSERT INTO request_source (is_automated, route, description)
                   SELECT false, '/api/auth/login', 'auth login attempts'
                   WHERE NOT EXISTS (SELECT 1 FROM request_source WHERE route = '/api/auth/login')
                   RETURNING id"#,
            )
            .fetch_optional(&state.db)
            .await
            {
                Ok(Some(id)) => id,
                _ => sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM request_source WHERE route = '/api/auth/login' LIMIT 1",
                )
                .fetch_one(&state.db)
                .await
                .unwrap_or(1),
            };
            if let Err(e) = crate::db::queries::insert_request_log(
                &state.db,
                auth_source_id,
                "auth_failed",
                "login",
                0,
                None,
                None,
                None,
                None,
                None,
                None,
                client_id.as_deref(),
                user_agent.as_deref(),
                Some(ip),
            )
            .await
            {
                tracing::warn!("failed to log auth_failed: {e}");
            }
            Err(e)
        }
    }
}

/// GET /api/v1/auth/me — return the current user from token
pub async fn me_handler(auth: AuthUser) -> Result<Json<Value>, AppError> {
    match auth.user_id {
        Some(id) => Ok(Json(json!({
            "err": 0,
            "user": {
                "id": id,
                "username": auth.username,
                "trust_level": auth.trust_level,
            }
        }))),
        None => Ok(Json(json!({ "err": 401, "msg": "Not authenticated" }))),
    }
}

/// POST /api/auth/refresh — exchange refresh token for new JWT
pub async fn refresh_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<Value>, AppError> {
    let secret = state.jwt_secret.clone();

    let claims = auth::verify_token(&body.refresh_token, &secret)
        .map_err(|_| AppError::Unauthorized("Invalid refresh token".into()))?;

    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp < now {
        return Err(AppError::Unauthorized("Refresh token expired".into()));
    }

    // Reads `is_admin` from the database rather than copying it off the refresh
    // token, so a demotion takes effect on the next refresh instead of lasting
    // until the old access token expires.
    let user = sqlx::query_as::<_, (i32, String, i16, i32, Option<String>, i16, i64, bool)>(
        "SELECT id, username, trust_level, reputation, email, level, exp, is_admin FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("User not found".into()))?;

    let user = auth::User {
        id: user.0,
        username: user.1,
        trust_level: user.2,
        reputation: user.3,
        email: user.4,
        level: user.5,
        exp: user.6,
        is_admin: user.7,
    };

    let token = auth::create_token(&user, &secret)?;
    let new_refresh = auth::create_refresh_token(user.id, &secret)?;

    Ok(Json(json!({
        "err": 0,
        "token": token,
        "refresh_token": new_refresh,
        "user": user,
    })))
}

// ── Bookmarks ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BookmarkBody {
    pub work_id: i32,
    pub notes: Option<String>,
    pub is_private: Option<bool>,
}

/// POST /api/v1/bookmarks — add a bookmark
pub async fn add_bookmark_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BookmarkBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Resolve the url_id (fic_info.id) for this work — the `bookmarks`
    // table requires `url_id NOT NULL` with `UNIQUE(user_id, url_id)`.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    sqlx::query(
        "INSERT INTO bookmarks (user_id, url_id, work_id, notes, is_private)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, url_id) DO UPDATE SET notes = $4, is_private = $5, work_id = $3",
    )
    .bind(user_id)
    .bind(&url_id)
    .bind(body.work_id)
    .bind(body.notes.as_deref().unwrap_or(""))
    .bind(body.is_private.unwrap_or(false))
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "err": 0, "msg": "Bookmarked" })))
}

/// DELETE /api/v1/bookmarks/{work_id} — remove a bookmark
pub async fn remove_bookmark_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    sqlx::query("DELETE FROM bookmarks WHERE user_id = $1 AND work_id = $2")
        .bind(user_id)
        .bind(work_id)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({ "err": 0, "msg": "Removed" })))
}

/// GET /api/v1/bookmarks — list current user's bookmarks
pub async fn list_bookmarks_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let rows = sqlx::query_as::<_, (i32, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT b.work_id, b.notes, b.is_private, b.created_at
         FROM bookmarks b WHERE b.user_id = $1 ORDER BY b.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let bookmarks: Vec<Value> = rows
        .into_iter()
        .map(|(work_id, notes, is_private, created_at)| {
            json!({
                "work_id": work_id,
                "notes": notes,
                "is_private": is_private,
                "created_at": created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "bookmarks": bookmarks })))
}

/// GET /api/bookmarks/export — export bookmarks as CSV
pub async fn export_bookmarks_csv(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let rows = sqlx::query_as::<_, (String, String, String, chrono::DateTime<chrono::Utc>)>(
        r#"SELECT w.canonical_title, w.canonical_author, fi.source, b.created_at
           FROM bookmarks b
           JOIN works w ON w.id = b.work_id
           JOIN fic_info fi ON fi.work_id = w.id AND fi.id = COALESCE(w.default_source_id, (SELECT id FROM fic_info WHERE work_id = w.id LIMIT 1))
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    // Build CSV manually (avoid adding a csv crate dep)
    let mut csv = String::from("title,author,source,bookmarked_at\n");
    for (title, author, source, created) in &rows {
        // Escape double quotes and commas
        let esc = |s: &str| -> String {
            if s.contains(',') || s.contains('"') || s.contains('\n') {
                format!("\"{}\"", s.replace('"', "\"\""))
            } else {
                s.to_string()
            }
        };
        csv.push_str(&format!(
            "{},{},{},{}\n",
            esc(title),
            esc(author),
            esc(source),
            created.format("%Y-%m-%d %H:%M:%S UTC"),
        ));
    }

    let headers = [
        (axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            axum::http::header::CONTENT_DISPOSITION,
            "attachment; filename=\"ficnexus-bookmarks.csv\"",
        ),
    ];

    Ok((headers, csv).into_response())
}

/// POST /api/bookmarks/import — import bookmarks from a CSV file (multipart upload)
///
/// The CSV is parsed in the background by `BookmarkImportWorker`; this
/// handler only validates the upload and enqueues a job onto
/// `bookmark_import_queue`. The user gets a `bookmark_import` notification
/// when the job finishes.
pub async fn import_bookmarks_csv(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let mut csv_data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            csv_data = Some(field.bytes().await.unwrap_or_default().to_vec());
        }
    }

    let csv_bytes =
        csv_data.ok_or_else(|| AppError::BadRequest("CSV file required".to_string()))?;

    // Size guard: refuse obviously-oversized uploads before queueing.
    const MAX_IMPORT_BYTES: usize = 10 * 1024 * 1024;
    if csv_bytes.len() > MAX_IMPORT_BYTES {
        return Err(AppError::BadRequest(format!(
            "CSV file too large (max {} bytes)",
            MAX_IMPORT_BYTES
        )));
    }

    let csv_str = String::from_utf8_lossy(&csv_bytes).to_string();

    let job = crate::services::bookmark_import::ImportJob {
        user_id,
        csv: csv_str,
    };
    crate::services::bookmark_import::enqueue_import(&state.db, &state.redis, &job)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue import job: {e}")))?;

    Ok(Json(json!({
        "err": 0,
        "status": "queued",
        "msg": "Import queued — you'll be notified when it finishes.",
    })))
}

// ── Ratings ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RatingBody {
    pub work_id: i32,
    pub rating: i16, // 1..=5 stars; -1 accepted for legacy clients (internal only)
}

/// POST /api/v1/ratings — rate a work (5-star scale).
///
/// `rating` must be 1..=5. The legacy value -1 (old "dislike") is still
/// accepted for backward compatibility and stored as an internal-only
/// rec-engine signal — it is never returned by the aggregate endpoints, which
/// expose only positive feedback (stars + like count).
pub async fn rate_work_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<RatingBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if !(1..=5).contains(&body.rating) && body.rating != -1 {
        return Err(AppError::BadRequest(
            "Rating must be 1..=5 stars (legacy -1 accepted)".to_string(),
        ));
    }

    // Resolve the url_id (fic_info.id) for this work — `work_ratings` has
    // UNIQUE(user_id, url_id), mirroring the bookmark handler.
    let url_id = crate::services::bookmark_import::resolve_url_id_for_work(&state.db, body.work_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", body.work_id)))?;

    sqlx::query(
        "INSERT INTO work_ratings (user_id, work_id, url_id, rating) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, url_id) DO UPDATE SET rating = $4, work_id = $2",
    )
    .bind(user_id)
    .bind(body.work_id)
    .bind(&url_id)
    .bind(body.rating)
    .execute(&state.db)
    .await?;

    Ok(Json(rating_aggregate_json(&state, body.work_id).await?))
}

/// GET /api/v1/ratings/{work_id} — public aggregate for a work.
///
/// Only positive signals are exposed: avg_rating (1..=5), the per-star
/// distribution and the like count (5-star ratings + non-deleted reviews).
/// Legacy dislike rows stay in the DB for the rec engine but are never
/// surfaced here.
pub async fn get_ratings_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(rating_aggregate_json(&state, work_id).await?))
}

/// Build the positive-only rating aggregate for a work.
async fn rating_aggregate_json(state: &Arc<AppState>, work_id: i32) -> Result<Value, AppError> {
    // Star-scale aggregate (1..=5 only — legacy -1 rows are internal).
    let (rating_count, sum): (i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN 1 ELSE 0 END), 0)::bigint,
            COALESCE(SUM(CASE WHEN rating BETWEEN 1 AND 5 THEN rating ELSE 0 END), 0)::bigint
         FROM work_ratings WHERE work_id = $1",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    let dist_rows: Vec<(i16, i64)> = sqlx::query_as(
        "SELECT rating, COUNT(*)::bigint FROM work_ratings
         WHERE work_id = $1 AND rating BETWEEN 1 AND 5 GROUP BY rating",
    )
    .bind(work_id)
    .fetch_all(&state.db)
    .await?;

    let mut distribution = [0i64; 5];
    for (star, count) in dist_rows {
        if (1..=5).contains(&star) {
            distribution[(star - 1) as usize] = count;
        }
    }

    let (review_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM reviews WHERE work_id = $1 AND deleted_at IS NULL AND constructive = TRUE",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    let avg_rating = if rating_count > 0 {
        f64::round((sum as f64 / rating_count as f64) * 100.0) / 100.0
    } else {
        0.0
    };

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "avg_rating": avg_rating,
        "rating_count": rating_count,
        // Positive-only public signal: a "like" = a 5-star rating (legacy
        // 1 rows were migrated to 5 in migration 014) or a written review.
        "likes": rating_count + review_count,
        "review_count": review_count,
        "rating_distribution": {
            "1": distribution[0],
            "2": distribution[1],
            "3": distribution[2],
            "4": distribution[3],
            "5": distribution[4],
        },
    }))
}

// ── Kudos ─────────────────────────────────────────────────────────────
// One-click positive appreciation (no text), completely separate from the
// 5-star `work_ratings` system. Logged-in users get one kudos per work
// (`UNIQUE(work_id, user_id)`); guests (user_id NULL) get at most one
// anonymous kudos per work via a partial unique index. `kudos_count`
// counts ONLY signed-in kudos; `guest_count` is 0 or 1.

/// Aggregate kudos state for a work.
///
/// * `kudos_count` — signed-in kudos only (`user_id IS NOT NULL`); this is
///   the number surfaced in search's `min_kudos` filter and `kudos` sort.
/// * `guest_count` — anonymous guest kudos (0 or 1, one per work).
/// * `my_kudos` — whether `viewer_id` (the signed-in user, if any) has
///   kudos'd the work. Always false for anonymous viewers.
async fn kudos_aggregate_json(
    state: &Arc<AppState>,
    work_id: i32,
    viewer_id: Option<i32>,
) -> Result<Value, AppError> {
    let (kudos_count, guest_count): (i64, i64) = sqlx::query_as(
        "SELECT
            COUNT(*) FILTER (WHERE user_id IS NOT NULL)::bigint,
            COUNT(*) FILTER (WHERE user_id IS NULL)::bigint
         FROM kudos WHERE work_id = $1",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    let my_kudos = match viewer_id {
        Some(uid) => {
            sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM kudos WHERE work_id = $1 AND user_id = $2)",
            )
            .bind(work_id)
            .bind(uid)
            .fetch_one(&state.db)
            .await?
        }
        None => false,
    };

    Ok(json!({
        "err": 0,
        "work_id": work_id,
        "kudos_count": kudos_count,
        "guest_count": guest_count,
        "my_kudos": my_kudos,
    }))
}

/// POST /api/kudos/{work_id} — give kudos (auth required).
///
/// Idempotent: a second POST from the same user is a no-op that still
/// returns the current count. 404 if the work does not exist.
pub async fn give_kudos_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let work_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM works WHERE id = $1)")
        .bind(work_id)
        .fetch_one(&state.db)
        .await?;
    if !work_exists {
        return Err(AppError::NotFound(format!("Work {work_id} not found")));
    }

    sqlx::query("INSERT INTO kudos (work_id, user_id) VALUES ($1, $2) ON CONFLICT (work_id, user_id) DO NOTHING")
        .bind(work_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        kudos_aggregate_json(&state, work_id, Some(user_id)).await?,
    ))
}

/// DELETE /api/kudos/{work_id} — remove kudos (auth required).
///
/// Idempotent: deleting a kudos you never gave returns the current count.
pub async fn remove_kudos_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    sqlx::query("DELETE FROM kudos WHERE work_id = $1 AND user_id = $2")
        .bind(work_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        kudos_aggregate_json(&state, work_id, Some(user_id)).await?,
    ))
}

/// GET /api/kudos/{work_id} — public kudos state for a work.
///
/// `my_kudos` reflects the signed-in viewer when a Bearer token is
/// present; anonymous viewers always get `my_kudos: false`.
pub async fn get_kudos_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(
        kudos_aggregate_json(&state, work_id, auth.user_id).await?,
    ))
}

// ── Comments ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CommentBody {
    pub work_id: i32,
    pub url_id: Option<String>,
    pub body: String,
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub form_opened_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReadingRecordBody {
    pub work_id: i32,
    pub words_read: i64,
}

/// POST /api/v1/comments — post a comment
pub async fn add_comment_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CommentBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // ── Honeypot + timing trap (silent rejection) ──────────────────────
    // Same trap as registration: a bot that fills `website` or submits too
    // fast gets a success-looking response and the comment is dropped.
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let verdict = honeypot::inspect_submission(
        body.website.as_deref(),
        body.form_opened_at.as_deref(),
        now_ms,
    );
    if verdict == TrapVerdict::RejectSilently {
        tracing::warn!("comment silently rejected (honeypot/timing trap)");
        return Ok(Json(json!({
            "err": 0,
            "comment_id": 0,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "username": auth.username,
        })));
    }

    if body.body.trim().is_empty() {
        return Err(AppError::BadRequest("Comment cannot be empty".to_string()));
    }
    if body.body.len() > 2000 {
        return Err(AppError::BadRequest(
            "Comment too long (max 2000 chars)".to_string(),
        ));
    }

    // If url_id not provided, use default_source_id from work
    let url_id = match body.url_id {
        Some(id) => id,
        None => {
            let work = sqlx::query_as::<_, (Option<String>,)>(
                "SELECT default_source_id FROM works WHERE id = $1",
            )
            .bind(body.work_id)
            .fetch_optional(&state.db)
            .await?
            .and_then(|r| r.0)
            .ok_or_else(|| {
                AppError::BadRequest("Work not found or no default source".to_string())
            })?;
            work
        }
    };

    // The comments.work_id FK requires the work row to exist. A stale/unknown
    // work_id must be a 400, not a 500 (it 500s via FK violation otherwise).
    {
        let work_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM works WHERE id = $1)")
                .bind(body.work_id)
                .fetch_one(&state.db)
                .await?;
        if !work_exists {
            return Err(AppError::BadRequest("Work not found".to_string()));
        }
    }

    let (id, created_at): (i64, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO comments (user_id, work_id, url_id, parent_id, body, constructive)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, created_at",
    )
    .bind(user_id)
    .bind(body.work_id)
    .bind(&url_id)
    .bind(body.parent_id)
    .bind(&body.body)
    .bind(crate::routes::comments::constructive_score(&body.body))
    .fetch_one(&state.db)
    .await?;

    // ── Comment moderation triage (best-effort, fire-and-forget) ─────
    // The comment post must succeed even if Ollama is down: classification
    // runs in its own task and every failure inside it falls back to a
    // `Fine` verdict with confidence 0.0 (never an error).
    {
        let db = state.db.clone();
        let ollama = state.ollama.clone();
        let body = body.body.clone();
        tokio::spawn(async move {
            let triage =
                crate::services::comment_triage::classify_and_store(&db, &ollama, id, &body).await;
            tracing::debug!("comment {id} triaged as {:?}", triage.category);
        });
    }

    Ok(Json(json!({
        "err": 0,
        "comment_id": id,
        "created_at": created_at.to_rfc3339(),
        "username": auth.username,
    })))
}

/// GET /api/v1/comments/{work_id} — list comments for a work
pub async fn list_comments_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            i32,
            String,
            String,
            Option<i64>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT c.id, c.user_id, u.username, c.body, c.parent_id, c.created_at
         FROM comments c
         JOIN users u ON u.id = c.user_id
         WHERE c.work_id = $1
           AND c.deleted_at IS NULL
           AND c.constructive = TRUE
         ORDER BY c.created_at ASC",
    )
    .bind(work_id)
    .fetch_all(&state.db)
    .await?;

    let comments: Vec<Value> = rows
        .into_iter()
        .map(|(id, user_id, username, body, parent_id, created_at)| {
            json!({
                "id": id,
                "user_id": user_id,
                "username": username,
                "body": body,
                "parent_id": parent_id,
                "created_at": created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "work_id": work_id,
        "comments": comments,
    })))
}

// ── Leaderboards ───────────────────────────────────────────────────────

/// GET /api/v1/leaderboard/curators — top curators by reputation
pub async fn leaderboard_curators_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT id, username, reputation FROM users
         WHERE reputation > 0 ORDER BY reputation DESC LIMIT 20",
    )
    .fetch_all(&state.db)
    .await?;

    let entries: Vec<Value> = rows
        .into_iter()
        .map(|(id, username, reputation)| {
            json!({
                "id": id,
                "username": username,
                "reputation": reputation,
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "leaderboard": entries })))
}

/// GET /api/v1/users/{id} — get a user's public profile
pub async fn user_profile_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query_as::<
        _,
        (
            i32,
            String,
            i16,
            i32,
            Option<String>,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT id, username, role, reputation, email, created_at FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let (id, username, role, reputation, email, created_at) = row;

    // Get badge count
    let badge_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_badges WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "user": {
            "id": id,
            "username": username,
            "role": role,
            "reputation": reputation,
            "email": email,
            "created_at": created_at.to_rfc3339(),
            "badges": badge_count.0,
        }
    })))
}

// ── Works ────────────────────────────────────────────────────────────

/// GET /api/works/{id} — get work metadata with sources
pub async fn get_work_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let work = crate::db::queries::get_work(&state.db, work_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Work not found".into()))?;

    // Get sources (fic_info rows linked to this work)
    let sources = crate::db::queries::get_work_sources(&state.db, work_id).await?;

    let source_list: Vec<Value> = sources
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id,
                "source": s.source,
                "title": s.title,
                "author": s.author,
                "words": s.words,
                "chapters": s.chapters,
                "status": s.status,
                "url": s.source,
            })
        })
        .collect();

    // Get aggregate stats
    let (total_bookmarks, total_ratings, total_comments): (i64, i64, i64) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM bookmarks WHERE work_id = $1),
            (SELECT COUNT(*) FROM work_ratings WHERE work_id = $1),
            (SELECT COUNT(*) FROM comments WHERE work_id = $1)",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "work": {
            "id": work.id,
            "canonical_title": work.canonical_title,
            "canonical_author": work.canonical_author,
            "description": work.description,
            "default_source_id": work.default_source_id,
            "created_at": work.created_at.to_rfc3339(),
            "updated_at": work.updated_at.to_rfc3339(),
            "sources": source_list,
            "total_bookmarks": total_bookmarks,
            "total_ratings": total_ratings,
            "total_comments": total_comments,
        }
    })))
}

/// GET /api/v1/works/{id}/stats — per-work engagement stats for the dashboard.
/// Returns kudos/guest counts + bookmark/rating/comment totals.
pub async fn work_stats_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(work_id): axum::extract::Path<i32>,
) -> Result<Json<Value>, AppError> {
    let (kudos_count, guest_count, total_bookmarks, total_ratings, total_comments): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT
            (SELECT COUNT(*) FROM kudos WHERE work_id = $1 AND user_id IS NOT NULL),
            (SELECT COUNT(*) FROM kudos WHERE work_id = $1 AND user_id IS NULL),
            (SELECT COUNT(*) FROM bookmarks WHERE work_id = $1),
            (SELECT COUNT(*) FROM work_ratings WHERE work_id = $1),
            (SELECT COUNT(*) FROM comments WHERE work_id = $1)",
    )
    .bind(work_id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "work_id": work_id,
        "kudos_count": kudos_count,
        "guest_count": guest_count,
        "total_bookmarks": total_bookmarks,
        "total_ratings": total_ratings,
        "total_comments": total_comments,
        "total_kudos": kudos_count + guest_count,
    })))
}

/// GET /api/my-works — list the authenticated user's own visible uploads, newest first.
pub async fn list_my_works_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let works = crate::db::queries::list_my_works(&state.db, user_id).await?;
    let items = works
        .into_iter()
        .map(|w| {
            json!({
                "id": w.id,
                "canonical_title": w.canonical_title,
                "canonical_author": w.canonical_author,
                "default_source_id": w.default_source_id,
                "created_at": w.created_at.to_rfc3339(),
                "updated_at": w.updated_at.to_rfc3339(),
                "is_visible": w.is_visible,
            })
        })
        .collect::<Vec<Value>>();
    Ok(Json(json!({ "err": 0, "works": items })))
}

/// Enforce the auth-tier rate limit (10/min per IP by default) for
/// login/register. In test mode (`dynamic_rate_limit == false`) this is a
/// no-op static delay, so existing honeypot/auth integration tests are
/// unaffected. Redis errors fail open — a Redis hiccup never blocks auth.
async fn enforce_auth_rate_limit(state: &Arc<AppState>) -> Result<(), AppError> {
    let ip = crate::limiter::client_ip_from_headers(
        None,
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    match state.rate_limiter.check(ip, None, Tier::Auth).await {
        crate::limiter::TieredRateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        crate::limiter::TieredRateLimitResult::Allowed => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::routes::auth::{User, create_token, verify_token};

    #[test]
    fn test_token_roundtrip() {
        let user = User {
            id: 42,
            username: "alice".into(),
            trust_level: 1,
            reputation: 100,
            email: Some("alice@example.com".into()),
            level: 0,
            exp: 0,
            is_admin: false,
        };
        let secret = "test-secret";
        let token = create_token(&user, secret).unwrap();
        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, 42);
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.trust_level, 1);
    }

    #[test]
    fn test_expired_token_rejected() {
        // Create a token with a past expiry by crafting claims manually
        let claims = super::super::auth::Claims {
            sub: 1,
            username: "test".into(),
            trust_level: 0,
            level: 0,
            is_admin: false,
            exp: 1, // Jan 1 1970 — expired
            iat: 1,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(b"test-secret"),
        )
        .unwrap();
        let result = verify_token(&token, "test-secret");
        assert!(result.is_err());
    }
}
