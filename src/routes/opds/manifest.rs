use axum::{extract::State, response::IntoResponse};
use serde_json::json;
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

use super::PageParams;

/// GET /opds/manifest — Minimal OPDS 2.0 / Web Publication Manifest (RWPM JSON).
///
/// Mirrors the 6 catalog sections exposed at the OPDS root, plus a `self` link
/// and an `opds:search` link. This lets Readium/PWDF clients navigate the
/// catalog natively without parsing the Atom feed.
pub async fn manifest(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<PageParams>,
) -> Result<impl IntoResponse, AppError> {
    let base_url = state.config.opds_base_url.as_deref();

    let abs = |path: &str| super::abs_url(path, base_url);

    let manifest = json!({
        "@context": [
            "https://schema.org",
            "https://www.w3.org/ns/wp-context"
        ],
        "type": "Catalog",
        "id": abs("/opds/manifest"),
        "url": abs("/opds"),
        "name": "FicNexus",
        "metadata": {
            "title": "FicNexus",
            "description": "Fanfiction archive and download platform",
            "language": "en"
        },
        "links": [
            {
                "rel": "self",
                "href": abs("/opds/manifest"),
                "type": "application/webpubmanifest+json"
            },
            {
                "rel": "search",
                "href": abs("/opds/search"),
                "type": "application/json"
            },
            {
                "rel": "manifest",
                "href": abs("/opds/manifest"),
                "type": "application/webpubmanifest+json"
            }
        ],
        "navigation": [
            {
                "type": "section",
                "name": "Recent Fics",
                "href": abs(&format!("/opds/new?page={}&per_page={}", params.page(), params.per_page())),
                "link": { "type": "application/atom+xml; profile=opds-catalog; kind=acquisition" }
            },
            {
                "type": "section",
                "name": "Popular Fics",
                "href": abs(&format!("/opds/popular?page={}&per_page={}", params.page(), params.per_page())),
                "link": { "type": "application/atom+xml; profile=opds-catalog; kind=acquisition" }
            },
            {
                "type": "section",
                "name": "Tags",
                "href": abs("/opds/tags"),
                "link": { "type": "application/atom+xml; profile=opds-catalog; kind=navigation" }
            },
            {
                "type": "section",
                "name": "Authors",
                "href": abs("/opds/authors"),
                "link": { "type": "application/atom+xml; profile=opds-catalog; kind=navigation" }
            },
            {
                "type": "section",
                "name": "Recommendations",
                "href": abs("/opds/recommendations/popular"),
                "link": { "type": "application/atom+xml; profile=opds-catalog; kind=acquisition" }
            },
            {
                "type": "section",
                "name": "Search",
                "href": abs("/opds/search"),
                "link": { "type": "application/opensearchdescription+xml" }
            }
        ]
    });

    let body =
        serde_json::to_string_pretty(&manifest).map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((
        [(
            axum::http::header::CONTENT_TYPE,
            "application/webpubmanifest+json",
        )],
        body,
    )
        .into_response())
}
