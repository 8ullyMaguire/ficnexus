//! Extension marketplace API.
//!
//! * `POST /api/extensions`            — publish (or, with `?upsert=1`, update)
//! * `GET  /api/extensions`            — gallery (kind=, q=, limit=)
//! * `GET  /api/extensions/kinds`      — supported kinds
//! * `GET  /api/extensions/{id}`       — fetch one (private owner view if owned)
//! * `POST /api/extensions/{id}/install`
//! * `POST /api/extensions/{id}/rate`  — `{ rating: 1..5 }`
//! * `POST /api/extensions/{id}/remix` — `{ slug, name }`
//!
//! Publishing is trust-gated (TL2+, see `services::trust::PUBLISH_MIN_TRUST`).

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;
use crate::services::extensions::{self, PublishExtBody, RateBody};
use crate::services::trust;

pub fn router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(gallery).post(publish))
        .route("/admin", get(admin_list))
        .route("/admin/{id}/verify", post(admin_verify))
        .route("/kinds", get(kinds))
        .route("/{id}", get(get_one))
        .route("/{id}/install", post(install))
        .route("/{id}/rate", post(rate))
        .route("/{id}/remix", post(remix))
        .route("/{id}/update", put(publish))
        .with_state(state)
}

/// GET /api/admin/extensions — admin listing of every extension (role >= 10).
pub(crate) async fn admin_list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<GalleryQuery>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let rows = extensions::gallery_all(&state.db, q.kind.as_deref(), q.q.as_deref(), 200).await?;
    Ok(Json(json!({ "err": 0, "items": rows })))
}

#[derive(Deserialize)]
pub(crate) struct GalleryQuery {
    kind: Option<String>,
    q: Option<String>,
    limit: Option<i64>,
}

/// GET /api/extensions/kinds — supported extension kinds.
pub(crate) async fn kinds() -> Json<Value> {
    Json(json!({ "err": 0, "kinds": extensions::KINDS }))
}

/// GET /api/extensions — gallery.
pub(crate) async fn gallery(
    State(state): State<Arc<AppState>>,
    Query(q): Query<GalleryQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = q.limit.unwrap_or(48).clamp(1, 100);
    let rows = extensions::gallery(&state.db, q.kind.as_deref(), q.q.as_deref(), limit).await?;
    Ok(Json(json!({ "err": 0, "items": rows })))
}

/// POST /api/extensions — publish a new extension (TL2+).
pub(crate) async fn publish(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(q): Query<PublishQuery>,
    Json(body): Json<PublishExtBody>,
) -> Result<Json<Value>, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let _ = trust::assert_staff_or_min_trust(&state.db, Some(uid), auth.role, trust::PUBLISH_MIN_TRUST, "Publishing extensions").await?;
    let ext = extensions::publish(&state.db, uid, body, q.upsert).await?;
    let verb = if q.upsert { "updated" } else { "published" };
    Ok(Json(json!({ "err": 0, "msg": format!("Extension {verb}"), "extension": ext })))
}

#[derive(Deserialize)]
pub(crate) struct PublishQuery {
    #[serde(default)]
    upsert: bool,
}

/// GET /api/extensions/{id} — fetch a single extension. Private (unpublished)
/// extensions are only visible to their owner.
pub(crate) async fn get_one(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
        let ext = extensions::get(&state.db, id).await?;
    if !ext.is_public
        && auth.user_id != Some(ext.author_id)
        && auth.role < 10
    {
        return Err(AppError::Forbidden("Not allowed".into()));
    }
    Ok(Json(json!({ "err": 0, "extension": ext })))
}

/// POST /api/extensions/{id}/install.
pub(crate) async fn install(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    extensions::install(&state.db, uid, id).await?;
    Ok(Json(json!({ "err": 0, "msg": "installed" })))
}

/// POST /api/extensions/{id}/rate — `{ rating: 1..5 }`.
pub(crate) async fn rate(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<RateBody>,
) -> Result<Json<Value>, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let avg = extensions::rate(&state.db, uid, id, body.rating).await?;
    Ok(Json(json!({ "err": 0, "average": avg })))
}

/// POST /api/admin/extensions/{id}/verify — toggle the curator-verified
/// badge (role >= 10). Verified extensions rank first in the gallery.
pub(crate) async fn admin_verify(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    if auth.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    let now_verified: Option<bool> = sqlx::query_scalar(
        "UPDATE extensions SET is_verified = NOT is_verified, updated_at = NOW()
         WHERE id = $1 RETURNING is_verified",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
    let Some(verified) = now_verified else {
        return Err(AppError::NotFound("Extension not found".into()));
    };
    crate::modlog::record(
        &state.db,
        auth.user_id,
        auth.username.clone(),
        if verified { "extension_verify" } else { "extension_unverify" },
        "extension",
        &id.to_string(),
        json!({ "verified": verified }),
    )
    .await;
    Ok(Json(json!({ "err": 0, "id": id, "is_verified": verified })))
}

/// POST /api/extensions/{id}/remix — `{ slug, name }`. Creates a private draft.
pub(crate) async fn remix(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<i32>,
    Json(body): Json<RemixBody>,
) -> Result<Json<Value>, AppError> {
    let uid = auth
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".into()))?;
    let ext = extensions::remix(&state.db, uid, id, &body.slug, &body.name).await?;
    Ok(Json(json!({ "err": 0, "msg": "remixed", "extension": ext })))
}

#[derive(Deserialize)]
pub(crate) struct RemixBody {
    slug: String,
    name: String,
}
