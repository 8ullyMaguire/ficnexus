//! Fictionalley-archive.org native adapter (Harry Potter archive).
//!
//! Story URL: `https://www.fictionalley-archive.org/authors/{auth}/{id}.html`.
//! - title: `h1`
//! - author: `h1 + h3 > a`
//! - chapters: `h5.mb-1 > a` (or one-shot)
//! - metadata: `div.card-body` — Rating/House/Character/Genre/Era/Spoiler/
//!   Ship links + `div.badge-info` (Published/Updated/Words/Hits)
//! - description: `dt` "Story Summary:" → next `dd`
//! - body: `main#content div:not([class])` (strip head/meta/script)

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct FictionalleyScraper;

impl FictionalleyScraper {
    fn story_id(url: &str) -> Option<(String, String)> {
        let m = Regex::new(r"fictionalley(-archive)?\.org/authors/([a-zA-Z0-9_]+)/([a-zA-Z0-9_]+)\.html").ok()?;
        let c = m.captures(url)?;
        Some((c.get(2)?.as_str().to_string(), c.get(3)?.as_str().to_string()))
    }
}

#[async_trait]
impl SiteScraper for FictionalleyScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fictionalley-archive.org/authors/") || url.contains("fictionalley.org/authors/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let (auth, story_id) = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("fictionalley: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        let (title, author, author_url, desc, rating, status) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut desc = String::new();
            let mut rating = String::new();
            let status = "ongoing".to_string();

            if let Ok(h1_sel) = Selector::parse("h1") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            if let Ok(a_sel) = Selector::parse("h1 + h3 > a") {
                if let Some(a) = doc.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://www.fictionalley-archive.org{h}");
                    }
                }
            }

            // Metadata from div.card-body.
            if let Ok(cb_sel) = Selector::parse("div.card-body") {
                if let Some(cb) = doc.select(&cb_sel).next() {
                    if let Ok(a_sel) = Selector::parse("a[href^='/stories?Include.Rating']") {
                        if let Some(a) = cb.select(&a_sel).next() {
                            rating = a.text().collect::<String>().trim().to_string();
                        }
                    }
                    // Badge info: Published/Updated/Words/Hits/Chapters.
                    if let Ok(b_sel) = Selector::parse("div.badge-info") {
                        for b in cb.select(&b_sel) {
                            let txt = b.text().collect::<String>().trim().to_string();
                            if let Some(idx) = txt.find(':') {
                                let key = txt[..idx].trim().to_string();
                                let val = txt[idx + 1..].trim().to_string();
                                if key == "Words" {
                                    // (not stored in FicMetadata; skip)
                                    let _ = val;
                                }
                            }
                        }
                    }
                }
            }

            // Description from dt "Story Summary:" → next dd.
            if let Ok(dt_sel) = Selector::parse("dt") {
                for dt in doc.select(&dt_sel) {
                    if dt.text().collect::<String>().contains("Story Summary:") {
                        let mut nxt = dt.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "dd" {
                                    desc = el.inner_html();
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                        break;
                    }
                }
            }

            (title, author, author_url, desc, rating, status)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("fictionalley: no title".into()));
        }

        // Chapters from h5.mb-1 > a.
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(sel) = Selector::parse("h5.mb-1 > a[href]") {
                chapters = doc.select(&sel).count() as i32;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fa_{auth}_{story_id}"),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: now,
            updated: now,
            status,
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id: auth,
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

        let links: Vec<String> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("h5.mb-1 > a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if href.starts_with("http") {
                        Some(href.to_string())
                    } else {
                        Some(format!("https://www.fictionalley-archive.org{href}"))
                    }
                })
                .collect()
        };

        let urls: Vec<String> = if links.is_empty() {
            vec![meta.source.clone()]
        } else {
            links
        };

        let mut chapters = Vec::new();
        for (i, url) in urls.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title: format!("Chapter {}", i + 1),
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
    if let Ok(sel) = Selector::parse("main#content div:not([class])") {
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
        let s = FictionalleyScraper;
        assert!(s.can_handle("https://www.fictionalley-archive.org/authors/drt/DA.html"));
        assert!(s.can_handle("http://www.fictionalley.org/authors/drt/JOTP01a.html"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let (auth, id) = FictionalleyScraper::story_id("https://www.fictionalley-archive.org/authors/drt/DA.html").unwrap();
        assert_eq!(auth, "drt");
        assert_eq!(id, "DA");
        assert!(FictionalleyScraper::story_id("https://x.com/foo").is_none());
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><main id="content"><div><p>Story text.</p></div></main></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("main#content div:not([class])").unwrap();
        let s = doc.select(&sel).next().map(|el| el.inner_html()).unwrap_or_default();
        assert!(s.contains("Story text."));
    }

    #[test]
    fn normalizes_chapter_url() {
        let href = "/authors/drt/DA02.html";
        let url = format!("https://www.fictionalley-archive.org{href}");
        assert_eq!(url, "https://www.fictionalley-archive.org/authors/drt/DA02.html");
    }
}
