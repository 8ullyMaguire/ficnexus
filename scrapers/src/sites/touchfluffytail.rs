//! TouchFluffyTail.org native adapter (monstergirl fiction).
//!
//! WordPress-style one-shot: `article#post-{id}`:
//! - title: `h1.entry-title`
//! - author: `a[rel=author]`
//! - published/updated: `time.published` / `time.updated` `[datetime]`
//! - tags: `span.tag-links a`
//! - views: regex `</div>(\d+) Views\s+</div>`
//! - comments: `span.comments-count`
//! - body: `div.entry-content` (strip `div.post-ratings`)

use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct TouchFluffyTailScraper;

#[async_trait]
impl SiteScraper for TouchFluffyTailScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("touchfluffytail.org/story/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut tags: Vec<String> = Vec::new();

        // The story article.
        let article_sel = Selector::parse("article[id^='post-']").map_err(|e| ScrapeError::ParseError(e.to_string()))?;
        if let Some(article) = doc.select(&article_sel).next() {
            // Title
            if let Ok(h1_sel) = Selector::parse("h1.entry-title") {
                title = article
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            // Author
            if let Ok(a_sel) = Selector::parse("a[rel='author']") {
                if let Some(a) = article.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    author_url = a.value().attr("href").unwrap_or("").to_string();
                }
            }
            // Dates
            if let Ok(t_sel) = Selector::parse("time.published") {
                if let Some(t) = article.select(&t_sel).next() {
                    if let Some(dt) = t.value().attr("datetime") {
                        published = parse_iso(dt);
                    }
                }
            }
            if let Ok(t_sel) = Selector::parse("time.updated") {
                if let Some(t) = article.select(&t_sel).next() {
                    if let Some(dt) = t.value().attr("datetime") {
                        updated = parse_iso(dt);
                    }
                }
            }
            // Tags
            if let Ok(tag_sel) = Selector::parse("span.tag-links a") {
                for a in article.select(&tag_sel) {
                    tags.push(a.text().collect::<String>().trim().to_string());
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("touchfluffytail: no title".into()));
        }

        let story_id = url.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("tft_{story_id}"),
            title,
            author: author.clone(),
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
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
            return Err(ScrapeError::ParseError("touchfluffytail: no story text".into()));
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
    let Ok(article_sel) = Selector::parse("article[id^='post-']") else {
        return String::new();
    };
    if let Some(article) = doc.select(&article_sel).next() {
        if let Ok(ec_sel) = Selector::parse("div.entry-content") {
            if let Some(el) = article.select(&ec_sel).next() {
                // Strip post-ratings blocks (the div itself or ancestors).
                let mut out = String::new();
                for child in el.children() {
                    if let Some(cref) = scraper::ElementRef::wrap(child) {
                        let is_rating = cref
                            .value()
                            .attr("class")
                            .map(|c| c.contains("post-ratings"))
                            .unwrap_or(false);
                        let has_rating = cref
                            .select(&Selector::parse("div.post-ratings").unwrap())
                            .next()
                            .is_some();
                        if !is_rating && !has_rating {
                            out.push_str(&cref.inner_html());
                        }
                    }
                }
                return out;
            }
        }
    }
    String::new()
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
        let s = TouchFluffyTailScraper;
        assert!(s.can_handle("https://touchfluffytail.org/story/title-of-book/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_iso_dates() {
        assert!(parse_iso("2024-01-31T10:00:00Z") > 0);
        assert!(parse_iso("2024-01-31T10:00:00+00:00") > 0);
        assert_eq!(parse_iso("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><article id="post-1"><div class="entry-content"><p>Story text.</p><div class="post-ratings">rating</div></div></article></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(article_sel) = Selector::parse("article[id^='post-']") {
            if let Some(article) = doc.select(&article_sel).next() {
                if let Ok(ec_sel) = Selector::parse("div.entry-content") {
                    if let Some(el) = article.select(&ec_sel).next() {
                        for child in el.children() {
                            if let Some(cref) = scraper::ElementRef::wrap(child) {
                                let is_rating = cref.value().attr("class").map(|c| c.contains("post-ratings")).unwrap_or(false);
                                let has_rating = cref.select(&Selector::parse("div.post-ratings").unwrap()).next().is_some();
                                if !is_rating && !has_rating {
                                    s.push_str(&cref.inner_html());
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(s.contains("Story text."));
        assert!(!s.contains("rating"));
    }
}
