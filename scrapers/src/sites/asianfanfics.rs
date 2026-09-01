//! AsianFanFics.com native adapter.
//!
//! Story page: `https://www.asianfanfics.com/story/view/{id}`.
//! - title: `h1`
//! - authors: `header.flow-root` → `span` "by" → `a[href*="/profile/u/"]`
//! - chapters: `aside a[data-toc-chapter]` (skip "Foreword")
//! - status: `span` text "Completed"
//! - description: htmx fetch of `div[hx-get^="/htmx/story/{id}/"]` →
//!   `div#story-description`
//! - dates: `<time>` tags (first = published, last = updated)
//! - word count: `span` "Total word count:" sibling
//! Chapter body: htmx fetch of `div[hx-get^="/htmx/chapter/"]` →
//! `div#user-submitted-body`.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct AsianFanFicsScraper;

#[async_trait]
impl SiteScraper for AsianFanFicsScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("asianfanfics.com/story/view/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("Please subscribe to read further chapters") {
            return Err(ScrapeError::AuthRequired("subscriber-only story".into()));
        }

        // Parse everything into owned values inside a scoped block: the
        // parsed document (scraper::Html) is not Send, so it must be
        // dropped before the description htmx fetch below.
        let (
            title,
            author,
            author_url,
            author_local_id,
            story_id,
            status,
            words,
            published,
            updated,
            chapters,
            desc_url,
        ) = {
            let doc = Html::parse_document(&html);

            // Title
            let mut title = String::new();
            if let Ok(h1_sel) = Selector::parse("h1") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }

            // Story id
            let story_id = url
                .split("/story/view/")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or("")
                .to_string();

            // Authors (multi-author)
            let mut authors: Vec<String> = Vec::new();
            let mut author_urls: Vec<String> = Vec::new();
            if let Ok(header_sel) = Selector::parse("header.flow-root") {
                if let Some(header) = doc.select(&header_sel).next() {
                    if let Ok(a_sel) = Selector::parse("a[href*='/profile/u/']") {
                        for a in header.select(&a_sel) {
                            authors.push(a.text().collect::<String>().trim().to_string());
                            if let Some(h) = a.value().attr("href") {
                                author_urls.push(format!("https://www.asianfanfics.com{h}"));
                            }
                        }
                    }
                }
            }
            let author = authors.join(", ");
            let author_url = author_urls.first().cloned().unwrap_or_default();
            let author_local_id = author_url.rsplit('/').next().unwrap_or("").to_string();

            // Status
            let status = if doc
                .select(&Selector::parse("header.flow-root span").unwrap())
                .any(|el| el.text().collect::<String>().trim() == "Completed")
            {
                "complete"
            } else {
                "ongoing"
            }
            .to_string();

            // Word count
            let mut words = 0i64;
            if let Ok(span_sel) = Selector::parse("span") {
                for span in doc.select(&span_sel) {
                    if span.text().collect::<String>().contains("Total word count:") {
                        let mut nxt = span.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                let t = el.text().collect::<String>();
                                words = t
                                    .chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .collect::<String>()
                                    .parse()
                                    .unwrap_or(0);
                                break;
                            }
                            nxt = node.next_sibling();
                        }
                        break;
                    }
                }
            }

            // Dates from <time> tags.
            let mut dates: Vec<i64> = Vec::new();
            if let Ok(time_sel) = Selector::parse("time") {
                for t in doc.select(&time_sel) {
                    if let Some(dt) = t.value().attr("datetime") {
                        if let Some(ms) = parse_dt(dt) {
                            dates.push(ms);
                        }
                    }
                }
            }
            let published = dates.first().copied().unwrap_or_else(now_ms);
            let updated = dates.last().copied().unwrap_or(published);

            // Chapters
            let mut chapters = 0i32;
            if let Ok(toc_sel) = Selector::parse("aside a[data-toc-chapter]") {
                chapters = doc.select(&toc_sel).count() as i32;
            }
            if chapters == 0 {
                chapters = 1;
            }

            // Description htmx endpoint (fetched after doc is dropped).
            let mut desc_url = String::new();
            if let Ok(desc_sel) = Selector::parse("div[hx-get^='/htmx/story/']") {
                if let Some(div) = doc.select(&desc_sel).next() {
                    if let Some(hx) = div.value().attr("hx-get") {
                        desc_url = format!("https://www.asianfanfics.com{hx}");
                    }
                }
            }

            (
                title, author, author_url, author_local_id, story_id,
                status, words, published, updated, chapters, desc_url,
            )
        }; // doc dropped here

        // Description: htmx fetch (no non-Send borrows alive now).
        let mut desc = String::new();
        if !desc_url.is_empty() {
            if let Ok(dhtml) = http::fetch(client, &desc_url).await {
                let ddoc = Html::parse_document(&dhtml);
                if let Ok(sd_sel) = Selector::parse("div#story-description") {
                    desc = ddoc
                        .select(&sd_sel)
                        .next()
                        .map(|el| el.inner_html())
                        .unwrap_or_default();
                }
            }
        }

        Ok(FicMetadata {
            url_id: format!("aff_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published,
            updated,
            status,
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
        let html = http::fetch(client, &meta.source).await?;

        // Collect chapter links into owned data first.
        let links: Vec<String> = {
            let doc = Html::parse_document(&html);
            let toc_sel = Selector::parse("aside a[data-toc-chapter]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&toc_sel)
                .filter_map(|a| {
                    let text = a.text().collect::<String>().trim().to_string();
                    if text == "Foreword" {
                        return None;
                    }
                    let href = a.value().attr("href")?;
                    if href.starts_with("http") {
                        Some(href.to_string())
                    } else {
                        Some(format!("https://www.asianfanfics.com{href}"))
                    }
                })
                .collect()
        };

        if links.is_empty() {
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, url) in links.into_iter().enumerate() {
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

/// Fetch an Asianfanfics chapter via its htmx endpoint and extract
/// `div#user-submitted-body`.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };

    // Extract the htmx chapter URL into owned data, dropping the non-Send
    // doc before the inner fetch.
    let chap_url = {
        let doc = Html::parse_document(&html);
        let mut found = String::new();
        if let Ok(hx_sel) = Selector::parse("div[hx-get^='/htmx/chapter/']") {
            if let Some(div) = doc.select(&hx_sel).next() {
                if let Some(hx) = div.value().attr("hx-get") {
                    found = format!("https://www.asianfanfics.com{hx}");
                }
            }
        }
        found
    };

    if chap_url.is_empty() {
        return String::new();
    }
    if let Ok(chap_html) = http::fetch(client, &chap_url).await {
        let cdoc = Html::parse_document(&chap_html);
        if let Ok(body_sel) = Selector::parse("div#user-submitted-body") {
            if let Some(el) = cdoc.select(&body_sel).next() {
                return el.inner_html();
            }
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> Option<i64> {
    let s = s.trim_end_matches('Z');
    let s = s.split('.').next().unwrap_or(s);
    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|dt| chrono::Utc.from_utc_datetime(&dt).timestamp_millis())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = AsianFanFicsScraper;
        assert!(s.can_handle("https://www.asianfanfics.com/story/view/1234567/title"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let url = "https://www.asianfanfics.com/story/view/1234567/title-here";
        let id = url
            .split("/story/view/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap();
        assert_eq!(id, "1234567");
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("2024-01-31T10:00:00").is_some());
        assert!(parse_dt("2024-01-31T10:00:00.000Z").is_some());
        assert_eq!(parse_dt("garbage"), None);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="user-submitted-body"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#user-submitted-body") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
