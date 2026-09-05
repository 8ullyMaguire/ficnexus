use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::scrape::ScrapeError;
use crate::scrape::registry::ScraperRegistry;

// ---------------------------------------------------------------------------
// Batch training pipeline (pluggable recommendation platform)
// ---------------------------------------------------------------------------

/// Run the batch training pipeline for all enabled strategies: refresh the
/// unified signal view, then each strategy's `train` step, recording every
/// run in `rec_training_runs`. Idempotent and failure-tolerant: one
/// strategy's failure does not abort the others.
///
/// Callers: the worker tick (every `REC_TRAIN_EVERY_H` hours) or the admin
/// endpoint. Returns per-strategy run summaries.
pub async fn run_training_pipeline(
    db: &PgPool,
    config: &Config,
    http_client: &Client,
    ollama: &crate::services::ollama::OllamaClient,
    registry: &crate::recommender::registry::StrategyRegistry,
) -> Vec<serde_json::Value> {
    use crate::recommender::registry::build_context;

    let ctx = build_context(
        db.clone(),
        std::sync::Arc::new(config.clone()),
        http_client.clone(),
        ollama.clone(),
    );

    // 1. Refresh the unified signal view (with the curator multiplier).
    if let Err(e) = crate::recommender::signals::refresh_signals(db, config.rec_curator_prior).await
    {
        tracing::warn!("rec signal refresh failed: {e}");
    }

    let mut results = Vec::new();
    for spec in registry.specs() {
        let Some(strategy) = registry.get(&spec.name) else {
            continue;
        };
        let started = std::time::Instant::now();
        let run_id: i64 = sqlx::query_scalar(
            r#"INSERT INTO rec_training_runs (strategy, started_at, duration_ms, metrics, ok)
               VALUES ($1, NOW(), 0, '{}'::jsonb, FALSE)
               RETURNING id"#,
        )
        .bind(&spec.name)
        .fetch_one(db)
        .await
        .unwrap_or(0);

        let outcome = match strategy.train(&ctx).await {
            Ok(metrics) => {
                let duration_ms = started.elapsed().as_millis() as i64;
                if run_id > 0 {
                    let _ = sqlx::query(
                        r#"UPDATE rec_training_runs SET duration_ms = $1, metrics = $2, ok = TRUE
                           WHERE id = $3"#,
                    )
                    .bind(duration_ms)
                    .bind(&metrics)
                    .bind(run_id)
                    .execute(db)
                    .await;
                }
                serde_json::json!({
                    "strategy": spec.name,
                    "ok": true,
                    "metrics": metrics,
                    "duration_ms": duration_ms,
                })
            }
            Err(e) => {
                let duration_ms = started.elapsed().as_millis() as i64;
                if run_id > 0 {
                    let _ = sqlx::query(
                        r#"UPDATE rec_training_runs SET duration_ms = $1,
                               metrics = jsonb_build_object('error', $2), ok = FALSE
                           WHERE id = $3"#,
                    )
                    .bind(duration_ms)
                    .bind(e.to_string())
                    .bind(run_id)
                    .execute(db)
                    .await;
                }
                tracing::warn!("rec strategy {} training failed: {e}", spec.name);
                serde_json::json!({
                    "strategy": spec.name,
                    "ok": false,
                    "error": e.to_string(),
                    "duration_ms": duration_ms,
                })
            }
        };
        results.push(outcome);
    }
    results
}

// ---------------------------------------------------------------------------
// SiteFetcher trait — per-site scraper for user favourites / bookmarks
// ---------------------------------------------------------------------------

/// Trait that each fanfiction site must implement to support the
/// collaborative-filtering recommendation collection worker.
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    /// Canonical domain for this site (e.g. "archiveofourown.org").
    fn site_domain(&self) -> &str;

    /// Per-site rate-limit delay in seconds.
    ///
    /// Checks `config.rec_site_rate_limits` for an override keyed by `domain`,
    /// falling back to `config.rec_default_delay_secs`.
    fn rate_limit_delay(&self, config: &Config, domain: &str) -> u64 {
        config
            .rec_site_rate_limits
            .get(domain)
            .copied()
            .unwrap_or(config.rec_default_delay_secs)
    }

    /// Collect users who have favourited / bookmarked the given work URL.
    ///
    /// Returns a list of user profile URLs (or user identifiers) that can be
    /// passed to `collect_user_favourites` and `user_hash`.
    async fn collect_favouriters(
        &self,
        client: &Client,
        work_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Collect the set of works favourited by a user.
    ///
    /// Returns a list of identifiers (url_ids or work URLs) representing the
    /// works this user has bookmarked / favourited.
    async fn collect_user_favourites(
        &self,
        client: &Client,
        user_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    /// Compute a deterministic hash for a user's profile URL.
    ///
    /// The hash is SHA-256 of the lowercased URL, encoded as a hex string.
    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}

// ---------------------------------------------------------------------------
// PerSiteRateLimiter — atomic CAS-based delay enforcement
// ---------------------------------------------------------------------------

/// Per-site rate limiter that uses an atomic CAS loop to coordinate
/// concurrent access and enforce a minimum delay between requests.
pub struct PerSiteRateLimiter {
    /// Unix-epoch-nanosecond timestamp of the last successful request.
    last_request: AtomicI64,
    /// Minimum delay between requests, in seconds.
    delay_secs: u64,
}

impl PerSiteRateLimiter {
    pub fn new(delay_secs: u64) -> Self {
        Self {
            // 0 means "never requested" — the first caller always passes.
            last_request: AtomicI64::new(0),
            delay_secs,
        }
    }

    /// Block until the required delay has elapsed since the last request.
    ///
    /// Uses a compare-and-swap loop to atomically claim the next time slot.
    /// Multiple concurrent callers for the same domain are serialised so that
    /// each waits the full delay from the previous actual request.
    pub async fn wait_if_needed(&self) {
        let delay_nanos = (self.delay_secs as u64) * 1_000_000_000;

        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;

            let last = self.last_request.load(Ordering::Acquire);
            let elapsed = now.wrapping_sub(last);

            if elapsed >= delay_nanos as i64 {
                // Enough time has passed — try to claim this slot.
                if self
                    .last_request
                    .compare_exchange(last, now, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    return;
                }
                // CAS failed (another thread claimed a slot between our load
                // and our CAS) — retry the whole check with a fresh timestamp.
            } else {
                // Still within the cooldown window — sleep for the remainder.
                let remaining = delay_nanos - elapsed as u64;
                tokio::time::sleep(Duration::from_nanos(remaining)).await;
                // After sleeping, loop back and try to claim the slot.
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Queue item serialised to / from Redis
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub url_id: String,
    pub site_domain: String,
    pub site_work_id: String,
}

// ---------------------------------------------------------------------------
// SiteFetcher implementations
// ---------------------------------------------------------------------------
// Only sites whose bookmark/favourite graphs are *publicly* scrapable are
// registered. A fetcher that returned empty result sets would silently poison
// the co-occurrence tables, and one that cannot actually see favourites must
// not pretend to scrape. Sites whose favourite lists are private
// (fanfiction.net favourites/alerts are account-scoped, XenForo reactions
// require a login, adult-fanfiction.org and hpfanfic.com gate their favourite
// lists behind accounts) are deliberately NOT registered; `enqueue` skips
// them and `enqueue_fic` refuses to queue their domains.
//
// AO3 is implemented for real: a work's bookmarkers are public at
// `/works/{id}/bookmarks` and a user's bookmarks at `/users/{name}/bookmarks`.

/// Canonical domain for AO3.
pub const AO3_DOMAIN: &str = "archiveofourown.org";
const AO3_BASE: &str = "https://archiveofourown.org";
/// AO3 is polite to ~1 req/sec; extra sleep between pagination pages.
const AO3_PAGE_DELAY: Duration = Duration::from_secs(1);

struct Ao3Fetcher;

/// Extract the numeric work id from an AO3 work URL
/// (`.../works/123456[/...]` → `123456`).
fn ao3_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}

/// Extract the username from an AO3 user href/URL (`/users/{name}[/...]`).
fn ao3_username(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/users/([^/]+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}

/// Percent-decode `%XX` escapes in a username pulled from an AO3 href.
fn ao3_decode_user(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 1 {
            if let Ok(v) = u8::from_str_radix(&raw[i + 1..i + 3], 16) {
                out.push(v as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Re-encode a decoded username for use in an AO3 URL path.
fn utf8_percent_encode_user(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for b in name.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Which side of the bookmark graph a page contains.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BookmarkKind {
    /// Work bookmarks page → bookmarking *users*.
    Bookmarkers,
    /// User bookmarks page → bookmarked *works*.
    Works,
}

/// Parse one AO3 bookmarks page. Returns the collected canonical URLs and
/// whether a next page exists (AO3 pagination exposes `<a rel="next">`
/// except on the last page).
fn parse_bookmarks_page(html: &str, kind: BookmarkKind) -> (Vec<String>, bool) {
    let doc = scraper::Html::parse_document(html);
    let li_sel = scraper::Selector::parse("li.bookmark").expect("valid selector");
    let a_sel = scraper::Selector::parse("a[href]").expect("valid selector");
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for li in doc.select(&li_sel) {
        for a in li.select(&a_sel) {
            let Some(href) = a.value().attr("href") else {
                continue;
            };
            let target = match kind {
                BookmarkKind::Bookmarkers => ao3_username(href)
                    .map(|raw| format!("{AO3_BASE}/users/{}", ao3_decode_user(&raw))),
                BookmarkKind::Works => ao3_work_id(href).map(|id| format!("{AO3_BASE}/works/{id}")),
            };
            if let Some(t) = target {
                if seen.insert(t.clone()) {
                    out.push(t);
                }
            }
        }
    }
    let next_sel = scraper::Selector::parse("a[rel='next']").expect("valid selector");
    let has_next = doc.select(&next_sel).next().is_some();
    (out, has_next)
}

/// Rewrite a bookmarks URL to point at `page` (AO3 uses `?page=N`).
fn next_page_url(current: &str, page: u32) -> String {
    let (base, query) = match current.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (current, None),
    };
    let mut kept: Vec<String> = query
        .map(|q| {
            q.split('&')
                .filter(|p| !p.starts_with("page="))
                .map(|p| p.to_string())
                .collect()
        })
        .unwrap_or_default();
    kept.push(format!("page={page}"));
    format!("{base}?{}", kept.join("&"))
}

/// Fetch a URL with a browser-ish UA, returning the body text.
async fn fetch_html(client: &Client, url: &str) -> Result<String, ScrapeError> {
    let resp = client
        .get(url)
        .header("User-Agent", fanfic_scrapers::sites::http::USER_AGENT)
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(ScrapeError::NotFound);
    }
    if !resp.status().is_success() {
        return Err(ScrapeError::Network(format!("HTTP {}", resp.status())));
    }
    resp.text()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))
}

impl Ao3Fetcher {
    /// Fetch all pages of an AO3 bookmarks listing, up to `max_pages`.
    async fn collect_bookmark_pages(
        client: &Client,
        first_url: String,
        kind: BookmarkKind,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        let cap = max_pages.max(1);
        let mut all = Vec::new();
        let mut url = Some(first_url);
        let mut pages = 0u32;
        while let Some(u) = url {
            if pages >= cap {
                break;
            }
            let html = fetch_html(client, &u).await?;
            let (mut items, has_next) = parse_bookmarks_page(&html, kind);
            all.append(&mut items);
            pages += 1;
            url = if has_next && pages < cap {
                Some(next_page_url(&u, pages + 1))
            } else {
                None
            };
            if url.is_some() {
                tokio::time::sleep(AO3_PAGE_DELAY).await;
            }
        }
        Ok(all)
    }
}

#[async_trait]
impl SiteFetcher for Ao3Fetcher {
    fn site_domain(&self) -> &str {
        AO3_DOMAIN
    }

    async fn collect_favouriters(
        &self,
        client: &Client,
        work_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        let work_id = ao3_work_id(work_url)
            .ok_or_else(|| ScrapeError::ParseError(format!("not an AO3 work URL: {work_url}")))?;
        let first = format!("{AO3_BASE}/works/{work_id}/bookmarks");
        Self::collect_bookmark_pages(client, first, BookmarkKind::Bookmarkers, max_pages).await
    }

    async fn collect_user_favourites(
        &self,
        client: &Client,
        user_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError> {
        let name = ao3_username(user_url)
            .ok_or_else(|| ScrapeError::ParseError(format!("not an AO3 user URL: {user_url}")))?;
        let first = format!(
            "{AO3_BASE}/users/{}/bookmarks",
            utf8_percent_encode_user(&name)
        );
        Self::collect_bookmark_pages(client, first, BookmarkKind::Works, max_pages).await
    }
}
// ---------------------------------------------------------------------------
// CollectionWorker — background worker that drives per-site collection
// ---------------------------------------------------------------------------

/// Background collection worker that scrapes user favourites / bookmarks
/// from fanfiction sites and populates the recommendation-engine tables.
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    #[allow(dead_code)]
    registry: Arc<ScraperRegistry>,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiter>,
}

impl CollectionWorker {
    /// Create a new collection worker.
    pub fn new(
        db: PgPool,
        redis: MultiplexedConnection,
        client: Client,
        config: Config,
        registry: Arc<ScraperRegistry>,
    ) -> Self {
        let mut fetchers: Vec<Box<dyn SiteFetcher>> = Vec::new();
        fetchers.push(Box::new(Ao3Fetcher));
        // Sites without a public bookmark graph are intentionally absent —
        // see the SiteFetcher implementations banner above.

        let mut rate_limiters = HashMap::new();
        for fetcher in &fetchers {
            let domain = fetcher.site_domain().to_string();
            let delay = fetcher.rate_limit_delay(&config, &domain);
            rate_limiters.insert(domain, PerSiteRateLimiter::new(delay));
        }

        Self {
            db,
            redis: Mutex::new(redis),
            client,
            config,
            registry,
            fetchers,
            rate_limiters,
        }
    }

    /// Whether a site domain has a registered `SiteFetcher`.
    pub fn supports_domain(&self, site_domain: &str) -> bool {
        self.fetchers.iter().any(|f| f.site_domain() == site_domain)
    }

    /// Resolve a fic's (site_domain, site_work_id) from `fic_info` and enqueue
    /// it for background favourite-collection when the site is supported.
    ///
    /// Scraped works store the original URL in `fic_info.source`; the local
    /// site work id is the `/works/{id}` (or equivalent) path segment. Works
    /// from unsupported sites (or manual uploads) are skipped with a debug
    /// log instead of being pushed onto a queue nobody polls.
    pub async fn enqueue_fic(&self, url_id: &str) -> Result<(), redis::RedisError> {
        let row: Option<(Option<String>, Option<i32>)> =
            sqlx::query_as("SELECT source, work_id FROM fic_info WHERE id = $1")
                .bind(url_id)
                .fetch_optional(&self.db)
                .await
                .unwrap_or(None);

        let Some((source, work_id)) = row else {
            debug!("enqueue_fic: url_id {url_id} not in fic_info — skipping");
            return Ok(());
        };

        // AO3 works: source is the fic URL used at scrape time.
        if let Some(src) = source.as_deref() {
            if src.contains(AO3_DOMAIN) {
                let local_id = ao3_work_id(src).or_else(|| work_id.map(|w| w.to_string()));
                if let Some(site_work_id) = local_id {
                    return self.enqueue(url_id, AO3_DOMAIN, &site_work_id).await;
                }
            }
        }

        debug!("enqueue_fic: {url_id} has no registered favourite-collection site — skipping");
        Ok(())
    }

    /// Enqueue a fic URL for background collection.
    ///
    /// Pushes a `QueueItem` as JSON onto the Redis list
    /// `collection_queue:{site_domain}` so the worker loop can pick it up.
    /// Domains without a registered fetcher are dropped (nothing polls their
    /// queues) — callers should prefer [`Self::enqueue_fic`].
    pub async fn enqueue(
        &self,
        url_id: &str,
        site_domain: &str,
        site_work_id: &str,
    ) -> Result<(), redis::RedisError> {
        if !self.supports_domain(site_domain) {
            debug!("enqueue: no SiteFetcher for {site_domain} — dropping {url_id}");
            return Ok(());
        }
        let item = QueueItem {
            url_id: url_id.to_string(),
            site_domain: site_domain.to_string(),
            site_work_id: site_work_id.to_string(),
        };
        let json = serde_json::to_string(&item).expect("QueueItem serialisation should not fail");

        let key = format!("collection_queue:{}", site_domain);
        let mut conn = self.redis.lock().await;

        redis::cmd("LPUSH")
            .arg(&[key.as_str(), json.as_str()])
            .query_async(&mut *conn)
            .await
    }

    /// Main worker loop — runs forever.
    ///
    /// For each known site domain:
    ///   1. Pops the next item from `collection_queue:{domain}`.
    ///   2. Applies the per-site rate limiter.
    ///   3. Processes the item (scrape favouriters, their favourites,
    ///      co-occurrence, and counter updates).
    pub async fn run(&self) {
        info!("Collection worker started — polling Redis queues for all known sites");

        loop {
            for fetcher in &self.fetchers {
                let domain = fetcher.site_domain();
                let key = format!("collection_queue:{}", domain);

                let item_str: Option<String> = {
                    let mut conn = self.redis.lock().await;
                    redis::cmd("LPOP")
                        .arg(&key)
                        .query_async(&mut *conn)
                        .await
                        .unwrap_or(None)
                };

                if let Some(item_str) = item_str {
                    // Apply per-site rate limit before making HTTP requests.
                    if let Some(rl) = self.rate_limiters.get(domain) {
                        rl.wait_if_needed().await;
                    }

                    match serde_json::from_str::<QueueItem>(&item_str) {
                        Ok(item) => {
                            debug!("Processing {} from {}", item.url_id, domain);
                            if let Err(e) = self.process_work(item).await {
                                error!("Error processing work on {}: {}", domain, e);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Invalid queue item on {}: {} — payload: {}",
                                domain, e, item_str
                            );
                        }
                    }
                }
            }

            // Brief sleep to avoid busy-looping Redis when queues are empty.
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Process a single queue item — the core collection procedure.
    ///
    /// a) Fetch favouriters (users who bookmarked this work).
    /// b) For each *new* user (user_hash not yet in `fic_bookmarks` for this
    ///    work), fetch their favourite works.
    /// c) Update co-occurrence: for each user's favourite set, increment
    ///    pairwise co-occurrence counts in `fic_bookmark_cooccur`.
    /// d) Update `favouriter_count` on `fic_works`.
    async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
        let fetcher = self
            .fetchers
            .iter()
            .find(|f| f.site_domain() == item.site_domain)
            .ok_or_else(|| format!("No SiteFetcher registered for domain {}", item.site_domain))?;

        let work_url = format!("https://{}/works/{}", item.site_domain, item.site_work_id);

        // ---- (a) Fetch favouriters ----
        let favouriters = fetcher
            .collect_favouriters(&self.client, &work_url, self.config.rec_max_favourite_pages)
            .await?;

        let mut new_user_count: u32 = 0;

        for user_url in &favouriters {
            let user_hash = fetcher.user_hash(user_url);

            // Skip users already recorded for this work.
            let already_exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)",
            )
            .bind(&user_hash)
            .bind(&item.url_id)
            .fetch_one(&self.db)
            .await
            .unwrap_or(false);

            if already_exists {
                continue;
            }

            new_user_count += 1;

            // ---- Ensure the fic_works row exists (or update counter) ----
            sqlx::query(
                r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count,
                                           first_favourite_scraped, last_favourite_scraped)
                   VALUES ($1, $2, $3, 1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   ON CONFLICT (url_id) DO UPDATE SET
                       favouriter_count    = fic_works.favouriter_count + 1,
                       last_favourite_scraped = CURRENT_TIMESTAMP"#,
            )
            .bind(&item.url_id)
            .bind(&item.site_domain)
            .bind(&item.site_work_id)
            .execute(&self.db)
            .await?;

            // ---- Record the bookmark ----
            sqlx::query(
                "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
                 VALUES ($1, $2, $3)
                 ON CONFLICT DO NOTHING",
            )
            .bind(&user_hash)
            .bind(&item.url_id)
            .bind(&item.site_domain)
            .execute(&self.db)
            .await?;

            // ---- (b) Fetch this user's favourite works ----
            let user_favourites = fetcher
                .collect_user_favourites(
                    &self.client,
                    user_url,
                    self.config.rec_max_user_favourite_pages,
                )
                .await?;

            // ---- (c) Map scraped work URLs to FicNexus url_ids, then update
            // co-occurrence for all pairs in the set. Works not yet in the
            // archive are skipped — they cannot be recommended, and raw
            // site URLs would never join `fic_works.url_id`.
            if user_favourites.len() >= 2 {
                let url_ids = self.resolve_work_url_ids(&user_favourites).await;
                if url_ids.len() >= 2 {
                    self.update_cooccurrence(&item.site_domain, &url_ids)
                        .await?;
                }
            }
        }

        // ---- (d) Touch the fic_works row if no new users (ensure it exists) ----
        if new_user_count == 0 {
            sqlx::query(
                r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count)
                   VALUES ($1, $2, $3, 0)
                   ON CONFLICT (url_id) DO UPDATE SET
                       last_favourite_scraped = CURRENT_TIMESTAMP
                   WHERE fic_works.first_favourite_scraped IS NULL"#,
            )
            .bind(&item.url_id)
            .bind(&item.site_domain)
            .bind(&item.site_work_id)
            .execute(&self.db)
            .await?;
        } else {
            info!(
                "Processed {} on {} — {} new user(s), {} favouriter(s) total",
                item.url_id,
                item.site_domain,
                new_user_count,
                favouriters.len(),
            );
        }

        Ok(())
    }

    /// Map scraped work URLs (e.g. `https://archiveofourown.org/works/123`)
    /// to FicNexus `url_id`s. Scraped works use the deterministic
    /// `generate_url_id(source_id, site_work_id)` scheme (AO3 source_id = 1);
    /// only works actually present in `fic_info` are returned, so co-occurrence
    /// rows always join against `fic_works.url_id`.
    async fn resolve_work_url_ids(&self, urls: &[String]) -> Vec<String> {
        let mut ids = Vec::new();
        for url in urls {
            let Some(work_id) = ao3_work_id(url) else {
                continue;
            };
            let url_id = crate::scrape::generate_url_id(1, &work_id);
            let exists: Option<i32> = sqlx::query_scalar("SELECT 1 FROM fic_info WHERE id = $1")
                .bind(&url_id)
                .fetch_optional(&self.db)
                .await
                .unwrap_or(None);
            if exists.is_some() {
                ids.push(url_id);
            } else {
                debug!("resolve_work_url_ids: {url} not archived — skipping");
            }
        }
        ids
    }

    /// Increment pairwise co-occurrence counts for all combinations of works
    /// in the given set.  The `fic_bookmark_cooccur` table enforces
    /// `work_a < work_b` via a CHECK constraint.
    async fn update_cooccurrence(
        &self,
        site_domain: &str,
        works: &[String],
    ) -> Result<(), sqlx::Error> {
        for i in 0..works.len() {
            for j in (i + 1)..works.len() {
                let (work_a, work_b) = if works[i] < works[j] {
                    (works[i].as_str(), works[j].as_str())
                } else {
                    (works[j].as_str(), works[i].as_str())
                };

                sqlx::query(
                    r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, site_domain, cooccur_count)
                       VALUES ($1, $2, $3, 1)
                       ON CONFLICT (work_a, work_b) DO UPDATE SET
                           cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                           last_updated  = CURRENT_TIMESTAMP"#,
                )
                .bind(work_a)
                .bind(work_b)
                .bind(site_domain)
                .execute(&self.db)
                .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ao3_work_id_extracts_numeric_id() {
        assert_eq!(
            ao3_work_id("https://archiveofourown.org/works/123456/bookmarks"),
            Some("123456".into())
        );
        assert_eq!(
            ao3_work_id("https://archiveofourown.org/works/999?view_full_work=true"),
            Some("999".into())
        );
        assert_eq!(ao3_work_id("https://archiveofourown.org/users/foo"), None);
    }

    #[test]
    fn ao3_username_extracts_name() {
        assert_eq!(
            ao3_username("/users/SomeUser/gifts?work_id=1"),
            Some("SomeUser".into())
        );
        assert_eq!(ao3_username("/works/123"), None);
    }

    #[test]
    fn username_round_trips_through_percent_encoding() {
        let encoded = utf8_percent_encode_user("Round About Town");
        assert_eq!(encoded, "Round%20About%20Town");
        assert_eq!(ao3_decode_user(&encoded), "Round About Town");
        // Plain names pass through unchanged.
        assert_eq!(ao3_decode_user("plain_user.name-x"), "plain_user.name-x");
    }

    #[test]
    fn parse_bookmarkers_page_finds_users_and_pagination() {
        let html = r#"
            <ol class="bookmark index group">
              <li id="bookmark_1" class="bookmark blurb group" >
                <p class="byline"><a href="/users/Alpha">Alpha</a></p>
                <h4 class="heading"><a href="/works/111">A work</a></h4>
              </li>
              <li id="bookmark_2" class="bookmark blurb group">
                <p class="byline"><a href="/users/Beta">Beta</a></p>
                <h4 class="heading"><a href="/works/222">Another</a></h4>
              </li>
            </ol>
            <ol class="pagination actions"><li class="next"><a rel="next" href="?page=2">Next →</a></li></ol>
        "#;
        let (users, has_next) = parse_bookmarks_page(html, BookmarkKind::Bookmarkers);
        assert_eq!(
            users,
            vec![
                "https://archiveofourown.org/users/Alpha".to_string(),
                "https://archiveofourown.org/users/Beta".to_string(),
            ]
        );
        assert!(has_next);
    }

    #[test]
    fn parse_user_bookmarks_page_finds_works_and_last_page_has_no_next() {
        let html = r#"
            <ol class="bookmark index group">
              <li id="bookmark_1" class="bookmark blurb group">
                <p class="byline"><a href="/users/Owner">Owner</a></p>
                <h4 class="heading"><a href="/works/111">One</a></h4>
              </li>
              <li id="bookmark_2" class="bookmark blurb group">
                <h4 class="heading"><a href="/works/111/chapters/999">One ch2</a></h4>
              </li>
            </ol>
        "#;
        let (works, has_next) = parse_bookmarks_page(html, BookmarkKind::Works);
        // Both chapter and work links resolve to the same canonical work URL,
        // and duplicates are removed.
        assert_eq!(
            works,
            vec!["https://archiveofourown.org/works/111".to_string()]
        );
        assert!(!has_next);
    }

    #[test]
    fn next_page_url_preserves_query_and_replaces_page() {
        assert_eq!(
            next_page_url("https://archiveofourown.org/works/1/bookmarks", 2),
            "https://archiveofourown.org/works/1/bookmarks?page=2"
        );
        assert_eq!(
            next_page_url("https://archiveofourown.org/works/1/bookmarks?page=2", 3),
            "https://archiveofourown.org/works/1/bookmarks?page=3"
        );
        assert_eq!(
            next_page_url(
                "https://archiveofourown.org/tags/x/works?exclude=1&page=4",
                5
            ),
            "https://archiveofourown.org/tags/x/works?exclude=1&page=5"
        );
    }

    #[test]
    fn supports_domain_matches_registered_fetchers() {
        // Pure check of the AO3 domain constant used for registration.
        assert_eq!(AO3_DOMAIN, "archiveofourown.org");
        assert!(ao3_work_id(&format!("{AO3_BASE}/works/42")).is_some());
    }
}
