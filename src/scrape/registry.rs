//! FicNexus's scraper registry — wraps [`fanfic_scrapers::Registry`] and adds
//! the FicNexus self-healing failure hook.

use super::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use crate::heal::classifier;
use fanfic_scrapers::sites;

/// Registry of all available scrapers.
pub struct ScraperRegistry {
    inner: fanfic_scrapers::Registry,
}

impl ScraperRegistry {
    /// Create a new registry with all known scrapers.
    /// FanFicFare is the primary scraper (handles 100+ sites).
    /// Native Rust scrapers are fallbacks for sites where FanFicFare
    /// may not work well or where we need finer control.
    pub fn new() -> Self {
        let mut inner = fanfic_scrapers::Registry::new();
        // Native Rust scrapers (full FanFicFare adapter parity — 107 real
        // sites). No Python fanficfare CLI dependency; the catch-all fallback
        // is feature-gated in the crate (fff-fallback) and deliberately not
        // registered here.
        inner.register(Box::new(sites::ao3::Ao3Scraper::ao3()));
        // Other OTW/Archive-software sites share AO3's layout.
        for d in ["adastrafanfic.com", "cfaa", "squidgeworld.org", "superlove"] {
            inner.register(Box::new(sites::ao3::Ao3Scraper::with_domain(d)));
        }
        inner.register(Box::new(sites::ffnet::FfNetScraper));
        inner.register(Box::new(sites::ficbook::FicBookScraper));
        inner.register(Box::new(sites::fimfiction::FimFictionScraper));
        inner.register(Box::new(sites::literotica::LiteroticaScraper));
        inner.register(Box::new(sites::xenforo::XenForoScraper));
        inner.register(Box::new(sites::fictionpress::FictionPressScraper));
        inner.register(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
        inner.register(Box::new(sites::hpfanfic::HpFanFicScraper));
        inner.register(Box::new(sites::royalroad::RoyalRoadScraper));
        inner.register(Box::new(sites::wattpad::WattpadScraper));
        inner.register(Box::new(sites::spiritfanfiction::SpiritFanfictionScraper));
        inner.register(Box::new(sites::syosetu::SyosetuScraper));
        inner.register(Box::new(sites::asianfanfics::AsianFanFicsScraper));
        inner.register(Box::new(sites::deviantart::DeviantArtScraper));
        inner.register(Box::new(sites::quotev::QuotevScraper));
        inner.register(Box::new(sites::fanfictionsfr::FanfictionsFrScraper));
        inner.register(Box::new(sites::kakuyomu::KakuyomuScraper));
        inner.register(Box::new(sites::mediaminer::MediaMinerScraper));
        inner.register(Box::new(sites::chireads::ChiReadsScraper));
        inner.register(Box::new(sites::pmdfanfiction::PmdFanfictionScraper));
        inner.register(Box::new(sites::lcfanfic::LcFanficScraper));
        inner.register(Box::new(sites::fireflyfans::FireflyFansScraper));
        inner.register(Box::new(sites::ficwad::FicwadScraper));
        inner.register(Box::new(sites::fictionmania::FictionManiaScraper));
        inner.register(Box::new(sites::fanfiktionde::FanfiktionDeScraper));
        inner.register(Box::new(sites::touchfluffytail::TouchFluffyTailScraper));
        inner.register(Box::new(sites::fanficsme::FanficsMeScraper));
        inner.register(Box::new(sites::fanficauthors::FanficAuthorsScraper));
        inner.register(Box::new(sites::fictionhunt::FictionHuntScraper));
        inner.register(Box::new(sites::inkbunny::InkBunnyScraper));
        inner.register(Box::new(sites::sofurry::SoFurryScraper));
        inner.register(Box::new(sites::dokuga::DokugaScraper));
        inner.register(Box::new(sites::phoenixsong::PhoenixSongScraper));
        inner.register(Box::new(sites::storiesofarda::StoriesOfArdaScraper));
        inner.register(Box::new(sites::fictionalley::FictionalleyScraper));
        inner.register(Box::new(sites::novelall::NovelAllScraper));
        inner.register(Box::new(sites::masseffect2in::MassEffect2InScraper));
        inner.register(Box::new(sites::readonlymind::ReadonlyMindScraper::default()));
        inner.register(Box::new(sites::utopiastories::UtopiaStoriesScraper::default()));
        inner.register(Box::new(sites::asexstories::ASexStoriesScraper::default()));
        inner.register(Box::new(sites::aneroticstory::AnEroticStoryScraper::default()));
        inner.register(Box::new(sites::mcstories::MCStoriesScraper::default()));
        inner.register(Box::new(sites::hentaifoundry::HentaiFoundryScraper::default()));
        inner.register(Box::new(sites::bdsmlibrary::BdsmLibraryScraper::default()));
        // eFiction-variant family: 26 viewstory.php?sid= archives
        // (psychfic, wolverineandrogue, sycophanthex sites, etc.).
        for s in sites::efiction_variant::EficVariantScraper::all() {
            inner.register(Box::new(s));
        }
        inner.register(Box::new(sites::tthfanfic::TthFanficScraper));
        // StoriesOnline family: storiesonline.net + scifistories + storyroom.
        for s in sites::storiesonline::StoriesOnlineScraper::all() {
            inner.register(Box::new(s));
        }
        // Native ScribbleHub — FFF's adapter crashes on newer markup
        // (upstream JimmXinu/FanFicFare#1397); ours is robust.
        inner.register(Box::new(sites::scribblehub::ScribbleHubScraper));
        // eFiction family — 19 niche archives via one config-driven adapter.
        for s in sites::efiction::EfictionScraper::all() {
            inner.register(Box::new(s));
        }
        // Generic WordPress novel sites (novelfull.net #1316, novelswd.com
        // #795, novelhall.com #762, hostednovel.com #523, ...).
        inner.register(Box::new(sites::wordpress_novel::WordPressNovelScraper {
            hosts: &["novelfull.com", "novelswd.com", "novelhall.com", "hostednovel.com"],
        }));
        ScraperRegistry { inner }
    }

    /// Find a scraper that can handle the given URL
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.inner.find_scraper(url)
    }

    /// All registered scrapers
    pub fn scrapers(&self) -> &[Box<dyn SiteScraper>] {
        self.inner.scrapers()
    }

    /// True when a NON-catch-all scraper can handle the URL. The FanFicFare
    /// fallback (pushed first) accepts every URL, so callers use this as a
    /// pre-flight gate to reject bogus hosts before spending a subprocess on
    /// them.
    pub fn has_specific_scraper(&self, url: &str) -> bool {
        self.inner.has_specific_scraper(url)
    }

    /// Find the best scraper for a URL: prefer a native (non-catch-all)
    /// scraper when one can handle the URL, falling back to FanFicFare.
    pub fn find_specific_or_fff(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.inner.find_specific_or_fallback(url)
    }

    /// Lookup metadata using the appropriate scraper.
    ///
    /// Self-heal hook: when NO scraper handles the URL and it looks like a
    /// fic URL (host has a dot + path len > 1), record a scrape_failures row
    /// (kind `unknown`) so the admin heal endpoint can see unsupported-host
    /// traffic. Best-effort: a DB error must never break the export path.
    pub async fn lookup(
        &self,
        client: &reqwest::Client,
        url: &str,
        heal: Option<&crate::heal::HealService>,
    ) -> Result<FicMetadata, ScrapeError> {
        self.lookup_authed(client, url, &[], heal).await
    }

    /// Lookup with optional login pre-pass (credentials from env).
    pub async fn lookup_authed(
        &self,
        client: &reqwest::Client,
        url: &str,
        creds: &[fanfic_scrapers::SiteCredentials],
        heal: Option<&crate::heal::HealService>,
    ) -> Result<FicMetadata, ScrapeError> {
        match self.inner.find_specific_or_fallback(url) {
            Some(scraper) => {
                if scraper.requires_login() {
                    let domain = fanfic_scrapers::sites::http::host_of(url);
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
            None => {
                if let Some(heal) = heal {
                    if classifier::looks_like_fic_url(url) {
                        heal.record_failure(
                            url,
                            None,
                            &classifier::ErrorKind::Unknown,
                            Some("no scraper handles this URL"),
                            None,
                        )
                        .await;
                    }
                }
                Err(ScrapeError::Unsupported(format!("no scraper for {url}")))
            }
        }
    }

    /// Fetch chapters for a work already looked up.
    pub async fn fetch_chapters(
        &self,
        client: &reqwest::Client,
        meta: &FicMetadata,
    ) -> Result<Vec<Chapter>, ScrapeError> {
        let scraper = self.find_specific_or_fff(&meta.source).ok_or_else(|| {
            ScrapeError::Unsupported(format!("no scraper for {}", meta.source))
        })?;
        scraper.fetch_chapters(client, meta).await
    }

    pub fn scraper_count(&self) -> usize {
        self.inner.scrapers().len()
    }
}

impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
