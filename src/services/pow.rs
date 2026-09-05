//! Hashcash-style proof-of-work (PoW) challenges for flagged clients.
//!
//! When a client is shadowbanned/flagged (see `src/limiter/`), the export
//! endpoint refuses to serve the request until the client proves it spent
//! real CPU work. The challenge is a random hex string; the client must find
//! a `nonce` such that `SHA-256(challenge || nonce)` (hex) starts with
//! `difficulty` zero bits (difficulty 16 → the first 4 hex chars are `0000`).
//!
//! This is friction, not a hard block: humans (or a tiny bit of browser JS)
//! never notice a ~65k-hash solve, while bulk bots pay ~1000x per request.
//! Solves are cached in Redis (`fichub:pow:solved:<challenge>`, TTL 10 min by
//! default) so a client solves each challenge once and reuses it across the
//! follow-up export requests.
//!
//! The pure math (`generate_challenge`, `verify_solution`,
//! `difficulty_to_hex_prefix`) lives here so it is unit-testable without
//! Redis; the Redis-backed helpers (`store_solution`, `solution_solved`) are
//! the small async wrappers used by the routes.

use rand::RngCore;
use sha2::Digest;

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

/// A freshly-issued challenge. `expires_at` is a unix timestamp (seconds)
/// so clients can decide whether to bother solving it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowChallenge {
    /// Random 16 bytes hex-encoded (32 hex chars).
    pub challenge: String,
    /// Number of leading zero BITS the solution hash must start with.
    /// Difficulty 16 → the first 4 hex chars must be `0000`.
    pub difficulty: u32,
    /// Unix timestamp (seconds) after which the challenge is stale.
    pub expires_at: i64,
}

/// Random source for `generate_challenge` — a `Box<dyn RngCore + Send>`
/// keeps the generation function testable while staying Send/Sync-friendly
/// (the real server uses `rand::rngs::OsRng`).
pub type Rng = Box<dyn RngCore + Send>;

/// Generate a fresh challenge: 16 random bytes hex-encoded (32 chars).
pub fn generate_challenge_with(difficulty: u32, ttl_secs: u64, mut rng: Rng) -> PowChallenge {
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    let challenge = hex::encode(bytes);
    let expires_at = chrono::Utc::now().timestamp() + ttl_secs as i64;
    PowChallenge {
        challenge,
        difficulty,
        expires_at,
    }
}

/// Generate a fresh challenge using the OS RNG (the default).
pub fn generate_challenge(difficulty: u32, ttl_secs: u64) -> PowChallenge {
    generate_challenge_with(difficulty, ttl_secs, Box::new(rand::rngs::OsRng))
}

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

/// The most recently issued challenge for `client_id` (if any). Used by the
/// export gate to decide whether the client already solved it.
pub fn latest_challenge_for_client(
    state: &crate::server::AppState,
    client_id: &str,
) -> Option<String> {
    let key = format!("{CHALLENGE_KEY_PREFIX}{client_id}");
    let mut conn = state.redis.clone();
    let mut cmd = redis::cmd("GET");
    cmd.arg(key);
    let fut = cmd.query_async::<Option<String>>(&mut conn);
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut)).unwrap_or(None)
}

/// Remember the challenge we issued to `client_id` so the export gate can
/// check its solve status on subsequent requests. Stored under the TTL of a
/// solve; a stale entry simply causes a fresh challenge to be issued.
pub fn set_latest_challenge_for_client(
    state: &crate::server::AppState,
    client_id: &str,
    challenge: &str,
) {
    let key = format!("{CHALLENGE_KEY_PREFIX}{client_id}");
    let mut conn = state.redis.clone();
    let mut cmd = redis::cmd("SETEX");
    cmd.arg(key).arg(state.config.pow_ttl_secs).arg(challenge);
    let fut = cmd.query_async::<()>(&mut conn);
    let _: Result<(), redis::RedisError> =
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut));
}

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

    /// SHA-256("abc") = ba7816bf... — a known NIST vector that must NOT
    /// satisfy any difficulty >= 1 (first hex char is `b`).
    #[test]
    fn verify_nist_abc_vector_not_a_solution() {
        // Verify the reference hash the test relies on, then the rejection.
        let digest = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(b"abc");
            hex::encode(hasher.finalize())
        };
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(!verify_solution("a", "bc", 1));
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
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let rng1: Rng = Box::new(StdRng::seed_from_u64(42));
        let rng2: Rng = Box::new(StdRng::seed_from_u64(42));
        let a = generate_challenge_with(16, 600, rng1);
        let b = generate_challenge_with(16, 600, rng2);
        assert_eq!(a.challenge, b.challenge, "same seed → same challenge");

        let c = generate_challenge(16, 600);
        assert_ne!(a.challenge, c.challenge, "thread rng differs from seed 42");
    }
}
