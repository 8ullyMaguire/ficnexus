//! Translation API — M1 of docs/plans/translation-everything.md
//!
//! Routes:
//! - POST /api/translate/batch   resolve existing translations (cache read)
//! - POST /api/translate/enqueue request machine translations (queue write)
//! - GET  /api/translate/status  coverage summary for a target

use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::routes::honeypot;
use crate::server::AppState;
use crate::services::translation::{self, TranslateBatchItem};

#[derive(Deserialize)]
pub struct BatchBody {
    locale: String,
    #[serde(default)]
    items: Vec<BatchQueryItem>,
}

#[derive(Deserialize)]
pub struct BatchQueryItem {
    #[serde(rename = "type")]
    typ: String,
    id: i64,
    #[serde(default = "default_field")]
    field: String,
}

fn default_field() -> String {
    "body".to_string()
}

/// POST /api/translate/batch — bulk read of resolved translations.
/// Anonymous, cheap: pure cache reads (approved > machine > null).
pub async fn batch(
    State(state): State<std::sync::Arc<AppState>>,
    Json(body): Json<BatchBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !state.config.translation_enabled {
        return Err(AppError::Forbidden("translation disabled".into()));
    }
    if body.items.len() > 50 {
        return Err(AppError::BadRequest("max 50 items per batch".into()));
    }
    let items: Vec<TranslateBatchItem> = body
        .items
        .iter()
        .map(|i| TranslateBatchItem {
            typ: i.typ.clone(),
            id: i.id.to_string(),
            field: i.field.clone(),
        })
        .collect();
    let out = translation::resolve_batch(&state.db, &items, &body.locale).await?;
    Ok(Json(json!({ "err": 0, "translations": out })))
}

#[derive(Deserialize)]
pub struct EnqueueBody {
    locale: String,
    #[serde(default)]
    items: Vec<BatchQueryItem>,
    // honeypot fields (see routes::honeypot)
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    form_opened_at: Option<String>,
}

/// POST /api/translate/enqueue — ask for machine translations.
/// Logged-in users only (budget tracking); honeypot-protected.
pub async fn enqueue(
    State(state): State<std::sync::Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<EnqueueBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !state.config.translation_enabled || !state.config.translate_llm_enabled {
        return Err(AppError::Forbidden("translation disabled".into()));
    }
    // Bots: silent fake success.
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    if honeypot::inspect_submission(body.website.as_deref(), body.form_opened_at.as_deref(), now_ms)
        == honeypot::TrapVerdict::RejectSilently
    {
        return Ok(Json(json!({ "err": 0, "queued": 0 })));
    }
    let Some(user_id) = user.user_id else {
        return Err(AppError::Unauthorized("login required".into()));
    };
    if body.items.is_empty() {
        return Err(AppError::BadRequest("no items".into()));
    }
    if body.items.len() > 50 {
        return Err(AppError::BadRequest("max 50 items per request".into()));
    }
    // locale must be one of the configured translate locales
    if !state
        .config
        .translate_locales
        .iter()
        .any(|l| l.eq_ignore_ascii_case(&body.locale))
    {
        return Err(AppError::BadRequest(format!(
            "unknown locale: {}",
            body.locale
        )));
    }
    let items: Vec<TranslateBatchItem> = body
        .items
        .iter()
        .map(|i| TranslateBatchItem {
            typ: i.typ.clone(),
            id: i.id.to_string(),
            field: i.field.clone(),
        })
        .collect();
    let mut conn = state.redis.clone();
    let queued = translation::enqueue(
        &mut conn,
        &items,
        &body.locale,
        &state.config,
        Some(user_id),
    )
    .await?;
    Ok(Json(json!({ "err": 0, "queued": queued })))
}

#[derive(Deserialize)]
pub struct StatusParams {
    target_type: String,
    target_id: String,
}

/// GET /api/translate/status?target_type=...&target_id=...
/// Coverage per configured locale: approved | machine | none.
pub async fn status(
    State(state): State<std::sync::Arc<AppState>>,
    Query(q): Query<StatusParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"SELECT locale, status FROM translation_strings
           WHERE target_type = $1 AND target_id = $2
             AND status IN ('approved','machine')"#,
    )
    .bind(&q.target_type)
    .bind(&q.target_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let mut coverage = serde_json::Map::new();
    for l in &state.config.translate_locales {
        let best = rows
            .iter()
            .filter(|(loc, _)| loc == l)
            .map(|(_, st)| if st == "approved" { "approved" } else { "machine" })
            .next()
            .unwrap_or("none");
        coverage.insert(l.clone(), json!(best));
    }
    Ok(Json(json!({ "err": 0, "coverage": coverage })))
}

