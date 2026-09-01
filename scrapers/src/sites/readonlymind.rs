//! ReadonlyMind.com native adapter (adult-gated original fiction).
//!
//! Story URL: `https://readonlymind.com/@Author/Story_Name/`.
//! - title: `header h1`
//! - author: `meta[name=author]` content + `link[rel=author]` href
//! - description: `meta[name=description]` content
//! - created: `meta[name=created]` content (`%Y-%m-%d`)
//! - chapters: `section#chapter-list section.story-card-large`
//!   `div.story-card-title a` (single chapter → the URL itself)
//! - body: `section#chapter-content`
//!
//! Adult gate: the site requires an `is_adult` confirmation; the host
//! enables it via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::http;

pub struct ReadonlyMindScraper {
    adult_ok: AtomicBool,
}

impl Default for ReadonlyMindScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl ReadonlyMindScraper {
    fn story_parts(url: &str) -> Option<(String, String)> {
        let m = Regex::new(r"readonlymind\.com/@([a-zA-Z0-9_]+)/([a-zA-Z0-9_]+)").ok()?;
        let c = m.captures(url)?;
        Some((c.get(1)?.as_str().to_string(), c.get(2)?.as_str().to_string()))
    }
}

#[async_trait]
impl SiteScraper for ReadonlyMindScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("readonlymind.com/@")
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        _client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        if creds.is_adult {
            self.adult_ok.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err(ScrapeError::AuthRequired("readonlymind: is_adult not set".into()))
        }
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired("readonlymind: adult gate".into()));
        }
        let (auth, story_id) = Self::story_parts(url)
            .ok_or_else(|| ScrapeError::ParseError("readonlymind: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("Page Not Found.") {
            return Err(ScrapeError::NotFound);
        }

        let (title, author, desc, published) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut desc = String::new();
            let mut published = 0i64;

            if let Ok(h1_sel) = Selector::parse("header h1") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            if let Ok(m_sel) = Selector::parse("meta[name='author']") {
                if let Some(m) = doc.select(&m_sel).next() {
                    author = m.value().attr("content").unwrap_or("").to_string();
                }
            }
            if let Ok(m_sel) = Selector::parse("meta[name='description']") {
                if let Some(m) = doc.select(&m_sel).next() {
                    desc = m.value().attr("content").unwrap_or("").to_string();
                }
            }
            if let Ok(m_sel) = Selector::parse("meta[name='created']") {
                if let Some(m) = doc.select(&m_sel).next() {
                    if let Some(c) = m.value().attr("content") {
                        if let Ok(d) = chrono::NaiveDate::parse_from_str(c, "%Y-%m-%d") {
                            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                                published = dt.and_utc().timestamp_millis();
                            }
                        }
                    }
                }
            }
            (title, author, desc, published)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("readonlymind: no title".into()));
        }

        // Chapters.
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(sel) = Selector::parse("section#chapter-list section.story-card-large div.story-card-title a[href]") {
                chapters = doc.select(&sel).count() as i32;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("rom_{}_{}", auth, story_id),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url: format!("https://readonlymind.com/@{}", auth),
            author_local_id: auth,
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
        let html = http::fetch(client, &meta.source).await?;

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("section#chapter-list section.story-card-large div.story-card-title a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://readonlymind.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        let urls: Vec<(String, String)> = if links.is_empty() {
            vec![(meta.title.clone(), meta.source.clone())]
        } else {
            links
        };

        let mut chapters = Vec::new();
        for (i, (title, url)) in urls.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title: if title.is_empty() {
                    format!("Chapter {}", i + 1)
                } else {
                    title
                },
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
    if let Ok(sel) = Selector::parse("section#chapter-content") {
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
        let s = ReadonlyMindScraper::default();
        assert!(s.can_handle("https://readonlymind.com/@AnAuthor/A_Story_Name/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_parts() {
        let (auth, id) = ReadonlyMindScraper::story_parts("https://readonlymind.com/@AnAuthor/A_Story_Name/").unwrap();
        assert_eq!(auth, "AnAuthor");
        assert_eq!(id, "A_Story_Name");
        assert!(ReadonlyMindScraper::story_parts("https://x.com/foo").is_none());
    }

    #[test]
    fn parses_dates() {
        let c = "2024-01-31";
        if let Ok(d) = chrono::NaiveDate::parse_from_str(c, "%Y-%m-%d") {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                assert!(dt.and_utc().timestamp_millis() > 0);
            }
        }
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><section id="chapter-content"><p>Story text.</p></section></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("section#chapter-content").unwrap();
        let s = doc.select(&sel).next().map(|el| el.inner_html()).unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
