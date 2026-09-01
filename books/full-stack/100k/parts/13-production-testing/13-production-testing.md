# Part 13 — Production, Testing & Ship It

> **Part 13 of 13** — This is it. The final part. For twelve parts and
> fifty-six chapters we've built FicHub layer by layer: the Rust
> foundation, the database, the scraper that pulls fics off AO3 and FFN,
> the exporter that turns them into EPUBs, the search engine, the
> recommendation platform, the AI features, the admin and transparency
> layer, and the entire SvelteKit frontend that ties it all together.
>
> But here's the thing about code: it isn't *real* until it's tested,
> deployed, and running somewhere people can actually use it. A scraper
> that only ever runs on your laptop is a hobby. A scraper running on a
> ThinkCentre in a corner of someone's home, behind a Cloudflare tunnel,
> serving EPUBs to readers at fichub.polarisocial.xyz — that's a
> *platform*.
>
> So in this final part we cross the finish line. Chapter 57 maps the
> whole testing strategy: unit tests inside the Rust modules, DB-gated
> integration suites that need a live PostgreSQL, and the vitest suite
> that locks down the frontend. Chapter 58 dissects the DB-gated
> harness itself — the global `Mutex`, the `#[ignore]` attribute, and
> the unique-seed discipline that lets ~110 tests share one real
> database without ever colliding. Chapter 59 takes us to production:
> the systemd units, the ThinkCentre, migrations on boot, and the
> body-cache drive that holds the most valuable data in the whole
> project. Chapter 60 is the developer workflow — git worktrees and the
> NFS quirk that taught us more about git internals than any tutorial
> ever would. Chapter 61 reads the roadmap: what's shipped, what's in
> flight, what comes next. And Chapter 62 — the last chapter of the
> last part — is a celebration, a recap of the whole journey, and a
> look at the finish line we just crossed.
>
> Let's ship it.

---


---

## Chapter 57 — The Testing Strategy: Unit, DB-Gated Integration, and Vitest

Let's start with a confession: when we began this project back in Part 1,
there were exactly two kinds of tests in FicHub — a handful of unit tests
tucked inside the route modules, and one integration file. Nothing was
automated, nothing was gated, and nothing stopped a well-meaning refactor
from silently breaking the search endpoint at 2 a.m. If that sounds
familiar, it's because *every* project starts this way.

Twelve parts later, FicHub has a testing story that would make a CI
engineer nod approvingly:

- **575+ unit tests** in the library (`cargo test` — pure Rust, no
  services needed)
- **~110 DB-gated integration tests** across 30+ suites that run against
  a *live* PostgreSQL and Redis, serialized by mutexes and skipped by
  default (`#[ignore]`)
- **438+ frontend tests** in 66 files (vitest + jsdom + Testing Library)
- **15 E2E tests** that boot the real SPA, click through the real layout,
  and visit every routed page
- **215+ scraper-crate adapter tests** — one suite per site adapter
- Plus a QA harness (`qa/run.js`, `qa/api-walk.js`) that crawls the whole
  app with Playwright and audits every API route

That's over **1,300 tests** guarding a codebase that started with none.
And the beautiful part? None of this required buying into a heavyweight
testing culture. It grew organically, one regression test at a time, in
exactly the same order it grows for every project: panic, fix, lock it
in.

### The pyramid, inverted for reality

Textbooks show you a testing pyramid: lots of unit tests at the bottom,
fewer integration tests in the middle, and a handful of E2E tests at the
top. FicHub's reality is close to that shape, but with a twist you'll
see in real projects constantly: the *unit* layer and the *DB-gated*
layer are both huge, because they test different things and run on
different schedules.

Let's map the three layers we're about to study in detail, because
Chapter 58 is going to dive deep into the middle one.

**Layer 1 — Unit tests: fast, everywhere, zero services.** These live
next to the code they test, in `#[cfg(test)] mod tests` blocks inside
the source files. They test pure functions: parsing, formatting,
serialization, decision logic. The health handler's tests in
`src/routes/health.rs` are a perfect specimen:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serializes_correctly() {
        let response = HealthResponse {
            status: "ok".to_string(),
            db: true,
            redis: true,
            version: "0.2.0".to_string(),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["db"], true);
        assert_eq!(json["redis"], true);
        assert_eq!(json["version"], "0.2.0");
    }
```

Note what this test *doesn't* do: it doesn't start a server, doesn't
touch a database, doesn't open a socket. It constructs a plain struct,
serializes it, and checks the JSON. That's the whole point of the unit
layer — it runs in milliseconds, so it runs *every single time you
compile*. These are the tests that catch typos, wrong field names, and
logic inversions before you ever get near a service.

> 💡 **Key Concept — The unit test contract.** A unit test should be
> deterministic, service-free, and fast. If your unit test needs a
> database, a network connection, or the phase of the moon, it's not a
> unit test — it's an integration test wearing a unit test's clothes,
> and it will make your `cargo test` slow and flaky. Keep the unit layer
> pure, and you'll actually *run* it.

**Layer 2 — DB-gated integration tests: real everything, on demand.**
These live in `tests/`, one file per feature area: `search_api.rs`,
`admin_api.rs`, `tags_api.rs`, `health_api.rs`, `integration.rs` — 44
files at last count. They build the *real* Axum router, wire up the
*real* `AppState` with real database pools and real Redis connections,
and fire actual HTTP requests through `tower::ServiceExt::oneshot`.
They're the closest thing to "run the server and poke it" without
actually running a server.

But they need a live PostgreSQL, which means they can't run on every
developer's machine and they definitely can't run in a bare CI container.
So they're marked `#[ignore]` — skipped by default — and run explicitly
when you want them:

```text
cargo test --test search_api -- --include-ignored --test-threads=1
```

We'll spend all of Chapter 58 inside this layer, because it's the most
clever (and most instructive) part of the whole test story.

**Layer 3 — Frontend tests: vitest, jsdom, and the mock-fetch pattern.**
The SvelteKit frontend has its own testing world, configured in
`frontend/vite.config.ts`. The unit layer tests the API client and the
stores with a mocked `fetch`; the page layer renders actual Svelte
components into a fake DOM (jsdom) and asserts on what a user would
see. The coverage gate lives here too — a hard threshold that fails the
build if the logic layer's coverage drops.

### One bug, three tests: the `redis:false` story

The best way to understand why FicHub's test stack looks the way it
does is to watch one real bug pass through all three layers. Open
`tests/health_api.rs` and read the header comment — it's a miniature
post-mortem:

```rust
//! Integration tests for `GET /api/health` — regression coverage for the
//! `redis:false` bug.
//!
//! The bug: `/api/health` used to PING Redis over `state.redis`, the shared
//! multiplexed connection the bookmark-import worker blocks on with an
//! unbounded BRPOP. A PING queued behind that BRPOP never completes in time,
//! so health reported `redis:false` even when Redis was perfectly healthy.
//!
//! The fix: health now uses a *dedicated* connection (`state.health_redis`)
//! with a short 2s PING timeout, so health is accurate and never hangs.
```

Let me unpack that, because it's a *fantastic* bug. FicHub runs a
bookmark-import worker that watches a Redis list with a blocking `BRPOP`
— a command that sits and waits indefinitely for a new item. The health
handler, meanwhile, checked Redis by issuing a `PING` on the *shared*
multiplexed connection. Redis processes commands on a multiplexed
connection in order — so a `PING` queued behind an unbounded `BRPOP`
waits forever, times out, and health reports `redis:false`. The site
was fine! Redis was fine! The dashboard just said otherwise, every time
the worker happened to be parked on that list.

The fix has two halves, and you can see both in `src/config.rs`'s
sibling, `tests/health_api.rs`'s `app()` helper:

```rust
let state = Arc::new(AppState {
    config: config.clone(),
    db: db.clone(),
    redis: redis.clone(),
    health_redis: redis_client
        .get_multiplexed_async_connection()
        .await
        .expect("health redis conn"),
    // ...
```

A *dedicated* `health_redis` connection, so the health PING never queues
behind the worker's BRPOP. And the test suite encodes the regression in
the strongest way FicHub knows — a DB-gated test that builds the real
state, hits the real handler, and asserts `redis: true`:

```rust
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_reports_ok_with_live_services() {
    let _guard = db_guard();
    let app = app().await;

    let (status, json) = get_health(&app, "/api/health").await;
    assert_eq!(status, StatusCode::OK, "expected 200, got {status}: {json}");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["db"], true);
    assert_eq!(json["redis"], true, "live Redis must report redis:true");
}
```

And because the *second* failure mode — the hang — is just as dangerous
as the wrong answer, there's a sibling test that hands the handler a
*dead* health connection and asserts it still answers within 10 seconds
with a 503:

```rust
// Bound the whole request so a regression (PING hanging) fails this test
// instead of hanging CI.
let (status, json) = tokio::time::timeout(
    std::time::Duration::from_secs(10),
    get_health(&app, "/api/health"),
)
.await
.expect("health handler must answer within 10s even when Redis is down");

assert_eq!(
    status,
    StatusCode::SERVICE_UNAVAILABLE,
    "expected 503, got {status}: {json}"
);
assert_eq!(json["status"], "degraded");
assert_eq!(json["db"], true);
assert_eq!(json["redis"], false);
```

Look at the craftsmanship here. This test doesn't just check "does
health fail when Redis is down" — it builds a *fake Redis server* on a
random local port that answers the connection handshake, then drops the
socket mid-command, producing the exact failure mode of a Redis that
goes away. And it wraps the whole request in `tokio::time::timeout` so
a regression that makes the handler hang fails the test instead of
hanging your terminal. That's what years of production scar tissue
teaches you to write.

> ⚠️ **Watch Out — The `unwrap_or_else(|e| e.into_inner())` pattern.**
> You'll see this everywhere in the DB-gated suites:
>
> ```rust
> fn db_guard() -> std::sync::MutexGuard<'static, ()> {
>     db_lock().lock().unwrap_or_else(|e| e.into_inner())
> }
> ```
>
> A normal `lock().unwrap()` would panic if a previous test panicked
> *while holding the lock*, poisoning it — and then every subsequent
> test fails with `PoisonError` instead of running. Because the DB-gated
> tests clean up after themselves (more on that in Chapter 58), the
> underlying data is still consistent after a panic, so recovering the
> guard with `into_inner()` is safe and keeps the whole suite alive.
> This is a subtle, professional touch — don't skip it when you copy
> the pattern.

### The vitest world: mock fetch, render, assert

Now hop to the frontend. Open `frontend/src/lib/api/kindle.test.ts` —
all 57 lines of it. This is the canonical FicHub frontend unit test
pattern, and it's *small enough to read in one gulp*:

```ts
import { describe, it, expect, vi, beforeEach } from 'vitest';

// Test the Send-to-Kindle client by mocking fetch + localStorage.
const mockFetch = vi.fn();
globalThis.fetch = mockFetch as unknown as typeof fetch;

beforeEach(() => {
  mockFetch.mockReset();
  localStorage.clear();
});

async function importKindle() {
  return await import('./kindle');
}

describe('sendToKindle', () => {
  it('throws when the user has no token', async () => {
    const { sendToKindle } = await importKindle();
    await expect(sendToKindle({ url: 'https://ao3.org/works/1' })).rejects.toThrow(
      'You must be logged in to log in.',
    );
    expect(mockFetch).not.toHaveBeenCalled();
  });
```

Three moves, every single test:

1. **Mock `fetch`** — `globalThis.fetch = mockFetch` replaces the real
   network with a function you control. No server, no network, no
   flakiness.
2. **Queue the response** — `mockFetch.mockResolvedValue({ ok: true,
   json: async () => ... })` tells the mock what the server "would
   have" said.
3. **Assert the behavior** — check what the client did with it: the URL
   it built, the headers it set, the error it threw.

The most instructive assertions are the *negative* ones. `expect(mockFetch)
.not.toHaveBeenCalled()` — the client must not even *try* the network
when the user isn't logged in. And this one, from the same file:

```ts
it('POSTs url with the Bearer token and parses the response', async () => {
    localStorage.setItem('fichub_token', 'kindle-jwt');
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({ err: 0, url_id: 'abc', to: 'me@kindle.com' }),
    });

    const { sendToKindle } = await importKindle();
    const res = await sendToKindle({ url: 'https://ao3.org/works/1' });

    expect(res.url_id).toBe('abc');
    const [url, init] = mockFetch.mock.calls[0];
    expect(url).toBe('/api/send-to-kindle');
    expect(init.method).toBe('POST');
    expect(init.headers.Authorization).toBe('Bearer kindle-jwt');
    expect(JSON.parse(init.body).url).toBe('https://ao3.org/works/1');
});
```

This test pins down the *wire format* — the exact URL, the exact HTTP
method, the exact auth header, the exact JSON body. If someone renames
the endpoint, drops the Bearer token, or switches the body key from
`url` to `source`, this test fails. That's a contract test, and contract
tests are the highest-leverage tests a frontend can have, because they
freeze the API shape the backend depends on.

Then there's the page layer. `frontend/src/routes/roadmap/page.test.ts`
tests an entire Svelte page against a mocked fetch — rendering the real
`+page.svelte` into jsdom and asserting on what's visible:

```ts
it('renders the 2x2 arena from the API', async () => {
    mockFetch
      .mockResolvedValueOnce({ ok: true, json: async () => ({ err: 0, clusters: [] }) }) // auth.init()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          err: 0,
          clusters: [
            { id: 1, text: 'Dark mode for night reading', matches_played: 14, elo_rating: 1600 },
            // ...
          ],
        }),
      });

    const { default: RoadmapPage } = await import('./+page.svelte');
    render(RoadmapPage);

    await waitFor(() => {
      expect(screen.getByText('Dark mode for night reading')).toBeTruthy();
    });
});
```

Two details are worth pausing on. First, `mockResolvedValueOnce` — the
mock is *queued*, one response per fetch call, in order. The first fetch
is `auth.init()` (every page calls it), the second is the roadmap API.
The comment on that line isn't decoration; it's a map of the page's
fetch sequence, and if the page starts fetching something else, the
queue shifts and the test fails — alerting you to the change. Second,
`waitFor` — because the page renders asynchronously (fetch → state →
DOM), you can't assert synchronously; you wait for the text to appear.
Testing Library's `waitFor` polls until the assertion passes or times
out.

> 🧪 **Try It Yourself — Write a contract test for an API client.**
> Pick any function in `frontend/src/lib/api/` — say
> `fetchExport` from `client.ts` (there are existing tests for it in
> `client.test.ts` to compare against, but write yours from scratch
> first). Mock `fetch`, then write three tests: one for the happy path
> (assert the returned object), one for the error path (assert it
> throws `ApiError` when `ok: false`), and one for the wire format
> (assert the exact URL and query string built). Then run
> `cd frontend && npx vitest run src/lib/api/client.test.ts` and watch
> your tests pass. The key insight: once you've written a contract
> test for one client, you've written the template for all of them.

### The coverage gate: thresholds that mean something

Every frontend test file in FicHub is just the *mechanism*. The *policy*
is the coverage gate in `frontend/vite.config.ts`. Read the comments
there — they're an operations log:

```ts
coverage: {
  provider: 'v8',
  reporter: ['text', 'json-summary', 'html'],
  reportsDirectory: 'coverage',
  include: ['src/lib/**'],
  exclude: [
    'src/lib/test/**',
    '**/*.test.{ts,svelte}',
    'src/test-setup.ts',
  ],
  thresholds: {
    // Baseline 2026-08-08: 70.27% lines / 58.08% funcs on src/lib.
    // Coverage wave (2026-08-09): 90.3% lines / 82.66% funcs / 81.31%
    // branches — set ~4-5pts under the achieved baseline so the gate is
    // a regression guard today, with headroom toward the 80/70 target
    // (now exceeded; the guard keeps future refactors honest).
    lines: 85,
    functions: 78,
    branches: 76,
    statements: 85,
  },
},
```

There's so much wisdom in these comments. The thresholds aren't plucked
from a "90% coverage" best-practices blog post — they're set *just under
the measured baseline*, so the gate protects against regressions without
demanding a heroic (and usually fake) 100%. And note the `include`:
`src/lib/**` — the logic layer: API clients, stores, utilities. The
comment explains why the Svelte page markup is excluded: full page
coverage is "noisy and drags the gate below any useful bar." The team
chose to gate what's meaningful, and documented the choice so nobody
"fixes" it later.

> 💡 **Key Concept — Coverage gates protect baselines, not ideals.**
> A coverage threshold that's set above what you can realistically
> achieve gets disabled within a week. A threshold set a few points
> *under* your current baseline is a regression guard that runs forever:
> refactor that deletes tests → coverage drops → CI fails → you notice
> immediately. Set the gate to protect what you have, then ratchet it
> up as the suite grows. FicHub's comment history shows exactly this
> ratchet: 70% → 85% lines over the course of a "coverage wave".

### The QA harness: a fourth layer, from the field

Before we move on, one more piece of the strategy: the QA harness in
`qa/`. These Node scripts run against the *live* site or a local server
and do what unit and integration tests can't: click through the real
UI, watch the console for errors, and audit the entire API surface.
From `docs/AGENTS.md`:

```text
- `qa/run.js` — full deterministic QA: API smoke, link crawl, OPDS checks,
  browser journeys (Playwright, console/network capture), backend log scan
  (journalctl). Run after any change: `node qa/run.js`.
- `qa/api-walk.js` — scriptable API-surface audit. Parses every route from
  `src/server.rs`, hits each endpoint, asserts auth gates (anonymous 4xx
  for auth/admin/non-GET), flags 5xx + stub-like responses. ... 97/97 green.
```

The `api-walk.js` is a small masterpiece of mechanical honesty — instead
of maintaining a hand-written list of endpoints (which would drift), it
*parses the router source* to discover routes:

```js
function discoverRoutes() {
  const src = fs.readFileSync(path.join(ROOT, 'src', 'server.rs'), 'utf8');
  const lines = src.split('\n');
  const out = [];
  for (const ln of lines) {
    const m = ln.match(/\.route\(\s*"(\/api\/[^"]+)"\s*,\s*(\w+)\(/);
    if (!m) continue;
    const route = m[1];
    const handler = m[2];
    // ...
```

The route list can never go stale, because it's generated from the same
source of truth the server uses. That walk has caught real production
bugs — two 500s that only appeared for edge-case inputs (`fetch_one`
on a missing user, `SUM` over an empty table), both fixed and both now
permanently guarded.

> ⚠️ **Watch Out — `fetch_one` vs `fetch_optional`.** The single most
> common 500 in SQLx codebases is calling `fetch_one` on a query that
> can legitimately return zero rows. `fetch_one` panics/errors on no
> rows; `fetch_optional` returns `None`. The api-walk caught exactly
> this in `/api/users/{id}/reading-stats` (unknown user → 500) and the
> fix was one line: switch to `fetch_optional` and return zeros. If
> you take one habit from this chapter, make it: *ask yourself whether
> the query can return zero rows before you write `fetch_one`.*

### What we just built

That's the strategy in one view: fast pure unit tests everywhere,
DB-gated integration suites that prove the real stack works and run on
demand, a vitest world with contract tests and page tests and a
regression-guard coverage gate, and a QA harness that audits the live
app like a tireless intern. Four layers, each with a job, each with a
schedule.

One of those layers deserves a chapter of its own, because it solves a
problem most juniors never even see coming: how do you run a hundred
integration tests against *one* real database without them smashing
into each other? That's the DB-gated harness — the `Mutex`, the
`#[ignore]`, the unique seeds — and it's Chapter 58.

---

## Chapter 58 — The DB-Gated Test Harness: Mutex, `#[ignore]`, and Unique Seeds

Here's a problem you won't find in most tutorials, because most tutorials
never ship. Your integration tests need a *real* database — not a mock,
not an in-memory fake, a real PostgreSQL with real indexes and real
constraints, because you want to test the actual SQL your queries run.
But you only have *one* database, and you want to run dozens of test
suites against it. What happens when `search_api` seeds a fic called
"The Testing of the Rings" at the same moment `tags_api` is counting
tags? Chaos. FicHub's answer to that chaos is the subject of this
chapter — and it's a three-part harness you'll see repeated across all
44 files in `tests/`.

The three parts are:

1. **The global `Mutex`** — every DB-gated test acquires the same lock
   before touching the database, so tests never run concurrently.
2. **`#[ignore]`** — the tests are skipped by default and run only when
   you explicitly ask, so the normal `cargo test` stays fast and
   service-free.
3. **Unique seeds + self-cleanup** — every test creates rows with names
   nobody else could create, and deletes exactly what it created.

Let's take them one at a time, reading real code from `tests/`.

### Part 1: The Mutex — one database, one test at a time

Open `tests/integration.rs` and look at the header — it tells the whole
story before the code starts:

```rust
//! Integration tests for the fichub backend stack.
//!
//! This file contains three test modules:
//! 1. `db_tests` — round-trip database query tests (require live PostgreSQL)
//! 2. `api_tests` — Axum router smoke tests with mock handlers
//! 3. `export_tests` — pure-function tests for slug, info-string and meta-json
//!
//! Database tests are serialised via a global Mutex and marked `#[ignore]` so they
//! are skipped unless the user explicitly runs `cargo test -- --include-ignored`.
//! Each database test creates its own temporary PostgreSQL schema for isolation.
```

Two isolation tricks in one header: the mutex *and* the per-test schema
(we'll get to schemas in a minute). Here's the mutex itself:

```rust
mod db_tests {
    use super::*;
    use chrono::Utc;
    use sqlx::AssertSqlSafe;
    use sqlx::postgres::PgPoolOptions;
    use sqlx::PgPool;

    /// Global mutex that serialises ALL database tests so they never step on
    /// each other (each test creates / drops its own schema, but concurrent
    /// migration runs can clash on the `sqlx_migrations` table).
    static DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    fn db_lock() -> &'static Mutex<()> {
        DB_LOCK.get_or_init(|| Mutex::new(()))
    }
```

Two small Rust details are doing real work here. `OnceLock` gives you a
`static` that initializes exactly once — the first call to `db_lock()`
creates the `Mutex`, every subsequent call gets the same one. And the
test acquires it like this:

```rust
let _guard = db_guard();
```

— holding the guard for the *entire* test. Because every DB-gated test
across every file calls `db_guard()` before doing anything, and Rust's
`Mutex` gives exclusive access, the net effect is: **at most one
DB-gated test runs at any moment, anywhere in the test binary.** No two
tests can interleave their seeds, migrations, or queries.

Why is this necessary? The header comment gives the real reason: each
test creates and drops its *own schema*, but they all share the
`sqlx_migrations` table, and concurrent migration runs clash on it.
Even without migrations, seeding and asserting on shared tables
concurrently is a race you'll lose eventually. Serializing is the honest
fix: these tests don't need to be fast, they need to be *right*.

> 💡 **Key Concept — Why a Mutex and not per-test databases?**
> The cleanest theoretical solution is a fresh database per test —
> and FicHub's `db_tests` module in `integration.rs` actually does the
> next-best thing, creating a fresh *schema* per test (more below).
> But creating a database is slow (seconds each) and requires
> privileges the test user may not have; creating a schema is fast and
> cheap. The mutex serializes the remaining shared state — the
> migration table, the connection budget, the Redis keyspace. This is
> the classic engineering trade: the *simplest* solution that is
> *correct*. Serialized tests take longer; flaky tests take forever,
> because you have to debug them.

### Part 2: `#[ignore]` — the opt-in gate

Now look at how a test opts into the mutex. From `tests/integration.rs`:

```rust
#[ignore]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn health_reports_ok_with_live_services() {
```

Wait — that's from `health_api.rs`, but the pattern is identical
everywhere. The `#[ignore]` attribute is the second pillar of the
harness. It does exactly what it says: **`cargo test` skips ignored
tests by default**, printing a reassuring "N ignored" line instead of
running them. They only run when you pass `--include-ignored`:

```text
set -a; . ./.env; set +a
cargo test --test search_api -- --include-ignored --test-threads=1
```

Let's decode that command line, because it's the daily ritual of
anyone working on FicHub's backend:

- `set -a; . ./.env; set +a` — source the environment file (`.env` is
  gitignored, remember) and export every variable, so the tests can
  find `DATABASE_URL` and `REDIS_URL`. The `-a` flag makes `source`
  export the variables, and the trailing `set +a` turns auto-export
  back off so you don't pollute your shell.
- `cargo test --test search_api` — run only the `search_api` suite, not
  the whole 44-file integration set. In CI or a full local pass you'd
  list several `--test` flags or just run all of them.
- `-- --include-ignored` — everything after the first `--` goes to the
  *test harness*, not cargo. `--include-ignored` overrides the
  `#[ignore]` attributes.
- `--test-threads=1` — belt and suspenders: even though every test
  grabs the same mutex, running on one thread makes the serialization
  explicit and avoids any mutex-adjacent surprise.

> ⚠️ **Watch Out — The `--` separates cargo args from test args.**
> Newcomers constantly write `cargo test --include-ignored` and watch
> it do nothing (or worse, cargo errors on the unknown flag). The rule:
> cargo gets everything before `--`, the test harness gets everything
> after. `cargo test -- --include-ignored --test-threads=1`. Write it
> on a sticky note if you have to — it's a top-five cargo confusion.

Why gate them at all? Because these tests need a live PostgreSQL and
Redis running with the right credentials, and the *default* `cargo
test` must not fail for someone who just cloned the repo and hasn't
set up services yet. `#[ignore]` is Rust's built-in way to say "this
test is real, but it has prerequisites." It keeps the fast suite green
for everyone and makes the slow suite a deliberate, explicit choice.

The `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
attribute deserves a word too. A plain `#[tokio::test]` runs on the
current-thread runtime; several of FicHub's components — notably the
`RedisBucketLimiter`'s shadowban probe, which calls
`tokio::task::block_in_place` — *panic* on a single-threaded runtime.
The `admin_api.rs` header documents exactly this trap:

```rust
// NOTE: `RedisBucketLimiter`'s shadowban probe (`is_shadowbanned`) runs
// `tokio::task::block_in_place`, which panics on the current-thread runtime,
// so every test must be `#[tokio::test(flavor = "multi_thread")]`.
```

The multi-thread flavor with 2 worker threads is the standard for all
the DB-gated suites. This is the kind of detail you only learn by
hitting the panic — and then you write it down in the header comment
so the next person doesn't hit it again.

### Part 3: Unique seeds — no two tests can collide

The mutex makes tests *sequential*, but sequential isn't enough. Two
runs of the same test — or two suites that happen to share a prefix —
would collide unless each test owns its data. FicHub's discipline, from
`tests/tags_api.rs`:

```rust
//! * Self-heal: every test deletes its own seed rows (unique prefix) at the
//!   START so reruns never collide.
```

"Unique prefix" is the key phrase. Tests don't seed rows named
"test-fic" — they seed rows named like `tag_e2e_alpha_<something>` that
only that test would ever create, and they clean up at both ends:
delete at the start (so a crashed previous run doesn't collide) and
delete at the end (so the database stays tidy). The seeding helpers in
`search_api.rs` show the pattern in full:

```rust
/// Seed a fic_info row. `ON CONFLICT (id) DO NOTHING` keeps this idempotent
/// so tests are re-runnable without explicit pre-cleanup.
async fn seed_fic(pool: &sqlx::PgPool, id: &str, title: &str, description: &str, words: i64, chapters: i32, status: &str, updated_days_ago: i64) {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (id) DO NOTHING"#,
    )
    // ... binds ...
    .execute(pool)
    .await
    .expect("seed_fic failed");
}
```

`INSERT ... ON CONFLICT (id) DO NOTHING` is the idempotency trick: if
the row already exists (from a previous run that didn't clean up), the
insert silently does nothing instead of erroring. The test then runs
its assertions, and the cleanup helper removes *only* what the seed
created:

```rust
/// Remove ONLY the rows the seeding helpers create, so tests are re-runnable.
async fn cleanup(pool: &sqlx::PgPool, fics: &[&str], tags: &[&str]) {
    for id in fics {
        let _ = sqlx::query("DELETE FROM fic_tags WHERE url_id = $1")
            .bind(id)
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM fic_info WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;
    }
    // ...
```

Note the `let _ =` — the cleanup *deliberately* ignores errors. If the
rows are already gone, deleting them fails harmlessly; cleanup must
never fail the test that already passed. And because cleanup runs on
both sides (start and end), even a test killed mid-run leaves nothing
behind for the next run to trip over. This is the discipline that makes
~110 tests across 44 files share one database safely.

> 🧪 **Try It Yourself — Trace the collision that the mutex prevents.**
> Pick any two DB-gated suites, say `tests/tags_api.rs` and
> `tests/search_api.rs`. In each, find the seed helpers and note which
> tables they touch. Now imagine both suites running *without* the
> mutex: `cargo test --test tags_api --test search_api -- --include-ignored`.
> What would happen if `tags_api` lists all tags while `search_api`'s
> seed fics are mid-insert? Even if the *rows* don't collide (unique
> prefixes), what shared resource could still race? Answer: the
> `sqlx_migrations` table (if either suite runs migrations), the Redis
> keyspace (both suites use real Redis), and the connection pool
> itself. The mutex serializes all of it with one line of code. That's
> the leverage — one global lock buys correctness for every suite
> added later.

### The per-test schema: the `integration.rs` upgrade

The search/tags/admin suites seed rows directly into the *shared*
`fichub` database (carefully, with unique prefixes). The original
`db_tests` module in `integration.rs` goes one step further: each test
creates its own temporary schema, runs the *real migrations* inside it,
and drops it at the end. Read the `TestDb::new()` helper:

```rust
struct TestDb {
    pool: PgPool,
    schema: String,
}

impl TestDb {
    /// Connect to `DATABASE_URL`, create a unique schema, run migrations.
    async fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for db tests");

        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("failed to connect to test database");

        // Unique schema name per invocation (PID + monotonic ns helps
        // avoid collisions when tests are run sequentially).
        let schema = format!(
            "test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        // Create the schema and set search_path.
        sqlx::raw_sql(AssertSqlSafe(format!(
            "CREATE SCHEMA IF NOT EXISTS \"{}\"",
            schema
        )))
        .execute(&pool)
        .await
        .expect("failed to create test schema");

        sqlx::raw_sql(AssertSqlSafe(format!(
            "SET search_path TO \"{}\", public",
            schema
        )))
        .execute(&pool)
        .await
        .expect("failed to set search_path");
```

The schema name encodes the recipe for uniqueness: `test_{PID}_{nanos}`.
PID alone isn't enough (two suites could run in the same process), and
nanos alone isn't enough (two processes could collide on the same
nanosecond — astronomically unlikely but free to defend against).
Together, they're unique across processes and across sequential runs.

Then the money shot — the test runs the *project's actual migrations*
against its fresh schema:

```rust
        // Run migrations from the project's migration directory.
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let migrations_path = manifest_dir.join("migrations");
        let migrator = sqlx::migrate::Migrator::new(migrations_path)
            .await
            .expect("failed to load migrations");
        migrator
            .run(&pool)
            .await
            .expect("failed to run migrations");

        TestDb { pool, schema }
```

That's a complete database lifecycle in ~40 lines: connect, create
schema, run all 38 migrations, return a handle. And the teardown is
just as clean:

```rust
    /// Drop the test schema entirely (cleanup).
    async fn cleanup(&self) {
        sqlx::raw_sql(AssertSqlSafe(format!(
            "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
            self.schema
        )))
        .execute(&self.pool)
        .await
        .unwrap_or_else(|e| panic!("failed to drop schema {}: {e}", self.schema));
    }
```

One `DROP SCHEMA ... CASCADE` and the entire test universe evaporates.
No cleanup loops, no leftover rows, no drift between what the test
sees and what a fresh database would look like. This is the gold
standard for integration testing: **your test exercises the exact
schema your production code creates, because it runs the exact same
migrations.**

> 💡 **Key Concept — Test against your migrations, not against your
> understanding of them.** The moment your test schema is built by
> hand ("let me just create the tables the test needs"), it will drift
> from production — a column added in migration 27 won't exist in your
> hand-rolled test schema, and your tests will pass while production
> breaks. Running `Migrator::new(...).run(&pool)` in every test setup
> means the migrations themselves are tested with every run, and the
> schema is always exactly what production would have. FicHub's
> `TestDb` does this; so should your projects.

And `AssertSqlSafe` deserves a footnote — it's sqlx's way of saying
"yes, I know this SQL string has a variable in it; I've constructed it
from a trusted value (a schema name we generated), so it's safe." It
silences the compile-time check that would otherwise reject dynamic
SQL, while keeping the safety *visible* — you have to consciously
assert that your interpolation is safe, which is exactly the right
nudge.

### The full ritual, and why it matters

Put it together and the DB-gated ritual looks like this:

```text
# one-time, per shell: load the real environment
set -a; . ./.env; set +a

# one suite, fully deterministic
cargo test --test search_api -- --include-ignored --test-threads=1

# the whole DB-gated army
cargo test --test integration --test search_api --test admin_api --test tags_api \
  -- --include-ignored --test-threads=1
```

And there's a documented quirk from the field, from `docs/AGENTS.md`:

```text
- DB-gated test suites each hold their OWN mutex — combined runs
  (`cargo test --test a --test b`) can transiently flake from cross-suite
  seed pollution; run suites individually (`--test-threads=1`) for
  deterministic results.
```

Each suite has its *own* `DB_LOCK` (they're separate statics in
separate files, so each test binary gets its own mutex — cross-binary
locking isn't possible in Rust's test model). Within one binary, the
mutex is airtight. Across binaries, you rely on `--test-threads=1`
plus the unique-prefix discipline to stay safe. Knowing *which* lock
protects *what* is the difference between a harness that works and a
harness that flaky-test-fails at 2 a.m.

> ⚠️ **Watch Out — Poisoning and the `db_guard` idiom.** We saw this
> in Chapter 57, but it's worth the second look here because it's part
> of the harness: if a test panics while holding the mutex, Rust marks
> the mutex poisoned, and every later `lock()` returns an error.
> FicHub's `db_guard()` swallows the poison with `into_inner()`:
>
> ```rust
> fn db_guard() -> std::sync::MutexGuard<'static, ()> {
>     db_lock().lock().unwrap_or_else(|e| e.into_inner())
> }
> ```
>
> This is safe *because* of the cleanup discipline: a panicked test has
> already deleted its own rows (or will on the next run's start-of-test
> cleanup), so the database state is still consistent and the next test
> can proceed. The three pillars of the harness support each other:
> the mutex serializes, the seeds isolate, and the poison recovery keeps
> one failure from cascading into twenty.

### What we just built

The DB-gated harness is three small Rust features — `Mutex`, `OnceLock`,
`#[ignore]` — plus one discipline (unique seeds, self-cleanup) and one
recipe (fresh schema + real migrations per test). None of it is exotic;
all of it is *thoughtful*. The header comments in each test file read
like a shared memory of every bug that shaped them: the BRPOP health
hang, the `block_in_place` panic, the cross-suite flakes. That's the
real lesson of this chapter: **a test harness is a living document of
your project's scars.** Write down why things are the way they are,
and the next person — possibly you, six months later — won't have to
re-learn the scars the hard way.

With the tests in place, the code is *trustworthy*. Now it's time to
put it on a real machine. Chapter 59 is the deployment story: the
ThinkCentre, the systemd units, migrations on boot, and the drive that
holds the site's most precious data.

---

## Chapter 59 — Deployment: The ThinkCentre, systemd, and Migrations on Boot

Every line of code you've written in this book has been leading to one
moment: the moment a reader in some other city types a URL into their
browser and gets served an EPUB of their favorite fic by *your*
software. That moment is deployment. And FicHub's deployment story is
deliciously unglamorous — no Kubernetes, no containers, no cloud. One
small Lenovo ThinkCentre M720q sitting somewhere in the house, running
Linux Mint, with the repo on an NFS pool and a Cloudflare tunnel
pointing a real domain at it.

If you're a junior developer who's only ever seen `docker-compose up`
or "deploy to Vercel" tutorials, this chapter might surprise you. This
is what a *self-hosted* production deployment actually looks like. And
the handover document in the repo — `docs/deployment-handover.md` —
is the single best production-operations document I've ever read in a
personal project. It's a chapter of this book all by itself, so let's
read it together.

### The machine and the layout

The first section of the handover doc is the machine layout, and every
line earns its place:

```markdown
# FicHub Deploy Machine (ThinkCentre M720q) — Handover Notes

Written 2026-08-08 when switching to gamingpc as the coding machine. This box is
now **deploy-only**: run the service, DB, Redis, Ollama, Cloudflare tunnel, and
nightly jobs. Do dev/build on gamingpc.

## Machine layout

- Host: `M720q` (thinkcentre), user `alvaro`, Linux Mint. LAN IP `192.168.1.13`.
- Repo (NFS-shared, single tree): `/personal/documents/code/rust/fichub`
  - Also reachable via symlink `/home/alvaro/code/rust/fichub` (same inode).
  - GamingPC mounts `/personal` over NFS and sees this same tree.
- Binary + data survive reboots (on /personal NFS pool, 5T, 11% used).
- `.env` lives at `/personal/documents/code/rust/fichub/.env` (gitignored).
  - Source it before running any bin: `set -a && . ./.env && set +a`
```

Let me decode the architecture for you, because it's genuinely clever
and it drives everything else in this chapter:

**Two machines, one shared tree.** The ThinkCentre is the deploy box —
it runs the services. But it's a low-power SFF machine, and compiling
a Rust release build on it takes forever. So there's a second machine,
`gamingpc` (a Ryzen 7 5700G with 16 threads), which is *2.5-3x faster*
at compiling. The repo lives on `/personal`, an NFS export served by
the ThinkCentre's storage pool — and the gaming PC mounts the exact
same tree over the network. So when you build on the gaming PC, the
resulting binary lands in the same filesystem the ThinkCentre serves.
"Deploy" stops being "copy files to a server" and becomes "build in
the shared tree, then restart the service." The handover doc states it
flatly:

> Because the repo + target dir are NFS-shared, a build on gamingpc writes the
> binary into the same tree the service runs.

This is a genuinely production-grade trick for a two-machine home
setup: the *slow box serves*, the *fast box builds*, and the filesystem
is the integration point. No artifact repository, no scp step, no
version-skew between "what I tested" and "what's deployed" — they're
literally the same file.

**The `.env` lives next to the code but never in git.** Secrets in the
repo tree, excluded by `.gitignore`, sourced before every run. That's
the classic self-hosted pattern, and it's why every test invocation
and every deploy script starts with `set -a; . ./.env; set +a`.

> 💡 **Key Concept — The handover document.** `docs/deployment-handover.md`
> is the single most valuable ops artifact in this repo, and it costs
> nothing but honesty. It records the machine, the paths, the services,
> the traps, the backup schedule, and the *mistakes already made*
> (like the stats job that died at 1 a.m. every night — fixed, with
> the root cause written down). When you run a self-hosted service,
> write a handover doc. Future you, returning to the project after
> six months, is a stranger who needs it.

### The service table: systemd as the deployment platform

Now the heart of the deployment — the systemd service table. Read it
like a census of everything FicHub needs to be alive:

```markdown
## Services (systemd)

| Unit | Purpose | Status |
|------|---------|--------|
| `fichub.service` | Rust Axum API :8000 (EPUB export, scrape, OPDS) | active |
| `cloudflared.service` | Cloudflare Tunnel (public domain) | active |
| `postgresql@16-main.service` | DB `fichub` | active |
| `redis-server.service` | cache / rate limits | active |
| `ollama.service` | embeddings (nomic-embed-text) | active |
| `fichub-bot-scorer.timer` | hourly bot scoring | active |
| `fichub-leaderboards.timer` | nightly 01:00 | active |
| `fichub-quests.timer` | nightly 00:05 | active |
| `fichub-stats.timer` | nightly 01:00 | active |
| `fichub-db-backup.timer` | nightly 03:30 pg_dump → NFS | **added 2026-08-08** |
```

Count the layers in this table: the *web app* (fichub), the *edge*
(cloudflared), the *data layer* (PostgreSQL, Redis), the *ML layer*
(Ollama — the local model server that powers embeddings and the
self-healing agent), and the *jobs* (the `.timer` units: bot scoring
hourly, leaderboards and stats and quests nightly, database backup at
03:30). That's a complete production stack — and every piece of it is
a systemd unit that survives reboots, restarts on failure, and logs to
journald.

Notice the `fichub-db-backup.timer` row with the **added 2026-08-08**
annotation. The handover doc is a *living* document — rows get added
when the operation is added. If you read the table and wonder "did
they always back up the database?", the annotation answers you:
no, backups were added on a specific date, and the doc was updated the
same day. That's the discipline of someone who's been burned by data
loss.

> 🧪 **Try It Yourself — Inventory your own services.** If you run any
> service on a machine you own (a home server, a Raspberry Pi, even a
> long-lived dev box), write its equivalent of that table today:
> unit name, purpose, port, schedule. Then add the three queries that
> make it useful: `systemctl list-units --type=service --state=running`,
> `systemctl list-timers`, and `journalctl -u <unit> -n 50`. You'll
> discover you have services you forgot existed — which is exactly why
> the table exists.

Timers deserve a special note: systemd timers are the modern
replacement for cron, and they're strictly better for services that
depend on other services. A timer unit (`fichub-stats.timer`) fires
its paired service unit (`fichub-stats.service`) on a schedule, and if
the service fails, the failure is visible in `systemctl --failed` and
journald instead of a silently-lost cron email. The handover doc shows
this in action with a real war story:

```markdown
## Daily jobs fixed 2026-08-08

- `fichub-stats.service` — was dying at 01:00 every night:
  - `compute_stats.rs` counted `works.source_type` (no such column — it's on
    `fic_info`) → fixed with `LEFT JOIN fic_info fi ON fi.work_id = w.id`.
  - `compute_stats.rs` counted `export_log.format` (no such column — it's
    `etype`) → fixed to `etype = 'epub'`.
  - Same latent bug fixed in `src/routes/admin.rs` (mod queue used `w.source_type`).
  - Verified: run writes a row into `admin_daily_stats` (was empty for weeks).
```

This is a masterclass in production debugging, and it's *in the
handover doc* because the fixes happened on 2026-08-08 and were written
down the same day. Two SQL columns that didn't exist (`works.source_type`
is on `fic_info`; `export_log.format` is `etype`) had been silently
killing the nightly stats job for *weeks* — and nobody noticed, because
the row in `admin_daily_stats` was just... empty. The job exited
non-zero every night, systemd marked it failed, and the failure mode
was invisible unless you looked. The fix was a `LEFT JOIN` and a column
rename, verified by "run writes a row into `admin_daily_stats`". Note
the verification: not "I think it works" but "the table that was empty
for weeks now has a row."

> ⚠️ **Watch Out — A failed job that writes nothing is silent.** The
> stats job's bug is the classic invisible-failure trap: the service
> *ran* every night, exited with an error, and produced nothing — but
> nothing *alerted* anyone, because "produced nothing" looks exactly
> like "nothing to report." This is why the handover doc's health line
> exists (`curl -s localhost:8000/` → 200) and why the roadmap's
> number-one ops item is an uptime probe with alerting. Rule of thumb:
> if a scheduled job can fail silently, it will. Give every job a
> "did it produce output?" check.

### The deploy script: build fast, restart, verify

Now the actual deploy workflow. `deploy.sh` is 42 lines and does the
whole job — here it is in full, because it's a model for small-team
deploys:

```bash
#!/usr/bin/env bash
# deploy.sh — build on gamingpc (fast compile host), deploy on thinkcentre.
#
# Layout: repo is NFS-shared between machines (/personal on thinkcentre,
# mounted read-write on gamingpc). The release binary built on gamingpc lands
# in the SAME shared tree, so "deploy" = restart the service here + verify.
#
# Usage:
#   ./deploy.sh          build on gamingpc + restart + health check
#   ./deploy.sh --skip-build   just restart + health check (binary already fresh)
#
# Requirements: ssh gamingpc must work passwordless; .env sourced from repo.

set -euo pipefail
cd "$(dirname "$0")"

GAMINGPC="${GAMINGPC:-gamingpc}"
SERVICE=fichub
BIN=target/release/fichub
FRONTEND=frontend/build

echo "==> $(date) FicHub deploy"

if [ "${1:-}" != "--skip-build" ]; then
  echo "==> Building release on $GAMINGPC (shared NFS tree)..."
  ssh "$GAMINGPC" "cd /personal/documents/code/rust/fichub && unset CARGO_TARGET_DIR && cargo build --release --bin fichub" || {
    echo "ERROR: build on $GAMINGPC failed"; exit 1
  }
fi

echo "==> Verifying binary fresh..."
stat -c '%y %n' "$BIN"

echo "==> Restarting $SERVICE..."
sudo systemctl restart "$SERVICE"
sleep 3
systemctl is-active --quiet "$SERVICE" || { echo "ERROR: $SERVICE not active"; exit 1; }

echo "==> Health check..."
curl -sf -o /dev/null -w 'root: %{http_code}\n' http://localhost:8000/ || { echo "ERROR: root not serving"; exit 1; }

echo "==> Deploy OK"
```

Study the structure, because every line is load-bearing:

1. **`set -euo pipefail`** — the Bash "be strict" incantation. `-e`
   exits on any error, `-u` errors on unset variables, `pipefail`
   propagates failures through pipes. A deploy script without these
   will happily continue after a failed build and restart a stale
   binary. This one can't.
2. **The build is remote** — `ssh "$GAMINGPC" "cd ... && unset
   CARGO_TARGET_DIR && cargo build --release"`. The heavy compile
   happens on the fast machine, over a passwordless SSH key, *into the
   shared tree*. And note `unset CARGO_TARGET_DIR` — the handover doc
   calls this the **CARGO_TARGET_DIR trap**:
   > A worktree-exported value redirects the build to the wrong dir and
   > leaves the served binary stale (playbook §1).
   We'll see the full story of that env var in Chapter 60, but for now:
   the deploy script explicitly unsets it because a stale value from a
   previous shell session would silently redirect the build output to
   the wrong directory, and the service would keep serving the old
   binary while everyone believes it's fresh.
3. **Verify the binary is fresh** — `stat -c '%y %n' "$BIN"` prints the
   binary's mtime *before* restarting, so the output of the script
   shows you (and anyone reading the logs later) that the file you're
   about to serve was just rebuilt.
4. **Restart and prove it** — `systemctl is-active --quiet` + a curl to
   the root URL. The script is not done when the restart command
   returns; it's done when the *service answers*. `sleep 3` gives the
   server a moment to bind the port (the binary starts fast, but not
   instant — and migrations on boot, which we're about to see, add a
   beat).
5. **Explicit, timestamped output** — every step prints `==> ...` with
   the date, so a `deploy.sh` run leaves a readable audit trail in
   your terminal history. If the deploy fails, the last `==> ERROR`
   line tells you exactly which gate it failed at.

> 💡 **Key Concept — Deploy = build + restart + *prove***. The
> difference between a script and a ritual: a ritual ends with proof.
> `deploy.sh` doesn't stop at `systemctl restart` — it checks the
> service is active *and* the root URL answers. The pattern to copy
> into your own deploys: after any deployment step, verify the
> observable outcome (process alive, port listening, HTTP 200,
> database row written). Three `==>` lines of verification are worth
> a hundred of "should be fine."

### Migrations on boot: the schema arrives with the service

Here's a deployment decision that shapes the whole project, and you've
seen it referenced a dozen times in this book: **FicHub runs its
migrations on service start.** The code lives in `src/db/mod.rs`, the
`init_pool` function that every test harness and every production boot
uses:

```rust
/// Initialize the database connection pool and run migrations
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Run migrations from the migrations directory relative to the binary
    let manifest_migrations = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        exe_path.parent()
            .map(|d| d.join("migrations"))
            .filter(|p| p.exists())
            .unwrap_or_else(|| manifest_migrations.clone())
    } else {
        manifest_migrations.clone()
    };

    if migrations_path.exists() {
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;
        tracing::info!("Database migrations applied");
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }

    Ok(pool)
}
```

Read the cleverness here, because it's a production lesson hiding in
ten lines:

**The migration directory is resolved relative to the *binary*, not
the build machine.** `env!("CARGO_MANIFEST_DIR")` is the compile-time
source dir — useful during development. But the release binary might
be deployed with the repo's `migrations/` directory copied next to it
(`exe_path.parent()`), which is what a packaged deployment would do.
The code tries the runtime location first (`exe_path.parent().join(
"migrations")`), and falls back to the compile-time path. In FicHub's
actual deployment both resolve to the same NFS tree, but the code
*handles* both layouts — which is exactly the kind of robustness a
junior-written `Migrator::new(Path::new("migrations"))` (relative to
the CWD, fragile) wouldn't have. A relative path depends on *where you
run the binary from*; this resolves from *where the binary is*.

**Migrations are idempotent by design.** `Migrator::run` checks the
`_sqlx_migrations` table — which migration files have already been
applied and their checksums — and applies only the ones that haven't.
That's why a `systemctl restart` is a safe operation at any time: on
every boot, the service connects, sees "migrations 1-38 applied",
applies nothing new, logs "Database migrations applied", and serves.
And when you add migration 39, the next restart applies exactly one
new migration. The ROADMAP confirms the live state:

> - Live: fichub.polarisocial.xyz (ThinkCentre, systemd fichub.service,
>   migrations 1-38 applied). Frontend built to `frontend/build` (served from
>   disk; no restart needed for static changes), backend binary at
>   `target/release/fichub` + restart.

**"No restart needed for static changes"** — that's the SPA split
paying off in ops: the frontend is built to `frontend/build/` and
served from disk by Axum's static fallback, so shipping new frontend
code is a file copy, not a service restart. The backend binary needs a
restart (it *is* the server); the frontend is just files. Deployment
granularity like this is a design decision you make in Chapter 1 and
reap in Chapter 59.

> ⚠️ **Watch Out — The psql-applied migration trap.** The handover doc
> records a real ownership failure:
>
> ```markdown
> - `fichub-quests.service` — was dying with "must be owner of table
>   feature_clusters" (psql-applied migration ownership trap). Fixed by the
>   2026-08-08 rebuild; verified exits 0.
> ```
>
> Someone once applied a migration by hand with `psql` as the `postgres`
> superuser — creating `feature_clusters` owned by `postgres` — while
> the app's `fichub` user was the owner of everything else. The quests
> job, running as the app user, then couldn't write to the table. The
> rule that prevents this, from `docs/AGENTS.md`, is blunt:
>
> ```text
> 3. Do not change migrations/schema without human approval. If a migration
>    already applied to prod is altered, mark in `_sqlx_migrations`.
> ```
>
> Never hand-apply a migration to production. Let the app's migration
> runner own the schema, so the ownership and the checksums stay
> consistent. The `_sqlx_migrations` table is the source of truth; if
> you ever touch it, you must say so.

### The body-cache drive: where the real treasure lives

We met the body cache in Part 4 — every scraped fic is saved as raw
HTML plus extracted chapters, sharded on disk under `BODY_CACHE_DIR`.
In production, that's the most valuable asset in the entire project.
Re-read the config default from `src/config.rs`:

```rust
let body_cache_dir = std::env::var("BODY_CACHE_DIR")
    .unwrap_or_else(|_| "/public/literature/fichub/bodies".to_string());
```

`/public/literature/fichub/bodies` — an *attached drive*, not the
system disk and not the database. The handover doc explains why:

```markdown
- Binary + data survive reboots (on /personal NFS pool, 5T, 11% used).
```

And the roadmap's ops section makes the value explicit:

> - **Backup the body cache + EPUBs.** `/public/literature/fichub/bodies` is
>   the most valuable data asset — include it in any maintenance routine.

Let's think about what this design means. The body cache is a *cache*,
but it's a cache of irreplaceable things: fics scraped from sites that
may block you tomorrow (remember the AO3 525 / FFN 403 story), fics
whose source pages might vanish. The moment a fic's body is on disk,
FicHub can serve exports for it forever without ever touching the
source site again — that's the "site-as-cache" philosophy from the
roadmap: the archive *is* the cache. Losing `/public/literature/fichub/bodies`
would mean re-scraping everything — or losing everything, if the
sources are unreachable. So it lives on the big NFS pool (5TB, 11%
used), it's backed up by the nightly `pg_dump` ritual's sibling
discipline (the roadmap's P2 item: "Backup the body cache + EPUBs"),
and its sharding layout is *git-objects style*, which we'll see
mattered for a very specific reason in Chapter 60.

The sharding function, from `src/body_cache.rs`:

```rust
fn shard_dir(root: &Path, url_id: &str) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    if id.len() >= 4 {
        root.join(&id[..2]).join(&id[2..4])
    } else {
        root.join("_").join("_")
    }
}

fn html_path(root: &Path, url_id: &str, version: i32) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    shard_dir(root, url_id).join(format!("{id}.v{version}.html"))
}

fn json_path(root: &Path, url_id: &str, version: i32) -> PathBuf {
    let id = url_id.to_ascii_lowercase();
    shard_dir(root, url_id).join(format!("{id}.v{version}.json"))
}
```

Two-level hex sharding: `ab/cdef/abcdef....v1.html`. If you've seen
git's object store, this looks familiar — and the comment in
`body_cache.rs` says so explicitly: "git-objects style". The point of
sharding is directory fan-out: instead of one directory with 10,000
files (which filesystems hate), you get 256 top-level directories
(`00`-`ff`), each with 256 second-level directories, each holding a
handful of files. The version suffix (`v1`, `v2`, ...) is the curator
fix system from Part 11 — a wrong body gets bumped to a new version
rather than overwritten, so a bad extraction can be re-done from the
saved HTML without re-scraping. `current_version()` scans the shard
dir for the highest `v{n}` — the "version" is discovered from disk, not
stored in a database, so the filesystem is the single source of truth.

> 🧪 **Try It Yourself — Explore the real cache.** If you have access
> to the FicHub machine (or any NFS mount of `/personal`), run:
>
> ```text
> du -sh /public/literature/fichub/bodies
> find /public/literature/fichub/bodies -name '*.json' | head -5
> cat "$(find /public/literature/fichub/bodies -name '*.json' | head -1)" | head -20
> ```
>
> You'll see the shard layout in the wild, and the JSON blob of an
> actual scraped fic — title, chapters, saved_at. Notice that a file
> this small represents a fic that may have taken 30 seconds and
> several HTTP requests to scrape. That's why it's the most valuable
> data asset: it's *paid-for* data, in time and in goodwill with the
> source sites.

### The full production picture

Step back and look at what production FicHub actually is:

- **One small PC** running Linux Mint, five services + five timers
  under systemd, everything logging to journald.
- **A Cloudflare tunnel** (`cloudflared.service`) exposing the local
  port 8000 as fichub.polarisocial.xyz — no router port-forwarding, no
  public IP needed, TLS handled by Cloudflare. (The doc's health line:
  `curl -s localhost:8000/` → 200 — the tunnel does the rest.)
- **PostgreSQL 16** with 38 migrations applied on boot, backed up
  nightly by a systemd timer + `pg_dump` to the NFS pool.
- **Redis** for rate limits, shadowbans, and caching.
- **Ollama** running a local embedding model — the AI features (Ask the
  Archive, the auto-tagger, roadmap consensus embeddings, the
  self-healing agent) all run on-device, no cloud API bills.
- **The body cache** on the big attached drive, holding the archive's
  most valuable asset.
- **A second, faster machine** doing the compilation into the shared
  tree, with a 42-line script orchestrating build → restart → prove.

There's no orchestration platform, no container registry, no
infrastructure-as-code. There *is* a handover document that tells you
exactly how everything works and what has already broken. That trade —
conventional tools, extraordinary documentation — is the real lesson
of this chapter. FicHub is production-grade not because of fancy
tooling but because someone wrote down how it works, made every
failure visible, and built verification into every ritual.

Before we ship the next feature, we need to talk about the *developer
workflow* — how a project living on an NFS-shared repo handles feature
branches without destroying the main checkout. That story involves git
worktrees, a filesystem quirk, and one of the strangest bugs in this
book. Chapter 60.

---

## Chapter 60 — Git Worktrees and the NFS Quirk: The Developer Workflow

Every chapter of this book has followed the same promise: we read the
real repo, and the real repo tells a true story. This chapter's story
is about the *developer's* side of production — how a solo developer
works on a project whose main repo lives on an NFS mount, and the two
git bugs that taught more about git internals than any tutorial ever
could.

The tools in this chapter are **git worktrees**, and the plot is the
split personality of the machine layout we met in Chapter 59: the main
repo lives at `/personal/documents/code/rust/fichub` (an NFS mount),
and the worktrees live at `/media/alvaro/code-worktrees` (local ext4
disk). That split sounds like a small detail. It is not. It is the
engine of two of the strangest, most instructive bugs in this entire
book.

### Why worktrees? The problem worktrees solve

First, the innocent question: why not just `git checkout -b feature`
in the main repo like everyone learns?

Because the main repo on `/personal` is the *deployed* tree. Remember
Chapter 59: the release binary is built *inside* that tree
(`target/release/fichub`), and the service runs from it. If you switch
branches in the main repo — or even run `git status` badly — you can
leave the deployed tree in a state that doesn't match what the service
expects. Worse, a long-running feature branch keeps you from merging
quick hotfixes without stashing. And Rust builds: the `target/`
directory in the main repo is a shared, enormous build cache; thrashing
it by switching branches invalidates half the incremental compilation.

The classic answer is worktrees. `git worktree add` checks out a
branch into a *separate directory* that shares the repository's
`.git` — same history, same objects, same refs — but has its own
working directory, its own index, and can have a different branch
checked out at any time. The canonical FicHub setup, from the handover
doc:

```markdown
- Worktrees + cargo targets live on the external/NFS pool:
  `/media/alvaro/code-worktrees/*` (targets via `.cargo/config.toml` or
  `CARGO_TARGET_DIR=/home/alvaro/<wt>-target`).
```

So a feature branch gets a fresh directory on the *local* disk:

```text
git worktree add /media/alvaro/code-worktrees/rec-bandit origin/main -b feat/rec-bandit
```

Now you can work on the feature in its own directory, build it with its
own `target/` (no cache thrashing), run its tests, and merge it when
it's ready — while the main repo stays clean and deployable. Multiple
branches, multiple working copies, one repository. `git worktree list`
shows the whole constellation.

> 💡 **Key Concept — Worktrees are cheap parallel universes.** A
> worktree is not a clone: it shares the `.git` directory (objects,
> refs, config), so `git push` from a worktree pushes the same remote
> refs, and commits made in a worktree are visible in the main repo
> immediately (they share the object store). What's *separate*: the
> working tree files, the index, and the HEAD. That means two branches
> can be checked out simultaneously with zero stash dances, and each
> worktree can have its own `target/` for builds. The price: the
> shared `.git` means shared config — which is exactly where the
> second bug in this chapter lives.

### The NFS quirk, failure 1: "error when closing loose object file"

Now the fun begins. Working in a worktree, you run the most routine
command in git — `git add` or `git commit` — and git screams:

```text
error when closing loose object file: Permission denied
```

The file on disk is fine. `ls` shows it, `cat` reads it. Your user owns
everything. You didn't change permissions. But git's *write path*
fails, every time, on the NFS-mounted object store.

Here's the thing to understand about how git stores objects: when you
commit, git writes the object data to a temp file in `.git/objects`
and then renames it into place. The way it creates that temp file is
specific — it opens it with mode `0444` (read-only!) and relies on the
rename to establish the final name. From the diagnosis recorded in the
FicHub development skill (which documents this bug exhaustively):

> **Root cause (observed via strace):** git creates the loose object temp
> file with mode `0444` (`openat(...tmp_obj_XXX, O_RDWR|O_CREAT|O_EXCL, 0444)`),
> and on this NFS mount `close()` then fails with `EACCES` (NFS flush checks
> write permission).

Wait, `0444` — read-only? Yes. Git creates the temp object file
read-only on purpose: the object will only ever be read, and the
read-only mode is a cheap safety property. Normally this works
perfectly: on a local filesystem, `close()` doesn't care about write
permission. But NFS is a *protocol*, not a filesystem — the server
flushes the file on close, and this particular NFS export is backed by
a **mergerfs FUSE pool** (the `/personal` export is itself a union of
multiple disks), and the FUSE layer's post-write attribute check sees
"file is 0444 and the write came through the NFS daemon" and returns
EACCES. The close fails, git aborts the object write, and you get a
mystifying "Permission denied" for an operation that *should* work.

The diagnosis chain here is a masterpiece of systems debugging, and the
skill document is brutally honest about the dead ends:

> **Deeper root cause (2026-08-12):** the NFS server's `/personal` export is
> itself a **mergerfs FUSE pool**; the EACCES originates in the mergerfs layer
> (nfsd propagates the FUSE daemon's error on post-write attribute/fsync of
> new files). NFS-level tuning does NOT fix it — all of these were tested and
> failed: `core.fsyncObjectFiles true` (made it deterministic), server export
> `rw,sync`, client `noac` mount, `core.preloadIndex false`. Don't waste time
> re-testing these.

Four tuning attempts, each with a plausible mechanism, each failing —
and then the note that will save the next person hours: *don't waste
time re-testing these*. That's a gift. When you document a bug, list
the things you tried that didn't work. It's as valuable as the fix.

**The fix** is as elegant as the bug is ugly: don't let git write to
the NFS object store at all. Give the worktree its own *local* object
directory, and tell git the main repo's objects are an *alternate*
(read-only fallback):

```bash
# one-time setup per worktree
mkdir -p /media/alvaro/code-worktrees/.<name>-gitobjects/info
echo '/personal/documents/code/rust/fichub/.git/objects' \
  > /media/alvaro/code-worktrees/.<name>-gitobjects/info/alternates

# every git WRITE command (add/commit) needs this env var
export GIT_OBJECT_DIRECTORY=/media/alvaro/code-worktrees/.<name>-gitobjects
git add ...
git commit ...
```

How this works is the beautiful part, so let's unpack it:

- `GIT_OBJECT_DIRECTORY` (the modern spelling of the old
  `GIT_OBJECT_DIRECTORY` env var; git also reads it from the
  `objects` entry in the worktree's git dir) points git at a *local*
  object store. New objects — your commits' blobs, trees, and commit
  objects — get written there, on ext4, where `close()` never fails.
- The `info/alternates` file tells git: "if an object isn't in my
  local store, look in `/personal/.../.git/objects`." So *reads* —
  `git diff`, `git status`, `git log`, checking out history — still
  work against the shared NFS objects, falling through to the
  alternates.
- Writes go local; reads fall through to shared. Both worlds get what
  they need. Git's alternates mechanism — designed for partial clones
  and shared object caches — becomes the bridge over the NFS failure.

The skill doc's summary is the practical rule:

> Reads (diff/status) work without the var because alternates fall back to the
> NFS object dir; writes must go to the local dir.

> ⚠️ **Watch Out — strace is your friend for filesystem mysteries.**
> The EACCES bug was invisible to every normal diagnostic: permissions
> looked right, `ls` and `cat` worked, and only the *system call*
> revealed the truth. `strace -f -e trace=file git add . 2>&1 | tail`
> showed the `openat(..., 0444)` followed by `close()` → `EACCES`,
> and suddenly the whole mystery made sense: git's write path, not
> your files, was the problem. When a filesystem bug makes no sense,
> trace the actual syscalls. The evidence is always there; you just
> have to look at the right layer.

### The NFS quirk, failure 2: the `core.worktree` leak

The second bug is more insidious, because its *symptoms* look like
you've corrupted the main repository. One morning you run `git status`
in the main repo and see... the worktree's files. Untracked files from
`/media/alvaro/code-worktrees/some-feature` appearing in the *main
repo's* status. Phantom `M` modifications on tracked files. `git diff
HEAD` showing the worktree's versions. The main repo looks destroyed.

Here's what actually happened. Somewhere along the way, a command ran
`git config core.worktree <path>` *inside* a worktree. That single
command is a footgun, and the skill doc states the rule in its first
sentence:

> **NEVER run `git config core.worktree <path>` inside a worktree.**
> Worktree `--local` config writes to the SHARED `.git/config` (the main
> repo's config file), so every git command in the MAIN repo suddenly treats
> the worktree as its working directory.

This is the subtle part, and it's worth slow-reading. `core.worktree`
is a git config key that overrides where git thinks the working tree
is. When you set it with `--local` *inside a worktree*, "local" means
"the shared `.git/config`" — because a worktree's git dir is inside
the main repo's `.git` (that's what makes it a worktree). So the
setting leaks into the *main repo's* configuration. And
`core.worktree` is the single most powerful location override in git:
the main repo's commands now think their working directory is the
worktree's directory. `git status` reads the worktree's files, `git
diff` diffs the worktree's versions, and everything looks broken.

The panic protocol — and this is the part every junior needs — is in
the skill doc:

> **Verify real state (do this BEFORE assuming damage):**
> ```bash
> git hash-object <file>          # reads the ACTUAL path
> git rev-parse HEAD:<file>       # committed version
> # if they match, on-disk file is fine; only config/index is confused
> git config --list --show-origin | grep -i worktree   # find the leak
> ```

Three commands, thirty seconds, and you know the truth: the on-disk
files are almost certainly *fine* — the repo isn't corrupted, only the
configuration is confused. `git hash-object` reads the actual path on
disk; `git rev-parse HEAD:<file>` reads the committed version; if they
agree, the file on disk matches what git committed, and nothing is
lost. Then `git config --list --show-origin` shows you exactly which
config file has the rogue `core.worktree` — the smoking gun.

And the fix is one command, run from the *main* repo:

```bash
git config --local --unset core.worktree   # run from the MAIN repo
git status --short                         # back to normal
```

> 🧪 **Try It Yourself — Reproduce the leak safely.** In a throwaway
> repo (never the real FicHub repo!), create a worktree, then run
> `git -C <worktree> config --local core.worktree /tmp/somewhere`,
> then run `git status` in the *main* repo and watch it freak out.
> Then run `git config --list --show-origin | grep worktree` to see
> the leak, and `git config --local --unset core.worktree` from the
> main repo to fix it. You'll have permanently internalized why
> "verify before assuming damage" and "never set core.worktree in a
> worktree" are the rules. (Make sure you're in a scratch repo —
> this exact mistake is what the rule exists to prevent.)

> 💡 **Key Concept — Config is shared, working trees are not.** The
> two NFS bugs are two faces of the same architectural fact: a
> worktree shares *everything in `.git`* — objects, refs, *and
> config* — while owning only its working directory and index. That
> sharing is what makes worktrees cheap and powerful, and it's exactly
> what makes worktree-local config writes dangerous. Whenever you run
> `git config` in a worktree, ask: "which `.git/config` am I writing?"
> If the answer is "the shared one," you've just changed the main
> repo's behavior. The safe pattern: set config with `git -C <main
> repo> config ...` explicitly, or use the worktree's *git dir*
> (`git rev-parse --git-path config`) — or simply don't set
> `core.worktree` at all.

### The workflow, end to end

Put it together and the mature FicHub developer workflow looks like
this:

1. **Main repo stays clean and deployable.** Branching happens in
   worktrees, on local disk, with the local object-directory trick for
   writes. `git worktree list` shows: main at `/personal/...` plus
   however many feature branches are in flight under
   `/media/alvaro/code-worktrees/`.
2. **Builds are isolated per worktree.** Each worktree gets its own
   `CARGO_TARGET_DIR` (e.g. `/home/alvaro/<wt>-target`), so switching
   branches never invalidates the shared `target/`, and the main
   repo's release binary is never disturbed by feature-branch builds.
   This is also why `deploy.sh` unsets `CARGO_TARGET_DIR` before
   building the release binary — a leftover export from a worktree
   session would redirect the production build into a worktree's
   target dir and leave the served binary stale. The env var is a
   one-shot tripwire that the deploy script defuses explicitly.
3. **Testing follows the same split.** Fast unit tests run anywhere.
   The DB-gated suites run on the ThinkCentre (where the services
   live) or against its services over the network — which is why the
   handover doc notes "The stack (Postgres/Redis/Ollama/service +
   DB-gated tests) stays HERE. Only compile happens on gamingpc."
   Compile on the fast box, test against the real stack, deploy by
   restart.
4. **Mirror discipline.** After merging to main, push to the remotes —
   and the handover doc records the *other* NFS quirk, the mirror
   lockfile:

   > Mirror rule: after merging to main, `git push github main`.
   > (remove stale `.git/refs/remotes/github/main.lock` first; NFS
   > git-objects workaround — see skill reference).

   NFS leaves stale lockfiles behind when a process dies mid-push; the
   fix is deleting the stale lock and pushing again. Same theme as
   everything else in this chapter: on NFS, git's assumptions about
   local atomicity break, and the workflow absorbs the breakage with
   explicit, documented rituals.

And the whole constellation is documented in `docs/AGENTS.md` under
"Known quirks" — because a quirk that isn't written down is a bug
someone will re-discover at 2 a.m.:

```text
- NFS shared-target quirk: `target/debug/incremental` rlib fingerprints go
  stale after mount wedges — `rm -rf target/debug/incremental` before
  canonical verify. Verify MUST run with
  `export CARGO_TARGET_DIR=/media/alvaro/cargo-target-sh` (local SSD; the
  NFS target-dir pin was removed 2026-08-12 — it caused E0463 rlib
  corruption).
```

Incremental-compilation fingerprints going stale after an NFS mount
wedges — meaning cargo rebuilds the world or, worse, links stale
artifacts (`E0463 rlib corruption`). The fix is a documented ritual:
clean the incremental dir before a canonical verify, and pin the
verify's target dir to local SSD. Every environment quirk in this
project has the same shape: *weird failure → root cause → documented
ritual that avoids it*. That's the meta-skill this whole chapter has
been teaching.

> ⚠️ **Watch Out — The `E0463` lesson.** "Can't find crate" errors
> (E0463) after an NFS mount hiccup are *not* a dependency problem.
> The incremental-compilation fingerprints that tell cargo "this rlib
> is fresh" live in `target/debug/incremental`; when the NFS mount
> wedges and unwedges, those fingerprints can lie, and cargo links
> stale or mismatched artifacts. The ritual — `rm -rf
> target/debug/incremental` — forces a clean recompile of the
> affected crates. If you ever see E0463 appear from nowhere on an
> NFS-mounted tree, clean the incremental dir before you blame your
> Cargo.toml.

### What we just built

If you take nothing else from this chapter, take the *method*: when the
environment does something impossible, don't fight it — *route around
it and write it down*. Git writes objects with `0444`? Give it a local
object store and alternates for reads. Config leaked into the shared
repo? Unset it, and add "never set core.worktree in a worktree" to the
rules. Incremental fingerprints lie? Clean them as a ritual. Every one
of these is a one-line fix backed by a paragraph of hard-won
understanding — and every one is written down in a doc or skill so the
next person (or the same person, six months later) skips the
three-hour diagnosis.

That documentation habit is also how you hand a project to its future
maintainers — which is exactly what the roadmap in Chapter 61 is for:
a single document that says where the project is, what's shipped, and
what comes next.

---

## Chapter 61 — The Roadmap: What's Next

Every project worth building has a future, and FicHub's future is
written down in a single file: `docs/ROADMAP.md`. Not in a wiki, not
in someone's head, not in a chat log — in the repo, versioned, right
next to the code it describes. The header of the file tells you its
philosophy in one sentence:

```markdown
# FicHub — Consolidated Roadmap (2026-08-12)

> **Canonical planning doc.** Supersedes the old STATUS.md / TODO.md / NEXT.md
> split: this single file tracks what's shipped, what's in flight, and every
> known suggestion/backlog item — with priorities grounded in the current
> codebase state. Feature specs live in `docs/SPECIFICATION.md`; deep-dive
> references in the `fichub-development` skill; operator docs in `docs/src/`.
```

"Supersedes the old STATUS.md / TODO.md / NEXT.md split." If you've
ever worked on a project with three overlapping planning docs, you know
exactly why this matters: planning docs rot. They multiply, they
contradict each other, and nobody knows which one is canonical. The
first act of a mature project is *consolidation* — one file, one
source of truth, with pointers to where the details live. That's a
process lesson, and it's the most important thing this chapter teaches
before we even read a backlog item.

The roadmap has one section we've already mined (the test-coverage
inventory), one we've seen (the deployment notes), and one that's pure
gold for understanding how a real project decides what to build next:
the prioritized backlog. Let's read it the way a maintainer reads it —
starting from "where we are."

### Where the project is: the one-paragraph summary

Every good roadmap starts with a brutally honest current-state
paragraph. FicHub's does not mince words:

```markdown
## 0. Where we are (one paragraph)

FicHub is a self-hosted fanfiction archive + community platform (Rust/Axum
backend, SvelteKit SPA, PostgreSQL + pgvector, Redis, Ollama). All five
feature waves (ask-the-archive, per-fic suggestions, public roadmap
consensus, pluggable recommender platform, full-site i18n) are merged,
deployed to fichub.polarisocial.xyz, and live-verified. This session
(2026-08-12) shipped **full one-to-one adapter parity with FanFicFare**:
the `fanfic-scrapers` crate (now v0.10.0 on crates.io) ports all **107 real
FFF adapters** as native Rust scrapers — including login support for
author/adult-gated sites and an `is_adult` flag for adult archives — plus
site-as-cache body blobs, curator peer-voted fixes, non-PII usage
analytics, a transparent modlog, and self-healing telemetry (M1). The site
is close to small-cohort invite readiness; the remaining content-side gap
is outbound reachability from the host (AO3 525 / FFN 403), tracked as
P1 cookie ingestion.
```

Read it for the *shape*, not the details: what's shipped (five feature
waves, live-verified), what was just finished (107 scrapers — a number
that would have been science fiction at the start of this book), what
the current state is ("close to small-cohort invite readiness"), and
what the *single biggest gap* is (outbound reachability — the host
server gets blocked by AO3 and FFN, so the archive can't ingest from
the two biggest sources). One paragraph, and you know the whole
project. That's the discipline: a roadmap that can't be summarized in
one paragraph isn't a roadmap, it's a novel.

> 💡 **Key Concept — The "one paragraph" test.** If you can't write
> where your project is right now in a single paragraph — shipped
> what, blocked on what — you don't actually know where it is. The
> paragraph forces the maintainer to *rank*: the things that make it
> into one paragraph are the things that actually matter. When you
> write your own roadmap, start with this paragraph and refuse to
> expand it. Everything else hangs off it.

### Priorities grounded in reality: the P1-P7 backlog

The backlog is the roadmap's engine room. It's organized by priority
tier (P1 through P7), and the ordering principle is stated right at
the top of the section:

```markdown
> Generated from the 2026-08-11 backlog review. Items are ordered by leverage;
> each notes what's already true vs what's needed.
```

**"Ordered by leverage"** — not by excitement, not by "what's the
coolest feature," but by *how much value each item unlocks per unit of
effort*. And the format rule is the killer detail: *"each notes what's
already true vs what's needed."* Every backlog item in this file
starts from the current state of the codebase, so you can judge the
item without re-reading the code. That's the difference between a
backlog that's actionable and a wish list.

Let's read the top of the pile, because P1 tells you what the
maintainer genuinely believes matters most:

```markdown
### P1 — Content bottleneck first

1. **User-supplied cookie ingestion for AO3/FFN.** The host is blocked
   (AO3 525 / FFN 403 from the server; force.net's bot-guard beat even
   Playwright — needs a real human click). The native adapter set now
   covers all 107 FFF sites, but the *host's outbound* blocking still
   limits what the archive can actually ingest from AO3/FFN. Pragmatic
   fix: a "cookie import" flow — user opens the blocked page in their own
   browser once, pastes cf_clearance / session cookie, scraper reuses it.
   Low-risk now: body cache + curator peer-voted fixes make failed scrapes
   recoverable. Without this the archive only grows from reachable hosts.
```

There's a whole strategic argument in that one item. The scrapers are
*ready* — 107 adapters, including login support — but the *server's
outbound connections* are blocked by the biggest sites' bot defenses.
The pragmatic fix isn't to build a better scraper (the bot-guard beat
even Playwright); it's to have *users* provide the cookie — a real
human click unlocks the page, and the archive reuses the session. Note
the risk analysis: "Low-risk now: body cache + curator peer-voted
fixes make failed scrapes recoverable." The team decided this is safe
to ship *because* of the infrastructure built in earlier waves. That's
the roadmap connecting to the code we built in Parts 4 and 11: the
body cache means a bad scrape is recoverable, so letting users feed
cookies is an acceptable risk.

And notice what's *not* in P1: no new AI features, no fancier
recommendations, no UI polish. The content bottleneck comes first,
because an archive with nothing new to read dies no matter how pretty
the reader is. Priorities like that are the signature of someone who
understands their product.

Item 2 in P1 is the one that should make you smile, because it's
*already built* by the time this book is being written — the
scriptable API-surface e2e:

```markdown
2. **Scriptable API-surface e2e (route-walk).** Boot the server, enumerate
   the router, hit every endpoint with a fixture admin token, assert 2xx (or
   expected 4xx), flag empty/stub responses. Mechanically catches "stub left
   in production" in CI today, no browser needed. (This is the scriptable
   version of the visual-audit ask.)
```

That's `qa/api-walk.js` — which we met in Chapter 57, where the
roadmap's "2b" section records its score: **97/97 green**, plus two
real 500-bugs it caught. This is the roadmap working as designed:
items get prioritized, get built, get verified, and get moved into the
"shipped" section with their evidence. The roadmap is not a static
wish list; it's a ledger of the project's evolution.

> 🧪 **Try It Yourself — Write your project's "already true vs
> needed" for one feature.** Pick any feature you'd like to build next
> (in FicHub or your own project) and write it in the roadmap format:
> what's already true in the codebase, what's needed, why it's worth
> doing, and what would make it safe to ship. Then rank it against
> two other candidate features by leverage. You'll be surprised how
> quickly the "exciting" feature falls behind the "unblocking"
> feature — which is the whole point of the exercise.

### The ops debt tier: what running a server teaches you

P2 is where the roadmap shows its real-world scars — the things you
only learn by running a service that real people use:

```markdown
### P2 — Ops debt

5. **CI/CD + uptime probe.** Internal Forgejo is unreachable; minimum viable
   = external uptime check (UptimeRobot or Hermes cron) alerting to Telegram
   when health flakes or the nightly QA harness (`fichub-nightly-qa`) finds
   new bugs.
6. **Secrets hygiene.** Move the DB password out of `.env` into a 600-perm
   secrets file via systemd EnvironmentFile.
7. **Backup the body cache + EPUBs.** `/public/literature/fichub/bodies` is
   the most valuable data asset — include it in any maintenance routine.
```

Three items, three lessons. Item 5: the internal CI server is
unreachable, so the *minimum viable* version is an external uptime
probe that alerts to Telegram — progress over perfection, and a
reminder that "CI is down" is a normal state to plan around, not a
disaster. Item 6: secrets hygiene — the DB password currently lives in
`.env` (gitignored, but world-readable-ish and in the repo tree);
moving it to a 600-permission file referenced by systemd's
`EnvironmentFile` is the standard hardening step. Item 7 is the one
we flagged in Chapter 59: the body cache is the most valuable data
asset, and it isn't in the nightly backup yet. The roadmap says it
plainly so it can't be forgotten.

> ⚠️ **Watch Out — Your most valuable asset is the one you don't
> back up.** The database has a nightly `pg_dump` (added 2026-08-08).
> The body cache — which contains every scraped fic, potentially
> unre-scrapeable if sources block you — did not, at roadmap time.
> That's the classic backup blind spot: we back up what's *easy to
> describe* (the database) and forget what's *hard to replace* (the
> derived data). When you set up any service, ask: if this disk died
> tonight, what could never be recreated? Back *that* up first. The
> roadmap's P2 item 7 exists precisely because someone asked that
> question.

### The deeper tiers: a platform that earns its keep

Skim the lower tiers and you'll see the roadmap's strategic spine —
turning FicHub from a download server into a *platform*. P3 is the
recommender earning its keep (shadow-run the `decay` and `embeddings`
strategies we built in Part 9, let `rec_impressions` decide the
winner). P4 has the differentiator that no scraper-only archive can
match:

```markdown
10. **Full-text search over fic bodies.** The body cache stores every fic as
    JSON on disk — index it (pg FTS or tantivy) for quote search ("fics where
    X says Y"). No scraper-only archive offers that.
```

Quote search — "fics where X says Y" — over the *entire corpus of
scraped fics*. That's the kind of feature that only becomes possible
because of an earlier architectural bet (the body cache in Part 4).
The roadmap item even names the candidate technologies (pg FTS or
tantivy) and states the moat ("No scraper-only archive offers that").
This is what a roadmap for a *product* looks like, not just a roadmap
for a *codebase*.

P5 is the curator/admin UI backlog — a list of items where "APIs mostly
exist," including the reminder that some are already done ("Blacklist
UI — DONE (page + tests exist)"). P6 is cheap UX wins, including a
tellingly specific one:

```markdown
22. **Verify the known mismatch**: search suggestions read
    `localStorage('fichub_token')` directly while the auth store may differ —
    personalization could be silently disabled.
```

That's a roadmap item that reads like a bug report — because it
basically is. The roadmap is also a *memory*: things that are "probably
fine but not verified" get tracked, not forgotten. And P7 is the big
bet that gives the whole roadmap its purpose:

```markdown
24. **Invite the small cohort**: onboarding flow, seed content from body-cache
    favorites, "new member" landing. The governance layer (modlog, consensus,
    transparency) is ready; the cohort is the missing piece.
```

Everything else — the modlog, the consensus engine, the transparent
moderation, the usage analytics — was built *toward* this moment:
opening the archive to a small group of real readers. The governance
layer is ready; the missing piece is people. That's the north star,
and the roadmap never loses sight of it.

> 💡 **Key Concept — Roadmaps encode bets, not just chores.** The
> difference between a task list and a roadmap is that a roadmap
> states *why each item matters* and *what it unlocks*. "Full-text
> search" is a chore; "quote search over every scraped fic — no
> scraper-only archive offers that" is a bet on what makes FicHub
> special. When you write your roadmap, every item should answer:
> what's already true, what's needed, and why does this unlock
> something that matters? If an item can't answer those, it probably
> shouldn't be on the roadmap.

### The shipped ledger: evidence, not vibes

The roadmap's "Shipped" sections are the part most roadmaps skip —
and the part that makes FicHub's roadmap *trustworthy*. Look at the
format of the entries in section 1:

```markdown
- **Self-healing M1** (7780915, 727e771): `scrape_failures` + `agent_runs`
  (migrations 029-030), pure-Rust classifier (transient/blocked/structural/
  systemic), fingerprint + debounce, HTML snapshots, `POST /api/admin/heal`
  diagnose-only. Autonomy OFF by default (`AGENT_ENABLED=false`); agent =
  CommandCode API (deepseek/deepseek-v4-flash) with Ollama fallback.
```

Every claim has a commit hash. Every feature names its migrations, its
endpoints, its config keys. "Shipped" is a *verifiable claim*, and the
roadmap provides the evidence inline. There's even a section
specifically for things that were *believed* shipped but turned out to
already exist:

```markdown
> Verified already-existing (ROADMAP items that were stale): auto-tag review
> UI (admin/auto-tag), manual fic approval (admin/moderation), blacklist UI,
> main_char_attr search UI (search page), search→export conversion
> (admin/search-analytics), zero-result queries report.
```

An entire subsection dedicated to "we thought this was missing, and it
was already built." That's the roadmap being honest about its own
stale items — which is exactly why you can trust the rest of it. A
roadmap that admits its errors is a roadmap you can plan against.

And the "In flight" section keeps the current focus to exactly two
items — invite readiness and self-healing M2:

```markdown
## 2. In flight / current focus

- **Invite readiness for a small cohort**: modlog + transparency + analytics
  are in; onboarding flow + seed content + a "new member" landing remain.
- **Self-healing M2** (documented in NEXT.md): on-the-fly scraper creation by
  the agent on structural failure, safety-gated. Autonomy stays OFF until the
  loop is proven.
```

Two items. That's the discipline of a maintainer who knows that
focus is a feature: the roadmap says *everything* is tracked but
*only two things are being worked on right now*. "Autonomy stays OFF
until the loop is proven" — a safety gate stated as a policy, not an
afterthought.

> ⚠️ **Watch Out — The autonomy safety gate.** The self-healing agent
> (Part 10) is the project's most powerful and most dangerous
> component: it can *create scrapers on the fly* (M2) and, eventually,
> modify code. The roadmap's rule is explicit: `AGENT_ENABLED=false`
> by default, diagnose-only for M1, and M2's autonomy stays OFF until
> the loop is proven. When you build AI features that can mutate
> state, the safety gate is not optional — it's the feature. The
> pattern to copy: default-off, human-visible actions only,
> opt-in escalation with evidence.

### What we just built

The roadmap is the project's memory and its compass: a one-paragraph
summary of reality, a shipped ledger with commit-hash evidence, a
backlog ordered by leverage with "already true vs needed" on every
item, and an explicit current focus of exactly two things. It's the
document that lets a project survive its own growth — and the document
that tells the next maintainer (or the same maintainer, next month)
exactly what matters.

And that brings us to the final chapter of the final part. Thirteen
parts, sixty-two chapters, one project built from nothing into a
living platform. Chapter 62 is not a technical chapter. It's the one
we've been walking toward since Chapter 1: the finish line.

---

## Chapter 62 — Congratulations, You Built FicHub!

Let's stop, right here, and do something we haven't done once in
sixty-one chapters: *look back*.

Because here's the thing. Somewhere around Chapter 8 — when the error
types were still fighting us, or Chapter 18, when the body cache's
sharding logic refused to behave, or Chapter 46, when the self-healing
classifier just wouldn't classify — there was probably a moment when
this whole project felt like it would never be finished. Every
codebase has those moments. The difference between a project that dies
in those moments and a project that ships is not talent. It's showing
up for the next chapter anyway.

So let's take the tour. The whole journey, thirteen parts, in one
breath.

### The journey, part by part

**Part 1 — Welcome.** We met FicHub: a fanfiction archive that scrapes
stories from across the web and turns them into EPUBs. We looked at
the repo for the first time — `src/`, `frontend/`, `migrations/`,
`tests/` — and got the stack running on a laptop for the first time.
Remember that first `cargo run`?

**Part 2 — The Rust foundation.** The entry point, the router, the
`AppState` that wires the whole dependency graph together, and the
error type that turns panics into clean HTTP responses. The skeleton
every other part hung off.

**Part 3 — Configuration & database.** Sixty-something environment
variables in `config.rs`, the SQLx pool, and migrations 1 through 34 —
the schema story that grew with the project, one numbered file at a
time. Plus the Redis limiter that would later become a three-tier,
shadowban-aware anti-bot system.

**Part 4 — The scraper subsystem.** How FicHub talks to AO3, FFN,
RoyalRoad, XenForo, and eventually 107 sites via the `fanfic-scrapers`
crate — parsing metadata, extracting chapters, deciding which scraper
to trust, and caching every body to disk. The site-as-cache idea was
born here.

**Part 5 — Exports.** From URL to EPUB: the semaphore-gated pipeline,
the pure-Rust EPUB builder, the HTML/TXT/MD side, and the Calibre
sidecar for MOBI/PDF/AZW3. Downloads cached with hashes, so the same
fic requested twice costs nothing twice.

**Part 6 — API, search, reader.** The boolean search parser with
AND/OR/NOT and phrases, the "Dark Harry" `main_char_attr` semantics,
typo tolerance, and the web reader that turned the archive into a
place you *read*, not just download from.

**Part 7 — Auth & social.** JWTs, roles and reputation tiers,
bookmarks, five-star ratings and reviews, threaded comments, follows,
updates feeds, notifications. FicHub became a community.

**Part 8 — Community.** Fic Requests, reading lists and shelves,
series and author pages, RSS/Atom feeds, and the roadmap-consensus
arena where users vote features against each other with Elo ratings.

**Part 9 — Recommendations.** The pluggable strategy registry, the
co-occurrence engine with its golden test, decay and embeddings and
matrix factorization and bandits, shadow mode with impressions logging
— A/B testing for a solo developer.

**Part 10 — AI features.** Ask the Archive, the auto-tagger, machine
translations, and the self-healing agent that diagnoses its own
scraping failures — autonomy off by default, safety-gated, evidence-
backed.

**Part 11 — Admin & transparency.** The anti-bot arsenal (honeypots,
tiered rate limits, shadowbans, proof-of-work), usage analytics that
respect privacy, and the modlog — every moderation action recorded,
readable by every user.

**Part 12 — The frontend.** SvelteKit 5 runes, the i18n system in six
languages, the offline PWA, the admin UI, and the design tokens that
hold it all together.

**Part 13 — This part.** Tests that lock everything in, a DB-gated
harness that lets a hundred integration tests share one database, a
deployment on a ThinkCentre with systemd and nightly backups and
migrations on boot, a git-worktree workflow that survives an NFS
filesystem that fights back, and a roadmap that says what's next.

That's the whole arc. And here's the part that matters: *none of it was
a single heroic leap*. It was sixty-two chapters of showing up. A
scraper here, a migration there, a test for the bug that bit you
yesterday. That's what building software actually is — and now you've
lived it, end to end, with a real project that runs at
fichub.polarisocial.xyz for real readers.

### What you actually learned

Let's be honest about the skills, because they're the real deliverable
of this book. You didn't just learn Rust and Svelte — you learned how
a *whole system* fits together, and how to keep it alive:

- **The full stack, end to end.** HTTP request → router → handler →
  SQL → JSON → Svelte component → DOM. You can trace a reader's click
  from the browser all the way to a row in PostgreSQL and back. That
  ability to *trace the whole path* is the single most valuable
  skill in this book, and most developers never get it because they
  never build both halves.
- **Rust in anger.** Ownership, the borrow checker, async runtimes,
  error types, `Option` and `Result` discipline, the standard library,
  sqlx with compile-checked queries, tokio tasks, mutexes and
  `OnceLock`s. You've read more real Rust than most juniors write in
  their first two years.
- **Databases for real.** Migrations, schemas that evolve over 38
  versions, indexes, `ON CONFLICT`, transactions, search over
  thousands of rows, and the discipline of never hand-applying a
  migration to production.
- **Testing as a craft.** Unit tests, contract tests, page tests,
  DB-gated integration suites with mutexes and unique seeds, coverage
  gates that protect baselines, and the rule that every fixed bug
  adds a regression test. Over 1,300 tests now guard this project.
- **Operations.** systemd units and timers, health checks, backups,
  deploy scripts that build on the fast machine and serve from the
  slow one, handover documents, and the art of making every failure
  visible.
- **Debugging at depth.** The `redis:false` BRPOP bug, the NFS
  loose-object EACCES traced through strace, the `core.worktree`
  config leak, the migration-ownership trap, the incremental-
  compilation fingerprints that lie. You've seen what real debugging
  looks like: form a hypothesis, gather evidence, find the layer that
  actually misbehaved, fix it, write it down.
- **And the meta-skills.** Writing things down. Documenting the
  quirks. Commenting the *why*. Keeping the roadmap honest. Making
  the smallest change that fixes the problem. Never deleting or
  weakening a test.

> 💡 **Key Concept — You now know the difference between a demo and a
> platform.** A demo works on your laptop when you demonstrate it. A
> platform works at 3 a.m., after a reboot, with a database to
> migrate, a disk to back up, and a roadmap deciding what ships next.
> Every part of this book past Part 1 was about crossing that line —
> and you crossed it. When you look at any other project now, you'll
> see it differently: you'll see the tests it's missing, the
> handover doc it doesn't have, the single point of failure it's
> ignoring. That's not cynicism. That's production awareness, and it
> only comes from having shipped something.

### What you built, quantified

Before we say goodbye, let's count what's sitting in that repo — the
things that used to be "hard" and are now just... done:

- **107 site scrapers** as native Rust adapters, full FanFicFare parity —
  login support, adult-site gating, turbo-stream decoders for SoFurry.
- **7 export formats** — EPUB, HTML, MOBI, PDF, AZW3, TXT, MD.
- **38 migrations** taking the schema from an empty `request_source`
  table to a full community platform.
- **A search engine** with boolean operators, fields, typo tolerance,
  and "Dark Harry" semantics.
- **A recommender platform** with 8 strategies, shadow mode, and an
  impressions ledger.
- **An AI layer** — NL search, auto-tagging, translation, self-healing —
  running on a local Ollama model, no cloud bills.
- **A trust layer** — honeypots, PoW, shadowbans, a transparent
  modlog, peer-voted curator fixes.
- **~1,300 tests** and a QA harness that walks every API route.
- **A live site** on a ThinkCentre, behind a Cloudflare tunnel, with
  nightly backups and migrations on boot.

None of those numbers existed when we started. Every single one of
them is the sum of small, ordinary commits — the kind you can make.
That's the real message of this book, and it's worth saying plainly:
**you just spent thirteen parts doing the work, and the work is what
made the platform.** Not a genius moment. Sixty-two chapters of
showing up.

> 🧪 **Try It Yourself — The last exercise in the book.** Go to
> fichub.polarisocial.xyz (or your own instance, if you deployed
> your own). Find a fic, read a chapter in the reader, export it as
> an EPUB, open the modlog, look at the roadmap arena, search for a
> phrase you remember from the fic. Click through everything you
> built in this book. Then open `docs/ROADMAP.md` and pick *one* P4-P7
> item that sounds fun, and write a paragraph about how you'd
> implement it — what's already true, what you'd need, what would
> make it safe. You don't have to build it today. But the fact that
> you *can* picture the implementation — that's the whole book
> working.

### A few words for the road

If this is the end of your journey with FicHub, thank you for reading
all the way through. If it's the beginning of your own project — the
one you've been quietly sketching while reading this — then here's the
only advice that matters, and it's the same advice FicHub's own
history proves: **start with something that works, then keep showing
up.** Your first migration will be ugly. Your first scraper will break.
Your first deploy will teach you something you couldn't have learned
any other way. None of that is failure. It's the raw material of the
thing you're building.

The roadmap says the site is "close to small-cohort invite
readiness." The body cache holds fics scraped from sites that may
never be reachable again. The nightly backup runs at 03:30. The
health endpoint answers on a dedicated Redis connection. Somewhere, a
reader is about to open an EPUB that came off a ThinkCentre in a
corner of someone's home — an EPUB that exists because a scraper
visited a source site, a body was cached to disk, a handler built a
file, and a tunnel carried it across the internet. And you know every
step of that story, because you built it.

So here it is, at last, the sentence this whole book has been walking
toward:

**Congratulations — you built FicHub.**

Not "we showed you how FicHub was built." Not "here is some code that
resembles FicHub." You read the real source, you traced the real
paths, you learned the real scars — and in every way that matters for
a developer, you *built* it. The repo is yours to understand, to
extend, to break and fix and improve. The roadmap's P1 cookie
ingestion is waiting. The full-text quote search is waiting. The
small cohort is waiting.

Go build something. Go make it real. And when the environment fights
back — and it will, because environments always fight back — remember
everything you learned here: trace the syscalls, verify before you
panic, write it down, and keep showing up for the next chapter.

The finish line wasn't the end. It was the starting line for
everything you build next.
