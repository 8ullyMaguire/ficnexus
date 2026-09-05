//! StoriesOnline.net family native adapter.
//!
//! Covers storiesonline.net plus its subclasses scifistories.com and
//! storyroom.com (FFF's StoriesOnlineNetAdapter + SciFiStories/StoryRoom
//! subclasses). The three share one layout:
//!
//! Story URL: `https://{domain}/s/{id}/{title}` (or `/n/{id}`).
//! - title: `h1`
//! - authors: `footer a[rel=author]` (href `/u/{id}/{name}`)
//! - chapters: `div#index-list a[href*="/s/"], a[href*="/n/"]`
//! - metadata (desc/words/rating/status): from the author's story list row
//!   (classic `tr` theme or modern `div.sdesc`/`div.misc` theme)
//! - body: `article` (chapter page)

use async_trait::async_trait;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

/// Per-domain config for the StoriesOnline family.
#[derive(Debug, Clone, Copy)]
pub struct StoriesOnlineSite {
    pub domain: &'static str,
    pub abbrev: &'static str,
}

pub const STORIESONLINE_SITES: &[StoriesOnlineSite] = &[
    StoriesOnlineSite {
        domain: "storiesonline.net",
        abbrev: "son",
    },
    StoriesOnlineSite {
        domain: "scifistories.com",
        abbrev: "sfst",
    },
    StoriesOnlineSite {
        domain: "storyroom.com",
        abbrev: "stryrm",
    },
];

/// Domain-driven scraper for the family.
pub struct StoriesOnlineScraper {
    site: &'static StoriesOnlineSite,
}

impl StoriesOnlineScraper {
    pub fn from_url(url: &str) -> Option<Self> {
        let site = STORIESONLINE_SITES
            .iter()
            .find(|s| url.contains(s.domain))?;
        Some(StoriesOnlineScraper { site })
    }

    pub fn all() -> Vec<StoriesOnlineScraper> {
        STORIESONLINE_SITES
            .iter()
            .map(|s| StoriesOnlineScraper { site: s })
            .collect()
    }
}

#[async_trait]
impl SiteScraper for StoriesOnlineScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains(self.site.domain) && (url.contains("/s/") || url.contains("/n/"))
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        // Force the index view (?ind=1) so chapter list is present.
        let page_url = if url.contains("?ind=1") {
            url.to_string()
        } else {
            format!("{url}?ind=1")
        };
        let html = http::fetch(client, &page_url).await?;

        if html.contains("being filtered by your choice of contents filtering") {
            return Err(ScrapeError::AuthRequired("content filter".into()));
        }

        // Parse metadata into owned values, dropping the non-Send doc before
        // the author-page fetch.
        let (story_id, title, author, author_url, author_local_id, chapters) = {
            let doc = Html::parse_document(&html);

            // Story id from URL /s/{id}/{title}.
            let story_id = url
                .split('/')
                .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
                .next()
                .unwrap_or("")
                .to_string();

            // Title
            let mut title = String::new();
            if let Ok(h1_sel) = Selector::parse("h1") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }

            // Author
            let mut author = String::new();
            let mut author_url = String::new();
            let mut author_local_id = String::new();
            if let Ok(a_sel) = Selector::parse("footer a[rel='author']") {
                if let Some(a) = doc.select(&a_sel).next() {
                    author = a.text().collect::<String>().trim().to_string();
                    if let Some(h) = a.value().attr("href") {
                        author_url = format!("https://{}{}", self.site.domain, h);
                        if let Some(id) = h.split('/').nth(2) {
                            author_local_id = id.to_string();
                        }
                    }
                }
            }

            // Chapters
            let mut chapters = 0i32;
            if let Ok(sel) =
                Selector::parse("div#index-list a[href*='/s/'], div#index-list a[href*='/n/']")
            {
                chapters = doc.select(&sel).count() as i32;
            }
            if chapters == 0 {
                chapters = 1;
            }

            (
                story_id,
                title,
                author,
                author_url,
                author_local_id,
                chapters,
            )
        }; // doc dropped here

        if title.is_empty() {
            return Err(ScrapeError::ParseError("storiesonline: no title".into()));
        }

        // Description/words/status from the author's story list row.
        let mut desc = String::new();
        let mut words = 0i64;
        let mut status = "ongoing".to_string();
        if !author_url.is_empty() {
            if let Ok(auth_html) = http::fetch(client, &author_url).await {
                let adoc = Html::parse_document(&auth_html);
                // Modern theme: div.sdesc + div.misc; classic: td.lc4.
                if let Ok(sd_sel) = Selector::parse("div.sdesc") {
                    if let Some(el) = adoc.select(&sd_sel).next() {
                        desc = el.inner_html();
                    }
                }
                if desc.is_empty() {
                    if let Ok(td_sel) = Selector::parse("td.lc4") {
                        if let Some(el) = adoc.select(&td_sel).next() {
                            desc = el.inner_html();
                        }
                    }
                }
                if let Ok(misc_sel) = Selector::parse("div.misc") {
                    for el in adoc.select(&misc_sel) {
                        let t = el.text().collect::<String>();
                        if t.contains("words") {
                            words = t
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .collect::<String>()
                                .parse()
                                .unwrap_or(0);
                        }
                        if t.contains("Completed") {
                            status = "complete".to_string();
                        }
                    }
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("{}_{}", self.site.abbrev, story_id),
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

        // Collect chapter links into owned data first.
        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel =
                Selector::parse("div#index-list a[href*='/s/'], div#index-list a[href*='/n/']")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://{}{}", self.site.domain, href)
                    };
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            // One-chapter story: the story page itself.
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

/// Fetch a StoriesOnline chapter and extract the `article` (strip header).
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("article") {
        if let Some(el) = doc.select(&sel).next() {
            let mut out = String::new();
            for child in el.children() {
                if let Some(cref) = scraper::ElementRef::wrap(child) {
                    if cref.value().name() != "header" && cref.value().name() != "h2" {
                        out.push_str(&cref.inner_html());
                    }
                }
            }
            return out;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_has_three_sites() {
        assert_eq!(STORIESONLINE_SITES.len(), 3);
    }

    #[test]
    fn from_url_matches_domains() {
        assert!(StoriesOnlineScraper::from_url("https://storiesonline.net/s/1234/title").is_some());
        assert!(StoriesOnlineScraper::from_url("https://scifistories.com/s/1234").is_some());
        assert!(StoriesOnlineScraper::from_url("https://storyroom.com/n/1234").is_some());
        assert!(StoriesOnlineScraper::from_url("https://www.fanfiction.net/s/123").is_none());
    }

    #[test]
    fn parses_story_id() {
        let url = "https://storiesonline.net/s/12345/story-title";
        let id = url
            .split('/')
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .next()
            .unwrap();
        assert_eq!(id, "12345");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><article><header>meta</header><p>Story text.</p></article></body></html>"#;
        let doc = Html::parse_document(html);
        let mut out = String::new();
        if let Ok(sel) = Selector::parse("article") {
            if let Some(el) = doc.select(&sel).next() {
                for child in el.children() {
                    if let Some(cref) = scraper::ElementRef::wrap(child) {
                        if cref.value().name() != "header" && cref.value().name() != "h2" {
                            out.push_str(&cref.inner_html());
                        }
                    }
                }
            }
        }
        assert!(out.contains("Story text."));
        assert!(!out.contains("meta"));
    }
}
