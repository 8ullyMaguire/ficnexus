//! Lane A privilege matrix: per-category per-group.
//!
//! Live schema (073): forum_privileges(category_id, group_id, privilege).
//! See contracts/forum-groups-privileges.md section 2.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::admin_level;
use crate::server::AppState;

pub const VALID_PRIVILEGES: &[&str] = &["read", "write", "reply", "moderate", "flag", "manage_membership"];

pub fn valid_privilege(p: &str) -> bool {
    VALID_PRIVILEGES.contains(&p)
}

/// Resolution order (contract 2.4 + NodeBB semantics):
/// staff bypass -> is_mod_only gate -> group membership matrix -> default deny.
pub async fn can(
    db: &sqlx::PgPool,
    user_id: Option<i32>,
    user_level: i16,
    user_role: i16,
    category_id: i64,
    privilege: &str,
) -> Result<bool, AppError> {
    if user_level >= admin_level() || user_role >= 10 {
        return Ok(true);
    }
    let is_mod_only: Option<bool> =
        sqlx::query_scalar("SELECT is_mod_only FROM forum_categories WHERE id = $1")
            .bind(category_id)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    let is_mod_only = is_mod_only.unwrap_or(false);
    if is_mod_only && user_level < 50 && user_role < 5 {
        return Ok(false);
    }
    let uid = match user_id {
        Some(u) => u,
        None => return Ok(false),
    };
    let hit: Option<i64> = sqlx::query_scalar(
        "SELECT fp.id FROM forum_privileges fp
         JOIN forum_group_members m ON m.group_id = fp.group_id
         WHERE fp.category_id = $1 AND fp.privilege = $2 AND m.user_id = $3
         LIMIT 1",
    )
    .bind(category_id)
    .bind(privilege)
    .bind(uid)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(hit.is_some())
}

fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if auth.level >= admin_level() || auth.trust_level >= 5 {
        Ok(())
    } else {
        Err(AppError::Forbidden("admin only".into()))
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct PrivilegeRow {
    pub id: i64,
    pub category_id: i64,
    pub group_id: i64,
    pub group_name: String,
    pub group_slug: String,
    pub privilege: String,
    pub granted_by: i32,
    pub granted_by_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_category_privs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(cat_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _ = crate::routes::forum::require_user(&auth)?;
    let rows: Vec<PrivilegeRow> = sqlx::query_as(
        "SELECT fp.id, fp.category_id, fp.group_id,
                g.name AS group_name, g.slug AS group_slug,
                fp.privilege, fp.granted_by,
                u.username AS granted_by_name, fp.created_at
         FROM forum_privileges fp
         JOIN forum_groups g ON g.id = fp.group_id
         LEFT JOIN users u ON u.id = fp.granted_by
         WHERE fp.category_id = $1
         ORDER BY fp.id",
    )
    .bind(cat_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "category_id": r.category_id,
                "group": {"id": r.group_id, "name": r.group_name, "slug": r.group_slug},
                "privilege": r.privilege,
                "granted_by": {"id": r.granted_by, "username": r.granted_by_name},
                "created_at": r.created_at,
            })
        })
        .collect();
    Ok(Json(json!({"err": 0, "privileges": items})))
}

#[derive(Deserialize)]
pub struct GrantBody {
    pub group_id: i64,
    pub privilege: String,
}

pub async fn grant_category_priv(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(cat_id): Path<i64>,
    Json(body): Json<GrantBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    require_admin(&auth)?;
    if !valid_privilege(&body.privilege) {
        return Err(AppError::BadRequest(format!(
            "unknown privilege '{}'; valid: {}",
            body.privilege,
            VALID_PRIVILEGES.join(", ")
        )));
    }
    let granter = crate::routes::forum::require_user(&auth)?;
    let row: PrivilegeRow = sqlx::query_as(
        "WITH ins AS (
           INSERT INTO forum_privileges (category_id, group_id, privilege, granted_by)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (category_id, group_id, privilege) DO NOTHING
           RETURNING id, category_id, group_id, privilege, granted_by, created_at
         )
         SELECT ins.id, ins.category_id, ins.group_id,
                g.name AS group_name, g.slug AS group_slug,
                ins.privilege, ins.granted_by,
                u.username AS granted_by_name, ins.created_at
         FROM ins
         JOIN forum_groups g ON g.id = ins.group_id
         LEFT JOIN users u ON u.id = ins.granted_by",
    )
    .bind(cat_id)
    .bind(body.group_id)
    .bind(&body.privilege)
    .bind(granter)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "err": 0,
            "id": row.id,
            "category_id": row.category_id,
            "group": {"id": row.group_id, "name": row.group_name, "slug": row.group_slug},
            "privilege": row.privilege,
            "granted_by": {"id": row.granted_by, "username": row.granted_by_name},
            "created_at": row.created_at,
        })),
    ))
}

pub async fn revoke_category_priv(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((cat_id, priv_id)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    require_admin(&auth)?;
    let res = sqlx::query("DELETE FROM forum_privileges WHERE id = $1 AND category_id = $2")
        .bind(priv_id)
        .bind(cat_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("privilege row not found".into()));
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_all_privs(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    require_admin(&auth)?;
    let rows: Vec<PrivilegeRow> = sqlx::query_as(
        "SELECT fp.id, fp.category_id, fp.group_id,
                g.name AS group_name, g.slug AS group_slug,
                fp.privilege, fp.granted_by,
                u.username AS granted_by_name, fp.created_at
         FROM forum_privileges fp
         JOIN forum_groups g ON g.id = fp.group_id
         LEFT JOIN users u ON u.id = fp.granted_by
         ORDER BY fp.category_id, fp.id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "category_id": r.category_id,
                "group": {"id": r.group_id, "name": r.group_name, "slug": r.group_slug},
                "privilege": r.privilege,
                "granted_by": {"id": r.granted_by, "username": r.granted_by_name},
                "created_at": r.created_at,
            })
        })
        .collect();
    Ok(Json(json!({"err": 0, "privileges": items})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privilege_allowlist_matches_live_check_constraint() {
        for p in ["read", "write", "reply", "moderate", "flag", "manage_membership"] {
            assert!(valid_privilege(p), "{p} must be accepted");
        }
        for p in ["", "admin", "READ", "post_create", "allow"] {
            assert!(!valid_privilege(p), "{p} must be rejected");
        }
    }
}
