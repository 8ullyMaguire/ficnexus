## Part 3 — Configuration & Data Layer

### Chapter 9: config.rs — Every Knob in One Place

`src/config.rs` is the file to read when you ask "how do I turn on X?". It
parses environment variables into one `Config` struct in `from_env()`.

```rust
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub body_cache_dir: PathBuf,        // default /public/literature/fichub/bodies
    pub rec_engine_mode: RecEngineMode, // legacy | pluggable
    pub rec_shadow_mode: bool,
    pub agent_enabled: bool,            // self-healing agent autonomy
    // ... dozens more
}
```

The patterns you'll see throughout:

- **Default then override.** `Config::from_env()` reads `std::env::var`,
  and when a var is missing, falls back to a sane default. Example:
  `BODY_CACHE_DIR` defaults to `/public/literature/fichub/bodies`.
- **Typed knobs.** Enums like `RecEngineMode` are parsed from strings with a
  `FromStr` impl. Booleans are parsed with a helper that accepts
  `true/false/1/0`.
- **One struct, shared everywhere.** Once built, `Config` is frozen inside
  `AppState` and handlers read `state.config.x`. No handler re-reads env
  vars.

Why centralize? Because configuration is *behavior*. If you want to know
what the server will do in production, you read `Config`. If you want to
test a different behavior, you construct a `Config` with the values you
want — no env mutation needed (well, almost; the tests use an `ENV_LOCK`
mutex for the rare cases that do touch env).

The test pattern in `config.rs` is worth stealing for your own features:

```rust
// ENV_LOCK guards env mutation so parallel tests don't stomp each other
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
```

> 🧪 **Try it:** read `from_env()` and list five knobs you didn't know
> existed. Then grep the codebase for one of them to see where it's read.
>
> ⚠️ **Watch out:** don't add a new `std::env::var(...)` call in a handler.
> That's the "rogue env read" anti-pattern — it bypasses `Config` and makes
> testing harder. Add the field to `Config` and thread it through
> `AppState`.
>
> 💡 **Key concept:** configuration is behavior; centralize it. `Config` is
> the contract between the deployment environment and the running server.

### Chapter 10: PostgreSQL + SQLx

Open `src/db/mod.rs`. It builds the pool and runs migrations on boot:

```rust
let pool = PgPoolOptions::new()
    .max_connections(...)
    .connect(&config.database_url).await?;

sqlx::migrate!().run(&pool).await?;   // applies migrations/1..N
```

Two ideas matter here.

**The pool.** `PgPool` manages a set of live connections. Handlers call
`state.db.clone()` (cheap) or `&state.db` and the pool hands out a
connection from its set. This is why the server can handle many concurrent
requests without opening a connection per request. Pool sizing is a real
tuning knob: too few connections and you serialize on the DB; too many and
you exhaust Postgres's `max_connections`.

**Migrations at boot.** `sqlx::migrate!()` embeds `migrations/*.sql` into
the binary at compile time. On startup, it compares the embedded list
against the `_sqlx_migrations` table in the database and applies anything
new, in order. This is why deploys "just work": the new binary carries its
own schema changes. Never run migrations by hand in production — the binary
does it, transactionally per file.

`src/db/models.rs` defines the `FromRow` structs that match table shapes:

```rust
#[derive(FromRow)]
pub struct FicInfo {
    pub id: String,          // url_id
    pub site_domain: String,
    pub title: String,
    pub author: String,
    // ...
}
```

One gotcha to internalize: this repo uses the **runtime query forms**
(`sqlx::query()`, `query_as()`, `query_scalar()`) heavily rather than the
compile-time-checked macros (`query!`, `query_as!`). The trade-off:
flexibility (dynamic SQL, fewer macro headaches) for less compile-time
safety. Column names in the SQL must match the `FromRow` struct fields at
runtime, or you get a runtime decode error. **When you change a table, keep
the model and the migration in lockstep.**

> 🧪 **Try it:** connect to the dev database and inspect the
> `_sqlx_migrations` table. You should see 34 rows, in order.
> `psql -h localhost -U fichub -d fichub -c "SELECT version, description FROM _sqlx_migrations ORDER BY version;"`
>
> ⚠️ **Watch out:** on this setup, Postgres tables are owned by the `fichub`
> role — if you create tables in a scratch DB with a different owner, you'll
> hit permission errors. Use `ALTER OWNER TO fichub` after manual DDL.
>
> 💡 **Key concept:** pool + embedded migrations = deployable schema. The
> binary is the source of truth for schema changes.

### Chapter 11: The Migrations — 34 Files, One Schema Story

`migrations/` is numbered `001_*.sql` … `034_*.sql`. Together they tell the
entire history of the product's data model. Reading them in order is like
reading the product's diary. The story they tell:

- **001–004:** the core — `fic_info` (sources), `works`, tags, users,
  bookmarks, and the generated tsvector column + trigger for search.
- **005–010:** search analytics, ratings/reviews, comments, and the
  `search_queries` table.
- **011:** roadmap consensus — `feature_clusters` with the pgvector
  embedding column (768 dims, HNSW index).
- **013:** user roles/reputation.
- **016–018:** roadmap seeding + fic requests (the prompt board).
- **020:** series & authors.
- **023–025:** translations, rating/score fixes, extra metadata.
- **029–030:** self-healing — `scrape_failures` + `agent_runs`.
- **033:** usage analytics — `usage_events`.
- **034:** modlog.

Let's look at two representative patterns.

**Upsert-style DDL.** Many migrations create tables designed for idempotent
writes: unique constraints plus `ON CONFLICT` handling in the queries
(Chapter 12). For example, fic metadata is upserted so re-scraping a URL
updates rather than duplicates.

**Generated columns + triggers.** Migration 001 creates a tsvector column
for full-text search:

```sql
ALTER TABLE fic_info ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (... ) STORED;
CREATE INDEX ... USING GIN (search_vector);
```

The DB maintains the search index as rows change — no application-side
sync.

**Rules for working with migrations:**

1. **Never edit an applied migration.** Add `035_...` instead. The
   `_sqlx_migrations` table records a checksum of each applied file; editing
   an applied one causes a checksum mismatch error on the next boot.
2. **One logical change per migration.** Numbering is sequential; if two
   people add migrations concurrently, the numbers conflict — coordinate.
3. **Down migrations don't exist here.** The project doesn't roll back; it
   moves forward. If a migration is wrong, add a corrective one.

> 🧪 **Try it:** `ls migrations/ | head -20` then `ls migrations/ | tail -6`.
> You'll see the progression from core tables to analytics and modlog.
>
> ⚠️ **Watch out:** the `search_vector` column and its trigger are
> *generated* — you can't insert into them directly. If a query fails with
> "column ... is generated," that's why.
>
> 💡 **Key concept:** migrations are the schema's version history, applied
> transactionally at boot, never edited after the fact.

### Chapter 12: queries.rs — The Big Query Module

`src/db/queries.rs` is the workhorse: roughly 2,700 lines of SQL functions.
The patterns matter more than any individual query, so let's study the
three you'll see everywhere.

**Pattern 1 — Upsert:**

```rust
sqlx::query(
    "INSERT INTO fic_info (id, site_domain, title, author, ...)
     VALUES ($1, $2, $3, ...)
     ON CONFLICT (id) DO UPDATE SET
       title = EXCLUDED.title, author = EXCLUDED.author, ..."
)
```

Scrapes are idempotent: running the same URL twice updates, never
duplicates. This is the foundation of the "cache of all gathered
fanfiction" idea.

**Pattern 2 — RETURNING:**

```rust
let id: i32 = sqlx::query_scalar(
    "INSERT INTO works (...) VALUES (...) RETURNING id"
).fetch_one(&pool).await?;
```

Insert and get the id in one round-trip. Saves a `SELECT` and a race.

**Pattern 3 — inet binding:**

```rust
.bind(ip_string)   // SQL says: ... WHERE client_ip = $N::inet
```

Postgres `inet` columns are bound as strings with an explicit `::inet` cast
in the SQL. Don't try to bind an IP as an integer or a CIDR type.

Three lessons for working here:

1. **Grep before you write.** The module is huge. The function you need may
   already exist — `get_fic_info`, `upsert_fic_info`, `insert_request_log`,
   `lookup_alias`, and dozens more. Search first.
2. **Add near siblings.** When you add a query, put it next to related ones
   and follow the naming convention (`<verb>_<noun>`).
3. **Parameterize everything.** No string interpolation into SQL. `$1, $2,
   ...` binds prevent SQL injection and keep types explicit.

> 🧪 **Try it:** find `upsert_fic_info` and trace what happens when you
> export a fic twice. The second export hits the body cache, but the
> metadata upsert still runs — safely updating.
>
> ⚠️ **Watch out:** runtime `query()` means typos surface at runtime, not
> compile time. Always exercise new queries with a real test (DB-gated or
> via an endpoint) before declaring done.
>
> 💡 **Key concept:** one query module keeps SQL discoverable and
> consistent. Upserts + RETURNING + parameterized binds are the house
> style.

### Chapter 12A: A Walk Through the Key Queries

Let's read a few real functions from `queries.rs` so the patterns click.

**`get_fic_info`** — the metadata lookup. It fetches one source row by
url_id:

```rust
pub async fn get_fic_info(pool: &PgPool, url_id: &str)
    -> Result<Option<FicInfo>, sqlx::Error>
{
    sqlx::query_as::<_, FicInfo>(
        "SELECT id, site_domain, title, author, description, words,
                status, updated_at, ...
         FROM fic_info WHERE id = $1"
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await
}
```

Note `fetch_optional` — it returns `Option`, not a row, so a missing fic is
`None`, not an error. That's the house pattern for "might not exist."

**`insert_request_log`** — writes a request/telemetry row. Notice the
`::inet` cast we mentioned:

```rust
sqlx::query(
    "INSERT INTO request_log (url_id, ip, client_id, etype, ...)
     VALUES ($1, $2::inet, $3, $4, ...)"
)
.bind(url_id)
.bind(ip_string)   // string → ::inet in SQL
.bind(client_id)
// ...
```

**`upsert_fic_info`** — the idempotent write. `ON CONFLICT (id) DO UPDATE
SET ...` with `EXCLUDED` referencing the would-be-inserted row. This is
why re-scraping never creates duplicates.

**`lookup_alias`** — tag alias lookup. Small, but shows the join style
used across the tag system.

The takeaways: `fetch_optional` for maybe-rows, `::inet` for IPs, `ON
CONFLICT` for idempotence, and every query parameterized with `$N`.

> 🧪 **Try it:** open `queries.rs` and find `get_fic_info`, then
> `upsert_fic_info`. Read them side by side — the select and the upsert
> of the same table. That pair is the backbone of the archive.
>
> ⚠️ **Watch out:** `fetch_optional` vs `fetch_one` — use `fetch_one` only
> when the row MUST exist, and handle `RowNotFound` via `?` (it converts
> to `AppError::NotFound` through the `From` impl).
>
> 💡 **Key concept:** the query module's style is: parameterized,
> idempotent, optional-aware. Match it and your queries will fit right in.

### Chapter 12B: The Model Types — FicInfo, WorkRow & Friends

`src/db/models.rs` defines the `FromRow` structs that map rows to Rust
types. The central ones:

```rust
#[derive(FromRow, Serialize)]
pub struct FicInfo {
    pub id: String,             // url_id
    pub site_domain: String,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub words: i64,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
    // ...
}
```

Notice `description: Option<String>` — nullable columns become `Option`.
That's a SQLx `FromRow` rule: if the column can be NULL, the field must be
`Option`. Get this wrong and you get a runtime decode error.

The **works model** is separate from fic_info because of the unified works
architecture: a `work` is the abstract story, and each `fic_info` row is a
*source* (URL) for it. The same story on AO3 and FFN is one work with two
sources. Fields like `work_id` on fic_info link them.

> 🧪 **Try it:** find `WorkRow` and compare it with `FicInfo`. Identify
> which fields are per-source vs per-work.
>
> ⚠️ **Watch out:** nullable columns MUST be `Option<T>` in the struct.
> When you add a nullable column, update the model in the same change.
>
> 💡 **Key concept:** models mirror the schema; `Option` marks NULLable;
> work vs fic_info is the abstraction boundary.

### Chapter 12C: A Tour of the API Areas

Now that you know the data layer, here's the tour of `src/routes/` you'll
reference constantly:

- **`auth.rs`** — register/login, JWT issue/verify, `AuthUser`.
- **`search.rs`** — the search handler (parser → SQL) + Ask the Archive.
- **`meta.rs`** — `/api/meta`: scrape metadata for a URL.
- **`export.rs`** — the export pipeline (URL → EPUB/HTML/...).
- **`download.rs` / `kindle.rs` / `updates.rs`** — download flows, Kindle
  email, updates feed.
- **`reader.rs`** — chapter content for the reader.
- **`social.rs` / `comments.rs` / `reviews.rs` / `follows.rs`** — the
  social layer.
- **`requests.rs`** — the fic-request board (create/answer/vote/accept/
  candidates).
- **`lists.rs` / `shelves.rs` / `series.rs` / `authors.rs`** — reading
  lists, shelves, series & author pages.
- **`roadmap.rs`** — the consensus arena.
- **`admin.rs`** — admin endpoints (users, bans, stats, bots, analytics).
- **`modlog.rs`** — the public moderation log.
- **`analytics.rs`** — usage analytics.
- **`curator_content.rs`** — peer-voted body fixes.
- **`auto_tag.rs` / `locales.rs`** — AI features (auto-tagger,
  translations).
- **`pow.rs` / `honeypot.rs`** — anti-bot.
- **`health.rs`** — liveness.
- **`notifications.rs` / `badges.rs` / `quests.rs` / `trending.rs`** —
  engagement features.

Each file follows the same shape: `pub async fn` handlers, `AppResult<T>`
returns, `State<Arc<AppState>>` extractor, and (for admin/curator actions)
a modlog record.

> 🧪 **Try it:** pick the feature you care most about and open its route
> file. Read one handler end to end: extractors → logic → response.
>
> ⚠️ **Watch out:** some handlers are long. Read the return type first,
> then the extractors, then skim the logic. You don't need every line to
> understand the flow.
>
> 💡 **Key concept:** one file per API area, one handler per endpoint,
> one pattern per handler. The tour above is your index.

---
