use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use base64::Engine as _;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::io::Write;
use std::sync::Arc;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::AppError;
use crate::export;
use crate::limiter::Tier;
use crate::routes::auth::AuthUser;
use crate::scrape::sites::ao3::Ao3Scraper;
use crate::scrape::sites::xenforo::XenForoScraper;
use crate::server::AppState;


/// Result of a bulk-download bundle operation.
/// - Single: 1 work × 1 format → stream directly (no ZIP overhead).
/// - Bundle:  N works × M formats → ZIP with per-work subdirectories.
pub enum DownloadBundle {
    Single { data: Vec<u8>, filename: String, mime: &'static str },
    Bundle { data: Vec<u8>, filename: String },
}

impl DownloadBundle {
    pub fn into_response(self) -> (StatusCode, HeaderMap, Vec<u8>) {
        match self {
            DownloadBundle::Single { data, filename, mime } => {
                let headers = HeaderMap::from_iter([
                    (header::CONTENT_TYPE, mime.parse().unwrap()),
                    (header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename).parse().unwrap()),
                ]);
                (StatusCode::OK, headers, data)
            }
            DownloadBundle::Bundle { data, filename } => {
                let headers = HeaderMap::from_iter([
                    (header::CONTENT_TYPE, "application/zip".parse().unwrap()),
                    (header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename).parse().unwrap()),
                ]);
                (StatusCode::OK, headers, data)
            }
        }
    }
}

/// Map user-facing format name to an export function.
/// Formats requiring Calibre (pdf, mobi, azw3) return None.
async fn export_one_format(
    meta: &fanfic_scrapers::FicMetadata,
    chapters: &[fanfic_scrapers::Chapter],
    tmp_dir: &std::path::Path,
    format: &str,
) -> Option<(String, std::path::PathBuf)> {
    match format {
        "epub" => export::epub::create_epub(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("epub".into(), p)),
        "html" => export::html_bundle::create_html_bundle(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("html".into(), p)),
        "txt"  => export::txt::create_txt(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("txt".into(), p)),
        "md"   => export::md::create_md(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("md".into(), p)),
        "docx" => export::docx::create_docx(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("docx".into(), p)),
        "kepub" => export::kepub::create_kepub(meta, chapters, tmp_dir).await.ok().map(|(p, _)| ("kepub".into(), p)),
        "pdf" | "mobi" | "azw3" => {
            tracing::debug!("format {} requires Calibre, skipping", format);
            None
        }
        _ => {
            tracing::warn!("unknown export format: {}", format);
            None
        }
    }
}

/// Sanitize a string for use as a ZIP entry filename.
fn safe_basename(title: &str, url_id: &str) -> String {
    let s: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let s: String = s.split_whitespace().collect::<Vec<_>>().join("_");
    if s.is_empty() { format!("work_{}", url_id) } else { s }
}

/// Query parameters for batch download requests
#[derive(Debug, Deserialize)]
pub struct DownloadAuthorQuery {
    pub url: String,
}

/// Query parameters for series download requests
#[derive(Debug, Deserialize)]
pub struct DownloadSeriesQuery {
    pub url: String,
}

/// Shared helper: scrape multiple work URLs, export in the given formats,
/// and return either a single file (1 work × 1 format) or a ZIP
/// (N works × M formats) with a suggested filename.
///
/// `user_id`: when Some (authenticated download), the user's stored site
/// credentials for each work's host are loaded and passed through the
/// registry's `lookup_authed` login pre-pass. When None (anonymous), empty
/// credentials are passed — login-requiring sites return AuthRequired,
/// exactly as before this feature.
///
/// `client`: the HTTP client to scrape through. Batch-download callers pass a
/// per-request client with a private cookie jar (see `isolated_scrape_client`);
/// the series endpoint passes the shared client (no logins happen there).
async fn create_bundle_from_works(
    state: &AppState,
    client: &reqwest::Client,
    work_urls: &[String],
    basename: &str,
    user_id: Option<i32>,
    formats: Vec<String>,
) -> Result<DownloadBundle, AppError> {
    let tmp_dir = state.config.tmp_dir.clone();
    let formats = if formats.is_empty() { vec!["epub".into()] } else { formats };
    let mut bundle_entries: Vec<(String, std::path::PathBuf)> = Vec::new();

    for work_url in work_urls {
        let scraper = state
            .scraper_registry
            .find_specific_or_fff(work_url)
            .ok_or_else(|| AppError::ScrapeError(format!("no scraper for URL: {work_url}")))?;

        // Load the requesting user's stored credentials for this host (if any).
        let mut creds: Vec<fanfic_scrapers::SiteCredentials> = Vec::new();
        if let Some(uid) = user_id {
            let host = fanfic_scrapers::sites::http::host_of(work_url);
            if let Some((username, password_enc)) =
                crate::db::queries::get_user_site_credential(&state.db, uid, &host).await?
            {
                let key =
                    std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
                if let Some(password) = crate::crypto::decrypt(&password_enc, &key) {
                    creds.push(fanfic_scrapers::SiteCredentials {
                        domain: host,
                        username,
                        password,
                        is_adult: false,
                    });
                }
            }
        }

        let meta = state
            .scraper_registry
            .lookup_authed(client, work_url, &creds, None)
            .await
            .map_err(|e| AppError::ScrapeError(format!("failed to scrape {work_url}: {e}")))?;

        let chapters = scraper
            .fetch_chapters(client, &meta)
            .await
            .map_err(|e| {
                AppError::ScrapeError(format!("failed to fetch chapters for {}: {e}", meta.title))
            })?;

        // Export in each requested format.
        let safe_name = safe_basename(&meta.title, &meta.url_id);
        for fmt in &formats {
            if let Some((ext, path)) = export_one_format(&meta, &chapters, &tmp_dir, fmt).await {
                let entry_name = if formats.len() == 1 && work_urls.len() == 1 {
                    format!("{}.{}", safe_name, ext)
                } else {
                    format!("{}/{}.{}", safe_name, safe_name, ext)
                };
                bundle_entries.push((entry_name, path));
            }
        }
    }

    if bundle_entries.is_empty() {
        return Err(AppError::BadRequest(
            "no formats available for the requested works".into(),
        ));
    }

    // 1 work × 1 format → stream directly, no ZIP overhead.
    if work_urls.len() == 1 && formats.len() == 1 {
        let (entry_name, path) = bundle_entries.remove(0);
        let data = std::fs::read(&path)
            .map_err(|e| AppError::Internal(format!("failed to read file: {e}")))?;
        let _ = std::fs::remove_file(&path);
        let mime = match entry_name.rsplit('.').next().unwrap_or("") {
            "epub"  => "application/epub+zip",
            "html"  => "application/zip",
            "txt"   => "text/plain",
            "md"    => "text/markdown",
            "docx"  => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "kepub" => "application/epub+zip",
            _       => "application/octet-stream",
        };
        return Ok(DownloadBundle::Single { data, filename: entry_name, mime });
    }

    // N works × M formats → ZIP with per-work subdirectories.
    let zip_uuid = uuid::Uuid::new_v4();
    let zip_path = tmp_dir.join(format!("{}.zip", zip_uuid));
    let zip_file = std::fs::File::create(&zip_path)?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);

    for (entry_name, entry_path) in &bundle_entries {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip_writer.start_file(entry_name, options).map_err(|e| {
            AppError::Internal(format!("failed to start ZIP entry '{entry_name}': {e}"))
        })?;
        let data = std::fs::read(entry_path)
            .map_err(|e| AppError::Internal(format!("failed to read file: {e}")))?;
        zip_writer.write_all(&data).map_err(|e| {
            AppError::Internal(format!("failed to write ZIP entry '{entry_name}': {e}"))
        })?;
    }

    zip_writer.finish()
        .map_err(|e| AppError::Internal(format!("failed to finalize ZIP: {e}")))?;

    // Clean up temp files.
    for (_, path) in &bundle_entries {
        let _ = std::fs::remove_file(path);
    }

    let zip_data = std::fs::read(&zip_path)
        .map_err(|e| AppError::Internal(format!("failed to read ZIP: {e}")))?;
    let _ = std::fs::remove_file(&zip_path);

    let zip_filename = format!("{}.zip", basename);
    Ok(DownloadBundle::Bundle { data: zip_data, filename: zip_filename })
}

/// GET /api/download/author?url=<author_page_url>
///
/// Batch-download all works by an author as a ZIP file containing EPUBs.
/// Supports AO3 and XenForo (QQ, SpaceBattles, SV) author pages.
pub async fn download_author_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadAuthorQuery>,
    headers: axum::http::HeaderMap,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // ── Tiered rate limit (download tier) ─────────────────────────────
    enforce_download_rate_limit(&state, &headers).await?;

    let url = params.url.trim().to_string();

    // FIX 5 (shared cookie jar): this request logs in with the caller's site
    // credentials, so scrape through a private jar — never the shared client.
    let scrape_client = isolated_scrape_client();

    // Determine which scraper handles this author page
    let work_urls = if Ao3Scraper::is_author_page(&url) {
        Ao3Scraper::list_author_works(&scrape_client, &url)
            .await
            .map_err(|e| AppError::ScrapeError(format!("failed to list author works: {e}")))?
    } else if XenForoScraper::is_author_page(&url) {
        // Load user credentials for XenForo login (needed for NSFW content)
        let mut creds: Vec<fanfic_scrapers::SiteCredentials> = Vec::new();
        if let Some(uid) = auth.user_id {
            let host = fanfic_scrapers::sites::http::host_of(&url);
            if let Some((username, password_enc)) =
                crate::db::queries::get_user_site_credential(&state.db, uid, &host).await?
            {
                let key =
                    std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
                if let Some(password) = crate::crypto::decrypt(&password_enc, &key) {
                    creds.push(fanfic_scrapers::SiteCredentials {
                        domain: host,
                        username,
                        password,
                        is_adult: true,
                    });
                }
            }
        }
        XenForoScraper::list_author_works(&scrape_client, &url, &creds)
            .await
            .map_err(|e| AppError::ScrapeError(format!("failed to list author works: {e}")))?
    } else {
        return Err(AppError::BadRequest(
            "unsupported author page URL; only AO3 and XenForo (QQ, SB, SV) are supported"
                .to_string(),
        ));
    };

    if work_urls.is_empty() {
        return Err(AppError::NotFound("no works found for this author".into()));
    }

    tracing::info!(
        "Batch download: found {} works for author URL: {}",
        work_urls.len(),
        url
    );

    // Extract author name from URL for the filename
    let author_name = url
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("author");

    let formats = if let Some(uid) = auth.user_id {
        crate::db::queries::get_user_format_preferences(&state.db, uid)
            .await
            .unwrap_or_else(|_| vec!["epub".to_string()])
    } else {
        vec!["epub".to_string()]
    };

    let bundle = create_bundle_from_works(
        &state,
        &scrape_client,
        &work_urls,
        &format!("{author_name}_works"),
        auth.user_id,
        formats,
    )
    .await?;

    let (status, headers, body) = bundle.into_response();
    Ok((status, headers, Body::from(body)))
}

// ── SSE streaming author download ────────────────────────────────────

/// Progress event emitted during streaming author download.
#[derive(Serialize)]
struct DownloadProgress {
    current: usize,
    total: usize,
    title: String,
    /// Running list of completed work titles
    completed: Vec<String>,
}

/// Completion event emitted when the download bundle is ready.
/// Carries the file inline (base64) so the client never re-fetches —
/// the pre-fix flow re-hit `GET /download/author`, re-scraping everything.
#[derive(Serialize)]
struct DownloadComplete {
    filename: String,
    size: usize,
    mime: String,
    /// Base64-encoded file bytes (single work file or ZIP bundle).
    data: String,
}

/// SSE progress event type wrapper for the stream.
struct ProgressEvent {
    event_type: &'static str,
    data: String,
}

/// Query parameters for the streaming author download endpoint.
#[derive(Debug, Deserialize)]
pub struct DownloadAuthorStreamQuery {
    pub url: String,
    /// Optional JWT for EventSource clients, which cannot set HTTP headers.
    /// Falls back to anonymous when absent or invalid.
    pub token: Option<String>,
}

/// Build an isolated HTTP client for one batch-download request.
///
/// The shared `AppState.http_client` carries a global cookie jar: a XenForo
/// login performed for user A would leave session cookies behind for user B.
/// Batch flows log in with the *requesting* user's site credentials, so each
/// request gets a fresh jar that is dropped when the download finishes.
fn isolated_scrape_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build isolated scrape client")
}

/// Resolve the effective user for a batch download: the `Authorization`
/// header (via `AuthUser`) wins; `?token=` is the fallback for EventSource
/// clients that cannot set headers. Returns `AuthUser::default()` (anonymous)
/// when neither yields a valid token.
fn resolve_batch_user(auth: &AuthUser, token: Option<&str>) -> AuthUser {
    if auth.user_id.is_some() {
        return auth.clone();
    }
    if let Some(tok) = token {
        if let Some(user) = crate::routes::auth::auth_user_from_token(tok) {
            return user;
        }
    }
    AuthUser::default()
}

/// GET /api/download/author/stream?url=<author_page_url>&token=<jwt>
///
/// SSE endpoint that streams progress events during a batch author download.
/// `?token=` carries the JWT because EventSource cannot set headers; the
/// `Authorization` header path still works and wins when both are present.
/// Events:
/// - `progress`: `{current, total, title, completed}` — emitted per work
/// - `complete`: `{filename, size, mime, data}` — emitted when the ZIP is
///   ready; `data` is the base64-encoded file, so the client saves it
///   directly instead of re-fetching (no second scrape).
/// - `error`: `{message}` — emitted on failure
pub async fn download_author_stream_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadAuthorStreamQuery>,
    headers: axum::http::HeaderMap,
    auth: AuthUser,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let url = params.url.trim().to_string();
    // FIX 1 (?token= auth): EventSource sends ?token=, never a header.
    let effective = resolve_batch_user(&auth, params.token.as_deref());
    let user_id = effective.user_id;

    // FIX 5 (shared cookie jar): per-request client with a private jar, so a
    // XenForo login for this user never leaks session cookies to anyone else.
    let scrape_client = isolated_scrape_client();

    // Rate limit check
    let rate_check = enforce_download_rate_limit(&state, &headers).await;

    let (tx, rx) = tokio::sync::mpsc::channel::<ProgressEvent>(256);

    // Spawn the background download task
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // Keep the isolated client alive for the whole batch: logins and
        // session cookies stay inside this request's private jar.
        let client = &scrape_client;
        if let Err(e) = rate_check {
            let _ = tx_clone
                .send(ProgressEvent {
                    event_type: "error",
                    data: serde_json::to_string(&serde_json::json!({
                        "message": format!("Rate limited: {e}")
                    }))
                    .unwrap_or_default(),
                })
                .await;
            return;
        }

        // Step 1: List author works
        let work_urls = if Ao3Scraper::is_author_page(&url) {
            match Ao3Scraper::list_author_works(client, &url).await {
                Ok(u) => u,
                Err(e) => {
                    let _ = send_error(&tx_clone, &format!("Failed to list works: {e}")).await;
                    return;
                }
            }
        } else if XenForoScraper::is_author_page(&url) {
            let mut creds: Vec<fanfic_scrapers::SiteCredentials> = Vec::new();
            if let Some(uid) = user_id {
                let host = fanfic_scrapers::sites::http::host_of(&url);
                if let Ok(Some((username, password_enc))) =
                    crate::db::queries::get_user_site_credential(&state_clone.db, uid, &host).await
                {
                    let key = std::env::var("JWT_SECRET")
                        .unwrap_or_else(|_| "fichub-dev-secret".into());
                    if let Some(password) = crate::crypto::decrypt(&password_enc, &key) {
                        creds.push(fanfic_scrapers::SiteCredentials {
                            domain: host,
                            username,
                            password,
                            is_adult: true,
                        });
                    }
                }
            }
            match XenForoScraper::list_author_works(client, &url, &creds).await {
                Ok(u) => u,
                Err(e) => {
                    let _ = send_error(&tx_clone, &format!("Failed to list works: {e}")).await;
                    return;
                }
            }
        } else {
            let _ = send_error(&tx_clone, "Unsupported author page URL").await;
            return;
        };

        let total = work_urls.len();
        if total == 0 {
            let _ = send_error(&tx_clone, "No works found for this author").await;
            return;
        }

        // Get user format preferences
        let formats = if let Some(uid) = user_id {
            crate::db::queries::get_user_format_preferences(&state_clone.db, uid)
                .await
                .unwrap_or_else(|_| vec!["epub".to_string()])
        } else {
            vec!["epub".to_string()]
        };

        let tmp_dir = state_clone.config.tmp_dir.clone();
        let formats = if formats.is_empty() {
            vec!["epub".to_string()]
        } else {
            formats
        };
        let mut bundle_entries: Vec<(String, std::path::PathBuf)> = Vec::new();
        let mut completed_titles: Vec<String> = Vec::new();

        // Step 2: Process each work with progress updates
        for (i, work_url) in work_urls.iter().enumerate() {
            // Check if the client disconnected (receiver dropped).
            if tx_clone.is_closed() {
                tracing::info!("Batch download cancelled (client disconnected) after {i} works");
                break;
            }

            let scraper = match state_clone
                .scraper_registry
                .find_specific_or_fff(work_url)
            {
                Some(s) => s,
                None => continue,
            };

            // Load credentials for this host
            let mut creds: Vec<fanfic_scrapers::SiteCredentials> = Vec::new();
            if let Some(uid) = user_id {
                let host = fanfic_scrapers::sites::http::host_of(work_url);
                if let Ok(Some((username, password_enc))) =
                    crate::db::queries::get_user_site_credential(&state_clone.db, uid, &host).await
                {
                    let key = std::env::var("JWT_SECRET")
                        .unwrap_or_else(|_| "fichub-dev-secret".into());
                    if let Some(password) = crate::crypto::decrypt(&password_enc, &key) {
                        creds.push(fanfic_scrapers::SiteCredentials {
                            domain: host,
                            username,
                            password,
                            is_adult: false,
                        });
                    }
                }
            }

            // Lookup metadata
            let meta = match state_clone
                .scraper_registry
                .lookup_authed(client, work_url, &creds, None)
                .await
            {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Skipping {work_url}: {e}");
                    continue;
                }
            };

            // Send progress event
            let _ = tx_clone
                .send(ProgressEvent {
                    event_type: "progress",
                    data: serde_json::to_string(&DownloadProgress {
                        current: i + 1,
                        total,
                        title: meta.title.clone(),
                        completed: completed_titles.clone(),
                    })
                    .unwrap_or_default(),
                })
                .await;

            // Fetch chapters
            let chapters = match scraper
                .fetch_chapters(client, &meta)
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Skipping chapters for {}: {e}", meta.title);
                    continue;
                }
            };

            // Export in each format
            let safe_name = safe_basename(&meta.title, &meta.url_id);
            for fmt in &formats {
                if let Some((ext, path)) = export_one_format(&meta, &chapters, &tmp_dir, fmt).await {
                    let entry_name = if formats.len() == 1 && work_urls.len() == 1 {
                        format!("{safe_name}.{ext}")
                    } else {
                        format!("{safe_name}/{safe_name}.{ext}")
                    };
                    bundle_entries.push((entry_name, path));
                }
            }

            completed_titles.push(meta.title.clone());

            // Rate limit between works — but bail early if client gone.
            if !tx_clone.is_closed() {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
        }

        // Step 3: Build the final bundle
        if bundle_entries.is_empty() {
            let _ = send_error(&tx_clone, "No formats available for the requested works").await;
            return;
        }

        let author_name = url
            .trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("author");

        let bundle = if work_urls.len() == 1 && formats.len() == 1 {
            let (entry_name, path) = bundle_entries.remove(0);
            let data = match std::fs::read(&path) {
                Ok(d) => d,
                Err(e) => {
                    let _ =
                        send_error(&tx_clone, &format!("Failed to read exported file: {e}")).await;
                    return;
                }
            };
            let _ = std::fs::remove_file(&path);
            let mime = match entry_name.rsplit('.').next().unwrap_or("") {
                "epub" => "application/epub+zip",
                "html" => "application/zip",
                "txt" => "text/plain",
                "md" => "text/markdown",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "kepub" => "application/epub+zip",
                _ => "application/octet-stream",
            };
            DownloadBundle::Single {
                data,
                filename: entry_name,
                mime,
            }
        } else {
            let zip_uuid = uuid::Uuid::new_v4();
            let zip_path = tmp_dir.join(format!("{zip_uuid}.zip"));
            let zip_file = match std::fs::File::create(&zip_path) {
                Ok(f) => f,
                Err(e) => {
                    let _ =
                        send_error(&tx_clone, &format!("Failed to create ZIP: {e}")).await;
                    return;
                }
            };
            let mut zip_writer = zip::ZipWriter::new(zip_file);
            for (entry_name, entry_path) in &bundle_entries {
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                if let Err(e) = zip_writer.start_file(entry_name, options) {
                    tracing::warn!("ZIP start_file error: {e}");
                    continue;
                }
                if let Ok(data) = std::fs::read(entry_path) {
                    let _ = zip_writer.write_all(&data);
                }
            }
            if let Err(e) = zip_writer.finish() {
                let _ = send_error(&tx_clone, &format!("Failed to finalize ZIP: {e}")).await;
                return;
            }
            for (_, path) in &bundle_entries {
                let _ = std::fs::remove_file(path);
            }
            let zip_data = match std::fs::read(&zip_path) {
                Ok(d) => d,
                Err(e) => {
                    let _ =
                        send_error(&tx_clone, &format!("Failed to read ZIP: {e}")).await;
                    return;
                }
            };
            let _ = std::fs::remove_file(&zip_path);
            DownloadBundle::Bundle {
                data: zip_data,
                filename: format!("{author_name}_works.zip"),
            }
        };

        // Send complete event — file bytes inline (base64), so the client
        // must NOT re-fetch GET /download/author (that re-scrapes the batch).
        let (_status, headers, data) = bundle.into_response();
        let mime = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let filename = headers
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(|cd| {
                cd.split("filename=\"")
                    .nth(1)
                    .and_then(|s| s.split('"').next())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("{author_name}_works.zip"));
        let complete = DownloadComplete {
            filename,
            size: data.len(),
            mime,
            data: base64::engine::general_purpose::STANDARD.encode(&data),
        };
        let _ = tx_clone
            .send(ProgressEvent {
                event_type: "complete",
                data: serde_json::to_string(&complete).unwrap_or_default(),
            })
            .await;
        drop(tx_clone);
    });

    // FIX 3 (busy-loop Stream): ReceiverStream parks the task on the channel
    // instead of wake_by_ref() spinning. The stream ends when the sender
    // (tx_clone, moved into the task) is dropped.
    let stream = ReceiverStream::new(rx).map(|evt: ProgressEvent| {
        Ok(Event::default().event(evt.event_type).data(evt.data))
    });
    // Keep the original sender alive until the task finishes: dropping `tx`
    // here would close the channel early only if the task's clone were also
    // gone — the task holds tx_clone, so this just releases our handle.
    drop(tx);

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

/// Helper to send an error event and close the channel.
async fn send_error(
    tx: &tokio::sync::mpsc::Sender<ProgressEvent>,
    msg: &str,
) -> Result<(), tokio::sync::mpsc::error::SendError<ProgressEvent>> {
    tx.send(ProgressEvent {
        event_type: "error",
        data: serde_json::to_string(&serde_json::json!({ "message": msg }))
            .unwrap_or_default(),
    })
    .await
}

/// Enforce the download-tier rate limit for a batch-download request.
///
/// Keys on the real client IP (X-Forwarded-For first hop, per the deployment's
/// nginx proxy) AND `(ip, client_id)` when the client identifies itself, so
/// shared-NAT readers aren't punished. Shadowbanned client_ids get a much
/// stricter bucket (friction, not a hard block). Redis errors fail open so a
/// hiccup never blocks a real download.
async fn enforce_download_rate_limit(
    state: &Arc<AppState>,
    headers: &axum::http::HeaderMap,
) -> Result<(), AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let xff = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok());
    let real_ip = crate::limiter::client_ip_from_headers(
        xff,
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    match state
        .rate_limiter
        .check(real_ip, client_id.as_deref(), Tier::Download)
        .await
    {
        crate::limiter::TieredRateLimitResult::Wait(secs) => Err(AppError::RateLimited(secs)),
        crate::limiter::TieredRateLimitResult::Allowed => Ok(()),
    }
}

/// GET /api/download/series?url=<series_page_url>
///
/// Batch-download all works in an AO3 series as a ZIP file containing EPUBs.
pub async fn download_series_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadSeriesQuery>,
    headers: axum::http::HeaderMap,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // ── Tiered rate limit (download tier) ─────────────────────────────
    enforce_download_rate_limit(&state, &headers).await?;

    let url = params.url.trim().to_string();

    // Validate: must be an AO3 series page
    if !Ao3Scraper::is_series_page(&url) {
        return Err(AppError::BadRequest(
            "only AO3 series pages (archiveofourown.org/series/{id}) are supported".to_string(),
        ));
    }

    // Step 1: List all work URLs from the series page
    let work_urls = Ao3Scraper::list_series_works(&state.http_client, &url)
        .await
        .map_err(|e| AppError::ScrapeError(format!("failed to list series works: {e}")))?;

    if work_urls.is_empty() {
        return Err(AppError::NotFound("no works found for this series".into()));
    }

    tracing::info!(
        "Series download: found {} works for series URL: {}",
        work_urls.len(),
        url
    );

    // Extract series ID from URL for the filename
    let series_id = url
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("series");

    let formats = if let Some(uid) = auth.user_id {
        crate::db::queries::get_user_format_preferences(&state.db, uid)
            .await
            .unwrap_or_else(|_| vec!["epub".to_string()])
    } else {
        vec!["epub".to_string()]
    };

    let bundle = create_bundle_from_works(
        &state,
        &state.http_client,
        &work_urls,
        &format!("series_{}", series_id),
        auth.user_id,
        formats,
    )
    .await?;

    let (status, headers, body) = bundle.into_response();
    Ok((status, headers, Body::from(body)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_query_accepts_token_param() {
        // FIX 1 regression: EventSource clients authenticate via ?token=.
        let q: DownloadAuthorStreamQuery = serde_urlencoded::from_str(
            "url=https%3A%2F%2Fforum.questionablequesting.com%2Fmembers%2Fbob.1%2F&token=jwt-here",
        )
        .expect("stream query must parse");
        assert_eq!(
            q.url,
            "https://forum.questionablequesting.com/members/bob.1/"
        );
        assert_eq!(q.token.as_deref(), Some("jwt-here"));
    }

    #[test]
    fn stream_query_token_is_optional() {
        let q: DownloadAuthorStreamQuery =
            serde_urlencoded::from_str("url=https%3A%2F%2Farchiveofourown.org%2Fusers%2Fbob")
                .expect("stream query must parse without token");
        assert!(q.token.is_none());
    }

    #[test]
    fn batch_user_prefers_header_auth_over_token() {
        // A valid header identity wins even when ?token= is garbage.
        let header_user = AuthUser {
            user_id: Some(7),
            username: Some("header-user".into()),
            role: 1,
            level: 10,
        };
        let resolved = resolve_batch_user(&header_user, Some("bogus-token"));
        assert_eq!(resolved.user_id, Some(7));
    }

    #[test]
    fn batch_user_falls_back_to_anonymous_on_bad_token() {
        // Anonymous header + invalid token → anonymous (never an error: the
        // endpoint serves public content anonymously by design).
        let resolved = resolve_batch_user(&AuthUser::default(), Some("not-a-jwt"));
        assert_eq!(resolved.user_id, None);
        let resolved = resolve_batch_user(&AuthUser::default(), None);
        assert_eq!(resolved.user_id, None);
    }

    #[test]
    fn complete_event_carries_file_inline() {
        // FIX 2 regression: the complete event must embed the file so the
        // client never re-fetches GET /download/author (double scrape).
        let bundle = DownloadBundle::Bundle {
            data: vec![1, 2, 3],
            filename: "author_works.zip".into(),
        };
        let (_status, headers, data) = bundle.into_response();
        let mime = headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        assert_eq!(mime, "application/zip");
        let complete = DownloadComplete {
            filename: "author_works.zip".into(),
            size: data.len(),
            mime,
            data: base64::engine::general_purpose::STANDARD.encode(&data),
        };
        let json = serde_json::to_string(&complete).expect("serializes");
        assert!(json.contains("\"data\":\"AQID\""));
        assert!(json.contains("\"mime\":\"application/zip\""));
    }
}
