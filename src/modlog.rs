//! Modlog: transparent record of moderator / curator / admin actions.
//!
//! Every mutating admin/curator action calls [`record`] (best-effort, never
//! fatal to the action itself). ANY logged-in user can read the log via
//! `GET /api/modlog` — moderation is completely transparent.

use serde_json::Value;

#[cfg(test)]
use serde_json::json;

/// Record a moderation action. Never fails the caller (errors are logged).
pub async fn record(
    db: &sqlx::PgPool,
    actor_id: Option<i32>,
    actor_username: Option<String>,
    action: &str,
    target_type: &str,
    target_id: &str,
    details: Value,
) {
    let res = sqlx::query(
        "INSERT INTO modlog (actor_id, actor_username, action, target_type, target_id, details)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(actor_id)
    .bind(actor_username)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(details)
    .execute(db)
    .await;
    if let Err(e) = res {
        tracing::warn!("modlog record failed for action '{action}': {e}");
    }
}

/// Convenience: record with a JSON object of extra detail keys.
pub async fn record_json(
    db: &sqlx::PgPool,
    actor_id: Option<i32>,
    actor_username: Option<String>,
    action: &str,
    target_type: &str,
    target_id: &str,
    extra: Vec<(&str, Value)>,
) {
    let mut details = serde_json::Map::new();
    for (k, v) in extra {
        details.insert(k.to_string(), v);
    }
    record(
        db,
        actor_id,
        actor_username,
        action,
        target_type,
        target_id,
        Value::Object(details),
    )
    .await;
}

/// Recent modlog entries, newest first.
pub async fn list(
    db: &sqlx::PgPool,
    limit: i64,
    action_filter: Option<&str>,
) -> sqlx::Result<Vec<ModlogEntry>> {
    let rows = if let Some(act) = action_filter {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             WHERE action = $2
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .bind(act)
        .fetch_all(db)
        .await?
    } else {
        sqlx::query_as::<_, ModlogEntry>(
            "SELECT id, actor_id, actor_username, action, target_type, target_id, details, created_at
             FROM modlog
             ORDER BY created_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(db)
        .await?
    };
    Ok(rows)
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ModlogEntry {
    pub id: i64,
    pub actor_id: Option<i32>,
    pub actor_username: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub details: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Map an actor (AuthUser) to (id, username) for logging.
pub fn actor_from_auth(auth: &crate::routes::auth::AuthUser) -> (Option<i32>, Option<String>) {
    (auth.user_id, auth.username.clone())
}

/// Simple guard for "any logged-in user" (not admin/curator-only).
pub fn require_logged_in(
    auth: &crate::routes::auth::AuthUser,
) -> Result<(), crate::error::AppError> {
    if auth.user_id.is_none() {
        return Err(crate::error::AppError::Unauthorized(
            "Login required".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_logged_in_rejects_anonymous() {
        let auth = crate::routes::auth::AuthUser::default();
        assert!(require_logged_in(&auth).is_err());
    }

    #[test]
    fn require_logged_in_accepts_user() {
        let auth = crate::routes::auth::AuthUser {
            user_id: Some(7),
            username: Some("u".into()),
            role: 0,
            level: 0,
        };
        assert!(require_logged_in(&auth).is_ok());
    }

    #[test]
    fn actor_from_auth_maps() {
        let auth = crate::routes::auth::AuthUser {
            user_id: Some(3),
            username: Some("mod".into()),
            role: 5,
            level: 50,
        };
        let (id, name) = actor_from_auth(&auth);
        assert_eq!(id, Some(3));
        assert_eq!(name.as_deref(), Some("mod"));
    }

    #[test]
    fn json_helper_builds_object() {
        let _ = record_json;
        let _ = json!({});
    }
}
