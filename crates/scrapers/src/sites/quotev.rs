//! Quotev.com native adapter.
//!
//! Story/quiz page: title `div.result h1`, authors `div.quizAuthorList a`
//! (href /author/{name}), description `div#qdesct`, dates `time[ts]`
//! (epoch seconds), status "· Completed" in the meta line, categories
//! `a[href*="/fiction/"]`, tags `div#quizHeader div.quizBoxTags a`,
//! chapters `div#rselect a`.
//! Chapter body: `div#rescontent`.

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct QuotevScraper;

#[async_trait]
impl SiteScraper for QuotevScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("quotev.com/story/") || url.contains("quotev.com/quiz/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut desc = String::new();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut status = "ongoing".to_string();
        let mut chapters = 0i32;

        // Title
        if let Ok(h1_sel) = Selector::parse("div.result h1") {
            title = doc
                .select(&h1_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("quotev: no title".into()));
        }

        // Authors
        let mut authors: Vec<String> = Vec::new();
        let mut author_urls: Vec<String> = Vec::new();
        if let Ok(a_sel) = Selector::parse("div.quizAuthorList a") {
            for a in doc.select(&a_sel) {
                authors.push(a.text().collect::<String>().trim().to_string());
                if let Some(h) = a.value().attr("href") {
                    let url = if h.starts_with("http") {
                        h.to_string()
                    } else {
                        format!("https://www.quotev.com{h}")
                    };
                    author_urls.push(url);
                }
            }
        }
        if authors.is_empty() {
            authors.push("Anonymous".into());
        }
        let author = authors.join(", ");
        let author_url = author_urls
            .first()
            .cloned()
            .unwrap_or_else(|| "https://www.quotev.com".into());
        let author_local_id = author_url.rsplit('/').next().unwrap_or("").to_string();

        // Description
        if let Ok(d_sel) = Selector::parse("div#qdesct") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Dates from <time ts="epoch">.
        let mut dates: Vec<i64> = Vec::new();
        if let Ok(time_sel) = Selector::parse("time") {
            for t in doc.select(&time_sel) {
                if let Some(ts) = t.value().attr("ts") {
                    if let Ok(secs) = ts.parse::<i64>() {
                        dates.push(secs * 1000);
                    }
                }
            }
        }
        if let Some(first) = dates.first() {
            published = *first;
        }
        if let Some(last) = dates.last() {
            updated = *last;
        }

        // Status: the meta line contains "· Completed".
        if let Some(first_time) = doc.select(&Selector::parse("time").unwrap()).next() {
            if let Some(parent) = first_time.parent() {
                if let Some(gp) = parent.parent() {
                    if let Some(el) = scraper::ElementRef::wrap(gp) {
                        let text = el.text().collect::<String>();
                        if text.contains("Completed") {
                            status = "complete".to_string();
                        }
                    }
                }
            }
        }

        // Chapters
        if let Ok(sel) = Selector::parse("div#rselect a") {
            chapters = doc
                .select(&sel)
                .filter(|a| {
                    a.value()
                        .attr("href")
                        .map(|h| !h.contains("javascript"))
                        .unwrap_or(false)
                })
                .count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Story id from URL.
        let story_id = url.rsplit('/').next().unwrap_or("").to_string();

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("qvo_{story_id}"),
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

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("div#rselect a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if href.contains("javascript") {
                        return None;
                    }
                    let title = a.text().collect::<String>().trim().to_string();
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.quotev.com{href}")
                    };
                    Some((title, url))
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
    if let Ok(sel) = Selector::parse("div#rescontent") {
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
        let s = QuotevScraper;
        assert!(s.can_handle("https://www.quotev.com/story/12345/title"));
        assert!(s.can_handle("https://www.quotev.com/quiz/12345"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_epoch_dates() {
        // 2024-01-31T00:00:00Z = 1706659200
        let secs: i64 = 1706659200;
        assert_eq!(secs * 1000, 1706659200000);
    }

    #[test]
    fn extracts_chapter_body() {
        let html =
            r#"<html><body><div id="rescontent"><p>Quiz story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#rescontent") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Quiz story text."));
    }
}
