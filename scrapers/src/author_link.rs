//! Author-link enrichment — after a scrape, make sure we have a profile URL
//! for the fic's author.
//!
//! Priority chain (cheapest first):
//!   1. The scraper already returned `author_url` → use it.
//!   2. Our own generic extraction: fetch the fic page and look for the
//!      most common author-profile link patterns (AO3 /users/, FFN /u/,
//!      XenForo /members/, RoyalRoad author pages).
//!   3. LLM fallback: if the generic pass finds nothing (new site, unusual
//!      markup), ask an LLM to extract the author profile URL from the
//!      page's visible text. Best-effort — never blocks the export path.
//!
//! All steps are best-effort: a failure returns `None` (the fic still
//! exports; the author page link just stays empty).

use async_trait::async_trait;

/// Minimal LLM interface the author-link fallback needs. Hosts implement
/// this for their model provider (e.g. Ollama); the crate never depends on
/// a specific provider.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Ask the model to produce JSON; returns the raw JSON text.
    async fn generate_json(&self, prompt: &str) -> Result<String, String>;
}

/// Generic author-profile URL patterns, keyed by URL fragment to avoid
/// false positives.
pub const AUTHOR_LINK_PATTERNS: &[&str] = &[
    // AO3: /users/{name} or /users/{name}/pseuds/{pseud}
    // (AO3 hrefs are RELATIVE — /users/... — so match the path too)
    "archiveofourown.org/users/",
    "/users/",
    "www.fanfiction.net/u/",
    "fanfiction.net/u/",
    // XenForo-style member profiles
    "/members/",
    // RoyalRoad
    "royalroadl.com/profile/",
    "www.royalroad.com/profile/",
    // FanFicFare source author pages
    "/profile/",
    "/author/",
];

/// Extract an author profile URL from scraped HTML text using our own
/// generic patterns. Returns the first match found in the page.
pub fn extract_author_url_from_html(page_text: &str, author_name: &str) -> Option<String> {
    let lower = page_text.to_lowercase();
    let name = author_name.to_lowercase();
    for pat in AUTHOR_LINK_PATTERNS {
        // Find the pattern, then read the URL it's part of
        if let Some(idx) = lower.find(pat) {
            // Walk backwards to the start of the href
            let start = lower[..idx].rfind("href=\"").map(|p| p + 6).unwrap_or(idx);
            let end = lower[idx..]
                .find('"')
                .map(|p| idx + p)
                .unwrap_or_else(|| lower.len());
            let url = &page_text[start..end];
            if !url.is_empty() {
                // Only accept when a part of the author name appears nearby
                // (within a few hundred chars) to avoid grabbing an
                // unrelated link. Multi-author fics: any of the creators is
                // a valid match.
                let nearby_start = start.saturating_sub(300);
                let nearby_end = (end + 300).min(page_text.len());
                let nearby = &lower[nearby_start..nearby_end];
                let name_parts: Vec<&str> = name.split(',').map(|p| p.trim()).collect();
                let name_matches = name_parts.iter().any(|p| p.len() >= 2 && nearby.contains(p));
                if name_matches {
                    return Some(url.to_string());
                }
            }
        }
    }
    None
}

/// Ask an LLM to extract the author profile URL from a page's text.
/// Returns a URL string on success (best-effort).
pub async fn llm_extract_author_url(
    llm: &dyn LlmClient,
    page_text: &str,
    author_name: &str,
) -> Option<String> {
    let prompt = format!(
        r#"You are extracting the author profile URL from a fanfiction page.
The story's author is "{author_name}".
Given the page text below, find the URL that links to this author's PROFILE page
(not the story itself, not a tag, not a comment link).

Rules:
- Return ONLY a JSON object: {{"author_url": "..."}} or {{"author_url": null}} if not found.
- The URL must be absolute (https://...) or start with "/".
- Do not invent URLs. Only use one that appears in the page text.

PAGE TEXT (first 6000 chars):
{page_text}"#,
        author_name = author_name,
        page_text = &page_text.chars().take(6000).collect::<String>(),
    );

    match llm.generate_json(&prompt).await {
        Ok(raw) => {
            let cleaned = raw.trim().trim_start_matches("```json").trim_end_matches("```").trim();
            match serde_json::from_str::<serde_json::Value>(cleaned) {
                Ok(v) => v.get("author_url").and_then(|u| u.as_str()).map(|s| s.to_string()),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

/// Enrich a scrape result with an author profile URL (best-effort chain).
///
/// `page_text` is the raw HTML of the fic page (already fetched by the
/// caller — we never fetch twice).
pub async fn ensure_author_url(
    llm: Option<&dyn LlmClient>,
    current_author_url: &str,
    author_name: &str,
    page_text: &str,
) -> Option<String> {
    // 1. Already have it from the scraper.
    if !current_author_url.is_empty() {
        return Some(current_author_url.to_string());
    }
    // 2. Own generic extraction.
    if let Some(url) = extract_author_url_from_html(page_text, author_name) {
        return Some(url);
    }
    // 3. LLM fallback (best-effort).
    if let Some(llm) = llm {
        return llm_extract_author_url(llm, page_text, author_name).await;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct FakeLlm {
        response: &'static str,
    }
    #[async_trait]
    impl LlmClient for FakeLlm {
        async fn generate_json(&self, _prompt: &str) -> Result<String, String> {
            Ok(self.response.to_string())
        }
    }

    #[test]
    fn extracts_ao3_author_url_from_html() {
        let html = r#"<div class="byline heading"><a rel="author" href="/users/Hermione_Granger">Hermione_Granger</a></div>"#;
        let url = extract_author_url_from_html(html, "Hermione_Granger");
        assert_eq!(url.as_deref(), Some("/users/Hermione_Granger"));
    }

    #[test]
    fn extracts_ffn_author_url() {
        let html = r#"<a href="https://www.fanfiction.net/u/1234567/AuthorName">AuthorName</a>"#;
        let url = extract_author_url_from_html(html, "AuthorName");
        assert_eq!(url.as_deref(), Some("https://www.fanfiction.net/u/1234567/AuthorName"));
    }

    #[test]
    fn ignores_unrelated_links() {
        let html = r#"<a href="/users/OtherAuthor">OtherAuthor</a> <a href="/works/999">the fic</a>"#;
        let url = extract_author_url_from_html(html, "MyAuthor");
        assert_eq!(url, None, "author name not near the link → no match");
    }

    #[tokio::test]
    async fn llm_fallback_uses_client() {
        let llm = FakeLlm {
            response: r#"{"author_url": "https://fanfics.example/profile-page/98765/silverquill"}"#,
        };
        let found = llm_extract_author_url(&llm, "<html>…</html>", "SilverQuill").await;
        assert_eq!(
            found.as_deref(),
            Some("https://fanfics.example/profile-page/98765/silverquill")
        );
    }

    #[tokio::test]
    async fn ensure_author_url_skips_llm_when_already_have_url() {
        let llm = FakeLlm { response: "{}" };
        let url = ensure_author_url(Some(&llm), "https://ao3.org/users/alice", "Alice", "<html/>").await;
        assert_eq!(url.as_deref(), Some("https://ao3.org/users/alice"));
    }
}
