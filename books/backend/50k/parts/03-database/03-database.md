# Part 3: Storing Data — PostgreSQL, SQLx, and Real Queries

*Chapters 8–11*

In Part 2, we built our `AppState` — a shared toolbox that every handler can reach into. One of the items in that toolbox was `db`, our PostgreSQL connection pool. Now it's time to put it to work. We're going to learn how to store fanfiction metadata, track every request that hits our server, and query that data back out when we need it.

This is where FicHub stops being a toy and starts being a real application.

---

## Chapter 8: PostgreSQL Basics

### What Is a Database?

Imagine you have a giant filing cabinet. Inside it, you have folders — each folder is like a *table*. Inside each folder, you have individual papers — each paper is like a *row*. And each paper has fields filled in at the top: Title, Author, Date — those are *columns*.

A database is exactly that: a structured collection of information organized into tables.

Here's how FicHub's main table looks when we think of it as a spreadsheet:

| id | title | author | chapters | words | status |
|----|-------|--------|----------|-------|--------|
| ao3_123456 | The Best Story Ever | JaneDoe | 12 | 85000 | complete |
| ffnet_789012 | Another Great Fic | JohnWriter | 5 | 32000 | incomplete |

Each row is one fanfiction. Each column is a property of that fanfiction. Together, they let us ask questions like "How many stories has JaneDoe written?" or "Show me all completed fics with more than 50,000 words."

### Installing PostgreSQL

On Arch Linux (what we're using), install PostgreSQL and start it:

```bash
sudo pacman -S postgresql
sudo systemctl enable --now postgresql
```

On Ubuntu/Debian:

```bash
sudo apt install postgresql
sudo systemctl enable --now postgresql
```

On macOS with Homebrew:

```bash
brew install postgresql@16
brew services start postgresql@16
```

After installation, verify it's running:

```bash
pg_isready
# Output: localhost:5432 - accepting connections
```

If you see "accepting connections," you're good to go!

### Creating a Database

PostgreSQL comes with a handy tool called `createdb`. Let's make a database for FicHub:

```bash
sudo -u postgres createdb fichub
```

And to connect to it from the command line, use `psql`:

```bash
sudo -u postgres psql fichub
```

You'll see a prompt that looks like `fichub=#`. That's the SQL shell — you can type commands right in here! Try it:

```sql
fichub=# SELECT 2 + 2;
 4

fichub=# SELECT now();
              now
-------------------------------
 2024-03-15 10:30:00.123456+00
```

When you're done, type `\q` to quit.

### Creating a Table

Let's create our first table to store fic metadata:

```sql
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    source TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

Let's break that down piece by piece:

- `CREATE TABLE fic_info` — Hey database, make a new table called `fic_info`.
- `id VARCHAR(128) PRIMARY KEY` — Each row needs a unique ID, up to 128 characters. The `PRIMARY KEY` part means "this column uniquely identifies each row."
- `TEXT NOT NULL` — A text field that can't be empty.
- `INT4` — A 32-bit integer (good for chapter counts — no fic has 2 billion chapters).
- `INT8` — A 64-bit integer (good for word counts — some fics are *huge*).
- `TIMESTAMPTZ` — A timestamp with timezone information. Always use this instead of `TIMESTAMP` — timezone bugs are the worst.
- `DEFAULT CURRENT_TIMESTAMP` — If you don't provide a value, use the current time automatically.

### The Five Basic Operations

SQL has five fundamental operations. Think of them as the "CRUD" family, plus one more:

**INSERT — Add a row:**

```sql
INSERT INTO fic_info (id, title, author, chapters, words, description, status, source)
VALUES ('ao3_123456', 'The Best Story Ever', 'JaneDoe', 12, 85000, 'A great story', 'complete', 'https://archiveofourown.org/works/123456');
```

**SELECT — Read rows back:**

```sql
SELECT * FROM fic_info WHERE id = 'ao3_123456';
SELECT title, author FROM fic_info WHERE chapters > 10;
SELECT * FROM fic_info WHERE author ILIKE '%jane%';
SELECT COUNT(*) FROM fic_info WHERE status = 'complete';
```

**UPDATE — Modify a row:**

```sql
UPDATE fic_info SET chapters = 13, updated = NOW() WHERE id = 'ao3_123456';
```

**DELETE — Remove a row:**

```sql
DELETE FROM fic_info WHERE id = 'ao3_123456';
```

**UPSERT — Insert or update (this one is special):**

```sql
INSERT INTO fic_info (id, title, author, chapters, words, description, status, source)
VALUES ('ao3_123456', 'Updated Title', 'JaneDoe', 15, 92000, 'Updated description', 'complete', 'https://archiveofourown.org/works/123456')
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    chapters = EXCLUDED.chapters,
    words = EXCLUDED.words,
    updated = NOW();
```

UPSERT is what FicHub uses most. When someone requests a fic we already have, we don't want to fail — we want to *update* the existing record with fresh data. The `ON CONFLICT` clause handles this beautifully. If the `id` doesn't exist yet, it inserts. If it already exists, it updates the specified columns with the new values.

### Data Types You'll Use Most

| Type | What it stores | Example |
|------|---------------|---------|
| `TEXT` | Any text of any length | "Harry Potter and the..." |
| `VARCHAR(128)` | Text with a max length | "ao3_123456" |
| `INT4` | 32-bit integers (-2B to +2B) | 42, 12, 50000 |
| `INT8` | 64-bit integers (huge numbers) | 1000000000 |
| `BOOLEAN` | true or false | true, false |
| `TIMESTAMPTZ` | Date + time + timezone | 2024-01-15 10:30:00+00 |
| `UUID` | Universally unique identifiers | 550e8400-e29b-41d4... |

> 💡 **Key Concept: TEXT vs VARCHAR**
>
> In PostgreSQL, `TEXT` and `VARCHAR` are practically identical in performance. The only difference is that `VARCHAR(128)` lets the database reject values longer than 128 characters. Use `VARCHAR` when you have a known maximum length (like IDs), and `TEXT` when the length could be anything (like descriptions). FicHub uses `VARCHAR(128)` for the `id` column because fic IDs from AO3 or FF.net have a predictable format.

### Why PostgreSQL?

There are lots of databases out there — MySQL, SQLite, MongoDB, Redis. Why did we pick PostgreSQL?

**ACID Compliance.** ACID stands for Atomicity, Consistency, Isolation, Durability. It means that when you save data, it either all saves or none of it saves — no half-saves. And if the server crashes right after a save, the data is still there when it comes back.

**JSON Support.** PostgreSQL can store JSON directly in a column. This is amazing when scraped metadata has fields you didn't predict. A fic's author page might have extra fields that aren't in our schema — we just stuff them into a `jsonb` column and query them later.

**Full-Text Search.** PostgreSQL can search through text like a mini search engine. We use this for the "did you mean?" suggestions. No need for Elasticsearch or a separate search service.

**It's free and battle-tested.** Almost every major website uses PostgreSQL. It's been around since 1996 and just keeps getting better. Instagram, Spotify, and the US federal government all run on PostgreSQL.

> 💡 **Key Concept: ACID Transactions**
>
> Imagine you're transferring money between two bank accounts. You need to subtract from one AND add to the other. If the power goes out halfway through, ACID ensures you don't end up with money that vanished into thin air — either both operations complete, or neither does. PostgreSQL handles this automatically.
>
> In FicHub terms: when we log a request, we need to insert into *both* `request_source` and `request_log`. If the server crashes after one but before the other, PostgreSQL guarantees we don't end up with an orphaned record.

### 🧪 Try It Yourself

1. Install PostgreSQL and create a `fichub` database
2. Create the `fic_info` table with the schema above
3. Insert a row for your favorite fanfiction
4. SELECT it back — did it appear?
5. UPDATE the word count to something different
6. Verify the update: `SELECT words FROM fic_info WHERE id = 'your_id';`
7. DELETE the row
8. Drop the table: `DROP TABLE fic_info;`

This gives you hands-on feel for how SQL works before we automate it in Rust.

---

## Chapter 9: SQLx and Migrations

### What Is SQLx?

SQLx is a Rust library for talking to PostgreSQL (and other databases). The magic trick? It checks your SQL queries at *compile time*. If you write a bad query — wrong column name, wrong table, wrong number of parameters — your code won't compile. The compiler catches your SQL bugs before your users find them.

That's a huge deal. Most database bugs are "silent" — the code runs fine, but the query returns the wrong data or crashes at 3 AM on a Saturday. SQLx eliminates that entire category of bugs.

There's one catch: to get compile-time checking, SQLx needs a live database connection when you run `cargo build`. It actually executes your queries against a test database to verify they're correct. For CI/CD, you need a database available during the build step.

### Adding SQLx to Cargo.toml

In FicHub's `Cargo.toml`, we have:

```toml
[dependencies]
sqlx = { version = "0.9", default-features = false, features = [
    "runtime-tokio",
    "postgres",
    "chrono",
    "uuid",
    "migrate",
    "tls-rustls-ring",
    "derive",
    "macros"
] }
```

Let's decode those features:

- **`runtime-tokio`** — Use Tokio as our async runtime (the engine that runs async code).
- **`postgres`** — Connect to PostgreSQL.
- **`chrono`** — Handle dates and times with the `chrono` library.
- **`uuid`** — Handle UUID values.
- **`migrate`** — Run SQL migrations automatically.
- **`tls-rustls-ring`** — Encrypted database connections (important for production!).
- **`derive`** — The `#[derive(FromRow)]` macro (turns a query row into a Rust struct).
- **`macros`** — Compile-time query checking macros.

### What Are Migrations?

A migration is a versioned SQL file that makes changes to your database schema. Think of it like a changelog for your database.

We don't write SQL by hand every time we start the server. Instead, we write migration files once, and SQLx runs them in order. If we add a new column later, we write a new migration file — `002_add_new_column.sql` — and SQLx knows to run it after the first one.

FicHub's migrations live in a `migrations/` directory:

```
fichub/
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_recommender.sql
│   ├── 003_tagging.sql
│   └── 004_shelves.sql
├── src/
└── Cargo.toml
```

Each migration is just a `.sql` file with a numbered prefix. SQLx reads them in order, checks which ones have already been applied (it keeps track in a special `_sqlx_migrations` table), and runs only the new ones.

### Creating Our First Migration

Let's write `001_initial_schema.sql`. Here's the real FicHub migration, simplified to the essentials:

```sql
-- Request source tracking
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);

-- Request log
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

-- Fic metadata cache
CREATE TABLE IF NOT EXISTS fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
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
    content_hash VARCHAR(256)
);

-- Export log (cache tracking)
CREATE TABLE IF NOT EXISTS export_log (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    version INT NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, version, etype, input_hash)
);
```

Notice a few important things:

- **`CREATE TABLE IF NOT EXISTS`** — Safe to run multiple times. If the table already exists, it doesn't complain. This is crucial for migrations — you might run them on a database that's already partially set up.
- **`BIGSERIAL PRIMARY KEY`** — PostgreSQL auto-generates an incrementing number for each row. The first row gets `1`, the next gets `2`, and so on. It's like a ticket dispenser.
- **`REFERENCES fic_info(id)`** — This is a *foreign key*. It means "this value must point to an existing row in another table." If you try to log a request for a fic that doesn't exist, PostgreSQL will say "nope!"
- **`UNIQUE(...)`** — These columns together must be unique. No two rows can have the same combination. This prevents duplicate entries.

> ⚠️ **Watch Out: Foreign Keys and Delete Order**
>
> Notice that `export_log` has `REFERENCES fic_info(id)`. This means you can't delete a row from `fic_info` if there are rows in `export_log` that point to it. If you try, PostgreSQL will throw an error. This is actually a *good* thing — it prevents orphaned data. But it means you need to think about deletion order: delete children first, then parents.

### Running Migrations

SQLx provides a command-line tool. Install it:

```bash
cargo install sqlx-cli
```

Then create your database URL and run migrations:

```bash
export DATABASE_URL="postgres://localhost/fichub"
sqlx migrate run
```

You'll see output like:

```
Applied 1 migration(s) (0.015s)
```

SQLx remembers which migrations have been applied. Run it again, and it says:

```
No migrations run; already at the latest.
```

Other useful commands:

```bash
sqlx migrate add add_user_table    # Creates a new migration file
sqlx migrate revert                 # Undo the last migration
sqlx migrate info                   # Show migration status
```

### Connecting in Rust

Now the fun part — connecting from our Rust code. Here's how FicHub does it in `src/db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Initialize the database connection pool and run migrations
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Run migrations from the migrations directory relative to the binary
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("migrations")
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
        }
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    };

    if migrations_path.exists() {
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;
        tracing::info!("Database migrations applied");
    }

    Ok(pool)
}
```

Let's unpack this:

1. **`PgPoolOptions::new()`** — Creates a builder for our connection pool.
2. **`.max_connections(20)`** — We allow up to 20 simultaneous database connections. Why not one? Because our server handles many requests at the same time — if every request had to wait for a single connection, things would grind to a halt.
3. **`.acquire_timeout(Duration::from_secs(10))`** — If all 20 connections are busy, wait up to 10 seconds before giving up.
4. **`.connect(database_url).await?`** — Actually connect! The `.await` means "this is an async operation — the computer might need to do network stuff."
5. **`sqlx::migrate::Migrator`** — Run our migration files automatically. Notice we check if the path exists first — this lets the code work both during development (when the path is in the source tree) and after building (when it's next to the binary).

> 💡 **Key Concept: Connection Pools**
>
> A connection pool is like a checkout desk at a library. Instead of each borrower (each HTTP request) waiting for a librarian to open a brand-new library for them, there's one shared library with 20 checkout windows. When a request comes in, it grabs an available window. When it's done, it returns the window for the next request. Much faster than building a new library for every borrower!
>
> Why 20 connections? It's a balance. Too few connections means requests wait in line. Too many connections wastes memory and can overwhelm PostgreSQL. For a typical web server, 20 is a good starting point. You can tune this later based on load testing.

### How It Fits in main.rs

Remember our `AppState` from Part 2? Here's how we wire the database in:

```rust
// In main.rs
let database_url = &config.database_url;
let db_pool = db::init_pool(database_url).await
    .expect("Failed to connect to database");

let state = Arc::new(AppState {
    config: config.clone(),
    db: db_pool,        // <-- The pool goes here!
    redis: redis_client,
    http_client: http_client,
    // ... other fields
});
```

Now every handler can access `state.db` — the connection pool — and run queries against our database. The pool handles connection management automatically: borrowing connections, returning them, and even reconnecting if the database hiccups.

> ⚠️ **Watch Out: Database URLs**
>
> A typical PostgreSQL URL looks like this:
> `postgres://username:password@localhost:5432/fichub`
>
> That's `postgres://` + user + `:` + password + `@` + host + `:` + port + `/` + database name.
>
> Make sure your `DATABASE_URL` environment variable is correct. The most common mistakes are:
> - Forgetting the password
> - Using the wrong port (default is 5432)
> - Having the username wrong (on Linux, it's often `postgres` for the default user)
> - Forgetting to actually create the database with `createdb`

---

## Chapter 10: Models and Queries

### From Database Rows to Rust Structs

When you SELECT data from a database, it comes back as rows. But in Rust, we want to work with *structs*. SQLx bridges this gap with `#[derive(FromRow)]`.

Here's how FicHub defines its main model in `src/db/models.rs`:

```rust
use chrono::{DateTime, Utc};
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
}
```

Let's look at the derive macros:

- **`FromRow`** — The star of the show. This tells SQLx "I can be created from a database row." The column names in the table must match the field names in the struct. If your database column is called `chapters` but your struct field is called `num_chapters`, you'll need a `#[sqlx(rename = "chapters")]` attribute to tell SQLx about the mismatch.
- **`Serialize` / `Deserialize`** — From `serde`. Lets us convert this struct to/from JSON. This is essential for API responses.
- **`Debug`** — Lets us print it for debugging with `println!("{:?}", fic)`.
- **`Clone`** — Lets us make copies of it. Useful when you need to pass the same fic info to multiple functions.

Notice the `Option<T>` fields. That means "this column might be NULL in the database." For example, `author_url` is `Option<String>` because some fics might not have an author URL. In Rust, we're forced to handle the possibility that the value is missing — no accidentally using a null pointer!

Compare the `FicInfo` struct to the database schema. The fields match up perfectly:

- `id: String` maps to `id VARCHAR(128) PRIMARY KEY`
- `chapters: i32` maps to `chapters INT4 NOT NULL`
- `words: i64` maps to `words INT8 NOT NULL`
- `Option<String>` maps to nullable `TEXT` columns

This isn't a coincidence — FicHub's models are designed to mirror the database exactly. This makes the code easier to understand: when you see `fic.chapters`, you know exactly which database column it came from.

### The Full Model Collection

FicHub has several models beyond `FicInfo`. Here's the `RequestSource`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestSource {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub is_automated: Option<bool>,
    pub route: Option<String>,
    pub description: Option<String>,
}
```

And `RequestLog`:

```rust
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

And `ExportLog`, which tracks cached exports:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}
```

Each model corresponds to a database table. The field names match the column names. This is the "model layer" of our application — it's the bridge between the database and our Rust logic.

### Building Queries

SQLx gives you two main ways to build queries:

**`sqlx::query()`** — For queries that don't return structured data (INSERT, UPDATE, DELETE):

```rust
use sqlx::PgPool;

async fn insert_example(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO fic_info (id, title, author, chapters, words, description, status, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind("ao3_123456")
    .bind("The Best Story Ever")
    .bind("JaneDoe")
    .bind(12_i32)
    .bind(85000_i64)
    .bind("A great story")
    .bind("complete")
    .bind("https://archiveofourown.org/works/123456")
    .execute(pool)
    .await?;

    Ok(())
}
```

**`sqlx::query_as()`** — For queries that return data (SELECT):

```rust
async fn get_example(pool: &PgPool) -> Result<Option<FicInfo>, sqlx::Error> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind("ao3_123456")
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
```

The difference: `query()` returns a `QueryResult` (which tells you how many rows were affected). `query_as()` returns your struct (or a collection of them).

### Binding Parameters with .bind()

See those `$1`, `$2`, `$3` in the SQL? Those are *parameters*. They're placeholders for values you'll provide later. The `.bind()` method fills them in, in order.

Why not just use string interpolation like `format!("SELECT * FROM fic_info WHERE id = '{}'", id)`?

**Because SQL injection.** If someone passes a value containing SQL code, string interpolation would execute it. Parameters are safe — the database treats them as *data*, never as *code*.

> ⚠️ **Watch Out: SQL Injection**
>
> NEVER build SQL queries by concatenating strings with user input. Always use `.bind()` with parameterized queries. SQL injection is one of the most dangerous and common security bugs in web applications.
>
> ```rust
> // ❌ DANGEROUS - never do this
> let query = format!("SELECT * FROM fic_info WHERE id = '{}'", user_input);
>
> // ✅ SAFE - use .bind()
> sqlx::query("SELECT * FROM fic_info WHERE id = $1")
>     .bind(&user_input)
> ```
>
> The `$1` parameters are safely escaped by the database driver. User input can never "break out" of them. Even if someone passes `'; DROP TABLE fic_info; --` as input, it gets treated as a literal string value, not SQL code.

### Fetching a Single Fic

Here's the real `get_fic_info` query from FicHub:

```rust
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

Key details:

- **`query_as::<_, FicInfo>`** — The first type parameter is the "output type" (we don't need to specify it; Rust figures it out). The second is our struct.
- **`.fetch_optional(pool)`** — Returns `Option<FicInfo>`. If no row matches, it returns `None`. If one row matches, it returns `Some(fic)`. If multiple match, it takes the first one.
- **`.fetch_one(pool)`** — Returns exactly one row, or an error if there's zero or more than one. Use this when you're certain there's exactly one result.
- **`.fetch_all(pool)`** — Returns a `Vec<FicInfo>` with all matching rows. Use this when you expect multiple results.

### The Upsert: INSERT ON CONFLICT

The most important query in FicHub is the upsert. When a user requests a fic, we scrape the metadata and save it. But we might have scraped this fic before — so we want to *update* if it exists, or *insert* if it doesn't.

Here's the real upsert from `src/db/queries.rs`:

```rust
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

The key line is:

```sql
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    chapters = EXCLUDED.chapters,
    ...
```

This says: "If there's already a row with this `id`, update its `title`, `chapters`, `words`, and other fields with the new values. Don't fail — just update."

`EXCLUDED` is a special keyword that refers to the values you were *trying* to insert. So `EXCLUDED.title` means "the title from the VALUES clause" (the new data). It's like a shortcut to refer to the incoming row without repeating yourself.

Notice that `NOW()` is used for the `updated` timestamp. This is a PostgreSQL function that returns the current time. We don't pass it from Rust — we let the database handle the timekeeping, which avoids clock skew issues between the app server and the database server.

### Searching with ILIKE

FicHub has a "did you mean?" feature. When you search for a fic, it looks for similar titles or authors:

```rust
pub async fn search_similar_fics(pool: &PgPool, query: &str) -> AppResult<Vec<FicInfo>> {
    let rows = sqlx::query_as::<_, FicInfo>(
        r#"SELECT * FROM fic_info
           WHERE title ILIKE $1 OR author ILIKE $2
           LIMIT 5"#,
    )
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

`ILIKE` is PostgreSQL's case-insensitive LIKE. The `%` characters are wildcards — `%jane%` matches any title containing "jane", like "The Adventures of Jane", "Jane's Journal", etc.

We limit to 5 results so we don't overwhelm the user with suggestions. The `format!("%{}%", query)` pattern creates a substring match — searching for "harry" matches "Harry Potter", "A Boy Called Harry", etc.

### 🧪 Try It Yourself

1. Create the `fic_info` table in your database
2. Write a Rust function that connects to the database
3. Upsert a `FicInfo` struct (insert it for the first time)
4. Fetch it back with `get_fic_info` — did you get it?
5. Change the title, upsert again, and fetch it back — did the title update?
6. Try fetching a fic that doesn't exist — what does `fetch_optional` return?
7. Insert three fics by the same author, then use the ILIKE search to find them all

---

## Chapter 11: Request Logging

### Why Track Requests?

When you run a web service, you need to know what's happening. Questions like:

- How many EPUBs are we generating per day?
- Which fanfiction sites are most popular?
- How long does scraping take on average?
- Are bots hitting our API?
- Did that bug report correspond to a real export failure?

All of these are answered by *request logging*. Every time someone hits our export endpoint, we record what happened. It's like a security camera for your server — you might not look at the footage every day, but when something goes wrong, you'll be glad it's there.

### The request_source Table

First, we need to know *who* is making requests. FicHub groups requests by "source" — essentially, what client is asking:

```sql
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);
```

The corresponding Rust struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RequestSource {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub is_automated: Option<bool>,
    pub route: Option<String>,
    pub description: Option<String>,
}
```

Notice the `UNIQUE(is_automated, route, description)`. This means we won't have duplicate sources. If someone requests `/api/v0/epub` from a web browser (not automated), there's exactly one `request_source` row for that combination.

The `is_automated` field is interesting. FicHub wants to distinguish between human users and automated scrapers. If `is_automated` is `true`, the request came from a bot or script. This helps us understand our real user base vs. automated traffic.

### Inserting a Request Source

Here's the real query from FicHub:

```rust
pub async fn insert_request_source(
    pool: &PgPool,
    is_automated: bool,
    route: &str,
    description: &str,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO request_source (is_automated, route, description)
           VALUES ($1, $2, $3)
           ON CONFLICT (is_automated, route, description)
           DO UPDATE SET route = EXCLUDED.route
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

This is another upsert — if the source already exists, we don't create a duplicate. The magic part is `RETURNING id`. After inserting (or finding the existing row), PostgreSQL sends back the `id` of the row.

The return type `(i64,)` is a Rust tuple with one element. SQLx uses tuples for simple queries where you don't need a full struct. We access the id with `row.0` (the first — and only — element of the tuple). It's a bit quirky, but it's efficient and clear once you get used to it.

### The request_log Table

Now that we know *who* is asking, we record *what* they asked for:

```sql
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

And the Rust struct:

```rust
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

Let's look at the interesting fields:

- **`etype`** — The export type: `"epub"`, `"html"`, `"mobi"`, or `"pdf"`. This tells us which format was requested.
- **`query`** — The URL the user requested (e.g., `https://archiveofourown.org/works/123456`). This is what they pasted into the search box.
- **`info_request_ms`** — How long the metadata lookup took, in milliseconds. If this is over 5000ms, the scraper for that site is probably slow.
- **`export_ms`** — How long the EPUB generation took. If this is over 10 seconds, we know something's wrong with our export pipeline.
- **`fic_info`** — A JSON snapshot of the metadata at the time of the request. This is the entire `FicMetadata` struct serialized as a JSON string. Useful for debugging: "What did we think this fic was called when the user downloaded it?"
- **`export_file_hash`** — The hash of the generated file. This links the log entry to a specific version of the export.

### Inserting a Request Log

```rust
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
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO request_log
           (source_id, etype, query, info_request_ms,
            url_id, fic_info, export_ms,
            export_file_name, export_file_hash, url)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
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
    .execute(pool)
    .await?;
    Ok(())
}
```

Notice the `Option<&str>` parameters. These represent nullable columns — if we don't have the data, we pass `None`, and the column gets NULL in the database. This is a clean way to handle optional data.

### Putting It All Together: The Export Handler

Now let's see how the export handler in `src/routes/export.rs` ties all of this together. When someone requests an EPUB, here's the complete flow:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    // 1. Find the right scraper for this URL
    let scraper = state.scraper_registry.find_scraper(query)?;

    // 2. Scrape metadata (how long did that take?)
    let meta = scraper.lookup(&state.http_client, query).await?;
    let info_request_ms = start.elapsed().as_millis() as i32;

    // 3. Save metadata to database
    let fic_info_row = FicInfo {
        id: meta.url_id.clone(),
        title: meta.title.clone(),
        author: meta.author.clone(),
        author_url: Some(meta.author_url.clone()),
        author_local_id: Some(meta.author_local_id.clone()),
        chapters: meta.chapters,
        words: meta.words,
        description: meta.desc.clone(),
        fic_created: chrono::DateTime::from_timestamp_millis(meta.published)
            .unwrap_or_default(),
        fic_updated: chrono::DateTime::from_timestamp_millis(meta.updated)
            .unwrap_or_default(),
        status: meta.status.clone(),
        source: meta.source.clone(),
        extra_meta: meta.extra_meta.clone(),
        raw_extended_meta: meta.raw_extended_meta.clone(),
        source_id: Some(meta.source_id),
        author_id: Some(meta.author_id),
        content_hash: meta.content_hash.clone(),
    };
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // 4. Check blacklists, check cache...

    // 5. If cache miss: fetch chapters, generate EPUB
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
    let (epub_path, epub_hash) = export::epub::create_epub(
        &meta, &chapters, &state.config.tmp_dir
    ).await?;

    // 6. Record timing
    let export_ms = start.elapsed().as_millis() as i32;

    // 7. Log the request
    let source_id = queries::insert_request_source(
        &state.db, false, "/api/v0/epub", "web request",
    ).await?;

    let fic_json = serde_json::to_string(&meta).ok();
    queries::insert_request_log(
        &state.db, source_id, "epub", query, info_request_ms,
        Some(&meta.url_id), fic_json.as_deref(),
        Some(export_ms), Some(&format!("{}.epub", epub_hash)),
        Some(&epub_hash), Some(query),
    ).await?;

    // 8. Return JSON response
    Ok(Json(json!({
        "err": 0,
        "q": query,
        "info": info_str,
        "url_id": meta.url_id,
        // ... more fields
    })))
}
```

Notice the pattern:

1. **Start a timer** (`let start = Instant::now()`)
2. **Do the work** (scrape, generate, etc.)
3. **Measure elapsed time** (`start.elapsed().as_millis()`)
4. **Insert the source** (who's requesting)
5. **Insert the log** (what happened, how long it took)

This timing data is incredibly valuable. After running FicHub for a month, you can run queries like:

```sql
-- Average export time per day
SELECT DATE(created) as day, AVG(export_ms) as avg_ms
FROM request_log
WHERE export_file_name IS NOT NULL
GROUP BY DATE(created)
ORDER BY day DESC;

-- Most popular sources
SELECT source, COUNT(*) as requests
FROM request_log
GROUP BY source
ORDER BY requests DESC
LIMIT 10;

-- Slowest exports (might need optimization)
SELECT query, export_ms
FROM request_log
WHERE export_ms > 5000
ORDER BY export_ms DESC
LIMIT 20;
```

### Building the Info String

FicHub shows users a nice summary of the fic. Here's how it builds that string:

```rust
pub fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) {
    let relative_time = {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - meta.updated;
        let diff_secs = diff_ms / 1000;
        if diff_secs < 60 {
            "less than a minute ago".to_string()
        } else if diff_secs < 3600 {
            format!("{} minutes ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{} hours ago", diff_secs / 3600)
        } else {
            format!("{} days ago", diff_secs / 86400)
        }
    };

    let info = format!(
        "{title} by {author}\n{words} words in {chapters} chapters\n\
         Status: {status}\nUpdated: {date} - {relative} ago\n",
        title = meta.title,
        author = meta.author,
        words = meta.words,
        chapters = meta.chapters,
        status = meta.status,
        date = updated_str,
        relative = relative_time,
    );

    (info, Vec::new())
}
```

This produces output like:

```
The Best Story Ever by JaneDoe
85000 words in 12 chapters
Status: complete
Updated: 2024-01-15 10:30:00 - 5 days ago
```

The function returns a tuple: the info string, and a vector of notes (warnings or special messages about the fic). The notes are empty for normal fics, but might contain messages like "This fic is greylisted" for fics that have been flagged.

### The Insert-Then-Return Pattern

You'll notice a pattern throughout FicHub's queries:

```rust
let source_id = queries::insert_request_source(
    &state.db, false, "/api/v0/epub", "web request",
).await?;
```

We insert a record and get back its ID in one step, using `RETURNING id`. This is more efficient than two separate queries (insert, then select to get the ID). PostgreSQL supports this natively, and it's a common pattern in real applications.

```sql
INSERT INTO request_source (is_automated, route, description)
VALUES ($1, $2, $3)
ON CONFLICT (is_automated, route, description) DO UPDATE SET route = EXCLUDED.route
RETURNING id
```

The `RETURNING` clause tells PostgreSQL: "After you're done inserting or updating, send me back the `id` column." We fetch it with `.fetch_one()` and extract the value from the tuple.

> 💡 **Key Concept: RETURNING**
>
> `RETURNING` is like getting a receipt after a purchase. You hand over money (INSERT), and you get back a receipt (the ID). Some databases don't support this — you'd have to do two queries. PostgreSQL makes it easy.
>
> You can also `RETURNING *` to get back the entire row, which is useful when the database fills in defaults (like `created` timestamps). This saves a round-trip: instead of INSERT + SELECT, you do INSERT ... RETURNING * and get everything back in one shot.

### Why Not Log Everything?

You might wonder: why don't we log *every* request, including cache hits?

Performance. Logging every single request would create a LOT of data. If your server handles 10,000 requests per day and you log every one, that's 3.65 million rows per year. The database would grow fast, and queries would slow down.

FicHub is selective: it logs new exports (cache misses) but not every cache hit. This gives you the data you need for analytics without overwhelming the database. If you need more detailed logging later, you can add it — that's the beauty of having a flexible schema.

### 🧪 Try It Yourself

1. Create both the `request_source` and `request_log` tables
2. Write an `insert_request_source` function that returns the ID
3. Call it twice with the same parameters — does it return the same ID? (It should!)
4. Write an `insert_request_log` function
5. Insert a source, then insert a log entry using the source's ID
6. SELECT back from `request_log` and verify all the fields are there
7. Try this aggregate query: count how many requests each source has made:
   ```sql
   SELECT rs.description, rs.route, COUNT(rl.id) as request_count
   FROM request_source rs
   LEFT JOIN request_log rl ON rl.source_id = rs.id
   GROUP BY rs.description, rs.route
   ORDER BY request_count DESC;
   ```
8. What's the average time between `info_request_ms` and `export_ms`? Can you find the slowest export?

### What We Built

In these four chapters, we went from zero to a fully database-backed server:

- **Chapter 8**: We learned what databases are and how SQL works. Tables, rows, columns, and the five basic operations. We installed PostgreSQL, created a database, and ran queries by hand.
- **Chapter 9**: We connected SQLx to PostgreSQL with compile-time checked queries and automatic migrations. We learned about connection pools and why 20 is a good starting number.
- **Chapter 10**: We defined Rust structs that mirror our database tables, and built queries to insert, update, and fetch data. We learned about upserts, parameterized queries, and the dangers of SQL injection.
- **Chapter 11**: We built a complete request logging system that tracks who's using our service and how performant it is. We saw how `RETURNING` lets us insert and fetch IDs in one step, and how the export handler ties everything together.

This is the backbone of FicHub. Every request flows through these database operations. The scraper scrapes, the database stores, the logger records. In the next part, we'll build the caching layer — because nobody wants to re-scrape a 50-chapter epic on every request.
