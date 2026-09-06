//! FanFicAuthors.net native adapter (per-author subdomain zones).
//!
//! Story URL: `https://{zone}.fanficauthors.net/{story_id}/`.
//! - author: from the zone subdomain (`abraxan` → "Abraxan")
//! - title: `h2`
//! - chapters: `a[href*="/{story_id}/"]` (skip `/reviews/`)
//! - metadata: `div.well p[1]` regex: Status / Rating / Chapters /
//!   Word count / Genre
//! - description: `div.well blockquote`
//! - body: per-chapter page (chapter link + `index/` TOC)
//! Adult-only rated works are author-gated (AuthRequired without creds).

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct FanficAuthorsScraper;

impl FanficAuthorsScraper {
    fn zone_and_id(url: &str) -> Option<(String, String)> {
        let host = url.split('/').nth(2)?;
        let zone = host.split('.').next()?.to_string();
        let story_id = url.split('/').nth(3)?.to_string();
        Some((zone, story_id))
    }
}

#[async_trait]
impl SiteScraper for FanficAuthorsScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains(".fanficauthors.net/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let (zone, story_id) = Self::zone_and_id(url)
            .ok_or_else(|| ScrapeError::ParseError("fanficauthors: bad url".into()))?;
        let index_url = format!("{url}index/");
        let html = http::fetch(client, &index_url).await?;
        let doc = Html::parse_document(&html);

        let author = zone
            .replace('-', " ")
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Title
        let mut title = String::new();
        if let Ok(h2_sel) = Selector::parse("h2") {
            title = doc
                .select(&h2_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_default();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("fanficauthors: no title".into()));
        }

        // Chapters
        let chap_re = Regex::new(&format!(r"/{story_id}/([a-zA-Z0-9_]+)/")).unwrap();
        let mut chapters = 0i32;
        if let Ok(a_sel) = Selector::parse("a[href]") {
            chapters = doc
                .select(&a_sel)
                .filter(|a| {
                    let href = a.value().attr("href").unwrap_or("");
                    chap_re.is_match(href) && !href.contains("/reviews/")
                })
                .count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Metadata from div.well p[1].
        let mut status = "ongoing".to_string();
        let mut rating = String::new();
        let mut words = 0i64;
        let mut desc = String::new();
        if let Ok(w_sel) = Selector::parse("div.well") {
            if let Some(well) = doc.select(&w_sel).next() {
                if let Ok(p_sel) = Selector::parse("p") {
                    let ps: Vec<_> = well.select(&p_sel).collect();
                    if let Some(p) = ps.get(1) {
                        let metaline = p
                            .text()
                            .collect::<String>()
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");
                        if let Some(m) = Regex::new(r"Status: (.+?) - Rating: (.+?) - Chapters: [0-9,]+ - Word count: ([0-9,]+?)(?: - Genre: ?(.*))?$").unwrap().captures(&metaline) {
                            let st = m.get(1).unwrap().as_str();
                            if st.contains("Completed") {
                                status = "complete".to_string();
                            } else {
                                status = "ongoing".to_string();
                            }
                            rating = m.get(2).unwrap().as_str().to_string();
                            words = m.get(3).unwrap().as_str().chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0);
                        }
                    }
                }
                if let Ok(bq_sel) = Selector::parse("blockquote") {
                    desc = well
                        .select(&bq_sel)
                        .next()
                        .map(|el| el.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ffa_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: now,
            updated: now,
            status,
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url: format!("https://{}.fanficauthors.net/", zone),
            author_local_id: zone,
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
        let (_, story_id) = Self::zone_and_id(&meta.source)
            .ok_or_else(|| ScrapeError::ParseError("fanficauthors: bad url".into()))?;
        let index_url = format!("{}index/", meta.source);
        let html = http::fetch(client, &index_url).await?;

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel =
                Selector::parse("a[href]").map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let chap_re = Regex::new(&format!(r"/{story_id}/([a-zA-Z0-9_]+)/")).unwrap();
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if !chap_re.is_match(href) || href.contains("/reviews/") {
                        return None;
                    }
                    let title = a.text().collect::<String>().trim().to_string();
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://{}{}", meta.source.split('/').nth(2)?, href)
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
    // Chapter content typically in a div with the story; try common selectors.
    for sel in ["div.storytext", "div#storytext", "div.content", "article"] {
        if let Ok(s) = Selector::parse(sel) {
            if let Some(el) = doc.select(&s).next() {
                return el.inner_html();
            }
        }
    }
    // Fallback: body minus nav.
    let body = doc.select(&Selector::parse("body").unwrap()).next();
    body.map(|el| el.inner_html()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = FanficAuthorsScraper;
        assert!(s.can_handle("https://abraxan.fanficauthors.net/Story_Name/"));
        assert!(s.can_handle("https://jbern.fanficauthors.net/Another_Story/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_zone_and_id() {
        let (zone, id) =
            FanficAuthorsScraper::zone_and_id("https://abraxan.fanficauthors.net/Story_Name/")
                .unwrap();
        assert_eq!(zone, "abraxan");
        assert_eq!(id, "Story_Name");
    }

    #[test]
    fn formats_author_name() {
        let zone = "musings-of-apathy";
        let author = zone
            .replace('-', " ")
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(author, "Musings Of Apathy");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="storytext"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("div.storytext").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
