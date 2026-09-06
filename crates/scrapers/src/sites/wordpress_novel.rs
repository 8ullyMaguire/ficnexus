//! Generic WordPress-based novel-site adapter.
//!
//! Covers the family of WordPress-driven webnovel sites whose layout
//! follows NovelFull's pattern (upstream requests: novelfull.net #1316,
//! novelswd.com #795, novelhall.com #762, hostednovel.com #523,
//! novelhall.com, etc.):
//!
//! - story URL: `https://{host}/{story-slug}.html`
//! - title: `h3.title`
//! - author: `a[href*='/author/']`
//! - status: `a[href*='status']` text ("Completed" / else in-progress)
//! - description: `div.desc-text`
//! - chapters: `div.list-chapter a[href*='.html']` (a chapter list page)
//!
//! The adapter is instantiated with a host list so one implementation
//! covers many sites; each URL is matched against the configured hosts.

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

use async_trait::async_trait;

/// WordPress-novel adapter for the configured hosts.
pub struct WordPressNovelScraper {
    /// Hosts this adapter handles, e.g. `["novelfull.com", "novelswd.com"]`.
    pub hosts: &'static [&'static str],
}

impl WordPressNovelScraper {
    /// Extract the story slug from a URL like
    /// `https://novelfull.com/some-story-title.html`.
    fn story_slug(url: &str) -> Option<String> {
        let lower = url.to_lowercase();
        // strip scheme + host, keep path
        let path = lower
            .split("://")
            .nth(1)
            .map(|r| r.split('/').nth(1))
            .flatten()
            .unwrap_or("");
        // strip .html suffix + any trailing path
        let slug = path.split(".html").next().unwrap_or(path);
        if slug.is_empty() {
            None
        } else {
            Some(slug.to_string())
        }
    }
}

#[async_trait]
impl SiteScraper for WordPressNovelScraper {
    fn can_handle(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        self.hosts.iter().any(|h| lower.contains(h)) && lower.contains(".html")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let slug = WordPressNovelScraper::story_slug(url)
            .ok_or_else(|| ScrapeError::ParseError("no story slug in URL".into()))?;

        let resp = client
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0",
            )
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if resp.status().is_client_error() || resp.status().is_server_error() {
            return Err(ScrapeError::NotFound);
        }

        let html = resp
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = scraper::Html::parse_document(&html);

        // Title: h3.title
        let title = document
            .select(
                &scraper::Selector::parse("h3.title")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?,
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Author: a[href*='/author/']
        let author_el = document
            .select(
                &scraper::Selector::parse("a[href*='/author/']")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?,
            )
            .next();
        let (author, author_url) = match author_el {
            Some(a) => {
                let name = a.text().collect::<String>().trim().to_string();
                let href = a.value().attr("href").unwrap_or("").to_string();
                (name, href)
            }
            None => (String::new(), String::new()),
        };

        // Status: a[href*='status'] text
        let status = document
            .select(
                &scraper::Selector::parse("a[href*='status']")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?,
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .map(|s| {
                if s.to_lowercase().contains("completed") {
                    "complete".to_string()
                } else {
                    "ongoing".to_string()
                }
            })
            .unwrap_or_else(|| "ongoing".to_string());

        // Description: div.desc-text
        let desc = document
            .select(
                &scraper::Selector::parse("div.desc-text")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?,
            )
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        if title.is_empty() && author.is_empty() {
            return Err(ScrapeError::ParseError(
                "wordpress novel page missing title+author".into(),
            ));
        }

        Ok(FicMetadata {
            url_id: crate::generate_url_id(8, &slug),
            title,
            author,
            chapters: 0,
            words: 0,
            desc,
            published: 0,
            updated: 0,
            status,
            source: url.to_string(),
            source_id: 8,
            author_id: 0,
            author_url,
            author_local_id: slug,
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
        // NovelFull-style sites put the chapter list on the story page
        // (possibly a "full" list). Look for div.list-chapter links.
        let resp = client
            .get(&meta.source)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0",
            )
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let html = resp
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        // Collect chapter (url, title) pairs in a scoped block.
        let links: Vec<(String, String)> = {
            let document = scraper::Html::parse_document(&html);
            let sel = scraper::Selector::parse("div.list-chapter a[href*='.html']")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let mut links = Vec::new();
            for link in document.select(&sel) {
                let href = link.value().attr("href").unwrap_or("");
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("https://{}{}", host_of(&meta.source), href)
                };
                let title = link.text().collect::<String>().trim().to_string();
                links.push((url, title));
            }
            links
        };

        let mut chapters = Vec::new();
        let mut i = 1;
        let content_sel = scraper::Selector::parse(
            "div.chapter-c, div.content, #chapter-content, .chapter-content",
        )
        .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
        for (url, title) in links {
            let chap_resp = client
                .get(&url)
                .header(
                    "User-Agent",
                    "Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0",
                )
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let chap_html = chap_resp
                .text()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let content = {
                let chap_doc = scraper::Html::parse_document(&chap_html);
                chap_doc
                    .select(&content_sel)
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default()
            };
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: i,
                    title,
                    content,
                });
                i += 1;
            }
        }
        Ok(chapters)
    }
}

/// Extract the host (e.g. `novelfull.com`) from a URL.
fn host_of(url: &str) -> String {
    url.split("://")
        .nth(1)
        .and_then(|r| r.split('/').next())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_slug_extraction() {
        assert_eq!(
            WordPressNovelScraper::story_slug("https://novelfull.com/some-story-title.html")
                .as_deref(),
            Some("some-story-title")
        );
        assert_eq!(
            WordPressNovelScraper::story_slug("https://novelfull.com/some-story-title.html?page=2")
                .as_deref(),
            Some("some-story-title")
        );
        assert_eq!(
            WordPressNovelScraper::story_slug("https://novelfull.com/"),
            None
        );
    }

    #[test]
    fn can_handle_hosts() {
        let s = WordPressNovelScraper {
            hosts: &["novelfull.com", "novelhall.com"],
        };
        assert!(s.can_handle("https://novelfull.com/story.html"));
        assert!(s.can_handle("https://www.novelhall.com/another.html"));
        assert!(!s.can_handle("https://archiveofourown.org/works/1"));
        assert!(!s.can_handle("https://novelfull.com/"));
    }

    #[test]
    fn host_of_parses() {
        assert_eq!(host_of("https://novelfull.com/a.html"), "novelfull.com");
    }
}
