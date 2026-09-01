# Building the FicHub Backend

## A Complete Guide to Building a Fanfiction Download Server in Rust

**By the FicHub Project**

*Version 1.0 — 2026*

---

> *"This book teaches you how to build a complete, production-ready backend server in Rust. From web scraping to EPUB generation, from Redis rate limiting to collaborative filtering recommendations — every chapter walks you through real code from a real project."*

---

## Table of Contents

### Part 1: Welcome to Rust Backend Development
1. What We're Building
2. How the Web Works
3. Setting Up Your Workshop
4. Your First Rust Program

### Part 2: Axum Web Framework
5. Your First Web Server
6. Configuration and Error Handling
7. Connecting to PostgreSQL
8. Database Migrations and Models
9. CRUD Operations
10. The Axum Router and Middleware

### Part 3: Web Scraping
11. Understanding Fanfiction Sites
12. Building Web Scrapers
13. The Scraper Registry
14. AO3 Scraper Deep Dive
15. Other Site Scrapers

### Part 4: Export and Caching
16. Generating EPUB Files
17. HTML Bundles
18. The Disk Cache
19. Rate Limiting with Redis
20. The Export Flow

### Part 5: Recommendation Engine
21. Collaborative Filtering
22. The Collection Worker
23. Community Suggestions
24. Voting and Scoring

### Part 6: Server and Deployment
25. The Full Axum Router
26. Serving Static Files
27. Docker and Docker Compose
28. Cross-Compilation and Deploy

### Part 7: Advanced Backend Features
29. The Search System
30. The Tag System
31. The OPDS Catalog
32. The API Documentation
33. Performance and Optimization

### Part 8: Testing and Polish
34. Backend Testing
35. Production Hardening
36. What's Next?

---

# Part 1: Welcome to Rust Backend Development

---

## Chapter 1: What We're Building

Welcome to the most exciting project you'll undertake this year. By the end of this book, you'll have built a complete, production-ready backend server in Rust that can scrape fanfiction from multiple websites, convert stories into beautiful EPUB files and HTML bundles, serve them through a blazing-fast web API, recommend similar stories to readers, and do all of this while running safely behind rate limits and a rock-solid caching layer. This isn't a toy project — it's a real, deployable application that handles real traffic and real data.

### What Is FicHub?

FicHub is a fanfiction download service. Think of it as a personal library assistant for the fanfiction world. You give it a URL to a story on Archive of Our Own, FanFiction.net, or several other sites, and it fetches the story, packages it up as an EPUB file (or HTML bundle, MOBI, or PDF), and hands it back to you for reading on your e-reader, tablet, or phone.

But FicHub is more than just a downloader. It's a complete backend system that:

- **Scrapes fanfiction sites** — It knows how to extract metadata and chapter content from AO3, FF.net, XenForo forums, and more
- **Generates export files** — It produces EPUBs with proper table of contents, metadata, and formatting, plus HTML bundles for browser reading
- **Caches aggressively** — Once a story is fetched, it's stored on disk and in a database so the next request is instant
- **Rate limits intelligently** — Token bucket algorithms prevent any single user (or bot) from hammering the upstream sites
- **Recommends stories** — A collaborative filtering engine uses reader bookmark data to suggest stories you'll love
- **Supports community tagging** — Users can tag stories, vote on tags, and curate a rich metadata layer
- **Offers advanced search** — Full-text search with tag filters, word count ranges, and more
- **Publishes an OPDS catalog** — E-reader apps can browse and download directly through the standard OPDS protocol
- **Deploys via Docker** — A multi-stage Dockerfile and docker-compose setup makes deployment trivial

This is a backend-only book. We won't build the SvelteKit frontend — that's a separate project. Instead, we'll focus entirely on the Rust server, the database, the scraping engine, the export pipeline, and everything that happens behind the scenes.

### Why Rust?

You might be wondering why we're building this in Rust instead of Python, Node.js, or Go. The answer comes down to three things: performance, safety, and the joy of building something that just works.

**Performance.** FicHub needs to scrape web pages, parse HTML, generate EPUB files, and serve HTTP requests — all potentially at the same time for different users. Rust's async runtime (Tokio) lets us handle thousands of concurrent connections without breaking a sweat. The zero-cost abstractions mean we get high-level ergonomics without paying a runtime tax. When benchmarked against Node.js for HTTP throughput, Rust with Axum consistently handles 2-3x more requests per second with lower latency. When compared to Python, the difference is even more dramatic — Rust is typically 10-50x faster for CPU-bound operations like HTML parsing and EPUB generation.

**Safety.** When you're building a web server that handles user input, database connections, and file system operations, bugs can be catastrophic. Rust's ownership system catches entire categories of bugs at compile time — no null pointer dereferences, no data races, no use-after-free. If the code compiles, it's already free of a huge class of bugs. For a web server that processes untrusted input from multiple fanfiction sites and serves it back to users, this safety guarantee is invaluable.

**Ecosystem.** The Rust web ecosystem has matured beautifully. Axum gives us a web framework that's fast, type-safe, and a pleasure to use. SQLx provides compile-time checked SQL queries against a real PostgreSQL database. The `scraper` crate handles HTML parsing. `epub-builder` generates EPUB files. Every piece of the puzzle has a high-quality, well-maintained crate. The Rust package registry (crates.io) has over 150,000 crates, and the quality bar is remarkably high.

**Joy.** This might sound surprising, but Rust is genuinely fun to write. The compiler is strict, but it's also incredibly helpful. Error messages explain what went wrong and suggest fixes. The `cargo` build system handles dependencies, testing, and formatting. And there's a deep satisfaction in knowing that when your code works, it really works.

### The Architecture at a Glance

Let's zoom out and look at how FicHub is organized:

```
fichub/
├── src/
│   ├── main.rs          — Entry point, loads config, starts server
│   ├── lib.rs           — Module declarations, re-exports for testing
│   ├── server.rs        — AppState, router construction, server binding
│   ├── config.rs        — Configuration from environment variables
│   ├── error.rs         — Application-wide error types
│   ├── db/              — Database connection, models, queries
│   ├── scrape/          — Web scraper trait and per-site implementations
│   ├── export/          — EPUB, HTML, MOBI/PDF generation
│   ├── cache/           — Disk caching and semaphore-based deduplication
│   ├── limiter/         — Rate limiting with Redis-backed token buckets
│   ├── recommender/     — Collaborative filtering engine and worker
│   ├── tags/            — Tag resolution, voting, curator tools
│   ├── search/          — Full-text search with dynamic query building
│   ├── routes/          — HTTP handlers for all API endpoints
│   │   └── opds/        — OPDS catalog feed generation
│   └── frontend/        — Static file serving for the SvelteKit build
├── migrations/          — SQL migration files (001-004)
├── templates/           — Tera templates
├── tests/               — Integration tests
├── docker/              — Additional Dockerfiles (Calibre)
├── Dockerfile           — Multi-stage production build
├── docker-compose.yml   — Full stack: Postgres, Redis, Calibre, App
└── Cargo.toml           — Dependencies and project metadata
```

The key insight is that FicHub is organized around **concerns**, not layers. Each module handles one thing:

- `scrape` knows how to talk to fanfiction websites
- `export` knows how to create files
- `cache` knows how to store and retrieve them
- `recommender` knows how to suggest stories
- `tags` knows how to organize metadata
- `search` knows how to find stories
- `routes` connects everything to HTTP endpoints
- `server` wires it all together

### What You'll Learn

This book is structured as a journey. We'll start with the basics and build up layer by layer:

**Part 1 (Chapters 1–4):** We set up our development environment and learn the Rust fundamentals we need. You'll install Rust, PostgreSQL, and Redis, then write your first Rust programs.

**Part 2 (Chapters 5–10):** We build the web server layer with Axum, connect to PostgreSQL, and handle configuration and errors. By the end, you'll have a working API server.

**Part 3 (Chapters 11–15):** We dive into web scraping — understanding fanfiction site structures, building scrapers, and implementing the registry pattern. You'll scrape AO3, FF.net, and XenForo forums.

**Part 4 (Chapters 16–20):** We tackle EPUB generation, HTML bundles, disk caching, rate limiting, and the complete export flow. We trace every step from request to response.

**Part 5 (Chapters 21–24):** We build the recommendation engine with collaborative filtering, community suggestions, and voting. You'll understand Jaccard coefficients and co-occurrence tables.

**Part 6 (Chapters 25–28):** We wire up the complete router, serve static files, Docker-ize everything, and deploy to a real server.

**Part 7 (Chapters 29–33):** We implement advanced features: search, tags, OPDS, API docs, and performance tuning. These make the difference between a prototype and a product.

**Part 8 (Chapters 34–36):** We test everything, harden for production, and look at what's next. You'll learn testing patterns, security best practices, and deployment strategies.

### Who Is This Book For?

This book is for developers who want to build real backend systems in Rust. You don't need to be a Rust expert — we'll cover the language features as we encounter them. But you should be comfortable with:

- Basic programming concepts (variables, functions, loops, conditionals)
- Command-line tools (we'll use `cargo`, `psql`, `redis-cli`)
- Web concepts (HTTP requests, JSON, databases, URLs)

If you've written code in any language before, you'll be fine. The concepts are universal; only the syntax is new.

### A Note on the Code

Every code example in this book comes from the actual FicHub source code. We'll walk through real functions, real structs, and real SQL queries. When you see code, it's not a simplified toy — it's the actual implementation, explained line by line.

Some things to keep in mind:

- The code uses **Rust 2024 edition** features
- Dependencies are pinned to specific versions in `Cargo.toml`
- SQL queries use `sqlx`'s compile-time checking
- The async runtime is **Tokio**
- We use `tracing` for structured logging
- Error handling uses a custom `AppError` enum, not panics or unwrap

Let's get started.

---

## Chapter 2: How the Web Works

Before we dive into code, let's make sure we're all on the same page about how web servers work. Even if you've built web apps before, a quick refresher will help when we encounter these concepts in Rust.

### HTTP: The Language of the Web

HTTP (HyperText Transfer Protocol) is how clients and servers talk to each other. When you type a URL into your browser or tap a link on your phone, your device sends an HTTP **request** to a server, and the server sends back an HTTP **response**.

A request has three key parts:

1. **Method** — What the client wants to do:
   - `GET` — Fetch data (the most common)
   - `POST` — Send data to create something
   - `PUT` — Update existing data
   - `DELETE` — Remove data

2. **Path** — Which resource the client wants:
   - `/api/v0/epub?q=https://archiveofourown.org/works/123456`
   - `/cache/epub/abc123?h=def456`

3. **Headers** — Metadata about the request:
   - `Content-Type: application/json`
   - `Authorization: Bearer secret-token`

A response has:

1. **Status code** — Did it work?
   - `200 OK` — Success!
   - `404 Not Found` — Resource doesn't exist
   - `500 Internal Server Error` — Something went wrong on the server
   - `429 Too Many Requests` — Rate limited

2. **Headers** — Metadata about the response

3. **Body** — The actual data (HTML, JSON, file bytes, etc.)

### JSON: The Data Format

Most modern APIs communicate using JSON (JavaScript Object Notation). It's lightweight, human-readable, and every programming language has great support for it.

A typical FicHub API response looks like this:

```json
{
  "err": 0,
  "url_id": "abc123def456",
  "meta": {
    "title": "My Favorite Story",
    "author": "GreatAuthor",
    "chapters": 10,
    "words": 50000,
    "status": "complete"
  },
  "urls": {
    "epub": "/cache/epub/abc123def456?h=md5hash",
    "html": "/cache/html/abc123def456?h=md5hash"
  }
}
```

The `err` field is the convention FicHub uses — `0` means success, negative numbers mean various errors. This is a pattern you'll see throughout the codebase. The response is compact and contains everything the client needs: story metadata, download URLs, and error information.

JSON objects are curly braces `{}`, arrays are square brackets `[]`, and key-value pairs are separated by colons. Strings are in double quotes, numbers don't need quotes, and booleans are `true`/`false`.

### Databases: Where Data Lives

A web server needs to remember things. When FicHub scrapes a story's metadata, it needs to store it somewhere so it doesn't have to scrape again. When a user votes on a tag, that vote needs to persist. When the cache is checked, we need to know if a file was already generated.

FicHub uses **PostgreSQL** — a powerful, open-source relational database. It's known for reliability, feature richness, and excellent performance. It handles everything from simple key-value lookups to complex full-text searches with ease.

Here's the core idea: data lives in **tables** (think spreadsheets), and you interact with them using **SQL** (Structured Query Language):

```sql
-- Create a table
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    chapters INT4 NOT NULL,
    words INT8 NOT NULL
);

-- Insert data
INSERT INTO fic_info (id, title, author, chapters, words)
VALUES ('abc123', 'My Story', 'Author Name', 10, 50000);

-- Query data
SELECT title, author FROM fic_info WHERE words > 10000;

-- Update data
UPDATE fic_info SET chapters = 11 WHERE id = 'abc123';

-- Delete data
DELETE FROM fic_info WHERE id = 'abc123';
```

FicHub uses **SQLx**, a Rust library that lets you write SQL queries directly in your Rust code. The magic part? SQLx checks your queries against the actual database schema at compile time. If your query has a typo or references a column that doesn't exist, your code won't compile. That's incredibly powerful for catching bugs early.

### Caching: Don't Do Work Twice

Imagine 100 users all request the same popular story within a minute. Without caching, FicHub would scrape the story 100 times — wasting bandwidth, annoying the upstream site, and making users wait.

With caching, the first request does the work, stores the result, and every subsequent request gets the cached version instantly.

FicHub uses a **two-layer cache**:

1. **Disk cache** — The actual EPUB/HTML files are stored on the filesystem in a hash-based directory structure. Files are organized by export type, then by url_id chunks, then by the content hash. This prevents any single directory from having too many files.

2. **Database cache** — The `export_log` table tracks which files have been generated, so we can look up the hash without checking the filesystem. This is much faster than scanning directories.

The flow looks like this:
1. Request comes in for a story
2. Check `export_log` — is there a cached version?
3. If yes, return the cached file immediately
4. If no, scrape the story, generate the file, store it, and return it

### Rate Limiting: Being a Good Citizen

Fanfiction sites aren't huge enterprises with unlimited bandwidth. AO3 is run by volunteers. FF.net has been known to block aggressive scrapers. As a good net citizen, FicHub needs to limit how fast it makes requests.

Rate limiting works like a token bucket:
- Imagine a bucket that holds tokens
- Each request costs one token
- Tokens drip into the bucket at a steady rate
- If the bucket is empty, the request has to wait

FicHub implements this with Redis (a fast in-memory data store) and Lua scripts for atomic operations. The Lua script runs inside Redis, so the token check-and-update happens atomically — no race conditions possible. We'll dive deep into this in Chapter 19.

### The Request Lifecycle in FicHub

Let's trace what happens when a user requests a story:

1. **HTTP request arrives** at `GET /api/v0/epub?q=https://archiveofourown.org/works/123456`
2. **Rate limiter** checks if this IP is allowed to make a request (checks global bucket, then per-IP bucket)
3. **Router** matches the URL to the `epub_handler` function
4. **Scraper registry** finds the AO3 scraper for this URL by calling `can_handle()` on each registered scraper
5. **AO3 scraper** fetches the story page and extracts metadata (title, author, word count, status)
6. **Database** checks if this story was already scraped (upsert fic_info)
7. **Blacklist check** — is this story or author blocked?
8. **Cache check** — is there a cached EPUB? (first pass)
9. **If cached**: build response with download URLs
10. **If not cached**: 
    a. Acquire semaphore (prevent duplicate exports for same story)
    b. Double-check cache (another request might have finished while we waited)
    c. Fetch all chapter content from AO3 (single request for full work view)
    d. Generate EPUB file with metadata and chapters
    e. Generate HTML bundle with navigation
    f. Move files to cache directory (atomic rename)
    g. Record in export_log (for future cache hits)
11. **Log the request** in request_log (for analytics)
12. **Return JSON response** with metadata and download URLs

That's a lot of steps! But each one is focused on one thing, and together they form a robust pipeline. We'll build each piece in the chapters ahead.

### API Design Principles

FicHub follows a few API design principles worth noting:

**Versioned endpoints.** All API routes start with `/api/v0/`. This means we can add new versions without breaking existing clients. The `v0` prefix is a commitment to stability — once an endpoint is in v0, it won't change in a breaking way.

**Consistent error format.** Every response has an `err` field. `0` means success. Negative numbers are specific error codes. This makes it easy for clients to handle errors programmatically.

**Stateless where possible.** The server doesn't store user sessions. Rate limiting is IP-based. This simplifies deployment — you can run multiple instances behind a load balancer without sticky sessions.

**Content negotiation.** The same endpoint can return different formats. `/api/v0/epub` returns metadata, while `/cache/epub/{url_id}` returns the actual file.

**RESTful naming.** Resources are nouns (`/api/v0/tags`, `/api/v0/recommendations`), operations are HTTP methods (GET for read, POST for create/update, DELETE for remove).

Now that you understand the web fundamentals, let's set up our development environment and start writing code.

---

## Chapter 3: Setting Up Your Workshop

Time to get our hands dirty. In this chapter, we'll install everything we need to build FicHub and make sure it all works together. By the end, you'll have Rust, PostgreSQL, Redis, and all the tools running on your machine.

### Installing Rust

Rust has the best installation experience of any systems language. One command does everything:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This installs `rustup` (the Rust toolchain manager), `rustc` (the compiler), and `cargo` (the build system and package manager). After installation, restart your terminal or run:

```bash
source $HOME/.cargo/env
```

Verify it worked:

```bash
rustc --version
# rustc 1.XX.0 (should show your installed version)

cargo --version
# cargo 1.XX.0

rustup --version
# rustup 1.XX.0
```

Rustup manages multiple toolchains. When the FicHub project specifies `edition = "2024"` in `Cargo.toml`, rustup will automatically download the right compiler version when you build. You don't need to manually manage this — just run `cargo build` and rustup handles the rest.

You can check which toolchain is active with:

```bash
rustup show
```

This shows your default toolchain, host triple, and installed targets.

🧪 **Try It Yourself:** Create a tiny Rust project to make sure everything works:

```bash
mkdir hello && cd hello
cargo init
cargo run
```

You should see `Hello, world!` printed. That's your first Rust program compiled and executed. The `cargo init` command creates a new project with a `Cargo.toml` (dependency manifest) and `src/main.rs` (entry point).

Now try adding a dependency. Edit `Cargo.toml` to add `rand`:

```toml
[dependencies]
rand = "0.8"
```

Then update `src/main.rs`:

```rust
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let n: u32 = rng.gen_range(1..=100);
    println!("Random number: {}", n);
}
```

Run `cargo run` and you'll see a random number. Cargo automatically downloaded, compiled, and linked the `rand` crate. That's the Rust package management experience.

### Installing PostgreSQL

FicHub requires PostgreSQL 14 or later. The installation depends on your operating system.

**On Arch/Manjaro:**

```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl enable --now postgresql
```

The `initdb` command creates the initial database cluster. The `-D` flag specifies where to store the data files.

**On Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl enable --now postgresql
```

The `postgresql-contrib` package includes useful extensions like `pg_trgm` (trigram matching for fuzzy search).

**On macOS:**

```bash
brew install postgresql@16
brew services start postgresql@16
```

**Via Docker (recommended for beginners):**

```bash
docker run -d   --name fichub-postgres   -e POSTGRES_DB=fichub   -e POSTGRES_USER=fichub   -e POSTGRES_PASSWORD=fichub   -p 5432:5432   postgres:16-alpine
```

Docker is the easiest way to get started because it handles all the system-level configuration automatically. The `-d` flag runs it in detached mode (background), `--name` gives it a friendly name, and `-p 5432:5432` maps the container's port to your host.

Once PostgreSQL is running, create the fichub database and user:

```bash
# If using system PostgreSQL (not Docker):
sudo -u postgres psql
```

```sql
CREATE USER fichub WITH PASSWORD 'fichub';
CREATE DATABASE fichub OWNER fichub;
GRANT ALL PRIVILEGES ON DATABASE fichub TO fichub;
\q
```

The `\q` command quits the psql client.

Test the connection:

```bash
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

If you see `?column?` with value `1`, you're connected.

⚠️ **Watch Out:** The default PostgreSQL authentication method on many systems is `peer` (which only works for the `postgres` OS user) or `scram-sha-256`. Make sure your `pg_hba.conf` allows password authentication for the `fichub` user. If you're using Docker, this is handled automatically.

If you get an authentication error, edit `/etc/postgresql/16/main/pg_hba.conf` (the path varies by version) and change the line:

```
local   all   all   peer
```

to:

```
local   all   all   md5
```

Then restart PostgreSQL:

```bash
sudo systemctl restart postgresql
```

### Installing Redis

Redis is used for rate limiting and the recommendation worker queue. It's lightweight and easy to install.

**On Arch/Manjaro:**

```bash
sudo pacman -S redis
sudo systemctl enable --now redis
```

**On Ubuntu/Debian:**

```bash
sudo apt install redis-server
sudo systemctl enable --now redis-server
```

**On macOS:**

```bash
brew install redis
brew services start redis
```

**Via Docker:**

```bash
docker run -d   --name fichub-redis   -p 6379:6379   redis:7-alpine
```

Test it:

```bash
redis-cli ping
# PONG

redis-cli set test "hello"
# OK

redis-cli get test
# "hello"
```

Redis stores data as key-value pairs. It supports strings, lists, sets, hashes, and more. FicHub uses it for:
- Rate limit counters (string keys with TTL)
- Token bucket state (hash keys with Lua scripts)
- Work queues (lists for background processing)

### Setting Up the Project

Clone the FicHub repository and set up the environment:

```bash
git clone https://github.com/your-username/fichub.git
cd fichub
```

Create your `.env` file from the example:

```bash
cp .env.example .env
```

Edit `.env` with your database and Redis connection strings:

```bash
DATABASE_URL=postgres://fichub:fichub@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug
```

The `DATABASE_URL` format is `postgres://user:password@host:port/database`. The `REDIS_URL` format is `redis://host:port`. The `RUST_LOG` variable controls logging verbosity — `info` for general messages, `fichub=debug` for detailed FicHub-specific messages.

Install the `sqlx-cli` tool for running migrations:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

Run the database migrations:

```bash
sqlx migrate run --source migrations
```

This creates all the tables FicHub needs:

```
[001] initial_schema.sql      — Core tables (fic_info, export_log, etc.)
[002] recommender.sql         — Recommendation engine tables
[003] tagging.sql             — Tag system tables and triggers
[004] shelves.sql             — OPDS shelf tables
```

Each migration is a versioned SQL script. SQLx tracks which migrations have been applied in a `_sqlx_migrations` table, so running `migrate run` again is safe — it only applies new migrations.

Build the project:

```bash
cargo build
```

The first build will take several minutes as it downloads and compiles all dependencies. Subsequent builds are much faster thanks to incremental compilation. Cargo downloads crates from crates.io, compiles them, and links them into your binary.

You'll see output like:

```
   Compiling libc v0.2.155
   Compiling proc-macro2 v1.0.86
   Compiling unicode-ident v1.0.12
   ...
   Compiling fichub v0.1.0 (/path/to/fichub)
    Finished dev [unoptimized + debuginfo] target(s) in 35.42s
```

### Your Editor Setup

The best Rust development experience comes with VS Code and the `rust-analyzer` extension. Install VS Code if you haven't, then:

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "rust-analyzer"
4. Install it

rust-analyzer provides:
- Real-time error checking as you type
- Auto-completion for functions, structs, and methods
- Inline type hints
- Code actions (auto-import, fill in match arms)
- Go to definition / find references
- Run and debug integration
- Inlay hints showing inferred types

Other excellent editors for Rust include Neovim (with rust-analyzer LSP), Helix, and Emacs (with rustic/lsp-mode). Any editor with LSP support works well.

🧪 **Try It Yourself:** Open the FicHub project in VS Code and navigate to `src/main.rs`. You should see syntax highlighting and rust-analyzer annotations. Try hovering over `config::Config` to see the type information. Try Ctrl+clicking on `Config::from_env` to jump to its definition.

### Understanding the Dependency Graph

FicHub's `Cargo.toml` pulls in a carefully chosen set of dependencies:

```toml
[dependencies]
# Web framework & runtime
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }

# Database
sqlx = { version = "0.9", features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate", "tls-rustls-ring", "derive", "macros"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# HTML parsing
scraper = "0.27"

# EPUB generation
epub-builder = "0.8"

# Template engine
tera = "2"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Config
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Metrics
axum-prometheus = "0.10"

# UUIDs
uuid = { version = "1", features = ["v4"] }

# Hashing
md-5 = "0.11"
sha2 = "0.11"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Compression (zip)
zip = { version = "8" }

# Async traits
async-trait = "0.1"
```

Each dependency has a specific role:
- **axum + tower-http**: The web framework and middleware
- **tokio**: The async runtime (like Node.js's event loop, but for Rust)
- **sqlx**: PostgreSQL client with compile-time query checking
- **redis**: Redis client for rate limiting
- **reqwest**: HTTP client for scraping websites
- **scraper**: HTML parsing library
- **epub-builder**: Creates EPUB files
- **serde + serde_json**: Serialization/deserialization
- **tracing**: Structured logging
- **md5 + sha2**: Cryptographic hashing for cache keys and URL IDs
- **chrono**: Date/time handling
- **zip**: ZIP file creation for HTML bundles
- **async-trait**: Async methods in traits

### Running FicHub

With everything set up, start the server:

```bash
cargo run
```

You should see:

```
2024-XX-XXTXX:XX:XX.XXXXXXXZ  INFO Starting fichub-rs server on port 3000
2024-XX-XXTXX:XX:XX.XXXXXXXZ  INFO Database migrations applied
2024-XX-XXTXX:XX:XX.XXXXXXXZ  INFO Registered 6 scrapers
2024-XX-XXTXX:XX:XX.XXXXXXXZ  INFO Listening on 0.0.0.0:3000
```

The startup sequence is:
1. Load `.env` file (dotenvy)
2. Initialize tracing subscriber (logging)
3. Load configuration from environment variables
4. Connect to PostgreSQL (creates pool, runs migrations)
5. Connect to Redis
6. Build HTTP client
7. Initialize scraper registry (registers 6 scrapers)
8. Initialize rate limiter
9. Build router
10. Bind to port and start serving

Test it:

```bash
curl http://localhost:3000/api/
```

You should get back the API documentation JSON. Congratulations — FicHub is running!

🧪 **Try It Yourself:** Open another terminal and try requesting a story:

```bash
curl "http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/123456"
```

If AO3 is accessible, you'll get back metadata about the story and download URLs. The response will be a JSON object with `err: 0` indicating success.

### Troubleshooting

**"DATABASE_URL must be set"** — Make sure your `.env` file exists and contains a valid `DATABASE_URL`. The dotenvy crate looks for `.env` in the current directory.

**"Failed to connect to database"** — Check that PostgreSQL is running (`systemctl status postgresql` or `docker ps`). Verify the connection string matches your database configuration.

**"Failed to connect to Redis"** — Check that Redis is running (`redis-cli ping`). If you're using Docker, make sure the Redis container is healthy.

**"migrations directory not found"** — Make sure you're running from the project root, or that the migrations directory is next to your binary. The `init_pool` function looks for migrations relative to the executable.

**Build errors** — Run `cargo clean && cargo build` to start fresh. Make sure you have the right Rust toolchain (`rustup show`). Check that system dependencies like `libpq-dev` and `libssl-dev` are installed.

**Port already in use** — Change the PORT in your `.env` file, or find and stop the process using port 3000: `lsof -i :3000` or `ss -tlnp | grep 3000`.

**Memory issues** — The first compilation can use a lot of memory. If you're on a system with limited RAM, try `cargo build --jobs 2` to limit parallelism.

With your workshop set up, let's learn the Rust basics we'll use throughout this book.

---

## Chapter 4: Your First Rust Program

Time to write some Rust. Don't worry if you've never written Rust before — we'll cover everything you need, one concept at a time. And if you're already a Rustacean, feel free to skim ahead to the sections you know.

### Hello, World (The Real Way)

We already ran `cargo init` and `cargo run` to see "Hello, world!". But let's look at what `cargo init` actually generated:

```rust
fn main() {
    println!("Hello, world!");
}
```

That's it. `fn main()` is the entry point — the first function that runs when your program starts. `println!` is a **macro** (note the `!`), not a function. Macros in Rust are powerful — they can inspect and transform code at compile time. The `!` distinguishes macros from regular functions.

The `println!` macro handles formatting, newline characters, and output buffering. The string `"Hello, world!"` is a string literal — it lives in the compiled binary and has the type `&str` (a string slice).

### Variables and Immutability

In Rust, variables are **immutable by default**:

```rust
fn main() {
    let x = 5;
    x = 6; // ERROR! Cannot assign twice to immutable variable
}
```

This is deliberate. Immutability is the default because it makes code safer — you can be sure a variable won't change unexpectedly. When you want to change a variable, you explicitly mark it as mutable:

```rust
fn main() {
    let mut x = 5;
    x = 6; // This is fine
    println!("x = {}", x); // x = 6
}
```

Rust also has **type inference** — you don't always need to specify types:

```rust
let name = "FicHub";       // inferred as &str
let count = 42;             // inferred as i32
let pi = 3.14;              // inferred as f64
let active = true;          // inferred as bool
let items = vec![1, 2, 3]; // inferred as Vec<i32>
```

But you can annotate types when needed or for clarity:

```rust
let count: i64 = 42;
let pi: f64 = 3.14;
let items: Vec<String> = vec!["hello".to_string()];
```

Type annotations are especially useful in complex expressions where the compiler might infer a different type than you expect.

### Ownership: Rust's Killer Feature

This is the concept that makes Rust special. Every value has exactly one **owner**. When the owner goes out of scope, the value is dropped (freed):

```rust
{
    let s = String::from("hello"); // s owns the string
    // ... use s ...
} // s goes out of scope, string is freed automatically
```

This is called RAII (Resource Acquisition Is Initialization). Resources are freed when they go out of scope, no garbage collector needed.

When you assign a value to another variable, ownership **moves**:

```rust
let s1 = String::from("hello");
let s2 = s1; // Ownership moves from s1 to s2
// println!("{}", s1); // ERROR! s1 no longer owns the string
```

This prevents double-free errors. If both s1 and s2 owned the same string, freeing both would crash.

To share ownership without moving, you use references (`&`):

```rust
fn print_title(fic: &FicInfo) {
    println!("{}", fic.title);
}

let fic = get_fic();
print_title(&fic); // Pass by reference
println!("{}", fic.title); // fic still owns the data
```

The `&` creates a borrow — a temporary reference that doesn't take ownership. The borrow checker ensures you can't modify data while it's borrowed.

For shared ownership across async tasks, FicHub uses `Arc` (Atomic Reference Counting):

```rust
let state = Arc::new(AppState { /* ... */ });
// Now state can be cloned and shared across tasks
let state_clone = Arc::clone(&state);
tokio::spawn(async move {
    // Use state_clone in another task
});
```

### Structs: Naming Your Data

Structs are how you create custom types in Rust. FicHub uses structs everywhere — for configuration, database models, API responses, and more.

Here's a simple struct:

```rust
struct FicInfo {
    title: String,
    author: String,
    chapters: i32,
    words: i64,
    status: String,
}
```

To create an instance:

```rust
let fic = FicInfo {
    title: "My Story".to_string(),
    author: "Author".to_string(),
    chapters: 10,
    words: 50000,
    status: "complete".to_string(),
};

println!("{} by {} — {} words", fic.title, fic.author, fic.words);
```

Access fields with dot notation: `fic.title`, `fic.author`.

Structs in FicHub often derive common traits using attributes:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub description: String,
    pub status: String,
    pub source: String,
    // ... more fields
}
```

- `Debug` — lets you print the struct with `{:?}` for debugging
- `Clone` — lets you make copies (implements `.clone()`)
- `Serialize` / `Deserialize` — lets serde convert to/from JSON
- `FromRow` — lets sqlx convert a database row to this struct

### Enums: Choice Types

Enums represent a value that could be one of several variants:

```rust
enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}
```

You can match on enums (this is one of Rust's superpowers):

```rust
impl EType {
    fn suffix(&self) -> &str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }
}
```

The `match` expression is exhaustive — you MUST handle every variant. If you add a new variant to the enum, the compiler will tell you every match statement that needs updating. This prevents forgotten cases.

Enums can also carry data:

```rust
enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}

// Network(String) carries an error message
// ParseError(String) carries parsing details
```

### Error Handling: Result and Option

Rust doesn't have exceptions. Instead, it has two powerful types for handling things that might go wrong:

**`Option<T>`** — A value that might not exist:

```rust
fn find_scraper(url: &str) -> Option<&dyn SiteScraper> {
    if url.contains("archiveofourown.org") {
        Some(&Ao3Scraper)
    } else {
        None
    }
}

// Using it with match:
match find_scraper(url) {
    Some(scraper) => println!("Found scraper!"),
    None => println!("No scraper for this URL"),
}

// Using it with if-let:
if let Some(scraper) = find_scraper(url) {
    println!("Found scraper!");
}

// Using it with unwrap_or (with a default):
let scraper = find_scraper(url).unwrap_or(&DefaultScraper);
```

**`Result<T, E>`** — An operation that might fail:

```rust
fn read_config() -> Result<Config, std::io::Error> {
    let content = std::fs::read_to_string("config.toml")?;
    // ... parse config ...
    Ok(config)
}

// Using it with match:
match read_config() {
    Ok(config) => println!("Config loaded: {:?}", config),
    Err(e) => eprintln!("Failed to read config: {}", e),
}

// Using it with the ? operator:
fn process() -> Result<(), Box<dyn std::error::Error>> {
    let config = read_config()?; // ? propagates errors
    Ok(())
}
```

The `?` operator is Rust's secret weapon for error handling. It propagates errors up the call stack automatically. If the operation fails, the function returns the error immediately. If it succeeds, it unwraps the value. This keeps error handling clean and linear.

FicHub's `AppResult<T>` is an alias for `Result<T, AppError>`:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

### Functions and Closures

Functions in Rust are straightforward:

```rust
fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])
}
```

Key points:
- Parameters need type annotations
- Return types go after `->`
- The last expression without a semicolon is the return value
- The `use` statement imports items into scope

Closures are anonymous functions (like lambdas in other languages):

```rust
let double = |x: i32| x * 2;
println!("{}", double(5)); // 10

// Closures can capture variables from their environment
let threshold = 10;
let is_large = |x: i32| x > threshold;
```

FicHub uses closures with iterators a lot:

```rust
let titles: Vec<String> = fics
    .iter()
    .map(|f| f.title.clone())    // Transform each fic to its title
    .filter(|t| !t.is_empty())   // Keep only non-empty titles
    .collect();                   // Collect into a Vec
```

Iterators are lazy — they don't do work until you call `.collect()` or another consuming method.

### Async Rust

FicHub is an async web server. In async Rust, you use `async fn` and `.await`:

```rust
async fn fetch_metadata(url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    // ... parse HTML ...
    Ok(metadata)
}
```

The `async` keyword makes the function return a future (a value that represents a computation that will complete later). The `.await` keyword pauses until the future completes. You can only `.await` inside `async` functions.

To run async code, you need a runtime. FicHub uses Tokio:

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt().init();
    let config = Config::from_env();
    server::run(config).await;
}
```

The `#[tokio::main]` attribute transforms your `async fn main()` into a regular `fn main()` that sets up the Tokio runtime and runs the async code. Under the hood, it creates a multi-threaded runtime that can execute many futures concurrently.

### Traits: Shared Behavior

Traits define shared behavior (like interfaces in other languages):

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

Any type that implements all the required methods satisfies the trait:

```rust
struct Ao3Scraper;

impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }
    // ... implement other methods
}
```

The `Send + Sync` bounds mean the trait object can be sent across thread boundaries and shared between threads. This is required for async code that runs on a multi-threaded runtime.

This is how FicHub supports multiple fanfiction sites with one interface. The scraper registry holds a `Vec<Box<dyn SiteScraper>>` — a collection of anything that implements `SiteScraper`.

### Putting It All Together

Here's a simplified version of what FicHub's main function does, using everything we've learned:

```rust
#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize structured logging
    tracing_subscriber::fmt().init();
    
    // Load configuration (can panic if env vars missing)
    let config = Config::from_env();
    
    // Connect to PostgreSQL
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    
    // Connect to Redis
    let redis = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    
    // Build the scraper registry
    let scrapers = Arc::new(ScraperRegistry::new());
    
    // Build shared application state
    let state = Arc::new(AppState {
        config,
        db: pool,
        redis: redis_conn,
        http_client: reqwest::Client::new(),
        scraper_registry: scrapers,
        // ...
    });
    
    // Build the router and start serving
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Every concept from this chapter appears in real FicHub code:
- **Variables** — `let config = Config::from_env();`
- **Structs** — `AppState { config, db: pool, ... }`
- **Enums** — `AppError::NotFound(...)`, `EType::Epub`
- **Result/Option** — `.await?`, `.expect("...")`
- **Functions** — `build_router(state)`
- **Ownership** — `Arc::new(...)` for shared state
- **Async** — `async fn`, `.await`, `#[tokio::main]`
- **Traits** — `SiteScraper`, `RateLimiter`

You now have the Rust fundamentals to understand every chapter that follows. Let's build a web server!


---

# Part 2: Axum Web Framework

---

## Chapter 5: Your First Web Server

Let's build a web server. Not a toy "Hello, World" server — a real one with proper routing, state management, extractors, and error handling. By the end of this chapter, you'll understand how Axum works and how FicHub uses it.

### Why Axum?

There are several Rust web frameworks — Actix-web, Rocket, Warp, and others. FicHub uses Axum because it's built on Tower (a middleware framework), integrates seamlessly with Tokio, and has a clean, type-safe API.

Axum's philosophy is "extractors for everything." Instead of manually parsing request bodies, query parameters, and headers, you declare what you want in your function signature, and Axum extracts it for you:

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,           // Shared state
    Query(params): Query<MyParams>,               // Query parameters
    Json(body): Json<MyBody>,                     // Request body
    ConnectInfo(remote): ConnectInfo<SocketAddr>, // Client address
) -> Result<Json<Value>, AppError> {
    // All the data is right here, properly typed
}
```

If any extraction fails (missing query param, invalid JSON body), Axum automatically returns an error response. You never write boilerplate parsing code. The type system ensures you get exactly the data you need.

### Building a Minimal Server

Let's start with the simplest possible Axum server and build up:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, FicHub!" }));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

This creates a single route: `GET /` returns "Hello, FicHub!". The `Router::new()` creates an empty router, `.route()` adds a path, and `axum::serve()` starts listening.

Let's break down each part:
- `Router::new()` — Creates a new, empty router. Routers are composable — you can nest them, merge them, and add middleware layers.
- `.route("/", get(handler))` — Adds a route for the path `/` that responds to GET requests. The `get()` function wraps the handler to only respond to GET.
- `tokio::net::TcpListener::bind("0.0.0.0:3000")` — Binds to all network interfaces on port 3000. The `.await` waits for the bind to complete.
- `axum::serve(listener, app)` — Creates a server that accepts connections on the listener and routes them through the app.
- `.await.unwrap()` — Runs the server forever. The `unwrap()` panics if the server encounters a fatal error.

### The AppState Pattern

In FicHub, almost every handler needs access to the database, Redis, the HTTP client, the scraper registry, and configuration. Instead of passing these as individual parameters, we bundle them into a shared `AppState`:

```rust
use std::sync::Arc;

pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn limiter::RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

Each field serves a specific purpose:
- `config` — All application settings (database URL, port, cache paths, etc.)
- `db` — A PostgreSQL connection pool that manages multiple database connections
- `redis` — A multiplexed Redis connection for rate limiting and queues
- `http_client` — A reqwest client configured with user-agent and timeout
- `scraper_registry` — Maps URLs to the correct site scraper
- `cache_semaphores` — Prevents duplicate concurrent exports for the same story
- `rate_limiter` — Token bucket rate limiter backed by Redis
- `recommender_engine` — Computes personalized story recommendations
- `collection_worker` — Background worker that scrapes user favourites

We wrap it in `Arc` (Atomic Reference Counting) so it can be shared safely across async tasks:

```rust
let state = Arc::new(AppState {
    config,
    db: db_pool,
    redis: redis_conn,
    http_client,
    scraper_registry,
    cache_semaphores: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    rate_limiter: Box::new(rate_limiter),
    recommender_engine,
    collection_worker,
});
```

Then we pass it to the router with `.with_state(state)`. Every handler receives it as `State(state): State<Arc<AppState>>`. The `State` extractor knows how to pull the AppState out of the request.

### Route Definitions

Axum's routing is straightforward:

```rust
Router::new()
    .route("/api/v0/epub", get(routes::export::epub_handler))
    .route("/api/v0/meta", get(routes::meta::meta_handler))
    .route("/api/v0/search", get(crate::search::routes::search_handler))
    .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
```

- `get()` — handles GET requests
- `post()` — handles POST requests
- You can chain multiple methods: `.route("/path", get(handler_a).post(handler_b))`
- You can also use `put()`, `delete()`, `patch()`, and `any()`

FicHub's full router has over 30 routes. We'll see them all in Chapter 25.

### Extractors in Detail

Let's look at FicHub's main export handler to understand extractors:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    // ... use state.db, state.scraper_registry, etc.
}
```

**`State(state)`** — Extracts the shared AppState. This is why we use `.with_state(state)` on the router. The type `Arc<AppState>` tells Axum what to look for.

**`Query(params)`** — Deserializes query parameters into a struct:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,        // Optional string parameter
    pub automated: Option<String>, // Optional flag
    pub format: Option<String>,    // Optional format preference
}
```

A request to `GET /api/v0/epub?q=https://example.com&format=epub` would populate `params.q` with `Some("https://example.com")` and `params.format` with `Some("epub")`. Missing parameters become `None`.

**`Json(body)`** — Deserializes the JSON request body into a struct:

```rust
#[derive(Debug, Deserialize)]
pub struct SubmitBody {
    pub url_id: String,        // Required
    pub tag_name: String,      // Required
    pub tag_type_id: i16,      // Required
}

pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();
    // ... use body.url_id, body.tag_name, etc.
}
```

If the request body isn't valid JSON or doesn't match the struct, Axum automatically returns a 400 Bad Request error.

**`Path(params)`** — Extracts path segments:

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // etype_str = "epub", url_id = "abc123", fname = "abc123.epub"
}
```

The tuple type tells Axum how many path segments to extract and what types they should be.

**`ConnectInfo(remote)`** — Extracts the client's socket address:

```rust
async fn remote_handler(
    ConnectInfo(remote_addr): ConnectInfo<std::net::SocketAddr>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ip": remote_addr.ip().to_string(),
        "port": remote_addr.port(),
    }))
}
```

This is useful for rate limiting, logging, and client identification.

### Return Types

Handlers return things that implement `IntoResponse`. The most common:

- `Json<Value>` — Returns JSON with `application/json` content type
- `Result<Json<Value>, AppError>` — Returns JSON on success or error
- `Response` — A raw HTTP response (useful for file downloads)
- `impl IntoResponse` — Any type that can become a response
- `StatusCode` — Just a status code with no body

The `Result` type is the most common in FicHub. If the handler returns `Ok(json)`, Axum sends a 200 response with the JSON body. If it returns `Err(app_error)`, Axum calls `AppError::into_response()` which maps the error to the right HTTP status code and JSON error body.

### Middleware Layers

Middleware wraps your handlers and adds behavior. Axum uses Tower layers:

```rust
Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new_for_http())  // Logs every request
    .layer(CorsLayer::permissive())     // Allows all CORS origins
    .with_state(state)
```

Layers are applied in order — the outermost layer wraps the innermost. So `TraceLayer` wraps `CorsLayer`, which wraps the handlers. This means logging includes CORS headers.

FicHub uses two layers:
- `TraceLayer` — Structured logging of every request/response. Uses the `tracing` crate for structured, context-rich logs.
- `CorsLayer` — Cross-Origin Resource Sharing headers. Needed if the frontend is on a different domain (e.g., running on localhost:5173 during development).

### Method Routing

For routes that accept multiple HTTP methods:

```rust
// Only GET
.route("/api/v0/epub", get(epub_handler))

// Only POST
.route("/api/v0/tags/submit", post(submit_tag))

// GET and POST on the same path
.route("/api/resource", get(get_handler).post(create_handler))

// All methods
.route("/api/resource", any(any_handler))
```

You can also use `put()`, `delete()`, `patch()`, `head()`, and `options()`.

### Nested Routers

For complex applications, you can nest routers:

```rust
let api_routes = Router::new()
    .route("/epub", get(epub_handler))
    .route("/meta", get(meta_handler));

let app = Router::new()
    .nest("/api/v0", api_routes)
    .with_state(state);
```

This creates routes `/api/v0/epub` and `/api/v0/meta`. Nesting helps organize large route trees.

FicHub doesn't use nesting for API routes (they're all defined flat for clarity), but it's good to know the pattern.

### The Server Startup Sequence

Looking at `server.rs`, here's how FicHub starts up:

```rust
pub async fn run(config: Config) {
    // 1. Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url).await
        .expect("Failed to connect to database");
    
    // 2. Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");
    
    // 3. Build HTTP client (with user agent and timeout)
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");
    
    // 4. Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    
    // 5. Initialize rate limiter (loads Lua script into Redis)
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    ).await.expect("Failed to initialize rate limiter");
    
    // 6. Load datacenter IPs for blocking
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }
    
    // 7. Create recommendation engine and collection worker
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let collection_worker = CollectionWorker::new(
        db_pool.clone(), redis_conn.clone(), http_client.clone(),
        config.clone(), scraper_registry.clone(),
    );
    
    // 8. Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine,
        collection_worker,
    });
    
    // 9. Build router
    let app = build_router(state).await;
    
    // 10. Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("Failed to bind to address");
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    ).await.expect("Server error");
}
```

Each step initializes a component. The order matters — you need the database pool before you can create the scraper registry, and you need everything before you can create AppState. The `into_make_service_with_connect_info` enables the `ConnectInfo` extractor so handlers can access the client's IP address.

🧪 **Try It Yourself:** Modify the minimal server from earlier to accept a query parameter:

```rust
use axum::{extract::Query, routing::get, Json, Router};
use serde::Deserialize;

#[derive(Deserialize)]
struct NameQuery {
    name: Option<String>,
}

async fn greet(Query(params): Query<NameQuery>) -> String {
    let name = params.name.unwrap_or_else(|| "World".to_string());
    format!("Hello, {}!", name)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/greet", get(greet));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

Visit `http://localhost:3000/greet?name=FicHub` and you'll see "Hello, FicHub!".

Now try adding a POST endpoint that accepts JSON:

```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct CreateFic {
    title: String,
    author: String,
}

#[derive(Serialize)]
struct FicResponse {
    id: String,
    title: String,
    author: String,
}

async fn create_fic(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateFic>,
) -> Json<FicResponse> {
    let id = generate_url_id(0, &body.title);
    Json(FicResponse {
        id,
        title: body.title,
        author: body.author,
    })
}
```

This shows how Axum extractors compose — you can have `State`, `Json`, and other extractors in any order.

---

## Chapter 6: Configuration and Error Handling

Every production application needs two things: a way to load configuration and a way to handle errors gracefully. FicHub does both in a type-safe, clean way.

### The Config Struct

FicHub's configuration is a single struct that holds every setting the application needs:

```rust
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub app_port: u16,
    pub frontend_dir: PathBuf,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
    pub rec_max_user_favourite_pages: u32,
    pub rec_max_recommendations: usize,
    pub rec_min_favouriters_for_collab: u32,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    pub rec_suggest_limit_per_hour: u32,
    pub rec_vote_limit_per_hour: u32,
    pub rec_precompute_enabled: bool,
    pub rec_precompute_interval_hours: u32,
    pub rec_enable_cross_site: bool,
    // Tagging v3 settings
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

That's a lot of fields! But it's organized by concern, and each field has a sensible default. The `Clone` derive is important — Config needs to be cloned into AppState and potentially shared across tasks.

### Loading from Environment Variables

FicHub loads configuration from environment variables using `std::env::var`:

```rust
impl Config {
    pub fn from_env() -> Self {
        // Required — these panic if missing
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        // Optional — these use defaults
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
            .ok()  // .ok() converts Result to Option
            .filter(|s| !s.is_empty())  // Ignore empty strings
            .map(PathBuf::from);
        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        let node_name = std::env::var("NODE_NAME")
            .unwrap_or_else(|_| "orion".to_string());
        let calibre_container = std::env::var("CALIBRE_CONTAINER")
            .unwrap_or_default();  // Empty string if not set
        let tmp_dir = std::env::var("TMP_DIR")
            .unwrap_or_else(|_| "./tmp".to_string());
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());
        
        // Trusted proxies: comma-separated list
        let trusted_proxies = std::env::var("TRUSTED_PROXIES")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        // IP tag sources: "path,type,tag" per line
        let ip_tag_sources = std::env::var("IP_TAG_SOURCES")
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() == 3 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string(), parts[2].trim().to_string()))
                } else {
                    None
                }
            })
            .collect();
        
        // Recommender settings with defaults
        let rec_default_delay_secs = std::env::var("REC_DEFAULT_DELAY_SECS")
            .unwrap_or_else(|_| "5".to_string())
            .parse().unwrap_or(5);
        let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
            .unwrap_or_else(|_| "{}".to_string());
        let rec_site_rate_limits: HashMap<String, u64> =
            serde_json::from_str(&rec_site_rates_str).unwrap_or_default();
        // ... more fields ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            secondary_cache_dir,
            export_version,
            dynamic_rate_limit,
            node_name,
            calibre_container,
            tmp_dir: PathBuf::from(tmp_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
            trusted_proxies,
            ip_tag_sources,
            rec_default_delay_secs,
            rec_site_rate_limits,
            // ...
        }
    }
}
```

Key patterns:
- **Required vars** use `.expect()` — the application can't start without them
- **Optional vars** use `.unwrap_or_else()` with sensible defaults
- **Typed vars** chain `.parse::<Type>()` after getting the string
- **Boolean vars** parse "true"/"false" strings
- **JSON vars** use `serde_json::from_str` for complex config like rate limits
- **List vars** split on commas and filter empty strings
- **Path vars** convert to `PathBuf` for filesystem operations

### The .env File

During development, you don't want to set environment variables in your shell every time. The `dotenvy` crate loads them from a `.env` file:

```bash
# .env
DATABASE_URL=postgres://fichub:fichub@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug
DYNAMIC_RATE_LIMIT=true
NODE_NAME=dev-machine
```

In `main.rs`:

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env if it exists
    // ... rest of startup
}
```

The `.ok()` at the end means "if loading .env fails, that's fine." In production, you'd set environment variables directly (in Docker, systemd, etc.) rather than using a .env file. The .env file is just for development convenience.

### The AppError Enum

Error handling is critical in a web server. FicHub defines a comprehensive error enum:

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message
    BadRequest(i32, String),
    /// Rate limited - wait N seconds
    RateLimited(u64),
    /// Resource not found
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Scraper error (upstream site problem)
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

Each variant represents a different kind of error with appropriate context:
- `BadRequest(code, message)` — Client sent bad input. The code is a numeric error code (like -1, -5, -7) that clients can programmatically handle.
- `RateLimited(retry_after)` — Too many requests. Contains the number of seconds to wait.
- `NotFound(resource)` — Requested resource doesn't exist.
- `Internal(msg)` — Something broke on the server. The message is logged but not sent to the client.
- `ScrapeError(msg)` — Failed to scrape a website. Could be network error, parse error, or blocked.
- `ExportError(msg)` — Failed to generate a file (EPUB, HTML, etc.).
- `Database(msg)` — Database operation failed (connection, query, etc.).
- `CacheError(msg)` — Cache operation failed (Redis, filesystem, etc.).

### IntoResponse: Mapping Errors to HTTP

The magic is in the `IntoResponse` implementation. It maps each error variant to the right HTTP status code and JSON body:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS, json!({
                    "err": -429,
                    "msg": "rate limited",
                    "retry_after": retry_after
                }))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "internal server error"
                }))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -5,
                    "msg": "export failed"
                }))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "database error"
                }))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({
                    "err": -1,
                    "msg": "cache error"
                }))
            }
        };
        (status, Json(body)).into_response()
    }
}
```

Notice a few important things:
- **Internal errors are logged** with `tracing::error!` before being returned. The client sees a generic message ("internal server error"), but the logs have the details. This is security best practice — never leak internal details to clients.
- **Scrape errors use 502 Bad Gateway** — the upstream site is the problem, not FicHub. This tells load balancers and monitoring tools that the issue is external.
- **Rate limits return 429** with a `retry_after` field telling the client when to retry.
- **All errors return JSON** — consistent format with the `err` field. Clients always know how to parse the response.

### From Implementations: Automatic Error Conversion

FicHub implements `From` for several error types, so the `?` operator works seamlessly:

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

This means you can write:

```rust
let data = tokio::fs::read(path).await?;  // io::Error → AppError::Internal
let row = sqlx::query_as!(…).fetch_one(&pool).await?;  // sqlx::Error → AppError::Database
let value: Value = serde_json::from_str(&text)?;  // serde_json::Error → AppError::Internal
```

The `?` operator automatically converts each error type to `AppError` using these `From` implementations. This keeps handler code clean — no manual error mapping needed.

### The AppResult Type Alias

To avoid writing `Result<T, AppError>` everywhere, FicHub defines a type alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Now you can write:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;  // sqlx::Error → AppError::Database
    Ok(row)
}
```

### Display for Errors

The `Display` trait provides a human-readable string representation:

```rust
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::BadRequest(code, msg) => write!(f, "BadRequest({}): {}", code, msg),
            AppError::RateLimited(retry_after) => write!(f, "RateLimited: retry after {}s", retry_after),
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

This is used when logging errors and in test assertions.

### Error Handling in Practice

Here's how errors flow through a typical handler:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // 1. Validate input → returns JSON error (not AppError)
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }
    
    // 2. Find scraper → AppError::BadRequest
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, 
            format!("unsupported URL: {}", query)))?;
    
    // 3. Scrape metadata → AppError::ScrapeError
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    
    // 4. Database operations → AppError::Database (via From impl)
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;
    
    // 5. Check blacklists → returns JSON (not an error)
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
    if !fic_blacklist.is_empty() {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }
    
    // 6. Generate export → AppError::ExportError
    let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await
        .map_err(|e| AppError::ExportError(e.to_string()))?;
    
    // 7. Return success
    Ok(Json(json!({"err": 0, "url_id": meta.url_id, ...})))
}
```

Every step either succeeds or produces a meaningful error. The `?` operator and `From` implementations make this flow clean and linear. Note that some "errors" (like blacklist hits) are actually normal responses — they return `Ok(Json(...))` with a non-zero `err` code.

⚠️ **Watch Out:** Don't use `.unwrap()` in handlers — it will panic and crash the server. Always use `.map_err()` or `?` to convert errors to `AppError`. The only exception is in startup code where a panic is acceptable (like `.expect("Failed to connect to database")`).

### Testing Errors

The error module includes tests for every variant:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_bad_request() {
        let err = AppError::BadRequest(400, "invalid input".into());
        let s = format!("{}", err);
        assert!(s.contains("BadRequest"));
        assert!(s.contains("400"));
        assert!(s.contains("invalid input"));
    }
    
    #[test]
    fn test_display_rate_limited() {
        let err = AppError::RateLimited(30);
        let s = format!("{}", err);
        assert!(s.contains("RateLimited"));
        assert!(s.contains("30"));
    }
    
    #[test]
    fn test_from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app: AppError = io.into();
        match app {
            AppError::Internal(msg) => assert!(msg.contains("file not found")),
            _ => panic!("expected Internal, got {:?}", app),
        }
    }
    
    #[test]
    fn test_from_sqlx_error() {
        let sqlx = sqlx::Error::Protocol("bad query".into());
        let app: AppError = sqlx.into();
        match app {
            AppError::Database(msg) => assert!(msg.contains("bad query")),
            _ => panic!("expected Database, got {:?}", app),
        }
    }
    
    #[test]
    fn test_from_redis_error() {
        let err = redis::RedisError::from((
            redis::ErrorKind::Io,
            "connection refused",
        ));
        let app: AppError = err.into();
        match app {
            AppError::CacheError(msg) => assert!(msg.contains("connection refused")),
            _ => panic!("expected CacheError, got {:?}", app),
        }
    }
    
    #[test]
    fn test_from_reqwest_error() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(10))
                .build()
                .unwrap()
                .get("http://127.0.0.1:1/")
                .send()
                .await
        });
        let app: AppError = match result {
            Err(e) => e.into(),
            Ok(_) => return,
        };
        match app {
            AppError::ScrapeError(_) => {}
            _ => panic!("expected ScrapeError, got {:?}", app),
        }
    }
    
    #[test]
    fn test_from_serde_json_error() {
        let json: Result<serde_json::Value, _> = serde_json::from_str("!invalid");
        let json_err = json.unwrap_err();
        let app: AppError = json_err.into();
        match app {
            AppError::Internal(_) => {}
            _ => panic!("expected Internal, got {:?}", app),
        }
    }
}
```

Testing error types ensures that the mapping from external error types to AppError works correctly. If someone changes a `From` implementation, these tests will catch it. The `#[cfg(test)]` attribute means these tests only compile when running `cargo test` — they're not included in the production binary.

---

## Chapter 7: Connecting to PostgreSQL

Time to connect to the database. FicHub uses SQLx, a Rust library that provides compile-time checked SQL queries. Let's walk through how the connection pool is initialized and why it matters.

### Why PostgreSQL?

PostgreSQL is the gold standard for relational databases. It's reliable, feature-rich, and handles everything from simple CRUD to full-text search. FicHub uses several PostgreSQL-specific features:

- **Full-text search** with `ts_rank` and `plainto_tsquery` — for the search system
- **GIN indexes** for fast text search — pre-built indexes for full-text queries
- **PL/pgSQL triggers** for automatic score updates — when a tag vote is inserted, the trigger updates the aggregate score
- **INSERT ON CONFLICT** (upsert) for idempotent operations — safe to retry
- **RETURNING** clauses to get inserted/updated values — avoid separate SELECT
- **INET type** for IP address storage — native IP address handling
- **CHECK constraints** for data validation — enforce business rules at the database level
- **Composite primary keys** — for tables like `fic_tags(url_id, tag_id)`

### The Connection Pool

FicHub doesn't create a new database connection for every request — that would be slow. Instead, it uses a connection pool:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

- **max_connections(20)** — Up to 20 simultaneous database connections. This balances concurrency with resource usage. Each connection uses memory on both the client and server.
- **acquire_timeout(10s)** — If all connections are busy, wait up to 10 seconds before failing. This prevents request timeouts during high load.
- **connect** — Establishes the pool, verifies connectivity, and returns the pool handle.

The pool manages connection lifecycle: checkout, use, return. You never manually open or close connections. When you call `sqlx::query(...).execute(&pool)`, the pool checks out a connection, runs the query, and returns the connection to the pool.

### Running Migrations

When the pool is created, FicHub automatically runs database migrations:

```rust
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
    } else {
        tracing::warn!("Migrations directory not found at {:?}", migrations_path);
    }
    
    Ok(pool)
}
```

The migration path logic looks for migrations next to the binary (for production — when you run the compiled binary) or in the project directory (for development — when you run `cargo run`). This means you can run `cargo run` in development and the migrations are found automatically.

SQLx tracks which migrations have been applied in a `_sqlx_migrations` table. Running `migrate run` again is safe — it only applies new migrations. Each migration runs in a transaction, so if it fails, the database is left unchanged.

### Understanding SQLx Queries

SQLx lets you write SQL directly in Rust code. The two main patterns are:

**`sqlx::query`** — For statements that don't return rows (INSERT, UPDATE, DELETE):

```rust
sqlx::query(
    r#"INSERT INTO fic_info (id, title, author, chapters, words)
       VALUES ($1, $2, $3, $4, $5)"#
)
.bind(&fic.id)
.bind(&fic.title)
.bind(&fic.author)
.bind(fic.chapters)
.bind(fic.words)
.execute(pool)
.await?;
```

**`sqlx::query_as`** — For queries that return rows (SELECT):

```rust
let fic: Option<FicInfo> = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1"
)
.bind(id)
.fetch_optional(pool)
.await?;
```

SQLx uses PostgreSQL's `$1, $2, ...` parameter syntax. Parameters are bound in order with `.bind()`. This prevents SQL injection — the database driver handles escaping.

### Query Methods

SQLx provides several fetch methods:

- `execute(pool)` — Run a statement, return row count (as `QueryResult`)
- `fetch_one(pool)` — Expect exactly one row (error if zero or more than one)
- `fetch_optional(pool)` — Zero or one row (returns `Option<T>`)
- `fetch_all(pool)` — Zero or more rows (returns `Vec<T>`)
- `fetch_scalar(pool)` — Fetch a single scalar value (like COUNT(*))

For `query_as`, the type parameter after `as` specifies the Rust type each row maps to:

```rust
// Map to a struct
let fic: FicInfo = sqlx::query_as::<_, FicInfo>("SELECT ...")
    .fetch_one(pool).await?;

// Map to a tuple
let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
    .fetch_one(pool).await?;

// Map to a scalar
let name: String = sqlx::query_scalar("SELECT title FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_one(pool).await?;

// Check existence
let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)")
    .bind(id)
    .fetch_one(pool).await?;
```

### Upserts: INSERT ON CONFLICT

One of FicHub's most common operations is the upsert — insert a row if it doesn't exist, update it if it does:

```rust
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
            updated = NOW()"#
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

The `EXCLUDED` keyword refers to the values that were going to be inserted. This is PostgreSQL's upsert syntax — clean, atomic, and efficient.

### Compile-Time Checking

SQLx can check your queries against the actual database at compile time. When you run `cargo build`, SQLx connects to the database and verifies that:
- Table names are correct
- Column names exist
- Parameter types match
- The query is syntactically valid

This catches typos and schema mismatches before your code runs. It's one of SQLx's most valuable features.

⚠️ **Watch Out:** Compile-time checking requires a running database during `cargo build`. If the database isn't available, SQLx falls back to runtime checking. Set `DATABASE_URL` in your environment. You can also use `sqlx prepare` to cache query metadata for offline builds.

### Using the Pool

Once initialized, the pool is stored in `AppState` and passed to all handlers:

```rust
let state = Arc::new(AppState {
    db: db_pool,  // sqlx::PgPool
    // ...
});
```

Handlers access it as `state.db`:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

// In a handler:
let fic = queries::get_fic_info(&state.db, url_id).await?;
```

The pool handles connection checkout and return automatically. Multiple handlers can query the database concurrently without interfering with each other. The pool's internal connection manager ensures thread safety.

### SQL Basics for FicHub

Here are the SQL operations FicHub uses most:

**SELECT with WHERE:**
```sql
SELECT * FROM fic_info WHERE id = $1
```

**SELECT with JOIN:**
```sql
SELECT t.id, t.name, t.tag_type_id, ft.score
FROM fic_tags ft
JOIN tags t ON t.id = ft.tag_id
WHERE ft.url_id = $1
```

**INSERT with RETURNING:**
```sql
INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id
```

**UPDATE:**
```sql
UPDATE fic_tag_votes SET value = $1, created_at = NOW()
WHERE url_id = $2 AND tag_id = $3 AND voter_ip = $4::inet
```

**DELETE:**
```sql
DELETE FROM tags WHERE id = $1
```

**Aggregate with GROUP BY:**
```sql
SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
FROM recommendation_suggestions s
JOIN recommendation_votes v ON v.suggestion_id = s.id
WHERE s.url_id = $1
GROUP BY s.suggested_url_id
```

**Full-text search:**
```sql
SELECT fi.*, ts_rank(fi.text_search, plainto_tsquery('english', $1)) AS rank
FROM fic_info fi
WHERE fi.text_search @@ plainto_tsquery('english', $1)
ORDER BY rank DESC
```

**EXISTS check:**
```sql
SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)
```

**CTE (Common Table Expression):**
```sql
WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
candidates AS (
  SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
         cooccur_count
  FROM fic_bookmark_cooccur
  WHERE work_a = $1 OR work_b = $1
)
SELECT ... FROM candidates c JOIN fic_works fw ...
```

Each of these patterns appears multiple times in `queries.rs`. The key is understanding that Rust and SQL work together — Rust handles the logic, SQL handles the data.

---

## Chapter 8: Database Migrations and Models

A database schema evolves over time. You add tables, columns, and indexes as your application grows. Migrations are how you track and apply those changes reliably.

### What Are Migrations?

A migration is a versioned SQL script that changes the database schema. Each migration has:
- A version number (determines order of application)
- An "up" script (applies the change)
- Sometimes a "down" script (reverts the change)

FicHub's migrations live in the `migrations/` directory:

```
migrations/
├── 001_initial_schema.sql
├── 002_recommender.sql
├── 003_tagging.sql
└── 004_shelves.sql
```

SQLx applies them in filename order. The `_sqlx_migrations` table tracks which have been applied.

### Migration 001: Core Schema

The first migration creates the foundational tables:

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

CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);

CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';

-- Fic metadata cache
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

-- Fic blacklist
CREATE TABLE IF NOT EXISTS fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);

-- Author blacklist
CREATE TABLE IF NOT EXISTS author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);

-- Version bump for cache invalidation
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT
);
```

Key design decisions:
- **`fic_info.id`** is a 128-character VARCHAR — it's a SHA-256 hash of the source ID and story ID, giving us a deterministic, compact identifier. No auto-increment IDs that might collide across restarts.
- **`export_log`** has a composite unique constraint on `(url_id, version, etype, input_hash)` — this is how the cache works. Different versions or input hashes produce different cache entries.
- **Timestamps** use `TIMESTAMPTZ` (timezone-aware) for consistency across time zones.
- **Indexes** are created on frequently queried columns to speed up lookups.

### Migration 002: Recommendation Engine

The recommendation system needs several tables:

```sql
-- Site-specific work metadata and favourite counts
CREATE TABLE IF NOT EXISTS fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
CREATE INDEX IF NOT EXISTS idx_fic_works_favouriter_count ON fic_works(favouriter_count);
CREATE INDEX IF NOT EXISTS idx_fic_works_site_domain ON fic_works(site_domain);

-- Tracks which user favourited which work
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
CREATE INDEX IF NOT EXISTS idx_fic_bookmarks_user ON fic_bookmarks(user_hash);
CREATE INDEX IF NOT EXISTS idx_fic_bookmarks_url ON fic_bookmarks(url_id);

-- Pairwise co-occurrence counts
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
CREATE INDEX IF NOT EXISTS idx_cooccur_a ON fic_bookmark_cooccur(work_a);
CREATE INDEX IF NOT EXISTS idx_cooccur_b ON fic_bookmark_cooccur(work_b);

-- Community-submitted recommendations
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);

-- Upvotes/downvotes on suggestions
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);

-- Precomputed recommendation cache
CREATE TABLE IF NOT EXISTS precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
CREATE INDEX IF NOT EXISTS idx_precomputed_url ON precomputed_recommendations(url_id, rank);
```

The `fic_bookmark_cooccur` table is the heart of the collaborative filtering system. The `CHECK (work_a < work_b)` constraint ensures each pair is stored exactly once (in alphabetical order). This halves storage and simplifies queries — you only need to check `work_a = X OR work_b = X`, not both orderings.

### Migration 003: Tagging System

The tag system is the most complex:

```sql
-- Tag types (fixed enum)
CREATE TABLE IF NOT EXISTS tag_types (
    id SMALLINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO tag_types (id, name) VALUES
    (1, 'fandom'), (2, 'character'), (3, 'relationship'),
    (4, 'freeform'), (5, 'warning'), (6, 'category'), (7, 'other')
ON CONFLICT (id) DO NOTHING;

-- Canonical tags
CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tags_type ON tags(tag_type_id);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);

-- Tag aliases
CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT NOT NULL UNIQUE COLLATE "C",
    canonical_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Fic-tag junction
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_fic_tags_url ON fic_tags(url_id);
CREATE INDEX IF NOT EXISTS idx_fic_tags_tag ON fic_tags(tag_id);

-- Tag votes
CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INTEGER NOT NULL,
    voter_ip INET NOT NULL,
    value SMALLINT NOT NULL CHECK (value IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id, voter_ip),
    FOREIGN KEY (url_id, tag_id) REFERENCES fic_tags(url_id, tag_id) ON DELETE CASCADE
);

-- Tag flags for curator review
CREATE TABLE IF NOT EXISTS tag_flags (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL,
    tag_id INTEGER NOT NULL,
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, tag_id, flagged_by_ip)
);
```

Notice the `COLLATE "C"` on tag names — this makes tag lookups case-sensitive ("Harry Potter" ≠ "harry potter"). This is important because "Harry Potter" and "harry potter" should be different tags in the tag system.

The tagging migration also creates a trigger that automatically updates scores:

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

CREATE TRIGGER trg_fic_tag_vote_insert
    AFTER INSERT ON fic_tag_votes
    FOR EACH ROW
    EXECUTE FUNCTION update_fic_tag_score();

CREATE TRIGGER trg_fic_tag_vote_update
    AFTER UPDATE ON fic_tag_votes
    FOR EACH ROW
    EXECUTE FUNCTION update_fic_tag_score();

CREATE TRIGGER trg_fic_tag_vote_delete
    AFTER DELETE ON fic_tag_votes
    FOR EACH ROW
    EXECUTE FUNCTION update_fic_tag_score();
```

This trigger automatically updates `fic_tags.score` whenever a vote is inserted, updated, or deleted. No Rust code needed — the database handles the aggregate calculation atomically.

### Migration 004: OPDS Shelves

```sql
-- OPDS shelves for shared reading lists
CREATE TABLE IF NOT EXISTS opds_shelves (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS opds_shelf_items (
    shelf_id INTEGER NOT NULL REFERENCES opds_shelves(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, url_id)
);

-- Insert a default shelf
INSERT INTO opds_shelves (id, name, description, token)
VALUES (1, 'Default Shelf', 'Shared reading list', 'fichub')
ON CONFLICT (id) DO NOTHING;
```

Shelves are protected by a shared token. Users must include the token in requests to access or modify a shelf.

### Rust Models: FromRow Derive

FicHub's Rust structs map directly to database tables using `sqlx::FromRow`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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

Each field maps to a database column. Nullable columns use `Option<T>`. Non-nullable columns use `T` directly. The `FromRow` derive macro generates the code that converts a database row into this struct.

Other models follow the same pattern:

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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicBlacklist {
    pub url_id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}

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

### Schema Design Principles

FicHub's schema follows several principles:

1. **Deterministic IDs** — `fic_info.id` is derived from the source site and story ID, not auto-generated. This makes upserts idempotent — the same story always gets the same ID.

2. **Audit timestamps** — Every table has `created` and/or `updated` columns. This helps with debugging, cache invalidation, and data analysis.

3. **Cascading deletes** — When a fic is deleted, all associated bookmarks, co-occurrences, tags, and votes are automatically removed via `ON DELETE CASCADE`.

4. **Index coverage** — Every frequently queried column has an index. Look at the migration for `CREATE INDEX` statements. Indexes speed up reads at the cost of slower writes.

5. **Type safety** — PostgreSQL types match Rust types: `VARCHAR(128)` → `String`, `INT4` → `i32`, `INT8` → `i64`, `INET` → `IpAddr`, `TIMESTAMPTZ` → `DateTime<Utc>`.

6. **Constraints** — CHECK constraints enforce business rules at the database level (e.g., `CHECK (work_a < work_b)`, `CHECK (vote IN (-1, 1))`).

7. **Unique constraints** — Prevent duplicate data (e.g., `UNIQUE(url_id, version, etype, input_hash)` on export_log).

🧪 **Try It Yourself:** Connect to the fichub database and explore the schema:

```bash
psql -h localhost -U fichub -d fichub -c "\dt"  # List all tables
psql -h localhost -U fichub -d fichub -c "\d fic_info"  # Describe fic_info
psql -h localhost -U fichub -d fichub -c "\d fic_tags"  # Describe fic_tags
psql -h localhost -U fichub -d fichub -c "SELECT COUNT(*) FROM fic_info;"  # Count fics
```

The `\d` command shows the table structure including columns, types, constraints, and indexes. This is invaluable for understanding the schema.


---

# Part 3: Web Scraping

---

## Chapter 11: Understanding Fanfiction Sites

Before we write a single line of scraper code, we need to understand what we're scraping. Fanfiction sites have wildly different structures — some use semantic HTML with clear classes, others use deeply nested divs with generated class names. Knowing the landscape helps us build resilient scrapers.

### The Fanfiction Landscape

FicHub currently supports six sites, each with its own quirks:

1. **Archive of Our Own (AO3)** — The most popular archive, run by the Organization for Transformative Works. Uses clean, semantic HTML with predictable class names. Full work view returns all chapters in a single page.

2. **FanFiction.net (FF.net)** — One of the oldest fanfiction sites (founded 1998). Uses a mix of semantic and non-semantic HTML. Each chapter requires a separate HTTP request.

3. **FictionPress** — FF.net's sister site for original fiction. Shares FF.net's HTML structure, so the same scraper handles both.

4. **SpaceBattles / SufficientVelocity / QuestionableQuesting** — XenForo-based forums where many stories are posted as thread replies. More complex scraping because stories are spread across forum pages.

5. **Adult FanFiction** — A smaller site with its own HTML structure.

6. **HPFanFic** — A Harry Potter-focused archive.

### How AO3 Is Structured

AO3 is the easiest to scrape because it uses clean, semantic HTML. Here's the structure of a typical story page:

```html
<div id="main" role="main">
  <div class="work" id="work_123456">
    <!-- Title -->
    <h2 class="title heading">
      <a href="/works/123456" rel="bookmark">My Amazing Story</a>
    </h2>
    
    <!-- Byline -->
    <h3 class="byline heading">
      by <a rel="author" href="/users/AuthorName/pseuds/AuthorName">AuthorName</a>
    </h3>
    
    <!-- Summary -->
    <div id="summary" class="summary module">
      <h2 class="heading">Summary</h2>
      <blockquote class="userstuff">
        <p>A wonderful story about adventure and friendship.</p>
      </blockquote>
    </div>
    
    <!-- Chapters -->
    <div class="chapter" id="chapter-1">
      <h3 class="heading">
        <a href="/works/123456/chapters/111111">Chapter 1</a>
      </h3>
      <div class="userstuff">
        <p>Once upon a time, in a land far away...</p>
        <p>The adventure began on a Tuesday morning.</p>
      </div>
    </div>
    
    <div class="chapter" id="chapter-2">
      <h3 class="heading">
        <a href="/works/123456/chapters/222222">Chapter 2</a>
      </h3>
      <div class="userstuff">
        <p>The next day brought unexpected challenges.</p>
      </div>
    </div>
    
    <!-- Stats -->
    <dl class="stats">
      <dt>Chapters:</dt>
      <dd class="chapters">2/2</dd>
      <dt>Words:</dt>
      <dd class="words">10,234</dd>
      <dt>Comments:</dt>
      <dd class="comments">45</dd>
      <dt>Kudos:</dt>
      <dd class="kudos">1,234</dd>
      <dt>Bookmarks:</dt>
      <dd class="bookmarks">567</dd>
      <dt>Hits:</dt>
      <dd class="hits">12,345</dd>
      <dt>Status:</dt>
      <dd class="status">Complete</dd>
    </dl>
  </div>
</div>
```

Key observations:
- The story title is in `<h2 class="title heading">`
- The author is in `<a rel="author">`
- Each chapter is wrapped in `<div class="chapter">`
- Chapter titles are in `<h3 class="heading">`
- Chapter content is in `<div class="userstuff">`
- Stats (words, chapters, kudos) are in `<dd>` elements with descriptive classes
- The full work URL uses `?view_full_work=true` to get all chapters

### How FF.net Is Structured

FF.net is more challenging:

```html
<div id="profile_top">
  <b class="xcontrast_txt">My Story Title</b>
  <a class="xcontrast_txt" href="/u/12345/AuthorName">AuthorName</a>
  <div class="xcontrast_txt">Summary goes here...</div>
  <span class="xgray">
    Rated: T | English | Words: 10,234 | Chapters: 5 | Reviews: 100
  </span>
  <span data-xutitle="word count">10,234</span>
  <span data-xutitle="chapters">5</span>
</div>

<div class="storytext" id="storytext">
  <p>Chapter content here...</p>
</div>
```

And for multi-chapter works, the chapter selector:

```html
<select id="chap_select">
  <option value="1" selected>1. Chapter One</option>
  <option value="2">2. Chapter Two</option>
  <option value="3">3. Chapter Three</option>
</select>
```

Key observations:
- Title is in `<b class="xcontrast_txt">` inside `#profile_top`
- Author is in `<a class="xcontrast_txt">` inside `#profile_top`
- Word count uses `data-xutitle` attributes
- Chapter content is in `<div class="storytext">`
- Chapter titles come from the `<select>` dropdown
- Each chapter has a separate URL: `/s/123456/1/`, `/s/123456/2/`, etc.

### How XenForo Forums Work

XenForo-based forums (SpaceBattles, SV, QQ) use a forum thread structure:

```html
<h1 class="p-title-value">Story Title [Worm, Star Wars]</h1>

<div class="p-body-inner">
  <article class="message" data-author="AuthorName">
    <div class="message-body">
      <div class="bbWrapper">
        <p>Story content here...</p>
      </div>
    </div>
  </article>
  
  <article class="message" data-author="AuthorName">
    <div class="message-body">
      <div class="bbWrapper">
        <p>More story content...</p>
      </div>
    </div>
  </article>
</div>
```

Key observations:
- Thread title is in `<h1 class="p-title-value">`
- Each post (potential chapter) is in `<article class="message-body">`
- The first post typically contains the story summary
- Subsequent posts may contain chapter content
- Pagination is handled through forum page parameters (`?page=2`)
- Not every post is story content — some are comments or discussion

### URL Patterns

Each site has distinct URL patterns:

```
AO3:          https://archiveofourown.org/works/123456
              https://archiveofourown.org/works/123456/chapters/789012
              https://archiveofourown.org/works/123456?view_full_work=true

FF.net:       https://www.fanfiction.net/s/123456/1/
              https://www.fanfiction.net/s/123456/5/

XenForo:      https://forums.spacebattles.com/threads/story-name.12345/
              https://forums.spacebattles.com/threads/story-name.12345/page-2

FictionPress: https://www.fictionpress.com/s/123456/1/
```

### The SiteScraper Trait

FicHub defines a common interface that all scrapers implement:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait SiteScraper: Send + Sync {
    /// Returns true if this scraper can handle the given URL
    fn can_handle(&self, url: &str) -> bool;
    
    /// Extract metadata from a story URL
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    
    /// Fetch all chapters given metadata
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    
    /// Extract structured tags from a fic URL (optional, default empty)
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

Three required methods and one optional:
- **`can_handle`** — Does this scraper know how to handle this URL? Simple string matching (e.g., `url.contains("archiveofourown.org")`).
- **`lookup`** — Fetch metadata (title, author, word count, status, etc.) without fetching chapter content.
- **`fetch_chapters`** — Fetch all chapter content. For AO3, this is one HTTP request. For FF.net, it's one request per chapter.
- **`extract_tags`** — Optionally extract structured tags. Only AO3 implements this (default returns empty vec).

The `Send + Sync` bounds are required for async traits that will be shared across threads.

### Data Structures

Every scraper produces the same data types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,           // Deterministic hash ID
    pub title: String,            // Story title
    pub author: String,           // Author name
    pub chapters: i32,            // Number of chapters
    pub words: i64,               // Word count
    pub desc: String,             // Story description/summary
    pub published: i64,           // Publication date (unix millis)
    pub updated: i64,             // Last update date (unix millis)
    pub status: String,           // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,           // Original URL
    pub source_id: i64,           // Site identifier (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Site-specific author ID
    pub author_url: String,       // Author profile URL
    pub author_local_id: String,  // Site-specific story/work ID
    pub content_hash: Option<String>,  // SHA-256 of content
    pub extra_meta: Option<String>,    // Additional metadata (JSON)
    pub raw_extended_meta: Option<String>,  // Raw metadata from site
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,     // Chapter number (1-indexed)
    pub title: String,       // Chapter title
    pub content: String,     // HTML content
}
```

The `url_id` is particularly important — it's a deterministic hash generated from the source ID and story ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])
}
```

This produces a 12-character hex string like `a1b2c3d4e5f6`. It's deterministic — the same source_id and story_id always produce the same url_id — and compact enough for database primary keys.

### Error Handling for Scraping

Scrapers can fail in several ways:

```rust
pub enum ScrapeError {
    NotFound,
    Blocked,
    Network(String),
    ParseError(String),
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::NotFound => write!(f, "fic not found"),
            ScrapeError::Blocked => write!(f, "blocked by site"),
            ScrapeError::Network(e) => write!(f, "network error: {}", e),
            ScrapeError::ParseError(e) => write!(f, "parse error: {}", e),
        }
    }
}

impl std::error::Error for ScrapeError {}
```

- **NotFound** — The story doesn't exist or has been deleted. Maps to HTTP 404.
- **Blocked** — The site is blocking our requests (rate limited, captcha, etc.). Maps to HTTP 502.
- **Network** — HTTP request failed (timeout, connection error, DNS failure). Maps to HTTP 502.
- **ParseError** — The HTML structure changed and we couldn't extract data. Maps to HTTP 502.

### Polite Scraping

FicHub follows ethical scraping practices:

1. **User-Agent header** — All requests include `User-Agent: fichub.net/0.1.0` so the upstream site knows who's making requests
2. **Rate limiting** — Token bucket limits prevent overwhelming upstream sites (30 req/sec global, ~1 req/8.6s per IP)
3. **Caching** — Once fetched, data is cached locally to avoid re-fetching
4. **Graceful failure** — If a site is down, we return a clear error instead of retrying aggressively
5. **No login bypass** — We don't attempt to access restricted content

### The Request Pattern

All scrapers follow the same HTTP request pattern:

```rust
let response = client
    .get(url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;

if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}

let html = response.text().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

The `.map_err()` converts reqwest errors to ScrapeError, preserving the error message for debugging.

🧪 **Try It Yourself:** Open your browser's developer tools (F12) and visit an AO3 story. Look at the HTML structure — find the `<h2 class="title heading">` element and the `<div class="userstuff">` elements. Try to identify the selectors for:
- Title
- Author
- Word count
- Chapter content

---

## Chapter 12: Building Web Scrapers

Let's build the actual scraping infrastructure. We'll use `reqwest` for HTTP requests and `scraper` for HTML parsing. By the end of this chapter, you'll understand how FicHub fetches and parses fanfiction sites.

### The HTTP Client

FicHub creates a single HTTP client and reuses it for all requests:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

Key settings:
- **User-Agent** — Identifies our scraper (sites may block unknown user agents)
- **Timeout** — 30-second timeout prevents hanging on slow responses
- **Connection pooling** — reqwest automatically pools connections (reuses TCP connections)

The client is stored in `AppState` and passed to every scraper call. This means all scrapers share the same connection pool, which is efficient.

### The reqwest Crate

Making HTTP requests with reqwest is straightforward:

```rust
// Simple GET request
let response = client.get(url).send().await?;

// With headers
let response = client
    .get(url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await?;

// Check status
if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}

// Get body as text
let html = response.text().await?;

// Check specific status codes
match response.status().as_u16() {
    200 => { /* success */ }
    403 => return Err(ScrapeError::Blocked),
    404 => return Err(ScrapeError::NotFound),
    429 => return Err(ScrapeError::Blocked),
    _ => return Err(ScrapeError::Network(format!("HTTP {}", response.status()))),
}
```

reqwest automatically handles:
- Connection pooling (reuses TCP connections)
- Redirects (follows up to 10 by default)
- TLS (using rustls — no OpenSSL dependency)
- Compression (gzip, deflate)

### The scraper Crate

The `scraper` crate parses HTML and lets you query it with CSS selectors:

```rust
use scraper::{Html, Selector};

// Parse HTML string into a document
let document = Html::parse_document(&html);

// Create a CSS selector
let selector = Selector::parse("h2.title.heading").unwrap();

// Find the first matching element
let title = document
    .select(&selector)
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

Key methods:
- `Html::parse_document(&html)` — Parse HTML into a document tree. Use `parse_fragment` for partial HTML.
- `Selector::parse("css-selector")` — Create a CSS selector. Returns a Result because invalid selectors are caught at runtime.
- `.select(&selector)` — Find all matching elements. Returns an iterator.
- `.next()` — Get the first match.
- `.text().collect::<String>()` — Get all text content (strips HTML tags).
- `.inner_html()` — Get the inner HTML (including child tags).
- `.html()` — Get the outer HTML (including the element itself).
- `.value().attr("href")` — Get an attribute value.

### Building a Complete Scraper

Let's walk through the AO3 scraper as a complete example:

```rust
pub struct Ao3Scraper;

impl Ao3Scraper {
    fn extract_work_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
    
    fn extract_chapter_number(url: &str) -> Option<i32> {
        let re = regex_lite::Regex::new(r"/chapters/(\d+)").ok()?;
        re.captures(url)?.get(1).and_then(|m| m.as_str().parse().ok())
    }
}

const BASE_URL: &str = "https://archiveofourown.org";

#[async_trait]
impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // 1. Extract work ID from URL
        let work_id = Self::extract_work_id(url)
            .ok_or_else(|| ScrapeError::ParseError(
                "could not extract work ID from AO3 URL".into()
            ))?;
        
        // 2. Build the full work URL
        let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
        
        // 3. Fetch the page
        let response = client
            .get(&fic_url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        // 4. Parse the HTML
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // 5. Extract metadata
        let title = document
            .select(&Selector::parse("h2.title.heading").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        let author = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        
        let author_url = document
            .select(&Selector::parse("a[rel='author']").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("{BASE_URL}{h}"))
            .unwrap_or_default();
        
        let description = document
            .select(&Selector::parse("blockquote.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        // Extract stats
        let stats_text: String = document
            .select(&Selector::parse("dd.chapters").unwrap())
            .next()
            .map(|el| el.text().collect())
            .unwrap_or_default();
        
        let chapters = if let Some(pos) = stats_text.find('/') {
            stats_text[..pos].trim().parse().unwrap_or(1)
        } else {
            1
        };
        
        let words = document
            .select(&Selector::parse("dd.words").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);
        
        let status_text = document
            .select(&Selector::parse("dd.status").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        
        let status = if status_text.contains("Complete") {
            "complete".to_string()
        } else {
            "ongoing".to_string()
        };
        
        // 6. Generate url_id and build response
        let url_id = crate::scrape::generate_url_id(1, &work_id);
        let now = Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters,
            words,
            desc: description,
            published: now,
            updated: now,
            status,
            source: fic_url,
            source_id: 1,  // AO3 source ID
            author_id: 0,
            author_url,
            author_local_id: work_id.clone(),
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
        // Fetch the full work page
        let url = format!(
            "{BASE_URL}/works/{}?view_full_work=true",
            meta.author_local_id
        );
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let chapter_sel = Selector::parse("div.chapter").unwrap();
        let title_sel = Selector::parse("h3.title").unwrap();
        let content_sel = Selector::parse("div.userstuff").unwrap();
        
        let mut chapters = Vec::new();
        for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
            let chapter_title = chapter_div
                .select(&title_sel)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| format!("Chapter {}", i + 1));
            
            let content = chapter_div
                .select(&content_sel)
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
            
            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: chapter_title,
                content,
            });
        }
        
        // Fallback for single-chapter works
        if chapters.is_empty() {
            if let Some(body) = document
                .select(&Selector::parse("div.userstuff").unwrap())
                .next()
            {
                let content = body.inner_html();
                if !content.is_empty() {
                    chapters.push(Chapter {
                        chapter_id: 1,
                        title: meta.title.clone(),
                        content,
                    });
                }
            }
        }
        
        Ok(chapters)
    }
}
```

### Regex for URL Parsing

Each scraper uses regex to extract IDs from URLs:

```rust
// AO3: /works/123456 or /works/123456/chapters/789012
let re = regex_lite::Regex::new(r"/works/(\d+)")?;

// FF.net: /s/123456 or /s/123456/5/
let re = regex_lite::Regex::new(r"/s/(\d+)")?;

// XenForo: /threads/story-name.12345/
let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?")?;

// HPFanFic: /story/123456
let re = regex_lite::Regex::new(r"/story/(\d+)")?;
```

The `regex_lite` crate is a lightweight regex library — smaller and faster than the full `regex` crate, with all the features we need (capturing groups, character classes, quantifiers).

### Error Mapping

Every external error gets mapped to `ScrapeError`:

```rust
let response = client.get(&url).send().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;

let html = response.text().await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

The `.map_err()` method converts `reqwest::Error` to `ScrapeError::Network`, preserving the error message for debugging. This pattern is used throughout the scraper code.

### Text vs HTML

Sometimes you want plain text, sometimes you want HTML:

```rust
// Plain text (strips all tags, concatenates text nodes)
let text: String = element.text().collect::<String>();
// Result: "Once upon a time..."

// Inner HTML (preserves child tags)
let html = element.inner_html();
// Result: "<p>Once upon a time...</p><p>More text...</p>"

// Outer HTML (includes the element itself)
let html = element.html();
// Result: "<div class="userstuff"><p>Once upon a time...</p></div>"
```

For chapter content, we want `inner_html()` because the content may contain formatting tags (paragraphs, bold, italics, blockquotes, images) that we want to preserve in the EPUB output.

### Testing Scrapers

You can test scrapers with captured HTML:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_work_id() {
        assert_eq!(
            Ao3Scraper::extract_work_id("https://archiveofourown.org/works/123456"),
            Some("123456".to_string())
        );
    }
    
    #[test]
    fn test_extract_work_id_with_chapters() {
        assert_eq!(
            Ao3Scraper::extract_work_id(
                "https://archiveofourown.org/works/123456/chapters/789012"
            ),
            Some("123456".to_string())
        );
    }
    
    #[test]
    fn test_extract_work_id_invalid_url() {
        assert_eq!(
            Ao3Scraper::extract_work_id("https://example.com/page"),
            None
        );
    }
    
    #[test]
    fn test_can_handle_ao3() {
        assert!(Ao3Scraper.can_handle("https://archiveofourown.org/works/123"));
        assert!(!Ao3Scraper.can_handle("https://fanfiction.net/s/123"));
    }
}
```

🧪 **Try It Yourself:** Write a small Rust program that fetches an AO3 story page and prints the title and author:

```rust
use scraper::{Html, Selector};

#[tokio::main]
async fn main() {
    let url = "https://archiveofourown.org/works/123456?view_full_work=true";
    let html = reqwest::get(url).await.unwrap().text().await.unwrap();
    let doc = Html::parse_document(&html);
    
    let title = doc.select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    let author = doc.select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    
    println!("Title: {}", title.trim());
    println!("Author: {}", author.trim());
}
```

---

## Chapter 13: The Scraper Registry

The scraper registry is a beautiful example of the strategy pattern in Rust. It holds all available scrapers and automatically finds the right one for any URL.

### The Registry Pattern

Instead of writing a big `if/else` chain to determine which scraper to use, FicHub stores scrapers in a vector and iterates to find the match:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

Each scraper is a `Box<dyn SiteScraper>` — a boxed trait object. This lets us store different types (Ao3Scraper, FfNetScraper, etc.) in the same collection. The `Box` allocates the scraper on the heap and the `dyn` makes it a dynamic dispatch.

### Creating the Registry

```rust
impl ScraperRegistry {
    pub fn new() -> Self {
        let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
        scrapers.push(Box::new(sites::ao3::Ao3Scraper));
        scrapers.push(Box::new(sites::ffnet::FfNetScraper));
        scrapers.push(Box::new(sites::xenforo::XenForoScraper));
        scrapers.push(Box::new(sites::fictionpress::FictionPressScraper));
        scrapers.push(Box::new(sites::adultfanfiction::AdultFanFictionScraper));
        scrapers.push(Box::new(sites::hpfanfic::HpFanFicScraper));
        ScraperRegistry { scrapers }
    }
}
```

Notice that each scraper is a unit struct (no fields). The scraper logic is in the `impl SiteScraper for XScraper` block. Unit structs are zero-sized — they take up no memory. The trait object is just a fat pointer (data pointer + vtable pointer).

### Finding a Scraper

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This iterates through all scrapers and calls `can_handle(url)` on each. The first one that returns `true` is used. This is O(n) but n is small (6 scrapers), so it's negligible.

The `can_handle` implementations are simple string matching:
- Ao3Scraper: `url.contains("archiveofourown.org")`
- FfNetScraper: `url.contains("fanfiction.net") || url.contains("fictionpress.com")`
- XenForoScraper: checks against a list of XenForo domains

### Delegating to the Found Scraper

Once we find the right scraper, we delegate the work:

```rust
pub async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    match self.find_scraper(url) {
        Some(scraper) => scraper.lookup(client, url).await,
        None => Err(ScrapeError::NotFound),
    }
}

pub async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    match self.find_scraper(&meta.source) {
        Some(scraper) => scraper.fetch_chapters(client, meta).await,
        None => Err(ScrapeError::NotFound),
    }
}
```

The `lookup` method takes a URL, finds the right scraper, and calls `lookup` on it. If no scraper matches, it returns `ScrapeError::NotFound`.

### Arc Pattern for Shared Access

The registry is stored as `Arc<ScraperRegistry>` in `AppState`:

```rust
let scraper_registry = Arc::new(ScraperRegistry::new());
```

`Arc` (Atomic Reference Counting) allows multiple async tasks to share ownership of the registry without copying it. This is essential because handlers run concurrently and all need access to the same registry. Without Arc, each handler would need its own copy, which is wasteful and incorrect (they should all share the same registry).

### Adding a New Scraper

Adding support for a new site requires three steps:

1. **Create a new file** in `src/scrape/sites/your_site.rs`:

```rust
use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};

pub struct NewSiteScraper;

#[async_trait]
impl SiteScraper for NewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("newsite.example.com")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let response = client.get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // Extract story ID from URL
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok();
        let story_id = re
            .and_then(|r| r.captures(url))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        
        // Extract metadata using CSS selectors
        let title = document
            .select(&Selector::parse("h1.story-title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        // ... extract more fields ...
        
        let url_id = crate::scrape::generate_url_id(7, &story_id);
        let now = chrono::Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters,
            words,
            desc: description,
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 7,
            author_id: 0,
            author_url: String::new(),
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // Fetch and parse chapters
        let mut chapters = Vec::new();
        // ... implementation ...
        Ok(chapters)
    }
}
```

2. **Register in `src/scrape/sites/mod.rs`**:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod your_site;  // Add this line
```

3. **Add to registry** in `src/scrape/registry.rs`:

```rust
scrapers.push(Box::new(sites::your_site::NewSiteScraper));
```

That's it! The registry automatically handles URL routing.

### The FictionPress Shortcut

FictionPress shares the same HTML structure as FF.net (they're run by the same people). Instead of duplicating code, FicHub uses a type alias:

```rust
// In fictionpress.rs
pub use FfNetScraper as FictionPressScraper;
```

This means FictionPress uses FF.net's scraper logic but is registered as a separate scraper. The `can_handle` method on `FfNetScraper` already checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

This is a great example of code reuse — we get support for FictionPress for free by recognizing the shared structure.

### Error Handling in the Registry

If no scraper can handle a URL, the registry returns `ScrapeError::NotFound`:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

The caller handles the `None` case with a clear error message:

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(
        -5, 
        format!("unsupported URL: {}", query)
    ))?;
```

This gives users a clear error message: "unsupported URL." The error code `-5` helps clients handle this programmatically.

### Performance Considerations

The registry is created once at startup and shared across all requests. There's no per-request allocation or setup. The `find_scraper` call is a simple linear scan with cheap string matching (`.contains()`).

For a production system with many more scrapers, you could optimize with:
- A `HashMap<&str, Box<dyn SiteScraper>>` keyed on domain
- A trie for URL prefix matching
- Compile-time registration with a proc macro

But for 6 scrapers, the simple approach is perfect. The linear scan completes in nanoseconds — far faster than the HTTP requests that follow.

⚠️ **Watch Out:** If two scrapers match the same URL, the first one in the vector wins. Make sure your `can_handle` implementations don't overlap. For example, FictionPress URLs contain "fictionpress.com" but NOT "fanfiction.net", so they're correctly handled by the FfNetScraper's combined check.

---

## Chapter 14: AO3 Scraper Deep Dive

Let's do a complete walkthrough of the AO3 scraper — FicHub's most important scraper. We'll examine every CSS selector, every extraction step, and the fallback logic.

### AO3 Page Structure

When you visit `https://archiveofourown.org/works/123456?view_full_work=true`, the HTML looks roughly like this:

```html
<div id="main" role="main">
  <div class="work" id="work_123456">
    <h2 class="title heading">
      <a href="/works/123456" rel="bookmark">The Story Title</a>
    </h2>
    <h3 class="byline heading">
      <a rel="author" href="/users/AuthorName/pseuds/AuthorName">AuthorName</a>
    </h3>
    <div id="summary" class="summary module">
      <h2 class="heading">Summary</h2>
      <blockquote class="userstuff">
        <p>A wonderful story about...</p>
      </blockquote>
    </div>
    <div class="chapter" id="chapter-1">
      <h3 class="heading">Chapter 1: The Beginning</h3>
      <div class="userstuff">
        <p>Once upon a time...</p>
      </div>
    </div>
    <div class="chapter" id="chapter-2">
      <h3 class="heading">Chapter 2: The Middle</h3>
      <div class="userstuff">
        <p>Meanwhile...</p>
      </div>
    </div>
    <dl class="stats">
      <dt>Chapters:</dt>
      <dd class="chapters">2/2</dd>
      <dt>Words:</dt>
      <dd class="words">10,000</dd>
      <dt>Status:</dt>
      <dd class="status">Complete</dd>
    </dl>
  </div>
</div>
```

### Extracting the Work ID

AO3 URLs follow the pattern `/works/NUMBER` or `/works/NUMBER/chapters/NUMBER`. We extract the work ID with regex:

```rust
fn extract_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The `\d+` matches one or more digits. The capturing group `(...)` extracts just the number. The `ok()?` handles the case where the regex is invalid (it shouldn't be, but Rust requires handling this).

### Extracting the Title

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

- `h2.title.heading` — An `<h2>` with both classes `title` and `heading`
- `.next()` — Get the first match (there should only be one title)
- `.text().collect::<String>()` — Gets all text content (including text from child elements like `<a>`)
- `.trim()` — Removes leading/trailing whitespace
- Fallback to "Unknown Title" if not found

### Extracting the Author

```rust
let author = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Author".to_string());
```

The `a[rel='author']` selector finds anchor tags with the `rel="author"` attribute — a semantic HTML pattern for author attribution. AO3 uses this consistently across all story pages.

### Extracting Author URL

```rust
let author_url = document
    .select(&Selector::parse("a[rel='author']").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"))
    .map(|h| format!("{BASE_URL}{h}"))
    .unwrap_or_default();
```

The `attr("href")` method gets the href attribute value. AO3 uses relative URLs (`/users/AuthorName`), so we prepend the base URL to make it absolute.

### Extracting Description

```rust
let description = document
    .select(&Selector::parse("blockquote.userstuff").unwrap())
    .next()
    .map(|el| el.inner_html())
    .unwrap_or_default();
```

We use `inner_html()` instead of `text()` because the description may contain HTML formatting (paragraphs, emphasis, links, blockquotes) that we want to preserve. The EPUB generation will handle this HTML content.

### Extracting Word Count

```rust
let words = document
    .select(&Selector::parse("dd.words").unwrap())
    .next()
    .and_then(|el| {
        el.text().collect::<String>().replace(',', "").parse().ok()
    })
    .unwrap_or(0);
```

- `dd.words` — The `<dd>` element with class `words`
- `.replace(',', "")` — Remove thousands separators (e.g., "10,000" → "10000")
- `.parse().ok()` — Parse as i64, returning None on failure
- `.unwrap_or(0)` — Default to 0 if parsing fails

### Extracting Chapter Count

```rust
let stats_text: String = document
    .select(&Selector::parse("dd.chapters").unwrap())
    .next()
    .map(|el| el.text().collect())
    .unwrap_or_default();

let chapters = if let Some(pos) = stats_text.find('/') {
    stats_text[..pos].trim().parse().unwrap_or(1)
} else {
    1
};
```

AO3 shows chapters as "current/total" (e.g., "5/10"). We take the part before the `/` for the current count. If there's no `/`, it's a single chapter.

### Extracting Status

```rust
let status_text = document
    .select(&Selector::parse("dd.status").unwrap())
    .next()
    .map(|el| el.text().collect::<String>())
    .unwrap_or_default();

let status = if status_text.contains("Complete") {
    "complete".to_string()
} else {
    "ongoing".to_string()
};
```

AO3 shows "Complete" or "In Progress" in the status field. We map these to our internal status strings.

### Fetching Chapter Content

The `fetch_chapters` method fetches the full work view and extracts all chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
    -> Result<Vec<Chapter>, ScrapeError> 
{
    let url = format!(
        "{BASE_URL}/works/{}?view_full_work=true",
        meta.author_local_id
    );
    let response = client.get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    
    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
    
    let chapter_sel = Selector::parse("div.chapter").unwrap();
    let title_sel = Selector::parse("h3.title").unwrap();
    let content_sel = Selector::parse("div.userstuff").unwrap();
    
    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));
        
        let content = chapter_div
            .select(&content_sel)
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }
    
    // Fallback for single-chapter works
    if chapters.is_empty() {
        if let Some(body) = document
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
        {
            let content = body.inner_html();
            if !content.is_empty() {
                chapters.push(Chapter {
                    chapter_id: 1,
                    title: meta.title.clone(),
                    content,
                });
            }
        }
    }
    
    Ok(chapters)
}
```

The `?view_full_work=true` parameter is critical — it tells AO3 to return all chapters in a single page, rather than paginating. This makes scraping much easier and faster.

### Content Hash Calculation

After scraping, FicHub can compute a hash of the content to detect changes:

```rust
use sha2::{Sha256, Digest};

fn compute_content_hash(chapters: &[Chapter]) -> String {
    let mut hasher = Sha256::new();
    for chapter in chapters {
        hasher.update(chapter.content.as_bytes());
    }
    hex::encode(hasher.finalize())
}
```

This hash is stored in `fic_info.content_hash` and used for cache invalidation. If the content changes (author updates the story), the hash changes, and the cached export is regenerated.

### Edge Cases

The AO3 scraper handles several edge cases:

1. **Deleted works** — Returns 404, which maps to `ScrapeError::NotFound`
2. **Restricted works** — Returns 403, which maps to `ScrapeError::NotFound` (we can't access them)
3. **Single-chapter works** — No `div.chapter` elements, falls back to reading the whole page
4. **Missing metadata** — Each field has a fallback default ("Unknown Title", "Unknown Author", 0 words)
5. **Encoding issues** — `scraper` handles UTF-8 automatically
6. **Multi-chapter works** — Properly iterates through all `div.chapter` elements
7. **Chapter titles with special characters** — Collected as text, no HTML interpretation

### Testing AO3 Extraction

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_work_id_standard() {
        assert_eq!(
            Ao3Scraper::extract_work_id("https://archiveofourown.org/works/123456"),
            Some("123456".to_string())
        );
    }
    
    #[test]
    fn test_extract_work_id_with_chapters() {
        assert_eq!(
            Ao3Scraper::extract_work_id("https://archiveofourown.org/works/123456/chapters/789012"),
            Some("123456".to_string())
        );
    }
    
    #[test]
    fn test_extract_work_id_no_match() {
        assert_eq!(Ao3Scraper::extract_work_id("https://example.com"), None);
    }
    
    #[test]
    fn test_can_handle() {
        assert!(Ao3Scraper.can_handle("https://archiveofourown.org/works/123"));
        assert!(Ao3Scraper.can_handle("http://archiveofourown.org/works/123"));
        assert!(!Ao3Scraper.can_handle("https://fanfiction.net/s/123"));
        assert!(!Ao3Scraper.can_handle("https://example.com"));
    }
}
```

🧪 **Try It Yourself:** Inspect an AO3 story page in your browser. Open the Elements tab and search for `div.chapter` — you'll see each chapter's container. Then search for `div.userstuff` inside each chapter to find the content. Try modifying the CSS selector and see what else you can find.

---

## Chapter 15: Other Site Scrapers

Now let's look at how the other scrapers work. Each one adapts to its site's unique HTML structure while producing the same output types.

### FanFiction.net Scraper

FF.net requires one HTTP request per chapter, unlike AO3's single-page approach:

```rust
pub struct FfNetScraper;

impl FfNetScraper {
    fn extract_story_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"/s/(\d+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
}

#[async_trait]
impl SiteScraper for FfNetScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiction.net") || url.contains("fictionpress.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let story_id = Self::extract_story_id(url)
            .ok_or_else(|| ScrapeError::ParseError(
                "could not extract story ID from FF.net URL".into()
            ))?;
        
        let fic_url = format!("https://www.fanfiction.net/s/{story_id}/1/");
        let response = client.get(&fic_url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // Title: #profile_top b.xcontrast_txt
        let title = document
            .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        // Author: #profile_top a.xcontrast_txt
        let author = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        
        // Author URL
        let author_url = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("https://www.fanfiction.net{h}"))
            .unwrap_or_default();
        
        // Word count: span[data-xutitle='word count']
        let words = document
            .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);
        
        // Chapter count: span[data-xutitle='chapters']
        let chapters = document
            .select(&Selector::parse("#profile_top span[data-xutitle='chapters']").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().trim().split('/').next()
                    .and_then(|s| s.trim().parse().ok())
            })
            .unwrap_or(1);
        
        let url_id = crate::scrape::generate_url_id(2, &story_id);
        let now = Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters,
            words,
            desc: description,
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: fic_url,
            source_id: 2,
            author_id: 0,
            author_url,
            author_local_id: story_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
        let mut chapters = Vec::new();
        let story_id = &meta.author_local_id;
        
        for i in 1..=meta.chapters {
            let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
            let response = client.get(&url)
                .header("User-Agent", "fichub.net/0.1.0")
                .send().await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            
            let html = response.text().await
                .map_err(|e| ScrapeError::Network(e.to_string()))?;
            let document = Html::parse_document(&html);
            
            // Content: div.storytext
            let content = document
                .select(&Selector::parse("div.storytext").unwrap())
                .next()
                .map(|el| el.inner_html())
                .unwrap_or_default();
            
            // Chapter title from select dropdown
            let title = document
                .select(&Selector::parse("select#chap_select option[selected]").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| format!("Chapter {i}"));
            
            chapters.push(Chapter {
                chapter_id: i,
                title,
                content,
            });
        }
        
        Ok(chapters)
    }
}
```

FF.net uses different CSS selectors than AO3:
- Title: `#profile_top b.xcontrast_txt`
- Author: `#profile_top a.xcontrast_txt`
- Word count: `#profile_top span[data-xutitle='word count']`
- Chapter count: `#profile_top span[data-xutitle='chapters']`
- Chapter content: `div.storytext`
- Chapter title: `select#chap_select option[selected]`

The `data-xutitle` attributes are custom data attributes used by FF.net's JavaScript. They're more stable than class names which might change with site redesigns.

### XenForo Scraper

XenForo forums (SpaceBattles, SV, QQ) have a different structure. Stories are posted as forum threads:

```rust
pub struct XenForoScraper;

const XENFORO_DOMAINS: &[&str] = &[
    "forums.spacebattles.com",
    "forums.sufficientvelocity.com",
    "forum.questionablequesting.com",
];

impl XenForoScraper {
    fn extract_thread_id(url: &str) -> Option<String> {
        let re = regex_lite::Regex::new(r"threads/.*\.(\d+)/?").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }
    
    fn is_xenforo_url(url: &str) -> bool {
        XENFORO_DOMAINS.iter().any(|d| url.contains(d))
    }
}

#[async_trait]
impl SiteScraper for XenForoScraper {
    fn can_handle(&self, url: &str) -> bool {
        Self::is_xenforo_url(url)
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let thread_id = Self::extract_thread_id(url)
            .ok_or_else(|| ScrapeError::ParseError("could not extract thread ID".into()))?;
        
        let response = client.get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        // Title: h1.p-title-value
        let title = document
            .select(&Selector::parse("h1.p-title-value").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        // Author: a.username
        let author_el = document
            .select(&Selector::parse("a.username").unwrap())
            .next();
        let author = author_el
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());
        
        let author_url = author_el
            .and_then(|el| el.value().attr("href"))
            .map(|h| {
                if h.starts_with('/') {
                    for domain in XENFORO_DOMAINS {
                        if url.contains(domain) {
                            return format!("https://{domain}{h}");
                        }
                    }
                }
                h.to_string()
            })
            .unwrap_or_default();
        
        let description = document
            .select(&Selector::parse("article.message-body").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();
        
        let url_id = crate::scrape::generate_url_id(3, &thread_id);
        let now = Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words: 0,
            desc: description,
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 3,
            author_id: 0,
            author_url,
            author_local_id: thread_id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
        let mut chapters = Vec::new();
        let url = &meta.source;
        
        let response = client.get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let article_sel = Selector::parse("article.message-body").unwrap();
        let mut chapter_idx = 0;
        
        for article in document.select(&article_sel) {
            chapter_idx += 1;
            let content = article.inner_html();
            let title = if chapter_idx == 1 {
                meta.title.clone()
            } else {
                format!("Chapter {chapter_idx}: Thread Page {chapter_idx}")
            };
            
            chapters.push(Chapter {
                chapter_id: chapter_idx,
                title,
                content,
            });
        }
        
        if chapters.is_empty() {
            return Err(ScrapeError::ParseError(
                "no content found in XenForo thread".into()
            ));
        }
        
        Ok(chapters)
    }
}
```

The XenForo scraper:
1. Extracts the thread ID from the URL using regex: `threads/.*.(\d+)/?`
2. Fetches the page
3. Extracts the title from `<h1 class="p-title-value">`
4. Finds the author from `<a class="username">`
5. Gets chapter content from `<article class="message-body">`

For multi-page threads, the scraper would need to handle pagination. Currently, FicHub fetches only the first page, treating each post as a chapter.

### HPFanFic Scraper

The HPFanFic scraper handles the Harry Potter fanfiction archive:

```rust
pub struct HpFanFicScraper;

#[async_trait]
impl SiteScraper for HpFanFicScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("hpfanfic.com")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let response = client.get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }
        
        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);
        
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok();
        let story_id = re
            .and_then(|r| r.captures(url))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        
        // Extract metadata using site-specific selectors
        let title = document
            .select(&Selector::parse("h2.story-title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        // ... extract more fields ...
        
        let url_id = crate::scrape::generate_url_id(6, &story_id);
        let now = Utc::now().timestamp_millis();
        
        Ok(FicMetadata {
            url_id,
            source_id: 6,
            // ...
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
        // Fetch and parse chapters
        let mut chapters = Vec::new();
        // ... implementation ...
        Ok(chapters)
    }
}
```

### Adult FanFiction Scraper

Similar to HPFanFic, this scraper handles the adult-fanfiction.org site:

```rust
pub struct AdultFanFictionScraper;

#[async_trait]
impl SiteScraper for AdultFanFictionScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("adult-fanfiction.org")
    }
    
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // Similar pattern to HPFanFic but with different selectors
        let re = regex_lite::Regex::new(r"/story/(\d+)").ok();
        let story_id = re
            .and_then(|r| r.captures(url))
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        
        let url_id = crate::scrape::generate_url_id(5, &story_id);
        // ... extract metadata ...
        
        Ok(FicMetadata {
            url_id,
            source_id: 5,
            // ...
        })
    }
    
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) 
        -> Result<Vec<Chapter>, ScrapeError> 
    {
        // Similar pattern
        Ok(chapters)
    }
}
```

### Common Patterns Across Scrapers

Despite different sites, all scrapers share these patterns:

1. **URL ID generation** — Each uses `generate_url_id(source_id, story_id)` with a unique source_id (1=AO3, 2=FF.net, 3=XenForo, 5=AdultFanFiction, 6=HPFanFic)
2. **User-Agent header** — All requests include `fichub.net/0.1.0`
3. **Error mapping** — All map reqwest errors to `ScrapeError::Network`
4. **Fallback defaults** — All have sensible defaults for missing data
5. **Timestamp generation** — All use `Utc::now().timestamp_millis()` for published/updated when the site doesn't provide dates
6. **Regex extraction** — All use regex_lite to extract IDs from URLs
7. **CSS selectors** — All use the scraper crate for HTML parsing

### Adding a New Scraper: Step by Step

Here's a complete checklist for adding support for a new fanfiction site:

1. **Create the scraper file** in `src/scrape/sites/your_site.rs`
2. **Define the struct** (usually a unit struct with no fields)
3. **Implement `SiteScraper`** with `can_handle`, `lookup`, and `fetch_chapters`
4. **Choose a source_id** (a unique integer not already used: 4 is available)
5. **Register in `mod.rs`** — add `pub mod your_site;`
6. **Add to registry** — add `scrapers.push(Box::new(sites::your_site::YourSiteScraper));`
7. **Add to CollectionWorker** — if you want recommendation data, add a `SiteFetcher` implementation
8. **Test with real URLs** — verify the scraper works end-to-end

### Scraper Testing Strategy

For each scraper, test:
- URL matching (`can_handle`)
- ID extraction (regex patterns)
- Metadata extraction (title, author, word count)
- Chapter fetching (content extraction)
- Error handling (not found, blocked, network errors)

You can test with captured HTML fixtures or by making real HTTP requests (with appropriate rate limiting).

⚠️ **Watch Out:** When testing scrapers, be respectful of the upstream sites. Don't run scraper tests in tight loops against production sites. Use captured HTML fixtures for unit tests. For integration tests, add delays between requests.


---

# Part 4: Export and Caching

---

## Chapter 16: Generating EPUB Files

EPUB is the standard format for e-books. It's supported by Kindle, Kobo, Apple Books, Google Play Books, and virtually every e-reader on the market. FicHub generates EPUB files from scraped fanfiction, complete with metadata, chapter navigation, and clean formatting. Let's walk through the entire EPUB generation pipeline.

### What Is EPUB?

An EPUB file is actually a ZIP archive containing a specific set of files:

```
story.epub (ZIP archive)
├── mimetype                    # Single line: "application/epub+zip"
├── META-INF/
│   └── container.xml           # Points to the content file
├── OEBPS/
│   ├── content.opf             # Book metadata and file manifest
│   ├── toc.ncx                 # Table of contents (for older readers)
│   ├── stylesheet.css          # Shared CSS for all chapters
│   ├── introduction.xhtml      # First page with metadata
│   ├── chapter_1.xhtml         # Chapter 1 content
│   ├── chapter_2.xhtml         # Chapter 2 content
│   └── ...                     # More chapters
```

The `epub-builder` crate handles all this complexity for us. We just provide metadata, chapters, and a stylesheet, and it produces a valid EPUB.

### The EPUB Builder Setup

Here's the complete EPUB generation function:

```rust
use std::fs;
use std::path::{Path, PathBuf};
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use md5::{Digest, Md5};
use uuid::Uuid;
use crate::export::ExportError;
use crate::scrape::{Chapter, FicMetadata};

pub async fn create_epub(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // 1. Create a unique work directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;
    
    // 2. Initialize the builder
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
    
    // 3. Set metadata
    builder.metadata("title", &meta.title)?;
    builder.metadata("author", &meta.author)?;
    builder.metadata("lang", "en")?;
    builder.metadata("description", &meta.desc)?;
    
    // 4. Add stylesheet
    let css = concat!(
        "body{font-family:serif;line-height:1.5;}",
        "h2{text-align:center;}",
        "p{margin:0.5em 0;}"
    );
    builder.stylesheet(css.as_bytes())?;
    
    // 5. Add introduction page
    // ... (see below)
    
    // 6. Add chapters
    // ... (see below)
    
    // 7. Write the EPUB file
    let epub_path = work_dir.join("output.epub");
    let file = fs::File::create(&epub_path)?;
    builder.generate(file)?;
    
    // 8. Compute MD5 hash
    let epub_data = fs::read(&epub_path)?;
    let md5_hex = Md5::digest(&epub_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    Ok((epub_path, md5_hex))
}
```

Each step is important:
1. We create a unique directory for this export using a UUID. This prevents conflicts when multiple exports happen concurrently.
2. The builder is initialized with `ZipLibrary` which handles ZIP creation.
3. Metadata goes into the OPF file (title, author, language, description).
4. The stylesheet is shared across all XHTML files.
5-6. We add the introduction and chapters.
7. `generate(file)` writes the complete EPUB to disk.
8. We compute the MD5 hash for cache storage.

### Adding the Introduction Page

Every FicHub EPUB starts with an introduction page containing the story's metadata:

```rust
let intro_html = format!(
    r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Introduction</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h1>{title}</h1>
    <h2>by {author}</h2>
    <table>
        <tr><td>Words:</td><td>{words}</td></tr>
        <tr><td>Chapters:</td><td>{chapters}</td></tr>
        <tr><td>Status:</td><td>{status}</td></tr>
        <tr><td>Published:</td><td>{published}</td></tr>
        <tr><td>Updated:</td><td>{updated}</td></tr>
    </table>
    <hr/>
    <p>{desc}</p>
</body>
</html>"#,
    title = escape_html(&meta.title),
    author = escape_html(&meta.author),
    words = meta.words,
    chapters = meta.chapters,
    status = escape_html(&meta.status),
    published = format_timestamp(meta.published),
    updated = format_timestamp(meta.updated),
    desc = escape_html(&meta.desc),
);

builder.add_content(
    EpubContent::new("introduction.xhtml", intro_html.as_bytes())
        .title("Introduction"),
)?;
```

The introduction gives readers context before they start reading. It includes all the key metadata: title, author, word count, chapter count, status, dates, and description.

### Adding Chapters

Each chapter gets its own XHTML file:

```rust
for chapter in chapters {
    let chapter_html = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
        title = escape_html(&chapter.title),
        content = chapter.content,
    );
    
    let filename = format!("chapter_{}.xhtml", chapter.chapter_id);
    builder.add_content(
        EpubContent::new(filename.as_str(), chapter_html.as_bytes())
            .title(&chapter.title),
    )?;
}
```

The `epub-builder` crate automatically:
- Generates the table of contents (NCX file) from the chapter titles
- Creates the content manifest (OPF file) listing all XHTML files
- Handles file numbering and ordering
- Packages everything into a valid EPUB ZIP
- Sets the correct MIME types

### HTML Escaping

User-provided content (titles, descriptions) must be escaped to prevent HTML injection and XML parsing errors:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace(''', "&#39;")
}
```

Without this, a title like `A <script>alert('XSS')</script> Story` would break the EPUB's XML structure. The `&` must be escaped first to avoid double-escaping.

### Timestamp Formatting

Unix milliseconds need to be formatted as human-readable dates:

```rust
fn format_timestamp(unix_millis: i64) -> String {
    use chrono::DateTime;
    let secs = unix_millis / 1000;
    let nsecs = ((unix_millis % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}
```

We convert milliseconds to seconds (for the DateTime API), compute nanoseconds for the sub-second part, and format as YYYY-MM-DD.

### MD5 Hash

After generating the EPUB, FicHub computes its MD5 hash:

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();
```

This hash is used for:
1. **Cache key** — The file is stored at `cache/epub/{url_id}/{hash}.epub`
2. **Integrity verification** — The download handler verifies the hash before serving
3. **Cache invalidation** — If the content changes, the hash changes, triggering regeneration
4. **URL generation** — The download URL includes the hash: `/cache/epub/abc123?h=md5hash`

### Export Error Types

```rust
#[derive(Debug)]
pub enum ExportError {
    IoError(String),
    TemplateError(String),
    EpubError(String),
    CalibreError(String),
    ZipError(String),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::IoError(e) => write!(f, "IO error: {}", e),
            ExportError::TemplateError(e) => write!(f, "template error: {}", e),
            ExportError::EpubError(e) => write!(f, "EPUB error: {}", e),
            ExportError::CalibreError(e) => write!(f, "Calibre error: {}", e),
            ExportError::ZipError(e) => write!(f, "zip error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

impl From<std::io::Error> for ExportError {
    fn from(e: std::io::Error) -> Self { ExportError::IoError(e.to_string()) }
}
impl From<epub_builder::Error> for ExportError {
    fn from(e: epub_builder::Error) -> Self { ExportError::EpubError(e.to_string()) }
}
impl From<zip::result::ZipError> for ExportError {
    fn from(e: zip::result::ZipError) -> Self { ExportError::ZipError(e.to_string()) }
}
```

Each error variant maps to a specific failure mode:
- `IoError` — File system operations (create dir, read/write files)
- `TemplateError` — Template rendering failures
- `EpubError` — epub-builder library errors (invalid metadata, build failures)
- `CalibreError` — Calibre conversion failures (MOBI/PDF)
- `ZipError` — ZIP archive creation failures (for HTML bundles)

### Version Tracking

The export module tracks versions for cache invalidation:

```rust
pub const ETYPE_EPUB: &str = "epub";
pub const ETYPE_HTML: &str = "html";
pub const ETYPE_MOBI: &str = "mobi";
pub const ETYPE_PDF: &str = "pdf";

pub fn etype_versions() -> HashMap<&'static str, i32> {
    let mut m = HashMap::new();
    m.insert(ETYPE_EPUB, 1);
    m.insert(ETYPE_HTML, 1);
    m.insert(ETYPE_MOBI, 0);
    m.insert(ETYPE_PDF, 0);
    m
}

pub fn compute_version(export_version: i32, etype_version: i32, fic_version_bump: i32) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

The version is the sum of:
- `export_version` — Global export module version (bumped on structural changes)
- `etype_version` — Format-specific version (EPUB v1, HTML v1, etc.)
- `fic_version_bump` — Per-fic version (incremented when content changes)

This ensures that any change to the export format or content triggers cache invalidation.

---

## Chapter 17: HTML Bundles

In addition to EPUB, FicHub generates HTML bundles — single-page HTML files wrapped in a ZIP archive. These are useful for reading in a browser without an e-reader.

### What's in an HTML Bundle?

An HTML bundle is a ZIP file containing a single `index.html` that includes:
- Story metadata (title, author, word count, status)
- A table of contents with chapter navigation links
- All chapter content in a single scrollable page
- Clean, readable CSS styling
- A footer with the source URL

### Building the HTML

The HTML generation builds one big string with embedded CSS:

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;
    
    // Build chapter navigation
    let mut chapters_nav = String::new();
    let mut chapters_content = String::new();
    
    for chapter in chapters {
        chapters_nav.push_str(&format!(
            r##"<li><a href="#ch{ch}">{title}</a></li>"##,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
        ));
        
        chapters_content.push_str(&format!(
            r#"<h2 id="ch{ch}">{title}</h2>
{content}"#,
            ch = chapter.chapter_id,
            title = escape_html(&chapter.title),
            content = chapter.content,
        ));
    }
    
    // Build the complete HTML document
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: Georgia, 'Times New Roman', serif;
            line-height: 1.7;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #fafafa;
        }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }}
        h2 {{
            text-align: center;
            margin: 1.5em 0 0.5em;
            font-size: 1.4em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }}
        .meta td {{ padding: 2px 8px; }}
        .desc {{
            margin: 1em 0;
            padding: 1em;
            background: #fff;
            border-radius: 4px;
            border: 1px solid #eee;
        }}
        .nav {{
            background: #fff;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1em;
            margin: 1.5em 0;
        }}
        .nav h3 {{ margin-bottom: 0.5em; }}
        .nav ul {{ list-style: none; columns: 2; }}
        .nav li {{ padding: 2px 0; }}
        .nav a {{ color: #1a5276; text-decoration: none; }}
        .nav a:hover {{ text-decoration: underline; }}
        .content p {{ margin: 0.5em 0; text-indent: 1.5em; }}
        .content p:first-of-type {{ text-indent: 0; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2em 0; }}
        .footer {{ text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }}
        a.back-to-top {{ display: block; text-align: right; font-size: 0.85em; color: #1a5276; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <div class="meta">
        <p>by <strong>{author}</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>{words}</td></tr>
            <tr><td>Chapters:</td><td>{chapters}</td></tr>
            <tr><td>Status:</td><td>{status}</td></tr>
            <tr><td>Published:</td><td>{published}</td></tr>
            <tr><td>Updated:</td><td>{updated}</td></tr>
        </table>
    </div>
    <div class="desc">
        {desc_escaped}
    </div>
    <hr/>
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>
            {nav}
        </ul>
    </div>
    <hr/>
    <div class="content">
        {content}
    </div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — {source}</p>
    </div>
</body>
</html>"#,
        title = escape_html(&meta.title),
        author = escape_html(&meta.author),
        words = meta.words,
        chapters = meta.chapters,
        status = escape_html(&meta.status),
        published = format_timestamp(meta.published),
        updated = format_timestamp(meta.updated),
        desc_escaped = escape_html(&meta.desc),
        nav = chapters_nav,
        content = chapters_content,
        source = escape_html(&meta.source),
    );
    
    // Write the HTML file
    let html_path = work_dir.join("index.html");
    fs::write(&html_path, &html)?;
    
    // Bundle into ZIP
    let zip_path = work_dir.join("bundle.zip");
    let zip_file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    
    zip.start_file("index.html", options)?;
    zip.write_all(html.as_bytes())?;
    zip.finish()?;
    
    // Compute MD5
    let zip_data = fs::read(&zip_path)?;
    let md5_hex = Md5::digest(&zip_data)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    Ok((zip_path, md5_hex))
}
```

### CSS Design

The CSS is carefully designed for comfortable reading:
- **Georgia font** — A serif font optimized for screen reading
- **1.7 line height** — Generous spacing for long-form text
- **800px max width** — Prevents lines from being too long (optimal readability is 60-80 characters per line)
- **Chapter navigation** — Two-column layout for quick jumping between chapters
- **Subtle borders** — Visual separation without harsh lines
- **Text indentation** — Paragraphs after the first are indented (traditional book style)

### ZIP Packaging

The HTML file is compressed and packaged into a ZIP:

```rust
let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

Deflated compression typically achieves 60-80% size reduction for HTML files. The ZIP file makes it easy to download and extract on any platform.

### When to Use HTML vs EPUB

HTML bundles are better when:
- The reader doesn't have an e-reader
- The story contains images or complex formatting
- The reader wants to print the story
- Quick preview without importing to an app

EPUB is better when:
- Reading on an e-ink device (Kindle, Kobo)
- The reader wants chapter navigation in their e-reader app
- The story is long (EPUB handles pagination better)
- The reader wants to adjust font size and reading settings

---

## Chapter 18: The Disk Cache

Caching is critical for performance. FicHub's disk cache stores generated files on the filesystem in a hash-based directory structure, preventing duplicate work and serving files instantly.

### Cache Directory Structure

```
cache/
├── epub/
│   ├── abc/
│   │   ├── def/
│   │   │   └── abcdef/
│   │   │       └── md5hash.epub
│   │   └── ghi/
│   │       └── ghijkl/
│   │           └── another.epub
│   └── ...
├── html/
│   └── ...
├── mobi/
│   └── ...
└── pdf/
    └── ...
```

The url_id is split into 3-character directory chunks (up to 3 levels deep). For a url_id like `abcdefgh1234`:
- Level 1: `abc/`
- Level 2: `def/`
- Level 3: `ghi/`
- Final: `abcdefgh1234/`

This prevents any single directory from having too many entries (which would slow down filesystem operations on some systems).

### Computing Cache Paths

```rust
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());
    
    // Split url_id into 3-char directory chunks
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }
    
    // Full url_id directory
    path = path.join(url_id);
    
    // File: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

For a url_id of `abcdef123456` with etype Epub and hash `md5hash`:
- Base: `/cache/epub/`
- Chunks: `abc/def/ghi/`
- Full ID: `abcdef123456/`
- File: `md5hash.epub`
- Full path: `/cache/epub/abc/def/ghi/abcdef123456/md5hash.epub`

### Cache Operations

**Ensure directory exists:**
```rust
pub fn ensure_cache_dir(path: &Path) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
```

**Check if file exists:**
```rust
pub fn cache_file_exists(path: &Path) -> bool {
    path.exists()
}
```

**Compute file hash:**
```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

**Move file to cache (atomic on same filesystem):**
```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

The `fs::rename` is atomic on the same filesystem — the file either moves completely or not at all. This prevents serving partially-written files.

**Clear stale cache files:**
```rust
pub fn clear_stale_cache(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    keep_hash: &str,
) -> AppResult<()> {
    let dir = cache_root.join(etype.as_str()).join(url_id);
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem != keep_hash {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
```

This removes old cache files when a story is updated. Only the current hash is kept.

### Cache Semaphores

When multiple requests come in for the same story simultaneously, FicHub uses semaphores to prevent duplicate exports:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;

pub async fn get_export_semaphore(
    semaphores: &CacheSemaphores,
    url_id: &str,
    etype: &EType,
) -> Arc<Semaphore> {
    let key = (url_id.to_string(), etype.clone());
    let mut map = semaphores.lock().await;
    map.entry(key)
        .or_insert_with(|| Arc::new(Semaphore::new(1)))
        .clone()
}
```

The semaphore allows only one concurrent export per (url_id, etype) pair. Other requests wait until the first export completes, then they find the cached result.

The `Arc<Mutex<HashMap<...>>>` pattern:
- `Arc` — Shared ownership across async tasks
- `Mutex` — Ensures only one task modifies the map at a time
- `HashMap<(String, EType), Arc<Semaphore>>` — Maps each story+format to a semaphore

### The Double-Check Pattern

FicHub uses a double-check pattern for cache lookups:

```rust
// 1. First check (without semaphore)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if cached.is_some() {
    return Ok(build_response(...)); // Cache hit!
}

// 2. Acquire semaphore
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// 3. Second check (another request might have finished while we waited)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if cached.is_some() {
    return Ok(build_response(...)); // Another request generated it!
}

// 4. Generate the file
let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await?;
```

This prevents thundering herd problems — 100 simultaneous requests for the same story will generate it once, not 100 times. The first request generates the file; the other 99 find it in the second check.

### EType Enum

The export type determines the file format and extension:

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

impl EType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EType::Epub => "epub",
            EType::Html => "html",
            EType::Mobi => "mobi",
            EType::Pdf => "pdf",
        }
    }
    
    pub fn suffix(&self) -> &'static str {
        match self {
            EType::Epub => ".epub",
            EType::Html => ".zip",
            EType::Mobi => ".mobi",
            EType::Pdf => ".pdf",
        }
    }
    
    pub fn version(&self) -> i32 {
        match self {
            EType::Epub => 1,
            EType::Html => 1,
            EType::Mobi => 0,
            EType::Pdf => 0,
        }
    }
}

impl std::str::FromStr for EType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "epub" => Ok(EType::Epub),
            "html" => Ok(EType::Html),
            "mobi" => Ok(EType::Mobi),
            "pdf" => Ok(EType::Pdf),
            _ => Err(()),
        }
    }
}
```

Note that HTML bundles have `.zip` suffix (not `.html`) because they're ZIP archives containing HTML.

---

## Chapter 19: Rate Limiting with Redis

Rate limiting protects both FicHub and the upstream fanfiction sites. FicHub implements a token bucket algorithm using Redis for distributed state.

### Token Bucket Algorithm

A token bucket works like this:
1. The bucket has a capacity (max tokens)
2. Tokens are added at a steady flow rate
3. Each request costs one token
4. If the bucket is empty, the request must wait
5. The bucket never exceeds its capacity

FicHub has two buckets:
- **Global** — Capacity: 150, Flow: 30 tokens/sec (total system throughput)
- **Per-IP** — Capacity: 30, Flow: 0.116 tokens/sec (~1 request per 8.6 seconds)

This means:
- The entire system can handle 30 requests per second
- Each individual IP can make about 7 requests per minute
- Burst requests (up to 30 at once) are allowed per IP
- Global capacity of 150 allows some burst before throttling

### The Lua Script

The token bucket logic runs atomically in Redis using a Lua script:

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
    return -1  -- allowed
else
    local wait = (requested - new_tokens) / flow
    return wait  -- seconds to wait
end
```

This script:
1. Gets the current token count and last drain time from Redis
2. Calculates how many tokens have been added since the last request (`elapsed * flow`)
3. Caps the tokens at capacity (can't overflow)
4. Checks if there are enough tokens for the request
5. Returns -1 if allowed, or the wait time in seconds if not
6. Updates the bucket state atomically

The script runs atomically in Redis — no race conditions possible, even with concurrent requests from multiple FicHub instances.

### The RedisBucketLimiter

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,        // SHA of loaded Lua script
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    global_capacity: f64,
    global_flow: f64,
    ip_capacity: f64,
    ip_flow: f64,
}
```

The `lua_sha` is the SHA hash of the loaded Lua script. Redis caches compiled scripts by SHA, so we only send the short SHA on each call instead of the full script text. This is more efficient.

### Loading the Lua Script

```rust
impl RedisBucketLimiter {
    pub async fn new(
        redis_conn: redis::aio::MultiplexedConnection,
        dynamic_rate_limit: bool,
    ) -> Result<Self, redis::RedisError> {
        let lua_script = r#"
            -- [full Lua script here]
        "#;
        
        let mut conn = redis_conn.clone();
        let lua_sha: String = redis::cmd("SCRIPT")
            .arg("LOAD")
            .arg(lua_script)
            .query_async(&mut conn)
            .await?;
        
        Ok(RedisBucketLimiter {
            redis: redis_conn,
            lua_sha,
            dynamic_rate_limit,
            static_delay_base: 0.1,
            datacenter_ips: Arc::new(RwLock::new(HashSet::new())),
            global_capacity: 150.0,
            global_flow: 30.0,
            ip_capacity: 30.0,
            ip_flow: 0.116,
        })
    }
}
```

### Checking Rate Limits

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        // Simple static delay
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }
    
    // Check datacenter IPs
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }
    
    // Check global bucket
    let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
        .await.unwrap_or(-1.0);
    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }
    
    // Check per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
        .await.unwrap_or(-1.0);
    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }
    
    RateLimitResult::Allowed
}

async fn check_bucket(&self, key: &str, capacity: f64, flow: f64) -> Result<f64, redis::RedisError> {
    let mut conn = self.redis.clone();
    let result: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)        // number of keys
        .arg(key)      // the key
        .arg(1.0)      // requested tokens
        .arg(capacity) // bucket capacity
        .arg(flow)     // token flow rate
        .query_async(&mut conn)
        .await?;
    Ok(result)
}
```

The check order is: datacenter block → global bucket → per-IP bucket. If any check fails, the request is rejected.

### Penalizing Failures

When a request fails (e.g., the upstream site returns an error), FicHub penalizes the rate limit by consuming extra tokens:

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}

async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)     // penalize with 1.5 tokens (50% more than normal)
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

The penalize function requests 1.5 tokens instead of 1, slowing down future requests from IPs that trigger errors.

### Datacenter IP Blocking

FicHub can block requests from known datacenter IP ranges:

```rust
pub async fn load_datacenter_ips(&self, sources: &[(String, String, String)]) {
    for (file_path, _type, _tag) in sources {
        if let Ok(content) = tokio::fs::read_to_string(file_path).await {
            let mut ips = self.datacenter_ips.write().await;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    if let Ok(ip) = line.parse::<IpAddr>() {
                        ips.insert(ip);
                    }
                }
            }
        }
    }
    tracing::info!("Loaded {} datacenter IPs", self.datacenter_ips.read().await.len());
}
```

Datacenter IPs are loaded from external tag source files and stored in a `RwLock<HashSet>` for thread-safe access. The `RwLock` allows multiple concurrent reads but exclusive writes.

### Rate Limit Result

```rust
pub enum RateLimitResult {
    Allowed,
    Wait(u64),  // seconds to wait
    Blocked,
}
```

The handler translates this into an HTTP response:
- `Allowed` → Process the request
- `Wait(n)` → Return 429 with `Retry-After: n` header
- `Blocked` → Return 403 Forbidden

### Testing Token Buckets

The tests use a pure-Rust implementation that mirrors the Lua script:

```rust
struct TokenBucket {
    value: f64,
    last_drain: f64,
    capacity: f64,
    flow: f64,
}

impl TokenBucket {
    fn new(capacity: f64, flow: f64, now: f64) -> Self {
        TokenBucket { value: capacity, last_drain: now, capacity, flow }
    }
    
    fn request(&mut self, requested: f64, now: f64) -> f64 {
        let elapsed = now - self.last_drain;
        let new_tokens = (self.value + elapsed * self.flow).min(self.capacity);
        let allowed = new_tokens - requested;
        if allowed >= 0.0 {
            self.value = allowed;
            self.last_drain = now;
            -1.0  // allowed
        } else {
            (requested - new_tokens) / self.flow  // wait time
        }
    }
    
    fn penalize(&mut self, now: f64) {
        self.request(1.5, now);
    }
}

#[test]
fn test_initial_bucket_fill() {
    let mut bucket = TokenBucket::new(100.0, 10.0, 0.0);
    let result = bucket.request(10.0, 0.0);
    assert!((result - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 90.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_refill_over_time() {
    let mut bucket = TokenBucket::new(50.0, 10.0, 0.0);
    bucket.request(50.0, 0.0); // Empty the bucket
    let r = bucket.request(20.0, 3.0); // After 3s: 30 tokens refilled
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_wait_calculation() {
    let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
    bucket.request(10.0, 0.0); // Empty
    let wait = bucket.request(5.0, 0.0); // Need 5, have 0, flow=2/sec
    assert!((wait - 2.5).abs() < f64::EPSILON); // Wait 2.5 seconds
}

#[test]
fn test_penalize_reduces_tokens() {
    let mut bucket = TokenBucket::new(10.0, 5.0, 0.0);
    bucket.request(8.0, 0.0);
    let before = bucket.value;
    bucket.penalize(0.0);
    assert!((bucket.value - (before - 1.5)).abs() < f64::EPSILON);
}
```

Testing the token bucket in pure Rust (without Redis) is much faster and catches logic errors before they reach production.

---

## Chapter 20: The Export Flow

Now let's trace the complete export flow — from the moment a request arrives to the moment the response is sent. This is where everything comes together.

### The 17-Step Export Flow

Here's every step the `epub_handler` performs:

**1. Validate Input**
```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}
if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
}
```

**2. Find the Right Scraper**
```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

**3. Lookup Metadata (HTTP Request to Upstream Site)**
```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
let info_request_ms = start.elapsed().as_millis() as i32;
```

**4. Upsert fic_info in Database**
```rust
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

**5. Auto-Populate Tags from Scraper**
```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(
            &state.db, &tag.name, tag.tag_type_id
        ).await {
            let _ = queries::upsert_fic_tag(&state.db, &meta.url_id, resolution.tag_id, &ip).await;
        }
    }
}
```

**6. Check Fic Blacklist**
```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if fic_blacklist.iter().any(|b| b.reason == 6) {
    // Greylist: show metadata but no download
    return Ok(build_metadata_response(&meta, &[], &state.config.export_version, None, true));
}
if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
    return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
}
```

**7. Check Author Blacklist**
```rust
let author_blacklist = queries::check_author_blacklist(&state.db, meta.source_id, meta.author_id).await?;
if !author_blacklist.is_empty() {
    return Ok(Json(json!({"err": -7, "msg": "author is blacklisted"})));
}
```

**8. Compute Cache Version**
```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
```

**9. Check Cache (First Pass)**
```rust
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_cached_response(...)); // Cache hit!
}
```

**10. Acquire Semaphore**
```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

**11. Double-Check Cache**
```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if cached.is_some() {
    return Ok(build_cached_response(...));
}
```

**12. Fetch Chapters**
```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

**13. Generate EPUB**
```rust
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

**14. Move EPUB to Cache**
```rust
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;
```

**15. Record Export Log**
```rust
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

**16. Generate HTML Bundle**
```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir).await?;
let html_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
let html_input_hash = format!("epub:{}", epub_hash);
queries::insert_export_log(&state.db, &meta.url_id, version, "html", &html_input_hash, &html_hash).await?;
```

**17. Build and Return Response**
```rust
Ok(Json(json!({
    "err": 0,
    "url_id": meta.url_id,
    "slug": generate_slug(&meta.title, &meta.url_id),
    "info": build_info_string(&meta),
    "meta": build_meta_json(&meta),
    "hashes": {"epub": epub_hash, "html": html_hash},
    "urls": {"epub": epub_url, "html": html_url},
    "epub_url": Some(epub_url),
    "html_url": Some(html_url),
})))
```

### Why This Flow Works

Each step is designed to be:
- **Idempotent** — Running it twice has the same effect as running it once
- **Resilient** — Failures at any step return clear error messages
- **Efficient** — Cache checks prevent unnecessary work
- **Safe** — Semaphores prevent duplicate exports

### Request Logging

After the export completes, FicHub logs the request for analytics:

```rust
let source_id = queries::insert_request_source(&state.db, false, "/api/v0/epub", "web request").await?;
queries::insert_request_log(
    &state.db, source_id, "epub", query, info_request_ms,
    Some(&meta.url_id), fic_json.as_deref(),
    Some(export_ms), Some(&format!("{}.epub", epub_hash)),
    Some(&epub_hash), Some(query),
).await?;
```

This logs:
- Which endpoint was called
- How long scraping took
- How long export took
- Which file was generated
- The input URL

This data is invaluable for monitoring, debugging, and usage analytics.

### The Slug Generator

FicHub generates URL-friendly slugs for story titles:

```rust
pub fn generate_slug(title: &str, url_id: &str) -> String {
    let sanitized: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let re = regex_lite::Regex::new(r"_+").unwrap();
    let slug = re.replace_all(&sanitized, "_").to_string();
    let slug = slug.trim_matches('_').to_string();
    format!("{}-{}", slug, url_id)
}
```

For example, "My Awesome Story!" with url_id "abc123" becomes "My_Awesome_Story-abc123".


---

# Part 5: Recommendation Engine

---

## Chapter 21: Collaborative Filtering

How do you recommend stories to someone? The most effective approach is collaborative filtering — finding readers with similar taste and suggesting what they liked. FicHub implements this using bookmark co-occurrence data.

### The Core Idea

If Reader A has bookmarked Story X and Story Y, and Reader B has bookmarked Story X, then Story Y is likely a good recommendation for Reader B. The more readers who share this pattern, the stronger the recommendation.

### Jaccard Coefficient

FicHub uses the Jaccard coefficient to measure similarity between stories:

```
Jaccard(A, B) = |A ∩ B| / |A ∪ B|
```

Where:
- `A ∩ B` = number of users who bookmarked both stories
- `A ∪ B` = total users who bookmarked either story

A Jaccard of 1.0 means identical reader sets. A Jaccard of 0.0 means no overlap.

In SQL, this is calculated as:

```sql
SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
    (c.cooccur_count::float
       / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
    ) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != $1
ORDER BY jaccard DESC
LIMIT $2
```

The denominator `s.favouriter_count + fw.favouriter_count - c.cooccur_count` is the union of the two reader sets (inclusion-exclusion principle).

### Co-occurrence Table

The `fic_bookmark_cooccur` table stores pairwise co-occurrence counts:

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL,
    work_b VARCHAR(128) NOT NULL,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

The `CHECK (work_a < work_b)` constraint ensures each pair is stored exactly once (in alphabetical order). This halves the storage and simplifies queries.

### Tag Fallback

When a story has few favouriters (less than `rec_min_favouriters_for_collab`, default 5), collaborative filtering isn't reliable. FicHub falls back to tag-based recommendations:

```rust
let min_collab = config.rec_min_favouriters_for_collab as i32;
let use_tag_fallback = favouriter_count < min_collab;

let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The tag fallback searches for:
1. Stories by the same author (strong signal, score 0.8)
2. Stories with matching title keywords (weaker signal, score 0.5)

### Blending Scores

The final score blends collaborative and tag-based scores:

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

for c in &mut candidates {
    let collab = c.score;
    let tag = c.tag_score;
    c.score = collab * weight + tag * (1.0 - weight);
}
```

- If a story has 5+ favouriters, `weight = 1.0` (pure collaborative filtering)
- If a story has 0 favouriters, `weight = 0.0` (pure tag-based)
- In between, it's a weighted blend

### Community Voting Boost

User votes on recommendations can boost scores:

```rust
let gamma = config.rec_voting_boost_gamma;  // default 0.2
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The `ln_1p` function ensures diminishing returns — the first few votes have a big impact, but subsequent votes add less. A story with 10 net upvotes gets a 48% boost; with 100 votes, a 96% boost.

### The Recommendation Query

The full SQL query combines co-occurrence data with seed metadata:

```sql
WITH seed AS (
    SELECT favouriter_count FROM fic_works WHERE url_id = $1
),
candidates AS (
    SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
           cooccur_count
    FROM fic_bookmark_cooccur
    WHERE work_a = $1 OR work_b = $1
)
SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
    (c.cooccur_count::float
       / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)
    ) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != $1
ORDER BY jaccard DESC
LIMIT $2
```

This CTE (Common Table Expression) approach is clean and efficient.

### Precomputed Cache

To avoid computing recommendations on every request, FicHub precomputes and caches them:

```rust
pub async fn compute_and_cache(&self, url_id: &str, config: &Config) -> Result<(), AppError> {
    let results = compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;
    
    // Clear old cache
    sqlx::query("DELETE FROM precomputed_recommendations WHERE url_id = $1")
        .bind(url_id)
        .execute(&self.db)
        .await?;
    
    // Insert new entries
    for (rank, result) in results.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO precomputed_recommendations
               (url_id, recommended_url_id, score, rank, computed_at)
               VALUES ($1, $2, $3, $4, NOW())"#
        )
        .bind(url_id)
        .bind(&result.url_id)
        .bind(result.score as f32)
        .bind(rank as i16)
        .execute(&self.db)
        .await?;
    }
    
    Ok(())
}
```

The cache has a configurable TTL (`rec_cache_ttl_hours`, default 12 hours).

---

## Chapter 22: The Collection Worker

The collection worker is a background task that scrapes user favourites from fanfiction sites and populates the recommendation engine tables.

### The SiteFetcher Trait

Each site implements the `SiteFetcher` trait:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    fn site_domain(&self) -> &str;
    
    async fn collect_favouriters(
        &self,
        client: &Client,
        work_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;
    
    async fn collect_user_favourites(
        &self,
        client: &Client,
        user_url: &str,
        max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;
    
    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

The `user_hash` method hashes user profile URLs to anonymize them while maintaining consistency.

### The Redis Queue

FicHub uses Redis lists as a work queue:

```rust
pub async fn enqueue(&self, url_id: &str, site_domain: &str, site_work_id: &str) 
    -> Result<(), redis::RedisError> 
{
    let item = QueueItem {
        url_id: url_id.to_string(),
        site_domain: site_domain.to_string(),
        site_work_id: site_work_id.to_string(),
    };
    let json = serde_json::to_string(&item)?;
    let key = format!("collection_queue:{}", site_domain);
    
    let mut conn = self.redis.lock().await;
    redis::cmd("LPUSH")
        .arg(&[key.as_str(), json.as_str()])
        .query_async(&mut *conn)
        .await
}
```

Items are pushed with `LPUSH` and popped with `LPOP`, creating a FIFO queue.

### The Worker Loop

```rust
pub async fn run(&self) {
    info!("Collection worker started");
    
    loop {
        for fetcher in &self.fetchers {
            let domain = fetcher.site_domain();
            let key = format!("collection_queue:{}", domain);
            
            let item_str: Option<String> = {
                let mut conn = self.redis.lock().await;
                redis::cmd("LPOP")
                    .arg(&key)
                    .query_async(&mut *conn)
                    .await
                    .unwrap_or(None)
            };
            
            if let Some(item_str) = item_str {
                // Apply per-site rate limit
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }
                
                match serde_json::from_str::<QueueItem>(&item_str) {
                    Ok(item) => {
                        if let Err(e) = self.process_work(item).await {
                            error!("Error processing work on {}: {}", domain, e);
                        }
                    }
                    Err(e) => warn!("Invalid queue item: {}", e),
                }
            }
        }
        
        // Brief sleep to avoid busy-looping
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

The worker polls all site queues in a round-robin fashion, with a 100ms sleep between iterations.

### Processing a Work

The core collection procedure:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    // (a) Fetch favouriters
    let favouriters = fetcher.collect_favouriters(&self.client, &work_url, max_pages).await?;
    
    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);
        
        // Skip already-recorded users
        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)"
        ).bind(&user_hash).bind(&item.url_id).fetch_one(&self.db).await.unwrap_or(false);
        
        if already_exists { continue; }
        
        // Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        ).bind(&user_hash).bind(&item.url_id).bind(&item.site_domain).execute(&self.db).await?;
        
        // (b) Fetch this user's favourite works
        let user_favourites = fetcher.collect_user_favourites(&self.client, user_url, max_pages).await?;
        
        // (c) Update co-occurrence
        if user_favourites.len() >= 2 {
            self.update_cooccurrence(&item.site_domain, &user_favourites).await?;
        }
    }
    
    Ok(())
}
```

### Updating Co-occurrence

```rust
async fn update_cooccurrence(&self, site_domain: &str, works: &[String]) -> Result<(), sqlx::Error> {
    for i in 0..works.len() {
        for j in (i + 1)..works.len() {
            let (work_a, work_b) = if works[i] < works[j] {
                (works[i].as_str(), works[j].as_str())
            } else {
                (works[j].as_str(), works[i].as_str())
            };
            
            sqlx::query(
                r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, site_domain, cooccur_count)
                   VALUES ($1, $2, $3, 1)
                   ON CONFLICT (work_a, work_b) DO UPDATE SET
                       cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                       last_updated = CURRENT_TIMESTAMP"#
            )
            .bind(work_a).bind(work_b).bind(site_domain)
            .execute(&self.db).await?;
        }
    }
    Ok(())
}
```

The `work_a < work_b` ordering is enforced before insertion, matching the CHECK constraint.

### Per-Site Rate Limiting

Each site has its own rate limiter using atomic CAS:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

impl PerSiteRateLimiter {
    pub async fn wait_if_needed(&self) {
        let delay_nanos = (self.delay_secs as u64) * 1_000_000_000;
        
        loop {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)
                .unwrap_or_default().as_nanos() as i64;
            let last = self.last_request.load(Ordering::Acquire);
            let elapsed = now.wrapping_sub(last);
            
            if elapsed >= delay_nanos as i64 {
                if self.last_request.compare_exchange(
                    last, now, Ordering::AcqRel, Ordering::Relaxed
                ).is_ok() {
                    return;
                }
            } else {
                let remaining = delay_nanos - elapsed as u64;
                tokio::time::sleep(Duration::from_nanos(remaining)).await;
            }
        }
    }
}
```

This uses compare-and-swap to atomically claim time slots, ensuring concurrent requests are properly serialized.

---

## Chapter 23: Community Suggestions

FicHub lets users submit and vote on recommendations. This chapter covers the suggestion and voting system.

### The Suggestion Table

```sql
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id),
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id),
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);
```

The unique constraint on `(url_id, suggested_url_id, submitted_by_ip)` means each user can only suggest each pair once (but they can update their comment).

### Submitting a Suggestion

```rust
pub async fn submit_suggestion(
    db: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    voter_ip: &str,
    comment: Option<&str>,
) -> Result<i64, AppError> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
               (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip)
           DO UPDATE SET comment = EXCLUDED.comment, created = CURRENT_TIMESTAMP
           RETURNING id"#
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(voter_ip)
    .bind(comment)
    .fetch_one(db)
    .await?;
    
    Ok(row.0)
}
```

The `ON CONFLICT ... DO UPDATE` means re-submitting updates the comment and timestamp instead of failing.

### Listing Suggestions

```rust
pub async fn get_community_suggestions(db: &PgPool, url_id: &str) -> Result<Vec<Suggestion>, AppError> {
    let rows: Vec<SuggestionRow> = sqlx::query_as(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(v.net, 0) AS net_votes, s.created
           FROM recommendation_suggestions s
           LEFT JOIN (
               SELECT suggestion_id, SUM(vote)::INT AS net
               FROM recommendation_votes
               GROUP BY suggestion_id
           ) v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           ORDER BY net_votes DESC, s.created DESC"#
    )
    .bind(url_id)
    .fetch_all(db)
    .await?;
    
    Ok(rows.into_iter().map(|r| Suggestion {
        id: r.id,
        suggested_url_id: r.suggested_url_id,
        comment: r.comment,
        net_votes: r.net_votes,
        created: r.created,
    }).collect())
}
```

Suggestions are sorted by net votes (most popular first), then by creation date.

### The Suggest API Endpoint

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    // Resolve the suggested URL to a url_id
    let scraper = state.scraper_registry.find_scraper(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", body.suggested_url)))?;
    let meta = scraper.lookup(&state.http_client, &body.suggested_url).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let suggested_url_id = meta.url_id;
    
    // Validate both fics exist
    let seed_exists = queries::get_fic_info(&state.db, &body.url_id).await?.is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id).await?.is_some();
    
    if !seed_exists {
        state.collection_worker.enqueue(&body.url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "seed fic not found"})));
    }
    
    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "suggested fic not yet collected"})));
    }
    
    let suggestion_id = submit_suggestion(&state.db, &body.url_id, &suggested_url_id, "0.0.0.0", body.comment.as_deref()).await?;
    
    Ok(Json(json!({"err": 0, "suggestion_id": suggestion_id})))
}
```

If either fic doesn't exist in the database, it's enqueued for background collection.

---

## Chapter 24: Voting and Scoring

Voting lets users upvote or downvote recommendations, surfacing the best suggestions.

### The Vote Table

```sql
CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

Each user (identified by IP) can cast one vote per suggestion. Changing a vote updates the existing row.

### Casting a Vote

```rust
pub async fn cast_vote(
    db: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    vote_value: i16,
) -> Result<i32, AppError> {
    let vote = vote_value.clamp(-1, 1);
    
    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip)
           DO UPDATE SET vote = EXCLUDED.vote, created = CURRENT_TIMESTAMP"#
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(vote)
    .execute(db)
    .await?;
    
    // Return the new net vote count
    let row: (Option<i32>,) = sqlx::query_as(
        "SELECT SUM(vote)::INT FROM recommendation_votes WHERE suggestion_id = $1"
    )
    .bind(suggestion_id)
    .fetch_one(db)
    .await?;
    
    Ok(row.0.unwrap_or(0))
}
```

The `clamp(-1, 1)` ensures only valid vote values are accepted.

### Optimistic Voting

The vote handler returns the new score immediately:

```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(json!({"err": -1, "msg": "vote must be 1 or -1"})));
    }
    
    let new_score = cast_vote(&state.db, body.suggestion_id, "0.0.0.0", body.vote as i16).await?;
    
    Ok(Json(json!({"err": 0, "new_score": new_score})))
}
```

The client can update the UI immediately with the new score, without waiting for a refresh.

### Community Score in Recommendations

When computing recommendations, FicHub aggregates community votes:

```rust
async fn get_community_votes(db: &PgPool, candidates: &[CandidateScore]) 
    -> Result<HashMap<String, i32>, AppError> 
{
    let url_ids: Vec<String> = candidates.iter().map(|c| c.url_id.clone()).collect();
    
    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.suggested_url_id = ANY($1)
           GROUP BY s.suggested_url_id"#
    )
    .bind(&url_ids)
    .fetch_all(db)
    .await?;
    
    Ok(rows.into_iter()
        .map(|(url_id, net)| (url_id, net.unwrap_or(0)))
        .collect())
}
```

The voting boost formula uses logarithmic scaling:

```rust
let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
c.score *= boost;
```

This ensures:
- Positive votes boost the score
- Negative votes don't penalize (they just don't boost)
- Diminishing returns prevent vote manipulation
- The gamma parameter controls boost strength

### Rate Limiting Votes

Votes are rate-limited per IP using Redis:

```rust
async fn check_tag_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    action: &str,
    ip: std::net::IpAddr,
    limit: u32,
) -> AppResult<()> {
    let key = format!("ratelimit:tag:{}:{}", action, ip);
    
    let count: Option<u32> = redis::cmd("GET")
        .arg(&key)
        .query_async(redis)
        .await
        .unwrap_or(None);
    
    if let Some(c) = count {
        if c >= limit {
            let ttl: u64 = redis::cmd("TTL")
                .arg(&key)
                .query_async(redis)
                .await
                .unwrap_or(3600);
            return Err(AppError::RateLimited(ttl));
        }
    }
    
    let new_count: u32 = redis::cmd("INCR")
        .arg(&key)
        .query_async(redis)
        .await?;
    
    if new_count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(3600i64)
            .query_async(redis)
            .await?;
    }
    
    Ok(())
}
```

The default limits are:
- Tag submissions: 10 per hour
- Tag votes: 20 per hour
- Recommendation suggestions: 5 per hour
- Recommendation votes: 10 per hour


---

# Part 6: Server and Deployment

---

## Chapter 25: The Full Axum Router

We've seen individual routes in earlier chapters. Now let's look at the complete router — every endpoint, its purpose, and how they all connect.

### Route Map

FicHub exposes these endpoint groups:

**Core API (v0):**
- `GET /api/` — API documentation
- `GET /api/v0/epub?q=<url>` — Export story (main endpoint)
- `GET /api/v0/meta?q=<url>` — Get metadata only
- `GET /api/v0/remote` — Client IP info

**Cache Downloads:**
- `GET /cache/{type}/{url_id}/{fname}?h=<hash>` — Download with hash validation
- `GET /cache/{type}/{url_id}?h=<hash>` — Download or trigger export

**Recommendations:**
- `GET /api/v0/recommendations?q=<url>` — Get recommendations
- `POST /api/v0/recommendations/suggest` — Submit a suggestion
- `POST /api/v0/recommendations/vote` — Vote on a suggestion
- `GET /api/v0/recommendations/votes?url_id=<id>` — List suggestions/votes

**Tags (v3):**
- `POST /api/v0/tags/submit` — Submit a tag
- `POST /api/v0/tags/vote` — Vote on a tag
- `POST /api/v0/tags/flag` — Flag a tag for review
- `GET /api/v0/tags?url_id=<id>` — Get tags for a fic

**Curator Tools:**
- `POST /api/v0/curator/alias` — Create tag alias
- `POST /api/v0/curator/merge` — Merge two tags
- `DELETE /api/v0/curator/tags/{id}` — Delete a tag
- `GET /api/v0/curator/flags` — List flagged tags
- `POST /api/v0/curator/flags/{id}/resolve` — Resolve a flag

**Search:**
- `GET /api/v0/search?q=<query>&include_tags=...` — Full-text search

**OPDS Catalog:**
- `GET /opds` — Root catalog
- `GET /opds/new` — Recent fics
- `GET /opds/popular` — Popular fics
- `GET /opds/tags` — Tag types
- `GET /opds/tags/{type_id}` — Tags of a type
- `GET /opds/tags/{type_id}/{tag_name}` — Fics by tag
- `GET /opds/authors` — Author list
- `GET /opds/recommendations/popular` — Popular recommendations
- `GET /opds/recommendations` — Fic recommendations
- `GET /opds/search?q=<query>` — Search feed
- `GET /opds/shelves` — Shelf list
- `GET /opds/shelf/{shelf_id}` — Shelf contents

**Legacy Redirects:**
- `GET /legacy/epub_export` → `/`
- `GET /fic/{url_id}` → `/`
- `GET /changes` → `/`
- `GET /popular/` → `/`

**Static Files:**
- `/*` — Frontend SPA (fallback)

### AppState Construction

Every handler needs access to the shared state. Here's how it's assembled:

```rust
let state = Arc::new(AppState {
    config: config.clone(),
    db: db_pool,
    redis: redis_conn,
    http_client,
    scraper_registry,
    cache_semaphores: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
    rate_limiter: Box::new(rate_limiter),
    recommender_engine,
    collection_worker,
});
```

Each field serves a specific purpose:
- `config` — All application settings
- `db` — PostgreSQL connection pool
- `redis` — Redis connection for rate limiting and queues
- `http_client` — reqwest client for scraping
- `scraper_registry` — Maps URLs to site scrapers
- `cache_semaphores` — Prevents duplicate concurrent exports
- `rate_limiter` — Token bucket rate limiter
- `recommender_engine` — Computes recommendations
- `collection_worker` — Background favourite collection

### Middleware Stack

```rust
.layer(TraceLayer::new_for_http())  // Request/response logging
.layer(CorsLayer::permissive())      // CORS headers
```

The order matters — `TraceLayer` wraps `CorsLayer`, which wraps the handlers.

### The Server Binding

```rust
let addr = format!("0.0.0.0:{}", config.app_port);
let listener = tokio::net::TcpListener::bind(&addr).await
    .expect("Failed to bind to address");

axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
).await.expect("Server error");
```

The `into_make_service_with_connect_info` enables the `ConnectInfo<SocketAddr>` extractor, which gives handlers access to the client's IP address.

---

## Chapter 26: Serving Static Files

FicHub serves the SvelteKit frontend as static files, with SPA routing support.

### The Fallback Service

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This is a three-layer fallback:
1. **ServeDir** — Serves files from the frontend build directory
2. **append_index_html** — Serves `index.html` for directory requests
3. **fallback** — Serves `index.html` for any unmatched path (SPA routing)

### SPA Routing

Single-page applications handle routing on the client side. When a user navigates to `/search?q=hello`, the browser requests that URL from the server. Without SPA support, the server would return 404.

The fallback ensures that any URL not matching a static file returns `index.html`, which loads the SPA JavaScript that handles the routing.

### Cache Headers

For production, you'd add cache headers to static assets:

```rust
use tower_http::set_header::SetResponseHeaderLayer;

let cache_headers = SetResponseHeaderLayer::overriding(
    header::CACHE_CONTROL,
    HeaderValue::from_static("public, max-age=31536000"),
);

Router::new()
    .nest_service("/assets", get(assets_handler).layer(cache_headers))
```

This tells browsers to cache static assets (JS, CSS, images) for one year.

### File Serving Performance

`ServeDir` uses `tokio::fs` for non-blocking file I/O. This means serving static files doesn't block the async runtime — other requests continue to be processed while files are being read from disk.

---

## Chapter 27: Docker and Docker Compose

FicHub deploys via Docker with a multi-stage Dockerfile and a complete docker-compose stack.

### The Dockerfile

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y     pkg-config     libssl-dev     libpq-dev     && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y     ca-certificates     libpq-dev     && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fichub /app/fichub
COPY --from=builder /app/migrations /app/migrations
COPY .env.example /app/.env

EXPOSE 3000

CMD ["/app/fichub"]
```

The multi-stage build:
1. **Builder stage** — Installs Rust, compiles the binary (large image, ~1GB)
2. **Runtime stage** — Only copies the binary and runtime dependencies (small image, ~100MB)

### Docker Compose

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: fichub
      POSTGRES_PASSWORD: fichub
      POSTGRES_USER: fichub
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  calibre:
    build:
      context: .
      dockerfile: docker/calibre.Dockerfile
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp

  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      CALIBRE_CONTAINER: calibre
      PORT: 3000
      FRONTEND_DIR: /app/frontend
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

volumes:
  pgdata:
  cache:
  tmp:
```

### Service Dependencies

The `depends_on` with `condition: service_healthy` ensures:
1. PostgreSQL is fully initialized before FicHub starts
2. Redis is ready before FicHub starts
3. Health checks verify each service is actually working

### Calibre Integration

FicHub optionally uses Calibre's `ebook-convert` for MOBI and PDF generation:

```rust
pub async fn convert_epub(
    epub_path: &Path,
    output_format: &str,
    calibre_container: &str,
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    if calibre_container.is_empty() {
        // Run directly on host
        run_direct(epub_path, &output_path, timeout_dur).await
    } else {
        // Run inside Docker container
        run_docker(calibre_container, epub_path, &output_path, timeout_dur).await
    }
}
```

The Docker compose stack includes a Calibre container, and FicHub runs `docker exec calibre ebook-convert ...` to convert files.

### Building and Running

```bash
# Build everything
docker compose build

# Start the stack
docker compose up -d

# Check logs
docker compose logs -f app

# Run migrations
docker compose exec app /app/fichub  # Migrations run automatically on startup

# Stop
docker compose down
```

---

## Chapter 28: Cross-Compilation and Deploy

For production, FicHub targets ARM64 (for Orange Pi or Raspberry Pi) and deploys via systemd.

### Cross-Compilation

To build for ARM64 on an x86_64 machine:

```bash
# Install the ARM64 target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compilation tools (Arch/Manjaro)
sudo pacman -S aarch64-linux-gnu-gcc

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

The binary is at `target/aarch64-unknown-linux-gnu/release/fichub`.

### Deploying to the Server

```bash
# Copy the binary
rsync -avz target/aarch64-unknown-linux-gnu/release/fichub user@server:/opt/fichub/

# Copy migrations
rsync -avz migrations/ user@server:/opt/fichub/migrations/

# Copy configuration
scp .env user@server:/opt/fichub/.env
```

### Systemd Service

```ini
[Unit]
Description=FicHub Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/opt/fichub
ExecStart=/opt/fichub/fichub
Restart=always
RestartSec=5
Environment=RUST_LOG=info,fichub=debug

[Install]
WantedBy=multi-user.target
```

```bash
# Install and enable
sudo cp fichub.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now fichub
```

### Nginx Reverse Proxy

For production with TLS:

```nginx
server {
    listen 443 ssl http2;
    server_name fichub.example.com;
    
    ssl_certificate /etc/letsencrypt/live/fichub.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.example.com/privkey.pem;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Cache static assets
    location /assets/ {
        proxy_pass http://127.0.0.1:3000;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}
```

### Health Checks

FicHub doesn't have a dedicated health endpoint, but you can use the API docs endpoint:

```bash
curl -f http://localhost:3000/api/ || exit 1
```

For systemd, add a watchdog:

```ini
WatchdogSec=30
```

### Monitoring

FicHub uses `tracing` for structured logging and `axum-prometheus` for metrics:

```bash
# View logs
journalctl -u fichub -f

# Prometheus metrics (if enabled)
curl http://localhost:3000/metrics
```


---
