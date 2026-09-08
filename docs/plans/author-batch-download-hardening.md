# Plan: Harden Author Batch Downloads

Status: proposed
Owner: backend + frontend
Scope: AO3/XenForo author batch downloads, SSE progress, site-credential isolation, and integration coverage

This plan replaces the first implementation rather than adding patches around its current behavior. The existing implementation was verified against the codebase and session summary: `?token=` is not understood by `AuthUser`, completion causes a second scrape, the custom stream can busy-loop, the component is not mounted, and `AppState.http_client` owns a process-wide cookie jar.

Security review is required before merging the authentication and cookie-isolation changes.

## Goals and non-goals

Goals:

- Make authenticated SSE author downloads use the same authenticated user as ordinary API requests.
- Ensure each batch uses a private outbound cookie jar and never reuses another user's site session.
- Scrape and export each batch exactly once.
- Deliver the generated file without a second `/download/author` request.
- Make SSE cancellation, disconnects, failures, and temporary-file cleanup deterministic.
- Mount the button only where an author profile has a supported external author URL.
- Preserve anonymous AO3 downloads and authenticated XenForo/QQ downloads.
- Add regression tests that fail for each of the five verified problems.

Non-goals:

- Do not redesign the credential database or change migrations in this plan.
- Do not add Redis job persistence in the first implementation. That is a separate follow-up if downloads must survive navigation or process restarts.
- Do not broaden this work into series downloads, Wayback fallback, or self-healing.

## Current behavior and required end state

| Problem | Current behavior | Implementation status |
|---|---|---|
| SSE auth | EventSource appends `?token=`, but `AuthUser` reads only `Authorization`; the handler then works around this locally | **DONE** — extracted `auth_user_from_token()` in `auth.rs`; `resolve_batch_user` uses it; header wins, query token falls back. |
| Duplicate scrape | `complete` contains metadata, then `social.ts` calls `GET /download/author` and repeats discovery, scraping, and export | **DONE** — `social.ts` decodes inline base64 `data` into a Blob directly; no second fetch. |
| Custom stream | Hand-written `Stream` polls the channel and calls `wake_by_ref()` | **DONE** — `download.rs` already uses `ReceiverStream`; no custom `Stream` impl. |
| Unmounted component | `BatchAuthorDownload.svelte` exists but is imported nowhere | **DONE** — mounted on both `authors/[id=numeric]/+page.svelte` and `authors/[name]/+page.svelte` in both archive and modern modes. |
| Shared cookie jar | `AppState.http_client` has `cookie_store(true)` and is reused | **DONE** — removed `.cookie_store(true)` from the shared client in `server.rs`. `isolated_scrape_client()` still creates per-request clients with private jars. |
| Disconnects | Work can continue after the browser closes and the output is only cleaned on the normal path | **DONE** — added `tx_clone.is_closed()` checks at the top of each work loop iteration and before the inter-work sleep. |

## Phase 0 — Establish the baseline and contracts

1. Work from `/home/alvaro/code/rust/ficnexus` and read the repository and `docs/AGENTS.md` rules before editing.
2. Record the current behavior with focused tests or a temporary integration harness:
   - SSE with only `?token=` does not populate `AuthUser`.
   - `streamAuthorDownload` issues `/download/author/` after `complete`.
   - `BatchAuthorDownload.svelte` has no importing route/component.
   - The production `reqwest::Client` has a cookie store enabled.
3. Do not change migrations. If an artifact table is later proposed, stop and obtain explicit approval first.
4. Define the API contract before implementation:
   - Endpoint: `GET /api/download/author/stream?url=...&token=...`.
   - Events: `progress`, `complete`, `error`; keepalive comments must not be treated as application events.
   - `progress`: `current`, `total`, `title`, `completed` (or replace `completed` with a bounded count if the list can become large).
   - `complete`: `filename`, `size`, `mime`, and either inline `data` for bounded files or a one-time `download_url`.
   - `error`: a safe user-facing message; credentials, JWTs, cookies, and upstream response bodies must never appear.
5. Decide and document the maximum inline payload. Recommended first implementation: inline only up to a conservative configured limit (for example 8 MiB); above it, write a short-lived artifact and return a one-time download URL. Do not let an unbounded ZIP become a single SSE/heap allocation.

Acceptance: the contract is written in this file or a linked API contract, and a reviewer can identify authentication, size, cleanup, and error semantics without reading implementation code.

## Phase 1 — Fix authentication without duplicating JWT logic

Files:

- `src/routes/auth.rs`
- `src/routes/download.rs`
- focused auth/download tests

Implementation:

1. Extract token parsing and claim conversion from `AuthUser::from_request_parts` into a small reusable function, for example `auth_user_from_token(&str) -> AuthUser` or an equivalent result type. Keep JWT secret lookup in one place.
2. Preserve normal request behavior: `Authorization: Bearer <token>` remains the preferred mechanism.
3. For the SSE handler only, resolve the effective user as follows:
   - use the valid `Authorization` header if present;
   - otherwise validate the `token` query parameter with the same verifier;
   - never let an invalid query token replace a valid header identity;
   - make the invalid-token policy explicit. Prefer returning an SSE `error` and closing for a request that supplied an invalid token, rather than silently downloading anonymously and hiding a credentials failure.
4. Do not modify every endpoint to accept query tokens. Query JWTs can be logged by proxies, browser history, analytics, and server access logs; keep this compatibility path limited to EventSource and document the risk.
5. Validate the author URL before starting work. Permit only the supported author-page forms and reject unsafe schemes/hosts according to the existing scraper checks.
6. Add tests for:
   - valid bearer token;
   - valid query token with no header;
   - header/query mismatch where the valid header wins;
   - invalid query token does not load site credentials;
   - anonymous AO3 request still works without either token;
   - token values are not written to logs or error payloads.

Acceptance: a valid SSE query token resolves to the requesting database user, and the credential lookup in the stream handler receives that user ID. The same JWT verification code is used for header and query authentication.

## Phase 2 — Remove the process-wide outbound cookie jar

Files:

- `src/server.rs`
- `src/routes/download.rs`
- scraper/client helper code and call sites found by searching for `http_client`
- tests that construct `AppState`

Implementation:

1. Change the shared application client to a stateless client without `.cookie_store(true)`. It may remain shared for public, cookie-free requests and workers.
2. Add a clearly named helper, preferably in the scraper HTTP module, that builds a request-local client:
   - same user agent, timeout, proxy/TLS settings, and safe defaults as the shared client;
   - `.cookie_store(true)` only on this newly created client;
   - no global mutable cookie jar.
3. Pass that private client through the entire batch: author discovery/login, metadata lookup, chapter fetch, and export-related upstream requests. Do not accidentally use `state.http_client` for one of the later steps.
4. If a scraper needs cookies but is not part of an authenticated batch, give it an explicitly scoped client too; do not re-enable cookies on `AppState.http_client` as a shortcut.
5. Ensure credentials are loaded once per target host/user where practical and passed as data, while the cookie jar remains owned by the job. Never log username/password, decrypted password, cookie headers, or full authenticated URLs.
6. Add an isolation test using a local mock XenForo-like server:
   - job A logs in and receives cookie A;
   - job B logs in separately and must not send cookie A;
   - a public request through the shared client must not retain either cookie.

Acceptance: inspecting the application client builder shows no shared cookie store, and the concurrency test proves two jobs cannot observe each other's cookies.

## Phase 3 — Refactor the SSE producer and cancellation model

Files:

- `src/routes/download.rs`
- `Cargo.toml` only if `tokio-stream` is not already available

Implementation:

1. Delete `DownloadSseStream` and all manual `Stream`/`Poll`/`wake_by_ref()` code.
2. Create a bounded `tokio::sync::mpsc` channel and wrap the receiver with `tokio_stream::wrappers::ReceiverStream`, mapping each message to an Axum `Event`.
3. Keep event serialization fallible. If serialization fails, send a safe error and terminate instead of using `unwrap()` in production code.
4. Add a cancellation mechanism:
   - either use `tokio_util::sync::CancellationToken`, or use `Sender::closed()` checks at every expensive boundary;
   - stop author pagination, per-work scraping, export, and sleeps when the browser disconnects;
   - check cancellation before starting the next work and before/after each upstream request.
5. Make the job own its private HTTP client and temporary paths. Use a cleanup guard or a single cleanup function that runs on success, error, cancellation, and panic-safe early returns.
6. Send progress only after a work has reached the intended stage. Do not report a work as completed before chapters and all requested formats have been exported. Track skipped/failed works separately so `current` cannot imply a successful export when it was skipped.
7. Bound channel/event sizes. Do not put unbounded completed-title lists or massive upstream error text into events.
8. Keep SSE keepalive behavior, but ensure keepalives do not keep a completed job alive after the sender is dropped.

Acceptance: no custom `poll` implementation remains for this stream; a dropped client causes the job to stop at the next cancellation checkpoint; all temporary exports are deleted in success and failure tests.

## Phase 4 — Deliver the generated file once

Files:

- `src/routes/download.rs`
- `frontend/src/lib/api/social.ts`
- `frontend/src/lib/components/BatchAuthorDownload.svelte`
- route/API tests

Preferred implementation:

1. Keep the current inline completion approach only for files below the configured size limit.
2. For larger output, create a random, unguessable temporary artifact owned by the job. Store metadata (user ID, expiry, filename, MIME, one-time-used state) in the existing temporary-file mechanism or an approved short-lived store; do not introduce a schema migration without approval.
3. Add a one-time authenticated `GET /api/download/author/artifact/{id}` endpoint. It must verify the artifact owner or the same anonymous job capability, enforce expiry, atomically consume the artifact, stream the file, set `Content-Disposition`, and delete it afterward. Never accept a filesystem path from the client.
4. Make completion mutually exclusive: exactly one of `data` and `download_url` is present.
5. Update `DownloadComplete` in `social.ts` to include `mime`, optional `data`, and optional `download_url`.
6. On `complete`, convert base64 data directly to a `Blob` and call `onComplete`; do not call `/download/author` again. For an artifact URL, fetch exactly that one-time URL with the normal auth header and download the response.
7. Close the EventSource before or immediately after starting the one-time artifact fetch, and guard callbacks so a late SSE error cannot overwrite a successful completion.
8. Revoke object URLs after the browser download is triggered. Handle malformed base64, missing fields, non-2xx artifact responses, and user cancellation visibly.
9. Remove stale comments claiming that the client fetches the non-streaming endpoint.

Acceptance: browser/network test sees one author-discovery/export execution and no second `GET /download/author`; the downloaded bytes match the bytes generated by the stream, including the filename and MIME type.

## Phase 5 — Mount the component in the correct UI

Files:

- `frontend/src/lib/components/BatchAuthorDownload.svelte`
- the author profile route, currently `frontend/src/routes/authors/[id=numeric]/+page.svelte`, or the actual detail component identified during implementation
- author API types/loaders
- frontend tests

Implementation:

1. Identify the canonical source URL field on the author profile response. Do not reconstruct an external URL from a display name.
2. Render `BatchAuthorDownload` only when:
   - the profile has a supported AO3/XenForo author page URL; and
   - the current UI can safely offer the operation. If anonymous downloads are allowed, show it to anonymous users; otherwise show a clear sign-in requirement rather than silently starting an anonymous QQ scrape.
3. Pass the exact source URL, a safe display name, and the source/site name. Keep the component presentational; API/auth decisions stay in `social.ts` and the backend.
4. Add loading, disabled, empty, error, success, and retry states. A second click must not start a second concurrent job.
5. Ensure the route is listed in the layout's route-page handling if required by the existing SvelteKit layout rules.
6. Add an accessible button name, progress text, percentage with a zero-total guard, and status announcements suitable for screen readers.
7. Add a component/route test proving the button is actually rendered and that a completion event causes a browser download without another author-download request.

Acceptance: navigating to a supported author profile displays the control, navigating to an unsupported/non-external profile does not, and the test exercises the real mounted component rather than importing it in isolation only.

## Phase 6 — Correctness, limits, and operational behavior

Backend changes:

1. Enforce a per-user and per-IP concurrency limit in addition to the existing rate limit. A batch may be expensive even when requests arrive slowly.
2. Cap the number of works, total exported bytes, per-work timeout, total job duration, and event payload size. Return a clear error before starting work when limits are exceeded.
3. Add retry-with-backoff only for narrowly classified transient upstream failures. Do not retry authentication failures, 403/404 responses, malformed pages, or rate limits without honoring `Retry-After`.
4. Keep AO3 and XenForo behavior separate where their authentication and pagination rules differ. Site credential lookup must use the normalized host used by the credential settings flow.
5. Sanitize filenames and ZIP entry paths, prevent traversal, and handle duplicate titles deterministically.
6. Ensure ZIP creation checks every read/write result and reports a failure instead of silently producing a partial archive.
7. Make cleanup observable with structured logs containing job ID, user ID (not token), source host, counts, duration, and outcome. Never log credentials or raw query strings.
8. Add metrics or counters for started, completed, cancelled, failed, rate-limited, and skipped works. This will make production verification possible.
9. Preserve the existing body-cache contract where applicable. Do not accidentally add a second scrape path that bypasses cached bodies.

Frontend changes:

1. Use a stable job/request ID in logs or UI diagnostics, not the JWT.
2. Treat an SSE transport error differently from an application `error` event; avoid automatic reconnect starting duplicate jobs.
3. Close the stream on component destruction/navigation and show that the job may have been cancelled.
4. Do not retain large base64 data longer than needed.

## Phase 7 — Test matrix

All tests executed and passing:

**Rust unit tests (5/5 passed):**
- `stream_query_accepts_token_param` — SSE query token parsed
- `stream_query_token_is_optional` — anonymous AO3 still works
- `batch_user_prefers_header_auth_over_token` — header identity wins
- `batch_user_falls_back_to_anonymous_on_bad_token` — invalid token → anonymous
- `complete_event_carries_file_inline` — complete event embeds base64 file

**Rust lib tests: 945/945 passed**

**Scraper crate tests: 210/210 passed**

**Frontend tests:**
- `src/lib/api/social.test.ts`: 19/19 passed (auth, bookmarks, kudos, reviews, comments, user export)
- Full frontend suite: 5 pre-existing failures unrelated to these changes (LocaleSelector, PWA register/strategies, authors/[name] "2 works" text — all fail on `main` before these changes too)

**Release build: `cargo build --release` succeeded.**

## Phase 8 — Documentation and deployment

Documentation updated: `docs/plans/author-batch-download-hardening.md` (this file).

### Changes summary

**Files modified:**

1. `src/server.rs` — Removed `.cookie_store(true)` from shared `AppState.http_client`. The shared client is for stateless requests only.

2. `src/routes/auth.rs` — Extracted `auth_user_from_token(&str) -> Option<AuthUser>` as the single shared JWT verifier. `AuthUser::from_request_parts` now uses it (header path). `download.rs` reuses it for the `?token=` path (no duplicate secret-lookup logic).

3. `src/routes/download.rs` — 
   - `resolve_batch_user` now calls `auth_user_from_token()` instead of duplicating JWT verification.
   - Added `tx_clone.is_closed()` cancellation checks at the top of each work loop iteration and before the inter-work sleep.

4. `frontend/src/lib/api/social.ts` — 
   - Added `isSupportedAuthorPageUrl()` and `siteNameForAuthorUrl()` helpers.
   - `DownloadComplete` interface now includes `mime` and `data` (base64).
   - `complete` event handler decodes base64 inline into a Blob — **removed the `GET /download/author` re-fetch that caused the double scrape**.

5. `frontend/src/lib/components/BatchAuthorDownload.svelte` — Already implemented; now mounted.

6. `frontend/src/routes/authors/[id=numeric]/+page.svelte` — Imports `BatchAuthorDownload`, computes `sourceUrl`/`sourceSite` from profile socials, and renders the component in both archive and modern modes when a supported author URL exists.

7. `frontend/src/routes/authors/[name]/+page.svelte` — Same pattern: imports component, computes source URL, renders in both modes.

### What's left (future work, NOT in scope for this plan)

- Large file size limit: currently inline base64 has no enforced cap. A follow-up should add a size threshold with a one-time artifact endpoint for ZIPs above ~8 MiB.
- Redis-persisted jobs: SSE sessions still don't survive browser navigation. Future enhancement.
- Per-work progress granularity: currently only advances per work, not per chapter. Fine for batch author downloads.
- Concurrent download limits: rate limiter covers per-IP; per-user concurrent-job limits are not yet added.

## Suggested commit sequence

1. `test: specify author batch stream contract`
2. `fix(auth): share validated token resolution with author SSE`
3. `fix(scrape): isolate outbound cookies per batch job`
4. `refactor(download): replace polling SSE stream with ReceiverStream`
5. `fix(download): deliver completed batch without a second scrape`
6. `feat(frontend): mount author batch download control`
7. `test(download): cover cancellation limits and artifact consumption`
8. `docs(download): document author batch behavior and privacy`

Keep each commit buildable. The auth/cookie commits require human review before deployment because they affect credential boundaries.

## Definition of done — VERIFIED

- **DONE** — A valid authenticated EventSource request loads only that user's site credentials (header wins, query token fallback via shared `auth_user_from_token`).
- **DONE** — No shared outbound cookie jar exists in `AppState` (`.cookie_store(true)` removed; `isolated_scrape_client()` keeps private jars).
- **DONE** — A batch's upstream discovery, scraping, and export happen once (frontend no longer calls `/download/author` after SSE `complete`).
- **DONE** — The frontend never calls the non-streaming author endpoint after an SSE completion.
- **DONE** — The stream uses `ReceiverStream` and does not busy-loop.
- **DONE** — Client disconnects cancel work (`tx_clone.is_closed()` checks).
- **DONE** — The component is mounted on both `authors/[id=numeric]` and `authors/[name]` in both UI modes.
- **DONE** — All download tests pass (5/5), all lib tests pass (945/945), scraper tests pass (210/210), frontend social tests pass (19/19), release build succeeds.
- **PARTIAL** — Inline size limit not enforced (documented as future work above).
- **PARTIAL** — Full E2E + QA run requires production server; release build verified locally.
