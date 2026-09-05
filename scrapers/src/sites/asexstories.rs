//! ASexStories.com native adapter (adult-gated erotic fiction).
//!
//! Story URL: `https://www.asexstories.com/StoryTitle/`.
//! - title: `div.story-top-block h1`
//! - author: `div.story-info div.story-info-bl`[1] `a`
//! - rating: `div.story-info-bl5 img[title]` (strip " - Rate")
//! - chapters: `div.pages a` (first chapter = the URL itself)
//! - body: `div.story-block`
//!
//! Adult gate: `is_adult` confirmation via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};

pub struct ASexStoriesScraper {
    adult_ok: AtomicBool,
}

impl Default for ASexStoriesScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl ASexStoriesScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"asexstories\.com/([a-zA-Z0-9_-]+)/").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for ASexStoriesScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("asexstories.com/")
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
            Err(ScrapeError::AuthRequired(
                "asexstories: is_adult not set".into(),
            ))
        }
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired("asexstories: adult gate".into()));
        }
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("asexstories: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("Page Not Found.") {
            return Err(ScrapeError::NotFound);
        }

        let (title, author, author_url, author_local_id, desc, rating) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            let mut desc = String::new();
            let mut rating = String::new();

            if let Ok(t_sel) = Selector::parse("div.story-top-block h1") {
                title = doc
                    .select(&t_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }

            // Author: 2nd story-info-bl div's a.
            if let Ok(i_sel) = Selector::parse("div.story-info div.story-info-bl a[href]") {
                let mut as_: Vec<_> = doc.select(&i_sel).collect();
                if as_.len() >= 2 {
                    let a = as_.remove(1);
                    author = a.text().collect::<String>().trim().to_string();
                    author_url = a.value().attr("href").unwrap_or("").to_string();
                    author_local_id = author_url
                        .rsplit('/')
                        .next()
                        .unwrap_or("")
                        .replace(".html", "")
                        .to_string();
                }
            }

            // Description: first 150 chars of story-block text.
            if let Ok(s_sel) = Selector::parse("div.story-block") {
                let text = doc
                    .select(&s_sel)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default();
                let trimmed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
                let mut excerpt: String = trimmed.chars().take(150).collect();
                if !excerpt.is_empty() {
                    excerpt.push_str("...");
                    desc = format!("Excerpt from beginning of story: {excerpt}");
                }
            }

            // Rating from story-info-bl5 img title.
            if let Ok(r_sel) = Selector::parse("div.story-info-bl5 img[title]") {
                rating = doc
                    .select(&r_sel)
                    .next()
                    .map(|el| {
                        el.value()
                            .attr("title")
                            .unwrap_or("")
                            .replace("- Rate", "")
                            .trim()
                            .to_string()
                    })
                    .unwrap_or_default();
            }

            (title, author, author_url, author_local_id, desc, rating)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("asexstories: no title".into()));
        }

        // Chapters: div.pages a (first = the URL itself).
        let mut chapters = 1i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(p_sel) = Selector::parse("div.pages a[href]") {
                let count = doc.select(&p_sel).count() as i32;
                if count > 0 {
                    chapters = count + 1;
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ass_{}", story_id),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: now,
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
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
            let sel = Selector::parse("div.pages a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with('/') {
                        format!("https://www.asexstories.com{href}")
                    } else {
                        format!("{}{}", meta.source.trim_end_matches('/'), href)
                    };
                    // Keep only same-story chapter URLs (they contain the
                    // story name or /index pattern).
                    if url.contains(&meta.source.trim_end_matches('/')) || url.contains("/index") {
                        Some((title, url))
                    } else {
                        None
                    }
                })
                .collect()
        };

        let mut chapters = Vec::new();
        // First chapter is the URL itself.
        let first = fetch_chapter_text(client, &meta.source).await;
        chapters.push(Chapter {
            chapter_id: 1,
            title: "1".to_string(),
            content: first,
        });
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 2,
                title: if title.is_empty() {
                    format!("{}", i + 2)
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
    if let Ok(sel) = Selector::parse("div.story-block") {
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
        let s = ASexStoriesScraper::default();
        assert!(s.can_handle("http://www.asexstories.com/Halloween-party-with-the-phantom/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            ASexStoriesScraper::story_id(
                "http://www.asexstories.com/Halloween-party-with-the-phantom/"
            ),
            Some("Halloween-party-with-the-phantom".to_string())
        );
        assert_eq!(ASexStoriesScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn strips_rating_suffix() {
        let t = "Mature - Rate".replace("- Rate", "");
        assert_eq!(t.trim(), "Mature");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="story-block"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.story-block").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
