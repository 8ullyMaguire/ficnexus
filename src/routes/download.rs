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
async fn create_bundle_from_works(
    state: &AppState,
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
            .lookup_authed(&state.http_client, work_url, &creds, None)
            .await
            .map_err(|e| AppError::ScrapeError(format!("failed to scrape {work_url}: {e}")))?;

        let chapters = scraper
            .fetch_chapters(&state.http_client, &meta)
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

    let formats = if let Some(uid) = auth.user_id {
        crate::db::queries::get_user_format_preferences(&state.db, uid)
            .await
            .unwrap_or_else(|_| vec!["epub".to_string()])
    } else {
        vec!["epub".to_string()]
    };

    let bundle = create_bundle_from_works(
        &state,
        &work_urls,
        &format!("{}_works", author_name),
        auth.user_id,
        formats,
    )
    .await?;

    let (status, headers, body) = bundle.into_response();
    Ok((status, headers, Body::from(body)))
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
        &work_urls,
        &format!("series_{}", series_id),
        auth.user_id,
        formats,
    )
    .await?;

    let (status, headers, body) = bundle.into_response();
    Ok((status, headers, Body::from(body)))
}
