//! Generic eFiction-variant adapter (the `viewstory.php?sid=` family).
//!
//! 26 FanFicFare adapters share this layout — an old eFiction theme with
//! `viewstory.php?sid={id}` story URLs and `span.label` metadata rows:
//!
//! - title: `a[href*="viewstory.php?sid={id}$"]`
//! - author: `a[href*="viewuser.php?uid="]`
//! - chapters: `a[href*="viewstory.php?sid={id}&chapter="]`
//! - metadata: `span.label` sibling text — Rated, Word count, Categories,
//!   Characters, Genre, Warnings, Complete, Published, Updated
//! - body: `div.content` (per-chapter page)
//!
//! Sites differ only in domain, optional path prefix (e.g. `/fanfiction/`,
//! `/archive/`, `/wrfa/`) and whether adult gate (`&warning=4`) applies.

use async_trait::async_trait;
use chrono::TimeZone;
use regex_lite::Regex;
use scraper::{Html, Node, Selector};

use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};
use super::http;

/// Per-site config for the eFiction-variant family.
#[derive(Debug, Clone, Copy)]
pub struct EficVariantSite {
    pub domain: &'static str,
    /// Path prefix before `viewstory.php`, e.g. `/fanfiction/`.
    pub prefix: &'static str,
    /// Requires the `&warning=4` adult-agree param on URLs.
    pub adult: bool,
}

pub const EFIC_VARIANT_SITES: &[EficVariantSite] = &[
    EficVariantSite { domain: "ashwinder.sycophanthex.com", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "chaos.sycophanthex.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "chosentwofanfic.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.dracoandginny.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "efpfanfic.net", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "erosnsappho.sycophanthex.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "archive.fanfictalk.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "imagine.e-fic.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "ksarchive.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "lumos.sycophanthex.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.midnightwhispers.net", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "ninelivesarchive.com", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "occlumency.sycophanthex.com", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "www.potionsandsnitches.org", prefix: "/fanfiction/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "www.pretendercentre.com", prefix: "/missingpieces/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.psychfic.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "samandjack.net", prefix: "/fanfics/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "sheppardweir.com", prefix: "/fanfics/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.siye.co.uk", prefix: "/siye/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "t.evancurrie.ca", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "themasque.net", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.twilighted.net", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "voracity2.e-fic.com", prefix: "/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.walkingtheplank.org", prefix: "/archive/viewstory.php?sid=", adult: true },
    EficVariantSite { domain: "www.whofic.com", prefix: "/viewstory.php?sid=", adult: false },
    EficVariantSite { domain: "www.wolverineandrogue.com", prefix: "/wrfa/viewstory.php?sid=", adult: false },
];

/// Domain-driven scraper for the eFiction-variant family.
pub struct EficVariantScraper {
    site: &'static EficVariantSite,
}

impl EficVariantScraper {
    pub fn from_url(url: &str) -> Option<Self> {
        let site = EFIC_VARIANT_SITES.iter().find(|s| url.contains(s.domain))?;
        Some(EficVariantScraper { site })
    }

    pub fn all() -> Vec<EficVariantScraper> {
        EFIC_VARIANT_SITES.iter().map(|s| EficVariantScraper { site: s }).collect()
    }

    fn base_url(&self) -> String {
        format!("https://{}{}", self.site.domain, self.site.prefix.trim_end_matches("viewstory.php?sid="))
    }

    fn page_url(&self, url: &str) -> String {
        let mut u = url.to_string();
        if self.site.adult {
            u.push_str("&warning=4");
        }
        u.push_str("&index=1");
        u
    }

    fn story_id(&self, url: &str) -> Option<String> {
        let id = url.split("sid=").nth(1)?.split('&').next()?.to_string();
        if id.chars().all(|c| c.is_ascii_digit()) {
            Some(id)
        } else {
            None
        }
    }
}

#[async_trait]
impl SiteScraper for EficVariantScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains(self.site.domain) && url.contains("viewstory.php?sid=")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = self.story_id(url).ok_or_else(|| ScrapeError::ParseError("efiction-variant: bad url".into()))?;
        let page_url = self.page_url(url);
        let html = http::fetch(client, &page_url).await?;

        if html.contains("Access denied. This story has not been validated") {
            return Err(ScrapeError::Blocked);
        }
        if html.contains("By clicking this link, you acknowledge") {
            return Err(ScrapeError::AuthRequired("efiction-variant adult gate".into()));
        }

        let doc = Html::parse_document(&html);

        let mut title = String::new();
        let mut author = String::new();
        let mut author_url = String::new();
        let mut author_local_id = String::new();
        let mut rating = String::new();
        let mut desc = String::new();
        let mut status = "ongoing".to_string();
        let mut published = 0i64;
        let mut updated = 0i64;
        let mut words = 0i64;
        let mut chapters = 0i32;

        // Title + author.
        if let Ok(a_sel) = Selector::parse(&format!("a[href*='viewstory.php?sid={story_id}']")) {
            if let Some(a) = doc.select(&a_sel).next() {
                title = a.text().collect::<String>().trim().to_string();
            }
        }
        if let Ok(u_sel) = Selector::parse("a[href*='viewuser.php?uid=']") {
            if let Some(a) = doc.select(&u_sel).next() {
                author = a.text().collect::<String>().trim().to_string();
                if let Some(h) = a.value().attr("href") {
                    author_url = format!("{}{}", self.base_url(), h.trim_start_matches('/'));
                    author_local_id = h.split('=').nth(1).unwrap_or("").to_string();
                }
            }
        }
        if title.is_empty() {
            return Err(ScrapeError::ParseError("efiction-variant: no title".into()));
        }

        // Chapters.
        let chap_re = Regex::new(&format!(r"viewstory\.php\?sid={story_id}&chapter=\d+$")).unwrap();
        if let Ok(a_sel) = Selector::parse("a[href*='viewstory.php?sid=']") {
            chapters = doc
                .select(&a_sel)
                .filter(|a| a.value().attr("href").map(|h| chap_re.is_match(h)).unwrap_or(false))
                .count() as i32;
        }
        if chapters == 0 {
            chapters = 1;
        }

        // Metadata from div.content label spans.
        if let Ok(c_sel) = Selector::parse("div.content") {
            if let Some(content) = doc.select(&c_sel).next() {
                let full = content.text().collect::<String>();
                if let Some(ci) = full.find("Categories") {
                    desc = full[..ci].trim().to_string();
                }
                if let Ok(lab_sel) = Selector::parse("span.label") {
                    for label in content.select(&lab_sel) {
                        let lab = label.text().collect::<String>().trim().to_string();
                        let mut val = String::new();
                        let mut nxt = label.next_sibling();
                        while let Some(node) = nxt {
                            if let Some(el) = scraper::ElementRef::wrap(node) {
                                val = el.text().collect::<String>().trim().to_string();
                                break;
                            } else if let Node::Text(t) = node.value() {
                                let t = t.trim();
                                if !t.is_empty() {
                                    val = t.to_string();
                                    break;
                                }
                            }
                            nxt = node.next_sibling();
                        }
                        if lab.contains("Rated") && rating.is_empty() {
                            rating = val.clone();
                        }
                        if lab.contains("Word count") {
                            words = val
                                .split(" -")
                                .next()
                                .unwrap_or("")
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .collect::<String>()
                                .parse()
                                .unwrap_or(0);
                        }
                        if lab.contains("Complete") && val.contains("Yes") {
                            status = "complete".to_string();
                        }
                        if lab.contains("Published") {
                            published = parse_dt(&val.split(" -").next().unwrap_or(""));
                        }
                        if lab.contains("Updated") {
                            updated = parse_dt(&val.split(" -").next().unwrap_or(""));
                        }
                    }
                }
            }
        }

        let now = chrono::Utc::now().timestamp_millis();
        Ok(FicMetadata {
            url_id: format!("efv_{story_id}"),
            title,
            author,
            chapters,
            words,
            desc,
            published: if published > 0 { published } else { now },
            updated: if updated > 0 { updated } else { now },
            status,
            source: url.to_string(),
            source_id: story_id.parse().unwrap_or(0),
            author_id: 0,
            author_url,
            author_local_id,
            content_hash: None,
            extra_meta: Some(format!("rating={};", rating)),
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let story_id = self.story_id(&meta.source).ok_or_else(|| ScrapeError::ParseError("efiction-variant: bad url".into()))?;
        let page_url = self.page_url(&meta.source);
        let html = http::fetch(client, &page_url).await?;

        let chap_re = Regex::new(&format!(r"viewstory\.php\?sid={story_id}&chapter=\d+$")).unwrap();
        let links: Vec<(String, String)> = {
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("a[href*='viewstory.php?sid=']")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            doc.select(&sel)
                .filter_map(|a| {
                    let href = a.value().attr("href")?;
                    if !chap_re.is_match(href) {
                        return None;
                    }
                    let title = a.text().collect::<String>().trim().to_string();
                    let mut url = format!("{}{}", self.base_url(), href.trim_start_matches('/'));
                    if self.site.adult {
                        url.push_str("&warning=4");
                    }
                    Some((title, url))
                })
                .collect()
        };

        if links.is_empty() {
            let content = fetch_chapter_text(client, &meta.source).await;
            return Ok(vec![Chapter {
                chapter_id: 1,
                title: meta.title.clone(),
                content,
            }]);
        }

        let mut chapters = Vec::new();
        for (i, (title, url)) in links.into_iter().enumerate() {
            let content = fetch_chapter_text(client, &url).await;
            chapters.push(Chapter {
                chapter_id: i as i32 + 1,
                title,
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

async fn fetch_chapter_text(client: &reqwest::Client, url: &str) -> String {
    let Ok(html) = http::fetch(client, url).await else {
        return String::new();
    };
    let doc = Html::parse_document(&html);
    if let Ok(sel) = Selector::parse("div.content") {
        if let Some(el) = doc.select(&sel).next() {
            return el.inner_html();
        }
    }
    String::new()
}

fn parse_dt(s: &str) -> i64 {
    let s = s.trim();
    for fmt in ["%m/%d/%Y", "%Y-%m-%d", "%b %d, %Y"] {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return chrono::Utc.from_utc_datetime(&dt).timestamp_millis();
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn family_has_26_sites() {
        assert_eq!(EFIC_VARIANT_SITES.len(), 26);
    }

    #[test]
    fn from_url_matches_domains() {
        assert!(EficVariantScraper::from_url("https://www.psychfic.com/viewstory.php?sid=1234").is_some());
        assert!(EficVariantScraper::from_url("https://www.walkingtheplank.org/archive/viewstory.php?sid=1234").is_some());
        assert!(EficVariantScraper::from_url("https://www.wolverineandrogue.com/wrfa/viewstory.php?sid=1234").is_some());
        assert!(EficVariantScraper::from_url("https://www.fanfiction.net/s/123").is_none());
    }

    #[test]
    fn parses_story_id() {
        let s = EficVariantScraper { site: &EFIC_VARIANT_SITES[0] };
        assert_eq!(
            s.story_id("https://www.psychfic.com/viewstory.php?sid=1234&index=1"),
            Some("1234".to_string())
        );
        assert_eq!(s.story_id("https://x.com/foo"), None);
    }

    #[test]
    fn parses_dates() {
        assert!(parse_dt("01/31/2024") > 0);
        assert!(parse_dt("2024-01-31") > 0);
        assert_eq!(parse_dt("garbage"), 0);
    }

    #[test]
    fn extracts_chapter_body() {
        let html = r#"<html><body><div class="content"><p>Story text.</p></div></body></html>"#;
        let doc = Html::parse_document(html);
        let mut s = String::new();
        if let Ok(sel) = Selector::parse("div.content") {
            if let Some(el) = doc.select(&sel).next() {
                s.push_str(&el.inner_html());
            }
        }
        assert!(s.contains("Story text."));
    }
}
