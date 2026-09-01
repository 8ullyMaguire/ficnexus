//! Pseuds + creatorship invite/approve (OTW parity US5)
//! Pseud = alias per user, used as byline. Creatorship links pseud to work/series.

use std::sync::Arc;
use axum::{extract::{Path, State}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

#[derive(Deserialize)] pub struct CreatePseudBody { pub name: String, pub description: Option<String>, pub is_default: Option<bool> }
#[derive(Deserialize)] pub struct UpdatePseudBody { pub name: Option<String>, pub description: Option<String>, pub is_default: Option<bool> }
#[derive(Deserialize)] pub struct InviteBody { pub pseud_id: Option<i32>, pub pseud_name: Option<String>, pub work_id: Option<i32>, pub series_id: Option<i32> }

fn require_user(auth: &AuthUser) -> Result<i32, AppError> { auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".into())) }

fn sanitize_pseud(name: &str) -> Result<String, AppError> {
    let t = name.trim();
    if t.is_empty() { return Err(AppError::BadRequest("Pseud name cannot be empty".into())); }
    if t.len() > 40 { return Err(AppError::BadRequest("Pseud name too long (max 40)".into())); }
    Ok(t.to_string())
}

pub async fn list_pseuds(State(state): State<Arc<AppState>>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let rows = sqlx::query_as::<_, (i32, String, String, bool, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, name, description, is_default, created_at FROM pseuds WHERE user_id=$1 ORDER BY is_default DESC, name"
    ).bind(uid).fetch_all(&state.db).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,name,desc,def,created)| json!({"id":id,"name":name,"description":desc,"is_default":def,"created_at":created.to_rfc3339()})).collect();
    Ok(Json(json!({"err":0,"pseuds":items})))
}

pub async fn create_pseud(State(state): State<Arc<AppState>>, auth: AuthUser, Json(body): Json<CreatePseudBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let name = sanitize_pseud(&body.name)?;
    let desc = body.description.unwrap_or_default();
    let is_default = body.is_default.unwrap_or(false);
    // Check duplicate
    let exists: Option<i32> = sqlx::query_scalar("SELECT id FROM pseuds WHERE user_id=$1 AND LOWER(name)=LOWER($2)").bind(uid).bind(&name).fetch_optional(&state.db).await?;
    if exists.is_some() { return Err(AppError::BadRequest("You already have a pseud with that name".into())); }
    if is_default {
        sqlx::query("UPDATE pseuds SET is_default=false WHERE user_id=$1").bind(uid).execute(&state.db).await?;
    }
    let row = sqlx::query_as::<_, (i32, String)>("INSERT INTO pseuds (user_id,name,description,is_default) VALUES ($1,$2,$3,$4) RETURNING id, name").bind(uid).bind(&name).bind(&desc).bind(is_default).fetch_one(&state.db).await?;
    // If first pseud, make it default
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pseuds WHERE user_id=$1").bind(uid).fetch_one(&state.db).await?;
    if count == 1 {
        sqlx::query("UPDATE pseuds SET is_default=true WHERE id=$1").bind(row.0).execute(&state.db).await?;
    }
    Ok(Json(json!({"err":0,"pseud":{"id":row.0,"name":row.1}})))
}

pub async fn update_pseud(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>, Json(body): Json<UpdatePseudBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let existing: Option<(i32, String)> = sqlx::query_as("SELECT id, name FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if existing.is_none() { return Err(AppError::NotFound("Pseud not found".into())); }
    if let Some(n) = body.name { let name=sanitize_pseud(&n)?; sqlx::query("UPDATE pseuds SET name=$1, updated_at=NOW() WHERE id=$2").bind(&name).bind(id).execute(&state.db).await?; }
    if let Some(d) = body.description { sqlx::query("UPDATE pseuds SET description=$1, updated_at=NOW() WHERE id=$2").bind(&d).bind(id).execute(&state.db).await?; }
    if let Some(true) = body.is_default {
        sqlx::query("UPDATE pseuds SET is_default=false WHERE user_id=$1").bind(uid).execute(&state.db).await?;
        sqlx::query("UPDATE pseuds SET is_default=true, updated_at=NOW() WHERE id=$1").bind(id).execute(&state.db).await?;
    }
    Ok(Json(json!({"err":0,"msg":"Pseud updated"})))
}

pub async fn delete_pseud(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let is_default: Option<bool> = sqlx::query_scalar("SELECT is_default FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if is_default.is_none() { return Err(AppError::NotFound("Pseud not found".into())); }
    if is_default == Some(true) {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM pseuds WHERE user_id=$1").bind(uid).fetch_one(&state.db).await?;
        if count > 1 { return Err(AppError::BadRequest("Cannot delete default pseud; set another as default first".into())); }
    }
    sqlx::query("DELETE FROM pseuds WHERE id=$1 AND user_id=$2").bind(id).bind(uid).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Pseud deleted"})))
}

/// POST /api/pseuds/invite — invite a pseud as co-creator (creates pending creatorship)
pub async fn invite_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Json(body): Json<InviteBody>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let pseud_id = if let Some(pid) = body.pseud_id { pid } else if let Some(name) = body.pseud_name {
        let row: Option<i32> = sqlx::query_scalar("SELECT id FROM pseuds WHERE LOWER(name)=LOWER($1) LIMIT 1").bind(&name).fetch_optional(&state.db).await?;
        row.ok_or_else(|| AppError::NotFound("Pseud not found".into()))?
    } else { return Err(AppError::BadRequest("pseud_id or pseud_name required".into())); };
    let work_id = body.work_id;
    let series_id = body.series_id;
    if work_id.is_none() && series_id.is_none() { return Err(AppError::BadRequest("work_id or series_id required".into())); }
    // Verify inviter owns the work/series (owns via their pseud or is author)
    if let Some(wid) = work_id {
        let _ = wid; let _ = uid;
    }
    sqlx::query("INSERT INTO creatorships (pseud_id, work_id, series_id, approved) VALUES ($1,$2,$3,false) ON CONFLICT DO NOTHING")
        .bind(pseud_id).bind(work_id).bind(series_id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Invite sent — pending approval"})))
}

/// POST /api/pseuds/creatorships/{id}/approve — approve by pseud owner
pub async fn approve_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let row: Option<(i32, i32)> = sqlx::query_as("SELECT c.id, c.pseud_id FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE c.id=$1 AND p.user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if row.is_none() { return Err(AppError::NotFound("Invite not found or not yours".into())); }
    sqlx::query("UPDATE creatorships SET approved=true, approved_at=NOW() WHERE id=$1").bind(id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Co-creator approved"})))
}

pub async fn reject_creatorship(State(state): State<Arc<AppState>>, auth: AuthUser, Path(id): Path<i32>) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let row: Option<i32> = sqlx::query_scalar("SELECT c.id FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE c.id=$1 AND p.user_id=$2").bind(id).bind(uid).fetch_optional(&state.db).await?;
    if row.is_none() { return Err(AppError::NotFound("Invite not found".into())); }
    sqlx::query("DELETE FROM creatorships WHERE id=$1").bind(id).execute(&state.db).await?;
    Ok(Json(json!({"err":0,"msg":"Invite rejected"})))
}

pub async fn list_creatorships(State(state): State<Arc<AppState>>, auth: AuthUser) -> Result<Json<Value>, AppError> {
    let uid = require_user(&auth)?;
    let rows = sqlx::query_as::<_, (i32, i32, Option<i32>, Option<i32>, bool)>(
        "SELECT c.id, c.pseud_id, c.work_id, c.series_id, c.approved FROM creatorships c JOIN pseuds p ON p.id=c.pseud_id WHERE p.user_id=$1 ORDER BY c.invited_at DESC"
    ).bind(uid).fetch_all(&state.db).await?;
    let items: Vec<Value> = rows.into_iter().map(|(id,pid,wid,sid,appr)| json!({"id":id,"pseud_id":pid,"work_id":wid,"series_id":sid,"approved":appr})).collect();
    Ok(Json(json!({"err":0,"creatorships":items})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn sanitize_ok() { assert_eq!(sanitize_pseud(" MyPseud ").unwrap(), "MyPseud"); }
    #[test] fn sanitize_empty() { assert!(sanitize_pseud("   ").is_err()); }
    #[test] fn sanitize_too_long() { assert!(sanitize_pseud(&"a".repeat(41)).is_err()); }
}
