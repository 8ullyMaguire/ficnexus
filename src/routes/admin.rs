use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use chrono::Utc;

// ═══════════════════════════════════════════════════════════════════
// Shared query params
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct PageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct TranslationListParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUserParams {
    pub q: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

// ═══════════════════════════════════════════════════════════════════
// A. Moderation Queue
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/moderation/queue — list pending manual uploads
pub async fn mod_queue(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;

    let rows = sqlx::query_as::<
        _,
        (
            i32,
            String,
            String,
            String,
            String,
            String,
            String,
            chrono::NaiveDateTime,
        ),
    >(
        r#"SELECT w.id, w.canonical_title, w.canonical_author, fi.description,
                  fi.source, u.username, fi.source_type, fi.created
           FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           LEFT JOIN users u ON w.uploader_id = u.id
           WHERE fi.source_type IN ('manual_epub', 'manual_text', 'import')
             AND w.is_visible = FALSE
           ORDER BY fi.created DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM works w LEFT JOIN fic_info fi ON fi.work_id = w.id WHERE fi.source_type IN ('manual_epub','manual_text','import') AND w.is_visible = FALSE"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": rows.into_iter().map(|(id, title, author, desc, source, username, source_type, created)| json!({
            "work_id": id, "title": title, "author": author,
            "description": desc, "source": source,
            "uploader": username, "source_type": source_type,
            "created": created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
        "total": total.0,
        "page": params.page.unwrap_or(1),
    })))
}

/// POST /api/admin/moderation/approve/{work_id} — approve a manual upload
pub async fn approve_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Get uploader_id before making visible
    let uploader: Option<(i32,)> =
        sqlx::query_as("SELECT uploader_id FROM works WHERE id = $1 AND is_visible = FALSE")
            .bind(work_id)
            .fetch_optional(&state.db)
            .await?;

    let (uploader_id,) =
        uploader.ok_or_else(|| AppError::NotFound("No pending upload with this ID".into()))?;

    sqlx::query("UPDATE works SET is_visible = TRUE WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    // v3 XP: qualified publish award (>=5k words) + auto-claim work bounties.
    let (words, status): (Option<i64>, Option<String>) =
        sqlx::query_as("SELECT fi.word_count, fi.status FROM fic_info fi WHERE fi.work_id = $1")
            .bind(work_id)
            .fetch_optional(&state.db)
            .await?
            .unwrap_or((None, None));
    let w = words.unwrap_or(0);
    let _ =
        crate::services::bounties::auto_claim_work_bounties(&state.db, work_id, uploader_id).await;
    // Resolve url_id once — used for publish + completion bonuses.
    let url_id = sqlx::query_scalar::<_, String>(
        "SELECT fi.source_url FROM fic_info fi WHERE fi.work_id = $1",
    )
    .bind(work_id)
    .fetch_optional(&state.db)
    .await?;
    if w >= 5000 {
        if let Some(ref uid) = url_id {
            let _ = crate::services::progression::award_xp(
                &state.db,
                uploader_id,
                "work_publish",
                Some(uid),
            )
            .await;
        } else {
            let _ = crate::services::progression::award_xp(
                &state.db,
                uploader_id,
                "work_publish",
                None,
            )
            .await;
        }
        // Completion bonus: status == 'complete' → scaled XP = 100 + 2*(words/1000).
        if status.as_deref() == Some("complete") {
            let scaled = 100 + 2 * (w / 1000);
            // Override the def's base xp with the scaled amount via a one-off insert.
            // award_xp enforces caps/streak on the def row (work_complete_qualified), but
            // the XP *amount* is scaled here before recording. We call a dedicated path:
            let _ = crate::services::progression::award_scaled_xp(
                &state.db,
                uploader_id,
                "work_complete_qualified",
                url_id.as_deref(),
                scaled as i32,
            )
            .await;
        }
    }
    // Retain legacy reputation path for a single transitional sprint.
    let _ = crate::db::queries::update_reputation_and_promote(
        &state.db,
        uploader_id,
        25,
        "upload_approved",
    )
    .await;
    let _ =
        crate::db::queries::check_and_award_badges(&state.db, uploader_id, "upload_approved").await;

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "approve_upload",
        "work",
        &work_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Upload approved and made visible"}),
    ))
}

/// POST /api/admin/moderation/reject/{work_id} — reject a manual upload
pub async fn reject_upload(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Delete the fic permanently
    sqlx::query("DELETE FROM works WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "reject_upload",
        "work",
        &work_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Upload rejected and deleted"}),
    ))
}

// ═══════════════════════════════════════════════════════════════════
// B. Scraper Health Dashboard
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/scraper-health — request success rates grouped by site
pub async fn scraper_health(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Group by source_url (domain), not fic_info.source — request_log has its own field
    let rows = sqlx::query_as::<_, (String, i64, i64, f64)>(
        r#"SELECT
            COALESCE(rl.source_url, 'unknown') as site,
            COUNT(*) as total,
            SUM(CASE WHEN rl.status = 'success' THEN 1 ELSE 0 END) as successes,
            ROUND(100.0 * SUM(CASE WHEN rl.status = 'success' THEN 1 ELSE 0 END) / COUNT(*), 1) as success_rate
           FROM request_log rl
           WHERE rl.created > NOW() - INTERVAL '7 days'
           GROUP BY rl.source_url
           ORDER BY total DESC
           LIMIT 50"#
    )
    .fetch_all(&state.db)
    .await?;

    // Also get recent errors
    let errors = sqlx::query_as::<_, (String, String, chrono::NaiveDateTime)>(
        r#"SELECT COALESCE(rl.source_url, 'unknown'), COALESCE(rl.error_message, ''), rl.created
           FROM request_log rl
           WHERE rl.status = 'error' AND rl.created > NOW() - INTERVAL '24 hours'
           ORDER BY rl.created DESC
           LIMIT 20"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "sites": rows.into_iter().map(|(url, total, successes, rate)| json!({
            "site": url,
            "total_requests": total,
            "successes": successes,
            "success_rate": rate,
        })).collect::<Vec<_>>(),
        "recent_errors": errors.into_iter().map(|(url, msg, time)| json!({
            "site": url,
            "error": msg,
            "time": time.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
    })))
}

// ═══════════════════════════════════════════════════════════════════
// C. User Management
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/users?q=&page= — search users
pub async fn admin_users(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<AdminUserParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;

    if let Some(ref q) = params.q {
        let rows = sqlx::query_as::<_, (i32, String, i16, i32, Option<i64>, bool, Option<String>)>(
            r#"SELECT id, username, role, reputation, total_words_read, is_banned, locale
               FROM users
               WHERE username ILIKE $1
               ORDER BY id
               LIMIT $2 OFFSET $3"#,
        )
        .bind(format!("%{}%", q))
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(&state.db)
        .await?;

        return Ok(Json(json!({
            "err": 0,
            "users": rows.into_iter().map(|(id, uname, role, rep, words, banned, locale)| json!({
                "id": id, "username": uname, "role": role, "reputation": rep,
                "total_words_read": words, "is_banned": banned, "locale": locale,
            })).collect::<Vec<_>>(),
            "page": params.page.unwrap_or(1),
        })));
    }

    let rows = sqlx::query_as::<_, (i32, String, i16, i32, Option<i64>, bool, Option<String>)>(
        r#"SELECT id, username, role, reputation, total_words_read, is_banned, locale
           FROM users ORDER BY id LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "users": rows.into_iter().map(|(id, uname, role, rep, words, banned, locale)| json!({
            "id": id, "username": uname, "role": role, "reputation": rep,
            "total_words_read": words, "is_banned": banned, "locale": locale,
        })).collect::<Vec<_>>(),
        "page": params.page.unwrap_or(1),
    })))
}

/// PUT /api/admin/users/{id}/role — change user role
pub async fn set_user_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let new_role: i16 = payload
        .get("trust_level")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16)
        .ok_or_else(|| AppError::BadRequest("trust_level field required (0-6)".to_string()))?;

    sqlx::query("UPDATE users SET trust_level = $1 WHERE id = $2")
        .bind(new_role)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "set_user_role",
        "user",
        &user_id.to_string(),
        vec![("trust_level", json!(new_role))],
    )
    .await;

    Ok(Json(json!({"err": 0, "msg": "Role updated"})))
}

/// PUT /api/admin/users/{id}/ban — toggle ban
pub async fn toggle_ban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(user_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let banned: bool = payload
        .get("is_banned")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| AppError::BadRequest("is_banned field required".to_string()))?;

    sqlx::query("UPDATE users SET is_banned = $1 WHERE id = $2")
        .bind(banned)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        if banned { "ban_user" } else { "unban_user" },
        "user",
        &user_id.to_string(),
        vec![],
    )
    .await;

    Ok(Json(
        json!({"err": 0, "msg": if banned { "User banned" } else { "User unbanned" }}),
    ))
}

/// POST /api/admin/reputation/award — manually grant an admin-only XP/reputation
/// source (scraper fixes, code PRs, tag-wiki merges, marathon titles, etc.).
///
/// Gated twice: route auth (`role >= 10`) + event-type allowlist (only rows whose
/// `xp_source_defs.description` starts with `Admin:`). Everything else is
/// reachable only through automated code paths. Every grant is modlogged.
#[derive(Debug, Deserialize)]
pub struct RepAwardReq {
    pub user_id: i32,
    pub event_type: String,
    pub source_ref: Option<String>,
    pub note: Option<String>,
}

pub async fn rep_award_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(req): Json<RepAwardReq>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let admin_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    // Allowlist: only Admin:-prefixed source defs are grantable here.
    let is_admin_only: (bool,) = sqlx::query_as::<_, (bool,)>(
        "SELECT description LIKE 'Admin:%' FROM xp_source_defs WHERE event_type = $1",
    )
    .bind(&req.event_type)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest(format!("Unknown event_type: {}", req.event_type)))?;
    if !is_admin_only.0 {
        return Err(AppError::BadRequest("Not an admin-awarded reward".into()));
    }
    let awarded = crate::services::progression::award_xp(
        &state.db,
        req.user_id,
        &req.event_type,
        req.source_ref.as_deref(),
    )
    .await?;
    // Badge side-effect: admin-granted event types (e.g. marathon_writer)
    // should also issue the matching user_badges record. check_and_award_badges
    // is keyed on event_type matching badge_definitions.trigger, so the same
    // grant that awards XP also stamps the badge.
    let _ =
        crate::db::queries::check_and_award_badges(&state.db, req.user_id, &req.event_type).await;
    crate::modlog::record_json(
        &state.db,
        Some(admin_id),
        auth.username.clone(),
        "reputation_award",
        "user",
        &req.user_id.to_string(),
        vec![
            ("event_type", json!(req.event_type)),
            ("source_ref", json!(req.source_ref)),
            ("note", json!(req.note)),
            ("xp_awarded", json!(awarded)),
        ],
    )
    .await;
    Ok(Json(json!({ "err": 0, "ok": true, "xp_awarded": awarded })))
}

// ═══════════════════════════════════════════════════════════════════
// E. Translation review workflow
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/translations?status=draft — list work translations in the
/// given review state (default `draft`). Curators post-edit the machine
/// output, then approve or reject. Only `approved` rows are served by the
/// public translation endpoints.
pub async fn admin_list_translations(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<TranslationListParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;

    // status filter — validated against the CHECK constraint values.
    let status = params.status.as_deref().unwrap_or("draft");
    if !matches!(status, "draft" | "approved" | "rejected") {
        return Err(AppError::BadRequest(
            "status must be one of draft, approved, rejected".to_string(),
        ));
    }

    let rows = sqlx::query_as::<
        _,
        (
            i64,
            i32,
            String,
            Option<String>,
            Option<String>,
            Option<i32>,
            chrono::DateTime<chrono::Utc>,
            String,
            Option<i32>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(
        r#"SELECT id, work_id, locale_code, title, summary, translated_by, translated_at,
                  status, reviewed_by, reviewed_at
           FROM work_translations
           WHERE status = $1
           ORDER BY translated_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(status)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM work_translations WHERE status = $1")
        .bind(status)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "err": 0,
        "items": rows.into_iter().map(|(id, work_id, locale, title, summary, by, at, st, reviewed_by, reviewed_at)| json!({
            "id": id,
            "work_id": work_id,
            "locale_code": locale,
            "title": title,
            "summary": summary,
            "translated_by": by,
            "translated_at": at.to_rfc3339(),
            "status": st,
            "reviewed_by": reviewed_by,
            "reviewed_at": reviewed_at.map(|t| t.to_rfc3339()),
        })).collect::<Vec<_>>(),
        "total": total.0,
        "status": status,
        "page": params.page.unwrap_or(1),
    })))
}

/// POST /api/admin/translations/{id}/approve — mark a translation approved.
/// The translated text is live from this point on.
pub async fn admin_approve_translation(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(translation_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let result = sqlx::query(
        r#"UPDATE work_translations
           SET status = 'approved', reviewed_by = $1, reviewed_at = NOW()
           WHERE id = $2 AND status = 'draft'"#,
    )
    .bind(admin_id)
    .bind(translation_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Translation not found or not in draft state".into(),
        ));
    }

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "approve_translation",
        "translation",
        &translation_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Translation approved", "id": translation_id}),
    ))
}

/// POST /api/admin/translations/{id}/reject — reject a draft translation.
/// Rejected rows are excluded from public serving; the original (pre-machine)
/// content is untouched.
pub async fn admin_reject_translation(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(translation_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let result = sqlx::query(
        r#"UPDATE work_translations
           SET status = 'rejected', reviewed_by = $1, reviewed_at = NOW()
           WHERE id = $2 AND status = 'draft'"#,
    )
    .bind(admin_id)
    .bind(translation_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Translation not found or not in draft state".into(),
        ));
    }

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "reject_translation",
        "translation",
        &translation_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Translation rejected", "id": translation_id}),
    ))
}

/// POST /api/admin/translations/{id}/edit — post-edit a draft translation
/// before approval. Body: `{"title": "...", "summary": "..."}` (either field
/// optional). Only drafts can be edited; approved rows are immutable.
pub async fn admin_edit_translation(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(translation_id): Path<i64>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let summary = payload
        .get("summary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if title.is_none() && summary.is_none() {
        return Err(AppError::BadRequest(
            "At least one of title or summary is required".to_string(),
        ));
    }

    let result = sqlx::query(
        r#"UPDATE work_translations
           SET title = COALESCE($1, title),
               summary = COALESCE($2, summary)
           WHERE id = $3 AND status = 'draft'"#,
    )
    .bind(title)
    .bind(summary)
    .bind(translation_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Translation not found or not in draft state".into(),
        ));
    }

    Ok(Json(
        json!({"err": 0, "msg": "Translation updated", "id": translation_id}),
    ))
}

// ═══════════════════════════════════════════════════════════════════
// F. Rating / warning verification queue
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/rating-checks — list works whose rating/warning metadata is
/// missing or unverified. Ratings are freeform tags (tag_type_id = 4),
/// archive warnings are warning tags (tag_type_id = 5). Works with no rating
/// tag or a "Not Rated" rating tag, and works that have never been verified,
/// land in the queue. `no_warnings` search relies on verified warning state.
pub async fn admin_rating_checks(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<PageParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let per_page = params.per_page.unwrap_or(20).max(1).min(100);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;

    let rows = sqlx::query_as::<
        _,
        (
            i32,
            String,
            String,
            Option<String>,
            Option<serde_json::Value>,
            Option<chrono::DateTime<chrono::Utc>>,
            String,
        ),
    >(
        r#"SELECT DISTINCT ON (w.id) w.id, w.canonical_title, w.canonical_author,
                  fi.extra_meta,
                  wrv.warnings,
                  wrv.verified_at,
                  COALESCE(wrv.rating, '')
           FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           LEFT JOIN work_rating_verifications wrv ON wrv.work_id = w.id
           WHERE wrv.work_id IS NULL
              OR wrv.rating IS NULL OR wrv.rating = '' OR wrv.rating = 'Not Rated'
           ORDER BY w.id DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"SELECT COUNT(DISTINCT w.id) FROM works w
           JOIN fic_info fi ON fi.work_id = w.id
           LEFT JOIN work_rating_verifications wrv ON wrv.work_id = w.id
           WHERE wrv.work_id IS NULL
              OR wrv.rating IS NULL OR wrv.rating = '' OR wrv.rating = 'Not Rated'"#,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": rows.into_iter().map(|(id, title, author, extra_meta, warnings, verified_at, rating)| json!({
            "work_id": id,
            "title": title,
            "author": author,
            "extra_meta": extra_meta,
            "rating": rating,
            "warnings": warnings.unwrap_or_else(|| serde_json::json!([])),
            "verified_at": verified_at.map(|t| t.to_rfc3339()),
        })).collect::<Vec<_>>(),
        "total": total.0,
        "page": params.page.unwrap_or(1),
    })))
}

/// POST /api/admin/rating-checks/{work_id}/verify — set (or update) a work's
/// verified rating + warnings. Body: `{"rating": "Explicit", "warnings":
/// ["Graphic Depictions Of Violence"]}` (either field optional). Stores the
/// audit row in work_rating_verifications and stamps works.rating_verified_at.
pub async fn admin_verify_rating(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let rating = payload
        .get("rating")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let warnings = payload
        .get("warnings")
        .filter(|v| v.is_array())
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    if rating.is_none() && warnings.as_array().map_or(true, |w| w.is_empty()) {
        return Err(AppError::BadRequest(
            "At least one of rating or warnings is required".to_string(),
        ));
    }

    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM works WHERE id = $1)")
        .bind(work_id)
        .fetch_one(&state.db)
        .await?;
    if !exists {
        return Err(AppError::NotFound("Work not found".into()));
    }

    sqlx::query(
        r#"INSERT INTO work_rating_verifications (work_id, rating, warnings, verified_by, verified_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (work_id) DO UPDATE SET
               rating = COALESCE($2, work_rating_verifications.rating),
               warnings = $3,
               verified_by = $4,
               verified_at = NOW()"#,
    )
    .bind(work_id)
    .bind(rating)
    .bind(&warnings)
    .bind(admin_id)
    .execute(&state.db)
    .await?;

    let _ = sqlx::query("UPDATE works SET rating_verified_at = NOW() WHERE id = $1")
        .bind(work_id)
        .execute(&state.db)
        .await;

    Ok(Json(
        json!({"err": 0, "msg": "Rating verified", "work_id": work_id}),
    ))
}

// ═══════════════════════════════════════════════════════════════════
// G. Character / relationship score fixing
// ═══════════════════════════════════════════════════════════════════

/// PUT /api/admin/characters/{id}/score — manually correct a mis-scored
/// character or relationship tag on a fic (scrapers order main characters by
/// score: main = 10, primary ship = 5, secondary = 1). `fic_tags.score` is
/// the column that drives `primary_tag` search. The body
/// `{"url_id": "...", "score": N}` identifies the fic + new score; the old
/// value is logged in `tag_score_fixes` for audit/revert.
pub async fn admin_fix_tag_score(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(tag_id): Path<i32>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let url_id = payload
        .get("url_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("url_id field required".to_string()))?;
    let new_score: i16 = payload
        .get("score")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16)
        .ok_or_else(|| AppError::BadRequest("score field required (integer)".to_string()))?;

    let old: Option<(i16,)> =
        sqlx::query_as("SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2")
            .bind(url_id)
            .bind(tag_id)
            .fetch_optional(&state.db)
            .await?;

    let old_score = match old {
        Some((s,)) => s,
        None => return Err(AppError::NotFound("fic_tags row not found".into())),
    };

    if old_score == new_score {
        return Ok(Json(
            json!({"err": 0, "msg": "Score unchanged", "tag_id": tag_id}),
        ));
    }

    sqlx::query("UPDATE fic_tags SET score = $1 WHERE url_id = $2 AND tag_id = $3")
        .bind(new_score)
        .bind(url_id)
        .bind(tag_id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"INSERT INTO tag_score_fixes (url_id, tag_id, old_score, new_score, fixed_by)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(old_score)
    .bind(new_score)
    .bind(admin_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "msg": "Score updated",
        "tag_id": tag_id,
        "url_id": url_id,
        "old_score": old_score,
        "new_score": new_score,
    })))
}

// ═══════════════════════════════════════════════════════════════════
// D. Platform Analytics
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/stats — platform analytics
pub async fn admin_stats(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let daily_stats = sqlx::query_as::<_, (String, i32, i32, i32, i32, i32, i64, i64)>(
        r#"SELECT to_char(date, 'YYYY-MM-DD'), total_users, new_users, total_works,
                  new_works, manual_uploads, epubs_downloaded, words_read
           FROM admin_daily_stats ORDER BY date DESC LIMIT 30"#,
    )
    .fetch_all(&state.db)
    .await?;

    // Live totals
    let totals: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM users), (SELECT COUNT(*) FROM works), (SELECT COUNT(*) FROM bookmarks), (SELECT COUNT(*) FROM request_log)"
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "daily": daily_stats.into_iter().map(|(d, tu, nu, tw, nw, mu, ed, wr)| json!({
            "date": d, "total_users": tu, "new_users": nu, "total_works": tw,
            "new_works": nw, "manual_uploads": mu, "epubs_downloaded": ed, "words_read": wr,
        })).collect::<Vec<_>>(),
        "totals": {
            "users": totals.0,
            "works": totals.1,
            "bookmarks": totals.2,
            "requests": totals.3,
        },
    })))
}

// ═══════════════════════════════════════════════════════════════════
// B. Behavioral Security — unusual-behavior leaderboard (Zero-PII)
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/bots — top clients by unusual behavior.
///
/// Zero-PII: rows are keyed by anonymized client_id (never raw IPs). Signals:
/// requests/hour, downloads, export_ratio (downloads/requests — a mirror bot
/// approaches 1.0), failed_auths (credential stuffing), and a composite
/// `bot_score` that ranks clients for admin review. Admin sees *what* looks
/// bot-like and can act (shadowban), not *who* the user is.
#[derive(Debug, Deserialize)]
pub struct BotParams {
    pub window_hours: Option<i64>,
    pub min_requests: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn admin_bots(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<BotParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let window_hours = params.window_hours.unwrap_or(24).max(1).min(168);
    let min_requests = params.min_requests.unwrap_or(10);
    let limit = params.limit.unwrap_or(50).max(1).min(200);

    // Aggregate per client over the window. Only expose client_id + signals.
    let rows = sqlx::query_as::<_, (String, i64, i64, f64, i64, i64)>(
        r#"
        SELECT
            client_id,
            SUM(requests)::bigint AS total_requests,
            SUM(downloads)::bigint AS total_downloads,
            CASE WHEN SUM(requests) = 0 THEN 0
                 ELSE (SUM(downloads)::float / SUM(requests)::float)
            END AS export_ratio,
            SUM(failed_auths)::bigint AS total_failed_auths,
            COUNT(*)::bigint AS windows
        FROM bot_scores
        WHERE window_start >= now() - ($1 * interval '1 hour')
          AND client_id <> 'anon'
        GROUP BY client_id
        HAVING SUM(requests) >= $2
        ORDER BY
            (CASE WHEN SUM(requests) = 0 THEN 0
                  ELSE (SUM(downloads)::float / SUM(requests)::float) END
             + CASE WHEN SUM(failed_auths) > 0 THEN 0.5 ELSE 0 END) DESC
        LIMIT $3
        "#,
    )
    .bind(window_hours)
    .bind(min_requests)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    let leaderboard = rows
        .into_iter()
        .map(|(cid, reqs, dl, ratio, fauths, wins)| {
            // Composite bot score 0..1: heavy on download ratio + failed auths.
            let bot_score = (ratio * 0.7 + if fauths > 0 { 0.3 } else { 0.0 }).min(1.0);
            let flags: Vec<&str> = {
                let mut v = vec![];
                if ratio > 0.9 {
                    v.push("mirror");
                }
                if fauths > 3 {
                    v.push("stuffing");
                }
                if reqs > 500 && wins <= 2 {
                    v.push("burst");
                }
                v
            };
            json!({
                "client_id": cid,
                "requests": reqs,
                "downloads": dl,
                "export_ratio": ratio,
                "failed_auths": fauths,
                "windows": wins,
                "bot_score": bot_score,
                "flags": flags,
                "shadowbanned": state.rate_limiter.is_shadowbanned(&cid),
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(json!({
        "err": 0,
        "window_hours": window_hours,
        "clients": leaderboard,
    })))
}

/// POST /api/admin/bots/{client_id}/shadowban — add a client to the Redis
/// shadowban set for the configured TTL (friction: stricter download bucket,
/// never a hard block). Zero-PII: only the anonymized client_id is touched.
pub async fn admin_bot_shadowban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if client_id.is_empty() || client_id == "anon" {
        return Err(AppError::BadRequest("invalid client_id".to_string()));
    }

    state
        .rate_limiter
        .shadowban(&client_id, state.config.rl_shadowban_ttl);
    Ok(Json(
        json!({ "err": 0, "client_id": client_id, "shadowbanned": true }),
    ))
}

/// POST /api/admin/bots/{client_id}/unshadowban — remove a client from the
/// Redis shadowban set (admin override).
pub async fn admin_bot_unshadowban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if client_id.is_empty() || client_id == "anon" {
        return Err(AppError::BadRequest("invalid client_id".to_string()));
    }

    state.rate_limiter.unshadowban(&client_id);
    Ok(Json(
        json!({ "err": 0, "client_id": client_id, "shadowbanned": false }),
    ))
}

// ═══════════════════════════════════════════════════════════════════
// Search Analytics — zero-result queries, trope popularity, volume
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/search-analytics — aggregate search analytics.
///
/// Zero-PII: only aggregate counts are returned (never client_id -> query
/// Three views over the last 7 days:
///   * zero_result_queries — top 50 queries that returned no results (search
///     quality blind spots), by frequency
///   * search_volume — queries per hour over the last 24h
pub async fn admin_search_analytics(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Top zero-result queries (last 7 days), by frequency.
    let zero_result = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT query, COUNT(*) AS cnt
           FROM search_queries
           WHERE total_results = 0
             AND ts >= now() - interval '7 days'
           GROUP BY query
           ORDER BY cnt DESC
           LIMIT 50"#,
    )
    .fetch_all(&state.db)
    .await?;

    // Search volume per hour over the last 24h.
    let search_volume = sqlx::query_as::<_, (String, i64)>(
        r#"SELECT to_char(date_trunc('hour', ts), 'YYYY-MM-DD"T"HH24:00:00Z') AS hour,
                  COUNT(*) AS cnt
           FROM search_queries
           WHERE ts >= now() - interval '24 hours'
           GROUP BY date_trunc('hour', ts)
           ORDER BY hour ASC"#,
    )
    .fetch_all(&state.db)
    .await?;

    // Search → export conversion (last 7 days). A search converts when the
    // same client performs an export (request_log with an export_file_name)
    // after searching. Zero-PII: client_id is only used to join, never
    // returned.
    let conversion = sqlx::query_as::<_, (i64, i64, i64)>(
        r#"WITH searching AS (
             SELECT DISTINCT client_id FROM search_queries
             WHERE ts >= now() - interval '7 days' AND client_id IS NOT NULL
           ),
           exporting AS (
             SELECT DISTINCT client_id FROM request_log
             WHERE created >= now() - interval '7 days'
               AND client_id IS NOT NULL
               AND export_file_name IS NOT NULL
           )
           SELECT
             (SELECT COUNT(*) FROM searching) AS searchers,
             (SELECT COUNT(*) FROM exporting) AS exporters,
             (SELECT COUNT(*) FROM searching s JOIN exporting e USING (client_id)) AS converted"#,
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "zero_result_queries": zero_result.into_iter().map(|(q, cnt)| json!({
            "query": q, "count": cnt,
        })).collect::<Vec<_>>(),
        "search_volume": search_volume.into_iter().map(|(h, cnt)| json!({
            "hour": h, "count": cnt,
        })).collect::<Vec<_>>(),
        "conversion": {
            "searchers_7d": conversion.0,
            "exporters_7d": conversion.1,
            "search_to_export_7d": conversion.2,
        },
    })))
}

// ═══════════════════════════════════════════════════════════════════
// C. Realtime pulse (cheap polling aggregates for the Command Center)
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/realtime — cheap live aggregates for the Command Center.
/// Polls lightweight COUNT(*) windows on request_log + bot_scores (indexed by
/// created / window_start) plus a couple of Redis counters. Zero-PII: counts
/// only, never rows. Avoids SSE/WebSocket infra; the UI polls every ~5s.
pub async fn admin_realtime(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Exports + searches in the last 5 minutes (request_log etypes).
    let (exports_5m, searches_5m): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE etype = 'epub')::bigint,
            COUNT(*) FILTER (WHERE etype = 'search')::bigint
        FROM request_log
        WHERE created >= now() - interval '5 minutes'
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    // Flagged clients right now (mirror-ish or stuffing in the last hour).
    let (flagged_clients, active_ips): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE (export_ratio > 0.9 OR failed_auths > 3))::bigint,
            COUNT(DISTINCT ip)::bigint
        FROM bot_scores
        WHERE window_start >= now() - interval '1 hour'
        "#,
    )
    .fetch_one(&state.db)
    .await?;

    // Redis counters (best-effort). Use the DEDICATED health_redis connection
    // (never shared state.redis): the bookmark-import worker parks an
    // unbounded BRPOP on state.redis, which would make PING time out and
    // report redis:false even when Redis is healthy (see health.rs).
    let mut redis_ok = false;
    let mut redis_mem: i64 = 0;
    if let Ok(()) = redis::cmd("PING")
        .query_async::<String>(&mut state.health_redis.clone())
        .await
        .map(|_| ())
    {
        redis_ok = true;
        redis_mem = redis::cmd("INFO")
            .query_async::<String>(&mut state.health_redis.clone())
            .await
            .ok()
            .and_then(|info| {
                info.lines()
                    .find(|l| l.starts_with("used_memory:"))
                    .and_then(|l| l.split(':').nth(1))
                    .and_then(|v| v.trim().parse().ok())
            })
            .unwrap_or(0);
    }

    Ok(Json(json!({
        "err": 0,
        "exports_5m": exports_5m,
        "searches_5m": searches_5m,
        "flagged_clients_1h": flagged_clients,
        "active_ips_1h": active_ips,
        "redis": { "ok": redis_ok, "used_memory_bytes": redis_mem },
        "ts": chrono::Utc::now().to_rfc3339(),
    })))
}

// ═══════════════════════════════════════════════════════════════════
// D. Roadmap Consensus (Pillar 7) — leaderboard + controversy
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/roadmap-consensus — the ranked cluster leaderboard plus the
/// controversy scatter data. Zero-PII: cluster texts + aggregates only (no
/// user/client mapping).
pub async fn admin_roadmap_consensus(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Leaderboard: ranked by Elo, with vote counters + suggestion count.
    let leaderboard = sqlx::query_as::<_, (i32, String, f64, i32, i32, i32, i64, String)>(
        r#"
        SELECT c.id, c.representative_text, c.elo_rating::float8, c.matches_played,
               c.times_picked_best, c.times_picked_worst, COUNT(s.id)::bigint AS suggestions,
               c.status
        FROM feature_clusters c
        LEFT JOIN feature_suggestions s ON s.cluster_id = c.id
        GROUP BY c.id
        ORDER BY c.elo_rating DESC
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
        WHERE status = 'open'
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

// ═══════════════════════════════════════════════════════════════════
// F. Comment moderation triage queue
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/moderation/comments — pending comment triage rows for
/// curators to review.
///
/// Returns every comment the LLM triage flagged as `non-constructive`,
/// `toxic`, or `spam`, joined with the comment body + the work's title so a
/// curator can decide without opening another page. Triage is advisory: this
/// queue exists for humans; nothing is auto-hidden.
pub async fn moderation_comments(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let rows = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            String,
            f32,
            Option<i32>,
            Option<String>,
            Option<i32>,
        ),
    >(
        r#"SELECT ct.comment_id::bigint, c.body, ct.category, ct.reason, ct.confidence,
                  c.work_id, w.canonical_title, c.user_id
           FROM comment_triage ct
           JOIN comments c ON c.id = ct.comment_id
           LEFT JOIN works w ON w.id = c.work_id
           WHERE ct.category IN ('toxic', 'spam', 'non-constructive')
           ORDER BY ct.created_at DESC
           LIMIT 200"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "items": rows.into_iter().map(|(comment_id, body, category, reason, confidence, work_id, title, user_id)| json!({
            "comment_id": comment_id,
            "body": body,
            "category": category,
            "reason": reason,
            "confidence": confidence,
            "work_id": work_id,
            "title": title,
            "user_id": user_id,
        })).collect::<Vec<_>>(),
    })))
}

// ═══════════════════════════════════════════════════════════════════
// G. Blacklist management (fic + author)
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct FicBlacklistBody {
    /// fic_info.id of the fic to blacklist.
    pub url_id: String,
    /// Blacklist reason code (see export.rs: 5/7/8 = hard block, 6 = greylist).
    #[serde(default)]
    pub reason: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AuthorBlacklistBody {
    /// fic_info.source_id of the source platform (e.g. 10 = ao3).
    pub source_id: i64,
    /// fic_info.author_id of the author on that source.
    pub author_id: i64,
    /// Blacklist reason code (any reason blocks the author, see export.rs).
    #[serde(default)]
    pub reason: Option<i32>,
}

/// POST /api/admin/blacklist/fic — add a fic to the blacklist (upsert).
pub async fn blacklist_fic(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<FicBlacklistBody>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    if body.url_id.trim().is_empty() {
        return Err(AppError::BadRequest("url_id required".to_string()));
    }
    let reason = body.reason.unwrap_or(5);

    sqlx::query(
        r#"INSERT INTO fic_blacklist (url_id, reason)
           VALUES ($1, $2)
           ON CONFLICT (url_id, reason) DO UPDATE SET updated = NOW()"#,
    )
    .bind(&body.url_id)
    .bind(reason)
    .execute(&state.db)
    .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "blacklist_fic",
        "fic",
        &body.url_id,
        vec![("reason", serde_json::json!(reason))],
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "fic blacklisted", "url_id": body.url_id, "reason": reason}),
    ))
}

/// POST /api/admin/blacklist/author — add an author to the blacklist (upsert).
pub async fn blacklist_author(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<AuthorBlacklistBody>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let reason = body.reason.unwrap_or(5);

    sqlx::query(
        r#"INSERT INTO author_blacklist (source_id, author_id, reason)
           VALUES ($1, $2, $3)
           ON CONFLICT (source_id, author_id, reason) DO UPDATE SET updated = NOW()"#,
    )
    .bind(body.source_id)
    .bind(body.author_id)
    .bind(reason)
    .execute(&state.db)
    .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "blacklist_author",
        "author",
        &body.author_id.to_string(),
        vec![
            ("source_id", serde_json::json!(body.source_id)),
            ("reason", serde_json::json!(reason)),
        ],
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "author blacklisted", "source_id": body.source_id, "author_id": body.author_id, "reason": reason}),
    ))
}

/// GET /api/admin/blacklist — list all blacklist entries (fics + authors).
pub async fn list_blacklist(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let fics = sqlx::query_as::<_, (String, i32, chrono::NaiveDateTime)>(
        "SELECT url_id, reason, created::timestamp FROM fic_blacklist ORDER BY created DESC LIMIT 500",
    )
    .fetch_all(&state.db)
    .await?;

    let authors = sqlx::query_as::<_, (i64, i64, i32, chrono::NaiveDateTime)>(
        "SELECT source_id, author_id, reason, created::timestamp FROM author_blacklist ORDER BY created DESC LIMIT 500",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "fics": fics.into_iter().map(|(url_id, reason, created)| json!({
            "url_id": url_id, "reason": reason,
            "created": created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
        "authors": authors.into_iter().map(|(source_id, author_id, reason, created)| json!({
            "source_id": source_id, "author_id": author_id, "reason": reason,
            "created": created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        })).collect::<Vec<_>>(),
    })))
}

/// POST /api/admin/moderation/comments/{id}/hide — hide a comment (curator
/// action from the triage queue). Reuses the same `is_hidden` flip the
/// PATCH /api/comment/{id}/hide endpoint performs, but as an admin action
/// (role >= 10, no body needed — always hides).
pub async fn admin_hide_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(comment_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let result = sqlx::query("UPDATE comments SET is_hidden = TRUE WHERE id = $1")
        .bind(comment_id)
        .execute(&state.db)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Comment not found".into()));
    }

    // Clear the triage row so the item leaves the review queue.
    let _ = sqlx::query("DELETE FROM comment_triage WHERE comment_id = $1")
        .bind(comment_id)
        .execute(&state.db)
        .await;

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "hide_comment",
        "comment",
        &comment_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Comment hidden", "comment_id": comment_id}),
    ))
}

/// POST /api/admin/moderation/comments/{id}/delete — soft-delete a comment
/// (admin action from the triage queue). Reuses the same `deleted_at` stamp
/// the DELETE /api/comment/{id} endpoint performs, minus ownership checks.
pub async fn admin_delete_comment(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(comment_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let result =
        sqlx::query("UPDATE comments SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
            .bind(comment_id)
            .execute(&state.db)
            .await?;
    if result.rows_affected() == 0 {
        // Either missing, or already deleted — treat both as not found.
        return Err(AppError::NotFound(
            "Comment not found or already deleted".into(),
        ));
    }

    // Clear the triage row so the item leaves the review queue.
    let _ = sqlx::query("DELETE FROM comment_triage WHERE comment_id = $1")
        .bind(comment_id)
        .execute(&state.db)
        .await;

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "delete_comment",
        "comment",
        &comment_id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(
        json!({"err": 0, "msg": "Comment deleted", "comment_id": comment_id}),
    ))
}

// ═══════════════════════════════════════════════════════════════════
// H. Metadata correction — admin-correctable canonical metadata
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct WorkMetadataBody {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    /// Story status: 'ongoing' | 'complete'.
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// PUT /api/admin/works/{id}/metadata — correct the canonical metadata of a
/// work (role >= 10). Updates `works.canonical_title`, `works.canonical_author`,
/// `works.description` and, when the work has a default source, its
/// `fic_info.title/author/status/description` so public readers (who render
/// fic_info) see the correction too. Partial update: only provided fields are
/// touched. Title, when present, must be non-empty.
pub async fn update_work_metadata(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(work_id): Path<i32>,
    Json(body): Json<WorkMetadataBody>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Validate: at least one field must be present, and a provided title
    // must not be empty.
    let has_any = body.title.is_some()
        || body.author.is_some()
        || body.status.is_some()
        || body.description.is_some();
    if !has_any {
        return Err(AppError::BadRequest(
            "No metadata fields provided".to_string(),
        ));
    }
    if let Some(ref t) = body.title {
        if t.trim().is_empty() {
            return Err(AppError::BadRequest("title must not be empty".to_string()));
        }
    }

    // The work must exist (and, for the fic_info sync, have a default source).
    let existing: Option<(i32, Option<String>)> =
        sqlx::query_as("SELECT id, default_source_id FROM works WHERE id = $1")
            .bind(work_id)
            .fetch_optional(&state.db)
            .await?;
    let Some((_, default_source_id)) = existing else {
        return Err(AppError::NotFound("Work not found".into()));
    };

    // 1. Update the canonical works row (partial via COALESCE).
    let updated = sqlx::query_as::<_, (String, String, String, chrono::DateTime<Utc>)>(
        r#"UPDATE works SET
               canonical_title = COALESCE($2, canonical_title),
               canonical_author = COALESCE($3, canonical_author),
               description = COALESCE($4, description),
               updated_at = NOW()
           WHERE id = $1
           RETURNING canonical_title, canonical_author, description, updated_at"#,
    )
    .bind(work_id)
    .bind(body.title.as_deref())
    .bind(body.author.as_deref())
    .bind(body.description.as_deref())
    .fetch_one(&state.db)
    .await?;

    // 2. Sync the default source (fic_info) so public pages show the
    //    correction immediately. Best-effort: the default source may be a
    //    legacy row whose fic_info was deleted.
    if let Some(ref url_id) = default_source_id {
        let _ = sqlx::query(
            r#"UPDATE fic_info SET
                   title = COALESCE($2, title),
                   author = COALESCE($3, author),
                   status = COALESCE($4, status),
                   description = COALESCE($5, description),
                   updated = NOW()
               WHERE id = $1"#,
        )
        .bind(url_id)
        .bind(body.title.as_deref())
        .bind(body.author.as_deref())
        .bind(body.status.as_deref())
        .bind(body.description.as_deref())
        .execute(&state.db)
        .await;
    }

    let (canonical_title, canonical_author, description, updated_at) = updated;
    Ok(Json(json!({
        "err": 0,
        "work": {
            "id": work_id,
            "canonical_title": canonical_title,
            "canonical_author": canonical_author,
            "description": description,
            "updated_at": updated_at.to_rfc3339(),
        }
    })))
}

// ═══════════════════════════════════════════════════════════════════
// Content scan (body sanity + warning verification)
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/content-scan — list scan results, optionally filtered by
/// review_status (pending/confirmed/dismissed) or classification.
pub async fn list_content_scan(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<ContentScanParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let per_page = params.per_page.unwrap_or(50).max(1).min(200);
    let offset = ((params.page.unwrap_or(1).max(1)) - 1) * per_page;
    let status_filter = params
        .review_status
        .unwrap_or_else(|| "pending".to_string());
    let class_filter = params.classification.unwrap_or_default();

    // Static SQL with a nullable optional filter (empty string = no filter)
    // avoids sqlx's dynamic-SQL audit lint.
    let rows = sqlx::query(
        "SELECT cs.url_id, cs.classification, cs.confidence, cs.detected_warnings, \
         cs.reason, cs.review_status, cs.scanned_at \
         FROM content_scan cs \
         WHERE cs.review_status = $1 AND ($2 = '' OR cs.classification = $2) \
         ORDER BY cs.scanned_at DESC LIMIT $3 OFFSET $4",
    )
    .bind(&status_filter)
    .bind(&class_filter)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            let scanned_at: chrono::DateTime<chrono::Utc> = r.get("scanned_at");
            json!({
                "url_id": r.get::<String, _>("url_id"),
                "classification": r.get::<String, _>("classification"),
                "confidence": r.get::<f32, _>("confidence"),
                "detected_warnings": r.get::<String, _>("detected_warnings"),
                "reason": r.get::<String, _>("reason"),
                "review_status": r.get::<String, _>("review_status"),
                "scanned_at": scanned_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(
        json!({ "err": 0, "items": items, "total": items.len() }),
    ))
}

/// POST /api/admin/content-scan/run — run a batch scan over the body cache.
/// Optional `limit` caps how many bodies are scanned this run.
pub async fn run_content_scan(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let limit = body
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);

    let (scanned, failed) =
        crate::services::content_scan::scan_cache(&state.db, &state.ollama, &state.config, limit)
            .await;

    Ok(Json(json!({
        "err": 0,
        "scanned": scanned,
        "failed": failed,
    })))
}

/// POST /api/admin/content-scan/{url_id}/review — mark a scan result
/// confirmed or dismissed (curator disposition).
pub async fn review_content_scan(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(url_id): Path<String>,
    Json(body): Json<ContentScanReviewBody>,
) -> Result<Json<Value>, AppError> {
    // Curators (role >= 5) and admins (role >= 10) can review
    if user.trust_level < 3 {
        return Err(AppError::Forbidden("Curator access required".into()));
    }
    let action = body.action.as_str();
    if action != "confirmed" && action != "dismissed" {
        return Err(AppError::BadRequest(
            "action must be 'confirmed' or 'dismissed'".to_string(),
        ));
    }

    // On dismiss, clear the deletion schedule (curator says it's fine)
    // On confirm (noise confirmed), clear schedule too (we'll handle deletion now)
    sqlx::query(
        "UPDATE content_scan SET review_status = $1, reviewed_by = $2, reviewed_at = NOW(), \
         deletion_scheduled_at = NULL WHERE url_id = $3",
    )
    .bind(action)
    .bind(user.user_id)
    .bind(&url_id)
    .execute(&state.db)
    .await?;

    // If confirmed as noise, delete the cached body
    if action == "confirmed" {
        let _ = crate::body_cache::delete_body(&state.config, &url_id);
    }

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "content_scan_review",
        "work",
        &url_id,
        json!({ "action": action }),
    )
    .await;

    Ok(Json(
        json!({ "err": 0, "url_id": url_id, "status": action }),
    ))
}

#[derive(Debug, serde::Deserialize)]
pub struct ContentScanParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub review_status: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ContentScanReviewBody {
    pub action: String,
}

// ═══════════════════════════════════════════════════════════════════
// Zero-result search mining (demand signals → acquisition list)
// ═══════════════════════════════════════════════════════════════════

/// GET /api/admin/search-mining — cluster zero-hit search queries into
/// acquisition themes via the LLM. Falls back to a plain listing when
/// Ollama is unavailable.
pub async fn search_mining(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<SearchMiningParams>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let days = params.days.unwrap_or(30).max(1).min(365);
    let limit = params.limit.unwrap_or(50).max(1).min(200);

    let (themes, raw) = crate::services::search_mining::mine(
        &state.db,
        &state.ollama,
        &state.config,
        days,
        limit as usize,
    )
    .await;

    Ok(Json(json!({
        "err": 0,
        "themes": themes.iter().map(|t| json!({
            "theme": t.theme,
            "suggestion": t.suggestion,
            "queries": serde_json::to_value(&t.queries).unwrap_or_else(|_| json!([])),
            "count": t.count,
        })).collect::<Vec<_>>(),
        "raw_queries": raw.iter().map(|(q, c)| json!({ "query": q, "count": c })).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct SearchMiningParams {
    pub days: Option<i32>,
    pub limit: Option<i32>,
}

/// POST /api/admin/dedupe/embeddings — find works with embedding similarity
/// above threshold. Auto-merges high-confidence (≥0.98), creates proposals
/// for lower confidence.
pub async fn run_embedding_dedupe(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let threshold = body
        .get("threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.95);
    let limit = body
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(50);
    let proposer_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let (auto_merged, proposed, examined) =
        crate::services::embedding_dedupe::run_dedupe(&state.db, proposer_id, threshold, limit)
            .await;

    Ok(Json(json!({
        "err": 0,
        "auto_merged": auto_merged,
        "proposed": proposed,
        "examined": examined,
        "threshold": threshold,
    })))
}

/// Approve an immutable chapter translation version.
pub async fn admin_approve_chapter_translation_version(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let result = sqlx::query("UPDATE chapter_translation_versions SET status='approved', reviewed_by=$1, reviewed_at=NOW() WHERE id=$2 AND status='draft'").bind(admin_id).bind(id).execute(&state.db).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Version not found or not in draft state".into(),
        ));
    }
    Ok(Json(
        json!({"err":0,"msg":"Chapter translation version approved","id":id}),
    ))
}

/// Reject an immutable chapter translation version.
pub async fn admin_reject_chapter_translation_version(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let admin_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let result = sqlx::query("UPDATE chapter_translation_versions SET status='rejected', reviewed_by=$1, reviewed_at=NOW() WHERE id=$2 AND status='draft'").bind(admin_id).bind(id).execute(&state.db).await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Version not found or not in draft state".into(),
        ));
    }
    Ok(Json(
        json!({"err":0,"msg":"Chapter translation version rejected","id":id}),
    ))
}

/// Backfill tags for existing works.
/// Iterates over works with fic_info entries and re-scrapes tags from the source.
pub async fn admin_backfill_tags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Get works that have fic_info but no fic_tags
    let works_without_tags: Vec<(String, i32)> = sqlx::query_as(
        r#"SELECT fi.id, fi.work_id
           FROM fic_info fi
           LEFT JOIN fic_tags ft ON ft.url_id = fi.id
           WHERE ft.url_id IS NULL
           AND fi.work_id IS NOT NULL
           LIMIT 100"#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut processed = 0;
    let mut errors = 0;

    for (url_id, _work_id) in &works_without_tags {
        // Re-scrape tags for this fic
        let query = &state
            .scraper_registry
            .lookup(&state.http_client, &url_id, None)
            .await;
        match query {
            Ok(_meta) => {
                // Tags would have been extracted during the lookup via extract_tags
                processed += 1;
            }
            Err(_) => {
                errors += 1;
            }
        }
    }

    Ok(Json(json!({
        "err": 0,
        "msg": "Tag backfill complete",
        "processed": processed,
        "errors": errors,
        "remaining": works_without_tags.len() - processed,
    })))
}

/// Backfill fic bodies (chapters) for works that have metadata but no cached content.
pub async fn admin_backfill_bodies(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    // Get works that have fic_info but may not have cached body content
    let works: Vec<(String, String, i32)> = sqlx::query_as(
        r#"SELECT fi.id, fi.source, COALESCE(fi.work_id, 0) as work_id
           FROM fic_info fi
           WHERE fi.work_id IS NOT NULL
           LIMIT 100"#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (url_id, _source, _work_id) in &works {
        // Check if body already cached
        if crate::body_cache::load_body(&state.config, url_id).is_some() {
            skipped += 1;
            continue;
        }

        // Re-scrape to get chapter content
        match state
            .scraper_registry
            .lookup(&state.http_client, url_id, None)
            .await
        {
            Ok(meta) => {
                match state
                    .scraper_registry
                    .fetch_chapters(&state.http_client, &meta)
                    .await
                {
                    Ok(chapters) => {
                        if !chapters.is_empty() {
                            let _ = crate::body_cache::save_body(
                                &state.config,
                                url_id,
                                &chapters,
                                None,
                                0,
                            );
                            processed += 1;
                        } else {
                            skipped += 1;
                        }
                    }
                    Err(_) => {
                        errors += 1;
                    }
                }
            }
            Err(_) => {
                errors += 1;
            }
        }
    }

    Ok(Json(json!({
        "err": 0,
        "msg": "Body backfill complete",
        "processed": processed,
        "skipped": skipped,
        "errors": errors,
    })))
}
