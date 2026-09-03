//! Twisting the Hellmouth (tthfanfic.org) native adapter.
//!
//! Story URL: `https://www.tthfanfic.org/Story-{id}`.
//! - author: `a[href^="/AuthorStories-"]` (from story page)
//! - title + description: from the author's story list page
//!   (`div#st{id}.storylistitem` → `a.storylink` title, `div.storydesc`)
//! - chapters: `select[name=chapnav] option` (single-author) or the
//!   StoryInfo page's second `table.verticaltable` (multi-author)
//! - metadata: `table.verticaltable` tds — rating[2], words[4],
//!   datePublished[8], dateUpdated[9], completed[10]
//! - body: `div#storyinnerbody` (strip included `h3` chapter title)

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct TthFanficScraper;

#[async_trait]
impl SiteScraper for TthFanficScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("tthfanfic.org/Story-")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("<h2>Story Not Found</h2>") {
            return Err(ScrapeError::NotFound);
        }
        if html.contains("rated FR21 which is above your chosen filter level") {
            return Err(ScrapeError::AuthRequired("TtH FR21 adult gate".into()));
        }

        // Parse everything into owned values inside a scoped block: the
        // parsed document is not Send, so drop it before the author-page
        // fetch below.
        let (story_id, author, author_url, author_local_id, rating, words, status, published, updated, chapters) = {
            let doc = Html::parse_document(&html);

            // Story id from URL.
            let story_id = url
                .split("/Story-")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or("")
                .to_string();

            // Author from story page.
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            if let Ok(a_sel) = Selector::parse("a[href^='/AuthorStories-']") {
                if let Some(a) = doc.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://www.tthfanfic.org{h}");
                        if let Some(id) = h.split('/').nth(1).and_then(|s| s.split('-').nth(1)) {
                            author_local_id = id.to_string();
                        }
                    }
                }
            }

            // Metadata from the verticaltable.
            let mut rating = String::new();
            let mut words = 0i64;
            let mut status = "ongoing".to_string();
            let mut published = 0i64;
            let mut updated = 0i64;
            if let Ok(vt_sel) = Selector::parse("table.verticaltable") {
                if let Some(table) = doc.select(&vt_sel).next() {
                    let tds: Vec<String> = table
                        .select(&Selector::parse("td").unwrap())
                        .map(|td| td.text().collect::<String>().trim().to_string())
                        .collect();
                    if tds.len() > 10 {
                        rating = tds[2].clone();
                        words = tds[4].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                        if tds[10].contains("Yes") {
                            status = "complete".to_string();
                        }
                        published = parse_date(&tds[8]);
                        updated = parse_date(&tds[9]);
                    }
                }
            }

            // Chapters from the chapnav select (single-author).
            let mut chapters = 0i32;
            if let Ok(sel) = Selector::parse("select[name='chapnav'] option") {
                let n = doc.select(&sel).count() as i32;
                if n > 0 {
                    chapters = n;
                }
            }
            if chapters == 0 {
                chapters = 1;
            }

            (story_id, author, author_url, author_local_id, rating, words, status, published, updated, chapters)
        }; // doc dropped here

        // Title + description: from the author's story list page.
        let mut title = String::new();
        let mut desc = String::new();
        if !author_url.is_empty() {
            let story_div_id = format!("st{story_id}");
            if let Ok(auth_html) = http::fetch(client, &author_url).await {
                let adoc = Html::parse_document(&auth_html);
                let story_div_sel = Selector::parse(&format!("div#{story_div_id}.storylistitem"))
                    .unwrap_or_else(|_| Selector::parse("div.storylistitem").unwrap());
                if let Some(div) = adoc.select(&story_div_sel).next() {
                    if let Ok(t_sel) = Selector::parse("a.storylink") {
                        title = div
                            .select(&t_sel)
                            .next()
                            .map(|el| el.text().collect::<String>().trim().to_string())
                            .unwrap_or_default();
                    }
                    if let Ok(d_sel) = Selector::parse("div.storydesc") {
                        desc = div
                            .select(&d_sel)
                            .next()
                            .map(|el| el.inner_html())
                            .unwrap_or_default();
                    }
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("tth_{story_id}"),
            title: if title.is_empty() { format!("Story {story_id}") } else { title },
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
        let html = http::fetch(client, &meta.source).await?;

        // Chapter links from the chapnav select.
        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("select[name='chapnav'] option")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|o| {
                    let title = o.text().collect::<String>().trim().to_string();
                    let href = o.value().attr("value")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.tthfanfic.org{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        // Multi-author: StoryInfo page second verticaltable.
        let links = if links.is_empty() {
            let info_url = format!(
                "https://www.tthfanfic.org/StoryInfo-{}-1",
                meta.url_id.trim_start_matches("tth_")
            );
            let mut out = Vec::new();
            if let Ok(info_html) = http::fetch(client, &info_url).await {
                let idoc = Html::parse_document(&info_html);
                let tables: Vec<_> = idoc
                    .select(&Selector::parse("table.verticaltable").unwrap())
                    .collect();
                if let Some(table) = tables.get(1) {
                    for a in table.select(&Selector::parse("a[href^='/Story-']").unwrap()) {
                        let title = a.text().collect::<String>().trim().to_string();
                        let href = a.value().attr("href").unwrap_or("").to_string();
                        let url = format!("https://www.tthfanfic.org{href}");
                        out.push((title, url));
                    }
                }
            }
            out
        } else {
            links
        };

        if links.is_empty() {
            // One-chapter story: the story page itself.
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

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

/// Fetch a TtH chapter and extract `div#storyinnerbody` (strip `h3`).
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div#storyinnerbody") {
        if let Some(el) = doc.select(&sel).next() {
            // Strip any included h3 (chapter title) to avoid doubling.
            let mut out = String::new();
            for child in el.children() {
                if let Some(cref) = scraper::ElementRef::wrap(child) {
                    if cref.value().name() != "h3" {
                        out.push_str(&cref.inner_html());
                    }
                }
            }
            return out;
        }
    }
    String::new()
}

/// Parse a TtH date (site format is "MM/dd/yyyy" or similar).
fn parse_date(s: &str) -> i64 {
    let s = s.trim();
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
            return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
        }
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        if let Some(dt) = d.and_hms_opt(0, 0, 0) {
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
        let s = TthFanficScraper;
        assert!(s.can_handle("https://www.tthfanfic.org/Story-12345"));
        assert!(s.can_handle("https://www.tthfanfic.org/Story-12345/7/ChapterTitle"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let url = "https://www.tthfanfic.org/Story-12345/7/Chapter";
        let id = url.split("/Story-").nth(1).and_then(|s| s.split('/').next()).unwrap();
        assert_eq!(id, "12345");
    }

    #[test]
    fn parses_dates() {
        assert!(parse_date("01/31/2024") > 0);
        assert!(parse_date("2024-01-31") > 0);
        assert_eq!(parse_date("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="storyinnerbody"><h3>Ch1</h3><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut out = String::new();
        if let Ok(sel) = Selector::parse("div#storyinnerbody") {
            if let Some(el) = doc.select(&sel).next() {
                for child in el.children() {
                    if let Some(cref) = scraper::ElementRef::wrap(child) {
                        if cref.value().name() != "h3" {
                            out.push_str(&cref.inner_html());
                        }
                    }
                }
            }
        }
        assert!(out.contains("Story text."));
        assert!(!out.contains("Ch1"));
    }
}
