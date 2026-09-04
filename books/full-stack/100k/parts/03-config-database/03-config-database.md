# Part 3 — Configuration & the Database Layer

> **Part 3 of 13** — At the end of Part 2 you could explain, with your eyes
> closed, exactly what happens between `cargo run` and a browser tab loading
> FicHub. You met the four spine files — `main.rs`, `lib.rs`, `server.rs`,
> `error.rs` — and the promise was made: next we go underground. This part
> keeps that promise. We open `config.rs`, the biggest environment-variable
> reader you'll ever see (~800 lines), then `db/mod.rs` where the connection
> pool is born, `db/models.rs` where database rows become Rust structs, the
> 34 migrations that tell the entire story of the schema, `db/queries.rs`
> (2,700 lines of SQL wearing a trench coat), and finally the Redis layer —
> token-bucket rate limiting, shadowbans, and proof-of-work challenges.
> By the end of Part 3, when someone asks "where does FicHub keep its data
> and how do you keep the bots out?" you'll be the one answering.

---

## Chapter 10 — Every Knob: config.rs

Part 2 ended with a promise: "Next stop: `config.rs` — every knob the
platform has, and how one struct tames eight hundred lines of environment
variables." Here we are. Open `src/config.rs`. Go ahead — it's the longest
file we've read so far, and the most repetitive. That repetition is the
point.

### 10.1 Why environment variables?

Before we read a single line, answer a question that sounds philosophical
but is actually engineering: *where does configuration live?*

There are three classic answers:

1. **In the code.** Hard-coded constants. `const PORT: u16 = 3000;` Simple,
   but every change means a recompile and a redeploy.
2. **In a config file.** A `config.toml` read at startup. No recompile —
   but every machine needs a copy of the file, and secrets sitting in a
   file on disk get committed to git and left on shared servers by
   accident.
3. **In environment variables.** The operating system hands each process
   a set of `KEY=VALUE` pairs at launch. The program reads them once at
   startup. No recompile, no config file to lose, and secrets can be
   injected by the deployment system (Docker, systemd, CI) without ever
   touching a developer's laptop.

This is the **twelve-factor app** doctrine, rule number three: *store
config in the environment*. FicHub follows it religiously — and virtually
every serious deployment (Docker Compose, Kubernetes, Heroku, Fly.io)
speaks the same language. Learning to read config from env is learning
how production servers are actually configured.

💡 **Key Concept — Environment variables are the interface between the
deployment and the program.** A binary that reads `DATABASE_URL` from its
environment can run anywhere: on your laptop pointing at a local Postgres,
in a Docker container pointing at the compose database, in production
pointing at a managed instance — same binary, zero recompiles. The
environment *is* the config file, and it's different on every machine by
design.

So `config.rs` exists to do one job: **translate the wild west of
environment variables into one typed, documented, easy-to-hold Rust
struct.** Every other file in the codebase gets a `Config` and never has
to think about strings like `"true"` or `"3000"` again. The messy parsing
happens in exactly one place.

### 10.2 The struct: one field per knob

Here's how the file opens:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,
    pub calibre_container: String,
    pub tmp_dir: PathBuf,
    /// Where scraped fic bodies are cached as JSON blobs (the site is a
    /// cache of all gathered fanfiction). Defaults to the attach drive:
    /// /public/literature/fichub/bodies.
    pub body_cache_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
```

Nothing fancy: a struct with a field per knob, each field carrying the
exact type the rest of the program wants to use. `database_url` is a
`String`, `app_port` is a `u16` (a port can never be negative or
`3.5` — the type says so), `trusted_proxies` is a `Vec<String>`, and
`body_cache_dir` gets a real doc comment explaining *why* it defaults to
`/public/literature/fichub/bodies`:

> *"the site is a cache of all gathered fanfiction"*

That's a sentence that tells you what FicHub fundamentally *is* — it
gathers fanfiction and caches it — which is exactly what a good doc
comment should do: explain the intent, not the mechanics.

Now notice the two derives:

```rust
#[derive(Debug, Clone)]
```

`Debug` lets us print the whole config for debugging. `Clone` means any
code that holds a `Config` can hand out copies cheaply — remember the
`AppState` from Part 2? It held a `config: Config` that was *cloned in*
for handlers. That's why `Clone` is here. If it weren't, every handler
that wanted config would need to borrow it forever.

Let's keep scrolling through the struct — I'll skip the middle so you can
appreciate the pattern, and jump to two groups that show how the struct
is organized. First, the tiered rate-limiting group — the knobs that
power Chapter 14's anti-bot limiter:

```rust
    // ── Tiered rate limiting (anti-bot) ──────────────────────────────
    /// Capacity (burst tokens) of the download-tier bucket per IP.
    pub rl_download_capacity: f64,
    /// Refill rate of the download-tier bucket per IP, tokens/second.
    pub rl_download_flow: f64,
    /// Capacity (burst tokens) of the auth-tier bucket per IP.
    pub rl_auth_capacity: f64,
    /// Refill rate of the auth-tier bucket per IP, tokens/second.
    pub rl_auth_flow: f64,
    /// Capacity (burst tokens) of the loose/search-tier bucket per IP.
    pub rl_search_capacity: f64,
    /// Refill rate of the loose/search-tier bucket per IP, tokens/second.
    pub rl_search_flow: f64,
    /// Extra capacity granted to the per-`(ip, client_id)` bucket on top of
    /// the per-IP bucket. `0` disables the client keying entirely.
    pub rl_client_bonus_capacity: f64,
    /// Extra refill rate granted to the per-`(ip, client_id)` bucket.
    pub rl_client_bonus_flow: f64,
```

Every single field has a comment. And the comments aren't decorative —
they carry the mental model: *capacity* is burst tokens, *flow* is
tokens/second, buckets are *per IP*, and there's a bonus bucket per
`(ip, client_id)`. In Chapter 14 we'll see these exact fields flowing
into the Redis bucket limiter. This is a recurring FicHub pattern: the
comments in config.rs are the documentation for the whole rate-limiting
subsystem.

And at the end of the struct, the self-healing agent group — the AI
loop that will diagnose and fix scraping problems on its own:

```rust
    // ── Self-healing agent (docs/AGENTS.md) ─────────────────────────
    /// Master switch for the diagnose/fix agent loop (default false).
    pub agent_enabled: bool,
    /// Model name for agent calls (default `deepseek/deepseek-v4-flash`).
    pub agent_model: String,
    /// API key for the remote agent endpoint (COMMANDCODE_API_KEY or
    /// AGENT_API_KEY). `None` when neither is set.
    pub agent_api_key: Option<String>,
    /// Base URL of the OpenAI-compatible agent endpoint.
    pub agent_base_url: String,
    /// Local Ollama fallback base URL.
    pub agent_ollama_url: String,
    /// Max agent runs per day (gate on `agent_runs.created_at`).
    pub agent_max_runs_per_day: u32,
    /// Min seconds between agent runs for the same domain (cooldown).
    pub agent_cooldown_domain_secs: u64,
}
```

Count the *types* in this struct and you'll see the whole story of the
platform: `String` URLs (Postgres, Redis, Ollama, SMTP, an AI agent
endpoint), `PathBuf`s (cache dirs, tmp dirs, the frontend build), numbers
of every size (`u16` ports, `u32` counts, `u64` seconds, `i32` versions,
`i16` thresholds, `f64` rates and gamma values), `bool` switches, and
`Option<T>` everywhere a knob might simply not exist. About 150 fields,
each with a comment. That's the "every knob" promise: if FicHub can be
tweaked, it's in this struct.

### 10.3 The pattern: expect, unwrap_or_else, parse

The struct is the *what*. `from_env()` is the *how*. Let's read it:

```rust
impl Config {
    /// Load configuration from environment variables.
    /// Required vars: DATABASE_URL, REDIS_URL, CACHE_DIR
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
```

There it is — the entire philosophy of this file in six lines, and it
comes down to two standard-library methods:

- `std::env::var("DATABASE_URL")` returns a `Result<String, VarError>`:
  `Ok(value)` if the variable exists, `Err` if it doesn't (or contains
  invalid Unicode).
- `.expect("DATABASE_URL must be set")` says: *if this fails, crash the
  program with this message.* FicHub **cannot run** without a database
  and a Redis. So it fails loudly and immediately, at startup, with a
  message that tells the operator exactly what to fix.
- `.unwrap_or_else(|_| "./cache".to_string())` says: *if this variable is
  missing, use this default.* The cache directory can live anywhere; a
  sensible local default is `./cache`. Fail *soft*.

⚠️ **Watch Out — `expect` is a landmine in library code, but a gift in
startup code.** Beginners are taught to fear `expect` and `unwrap`
because they panic. But panic *at the right moment* is a feature: if the
database URL is missing, there is no graceful way to continue — every
single request would fail. Crashing in the first 100 milliseconds with
`DATABASE_URL must be set` is infinitely better than crashing at 2 a.m.
with an opaque connection error. FicHub only uses `expect` where failure
is truly unrecoverable: the two required connection strings. Everything
else gets a default.

This `expect` / `unwrap_or_else` pair is the skeleton of the entire
function. Almost every one of the ~150 fields follows the same three
steps, and you'll see the third step in this next example:

```rust
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
```

Read that chain from the inside out:

1. `std::env::var("PORT")` — fetch the variable (a `Result`).
2. `.unwrap_or_else(|_| "3000".to_string())` — missing? Use the string
   `"3000"`. Now we have a `String` no matter what.
3. `.parse()` — convert the `String` into... what? The type is decided by
   where the result goes: `app_port` is a `u16`, so `.parse()` tries to
   produce a `u16` and returns a `Result<u16, _>`.
4. `.unwrap_or(3000)` — parse failed (someone set `PORT=banana`)? Fall
   back to the numeric default.

So the *string* default and the *numeric* default are both 3000 — the
first catches a missing variable, the second catches a garbage variable.
Two failure modes, two fallbacks. Notice the difference between
`unwrap_or_else` (lazy, takes a closure, only computes the default when
needed) and `unwrap_or` (eager, takes a value) — for a `"3000".to_string()`
it makes no practical difference, but Rust gives you both so that
expensive defaults aren't built when they're not needed.

Some knobs need a smarter fallback than a constant. Look at how
`secondary_cache_dir` handles "set but empty":

```rust
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
```

This one deserves a slow read. `std::env::var(...)` gives a
`Result<String, VarError>`. `.ok()` converts it to an `Option<String>`
— `Some` on success, `None` on error. `.filter(|s| !s.is_empty())`
turns `Some("")` into `None` — an *empty* secondary cache dir means "no
secondary cache dir", and an empty string is exactly what a half-hearted
`.env` file or a `docker-compose` override tends to leave behind.
`.map(PathBuf::from)` converts `Some(String)` into `Some(PathBuf)`. The
end result: a `None` means "not configured", and the field's type —
`Option<PathBuf>` — makes that meaning explicit at the type level.
Remember this `.ok().filter().map()` chain: it's the idiomatic way to
say "optional, and empty means absent."

💡 **Key Concept — `Option` is the type-safe way to say "this might not
exist."** In many languages, "not configured" is a magic value: an empty
string, a `-1`, a `null` that you must remember to check. In Rust, the
compiler *forces* you to handle absence: you can't use an
`Option<PathBuf>` as a `PathBuf` without deciding what happens when it's
`None`. Half of FicHub's config fields are `Option<T>` for exactly this
reason — `agent_api_key` is `Option<String>` because the platform runs
fine without an agent key, and every use site has to admit that.

### 10.4 Parsing structured values: the fun part

Most env vars are single values. But three of them smuggle *structure*
into a string, and FicHub shows three different parsing tricks.

**Trick 1 — comma-separated list.** `TRUSTED_PROXIES` holds the IPs of
proxies that are allowed to set headers on our behalf:

```rust
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
```

`"10.0.0.1, 10.0.0.2"` becomes `["10.0.0.1", "10.0.0.2"]`. The
`.trim()` handles the space after the comma, the `.filter` drops the
empty strings that a trailing comma would produce (`"10.0.0.1,"` →
`["10.0.0.1", ""]` → `["10.0.0.1"]`). `.collect()` is the magic wand
that gathers the iterator of `String`s into a `Vec<String>` — its
destination is inferred from the field type. This is the standard
"CSV-ish list in an env var" idiom, and it's robust against sloppy
input.

**Trick 2 — line-separated tuples.** `IP_TAG_SOURCES` is a list of
`path,type,tag` triples, one per line. That's too rich for commas (the
paths themselves could contain them), so FicHub splits on *lines* first:

```rust
        // IP tag sources: format "path,type,tag" per line in IP_TAG_SOURCES
        let ip_tag_sources = std::env::var("IP_TAG_SOURCES")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].trim().to_string(),
                        parts[1].trim().to_string(),
                        parts[2].trim().to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();
```

Two details worth pausing on. First, `.splitn(3, ',')` — "split into at
most 3 pieces" — so a path containing a comma (yes, paths can contain
commas) doesn't shatter the tuple: `"/a,b.txt,type,tag"` splits into
`["/a", "b.txt", "type,tag"]` — wait, no. Let's count again: `splitn(3,
',')` on `"/a,b.txt,type,tag"` gives `["/a", "b.txt", "type,tag"]` —
three pieces, and the *rest of the string stays in the last piece*. The
last comma can't split a path. That's the point of `splitn`: cap the
number of splits so the trailing fields can absorb commas.

Second, `.filter_map(...)` — the closure returns `Option<...>`: `Some`
for well-formed lines, `None` for garbage. `filter_map` keeps the
`Some`s and discards the `None`s, so a malformed line is silently
skipped instead of crashing the whole config load. The same "filter out
bad input" philosophy as the `.filter` above, but now the *parsing*
itself can fail per-line.

**Trick 3 — JSON in an env var.** The recommender needs a map of
site name → rate limit. That's a `HashMap<String, u64>`. FicHub's
answer? Just put JSON in the env var and let `serde_json` do the work:

```rust
        let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
            .unwrap_or_else(|_| "{}".to_string());
        let rec_site_rate_limits: HashMap<String, u64> =
            serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();
```

`REC_SITE_RATE_LIMITS='{"ao3": 10, "ffn": 5}'` becomes a real
`HashMap`. The default is the empty map `{}`, and if the JSON is
malformed, `.unwrap_or_default()` again produces the empty map — *the
recommender will just treat every site as having no custom limit*. There
are actual unit tests for this exact behavior in the file, which we'll
get to in a moment.

And there's one helper function that packages the `f64` parse for the
rate-limit knobs, because thirteen nearly identical chains would be
thirteen chances to typo a default:

```rust
/// Parse a `f64` env var with a fallback (used by the tiered rate-limit config).
fn env_f64(key: &str, default: f64) -> f64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
```

So instead of writing the chain thirteen times, the code writes:

```rust
        let rl_download_capacity = env_f64("RL_DOWNLOAD_CAPACITY", 10.0);
        let rl_download_flow = env_f64("RL_DOWNLOAD_FLOW", 60.0 / 3600.0);
```

Note the comment above these lines — it documents the *units* and the
*intent*, which is where rate limits go to die without comments:

```rust
        // ── Tiered rate limiting (anti-bot) ─────────────────────────
        // Tiers are (capacity, flow-tokens/sec). Defaults implement:
        //   download: 10 burst, refills 60/hr  = 60 downloads/hour, burst 10
        //   auth:     10 burst, refills 10/min = 10 auth attempts/minute
        //   search:   1000 burst, refills 1000/min (very high — bots don't hurt)
        //   shadowban download: 5 burst, refills 5/hr (friction, not a hard block)
```

`60.0 / 3600.0` is the *flow* — tokens per second — expressed in
human terms: 60 tokens per hour. The comment converts it back into
"60 downloads/hour, burst 10". That comment is the spec. We'll meet
these exact numbers again in Chapter 14, living inside the Redis bucket
limiter.

### 10.5 When a knob is a type: RecEngineMode

Some configuration isn't a number — it's a *mode*, a choice between
behaviors. FicHub's recommendation engine can run in two ways: the
original `Legacy` behavior and the new pluggable `Pluggable` strategy
system. Here's how the enum is born from an env var:

```rust
/// Recommendation engine mode. `Legacy` (default) preserves today's
/// behavior exactly; `Pluggable` routes through the strategy registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecEngineMode {
    Legacy,
    Pluggable,
}

impl RecEngineMode {
    pub fn from_env() -> Self {
        match std::env::var("REC_ENGINE_MODE").as_deref() {
            Ok("pluggable") => RecEngineMode::Pluggable,
            _ => RecEngineMode::Legacy,
        }
    }

    pub fn is_pluggable(&self) -> bool {
        matches!(self, RecEngineMode::Pluggable)
    }
}
```

This is the *correct* way to parse a mode switch. Don't use a `bool`
(`REC_PLUGGABLE=true`) — booleans can't grow. Add a third mode later
and you're stuck renaming variables. An enum with a
`from_env` that treats *any* unrecognized value as the safe default
(`Legacy` — "preserves today's behavior exactly") gives you: type
safety (you can't typo `"Pluggabel"` into a runtime string comparison,
because the value is a real `RecEngineMode`), extensibility (add a
`Hybrid` variant and a match arm), and a safe default. And `Copy` means
checking `config.rec_engine_mode.is_pluggable()` costs nothing.

### 10.6 The constructor and the struct literal

The function ends with one giant struct literal, matching every local
variable to its field:

```rust
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            ...
            agent_cooldown_domain_secs,
        }
    }
}
```

Rust's field shorthand (`database_url,` means `database_url:
database_url`) makes this ~80-line literal mostly mechanical — which is
good, because mechanical means *checkable*. And the compiler enforces
the contract at compile time: if you add a field to the struct and
forget it here, the build fails. You cannot ship a config that silently
drops a knob. The struct literal is where parsing and structure meet,
and the compiler is the referee.

### 10.7 Testing the untestable: ENV_LOCK, EnvGuard, and friends

`config.rs` ends with a test module that's a masterclass in testing
hostile code. `from_env()` reads *global process state* — environment
variables — which means every test that calls it mutates the
environment and could poison other tests. FicHub's answer has three
parts.

**Part 1 — serialize the tests.** A global mutex makes sure only one
env-mutating test runs at a time:

```rust
    /// Serializes all env-var-manipulating tests to prevent cross-pollution.
    static ENV_LOCK: Mutex<()> = Mutex::new(());
```

**Part 2 — a guard that cleans up after itself.** Instead of manually
deleting env vars after each test, a tiny RAII helper records what it
set and undoes it on `drop`:

```rust
    /// Helper to set env vars and restore on drop.
    struct EnvGuard {
        keys: Vec<String>,
    }

    impl EnvGuard {
        fn new() -> Self {
            EnvGuard { keys: Vec::new() }
        }

        fn set(&mut self, key: &str, val: &str) {
            self.keys.push(key.to_string());
            // SAFETY: Test-only env var manipulation under ENV_LOCK, single-threaded.
            unsafe { std::env::set_var(key, val); }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for key in &self.keys {
                // SAFETY: Test-only env var cleanup under ENV_LOCK, single-threaded.
                unsafe { std::env::remove_var(key); }
            }
        }
    }
```

Rust 2024 made `std::env::set_var` unsafe (it can race with other
threads reading the environment), so the tests annotate *why* the
unsafety is contained: "under ENV_LOCK, single-threaded." That's the
right way to handle `unsafe` — not with a shrug, but with a comment
explaining the invariant that makes it safe.

**Part 3 — a clean slate.** Before each test, every config env var is
removed, so no test inherits another's leftovers:

```rust
    /// Clear all known config env vars (before each test).
    fn clear_config_env() {
        let keys = [
            "DATABASE_URL", "REDIS_URL", "CACHE_DIR", "SECONDARY_CACHE_DIR",
            ...
            "SMTP_HOST", "SMTP_PORT", "SMTP_USER", "SMTP_PASS", "SMTP_FROM",
        ];
        for key in &keys {
            // SAFETY: Test-only env var cleanup under ENV_LOCK, single-threaded.
            unsafe { std::env::remove_var(key); }
        }
    }
```

(That list of keys is itself documentation — it's the *complete*
inventory of config env vars, generated by maintenance, not by design.
If you ever wonder "what env vars does FicHub read?", this test is your
answer key.)

With those three tools, the tests themselves read beautifully. Here's
the default-values test:

```rust
    #[test]
    fn test_from_env_defaults() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test_db");
        guard.set("REDIS_URL", "redis://localhost/0");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://localhost/test_db");
        assert_eq!(config.redis_url, "redis://localhost/0");
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));
        assert!(config.secondary_cache_dir.is_none());
        assert_eq!(config.export_version, 1);
        assert!(config.dynamic_rate_limit);
        assert_eq!(config.node_name, "orion");
        ...
    }
```

Set only the two required vars, load, and assert that *every* default is
exactly what the comments promised. This is the config file's contract,
locked in by tests. And the panic tests use `#[should_panic]` to verify
the fail-fast behavior:

```rust
    #[test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    fn test_panics_without_database_url() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("REDIS_URL", "redis://localhost");
        let _ = Config::from_env();
    }
```

Note the `expected = "DATABASE_URL must be set"` — the test doesn't just
assert "it panics", it asserts the *message*, so if someone changes the
message the test forces them to re-read the contract. And the custom
values test sets a dozen vars and asserts each parsed field — including
the tricky ones:

```rust
        guard.set("IP_TAG_SOURCES", "/p1,type1,tag1\n/p2,type2,tag2");
        ...
        assert_eq!(
            config.ip_tag_sources[0],
            ("/p1".to_string(), "type1".to_string(), "tag1".to_string())
        );
```

The JSON-parsing behavior gets its own pair of tests — one for valid
JSON, one for garbage:

```rust
    #[test]
    fn test_rec_site_rate_limits_invalid_json() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("DATABASE_URL", "postgres://localhost/test");
        guard.set("REDIS_URL", "redis://localhost");
        guard.set("REC_SITE_RATE_LIMITS", "not valid json");

        let config = Config::from_env();
        assert!(config.rec_site_rate_limits.is_empty());
    }
```

A malformed config value must *never* take the server down — it falls
back to the empty map, and the test proves it.

🧪 **Try It Yourself — break the config, watch the defaults.** Open a
terminal in the FicHub repo and run the config tests:

```bash
cargo test --lib config::tests -- --test-threads=1
```

(The `--test-threads=1` matters: `ENV_LOCK` serializes the tests
anyway, but running single-threaded makes the intent visible.) Then try
your own experiment in a scratch Rust file — or just eyeball the test:
set `REC_ENGINE_MODE=pluggable` and confirm `config.rec_engine_mode`
is `Pluggable`; set it to `"banana"` and confirm it's `Legacy`. Every
`_ => RecEngineMode::Legacy` arm is a decision: *unknown input gets the
safe default*. Internalize that habit — it's how production config
survives typos.

⚠️ **Watch Out — a typo'd env var is silent.** `PORT=300O` (letter O
instead of zero) fails `.parse()`, and the fallback silently kicks in:
your server binds to 3000 and you never know you typo'd. That's the
price of fail-soft. FicHub's mitigation is the test suite — every
default is asserted, so if a fallback *is* wrong, a test catches the
*default*, not your typo. When you build your own config, decide per
knob: required knobs `expect` (crash loud), optional knobs fall back
(keep going), and *documented* is better than *surprising*.

### 10.8 What config.rs teaches

Read it once more, top to bottom, and you'll see it's really five things
in one file:

1. A **contract**: the struct is the list of everything the platform can
   be told.
2. A **parser**: `from_env` converts strings to types, one knob at a
   time.
3. A **safety net**: `expect` for the two required values, defaults for
   everything else.
4. A **manual**: every field has a comment explaining units and intent.
5. A **test suite**: the contract locked in, including the failure
   modes.

When we later read `server.rs`'s `AppState` — remember it from Part 2? —
the `Config` just sits there, already parsed, already typed. The rest of
the codebase never sees an unparsed string. That's the payoff: **all the
ugly parsing lives in exactly one file, so the other 99% of the codebase
can hold a typed `Config` and never think about environment variables
again.**

Next up: the config is loaded — now let's actually *connect* to the
database it points at. Chapter 11 opens `db/mod.rs`, where a pool is
born.

---

## Chapter 11 — PostgreSQL + SQLx: db/mod.rs and models.rs

The config gave us a `database_url`. Now we need to turn that string into
a living, breathing connection to PostgreSQL. That's the job of
`src/db/mod.rs` — all forty lines of it.

### 11.1 What SQLx is (and isn't)

FicHub talks to PostgreSQL through **SQLx**, the async SQL toolkit for
Rust. Before we read code, three facts you need:

- SQLx is **async** — it plays perfectly with Tokio, the async runtime
  we met in Part 2. A database call never blocks the thread; while the
  query is in flight, the same thread serves other requests.
- SQLx is **not an ORM**. An ORM (like Diesel, or Django's ORM, or
  Rails' ActiveRecord) hides SQL behind object-oriented APIs. SQLx does
  the opposite: **you write raw SQL, and SQLx handles the plumbing** —
  connections, pooling, parameter binding, decoding rows into structs.
  FicHub's authors chose raw SQL on purpose: it's explicit, it's
  debuggable (you can paste the query into `psql`), and it puts you in
  complete control of performance.
- SQLx can check queries **at compile time** (the `query!` macros),
  but FicHub mostly uses the runtime API (`sqlx::query` and
  `sqlx::query_as`), which is more flexible — you'll see why in Chapter
  13.

💡 **Key Concept — A connection pool is a shared resource, not a
per-request resource.** Opening a PostgreSQL connection is expensive:
TCP handshake, TLS, authentication, backend process spawn — tens of
milliseconds of work you do *not* want per request. The fix is a
**pool**: a cache of open connections that requests borrow, use, and
return. With 20 connections in the pool, 20 concurrent requests can run
queries simultaneously; the 21st waits its turn. The pool is the
database's front door, and every request walks through it.

### 11.2 The whole file

Here it is. All of `src/db/mod.rs`:

```rust
pub mod models;
pub mod queries;
pub mod reviews;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

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

Three module declarations at the top (`models`, `queries`, `reviews`) —
the database layer is split into three files by concern, and this file
is the shared home. Then one function: `init_pool`. Let's take it in
three movements: the pool, the migrations path puzzle, and the run.

### 11.3 The pool: builder pattern

```rust
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
```

This is the **builder pattern**, and it's everywhere in Rust: instead of
a constructor with twenty positional arguments (unreadable, error-prone),
you chain named methods that each set one option, then call the final
`.connect()`.

- `.max_connections(20)` — the pool holds up to 20 open connections.
  Twenty concurrent database operations; anything beyond waits.
- `.acquire_timeout(Duration::from_secs(10))` — if all 20 connections
  are busy, a request will wait up to 10 seconds for one to free up.
  After that, it errors out — better a clear error than a request
  hanging forever.
- `.connect(database_url).await?` — the actual connection. It's
  `async`, so `.await`; and it returns a `Result`, so `?`.

And notice the return type of the whole function:
`Result<PgPool, sqlx::Error>`. The pool itself — not a connection, the
whole pool — is what callers get. Part 2's `AppState` held
`db: PgPool`, remember? Now you know where it came from: `init_pool`
made it. Every handler that needs the database borrows a connection from
this pool on demand.

⚠️ **Watch Out — 20 connections is a dial, not a law.** `max_connections(20)`
is tuned for FicHub's workload on its hardware. Postgres itself has a
`max_connections` limit (usually 100); if every app instance opens 20
connections and you run five instances, you're at the ceiling. When you
build your own service, start conservative and watch `pg_stat_activity`
for connection churn. Also note `acquire_timeout` — without it, a
saturated pool makes requests hang *forever*; with it, they fail with a
clear error in 10 seconds. Always set a timeout on acquisition.

### 11.4 The migrations path puzzle: dev vs. production

The next block solves a problem you'll hit the moment you deploy any
Rust app: **where do the migration files live?**

```rust
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
```

`env!("CARGO_MANIFEST_DIR")` is a **compile-time** macro: it embeds the
path of the crate's directory (where `Cargo.toml` lives) into the
binary. On your dev machine, that's
`/personal/documents/code/rust/fichub`, so
`manifest_migrations` points at the repo's `migrations/` folder.
Perfect for development.

But in production, the binary gets copied somewhere else — a Docker
container, a server — and the repo's `migrations/` folder isn't there.
So the code *also* tries `std::env::current_exe()`, the path of the
running binary, and checks whether `<binary's directory>/migrations`
exists. If it does (the deployment copied migrations next to the
binary), use that. If not, fall back to the manifest path.

So the logic reads: *prefer migrations next to the running binary;
otherwise use the build-time path.* Two environments, one function.

Then the actual run:

```rust
    if migrations_path.exists() {
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;
        tracing::info!("Database migrations applied");
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }
```

If the migrations folder exists, build a `Migrator` from it and `.run()`
it against the pool. The `Migrator` scans the directory, compares what's
there against a bookkeeping table inside the database, and applies
whatever's new. If the folder doesn't exist, log a warning — and *keep
going*. The app can still start; it just assumes the schema is already
in place. That's a deliberate choice: a missing migrations folder should
not brick the server.

The `tracing::info!` / `tracing::warn!` macros are the structured
logging system FicHub uses everywhere — you saw `tracing` mentioned in
Part 2. `info!` = "this happened and it's good news"; `warn!` = "this
isn't fatal but you should know."

### 11.5 models.rs: rows become structs

`db/mod.rs` declares `pub mod models;` — so now open `src/db/models.rs`.
This file defines what a database row *looks like* in Rust. Every table
worth reading gets a struct, and every struct has the same three derives:

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Fanfiction metadata as stored in the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub title: String,
    pub author: String,
    pub author_url: Option<String>,
    pub author_local_id: Option<String>,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub fic_created: DateTime<Utc>,
    pub fic_updated: DateTime<Utc>,
    pub status: String,
    pub source: String,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
    pub work_id: Option<i32>,
}
```

Let's decode the derives one by one, because this one line carries the
whole data layer:

- `Debug` — print the struct for logging.
- `Clone` — hand out copies.
- `Serialize, Deserialize` — convert to and from JSON (via serde).
  This is how a `FicInfo` becomes the JSON body of an API response, and
  how incoming JSON becomes a struct. In Part 2 you met
  `serde_json::json!`; this is the typed version of the same idea.
- `FromRow` — the SQLx magic. `FromRow` tells SQLx: *given a row from a
  query, fill this struct by matching column names to field names.*
  `SELECT * FROM fic_info` returns rows whose columns are `id`, `created`,
  `title`, ... and `FromRow` maps them onto this struct's fields. That's
  the entire bridge between SQL and Rust: declare the shape, derive
  `FromRow`, and rows become structs with zero manual mapping code.

Now look at the field types and compare them to the SQL you'll see in
Chapter 12's migration. `id` is a `String` because the primary key is
`VARCHAR(128)` (the URL-based id, e.g. `ffn_12345678`). `chapters` is
`i32` (SQL `INT4`), `words` is `i64` (SQL `INT8` — word counts exceed
2 billion? Not on FicHub, but the column was designed generously).
`fic_created` and `fic_updated` are `DateTime<Utc>` (SQL `TIMESTAMPTZ`).
And the `Option<T>` fields map to *nullable* columns: `author_url` may
be `NULL`, so it's `Option<String>`; `created`/`updated` are set by the
database's `DEFAULT CURRENT_TIMESTAMP`, so they're `Option` too.

💡 **Key Concept — The type is the truth.** The mapping between SQL
types and Rust types is a contract the compiler enforces. If a query
returns `NULL` into a non-`Option` field, SQLx *errors at runtime* — you
can't silently ship `null` where a `String` is promised. If you return
an `INT8` column into an `i32` field, SQLx refuses to decode. The
`Option<T>` fields aren't a style choice: they're the database's
nullability, reflected in the type system. When you write your own
models, let the schema drive the types — nullable column → `Option`,
non-null → plain.

### 11.6 A tour of the model zoo

`FicInfo` is the star, but models.rs has dozens of structs, each one a
table. A few representative ones show how the schema's story unfolds.
The request-tracking trio from migration 001:

```rust
/// Request source tracking
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestSource {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub is_automated: Option<bool>,
    pub route: Option<String>,
    pub description: Option<String>,
}

/// Request log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestLog {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub source_id: Option<i64>,
    pub etype: String,
    pub query: String,
    pub info_request_ms: i32,
    pub url_id: Option<String>,
    pub fic_info: Option<String>,
    pub export_ms: Option<i32>,
    pub export_file_name: Option<String>,
    pub export_file_hash: Option<String>,
    pub url: Option<String>,
}
```

Every request that hits the export endpoint gets a `RequestLog` row:
which source type made it (`source_id` links to `RequestSource`), what
format was requested (`etype`, e.g. `"epub"`), how long the metadata
lookup took (`info_request_ms`), and the export's file name and hash.
This is the analytics table — and in Chapter 13 we'll see the exact
function that fills it.

Then the social layer (migration 003 and friends):

```rust
/// Canonical story entry (unified work)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkRow {
    pub id: i32,
    pub canonical_title: String,
    pub canonical_author: String,
    pub description: String,
    pub default_source_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader_id: Option<i32>,
    pub is_visible: Option<bool>,
}
```

`WorkRow` is the *unified* story: the same fanfiction posted on AO3 *and*
FanFiction.Net is one `works` row with two `fic_info` rows pointing at it
(linked by `work_id`). That's a huge product decision — deduplication
across sources — and it lives in the schema. And you can see it
growing in the model: `uploader_id` and `is_visible` were added later
(migration 005, "manual uploads + moderation"), yet here they are in
the struct, `Option` because they didn't always exist.

The gamification layer shows off the type variety:

```rust
/// Daily quests
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyQuest {
    pub id: i32,
    pub quest_type: String,
    pub title: String,
    pub description: String,
    pub target_count: i32,
    pub reward_xp: i32,
    pub badge_type: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginStreak {
    pub user_id: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub last_login_date: NaiveDate,
    pub updated_at: DateTime<Utc>,
}
```

Note `NaiveDate` — a *date without timezone* — for `last_login_date`.
Streaks are measured in days, not instants; a `NaiveDate` says "the 11th
of August" with no timezone baggage. SQLx maps it to SQL `DATE`. When a
column is a calendar day and not a moment in time, `NaiveDate` is the
honest type. (And `DateTime<Utc>` maps to `TIMESTAMPTZ` — Postgres
stores the instant, converting to UTC on write.)

There are also joined views — structs that don't match one table but
match a query's output. Like this one:

```rust
/// A reading-list item joined with its work's title/author (for listing).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReadingListItemRow {
    pub id: i32,
    pub list_id: i32,
    pub work_id: i32,
    pub position: i32,
    pub blurb: String,
    pub created_at: DateTime<Utc>,
    pub canonical_title: String,
    pub canonical_author: String,
}
```

The comment says it: the query joins `reading_list_items` with `works`
to fetch the title/author in one round trip, and this struct is the
shape of that joined row. `FromRow` doesn't care whether a struct
matches a table or a `JOIN` — only that column names line up. That's the
flexibility that makes SQLx shine: the models are whatever your queries
need them to be.

### 11.7 The pattern: three files, one layer

Look back at `db/mod.rs` — three `pub mod` lines. That's the whole
architecture of the data layer, and it's worth internalizing:

- `models.rs` — **shapes**. Structs with `FromRow`. No logic.
- `queries.rs` — **actions**. Async functions that take a `&PgPool` and
  run SQL. (Chapter 13.)
- `reviews.rs` — the reviews feature's queries, split out because it
  got big enough to deserve its own file.

This is a "queries as functions" architecture: no repository objects, no
traits, no ceremony — just `pub async fn get_fic_info(pool: &PgPool, id:
&str) -> AppResult<Option<FicInfo>>`. The pool is passed explicitly; the
function is a pure operation on the database. You can test it in
isolation, read it in isolation, and grep for it by name.

🧪 **Try It Yourself — meet your database.** Spin up the stack (`docker
compose up -d postgres` if you haven't already), then connect with
`psql` and look at the tables the migrations created:

```bash
psql "$DATABASE_URL" -c "\d fic_info"
psql "$DATABASE_URL" -c "\dt"
```

`\d fic_info` shows the real columns, types, defaults, and indexes of
the table that `FicInfo` models. Compare it field-by-field with the
struct: every column has a Rust twin. Then run `\dt` and count the
tables — every one of them has (or will have) a struct in models.rs.
Reading a schema through psql and reading it through models.rs should
tell the same story; when they ever disagree, one of them is lying, and
it's your job to find out which.

⚠️ **Watch Out — `SELECT *` binds you to the model.** `get_fic_info`
uses `SELECT * FROM fic_info` and decodes into `FicInfo`. Add a column
to the table without adding a field to the struct and the query still
works (SQLx ignores extra columns) — but the reverse is fatal: add a
*field* to the struct that the table doesn't have, and every `SELECT *`
decode of that table fails at runtime. When you evolve your own schema,
add the column and the struct field in the same change, and let the
tests (Chapter 13 will show the pattern) catch the mismatch.

### 11.8 What db/mod.rs and models.rs teach

Two files, two jobs, one principle. `db/mod.rs` is *infrastructure*: it
builds the pool with sane limits, solves the dev-vs-production
migrations puzzle, and runs the migrations on startup. `models.rs` is
*vocabulary*: it declares what a row means in Rust, with the type system
carrying nullability, dates, and JSON. Together they answer the two
questions every database layer must answer: **how do we connect, and
what do rows look like?**

But wait — `init_pool` ran "the migrations". What migrations? Where did
34 numbered SQL files come from, and what story do they tell? That's
Chapter 12 — the schema's autobiography.

---

## Chapter 12 — The Migrations: 34 Files, One Schema Story

Open the `migrations/` directory in the FicHub repo. Go on — `ls` it.
You'll see 34 files, numbered `001_initial_schema.sql` through
`034_modlog.sql`. Each one is a chapter in the autobiography of a
database that grew from a simple cache into a full social platform. This
chapter is the guided tour.

### 12.1 What a migration is

A **migration** is a script that changes the database schema *in
order*. The rules are simple and absolute:

1. Migrations are numbered. `001` runs before `002`, always.
2. They run **exactly once**. SQLx's `Migrator` keeps a
   `_sqlx_migrations` bookkeeping table inside the database; after `001`
   succeeds, it's recorded as applied, and no server restart will ever
   re-run it.
3. They are **forward-only** — that's the whole point of the
   `_sqlx_migrations` table. You don't edit migration 001 after it's
   shipped; if you need a change, you write migration 035.
4. They make the schema *reproducible*: any machine can go from empty
   database to current schema by running the same 34 files in order.
   Your laptop, the staging server, the production box — identical
   results. This is the answer to "it works on my machine": it works on
   every machine, because the schema is versioned like code.

💡 **Key Concept — The database schema is code, and it needs version
control just as much as Rust does.** Nobody would edit a Rust file on
the production server by hand; the same goes for the schema. Migrations
are git for your database. And there's a second, subtler gift: because
every schema change is a numbered file with a timestamp of intent,
later you can read the *history* of the product in the migrations — and
that's exactly what this chapter does.

### 12.2 Migration 001: the founding document

`001_initial_schema.sql` is the biggest migration in the repo — nearly
500 lines — and it's the entire platform in embryo. It starts with a
header that tells you how to read everything that follows:

```sql
-- FicHub database schema — single initial migration
-- All tables in dependency order. Uses IF NOT EXISTS to be safe on existing databases.
```

*"All tables in dependency order"* — this is a rule you'll see respected
across all 34 files: a table that references another table (via a
foreign key) is created *after* it. If `request_log` references
`request_source`, then `request_source` comes first. Postgres enforces
this at creation time, so the order isn't a style preference — it's a
requirement, and the comment tells you it was deliberate.

The very first table is the request-tracking pair:

```sql
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);

CREATE TABLE IF NOT EXISTS request_log (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    source_id BIGINT REFERENCES request_source(id),
    etype TEXT NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4 NOT NULL,
    url_id TEXT,
    fic_info TEXT,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);
```

Let's decode the vocabulary, because it repeats across the entire
schema:

- `BIGSERIAL` — an auto-incrementing 64-bit integer. `SERIAL` is 32-bit.
  FicHub uses `BIGSERIAL` for anything that could grow large
  (`request_log` rows — one per request! — definitely qualify).
- `PRIMARY KEY` — the unique identifier of each row, indexed
  automatically.
- `TIMESTAMPTZ` — timestamp *with* time zone. Postgres normalizes to
  UTC internally. Always `TIMESTAMPTZ`, never `TIMESTAMP`, for
  "when did this happen" columns.
- `DEFAULT CURRENT_TIMESTAMP` — if the insert doesn't specify the
  column, the database fills it with now. This is why the Rust struct's
  `created` field is `Option<DateTime<Utc>>` — the app usually *doesn't*
  set it.
- `REFERENCES request_source(id)` — a **foreign key**: `source_id` must
  be a real `request_source` row's id. The database refuses to insert a
  dangling reference. This is referential integrity, enforced by the
  database, not by your code.
- `INT4` / `INT8` — Postgres spellings for 32-bit / 64-bit integers
  (`INT4` = `i32` in Rust, `INT8` = `i64`).
- `TEXT` — unbounded text. Postgres's `VARCHAR(n)` has a length limit;
  `TEXT` doesn't, and for descriptions and URLs, unbounded is the safe
  call.

Now, `request_log` also has a subtle design decision: it's a *log*, so
it gets **indexes** tuned for how it's *read*, not how it's written.
Look:

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);
CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

The second one is a **partial index** — it only indexes rows where an
export file actually exists. That's the "exports per day" dashboard
query, made fast by indexing only what the query cares about. Indexes
aren't free (every insert must update them), so FicHub indexes
deliberately: only the query patterns that matter, and sometimes only
the subset of rows that matter.

The centerpiece of migration 001 is the `fic_info` table — the heart of
the whole platform:

```sql
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id INT8,
    author_id INT8,
    content_hash VARCHAR(256),
    raw_json JSONB,
    work_id INTEGER,  -- FK to works, added after works is created
    text_search tsvector
        GENERATED ALWAYS AS (
            setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
            setweight(to_tsvector('english', coalesce(description,'')), 'B')
        ) STORED
);
```

So many lessons in one table. The `id` is `VARCHAR(128)` — a human-made
key (`ffn_12345678`), not an auto-increment. `fic_created` /
`fic_updated` are `NOT NULL` — these are *facts about the story* (when
the author posted/updated it on the source site), always known. The
`source` column says *where* the fic came from (ao3, ffn, ...).

And then there's the fascinating part — the **generated column**:

```sql
    text_search tsvector
        GENERATED ALWAYS AS (
            setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
            setweight(to_tsvector('english', coalesce(description,'')), 'B')
        ) STORED
```

Postgres full-text search: `to_tsvector('english', ...)` turns text
into a searchable vector of lexemes (stemmed words); `setweight(...,'A')`
marks title words as high-priority; the `||` concatenates title-vector
with description-vector; `GENERATED ALWAYS AS ... STORED` means the
database maintains this column *itself* — every insert and update
recomputes it. Your app never writes it. It's the search index, kept
fresh by the database. Then:

```sql
CREATE INDEX IF NOT EXISTS idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

A `GIN` index (generalized inverted index) is what makes `WHERE
text_search @@ to_tsquery('english', $1)` fast. That's the engine behind
FicHub's full-text search — no external search service, just Postgres.

Migration 001 continues: `export_log` (with a `UNIQUE(url_id, version,
etype, input_hash)` — the cache-key of exports), blacklists,
`fic_version_bump` (cache invalidation), and then the user system:

```sql
-- User roles: 0=reader, 1=curator, 2=senior_curator, 3=admin
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE,
    role SMALLINT NOT NULL DEFAULT 0,
    reputation INT4 NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    curator_since TIMESTAMPTZ,
    curator_status TEXT DEFAULT 'active' CHECK (curator_status IN ('active','suspended'))
);
```

`role` is an integer with a comment explaining the enum (`0=reader,
1=curator, 2=senior_curator, 3=admin`). `CHECK (curator_status IN
('active','suspended'))` is a **constraint**: the database refuses any
other value. Two different ways to encode "a small set of allowed
values" — integer enum (cheap, compact) vs. CHECK on text (readable,
self-documenting) — both used deliberately.

Then comes the clever part — the `works` table, and the *reason* the
migration is ordered the way it is:

```sql
CREATE TABLE IF NOT EXISTS works (
    id SERIAL PRIMARY KEY,
    canonical_title TEXT NOT NULL DEFAULT '',
    canonical_author TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    default_source_id TEXT,  -- FK to fic_info(id)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE works IS 'Canonical story entries. One work = one story across all platforms.';
COMMENT ON COLUMN works.canonical_title IS 'Canonical title across all sources';
```

The `COMMENT ON` statements are a gift to future developers: the schema
*itself* carries the product explanation ("One work = one story across
all platforms"). And notice `fic_info.work_id` — declared in `fic_info`
*above* — references `works`, which doesn't exist yet. That's why the
foreign key is added *after* `works` is created, in a `DO` block that
checks for its existence first:

```sql
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_fic_info_work_id'
    ) THEN
        ALTER TABLE fic_info
            ADD CONSTRAINT fk_fic_info_work_id
            FOREIGN KEY (work_id) REFERENCES works(id);
    END IF;
END $$;
```

The `DO $$ ... $$` block is a procedural guard — "add this constraint
only if it isn't there." Combined with `IF NOT EXISTS` everywhere, the
whole migration is **idempotent**: you can run it twice and nothing
breaks. That's a recurring theme in these files ("Uses IF NOT EXISTS to
be safe on existing databases") — a defensive habit born from real
deployments where a partial run left a half-applied schema.

Migration 001 also seeds reference data with `INSERT ... ON CONFLICT
DO NOTHING` — the tag types, the default OPDS shelf:

```sql
INSERT INTO tag_types (id, name) VALUES
    (1, 'fandom'),
    (2, 'character'),
    (3, 'relationship'),
    (4, 'freeform'),
    (5, 'warning'),
    (6, 'category'),
    (7, 'other')
ON CONFLICT (id) DO NOTHING;
```

`ON CONFLICT (id) DO NOTHING` is the idempotent insert: if the row
exists, skip it. Seed data that survives re-runs.

And the *pièce de résistance* — a trigger function. The tags system
keeps a running `score` per fic-tag pair, updated automatically by the
database whenever a vote is cast:

```sql
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value
        WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value
        WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

Then three triggers wire it up:

```sql
CREATE TRIGGER trg_fic_tag_vote_insert
    AFTER INSERT ON fic_tag_votes
    FOR EACH ROW EXECUTE FUNCTION update_fic_tag_score();
```

This is **logic in the database** — and it's the *right* place for it.
The score must stay consistent no matter *which* code path casts a vote
(web UI, API, admin script, future feature). If the score lived in
application code, every caller would have to remember to update it.
Here, the database guarantees it — atomically, transactionally, for
every path, forever. The `CREATE OR REPLACE FUNCTION` + `DROP TRIGGER IF
EXISTS` + `CREATE TRIGGER` trio makes the whole thing re-runnable.

⚠️ **Watch Out — triggers are powerful and invisible.** Code in a
trigger runs on *every* qualifying write, even writes from `psql`, even
writes from scripts you forgot about. That's the superpower and the
trap: when you see a `fic_tags.score` value change and you don't know
why, the trigger is why. Debugging tip: when data changes "on its own,"
search the migrations for `CREATE TRIGGER` and `GENERATED ALWAYS` first
— those are the database's own hands.

### 12.3 The story of growth: migrations 002–034

Migration 001 is the whole app in one file. The other 33 files are the
story of the app *growing* — and reading their headers is reading the
product's history. Let's walk the timeline in five movements.

**Movement 1 — analytics and trust (002, 007, 009, 010, 013).** The
platform starts logging better. Migration 002 adds client tracking to
`request_log` — an anonymous client id and a user agent:

```sql
-- Add client tracking to request_log
-- Tracks unique visitors and their actions (anonymous - no IP storage)

ALTER TABLE request_log 
ADD COLUMN IF NOT EXISTS client_id VARCHAR(36),
ADD COLUMN IF NOT EXISTS user_agent TEXT;

-- Index for unique visitor queries
CREATE INDEX IF NOT EXISTS idx_request_log_client_id 
ON request_log(client_id) WHERE client_id IS NOT NULL;
```

Note the comment: *"anonymous — no IP storage"*. Zero-PII is a design
principle you'll see repeated in nearly every migration's header —
FicHub deliberately does not store raw IPs in its analytics. `client_id`
is a UUID the browser generates; it can't be traced back to a person.
That's a *privacy-by-design* decision encoded in the schema. (Later,
migration 009 adds an `ip inet` column to `request_log` for abuse
detection — but the header is careful to say "Zero-PII: we store only
the IP needed for abuse detection; the admin API never exposes raw IPs.")

Migration 007 brings the admin dashboard, 009 adds bot scoring
foundation, 010 adds `search_queries` analytics with this header gem:

```sql
-- 010: Search analytics — per-search logging + trope popularity.
-- ...
-- Zero-PII: only aggregate counts are exposed, never client_id -> query maps.
```

And 013 adds a `kindle_email` column to `users` — one line, but it's the
entire send-to-Kindle feature's schema:

```sql
-- 013: Send-to-Kindle support.
-- Adds an optional per-user Kindle email address used by POST /api/send-to-kindle.
ALTER TABLE users ADD COLUMN IF NOT EXISTS kindle_email TEXT;
```

**Movement 2 — social features (003, 004, 008, 017, 018, 019, 020).**
Migration 003 is the big one: follows, notifications, gamification, and
translations. Its `follows` table shows a brilliant constraint:

```sql
CREATE TABLE IF NOT EXISTS follows (
    id BIGSERIAL PRIMARY KEY,
    follower_id INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    followee_id INT4 REFERENCES users(id) ON DELETE CASCADE,      -- follow a user
    work_id INT4 REFERENCES works(id) ON DELETE CASCADE,          -- follow a work
    author_name TEXT,                                              -- follow an author by name
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT follows_target CHECK (
        num_nonnulls(followee_id, work_id, author_name) = 1
    )
);
```

You can follow a *user*, a *work*, or an *author by name* — but exactly
one of those, and the **CHECK constraint enforces it**: `num_nonnulls(...)
= 1` means "of these three columns, exactly one must be non-null." The
database refuses a follow row with two targets or none. This is the
kind of invariant that application code *shouldn't* be trusted to
maintain — the database guarantees it for every code path.

`ON DELETE CASCADE` appears constantly — delete a user, and their
follows, bookmarks, badges, and notifications vanish with them. No
orphan rows, no manual cleanup. Migration 004 adds shelves and reading
status; 008 adds author profiles and merging; 017 adds the "updated
since you last looked" tracking; 018 adds fic requests (a prompt
board); 019 adds reading lists; 020 adds series.

**Movement 3 — AI arrives (011, 012, 015, 021, 028).** Now the schema
gets *intelligent*. Migration 011 introduces pgvector and the Roadmap
Consensus Engine — semantic clustering of user-submitted feature
requests:

```sql
-- 011: Roadmap Consensus Engine — semantic clustering + MaxDiff/Elo voting.
-- ...
-- Embeddings use pgvector (0.6.0 available). The Ollama API (nomic-embed-text)
-- produces 768-dim vectors (nomic-embed-text is 768-d, NOT 384 — the blueprint's
-- 384 was for all-MiniLM-L6-v2; we use nomic-embed-text since it's installed).

CREATE EXTENSION IF NOT EXISTS vector;
```

`CREATE EXTENSION IF NOT EXISTS vector;` — this is where **pgvector**
enters the story: PostgreSQL grows a `VECTOR` column type and vector
similarity operators. The comment even preserves a *design argument* —
"768-d, NOT 384" — the kind of reasoning that would otherwise be lost
forever. Migration 012 adds trigram fuzzy search (`pg_trgm`) so "Hary
Pottr" still finds "Harry Potter". Migration 015 adds the auto-tagger
(embedding-based tag suggestions with a review queue). Migration 021
adds LLM comment triage. Migration 028 seeds i18n translations. The
schema is no longer just tables — it's embedding vectors, similarity
thresholds, and AI review queues.

💡 **Key Concept — Extensions are how Postgres grows beyond a
relational database.** `CREATE EXTENSION vector` gives you machine
learning. `CREATE EXTENSION pg_trgm` gives you fuzzy string matching.
`CREATE EXTENSION` is Postgres's plugin system — and FicHub uses it to
turn one database into the whole backend: relational store, full-text
search, vector search, and analytics, all in one server.

**Movement 4 — moderation and trust (014, 022, 023, 024, 025, 031,
032, 034).** A platform with users needs guards. Migration 014 reworks
feedback into 5-star ratings (with a careful design note about keeping
legacy 1/-1 rows valid — the CHECK constraint admits both namespaces).
022 adds user reports. 023 adds a translation review workflow — the
schema comment spells out the state machine: *"Machine/user
translations land as 'draft'; curators post-edit, approve or reject
them before they go live."* 031 lets curators override bad scraped
bodies; 032 makes those overrides *votable* so "a single curator cannot
silently replace a fic's content" — a governance rule encoded in a
schema comment. And 034 is the modlog:

```sql
-- Moderation log (modlog): a transparent, public-by-default record of every
-- moderator / curator / admin action. ANY logged-in user can read it — the
-- point is that moderation is completely transparent.
CREATE TABLE IF NOT EXISTS modlog (
    id            BIGSERIAL PRIMARY KEY,
    actor_id      INTEGER,
    actor_username TEXT,
    action        TEXT NOT NULL,
    target_type   TEXT NOT NULL DEFAULT '',
    target_id     TEXT NOT NULL DEFAULT '',
    details       JSONB NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Note `details JSONB NOT NULL DEFAULT '{}'` — a flexible, schema-less
bag for whatever an action needs to record. When you don't know the
shape of data in advance, `JSONB` is Postgres's answer: validated JSON,
indexable, queryable, but free-form. And the header declares a
*philosophy*: moderation is transparent, public by default. The last
migration in the series is the platform saying "we take our own
community seriously."

**Movement 5 — scale and automation (026, 027, 029, 030, 033).** The
final migrations are about recommendation engines and self-healing.
Migration 027 is a showpiece — the pluggable recommendation platform.
Its header is a design document in itself:

```sql
-- 027: Pluggable recommendation platform — Stage 0 schema.
--
-- DESIGN NOTES
-- ------------
-- Everything here is INERT: tables are empty until a strategy runs, and the
-- default `REC_ENGINE_MODE=legacy` code path never touches them. No existing
-- behavior changes (golden test: legacy mode output is byte-identical before
-- and after this migration).
```

*"Everything here is INERT"* — the tables exist but nothing writes to
them yet. The migration is *staged*: schema first, code later, with a
golden test guaranteeing no behavior change. That's how you ship a big
rewrite without breaking a running platform. The migration then creates
the strategy registry's data: `rec_user_signals` (unified per-user
signal view with weights — "bookmarks weight 4.0, downloads 2.0,
ratings = rating, reviews 3.0"), `rec_embeddings` (pgvector, with an
HNSW index for ANN search):

```sql
-- HNSW index for ANN search (cosine). Name is unique across the schema.
CREATE INDEX IF NOT EXISTS idx_rec_embeddings_hnsw
    ON rec_embeddings USING hnsw (embedding vector_cosine_ops);
```

`USING hnsw` — the **HNSW** (Hierarchical Navigable Small World) index:
the data structure that makes "find the 20 most similar embeddings"
fast, the standard for vector search. And `rec_bandit_arms` — a
contextual bandit's Beta posteriors, with the comment explaining the
math: "engagement increments alpha, a shown-but-not-engaged impression
increments beta." The schema doesn't just store data; it stores the
*machine learning system's state*.

Then 029 and 030 bring the self-healing agent (scrape failure
telemetry, an agent run ledger), and 033 usage analytics.

### 12.4 What the migrations teach

Read all 34 headers and you've read the product's history:
cache → analytics → social → gamification → AI → moderation →
recommendations → self-healing. Here's the meta-lesson: **the schema
tells the truth about a platform better than any README**, because the
schema can't lie — it's what the code actually runs against.

And the practical habits to steal:

- **Number strictly, run once, never edit the past.** New change = new
  file.
- **`IF NOT EXISTS` + `ON CONFLICT DO NOTHING` + `DO $$` guards** for
  idempotency — re-running a migration must be safe.
- **`COMMENT ON` your tables and columns.** The schema should explain
  itself.
- **Constraints over code.** `CHECK`, `UNIQUE`, `REFERENCES`, and
  triggers keep invariants in the database, where they can't be
  forgotten.
- **Indexes are deliberate.** Partial indexes, GIN for text, HNSW for
  vectors — each one exists because a query needs it.
- **Write the "why" in the header.** Design decisions, dead ends,
  golden tests — the migration header is the only place these survive.

🧪 **Try It Yourself — read a schema like a book.** In the repo, pick
any migration from the list above (say, `003` for follows or `027` for
recs) and read its header comment, then `psql "$DATABASE_URL" -c "\d
follows"` to see the live table. Notice how the header's design notes
match the real columns. Then try a historian's trick: run `\dt` and see
if you can guess *which migration* created each table before checking.
After a few tables you'll start reading the product's timeline from
memory — that's schema literacy, and it's a real skill.

⚠️ **Watch Out — migrations that "fix" data can be one-way doors.**
Schema changes are easy to apply and hard to undo. Dropping a column,
changing a column type, or rewriting rows in a `DO` block is
*irreversible* — and in FicHub's case, the files literally say
"forward-only" (see the 027 header). Before you write a destructive
migration, ask: what happens to the data? Is there a backup? Can the
change be staged (add column → backfill → swap code → drop old
column)? FicHub's answer to almost every destructive need is: *add a
new column, migrate the code, drop the old one later*. Slow, boring,
and safe.

Now that the schema exists, how does the code actually *talk* to it?
Time for the biggest file in the database layer: `db/queries.rs` — the
2,700-line SQL workhorse. That's Chapter 13.

---

## Chapter 13 — queries.rs: The Giant Query Module

Open `src/db/queries.rs` and scroll. Just scroll. You'll pass `pub async
fn` after `pub async fn` — get_fic_info, upsert_fic_info,
insert_request_log, find_export_log, search_similar_fics,
lookup_tag_by_name, follow_user, create_notification, award_badge,
record_login_streak, compute_weekly_leaderboard, create_work,
cast_proposal_vote... nearly 150 functions across 2,700 lines. This is
where the entire application meets the database: every page, every API
response, every background job's data, all flowing through functions
that look almost identical.

This chapter reads the three functions that matter most for the
platform's core loop — *a user requests an export*: `upsert_fic_info`
(the cache writer), `get_fic_info` (the cache reader), and
`insert_request_log` (the auditor).

### 13.1 The shape of a query function

Almost every function in queries.rs follows the same skeleton. Let's
dissect the writer first:

```rust
/// Upsert a fic_info record (INSERT ON CONFLICT UPDATE)
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW())
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            author = EXCLUDED.author,
            chapters = EXCLUDED.chapters,
            words = EXCLUDED.words,
            description = EXCLUDED.description,
            fic_updated = EXCLUDED.fic_updated,
            status = EXCLUDED.status,
            extra_meta = EXCLUDED.extra_meta,
            raw_extended_meta = EXCLUDED.raw_extended_meta,
            content_hash = EXCLUDED.content_hash,
            updated = NOW()"#,
    )
    .bind(&fic.id)
    .bind(&fic.title)
    .bind(&fic.author)
    .bind(&fic.author_url)
    .bind(&fic.author_local_id)
    .bind(fic.chapters)
    .bind(fic.words)
    .bind(&fic.description)
    .bind(fic.fic_created)
    .bind(fic.fic_updated)
    .bind(&fic.status)
    .bind(&fic.source)
    .bind(&fic.extra_meta)
    .bind(&fic.raw_extended_meta)
    .bind(fic.source_id)
    .bind(fic.author_id)
    .bind(&fic.content_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

Read the skeleton, and you'll never be lost in this file again:

1. **A doc comment** saying what the function does, in one line.
2. **`pub async fn`** — public (other modules call it), asynchronous
   (never blocks the runtime).
3. **`pool: &PgPool`** — the *first* parameter of nearly every function.
   The pool is passed explicitly, never stored globally. This is the
   "queries as functions" architecture from Chapter 11: no repository
   objects, no hidden state — the function is a pure operation on the
   database.
4. **The SQL string** — written in a raw string literal `r#"..."#` so
   quotes and newlines pass through untouched. The SQL is *the* core;
   everything else is plumbing.
5. **Placeholders `$1, $2, ...`** — SQLx's parameter binding. **Never**
   string interpolation. This is the single most important security
   habit in this file, and we'll come back to it.
6. **`.bind(...)` chains** — one per placeholder, in order. The types
   flow from the struct: `&fic.id` is a `&String`, `fic.chapters` is an
   `i32`.
7. **`.execute(pool).await?`** or `.fetch_one(pool).await?` — run it.
8. **`Ok(())`** — return success, or let `?` bubble the `sqlx::Error`
   up (Part 2 taught us how `AppError` catches it).

That's it. Learn the skeleton, and 90% of queries.rs reads itself.

Now look at the *SQL* itself, because it's a masterpiece of database
thinking. This is an **upsert**: INSERT, and if the row already exists
(conflict on the primary key `id`), UPDATE it instead. The mechanism is
Postgres's `ON CONFLICT ... DO UPDATE`. And here's the key detail:

```sql
        ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
```

`EXCLUDED` is Postgres's name for "the row we *tried* to insert." So
`title = EXCLUDED.title` means "overwrite the stored title with the new
one." The effect: the scraper fetches a fic, calls `upsert_fic_info`,
and *one* statement either creates the row or refreshes it. No
SELECT-then-INSERT race, no "row exists?" check in Rust — the database
decides, atomically.

⚠️ **Watch Out — read the upsert's column list carefully.** Count the
`SET` clauses: `title, author, chapters, words, description,
fic_updated, status, extra_meta, raw_extended_meta, content_hash` —
ten columns. Now count the INSERT columns: eighteen. The difference?
`created` isn't touched (it stays the *first* time we saw the fic —
that's the whole point of `created`), `author_url` and
`author_local_id` aren't refreshed (they rarely change), and
`fic_created` isn't updated (that's the *author's* publish date — it
doesn't change when the fic gets a new chapter). An upsert's SET list
is a *decision* about which fields are refreshable. When you write your
own, ask of every column: should a re-insert overwrite this? If not,
leave it out — and comment why.

💡 **Key Concept — One round trip beats three.** Without `ON
CONFLICT`, the naive "save" logic is: SELECT the row, decide insert or
update, run the write. Three round trips, with a race window between
them (two requests could both see "missing" and both INSERT — one
fails on the unique key). The upsert collapses all of it into *one*
atomic statement. Whenever you catch yourself writing
"check then write" logic, look for the single-statement version:
`ON CONFLICT`, `INSERT ... RETURNING`, or a `MERGE` — your future self
will thank you.

### 13.2 The reader: get_fic_info

Now the reader — the function that runs on *every* fic page and *every*
export:

```rust
/// Get fic_info by ID
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

Five lines. This is the *entire* function, and it introduces the second
SQLx API: **`query_as`**. Where `sqlx::query` returns raw rows you'd
have to decode by hand, `query_as::<_, FicInfo>` decodes each row into
a `FicInfo` automatically — using the `FromRow` derive we met in
Chapter 11. The `_` is "infer the parameter types"; `FicInfo` is the
output type. The column names in the result (`id`, `title`, `chapters`,
...) are matched to struct fields by name, and SQLx *validates the types
at runtime* — a `NULL` into a non-`Option` field is an error, a text
column into an `i32` field is an error.

And note `fetch_optional` vs `fetch_one` vs `fetch_all` — SQLx's trio
of row-collectors:

- `fetch_one` — exactly one row, or error.
- `fetch_optional` — at most one row: `Some(row)` or `None`. Perfect
  for "get by primary key" — the fic may not exist, and `None` is the
  honest answer (no error, no `unwrap`).
- `fetch_all` — any number of rows, as a `Vec`.

The return type says it all: `AppResult<Option<FicInfo>>` — "either an
error, or a maybe-fic." The caller (a route handler) matches on
`Some(fic)` vs `None` and returns a 404 or the JSON. No exceptions, no
null-pointer surprises — the "not found" case is a first-class citizen
of the type system.

### 13.3 The auditor: insert_request_log

Every export request writes a row to `request_log`. Here's the function:

```rust
/// Log a request
pub async fn insert_request_log(
    pool: &PgPool,
    source_id: i64,
    etype: &str,
    query: &str,
    info_request_ms: i32,
    url_id: Option<&str>,
    fic_info: Option<&str>,
    export_ms: Option<i32>,
    export_file_name: Option<&str>,
    export_file_hash: Option<&str>,
    url: Option<&str>,
    client_id: Option<&str>,
    user_agent: Option<&str>,
    ip: Option<std::net::IpAddr>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO request_log
          (source_id, etype, query, info_request_ms, url_id, fic_info, export_ms, export_file_name, export_file_hash, url, client_id, user_agent, ip)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::inet)"#,
    )
    .bind(source_id)
    .bind(etype)
    .bind(query)
    .bind(info_request_ms)
    .bind(url_id)
    .bind(fic_info)
    .bind(export_ms)
    .bind(export_file_name)
    .bind(export_file_hash)
    .bind(url)
    .bind(client_id)
    .bind(user_agent)
    .bind(ip.map(|i| i.to_string()))
    .execute(pool)
    .await?;
    Ok(())
}
```

Thirteen parameters, thirteen placeholders, one INSERT. Two things to
notice.

First, the `Option` pattern: `url_id`, `fic_info`, `export_ms`,
`export_file_name`, `export_file_hash`, `url`, `client_id`,
`user_agent`, `ip` are all `Option<&str>` (or `Option<IpAddr>`). A
request *might* have an export file name (if the export succeeded), it
*might* have a client id (if the browser sent the header). SQLx binds
`None` as SQL `NULL` automatically, and the migration's columns are
nullable to match. The Rust `Option` and the SQL `NULL` are the same
idea in two languages, and they line up perfectly.

Second — the cast at the very end:

```sql
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::inet)
```

`$13::inet` — the 13th parameter is explicitly cast to Postgres's
`inet` type (the IP-address type we saw in migration 009). The Rust
side binds `ip.map(|i| i.to_string())` — an `IpAddr` converted to its
string form (`"93.184.216.34"` or `"2001:db8::1"`). The `::inet` cast
tells Postgres to parse that string as an IP address. The cast is
necessary because SQLx can't infer the column type from `$13` alone —
the table column exists, sure, but an explicit cast makes the intent
unambiguous and avoids surprises. When you pass a value that needs a
Postgres type with special parsing (`inet`, `jsonb`, `vector`, arrays),
cast the placeholder: `$1::inet`, `$2::jsonb`, `$3::vector`.

Also note: `info_request_ms: i32` — the *milliseconds the metadata
lookup took*. This function isn't just a log; it's the platform's
performance instrumentation. Every request carries its timing, and the
admin dashboards (migration 007) aggregate these columns.

### 13.4 Two more patterns: query_scalar and tuples

Two more query shapes round out the toolkit. When you need *one value*
instead of a row, there's `query_scalar`:

```rust
/// Look up an alias, returning the canonical tag ID
pub async fn lookup_alias(pool: &PgPool, alias_name: &str) -> AppResult<Option<i32>> {
    let row = sqlx::query_scalar::<_, i32>(
        "SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = $1",
    )
    .bind(alias_name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

`query_scalar::<_, i32>` — the result is a single `i32`, not a struct.
For "give me one number" queries, this skips the struct entirely.

And when a query returns a *handful* of columns that don't deserve
their own struct, SQLx decodes into a tuple — remember
`insert_request_source` returning the new id:

```rust
pub async fn insert_request_source(pool: &PgPool, is_automated: bool, route: &str, description: &str) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO request_source (is_automated, route, description)
           VALUES ($1, $2, $3)
           ON CONFLICT (is_automated, route, description) DO UPDATE SET route = EXCLUDED.route
           RETURNING id"#,
    )
    .bind(is_automated)
    .bind(route)
    .bind(description)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

Two moves here: `RETURNING id` (Postgres hands back the inserted row's
id — *one* round trip for insert-and-get-key, no second SELECT) and
`ON CONFLICT ... DO UPDATE SET route = EXCLUDED.route` — a crafty
idempotent insert: if the `(is_automated, route, description)` triple
already exists, "update" it to itself and return *its* id. Either way,
`RETURNING id` gives you the canonical row's id. That's how FicHub
deduplicates request sources: the `UNIQUE` constraint from migration
001 is the arbiter, and the query just asks "give me the id, whatever
it is." And the tuple `(i64,)` (with the trailing comma!) is how Rust
spells "one-element tuple" — `query_as` decoding into it, then
`row.0` to pull the value out.

### 13.5 Boring, on purpose

2,700 lines of this. Same skeleton, over and over. Is that a failure of
imagination? No — it's a *choice*, and it's the right one. Compare the
alternatives:

- An ORM would hide the SQL behind method calls — but then every query
  is a black box you can't paste into `psql`, and performance
  mysteries hide behind abstraction.
- A query builder would add a layer of API to learn, for zero benefit
  on queries this simple.
- Hand-rolled per-feature data layers would mean 150 different
  conventions instead of one.

Instead: one skeleton, learned once, readable forever. Every function
in this file answers the same three questions at a glance — *what SQL,
what parameters, what comes back*. When a bug surfaces, you read the
SQL directly — no translation layer. When you need a new query, you
copy the nearest function and adjust. This is the "small, boring,
well-commented" philosophy from Part 2, applied to the data layer.

🧪 **Try It Yourself — trace a request through the queries.** Start the
server, then open a second terminal and watch the database react:

```bash
psql "$DATABASE_URL" -c "SELECT id, source, chapters, words FROM fic_info ORDER BY created DESC LIMIT 5;"
```

Then request a fic in the browser (`http://localhost:3000/download/...`
or any fic page). Re-run the query: the row either appeared (the
scraper upserted it) or its `updated` timestamp moved (the upsert
refreshed it). Then check the auditor's work:

```bash
psql "$DATABASE_URL" -c "SELECT etype, info_request_ms, url_id FROM request_log ORDER BY id DESC LIMIT 5;"
```

Every row in `request_log` came from `insert_request_log` — 13 bound
parameters, one INSERT. You're now watching the Chapter 13 functions
execute in real time. That's the whole loop: `get_fic_info` reads,
`upsert_fic_info` writes, `insert_request_log` records.

⚠️ **Watch Out — SQL injection is a `format!` away.** The `$1`
placeholders exist for a reason: `bind` sends the value *separately
from the SQL*, so no matter what the user types, the database never
mistakes it for SQL. The moment you write
`format!("SELECT * FROM fic_info WHERE id = '{}'", id)`, you've opened
the door — a `id` of `"'; DROP TABLE fic_info; --"` becomes a second
statement. Rule: **never build SQL with string interpolation. Ever.**
If you find yourself reaching for `format!` inside a query, stop and
find the placeholder version. FicHub's 2,700 lines are a monument to
this single habit.

### 13.6 What queries.rs teaches

One file, one pattern, three APIs (`query`, `query_as`, `query_scalar`),
and a handful of Postgres superpowers (`ON CONFLICT`, `RETURNING`,
`::inet` casts, `fetch_optional` for the maybe-row). The entire data
layer is now in your head: models declare the shapes, queries declare
the operations, and the pool makes them fast.

But there's a second database in this platform, and it's the one that
stands between FicHub and the bots. Redis. It holds the token buckets,
the shadowban list, and the proof-of-work solves — and it's the subject
of the final chapter of this part.

---

## Chapter 14 — Redis: Buckets, Bans, and Proof of Work

PostgreSQL holds FicHub's *memory*: the fics, the users, the votes.
Redis holds FicHub's *reflexes*: how fast anyone may go, who's on the
naughty list, and who has proved they're human. It's the bouncer at the
door, and it's a lot smarter than it looks.

Redis is an **in-memory key-value store**. Keys are strings; values can
be strings, hashes, lists, sets, and more; and everything lives in RAM,
so operations are microseconds. That's why rate limiting lives here: a
rate-limit check has to happen on *every single request* — it can't
afford a disk round trip. Postgres answers questions about *history*
("what fics does this user follow?"); Redis answers questions about
*now* ("has this IP used up its allowance this minute?").

The code lives in two places: `src/limiter/` (the rate limiter itself)
and `src/services/` (the proof-of-work helper, among others). We'll
read both.

### 14.1 The limiter's front door: limiter/mod.rs

`src/limiter/mod.rs` is only 90 lines, and it defines the *shape* of
rate limiting — the traits and types the rest of the platform depends
on. It starts with the answer a request can get:

```rust
/// Per-request rate limiting result
#[derive(Debug)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Wait this many seconds before retrying
    Wait(u64),
    /// Blocked (datacenter IP, etc.)
    Blocked,
}
```

Three outcomes, encoded as an enum — this is the "make illegal states
unrepresentable" principle from Part 2: a rate-limit check *cannot*
return a string like `"maybe"`; it returns one of exactly three
variants, and the compiler makes you handle all of them. `Wait(u64)`
even carries *how many seconds* to wait — the HTTP 429 response can
tell the client exactly when to retry.

Then the traits. The old-style limiter (the `RateLimiter` trait), and
the newer tiered one:

```rust
/// Endpoint-class tiers for the tiered rate limiter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// /api/epub, /api/v0/epub, /api/download/*, /cache/*, /api/upload* —
    /// strict (60/hr per IP by default).
    Download,
    /// /api/auth/*, /login, /register — strict (10/min per IP).
    Auth,
    /// /api/search, /docs, /static — very high (1000/min).
    Search,
    /// Everything else — moderate (the legacy global bucket).
    Default,
}
```

Here's the design idea, and it's a good one: **not all requests are
created equal.** Downloading an EPUB costs real bandwidth and CPU (and
is what scrapers want to do), so it gets the strictest bucket — 60 per
hour. Trying passwords (`/api/auth/*`) is also abuse-prone — 10 per
minute. But *searching* is cheap and bots searching don't hurt
anybody — 1,000 per minute. One limiter, four dials, tuned per
endpoint class. And note the doc comments: each variant *lists the
routes it covers*, so the mapping is documented right where the enum is
defined.

The `TieredRateLimiter` trait spells out the full job:

```rust
#[async_trait::async_trait]
pub trait TieredRateLimiter: Send + Sync {
    /// Check a request against the tier bucket for `tier`. `client_id` is the
    /// `X-Client-Id` header value when present.
    async fn check(&self, ip: IpAddr, client_id: Option<&str>, tier: Tier) -> TieredRateLimitResult;

    /// Map a request path to its tier.
    fn tier_for_path(&self, path: &str) -> Tier;

    /// True when `client_id` is shadowbanned (in the `fichub:shadowban` set).
    fn is_shadowbanned(&self, client_id: &str) -> bool;

    /// Add a client_id to the shadowban set for `ttl_seconds` (friction, not a
    /// hard block: shadowbanned clients get a much stricter download bucket).
    fn shadowban(&self, client_id: &str, ttl_seconds: u64);

    /// Remove a client_id from the shadowban set (admin un-shadowban).
    fn unshadowban(&self, client_id: &str);
}
```

A trait is a *contract*: anything implementing `TieredRateLimiter`
promises these five operations. Note what's *not* here: the check takes
both `ip` **and** `client_id` (the anonymous browser UUID we met in
migration 002 — so the limiter can be fair to identified clients),
shadowbanning is "friction, not a hard block" (the comment again!), and
there's an `unshadowban` for admins. And why a trait at all, instead of
a concrete struct? Because `AppState` (Part 2!) holds the limiter as
`Arc<dyn TieredRateLimiter + Send + Sync>` — the trait lets the server
swap limiter implementations without touching the routes, and tests can
substitute a fake.

💡 **Key Concept — A trait is a promise with multiple implementations.**
`Arc<dyn TieredRateLimiter>` in `AppState` means "anyone holding this
state can ask 'check this request' without knowing *how* it's
implemented." Production uses the Redis bucket limiter; a test could
use a limiter that always says `Allowed`. The routes don't care — and
that's the whole point of the trait. When you see `dyn` in a codebase,
someone is buying the freedom to swap implementations.

One more gem in mod.rs — how FicHub finds the *real* client IP:

```rust
/// Extract the real client IP from the request headers.
///
/// Nginx (the only proxy in this deployment) sets `X-Forwarded-For` and
/// overwrites any client-supplied value, so the FIRST hop is the client. When
/// the header is absent (or unparsable) we fall back to the peer address from
/// the transport. This is the same convention used by `request_log` in
/// `src/routes/export.rs` (Zero-PII: the IP is only used for abuse
/// aggregation, never exposed to admins).
pub fn client_ip_from_headers(
    xff: Option<&str>,
    remote_addr: IpAddr,
) -> IpAddr {
    xff.and_then(|s| s.split(',').next())
        .map(str::trim)
        .and_then(|s| s.parse().ok())
        .unwrap_or(remote_addr)
}
```

When a request passes through a proxy (Nginx), the *server* sees the
proxy's IP — useless for rate limiting. The proxy adds
`X-Forwarded-For: <client>, <proxy1>, ...`, so the **first** entry is
the client. `s.split(',').next()` grabs it, `str::trim` cleans
whitespace, `.parse().ok()` tries to parse it as an `IpAddr` — and if
anything fails, `.unwrap_or(remote_addr)` falls back to the socket's
peer address. The chain is: try the header, fall back to the socket.
And the doc comment adds the *security reasoning*: it's safe to trust
the first hop because Nginx overwrites client-supplied values. (If the
deployment ever gains a second proxy, this comment is the first thing
to revisit.)

### 14.2 The token bucket: redis_bucket.rs

Now the muscle. `src/limiter/redis_bucket.rs` implements
`RedisBucketLimiter`, and its heart is a **token bucket** — the classic
rate-limiting algorithm. The idea: a bucket holds *tokens* (up to a
capacity — the burst). Every request takes a token. Tokens refill
continuously at a *flow* rate. Empty bucket = wait for refill. That's
it — and it gives you both a burst allowance *and* a sustained rate.

FicHub's twist: the bucket lives **inside Redis**, in a Lua script, so
the check is atomic and shared across all server instances. Here's the
script, in full, and it's only 28 lines:

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    local wait = (requested - new_tokens) / flow
    return wait
end
```

Lua is a tiny scripting language embedded in Redis, and scripts run
*atomically* — no other command interleaves while one runs. Let's read
it line by line, because it's the whole rate limiter in miniature:

- `KEYS[1]`, `ARGV[1..3]` — the script's inputs: the bucket's Redis
  key, how many tokens this request wants, the bucket's capacity, and
  its refill flow. (Called with `requested = 1.0` normally — and `1.5`
  when *penalizing* a failed request, as we'll see.)
- `HMGET key 'value' 'last_drain'` — each bucket is a Redis *hash*
  storing two numbers: the current token count and the last time it was
  touched ("drained").
- `redis.call('TIME')` — Redis's wall clock, returned as
  `(seconds, microseconds)`. The script uses **Redis's own clock**, not
  the application's — every server instance sees the same time, so
  buckets stay consistent across the fleet. (A subtle but critical
  detail: if each server used its own clock, a skewed server could
  corrupt the buckets.)
- `if value == nil` — first visit: the bucket doesn't exist yet, so
  start it *full* (`value = capacity`). New IPs get a full burst
  allowance — friendly to real users.
- The refill: `new_tokens = math.min(capacity, value + elapsed *
  flow)` — tokens accumulate as time passes, capped at capacity. Ten
  seconds idle at flow 60/hr adds a tiny fraction of a token; an hour
  idle refills the whole bucket.
- `allowed = new_tokens - requested` — do we have enough?
- If yes: write the new value back, return `-1` (the convention for
  "allowed"). If no: return the *wait time in seconds* —
  `(requested - new_tokens) / flow` — so the caller can send `429
  Retry-After`.

The whole algorithm in one atomic script. And because it's a script,
the check-and-update can't race: two requests can't both read "full"
and both pass. This is the *same* "one round trip beats three" lesson
from Chapter 13, applied at the Redis level.

Back in Rust, the script is loaded once at startup, and checks run by
SHA:

```rust
        // Load the token bucket Lua script
        let lua_script = r#"..."#;

        let mut conn = redis_conn.clone();
        let lua_sha: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(lua_script)
            .query_async(&mut conn)
            .await?;
```

`SCRIPT LOAD` compiles the script once and returns a SHA — from then
on, every check is `EVALSHA <sha> <key> <args>`, a single fast Redis
round trip. And the struct then stores the config from Chapter 10 —
remember `rl_download_capacity`, `rl_download_flow`, and friends? Here
they land:

```rust
        let (download_capacity, download_flow) = config
            .map(|c| (c.rl_download_capacity, c.rl_download_flow))
            .unwrap_or((DEFAULT_DOWNLOAD_CAPACITY, DEFAULT_DOWNLOAD_FLOW));
        ...
        let (shadowban_capacity, shadowban_flow) = config
            .map(|c| (c.rl_shadowban_capacity, c.rl_shadowban_flow))
            .unwrap_or((DEFAULT_SHADOWBAN_CAPACITY, DEFAULT_SHADOWBAN_FLOW));
        let shadowban_ttl = config
            .map(|c| c.rl_shadowban_ttl)
            .unwrap_or(DEFAULT_SHADOWBAN_TTL);
        let tiered_enabled = config.map(|c| c.rl_tiered_enabled).unwrap_or(true);
```

The Config flows straight into the limiter. The constants at the top of
the file are the *same* numbers as the config defaults — documented
together:

```rust
pub const DEFAULT_DOWNLOAD_CAPACITY: f64 = 10.0;
pub const DEFAULT_DOWNLOAD_FLOW: f64 = 60.0 / 3600.0; // 60/hour
pub const DEFAULT_AUTH_CAPACITY: f64 = 10.0;
pub const DEFAULT_AUTH_FLOW: f64 = 10.0 / 60.0; // 10/min
pub const DEFAULT_SEARCH_CAPACITY: f64 = 1000.0;
pub const DEFAULT_SEARCH_FLOW: f64 = 1000.0 / 60.0; // 1000/min
...
pub const SHADOWBAN_SET: &str = "fichub:shadowban";
```

Every constant carries its human-readable meaning in a comment —
`60.0 / 3600.0` is meaningless without "// 60/hour". And
`SHADOWBAN_SET: "fichub:shadowban"` is the Redis key of the shadowban
set — every Redis key in FicHub is namespaced like this
(`fichub:...`), so all the platform's keys live in one obvious family.

### 14.3 The check: two buckets per request

The `check_tiered` function is where it all comes together. The strategy:
check the per-IP bucket *and* the per-`(ip, client_id)` bucket, so a
shared-NAT house doesn't get punished as one person, while the IP still
has a ceiling:

```rust
        // Per-IP bucket first (the NAT ceiling). Identified clients get a
        // scaled ceiling so one client can't starve the IP.
        let ip_cap = if client_id.is_some() {
            self.nat_scaled_capacity(tier)
        } else {
            let (cap, _, _, _) = self.tier_params(tier);
            cap
        };
        ...
        let ip_key = format!("rate:tier:{:?}:ip:{}", tier, ip);
        let ip_wait = self
            .check_bucket(&ip_key, ip_cap, ip_flow)
            .await
            .unwrap_or(-1.0);
        if ip_wait > 0.0 {
            return TieredRateLimitResult::Wait(ip_wait.ceil() as u64);
        }

        // Per-(ip, client_id) bucket. The download tier uses the shadowban
        // params when the client is flagged; other tiers use the bonus params.
        if let Some(cid) = client_id {
            let client_key = format!("rate:tier:{:?}:client:{}:{}", tier, ip, cid);
            let (client_cap, client_flow) = if tier == Tier::Download && is_shadowbanned {
                (shadow_cap, shadow_flow)
            } else {
                (
                    ip_cap + self.client_bonus_capacity,
                    ip_flow + self.client_bonus_flow,
                )
            };
            let client_wait = self
                .check_bucket(&client_key, client_cap, client_flow)
                .await
                .unwrap_or(-1.0);
            if client_wait > 0.0 {
                return TieredRateLimitResult::Wait(client_wait.ceil() as u64);
            }
        }

        TieredRateLimitResult::Allowed
```

Notice the Redis **key design** — it's a naming convention that makes
each bucket unique and debuggable:

```
rate:tier:Download:ip:93.184.216.34
rate:tier:Download:client:93.184.216.34:abc-123
```

The tier's name, the scope (`ip` vs `client`), the IP, and the client
id are all baked into the key. You can `redis-cli KEYS 'rate:tier:*'`
and *see* every active bucket — which IPs are downloading, how full
their buckets are. That's the debugging superpower of descriptive keys.
(The `{:?}` in `format!` is the Debug format — `Tier::Download`
formats as `Download`.)

And the shadowban routing is one tidy conditional: a shadowbanned
client's download bucket uses the *shadowban* parameters — 5 tokens,
refilling 5/hour — instead of the normal bonus. The ban isn't a
block; it's a *throttle to near-uselessness*. "Friction, not a hard
block" — the comment from the trait, now implemented.

There's also a dev-mode escape hatch, right at the top of
`check_tiered`:

```rust
        if !self.tiered_enabled || !self.dynamic_rate_limit {
            // Test/dev mode: legacy static delay, always allowed.
            let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(delay)).await;
            return TieredRateLimitResult::Allowed;
        }
```

In development (`dynamic_rate_limit == false`, which we met in
Chapter 10), no buckets at all — just a tiny random delay (0.1–0.2s)
to simulate the shape of rate limiting without ever blocking the
developer. The integration tests never trip a real bucket; production
flips the switch and gets the real thing. Same code path, two modes,
one boolean.

### 14.4 The shadowban set

The shadowban itself is a Redis **set** — the `fichub:shadowban` set
of client_ids, with per-member expiry:

```rust
    /// Add a client_id to the shadowban set for `ttl_seconds` (24h default).
    /// The SET itself never expires — individual members do — so the set is
    /// cheap to probe and self-cleaning. NOTE: blocking, like
    /// `is_shadowbanned`.
    pub fn shadowban(&self, client_id: &str, ttl_seconds: u64) {
        let ttl = if ttl_seconds > 0 { ttl_seconds } else { self.shadowban_ttl };
        let _ = self.block_on_sadd(client_id, ttl);
    }
```

The trick: Redis sets can't have per-member TTLs (only whole keys can
expire). So FicHub adds the member *and* sets a TTL on a per-member
key:

```rust
    fn block_on_sadd(&self, client_id: &str, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let member_key = format!("{}:member:{}", SHADOWBAN_SET, client_id);
        let mut pipe = redis::pipe();
        pipe.cmd("SADD")
            .arg(SHADOWBAN_SET)
            .arg(client_id)
            .cmd("PEXPIRE")
            .arg(&member_key)
            .arg((ttl_seconds * 1000) as i64);
        let fut = pipe.query_async::<()>(&mut conn);
        let _: Result<(), redis::RedisError> = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(fut)
        });
        Ok(())
    }
```

`redis::pipe()` batches two commands — `SADD` the member, `PEXPIRE` a
per-member key — in one round trip. The comment on the trait explained
the rest: *"Lazy per-member expiry: a member whose per-member TTL key
is gone is pruned on the next probe, keeping SISMEMBER fast and the set
bounded."* A shadowban expires after 24 hours automatically; the admin
can `unshadowban` early. And it's **friction, not a hard block** — the
shadowbanned client is still served, just at 5 downloads/hour (and, as
we'll see, with a puzzle to solve).

One more honest detail in this code: the `block_in_place` wrappers.
The trait methods are *synchronous* (`fn shadowban(...)`, not
`async fn`), but Redis is async — so the implementation blocks the
current thread briefly to run the quick round trip, and says so in the
comments: *"they are quick single-roundtrip SET ops."* Blocking is
usually a sin in async code; here it's a documented, bounded exception.
When you see `block_in_place` in a codebase, read the comment — it
should explain why the blocking is acceptable.

### 14.5 The penalty: report_failure

The old-style limiter also has a memory: when a request *fails* (bad
auth, a 4xx from a bot), the IP is penalized:

```rust
    async fn report_failure(&self, ip: IpAddr) {
        let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
        let ip_key = format!("rate:ip:{}", ip);
        let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
    }
```

And `penalize` is just `check_bucket` with a bigger request:

```rust
    /// Penalize by requesting extra tokens (on failure)
    async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.clone();
        let _: f64 = redis::cmd("EVALSHA")
            .arg(&self.lua_sha[..])
            .arg(1)
            .arg(key)
            .arg(1.5)        // penalize with 1.5 tokens
            .arg(capacity)
            .arg(flow)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
```

Beautiful: the *same* Lua script, asked for 1.5 tokens instead of 1.
No second code path — failing behavior is just "spend extra tokens."
The script already computes wait times; penalizing is free. That's the
kind of design where one algorithm does all the work.

### 14.6 Proof of work: make the bot pay

The last defense is the most elegant. When a client is shadowbanned
(or otherwise flagged), the export endpoint demands **proof of work**:
solve a hash puzzle before serving the file. The pure math lives in
`src/services/pow.rs`, and it's the hashcash construction — the same
idea Bitcoin uses (and that was invented for *spam* in 1997):

```rust
//! Hashcash-style proof-of-work (PoW) challenges for flagged clients.
//!
//! When a client is shadowbanned/flagged (see `src/limiter/`), the export
//! endpoint refuses to serve the request until the client proves it spent
//! real CPU work. The challenge is a random hex string; the client must find
//! a `nonce` such that `SHA-256(challenge || nonce)` (hex) starts with
//! `difficulty` zero bits (difficulty 16 → the first 4 hex chars are `0000`).
//!
//! This is friction, not a hard block: humans (or a tiny bit of browser JS)
//! never notice a ~65k-hash solve, while bulk bots pay ~1000x per request.
```

Read that header twice. It explains the *whole feature*: what a
challenge is, what the client must do, why it's fair (humans never
notice ~65,000 hashes; bots pay ~1000× per request). The challenge
itself:

```rust
/// Generate a fresh challenge: 16 random bytes hex-encoded (32 chars).
pub fn generate_challenge_with(difficulty: u32, ttl_secs: u64, mut rng: Rng) -> PowChallenge {
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    let challenge = hex::encode(bytes);
    let expires_at = chrono::Utc::now().timestamp() + ttl_secs as i64;
    PowChallenge {
        challenge,
        difficulty,
        expires_at,
    }
}
```

16 random bytes from the OS RNG, hex-encoded. And the verification —
the part the *client* has to work for:

```rust
/// Verify a `(challenge, nonce)` pair: the SHA-256 hex of
/// `challenge || nonce` must start with `difficulty` zero bits.
pub fn verify_solution(challenge: &str, nonce: &str, difficulty: u32) -> bool {
    if difficulty == 0 {
        // Zero-bit difficulty is trivially satisfiable by any nonce.
        return nonce.chars().all(|c| c.is_ascii_digit());
    }
    if !nonce.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(challenge.as_bytes());
    hasher.update(nonce.as_bytes());
    let digest = hasher.finalize();
    let hex_digest = hex::encode(digest);
    hex_digest.starts_with(&difficulty_to_hex_prefix(difficulty))
}
```

The nonce is a decimal counter; the client tries `nonce = 0`, `1`, `2`,
... hashing `challenge || nonce` each time until the hex digest starts
with enough zeros. With difficulty 16, that's a 1-in-65,536 chance per
hash — so about 65,000 tries, a fraction of a second on any computer,
but *multiplied by every request* it's ruinous for a bot. And the
difficulty math is spelled out honestly:

```rust
/// The minimum hex prefix a valid solution hash must start with.
///
/// `difficulty` counts leading zero BITS; each hex char is 4 bits, so the
/// required prefix is `ceil(difficulty / 4)` zero chars. Difficulty 16 →
/// `"0000"`. A non-multiple-of-4 difficulty (e.g. 17) still requires
/// `ceil(17/4) = 5` zero hex chars, which is slightly *harder* than 17 bits
/// — an acceptable granularity for a friction layer.
pub fn difficulty_to_hex_prefix(difficulty: u32) -> String {
    let nibbles = difficulty.div_ceil(4);
    "0".repeat(nibbles as usize)
}
```

Now — who stores the solves? Redis, of course. Once a client solves a
challenge, the solve is cached so follow-up exports don't re-puzzle:

```rust
/// Store a solved challenge in Redis with a TTL (fail-open: errors are
/// logged, the solve is treated as accepted by the caller).
pub async fn store_solution(
    redis: &mut redis::aio::MultiplexedConnection,
    challenge: &str,
    ttl_secs: u64,
) -> Result<(), redis::RedisError> {
    let key = format!("{SOLVED_KEY_PREFIX}{challenge}");
    redis::cmd("SETEX")
        .arg(key)
        .arg(ttl_secs)
        .arg("1")
        .query_async::<()>(redis)
        .await?;
    Ok(())
}
```

`SETEX key ttl value` — set a key with a TTL in one atomic command.
The key is `fichub:pow:solved:<challenge>`; it lives 10 minutes
(remember `pow_ttl_secs` from Chapter 10? there it is). And the check
side is deliberately fail-open:

```rust
/// True when `challenge` has a stored, unexpired solve in Redis
/// (fail-open: any Redis error returns `false`, so a Redis hiccup never
/// un-blocks a flagged client — it just re-issues the challenge).
pub async fn solution_solved(
    redis: &mut redis::aio::MultiplexedConnection,
    challenge: &str,
) -> bool {
    let key = format!("{SOLVED_KEY_PREFIX}{challenge}");
    let exists: Option<String> = redis::cmd("GET")
        .arg(key)
        .query_async(redis)
        .await
        .unwrap_or(None);
    exists.is_some()
}
```

If Redis errors, the client gets re-challenged — annoying but safe. The
fail-open direction is chosen deliberately: *never* un-block a flagged
client because of an infrastructure hiccup. When you write
Redis-backed security code, decide your failure direction on purpose,
and write it in the comment.

💡 **Key Concept — Proof of work is friction, not identity.** The goal
isn't to *identify* humans — it's to make automation *uneconomical*.
A human solves one puzzle in milliseconds and downloads all day; a bot
scraping thousands of fics pays thousands of puzzles, and the marginal
cost of each scrape explodes. Rate limits throttle *volume*; PoW
throttles *velocity per action*. Together they make bulk scraping cost
more than it's worth. And the beauty of `difficulty` as a config knob
(Chapter 10!) is that the platform can turn the screws without a
redeploy: `POW_DIFFICULTY=20` and suddenly bots pay 1M hashes per
download.

### 14.7 The limiter and the rest of the platform

Step back and look at the whole anti-bot stack, from config to Redis:

1. **Config** (Chapter 10) declares every dial: `rl_download_capacity`,
   `pow_difficulty`, `rl_shadowban_ttl`, ...
2. **`limiter/mod.rs`** defines the contract: `Tier`, traits, and the
   IP-extraction helper.
3. **`limiter/redis_bucket.rs`** implements it: the Lua token bucket,
   per-IP + per-client keys, the shadowban set, dev-mode delay.
4. **`services/pow.rs`** adds the puzzle layer for flagged clients.
5. **The routes** (Part 4) call `limiter.check(...)` on the way in and
   `pow` on the way to the export.

And every layer is *tunable without a code change* — the config knobs
from Chapter 10 feed the limiter at startup. That's the architecture
paying off: the same `Config` struct that tamed 800 lines of env vars
is what makes the whole anti-bot system adjustable.

🧪 **Try It Yourself — watch a bucket fill and drain.** With the stack
running, open `redis-cli` and watch the limiter's keys appear as you
make requests:

```bash
redis-cli KEYS 'rate:*'
redis-cli HGETALL rate:tier:Download:ip:<your-ip>   # if you downloaded an epub
```

Each `HGETALL` shows `value` (tokens left) and `last_drain` — the
bucket's state. Hammer the download endpoint a few times and watch
`value` fall; wait a minute and watch it creep back up. Then check the
shadowban machinery:

```bash
redis-cli SISMEMBER fichub:shadowban <some-client-id>
```

And if you want to see the PoW in action, set `POW_DIFFICULTY=4`
(cheap — the config comment says "4 bits for cheap tests"),
shadowban your own client id, and request an export: the server will
hand you a challenge instead of a file. Solve it with a tiny script
(loop nonces, hash `challenge || nonce`, check for `0000`) and watch
the export go through. You've now *felt* the entire chapter.

⚠️ **Watch Out — rate limiting is only as good as its keys.** If two
requests can't be told apart, the limiter can't limit them fairly. The
per-IP bucket trusts `client_ip_from_headers` — which trusts Nginx to
overwrite `X-Forwarded-For` (see the comment in mod.rs). If your
deployment ever lets clients *set* that header (a misconfigured proxy,
or direct exposure), attackers can forge IPs and empty any bucket they
want. And if you ever run multiple server instances, the buckets *must*
live in shared Redis — a per-process in-memory limiter would let each
instance spend its own allowance. FicHub's design (shared Redis,
script-atomic buckets) is exactly the production shape; replicate it,
don't improvise around it.

### 14.8 What Part 3 taught you

Take a breath. You've now read the entire data and defense layer of
FicHub — from the first env var to the last Redis key. Let's stack the
whole part:

- **Config is a contract.** One struct, ~150 typed fields, one
  parser, defaults everywhere, `expect` only where survival depends on
  it, and tests that lock every default and every failure mode.
- **Postgres is the memory.** The pool (20 connections, 10-second
  acquire timeout) is the front door; migrations (34 files) are the
  versioned autobiography of the schema; models (`FromRow`) are the
  shapes; queries (2,700 lines, one skeleton) are the operations —
  upserts for atomic writes, `RETURNING` for round-trip-free keys,
  placeholders against injection.
- **Redis is the reflex.** Token buckets in Lua (atomic, shared,
  tunable), per-IP + per-client keys for fairness, a self-cleaning
  shadowban set for friction, and hashcash PoW to make bots pay.

And the meta-lesson, the one that ties all five chapters together:
**every layer of FicHub is configurable, typed, tested, and
documented — and the documentation lives where the code lives.**
Comments explain *intent* ("friction, not a hard block"), tests lock
*behavior* (every default, every panic, every fallback), and the type
system makes illegal states unrepresentable (`Option` for nullable,
enums for modes, `FromRow` for rows).

This is the foundation everything else stands on. In Part 4, the
handlers — the code that actually *receives* HTTP requests — will lean
on all of it: `AppState` (which holds the pool, the config, and the
limiter), the queries, and the rate-limit checks, every single request.

One last thing before we go. The config's `agent_enabled`, the
migrations' `agent_runs` table, the `rec_*` tables and the
`REC_ENGINE_MODE` switch — you've now seen the *schema* of FicHub's
future. But you haven't seen the *roads* yet: the routes. Every URL,
every handler, every JSON response. That's the next part — where the
platform finally starts talking.

See you in Part 4. The data's in place — now we make it answer.

---
