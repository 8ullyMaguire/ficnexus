/// AdultFanFiction.org site scraper
pub struct AdultFanFictionScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

#[async_trait]
impl SiteScraper for AdultFanFictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("adult-fanfiction.org") || url.contains("adultfanfiction.net")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // AFF uses numeric story IDs
        let story_id = url.split('/').filter_map(|s| s.parse::<i64>().ok()).next()
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;

        let response = client
            .get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response.text().await.map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1.story_title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("a.author").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let description = document
            .select(&Selector::parse("div.story_description").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let url_id = crate::scrape::generate_url_id(4, &story_id.to_string());
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
            source_id: 4,
            author_id: 0,
            author_url: String::new(),
            author_local_id: story_id.to_string(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let response = client
            .get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await.map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content_sel = Selector::parse("div.story_content").unwrap();
        let mut chapters = Vec::new();

        for (i, content_div) in document.select(&content_sel).enumerate() {
            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: format!("Chapter {}", i + 1),
                content: content_div.inner_html(),
            });
        }

        Ok(chapters)
    }
}
