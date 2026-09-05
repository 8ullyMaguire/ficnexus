use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct NotifQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/v1/notifications — list notifications for current user
pub async fn list_notifications_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<NotifQueryParams>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;
    let limit = params.limit.unwrap_or(20).min(50).max(1);
    let offset = params.offset.unwrap_or(0).max(0);

    let notifs = queries::list_notifications(&state.db, user_id, limit, offset).await?;
    let unread_count = queries::get_unread_notification_count(&state.db, user_id).await?;

    let items: Vec<Value> = notifs
        .into_iter()
        .map(|n| {
            json!({
                "id": n.id,
                "notification_type": n.notification_type,
                "title": n.title,
                "body": n.body,
                "link": n.link,
                "reference_type": n.reference_type,
                "reference_id": n.reference_id,
                "is_read": n.is_read,
                "created_at": n.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "notifications": items,
        "unread_count": unread_count,
    })))
}

/// POST /api/v1/notifications/{id}/read — mark notification as read
pub async fn mark_notification_read_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(notif_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let marked = queries::mark_notification_read(&state.db, user_id, notif_id).await?;

    Ok(Json(json!({
        "err": 0,
        "marked": marked,
    })))
}

/// POST /api/v1/notifications/read-all — mark all as read
pub async fn mark_all_read_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    queries::mark_all_notifications_read(&state.db, user_id).await?;

    Ok(Json(json!({ "err": 0, "msg": "All marked read" })))
}

/// GET /api/v1/notifications/unread-count
pub async fn unread_count_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let count = queries::get_unread_notification_count(&state.db, user_id).await?;

    Ok(Json(json!({ "err": 0, "unread_count": count })))
}

#[derive(Debug, Deserialize)]
pub struct PrefsBody {
    pub comment_reply: Option<bool>,
    pub follow_update: Option<bool>,
    pub work_update: Option<bool>,
    pub badge_earned: Option<bool>,
    pub curator_promotion: Option<bool>,
    pub recommendation: Option<bool>,
    pub email_digest: Option<String>,
    // ── Granular AO3-style toggles (migration 060) ─────────────────────
    pub comments_on_work: Option<bool>,
    pub replies_to_comments: Option<bool>,
    pub kudos_on_work: Option<bool>,
    pub bookmarks_on_work: Option<bool>,
    pub follows: Option<bool>,
    pub mentions: Option<bool>,
}

/// GET /api/v1/notifications/preferences
pub async fn get_preferences_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let prefs = queries::get_notification_preferences(&state.db, user_id).await?;

    Ok(Json(json!({
        "err": 0,
        "preferences": {
            "comment_reply": prefs.comment_reply,
            "follow_update": prefs.follow_update,
            "work_update": prefs.work_update,
            "badge_earned": prefs.badge_earned,
            "curator_promotion": prefs.curator_promotion,
            "recommendation": prefs.recommendation,
            "email_digest": prefs.email_digest,
            "comments_on_work": prefs.comments_on_work,
            "replies_to_comments": prefs.replies_to_comments,
            "kudos_on_work": prefs.kudos_on_work,
            "bookmarks_on_work": prefs.bookmarks_on_work,
            "follows": prefs.follows,
            "mentions": prefs.mentions,
        }
    })))
}

/// PUT /api/v1/notifications/preferences
pub async fn update_preferences_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PrefsBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let mut prefs = queries::get_notification_preferences(&state.db, user_id).await?;

    if let Some(v) = body.comment_reply {
        prefs.comment_reply = v;
    }
    if let Some(v) = body.follow_update {
        prefs.follow_update = v;
    }
    if let Some(v) = body.work_update {
        prefs.work_update = v;
    }
    if let Some(v) = body.badge_earned {
        prefs.badge_earned = v;
    }
    if let Some(v) = body.curator_promotion {
        prefs.curator_promotion = v;
    }
    if let Some(v) = body.recommendation {
        prefs.recommendation = v;
    }
    if let Some(ref d) = body.email_digest {
        if !["instant", "daily", "weekly", "never"].contains(&d.as_str()) {
            return Err(AppError::BadRequest(
                "email_digest must be one of: instant, daily, weekly, never".to_string(),
            ));
        }
        prefs.email_digest = d.clone();
    }
    // ── Granular toggles (migration 060) ────────────────────────────────
    if let Some(v) = body.comments_on_work {
        prefs.comments_on_work = v;
    }
    if let Some(v) = body.replies_to_comments {
        prefs.replies_to_comments = v;
    }
    if let Some(v) = body.kudos_on_work {
        prefs.kudos_on_work = v;
    }
    if let Some(v) = body.bookmarks_on_work {
        prefs.bookmarks_on_work = v;
    }
    if let Some(v) = body.follows {
        prefs.follows = v;
    }
    if let Some(v) = body.mentions {
        prefs.mentions = v;
    }

    queries::update_notification_preferences(&state.db, user_id, &prefs).await?;

    Ok(Json(json!({ "err": 0, "msg": "Preferences updated" })))
}
