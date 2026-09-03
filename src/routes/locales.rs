use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::db::queries;
use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// GET /api/v1/locales — list supported locales
pub async fn list_locales_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, AppError> {
    let locales = queries::get_locales(&state.db).await?;

    let items: Vec<Value> = locales.into_iter().map(|l| {
        json!({
            "code": l.code,
            "name": l.name,
            "is_rtl": l.is_rtl,
        })
    }).collect();

    Ok(Json(json!({ "err": 0, "locales": items })))
}

/// GET /api/v1/translations/{locale_code} — get UI translations
pub async fn get_ui_translations_handler(
    State(state): State<Arc<AppState>>,
    Path(locale_code): Path<String>,
) -> Result<Json<Value>, AppError> {
    let translations = queries::get_ui_translations(&state.db, &locale_code).await?;

    // Group by namespace
    let mut grouped = serde_json::Map::new();
    for t in translations {
        let ns = grouped.entry(t.namespace.clone())
            .or_insert_with(|| json!({}));
        if let Some(obj) = ns.as_object_mut() {
            obj.insert(t.key, json!(t.value));
        }
    }

    Ok(Json(json!({ "err": 0, "translations": grouped })))
}

#[derive(Debug, Deserialize)]
pub struct WorkTranslationBody {
    pub locale_code: String,
    pub title: Option<String>,
    pub summary: Option<String>,
}

/// POST /api/v1/works/{id}/translations — submit/update a work translation
pub async fn upsert_work_translation_handler(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(work_id): Path<i32>,
    Json(body): Json<WorkTranslationBody>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    if body.title.is_none() && body.summary.is_none() {
        return Err(AppError::BadRequest("At least one of title or summary is required".to_string()));
    }

    queries::upsert_work_translation(
        &state.db,
        work_id,
        &body.locale_code,
        body.title.as_deref(),
        body.summary.as_deref(),
        user_id,
    ).await?;

    Ok(Json(json!({ "err": 0, "msg": "Translation submitted" })))
}

/// GET /api/v1/works/{id}/translations/{locale_code}
pub async fn get_work_translation_handler(
    State(state): State<Arc<AppState>>,
    Path((work_id, locale_code)): Path<(i32, String)>,
) -> Result<Json<Value>, AppError> {
    let translation = queries::get_work_translation(&state.db, work_id, &locale_code).await?;

    match translation {
        Some(t) => Ok(Json(json!({
            "err": 0,
            "translation": {
                "work_id": t.work_id,
                "locale_code": t.locale_code,
                "title": t.title,
                "summary": t.summary,
                "translated_by": t.translated_by,
                "translated_at": t.translated_at.to_rfc3339(),
            }
        }))),
        None => Ok(Json(json!({ "err": 0, "translation": null }))),
    }
}

// ── Chapter-level translation routes ─────────────────────────────────

/// GET /api/works/{id}/chapter-translations/{locale}
pub async fn get_chapter_translations(
    State(state): State<Arc<AppState>>,
    Path((work_id, locale_code)): Path<(i32, String)>,
) -> Result<Json<Value>, AppError> {
    let row: Option<(serde_json::Value, Option<i32>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT chapters, translated_by, created_at FROM chapter_translation_versions WHERE work_id = $1 AND locale_code = $2 AND status = 'approved' ORDER BY version DESC LIMIT 1"
    )
    .bind(work_id).bind(&locale_code)
    .fetch_optional(&state.db).await?;

    match row {
        Some((chapters, translated_by, created_at)) => Ok(Json(json!({
            "err": 0,
            "work_id": work_id,
            "locale_code": locale_code,
            "chapters": chapters,
            "translated_by": translated_by,
            "created_at": created_at.to_rfc3339(),
        }))),
        None => Ok(Json(json!({"err": 0, "chapters": [], "locale_code": locale_code}))),
    }
}

/// PUT /api/works/{id}/chapter-translations/{locale}
pub async fn upsert_chapter_translations(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((work_id, locale_code)): Path<(i32, String)>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = user.user_id.ok_or_else(|| AppError::Unauthorized("Login required".to_string()))?;

    let chapters = payload.get("chapters")
        .ok_or_else(|| AppError::BadRequest("chapters field required".to_string()))?;

    sqlx::query(
        r#"INSERT INTO chapter_translation_versions (work_id, locale_code, version, chapters, translated_by)
           SELECT $1, $2, COALESCE(MAX(version), 0) + 1, $3, $4
           FROM chapter_translation_versions
           WHERE work_id = $1 AND locale_code = $2"#,
    )
    .bind(work_id).bind(&locale_code).bind(chapters).bind(user_id)
    .execute(&state.db).await?;

    // Award XP for translation
    let _ = queries::update_reputation_and_promote(&state.db, user_id, 15, "translation_submit").await;
    let _ = queries::check_and_award_badges(&state.db, user_id, "translation_submit").await;

    let reputation: i32 = sqlx::query_scalar("SELECT reputation FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(json!({
        "err": 0,
        "msg": "Translation saved",
        "rep_gained": 15,
        "reputation": reputation,
    })))
}
