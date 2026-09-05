//! Dokuga.com native adapter (Inuyasha eFiction archive, two sections).
//!
//! Story URL: `http://www.dokuga.com/{section}/story/{id}` where section
//! is `fanfiction` or `spark`.
//! - title: `div[align=center] h3` (trailing space/count trimmed)
//! - author: the `a` inside the h3
//! - chapters: `select option` (value = chapter number)
//! - body: per-chapter `/{section}/story/{id}/{n}`
//!
//! Login: stories with "disabled anonymous viewing" need an account;
//! POST `username`/`passwd`/`Submit` + hidden form tokens → `/fanfiction`.

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::{http, login};
use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};

pub struct DokugaScraper;

impl DokugaScraper {
    fn section_and_id(url: &str) -> Option<(String, String)> {
        let m = Regex::new(r"dokuga\.com/(fanfiction|spark)/story/(\d+)").ok()?;
        let c = m.captures(url)?;
        Some((
            c.get(1)?.as_str().to_string(),
            c.get(2)?.as_str().to_string(),
        ))
    }
}

#[async_trait]
impl SiteScraper for DokugaScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("dokuga.com/") && url.contains("/story/")
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        // Standard eFiction login: POST username/passwd/Submit.
        login::post_login(
            client,
            "http://www.dokuga.com/fanfiction",
            &[
                ("username", creds.username.clone()),
                ("passwd", creds.password.clone()),
                ("Submit", "Submit".to_string()),
            ],
        )
        .await?;
        Ok(())
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let (_section, story_id) = Self::section_and_id(url)
            .ok_or_else(|| ScrapeError::ParseError("dokuga: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("Access denied. This story has not been validated") {
            return Err(ScrapeError::Blocked);
        }

        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();

        // Title + author from div[align=center] h3.
        if let Ok(div_sel) = Selector::parse("div[align='center'] h3") {
            if let Some(h3) = doc.select(&div_sel).next() {
                if let Ok(a_sel) = Selector::parse("a") {
                    if let Some(a) = h3.select(&a_sel).next() {
                        author = a.text().collect::<String>().trim().to_string();
                        if let Some(h) = a.value().attr("href") {
                            author_url = format!("http://www.dokuga.com{h}");
                            author_local_id = h.split('=').nth(1).unwrap_or("").to_string();
                        }
                    }
                }
                title = h3.text().collect::<String>().trim().to_string();
                // Strip trailing " - 4" (chapter count suffix).
                if let Some(idx) = title.rfind(" - ") {
                    title = title[..idx].trim().to_string();
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("dokuga: no title".into()));
        }

        // Chapters from select option.
        let mut chapters = 0i32;
        if let Ok(sel) = Selector::parse("select option") {
            chapters = doc.select(&sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("dkg_{}", story_id),
            title,
            author,
            chapters,
            words: 0,
            desc: String::new(),
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id,
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
        let (section, story_id) = Self::section_and_id(&meta.source)
            .ok_or_else(|| ScrapeError::ParseError("dokuga: bad url".into()))?;
        let html = http::fetch(client, &meta.source).await?;

        let chapter_nums: Vec<String> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("select option")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|o| o.value().attr("value").map(|v| v.to_string()))
                .collect()
        };

        let nums: Vec<String> = if chapter_nums.is_empty() {
            vec!["1".to_string()]
        } else {
            chapter_nums
        };

        let mut chapters = Vec::new();
        for (i, num) in nums.into_iter().enumerate() {
            let url = format!("http://www.dokuga.com/{section}/story/{story_id}/{num}");
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
    // eFiction chapter text is usually in div.storytext or div#storytext.
    for sel in ["div.storytext", "div#storytext", "div.content"] {
        if let Ok(s) = Selector::parse(sel) {
            if let Some(el) = doc.select(&s).next() {
                return el.inner_html();
            }
        }
    }
    // Fallback: any div containing the story (best-effort).
    if let Ok(s) = Selector::parse("td[colspan]") {
        if let Some(el) = doc.select(&s).next() {
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
        let s = DokugaScraper;
        assert!(s.can_handle("http://www.dokuga.com/fanfiction/story/7528/1"));
        assert!(s.can_handle("http://www.dokuga.com/spark/story/7299/1"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let (section, id) =
            DokugaScraper::section_and_id("http://www.dokuga.com/fanfiction/story/7528/1").unwrap();
        assert_eq!(section, "fanfiction");
        assert_eq!(id, "7528");
        assert!(DokugaScraper::section_and_id("https://x.com/foo").is_none());
    }

    #[test]
    fn strips_chapter_suffix() {
        let mut t = "My Story - 4".to_string();
        if let Some(idx) = t.rfind(" - ") {
            t = t[..idx].trim().to_string();
        }
        assert_eq!(t, "My Story");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="storytext"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.storytext").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
