//! Fichub.net v0 API fallback client.
//!
//! When a native scraper gets a Cloudflare-partial page or a markup change
//! (selector miss → words=0 / chapters=1), the fichub.net export API can
//! still return accurate metadata for the same story. This module is a tiny
//! read-only client for `GET /api/v0/epub?q=<url>` mirroring the behavior of
//! the standalone `fichub-cli-rs` crate (same endpoint, same response shape).
//!
//! It is intentionally independent of the published crate so the server does
//! not need a second HTTP client dependency; reqwest is already present.

use crate::ScrapeError;
use serde::{Deserialize, Serialize};

/// Endpoint for the fichub.net v0 export API.
const FICHUB_NET_API: &str = "https://fichub.net/api/v0/epub";

/// Metadata returned by the fichub.net API (only the fields we consume).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FallbackMeta {
    pub title: String,
    pub author: String,
    pub words: i64,
    pub chapters: i64,
    pub status: String,
    pub updated: String,
    pub description: String,
    pub id: String,
    pub source_id: i64,
    pub author_url: String,
    #[serde(rename = "rawExtendedMeta")]
    pub raw_extended_meta: Option<RawExtendedMeta>,
    pub author_id: i64,
    #[serde(rename = "authorLocalId")]
    pub author_local_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct RawExtendedMeta {
    pub rated: Option<String>,
    pub language: Option<String>,
    pub genres: Option<String>,
    #[serde(rename = "raw_fandom")]
    pub raw_fandom: Option<String>,
    pub characters: Option<String>,
    pub published: Option<String>,
    pub updated: Option<String>,
    pub reviews: Option<String>,
    pub favorites: Option<String>,
    pub follows: Option<String>,
    pub words: Option<String>,
}

/// Top-level /api/v0/epub response (snake_case keys on the wire).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct EpubResponse {
    err: i64,
    fixits: Vec<String>,
    meta: Option<Meta>,
    info: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Meta {
    author: String,
    #[serde(rename = "authorId")]
    author_id: i64,
    #[serde(rename = "authorLocalId")]
    author_local_id: String,
    #[serde(rename = "authorUrl")]
    author_url: String,
    chapters: i64,
    description: String,
    id: String,
    #[serde(rename = "rawExtendedMeta")]
    raw_extended_meta: Option<RawExtendedMeta>,
    source: String,
    #[serde(rename = "sourceId")]
    source_id: i64,
    status: String,
    title: String,
    updated: String,
    words: i64,
}

/// Fetch metadata for `url` from the fichub.net v0 API.
pub async fn fetch_metadata(
    client: &reqwest::Client,
    url: &str,
) -> Result<FallbackMeta, ScrapeError> {
    let resp = client
        .get(FICHUB_NET_API)
        .query(&[("q", url)])
        .header("User-Agent", "fichub_cli/1.0.0 (fichub-cli-rs fallback)")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(format!("fichub.net API: {e}")))?;

    if !resp.status().is_success() {
        return Err(ScrapeError::Network(format!(
            "fichub.net API returned HTTP {}",
            resp.status()
        )));
    }

    let body = resp
        .text()
        .await
        .map_err(|e| ScrapeError::Network(format!("fichub.net API body: {e}")))?;

    let parsed: EpubResponse = serde_json::from_str(&body)
        .map_err(|e| ScrapeError::ParseError(format!("fichub.net API JSON: {e}")))?;

    if parsed.err != 0 {
        return Err(ScrapeError::NotFound);
    }

    let meta = parsed
        .meta
        .ok_or_else(|| ScrapeError::ParseError("fichub.net API: missing meta".into()))?;

    Ok(FallbackMeta {
        title: meta.title,
        author: meta.author,
        words: meta.words,
        chapters: meta.chapters,
        status: meta.status,
        updated: meta.updated,
        description: meta.description,
        id: meta.id,
        source_id: meta.source_id,
        author_url: meta.author_url,
        raw_extended_meta: meta.raw_extended_meta,
        author_id: meta.author_id,
        author_local_id: meta.author_local_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_api_shape() {
        let body = r#"{
            "epub_url": "/cache/epub/yKrqX55e/Dodging.epub?h=abc",
            "err": 0,
            "fixits": [],
            "hashes": {"epub": "9a19162d2a2b302fe3ff52ad01f6e066"},
            "info": "Dodging Prison...\n711600 words in 69 chapters",
            "meta": {
                "author": "LeadVonE",
                "authorId": 492,
                "authorLocalId": "6791440",
                "authorUrl": "/u/6791440/",
                "chapters": 69,
                "created": "2015-10-23T15:02:23",
                "description": "Description here",
                "extraMeta": "Rated: Fiction M - Language: English",
                "id": "yKrqX55e",
                "rawExtendedMeta": {
                    "category": "Harry Potter",
                    "chapters": "69",
                    "characters": "Harry P., Hermione G.",
                    "crossover": false,
                    "fandom_stubs": ["harry-potter"],
                    "favorites": "21322",
                    "follows": "24097",
                    "genres": "Adventure/Romance",
                    "id": "yKrqX55e",
                    "language": "English",
                    "published": "2015-10-23",
                    "rated": "M",
                    "raw_fandom": "Harry Potter",
                    "reviews": "10051",
                    "updated": "2024-09-13",
                    "words": "711600"
                },
                "source": "https://www.fanfiction.net/s/11574569/1/",
                "sourceId": 2,
                "status": "ongoing",
                "title": "Dodging Prison and Stealing Witches - Revenge is Best Served Raw",
                "updated": "2024-09-13T19:23:18",
                "words": 711600
            }
        }"#;
        let parsed: EpubResponse = serde_json::from_str(body).expect("parse");
        assert_eq!(parsed.err, 0);
        let meta = parsed.meta.expect("meta present");
        assert_eq!(meta.words, 711600);
        assert_eq!(meta.chapters, 69);
        assert_eq!(
            meta.title,
            "Dodging Prison and Stealing Witches - Revenge is Best Served Raw"
        );
        assert_eq!(meta.author, "LeadVonE");
        let raw = meta.raw_extended_meta.expect("raw meta");
        assert_eq!(raw.rated.as_deref(), Some("M"));
        assert_eq!(raw.genres.as_deref(), Some("Adventure/Romance"));
    }

    #[test]
    fn errors_on_err_flag() {
        let body = r#"{"err": -1, "fixits": [], "meta": null, "info": null}"#;
        let parsed: EpubResponse = serde_json::from_str(body).expect("parse");
        assert_eq!(parsed.err, -1);
        assert!(parsed.meta.is_none());
    }

    #[test]
    fn handles_missing_fields() {
        let body = r#"{"err": 0, "fixits": [], "meta": {"title": "T", "author": "A"}}"#;
        let parsed: EpubResponse = serde_json::from_str(body).expect("parse");
        let meta = parsed.meta.expect("meta");
        assert_eq!(meta.title, "T");
        assert_eq!(meta.words, 0); // default
        assert_eq!(meta.chapters, 0);
    }
}
