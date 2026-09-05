use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;
use std::io::Write;
use std::sync::Arc;

use crate::error::AppError;
use crate::export;
use crate::limiter::Tier;
use crate::routes::auth::AuthUser;
use crate::scrape::sites::ao3::Ao3Scraper;
use crate::server::AppState;

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

/// Shared helper: scrape multiple work URLs into EPUBs, bundle them as a ZIP,
/// and return the ZIP bytes + a suggested filename.
///
/// `user_id`: when Some (authenticated download), the user's stored site
/// credentials for each work's host are loaded and passed through the
/// registry's `lookup_authed` login pre-pass. When None (anonymous), empty
/// credentials are passed — login-requiring sites return AuthRequired,
/// exactly as before this feature.
async fn create_zip_from_works(
    state: &AppState,
    work_urls: &[String],
    zip_basename: &str,
    user_id: Option<i32>,
) -> Result<(Vec<u8>, String), AppError> {
    let tmp_dir = state.config.tmp_dir.clone();
    let mut epub_paths: Vec<(String, std::path::PathBuf)> = Vec::new();

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
            .lookup_authed(&state.http_client, work_url, &creds, None)
            .await
            .map_err(|e| AppError::ScrapeError(format!("failed to scrape {work_url}: {e}")))?;

        let chapters = scraper
            .fetch_chapters(&state.http_client, &meta)
            .await
            .map_err(|e| {
                AppError::ScrapeError(format!("failed to fetch chapters for {}: {e}", meta.title))
            })?;

        let (epub_path, _hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir)
            .await
            .map_err(|e| {
                AppError::ExportError(format!("failed to generate EPUB for {}: {e}", meta.title))
            })?;

        // Sanitize filename for the EPUB inside the ZIP
        let safe_name: String = meta
            .title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let safe_name: String = safe_name.split_whitespace().collect::<Vec<_>>().join("_");
        let zip_entry_name = if safe_name.is_empty() {
            format!("work_{}.epub", meta.url_id)
        } else {
            format!("{}.epub", safe_name)
        };

        epub_paths.push((zip_entry_name, epub_path));
    }

    // Create ZIP file from all EPUBs
    let zip_uuid = uuid::Uuid::new_v4();
    let zip_path = tmp_dir.join(format!("{}.zip", zip_uuid));

    let zip_file = std::fs::File::create(&zip_path)?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);

    for (entry_name, epub_path) in &epub_paths {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip_writer.start_file(entry_name, options).map_err(|e| {
            AppError::Internal(format!("failed to start ZIP entry '{entry_name}': {e}"))
        })?;

        let epub_data = std::fs::read(epub_path)
            .map_err(|e| AppError::Internal(format!("failed to read EPUB file: {e}")))?;
        zip_writer.write_all(&epub_data).map_err(|e| {
            AppError::Internal(format!("failed to write ZIP entry '{entry_name}': {e}"))
        })?;
    }

    zip_writer
        .finish()
        .map_err(|e| AppError::Internal(format!("failed to finalize ZIP: {e}")))?;

    // Clean up individual EPUB temp files
    for (_, epub_path) in &epub_paths {
        let _ = std::fs::remove_file(epub_path);
    }

    // Read the ZIP into memory
    let zip_data = std::fs::read(&zip_path)
        .map_err(|e| AppError::Internal(format!("failed to read ZIP file: {e}")))?;

    // Clean up the ZIP temp file
    let _ = std::fs::remove_file(&zip_path);

    let zip_filename = format!("{}.zip", zip_basename);
    Ok((zip_data, zip_filename))
}

/// GET /api/download/author?url=<author_page_url>
///
/// Batch-download all works by an AO3 author as a ZIP file containing EPUBs.
pub async fn download_author_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DownloadAuthorQuery>,
    headers: axum::http::HeaderMap,
    auth: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    // ── Tiered rate limit (download tier) ─────────────────────────────
    enforce_download_rate_limit(&state, &headers).await?;

    let url = params.url.trim().to_string();

    // Validate: must be an AO3 author page
    if !Ao3Scraper::is_author_page(&url) {
        return Err(AppError::BadRequest(
            "only AO3 author pages (archiveofourown.org/users/{name}) are supported".to_string(),
        ));
    }

    // Step 1: List all work URLs from the author page
    let work_urls = Ao3Scraper::list_author_works(&state.http_client, &url)
        .await
        .map_err(|e| AppError::ScrapeError(format!("failed to list author works: {e}")))?;

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

    let (zip_data, zip_filename) = create_zip_from_works(
        &state,
        &work_urls,
        &format!("{}_works", author_name),
        auth.user_id,
    )
    .await?;

    let headers = HeaderMap::from_iter([
        (header::CONTENT_TYPE, "application/zip".parse().unwrap()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", zip_filename)
                .parse()
                .unwrap(),
        ),
    ]);

    Ok((StatusCode::OK, headers, Body::from(zip_data)))
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

    let (zip_data, zip_filename) = create_zip_from_works(
        &state,
        &work_urls,
        &format!("series_{}", series_id),
        auth.user_id,
    )
    .await?;

    let headers = HeaderMap::from_iter([
        (header::CONTENT_TYPE, "application/zip".parse().unwrap()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", zip_filename)
                .parse()
                .unwrap(),
        ),
    ]);

    Ok((StatusCode::OK, headers, Body::from(zip_data)))
}
