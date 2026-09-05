//! # fanfic-scrapers
//!
//! General-purpose fanfiction scraping library. Provides:
//!
//! - a shared, site-agnostic metadata model ([`FicMetadata`], [`Chapter`])
//! - the [`SiteScraper`] adapter trait for pluggable site backends
//! - a [`Registry`] that routes URLs to the right adapter (native scrapers
//!   preferred, FanFicFare as a catch-all)
//! - native adapters for AO3, FanFiction.net, FictionPress, RoyalRoad,
//!   XenForo boards, HPFanficArchive, and adultfanfiction.org
//! - an optional LLM-assisted author-profile-link extractor
//!   ([`author_link`])
//!
//! The crate is deliberately free of any host application's types: scrapers
//! return [`FicMetadata`] and the host decides what to do with it. Failure
//! recording (e.g. a self-healing hook) is injected via [`HealHook`].

pub mod author_link;
pub mod cloudflare;
pub mod fichub_net;
pub mod registry;
pub mod sites;

pub use registry::Registry;

use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A single extracted tag (freeform/category/fandom/character/...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i32,
    pub score: f64,
}

impl ExtractedTag {
    pub fn category(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 6,
            score: 0.0,
        }
    }
    pub fn fandom(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 1,
            score: 0.0,
        }
    }
    pub fn character(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 2,
            score: 0.0,
        }
    }
    pub fn character_main(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 2,
            score: 1.0,
        }
    }
    pub fn relationship(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 3,
            score: 0.0,
        }
    }
    pub fn relationship_primary(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 3,
            score: 1.0,
        }
    }
    pub fn warning(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 5,
            score: 0.0,
        }
    }
    pub fn freeform(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tag_type_id: 4,
            score: 0.0,
        }
    }
}

/// Metadata scraped from a fanfiction site. Site-agnostic: every adapter
/// fills this; hosts map it into their own storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    /// Stable, site-local identifier (e.g. `ao3_21845264`). The host
    /// usually derives its cache key from this.
    pub url_id: String,
    pub title: String,
    /// Comma-joined creator list for multi-author fics ("Alice, Bob").
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    /// Raw description (may contain HTML / whitespace-only placeholders).
    pub desc: String,
    /// Unix millis.
    pub published: i64,
    /// Unix millis.
    pub updated: i64,
    /// One of: ongoing, complete, hiatus, cancelled.
    pub status: String,
    /// The original fic URL.
    pub source: String,
    /// Numeric site id in the host's source table (if any).
    pub source_id: i64,
    /// Numeric author id on the source site (when the adapter knows it).
    pub author_id: i64,
    /// Author profile URL, when the adapter found one.
    pub author_url: String,
    /// Site-local author id (e.g. AO3 user id).
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}

impl FicMetadata {
    /// Convenience for adapters that can't extract an author URL.
    pub fn with_author_url(mut self, url: impl Into<String>) -> Self {
        self.author_url = url.into();
        self
    }
}

/// A single chapter's content (HTML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,
}

/// Errors produced by scrapers. Hosts map these to their own error types.
#[derive(Debug)]
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
    Unsupported(String),
    AuthRequired(String),
    RateLimited(String),
    Internal(String),
}

impl fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(m) => write!(f, "network: {m}"),
            ScrapeError::ParseError(m) => write!(f, "parse error: {m}"),
            ScrapeError::Unsupported(m) => write!(f, "unsupported: {m}"),
            ScrapeError::AuthRequired(m) => write!(f, "auth required: {m}"),
            ScrapeError::RateLimited(m) => write!(f, "rate limited: {m}"),
            ScrapeError::Internal(m) => write!(f, "internal: {m}"),
        }
    }
}

impl std::error::Error for ScrapeError {}

/// Hook for recording scrape failures (self-healing / observability).
/// Hosts provide an implementation; the crate calls it best-effort and
/// never lets a hook error break the scrape path.
pub trait HealHook: Send + Sync {
    fn record_failure(&self, url: &str, error: &str, kind: &str);
}

/// No-op hook used when the host doesn't care about failure recording.
pub struct NoopHealHook;

impl HealHook for NoopHealHook {
    fn record_failure(&self, _url: &str, _error: &str, _kind: &str) {}
}

/// An adapter for one (or several) fanfiction sites.
#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Site id used by hosts (AO3=1, FFN=2, ...). Adapters may return 0
    /// when the host uses its own source table.
    fn source_id(&self) -> i64 {
        0
    }
    /// Whether this adapter can handle `url`.
    fn can_handle(&self, url: &str) -> bool;
    /// Whether this site needs a login before scrapes work (some stories
    /// are author-gated or adult-gated behind an account).
    fn requires_login(&self) -> bool {
        false
    }
    /// Log in with the given credentials. Called by hosts before
    /// [`lookup`](Self::lookup) when [`requires_login`](Self::requires_login)
    /// is true. The `client` must have a cookie store enabled so the
    /// session persists for subsequent requests.
    ///
    /// Adapters that don't need auth keep the default (error).
    async fn login(
        &self,
        _client: &reqwest::Client,
        _creds: &SiteCredentials,
    ) -> Result<(), ScrapeError> {
        Err(ScrapeError::AuthRequired("no login support".into()))
    }
    /// Fetch + parse metadata for `url`.
    async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>;
    /// Fetch chapter contents for a work already looked up.
    async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError>;
    /// Parse metadata from a pre-fetched HTML document (Wayback, cookie
    /// ingest, browser-extension capture, "paste the HTML", fixtures).
    ///
    /// Adapters override this to make their parsing testable against
    /// fixtures and usable by alternate ingest sources. The default
    /// implementation fetches `url` and delegates to the same parsing
    /// used by [`lookup`](Self::lookup) when the adapter implements
    /// [`fetch_html`](Self::fetch_html).
    async fn lookup_from_html(&self, _html: &str, _url: &str) -> Result<FicMetadata, ScrapeError> {
        Err(ScrapeError::Unsupported(
            "parse-from-HTML not implemented for this site".into(),
        ))
    }
    /// Parse chapters from a pre-fetched HTML document. Default returns
    /// an error; adapters override it to support alternate ingest sources.
    async fn fetch_chapters_from_html(
        &self,
        _html: &str,
        _meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        Err(ScrapeError::Unsupported(
            "fetch-chapters-from-HTML not implemented for this site".into(),
        ))
    }
    /// Fetch the raw HTML for `url` (used by the default `lookup` /
    /// `fetch_chapters` when the adapter delegates to the `_from_html`
    /// variants). Adapters may override for site-specific headers.
    async fn fetch_html(&self, client: &reqwest::Client, url: &str) -> Result<String, ScrapeError> {
        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(ScrapeError::Blocked);
        }
        resp.text()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))
    }
    /// Optional: extract tags for a work.
    async fn extract_tags(
        &self,
        _client: &reqwest::Client,
        _url: &str,
    ) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(vec![])
    }
}

/// Credentials for a site that requires login. Hosts source these from
/// config/env and pass them through the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteCredentials {
    /// Site domain this credential applies to (e.g. `fanficauthors.net`).
    pub domain: String,
    pub username: String,
    pub password: String,
    /// Whether adult content is allowed. Some archives gate adult works
    /// behind an "I am 18+" confirmation rather than a real login; hosts
    /// set this (e.g. from an `is_adult` config) to unlock them.
    #[serde(default)]
    pub is_adult: bool,
}

impl SiteCredentials {
    /// Build credentials with adult access explicitly enabled.
    pub fn with_adult(mut self) -> Self {
        self.is_adult = true;
        self
    }
}

/// Generate a host-style `url_id` from a numeric source id + work id.
/// Kept for compatibility with hosts that store `ao3_12345`-style ids.
pub fn generate_url_id(source_id: i64, local_id: &str) -> String {
    format!("{source_id}_{local_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_error_display() {
        assert_eq!(ScrapeError::NotFound.to_string(), "fic not found");
        assert_eq!(
            ScrapeError::Network("timeout".into()).to_string(),
            "network: timeout"
        );
    }

    #[test]
    fn test_generate_url_id() {
        assert_eq!(generate_url_id(1, "21845264"), "1_21845264");
    }

    #[test]
    fn test_extracted_tag_category() {
        let t = ExtractedTag::category("Fluff");
        assert_eq!(t.tag_type_id, 6);
    }
}
