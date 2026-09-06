//! Wattpad.com native adapter (REST API based).
//!
//! Wattpad's public API is far more reliable than its HTML:
//! - Story info: `GET https://www.wattpad.com/api/v3/stories/{id}`
//!   returns title, user.name (author), completed, mature, description,
//!   createDate/modifyDate, tags, parts[] (chapter title/url), language.
//! - Chapter text: `GET https://www.wattpad.com/apiv2/storytext?id={chapterId}`
//!   returns the body HTML.

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct WattpadScraper;

const API_STORY: &str = "https://www.wattpad.com/api/v3/stories/{id}";
const API_STORYTEXT: &str = "https://www.wattpad.com/apiv2/storytext?id={id}";

impl WattpadScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        // /story/{id} or /{chapterId}-title
        let re = Regex::new(r"wattpad\.com/(?:story/)?(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for WattpadScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("wattpad.com/story/")
            || url.contains("wattpad.com/") && url.chars().any(|c| c.is_ascii_digit())
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("no story id in Wattpad URL".into()))?;

        let api_url = API_STORY.replace("{id}", &story_id);
        let body = http::fetch(client, &api_url).await?;
        let v: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| ScrapeError::ParseError(format!("wattpad api: {e}")))?;

        let title = v["title"].as_str().unwrap_or("").to_string();
        if title.is_empty() {
            return Err(ScrapeError::ParseError("wattpad: no title".into()));
        }
        let author = v["user"]["name"].as_str().unwrap_or("").to_string();
        let completed = v["completed"].as_bool().unwrap_or(false);
        let mature = v["mature"].as_bool().unwrap_or(false);
        let desc = v["description"].as_str().unwrap_or("").to_string();
        let status = if completed { "complete" } else { "ongoing" }.to_string();

        // Dates: ISO-8601 strings.
        let create = v["createDate"].as_str().unwrap_or("");
        let modify = v["modifyDate"].as_str().unwrap_or("");
        let published = parse_iso(create);
        let updated = parse_iso(modify);

        let mut chapters = 0i32;
        if let Some(parts) = v["parts"].as_array() {
            chapters = parts.len() as i32;
        }

        // Tags → extra_meta (genre list).
        let tags: Vec<String> = v["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|t| t.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(FicMetadata {
            url_id: format!("watt_{story_id}"),
            title,
            author: author.clone(),
            chapters,
            words: 0,
            desc,
            published: published.unwrap_or_else(now_ms),
            updated: updated.unwrap_or_else(now_ms),
            status,
            source: format!("https://www.wattpad.com/story/{story_id}"),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url: format!("https://www.wattpad.com/user/{author}"),
            author_local_id: author.clone(),
            content_hash: None,
            extra_meta: Some(format!("tags={};mature={};", tags.join(","), mature)),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let story_id = meta.source.rsplit('/').next().unwrap_or("").to_string();
        let api_url = API_STORY.replace("{id}", &story_id);
        let body = http::fetch(client, &api_url).await?;
        let v: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| ScrapeError::ParseError(format!("wattpad api: {e}")))?;

        let mut chapters = Vec::new();
        if let Some(parts) = v["parts"].as_array() {
            for (i, part) in parts.iter().enumerate() {
                let title = part["title"].as_str().unwrap_or("").to_string();
                let part_url = part["url"].as_str().unwrap_or("").to_string();
                // Chapter id = last numeric segment of the URL.
                let chapter_id = part_url
                    .rsplit('/')
                    .next()
                    .and_then(|s| s.split('-').next())
                    .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or("");
                let text_url = API_STORYTEXT.replace("{id}", chapter_id);
                let content = http::fetch(client, &text_url).await.unwrap_or_default();
                chapters.push(Chapter {
                    chapter_id: i as i32 + 1,
                    title,
                    content,
                });
            }
        }

        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("wattpad: no chapters".into()));
        }
        Ok(chapters)
    }

    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str,
    ) -> Result<Vec<crate::ExtractedTag>, ScrapeError> {
        Ok(vec![])
    }
}

fn parse_iso(s: &str) -> Option<i64> {
    let s = s.trim_end_matches('Z');
    // "2024-01-31T10:00:00" (strip fractional seconds if present)
    let s = s.split('.').next().unwrap_or(s);
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|dt| chrono::Utc.from_utc_datetime(&dt).timestamp_millis())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_story_id() {
        assert_eq!(
            WattpadScraper::extract_story_id("https://www.wattpad.com/story/1234567-title")
                .as_deref(),
            Some("1234567")
        );
        assert_eq!(
            WattpadScraper::extract_story_id("https://www.wattpad.com/987654-chapter-title")
                .as_deref(),
            Some("987654")
        );
    }

    #[test]
    fn can_handle_matches() {
        let s = WattpadScraper;
        assert!(s.can_handle("https://www.wattpad.com/story/1234567-title"));
        assert!(s.can_handle("https://www.wattpad.com/987654-chapter-title"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_iso_dates() {
        assert!(parse_iso("2024-01-31T10:00:00").is_some());
        assert!(parse_iso("2024-01-31T10:00:00.000Z").is_some());
        assert_eq!(parse_iso("garbage"), None);
    }
}
