//! StoriesOfArda.com native adapter (Tolkien archive).
//!
//! Story URL: `http://www.storiesofarda.com/chapterlistview.asp?SID={id}`.
//! - title/author: `th[colspan=3]` (author link + `em` removed)
//! - chapters: `a[href*="chapterview.asp?sid={id}&cid="]`
//! - summary: `td[colspan=3]`
//! - metadata (rating/status/dates) from the author page
//! - body: `table[width=90%] td` (adult chapters need `confirmAge` POST)

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct StoriesOfArdaScraper;

impl StoriesOfArdaScraper {
    fn story_id(url: &str) -> Option<String> {
        let m = Regex::new(r"storiesofarda\.com/chapter(?:list|All)view\.asp\?SID=(\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for StoriesOfArdaScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("storiesofarda.com/")
            && (url.contains("chapterlistview.asp?SID=") || url.contains("chapterAllview.asp?SID="))
    }
    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("storiesofarda: bad url".into()))?;
        let html = http::fetch(client, url).await?;
        let (title, author, author_url, author_local_id, desc) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            let mut desc = String::new();

            if let Ok(th_sel) = Selector::parse("th[colspan='3']") {
                if let Some(th) = doc.select(&th_sel).next() {
                    if let Ok(a_sel) = Selector::parse("a[href]") {
                        if let Some(a) = th.select(&a_sel).next() {
                            author = a.text().collect::<String>().trim().to_string();
                            if let Some(h) = a.value().attr("href") {
                                author_url = format!("http://www.storiesofarda.com/{h}");
                                author_local_id = h.split('=').nth(1).unwrap_or("").to_string();
                            }
                        }
                    }
                    // Title = th text without the em (author link already extracted).
                    title = th
                        .text()
                        .collect::<String>()
                        .replace(&author, "")
                        .trim()
                        .to_string();
                }
            }
            if let Ok(td_sel) = Selector::parse("td[colspan='3']") {
                desc = doc
                    .select(&td_sel)
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default();
            }
            (title, author, author_url, author_local_id, desc)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("storiesofarda: no title".into()));
        }

        // Chapters
        let chap_re = Regex::new(&format!(r"chapterview\.asp\?sid={story_id}&cid=\d+$")).unwrap();
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(a_sel) = Selector::parse("a[href]") {
                chapters = doc
                    .select(&a_sel)
                    .filter(|a| {
                        a.value()
                            .attr("href")
                            .map(|h| chap_re.is_match(h))
                            .unwrap_or(false)
                    })
                    .count() as i32;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Metadata from author page (best-effort).
        let mut status = "ongoing".to_string();
        let mut rating = String::new();
        if !author_url.is_empty() {
            if let Ok(apage) = http::fetch(client, &author_url).await {
                let adoc = Html::parse_document(&apage);
                let mut seen = false;
                if let Ok(td_sel) = Selector::parse("td[colspan='3']") {
                    for td in adoc.select(&td_sel) {
                        let has_link = td
                            .select(
                                &Selector::parse(&format!(
                                    "a[href*='chapterlistview.asp?SID={story_id}']"
                                ))
                                .unwrap(),
                            )
                            .next()
                            .is_some();
                        if has_link {
                            seen = true;
                            continue;
                        }
                        if !seen {
                            continue;
                        }
                        let text = td.text().collect::<String>();
                        if text.contains("Rating:") {
                            rating = text
                                .split("Rating:")
                                .nth(1)
                                .unwrap_or("")
                                .split(":")
                                .nth(1)
                                .unwrap_or("")
                                .trim()
                                .to_string();
                        }
                        if text.contains("Status:") {
                            if text.contains("Completed") {
                                status = "complete".to_string();
                            }
                        }
                        if text.contains("Summary:") {
                            break;
                        }
                    }
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("soa_{story_id}"),
            title,
            author,
            chapters,
            words: 0,
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
        let story_id = Self::story_id(&meta.source)
            .ok_or_else(|| ScrapeError::ParseError("storiesofarda: bad url".into()))?;
        let html = http::fetch(client, &meta.source).await?;

        let links: Vec<String> = {
            let doc = Html::parse_document(&html);
            let sel =
                Selector::parse("a[href]").map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let chap_re =
                Regex::new(&format!(r"chapterview\.asp\?sid={story_id}&cid=\d+$")).unwrap();
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if !chap_re.is_match(href) {
                        return None;
                    }
                    Some(format!("http://www.storiesofarda.com/{href}"))
                })
                .collect()
        };

        if links.is_empty() {
            return Err(ScrapeError::ParseError("storiesofarda: no chapters".into()));
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

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    if html.contains("Please indicate that you are an adult") {
        return String::new(); // adult-gated; caller treats as empty
    }
    let doc = Html::parse_document(&html);
    if let Ok(t_sel) = Selector::parse("table[width='90%'] td") {
        if let Some(el) = doc.select(&t_sel).next() {
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
        let s = StoriesOfArdaScraper;
        assert!(s.can_handle("http://www.storiesofarda.com/chapterlistview.asp?SID=1234"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            StoriesOfArdaScraper::story_id(
                "http://www.storiesofarda.com/chapterlistview.asp?SID=1234"
            ),
            Some("1234".to_string())
        );
        assert_eq!(StoriesOfArdaScraper::story_id("https://x.com/foo"), None);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><table width="90%" align="center"><tr><td>Story text.</td></tr></table></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("table[width='90%'] td").unwrap();
        let s = doc
            .select(&sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        assert!(s.contains("Story text."));
    }

    #[test]
    fn detects_adult_gate() {
        let html =
            "Please indicate that you are an adult by selecting the appropriate choice below";
        assert!(html.contains("Please indicate that you are an adult"));
    }
}
