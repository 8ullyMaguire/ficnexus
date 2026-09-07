//! Lane A: forum groups + memberships.
//!
//! Routes (NodeBB parity for groups — see `contracts/forum-groups-privileges.md`):
//!   GET    /api/forum/groups                       — list (cursor pagination)
//!   POST   /api/forum/groups                       — create (TL2+)
//!   GET    /api/forum/groups/{groupId}            — detail + members page
//!   PATCH  /api/forum/groups/{groupId}            — update (owner/manager/admin)
//!   DELETE /api/forum/groups/{groupId}            — delete (owner/admin; never system)
//!   POST   /api/forum/groups/{groupId}/join       — join public / request private
//!   POST   /api/forum/groups/{groupId}/leave      — leave
//!   POST   /api/forum/groups/{groupId}/invite     — invite user (owner/manager)
//!   POST   /api/forum/groups/{groupId}/members/{userId}/role   — change role
//!   DELETE /api/forum/groups/{groupId}/members/{userId}        — remove member
//!
//! System groups (`is_system=true`) cannot be deleted and their name/slug
//! cannot change (they are referenced by Lane A's auto-membership cron and
//! by the `can()` privilege resolver in Lane A.2).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::forum::admin_level;
use crate::server::AppState;
use crate::services::trust::PUBLISH_MIN_TRUST;

/// Wire all groups routes into the given router. Returns a nested router
/// that can be merged into the forum chunk in `server.rs`.
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/groups", get(list_groups).post(create_group))
        .route("/groups/{group_id}", get(get_group).patch(update_group).delete(delete_group))
        .route("/groups/{group_id}/join", post(join_group))
        .route("/groups/{group_id}/leave", post(leave_group))
        .route("/groups/{group_id}/invite", post(invite_to_group))
        .route(
            "/groups/{group_id}/members/{user_id}",
            axum::routing::delete(remove_group_member),
        )
        .route(
            "/groups/{group_id}/members/{user_id}/role",
            post(change_member_role),
        )
}

// ── helpers ──────────────────────────────────────────────────────────────

const SYSTEM_PROTECTED_ERR: &str = "system groups cannot be modified";
const FORBIDDEN_NOT_OWNER: &str = "owner or moderator required";
const FORBIDDEN_NOT_ADMIN: &str = "admin required";

/// Caller is the group owner, a manager, or a site admin (level >= admin_level).
async fn is_group_admin(
    db: &PgPool,
    auth: &AuthUser,
    group_id: i64,
) -> Result<bool, AppError> {
    if auth.level >= crate::routes::forum::admin_level() {
        return Ok(true);
    }
    let uid = match auth.user_id {
        Some(id) => id,
        None => return Ok(false),
    };
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM forum_group_members
         WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(uid)
    .fetch_optional(db)
    .await?;
    Ok(matches!(role.as_deref(), Some("owner") | Some("manager")))
}

async fn group_exists(db: &PgPool, group_id: i64) -> Result<bool, AppError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM forum_groups WHERE id = $1)",
    )
    .bind(group_id)
    .fetch_one(db)
    .await?;
    Ok(exists)
}

fn validate_group_slug(slug: &str) -> Result<String, AppError> {
    let s = slug.trim().to_lowercase();
    if !(2..=60).contains(&s.chars().count()) {
        return Err(AppError::BadRequest(
            "slug must be 2-60 characters".to_string(),
        ));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest(
            "slug must be lowercase alphanumeric + - or _".to_string(),
        ));
    }
    Ok(s)
}

fn validate_group_name(name: &str) -> Result<String, AppError> {
    let t = name.trim();
    let n = t.chars().count();
    if !(1..=100).contains(&n) {
        return Err(AppError::BadRequest(
            "name must be 1-100 characters".to_string(),
        ));
    }
    Ok(t.to_string())
}

// ── list / get ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListGroupsQuery {
    pub search: Option<String>,
    pub r#type: Option<String>,
    pub limit: Option<i64>,
    pub cursor: Option<i64>,
}

/// `GET /api/forum/groups` — list groups visible to the caller, with
/// cursor pagination, optional type filter, and optional search by
/// name/description.
///
/// Visibility (contract §1.1):
///   - public groups: everyone
///   - private groups: members and staff
///   - system groups: staff only
#[allow(clippy::sql_injection)] // see body — fragments are literals only
pub async fn list_groups(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<ListGroupsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = q.limit.unwrap_or(25).clamp(1, 100);
    let cursor = q.cursor.unwrap_or(i64::MAX);
    let caller_uid = auth.user_id.unwrap_or(0);
    let caller_role = auth.role;
    let caller_level = auth.level;
    let is_staff = caller_level >= crate::routes::forum::admin_level()
        || caller_role >= 10;

    // Build WHERE per filters and visibility:
    //   - public groups are visible to everyone
    //   - private groups visible only to members and staff
    //   - system groups only visible to staff
    // All branches insert LITERAL SQL fragments from this function
    // (no caller data), so the `format!` is safe — clippy's
    // `sql_injection` lint just can't see that statically. Each push
    // is `String` so we can keep them in a single Vec.
    #[allow(clippy::sql_injection)] // literal fragments only
    {
        let mut conds: Vec<&str> = vec!["g.id < $1"];
        if let Some(t) = q.r#type.as_deref() {
            match t {
                "public" => conds.push("g.is_private = FALSE"),
                "private" => conds.push("g.is_private = TRUE"),
                "system" => conds.push("g.is_system = TRUE"),
                _ => {
                    return Err(AppError::BadRequest(
                        "type must be one of public|private|system".to_string(),
                    ))
                }
            }
        } else if !is_staff {
            conds.push(
                "(g.is_system = FALSE AND (g.is_private = FALSE OR member.cnt > 0))",
            );
        }
        if q.search.is_some() {
            // search binding will be $4
            conds.push("(g.name ILIKE $4 OR g.description ILIKE $4)");
        }
        // Single static query: `type` and `search` are encoded as
        // COALESCE filters, so the SQL never depends on runtime values.
        // Staff sees system groups; everyone else only sees public +
        // private groups they're a member of.
        let type_filter: Option<String> = q.r#type.as_deref().map(|t| t.to_string());
        let search: Option<String> = q
            .search
            .as_deref()
            .map(|s| format!("%{}%", s.trim()));

        let sql = "\
            SELECT g.id, g.name, g.slug, g.description, g.cover_url, g.is_private, g.is_system, \
                   g.owner_id, u.username AS owner_username, \
                   g.created_at, \
                   (SELECT COUNT(*)::BIGINT FROM forum_group_members m WHERE m.group_id = g.id) AS member_count, \
                   EXISTS(SELECT 1 FROM forum_group_members m3 \
                          WHERE m3.group_id = g.id AND m3.user_id = $2) AS caller_is_member \
            FROM forum_groups g \
            JOIN users u ON u.id = g.owner_id \
            LEFT JOIN ( \
                SELECT group_id, COUNT(*)::BIGINT AS cnt \
                  FROM forum_group_members WHERE user_id = $2 GROUP BY group_id \
            ) member ON member.group_id = g.id \
            WHERE g.id < $1 \
              AND ($5::text IS NULL OR \
                   ($5 = 'public'  AND g.is_private = FALSE) OR \
                   ($5 = 'private' AND g.is_private = TRUE)  OR \
                   ($5 = 'system'  AND g.is_system  = TRUE)) \
              AND ($6::bool IS FALSE OR g.is_system = FALSE) \
              AND ($6::bool IS FALSE OR g.is_private = FALSE OR member.cnt > 0) \
              AND ($7::text IS NULL OR g.name ILIKE $7 OR g.description ILIKE $7) \
            ORDER BY g.id DESC \
            LIMIT $3";

        let query = sqlx::query_as::<_, GroupRow>(sql)
            .bind(cursor)
            .bind(caller_uid)
            .bind(limit)
            .bind(0_i64) // $4 reserved for future
            .bind(type_filter) // $5
            .bind(is_staff) // $6 (bool)
            .bind(search); // $7

        let rows = query.fetch_all(&state.db).await?;
        let next_cursor = rows.last().map(|r| r.id);
        let has_more = rows.len() as i64 == limit;

        let groups: Vec<Value> = rows
            .into_iter()
            .map(|r| {
                let user_role = if r.caller_is_member {
                    Some("member".to_string())
                } else {
                    None
                };
                json!({
                    "id": r.id,
                    "name": r.name,
                    "slug": r.slug,
                    "description": r.description,
                    "cover_url": r.cover_url,
                    "is_private": r.is_private,
                    "is_system": r.is_system,
                    "owner": { "id": r.owner_id, "username": r.owner_username },
                    "member_count": r.member_count,
                    "user_role": user_role,
                    "created_at": r.created_at,
                })
            })
            .collect();

        Ok(Json(json!({
            "err": 0,
            "groups": groups,
            "next_cursor": next_cursor,
            "has_more": has_more,
        })))
    }
}

#[derive(sqlx::FromRow)]
struct GroupRow {
    id: i64,
    name: String,
    slug: String,
    description: String,
    cover_url: Option<String>,
    is_private: bool,
    is_system: bool,
    owner_id: i32,
    owner_username: String,
    member_count: i64,
    caller_is_member: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_group(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(group_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    if !group_exists(&state.db, group_id).await? {
        return Err(AppError::NotFound("group not found".to_string()));
    }
    let row: Option<GroupRow> = sqlx::query_as::<_, GroupRow>(
        "SELECT g.id, g.name, g.slug, g.description, g.cover_url, g.is_private, g.is_system,
                g.owner_id, u.username AS owner_username,
                g.created_at,
                (SELECT COUNT(*)::BIGINT FROM forum_group_members m WHERE m.group_id = g.id) AS member_count,
                EXISTS(SELECT 1 FROM forum_group_members m3
                       WHERE m3.group_id = g.id AND m3.user_id = $2) AS caller_is_member
           FROM forum_groups g
           JOIN users u ON u.id = g.owner_id
          WHERE g.id = $1",
    )
    .bind(group_id)
    .bind(auth.user_id.unwrap_or(0))
    .fetch_optional(&state.db)
    .await?;
    let r = row.ok_or_else(|| AppError::NotFound("group not found".to_string()))?;
    let is_staff = auth.level >= crate::routes::forum::admin_level();
    if r.is_system && !is_staff {
        return Err(AppError::Forbidden("not allowed".to_string()));
    }
    if r.is_private && !r.caller_is_member && !is_staff {
        return Err(AppError::Forbidden("not a member".to_string()));
    }

    // First page of members (cursor pagination)
    let members: Vec<(i32, String, String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT gm.user_id, u.username, gm.role, gm.joined_at
           FROM forum_group_members gm
           JOIN users u ON u.id = gm.user_id
          WHERE gm.group_id = $1
          ORDER BY gm.joined_at ASC
          LIMIT 50",
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await?;
    let members_json: Vec<Value> = members
        .into_iter()
        .map(|(uid, uname, role, joined)| {
            json!({
                "user": { "id": uid, "username": uname },
                "role": role,
                "joined_at": joined,
            })
        })
        .collect();

    let caller_role = if r.caller_is_member {
        sqlx::query_scalar::<_, String>(
            "SELECT role FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
        )
        .bind(group_id)
        .bind(auth.user_id.unwrap_or(0))
        .fetch_optional(&state.db)
        .await?
        .unwrap_or_else(|| "member".to_string())
    } else {
        String::new()
    };

    Ok(Json(json!({
        "err": 0,
        "id": r.id,
        "name": r.name,
        "slug": r.slug,
        "description": r.description,
        "cover_url": r.cover_url,
        "is_private": r.is_private,
        "is_system": r.is_system,
        "owner": { "id": r.owner_id, "username": r.owner_username },
        "member_count": r.member_count,
        "user_role": caller_role,
        "created_at": r.created_at,
        "members": members_json,
    })))
}

// ── create / update / delete ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateGroupBody {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub is_private: Option<bool>,
}

pub async fn create_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateGroupBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = crate::routes::forum::require_user(&auth)?;
    crate::services::trust::assert_staff_or_min_trust(
        &state.db,
        Some(user_id),
        auth.role,
        PUBLISH_MIN_TRUST,
        "create forum group",
    )
    .await?;
    let name = validate_group_name(&body.name)?;
    let slug = validate_group_slug(&body.slug)?;
    let desc = body.description.unwrap_or_default();
    let is_private = body.is_private.unwrap_or(false);
    let mut tx = state.db.begin().await?;
    let group_id: i64 = sqlx::query_scalar(
        "INSERT INTO forum_groups (name, slug, description, cover_url, is_private, owner_id)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&name)
    .bind(&slug)
    .bind(&desc)
    .bind(&body.cover_url)
    .bind(is_private)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        use sqlx::Error::*;
        match e {
            Database(db) if db.constraint().is_some() => {
                AppError::Conflict("group name or slug already exists".to_string())
            }
            _ => AppError::Database(e.to_string()),
        }
    })?;
    // Owner auto-joins
    sqlx::query(
        "INSERT INTO forum_group_members (group_id, user_id, role, invited_by)
         VALUES ($1, $2, 'owner', $2)",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "err": 0,
            "id": group_id,
            "name": name,
            "slug": slug,
            "is_private": is_private,
        })),
    ))
}

#[derive(Deserialize)]
pub struct UpdateGroupBody {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub is_private: Option<bool>,
}

pub async fn update_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(body): Json<UpdateGroupBody>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::forum::require_user(&auth)?;
    if !is_group_admin(&state.db, &auth, group_id).await? {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_OWNER.to_string()));
    }
    let is_system: bool =
        sqlx::query_scalar("SELECT is_system FROM forum_groups WHERE id = $1")
            .bind(group_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("group not found".to_string()))?;
    if is_system && body.name.is_some() {
        return Err(AppError::Forbidden(SYSTEM_PROTECTED_ERR.to_string()));
    }
    let name = match &body.name {
        Some(n) => Some(validate_group_name(n)?),
        None => None,
    };
    let desc = body.description.as_deref();
    let cover = body.cover_url.clone();
    let priv_ = body.is_private;
    let res = sqlx::query(
        "UPDATE forum_groups
            SET name = COALESCE($1, name),
                description = COALESCE($2, description),
                cover_url = COALESCE($3, cover_url),
                is_private = COALESCE($4, is_private),
                updated_at = NOW()
          WHERE id = $5",
    )
    .bind(name)
    .bind(desc)
    .bind(cover)
    .bind(priv_)
    .bind(group_id)
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("group not found".to_string()));
    }
    // Touch caller so the request is not a no-op for empty bodies
    let _ = uid;
    Ok(Json(json!({ "err": 0, "id": group_id, "updated": true })))
}

pub async fn delete_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::forum::require_user(&auth)?;
    if auth.level < crate::routes::forum::admin_level() {
        // Non-admins must be owner
        let role: Option<String> = sqlx::query_scalar(
            "SELECT role FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
        )
        .bind(group_id)
        .bind(uid)
        .fetch_optional(&state.db)
        .await?;
        if role.as_deref() != Some("owner") {
            return Err(AppError::Forbidden(FORBIDDEN_NOT_OWNER.to_string()));
        }
    }
    let is_system: Option<bool> =
        sqlx::query_scalar("SELECT is_system FROM forum_groups WHERE id = $1")
            .bind(group_id)
            .fetch_optional(&state.db)
            .await?;
    if matches!(is_system, Some(true)) {
        return Err(AppError::Forbidden(SYSTEM_PROTECTED_ERR.to_string()));
    }
    let res = sqlx::query("DELETE FROM forum_groups WHERE id = $1")
        .bind(group_id)
        .execute(&state.db)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("group not found".to_string()));
    }
    Ok(Json(json!({ "err": 0, "deleted": true })))
}

// ── membership ───────────────────────────────────────────────────────────

pub async fn join_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::forum::require_user(&auth)?;
    let g: Option<(bool, bool)> = sqlx::query_as(
        "SELECT is_private, is_system FROM forum_groups WHERE id = $1",
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await?;
    let (is_private, is_system) =
        g.ok_or_else(|| AppError::NotFound("group not found".to_string()))?;
    if is_system {
        return Err(AppError::Forbidden("system groups are not joinable".to_string()));
    }
    if is_private {
        return Err(AppError::Forbidden(
            "private groups require an invite".to_string(),
        ));
    }
    sqlx::query(
        "INSERT INTO forum_group_members (group_id, user_id, role, invited_by)
         VALUES ($1, $2, 'member', $2)
         ON CONFLICT (group_id, user_id) DO NOTHING",
    )
    .bind(group_id)
    .bind(uid)
    .execute(&state.db)
    .await?;
    Ok(Json(json!({ "err": 0, "joined": true })))
}

pub async fn leave_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let uid = crate::routes::forum::require_user(&auth)?;
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(uid)
    .fetch_optional(&state.db)
    .await?;
    if role.as_deref() == Some("owner") {
        return Err(AppError::Forbidden(
            "owner cannot leave; transfer ownership or delete the group".to_string(),
        ));
    }
    let res = sqlx::query(
        "DELETE FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(uid)
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("not a member".to_string()));
    }
    Ok(Json(json!({ "err": 0, "left": true })))
}

#[derive(Deserialize)]
pub struct InviteBody {
    pub user_id: i32,
    pub role: Option<String>,
}

pub async fn invite_to_group(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i64>,
    Json(body): Json<InviteBody>,
) -> Result<Json<Value>, AppError> {
    let inviter = crate::routes::forum::require_user(&auth)?;
    if !is_group_admin(&state.db, &auth, group_id).await? {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_OWNER.to_string()));
    }
    let role = body.role.as_deref().unwrap_or("member");
    if !matches!(role, "owner" | "manager" | "member") {
        return Err(AppError::BadRequest(
            "role must be owner|manager|member".to_string(),
        ));
    }
    // Only admin can set role=owner
    if role == "owner" && auth.level < crate::routes::forum::admin_level() {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_ADMIN.to_string()));
    }
    sqlx::query(
        "INSERT INTO forum_group_members (group_id, user_id, role, invited_by)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (group_id, user_id) DO UPDATE SET role = EXCLUDED.role",
    )
    .bind(group_id)
    .bind(body.user_id)
    .bind(role)
    .bind(inviter)
    .execute(&state.db)
    .await?;
    Ok(Json(json!({ "err": 0, "invited": true, "role": role })))
}

#[derive(Deserialize)]
pub struct ChangeRoleBody {
    pub role: String,
}

pub async fn change_member_role(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((group_id, user_id)): Path<(i64, i32)>,
    Json(body): Json<ChangeRoleBody>,
) -> Result<Json<Value>, AppError> {
    if !is_group_admin(&state.db, &auth, group_id).await? {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_OWNER.to_string()));
    }
    if !matches!(body.role.as_str(), "owner" | "manager" | "member") {
        return Err(AppError::BadRequest(
            "role must be owner|manager|member".to_string(),
        ));
    }
    if body.role == "owner" && auth.level < crate::routes::forum::admin_level() {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_ADMIN.to_string()));
    }
    let res = sqlx::query(
        "UPDATE forum_group_members SET role = $1
          WHERE group_id = $2 AND user_id = $3",
    )
    .bind(&body.role)
    .bind(group_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("member not found".to_string()));
    }
    Ok(Json(json!({ "err": 0, "role": body.role })))
}

pub async fn list_group_members(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    // 1. Verify visibility
    let row: Option<(bool, i64)> =
        sqlx::query_as("SELECT is_private, created_by FROM forum_groups WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.db)
            .await?;
    let (is_private, owner) = match row {
        Some(r) => r,
        None => return Err(AppError::BadRequest("group not found".into())),
    };
    if is_private {
        let caller = auth.user_id;
        let is_staff =
            auth.level >= admin_level() || auth.role == 2 || caller == Some(owner as i32);
        if !is_staff {
            let member: Option<(i32,)> = if let Some(uid) = caller {
                sqlx::query_as(
                    "SELECT user_id FROM forum_group_members \
                     WHERE group_id = $1 AND user_id = $2",
                )
                .bind(id)
                .bind(uid)
                .fetch_optional(&state.db)
                .await?
            } else {
                None
            };
            if member.is_none() {
                return Err(AppError::Forbidden("not a member".into()));
            }
        }
    }

    let rows: Vec<(i32, String, chrono::DateTime<chrono::Utc>, Option<i32>)> = sqlx::query_as(
        "SELECT user_id, role, joined_at, invited_by \
         FROM forum_group_members WHERE group_id = $1 \
         ORDER BY (role = 'owner') DESC, (role = 'manager') DESC, joined_at ASC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let members: Vec<Value> = rows
        .into_iter()
        .map(|(user_id, role, joined_at, invited_by)| {
            json!({
                "user_id": user_id,
                "role": role,
                "joined_at": joined_at,
                "invited_by": invited_by,
            })
        })
        .collect();

    Ok(Json(json!({ "members": members })))
}

/// Remove a member (manager or admin). `user_id` is the user being removed.
pub async fn remove_group_member(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((group_id, user_id)): Path<(i64, i32)>,
) -> Result<Json<Value>, AppError> {
    if !is_group_admin(&state.db, &auth, group_id).await? {
        return Err(AppError::Forbidden(FORBIDDEN_NOT_OWNER.to_string()));
    }
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    let role = role.ok_or_else(|| AppError::NotFound("member not found".to_string()))?;
    if role == "owner" {
        return Err(AppError::Forbidden(
            "cannot remove owner; transfer ownership first".to_string(),
        ));
    }
    sqlx::query(
        "DELETE FROM forum_group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(Json(json!({ "err": 0, "removed": true })))
}

// ── system group seeder (idempotent) ─────────────────────────────────────

/// Insert the four system groups defined in the contract (`Administrators`,
/// `Moderators`, `Members`, `tl3-plus`) if they don't already exist.
/// Returns the slug→id map. Used by the bootstrap step in
/// `server::chunk_forum_activitypub` and by trust-promotion when adding
/// the user to `tl3-plus`.
pub async fn ensure_system_groups(db: &PgPool) -> Result<std::collections::HashMap<String, i64>, AppError> {
    const DEFS: &[(&str, &str, &str, bool)] = &[
        ("administrators", "Administrators", "Full access (role >= 10)", true),
        ("moderators", "Moderators", "Moderate privilege (TL5+ or role >= 5)", true),
        ("members", "Members", "Base write/reply (TL1+)", false),
        ("tl3-plus", "Trust Level 3+", "Tag creation, etc.", false),
    ];
    // Owner id 1 (the seed admin) — fine for system groups; the resolver
    // never relies on owner_id for privilege checks.
    let mut out = std::collections::HashMap::new();
    for (slug, name, desc, is_private) in DEFS {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO forum_groups (name, slug, description, is_private, is_system, owner_id)
             VALUES ($1, $2, $3, $4, TRUE, 1)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(name)
        .bind(slug)
        .bind(desc)
        .bind(is_private)
        .fetch_one(db)
        .await?;
        out.insert((*slug).to_string(), id);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_group_slug_rejects_short() {
        assert!(validate_group_slug("a").is_err());
        assert!(validate_group_slug("ab").is_ok());
    }

    #[test]
    fn validate_group_slug_rejects_bad_chars() {
        assert!(validate_group_slug("Has Space").is_err());
        assert!(validate_group_slug("ok-slug_1").is_ok());
    }

    #[test]
    fn validate_group_name_trims_and_bounds() {
        assert!(validate_group_name("   ").is_err());
        assert!(validate_group_name("My Group").is_ok());
        let long = "x".repeat(101);
        assert!(validate_group_name(&long).is_err());
    }
}
