/// AO3 site scraper
/// Handles URLs like: https://archiveofourown.org/works/123456 or /works/123456/chapters/789012
/// Also handles author pages: https://archiveofourown.org/users/{name}/works
///
/// The same layout is used by the other OTW/Archive software sites
/// (adastrafanfic.com, cfaa, squidgeworld.org, superlove), so the struct
/// carries a domain and can be constructed per-site.
pub struct Ao3Scraper {
    domain: String,
}

impl Ao3Scraper {
    /// The canonical AO3 site.
    pub fn ao3() -> Self {
        Self { domain: "archiveofourown.org".into() }
    }

    /// An OTW-family site with the same layout but a different domain.
    pub fn with_domain(domain: &str) -> Self {
        Self { domain: domain.to_string() }
    }

    fn base_url(&self) -> String {
        format!("https://{}", self.domain)
    }
}

use async_trait::async_trait;
use crate::{FicMetadata, Chapter, SiteScraper, ScrapeError, ExtractedTag};
use scraper::{Html, Selector};
use chrono::Utc;

const BASE_URL: &str = "https://archiveofourown.org";

impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        // Matches /works/NUMBER or /works/NUMBER/chapters/NUMBER
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    #[allow(dead_code)]
    fn extract_chapter_number(url: &str) -> Option<i32> {
        let re = regex_lite::Regex::new(r"/chapters/(\d+)").ok()?;
        re.captures(url)?.get(1).and_then(|m| m.as_str().parse().ok())
    }

    /// Check if a URL is an AO3 series page (archiveofourown.org/series/{id})
    pub fn is_series_page(url: &str) -> bool {
        let re = regex_lite::Regex::new(r"archiveofourown\.org/series/(\d+)").ok();
        re.map(|r| r.is_match(url)).unwrap_or(false)
    }

    /// Scrape an AO3 series page and return all work URLs.
    pub async fn list_series_works(client: &reqwest::Client, url: &str) -> Result<Vec<String>, ScrapeError> {
        let resp = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = resp
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let doc = Html::parse_document(&html);

        // AO3 series pages list works in <li class="work blurb group"> with <h4><a href="/works/{id}">
        let link_sel = Selector::parse("li.work h4.heading a[href^='/works/']").unwrap();
        let work_urls: Vec<String> = doc
            .select(&link_sel)
            .filter_map(|el| el.value().attr("href"))
            .map(|h| format!("{BASE_URL}{h}"))
            .collect();

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        Ok(work_urls.into_iter().filter(|u| seen.insert(u.clone())).collect())
    }

    /// Check if a URL is an AO3 author page (archiveofourown.org/users/{name} or /users/{name}/works)
    pub fn is_author_page(url: &str) -> bool {
        let re = regex_lite::Regex::new(r"archiveofourown\.org/users/([^/]+)").ok();
        re.map(|r| r.is_match(url)).unwrap_or(false)
    }

    /// Scrape an AO3 author page and return all work URLs.
    /// Paginates through all pages of /users/{author}/works.
    pub async fn list_author_works(client: &reqwest::Client, url: &str) -> Result<Vec<String>, ScrapeError> {
        let re = regex_lite::Regex::new(r"archiveofourown\.org/users/([^/]+)").ok();
        let author = re
            .and_then(|r| r.captures(url))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| ScrapeError::ParseError("cannot extract author name from URL".into()))?;

        let mut all_urls = Vec::new();
        let mut page = 1;

        loop {
            let page_url = format!("{BASE_URL}/users/{author}/works?page={page}&show_work=true");
            let resp = client
                .get(&page_url)
                .header("User-Agent", "fichub.net/0.1.0")
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;

            if !resp.status().is_success() {
                break;
            }

            let html = resp
                .text()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let doc = Html::parse_document(&html);

            // Find all fic links on the page — AO3 uses <li class="work blurb group">
            // with work blurb groups containing <a href="/works/{id}">
            let link_sel = Selector::parse("li.work a[href^='/works/']").unwrap();
            let page_urls: Vec<String> = doc
                .select(&link_sel)
                .filter_map(|el| el.value().attr("href"))
                .map(|h| format!("{BASE_URL}{h}"))
                .collect();

            if page_urls.is_empty() {
                break;
            }

            let before = all_urls.len();
            all_urls.extend(page_urls);
            // No new URLs found on this page
            if all_urls.len() == before {
                break;
            }

            // Check if there's a "next" page link
            let next_sel = Selector::parse("a[rel='next']").unwrap();
            if doc.select(&next_sel).next().is_none() {
                break;
            }

            page += 1;
        }

        // Deduplicate while preserving order
        let mut seen = std::collections::HashSet::new();
        all_urls.retain(|u| seen.insert(u.clone()));

        Ok(all_urls)
    }
}

#[async_trait]
impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains(&self.domain)
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

        let fic_url = format!("{}/works/{work_id}?view_full_work=true", self.base_url());
        let response = client
            .get(&fic_url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        self.lookup_from_html(&html, url).await
    }

    /// Parse metadata from a pre-fetched AO3 page (live, Wayback snapshot,
    /// cookie ingest, fixture).
    async fn lookup_from_html(&self, html: &str, url: &str) -> Result<FicMetadata, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;
        let document = Html::parse_document(html);

        // Parse metadata from AO3 page
        let title = document
            .select(&Selector::parse("h2.title.heading").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| {
                // Fallback: try <title> tag
                let fallback = document
                    .select(&Selector::parse("title").unwrap())
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default();
                // Extract title from "Title - Author - Fandom | Archive" format
                let t = fallback.split(" - ").next().unwrap_or("").trim().to_string();
                if t.is_empty() || t.len() < 2 {
                    tracing::warn!("AO3 title not found via h2.title.heading, HTML snippet: {:?}",
                        html.chars().take(2000).collect::<String>());
                    "Unknown Title".to_string()
                } else {
                    tracing::info!("AO3 title fallback from <title> tag: {}", t);
                    t
                }
            });

        // AO3 supports multiple creators (a[rel='author']). Collect ALL of
        // them (comma-joined) so multi-author fics keep every creator; the
        // first author is the primary (their profile link is used).
        let author_elements: Vec<String> = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let author = if author_elements.is_empty() {
            "Unknown Author".to_string()
        } else {
            author_elements.join(", ")
        };

        let author_url = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("{BASE_URL}{h}"))
            .unwrap_or_default();

        let description = document
            .select(&Selector::parse("blockquote.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        // Try to find stats (words, chapters, etc.)
        let stats_text: String = document
            .select(&Selector::parse("dd.chapters").unwrap())
            .next()
            .map(|el| el.text().collect())
            .unwrap_or_default();

        let chapters = if let Some(pos) = stats_text.find('/') {
            stats_text[..pos].trim().parse().unwrap_or(1)
        } else {
            1
        };

        let words = document
            .select(&Selector::parse("dd.words").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);

        let status_text = document
            .select(&Selector::parse("dd.status").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        let status = if status_text.contains("Complete") {
            "complete".to_string()
        } else {
            "ongoing".to_string()
        };

        // Generate url_id from source_id + work_id
        let url_id = crate::generate_url_id(1, &work_id);
        let now = Utc::now().timestamp_millis();
        let fic_url = format!("{}/works/{work_id}?view_full_work=true", self.base_url());

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters,
            words,
            desc: description,
            published: now,
            updated: now,
            status,
            source: fic_url,
            source_id: 1, // AO3 source ID
            author_id: 0,
            author_url,
            author_local_id: work_id.clone(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let url = format!("{}/works/{}?view_full_work=true", self.base_url(), meta.author_local_id);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        self.fetch_chapters_from_html(&html, meta).await
    }

    /// Parse chapters from a pre-fetched AO3 page (live, Wayback snapshot,
    /// cookie ingest, fixture).
    async fn fetch_chapters_from_html(&self, html: &str, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let document = Html::parse_document(html);

        let chapter_sel = Selector::parse("div.chapter").unwrap();
        let title_sel = Selector::parse("h3.title").unwrap();

        let mut chapters = Vec::new();
        for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
            let chapter_title = chapter_div
                .select(&title_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| format!("Chapter {i}"));

            let content = chapter_div
                .select(&Selector::parse("div.userstuff").unwrap())
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();

            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: chapter_title,
                content,
            });
        }

        if chapters.is_empty() {
            // If no chapter divs found, try reading the full work content
            if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
                let content = body.inner_html();
                if !content.is_empty() {
                    chapters.push(Chapter {
                        chapter_id: 1,
                        title: meta.title.clone(),
                        content,
                    });
                }
            }
        }

        Ok(chapters)
    }

    /// Extract tags from an AO3 work page.
    async fn extract_tags(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<Vec<ExtractedTag>, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

        let fic_url = format!("{}/works/{}?view_full_work=true", self.base_url(), work_id);
        let response = client
            .get(&fic_url)
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
        let mut tags = Vec::new();

        // Fandom: dd.fandom a.tag
        let fandom_sel = Selector::parse("dd.fandom a.tag").unwrap();
        let fandoms: Vec<String> = document
            .select(&fandom_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for (i, fandom) in fandoms.into_iter().enumerate() {
            let score = if i == 0 { 5.0 } else { 1.0 };
            tags.push(ExtractedTag::fandom(&fandom));
            tags.last_mut().unwrap().score = score;
        }

        // Characters: dd.characters a.tag
        let character_sel = Selector::parse("dd.characters a.tag").unwrap();
        let characters: Vec<String> = document
            .select(&character_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for (i, character) in characters.into_iter().enumerate() {
            let score = if i == 0 { 10.0 } else { 1.0 };
            let mut tag = ExtractedTag::character(&character);
            tag.score = score;
            tags.push(tag);
        }

        // Relationships: dd.relationships a.tag
        let relationship_sel = Selector::parse("dd.relationships a.tag").unwrap();
        let relationships: Vec<String> = document
            .select(&relationship_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for (i, relationship) in relationships.into_iter().enumerate() {
            let score = if i == 0 { 5.0 } else { 1.0 };
            let mut tag = ExtractedTag::relationship(&relationship);
            tag.score = score;
            tags.push(tag);

            // Categories embedded in relationships: Gen, F/M, M/M, Multi, Other
            let lower = relationship.to_lowercase();
            let category = if lower.contains("gen") && !lower.contains("gender") {
                Some("Gen")
            } else if lower.contains("f/m") {
                Some("F/M")
            } else if lower.contains("m/m") {
                Some("M/M")
            } else if lower.contains("multi") {
                Some("Multi")
            } else if lower.contains("other") {
                Some("Other")
            } else {
                None
            };
            if let Some(cat) = category {
                tags.push(ExtractedTag::category(cat));
            }
        }

        // Freeforms/Additional tags: dd.freeforms a.tag
        let freeform_sel = Selector::parse("dd.freeforms a.tag").unwrap();
        let freeforms: Vec<String> = document
            .select(&freeform_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for freeform in freeforms {
            tags.push(ExtractedTag::freeform(&freeform));
        }

        // Warnings: dd.warning a.tag
        let warning_sel = Selector::parse("dd.warning a.tag").unwrap();
        let warnings: Vec<String> = document
            .select(&warning_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for warning in warnings {
            tags.push(ExtractedTag::warning(&warning));
        }

        // Ratings: dd.rating a.tag (score 0)
        let rating_sel = Selector::parse("dd.rating a.tag").unwrap();
        let ratings: Vec<String> = document
            .select(&rating_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for rating in ratings {
            let mut tag = ExtractedTag::freeform(&rating); // Ratings stored as freeforms with score 0
            tag.score = 0.0;
            tags.push(tag);
        }

        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::{Html, Selector};

    #[test]
    fn test_title_selector_matches_ao3_html() {
        let html = r#"<h2 class="title heading">
    Shadows of Orario
  </h2>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("h2.title.heading").unwrap();
        let title = doc.select(&sel).next()
            .map(|el| el.text().collect::<String>().trim().to_string());
        assert_eq!(title.as_deref(), Some("Shadows of Orario"));
    }

    #[test]
    fn test_title_selector_matches_full_page() {
        let html = r#"<!DOCTYPE html><html><body>
        <div class="preface group">
          <h2 class="title heading">
            Shadows of Orario
          </h2>
          <h3 class="byline heading"><a rel="author" href="/users/test">Author</a></h3>
        </div>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("h2.title.heading").unwrap();
        let title = doc.select(&sel).next()
            .map(|el| el.text().collect::<String>().trim().to_string());
        assert_eq!(title.as_deref(), Some("Shadows of Orario"));
    }

    #[test]
    fn test_multi_author_fics_keep_all_creators() {
        // AO3 fics can have multiple creators — all a[rel='author'] links
        // must be captured (comma-joined), and the first is the primary
        // (whose profile URL is kept).
        let html = r#"<!DOCTYPE html><html><body>
        <div class="preface group">
          <h2 class="title heading">Collab Fic</h2>
          <h3 class="byline heading">
            <a rel="author" href="/users/alice">Alice</a>,
            <a rel="author" href="/users/bob">Bob</a>,
            <a rel="author" href="/users/carol">Carol</a>
          </h3>
        </div>
        </body></html>"#;
        let doc = Html::parse_document(html);

        let author_elements: Vec<String> = doc
            .select(&Selector::parse("a[rel='author']").unwrap())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let author = if author_elements.is_empty() {
            "Unknown Author".to_string()
        } else {
            author_elements.join(", ")
        };
        let author_url = doc
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("https://archiveofourown.org{h}"))
            .unwrap_or_default();

        assert_eq!(author, "Alice, Bob, Carol", "all creators must be kept");
        assert_eq!(author_url, "https://archiveofourown.org/users/alice", "primary author URL kept");
        assert_eq!(author_elements.len(), 3, "three creators");
    }

    #[test]
    fn test_single_author_still_works() {
        let html = r#"<!DOCTYPE html><html><body>
        <div class="preface group">
          <h2 class="title heading">Solo Fic</h2>
          <h3 class="byline heading"><a rel="author" href="/users/dave">Dave</a></h3>
        </div>
        </body></html>"#;
        let doc = Html::parse_document(html);
        let author_elements: Vec<String> = doc
            .select(&Selector::parse("a[rel='author']").unwrap())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let author = if author_elements.is_empty() {
            "Unknown Author".to_string()
        } else {
            author_elements.join(", ")
        };
        assert_eq!(author, "Dave");
    }

    #[test]
    fn otw_variant_domains_work() {
        let sq = Ao3Scraper::with_domain("squidgeworld.org");
        assert!(sq.can_handle("https://squidgeworld.org/works/12345"));
        assert!(!sq.can_handle("https://archiveofourown.org/works/12345"));
        let ao3 = Ao3Scraper::ao3();
        assert!(ao3.can_handle("https://archiveofourown.org/works/12345"));
        assert!(!ao3.can_handle("https://squidgeworld.org/works/12345"));
    }
}
