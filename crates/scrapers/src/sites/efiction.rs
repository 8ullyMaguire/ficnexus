//! eFiction-family native adapter.
//!
//! eFiction is an old-school PHP fanfiction archive engine used by dozens
//! of niche fandom sites. FanFicFare covers 19 of them with per-site config
//! over a shared `BaseEfictionAdapter`. This port does the same: one generic
//! implementation driven by a per-domain config table.
//!
//! The sites share a common URL shape:
//!   `{protocol}://{domain}{archive_path}/{viewstory.php}?sid={storyId}`
//! with a `&chapter=N` param, an `&action=printable` print view, and an
//! `&index=1` TOC view. Metadata lives in a `<div class="infobox">` with
//! `<span class="label">Key:</span> value` pairs.

use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use scraper::{ElementRef, Html, Node, Selector};

use super::http;
use crate::{Chapter, FicMetadata, ScrapeError, SiteScraper};

/// Per-site configuration for an eFiction archive.
#[derive(Debug, Clone, Copy)]
pub struct EfictionSite {
    /// Canonical host, e.g. "dark-solace.org".
    pub domain: &'static str,
    /// Path prefix to the archive, e.g. "/elysian".
    pub archive_path: &'static str,
    /// View-story php filename, e.g. "viewstory.php".
    pub view_story: &'static str,
    /// View-user php filename, e.g. "viewuser.php".
    pub view_user: &'static str,
    /// strftime-style date format used by the site's "Published" values.
    pub date_format: &'static str,
    /// FFF-style site abbrev (used for url_ids), e.g. "dksl".
    pub abbrev: &'static str,
}

/// The 19 eFiction sites FanFicFare supports.
pub const EFICTION_SITES: &[EfictionSite] = &[
    EfictionSite {
        domain: "dark-solace.org",
        archive_path: "/elysian",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "dksl",
    },
    EfictionSite {
        domain: "gluttonyfiction.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "gltn",
    },
    EfictionSite {
        domain: "libraryofmoria.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "lmor",
    },
    EfictionSite {
        domain: "mttjustonce.net",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "mttj",
    },
    EfictionSite {
        domain: "mugglenetfanfiction.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "mugl",
    },
    EfictionSite {
        domain: "naiceanilme.net",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "naic",
    },
    EfictionSite {
        domain: "narutofic.org",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "nart",
    },
    EfictionSite {
        domain: "ncisfiction.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "ncis",
    },
    EfictionSite {
        domain: "ninelivesarchive.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "nla",
    },
    EfictionSite {
        domain: "sinfuldreams.com",
        archive_path: "/unicornfic",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "sdun",
    },
    EfictionSite {
        domain: "sinfuldreams.com",
        archive_path: "/wickedtemptation",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "sdwk",
    },
    EfictionSite {
        domain: "spikeluver.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "splv",
    },
    EfictionSite {
        domain: "starslibrary.net",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%d %b %Y",
        abbrev: "stlb",
    },
    EfictionSite {
        domain: "tgstorytime.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "tgst",
    },
    EfictionSite {
        domain: "thedelphicexpanse.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "delp",
    },
    EfictionSite {
        domain: "thehookupzone.net",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "hkup",
    },
    EfictionSite {
        domain: "valentchamber.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "valc",
    },
    EfictionSite {
        domain: "www.giantessworld.net",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "gsw",
    },
    EfictionSite {
        domain: "www.sunnydaleafterdark.com",
        archive_path: "/",
        view_story: "viewstory.php",
        view_user: "viewuser.php",
        date_format: "%B %d, %Y",
        abbrev: "sad",
    },
];

/// Config-driven eFiction scraper for one domain.
pub struct EfictionScraper {
    site: &'static EfictionSite,
}

impl EfictionScraper {
    /// Return the config for a URL, if it's an eFiction site we know.
    pub fn from_url(url: &str) -> Option<Self> {
        let site = EFICTION_SITES.iter().find(|s| url.contains(s.domain))?;
        Some(EfictionScraper { site })
    }

    pub fn all() -> Vec<EfictionScraper> {
        EFICTION_SITES
            .iter()
            .map(|s| EfictionScraper { site: s })
            .collect()
    }

    fn user_url(&self, user_id: &str) -> String {
        format!(
            "https://{}{}/{}{}",
            self.site.domain,
            self.site.archive_path,
            self.site.view_user,
            format!("?uid={user_id}")
        )
    }
}

impl EfictionScraper {
    /// Fetch one chapter's printable view and extract the <div class="chapter">.
    async fn fetch_chapter_text(&self, client: &reqwest::Client, chap_url: &str) -> String {
        match http::fetch(client, chap_url).await {
            Ok(html) => {
                let doc = Html::parse_document(&html);
                if let Ok(sel) = Selector::parse("div.chapter") {
                    if let Some(el) = doc.select(&sel).next() {
                        return el.inner_html();
                    }
                }
                if let Ok(body) = Selector::parse("body") {
                    if let Some(el) = doc.select(&body).next() {
                        return el.inner_html();
                    }
                }
                String::new()
            }
            Err(_) => String::new(),
        }
    }
}

#[async_trait]
impl SiteScraper for EfictionScraper {
    fn source_id(&self) -> i64 {
        3
    }

    fn can_handle(&self, url: &str) -> bool {
        let story = format!("{}?sid=", self.site.view_story);
        url.contains(self.site.domain) && (url.contains(story.as_str()) || url.contains("?sid="))
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let html = http::fetch(
            client,
            &format!("{url}&action=printable&textsize=0&chapter=1"),
        )
        .await?;
        let doc = Html::parse_document(&html);
        parse_metadata(&doc, self, url)
    }

    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let url = &meta.source;
        let toc_html = http::fetch(client, &format!("{url}&index=1")).await?;

        // Parse the TOC fully into owned data BEFORE any await: the parsed
        // document (scraper::Html) is not Send, so it must be dropped before
        // we enter the per-chapter fetch loop.
        let mut chapters_plan: Vec<(i32, String, String)> = Vec::new();
        {
            let toc = Html::parse_document(&toc_html);
            let link_sel = Selector::parse("a[href*='chapter=']")
                .map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let b_sel = Selector::parse("b").map_err(|e| ScrapeError::ParseError(e.to_string()))?;
            let mut seen = std::collections::HashSet::new();

            // eFiction TOC marks each chapter with <b>N.</b> followed by a link.
            // We collect (chapter_num, title, href) from the ordered <b>N</b>
            // markers, then fill chapter numbers from the link order.
            let mut toc_chapters: Vec<(i32, String, String)> = Vec::new();
            for b in toc.select(&b_sel) {
                let btext = b.text().collect::<String>();
                let chapter_num = btext.trim().trim_end_matches('.').parse::<i32>().ok();
                if chapter_num.is_none() {
                    continue;
                }
                // The chapter link is the next <a> after this <b> (possibly
                // separated by <br>). Collect from the parent's direct children.
                if let Some(parent) = b.parent() {
                    let parent = ElementRef::wrap(parent).unwrap();
                    let mut capture = false;
                    for child in parent.children() {
                        if let Some(cref) = ElementRef::wrap(child) {
                            if cref.value().name() == "b" {
                                // Start capturing after the matching <b>.
                                capture = cref.text().collect::<String>().trim() == btext.trim();
                                continue;
                            }
                            if capture && cref.value().name() == "a" {
                                let href = cref.value().attr("href").unwrap_or("");
                                if href.contains("chapter=") {
                                    let title = cref.text().collect::<String>().trim().to_string();
                                    toc_chapters.push((
                                        chapter_num.unwrap_or(1),
                                        title,
                                        href.to_string(),
                                    ));
                                    capture = false;
                                }
                            }
                        }
                    }
                }
            }

            for (num, title, href) in toc_chapters {
                if seen.contains(&href) {
                    continue;
                }
                seen.insert(href.clone());
                chapters_plan.push((num, title, href));
            }

            // Fallback: list all chapter links.
            if chapters_plan.is_empty() {
                for a in toc.select(&link_sel) {
                    let href = a.value().attr("href").unwrap_or("").to_string();
                    if href.contains("chapter=") && !seen.contains(&href) {
                        seen.insert(href.clone());
                        let title = a.text().collect::<String>().trim().to_string();
                        chapters_plan.push((chapters_plan.len() as i32 + 1, title, href));
                    }
                }
            }
        } // toc dropped here

        if chapters_plan.is_empty() {
            return Err(ScrapeError::ParseError("no chapters found".into()));
        }

        let mut chapters = Vec::new();
        for (num, title, _href) in chapters_plan {
            let chap_url = format!("{url}&chapter={num}&action=printable");
            let content = self.fetch_chapter_text(client, &chap_url).await;
            chapters.push(Chapter {
                chapter_id: num,
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
        // eFiction metadata (characters, ships, warnings) is already in the
        // lookup metadata; nothing extra to tag from the body.
        Ok(vec![])
    }
}

/// Parse the eFiction printable view into metadata.
fn parse_metadata(
    doc: &Html,
    scraper: &EfictionScraper,
    url: &str,
) -> Result<FicMetadata, ScrapeError> {
    // Title + author from #pagetitle: first <a> = title, second = author.
    let mut title = String::new();
    let mut author = String::new();
    let mut author_url = String::new();
    let mut author_local_id = String::new();
    if let Ok(sel) = Selector::parse("#pagetitle a") {
        let links: Vec<_> = doc.select(&sel).collect();
        if !links.is_empty() {
            title = links[0].text().collect::<String>().trim().to_string();
        }
        if links.len() > 1 {
            author = links[1].text().collect::<String>().trim().to_string();
            if let Some(href) = links[1].value().attr("href") {
                if let Some(id) = href.rsplit(['=', '/']).next() {
                    if id.chars().all(|c| c.is_ascii_digit()) {
                        author_local_id = id.to_string();
                    }
                }
                author_url = scraper.user_url(&author_local_id);
            }
        }
    }
    if title.is_empty() {
        return Err(ScrapeError::ParseError("eFiction: no title found".into()));
    }

    // Story id from the URL (?sid=NNN).
    let story_id = url
        .split("?sid=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or("")
        .to_string();

    let mut meta = FicMetadata {
        url_id: format!("{}_{}", scraper.site.abbrev, story_id),
        title,
        author,
        chapters: 0,
        words: 0,
        desc: String::new(),
        published: 0,
        updated: 0,
        status: "ongoing".into(),
        source: url.to_string(),
        source_id: story_id.parse().unwrap_or(0),
        author_id: 0,
        author_url,
        author_local_id,
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    };

    if let Ok(label_sel) = Selector::parse(".infobox .content span.label") {
        for label in doc.select(&label_sel) {
            let key = label.text().collect::<String>();
            let key = key.trim().trim_end_matches(':').trim().to_string();
            let mut value = String::new();
            let mut nxt = label.next_sibling();
            while let Some(node) = nxt {
                if let Some(el) = ElementRef::wrap(node) {
                    if el.value().name() == "span"
                        && el
                            .value()
                            .attr("class")
                            .map(|c| c.contains("label"))
                            .unwrap_or(false)
                    {
                        break;
                    }
                    value.push_str(&el.inner_html());
                } else if let Node::Text(t) = node.value() {
                    value.push_str(t);
                } else if let Node::Comment(c) = node.value() {
                    value.push_str(c);
                }
                nxt = node.next_sibling();
            }
            let value = strip_html(&value);
            apply_metadata(&mut meta, &key, &value);
        }
    }

    Ok(meta)
}

/// Apply one eFiction metadata pair to the FicMetadata (mirrors FFF's
/// handleMetadataPair logic).
fn apply_metadata(meta: &mut FicMetadata, key: &str, value: &str) {
    if value.is_empty() || value == "None" {
        return;
    }
    match key {
        "Summary" => meta.desc = value.to_string(),
        "Rating" | "Rated" => {
            // Ratings don't have a FicMetadata slot; fold into extra_meta.
            let mut extra = meta.extra_meta.take().unwrap_or_default();
            extra.push_str(&format!("rating={};", value));
            meta.extra_meta = Some(extra);
        }
        "Word count" => {
            let n: i64 = value
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0);
            meta.words = n;
        }
        "Completed" => {
            meta.status = if value.contains("Yes")
                || value.contains("Completed")
                || value.contains("Ja")
                || value.contains("Igen")
            {
                "complete".into()
            } else {
                "ongoing".into()
            };
        }
        "Published" => {
            meta.published = parse_common_date(value).unwrap_or(0);
        }
        "Updated" => {
            meta.updated = parse_common_date(value).unwrap_or(0);
        }
        "Category" | "Categories" | "Characters" | "Pairing" | "Ships" | "Warning" | "Warnings"
        | "Genre" | "Fandoms" => {
            // Fold into extra_meta (the metadata model is intentionally
            // generic; hosts can split on the key themselves).
            let mut extra = meta.extra_meta.take().unwrap_or_default();
            extra.push_str(&format!("{key}={value};"));
            meta.extra_meta = Some(extra);
        }
        _ => {
            let mut extra = meta.extra_meta.take().unwrap_or_default();
            extra.push_str(&format!("{key}={value};"));
            meta.extra_meta = Some(extra);
        }
    }
}

/// Parse a date in any of the common eFiction formats → Unix millis.
fn parse_common_date(value: &str) -> Option<i64> {
    let v = value.trim();
    let d = NaiveDate::parse_from_str(v, "%B %d, %Y")
        .or_else(|_| NaiveDate::parse_from_str(v, "%d %b %Y"))
        .ok()?;
    let dt = d.and_hms_opt(0, 0, 0)?;
    Some(Utc.from_utc_datetime(&dt).timestamp_millis())
}

/// Very small HTML stripper.
fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_all_19_domains() {
        assert_eq!(EFICTION_SITES.len(), 19);
    }

    #[test]
    fn from_url_matches_domains() {
        assert!(
            EfictionScraper::from_url("https://dark-solace.org/elysian/viewstory.php?sid=123")
                .is_some()
        );
        assert!(
            EfictionScraper::from_url("https://starslibrary.net/viewstory.php?sid=42").is_some()
        );
        assert!(
            EfictionScraper::from_url("https://not-efiction.com/viewstory.php?sid=1").is_none()
        );
    }

    #[test]
    fn parses_dates() {
        assert_eq!(parse_common_date("February 14, 2024"), Some(1707868800000));
        assert!(parse_common_date("14 Feb 2024").is_some());
    }

    #[test]
    fn strips_html() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn applies_metadata() {
        let mut meta = FicMetadata {
            url_id: "x_1".into(),
            title: "t".into(),
            author: "a".into(),
            chapters: 0,
            words: 0,
            desc: String::new(),
            published: 0,
            updated: 0,
            status: "ongoing".into(),
            source: "u".into(),
            source_id: 0,
            author_id: 0,
            author_url: String::new(),
            author_local_id: String::new(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        };
        apply_metadata(&mut meta, "Word count", "12,345");
        assert_eq!(meta.words, 12345);
        apply_metadata(&mut meta, "Completed", "Yes");
        assert_eq!(meta.status, "complete");
        apply_metadata(&mut meta, "Characters", "Harry Potter, Hermione Granger");
        assert!(
            meta.extra_meta
                .as_deref()
                .unwrap_or("")
                .contains("Harry Potter")
        );
    }
}
