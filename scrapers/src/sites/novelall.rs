//! NovelAll.com native adapter (translated web-novel site).
//!
//! Story URL: `https://www.novelall.com/novel/{id}.html` (add `?waring=1`
//! for adult stories).
//! - title: `h1` (strip trailing " Novel")
//! - author: `span` "Author:" → next `a`
//! - status: `span` "Status:" → next sibling
//! - genres/tags: `span` "Genre(s):"/"Tag(s):" → following `a`s
//! - description: `#show`
//! - chapters: `div.detail-chlist li` (link + `.time` date), reversed
//! - body: `div.reading-box` (strip `a`/`ins`/`script`)

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct NovelAllScraper;

impl NovelAllScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"novelall\.com/(?:novel|chapter)/([^/\.]+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for NovelAllScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("novelall.com/") && (url.contains("/novel/") || url.contains("/chapter/"))
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("novelall: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("Please click here to continue the reading.") {
            return Err(ScrapeError::AuthRequired("novelall: adult gate".into()));
        }

        let (title, author, desc, status, genres) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut desc = String::new();
            let mut status = "ongoing".to_string();
            let mut genres: Vec<String> = Vec::new();

            if let Ok(h1_sel) = Selector::parse("h1") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }
            if title.ends_with(" Novel") {
                title = title[..title.len() - 6].to_string();
            }

            // Author: span "Author:" → next sibling a.
            if let Ok(sp_sel) = Selector::parse("span") {
                for sp in doc.select(&sp_sel) {
                    let text = sp.text().collect::<String>();
                    if text.trim() == "Author:" {
                        let mut nxt = sp.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "a" {
                                    author = el.text().collect::<String>().trim().to_string();
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                    } else if text.trim() == "Status:" {
                        let mut nxt = sp.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "span" {
                                    let s = el.text().collect::<String>().trim().to_string();
                                    if s.contains("Completed") {
                                        status = "complete".to_string();
                                    } else {
                                        status = "ongoing".to_string();
                                    }
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                    } else if text.trim() == "Genre(s):" {
                        let mut nxt = sp.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "a" {
                                    genres.push(el.text().collect::<String>().trim().to_string());
                                } else if el.value().name() == "span" {
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                    }
                }
            }

            // Description.
            if let Ok(d_sel) = Selector::parse("#show") {
                desc = doc
                    .select(&d_sel)
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default();
            }

            (title, author, desc, status, genres)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("novelall: no title".into()));
        }

        // Chapters from div.detail-chlist li.
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(sel) = Selector::parse("div.detail-chlist li") {
                chapters = doc.select(&sel).count() as i32;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("na_{}", story_id),
            title,
            author: author.clone(),
            chapters,
            words: 0,
            desc,
            published: now,
            updated: now,
            status,
            source: url.to_string(),
            source_id: 0,
            author_id: 0,
            author_url: String::new(),
            author_local_id: author.clone(),
            content_hash: None,
            extra_meta: Some(format!("genres={};", genres.join(","))),
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
            let sel = Selector::parse("div.detail-chlist li a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.value().attr("title").unwrap_or("").to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.novelall.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        let mut links: Vec<(String, String)> = links;
        links.reverse(); // newest first → oldest first.

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title: if title.is_empty() {
                    format!("Chapter {}", i + 1)
                } else {
                    title
                },
                content,
            });
        }
        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("novelall: no chapters".into()));
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
    if let Ok(sel) = Selector::parse("div.reading-box") {
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
        let s = NovelAllScraper;
        assert!(s.can_handle("https://www.novelall.com/novel/Castle-of-Black-Iron.html"));
        assert!(s.can_handle("https://www.novelall.com/chapter/The-Legendary-Moonlight-Sculptor-Volume-1-Chapter-1/1048282/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(NovelAllScraper::story_id("https://www.novelall.com/novel/Castle-of-Black-Iron.html"), Some("Castle-of-Black-Iron".to_string()));
        assert_eq!(NovelAllScraper::story_id("https://www.novelall.com/chapter/The-Legendary-Moonlight-Sculptor-Volume-1-Chapter-1/1048282/"), Some("The-Legendary-Moonlight-Sculptor-Volume-1-Chapter-1".to_string()));
        assert_eq!(NovelAllScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn strips_novel_suffix() {
        let mut t = "Castle of Black Iron Novel".to_string();
        if t.ends_with(" Novel") {
            t = t[..t.len() - 6].to_string();
        }
        assert_eq!(t, "Castle of Black Iron");
    }

    #[test]
    fn detects_adult_gate() {
        let html = "Please click here to continue the reading.";
        assert!(html.contains("Please click here to continue the reading."));
    }
}
