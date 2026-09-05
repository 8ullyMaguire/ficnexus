//! FicBook.net native adapter (Russian fanfiction archive).
//!
//! Story page: `https://ficbook.net/readfic/{storyId}`.
//! - title: `section.chapter-info h1` (strip `<sup>` marks)
//! - author: `a.creator-username` (name is the id)
//! - chapters: `ul.list-of-fanfic-parts li.part a[href*="/readfic/{id}/N#part_content"]`
//! - description: `div[itemprop=description]`
//! - status: `div.ds-label-status-finished` presence
//! - rating: `div[class*=ds-label-rating-]`
//! - words/pages: regex over embedded text
//! Chapter body: `div#content` (+ optional `part_text` formatting class),
//! with head/foot notes in `div.js-public-beta-comment-before/after`.

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FicBookScraper;

#[async_trait]
impl SiteScraper for FicBookScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("ficbook.net/readfic/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("id=\"adultCoverWarning\"") {
            return Err(ScrapeError::AuthRequired("FicBook adult gate".into()));
        }

        let doc = Html::parse_document(&html);

        // Title: section.chapter-info h1 (strip sup marks)
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("section.chapter-info h1") {
            if let Some(h1) = doc.select(&h1_sel).next() {
                title = h1.text().collect::<String>().trim().to_string();
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("ficbook: no title".into()));
        }

        // Author
        let mut author = String::new();
        let mut author_url = String::new();
        if let Ok(a_sel) = Selector::parse("a.creator-username") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://ficbook.net{h}");
                }
            }
        }

        // Story id from URL
        let story_id = url
            .split("/readfic/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("")
            .to_string();

        // Description
        let mut desc = String::new();
        if let Ok(d_sel) = Selector::parse("div[itemprop='description']") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Status
        let status = if doc
            .select(&Selector::parse("div.ds-label-status-finished").unwrap())
            .next()
            .is_some()
        {
            "complete"
        } else {
            "ongoing"
        }
        .to_string();

        // Rating
        let mut rating = String::new();
        if let Ok(r_sel) = Selector::parse("div[class*='ds-label-rating-']") {
            rating = doc
                .select(&r_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }

        // Word count from embedded text ("1234 слова/слов")
        let mut words = 0i64;
        if let Ok(re) = Regex::new(r"(\d+)(?:слова|слов)") {
            if let Some(m) = re.captures(&html) {
                words = m.get(1).and_then(|x| x.as_str().parse().ok()).unwrap_or(0);
            }
        }

        // Chapters
        let mut chapters = 0i32;
        let chap_pat = format!("/readfic/{story_id}/");
        if let Ok(li_sel) = Selector::parse("ul.list-of-fanfic-parts li.part a") {
            chapters = doc
                .select(&li_sel)
                .filter(|el| {
                    el.value()
                        .attr("href")
                        .map(|h| h.contains(&chap_pat))
                        .unwrap_or(false)
                })
                .count() as i32;
        }
        if chapters == 0 {
            chapters = 1; // one-shot
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ficb_{story_id}"),
            title,
            author: author.clone(),
            chapters,
            words,
            desc,
            published: now,
            updated: now,
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id: author.clone(),
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

        // Collect chapter URLs into owned data (drop non-Send doc before loop).
        let story_id = meta
            .source
            .split("/readfic/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("")
            .to_string();
        let chap_pat = format!("/readfic/{story_id}/");

        let links: Vec<String> = {
            let doc = Html::parse_document(&html);
            let li_sel = Selector::parse("ul.list-of-fanfic-parts li.part a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&li_sel)
                .filter_map(|el| {
                    let href = el.value().attr("href")?;
                    if href.contains(&chap_pat) && href.contains("#part_content") {
                        Some(format!("https://ficbook.net{href}"))
                    } else {
                        None
                    }
                })
                .collect()
        };

        if links.is_empty() {
            // One-shot: the story page body.
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, url) in links.into_iter().enumerate() {
            let title = format!("Chapter {}", i + 1);
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
                content,
            });
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

/// Fetch a FicBook chapter and extract `div#content` + notes.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    let mut out = String::new();

    if let Ok(note_sel) = Selector::parse("div.js-public-beta-comment-before") {
        for el in doc.select(&note_sel) {
            out.push_str(&format!(
                "<div class=\"fff_head_notes\">{}</div>",
                el.inner_html()
            ));
        }
    }
    if let Ok(sel) = Selector::parse("div#content") {
        if let Some(el) = doc.select(&sel).next() {
            out.push_str(&el.inner_html());
        }
    }
    if let Ok(note_sel) = Selector::parse("div.js-public-beta-comment-after") {
        for el in doc.select(&note_sel) {
            out.push_str(&format!(
                "<div class=\"fff_foot_notes\">{}</div>",
                el.inner_html()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches_readfic_urls() {
        let s = FicBookScraper;
        assert!(s.can_handle("https://ficbook.net/readfic/12345678"));
        assert!(s.can_handle("https://ficbook.net/readfic/12345678/246417#part_content"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let url = "https://ficbook.net/readfic/12345678/246417#part_content";
        let id = url
            .split("/readfic/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap();
        assert_eq!(id, "12345678");
    }

    #[test]
    fn extracts_chapter_body() {
        let html =
            r#"<html><body><div id="content"><p>Russian story text.</p></div></body></html>"#;
        let out = {
            // fetch_chapter_text needs a client; test the body extraction logic
            // inline via parse.
            let doc = Html::parse_document(html);
            let mut s = String::new();
            if let Ok(sel) = Selector::parse("div#content") {
                if let Some(el) = doc.select(&sel).next() {
                    s.push_str(&el.inner_html());
                }
            }
            s
        };
        assert!(out.contains("Russian story text."));
    }

    #[test]
    fn status_detection() {
        let html = r#"<html><body><div class="ds-label-status-finished"></div></body></html>"#;
        let doc = Html::parse_document(html);
        let is_complete = doc
            .select(&Selector::parse("div.ds-label-status-finished").unwrap())
            .next()
            .is_some();
        assert!(is_complete);
    }
}
