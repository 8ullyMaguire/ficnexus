use axum::{response::IntoResponse, Json};
use serde_json::json;

/// API docs page: GET /api/
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "ficnexus API",
        "version": "0.2.0",
        "endpoints": {
            "/api/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links for a fanfiction"
            },
            "/api/meta": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata only (no download links)"
            },
            "/api/remote": {
                "method": "GET",
                "description": "Get request source information"
            },
            "/api/search": {
                "method": "GET",
                "params": {
                    "q": "search query",
                    "include_tags": "tags to include (AND)",
                    "exclude_tags": "tags to exclude",
                    "min_words": "minimum word count",
                    "max_words": "maximum word count",
                    "sort": "sort order (kudos, comments, words, updated)"
                },
                "description": "Advanced search with tag filters, word count, and sort options"
            },
            "/api/auth/register": {
                "method": "POST",
                "params": { "username": "username", "password": "password" },
                "description": "Register a new user account"
            },
            "/api/auth/login": {
                "method": "POST",
                "params": { "username": "username", "password": "password" },
                "description": "Login and get JWT token"
            },
            "/api/auth/me": {
                "method": "GET",
                "auth": "Bearer token",
                "description": "Get current user info"
            },
            "/api/bookmarks": {
                "method": "GET/POST/DELETE",
                "auth": "Bearer token",
                "description": "Manage user bookmarks"
            },
            "/api/ratings": {
                "method": "GET/POST",
                "auth": "Bearer token",
                "description": "Rate works (1=like, -1=dislike)"
            },
            "/api/works/{url_id}/comments": {
                "method": "GET/POST",
                "description": "Threaded comments for a work"
            },
            "/api/tags": {
                "method": "GET",
                "params": { "url_id": "fic URL ID" },
                "description": "Get tags for a fic"
            },
            "/api/recommendations": {
                "method": "GET",
                "params": { "url_id": "fic URL ID", "n": "number of recommendations" },
                "description": "Get recommendations based on a fic"
            },
            "/cache/{type}/{url_id}": {
                "method": "GET",
                "description": "Download cached export file (epub, html, mobi, pdf)"
            },
            "/opds": {
                "method": "GET",
                "description": "OPDS catalog root"
            }
        }
    }))
}
