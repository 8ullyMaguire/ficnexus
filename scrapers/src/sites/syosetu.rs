//! Syosetu.com native adapter (Japanese web-novel site, ncode/novel18).
//!
//! Metadata lives on the info page:
//!   `https://{ncode|novel18}.syosetu.com/novelview/infotop/ncode/{id}/`
//! with a `<dl>` table of `<dt>label</dt><dd>value</dd>` pairs (labels:
//! 作者名 author, あらすじ description, 掲載日 published, 文字数 word count,
//! キーワード keywords, ジャンル genre, 掲載サイト imprint...).
//!
//! TOC: `https://{host}/{id}/?p=N` (paginated 100 chapters/page), with
//! `div.p-eplist__chapter-title` section headers and
//! `div.p-eplist__sublist a` episode links.
//!
//! Chapter body: `div.p-novel__text` (`--preface`/`--afterword` → notes,
//! plain → body).

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

pub struct SyosetuScraper;

impl SyosetuScraper {
    fn story_id_from_url(url: &str) -> Option<String> {
        // /n1234ab/ or /novelview/infotop/ncode/n1234ab/
        let re = Regex::new(r"(?:ncode/)?(n\d+[a-z]*)/?").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for SyosetuScraper {
    fn can_handle(&self, url: &str) -> bool {
        (url.contains("ncode.syosetu.com") || url.contains("novel18.syosetu.com"))
            && (url.contains("/n") || url.contains("novelview"))
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::story_id_from_url(url)
            .ok_or_else(|| ScrapeError::ParseError("no syosetu id".into()))?;
        let host = if url.contains("novel18") {
            "novel18.syosetu.com"
        } else {
            "ncode.syosetu.com"
        };

        // Adult gate for novel18.
        if host == "novel18.syosetu.com" {
            return Err(ScrapeError::AuthRequired("novel18 adult gate".into()));
        }

        let info_url = format!("https://{host}/novelview/infotop/ncode/{story_id}/");
        let html = http::fetch(client, &info_url).await?;

        if html.contains("投稿済作品が見つかりません")
            || html.contains("この作品は作者によって削除されました")
        {
            return Err(ScrapeError::NotFound);
        }

        let doc = Html::parse_document(&html);

        // Title: the info page has <a href="{story_url}">{title}</a>.
        let story_url = format!("https://{host}/{story_id}/");
        let mut title = String::new();
        let title_sel = Selector::parse(&format!("a[href='{story_url}']"))
            .unwrap_or_else(|_| Selector::parse("a").unwrap());
        if let Some(a) = doc.select(&title_sel).next() {
            title = a.text().collect::<String>().trim().to_string();
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("syosetu: no title".into()));
        }

        // Author from 作者名 dt/dd.
        let mut author = String::new();
        let mut author_url = String::new();
        if let Some(a) = entry(&doc, &["作者名"]) {
            author = a.text().collect::<String>().trim().to_string();
            if let Ok(sel) = Selector::parse("a") {
                if let Some(link) = a.select(&sel).next() {
                    author_url = link.value().attr("href").unwrap_or("").to_string();
                }
            }
        }

        // Description from あらすじ.
        let mut desc = String::new();
        if let Some(d) = entry(&doc, &["あらすじ"]) {
            desc = d.inner_html();
        }

        // Published from 掲載日 "2017年 05月16日 17時30分".
        let mut published = 0i64;
        if let Some(p) = entry(&doc, &["掲載日"]) {
            published = parse_jp_date(&p.text().collect::<String>()).unwrap_or(0);
        }

        // Word count from 文字数 "123,789文字".
        let mut words = 0i64;
        if let Some(w) = entry(&doc, &["文字数"]) {
            words = w
                .text()
                .collect::<String>()
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
        }

        // Status + chapter count.
        let mut status = "ongoing".to_string();
        let mut chapters = 1i32;
        if let Ok(sel) = Selector::parse("span.p-infotop-type__type") {
            if let Some(el) = doc.select(&sel).next() {
                let t = el.text().collect::<String>().trim().to_string();
                if t.contains("短編") {
                    status = "complete".to_string();
                    chapters = 1;
                } else if t.contains("完結済") {
                    status = "complete".to_string();
                }
            }
        }
        if let Ok(sel) = Selector::parse("span.p-infotop-type__allep") {
            if let Some(el) = doc.select(&sel).next() {
                let n: i32 = el
                    .text()
                    .collect::<String>()
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap_or(1);
                if n > 0 {
                    chapters = n;
                }
            }
        }

        // Keywords from キーワード.
        let mut tags: Vec<String> = Vec::new();
        if let Some(k) = entry(&doc, &["キーワード"]) {
            tags = k
                .text()
                .collect::<String>()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("syos_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: if published > 0 { published } else { now },
            updated: now,
            status,
            source: story_url,
            source_id: 0,
            author_id: 0,
            author_url,
            author_local_id: story_id.clone(),
            content_hash: None,
            extra_meta: Some(format!("tags={};", tags.join(","))),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        // One-shot: single chapter at the story URL.
        if meta.chapters <= 1 {
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        // Serialized: paginated TOC (?p=N, 100/page).
        let mut chapters = Vec::new();
        let pages = (meta.chapters as f64 / 100.0).ceil() as i32;
        for page in 1..=pages.max(1) {
            let toc_url = format!("{}?p={}", meta.source, page);
            let Ok(toc_html) = http::fetch(client, &toc_url).await else {
                continue;
            };

            // Collect (title, url) per page into owned data, dropping the
            // non-Send doc before the per-chapter body fetch.
            let page_links: Vec<(String, String)> = {
                let doc = Html::parse_document(&toc_html);
                let mut section = String::new();
                if let Ok(ch_sel) = Selector::parse("div.p-eplist__chapter-title") {
                    for el in doc.select(&ch_sel) {
                        section = el.text().collect::<String>().trim().to_string();
                    }
                }
                let sub_sel = Selector::parse("div.p-eplist__sublist a")
                    .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
                doc.select(&sub_sel)
                    .filter_map(|a| {
                        let ep_title = a.text().collect::<String>().trim().to_string();
                        let href = a.value().attr("href")?;
                        let ep_url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://ncode.syosetu.com{href}")
                        };
                        let title = if !section.is_empty() && !ep_title.is_empty() {
                            format!("{section} — {ep_title}")
                        } else {
                            ep_title
                        };
                        Some((title, ep_url))
                    })
                    .collect()
            };

            for (title, ep_url) in page_links {
                let content = fetch_chapter_text(client, &ep_url).await;
                chapters.push(Chapter {
                    chapter_id: chapters.len() as i32 + 1,
                    title,
                    content,
                });
            }
        }

        if chapters.is_empty() {
            return Err(ScrapeError::ParseError("syosetu: no chapters".into()));
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

/// Find a `<dt>` label and return its following `<dd>` sibling (getEntry port).
fn entry<'a>(doc: &'a Html, labels: &[&str]) -> Option<scraper::ElementRef<'a>> {
    for label in labels {
        if let Ok(sel) = Selector::parse("dt") {
            for dt in doc.select(&sel) {
                if dt.text().collect::<String>().trim() == *label {
                    // The <dd> is the next sibling of the <dt>.
                    let mut nxt = dt.next_sibling();
                    while let Some(node) = nxt {
                        if let Some(el) = scraper::ElementRef::wrap(node) {
                            if el.value().name() == "dd" {
                                return Some(el);
                            }
                        }
                        nxt = node.next_sibling();
                    }
                }
            }
        }
    }
    None
}

/// Parse "2017年 05月16日 17時30分" → unix millis.
fn parse_jp_date(s: &str) -> Option<i64> {
    // "2017年 05月16日 17時30分" (spaces may vary)
    let re = Regex::new(r"(\d{4})年\s*(\d{1,2})月(\d{1,2})日\s*(\d{1,2})時(\d{1,2})分").ok()?;
    let c = re.captures(s)?;
    let y: i32 = c.get(1)?.as_str().parse().ok()?;
    let m: u32 = c.get(2)?.as_str().parse().ok()?;
    let d: u32 = c.get(3)?.as_str().parse().ok()?;
    let h: u32 = c.get(4)?.as_str().parse().ok()?;
    let mi: u32 = c.get(5)?.as_str().parse().ok()?;
    let dt = chrono::NaiveDate::from_ymd_opt(y, m, d)?.and_hms_opt(h, mi, 0)?;
    Some(chrono::Utc.from_utc_datetime(&dt).timestamp_millis())
}

/// Fetch a Syosetu chapter page and extract `div.p-novel__text`.
async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    let mut out = String::new();
    if let Ok(sel) = Selector::parse("div.p-novel__text") {
        for el in doc.select(&sel) {
            let class = el.value().attr("class").unwrap_or("");
            if class.contains("preface") || class.contains("afterword") {
                out.push_str(&format!(
                    "<div class=\"fff_chapter_notes\">{}</div>",
                    el.inner_html()
                ));
            } else {
                out.push_str(&el.inner_html());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_story_id() {
        assert_eq!(
            SyosetuScraper::story_id_from_url("https://ncode.syosetu.com/n1234ab/").as_deref(),
            Some("n1234ab")
        );
        assert_eq!(
            SyosetuScraper::story_id_from_url(
                "https://ncode.syosetu.com/novelview/infotop/ncode/n1234ab/"
            )
            .as_deref(),
            Some("n1234ab")
        );
    }

    #[test]
    fn can_handle_matches() {
        let s = SyosetuScraper;
        assert!(s.can_handle("https://ncode.syosetu.com/n1234ab/"));
        assert!(s.can_handle("https://novel18.syosetu.com/n1234a/"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_jp_dates() {
        assert!(parse_jp_date("2017年 05月16日 17時30分").is_some());
        assert_eq!(parse_jp_date("garbage"), None);
    }

    #[test]
    fn extracts_chapter_body() {
        let html =
            r#"<html><body><div class="p-novel__text"><p>Japanese text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div.p-novel__text") {
            for el in doc.select(&sel) {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Japanese text."));
    }
}
