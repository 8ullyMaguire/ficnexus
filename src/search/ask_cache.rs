//! Search query translation cache — Redis primary, Postgres write-only archive.
//!
//! \"Ask the Archive\" (`POST /api/search/ask`) sends every natural-language
//! query to Ollama to translate it into search filters. Translating is
//! expensive (a llama generate call), so the resulting parameter set is
//! cached per exact NL query string. Repeat asks for the same phrase hit
//! cache instead of the model.
//!
//! Tier 1 — Redis (fast, 24-hour TTL): the ONLY lookup tier. Survives the
//! common repeat window (hours-days) and is what the handler checks.
//!
//! Tier 2 — Postgres (permanent, write-only): every translation is saved
//! here as a durable archive, but the handler never reads it (reads are
//! disabled by design). The table exists so a future re-enable or an
//! analytics pass can mine what the model produced over time.
//!
//! The cache stores ONLY the translation (NL → params), never search
//! results: results go stale as the archive grows and must always come
//! from a live search.
//!
//! The cache is best-effort by design: a Redis OR Postgres outage must
//! never fail the ask endpoint (it falls back to a plain full-text
//! search), so every operation here degrades to a no-op/miss on error and
//! only warns at debug level.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};

/// How long a translation lives in Redis (seconds). 24 hours.
pub const ASK_CACHE_TTL_SECS: u64 = 60 * 60 * 24;

/// Redis key prefix for cached translations.
pub const ASK_CACHE_KEY_PREFIX: &str = "fichub:ask:translation:";

/// How long a full AskResponse lives in Redis (seconds). 1 hour.
pub const ASK_RESPONSE_TTL_SECS: u64 = 60 * 60;

/// Redis key prefix for cached full Ask responses.
pub const ASK_RESPONSE_KEY_PREFIX: &str = "fichub:ask:response:";

/// In-process counter for AskResponse cache hits (metric).
static ASK_RESPONSE_CACHE_HITS: AtomicU64 = AtomicU64::new(0);

pub fn ask_response_cache_hits() -> u64 {
    ASK_RESPONSE_CACHE_HITS.load(Ordering::Relaxed)
}

fn inc_ask_response_cache_hits() {
    ASK_RESPONSE_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
}

/// For tests: reset the hit counter.
#[cfg(test)]
pub fn reset_ask_response_cache_hits() {
    ASK_RESPONSE_CACHE_HITS.store(0, Ordering::Relaxed);
}

/// Normalize an NL query for cache lookup: trim + lowercase. Raises the
/// hit rate versus the historical exact-string key ("Dark Harry" ==
/// "dark harry"). Cheap, lossless for translation purposes.
pub fn normalize_nl_query(nl: &str) -> String {
    nl.trim().to_lowercase()
}

/// Hash an NL query for the AskResponse cache key (sha256 of normalized query).
pub fn hash_nl_query(nl: &str) -> String {
    let normalized = normalize_nl_query(nl);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

fn ask_response_key(nl: &str) -> String {
    format!("{}{}", ASK_RESPONSE_KEY_PREFIX, hash_nl_query(nl))
}

/// Read a cached translation for `nl_query` from Redis. Returns `None` on
/// a miss OR any Redis error (fail-open).
pub async fn get_cached_translation(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
) -> Option<Value> {
    let key = format!("{}{}", ASK_CACHE_KEY_PREFIX, normalize_nl_query(nl_query));
    let raw: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(redis)
        .await
        .unwrap_or(None);
    raw.and_then(|s| serde_json::from_str(&s).ok())
}

/// Store a translation for `nl_query` in Redis with a 30-day TTL.
/// Fail-open: any Redis error is swallowed (the caller still proceeds).
pub async fn set_cached_translation(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
    params: &Value,
) {
    let key = format!("{}{}", ASK_CACHE_KEY_PREFIX, normalize_nl_query(nl_query));
    let body = serde_json::to_string(params).unwrap_or_else(|_| "{}".into());
    if let Err(e) = redis::cmd("SETEX")
        .arg(key)
        .arg(ASK_CACHE_TTL_SECS)
        .arg(body)
        .query_async::<()>(redis)
        .await
    {
        tracing::debug!("ask translation redis cache set failed (non-fatal): {e}");
    }
}

/// Remove a cached translation from Redis (used by tests to avoid
/// cross-test cache hits). Fail-open.
pub async fn delete_cached_translation(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
) {
    let key = format!("{}{}", ASK_CACHE_KEY_PREFIX, normalize_nl_query(nl_query));
    let _ = redis::cmd("DEL").arg(key).query_async::<()>(redis).await;
}

/// Read a cached full AskResponse for `nl_query` from Redis. Increments the
/// hit counter on success. Fail-open.
pub async fn get_cached_ask_response(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
) -> Option<Value> {
    let key = ask_response_key(nl_query);
    let raw: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(redis)
        .await
        .unwrap_or(None);
    let val = raw.and_then(|s| serde_json::from_str(&s).ok())?;
    inc_ask_response_cache_hits();
    tracing::debug!("ask response cache hit for {:?}", nl_query);
    Some(val)
}

/// Store a full AskResponse for `nl_query` in Redis with 1h TTL. Fail-open.
pub async fn set_cached_ask_response(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
    response: &Value,
) {
    let key = ask_response_key(nl_query);
    let body = serde_json::to_string(response).unwrap_or_else(|_| "{}".into());
    if let Err(e) = redis::cmd("SETEX")
        .arg(key)
        .arg(ASK_RESPONSE_TTL_SECS)
        .arg(body)
        .query_async::<()>(redis)
        .await
    {
        tracing::debug!("ask response redis cache set failed (non-fatal): {e}");
    }
}

/// Remove a cached AskResponse from Redis (used by tests). Fail-open.
pub async fn delete_cached_ask_response(
    redis: &mut redis::aio::MultiplexedConnection,
    nl_query: &str,
) {
    let key = ask_response_key(nl_query);
    let _ = redis::cmd("DEL").arg(key).query_async::<()>(redis).await;
}

/// Read a cached translation for `nl_query` from Postgres. Only rows whose
/// `model` matches the current model are considered a hit (a translation
/// produced by one model is not reused for another). Returns `None` on a
/// miss OR any DB error (fail-open).
///
/// NOTE: reads are DISABLED by design — the ask handler never calls this
/// (Redis is the only lookup tier). The function remains for tests and for
/// a future re-enable; the PG table is currently a write-only archive.
pub async fn get_cached_translation_pg(
    db: &sqlx::PgPool,
    nl_query: &str,
    model: &str,
) -> Option<Value> {
    let normalized = normalize_nl_query(nl_query);
    let row: Option<(sqlx::types::Json<Value>, String)> =
        sqlx::query_as("SELECT params, model FROM ask_translation_cache WHERE nl_query = $1")
            .bind(&normalized)
            .fetch_optional(db)
            .await
            .ok()?;
    match row {
        Some((params, cached_model)) if cached_model == model => Some(params.0),
        _ => None,
    }
}

/// Store a translation for `nl_query` in Postgres (permanent write-only
/// archive). UPSERT so a re-translation by the same model refreshes the row.
/// Fail-open: any DB error is swallowed (the caller still proceeds with the
/// fresh translation). The handler never reads this table back; it exists
/// as a durable record of what the model produced.
pub async fn set_cached_translation_pg(
    db: &sqlx::PgPool,
    nl_query: &str,
    model: &str,
    params: &Value,
) {
    let normalized = normalize_nl_query(nl_query);
    let body = serde_json::to_string(params).unwrap_or_else(|_| "{}".into());
    if let Err(e) = sqlx::query(
        r#"INSERT INTO ask_translation_cache (nl_query, params, model, updated_at)
           VALUES ($1, $2::jsonb, $3, NOW())
           ON CONFLICT (nl_query) DO UPDATE
             SET params = EXCLUDED.params,
                 model = EXCLUDED.model,
                 updated_at = NOW()"#,
    )
    .bind(&normalized)
    .bind(&body)
    .bind(model)
    .execute(db)
    .await
    {
        tracing::debug!("ask translation postgres cache set failed (non-fatal): {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cache_roundtrip() {
        let client = redis::Client::open("redis://localhost:6379".to_string())
            .expect("invalid redis url for test");
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis must be up for this test (Redis 6379)");

        let q = "dark harry potter complete over 50k";
        delete_cached_translation(&mut conn, q).await;
        assert!(get_cached_translation(&mut conn, q).await.is_none());

        let params = serde_json::json!({
            "q": "harry potter",
            "complete": true,
            "min_words": 50000,
        });
        set_cached_translation(&mut conn, q, &params).await;

        let got = get_cached_translation(&mut conn, q).await;
        assert_eq!(got, Some(params), "cached translation must round-trip");
        delete_cached_translation(&mut conn, q).await;
    }

    #[tokio::test]
    async fn cache_miss_on_unknown_query() {
        let client = redis::Client::open("redis://localhost:6379".to_string())
            .expect("invalid redis url for test");
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis must be up for this test (Redis 6379)");
        delete_cached_translation(&mut conn, "definitely-not-cached-xyz").await;
        assert!(
            get_cached_translation(&mut conn, "definitely-not-cached-xyz")
                .await
                .is_none()
        );
    }

    #[test]
    fn normalization_is_trim_lowercase() {
        assert_eq!(
            normalize_nl_query("  Dark Harry Potter  "),
            "dark harry potter"
        );
        assert_eq!(normalize_nl_query("same"), normalize_nl_query("SAME"));
    }

    #[test]
    fn hash_is_deterministic_and_normalized() {
        let h1 = hash_nl_query("  Dark Harry  ");
        let h2 = hash_nl_query("dark harry");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert_ne!(hash_nl_query("foo"), hash_nl_query("bar"));
    }

    #[tokio::test]
    async fn response_cache_roundtrip() {
        let client = redis::Client::open("redis://localhost:6379".to_string())
            .expect("invalid redis url for test");
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .expect("redis must be up for this test (Redis 6379)");
        let q = "response cache test query xyz";
        delete_cached_ask_response(&mut conn, q).await;
        reset_ask_response_cache_hits();
        assert!(get_cached_ask_response(&mut conn, q).await.is_none());
        assert_eq!(ask_response_cache_hits(), 0);
        let resp = serde_json::json!({"total":1,"results":[],"translated":true,"nl_query":q});
        set_cached_ask_response(&mut conn, q, &resp).await;
        let got = get_cached_ask_response(&mut conn, q).await;
        assert_eq!(got, Some(resp));
        assert_eq!(ask_response_cache_hits(), 1, "hit counter must increment");
        // second hit
        let got2 = get_cached_ask_response(&mut conn, q).await;
        assert!(got2.is_some());
        assert_eq!(ask_response_cache_hits(), 2);
        delete_cached_ask_response(&mut conn, q).await;
        reset_ask_response_cache_hits();
    }

    #[tokio::test]
    async fn pg_cache_roundtrip() {
        // DB-gated: skip cleanly when ambient DATABASE_URL is unavailable
        // (config tests clear env vars in parallel, racing this read; the
        // real PG coverage lives in tests/ask_archive_api.rs).
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(u) => u,
            Err(_) => {
                eprintln!("skipping: DATABASE_URL not set");
                return;
            }
        };
        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("failed to connect to test database (is Postgres up?)");

        let q = "pg cache roundtrip query";
        let params = serde_json::json!({
            "q": "stars",
            "min_words": 50000,
            "complete": true,
        });
        // ensure clean start
        sqlx::query("DELETE FROM ask_translation_cache WHERE nl_query = $1")
            .bind(normalize_nl_query(q))
            .execute(&pool)
            .await
            .expect("cleanup delete");

        assert!(
            get_cached_translation_pg(&pool, q, "test-model")
                .await
                .is_none()
        );
        set_cached_translation_pg(&pool, q, "test-model", &params).await;
        let got = get_cached_translation_pg(&pool, q, "test-model").await;
        assert_eq!(got, Some(params.clone()), "PG cache must round-trip");

        // model mismatch = miss (a different model's translation is not reused)
        assert!(
            get_cached_translation_pg(&pool, q, "other-model")
                .await
                .is_none(),
            "different model must miss"
        );

        // upsert refreshes (same model re-translation updates the row)
        let params2 = serde_json::json!({ "q": "nebula", "complete": false });
        set_cached_translation_pg(&pool, q, "test-model", &params2).await;
        let got2 = get_cached_translation_pg(&pool, q, "test-model").await;
        assert_eq!(got2, Some(params2), "upsert must refresh the cached params");

        sqlx::query("DELETE FROM ask_translation_cache WHERE nl_query = $1")
            .bind(normalize_nl_query(q))
            .execute(&pool)
            .await
            .expect("cleanup delete");
    }
}
