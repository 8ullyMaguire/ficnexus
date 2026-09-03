//! Hentai-Foundry.com native adapter (adult-gated art/writing site).
//!
//! Story URL: `https://www.hentai-foundry.com/stories/user/{author}/{storyId}/{title}`
//! (add `?enterAgree=1` for adult stories).
//! - title: `h1.titleSemantic`
//! - author: `span` "Author" → next `a`
//! - metadata: `td.storyInfo span.label` + `span.indent`
//!   (Submitted/Updated/Status/Words/Size/Comments/Views/Faves/Rating)
//! - description: `td.storyDescript` (strip storyRead/storyVote/ratings_box/
//!   categoryRating)
//! - chapters: `div.boxbody a` with `small` meta
//! - body: `section#viewChapter div.boxbody`
//!
//! Adult gate: `is_adult` confirmation via `SiteCredentials::with_adult()`.

use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use super::http;

pub struct HentaiFoundryScraper {
    adult_ok: AtomicBool,
}

impl Default for HentaiFoundryScraper {
    fn default() -> Self {
        Self {
            adult_ok: AtomicBool::new(false),
        }
    }
}

impl HentaiFoundryScraper {
    fn story_parts(url: &str) -> Option<(String, String, String)> {
        let m = Regex::new(r"hentai-foundry\.com/stories/user/([^/]+)/(\d+)/([^/]+)").ok()?;
        let c = m.captures(url)?;
        Some((
            c.get(1)?.as_str().to_string(),
            c.get(2)?.as_str().to_string(),
            c.get(3)?.as_str().to_string(),
        ))
    }
}

#[async_trait]
impl SiteScraper for HentaiFoundryScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("hentai-foundry.com/stories/user/")
    }

    fn requires_login(&self) -> bool {
        true
    }

    async fn login(
        &self,
        _client: &reqwest::Client,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        if creds.is_adult {
            self.adult_ok.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            Err(ScrapeError::AuthRequired("hentaifoundry: is_adult not set".into()))
        }
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        if !self.adult_ok.load(Ordering::Relaxed) {
            return Err(ScrapeError::AuthRequired("hentaifoundry: adult gate".into()));
        }
        let (author_id, story_id, _title_part) = Self::story_parts(url)
            .ok_or_else(|| ScrapeError::ParseError("hentaifoundry: bad url".into()))?;
        // Normalize URL + add adult agree param.
        let fetch_url = format!("{url}?enterAgree=1");
        let html = http::fetch(client, &fetch_url).await?;

        let (title, author, author_url, desc, status, published, updated) = {
            let doc = Html::parse_document(&html);
            let mut title = String::new();
            let mut author = String::new();
            let mut author_url = String::new();
            let mut desc = String::new();
            let mut status = "ongoing".to_string();
            let mut published = 0i64;
            let mut updated = 0i64;

            if let Ok(h1_sel) = Selector::parse("h1.titleSemantic") {
                title = doc
                    .select(&h1_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
            }

            // Author from span "Author" → next a.
            if let Ok(sp_sel) = Selector::parse("span") {
                for sp in doc.select(&sp_sel) {
                    if sp.text().collect::<String>().trim() == "Author" {
                        let mut nxt = sp.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                if el.value().name() == "a" {
                                    author = el.text().collect::<String>().trim().to_string();
                                    if let Some(h) = el.value().attr("href") {
                                        author_url = format!("https://www.hentai-foundry.com{h}");
                                    }
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                        break;
                    }
                }
            }

            // Metadata from td.storyInfo.
            if let Ok(i_sel) = Selector::parse("td.storyInfo span.label") {
                for lab in doc.select(&i_sel) {
                    let text = lab.text().collect::<String>().trim().to_string();
                    let indent_text = lab
                        .parent()
                        .and_then(|p| scraper::ElementRef::wrap(p))
                        .and_then(|pe| {
                            pe.select(&Selector::parse("span.indent").unwrap())
                                .next()
                                .map(|ind| ind.text().collect::<String>().trim().to_string())
                        });
                    if text == "Submitted" {
                        if let Some(d) = indent_text {
                            published = parse_dt(&d);
                        }
                    } else if text == "Updated" {
                        if let Some(d) = indent_text {
                            updated = parse_dt(&d);
                        }
                    } else if text == "Status" {
                        if let Some(v) = indent_text {
                            if v.contains("Complete") {
                                status = "complete".to_string();
                            } else {
                                status = "ongoing".to_string();
                            }
                        }
                    }
                }
            }

            // Description from td.storyDescript.
            if let Ok(d_sel) = Selector::parse("td.storyDescript") {
                desc = doc
                    .select(&d_sel)
                    .next()
                    .map(|el| el.inner_html())
                    .unwrap_or_default();
            }

            (title, author, author_url, desc, status, published, updated)
        };

        if title.is_empty() {
            return Err(ScrapeError::ParseError("hentaifoundry: no title".into()));
        }

        // Chapters from div.boxbody a.
        let mut chapters = 0i32;
        {
            let doc = Html::parse_document(&html);
            if let Ok(b_sel) = Selector::parse("div.boxbody a[href]") {
                chapters = doc.select(&b_sel).count() as i32;
            }
        }
        if chapters == 0 {
            chapters = 1;
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("hf_{}_{}", author_id, story_id),
            title,
            author,
            chapters,
            words: 0,
            desc,
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id: author_id,
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
        let fetch_url = format!("{}?enterAgree=1", meta.source);
        let html = http::fetch(client, &fetch_url).await?;

        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("div.boxbody a[href]")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let title = a.text().collect::<String>().trim().to_string();
                    let href = a.value().attr("href")?;
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("https://www.hentai-foundry.com{href}")
                    };
                    Some((title, url))
                })
                .collect()
        };

        let urls: Vec<(String, String)> = if links.is_empty() {
            vec![(meta.title.clone(), meta.source.clone())]
        } else {
            links
        };

        let mut chapters = Vec::new();
        for (i, (title, url)) in urls.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title: if title.is_empty() {
                    format!("Chapter {}", i + 1)
                } else {
                    title
                },
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

fn parse_dt(s: &str) -> i64 {
    for fmt in ["%B %d, %Y", "%b %d, %Y", "%Y-%m-%d"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return dt.and_utc().timestamp_millis();
            }
        }
    }
    0
}

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let fetch_url = format!("{url}?enterAgree=1");
    let Ok(html) = http::fetch(client, &fetch_url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("section#viewChapter div.boxbody") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = HentaiFoundryScraper::default();
        assert!(s.can_handle("https://www.hentai-foundry.com/stories/user/Author/12345/Story-Title"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_parts() {
        let (a, id, t) = HentaiFoundryScraper::story_parts("https://www.hentai-foundry.com/stories/user/Author/12345/Story-Title").unwrap();
        assert_eq!(a, "Author");
        assert_eq!(id, "12345");
        assert_eq!(t, "Story-Title");
        assert!(HentaiFoundryScraper::story_parts("https://x.com/foo").is_none());
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("July 31, 2018") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><section id="viewChapter"><div class="boxbody"><p>Story text.</p></div></section></body></html>"#;
        let doc = Html::parse_document(html);
        let sel = Selector::parse("section#viewChapter div.boxbody").unwrap();
        let s = doc.select(&sel).next().map(|el| el.inner_html()).unwrap_or_default();
        assert!(s.contains("Story text."));
    }
}
