//! BDSMLibrary.com native adapter (adult-gated BDSM fiction archive).
//!
//! Story URL: `https://www.bdsmlibrary.com/stories/story.php?storyid={id}`.
//! - title: `<title>` (strip "BDSM Library - Story: " + backslashes)
//! - author: `a[href*="/stories/author.php?authorid="]`
//! - chapters: `a[href*="/stories/chapter.php?storyid={id}&chapterid="]`
//!   (chapter "added on" date in following tds)
//! - erotic tags: `a[href*="/stories/search.php?selectedcode"]`
//! - metadata: `td` "Added on:"/"Synopsis:"/"Size:"/"Comments:"
//! - body: `div.storyblock` (or `pre`)
//!
//! Adult gate: `is_adult` confirmation via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::http;

pub struct BdsmLibraryScraper {
    adult_ok: AtomicBool,
}

impl Default for BdsmLibraryScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl BdsmLibraryScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"bdsmlibrary\.com/stories/story\.php\?storyid=(\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for BdsmLibraryScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("bdsmlibrary.com/stories/story.php?storyid=")
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
            Err(ScrapeError::AuthRequired("bdsmlibrary: is_adult not set".into()))
        }
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired("bdsmlibrary: adult gate".into()));
        }
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("bdsmlibrary: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("The story does not exist") {
            return Err(ScrapeError::NotFound);
        }

        let (title, author, author_url, author_local_id, desc, published, tags) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            let mut desc = String::new();
            let mut published = 0i64;
            let mut tags: Vec<String> = Vec::new();

            if let Ok(t_sel) = Selector::parse("title") {
                title = doc
                    .select(&t_sel)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default()
                    .replace("BDSM Library - Story: ", "")
                    .replace('\\', "")
                    .trim()
                    .to_string();
            }

            if let Ok(a_sel) = Selector::parse("a[href*='/stories/author.php?authorid=']") {
                if let Some(a) = doc.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://www.bdsmlibrary.com{h}");
                        author_local_id = h.split('=').nth(1).unwrap_or("").to_string();
                    }
                }
            }
            if author.is_empty() {
                author = "Anonymous".to_string();
                author_url = "https://www.bdsmlibrary.com/".to_string();
                author_local_id = "0".to_string();
            }

            // Erotic tags.
            if let Ok(t_sel) = Selector::parse("a[href*='/stories/search.php?selectedcode']") {
                for a in doc.select(&t_sel) {
                    tags.push(a.text().collect::<String>().trim().to_string());
                }
            }

            // Metadata from tds.
            if let Ok(td_sel) = Selector::parse("td") {
                for td in doc.select(&td_sel) {
                    let text = td.text().collect::<String>();
                    if text.contains("Added on:") {
                        let val = text.replacen("Added on:", "", 1).trim().to_string();
                        published = parse_dt(&val);
                    } else if text.contains("Synopsis:") {
                        desc = text
                            .replace('\n', "")
                            .replacen("Synopsis:", "", 1)
                            .trim()
                            .to_string();
                    }
                }
            }

            (title, author, author_url, author_local_id, desc, published, tags)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("bdsmlibrary: no title".into()));
        }

        // Chapters.
        let chap_re = Regex::new(&format!(r"/stories/chapter\.php\?storyid={story_id}&chapterid=\d+$")).unwrap();
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(a_sel) = Selector::parse("a[href]") {
                chapters = doc
                    .select(&a_sel)
                    .filter(|a| a.value().attr("href").map(|h| chap_re.is_match(h)).unwrap_or(false))
                    .count() as i32;
            }
        }
        if chapters == 0 {
            return Err(ScrapeError::ParseError("bdsmlibrary: no chapters".into()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("bdsm_{}", story_id),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id,
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
        let story_id = Self::story_id(&meta.source).ok_or_else(|| ScrapeError::ParseError("bdsmlibrary: bad url".into()))?;
        let html = http::fetch(client, &meta.source).await?;

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let chap_re = Regex::new(&format!(r"/stories/chapter\.php\?storyid={story_id}&chapterid=\d+$")).unwrap();
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if !chap_re.is_match(href) {
                        return None;
                    }
                    let title = a.text().collect::<String>().trim().to_string();
                    let url = format!("https://www.bdsmlibrary.com{href}");
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            return Err(ScrapeError::ParseError("bdsmlibrary: no chapters".into()));
        }

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
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

fn parse_dt(s: &str) -> i64 {
    for fmt in ["%b %d, %Y", "%B %d, %Y", "%Y-%m-%d"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return dt.and_utc().timestamp_millis();
            }
        }
    }
    0
}

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div.storyblock") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    if let Ok(sel) = Selector::parse("pre") {
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
        let s = BdsmLibraryScraper::default();
        assert!(s.can_handle("https://www.bdsmlibrary.com/stories/story.php?storyid=1234"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(BdsmLibraryScraper::story_id("https://www.bdsmlibrary.com/stories/story.php?storyid=1234"), Some("1234".to_string()));
        assert_eq!(BdsmLibraryScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("Oct 24, 2016") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="storyblock"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.storyblock").unwrap();
        let s = doc.select(&sel).next().map(|el| el.inner_html()).unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
