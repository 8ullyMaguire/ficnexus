//! SoFurry.com native adapter (furry archive; members-only stories).
//!
//! Story URL: `https://sofurry.com/s/{id}`. Metadata comes from the
//! turbo-stream `.data` endpoint (`url + ".data"`), decoded via the
//! [`super::turbo_stream`] module. Chapter bodies are fetched from
//! `displayUrl` (S3, expiring links).
//!
//! Login: members-only stories return 401; POST `_token` (csrf meta from
//! `/fe/auth/sofurry`) + `email`/`password` → `/login`.

use async_trait::async_trait;
use regex_lite::Regex;

use super::{http, login, turbo_stream};
use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};

pub struct SoFurryScraper;

impl SoFurryScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"sofurry\.com/s/([a-zA-Z0-9]+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for SoFurryScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("sofurry.com/s/")
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        let auth_page = "https://sofurry.com/fe/auth/sofurry";
        let html = http::fetch(client, auth_page).await?;
        let token = login::csrf_meta(&html)
            .ok_or_else(|| ScrapeError::ParseError("sofurry: no csrf token".into()))?;
        login::post_login(
            client,
            "https://sofurry.com/login",
            &[
                ("_token", token),
                ("email", creds.username.clone()),
                ("password", creds.password.clone()),
            ],
        )
        .await?;
        Ok(())
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("sofurry: bad url".into()))?;
        let data_url = format!("{url}.data");
        let data = http::fetch(client, &data_url).await?;
        let decoded = turbo_stream::decode(&data)
            .map_err(|e| ScrapeError::ParseError(format!("sofurry: {e}")))?;

        // Path: routes/submission.$id.data.submission
        let sub = decoded
            .pointer("/routes/submission.$id/data/submission")
            .ok_or_else(|| ScrapeError::ParseError("sofurry: no submission".into()))?;

        let title = sub
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if title.is_empty() {
            return Err(ScrapeError::ParseError("sofurry: no title".into()));
        }
        let author = sub
            .pointer("/author/username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let author_url = sub
            .pointer("/author/handle")
            .and_then(|v| v.as_str())
            .map(|h| format!("https://sofurry.com/u/{h}"))
            .unwrap_or_default();
        let author_local_id = sub
            .pointer("/author/handle")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let desc = sub
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let published = sub
            .get("publishedAt")
            .and_then(|v| v.as_str())
            .map(parse_iso)
            .unwrap_or(0);
        let rating_num = sub.get("rating").and_then(|v| v.as_i64()).unwrap_or(0);
        let rating = if rating_num >= 20 {
            "Adult".to_string()
        } else if rating_num >= 10 {
            "Mature".to_string()
        } else {
            "Clean".to_string()
        };

        // Chapters from content array.
        let chapters = sub
            .get("content")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len() as i32)
            .unwrap_or(1);

        let mut words = 0i64;
        if let Some(arr) = sub.get("content").and_then(|v| v.as_array()) {
            for c in arr {
                if let Some(w) = c.pointer("/meta/wordCount").and_then(|v| v.as_i64()) {
                    words += w;
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("sf_{story_id}"),
            title,
            author,
            chapters: chapters.max(1),
            words,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!("rating={};", rating)),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let data_url = format!("{}.data", meta.source);
        let data = http::fetch(client, &data_url).await?;
        let decoded = turbo_stream::decode(&data)
            .map_err(|e| ScrapeError::ParseError(format!("sofurry: {e}")))?;
        let sub = decoded
            .pointer("/routes/submission.$id/data/submission")
            .ok_or_else(|| ScrapeError::ParseError("sofurry: no submission".into()))?;
        let story_title = sub
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut chapters = Vec::new();
        if let Some(arr) = sub.get("content").and_then(|v| v.as_array()) {
            for (i, c) in arr.iter().enumerate() {
                let chap_title = c
                    .get("title")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&story_title)
                    .to_string();
                let display_url = c
                    .get("displayUrl")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = if display_url.is_empty() {
                    String::new()
                } else {
                    match http::fetch(client, &display_url).await {
                        Ok(body) => body,
                        Err(_) => String::new(),
                    }
                };
                chapters.push(Chapter {
                    chapter_id: i as i32 + 1,
                    title: chap_title,
                    content,
                });
            }
        }
        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("sofurry: no chapters".into()));
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

fn parse_iso(s: &str) -> i64 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return dt.timestamp_millis();
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = SoFurryScraper;
        assert!(s.can_handle("https://sofurry.com/s/abc123"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            SoFurryScraper::story_id("https://sofurry.com/s/abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(SoFurryScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_iso_dates() {
        assert!(parse_iso("2024-01-31T10:00:00.000Z") > 0);
        assert_eq!(parse_iso("garbage"), 0);
    }

    #[test]
    fn rating_mapping() {
        let rating = |n: i64| {
            if n >= 20 {
                "Adult".to_string()
            } else if n >= 10 {
                "Mature".to_string()
            } else {
                "Clean".to_string()
            }
        };
        assert_eq!(rating(25), "Adult");
        assert_eq!(rating(12), "Mature");
        assert_eq!(rating(5), "Clean");
    }
}
