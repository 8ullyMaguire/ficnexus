//! FicWad.com native adapter (older general archive).
//!
//! Story page: `https://ficwad.com/story/{id}`.
//! - title: `div.storylist h4 a`
//! - author: `span.author a[href^="/a/"]`
//! - description: `div#story blockquote.summary p`
//! - metadata: `div.meta` text — regex for Rating/Genres/Characters/
//!   Published/Updated/words/Complete
//! - chapters: `ul.storylist li h4 a` (blocked → AuthRequired)
//! - body: `div#storytext`

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct FicwadScraper;

#[async_trait]
impl SiteScraper for FicwadScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("ficwad.com/story/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("<h4>Featured Story</h4>") {
            return Err(ScrapeError::NotFound);
        }
        if html.contains("class=\"blocked\"") {
            return Err(ScrapeError::AuthRequired("ficwad login gate".into()));
        }

        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut desc = String::new();
        let mut rating = String::new();
        let mut status = "ongoing".to_string();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut words = 0i64;
        let mut chapters = 0i32;

        // Title
        if let Ok(t_sel) = Selector::parse("div.storylist h4 a") {
            title = doc
                .select(&t_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() || title.contains("Deleted story") {
            return Err(ScrapeError::NotFound);
        }

        // Author
        if let Ok(a_sel) = Selector::parse("span.author a[href^='/a/']") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://ficwad.com{h}");
                    author_local_id = h.split('/').nth(2).unwrap_or("").to_string();
                }
            }
        }

        // Description
        if let Ok(d_sel) = Selector::parse("div#story blockquote.summary p") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Metadata from div.meta text.
        if let Ok(m_sel) = Selector::parse("div.meta") {
            if let Some(meta) = doc.select(&m_sel).next() {
                let metastr = meta.text().collect::<String>().replace('\u{a0}', " ");
                if let Some(m) = Regex::new(r"Rating: (.+?) -").unwrap().captures(&metastr) {
                    rating = m.get(1).unwrap().as_str().trim().to_string();
                }
                if let Some(m) = Regex::new(r"Published: ([0-9-]+?) -").unwrap().captures(&metastr) {
                    published = parse_dt(m.get(1).unwrap().as_str());
                }
                if let Some(m) = Regex::new(r"Updated: ([0-9-]+?) +-").unwrap().captures(&metastr) {
                    updated = parse_dt(m.get(1).unwrap().as_str());
                }
                if let Some(m) = Regex::new(r" - ([0-9,]+?) words").unwrap().captures(&metastr) {
                    words = m.get(1).unwrap().as_str().chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                }
                if metastr.trim_end().ends_with("Complete") {
                    status = "complete".to_string();
                }
            }
        }

        // Chapters
        if let Ok(sel) = Selector::parse("ul.storylist li h4 a") {
            let n = doc.select(&sel).count() as i32;
            if n > 0 {
                chapters = n;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Story id from URL.
        let story_id = url.rsplit('/').next().unwrap_or("").to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fw_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status,
            source: url.to_string(),
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

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("ul.storylist li h4 a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://ficwad.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
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

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div#storytext") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
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
        let s = FicwadScraper;
        assert!(s.can_handle("https://ficwad.com/story/12345"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn parses_words() {
        let raw = " - 12,345 words";
        let m = Regex::new(r" - ([0-9,]+?) words").unwrap().captures(raw).unwrap();
        let words: i64 = m.get(1).unwrap().as_str().chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap();
        assert_eq!(words, 12345);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="storytext"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#storytext") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
