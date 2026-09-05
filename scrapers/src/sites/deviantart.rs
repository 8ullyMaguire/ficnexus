//! DeviantArt.com native adapter (one-shot deviations/literature).
//!
//! dA literature entries are single-page (no chapters). Metadata:
//! - title: `h1`
//! - author: from the URL path (`/USERNAME/art/...`)
//! - date: first `<time>` (text like "Jan 31, 2024")
//! - tags: `a[href^="https://www.deviantart.com/tag"] span`
//! Body selectors (newest → oldest):
//! - `[data-editor-viewer="1"]`
//! - `[data-id="rich-content-viewer"]`
//! - `.legacy-journal`
//! Login/mature-gated deviations return AuthRequired.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct DeviantArtScraper;

#[async_trait]
impl SiteScraper for DeviantArtScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("deviantart.com/") && !url.contains("/gallery/") && !url.contains("/shop/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        // Mature gate detection (no login support in the crate).
        if html.contains("mature-content-filter")
            || html.contains("requiresMatureContentEnabled")
            || html.contains("only available for watchers")
        {
            return Err(ScrapeError::AuthRequired("dA login/mature gate".into()));
        }

        let doc = Html::parse_document(&html);

        // Title
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("h1") {
            title = doc
                .select(&h1_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("dA: no title".into()));
        }

        // Author from URL: /USERNAME/art/title
        let author = url
            .split("deviantart.com/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("")
            .to_string();

        // Published date from first <time>.
        let mut published = 0i64;
        if let Ok(time_sel) = Selector::parse("time") {
            if let Some(t) = doc.select(&time_sel).next() {
                published = parse_da_date(&t.text().collect::<String>()).unwrap_or(0);
            }
        }

        // Tags
        let mut tags: Vec<String> = Vec::new();
        if let Ok(tag_sel) = Selector::parse("a[href*='deviantart.com/tag'] span") {
            for el in doc.select(&tag_sel) {
                let t = el.text().collect::<String>().trim().to_string();
                if !t.is_empty() && !tags.contains(&t) {
                    tags.push(t);
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("da_{}", url.rsplit('/').next().unwrap_or("")),
            title,
            author: author.clone(),
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: if published > 0 { published } else { now },
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url: format!("https://www.deviantart.com/{author}"),
            author_local_id: author.clone(),
            content_hash: None,
            extra_meta: Some(format!("tags={};", tags.join(","))),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let content = fetch_chapter_text(client, &meta.source).await;
        if content.is_empty() {
            return Err(ScrapeError::ParseError("dA: no story text".into()));
        }
        Ok(vec![Chapter {
            chapter_id: 1,
            title: meta.title.clone(),
            content,
        }])
    }

    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str,
    ) -> Result<Vec<crate::ExtractedTag>, ScrapeError> {
        Ok(vec![])
    }
}

/// Fetch a dA page and extract the story body.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    // Remove comments to avoid false body matches.
    let selectors = [
        "[data-editor-viewer='1']",
        "[data-id='rich-content-viewer']",
        ".legacy-journal",
    ];
    for sel in selectors {
        if let Ok(s) = Selector::parse(sel) {
            if let Some(el) = doc.select(&s).next() {
                return el.inner_html();
            }
        }
    }
    String::new()
}

/// Parse "Jan 31, 2024" or a relative date into unix millis.
fn parse_da_date(s: &str) -> Option<i64> {
    let s = s.trim();
    let d = chrono::NaiveDate::parse_from_str(s, "%b %d, %Y")
        .or_else(|_| chrono::NaiveDate::parse_from_str(s, "%B %d, %Y"))
        .ok()?;
    let dt = d.and_hms_opt(0, 0, 0)?;
    Some(chrono::Utc.from_utc_datetime(&dt).timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = DeviantArtScraper;
        assert!(s.can_handle("https://www.deviantart.com/author/art/title-123"));
        assert!(!s.can_handle("https://www.deviantart.com/gallery/123"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_author_from_url() {
        let url = "https://www.deviantart.com/some-author/art/story-title-123";
        let author = url
            .split("deviantart.com/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap();
        assert_eq!(author, "some-author");
    }

    #[test]
    fn parses_dates() {
        assert!(parse_da_date("Jan 31, 2024").is_some());
        assert!(parse_da_date("January 31, 2024").is_some());
        assert_eq!(parse_da_date("garbage"), None);
    }

    #[test]
    fn extracts_chapter_body() {
        let html =
            r#"<html><body><div data-editor-viewer="1"><p>Deviant text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("[data-editor-viewer='1']") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Deviant text."));
    }
}
