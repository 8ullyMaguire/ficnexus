# FicNexus mature polished refactor plan

Status: proposed plan; no source code changed
Audience: junior developers working in small, reviewable pull requests
Repository: `~/code/rust/ficnexus`

## 1. Purpose and product destination

This plan turns the current feature-rich FicNexus codebase into a coherent,
calm, maintainable product rather than adding another disconnected feature.
The destination is a mature fanfiction archive and community:

- A visitor can find a story, understand its quality and content warnings,
  read it immediately, and export it without learning the site.
- A returning reader has one dependable library: bookmarks, shelves, reading
  progress, history, follows, saved searches, downloads, and notifications.
- Discovery is useful but explainable. Recommendations, search suggestions,
  fandom hubs, author pages, and roadmap results show why they are shown.
- Community features help readers and authors without turning the archive into
  a popularity contest. Trust, moderation, reports, and public audit trails
  are understandable and consistent.
- The site works for keyboard users, mobile readers, screen readers, low-bandwidth
  users, and e-reader clients.
- Operators can run it on one modest self-hosted machine, understand failures,
  restore data, and upgrade without guessing.
- The product reflects the needs that users repeatedly selected: dependable
  access to stories, excellent reading and downloading, strong discovery,
  personal library tools, respectful community features, accessibility, and
  operator reliability.

The Elo/arena is only the method used to discover and prioritize those needs;
it is not a user-facing product concept. A normal reader should not need to
know that it exists, see Elo scores while browsing, or organize their library
around rankings.

The current repository already contains most of the product surface: Rust/Axum,
SvelteKit, PostgreSQL, Redis, Ollama, native scrapers, exports, reader, search,
recommendations, forum, trust, marketplace, ActivityPub, OPDS, and QA tooling.
The work below is therefore a refactor and consolidation program. It is not a
rewrite and it does not assume that every backlog item should be implemented.

## 2. Evidence used for this plan

The plan is based on the repository as inspected on 2026-09-08:

- `docs/brainstorm-02-roadmap-status.md` is the roadmap/status source. It says
  the core platform, scraper parity, forum, reader, advanced search, roadmap
  arena, recommendation platform, trust system, extension marketplace, and
  author batch download are already shipped or substantially implemented.
- The product roadmap explicitly ranks work by P1–P9 and by the roadmap
  Elo/arena. The highest-leverage themes are content reach, reliability,
  route-wide QA, personalized/explainable discovery, moderation, and a small
  cohort-ready experience. The P8 user-facing batch emphasizes download
  reliability, update notifications, reading identity, accessibility, and
  recommendation trust.
- The current codebase is large enough that structure is now a product concern:
  approximately 318 Rust files / 109k Rust lines, 179 Svelte files / 54k Svelte
  lines, 184 frontend route files, and 38 SQL migrations.
- `src/routes/forum.rs` is a particularly large concentration of behavior.
  The frontend has many route-local implementations and more than one historical
  API style. This makes consistency more important than another isolated UI.
- Existing working-tree changes were not modified by this plan. Before coding,
  commit or stash unrelated changes and re-run the baseline checks.

The roadmap does not provide a single stable numeric Elo table in a form that
should be copied into code. Therefore, this plan uses the consensus themes and
priority ordering currently recorded in the roadmap. Before each major phase,
confirm that the selected work still matches the latest user consensus. Do not
invent Elo values in implementation code or expose prioritization machinery as
part of the reader experience.

## 3. Product principles

1. Preserve the archive promise first. Downloading, reading, search, and
   library state outrank gamification, marketplace polish, and novelty.
2. Prefer one concept and one API shape. A user should not see separate rules
   for legacy comments, forum posts, requests, and reviews when the behavior is
   the same.
3. Make state visible. Show loading, empty, error, retry, queued, stale, and
   completed states rather than silently doing nothing.
4. Explain ranking. Every recommendation, search sort, consensus result, and
   moderation decision gets a short human-readable reason.
5. Make destructive actions reversible or auditable. Use soft deletion,
   proposals, confirmation, and the public/private modlog rules already present.
6. Keep self-hosting viable. Prefer PostgreSQL, Redis, filesystem storage, and
   existing background binaries over adding a mandatory distributed service.
7. Migrate incrementally. Every phase must leave the application buildable and
   deployable.
8. Test behavior at the boundary. A route is not finished when its handler
   compiles; it needs an API test, UI state coverage, and route-walk coverage.
9. Treat user feedback as data. Record which feedback informed a change and
   link the implementation issue to the roadmap feature/arena item.
10. Trust is the governance axis. Do not create a second role/level system when
    an existing trust rule can express the requirement.

## 4. Target architecture

### 4.1 Backend layout

Refactor the backend toward feature modules with the same internal shape:

```text
src/features/<feature>/
  mod.rs              public types and router()
  model.rs            request/response/domain types
  repository.rs       SQL and database mapping
  service.rs          business rules and transactions
  routes.rs           Axum handlers only
  tests.rs            unit tests for pure rules
```

Use this first for new or actively changed features. Do not move every file in
one PR. Existing shared infrastructure remains in `src/` until a feature is
migrated.

Target feature boundaries:

- `archive`: works, sources, metadata, author, series, tags, ingestion
- `downloads`: export formats, jobs, progress, cache, Kindle
- `reader`: reader bundle, progress, annotations, reading history
- `discovery`: search, body search, Ask, recommendations, roadmap arena
- `library`: bookmarks, shelves, lists, follows, saved searches, history
- `community`: comments, reviews, requests, reactions, forum, messaging
- `governance`: trust, reports, moderation, modlog, curator approvals
- `identity`: auth, sessions, settings, privacy, data export
- `integrations`: RSS/Atom, OPDS, ActivityPub, bot/CLI API
- `operations`: health, analytics, scraper health, jobs, retention, admin

The router in `src/server.rs` should eventually only compose feature routers,
middleware, and static serving. It should not contain business decisions.

### 4.2 Shared application services

Create small explicit services instead of copying rules into handlers:

- `AuthContext`: authenticated user, trust level, admin/curator capabilities.
- `ProblemResponse`: one JSON error format with status, stable code, message,
  field errors, request ID, and optional retry-after.
- `Pagination`: cursor encoding/decoding with a maximum page size.
- `WritePolicy`: rate limit, idempotency, honeypot, trust gate, and audit hook.
- `JobQueue`: database-backed job records plus existing Tokio workers for
  exports, scraping, notifications, alerts, and maintenance.
- `Audit`: one service for user-visible moderation records and private security
  records, with explicit redaction.
- `FeatureFlags`: configuration and gradual rollout, not scattered env checks.
- `Telemetry`: structured event names and zero-PII analytics.

The service functions must be usable without HTTP. A junior developer should
be able to test a rule by constructing input data without starting the server.

### 4.3 Frontend layout

Keep SvelteKit SPA mode, but standardize the frontend around:

```text
frontend/src/lib/
  api/                  typed client and endpoint modules
  components/           reusable product components
  features/<feature>/   feature-specific stores/components/types
  stores/               auth, preferences, notifications, reader
  ui/                   primitives and layout
  validation/           form and response validation
  a11y/                 focus, announcements, keyboard helpers
```

Every route should follow the same state model:

```text
loading -> ready(data)
                 -> empty
                 -> error(retryable | forbidden | not_found)
```

Avoid route files that contain a complete API client, duplicated card markup,
or a second interpretation of authentication state. Keep all new UI strings in
all supported dictionaries, and add tests for the locale fallback.

### 4.4 Data and jobs

Keep PostgreSQL as the source of truth. Use Redis for rate limits, short-lived
caches, locks, and progress fan-out only. Store long-lived export/body files on
the configured filesystem cache and keep their metadata in PostgreSQL.

Introduce a durable `jobs` model for work that outlives one HTTP request. A job
needs: id, kind, owner, status, progress, input JSON, result reference, error
code, attempts, timestamps, cancellation request, and retention deadline.
Workers must be idempotent: retrying a job must not duplicate a bookmark,
notification, export, or moderation decision.

## 5. Phase 0: baseline, decisions, and safety rails

Goal: make the current behavior measurable before moving code.

### Tasks

1. Create a clean worktree or commit unrelated changes. Do not mix this plan
   with the existing batch-download changes.
2. Record the baseline in `docs/plans/mature-polished-baseline.md`:
   - `cargo test --lib`
   - each DB-gated suite individually with `--test-threads=1`
   - `cargo test -p fanfic-scrapers`
   - `cd frontend && npm test`
   - `cd frontend && npm run build`
   - `node qa/run.js` if the local server and QA dependencies are available
   - `node qa/api-walk.js` against the local server
3. Create `docs/architecture/` with:
   - `current-map.md`: where each feature currently lives
   - `target-map.md`: target module and owner
   - `api-conventions.md`: auth, errors, pagination, idempotency
   - `data-lifecycle.md`: database, Redis, body cache, export cache, backups
4. Add a `just architecture-check` or equivalent script that checks:
   - route registration is still present,
   - migrations are numbered and unique,
   - frontend build succeeds,
   - API route-walk has no unexpected 5xx responses.
5. Add a pull-request template checklist requiring roadmap item, user
   feedback source, migration note, tests, accessibility check, and rollback.

### Done when

A new developer can start the application, run the baseline commands, find the
owner of each major feature, and see which failures predate the refactor.

## 6. Phase 1: establish contracts before moving behavior

Goal: stop API and UI drift.

### Backend contract

1. Add `src/api/response.rs` with the standard response and problem types.
2. Add `src/api/pagination.rs` with cursor helpers and tests for malformed,
   expired, oversized, and cross-resource cursors.
3. Add `src/api/request_id.rs`; accept a safe client request ID or generate one,
   return it in the response header, and include it in tracing spans.
4. Add a typed API contract document generated from route metadata. Until an
   OpenAPI generator is selected, maintain a checked-in endpoint table and make
   `qa/api-walk.js` consume the same route inventory.
5. Convert one low-risk endpoint in each family as examples: health, search,
   bookmarks, forum read, and export metadata. Do not convert all endpoints in
   this phase.

### Frontend contract

1. Add a single `ApiError` parser that reads the standard problem response.
2. Add typed API modules for `archive`, `library`, `discovery`, `community`,
   and `identity`. Keep legacy wrappers as compatibility shims until callers
   are migrated.
3. Add shared `AsyncState`, `EmptyState`, `ErrorState`, `LoadingSkeleton`,
   `RetryButton`, and `PermissionNotice` components.
4. Add a `useMutation` helper that handles disabled state, duplicate clicks,
   success announcement, error display, and retry.
5. Add contract tests for auth-required, permission-denied, validation-error,
   rate-limited, and not-found responses.

### Done when

New code uses one error shape and one pagination shape. Existing API clients
still work. The five example routes have consistent UI states and tests.

## 7. Phase 2: archive, download, and reader reliability

This is the first product phase because it serves the highest-value roadmap
promise and the largest number of users.

### 7.1 Durable download jobs

Implement the `jobs` table and `downloads` view/model as an append-only
migration. Preserve existing synchronous downloads for small cached files, but
route uncached, batch, conversion, and multi-format work through jobs.

Backend steps:

1. Extract export input validation from `src/routes/export.rs` into a service.
2. Create a job when a request is queued. Return `202` plus job ID for work
   that cannot finish quickly.
3. Reuse existing cache semaphores and cache paths. The worker must check the
   cache before scraping or converting.
4. Emit progress stages: queued, fetching, parsing, building, converting,
   packaging, complete, failed, cancelled.
5. Implement `GET /api/downloads/{id}`, `POST /api/downloads/{id}/cancel`,
   and a progress stream using SSE. Reconnect must replay the current state.
6. Enforce owner/admin authorization on every job read and cancellation.
7. Add cleanup for abandoned temporary files and expired job rows.

Frontend steps:

1. Add a persistent download tray visible from every page.
2. Show one row per job with format, title, progress stage, cancel, retry, and
   download action.
3. On refresh, reload active jobs rather than losing progress.
4. On SSE failure, poll with backoff and show a non-blocking stale indicator.
5. Use the same component for single, author, and series downloads.

Tests:

- two identical clicks create one idempotent job;
- a worker retry does not create duplicate files;
- cancellation stops before the next expensive stage;
- an SSE reconnect receives the current state;
- anonymous users cannot read another user's job;
- batch download reports partial failures without losing successful files.

### 7.2 Source reachability and update freshness

Implement the roadmap's P1 cookie/session flow only after a product/security
review. Store encrypted, expiring credentials; never return cookies in JSON or
logs. Make scraper attempts record source, status class, retry time, and cache
fallback used.

Then implement the P8 update watcher as a job:

- users follow a work/author/series;
- the watcher checks only due ongoing sources;
- changed chapter count/content hash creates a version record;
- followers receive one coalesced notification;
- failures back off by domain and appear in the scraper-health board.

### 7.3 Reader polish

Refactor reader state into one store containing work ID, chapter, scroll
position, font, width, theme, progress, and offline state. Add keyboard commands
with a visible help dialog, reduced-motion behavior, estimated reading time,
and a clear resume button. Keep annotations and TTS behind independent feature
flags until their storage and privacy rules are approved.

Acceptance: a reader can close the browser, reopen on another device, resume at
the saved position, work offline with a cached chapter, and understand when a
chapter is stale.

## 8. Phase 3: library and identity consolidation

Goal: make the site feel like one personal archive.

### Backend

1. Define a single `LibraryItem` projection for bookmark, shelf, list,
   reading status, progress, and last-read time.
2. Add cursor pagination and consistent filtering/sorting for bookmarks,
   history, shelves, lists, and follows.
3. Make bookmark/import operations idempotent and transactional.
4. Add a unified notification preference model: feature, channel, frequency,
   mute-until. Coalesce duplicate notifications server-side.
5. Complete one-click JSON/CSV export for bookmarks, history, ratings,
   reviews, lists, preferences, and installed extensions. Include a schema
   version and a documented import strategy.
6. Add session management and account deletion/export verification. Do not
   expose raw IPs or site credentials.

### Frontend

1. Build `/library` as the default logged-in home: continue reading, active
   downloads, recently added, saved searches, and follows.
2. Make shelves and lists use one card/list component with configurable actions.
3. Add bulk selection with keyboard support and confirmation for destructive
   changes.
4. Add a notification center with unread count, grouped events, mark-read, and
   preferences.
5. Replace direct `localStorage` token reads with the auth store everywhere.
6. Add onboarding: choose three favorite works/tags, select content preferences,
   and explain privacy and trust levels. This is the small-cohort readiness
   work, not a forced gamification tutorial.

Acceptance: a user can find every saved story, resume reading, export their
library, disable a notification category, and delete their account without
contacting an operator.

## 9. Phase 4: discovery and personalization

Goal: make search and recommendations earn trust.

Use the latest user-consensus priorities from the roadmap to select the first
discovery improvements. The implementation should produce a better discovery
experience; it should not expose the prioritization method to readers. Do not
promote a technically interesting feature over a higher-consensus user need.

### 9.1 Search as the primary discovery surface

1. Split the search implementation into parser, query planner, repository,
   ranking, and presentation modules.
2. Define a typed `SearchQuery` AST. Parse simple, guided, and power syntax into
   the same AST; report field-level parse errors instead of silently dropping
   terms.
3. Add a `SearchExplanation` to results: matched tags, filters, sort, and
   exclusions. Hide technical details behind a details disclosure.
4. Add stable cursor pagination and saved-query serialization.
5. Use zero-result and search-to-export analytics to prioritize tag aliases,
   scraper coverage, and ingestion. Never use raw query text in public reports.
6. Add semantic search only as a separate opt-in mode. It must show that it is
   semantic, preserve normal filters, and fall back to lexical search if Ollama
   is unavailable.

### 9.2 Recommendations

1. Keep the strategy registry, but define one `Recommendation` type with:
   work/entity ID, score, strategy, reason code, reason parameters, and
   generated-at timestamp.
2. Make logged-in personalized recommendations the default only after the
   shadow-run comparison meets a written threshold for engagement and complaint
   rate. Logged-out users retain generic recommendations.
3. Add reason templates such as “because you bookmarked X”, “similar tags”,
   “readers with this list also saved”, and “new from an author you follow”.
4. Add feedback actions: not interested, already read, more like this, less
   like this. Store them as durable signals and make them reversible.
5. Add a taste profile page showing broad tag/fandom/length tendencies, with a
   privacy control and no sensitive inference.
6. Add circuit breakers and timeouts for external or optional strategies. A
   failing strategy must fall back to co-occurrence or lexical results.
7. Add per-surface recipes only after the default recommendation path is stable.

### 9.3 Roadmap presentation

Treat the roadmap as a product feedback loop:

- feedback comparisons are short and accessible;
- every roadmap item has status, recent change, expected user benefit, and
  feedback link;
- maintainers use the consensus result and confidence to choose work;
- controversial items are reviewed rather than treated as automatically low
  priority;
- shipped items remain searchable with a changelog and outcome metrics;
- users can withdraw or amend feedback where the policy permits;
- the public product emphasizes outcomes and benefits, not Elo mechanics.

Acceptance: a user can search without learning syntax, understand why a result
or recommendation appeared, correct a bad recommendation, and vote on a
roadmap item knowing how votes are used.

## 10. Phase 5: community and governance consolidation

Goal: make community participation safe, welcoming, and consistent.

### 10.1 Shared content actions

Define a common moderation target abstraction for comments, reviews, requests,
forum posts, translations, uploads, and metadata proposals. Keep the existing
feature-specific tables, but route reports, status, decisions, and audit events
through the shared governance service.

Rules:

- trust level controls participation and community moderation capability;
- admin authority remains separate and explicit;
- never silently delete a user's content without an audit record;
- use soft hide, appeal/review, and clear reasons;
- public users see the public decision record, not private evidence;
- automated classification is advisory or grace-period based unless a policy
  explicitly allows immediate hiding.

### 10.2 Forum refactor

Do not rewrite forum behavior in one file. Extract `src/routes/forum.rs` in
this order:

1. read-only category/topic/post queries;
2. topic and post write services;
3. read state and follows;
4. reactions and notifications;
5. moderation and metamoderation;
6. admin pin/lock/ban operations.

After each extraction, keep the same route paths and response shapes, add
contract tests, and delete only code proven unused. `forum-core` remains the
pure domain engine; SQL and Axum belong in the application crate.

### 10.3 User experience

Add a clear community rules page, report explanation, moderation status,
appeal path, block/mute controls, and notification preferences. Use progressive
disclosure: normal readers should not have to understand moderation points,
Elo, or trust metrics to read a forum topic.

Acceptance: a new user can post safely, report a problem, see what happened to
the report, mute unwanted content, and understand why a feature is unavailable.

## 11. Phase 6: accessible, responsive, and international UI

Run a full WCAG-oriented audit after the core flows are stable.

### Checklist

- every interactive element is keyboard reachable and has a visible focus ring;
- dialogs trap focus and return it to the opener;
- live updates use polite/assertive announcements appropriately;
- icon-only controls have accessible names;
- color is never the only status signal;
- contrast passes normal and high-contrast themes;
- reader supports browser zoom to 200% without hiding content;
- reduced-motion preference disables nonessential animation;
- touch targets are large enough on mobile;
- tables have headers and a mobile alternative;
- RTL layout is tested with Arabic/Hebrew fixture text;
- all six dictionaries have the same key set, with a test enforcing it;
- dates, numbers, word counts, and reading times use locale formatting;
- adult-content gates and warnings are announced, not only colored.

Add Playwright journeys for anonymous search/download, logged-in library/reader,
roadmap vote, forum report, mobile viewport, keyboard-only navigation, and RTL.

## 12. Phase 7: operator maturity and performance

### Observability

Add structured spans and metrics for:

- request count, latency, status, and request ID;
- scrape attempts by domain and result class;
- export cache hit/miss and conversion time;
- job queue depth, age, retries, and failures;
- search zero-result rate and recommendation fallback rate;
- moderation queue age;
- PostgreSQL slow queries and pool exhaustion;
- Redis errors and limiter fallbacks;
- Ollama latency, queueing, model load, and memory pressure.

Metrics must be aggregate and zero-PII. Add `/api/health/ready` for dependency
readiness and retain a cheap liveness endpoint.

### Backups and upgrades

Document and test:

- PostgreSQL backup and restore;
- body-cache and export-cache backup policy;
- Redis loss behavior;
- migration rollback/forward procedure;
- secret rotation;
- restoring a staging instance from backup;
- deployment smoke test and rollback to the previous binary.

Add a monthly restore drill. A backup that has never been restored is not an
acceptance criterion.

### Performance budgets

Set and measure budgets rather than guessing:

- cached page/API response p95;
- first reader content response;
- search p95 for common and complex queries;
- export queue wait;
- frontend JavaScript size and route load;
- database query count on home, search, reader, and forum pages.

Use explain plans and indexes before adding caches. Add load tests for the four
critical journeys with a small realistic fixture dataset.

## 13. Phase 8: remove duplication and retire obsolete systems

Only start this phase after the target paths are in production and telemetry
shows no remaining callers.

1. Inventory every route/API client marked legacy, deprecated, or duplicated.
2. Add a deprecation response/header and a removal date.
3. Search the repository and frontend for callers; include bot, CLI, OPDS,
   ActivityPub, scripts, and docs.
4. Remove one compatibility layer per PR and run API-walk plus integration tests.
5. Remove dead frontend components, old auth reads, unused quest/XP paths, and
   obsolete docs only when the roadmap/status document confirms the replacement.
6. Keep database migrations append-only. Retire data with an explicit migration
   and backup plan; never edit an applied migration.
7. Update README, API docs, contributor docs, and `docs/brainstorm-02...` in the
   same PR as the final removal.

The goal is not minimum file count. The goal is one obvious implementation per
behavior.

## 14. Delivery order and pull-request rules

Use this order:

1. baseline and architecture map;
2. response/pagination/error contracts;
3. durable download jobs and reader reliability;
4. library and identity consolidation;
5. search/recommendation explanations and feedback;
6. governance/forum extraction;
7. accessibility/i18n audit;
8. operations, backup, and performance;
9. compatibility removal.

Each PR should:

- change one feature boundary;
- include a short user-visible outcome;
- name the roadmap item and the current user-consensus evidence;
- include migration and rollback notes;
- add or update unit, API, and UI tests;
- avoid drive-by formatting of unrelated files;
- include screenshots or a Playwright result for UI changes;
- leave the application buildable.

Suggested commit prefixes: `refactor:`, `feat:`, `fix:`, `test:`, `docs:`,
`chore:`. Commit after a coherent boundary, not after every edited line.

## 15. Definition of mature and polished

FicNexus reaches the target when all of the following are true:

- The top five user journeys have documented, tested success/error/empty/offline
  behavior: search, read, download, save, and return to the library.
- No critical journey depends on a single optional Ollama request or an
  unbounded HTTP request.
- All mutating endpoints have consistent auth, validation, rate limiting,
  idempotency where appropriate, audit behavior, and error responses.
- A fresh developer can locate the feature service, repository, route, model,
  frontend components, tests, and documentation from one index.
- Search and recommendations explain themselves and accept negative feedback.
- Trust, moderation, reports, appeals, and public audit records use one coherent
  governance vocabulary.
- Keyboard, mobile, screen-reader, high-contrast, reduced-motion, and RTL checks
  pass for the critical journeys.
- The operator can observe queue health, scraper health, storage health, and
  dependency health; restore from backup; and roll back a deployment.
- Roadmap decisions cite feedback and the resulting consensus, and shipped
  features report whether they improved the intended user outcome.
- Every retired feature has a migration, compatibility period, documentation
  update, and verified absence of callers.

## 16. First five implementation tickets

These are intentionally small starting tickets for a junior developer:

1. Add `docs/architecture/current-map.md` by listing each route group, its
   handler file, frontend route, migration dependencies, and test file. Do not
   move code.
2. Implement `ApiError` parsing and shared `ErrorState`/`RetryButton` in the
   frontend. Convert one read-only page and add tests.
3. Add request IDs to one Axum middleware layer and one API response. Add a
   tracing test or deterministic header assertion.
4. Extract one pure pagination helper from an existing list endpoint. Preserve
   its response shape and add malformed-cursor tests.
5. Add a roadmap evidence template at `docs/roadmap/feature-evidence.md` with
   fields for Elo snapshot, feedback links, success metric, rollout flag, and
   rollback plan.

Do not begin with a broad directory rename or a database rewrite. The mature
version is reached by repeatedly making one user journey clearer and one code
boundary more explicit.