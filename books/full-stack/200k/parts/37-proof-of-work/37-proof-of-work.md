# Part 37 — Proof of Work

Bots are fast. A bot can fire off hundreds of export requests per second, hammering the scrapers and filling the cache. Humans are slower — one click, one download. Proof-of-work (PoW) is a clever trick that makes *every* request pay a tiny amount of CPU work to prove it's backed by a real browser (or a patient human), while making it economically painful for a fast bulk bot.

This part builds the PoW challenge system that sits in front of the export endpoint for **shadowbanned** clients — clients the rate limiter has flagged as suspicious. Normal users never see a challenge at all.

---

## 37.1 Backend: `GET /api/pow/challenge` — issue a challenge

Open `src/routes/pow.rs`:

```rust
// src/routes/pow.rs (lines 1-77, excerpt)
use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;
use crate::services::pow;

/// `POST /api/pow/solve` body.
#[derive(Debug, Deserialize)]
pub struct SolveRequest {
    /// The challenge string from `GET /api/pow/challenge`.
    pub challenge: String,
    /// The client's solution nonce (decimal digits).
    pub nonce: String,
}

/// `GET /api/pow/challenge` — issue a challenge to a shadowbanned client.
///
/// Requires the `X-Client-Id` header: the shadowban set is keyed by
/// client_id (see `src/limiter/`). When the client is NOT shadowbanned the
/// response is `{"err":0, "not_needed":true}` so normal traffic sees no
/// friction at all.
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

    // Fail-open: a Redis hiccup means is_shadowbanned() is false, so normal
    // traffic is never accidentally challenged.
    if !state.rate_limiter.is_shadowbanned(&cid) {
        return Ok(Json(json!({
            "err": 0,
            "not_needed": true,
        })));
    }

    let challenge = pow::generate_challenge(
        state.config.pow_difficulty,
        state.config.pow_ttl_secs,
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

### Breakdown

**The `X-Client-Id` header**: This is the key. FicHub clients (the browser extension, mobile apps, etc.) send a stable `X-Client-Id` header with every request. The shadowban set is keyed by this client_id — it identifies *which client* is behaving badly, not *who* the user is. A client_id is just a random UUID the client generates and stores locally.

**Fail-open design**: If the client_id header is missing, or if the client is *not* in the shadowban set, the response is `{"err": 0, "not_needed": true}`. Normal users breeze through without ever seeing a challenge. This is critical — you never want a rate-limiter bug to lock out real users.

**The challenge itself**: When the client *is* shadowbanned, the handler calls `pow::generate_challenge()` which returns a random 19-character hex string (16 random bytes) plus the current difficulty setting. The client must find a nonce that, when hashed with SHA-256 alongside the challenge, produces a hash starting with enough zero bits.

### The route registration

The routes are registered in `src/server.rs` alongside the export route:

```rust
// src/server.rs (lines 262-264, excerpt)
// Proof-of-work challenge for shadowbanned clients
.route("/api/pow/challenge", get(routes::pow::challenge_handler))
.route("/api/pow/solve", axum::routing::post(routes::pow::solve_handler))
```

Notice there are two routes: one GET (issue a challenge) and one POST (submit a solution). They're separate so issuing a challenge is idempotent — you can call it repeatedly without side effects.

---

## 37.2 Backend: `POST /api/pow/solve` — verify the solution

The solve handler is in the same file:

```rust
// src/routes/pow.rs (lines 84-139, excerpt)
/// `POST /api/pow/solve` — verify a solution and cache it in Redis.
///
/// Returns 429 until a valid solution is stored, then `{"err":0}`. A valid
/// solve is cached with a 10-minute TTL so subsequent export requests pass
/// the epub_handler's PoW gate without re-solving.
pub async fn solve_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<SolveRequest>,
) -> Result<Json<Value>, AppError> {
    // Only shadowbanned clients are allowed to solve (the challenge is
    // meaningless for everyone else).
    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(ref cid) = client_id {
        if !state.rate_limiter.is_shadowbanned(cid) {
            return Ok(Json(json!({
                "err": 0,
                "not_needed": true,
            })));
        }
    }

    // Basic sanity: the challenge must be a plausible 19-hex-char value.
    if body.challenge.len() != 32 || !body.challenge.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::RateLimited(0));
    }

    // Remember the challenge we are about to accept so the export gate can
    // tie the solve to the challenge it actually checks.
    if let Some(ref cid) = client_id {
        pow::set_latest_challenge_for_client(&state, cid, &body.challenge);
    }

    let difficulty = state.config.pow_difficulty;
    if pow::verify_solution(&body.challenge, &body.nonce, difficulty) {
        let _ = pow::store_solution(
            &mut state.redis.clone(),
            &body.challenge,
            state.config.pow_ttl_secs,
        )
        .await;
        return Ok(Json(json!({
            "err": 0,
            "solved": true,
        })));
    }

    // Invalid solution → 429 with a fresh challenge (friction, not a block).
    let challenge = pow::generate_challenge(difficulty, state.config.pow_ttl_secs);
    Err(AppError::RateLimitedJson(json!({
        "err": -429,
        "msg": "invalid proof of work",
        "challenge": challenge.challenge,
        "difficulty": challenge.difficulty,
        "expires_at": challenge.expires_at,
    })))
}
```

### Breakdown

**Only shadowbanned clients solve**: Just like the challenge handler, the solve handler first checks `is_shadowbanned`. If the client isn't flagged, it returns `{"err": 0, "not_needed": true}` — the solve endpoint is a no-op for normal users.

**Challenge validation**: Before doing any crypto work, the handler sanity-checks that the challenge is 32 hex characters long. This is a cheap guard against malformed requests that would just waste CPU.

**Storing the challenge per client**: When a valid challenge is submitted (even before verification), the handler calls `set_latest_challenge_for_client` to remember which challenge this client is working on. This ties the solve to the export gate — more on that in the next chapter.

**Verification and caching**: The real work happens in `pow::verify_solution()`. If the nonce is valid, the challenge is stored in Redis with a 10-minute TTL under the key `fichub:pow:solved:<challenge>`. This is what lets the client reuse the same solution across multiple export requests without solving again.

**Invalid solution → fresh challenge**: If the nonce doesn't satisfy the difficulty, the handler returns HTTP 429 (Too Many Requests) with a *brand new* challenge. This is friction, not a block — the client just tries again with the new challenge. The difficulty is the same, so a legitimate browser doing the work in JavaScript will solve it in under a second.

### The verification math (from `src/services/pow.rs`)

```rust
// src/services/pow.rs (lines 80-105, excerpt)
/// The minimum hex prefix a valid solution hash must start with.
///
/// `difficulty` counts leading zero BITS; each hex char is 4 bits, so the
/// required prefix is `ceil(difficulty / 4)` zero chars. Difficulty 16 →
/// `"0000"`. A non-multiple-of-4 difficulty (e.g. 17) still requires
/// `ceil(17/4) = 5` zero hex chars, which is slightly *harder* than 17 bits
/// — an acceptable granularity for a friction layer.
pub fn difficulty_to_hex_prefix(difficulty: u32) -> String {
    let nibbles = difficulty.div_ceil(4);
    "0".repeat(nibbles as usize)
}

/// Verify a `(challenge, nonce)` pair: the SHA-256 hex of
/// `challenge || nonce` must start with `difficulty` zero bits.
///
/// The nonce is the decimal ASCII encoding of an unsigned counter, matching
/// the classic hashcash construction. Returns `false` (never panics) for a
/// non-numeric nonce.
pub fn verify_solution(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        // Zero-bit difficulty is trivially satisfiable by any nonce.
        return nonce.chars().all(|c| c.is_ascii_digit());
    }
    if !nonce.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();
    let hex_digest = hex::encode(digest);
    hex_digest.starts_with(&difficulty_to_hex_prefix(difficulty))
}
```

### How the difficulty works

The difficulty is measured in **leading zero bits**. Here's the key insight: SHA-256 produces a 256-bit hash that looks completely random. The probability that a random hash starts with N zero bits is 1/2^N. So:

- Difficulty 4 (1 hex char "0"): ~1 in 16 hashes
- Difficulty 8 (2 hex chars "00"): ~1 in 256 hashes
- Difficulty 16 (4 hex chars "0000"): ~1 in 65,536 hashes
- Difficulty 20 (5 hex chars "00000"): ~1 in 1,048,576 hashes

The default difficulty is 16, which means the client needs to try about 65,536 nonces on average. On a modern laptop, that's roughly 0.1–1 second of CPU time. The config lives in `src/config.rs`:

```rust
// src/config.rs (lines 123-129, excerpt)
/// ── Proof-of-work (PoW) challenge for shadowbanned clients ──────
/// Leading zero BITS the SHA-256(challenge || nonce) hex must start with.
/// 16 bits ≈ 65k hashes ≈ 0.1–1s on a laptop; 4 bits for cheap tests.
pub pow_difficulty: u32,
/// TTL in seconds of a stored solve (`fichub:pow:solved:<challenge>`)
/// and of a challenge. 600 = 10 minutes.
pub pow_ttl_secs: u64,
```

And it's loaded from environment variables:

```rust
// src/config.rs (lines 456-464, excerpt)
// Proof-of-work challenge for shadowbanned clients
let pow_difficulty = std::env::var("POW_DIFFICULTY")
    .unwrap_or_else(|_| "16".to_string())
    .parse::<u32>()
    .unwrap_or(16);
let pow_ttl_secs = std::env::var("POW_TTL_SECS")
    .unwrap_or_else(|_| "600".to_string())
    .parse::<u64>()
    .unwrap_or(600);
```

### Redis keys for solutions

The solution is cached in Redis so the client doesn't have to re-solve:

```rust
// src/services/pow.rs (lines 21-31, excerpt)
/// Redis key prefix for a stored solve: `fichub:pow:solved:<challenge>`.
/// `GET` returns `"1"` when the challenge was solved within the TTL.
pub const SOLVED_KEY_PREFIX: &str = "fichub:pow:solved:";

/// Redis key prefix for the latest issued challenge per client_id:
/// `fichub:pow:challenge:<client_id>`. The epub_handler gate re-checks
/// THIS challenge's solve status on every request, so the client solves
/// one challenge and its follow-up export requests pass without re-solving.
pub const CHALLENGE_KEY_PREFIX: &str = "fichub:pow:challenge:";

/// Default challenge lifetime (seconds). Challenges are cheap to issue and
/// the stored solve must outlive the client's follow-up requests.
pub const DEFAULT_POW_TTL_SECS: u64 = 600; // 10 minutes
```

And here's how solutions are stored and checked:

```rust
// src/services/pow.rs (lines 107-138, excerpt)
/// Store a solved challenge in Redis with a TTL (fail-open: errors are
/// logged, the solve is treated as accepted by the caller).
pub async fn store_solution(
    redis: &mut redis::aio::MultiplexedConnection,
    challenge: &str,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let key = format!("{SOLVED_KEY_PREFIX}{challenge}");
    redis::cmd("SETEX")
        .arg(key)
        .arg(ttl_secs)
        .arg("1")
        .query_async::<()>(redis)
        .await?;
    Ok(())
}

/// True when `challenge` has a stored, unexpired solve in Redis
/// (fail-open: any Redis error returns `false`, so a Redis hiccup never
/// un-blocks a flagged client — it just re-issues the challenge).
pub async fn solution_solved(
    redis: &mut redis::aio::MultiplexedConnection,
    challenge: &str,
) -> bool {
    let key = format!("{SOLVED_KEY_PREFIX}{challenge}");
    let exists: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(redis)
        .await
        .unwrap_or(None);
    exists.is_some()
}
```

The `store_solution` uses Redis's `SETEX` command — set with expiry. The challenge is stored as a key for exactly `pow_ttl_secs` (600 seconds by default), then it automatically evaporates. No cleanup job needed.

---

## 37.3 Backend: shadowban flow — challenge → solve → allow

Now let's see how all of this connects. When a shadowbanned client tries to download a fic, the flow is:

1. Client calls `GET /api/epub?q=<url>` with `X-Client-Id` header
2. The export handler checks `is_shadowbanned` — yes, this client is flagged
3. The handler looks up the "latest challenge" for this client in Redis
4. If no solve is cached for that challenge, the handler returns HTTP 429 with a fresh challenge
5. The client solves the challenge and POSTs it to `POST /api/pow/solve`
6. The solve handler verifies the nonce, stores the solution in Redis
7. The client retries `GET /api/epub?q=<url>` — now the solve is cached, the PoW gate passes, and the export proceeds

Here's the gate code in the export handler:

```rust
// src/routes/export.rs (lines 55-86, excerpt)
    // ── Proof-of-work gate (shadowbanned clients only) ───────────────
    // A flagged client must solve a hashcash-style challenge before the
    // export is served. The challenge is re-issued (429 + fresh challenge)
    // until the client POSTs a valid solve, which is cached in Redis for
    // POW_TTL_SECS (10 min by default) so the follow-up export requests
    // pass without re-solving. Non-shadowbanned clients and requests
    // without a client_id skip this entirely. Redis errors fail open: a
    // hiccup re-issues the challenge instead of un-blocking a bot.
    if let Some(ref cid) = client_id {
        if state.rate_limiter.is_shadowbanned(cid) {
            let last_challenge = pow::latest_challenge_for_client(&state, cid);
            let solved = pow::solution_solved(
                &mut state.redis.clone(),
                last_challenge.as_deref().unwrap_or(""),
            )
            .await;
            if !solved {
                let challenge = pow::generate_challenge(
                    state.config.pow_difficulty,
                    state.config.pow_ttl_secs,
                );
                pow::set_latest_challenge_for_client(&state, cid, &challenge.challenge);
                return Err(AppError::RateLimitedJson(json!({
                    "err": -429,
                    "msg": "proof of work required",
                    "challenge": challenge.challenge,
                    "difficulty": challenge.difficulty,
                    "expires_at": challenge.expires_at,
                })));
            }
        }
    }
```

### The shadowban set

The shadowban state lives in Redis as a SET called `fichub:shadowban`:

```rust
// src/limiter/redis_bucket.rs (lines 67, 393-409, excerpt)
pub const SHADOWBAN_SET: &str = "fichub:shadowban";

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
```

The `is_shadowbanned` check uses Redis's `SISMEMBER` — a single, fast O(1) lookup. It's a blocking call (runs on the current thread via `block_in_place`), but it's a single Redis roundtrip so it's nearly instant. On any Redis error, it returns `false` (fail-open: never block a real user because of a Redis hiccup).

The shadowban TTL defaults to 24 hours (configurable via `RL_SHADOWBAN_TTL`):

```rust
// src/config.rs (lines 117-119)
    /// TTL in seconds of a shadowban set entry (admin-unshadowbar after).
    pub rl_shadowban_ttl: u64,
```

```rust
// src/config.rs (lines 447-450, excerpt)
    let rl_shadowban_ttl = std::env::var("RL_SHADOWBAN_TTL")
        .unwrap_or_else(|_| "86400".to_string()) // 24h
        .parse::<u64>()
        .unwrap_or(86400);
```

### Shadowban vs. hard block

This is a crucial design choice. Shadowbanning isn't a hard block — it's *friction*:

- **Shadowbanned client**: Gets a stricter download rate limit (5 downloads/hour instead of 60) AND must solve a PoW challenge for each export.
- **Hard block**: The client gets HTTP 429 with a `retry_after` and nothing else.

The shadowban is a "speed bump," not a wall. A real user with a shadowbanned client_id can still download — they just have to wait a few seconds for the PoW to solve. But a bot doing bulk downloads pays ~1000x more per request (65k hash iterations × hundreds of parallel requests), which makes scraping uneconomical.

### Admin shadowban endpoints

Admins can shadowban and unshadowban clients via the admin API:

```rust
// src/routes/admin.rs (lines 939-974, excerpt)
/// POST /api/admin/bots/{client_id}/shadowban — add a client to the Redis
/// shadowban set for the configured TTL (friction: stricter download bucket,
/// PoW challenge).
pub async fn admin_bot_shadowban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    state
        .rate_limiter
        .shadowban(&client_id, state.config.rl_shadowban_ttl);
    Ok(Json(json!({ "err": 0, "client_id": client_id, "shadowbanned": true })))
}

/// POST /api/admin/bots/{client_id}/unshadowban — remove a client from the
/// Redis shadowban set (admin override).
pub async fn admin_bot_unshadowban(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(client_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    if user.role < 10 {
        return Err(AppError::Forbidden("Admin access required".into()));
    }
    state.rate_limiter.unshadowban(&client_id);
    Ok(Json(json!({ "err": 0, "client_id": client_id, "shadowbanned": false })))
}
```

These are registered in the router alongside the other admin routes:

```rust
// src/server.rs (lines 836-837, excerpt)
.route("/api/admin/bots/{client_id}/shadowban", axum::routing::post(crate::routes::admin::admin_bot_shadowban))
.route("/api/admin/bots/{client_id}/unshadowban", axum::routing::post(crate::routes::admin::admin_bot_unshadowban))
```

---

## How the pieces fit together

Let's trace the full flow one more time, from the client's perspective:

```
┌─────────────┐  1. GET /api/epub?q=<url>
│   Browser   │  X-Client-Id: abc-123
│  (shadow-   │  ← shadowbanned? yes
│   banned)   │
└──────┬──────┘
       │
       │  2. HTTP 429: {"err": -429, "msg": "proof of work required",
       │                 "challenge": "a1b2c3…", "difficulty": 16,
       │                 "expires_at": 1234567890}
       ▼
┌─────────────────┐
│  JS PoW solver  │  3. Try nonce = 0, 1, 2, …
│  in browser     │     SHA-256(challenge + nonce)
│                 │     find one with 4 leading zero bytes
└──────┬──────┘
       │
       │  4. POST /api/pow/solve
       │     {"challenge": "a1b2c3…", "nonce": "41235"}
       ▼
┌─────────────┐
│   Server    │  5. SHA-256("a1b2c3…" + "41235") = "0000abc…" ✓
│             │     → store "1" at fichub:pow:solved:a1b2c3… (10min TTL)
│             │  6. HTTP 200: {"err": 0, "solved": true}
└──────┬──────┘
       │
       │  7. GET /api/epub?q=<url>  (retry)
       │  X-Client-Id: abc-123
       ▼
┌─────────────────┐
│  PoW gate      │  8. is_shadowbanned? yes
│  (export.rs)   │     latest_challenge = "a1b2c3…"
│                │     solution_solved = true (cached in Redis) ✓
│                │     → pass the gate, proceed to export
└──────┬──────┘
       │
       │  9. Export served (epub download)
       ▼
```

The clever part is step 8: the server remembered which challenge this client was working on (via `set_latest_challenge_for_client`), and the solution is cached in Redis under that challenge key. The client only solves once and reuses the cached solution.

---

## Unit tests

The PoW math is pure functions, so it's extensively unit-tested in `src/services/pow.rs`:

```rust
// src/services/pow.rs (lines 174-293, excerpt)
#[cfg(test)]
mod tests {
    use super::*;

    /// A known-good (challenge, nonce, difficulty) vector computed with the
    /// reference Python implementation. SHA-256("unit-test-challenge241")
    /// starts with `00` (8 zero bits).
    #[test]
    fn verify_known_good_vector_difficulty_8() {
        assert!(verify_solution("unit-test-challenge", "241", 8));
    }

    /// A known-good difficulty-16 vector: SHA-256("fichub-db-test-challenge17269")
    /// starts with `0000` (16 zero bits).
    #[test]
    fn verify_known_good_vector_difficulty_16() {
        assert!(verify_solution("fichub-db-test-challenge", "17269", 16));
    }

    /// The same nonce must NOT satisfy a higher difficulty. The diff-16
    /// nonce (17269) starts with `0000` so it trivially also starts with
    /// `00` — the diff-8 half uses a nonce that is valid at 8 bits but NOT
    /// at 16: SHA-256("fichub-db-test-challenge38") = 00a5da6a... (starts
    /// `00`, not `0000`).
    #[test]
    fn verify_rejects_wrong_difficulty() {
        assert!(!verify_solution("fichub-db-test-challenge", "17269", 32));
        assert!(verify_solution("fichub-db-test-challenge", "38", 8));
        assert!(!verify_solution("fichub-db-test-challenge", "38", 16));
    }

    /// A wrong nonce for the same challenge must fail.
    #[test]
    fn verify_rejects_wrong_nonce() {
        assert!(!verify_solution("unit-test-challenge", "242", 8));
    }

    /// A wrong challenge with the same nonce must fail.
    #[test]
    fn verify_rejects_wrong_challenge() {
        assert!(!verify_solution("unit-test-challenge-x", "241", 8));
    }

    /// Non-numeric nonces are never valid (no panic).
    #[test]
    fn verify_rejects_non_numeric_nonce() {
        assert!(!verify_solution("unit-test-challenge", "abc", 8));
        assert!(!verify_solution("unit-test-challenge", "12a", 8));
        assert!(!verify_solution("unit-test-challenge", "-5", 8));
    }

    /// Empty nonce is numeric but never a valid solve at any real difficulty.
    #[test]
    fn verify_rejects_empty_nonce() {
        assert!(!verify_solution("unit-test-challenge", "", 8));
    }

    /// Zero difficulty accepts any numeric nonce (trivially satisfiable).
    #[test]
    fn verify_zero_difficulty_accepts_any_numeric_nonce() {
        assert!(verify_solution("anything", "0", 0));
        assert!(verify_solution("anything", "12345", 0));
        assert!(!verify_solution("anything", "not-a-number", 0));
    }

    /// Prefix length: 16 bits → 4 hex chars, 17 bits → 5 (ceil), 4 → 1.
    #[test]
    fn difficulty_prefix_length() {
        assert_eq!(difficulty_to_hex_prefix(16), "0000");
        assert_eq!(difficulty_to_hex_prefix(0), "");
        assert_eq!(difficulty_to_hex_prefix(4), "0");
        assert_eq!(difficulty_to_hex_prefix(17), "00000");
        assert_eq!(difficulty_to_hex_prefix(32), "00000000");
    }

    /// A generated challenge is 32 hex chars (16 random bytes), has the
    /// requested difficulty, and expires in the future.
    #[test]
    fn generate_challenge_shape() {
        let ch = generate_challenge(16, 600);
        assert_eq!(ch.challenge.len(), 32);
        assert!(ch.challenge.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(ch.difficulty, 16);
        assert!(ch.expires_at > chrono::Utc::now().timestamp());
    }

    /// Two challenges from a fixed-seed RNG are deterministic and distinct
    /// from thread-rng ones; a fixed RNG makes the generation testable.
    #[test]
    fn generate_challenge_with_fixed_rng() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let rng1: Rng = Box::new(StdRng::seed_from_u64(42));
        let rng2: Rng = Box::new(StdRng::seed_from_u64(42));
        let a = generate_challenge_with(16, 600, rng1);
        let b = generate_challenge_with(16, 600, rng2);
        assert_eq!(a.challenge, b.challenge, "same seed → same challenge");

        let c = generate_challenge(16, 600);
        assert_ne!(a.challenge, c.challenge, "thread rng differs from seed 42");
    }
}
```

### Check

- `cargo test --lib pow` runs all the PoW unit tests — the known-good vectors, rejection cases, and difficulty/prefix-length checks.
- `curl -H "X-Client-Id: test-client" http://localhost:8000/api/pow/challenge` returns `{"err":0,"not_needed":true}` for a non-shadowbanned client.
- After shadowbanning a client_id, the same request returns a real challenge with `not_needed: false`.

### Troubleshooting

| Problem | Cause | Fix |
|---|---|---|
| `not_needed: true` even though client should be shadowbanned | Client isn't in the `fichub:shadowban` Redis set | Check `SISMEMBER fichub:shadowban <client_id>` in Redis CLI |
| `is_shadowbanned` always returns `false` | Redis connection failed — fail-open | Check Redis is reachable; the server logs connection errors at startup |
| Solve returns 429 with "invalid proof of work" | Nonce doesn't produce a hash with enough leading zeros | Re-read the challenge from the 429 response body; the nonce counter must start from 0 for the *new* challenge |
| Export still returns 429 after solving | Challenge mismatch — `latest_challenge_for_client` is stale | The solve handler stores the *submitted* challenge, not the latest one. Make sure you're solving the challenge from the *current* 429 response |
| PoW makes every export slow | Difficulty is too high for your hardware | Set `POW_DIFFICULTY=8` for faster solves (~256 hashes) |

---

## Try It Yourself: solve a PoW challenge by hand

### Step 1: Run the unit tests

```bash
cargo test --lib pow
```

This verifies the known-good (challenge, nonce) vectors and the difficulty math. If any test fails, something is wrong with the hash function wiring.

### Step 2: Solve a challenge in Python

The PoW is simple enough to reproduce in a few lines of Python:

```python
import hashlib
import secrets

# Step A: Generate your own challenge (simulating the server)
challenge = secrets.token_hex(16)
difficulty = 16  # need 4 leading zero hex chars → "0000"

# Step B: Find a nonce that satisfies the difficulty
nonce = 0
while True:
    digest = hashlib.sha256((challenge + str(nonce)).encode()).hexdigest()
    if digest.startswith("0" * (difficulty // 4)):
        print(f"Solved! nonce={nonce}, hash={digest}")
        break
    nonce += 1

# Step C: Verify your solution matches the Rust verify_solution logic
assert hashlib.sha256((challenge + str(nonce)).encode()).hexdigest()[:4] == "0000"
print("✓ Verification passed — your solution would be accepted by the server")
```

### Step 3: Test against a local server

Start the FicHub server with a low difficulty for testing:

```bash
POW_DIFFICULTY=4 POW_TTL_SECS=60 cargo run
```

Then simulate the flow:

```bash
# 1. Shadowban a test client (you'll need admin access)
#    This adds the client_id to the Redis fichub:shadowban set

# 2. Request a challenge (use a fake shadowbanned client_id)
curl -s -H "X-Client-Id: test-bot-001" \
  http://localhost:3000/api/pow/challenge | python3 -m json.tool

# 3. Solve it in Python (modify the script above to use the challenge
#    from step 2), then submit your solution
curl -s -X POST http://localhost:3000/api/pow/solve \
  -H "X-Client-Id: test-bot-001" \
  -H "Content-Type: application/json" \
  -d '{
    "challenge": "<challenge-from-step-2>",
    "nonce": "<your-solution-nonce>"
  }' | python3 -m json.tool

# 4. Challenge solved! The solution is now cached in Redis for 60 seconds.
#    Retry your original export request — it should pass the PoW gate.
```

### Difficulty explorer

Try computing how long each difficulty takes on your machine:

| Difficulty | Zero bits | Expected hashes | Hex prefix |
|---|---|---|---|
| 4 | 4 | ~16 | `"0"` |
| 8 | 8 | ~256 | `"00"` |
| 16 | 16 | ~65,536 | `"0000"` |
| 20 | 20 | ~1,048,576 | `"00000"` |
| 24 | 24 | ~16,777,216 | `"000000"` |

The production default is 16 (~65k hashes, ~0.1–1 second on a laptop). A bot doing 100 requests/second would need to perform ~6.5 million hashes/second to keep up — entirely feasible for a dedicated rig, but expensive enough to deter casual scrapers. Combined with the shadowban's stricter rate limit (5 downloads/hour), the bot's effective throughput drops to near-zero.

---

## What you have now

- You understand **proof-of-work** as a bot-deterrent: clients must find a nonce where `SHA-256(challenge || nonce)` starts with N zero bits, proving they spent real CPU time.
- You understand the **three-layer defense**: tiered rate limiting (token buckets in Redis) → shadowban detection (Redis SET) → PoW challenge (hashcash-style) — each layer only triggers if the previous one flags the client as suspicious.
- You understand the **challenge lifecycle**: `GET /api/pow/challenge` issues a 19-hex-char challenge → the client solves it in JS → `POST /api/pow/solve` verifies and caches the solution in Redis for 10 minutes → the export gate checks the cached solve on subsequent requests.
- You understand the **fail-open design principle**: Redis errors never block real users — they just re-issue the challenge. Normal users (not shadowbanned) see `{"err": 0, "not_needed": true}` and sail right through.
- You understand the **admin workflow**: shadowban and unshadowban via `POST /api/admin/bots/{client_id}/shadowban|unshadowban`.
- You solved a PoW challenge in Python and verified the math matches the Rust implementation.

On to [Part 38 — Frontend Foundations](./38-frontend-foundations/19-frontend-foundations.md).
