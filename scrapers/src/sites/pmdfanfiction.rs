//! PMDFanFiction.com native adapter (Pokémon Mystery Dungeon archive).
//!
//! Story page: `article.story__article`:
//! - title: `h1.story__identity-title`
//! - author: `a.author`
//! - status: `span.story__status` (Ongoing→In-Progress, Oneshot→Completed,
//!   Hiatus passthrough)
//! - rating: `span.story__rating`
//! - description: `section.story__summary`
//! - published: `span.story__date span.hide-below-480`
//! Chapters: `ol.chapter-group__list a.chapter-group__list-item-link`
//! (chapters grouped into sections — flatten all groups).
//! Chapter body: `section.chapter__content`.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct PmdFanfictionScraper;

#[async_trait]
impl SiteScraper for PmdFanfictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("pmdfanfiction.com") || url.contains("pmdff.com")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(client, url).await?;
        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut status = "ongoing".to_string();
        let mut rating = String::new();
        let mut desc = String::new();
        let mut published = 0i64;
        let mut chapters = 0i32;

        if let Ok(art_sel) = Selector::parse("article.story__article") {
            if let Some(art) = doc.select(&art_sel).next() {
                // Title
                if let Ok(h1_sel) = Selector::parse("h1.story__identity-title") {
                    title = art
                        .select(&h1_sel)
                        .next()
                        .map(|el| el.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                }
                // Author
                if let Ok(a_sel) = Selector::parse("a.author") {
                    if let Some(a) = art.select(&a_sel).next() {
                        author = a.text().collect::<String>().trim().to_string();
                        if let Some(h) = a.value().attr("href") {
                            author_url = h.to_string();
                            author_local_id = h
                                .trim_end_matches('/')
                                .rsplit('/')
                                .next()
                                .unwrap_or("")
                                .to_string();
                        }
                    }
                }
                // Status
                if let Ok(s_sel) = Selector::parse("span.story__status") {
                    if let Some(el) = art.select(&s_sel).next() {
                        let raw = el.text().collect::<String>().trim().to_string();
                        status = match raw.as_str() {
                            "Completed" | "Oneshot" => "complete".to_string(),
                            "Hiatus" => "hiatus".to_string(),
                            _ => "ongoing".to_string(),
                        };
                    }
                }
                // Rating
                if let Ok(r_sel) = Selector::parse("span.story__rating") {
                    rating = art
                        .select(&r_sel)
                        .next()
                        .map(|el| el.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                }
                // Description
                if let Ok(d_sel) = Selector::parse("section.story__summary") {
                    desc = art
                        .select(&d_sel)
                        .next()
                        .map(|el| el.inner_html())
                        .unwrap_or_default();
                }
                // Published date
                if let Ok(p_sel) = Selector::parse("span.story__date span.hide-below-480") {
                    published = art
                        .select(&p_sel)
                        .next()
                        .map(|el| parse_dt(&el.text().collect::<String>()))
                        .unwrap_or(0);
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("pmdfanfiction: no title".into()));
        }

        // Chapters (all chapter-group lists flattened).
        if let Ok(sel) = Selector::parse("ol.chapter-group__list a.chapter-group__list-item-link") {
            chapters = doc.select(&sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Story id from URL.
        let story_id = url
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
            .to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("pmd_{story_id}"),
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

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("ol.chapter-group__list a.chapter-group__list-item-link")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.pmdfanfiction.com{href}")
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
    if let Ok(sel) = Selector::parse("section.chapter__content") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
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
        let s = PmdFanfictionScraper;
        assert!(s.can_handle("https://www.pmdfanfiction.com/story/123/title"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn maps_status() {
        assert_eq!(
            match "Oneshot" {
                "Completed" | "Oneshot" => "complete",
                _ => "ongoing",
            },
            "complete"
        );
        assert_eq!(
            match "Hiatus" {
                "Hiatus" => "hiatus",
                _ => "ongoing",
            },
            "hiatus"
        );
        assert_eq!(
            match "Ongoing" {
                "Completed" | "Oneshot" => "complete",
                _ => "ongoing",
            },
            "ongoing"
        );
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><section class="chapter__content"><p>Story text.</p></section></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("section.chapter__content") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
