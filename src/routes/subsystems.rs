//! F7 entry-curation + site subsystems (SPEC-COMMUNITY-PLATFORM §8 milestone
//! 6-7, THREADLIGHT-FEATURES §2.1 rows 10-13).
//!
//! * `GET /api/site` — public site info: name, description, registration
//!   mode (`open` | `invite` | `application`), version. Values come from the
//!   `site_settings` table (seeded by migration 046, admin-overridable),
//!   falling back to compiled defaults; `registration_mode` is driven by the
//!   `REGISTRATION_MODE` env var (default `open`).
//! * `POST /api/admin/invites` (role ≥ 5) — generate a single-use invite
//!   code. Only allowed when `REGISTRATION_MODE=invite` (400 otherwise).
//! * `GET /api/admin/invites` (role ≥ 5) — list invites + usage.
//! * `POST /api/registration-applications` — any logged-in user applies to
//!   join `{reason}`. One pending application per user (409 on a second).
//! * `GET /api/admin/registration-applications` (role ≥ 5) — list
//!   applications (filter by status).
//! * `POST /api/admin/registration-applications/{id}/review` (role ≥ 5) —
//!   approve/reject an application; records a `registration_app_review`
//!   modlog entry.
//! * `POST /api/blocks {user_id}` — block a user (self-block 400,
//!   idempotent). `DELETE /api/blocks/{user_id}` — unblock (idempotent).
//!   `GET /api/blocks` — list blocked user ids + usernames.
//!
//! Registration-mode behavior (documented choice): `REGISTRATION_MODE`
//!   * `open` (default) — registration is unrestricted, `invite_code` is
//!     optional (a valid unused code, when supplied, is still consumed).
//!   * `invite` — `POST /api/auth/register` REQUIRES a valid unused
//!     unexpired `invite_code`; missing/invalid/used/expired → 403. The code
//!     is atomically marked used on success.
//!   * `application` — open registration is disabled (403 with a pointer to
//!     `/api/registration-applications`). Admins review applications
//!     (`POST .../review`); on approval the applicant's existing account is
//!     enabled by setting `users.role = 1` (trusted) — the account exists
//!     from the moment the user registers, but is inert (role 0, no forum
//!     write access) until a curator approves. Rejection leaves the account
//!     at role 0; the same user may re-apply (a new application is allowed
//!     once the previous one is no longer pending).
//!
//! This keeps registration one endpoint (no dual signup flows), makes
//! approval a single role flip, and stays minimal for the F7 backend cut.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Compiled defaults for the site_settings keys (migration 046 seeds the
/// same values into the table; these cover DBs where the migration hasn't
/// been applied yet).
const SITE_NAME_DEFAULT: &str = "FicNexus";
const SITE_DESCRIPTION_DEFAULT: &str = "Fanfiction archive and download platform.";
const SITE_VERSION_DEFAULT: &str = env!("CARGO_PKG_VERSION");

// ── Helpers ────────────────────────────────────────────────────────────────

fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}

/// Require curator (role ≥ 5); 403 otherwise (also requires login → 401).
fn require_mod(auth: &AuthUser) -> Result<i32, AppError> {
    let uid = require_user(auth)?;
    if auth.trust_level < 3 {
        return Err(AppError::Forbidden("Moderator access required".to_string()));
    }
    Ok(uid)
}

/// Read `REGISTRATION_MODE` env: `open` (default), `invite`, `application`.
/// Anything unrecognized falls back to `open`.
pub fn registration_mode() -> &'static str {
    match std::env::var("REGISTRATION_MODE").as_deref() {
        Ok("invite") => "invite",
        Ok("application") => "application",
        _ => "open",
    }
}

/// Read a site_settings value, falling back to the compiled default.
async fn site_setting(db: &sqlx::PgPool, key: &str, default: &str) -> String {
    sqlx::query_scalar("SELECT value FROM site_settings WHERE key = $1")
        .bind(key)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .filter(|v: &String| !v.is_empty())
        .unwrap_or_else(|| default.to_string())
}

// ── GET /api/site ──────────────────────────────────────────────────────────

/// Public site info (no auth required).
pub async fn site_info(State(state): State<Arc<AppState>>) -> Result<Json<Value>, AppError> {
    let name = site_setting(&state.db, "site.name", SITE_NAME_DEFAULT).await;
    let description = site_setting(&state.db, "site.description", SITE_DESCRIPTION_DEFAULT).await;
    let version = site_setting(&state.db, "site.version", SITE_VERSION_DEFAULT).await;
    let mode = registration_mode().to_string();
    Ok(Json(json!({
        "err": 0,
        "site": {
            "name": name,
            "description": description,
            "registration_mode": mode,
            "version": version,
        }
    })))
}

// ── Invites ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateInviteBody {
    /// Optional note stored in the modlog entry, not on the invite row.
    #[serde(default)]
    pub note: Option<String>,
    /// Optional RFC3339 expiry for the invite.
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// POST /api/admin/invites — generate a single-use invite code (role ≥ 5).
/// Only meaningful when REGISTRATION_MODE=invite (400 otherwise).
pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateInviteBody>,
) -> Result<Json<Value>, AppError> {
    let actor_id = require_mod(&auth)?;

    if registration_mode() != "invite" {
        return Err(AppError::BadRequest(
            "invites are only usable when REGISTRATION_MODE=invite".to_string(),
        ));
    }

    let expires_at = match body.expires_at.as_deref() {
        None | Some("") => None,
        Some(raw) => {
            let parsed = DateTime::parse_from_rfc3339(raw).map_err(|_| {
                AppError::BadRequest("expires_at must be an RFC3339 timestamp".to_string())
            })?;
            Some(parsed.with_timezone(&Utc))
        }
    };

    // 12 chars of base62 ≈ 71 bits of entropy — unguessable but copy-friendly.
    const CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let code: String = {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..12)
            .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
            .collect()
    };

    let invite_id: i64 = sqlx::query_scalar(
        "INSERT INTO user_invites (code, created_by, expires_at) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&code)
    .bind(actor_id)
    .bind(expires_at)
    .fetch_one(&state.db)
    .await?;

    let mut extra = vec![("invite_id", json!(invite_id)), ("code", json!(code))];
    if let Some(note) = body.note.as_deref().filter(|n| !n.trim().is_empty()) {
        extra.push(("note", json!(note.trim())));
    }
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        "invite_create",
        "invite",
        &invite_id.to_string(),
        extra,
    )
    .await;

    Ok(Json(json!({
        "err": 0,
        "id": invite_id,
        "code": code,
        "created_at": Utc::now().to_rfc3339(),
        "expires_at": expires_at.map(|d| d.to_rfc3339()),
    })))
}

/// GET /api/admin/invites — list invites + usage (role ≥ 5).
pub async fn list_invites(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    require_mod(&auth)?;

    let rows = sqlx::query_as::<
        _,
        (
            i64,
            String,
            i32,
            DateTime<Utc>,
            Option<DateTime<Utc>>,
            Option<i32>,
            Option<String>,
            Option<DateTime<Utc>>,
        ),
    >(
        r#"SELECT i.id, i.code, i.created_by, i.created_at, i.expires_at,
                  i.used_by, u.username, i.used_at
           FROM user_invites i
           LEFT JOIN users u ON u.id = i.used_by
           ORDER BY i.created_at DESC
           LIMIT 200"#,
    )
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(id, code, created_by, created_at, expires_at, used_by, used_username, used_at)| {
                json!({
                    "id": id,
                    "code": code,
                    "created_by": created_by,
                    "created_at": created_at.to_rfc3339(),
                    "expires_at": expires_at.map(|d| d.to_rfc3339()),
                    "used_by": used_by,
                    "used_username": used_username,
                    "used_at": used_at.map(|d| d.to_rfc3339()),
                })
            },
        )
        .collect();

    Ok(Json(json!({ "err": 0, "items": items })))
}

/// Validate an invite code for registration. Returns Ok(()) when the code is
/// valid (exists, unused, unexpired) — the caller then marks it used. Used
/// by the register endpoint when REGISTRATION_MODE=invite.
pub async fn validate_invite_code(db: &sqlx::PgPool, code: &str) -> Result<(), AppError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(AppError::Forbidden(
            "Registration is by invitation only — an invite code is required".to_string(),
        ));
    }
    let row = sqlx::query_as::<_, (Option<i32>, Option<DateTime<Utc>>)>(
        "SELECT used_by, expires_at FROM user_invites WHERE code = $1",
    )
    .bind(code)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    match row {
        None => Err(AppError::Forbidden("Invalid invite code".to_string())),
        Some((used_by, expires_at)) => {
            if used_by.is_some() {
                return Err(AppError::Forbidden("Invite code already used".to_string()));
            }
            if let Some(exp) = expires_at {
                if exp < Utc::now() {
                    return Err(AppError::Forbidden("Invite code has expired".to_string()));
                }
            }
            Ok(())
        }
    }
}

/// Atomically consume an invite code for `user_id` (idempotent — a second
/// call for the same user is a no-op; a different user on an already-used
/// code is a no-op too, and the earlier validator already rejected it).
pub async fn consume_invite_code(db: &sqlx::PgPool, code: &str, user_id: i32) {
    let res = sqlx::query(
        "UPDATE user_invites SET used_by = $1, used_at = NOW()
         WHERE code = $2 AND used_by IS NULL",
    )
    .bind(user_id)
    .bind(code.trim())
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!("invite consume failed for code {code}: {e}");
    }
}

// ── Registration applications ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ApplyBody {
    pub reason: String,
}

/// POST /api/registration-applications — apply to join (logged-in users).
/// One pending application per user → 409.
pub async fn apply_registration(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<ApplyBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(AppError::BadRequest("reason required".to_string()));
    }
    if reason.chars().count() > 2000 {
        return Err(AppError::BadRequest(
            "reason too long (max 2000 chars)".to_string(),
        ));
    }

    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM registration_applications
         WHERE user_id = $1 AND status = 'pending'",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    if existing.is_some() {
        return Err(AppError::Conflict(
            "You already have a pending registration application".to_string(),
        ));
    }

    let app_id: i64 = sqlx::query_scalar(
        "INSERT INTO registration_applications (user_id, reason) VALUES ($1, $2) RETURNING id",
    )
    .bind(user_id)
    .bind(reason)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "id": app_id,
        "status": "pending",
    })))
}

#[derive(Debug, Deserialize)]
pub struct AppListQuery {
    #[serde(default = "default_app_status")]
    pub status: String,
}

fn default_app_status() -> String {
    "all".into()
}

/// GET /api/admin/registration-applications?status=pending|approved|rejected|all
/// (role ≥ 5).
pub async fn list_registration_applications(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(params): Query<AppListQuery>,
) -> Result<Json<Value>, AppError> {
    require_mod(&auth)?;

    let status = params.status.trim();
    if !matches!(status, "pending" | "approved" | "rejected" | "all") {
        return Err(AppError::BadRequest(
            "status must be one of: pending, approved, rejected, all".to_string(),
        ));
    }

    let rows = if status == "all" {
        sqlx::query_as::<
            _,
            (
                i64,
                i32,
                Option<String>,
                String,
                String,
                Option<i32>,
                Option<String>,
                Option<DateTime<Utc>>,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT a.id, a.user_id, u.username, a.reason, a.status,
                      a.reviewed_by, rv.username, a.reviewed_at, a.created_at
               FROM registration_applications a
               JOIN users u ON u.id = a.user_id
               LEFT JOIN users rv ON rv.id = a.reviewed_by
               ORDER BY a.created_at DESC
               LIMIT 200"#,
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<
            _,
            (
                i64,
                i32,
                Option<String>,
                String,
                String,
                Option<i32>,
                Option<String>,
                Option<DateTime<Utc>>,
                DateTime<Utc>,
            ),
        >(
            r#"SELECT a.id, a.user_id, u.username, a.reason, a.status,
                      a.reviewed_by, rv.username, a.reviewed_at, a.created_at
               FROM registration_applications a
               JOIN users u ON u.id = a.user_id
               LEFT JOIN users rv ON rv.id = a.reviewed_by
               WHERE a.status = $1
               ORDER BY a.created_at DESC
               LIMIT 200"#,
        )
        .bind(status)
        .fetch_all(&state.db)
        .await?
    };

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                user_id,
                username,
                reason,
                status,
                reviewed_by,
                reviewer_name,
                reviewed_at,
                created_at,
            )| {
                json!({
                    "id": id,
                    "user_id": user_id,
                    "username": username,
                    "reason": reason,
                    "status": status,
                    "reviewed_by": reviewed_by,
                    "reviewer_name": reviewer_name,
                    "reviewed_at": reviewed_at.map(|d| d.to_rfc3339()),
                    "created_at": created_at.to_rfc3339(),
                })
            },
        )
        .collect();

    Ok(Json(json!({ "err": 0, "status": status, "items": items })))
}

#[derive(Debug, Deserialize)]
pub struct ReviewBody {
    /// 'approved' or 'rejected'.
    pub status: String,
    /// Optional note stored in the modlog details.
    #[serde(default)]
    pub reason: Option<String>,
}

/// POST /api/admin/registration-applications/{id}/review (role ≥ 5).
/// Approving flips the applicant's `users.role` to 1 (trusted) so the
/// account becomes usable. Records a `registration_app_review` modlog entry.
pub async fn review_registration_application(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(app_id): Path<i64>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<Value>, AppError> {
    let actor_id = require_mod(&auth)?;

    let status = body.status.trim();
    if !matches!(status, "approved" | "rejected") {
        return Err(AppError::BadRequest(
            "status must be 'approved' or 'rejected'".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    // Fetch the application (only pending ones may be reviewed).
    let (user_id, current_status): (i32, String) =
        sqlx::query_as("SELECT user_id, status FROM registration_applications WHERE id = $1")
            .bind(app_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if current_status != "pending" {
        return Err(AppError::Conflict(format!(
            "Application already {current_status}"
        )));
    }

    let updated = sqlx::query(
        "UPDATE registration_applications
         SET status = $1, reviewed_by = $2, reviewed_at = NOW()
         WHERE id = $3 AND status = 'pending'",
    )
    .bind(status)
    .bind(actor_id)
    .bind(app_id)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "Application already reviewed".to_string(),
        ));
    }

    // Approval activates the applicant's account (role 0 → 1 = trusted).
    if status == "approved" {
        sqlx::query("UPDATE users SET role = 1 WHERE id = $1 AND role = 0")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    let mut extra = vec![
        ("application_id", json!(app_id)),
        ("applicant_id", json!(user_id)),
        ("status", json!(status)),
    ];
    if let Some(r) = body.reason.as_deref().filter(|r| !r.trim().is_empty()) {
        extra.push(("reason", json!(r.trim())));
    }
    crate::modlog::record_json(
        &state.db,
        Some(actor_id),
        auth.username.clone(),
        "registration_app_review",
        "registration_application",
        &app_id.to_string(),
        extra,
    )
    .await;

    Ok(Json(json!({
        "err": 0,
        "id": app_id,
        "status": status,
        "user_id": user_id,
    })))
}

// ── Blocks ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BlockBody {
    pub user_id: i32,
}

/// POST /api/blocks — block a user. Self-block → 400. Idempotent.
pub async fn block_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<BlockBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    if body.user_id == user_id {
        return Err(AppError::BadRequest(
            "You cannot block yourself".to_string(),
        ));
    }
    // The target must exist.
    let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(body.user_id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    sqlx::query(
        "INSERT INTO blocked_users (user_id, blocked_user_id) VALUES ($1, $2)
         ON CONFLICT (user_id, blocked_user_id) DO NOTHING",
    )
    .bind(user_id)
    .bind(body.user_id)
    .execute(&state.db)
    .await?;

    Ok(Json(
        json!({ "err": 0, "user_id": body.user_id, "blocked": true }),
    ))
}

/// DELETE /api/blocks/{user_id} — unblock. Idempotent (missing row → ok).
pub async fn unblock_user(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(user_id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let actor_id = require_user(&auth)?;

    sqlx::query("DELETE FROM blocked_users WHERE user_id = $1 AND blocked_user_id = $2")
        .bind(actor_id)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(
        json!({ "err": 0, "user_id": user_id, "blocked": false }),
    ))
}

/// GET /api/blocks — list the caller's blocked users (ids + usernames).
pub async fn list_blocks(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let user_id = require_user(&auth)?;

    let rows = sqlx::query_as::<_, (i32, String, DateTime<Utc>)>(
        r#"SELECT b.blocked_user_id, u.username, b.created_at
           FROM blocked_users b
           JOIN users u ON u.id = b.blocked_user_id
           WHERE b.user_id = $1
           ORDER BY b.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = rows
        .into_iter()
        .map(|(id, username, created_at)| {
            json!({
                "user_id": id,
                "username": username,
                "created_at": created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({ "err": 0, "items": items })))
}
