use axum::{Json, response::IntoResponse};
use serde_json::json;

/// API description: GET /api/
///
/// This is the whole public surface of the service. If an endpoint is not
/// listed here it is not served — see `docs/EXPORTER-SCOPE.md`.
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "ficnexus exporter",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Give it a story URL, get a downloadable file.",
        "endpoints": {
            "/api/epub": {
                "method": "GET",
                "params": {
                    "q": "URL of the fanfiction (or a fic hash previously returned)",
                    "format": "optional output format (default epub); see /api/formats"
                },
                "description": "Scrape the work, build the file and return its download links"
            },
            "/api/epub/convert": {
                "method": "GET",
                "params": {
                    "q": "URL of the fanfiction",
                    "format": "epub|html|txt|md|pdf|mobi|azw3|kepub|docx|fb2"
                },
                "description": "Build a specific output format directly"
            },
            "/api/meta": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Metadata only (title, author, words, chapters, tags, status)"
            },
            "/api/formats": {
                "method": "GET",
                "description": "Supported output formats and which need Calibre"
            },
            "/api/health": {
                "method": "GET",
                "description": "Health check (database + Redis ping)"
            },
            "/cache/{etype}/{url_id}": {
                "method": "GET",
                "params": { "h": "content hash from /api/epub" },
                "description": "Download a built file from the cache"
            },
            "/cache/{etype}/{url_id}/{fname}": {
                "method": "GET",
                "description": "Download a built file, hash verified against the filename"
            },
            "/api/pow/challenge": {
                "method": "GET",
                "description": "Proof-of-work challenge for rate-limited clients"
            },
            "/api/pow/solve": {
                "method": "POST",
                "description": "Submit a solved proof-of-work challenge"
            }
        }
    }))
}
