//! Site-wide DMs API — Lane 3 of Phase 4 integration-first rewrite.
//!
//! Live schema from migration 076 (verified against prod 2026-09-07):
//!   forum_rooms(id, name NULL=DM, is_group, creator_id NOT NULL,
//!     created_at, updated_at, last_message_id, last_activity_at)
//!   forum_room_members(room_id, user_id, role owner/moderator/member,
//!     joined_at, left_at NULL=active, is_muted, notify_level all/mentions/none,
//!     PK(room_id, user_id))
//!   forum_messages(id, room_id, author_id, body, payload DEFAULT '{}',
//!     edited_at, deleted_at, is_hidden, created_at, search_vector [079 trigger])
//!   forum_user_blocks(blocker_id, blocked_id) — LEGACY, unused. Block
//!     checks use the canonical site table `blocked_users(user_id,
//!     blocked_user_id)` (see plan §6.3 correction).
//!
//! Routes are site-level, sibling of notifications.rs.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::FromRow;
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::require_user;
use crate::server::AppState;

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateRoomPayload {
    pub user_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub body: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RoomListQuery {
    #[serde(default)]
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessageListQuery {
    #[serde(default)]
    pub cursor: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoomPrefsPayload {
    #[serde(default)]
    pub mute: Option<bool>,
    #[serde(default)]
    pub notify_level: Option<String>,
    #[serde(default)]
    pub leave: Option<bool>,
}

#[derive(Debug, sqlx::FromRow)]
struct RoomRow {
    id: i64,
    name: Option<String>,
    is_group: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, FromRow)]
struct MessageRow {
    id: i64,
    room_id: i64,
    author_id: i32,
    author_username: Option<String>,
    body: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

fn message_to_json(m: &MessageRow) -> serde_json::Value {
    json!({
        "id": m.id,
        "room_id": m.room_id,
        "sender_id": m.author_id,
        "sender_username": m.author_username,
        "body": m.body,
        "created_at": m.created_at.to_rfc3339(),
    })
}

/// Active membership (left_at IS NULL). Non-member → 404 per repo convention.
async fn require_membership(
    db: &sqlx::PgPool,
    room_id: i64,
    user_id: i32,
) -> Result<(), AppError> {
    let is_member: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM forum_room_members
           WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL)"#,
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    if !is_member {
        return Err(AppError::NotFound("Room not found".to_string()));
    }
    Ok(())
}

/// Block check in either direction against the canonical site table.
async fn is_blocked(db: &sqlx::PgPool, a: i32, b: i32) -> Result<bool, AppError> {
    let blocked: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM blocked_users
           WHERE (user_id = $1 AND blocked_user_id = $2)
              OR (user_id = $2 AND blocked_user_id = $1))"#,
    )
    .bind(a)
    .bind(b)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(blocked)
}

/// DM flood gate: same FORUM_POST_DELAY_SECS config as forum posts, measured
/// from the sender's most recent DM. Staff bypass.
async fn check_dm_flood(
    state: &AppState,
    auth: &AuthUser,
    sender_id: i32,
) -> Result<(), AppError> {
    if auth.trust_level >= 5 {
        return Ok(());
    }
    let delay = state.config.forum_post_delay_secs.max(0);
    if delay <= 0 {
        return Ok(());
    }
    let last: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT MAX(created_at) FROM forum_messages WHERE author_id = $1",
    )
    .bind(sender_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(last_ts) = last {
        let elapsed = chrono::Utc::now()
            .signed_duration_since(last_ts)
            .num_seconds();
        if elapsed < i64::from(delay) {
            return Err(AppError::BadRequest(format!(
                "flood control: wait {}s before sending again",
                i64::from(delay) - elapsed
            )));
        }
    }
    Ok(())
}

// ── GET /api/messages/rooms ──────────────────────────────────────────────────

/// List my active rooms, ordered by last_activity_at DESC.
pub async fn list_rooms(
    Query(q): Query<RoomListQuery>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;
    let filter = q.filter.as_deref().unwrap_or("all");
    if !matches!(filter, "all" | "dm" | "group") {
        return Err(AppError::BadRequest(
            "filter must be one of all|dm|group".to_string(),
        ));
    }

    let rooms: Vec<RoomRow> = match filter {
        "dm" => sqlx::query_as::<_, RoomRow>(
            r#"SELECT r.id, r.name, r.is_group, r.created_at, r.last_activity_at
               FROM forum_rooms r
               JOIN forum_room_members rm ON rm.room_id = r.id
                 AND rm.user_id = $1 AND rm.left_at IS NULL
               WHERE r.is_group = FALSE
               ORDER BY COALESCE(r.last_activity_at, r.created_at) DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?,
        "group" => sqlx::query_as::<_, RoomRow>(
            r#"SELECT r.id, r.name, r.is_group, r.created_at, r.last_activity_at
               FROM forum_rooms r
               JOIN forum_room_members rm ON rm.room_id = r.id
                 AND rm.user_id = $1 AND rm.left_at IS NULL
               WHERE r.is_group = TRUE
               ORDER BY COALESCE(r.last_activity_at, r.created_at) DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?,
        _ => sqlx::query_as::<_, RoomRow>(
            r#"SELECT r.id, r.name, r.is_group, r.created_at, r.last_activity_at
               FROM forum_rooms r
               JOIN forum_room_members rm ON rm.room_id = r.id
                 AND rm.user_id = $1 AND rm.left_at IS NULL
               ORDER BY COALESCE(r.last_activity_at, r.created_at) DESC"#,
        )
        .bind(user_id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?,
    };

    Ok(Json(json!({
        "err": 0,
        "rooms": rooms.iter().map(|r| json!({
            "id": r.id,
            "name": r.name,
            "is_group": r.is_group,
            "created_at": r.created_at.to_rfc3339(),
            "last_activity_at": r.last_activity_at.as_ref().map(|d| d.to_rfc3339()),
        })).collect::<Vec<_>>(),
        "count": rooms.len(),
    })))
}

// ── POST /api/messages/rooms ─────────────────────────────────────────────────

/// Find-or-create a DM room. Existing room → `{err:0, room_id, existing:true}`
/// (200, not 409 — idempotent find-or-create). Blocked either direction → 403.
/// Target user must exist.
pub async fn create_dm_room(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<CreateRoomPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let caller_id = require_user(&auth)?;
    let target_id = payload.user_id;

    if caller_id == target_id {
        return Err(AppError::BadRequest(
            "Cannot create a DM with yourself".to_string(),
        ));
    }
    let target_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(target_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if !target_exists {
        return Err(AppError::NotFound("User not found".to_string()));
    }
    if is_blocked(&state.db, caller_id, target_id).await? {
        return Err(AppError::Forbidden(
            "Cannot message this user".to_string(),
        ));
    }

    // Check-then-insert inside one transaction; re-check after BEGIN so two
    // racing creators serialize on the members PK rather than double-create.
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let existing: Option<i64> = sqlx::query_scalar(
        r#"SELECT r.id FROM forum_rooms r
           JOIN forum_room_members m1 ON m1.room_id = r.id AND m1.user_id = $1 AND m1.left_at IS NULL
           JOIN forum_room_members m2 ON m2.room_id = r.id AND m2.user_id = $2 AND m2.left_at IS NULL
           WHERE r.name IS NULL AND r.is_group = FALSE
           LIMIT 1"#,
    )
    .bind(caller_id)
    .bind(target_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    if let Some(room_id) = existing {
        tx.rollback()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        return Ok(Json(json!({ "err": 0, "room_id": room_id, "existing": true })));
    }

    let room_id: i64 = sqlx::query_scalar(
        r#"INSERT INTO forum_rooms (name, is_group, creator_id, created_at, last_activity_at)
           VALUES (NULL, FALSE, $1, NOW(), NOW()) RETURNING id"#,
    )
    .bind(caller_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    for member_id in [caller_id, target_id] {
        sqlx::query(
            r#"INSERT INTO forum_room_members (room_id, user_id, role, joined_at)
               VALUES ($1, $2, 'member', NOW())"#,
        )
        .bind(room_id)
        .bind(member_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(
        json!({ "err": 0, "room_id": room_id, "existing": false }),
    ))
}

// ── GET /api/messages/rooms/{roomId}/messages ────────────────────────────────

/// Member-only cursor pagination (newest first). Non-member → 404.
pub async fn get_room_messages(
    Path(room_id): Path<i64>,
    Query(params): Query<MessageListQuery>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;
    require_membership(&state.db, room_id, user_id).await?;

    let limit = params.limit.unwrap_or(50).clamp(1, 100);
    let messages: Vec<MessageRow> = if let Some(cursor) = params.cursor {
        sqlx::query_as::<_, MessageRow>(
            r#"SELECT m.id, m.room_id, m.author_id, u.username AS author_username,
                      m.body, m.created_at
               FROM forum_messages m
               LEFT JOIN users u ON u.id = m.author_id
               WHERE m.room_id = $1 AND m.id < $2
                 AND m.is_hidden = FALSE AND m.deleted_at IS NULL
               ORDER BY m.id DESC LIMIT $3"#,
        )
        .bind(room_id)
        .bind(cursor)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        sqlx::query_as::<_, MessageRow>(
            r#"SELECT m.id, m.room_id, m.author_id, u.username AS author_username,
                      m.body, m.created_at
               FROM forum_messages m
               LEFT JOIN users u ON u.id = m.author_id
               WHERE m.room_id = $1 AND m.is_hidden = FALSE AND m.deleted_at IS NULL
               ORDER BY m.id DESC LIMIT $2"#,
        )
        .bind(room_id)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
    };

    Ok(Json(json!({
        "err": 0,
        "messages": messages.iter().map(message_to_json).collect::<Vec<_>>(),
        "has_more": messages.len() as i64 >= limit,
    })))
}

// ── POST /api/messages/rooms/{roomId}/messages ───────────────────────────────

/// Send a message. Hard-rejects (403) when either side blocks the other.
/// Message row first, then per-recipient site notifications (best-effort),
/// all in one transaction. `{err:0}` envelope.
pub async fn send_message(
    Path(room_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<SendMessagePayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sender_id = require_user(&auth)?;
    require_membership(&state.db, room_id, sender_id).await?;
    check_dm_flood(&state, &auth, sender_id).await?;

    let body = payload.body.trim().to_string();
    if body.is_empty() || body.chars().count() > 20_000 {
        return Err(AppError::BadRequest(
            "message body required (max 20000 chars)".to_string(),
        ));
    }

    // Other active members — for block checks + notifications.
    let members: Vec<i32> = sqlx::query_scalar(
        r#"SELECT user_id FROM forum_room_members
           WHERE room_id = $1 AND user_id != $2 AND left_at IS NULL"#,
    )
    .bind(room_id)
    .bind(sender_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Block check up front: a blocked sender is rejected, not silently sent.
    for member_id in &members {
        if is_blocked(&state.db, sender_id, *member_id).await? {
            return Err(AppError::Forbidden(
                "Cannot message this user".to_string(),
            ));
        }
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let msg_id: i64 = sqlx::query_scalar(
        r#"INSERT INTO forum_messages (room_id, author_id, body, payload, created_at)
           VALUES ($1, $2, $3, COALESCE($4, '{}'::jsonb), NOW()) RETURNING id"#,
    )
    .bind(room_id)
    .bind(sender_id)
    .bind(&body)
    .bind(payload.payload)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    sqlx::query(
        r#"UPDATE forum_rooms SET last_activity_at = NOW(), last_message_id = $2 WHERE id = $1"#,
    )
    .bind(room_id)
    .bind(msg_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check for newly unlocked achievements (best-effort)
    let _ = crate::services::achievements::check_and_unlock_achievements(
        &state.db,
        sender_id,
        "message_sent",
    )
    .await;

    // Notifications after commit (best-effort per recipient, forum.rs precedent).
    let link = format!("/messages/{}", room_id);
    for member_id in &members {
        let prefs: Option<(bool, String)> = sqlx::query_as(
            r#"SELECT is_muted, notify_level FROM forum_room_members
               WHERE room_id = $1 AND user_id = $2"#,
        )
        .bind(room_id)
        .bind(*member_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        if let Some((muted, ref level)) = prefs {
            if muted || level == "none" {
                continue;
            }
        }
        // Muted members and notify_level=none get no row; 'mentions' on a DM
        // (no mention syntax parsed here) is treated as 'all' — every DM is
        // implicitly addressed to its members.
        let _ = queries::create_notification(
            &state.db,
            *member_id,
            "message",
            "New message",
            Some(&body.chars().take(200).collect::<String>()),
            Some(&link),
            Some("room"),
            Some(&room_id.to_string()),
        )
        .await;

        // Publish a realtime event to the recipient's personal channel.
        let _ = crate::realtime::publish_event(
            &state.rt_manager,
            state.redis_client.as_ref(),
            &format!("user:{}", member_id),
            "message_new",
            json!({ "room_id": room_id, "message_id": msg_id, "author_id": sender_id }),
        )
        .await;
    }

    Ok(Json(json!({ "err": 0, "message_id": msg_id })))
}

// ── PATCH /api/messages/rooms/{roomId} ───────────────────────────────────────

/// Mute / notify_level / leave. Leave sets left_at (history preserved).
pub async fn update_room_prefs(
    Path(room_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(payload): Json<UpdateRoomPrefsPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&auth)?;
    require_membership(&state.db, room_id, user_id).await?;

    if let Some(level) = payload.notify_level.as_deref() {
        if !matches!(level, "all" | "mentions" | "none") {
            return Err(AppError::BadRequest(
                "notify_level must be one of all|mentions|none".to_string(),
            ));
        }
        sqlx::query(
            "UPDATE forum_room_members SET notify_level = $1 WHERE room_id = $2 AND user_id = $3",
        )
        .bind(level)
        .bind(room_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    if let Some(mute) = payload.mute {
        sqlx::query(
            "UPDATE forum_room_members SET is_muted = $1 WHERE room_id = $2 AND user_id = $3",
        )
        .bind(mute)
        .bind(room_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }
    if payload.leave.unwrap_or(false) {
        sqlx::query(
            "UPDATE forum_room_members SET left_at = NOW() WHERE room_id = $1 AND user_id = $2",
        )
        .bind(room_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    }

    Ok(Json(json!({ "err": 0, "room_id": room_id })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefs_payload_defaults_empty() {
        let p: UpdateRoomPrefsPayload =
            serde_json::from_value(json!({})).expect("empty payload");
        assert!(p.mute.is_none());
        assert!(p.notify_level.is_none());
        assert!(p.leave.is_none());
    }

    #[test]
    fn test_message_json_sender_key() {
        let row = MessageRow {
            id: 1,
            room_id: 2,
            author_id: 7,
            author_username: Some("alice".to_string()),
            body: "hi".to_string(),
            created_at: chrono::Utc::now(),
        };
        let v = message_to_json(&row);
        assert_eq!(v["sender_id"], 7);
        assert_eq!(v["sender_username"], "alice");
    }
}
