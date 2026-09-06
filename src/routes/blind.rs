//! Blind Date with a Fic — a discovery endpoint.
//!
//! `GET /api/blind-date` returns a random eligible fic with the title and
//! fandom hidden: only a spoiler-free description, word count, chapter
//! count, status, and the top 3 "core tropes" (character/freeform tags by
//! score). The UI can pass `?exclude=url_id1,url_id2` to avoid repeats.
//!
//! `GET /api/blind-date/reveal?url_id=...&nonce=...&sig=...` returns the
//! full metadata (title + author + source + fandom) for the current
//! blind-date fic once the user chooses to reveal it. The reveal is
//! HMAC-signed with a per-fic nonce so the title/fandom never appears in
//! the discovery payload — the frontend can only fetch it after showing
//! the summary first.

use axum::Json;
use axum::extract::{Query, State};
use rand::prelude::*;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::AppError;
use crate::server::AppState;

/// Number of core tropes returned per fic.
pub const TROPE_COUNT: usize = 3;

/// Query parameters for the blind-date endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct BlindDateParams {
    /// Optional opaque client identifier for analytics.
    pub client_id: Option<String>,
    /// Comma-separated list of `url_id`s to skip (so the UI can avoid
    /// showing the same fic twice in a row).
    pub exclude: Option<String>,
}

/// Pick the top `n` tag names by score (ties broken by name for
/// determinism). Pure function — unit-tested.
pub fn top_tropes(tags: Vec<(String, i32)>, n: usize) -> Vec<String> {
    let mut sorted = tags;
    // Higher score first; ties → lexicographic name so the result is stable.
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted.into_iter().take(n).map(|(name, _)| name).collect()
}

/// GET /api/blind-date — a random eligible fic, title & fandom hidden.
///
/// Eligible = has a non-empty description AND is not blacklisted. Status is
/// intentionally not filtered (both 'complete' and ongoing fics are fair
/// game for discovery). The pick is a plain `ORDER BY random() LIMIT 1`;
/// the `exclude` list is bound as a Postgres array so it can't be
/// injected.
pub async fn blind_date_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<BlindDateParams>,
) -> Result<Json<Value>, AppError> {
    let exclude: Vec<String> = params
        .exclude
        .as_deref()
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    // 1. Pull up to 200 eligible fics (capped for Thompson sampling perf).
    let eligible: Vec<(String, String, i64, i32, String)> = sqlx::query_as(
        r#"
        SELECT fi.id, fi.description, fi.words, fi.chapters, fi.status
        FROM fic_info fi
        WHERE btrim(fi.description) <> ''
          AND NOT EXISTS (SELECT 1 FROM fic_blacklist b WHERE b.url_id = fi.id)
          AND ($1::text[] IS NULL OR NOT (fi.id = ANY ($1::text[])))
        ORDER BY random()
        LIMIT 200
        "#,
    )
    .bind(if exclude.is_empty() { None } else { Some(&exclude) })
    .fetch_all(&state.db)
    .await?;

    if eligible.is_empty() {
        return Ok(Json(json!({ "err": 0, "fic": null })));
    }

    // 2. Load bandit arms for the eligible set (strategy = 'blind_date').
    //    Missing arms default to (1, 1) = uniform prior.
    let url_ids: Vec<&str> = eligible.iter().map(|(id, ..)| id.as_str()).collect();
    let arms: Vec<(String, f64, f64)> = {
        let rows: Vec<(String, f64, f64)> = sqlx::query_as(
            r#"SELECT work_id, alpha, beta FROM rec_bandit_arms
               WHERE strategy = 'blind_date' AND work_id = ANY($1)"#,
        )
        .bind(&url_ids)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
        rows
    };
    let arm_map: std::collections::HashMap<&str, (f64, f64)> = arms
        .iter()
        .map(|(id, a, b)| (id.as_str(), (*a, *b)))
        .collect();

    // 3. Thompson sample: draw from Beta(alpha, beta) for each arm and pick max.
    use rand_distr::{Beta, Distribution};
    let mut rng = rand::thread_rng();
    let best = eligible
        .iter()
        .map(|(id, ..)| id.as_str())
        .map(|id| {
            let (alpha, beta) = arm_map.get(id).copied().unwrap_or((1.0, 1.0));
            let sample = Beta::new(alpha, beta)
                .map(|d| d.sample(&mut rng))
                .unwrap_or(0.5);
            (id, sample)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(id, _)| *id)
        .unwrap();

    // 4. Look up the chosen fic's full metadata.
    let (url_id, description, words, chapters, status) = eligible
        .into_iter()
        .find(|(id, ..)| id == best)
        .unwrap();

    // 5. Log the impression to usage_events (best-effort; never fails the pick).
    let client_id = params.client_id.as_deref().filter(|s| !s.is_empty());
    let _ = sqlx::query(
        r#"INSERT INTO usage_events (client_id, path, event_type, user_agent)
           VALUES ($1, '/api/blind-date', $2, '')"#,
    )
    .bind(client_id)
    .bind(format!("blind_date_impression:{}", url_id))
    .execute(&state.db)
    .await;

    // 6. Core tropes: character (2) + freeform (4) tags by score.
    let tag_rows: Vec<(String, i32)> = sqlx::query_as(
        r#"
        SELECT t.name, ft.score::int4
        FROM fic_tags ft
        JOIN tags t ON t.id = ft.tag_id
        WHERE ft.url_id = $1
          AND t.tag_type_id IN (2, 4)
        ORDER BY ft.score DESC, t.name ASC
        LIMIT $2
        "#,
    )
    .bind(&url_id)
    .bind(TROPE_COUNT as i32)
    .fetch_all(&state.db)
    .await?;
    let tropes = top_tropes(tag_rows, TROPE_COUNT);

    // 7. Nonce for the signed reveal.
    let nonce = random_hex();

    Ok(Json(json!({
        "err": 0,
        "fic": {
            "url_id": url_id,
            "description": description,
            "words": words,
            "chapters": chapters,
            "status": status,
            "tropes": tropes,
            "reveal": {
                "nonce": nonce,
                "sig": reveal_signature(&url_id, &nonce),
            },
        },
    })))
}

/// Query parameters for the reveal endpoint.
#[derive(Debug, Deserialize)]
pub struct RevealParams {
    pub url_id: String,
    pub nonce: String,
    pub sig: String,
}

/// GET /api/blind-date/reveal?url_id=...&nonce=...&sig=...
///
/// Returns full metadata for the fic (title + fandom) only when the HMAC
/// signature matches. The signature is `HMAC-SHA256(secret, url_id || ":" || nonce)`
/// computed server-side; the frontend obtains it from the discovery payload
/// and never sees the title until it asks for this reveal.
pub async fn blind_date_reveal_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RevealParams>,
) -> Result<Json<Value>, AppError> {
    let expected = reveal_signature(&params.url_id, &params.nonce);
    if !signatures_equal(&expected, &params.sig) {
        return Err(AppError::BadRequest("invalid reveal signature".to_string()));
    }

    let row = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, title, author, source FROM fic_info WHERE id = $1",
    )
    .bind(&params.url_id)
    .fetch_optional(&state.db)
    .await?;

    let Some((id, title, author, source)) = row else {
        return Err(AppError::NotFound("fic not found".into()));
    };

    // Fandom = the single strongest fandom tag (tag_type_id = 1) by score,
    // if any; None keeps it spoiler-light.
    let fandom: Option<String> = sqlx::query_scalar(
        r#"
        SELECT t.name
        FROM fic_tags ft
        JOIN tags t ON t.id = ft.tag_id
        WHERE ft.url_id = $1 AND t.tag_type_id = 1
        ORDER BY ft.score DESC, t.name ASC
        LIMIT 1
        "#,
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await?;

    // Log the click to usage_events for bandit training reconciliation.
    let _ = sqlx::query(
        r#"INSERT INTO usage_events (client_id, path, event_type, user_agent)
           VALUES (NULL, '/api/blind-date/reveal', $1, '')"#,
    )
    .bind(format!("blind_date_click:{}", id))
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "err": 0,
        "fic": {
            "url_id": id,
            "title": title,
            "author": author,
            "source": source,
            "fandom": fandom,
        },
    })))
}

/// HMAC-SHA256(secret, url_id || ":" || nonce), hex-encoded.
///
/// The secret is the app's JWT_SECRET env (already in the process env for
/// the service; tests set it). We use the JWT secret rather than adding a
/// new config knob so no migration/config change is required — and the
/// value is never transmitted, only used to sign/verify.
fn reveal_signature(url_id: &str, nonce: &str) -> String {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "fichub-dev-secret".into());
    hmac_sha256_hex(&secret, &format!("{url_id}:{nonce}"))
}

/// HMAC-SHA256 per RFC 2104, implemented with the project's sha2 0.11
/// (the `hmac` crate wants sha2 0.10, which would force a second sha2
/// version into the graph — not worth it for one signature).
fn hmac_sha256_hex(key: &str, msg: &str) -> String {
    use sha2::{Digest, Sha256};

    let block_size = 64usize;
    let mut k = key.as_bytes().to_vec();
    if k.len() > block_size {
        k = Sha256::digest(&k).to_vec();
    }
    k.resize(block_size, 0);
    let mut ipad = vec![0x36u8; block_size];
    let mut opad = vec![0x5cu8; block_size];
    for i in 0..block_size {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(msg.as_bytes());
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(inner_hash);
    hex::encode(outer.finalize())
}

/// Constant-time-ish comparison to avoid leaking the expected signature in
/// timing. (The signature is not a secret from the client's perspective —
/// it's in the payload — but comparing with a fixed loop avoids accidental
/// early-exit behaviour.)
fn signatures_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let a = a.as_bytes();
    let b = b.as_bytes();
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// 32-char hex nonce from `rand` (already a dependency via the scraper).
fn random_hex() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_tropes_ranks_by_score_desc() {
        let tags = vec![
            ("Fluff".to_string(), 5),
            ("Angst".to_string(), 12),
            ("Humor".to_string(), 3),
        ];
        assert_eq!(
            top_tropes(tags, 2),
            vec!["Angst".to_string(), "Fluff".to_string()]
        );
    }

    #[test]
    fn top_tropes_ties_break_alphabetically() {
        let tags = vec![
            ("Humor".to_string(), 7),
            ("Angst".to_string(), 7),
            ("Fluff".to_string(), 7),
        ];
        // All scores equal → deterministic name order, capped at n.
        assert_eq!(
            top_tropes(tags, 2),
            vec!["Angst".to_string(), "Fluff".to_string()]
        );
    }

    #[test]
    fn top_tropes_caps_at_n_and_handles_fewer() {
        let tags = vec![("Only".to_string(), 1)];
        assert_eq!(top_tropes(tags.clone(), 3), vec!["Only".to_string()]);
        assert!(top_tropes(tags, 0).is_empty());
    }

    #[test]
    fn signatures_equal_constant_time() {
        assert!(signatures_equal("abc", "abc"));
        assert!(!signatures_equal("abc", "abd"));
        assert!(!signatures_equal("abc", "abcd"));
        assert!(!signatures_equal("", "a"));
        assert!(signatures_equal("", ""));
    }

    #[test]
    fn reveal_signature_is_deterministic_and_signed() {
        // The signature depends on the secret + url_id + nonce, and is
        // stable for the same inputs.
        let s1 = reveal_signature("fic_1", "nonce1");
        let s2 = reveal_signature("fic_1", "nonce1");
        let s3 = reveal_signature("fic_2", "nonce1");
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
        assert_eq!(s1.len(), 64); // sha256 hex
    }

    #[test]
    fn hmac_matches_known_vector() {
        // RFC 4231 test case 1: key = 0x0b × 20, data = "Hi There".
        let key =
            "\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b";
        let sig = hmac_sha256_hex(key, "Hi There");
        assert_eq!(
            sig,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }
}
