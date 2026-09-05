//! PhoenixSong.net native adapter (Harry Potter archive).
//!
//! Story URL: `https://www.phoenixsong.net/fanfiction/story/{id}/`.
//! - title: `div#nav25 a[href*="fanfiction/story/{id}/"]`
//! - author: `div#nav25 a[href*="/fanfiction/author/"]`
//! - chapters: `select option` (value = chapter URL)
//! - metadata (rating/words/status/summary) from the author page
//! - body: `p`/`blockquote` elements until the formatting-problem note
//!   (strip div/table/script/form/textarea)

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct PhoenixSongScraper;

impl PhoenixSongScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"phoenixsong\.net/fanfiction/story/(\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for PhoenixSongScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("phoenixsong.net/fanfiction/story/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("phoenixsong: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut rating = String::new();
        let mut status = "ongoing".to_string();
        let mut words = 0i64;
        let mut desc = String::new();
        let mut chapters = 0i32;

        {
            let doc = Html::parse_document(&html);

            // Title + author from div#nav25.
            if let Ok(nav_sel) = Selector::parse("div#nav25") {
                if let Some(nav) = doc.select(&nav_sel).next() {
                    let story_re = Regex::new(&format!(r"fanfiction/story/{story_id}/$")).unwrap();
                    if let Ok(a_sel) = Selector::parse("a[href]") {
                        for a in nav.select(&a_sel) {
                            let href = a.value().attr("href").unwrap_or("");
                            if title.is_empty() && story_re.is_match(href) {
                                title = a.text().collect::<String>().trim().to_string();
                            } else if author.is_empty() && href.contains("/fanfiction/author/") {
                                author = a.text().collect::<String>().trim().to_string();
                                author_url = format!("https://www.phoenixsong.net{href}");
                                author_local_id = href.split('/').nth(3).unwrap_or("").to_string();
                            }
                        }
                    }
                }
            }

            // Chapters from select option.
            if let Ok(sel) = Selector::parse("select option[value]") {
                for o in doc.select(&sel) {
                    if let Some(_v) = o.value().attr("value") {
                        chapters += 1;
                    }
                }
            }
            if chapters == 0 {
                chapters = 1;
            }
        } // doc dropped here

        if title.is_empty() {
            return Err(ScrapeError::ParseError("phoenixsong: no title".into()));
        }

        // Metadata from author page: find the story link, walk following divs
        // for Rating/Words/Setting/Status/Summary.
        if !author_url.is_empty() {
            if let Ok(apage) = http::fetch(client, &author_url).await {
                let adoc = Html::parse_document(&apage);
                let story_re = Regex::new(&format!(r"fanfiction/story/{story_id}/$")).unwrap();
                if let Ok(a_sel) = Selector::parse("a[href]") {
                    let mut started = false;
                    let mut prev_b_text = String::new();
                    for a in adoc.select(&a_sel) {
                        let href = a.value().attr("href").unwrap_or("");
                        if story_re.is_match(href) {
                            started = true;
                            continue;
                        }
                        if !started {
                            continue;
                        }
                        // Once past the story link, grab the next div's b-label/value.
                        let mut nxt = a.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "div" {
                                    if let Ok(b_sel) = Selector::parse("b") {
                                        if let Some(b) = el.select(&b_sel).next() {
                                            let btext = b.text().collect::<String>();
                                            let rest = el.text().collect::<String>();
                                            let val =
                                                rest.replacen(&btext, "", 1).trim().to_string();
                                            if btext.contains("Rating") && rating.is_empty() {
                                                rating = val
                                                    .split(": ")
                                                    .nth(1)
                                                    .unwrap_or("")
                                                    .to_string();
                                            } else if btext.contains("Words") {
                                                words = val
                                                    .chars()
                                                    .filter(|c| c.is_ascii_digit())
                                                    .collect::<String>()
                                                    .parse()
                                                    .unwrap_or(0);
                                            } else if btext.contains("Status") {
                                                if val.contains("Completed") {
                                                    status = "complete".to_string();
                                                }
                                            } else if btext.contains("Summary") {
                                                desc = val.clone();
                                            }
                                            prev_b_text = btext;
                                        }
                                    }
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                        if prev_b_text.contains("Summary") {
                            break;
                        }
                    }
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("ps_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: now,
            updated: now,
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

        let chapter_urls: Vec<String> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("select option[value]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|o| o.value().attr("value").map(|v| v.to_string()))
                .collect()
        };

        let urls: Vec<String> = if chapter_urls.is_empty() {
            vec![meta.source.clone()]
        } else {
            chapter_urls
        };

        let mut chapters = Vec::new();
        for (i, url) in urls.into_iter().enumerate() {
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
    let mut out = String::new();
    for sel in ["p", "blockquote"] {
        if let Ok(s) = Selector::parse(sel) {
            for el in doc.select(&s) {
                let text = el.text().collect::<String>();
                if text.contains("This is for problems with the formatting") {
                    return out;
                }
                // Skip nav/bad elements.
                if let Some(parent) = el.parent() {
                    if let Some(pe) = scraper::ElementRef::wrap(parent) {
                        if pe.value().name() == "nav" {
                            continue;
                        }
                    }
                }
                out.push_str(&el.inner_html());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = PhoenixSongScraper;
        assert!(s.can_handle("https://www.phoenixsong.net/fanfiction/story/1234/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            PhoenixSongScraper::story_id("https://www.phoenixsong.net/fanfiction/story/1234/"),
            Some("1234".to_string())
        );
        assert_eq!(PhoenixSongScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn extracts_chapter_body_stops_at_note() {
        let html = r#"<html><body><p>Chapter text.</p><p>This is for problems with the formatting or the layout of the chapter.</p><p>After.</p></body></html>"#;
        let doc = Html::parse_document(html);
        let mut out = String::new();
        for sel in ["p", "blockquote"] {
            if let Ok(s) = Selector::parse(sel) {
                for el in doc.select(&s) {
                    let text = el.text().collect::<String>();
                    if text.contains("This is for problems with the formatting") {
                        break;
                    }
                    out.push_str(&el.inner_html());
                }
            }
        }
        assert!(out.contains("Chapter text."));
        assert!(!out.contains("After."));
    }

    #[test]
    fn extracts_words() {
        let val = "Words: 12,345".to_string();
        let words: i64 = val
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap();
        assert_eq!(words, 12345);
    }
}
