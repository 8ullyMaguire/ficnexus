pub mod geoip;
pub mod redis_bucket;

use std::net::IpAddr;

/// Per-request rate limiting result
#[derive(Debug)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Wait this many seconds before retrying
    Wait(u64),
    /// Blocked (datacenter IP, etc.)
    Blocked,
}

/// Rate limiter trait
#[async_trait::async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}

/// Endpoint-class tiers for the tiered rate limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// /api/epub, /api/v0/epub, /api/download/*, /cache/*, /api/upload* —
    /// strict (60/hr per IP by default).
    Download,
    /// /api/auth/*, /login, /register — strict (10/min per IP).
    Auth,
    /// /api/search, /docs, /static — very high (1000/min).
    Search,
    /// Everything else — moderate (the legacy global bucket).
    Default,
}

/// Result of a tiered rate-limit check.
#[derive(Debug)]
pub enum TieredRateLimitResult {
    Allowed,
    /// Wait this many seconds before retrying (HTTP 429 `retry_after`).
    Wait(u64),
}

/// Tiered rate limiting over the Redis token buckets, with per-IP AND
/// per-`(ip, client_id)` keys and a Redis SET (`fichub:shadowban`) of flagged
/// client_ids. Implemented by `RedisBucketLimiter`.
#[async_trait::async_trait]
pub trait TieredRateLimiter: Send + Sync {
    /// Check a request against the tier bucket for `tier`. `client_id` is the
    /// `X-Client-Id` header value when present.
    async fn check(&self, ip: IpAddr, client_id: Option<&str>, tier: Tier)
    -> TieredRateLimitResult;

    /// Map a request path to its tier.
    fn tier_for_path(&self, path: &str) -> Tier;

    /// True when `client_id` is shadowbanned (in the `fichub:shadowban` set).
    fn is_shadowbanned(&self, client_id: &str) -> bool;

    /// Add a client_id to the shadowban set for `ttl_seconds` (friction, not a
    /// hard block: shadowbanned clients get a much stricter download bucket).
    fn shadowban(&self, client_id: &str, ttl_seconds: u64);

    /// Remove a client_id from the shadowban set (admin un-shadowban).
    fn unshadowban(&self, client_id: &str);
}

/// Extract the real client IP from the request headers.
///
/// Nginx (the only proxy in this deployment) sets `X-Forwarded-For` and
/// overwrites any client-supplied value, so the FIRST hop is the client. When
/// the header is absent (or unparsable) we fall back to the peer address from
/// the transport. This is the same convention used by `request_log` in
/// `src/routes/export.rs` (Zero-PII: the IP is only used for abuse
/// aggregation, never exposed to admins).
pub fn client_ip_from_headers(xff: Option<&str>, remote_addr: IpAddr) -> IpAddr {
    xff.and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
        .unwrap_or(remote_addr)
}
