//! LCFanFic.com native adapter (Harry Potter archive).
//!
//! One-shot story per page: `http://lcfanfic.com/stories/{year}/html/{name}.html`.
//! - title: `div.lcfheader h2`
//! - author: `div.lcfheader p` first link (strip leading "By ")
//! - rating: next `p` ("Rated: X")
//! - description: remaining `div.lcfheader` body after h2/p
//! - story text: everything after the `***` separator (scripts stripped)

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct LcFanficScraper;

#[async_trait]
impl SiteScraper for LcFanficScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("lcfanfic.com/stories/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut rating = String::new();
        let mut desc = String::new();

        if let Ok(hdr_sel) = Selector::parse("div.lcfheader") {
            if let Some(hdr) = doc.select(&hdr_sel).next() {
                // Title
                if let Ok(h2_sel) = Selector::parse("h2") {
                    title = hdr
                        .select(&h2_sel)
                        .next()
                        .map(|el| el.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                }
                // Author: first p, strip "By "
                if let Ok(p_sel) = Selector::parse("p") {
                    let ps: Vec<_> = hdr.select(&p_sel).collect();
                    if let Some(p) = ps.first() {
                        // Author link if present.
                        if let Ok(a_sel) = Selector::parse("a") {
                            if let Some(a) = p.select(&a_sel).next() {
                                author_url = a.value().attr("href").unwrap_or("").to_string();
                            }
                        }
                        let raw = p.text().collect::<String>();
                        author = raw.trim_start_matches("By ").trim().to_string();
                        // strip email in <> or ()
                        author = author.split('<').next().unwrap_or("").trim().to_string();
                        author = author.split('(').next().unwrap_or("").trim().to_string();
                    }
                    // Rating: next p
                    if let Some(p) = ps.get(1) {
                        let raw = p.text().collect::<String>().trim().to_string();
                        if raw.contains("Rated") {
                            rating = raw.replace("Rated", "").replace(':', "").trim().to_string();
                        }
                    }
                }
                // Description: the header body minus h2/p (collect text).
                desc = hdr.text().collect::<String>();
                desc = desc
                    .replace(&title, "")
                    .replace(&author, "")
                    .replace(&rating, "")
                    .replace("Rated:", "")
                    .trim()
                    .to_string();
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("lcfanfic: no title".into()));
        }

        let story_id = url
            .trim_end_matches(".html")
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("lcff_{story_id}"),
            title,
            author: author.clone(),
            chapters: 1,
            words: 0,
            desc,
            published: now,
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
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
        let content = fetch_chapter_text(client, &meta.source).await;
        if content.is_empty() {
            return Err(ScrapeError::ParseError("lcfanfic: no story text".into()));
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

/// Fetch the story text: everything after the `***` separator, scripts stripped.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    // Find the *** separator.
    let sep = html
        .find("<p align=center>***</p>")
        .or_else(|| html.find("<p align=\"center\">***</p>"));
    let Some(idx) = sep else { return String::new() };
    let after = &html[idx..];
    let doc = Html::parse_document(after);
    // Remove scripts.
    let mut out = String::new();
    if let Some(body) = doc.select(&Selector::parse("body").unwrap()).next() {
        for child in body.children() {
            if let Some(el) = scraper::ElementRef::wrap(child) {
                if el.value().name() != "script" {
                    out.push_str(&el.inner_html());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = LcFanficScraper;
        assert!(s.can_handle("http://lcfanfic.com/stories/2017/html/story-name.html"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn strips_author_prefix() {
        let raw = "By Some Author <email@x.com>".to_string();
        let mut a = raw.trim_start_matches("By ").trim().to_string();
        a = a.split('<').next().unwrap_or("").trim().to_string();
        assert_eq!(a, "Some Author");
    }

    #[test]
    fn strips_rating_prefix() {
        let raw = "Rated: Teen".to_string();
        let rating = raw.replace("Rated", "").replace(':', "").trim().to_string();
        assert_eq!(rating, "Teen");
    }

    #[test]
    fn extracts_story_text_after_separator() {
        let html = r#"<html><body><div class="lcfheader"><h2>T</h2><p>By A</p></div><p align=center>***</p><p>Story body.</p></body></html>"#;
        let idx = html.find("<p align=center>***</p>").unwrap();
        let after = &html[idx..];
        let doc = Html::parse_document(after);
        let mut s = String::new();
        if let Some(body) = doc.select(&Selector::parse("body").unwrap()).next() {
            for child in body.children() {
                if let Some(el) = scraper::ElementRef::wrap(child) {
                    if el.value().name() != "script" {
                        s.push_str(&el.inner_html());
                    }
                }
            }
        }
        assert!(s.contains("Story body."));
    }
}
