//! AnEroticStory.com native adapter (adult-gated erotic fiction).
//!
//! Story URL: `https://www.aneroticstory.com/story/{id-title}`.
//! - title: `h1.tit` (title-cased)
//! - author: `a[rel=author]`
//! - description: first 350 chars of `div.tes`
//! - date: `div.infos strong`[1] (`%Y-%m-%d`)
//! - genre: `div.story a[href^="/genres/"]`
//! - body: `div.tes` (one-shot)
//!
//! Adult gate: `is_adult` confirmation via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};

pub struct AnEroticStoryScraper {
    adult_ok: AtomicBool,
}

impl Default for AnEroticStoryScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl AnEroticStoryScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"aneroticstory\.com/story/([^/]+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    fn title_case(s: &str) -> String {
        s.split(' ')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[async_trait]
impl SiteScraper for AnEroticStoryScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("aneroticstory.com/story/")
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
                "aneroticstory: is_adult not set".into(),
            ))
        }
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired(
                "aneroticstory: adult gate".into(),
            ));
        }
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("aneroticstory: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        let (title, author, author_url, author_local_id, desc, published, genre) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            let mut desc = String::new();
            let mut published = 0i64;
            let mut genre = String::new();

            if let Ok(t_sel) = Selector::parse("h1.tit") {
                title = doc
                    .select(&t_sel)
                    .next()
                    .map(|el| Self::title_case(&el.text().collect::<String>().trim()))
                    .unwrap_or_default();
            }

            if let Ok(a_sel) = Selector::parse("a[rel='author']") {
                if let Some(a) = doc.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://www.aneroticstory.com{h}");
                        author_local_id = h
                            .trim_end_matches('/')
                            .rsplit('/')
                            .next()
                            .unwrap_or("")
                            .to_string();
                    }
                }
            }

            // Description: first 350 chars of div.tes.
            if let Ok(s_sel) = Selector::parse("div.tes") {
                let text = doc
                    .select(&s_sel)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default();
                let trimmed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
                let mut excerpt: String = trimmed.chars().take(350).collect();
                if !excerpt.is_empty() {
                    excerpt.push_str("...");
                    desc = format!("Excerpt from beginning of story: {excerpt}");
                }
            }

            // Date: div.infos strong[1].
            if let Ok(i_sel) = Selector::parse("div.infos strong") {
                let strongs: Vec<_> = doc.select(&i_sel).collect();
                if let Some(s) = strongs.get(1) {
                    let date = s.text().collect::<String>().trim().to_string();
                    if let Ok(d) = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                            published = dt.and_utc().timestamp_millis();
                        }
                    }
                }
            }

            // Genre.
            if let Ok(g_sel) = Selector::parse("div.story a[href^='/genres/']") {
                genre = doc
                    .select(&g_sel)
                    .next()
                    .map(|el| Self::title_case(&el.text().collect::<String>().trim()))
                    .unwrap_or_default();
            }

            (
                title,
                author,
                author_url,
                author_local_id,
                desc,
                published,
                genre,
            )
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("aneroticstory: no title".into()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("aes_{}", story_id),
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
            extra_meta: Some(format!("genre={};", genre)),
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
            return Err(ScrapeError::ParseError(
                "aneroticstory: no story text".into(),
            ));
        }
        Ok(vec![Chapter {
            chapter_id: 1,
            title: "1".to_string(),
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
    if let Ok(sel) = Selector::parse("div.tes") {
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
        let s = AnEroticStoryScraper::default();
        assert!(s.can_handle("https://www.aneroticstory.com/story/565-daddy-explores-jessica"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            AnEroticStoryScraper::story_id(
                "https://www.aneroticstory.com/story/565-daddy-explores-jessica"
            ),
            Some("565-daddy-explores-jessica".to_string())
        );
        assert_eq!(AnEroticStoryScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn title_cases() {
        assert_eq!(
            AnEroticStoryScraper::title_case("daddy explores jessica"),
            "Daddy Explores Jessica"
        );
        assert_eq!(AnEroticStoryScraper::title_case("hello"), "Hello");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="tes"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.tes").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
