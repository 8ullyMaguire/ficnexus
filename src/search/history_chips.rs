//! Search-history "chips" endpoint.
//!
//! `GET /api/search/history/chips` returns the signed-in user's top-8 most
//! frequent search queries, ranked by occurrence count (ties broken by
//! most-recent-first). The frontend renders these as clickable autocomplete
//! chips above the search box so a user can re-run a favourite query in one
//! tap. Anonymous callers get an empty list (no historical identity to key
//! off of).

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// `GET /api/search/history/chips`
///
/// Returns the authenticated user's most frequent search queries as
/// `{"query": "...", "count": N}` chips, limited to 8. Anonymous users
/// receive an empty `chips` array. The query is best-effort: a DB error
/// is logged but yields an empty (200 OK) response rather than failing
/// the request, since chips are a non-critical UI affordance.
pub async fn search_history_chips_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let chips = match crate::db::queries::recent_search_chips(&state.db, auth.user_id).await {
        Ok(rows) => rows
            .into_iter()
            .map(|(query, count)| {
                json!({ "query": query, "count": count })
            })
            .collect::<Vec<Value>>(),
        Err(e) => {
            tracing::warn!(error = %e, "failed to load search history chips");
            Vec::new()
        }
    };

    Ok(Json(json!({ "err": 0, "chips": chips })))
}
