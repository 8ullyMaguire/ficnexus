/// AO3 site scraper
/// Handles URLs like: https://archiveofourown.org/works/123456 or /works/123456/chapters/789012
pub struct Ao3Scraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
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
}

#[async_trait]
impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract work ID from AO3 URL".into()))?;

        let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
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

        let author = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

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
        let url_id = crate::scrape::generate_url_id(1, &work_id);
        let now = Utc::now().timestamp_millis();

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
        let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let chapter_sel = Selector::parse("div.chapter").unwrap();
        let title_sel = Selector::parse("h3.title").unwrap();

        let mut chapters = Vec::new();
        for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
            let chapter_title = chapter_div
                .select(&title_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| format!("Chapter {}", i + 1));

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
}
