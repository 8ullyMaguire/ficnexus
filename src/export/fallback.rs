use md5::{Digest, Md5};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::time::{Duration, timeout};

use crate::cache::EType;
use crate::export::ExportError;

const FICHUB_API_TIMEOUT: u64 = 60;
const FICHUB_EPUB_TIMEOUT: u64 = 120;

/// Fichub.net API response for EPUB generation.
#[derive(Debug, Deserialize)]
struct FichubResponse {
    epub_url: Option<String>,
    error: Option<String>,
    #[allow(dead_code)]
    status: Option<String>,
}

/// Fichub.net fallback: use fichub.net's hosted service to generate an EPUB,
/// then optionally convert it to the desired format via Calibre.
///
/// # Arguments
/// * `url` — the original fic URL (passed to fichub.net)
/// * `target_format` — desired output format (epub, mobi, pdf, txt, html)
/// * `tmp_dir` — temporary directory for intermediate files
/// * `calibre_container` — Docker container name for ebook-convert, or empty
///
/// Returns `(path_to_file, md5_hex)` on success.
pub async fn fichub_fallback(
    url: &str,
    target_format: &EType,
    tmp_dir: &Path,
    calibre_container: &str,
) -> Result<(PathBuf, String), ExportError> {
    // ---- Step 1: Request EPUB from fichub.net API -------------------------
    let epub_path = request_epub_from_fichub(url, tmp_dir).await?;

    // If the user wanted EPUB, we're done
    if *target_format == EType::Epub {
        let data = std::fs::read(&epub_path)?;
        let md5_hex = Md5::digest(&data)
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        return Ok((epub_path, md5_hex));
    }

    // ---- Step 2: Convert via Calibre for non-EPUB formats -----------------
    let format_str = target_format.as_str();
    crate::export::convert::convert_epub(&epub_path, format_str, calibre_container, tmp_dir).await
}

/// Request an EPUB from fichub.net's API.
///
/// Calls `GET https://fichub.net/api/epub?q=<url>`, polls for the epub_url,
/// downloads the EPUB, and returns the local path.
async fn request_epub_from_fichub(url: &str, tmp_dir: &Path) -> Result<PathBuf, ExportError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(FICHUB_API_TIMEOUT))
        .build()
        .map_err(|e| ExportError::NetworkError(format!("Failed to build HTTP client: {e}")))?;

    // ---- Call the API -----------------------------------------------------
    let api_url = format!("https://fichub.net/api/epub?q={}", urlencoding::encode(url));

    let response = timeout(
        Duration::from_secs(FICHUB_API_TIMEOUT),
        client.get(&api_url).send(),
    )
    .await
    .map_err(|_| {
        ExportError::NetworkError(format!(
            "fichub.net API request timed out after {FICHUB_API_TIMEOUT}s"
        ))
    })?
    .map_err(|e| ExportError::NetworkError(format!("fichub.net API request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        return Err(ExportError::NetworkError(format!(
            "fichub.net API returned status {status}"
        )));
    }

    let body: FichubResponse = response.json().await.map_err(|e| {
        ExportError::NetworkError(format!("Failed to parse fichub.net response: {e}"))
    })?;

    if let Some(err) = &body.error {
        return Err(ExportError::NetworkError(format!(
            "fichub.net API error: {err}"
        )));
    }

    let epub_url = body.epub_url.ok_or_else(|| {
        ExportError::NetworkError("fichub.net API returned no epub_url".to_string())
    })?;

    // ---- Download the EPUB ------------------------------------------------
    download_file(&client, &epub_url, tmp_dir, "fichub_output.epub").await
}

/// Download a file from a URL and return its local path.
async fn download_file(
    client: &reqwest::Client,
    url: &str,
    tmp_dir: &Path,
    filename: &str,
) -> Result<PathBuf, ExportError> {
    let response = timeout(
        Duration::from_secs(FICHUB_EPUB_TIMEOUT),
        client.get(url).send(),
    )
    .await
    .map_err(|_| {
        ExportError::NetworkError(format!("Download timed out after {FICHUB_EPUB_TIMEOUT}s"))
    })?
    .map_err(|e| ExportError::NetworkError(format!("Download request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        return Err(ExportError::NetworkError(format!(
            "Download from {url} returned status {status}"
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| ExportError::NetworkError(format!("Failed to read download body: {e}")))?;

    let out_path = tmp_dir.join(filename);
    std::fs::write(&out_path, &bytes)?;

    Ok(out_path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fichub_response_deserialize_success() {
        let json = r#"{"epub_url":"https://fichub.net/example.epub","status":"ok"}"#;
        let resp: FichubResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.epub_url.as_deref(),
            Some("https://fichub.net/example.epub")
        );
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_fichub_response_deserialize_error() {
        let json = r#"{"error":"not found"}"#;
        let resp: FichubResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.error.as_deref(), Some("not found"));
        assert!(resp.epub_url.is_none());
    }

    #[test]
    fn test_fichub_response_deserialize_empty() {
        let json = r#"{}"#;
        let resp: FichubResponse = serde_json::from_str(json).unwrap();
        assert!(resp.epub_url.is_none());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_fichub_response_minimal() {
        let json = r#"{"epub_url":"https://example.com/book.epub"}"#;
        let resp: FichubResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.epub_url.as_deref(),
            Some("https://example.com/book.epub")
        );
    }

    #[test]
    fn test_api_url_construction() {
        let url = "https://archiveofourown.org/works/12345";
        let api_url = format!("https://fichub.net/api/epub?q={}", urlencoding::encode(url));
        assert!(api_url.contains("q=https%3A%2F%2F"));
    }
}
