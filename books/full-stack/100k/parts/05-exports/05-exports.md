# Part 5 — Exports: From URL to EPUB

> **Part 5 of 13** — Part 4 taught us how a URL becomes a `FicMetadata` and
> how every scraped body is preserved on disk. That was the *ingestion*
> half of the loop. This part is the *delivery* half: how FicHub turns
> that metadata and those chapters into actual files — EPUB, HTML, TXT,
> Markdown, MOBI, PDF, and AZW3 — and serves them back to a reader. We
> start at the top of the pipeline in `src/routes/export.rs` (the
> semaphore-guarded, double-checked export endpoint), then spend a
> chapter inside the pure-Rust EPUB builder, a chapter on the
> HTML/TXT/MD generators plus the Calibre sidecar that produces
> MOBI/PDF/AZW3, and finish with the download side: `cache_download.rs`
> and the sharded, hash-addressed cache on disk. By the end of Part 5,
> when someone pastes a URL into FicHub and gets a download link, you'll
> be able to narrate every single step between the two.

---

## Chapter 19 — The Export Pipeline: routes/export.rs

Welcome to the heart of FicHub's delivery system. Everything we built in
Parts 3 and 4 — the config, the database, the scrapers, the body cache —
converges on one endpoint: `GET /api/v0/epub?q=<url>`. Paste a URL into
FicHub, and this handler decides, in a few hundred milliseconds, whether
you get instant download links or whether the server has to build your
book from scratch.

Before we read a single line, let me draw the full journey, because
Chapter 19 is essentially a map. When `q=https://archiveofourown.org/works/21845264`
arrives, the handler runs through these stages:

1. **Gates** — parse headers, check proof-of-work and rate limits, block
   `automated=true`.
2. **Resolve** — decide whether `q` is a URL or a fic hash; find a
   scraper; fetch `FicMetadata`; upsert it into the database.
3. **Check** — blacklists first, then the export cache.
4. **Build or reuse** — if cached, return links immediately. If not,
   take the semaphore, double-check the cache, gather chapters (body
   cache or network), and generate all seven formats.
5. **Respond** — a JSON payload of hashes and `/cache/...` URLs.

The file is `src/routes/export.rs`, 962 lines of real production code.
We're going to read the parts that teach you the most.

```rust
/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}
```

The `ExportQuery` struct is the URL query-string contract: `q` is the
fic URL, `automated` is a flag for bot clients (blocked outright), and
`format` exists for future use. Axum's `Query` extractor deserializes
the query string into this struct for us — one more reminder of how much
boilerplate serde kills for free.

```rust
/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
```

The handler signature is classic Axum: extract `AppState` (which we met
in Part 2 — it carries the DB pool, Redis, the scraper registry, the
config, and the cache semaphores), extract the query params, grab the
raw headers, and return a `Result<Json<Value>, AppError>`. Note the
`Result` — every early exit uses `return Err(...)` or `return Ok(...)`,
so the happy path reads top to bottom and the error paths read like
stamps on a passport.

### 19.1 The front door: validation, rate limits, and gates

The first ~90 lines of the handler are all about *who is asking*, not
*what they asked for*. There's a proof-of-work gate for shadowbanned
clients (we'll meet the full anti-bot stack in Part 11), a tiered rate
limit keyed by real client IP *and* client ID, and a few lines of
interesting philosophy in the comments:

```rust
    // ── Tiered rate limit (download tier) ─────────────────────────────
    // Keyed by real client IP AND (ip, client_id) so shared-NAT readers are
    // not punished and identified bots are throttled per-client. A
    // shadowbanned client_id gets a much stricter bucket (friction, not a
    // hard block). Errors are treated as allow (fail-open) so a Redis hiccup
    // never blocks real downloads.
```

That comment is a mini design document: shared-NAT readers (a whole
university behind one IP) shouldn't share one bucket; identified bots
get per-client throttling; and if Redis hiccups, the *fail-open* choice
means real readers keep downloading rather than getting locked out. We
won't dwell on the limiter here — Part 11 owns it — but notice the
pattern: the export endpoint composes defenses instead of reinventing
them.

Then the validation itself:

```rust
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Check for automated flag (block if present)
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // If q looks like a hash (not a URL), look up from fic_info directly
    if !query.starts_with("http") {
        return handle_hash_lookup(&state, query, start.elapsed().as_millis() as i32).await;
    }
```

Three rules, three lines of insight:

- **Empty query** → a clean JSON error. Note the error contract: `err`
  is a negative number, `msg` is human-readable, and the original `q` is
  echoed back. This `{err, msg, ...}` envelope is FicHub's API-wide
  convention — the frontend can check `err !== 0` and show `msg`.
- **`automated=true`** → refused outright with `-10`. FicHub serves
  humans; scripts that self-identify get a friendly no.
- **Not a URL?** → treat `q` as a fic hash and take the fast path.
  `handle_hash_lookup` skips scraping entirely: it reads `fic_info`
  straight from Postgres and returns whatever exports are already cached
  (or an empty `urls` map). This is how a link like
  `/api/v0/epub?q=1_21845264` works without ever touching the network —
  you'll see this pattern again in Part 6's meta endpoints.

🧪 **Try It Yourself — hit the endpoint without a query.** If you have
FicHub running (Part 1 showed you how), fire up a terminal and run:

```bash
curl -s "http://localhost:3000/api/v0/epub" | jq
# {"err": -1, "msg": "no query", "q": ""}
```

Then try the automated flag with any URL:

```bash
curl -s "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/21845264&automated=true" | jq
# {"err": -10, "msg": "automated requests blocked"}
```

You've just exercised the two cheapest branches of the pipeline — no
scraping, no database, no disk. The endpoint answered in milliseconds
because validation happens *before* any expensive work.

⚠️ **Watch Out — `unwrap_or("")` hides a missing parameter.** If a
client sends no `q` at all, `params.q` is `None` and the handler treats
it as an empty query. That's the right call for an API, but notice what
it means for your own code: you can't distinguish "no query" from
"query is the empty string". When the distinction matters (e.g. logging
malformed requests), prefer matching on `Option` explicitly.

### 19.2 Metadata, blacklists, and the version number

With the gates passed, the handler finds a scraper, fetches metadata,
upserts it, auto-merges a *work* record, and extracts tags — a block
that reads like a checklist of everything Part 4 taught us. I'll skip
the middle (you know it chapter and verse by now) and land on the part
that's new: **the version number and the cache check**.

```rust
    // Compute cache version
    let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;

    // Check cache (try EPUB first, then all formats)
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
```

Here's the idea that makes the whole cache sound: **an export is only
valid for a specific version of its input.** Three numbers contribute:

- `state.config.export_version` — the version of FicHub's *export
  code*. Bump it (via `EXPORT_VERSION` in `.env`, which we saw in Part
  3) when the EPUB template or CSS changes, and every cached export
  becomes stale at once.
- `version_bump` — how many times this fic has been re-scraped/updated.
  When a fic changes upstream, its version changes, and old exports
  stop matching.
- `input_hash` — the content hash from the scraper (Part 4). If the
  upstream story changed, the hash changes, so even the *same version
  number* with a *different hash* misses the cache.

The `export_log` table (Part 3's migrations) records every export:
`url_id`, `version`, `etype`, `input_hash`, and the `export_hash` of the
generated file. A cache hit means: "I have already built an EPUB for
this fic, at this export version, from this exact content hash — here's
the file's hash."

💡 **Key Concept — cache invalidation by content addressing.** Instead
of storing files under a timestamp and deleting them when they get old,
FicHub names cache files *by their content* (the MD5 hash) and records
*what input they were built from*. A file never needs to be "refreshed"
— it needs to be *matched*. If the input changes, the lookup misses and
a new file is born with a new hash. The old file becomes garbage that a
sweeper can remove safely, because nothing references it anymore. This
is the same idea as Git's content-addressed object store, scaled down
to fanfiction files.

### 19.3 Cache hit: the fast path

If `find_export_log` returns a row, the handler does the *minimum work
possible*: build URLs from the stored hashes and respond.

```rust
    if let Some(export_log) = cached {
        // Cache hit - build URLs from cached hash
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

        // Other formats from cached EPUB
        for etype_str in &["html", "mobi", "pdf"] {
            if let Ok(_etype) = etype_str.parse::<EType>() {
                let e_input_hash = format!("epub:{}", epub_hash);
                if let Ok(Some(entry)) = queries::find_export_log(
                    &state.db, &meta.url_id, version, etype_str, &e_input_hash,
                ).await {
                    hashes.insert(etype_str.to_string(), entry.export_hash.clone());
                    urls.insert(etype_str.to_string(), format!("/cache/{}/{}?h={}", etype_str, meta.url_id, entry.export_hash));
                }
            }
        }
```

Read that loop carefully, because it encodes a subtle dependency: the
MOBI and PDF exports are *derived from the EPUB*, so their input hash is
`epub:<epub_hash>` — literally "this mobi was converted from an EPUB
with this hash." The `export_log` table isn't a flat list of files; it's
a tiny dependency graph, and `input_hash` is the edge.

The URL format `/cache/epub/{url_id}?h={hash}` is the contract the
download handler (Chapter 22) will enforce: the hash is a *query
parameter*, not part of the path. That lets the frontend construct URLs
without knowing the layout of the cache on disk — and lets the download
handler verify the file against its claimed hash before serving a byte.

⚠️ **Watch Out — only HTML, MOBI, and PDF ride the cached-EPUB path
here.** TXT and MD are generated directly from chapters (as we'll see in
Chapter 21), so the fast path only looks for the three Calibre-family
formats plus EPUB. If you're tracing why a cached fic returns no `txt_url`,
it's not a bug — TXT/MD are cheap enough to regenerate, so they're only
built during a full export. The response shape still includes `txt_url`
and `md_url` keys (set to `null`), keeping the frontend contract stable.

### 19.4 The semaphore and the double-check

Here's where Chapter 19 earns its outline bullet. Cache miss — two
requests for the same fic arrive at the same instant; both miss; both
decide to build. Without a guard, you'd scrape and build the same EPUB
twice, burn twice the bandwidth, and the second writer would clobber
the first's file. The classic fix is a **lock per resource**, and
FicHub's is a per-(url_id, etype) async semaphore:

```rust
    // Cache miss - generate EPUB
    // Acquire semaphore to prevent duplicate concurrent exports
    let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
    let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Check cache again (double-check pattern)
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
    if let Some(export_log) = cached {
        // Another concurrent request already generated it
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));
        ...
        return Ok(Json(json!({ ... })));
    }
```

This is the **double-checked locking** pattern, and if you know it from
Java's singleton discussions, you know why it's here: check the cache
(cheap), take the lock (only one thread proceeds), *check the cache
again* (because the world may have changed while you waited), and only
then do the expensive work. The second request blocks on `acquire()`,
gets the permit, re-queries, sees the first request's row, and returns
the same links — no duplicate build.

The semaphore itself lives in `src/cache/mod.rs`, and its design
deserves a closer look:

```rust
/// Semaphore map to prevent duplicate concurrent exports per (url_id, etype)
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

/// Upper bound on semaphore-map entries. Prevents the map from growing
/// unboundedly (each key is a unique url_id+etype from an export; without a
/// cap this leaks one entry per fic exported). The map is a concurrency
/// guard, not a cache — evicting entries is always safe because an in-flight
/// export holds its own `Arc<Semaphore>`.
const MAX_SEMAPHORE_KEYS: usize = 10_000;

/// Get or create a semaphore for concurrent export prevention
pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    // Bounded growth: when over the cap, drop the map (in-flight exports keep
    // their Arc<Semaphore>, so this cannot break an active export).
    if map.len() >= MAX_SEMAPHORE_KEYS {
        map.clear();
    }
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

Every idea here is worth stealing:

- **`Semaphore::new(1)` is a mutex with a ticket** — a single-permit
  semaphore. `acquire().await` parks the task until the permit is free;
  dropping the permit (when `_permit` goes out of scope) releases it.
  The `let _permit = ...` binding is deliberate: the underscore-prefixed
  name says "I only care about the RAII guard, not the value."
- **The map is `Arc<Mutex<HashMap<...>>>`** because the semaphore for a
  given fic must be shared across all requests. If each request created
  its own `Semaphore::new(1)`, the lock would be worthless — two
  requests would hold two different "locks".
- **`Arc::new(Semaphore::new(1))` returned by *clone*** means callers
  share one semaphore but each holds their own `Arc`. That's what makes
  eviction safe (next bullet).
- **Bounded growth.** Every exported fic adds a key. Ten thousand keys
  is the cap; past it, the map is *cleared* — and that's safe because an
  in-flight export holds its own `Arc` and doesn't consult the map
  again. The comment says it outright: "The map is a concurrency guard,
  not a cache."

💡 **Key Concept — a lock map must be evictable.** Any `Mutex<HashMap<K,
Lock>>` pattern has a leak problem: if keys are never removed, the map
grows forever. The trick is to make eviction *provably safe*. FicHub
does it by giving every caller an `Arc` to the semaphore — the map only
needs to answer "who else is waiting for this key?" If the map forgets a
key, the next request for that key just creates a fresh semaphore; any
export that was actually running still holds its own reference. Evicting
a lock can never break a lock-holder. Hold that bar for your own code:
if eviction could deadlock or double-run a critical section, your design
isn't finished.

🧪 **Try It Yourself — read the semaphore tests.** `src/cache/mod.rs`
ends with two `#[tokio::test]`s that pin down exactly these properties.
`semaphore_map_is_bounded` inserts `MAX_SEMAPHORE_KEYS + 100` keys and
asserts the map stays at or under the cap. `semaphore_reuses_existing_key`
asserts that two calls for the same key return the *same* `Arc`
(`Arc::ptr_eq`) and that the map has exactly one entry. Run them:

```bash
cargo test --lib cache:: 2>&1 | tail -20
```

Both should pass — and if you ever "improve" the eviction logic, these
tests are your safety net. That's the real lesson: tricky concurrency
code without tests is a landmine; with tests, it's a documented bet.

### 19.5 Chapters from the body cache or the network

With the lock held and the double-check done, the handler needs the
actual story text. This is where Part 4 pays off:

```rust
    let chapters = if let Some(cached) = crate::body_cache::load_body(&state.config, &meta.url_id) {
        cached
    } else {
        let fetched = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => { ... }
        };
        let version = crate::body_cache::current_version(&state.config, &meta.url_id);
        if let Err(e) = crate::body_cache::save_body(...) {
            tracing::warn!("body_cache save failed for {}: {e}", meta.url_id);
        }
        fetched
    };
```

If the body cache (Chapter 18) already has the chapters on disk, no
network request happens at all — the export is built from local JSON. If
not, `fetch_chapters` scrapes the site, and the freshly-fetched chapters
are *saved to the body cache immediately*, so the next export of this
fic won't need the network either. Every export makes the site more
self-sufficient — "cache of all gathered fanfiction" in action.

Note the `tracing::warn!` on cache-save failure: a failed body-cache
write is *not fatal*. The export continues with the in-memory chapters;
we just lost the optimization for next time. This is a wonderful
pattern to copy: **decide which failures are fatal and which are
logged-and-ignored, and make the choice explicit at each call site.**

⚠️ **Watch Out — failure paths in `match Err(e)` do more than log.**
The elided `Err` branch calls `state.heal.record_failure(...)` and, for
parse errors, captures an HTML snapshot for the self-healing system
(Part 10). The pattern to notice: `record_failure` is called with the
original URL and the *source* URL separately, and the handler returns
`AppError::ScrapeError`. A scrape failure isn't just an error — it's an
*event* the heal subsystem feeds on. When you build error handling,
think about whether your error is also data.

### 19.6 Seven formats, one pipeline

Now the assembly line. Each format follows the identical recipe:
**generate into a UUID-named temp dir → move the file into the cache →
record the export in `export_log`.** First, EPUB:

```rust
    // Generate EPUB
    let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Move to cache
    let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    // Record in export_log
    queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

Then HTML, TXT, and MD repeat the pattern with their own builders (each
returns `(path, md5)`), and the Calibre formats get a special twist —
they're *optional*:

```rust
    // Generate MOBI via Calibre
    let calibre_container = &state.config.calibre_container;
    let mobi_result = export::convert::convert_epub(
        &cache_dest, "mobi", calibre_container, &state.config.tmp_dir,
    ).await;
    let mobi_hash = if let Ok((mobi_path, mobi_md5)) = mobi_result {
        ...
        Some(mobi_md5)
    } else {
        None
    };
```

The `Option<String>` is a beautiful piece of honesty: EPUB, HTML, TXT,
and MD are guaranteed (pure Rust, no external dependencies), but MOBI,
PDF, and AZW3 require Calibre's `ebook-convert`, which might not be
installed, might time out, or might fail on a weird fic. So the handler
doesn't `?` on those — it converts, and on success adds the hash and
URL; on failure, the fic simply has no mobi link. The response JSON
reflects reality (`mobi_url: null`), and the download links that *do*
exist are real.

💡 **Key Concept — hard and soft dependencies.** A hard dependency
breaks the whole request when it fails; a soft dependency degrades
gracefully. FicHub's export pipeline is a masterclass in drawing that
line: the pure-Rust builders are hard (if EPUB generation fails, the
request fails — there's nothing to serve), while Calibre is soft (if
conversion fails, the user still gets their EPUB and HTML). When you
design a pipeline with optional stages, return `Option` or `Result` at
the boundary and merge successes into the response — never let a
non-essential stage take down the whole product.

### 19.7 The response: hashes, urls, and the slug

The endpoint finishes by assembling the payload:

```rust
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json(&meta, Some(work_id)),
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "txt_url": urls.get("txt"),
        "md_url": urls.get("md"),
        "azw3_url": urls.get("azw3"),
        "notes": notes,
    })))
```

A few details worth stopping on:

- **`hashes` and `urls` are maps, and the `*_url` keys are conveniences**
  — the frontend can use either. Notice the shape stays identical
  whether the request was a cache hit, a full export, or a hash lookup.
  A stable response contract is what lets the SPA (Part 6) treat all
  three paths the same.
- **`info` is a human-readable summary** — "Title by Author, N words in
  M chapters, Status: complete, Updated: ..." — built by
  `build_info_string`. The API serves both machines (the JSON fields)
  and humans (the `info` string) from one response.
- **`slug`** comes from `generate_slug`, a function with its own unit
  tests at the bottom of the file:

```rust
/// Generate a URL-safe slug from title and url_id
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    // Collapse multiple underscores
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

Every non-alphanumeric character becomes `_`, runs of underscores
collapse, leading/trailing underscores are trimmed, and the `url_id` is
appended as a guaranteed-unique suffix. "The Best Story" + `1_21845264`
→ `The_Best_Story-1_21845264`. Never trust a title to be unique; always
append the id. (If you want a fun exercise: the test module at the end
of `export.rs` has `test_generate_slug_basic` — read it and think about
what edge cases it *doesn't* cover, like a title that's entirely
punctuation.)

🧪 **Try It Yourself — a full export, end to end.** With FicHub
running, export a real fic and inspect the shape:

```bash
curl -s "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/21845264" | jq '{err, url_id, slug, epub_url, html_url, mobi_url, hashes}'
```

Run it twice. The second call should return in a fraction of the time —
that's the Chapter 19 fast path answering from `export_log` without
re-scraping. Then check the cache on disk and find your file:

```bash
ls -la cache/epub/1/218/45264/1_21845264/*.epub 2>/dev/null || find cache -name "*.epub" | head
```

The sharded directory layout (`<etype>/<3-char chunks>/<url_id>/<hash>.epub`)
is exactly what `cache::disk::cache_path` produces — we'll dissect it in
Chapter 22.

⚠️ **Watch Out — the response is JSON, not a file.** The export
endpoint *never* streams the EPUB. It returns URLs (`/cache/epub/...?h=...`),
and the *frontend* decides whether to link, redirect, or download. This
separation — "generate and describe" vs. "serve" — is a deliberate
architectural choice. The export endpoint stays cheap to call, the
download endpoint (Chapter 22) owns file I/O, and both can be scaled
independently. When you design an API, ask yourself: *should this
endpoint produce the resource, or produce a way to get the resource?*

That's the pipeline. From URL to seven format links in one handler —
with a semaphore, a double-check, and a versioned, content-addressed
cache keeping it honest. In Chapter 20, we zoom into the first stage of
the assembly line: the pure-Rust EPUB builder.

```rust
/// Query parameters for export requests
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}
```

```rust
/// Main export handler: GET /api/v0/epub?q=<url>
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
    headers: axum::http::HeaderMap,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();
```

### 19.1 The front door: validation, rate limits, and gates

The first ~90 lines of the handler are all about *who is asking*, not
*what they asked for*. There's a proof-of-work gate for shadowbanned
clients (we'll meet the full anti-bot stack in Part 11), a tiered rate
limit keyed by real client IP *and* client ID, and a few lines of
interesting philosophy in the comments:

```rust
    // ── Tiered rate limit (download tier) ─────────────────────────────
    // Keyed by real client IP AND (ip, client_id) so shared-NAT readers are
    // not punished and identified bots are throttled per-client. A
    // shadowbanned client_id gets a much stricter bucket (friction, not a
    // hard block). Errors are treated as allow (fail-open) so a Redis hiccup
    // never blocks real downloads.
```

That comment is a mini design document: shared-NAT readers (a whole
university behind one IP) shouldn't share one bucket; identified bots
get per-client throttling; and if Redis hiccups, the *fail-open* choice
means real readers keep downloading rather than getting locked out. We
won't dwell on the limiter here — Part 11 owns it — but notice the
pattern: the export endpoint composes defenses instead of reinventing
them.

Then the validation itself:

```rust
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Check for automated flag (block if present)
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // If q looks like a hash (not a URL), look up from fic_info directly
    if !query.starts_with("http") {
        return handle_hash_lookup(&state, query, start.elapsed().as_millis() as i32).await;
    }
```

Three rules, three lines of insight:

- **Empty query** → a clean JSON error. Note the error contract: `err`
  is a negative number, `msg` is human-readable, and the original `q` is
  echoed back. This `{err, msg, ...}` envelope is FicHub's API-wide
  convention — the frontend can check `err !== 0` and show `msg`.
- **`automated=true`** → refused outright with `-10`. FicHub serves
  humans; scripts that self-identify get a friendly no.
- **Not a URL?** → treat `q` as a fic hash and take the fast path.
  `handle_hash_lookup` skips scraping entirely: it reads `fic_info`
  straight from Postgres and returns whatever exports are already cached
  (or an empty `urls` map). This is how a link like
  `/api/v0/epub?q=1_21845264` works without ever touching the network —
  you'll see this pattern again in Part 6's meta endpoints.

🧪 **Try It Yourself — hit the endpoint without a query.** If you have
FicHub running (Part 1 showed you how), fire up a terminal and run:

```bash
curl -s "http://localhost:3000/api/v0/epub" | jq
# {"err": -1, "msg": "no query", "q": ""}
```

Then try the automated flag with any URL:

```bash
curl -s "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/21845264&automated=true" | jq
# {"err": -10, "msg": "automated requests blocked"}
```

You've just exercised the two cheapest branches of the pipeline — no
scraping, no database, no disk. The endpoint answered in milliseconds
because validation happens *before* any expensive work.

⚠️ **Watch Out — `unwrap_or("")` hides a missing parameter.** If a
client sends no `q` at all, `params.q` is `None` and the handler treats
it as an empty query. That's the right call for an API, but notice what
it means for your own code: you can't distinguish "no query" from
"query is the empty string". When the distinction matters (e.g. logging
malformed requests), prefer matching on `Option` explicitly.

### 19.2 Metadata, blacklists, and the version number

With the gates passed, the handler finds a scraper, fetches metadata,
upserts it, auto-merges a *work* record, and extracts tags — a block
that reads like a checklist of everything Part 4 taught us. I'll skip
the middle (you know it chapter and verse by now) and land on the part
that's new: **the version number and the cache check**.

```rust
    // Compute cache version
    let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
    let version = state.config.export_version + version_bump;

    // Check cache (try EPUB first, then all formats)
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
```

Here's the idea that makes the whole cache sound: **an export is only
valid for a specific version of its input.** Three numbers contribute:

- `state.config.export_version` — the version of FicHub's *export
  code*. Bump it (via `EXPORT_VERSION` in `.env`, which we saw in Part
  3) when the EPUB template or CSS changes, and every cached export
  becomes stale at once.
- `version_bump` — how many times this fic has been re-scraped/updated.
  When a fic changes upstream, its version changes, and old exports
  stop matching.
- `input_hash` — the content hash from the scraper (Part 4). If the
  upstream story changed, the hash changes, so even the *same version
  number* with a *different hash* misses the cache.

The `export_log` table (Part 3's migrations) records every export:
`url_id`, `version`, `etype`, `input_hash`, and the `export_hash` of the
generated file. A cache hit means: "I have already built an EPUB for
this fic, at this export version, from this exact content hash — here's
the file's hash."

💡 **Key Concept — cache invalidation by content addressing.** Instead
of storing files under a timestamp and deleting them when they get old,
FicHub names cache files *by their content* (the MD5 hash) and records
*what input they were built from*. A file never needs to be "refreshed"
— it needs to be *matched*. If the input changes, the lookup misses and
a new file is born with a new hash. The old file becomes garbage that a
sweeper can remove safely, because nothing references it anymore. This
is the same idea as Git's content-addressed object store, scaled down
to fanfiction files.

### 19.3 Cache hit: the fast path

If `find_export_log` returns a row, the handler does the *minimum work
possible*: build URLs from the stored hashes and respond.

```rust
    if let Some(export_log) = cached {
        // Cache hit - build URLs from cached hash
        let epub_hash = &export_log.export_hash;
        hashes.insert("epub".to_string(), epub_hash.clone());
        urls.insert("epub".to_string(), format!("/cache/epub/{}?h={}", meta.url_id, epub_hash));

        // Other formats from cached EPUB
        for etype_str in &["html", "mobi", "pdf"] {
            if let Ok(_etype) = etype_str.parse::<EType>() {
                let e_input_hash = format!("epub:{}", epub_hash);
                if let Ok(Some(entry)) = queries::find_export_log(
                    &state.db, &meta.url_id, version, etype_str, &e_input_hash,
                ).await {
                    hashes.insert(etype_str.to_string(), entry.export_hash.clone());
                    urls.insert(etype_str.to_string(), format!("/cache/{}/{}?h={}", etype_str, meta.url_id, entry.export_hash));
                }
            }
        }
```

Read that loop carefully, because it encodes a subtle dependency: the
MOBI and PDF exports are *derived from the EPUB*, so their input hash is
`epub:<epub_hash>` — literally "this mobi was converted from an EPUB
with this hash." The `export_log` table isn't a flat list of files; it's
a tiny dependency graph, and `input_hash` is the edge.

The URL format `/cache/epub/{url_id}?h={hash}` is the contract the
download handler (Chapter 22) will enforce: the hash is a *query
parameter*, not part of the path. That lets the frontend construct URLs
without knowing the layout of the cache on disk — and lets the download
handler verify the file against its claimed hash before serving a byte.

⚠️ **Watch Out — only HTML, MOBI, and PDF ride the cached-EPUB path
here.** TXT and MD are generated directly from chapters (as we'll see in
Chapter 21), so the fast path only looks for the three Calibre-family
formats plus EPUB. If you're tracing why a cached fic returns no `txt_url`,
it's not a bug — TXT/MD are cheap enough to regenerate, so they're only
built during a full export. The response shape still includes `txt_url`
and `md_url` keys (set to `null`), keeping the frontend contract stable.

Here's where Chapter 19 earns its outline bullet. Cache miss — two
requests for the same fic arrive at the same instant; both miss; both
decide to build. Without a guard, you'd scrape and build the same EPUB
twice, burn twice the bandwidth, and the second writer would clobber
the first's file. The classic fix is a **lock per resource**, and
FicHub's is a per-(url_id, etype) async semaphore:

```rust
    // Cache miss - generate EPUB
    // Acquire semaphore to prevent duplicate concurrent exports
    let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
    let _permit = sem.acquire().await.map_err(|e| AppError::Internal(e.to_string()))?;

    // Check cache again (double-check pattern)
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
    if let Some(export_log) = cached {
        // Another concurrent request already generated it
        ...
        return Ok(Json(json!({ ... })));
    }
```

This is the **double-checked locking** pattern, and if you know it from
Java's singleton discussions, you know why it's here: check the cache
(cheap), take the lock (only one thread proceeds), *check the cache
again* (because the world may have changed while you waited), and only
then do the expensive work. The second request blocks on `acquire()`,
gets the permit, re-queries, sees the first request's row, and returns
the same links — no duplicate build.

The semaphore itself lives in `src/cache/mod.rs`, and its design
deserves a closer look:

```rust
/// Semaphore map to prevent duplicate concurrent exports per (url_id, etype)
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

/// Upper bound on semaphore-map entries. Prevents the map from growing
/// unboundedly (each key is a unique url_id+etype from an export; without a
/// cap this leaks one entry per fic exported). The map is a concurrency
/// guard, not a cache — evicting entries is always safe because an in-flight
/// export holds its own `Arc<Semaphore>`.
const MAX_SEMAPHORE_KEYS: usize = 10_000;

/// Get or create a semaphore for concurrent export prevention
pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    // Bounded growth: when over the cap, drop the map (in-flight exports keep
    // their Arc<Semaphore>, so this cannot break an active export).
    if map.len() >= MAX_SEMAPHORE_KEYS {
        map.clear();
    }
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

Every idea here is worth stealing:

- **`Semaphore::new(1)` is a mutex with a ticket** — a single-permit
  semaphore. `acquire().await` parks the task until the permit is free;
  dropping the permit (when `_permit` goes out of scope) releases it.
  The `let _permit = ...` binding is deliberate: the underscore-prefixed
  name says "I only care about the RAII guard, not the value."
- **The map is `Arc<Mutex<HashMap<...>>>`** because the semaphore for a
  given fic must be shared across all requests. If each request created
  its own `Semaphore::new(1)`, the lock would be worthless — two
  requests would hold two different "locks".
- **`Arc::new(Semaphore::new(1))` returned by *clone*** means callers
  share one semaphore but each holds their own `Arc`. That's what makes
  eviction safe (next bullet).
- **Bounded growth.** Every exported fic adds a key. Ten thousand keys
  is the cap; past it, the map is *cleared* — and that's safe because an
  in-flight export holds its own `Arc` and doesn't consult the map
  again. The comment says it outright: "The map is a concurrency guard,
  not a cache."

💡 **Key Concept — a lock map must be evictable.** Any `Mutex<HashMap<K,
Lock>>` pattern has a leak problem: if keys are never removed, the map
grows forever. The trick is to make eviction *provably safe*. FicHub
does it by giving every caller an `Arc` to the semaphore — the map only
needs to answer "who else is waiting for this key?" If the map forgets a
key, the next request for that key just creates a fresh semaphore; any
export that was actually running still holds its own reference. Evicting
a lock can never break a lock-holder. Hold that bar for your own code:
if eviction could deadlock or double-run a critical section, your design
isn't finished.

🧪 **Try It Yourself — read the semaphore tests.** `src/cache/mod.rs`
ends with two `#[tokio::test]`s that pin down exactly these properties.
`semaphore_map_is_bounded` inserts `MAX_SEMAPHORE_KEYS + 100` keys and
asserts the map stays at or under the cap. `semaphore_reuses_existing_key`
asserts that two calls for the same key return the *same* `Arc`
(`Arc::ptr_eq`) and that the map has exactly one entry. Run them:

```bash
cargo test --lib cache:: 2>&1 | tail -20
```

Both should pass — and if you ever "improve" the eviction logic, these
tests are your safety net. That's the real lesson: tricky concurrency
code without tests is a landmine; with tests, it's a documented bet.

### 19.5 Chapters from the body cache or the network

With the lock held and the double-check done, the handler needs the
actual story text. This is where Part 4 pays off:

```rust
    let chapters = if let Some(cached) = crate::body_cache::load_body(&state.config, &meta.url_id) {
        cached
    } else {
        let fetched = match scraper.fetch_chapters(&state.http_client, &meta).await {
            Ok(ch) => ch,
            Err(e) => { ... }
        };
        let version = crate::body_cache::current_version(&state.config, &meta.url_id);
        if let Err(e) = crate::body_cache::save_body(...) {
            tracing::warn!("body_cache save failed for {}: {e}", meta.url_id);
        }
        fetched
    };
```

If the body cache (Chapter 18) already has the chapters on disk, no
network request happens at all — the export is built from local JSON. If
not, `fetch_chapters` scrapes the site, and the freshly-fetched chapters
are *saved to the body cache immediately*, so the next export of this
fic won't need the network either. Every export makes the site more
self-sufficient — "cache of all gathered fanfiction" in action.

Note the `tracing::warn!` on cache-save failure: a failed body-cache
write is *not fatal*. The export continues with the in-memory chapters;
we just lost the optimization for next time. This is a wonderful
pattern to copy: **decide which failures are fatal and which are
logged-and-ignored, and make the choice explicit at each call site.**

⚠️ **Watch Out — failure paths in `match Err(e)` do more than log.**
The elided `Err` branch calls `state.heal.record_failure(...)` and, for
parse errors, captures an HTML snapshot for the self-healing system
(Part 10). The pattern to notice: `record_failure` is called with the
original URL and the *source* URL separately, and the handler returns
`AppError::ScrapeError`. A scrape failure isn't just an error — it's an
*event* the heal subsystem feeds on. When you build error handling,
think about whether your error is also data.

### 19.6 Seven formats, one pipeline

Now the assembly line. Each format follows the identical recipe:
**generate into a UUID-named temp dir → move the file into the cache →
record the export in `export_log`.** First, EPUB:

```rust
    // Generate EPUB
    let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir)
        .await
        .map_err(|e| AppError::ExportError(e.to_string()))?;

    // Move to cache
    let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    // Record in export_log
    queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

Then HTML, TXT, and MD repeat the pattern with their own builders (each
returns `(path, md5)`), and the Calibre formats get a special twist —
they're *optional*:

```rust
    // Generate MOBI via Calibre
    let calibre_container = &state.config.calibre_container;
    let mobi_result = export::convert::convert_epub(
        &cache_dest, "mobi", calibre_container, &state.config.tmp_dir,
    ).await;
    let mobi_hash = if let Ok((mobi_path, mobi_md5)) = mobi_result {
        ...
        Some(mobi_md5)
    } else {
        None
    };
```

The `Option<String>` is a beautiful piece of honesty: EPUB, HTML, TXT,
and MD are guaranteed (pure Rust, no external dependencies), but MOBI,
PDF, and AZW3 require Calibre's `ebook-convert`, which might not be
installed, might time out, or might fail on a weird fic. So the handler
doesn't `?` on those — it converts, and on success adds the hash and
URL; on failure, the fic simply has no mobi link. The response JSON
reflects reality (`mobi_url: null`), and the download links that *do*
exist are real.

💡 **Key Concept — hard and soft dependencies.** A hard dependency
breaks the whole request when it fails; a soft dependency degrades
gracefully. FicHub's export pipeline is a masterclass in drawing that
line: the pure-Rust builders are hard (if EPUB generation fails, the
request fails — there's nothing to serve), while Calibre is soft (if
conversion fails, the user still gets their EPUB and HTML). When you
design a pipeline with optional stages, return `Option` or `Result` at
the boundary and merge successes into the response — never let a
non-essential stage take down the whole product.

### 19.7 The response: hashes, urls, and the slug

The endpoint finishes by assembling the payload:

```rust
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, notes) = build_info_string(&meta);

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "work_id": work_id,
        "slug": slug,
        "meta": build_meta_json(&meta, Some(work_id)),
        "hashes": hashes,
        "urls": urls,
        "epub_url": urls.get("epub"),
        "html_url": urls.get("html"),
        "mobi_url": urls.get("mobi"),
        "pdf_url": urls.get("pdf"),
        "txt_url": urls.get("txt"),
        "md_url": urls.get("md"),
        "azw3_url": urls.get("azw3"),
        "notes": notes,
    })))
```

A few details worth stopping on:

- **`hashes` and `urls` are maps, and the `*_url` keys are conveniences**
  — the frontend can use either. Notice the shape stays identical
  whether the request was a cache hit, a full export, or a hash lookup.
  A stable response contract is what lets the SPA (Part 6) treat all
  three paths the same.
- **`info` is a human-readable summary** — "Title by Author, N words in
  M chapters, Status: complete, Updated: ..." — built by
  `build_info_string`. The API serves both machines (the JSON fields)
  and humans (the `info` string) from one response.
- **`slug`** comes from `generate_slug`, a function with its own unit
  tests at the bottom of the file:

```rust
/// Generate a URL-safe slug from title and url_id
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    // Collapse multiple underscores
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

Every non-alphanumeric character becomes `_`, runs of underscores
collapse, leading/trailing underscores are trimmed, and the `url_id` is
appended as a guaranteed-unique suffix. "The Best Story" + `1_21845264`
→ `The_Best_Story-1_21845264`. Never trust a title to be unique; always
append the id. (If you want a fun exercise: the test module at the end
of `export.rs` has `test_generate_slug_basic` — read it and think about
what edge cases it *doesn't* cover, like a title that's entirely
punctuation.)

🧪 **Try It Yourself — a full export, end to end.** With FicHub
running, export a real fic and inspect the shape:

```bash
curl -s "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/21845264" | jq '{err, url_id, slug, epub_url, html_url, mobi_url, hashes}'
```

Run it twice. The second call should return in a fraction of the time —
that's the Chapter 19 fast path answering from `export_log` without
re-scraping. Then check the cache on disk and find your file:

```bash
ls -la cache/epub/1/218/45264/1_21845264/*.epub 2>/dev/null || find cache -name "*.epub" | head
```

The sharded directory layout (`<etype>/<3-char chunks>/<url_id>/<hash>.epub`)
is exactly what `cache::disk::cache_path` produces — we'll dissect it in
Chapter 22.

⚠️ **Watch Out — the response is JSON, not a file.** The export
endpoint *never* streams the EPUB. It returns URLs (`/cache/epub/...?h=...`),
and the *frontend* decides whether to link, redirect, or download. This
separation — "generate and describe" vs. "serve" — is a deliberate
architectural choice. The export endpoint stays cheap to call, the
download endpoint (Chapter 22) owns file I/O, and both can be scaled
independently. When you design an API, ask yourself: *should this
endpoint produce the resource, or produce a way to get the resource?*

That's the pipeline. From URL to seven format links in one handler —
with a semaphore, a double-check, and a versioned, content-addressed
cache keeping it honest. In Chapter 20, we zoom into the first stage of
the assembly line: the pure-Rust EPUB builder.

---

## Chapter 20 — The EPUB Builder: export/epub.rs

An EPUB file, in case you've never peeked inside one, is a ZIP archive
with a very specific skeleton: a `mimetype` file (stored, not
compressed — readers check it first), a `META-INF/container.xml`
pointing at the spine, a table of contents, and a set of XHTML content
documents. Writing all of that by hand is fiddly and error-prone — which
is exactly why FicHub uses the `epub-builder` crate (version 0.8, in
`Cargo.toml`). It's the *pure Rust* workhorse of the export system:
no Python, no Calibre, no external process. Just `EpubBuilder`,
`EpubContent`, and a `ZipLibrary` — and the result is a standards
compliant `.epub` that Kobo, Kindle (via conversion), and every
phone reader can open.

The whole builder is 145 lines, and its job description is written in
the doc comment:

```rust
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};

/// Generate an EPUB file for the given fic metadata and chapters.
///
/// Creates a UUID-named subdirectory inside `tmp_dir`, writes the EPUB there,
/// computes its MD5 hash, and returns `(path_to_epub, md5_hex)`.
pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // ---- work directory ---------------------------------------------------
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // ---- builder setup ----------------------------------------------------
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;
```

The first thing you notice is the **UUID work directory**: every export
gets a fresh random subdirectory under `tmp_dir` (`./tmp` by default,
per Part 3's config). Why not a fixed directory? Because concurrent
exports (remember the semaphore is per-fic, not global) would stomp on
each other's `output.epub`. A UUID per build makes collisions
structurally impossible — the classic "temp file" problem solved by
namespacing rather than locking.

Then the builder is set up: `ZipLibrary::new()` selects the ZIP backend,
and `metadata()` calls fill in the Dublin Core fields (title, author,
language, description) that every EPUB reader shows in its library
screen. Notice the `?`s everywhere: `epub-builder` returns
`Result<_, epub_builder::Error>`, and FicHub's `ExportError` has a
`From<epub_builder::Error>` impl (in `src/export/mod.rs`), so every
builder call can use `?` and surface as a typed export error.

### 20.1 Metadata and the book's CSS

The book's look — the part readers actually *feel* — is a stylesheet:

```rust
    // Book-style paragraphs: first line indented, no extra margin between
    // paragraphs (issue #21). First paragraph after a heading keeps margin-top.
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0 0 0.8em 0;text-indent:1.5em;}",
        "h1+p, h2+p, h3+p, h4+p, p:first-of-type{text-indent:0;}"
    );
    // epub-builder writes this as "stylesheet.css" automatically
    builder.stylesheet(css.as_bytes())?;
```

This tiny stylesheet is a lesson in typography *and* in how small
details accumulate into quality:

- `p{text-indent:1.5em;}` — first-line indents, the way printed novels
  look. The comment names the issue number ("issue #21") — someone once
  opened a bug saying "paragraphs look like a webpage, not a book", and
  this CSS is the fix. Real projects record *why* in the code.
- `h1+p, h2+p, p:first-of-type{text-indent:0;}` — the paragraph *right
  after a heading* doesn't get indented, because a first paragraph that
  starts indented looks broken. That's a typographic rule most readers
  never notice, but they'd notice its absence.
- `h2{text-align:center;}` — chapter titles centered, the convention in
  fiction.
- The whole thing is a `concat!` of string literals, so the compiler
  concatenates at compile time — zero runtime cost.

💡 **Key Concept — you are designing the reading experience, not just
writing files.** An EPUB is a container; the *typography* is the
product. Every device renders it slightly differently, so the CSS you
ship is a floor, not a ceiling. The epub-builder crate does the heavy
lifting of container.xml and the OPF manifest — but the difference
between a "file that opens" and a "book that feels like a book" is the
ten lines of CSS above. When you build exporters, spend real time on
the output's polish; it's the only part of your pipeline the end user
ever touches.

### 20.2 The introduction page

Every FicHub EPUB opens with a title page: the fic's metadata in a tidy
table, then the description. It's built as one big `format!` template:

```rust
    let intro_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        ...
    );

    builder.add_content(
        EpubContent::new("introduction.xhtml", intro_html.as_bytes())
            .title("Introduction"),
    )?;
```

Three details are easy to miss and worth their weight:

- **`r#"..."#` raw strings.** The template contains `#` inside URLs and
  the `"` characters that XHTML attributes require. A raw string tells
  Rust "don't interpret escapes here" — the alternative would be a
  backslash soup. The `#`-delimiter (`r#"` ... `"#`) exists precisely
  for strings that contain quotes.
- **`escape_html` on every interpolated value.** Titles and authors are
  user-generated text scraped from the internet. A fic titled `It's <b>fine</b>`
  would break the XHTML (or worse, inject markup) if inserted raw.
  `escape_html` turns `<`, `>`, `&`, `"`, `'` into entities before
  interpolation. This is the export-side twin of SQL injection
  protection: **never interpolate untrusted text into a structured
  format without escaping it.**
- **The chapter *content* is NOT escaped** — look closely. `{content}`
  in the chapter template is inserted raw, because scraped chapter
  bodies are *already HTML* (the scraper preserves the source site's
  markup — Part 4). Escape the metadata, trust the body. Knowing which
  inputs are trusted is a design decision you make once and document.

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

⚠️ **Watch Out — `escape_html` must escape `&` FIRST.** If you escaped
`<` before `&`, the sequence `&lt;` produced by the first replacement
would be re-escaped into `&amp;lt;` by the second. The order of
`replace` calls is the correctness of this function. It's a five-line
function with a hidden invariant — exactly the kind of thing a unit test
should pin down, and exactly the kind of thing nobody writes a unit test
for. When you write your own escaping/encoding helpers, remember: order
matters, and a comment costs nothing.

### 20.3 The chapters

The heart of the book — one XHTML document per chapter, named
`chapter_{chapter_id}.xhtml` so ordering is deterministic:

```rust
    for chapter in chapters {
        let chapter_html = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
            title = escape_html(&chapter.title),
            content = chapter.content,
        );

        let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
        builder.add_content(
            EpubContent::new(filename.as_str(), chapter_html.as_bytes())
                .title(&chapter.title),
        )?;
    }
```

`EpubContent::new(name, bytes).title(...)` tells epub-builder both the
internal filename and the table-of-contents label — so the reader's TOC
shows real chapter titles, and `chapter_id` (the source site's own
chapter number) keeps documents in the right order. The `builder` also
automatically orders content by insertion, so the loop order *is* the
book order.

💡 **Key Concept — the builder pattern makes correct-by-construction
output.** Notice what the code *doesn't* do: it never writes
`container.xml`, never builds the OPF manifest, never maintains the
spine. `EpubBuilder` accumulates state (metadata, stylesheet, content)
and `generate()` produces a valid archive or an error. This is the
builder pattern's whole pitch: complex, multi-part output becomes a
sequence of small, individually-testable calls. If you ever find
yourself hand-assembling a spec-compliant file format, ask whether a
crate like this can own the compliance for you — that's 145 lines of
code you don't have to maintain.

### 20.4 Write it, hash it

Finally, the file is written and hashed — and the hash is *not* an
afterthought:

```rust
    // ---- write EPUB file --------------------------------------------------
    let epub_path = work_dir.join("output.epub");
    let file = fs::File::create(&epub_path)?;
    builder.generate(file)?;

    // ---- MD5 hash ---------------------------------------------------------
    let epub_data = fs::read(&epub_path)?;
    let md5_hex = Md5::digest(&epub_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    Ok((epub_path, md5_hex))
}
```

`generate(file)` writes the ZIP. Then the whole file is read back and
MD5'd — because this hash is about to become the *filename* (Chapter 22:
`<hash>.epub` in the cache), the cache key in `export_log`, and the
integrity proof the download handler will verify. The MD5 here isn't for
security; it's a *content fingerprint* — deterministic, verifiable, and
useful. Two exports of the same fic at the same version produce the
same bytes and therefore the same hash, which is exactly why the cache
can deduplicate them.

Note the timestamp helper used on the intro page — a small function
with its own correctness details:

```rust
fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;

    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;

    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

FicHub timestamps are Unix milliseconds (from scraping). `chrono` wants
seconds and nanoseconds, so the code splits the millis — `secs` is
integer division, `nsecs` is the remainder scaled to nanoseconds. And
`unwrap_or_default()` means a garbage timestamp renders as an empty
string rather than crashing the export. Small function, three decisions:
split the unit, format as date-only (`%Y-%m-%d`), and fail soft.

🧪 **Try It Yourself — peek inside an EPUB.** EPUBs are ZIP files. Take
any `.epub` out of your cache (or make one) and unzip it:

```bash
cd /tmp && mkdir -p epub-peek && cd epub-peek
unzip -o "$(find /personal/documents/code/rust/fichub/cache -name '*.epub' | head -1)"
ls -la
```

You'll see `mimetype`, `META-INF/container.xml`, `stylesheet.css`,
`introduction.xhtml`, and `chapter_*.xhtml` — the exact structure the
builder assembled. Open `stylesheet.css` and confirm it's the same
`concat!` CSS from the source. That's your pipeline's output, on disk,
in its native habitat. (If you don't have a cached EPUB yet, run the
export curl from Chapter 19 first — same thing, one command.)

⚠️ **Watch Out — async function, sync work.** `create_epub` is `async`
but does no `.await` inside — it's synchronous work wearing an async
hat. Why? So the caller (`epub_handler`) can chain it in its async
pipeline with one consistent error path. In real FicHub this is fine
(the work is fast), but the pattern is worth flagging: an `async fn`
with no awaits still runs on the async executor and blocks a worker
thread. If you copy this shape for genuinely slow work (huge fics,
compression-heavy formats), consider `tokio::task::spawn_blocking` so
the blocking I/O doesn't starve the executor. Recognizing when "async"
is an interface choice vs. a concurrency choice is a junior-to-senior
skill.

That's the EPUB builder: metadata, a stylesheet, an intro page, N
chapters, and a content-addressed filename — 145 lines, zero external
processes. But EPUB isn't the only format readers want. In Chapter 21,
we build the HTML bundle, the plain-text export, the Markdown export,
and the Calibre sidecar that converts the EPUB we just built into MOBI,
PDF, and AZW3.

---

## Chapter 21 — HTML, TXT, MD, and the Calibre Sidecar

Not every reader wants an EPUB. Some want a single HTML file they can
read in any browser; some want plain text for a terminal or an
old-school e-reader; some want Markdown for notes; and some want the
Amazon formats (MOBI, AZW3) or a PDF for printing. FicHub's answer is a
family of generators in `src/export/` — each one a small, focused
module with the same signature as `create_epub`:

- `html_bundle.rs` — a single self-contained HTML file, zipped.
- `txt.rs` — HTML stripped down to clean plain text.
- `md.rs` — HTML converted to Markdown.
- `convert.rs` — the Calibre sidecar: EPUB → MOBI/PDF/AZW3.
- `fallback.rs` — a fichub.net hosted fallback when everything else
  fails.

Each returns `(path, md5_hex)` — the same contract the pipeline in
Chapter 19 consumes. One signature, five implementations: that's the
whole design. In this chapter we build all four "content" formats and
then meet the external process that makes the Amazon/PDF formats work.

### 21.1 The HTML bundle: one file to rule them all

The HTML export is a *single self-contained page* — title, metadata
table, description, chapter navigation, and every chapter's text in one
scrollable document — wrapped in a ZIP so the download is one artifact.
(It's zipped, not served raw, because a 2 MB `index.html` would be a
pain to download and browsers handle `.zip` attachment downloads more
reliably.) Building it is two passes over the chapters: first collect
the navigation list and the content blocks, then interpolate both into
the page template:

```rust
    let mut chapters_nav = String::new();
    let mut chapters_content = String::new();

    for chapter in chapters {
        chapters_nav.push_str(&format!(
            r##"<li><a href="#ch{ch}">{title}</a></li>"##,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
        ));

        chapters_content.push_str(&format!(
            r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
            content = chapter.content,
        ));
    }
```

The navigation anchors (`#ch{ch}`) are the *same* ids as the content
headings (`<h2 id="ch{ch}">`), so the nav links jump to chapters. And
look at the raw-string syntax: `r##"..."##` — double-hash delimiters.
Why? Because the template itself contains `"#` (the href starts with
`#`), which would terminate a single-hash raw string. Rust's `r#`/
`r##`/`r###` ladder exists exactly for this. The code comment in the
repo even says so.

Then the whole page is assembled — a full HTML document with embedded
CSS (Georgia serif, max-width 800px, a `.nav` box with a two-column
chapter list, `.content p` with the same first-line indent rule we saw
in the EPUB). The page is written to `index.html`, then zipped:

```rust
    // ---- write index.html ---------------------------------------------------
    let html_path = work_dir.join("index.html");
    fs::write(&html_path, &html)?;

    // ---- bundle into ZIP ----------------------------------------------------
    let zip_path = work_dir.join("bundle.zip");
    let zip_file =
        fs::File::create(&zip_path).map_err(|e| ExportError::IoError(e.to_string()))?;
    let mut zip = ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    zip.start_file("index.html", options)
        .map_err(|e| ExportError::ZipError(e.to_string()))?;
    zip.write_all(html.as_bytes())
        .map_err(|e| ExportError::ZipError(e.to_string()))?;

    zip.finish()
        .map_err(|e| ExportError::ZipError(e.to_string()))?;
```

Three details worth naming:

- **`unix_permissions(0o644)`** — the zip entry records a Unix mode so
  unzipping on Linux produces a world-readable file (no accidental
  `rwx------` surprises). Small, but it's the kind of thing that
  surfaces as a support ticket otherwise.
- **Every `zip` call maps its error to a `ZipError`** — the `zip`
  crate's error type is converted by hand here because `ExportError`
  only implements `From<zip::result::ZipError>` for the crate's
  *re-exported* type; the explicit `.map_err` is the honest adapter.
- **The `zip` dependency is version `8` (the fork), not the older
  `zip` crate** — `Cargo.toml` even has a comment that the integration
  tests unzip user-export archives with the same crate, so parsing and
  producing share one codebase.

💡 **Key Concept — an export format is a product decision.** EPUB is
the flagship, but each reader demographic needs its own artifact: HTML
for "read anywhere, no app", TXT for "maximum compatibility, zero
frills", MD for "I keep notes in Obsidian", MOBI/AZW3 for Kindle,
PDF for "I print things". Notice FicHub didn't build one universal
format — it built a *family* sharing one pipeline. When you design
exports, ask who the artifact is for, and don't let "we have EPUB"
excuse ignoring the other 40% of your users.

⚠️ **Watch Out — zipping one file looks silly until it doesn't.** A
single-file ZIP is a deliberate trade: browsers handle `.zip` download
attachments predictably, the archive keeps the byte stream
self-identifying, and the `application/zip` MIME type (you'll see it in
Chapter 22) is universally understood. If you're ever tempted to serve
raw HTML with `Content-Disposition: attachment`, remember the filename
and extension you'd have to fight with for every browser and every
download manager. Sometimes the "wasteful" wrapper is the robust
choice.

### 21.2 TXT: HTML down to plain text, in four steps

Plain text sounds trivial — "just delete the tags" — but doing it *well*
is a four-stage pipeline with careful ordering. Here is FicHub's
`strip_html`, in full:

```rust
fn strip_html(input: &str) -> String {
    use regex_lite::Regex;

    // Step 1: Replace block-level opening/closing tags with double newlines.
    let block_re = Regex::new(r"(?i)</?(?:p|div|h[1-6])[^>]*>|<br\s*/?>").unwrap();
    let s = block_re.replace_all(input, "\n\n");

    // Step 2: Strip all remaining tags.
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let s = tag_re.replace_all(&s, "");
    let mut result = s.to_string();

    // Step 3: Decode HTML entities (AFTER tag stripping, so `<3` in text is safe).
    result = decode_entities(&result);

    // Step 4: Collapse whitespace — split on newlines, trim each line, join.
    let lines: Vec<String> = result
        .split('\n')
        .map(|line| {
            line.split_whitespace()
                .collect::<Vec<&str>>()
                .join(" ")
        })
        .collect();
    let mut cleaned = lines.join("\n");

    // Collapse 3+ consecutive newlines into exactly 2.
    while cleaned.contains("\n\n\n") {
        cleaned = cleaned.replace("\n\n\n", "\n\n");
    }

    let cleaned = cleaned.trim().to_string();
    cleaned
}
```

Walk the four steps and you'll see each one exists because of a
specific failure the previous step would cause:

1. **Block tags → `\n\n`.** If you just stripped `<p>` you'd get
   `First paragraph.Second paragraph.` — words glued together. Block
   tags *are* paragraph structure, so they become blank lines *before*
   anything else runs. The regex covers `p`, `div`, `h1`–`h6`, and `<br>`
   variants.
2. **Remaining tags → nothing.** Inline tags like `<em>` or `<b>` leave
   no trace — text only. (Their *content* survives; only the markup
   dies.)
3. **Entities → characters, *after* tag-stripping.** The comment
   explains why: `<3` written as `&lt;3` must not be re-interpreted as
   an opening tag, so entity decoding happens after tags are gone. If
   the order were reversed, `&lt;3` would become `<3` and then the tag
   stripper would eat it.
4. **Whitespace collapse.** Scraped HTML is full of indentation and
   newlines from the source markup. Each line is split on whitespace
   and rejoined with single spaces, then 3+ newlines collapse to 2 —
   and the whole thing is trimmed. The output is *clean text*, not
   source markup.

The module's tests pin down every one of these behaviors — `Tom &amp;
Jerry &lt;3 &quot;quoted&quot;` becomes `Tom & Jerry <3 "quoted"`,
`<p></p>` becomes `""`, nested tags flatten to `Hello world`. That test
suite is the real spec of this function.

💡 **Key Concept — order of operations is the algorithm.** `strip_html`
looks like a few regexes, but it's actually a *sequence with
dependencies*: blocks-before-strip, strip-before-decode,
decode-before-collapse. Swap any two steps and the output breaks
(`<3` vanishes; paragraphs fuse; entities survive). This is the single
most underrated idea in text processing: **when you write a
transform pipeline, the ordering *is* the correctness**, and each step's
comment should say what failure it prevents. That's why the repo's
doc comment spells the order out explicitly — treat it as a spec.

🧪 **Try It Yourself — read the TXT test suite.** `src/export/txt.rs`
ends with fourteen tests. Pick three that look trivial
(`test_strip_html_empty`, `test_strip_html_only_whitespace`,
`test_decode_entities_nbsp`) and predict the exact assertion before
reading it. Then run the module's tests:

```bash
cargo test --lib export::txt 2>&1 | tail -15
```

All green? Good — now delete the `while cleaned.contains("\n\n\n")`
loop mentally and predict which test fails. That exercise — *predict,
then verify* — is how you learn to read other people's tests as
documentation.

### 21.3 MD: HTML to Markdown by regex

Markdown export is the same problem with a different target: instead of
*removing* structure, we *translate* it. `md.rs`'s `html_to_md` runs a
cascade of regex replacements, each converting one HTML construct into
its Markdown equivalent:

```rust
fn html_to_md(input: &str) -> String {
    let s = decode_entities(input);

    // Block-level: replace with markdown equivalents
    let s = regex_replace(r"(?i)<h1[^>]*>(.*?)</h1>", &s, "# $1");
    let s = regex_replace(r"(?i)<h2[^>]*>(.*?)</h2>", &s, "## $1");
    let s = regex_replace(r"(?i)<h3[^>]*>(.*?)</h3>", &s, "### $1");
    ...
    let s = regex_replace(r"(?i)<blockquote[^>]*>(.*?)</blockquote>", &s, "> $1");
    let s = regex_replace(r"(?i)<li[^>]*>(.*?)</li>", &s, "- $1");
    let s = regex_replace(r"(?i)<hr\s*/?>", &s, "\n---\n");
    ...
    let s = regex_replace(r"(?i)<b[^>]*>(.*?)</b>", &s, "**$1**");
    let s = regex_replace(r"(?i)<i[^>]*>(.*?)</i>", &s, "*$1*");
    ...
    let s = regex_replace(r#"(?i)<a[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#, &s, "[$2]($1)");
    ...
    s.trim().to_string()
}
```

The `regex_replace` helper (which I've elided here) does the work
`str::replace` can't: it walks `captures_iter`, substituting `$1`, `$2`
etc. from capture groups — that's how `<h1>Title</h1>` becomes
`# Title` and `<a href="https://x">y</a>` becomes `[y](https://x)`.

💡 **Key Concept — regex conversion is a heuristic, not a parser.** This
approach handles the markup fanfiction sites actually produce — and the
module's tests prove it: headers, paragraphs, bold, italic, links,
blockquotes, lists, `<br>`, `<hr>`, images, entities, and a gnarly
"complex" test with nested tags. But a regex can't parse arbitrary HTML
(think malformed nesting, attributes in odd orders). The right tool
depends on the threat model: for converting *your own scrapers'*
well-formed output, regex is fast, dependency-free, and good enough.
For arbitrary untrusted HTML, you'd want a real HTML parser crate. The
`regex_lite` choice here — a dependency-light regex engine — keeps the
export module lean. Knowing when "good enough" is correct is a real
engineering judgment.

⚠️ **Watch Out — the image rule needs both attribute orders.** Look at
the three `<img>` patterns in the source: `src` first then `alt`, `alt`
first then `src`, and `src`-only. Real-world HTML writes attributes in
both orders, and a single regex would silently produce `![](...)` for
half the images. The two-pattern approach is defensive duplication —
and it's exactly the kind of edge case the module's `test_html_to_md_images`
test exists for. When you write regex-based converters, enumerate the
variants the data actually arrives in, and test each one.

### 21.4 The Calibre sidecar: MOBI, PDF, AZW3

EPUB, HTML, TXT, and MD are pure Rust. MOBI, PDF, and AZW3 are *not* —
each needs Calibre's `ebook-convert`, the Swiss-army-knife converter
that ships with every Calibre install. FicHub calls it as an external
process, and `convert.rs` is the polite caller: it isolates the work in
a UUID temp dir, retries once, enforces a timeout, and refuses to lie
about success:

```rust
const CONVERT_TIMEOUT_SECS: u64 = 300;

pub async fn convert_epub(
    epub_path: &Path,
    output_format: &str,
    calibre_container: &str,
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    ...
    // ---- first attempt ----------------------------------------------------
    let first = run_conversion(epub_path, &output_path, calibre_container).await;

    match first {
        Ok(()) => {}
        Err(e) => {
            // ---- retry once -----------------------------------------------
            tracing::warn!(
                "First conversion attempt failed: {e}. Retrying once ..."
            );
            run_conversion(epub_path, &output_path, calibre_container)
                .await
                .map_err(|retry_err| {
                    ExportError::CalibreError(format!(
                        "Calibre conversion failed after retry: {retry_err}"
                    ))
                })?;
        }
    }
    ...
}
```

The signature is the giveaway that this is a *sidecar*, not a peer:
it takes an `epub_path` — an already-built EPUB — and produces another
format. It never sees chapters or metadata; the pipeline hands it the
finished EPUB, and `ebook-convert` does the rest. The retry-once policy
is pragmatic: conversions of huge fics occasionally fail for
transient reasons (memory pressure, container hiccups), and a single
retry fixes most of them without turning one export into a saga.

The *where* of `ebook-convert` is config-driven, and the two branches
of `run_conversion` show the deployment story:

```rust
/// Run a single `ebook-convert` invocation (direct or inside Docker) with a
/// 300-second timeout.
async fn run_conversion(
    epub_path: &Path,
    output_path: &Path,
    calibre_container: &str,
) -> Result<(), ExportError> {
    let timeout_dur = Duration::from_secs(CONVERT_TIMEOUT_SECS);

    if calibre_container.is_empty() {
        run_direct(epub_path, output_path, timeout_dur).await
    } else {
        run_docker(calibre_container, epub_path, output_path, timeout_dur).await
    }
}
```

Remember `CALIBRE_CONTAINER` from Part 3's config? If it's empty,
FicHub assumes `ebook-convert` is on the host PATH and runs it
directly. If it's set (say, `calibre-srv`), the conversion happens
inside a Docker container:

```rust
/// Run `docker exec <container> ebook-convert <input> <output>`.
async fn run_docker(
    container: &str,
    epub_path: &Path,
    output_path: &Path,
    timeout_dur: Duration,
) -> Result<(), ExportError> {
    let result = timeout(timeout_dur, async {
        Command::new("docker")
            .arg("exec")
            .arg(container)
            .arg("ebook-convert")
            .arg(epub_path)
            .arg(output_path)
            .output()
            .await
    })
    .await;

    handle_command_result(result, &format!("docker exec {container} ebook-convert"))
}
```

Containerizing Calibre is a production survival move: Calibre's Python
environment is notorious for dependency hell, and pinning it in a
container means the host OS upgrades can't break conversions. The
*paths* are the subtle part — `docker exec` runs inside the container's
filesystem, so `epub_path` and `output_path` must be paths that exist
*both* in the container and on the host (a bind-mounted temp dir). That
is why the pipeline uses `state.config.tmp_dir` everywhere: the
container and the host agree on where `/tmp` lives.

And the timeout: `tokio::time::timeout` wraps the whole `Command`
future, so a hung `ebook-convert` (stuck on a giant fic, waiting on a
networked font server) can't hold a worker thread hostage forever. The
`handle_command_result` helper converts the nested
`Result<Result<Output, io::Error>, Elapsed>` into a clean
`ExportError::CalibreError` — one message for "timed out", one for
"process error", one for "non-zero exit with stderr". The stderr is
included in the error, because "Calibre failed" without the *why* is a
support dead end.

💡 **Key Concept — treat external processes as fallible services.**
Every time you shell out, you're calling a service that can hang,
crash, disagree with you, or be absent. `convert.rs` handles all four:
a timeout bounds the hang, the retry absorbs the crash, the
`container-or-host` branch handles absence, and every error carries
stderr so the *next* debugging step is obvious. Then the pipeline
(Chapter 19) wraps the whole thing in `if let Ok(...)` — so even when
conversion *fails*, the fic still has its EPUB, HTML, TXT, and MD. The
lesson: external tools are soft dependencies, and soft dependencies
must be allowed to fail loudly but not fatally.

🧪 **Try It Yourself — run ebook-convert yourself.** On a machine with
Calibre installed, convert the EPUB you peeked at in Chapter 20:

```bash
ebook-convert "$(find /personal/documents/code/rust/fichub/cache -name '*.epub' | head -1)" /tmp/out.mobi
ls -la /tmp/out.mobi
```

Then delete `/tmp/out.mobi` and run the same command with a bogus input
path — watch the stderr. That's exactly the `CalibreError` payload the
handler would log. Understanding the *tool* makes the *wrapper* obvious.

⚠️ **Watch Out — 300 seconds is a long time under load.** The timeout
is generous by design (huge fics, slow disks), but it means a single
stuck conversion can tie up a worker for five minutes. In production
you'd pair this with per-fic semaphores (already done in Chapter 19),
queue limits, and alerts when conversion latency grows. A timeout is a
boundary, not a strategy — the strategy is making sure the timeout
rarely fires.

That's the format family: a self-contained HTML bundle, a four-step
plain-text stripper, a regex-based Markdown converter, and a
timeout-bounded Calibre sidecar. Everything produced so far has ended
up in the cache — which means it's time for the last chapter of this
part: how cached files are served back out, and why the hash in every
filename is also the lock on the door.

---

## Chapter 22 — Cache Downloads and Hashes

We've generated seven formats and parked them in a cache directory —
but a file on disk that no one can download is just a disk-space
donation. This chapter closes the loop: the two handlers in
`src/routes/cache_download.rs` that *serve* those files, plus the
`cache/disk.rs` helpers that decide where files live and prove they're
intact. Two routes, registered back in Part 2's router:

```text
GET /cache/{etype}/{url_id}/{fname}   → download_with_hash (serve a specific file)
GET /cache/{etype}/{url_id}           → download_or_export (serve, or bounce to the frontend)
```

The first is what the export endpoint's URLs point at
(`/cache/epub/1_21845264?h=<hash>`). The second is the "dumb link"
fallback: no hash? No problem — check the disk anyway, then redirect
the browser to the frontend with `?id=` so the SPA can trigger an
export. One vocabulary (the `EType` enum), one path convention, and one
MD5 rule bind all of it together.

### 22.1 The EType enum: seven formats, one vocabulary

Every part of the cache system speaks through a single enum:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
    Txt,
    Azw3,
    Md,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
            EType::Txt => "txt",
            EType::Azw3 => "azw3",
            EType::Md => "md",
        }
    }

    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
            EType::Txt => ".txt",
            EType::Azw3 => ".azw3",
            EType::Md => ".md",
        }
    }
}

impl std::str::FromStr for EType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            "txt" => Ok(EType::Txt),
            "azw3" => Ok(EType::Azw3),
            "md" => Ok(EType::Md),
            _ => Err(()),
        }
    }
}
```

This one type is doing five jobs at once, and every job is visible in
the derives:

- **`Debug`** — log it in tracing statements.
- **`Clone, Hash, Eq, PartialEq`** — it's the *key* in the semaphore
  map we met in Chapter 19: `HashMap<(String, EType), Arc<Semaphore>>`.
- **`as_str()`** — the directory name (`cache/epub/...`) and the URL
  segment (`/cache/epub/...`).
- **`suffix()`** — the file extension (note `Html → ".zip"`, a
  reminder that the "HTML" export is delivered as a zip).
- **`FromStr`** — parses the URL path segment back into an enum, so
  `"epub".parse::<EType>()` works and unknown formats fail loudly. The
  `to_lowercase()` means `EPUB` and `Epub` both parse — URLs from old
  links and new links behave identically.

💡 **Key Concept — one enum, five behaviors, zero typos.** Before
`EType` existed, every module probably had its own `match` on string
literals — and every one of those matches could drift ("html" vs
"HTML", ".zip" vs "zip"). Centralizing the vocabulary in one enum means
the *compiler* enforces consistency: if you add a format, every
`match` in the codebase stops compiling until you've handled it. This
is the "make illegal states unrepresentable" idea from Part 2, applied
to file formats instead of HTTP states. The cost of the enum is one
file of match arms; the benefit is that cache paths, URLs, extensions,
semaphore keys, and MIME types can never disagree.

🧪 **Try It Yourself — the EType test suite.** `src/cache/mod.rs` has
six tests covering exactly this vocabulary: `test_etype_from_str`
(including uppercase inputs and the `"invalid"` rejection),
`test_etype_as_str`, `test_etype_suffix` (note `Html` → `.zip`),
`test_etype_version`, plus the two semaphore tests from Chapter 19.
Run the whole cache module:

```bash
cargo test --lib cache:: 2>&1 | tail -15
```

Count the tests. Every behavior this chapter describes — parsing,
naming, suffixes, bounded semaphores — is pinned down by one of them.
That's what a well-tested module looks like: the tests read like a
checklist of the design.

### 22.2 The sharded, hash-addressed cache layout

`cache/disk.rs` owns the *shape* of the cache on disk. The single most
important function is `cache_path` — and it repays close reading:

```rust
/// Compute the cache path for a given etype, url_id, and hash
///
/// Directory structure: <cache_root>/<etype>/<prefix1>/<prefix2>/<prefix3>/<url_id>/<hash><suffix>
/// where prefix chunks are 3 chars each from the url_id.
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    // Split url_id into 3-char directory chunks (up to 9 chars = 3 levels deep)
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    // Full url_id directory
    path = path.join(url_id);

    // Actual file: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

Trace it with a concrete example. For `url_id = "1_21845264"`,
`etype = Epub`, `hash = "ab12cd34"`:

```text
<cache_root>/epub/1_2/184/526/1_21845264/ab12cd34.epub
```

Why shard at all? **Filesystem scalability.** `ext4` (and every other
real filesystem) degrades when a single directory holds tens of
thousands of entries — directory lookup becomes linear, and the dentry
cache thrashes. By carving `url_id` into 3-character chunks, FicHub
spreads files across up to three levels of subdirectories, so no
directory ever holds more than a handful of related fics. The `take(3)`
caps the depth: even a 200-character url_id only gets three shard
levels. And the doc comment spells the whole layout out, which makes
`cache_path` self-documenting.

The companion helpers are small and deliberately simple:

```rust
/// Move a file from source to destination, creating parent dirs
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

```rust
/// Compute MD5 hash of a file
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

`move_to_cache` is the *commit* of the pipeline: `fs::rename` is
atomic on the same filesystem, so a file either fully appears at its
cache address or doesn't appear at all — a reader can never catch a
half-written export. And `file_md5` is the verification primitive the
download handlers are about to lean on. Notice `move_to_cache` creates
parent dirs itself: the pipeline (Chapter 19) never has to call
`ensure_cache_dir` separately — the mover is responsible for its own
destination.

⚠️ **Watch Out — `fs::rename` is atomic *within* a filesystem.** If
`tmp_dir` and `cache_dir` live on different mounts (Part 3's config
even has a `SECONDARY_CACHE_DIR` for an attached drive), rename falls
back to copy+delete — and a crash mid-copy leaves a partial file at the
destination. The code takes the simple path (rename) because in
FicHub's deployment both dirs sit on the same disk; but if you deploy
with the cache on a separate volume, you'd want a copy-then-rename with
a temp name. Know your filesystem topology before relying on atomic
rename — it's one of those assumptions that's invisible until a crash
exposes it.

### 22.3 Serving files: download_with_hash

Now the payoff — the handler that turns `/cache/epub/1_21845264?h=ab12cd34`
into a downloaded file. The security model is the interesting part:
**the client-supplied hash is verified against the file's real MD5
before a single byte is served.**

```rust
/// Direct download with hash validation:
/// GET /cache/:etype/:url_id/:fname
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    // Extract hash from query param or filename
    let hash = match params.h {
        Some(h) => h,
        None => {
            // Try to extract from filename: <hash><suffix>
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash,
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    // Validate hash
    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            // Serve file
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
                EType::Txt => "text/plain",
                EType::Azw3 => "application/vnd.amazon.ebook",
                EType::Md => "text/markdown",
            };

            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

Walk the failure modes, because this handler is a small security
curriculum:

- **Bad `etype`** → `-1 invalid format`, before touching disk. An
  attacker can't use the path segment to escape the cache root — it
  must parse into the enum first.
- **Hash from query *or* filename** → the handler accepts both forms of
  the URL (with `?h=` or with the hash embedded in the path), so old
  links keep working. Two URL shapes, one lookup.
- **File missing** → `-5 file not found`.
- **Hash mismatch** → `-5 hash mismatch`, and *nothing is served*. This
  is the important one: `cache_path` is built from client input, so a
  wrong hash means "this URL claims a file that doesn't exist at that
  address" — and the only correct response is to refuse. The
  `Ok(actual_hash) if actual_hash == hash` guard is the whole security
  model in one line.
- **Read error** → `-1 read error`. The handler reads the whole file
  with `tokio::fs::read` (async, so a big fic doesn't block a worker
  thread) and streams it with a `Content-Disposition: attachment`
  header so browsers download rather than render.

And the MIME map deserves a moment: `application/epub+zip` for EPUB,
`application/x-mobipocket-ebook` for MOBI, `application/vnd.amazon.ebook`
for AZW3, `text/markdown` for MD. These are the strings that tell
browsers and OSes what to do with the file — get one wrong and Kindles
mystify their owners, phones refuse to open EPUBs, and PDF viewers
grumble. The enum makes the map exhaustive (the compiler knows you
covered all seven variants).

💡 **Key Concept — verify, don't trust: the hash is the authorization.**
Why does the server *re-hash the file on every download* instead of
trusting the filename? Because the hash in the URL is user-supplied —
and the whole point of a content-addressed cache is that the address
*is* the proof. Re-hashing on read catches every failure mode at once:
corrupted files on disk (a bad byte changes the MD5 and the file is
refused), stale links (the hash points at a file that no longer
exists), and crafted URLs (a guess at another user's hash gets a
`-5`, not a file). This is the same trust model as Git: if the hash
matches, the content is *that* content. The cost — one MD5 pass per
download — is trivial next to the integrity guarantee.

🧪 **Try It Yourself — provoke the guards.** With a cached file handy,
hit all four failure modes:

```bash
# 1. Valid download (should return the file)
curl -sL -o /tmp/test.epub "http://localhost:3000$(curl -s 'http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/21845264' | jq -r '.epub_url')" && file /tmp/test.epub

# 2. Wrong hash (hash mismatch)
curl -s "http://localhost:3000/cache/epub/1_21845264?h=deadbeef" | jq

# 3. Unknown format (invalid format)
curl -s "http://localhost:3000/cache/exe/1_21845264?h=deadbeef" | jq

# 4. Nonexistent fic (file not found)
curl -s "http://localhost:3000/cache/epub/9_99999999?h=deadbeef" | jq
```

You'll get the file for #1, and three different `err` codes for the
rest. Memorize that shape: `err: -1` for format problems, `-5` for
missing or mismatched files. The frontend matches on these codes to
decide whether to show an error or trigger an export.

⚠️ **Watch Out — MD5 is for integrity, not security.** MD5 is broken
as a *cryptographic* hash — collisions are cheap to fabricate, so you
must never use it for signatures or password hashing. Here it's the
right tool, because the threat is *accidental* corruption, not
*adversarial* forgery: we're checking that the file on disk matches
the file we made, not authenticating it against an attacker. If you
ever need real tamper-resistance (e.g. signed downloads), reach for
SHA-256 or better and treat the MD5 as a content fingerprint only.
Knowing *why* a hash is being used is what makes the choice safe.

### 22.4 Trigger-and-redirect: download_or_export

The second route handles the "bare link" case: someone has
`/cache/epub/1_21845264` — no hash, maybe an old bookmark, maybe a
link from before the export ever ran. The handler tries the optimistic
path first (if a hash is given and the file exists and matches, serve
it), then falls back to the graceful nudge:

```rust
/// Trigger export and download:
/// GET /cache/:etype/:url_id
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // If a hash is provided and file exists, serve directly
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash,
            );
            if cache_path.exists() {
                match crate::cache::disk::file_md5(&cache_path) {
                    Ok(actual_hash) if actual_hash == *hash => { ... return file ... }
                    _ => {}
                }
            }
        }
    }

    // No cached file - redirect to the main page with ?id= parameter
    // so the frontend can trigger export
    Redirect::to(&format!("/?id={}", url_id)).into_response()
}
```

When there's nothing to serve, the handler *redirects the browser to
the frontend* with `/?id=<url_id>` — and the SPA (Part 6) sees the
`id` parameter, calls the export endpoint, gets the hashed URLs, and
swaps in the download link. The whole dance — request → redirect →
frontend → export API → hashed URL → download — is how FicHub turns a
bare cache URL into a finished file without a second server round trip.
It's also a wonderful example of **fail-forward UX**: instead of a 404,
the user lands on a page that can *fix the problem*.

⚠️ **Watch Out — the redirect is a UX decision, not an HTTP purity
one.** Purists would return 404 for a missing file. FicHub returns 302
to the frontend because 404 is a dead end for a real reader — and
readers arriving with stale links are the common case, not the
exception. The cost: every bot crawling a stale cache URL gets
redirected to the SPA, which is exactly where you *want* crawlers to
go (the frontend is crawlable). Design your error responses around the
user's next step, not the spec's least surprise.

💡 **Key Concept — content addressing is a full-stack contract.** Step
back and look at what we've built across this part: the export
endpoint *returns* hashes; the cache *names* files by hash; the
download handler *verifies* by re-hashing; and the frontend *carries*
the hash in the URL. The hash is the contract that makes all four
agree. It's the filename, the cache key, the integrity proof, and the
authorization token — four roles, one string, no shared state required
beyond the file itself. That's why content-addressed storage is such a
powerful idea: the data and its name are inseparable, and every layer
can verify the others without trusting them.

That closes the loop — URL in, file out, hash verified at every step.
You now own the entire export journey: the pipeline that gates, caches,
and coordinates; the pure-Rust EPUB builder; the HTML/TXT/MD family and
the Calibre sidecar; and the sharded, hash-addressed cache that serves
it all back with integrity checks on every download.

---

That's Part 5 done — the export pipeline is now your playground. You
understand how a URL passes through rate limits and blacklists, why the
semaphore plus double-check keeps concurrent exports from duplicating
work, how the pure-Rust EPUB builder turns chapters into a real `.epub`,
how HTML/TXT/MD are generated and how Calibre's `ebook-convert` (or a
Docker container running it) produces MOBI/PDF/AZW3, and how every file
lands in a sharded, hash-addressed cache where the hash is both the
filename and the proof of integrity.

Next, in **Part 6 — The API: Meta, Search, and Reader**, we zoom out
from files back to JSON: the metadata endpoint, the boolean search
engine, and the web reader that renders those cached chapters in the
browser. See you there.
