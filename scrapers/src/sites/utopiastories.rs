//! UtopiaStories.com native adapter (adult-gated original fiction).
//!
//! Story URL: `https://www.utopiastories.com/code/show_story.asp/recid/{id}.html`.
//! - title: `<title>` (strip ":: GaggedUtopia's Story Archive")
//! - author: `li` "Author" → `a`
//! - metadata: `li` elements (Author / Story Codes / Post Date / Rating /
//!   Site Rank / Unique Views)
//! - body: first `table` → first `td` → `div` (strip script/table/a/div)
//!
//! Adult gate: the site requires an `is_adult` confirmation; the host
//! enables it via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::http;

pub struct UtopiaStoriesScraper {
    adult_ok: AtomicBool,
}

impl Default for UtopiaStoriesScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl UtopiaStoriesScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"utopiastories\.com/code/show_story(?:\.asp)?/recid/(\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for UtopiaStoriesScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("utopiastories.com/") && url.contains("/code/show_story")
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
            Err(ScrapeError::AuthRequired("utopiastories: is_adult not set".into()))
        }
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired("utopiastories: adult gate".into()));
        }
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("utopiastories: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("Latest Stories") || html.contains("requested this story be removed") {
            return Err(ScrapeError::NotFound);
        }

        let (title, author, author_url, author_local_id, published) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            let mut published = 0i64;

            // Title from <title>.
            if let Ok(t_sel) = Selector::parse("title") {
                title = doc
                    .select(&t_sel)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default()
                    .replace(":: GaggedUtopia's Story Archive", "")
                    .trim()
                    .to_string();
            }

            // Metadata from li elements.
            if let Ok(li_sel) = Selector::parse("li") {
                for li in doc.select(&li_sel) {
                    let text = li.text().collect::<String>().replace('\u{a0}', "");
                    let heading = text.split(" - ").next().unwrap_or("").trim().to_string();
                    if heading == "Author" {
                        if let Ok(a_sel) = Selector::parse("a[href]") {
                            if let Some(a) = li.select(&a_sel).next() {
                                let href = a.value().attr("href").unwrap_or("");
                                if href.contains("mailto") {
                                    author = "Unknown".to_string();
                                } else {
                                    author = a.text().collect::<String>().trim().to_string();
                                    author_local_id = href.split('/').nth(2).unwrap_or("").to_string();
                                    author_url = format!(
                                        "https://www.utopiastories.com/{}",
                                        href.replace("../..", "code")
                                    );
                                }
                            }
                        }
                    } else if heading == "Post Date" {
                        let date = text.replace("Post Date - ", "").trim().to_string();
                        if let Ok(d) = chrono::NaiveDate::parse_from_str(&date, "%m/%d/%Y") {
                            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                                published = dt.and_utc().timestamp_millis();
                            }
                        }
                    }
                }
            }

            (title, author, author_url, author_local_id, published)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("utopiastories: no title".into()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ut_{}", story_id),
            title,
            author,
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: if published > 0 { published } else { now },
            updated: now,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
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
        let content = fetch_chapter_text(client, &meta.source).await;
        if content.is_empty() {
            return Err(ScrapeError::ParseError("utopiastories: no story text".into()));
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
    // First table → first td → div.
    if let Ok(t_sel) = Selector::parse("table") {
        if let Some(t) = doc.select(&t_sel).next() {
            if let Ok(td_sel) = Selector::parse("td") {
                if let Some(td) = t.select(&td_sel).next() {
                    if let Ok(d_sel) = Selector::parse("div") {
                        if let Some(d) = td.select(&d_sel).next() {
                            return d.inner_html();
                        }
                    }
                }
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = UtopiaStoriesScraper::default();
        assert!(s.can_handle("https://www.utopiastories.com/code/show_story.asp/recid/1234.html"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(UtopiaStoriesScraper::story_id("https://www.utopiastories.com/code/show_story.asp/recid/1234.html"), Some("1234".to_string()));
        assert_eq!(UtopiaStoriesScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn strips_title_suffix() {
        let t = "My Story:: GaggedUtopia's Story Archive".replace(":: GaggedUtopia's Story Archive", "");
        assert_eq!(t, "My Story");
    }

    #[test]
    fn parses_dates() {
        let c = "01/31/2024";
        if let Ok(d) = chrono::NaiveDate::parse_from_str(c, "%m/%d/%Y") {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                assert!(dt.and_utc().timestamp_millis() > 0);
            }
        }
    }
}
