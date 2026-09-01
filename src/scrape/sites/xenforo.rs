/// XenForo forum scraper (SpaceBattles, SufficientVelocity, QuestionableQuesting)
pub struct XenForoScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

// Domains that use XenForo
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
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
            .header("User-Agent", "fichub.net/0.1.0")
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

        let url_id = crate::scrape::generate_url_id(3, &thread_id);
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
            .header("User-Agent", "fichub.net/0.1.0")
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
