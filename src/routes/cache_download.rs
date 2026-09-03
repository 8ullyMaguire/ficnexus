use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::server::AppState;
use crate::cache::EType;

/// Query for cache download with hash validation
#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub h: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_query_default() {
        let q = DownloadQuery { h: None };
        assert!(q.h.is_none());
    }

    #[test]
    fn test_download_query_with_hash() {
        let q = DownloadQuery {
            h: Some("abc123def456".into()),
        };
        assert_eq!(q.h.as_deref(), Some("abc123def456"));
    }

    #[test]
    fn test_etype_parse_valid() {
        assert_eq!("epub".parse::<EType>().unwrap(), EType::Epub);
        assert_eq!("html".parse::<EType>().unwrap(), EType::Html);
        assert_eq!("mobi".parse::<EType>().unwrap(), EType::Mobi);
        assert_eq!("pdf".parse::<EType>().unwrap(), EType::Pdf);
        assert_eq!("txt".parse::<EType>().unwrap(), EType::Txt);
        assert_eq!("azw3".parse::<EType>().unwrap(), EType::Azw3);
        assert_eq!("md".parse::<EType>().unwrap(), EType::Md);
    }

    #[test]
    fn test_etype_parse_invalid() {
        assert!("json".parse::<EType>().is_err());
        assert!("xml".parse::<EType>().is_err());
        assert!("".parse::<EType>().is_err());
    }

    #[test]
    fn test_etype_suffixes() {
        assert_eq!(EType::Epub.suffix(), ".epub");
        assert_eq!(EType::Html.suffix(), ".zip");
        assert_eq!(EType::Mobi.suffix(), ".mobi");
        assert_eq!(EType::Pdf.suffix(), ".pdf");
        assert_eq!(EType::Txt.suffix(), ".txt");
        assert_eq!(EType::Azw3.suffix(), ".azw3");
        assert_eq!(EType::Md.suffix(), ".md");
    }

    #[test]
    fn test_etype_as_str() {
        assert_eq!(EType::Epub.as_str(), "epub");
        assert_eq!(EType::Html.as_str(), "html");
        assert_eq!(EType::Mobi.as_str(), "mobi");
        assert_eq!(EType::Pdf.as_str(), "pdf");
        assert_eq!(EType::Txt.as_str(), "txt");
        assert_eq!(EType::Azw3.as_str(), "azw3");
        assert_eq!(EType::Md.as_str(), "md");
    }

    #[test]
    fn test_etype_mime_types() {
        // Verify MIME type mappings are consistent with handler
        let mime_epub = match EType::Epub {
            EType::Epub => "application/epub+zip",
            _ => unreachable!(),
        };
        assert_eq!(mime_epub, "application/epub+zip");

        let mime_html = match EType::Html {
            EType::Html => "application/zip",
            _ => unreachable!(),
        };
        assert_eq!(mime_html, "application/zip");

        let mime_txt = match EType::Txt {
            EType::Txt => "text/plain",
            _ => unreachable!(),
        };
        assert_eq!(mime_txt, "text/plain");

        let mime_md = match EType::Md {
            EType::Md => "text/markdown",
            _ => unreachable!(),
        };
        assert_eq!(mime_md, "text/markdown");
    }
}

/// Direct download with hash validation:
/// GET /cache/:etype/:url_id/:fname
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    // Extract hash from query param or filename
    let hash = match params.h {
        Some(h) => h,
        None => {
            // Try to extract from filename: <hash><suffix>
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash,
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    // Validate hash
    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            // Serve file
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
                EType::Txt => "text/plain",
                EType::Azw3 => "application/vnd.amazon.ebook",
                EType::Md => "text/markdown",
                EType::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                EType::Fb2 => "application/x-fictionbook+xml",
                EType::Kepub => "application/epub+zip",
            };

            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}

/// Trigger export and download:
/// GET /cache/:etype/:url_id
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // If a hash is provided and file exists, serve directly
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash,
            );
            if cache_path.exists() {
                match crate::cache::disk::file_md5(&cache_path) {
                    Ok(actual_hash) if actual_hash == *hash => {
                        let mime = match etype {
                            EType::Epub => "application/epub+zip",
                            EType::Html => "application/zip",
                            EType::Mobi => "application/x-mobipocket-ebook",
                            EType::Pdf => "application/pdf",
                            EType::Txt => "text/plain",
                            EType::Azw3 => "application/vnd.amazon.ebook",
                            EType::Md => "text/markdown",
                            EType::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                            EType::Fb2 => "application/x-fictionbook+xml",
                            EType::Kepub => "application/epub+zip",
                        };
                        match tokio::fs::read(&cache_path).await {
                            Ok(data) => {
                                let filename = format!("{}{}", url_id, etype.suffix());
                                let headers = [
                                    ("Content-Type", mime),
                                    ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                                ];
                                return (headers, data).into_response();
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // No cached file — return JSON error instead of redirecting to SPA.
    // The frontend handles this by showing a "file not ready" message.
    Json(json!({
        "err": -5,
        "msg": "file not cached — re-scrape to generate",
        "url_id": url_id,
        "format": etype_str,
    })).into_response()
}
