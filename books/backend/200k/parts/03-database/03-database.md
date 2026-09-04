# Part 3: Database Layer

---

In Part 2, we built the HTTP foundation of our server — the router, the middleware, the error system, and the shared `AppState`. But right now, every time the server restarts, all the data disappears. That's because our data lives in memory, and memory is temporary.

In this part, we're going to fix that. We'll connect to a real database — PostgreSQL — and start storing fanfiction metadata, request logs, cache entries, and blacklist records. By the end of this part, our server will remember everything between restarts.

This is a big step. Up until now, our server was like a goldfish — every restart was a fresh start. After this part, it'll have a memory. And a good memory is what separates a toy from a tool.

Let's dive in.

---

# Chapter 13: PostgreSQL Basics

## What Is a Database?

Imagine you have a really, really organized filing cabinet. Each drawer is a **table**. Inside each drawer, you have index cards — those are **rows**. And each card has labeled fields — those are **columns**.

That's essentially what a database is: a structured way to store, find, and update information.

Here's a simple example. Say we want to store information about fanfics:

```
| id       | title               | author       | words  | status      |
|----------|---------------------|--------------|--------|-------------|
| abc123   | The Best Fic Ever   | CoolAuthor   | 50000  | complete    |
| def456   | Midnight Dreams     | NightWriter  | 120000 | in-progress |
| ghi789   | The Long Road Home  | Wanderer     | 85000  | complete    |
```

Each row is one fanfic. Each column is a piece of information about that fanfic. The `id` column is special — it's called a **primary key**, and it uniquely identifies each row. No two rows can have the same primary key, so you can always find exactly the row you want.

But databases are way more than just a table on paper. They can:

- **Store millions of rows** without slowing down (thanks to indexes)
- **Enforce rules** — you can't insert a fic with no title, or a word count of -5
- **Connect related data** — link a fic to its author, link a request log to the fic it was about
- **Search efficiently** — find all fics by a specific author in milliseconds, even with millions of rows
- **Handle concurrent access** — 100 users can read and write at the same time without conflicts

## Why PostgreSQL?

There are many databases out there — SQLite, MySQL, MongoDB, and more. We're choosing **PostgreSQL** (often just called "Postgres") for several reasons:

1. **It's free and open source.** No license fees. No vendor lock-in. You own your data.

2. **It's incredibly reliable.** It's been around since the 1990s and is battle-tested by companies big and small.

3. **It handles complex queries.** When you need to search, filter, and combine data from multiple tables, Postgres is excellent.

4. **It supports advanced features.** Full-text search, JSON storage, automatic timestamps, triggers, and stored procedures — Postgres has it all built in.

5. **It plays nicely with Rust.** The `sqlx` crate (which we'll use in the next chapter) has first-class PostgreSQL support.

6. **It scales.** Starting with a single developer on a laptop? Postgres works. Growing to millions of requests per day? Postgres handles that too.

> 💡 **Key Concept**
>
> When we say "database" in this chapter, we sometimes mean the whole PostgreSQL server, and sometimes mean a specific database within it. PostgreSQL can run multiple databases on the same server. We'll create one called `fichub` for our app. Think of the PostgreSQL server as the building, and each database as a different office inside it.

## Installing PostgreSQL

The exact steps depend on your operating system. Here are the instructions for the most common ones.

### Arch Linux (our development environment)

```bash
# Install PostgreSQL
sudo pacman -S postgresql

# Initialize the database (first time only!)
sudo -u postgres initdb -D /var/lib/postgres/data

# Start the service
sudo systemctl enable --now postgresql

# Create a user and database
sudo -u postgres createuser -s fichub
sudo -u postgres createdb fichub -O fichub
```

The `-s` flag on `createuser` gives our user "superuser" privileges (useful during development, but in production you'd want more restricted permissions). The `-O` flag on `createdb` sets the owner of the database.

### Ubuntu / Debian

```bash
# Install PostgreSQL
sudo apt update
sudo apt install postgresql

# Create a user and database
sudo -u postgres createuser -s $USER
sudo -u postgres createdb fichub
```

### macOS

```bash
# Using Homebrew
brew install postgresql@16
brew services start postgresql@16

# Create a database
createdb fichub
```

### Verifying the Installation

Once installed, you can check that everything is working:

```bash
psql -U fichub -d fichub -c "SELECT version();"
```

You should see something like:

```
 PostgreSQL 16.2 on x86_64-pc-linux-gnu, compiled by gcc...
```

The **connection string** we'll use throughout this book looks like this:

```
postgresql://fichub@localhost/fichub
```

This says: connect to PostgreSQL on localhost, as user `fichub`, to database `fichub`. If you set a password, the URL becomes:

```
postgresql://fichub:yourpassword@localhost/fichub
```

In practice, you'll want to store this URL in an environment variable, not hardcoded in your source code. The standard approach is to use a `.env` file:

```bash
# .env file (never commit this!)
DATABASE_URL=postgresql://fichub@localhost/fichub
```

And load it in your Rust code with the `dotenvy` crate:

```rust
use dotenvy::dotenv;

fn main() {
    dotenv().ok(); // Load .env file if it exists
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    // Use database_url to connect...
}
```

This way, different environments (development, testing, production) can have different database URLs without changing any code.

> ⚠️ **Watch Out**
>
> Never commit your database URL with a password to git! Use environment variables instead. We'll store it in a `.env` file (which should be in `.gitignore`) or read it from an environment variable at runtime. Hardcoded passwords in source code are one of the most common security mistakes.

## The psql Command Line Tool

PostgreSQL comes with a command-line tool called `psql`. It's like a chat window where you can talk to your database directly. You type SQL commands, and the database responds.

Let's connect:

```bash
psql -U fichub fichub
```

This drops you into a prompt that looks like:

```
fichub=>
```

Now you can type SQL commands directly. A few useful commands for `psql`:

| Command | What it does |
|---------|-------------|
| `\l` | List all databases |
| `\dt` | List all tables in the current database |
| `\d tablename` | Show the structure of a table |
| `\q` | Quit psql |
| `\?` | Show help |

Let's try creating a table:

```sql
fichub=> CREATE TABLE test (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL
);
CREATE TABLE
```

The `CREATE TABLE` message means it worked! Now let's insert some data:

```sql
fichub=> INSERT INTO test (name) VALUES ('Hello'), ('World');
INSERT 0 2
```

The `INSERT 0 2` means: inserted 0 errors, 2 rows. Let's query it:

```sql
fichub=> SELECT * FROM test;
 id |  name
----+-------
  1 | Hello
  2 | World
(2 rows)
```

Type `\q` to exit. You just ran SQL directly against a PostgreSQL database!

## Basic SQL Commands

Let's learn the five essential SQL commands. Think of them as the verbs of database language: CREATE, INSERT, SELECT, UPDATE, and DELETE.

### Creating a Table

```sql
CREATE TABLE fic_info (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    words BIGINT NOT NULL,
    status TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

This creates a table called `fic_info` with six columns. Let's break down the type annotations:

- `TEXT PRIMARY KEY` — a text string that uniquely identifies each row
- `TEXT NOT NULL` — a text string that cannot be empty
- `BIGINT NOT NULL` — a very large integer (8 bytes, up to 9 quintillion)
- `TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP` — a timestamp that defaults to "right now"

The `NOT NULL` constraint means you can't insert a row without providing a value for that column. If you try, the database will reject it with an error.

### Inserting Data

```sql
INSERT INTO fic_info (id, title, author, words, status)
VALUES ('abc123', 'The Best Fic Ever', 'CoolAuthor', 50000, 'complete');
```

This adds one row to our table. Notice we didn't provide `created` — the `DEFAULT CURRENT_TIMESTAMP` took care of that.

You can insert multiple rows at once:

```sql
INSERT INTO fic_info (id, title, author, words, status)
VALUES
    ('abc123', 'The Best Fic Ever', 'CoolAuthor', 50000, 'complete'),
    ('def456', 'Midnight Dreams', 'NightWriter', 120000, 'in-progress'),
    ('ghi789', 'The Long Road Home', 'Wanderer', 85000, 'complete');
```

### Selecting Data

SELECT is where the fun begins. You can filter, sort, and combine data in endless ways:

```sql
-- Get everything
SELECT * FROM fic_info;

-- Get just titles
SELECT title FROM fic_info;

-- Get fics with more than 100,000 words
SELECT title, words FROM fic_info WHERE words > 100000;

-- Search by author
SELECT * FROM fic_info WHERE author = 'CoolAuthor';

-- Sort by word count (descending)
SELECT title, words FROM fic_info ORDER BY words DESC;

-- Get the top 5 longest fics
SELECT title, words FROM fic_info ORDER BY words DESC LIMIT 5;

-- Count how many fics each author has
SELECT author, COUNT(*) as fic_count FROM fic_info GROUP BY author;
```

### Updating Data

```sql
-- Update a specific fic's word count
UPDATE fic_info SET words = 55000 WHERE id = 'abc123';

-- Update multiple columns at once
UPDATE fic_info
SET words = 55000, status = 'complete'
WHERE id = 'abc123';

-- Update all fics by an author
UPDATE fic_info SET status = 'complete' WHERE author = 'CoolAuthor';
```

> ⚠️ **Watch Out**
>
> An UPDATE without a WHERE clause will update EVERY row in the table! Always double-check your WHERE clause before running an update. There's no "undo" in SQL.

### Deleting Data

```sql
-- Delete a specific fic
DELETE FROM fic_info WHERE id = 'abc123';

-- Delete all fics by an author
DELETE FROM fic_info WHERE author = 'CoolAuthor';

-- Delete everything (dangerous!)
DELETE FROM fic_info;
```

> ⚠️ **Watch Out**
>
> Like UPDATE, a DELETE without WHERE deletes everything. Be very careful. In production, some teams disable direct DELETE access entirely and require everything to go through a reviewed migration.

## SQL Data Types

When you create columns, you need to choose a data type. Here are the ones we use most in FicHub:

| Data Type | What It Stores | Size | Example |
|-----------|---------------|------|---------|
| `TEXT` | Any text string | Variable | `'The Best Fic Ever'` |
| `VARCHAR(128)` | Text with a max length | Up to 128 chars | `'abc123'` |
| `INT4` / `INTEGER` | Whole numbers | 4 bytes | `42` |
| `INT8` / `BIGINT` | Very large whole numbers | 8 bytes | `5000000000` |
| `BOOLEAN` | True or false | 1 byte | `TRUE` or `FALSE` |
| `TIMESTAMPTZ` | Date + time + timezone | 8 bytes | `'2024-01-15 10:30:00+00'` |
| `SERIAL` / `BIGSERIAL` | Auto-incrementing number | 4/8 bytes | Automatically: 1, 2, 3... |
| `INET` | An IP address | Variable | `'192.168.1.1'` |
| `REAL` | Decimal numbers | 4 bytes | `3.14` |

> 💡 **Key Concept**
>
> `TIMESTAMPTZ` stores the timezone along with the date and time. This is crucial for a web server — when someone in Tokyo and someone in New York both look at a timestamp, they should see the same moment in time, just displayed in their local timezone. Always use `TIMESTAMPTZ` instead of plain `TIMESTAMP` in web applications.

> 💡 **Key Concept**
>
> Why do we use `TEXT` instead of `VARCHAR`? In PostgreSQL, there's essentially no performance difference. `TEXT` is simpler to work with (no max length to worry about) and is the preferred type in most PostgreSQL projects. We use `VARCHAR(128)` only for the `fic_info.id` column because that has a well-defined maximum length.

> 💡 **Key Concept**
>
> `BIGINT` (8 bytes) vs `INTEGER` (4 bytes): `INTEGER` goes up to about 2.1 billion. That's fine for most counts and IDs. But word counts for very long fanfics can exceed 2 billion (some serials have 10+ million words), so we use `BIGINT` for the `words` column.

## Schema: The Blueprint

In database lingo, the structure of your tables — their names, columns, types, and relationships — is called the **schema**. Think of it like an architectural blueprint for a house. Before you can move furniture in (insert data), you need to build the rooms (create tables).

When we talk about "the database schema" for FicHub, we mean all the tables and their structure. Our schema has these main tables:

- `fic_info` — metadata about every fanfic we've seen
- `request_source` — who is making requests
- `request_log` — what they asked for
- `export_log` — what we've already generated (cache tracking)
- `fic_blacklist` — fics that are blocked
- `author_blacklist` — authors that are blocked
- `fic_version_bump` — cache invalidation records

These tables are related to each other. For example, `request_log.source_id` points to `request_source.id`. This is called a **relationship** — specifically, a foreign key relationship. It means every log entry is connected to exactly one source. We'll explore these relationships in detail in Chapter 15 when we create the schema with migrations.

We'll create this schema using **migrations**, which is the topic of Chapter 15. But first, let's learn how to talk to the database from Rust.

## Relationships Between Tables

Tables don't exist in isolation — they're connected to each other through **relationships**. In FicHub:

- Every `request_log` entry points to a `request_source` via the `source_id` foreign key
- Every `export_log` entry points to a `fic_info` via the `url_id` foreign key
- Every `fic_blacklist` entry points to a `fic_info` via the `url_id` foreign key

These connections form a web of relationships. When you query a request log, you can JOIN with the request source to get more details:

```sql
SELECT rl.query, rl.created, rs.route, rs.is_automated
FROM request_log rl
JOIN request_source rs ON rl.source_id = rs.id
WHERE rl.url_id = 'abc123'
ORDER BY rl.created DESC;
```

This gives you not just the log entry, but also which endpoint was called and whether it was automated. JOINs are one of the most powerful features of relational databases — they let you combine data from multiple tables in a single query.

## A Quick Tour of psql

Before we leave the command line, let's get comfortable with a few more `psql` tricks:

```sql
-- See the structure of a table
\d fic_info

-- Count rows
SELECT COUNT(*) FROM fic_info;

-- Find the longest fic
SELECT title, words FROM fic_info ORDER BY words DESC LIMIT 1;

-- Find all completed fics over 100k words
SELECT title, words FROM fic_info WHERE status = 'complete' AND words > 100000;

-- Delete everything (don't do this in production!)
TRUNCATE fic_info;
```

The `\d tablename` command is especially useful — it shows you all the columns, their types, indexes, and constraints. It's like reading the blueprint of a table.

🧪 **Try It Yourself**

Open `psql` and create a table with at least four different data types — use TEXT, INTEGER, BOOLEAN, and TIMESTAMPTZ. Insert three rows with different values. Then write a `SELECT` query that filters on the BOOLEAN column. Try using `UPDATE` to change one row's boolean value. Finally, `DELETE` a row and verify it's gone. Get comfortable with the basic verbs before moving on.

After that, try creating a second table with a foreign key to the first one. Insert some rows and use a JOIN to query data from both tables at once. For example:

```sql
CREATE TABLE chapters (
    id SERIAL PRIMARY KEY,
    fic_id TEXT REFERENCES fic_info(id),
    number INT NOT NULL,
    title TEXT
);

INSERT INTO chapters (fic_id, number, title)
VALUES ('abc123', 1, 'The Beginning'), ('abc123', 2, 'The Middle');

-- Join to get fic info with chapter info
SELECT fic_info.title, chapters.number, chapters.title
FROM fic_info
JOIN chapters ON fic_info.id = chapters.fic_id;
```

This is exactly the kind of relationship our database uses to link request logs to sources.

---

# Chapter 14: SQLx Basics

## What Is SQLx?

Now we know SQL. But we're writing Rust, not SQL. How do we run SQL from Rust code?

There are several options in the Rust ecosystem:

- `postgres` crate — raw SQL, no compile-time checks. If your SQL has a typo, you won't know until runtime.
- `diesel` — a full ORM (Object-Relational Mapper) with lots of macros. Powerful but steep learning curve.
- **`sqlx`** — the sweet spot: real SQL with compile-time checking.

SQLx lets you write SQL as regular strings in your Rust code, and then checks them against a real database at compile time. If your SQL has a typo, you'll get a compile error instead of a runtime crash.

> 💡 **Key Concept**
>
> "Compile-time checked" means the Rust compiler actually connects to your database and verifies your SQL is valid before the program runs. If the table doesn't exist or a column name is wrong, you find out immediately — not in production at 3am when a customer reports an error. This is one of SQLx's best features.

## Adding SQLx to Cargo.toml

Open your `Cargo.toml` and add these dependencies:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "chrono"] }
```

Let's break down those features:

- `runtime-tokio` — tells SQLx to use the Tokio async runtime (which Axum also uses). SQLx supports other runtimes too, but Tokio is what we're using.
- `postgres` — enables PostgreSQL support. If you wanted MySQL instead, you'd use `"mysql"`.
- `chrono` — lets us work with dates and times using the `chrono` crate. This is important because several of our database columns are timestamps.

After adding this, run:

```bash
cargo build
```

This will download and compile SQLx and all its dependencies. It might take a few minutes the first time — SQLx has quite a few transitive dependencies.

> ⚠️ **Watch Out**
>
> If you see errors about missing features, double-check that you've included all three features: `"runtime-tokio"`, `"postgres"`, and `"chrono"`. Forgetting a feature is one of the most common gotcha moments with SQLx.

## The Connection Pool

Here's a question: should we open a new database connection for every HTTP request?

No! Opening connections is expensive. It involves a network handshake, authentication, and resource allocation on the server. If 100 users hit our server at once, we'd open 100 connections, which would slow everything down.

Instead, we use a **connection pool**. A pool keeps a bunch of connections ready to go. When a request comes in, it borrows a connection from the pool, uses it, and returns it when done. It's like a library — you don't build a new library every time someone wants to read a book. You have one library with many copies of popular books, and people borrow and return them.

In SQLx, the pool type is `PgPool`:

```rust
use sqlx::PgPool;

let pool: PgPool = PgPool::connect("postgresql://fichub@localhost/fichub")
    .await?;
```

That's it! One line to get a pool. Under the hood, SQLx creates multiple connections and manages them automatically. The `PgPool` is `Send + Sync`, so you can share it across async tasks and threads safely.

## Controlling Pool Size

The default pool size is 10 connections. For most development, that's fine. But in production, you might want more control:

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

let pool = PgPoolOptions::new()
    .max_connections(20)           // Allow up to 20 simultaneous connections
    .acquire_timeout(Duration::from_secs(10))  // Wait up to 10 seconds for a connection
    .connect("postgresql://fichub@localhost/fichub")
    .await?;
```

- `max_connections(20)` — if all 20 connections are busy and another request comes in, it will wait up to `acquire_timeout` seconds for one to become available. If no connection frees up in time, the request fails.
- `acquire_timeout(Duration::from_secs(10))` — how long to wait for a connection before giving up.

This is exactly what FicHub does in `src/db/mod.rs`:

```rust
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
    // ... run migrations ...
    Ok(pool)
}
```

> ⚠️ **Watch Out**
>
> The `PgPool` is cheap to clone. It's actually just a reference-counted pointer to the shared pool. So when you pass it to handlers, you can clone it freely: `pool.clone()`. Don't try to create multiple pools for the same database — use one pool shared across your whole application.

## Running a Simple Query

Let's run our first SQL query from Rust:

```rust
use sqlx::PgPool;

async fn greet_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT 1 + 1")
        .fetch_one(pool)
        .await?;

    println!("The answer is: {}", row.0);
    Ok(())
}
```

Here's what's happening, step by step:

1. `sqlx::query_as("SELECT 1 + 1")` — creates a query that will be deserialized into a Rust type. The SQL is `SELECT 1 + 1`, which PostgreSQL evaluates to `2`.

2. `.fetch_one(pool)` — runs the query against the pool and gets exactly one row. If the query returns zero rows or more than one row, this returns an error.

3. `row.0` — accesses the first (and only) field of the tuple. The result is `(2,)` — a tuple with one element.

The type annotation `(i64,)` tells SQLx what Rust type to expect. Since `1 + 1` returns an integer in PostgreSQL, we say `i64`. The trailing comma is required for single-element tuples in Rust.

## Fetching Multiple Rows

What if the query returns more than one row?

```rust
async fn get_all_fics(pool: &PgPool) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT id, title FROM fic_info")
        .fetch_all(pool)
        .await?;

    for (id, title) in &rows {
        println!("{}: {}", id, title);
    }
    Ok(())
}
```

The difference is `fetch_all` instead of `fetch_one`. It returns a `Vec` of all matching rows. Each row is a `(String, String)` tuple because we selected two TEXT columns.

## Fetching Optional Results

Sometimes a query might return zero rows, and that's okay — it's not an error. For example, looking up a fic by ID might not find anything:

```rust
async fn find_fic(pool: &PgPool, id: &str) -> Result<Option<(String, i32)>, sqlx::Error> {
    let row = sqlx::query_as("SELECT title, words FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some((title, words)) => println!("Found: {} ({} words)", title, words),
        None => println!("No fic found with id: {}", id),
    }
    Ok(row)
}
```

`fetch_optional` returns `Option<Row>` — `Some(row)` if a match was found, `None` if not.

## Query Methods Summary

SQLx gives you several ways to run queries:

| Method | Returns | Use When |
|--------|---------|----------|
| `fetch_one(pool)` | Exactly one row | You expect exactly one result (error if zero or multiple) |
| `fetch_optional(pool)` | `Option<Row>` | There might be zero or one result |
| `fetch_all(pool)` | `Vec<Row>` | Zero or more results |
| `execute(pool)` | `QueryResult` | You don't need the row data (INSERT, UPDATE, DELETE) |

The `QueryResult` from `execute()` tells you how many rows were affected, which is useful for knowing if an UPDATE or DELETE actually changed anything.

## Passing Parameters

We've already seen `$1` parameters in action. Let's look at it more carefully:

```rust
async fn get_fics_by_author(pool: &PgPool, author: &str) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, title FROM fic_info WHERE author = $1"
    )
    .bind(author)  // $1 = author
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

The `.bind()` calls fill in the `$N` parameters in order. You can have as many as you need:

```rust
let rows = sqlx::query_as::<_, (String,)>(
    "SELECT title FROM fic_info WHERE author = $1 AND words > $2 AND status = $3"
)
.bind("CoolAuthor")        // $1
.bind(50000_i64)           // $2
.bind("complete")          // $3
.fetch_all(pool)
.await?;
```

> 💡 **Key Concept**
>
> Parameterized queries ($1, $2, $3) prevent SQL injection attacks. The database treats your parameters as data, never as SQL commands. If someone passes `'; DROP TABLE fic_info; --` as an author name, it gets stored literally as that string — it doesn't get executed as SQL. This is incredibly important for security.

🧪 **Try It Yourself**

Create a simple Rust program that connects to PostgreSQL, runs `SELECT NOW()`, and prints the current database time. Then try running a query that returns multiple rows — maybe `SELECT generate_series(1, 10)` which creates numbers 1 through 10. Try changing the type annotation to something wrong (like `(String,)` instead of `(i64,)`) and see what the compile-time error looks like.

## Error Handling with SQLx

SQLx functions return `Result<T, sqlx::Error>`. The `sqlx::Error` enum has several variants:

| Variant | What it means |
|---------|---------------|
| `ConnectionRefused` | Couldn't connect to the database |
| `RowNotFound` | `fetch_one` was called but no rows matched |
| `TypeCastError` | A column's SQL type doesn't match the Rust type |
| `DatabaseError` | The database returned an error (bad SQL, constraint violation, etc.) |

In FicHub, we wrap `sqlx::Error` in our own `AppError` type using the `?` operator:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;  // sqlx::Error is converted to AppError here
    Ok(row)
}
```

The `?` operator at the end of `.await?` automatically converts `sqlx::Error` into our `AppError`. This means every database query in FicHub can fail gracefully — the error propagates up through the call stack, and the middleware eventually sends a proper error response to the client.

> 💡 **Key Concept**
>
> The `?` operator is Rust's way of handling errors concisely. Instead of writing `match` statements for every possible error, `?` either returns the value on success or propagates the error up the call stack. It's like saying "do this, and if it fails, pass the error to whoever called me."

## Connecting at Startup

The standard pattern is to create the pool once at application startup and share it with all handlers. In FicHub, the pool is stored in `AppState`:

```rust
// In main.rs or the initialization code
let pool = init_pool(&database_url).await?;

// Create the shared state
let state = Arc::new(AppState {
    db: pool,          // PgPool is cloned cheaply
    http_client: client,
    config: config,
    // ...
});

// Pass state to the router
let app = Router::new()
    .route("/api/v0/epub", get(export::epub_handler))
    .with_state(state);
```

Each handler receives the pool through the `State` extractor:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // Use state.db anywhere you need database access
    let fic = queries::get_fic_info(&state.db, "some-id").await?;
    // ...
}
```

This pattern — create once, share everywhere — is the standard for Axum applications. The `PgPool` is reference-counted internally, so cloning it is cheap (just increments a counter).

> ⚠️ **Watch Out**
>
> Don't create a new `PgPool` in every handler! That would create a new connection pool for every request, which defeats the purpose of connection pooling. Always create the pool once at startup and share it through `AppState`.

## Common SQLx Pitfalls

Before we move on, here are some common mistakes people make with SQLx:

1. **Mismatched field names** — If your Rust struct has `author_name` but the SQL column is `author`, you'll get a runtime error. Always double-check that field names match exactly.

2. **Wrong type annotations** — If a SQL column is `INT8` but you declared the Rust field as `i32`, you'll get a `TypeCastError`. Use `i64` for `INT8` and `i32` for `INT4`.

3. **Missing features** — Forgetting to enable `"chrono"` in Cargo.toml means you can't use `DateTime<Utc>` types. The error message from SQLx isn't always obvious.

4. **Not using `fetch_optional`** — When a query might return zero rows, use `fetch_optional` instead of `fetch_one`. `fetch_one` returns `RowNotFound` if no rows match, which is usually not what you want.

5. **Forgetting `.await`** — All SQLx query methods are async. If you forget `.await`, you'll get a compiler error about an unsatisfied future.

These are all things you'll learn through practice, but it's good to know them upfront so you don't waste time debugging confusing errors.

---

# Chapter 15: Migrations

## What Are Migrations?

Imagine you build version 1 of your database schema. Everything works. Then you realize you need to add a new column to store the fic's language. Do you just alter the database by hand? What if you need to do this on your development machine, the testing server, AND the production server?

**Migrations** solve this problem. A migration is a file with SQL commands that transform your database from one version to the next. You write the SQL, give it a number, and then run it.

The key principles of good migrations:

1. **Each migration is numbered and ordered.** Migration 001 runs before 002, which runs before 003.

2. **Each migration should be idempotent.** That means safe to run multiple times without breaking anything. If a table already exists, the migration shouldn't error — it should just do nothing.

3. **Migrations are one-way.** You move forward through versions, not backward. If you make a mistake, you write a new migration to fix it.

4. **Migrations are version-controlled.** They live in your git repository, so everyone on the team uses the same schema.

The standard pattern is:

```
migrations/
├── 001_initial_schema.sql      ← create all your tables
├── 002_add_recommender.sql     ← add the recommendation engine
└── 003_add_tagging.sql         ← add the tagging system
```

## The Migrations Directory in FicHub

In FicHub, the migrations live at the project root:

```
fichub/
├── Cargo.toml
├── migrations/
│   ├── 001_initial_schema.sql
│   ├── 002_recommender.sql
│   └── 003_tagging.sql
├── src/
│   ├── main.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models.rs
│   │   └── queries.rs
│   └── ...
```

The naming convention is simple: a three-digit number, an underscore, a descriptive name, and `.sql`. SQLx uses the number to determine the order to run migrations.

## Creating the Initial Schema

Let's look at FicHub's first migration. This is the foundation of everything — it creates all the core tables:

```sql
-- FicHub database schema
-- Migration 001: Initial schema

-- Request source tracking
CREATE TABLE IF NOT EXISTS request_source (
    id BIGSERIAL PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT,
    UNIQUE(is_automated, route, description)
);
```

This is the foundation of everything — it creates all the core tables. Let's break down each line:

- `CREATE TABLE IF NOT EXISTS` — the `IF NOT EXISTS` makes this idempotent. If the table already exists, it's a no-op.
- `id BIGSERIAL PRIMARY KEY` — an auto-incrementing 8-byte integer. PostgreSQL will automatically assign 1, 2, 3... and never repeat. `BIGSERIAL` means it can grow very large.
- `created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP` — when this row was created. The `DEFAULT` means if you don't provide a value, it uses the current time automatically.
- `is_automated BOOLEAN DEFAULT FALSE` — is this a bot request? Defaults to false.
- `route TEXT` — which API endpoint was called (like `/api/v0/epub`).
- `UNIQUE(is_automated, route, description)` — this is a **composite unique constraint**. It means you can't have two rows with the same combination of all three fields. But you CAN have multiple rows with the same `route` if the `is_automated` flag or `description` is different.

The `BIGSERIAL` type is PostgreSQL-specific. It creates a sequence (an auto-incrementing counter) and binds it to the column. In standard SQL, you'd write `GENERATED ALWAYS AS IDENTITY`, but `BIGSERIAL` is more convenient for PostgreSQL.

Now the request log:

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

The key line here is `source_id BIGINT REFERENCES request_source(id)`. This is a **foreign key** — it links each log entry back to a source. The `REFERENCES` clause means you can't log a request for a source that doesn't exist. PostgreSQL enforces this: if you try to insert a row with a `source_id` that doesn't exist in `request_source`, the database rejects it.

We also create indexes to speed up common queries:

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);

CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

Indexes are like the index in the back of a book. Instead of reading every page to find what you want, you look up the page number in the index. Database indexes work the same way — they create a sorted lookup structure that makes queries much faster.

The second index is a **partial index** — it only indexes rows where an EPUB export was done. This saves space and speeds up those specific queries because the index is smaller.

### The fic_info Table

This is the heart of our database — it stores metadata about every fanfic we've seen. It's the most important table in the system because almost everything else connects back to it:

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
    content_hash VARCHAR(256)
);
```

Notice a few things:

- `id VARCHAR(128)` — the fic's unique identifier is a string, not a number, because different fanfiction sites have different ID formats. AO3 uses numeric IDs like `123456`, while FFN uses different formats. The `VARCHAR(128)` gives us enough room for any ID format.
- `words INT8 NOT NULL` — word counts can be huge (some serial fics have millions of words), so we use 8-byte integers.
- `created` vs `updated` — `created` is when we first saw this fic, `updated` is when the metadata was last refreshed. These are managed by the database (via `DEFAULT CURRENT_TIMESTAMP` and `NOW()` in the upsert query).
- `fic_created` vs `fic_updated` — these are the dates from the source site, not our database. A fic might have been published in 2020 but we only discovered it in 2024.
- `content_hash VARCHAR(256)` — a hash of the fic's content, used to detect when a fic has been updated on the source site. If the hash changes, the fic has been modified.
- `source_id INT8` and `author_id INT8` — numeric IDs from the source site (like AO3's work ID and author ID). These are stored as `Option<i64>` in Rust because not all source sites provide them.

The `description` column stores the fic's summary as-is, including any HTML formatting. This is stored in the API response as `raw_extended_meta` when available.

### The export_log Table

This table tracks which files we've already generated:

```sql
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

The `UNIQUE` constraint is key: for a given fic, version, format type, and input hash, there can only be one export. This is how caching works — if we've already generated this exact export, we skip the expensive generation step.

### The Blacklist Tables

```sql
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);

CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);
```

### The Version Bump Table

```sql
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT
);
```

We'll learn how all these tables are used in later chapters.

## Running Migrations

### With the SQLx CLI

The easiest way to run migrations is with the SQLx command-line tool:

```bash
# Install the CLI (only needed once)
cargo install sqlx-cli

# Run all pending migrations
sqlx migrate run

# Check which migrations have been applied
sqlx migrate info

# Revert the last migration (if the migration has a down.sql file)
sqlx migrate revert
```

### Programmatically in Rust

FicHub runs migrations automatically when the server starts. Here's how, using the real code from `src/db/mod.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use std::path::Path;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    // Create the connection pool
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Find the migrations directory
    // First try next to the binary, then fall back to CARGO_MANIFEST_DIR
    let migrations_path = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            exe_dir.join("migrations")
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
        }
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")
    };

    // Run all pending migrations
    if migrations_path.exists() {
        Migrator::new(migrations_path)
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

This is smart about finding the migrations directory. During development, `CARGO_MANIFEST_DIR` points to the project root. In production, when the binary is compiled and moved, the migrations are next to the binary.

This is brilliant because:

1. It finds the `migrations/` directory automatically
2. It runs every migration that hasn't been run yet
3. It records which migrations have been applied in an internal `_sqlx_migrations` table
4. It does all this when the server starts, so the database is always up to date
5. It's safe to run multiple times — only unapplied migrations are executed

> ⚠️ **Watch Out**
>
> Migrations are not reversible by default. If you create a table in migration 001, you can't "un-run" it to drop the table. This is intentional — reversibility is hard to get right (what if the table has data?). If you make a mistake, write a new migration to fix it.

> 💡 **Key Concept**
>
> The `IF NOT EXISTS` clause in `CREATE TABLE` makes migrations idempotent. Running the same migration twice won't cause errors — it just does nothing the second time. This is essential for safety. If the server crashes during migration and restarts, the migration can run again without breaking anything.

🧪 **Try It Yourself**

Create a new migration file called `004_experiments.sql` in the `migrations/` directory. Create a table called `experiments` with at least three columns of different types. Add an index on one column. Make sure to use `IF NOT EXISTS`. Then run `sqlx migrate run` and verify the table exists with `psql -c "\d experiments"`.

---

# Chapter 16: Models and Queries

## From Tables to Structs

We have tables in the database. We have structs in Rust. Now we need to connect them. SQLx makes this easy with the `FromRow` derive macro — but the struct fields must match the database column names exactly.

## Creating the FicInfo Model

Remember the `fic_info` table from the migration? Let's create a Rust struct that matches it:

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

Let's look at each derive:

- `Debug` — lets us print the struct with `{:?}` for debugging. Essential during development.
- `Clone` — lets us duplicate the struct. Useful when you need to pass it to multiple functions.
- `Serialize` — lets us convert it to JSON (using `serde_json`). We need this to send API responses.
- `Deserialize` — lets us convert from JSON. We need this for config files and testing.
- **`FromRow`** — the magic one! SQLx uses this to convert a database row into this struct automatically.

Notice the `Option<>` types. In the database, columns like `author_url` can be `NULL` (empty). In Rust, `NULL` maps to `Option::None`. Columns marked `NOT NULL` in SQL map to plain types in Rust (like `String` and `i32`).

The mapping between SQL types and Rust types:

| SQL Type | Rust Type |
|----------|-----------|
| `TEXT` / `VARCHAR` | `String` |
| `INT4` | `i32` |
| `INT8` | `i64` |
| `BOOLEAN` | `bool` |
| `TIMESTAMPTZ` | `DateTime<Utc>` |
| `REAL` | `f32` |
| `*` (nullable) | `Option<T>` |

> 💡 **Key Concept**
>
> The field names in your Rust struct must match the column names in your database table. If your SQL column is `author_url`, your Rust field must be `author_url`. SQLx uses these names to map database rows to struct fields. If they don't match, you'll get a runtime error (not a compile-time error, unfortunately).

## Upserting: The INSERT ON CONFLICT Pattern

One of the most important patterns in FicHub is the **upsert** — an INSERT that gracefully handles conflicts. Here's the scenario:

1. Someone requests a fic we've never seen before → **INSERT** it
2. Someone requests the same fic again with updated metadata → **UPDATE** it

We could check if the fic exists first, then insert or update based on the result. But that's two database queries. Instead, PostgreSQL gives us a single-query solution with `ON CONFLICT`:

```rust
use sqlx::PgPool;
use crate::db::models::FicInfo;
use crate::error::AppResult;

pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_info (
            id, title, author, author_url, author_local_id,
            chapters, words, description, fic_created, fic_updated,
            status, source, extra_meta, raw_extended_meta,
            source_id, author_id, content_hash, updated
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                  $11, $12, $13, $14, $15, $16, $17, NOW())
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

This is a big function, so let's break it down carefully:

**The INSERT part** (`INSERT INTO fic_info ... VALUES ($1, $2, ...)`):
- `$1`, `$2`, etc. are **parameters**. They get replaced with the values you `.bind()` later.
- This is called **parameterized queries** — the database never sees your raw strings, which prevents SQL injection attacks.
- We have 17 parameters (`$1` through `$17`) plus `NOW()` which is a PostgreSQL built-in.

**The ON CONFLICT part** (`ON CONFLICT (id) DO UPDATE SET ...`):
- If a row with the same `id` already exists, don't error — instead, update the existing row.
- `EXCLUDED.title` means "the value that was going to be inserted." So `title = EXCLUDED.title` says "set the title to the new value."
- We update most columns but not all — `created` stays unchanged (it's set once when the fic is first inserted), and `source` stays unchanged (it shouldn't change for a given fic).

**The `.bind()` chain**:
- Each `.bind()` fills in the next `$N` parameter. The order matters!
- `&fic.id` passes a reference to the string (because strings are borrowed). `fic.chapters` passes the integer by value (because integers are `Copy`).

**The `NOW()` calls**:
- `NOW()` is a PostgreSQL function that returns the current timestamp.
- We use it for the `updated` column so we know when the record was last modified.
- Notice we don't set `created` in the UPDATE part — it keeps the original value.

> ⚠️ **Watch Out**
>
> The number of `.bind()` calls must exactly match the number of `$N` parameters in your SQL. If you have `$17` in the SQL but only 16 `.bind()` calls, you'll get a compile-time error from SQLx. This is actually a good thing — it catches bugs early! Always count your parameters.

> 💡 **Key Concept**
>
> Why use `r#"..."#` (Rust raw strings) for SQL? Raw strings don't interpret escape sequences like `\n` or `\t`. This means you can write multi-line SQL without worrying about escaping. The `#` characters are just delimiters — you could use any character that doesn't appear in your SQL.

## Getting a Fic by ID

Now let's read data back from the database:

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

A few things to note:

1. `sqlx::query_as::<_, FicInfo>` — the `<_, FicInfo>` is **turbofish syntax**. It tells SQLx "deserialize the result into a `FicInfo` struct." The `_` means "figure out the row type yourself."

2. `"SELECT * FROM fic_info WHERE id = $1"` — we use `*` to select all columns. This works because our `FicInfo` struct has fields matching all the columns. If you only need some columns, write them out explicitly — it's faster and less fragile.

3. `.fetch_optional(pool)` — returns `Option<FicInfo>`. If no fic has that ID, you get `None`. This is much better than `.fetch_one()`, which would error if no row was found.

4. The return type is `AppResult<Option<FicInfo>>` — a result that contains an optional fic. Two layers of wrapping: `Result` for database errors, `Option` for "no fic found."

## query vs query_as

SQLx has two main query functions:

- `sqlx::query()` — returns raw rows that you access by column index or name
- `sqlx::query_as()` — automatically deserializes into a struct or tuple

Use `query_as` when you have a matching struct. Use `query` when you're doing something simple or getting a single value:

```rust
// Getting a single value (no struct needed)
let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
    .fetch_one(pool)
    .await?;
println!("Total fics: {}", count.0);

// Getting multiple rows as tuples
let titles: Vec<(String, i32)> = sqlx::query_as(
    "SELECT title, chapters FROM fic_info"
)
.fetch_all(pool)
.await?;

for (title, chapters) in &titles {
    println!("{}: {} chapters", title, chapters);
}
```

## The Search Query

FicHub has a "did you mean?" feature that searches for similar fics:

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

This introduces several new concepts:

- `ILIKE` — PostgreSQL's case-insensitive version of `LIKE`. `LIKE` is case-sensitive ('CoolAuthor' ≠ 'coolauthor'), while `ILIKE` treats them the same.
- The `%` wildcard — `%best%` matches any string containing "best" anywhere. So "The Best Fic Ever" matches, but "BestThe Fic" also matches.
- `LIMIT 5` — don't return more than 5 results. Without this, a search for "a" could return thousands of fics.
- `format!("%{}%", query)` — we build the pattern in Rust, not SQL. This keeps the SQL template simple and avoids string concatenation in the query itself.

> ⚠️ **Watch Out**
>
> The `format!("%{}%", query)` approach is safe here because `query` is bound as a parameter. But if you were building the SQL string directly (like `format!("SELECT * FROM fic_info WHERE title LIKE '%{}%'", query)`), you'd have a SQL injection vulnerability! Always use parameterized queries.

## Handling the Export Handler

Let's see how these queries come together in the real export handler. This is the core flow of FicHub:

```rust
// 1. Scrape metadata from the source site
let meta = scraper.lookup(&state.http_client, query).await?;

// 2. Build a FicInfo struct from the scraped metadata
let fic_info_row = FicInfo {
    id: meta.url_id.clone(),
    created: None,        // Will be set by the database DEFAULT
    updated: None,        // Will be set by the database DEFAULT
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

// 3. Upsert it into the database
queries::upsert_fic_info(&state.db, &fic_info_row).await?;

// 4. Later, look it up
let existing = queries::get_fic_info(&state.db, &meta.url_id).await?;
```

This is the fundamental cycle: **scrape → store → retrieve**. The database acts as a persistent cache of metadata we've already seen. The first time someone requests a fic, we scrape it and store it. The next time, the metadata is already in the database.

🧪 **Try It Yourself**

1. Add a `search_fics_by_author` function that takes an author name and returns all their fics.
2. Add a `get_fic_count` function that returns the total number of fics in the database.
3. Try using `sqlx::query` (without `query_as`) to select just the `title` column and print it. Hint: use `fetch_all` and access columns with `.get("title")`.

---

# Chapter 17: Request Logging

## Why Log Requests?

Every time someone asks FicHub to download a fic, we want to record that it happened. Why?

1. **Analytics** — How many requests per day? Which sites are most popular? What are peak hours?
2. **Debugging** — When something goes wrong, logs help us figure out what happened and when.
3. **Rate limiting** — We need to track how many requests each source has made to prevent abuse.
4. **Performance monitoring** — How long do metadata lookups take? How long do exports take?

FicHub has two tables for this: `request_source` (who is asking) and `request_log` (what they asked and what happened).

## The Request Source Table

A "source" is something that makes requests. It could be a web browser, a mobile app, or a bot. We track three things about each source:

- Is it automated?
- Which route did it call?
- A human-readable description

Here's the Rust model:

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

## Inserting a Request Source (UPSERT with RETURNING)

The `insert_request_source` function uses a powerful combination of UPSERT and RETURNING:

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

This is a great example of combining several SQL features:

1. **UPSERT** — if a source with the same `(is_automated, route, description)` already exists, don't create a duplicate. Just update the route (which is effectively a no-op, but it satisfies the `DO UPDATE SET` requirement — PostgreSQL requires at least one column in the SET clause).

2. **RETURNING id** — instead of throwing away the result, give us back the `id` of the row (whether it was just created or already existed). This is much better than doing a separate SELECT after the INSERT.

3. **`(i64,)` tuple type** — since `RETURNING id` returns a single column, we use a tuple with one element. The `.0` accesses that element.

This pattern is called "get or create" — we want the ID of a source, and we want to create it if it doesn't exist. It's one of the most common database patterns in web applications.

## The Request Log Table

Each time someone makes a successful request, we log all the details:

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
           (source_id, etype, query, info_request_ms, url_id,
            fic_info, export_ms, export_file_name, export_file_hash, url)
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

Let's look at the parameters in detail:

- `source_id` — links to `request_source` via foreign key. This is how we know WHO made the request.
- `etype` — the export type, like `"epub"` or `"html"`. Different endpoints produce different types.
- `query` — the original URL the user asked for (like `https://archiveofourown.org/works/123456`).
- `info_request_ms` — how long the metadata lookup took, in milliseconds. This measures network performance.
- `url_id` — the fic's unique ID after lookup. This might be `None` if the lookup failed.
- `fic_info` — a JSON blob of the full metadata. Stored as TEXT (JSON-as-string) for flexibility.
- `export_ms` — how long the export generation took. This measures CPU performance.
- `export_file_name` — like `"abc123.epub"`. Useful for finding files on disk.
- `export_file_hash` — hash of the generated file. Used for cache lookup.
- `url` — the original URL again (yes, it's duplicated — but it's useful for analytics queries).

Notice the `Option<&str>` parameters. These represent nullable database columns. In Rust, we pass `Some("value")` for present values and `None` for NULL. SQLx handles the conversion automatically.

## Building the Info String

When FicHub returns results, it includes a human-readable info string that tells the user about the fic. Here's how it's built:

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
        date = chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        relative = relative_time,
    );

    (info, Vec::new())
}
```

This produces something like:

```
The Best Fic Ever by CoolAuthor
50000 words in 10 chapters
Status: complete
Updated: 2024-01-15 10:30:00 - 3 days ago
```

The function returns a tuple of `(String, Vec<String>)` — the info string and a list of notes (warnings, greylist messages, etc.). The notes vector is empty in the normal case but gets populated for special situations like greylisting.

The relative time calculation is a nice touch — "3 days ago" is more human-friendly than just a raw timestamp.

## Putting It All Together

In the export handler, logging happens at the end of a successful request:

```rust
// Get or create a request source
let source_id = queries::insert_request_source(
    &state.db, false, "/api/v0/epub", "web request",
).await?;

// Build the fic_info JSON for logging
let fic_json = serde_json::to_string(&meta).ok();

// Log the request with all details
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;
```

The pattern is:
1. Get the source ID (creating the source if it doesn't exist)
2. Serialize the metadata to JSON
3. Log the full request with all details — timing, file info, and metadata

The `.ok()` on `serde_json::to_string` converts a `Result` to an `Option` — if serialization fails (unlikely), we just log `None` for the fic_info field instead of crashing.

> ⚠️ **Watch Out**
>
> Notice that logging happens AFTER the export succeeds. If the export fails, we don't log it. This means our analytics only count successful requests. If you want to track failures too, you'd need to add error logging in the error path as well. For a production system, you'd probably want both.

> 💡 **Key Concept**
>
> The `info_request_ms` and `export_ms` fields give us performance data without any extra work. Over time, we can query these to find slow requests, identify bottlenecks, and track performance trends. This is the kind of observability that makes debugging production issues possible.

🧪 **Try It Yourself**

1. Write a function `get_recent_requests(pool, limit)` that returns the N most recent request log entries. Use `ORDER BY created DESC LIMIT $1`.
2. Write a function `get_requests_by_type(pool, etype)` that returns all requests of a given export type.
3. Think about: what SQL query would show you the most popular fics (by request count)? Hint: you'll need a `GROUP BY` and `COUNT(*)`.

## Querying Logs for Analytics

Once you have request logs, you can answer interesting questions about your system. Here are some useful queries:

```sql
-- How many requests per day in the last 30 days?
SELECT DATE(created) as day, COUNT(*) as requests
FROM request_log
WHERE created > NOW() - INTERVAL '30 days'
GROUP BY day
ORDER BY day;

-- Average metadata lookup time
SELECT AVG(info_request_ms) as avg_ms, MAX(info_request_ms) as max_ms
FROM request_log
WHERE etype = 'epub';

-- Most requested fics
SELECT url_id, COUNT(*) as request_count
FROM request_log
WHERE url_id IS NOT NULL
GROUP BY url_id
ORDER BY request_count DESC
LIMIT 10;

-- Requests by source (automated vs manual)
SELECT rs.is_automated, COUNT(*) as requests
FROM request_log rl
JOIN request_source rs ON rl.source_id = rs.id
GROUP BY rs.is_automated;
```

These kinds of queries are what make request logging valuable. Without logs, you're flying blind — you don't know which fics are popular, whether the system is slow, or if bots are hammering your endpoints.

The `request_ms` fields are especially useful. If you notice that `info_request_ms` is climbing over time, it might mean the source site is throttling you. If `export_ms` is high, it might mean your EPUB generation is slow and needs optimization.

> 💡 **Key Concept**
>
> The request log is also useful for debugging individual problems. If a user reports that a download failed, you can look up their request in the log: `SELECT * FROM request_log WHERE url_id = 'the-fic-id' ORDER BY created DESC`. This shows you exactly what happened — the timing, the source, whether the export succeeded.

---

# Chapter 18: Blacklists

## What Is a Blacklist?

Sometimes a fanfic needs to be blocked from FicHub. Maybe it's against a site's terms of service, or it contains content that shouldn't be downloaded. A **blacklist** is a list of items that are blocked.

FicHub has two kinds of blacklists:

1. **Fic blacklist** — blocks a specific fanfic by its `url_id`
2. **Author blacklist** — blocks ALL works by a specific author

These serve different purposes. A fic blacklist might be used for a DMCA takedown of one specific story. An author blacklist might be used when an author has requested their works not be archived.

## The Fic Blacklist Table

```sql
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);
```

Each blacklist entry has a **reason code** — a number that explains why it was blocked:

| Reason Code | Meaning |
|-------------|---------|
| 1 | General block |
| 5 | Hard block (admin request) |
| 6 | **Greylist** — show metadata, no download |
| 7 | DMCA takedown |
| 8 | Content policy violation |

The `UNIQUE(url_id, reason)` constraint means you can't blacklist the same fic twice for the same reason. But you could blacklist it for reason 5 AND reason 6 (which would be weird, but the schema allows it).

## The Author Blacklist Table

```sql
CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);
```

This blocks all works by an author on a specific site. The `source_id` specifies which site (1 = AO3, 2 = FFN, etc.) and `author_id` identifies the author on that site.

## The Rust Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicBlacklist {
    pub url_id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthorBlacklist {
    pub source_id: i64,
    pub author_id: i64,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}
```

These are straightforward structs that map directly to the database tables.

## Checking the Blacklists

```rust
pub async fn check_fic_blacklist(
    pool: &PgPool,
    url_id: &str,
) -> AppResult<Vec<FicBlacklist>> {
    let rows = sqlx::query_as::<_, FicBlacklist>(
        "SELECT * FROM fic_blacklist WHERE url_id = $1",
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn check_author_blacklist(
    pool: &PgPool,
    source_id: i64,
    author_id: i64,
) -> AppResult<Vec<AuthorBlacklist>> {
    let rows = sqlx::query_as::<_, AuthorBlacklist>(
        "SELECT * FROM author_blacklist WHERE source_id = $1 AND author_id = $2",
    )
    .bind(source_id)
    .bind(author_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
```

Both functions return `Vec` because a fic or author could have multiple blacklist entries (with different reasons). We check if the vector is empty — if it's not, the fic or author is blacklisted.

We use `fetch_all` instead of `fetch_optional` because we want to see ALL the blacklist reasons, not just the first one. This matters for the greylist logic below.

## Why Check Fic Blacklist Before Author Blacklist?

You might notice in the export handler that we check the fic blacklist first, then the author blacklist. This order matters for two reasons:

1. **Specificity** — a fic blacklist is more specific. If a single fic is blocked but the author isn't, we want to handle that case before checking the broader author blacklist.

2. **Performance** — the fic blacklist check is a single lookup by `url_id`. The author blacklist check requires knowing the `source_id` and `author_id`, which we already have from the metadata. But the fic check can short-circuit earlier.

If a fic is blacklisted AND its author is blacklisted, the fic blacklist response takes precedence (greylist or hard block). The author blacklist would only matter for fics that aren't individually blacklisted.

> 💡 **Key Concept**
>
> The blacklist check happens AFTER we scrape metadata but BEFORE we generate the export. This is the right place because:
> - We need the metadata to know if the fic is blacklisted
> - We don't want to waste time generating an export for a blocked fic
> - We DO store the metadata (upsert happens before blacklist check) so we can track requests even for blocked fics

## Greylisting: Reason Code 6

The most interesting blacklist feature is **greylisting** (reason code 6). Here's how it works:

- A **hard blacklist** (reason 5, 7, or 8) completely blocks the fic. The user gets an error message.
- A **greylist** (reason 6) is softer. It shows the fic's metadata (title, author, word count, etc.) but doesn't provide download links.

The greylist is a compromise. It lets people see that a fic exists and read its description, without being able to download it. This is useful for fics that are controversial but not outright prohibited.

In the export handler, the logic looks like this:

```rust
// Check fic blacklist
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    // Greylist (reason 6) shows metadata but no download
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        return Ok(build_metadata_response(
            &meta, &[], &state.config.export_version, None, true,
        ));
    }
    // Hard blacklist
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({
            "err": -7,
            "msg": "fic is blacklisted",
            "q": query
        })));
    }
}

// Check author blacklist
let author_blacklist = queries::check_author_blacklist(
    &state.db, meta.source_id, meta.author_id,
).await?;
if !author_blacklist.is_empty() {
    return Ok(Json(json!({
        "err": -7,
        "msg": "author is blacklisted",
        "q": query
    })));
}
```

The key difference between greylisting and hard blocking is in the response. For greylisted fics, we return a metadata-only response:

```rust
fn build_metadata_response(
    meta: &FicMetadata,
    notes: &[String],
    _export_version: &i32,
    _cached: Option<&crate::db::models::ExportLog>,
    is_greylisted: bool,
) -> axum::Json<serde_json::Value> {
    let slug = generate_slug(&meta.title, &meta.url_id);
    let (info_str, _) = build_info_string(meta);

    let mut notes_vec: Vec<String> = if is_greylisted {
        vec!["This fic is greylisted - download links are not available.".to_string()]
    } else {
        Vec::new()
    };
    notes_vec.extend_from_slice(notes);

    Json(json!({
        "err": 0,
        "fixits": [],
        "info": info_str,
        "url_id": meta.url_id,
        "slug": slug,
        "meta": build_meta_json(meta),
        "hashes": {},
        "urls": {},
        "epub_url": null,
        "html_url": null,
        "mobi_url": null,
        "pdf_url": null,
        "notes": notes_vec,
    }))
}
```

The critical difference: `"urls"` is an empty object `{}` and all `_url` fields are `null`. The client sees the metadata (title, author, word count) but has nothing to download. The note "This fic is greylisted" tells the user why.

> 💡 **Key Concept**
>
> Greylisting is a middle ground between allowing and blocking. It's useful for fics that are controversial but not prohibited. The metadata is public knowledge (anyone can see it on the source site); only the export is restricted. This respects the author's wishes while still providing information.

> ⚠️ **Watch Out**
>
> The blacklist check happens AFTER metadata lookup but BEFORE export generation. This means we still scrape the source site for metadata, even for greylisted fics. This is intentional — we need the metadata to show it to the user. But it means greylisted fics still cause network requests to the source site. If you wanted to avoid that, you'd need to cache metadata separately from the blacklist check.

> 💡 **Key Concept**
>
> The blacklist check also happens AFTER upserting the fic_info. This means even blacklisted fics get their metadata stored in the database. This is useful for analytics (we can see how many requests were for blacklisted fics) and for the "did you mean?" feature (searching still works even for blacklisted fics).

🧪 **Try It Yourself**

1. Write a query to list all blacklisted fics with their reason codes: `SELECT url_id, reason FROM fic_blacklist`.
2. Write a query to count how many fics are blacklisted for each reason code: `SELECT reason, COUNT(*) FROM fic_blacklist GROUP BY reason`.
3. Think about: how would you add a new reason code (say, reason 9 for "author request")? Answer: you just start using 9 in the `reason` column — no schema changes needed!

---

# Chapter 19: Cache Queries

## The Cache System

Generating an EPUB from a fanfic is expensive. It involves:

1. Fetching every chapter from the source site (network I/O — could take seconds or minutes)
2. Parsing the HTML content
3. Building the EPUB file (CPU work)
4. Potentially generating other formats (HTML, MOBI, PDF)

We don't want to redo this work every time someone requests the same fic. So we cache the results. The cache system has two parts:

1. **Disk cache** — the actual EPUB/HTML files stored on the filesystem
2. **Export log** — a database record tracking what we've generated and when

The export log is the index; the disk cache is the data.

## The export_log Table

```sql
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

Each row says: "For fic `url_id`, at cache `version`, format `etype`, with input matching `input_hash`, we generated a file with hash `export_hash`."

Let's break down each field:

- `url_id` — which fic this export is for
- `version` — the cache version (more on this below)
- `etype` — the export type: `"epub"`, `"html"`, `"mobi"`, or `"pdf"`
- `input_hash` — a hash of the input data. For EPUBs, this is the fic's `content_hash`. For HTML bundles, it's `"epub:{epub_hash}"` (because HTML is generated from the EPUB).
- `export_hash` — the hash of the generated file. This is used as part of the cache URL.

The `UNIQUE` constraint ensures there's at most one export for each combination. If we try to insert a duplicate, the `ON CONFLICT` clause updates the existing row.

## The ExportLog Model

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

This maps directly to the database row.

## Finding a Cached Export

```rust
pub async fn find_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
) -> AppResult<Option<ExportLog>> {
    let row = sqlx::query_as::<_, ExportLog>(
        r#"SELECT * FROM export_log
           WHERE url_id = $1 AND version = $2 AND etype = $3 AND input_hash = $4"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

This is a cache lookup. We search for an exact match on all four fields:

1. **url_id** — is this the same fic?
2. **version** — is the cache version the same? (If the version was bumped, the old entry won't match.)
3. **etype** — are we looking for the same format?
4. **input_hash** — has the fic's content changed? (If the source site updated the fic, the content hash changes.)

If all four match, we return the `ExportLog` with the `export_hash` — the hash of the file on disk. If any field doesn't match, we return `None` (cache miss) and need to regenerate.

## Recording a New Export

```rust
pub async fn insert_export_log(
    pool: &PgPool,
    url_id: &str,
    version: i32,
    etype: &str,
    input_hash: &str,
    export_hash: &str,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO export_log (url_id, version, etype, input_hash, export_hash)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (url_id, version, etype, input_hash) DO UPDATE SET
               export_hash = EXCLUDED.export_hash,
               created = NOW()"#,
    )
    .bind(url_id)
    .bind(version)
    .bind(etype)
    .bind(input_hash)
    .bind(export_hash)
    .execute(pool)
    .await?;
    Ok(())
}
```

After we generate an EPUB and store it on disk, we record it here. The `ON CONFLICT` clause means if we somehow generate the same export twice (maybe two requests came in at the same time), we just update the existing record instead of erroring.

## The Version Bump System

Here's a subtle but important problem: what if a fanfic gets updated on the source site? The old cached EPUB is outdated, but `export_log` still has a record for it.

The solution is the **version bump** system:

```sql
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT
);
```

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicVersionBump {
    pub id: String,
    pub value: Option<i32>,
}

pub async fn get_fic_version_bump(
    pool: &PgPool,
    url_id: &str,
) -> AppResult<Option<i32>> {
    let row = sqlx::query_as::<_, FicVersionBump>(
        "SELECT * FROM fic_version_bump WHERE id = $1",
    )
    .bind(url_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.value))
}
```

Here's how the version system works:

1. The system has a global `export_version` stored in the config (say, `1`).
2. Each fic can have an optional `version_bump` value in the `fic_version_bump` table.
3. When checking the cache, we compute the effective version: `version = export_version + version_bump`.
4. The old `export_log` entry has `version = 1`. If we bump the version to 2, the old entry doesn't match anymore — it's effectively invalidated.

In the export handler:

```rust
// Compute cache version
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id)
    .await?
    .unwrap_or(0);
let version = state.config.export_version + version_bump;

// Check cache with the effective version
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash,
).await?;
```

This is elegant because:

- **Bumping all fics**: Increase `export_version` in config (e.g., from 1 to 2). Every cache entry becomes stale. Useful for schema changes or export format updates that affect all fics.
- **Bumping one fic**: Insert a `fic_version_bump` row for just that fic. Only that fic's cache is invalidated. Useful when we detect a fic has been updated on the source site.

> 💡 **Key Concept**
>
> The version system combines a global version with a per-fic bump. This gives you two levers: one that affects everything, and one that affects individual fics. It's like having a "nuke all caches" button and a "rebuild this one cache" button.

> ⚠️ **Watch Out**
>
> The `input_hash` in `export_log` is based on the fic's `content_hash`. If the source site doesn't provide content hashes (some don't), we use the string `"upstream"` as the input hash. This means we can't detect content updates for those fics — the cache will persist until the version is manually bumped. This is a tradeoff between freshness and efficiency.

## The Full Cache Flow

Let's trace through a complete cache lookup, step by step:

1. **User requests a fic** by providing a URL.
2. **We scrape metadata** from the source site and get the `content_hash` (which becomes the `input_hash`).
3. **We compute the version**: `version = export_version + version_bump`.
4. **We call `find_export_log`** with the four fields: `(url_id, version, "epub", input_hash)`.
5. **Cache hit**: We get back an `ExportLog` with the `export_hash`. The client downloads the file from `/cache/epub/{url_id}?h={export_hash}`.
6. **Cache miss**: We return `None`, so we need to generate the EPUB.

After the EPUB is generated:

7. **Store on disk**: Move the EPUB to the cache directory with a path based on the hash.
8. **Record in database**: Call `insert_export_log` with all four fields.
9. **Generate other formats**: The HTML bundle is generated FROM the EPUB (not from the source site), so its `input_hash` is `"epub:{epub_hash}"`.

We also check for cached versions of other formats:

```rust
// Check for other formats (HTML, MOBI, PDF) from the cached EPUB
for etype_str in &["html", "mobi", "pdf"] {
    let e_input_hash = format!("epub:{}", epub_hash);
    if let Ok(Some(entry)) = queries::find_export_log(
        &state.db, &meta.url_id, version, etype_str, &e_input_hash,
    ).await {
        hashes.insert(etype_str.to_string(), entry.export_hash.clone());
        urls.insert(etype_str.to_string(), format!(
            "/cache/{}/{}?h={}", etype_str, meta.url_id, entry.export_hash
        ));
    }
}
```

The trick here is that HTML/MOBI/PDF are generated from the EPUB, not from the original fic. So their `input_hash` is `"epub:{epub_hash}"` — they depend on the EPUB, not the source fic. This means if the EPUB changes (because the fic was updated), the HTML/MOBI/PDF caches are also invalidated automatically.

> 💡 **Key Concept**
>
> The input hash chain (`content_hash → epub_hash → html_hash`) creates a dependency graph. When the source fic changes, the EPUB cache is invalidated. When the EPUB changes, the HTML cache is invalidated. This cascading invalidation ensures you never serve outdated files.

The cache URL pattern is: `/cache/{format}/{url_id}?h={export_hash}`. The `?h=` parameter is a cache-busting query string — if the hash changes, browsers and CDNs know to fetch the new file.

## Preventing Duplicate Exports

Here's a subtle but important problem: what if two users request the same uncached fic at the same time? Both requests would see a cache miss, both would start generating the EPUB, and we'd waste resources (and potentially create conflicts on disk).

FicHub solves this with a **semaphore** — a concurrency limiter that ensures only one export happens at a time for a given fic:

```rust
// Acquire semaphore to prevent duplicate concurrent exports
let sem = cache::get_export_semaphore(
    &state.cache_semaphores, &meta.url_id, &EType::Epub,
).await;
let _permit = sem.acquire().await
    .map_err(|e| AppError::Internal(e.to_string()))?;

// Check cache again (double-check pattern)
let cached = queries::find_export_log(
    &state.db, &meta.url_id, version, "epub", &input_hash,
).await?;
if let Some(export_log) = cached {
    // Another concurrent request already generated it!
    // Just use the cached result.
    // ...
}
```

This is the **double-check pattern**:

1. First, we check the cache (the "first check"). If it's a hit, great — no export needed.
2. If it's a miss, we acquire the semaphore (which blocks until no one else is generating for this fic).
3. After acquiring the semaphore, we check the cache again (the "second check"). By now, another request might have finished generating the export, so the cache might be hit.
4. If the second check is still a miss, we generate the export.

This prevents duplicate work while still handling the race condition where two requests arrive simultaneously.

> 💡 **Key Concept**
>
> The semaphore is per-fic, not global. Two different fics can have exports generated simultaneously — only the same fic is serialized. This maximizes throughput while preventing duplicate work.

## Cache Hit Performance

When there's a cache hit, the entire export process is skipped. The handler:

1. Scrapes metadata (network I/O, ~100-500ms)
2. Looks up the cache in the database (single query, ~1-5ms)
3. Returns the cached URLs

That's it. No chapter fetching, no EPUB generation, no file I/O. The cached response is nearly instant after the metadata lookup.

For a cache miss, the process adds:

4. Fetch all chapters (network I/O, could be seconds to minutes)
5. Generate the EPUB (CPU work, ~1-10 seconds)
6. Move file to cache (disk I/O, ~100ms)
7. Record in export_log (database write, ~5ms)
8. Generate HTML bundle (CPU work, ~1-5 seconds)
9. Move HTML to cache (disk I/O, ~100ms)
10. Record HTML in export_log (database write, ~5ms)

The difference between a cache hit and miss can be dramatic — from milliseconds to minutes. For a popular fic that gets requested frequently, the cache pays for itself almost immediately.

## Cache Maintenance

Over time, the cache will grow. Here are some things to keep in mind:

1. **Stale entries** — when a fic's content changes on the source site, the old cache entry becomes stale. The version bump system handles this, but if a fic is never re-requested, the stale entry stays in the database forever.

2. **Disk space** — cached EPUB and HTML files take up disk space. A cleanup job could delete files that haven't been accessed in a long time (but the export_log records would remain).

3. **Cache hit rate** — the ratio of cache hits to total requests tells you how effective the cache is. A high hit rate means most requests are served from cache. A low hit rate might mean you need to warm the cache for popular fics.

These are operational concerns we'll address later in the book when we build the monitoring and maintenance tools.

🧪 **Try It Yourself**

1. Write a function `get_all_cached_formats(pool, url_id, version)` that returns all cached export types for a given fic at a given version. Hint: use `WHERE url_id = $1 AND version = $2`.
2. Write a function `count_cache_entries(pool)` that counts how many total cache entries exist: `SELECT COUNT(*) FROM export_log`.
3. Write a function `get_oldest_cache_entries(pool, limit)` that returns the oldest cache entries by `created` date. This could be useful for cleanup.
4. Think about: what would happen if you bumped the global version for every code change? What are the tradeoffs between freshness and server load? Answer: every cache entry becomes stale, so the first request for every fic would trigger a full regeneration. This is fine for occasional bumps (like format changes) but would be catastrophic for frequent bumps.

---

# Summary of Part 3

In this part, we connected our Rust server to a PostgreSQL database and built the entire data layer. Here's what we covered:

**Chapter 13: PostgreSQL Basics** — We learned what databases are (tables with rows and columns), installed PostgreSQL, created a database and user, learned to use the `psql` command-line tool, and mastered the five basic SQL commands: CREATE TABLE, INSERT, SELECT, UPDATE, and DELETE. We covered essential data types (TEXT, INTEGER, BIGINT, BOOLEAN, TIMESTAMPTZ) and discussed why PostgreSQL is the right choice for FicHub.

**Chapter 14: SQLx Basics** — We added the `sqlx` crate to our project with the right features, learned about connection pools (`PgPool` and `PgPoolOptions`), and ran our first queries from Rust code. We saw how `fetch_one`, `fetch_optional`, `fetch_all`, and `execute` serve different purposes, and explored parameterized queries for SQL injection prevention.

**Chapter 15: Migrations** — We created the database schema using numbered migration files in the `migrations/` directory. We studied every table in FicHub's initial schema: `request_source`, `request_log`, `fic_info`, `export_log`, `fic_blacklist`, `author_blacklist`, and `fic_version_bump`. We saw how to run migrations both with the SQLx CLI and programmatically in Rust, and discussed why `IF NOT EXISTS` makes migrations safe to re-run. We also explored indexes (regular and partial) and how they speed up common queries.

**Chapter 16: Models and Queries** — We created Rust structs with `#[derive(FromRow)]` to map database rows to Rust types. We built the `upsert_fic_info` function using `INSERT ON CONFLICT DO UPDATE`, learned about the `EXCLUDED` keyword, explored `query_as` vs `query`, and saw the search function with `ILIKE` wildcards.

**Chapter 17: Request Logging** — We built the request tracking system with `request_source` and `request_log` tables. We saw the UPSERT-with-RETURNING pattern for "get or create" operations, learned how to log all the details of each request, and explored the `build_info_string` function that creates human-readable summaries.

**Chapter 18: Blacklists** — We implemented the fic and author blacklisting system with different severity levels. We saw how greylisting (reason code 6) shows metadata without download links, while hard blacklisting returns error messages. We explored the `build_metadata_response` function and discussed the tradeoffs of greylisting.

**Chapter 19: Cache Queries** — We built the caching system with `export_log` and `fic_version_bump`. We learned how version bumping invalidates caches at both global and per-fic granularity, traced through the full cache lookup flow, and understood the cascading invalidation chain from content_hash → epub_hash → html_hash.

The database layer is the memory of our server. Every piece of data that persists between restarts flows through these queries. The schema we built in this part is the foundation that everything else depends on — the scraping system fills these tables with data, the API handlers read from them, and the analytics tools query them for insights.

In the next part, we'll build the scraping system that fetches fanfiction metadata from external sites — the system that produces the data we've been storing in these tables. We'll learn how to parse HTML, extract metadata, and handle the quirks of different fanfiction sites.

---

*End of Part 3*
