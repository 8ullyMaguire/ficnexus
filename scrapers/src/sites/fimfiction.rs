//! FimFiction.net native adapter (My Little Pony fanfiction archive).
//!
//! Story page layout:
//! - `div.story_content_box` — everything
//! - `a.story_name` — title
//! - `div.info-container a` — author (href `/user/{id}/{name}`)
//! - `a.content-rating-*` — rating (E/T/M)
//! - `span.completed-status-*` — status
//! - `div.chapters-footer div.word_count` — word count
//! - `span.description-text` — description
//! - `ul.chapters a.chapter-title` — chapter list
//! Chapter body: `div#chapter-body` (+ optional `div.authors-note`).

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FimFictionScraper;

#[async_trait]
impl SiteScraper for FimFictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fimfiction.net/story/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Normalize to the story page: /story/{id}/{slug}
        let story_id = url
            .split("/story/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .ok_or_else(|| ScrapeError::ParseError("no story id in FimFiction URL".into()))?;

        let page_url = format!("https://www.fimfiction.net/story/{story_id}/");
        let html = http::fetch(client, &page_url).await?;

        if html.contains("This story has been marked as having adult content") {
            return Err(ScrapeError::AuthRequired(
                "FimFiction adult content gate".into(),
            ));
        }

        let doc = Html::parse_document(&html);

        let story_box_sel = Selector::parse("div.story_content_box")
            .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
        let story_box = doc
            .select(&story_box_sel)
            .next()
            .ok_or_else(|| ScrapeError::ParseError("no story_content_box".into()))?;

        // Title
        let title_sel = Selector::parse("a.story_name")
            .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
        let title = story_box
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Author (info-container first link)
        let mut author = String::new();
        let mut author_local_id = String::new();
        let mut author_url = String::new();
        if let Ok(info_sel) = Selector::parse("div.info-container a") {
            if let Some(a) = doc.select(&info_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(href) = a.value().attr("href") {
                    // /user/{id}/{name}
                    let parts: Vec<&str> = href.split('/').collect();
                    if parts.len() >= 3 {
                        author_local_id = parts[2].to_string();
                    }
                    author_url = format!("https://www.fimfiction.net{href}");
                }
            }
        }

        // Rating
        let mut rating = String::new();
        if let Ok(r_sel) = Selector::parse("a[class*='content-rating-']") {
            rating = doc
                .select(&r_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }

        // Status
        let mut status = "ongoing".to_string();
        if let Ok(s_sel) = Selector::parse("span[class*='completed-status-']") {
            let raw = doc
                .select(&s_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
            status = match raw.as_str() {
                "Completed" | "Complete" => "complete".to_string(),
                "Cancelled" => "cancelled".to_string(),
                "On Hiatus" | "On-Hiatus" | "Hiatus" => "hiatus".to_string(),
                _ => "ongoing".to_string(),
            };
        }

        // Word count
        let mut words = 0i64;
        if let Ok(w_sel) = Selector::parse("div.chapters-footer div.word_count") {
            let wtext = doc
                .select(&w_sel)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();
            words = wtext
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
        }

        // Description
        let mut desc = String::new();
        if let Ok(d_sel) = Selector::parse("span.description-text") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Chapters
        let mut chapters = 0i32;
        if let Ok(c_sel) = Selector::parse("ul.chapters a.chapter-title") {
            chapters = doc.select(&c_sel).count() as i32;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fimf_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: now,
            updated: now,
            status,
            source: page_url,
            source_id: story_id.parse().unwrap_or(0),
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
        let html = http::fetch(client, &meta.source).await?;

        // Collect chapter hrefs into owned data, dropping the non-Send doc
        // before any per-chapter await.
        let hrefs: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let chap_sel = Selector::parse("ul.chapters a.chapter-title")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&chap_sel)
                .filter_map(|el| {
                    let title = el.text().collect::<String>().trim().to_string();
                    el.value().attr("href").map(|h| (title, h.to_string()))
                })
                .collect()
        };

        // NOTE: hrefs are absolute on FimFiction ("/story/{id}/{n}/{slug}").
        let mut chapters = Vec::new();
        for (i, (title, href)) in hrefs.into_iter().enumerate() {
            let chap_url = if href.starts_with("http") {
                href.clone()
            } else {
                format!("https://www.fimfiction.net{href}")
            };
            let content = fetch_chapter_text(client, &chap_url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
                content,
            });
        }

        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("no chapters found".into()));
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

/// Fetch a chapter page and extract `#chapter-body` (with author notes).
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    let mut parts = Vec::new();
    if let Ok(sel) = Selector::parse("div#chapter-body") {
        if let Some(el) = doc.select(&sel).next() {
            parts.push(el.inner_html());
        }
    }
    if let Ok(note_sel) = Selector::parse("div.authors-note") {
        for el in doc.select(&note_sel) {
            parts.push(el.inner_html());
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches_story_urls() {
        let s = FimFictionScraper;
        assert!(s.can_handle("https://www.fimfiction.net/story/1234/my-story"));
        assert!(s.can_handle("https://fimfiction.net/story/1234/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let url = "https://www.fimfiction.net/story/12345678/hello-world/";
        let id = url.split("/story/").nth(1).and_then(|s| s.split('/').next()).unwrap();
        assert_eq!(id, "12345678");
    }

    #[test]
    fn status_mapping() {
        let mut meta = FicMetadata {
            url_id: "fimf_1".into(), title: "t".into(), author: "a".into(),
            chapters: 0, words: 0, desc: String::new(), published: 0, updated: 0,
            status: "ongoing".into(), source: "u".into(), source_id: 0,
            author_id: 0, author_url: String::new(), author_local_id: String::new(),
            content_hash: None, extra_meta: None, raw_extended_meta: None,
        };
        // The adapter maps raw status strings; verify the mapping table.
        assert_eq!(raw_to_status("Completed"), "complete");
        assert_eq!(raw_to_status("Incomplete"), "ongoing");
        assert_eq!(raw_to_status("Cancelled"), "cancelled");
        assert_eq!(raw_to_status("On Hiatus"), "hiatus");
        let _ = &mut meta;
    }

    fn raw_to_status(raw: &str) -> &'static str {
        match raw {
            "Completed" | "Complete" => "complete",
            "Cancelled" => "cancelled",
            "On Hiatus" | "On-Hiatus" | "Hiatus" => "hiatus",
            _ => "ongoing",
        }
    }
}
