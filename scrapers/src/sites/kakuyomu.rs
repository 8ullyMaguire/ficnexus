//! Kakuyomu.jp native adapter (Japanese web-novel site).
//!
//! Story URL: `https://kakuyomu.jp/works/{storyId}`.
//! Metadata + TOC come from the Next.js payload:
//! `__NEXT_DATA__` → `props.pageProps.__APOLLO_STATE__` → `Work:{id}`:
//! - title, introduction (description), catchphrase
//! - author → `author.__ref` → `User:{id}` → activityName + name
//! - publishedAt / editedAt ("YYYY-MM-DDTHH:MM:SSZ"), totalCharacterCount,
//!   serialStatus ("COMPLETED" | "SERIALIZED"), tagLabels, genre
//! - tableOfContentsV2: TOC nodes → chapter refs (nested, level 1/2) +
//!   episodeUnions → episode refs (skip EmptyEpisode)
//! Episode body: `div.widget-episodeBody.js-episode-body`.

use async_trait::async_trait;
use chrono::TimeZone;
use scraper::{Html, Selector};
use serde_json::Value;

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct KakuyomuScraper;

impl KakuyomuScraper {
    fn story_id(url: &str) -> Option<String> {
        let id = url.split("/works/").nth(1)?.split('/').next()?.to_string();
        if id.chars().all(|c| c.is_ascii_digit()) {
            Some(id)
        } else {
            None
        }
    }
}

#[async_trait]
impl SiteScraper for KakuyomuScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("kakuyomu.jp/works/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id(url).ok_or_else(|| ScrapeError::ParseError("kakuyomu: bad url".into()))?;
        let html = http::fetch(client, url).await?;

        if html.contains("お探しのページは見つかりませんでした") {
            return Err(ScrapeError::NotFound);
        }

        // Extract the __NEXT_DATA__ JSON.
        let doc = Html::parse_document(&html);
        let payload = doc
            .select(&Selector::parse("#__NEXT_DATA__").map_err(|e| ScrapeError::ParseError(e.to_string()))?)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        let v: Value = serde_json::from_str(&payload).map_err(|e| ScrapeError::ParseError(format!("kakuyomu: json {e}")))?;

        let state = v
            .pointer("/props/pageProps/__APOLLO_STATE__")
            .ok_or_else(|| ScrapeError::ParseError("kakuyomu: no apollo state".into()))?;
        let work = state
            .get(&format!("Work:{story_id}"))
            .ok_or_else(|| ScrapeError::ParseError("kakuyomu: no work node".into()))?;

        let s = |k: &str| work.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        let title = s("title");
        if title.is_empty() {
            return Err(ScrapeError::ParseError("kakuyomu: no title".into()));
        }

        // Author via __ref.
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        if let Some(author_ref) = work.get("author").and_then(|a| a.get("__ref")).and_then(Value::as_str) {
            if let Some(user) = state.get(author_ref) {
                author = user.get("activityName").and_then(Value::as_str).unwrap_or("").to_string();
                author_local_id = author_ref.split(':').nth(1).unwrap_or("").to_string();
                if let Some(name) = user.get("name").and_then(Value::as_str) {
                    author_url = format!("https://kakuyomu.jp/users/{name}");
                }
            }
        }

        // Dates.
        let parse_dt = |s: &str| -> i64 {
            let s = s.trim_end_matches('Z');
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
            }
            0
        };
        let published = parse_dt(&s("publishedAt"));
        let updated = parse_dt(&s("editedAt"));

        // Words = totalCharacterCount.
        let words = work.get("totalCharacterCount").and_then(Value::as_i64).unwrap_or(0);

        // Status.
        let status = if s("serialStatus") == "COMPLETED" { "complete" } else { "ongoing" }.to_string();

        // Description.
        let desc = s("introduction");

        // Genre/tags (raw extended meta).
        let mut tags: Vec<String> = Vec::new();
        if let Some(tl) = work.get("tagLabels").and_then(Value::as_array) {
            for t in tl {
                if let Some(t) = t.as_str() {
                    tags.push(t.to_string());
                }
            }
        }
        let genre = s("genre");

        // Chapter count + TOC (collected for count only here).
        let chapters = count_toc_episodes(state, &story_id);

        Ok(FicMetadata {
            url_id: format!("kak_{story_id}"),
            title,
            author,
            chapters: chapters.max(1),
            words,
            desc,
            published,
            updated,
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!("genre={};tags={};", genre, tags.join(","))),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let story_id = Self::story_id(&meta.source).ok_or_else(|| ScrapeError::ParseError("kakuyomu: bad url".into()))?;
        let html = http::fetch(client, &meta.source).await?;

        // Parse TOC into owned (title, url) data.
        let toc: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let payload = doc
                .select(&Selector::parse("#__NEXT_DATA__").unwrap())
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
            let Ok(v) = serde_json::from_str::<Value>(&payload) else { return Ok(vec![]) };
            let Some(state) = v.pointer("/props/pageProps/__APOLLO_STATE__") else { return Ok(vec![]) };
            let Some(work) = state.get(&format!("Work:{story_id}")) else { return Ok(vec![]) };

            // Walk tableOfContentsV2 with chapter-title nesting (level 1/2).
            let mut titles: Vec<String> = Vec::new();
            let mut nesting = 0i32;
            let mut new_section = false;
            let mut out: Vec<(String, String)> = Vec::new();

            if let Some(toc_nodes) = work.get("tableOfContentsV2").and_then(Value::as_array) {
                for node_ref in toc_nodes {
                    let node_ref_str = match node_ref.get("__ref").and_then(Value::as_str) {
                        Some(r) => r.to_string(),
                        None => continue,
                    };
                    let Some(node) = state.get(&node_ref_str) else { continue };

                    if let Some(ch_ref) = node.get("chapter").and_then(|c| c.get("__ref")).and_then(Value::as_str) {
                        if let Some(chapter) = state.get(ch_ref) {
                            let level = chapter.get("level").and_then(Value::as_i64).unwrap_or(1) as i32;
                            while level <= nesting {
                                titles.pop();
                                nesting -= 1;
                            }
                            if let Some(t) = chapter.get("title").and_then(Value::as_str) {
                                titles.push(t.to_string());
                            }
                            nesting = level;
                            new_section = true;
                        }
                    } else {
                        titles.clear();
                        nesting = 0;
                        new_section = false;
                    }

                    if let Some(eps) = node.get("episodeUnions").and_then(Value::as_array) {
                        for ep_ref in eps {
                            let Some(ep_ref_str) = ep_ref.get("__ref").and_then(Value::as_str) else { continue };
                            if ep_ref_str.starts_with("EmptyEpisode") {
                                continue;
                            }
                            let Some(ep) = state.get(ep_ref_str) else { continue };
                            let ep_id = ep.get("id").and_then(Value::as_str).unwrap_or("");
                            let ep_title = ep.get("title").and_then(Value::as_str).unwrap_or("").to_string();
                            let ep_url = format!("https://kakuyomu.jp/works/{story_id}/episodes/{ep_id}");

                            // Prepend section titles (firstepisode mode).
                            let final_title = if !titles.is_empty() && new_section {
                                let mut parts = titles.clone();
                                parts.push(ep_title.clone());
                                parts.join("　")
                            } else {
                                ep_title
                            };
                            out.push((final_title, ep_url));
                        }
                        new_section = false;
                    }
                }
            }
            out
        };

        let mut chapters = Vec::new();
        for (i, (title, url)) in toc.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
                content,
            });
        }

        if chapters.is_empty() {
            let content = fetch_chapter_text(client, &meta.source).await;
            chapters.push(Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
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

/// Count episodes in the TOC (for the chapter count).
fn count_toc_episodes(state: &Value, story_id: &str) -> i32 {
    let Some(work) = state.get(&format!("Work:{story_id}")) else { return 0 };
    let Some(toc_nodes) = work.get("tableOfContentsV2").and_then(Value::as_array) else { return 0 };
    let mut n = 0i32;
    for node_ref in toc_nodes {
        let Some(node_ref_str) = node_ref.get("__ref").and_then(Value::as_str) else { continue };
        let Some(node) = state.get(node_ref_str) else { continue };
        if let Some(eps) = node.get("episodeUnions").and_then(Value::as_array) {
            for ep_ref in eps {
                if let Some(r) = ep_ref.get("__ref").and_then(Value::as_str) {
                    if !r.starts_with("EmptyEpisode") {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

/// Fetch a Kakuyomu episode and extract the episode body.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div.widget-episodeBody.js-episode-body") {
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
        let s = KakuyomuScraper;
        assert!(s.can_handle("https://kakuyomu.jp/works/12345678901234567890"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_story_id() {
        assert_eq!(
            KakuyomuScraper::story_id("https://kakuyomu.jp/works/12345678901234567890"),
            Some("12345678901234567890".to_string())
        );
        assert_eq!(KakuyomuScraper::story_id("https://kakuyomu.jp/works/abc"), None);
    }

    #[test]
    fn parses_next_data() {
        let payload = r#"{"props":{"pageProps":{"__APOLLO_STATE__":{"Work:42":{"title":"T","introduction":"D","totalCharacterCount":100,"serialStatus":"COMPLETED"}}}}}"#;
        let v: Value = serde_json::from_str(payload).unwrap();
        let state = v.pointer("/props/pageProps/__APOLLO_STATE__").unwrap();
        let work = state.get("Work:42").unwrap();
        assert_eq!(work.get("title").and_then(Value::as_str).unwrap(), "T");
        assert_eq!(work.get("totalCharacterCount").and_then(Value::as_i64).unwrap(), 100);
        assert_eq!(work.get("serialStatus").and_then(Value::as_str).unwrap(), "COMPLETED");
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="widget-episodeBody js-episode-body"><p>Japanese text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div.widget-episodeBody.js-episode-body") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Japanese text."));
    }
}
