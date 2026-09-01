// ── Send-to-Kindle route ──────────────────────────────────────────────
// POST /api/send-to-kindle
//
// Generates an EPUB for a fic (by `url` or `url_id`) and emails it to the
// authenticated user's configured Kindle address (users.kindle_email),
// falling back to their account email (users.email) when unset. The EPUB
// is also cached and recorded in export_log, matching the export pipeline.

use axum::{
    extract::State,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::routes::auth::AuthUser;
use crate::server::AppState;

/// Request body for POST /api/send-to-kindle.
/// Exactly one of `url` (scrape + export) or `url_id` (export from DB) is used.
#[derive(Debug, Deserialize)]
pub struct SendToKindleReq {
    pub url: Option<String>,
    pub url_id: Option<String>,
}

/// Generate an EPUB for `url_id` and email it to `to`.
/// Shared by both request shapes (url_id is resolved from the URL first).
async fn build_and_send(
    state: &AppState,
    to: &str,
    url_id: &str,
    _title: &str,
) -> Result<Value, AppError> {
    // Resolve chapters: prefer a cached fic_info entry, else scrape live.
    let (meta, chapters) = if let Some(fic) =
        crate::db::queries::get_fic_info(&state.db, url_id).await?
    {
        let meta = crate::scrape::FicMetadata {
            url_id: fic.id.clone(),
            title: fic.title.clone(),
            author: fic.author.clone(),
            chapters: fic.chapters,
            words: fic.words,
            desc: fic.description,
            published: fic.fic_created.timestamp_millis(),
            updated: fic.fic_updated.timestamp_millis(),
            status: fic.status.clone(),
            source: fic.source.clone(),
            source_id: fic.source_id.unwrap_or(0),
            author_id: fic.author_id.unwrap_or(0),
            author_url: fic.author_url.clone().unwrap_or_default(),
            author_local_id: fic.author_local_id.clone().unwrap_or_default(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        };
        // Cached metadata only: fetch chapters from the source.
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(&meta.source)
            .ok_or_else(|| {
                AppError::BadRequest(format!("no scraper for {}", meta.source))
            })?;
        let chapters = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => {
                state
                    .heal
                    .record_failure(
                        &meta.source,
                        Some(&meta.url_id),
                        &crate::heal::classifier::ErrorKind::from(&e),
                        Some(&e.to_string()),
                        None,
                    )
                    .await;
                return Err(AppError::ScrapeError(format!("failed to fetch chapters: {e}")));
            }
        };
        (meta, chapters)
    } else {
        // Not in DB: scrape fresh
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(url_id)
            .ok_or_else(|| AppError::NotFound(format!("fic {url_id} not found")))?;
        let meta = match scraper.lookup(&state.http_client, url_id).await {
            Ok(m) => m,
            Err(e) => {
                state
                    .heal
                    .record_failure(
                        url_id,
                        None,
                        &crate::heal::classifier::ErrorKind::from(&e),
                        Some(&e.to_string()),
                        None,
                    )
                    .await;
                return Err(AppError::ScrapeError(e.to_string()));
            }
        };
        let chapters = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => {
                state
                    .heal
                    .record_failure(
                        &meta.source,
                        Some(&meta.url_id),
                        &crate::heal::classifier::ErrorKind::from(&e),
                        Some(&e.to_string()),
                        None,
                    )
                    .await;
                return Err(AppError::ScrapeError(e.to_string()));
            }
        };
        (meta, chapters)
    };

    // Generate the EPUB
    let (epub_path, epub_hash) =
        crate::export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
            .await
            .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Cache it (same flow as the export handler)
    let cache_dest = crate::cache::disk::cache_path(
        &state.config.cache_dir,
        &crate::cache::EType::Epub,
        &meta.url_id,
        &epub_hash,
    );
    if let Err(e) = crate::cache::disk::move_to_cache(&epub_path, &cache_dest) {
        tracing::warn!("send-to-kindle: failed to cache EPUB: {e}");
    }

    // Log the export (reuses the export_log table)
    let version = state.config.export_version;
    let input_hash = meta
        .content_hash
        .clone()
        .unwrap_or_else(|| "upstream".to_string());
    crate::db::queries::insert_export_log(
        &state.db,
        &meta.url_id,
        version,
        "epub",
        &input_hash,
        &epub_hash,
    )
    .await?;

    // Sanitized attachment name: <slug>.epub
    let slug: String = meta
        .title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_");
    let attachment_name = if slug.is_empty() {
        format!("{}.epub", meta.url_id)
    } else {
        format!("{}.epub", slug)
    };

    let subject = format!("{} by {}", meta.title, meta.author);
    let body = format!(
        "Your fic is attached.\n\nTitle: {}\nAuthor: {}\nURL: {}\n\nSent from FicNexus.",
        meta.title, meta.author, meta.source
    );

    let mail = crate::services::mailer::KindleMail {
        to: to.to_string(),
        subject,
        body,
        attachment_path: epub_path.clone(),
        attachment_name,
    };

    // The mailer may be the real SMTP transport (blocking I/O) or a test
    // mock. Run it off the async runtime so a real relay never blocks a
    // worker; the mock returns instantly so spawn_blocking overhead is
    // negligible.
    let mailer = state.mailer.clone_box();
    tokio::task::spawn_blocking(move || mailer.send_kindle(&mail))
        .await
        .map_err(|e| AppError::Internal(format!("mailer task failed: {e}")))?
        .map_err(|e| AppError::Internal(format!("failed to send email: {e}")))?;

    // Clean up the temp EPUB
    let _ = std::fs::remove_file(&epub_path);

    Ok(json!({
        "err": 0,
        "url_id": meta.url_id,
        "to": to,
        "msg": "EPUB sent to your Kindle",
    }))
}

/// POST /api/send-to-kindle
pub async fn send_to_kindle_handler(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(req): Json<SendToKindleReq>,
) -> Result<Json<Value>, AppError> {
    let user_id = user
        .user_id
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Exactly one of url / url_id must be provided
    let resolved_url_id = match (&req.url, &req.url_id) {
        (Some(url), None) => {
            if !url.starts_with("http") {
                return Err(AppError::BadRequest("url must be an http(s) fic URL".to_string()));
            }
            let scraper = state
                .scraper_registry
                .find_specific_or_fff(url)
                .ok_or_else(|| AppError::BadRequest(format!("unsupported URL: {url}")))?;
            scraper
                .lookup(&state.http_client, url)
                .await
                .map_err(|e| AppError::ScrapeError(e.to_string()))?
                .url_id
        }
        (None, Some(url_id)) => {
            if url_id.is_empty() {
                return Err(AppError::BadRequest("url_id must not be empty".to_string()));
            }
            url_id.clone()
        }
        _ => {
            return Err(AppError::BadRequest("provide exactly one of url or url_id".to_string()));
        }
    };

    // Load the user's email + kindle_email from the DB
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT COALESCE(email, ''), kindle_email FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let (account_email, kindle_email) = row
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let to = kindle_email
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(account_email.clone());

    if to.trim().is_empty() {
        return Err(AppError::BadRequest("no email on file — add an email or a Kindle address to your account".to_string()));
    }

    // Graceful invalid-email handling: reject before any EPUB work so the
    // user gets a clear 400 instead of a mid-send failure.
    if let Err(e) = crate::services::mailer::validate_email(&to) {
        return Err(AppError::BadRequest(format!("invalid email: {e}")));
    }

    let title = format!("Fic {resolved_url_id}");
    let resp = build_and_send(&state, &to, &resolved_url_id, &title).await?;
    Ok(Json(resp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_to_kindle_req_accepts_url_or_url_id() {
        let by_url: SendToKindleReq = serde_json::from_value(json!({
            "url": "https://archiveofourown.org/works/123"
        }))
        .unwrap();
        assert_eq!(by_url.url.as_deref(), Some("https://archiveofourown.org/works/123"));
        assert!(by_url.url_id.is_none());

        let by_id: SendToKindleReq = serde_json::from_value(json!({
            "url_id": "a1b2c3"
        }))
        .unwrap();
        assert_eq!(by_id.url_id.as_deref(), Some("a1b2c3"));
        assert!(by_id.url.is_none());
    }

    #[test]
    fn send_to_kindle_req_allows_empty_body() {
        // Both fields optional so the handler can return a clear error.
        let empty: SendToKindleReq = serde_json::from_value(json!({})).unwrap();
        assert!(empty.url.is_none() && empty.url_id.is_none());
    }
}

// ── DB-gated tests (require a live Postgres: `source .env`) ───────────
#[cfg(test)]
mod db_tests {

    #[test]
    fn kindle_email_column_exists_and_updates() {
        let url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("skipping: DATABASE_URL not set");
                return;
            }
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Run the whole DB interaction inside the runtime so the pool's
        // background maintenance task has a Tokio context. sqlx's own
        // pool-connect hook also runs lazily here, so the connect_lazy
        // handle stays inside the same runtime.
        rt.block_on(async {
            let pool = match sqlx::PgPool::connect_lazy(&url) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("skipping: cannot connect to DB: {e}");
                    return;
                }
            };
            // Column exists (migration 013 applied)
            let has_col = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (
                    SELECT 1 FROM information_schema.columns
                    WHERE table_name = 'users' AND column_name = 'kindle_email'
                )",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(has_col, "users.kindle_email column must exist");

            // Insert a throwaway user, set kindle_email, read it back
            let username = format!("kindle_test_{}", uuid::Uuid::new_v4());
            let id: i32 = sqlx::query_scalar(
                "INSERT INTO users (username, password_hash, email, kindle_email)
                 VALUES ($1, 'x', 'kindle_test@example.com', 'kindle@example.com')
                 RETURNING id",
            )
            .bind(&username)
            .fetch_one(&pool)
            .await
            .unwrap();

            sqlx::query("UPDATE users SET kindle_email = 'new.kindle@example.com' WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();

            let (email, kindle): (String, Option<String>) =
                sqlx::query_as("SELECT email, kindle_email FROM users WHERE id = $1")
                    .bind(id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(email, "kindle_test@example.com");
            assert_eq!(kindle.as_deref(), Some("new.kindle@example.com"));

            // Cleanup
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .unwrap();

            pool.close().await;
        });
    }
}
