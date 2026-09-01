# Part 3 — Database Foundations

The server boots, but it has nowhere to store data. Every fanfiction, every user, every rating, every bookmark needs a database. In this part you set up PostgreSQL, read the schema, and write your first query.

---

## 3.1 PostgreSQL + SQLx setup: `.env`, connection pool

FicHub uses PostgreSQL. The connection is configured via `DATABASE_URL` in `.env`.

### The `.env` file

Copy the example:

```bash
cp .env.example .env
```

Open `.env` and set:

```bash
DATABASE_URL=postgres://fichub:password@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
```

You need a PostgreSQL server running. If you have Docker:

```bash
docker run --name fichub-postgres -e POSTGRES_USER=fichub -e POSTGRES_PASSWORD=password -e POSTGRES_DB=fichub -p 5432:5432 -d postgres:15
```

### The connection pool: `src/db/mod.rs`

Open `src/db/mod.rs`:

```rust
// src/db/mod.rs
pub mod models;
pub mod queries;
pub mod reviews;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    if std::env::var("FICHUB_SKIP_MIGRATIONS").is_ok() {
        tracing::info!("FICHUB_SKIP_MIGRATIONS set — skipping migrations on boot");
        return Ok(pool);
    }

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

### Breakdown

**`PgPoolOptions::new()`** — builds a connection pool configuration.

- `.max_connections(20)` — at most 20 simultaneous database connections. PostgreSQL can handle more, but 20 is plenty for a single server.
- `.acquire_timeout(Duration::from_secs(10))` — if all 20 connections are busy, wait up to 10 seconds for one to free up. If it times out, the request fails.
- `.connect(database_url)` — connect to PostgreSQL using the URL.

**`FICHUB_SKIP_MIGRATIONS`** — an environment variable that skips migrations on boot. This is used in production where migrations are run as a separate deploy step (`deploy.sh` runs `migrate` before restarting the service). Skipping migrations on boot avoids "migration surprise" during a restart.

**`env!("CARGO_MANIFEST_DIR")`** — a compile-time macro that expands to the directory containing `Cargo.toml`. This is where the `migrations/` folder lives.

**`sqlx::migrate::Migrator::new(migrations_path)`** — creates a migrator that reads the `migrations/` directory.

**`.run(&pool)`** — runs all pending migrations against the pool. Migrations are tracked in a `sqlx_migrations` table — each migration runs once.

### The migration table

SQLx creates a table called `sqlx_migrations` to track which migrations have run:

```sql
CREATE TABLE sqlx_migrations (
    version bigint NOT NULL PRIMARY KEY,
    description text NOT NULL,
    checksum bytea NOT NULL,
    applied_at timestamp with time zone NOT NULL
);
```

Each migration file has a version number (the filename, like `001_initial.sql`) and a checksum. SQLx computes the checksum when the migration runs and stores it. On the next boot, it checks if the migration version is in the table. If not, it runs it. If the checksum doesn't match (the file was modified after running), SQLx refuses to start — this prevents accidental schema changes.

---

## 3.2 `migrations/001_initial.sql`: the schema

Open `migrations/001_initial.sql`. This is a large file (8741 lines) because it's a consolidated schema dump from a live production database. Let's look at the key parts.

### Extensions

```sql
CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;
CREATE EXTENSION IF NOT EXISTS vector WITH SCHEMA public;
```

- **pg_trgm** — trigram-based text search. Used for fuzzy matching on titles and authors.
- **vector** — vector data type. Used for embeddings (Ollama-generated vectors for fic content).

### The trigger: `update_fic_tag_score()`

```sql
CREATE FUNCTION public.update_fic_tag_score() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;
```

This is a trigger function. It maintains `fic_tags.score` automatically. When a tag is added to a fic (`INSERT`), the score goes up. When a tag is removed (`DELETE`), the score goes down. When a tag's value changes (`UPDATE`), the score is adjusted. This is how FicHub ranks tags — the score reflects how many users have applied the tag.

### Core tables (excerpt)

The schema has many tables. Here are the core ones:

- **`fic_info`** — fanfiction metadata (url_id, title, author, words, chapters, etc.)
- **`works`** — canonical work entries (unified works that merge multiple fic_info rows)
- **`users`** — user accounts
- **`user_bookmarks`** — user bookmarks
- **`user_ratings`** — user ratings (5-star)
- **`user_kudos`** — user kudos (one-click likes)
- **`reviews`** — user reviews
- **`comments`** — threaded comments
- **`follows`** — follows (users follow authors/works)
- **`fic_tags`** — tags applied to fics, with scores
- **`tags`** — tag definitions
- **`notifications`** — user notifications
- **`reading_lists`** — reading lists (bundles)
- **`shelves`** — shelves/collections
- **`collections`** — AO3-style collections
- **`forum_categories`**, **`forum_topics`**, **`forum_posts`** — forum
- **`bounties`** — bounty requests
- **`saved_searches`** — saved searches with alerts

The file is consolidated from a live production dump, so it includes tables for features that evolved over time. Not every table is used in every part of this tutorial — we focus on the ones relevant to each feature.

### Why consolidated?

The comment at the top says: "consolidated schema (generated from live prod dump)". This means the migration was created by dumping the entire production database schema and pasting it into a single file. This is a pragmatic choice — it ensures the local development database matches production exactly. The downside is that the migration is large and not granular. But it works.

---

## 3.3 `src/db/models.rs`: SQLx `FromRow` structs

Open `src/db/models.rs`. This file defines Rust structs that map to database rows. SQLx uses the `FromRow` derive macro to automatically map columns to struct fields.

### The first struct: `FicInfo`

```rust
// src/db/models.rs (lines 5-28)
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

### Breakdown

- **`#[derive(FromRow)]`** — SQLx can map a database row to this struct automatically. Each field must match a column name (case-insensitive).
- **`#[derive(Serialize, Deserialize)]`** — serde can convert this to/from JSON. This is needed when the struct is returned from an API endpoint.
- **`#[derive(Debug, Clone)]`** — standard Rust derives for debugging and cloning.

### Field types

- `id: String` — the url_id, a string identifier for the fic.
- `created: Option<DateTime<Utc>>` — when the row was created in FicHub's database. `Option` means it can be null.
- `title: String` — the fic title.
- `author: String` — the author name.
- `author_url: Option<String>` — the author's profile URL on the source site (optional).
- `chapters: i32` — number of chapters.
- `words: i64` — word count.
- `description: String` — the fic synopsis.
- `fic_created: DateTime<Utc>` — when the fic was originally published on the source site.
- `status: String` — "Completed", "In Progress", etc.
- `source: String` — which site this came from (e.g., "AO3", "FanFiction.net").
- `work_id: Option<i32>` — the ID of the canonical work this fic is part of (if merged).

### Other structs (summary)

The file has many more structs:

- **`WorkRow`** — canonical work entry (id, title, author, description, etc.)
- **`Follow`** — a follow relationship
- **`FollowExclusion`** — an exclusion on a follow
- **`Notification`** — a notification
- **`NotificationPreference`** — notification settings
- **`BadgeDefinition`** — badge definition
- **`UserBadge`** — earned badge
- **`ReadingStats`** — per-work reading stats
- **`Shelf`**, **`WorkShelf`** — shelves and their contents
- **`ReadingList`**, **`ReadingListItemRow`** — reading lists
- **`CollectionInfo`**, **`CollectionItemRequestRow`** — collections
- **`Locale`**, **`Translation`**, **`WorkTranslation`** — translations
- **`LeaderboardWeekly`**, **`LeaderboardMonthly`** — leaderboards

Each struct represents a table or a view in the database. When you write a query that returns rows, you use these structs to get typed results.

---

## 3.4 `src/db/queries.rs`: your first query function

Open `src/db/queries.rs`. This is where all database queries live. It's a large file (3594 lines) because it contains every query in the application.

Let's look at a simple query:

```rust
// src/db/queries.rs (excerpt — first 80 lines)
use chrono::Datelike;
use sqlx::{PgPool, Row};
use crate::db::models::*;
use crate::error::AppResult;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SiteCredRow {
    pub domain: String,
    pub username: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn set_site_credentials(
    pool: &PgPool,
    user_id: i32,
    domain: &str,
    username: &str,
    password_enc: &str,
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO user_site_credentials (user_id, domain, username, password_enc, created_at, expires_at)
           VALUES ($1, $2, $3, $4, now(), $5)
           ON CONFLICT (user_id, domain) DO UPDATE
           SET username = EXCLUDED.username,
               password_enc = EXCLUDED.password_enc,
               created_at = now(),
               expires_at = EXCLUDED.expires_at"#,
    )
    .bind(user_id)
    .bind(domain.trim().to_lowercase())
    .bind(username)
    .bind(password_enc)
    .bind(expires_at)
    .execute(pool)
    .await?;
    let _ = sqlx::query("DELETE FROM user_site_credentials WHERE user_id = $1 AND expires_at <= now()")
        .bind(user_id)
        .execute(pool)
        .await;
    Ok(())
}
```

### Breakdown

**`sqlx::query(string)`** — creates a query from a raw SQL string.

**`$1, $2, $3, ...`** — positional parameters. SQLx binds values to these positions.

**`.bind(value)`** — binds a value to the next positional parameter. The value must implement `Borrow<dyn Encode<'_, Postgres>>` — most types do.

**`.execute(pool)`** — executes the query against the pool. Returns the number of affected rows.

**`ON CONFLICT (user_id, domain) DO UPDATE`** — PostgreSQL upsert syntax. If a row with the same `(user_id, domain)` already exists, update it instead of inserting.

### A query that returns rows: `get_fic_info`

Further down in the file, there's a query that returns data:

```rust
// src/db/queries.rs (excerpt)
pub async fn get_fic_info(pool: &PgPool, url_id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        r#"SELECT * FROM fic_info WHERE id = $1"#
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

**`sqlx::query_as::<_, FicInfo>(sql)`** — like `sqlx::query`, but maps the result to `FicInfo` structs automatically (using `FromRow`).

**`.fetch_optional(pool)`** — returns `Option<FicInfo>`. If no row matches, returns `None`. If one row matches, returns `Some(FicInfo)`. If multiple rows match, returns an error (this query should return at most one row).

**`Ok(row)`** — wraps the result in `AppResult<Option<FicInfo>>`.

---

## 3.5 Try It Yourself: write a query that counts works by fandom

Now you write a query.

### Step 1: Open `src/db/queries.rs`

Scroll to the end of the file. You'll add your query there.

### Step 2: Add the query function

```rust
// Add this at the end of src/db/queries.rs
pub async fn count_works_by_fandom(pool: &PgPool) -> AppResult<Vec<(String, i64)>> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT f.source, COUNT(*)::bigint
        FROM fic_info f
        GROUP BY f.source
        ORDER BY COUNT(*) DESC
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

### Step 3: Understand the query

- **`SELECT f.source, COUNT(*)::bigint`** — select the source (fandom/site) and the count, cast to bigint.
- **`FROM fic_info f`** — from the fic_info table, aliased as `f`.
- **`GROUP BY f.source`** — group by source, so each source gets one row.
- **`ORDER BY COUNT(*) DESC`** — order by count, highest first.

### Step 4: Add a test in `src/db/mod.rs`

Open `src/db/mod.rs` and add a test at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::queries;

    #[sqlx::test]
    async fn test_count_works_by_fandom(pool: sqlx::PgPool) -> anyhow::Result<()> {
        // This test needs data. For now, just check it runs without error.
        let result = queries::count_works_by_fandom(&pool).await?;
        println!("Works by fandom: {:?}", result);
        Ok(())
    }
}
```

### Step 5: Run the test

```bash
cargo test count_works_by_fandom
```

**Expected output**: the test compiles and runs. If the database is not set up, it fails. If it is set up, it prints the counts.

### What you learned

- How to write a query that returns rows.
- How to use `sqlx::query_as` with a tuple type.
- How to group and order results.
- How to write a `#[sqlx::test]` integration test.

---

## 3.6 The `sqlx::test` macro

The `#[sqlx::test]` macro is a testing utility provided by SQLx. It creates a temporary PostgreSQL database for each test, runs the migrations, and passes the pool to the test function.

To use it, you need a PostgreSQL server running. The macro connects to it, creates a test database, runs migrations, and tears it down after the test.

In `Cargo.toml`, you need the `sqlx` dependency with the `test` feature (it's already there in the main dependency list).

```bash
cargo test -- --test-threads=1
```

The `--test-threads=1` flag ensures tests run one at a time, because each test needs its own database.

---

## 3.7 What you have now

- You understand the `.env` configuration for the database.
- You understand the connection pool: `PgPoolOptions`, `max_connections`, `acquire_timeout`.
- You understand migrations: `Migrator`, `sqlx_migrations` table, checksums.
- You understand the `001_initial.sql` schema: extensions, triggers, core tables.
- You understand `FromRow` structs: how SQLx maps rows to Rust structs.
- You understand query functions: `sqlx::query`, `sqlx::query_as`, `.bind()`, `.fetch_optional()`, `.fetch_all()`.
- You wrote a query that counts works by fandom.
- You wrote a `#[sqlx::test]` integration test.

Next: Part 4 — Get a Single Work. You will build the `GET /api/works/:id` endpoint and the frontend page that displays it.

---

*End of Part 3. On to [Part 4 — Get a Single Work](./04-get-a-single-work/04-get-a-single-work.md).*
