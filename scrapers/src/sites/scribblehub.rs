//! ScribbleHub.com native adapter.
//!
//! FFF's ScribbleHub adapter (adapter_scribblehubcom.py) is fragile — it
//! crashes on newer markup ("NoneType object has no attribute get_text",
//! upstream issue #1397). A native Rust adapter bypasses FFF entirely for
//! this site and is robust to FFF's Python version drift.
//!
//! Story URL: https://www.scribblehub.com/series/{id}/{slug}/
//! Metadata: author link in `.author div[property=author] span[property=name] a`,
//! dates in `span.fic_date_pub` / `span.fic_date_update`, word count in
//! `span.fic_word_count`, status in `span.fic_status`.

use crate::{FicMetadata, Chapter, SiteScraper, ScrapeError};

use async_trait::async_trait;

/// ScribbleHub adapter.
pub struct ScribbleHubScraper;

impl ScribbleHubScraper {
    fn story_id(url: &str) -> Option<String> {
        let lower = url.to_lowercase();
        let pos = lower.find("/series/")?;
        let rest = &url[pos + "/series/".len()..];
        let id_end = rest.find('/').unwrap_or(rest.len());
        let id = &rest[..id_end];
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        Some(id.to_string())
    }
}

#[async_trait]
impl SiteScraper for ScribbleHubScraper {
    fn can_handle(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("scribblehub.com") && lower.contains("/series/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = ScribbleHubScraper::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("no story id in URL".into()))?;

        let resp = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if resp.status().is_client_error() || resp.status().is_server_error() {
            return Err(ScrapeError::NotFound);
        }

        let html = resp.text().await.map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = scraper::Html::parse_document(&html);

        // Title: <div class="fic_title"> or <h1 class="fic_title">
        let title = document
            .select(&scraper::Selector::parse("div.fic_title, h1.fic_title").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Author: <div class="author"> … <a href="/user/…">Name</a>
        let author_el = document
            .select(&scraper::Selector::parse("div.author a[href*='/user/']").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next();
        let (author, author_url) = match author_el {
            Some(a) => {
                let name = a.text().collect::<String>().trim().to_string();
                let href = a.value().attr("href").unwrap_or("").to_string();
                (name, if href.starts_with("http") { href } else { format!("https://www.scribblehub.com{href}") })
            }
            None => (String::new(), String::new()),
        };

        // Chapter count: <span class="fic_ep_count"> or "Chapters:" row
        let chapters = document
            .select(&scraper::Selector::parse("span.fic_ep_count").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .and_then(|el| el.text().collect::<String>().trim().parse::<i32>().ok())
            .unwrap_or(0);

        // Word count: <span class="fic_word_count"> — text like "123,456"
        let words = document
            .select(&scraper::Selector::parse("span.fic_word_count").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .and_then(|el| el.text().collect::<String>().replace(',', "").trim().parse::<i64>().ok())
            .unwrap_or(0);

        // Dates: <span class="fic_date_pub"> / <span class="fic_date_update"> (text like "Jul 12, 2024")
        let date_text = |sel: &str| -> i64 {
            document
                .select(&scraper::Selector::parse(sel).map_err(|e| ScrapeError::ParseError(e.to_string())).unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .and_then(|d| crate::sites::parse_short_date(&d))
                .unwrap_or(0)
        };
        let published = date_text("span.fic_date_pub");
        let updated = date_text("span.fic_date_update");

        // Status: <span class="fic_status"> — text like "Ongoing" / "Completed"
        let status = document
            .select(&scraper::Selector::parse("span.fic_status").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_lowercase())
            .map(|s| {
                if s.contains("completed") { "complete".to_string() }
                else if s.contains("hiatus") { "hiatus".to_string() }
                else if s.contains("abandoned") { "cancelled".to_string() }
                else { "ongoing".to_string() }
            })
            .unwrap_or_else(|| "ongoing".to_string());

        // Description: <div class="wi_fic_desc"> (may contain HTML)
        let desc = document
            .select(&scraper::Selector::parse("div.wi_fic_desc").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        if title.is_empty() && author.is_empty() {
            return Err(ScrapeError::ParseError("ScribbleHub page missing title+author".into()));
        }

        Ok(FicMetadata {
            url_id: crate::generate_url_id(7, &story_id),
            title,
            author,
            chapters,
            words,
            desc,
            published,
            updated,
            status,
            source: url.to_string(),
            source_id: 7,
            author_id: 0,
            author_url,
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        // ScribbleHub loads chapter content via JS; the chapter list is in
        // the story page's `.chapter_list .chapter-title` links. Each chapter
        // is a separate URL we must fetch + extract `div.chp_rawhtml`.
        let resp = client
            .get(&meta.source)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let html = resp.text().await.map_err(|e| ScrapeError::Network(e.to_string()))?;
        // Collect (url, title) pairs in a scoped block so the parsed doc
        // (non-Send tendril) drops before any await below.
        let links: Vec<(String, String)> = {
            let document = scraper::Html::parse_document(&html);
            let sel = scraper::Selector::parse("div.chapter_list a[href*='/chapter/']")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let mut links: Vec<(String, String)> = Vec::new();
            for link in document.select(&sel) {
                let href = link.value().attr("href").unwrap_or("");
                let url = if href.starts_with("http") { href.to_string() } else { format!("https://www.scribblehub.com{href}") };
                let title = link.text().collect::<String>().trim().to_string();
                links.push((url, title));
            }
            links
        };

        let mut chapters = Vec::new();
        let mut i = 1;
        let content_sel = scraper::Selector::parse("div.chp_rawhtml")
            .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
        for (url, title) in links {
            let chap_resp = client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let chap_html = chap_resp.text().await.map_err(|e| ScrapeError::Network(e.to_string()))?;
            // Scope the parsed doc so it (and its non-Send tendril) drops
            // before the next loop iteration awaits.
            let content = {
                let chap_doc = scraper::Html::parse_document(&chap_html);
                chap_doc
                    .select(&content_sel)
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default()
            };
            if !content.is_empty() {
                chapters.push(Chapter { chapter_id: i, title, content });
                i += 1;
            }
        }
        Ok(chapters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_id_extraction() {
        assert_eq!(
            ScribbleHubScraper::story_id("https://www.scribblehub.com/series/123456/some-title/").as_deref(),
            Some("123456")
        );
        assert_eq!(
            ScribbleHubScraper::story_id("https://www.scribblehub.com/series/987654/Other-Title/").as_deref(),
            Some("987654")
        );
        assert_eq!(ScribbleHubScraper::story_id("https://example.com/x"), None);
        assert_eq!(ScribbleHubScraper::story_id("https://www.scribblehub.com/series/abc/x/"), None);
    }

    #[test]
    fn can_handle_scribblehub() {
        let s = ScribbleHubScraper;
        assert!(s.can_handle("https://www.scribblehub.com/series/123456/title/"));
        assert!(!s.can_handle("https://archiveofourown.org/works/1"));
        assert!(!s.can_handle("https://www.scribblehub.com/profile/123"));
    }
}
