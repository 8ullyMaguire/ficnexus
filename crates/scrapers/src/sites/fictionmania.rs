//! FictionMania.tv native adapter (transgender fanfiction archive).
//!
//! Details page: `https://fictionmania.tv/details.html?storyID={id}`.
//! Table rows (key in `td b`, value in next `td`): Title, File Name,
//! File Size, Author, Date Added, Old/New Name, Other Names, Rating,
//! Complete, Categories, Key Words, Age, Synopsis, Reads.
//! Chapter text: `readtextstory.html?storyID={id}` → `pre` tag (one-shot).

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FictionManiaScraper;

impl FictionManiaScraper {
    fn story_id(url: &str) -> Option<String> {
        let id = url.split("storyID=").nth(1)?.split('&').next()?.to_string();
        if id.chars().all(|c| c.is_ascii_digit()) {
            Some(id)
        } else {
            None
        }
    }
}

#[async_trait]
impl SiteScraper for FictionManiaScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fictionmania.tv/") && (url.contains("storyID=") || url.contains("storyid="))
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("fictionmania: bad url".into()))?;
        let details_url = format!("https://fictionmania.tv/details.html?storyID={story_id}");
        let html = http::fetch(client, &details_url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut rating = String::new();
        let mut status = "ongoing".to_string();
        let mut published = 0i64;
        let mut desc = String::new();

        // Iterate table rows: key = td b text, value = td[1].
        if let Ok(tr_sel) = Selector::parse("table tr") {
            for tr in doc.select(&tr_sel) {
                let tds: Vec<_> = tr.select(&Selector::parse("td").unwrap()).collect();
                if tds.len() < 2 {
                    continue;
                }
                let key = tds[0]
                    .select(&Selector::parse("b").unwrap())
                    .next()
                    .map(|b| {
                        b.text()
                            .collect::<String>()
                            .trim()
                            .trim_end_matches(':')
                            .to_string()
                    })
                    .unwrap_or_else(|| {
                        tds[0]
                            .text()
                            .collect::<String>()
                            .trim()
                            .trim_end_matches(':')
                            .to_string()
                    });
                let value = tds[1].text().collect::<String>().trim().to_string();
                match key.as_str() {
                    "Title" => title = value.clone(),
                    "Author" => {
                        author = value.clone();
                        if let Ok(a_sel) = Selector::parse("a") {
                            if let Some(a) = tds[1].select(&a_sel).next() {
                                if let Some(h) = a.value().attr("href") {
                                    author_url = format!("https://fictionmania.tv{h}");
                                    author_local_id =
                                        h.rsplit('=').next().unwrap_or("").to_string();
                                }
                            }
                        }
                    }
                    "Date Added" => {
                        published = parse_dt(&value);
                    }
                    "Rating" => rating = value.clone(),
                    "Complete" => {
                        if value.eq_ignore_ascii_case("yes") {
                            status = "complete".to_string();
                        }
                    }
                    "Synopsis" => desc = value.clone(),
                    _ => {}
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fictionmania: no title".into()));
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("fm_{story_id}"),
            title,
            author,
            chapters: 1,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
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
        let story_id = Self::story_id(&meta.source)
            .ok_or_else(|| ScrapeError::ParseError("fictionmania: bad url".into()))?;
        let text_url = format!("https://fictionmania.tv/readtextstory.html?storyID={story_id}");
        let content = fetch_chapter_text(client, &text_url).await;
        if content.is_empty() {
            return Err(ScrapeError::ParseError(
                "fictionmania: no story text".into(),
            ));
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
    if let Ok(sel) = Selector::parse("pre") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

/// Parse "01/31/2024" or "01/31/24" (2-digit year → 2000s).
fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 3 {
        return 0;
    }
    let (Ok(mo), Ok(d)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) else {
        return 0;
    };
    let year = if parts[2].len() == 2 {
        2000 + parts[2].parse::<i32>().unwrap_or(0)
    } else {
        parts[2].parse::<i32>().unwrap_or(0)
    };
    let Some(date) = chrono::NaiveDate::from_ymd_opt(year, mo, d) else {
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
        let s = FictionManiaScraper;
        assert!(s.can_handle("https://fictionmania.tv/details.html?storyID=12345"));
        assert!(s.can_handle("https://fictionmania.tv/readtextstory.html?storyID=12345"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            FictionManiaScraper::story_id("https://fictionmania.tv/details.html?storyID=12345&x=1"),
            Some("12345".to_string())
        );
        assert_eq!(FictionManiaScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("01/31/2024") > 0);
        assert!(parse_dt("01/31/24") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><pre>Story text.</pre></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("pre") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
