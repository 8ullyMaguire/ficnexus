//! FictionHunt.com native adapter (AO3-style Fate/HP archive).
//!
//! Story URL: `https://fictionhunt.com/stories/{id}/{title}` or
//! `https://fictionhunt.com/read/{id}/1`.
//! - title: `h1.Story__title`
//! - author: `div.StoryContents__meta a` (first)
//! - description: `h5` "Summary" → next `div`
//! - dates: `label` "Published:"/"Last Updated:" → next `time[datetime]`
//! - status: `div.dates label` (Completed vs In-Progress)
//! - genre/chars/ships/fandoms from tags links
//! - chapters: `ul.StoryContents__chapters li a`
//! - body: `div.StoryChapter__text`
//!
//! Login (site has toggled requiring it): GET `/login` for `_token`,
//! POST `identifier`/`password`/`remember`/`_token` → `/login`.

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::{http, login};
use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};

pub struct FictionHuntScraper;

impl FictionHuntScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"fictionhunt\.com/(?:read|stories)/([0-9a-z]+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for FictionHuntScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fictionhunt.com/") && (url.contains("/stories/") || url.contains("/read/"))
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        let login_url = "https://fictionhunt.com/login";
        let html = http::fetch(client, login_url).await?;
        let token = login::hidden_input(&html, "_token")
            .ok_or_else(|| ScrapeError::ParseError("fictionhunt: no _token".into()))?;
        login::post_login(
            client,
            login_url,
            &[
                ("identifier", creds.username.clone()),
                ("password", creds.password.clone()),
                ("remember", "on".to_string()),
                ("_token", token),
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
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("fictionhunt: bad url".into()))?;
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut desc = String::new();
        let mut status = "ongoing".to_string();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut genres: Vec<String> = Vec::new();
        let mut fandoms: Vec<String> = Vec::new();
        let mut ships: Vec<String> = Vec::new();
        let mut chapters = 0i32;

        // Title
        if let Ok(t_sel) = Selector::parse("h1.Story__title") {
            title = doc
                .select(&t_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fictionhunt: no title".into()));
        }

        // Author
        if let Ok(m_sel) = Selector::parse("div.StoryContents__meta a[href]") {
            if let Some(a) = doc.select(&m_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                author_url = a.value().attr("href").unwrap_or("").to_string();
                author_local_id = author_url.split('/').nth(4).unwrap_or("").to_string();
            }
        }

        // Description: h5 "Summary" → next div.
        for h5 in doc.select(&Selector::parse("h5").unwrap()) {
            if h5.text().collect::<String>().trim() == "Summary" {
                let mut nxt = h5.next_sibling();
                while let Some(node) = nxt {
                    if let Some(el) = scraper::ElementRef::wrap(node) {
                        if el.value().name() == "div" {
                            desc = el.inner_html();
                            break;
                        }
                    }
                    nxt = node.next_sibling();
                }
                break;
            }
        }

        // Dates: label Published:/Last Updated: → next time[datetime].
        if let Ok(lab_sel) = Selector::parse("label") {
            for lab in doc.select(&lab_sel) {
                let text = lab.text().collect::<String>().trim().to_string();
                if text == "Published:" {
                    let mut nxt = lab.next_sibling();
                    while let Some(node) = nxt {
                        if let Some(el) = scraper::ElementRef::wrap(node) {
                            if el.value().name() == "time" {
                                if let Some(dt) = el.value().attr("datetime") {
                                    published = parse_dt(dt);
                                }
                                break;
                            }
                        }
                        nxt = node.next_sibling();
                    }
                } else if text == "Last Updated:" {
                    let mut nxt = lab.next_sibling();
                    while let Some(node) = nxt {
                        if let Some(el) = scraper::ElementRef::wrap(node) {
                            if el.value().name() == "time" {
                                if let Some(dt) = el.value().attr("datetime") {
                                    updated = parse_dt(dt);
                                }
                                break;
                            }
                        }
                        nxt = node.next_sibling();
                    }
                }
            }
        }

        // Status from div.dates first label.
        if let Ok(d_sel) = Selector::parse("div.dates label") {
            if let Some(lab) = doc.select(&d_sel).next() {
                if lab.text().collect::<String>().contains("Completed") {
                    status = "complete".to_string();
                }
            }
        }

        // Genres
        if let Ok(g_sel) = Selector::parse("div.genres a") {
            for a in doc.select(&g_sel) {
                genres.push(a.text().collect::<String>().trim().to_string());
            }
        }
        // Fandoms
        if let Ok(f_sel) = Selector::parse("div.Story__type a[href*='fandoms=']") {
            for a in doc.select(&f_sel) {
                fandoms.push(
                    a.text()
                        .collect::<String>()
                        .replace(" Fanfiction", "")
                        .trim()
                        .to_string(),
                );
            }
        }
        // Ships
        if let Ok(s_sel) = Selector::parse("a[href*='pairings=']") {
            for a in doc.select(&s_sel) {
                ships.push(
                    a.text()
                        .collect::<String>()
                        .replace('+', "/")
                        .trim()
                        .to_string(),
                );
            }
        }

        // Chapters
        if let Ok(c_sel) = Selector::parse("ul.StoryContents__chapters li a[href]") {
            chapters = doc.select(&c_sel).count() as i32;
        }
        if chapters == 0 {
            return Err(ScrapeError::ParseError("fictionhunt: no chapters".into()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fh_{story_id}"),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status,
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!(
                "genres={};fandoms={};ships={};",
                genres.join(","),
                fandoms.join(","),
                ships.join(",")
            )),
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
            let sel = Selector::parse("ul.StoryContents__chapters li a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a
                        .select(&Selector::parse("span.chapter-title").unwrap())
                        .next()
                        .map(|s| s.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://fictionhunt.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
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
    if let Ok(sel) = Selector::parse("div.StoryChapter__text") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d"] {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = FictionHuntScraper;
        assert!(s.can_handle("https://fictionhunt.com/stories/7edm248/the-last-of-his-kind"));
        assert!(s.can_handle("http://fictionhunt.com/read/12411643/1"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            FictionHuntScraper::story_id(
                "https://fictionhunt.com/stories/7edm248/the-last-of-his-kind/chapters/1"
            ),
            Some("7edm248".to_string())
        );
        assert_eq!(
            FictionHuntScraper::story_id("https://fictionhunt.com/read/12411643/1"),
            Some("12411643".to_string())
        );
        assert_eq!(FictionHuntScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("2024-01-31 10:00:00") > 0);
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html =
            r#"<html><body><div class="StoryChapter__text"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.StoryChapter__text").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
