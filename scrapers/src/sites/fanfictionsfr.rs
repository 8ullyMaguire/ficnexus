//! Fanfictions.fr native adapter (French archive).
//!
//! Chapters page: `https://www.fanfictions.fr/fanfictions/{fandom}/{id}/chapters.html`.
//! - title: `h1[itemprop=name]`
//! - author: `div[itemprop=author] a` (href ends {author_id}.html)
//! - published: `span.date-distance[data-date]` ("YYYY-MM-DD HH:MM:SS")
//! - status: `p[title="Statut de la fanfiction"] span.badge`
//!   ("En cours" → ongoing, "Terminée"/"One-shot" → complete)
//! - description: `p[itemprop=abstract]`
//! - chapters: `div.card.chapter h2 a`
//! Chapter body: `div#readarea`.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct FanfictionsFrScraper;

#[async_trait]
impl SiteScraper for FanfictionsFrScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfictions.fr/fanfictions/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;

        if html.contains("id=\"alertInactiveFic\"") {
            return Err(ScrapeError::NotFound);
        }

        let doc = Html::parse_document(&html);

        // Title
        let mut title = String::new();
        if let Ok(h1_sel) = Selector::parse("h1[itemprop='name']") {
            title = doc
                .select(&h1_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fanfictions.fr: no title".into()));
        }

        // Author
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        if let Ok(a_sel) = Selector::parse("div[itemprop='author'] a") {
            if let Some(a) = doc.select(&a_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("https://www.fanfictions.fr{h}");
                    author_local_id = h
                        .rsplit('/')
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(".html")
                        .to_string();
                }
            }
        }

        // Published from span.date-distance data-date.
        let mut published = 0i64;
        if let Ok(d_sel) = Selector::parse("span.date-distance") {
            if let Some(el) = doc.select(&d_sel).next() {
                if let Some(ds) = el.value().attr("data-date") {
                    published = parse_dt(ds);
                }
            }
        }

        // Status
        let mut status = "ongoing".to_string();
        if let Ok(p_sel) = Selector::parse("p[title='Statut de la fanfiction'] span.badge") {
            if let Some(el) = doc.select(&p_sel).next() {
                let raw = el.text().collect::<String>().trim().to_string();
                status = match raw.as_str() {
                    "Terminée" | "One-shot" => "complete".to_string(),
                    _ => "ongoing".to_string(),
                };
            }
        }

        // Description
        let mut desc = String::new();
        if let Ok(d_sel) = Selector::parse("p[itemprop='abstract']") {
            desc = doc
                .select(&d_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
        }

        // Chapters
        let mut chapters = 0i32;
        if let Ok(c_sel) = Selector::parse("div.card.chapter h2 a") {
            chapters = doc.select(&c_sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Story id from URL.
        let story_id = url
            .split("/fanfictions/")
            .nth(1)
            .and_then(|s| s.split('/').nth(1))
            .unwrap_or("")
            .to_string();

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ffr_{story_id}"),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
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
            let sel = Selector::parse("div.card.chapter h2 a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.fanfictions.fr{href}")
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
    if let Ok(sel) = Selector::parse("div#readarea") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
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
        let s = FanfictionsFrScraper;
        assert!(s.can_handle("https://www.fanfictions.fr/fanfictions/hp/12345/chapters.html"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        let url = "https://www.fanfictions.fr/fanfictions/hp/12345/chapters.html";
        let id = url.split("/fanfictions/").nth(1).and_then(|s| s.split('/').nth(1)).unwrap();
        assert_eq!(id, "12345");
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("2024-01-31 10:00:00") > 0);
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="readarea"><p>French story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div#readarea") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("French story text."));
    }
}
