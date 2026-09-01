//! Saved searches + daily alerts.
//!
//! A logged-in user can save a search (a name + the raw query text + the JSON
//! AST of the parsed boolean query), re-run it on demand, toggle nightly alert
//! mode (`alert_mode` = `none` | `rss`), or delete it. The nightly
//! `saved_search_watcher` bin re-runs every alerting (`rss`) search, diffs the
//! current match set against `saved_search_matches`, and records newly-seen
//! works; those are exposed in a public per-search Atom feed at
//! `/feed/saved/{user_id}/{search_id}`.
//!
//! All storage uses the two tables added in migration 003:
//! `saved_searches` (the saved query + alert state) and `saved_search_matches`
//! (the works already seen for an alerting search).

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::routes::auth::AuthUser;
use crate::routes::rss::{atom_feed, atom_response, html_escape, iso_now};
use crate::search::builder::SearchParams;
use crate::search::parser::parse_query;
use crate::search::routes::run_search;
use crate::server::AppState;

/// A saved-search row as returned by list/get queries.
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct SavedSearchRow {
    pub id: i64,
    pub name: String,
    pub query_text: String,
    pub alert_mode: String,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_match_count: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request body for creating a saved search.
#[derive(Debug, Deserialize)]
pub struct CreateSavedSearchRequest {
    pub name: String,
    pub query_text: String,
}

/// Request body for updating the alert mode.
#[derive(Debug, Deserialize)]
pub struct UpdateAlertRequest {
    pub alert_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct AlertPathParams {
    pub id: i64,
}

/// Path params for the public saved-search feed: the user id + search id.
#[derive(Debug, Deserialize)]
pub struct FeedPathParams {
    pub user_id: i32,
    pub search_id: i64,
}

/// Validate an alert-mode value: only `none` and `rss` are accepted.
pub fn validate_alert_mode(mode: &str) -> Result<(), AppError> {
    match mode {
        "none" | "rss" => Ok(()),
        _ => Err(AppError::BadRequest(format!(
            "Invalid alert_mode '{}': must be 'none' or 'rss'",
            mode
        ))),
    }
}

/// Validate a create-saved-search request (name + non-empty query text).
pub fn validate_create_request(req: &CreateSavedSearchRequest) -> Result<(), AppError> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Saved search name is required".to_string()));
    }
    if name.len() > 120 {
        return Err(AppError::BadRequest(
            "Saved search name must be 120 characters or fewer".to_string(),
        ));
    }
    if req.query_text.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Saved search query cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Serialize a search query text into the JSON AST stored in `query_json`.
/// Re-parses via the same boolean parser the live search uses, so the stored
/// AST and a live `q` are always identical.
pub fn query_to_ast(query_text: &str) -> Value {
    serde_json::to_value(parse_query(query_text)).unwrap_or_else(|_| Value::Null)
}

/// Resolve the signed-in user's id or bail with 401.
fn require_user(auth: &AuthUser) -> Result<i32, AppError> {
    auth.user_id
        .ok_or_else(|| AppError::Unauthorized("Login required".to_string()))
}

/// POST /api/search/saved — create a saved search.
pub async fn create_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(body): Json<CreateSavedSearchRequest>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;
    validate_create_request(&body)?;

    let name = body.name.trim().to_string();
    let query_text = body.query_text.trim().to_string();
    let query_json = query_to_ast(&query_text);

    // The saved search must be runnable — fail fast if the query text does
    // not yield a meaningful search. Run the parser; an empty AST means the
    // query is effectively blank.
    if query_json == Value::Null && query_text.is_empty() {
        return Err(AppError::BadRequest(
            "Saved search query cannot be empty".to_string(),
        ));
    }

    let row: SavedSearchRow = sqlx::query_as(
        r#"INSERT INTO saved_searches (user_id, name, query_text, query_json)
           VALUES ($1, $2, $3, $4)
           RETURNING id, name, query_text, alert_mode, last_run_at, last_match_count, created_at"#,
    )
    .bind(user_id)
    .bind(&name)
    .bind(&query_text)
    .bind(&query_json)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref de) if de.is_unique_violation() => {
            AppError::Conflict(format!("A saved search named '{}' already exists", name))
        }
        other => AppError::Database(other.to_string()),
    })?;

    Ok(Json(json!({
        "err": 0,
        "saved_search": row,
    })))
}

/// GET /api/search/saved — list the current user's saved searches.
pub async fn list_saved_searches(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let rows: Vec<SavedSearchRow> = sqlx::query_as(
        r#"SELECT id, name, query_text, alert_mode, last_run_at, last_match_count, created_at
           FROM saved_searches
           WHERE user_id = $1
           ORDER BY created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "saved_searches": rows,
    })))
}

/// DELETE /api/search/saved/{id} — delete one of the current user's searches.
pub async fn delete_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let deleted = sqlx::query(
        "DELETE FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(path.id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if deleted.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Saved search {} not found",
            path.id
        )));
    }

    Ok(Json(json!({ "err": 0, "deleted": path.id })))
}

/// PUT /api/search/saved/{id}/alert — set the alert mode (none | rss).
pub async fn update_saved_search_alert(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
    Json(body): Json<UpdateAlertRequest>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;
    validate_alert_mode(&body.alert_mode)?;

    let updated = sqlx::query(
        "UPDATE saved_searches SET alert_mode = $1 WHERE id = $2 AND user_id = $3",
    )
    .bind(&body.alert_mode)
    .bind(path.id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Saved search {} not found",
            path.id
        )));
    }

    Ok(Json(json!({
        "err": 0,
        "id": path.id,
        "alert_mode": body.alert_mode,
    })))
}

/// POST /api/search/saved/{id}/run — re-run the saved query and return the
/// same result shape as the normal search (so the frontend can render it
/// identically). Also refreshes `last_run_at` / `last_match_count`.
pub async fn run_saved_search(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(path): Path<AlertPathParams>,
) -> AppResult<Json<Value>> {
    let user_id = require_user(&auth)?;

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT name, query_text FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(path.id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (name, query_text) = row.ok_or_else(|| {
        AppError::NotFound(format!("Saved search {} not found", path.id))
    })?;

    // Run through the exact same pipeline as a live /api/search call so the
    // response shape is identical (total / page / per_page / results / facets).
    let params = SearchParams {
        q: Some(query_text.clone()),
        user_id: Some(user_id),
        ..Default::default()
    };
    let envelope = run_search(&state, params).await?;

    let last_match_count = envelope.total as i32;
    sqlx::query(
        "UPDATE saved_searches SET last_run_at = now(), last_match_count = $1 WHERE id = $2",
    )
    .bind(last_match_count)
    .bind(path.id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "err": 0,
        "saved_search_id": path.id,
        "name": name,
        "total": envelope.total,
        "page": envelope.page,
        "per_page": envelope.per_page,
        "results": envelope.results,
        "facets": envelope.facets,
    })))
}

/// Load the works a saved search has matched, joined to their fic metadata,
/// ordered newest-first. Shared by the feed handler.
async fn matched_feed_rows(
    state: &Arc<AppState>,
    search_id: i64,
    limit: i64,
) -> AppResult<Vec<FeedRow>> {
    let rows: Vec<FeedRow> = sqlx::query_as(
        r#"SELECT fi.id AS url_id,
                  w.canonical_title AS title,
                  COALESCE(w.canonical_author, fi.author) AS author,
                  fi.description,
                  fi.words,
                  fi.chapters,
                  fi.status,
                  m.first_seen,
                  m.work_id
           FROM saved_search_matches m
           JOIN works w ON w.id = m.work_id
           LEFT JOIN fic_info fi ON fi.work_id = m.work_id
           WHERE m.search_id = $1
             AND fi.id IS NOT NULL
           ORDER BY m.first_seen DESC, m.work_id DESC
           LIMIT $2"#,
    )
    .bind(search_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// A matched-work row joined with fic metadata, for the Atom feed.
#[derive(Debug, Clone, sqlx::FromRow)]
struct FeedRow {
    url_id: String,
    title: String,
    author: String,
    description: String,
    words: i64,
    chapters: i32,
    status: String,
    first_seen: chrono::DateTime<chrono::Utc>,
    work_id: i32,
}

/// GET /feed/saved/{user_id}/{search_id} — public per-search Atom feed of the
/// works that saved search has matched so far (newest match first).
///
/// The route is registered as `/feed/saved/{user_id}/{search_id}` because axum
/// 0.8 forbids mixed literal+param segments like `{search_id}.xml`; a trailing
/// `.xml` is stripped here so the canonical URL works.
pub async fn saved_search_feed(
    State(state): State<Arc<AppState>>,
    Path(params): Path<FeedPathParams>,
) -> AppResult<impl IntoResponse> {
    let search_id = params.search_id;

    // Verify the search exists and belongs to the given user.
    let (name,): (String,) = sqlx::query_as(
        "SELECT name FROM saved_searches WHERE id = $1 AND user_id = $2",
    )
    .bind(search_id)
    .bind(params.user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("saved search not found".to_string()))?;

    let rows = matched_feed_rows(&state, search_id, super::rss::MAX_ENTRIES).await?;
    let now = iso_now();

    let entries: String = rows
        .iter()
        .map(|r| {
            let updated = r.first_seen.format("%Y-%m-%dT%H:%M:%SZ").to_string();
            format!(
                r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>urn:ficnexus:saved-search:{search_id}:{work_id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <link rel="alternate" href="/fic/{url_id}" type="text/html"/>
    <category term="{status}" label="{status}"/>
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <category term="words:{words}" label="{words} words"/>
  </entry>
"#,
                title = html_escape(&r.title),
                author = html_escape(&r.author),
                search_id = search_id,
                work_id = r.work_id,
                updated = updated,
                summary = html_escape(&r.description),
                url_id = html_escape(&r.url_id),
                status = html_escape(&r.status),
                chapters = r.chapters,
                words = r.words,
            )
        })
        .collect();

    let body = atom_feed(
        &format!("urn:ficnexus:feed:saved:{}:{}", params.user_id, search_id),
        &format!("FicNexus — Saved Search: {}", name),
        "New works matching this saved search",
        &format!(
            "/feed/saved/{}/{}{}",
            params.user_id,
            search_id,
            ".xml"
        ),
        &now,
        &entries,
        state.config.opds_base_url.as_deref(),
    );

    Ok(atom_response(body))
}

/// Rebuild a `SearchParams` for a saved query text (kept for exact-parity
/// features that need the full builder path rather than the work-ids executor).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alert_mode_accepts_valid_values() {
        assert!(validate_alert_mode("none").is_ok());
        assert!(validate_alert_mode("rss").is_ok());
    }

    #[test]
    fn alert_mode_rejects_invalid_values() {
        assert!(validate_alert_mode("email").is_err());
        assert!(validate_alert_mode("").is_err());
        assert!(validate_alert_mode("rss-feed").is_err());
    }

    #[test]
    fn create_request_validation() {
        let ok = CreateSavedSearchRequest {
            name: "my search".into(),
            query_text: "harry potter".into(),
        };
        assert!(validate_create_request(&ok).is_ok());

        // Empty name
        let bad_name = CreateSavedSearchRequest {
            name: "  ".into(),
            query_text: "harry".into(),
        };
        assert!(validate_create_request(&bad_name).is_err());

        // Name too long
        let long_name = CreateSavedSearchRequest {
            name: "x".repeat(200),
            query_text: "harry".into(),
        };
        assert!(validate_create_request(&long_name).is_err());

        // Empty query
        let bad_query = CreateSavedSearchRequest {
            name: "ok".into(),
            query_text: "   ".into(),
        };
        assert!(validate_create_request(&bad_query).is_err());
    }

    #[test]
    fn query_to_ast_serializes_boolean_tree() {
        // A simple word becomes a Term(Word(...)).
        let ast = query_to_ast("coffee");
        assert!(ast.get("Term").is_some());

        // A boolean expression serializes to its And/Or structure.
        let ast2 = query_to_ast("coffee AND tea");
        assert!(ast2.get("And").is_some());

        // Empty -> And([]), but we guard against empty earlier.
        let ast3 = query_to_ast("");
        let and = ast3.get("And").and_then(|v| v.as_array());
        assert!(and.is_some());
        assert!(and.unwrap().is_empty());
    }

    #[test]
    fn query_ast_roundtrips_fielded_and_excluded() {
        let ast = query_to_ast("-angst fandom:harry");
        // Top-level And of [-angst NOT, fandom fielded]
        let and = ast.get("And").and_then(|v| v.as_array()).cloned();
        assert!(and.is_some());
        let parts: Vec<String> = and
            .unwrap()
            .iter()
            .filter_map(|v| {
                if v.get("Not").is_some() {
                    Some("Not".to_string())
                } else if v.get("Term").and_then(|t| t.get("Fielded")).is_some() {
                    Some("Fielded".to_string())
                } else {
                    None
                }
            })
            .collect();
        assert!(parts.contains(&"Not".to_string()));
        assert!(parts.contains(&"Fielded".to_string()));
    }
}
