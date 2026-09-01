# Wayback Machine Fallback — Implementation Plan

> **Status**: draft 2026-08-27. Slots into P8#5 (Wayback fallback) + P1#1 (host-blocking mitigation) + Self-Healing M1/M2.

## Objective

Fall back to the Internet Archive Wayback Machine when a native scraper fails due to host-level blocking (FFN 403 Cloudflare, AO3 404+challenge) or transient downtime, so the archive can still ingest content without requiring immediate user-supplied cookies.

## Architecture & design decisions

1. **FicHub-level wrapper, not crate-level.** Fallback lives in `src/scrape/wayback_fallback.rs`, not inside `fanfic-scrapers`. The crate stays a pure site-agnostic adapter registry. FicHub's dispatch path handles "try native → catch specific errors → try Wayback".

2. **HTTP client.** `reqwest` directly for the CDX API. Keep dependencies minimal — no dedicated Wayback crate.

3. **HTML pre-processing.** Wayback injects a toolbar (`#wm-ipp`) and rewrites URLs. Strip the toolbar + archive analytics scripts, optionally inject `<base href>` so relative URL resolution in native scrapers doesn't break.

4. **Rate limiting.** CDX API is rate-limited. Reuse FicHub's existing Redis token bucket or a local `tokio::sync::Semaphore` at a conservative 1 req/sec for CDX queries.

5. **Trigger conditions.** Only trigger on `ScrapeError` variants that indicate host blocking or transient failure: HTTP 403, HTTP 404 (AO3 challenge), and selected transient/timeout errors. Do NOT trigger on `Unsupported`, `AuthRequired`, or structural parse failures (those need M2, not Wayback).

## Configuration

In `src/config.rs` and `.env.example`:

```rust
pub wayback_fallback_enabled: bool,        // default: false (opt-in until proven stable)
pub wayback_max_snapshot_age_days: i32,   // default: 365
pub wayback_cdx_rate_limit_per_sec: u64,  // default: 1
pub wayback_user_agent: String,           // default: "FicHub-Archive-Bot/1.0 (+https://fichub.polarisocial.xyz)"
```

## Step 1: Config + env

- Add the four fields above to `Config`, parsed from env with the defaults listed.
- Add to `.env.example` with comments.
- Gate the entire fallback path on `wayback_fallback_enabled` — off by default.

## Step 2: Wayback CDX client (`src/scrape/wayback.rs`)

- `fn get_latest_wayback_url(original_url: &str, cfg: &Config, client: &reqwest::Client) -> Result<Option<String>, ScrapeError>`
- Encode `original_url`, query:
  `https://web.archive.org/cdx/search/cdx?url={encoded}&output=json&collapse=urlkey&filter=statuscode:200&limit=1&from={date}`
  where `from` = today - `wayback_max_snapshot_age_days`.
- Parse JSON: take the most recent snapshot timestamp, construct
  `https://web.archive.org/web/{timestamp}/{original_url}`.
- Enforce rate limit: semaphore or Redis bucket at `wayback_cdx_rate_limit_per_sec`.
- Return `Ok(None)` when no snapshot exists (not an error — just no fallback).

## Step 3: HTML pre-processing (`src/scrape/wayback.rs` or a shared helper)

- `fn clean_wayback_html(html: &str, original_url: &str) -> String`
- Remove `<div id="wm-ipp">...</div>` (toolbar).
- Remove `<script>` tags whose src/domain points to `archive.org` analytics.
- Optional: inject `<base href="{original_url}">` into `<head>`.
- Keep it lightweight: `scraper` or well-anchored regex. Do NOT run the full native parse on raw Wayback HTML.

## Step 4: Scraper registry fallback wrapper

Location: FicHub's dispatch path — likely `src/scrape/registry.rs` (FicHub-specific wrapper around the crate registry) or the export handler in `src/routes/export.rs`.

Flow:

1. Call `registry.lookup(&client, url, creds)`.
2. If `Ok`, proceed normally.
3. If `Err(ScrapeError::HttpError(403|404))` or selected transient/timeout variants, **and** `cfg.wayback_fallback_enabled`:
   - Log to `scrape_failures` with classification `blocked` or `transient`, plus a `fallback_attempted: wayback` marker.
   - Call `get_latest_wayback_url(url, cfg, client)`.
   - If `Some(wayback_url)`: fetch the snapshot HTML with a fresh/headless `reqwest::Client`, run `clean_wayback_html`, then hand the cleaned HTML to the native scraper's parse path (or a FicHub-level parse shim that accepts pre-fetched HTML).
   - If `None` or Wayback fetch fails: return the original error; let M1/M2 or user cookie ingestion pick it up.
4. If the error is `Unsupported`, `AuthRequired`, `ParseError(Structural)`, do NOT fall back to Wayback.

Key implementation note: do NOT pass the Wayback URL to the native scraper as if it were the original URL — native selectors may break on rewritten internal links. Fetch + clean HTML first, then invoke parsing on the cleaned string or via a client middleware that transparently serves the snapshot when the scraper requests the original URL.

## Step 5: Database + telemetry

- Add `fallback_source` to `scrape_failures` (text, e.g. `"wayback_20231015"`).
- In exported EPUB metadata or the work's UI, surface a subtle note when a snapshot was used: "Retrieved via Wayback Machine (snapshot: YYYY-MM-DD)" — curators and readers should know the source may be slightly stale.
- Record the Wayback snapshot timestamp used, so a later re-scrape without the fallback can be detected.

## Step 6: Testing + QA

- **Unit:** mock the CDX JSON response; assert `get_latest_wayback_url` parses correctly and returns the right snapshot URL; assert `clean_wayback_html` strips the toolbar.
- **DB-gated:** a test that forces a 403 on a known URL and asserts the fallback path is invoked when enabled, and that it is NOT invoked when disabled.
- **QA:** `qa/run.js` / `qa/api-walk.js` should still pass with the fallback disabled; add a config-awareness check so the harness doesn't break when the feature is off.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| CDX rate-limit / IP ban | 1 req/sec local limit, dedicated user-agent, honour `Retry-After` if returned. |
| Stale content | `wayback_max_snapshot_age_days` caps age; CDX sorted by date gives the most recent. |
| Broken relative URLs | Strip toolbar + analytics; inject `<base href>`; most FicHub selectors use absolute URLs or ignore href values. |
| Wayback CAPTCHA | Fail gracefully, log to `scrape_failures`, defer to M2 agent or P1#1 cookie ingestion. |
| HTML drift (Wayback markup changes) | Keep the cleaner anchored on stable IDs (`#wm-ipp`); if it breaks, it fails closed (returns original error), not open (bad parse). |

## Alignment with roadmap

- **P1#1 cookie ingestion:** complementary stopgap. Wayback handles immediate host-blocking without user action; cookie ingestion remains the long-term path for new chapters on blocked sites.
- **Self-Healing M1/M2:** fits M1 telemetry — native fail → try Wayback → if Wayback also fails or hits CAPTCHA → log `blocked` → M2 agent or user cookies pick it up.
- **P8#5 Wayback fallback:** this is the direct implementation of that backlog item.

## Sequence (implementation order)

1. Config + env (`src/config.rs`, `.env.example`).
2. CDX client + HTML cleaner (`src/scrape/wayback.rs`).
3. Fallback wrapper in FicHub dispatch path, gated on config.
4. `scrape_failures.fallback_source` column + metadata/UI indicator.
5. Unit + DB-gated tests.
6. QA pass with feature off, then a controlled live smoke with feature on against a known-archived URL.

## Open questions

- Exact `ScrapeError` variants that should trigger Wayback (403/404 yes; which transient/timeout codes; what about 5xx from the target vs 5xx from Wayback).
- Whether to inject `<base href>` always or only when the native scraper is known to use relative URL resolution.
- Whether the snapshot indicator should appear in EPUB metadata, the work UI, or both.
- Rate-limit back-end: Redis token bucket vs local `Semaphore` — Redis is more correct across processes but adds a dependency for a feature that's off by default.
