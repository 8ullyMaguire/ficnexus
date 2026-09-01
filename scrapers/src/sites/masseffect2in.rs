//! MassEffect2.in native adapter (Russian Ucoz Mass Effect archive).
//!
//! Story URL: `https://www.masseffect2.in/publ/{doc-id}` where doc-id is
//! `{n}-{n}-{n}-{n}`. Chapters are followed via prev/next links
//! (`a[title="Предыдущая глава"]` / `Следующая глава`). Heading is
//! `h1[itemprop=headline]` ("Story — Chapter"), author from
//! `span.glyphicon-user` → next `a`, date `time[itemprop=dateCreated]`,
//! body `div[itemprop=articleBody]`. The story title is the longest
//! common prefix of all chapter headings.

use async_trait::async_trait;
use regex_lite::Regex;
use scraper::{Html, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

pub struct MassEffect2InScraper;

impl MassEffect2InScraper {
    fn doc_id(url: &str) -> Option<String> {
        let m = Regex::new(r"masseffect2\.in/publ/(\d+-\d+-\d+-\d+)").ok()?;
        m.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[derive(Clone)]
struct ChapterPage {
    heading: String,
    author: Option<(String, String)>, // (id, name)
    date: Option<i64>,
    body: String,
    next: Option<String>,
    prev: Option<String>,
}

fn parse_chapter_page(html: &str) -> ChapterPage {
    let doc = Html::parse_document(html);
    let mut cp = ChapterPage {
        heading: String::new(),
        author: None,
        date: None,
        body: String::new(),
        next: None,
        prev: None,
    };

    if let Ok(h1_sel) = Selector::parse("h1[itemprop='headline']") {
        cp.heading = doc
            .select(&h1_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
    }

    // Author: span.glyphicon-user → next sibling a.
    if let Ok(sp_sel) = Selector::parse("span.glyphicon-user") {
        if let Some(sp) = doc.select(&sp_sel).next() {
            let mut nxt = sp.next_sibling();
            while let Some(node) = nxt {
                if let Some(el) = scraper::ElementRef::wrap(node) {
                    if el.value().name() == "a" {
                        let name = el.text().collect::<String>().trim().to_string();
                        let onclick = el.value().attr("onclick").unwrap_or("");
                        let id = Regex::new(r"(8-\d+)")
                            .ok()
                            .and_then(|re| re.captures(onclick))
                            .map(|c| c.get(1).unwrap().as_str().to_string())
                            .unwrap_or_default();
                        cp.author = Some((id, name));
                        break;
                    }
                }
                nxt = node.next_sibling();
            }
        }
    }

    // Date
    if let Ok(t_sel) = Selector::parse("time[itemprop='dateCreated']") {
        if let Some(t) = doc.select(&t_sel).next() {
            let text = t.text().collect::<String>().trim().to_string();
            cp.date = parse_russian_date(&text);
        }
    }

    // Body
    if let Ok(b_sel) = Selector::parse("div[itemprop='articleBody']") {
        cp.body = doc
            .select(&b_sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
    }

    // Prev/next links.
    if let Ok(a_sel) = Selector::parse("a[title]") {
        for a in doc.select(&a_sel) {
            let title = a.value().attr("title").unwrap_or("");
            let href = a.value().attr("href").unwrap_or("");
            let url = if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://www.masseffect2.in{href}")
            };
            if title == "Предыдущая глава" {
                cp.prev = Some(url);
            } else if title == "Следующая глава" {
                cp.next = Some(url);
            }
        }
    }

    cp
}

/// Parse Russian date formats: "Вчера" (yesterday), "Сегодня" (today),
/// or "DD.MM.YYYY, HH:MM" (Europe/Moscow = UTC+3).
fn parse_russian_date(text: &str) -> Option<i64> {
    let now = chrono::Utc::now();
    if text == "Сегодня" {
        return Some(now.timestamp_millis());
    }
    if text == "Вчера" {
        return Some((now - chrono::Duration::days(1)).timestamp_millis());
    }
    for fmt in ["%d.%m.%Y, %H:%M", "%d.%m.%Y"] {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(text, fmt) {
            // Moscow is UTC+3.
            let ts = dt.and_utc().timestamp_millis() - 3 * 3600 * 1000;
            return Some(ts);
        }
        if let Ok(d) = chrono::NaiveDate::parse_from_str(text, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                let ts = dt.and_utc().timestamp_millis() - 3 * 3600 * 1000;
                return Some(ts);
            }
        }
    }
    None
}

/// Longest common prefix of two strings (by chars).
fn common_prefix(a: &str, b: &str) -> String {
    let mut out = String::new();
    for (ca, cb) in a.chars().zip(b.chars()) {
        if ca == cb {
            out.push(ca);
        } else {
            break;
        }
    }
    out
}

#[async_trait]
impl SiteScraper for MassEffect2InScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("masseffect2.in/publ/")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let doc_id = Self::doc_id(url).ok_or_else(|| ScrapeError::ParseError("masseffect2.in: bad url".into()))?;
        let html = http::fetch(client, url).await?;
        let start = parse_chapter_page(&html);

        // Follow prev to find the first chapter.
        let mut pages = Vec::new();
        let _cur = Some(url.to_string());
        let mut seen = std::collections::HashSet::new();
        // Go back to the start.
        let mut first = start;
        while let Some(p) = first.prev.clone() {
            if !seen.insert(p.clone()) {
                break;
            }
            if let Ok(h) = http::fetch(client, &p).await {
                let np = parse_chapter_page(&h);
                if np.heading.is_empty() {
                    break;
                }
                first = np;
            } else {
                break;
            }
        }
        pages.push(first.clone());

        // Follow next from the start.
        let mut cur = first;
        while let Some(n) = cur.next.clone() {
            if !seen.insert(n.clone()) {
                break;
            }
            if let Ok(h) = http::fetch(client, &n).await {
                let np = parse_chapter_page(&h);
                if np.heading.is_empty() {
                    break;
                }
                pages.push(np.clone());
                cur = np;
            } else {
                break;
            }
        }

        // Title = longest common prefix of headings.
        let mut title = pages[0].heading.clone();
        for p in pages.iter().skip(1) {
            title = common_prefix(&title, &p.heading);
        }
        let title = title
            .trim_matches(|c: char| c == ':' || c == '.' || c.is_whitespace())
            .to_string();
        if title.is_empty() {
            return Err(ScrapeError::ParseError("masseffect2.in: no title".into()));
        }

        // Author from the first page that has one.
        let mut author = String::new();
        let mut author_id = String::new();
        let mut author_url = String::new();
        for p in pages.iter() {
            if let Some((id, name)) = &p.author {
                author = name.clone();
                author_id = id.clone();
                author_url = format!("https://www.masseffect2.in/index/{id}");
                break;
            }
        }

        let dates: Vec<i64> = pages.iter().filter_map(|p| p.date).collect();
        let published = dates.first().copied().unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let updated = dates.last().copied().unwrap_or(published);

        Ok(FicMetadata {
            url_id: format!("me2_{}", doc_id),
            title,
            author,
            chapters: pages.len() as i32,
            words: 0,
            desc: String::new(),
            published,
            updated,
            status: "complete".to_string(),
            source: url.to_string(),
            source_id: 0,
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
        let html = http::fetch(client, &meta.source).await?;
        let start = parse_chapter_page(&html);

        let mut pages = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Back to start.
        let mut first = start;
        while let Some(p) = first.prev.clone() {
            if !seen.insert(p.clone()) {
                break;
            }
            if let Ok(h) = http::fetch(client, &p).await {
                let np = parse_chapter_page(&h);
                if np.heading.is_empty() {
                    break;
                }
                first = np;
            } else {
                break;
            }
        }
        pages.push(first.clone());

        let mut cur = first;
        while let Some(n) = cur.next.clone() {
            if !seen.insert(n.clone()) {
                break;
            }
            if let Ok(h) = http::fetch(client, &n).await {
                let np = parse_chapter_page(&h);
                if np.heading.is_empty() {
                    break;
                }
                pages.push(np.clone());
                cur = np;
            } else {
                break;
            }
        }

        if pages.is_empty() {
            return Err(ScrapeError::ParseError("masseffect2.in: no chapters".into()));
        }

        let mut chapters = Vec::new();
        for (i, p) in pages.iter().enumerate() {
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title: p.heading.clone(),
                content: p.body.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_matches() {
        let s = MassEffect2InScraper;
        assert!(s.can_handle("https://www.masseffect2.in/publ/19-1-0-1234"));
        assert!(!s.can_handle("https://www.fanfiction.net/s/123"));
    }

    #[test]
    fn parses_doc_id() {
        assert_eq!(MassEffect2InScraper::doc_id("https://www.masseffect2.in/publ/19-1-0-1234"), Some("19-1-0-1234".to_string()));
        assert_eq!(MassEffect2InScraper::doc_id("https://x.com/foo"), None);
    }

    #[test]
    fn common_prefix_works() {
        assert_eq!(common_prefix("Mass Effect: Story — Ch 1", "Mass Effect: Story — Ch 2"), "Mass Effect: Story — Ch ");
        assert_eq!(common_prefix("abc", "abd"), "ab");
        assert_eq!(common_prefix("abc", "xyz"), "");
    }

    #[test]
    fn parses_russian_dates() {
        assert!(parse_russian_date("12.04.2024, 15:30").is_some());
        assert!(parse_russian_date("12.04.2024").is_some());
        assert!(parse_russian_date("Сегодня").is_some());
        assert!(parse_russian_date("garbage").is_none());
    }
}
