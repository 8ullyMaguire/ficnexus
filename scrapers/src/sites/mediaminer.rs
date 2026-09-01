//! MediaMiner.org native adapter.
//!
//! Story page (w/o trailing slash gives the chapter list even for one-shots):
//! `https://www.mediaminer.org/fanfic/s/{cat}/{title}/{id}`.
//! - title: `h1#post-title` (strip the "A, A' Fan Fiction ❯ " prefix)
//! - rating: `div#post-rating` ("[ A - All Readers ]" → strip "[ " " ]")
//! - author: `a[href*="/user_info.php/"]`
//! - chapters: `p[style=margin-left:10px] a`
//! - description: `div.post-meta` html between `</a><br/>` and `<br/><b>`
//! - words/status: regex on the post-meta text ("Words: 123K", "Status: Completed")
//! - body: `div#fanfic-text`

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct MediaMinerScraper;

#[async_trait]
impl SiteScraper for MediaMinerScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("mediaminer.org/fanfic/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Chapter-list page is the no-trailing-slash URL.
        let page_url = url.trim_end_matches('/').to_string();
        let html = http::fetch(client, &page_url).await?;
        let doc = Html::parse_document(&html);

        // Title
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("h1#post-title") {
            if let Some(h1) = doc.select(&h1_sel).next() {
                let raw = h1.text().collect::<String>().trim().to_string();
                // Strip "A, A' Fan Fiction ❯ " prefix.
                if let Some(idx) = raw.rfind('❯') {
                    title = raw[idx + '❯'.len_utf8()..].trim().to_string();
                } else {
                    title = raw;
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("mediaminer: no title".into()));
        }

        // Rating
        let mut rating = String::new();
        if let Ok(r_sel) = Selector::parse("div#post-rating") {
            if let Some(el) = doc.select(&r_sel).next() {
                let raw = el.text().collect::<String>().trim().to_string();
                rating = raw.trim_start_matches("[ ").trim_end_matches(" ]").to_string();
            }
        }

        // Author
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        if let Ok(a_sel) = Selector::parse("a[href*='/user_info.php/']") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://www.mediaminer.org{h}");
                    author_local_id = h.rsplit('/').next().unwrap_or("").to_string();
                }
            }
        }

        // Chapters
        let mut chapters = 0i32;
        if let Ok(p_sel) = Selector::parse("p[style='margin-left:10px;'] a") {
            chapters = doc.select(&p_sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Description + words + status from post-meta.
        let mut desc = String::new();
        let mut words = 0i64;
        let mut status = "ongoing".to_string();
        if let Ok(m_sel) = Selector::parse("div.post-meta") {
            if let Some(meta) = doc.select(&m_sel).next() {
                let html = meta.inner_html();
                let text = meta.text().collect::<String>();
                if let Some(start) = html.find("</a><br/>") {
                    let start = start + "</a><br/>".len();
                    let end = html.find("<br/><b>").unwrap_or(html.len());
                    if start < end {
                        desc = html[start..end].to_string();
                    }
                }
                // Words: 123 or 23.1K or 1.0M
                if let Some(m) = Regex::new(r"\|\s*Words:\s*([\d.]+)(K|M)?\s*\|").unwrap().captures(&text) {
                    let num: f64 = m.get(1).unwrap().as_str().parse().unwrap_or(0.0);
                    let factor: f64 = match m.get(2).map(|x| x.as_str()) {
                        Some("K") => 1000.0,
                        Some("M") => 1_000_000.0,
                        _ => 1.0,
                    };
                    words = (num * factor) as i64;
                }
                if text.contains("Status: Completed") {
                    status = "complete".to_string();
                }
            }
        }

        // Story id from URL.
        let story_id = url.rsplit('/').next().unwrap_or("").to_string();

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("mm_{story_id}"),
            title,
            author,
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
            let sel = Selector::parse("p[style='margin-left:10px;'] a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.mediaminer.org{href}")
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
    if let Ok(sel) = Selector::parse("div#fanfic-text") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = MediaMinerScraper;
        assert!(s.can_handle("https://www.mediaminer.org/fanfic/s/cat/title/123456"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_words_k() {
        let text = "| Words: 23.1K | Status: In-Progress |";
        let m = Regex::new(r"\|\s*Words:\s*([\d.]+)(K|M)?\s*\|").unwrap().captures(text).unwrap();
        let num: f64 = m.get(1).unwrap().as_str().parse().unwrap();
        let factor: f64 = match m.get(2).map(|x| x.as_str()) {
            Some("K") => 1000.0,
            Some("M") => 1_000_000.0,
            _ => 1.0,
        };
        assert_eq!((num * factor) as i64, 23100);
    }

    #[test]
    fn strips_title_prefix() {
        let raw = "A, A' Fan Fiction ❯ My Story";
        let idx = raw.rfind('❯').unwrap();
        assert_eq!(raw[idx + '❯'.len_utf8()..].trim(), "My Story");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="fanfic-text"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#fanfic-text") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
