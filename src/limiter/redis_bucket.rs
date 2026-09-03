use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{RateLimiter, RateLimitResult, TieredRateLimiter, TieredRateLimitResult, Tier};

/// Redis-backed token bucket rate limiter.
///
/// In production (`dynamic_rate_limit == true`) it enforces per-endpoint-class
/// tiers (download / auth / search) with per-IP AND per-`(ip, client_id)`
/// buckets, plus a Redis SET of shadowbanned client_ids
/// (`fichub:shadowban`). In test/dev mode (`dynamic_rate_limit == false`) it
/// keeps the legacy static-delay behavior so integration tests never hit
/// real buckets.
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,        // SHA of loaded Lua script
    dynamic_rate_limit: bool,
    static_delay_base: f64, // seconds

    // Datacenter IP cache (legacy flat-file path; superseded by geoip)
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    // MaxMind GeoLite2-ASN reader for datacenter detection (fail-open: None
    // when no MAXMIND_DB is configured or the file is unreadable).
    geoip: Option<crate::limiter::geoip::GeoIp>,

    // Configuration
    global_capacity: f64,
    global_flow: f64,
    ip_capacity: f64,
    ip_flow: f64,

    // Tiered rate limiting (anti-bot) — see Config::from_env for defaults.
    download_capacity: f64,
    download_flow: f64,
    auth_capacity: f64,
    auth_flow: f64,
    search_capacity: f64,
    search_flow: f64,
    client_bonus_capacity: f64,
    client_bonus_flow: f64,
    nat_multiplier: f64,
    shadowban_capacity: f64,
    shadowban_flow: f64,
    shadowban_ttl: u64,
    tiered_enabled: bool,
}

/// Default tier parameters (used by `new()` and by the pure-Rust tests).
/// These mirror the `Config::from_env` defaults:
///   download: 10 burst, refills 60/hr
///   auth:     10 burst, refills 10/min
///   search:   1000 burst, refills 1000/min
pub const DEFAULT_DOWNLOAD_CAPACITY: f64 = 10.0;
pub const DEFAULT_DOWNLOAD_FLOW: f64 = 60.0 / 3600.0; // 60/hour
pub const DEFAULT_AUTH_CAPACITY: f64 = 10.0;
pub const DEFAULT_AUTH_FLOW: f64 = 10.0 / 60.0; // 10/min
pub const DEFAULT_SEARCH_CAPACITY: f64 = 1000.0;
pub const DEFAULT_SEARCH_FLOW: f64 = 1000.0 / 60.0; // 1000/min
pub const DEFAULT_CLIENT_BONUS_CAPACITY: f64 = 5.0;
pub const DEFAULT_CLIENT_BONUS_FLOW: f64 = 30.0 / 3600.0;
pub const DEFAULT_NAT_MULTIPLIER: f64 = 4.0;
pub const DEFAULT_SHADOWBAN_CAPACITY: f64 = 5.0;
pub const DEFAULT_SHADOWBAN_FLOW: f64 = 5.0 / 3600.0; // 5/hour — friction, not a hard block
pub const DEFAULT_SHADOWBAN_TTL: u64 = 86400; // 24h
pub const SHADOWBAN_SET: &str = "fichub:shadowban";
/// Retry-after (seconds) returned for datacenter/hosting IPs — effectively a
/// hard block via the existing rate-limit error path.
pub const DATACENTER_BLOCK_SECS: u64 = 3600; // 1h

/// Pick the rate-limit tier for a request path. Pure function so it is
/// trivially unit-testable.
pub fn tier_for_path(path: &str) -> Tier {
    if path.starts_with("/api/epub")
        || path.starts_with("/api/v0/epub")
        || path.starts_with("/api/download")
        || path.starts_with("/cache/")
        || path.starts_with("/api/upload")
        || path.starts_with("/legacy/epub_export")
    {
        Tier::Download
    } else if path.starts_with("/api/auth/")
        || path.starts_with("/login")
        || path.starts_with("/register")
    {
        Tier::Auth
    } else if path.starts_with("/api/search")
        || path.starts_with("/docs")
        || path.starts_with("/static")
    {
        Tier::Search
    } else {
        Tier::Default
    }
}

impl RedisBucketLimiter {
    /// Create a new RedisBucketLimiter and load the Lua script
    pub async fn new(
        redis_conn: redis::aio::MultiplexedConnection,
        dynamic_rate_limit: bool,
    ) -> Result<Self, redis::RedisError> {
        Self::with_config(redis_conn, dynamic_rate_limit, None).await
    }
    /// Create a limiter with explicit tier parameters. `config` may supply
    /// the tiered settings from `Config`; `None` uses the defaults.
    pub async fn with_config(
        redis_conn: redis::aio::MultiplexedConnection,
        dynamic_rate_limit: bool,
        config: Option<&crate::config::Config>,
    ) -> Result<Self, redis::RedisError> {
        // Load the token bucket Lua script
        let lua_script = r#"
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
"#;

        let mut conn = redis_conn.clone();
        let lua_sha: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(lua_script)
            .query_async(&mut conn)
            .await?;

        let (download_capacity, download_flow) = config
            .map(|c| (c.rl_download_capacity, c.rl_download_flow))
            .unwrap_or((DEFAULT_DOWNLOAD_CAPACITY, DEFAULT_DOWNLOAD_FLOW));
        let (auth_capacity, auth_flow) = config
            .map(|c| (c.rl_auth_capacity, c.rl_auth_flow))
            .unwrap_or((DEFAULT_AUTH_CAPACITY, DEFAULT_AUTH_FLOW));
        let (search_capacity, search_flow) = config
            .map(|c| (c.rl_search_capacity, c.rl_search_flow))
            .unwrap_or((DEFAULT_SEARCH_CAPACITY, DEFAULT_SEARCH_FLOW));
        let (client_bonus_capacity, client_bonus_flow) = config
            .map(|c| (c.rl_client_bonus_capacity, c.rl_client_bonus_flow))
            .unwrap_or((DEFAULT_CLIENT_BONUS_CAPACITY, DEFAULT_CLIENT_BONUS_FLOW));
        let nat_multiplier = config
            .map(|c| c.rl_nat_multiplier)
            .unwrap_or(DEFAULT_NAT_MULTIPLIER);
        let (shadowban_capacity, shadowban_flow) = config
            .map(|c| (c.rl_shadowban_capacity, c.rl_shadowban_flow))
            .unwrap_or((DEFAULT_SHADOWBAN_CAPACITY, DEFAULT_SHADOWBAN_FLOW));
        let shadowban_ttl = config
            .map(|c| c.rl_shadowban_ttl)
            .unwrap_or(DEFAULT_SHADOWBAN_TTL);
        let tiered_enabled = config.map(|c| c.rl_tiered_enabled).unwrap_or(true);
        let geoip = config
            .and_then(|c| c.maxmind_db.as_deref())
            .and_then(|db_path| {
                let path = std::path::Path::new(db_path);
                let g = crate::limiter::geoip::GeoIp::open(path);
                if g.is_some() {
                    tracing::info!("Loaded GeoLite2-ASN datacenter DB from {db_path}");
                } else {
                    tracing::warn!(
                        "MAXMIND_DB={db_path} unreadable — datacenter blocking disabled (fail-open)"
                    );
                }
                g
            });

        Ok(RedisBucketLimiter {
            redis: redis_conn,
            lua_sha,
            dynamic_rate_limit,
            static_delay_base: 0.1,
            datacenter_ips: Arc::new(RwLock::new(HashSet::new())),
            geoip,
            global_capacity: 150.0,
            global_flow: 30.0,
            ip_capacity: 30.0,
            ip_flow: 0.116,   // ~1/8.6 tokens per second
            download_capacity,
            download_flow,
            auth_capacity,
            auth_flow,
            search_capacity,
            search_flow,
            client_bonus_capacity,
            client_bonus_flow,
            nat_multiplier,
            shadowban_capacity,
            shadowban_flow,
            shadowban_ttl,
            tiered_enabled,
        })
    }

    /// Check a token bucket and return wait time in seconds
    async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) -> Result<f64, redis::RedisError> {
        let mut conn = self.redis.clone();
        let result: f64 = redis::cmd("EVALSHA")
            .arg(&self.lua_sha[..])
            .arg(1)
            .arg(key)
            .arg(1.0)        // requested tokens
            .arg(capacity)
            .arg(flow)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Penalize by requesting extra tokens (on failure)
    async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let _: f64 = redis::cmd("EVALSHA")
            .arg(&self.lua_sha[..])
            .arg(1)
            .arg(key)
            .arg(1.5)        // penalize with 1.5 tokens
            .arg(capacity)
            .arg(flow)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }

    /// Load datacenter IP ranges from tag source files
    pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
        for (file_path, _type, _tag) in sources {
            match tokio::fs::read_to_string(file_path).await {
                Ok(content) => {
                    let mut ips = self.datacenter_ips.write().await;
                    for line in content.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            if let Ok(ip) = line.parse::<IpAddr>() {
                                ips.insert(ip);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Could not load IP tag file {}: {}", file_path, e);
                }
            }
        }
        tracing::info!(
            "Loaded {} datacenter IPs",
            self.datacenter_ips.read().await.len()
        );
    }

    /// Tier parameters for a given tier. The per-IP bucket uses the base
    /// params; the per-`(ip, client_id)` bucket gets the NAT bonus so a
    /// shared-NAT client is never punished for its neighbors.
    fn tier_params(&self, tier: Tier) -> (f64, f64, f64, f64) {
        let (cap, flow) = match tier {
            Tier::Download => (self.download_capacity, self.download_flow),
            Tier::Auth => (self.auth_capacity, self.auth_flow),
            Tier::Search => (self.search_capacity, self.search_flow),
            Tier::Default => (self.global_capacity, self.global_flow),
        };
        // Shadowbanned clients get a much stricter download bucket (friction,
        // not a hard block): 5 downloads/hour by default.
        if tier == Tier::Download {
            (cap, flow, self.shadowban_capacity, self.shadowban_flow)
        } else {
            (cap, flow, cap, flow)
        }
    }

    /// Effective per-IP bucket when the request carries a client_id: scale by
    /// `nat_multiplier` so one abusive identified client cannot exhaust the
    /// whole IP's allowance, while still capping the IP as a whole.
    fn nat_scaled_capacity(&self, tier: Tier) -> f64 {
        let (cap, _, _, _) = self.tier_params(tier);
        if self.nat_multiplier > 1.0 {
            cap * self.nat_multiplier
        } else {
            cap
        }
    }

    /// Enforce the tiered limit for a request. Bucket keys:
    ///   `rate:tier:{tier}:ip:{ip}`
    ///   `rate:tier:{tier}:client:{ip}:{client_id}` (only when client_id present)
    ///
    /// A shadowbanned client_id is routed to the shadowban bucket for the
    /// download tier (its `(ip, client_id)` key uses the stricter params).
    pub async fn check_tiered(
        &self,
        ip: IpAddr,
        client_id: Option<&str>,
        tier: Tier,
    ) -> TieredRateLimitResult {
        if !self.tiered_enabled || !self.dynamic_rate_limit {
            // Test/dev mode: legacy static delay, always allowed.
            let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
            return TieredRateLimitResult::Allowed;
        }

        // Datacenter / hosting IPs are blocked outright (scraper/bot egress).
        // Fail-open: no GeoIP DB loaded → skip the check. A 1-hour
        // retry-after is effectively a hard block via the existing rate-limit
        // error path (avoids adding a new enum variant across 11 call sites).
        if let Some(geo) = &self.geoip {
            if geo.asn_org_is_datacenter(ip) {
                return TieredRateLimitResult::Wait(DATACENTER_BLOCK_SECS);
            }
        }

        let is_shadowbanned = match client_id {
            Some(c) => self.is_shadowbanned(c),
            None => false,
        };
        let (_, _, shadow_cap, shadow_flow) = self.tier_params(tier);

        // Per-IP bucket first (the NAT ceiling). Identified clients get a
        // scaled ceiling so one client can't starve the IP.
        let ip_cap = if client_id.is_some() {
            self.nat_scaled_capacity(tier)
        } else {
            let (cap, _, _, _) = self.tier_params(tier);
            cap
        };
        let ip_flow = {
            let (_, flow, _, _) = self.tier_params(tier);
            flow
        };
        let ip_key = format!("rate:tier:{:?}:ip:{}", tier, ip);
        let ip_wait = self
            .check_bucket(&ip_key, ip_cap, ip_flow)
            .await
            .unwrap_or(-1.0);
        if ip_wait > 0.0 {
            return TieredRateLimitResult::Wait(ip_wait.ceil() as u64);
        }

        // Per-(ip, client_id) bucket. The download tier uses the shadowban
        // params when the client is flagged; other tiers use the bonus params.
        if let Some(cid) = client_id {
            let client_key = format!("rate:tier:{:?}:client:{}:{}", tier, ip, cid);
            let (client_cap, client_flow) = if tier == Tier::Download && is_shadowbanned {
                (shadow_cap, shadow_flow)
            } else {
                (
                    ip_cap + self.client_bonus_capacity,
                    ip_flow + self.client_bonus_flow,
                )
            };
            let client_wait = self
                .check_bucket(&client_key, client_cap, client_flow)
                .await
                .unwrap_or(-1.0);
            if client_wait > 0.0 {
                return TieredRateLimitResult::Wait(client_wait.ceil() as u64);
            }
        }

        TieredRateLimitResult::Allowed
    }

    // ── Shadowban set (Redis SET `fichub:shadowban` of client_ids) ─────

    /// True when `client_id` is in the shadowban set (expiring entries).
    /// Lazy per-member expiry: a member whose per-member TTL key is gone is
    /// pruned on the next probe, keeping SISMEMBER fast and the set bounded.
    ///
    /// NOTE: blocking — runs the quick single-roundtrip Redis probe on the
    /// current thread (this is the sync impl used by the trait). Errors
    /// return `false` (fail-open).
    pub fn is_shadowbanned(&self, client_id: &str) -> bool {
        self.block_on_sismember(client_id)
    }

    /// Add a client_id to the shadowban set for `ttl_seconds` (24h default).
    /// The SET itself never expires — individual members do — so the set is
    /// cheap to probe and self-cleaning. NOTE: blocking, like
    /// `is_shadowbanned`.
    pub fn shadowban(&self, client_id: &str, ttl_seconds: u64) {
        let ttl = if ttl_seconds > 0 { ttl_seconds } else { self.shadowban_ttl };
        let _ = self.block_on_sadd(client_id, ttl);
    }

    /// Remove a client_id from the shadowban set (admin un-shadowban).
    /// NOTE: blocking, like `is_shadowbanned`.
    pub fn unshadowban(&self, client_id: &str) {
        let _ = self.block_on_srem(client_id);
    }
}

// Shadowban helpers used by the TieredRateLimiter impl below. Because the
// trait methods are non-async, the Redis calls run in a blocking wrapper on
// the current thread (they are quick single-roundtrip SET ops).

impl RedisBucketLimiter {
    /// Blocking `SISMEMBER` probe (fail-open: `false` on error).
    /// Runs on the current thread (single quick Redis roundtrip) — callers
    /// are the sync trait methods invoked from async handlers.
    fn block_on_sismember(&self, client_id: &str) -> bool {
        let mut conn = self.redis.clone();
        // The trait surface is sync; drive the future to completion inline.
        let mut cmd = redis::cmd("SISMEMBER");
        cmd.arg(SHADOWBAN_SET).arg(client_id);
        let fut = cmd.query_async::<bool>(&mut conn);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        })
        .unwrap_or(false)
    }

    /// Blocking `SADD` + per-member TTL (fail-open: `()` on error).
    fn block_on_sadd(&self, client_id: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let member_key = format!("{}:member:{}", SHADOWBAN_SET, client_id);
        let mut pipe = redis::pipe();
        pipe.cmd("SADD")
            .arg(SHADOWBAN_SET)
            .arg(client_id)
            .cmd("PEXPIRE")
            .arg(&member_key)
            .arg((ttl_seconds * 1000) as i64);
        let fut = pipe.query_async::<()>(&mut conn);
        let _: Result<(), redis::RedisError> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        });
        Ok(())
    }

    /// Blocking `SREM` from the shadowban set (fail-open: `()` on error).
    fn block_on_srem(&self, client_id: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let mut cmd = redis::cmd("SREM");
        cmd.arg(SHADOWBAN_SET).arg(client_id);
        let fut = cmd.query_async::<i64>(&mut conn);
        let _: Result<i64, redis::RedisError> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        });
        Ok(())
    }
}

#[async_trait::async_trait]
impl RateLimiter for RedisBucketLimiter {
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
        if !self.dynamic_rate_limit {
            // Simple static delay
            let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
            return RateLimitResult::Allowed;
        }

        // Check datacenter IPs
        if self.is_datacenter_ip(ip) {
            return RateLimitResult::Blocked;
        }

        // Check global bucket
        let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
            .await
            .unwrap_or(-1.0);

        if global_wait > 0.0 {
            return RateLimitResult::Wait(global_wait.ceil() as u64);
        }

        // Check per-IP bucket
        let ip_key = format!("rate:ip:{}", ip);
        let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
            .await
            .unwrap_or(-1.0);

        if ip_wait > 0.0 {
            return RateLimitResult::Wait(ip_wait.ceil() as u64);
        }

        RateLimitResult::Allowed
    }

    async fn report_failure(&self, ip: IpAddr) {
        let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
        let ip_key = format!("rate:ip:{}", ip);
        let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
    }

    fn is_datacenter_ip(&self, ip: IpAddr) -> bool {
        // Best-effort check: consult the MaxMind GeoIP DB first, then the
        // legacy flat-file set (loaded from IP_TAG_SOURCES).
        is_datacenter_checked(
            self.geoip.as_ref(),
            &self.datacenter_ips.try_read().map(|s| s.clone()).unwrap_or_default(),
            ip,
        )
    }
}

/// Pure datacenter decision shared by the trait method and unit tests.
/// Fail-open: `None` GeoIP means no GeoIP-based match; the legacy flat-file
/// set still applies if populated.
fn is_datacenter_checked(
    geoip: Option<&crate::limiter::geoip::GeoIp>,
    flat_file_ips: &HashSet<IpAddr>,
    ip: IpAddr,
) -> bool {
    if let Some(geo) = geoip {
        if geo.asn_org_is_datacenter(ip) {
            return true;
        }
    }
    flat_file_ips.contains(&ip)
}

#[async_trait::async_trait]
impl TieredRateLimiter for RedisBucketLimiter {
    async fn check(&self, ip: IpAddr, client_id: Option<&str>, tier: Tier) -> TieredRateLimitResult {
        self.check_tiered(ip, client_id, tier).await
    }

    fn tier_for_path(&self, path: &str) -> Tier {
        tier_for_path(path)
    }

    fn is_shadowbanned(&self, client_id: &str) -> bool {
        self.is_shadowbanned(client_id)
    }

    fn shadowban(&self, client_id: &str, ttl_seconds: u64) {
        self.shadowban(client_id, ttl_seconds)
    }

    fn unshadowban(&self, client_id: &str) {
        self.unshadowban(client_id)
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    // Pure-Rust token bucket implementation matching the Lua script logic.
    #[derive(Clone)]
    struct TokenBucket {
        value: f64,
        last_drain: f64,
        capacity: f64,
        flow: f64,
    }

    impl TokenBucket {
        fn new(capacity: f64, flow: f64, now: f64) -> Self {
            TokenBucket {
                value: capacity,
                last_drain: now,
                capacity,
                flow,
            }
        }

        /// Request tokens. Returns -1 if allowed, or wait time in seconds.
        fn request(&mut self, requested: f64, now: f64) -> f64 {
            let elapsed = now - self.last_drain;
            let new_tokens = (self.value + elapsed * self.flow).min(self.capacity);
            let allowed = new_tokens - requested;

            if allowed >= 0.0 {
                self.value = allowed;
                self.last_drain = now;
                -1.0
            } else {
                (requested - new_tokens) / self.flow
            }
        }

        /// Penalize by requesting 1.5 extra tokens.
        fn penalize(&mut self, now: f64) {
            self.request(1.5, now);
        }
    }

    #[test]
    fn test_initial_bucket_fill() {
        let mut bucket = TokenBucket::new(100.0, 10.0, 0.0);
        assert!((bucket.value - 100.0).abs() < f64::EPSILON);
        // Consume 10 at t=0 -> allowed, value=90
        let result = bucket.request(10.0, 0.0);
        assert!((result - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_consumption() {
        let mut bucket = TokenBucket::new(50.0, 5.0, 0.0);
        let r = bucket.request(20.0, 0.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exact_tokens_available() {
        let mut bucket = TokenBucket::new(10.0, 1.0, 0.0);
        let r = bucket.request(10.0, 0.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_refill_over_time() {
        let mut bucket = TokenBucket::new(50.0, 10.0, 0.0);
        // Empty the bucket
        bucket.request(50.0, 0.0);
        assert!((bucket.value - 0.0).abs() < f64::EPSILON);

        // After 3s -> 30 tokens refilled (capped at 50)
        let r = bucket.request(20.0, 3.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 10.0).abs() < f64::EPSILON); // 30 - 20 = 10
    }

    #[test]
    fn test_capacity_limits_refill() {
        let mut bucket = TokenBucket::new(50.0, 100.0, 0.0);
        // Empty
        bucket.request(50.0, 0.0);
        // After 10s: flow adds 1000, but cap at 50
        // Request 60 -> should wait
        let wait = bucket.request(60.0, 10.0);
        assert!(wait > 0.0);
        let expected = (60.0 - 50.0) / 100.0;
        assert!((wait - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wait_calculation() {
        let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
        // Empty the bucket
        bucket.request(10.0, 0.0);
        // Request 5 at same time -> need 5 tokens, flow=2/sec
        let wait = bucket.request(5.0, 0.0);
        let expected = 5.0 / 2.0;
        assert!((wait - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_requests_over_time() {
        let mut bucket = TokenBucket::new(20.0, 5.0, 0.0);

        // t=0: use 10, value=10
        let r = bucket.request(10.0, 0.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);

        // t=2: refilled 10 → min(20, 10 + 2*5) = 20, use 15, value=5
        let r = bucket.request(15.0, 2.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 5.0).abs() < f64::EPSILON);

        // t=5: refilled 5 + 3*5 = 20 (capped), use all 20
        let r = bucket.request(20.0, 5.0);
        assert!((r - (-1.0)).abs() < f64::EPSILON);
        assert!((bucket.value - 0.0).abs() < f64::EPSILON);

        // t=6: only 5 tokens refilled, request 10 -> wait 1s
        let wait = bucket.request(10.0, 6.0);
        let expected = (10.0 - 5.0) / 5.0;
        assert!((wait - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penalize_reduces_tokens() {
        let mut bucket = TokenBucket::new(10.0, 5.0, 0.0);
        bucket.request(8.0, 0.0);
        let before = bucket.value;
        bucket.penalize(0.0);
        assert!((bucket.value - (before - 1.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penalize_on_empty_bucket_does_not_modify() {
        let mut bucket = TokenBucket::new(5.0, 1.0, 0.0);
        // Completely empty
        bucket.request(5.0, 0.0);
        // Penalize when empty: allowed = 0 - 1.5 < 0, so no update
        bucket.penalize(0.0);
        // Value unchanged (still 0)
        assert!((bucket.value - 0.0).abs() < f64::EPSILON);
    }

    // ── Tiered rate-limit math (pure Rust, mirrors the Lua bucket) ─────

    /// Drive a tier bucket over a wall-clock-free simulated timeline.
    /// Simulates up to `max_seconds` of 1-request-per-second hammering and
    /// returns the time (in seconds) when the bucket first refuses a request,
    /// or `max_seconds` if the tier survives the whole window.
    fn tier_exhaustion_time(capacity: f64, flow_per_sec: f64, burst: u64, _limit_per_hour: f64, max_seconds: f64) -> f64 {
        let mut bucket = TokenBucket::new(capacity, flow_per_sec, 0.0);
        let mut now = 0.0;
        for _ in 0..burst {
            let wait = bucket.request(1.0, now);
            assert!(wait <= 0.0, "burst requests must be allowed");
            now += 0.001;
        }
        // Simulate an attacker hammering once per second (bounded loop).
        while now < max_seconds {
            now += 1.0;
            let wait = bucket.request(1.0, now);
            if wait > 0.0 {
                return now;
            }
        }
        now
    }

    #[test]
    fn test_download_tier_60_per_hour_burst_10() {
        // Defaults: capacity 10 (burst), flow 60/hr = 0.0167/sec.
        // Hammering at 1/sec with a 10-token bucket: net drain ≈ 0.983/sec,
        // so exhaustion ≈ 10 / 0.983 ≈ 10 seconds (not an hour — 1/sec is
        // 60x the tier's 60/hr sustained rate).
        let t = tier_exhaustion_time(
            super::DEFAULT_DOWNLOAD_CAPACITY,
            super::DEFAULT_DOWNLOAD_FLOW,
            10,
            60.0,
            7200.0,
        );
        assert!(t >= 1.0 && t < 10.0, "exhaustion at {t:.0}s (burst drains the 10-token bucket; next 1/sec request is refused)");
    }

    #[test]
    fn test_auth_tier_10_per_minute() {
        // 10/min = 600/hr = 0.167/sec. 1/sec hammering drains the 10-token
        // bucket in ≈ 10 / 0.833 ≈ 12s.
        let t = tier_exhaustion_time(
            super::DEFAULT_AUTH_CAPACITY,
            super::DEFAULT_AUTH_FLOW,
            10,
            600.0,
            7200.0,
        );
        assert!(t >= 1.0 && t < 10.0, "exhaustion at {t:.0}s (burst drains the 10-token bucket; next 1/sec request is refused)");
    }

    #[test]
    fn test_search_tier_loose() {
        // 1000/min = 16.67/sec. 1/sec hammering NEVER exhausts (the bucket
        // refills 16.67/sec and only 1/sec is consumed), so the simulation
        // runs to the 2h cap and the tier survives.
        let t = tier_exhaustion_time(
            super::DEFAULT_SEARCH_CAPACITY,
            super::DEFAULT_SEARCH_FLOW,
            1000,
            60000.0,
            7200.0,
        );
        assert!(t >= 7200.0, "search tier must survive 1/sec hammering for the whole 2h window (got {t:.0}s)");
    }

    #[test]
    fn test_shadowban_tier_strict() {
        // 5/hour = 0.00139/sec. 1/sec hammering drains the 5-token bucket in
        // ≈ 5 / 0.9986 ≈ 5 seconds — very strict (that's the point: a
        // shadowbanned client gets ~5 downloads, then friction).
        let t = tier_exhaustion_time(
            super::DEFAULT_SHADOWBAN_CAPACITY,
            super::DEFAULT_SHADOWBAN_FLOW,
            5,
            5.0,
            7200.0,
        );
        assert!(t >= 1.0 && t < 10.0, "exhaustion at {t:.0}s (burst drains the 5-token bucket; next 1/sec request is refused)");
    }

    #[test]
    fn test_tier_keys_are_distinct() {
        // (ip) and (ip, client_id) keys must never collide.
        let ip_key = format!("rate:tier:Download:ip:{}", "203.0.113.7");
        let client_key = format!("rate:tier:Download:client:{}:{}", "203.0.113.7", "reader-1");
        assert_ne!(ip_key, client_key);
        // Different client_ids on the same IP are distinct too.
        let client_key2 = format!("rate:tier:Download:client:{}:{}", "203.0.113.7", "reader-2");
        assert_ne!(client_key, client_key2);
    }

    #[test]
    fn test_tier_for_path_mapping() {
        use super::Tier;
        assert_eq!(super::tier_for_path("/api/epub?q=x"), Tier::Download);
        assert_eq!(super::tier_for_path("/api/v0/epub"), Tier::Download);
        assert_eq!(super::tier_for_path("/api/download/author"), Tier::Download);
        assert_eq!(super::tier_for_path("/cache/epub/abc123"), Tier::Download);
        assert_eq!(super::tier_for_path("/api/upload"), Tier::Download);
        assert_eq!(super::tier_for_path("/api/auth/login"), Tier::Auth);
        assert_eq!(super::tier_for_path("/api/auth/register"), Tier::Auth);
        assert_eq!(super::tier_for_path("/login"), Tier::Auth);
        assert_eq!(super::tier_for_path("/register"), Tier::Auth);
        assert_eq!(super::tier_for_path("/api/search?q=harry"), Tier::Search);
        assert_eq!(super::tier_for_path("/docs"), Tier::Search);
        assert_eq!(super::tier_for_path("/static/foo.css"), Tier::Search);
        assert_eq!(super::tier_for_path("/api/recommendations"), Tier::Default);
        assert_eq!(super::tier_for_path("/"), Tier::Default);
    }

    #[test]
    fn test_datacenter_block_requires_geoip() {
        // Fail-open: with no GeoIP database configured and no flat-file IPs,
        // no IP is treated as a datacenter.
        let empty: HashSet<std::net::IpAddr> = HashSet::new();
        let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST);
        assert!(
            !super::is_datacenter_checked(None, &empty, ip),
            "fail-open: no geoip + no flat-file set must not block"
        );
        // Legacy flat-file set still honored when populated.
        let mut set = HashSet::new();
        set.insert(ip);
        assert!(super::is_datacenter_checked(None, &set, ip));
        // GeoIP takes precedence: with a reader that cannot match, a
        // flat-file hit still wins via the legacy path.
        assert!(!super::is_datacenter_checked(None, &empty, ip));
    }

    /// Pure-Rust model of the tiered decision: per-IP bucket then
    /// per-(ip, client_id) bucket; shadowbanned clients get the strict bucket
    /// on the download tier.
    #[allow(dead_code)]
    struct TieredModel {
        ip_bucket: TokenBucket,
        client_bucket: TokenBucket,
        ip_capacity: f64,
        client_capacity: f64,
    }

    impl TieredModel {
        fn new(ip_capacity: f64, ip_flow: f64, client_capacity: f64, client_flow: f64) -> Self {
            TieredModel {
                ip_bucket: TokenBucket::new(ip_capacity, ip_flow, 0.0),
                client_bucket: TokenBucket::new(client_capacity, client_flow, 0.0),
                ip_capacity,
                client_capacity,
            }
        }

        fn allow(&mut self, now: f64) -> bool {
            if self.ip_bucket.request(1.0, now) > 0.0 {
                return false;
            }
            self.client_bucket.request(1.0, now) <= 0.0
        }

        fn allow_shadow(&mut self, now: f64, shadow_capacity: f64, shadow_flow: f64) -> bool {
            if self.ip_bucket.request(1.0, now) > 0.0 {
                return false;
            }
            // Shadowbanned client → its own bucket uses the strict params.
            if self.client_bucket.capacity != shadow_capacity {
                self.client_bucket = TokenBucket::new(shadow_capacity, shadow_flow, now);
            }
            self.client_bucket.request(1.0, now) <= 0.0
        }
    }

    #[test]
    fn test_nat_bonus_not_punishing_shared_ip() {
        // Two identified clients behind one NAT IP. Each client bucket gets
        // bonus tokens, so neither starves the other.
        let nat_ip_cap = super::DEFAULT_DOWNLOAD_CAPACITY * super::DEFAULT_NAT_MULTIPLIER; // 40
        let client_cap = super::DEFAULT_DOWNLOAD_CAPACITY + super::DEFAULT_CLIENT_BONUS_CAPACITY; // 15
        let mut a = TieredModel::new(nat_ip_cap, super::DEFAULT_DOWNLOAD_FLOW, client_cap, super::DEFAULT_CLIENT_BONUS_FLOW);
        let mut b = TieredModel::new(nat_ip_cap, super::DEFAULT_DOWNLOAD_FLOW, client_cap, super::DEFAULT_CLIENT_BONUS_FLOW);

        let mut now = 0.0;
        let mut a_ok = 0;
        let mut b_ok = 0;
        for _ in 0..10 {
            if a.allow(now) { a_ok += 1; }
            if b.allow(now) { b_ok += 1; }
            now += 1.0;
        }
        // Each client gets its own 10-burst + 5 bonus; IP ceiling is 40.
        assert!(a_ok >= 10, "client A must get its burst (got {a_ok})");
        assert!(b_ok >= 10, "client B must get its burst (got {b_ok})");
    }

    #[test]
    fn test_shadowbanned_client_gets_strict_bucket() {
        let ip_cap = super::DEFAULT_DOWNLOAD_CAPACITY * super::DEFAULT_NAT_MULTIPLIER;
        let client_cap = super::DEFAULT_DOWNLOAD_CAPACITY + super::DEFAULT_CLIENT_BONUS_CAPACITY;
        let mut m = TieredModel::new(ip_cap, super::DEFAULT_DOWNLOAD_FLOW, client_cap, super::DEFAULT_CLIENT_BONUS_FLOW);

        let mut now = 0.0;
        let mut normal_ok = 0;
        for _ in 0..10 {
            if m.allow(now) { normal_ok += 1; }
            now += 1.0;
        }
        // Reset for the shadowed client: it gets a fresh strict bucket.
        let mut s = TieredModel::new(ip_cap, super::DEFAULT_DOWNLOAD_FLOW, 0.0, 0.0);
        let mut shadow_ok = 0;
        for _ in 0..10 {
            if s.allow_shadow(now, super::DEFAULT_SHADOWBAN_CAPACITY, super::DEFAULT_SHADOWBAN_FLOW) {
                shadow_ok += 1;
            }
            now += 1.0;
        }
        // Normal identified client: 10+ burst ok. Shadowbanned: capped at 5.
        assert!(normal_ok >= 10, "normal client burst (got {normal_ok})");
        assert!(shadow_ok <= 5, "shadowbanned client capped at 5/hr (got {shadow_ok})");
    }
}
