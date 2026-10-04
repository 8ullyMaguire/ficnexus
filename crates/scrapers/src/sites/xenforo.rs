/// XenForo forum scraper (SpaceBattles, SufficientVelocity, QuestionableQuesting)
pub struct XenForoScraper;

use crate::{Chapter, FicMetadata, ScrapeError, SiteCredentials, SiteScraper};
use async_trait::async_trait;
use chrono::Utc;
use scraper::{Html, Selector};

// Domains that use XenForo
const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
    "boards.theforce.net",
    // Fiction.live is XenForo-based; the native adapter also skips empty
    // chapters (upstream FFF #1288: empty "Home" chapter breaks updates).
    "fiction.live",
    // XenForo2 family (FFF base_xenforo2forum_adapter).
    "www.alternatehistory.com",
    "althistory.com",
    "www.the-sietch.com",
];

/// QQ Creative Writing forum node IDs (used for author work discovery).
const QQ_CW_NODES: &[&str] = &["19", "28", "29", "30"];

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

impl XenForoScraper {
    fn extract_thread_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    pub fn is_xenforo_url(url: &str) -> bool {
        XENFORO_DOMAINS.iter().any(|d| url.contains(d))
    }

    /// Check if a URL is a XenForo member profile page.
    /// Matches: `/members/username.12345/` or `/members/username.12345`
    pub fn is_author_page(url: &str) -> bool {
        let re = regex_lite::Regex::new(r"/members/[^/]+\.\d+/?").ok();
        re.map(|r| r.is_match(url)).unwrap_or(false)
    }

    /// Extract the numeric user ID from a member profile URL.
    /// e.g. `https://forum.questionablequesting.com/members/gothicjedi666.22859/` → `22859`
    pub fn extract_user_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/members/[^/]+\.(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Extract the username from a member profile URL.
    /// e.g. `https://forum.questionablequesting.com/members/gothicjedi666.22859/` → `gothicjedi666`
    pub fn extract_user_name(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/members/([^.]+)\.\d+").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Get the base URL (scheme + host) from any URL on this domain.
    fn base_url(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"(https?://[^/]+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    /// Login via XenForo AJAX endpoint. Requires the client to have a
    /// cookie store so session cookies persist for subsequent requests.
    async fn xf_login(
        client: &reqwest::Client,
        base: &str,
        creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        // 1. Fetch login page for CSRF token
        let login_page = client
            .get(format!("{base}/login/"))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = login_page
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let token_re = regex_lite::Regex::new(r#"_xfToken.*?value="([^"]+)"#)
            .map_err(|e| ScrapeError::Internal(format!("regex compile error: {e}")))?;
        let token = token_re
            .captures(&html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| ScrapeError::ParseError("could not extract CSRF token".into()))?;

        // 2. POST to AJAX login endpoint
        let params = [
            ("login", creds.username.as_str()),
            ("password", creds.password.as_str()),
            ("_xfToken", token),
            ("remember", "1"),
        ];

        client
            .post(format!("{base}/login/login"))
            .header("User-Agent", USER_AGENT)
            .header("X-Requested-With", "XMLHttpRequest")
            .form(&params)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        Ok(())
    }

    /// List all thread URLs started by the given author on a XenForo forum.
    ///
    /// Uses the XenForo search with user filter, then verifies authorship
    /// by checking each thread's first post author. This is necessary because
    /// XenForo search returns threads where the user *posted*, not just
    /// threads they *started*.
    ///
    /// `creds` provides optional login credentials for NSFW-gated forums.
    pub async fn list_author_works(
        client: &reqwest::Client,
        profile_url: &str,
        creds: &[SiteCredentials],
    ) -> Result<Vec<String>, ScrapeError> {
        let base = Self::base_url(profile_url)
            .ok_or_else(|| ScrapeError::ParseError("cannot extract base URL".into()))?;
        let user_id = Self::extract_user_id(profile_url)
            .ok_or_else(|| ScrapeError::ParseError("cannot extract user ID".into()))?;
        let username = Self::extract_user_name(profile_url)
            .ok_or_else(|| ScrapeError::ParseError("cannot extract username".into()))?;

        // Login if credentials are provided
        if let Some(cred) = creds.iter().find(|c| {
            base.contains(&c.domain)
                || c.domain.contains(
                    &base
                        .split("://")
                        .nth(1)
                        .unwrap_or("")
                        .split('/')
                        .next()
                        .unwrap_or(""),
                )
        }) {
            Self::xf_login(client, &base, cred).await?;
            // Small delay after login
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // Create a XenForo search session
        // POST to /search/search with user filter + CW node filter
        let search_token = Self::get_csrf_token(client, &base).await?;

        let mut form = std::collections::HashMap::new();
        form.insert("keywords", "");
        form.insert("t", "post");
        form.insert("c[content]", "thread");
        form.insert("c[users][]", &username);
        form.insert("order", "word_count");
        form.insert("_xfToken", &search_token);

        // Add node filters for QQ — must be sent as repeated keys
        let mut body_parts: Vec<String> = form
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect();

        if base.contains("questionablequesting") {
            for node in QQ_CW_NODES {
                body_parts.push(format!("c[nodes][]={}", urlencoding::encode(node)));
            }
        }

        let body = body_parts.join("&");

        let search_resp = client
            .post(format!("{base}/search/search"))
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let search_url = search_resp.url().to_string();

        // Extract search ID from redirect URL
        let sid_re = regex_lite::Regex::new(r"/search/(\d+)/")
            .map_err(|e| ScrapeError::Internal(format!("regex error: {e}")))?;
        let sid = sid_re
            .captures(&search_url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| ScrapeError::ParseError("could not extract search ID".into()))?;

        // Paginate through search results
        let mut all_thread_paths: Vec<String> = Vec::new();
        let mut page = 1u32;

        loop {
            let page_url = format!(
                "{base}/search/{sid}/?page={page}&q=%2A&c[users][0]={username}&o=word_count"
            );

            let page_resp = client
                .get(&page_url)
                .header("User-Agent", USER_AGENT)
                .send()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;

            let html = page_resp
                .text()
                .await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;

            // Extract thread paths
            let link_re = regex_lite::Regex::new(r#"href="(/threads/[^"]+)""#)
                .map_err(|e| ScrapeError::Internal(format!("regex error: {e}")))?;

            let mut page_threads = std::collections::HashSet::new();
            for cap in link_re.captures_iter(&html) {
                let raw = &cap[1];
                // Clean: remove /post-N suffix and /threadmarks suffix
                let cleaned = raw
                    .split("/post-")
                    .next()
                    .unwrap_or(raw)
                    .split("/threadmarks")
                    .next()
                    .unwrap_or(raw)
                    .trim_end_matches('/')
                    .to_string();
                if cleaned.starts_with("/threads/") {
                    page_threads.insert(cleaned);
                }
            }

            let before = all_thread_paths.len();
            for t in page_threads {
                if !all_thread_paths.contains(&t) {
                    all_thread_paths.push(t);
                }
            }

            // Check for next page
            let has_next = html.contains("pageNav-jump--next");
            if !has_next || all_thread_paths.len() == before {
                break;
            }

            page += 1;
            if page > 100 {
                break; // safety limit
            }

            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }

        tracing::info!(
            "XenForo author search: {} threads found for {username}",
            all_thread_paths.len()
        );

        // Verify authorship: fetch each thread and check first post author
        let mut verified = Vec::new();
        let uid_re = regex_lite::Regex::new(
            r#"<article[^>]*class="message[^"]*message--post[^"]*"[^>]*>.*?data-user-id="(\d+)""#,
        )
        .map_err(|e| ScrapeError::Internal(format!("regex error: {e}")))?;

        for path in &all_thread_paths {
            let thread_url = format!("{base}{path}/");
            let resp = client
                .get(&thread_url)
                .header("User-Agent", USER_AGENT)
                .send()
                .await;

            let html = match resp {
                Ok(r) => r.text().await.unwrap_or_default(),
                Err(_) => continue,
            };

            // Check if the first post's author matches our target user
            if let Some(caps) = uid_re.captures(&html) {
                if &caps[1] == user_id {
                    verified.push(format!("{base}{path}/"));
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        tracing::info!(
            "XenForo author verification: {}/{} threads are started by {username}",
            verified.len(),
            all_thread_paths.len()
        );

        Ok(verified)
    }

    /// Get CSRF token from a XenForo page.
    async fn get_csrf_token(client: &reqwest::Client, base: &str) -> Result<String, ScrapeError> {
        let resp = client
            .get(format!("{base}/login/"))
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = resp
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let re = regex_lite::Regex::new(r#"_xfToken.*?value="([^"]+)"#)
            .map_err(|e| ScrapeError::Internal(format!("regex error: {e}")))?;

        re.captures(&html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| ScrapeError::ParseError("could not extract CSRF token".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_handle_theforce_net() {
        let s = XenForoScraper;
        let url = "https://boards.theforce.net/threads/kyp-durron-skiing-triathlon.50062326/";
        assert!(s.can_handle(url));
        assert_eq!(
            XenForoScraper::extract_thread_id(url).as_deref(),
            Some("50062326")
        );
    }

    #[test]
    fn can_handle_existing_xenforo_sites() {
        let s = XenForoScraper;
        for url in [
            "https://forums.spacebattles.com/threads/foo.123456/",
            "https://forums.sufficientvelocity.com/threads/bar.789/",
            "https://forum.questionablequesting.com/threads/baz.1/",
        ] {
            assert!(s.can_handle(url), "should handle {url}");
        }
        assert!(!s.can_handle("https://archiveofourown.org/works/123"));
    }

    #[test]
    fn is_author_page_detects_member_urls() {
        assert!(XenForoScraper::is_author_page(
            "https://forum.questionablequesting.com/members/gothicjedi666.22859/"
        ));
        assert!(XenForoScraper::is_author_page(
            "https://forums.spacebattles.com/members/author-name.12345"
        ));
        assert!(!XenForoScraper::is_author_page(
            "https://forum.questionablequesting.com/threads/baz.1/"
        ));
        assert!(!XenForoScraper::is_author_page(
            "https://archiveofourown.org/users/someone"
        ));
    }

    #[test]
    fn extract_user_id_from_member_url() {
        assert_eq!(
            XenForoScraper::extract_user_id(
                "https://forum.questionablequesting.com/members/gothicjedi666.22859/"
            )
            .as_deref(),
            Some("22859")
        );
        assert_eq!(
            XenForoScraper::extract_user_id(
                "https://forums.spacebattles.com/members/author-name.12345"
            )
            .as_deref(),
            Some("12345")
        );
        assert_eq!(
            XenForoScraper::extract_user_id(
                "https://forum.questionablequesting.com/threads/baz.1/"
            )
            .as_deref(),
            None
        );
    }

    #[test]
    fn extract_user_name_from_member_url() {
        assert_eq!(
            XenForoScraper::extract_user_name(
                "https://forum.questionablequesting.com/members/gothicjedi666.22859/"
            )
            .as_deref(),
            Some("gothicjedi666")
        );
        assert_eq!(
            XenForoScraper::extract_user_name(
                "https://forums.spacebattles.com/members/author-name.12345"
            )
            .as_deref(),
            Some("author-name")
        );
    }

    #[test]
    fn is_xenforo_domain_check() {
        assert!(XenForoScraper::is_xenforo_url(
            "https://forum.questionablequesting.com/threads/foo.1/"
        ));
        assert!(XenForoScraper::is_xenforo_url(
            "https://forums.spacebattles.com/threads/bar.2/"
        ));
        assert!(!XenForoScraper::is_xenforo_url(
            "https://archiveofourown.org/works/123"
        ));
    }
}

#[async_trait]
impl SiteScraper for XenForoScraper {
    fn can_handle(&self, url: &str) -> bool {
        Self::is_xenforo_url(url)
    }

    async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> Result<FicMetadata, ScrapeError> {
        let thread_id = Self::extract_thread_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract thread ID".into()))?;

        let response = client
            .get(url)
            .header("User-Agent", USER_AGENT)
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
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1.p-title-value").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author_el = document
            .select(&Selector::parse("a.username").unwrap())
            .next();
        let author = author_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let author_url = author_el
            .and_then(|el| el.value().attr("href"))
            .map(|h| {
                if h.starts_with('/') {
                    // Determine domain from the URL
                    for domain in XENFORO_DOMAINS {
                        if url.contains(domain) {
                            return format!("https://{domain}{h}");
                        }
                    }
                }
                h.to_string()
            })
            .unwrap_or_default();

        let description = document
            .select(&Selector::parse("article.message-body").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        let url_id = crate::generate_url_id(3, &thread_id);
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
            source_id: 3,
            author_id: 0,
            author_url,
            author_local_id: thread_id,
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
        let mut chapters = Vec::new();

        // For XenForo, we fetch the thread page by page
        // The first post is the story content
        let url = &meta.source;
        let response = client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response
            .text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let article_sel = Selector::parse("article.message-body").unwrap();
        let mut chapter_idx = 0;

        for article in document.select(&article_sel) {
            chapter_idx += 1;
            let content = article.inner_html();
            // Skip empty posts — Fiction.live has an empty "Home" chapter
            // that would otherwise break update detection (upstream FFF
            // #1288). Trim tags/whitespace to test for real content.
            let text_only = article.text().collect::<String>();
            if text_only.trim().is_empty() {
                continue;
            }
            let title = if chapter_idx == 1 {
                meta.title.clone()
            } else {
                format!("Chapter {chapter_idx}: Thread Page {chapter_idx}")
            };

            chapters.push(Chapter {
                chapter_id: chapter_idx,
                title,
                content,
            });
        }

        if chapters.is_empty() {
            return Err(ScrapeError::ParseError(
                "no content found in XenForo thread".into(),
            ));
        }

        Ok(chapters)
    }

    async fn list_author_works(
        &self,
        client: &reqwest::Client,
        profile_url: &str,
        creds: &[SiteCredentials],
    ) -> Result<Vec<String>, ScrapeError> {
        Self::list_author_works(client, profile_url, creds).await
    }
}
