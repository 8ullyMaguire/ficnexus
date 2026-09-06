//! Literotica.com native adapter.
//!
//! Literotica's modern layout is partly JS-rendered, but the core story
//! page carries enough in the DOM + embedded JSON:
//! - Story page: `div._content_` with title/author in the header, category
//!   breadcrumbs, description in `div._introduction-wrap` or the info tab,
//!   chapter list in `section li._item_ > a` (href = /s/{slug}).
//! - Chapter body: `div[class^="_article__content_"]` with optional
//!   pagination (`nav._pagination_` → `?page=N`).
//!
//! This covers the per-fic scrape path (metadata + chapters + body).
//! Author-page listing + API series fetch (FFF library mode) are not part
//! of the crate's SiteScraper contract.

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct LiteroticaScraper;

#[async_trait]
impl SiteScraper for LiteroticaScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("literotica.com/s/") || url.contains("literotica.com/stories/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        // Title from <h1> in the article header, or og:title meta.
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("h1") {
            title = doc
                .select(&h1_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            if let Ok(og_sel) = Selector::parse("meta[property='og:title']") {
                title = doc
                    .select(&og_sel)
                    .next()
                    .and_then(|el| el.value().attr("content"))
                    .unwrap_or("")
                    .to_string();
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("literotica: no title".into()));
        }

        // Author: byline link. Modern layout: a[href*="/authors/"].
        let mut author = String::new();
        let mut author_url = String::new();
        if let Ok(a_sel) = Selector::parse("a[href*='/authors/']") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://www.literotica.com{h}");
                }
            }
        }

        // Status: embedded JS `state:"..."` (completed / in-progress).
        let mut status = "ongoing".to_string();
        if let Some(m) = Regex::new(r#"state:"([^"]+)"#)
            .ok()
            .and_then(|re| re.captures(&html))
        {
            let raw = m.get(1).map(|x| x.as_str()).unwrap_or("");
            if raw.contains("completed") {
                status = "complete".to_string();
            }
        }

        // Dates from embedded JS `"date_approve":"MM/DD/YYYY"`.
        let mut published = 0i64;
        let mut updated = 0i64;
        if let Ok(re) = Regex::new(r#""date_approve":"(\d\d/\d\d/\d\d\d\d)""#) {
            let mut dates: Vec<i64> = re
                .captures_iter(&html)
                .filter_map(|c| {
                    let d = c.get(1)?;
                    parse_us_date(d.as_str())
                })
                .collect();
            dates.sort();
            if let Some(first) = dates.first() {
                published = *first;
            }
            if let Some(last) = dates.last() {
                updated = *last;
            }
        }

        // Chapters: `section li[class^="_item_"] > a`.
        let mut chapters = 0i32;
        if let Ok(li_sel) = Selector::parse("section li[class^='_item_'] > a") {
            chapters = doc.select(&li_sel).count() as i32;
        }

        // Words: chapter list descriptions usually carry it; fall back to 0.
        let mut words = 0i64;
        if let Ok(w_sel) = Selector::parse("p[class^='_description_']") {
            for el in doc.select(&w_sel) {
                let t = el.text().collect::<String>();
                if t.contains("Words") {
                    words = t
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0);
                    break;
                }
            }
        }

        // Description: intro paragraph or info-tab desc.
        let mut desc = String::new();
        if let Ok(intro_sel) =
            Selector::parse("div[class^='_content_'] div[class^='_introduction-wrap'] p")
        {
            desc = doc
                .select(&intro_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        let now = chrono::Utc::now().timestamp_millis();
        if published == 0 {
            published = now;
        }
        if updated == 0 {
            updated = now;
        }

        Ok(FicMetadata {
            url_id: format!("ltr_{}", url.rsplit('/').next().unwrap_or("")),
            title,
            author,
            chapters,
            words,
            desc,
            published,
            updated,
            status,
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id: String::new(),
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
        let html = http::fetch(client, &meta.source).await?;

        // Collect chapter links into owned data first (drop the non-Send doc
        // before the per-chapter body fetch loop).
        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let li_sel = Selector::parse("section li[class^='_item_'] > a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&li_sel)
                .filter_map(|el| {
                    let title = el.text().collect::<String>().trim().to_string();
                    let href = el.value().attr("href")?.to_string();
                    let url = if href.starts_with("http") {
                        href
                    } else {
                        format!("https://www.literotica.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            // One-shot: the story page itself is the only chapter.
            let content = fetch_body(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_body(client, &url).await;
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

/// Fetch a Literotica chapter (with pagination) and extract the body.
async fn fetch_body(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let mut full = extract_article_body(&html);

    // Pagination: find the highest page number from the nav links (drop the
    // non-Send doc before the per-page fetch loop).
    let last_page = {
        let doc = Html::parse_document(&html);
        let mut last = 1;
        if let Ok(nav_sel) = Selector::parse("nav[class*='_pagination_'] a") {
            for a in doc.select(&nav_sel) {
                if let Some(href) = a.value().attr("href") {
                    if let Some(q) = href.split('?').nth(1) {
                        for kv in q.split('&') {
                            if let Some(page) = kv.strip_prefix("page=") {
                                if let Ok(n) = page.parse::<i32>() {
                                    last = last.max(n);
                                }
                            }
                        }
                    }
                }
            }
        }
        last
    };
    for page in 2..=last_page {
        let page_url = format!("{url}?page={page}");
        if let Ok(html2) = http::fetch(client, &page_url).await {
            full.push_str(&extract_article_body(&html2));
        }
    }
    full
}

fn extract_article_body(html: &str) -> String {
    let doc = Html::parse_document(html);
    let mut out = String::new();
    if let Ok(sel) = Selector::parse("div[class^='_article__content_']") {
        for el in doc.select(&sel) {
            out.push_str(&el.inner_html());
        }
    }
    // Legacy body container fallback.
    if out.is_empty() {
        if let Ok(sel) = Selector::parse("div.aa_ht") {
            for el in doc.select(&sel) {
                out.push_str(&el.inner_html());
            }
        }
    }
    out
}

fn parse_us_date(s: &str) -> Option<i64> {
    let mut parts = s.split('/');
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    let y: i32 = parts.next()?.parse().ok()?;
    let dt = chrono::NaiveDate::from_ymd_opt(y, m, d)?.and_hms_opt(0, 0, 0)?;
    Some(chrono::Utc.from_utc_datetime(&dt).timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches_story_urls() {
        let s = LiteroticaScraper;
        assert!(s.can_handle("https://www.literotica.com/s/my-story-title"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_us_dates() {
        assert!(parse_us_date("01/31/2024").is_some());
        assert_eq!(parse_us_date("bad"), None);
    }

    #[test]
    fn extracts_article_body() {
        let html = r#"<html><body><div class="_article__content_x"><p>Story text here.</p></div></body></html>"#;
        let out = extract_article_body(html);
        assert!(out.contains("Story text here."));
    }

    #[test]
    fn one_shot_fallback_works() {
        let html =
            r#"<html><body><div class="aa_ht"><div><p>Legacy body</p></div></div></body></html>"#;
        let out = extract_article_body(html);
        assert!(out.contains("Legacy body"));
    }
}
