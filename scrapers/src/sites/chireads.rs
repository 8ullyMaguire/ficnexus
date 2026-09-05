//! ChiReads.com native adapter (French translation site).
//!
//! Story URL: `https://chireads.com/category/translatedtales/{id}`.
//! - title: `.inform-inform-data h3` (split " | " — strip fandom suffix)
//! - author: `.inform-inform-data h6` (strip "Auteur : " prefix)
//! - updated: newest chapter link href tail ("/YYYY/MM/DD/")
//! - description: `.inform-inform-data .inform-inform-txt span`
//! - chapters: `div#content a`
//! - body: `#content`

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct ChiReadsScraper;

#[async_trait]
impl SiteScraper for ChiReadsScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("chireads.com/category/translatedtales/")
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
        let mut desc = String::new();
        let mut updated = 0i64;
        let mut chapters = 0i32;

        // Info block.
        if let Ok(info_sel) = Selector::parse(".inform-inform-data") {
            if let Some(info) = doc.select(&info_sel).next() {
                // Title: h3, split " | " (fandom suffix).
                if let Ok(h3_sel) = Selector::parse("h3") {
                    title = info
                        .select(&h3_sel)
                        .next()
                        .map(|el| {
                            el.text()
                                .collect::<String>()
                                .trim()
                                .split(" | ")
                                .next()
                                .unwrap_or("")
                                .to_string()
                        })
                        .unwrap_or_default();
                }
                // Author: h6, strip "Auteur : ".
                if let Ok(h6_sel) = Selector::parse("h6") {
                    author = info
                        .select(&h6_sel)
                        .next()
                        .map(|el| {
                            el.text()
                                .collect::<String>()
                                .trim()
                                .replace("Auteur : ", "")
                                .split("Babelcheck")
                                .next()
                                .unwrap_or("")
                                .trim()
                                .to_string()
                        })
                        .unwrap_or_default();
                }
                // Description.
                if let Ok(txt_sel) = Selector::parse(".inform-inform-txt span") {
                    desc = info
                        .select(&txt_sel)
                        .next()
                        .map(|el| el.inner_html())
                        .unwrap_or_default();
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("chireads: no title".into()));
        }

        // Updated date from newest chapter link href (".../YYYY/MM/DD/").
        if let Ok(nc_sel) = Selector::parse(".newestchapitre > div > a") {
            if let Some(a) = doc.select(&nc_sel).next() {
                if let Some(h) = a.value().attr("href") {
                    if let Some(m) = Regex::new(r"/(\d{4})/(\d{2})/(\d{2})/")
                        .unwrap()
                        .captures(h)
                    {
                        let y: i32 = m.get(1).unwrap().as_str().parse().unwrap_or(0);
                        let mo: u32 = m.get(2).unwrap().as_str().parse().unwrap_or(0);
                        let d: u32 = m.get(3).unwrap().as_str().parse().unwrap_or(0);
                        if let Some(dt) = chrono::NaiveDate::from_ymd_opt(y, mo, d)
                            .and_then(|x| x.and_hms_opt(0, 0, 0))
                        {
                            updated = chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
                        }
                    }
                }
            }
        }

        // Chapters.
        if let Ok(c_sel) = Selector::parse("div#content a") {
            chapters = doc.select(&c_sel).count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        let story_id = url.rsplit('/').next().unwrap_or("").to_string();
        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("chi_{story_id}"),
            title,
            author: author.clone(),
            chapters,
            words: 0,
            desc,
            published: now,
            updated: if updated > 0 { updated } else { now },
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url: String::new(),
            author_local_id: author.clone(),
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
            let sel = Selector::parse("div#content a")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://chireads.com{href}")
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
    if let Ok(sel) = Selector::parse("#content") {
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
        let s = ChiReadsScraper;
        assert!(s.can_handle("https://chireads.com/category/translatedtales/123"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn strips_title_fandom() {
        let raw = "My Story | Harry Potter".to_string();
        let title = raw.split(" | ").next().unwrap_or("").to_string();
        assert_eq!(title, "My Story");
    }

    #[test]
    fn strips_author_prefix() {
        let raw = "Auteur : Someone".to_string();
        let author = raw
            .replace("Auteur : ", "")
            .split("Babelcheck")
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        assert_eq!(author, "Someone");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div id="content"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("#content") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
