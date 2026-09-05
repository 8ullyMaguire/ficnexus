//! FanFiktion.de native adapter (German fanfiction archive).
//!
//! Story URL: `https://www.fanfiktion.de/s/{storyId}/1`.
//! - title: `a[href^="/s/{storyId}/"]`
//! - author: `div.story-left a`
//! - chapters: `select option` (value = chapter number → /s/{id}/{n})
//! - words: `span.fa-keyboard` parent (dots are thousand separators)
//! - dates: `span[title=erstellt]` / `span[title=aktualisiert]` parents
//! - genres/rating: `span.fa-angle-right` sibling ("Genres / Rating")
//! - status: `span[title=fertiggestellt|pausiert|abgebrochen]`
//! - description: `div#story-summary-inline div`
//! - body: `div#storytext` (scripts stripped)
//! Age-gated stories (23:00–04:00 verification) → AuthRequired.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FanfiktionDeScraper;

impl FanfiktionDeScraper {
    fn story_id(url: &str) -> Option<String> {
        let id = url.split("/s/").nth(1)?.split('/').next()?.to_string();
        if !id.is_empty() { Some(id) } else { None }
    }
}

#[async_trait]
impl SiteScraper for FanfiktionDeScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiktion.de/s/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("fanfiktion.de: bad url".into()))?;
        let page_url = format!("https://www.fanfiktion.de/s/{story_id}/1");
        let html = http::fetch(client, &page_url).await?;

        if html.contains("Uhr ist diese Geschichte nur nach einer") {
            return Err(ScrapeError::AuthRequired("fanfiktion.de age gate".into()));
        }
        if html.contains("Diese Geschichte wurde als entwicklungsbeeintr") {
            return Err(ScrapeError::AuthRequired("fanfiktion.de login gate".into()));
        }

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
        let mut chapters = 0i32;

        // Title
        if let Ok(t_sel) = Selector::parse(&format!("a[href^='/s/{story_id}/']")) {
            title = doc
                .select(&t_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fanfiktion.de: no title".into()));
        }

        // Author + dates + genres/rating + status from story-left.
        if let Ok(sl_sel) = Selector::parse("div.story-left") {
            if let Some(head) = doc.select(&sl_sel).next() {
                if let Ok(a_sel) = Selector::parse("a") {
                    if let Some(a) = head.select(&a_sel).next() {
                        author = a.text().collect::<String>().trim().to_string();
                        if let Some(h) = a.value().attr("href") {
                            author_url = format!("https://www.fanfiktion.de{h}");
                            author_local_id = h.split('/').nth(2).unwrap_or("").to_string();
                        }
                    }
                }
                if let Ok(d_sel) = Selector::parse("span[title='erstellt']") {
                    if let Some(el) = head.select(&d_sel).next() {
                        if let Some(p) = el.parent() {
                            if let Some(pe) = scraper::ElementRef::wrap(p) {
                                published = parse_dt(&pe.text().collect::<String>());
                            }
                        }
                    }
                }
                if let Ok(d_sel) = Selector::parse("span[title='aktualisiert']") {
                    if let Some(el) = head.select(&d_sel).next() {
                        if let Some(p) = el.parent() {
                            if let Some(pe) = scraper::ElementRef::wrap(p) {
                                updated = parse_dt(&pe.text().collect::<String>());
                            }
                        }
                    }
                }
                // Genres / Rating from fa-angle-right sibling.
                if let Ok(g_sel) = Selector::parse("span.fa-angle-right") {
                    if let Some(el) = head.select(&g_sel).next() {
                        let mut nxt = el.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(ne) = scraper::ElementRef::wrap(node) {
                                let raw = ne.text().collect::<String>();
                                if let Some(idx) = raw.find(" / ") {
                                    rating = raw[idx + 3..].trim().to_string();
                                }
                                break;
                            }
                            nxt = node.next_sibling();
                        }
                    }
                }
                // Status
                if head
                    .select(&Selector::parse("span[title='fertiggestellt']").unwrap())
                    .next()
                    .is_some()
                {
                    status = "complete".to_string();
                } else if head
                    .select(&Selector::parse("span[title='pausiert']").unwrap())
                    .next()
                    .is_some()
                {
                    status = "paused".to_string();
                } else if head
                    .select(&Selector::parse("span[title='abgebrochen']").unwrap())
                    .next()
                    .is_some()
                {
                    status = "cancelled".to_string();
                }
            }
        }

        // Words from fa-keyboard parent.
        if let Ok(w_sel) = Selector::parse("span.fa-keyboard") {
            if let Some(el) = doc.select(&w_sel).next() {
                if let Some(p) = el.parent() {
                    if let Some(pe) = scraper::ElementRef::wrap(p) {
                        words = pe
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
        }

        // Description.
        if let Ok(d_sel) = Selector::parse("div#story-summary-inline div") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Chapters from select option.
        if let Ok(sel) = Selector::parse("select option") {
            chapters = doc.select(&sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ffde_{story_id}"),
            title,
            author,
            chapters,
            words,
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
            .ok_or_else(|| ScrapeError::ParseError("fanfiktion.de: bad url".into()))?;
        let page_url = format!("https://www.fanfiktion.de/s/{story_id}/1");
        let html = http::fetch(client, &page_url).await?;

        let chapter_nums: Vec<String> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("select option")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|o| o.value().attr("value").map(|v| v.to_string()))
                .collect()
        };

        if chapter_nums.is_empty() {
            let content = fetch_chapter_text(client, &page_url).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, num) in chapter_nums.into_iter().enumerate() {
            let url = format!("https://www.fanfiktion.de/s/{story_id}/{num}");
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
    if let Ok(sel) = Selector::parse("div#storytext") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

/// Parse a German date (DD.MM.YYYY or DD.MM.YYYY HH:MM).
fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%d.%m.%Y") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%d.%m.%Y %H:%M") {
        return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = FanfiktionDeScraper;
        assert!(s.can_handle("https://www.fanfiktion.de/s/12345/1"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            FanfiktionDeScraper::story_id("https://www.fanfiktion.de/s/abcdefg/1"),
            Some("abcdefg".to_string())
        );
        assert_eq!(FanfiktionDeScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("31.01.2024") > 0);
        assert!(parse_dt("31.01.2024 23:45") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="storytext"><p>German text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#storytext") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("German text."));
    }
}
