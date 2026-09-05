/// RoyalRoad site scraper
/// Handles URLs like: https://www.royalroad.com/fiction/12345/fiction-title
pub struct RoyalRoadScraper;

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper, generate_url_id};
use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};

impl RoyalRoadScraper {
    fn extract_fiction_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/fiction/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for RoyalRoadScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("royalroad.com/fiction/")
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let response = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let doc = Html::parse_document(&html);
        let doc_html = html.clone();

        // Title from fiction page heading (RoyalRoad markup: h1.font-white)
        let title = doc
            .select(
                &Selector::parse("h1.font-white, h1[property='name'], .fiction-title h1").unwrap(),
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".into());

        // Author name (markup: <h4 class="font-white"><span>by</span><span><a href="/profile/…">)
        let author = doc
            .select(
                &Selector::parse(
                    "h4.font-white a[href*='/profile/'], h4[property='author'] a, .author-name a",
                )
                .unwrap(),
            )
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".into());

        // Description
        let desc = doc
            .select(&Selector::parse("div[property='description'], .description").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // RoyalRoad shows word count in a stats panel — look for inline text like "123,456 Words"
        let stats_text = doc
            .select(&Selector::parse(".fiction-stats, .stats-content").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        // Parse words from stats text
        let mut words = extract_number_after(&stats_text, "Words");

        // RoyalRoad removed the plain "N Words" stat from the fiction page
        // (2026 layout): the count now only appears inside the Pages tooltip
        // ("... calculated from 297,036 words."). Fall back to that.
        if words == 0 {
            if let Some(w) = extract_words_from_pages_tooltip(&doc_html) {
                words = w;
            }
        }

        // Parse chapter count from stats text
        let mut chapters = extract_number_after(&stats_text, "Chapters") as i32;

        // Fallback: chapter count from the "109 Chapters" pill on the page
        // (markup: <span class="label label-default pull-right">109 Chapters</span>)
        if chapters == 0 {
            // The "109 Chapters" pill: <span class="label label-default pull-right">
            let pill_text = doc
                .select(&Selector::parse("span.label.label-default.pull-right").unwrap())
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_default();
            chapters = extract_number_before(&pill_text, "Chapters") as i32;
        }

        let fiction_id = Self::extract_fiction_id(url).unwrap_or_default();
        let url_id = generate_url_id(6, &fiction_id);

        // Parse genre tags
        let tag_sel = Selector::parse("span[property='genre'], .tags a").unwrap();
        let tags: Vec<String> = doc
            .select(&tag_sel)
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters,
            words: words as i64,
            desc,
            published: 0, // RoyalRoad doesn't expose publish date easily in the DOM
            updated: Utc::now().timestamp_millis(),
            status: "ongoing".into(),
            source: url.to_string(),
            source_id: 6,
            author_id: 0,
            author_url: String::new(),
            author_local_id: fiction_id,
            content_hash: None,
            extra_meta: Some(tags.join(", ")),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        // Fetch the fiction page to extract chapter URLs
        let chapter_urls = {
            let response = client
                .get(&meta.source)
                .header("User-Agent", "fichub.net/0.1.0")
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;

            let html = response
                .text()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let doc = Html::parse_document(&html);

            // Get all chapter links from the table — drop doc before any further awaits
            let sel = Selector::parse("#chapters tbody tr a[href*='/chapter/']").unwrap();
            doc.select(&sel)
                .filter_map(|el| el.value().attr("href"))
                .map(|h| format!("https://www.royalroad.com{}", h))
                .collect::<Vec<String>>()
        }; // doc dropped here

        let mut chapters = Vec::new();
        for (i, ch_url) in chapter_urls.iter().enumerate() {
            let (title, content) = {
                let ch_resp = client
                    .get(ch_url)
                    .header("User-Agent", "fichub.net/0.1.0")
                    .send()
                    .await
                    .map_err(|e| ScrapeError::Network(e.to_string()))?;

                let ch_html = ch_resp
                    .text()
                    .await
                    .map_err(|e| ScrapeError::Network(e.to_string()))?;
                let ch_doc = Html::parse_document(&ch_html);

                let content = ch_doc
                    .select(&Selector::parse("div.chapter-content, .chapter-inner").unwrap())
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default();

                let title = ch_doc
                    .select(&Selector::parse("h2.chapter-title, .chapter-header h2").unwrap())
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| format!("Chapter {}", i + 1));

                (title, content)
            }; // ch_doc dropped here

            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title,
                content,
            });
        }

        Ok(chapters)
    }
}

/// Extract a number that follows a given label in text.
/// Finds patterns like "Words: 123,456" or "Words 123456" (case-insensitive).
/// Extract the true word count from the Pages tooltip text ("... calculated from 297,036 words.") used by current RoyalRoad pages.
fn extract_words_from_pages_tooltip(html: &str) -> Option<i64> {
    let re = regex_lite::Regex::new(r"calculated from ([\d,]+) words").ok()?;
    let caps = re.captures(html)?;
    caps.get(1)?.as_str().replace(',', "").parse().ok()
}

fn extract_number_after(text: &str, label: &str) -> i64 {
    let pattern = format!(r"(?i){}[:\s]*([\d,]+)", regex_lite::escape(label));
    let re = regex_lite::Regex::new(&pattern).ok();
    if let Some(caps) = re.and_then(|r| r.captures(text)) {
        if let Some(m) = caps.get(1) {
            return m.as_str().replace(',', "").parse().unwrap_or(0);
        }
    }
    0
}

/// Extract a number that appears BEFORE a label: "109 Chapters" → 109.
fn extract_number_before(text: &str, label: &str) -> i64 {
    let pattern = format!(r"(?i)([\d,]+)[:\s]*{}", regex_lite::escape(label));
    let re = regex_lite::Regex::new(&pattern).ok();
    if let Some(caps) = re.and_then(|r| r.captures(text)) {
        if let Some(m) = caps.get(1) {
            return m.as_str().replace(',', "").parse().unwrap_or(0);
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle_royalroad() {
        let scraper = RoyalRoadScraper;
        assert!(scraper.can_handle("https://www.royalroad.com/fiction/12345/test-story"));
        assert!(scraper.can_handle("https://royalroad.com/fiction/67890/another-story"));
        assert!(!scraper.can_handle("https://example.com/story"));
        assert!(!scraper.can_handle("https://fanfiction.net/story/123"));
    }

    #[test]
    fn test_extract_fiction_id() {
        assert_eq!(
            RoyalRoadScraper::extract_fiction_id(
                "https://www.royalroad.com/fiction/12345/test-story"
            ),
            Some("12345".into())
        );
        assert_eq!(
            RoyalRoadScraper::extract_fiction_id("https://royalroad.com/fiction/67890/"),
            Some("67890".into())
        );
        assert_eq!(
            RoyalRoadScraper::extract_fiction_id("https://example.com/not-royalroad"),
            None
        );
    }

    #[test]
    fn test_extract_number_after() {
        assert_eq!(extract_number_after("Words: 123,456", "Words"), 123456);
        assert_eq!(extract_number_after("Chapters 42", "Chapters"), 42);
        assert_eq!(extract_number_before("109 Chapters", "Chapters"), 109);
        assert_eq!(extract_number_before("12,345 Words", "Words"), 12345);
        assert_eq!(extract_number_after("Words 1,234", "Words"), 1234);
        assert_eq!(extract_number_after("nothing here", "Words"), 0);
        assert_eq!(extract_number_after("", "Words"), 0);
    }

    #[test]
    fn test_extract_number_after_case_insensitive() {
        assert_eq!(extract_number_after("words 500", "Words"), 500);
        assert_eq!(extract_number_after("WORDS 1000", "Words"), 1000);
    }
}
