# Part 1: Welcome to Rust Backend Development

---

# Chapter 1: What We're Building

Welcome to the most exciting project you'll undertake this year. By the end of this book, you'll have built a complete, production-ready backend server in Rust that can scrape fanfiction from multiple websites, convert stories into beautiful EPUB files and HTML bundles, serve them through a blazing-fast web API, recommend similar stories to readers, and do all of this while running safely behind rate limits and a rock-solid caching layer. This isn't a toy project — it's a real, deployable application that handles real traffic and real data.

## What Is FicHub?

FicHub is a fanfiction download service. Think of it as a personal library assistant for the fanfiction world. You give it a URL to a story on Archive of Our Own, FanFiction.net, or several other sites, and it fetches the story, packages it up as an EPUB file (or HTML bundle, MOBI, or PDF), and hands it back to you for reading on your e-reader, tablet, or phone.

**Real-world analogy:** Imagine you have a friend who is incredibly fast at copying books by hand. You give them a URL, they run to the library, photocopy the entire book, bind it beautifully, and hand it back to you — all in under a minute. FicHub is that friend, except it's a computer program, it never gets tired, and it can do this for thousands of people simultaneously.

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

### Why Fanfiction?

You might wonder why we're building a fanfiction download server. The answer is that fanfiction sites present a unique set of technical challenges that make for an excellent learning project:

1. **Real-world web scraping** — Fanfiction sites have complex HTML structures, anti-scraping measures, and varying page layouts across chapters. Learning to scrape them teaches you more about HTML parsing than any textbook.

2. **Multi-format export** — Stories need to be converted into EPUB, HTML, MOBI, and PDF formats. Each format has its own quirks and requirements.

3. **High-concurrency demands** — Popular stories get requested hundreds of times per day. Building a system that handles this efficiently teaches you about caching, rate limiting, and async programming.

4. **Community features** — Tags, recommendations, and voting systems teach you about database design, user interactions, and collaborative filtering algorithms.

5. **Real users** — Unlike a todo app or a blog, FicHub has real users who depend on it working correctly. This adds pressure to write robust, well-tested code.

## Why Rust?

You might be wondering why we're building this in Rust instead of Python, Node.js, or Go. The answer comes down to three things: performance, safety, and the joy of building something that just works.

### Performance

FicHub needs to scrape web pages, parse HTML, generate EPUB files, and serve HTTP requests — all potentially at the same time for different users. Rust's async runtime (Tokio) lets us handle thousands of concurrent connections without breaking a sweat. The zero-cost abstractions mean we get high-level ergonomics without paying a runtime tax.

When benchmarked against Node.js for HTTP throughput, Rust with Axum consistently handles 2-3x more requests per second with lower latency. When compared to Python, the difference is even more dramatic — Rust is typically 10-50x faster for CPU-bound operations like HTML parsing and EPUB generation.

**Real-world analogy:** Think of Python as a reliable family sedan — it gets you where you need to go, it's comfortable, and everyone knows how to drive one. Rust is a Formula 1 car — it's built for speed, every component is precision-engineered, and when you need to go fast, nothing else comes close.

Here's a concrete example. When FicHub generates an EPUB file, it needs to:
1. Parse the HTML content of each chapter
2. Convert it to EPUB-compatible XHTML
3. Build a table of contents
4. Add metadata (title, author, description)
5. Compress everything into a ZIP archive
6. Compute an MD5 hash for caching

In Rust, this entire pipeline completes in under 50 milliseconds for a typical story. In Python, the same operation would take 500-1000 milliseconds. For a single request, that difference might not matter. But when you're serving hundreds of concurrent requests, those milliseconds add up fast.

### Safety

When you're building a web server that handles user input, database connections, and file system operations, bugs can be catastrophic. Rust's ownership system catches entire categories of bugs at compile time — no null pointer dereferences, no data races, no use-after-free. If the code compiles, it's already free of a huge class of bugs.

For a web server that processes untrusted input from multiple fanfiction sites and serves it back to users, this safety guarantee is invaluable. Consider what could go wrong in a less safe language:

- **Buffer overflow** — An attacker sends a crafted URL that causes a buffer overflow, potentially allowing arbitrary code execution. In Rust, this is impossible because the compiler enforces bounds checking.

- **Use-after-free** — A database connection is closed while a query is still in progress. In Rust, the ownership system prevents you from using a closed connection.

- **Data race** — Two threads try to update the same cache entry simultaneously, causing corruption. Rust's type system prevents data races at compile time.

- **Null pointer dereference** — A scraper returns `None` for a field, and the code tries to use it as if it were `Some`. In Rust, you must explicitly handle the `None` case.

**Real-world analogy:** Rust's safety guarantees are like having a spell-checker, grammar-checker, and fact-checker built into your word processor. You can still write bad prose, but you can't accidentally submit a document with obvious errors.

### Ecosystem

The Rust web ecosystem has matured beautifully. Axum gives us a web framework that's fast, type-safe, and a pleasure to use. SQLx provides compile-time checked SQL queries against a real PostgreSQL database. The `scraper` crate handles HTML parsing. `epub-builder` generates EPUB files. Every piece of the puzzle has a high-quality, well-maintained crate.

The Rust package registry (crates.io) has over 150,000 crates, and the quality bar is remarkably high. Unlike npm or PyPI, where package quality varies wildly, Rust crates tend to be well-documented, well-tested, and actively maintained. The strict type system means that most bugs are caught at compile time, so you spend less time debugging and more time building.

### Joy

This might sound surprising, but Rust is genuinely fun to write. The compiler is strict, but it's also incredibly helpful. Error messages explain what went wrong and suggest fixes. The `cargo` build system handles dependencies, testing, and formatting. And there's a deep satisfaction in knowing that when your code works, it really works.

The Rust community is also exceptionally welcoming. The Rust book, the Rust by Example site, and the Rust Reference are some of the best programming language documentation ever written. The community forums, Discord servers, and conferences are full of helpful, knowledgeable people.

## The Architecture at a Glance

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

This organization makes the codebase easy to understand and modify. Want to add a new fanfiction site? Add a new scraper in `scrape/sites/`. Want to change how EPUBs are generated? Modify `export/epub.rs`. Want to tune the recommendation algorithm? Edit `recommender/engine.rs`.

### Module Dependency Graph

Here's how the modules depend on each other:

```
main.rs → server.rs → routes/* → scrape/*, export/*, cache/*, tags/*, search/*
                                     ↓
                                   db/* (models, queries)
                                     ↓
                               PostgreSQL, Redis
```

The dependency flow is clean: `main.rs` starts the server, the server creates the router, the router dispatches to handlers, and the handlers use the domain modules (scrape, export, cache, tags, search) which in turn use the database layer.

This is a "star" architecture — all domain modules are independent of each other and only depend on the database layer. The `routes` layer depends on everything because it's the integration point. This means you can test any domain module in isolation, which is crucial for maintainability.

### The AppState Struct

The central nervous system of FicHub is the `AppState` struct. This is a shared state object that's passed to every request handler. It contains everything the handlers need to do their work:

```rust
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

Let's break down each field:

- **`config`** — Application configuration loaded from environment variables. Contains database URLs, Redis URLs, port numbers, rate limit settings, and more.

- **`db`** — A PostgreSQL connection pool. SQLx manages a pool of connections that are shared across all requests. When a handler needs to query the database, it borrows a connection from the pool and returns it when done.

- **`redis`** — A multiplexed Redis connection. Multiplexing means multiple requests can share the same TCP connection, which is more efficient than having one connection per request.

- **`http_client`** — A shared HTTP client for making outgoing requests to fanfiction sites. The client is shared because it maintains a connection pool internally.

- **`scraper_registry`** — The registry of all available scrapers. When a request comes in, the registry finds the right scraper for the URL.

- **`cache_semaphores`** — A map of semaphores that prevent duplicate concurrent exports. If two users request the same story simultaneously, only one export runs — the other waits.

- **`rate_limiter`** — The rate limiter that prevents too many requests to upstream sites. Uses a token bucket algorithm backed by Redis.

- **`recommender_engine`** — The recommendation engine that computes similar stories based on collaborative filtering.

- **`collection_worker`** — A background worker that scrapes user favourite lists to build the data for the recommendation engine.

The `AppState` is wrapped in an `Arc` (atomic reference counter) so it can be shared across async tasks. Each handler receives a reference to the state via Axum's `State` extractor.

## What You'll Learn

This book is structured as a journey. We'll start with the basics and build up layer by layer:

**Part 1 (Chapters 1–4):** We set up our development environment and learn the Rust fundamentals we need. You'll install Rust, PostgreSQL, and Redis, then write your first Rust programs.

**Part 2 (Chapters 5–10):** We build the web server layer with Axum, connect to PostgreSQL, and handle configuration and errors. By the end, you'll have a working API server.

**Part 3 (Chapters 11–15):** We dive into web scraping — understanding fanfiction site structures, building scrapers, and implementing the registry pattern. You'll scrape AO3, FF.net, and XenForo forums.

**Part 4 (Chapters 16–20):** We tackle EPUB generation, HTML bundles, disk caching, rate limiting, and the complete export flow. We trace every step from request to response.

**Part 5 (Chapters 21–24):** We build the recommendation engine with collaborative filtering, community suggestions, and voting. You'll understand Jaccard coefficients and co-occurrence tables.

**Part 6 (Chapters 25–28):** We wire up the complete router, serve static files, Docker-ize everything, and deploy to a real server.

**Part 7 (Chapters 29–33):** We implement advanced features: search, tags, OPDS, API docs, and performance tuning. These make the difference between a prototype and a product.

**Part 8 (Chapters 34–36):** We test everything, harden for production, and look at what's next. You'll learn testing patterns, security best practices, and deployment strategies.

**Part 9 (Chapters 37–43):** Advanced Rust patterns — the builder pattern, traits, smart pointers, newtype, error handling, serde, and property-based testing.

**Part 10 (Chapters 44–50):** Async deep dive — understanding futures, executors, Tokio internals, async I/O patterns, structured concurrency, backpressure, and async cleanup.

**Part 11 (Chapters 51–57):** Database performance — PostgreSQL internals, connection pool tuning, query optimization, indexing strategies, batch operations, and migrations at scale.

**Part 12 (Chapters 58–64):** Security hardening — input validation, SQL injection prevention, rate limiting security, CORS, secrets management, dependency auditing, and container security.

**Part 13 (Chapters 65–69):** Monitoring and observability — structured logging, metrics, distributed tracing, health checks, and alerting.

**Part 14 (Chapters 70–75):** Troubleshooting guide — build errors, runtime debugging, memory leaks, performance profiling, database troubleshooting, and network issues.

**Part 15:** Glossary of all technical terms used in the book.

## Who Is This Book For?

This book is for developers who want to build real backend systems in Rust. You don't need to be a Rust expert — we'll cover the language features as we encounter them. But you should be comfortable with:

- Basic programming concepts (variables, functions, loops, conditionals)
- Command-line tools (we'll use `cargo`, `psql`, `redis-cli`)
- Web concepts (HTTP requests, JSON, databases, URLs)

If you've written code in any language before, you'll be fine. The concepts are universal; only the syntax is new.

### If You're Coming from Python

Python developers will find Rust's type system refreshing but challenging. Where Python gives you flexibility (duck typing, dynamic attributes), Rust gives you safety (compile-time type checking, ownership). The trade-off is that you write more code upfront, but you spend less time debugging at 3 AM.

The biggest adjustment will be Rust's ownership model. In Python, you can pass an object around freely and the garbage collector handles cleanup. In Rust, you need to think about who owns each piece of data and who borrows it. This takes practice, but once it clicks, you'll wonder how you ever lived without it.

### If You're Coming from Node.js/TypeScript

Node.js developers will find Rust's async model familiar but different. Both use an event loop and non-blocking I/O, but Rust's async is zero-cost — there's no runtime overhead for the async machinery. You'll also appreciate that Rust's type system is much stronger than TypeScript's.

The biggest adjustment will be the lack of `null` and `undefined`. In Rust, everything is either `Some(value)` or `None`, and you must explicitly handle both cases. This eliminates the "cannot read property of undefined" errors that plague JavaScript code.

### If You're Coming from Go

Go developers will find Rust's concurrency model different. Go uses goroutines and channels, while Rust uses async/await and Tokio. Both are efficient, but Rust's approach is more explicit and gives you more control over what's happening under the hood.

The biggest adjustment will be the ownership system. Go has a garbage collector, so you don't need to think about memory management. In Rust, you need to think about ownership and borrowing, which takes practice but gives you more predictable performance.

## A Note on the Code

Every code example in this book comes from the actual FicHub source code. We'll walk through real functions, real structs, and real SQL queries. When you see code, it's not a simplified toy — it's the actual implementation, explained line by line.

Some things to keep in mind:
- The code uses **Rust 2024 edition** features
- Dependencies are pinned to specific versions in `Cargo.toml`
- SQL queries use `sqlx`'s compile-time checking
- The async runtime is **Tokio**
- We use `tracing` for structured logging
- Error handling uses a custom `AppError` enum, not panics or unwrap

Let's get started.

## 📝 Practice Exercises

1. **Explore the Architecture:** Download the FicHub source code from GitHub and spend 30 minutes exploring the directory structure. Identify each module and try to guess what it does based on the file names.

2. **Draw the Dependency Graph:** On a piece of paper, draw the module dependency graph. Start with `main.rs` and trace which modules it imports. Then trace the dependencies of each of those modules.

3. **Find the Entry Points:** Search the codebase for all `pub async fn` functions. These are the public API entry points. How many are there? What do they do?

4. **Compare to Your Favorite Backend:** Think of a backend project you've worked on before (in any language). How does its architecture compare to FicHub's? What's similar? What's different?

5. **Research Rust Alternatives:** What would it take to build FicHub in Python? In Node.js? In Go? What would you gain? What would you lose?

---

# Chapter 2: How the Web Works

Before we dive into code, let's make sure we're all on the same page about how web servers work. Even if you've built web apps before, a quick refresher will help when we encounter these concepts in Rust.

## HTTP: The Language of the Web

HTTP (HyperText Transfer Protocol) is how clients and servers talk to each other. When you type a URL into your browser or tap a link on your phone, your device sends an HTTP **request** to a server, and the server sends back an HTTP **response**.

**Real-world analogy:** HTTP is like a conversation between two people. The client says "I'd like to see page X" (the request), and the server responds with "Here's page X" (the response) or "Sorry, that page doesn't exist" (404 error). The conversation follows a specific format — just like a formal letter has a sender address, recipient address, subject line, and body.

### Request Methods

A request has three key parts:

1. **Method** — What the client wants to do:
   - `GET` — Fetch data (the most common)
   - `POST` — Send data to create something
   - `PUT` — Update existing data
   - `DELETE` — Remove data
   - `PATCH` — Partially update data
   - `HEAD` — Same as GET but without the body (used for checking if a resource exists)
   - `OPTIONS` — Ask the server what methods are allowed (used for CORS preflight)

2. **Path** — Which resource the client wants:
   - `/api/v0/epub?q=https://archiveofourown.org/works/123456`
   - `/cache/epub/abc123?h=def456`
   - `/api/v0/tags?url_id=abc123`

3. **Headers** — Metadata about the request:
   - `Content-Type: application/json` — What format the body is in
   - `Authorization: Bearer ***` — Authentication credentials
   - `User-Agent: fichub.net/0.1.0` — Who's making the request
   - `Accept: application/json` — What format the client wants back

### Response Structure

A response has:

1. **Status code** — Did it work?
   - `200 OK` — Success!
   - `201 Created` — Successfully created a resource
   - `301 Moved Permanently` — Resource has moved to a new URL
   - `304 Not Modified` — Cached version is still valid
   - `400 Bad Request` — Client sent invalid data
   - `401 Unauthorized` — Authentication required
   - `403 Forbidden` — Authenticated but not authorized
   - `404 Not Found` — Resource doesn't exist
   - `429 Too Many Requests` — Rate limited
   - `500 Internal Server Error` — Something went wrong on the server
   - `502 Bad Gateway` — Upstream server returned an error
   - `503 Service Unavailable` — Server is temporarily overloaded

2. **Headers** — Metadata about the response
   - `Content-Type: application/json` — Format of the response body
   - `Cache-Control: max-age=3600` — How long the client can cache this
   - `X-RateLimit-Remaining: 42` — How many requests the client has left

3. **Body** — The actual data (HTML, JSON, file bytes, etc.)

### HTTP/1.1 vs HTTP/2 vs HTTP/3

Most modern web applications use HTTP/1.1, but HTTP/2 and HTTP/3 offer significant improvements:

**HTTP/1.1** sends one request per TCP connection (or uses pipelining, which is rarely used). This means if you need to fetch 10 resources, you need 10 TCP connections (or you wait for each request to complete before sending the next).

**HTTP/2** multiplexes multiple requests over a single TCP connection. This means you can send 10 requests simultaneously without waiting for each one to complete. It also supports header compression and server push.

**HTTP/3** uses QUIC instead of TCP, which provides better performance on unreliable networks (like mobile connections) and eliminates head-of-line blocking.

FicHub primarily interacts with HTTP/1.1 when scraping fanfiction sites (because that's what they support), but the Axum server can handle HTTP/2 requests from modern clients.

## JSON: The Data Format

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

**Real-world analogy:** JSON is like a standardized form. Everyone knows how to read it, fill it out, and process it. XML is like a more verbose form with lots of extra fields. CSV is like a spreadsheet — simple but limited. JSON hits the sweet spot between simplicity and expressiveness.

### JSON in Rust with Serde

In Rust, we use the `serde` crate to convert between Rust structs and JSON. Here's how FicHub defines its API response:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    err: i32,
    url_id: String,
    meta: Option<StoryMeta>,
    urls: Option<DownloadUrls>,
}

#[derive(Serialize, Deserialize)]
struct StoryMeta {
    title: String,
    author: String,
    chapters: i32,
    words: i64,
    status: String,
}

#[derive(Serialize, Deserialize)]
struct DownloadUrls {
    epub: String,
    html: String,
}
```

The `#[derive(Serialize, Deserialize)]` attribute tells serde to automatically generate code for converting between Rust structs and JSON. This is a "derive macro" — a piece of code that runs at compile time and generates additional code for you.

When you return this struct from an Axum handler with `Json(response)`, Axum automatically serializes it to JSON and sets the `Content-Type: application/json` header.

### JSON Best Practices

1. **Use consistent naming conventions** — FicHub uses snake_case for JSON field names (`url_id`, `word_count`). This matches Rust's convention and avoids confusion.

2. **Include error information in every response** — The `err` field in every response means clients always know whether the request succeeded, even if the HTTP status code is 200.

3. **Use nullable fields carefully** — In JSON, a field can be `null`, missing, or present with a value. FicHub uses `Option<T>` in Rust to represent nullable fields, which serde serializes as `null` when the value is `None`.

4. **Version your API** — FicHub uses `/api/v0/` as the prefix for all API routes. This means we can add new versions without breaking existing clients.

## Databases: Where Data Lives

A web server needs to remember things. When FicHub scrapes a story's metadata, it needs to store it somewhere so it doesn't have to scrape again. When a user votes on a tag, that vote needs to persist. When the cache is checked, we need to know if a file was already generated.

FicHub uses **PostgreSQL** — a powerful, open-source relational database. It's known for reliability, feature richness, and excellent performance. It handles everything from simple key-value lookups to complex full-text searches with ease.

**Real-world analogy:** A database is like a filing cabinet. Tables are the drawers, rows are the files, and columns are the fields on each file. SQL is the language you use to find, add, update, and remove files. The database engine is the librarian who knows exactly where everything is and can find it quickly, even when the cabinet has millions of files.

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

### SQLx Compile-Time Checking

Here's how SQLx works in practice. When you write a query like this:

```rust
let fic: FicInfo = sqlx::query_as::<_, FicInfo>(
    "SELECT * FROM fic_info WHERE id = $1"
)
.bind(url_id)
.fetch_one(&pool)
.await?;
```

SQLx verifies at compile time that:
1. The table `fic_info` exists in your database
2. All columns in `SELECT *` exist in the table
3. The `$1` parameter is valid
4. The result columns match the `FicInfo` struct fields

If any of these checks fail, the code won't compile. This means you catch SQL errors at compile time instead of at runtime, which is a huge productivity win.

### Why PostgreSQL?

PostgreSQL is the best choice for FicHub because:

1. **Full-text search** — PostgreSQL has built-in full-text search with `tsvector` and `tsquery`. FicHub uses this for its search feature.

2. **JSON support** — PostgreSQL can store and query JSON data natively. FicHub uses this for extra metadata that doesn't fit the fixed schema.

3. **Extensions** — PostgreSQL has a rich extension ecosystem. FicHub uses `pg_trgm` for fuzzy search matching.

4. **Reliability** — PostgreSQL is battle-tested and handles crashes, power failures, and data corruption gracefully.

5. **Performance** — PostgreSQL handles millions of rows with sub-millisecond query times when properly indexed.

6. **ACID compliance** — All transactions are Atomic, Consistent, Isolated, and Durable. This means your data is always consistent, even in the face of errors or crashes.

### Database Schema

FicHub's database schema is defined in migration files. Here's a simplified version of the core tables:

```sql
-- Core fic metadata
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    updated TIMESTAMPTZ DEFAULT NOW(),
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INT4 NOT NULL DEFAULT 1,
    words INT8 NOT NULL DEFAULT 0,
    description TEXT DEFAULT '',
    fic_created TIMESTAMPTZ NOT NULL,
    fic_updated TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    source_id INT8,
    author_id INT8,
    content_hash TEXT,
    extra_meta JSONB,
    raw_extended_meta JSONB
);

-- Export log (cache tracking)
CREATE TABLE export_log (
    url_id VARCHAR(128) NOT NULL,
    version INT4 NOT NULL DEFAULT 1,
    etype VARCHAR(32) NOT NULL,
    input_hash VARCHAR(64) NOT NULL,
    export_hash VARCHAR(64) NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);

-- Request tracking
CREATE TABLE request_source (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    is_automated BOOLEAN DEFAULT FALSE,
    route TEXT,
    description TEXT
);

CREATE TABLE request_log (
    id INT8 GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    created TIMESTAMPTZ DEFAULT NOW(),
    source_id INT8 REFERENCES request_source(id),
    etype VARCHAR(32) NOT NULL,
    query TEXT NOT NULL,
    info_request_ms INT4,
    url_id VARCHAR(128),
    fic_info JSONB,
    export_ms INT4,
    export_file_name TEXT,
    export_file_hash TEXT,
    url TEXT
);
```

## Caching: Don't Do Work Twice

Imagine 100 users all request the same popular story within a minute. Without caching, FicHub would scrape the story 100 times — wasting bandwidth, annoying the upstream site, and making users wait.

With caching, the first request does the work, stores the result, and every subsequent request gets the cached version instantly.

**Real-world analogy:** Caching is like keeping a photocopy of a book you borrow from the library. The first time you need it, you go to the library and photocopy it. Every subsequent time, you just pull out your photocopy — no need to go back to the library. The photocopy is your cache.

FicHub uses a **two-layer cache**:

### Layer 1: Disk Cache

The actual EPUB/HTML files are stored on the filesystem in a hash-based directory structure. Files are organized by export type, then by url_id chunks, then by the content hash. This prevents any single directory from having too many files.

```
cache/
├── epub/
│   ├── abc/
│   │   ├── 123/
│   │   │   ├── 456/
│   │   │   │   ├── abc123456def/
│   │   │   │   │   └── a1b2c3d4e5f6.epub
├── html/
│   ├── def/
│   │   ├── 789/
│   │   │   ├── 012/
│   │   │   │   ├── def789012abc/
│   │   │   │   │   └── a1b2c3d4e5f6.zip
```

The directory structure splits the url_id into 3-character chunks. This keeps the number of files per directory manageable — important for filesystem performance.

### Layer 2: Database Cache

The `export_log` table tracks which files have been generated, so we can look up the hash without checking the filesystem. This is much faster than scanning directories.

The flow looks like this:
1. Request comes in for a story
2. Check `export_log` — is there a cached version?
3. If yes, return the cached file immediately
4. If no, scrape the story, generate the file, store it, and return it

### Cache Invalidation

One of the hardest problems in computer science is cache invalidation — knowing when to throw away cached data and regenerate it. FicHub handles this with several mechanisms:

1. **Content hash** — When a story is updated on the source site, its content hash changes, which causes a cache miss.

2. **Version bumps** — The `fic_version_bump` table allows manual cache invalidation.

3. **TTL (Time To Live)** — Some cached data has an expiration time.

4. **Manual invalidation** — Administrators can clear specific cached files.

## Rate Limiting: Being a Good Citizen

Fanfiction sites aren't huge enterprises with unlimited bandwidth. AO3 is run by volunteers. FF.net has been known to block aggressive scrapers. As a good net citizen, FicHub needs to limit how fast it makes requests.

Rate limiting works like a token bucket:
- Imagine a bucket that holds tokens
- Each request costs one token
- Tokens drip into the bucket at a steady rate
- If the bucket is empty, the request has to wait

**Real-world analogy:** Rate limiting is like having a limited number of movie tickets. You can buy tickets at a steady rate (say, one per minute). If you want to see multiple movies at once, you need to wait until you have enough tickets. If you try to buy more tickets than you have, you're told to wait.

FicHub implements this with Redis (a fast in-memory data store) and Lua scripts for atomic operations. The Lua script runs inside Redis, so the token check-and-update happens atomically — no race conditions possible. We'll dive deep into this in Chapter 19.

### Why Rate Limit?

1. **Be polite** — Don't overwhelm the upstream sites with requests
2. **Avoid blocks** — Aggressive scraping can get your IP banned
3. **Fair usage** — Ensure all users get a chance to make requests
4. **Cost control** — If using a paid proxy or API, rate limiting controls costs
5. **Stability** — Prevents cascading failures when upstream sites are slow

### Rate Limiting Architecture

FicHub uses a two-tier rate limiting approach:

1. **Global bucket** — Limits the total number of requests per IP across all endpoints
2. **Per-IP bucket** — Limits requests per individual IP address
3. **Per-site buckets** — Limits requests to specific fanfiction sites

The Lua script in Redis checks all three buckets atomically:

```lua
-- Lua script for atomic rate limit check
local key = KEYS[1]           -- e.g., "rate_limit:ip:192.168.1.1"
local max_tokens = tonumber(ARGV[1])  -- e.g., 60
local refill_rate = tonumber(ARGV[2]) -- e.g., 1 token per second
local now = tonumber(ARGV[3])         -- current time in seconds

-- Get current bucket state
local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1]) or max_tokens
local last_refill = tonumber(bucket[2]) or now

-- Refill tokens based on time elapsed
local elapsed = now - last_refill
tokens = math.min(max_tokens, tokens + elapsed * refill_rate)

-- Check if request is allowed
if tokens >= 1 then
    tokens = tokens - 1
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    redis.call('EXPIRE', key, math.ceil(max_tokens / refill_rate) + 10)
    return 1  -- allowed
else
    redis.call('HMSET', key, 'tokens', tokens, 'last_refill', now)
    return 0  -- denied
end
```

## The Request Lifecycle in FicHub

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

**Real-world analogy:** The request lifecycle is like ordering a pizza. You call the restaurant (HTTP request), the host checks if you're on the customer list (rate limiter), finds your order in the system (scraper registry), checks if your usual order is in the oven (cache check), and either hands it to you immediately or starts making a new one. Each step is simple, but together they create a complete experience.

## API Design Principles

FicHub follows a few API design principles worth noting:

**Versioned endpoints.** All API routes start with `/api/v0/`. This means we can add new versions without breaking existing clients. The `v0` prefix is a commitment to stability — once an endpoint is in v0, it won't change in a breaking way.

**Consistent error format.** Every response has an `err` field. `0` means success. Negative numbers are specific error codes. This makes it easy for clients to handle errors programmatically.

**Stateless where possible.** The server doesn't store user sessions. Rate limiting is IP-based. This simplifies deployment — you can run multiple instances behind a load balancer without sticky sessions.

**Content negotiation.** The same endpoint can return different formats. `/api/v0/epub` returns metadata, while `/cache/epub/{url_id}` returns the actual file.

**RESTful naming.** Resources are nouns (`/api/v0/tags`, `/api/v0/recommendations`), operations are HTTP methods (GET for read, POST for create/update, DELETE for remove).

## 📝 Practice Exercises

1. **Trace a Request:** Using the FicHub source code, trace the complete path of a request from `GET /api/v0/epub?q=...` to the JSON response. Write down each function that's called.

2. **Design an API:** Imagine you're adding a new endpoint `GET /api/v0/stats` that returns usage statistics. Design the request and response format. What HTTP status codes should it return for different scenarios?

3. **Cache Strategy:** Think about what data FicHub should cache and for how long. Write down your cache strategy for: story metadata, EPUB files, search results, and recommendation lists.

4. **Rate Limiting Rules:** Design rate limiting rules for FicHub. How many requests per minute should an IP be allowed? What about per site? What about for API keys vs. anonymous users?

5. **Database Schema:** Design a simple database schema for a bookmarking feature where users can save stories. What tables do you need? What columns? What indexes?

---

# Chapter 3: Setting Up Your Workshop

Time to get our hands dirty. In this chapter, we'll install everything we need to build FicHub and make sure it all works together. By the end, you'll have Rust, PostgreSQL, Redis, and all the tools running on your machine.

## Installing Rust

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

**Real-world analogy:** Installing Rust is like setting up a professional workshop. `rustup` is the workshop manager who ensures all the tools are the right version and work together. `rustc` is the compiler (the power tool that turns your code into an executable). `cargo` is the project manager who handles dependencies, builds, and tests.

### Understanding the Toolchain

Rustup manages multiple toolchains. When the FicHub project specifies `edition = "2024"` in `Cargo.toml`, rustup will automatically download the right compiler version when you build. You don't need to manually manage this — just run `cargo build` and rustup handles the rest.

You can check which toolchain is active with:

```bash
rustup show
```

This shows your default toolchain, host triple, and installed targets.

### Updating Rust

Rust releases a new stable version every 6 weeks. To update:

```bash
rustup update
```

This updates the compiler, cargo, and all installed toolchains. It's a good practice to update regularly, but be aware that new editions may introduce breaking changes (though these are rare in stable releases).

### Installing Additional Targets

If you want to cross-compile for other platforms (like ARM for Raspberry Pi), you can add targets:

```bash
# For ARM64 (e.g., Raspberry Pi)
rustup target add aarch64-unknown-linux-gnu

# For ARM32
rustup target add armv7-unknown-linux-gnueabihf
```

We'll use this in Chapter 28 when we deploy to a remote server.

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

## Installing PostgreSQL

FicHub requires PostgreSQL 14 or later. The installation depends on your operating system.

### On Arch/Manjaro

```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl enable --now postgresql
```

The `initdb` command creates the initial database cluster. The `-D` flag specifies where to store the data files.

**Understanding initdb:** The `initdb` command creates a new PostgreSQL database cluster — a directory structure that PostgreSQL uses to store data. It initializes the system catalogs, creates template databases, and sets up the default configuration. Think of it like formatting a new hard drive before you can use it.

### On Ubuntu/Debian

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl enable --now postgresql
```

The `postgresql-contrib` package includes useful extensions like `pg_trgm` (trigram matching for fuzzy search).

**Understanding pg_hba.conf:** On Debian-based systems, PostgreSQL uses a file called `pg_hba.conf` to control client authentication. The "HBA" stands for "Host-Based Authentication." This file specifies which users can connect from which hosts using which authentication methods.

### On macOS

```bash
brew install postgresql@16
brew services start postgresql@16
```

### Via Docker (recommended for beginners)

```bash
docker run -d \
  --name fichub-postgres \
  -e POSTGRES_DB=fichub \
  -e POSTGRES_USER=fichub \
  -e POSTGRES_PASSWORD=fichub \
  -p 5432:5432 \
  postgres:16-alpine
```

Docker is the easiest way to get started because it handles all the system-level configuration automatically. The `-d` flag runs it in detached mode (background), `--name` gives it a friendly name, and `-p 5432:5432` maps the container's port to your host.

**Understanding Docker:** Docker is a platform for running applications in isolated containers. Think of a container as a lightweight virtual machine — it has its own filesystem, network, and process space, but it shares the host's kernel. This makes containers much faster to start and use less memory than full VMs.

**Understanding Docker Ports:** The `-p 5432:5432` flag maps port 5432 inside the container to port 5432 on your host. This means you can connect to PostgreSQL at `localhost:5432` from your host machine, even though PostgreSQL is running inside a Docker container.

### Creating the Database User

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

**Understanding PostgreSQL Users:** In PostgreSQL, users are login roles. When you create a user with `CREATE USER`, PostgreSQL creates a role with the `LOGIN` privilege. The `GRANT ALL PRIVILEGES` command gives the user full access to the database.

### Testing the Connection

Test the connection:

```bash
psql -h localhost -U fichub -d fichub -c "SELECT 1;"
```

If you see `?column?` with value `1`, you're connected.

**Understanding the psql Command:** The `psql` command is PostgreSQL's interactive terminal. The flags mean:
- `-h localhost` — Connect to the server on localhost
- `-U fichub` — Connect as the user "fichub"
- `-d fichub` — Connect to the database "fichub"
- `-c "SELECT 1;"` — Execute the SQL command and exit

### Troubleshooting PostgreSQL

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

**Understanding Authentication Methods:**
- **peer** — Authenticates by the OS username matching the PostgreSQL username. Only works for local connections.
- **md5** — Uses MD5-hashed passwords. Good for most use cases.
- **scram-sha-256** — Uses the SCRAM-SHA-256 protocol. More secure than MD5 but requires newer clients.
- **trust** — Allows all connections without authentication. Never use this in production!

## Installing Redis

Redis is used for rate limiting and the recommendation worker queue. It's lightweight and easy to install.

### On Arch/Manjaro

```bash
sudo pacman -S redis
sudo systemctl enable --now redis
```

### On Ubuntu/Debian

```bash
sudo apt install redis-server
sudo systemctl enable --now redis-server
```

### On macOS

```bash
brew install redis
brew services start redis
```

### Via Docker

```bash
docker run -d \
  --name fichub-redis \
  -p 6379:6379 \
  redis:7-alpine
```

### Testing Redis

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

**Real-world analogy:** Redis is like a super-fast whiteboard. You can write data on it (SET), read data from it (GET), and erase it (DEL). Everything happens in memory, so it's incredibly fast — typically under a millisecond. The downside is that data is lost when Redis restarts (unless you configure persistence). For rate limiting, this is fine because rate limit state is ephemeral.

**Understanding Redis Data Structures:**
- **Strings** — Simple key-value pairs. Used for rate limit counters.
- **Hashes** — Maps of field-value pairs. Used for token bucket state.
- **Lists** — Ordered lists of strings. Used for work queues.
- **Sets** — Unordered collections of unique strings. Used for tracking unique visitors.
- **Sorted Sets** — Like sets but with scores. Used for leaderboards.
- **Streams** — Append-only logs. Used for event sourcing.

## Setting Up the Project

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
DATABASE_URL=postgres://fichub:***@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug
```

The `DATABASE_URL` format is `postgres://user:password@host:port/database`. The `REDIS_URL` format is `redis://host:port`. The `RUST_LOG` variable controls logging verbosity — `info` for general messages, `fichub=debug` for detailed FicHub-specific messages.

**Understanding Environment Variables:** Environment variables are key-value pairs that are set in your shell and passed to programs. They're a common way to configure applications without hardcoding values in source code. The `.env` file is a convenience — the `dotenvy` crate reads this file and sets the environment variables automatically.

### Installing sqlx-cli

Install the `sqlx-cli` tool for running migrations:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

**Understanding sqlx-cli:** This is a command-line tool for managing SQLx databases. It can run migrations, check queries at compile time, and generate code. The `--no-default-features --features postgres` flags mean we only want PostgreSQL support (not MySQL or SQLite).

### Running Migrations

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

**Understanding Migrations:** A migration is a versioned SQL script that modifies the database schema. Migrations are numbered sequentially and are applied in order. Once applied, they're never reapplied. This ensures that every developer and every deployment has the same database schema.

### Building the Project

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

**Understanding Cargo Build:** Cargo is Rust's build system and package manager. When you run `cargo build`, it:
1. Reads `Cargo.toml` to find dependencies
2. Downloads missing dependencies from crates.io
3. Compiles all dependencies
4. Compiles your project
5. Links everything into a binary

The `dev` profile is for development — it compiles quickly but produces slower binaries. The `release` profile produces optimized binaries but takes longer to compile. Use `cargo build --release` for production builds.

## Your Editor Setup

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

**Real-world analogy:** rust-analyzer is like having a expert Rust developer looking over your shoulder as you code. It catches mistakes before you make them, suggests improvements, and helps you navigate the codebase. It's the difference between writing with a pencil (no feedback) and writing with a smart pen that corrects your spelling and grammar in real-time.

Other excellent editors for Rust include Neovim (with rust-analyzer LSP), Helix, and Emacs (with rustic/lsp-mode). Any editor with LSP support works well.

### rust-analyzer Tips and Tricks

1. **Hover for types** — Hover over any variable or function to see its type. This is incredibly helpful when learning Rust's type inference.

2. **Ctrl+Click for navigation** — Ctrl+Click (or Cmd+Click on macOS) on any function, struct, or trait to jump to its definition.

3. **Find All References** — Right-click on any symbol and select "Find All References" to see everywhere it's used.

4. **Rename Symbol** — Right-click on any symbol and select "Rename Symbol" to rename it everywhere in the codebase.

5. **Quick Fix** — When you see a red squiggly line, press Ctrl+. to see available quick fixes.

6. **Inline Type Hints** — Enable inlay hints to see inferred types next to variables. This helps you understand what Rust's type inference is doing.

🧪 **Try It Yourself:** Open the FicHub project in VS Code and navigate to `src/main.rs`. You should see syntax highlighting and rust-analyzer annotations. Try hovering over `config::Config` to see the type information. Try Ctrl+clicking on `Config::from_env` to jump to its definition.

## Understanding the Dependency Graph

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

# Cryptographic hashing
md5 = "0.7"
sha2 = "0.10"
hex = "0.4"

# UUID generation
uuid = { version = "1", features = ["v4"] }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Regex
regex-lite = "0.1"

# Async traits
async-trait = "0.1"

# ZIP file creation
zip = "2"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Environment variables
dotenvy = "0.15"

# Error handling
thiserror = "2"

# IP address parsing
ipnet = "2"
```

Let's break down the key dependencies:

**Axum** is the web framework. It's built on Tower (middleware framework) and Tokio (async runtime). Axum is designed for type-safety and ergonomics — you write handlers as regular functions and Axum extracts the parameters using Rust's type system.

**Tokio** is the async runtime. It provides the event loop, task scheduler, timers, and I/O primitives. The `features = ["full"]` enables all Tokio features, including timers, IO, signal handling, and more.

**Tower** is the middleware framework. It provides abstractions for services (things that handle requests) and layers (middleware that wrap services). Axum's middleware is built on Tower.

**Tower-HTTP** provides HTTP-specific middleware like CORS, compression, and static file serving.

**SQLx** is the database library. It provides compile-time checked SQL queries, connection pooling, and async support. The feature flags enable PostgreSQL, chrono (date/time), UUID, migrations, TLS, and derive macros.

**Redis** is the Redis client. The `aio` feature enables async support, and `tokio-comp` enables compatibility with the Tokio runtime.

**Reqwest** is the HTTP client. It's used to make outgoing requests to fanfiction sites. The `rustls-tls` feature uses the Rustls TLS library instead of OpenSSL, which is easier to cross-compile.

**Scraper** is the HTML parser. It provides CSS selector-based querying of HTML documents.

**EPUB-Builder** is the EPUB generation library. It handles the EPUB format's complexities so we don't have to.

**Tera** is the template engine. It's used for generating HTML templates (like the API docs page).

**Serde** is the serialization framework. It handles converting between Rust structs and JSON (or other formats). The `derive` feature enables the `#[derive(Serialize, Deserialize)]` attributes.

### Why These Specific Versions?

You might notice that the dependencies use specific version numbers (like `axum = "0.8"`) rather than exact versions (like `axum = "=0.8.0"`). This is called "semver compatibility" — Cargo will accept any version that's compatible with the specified version.

For example, `axum = "0.8"` means "any version 0.8.x where x >= 0." This gives you bug fixes and minor improvements without breaking changes. Major version bumps (like going from 0.8 to 0.9) may have breaking changes, so you'd need to update your code.

### The Dependency Tree

When you run `cargo tree`, you can see the complete dependency tree:

```bash
cargo tree
```

This shows every dependency and its sub-dependencies. FicHub has hundreds of transitive dependencies — code that its direct dependencies rely on. This is normal for a modern Rust project. The Rust ecosystem is rich, and we stand on the shoulders of thousands of open-source contributors.

## 📝 Practice Exercises

1. **Toolchain Check:** Run `rustup show` and write down the output. What toolchain are you using? What host triple?

2. **Cargo Exercises:** Create a new Cargo project and add the following dependencies one at a time. After each addition, run `cargo build` and observe what happens:
   - `serde = "1"`
   - `reqwest = "0.12"`
   - `tokio = { version = "1", features = ["full"] }`

3. **PostgreSQL Exploration:** Connect to PostgreSQL with `psql` and run:
   - `\dt` — List all tables
   - `\d fic_info` — Describe the fic_info table
   - `SELECT COUNT(*) FROM fic_info;` — Count all stories

4. **Redis Exploration:** Use `redis-cli` to:
   - Set and get a value
   - Set a key with a TTL (expiration time)
   - List all keys matching a pattern

5. **Editor Setup:** Install rust-analyzer and verify it works by opening a Rust file and checking for syntax highlighting and type hints.

---

# Chapter 4: Your First Rust Program

Now that our workshop is set up, let's write some Rust code. In this chapter, we'll learn the fundamental Rust concepts we need for building FicHub: variables, functions, types, error handling, and the ownership system.

## Rust Basics

Let's start with the basics. Open your editor and create a new file called `basics.rs`:

```rust
fn main() {
    println!("Hello, FicHub!");
}
```

Run it with:

```bash
rustc basics.rs
./basics
```

Or better yet, use Cargo:

```bash
cargo new basics
cd basics
cargo run
```

The `cargo new` command creates a new project with the standard structure. The `cargo run` command compiles and runs it.

### Variables and Mutability

In Rust, variables are immutable by default. This means once you assign a value to a variable, you can't change it:

```rust
fn main() {
    let x = 5;
    x = 6;  // Error: cannot assign twice to immutable variable
}
```

To make a variable mutable, use `mut`:

```rust
fn main() {
    let mut x = 5;
    println!("x = {}", x);  // x = 5
    x = 6;
    println!("x = {}", x);  // x = 6
}
```

**Real-world analogy:** Immutable variables are like constants written in stone — they can't be changed. Mutable variables are like variables written on a whiteboard — they can be erased and rewritten. Rust defaults to "stone" because it's safer. You have to explicitly opt into "whiteboard" mode with `mut`.

### Types

Rust is a statically typed language, which means every variable has a known type at compile time. However, Rust can usually infer the type from the context, so you don't always need to specify it explicitly:

```rust
fn main() {
    let x = 5;           // Rust infers i32
    let y = 3.14;        // Rust infers f64
    let name = "FicHub"; // Rust infers &str
    let active = true;   // Rust infers bool
    
    // You can specify the type explicitly
    let x: i64 = 5;
    let y: f32 = 3.14;
}
```

Here are the common types you'll use:

- **Integers:** `i8`, `i16`, `i32`, `i64`, `i128` (signed), `u8`, `u16`, `u32`, `u64`, `u128` (unsigned)
- **Floating-point:** `f32`, `f64`
- **Boolean:** `bool` (true or false)
- **Character:** `char` (a single Unicode character)
- **String slice:** `&str` (a reference to a string)
- **Owned string:** `String` (a growable string)

In FicHub, we use specific types for clarity:

```rust
pub struct FicInfo {
    pub id: String,           // Unique identifier
    pub title: String,        // Story title
    pub author: String,       // Author name
    pub chapters: i32,        // Number of chapters
    pub words: i64,           // Word count (can be very large)
    pub status: String,       // "ongoing", "complete", etc.
}
```

### Functions

Functions in Rust are defined with `fn`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = return value
}

fn greet(name: &str) {
    println!("Hello, {}!", name);  // Semicolon = no return value
}

fn main() {
    let sum = add(3, 4);
    greet("FicHub");
    println!("3 + 4 = {}", sum);
}
```

Key differences from other languages:
- Parameters must have type annotations
- Return type is specified after `->`
- The last expression without a semicolon is the return value
- No parentheses around conditionals (no `if (x > 5)` — just `if x > 5`)

### Ownership

Ownership is Rust's most unique feature. Every value has exactly one owner. When the owner goes out of scope, the value is dropped (freed). This eliminates the need for a garbage collector:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is moved to s2
    // println!("{}", s1);  // Error: s1 is no longer valid
    println!("{}", s2);  // This works
}
```

**Real-world analogy:** Ownership is like a library book checkout system. Only one person can check out a book at a time. When you "check out" a book (move it to a new variable), the original person no longer has it. When you're done with the book (the variable goes out of scope), it's returned to the library (freed).

To share data without transferring ownership, use references:

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);  // Borrow s1
    println!("{} has length {}", s1, len);  // s1 is still valid
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

References are like library cards — they let you look at the data without taking it home. You can have multiple references at once, but they must all be immutable (read-only), or exactly one can be mutable (read-write).

This is Rust's "borrow checker" — it ensures that references are always valid and that you never have data races. It's the source of many compile-time errors for beginners, but once you understand it, you'll appreciate the safety it provides.

### Error Handling

Rust doesn't have exceptions. Instead, it uses `Result<T, E>` for recoverable errors and `panic!` for unrecoverable errors:

```rust
use std::fs;

fn read_config() -> Result<String, std::io::Error> {
    let content = fs::read_to_string("config.toml")?;
    Ok(content)
}

fn main() {
    match read_config() {
        Ok(config) => println!("Config: {}", config),
        Err(e) => println!("Error reading config: {}", e),
    }
}
```

The `?` operator is Rust's error propagation shorthand. If the result is an `Err`, it returns early from the function. If it's an `Ok`, it unwraps the value. This is much cleaner than nested `match` statements.

In FicHub, we define custom error types:

```rust
#[derive(Debug)]
pub enum AppError {
    BadRequest(i32, String),
    RateLimited(u64),
    NotFound(String),
    Internal(String),
    ScrapeError(String),
    ExportError(String),
    Database(String),
    CacheError(String),
}

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

This enum represents all the different ways FicHub can fail. Each variant carries the information needed to diagnose the problem. The `Display` implementation provides a human-readable error message.

### Enums and Pattern Matching

Enums in Rust are much more powerful than in most languages. Each variant can carry data:

```rust
enum RateLimitResult {
    Allowed,
    Wait(u64),      // Wait N seconds
    Blocked,        // IP is blocked
}

fn check_rate_limit(ip: &str) -> RateLimitResult {
    // ... rate limiting logic ...
    RateLimitResult::Allowed
}

fn main() {
    let result = check_rate_limit("192.168.1.1");
    match result {
        RateLimitResult::Allowed => println!("Request allowed"),
        RateLimitResult::Wait(secs) => println!("Please wait {} seconds", secs),
        RateLimitResult::Blocked => println!("Your IP is blocked"),
    }
}
```

Pattern matching with `match` is exhaustive — the compiler ensures you handle every possible variant. If you add a new variant to the enum, the compiler will tell you every `match` statement that needs to be updated.

### Structs

Structs are Rust's way of creating custom types with named fields:

```rust
#[derive(Debug, Clone)]
struct Chapter {
    chapter_id: i32,
    title: String,
    content: String,
}

impl Chapter {
    fn new(id: i32, title: &str, content: &str) -> Self {
        Chapter {
            chapter_id: id,
            title: title.to_string(),
            content: content.to_string(),
        }
    }
    
    fn word_count(&self) -> usize {
        self.content.split_whitespace().count()
    }
}

fn main() {
    let chapter = Chapter::new(1, "Introduction", "This is the first chapter.");
    println!("Chapter: {:?}", chapter);
    println!("Words: {}", chapter.word_count());
}
```

The `#[derive(Debug, Clone)]` attributes automatically implement the `Debug` trait (for printing) and the `Clone` trait (for copying). The `impl` block defines methods on the struct.

### Traits

Traits are Rust's way of defining shared behavior. They're similar to interfaces in other languages:

```rust
trait Scraper {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, url: &str) -> Result<FicMetadata, ScrapeError>;
}

struct Ao3Scraper;
struct FfNetScraper;

impl Scraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }
    
    async fn lookup(&self, url: &str) -> Result<FicMetadata, ScrapeError> {
        // AO3-specific scraping logic
        todo!()
    }
}

impl Scraper for FfNetScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("fanfiction.net")
    }
    
    async fn lookup(&self, url: &str) -> Result<FicMetadata, ScrapeError> {
        // FF.net-specific scraping logic
        todo!()
    }
}
```

Traits enable polymorphism. You can write code that works with any type that implements the trait:

```rust
async fn get_metadata(scraper: &dyn Scraper, url: &str) -> Result<FicMetadata, ScrapeError> {
    scraper.lookup(url).await
}
```

The `dyn Scraper` is a "trait object" — a value whose type is only known at runtime. This is how FicHub supports multiple scraping backends with a single interface.

### Closures

Closures are anonymous functions that can capture variables from their surrounding scope:

```rust
fn main() {
    let threshold = 1000;
    
    let is_long = |word_count: i64| -> bool {
        word_count > threshold  // Captures `threshold` from outer scope
    };
    
    println!("500 words is long: {}", is_long(500));
    println!("2000 words is long: {}", is_long(2000));
}
```

Closures are used extensively in Rust, especially with iterators:

```rust
fn main() {
    let words = vec!["hello", "world", "foo", "bar"];
    let long_words: Vec<&str> = words
        .into_iter()
        .filter(|w| w.len() > 3)
        .collect();
    println!("{:?}", long_words);  // ["hello", "world"]
}
```

### Iterators

Iterators are a fundamental Rust pattern. They provide a way to process sequences of values:

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    // Map: transform each element
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    println!("Doubled: {:?}", doubled);  // [2, 4, 6, 8, 10]
    
    // Filter: keep only elements that match
    let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();
    println!("Evens: {:?}", evens);  // [2, 4]
    
    // Fold: combine all elements into a single value
    let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);
    println!("Sum: {}", sum);  // 15
    
    // Chaining
    let result: i32 = numbers
        .iter()
        .filter(|x| *x % 2 == 0)
        .map(|x| x * 3)
        .sum();
    println!("Evens times 3: {}", result);  // 18
}
```

Iterators in Rust are zero-cost — the compiler optimizes them into code that's as fast as hand-written loops. This means you can use expressive iterator chains without worrying about performance.

### Lifetime Annotations

Lifetimes are Rust's way of tracking how long references are valid. Most of the time, the compiler can infer lifetimes automatically. But sometimes you need to annotate them explicitly:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn main() {
    let result;
    let string1 = String::from("long string");
    {
        let string2 = String::from("xyz");
        result = longest(&string1, &string2);
        println!("Longest: {}", result);
    }
    // Can't use `result` here because `string2` was dropped
}
```

The `'a` is a lifetime parameter. It says "the returned reference lives as long as the shorter of the two input lifetimes." This prevents dangling references — references that point to freed memory.

**Real-world analogy:** Lifetimes are like expiration dates on food. A reference is only valid as long as the data it points to. If the data expires (is dropped), the reference becomes invalid. The lifetime annotation tells the compiler how long the reference needs to be valid.

In FicHub, you'll see lifetimes in function signatures and struct definitions, but the compiler infers them in most cases. Don't worry too much about lifetimes for now — the compiler will help you when you get it wrong.

## Putting It All Together

Let's write a small program that uses all these concepts:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Story {
    title: String,
    author: String,
    word_count: i64,
    tags: Vec<String>,
}

impl Story {
    fn new(title: &str, author: &str, word_count: i64, tags: Vec<String>) -> Self {
        Story {
            title: title.to_string(),
            author: author.to_string(),
            word_count,
            tags,
        }
    }
    
    fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

trait Searchable {
    fn matches_query(&self, query: &str) -> bool;
}

impl Searchable for Story {
    fn matches_query(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self.author.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
    }
}

fn build_index(stories: &[Story]) -> HashMap<String, Vec<usize>> {
    let mut index: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, story) in stories.iter().enumerate() {
        for tag in &story.tags {
            index
                .entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(i);
        }
    }
    index
}

fn main() {
    let stories = vec![
        Story::new("Harry Potter and the Methods of Rationality", "Less Wrong", 660000, vec!["Harry Potter".into(), "Rational Fiction".into()]),
        Story::new("The Metaworld Chronicles", "Wutosama", 1200000, vec!["Harry Potter".into(), "Magic".into()]),
        Story::new("A Practical Guide to Evil", "ErraticErrata", 2800000, vec!["Fantasy".into(), "Strong Female Lead".into()]),
    ];
    
    // Build inverted index
    let index = build_index(&stories);
    println!("Index: {:?}", index);
    
    // Search for stories
    let query = "harry";
    let results: Vec<&Story> = stories
        .iter()
        .filter(|s| s.matches_query(query))
        .collect();
    
    println!("Search results for '{}':", query);
    for story in &results {
        println!("  - {} by {} ({} words)", story.title, story.author, story.word_count);
    }
    
    // Find long stories
    let long_stories: Vec<&Story> = stories
        .iter()
        .filter(|s| s.word_count > 1000000)
        .collect();
    
    println!("\nLong stories (>1M words):");
    for story in &long_stories {
        println!("  - {} ({} words)", story.title, story.word_count);
    }
}
```

This small program demonstrates:
- **Structs** with methods (`Story`)
- **Traits** for shared behavior (`Searchable`)
- **Enums** (via `HashMap`'s `Entry` API)
- **Pattern matching** (via `any` and closures)
- **Ownership and borrowing** (references throughout)
- **Iterators** (chains of `filter`, `map`, `any`)
- **Lifetimes** (implicitly, in the return types)

## 📝 Practice Exercises

1. **Implement a Trait:** Create a `WordCounter` trait with a `word_count(&self) -> usize` method. Implement it for both `String` and `&str`. Write a function that takes any type implementing `WordCounter` and returns the word count.

2. **Ownership Practice:** Write a function that takes ownership of a `String` and returns it back. Then write a function that borrows a `String` and returns its length. Explain the difference.

3. **Error Handling:** Write a function that divides two numbers and returns a `Result<i32, String>`. Handle the division-by-zero case. Then write a function that calls this one and propagates the error with `?`.

4. **Pattern Matching:** Create an enum `ExportFormat` with variants `Epub`, `Html`, `Mobi`, and `Pdf`. Write a function that takes this enum and returns the file extension. Then write a function that takes a URL and returns the appropriate format.

5. **Iterator Practice:** Given a vector of `Story` structs, use iterators to:
   - Find the story with the most words
   - Group stories by author
   - Find all stories with a specific tag
   - Calculate the average word count

6. **Build a Mini Scraper:** Write a trait `SiteChecker` with a method `can_handle(&self, url: &str) -> bool`. Implement it for at least 3 different "sites" (you can just check URL patterns). Write a function that takes a URL and finds the right checker.

