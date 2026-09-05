use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::routes::auth::AuthUser;
use crate::server::AppState;

// ---- Auth helper ----

/// Gate a tag-curator action on a logged-in user with role >= 10 (admin).
///
/// Follows the project's 400-as-403 convention: anonymous callers get
/// `400 {err:401}` ("Login required", same as the other authed routes) and
/// logged-in users below admin get `400 {err:403}`.
fn require_curator(user: &AuthUser) -> AppResult<()> {
    if user.user_id.is_none() {
        return Err(AppError::Unauthorized("Login required".to_string()));
    }
    if user.role < 10 {
        return Err(AppError::Forbidden("Curator access required".to_string()));
    }
    Ok(())
}

// ---- Request structures ----

#[derive(Debug, Deserialize)]
pub struct CreateAliasBody {
    pub alias_name: String,
    pub canonical_tag_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct MergeTagsBody {
    pub source_tag_id: i32,
    pub target_tag_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct FlagListQuery {
    pub resolved: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveFlagBody {
    pub resolved: bool,
}

// ---- Handlers ----

/// GET /api/curator/aliases
///
/// List all tag aliases with their canonical tag's name/type, most recent
/// first. Powers the curator Tag Aliases UI.
pub async fn list_aliases(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    #[derive(sqlx::FromRow)]
    struct AliasRow {
        alias_name: String,
        canonical_tag_id: i32,
        canonical_name: Option<String>,
        tag_type_id: Option<i16>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<AliasRow> = sqlx::query_as(
        r#"SELECT ta.alias_name,
                  ta.canonical_tag_id,
                  t.name AS canonical_name,
                  t.tag_type_id,
                  ta.created_at
           FROM tag_aliases ta
           LEFT JOIN tags t ON t.id = ta.canonical_tag_id
           ORDER BY ta.created_at DESC
           LIMIT 500"#,
    )
    .fetch_all(&state.db)
    .await?;

    let aliases: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "alias_name": r.alias_name,
                "canonical_tag_id": r.canonical_tag_id,
                "canonical_name": r.canonical_name,
                "tag_type_id": r.tag_type_id,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "aliases": aliases,
        "total": aliases.len(),
    })))
}

/// POST /api/v0/curator/alias
///
/// Create an alias pointing an alternative name to a canonical tag.
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateAliasBody>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    if body.alias_name.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "alias_name is required"})));
    }

    sqlx::query(
        "INSERT INTO tag_aliases (alias_name, canonical_tag_id) VALUES ($1, $2) ON CONFLICT (alias_name) DO NOTHING",
    )
    .bind(&body.alias_name)
    .bind(body.canonical_tag_id)
    .execute(&state.db)
    .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "create_alias",
        "tag_alias",
        &body.alias_name,
        vec![("canonical_tag_id", serde_json::json!(body.canonical_tag_id))],
    )
    .await;
    Ok(Json(json!({"err": 0, "msg": "alias created"})))
}

/// DELETE /api/curator/aliases/{alias_name}
///
/// Remove an alias mapping. The canonical tag and any tags are untouched —
/// only the alias row is deleted.
pub async fn delete_alias(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(alias_name): Path<String>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    let result = sqlx::query("DELETE FROM tag_aliases WHERE alias_name = $1")
        .bind(&alias_name)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Ok(Json(json!({"err": -5, "msg": "alias not found"})));
    }

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "delete_alias",
        "tag_alias",
        &alias_name,
        vec![],
    )
    .await;
    Ok(Json(json!({"err": 0, "msg": "alias deleted"})))
}

/// POST /api/v0/curator/merge
///
/// Merge `source_tag_id` into `target_tag_id`. All fic_tags and flags
/// referencing the source are migrated, then the source tag is deleted.
pub async fn merge_tags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<MergeTagsBody>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    if body.source_tag_id == body.target_tag_id {
        return Ok(Json(
            json!({"err": -1, "msg": "cannot merge a tag into itself"}),
        ));
    }

    // Migrate fic_tags references (skip duplicates)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft
           WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    )
    .bind(body.source_tag_id)
    .bind(body.target_tag_id)
    .execute(&state.db)
    .await?;

    // Migrate tag_flags
    sqlx::query(
        r#"INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason, resolved)
           SELECT tf.url_id, $2, tf.flagged_by_ip, tf.reason, tf.resolved
           FROM tag_flags tf
           WHERE tf.tag_id = $1
           ON CONFLICT (url_id, tag_id, flagged_by_ip) DO NOTHING"#,
    )
    .bind(body.source_tag_id)
    .bind(body.target_tag_id)
    .execute(&state.db)
    .await?;

    // Remove the old fic_tags rows (CASCADE will clean up votes)
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(body.source_tag_id)
        .execute(&state.db)
        .await?;

    // Delete the source tag itself
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(body.source_tag_id)
        .execute(&state.db)
        .await?;

    crate::modlog::record_json(
        &state.db,
        user.user_id,
        user.username.clone(),
        "merge_tags",
        "tag",
        &body.source_tag_id.to_string(),
        vec![("target_tag_id", serde_json::json!(body.target_tag_id))],
    )
    .await;
    Ok(Json(json!({"err": 0, "msg": "tags merged"})))
}

/// DELETE /api/v0/curator/tags/:id
///
/// Permanently delete a tag and all its associations (CASCADE).
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Ok(Json(json!({"err": -5, "msg": "tag not found"})));
    }

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "delete_tag",
        "tag",
        &id.to_string(),
        serde_json::json!({}),
    )
    .await;
    Ok(Json(json!({"err": 0, "msg": "tag deleted"})))
}

// ---- Tag edit (NEXT.md: curated tag metadata) ----

#[derive(Debug, Deserialize)]
pub struct UpdateTagBody {
    /// Human-readable description shown on the tag page.
    pub description: Option<String>,
    /// Re-classify the tag into another tag_type (1..=7).
    pub tag_type_id: Option<i16>,
    /// Optional: mark the tag as canonical (accepted only if the `tags`
    /// table has a `canonical` column; silently ignored otherwise).
    pub canonical: Option<bool>,
}

/// PUT /api/curator/tags/:id
///
/// Edit a tag's description / tag_type_id (and `canonical` when the column
/// exists). The `name` field is intentionally immutable — renaming a tag is
/// a merge operation (see `merge_tags`). At least one field is required;
/// unset fields keep their current value.
pub async fn update_tag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<UpdateTagBody>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    if body.description.is_none() && body.tag_type_id.is_none() && body.canonical.is_none() {
        return Ok(Json(
            json!({"err": -1, "msg": "nothing to update (send description, tag_type_id and/or canonical)"}),
        ));
    }

    // Verify the tag exists first (404-style err -5, matching delete_tag).
    let exists: Option<(i32,)> = sqlx::query_as("SELECT id FROM tags WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    if exists.is_none() {
        return Ok(Json(json!({"err": -5, "msg": "tag not found"})));
    }

    // Validate tag_type_id against the tag_types lookup table when provided.
    if let Some(type_id) = body.tag_type_id {
        if type_id < 1 {
            return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be >= 1"})));
        }
        let type_ok: Option<(i16,)> = sqlx::query_as("SELECT id FROM tag_types WHERE id = $1")
            .bind(type_id)
            .fetch_optional(&state.db)
            .await?;
        if type_ok.is_none() {
            return Ok(Json(
                json!({"err": -1, "msg": format!("unknown tag_type_id {type_id}")}),
            ));
        }
    }

    // `canonical` is optional: only apply it when the column exists (it is
    // not part of the base schema yet, so probe information_schema once).
    let has_canonical: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tags' AND column_name = 'canonical')",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(false);

    let (description, tag_type_id): (Option<String>, Option<i16>) = if has_canonical {
        let row: (Option<String>, Option<i16>, bool) = sqlx::query_as(
            r#"UPDATE tags
               SET description = COALESCE($2, description),
                   tag_type_id = COALESCE($3, tag_type_id),
                   canonical = COALESCE($4, canonical)
               WHERE id = $1
               RETURNING description, tag_type_id, canonical"#,
        )
        .bind(id)
        .bind(&body.description)
        .bind(body.tag_type_id)
        .bind(body.canonical)
        .fetch_one(&state.db)
        .await?;
        (row.0, row.1)
    } else {
        let row: (Option<String>, Option<i16>) = sqlx::query_as(
            r#"UPDATE tags
               SET description = COALESCE($2, description),
                   tag_type_id = COALESCE($3, tag_type_id)
               WHERE id = $1
               RETURNING description, tag_type_id"#,
        )
        .bind(id)
        .bind(&body.description)
        .bind(body.tag_type_id)
        .fetch_one(&state.db)
        .await?;
        (row.0, row.1)
    };

    Ok(Json(json!({
        "err": 0,
        "tag": {
            "id": id,
            "description": description,
            "tag_type_id": tag_type_id,
            "canonical": if has_canonical { body.canonical } else { None },
        },
        "msg": "tag updated",
    })))
}

/// GET /api/v0/curator/flags
///
/// List all tag flags, optionally filtered by resolved status.
pub async fn list_flags(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Query(params): Query<FlagListQuery>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    let resolved_filter = params.resolved.unwrap_or(false);

    #[derive(sqlx::FromRow)]
    struct FlagRow {
        id: i64,
        url_id: String,
        tag_id: i32,
        flagged_by_ip: String,
        reason: Option<String>,
        resolved: bool,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<FlagRow> = sqlx::query_as::<_, FlagRow>(
        r#"SELECT id, url_id, tag_id,
                  flagged_by_ip::text AS flagged_by_ip,
                  reason, resolved, created_at
           FROM tag_flags
           WHERE resolved = $1
           ORDER BY created_at DESC"#,
    )
    .bind(resolved_filter)
    .fetch_all(&state.db)
    .await?;

    let flags: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "url_id": r.url_id,
                "tag_id": r.tag_id,
                "flagged_by_ip": r.flagged_by_ip,
                "reason": r.reason,
                "resolved": r.resolved,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(json!({
        "err": 0,
        "flags": flags,
    })))
}

/// POST /api/v0/curator/flags/:id/resolve
///
/// Mark a flag as resolved (or unresolved).
pub async fn resolve_flag(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<i64>,
    Json(body): Json<ResolveFlagBody>,
) -> Result<Json<Value>, AppError> {
    require_curator(&user)?;

    let result = sqlx::query("UPDATE tag_flags SET resolved = $1 WHERE id = $2")
        .bind(body.resolved)
        .bind(id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Ok(Json(json!({"err": -5, "msg": "flag not found"})));
    }

    crate::modlog::record(
        &state.db,
        user.user_id,
        user.username.clone(),
        "resolve_flag",
        "tag_flag",
        &id.to_string(),
        serde_json::json!({"resolved": body.resolved}),
    )
    .await;

    Ok(Json(json!({"err": 0, "msg": "flag updated"})))
}
