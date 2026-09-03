//! InkBunny.net native adapter (furry art/writing site).
//!
//! Story URL: `https://inkbunny.net/s/{id}` (or `submissionview.php?id=`).
//! - title: `h1` (second one)
//! - author: `table.pooltable a[href^="/gallery/"|"/scraps/"]`
//! - description: `div.elephant div.content span`
//! - keywords: `div#kw_scroll` next sibling links
//! - category/rating from detail divs
//! - date: `span#submittime_exact`
//! - body: `div#storysectionbar`
//!
//! Login: blocked submissions need an account; GET the page for a `token`
//! input, POST `token`/`username`/`password` → `/login_process.php`.

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::{http, login};

pub struct InkBunnyScraper;

impl InkBunnyScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"inkbunny\.net/(?:s/|submissionview\.php\?id=)(\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for InkBunnyScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("inkbunny.net/") && (url.contains("/s/") || url.contains("submissionview.php"))
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        // GET the submission page to find the login token.
        let page = "https://inkbunny.net/s/1";
        let html = http::fetch(client, page).await?;
        let token = login::hidden_input(&html, "token")
            .ok_or_else(|| ScrapeError::ParseError("inkbunny: no token".into()))?;
        login::post_login(
            client,
            "https://inkbunny.net/login_process.php",
            &[
                ("token", token),
                ("username", creds.username.clone()),
                ("password", creds.password.clone()),
            ],
        )
        .await?;
        Ok(())
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("inkbunny: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("ERROR: Invalid submission_id") {
            return Err(ScrapeError::NotFound);
        }

        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut desc = String::new();
        let mut genres: Vec<String> = Vec::new();
        let mut rating = String::new();
        let mut updated = 0i64;

        // Title: second h1.
        let h1s: Vec<_> = doc.select(&Selector::parse("h1").unwrap()).collect();
        if let Some(h1) = h1s.get(1) {
            title = h1.text().collect::<String>().trim().to_string();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("inkbunny: no title".into()));
        }

        // Author
        if let Ok(a_sel) = Selector::parse("table.pooltable a[href*='/gallery/'], table.pooltable a[href*='/scraps/']") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                author_local_id = author.clone();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://inkbunny.net{h}");
                }
            }
        }

        // Description
        if let Ok(e_sel) = Selector::parse("div.elephant div.content span") {
            desc = doc
                .select(&e_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }

        // Keywords
        if let Ok(kw_sel) = Selector::parse("div#kw_scroll") {
            if let Some(kw) = doc.select(&kw_sel).next() {
                let mut nxt = kw.next_sibling();
                while let Some(node) = nxt {
                    if let Some(el) = scraper::ElementRef::wrap(node) {
                        for a in el.select(&Selector::parse("a").unwrap()) {
                            genres.push(a.text().collect::<String>().trim().to_string());
                        }
                        break;
                    }
                    nxt = node.next_sibling();
                }
            }
        }

        // Rating
        if let Ok(d_sel) = Selector::parse("div.elephant div.content div") {
            for d in doc.select(&d_sel) {
                let text = d.text().collect::<String>();
                if text.starts_with("Rating:") {
                    rating = text.replacen("Rating:", "", 1).trim().to_string();
                    break;
                }
            }
        }

        // Date
        if let Ok(t_sel) = Selector::parse("span#submittime_exact") {
            if let Some(t) = doc.select(&t_sel).next() {
                let raw = t.text().collect::<String>();
                let date = raw.split(':').take(2).collect::<Vec<_>>().join(":");
                updated = parse_dt(&date);
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ib_{story_id}"),
            title,
            author,
            chapters: 1,
            words: 0,
            desc,
            published: if updated > 0 { updated } else { now },
            updated: if updated > 0 { updated } else { now },
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!("rating={};genres={};", rating, genres.join(","))),
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
            return Err(ScrapeError::ParseError("inkbunny: no story text".into()));
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
    if let Ok(sel) = Selector::parse("div#storysectionbar") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    for fmt in ["%b %d, %Y %I:%M %p", "%b %d, %Y %H:%M", "%Y-%m-%d %H:%M:%S"] {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = InkBunnyScraper;
        assert!(s.can_handle("https://inkbunny.net/s/1234567"));
        assert!(s.can_handle("https://inkbunny.net/submissionview.php?id=1234567"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(InkBunnyScraper::story_id("https://inkbunny.net/s/1234567"), Some("1234567".to_string()));
        assert_eq!(InkBunnyScraper::story_id("https://inkbunny.net/submissionview.php?id=1234567"), Some("1234567".to_string()));
        assert_eq!(InkBunnyScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("Jan 31, 2024 10:00 PM") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="storysectionbar"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div#storysectionbar").unwrap();
        let s = doc.select(&sel).next().map(|el| el.inner_html()).unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
