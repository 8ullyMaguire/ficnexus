/// FanFiction.net site scraper
pub struct FfNetScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        // Matches fanfiction.net/s/NUMBER or /s/NUMBER/...
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for FfNetScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiction.net") || url.contains("fictionpress.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID from FF.net URL".into()))?;

        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
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

        let title = document
            .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let author_url = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("https://www.fanfiction.net{h}"))
            .unwrap_or_default();

        let description = document
            .select(&Selector::parse("#profile_top div.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let _stats_text: String = document
            .select(&Selector::parse("#profile_top span.xgray").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let words = document
            .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);

        let chapters = document
            .select(&Selector::parse("#profile_top span[data-xutitle='chapters']").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().trim().split('/').next()
                    .and_then(|s| s.trim().parse().ok())
            })
            .unwrap_or(1);

        let url_id = crate::scrape::generate_url_id(2, &story_id);
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
            status: "ongoing".to_string(),
            source: fic_url,
            source_id: 2,
            author_id: 0,
            author_url,
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();
        let story_id = &meta.author_local_id;

        for i in 1..=meta.chapters {
            let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
            let response = client
                .get(&url)
                .header("User-Agent", "fichub.net/0.1.0")
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;

            let html = response.text().await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let document = Html::parse_document(&html);

            let chapter_sel = Selector::parse("div.storytext").unwrap();
            let content = document
                .select(&chapter_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();

            let title = document
                .select(&Selector::parse("select#chap_select option[selected]").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| format!("Chapter {i}"));

            chapters.push(Chapter {
                chapter_id: i,
                title,
                content,
            });
        }

        Ok(chapters)
    }
}

/// FictionPress scraper (site shares same structure as FF.net)
pub use FfNetScraper as FictionPressScraper;
