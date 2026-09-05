//! Ask the Archive — natural-language search via Ollama.
//!
//! `POST /api/search/ask` turns a free-text request ("dark harry potter
//! completed, over 50k words") into a FicNexus **v2 search-query string** by
//! asking Ollama (llama3.1:8b) to emit a strict query-string, validating the
//! model output (stripped of fences/labels + length-capped — never trust the
//! LLM blindly), then running that query through the exact same pipeline
//! (`apply_query_parse` → builder) as `GET /api/search`. Same response shape
//! plus a `translated` flag and the `applied_params` the UI renders as an
//! "Interpreted as" chip.
//!
//! Because ask now emits v2 syntax, it speaks the SAME search language as the
//! advanced search box. There is no second, weaker parameter schema for the
//! model to invent: only operators the parser already understands can survive
//! (everything else is a plain keyword), and every value is bound as a SQL
//! parameter, so a hallucinating model cannot inject or degrade the query.
//!
//! Degradation contract: Ollama down / times out / returns garbage → the raw
//! NL string becomes `q` and the search still runs (plain full-text search,
//! `translated: false`). A translation, once produced, is cached in Redis
//! (24-hour TTL) so repeat asks never hit the model again; every translation
//! is ALSO saved to Postgres as a write-only archive (the handler never reads
//! it back).

use axum::{Json, extract::State, http::HeaderMap};
// `async fn` requires edition 2018+; the crate is edition 2024 but the
// lint wrapper below runs a bare `rustc` — this import is a no-op that
// makes the edition explicit for tooling.
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::search::builder::SearchParams;
use crate::search::parser::{
    extract_excluded_terms, extract_field_queries, extract_fielded_terms, extract_text_tsquery,
    parse_query,
};
use crate::search::routes::{SearchResponseEnvelope, run_search};
use crate::server::AppState;

use super::ask_cache::{
    get_cached_ask_response, get_cached_translation, set_cached_ask_response,
    set_cached_translation, set_cached_translation_pg,
};

/// Maximum NL query length (chars). Prevents prompt-abuse and absurd cache
/// keys. 500 chars is far beyond any realistic ask.
pub const MAX_ASK_LEN: usize = 500;

/// Number of results /api/search/ask returns. Natural-language asks are
/// discovery requests — a solid first page is the point.
pub const ASK_PER_PAGE: usize = 20;

/// Request body for `POST /api/search/ask`.
#[derive(Debug, Deserialize)]
pub struct AskRequest {
    /// The natural-language request, e.g. "dark harry potter complete over 50k".
    pub q: String,
}

/// The prompt sent to Ollama. The model is asked for a bare v2 query string
/// — NO JSON, no explanation. It may only emit operators the search parser
/// understands; anything else degrades to a plain keyword. Two hard rules
/// are drilled in: never invent tag ids / type ids (tag filters are resolved
/// server-side by name, and the model must not guess ids), and never invent
/// word-count floors or other numbers from thin air — only map explicit
/// size/completion/source language to operators.
pub fn build_ask_prompt(nl_query: &str) -> String {
    format!(
        r#"You translate a natural-language fanfiction search request into ONE FicNexus search query string.

The query string uses this syntax (each piece optional, add ONLY what the request states):

- topics / plain keywords: harry potter time travel
- main / protagonist character: @char:Harry
- any character (not necessarily main): char:Hermione
- romantic pairing: romship:Harry/Draco     platonic pairing: platship:Harry&Ron
- fandom: fandom:Harry Potter    primary fandom: @fandom:Harry Potter
- freeform trope/attribute: attr:"Slow Burn"
- rating: rating:Explicit    category: category:Gen
- archive warning: warning:"Major Character Death"
- completion: status:complete  (or do NOT add anything for "wip"/"in progress" — an absent status is the default; use status:complete only for explicit "completed"/"finished"/"complete only")
- word count: words:>50k   words:10k-100k   words:5000
- chapter count: chapters:5-50
- dates: published:2018-2021   updated:>2019
- site source: source:archiveofourown.org   source:fanfiction.net
- negation: -angst   NOT attr:Dark

Rules:
- ALWAYS start with the topic keywords as plain words.
- ONLY add a structured operator when the request EXPLICITLY states it. Never invent numbers or presence from thin air.
  - Size words only: "long fic" → words:>100k, "over 50k" → words:>50000, "short" → words:<20000, "oneshot" → words:<5000.
  - Completion ONLY: "completed"/"finished"/"complete only" → status:complete. "wip"/"in progress" → nothing.
  - Source ONLY from an explicit site name: "ao3"/"archiveofourown" → source:archiveofourown.org, "ffn"/"fanfiction" → source:fanfiction.net.
- NEVER invent tag type ids, tag ids, character ids or numeric tag ids — tag filters are matched by NAME server-side; include tag names only.
- NEVER output JSON, markdown, code fences, backticks, explanations, or quotes around the query. Output the bare query string and nothing else.

Examples:
Request: "dark harry potter completed over 50k"
Output: harry potter @char:harry status:complete words:>50k

Request: "fluffy hinata and kageyama, wip, from ao3"
Output: hinata kageyama status:ongoing? source:archiveofourown.org

Request: "finished drarry fics over 100k words"
Output: drarry status:complete words:>100k

Request: "anything with time travel and angst"
Output: time travel angst

Request: "{nl_query}"
Output:"#
    )
}

/// Validate + sanitise the model's raw reply into a usable v2 query string.
///
/// Defensive by construction — the model's output is attacker-controllable
/// input:
/// * Surrounding code fences / backticks / a trailing label line are dropped.
/// * Multi-line rambling (a re-emitted prompt, extra prose) is rejected.
/// * Length is capped at [`MAX_ASK_LEN`].
/// * The result must parse to at least one usable term (text, fielded,
///   field-query, or excluded); a string of only structural junk is rejected.
///
/// Returns `None` when the reply is unusable — the caller falls back to a
/// plain keyword search with the raw NL string as `q`.
pub fn validate_llm_params(raw: &str) -> Option<String> {
    let mut s = raw.trim().to_string();
    // Strip a single code-fence (start and/or end) and wrapping backticks.
    for fence in ["```", "``", "`"] {
        if s.starts_with(fence) {
            s = s.trim_start_matches(fence).trim_start().to_string();
        }
        if s.ends_with(fence) {
            s = s.trim_end_matches(fence).trim_end().to_string();
        }
    }
    // A trailing newline means multi-line output — not a bare query. Reject
    // (plain fallback).
    if s.contains('\n') {
        return None;
    }
    if s.is_empty() {
        return None;
    }
    if s.chars().count() > MAX_ASK_LEN {
        return None;
    }
    // Require a real token: pure punctuation ("{}", "::", "---") is never a
    // usable query even though the tolerant parser may yield an empty-field
    // Fielded term from it.
    if !s.chars().any(|c| c.is_alphanumeric()) {
        return None;
    }
    // Ensure it parses to something real, not structural junk.
    let expr = parse_query(&s);
    if extract_text_tsquery(&expr).is_empty()
        && extract_field_queries(&expr).is_empty()
        && extract_fielded_terms(&expr).is_empty()
        && extract_excluded_terms(&expr).is_empty()
    {
        return None;
    }
    Some(s.to_string())
}

/// Best-effort analytics: log the ORIGINAL NL query with a marker so the
/// admin search-analytics view shows Ask traffic separately from plain
/// searches. Mirrors `search_handler`'s best-effort insert — any failure is
/// swallowed.
async fn log_ask_analytics(
    db: &sqlx::PgPool,
    nl_query: &str,
    total: i64,
    translated: bool,
    client_id: Option<&str>,
    user_id: Option<i32>,
) {
    let marked = if translated {
        format!("[ask-translated] {nl_query}")
    } else {
        format!("[ask-plain] {nl_query}")
    };
    if let Err(e) =
        crate::db::queries::insert_search_query(db, &marked, total, None, client_id, user_id).await
    {
        tracing::warn!(error = %e, "failed to log ask query for analytics");
    }
}

/// Build `SearchParams` from a v2 query string + the authenticated user id.
/// The query string is fed straight into the shared pipeline (`run_search`
/// calls `apply_query_parse`), so an ask matches exactly like the same string
/// typed into the advanced search box.
fn translated_into_search_params(v2_query: &str, user_id: Option<i32>) -> AppResult<SearchParams> {
    Ok(SearchParams {
        q: Some(v2_query.to_string()),
        user_id,
        per_page: Some(ASK_PER_PAGE),
        ..Default::default()
    })
}

/// `POST /api/search/ask`
///
/// 1. Rate-limit like a write (search-tier bucket — very high ceiling, but
///    it keeps the path honest).
/// 2. Redis cache lookup (24-hour TTL, per exact NL string) — the ONLY
///    lookup tier. PG is write-only archive.
/// 3. Ollama strict-query translation → validate → (cache Redis + archive
///    PG) → search.
/// 4. Any Ollama/validation failure → plain search with the raw NL as `q`.
/// 5. Response = the standard search envelope + `translated`,
///    `nl_query`, `applied_params` (the v2 query the UI renders as the
///    "Interpreted as" chip / builds a /search?q= link from).
pub async fn ask_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    auth: crate::routes::auth::AuthUser,
    Json(body): Json<AskRequest>,
) -> Result<Json<Value>, AppError> {
    use crate::limiter::Tier;

    let nl = body.q.trim().to_string();
    if nl.is_empty() {
        return Err(AppError::BadRequest("q must not be empty".to_string()));
    }
    if nl.chars().count() > MAX_ASK_LEN {
        return Err(AppError::BadRequest(format!(
            "q too long (max {MAX_ASK_LEN} chars)"
        )));
    }

    let client_id = headers
        .get("x-client-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // ── Rate limit (search tier: very high ceiling; fail-open) ─────────
    let ip = crate::limiter::client_ip_from_headers(
        headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()),
        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
    );
    if let crate::limiter::TieredRateLimitResult::Wait(secs) = state
        .rate_limiter
        .check(ip, client_id.as_deref(), Tier::Search)
        .await
    {
        return Err(AppError::RateLimited(secs));
    }

    // ── 1. Cache lookup: Redis only (24h TTL). Best-effort: a miss/error
    // just falls through to the model. Only the TRANSLATION is cached —
    // search results always come from a live search below. Postgres is a
    // write-only archive (never read here). ─────────────────────────────
    // Use a DEDICATED connection, not the shared `state.redis`: the
    // bookmark-import worker parks an unbounded BRPOP on the shared
    // multiplexed connection (see `health_redis`), so a SETEX queued behind
    // it can time out and silently drop the cache write. A fresh local
    // Redis connection per request is cheap and this endpoint is
    // rate-limited. Fall back to the shared connection only if the
    // dedicated one cannot be opened.
    let mut redis = match redis::Client::open(state.config.redis_url.as_str()) {
        Ok(client) => match client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(_) => state.redis.clone(),
        },
        Err(_) => state.redis.clone(),
    };
    let model = state.config.ollama_chat_model.clone();

    // ── 0. Full AskResponse cache (1h TTL, hash key) — avoids both Ollama
    // and the DB search on repeat queries. Checked BEFORE translation cache.
    // keep_alive:30m in Ollama client keeps the model warm; this cache makes
    // cold ~17.6s hits disappear for hot queries.
    if let Some(cached_response) = get_cached_ask_response(&mut redis, &nl).await {
        return Ok(Json(cached_response));
    }

    if let Some(cached) = get_cached_translation(&mut redis, &nl).await {
        let v2: Option<String> = serde_json::from_value(cached.clone())
            .ok()
            // Pre-v2 cache rows are a JSON object `{q,min_words,...}` — not a
            // v2 string; drop them to the plain search (they'll miss the
            // model and re-translate), never trust a wrong-shaped row.
            .or_else(|| cached.as_str().map(|s| s.to_string()));
        let v2 = v2.unwrap_or_else(|| nl.clone());
        tracing::debug!("ask translation cache hit (redis) for {nl:?}");
        let result =
            run_ask_search(&state, &nl, &v2, true, client_id.as_deref(), auth.user_id).await;
        // Also populate the full response cache on this path (translation hit
        // but response miss — e.g. first request after deploy).
        if let Ok(Json(ref val)) = result {
            set_cached_ask_response(&mut redis, &nl, val).await;
        }
        return result;
    }

    // ── 2. Ollama translation (best-effort) ─────────────────────────────
    let mut translated = false;
    let v2: String = match translate_with_ollama(&state, &nl).await {
        Some(q) => {
            translated = true;
            // Cache the fresh translation: Redis (lookup tier) + PG archive
            // (write-only; never read back). Both store the v2 query string
            // as a JSON string value. Fail-open.
            let v = serde_json::to_value(&q).unwrap_or_else(|_| Value::Null);
            set_cached_translation(&mut redis, &nl, &v).await;
            set_cached_translation_pg(&state.db, &nl, &model, &v).await;
            q
        }
        None => {
            tracing::debug!("ask translation unavailable — plain search for {nl:?}");
            nl.clone()
        }
    };

    let result = run_ask_search(
        &state,
        &nl,
        &v2,
        translated,
        client_id.as_deref(),
        auth.user_id,
    )
    .await;
    if let Ok(Json(ref val)) = result {
        set_cached_ask_response(&mut redis, &nl, val).await;
    }
    result
}

/// Public wrapper for the Fic Requests archivist answer: translate an NL
/// request into a v2 search query string (Ollama, best-effort) — falls back
/// to the raw NL so callers always get a usable search link. Never fails.
pub async fn translate_for_request(state: &Arc<AppState>, nl: &str) -> String {
    translate_with_ollama(state, nl)
        .await
        .unwrap_or_else(|| nl.to_string())
}

/// Call Ollama and validate the reply. Returns `None` on any failure
/// (down / timeout / invalid output) — never panics, never propagates the
/// error; the caller falls back to plain search.
async fn translate_with_ollama(state: &Arc<AppState>, nl: &str) -> Option<String> {
    let prompt = build_ask_prompt(nl);

    // Hard timeout: Ollama can hang (cold model load can take 10-30s on
    // first generate). The shared reqwest client's 30s cap is the outer
    // bound; we allow the translation step 25s so a cold llama3.1:8b load
    // still succeeds while a genuinely stuck model can't stall the endpoint
    // past that.
    let reply = match tokio::time::timeout(
        std::time::Duration::from_secs(25),
        state
            .ollama
            .generate_json(&prompt, &state.config.ollama_chat_model),
    )
    .await
    {
        Ok(Ok(reply)) => reply,
        Ok(Err(e)) => {
            tracing::debug!("ask translation ollama error: {e}");
            return None;
        }
        Err(_) => {
            tracing::debug!("ask translation timed out");
            return None;
        }
    };

    validate_llm_params(&reply)
}

/// Run the real search with the translated v2 query string and wrap the
/// standard search response with the Ask metadata. Errors from the search
/// itself are propagated (a broken search is a real failure, not a fallback
/// case).
async fn run_ask_search(
    state: &Arc<AppState>,
    nl: &str,
    v2_query: &str,
    translated: bool,
    client_id: Option<&str>,
    user_id: Option<i32>,
) -> Result<Json<Value>, AppError> {
    let params = translated_into_search_params(v2_query, user_id)?;

    let envelope: SearchResponseEnvelope = run_search(state, params).await?;

    // Best-effort analytics with the NL marker (never fails the request).
    log_ask_analytics(
        &state.db,
        nl,
        envelope.total,
        translated,
        client_id,
        user_id,
    )
    .await;

    Ok(Json(json!({
        "total": envelope.total,
        "page": envelope.page,
        "per_page": envelope.per_page,
        "results": envelope.results,
        "facets": envelope.facets,
        "translated": translated,
        "nl_query": nl,
        "applied_params": v2_query,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_happy_path() {
        let s = validate_llm_params("harry potter @char:harry status:complete words:>50k")
            .expect("valid output");
        assert_eq!(s, "harry potter @char:harry status:complete words:>50k");
    }

    #[test]
    fn validate_accepts_minimal_output() {
        let s = validate_llm_params("time travel angst").expect("minimal output");
        assert_eq!(s, "time travel angst");
    }

    #[test]
    fn validate_strips_fences_and_backticks() {
        let s = validate_llm_params("```\nharry potter\n```").expect("fenced output");
        assert_eq!(s, "harry potter");
        let s = validate_llm_params("`drarry status:complete`").expect("backticked output");
        assert_eq!(s, "drarry status:complete");
    }

    #[test]
    fn validate_rejects_multiline_rambling() {
        assert!(
            validate_llm_params("harry potter\nHere are some stories with harry potter.").is_none(),
            "multi-line prose must be rejected"
        );
        assert!(
            validate_llm_params("Here is the translated query: harry potter\nextra").is_none(),
            "multi-line output rejected"
        );
    }

    #[test]
    fn validate_clamps_long_fields() {
        let long = "x".repeat(MAX_ASK_LEN + 10);
        assert!(
            validate_llm_params(&long).is_none(),
            "overlong output rejected"
        );
    }

    #[test]
    fn validate_rejects_empty_and_junk() {
        assert!(validate_llm_params("").is_none());
        assert!(validate_llm_params("   ").is_none());
        assert!(
            validate_llm_params("::").is_none(),
            "structural junk rejected"
        );
        assert!(
            validate_llm_params("{}").is_none(),
            "json braces not a query"
        );
    }

    #[test]
    fn prompt_embeds_query_and_rules() {
        let p = build_ask_prompt("dark harry potter completed over 50k");
        assert!(p.contains("dark harry potter completed over 50k"));
        assert!(p.contains("status:complete"));
        assert!(p.contains("words:>50k"));
        assert!(p.contains("source:archiveofourown.org"));
        // The no-ids rule is in the prompt, not in the schema.
        assert!(p.contains("tag ids"));
    }

    #[test]
    fn ask_request_requires_q() {
        // Body without q fails serde deserialization (missing field).
        let err = serde_json::from_str::<AskRequest>(r#"{}"#);
        assert!(err.is_err());
    }
}
