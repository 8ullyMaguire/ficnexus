//! URL routing: pick the right [`SiteScraper`] for a fic URL.

use crate::{FicMetadata, HealHook, ScrapeError, SiteScraper};

/// Collection of scrapers. The FanFicFare catch-all is expected to be
/// registered FIRST (it accepts every URL); native scrapers follow and are
/// preferred when they match.
pub struct Registry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self { scrapers: Vec::new() }
    }

    /// Register an adapter (call order matters: first = catch-all).
    pub fn register(&mut self, scraper: Box<dyn SiteScraper>) -> &mut Self {
        self.scrapers.push(scraper);
        self
    }

    /// Find the first scraper that can handle `url`.
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }

    /// All registered scrapers.
    pub fn scrapers(&self) -> &[Box<dyn SiteScraper>] {
        &self.scrapers
    }

    /// True when a NON-catch-all scraper can handle the URL. The FanFicFare
    /// fallback (first) accepts every URL, so callers use this as a
    /// pre-flight gate to reject bogus hosts before spending a subprocess
    /// on them.
    pub fn has_specific_scraper(&self, url: &str) -> bool {
        self.scrapers.iter().skip(1).any(|s| s.can_handle(url))
    }

    /// Find the best scraper: prefer a native (non-catch-all) scraper when
    /// one can handle the URL, falling back to the catch-all (index 0).
    pub fn find_specific_or_fallback(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers
            .iter()
            .skip(1)
            .find(|s| s.can_handle(url))
            .or_else(|| self.scrapers.first())
    }

    /// Lookup metadata using the appropriate scraper. Best-effort failure
    /// recording via the optional [`HealHook`].
    pub async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
        heal: Option<&dyn HealHook>,
    ) -> Result<FicMetadata, ScrapeError> {
        match self.find_scraper(url) {
            Some(scraper) => scraper.lookup(client, url).await,
            None => {
                if let Some(heal) = heal {
                    heal.record_failure(url, "no scraper handles this URL", "unknown");
                }
                Err(ScrapeError::Unsupported(format!("no scraper for {url}")))
            }
        }
    }

    /// Lookup with optional login pre-pass: when the chosen scraper
    /// requires login and matching credentials are supplied, log in first
    /// (the client must have a cookie store so the session persists).
    pub async fn lookup_authed(
        &self,
        client: &reqwest::Client,
        url: &str,
        creds: &[crate::SiteCredentials],
        heal: Option<&dyn HealHook>,
    ) -> Result<FicMetadata, ScrapeError> {
        let scraper = match self.find_scraper(url) {
            Some(s) => s,
            None => {
                if let Some(heal) = heal {
                    heal.record_failure(url, "no scraper handles this URL", "unknown");
                }
                return Err(ScrapeError::Unsupported(format!("no scraper for {url}")));
            }
        };
        if scraper.requires_login() {
            // Find credentials for this scraper's domain.
            let domain = scraper_domain(scraper.as_ref(), url);
            if let Some(creds) = creds.iter().find(|c| c.domain == domain) {
                scraper.login(client, creds).await?;
            } else {
                return Err(ScrapeError::AuthRequired(format!(
                    "{} requires login; no credentials configured",
                    domain
                )));
            }
        }
        scraper.lookup(client, url).await
    }
}

/// Best-effort domain for a scraper given a URL (used to match creds).
/// Falls back to the URL's host.
fn scraper_domain(_scraper: &dyn SiteScraper, url: &str) -> String {
    url.split('/')
        .nth(2)
        .unwrap_or("")
        .trim_start_matches("www.")
        .to_lowercase()
}

/// Convenience: a [`Registry`] with only a no-op heal hook.
pub struct RegistryBuilder {
    inner: Registry,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self {
            inner: Registry::new(),
        }
    }
    pub fn register(mut self, scraper: Box<dyn SiteScraper>) -> Self {
        self.inner.register(scraper);
        self
    }
    pub fn build(self) -> Registry {
        self.inner
    }
}

impl Default for RegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Keep `NoopHealHook` importable from this module for convenience.
pub use crate::NoopHealHook as _NoopHealHook;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::NoopHealHook;

    struct DummyScraper {
        prefix: &'static str,
    }
    #[async_trait]
    impl SiteScraper for DummyScraper {
        fn can_handle(&self, url: &str) -> bool {
            url.starts_with(self.prefix)
        }
        async fn lookup(
            &self,
            _client: &reqwest::Client,
            _url: &str,
        ) -> Result<FicMetadata, ScrapeError> {
            Err(ScrapeError::Unsupported("dummy".into()))
        }
        async fn fetch_chapters(
            &self,
            _client: &reqwest::Client,
            _meta: &FicMetadata,
        ) -> Result<Vec<crate::Chapter>, ScrapeError> {
            Ok(vec![])
        }
    }

    #[test]
    fn finds_specific_over_fallback() {
        let mut r = Registry::new();
        r.register(Box::new(DummyScraper { prefix: "https://" })); // catch-all
        r.register(Box::new(DummyScraper {
            prefix: "https://ao3.org",
        }));
        assert!(r.has_specific_scraper("https://ao3.org/works/1"));
        assert!(!r.has_specific_scraper("https://unknown.example/x"));
        assert!(r.find_specific_or_fallback("https://ao3.org/works/1").is_some());
    }

    #[test]
    fn lookup_records_unknown_via_hook() {
        struct Hook;
        impl HealHook for Hook {
            fn record_failure(&self, url: &str, _error: &str, kind: &str) {
                assert!(url.starts_with("https://nope.example"));
                assert_eq!(kind, "unknown");
            }
        }
        let r = Registry::new();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let res = rt.block_on(r.lookup(&reqwest::Client::new(), "https://nope.example/x", Some(&Hook)));
        assert!(matches!(res, Err(ScrapeError::Unsupported(_))));
        let _ = NoopHealHook;
    }
}
