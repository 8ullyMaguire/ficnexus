# Part 2 — Laying the Rust Foundation

> **Part 2 of 13** — In Part 1 you ran the whole platform on your machine and
> watched the terminal print `Listening on 0.0.0.0:3000`. That line is the end
> of a long chain of setup — and this part is about that chain. We're going to
> read the four files that form the spine of the entire backend, line by line:
> `main.rs` (where the program starts), `lib.rs` (the library that tests
> against), `server.rs` (where the state and the router live), and `error.rs`
> (where mistakes become polite JSON instead of crashes). By the end of Part 2
> you'll be able to explain, with your eyes closed, exactly what happens
> between you typing `cargo run` and a browser tab loading FicHub.

---

## Chapter 5 — The Entry Point: main.rs and lib.rs

Close your eyes for a second. Think about the last time you ran `cargo run` in
the FicHub repo. A wall of warnings scrolled past, maybe some compiling
spinners, and then, at the very bottom:

```
Listening on 0.0.0.0:3000
```

That one line is the *end* of a story that starts in the smallest, simplest
file in the whole backend. Let's open it.

### 5.1 The file where everything starts

Every Rust binary has exactly one entry point: a function named `main`. When
you run `cargo run`, Cargo compiles the crate, hands the resulting executable
to the operating system, and the OS says "start executing at `main`." So the
question "where does FicHub begin?" has a literal answer: `src/main.rs`.

Here is the entire file. All forty-two lines of it. Read it once all the way
through, and then we'll take it apart line by line:

```rust
pub mod body_cache;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod fic_suggestions;
pub mod frontend;
pub mod heal;
pub mod ingest;
pub mod limiter;
pub mod modlog;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod tags;
pub mod works;

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

That's the whole entry point. Twenty module declarations, one `main` function,
four steps. Let's go through it in order, because *order is the plot* here.

### 5.2 The module tree: twenty doors into the codebase

The first twenty lines are `pub mod` declarations. In Rust, a `mod` declaration
tells the compiler "there is a module with this name, find it and include it."
When you see `pub mod db;`, the compiler looks for either `src/db.rs` or
`src/db/mod.rs` and makes everything public inside it available as
`crate::db`. Every folder you walked through in Part 1's repo tour has a door
here: `db`, `scrape`, `routes`, `recommender`, `search`, `cache`, `heal`,
`tags`, `works`, `services`, `limiter`, `modlog`, `export`, `frontend` — plus
a few single-file modules like `config`, `error`, `body_cache`, and
`fic_suggestions`.

Why is this interesting? Because this list *is* the architecture. Look at it
as a sentence: FicHub is a server (`server`) with configuration (`config`),
error handling (`error`), a database layer (`db`), caching (`cache`,
`body_cache`), scraping (`scrape`), exporting (`export`), search (`search`),
recommendations (`recommender`), healing (`heal`), community features
(`routes`, `modlog`, `works`, `tags`, `fic_suggestions`), and infrastructure
(`services`, `limiter`, `ingest`, `frontend`). When you're lost in a Rust
codebase, the module tree at the top of `main.rs` is your map — every
subsystem gets one line.

💡 **Key Concept — The module tree is the architecture**
In Rust, `pub mod X;` at the top of `main.rs` is how you declare, in one
place, every subsystem your program has. This is different from languages
where files are just "there" — here the compiler *requires* you to declare
the tree explicitly. That's a feature: the moment you look at a new Rust
project, you read its skeleton in about five seconds. The same list appears
in `lib.rs` (we'll see why in a moment), and the two lists are the
project's table of contents.

### 5.3 `#[tokio::main]`: the async runtime

Now the real fun starts. Look at the line above `async fn main()`:

```rust
#[tokio::main]
async fn main() {
```

Two things are going on. First, `main` is `async` — it can `await` things.
But the operating system doesn't know anything about `async`; it expects a
synchronous `fn main() -> ()`. So how does an `async fn main` even compile?

That's what `#[tokio::main]` is for. It's a *procedural macro* — a piece of
code that runs at compile time and rewrites your function into something
else. When you write:

```rust
#[tokio::main]
async fn main() { ... }
```

the macro expands it into roughly:

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { ... })
}
```

In plain English: create a Tokio runtime (Tokio is the async runtime library
in the `[dependencies]` section of `Cargo.toml`), then block the main thread
while the async body runs on the runtime. Everything after this point — every
database query, every HTTP request, every scraper call — runs inside Tokio's
world of tasks and `await` points.

Why do we need this at all? Because serving a web platform is *I/O-bound*: we
spend almost all our time waiting — waiting for PostgreSQL to answer, waiting
for Redis, waiting for a scraper to fetch a page, waiting for a network
request. While we wait, Tokio can run *other* tasks on the same thread
instead of sitting idle. That's the "async" in async Rust: one thread, many
concurrent tasks, each paused at its `await` points, all sharing the thread
fairly. The whole backend is built on this — every handler you'll meet in
later parts is an `async fn`.

### 5.4 Step one: load `.env`

```rust
    // Load .env if present
    dotenvy::dotenv().ok();
```

You might remember from Part 1 that FicHub's configuration lives in
environment variables — `DATABASE_URL`, `REDIS_URL`, `PORT`, and dozens more.
But typing `export DATABASE_URL=...` into a terminal every time you start the
server would be miserable. So the project uses a `.env` file in the repo
root, and `dotenvy` is the tiny library that reads it and stuffs every
`KEY=VALUE` line into the process's environment *before* the rest of the
program runs.

Two details matter here. First, the `.env` file is optional: `.ok()` converts
the `Result` into an `Option` and throws the error away. If there's no `.env`
file (say, in production, where environment variables come from systemd),
`dotenv()` fails — and we don't care, because the variables are already
there. Second, the *order* matters: `dotenv` must run before `Config::from_env()`
reads the variables. That's why the boot sequence is a sequence.

⚠️ **Watch Out — `.ok()` means "I don't care if this fails"**
`dotenvy::dotenv()` returns a `Result`. `.ok()` converts it to `Option`
and *discards the error*. That's deliberate here — a missing `.env` is a
normal situation, not a crash. But the same pattern can hide real
problems: if you ever write `.ok()` and the thing you're ignoring is
actually critical, you'll get confusing "missing variable" panics later
instead of a clear "couldn't read .env" error now. Always ask: *is this
failure a normal case, or a bug?* Normal case → `.ok()` or `unwrap_or`.
Bug → `expect` with a message.

### 5.5 Step two: turn on the lights (logging)

```rust
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
```

This block sets up **tracing**, the logging framework the whole project uses.
You've already seen its output — every `Listening on...` line, every
`tracing::info!` message you watched scroll by in Part 1 came from here.

Let's read it from the inside out. `EnvFilter::try_from_default_env()` asks
the environment for a `RUST_LOG` variable that says *how verbose* logging
should be ("info", "debug", "warn", and so on). If `RUST_LOG` isn't set, the
`unwrap_or_else` fallback kicks in: `EnvFilter::new("info,fichub=debug")`.

That fallback string is worth reading carefully because it's a mini-language:
`info` means "show info-level messages and above from everything," and
`fichub=debug` means "but for the `fichub` crate specifically, show debug
messages too." The project logs at `debug` level for its own code — the
detailed, chatty messages that help you trace what a request is doing — while
keeping third-party libraries at the quieter `info` level. Then
`tracing_subscriber::fmt().with_env_filter(...).init()` wires that filter
into the subscriber that formats and prints log lines to stdout.

💡 **Key Concept — Log levels are a debugging dial**
`error` < `warn` < `info` < `debug` < `trace`. Your server is running fine
and you want *more* insight into a bug? Run with `RUST_LOG=fichub=debug
cargo run` and every `tracing::debug!` line in the codebase suddenly
speaks. Want it quiet in production? Set `RUST_LOG=warn`. The code doesn't
change — you just turn the dial. FicHub's code is full of `tracing::info!`
and `tracing::debug!` calls (you'll see a `tracing::error!` in Chapter 8
that matters a lot), and they're all governed by this one setup block.

### 5.6 Step three: load the configuration

```rust
    // Load configuration
    let config = config::Config::from_env();
```

One line, enormous consequences. `Config::from_env()` reads *every* relevant
environment variable and assembles them into one typed `Config` struct — the
database URL, the Redis URL, the port, the frontend directory, the rate
limiting knobs, the Ollama URLs, the SMTP settings, and roughly a hundred
more fields. We're going to spend an entire part (Part 3, Chapter 9) inside
this function, so for now all you need is the shape of the thing: config is
*one value* that describes *everything the server needs to know about its
world*, loaded once, at boot.

Notice the type annotation: `let config: Config`. It's `config::Config` from
the module we declared in line 3. The `::` path — `config::Config` — is the
same path syntax you'll see everywhere in Rust: module, then item.

### 5.7 Step four: announce, then hand off

```rust
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
```

Two final lines. The `tracing::info!` prints the boot message with the actual
port interpolated in — that's the `{}` placeholder, Rust's formatting syntax.
Then the *entire* `main` function hands the config to `server::run(config)`
and `await`s it.

This is the hand-off. Everything we've done so far — runtime, logging,
config — was preparation. The actual server — connecting to databases,
building state, registering routes, accepting connections — lives in
`server::run`, and it's the subject of Chapters 6 and 7. `main` doesn't just
call it; it *awaits* it, because `run` never returns until the server stops.

Also note: `main` is *thin*. It's four steps and a hand-off. This is a
deliberate, industry-standard shape: the entry point should be so small that
you can understand the whole boot sequence in one glance. All the real
complexity is pushed into functions with names that say what they do —
`Config::from_env()`, `server::run()`.

### 5.8 But wait — where's lib.rs?

We opened `main.rs`, but every Rust project you'll meet has *two* top-level
files in `src/`: the binary's `main.rs` *and* the library's `lib.rs`. FicHub
has both. Here's the entire `src/lib.rs`:

```rust
pub mod body_cache;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod heal;
pub mod fic_suggestions;
pub mod ingest;
pub mod limiter;
pub mod modlog;
pub mod recommender;
pub mod roadmap_seed;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod services;
pub mod tags;
pub mod works;

/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

Almost the same module list — with `roadmap_seed` added, and three lines at
the bottom that aren't `pub mod` at all. What's going on?

**The two-crate setup.** When Cargo builds this project, it actually builds
two crates from `src/`: a *library crate* from `lib.rs`, and a *binary crate*
from `main.rs`. The binary's `main.rs` declares `pub mod db;` etc. — but here
those declarations aren't *defining* fresh modules, they're re-declaring the
same source files as part of the binary crate. Both crates can see
`src/db.rs`; each just has its own view of it.

Why would you split a project like this? **Testing.** Remember how the
binary's `main` has to be an `async fn` with a runtime? Testing code that
lives in a binary crate is awkward — you can't `use` a binary crate as a
library. But you *can* `use` a library crate. By putting all the real code in
`lib.rs`'s crate (and only a thin `main` in `main.rs`'s crate), the project
can write integration tests in `tests/` that `use fichub::...` and call real
functions — the ones re-exported at the bottom of `lib.rs`.

Look at those re-exports again:

```rust
/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

`pub use` means "make this item available at this path, as if it were
defined here." The integration tests (in `tests/`, which we'll meet properly
in Part 13) can now write `use fichub::generate_slug;` instead of having to
reach through `fichub::routes::export::generate_slug`. It's a tiny
convenience, but it's also a statement: *these* functions are the public
face the tests rely on.

🧪 **Try It Yourself — Make the boot sequence talk**
1. Open `src/main.rs`. Change the boot message to something personal:
   `tracing::info!("{}", "My server is ALIVE");` (or just edit the
   existing line's text). Run `cargo run` and watch your message appear
   after the compile finishes.
2. Now turn the logging dial: run with `RUST_LOG=warn cargo run` (yes,
   `RUST_LOG` as a prefix to the command — that sets the env var for just
   that process). Your `info!` message *disappears*, because `warn` is
   quieter than `info`. Run again with `RUST_LOG=fichub=debug cargo run`
   and watch the much noisier debug output appear.
3. Bonus: move the `dotenvy::dotenv().ok();` line *below* the config load
   (temporarily), then run. You'll get a panic about `DATABASE_URL` — proof
   that order is the plot.

### 5.9 What the compiler knows: Cargo.toml's role

One more piece of the boot puzzle, and it's in a file you've seen but maybe
haven't read: `Cargo.toml`. Two details there matter for the boot sequence:

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
# The web server (src/main.rs) is the binary `cargo run` should pick when no
# --bin is given. The [[bin]] entries below are one-shot CLI tools. Explicit
# default-run keeps `cargo run` (and the hermes verify boot phase) from
# erroring on multiple binaries.
default-run = "fichub"
```

The comment says it all. This project has *more than one binary* — the
`[[bin]]` entries further down declare one-shot CLI tools like
`assign-quests`, `compute-stats`, `compute-leaderboards`, `bot-scorer`,
`backfill-scores`, and `seed-roadmap`. When Cargo sees multiple binaries, a
bare `cargo run` would be ambiguous — which one? The `default-run = "fichub"`
line answers: *the web server*. So `cargo run` always boots the server, and
the CLI tools run explicitly via `cargo run --bin compute-stats`.

Also worth noticing: `edition = "2024"`. Rust editions are the language's
versioning scheme for breaking changes — "2024" is the current one, and it
affects things like how `async fn` in traits work and which keywords are
reserved. The code you're reading throughout this book is written against it.

### 5.10 The boot sequence, end to end

Put it all together, and this is the story of every `cargo run`:

```
cargo run
  └─ Cargo reads Cargo.toml → default-run = "fichub" → builds src/main.rs
        └─ OS starts executing fn main
              └─ #[tokio::main] expands → Tokio multi-threaded runtime starts
                    └─ 1. dotenvy::dotenv().ok()        → .env loaded into env
                    └─ 2. tracing_subscriber::fmt().init() → logging lights up
                    └─ 3. config::Config::from_env()    → every knob read
                    └─ 4. tracing::info!("Starting fichub-rs server...")
                    └─ 5. server::run(config).await     → never returns
                          └─ (Chapters 6–7: databases, state, router, serve)
```

Notice how each step *depends* on the previous one: logging needs the runtime;
config needs the environment (`.env` loaded in step 1); the server needs the
config. Fail any earlier step and the later ones panic loudly — that's
fail-fast, and it's why your Part 1 troubleshooting table worked: the panic
message told you *which* subsystem to fix.

`main.rs` is done. But it handed everything to `server::run` — and that
function contains the most important data structure in the entire backend.
It's time to meet `AppState`.

---

## Chapter 6 — AppState: The Dependency-Injection Heart

Every handler in this codebase — every function that answers an HTTP request
— needs access to *things*: the database, Redis, the scraper registry, the
rate limiter, config, and more. A bookmark handler needs `state.db`. A
recommendation handler needs the engine. The health endpoint needs its own
private Redis connection. How does all of that get from "created once at
boot" to "available inside every handler"?

The answer is one struct, and it's the single most important type in the
backend. Meet `AppState`, straight from `src/server.rs`:

```rust
/// Shared application state accessible by all handlers
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    /// Dedicated Redis connection for `/api/health` PING checks. Kept
    /// separate from `redis` because the bookmark-import worker blocks on a
    /// shared multiplexed connection with an unbounded BRPOP; a PING queued
    /// behind that BRPOP would time out and make health report `redis:false`
    /// even though Redis is fine. Health also wraps its PING in a short
    /// timeout so a stuck Redis can never hang the endpoint.
    pub health_redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::TieredRateLimiter>,
    pub recommender_engine: RecommendationEngine,
    /// Pluggable recommendation platform: config-driven strategy registry
    /// (REC_ENGINE_MODE=pluggable). In legacy mode the handlers route around
    /// it — the registry is still built so `/api/recommendations/strategies`
    /// works and the pipeline worker has something to run.
    pub strategy_registry: crate::recommender::registry::StrategyRegistry,
    pub collection_worker: CollectionWorker,
    /// Self-healing telemetry + agent-run persistence.
    pub heal: crate::heal::HealService,
    /// 300s cache for the non-personalized popular suggestions
    /// (`GET /api/search/suggest`). Keyed by nothing: the popular list is
    /// global and only reflects `q`/`tag_type_id`-independent data once the
    /// query params become part of the SQL. Personalized requests bypass it.
    pub suggest_cache: Arc<tokio::sync::Mutex<Option<(std::time::Instant, Vec<serde_json::Value>)>>>,
    /// Ollama embeddings client for the Roadmap Consensus Engine.
    pub ollama: crate::services::ollama::OllamaClient,
    /// Email transport for Send-to-Kindle. `SmtpMailer` in production,
    /// `MockMailer` in tests (recorded sends, no relay needed).
    pub mailer: Box<dyn crate::services::mailer::Mailer>,
}
```

### 6.1 What this struct really is

Let's translate. `AppState` is a struct — a bundle of named fields — that
holds *one of everything the server needs to do its job*. Read the fields as
a shopping list:

- `config: Config` — the same Config from Chapter 5, cloned in so handlers
  can read any knob (frontend dir, rate-limit tiers, port).
- `db: sqlx::PgPool` — a PostgreSQL *connection pool*, not a single
  connection. SQLx keeps a pool of ready connections so twenty requests can
  query the database concurrently without each opening its own connection.
- `redis: redis::aio::MultiplexedConnection` — one async Redis connection
  shared by all the normal traffic.
- `health_redis` — a *second* Redis connection, for health checks only, with
  a fascinating reason we'll unpack in 6.3.
- `http_client: reqwest::Client` — the HTTP client the scrapers use to fetch
  pages. `reqwest::Client` is itself a pool: it reuses TCP connections, so
  scraping fifty stories doesn't open fifty sockets.
- `scraper_registry: Arc<ScraperRegistry>` — the registry of scrapers (AO3,
  FFN, RoyalRoad, XenForo, the FanFicFare catch-all). Note the `Arc`.
- `cache_semaphores` — a concurrency limiter for cache writes.
- `rate_limiter: Box<dyn limiter::TieredRateLimiter>` — the anti-bot token
  buckets. Note the `Box<dyn ...>`.
- `recommender_engine`, `strategy_registry`, `collection_worker` — the
  recommendation platform's pieces.
- `heal: HealService` — self-healing telemetry.
- `suggest_cache` — a 300-second cache for popular search suggestions.
- `ollama: OllamaClient` — the client for local LLM embeddings.
- `mailer: Box<dyn Mailer>` — email transport for Send-to-Kindle.

Now, the key insight: **this struct is dependency injection.** That's a
fancy term that means "instead of every function creating what it needs, you
create everything once, put it in a box, and *hand the box to whoever asks*."
Handlers don't construct their own `PgPool` — they receive a shared one from
state. The server owns exactly one `AppState`, and every handler borrows it.

That single-owner design has three superpowers:

1. **Created once.** Database pools, HTTP clients, Redis connections — these
   are expensive to create and cheap to share. One `AppState` means they're
   each built exactly once, at boot.
2. **Configured once.** The rate limiter is constructed with the config's
   tier settings in `server.rs`; every handler that uses it gets the *same*
   configured instance. There's no way for a handler to accidentally create
   a second limiter with different settings.
3. **Testable.** In tests, you build an `AppState` with a test database, a
   mock mailer, whatever you like — and hand *that* to the same handlers.
   The comment on `mailer` says it explicitly: "`SmtpMailer` in production,
   `MockMailer` in tests (recorded sends, no relay needed)."

### 6.2 Why so many `Arc` and `Box`?

Two of those field types deserve their own moment, because they answer the
question "what type do I write when multiple things need to share one
thing?" — one of the most common questions in real Rust.

**`Arc<T>` — shared ownership.** `Arc` stands for *Atomically Reference
Counted*. It's a smart pointer: when you clone an `Arc`, you don't copy the
data — you copy a pointer and bump a counter. The data lives as long as any
clone exists, and the last clone to drop frees it. That's exactly the
semantics you want for state: the server holds one `Arc<AppState>`, clones
it into the router, clones it into background tasks, and every clone points
at the *same* struct in memory.

Look at how the field types use it: `scraper_registry: Arc<ScraperRegistry>`
is shared by every export handler (and the collection worker); the
`cache_semaphores` field wraps a `HashMap` in an `Arc<tokio::sync::Mutex<...>>`.
Why a `Mutex` inside the `Arc`? Because `Arc` gives you shared *ownership*
but not shared *mutation* — multiple threads can't write the same `HashMap`
at once. The `Mutex` adds the "only one writer at a time" rule, and the
`Arc` adds the "everyone can hold it" rule. Together: a cache that every
request can read and update safely. That pairing — `Arc<Mutex<T>>` — is one
of the most common type signatures in all of Rust, and you'll see it again
in the `suggest_cache` field.

**`Box<dyn Trait>` — one interface, many implementations.** `rate_limiter:
Box<dyn limiter::TieredRateLimiter>` means "a box containing *some type* that
implements the `TieredRateLimiter` trait." The server doesn't care *which*
limiter it is — a Redis-backed bucket limiter in production, a fake in tests
— only that it behaves like a `TieredRateLimiter`. That's the `dyn Trait`
part: dynamic dispatch, the same idea as an interface in other languages.
The `Box` part is about size: trait objects don't have a known size at
compile time (different implementors are different sizes), so they live on
the heap behind a box. Same story for `mailer: Box<dyn Mailer>` — one field,
two possible implementations, chosen by the caller (production `SmtpMailer`
vs test `MockMailer`).

💡 **Key Concept — Three ways to share, and when to use which**
- `T` (plain value): one owner. The `Config` in `AppState` is a plain
  value — cloned when needed.
- `Arc<T>`: many owners, read-only (or with interior mutability).
  `Arc<AppState>` itself, `Arc<ScraperRegistry>`.
- `Box<dyn Trait>`: one owner, but the concrete type is hidden behind a
  trait. `rate_limiter`, `mailer`.
The `?Sized`-ness of trait objects is why `dyn` types sit in a `Box`; and
the "does it need to outlive multiple tasks" question decides `Arc`. When
you design your own state structs later, this exact vocabulary will come
back.

### 6.3 The `health_redis` story: a comment that saved a bug

The `health_redis` field has the longest comment in the struct, and it's not
there for decoration. Read it again:

> Dedicated Redis connection for `/api/health` PING checks. Kept separate
> from `redis` because the bookmark-import worker blocks on a shared
> multiplexed connection with an unbounded BRPOP; a PING queued behind that
> BRPOP would time out and make health report `redis:false` even though
> Redis is fine. Health also wraps its PING in a short timeout so a stuck
> Redis can never hang the endpoint.

This comment is a *debugging story frozen in amber*, and it's worth
unpacking because it teaches you how real systems fail. In Redis, `BRPOP` is
"blocking right-pop": it waits for an item to appear in a list, and it can
wait *forever*. The bookmark-import worker sits on a shared multiplexed
connection doing exactly that — `BRPOP bookmark_import_queue` with no
timeout. A multiplexed connection shares one TCP connection between many
logical operations. So imagine the health check doing `PING` on that same
connection: its `PING` gets queued *behind* the parked `BRPOP`... and waits.
And waits. The health endpoint times out and reports `redis: false` — while
Redis is perfectly healthy. A false alarm, every single time the worker is
parked.

The fix is a second connection, used *only* for health checks. The `PING`
never queues behind anything, so health reports the truth. And as a second
layer of defense, the health handler wraps its `PING` in a 2-second timeout
(`tokio::time::timeout`) so even a genuinely stuck Redis degrades health
rather than hanging the endpoint.

The lesson here is bigger than Redis: **comments that explain *why* are the
most valuable code in the project.** The type alone (`MultiplexedConnection`)
doesn't tell you why there are *two* of them. The comment does. When you
write code that has a subtle reason for existing — especially a workaround —
leave the future you (or the junior dev reading your code) a note. This
codebase does this constantly, and we'll keep reading those notes all book
long.

### 6.4 How the state gets built

We've seen the struct. Now let's watch it get *constructed* — because the
order of construction is itself a mini-tutorial in dependencies. This is the
heart of `run()` in `server.rs`:

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    // Dedicated connection for the health endpoint. Health must never share
    // the main multiplexed connection: the bookmark-import worker parks an
    // unbounded BRPOP on it, and a PING sent behind that block would time out
    // (health would report redis:false while Redis is perfectly healthy).
    let health_redis = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis for health checks");
```

Three connections, in dependency order. First PostgreSQL: `db::init_pool` —
the function we met in Chapter 5's preview of Part 3 — connects and runs
migrations. Then Redis: one client object, two connections pulled from it.
Every `.await` is a real network round-trip; every `.expect` is a loud,
descriptive panic if the subsystem is down (those are the exact panics from
Part 1's troubleshooting table: `Failed to connect to database`, `Failed to
connect to Redis`).

Then the client and the rest of the pieces:

```rust
    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");
```

Notice the details that make this production code, not a tutorial: the HTTP
client sends a real `User-Agent` header (`fichub.net/0.1.0`) and a 30-second
timeout so no scrape hangs forever. The rate limiter — which lives in Redis
— is constructed *after* the Redis connection it needs, taking a *clone* of
it. And `Arc::new(ScraperRegistry::new())`: the registry starts empty-ish
(well, registered at construction) and is wrapped in `Arc` immediately,
because it's about to be shared.

The engine, worker, and services follow the same pattern — each takes the
things it needs as arguments:

```rust
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let strategy_registry = crate::recommender::registry::StrategyRegistry::new(
        crate::recommender::available_strategies(&config),
        &config.rec_strategies,
    );
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
    let ollama_client = crate::services::ollama::OllamaClient::new(
        config.ollama_url.clone(),
        config.ollama_embed_model.clone(),
        http_client.clone(),
    );
    let heal_service = crate::heal::HealService::new(db_pool.clone(), config.clone());
```

This is the *pattern* behind the whole backend, and once you see it you'll
see it everywhere: **everything is constructed once, from its dependencies,
and shared via clones.** `db_pool.clone()`, `http_client.clone()`,
`redis_conn.clone()` — these clones are cheap for pools and clients (they
share the underlying resources), and they let the same object flow into the
engine, the worker, and the final state. Each component says, in its
constructor signature, exactly what it depends on. There are no hidden
global singletons to hunt down — the dependency graph is written in plain
Rust function calls.

### 6.5 The grand assembly

And finally, the moment all those pieces were building toward — the struct
literal that assembles the whole universe:

```rust
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        health_redis,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        strategy_registry,
        collection_worker,
        heal: heal_service,
        suggest_cache: Arc::new(tokio::sync::Mutex::new(None)),
        ollama: ollama_client,
        mailer: Box::new(crate::services::mailer::SmtpMailer {
            cfg: crate::services::mailer::SmtpConfig {
                host: config.smtp_host.clone(),
                port: config.smtp_port,
                user: config.smtp_user.clone(),
                pass: config.smtp_pass.clone(),
                from: config.smtp_from.clone(),
            },
        }),
    });
```

Read it as a table: each field gets its value. `db: db_pool` moves the pool
in (no clone — the state owns it now). `health_redis` uses shorthand — when
the field name and the local variable match, Rust lets you write the name
once. `cache_semaphores` is built *inline*: a fresh empty `HashMap` wrapped
in `Mutex` wrapped in `Arc`. `rate_limiter` gets wrapped in `Box` to become
the trait object its field type demands. And `mailer` constructs the
production `SmtpMailer` *inline*, pulling SMTP credentials straight from
config.

Then the last line — the most important one:

```rust
    let state = Arc::new(AppState { ... });
```

**The state is wrapped in `Arc` at birth.** From here on, every clone of
`state` points at the same struct. The router will hold one clone, background
workers will hold theirs, and handlers will receive theirs per-request. One
struct, one set of connections, shared safely across the whole process.

🧪 **Try It Yourself — Follow one dependency by hand**
Pick any field of `AppState` — say, `http_client`. Now trace its life:
1. In `server.rs`, find where it's created (the `reqwest::Client::builder()`
   block).
2. Count every `.clone()` of it between creation and the `AppState` literal.
3. Open `src/routes/export.rs` (or any other handler module) and find a
   handler that extracts `State(state)`. Can you find where it reaches into
   `state.http_client`?
You've just traced a dependency injection path through a real codebase —
from "created once" to "used in a handler" — without a single global
variable.

### 6.6 Why this design wins (and when it gets annoying)

Dependency injection via a shared state struct is the backbone of virtually
every Axum application, and it's worth being honest about both sides:

**What it buys you.** Testability (swap in mocks), configuration-once (no
drift between components), and a single obvious place to look when something
needs a new dependency — you add a field, build it in `run()`, done.

**What it costs.** `AppState` grows. FicHub's has seventeen fields, and every
new subsystem adds one. Handlers that need five different fields write
`state.config...`, `state.db...`, `state.ollama...` — fine, but verbose.
Some codebases split state into smaller structs composed together; FicHub
keeps one flat struct and lives with it. That's a legitimate design choice:
for a codebase this size, one obvious `AppState` beats five clever nested
ones. As you build your own apps, start flat and split only when the pain is
real.

⚠️ **Watch Out — State fields are long-lived; don't put request data in them**
`AppState` lives for the entire process. Anything you store there is
shared by *every* request. So: connections, pools, clients, registries,
caches — yes. A user's session, a per-request counter, a one-off value —
**no**. If you ever find yourself tempted to store request-specific data in
state, stop — that's what the request's own extractors are for. The line
between "shared infrastructure" and "per-request data" is one of the most
important boundaries in server design, and `AppState` sits firmly on the
shared side.

### 6.7 How handlers receive it: the `State` extractor

One quick peek at the *receiving* end, because it completes the picture. A
handler pulls state out of the request with the `State` extractor — you'll
see this pattern in nearly every handler file:

```rust
pub async fn health_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> impl IntoResponse {
```

The `State<Arc<AppState>>` extractor means: "Axum, hand me a clone of the
app state." Because it's an `Arc`, the clone is a pointer bump — cheap. Then
the handler can reach into `state.db`, `state.health_redis`, whatever it
needs. The extractor is the delivery mechanism; `AppState` is the package;
`server.rs` is the post office that built it. (We'll spend real time with
the health handler in Chapter 8 — it's the perfect example of error-aware
code.)

So: state is built once, shared everywhere, received by extractor. That's
the dependency-injection heart, fully beating. Now let's look at the organ
that pumps every request through it — the router.

---

## Chapter 7 — The Router: Every Route Registered

We have state. We have handlers (hundreds of them, tucked into the `routes/`
module and beyond). The router is the *switchboard* that connects them: for
every URL the platform answers, there is one line in `build_router` that
says "when a request for *this* path with *this* method arrives, call *that*
handler."

Open `src/server.rs` and scroll down to the function. It's the longest
function in the file — more than 350 lines of chained `.route()` calls — and
it's *almost embarrassingly simple*. That's the beauty of Axum: a router is
just a builder chain. Let's read its shape first, then dive into the details.

### 7.1 The opening: redirect helpers and the chain

```rust
/// Build the Axum router with all routes
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    // Redirect handler functions (avoid closures with async blocks)
    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }
    Router::new()
        // API routes
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/epub", get(routes::export::epub_handler))
        .route("/api/meta", get(routes::meta::meta_handler))
        .route("/api/remote", get(remote_handler))
        .route("/api/health", get(routes::health::health_handler))
```

A few things to notice before the route explosion:

- `build_router` is `async` and takes `Arc<AppState>` — it needs state for
  the middleware layers and the `with_state` call at the end (we'll see
  those in 7.5).
- The first thing it does is pull `frontend_dir` out of config, because the
  static file serving at the end of the chain needs it.
- There's a tiny nested function, `redirect_to_root`, defined *inside*
  `build_router`. The comment explains why: `Redirect::to("/")` is needed by
  three legacy routes, and a plain closure would fight with Axum's handler
  requirements (async blocks in closures are awkward). A named inner
  function is cleaner. It's a nice example of a codebase choosing the
  boring, obvious solution.
- Then: `Router::new().route(...).route(...)...`. Each `.route()` call
  returns a new `Router` with that route added, so you chain forever. The
  first five routes are the platform's front door: API docs, EPUB export,
  metadata lookup, the remote-info handler, and health.

**The `get()` helper.** `get(routes::export::epub_handler)` wraps a handler
function in a *method router* — "respond to GET requests on this path with
this handler." For other methods, Axum provides `post(...)`, `put(...)`,
`delete(...)`, `patch(...)` — or you can write `get(handler).post(other)` on
one route to serve multiple methods. Keep an eye out; the codebase uses
both styles.

💡 **Key Concept — Route = path + method + handler**
Every line in the router is the same three-part sentence: *path* ("when
someone asks for `/api/epub`"), *method* (`get` → GET requests only), and
*handler* (`routes::export::epub_handler` → run this function). When you
read a route you don't recognize, you can always decompose it this way.
And when you can't find a route for a URL the frontend is calling, you now
know exactly what to grep for.

### 7.2 Path parameters: the `{name}` syntax

A huge fraction of the routes include *dynamic* path segments — URLs that
vary per resource. Axum 0.8 (this project's version) uses `{curly_braces}`
for them. A few examples, right from the router:

```rust
        // Cache download routes
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))
```

A request for `/cache/epub/abc123/my-story.epub` matches the first route:
`{etype}` = `epub`, `{url_id}` = `abc123`, `{fname}` = `my-story.epub`. The
handler extracts them with an extractor (we'll meet that in Part 5). The
point for now: **the router is where URL structure is declared**, and every
`{name}` is a slot the handler will fill at request time.

Sometimes the same path hosts multiple methods — this happens all over the
social routes:

```rust
        .route("/api/bookmarks", axum::routing::post(crate::routes::social::add_bookmark_handler))
        .route("/api/bookmarks", get(crate::routes::social::list_bookmarks_handler))
        .route("/api/bookmarks/{work_id}", axum::routing::delete(crate::routes::social::remove_bookmark_handler))
```

Two routes, same path, different methods: `POST /api/bookmarks` creates a
bookmark, `GET /api/bookmarks` lists them, `DELETE /api/bookmarks/{work_id}`
removes one. Same URL, three verbs, three meanings — this is the RESTful
style the whole API follows. (A single route can also do this with
`get(h).post(h2)`, but the codebase here prefers separate `.route()` lines
— easier to grep, easier to see at a glance.)

⚠️ **Watch Out — Route ordering and overlapping patterns**
Axum matches routes by specificity, not by the order you write them —
literal segments beat parameters. But overlapping *parameter* patterns can
still surprise you. FicHub hit exactly this with the RSS feeds, and the
comment in the router is a great artifact of it:

```rust
        // RSS/Atom feeds. The per-fic route uses a bare {url_id} segment
        // (axum 0.8 forbids mixed literal+param segments like {url_id}.xml);
        // the handler strips a trailing ".xml" so /feed/works/<id>.xml works.
        .route("/feed.xml", get(crate::routes::rss::new_arrivals_feed))
        .route("/feed/follows.xml", get(crate::routes::rss::follows_feed))
        .route("/feed/works/{url_id}", get(crate::routes::rss::work_feed))
```

The "obvious" way to write the per-fic feed would be
`/feed/works/{url_id}.xml` — a mixed segment with both a parameter and a
literal suffix. Axum 0.8 *forbids* that shape (it's ambiguous to match), so
the route is a bare `{url_id}` and the handler strips a trailing `.xml`
itself. Same URL works, different plumbing. That comment saves the next
developer from "fixing" it back into the forbidden form.

### 7.3 A walk through the map: what the API can do

Now let's actually *walk* the router like a map of the platform, because
each cluster of routes is a feature you'll meet in a later part. Grouped
mentally, the ~200 routes look like this:

**Downloads and exports** (`/api/epub`, `/cache/{etype}/...`,
`/api/download/author`, `/api/download/series`) — the original FicHub
purpose: turn a story URL into a file. Part 5's territory.

**Meta and reader** (`/api/meta`, `/api/reader/{url_id}`, `/api/works/{id}`)
— fetch a story's metadata or read it in the browser. Parts 5–6.

**Search** (`/api/search`, `/api/search/ask`, `/api/search/suggest`,
`/api/search/similar/{work_id}`, `/api/works/random`, `/api/blind-date`) —
boolean search, natural-language "Ask the Archive," suggestions, vector
similarity, and the delightful Blind Date with a Fic (a random fic with
title and fandom hidden). Parts 6 and 10.

**Auth and users** (`/api/auth/register`, `/api/auth/login`, `/api/auth/me`,
`/api/users/{id}`) — JWT auth and profiles. Part 7.

**Social** (`/api/bookmarks...`, `/api/ratings...`, `/api/reviews...`,
`/api/comments...`, `/api/follows...`, `/api/feed`,
`/api/notifications...`) — bookmarks, ratings, reviews, threaded comments,
follows, an updates feed, notifications. Parts 7 and 8.

**Community** (`/api/requests...`, `/api/lists...`, `/api/shelves...`,
`/api/quests...`, `/api/reading/...`, `/api/work-proposals...`) — the Fic
Requests prompt board, reading lists, shelves, reading quests and streaks.
Part 8.

**Tags and curation** (`/api/tags...`, `/api/curator/...`, `/api/tropes`)
— the v3 tagging system: submit, vote, flag, autocomplete; curator
merge/alias tools; a public trope browser. Parts 8 and 11.

**Recommendations** (`/api/recommendations...`, including `/personal`,
`/strategies`, `/suggest`, `/vote`, `/train`) — the pluggable
recommendation platform. Part 9.

**AI features** (`/api/search/ask`, `/api/admin/auto-tag...`,
`/api/roadmap/...` — suggest, arena, vote, consensus) — Ollama-powered
natural-language search, the auto-tagger, and the Roadmap Consensus Engine
with its Elo arena. Part 10.

**Admin** (`/api/admin/...` — users, bans, stats, bots, moderation,
blacklist, scrapers, translations, rating-checks, auto-tag, search
analytics, realtime, roadmap-consensus) — the whole back office. Part 11.

**Platform feeds** (`/opds/...` catalog, `/feed.xml`, `/feed/follows.xml`,
`/feed/works/{url_id}`) — OPDS catalogs for e-readers and RSS feeds.
Part 8.

**Miscellaneous** — `/api/pow/challenge` and `/api/pow/solve`
(proof-of-work for shadowbanned clients, Part 11), `/api/modlog`
(transparent moderation log, Part 11), `/api/user/export` (one-click ZIP
of all personal data), `/api/send-to-kindle` (SMTP email delivery,
Part 5), `/api/locales` and `/api/translations/...` (i18n, Part 12), and
`/api/analytics...` (Part 11).

That's the whole product, in one function. This is why the router is such a
great orientation tool: **read it top to bottom and you've read the
platform's feature list.** Each cluster we'll revisit, line by line, in its
own part.

🧪 **Try It Yourself — Map a feature to its routes**
Pick any feature from the frontend you noticed in Part 1 — bookmarks,
notifications, trending, whatever. Now find it in `build_router`: grep
for the feature keyword (e.g. `search_files` for `bookmarks` in
`src/server.rs`). List the routes. Then open one of the handlers and read
its signature. You've just reverse-engineered the API contract for a
feature by reading one function — the skill you'll use constantly as a
professional.

### 7.4 The fallback: where the SPA lives

The API routes all have `/api/` or `/opds/` or `/feed` prefixes. But what
about the *frontend* — the SvelteKit app that runs in the browser? Its HTML,
JS, and CSS files must be served by *something*, and in production that
something is this same Rust server. The router's last piece handles it:

```rust
        // Static frontend
        .fallback_service(
            crate::frontend::cache_headers::CacheHeadersLayer
                .layer(
                    ServeDir::new(&frontend_dir)
                        .append_index_html_on_directories(true)
                        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
                ),
        )
```

`ServeDir::new(&frontend_dir)` serves the built SvelteKit app — the files in
`frontend/build` (that's the `FRONTEND_DIR` config default we saw in Part
1). `.append_index_html_on_directories(true)` means visiting a directory
serves its `index.html`. And here's the crucial SPA trick: `.fallback(
ServeFile::new(frontend_dir.join("index.html")))` — **any request that
doesn't match a file falls back to `index.html`.** Why? Because SvelteKit
uses client-side routing: the browser asks for `/settings`, there is no
`settings.html` on disk, but the SPA is perfectly capable of rendering that
page itself — so the server hands over `index.html` and the JavaScript
router takes it from there. One server, one origin, SPA included.

It's wrapped in `CacheHeadersLayer` — a custom middleware that sets
cache-control headers on static assets (we'll meet it in Part 12). And it's
the *fallback service*, not a route: it catches everything that didn't match
a route above it.

⚠️ **Watch Out — The fallback catches everything**
`fallback_service` is the last resort. If you add a route *after* the
fallback in the chain, it still works (Axum routes always beat the
fallback), but if you *remove* a route by accident, its URLs silently
become `index.html` instead of 404s. A missing API route is very easy to
miss when the SPA is healthy — the frontend will just show empty data.
When the UI loads but the network tab shows `index.html` responses for API
calls, you've found a missing route or a path mismatch. This exact
confusion is a rite of passage for Axum + SPA developers.

### 7.5 The tail: middleware and the state hand-off

The chain doesn't end at routes. The last three links are middleware
layers — functions that run *around* every request, before and after the
handler — and the state delivery:

```rust
        // Middleware
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::routes::analytics::track_usage,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())

        // Shared state
        .with_state(state)
```

Three layers, three jobs:

1. **`track_usage`** — a custom middleware (it has state, hence
   `from_fn_with_state`) that records an analytics event for every request
   (Part 11's topic). It runs first — so even before the handler sees the
   request, the analytics machinery has.
2. **`TraceLayer::new_for_http()`** — tower-http's request tracing: logs
   method, path, status, latency for every request. This is the layer
   behind the per-request log lines you'll see in your terminal. In dev,
   it's your friend when debugging.
3. **`CorsLayer::permissive()`** — Cross-Origin Resource Sharing wide open.
   In dev, the frontend on port 5173 talks to the backend on port 3000 —
   different origins! — and permissive CORS lets that happen without
   ceremony. (A production deployment behind a reverse proxy with a single
   origin could tighten this; the codebase chooses permissive and documents
   the tradeoff implicitly by keeping one origin in prod.)

Then, the final line of the whole function:

```rust
        // Shared state
        .with_state(state)
```

This is where the `Arc<AppState>` we built in Chapter 6 gets *attached* to
the router. From this moment on, every handler that declares a `State`
extractor will receive a clone of this state for every request. It's the
last piece of the puzzle: routes + middleware + state = a complete,
stateful application.

💡 **Key Concept — Middleware order matters**
Layers wrap in the order they're added, LIFO-style: the first `.layer()`
added is the *outermost* wrapper (runs first), and each subsequent layer
wraps inside it. Here `track_usage` runs before `TraceLayer` before CORS
(roughly). When you add your own middleware, think about what should run
*before* the handler (auth checks, logging, rate limiting) vs *after* (CORS
headers, response compression) — order is semantics.

### 7.6 A small handler, fully read

Before we leave the router, let's read one whole handler that lives right
next to it in `server.rs` — because it's a perfect specimen of "small
handler with an extractor," and it shows you the shape your own handlers
will take:

```rust
/// Remote info handler: GET /api/remote
async fn remote_handler(
    axum::extract::ConnectInfo(remote_addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
        "is_automated": false,
    }))
}
```

Decompose it: `ConnectInfo<SocketAddr>` is an extractor that provides the
caller's address (it's enabled by the `into_make_service_with_connect_info`
call in `run()`, remember?). The handler builds a tiny JSON object — the
caller's IP, port, and a hardcoded `is_automated: false` — and returns it as
`axum::Json`. That's it. A handler is just `async fn(extractors) -> response`.
Every handler you'll meet in the next ten parts is this same shape, scaled
up.

### 7.7 The router in one breath

`Router::new()` — chain two hundred `.route(path, method(handler))` calls,
grouped by feature; add a SPA fallback; wrap in middleware; attach state.
The result is a single `Router` value that knows how to answer *every*
request the platform serves. `run()` hands it to `axum::serve`, which binds
the port and starts accepting connections.

But there's one question we haven't answered, and it's the difference
between a toy server and a real one: **what happens when something goes
wrong inside a handler?** A database query fails. A scrape times out. A user
asks for a story that doesn't exist. If the answer is "the server crashes,"
then one bad request takes down the whole platform — which would be
unacceptable. FicHub's answer lives in a file we haven't opened yet:
`error.rs`. That's Chapter 8.

---

## Chapter 8 — Errors That Don't Crash

Here's a question every server developer must answer: **what happens when a
handler fails?**

Think about what's at stake. A request arrives for a story that was deleted.
A scraper times out because the source site is slow. Redis hiccups for a
second. The database returns an error. If any of these crashed the whole
process, the platform would be down constantly — one bad request, one
transient network blip, and every reader on the site loses the server. That's
not acceptable for something whose whole job is reliability.

The naive solution — catch every error, everywhere, manually — is a
maintenance nightmare. FicHub's solution is the opposite: one central error
type, one conversion rule for each kind of failure, and one place that turns
errors into HTTP responses. It's called `AppError`, it lives in
`src/error.rs`, and it's a masterclass in designing for failure. Let's read
it top to bottom.

### 8.1 The enum: ten ways to fail

```rust
/// Application-wide error type
#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message
    BadRequest(i32, String),
    /// Rate limited - wait N seconds
    RateLimited(u64),
    /// Rate limited with a custom JSON body (e.g. a PoW challenge).
    /// The payload is served as-is with HTTP 429 so callers can hand the
    /// client the next challenge without inventing a new status code.
    RateLimitedJson(serde_json::Value),
    /// Forbidden access
    Forbidden(String),
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Scraper error
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

An enum is a type with a fixed set of *variants* — and each variant can carry
data. Read this as a taxonomy of everything that can go wrong in FicHub:

- `BadRequest(i32, String)` — the client did something wrong: an error code
  and a message.
- `RateLimited(u64)` — the client hit the rate limiter; the `u64` is how
  many seconds to wait (retry-after).
- `RateLimitedJson(serde_json::Value)` — rate-limited, *with a custom
  payload*, used to hand a shadowbanned client its next proof-of-work
  challenge. The comment explains the design decision: serve the payload
  as-is with HTTP 429, "so callers can hand the client the next challenge
  without inventing a new status code."
- `Forbidden(String)` — authenticated but not allowed.
- `NotFound(String)` — the thing doesn't exist.
- `Internal(String)` — something went wrong on our side.
- `ScrapeError(String)` — the scraper failed (source site unreachable,
  structure changed).
- `ExportError(String)` — building the EPUB/HTML/TXT failed.
- `Database(String)` — the database errored.
- `CacheError(String)` — Redis/cache failed.

Two design notes before we go on. First, look at what's *not* here: no
"everything else" variant, no `String`-only blob for every case. The enum
distinguishes *kinds* of failure because each kind needs different handling
— a `NotFound` gets a 404, a `RateLimited` gets a 429 with retry-after, an
`Internal` gets *hidden* details. Second, every message-bearing variant
carries a `String` — the *what* of the failure — but the enum type itself is
the *what kind*. Both matter.

💡 **Key Concept — Why an enum beats a bare string**
A naive error type is `type Error = String` — but a string can't tell you
whether to return 404 or 429 or 502, and you can't `match` on it. An enum
*encodes the category of failure in the type itself*, which means the
compiler can force you to handle each category. Later you'll see the
`IntoResponse` impl `match` on exactly this enum — and the compiler will
refuse to build if a variant isn't handled. That's the type system working
for you.

### 8.2 Display: how errors read as text

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(code, msg) => write!(f, "BadRequest({}): {}", code, msg),
            AppError::RateLimited(retry_after) => write!(f, "RateLimited: retry after {}s", retry_after),
            AppError::RateLimitedJson(_) => write!(f, "RateLimitedJson"),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::NotFound(msg) => write!(f, "NotFound: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
            AppError::ScrapeError(msg) => write!(f, "ScrapeError: {}", msg),
            AppError::ExportError(msg) => write!(f, "ExportError: {}", msg),
            AppError::Database(msg) => write!(f, "Database: {}", msg),
            AppError::CacheError(msg) => write!(f, "CacheError: {}", msg),
        }
    }
}
```

`Display` is Rust's trait for "render me as human-readable text" — the thing
`println!("{}", err)` uses. Implementing it is one `match` over every
variant, each arm producing a readable line: `BadRequest(400): invalid
input`, `RateLimited: retry after 30s`, and so on.

Two things worth noting. First, `RateLimitedJson(_)` ignores its payload in
the text form — the `_` pattern — because a JSON challenge isn't readable as
a one-liner; it's meant for clients, not logs. Second, the *match is
exhaustive*: every variant has an arm. If someone adds an eleventh variant
later, the compiler refuses to build until they add an arm here *and* in
`IntoResponse` — which is exactly the safety we talked about. You cannot
silently forget an error category.

Why does `Display` matter at all if errors mostly become JSON? Because
`Display` is the cheap, universal text representation — used in logs, in
`tracing::error!` lines, in test assertions (you'll see the tests use
`format!("{}", err)` and check the text). It's the human face of the error;
`IntoResponse` is the HTTP face.

### 8.3 IntoResponse: the heart of the design

Now the part that makes the whole thing work. `IntoResponse` is Axum's
trait for "this type can become an HTTP response." By implementing it for
`AppError`, the codebase makes a promise: **any handler that returns
`Result<_, AppError>` can return an error, and Axum will automatically
convert it into a proper HTTP response with the right status code and JSON
body.** No handler has to write error-response logic. Let's read it:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS, json!({"err": -429, "msg": "rate limited", "retry_after": retry_after}))
            }
            AppError::RateLimitedJson(payload) => {
                (StatusCode::TOO_MANY_REQUESTS, payload)
            }
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, json!({"err": -403, "msg": msg}))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "internal server error"}))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -5, "msg": "export failed"}))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "database error"}))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "cache error"}))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

Read the `match` as a translation table: every failure category maps to an
HTTP status code and a JSON body.

- `BadRequest` → **400** with the code and message.
- `RateLimited` → **429** (Too Many Requests) with `err: -429` and the
  retry-after seconds.
- `Forbidden` → **403**.
- `NotFound` → **404** with `err: -5`.
- `Internal` → **500** — and notice: the *real* message is logged with
  `tracing::error!`, but the JSON body says only
  `"internal server error"`. The client doesn't get our internal details
  (which could leak SQL or file paths); *we* get them in the logs.
- `ScrapeError` → **502 Bad Gateway** — the server, acting as a gateway to
  the source site, couldn't get a valid response upstream. That's a
  deliberately *different* status from 500: it tells operators "the problem
  is upstream, not our code."
- `ExportError`, `Database`, `CacheError` → **500**, each logged, each with
  a generic client-facing message.

Then the last line: `(status, Json(body)).into_response()` — build the
response from the status and the JSON. Axum knows how to turn a
`(StatusCode, Json<Value>)` pair into a `Response`.

⚠️ **Watch Out — Never leak internals to the client**
Study the asymmetry in the `Internal`, `ExportError`, `Database`, and
`CacheError` arms: the *full* message goes to the server logs
(`tracing::error!`), while the client gets a fixed, generic string. This
is deliberate and important. Error strings often contain SQL, file paths,
hostnames, or stack details — gold for an attacker probing your API, noise
for a normal user. Rule of thumb: **details in, generic out.** Log
everything on the server; return the minimum to the client. FicHub does
this consistently, and you should too, from day one.

The elegant part is what this buys every handler in the codebase. A handler
can now write:

```rust
let fic = sqlx::query_as::<_, Work>("SELECT ...")
    .fetch_one(&state.db)
    .await?;
```

and the `?` operator will convert a `sqlx::Error` into an `AppError::Database`
(thanks to the `From` impls we're about to read), which becomes a logged 500
with a generic body — automatically, with zero error-handling code in the
handler. The whole error pipeline is centralized, and handlers stay readable.

### 8.4 From: converting the world into AppError

The last piece of the machinery is a set of `From` implementations — the
conversion rules that let `?` work. In Rust, `?` on a `Result<T, E>` inside
a function returning `Result<T, F>` compiles only if `E: Into<F>` — that is,
if there's a `From<E> for F`. So the codebase declares: *here's how each
external error becomes an AppError*:

```rust
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        AppError::CacheError(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
```

Five rules, each a one-liner, each mapping a library's error type to the
right *category*:

- `std::io::Error` (file I/O) → `Internal` — our side.
- `sqlx::Error` (PostgreSQL) → `Database`.
- `redis::RedisError` → `CacheError`.
- `reqwest::Error` (outgoing HTTP) → `ScrapeError` — the scraper's world.
- `serde_json::Error` (parsing) → `Internal`.

And then the type alias that makes every handler's signature pleasant to
read:

```rust
/// Standard API result type
pub type AppResult<T> = Result<T, AppError>;
```

`AppResult<T>` is just `Result<T, AppError>` with a shorter name — so
handlers across the codebase declare `-> AppResult<Json<Something>>`
instead of the longhand. It also gives you one searchable name: grep for
`AppResult` and you've found every handler that can fail through the
central pipeline.

💡 **Key Concept — `?` is a conversion pipeline**
The `?` operator is secretly the whole design. `foo()?` means "if this
returned `Err(e)`, convert `e` into my function's error type via `From`,
and return that early." By implementing `From` for exactly five external
error types, FicHub turns `?` into a *routing system*: a database error
becomes `Database`, a Redis error becomes `CacheError`, and every one of
them flows into `IntoResponse` and becomes the right HTTP status + body.
Three traits — `From` (convert), `IntoResponse` (render), `Display`
(describe) — and the entire codebase's error handling is uniform. That's
the whole trick, and it's worth savoring because you will use it in every
Rust service you ever write.

### 8.5 The tests: proving the contract

`error.rs` doesn't just define the machinery — it *proves* it with tests.
There are test modules at the bottom of the file (`#[cfg(test)] mod tests`)
with three groups of tests: `Display` tests, `From` tests, and
`IntoResponse` tests. Here's a sample of each group:

```rust
    #[test]
    fn test_display_bad_request() {
        let err = AppError::BadRequest(400, "invalid input".into());
        let s = format!("{}", err);
        assert!(s.contains("BadRequest"));
        assert!(s.contains("400"));
        assert!(s.contains("invalid input"));
    }
```

A `Display` test: build the error, format it, assert the text contains the
expected pieces.

```rust
    #[test]
    fn test_from_sqlx_error() {
        let sqlx = sqlx::Error::Protocol("bad query".into());
        let app: AppError = sqlx.into();
        match app {
            AppError::Database(msg) => assert!(msg.contains("bad query")),
            _ => panic!("expected Database, got {:?}", app),
        }
    }
```

A `From` test: take a real `sqlx::Error`, convert it with `.into()`, and
assert it landed in the `Database` category with the message preserved. Note
the `match` with a `panic!` in the fallback arm — the test fails loudly if
the conversion is wrong.

```rust
    #[test]
    fn test_into_response_not_found_status() {
        let err = AppError::NotFound("gone".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
```

An `IntoResponse` test: convert the error into a real HTTP `Response` and
assert the status code is what the contract promises. There's one of these
for every category — `test_into_response_rate_limited_status` asserts 429,
`test_into_response_scrape_error_status` asserts 502, and so on.

Why do these tests matter? Because **the error contract is the API's
contract.** The frontend's error handling (Part 12) depends on these exact
status codes and `err`/`msg` shapes. If someone refactors `IntoResponse` and
accidentally maps `NotFound` to 500, a test fails immediately. The tests pin
the contract down so refactors can't silently break the API's behavior. This
is the same testing spirit you'll see everywhere in the repo — and the
reason `lib.rs` exists as a library crate at all.

🧪 **Try It Yourself — Break the error contract, watch the test scream**
1. Open `src/error.rs`. In `IntoResponse`, change the `NotFound` arm's
   status to `StatusCode::INTERNAL_SERVER_ERROR` (temporarily).
2. Run `cargo test --lib error` (or just `cargo test`) and watch
   `test_into_response_not_found_status` fail with an assertion mismatch.
3. Change it back. You've now experienced the safety net that keeps error
   handling honest — a test that pins a *behavior contract*, not just
   implementation details.
4. Bonus: run `cargo test` and count how many tests in `error.rs` pass.
   Every one of them is a promise the codebase makes to itself.

### 8.6 A handler that lives in the error world

To see the whole pipeline in action — state, extractor, `?`, AppError,
`IntoResponse` — let's read the health handler's ending. We met its opening
in Chapter 6; here's how it finishes:

```rust
    // Return 503 if not fully healthy
    let status_code = if status == "ok" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status,
        db: db_ok,
        redis: redis_ok,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    (status_code, Json(response)).into_response()
}
```

No `AppError` in sight here — and that's the point. This handler *can't*
fail in the usual way; it degrades gracefully: if the database check fails,
`db_ok` is false, the status becomes 503, and the JSON says so. The health
endpoint is the one place where "failure" is a *response*, not an error —
which is exactly what you want from a health check: it should never crash,
it should always answer, and its answer should tell you the truth about each
subsystem.

It also shows the second use of `env!("CARGO_PKG_VERSION")` — the
compile-time macro that injects the crate version from `Cargo.toml` into the
binary. The version string in the health response isn't hardcoded; it's
always whatever `Cargo.toml` says. (You met the first use in the `ServeDir`
context in Part 1's tour.) Compile-time macros like this are a lovely Rust
habit: let the build system tell the truth.

(For the record: the health handler *does* touch the error world — its
`SELECT 1` and `PING` results are checked with `.is_ok()` and `timeout`
rather than `?`, precisely so a failure becomes a `db: false` field instead
of a 500. If you want to see `?` and `AppError` in action in a real
handler, open `src/routes/export.rs` — every `.await?` there is a scrape,
cache, or database step flowing through the pipeline we just built.)

### 8.7 The error philosophy, in one page

Let's step back and name the four principles this chapter taught — because
they apply to *every* server you'll ever write:

1. **One error type for the whole app.** Ten variants, each a category of
   failure. No scattered ad-hoc error strings.
2. **Convert at the boundary.** `From` impls map external errors (sqlx,
   redis, reqwest, io, serde_json) into categories at the moment they enter
   your code. `?` does the plumbing.
3. **Render in one place.** `IntoResponse` is the single translation table
   from failure category → HTTP status + JSON body. Handlers never build
   error responses themselves.
4. **Details in, generic out.** Log the full message; send the client a
   fixed, safe string. Never leak internals.

Follow those four rules and your error handling is *done* — uniformly,
testably, and safely — for the entire codebase. FicHub doesn't special-case
errors per handler; it built the machine once, in 317 lines, and every
handler plugs into it.

⚠️ **Watch Out — `Result<T, AppError>` vs panics**
`AppError` handles *expected* failures — things that can go wrong in
normal operation (bad input, missing rows, upstream timeouts). Panics
(`expect`, `unwrap`) are for *programmer* errors — invariants that should
never be violated (like "the config parsing code is buggy"). In this
codebase, note the split: `run()` uses `expect("Failed to connect to
database")` at boot — if the database is down at startup, there's nothing
to serve, so failing loudly is correct. But once the server is *running*,
a database error mid-request is `AppError::Database`, because the server
should keep serving. Boot: panic. Steady state: error. Knowing which
situation you're in is the difference between robust and fragile code.

---

## Chapter 9 — Putting the Foundation Together (Bridge to Part 3)

We've now read every file that makes the backend *exist*: the entry point
that boots it (`main.rs`), the library that tests it (`lib.rs`), the state
that wires it (`server.rs`), and the errors that keep it alive (`error.rs`).
Before we close Part 2, let's do one final pass — a single request, from
browser to database and back — through the whole foundation. This is the
"everything connects" moment.

### 9.1 One request, through the whole foundation

Imagine a reader on the FicHub home screen pastes a story URL and hits
Download. Here's the path of that one request through everything we've
built:

1. **`main.rs`** — already ran: runtime up, `.env` loaded, tracing lit,
   config read. The server is listening.
2. **`server.rs` `run()`** — already ran: database pool connected,
   migrations applied, Redis connected (twice!), HTTP client built,
   scrapers registered, rate limiter initialized, engine and workers
   constructed, `AppState` assembled in an `Arc`, router built with state
   attached. `axum::serve` is accepting connections.
3. **The router** — the request arrives at `/api/epub?...`. Axum matches it
   against `build_router`'s table, finds
   `.route("/api/epub", get(routes::export::epub_handler))`, and runs the
   middleware layers around it.
4. **The handler** — `epub_handler` declares `State(state): State<Arc<AppState>>`
   and receives a clone of the state. It reaches into `state.scraper_registry`
   to scrape the story, `state.db` to look up or store metadata, and
   `state.http_client` — all injected, all shared.
5. **Errors** — if the scrape fails, the handler returns
   `AppError::ScrapeError(...)` (or `?` converts a `reqwest::Error` into
   it), and `IntoResponse` turns it into a 502 with a JSON body. The server
   keeps running. No crash.
6. **The response** — success or error, the response flows back through the
   middleware (traced, CORS'd) to the browser. The SPA renders the result.

Six steps. Two files define the boot, one struct wires it, one function maps
it, one type tames its failures. That's the Rust foundation — and everything
else in this book — every scraper, every search feature, every
recommendation, every admin page — plugs into exactly this skeleton.

⚠️ **Watch Out — The foundation is load-bearing; don't bolt around it**
The temptation when you start your own project is to skip the boring
infrastructure and write the fun handler first. Resist it. Every subsystem
you'll meet in the coming parts — scrapers, search, recommendations — was
built *on top of* `AppState` and `AppError`, not around them. When you add a
feature to your own server, the professional move is to add its dependency
to state, register its routes in the router, and route its failures through
the error type. The foundation only works if everything stands on it.

💡 **Key Concept — The kernel and the satellites**
Think of these four files as a kernel: small, stable, and absolutely
required. Around it orbit satellites — big, impressive subsystems that
change constantly. Kernels change rarely and carefully (a change to
`AppError` ripples through every handler); satellites change freely. When
you're reading any unfamiliar codebase, find the kernel first. In FicHub
you've just read all four of its files — that's why every later part will
feel like building on known ground.

### 9.2 What you can now do

Stop and take stock, because this is a real milestone. After Part 1 you
could *run* FicHub. After Part 2 you can *explain* it:

- You can read any Rust binary's boot sequence (`main.rs`, the module tree,
  `#[tokio::main]`, the four boot steps).
- You can explain why a project has both `main.rs` and `lib.rs`, and what
  `pub use` re-exports are for.
- You can read `AppState` and name what each of its seventeen fields does —
  and why `Arc` and `Box<dyn Trait>` appear where they do.
- You can read `build_router` as a feature map, explain `{params}`,
  fallbacks, middleware order, and the SPA-serving trick.
- You can explain the entire error pipeline: enum → `From` → `?` →
  `IntoResponse` → status + JSON, and why internals never leak.

Those five sentences describe a junior developer who has *internalized* how
a real server is put together. Before Part 3, try this:

🧪 **Try It Yourself — Explain FicHub to a rubber duck**
Open a blank file (or a fresh chat with a rubber duck — a rubber duck
works). Write down, from memory: (1) the four steps of `main`; (2) the
three sharing types in `AppState` (`T`, `Arc`, `Box<dyn>`); (3) the
three-part structure of any route; (4) the four principles of the error
design. If you can't write all four from memory, re-read the sections —
then try again tomorrow. Explaining from memory is the test that turns
"read it" into "know it."

### 9.3 Where the foundation leads

The foundation is laid, but a foundation is only useful if something stands
on it. Here's what's next:

In **Part 3** we go underground first — `config.rs` is the biggest config
reader you'll ever see (roughly 800 lines of environment variables, each
with a default, a parse, and a comment), and `db/mod.rs` + the 34
migrations tell the story of the entire schema. That's the data layer the
handlers all lean on.

But before we get there, one more confession: this chapter you just read
isn't the end of Part 2 — it's the bridge. The foundation files we've read
are deceptively small: `main.rs` is 42 lines, `lib.rs` is 25, `error.rs` is
317, and `server.rs` is 632. Four files, just over a thousand lines, and
they support a platform of many thousands more. The reason is the pattern
you now hold: small, boring, well-commented infrastructure files that make
the exciting parts — scrapers, search, recommendations, AI — possible.

Hold onto that thought as we climb: the most important files in a codebase
are rarely the most impressive. They're the ones that make everything else
possible. We just read them all.

Next stop: `config.rs` — every knob the platform has, and how one struct
tames eight hundred lines of environment variables. Part 3 begins there.

See you in the next part — your foundation is solid.

---
