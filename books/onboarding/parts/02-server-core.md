## Part 2 — The Server Core

### Chapter 5: Entry Point & Module Tree

Open `src/main.rs`. It is deliberately small:

```rust
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let config = fichub::config::Config::from_env();
    fichub::server::run(config).await;
}
```

That's essentially the whole binary. Three lines of substance:

1. **`#[tokio::main]`** — the attribute that turns `main` into a tokio
   async runtime. Tokio is the async executor that makes thousands of
   concurrent connections possible on a handful of OS threads.
2. **`tracing_subscriber::fmt().init()`** — sets up structured logging.
   You'll see `tracing` and `log` macros throughout the codebase; this line
   is why `RUST_LOG=debug` actually shows you things.
3. **`Config::from_env()` then `server::run(config)`** — the entire
   startup flow lives in the library crate, not here.

Why keep `main.rs` boring? Because a boring entry point is a *readable*
entry point. Anyone — including a junior dev on day one — can see exactly
what the binary does: read config, run the server. All the interesting work
is importable and testable.

Now open `src/lib.rs`. It declares the module tree:

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod limiter;
pub mod modlog;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod works;
// ... plus body_cache, heal, etc.
```

Two things to notice:

- **Everything is `pub`** — that's what makes `fichub::routes::admin::...`
  reachable from integration tests.
- **The module list is your table of contents.** If you want to know where
  something lives, this list plus `src/routes/mod.rs` is your index.

The crate is a *library + a thin binary* — the "two-crate trick." Integration
tests in `tests/` use the library crate's public API, so they test the real
server, the real config, the real queries. This is a hugely underrated
design decision: it means you can write end-to-end-ish tests in plain Rust
with `cargo test`, no separate test harness.

> 💡 **Key concept:** the two-crate split. `main.rs` = boot. `lib.rs` =
> everything. Tests import the library. If you're tempted to put logic in
> `main.rs`, don't — put it in the library so it's testable.

### Chapter 6: AppState — the Dependency-Injection Heart

Every handler in this codebase receives the same shared state. It's defined
in `src/server.rs`:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: Arc<RedisPool>,
    pub health_redis: RedisPool,      // dedicated conn for health checks
    pub http_client: reqwest::Client,
    pub scraper_registry: ScraperRegistry,
    pub cache_semaphores: CacheSemaphores,
    // ... heal state, suggest cache, worker handles, etc.
}
```

Handlers grab it with Axum's `State` extractor:

```rust
pub async fn health_handler(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let db_ok = state.db.acquire().await.is_ok();
    // ...
}
```

Why `Arc<AppState>`? Because every request shares the same state; `Arc`
(the atomic reference-counted pointer) lets thousands of concurrent
requests hold a cheap shared reference. Nothing is per-request except the
extractor itself.

A few fields deserve explanation:

- **`config`** — the frozen config snapshot. Handlers read settings from
  here rather than re-reading env vars.
- **`db`** — the SQLx connection pool. `PgPool` internally manages a set of
  connections; `state.db` is cheap to clone (it's an `Arc` under the hood).
- **`redis` vs `health_redis`** — here's the story I promised in Chapter 4.
  The **bookmark-import worker parks an unbounded BRPOP on the shared
  `state.redis` connection**. BRPOP blocks the connection waiting for list
  items. If a health check PINGs *that* connection, the PING queues behind
  the BRPOP and times out — falsely reporting "Redis down." That's why
  `health_redis` exists: a dedicated connection for health checks, never
  touched by the worker. **Lesson: never PING the shared connection from a
  health check.**
- **`http_client`** — a shared `reqwest::Client` with connection pooling,
  used by scrapers and the LLM feature.
- **`scraper_registry`** — the list of site adapters (Part 4 covers this in
  depth).
- **`cache_semaphores`** — the bounded map that prevents duplicate
  concurrent exports (Part 5).

`src/server.rs` also contains:

- **`run()`** — builds the pool, connects Redis, spawns background workers
  (recommender worker, bookmark importer), then serves.
- **`build_router()`** — the big route-registration function. Read it as
  your feature map: every endpoint in the product is listed there, grouped
  by area.

> 🧪 **Try it:** `grep -n "route(" src/server.rs | wc -l` — count the
> routes. Then open `build_router()` and find the route for the feature you
> care about most.
>
> ⚠️ **Watch out:** when you add a new subsystem, it goes into `AppState`
> — and then **every** literal `AppState` construction in tests must be
> updated. The skill notes literally say "every literal construction in
> tests needs the new field or they won't compile." This is annoying but
> safe: the compiler tells you exactly which test files need the field.
>
> 💡 **Key concept:** dependency injection via a shared state struct +
> `State` extractor. New subsystems = new `AppState` field + new handler
> access. Keep the pattern; don't invent global singletons.

### Chapter 7: The Router — Every Route Registered

`build_router()` in `src/server.rs` chains `.route()` calls. The path syntax
uses `{param}` (Axum 0.8's syntax). Examples you'll see constantly:

```rust
.route("/api/epub", get(export::epub_handler))
.route("/api/search", get(search::search_handler))
.route("/api/requests/{id}/candidates", get(requests::candidates))
.route("/api/works/{url_id}/also-bookmarked", get(works::also_bookmarked))
```

Two things to internalize:

1. **Order matters.** Some routes are registered before the `ServeDir`
   fallback that serves the SPA. API routes must win over the fallback. If
   you add a route and the SPA swallows it, check the order.
2. **The SPA fallback.** Any path that doesn't match an API route gets
   `index.html` — that's how `/read/xyz` works in the browser while
   `/api/*` stays JSON. This is the "one origin" idea made concrete.

Route naming conventions:

- `/api/<area>` for the main endpoints.
- `/api/v1/...` for a few versioned legacy endpoints (e.g.
  `/api/v1/works/{url_id}/also-bookmarked`).
- `/api/admin/<area>` for admin-only endpoints (role ≥ 10).
- Static files served from `frontend/build` via ServeDir.

When you add a feature, you typically touch four places: `AppState` (if new
state), a handler in `src/routes/<area>.rs`, a `.route()` in
`build_router()`, and a frontend page. We'll do this end-to-end in
Chapter 22.

> 🧪 **Try it:** add a trivial route in your head: `GET /api/ping` returning
> `{"pong":true}`. Find where you'd register it. (Hint: `build_router` +
> a handler in `src/routes/health.rs` or a new file.)
>
> ⚠️ **Watch out:** path param syntax changed across Axum versions. This
> repo uses `{param}` (Axum 0.8), not `:param`. If you copy old Axum code
> with `:id`, it won't compile.

### Chapter 8: Errors That Don't Crash

Open `src/error.rs`. This file defines the backbone of every handler's
return type.

```rust
pub enum AppError {
    NotFound,
    BadRequest(i32, String),      // custom code + message
    Unauthorized,
    Database(sqlx::Error),
    Network(String),
    RateLimitedJson(serde_json::Value),  // 429 with a custom JSON body
    // ... more variants
}

pub type AppResult<T> = Result<T, AppError>;
```

The magic is the `IntoResponse` impl:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // translate AppError → HTTP status + JSON body {err, msg}
    }
}
```

Handlers return `AppResult<T>`. The `?` operator converts anything that
implements `From<AppError>`-compatible conversions — sqlx errors, reqwest
errors, etc. — via `From` impls in this same file. **A handler can never
panic from a failed DB query**: it becomes a clean JSON error instead.

The JSON error shape is the same everywhere:

```json
{"err": -6, "msg": "network error: ..."}
```

Negative error codes are custom (e.g. `-6` network, `-403` admin access
required, `-429` rate limited). The frontend's API client checks `err != 0`
and shows `msg`.

Three rules to live by when you touch errors:

1. **Return `AppResult<T>`, let `?` do the work.** Don't hand-map every
   failure; add a `From` impl when you introduce a new error source.
2. **Details in, generic out.** Log the real error (tracing), return a safe
   message. Never leak internal paths, credentials, or SQL.
3. **Custom codes are a contract.** The frontend and the API tests depend
   on specific `err` values. Change codes deliberately, with tests.

> 🧪 **Try it:** `curl localhost:8000/api/meta?q=https://archiveofourown.org/works/21845264`
> — a real scrape. Then try a bogus URL and watch the `{err, msg}` JSON.
> Notice the shape stays consistent.
>
> ⚠️ **Watch out:** the `?` operator needs the `From` impls. If you return
> `AppResult` from a handler that calls a function returning `sqlx::Error`,
> make sure `From<sqlx::Error> for AppError` exists (it does).
>
> 💡 **Key concept:** errors are values, not panics. `AppError` + `From` +
> `?` + `IntoResponse` = handlers that read like happy-path code but fail
> gracefully everywhere.

### Chapter 8A: Middleware — What Happens Before/After Handlers

Axum middleware wraps the router and runs before (and after) your handler.
FicHub layers several:

1. **Tracing** — logs every request with method, path, status, duration.
2. **Usage analytics recorder** (`from_fn_with_state`) — classifies each
   path as `view` or `action` and writes to `usage_events` (with the
   anonymous X-Client-ID). Health = view; meta = action.
3. **Rate limiting** — the tiered limiter (per-IP + per-client token
   buckets).
4. **CORS** — though with the one-origin architecture, CORS is mostly
   moot in production.
5. **The SPA fallback** — ServeDir at the end.

Middleware order matters: tracing first (see everything), then analytics
(measure everything), then rate limiting (protect), then routes, then the
fallback.

When you add middleware, think about *what it needs to know*: request
metadata (path, method, headers) → run before; response status → run
after. The `from_fn_with_state` middleware pattern in this repo is a good
template:

```rust
pub async fn track_usage(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    let client_id = req.headers().get("x-client-id")...;
    let resp = next.run(req).await;
    // classify + record
    state.analytics.record(&path, &client_id, &resp).await;
    resp
}
```

> 🧪 **Try it:** read the analytics middleware in `src/routes/analytics.rs`
> and identify the view vs action classification.
>
> ⚠️ **Watch out:** middleware runs on EVERY request — keep it cheap. A
> slow middleware becomes a global latency tax.
>
> 💡 **Key concept:** middleware is where cross-cutting concerns live:
> logging, analytics, rate limiting. Order = visibility → measurement →
> protection.

### Chapter 8B: The Health Check Dissected

`src/routes/health.rs` is small but teaches several lessons. The handler:

1. Acquires a DB connection (`state.db.acquire()`) → `db: bool`.
2. PINGs the dedicated `health_redis` connection → `redis: bool`.
3. Returns `{"status":"ok","db":true,"redis":true,"version":"0.2.0"}`.

Why the dedicated Redis connection (again, because it's important): the
bookmark-import worker parks an unbounded BRPOP on the shared `state.redis`
connection. A PING on that connection would queue behind the BRPOP and
time out — falsely reporting Redis down. `health_redis` is a separate
connection reserved for health checks. **Rule: health checks must never
share connections with workers.**

The version string comes from the crate version (Cargo.toml). The status
is "ok" only when both DB and Redis report healthy; degraded components
flip their boolean so monitoring can see exactly what's wrong.

> 🧪 **Try it:** read `health.rs` and trace where `version` comes from.
> Then run `curl localhost:8000/api/health` and correlate the booleans
> with the actual DB/Redis state.
>
> ⚠️ **Watch out:** never add slow checks to the health endpoint — it's
> polled by uptime monitors and the deploy script. Keep it fast.
>
> 💡 **Key concept:** the health check is the contract with monitoring:
> fast, honest, and connection-safe (dedicated `health_redis`).

---
