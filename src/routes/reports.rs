//! User reports — user-submitted reports about comments, works, or users,
//! reviewed by admins (role >= 10).
//!
//! * `POST /api/reports` — any logged-in user files a report
//!   `{target_type, target_id, reason, details?}`.
//! * `GET /api/admin/reports?status=open` — admin lists reports.
//! * `POST /api/admin/reports/{id}/resolve` — admin resolves or dismisses a
//!   report (`{action: "resolved"|"dismissed"}`).

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::trust::{self, flag_weight};

/// Report weight threshold that marks a target as requiring admin review.
const CONTESTED_WEIGHT: i32 = 3;
/// Report weight threshold that auto-hides a target pending review.
const AUTO_HIDE_WEIGHT: i32 = 6;

#[derive(Debug, Deserialize)]
pub struct CreateBanAppealBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ReviewBanAppealBody {
    pub action: String,
    pub note: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BanAppealEntry {
    pub id: i64,
    pub ban_id: i64,
    pub user_id: i32,
    pub reason: String,
    pub status: String,
    pub reviewed_by: Option<i32>,
    pub reviewed_at: Option<chrono::DateTime<Utc>>,
    pub reviewer_note: Option<String>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReportBody {
    pub target_type: String,
    pub target_id: i32,
    pub reason: String,
    #[serde(default)]
    pub details: Option<Value>,
    #[serde(default = "default_report_category")]
    pub category: String,
}

fn default_report_category() -> String {
    "other".into()
}

#[derive(Debug, Deserialize)]
pub struct ReportQuery {
    #[serde(default = "default_report_status")]
    pub status: String,
}

fn default_report_status() -> String {
    "open".into()
}

#[derive(Debug, Deserialize)]
pub struct ResolveReportBody {
    /// 'resolved' (actionable, fixed) or 'dismissed' (no action needed).
    pub action: String,
}

/// POST /api/reports — file a report (logged-in users only).
pub async fn create_report(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateReportBody>,
) -> Result<Json<Value>, AppError> {
    let reporter_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Trust sandbox: TL0 (brand-new) may read but may not flag — flags carry
    // trust-weighted value, so a guest account must first engage.
    trust::assert_staff_or_min_trust(&state.db, Some(reporter_id), auth.trust_level, 1, "Filing reports")
        .await?;
    let reporter_trust = trust::fetch_trust_level(&state.db, Some(reporter_id)).await;
    let weight = flag_weight(reporter_trust);

    if !matches!(
        body.target_type.as_str(),
        "comment" | "work" | "user" | "forum_post" | "forum_topic"
    ) {
        return Err(AppError::BadRequest(
            "target_type must be one of: comment, work, user, forum_post, forum_topic".to_string(),
        ));
    }
    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(AppError::BadRequest("reason required".to_string()));
    }
    if !matches!(
        body.category.as_str(),
        "spam" | "harassment" | "copyright" | "inappropriate" | "other"
    ) {
        return Err(AppError::BadRequest(
            "category must be one of: spam, harassment, copyright, inappropriate, other".to_string(),
        ));
    }
    if reason.chars().count() > 1000 {
        return Err(AppError::BadRequest(
            "reason too long (max 1000 chars)".to_string(),
        ));
    }

    // Lightweight existence validation so admins never see reports about
    // nothing. Unknown targets are still stored (targets can be deleted
    // after the report was filed), so a missing target is a soft warning,
    // not an error.
    match body.target_type.as_str() {
        "work" => {
            let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM works WHERE id = $1")
                .bind(body.target_id)
                .fetch_optional(&state.db)
                .await?;
            if exists.is_none() {
                tracing::warn!("report for missing work {} stored", body.target_id);
            }
        }
        "comment" => {
            let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM comments WHERE id = $1")
                .bind(body.target_id as i64)
                .fetch_optional(&state.db)
                .await?;
            if exists.is_none() {
                tracing::warn!("report for missing comment {} stored", body.target_id);
            }
        }
        "forum_post" => {
            let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM forum_posts WHERE id = $1")
                .bind(body.target_id as i64)
                .fetch_optional(&state.db)
                .await?;
            if exists.is_none() {
                tracing::warn!("report for missing forum_post {} stored", body.target_id);
            }
        }
        "forum_topic" => {
            let exists: Option<(i64,)> =
                sqlx::query_as("SELECT id FROM forum_topics WHERE id = $1")
                    .bind(body.target_id as i64)
                    .fetch_optional(&state.db)
                    .await?;
            if exists.is_none() {
                tracing::warn!("report for missing forum_topic {} stored", body.target_id);
            }
        }
        _ => {}
    }

    let report_id: i64 = sqlx::query_scalar(
        r#"INSERT INTO user_reports (reporter_id, target_type, target_id, reason, details, weight, category)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(reporter_id)
    .bind(body.target_type.trim())
    .bind(body.target_id)
    .bind(reason)
    .bind(body.details)
    .bind(weight)
    .bind(&body.category)
    .fetch_one(&state.db)
    .await?;

    // Re-triage the target now that a new weighted flag landed: update
    // auto_status on all still-open reports for this target.
    run_report_triage(&state.db, &body.target_type, body.target_id).await?;

    Ok(Json(
        json!({"err": 0, "report_id": report_id, "msg": "Report submitted", "weight": weight}),
    ))
}

/// Aggregate weighted flags for a target and mark an `auto_status`:
///   pending      total weight < CONTESTED_WEIGHT
///   needs_admin  contested (weight >= CONTESTED_WEIGHT)
///   auto_hidden  overwhelming (weight >= AUTO_HIDE_WEIGHT) — surfaced to the
///                weekly digest / admin for a final call
/// Open reports keep their status; already-handled targets are left alone.
async fn run_report_triage(
    db: &sqlx::PgPool,
    target_type: &str,
    target_id: i32,
) -> Result<(), AppError> {
    let open_weighted: Option<(i64, i32)> = sqlx::query_as(
        r#"SELECT COUNT(*), COALESCE(SUM(weight), 0)
           FROM user_reports
           WHERE target_type = $1 AND target_id = $2 AND status = 'open'"#,
    )
    .bind(target_type)
    .bind(target_id)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(format!("report triage query: {e}")))?;

    let Some((count, total)) = open_weighted else {
        return Ok(());
    };

    let auto_status = if total >= AUTO_HIDE_WEIGHT {
        "auto_hidden"
    } else if total >= CONTESTED_WEIGHT {
        "needs_admin"
    } else {
        "pending"
    };

    sqlx::query(
        "UPDATE user_reports SET auto_status = $1
         WHERE target_type = $2 AND target_id = $3 AND status = 'open'",
    )
    .bind(auto_status)
    .bind(target_type)
    .bind(target_id)
    .execute(db)
    .await
    .map_err(|e| AppError::Database(format!("report triage update: {e}")))?;

    // Apply the auto-hide effect when a target crosses the threshold.
    // Mirrors the existing admin-hide conventions so moderators can undo it
    // with the tools they already have: comments use `is_hidden` (admin
    // unhide endpoint), forum posts use `hidden_until` with the same 72h
    // window as curator fast-hide — an unreviewed auto-hide self-reverses
    // rather than permanently censoring.
    if auto_status == "auto_hidden" {
        match target_type {
            "comment" => {
                sqlx::query("UPDATE comments SET is_hidden = TRUE WHERE id = $1")
                    .bind(target_id)
                    .execute(db)
                    .await
                    .map_err(|e| AppError::Database(format!("auto-hide comment: {e}")))?;
            }
            "forum_post" => {
                sqlx::query(
                    "UPDATE forum_posts SET hidden_until = NOW() + interval '72 hours'
                     WHERE id = $1 AND deleted_at IS NULL",
                )
                .bind(target_id)
                .execute(db)
                .await
                .map_err(|e| AppError::Database(format!("auto-hide forum post: {e}")))?;
            }
            // `work` and `user` targets have no visibility flag: hiding a
            // whole work or suspending a user on auto-weight alone would be
            // disproportionate — those stay `auto_hidden` for human review.
            _ => {}
        }
    }

    // Transparent modlog line when a target is escalated.
    if auto_status == "auto_hidden" {
        crate::modlog::record(
            db,
            None,
            None,
            "report_auto_hide",
            target_type,
            &target_id.to_string(),
            json!({ "weight": total, "reporters": count, "effect": auto_hide_effect(target_type) }),
        )
        .await;
    }
    Ok(())
}

/// Human-readable summary of what an auto-hide does to a target type.
fn auto_hide_effect(target_type: &str) -> &'static str {
    match target_type {
        "comment" => "comment hidden (visible after curator unhide)",
        "forum_post" => "forum post hidden for 72h (auto-expires, mirrors curator fast-hide)",
        _ => "flagged for admin review (no auto-hide effect)",
    }
}

/// GET /api/admin/reports?status=open — list reports (role >= 10).
pub async fn list_reports(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<ReportQuery>,
) -> Result<Json<Value>, AppError> {
    if user.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let status = params.status.trim();
    if !matches!(status, "open" | "resolved" | "dismissed" | "all") {
        return Err(AppError::BadRequest(
            "status must be one of: open, resolved, dismissed, all".to_string(),
        ));
    }

    let rows = if status == "all" {
        sqlx::query_as::<
            _,
            (
                i64,
                Option<i32>,
                Option<String>,
                String,
                i32,
                String,
                Option<Value>,
                String,
                String,
                chrono::DateTime<Utc>,
            ),
        >(
            r#"SELECT r.id, r.reporter_id, u.username, r.target_type, r.target_id,
                      r.reason, r.details, r.category, r.status, r.created_at
               FROM user_reports r
               LEFT JOIN users u ON u.id = r.reporter_id
               ORDER BY r.created_at DESC
               LIMIT 200"#,
        )
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<
            _,
            (
                i64,
                Option<i32>,
                Option<String>,
                String,
                i32,
                String,
                Option<Value>,
                String,
                String,
                chrono::DateTime<Utc>,
            ),
        >(
            r#"SELECT r.id, r.reporter_id, u.username, r.target_type, r.target_id,
                      r.reason, r.details, r.category, r.status, r.created_at
               FROM user_reports r
               LEFT JOIN users u ON u.id = r.reporter_id
               WHERE r.status = $1
               ORDER BY r.created_at DESC
               LIMIT 200"#,
        )
        .bind(status)
        .fetch_all(&state.db)
        .await?
    };

    // Resolve deep links for forum targets so the mod surface can link straight
    // to the content. forum_topic → /forum/board/{slug}.{id}; forum_post → its
    // topic's /forum/board/{slug}.{id}. Other target types have no forum link.
    let topic_ids: Vec<i32> = rows
        .iter()
        .filter(|r| r.3 == "forum_topic")
        .map(|r| r.4)
        .collect();
    let post_ids: Vec<i32> = rows
        .iter()
        .filter(|r| r.3 == "forum_post")
        .map(|r| r.4)
        .collect();

    let mut links: HashMap<String, String> = HashMap::new();
    if !topic_ids.is_empty() {
        if let Ok(rows) = sqlx::query_as::<_, (i32, String)>(
            "SELECT id, topic_slug FROM forum_topics WHERE id = ANY($1)",
        )
        .bind(&topic_ids)
        .fetch_all(&state.db)
        .await
        {
            for (id, slug) in rows {
                links.insert(format!("forum_topic:{id}"), format!("/forum/board/{slug}.{id}"));
            }
        }
    }
    if !post_ids.is_empty() {
        if let Ok(rows) = sqlx::query_as::<_, (i32, i32, String)>(
            "SELECT p.id, t.id, t.topic_slug FROM forum_posts p
             JOIN forum_topics t ON t.id = p.topic_id WHERE p.id = ANY($1)",
        )
        .bind(&post_ids)
        .fetch_all(&state.db)
        .await
        {
            for (post_id, topic_id, slug) in rows {
                links.insert(format!("forum_post:{post_id}"), format!("/forum/board/{slug}.{topic_id}"));
            }
        }
    }

    let items: Vec<Value> = rows
        .into_iter()
        .map(
            |(
                id,
                reporter_id,
                reporter_name,
                target_type,
                target_id,
                reason,
                details,
                category,
                status,
                created,
            )| {
                let link = links.get(&format!("{target_type}:{target_id}")).cloned();
                json!({
                    "id": id,
                    "reporter_id": reporter_id,
                    "reporter_name": reporter_name,
                    "target_type": target_type,
                    "target_id": target_id,
                    "reason": reason,
                    "details": details,
                    "category": category,
                    "status": status,
                    "created_at": created.to_rfc3339(),
                    "link": link,
                })
            },
        )
        .collect();

    Ok(Json(json!({"err": 0, "status": status, "items": items})))
}

/// GET /api/reports/{id}/status — reporter can check the status of their own report.
/// Returns the current status, resolution details, and any admin notes.
pub async fn report_status(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(report_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let row: Option<(
        i32,
        String,
        i32,
        String,
        String,
        String,
        Option<chrono::DateTime<Utc>>,
        Option<i32>,
    )> = sqlx::query_as(
        "SELECT reporter_id, target_type, target_id, status, category, auto_status,
                resolved_at, resolved_by
         FROM user_reports WHERE id = $1",
    )
    .bind(report_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Err(AppError::NotFound("Report not found".into()));
    };

    // Only the reporter or staff can view status
    if row.0 != user_id && auth.trust_level < 5 {
        return Err(AppError::Forbidden("Not allowed".into()));
    }

    Ok(Json(json!({
        "err": 0,
        "report_id": report_id,
        "status": row.3,
        "category": row.4,
        "auto_status": row.5,
        "resolved_at": row.6.map(|d| d.to_rfc3339()),
        "resolved_by": row.7,
    })))
}
pub async fn resolve_report(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(report_id): Path<i64>,
    Json(body): Json<ResolveReportBody>,
) -> Result<Json<Value>, AppError> {
    // Admins always; Community Moderators (TL5+) may also resolve reports.
    let is_admin = user.trust_level >= 5;
    if !is_admin {
        trust::assert_min_trust(
            &state.db,
            user.user_id,
            trust::RESOLVE_MIN_TRUST,
            "Resolving reports",
        )
        .await?;
    }

    let new_status = match body.action.as_str() {
        "resolved" => "resolved",
        "dismissed" => "dismissed",
        _ => {
            return Err(AppError::BadRequest(
                "action must be 'resolved' or 'dismissed'".to_string(),
            ));
        }
    };

    let result = sqlx::query(
        "UPDATE user_reports SET status = $1, auto_status = $1, resolved_by = $2, resolved_at = NOW()
         WHERE id = $3 AND status = 'open'",
    )
    .bind(new_status)
    .bind(user.user_id)
    .bind(report_id)
    .execute(&state.db)
    .await?;
    if result.rows_affected() == 0 {
        // Either missing, or already resolved/dismissed.
        return Err(AppError::NotFound(
            "Report not found or already handled".into(),
        ));
    }

    Ok(Json(
        json!({"err": 0, "report_id": report_id, "status": new_status}),
    ))
}

/// POST /api/bans/{id}/appeal — submit a ban appeal (banned user only).
pub async fn create_ban_appeal(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(ban_id): Path<i64>,
    Json(body): Json<CreateBanAppealBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    // Verify the ban exists and belongs to this user
    let ban: Option<(i32,)> = sqlx::query_as("SELECT user_id FROM forum_bans WHERE id = $1")
        .bind(ban_id)
        .fetch_optional(&state.db)
        .await?;

    let Some(ban) = ban else {
        return Err(AppError::NotFound("Ban not found".into()));
    };

    if ban.0 != user_id {
        return Err(AppError::Forbidden("You can only appeal your own bans".into()));
    }

    // Check for existing pending appeal
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM ban_appeals WHERE ban_id = $1 AND status = 'pending'",
    )
    .bind(ban_id)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "You already have a pending appeal for this ban".into(),
        ));
    }

    let reason = body.reason.trim();
    if reason.is_empty() {
        return Err(AppError::BadRequest("Appeal reason required".into()));
    }
    if reason.chars().count() > 2000 {
        return Err(AppError::BadRequest(
            "Appeal reason too long (max 2000 chars)".into(),
        ));
    }

    let appeal_id: i64 = sqlx::query_scalar(
        "INSERT INTO ban_appeals (ban_id, user_id, reason)
         VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(ban_id)
    .bind(user_id)
    .bind(reason)
    .fetch_one(&state.db)
    .await?;

    // Notify admins
    crate::modlog::record_json(
        &state.db,
        Some(user_id),
        auth.username.clone(),
        "ban_appeal_created",
        "forum_ban",
        &ban_id.to_string(),
        vec![("appeal_id", json!(appeal_id)), ("reason", json!(reason))],
    )
    .await;

    Ok(Json(
        json!({"err": 0, "appeal_id": appeal_id, "status": "pending"}),
    ))
}

/// GET /api/bans/{id}/appeals — list appeals for a ban (staff only).
pub async fn list_ban_appeals(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(ban_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let appeals = sqlx::query_as::<_, BanAppealEntry>(
        "SELECT id, ban_id, user_id, reason, status, reviewed_by, reviewed_at, reviewer_note, created_at
         FROM ban_appeals WHERE ban_id = $1 ORDER BY created_at DESC",
    )
    .bind(ban_id)
    .fetch_all(&state.db)
    .await?;

    let items: Vec<Value> = appeals
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "ban_id": a.ban_id,
                "user_id": a.user_id,
                "reason": a.reason,
                "status": a.status,
                "reviewed_by": a.reviewed_by,
                "reviewed_at": a.reviewed_at.map(|d| d.to_rfc3339()),
                "reviewer_note": a.reviewer_note,
                "created_at": a.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({"err": 0, "appeals": items})))
}

/// POST /api/bans/{id}/appeals/{appealId}/review — approve/reject an appeal (staff only).
pub async fn review_ban_appeal(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((ban_id, appeal_id)): Path<(i64, i64)>,
    Json(body): Json<ReviewBanAppealBody>,
) -> Result<Json<Value>, AppError> {
    if auth.trust_level < 5 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }

    let new_status = match body.action.as_str() {
        "approved" => "approved",
        "rejected" => "rejected",
        _ => {
            return Err(AppError::BadRequest(
                "action must be 'approved' or 'rejected'".to_string(),
            ));
        }
    };

    let result = sqlx::query(
        "UPDATE ban_appeals SET status = $1, reviewed_by = $2, reviewed_at = NOW(), reviewer_note = $3
         WHERE id = $4 AND ban_id = $5 AND status = 'pending'",
    )
    .bind(new_status)
    .bind(auth.user_id)
    .bind(body.note.as_deref().unwrap_or_default())
    .bind(appeal_id)
    .bind(ban_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Appeal not found or already reviewed".into(),
        ));
    }

    // If approved, delete the ban
    if new_status == "approved" {
        sqlx::query("DELETE FROM forum_bans WHERE id = $1")
            .bind(ban_id)
            .execute(&state.db)
            .await?;
    }

    // Notify the user
    let appeal: Option<(i32,)> = sqlx::query_as("SELECT user_id FROM ban_appeals WHERE id = $1")
        .bind(appeal_id)
        .fetch_optional(&state.db)
        .await?;

    if let Some(appeal) = appeal {
        let notif_title = if new_status == "approved" {
            "Ban Appeal Approved"
        } else {
            "Ban Appeal Rejected"
        };
        let notif_body = if new_status == "approved" {
            "Your ban appeal has been approved. Your access has been restored.".to_string()
        } else {
            format!(
                "Your ban appeal has been rejected. Reason: {}",
                body.note.as_deref().unwrap_or("No reason provided")
            )
        };
        let _ = crate::db::queries::social::create_notification(
            &state.db,
            appeal.0,
            "ban_appeal",
            notif_title,
            Some(&notif_body),
            Some("/appeals"),
            Some("ban_appeal"),
            Some(&appeal_id.to_string()),
        )
        .await;
    }

    crate::modlog::record_json(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        "ban_appeal_reviewed",
        "ban_appeal",
        &appeal_id.to_string(),
        vec![("action", json!(new_status)), ("note", json!(body.note))],
    )
    .await;

    Ok(Json(
        json!({"err": 0, "appeal_id": appeal_id, "status": new_status}),
    ))
}
