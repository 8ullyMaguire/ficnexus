use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::db::queries;
use crate::scrape::author_link::LlmClient;

use async_trait::async_trait;

/// Adapts FicNexus's [`crate::services::ollama::OllamaClient`] to the crate's
/// [`LlmClient`] trait for author-link extraction. Captures the chat model
/// name (the client's own `model` field is the embedding model).
struct OllamaLlmAdapter<'a> {
    ollama: &'a crate::services::ollama::OllamaClient,
    chat_model: &'a str,
}

#[async_trait]
impl LlmClient for OllamaLlmAdapter<'_> {
    async fn generate_json(&self, prompt: &str) -> Result<String, String> {
        self.ollama
            .generate_json(prompt, self.chat_model)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Query parameters for meta requests
#[derive(Debug, Deserialize)]
pub struct MetaQuery {
    pub q: Option<String>,
}

/// Metadata-only handler: GET /api/meta?q=<url_or_hash>
/// Returns same response as epub but without generating/downloading files
pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // If q looks like a hash (not a URL), look up from fic_info directly
    if !query.starts_with("http") {
        return handle_hash_lookup_meta(&state, query).await;
    }

    // Find scraper (prefer native scraper, fall back to FanFicFare)
    let scraper = state.scraper_registry.find_specific_or_fff(query)
        .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {}", query)))?;

    // Lookup metadata only (no chapters fetch)
    let mut meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Author-link enrichment: if the scraper didn't return an author
    // profile URL (FanFicFare often omits it for newer sites), try our own
    // generic extraction, then an LLM fallback. Best-effort — never blocks.
    if meta.author_url.is_empty() && !meta.author.trim().is_empty() {
        match state.http_client.get(query).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(page_text) = resp.text().await {
                    let llm = OllamaLlmAdapter {
                        ollama: &state.ollama,
                        chat_model: &state.config.ollama_chat_model,
                    };
                    if let Some(author_url) = crate::scrape::author_link::ensure_author_url(
                        Some(&llm),
                        &meta.author_url,
                        &meta.author,
                        &page_text,
                    )
                    .await
                    {
                        meta.author_url = author_url;
                    }
                }
            }
            _ => {}
        }
    }

    // Auto-merge: find or create work for this source
    let auto_merge_result = crate::works::find_or_create_work(&state.db, &meta).await?;
    let work_id = auto_merge_result.work_id;

    // Check blacklists
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;

    let slug = {
        let sanitized: String = meta.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let re = regex_lite::Regex::new(r"_+").unwrap();
        let slug = re.replace_all(&sanitized, "_").to_string();
        format!("{}-{}", slug.trim_matches('_'), meta.url_id)
    };

    let notes: Vec<String> = if fic_blacklist.iter().any(|b| b.reason == 6) {
        vec!["This fic is greylisted.".to_string()]
    } else {
        vec![]
    };

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": format!("{} by {} - {} words, {} chapters", meta.title, meta.author, meta.words, meta.chapters),
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": {
            "id": meta.url_id,
            "work_id": work_id,
            "title": meta.title,
            "author": meta.author,
            "chapters": meta.chapters,
            "words": meta.words,
            "description": meta.desc,
            "status": meta.status,
            "source": meta.source,
            "created": chrono::DateTime::from_timestamp_millis(meta.published)
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
                .map(|d| d.to_rfc3339()).unwrap_or_default(),
            "extra_meta": meta.extra_meta,
            "raw_extended_meta": meta.raw_extended_meta,
            "author_url": meta.author_url,
            "author_local_id": meta.author_local_id,
            "source_id": meta.source_id,
            "author_id": meta.author_id,
        },
        "hashes": {},
        "urls": {},
        "epub_url": null,
        "html_url": null,
        "mobi_url": null,
        "pdf_url": null,
        "notes": notes,
    })))
}

/// Handle hash lookup for meta endpoint
async fn handle_hash_lookup_meta(
    state: &Arc<AppState>,
    hash: &str,
) -> Result<Json<Value>, AppError> {
    let fic = queries::get_fic_info(&state.db, hash).await?
        .ok_or_else(|| AppError::NotFound(format!("fic not found: {}", hash)))?;

    let slug = {
        let sanitized: String = fic.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let re = regex_lite::Regex::new(r"_+").unwrap();
        let slug = re.replace_all(&sanitized, "_").to_string();
        format!("{}-{}", slug.trim_matches('_'), fic.id)
    };

    let notes: Vec<String> = vec![];

    Ok(Json(json!({
        "err": 0,
        "q": &fic.source,
        "fixits": [],
        "info": format!("{} by {} - {} words, {} chapters", fic.title, fic.author, fic.words, fic.chapters),
        "url_id": fic.id,
        "work_id": fic.work_id,
        "slug": slug,
        "meta": {
            "id": fic.id,
            "work_id": fic.work_id,
            "title": fic.title,
            "author": fic.author,
            "chapters": fic.chapters,
            "words": fic.words,
            "description": fic.description,
            "status": fic.status,
            "source": fic.source,
            "created": fic.fic_created.to_rfc3339(),
            "updated": fic.fic_updated.to_rfc3339(),
            "extra_meta": fic.extra_meta,
            "raw_extended_meta": fic.raw_extended_meta,
            "author_url": fic.author_url,
            "author_local_id": fic.author_local_id,
            "source_id": fic.source_id,
            "author_id": fic.author_id,
        },
        "hashes": {},
        "urls": {},
        "epub_url": null,
        "html_url": null,
        "mobi_url": null,
        "pdf_url": null,
        "notes": notes,
    })))
}
