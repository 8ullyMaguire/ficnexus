//! FireflyFans.net native adapter (Firefly/Serenity archive).
//!
//! One-shot story page (all chapters on one page, ASP.NET layout):
//! - title: `span#MainContent_txtItemName`
//! - author: `a[href*="profileshow.aspx?u="]` (id after `u=`)
//! - description: `span#MainContent_txtItemDescription`
//! - published: `span#MainContent_txtItemInfo` (text after ", ")
//! - category/series: `span#MainContent_txtItemDetails` ("CATEGORY: X
//!   SERIES: Y" label/value pairs)
//! - genre: `span#MainContent_txtBlueSunHeader` (strip
//!   "BLUE SUN ROOM FAN FICTION - " prefix)
//! - body: `div.fanfic`

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FireflyFansScraper;

#[async_trait]
impl SiteScraper for FireflyFansScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fireflyfans.net")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("Something bad happened, but hell if I know what it is") {
            return Err(ScrapeError::NotFound);
        }

        let doc = Html::parse_document(&html);

        // Title
        let mut title = String::new();
        if let Ok(t_sel) = Selector::parse("span#MainContent_txtItemName") {
            title = doc
                .select(&t_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fireflyfans: no title".into()));
        }

        // Author
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        if let Ok(a_sel) = Selector::parse("a[href*='profileshow.aspx?u=']") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://www.fireflyfans.net/{h}");
                    author_local_id = h.split('=').nth(1).unwrap_or("").to_string();
                }
            }
        }

        // Description
        let mut desc = String::new();
        if let Ok(d_sel) = Selector::parse("span#MainContent_txtItemDescription") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Published date from txtItemInfo (after ", ").
        let mut published = 0i64;
        if let Ok(i_sel) = Selector::parse("span#MainContent_txtItemInfo") {
            if let Some(el) = doc.select(&i_sel).next() {
                let raw = el.text().collect::<String>();
                if let Some(idx) = raw.find(", ") {
                    published = parse_dt(raw[idx + 2..].trim());
                }
            }
        }

        // Category/series from txtItemDetails.
        let mut category = String::new();
        let mut series = String::new();
        if let Ok(d_sel) = Selector::parse("span#MainContent_txtItemDetails") {
            if let Some(el) = doc.select(&d_sel).next() {
                let raw = el.text().collect::<String>().replace('\u{a0}', " ");
                for meta in raw.split("    ") {
                    if let Some(idx) = meta.find(':') {
                        let label = meta[..idx].trim().to_uppercase();
                        let value = meta[idx + 1..].trim().to_string();
                        if label == "CATEGORY" {
                            category = value;
                        } else if label == "SERIES" {
                            series = value;
                        }
                    }
                }
            }
        }

        // Genre from txtBlueSunHeader.
        let mut genre = String::new();
        if let Ok(g_sel) = Selector::parse("span#MainContent_txtBlueSunHeader") {
            genre = doc
                .select(&g_sel)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default()
                .replace("BLUE SUN ROOM FAN FICTION - ", "")
                .trim()
                .to_string();
        }

        // Story id from URL.
        let story_id = url.rsplit('/').next().unwrap_or("").to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ffn_{story_id}"),
            title,
            author,
            chapters: 1,
            words: 0,
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
            extra_meta: Some(format!(
                "category={};series={};genre={};",
                category, series, genre
            )),
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
            return Err(ScrapeError::ParseError("fireflyfans: no story text".into()));
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

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div.fanfic") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = FireflyFansScraper;
        assert!(s.can_handle("https://www.fireflyfans.net/mstory.asp?t=123"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_author_id() {
        let href = "mstory.asp?u=42";
        assert_eq!(href.split('=').nth(1).unwrap(), "42");
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("01/31/2024") > 0);
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="fanfic"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div.fanfic") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
