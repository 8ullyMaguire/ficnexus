# Part 17 — Ask the Archive

Ask the Archive turns natural-language questions like "completed slow-burn Dramione over 50k, no major character death" into structured FicHub search queries using Ollama. This part builds the Ask endpoint, the LLM translation layer, caching, and the fallback when Ollama is down.

---

## 17.1 Overview

Open `src/search/ask.rs`. The file starts with a detailed module comment:

```rust
// src/search/ask.rs (lines 1-24)
//! Ask the Archive — natural-language search via Ollama.
//!
//! `POST /api/search/ask` turns a free-text request ("dark harry potter
//! completed, over 50k words") into a FicHub **v2 search-query string** by
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
```

### Why Ask the Archive?

Users don't want to learn boolean query syntax. They want to type "completed Harry Potter fanfiction over 50k words with slow burn" and get results. Ask the Archive bridges that gap by using an LLM to translate natural language into the structured query language that the search builder understands.

### The key design decisions

1. **Same search pipeline**: Ask emits v2 syntax that goes through the exact same `apply_query_parse` → builder pipeline as regular search. No second, weaker schema.
2. **Never trust the LLM**: Output is stripped of fences/labels, length-capped, and validated. A hallucinating model cannot inject SQL or degrade the query.
3. **Graceful degradation**: If Ollama is down, the raw NL string becomes `q` and the search still runs as plain full-text search.
4. **Caching**: Translations are cached in Redis (24-hour TTL) and archived in Postgres.

---

## 17.2 Request and response types

```rust
// src/search/ask.rs (lines 59-90, excerpt)
/// Maximum NL query length (chars). Prevents prompt-abuse and absurd cache
/// keys. 500 chars is far beyond any realistic ask.
pub const MAX_ASK_LEN: usize = 500;

/// Number of results /api/search/ask returns. Natural-language asks are
/// discovery requests — a solid first page is the point.
pub const ASK_PER_PAGE: usize = 20;

/// Request body for `POST /api/search/ask`.
#[derive(Debug, Deserialize)]
pub struct AskRequest {
    /// Natural-language query, e.g. "completed Harry Potter over 50k words"
    pub q: String,
    /// Optional: client_id for caching + dedup (anonymous users get a fresh one).
    pub client_id: Option<String>,
}

/// Response envelope for `POST /api/search/ask`.
#[derive(Debug, Serialize)]
pub struct AskResponse {
    pub err: i32,
    pub results: Vec<SearchHit>,
    pub total: i64,
    pub translated: bool,
    /// The v2 query string the LLM produced (or the raw NL if not translated).
    pub applied_query: String,
    /// The interpreted parameters the UI renders as "Interpreted as" chips.
    pub applied_params: std::collections::HashMap<String, String>,
    /// When `translated: true`, the cache key for this translation (Redis + PG).
    pub cache_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub source: String,
    pub words: i64,
    pub chapters: i32,
    pub status: String,
    pub description: String,
    pub updated: Option<String>,
    pub kudos_count: i64,
    pub comment_count: i64,
    pub bookmark_count: i64,
}
```

---

## 17.3 The ask handler

```rust
// src/search/ask.rs (conceptual — ask_handler)
use crate::search::ask_cache::{get_cached_ask_response, get_cached_translation, set_cached_ask_response, set_cached_translation, set_cached_translation_pg};
use crate::search::builder::SearchParams;
use crate::search::parser::{extract_excluded_terms, extract_field_queries, extract_fielded_terms, extract_text_tsquery, parse_query};
use crate::search::routes::{run_search, SearchResponseEnvelope};

pub async fn ask_handler(
    State(state): State<Arc<AppState>>,
    HeaderMap headers,
    Json(body): Json<AskRequest>,
) -> Result<Json<AskResponse>, AppError> {
    // Validate input
    if body.q.trim().is_empty() {
        return Err(AppError::BadRequest("Query cannot be empty".to_string()));
    }
    if body.q.len() > MAX_ASK_LEN {
        return Err(AppError::BadRequest(format!("Query too long (max {} chars)", MAX_ASK_LEN)));
    }

    let client_id = body.client_id.unwrap_or_else(|| generate_client_id());
    let cache_key = format!("ask:{}:{}", client_id, hash_string(&body.q));

    // Check cache first
    if let Some(cached) = get_cached_ask_response(&state.redis, &cache_key).await? {
        return Ok(Json(cached));
    }

    // Try to translate via Ollama
    let (translated, applied_query, applied_params) = match translate_query(&state, &body.q).await {
        Ok(result) => result,
        Err(_) => {
            // Ollama down / failed — fall back to raw NL as q
            tracing::warn!("Ask translation failed, falling back to raw query");
            (false, body.q.clone(), std::collections::HashMap::new())
        },
    };

    // Run the search through the normal pipeline
    let mut params = SearchParams::default();
    params.q = Some(applied_query.clone());
    apply_query_parse(&mut params);
    let search_result = run_search(&state, &params, ASK_PER_PAGE, 1).await?;

    let response = AskResponse {
        err: 0,
        results: search_result.hits,
        total: search_result.total,
        translated,
        applied_query,
        applied_params,
        cache_key: Some(cache_key.clone()),
    };

    // Cache the response
    set_cached_ask_response(&state.redis, &cache_key, &response, CACHE_TTL_SECS).await?;

    // Archive the translation in Postgres (write-only)
    if translated {
        set_cached_translation_pg(&state.db, &cache_key, &body.q, &applied_query).await?;
    }

    Ok(Json(response))
}
```

### Breakdown

**Validation**: The query is checked for emptiness and length. MAX_ASK_LEN is 500 chars.

**Client ID**: Used for caching and deduplication. Anonymous users get a fresh UUID.

**Cache check**: Before hitting Ollama, check if this exact query from this client was already translated. If so, return the cached response.

**Translation**: The `translate_query` function calls Ollama with a prompt that asks the model to produce a v2 search query string. If it fails (Ollama down, timeout, garbage output), the raw NL string is used as `q` and `translated` is false.

**Search**: The translated (or raw) query goes through `apply_query_parse` and `run_search` — the exact same pipeline as regular search.

**Caching**: The response is cached in Redis. The translation is also archived in Postgres (write-only — the handler never reads it back).

---

## 17.4 The translation function

```rust
// src/search/ask.rs (conceptual — translate_query)
async fn translate_query(
    state: &AppState,
    nl_query: &str,
) -> Result<(bool, String, std::collections::HashMap<String, String>), AppError> {
    // Build the prompt
    let prompt = format!(
        r#"You are a search query translator for a fanfiction archive.
Convert the following natural language request into a FicHub v2 search query string.

Rules:
- Output ONLY the query string, no explanation, no markdown, no quotes.
- Use these operators: AND, OR, NOT, -, title:, author:, fandom:, char:, ship:, attr:, rating:, status:, words:, chapters:, kudos:, comments:, bookmarks:, min_words:, max_words:, min_chapters:, max_chapters:, complete:, source:
- Use quotes for phrases: "slow burn"
- Use - for exclusion: -angst
- Use ranges: words:50000-100000, chapters:1-10
- Use booleans: coffee AND angst, fluff OR humor
- If you cannot determine a filter, leave it out (don't guess).

Natural language request:
{}\n"#,
        nl_query
    );

    // Call Ollama
    let response = state.ollama.generate_json(&prompt, &state.config.ollama_chat_model).await?;

    // Parse and validate the response
    let query_string = parse_llm_response(&response)?;

    // Extract applied parameters for the UI
    let params = extract_applied_params(&query_string);

    Ok((true, query_string, params))
}
```

### Breakdown

**Prompt engineering**: The prompt tells the LLM exactly what to do and what operators are available. It emphasizes "output ONLY the query string" to prevent chit-chat.

**Ollama call**: Uses the `OllamaClient` to call the chat model (configured via `OLLAMA_CHAT_MODEL` env var, default likely `llama3.1:8b`).

**Response parsing**: The `parse_llm_response` function strips markdown fences, labels, and other LLM artifacts. It's strict — if the response doesn't look like a valid query, it errors.

**Parameter extraction**: The `extract_applied_params` function parses the query string and extracts the key-value pairs for the UI's "Interpreted as" chips.

---

## 17.5 Response parsing and validation

```rust
// src/search/ask.rs (conceptual — parse_llm_response)
fn parse_llm_response(response: &str) -> Result<String, AppError> {
    let trimmed = response.trim();

    // Strip markdown code fences
    let stripped = trimmed
        .strip_prefix("```")
        .or_else(|| trimmed.strip_prefix("```sql"))
        .or_else(|| trimmed.strip_prefix("```text"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(trimmed);

    // Strip "Query:" or "Search:" labels
    let final_query = stripped
        .split('\n')
        .next()  // Take first line only
        .unwrap_or("")
        .strip_prefix("Query:")
        .or_else(|| stripped.strip_prefix("Search:"))
        .or_else(|| stripped.strip_prefix("v2:"))
        .unwrap_or(stripped)
        .trim()
        .to_string();

    // Validate: must be non-empty and reasonable length
    if final_query.is_empty() {
        return Err(AppError::BadRequest("LLM returned empty query".to_string()));
    }
    if final_query.len() > 500 {
        return Err(AppError::BadRequest("LLM query too long".to_string()));
    }

    // Basic sanity: if it looks like a sentence (lots of spaces, no operators),
    // it's probably not a valid query
    let operator_count = final_query.matches(|c: char| "!@#$%^&*()_+-=[]{}|;:',.<>?/".contains(c)).count();
    if final_query.split_whitespace().count() > 20 && operator_count == 0 {
        return Err(AppError::BadRequest("LLM did not produce a valid query".to_string()));
    }

    Ok(final_query)
}
```

### Breakdown

**Strip fences**: Removes ``` code blocks.

**Strip labels**: Removes "Query:", "Search:", "v2:" prefixes.

**Length check**: Must be non-empty and under 500 chars.

**Sanity check**: If the response looks like a natural language sentence (many words, no operators), it's rejected. This prevents the LLM from just echoing the input.

---

## 17.6 Caching

```rust
// src/search/ask_cache.rs (conceptual)
use redis::AsyncCommands;

const CACHE_TTL_SECS: u64 = 24 * 3600;  // 24 hours

pub async fn get_cached_ask_response(
    redis: &redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<Option<AskResponse>, AppError> {
    let value: Option<String> = redis.get(key).await.ok().flatten()?;
    if let Some(json_str) = value {
        let response: AskResponse = serde_json::from_str(&json_str)
            .map_err(|e| AppError::Internal(format!("Cache parse error: {}", e)))?;
        return Ok(Some(response));
    }
    Ok(None)
}

pub async fn set_cached_ask_response(
    redis: &redis::aio::MultiplexedConnection,
    key: &str,
    response: &AskResponse,
    ttl_secs: u64,
) -> Result<(), AppError> {
    let json_str = serde_json::to_string(response)
        .map_err(|e| AppError::Internal(format!("Serialization error: {}", e)))?;
    redis.set_ex(key, json_str, ttl_secs as usize).await
        .map_err(|e| AppError::Internal(format!("Cache set error: {}", e)))?;
    Ok(())
}

pub async fn get_cached_translation(
    redis: &redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<Option<String>, AppError> {
    let value: Option<String> = redis.get(key).await.ok().flatten()?;
    Ok(value)
}

pub async fn set_cached_translation(
    redis: &redis::aio::MultiplexedConnection,
    key: &str,
    query: &str,
    ttl_secs: u64,
) -> Result<(), AppError> {
    redis.set_ex(key, query, ttl_secs as usize).await
        .map_err(|e| AppError::Internal(format!("Cache set error: {}", e)))?;
    Ok(())
}

pub async fn set_cached_translation_pg(
    db: &PgPool,
    cache_key: &str,
    original_query: &str,
    translated_query: &str,
) -> Result<(), AppError> {
    // Write-only archive — never read back
    sqlx::query(
        "INSERT INTO ask_translations (cache_key, original_query, translated_query, created_at)
         VALUES ($1, $2, $3, NOW())
         ON CONFLICT (cache_key) DO NOTHING",
    )
    .bind(cache_key)
    .bind(original_query)
    .bind(translated_query)
    .execute(db)
    .await?;
    Ok(())
}
```

---

## 17.7 Frontend: Ask the Archive page

The Ask page (`/ask`) has a text input, a "Ask" button, and displays results with an "Interpreted as" chip showing what the LLM translated the query to.

```svelte
<!-- frontend/src/routes/ask/+page.svelte (conceptual) -->
<script lang="ts">
  import { askSearch } from '$lib/api/search';

  let query = $state('');
  let results = $state([]);
  let total = $state(0);
  let translated = $state(false);
  let appliedQuery = $state('');
  let loading = $state(false);
  let error = $state('');

  async function search() {
    if (!query.trim()) return;
    loading = true;
    error = '';
    try {
      const res = await askSearch(query);
      if (res.err !== 0) throw new Error(res.msg || 'Ask failed');
      results = res.results;
      total = res.total;
      translated = res.translated;
      appliedQuery = res.applied_query;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Unknown error';
    } finally {
      loading = false;
    }
  }

  async function turnIntoRequest() {
    // Convert empty ask into a Fic Request in one click
    const res = await createRequest({
      title: `Ask: ${query}`,
      body: `Natural language search: "${query}"\nInterpreted as: ${appliedQuery}`,
    });
    if (res.err === 0) {
      goto(`/requests/${res.id}`);
    }
  }
</script>

<div class="ask-page">
  <h1>Ask the Archive</h1>
  <p class="subtitle">Describe what you're looking for in plain English.</p>

  <div class="ask-box">
    <textarea
      bind:value={query}
      placeholder="e.g. completed Harry Potter fanfiction over 50k words with slow burn, no major character death"
      rows="4"
    ></textarea>
    <div class="ask-actions">
      <button onclick={search} disabled={loading}>Ask</button>
      <button onclick={turnIntoRequest} disabled={!query.trim()}>Turn into Request</button>
    </div>
  </div>

  {#if loading}
    <p class="loading">Thinking...</p>
  {:else if error}
    <p class="error">{error}</p>
    {#if !translated}
      <p class="hint">Ollama is unavailable. Showing results for raw query.</p>
    {/if}
  {:else if results.length > 0}
    {#if translated}
      <div class="interpreted">
        <strong>Interpreted as:</strong>
        <code>{appliedQuery}</code>
      </div>
    {/if}

    <div class="results">
      <p>{total} results</p>
      {#each results as hit}
        <a href={`/works/${hit.url_id}`} class="result">
          <h3>{hit.title}</h3>
          <p>by {hit.author} · {hit.source} · {hit.words} words</p>
          <p>{hit.description}</p>
        </a>
      {/each}
    </div>
  {:else}
    <p class="empty">No results found. Try different keywords.</p>
  {/if}
</div>
```

---

## 17.8 The "empty ask → request" flow

One of the powerful features: an empty ask turns into a Fic Request in one click. The request page also has an "Ask the Archive" box that surfaces in-library matches as one-click answers.

This creates a tight loop:
1. User types a natural-language search.
2. If no results, they click "Turn into Request".
3. The request is created with the original query and the interpreted v2 query.
4. Other users can answer the request with works that match.

---

## 17.9 Try It Yourself: ask a question

### Step 1: Ask the archive

```bash
curl -s -X POST http://localhost:8000/api/search/ask \
  -H "Content-Type: application/json" \
  -d '{"q": "completed Harry Potter over 50000 words"}' | python3 -m json.tool
```

**Expected**: If Ollama is running, you get translated results. If not, you get raw full-text search results with `translated: false`.

### Step 2: Check the response

Look for:
- `translated: true` if Ollama translated the query.
- `applied_query`: the v2 query string.
- `results`: the search hits.

---

## 17.10 What you have now

- You understand Ask the Archive: NL → LLM translation → v2 query → search pipeline.
- You understand the degradation contract: Ollama down → raw NL as full-text search.
- You understand caching: Redis (24h TTL) + Postgres archive.
- You understand response parsing: strip fences/labels, validate, sanity check.
- You understand the frontend Ask page: text input, results, "Interpreted as" chip.
- You understand the empty-ask-to-request flow.
- You tested Ask with curl.

Next: Part 18 — Bounties and Reputation. You will build the bounty system for requesting specific fics and the reputation/XP progression system.

---

*End of Part 17. On to [Part 18 — Bounties and Reputation](./18-bounties-reputation/18-bounties-reputation.md).*
