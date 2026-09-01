/// XenForo forum scraper (SpaceBattles, SufficientVelocity, QuestionableQuesting)
pub struct XenForoScraper;

use async_trait::async_trait;
use crate::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

// Domains that use XenForo
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
    "boards.theforce.net",
    // Fiction.live is XenForo-based; the native adapter also skips empty
    // chapters (upstream FFF #1288: empty "Home" chapter breaks updates).
    "fiction.live",
    // XenForo2 family (FFF base_xenforo2forum_adapter).
    "www.alternatehistory.com",
    "althistory.com",
    "www.the-sietch.com",
];

impl XenForoScraper {
    fn extract_thread_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    fn is_xenforo_url(url: &str) -> bool {
        XENFORO_DOMAINS.iter().any(|d| url.contains(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_theforce_net() {
        let s = XenForoScraper;
        let url = "https://boards.theforce.net/threads/kyp-durron-skiing-triathlon.50062326/";
        assert!(s.can_handle(url));
        assert_eq!(XenForoScraper::extract_thread_id(url).as_deref(), Some("50062326"));
    }

    #[test]
    fn can_handle_existing_xenforo_sites() {
        let s = XenForoScraper;
        for url in [
            "https://forums.spacebattles.com/threads/foo.123456/",
            "https://forums.sufficientvelocity.com/threads/bar.789/",
            "https://forum.questionablequesting.com/threads/baz.1/",
        ] {
            assert!(s.can_handle(url), "should handle {url}");
        }
        assert!(!s.can_handle("https://archiveofourown.org/works/123"));
    }
}

#[async_trait]
impl SiteScraper for XenForoScraper {
    fn can_handle(&self, url: &str) -> bool {
        Self::is_xenforo_url(url)
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let thread_id = Self::extract_thread_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract thread ID".into()))?;

        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1.p-title-value").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author_el = document
            .select(&Selector::parse("a.username").unwrap())
            .next();
        let author = author_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let author_url = author_el
            .and_then(|el| el.value().attr("href"))
            .map(|h| {
                if h.starts_with('/') {
                    // Determine domain from the URL
                    for domain in XENFORO_DOMAINS {
                        if url.contains(domain) {
                            return format!("https://{domain}{h}");
                        }
                    }
                }
                h.to_string()
            })
            .unwrap_or_default();

        let description = document
            .select(&Selector::parse("article.message-body").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        let url_id = crate::generate_url_id(3, &thread_id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words: 0,
            desc: description,
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 3,
            author_id: 0,
            author_url,
            author_local_id: thread_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();

        // For XenForo, we fetch the thread page by page
        // The first post is the story content
        let url = &meta.source;
        let response = client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let article_sel = Selector::parse("article.message-body").unwrap();
        let mut chapter_idx = 0;

        for article in document.select(&article_sel) {
            chapter_idx += 1;
            let content = article.inner_html();
            // Skip empty posts — Fiction.live has an empty "Home" chapter
            // that would otherwise break update detection (upstream FFF
            // #1288). Trim tags/whitespace to test for real content.
            let text_only = article.text().collect::<String>();
            if text_only.trim().is_empty() {
                continue;
            }
            let title = if chapter_idx == 1 {
                meta.title.clone()
            } else {
                format!("Chapter {chapter_idx}: Thread Page {chapter_idx}")
            };

            chapters.push(Chapter {
                chapter_id: chapter_idx,
                title,
                content,
            });
        }

        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("no content found in XenForo thread".into()));
        }

        Ok(chapters)
    }
}
