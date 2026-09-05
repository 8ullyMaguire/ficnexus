/// FanFiction.net site scraper
pub struct FfNetScraper;

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};

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

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url).ok_or_else(|| {
            ScrapeError::ParseError("could not extract story ID from FF.net URL".into())
        })?;

        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
        // FFN sits behind Cloudflare's "Just a moment" wall. Plain reqwest is
        // rejected regardless of UA; use TLS impersonation first, one-shot
        // chromium as fallback (see crate::cloudflare).
        let html = crate::cloudflare::fetch_with_cf_fallback(&fic_url).await?;

        if crate::cloudflare::is_cf_challenge(&html) {
            // Hard Cloudflare wall — the page we got is not the story. Fall
            // back to fichub.net's metadata API, which mirrors FFN and can
            // still tell us the real words/chapters for this story id.
            tracing::warn!("FFN CF challenge for {fic_url}; using fichub.net metadata fallback");
            return fallback_metadata(client, &fic_url, &story_id).await;
        }

        let meta = self.lookup_from_html(&html, url).await?;

        // Selector miss (markup change / Cloudflare-partial page) yields
        // words=0. fichub.net mirrors FFN and reports the real stats; use it
        // as a metadata fallback instead of persisting a 0-word fic.
        if meta.words == 0 && !meta.author.is_empty() && !meta.title.is_empty() {
            tracing::warn!(
                "FFN selector miss for {fic_url} (words=0); using fichub.net metadata fallback"
            );
            if let Ok(fb) = crate::fichub_net::fetch_metadata(client, &fic_url).await {
                if fb.words > 0 {
                    return Ok(fallback_to_metadata(&story_id, &fic_url, fb));
                }
            }
        }
        Ok(meta)
    }

    async fn lookup_from_html(&self, html: &str, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url).ok_or_else(|| {
            ScrapeError::ParseError("could not extract story ID from FF.net URL".into())
        })?;
        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");

        let (title, author, author_url, description, words, chapters) = {
            let document = Html::parse_document(html);

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
                .and_then(|el| el.text().collect::<String>().replace(',', "").parse().ok())
                .unwrap_or(0);

            let chapters = document
                .select(&Selector::parse("#profile_top span[data-xutitle='chapters']").unwrap())
                .next()
                .and_then(|el| {
                    el.text()
                        .collect::<String>()
                        .trim()
                        .split('/')
                        .next()
                        .and_then(|s| s.trim().parse().ok())
                })
                .unwrap_or(1);

            (title, author, author_url, description, words, chapters)
        };

        // Selector miss (markup change / Cloudflare-partial page) yields
        // words=0. fichub.net mirrors FFN and reports the real stats; use it
        // as a metadata fallback instead of persisting a 0-word fic.
        // (client is unavailable in the from-html path; the live lookup
        // handles the fichub fallback after this returns.)
        let meta = {
            let url_id = crate::generate_url_id(2, &story_id);
            let now = Utc::now().timestamp_millis();

            FicMetadata {
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
            }
        };
        Ok(meta)
    }

    async fn fetch_chapters(
        &self,
        _client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();
        let story_id = &meta.author_local_id;

        for i in 1..=meta.chapters {
            let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
            let html = crate::cloudflare::fetch_with_cf_fallback(&url).await?;
            let chapter = Self::parse_chapter_page(&html, i)?;
            chapters.push(chapter);
        }

        Ok(chapters)
    }

    async fn fetch_chapters_from_html(
        &self,
        _html: &str,
        _meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        // FFN chapters are per-page; a Wayback snapshot of the full-work URL
        // is not something FFN produces (each chapter has its own URL). For
        // from-html ingest, we can only parse a single page: use the
        // `?view_full_work=true` snapshot equivalent — FFN has no such mode,
        // so return Unsupported and let the caller fall back to M2.
        Err(ScrapeError::Unsupported(
            "FFN has no full-work view; wayback from-html chapter ingest unsupported".into(),
        ))
    }

    /// Extract tags from an FFN work page.
    /// FFN shows fandom in the breadcrumb/header area and sometimes categories.
    /// Characters/Relationships are NOT available on FFN pages.
    async fn extract_tags(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<Vec<crate::ExtractedTag>, ScrapeError> {
        let story_id = Self::extract_story_id(url).ok_or_else(|| {
            ScrapeError::ParseError("could not extract story ID from FFN URL".into())
        })?;

        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1");
        let response = client
            .get(&fic_url)
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
        let mut tags = Vec::new();

        // FFN fandom is in the breadcrumb: <a href='/fanfiction/book/...'>Fandom Name</a>
        if let Ok(sel) = Selector::parse("a[href*='/fanfiction/book/']") {
            for el in doc.select(&sel) {
                let name = el.text().collect::<String>().trim().to_string();
                if !name.is_empty() {
                    tags.push(crate::ExtractedTag {
                        name,
                        tag_type_id: 1, // fandom
                        score: 0.0,
                    });
                }
            }
        }

        // FFN sometimes shows genre in a link or text
        // The genre is usually shown as a standalone link or in a <select> element
        if let Ok(sel) = Selector::parse("select#genre option[selected]") {
            for el in doc.select(&sel) {
                let name = el.text().collect::<String>().trim().to_string();
                if !name.is_empty() && name != "All" {
                    tags.push(crate::ExtractedTag {
                        name,
                        tag_type_id: 6, // category
                        score: 0.0,
                    });
                }
            }
        }

        Ok(tags)
    }
}

impl FfNetScraper {
    /// Parse a single chapter page (`div.storytext` content + selected
    /// chapter title). Used by both the live fetch loop and any from-html
    /// caller that passes a per-chapter snapshot.
    fn parse_chapter_page(html: &str, i: i32) -> Result<Chapter, ScrapeError> {
        let document = Html::parse_document(html);

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

        Ok(Chapter {
            chapter_id: i,
            title,
            content,
        })
    }
}

/// Map fichub.net API metadata into our `FicMetadata`.
fn fallback_to_metadata(
    story_id: &str,
    fic_url: &str,
    fb: crate::fichub_net::FallbackMeta,
) -> FicMetadata {
    let now = chrono::Utc::now().timestamp_millis();
    let updated = chrono::DateTime::parse_from_rfc3339(&fb.updated)
        .map(|dt| dt.timestamp_millis())
        .unwrap_or(now);
    FicMetadata {
        url_id: crate::generate_url_id(2, story_id),
        title: fb.title,
        author: fb.author,
        chapters: fb.chapters.clamp(1, i32::MAX as i64) as i32,
        words: fb.words.max(0),
        desc: fb.description,
        published: now,
        updated,
        status: if fb.status.is_empty() {
            "ongoing".to_string()
        } else {
            fb.status
        },
        source: fic_url.to_string(),
        source_id: 2,
        author_id: fb.author_id,
        author_url: if fb.author_url.starts_with("http") {
            fb.author_url
        } else if fb.author_url.is_empty() {
            String::new()
        } else {
            format!("https://www.fanfiction.net{}", fb.author_url)
        },
        author_local_id: if fb.author_local_id.is_empty() {
            story_id.to_string()
        } else {
            fb.author_local_id
        },
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    }
}

/// Fetch metadata from the fichub.net API, mapping the response into
/// `FicMetadata`. Used when the native page fetch hit a hard CF wall.
async fn fallback_metadata(
    client: &reqwest::Client,
    fic_url: &str,
    story_id: &str,
) -> Result<FicMetadata, ScrapeError> {
    let fb = crate::fichub_net::fetch_metadata(client, fic_url).await?;
    Ok(fallback_to_metadata(story_id, fic_url, fb))
}

/// FictionPress scraper (site shares same structure as FF.net)
pub use FfNetScraper as FictionPressScraper;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_metadata_maps_fields() {
        let fb = crate::fichub_net::FallbackMeta {
            title: "New Blood".into(),
            author: "artemisgirl".into(),
            words: 2_100_000,
            chapters: 661,
            status: "ongoing".into(),
            updated: "2024-09-13T19:23:18+00:00".into(),
            description: "Desc".into(),
            id: "Fe5PRMGE".into(),
            source_id: 2,
            author_url: "/u/12345/".into(),
            raw_extended_meta: None,
            author_id: 0,
            author_local_id: String::new(),
        };
        let meta = fallback_to_metadata("13051824", "https://www.fanfiction.net/s/13051824/1/", fb);

        assert_eq!(meta.url_id, "2_13051824");
        assert_eq!(meta.title, "New Blood");
        assert_eq!(meta.author, "artemisgirl");
        assert_eq!(meta.words, 2_100_000);
        assert_eq!(meta.chapters, 661);
        assert_eq!(meta.author_url, "https://www.fanfiction.net/u/12345/");
        assert_eq!(meta.author_local_id, "13051824");
        assert_eq!(meta.status, "ongoing");
        assert_eq!(meta.source_id, 2);
    }

    #[test]
    fn fallback_metadata_clamps_chapters() {
        let fb = crate::fichub_net::FallbackMeta {
            chapters: 0,
            words: 0,
            status: String::new(),
            ..Default::default()
        };
        let meta = fallback_to_metadata("11574569", "https://www.fanfiction.net/s/11574569/1/", fb);
        assert_eq!(meta.chapters, 1);
        assert_eq!(meta.words, 0);
        assert_eq!(meta.status, "ongoing");
    }
}
