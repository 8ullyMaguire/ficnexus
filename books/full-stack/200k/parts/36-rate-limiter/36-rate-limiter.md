# Part 36 — Rate Limiter and Redis

> In this chapter you will learn how FicHub protects its servers from scraper bots and abusive clients using a Redis-backed token bucket rate limiter. We'll build a tiered system that gives generous limits to humans, stricter limits to datacenter IPs, and a proof-of-work challenge for shadowbanned clients.

---

## Overview

FicHub serves a lot of traffic — millions of EPUB downloads, thousands of search queries, and bots constantly scraping the site. A naive rate limiter (one request per N seconds per IP) would block legitimate users behind shared NATs or mobile carriers. Instead, FicHub uses a **tiered Redis token bucket** system:

- **Tier 1 (Download)**: `/api/epub`, `/api/download/*`, `/cache/*` — 10 requests burst, refills at 60/hour. Strict because EPUBs are expensive to generate.
- **Tier 2 (Auth)**: `/api/auth/*`, `/login`, `/register` — 10 requests burst, refills at 10/minute. Prevents credential-stuffing.
- **Tier 3 (Search)**: `/api/search`, `/docs`, `/static` — 1000/minute. Very loose because search is cheap.
- **Tier 4 (Default)**: Everything else — moderate, shared bucket.

On top of tiers, FicHub adds:

- **Datacenter IP blocking**: requests from AWS/DO/Vultr IPs are hard-blocked (1-hour retry-after) using MaxMind GeoLite2-ASN.
- **Shadowban set**: repeat offenders get flagged in a Redis SET. Shadowbanned clients get a stricter 5/hour download bucket.
- **PoW challenge**: shadowbanned clients must solve a hashcash puzzle before downloading.
- **NAT bonus**: identified clients (with `X-Client-ID` header) get their own bucket so one abusive client on a shared IP can't starve the whole IP.

In test/dev mode (`dynamic_rate_limit == false`), the limiter applies a simple static random delay — no Redis needed.

---

## Prerequisites

- You have completed Parts 1–5 (booting the server, database, routes).
- Redis is running and `REDIS_URL` is set in your environment.
- You have the `redis-cli` tool available.
- Your dev mode has `DYNAMIC_RATE_LIMIT=false` so you can test without Redis initially.

---

## Chapter 36.1 — Rate Limiter Architecture

### Goal

Understand the trait design: `TieredRateLimiter` with per-path tier mapping, and how `RedisBucketLimiter` implements it.

### Actions

#### 1. The Tier enum

Defined in `src/limiter/mod.rs`:

```rust
// src/limiter/mod.rs
/// Endpoint-class tiers for the tiered rate limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// /api/epub, /api/v0/epub, /api/download/*, /cache/*, /api/upload*
    Download,
    /// /api/auth/*, /login, /register
    Auth,
    /// /api/search, /docs, /static
    Search,
    /// Everything else — moderate (the legacy global bucket)
    Default,
}

/// Result of a tiered rate-limit check.
#[derive(Debug)]
pub enum TieredRateLimitResult {
    Allowed,
    /// Wait this many seconds before retrying (HTTP 429 `retry_after`).
    Wait(u64),
}

/// Tiered rate limiting over the Redis token buckets.
#[async_trait::async_trait]
pub trait TieredRateLimiter: Send + Sync {
    async fn check(&self, ip: IpAddr, client_id: Option<&str>, tier: Tier)
        -> TieredRateLimitResult;
    fn tier_for_path(&self, path: &str) -> Tier;
    fn is_shadowbanned(&self, client_id: &str) -> bool;
    fn shadowban(&self, client_id: &str, ttl_seconds: u64);
    fn unshadowban(&self, client_id: &str);
}

/// Extract the real client IP from request headers.
pub fn client_ip_from_headers(
    xff: Option<&str>,
    remote_addr: IpAddr,
) -> IpAddr {
    xff.and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
        .unwrap_or(remote_addr)
}
```

#### 2. Path-to-tier mapping

The `tier_for_path` function is a pure function — no state, easy to unit test:

```rust
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
```

#### 3. Rate limit tiers at a glance

| Tier | Paths | Burst | Refill | Config fields |
|------|-------|-------|--------|---------------|
| Download | `/api/epub`, `/api/download/*`, `/cache/*` | 10 | 60/hour | `rl_download_capacity`, `rl_download_flow` |
| Auth | `/api/auth/*`, `/login`, `/register` | 10 | 10/minute | `rl_auth_capacity`, `rl_auth_flow` |
| Search | `/api/search`, `/docs`, `/static` | 1000 | 1000/minute | `rl_search_capacity`, `rl_search_flow` |
| Default | everything else | 150 | 30/minute | `global_capacity`, `global_flow` (legacy) |
| Shadowban Download | only for shadowbanned clients | 5 | 5/hour | `rl_shadowban_capacity`, `rl_shadowban_flow` |

### Try It Yourself

Run the unit tests for `tier_for_path`:

```bash
cargo test tier_for_path -- --nocapture
```

### Check

- ✅ `tier_for_path("/api/epub?q=x")` returns `Tier::Download`.
- ✅ `tier_for_path("/api/auth/login")` returns `Tier::Auth`.
- ✅ `tier_for_path("/api/search?q=harry")` returns `Tier::Search`.
- ✅ `tier_for_path("/")` returns `Tier::Default`.

### What you built

The tiered architecture: a `Tier` enum, a `TieredRateLimiter` trait, and a pure function that maps request paths to tiers.

---

## Chapter 36.2 — Redis Token Bucket

### Goal

Implement the actual rate-limiting logic using Lua scripts in Redis. Each bucket is an atomic operation — the Lua script reads, refills, and writes the bucket in one round-trip.

### Actions

#### 1. The RedisBucketLimiter struct

Defined in `src/limiter/redis_bucket.rs`:

```rust
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{RateLimiter, RateLimitResult, TieredRateLimiter, TieredRateLimitResult, Tier};

pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,          // SHA of loaded Lua script
    dynamic_rate_limit: bool, // false = test/dev mode (static delay)
    static_delay_base: f64,    // seconds — only used in test/dev
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    geoip: Option<crate::limiter::geoip::GeoIp>,
    // Tiered configuration
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
```

#### 2. Default tier parameters

```rust
pub const DEFAULT_DOWNLOAD_CAPACITY: f64 = 10.0;
pub const DEFAULT_DOWNLOAD_FLOW: f64 = 60.0 / 3600.0;  // 60/hour → tokens/sec
pub const DEFAULT_AUTH_CAPACITY: f64 = 10.0;
pub const DEFAULT_AUTH_FLOW: f64 = 10.0 / 60.0;        // 10/min
pub const DEFAULT_SEARCH_CAPACITY: f64 = 1000.0;
pub const DEFAULT_SEARCH_FLOW: f64 = 1000.0 / 60.0;    // 1000/min
pub const DEFAULT_CLIENT_BONUS_CAPACITY: f64 = 5.0;    // extra burst for identified clients
pub const DEFAULT_CLIENT_BONUS_FLOW: f64 = 30.0 / 3600.0; // 30/hour bonus
pub const DEFAULT_NAT_MULTIPLIER: f64 = 4.0;           // NAT IP gets 4x ceiling
pub const DEFAULT_SHADOWBAN_CAPACITY: f64 = 5.0;       // 5/hr after shadowban
pub const DEFAULT_SHADOWBAN_FLOW: f64 = 5.0 / 3600.0;
pub const DEFAULT_SHADOWBAN_TTL: u64 = 86400;          // 24h
pub const SHADOWBAN_SET: &str = "fichub:shadowban";
pub const DATACENTER_BLOCK_SECS: u64 = 3600;           // 1h hard block
```

#### 3. The Lua token bucket script

The core of the rate limiter is a Lua script loaded once into Redis via `SCRIPT LOAD`. The script implements a standard token bucket: refill tokens based on elapsed time, cap at capacity, subtract the request, and either allow (return -1) or tell the client how long to wait:

```rust
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
```

#### 4. Loading the script

```rust
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;
```

#### 5. Checking a bucket

```rust
async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) -> Result<f64, redis::RedisError> {
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)        // number of KEYS
        .arg(key)      // KEYS[1]
        .arg(1.0)      // requested tokens
        .arg(capacity) // ARGV[2]
        .arg(flow)     // ARGV[3]
        .query_async(&mut conn)
        .await?;
    Ok(result)  // -1.0 if allowed, wait-time if not
}
```

#### 6. The tiered check

`check_tiered` does the full dance: datacenter check, per-IP bucket, per-`(ip, client_id)` bucket, shadowban routing.

```rust
pub async fn check_tiered(
    &self,
    ip: IpAddr,
    client_id: Option<&str>,
    tier: Tier,
) -> TieredRateLimitResult {
    // 1. Datacenter IPs are hard-blocked
    if let Some(geo) = &self.geoip {
        if geo.asn_org_is_datacenter(ip) {
            return TieredRateLimitResult::Wait(DATACENTER_BLOCK_SECS);  // 3600s
        }
    }

    let is_shadowbanned = match client_id {
        Some(c) => self.is_shadowbanned(c),
        None => false,
    };

    // 2. Per-IP bucket (the NAT ceiling)
    let ip_cap = if client_id.is_some() {
        self.nat_scaled_capacity(tier)  // capacity * nat_multiplier
    } else {
        let (cap, _, _, _) = self.tier_params(tier);
        cap
    };
    let ip_flow = { let (_, flow, _, _) = self.tier_params(tier); flow };
    let ip_key = format!("rate:tier:{:?}:ip:{}", tier, ip);
    let ip_wait = self.check_bucket(&ip_key, ip_cap, ip_flow).await.unwrap_or(-1.0);
    if ip_wait > 0.0 {
        return TieredRateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    // 3. Per-(ip, client_id) bucket (only when client_id present)
    if let Some(cid) = client_id {
        let client_key = format!("rate:tier:{:?}:client:{}:{}", tier, ip, cid);
        let (client_cap, client_flow) = if tier == Tier::Download && is_shadowbanned {
            let (_, _, s_cap, s_flow) = self.tier_params(tier);
            (s_cap, s_flow)           // shadowban bucket: 5/hr
        } else {
            (ip_cap + self.client_bonus_capacity, ip_flow + self.client_bonus_flow)
        };
        let client_wait = self.check_bucket(&client_key, client_cap, client_flow)
            .await.unwrap_or(-1.0);
        if client_wait > 0.0 {
            return TieredRateLimitResult::Wait(client_wait.ceil() as u64);
        }
    }

    TieredRateLimitResult::Allowed
}
```

> **💡 Key Concept**: The per-IP bucket is the "NAT ceiling" — it caps traffic from one IP address regardless of how many clients share it. The per-`(ip, client_id)` bucket gives each identified client its own allowance, so one abusive client doesn't punish everyone behind the same router.

### Try It Yourself

In `redis-cli`, inspect the bucket keys after making requests:

```bash
# List all rate-limit keys
redis-cli KEYS "rate:*"

# Inspect a specific bucket
redis-cli HMGET "rate:tier:Download:ip:127.0.0.1" value last_drain

# Check the shadowban set
redis-cli SMEMBERS "fichub:shadowban"
```

### Check

- ✅ `check_bucket` returns `-1.0` when tokens are available, positive wait-time otherwise.
- ✅ `tier_for_path("/api/epub")` returns `Tier::Download` (used to pick bucket params).
- ✅ Datacenter IPs get `Wait(3600)` (1-hour hard block).

### What you built

A Redis-backed token bucket with tiered per-endpoint-class limits, per-IP ceilings, per-client bonuses, datacenter blocking, and shadowban routing.

---

## Chapter 36.3 — GeoIP Datacenter Detection

### Goal

Block datacenter and hosting-provider IPs (AWS, DigitalOcean, Vultr, etc.) that are almost always scrapers or bots. Use MaxMind GeoLite2-ASN with a fail-open design.

### Actions

#### 1. The GeoIp struct

`src/limiter/geoip.rs` opens a MaxMind `.mmdb` file and queries the ASN (Autonomous System Number) organization name:

```rust
// src/limiter/geoip.rs (simplified)
use maxminddb::Reader;
use serde::Deserialize;

pub struct GeoIp {
    reader: Reader<Vec<u8>>,
}

#[derive(Deserialize)]
struct AsnRecord {
    #[serde(rename = "autonomous_system_organization")]
    autonomous_system_organization: Option<String>,
}

impl GeoIp {
    pub fn open(path: &Path) -> Option<Self> {
        Reader::open_readfile(path).ok().map(|reader| GeoIp { reader })
    }

    /// Returns true if the IP belongs to a datacenter/hosting provider.
    pub fn asn_org_is_datacenter(&self, ip: IpAddr) -> bool {
        if let Ok(record) = self.reader.lookup::<AsnRecord>(ip) {
            if let Some(org) = record.autonomous_system_organization {
                let lower = org.to_lowercase();
                // Known datacenter org patterns
                return lower.contains("amazon")
                    || lower.contains("google")
                    || lower.contains("microsoft")
                    || lower.contains("digitalocean")
                    || lower.contains("vultr")
                    || lower.contains("ovh")
                    || lower.contains("linode")
                    || lower.contains("cloud")
                    || lower.contains("hosting");
            }
        }
        false
    }
}
```

#### 2. Fail-open design

If no `MAXMIND_DB` path is configured or the file is unreadable, datacenter blocking is disabled — the site keeps working, just without IP blocking:

```rust
// src/limiter/redis_bucket.rs (in with_config)
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
```

> **⚠️ Watch Out**: The GeoIP DB is updated monthly. Download the latest GeoLite2-ASN MMDB from the MaxMind website and set `MAXMIND_DB=/path/to/GeoLite2-ASN.mmdb` in your `.env`. Without it, all datacenter IPs bypass rate limiting.

#### 3. Legacy flat-file fallback

In addition to MaxMind, the limiter supports a legacy flat-file approach where datacenter IPs are listed one per line in a source file:

```rust
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
}
```

### Try It Yourself

```bash
# Check if an IP is a datacenter
# (Requires the MaxMind DB — in dev mode this is a no-op)
```

### Check

- ✅ With `MAXMIND_DB` set, AWS IPs (`.contains("amazon")`) are blocked with `Wait(3600)`.
- ✅ Without `MAXMIND_DB`, all IPs pass the datacenter check (fail-open).

### What you built

A GeoIP-based datacenter blocker that fails open when the database is missing, plus a legacy flat-file fallback for hardcoded IP lists.

---

## Chapter 36.4 — Shadowban and Client-ID Keying

### Goal

Implement the shadowban set: a Redis SET of flagged client IDs that get stricter rate limits. Also implement the per-`(ip, client_id)` bucket with NAT scaling.

### Actions

#### 1. The X-Client-ID header

The frontend's `client.ts` automatically generates and sends a UUID `X-Client-ID` header on every request:

```typescript
// frontend/src/lib/api/client.ts
function getClientId(): string {
  const STORAGE_KEY = 'fichub_client_id';
  let clientId = localStorage.getItem(STORAGE_KEY);
  if (!clientId) {
    clientId = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
      const r = (Math.random() * 16) | 0;
      const v = c === 'x' ? r : (r & 0x3) | 0x8;
      return v.toString(16);
    });
    localStorage.setItem(STORAGE_KEY, clientId);
  }
  return clientId;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers);
  headers.set('X-Client-ID', getClientId());
  const res = await fetch(`${BASE}${path}`, { ...init, headers });
  // ...
}
```

#### 2. The shadowban SET

Shadowbanned client IDs are stored in a Redis SET with per-member TTL:

```rust
pub const SHADOWBAN_SET: &str = "fichub:shadowban";
pub const DEFAULT_SHADOWBAN_TTL: u64 = 86400;  // 24h

pub fn is_shadowbanned(&self, client_id: &str) -> bool {
    self.block_on_sismember(client_id)
}

pub fn shadowban(&self, client_id: &str, ttl_seconds: u64) {
    let ttl = if ttl_seconds > 0 { ttl_seconds } else { self.shadowban_ttl };
    let _ = self.block_on_sadd(client_id, ttl);
}

pub fn unshadowban(&self, client_id: &str) {
    let _ = self.block_on_srem(client_id);
}
```

The `SADD` + `PEXPIRE` pipeline:

```rust
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
```

#### 3. NAT scaling

When a request carries a `client_id`, the per-IP bucket's capacity is multiplied by `nat_multiplier` (default 4x). This means an IP shared by 4 abusive clients gets 4x the per-client ceiling, so no single client can starve the IP:

```rust
fn nat_scaled_capacity(&self, tier: Tier) -> f64 {
    let (cap, _, _, _) = self.tier_params(tier);
    if self.nat_multiplier > 1.0 {
        cap * self.nat_multiplier
    } else {
        cap
    }
}

fn tier_params(&self, tier: Tier) -> (f64, f64, f64, f64) {
    let (cap, flow) = match tier {
        Tier::Download => (self.download_capacity, self.download_flow),
        Tier::Auth => (self.auth_capacity, self.auth_flow),
        Tier::Search => (self.search_capacity, self.search_flow),
        Tier::Default => (self.global_capacity, self.global_flow),
    };
    // Shadowbanned clients get a stricter download bucket
    if tier == Tier::Download {
        (cap, flow, self.shadowban_capacity, self.shadowban_flow)
    } else {
        (cap, flow, cap, flow)
    }
}
```

### Try It Yourself

```bash
# Shadowban a client
redis-cli SADD fichub:shadowban "malicious-client-id"

# Verify it's in the set
redis-cli SISMEMBER fichub:shadowban "malicious-client-id"

# Un-shadowban
redis-cli SREM fichub:shadowban "malicious-client-id"
```

### Check

- ✅ `shadowban("bad-id", 86400)` adds the client to the Redis SET with a 24h TTL.
- ✅ `is_shadowbanned("bad-id")` returns `true` after shadowbanning.
- ✅ The per-IP bucket for a client_id'd request uses `capacity * nat_multiplier`.

### What you built

A shadowban system with Redis SETs for per-client blocking, plus NAT-aware bucket scaling so shared-IP clients don't punish each other.

---

## Chapter 36.5 — PoW Challenge for Flagged Clients

### Goal

Implement the hashcash-style proof-of-work challenge that shadowbanned clients must solve before downloading. This adds CPU friction — cheap for humans (0.1s of JS), expensive for bulk bots (65k hashes per request).

### Actions

#### 1. The challenge endpoints

`src/routes/pow.rs` — two endpoints: `GET /api/pow/challenge` issues a puzzle, `POST /api/pow/solve` verifies it:

```rust
use axum::{extract::State, http::HeaderMap, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use crate::error::AppError;
use crate::server::AppState;
use crate::services::pow;

#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    pub challenge: String,  // 32 hex chars
    pub nonce: String,      // decimal counter
}

/// `GET /api/pow/challenge` — issue a challenge to a shadowbanned client.
pub async fn challenge_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let Some(cid) = client_id else {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
            "message": "no client id",
        })));
    };

    // Fail-open: Redis hiccup → not shadowbanned → no challenge
    if !state.rate_limiter.is_shadowbanned(&cid) {
        return Ok(Json(json!({ "err": 0, "not_needed": true })));
    }

    let challenge = pow::generate_challenge(
        state.config.pow_difficulty,    // 16 bits = ~65k hashes
        state.config.pow_ttl_secs,      // 600s = 10 min
    );
    Ok(Json(json!({
        "err": 0,
        "not_needed": false,
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}
```

#### 2. The verification logic

`src/services/pow.rs` — pure functions, no Redis needed for the math:

```rust
use rand::RngCore;
use sha2::Digest;

pub const SOLVED_KEY_PREFIX: &str = "fichub:pow:solved:";
pub const CHALLENGE_KEY_PREFIX: &str = "fichub:pow:challenge:";
pub const DEFAULT_POW_TTL_SECS: u64 = 600;  // 10 minutes

pub struct PowChallenge {
    pub challenge: String,  // 16 random bytes hex-encoded
    pub difficulty: u32,    // leading zero bits
    pub expires_at: i64,    // unix timestamp
}

pub fn generate_challenge(difficulty: u32, ttl_secs: u64) -> PowChallenge {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let challenge = hex::encode(bytes);
    let expires_at = chrono::Utc::now().timestamp() + ttl_secs as i64;
    PowChallenge { challenge, difficulty, expires_at }
}

/// Difficulty N → ceil(N/4) zero hex chars. Difficulty 16 → "0000".
pub fn difficulty_to_hex_prefix(difficulty: u32) -> String {
    let nibbles = difficulty.div_ceil(4);
    "0".repeat(nibbles as usize)
}

/// SHA-256(challenge || nonce) must start with `difficulty` zero bits.
pub fn verify_solution(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        return nonce.chars().all(|c| c.is_ascii_digit());
    }
    if !nonce.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let hex_digest = hex::encode(hasher.finalize());
    hex_digest.starts_with(&difficulty_to_hex_prefix(difficulty))
}
```

#### 3. Verify at the export gate

When a shadowbanned client requests `/api/epub`, the handler checks if the challenge was solved:

```rust
/// In src/routes/export.rs — the epub_handler checks PoW before serving.
// (Simplified — the full export handler is ~1400 lines)
let client_id = headers
    .get("x-client-id")
    .and_then(|v| v.to_str().ok())
    .map(|s| s.trim().to_string());

if let Some(cid) = &client_id {
    if state.rate_limiter.is_shadowbanned(cid) {
        // Look up the latest challenge we issued to this client
        let challenge = pow::latest_challenge_for_client(&state, cid);
        if let Some(ch) = challenge {
            // Check if the challenge was solved (cached in Redis)
            if !pow::solution_solved(&mut state.redis.clone(), &ch).await {
                // Issue a new challenge — the export request is rejected with 429
                let new_ch = pow::generate_challenge(
                    state.config.pow_difficulty,
                    state.config.pow_ttl_secs,
                );
                return Err(AppError::RateLimitedJson(json!({
                    "err": -429,
                    "msg": "proof of work required",
                    "challenge": new_ch.challenge,
                    "difficulty": new_ch.difficulty,
                })));
            }
        }
    }
}
```

### Try It Yourself

```rust
// Known-good test vector: SHA-256("fichub-db-test-challenge17269") starts with "0000"
#[test]
fn verify_known_good_vector_difficulty_16() {
    assert!(verify_solution("fichub-db-test-challenge", "17269", 16));
}
```

Run: `cargo test --lib verify_solution -- --nocapture`

### Check

- ✅ `generate_challenge(16, 600)` returns a 19-hex-char challenge string.
- ✅ `verify_solution("test-challenge", "17269", 16)` returns `true` (known vector).
- ✅ `difficulty_to_hex_prefix(16)` returns `"0000"`.
- ✅ `difficulty_to_hex_prefix(8)` returns `"00"`.

### What you built

A complete PoW challenge system: the frontend gets a challenge via `/api/pow/challenge`, solves it in browser JavaScript (finding a nonce where `SHA-256(challenge || nonce)` starts with N zero bits), and submits it to `/api/pow/solve`. The export endpoint checks the Redis cache to verify the solve before serving the file.

---

## Conclusion

You now understand FicHub's complete rate-limiting stack:

- **Token buckets** in Redis Lua scripts — atomic, fast, no race conditions.
- **Four tiers** — download (strict), auth (strict), search (loose), default (moderate).
- **Datacenter blocking** via MaxMind GeoLite2-ASN — fails open when unconfigured.
- **Shadowban set** — Redis SET with per-member TTL, stricter buckets for flagged clients.
- **NAT awareness** — per-IP ceiling + per-client bonus, so shared IPs aren't punished.
- **PoW challenge** — hashcash puzzles for shadowbanned clients, cached in Redis for 10 minutes.
- **Static delay in dev** — no Redis needed when `DYNAMIC_RATE_LIMIT=false`.

All tier parameters are configurable via environment variables (`RL_DOWNLOAD_CAPACITY`, `RL_AUTH_CAPACITY`, `RL_SEARCH_CAPACITY`, `MAXMINED_DB`, `POW_DIFFICULTY`, etc.).

---

## On to the next part

In Part 21 we'll build [Proof of Work] — a deep dive into the hashcash math, browser-side solving with Web Workers, and the test vectors that keep the system safe.
