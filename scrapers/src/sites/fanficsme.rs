//! Fanfics.me native adapter (Russian fanfiction site, login for R+ ratings).
//!
//! Story URL: `https://fanfics.me/read.php?id={id}`.
//! Metadata from `div.FicHead` (label→value divs, Russian labels):
//! - title: `h1` (minus the `(гет)` category span)
//! - authors: `a.user` under the Авторы label
//! - rating: Рейтинг label (General / PG-13 / R / NC-17)
//! - genre: Жанр, warnings: Предупреждение
//! - status: Статус label span class color (green=complete, red=ongoing,
//!   blue=hiatus)
//! - words: Размер label link digits
//! - dates: `span.DateUpdate` ("DD.MM.YYYY - DD.MM.YYYY")
//! Chapters: `ul.FicContents li a`; body: `div#c{n}` from
//! `/read.php?id={id}&chapter={n}`.
//!
//! Login: ratings above General require an account; POST `name`/`pass` to
//! `/autent.php`.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::{http, login};

pub struct FanficsMeScraper;

impl FanficsMeScraper {
    fn story_id(url: &str) -> Option<String> {
        let id = url.split("id=").nth(1)?.split('&').next()?.to_string();
        if id.chars().all(|c| c.is_ascii_digit()) {
            Some(id)
        } else {
            None
        }
    }

    /// Find the value div following a Russian label div.
    fn meta_value<'a>(doc: &'a Html, label: &str) -> Option<scraper::ElementRef<'a>> {
        let sel = Selector::parse("div.FicHead div").ok()?;
        let mut prev: Option<scraper::ElementRef<'a>> = None;
        for el in doc.select(&sel) {
            let text = el.text().collect::<String>();
            if text.trim_start().starts_with(label) && text.trim_end().ends_with(':') {
                prev = Some(el);
            } else if let Some(_p) = prev {
                // The div following the label is the value (skip empties).
                if !text.trim().is_empty() {
                    return Some(el);
                }
            }
        }
        None
    }
}

#[async_trait]
impl SiteScraper for FanficsMeScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfics.me/") && (url.contains("read.php?id=") || url.contains("/fic"))
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        let url = "https://fanfics.me/autent.php";
        // Prime cookies.
        let _ = http::fetch(client, url).await;
        login::post_login(
            client,
            url,
            &[
                ("name", creds.username.clone()),
                ("pass", creds.password.clone()),
            ],
        )
        .await?;
        // If the login page form is still present, credentials failed.
        let check = http::fetch(client, "https://fanfics.me/").await.unwrap_or_default();
        if check.contains(r#"name="autent""#) {
            return Err(ScrapeError::AuthRequired("fanfics.me login failed".into()));
        }
        Ok(())
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("fanfics.me: bad url".into()))?;
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut rating = String::new();
        let mut status = "ongoing".to_string();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut words = 0i64;
        let mut desc = String::new();

        // Title + category span from h1.
        if let Ok(h1_sel) = Selector::parse("div.FicHead h1") {
            if let Some(h1) = doc.select(&h1_sel).next() {
                let mut t = String::new();
                for child in h1.children() {
                    if let Some(el) = scraper::ElementRef::wrap(child) {
                        if el.value().name() != "span" {
                            t.push_str(&el.text().collect::<String>());
                        }
                    }
                }
                title = t.replace('\u{a0}', " ").trim().to_string();
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fanfics.me: no title".into()));
        }

        // Rating (need it before deciding status).
        if let Some(v) = Self::meta_value(&doc, "Рейтинг") {
            rating = v.text().collect::<String>().trim().to_string();
        }

        // Authors from the Авторы label (first a.user).
        if let Some(v) = Self::meta_value(&doc, "Авторы") {
            if let Ok(a_sel) = Selector::parse("a.user") {
                if let Some(a) = v.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://fanfics.me{h}");
                        author_local_id = h.split("/user").nth(1).unwrap_or("").to_string();
                    }
                }
            }
            if author.is_empty() {
                author = "Anonymous".to_string();
            }
        }

        // Status from Статус label span color.
        if let Some(v) = Self::meta_value(&doc, "Статус") {
            if let Ok(span_sel) = Selector::parse("span") {
                if let Some(span) = v.select(&span_sel).next() {
                    let cls = span.value().attr("class").unwrap_or("");
                    if cls.contains("green") {
                        status = "complete".to_string();
                    } else if cls.contains("red") {
                        status = "ongoing".to_string();
                    } else if cls.contains("blue") {
                        status = "hiatus".to_string();
                    }
                }
            }
        }

        // Words from Размер label link.
        if let Some(v) = Self::meta_value(&doc, "Размер") {
            if let Ok(a_sel) = Selector::parse("a") {
                if let Some(a) = v.select(&a_sel).next() {
                    words = a
                        .text()
                        .collect::<String>()
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0);
                }
            }
        }

        // Dates.
        if let Ok(d_sel) = Selector::parse("span.DateUpdate") {
            if let Some(d) = doc.select(&d_sel).next() {
                let raw = d.text().collect::<String>();
                let parts: Vec<&str> = raw.split(" - ").collect();
                if parts.len() == 2 {
                    published = parse_dt(parts[0].trim());
                    updated = parse_dt(parts[1].trim());
                }
            }
        }

        // Description.
        if let Ok(s_sel) = Selector::parse(&format!("div#summary_{story_id}")) {
            desc = doc
                .select(&s_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Chapters.
        let mut chapters = 0i32;
        if let Ok(ul_sel) = Selector::parse("ul.FicContents li a[href]") {
            chapters = doc.select(&ul_sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fms_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
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
        let story_id = Self::story_id(&meta.source).ok_or_else(|| ScrapeError::ParseError("fanfics.me: bad url".into()))?;
        let html = http::fetch(client, &meta.source).await?;

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("ul.FicContents li a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://fanfics.me{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        let urls: Vec<(String, String)> = if links.is_empty() {
            vec![(
                meta.title.clone(),
                format!("https://fanfics.me/read.php?id={story_id}&chapter=0"),
            )]
        } else {
            links
        };

        let mut chapters = Vec::new();
        for (i, (title, url)) in urls.into_iter().enumerate() {
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
    // Chapter number from the URL (&chapter=N), body is div#cN.
    let chapter = url.split("chapter=").nth(1).unwrap_or("0");
    let doc = Html::parse_document(&html);
    let Ok(sel) = Selector::parse(&format!("div#c{chapter}")) else {
        return String::new();
    };
    doc.select(&sel)
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return 0;
    }
    let (Ok(d), Ok(mo), Ok(y)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<i32>(),
    ) else {
        return 0;
    };
    let Some(date) = chrono::NaiveDate::from_ymd_opt(y, mo, d) else {
        return 0;
    };
    let Some(dt) = date.and_hms_opt(0, 0, 0) else {
        return 0;
    };
    chrono::Utc.from_utc_datetime(&dt).timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = FanficsMeScraper;
        assert!(s.can_handle("https://fanfics.me/read.php?id=137282"));
        assert!(s.can_handle("https://fanfics.me/fic137282"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            FanficsMeScraper::story_id("https://fanfics.me/read.php?id=137282&chapter=2"),
            Some("137282".to_string())
        );
        assert_eq!(FanficsMeScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("22.04.2020") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="c2"><p>Russian text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div#c2").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Russian text."));
    }
}
