# Building the FicHub Backend

## A Complete Guide to Building a Rust Fanfiction Download Server

**Author:** FicHub Contributors
**Version:** 1.0
**Chapters:** 36

---

# Part 1: Welcome

---

# Chapter 1: What We're Building

## Welcome to FicHub

Have you ever been reading a fanfiction on Archive of Our Own, and thought to yourself, "I wish I could download this onto my e-reader and read it on the train"? Or maybe you've been browsing FanFiction.net and wished there was a simple, clean way to grab a story as an EPUB file you could carry around? If so, then you already understand why FicHub exists — and by the time you finish this book, you'll understand exactly how it works, right down to every line of Rust code that makes it tick.

FicHub is a self-hosted fanfiction download server. You give it a URL from one of several popular fanfiction sites — Archive of Our Own (AO3), FanFiction.net (FF.net), XenForo forums like SpaceBattles and SufficientVelocity, FictionPress, AdultFanFiction.org, and the Harry Potter Fan Fiction Archive — and it gives you back a beautifully formatted EPUB file, an HTML bundle, or even a MOBI or PDF file that you can read anywhere, anytime, on any device.

But FicHub is much more than just a scraper and converter. It's a full-featured web application with:

- **A recommendation engine** that uses collaborative filtering to suggest similar stories based on what other readers have bookmarked
- **A community tagging system** where users can submit, vote on, and flag tags for fanfiction
- **An advanced search system** with full-text search, tag filters, and complex queries
- **An OPDS catalog** that lets e-readers like Kindle, Kobo, and Calibre browse and download stories directly
- **A rate limiter** that protects both our server and the fanfiction sites we scrape from excessive traffic
- **A disk cache** that ensures we only scrape each story once, then serve it from fast local storage

All of this is built in Rust, and all of it is open source.

## Why This Book Exists

This book is a complete, end-to-end walkthrough of how the FicHub backend is built. We're going to look at every module, every function, every database query, and every architectural decision. We'll start from nothing — not even a `Hello, World!` — and build up to a production-ready server that can handle thousands of requests.

More importantly, this book is designed to be *educational*. Every chapter teaches you something new about Rust, web development, or system design. We assume you know a little bit of programming — maybe you've written some Python scripts or played with JavaScript — but we don't assume you know Rust, web servers, databases, or any of the other technologies we'll be using. We'll explain everything from the ground up.

Here's what you'll learn:

**Part 1 (Chapters 1-4):** We'll set up your development environment, introduce the Rust programming language, and explain the basics of how web servers work. By the end of Part 1, you'll be able to write a simple HTTP server in Rust.

**Part 2 (Chapters 5-10):** We'll build the core of FicHub using Axum, Rust's premier web framework. You'll learn about routing, database connections, migrations, CRUD operations, and middleware. By the end of Part 2, you'll have a working API server connected to PostgreSQL.

**Part 3 (Chapters 11-15):** We'll dive into the scraping system — the heart of FicHub. You'll learn how to parse HTML from fanfiction sites, build a pluggable scraper system, and handle the quirks of different sites. We'll take a deep look at the AO3 scraper and see how to add support for new sites.

**Part 4 (Chapters 16-20):** We'll explore the export system — how FicHub generates EPUB files, HTML bundles, and uses Calibre for format conversion. You'll learn about disk caching with hash-based directories, rate limiting with token buckets, and the complete 17-step export flow.

**Part 5 (Chapters 21-24):** We'll build the recommendation engine from scratch. You'll learn about collaborative filtering, the Jaccard coefficient, co-occurrence matrices, and how to build a background worker that scrapes user favourites to populate the recommendation tables.

**Part 6 (Chapters 25-28):** We'll put it all together into a production-ready server. You'll learn about the full route tree, static file serving, Docker deployment, and cross-compiling for ARM64 to deploy on a Raspberry Pi or Orange Pi.

**Part 7 (Chapters 29-33):** We'll explore advanced features — the full-text search system, the tag resolution and curation system, the OPDS catalog, API documentation, and performance optimization techniques.

**Part 8 (Chapters 34-36):** We'll cover testing, production hardening (TLS, logging, monitoring, backups), and point you toward ideas for extending FicHub.

## The Big Picture

Let's zoom out and look at how FicHub works at a high level. When a user visits FicHub's website and pastes in a URL like `https://archiveofourown.org/works/12345678`, here's what happens:

1. **The frontend** (a SvelteKit app) sends a request to the backend API
2. **The API** looks up the appropriate scraper for the URL
3. **The scraper** fetches the fanfiction site's page and parses out the metadata — title, author, word count, chapters, description
4. **The metadata** is stored in PostgreSQL for future reference
5. **The cache** is checked — if we've already generated an EPUB for this story, we skip to step 11
6. **The scraper** fetches each chapter's content from the fanfiction site
7. **The EPUB generator** assembles the chapters into a well-formatted EPUB file
8. **The HTML generator** creates a standalone HTML bundle with chapter navigation
9. **Calibre** optionally converts the EPUB to MOBI or PDF
10. **The cache** stores all generated files on disk
11. **The API** returns JSON with download URLs for all available formats
12. **The frontend** displays the download links to the user
13. **The user** clicks a download link
14. **The cache download handler** verifies the file's MD5 hash and serves it

Meanwhile, in the background:

- The **recommender** scrapes user favourites from fanfiction sites and builds a co-occurrence matrix
- The **tag system** collects user-submitted tags and counts votes
- The **OPDS catalog** serves Atom XML feeds that e-readers can browse
- The **rate limiter** ensures we don't overwhelm the fanfiction sites

This is a lot of moving parts, but they all fit together beautifully. The key architectural insight is **separation of concerns** — each module has a single job, and they communicate through well-defined interfaces.

## Why Rust?

FicHub is written in Rust, and there are several good reasons for this choice:

**Performance.** Rust compiles to native machine code with no garbage collector overhead. For a web server that needs to handle many concurrent requests while doing CPU-intensive work like HTML parsing and EPUB generation, this matters. A single FicHub instance can serve hundreds of requests per second on modest hardware.

**Safety.** Rust's ownership system prevents entire classes of bugs — null pointer dereferences, buffer overflows, data races, use-after-free — at compile time. In a long-running server application, these kinds of bugs can cause crashes, memory leaks, or security vulnerabilities. Rust catches them before the code ever runs.

**Concurrency.** Rust's async/await system, combined with Tokio, makes it straightforward to handle thousands of concurrent connections. The borrow checker ensures that shared state is accessed safely, so you don't have to worry about race conditions.

**Ecosystem.** The Rust ecosystem has excellent crates for everything FicHub needs:
- **Axum** for the web framework
- **SQLx** for PostgreSQL access with compile-time checked queries
- **Redis** for caching and rate limiting
- **reqwest** for making HTTP requests to fanfiction sites
- **scraper** for parsing HTML
- **epub-builder** for generating EPUB files
- **tokio** for the async runtime

**Deployment.** Rust produces a single static binary with no runtime dependencies. This makes deployment trivial — you just copy the binary to your server and run it. No need to install Python, Node.js, or any runtime. Cross-compilation for ARM64 (for deploying on Raspberry Pi or Orange Pi) is straightforward.

Of course, Rust has a learning curve. The borrow checker can be frustrating at first, and the type system is more complex than what you're used to from dynamically-typed languages. But once you internalize the mental model, Rust becomes remarkably productive. And the peace of mind that comes from knowing your code is free of data races and null pointer bugs? That's priceless.

## A Note About the Original FicHub

The original FicHub (fichub.net) is a closed-source Python application that has served the fanfiction community well for years. FicHub-rs is not a fork of that project — it's a clean-room reimplementation in Rust, designed from the ground up for performance, maintainability, and self-hosting.

The original FicHub inspired the feature set and API design, but the implementation is entirely new. We've taken the best ideas from the original and improved upon them where Rust's strengths allow — particularly in performance, memory safety, and ease of deployment.

If you're running the original FicHub and considering switching, the API is compatible, so your existing tools and integrations will continue to work. And if you're new to fanfiction download tools, FicHub-rs gives you a modern, fast, self-hosted solution that you can customize and extend.

## What You'll Build

By the end of this book, you'll have a complete understanding of every component in FicHub:

- A web server built on Axum that handles 40+ API routes
- A database layer using SQLx with PostgreSQL, including 4 migration files and dozens of queries
- A pluggable scraping system supporting 6 fanfiction sites
- An EPUB and HTML export pipeline with disk caching and hash-based validation
- A token bucket rate limiter backed by Redis with Lua scripting
- A collaborative filtering recommendation engine with background workers
- A community tagging system with voting, flagging, and curator tools
- An advanced search system with full-text search and complex tag filters
- An OPDS catalog for e-reader integration
- Docker deployment and cross-compilation for ARM64

This is not a toy project. It's a production application with real users, real data, and real-world constraints. The code you'll study has been tested, deployed, and refined through actual use. Every architectural decision has a reason, and we'll explore those reasons together.

Let's get started.

---

# Chapter 2: How the Web Works

## The Internet in a Nutshell

Before we dive into code, let's make sure we're all on the same page about how the internet works. Don't worry — we'll keep this high-level and practical. You don't need to know about TCP window sizes or BGP routing to build a web application. But you do need to understand the request-response cycle, because that's the foundation everything else is built on.

When you type `https://archiveofourown.org/works/12345678` into your browser and press Enter, here's what happens (simplified):

1. Your browser looks up the IP address of `archiveofourown.org` using DNS
2. Your browser establishes a TCP connection to that IP address on port 443 (HTTPS)
3. Your browser sends an HTTP request — specifically, a `GET` request for the path `/works/12345678`
4. The AO3 server receives the request, looks up the work in its database, and generates an HTML response
5. The server sends the HTML response back to your browser
6. Your browser parses the HTML, fetches any CSS/JavaScript/images, and renders the page

This is the **request-response cycle**, and it's the foundation of the World Wide Web. Every interaction between a client (your browser, an e-reader, a mobile app) and a server (AO3, FF.net, FicHub) follows this pattern.

## HTTP: The Language of the Web

**HTTP** (HyperText Transfer Protocol) is the protocol that defines how clients and servers communicate. An HTTP request has a few key parts:

**Method:** What action the client wants to perform.
- `GET` — Retrieve data (most common)
- `POST` — Submit data
- `PUT` — Update data
- `DELETE` — Remove data

**URL:** The address of the resource the client wants. For example:
```
https://archiveofourown.org/works/12345678?view_full_work=true
```

Breaking this down:
- `https://` — Use HTTPS (encrypted connection)
- `archiveofourown.org` — The server to contact
- `/works/12345678` — The path on that server
- `?view_full_work=true` — Query parameters (optional)

**Headers:** Key-value pairs that provide additional information. For example:
```
User-Agent: fichub.net/0.1.0
Accept: text/html
```

**Body:** (Optional) Additional data, typically for POST requests.

An HTTP response has similar parts:

**Status Code:** Whether the request succeeded.
- `200 OK` — Success
- `301 Moved Permanently` — Redirect
- `404 Not Found` — Resource doesn't exist
- `500 Internal Server Error` — Server bug

**Headers:** Metadata about the response.
```
Content-Type: text/html; charset=utf-8
Content-Length: 12345
```

**Body:** The actual content (HTML, JSON, files, etc.)

## APIs: Talking to Servers Programmatically

An **API** (Application Programming Interface) is just a way for software to talk to other software. When FicHub's frontend sends a request to FicHub's backend, it's using an API. When FicHub scrapes AO3, it's using AO3's web interface as an informal API.

Modern web APIs typically use **JSON** (JavaScript Object Notation) as the data format. JSON is lightweight, human-readable, and easy to parse. Here's what a typical API response from FicHub looks like:

```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345678",
    "url_id": "a1b2c3d4e5f6",
    "slug": "My_Amazing_Fic-a1b2c3d4e5f6",
    "meta": {
        "id": "a1b2c3d4e5f6",
        "title": "My Amazing Fic",
        "author": "SomeAuthor",
        "chapters": 15,
        "words": 125000,
        "description": "A story about things...",
        "status": "complete",
        "source": "https://archiveofourown.org/works/12345678"
    },
    "epub_url": "/cache/epub/a1b2c3d4e5f6?h=abc123def456",
    "html_url": "/cache/html/a1b2c3d4e5f6?h=789xyz012abc"
}
```

The `err` field is a convention from the original FicHub — `0` means success, and negative numbers indicate specific error types. This is a bit unusual (most modern APIs use HTTP status codes instead), but it's maintained for backward compatibility.

## Databases: Where Data Lives

A web server needs to store and retrieve data. FicHub stores:
- Fanfiction metadata (title, author, word count, etc.)
- Request logs (who requested what, when)
- Export logs (which files have been generated, their hashes)
- Tags and tag votes
- Recommendations and community suggestions
- OPDS shelves

All of this data lives in a **database** — specifically, **PostgreSQL**, which is the most advanced open-source relational database in the world.

PostgreSQL uses **SQL** (Structured Query Language) to interact with data. Here are the basic operations:

**INSERT** — Add new data:
```sql
INSERT INTO fic_info (id, title, author, words, chapters, status, source)
VALUES ('a1b2c3d4e5f6', 'My Amazing Fic', 'SomeAuthor', 125000, 15, 'complete', 'https://...');
```

**SELECT** — Retrieve data:
```sql
SELECT title, author, words FROM fic_info WHERE id = 'a1b2c3d4e5f6';
```

**UPDATE** — Modify existing data:
```sql
UPDATE fic_info SET words = 126000, chapters = 16 WHERE id = 'a1b2c3d4e5f6';
```

**DELETE** — Remove data:
```sql
DELETE FROM fic_info WHERE id = 'a1b2c3d4e5f6';
```

FicHub also uses **UPSERT** (INSERT ... ON CONFLICT DO UPDATE), which inserts a row if it doesn't exist, or updates it if it does. This is very handy for caching scraped metadata:

```sql
INSERT INTO fic_info (id, title, author, words)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    author = EXCLUDED.author,
    words = EXCLUDED.words;
```

## Caching: Trading Space for Time

Every time FicHub generates an EPUB for a story, it does a lot of work:
1. Scrapes the story's metadata from the fanfiction site
2. Scrapes every chapter's content
3. Generates an EPUB file
4. Generates an HTML bundle
5. Optionally converts to MOBI/PDF

This takes several seconds (or even minutes for long stories). It would be wasteful to redo all this work every time someone requests the same story. So FicHub **caches** the results — it saves the generated files on disk and records their hashes in the database. The next time someone requests the same story, FicHub just serves the cached file.

The caching system has several layers:

**Database cache:** The `export_log` table records which files have been generated, along with their input and output hashes. Before generating a new EPUB, FicHub checks this table to see if it already exists.

**Disk cache:** The actual EPUB and HTML files are stored on disk in a hash-based directory structure:
```
cache/epub/a1b/def/a1b2c3d4e5f6/abc123def456.epub
cache/html/a1b/def/a1b2c3d4e5f6/789xyz012abc.zip
```

**Hash-based validation:** When serving a cached file, FicHub computes its MD5 hash and compares it to the expected hash. If they don't match (which could happen if the file was corrupted or tampered with), the file is rejected.

**Version bumping:** If the underlying fanfiction site changes its format, FicHub can bump the version number, which invalidates all cached files and forces regeneration.

This multi-layered caching strategy ensures that FicHub is fast for repeat requests while still producing correct output when the underlying data changes.

## Concurrency: Handling Multiple Requests

A web server needs to handle many requests simultaneously. If Alice requests an EPUB while Bob is requesting metadata for the same story, both requests should proceed without interfering with each other.

FicHub uses **async/await** concurrency, which allows a single thread to handle thousands of concurrent connections. When a handler needs to wait for something (a database query, an HTTP request, disk I/O), it yields control to the runtime, which can then process other requests. When the I/O completes, the handler resumes.

There's a subtlety here: what if Alice and Bob both request the EPUB for the same story at the same time? Without coordination, FicHub might generate the EPUB twice — wasting time and disk space. FicHub solves this with **semaphores**:

```rust
pub type CacheSemaphores = Arc<Mutex<HashMap<(String, EType), Arc<Semaphore>>>>;
```

Before generating an EPUB, a handler acquires a semaphore for that specific `(url_id, etype)` pair. If another handler is already generating the same EPUB, the second handler waits. After acquiring the semaphore, the handler does a **double-check** — it looks at the cache again, because the first handler might have already generated the file by now.

This is the classic "double-checked locking" pattern, and it's essential for avoiding duplicate work in a concurrent system.

## Summary

In this chapter, we've covered the fundamental concepts that underpin FicHub:

- **HTTP** — the protocol for client-server communication
- **APIs** — programmatic interfaces for software to talk to other software
- **JSON** — the data format used for API responses
- **Databases** — where persistent data lives (PostgreSQL for FicHub)
- **SQL** — the language for interacting with relational databases
- **Caching** — trading disk space for computation time
- **Concurrency** — handling multiple requests simultaneously

You don't need to be an expert in any of these topics to follow along — we'll explain everything as we go. But having this mental model will make the code much easier to understand.

Next up: we'll set up your development environment and write your first Rust program.

---

# Chapter 3: Setting Up Your Workshop

## Getting Ready to Code

Before we can write any Rust code, we need to set up our development environment. This chapter will walk you through installing everything you need — Rust, PostgreSQL, Redis, a code editor, and a few other tools.

Don't skip this step! Even if you think you already have some of these installed, it's worth verifying that you have the right versions. FicHub requires specific versions of several tools, and using the wrong version can cause confusing errors.

## Installing Rust

Rust is installed through `rustup`, which is Rust's official toolchain installer. It manages the Rust compiler, the standard library, and Cargo (Rust's build tool and package manager).

On Linux or macOS, open a terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This will download and run the installer. Accept the defaults (just press Enter) when prompted. After installation, reload your shell:

```bash
source $HOME/.cargo/env
```

Verify the installation:

```bash
rustc --version
# rustc 1.78.0 (9b00956e5 2024-04-29)
cargo --version
# cargo 1.78.0 (54d8815d0 2024-03-26)
```

On Windows, download and run `rustup-init.exe` from https://rustup.rs.

**Watch Out!** If you've previously installed Rust through your system package manager (apt, brew, etc.), you might have a conflicting installation. Use `rustup` instead — it's the officially supported method and gives you much better control over toolchain versions.

## Installing PostgreSQL

FicHub uses PostgreSQL as its primary database. You need version 14 or later for full-text search support.

On Arch Linux / Manjaro:
```bash
sudo pacman -S postgresql
sudo systemctl enable --now postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
```

On Ubuntu / Debian:
```bash
sudo apt install postgresql postgresql-contrib
sudo systemctl enable --now postgresql
```

On macOS (with Homebrew):
```bash
brew install postgresql@16
brew services start postgresql@16
```

After installation, create a database for FicHub:

```bash
sudo -u postgres createdb fichub
```

And create a user with a password:

```bash
sudo -u postgres psql -c "CREATE USER fichub WITH PASSWORD 'fichub' CREATEDB;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE fichub TO fichub;"
```

## Installing Redis

Redis is an in-memory data store that FicHub uses for rate limiting and queuing. It's much faster than PostgreSQL for operations that need to be fast (like checking if a request is allowed).

On Arch Linux / Manjaro:
```bash
sudo pacman -S redis
sudo systemctl enable --now redis
```

On Ubuntu / Debian:
```bash
sudo apt install redis-server
sudo systemctl enable --now redis-server
```

On macOS:
```bash
brew install redis
brew services start redis
```

Verify Redis is running:

```bash
redis-cli ping
# PONG
```

## Setting Up Your Editor

We recommend **Visual Studio Code** with the `rust-analyzer` extension. This gives you:
- Real-time error checking as you type
- Auto-completion for Rust types and functions
- Inline type hints
- Code navigation (go to definition, find all references)
- Integrated debugging

Install VS Code from https://code.visualstudio.com, then install the `rust-analyzer` extension from the Extensions panel.

**Alternative editors:** If you prefer Vim/Neovim, the `rust-analyzer` LSP works great. Same for Emacs with `eglot` or `lsp-mode`, Helix, or Zed. Use whatever makes you productive.

## Setting Up the Environment

FicHub reads its configuration from environment variables. You can set these in your shell, but the easiest way is to use a `.env` file in the project root.

Create a `.env` file in the FicHub directory:

```bash
# Database connection
DATABASE_URL=postgres://fichub:fichub@localhost/fichub

# Redis connection
REDIS_URL=redis://localhost/0

# Where to store cached EPUB/HTML files
CACHE_DIR=./cache

# Where to store temporary files during EPUB generation
TMP_DIR=./tmp

# Port to listen on
PORT=3000

# Where the SvelteKit frontend build output lives
FRONTEND_DIR=./frontend/build

# Logging level
RUST_LOG=info,fichub=debug
```

The `dotenvy` crate (which FicHub uses) will automatically load this file when the server starts, so you don't need to export these variables manually.

**Try It Yourself:** Create the `.env` file now and verify that your database and Redis connections work:

```bash
# Test PostgreSQL connection
psql "postgres://fichub:fichub@localhost/fichub" -c "SELECT 1"

# Test Redis connection
redis-cli ping
```

If both commands succeed, you're ready to go.

## Cloning the Repository

If you haven't already, clone the FicHub repository:

```bash
git clone https://github.com/user/fichub-rs.git
cd fichub-rs
```

Take a look at the project structure:

```
fichub-rs/
├── Cargo.toml          # Project manifest (dependencies, metadata)
├── .env                # Environment variables (not committed to git)
├── migrations/         # SQL migration files
│   ├── 001_initial_schema.sql
│   ├── 002_recommender.sql
│   ├── 003_tagging.sql
│   └── 004_shelves.sql
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library re-exports
│   ├── server.rs       # HTTP server setup
│   ├── config.rs       # Configuration from env vars
│   ├── error.rs        # Error types
│   ├── db/             # Database layer
│   ├── scrape/         # Web scraping
│   ├── export/         # EPUB/HTML generation
│   ├── cache/          # Disk cache management
│   ├── limiter/        # Rate limiting
│   ├── recommender/    # Recommendation engine
│   ├── search/         # Full-text search
│   ├── tags/           # Tag system
│   ├── routes/         # API route handlers
│   └── frontend/       # Frontend module
└── frontend/           # SvelteKit frontend (separate)
```

## Building for the First Time

Let's build FicHub and see if everything compiles:

```bash
cargo build 2>&1
```

The first build will take several minutes because Rust needs to download and compile all dependencies. Subsequent builds will be much faster thanks to incremental compilation.

If the build succeeds, you'll see something like:

```
   Compiling fichub v0.1.0 (/path/to/fichub-rs)
    Finished dev [unoptimized + debuginfo] target(suitable) in 45.23s
```

**Watch Out!** If you see an error about `sqlx` needing a database connection, you need to either:
1. Set the `DATABASE_URL` environment variable (which you've already done in `.env`)
2. Or run `cargo sqlx prepare` to create a query cache for offline builds

The SQLx offline mode is useful for CI/CD and Docker builds where the database isn't available.

## Running the Tests

FicHub has a comprehensive test suite. Let's run it:

```bash
cargo test 2>&1
```

This will run all unit tests — the tests embedded in the source files using `#[cfg(test)]` modules. You should see output like:

```
running 42 tests
test config::tests::test_from_env_defaults ... ok
test config::tests::test_panics_without_database_url ... ok
test scrape::tests::test_generate_url_id_deterministic ... ok
test cache::disk::tests::test_cache_path_medium_url_id ... ok
...
test result: ok. 42 passed; 0 failed; 0 ignored
```

These tests verify that individual functions work correctly in isolation. We'll cover testing in detail in Chapter 34.

## Your First Rust Program

Now that your environment is set up, let's write a tiny Rust program to make sure everything works. Create a new file called `hello.rs`:

```rust
fn main() {
    println!("Hello, FicHub!");
}
```

Run it:

```bash
rustc hello.rs
./hello
# Hello, FicHub!
```

That's Rust in its simplest form — a `main` function that prints a message. Let's make it slightly more interesting:

```rust
fn main() {
    let name = "FicHub";
    let version = 1;
    println!("{name} v{version} is ready to go!");
    
    let words: Vec<i32> = (1..=10).collect();
    let sum: i32 = words.iter().sum();
    println!("Sum of 1..10: {sum}");
}
```

```bash
rustc hello.rs && ./hello
# FicHub v1 is ready to go!
# Sum of 1..10: 55
```

Notice a few things about Rust syntax:
- `let` declares a variable (immutable by default!)
- `fn` defines a function
- `println!` with the `!` is a macro (more powerful than a function)
- `{name}` inside a string is string interpolation
- `Vec<i32>` specifies a vector of 32-bit integers
- `.iter().sum()` is method chaining — calling `iter()` on a vector, then `sum()` on the iterator

We'll explore all of this in detail in the next chapter. For now, just know that Rust compiles your code to a fast, self-contained binary with no runtime dependencies. That `hello` binary you just created? It'll run on any Linux system without needing to install anything.

## Summary

In this chapter, you've:
- Installed Rust (via rustup)
- Installed PostgreSQL and created a database
- Installed Redis
- Set up your editor (VS Code with rust-analyzer)
- Created a `.env` file with your database and Redis credentials
- Built FicHub for the first time
- Run the test suite
- Written your first Rust program

Your workshop is ready. Next, we'll dive into the Rust language itself — variables, functions, structs, enums, async, and Cargo.

---

# Chapter 4: Hello Rust

## Rust in a Nutshell

Welcome to your Rust crash course! We're not going to cover every feature of the language — that would take an entire book by itself. Instead, we'll focus on exactly the Rust features that FicHub uses, so you can understand the code as we walk through it.

If you've used languages like Python, JavaScript, or Go, Rust will feel familiar in some ways and completely alien in others. The alien parts — ownership, borrowing, lifetimes — are what make Rust special. Let's start with the basics and work our way up.

## Variables and Mutability

In Rust, variables are **immutable by default**. This is the opposite of most languages, where variables can be changed freely.

```rust
let x = 5;
x = 6;  // ERROR: cannot assign twice to immutable variable
```

To make a variable mutable, use `mut`:

```rust
let mut x = 5;
x = 6;  // This is fine
```

Why immutable by default? Because immutability makes your code safer and easier to reason about. When you see `let x = 5`, you know that `x` will always be `5` — no function or other code can change it. This eliminates an entire class of bugs.

In FicHub, you'll see both immutable and mutable variables:

```rust
// Immutable — this never changes
let query = params.q.as_deref().unwrap_or("");

// Mutable — we add to this as we build the response
let mut urls = std::collections::HashMap::new();
urls.insert("epub".to_string(), "/cache/epub/abc123".to_string());
```

**Shadowing** is also allowed — you can re-declare a variable with the same name:

```rust
let x = 5;
let x = x + 1;  // This creates a NEW variable, shadowing the old one
let x = x * 2;  // And again
// x is now 12
```

Shadowing is different from mutation — it creates a new variable that happens to have the same name. This is useful when you want to transform a value:

```rust
let name = "  FicHub  ";
let name = name.trim();  // Shadow with trimmed version
```

## Functions

Functions in Rust are defined with `fn`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = this is the return value
}
```

A few things to notice:
- Parameters **must** have type annotations
- The return type comes after `->`
- The last expression (without a semicolon) is the return value
- You can also use `return` explicitly, but it's idiomatic to omit it for the last expression

In FicHub, functions look like this:

```rust
/// Generate a URL-safe slug from title and url_id
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

Notice the `pub` keyword — this makes the function publicly accessible. Without `pub`, a function is private to its module.

**Closures** are like anonymous functions:

```rust
let add_one = |x: i32| -> i32 { x + 1 };
let result = add_one(5);  // 6

// If the body is a single expression, you can omit the braces:
let add_one = |x| x + 1;
```

Closures are used extensively in FicHub for iterators and callbacks:

```rust
// Map over chapters, creating HTML for each one
for chapter in chapters {
    chapters_nav.push_str(&format!(
        r##"<li><a href="#ch{ch}">{title}</a></li>"##,
        ch = chapter.chapter_id,
        title = escape_html(&chapter.title),
    ));
}
```

## Structs

**Structs** are Rust's way of creating custom types with named fields:

```rust
struct FicMetadata {
    url_id: String,
    title: String,
    author: String,
    chapters: i32,
    words: i64,
    desc: String,
    status: String,
    source: String,
}
```

You create an instance of a struct like this:

```rust
let meta = FicMetadata {
    url_id: "a1b2c3d4e5f6".to_string(),
    title: "My Amazing Fic".to_string(),
    author: "SomeAuthor".to_string(),
    chapters: 15,
    words: 125000,
    desc: "A story about things...".to_string(),
    status: "complete".to_string(),
    source: "https://archiveofourown.org/works/12345678".to_string(),
};
```

Access fields with a dot:

```rust
println!("{} by {}", meta.title, meta.author);
```

FicHub's `FicMetadata` struct is defined in `src/scrape/mod.rs` and is used throughout the codebase. It's the universal representation of a scraped fanfiction's metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub published: i64,         // unix millis
    pub updated: i64,           // unix millis
    pub status: String,         // ongoing, complete, hiatus, cancelled
    pub source: String,         // original URL
    pub source_id: i64,
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

The `#[derive(...)]` attribute automatically implements common traits:
- `Debug` — lets you print the struct with `{:?}`
- `Clone` — lets you clone the struct with `.clone()`
- `Serialize` / `Deserialize` — lets you convert to/from JSON (via serde)

**Tuple structs** are unnamed structs with positional fields:

```rust
struct Color(u8, u8, u8);
let red = Color(255, 0, 0);
```

**Unit structs** have no fields:

```rust
struct Marker;
```

These are used when you need a type to implement a trait but don't need any data.

## Enums

**Enums** are one of Rust's most powerful features. An enum represents a value that can be one of several variants:

```rust
enum EType {
    Epub,
    Html,
    Mobi,
    Pdf,
}
```

Unlike C-style enums, Rust enums can carry data with each variant:

```rust
enum AppError {
    BadRequest(i32, String),
    RateLimited(u64),
    NotFound(String),
    Internal(String),
    ScrapeError(String),
    ExportError(String),
    Database(String),
    CacheError(String),
}
```

Each variant can have different fields! `BadRequest` has a numeric code and a message. `RateLimited` has a retry-after duration. `NotFound` has just a message. This is much more expressive than a generic "error code + message" approach.

You create enum values like this:

```rust
let err = AppError::BadRequest(400, "invalid input".into());
let err = AppError::RateLimited(30);
```

**Pattern matching** is how you handle different variants:

```rust
match err {
    AppError::BadRequest(code, msg) => {
        println!("Bad request {}: {}", code, msg);
    }
    AppError::RateLimited(retry_after) => {
        println!("Rate limited, retry after {}s", retry_after);
    }
    AppError::NotFound(msg) => {
        println!("Not found: {}", msg);
    }
    _ => {
        println!("Some other error");
    }
}
```

FicHub's `AppError` enum uses `match` to convert each error variant into an appropriate HTTP response:

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
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"err": -1, "msg": "internal server error"}))
            }
            // ... more variants
        };
        (status, Json(body)).into_response()
    }
}
```

This is elegant — the error type itself knows how to become an HTTP response. The handler just returns `Err(AppError::NotFound("story".into()))`, and Axum handles the rest.

## Option and Result

Rust doesn't have `null` or `nil`. Instead, it has **Option** and **Result**:

**Option** represents a value that might or might not exist:

```rust
fn find_scraper(url: &str) -> Option<&Box<dyn SiteScraper>> {
    if url.contains("archiveofourown.org") {
        Some(&ao3_scraper)
    } else {
        None
    }
}
```

You handle Option with `match`, `if let`, or methods like `unwrap`, `unwrap_or`, `map`:

```rust
// Match
match find_scraper(url) {
    Some(scraper) => println!("Found scraper!"),
    None => println!("No scraper found"),
}

// If let (shorthand for match with one pattern)
if let Some(scraper) = find_scraper(url) {
    println!("Found scraper!");
}

// Unwrap (panics if None — use sparingly!)
let scraper = find_scraper(url).unwrap();

// Unwrap or default
let scraper = find_scraper(url).unwrap_or(&default_scraper);

// Map
let name = find_scraper(url).map(|s| s.site_name());
```

**Result** represents an operation that might fail:

```rust
fn parse_story_id(url: &str) -> Result<i64, String> {
    url.split('/').last()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("could not parse story ID from {}", url))
}
```

Results are handled similarly to Options:

```rust
match parse_story_id(url) {
    Ok(id) => println!("Story ID: {}", id),
    Err(e) => println!("Error: {}", e),
}

// Or with the ? operator (returns early on error):
let id = parse_story_id(url)?;
```

The `?` operator is Rust's error propagation shorthand. If the expression is `Err`, the function returns that error immediately. If it's `Ok`, the value is extracted. This makes error handling concise:

```rust
async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;  // ? propagates sqlx::Error → AppError::Database
    Ok(row)
}
```

## Traits

**Traits** are Rust's version of interfaces. They define a set of methods that a type must implement:

```rust
trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

Each scraper (AO3, FF.net, XenForo, etc.) implements this trait:

```rust
struct Ao3Scraper;

impl SiteScraper for Ao3Scraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("archiveofourown.org")
    }
    // ...
}
```

Traits enable **polymorphism** — you can write code that works with any scraper:

```rust
fn find_scraper(url: &str) -> Option<&dyn SiteScraper> {
    scrapers.iter().find(|s| s.can_handle(url))
}
```

The `dyn SiteScraper` is a **trait object** — a reference to any type that implements `SiteScraper`. This is Rust's version of dynamic dispatch (like virtual methods in Java or C++).

## Async/Await

FicHub is an async application. Every handler, database query, and HTTP request is asynchronous. Here's why:

When FicHub handles a request, it needs to:
1. Look up metadata from the database (I/O — takes milliseconds)
2. Scrape the fanfiction site (network I/O — takes seconds)
3. Generate an EPUB (CPU + disk I/O — takes seconds)

Without async, FicHub would be blocked during each of these operations, unable to handle other requests. With async, the handler can yield control while waiting, allowing the server to process other requests.

Here's how async looks in Rust:

```rust
async fn get_metadata(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    Ok(html)
}
```

The `async` keyword marks a function as asynchronous. Inside, `.await` pauses execution until the future completes, allowing other tasks to run.

You spawn async tasks with `tokio::spawn`:

```rust
tokio::spawn(async {
    // This runs concurrently with other tasks
    process_fic(url).await;
});
```

And you can run multiple async operations concurrently with `tokio::join!` or `tokio::select!`:

```rust
// Run both concurrently, wait for both to finish
let (epub_result, html_result) = tokio::join!(
    create_epub(&meta, &chapters, &tmp_dir),
    create_html_bundle(&meta, &chapters, &tmp_dir),
);
```

## Ownership and Borrowing

This is the part of Rust that confuses newcomers, but it's also what makes Rust special. In Rust, every value has exactly one **owner** — the variable that's responsible for cleaning it up when it goes out of scope.

```rust
let name = String::from("FicHub");
let greeting = name;  // Ownership transferred to 'greeting'
// println!("{}", name);  // ERROR: 'name' no longer owns the value
println!("{}", greeting);  // Fine
```

When you want to use a value without taking ownership, you **borrow** it:

```rust
fn print_name(name: &String) {  // Borrows 'name'
    println!("{}", name);
}

let name = String::from("FicHub");
print_name(&name);  // Pass a reference
println!("{}", name);  // 'name' still owns the value
```

There are two kinds of borrows:
- `&T` — shared borrow (many readers, no writers)
- `&mut T` — mutable borrow (one writer, no readers)

This is enforced at compile time. You can have many shared references or one mutable reference, but never both. This prevents data races without a garbage collector.

In FicHub, you'll see borrows everywhere:

```rust
// Shared borrows — multiple functions can read 'meta'
fn build_info_string(meta: &FicMetadata) -> (String, Vec<String>) { ... }
fn build_meta_json(meta: &FicMetadata) -> Value { ... }

// Mutable borrows — only one function can modify 'urls'
let mut urls = HashMap::new();
urls.insert("epub".to_string(), epub_url);
```

## Cargo: The Build System

Cargo is Rust's build system and package manager. You've already used it with `cargo build` and `cargo test`. Here are the most important commands:

```bash
cargo build          # Build in debug mode (fast compile, slow runtime)
cargo build --release  # Build in release mode (slow compile, fast runtime)
cargo test           # Run all tests
cargo run            # Build and run
cargo check          # Check for errors without producing a binary
cargo fmt            # Format code according to Rust style
cargo clippy         # Run the linter for common mistakes
cargo doc --open     # Generate and open documentation
```

Dependencies are declared in `Cargo.toml`:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.9", features = ["postgres", "migrate"] }
```

When you run `cargo build`, Cargo downloads, compiles, and links all dependencies. The first time is slow (many crates to compile), but subsequent builds only recompile what changed.

**Dev-dependencies** are only used for tests:

```toml
[dev-dependencies]
axum-test = "16"
```

## Putting It All Together

Let's look at a small but complete example that combines everything we've learned:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct FicInfo {
    title: String,
    author: String,
    words: i64,
    status: String,
}

impl FicInfo {
    fn new(title: &str, author: &str, words: i64, status: &str) -> Self {
        FicInfo {
            title: title.to_string(),
            author: author.to_string(),
            words,
            status: status.to_string(),
        }
    }

    fn is_complete(&self) -> bool {
        self.status == "complete"
    }
}

fn format_fic(info: &FicInfo) -> String {
    format!("{} by {} — {} words [{}]", info.title, info.author, info.words, info.status)
}

fn main() {
    let fics: Vec<FicInfo> = vec![
        FicInfo::new("My Story", "Author A", 50_000, "complete"),
        FicInfo::new("Another Tale", "Author B", 120_000, "ongoing"),
        FicInfo::new("Third Fic", "Author A", 30_000, "complete"),
    ];

    let complete: Vec<&FicInfo> = fics.iter().filter(|f| f.is_complete()).collect();
    
    for fic in &complete {
        println!("{}", format_fic(fic));
    }

    let total_words: i64 = fics.iter().map(|f| f.words).sum();
    println!("Total words: {}", total_words);
}
```

This program:
1. Defines a `FicInfo` struct with an `impl` block for methods
2. Creates a vector of fics
3. Filters for complete ones
4. Prints each one
5. Sums up the word counts

Every concept — structs, methods, vectors, closures, iterators, pattern matching, formatting — is something we'll see in the FicHub codebase.

## Summary

In this chapter, we've covered the Rust features that FicHub uses most:
- **Variables:** immutable by default, mutable with `mut`, shadowing
- **Functions:** typed parameters, return types, closures
- **Structs:** named fields, `impl` blocks, derive macros
- **Enums:** variants with data, pattern matching
- **Option and Result:** null safety and error handling without exceptions
- **Traits:** interfaces and polymorphism
- **Async/await:** non-blocking I/O with Tokio
- **Ownership and borrowing:** memory safety without a garbage collector
- **Cargo:** build system, package manager, testing

You don't need to memorize all of this — we'll see these concepts in action throughout the book. But having this reference will help when you encounter something unfamiliar in the code.

In the next chapter, we'll build our first Axum server and start writing real HTTP handlers.


---

# Part 2: Building the Axum Backend

---

# Chapter 5: First Axum Server

## Hello, Axum!

Now that you understand the basics of Rust, it's time to build something real. In this chapter, we'll create our first Axum web server — a minimal HTTP server that responds to requests. By the end of this chapter, you'll understand how Axum works and how FicHub uses it.

Axum is a web framework built on top of Tokio (the async runtime) and Tower (a middleware framework). It's designed to be ergonomic, type-safe, and composable. Unlike some web frameworks that rely heavily on macros or runtime reflection, Axum uses Rust's type system to ensure correctness at compile time.

Let's create a new project:

```bash
cargo new fichub-demo
cd fichub-demo
cargo add axum tokio --features tokio/full
```

Now edit `src/main.rs`:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, FicHub!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
```

Run it:

```bash
cargo run
```

In another terminal:

```bash
curl http://localhost:3000/
# Hello, FicHub!
```

That's a working web server in 12 lines of code. Let's break down what's happening:

1. `Router::new()` creates a new router — the central routing table that maps URLs to handlers
2. `.route("/", get(...))` adds a route: when a GET request comes in for `/`, call the given handler
3. `TcpListener::bind("0.0.0.0:3000")` binds to all network interfaces on port 3000
4. `axum::serve(listener, app)` starts the server, handling requests as they arrive

## Handlers

A **handler** is an async function that takes some **extractors** and returns a **response**:

```rust
async fn hello_handler() -> &'static str {
    "Hello, FicHub!"
}
```

The return type determines the response. A `&'static str` becomes a plain text response. An `axum::Json<Value>` becomes a JSON response. Let's make a JSON endpoint:

```rust
use axum::Json;
use serde_json::json;

async fn api_info() -> Json<serde_json::Value> {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": "Fetch metadata and download links",
            "/api/v0/meta": "Fetch metadata only",
        }
    }))
}
```

Add this route to your router:

```rust
let app = Router::new()
    .route("/", get(|| async { "Hello, FicHub!" }))
    .route("/api/", get(api_info));
```

Now `curl http://localhost:3000/api/` returns a JSON response with all the endpoint information.

This is exactly what FicHub does in `src/routes/api_docs.rs`:

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links"
            },
            // ...
        }
    }))
}
```

## Extractors

**Extractors** are how Axum pulls data out of the incoming request. They're the parameters to your handler functions. Axum figures out what to extract based on the type:

```rust
use axum::extract::{Path, Query, State};
use std::sync::Arc;

async fn handler(
    State(state): State<Arc<AppState>>,     // Shared application state
    Path(url_id): Path<String>,              // URL path parameter
    Query(params): Query<MyParams>,          // Query string parameters
) -> Json<serde_json::Value> {
    Json(json!({ "url_id": url_id }))
}
```

Let's look at each extractor type:

### Path Extractor

The `Path` extractor captures parts of the URL path:

```rust
.route("/cache/{etype}/{url_id}", get(download_handler))

async fn download_handler(
    Path((etype, url_id)): Path<(String, String)>,
) -> String {
    format!("Download {} in format {}", url_id, etype)
}
```

If the URL is `/cache/epub/abc123`, then `etype` is `"epub"` and `url_id` is `"abc123"`.

### Query Extractor

The `Query` extractor captures query string parameters:

```rust
.route("/api/v0/epub", get(epub_handler))

#[derive(Deserialize)]
struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

async fn epub_handler(
    Query(params): Query<ExportQuery>,
) -> Json<serde_json::Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({ "q": query }))
}
```

If the URL is `/api/v0/epub?q=https://ao3.org/works/123`, then `params.q` is `Some("https://ao3.org/works/123")`.

### State Extractor

The `State` extractor provides shared application state — the database pool, Redis connection, scraper registry, and everything else that needs to be shared across handlers:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    // ...
}

async fn handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    // Access the database
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db)
        .await
        .unwrap();
    
    Json(json!({ "fic_count": count.0 }))
}
```

We wrap `AppState` in `Arc` (Atomic Reference Counting) because it's shared across all handlers. `Arc` lets multiple tasks hold a reference to the same data without taking ownership.

### ConnectInfo Extractor

The `ConnectInfo` extractor gives you the client's IP address and port:

```rust
use axum::extract::ConnectInfo;
use std::net::SocketAddr;

async fn handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> String {
    format!("Your IP is {}", remote.ip())
}
```

This requires the server to be started with `into_make_service_with_connect_info`:

```rust
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
).await.unwrap();
```

FicHub uses this to track client IPs for rate limiting and tag voting.

## Building the Router

The router is the central mapping from URLs to handlers. You build it by chaining `.route()` calls:

```rust
let app = Router::new()
    .route("/", get(root_handler))
    .route("/api/", get(api_docs_handler))
    .route("/api/v0/epub", get(epub_handler))
    .route("/api/v0/meta", get(meta_handler))
    .route("/api/v0/search", get(search_handler))
    .route("/cache/{etype}/{url_id}", get(download_handler));
```

You can also use `nest` to group routes under a common prefix:

```rust
let app = Router::new()
    .nest("/api/v0", api_routes)
    .nest("/opds", opds_routes);
```

In FicHub, the router is built in `server.rs` and includes about 40 routes. The full route tree looks like this:

```rust
Router::new()
    // API routes
    .route("/api/", get(routes::api_docs::api_docs_handler))
    .route("/api/v0/epub", get(routes::export::epub_handler))
    .route("/api/v0/meta", get(routes::meta::meta_handler))
    .route("/api/v0/remote", get(remote_handler))

    // Cache download routes
    .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
    .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))

    // Recommender routes
    .route("/api/v0/recommendations", get(recommendations_handler))
    .route("/api/v0/recommendations/suggest", post(suggest_handler))
    .route("/api/v0/recommendations/vote", post(vote_handler))
    .route("/api/v0/recommendations/votes", get(votes_handler))

    // Tag routes
    .route("/api/v0/tags/submit", post(submit_tag))
    .route("/api/v0/tags/vote", post(vote_tag))
    .route("/api/v0/tags/flag", post(flag_tag))
    .route("/api/v0/tags", get(get_tags))

    // Curator routes
    .route("/api/v0/curator/alias", post(create_alias))
    .route("/api/v0/curator/merge", post(merge_tags))
    .route("/api/v0/curator/tags/{id}", delete(delete_tag))
    .route("/api/v0/curator/flags", get(list_flags))
    .route("/api/v0/curator/flags/{id}/resolve", post(resolve_flag))

    // Search
    .route("/api/v0/search", get(search_handler))

    // OPDS catalog routes
    .route("/opds", get(root_catalog))
    .route("/opds/new", get(recent_feed))
    .route("/opds/popular", get(popular_feed))
    .route("/opds/tags", get(tag_types))
    .route("/opds/tags/{type_id}", get(tags_by_type))
    .route("/opds/tags/{type_id}/{tag_name}", get(fics_by_tag))
    .route("/opds/authors", get(author_list))
    .route("/opds/recommendations/popular", get(popular_recommendations))
    .route("/opds/recommendations", get(fic_recommendations))
    .route("/opds/search", get(search_feed))
    .route("/opds/shelves", get(shelf_list))
    .route("/opds/shelf/{shelf_id}", get(shelf_contents))

    // Legacy redirects
    .route("/legacy/epub_export", get(redirect_to_root))
    .route("/fic/{url_id}", get(redirect_to_root))

    // Static frontend (SPA fallback)
    .fallback_service(
        ServeDir::new(&frontend_dir)
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(frontend_dir.join("index.html"))),
    )

    // Middleware
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())

    // Shared state
    .with_state(state)
```

Notice the three different HTTP methods: `get()` for read-only endpoints, `post()` for creating/updating data, and `delete()` for removing data. The tags and curator routes use POST because they modify state. The search and OPDS routes use GET because they only read.

## Method Routing

Axum provides routing functions for each HTTP method:

```rust
use axum::routing::{get, post, put, delete};

Router::new()
    .route("/items", get(list_items).post(create_item))
    .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
```

You can chain multiple methods on the same route. In FicHub, most routes are GET (read-only), with POST for creating suggestions, voting, and submitting tags. The only DELETE route is for curator tag deletion.

## Middleware

**Middleware** is code that runs before or after your handler. FicHub uses two middleware layers:

**TraceLayer** logs every request:

```rust
.layer(TraceLayer::new_for_http())
```

This produces log lines like:

```
2024-01-15T10:30:00Z INFO request{method=GET uri=/api/v0/epub}: fichub::routes: 200 OK
```

**CorsLayer** handles Cross-Origin Resource Sharing:

```rust
.layer(CorsLayer::permissive())
```

This allows any origin to make requests to the API. In production, you'd restrict this to your frontend's domain, but `permissive()` is fine for development.

## Returning Different Response Types

Handlers can return different types depending on the situation:

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    // Do some work...
    let result = do_work(query).await?;
    
    Ok(Json(json!({"err": 0, "data": result})))
}
```

Notice the `Result<Json<Value>, AppError>` return type. If the handler returns `Ok(json)`, the client gets a JSON response. If it returns `Err(error)`, Axum calls `error.into_response()` to produce the appropriate HTTP error response.

This is FicHub's pattern — every handler returns `Result<Json<Value>, AppError>`, and the `AppError` type knows how to become an HTTP response.

## Try It Yourself

Let's build a complete mini-server that mimics a few FicHub endpoints:

```rust
use axum::{extract::{Query, State}, routing::get, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

struct AppState {
    fic_count: i64,
}

#[derive(Deserialize)]
struct MetaQuery {
    q: Option<String>,
}

async fn api_docs() -> Json<Value> {
    Json(json!({
        "name": "FicHub Demo",
        "endpoints": ["/api/v0/meta", "/api/v0/epub"]
    }))
}

async fn meta_handler(
    Query(params): Query<MetaQuery>,
) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({
        "err": 0,
        "q": query,
        "meta": { "title": "Demo Fic", "author": "Demo Author" }
    }))
}

async fn epub_handler(
    Query(params): Query<MetaQuery>,
) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    Json(json!({
        "err": 0,
        "q": query,
        "epub_url": format!("/cache/epub/abc123"),
        "html_url": format!("/cache/html/abc123")
    }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState { fic_count: 42 });

    let app = Router::new()
        .route("/api/", get(api_docs))
        .route("/api/v0/meta", get(meta_handler))
        .route("/api/v0/epub", get(epub_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
```

Try these commands:

```bash
curl http://localhost:3000/api/
curl "http://localhost:3000/api/v0/meta?q=https://ao3.org/works/123"
curl "http://localhost:3000/api/v0/epub?q=https://ao3.org/works/123"
```

This is essentially a simplified version of what FicHub does. The real thing has more error handling, database queries, and scraping — but the structure is the same.

## Watch Out!

**Don't forget `with_state`!** If you use `State` in your handlers but forget to call `.with_state(state)` on the router, you'll get a confusing compile error.

**Async handlers must be `Send`!** Axum requires all handlers to be `Send`, which means they can't hold non-Send types across `.await` points. This is rarely an issue, but if you see errors about `Send` bounds, that's why.

**JSON responses require `serde_json::Value`!** You can't return raw strings as JSON — you need to wrap them in `Json(json!({...}))`. The `json!` macro creates a `serde_json::Value` from a JSON-like syntax.

## Summary

In this chapter, you learned:
- How to create a minimal Axum server
- What handlers are and how they work
- How extractors pull data from requests (Path, Query, State, ConnectInfo)
- How to build a router with multiple routes
- How middleware adds logging and CORS
- How handlers return JSON responses or errors
- How FicHub's route tree is organized

Next, we'll look at how FicHub handles configuration and errors in detail.

---

# Chapter 6: Config and Errors

## Configuration: One Place, Many Settings

Every web application needs configuration — database URLs, port numbers, feature flags, rate limits. The question is: where should this configuration come from?

FicHub follows the **twelve-factor app** methodology: configuration comes from environment variables. This is the standard for containerized and cloud-native applications. It means you can run the same binary in development, staging, and production — just change the environment variables.

The `Config` struct in `src/config.rs` holds all of FicHub's configuration:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // Infrastructure
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub app_port: u16,
    pub tmp_dir: PathBuf,
    pub frontend_dir: PathBuf,

    // Feature flags
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub node_name: String,

    // Recommender settings
    pub rec_default_delay_secs: u64,
    pub rec_site_rate_limits: HashMap<String, u64>,
    pub rec_max_favourite_pages: u32,
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

    // OPDS
    pub opds_shelf_token: String,

    // Other
    pub calibre_container: String,
    pub trusted_proxies: Vec<String>,
    pub ip_tag_sources: Vec<(String, String, String)>,
}
```

That's a lot of settings! Each one serves a specific purpose. Let's look at the most important groups:

### Infrastructure Settings

These are required — the server can't start without them:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let redis_url = std::env::var("REDIS_URL")
    .expect("REDIS_URL must be set");
```

The `.expect()` call will panic (crash the server) if the variable isn't set. This is intentional — if you forget to set the database URL, you want to know immediately, not discover it 5 minutes later when a request fails.

### Optional Settings with Defaults

Most settings have sensible defaults:

```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());

let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

The `.unwrap_or_else()` pattern is more user-friendly — if the variable isn't set, use the default. The `.parse()` converts the string to the appropriate type (u16 for port).

### The from_env Method

The entire configuration is loaded in one place:

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        // ... all other settings ...
        
        Config {
            database_url,
            redis_url,
            // ...
        }
    }
}
```

This is called once at startup, in `main.rs`:

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    let config = config::Config::from_env();
    
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    server::run(config).await;
}
```

The `.dotenv().ok()` call loads a `.env` file if it exists (and silently does nothing if it doesn't). The tracing subscriber is configured from the `RUST_LOG` environment variable.

### The Clone Derive

Notice that `Config` derives `Clone`:

```rust
#[derive(Debug, Clone)]
pub struct Config { ... }
```

This lets us clone the config and pass it to different parts of the application. Since `Config` contains only simple types (strings, numbers, booleans, PathBufs), cloning is cheap.

## Error Handling: The FicHub Way

Error handling is one of the most important aspects of a web application. FicHub has a clean, centralized error type that handles all error scenarios:

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
```

Each variant represents a different class of error:

- **BadRequest** — The client sent invalid input (bad URL, missing parameters)
- **RateLimited** — Too many requests from this IP
- **NotFound** — The requested resource doesn't exist
- **Internal** — Something went wrong on the server (but not related to any specific subsystem)
- **ScrapeError** — The scraper failed (site is down, blocked, or HTML changed)
- **ExportError** — EPUB/HTML generation failed
- **Database** — A database query failed
- **CacheError** — A Redis operation failed

### Display Implementation

Every error needs a human-readable representation:

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

This is used for logging — when an error occurs, it's logged with its Display representation.

### IntoResponse Implementation

The key feature of `AppError` is that it implements `IntoResponse` — it knows how to become an HTTP response:

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
                (StatusCode::BAD_GATEWAY, json!({
                    "err": -6,
                    "msg": msg
                }))
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

A few design choices here:

1. **BadRequest** returns the error code to the client, so the frontend can handle specific error types
2. **RateLimited** includes `retry_after` so the client knows when to retry
3. **ScrapeError** returns `502 Bad Gateway` because the error originated from an upstream site
4. **Internal, Database, CacheError, ExportError** all return `500 Internal Server Error` and log the error — the client gets a generic message, but the server logs the details
5. **NotFound** returns `404` with a descriptive message

### From Implementations

The `?` operator works because `AppError` implements `From` for various error types:

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

This means you can use `?` on any of these error types and they'll automatically be converted to the appropriate `AppError` variant:

```rust
// sqlx::Error → AppError::Database
let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

// redis::RedisError → AppError::CacheError
let count: Option<u32> = redis.get(key).await?;

// reqwest::Error → AppError::ScrapeError
let response = client.get(url).send().await?;
```

This is incredibly ergonomic — you never have to manually convert errors. The `?` operator does it for you.

### The AppResult Type

FicHub defines a convenience alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

This means handlers can return `AppResult<Json<Value>>` instead of the longer `Result<Json<Value>, AppError>`. Most query functions also return `AppResult<T>`:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
```

### Custom Error Codes

The `BadRequest` variant uses custom numeric error codes:

- `-1` — Generic bad request
- `-5` — Resource not found (used in JSON body, separate from HTTP 404)
- `-6` — Scraper error
- `-7` — Blacklisted
- `-10` — Automated request blocked
- `-429` — Rate limited

These codes are used by the frontend to determine how to display errors. They're a legacy convention from the original FicHub.

## Watch Out!

**Don't log sensitive information in errors!** The `Internal` variant logs the error message, but it should never contain passwords, tokens, or other secrets. Notice that the JSON response to the client says `"internal server error"` — it doesn't expose the actual error details.

**Be careful with `unwrap()`!** In production code, prefer `?` or `unwrap_or_else()` over `unwrap()`. A panic crashes the server and disconnects all connected clients. The only safe place for `unwrap()` is in initialization code (like config loading) where a panic is acceptable because the server hasn't started yet.

**Error conversion is lossy!** When you convert `sqlx::Error` to `AppError::Database`, you lose the original error type. The error message is preserved, but the structured information is gone. This is fine for logging and HTTP responses, but if you need to handle specific SQL errors differently, match on the `sqlx::Error` before converting.

## Summary

In this chapter, we explored:
- How FicHub loads configuration from environment variables
- The `Config` struct and its 40+ settings
- How the `AppError` enum centralizes all error types
- How `IntoResponse` converts errors to HTTP responses
- How `From` implementations make `?` error propagation work
- The `AppResult<T>` type alias for cleaner function signatures
- Custom error codes for the frontend

Next, we'll dive into PostgreSQL and SQLx — how FicHub connects to the database and runs queries.

---

# Chapter 7: PostgreSQL

## Your Data's Best Friend

Every web application needs somewhere to store data that persists across restarts. FicHub uses PostgreSQL — the world's most advanced open-source relational database. PostgreSQL is reliable, feature-rich, and handles everything from simple key-value lookups to complex full-text search queries.

In this chapter, we'll look at how FicHub connects to PostgreSQL, creates a connection pool, and runs queries using SQLx.

## What is SQLx?

SQLx is a Rust library for interacting with PostgreSQL (and other databases). What makes SQLx special is that it can **check your SQL queries at compile time**. If you have a typo in a column name or a type mismatch, the Rust compiler catches it before your code ever runs.

```rust
// This won't compile if the 'title' column doesn't exist in fic_info!
let title: (String,) = sqlx::query_as("SELECT title FROM fic_info WHERE id = $1")
    .bind("abc123")
    .fetch_one(&pool)
    .await?;
```

This is incredibly valuable for catching bugs early. You don't have to run the application and make a request to discover that your SQL is wrong — the compiler tells you immediately.

**Watch Out!** For compile-time SQL checking to work, you need a running database at compile time (or an offline query cache). If you're building in CI/CD or in a Docker container without a database, you'll need to use `sqlx prepare` to generate the cache. For development, just make sure your `DATABASE_URL` environment variable is set.

## Connection Pools

A **connection pool** is a cache of database connections that are reused across requests. Creating a new database connection for every request is expensive (TCP handshake, authentication, etc.), so connection pools maintain a set of open connections and hand them out as needed.

FicHub creates its connection pool in `src/db/mod.rs`:

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

    // Run migrations
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

Let's break this down:

**`PgPoolOptions::new()`** creates a builder for the connection pool.

**`.max_connections(20)`** sets the maximum number of simultaneous connections. 20 is a reasonable default for a web server — enough to handle concurrent requests without overwhelming PostgreSQL.

**`.acquire_timeout(Duration::from_secs(10))`** sets how long to wait for a connection from the pool. If all 20 connections are busy and a new request needs one, it waits up to 10 seconds before failing.

**`.connect(database_url).await?`** actually connects to PostgreSQL and creates the pool.

The migrations section is interesting — it looks for the `migrations/` directory relative to the binary's location. This is because Rust binaries can be moved around, and we want the migrations to follow the binary.

### The Pool Type

`PgPool` is the type that represents a connection pool. It's cheaply clonable (it's an `Arc` internally) and can be shared across async tasks:

```rust
// Clone the pool — this doesn't create a new connection
let pool_clone = pool.clone();

// Use it in different tasks
tokio::spawn(async move {
    let result = sqlx::query("SELECT 1").fetch_one(&pool_clone).await;
});
```

In FicHub, the pool is stored in `AppState` and shared across all handlers:

```rust
pub struct AppState {
    pub db: sqlx::PgPool,
    // ...
}

// In server.rs
let db_pool = db::init_pool(&config.database_url)
    .await
    .expect("Failed to connect to database");

let state = Arc::new(AppState {
    db: db_pool,
    // ...
});
```

## SQL Basics

If you're new to SQL, here's a quick primer on the operations FicHub uses most:

### SELECT — Reading Data

```sql
-- Simple select
SELECT * FROM fic_info WHERE id = 'abc123';

-- Select specific columns
SELECT title, author, words FROM fic_info WHERE status = 'complete';

-- Aggregate functions
SELECT COUNT(*) FROM fic_info;
SELECT SUM(words) FROM fic_info WHERE author = 'SomeAuthor';

-- ORDER BY and LIMIT
SELECT * FROM fic_info ORDER BY words DESC LIMIT 10;

-- JOINs
SELECT fi.title, t.name 
FROM fic_info fi
JOIN fic_tags ft ON ft.url_id = fi.id
JOIN tags t ON t.id = ft.tag_id
WHERE fi.id = 'abc123';

-- Full-text search
SELECT * FROM fic_info 
WHERE text_search @@ plainto_tsquery('english', 'harry potter');
```

### INSERT — Creating Data

```sql
-- Simple insert
INSERT INTO fic_info (id, title, author, words, chapters, status, source, description, fic_created, fic_updated)
VALUES ('abc123', 'My Story', 'Author', 50000, 10, 'complete', 'https://...', 'A story...', NOW(), NOW());

-- Insert with conflict handling (upsert)
INSERT INTO fic_info (id, title, author, words)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    words = EXCLUDED.words;
```

The `ON CONFLICT DO UPDATE` clause is what makes this an "upsert" — if a row with the same `id` already exists, it updates instead of failing. This is crucial for FicHub's caching strategy.

### UPDATE — Modifying Data

```sql
-- Update specific columns
UPDATE fic_info SET words = 51000, chapters = 11 WHERE id = 'abc123';

-- Conditional update
UPDATE fic_tag_votes SET value = 1 WHERE url_id = $1 AND tag_id = $2 AND voter_ip = $3;
```

### DELETE — Removing Data

```sql
-- Delete specific rows
DELETE FROM fic_blacklist WHERE url_id = 'abc123';

-- Delete with cascade (handled by foreign keys)
DELETE FROM tags WHERE id = 1;  -- Also deletes fic_tags, tag_votes, etc.
```

## Querying with SQLx

SQLx provides several ways to run queries:

### `query` — For queries without structured return types

```rust
sqlx::query("INSERT INTO fic_info (id, title) VALUES ($1, $2)")
    .bind("abc123")
    .bind("My Story")
    .execute(&pool)
    .await?;
```

`.execute()` runs the query and returns a `QueryResult` with `rows_affected()`.

### `query_as` — For queries that return structured data

```rust
#[derive(Debug, FromRow)]
struct FicInfo {
    id: String,
    title: String,
    author: String,
    words: i64,
}

let fic: FicInfo = sqlx::query_as("SELECT id, title, author, words FROM fic_info WHERE id = $1")
    .bind("abc123")
    .fetch_one(&pool)
    .await?;
```

The `#[derive(FromRow)]` macro automatically implements the trait that converts database rows to your struct.

### Fetch Methods

- `.fetch_one(&pool)` — Returns exactly one row (errors if zero or multiple)
- `.fetch_optional(&pool)` — Returns `Option<Row>` (None if no rows)
- `.fetch_all(&pool)` — Returns `Vec<Row>` (all matching rows)

### Bind Parameters

SQLx uses PostgreSQL's parameterized queries to prevent SQL injection:

```rust
// SAFE — parameterized query
sqlx::query("SELECT * FROM fic_info WHERE title = $1")
    .bind(user_input)
    .fetch_all(&pool)
    .await?;

// DANGEROUS — string interpolation (NEVER do this!)
let query = format!("SELECT * FROM fic_info WHERE title = '{}'", user_input);
```

The `$1`, `$2`, etc. are parameter placeholders. SQLx sends the parameters separately from the SQL query, so PostgreSQL treats them as data, not code. Even if `user_input` contains SQL metacharacters, they're harmless.

## FicHub's Database Models

FicHub defines its data models in `src/db/models.rs`. The main model is `FicInfo`:

```rust
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

The `Option<T>` types represent nullable columns. A fic might not have an `author_url` or `content_hash`, so those are `Option<String>`.

Other models include:
- `RequestSource` — tracks where requests come from
- `RequestLog` — records every request
- `ExportLog` — tracks generated exports
- `FicBlacklist` — fics that shouldn't be exported
- `AuthorBlacklist` — authors whose fics shouldn't be exported
- `FicVersionBump` — manual cache invalidation

## Watch Out!

**Connection pool sizing matters!** Too few connections and requests queue up waiting. Too many and PostgreSQL runs out of memory or context-switches excessively. 20 is a good starting point for a single-server deployment. For high-traffic deployments, monitor `pg_stat_activity` and adjust accordingly.

**Always use parameterized queries!** Never interpolate user input into SQL strings. SQLx's bind parameters are your defense against SQL injection.

**Migrations run automatically on startup.** This is convenient for development but can be dangerous in production. If a migration fails, the server won't start. Consider running migrations separately in production (with backups!).

**Close connections gracefully.** SQLx's pool handles this automatically, but be aware that long-running queries can hold connections and cause pool exhaustion.

## Summary

In this chapter, you learned:
- What SQLx is and why it's great for Rust + PostgreSQL
- How to create a connection pool with `PgPoolOptions`
- SQL basics: SELECT, INSERT, UPDATE, DELETE
- How SQLx's query methods work (`query`, `query_as`, `fetch_one`, etc.)
- Why parameterized queries prevent SQL injection
- FicHub's database models and their column types
- How migrations run automatically on startup

Next, we'll look at the migration files in detail — the SQL that defines FicHub's database schema.

---

# Chapter 8: Migrations

## The Database Blueprint

Before FicHub can store any data, it needs tables. Tables are defined by the database schema — a blueprint that says "this table has these columns, with these types and constraints." FicHub defines its schema in SQL migration files.

Migrations are versioned SQL files that evolve the database schema over time. Each migration is a set of SQL statements that transform the database from one state to the next. They're designed to be idempotent (safe to run multiple times) and irreversible in practice.

FicHub has four migration files:

```
migrations/
├── 001_initial_schema.sql      # Core tables
├── 002_recommender.sql         # Recommendation engine
├── 003_tagging.sql             # Tag system v3
└── 004_shelves.sql             # OPDS shelves
```

Let's examine each one.

## Migration 001: Initial Schema

This is the foundation — the core tables that everything else builds on.

### request_source

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

This table tracks where requests come from. Each unique combination of `is_automated`, `route`, and `description` gets its own row. The `UNIQUE` constraint ensures we don't create duplicate entries.

- `id` — auto-incrementing primary key (BIGSERIAL is PostgreSQL's 64-bit auto-increment)
- `created` — when the record was created
- `is_automated` — whether the request came from a bot
- `route` — which API route was hit (e.g., "/api/v0/epub")
- `description` — human-readable description (e.g., "web request")

### request_log

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

Every API request is logged here. This table is useful for analytics, debugging, and monitoring:

- `source_id` — links to `request_source`
- `etype` — the export type ("epub", "html", "mobi", "pdf")
- `query` — the original URL the user submitted
- `info_request_ms` — how long the metadata lookup took (in milliseconds)
- `url_id` — the FicHub-internal ID for this fic
- `fic_info` — the scraped metadata (stored as JSON text)
- `export_ms` — how long the export generation took
- `export_file_name` — the filename of the generated file
- `export_file_hash` — the MD5 hash of the generated file

The indexes on this table are carefully chosen:

```sql
CREATE INDEX IF NOT EXISTS idx_request_log_url_id_etype_created
    ON request_log(url_id, etype, created);

CREATE INDEX IF NOT EXISTS idx_request_log_date_export
    ON request_log(created)
    WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

The first index supports lookups by `url_id` and `etype` (used for cache checking). The second is a **partial index** — it only indexes rows where an EPUB was actually exported. This is more efficient than indexing all rows.

### fic_info

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

This is the most important table — it stores metadata for every fic FicHub has ever seen. The `id` is a 12-character hex string generated by SHA-256 hashing the source site's story ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 6 bytes = 12 hex chars
}
```

This ensures the same story always gets the same ID, regardless of when it was first seen.

### export_log

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

This table is the heart of the caching system. For each fic, export type, and input hash, it records the output hash. When a request comes in, FicHub checks this table to see if the export already exists.

The `input_hash` is typically the `content_hash` from the scraper (which changes when the story is updated). The `export_hash` is the MD5 of the generated file.

### fic_blacklist and author_blacklist

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

These tables allow blocking specific fics or authors from being exported. The `reason` column indicates why:
- `5` — DMCA or legal takedown
- `6` — Greylisted (metadata shown, no download)
- `7` — Content policy violation
- `8` — Other

### fic_version_bump

```sql
CREATE TABLE IF NOT EXISTS fic_version_bump (
    id VARCHAR(128) PRIMARY KEY,
    value INT
);
```

This table allows manually invalidating cached exports for a specific fic. If the scraping format changes, you can bump the version to force regeneration.

## Migration 002: Recommender System

This migration adds the tables needed for collaborative filtering recommendations.

### fic_works

```sql
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
```

This table tracks site-specific metadata for each fic, particularly the number of users who have favourited/bookmarked it. The `favouriter_count` is crucial for the recommendation engine — works with more favouriters provide better collaborative filtering signals.

### fic_bookmarks

```sql
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

This records which users have bookmarked which fics. Users are identified by a SHA-256 hash of their profile URL — this preserves anonymity while allowing us to track their preferences.

### fic_bookmark_cooccur

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

This is the co-occurrence matrix — it tracks how many users have bookmarked both `work_a` and `work_b`. The `CHECK (work_a < work_b)` constraint ensures we only store each pair once (avoiding duplicates like A-B and B-A).

The co-occurrence count is the foundation of the recommendation engine. Two works with a high co-occurrence count are frequently bookmarked together, which means they're likely similar.

### recommendation_suggestions and recommendation_votes

```sql
CREATE TABLE IF NOT EXISTS recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);

CREATE TABLE IF NOT EXISTS recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

These tables support community-submitted recommendations. Users can suggest that fic A is similar to fic B, and other users can upvote or downvote that suggestion.

### precomputed_recommendations

```sql
CREATE TABLE IF NOT EXISTS precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

This is a materialized cache of computed recommendations. When the recommendation engine runs, it stores its results here so that subsequent requests can be served instantly without recomputing.

## Migration 003: Tagging System v3

This is the most complex migration, adding the tag system with community voting.

### tag_types

```sql
CREATE TABLE IF NOT EXISTS tag_types (
    id SMALLINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

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

Tag types are a fixed enum — there are exactly 7 types, and they're inserted at migration time. The `ON CONFLICT DO NOTHING` makes this idempotent.

### tags

```sql
CREATE TABLE IF NOT EXISTS tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` is important — it makes tag name comparisons case-sensitive. This means "Harry Potter" and "harry potter" are different tags. Case-sensitive matching is the convention in fanfiction tagging.

### tag_aliases

```sql
CREATE TABLE IF NOT EXISTS tag_aliases (
    alias_name TEXT NOT NULL UNIQUE COLLATE "C",
    canonical_tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

Aliases let curators redirect variant spellings to a canonical tag. For example, "HP" could be an alias for "Harry Potter". When a user submits a tag that matches an alias, it's automatically resolved to the canonical tag.

### fic_tags

```sql
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

This is the junction table connecting fics to tags. The `score` column tracks the net votes (upvotes minus downvotes). Tags with scores below the hidden threshold are not shown to users.

### The Vote Trigger

This is the most sophisticated part of the migration — a PostgreSQL trigger that automatically updates `fic_tags.score` when votes are inserted, updated, or deleted:

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

This trigger runs automatically whenever a row in `fic_tag_votes` is inserted, updated, or deleted. It adjusts the `score` on `fic_tags` to reflect the new vote total. This means the application code never needs to manually recalculate scores — the database handles it.

### Full-Text Search

The migration also adds a generated column for full-text search:

```sql
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX IF NOT EXISTS idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

The `text_search` column is automatically generated from the title (weight A, more important) and description (weight B, less important). The GIN index makes full-text search queries very fast.

## Migration 004: Shelves

The simplest migration — adding OPDS shelves:

```sql
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

INSERT INTO opds_shelves (id, name, description, token)
VALUES (1, 'Default Shelf', 'Shared reading list', 'fichub')
ON CONFLICT (id) DO NOTHING;
```

Shelves are shared reading lists that users can access via OPDS. The `token` field provides authentication — users need to know the token to access a shelf.

## How Migrations Run

FicHub runs migrations automatically at startup. SQLx tracks which migrations have been applied in a special `_sqlx_migrations` table. When the server starts, it checks this table and applies any new migrations.

```rust
sqlx::migrate::Migrator::new(migrations_path)
    .await?
    .run(&pool)
    .await?;
```

This is convenient for development — you just add a new migration file and restart the server. In production, you might want more control (e.g., running migrations during a maintenance window), but for most deployments, automatic migrations work fine.

## Watch Out!

**Never modify applied migrations!** Once a migration has been applied to production, you can't change it. If you need to fix a schema issue, create a new migration.

**Migrations are not reversible!** SQLx doesn't track down-migrations. If you need to roll back, you'll need to write a new migration that undoes the changes.

**Test migrations before deploying!** Run migrations against a copy of your production database to verify they work correctly.

## Summary

In this chapter, we examined all four of FicHub's migration files:
- **001:** Core tables (fic_info, request_log, export_log, blacklists)
- **002:** Recommender tables (fic_works, bookmarks, co-occurrence, suggestions, votes)
- **003:** Tag system (tag_types, tags, aliases, fic_tags, votes, flags, full-text search)
- **004:** OPDS shelves

We also learned about PostgreSQL triggers, generated columns, GIN indexes, and how SQLx manages migrations automatically.

Next, we'll look at how FicHub performs CRUD operations using these tables.

---

# Chapter 9: CRUD Operations

## Create, Read, Update, Delete

The four basic database operations — **CRUD** — are the building blocks of any data-driven application. In this chapter, we'll see how FicHub implements each one using SQLx.

FicHub's query functions live in `src/db/queries.rs`. Let's walk through the most important ones.

## CREATE: Upserting Fic Metadata

The most important "create" operation in FicHub is upserting fic metadata. This is called every time a story is scraped:

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

This is a classic upsert — if the fic already exists (same `id`), update its metadata. If it's new, insert it. The `EXCLUDED` keyword refers to the values that were proposed for insertion.

Notice how many fields there are! FicHub stores a lot of metadata about each fic. The `content_hash` is particularly important — it changes when the story's content changes, which triggers cache invalidation.

## READ: Getting Fic Info

The simplest read operation:

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

Using `fetch_optional` instead of `fetch_one` is important — if the fic doesn't exist, we get `None` instead of an error. This lets the handler decide how to respond.

## READ: Searching Similar Fics

For the "did you mean?" feature:

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

The `ILIKE` operator is case-insensitive `LIKE`. The `%` wildcards match any characters before and after the query.

## CREATE: Logging Requests

Every API request is logged:

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
           (source_id, etype, query, info_request_ms, url_id, fic_info,
            export_ms, export_file_name, export_file_hash, url)
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

Notice the many `Option<&str>` parameters — not all fields are available for every request. The `export_ms` and `export_file_name` are only set when an export is actually generated.

## CREATE: Request Sources

Before logging a request, we need to know the source:

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

The `RETURNING id` clause returns the auto-generated ID, which we need for the request log.

## READ: Cache Lookup

Before generating an export, FicHub checks if it's already cached:

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

The cache key is a composite of `(url_id, version, etype, input_hash)`. If any of these change, the cache is invalidated.

## CREATE: Recording Exports

After generating an export, FicHub records it:

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

## READ: Blacklist Checks

Before exporting, FicHub checks blacklists:

```rust
pub async fn check_fic_blacklist(pool: &PgPool, url_id: &str) -> AppResult<Vec<FicBlacklist>> {
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

These return vectors because a fic or author can be blacklisted for multiple reasons.

## Tag Operations

FicHub's tag system has several CRUD operations:

### Lookup a Tag

```rust
pub async fn lookup_tag_by_name(pool: &PgPool, name: &str) -> AppResult<Option<(i32, String, i16)>> {
    let row = sqlx::query_as::<_, (i32, String, i16)>(
        "SELECT id, name, tag_type_id FROM tags WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
```

### Create a Tag

```rust
pub async fn create_tag(pool: &PgPool, name: &str, tag_type_id: i16) -> AppResult<i32> {
    let row: (i32,) = sqlx::query_as(
        "INSERT INTO tags (name, tag_type_id) VALUES ($1, $2) RETURNING id",
    )
    .bind(name)
    .bind(tag_type_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
```

### Attach a Tag to a Fic

```rust
pub async fn upsert_fic_tag(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    ip: &std::net::IpAddr,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, 0)
           ON CONFLICT (url_id, tag_id) DO UPDATE SET added_by_ip = EXCLUDED.added_by_ip"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(ip.to_string())
    .execute(pool)
    .await?;
    Ok(())
}
```

### Get Tags for a Fic

```rust
pub async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>(
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#,
    )
    .bind(url_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, type_id, score)| {
            let hidden = score < hidden_threshold;
            (id, name, type_id, score, hidden)
        })
        .collect())
}
```

### Record a Vote

```rust
pub async fn upsert_tag_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: &std::net::IpAddr,
    value: i16,
) -> AppResult<()> {
    // Try update first, then insert
    let affected = sqlx::query(
        r#"UPDATE fic_tag_votes SET value = $1, created_at = NOW()
           WHERE url_id = $2 AND tag_id = $3 AND voter_ip = $4::inet"#,
    )
    .bind(value)
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        sqlx::query(
            r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
               VALUES ($1, $2, $3::inet, $4)"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .bind(value)
        .execute(pool)
        .await?;

        // Ensure fic_tags row exists
        sqlx::query(
            r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
               VALUES ($1, $2, $3::inet, 0)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(url_id)
        .bind(tag_id)
        .bind(voter_ip.to_string())
        .execute(pool)
        .await?;
    }
    Ok(())
}
```

This is a manual upsert — try UPDATE first, and if no rows were affected, INSERT. The trigger on `fic_tag_votes` automatically updates the score on `fic_tags`.

## Curator Operations

### Merge Tags

```rust
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    // Reassign fic_tags from source to target (skip duplicates)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft
           WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO UPDATE SET score = GREATEST(fic_tags.score, EXCLUDED.score)"#,
    )
    .bind(source_tag_id)
    .bind(target_tag_id)
    .execute(pool)
    .await?;

    // Delete source fic_tags
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    // Delete source tag
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(source_tag_id)
        .execute(pool)
        .await?;

    Ok(())
}
```

When merging tags, FicHub reassigns all fic-tag associations from the source to the target, keeping the higher score when there's a conflict.

### Delete a Tag

```rust
pub async fn delete_tag(pool: &PgPool, tag_id: i32, force: bool) -> AppResult<()> {
    if !force {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM fic_tags WHERE tag_id = $1",
        )
        .bind(tag_id)
        .fetch_one(pool)
        .await?;
        if count.0 > 0 {
            return Err(AppError::BadRequest(
                -5,
                format!("tag {} is used by {} fics, use ?force=true", tag_id, count.0),
            ));
        }
    }
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

The `force` parameter prevents accidental deletion of tags that are in use. Without `force`, the function checks if any fics use the tag and returns an error if so.

## Watch Out!

**Watch for N+1 queries!** If you fetch a list of fics and then query tags for each one individually, you'll have N+1 database queries. FicHub's search handler avoids this by batching tag lookups:

```rust
let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
let tag_rows = sqlx::query_as::<_, TagDbRow>(
    "SELECT ... WHERE ft.url_id = ANY($1) ..."
)
.bind(&url_ids)
.fetch_all(&pool)
.await?;
```

The `ANY($1)` operator lets you pass an array of IDs and get all matching rows in a single query.

**Always check rows_affected()** when doing updates or deletes. If you expected to update a row but `rows_affected()` is 0, something went wrong.

**Use transactions for multi-step operations!** The `merge_tags` function should really use a transaction to ensure all three operations succeed or none do. Without a transaction, a failure between steps could leave the database in an inconsistent state.

## Summary

In this chapter, we saw how FicHub implements CRUD operations:
- **Upsert** fic metadata with `ON CONFLICT DO UPDATE`
- **Read** fic info, blacklist entries, and export logs
- **Create** request sources and logs
- **Record** tag votes and suggestions
- **Merge** and **delete** tags with curator tools
- **Batch** queries to avoid N+1 problems

Next, we'll look at the router and middleware — how FicHub organizes its routes and adds cross-cutting concerns.

---

# Chapter 10: Router and Middleware

## The Full Route Tree

In Chapter 5, we built a simple router with a few routes. Now it's time to see FicHub's complete router — all 40+ routes organized by subsystem.

The router is built in `server.rs`'s `build_router` function. Let's walk through the entire thing:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    let frontend_dir = state.config.frontend_dir.clone();

    async fn redirect_to_root() -> Redirect {
        Redirect::to("/")
    }

    Router::new()
        // ── Core API ─────────────────────────────────────────────
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // ── Cache Downloads ──────────────────────────────────────
        .route("/cache/{etype}/{url_id}/{fname}", get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}", get(routes::cache_download::download_or_export))

        // ── Recommender ──────────────────────────────────────────
        .route("/api/v0/recommendations", get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest", post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote", post(crate::recommender::routes::vote_handler))
        .route("/api/v0/recommendations/votes", get(crate::recommender::routes::votes_handler))

        // ── Tags (v3) ───────────────────────────────────────────
        .route("/api/v0/tags/submit", post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote", post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag", post(crate::tags::routes::flag_tag))
        .route("/api/v0/tags", get(crate::tags::routes::get_tags))

        // ── Curator ─────────────────────────────────────────────
        .route("/api/v0/curator/alias", post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge", post(crate::tags::curator::merge_tags))
        .route("/api/v0/curator/tags/{id}", delete(crate::tags::curator::delete_tag))
        .route("/api/v0/curator/flags", get(crate::tags::curator::list_flags))
        .route("/api/v0/curator/flags/{id}/resolve", post(crate::tags::curator::resolve_flag))

        // ── Search ──────────────────────────────────────────────
        .route("/api/v0/search", get(crate::search::routes::search_handler))

        // ── OPDS Catalog ────────────────────────────────────────
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/tags/{type_id}", get(crate::routes::opds::tags::tags_by_type))
        .route("/opds/tags/{type_id}/{tag_name}", get(crate::routes::opds::tags::fics_by_tag))
        .route("/opds/authors", get(crate::routes::opds::authors::author_list))
        .route("/opds/recommendations/popular", get(crate::routes::opds::recommendations::popular_recommendations))
        .route("/opds/recommendations", get(crate::routes::opds::recommendations::fic_recommendations))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))
        .route("/opds/shelves", get(crate::routes::opds::shelves::shelf_list))
        .route("/opds/shelf/{shelf_id}", get(crate::routes::opds::shelves::shelf_contents))

        // ── Legacy Redirects ────────────────────────────────────
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))
        .route("/changes", get(redirect_to_root))
        .route("/popular/", get(redirect_to_root))

        // ── Static Frontend (SPA) ───────────────────────────────
        .fallback_service(
            ServeDir::new(&frontend_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(frontend_dir.join("index.html"))),
        )

        // ── Middleware ───────────────────────────────────────────
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())

        // ── State ───────────────────────────────────────────────
        .with_state(state)
}
```

## Route Organization

The routes are organized by subsystem:

### Core API Routes (GET only)
- `/api/` — API documentation
- `/api/v0/epub` — Export endpoint (the main feature)
- `/api/v0/meta` — Metadata-only endpoint
- `/api/v0/remote` — Client IP information

### Cache Download Routes (GET only)
- `/cache/{etype}/{url_id}/{fname}` — Download with hash validation
- `/cache/{etype}/{url_id}` — Download or trigger export

### Recommender Routes (GET + POST)
- `GET /api/v0/recommendations` — Get recommendations
- `POST /api/v0/recommendations/suggest` — Submit a suggestion
- `POST /api/v0/recommendations/vote` — Vote on a suggestion
- `GET /api/v0/recommendations/votes` — List votes for a fic

### Tag Routes (GET + POST)
- `POST /api/v0/tags/submit` — Submit a tag
- `POST /api/v0/tags/vote` — Vote on a tag
- `POST /api/v0/tags/flag` — Flag a tag
- `GET /api/v0/tags` — Get tags for a fic

### Curator Routes (POST + DELETE + GET)
- `POST /api/v0/curator/alias` — Create a tag alias
- `POST /api/v0/curator/merge` — Merge two tags
- `DELETE /api/v0/curator/tags/{id}` — Delete a tag
- `GET /api/v0/curator/flags` — List unresolved flags
- `POST /api/v0/curator/flags/{id}/resolve` — Resolve a flag

### Search Routes (GET only)
- `GET /api/v0/search` — Full-text search with filters

### OPDS Routes (GET only)
All OPDS routes return Atom XML feeds for e-readers.

## Middleware

FicHub uses two middleware layers:

### TraceLayer

```rust
.layer(TraceLayer::new_for_http())
```

This logs every HTTP request with its method, URI, status code, and response time. The log output looks like:

```
2024-01-15T10:30:00.123Z  INFO request{method=GET uri=/api/v0/epub version=HTTP/1.1}: fichub::routes::export: 200 response_time=1.234s
```

This is invaluable for debugging and monitoring. You can see at a glance which requests are slow, which are failing, and how often each endpoint is called.

### CorsLayer

```rust
.layer(CorsLayer::permissive())
```

CORS (Cross-Origin Resource Sharing) is a browser security feature that prevents web pages from making requests to a different domain. Since FicHub's frontend and backend are served from the same domain, this shouldn't be an issue. But `CorsLayer::permissive()` allows any origin, which is useful during development when the frontend might be running on a different port.

In production, you'd restrict CORS to your frontend's domain:

```rust
.layer(CorsLayer::new()
    .allow_origin("https://fichub.example.com".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any))
```

## The Fallback Service

The fallback service handles any request that doesn't match a route:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This serves the SvelteKit frontend's build output as static files. If a request doesn't match any API route, it tries to serve the corresponding file from the frontend directory. If the file doesn't exist, it serves `index.html` — this is the standard SPA (Single Page Application) fallback pattern.

For example:
- `GET /` → serves `frontend/build/index.html`
- `GET /assets/main.js` → serves `frontend/build/assets/main.js`
- `GET /some/random/path` → serves `frontend/build/index.html` (SPA fallback)

## The Server Startup Sequence

Let's look at how the server starts up in `server.rs`:

```rust
pub async fn run(config: Config) {
    // 1. Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // 2. Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // 3. Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // 4. Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // 5. Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(),
        config.dynamic_rate_limit,
    )
    .await
    .expect("Failed to initialize rate limiter");

    // 6. Load datacenter IPs
    if !config.ip_tag_sources.is_empty() {
        rate_limiter.load_datacenter_ips(&config.ip_tag_sources).await;
    }

    // 7. Create shared state
    let recommender_engine = RecommendationEngine::new(db_pool.clone());
    let collection_worker = CollectionWorker::new(
        db_pool.clone(),
        redis_conn.clone(),
        http_client.clone(),
        config.clone(),
        scraper_registry.clone(),
    );
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

    // 8. Build router
    let app = build_router(state).await;

    // 9. Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server error");
}
```

The startup sequence is:
1. Connect to PostgreSQL (with migrations)
2. Connect to Redis
3. Build the HTTP client (with user agent and timeout)
4. Initialize the scraper registry (register all scrapers)
5. Initialize the rate limiter (load the Lua script)
6. Load datacenter IPs (for blocking bot traffic)
7. Create the application state (the big `AppState` struct)
8. Build the router (all routes + middleware)
9. Bind to a port and start serving

Each step is logged, so if something fails during startup, you'll know exactly where the problem is.

## The AppState Struct

The `AppState` struct is the heart of FicHub — it holds everything that needs to be shared across handlers:

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

Let's look at each field:

- **config** — All configuration values
- **db** — PostgreSQL connection pool
- **redis** — Redis multiplexed connection (supports concurrent operations)
- **http_client** — Shared HTTP client for scraping (reuses TCP connections)
- **scraper_registry** — All registered site scrapers
- **cache_semaphores** — Prevents duplicate concurrent exports
- **rate_limiter** — Token bucket rate limiter backed by Redis
- **recommender_engine** — Computes recommendations
- **collection_worker** — Background worker for scraping user favourites

The entire `AppState` is wrapped in `Arc` so it can be shared across all handlers without cloning.

## Watch Out!

**Route order matters!** Axum matches routes in the order they're defined. If you have both `/api/v0/tags` and `/api/v0/tags/submit`, the more specific route should come first. In FicHub, the routes are ordered correctly, but be careful when adding new ones.

**Middleware order matters!** Layers are applied in reverse order. The last `.layer()` added runs first. So `TraceLayer` runs before `CorsLayer`. This is usually what you want, but be aware of it.

**The fallback catches everything!** If you accidentally put your fallback before your API routes, all requests would be served from the frontend directory. Always put the fallback last.

**ConnectInfo requires special setup!** If you use `ConnectInfo` in a handler but forget `into_make_service_with_connect_info`, you'll get a runtime error. The `?` in `into_make_service_with_connect_info::<std::net::SocketAddr>()` is the turbofish syntax for specifying the type.

## Summary

In this chapter, we've seen:
- FicHub's complete route tree (40+ routes)
- How routes are organized by subsystem
- How middleware adds logging and CORS
- How the fallback service serves the SPA frontend
- The server startup sequence (9 steps)
- The `AppState` struct and its role

You now have a solid understanding of the Axum layer of FicHub. In the next part, we'll dive into the scraping system — the heart of FicHub's functionality.


---

# Part 3: The Scraping System

---

# Chapter 11: Fanfiction Sites

## The Landscape of Fanfiction

Fanfiction is a vast, vibrant community where millions of writers create stories based on existing universes — Harry Potter, Marvel, Star Wars, anime, video games, and thousands more. These stories are published on dedicated platforms, each with its own community, culture, and technical infrastructure.

Understanding these platforms is the first step in building FicHub's scraping system. Each site has unique URL patterns, HTML structures, rate limiting policies, and quirks that our scrapers must handle.

## Archive of Our Own (AO3)

AO3 is the largest and most popular fanfiction archive, operated by the Organization for Transformative Works (OTW), a nonprofit organization. It hosts millions of stories across hundreds of thousands of fandoms.

**URL Patterns:**
- Work page: `https://archiveofourown.org/works/12345678`
- Chapter page: `https://archiveofourown.org/works/12345678/chapters/87654321`
- Full work: `https://archiveofourown.org/works/12345678?view_full_work=true`

The work ID is a numeric identifier. The `?view_full_work=true` parameter is crucial — it tells AO3 to render all chapters on a single page, which dramatically simplifies scraping.

**HTML Structure:**

AO3 uses clean, semantic HTML with consistent class names:

```html
<div id="workskin">
  <div class="preface group">
    <h2 class="title heading">
      My Amazing Fanfiction
      <span class="chapter" title="Chapters: 10">Chapters: 10/10</span>
    </h2>
    <h3 class="byline heading">
      <a href="/users/SomeAuthor" rel="author">SomeAuthor</a>
    </h3>
    <div class="summary module">
      <blockquote class="userstuff">
        <p>A story about things happening...</p>
      </blockquote>
    </div>
    <dl class="stats">
      <dt>Words:</dt><dd class="words">125,000</dd>
      <dt>Chapters:</dt><dd class="chapters">10/10</dd>
      <dt>Status:</dt><dd class="status">Complete</dd>
      <dt>Published:</dt><dd class="published">2023-01-15</dd>
      <dt>Updated:</dt><dd class="updated">2023-06-20</dd>
    </dl>
  </div>
  <div id="chapters">
    <div class="chapter" id="chapter-1">
      <h3 class="heading">
        <a href="/works/12345678/chapters/11111111">Chapter 1</a>
      </h3>
      <div class="userstuff">
        <p>Once upon a time in a land far away...</p>
      </div>
    </div>
    <div class="chapter" id="chapter-2">
      <h3 class="heading">
        <a href="/works/12345678/chapters/22222222">Chapter 2</a>
      </h3>
      <div class="userstuff">
        <p>The adventure continued as they...</p>
      </div>
    </div>
  </div>
</div>
```

**Key Selectors:**
- Title: `h2.title.heading`
- Author: `a[rel='author']`
- Description: `blockquote.userstuff`
- Word count: `dd.words`
- Chapter count: `dd.chapters` (format: "3/10")
- Status: `dd.status` ("Complete" or "In Progress")
- Chapter content: `div.chapter` > `div.userstuff`
- Tags: `ul.tags li.fandom a.tag`, `ul.tags li.character a.tag`, etc.

**Rate Limiting:** AO3 is strict about rate limiting. FicHub identifies itself with a User-Agent header and includes reasonable delays. Excessive requests result in temporary blocks or CAPTCHA challenges.

**Content Warnings:** AO3 uses a warnings system (e.g., "No Archive Warnings Apply", "Graphic Depictions Of Violence"). These appear in the metadata and can be extracted as tags.

**Series and Collections:** Stories can be part of series or collections. FicHub currently doesn't scrape these, but they could be added in the future.

## FanFiction.net (FF.net)

FF.net is one of the oldest fanfiction sites, launched in 1998. It has a more complex HTML structure with many CSS classes and data attributes.

**URL Patterns:**
- Story page: `https://www.fanfiction.net/s/12345678/1/` (story ID / chapter number)
- Chapter page: `https://www.fanfiction.net/s/12345678/3/` (chapter 3)

The story ID and chapter number are both in the URL path. Unlike AO3, FF.net doesn't have a "view full work" option, so each chapter must be fetched individually.

**HTML Structure:**

FF.net uses a more complex structure with data attributes:

```html
<div id="profile_top">
  <b class="xcontrast_txt">My Amazing Fanfiction</b>
  <a class="xcontrast_txt" href="/u/123456/SomeAuthor">SomeAuthor</a>
  <div class="xcontrast_txt">A story about things happening...</div>
  <span class="xgray">
    Rated: T | English | Humor/Romance | Chapters: 10 | Words: 125,000
  </span>
  <span data-xutitle="word count">125,000</span>
  <span data-xutitle="chapters">10/10</span>
</div>
<div class="storytext" id="storytext">
  <p>Once upon a time in a land far away...</p>
</div>
```

**Key Selectors:**
- Title: `#profile_top b.xcontrast_txt`
- Author: `#profile_top a.xcontrast_txt`
- Description: `#profile_top div.xcontrast_txt`
- Word count: `#profile_top span[data-xutitle='word count']`
- Chapter count: `#profile_top span[data-xutitle='chapters']`
- Chapter content: `div.storytext`
- Chapter select: `select#chap_select option[selected]`

**Fandom Categories:** FF.net uses a category system (Anime, Books, Cartoons, etc.) with subcategories (Harry Potter, Lord of the Rigs, etc.). These appear in the page metadata.

**Favorites and Follows:** FF.net tracks favorites and follows, but these aren't exposed in the same way as AO3's kudos.

**Challenges:** FF.net has been around since 1998 and its HTML reflects that legacy. Selectors can be fragile, and the site occasionally makes changes that break scrapers.

## XenForo Forums

XenForo is forum software used by several fanfiction communities. The most notable are:
- SpaceBattles (`forums.spacebattles.com`)
- SufficientVelocity (`forums.sufficientvelocity.com`)
- QuestionableQuesting (`forum.questionablequesting.com`)

**URL Pattern:** `https://forums.spacebattles.com/threads/story-name.12345/`

Stories are posted as forum threads. Each post can be a chapter, a comment, or off-topic discussion.

**HTML Structure:**

```html
<h1 class="p-title-value">Story Title</h1>
<div class="message-userContent">
  <a class="username" href="/members/author.12345/">AuthorName</a>
</div>
<article class="message-body">
  <div class="bbWrapper">
    <p>Once upon a time...</p>
  </div>
</article>
```

**Key Selectors:**
- Thread title: `h1.p-title-value`
- Author: `a.username`
- Post content: `article.message-body`
- Post content inner: `div.bbWrapper`

**Challenges:**
1. **Not all posts are story content.** Posts can be comments, discussions, or off-topic. A production scraper needs to filter these.
2. **Pagination.** Long threads span multiple pages. FicHub currently only scrapes the first page.
3. **User formatting.** Forum posts can contain embedded images, videos, spoilers, and custom BBCode that doesn't translate cleanly to HTML.
4. **Thread prefixes.** Threads often have prefixes like "[Complete]" or "[WIP]" that indicate story status.

## FictionPress

FictionPress is owned by the same company as FF.net and uses an identical HTML structure. FicHub handles this by reusing the FF.net scraper:

```rust
pub use super::ffnet::FfNetScraper as FictionPressScraper;
```

This is a clean example of code reuse through Rust's type system.

## AdultFanFiction.org (AFF)

AFF is a smaller site for adult-rated fanfiction. It has a simpler HTML structure:

**URL Pattern:** `https://www.adult-fanfiction.org/story/12345`

**Key Selectors:**
- Title: `h1.story_title`
- Author: `a.author`
- Description: `div.story_description`
- Content: `div.story_content`

AFF requires age verification, but the story pages themselves are accessible without authentication.

## HP Fan Fiction Archive

This covers two related sites:
- hpfanficarchive.com
- fanficauthors.net

Both are Harry Potter-specific archives with similar structures:

**Key Selectors:**
- Title: `h1`
- Author: `a[href*='author']`
- Content: `div.story-content, div.fic-content, article`

The flexible content selector tries multiple possible structures, making the scraper more resilient to layout changes.

## Site Comparison

| Feature | AO3 | FF.net | XenForo | AFF |
|---------|-----|--------|---------|-----|
| Full work view | Yes | No | No | No |
| Chapter selection | URL param | URL path | N/A | N/A |
| Tag system | Rich | Categories | Thread prefix | Basic |
| Rate limiting | Strict | Moderate | Varies | Lenient |
| HTML quality | Good | Legacy | Forum | Simple |
| Authentication | Optional | None | None | Age gate |

## Anti-Scraping Considerations

Each site has different anti-scraping measures:

- **AO3:** Rate limiting, CAPTCHA after excessive requests
- **FF.net:** Aggressive bot detection, IP blocking
- **XenForo:** Varies by installation, some use Cloudflare
- **AFF:** Age verification gate
- **HP FanFic:** Minimal protections

FicHub's approach is respectful: we identify ourselves with a User-Agent header, include reasonable delays, and don't overload the sites. This is both ethical and practical — aggressive scraping leads to blocks.

## Summary

Each fanfiction site has its own personality, technical infrastructure, and challenges. FicHub handles this diversity with a pluggable scraper system where each site has its own implementation. In the next chapter, we'll see how these scrapers are built using the `reqwest` and `scraper` crates.

---

# Chapter 12: Building Scrapers

## The HTTP Client

Every scraper needs to make HTTP requests. FicHub uses `reqwest`, the most popular Rust HTTP client:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

The `user_agent` is crucial — many sites block requests without a recognized User-Agent. FicHub identifies itself as "fichub.net/0.1.0" so site administrators can contact us if there are issues.

The timeout prevents requests from hanging indefinitely if a site is slow or unresponsive.

## The HTML Parser

FicHub uses the `scraper` crate for parsing HTML:

```rust
use scraper::{Html, Selector};

let html = response.text().await?;
let document = Html::parse_document(&html);

// Parse a CSS selector
let selector = Selector::parse("h2.title.heading").unwrap();

// Find matching elements
for element in document.select(&selector) {
    let text: String = element.text().collect();
    println!("Found: {}", text);
}
```

The `scraper` crate is built on top of `html5ever`, which is the same HTML parser used by Firefox. It handles malformed HTML gracefully.

## The SiteScraper Trait

Every scraper implements this trait:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;

    async fn lookup(&self, client: &reqwest::Client, url: &str)
        -> Result<FicMetadata, ScrapeError>;

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
        -> Result<Vec<Chapter>, ScrapeError>;

    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str)
        -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

### can_handle

This method determines if a scraper can handle a given URL:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

It's simple string matching. For more complex patterns, you could use regex:

```rust
fn can_handle(&self, url: &str) -> bool {
    regex_lite::Regex::new(r"https?://(www\.)?fanfiction\.net/s/\d+").unwrap().is_match(url)
}
```

### lookup

This method fetches metadata without downloading chapter content:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str)
    -> Result<FicMetadata, ScrapeError>
{
    // 1. Fetch the page
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    
    // 2. Parse HTML
    let document = Html::parse_document(&html);
    
    // 3. Extract data
    let title = extract_text(&document, "h2.title.heading");
    let author = extract_text(&document, "a[rel='author']");
    // ... more fields ...
    
    // 4. Build result
    Ok(FicMetadata { title, author, /* ... */ })
}
```

### fetch_chapters

This method downloads all chapter content:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata)
    -> Result<Vec<Chapter>, ScrapeError>
{
    let mut chapters = Vec::new();
    
    for i in 1..=meta.chapters {
        let url = format!("{}/s/{}/{}", BASE_URL, meta.author_local_id, i);
        let response = client.get(&url).send().await?;
        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        let content = extract_html(&document, "div.storytext");
        chapters.push(Chapter {
            chapter_id: i,
            title: format!("Chapter {}", i),
            content,
        });
    }
    
    Ok(chapters)
}
```

### extract_tags

This optional method extracts structured tags:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str)
    -> Result<Vec<ExtractedTag>, ScrapeError>
{
    let response = client.get(url).send().await?;
    let html = response.text().await?;
    let document = Html::parse_document(&html);
    
    let mut tags = Vec::new();
    
    // Extract fandom tags
    for el in document.select(&Selector::parse("ul.tags li.fandom a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }
    
    // ... more tag types ...
    
    Ok(tags)
}
```

## Helper Functions

Let's define some helper functions that all scrapers can use:

```rust
/// Extract text content from the first matching element
fn extract_text(document: &Html, selector: &str) -> String {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_default()
}

/// Extract HTML content from the first matching element
fn extract_html(document: &Html, selector: &str) -> String {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default()
}

/// Extract an attribute from the first matching element
fn extract_attr(document: &Html, selector: &str, attr: &str) -> Option<String> {
    document
        .select(&Selector::parse(selector).unwrap())
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.to_string())
}

/// Extract text with a fallback default
fn extract_text_or(document: &Html, selector: &str, default: &str) -> String {
    let text = extract_text(document, selector);
    if text.is_empty() {
        default.to_string()
    } else {
        text
    }
}
```

## URL ID Generation

Every fic needs a unique, deterministic ID:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 6 bytes = 12 hex chars
}
```

The ID is derived from the source site's story ID, ensuring the same story always gets the same ID regardless of when it was first seen.

## The FicMetadata Struct

This is the universal representation of scraped metadata:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicMetadata {
    pub url_id: String,           // Deterministic ID (12 hex chars)
    pub title: String,            // Story title
    pub author: String,           // Author name
    pub chapters: i32,            // Number of chapters
    pub words: i64,               // Word count
    pub desc: String,             // Description/summary (HTML)
    pub published: i64,           // Publication date (unix millis)
    pub updated: i64,             // Last update date (unix millis)
    pub status: String,           // "ongoing", "complete", "hiatus", "cancelled"
    pub source: String,           // Original URL
    pub source_id: i64,           // Site identifier (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,           // Author identifier (site-specific)
    pub author_url: String,       // Author's profile URL
    pub author_local_id: String,  // Site-specific story/author ID
    pub content_hash: Option<String>,  // Hash of story content (for cache invalidation)
    pub extra_meta: Option<String>,    // Additional metadata (JSON)
    pub raw_extended_meta: Option<String>,  // Raw extended metadata
}
```

The `content_hash` is particularly important — it changes when the story's content changes, which triggers cache invalidation.

## The Chapter Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,  // HTML content
}
```

The `content` field contains HTML, not plain text. This is crucial because the EPUB generator needs HTML for proper formatting.

## Error Handling

Scrapers can fail in several ways:

```rust
#[derive(Debug)]
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
```

- **NotFound** — The story doesn't exist or has been deleted
- **Blocked** — The site is blocking our requests
- **Network** — A network error occurred (timeout, connection refused, etc.)
- **ParseError** — The HTML structure changed and we couldn't extract data

## The ExtractedTag Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTag {
    pub name: String,
    pub tag_type_id: i16,
}

impl ExtractedTag {
    pub fn fandom(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 1 } }
    pub fn character(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 2 } }
    pub fn relationship(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 3 } }
    pub fn freeform(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 4 } }
    pub fn warning(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 5 } }
    pub fn category(name: &str) -> Self { Self { name: name.to_string(), tag_type_id: 6 } }
}
```

These type IDs match the `tag_types` table in the database.

## Building a Complete Scraper: AO3

Let's build the AO3 scraper from scratch, step by step.

### Step 1: Define the struct

```rust
pub struct Ao3Scraper;
```

It's a unit struct — no fields needed.

### Step 2: Implement can_handle

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

### Step 3: Extract the work ID

```rust
fn extract_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

### Step 4: Implement lookup

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let title = extract_text_or(&document, "h2.title.heading", "Unknown Title");
    let author = extract_text_or(&document, "a[rel='author']", "Unknown Author");
    let author_url = extract_attr(&document, "a[rel='author']", "href")
        .map(|h| format!("{BASE_URL}{h}"))
        .unwrap_or_default();
    let description = extract_html(&document, "blockquote.userstuff");

    // Extract word count (remove commas)
    let words_text = extract_text(&document, "dd.words");
    let words = words_text.replace(',', "").parse().unwrap_or(0);

    // Extract chapter count
    let chapters_text = extract_text(&document, "dd.chapters");
    let chapters = if let Some(pos) = chapters_text.find('/') {
        chapters_text[..pos].trim().parse().unwrap_or(1)
    } else {
        1
    };

    // Extract status
    let status_text = extract_text(&document, "dd.status");
    let status = if status_text.contains("Complete") {
        "complete".to_string()
    } else {
        "ongoing".to_string()
    };

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
        source_id: 1,
        author_id: 0,
        author_url,
        author_local_id: work_id,
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    })
}
```

### Step 5: Implement fetch_chapters

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
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

    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }

    // Handle single-chapter works
    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
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

## Watch Out!

**Selector fragility!** CSS selectors break when sites change their HTML. Use the most stable selectors possible. IDs are better than classes. Semantic elements are better than divs.

**Don't trust the HTML!** Fanfiction sites can have malformed HTML. The `scraper` crate is tolerant of this, but always handle missing elements gracefully.

**Respect robots.txt!** Check the site's robots.txt before scraping. FicHub identifies itself and requests reasonable rates.

**User-Agent matters!** Many sites block requests without a recognized User-Agent. Always set one.

## Summary

In this chapter, we learned:
- How `reqwest` and `scraper` work together for HTTP + HTML parsing
- The `SiteScraper` trait and its four methods
- How to extract text, HTML, and attributes with CSS selectors
- The `FicMetadata` and `Chapter` structs
- Error handling with `ScrapeError`
- How to build a complete scraper from scratch
- Helper functions for common extraction patterns

---

# Chapter 13: Scraper Registry

## The Registry Pattern

FicHub supports multiple fanfiction sites, each with its own scraper. The **ScraperRegistry** is a central lookup table that maps URLs to the appropriate scraper.

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}
```

The registry holds a vector of boxed trait objects. Each scraper is boxed because they're different concrete types that all implement the same trait.

## Creating the Registry

The registry is created with all known scrapers:

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

The `Default` trait is also implemented:

```rust
impl Default for ScraperRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

## Finding a Scraper

The `find_scraper` method iterates through all scrapers:

```rust
pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
    self.scrapers.iter().find(|s| s.can_handle(url))
}
```

This returns a reference to the first scraper that can handle the URL.

## Convenience Methods

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

pub fn scraper_count(&self) -> usize {
    self.scrapers.len()
}
```

## Sharing the Registry

The registry is wrapped in `Arc` and shared across all handlers:

```rust
let scraper_registry = Arc::new(ScraperRegistry::new());
```

`Arc` (Atomic Reference Counting) allows multiple handlers to hold a reference to the same registry without cloning it.

## Adding a New Scraper

To add support for a new site:

1. Create `src/scrape/sites/mynewsite.rs`
2. Implement `SiteScraper` for your struct
3. Add `pub mod mynewsite;` to `src/scrape/sites/mod.rs`
4. Register it in `ScraperRegistry::new()`

```rust
// In registry.rs
scrapers.push(Box::new(sites::mynewsite::MyNewSiteScraper));
```

## Watch Out!

**Scraper order matters!** If two scrapers can handle the same URL, the first one wins. Make sure `can_handle` methods are specific.

**Dynamic dispatch has overhead!** Each method call goes through dynamic dispatch. For FicHub's use case, this is negligible — network I/O dominates.

## Summary

The ScraperRegistry provides a clean interface for managing multiple scrapers. It's easy to extend and efficiently shares state across handlers.

---

# Chapter 14: AO3 Deep Dive

## Understanding AO3's HTML

AO3 has relatively clean, semantic HTML. The key elements are:

**Title:** `h2.title.heading`
**Author:** `a[rel='author']`
**Description:** `blockquote.userstuff`
**Word count:** `dd.words`
**Chapter count:** `dd.chapters` (format: "3/10")
**Status:** `dd.status` ("Complete" or "In Progress")
**Chapter content:** `div.chapter` > `div.userstuff`
**Tags:** `ul.tags li.fandom a.tag`, `ul.tags li.character a.tag`, etc.

## The lookup Method

The lookup method extracts metadata from the page:

```rust
async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("could not extract work ID".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client
        .get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ScrapeError::NotFound);
    }

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);
```

The `?view_full_work=true` parameter tells AO3 to show all chapters on a single page.

### Extracting the Title

```rust
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());
```

### Extracting the Author

```rust
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
```

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

Word counts are formatted with commas (e.g., "125,000"), so we remove them before parsing.

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

The chapter count is in the format "10/10" (current/total).

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

## The fetch_chapters Method

This method downloads all chapter content:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
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

    let mut chapters = Vec::new();
    for (i, chapter_div) in document.select(&chapter_sel).enumerate() {
        let chapter_title = chapter_div
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| format!("Chapter {}", i + 1));

        let content = chapter_div
            .select(&Selector::parse("div.userstuff").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

        chapters.push(Chapter {
            chapter_id: (i + 1) as i32,
            title: chapter_title,
            content,
        });
    }

    if chapters.is_empty() {
        if let Some(body) = document.select(&Selector::parse("div.userstuff").unwrap()).next() {
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

The key insight: AO3 wraps each chapter in `div.chapter`. We iterate over all chapter divs and extract the title and content from each one.

**inner_html vs text:** We use `.inner_html()` for chapter content because the EPUB generator needs HTML structure for proper formatting. We use `.text()` for metadata where we only need plain text.

## Extracting Tags

AO3 provides structured tags:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    let work_id = Self::extract_work_id(url)
        .ok_or_else(|| ScrapeError::ParseError("no work ID".into()))?;

    let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
    let response = client.get(&fic_url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;

    let html = response.text().await
        .map_err(|e| ScrapeError::Network(e.to_string()))?;
    let document = Html::parse_document(&html);

    let mut tags = Vec::new();

    // Fandom tags
    for el in document.select(&Selector::parse("ul.tags li.fandom a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }

    // Character tags
    for el in document.select(&Selector::parse("ul.tags li.character a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::character(&name));
        }
    }

    // Relationship tags
    for el in document.select(&Selector::parse("ul.tags li.relationship a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::relationship(&name));
        }
    }

    // Freeform tags
    for el in document.select(&Selector::parse("ul.tags li.freeform a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::freeform(&name));
        }
    }

    // Warning tags
    for el in document.select(&Selector::parse("ul.tags li.warnings a.tag").unwrap()) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::warning(&name));
        }
    }

    Ok(tags)
}
```

## Watch Out!

**AO3 rate limits aggressively!** Excessive requests lead to CAPTCHAs or temporary blocks.

**Some works require authentication!** Mature-rated works may need login.

**HTML changes break scrapers!** AO3 occasionally updates its HTML structure.

## Summary

The AO3 scraper demonstrates the full scraping workflow: URL detection, HTTP fetching, HTML parsing, data extraction, and error handling. It also shows how to extract structured tags for the tagging system.

---

# Chapter 15: Other Scrapers

## FF.net Scraper

FF.net's scraper fetches chapters individually:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let story_id = &meta.author_local_id;

    for i in 1..=meta.chapters {
        let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
        let response = client
            .get(&url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content = document
            .select(&Selector::parse("div.storytext").unwrap())
            .next()
            .map(|el| el.inner_html())
            .unwrap_or_default();

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
```

The chapter title is extracted from the chapter selector dropdown.

## XenForo Scraper

XenForo treats forum posts as chapters:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let mut chapters = Vec::new();
    let url = &meta.source;
    let response = client.get(url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send()
        .await
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
            chapter_idx as i32,
            title,
            content,
        });
    }

    if chapters.is_empty() {
        return Err(ScrapeError::ParseError("no content found".into()));
    }

    Ok(chapters)
}
```

## AdultFanFiction Scraper

AFF has a simpler structure with flexible content selectors:

```rust
let content_sel = Selector::parse("div.story_content, div.content, div#story").unwrap();
```

The comma-separated selector tries multiple possible structures.

## HP FanFic Scraper

The HP FanFic scraper handles multiple domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("hpfanficarchive.com") || url.contains("fanficauthors.net")
}
```

## Adding a New Scraper: Complete Example

Here's a template for adding a hypothetical new site:

### Step 1: Create the file

Create `src/scrape/sites/mynewsite.rs`:

```rust
pub struct MyNewSiteScraper;

use async_trait::async_trait;
use crate::scrape::{FicMetadata, Chapter, SiteScraper, ScrapeError};
use scraper::{Html, Selector};
use chrono::Utc;

#[async_trait]
impl SiteScraper for MyNewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("mynewsite.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        let response = client.get(url)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ScrapeError::NotFound);
        }

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let title = document
            .select(&Selector::parse("h1").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("a.author").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let id = url.split('/').last().unwrap_or("unknown").to_string();
        let url_id = crate::scrape::generate_url_id(6, &id);
        let now = Utc::now().timestamp_millis();

        Ok(FicMetadata {
            url_id,
            title,
            author,
            chapters: 1,
            words: 0,
            desc: String::new(),
            published: now,
            updated: now,
            status: "ongoing".to_string(),
            source: url.to_string(),
            source_id: 6,
            author_id: 0,
            author_url: String::new(),
            author_local_id: id,
            content_hash: None,
            extra_meta: None,
            raw_extended_meta: None,
        })
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let response = client.get(&meta.source)
            .header("User-Agent", "fichub.net/0.1.0")
            .send()
            .await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;

        let html = response.text().await
            .map_err(|e| ScrapeError::Network(e.to_string()))?;
        let document = Html::parse_document(&html);

        let content_sel = Selector::parse("div.story-content, div.content, article").unwrap();
        let mut chapters = Vec::new();

        for (i, div) in document.select(&content_sel).enumerate() {
            chapters.push(Chapter {
                chapter_id: (i + 1) as i32,
                title: format!("Chapter {}", i + 1),
                content: div.inner_html(),
            });
        }

        Ok(chapters)
    }
}
```

### Step 2: Add the module

In `src/scrape/sites/mod.rs`:

```rust
pub mod ao3;
pub mod ffnet;
pub mod xenforo;
pub mod fictionpress;
pub mod adultfanfiction;
pub mod hpfanfic;
pub mod mynewsite;
```

### Step 3: Register the scraper

In `src/scrape/registry.rs`:

```rust
scrapers.push(Box::new(sites::mynewsite::MyNewSiteScraper));
```

### Step 4: Test

```bash
cargo build
cargo test
```

## Watch Out!

**Source IDs must be unique!** Each site needs a unique `source_id`.

**Selector specificity matters!** Use more specific selectors to avoid matching wrong elements.

**Handle missing data gracefully!** Always provide defaults for optional fields.

## Summary

Each scraper handles its site's unique quirks. Adding a new scraper follows a consistent four-step process: create the file, add the module, register the scraper, and test.


---

# Part 4: Export and Caching

---

# Chapter 16: EPUB Generation

## What is EPUB?

EPUB (Electronic Publication) is the most widely supported e-book format, defined by the IDPF (International Digital Publishing Forum). It's essentially a ZIP file containing XHTML files, CSS stylesheets, images, and metadata described by XML files. The structure follows the Open Packaging Format (OPF).

EPUB files can be read on virtually any e-reader: Kindle (with conversion to AZW3), Kobo, Nook, Apple Books, Google Play Books, and many Android reading apps. They're also readable in web browsers using browser extensions or online readers.

FicHub generates EPUBs using the `epub-builder` Rust crate, which provides a clean API for constructing EPUB files without dealing with the underlying XML complexity.

## The EPUB File Structure

An EPUB file is a ZIP archive with a specific internal structure:

```
story.epub (ZIP file)
├── mimetype                    # Must be first, uncompressed
├── META-INF/
│   └── container.xml           # Points to the OPF file
└── OEBPS/
    ├── content.opf             # Package document (metadata, manifest, spine)
    ├── toc.ncx                 # Navigation control (for older readers)
    ├── stylesheet.css          # Shared CSS
    ├── introduction.xhtml      # Title page
    ├── chapter_1.xhtml         # Chapter 1
    ├── chapter_2.xhtml         # Chapter 2
    └── ...                     # More chapters
```

The `epub-builder` crate handles all of this complexity — you just provide the metadata, content, and styles, and it assembles the EPUB correctly.

## The EPUB Generation Process

When FicHub generates an EPUB, it follows these steps:

### Step 1: Create a Working Directory

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

Each EPUB is generated in a unique UUID-named directory. This prevents conflicts when multiple EPUBs are generated concurrently.

### Step 2: Initialize the Builder

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
```

The `ZipLibrary` provides the ZIP compression backend. The builder manages the internal EPUB structure.

### Step 3: Set Metadata

```rust
builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

Metadata is stored in the OPF file and is used by e-readers to display book information.

### Step 4: Add CSS Stylesheet

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

The stylesheet is inlined into the EPUB. The `epub-builder` crate writes it as `stylesheet.css` and links it from all content files.

### Step 5: Add Introduction Page

The introduction page displays the story's metadata:

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

The introduction gives readers context before they start reading.

### Step 6: Add Chapters

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

Each chapter becomes a separate XHTML file in the EPUB. The `epub-builder` crate handles the internal linking and spine ordering.

### Step 7: Write the EPUB File

```rust
let epub_path = work_dir.join("output.epub");
let file = fs::File::create(&epub_path)?;
builder.generate(file)?;
```

The `generate` method writes the complete EPUB structure to the file.

### Step 8: Compute MD5 Hash

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();
```

The MD5 hash serves two purposes:
1. **Cache key** — Stored in the database to check if the same EPUB already exists
2. **Integrity check** — Verified when serving the file to detect corruption

## HTML Escaping

User-provided content must be escaped to prevent HTML injection:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

Without this, a title like `My <script>alert('xss')</script> Fic` would be rendered as HTML instead of displayed as text.

## Timestamp Formatting

Unix millisecond timestamps are converted to human-readable dates:

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

## Watch Out!

**EPUB generation is synchronous!** The `epub-builder` crate blocks the current task. For most stories, this is fine (a few seconds). For very long stories, consider using `tokio::task::spawn_blocking`.

**Memory usage!** The entire EPUB is built in memory. For very long stories, this could use significant memory.

**XML compliance!** All content must be valid XHTML. The `epub-builder` crate handles this, but malformed HTML from scrapers can cause issues.

## Summary

FicHub's EPUB generation produces well-formatted e-books with proper metadata, chapter navigation, and CSS styling. The MD5 hash ensures cache validity and file integrity.

---

# Chapter 17: HTML Bundles

## Why HTML Bundles?

While EPUBs are great for e-readers, sometimes you just want a simple HTML file you can open in any browser. FicHub generates HTML bundles alongside EPUBs — self-contained ZIP files containing a single HTML page with all chapters, metadata, and navigation.

## The HTML Bundle Structure

The generated HTML has a clean, readable layout:

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>My Amazing Fanfiction — SomeAuthor</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: Georgia, 'Times New Roman', serif;
            line-height: 1.7;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #fafafa;
        }
        h1 { text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }
        h2 {
            text-align: center;
            margin: 1.5em 0 0.5em;
            font-size: 1.4em;
            border-bottom: 1px solid #ddd;
            padding-bottom: 0.3em;
        }
        .meta { text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }
        .meta td { padding: 2px 8px; }
        .desc {
            margin: 1em 0;
            padding: 1em;
            background: #fff;
            border-radius: 4px;
            border: 1px solid #eee;
        }
        .nav {
            background: #fff;
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 1em;
            margin: 1.5em 0;
        }
        .nav h3 { margin-bottom: 0.5em; }
        .nav ul { list-style: none; columns: 2; }
        .nav li { padding: 2px 0; }
        .nav a { color: #1a5276; text-decoration: none; }
        .nav a:hover { text-decoration: underline; }
        .content p { margin: 0.5em 0; text-indent: 1.5em; }
        .content p:first-of-type { text-indent: 0; }
        hr { border: none; border-top: 1px solid #ddd; margin: 2em 0; }
        .footer { text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }
    </style>
</head>
<body>
    <h1>My Amazing Fanfiction</h1>
    <div class="meta">
        <p>by <strong>SomeAuthor</strong></p>
        <table align="center">
            <tr><td>Words:</td><td>125000</td></tr>
            <tr><td>Chapters:</td><td>10</td></tr>
            <tr><td>Status:</td><td>complete</td></tr>
            <tr><td>Published:</td><td>2023-01-15</td></tr>
            <tr><td>Updated:</td><td>2023-06-20</td></tr>
        </table>
    </div>
    <div class="desc">
        A story about things happening...
    </div>
    <hr/>
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>
            <li><a href="#ch1">Chapter 1</a></li>
            <li><a href="#ch2">Chapter 2</a></li>
            <li><a href="#ch3">Chapter 3</a></li>
        </ul>
    </div>
    <hr/>
    <div class="content">
        <h2 id="ch1">Chapter 1</h2>
        <p>Once upon a time in a land far away...</p>
        <h2 id="ch2">Chapter 2</h2>
        <p>The adventure continued as they...</p>
        <h2 id="ch3">Chapter 3</h2>
        <p>And so the journey reached its climax...</p>
    </div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — https://archiveofourown.org/works/12345678</p>
    </div>
</body>
</html>
```

Key features:
- **Responsive design** — Works on mobile and desktop
- **Chapter navigation** — Clickable links to jump between chapters
- **Clean typography** — Serif font, comfortable line height, proper spacing
- **Metadata header** — Title, author, word count, status, dates
- **Description** — Story summary in a highlighted box

## The Generation Process

```rust
pub async fn create_html_bundle(
    meta: &FicMetadata,
    chapters: &[Chapter],
    tmp_dir: &Path,
) -> Result<(PathBuf, String), ExportError> {
    // Create working directory
    let uuid = Uuid::new_v4();
    let work_dir = tmp_dir.join(uuid.to_string());
    fs::create_dir_all(&work_dir)?;

    // Build navigation and content
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

    // Assemble the full HTML
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{ font-family: Georgia, 'Times New Roman', serif; line-height: 1.7; color: #333; max-width: 800px; margin: 0 auto; padding: 20px; background: #fafafa; }}
        h1 {{ text-align: center; margin: 1.5em 0 0.3em; font-size: 1.8em; }}
        h2 {{ text-align: center; margin: 1.5em 0 0.5em; font-size: 1.4em; border-bottom: 1px solid #ddd; padding-bottom: 0.3em; }}
        .meta {{ text-align: center; color: #666; margin-bottom: 2em; font-size: 0.95em; }}
        .meta td {{ padding: 2px 8px; }}
        .desc {{ margin: 1em 0; padding: 1em; background: #fff; border-radius: 4px; border: 1px solid #eee; }}
        .nav {{ background: #fff; border: 1px solid #ddd; border-radius: 4px; padding: 1em; margin: 1.5em 0; }}
        .nav h3 {{ margin-bottom: 0.5em; }}
        .nav ul {{ list-style: none; columns: 2; }}
        .nav li {{ padding: 2px 0; }}
        .nav a {{ color: #1a5276; text-decoration: none; }}
        .nav a:hover {{ text-decoration: underline; }}
        .content p {{ margin: 0.5em 0; text-indent: 1.5em; }}
        .content p:first-of-type {{ text-indent: 0; }}
        hr {{ border: none; border-top: 1px solid #ddd; margin: 2em 0; }}
        .footer {{ text-align: center; color: #999; font-size: 0.85em; margin: 3em 0; }}
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
    <div class="desc">{desc_escaped}</div>
    <hr/>
    <div class="nav"><h3>Chapter Navigation</h3><ul>{nav}</ul></div>
    <hr/>
    <div class="content">{content}</div>
    <hr/>
    <div class="footer"><p>Generated by fICHub — {source}</p></div>
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

    // Write to file
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

## ZIP Compression

The HTML is compressed into a ZIP file:

```rust
let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

HTML compresses well — typically achieving 70-80% reduction in file size.

## EPUB vs HTML Bundle

| Feature | EPUB | HTML Bundle |
|---------|------|-------------|
| Format | EPUB (ZIP with XHTML) | ZIP with single HTML |
| Reader support | E-readers, Apple Books | Any web browser |
| Chapter navigation | Built-in TOC | Anchor links |
| Styling | CSS (limited by reader) | Full CSS control |
| File size | Smaller (no duplication) | Larger (all content in one file) |
| Offline reading | Yes | Yes (if extracted) |
| Search | Reader's search | Browser's search |
| Bookmarking | Reader's bookmarking | Browser's bookmarking |

## Watch Out!

**Large files!** For very long stories (100+ chapters), the HTML bundle can be several megabytes. The ZIP compression helps, but the file is still larger than an EPUB.

**CSS compatibility!** The CSS is designed for modern browsers. Older browsers might not render it correctly.

**Encoding!** The HTML uses UTF-8 encoding. Non-ASCII characters (common in fanfiction) are handled correctly.

## Summary

HTML bundles provide a universal format that works in any browser. They offer clean typography, chapter navigation, and responsive design in a self-contained package.

---

# Chapter 18: Disk Cache

## The Caching Strategy

FicHub generates EPUBs and HTML bundles on-demand, which is expensive. To avoid regenerating the same file multiple times, it uses a multi-layered caching strategy.

## Hash-Based Directory Structure

Cached files are stored in a hash-based directory structure:

```
cache/
├── epub/
│   └── a1b/
│       └── def/
│           └── a1b2c3d4e5f6/
│               └── abc123def456.epub
├── html/
│   └── a1b/
│       └── def/
│           └── a1b2c3d4e5f6/
│               └── 789xyz012abc.zip
```

The first three directory levels are chunks of the `url_id` (3 characters each). This spreads files across directories to avoid having too many files in a single folder.

The `cache_path` function computes this:

```rust
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    path = path.join(url_id);
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

## EType Enum

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
```

## Cache Semaphores

When multiple requests arrive for the same story simultaneously, FicHub uses semaphores to prevent duplicate generation:

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

Each `(url_id, etype)` pair gets its own semaphore with capacity 1.

## The Double-Check Pattern

```rust
// First check (fast — no semaphore needed)
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Acquire semaphore
let sem = cache::get_export_semaphore(&state.cache_semaphores, &url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Generate
let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await?;
```

## Moving Files to Cache

```rust
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}
```

`fs::rename` is atomic on most filesystems.

## Cache Validation

```rust
match crate::cache::disk::file_md5(&cache_path) {
    Ok(actual_hash) if actual_hash == hash => {
        // Serve the file
        let data = tokio::fs::read(&cache_path).await?;
        // ...
    }
    _ => {
        // Hash mismatch
        Json(json!({"err": -5, "msg": "hash mismatch"})).into_response()
    }
}
```

## File MD5 Computation

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

## Clearing Stale Cache

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

## Watch Out!

**Disk space management!** Cached files grow over time. Monitor disk usage and clean up old files if needed.

**Filesystem limits!** Some filesystems limit files per directory. The 3-level hash helps.

**Atomic moves!** If source and destination are on different filesystems, `fs::rename` falls back to copy-then-delete.

## Summary

FicHub's disk cache uses hash-based directories, semaphore-controlled generation, double-check locking, and MD5 validation for efficient, concurrent-safe caching.

---

# Chapter 19: Rate Limiting

## Why Rate Limit?

Rate limiting serves two purposes:
1. **Protect FicHub** — Prevent abuse and ensure fair usage
2. **Protect fanfiction sites** — Avoid overwhelming the sites we scrape

## Token Bucket Algorithm

A token bucket has two parameters:
- **Capacity** — Maximum tokens
- **Flow rate** — Tokens added per second

When a request comes in, it consumes one token. If no tokens are available, the request is delayed.

FicHub uses two buckets:
- **Global** — Capacity: 150, Flow: 30/sec
- **Per-IP** — Capacity: 30, Flow: 0.116/sec (~1 request every 8.6 seconds)

## The Redis Lua Script

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

Redis executes Lua scripts atomically, so there are no race conditions.

## The RedisBucketLimiter

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    global_capacity: f64,
    global_flow: f64,
    ip_capacity: f64,
    ip_flow: f64,
}
```

## Checking Rate Limits

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }

    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
        .await.unwrap_or(-1.0);

    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
        .await.unwrap_or(-1.0);

    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

## The RateLimiter Trait

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;
    async fn report_failure(&self, ip: IpAddr);
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}
```

## RateLimitResult Enum

```rust
pub enum RateLimitResult {
    Allowed,
    Wait(u64),   // Seconds to wait
    Blocked,     // Datacenter IP
}
```

## Penalties on Failure

```rust
async fn report_failure(&self, ip: IpAddr) {
    let _ = self.penalize("rate:global", self.global_capacity, self.global_flow).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, self.ip_capacity, self.ip_flow).await;
}
```

## Datacenter IP Blocking

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
}
```

## Watch Out!

**Rate limits are per-server!** For shared rate limiting across instances, you'd need a distributed solution.

**Lua scripts are atomic!** Redis ensures no race conditions in the token bucket logic.

## Summary

FicHub's rate limiter uses a token bucket algorithm in Redis Lua scripts, providing global and per-IP rate limiting with datacenter IP blocking and failure penalties.

---

# Chapter 20: Export Flow

## The Complete Journey

Let's trace the complete 17-step export flow.

## Step 1: HTTP Request

User sends `GET /api/v0/epub?q=https://archiveofourown.org/works/12345678`

## Step 2: Validate Query

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}
```

## Step 3: Check Automated Flag

```rust
if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
}
```

## Step 4: Find Scraper

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

## Step 5: Lookup Metadata

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
let info_request_ms = start.elapsed().as_millis() as i32;
```

## Step 6: Upsert Fic Info

```rust
queries::upsert_fic_info(&state.db, &fic_info_row).await?;
```

## Step 7: Auto-populate Tags

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(&state.db, &tag.name, tag.tag_type_id).await {
            let _ = queries::upsert_fic_tag(&state.db, &meta.url_id, resolution.tag_id, &ip).await;
        }
    }
}
```

## Step 8: Check Blacklists

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }
}
```

## Step 9: Compute Version

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
```

## Step 10: Check Cache (First)

```rust
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
```

## Step 11: Cache Hit Path

If cached, build response and return immediately.

## Step 12: Acquire Semaphore

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

## Step 13: Double-Check

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}
```

## Step 14: Fetch Chapters

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

## Step 15: Generate EPUB

```rust
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

## Step 16: Move to Cache

```rust
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

## Step 17: Generate HTML and Build Response

```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir).await?;
// ... move to cache, record ...

Ok(Json(json!({
    "err": 0,
    "q": query,
    "url_id": meta.url_id,
    "slug": generate_slug(&meta.title, &meta.url_id),
    "meta": build_meta_json(&meta),
    "epub_url": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
    "html_url": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
})))
```

## Performance Characteristics

- **Cache hit:** ~10ms
- **Concurrent generation:** ~100ms (waiting)
- **Full generation:** ~5-30 seconds

## Summary

The 17-step export flow handles caching, scraping, generation, and response building with efficient concurrency and cache validation.


---

# Part 5: Recommendations

---

# Chapter 21: Collaborative Filtering

## The Recommendation Problem

When you read a great fanfiction, you often want to find similar stories. FicHub's recommendation engine solves this using **collaborative filtering** — the same fundamental technique Netflix, Spotify, and Amazon use for their "recommended for you" features.

The core insight is simple but powerful: people who liked the same things you liked will probably like other things you'll like too. If Alice and Bob both bookmarked Story X, and Alice also bookmarked Story Y, then Story Y might interest Bob.

There are two main approaches to collaborative filtering:

1. **User-based** — Find users similar to you, recommend what they liked
2. **Item-based** — Find items similar to what you liked, recommend those

FicHub uses item-based collaborative filtering because it scales better and works well with the data available from fanfiction sites.

## The Jaccard Coefficient

The similarity between two stories is measured using the **Jaccard coefficient**, a standard metric in information retrieval:

```
Jaccard(A, B) = |A ∩ B| / |A ∪ B|
```

Where:
- A = set of users who bookmarked Story A
- B = set of users who bookmarked Story B
- |A ∩ B| = number of users who bookmarked both (intersection)
- |A ∪ B| = total number of unique users who bookmarked either (union)

### Example

Suppose:
- 100 users bookmarked Story A
- 80 users bookmarked Story B
- 60 users bookmarked both A and B

```
Jaccard = 60 / (100 + 80 - 60) = 60 / 120 = 0.5
```

A Jaccard of 0.5 means the stories are moderately similar — half of the combined audience overlaps.

### Properties of Jaccard

- Range: 0 to 1
- 0 = no overlap (completely different audiences)
- 1 = perfect overlap (same audience)
- Symmetric: Jaccard(A, B) = Jaccard(B, A)
- Doesn't consider absolute sizes, only overlap proportions

### Why Not Cosine Similarity?

Cosine similarity is another popular metric, but Jaccard is better for binary data (bookmarked or not). Cosine would give too much weight to stories with many bookmarkers, even if the overlap proportion is small.

## The Co-occurrence Matrix

FicHub stores co-occurrence data in the `fic_bookmark_cooccur` table:

```sql
CREATE TABLE fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

The `CHECK (work_a < work_b)` constraint ensures each pair is stored only once. This is a common optimization — instead of storing both (A, B) and (B, A), we only store one and canonicalize the order.

The `cooccur_count` is the number of users who bookmarked both stories. When a new user's favourites are processed, this count is incremented.

## Computing Recommendations

The recommendation query computes Jaccard coefficients for all stories that co-occur with the seed story:

```sql
WITH seed AS (
    SELECT favouriter_count FROM fic_works WHERE url_id = $1
),
candidates AS (
    SELECT
        CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
        cooccur_count
    FROM fic_bookmark_cooccur
    WHERE work_a = $1 OR work_b = $1
)
SELECT
    c.candidate_id,
    c.cooccur_count,
    fw.favouriter_count,
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

Let's break down this query:

1. **CTE `seed`** — Gets the seed story's favouriter count
2. **CTE `candidates`** — Finds all stories that co-occur with the seed, using CASE to normalize the order
3. **Main query** — Computes Jaccard = cooccur / (seed_count + candidate_count - cooccur)
4. **ORDER BY jaccard DESC** — Most similar first
5. **LIMIT** — Return top N results

## Tag Fallback

For new stories with few favouriters, collaborative filtering doesn't work well — there's simply not enough data. FicHub falls back to tag-based recommendations:

```rust
let min_collab = config.rec_min_favouriters_for_collab as i32;
let use_tag_fallback = favouriter_count < min_collab;

let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The `rec_min_favouriters_for_collab` config (default: 5) determines when to switch from tag-based to collaborative.

### Same Author (Strong Signal)

```rust
if !seed_author.is_empty() {
    let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
        r#"SELECT id, title, author, words, chapters, status, source, description
           FROM fic_info
           WHERE id != $1 AND author ILIKE $2
           ORDER BY words DESC
           LIMIT $3"#,
    )
    .bind(url_id)
    .bind(format!("%{}%", seed_author))
    .bind(limit as i64)
    .fetch_all(db)
    .await?;

    for row in rows {
        if seen.insert(row.id.clone()) {
            results.push((row.id, 0.8));  // Score: 0.8
        }
    }
}
```

Stories by the same author get a high base similarity score of 0.8.

### Title Keywords (Weaker Signal)

```rust
let keywords: Vec<&str> = seed_title
    .split_whitespace()
    .filter(|w| w.len() >= 3)
    .collect();

for kw in keywords {
    if results.len() >= limit { break; }
    let remaining = limit - results.len();
    let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
        r#"SELECT id, title, author, words, chapters, status, source, description
           FROM fic_info
           WHERE id != $1 AND title ILIKE $2
           ORDER BY words DESC
           LIMIT $3"#,
    )
    .bind(url_id)
    .bind(format!("%{}%", kw))
    .bind(remaining as i64)
    .fetch_all(db)
    .await?;

    for row in rows {
        if seen.insert(row.id.clone()) {
            results.push((row.id, 0.5));  // Score: 0.5
        }
    }
}
```

Stories with matching title keywords get a lower base score of 0.5.

## Blending Scores

FicHub smoothly blends collaborative and tag-based scores:

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

for c in &mut candidates {
    let collab = c.score;
    let tag = c.tag_score;
    c.score = collab * weight + tag * (1.0 - weight);
}
```

The weight transitions based on data availability:
- 0 favouriters → weight = 0 (pure tag-based)
- 1 favouriter → weight = 0.2
- 2 favouriters → weight = 0.4
- 3 favouriters → weight = 0.6
- 4 favouriters → weight = 0.8
- 5+ favouriters → weight = 1.0 (pure collaborative)

This ensures smooth degradation as data becomes sparse.

## Community Voting Boost

FicHub considers community votes on recommendations:

```rust
let gamma = config.rec_voting_boost_gamma;  // Default: 0.2
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The `ln_1p` function ensures logarithmic scaling:
- 0 votes → boost = 1.0 (no change)
- 1 vote → boost ≈ 1.14
- 5 votes → boost ≈ 1.32
- 10 votes → boost ≈ 1.46
- 50 votes → boost ≈ 1.78

This means the first few votes have a big impact, but additional votes have diminishing returns — exactly what we want.

## Caching Recommendations

Computed recommendations are cached in the `precomputed_recommendations` table:

```sql
CREATE TABLE precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

The engine first checks this cache:

```rust
let cached = check_cache(&self.db, query, config, limit).await?;
if !cached.is_empty() {
    return Ok(cached);
}

compute_live(&self.db, query, config, limit).await
```

### Cache Check Query

```sql
SELECT pr.recommended_url_id, pr.score,
       fi.title, fi.author, fi.words, fi.chapters,
       fi.status, fi.source, fi.description
FROM precomputed_recommendations pr
JOIN fic_info fi ON fi.id = pr.recommended_url_id
WHERE pr.url_id = $1
  AND pr.computed_at > NOW() - ($2 * INTERVAL '1 hour')
ORDER BY pr.rank ASC
LIMIT $3
```

The cache expires after `rec_cache_ttl_hours` (default: 12 hours).

### Cache Population

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
               VALUES ($1, $2, $3, $4, NOW())"#,
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

## Community Scores

The engine also fetches community scores for display:

```rust
async fn get_community_scores(db: &PgPool) -> Result<HashMap<String, i32>, AppError> {
    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           GROUP BY s.suggested_url_id"#,
    )
    .fetch_all(db)
    .await?;

    let mut map = HashMap::new();
    for (url_id, score) in rows {
        map.insert(url_id, score.unwrap_or(0));
    }
    Ok(map)
}
```

## Watch Out!

**Cold start problem!** New stories with no favouriters can't be recommended via collaborative filtering. The tag fallback helps but is less accurate.

**Popularity bias!** Stories with many favouriters dominate. Jaccard helps mitigate this by considering overlap proportion, not absolute counts.

**Site isolation!** FicHub can optionally filter recommendations by site domain, preventing cross-site recommendations that might not make sense.

## Summary

FicHub's recommendation engine uses collaborative filtering with Jaccard coefficients, tag fallback for new stories, community voting boosts, and a caching layer. The system smoothly transitions from tag-based to collaborative as more data becomes available.

---

# Chapter 22: Collection Worker

## The Background Worker

FicHub's recommendation engine needs data — specifically, it needs to know which users have bookmarked which stories. This data is collected by the **CollectionWorker**, a background task that scrapes user favourites from fanfiction sites.

## The SiteFetcher Trait

Each fanfiction site implements the `SiteFetcher` trait:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    fn site_domain(&self) -> &str;

    fn rate_limit_delay(&self, config: &Config, domain: &str) -> u64 {
        config.rec_site_rate_limits.get(domain).copied()
            .unwrap_or(config.rec_default_delay_secs)
    }

    async fn collect_favouriters(
        &self, client: &Client, work_url: &str, max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    async fn collect_user_favourites(
        &self, client: &Client, user_url: &str, max_pages: u32,
    ) -> Result<Vec<String>, ScrapeError>;

    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

- `site_domain` — Canonical domain for the site
- `rate_limit_delay` — Per-site delay between requests
- `collect_favouriters` — Find users who bookmarked a story
- `collect_user_favourites` — Find all stories a user bookmarked
- `user_hash` — Create a deterministic, anonymous hash of a user's profile URL

The `user_hash` method uses SHA-256 to create a one-way hash. This preserves anonymity while allowing us to track user preferences.

## The Redis Queue

The worker uses Redis as a job queue:

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

Jobs are pushed with `LPUSH` and popped with `LPOP`.

## The Per-Site Rate Limiter

Each site has its own rate limiter using an atomic CAS loop:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

impl PerSiteRateLimiter {
    pub fn new(delay_secs: u64) -> Self {
        Self {
            last_request: AtomicI64::new(0),
            delay_secs,
        }
    }

    pub async fn wait_if_needed(&self) {
        let delay_nanos = (self.delay_secs as u64) * 1_000_000_000;

        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as i64;

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

The CAS (compare-and-swap) loop atomically claims a time slot. Multiple concurrent callers are serialized so each waits the full delay.

## The Worker Loop

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
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }

                match serde_json::from_str::<QueueItem>(&item_str) {
                    Ok(item) => {
                        if let Err(e) = self.process_work(item).await {
                            error!("Error processing work: {}", e);
                        }
                    }
                    Err(e) => warn!("Invalid queue item: {}", e),
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

## Processing a Work

The `process_work` method is the core collection procedure:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = self.fetchers.iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or("No fetcher for domain")?;

    let work_url = format!("https://{}/works/{}", item.site_domain, item.site_work_id);

    // 1. Fetch favouriters
    let favouriters = fetcher.collect_favouriters(
        &self.client, &work_url, self.config.rec_max_favourite_pages
    ).await?;

    let mut new_user_count: u32 = 0;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // 2. Skip if already recorded
        let already_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)"
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .fetch_one(&self.db)
        .await?;

        if already_exists { continue; }
        new_user_count += 1;

        // 3. Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        )
        .bind(&user_hash)
        .bind(&item.url_id)
        .bind(&item.site_domain)
        .execute(&self.db)
        .await?;

        // 4. Fetch user's favourites
        let user_favs = fetcher.collect_user_favourites(
            &self.client, user_url, self.config.rec_max_user_favourite_pages
        ).await?;

        // 5. Update co-occurrence
        for other_url_id in &user_favs {
            if other_url_id == &item.url_id { continue; }
            let (a, b) = if item.url_id < *other_url_id {
                (&item.url_id, other_url_id)
            } else {
                (other_url_id, &item.url_id)
            };

            sqlx::query(
                r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, site_domain, cooccur_count)
                   VALUES ($1, $2, $3, 1)
                   ON CONFLICT (work_a, work_b) DO UPDATE SET
                       cooccur_count = fic_bookmark_cooccur.cooccur_count + 1,
                       last_updated = NOW()"#,
            )
            .bind(a)
            .bind(b)
            .bind(&item.site_domain)
            .execute(&self.db)
            .await?;
        }
    }

    // 6. Update favouriter count
    sqlx::query(
        r#"INSERT INTO fic_works (url_id, site_domain, site_work_id, favouriter_count,
                                   first_favourite_scraped, last_favourite_scraped)
           VALUES ($1, $2, $3, $4, NOW(), NOW())
           ON CONFLICT (url_id) DO UPDATE SET
               favouriter_count = fic_works.favouriter_count + 1,
               last_favourite_scraped = NOW()"#,
    )
    .bind(&item.url_id)
    .bind(&item.site_domain)
    .bind(&item.site_work_id)
    .bind(favouriters.len() as i32)
    .execute(&self.db)
    .await?;

    info!("Processed {} from {}: {} favouriters, {} new users",
        item.url_id, item.site_domain, favouriters.len(), new_user_count);

    Ok(())
}
```

## Stub SiteFetcher Implementations

Currently, FicHub has stub implementations for all sites:

```rust
struct Ao3Fetcher;
struct FfNetFetcher;
struct XenForoFetcher;
struct FictionPressFetcher;
struct AdultFanFictionFetcher;
struct HpFanFicFetcher;

#[async_trait]
impl SiteFetcher for Ao3Fetcher {
    fn site_domain(&self) -> &str { "archiveofourown.org" }

    async fn collect_favouriters(&self, _client: &Client, _work_url: &str, _max_pages: u32)
        -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())  // Stub — implement later
    }

    async fn collect_user_favourites(&self, _client: &Client, _user_url: &str, _max_pages: u32)
        -> Result<Vec<String>, ScrapeError> {
        Ok(Vec::new())  // Stub — implement later
    }
}
```

The architecture is the focus — real scraping logic can be added per-site without changing the worker.

## Watch Out!

**The worker is CPU and network intensive!** Run it on a separate thread to avoid blocking the main server.

**Rate limits are essential!** Without per-site rate limiting, the worker could get blocked.

**Privacy matters!** User hashes are one-way — you can't reverse them.

## Summary

The CollectionWorker is a background task that scrapes user favourites, populates the co-occurrence matrix, and updates favouriter counts. It uses Redis queues, per-site rate limiters, and atomic CAS loops.

---

# Chapter 23: Community Suggestions

## Beyond Algorithmic Recommendations

While collaborative filtering is powerful, it can't capture everything. Sometimes a human reader knows that two stories are similar in ways algorithms can't detect — maybe they share a specific plot device, or they're both part of the same crossover universe.

## The Suggestion Table

```sql
CREATE TABLE recommendation_suggestions (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    suggested_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    submitted_by_ip INET NOT NULL,
    comment TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, suggested_url_id, submitted_by_ip)
);
```

The `UNIQUE` constraint ensures each user can only suggest a specific pair once.

## Submitting a Suggestion

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    if body.url_id.is_empty() || body.suggested_url.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id and suggested_url required"})));
    }

    // Resolve the suggested URL
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
        return Ok(Json(json!({"err": -5, "msg": "seed fic not found — enqueued"})));
    }

    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "suggested fic not collected — enqueued"})));
    }

    // Rate limit check
    check_tag_rate_limit(&mut redis, "suggest", ip, state.config.rec_suggest_limit_per_hour).await?;

    // Submit
    let suggestion_id = submit_suggestion(
        &state.db, &body.url_id, &suggested_url_id, "0.0.0.0", body.comment.as_deref(),
    ).await?;

    Ok(Json(json!({"err": 0, "suggestion_id": suggestion_id})))
}
```

## The submit_suggestion Function

```rust
pub async fn submit_suggestion(
    pool: &PgPool,
    url_id: &str,
    suggested_url_id: &str,
    submitted_by_ip: &str,
    comment: Option<&str>,
) -> AppResult<i64> {
    let row: (i64,) = sqlx::query_as(
        r#"INSERT INTO recommendation_suggestions
           (url_id, suggested_url_id, submitted_by_ip, comment)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, suggested_url_id, submitted_by_ip) DO NOTHING
           RETURNING id"#,
    )
    .bind(url_id)
    .bind(suggested_url_id)
    .bind(submitted_by_ip)
    .bind(comment)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
```

## Rate Limiting

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

The rate limit key has a 1-hour TTL, so the counter resets automatically.

## Watch Out!

**IP-based identification is imperfect!** Multiple users behind the same NAT share an IP.

**Suggestions require both fics in the database!** Missing fics are enqueued for background collection.

## Summary

Community suggestions let users contribute to the recommendation system. The system validates inputs, rate-limits submissions, and enqueues missing stories.

---

# Chapter 24: Voting

## The Vote Table

```sql
CREATE TABLE recommendation_votes (
    suggestion_id BIGINT NOT NULL REFERENCES recommendation_suggestions(id) ON DELETE CASCADE,
    voter_ip INET NOT NULL,
    vote SMALLINT NOT NULL CHECK (vote IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (suggestion_id, voter_ip)
);
```

## Casting a Vote

```rust
pub async fn cast_vote(
    pool: &PgPool,
    suggestion_id: i64,
    voter_ip: &str,
    value: i16,
) -> AppResult<i32> {
    sqlx::query(
        r#"INSERT INTO recommendation_votes (suggestion_id, voter_ip, vote)
           VALUES ($1, $2::inet, $3)
           ON CONFLICT (suggestion_id, voter_ip) DO UPDATE SET vote = EXCLUDED.vote"#,
    )
    .bind(suggestion_id)
    .bind(voter_ip)
    .bind(value)
    .execute(pool)
    .await?;

    let row: (i32,) = sqlx::query_as(
        "SELECT COALESCE(SUM(vote)::INT, 0) FROM recommendation_votes WHERE suggestion_id = $1"
    )
    .bind(suggestion_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
```

The `ON CONFLICT DO UPDATE` lets users change their vote.

## Net Score

```sql
SELECT COALESCE(SUM(vote)::INT, 0)
FROM recommendation_votes
WHERE suggestion_id = $1
```

## Community Scores in Recommendations

```rust
let gamma = config.rec_voting_boost_gamma;
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    c.community_score = nv;
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

## Getting Community Suggestions

```rust
pub async fn get_community_suggestions(
    db: &PgPool,
    url_id: &str,
) -> AppResult<Vec<Suggestion>> {
    let rows: Vec<SuggestionRow> = sqlx::query_as::<_, SuggestionRow>(
        r#"SELECT s.id, s.suggested_url_id, s.comment,
                  COALESCE(SUM(v.vote)::INT, 0) AS net_votes,
                  s.created
           FROM recommendation_suggestions s
           LEFT JOIN recommendation_votes v ON v.suggestion_id = s.id
           WHERE s.url_id = $1
           GROUP BY s.id, s.suggested_url_id, s.comment, s.created
           ORDER BY net_votes DESC"#,
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

## Vote Validation

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

## Watch Out!

**Single vote per user!** The PRIMARY KEY ensures each IP can only vote once per suggestion.

**No anonymous voting without accounts!** IP-based voting is the current approach.

## Summary

FicHub's voting system lets users upvote or downvote community suggestions with optimistic voting, net score computation, and logarithmic boosting for the recommendation engine.


---

# Part 6: Server Deployment

---

# Chapter 25: Full Router

## Every Route in FicHub

FicHub has 40+ routes organized by subsystem. This chapter catalogs every route, explains its purpose, and shows the handler signature.

## Core API Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/` | `api_docs_handler` | API documentation |
| GET | `/api/v0/epub` | `epub_handler` | Main export endpoint |
| GET | `/api/v0/meta` | `meta_handler` | Metadata-only endpoint |
| GET | `/api/v0/remote` | `remote_handler` | Client IP info |

### `/api/` — API Documentation

Returns a JSON object describing all available endpoints:

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links"
            },
            "/api/v0/meta": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata only"
            },
            "/api/v0/remote": {
                "method": "GET",
                "description": "Get request source information"
            },
            "/cache/:etype/:url_id": {
                "method": "GET",
                "params": { "h": "MD5 hash for validation" },
                "description": "Download cached export file"
            }
        }
    }))
}
```

### `/api/v0/epub` — Main Export

The most important endpoint. Accepts a fanfiction URL and returns metadata plus download links:

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}

pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    // 17-step export flow (see Chapter 20)
}
```

### `/api/v0/meta` — Metadata Only

Same as `/api/v0/epub` but without generating or serving files:

```rust
#[derive(Debug, Deserialize)]
pub struct MetaQuery {
    pub q: Option<String>,
}

pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "q": query,
        "url_id": meta.url_id,
        "slug": generate_slug(&meta.title, &meta.url_id),
        "meta": build_meta_json(&meta),
        "hashes": {},
        "urls": {},
    })))
}
```

### `/api/v0/remote` — Client IP Info

Returns the client's IP address:

```rust
async fn remote_handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<serde_json::Value> {
    Json(json!({
        "ip": remote.ip().to_string(),
        "port": remote.port(),
        "is_automated": false,
    }))
}
```

## Cache Download Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/cache/{etype}/{url_id}/{fname}` | `download_with_hash` | Download with hash validation |
| GET | `/cache/{etype}/{url_id}` | `download_or_export` | Download or trigger export |

### Download with Hash Validation

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let etype = match etype_str.parse::<EType>() {
        Ok(e) => e,
        Err(_) => return Json(json!({"err": -1, "msg": "invalid format"})).into_response(),
    };

    let hash = match params.h {
        Some(h) => h,
        None => {
            let stem = fname.trim_end_matches(etype.suffix());
            stem.to_string()
        }
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash,
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
            };

            match tokio::fs::read(&cache_path).await {
                Ok(data) => {
                    let filename = format!("{}{}", url_id, etype.suffix());
                    let headers = [
                        ("Content-Type", mime),
                        ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                    ];
                    (headers, data).into_response()
                }
                Err(_) => Json(json!({"err": -1, "msg": "read error"})).into_response(),
            }
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

### Download or Export

```rust
pub async fn download_or_export(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id)): Path<(String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    // If hash provided and file exists, serve directly
    if let Some(ref hash) = params.h {
        if let Ok(etype) = etype_str.parse::<EType>() {
            let cache_path = crate::cache::disk::cache_path(
                &state.config.cache_dir, &etype, &url_id, hash,
            );
            if cache_path.exists() {
                // ... serve file ...
            }
        }
    }

    // No cached file — redirect to frontend
    Redirect::to(&format!("/?id={}", url_id)).into_response()
}
```

## Recommender Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/v0/recommendations` | `recommendations_handler` | Get recommendations |
| POST | `/api/v0/recommendations/suggest` | `suggest_handler` | Submit a suggestion |
| POST | `/api/v0/recommendations/vote` | `vote_handler` | Vote on a suggestion |
| GET | `/api/v0/recommendations/votes` | `votes_handler` | List votes |

### Recommendations Handler

```rust
pub async fn recommendations_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecQueryParams>,
) -> Result<Json<Value>, AppError> {
    let (url_id, _seed_meta) = if let Some(q) = &params.q {
        let scraper = state.scraper_registry.find_scraper(q)
            .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", q)))?;
        let meta = scraper.lookup(&state.http_client, q).await
            .map_err(|e| AppError::ScrapeError(e.to_string()))?;
        (meta.url_id, Some(meta))
    } else if let Some(id) = &params.url_id {
        (id.clone(), None)
    } else {
        return Ok(Json(json!({"err": -1, "msg": "no query or url_id"})));
    };

    let n = params.n.unwrap_or(20).max(1).min(100) as usize;

    let rec_query = RecQuery {
        url_id: url_id.clone(),
        n,
        site_domain: params.site_domain.clone(),
    };

    let recommendations = state.recommender_engine
        .get_recommendations(&rec_query, &state.config).await?;

    Ok(Json(json!({
        "err": 0,
        "url_id": url_id,
        "recommendations": recommendations,
    })))
}
```

## Tag Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/v0/tags/submit` | `submit_tag` | Submit a tag |
| POST | `/api/v0/tags/vote` | `vote_tag` | Vote on a tag |
| POST | `/api/v0/tags/flag` | `flag_tag` | Flag a tag |
| GET | `/api/v0/tags` | `get_tags` | Get tags for a fic |

### Submit Tag

```rust
pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<Value>, AppError> {
    let ip = remote.ip();

    // Validate
    if body.url_id.is_empty() || body.tag_name.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "url_id and tag_name required"})));
    }
    if !(1..=7).contains(&body.tag_type_id) {
        return Ok(Json(json!({"err": -1, "msg": "tag_type_id must be 1-7"})));
    }

    // Rate limit
    check_tag_rate_limit(&mut redis, "submit", ip, state.config.tag_submit_limit_per_hour).await?;

    // Verify fic exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fic_info WHERE id = $1)"
    ).bind(&body.url_id).fetch_one(&state.db).await?;

    if !exists {
        return Ok(Json(json!({"err": -5, "msg": "fic not found"})));
    }

    // Resolve tag
    let resolution = resolve::resolve_tag(&state.db, &body.tag_name, body.tag_type_id).await?;

    // Attach tag (idempotent)
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           VALUES ($1, $2, $3::inet, 0)
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    ).bind(&body.url_id).bind(resolution.tag_id).bind(&ip.to_string())
     .execute(&state.db).await?;

    Ok(Json(json!({
        "err": 0,
        "tag_id": resolution.tag_id,
        "tag_name": resolution.tag_name,
        "is_new": resolution.is_new,
    })))
}
```

## Curator Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| POST | `/api/v0/curator/alias` | `create_alias` | Create a tag alias |
| POST | `/api/v0/curator/merge` | `merge_tags` | Merge two tags |
| DELETE | `/api/v0/curator/tags/{id}` | `delete_tag` | Delete a tag |
| GET | `/api/v0/curator/flags` | `list_flags` | List unresolved flags |
| POST | `/api/v0/curator/flags/{id}/resolve` | `resolve_flag` | Resolve a flag |

All curator endpoints require Bearer token authentication.

## Search Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/api/v0/search` | `search_handler` | Full-text search |

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let search_params = params.into_search_params()?;
    let builder = SearchQueryBuilder::new(capped_params, state.config.tag_hidden_threshold);

    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query.build_query_as().fetch_one(&state.db).await?;

    let mut data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query.build_query_as().fetch_all(&state.db).await?;

    // Batch-fetch tags
    let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let tag_rows = sqlx::query_as::<_, TagDbRow>(
        "SELECT ft.url_id, t.name, t.tag_type_id, tt.name AS type_name, ft.score
         FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id JOIN tag_types tt ON tt.id = t.tag_type_id
         WHERE ft.url_id = ANY($1) AND ft.score >= $2"
    ).bind(&url_ids).bind(state.config.tag_hidden_threshold)
     .fetch_all(&state.db).await?;

    // Build response
    Ok(Json(json!({
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": results,
    })))
}
```

## OPDS Routes

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/opds` | `root_catalog` | Root catalog |
| GET | `/opds/new` | `recent_feed` | Recent fics |
| GET | `/opds/popular` | `popular_feed` | Popular fics |
| GET | `/opds/tags` | `tag_types` | Browse by tag type |
| GET | `/opds/tags/{type_id}` | `tags_by_type` | Tags in a type |
| GET | `/opds/tags/{type_id}/{tag_name}` | `fics_by_tag` | Fics with a tag |
| GET | `/opds/authors` | `author_list` | Browse by author |
| GET | `/opds/recommendations/popular` | `popular_recommendations` | Popular recs |
| GET | `/opds/recommendations` | `fic_recommendations` | Fic-specific recs |
| GET | `/opds/search` | `search_feed` | Search feed |
| GET | `/opds/shelves` | `shelf_list` | List shelves |
| GET | `/opds/shelf/{shelf_id}` | `shelf_contents` | Shelf contents |

All OPDS routes return Atom XML feeds for e-readers.

## Legacy Redirects

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| GET | `/legacy/epub_export` | `redirect_to_root` | Redirect to / |
| GET | `/fic/{url_id}` | `redirect_to_root` | Redirect to / |
| GET | `/changes` | `redirect_to_root` | Redirect to / |
| GET | `/popular/` | `redirect_to_root` | Redirect to / |

## AppState

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

Every handler receives `State(state): State<Arc<AppState>>` and accesses shared resources through it.

## Summary

FicHub's router is well-organized with 40+ routes across 8 subsystems. The AppState struct centralizes shared state, and each handler follows a consistent pattern of extract → validate → process → respond.

---

# Chapter 26: Static Files

## Serving the Frontend

FicHub's frontend is a SvelteKit application built into static files. These are served by tower-http's `ServeDir`:

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

## How ServeDir Works

`ServeDir` serves files from a directory based on the request path:

1. Request for `/` → serves `/index.html`
2. Request for `/assets/main.js` → serves the JS file
3. Request for `/unknown/path` → falls through to fallback (index.html)

## SPA Routing

SvelteKit builds to static files:

```
frontend/build/
├── index.html
├── _app/
│   └── immutable/
│       ├── assets/
│       │   ├── main-abc123.js
│       │   └── main-def456.css
│       └── chunks/
│           └── 0-ghi789.js
└── favicon.png
```

When a user visits `/fic/abc123`:
1. ServeDir looks for `frontend/build/fic/abc123` — not found
2. Falls through to `frontend/build/index.html`
3. SvelteKit boots and handles the route client-side

## Cache Headers

```rust
ServeDir::new(&frontend_dir)
    .precompressed_br()
    .precompressed_gzip()
    .append_index_html_on_directories(true)
```

For production, add cache headers via middleware:

```rust
.layer(CacheControl::new()
    .set_max_age(Duration::from_secs(3600))
    .set_public())
```

## MIME Types

| Extension | MIME Type |
|-----------|-----------|
| `.html` | `text/html` |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff2` | `font/woff2` |

## Watch Out!

**Order matters!** The fallback must come after all API routes.

**Asset paths must match!** The build output paths must match what the HTML expects.

## Summary

FicHub uses `ServeDir` with SPA fallback to serve the SvelteKit frontend. The configuration ensures proper routing and fallback behavior.

---

# Chapter 27: Docker

## Multi-Stage Build

```dockerfile
FROM rust:1.78-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/fichub .
COPY --from=builder /app/migrations migrations/
EXPOSE 3000
CMD ["./fichub"]
```

Stage 1 compiles, Stage 2 runs. The runtime image is ~50-100MB vs ~1GB+ for build.

## Docker Compose

```yaml
version: '3.8'
services:
  fichub:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://fichub:fichub@db/fichub
      - REDIS_URL=redis://redis:6379
      - CACHE_DIR=/data/cache
    volumes:
      - cache_data:/data/cache
    depends_on:
      - db
      - redis

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=fichub
      - POSTGRES_PASSWORD=fichub
      - POSTGRES_DB=fichub
    volumes:
      - pg_data:/var/lib/postgres/data

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data

volumes:
  cache_data:
  pg_data:
  redis_data:
```

## Building and Running

```bash
docker compose build
docker compose up -d
docker compose logs -f fichub
docker compose down
```

## Environment Variables

Use a `.env` file:

```bash
DATABASE_URL=postgres://fichub:fichub@db/fichub
REDIS_URL=redis://redis:6379
CACHE_DIR=/data/cache
PORT=3000
RUST_LOG=info,fichub=debug
```

## Watch Out!

**Database readiness!** Make sure PostgreSQL is ready before FicHub starts. Use health checks or retry logic.

**Volume permissions!** The container runs as root. Ensure host volume permissions are correct.

**Redis persistence!** Without the `redis_data` volume, Redis data is lost on container stop.

## Summary

FicHub's Docker setup uses multi-stage builds, Docker Compose for orchestration, and volumes for persistence.

---

# Chapter 28: Cross-Compile

## Setting Up

```bash
rustup target add aarch64-unknown-linux-gnu
sudo pacman -S aarch64-linux-gnu-gcc  # Arch
# or: sudo apt install gcc-aarch64-linux-gnu  # Ubuntu
```

Create `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Building

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

## Deploying

```bash
rsync -avz target/aarch64-unknown-linux-gnu/release/fichub user@server:~/fichub/
rsync -avz migrations/ user@server:~/fichub/migrations/
rsync -avz frontend/build/ user@server:~/fichub/frontend/build/
```

## Systemd Service

```ini
[Unit]
Description=FicHub Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/home/fichub
Environment=DATABASE_URL=postgres://fichub:fichub@localhost/fichub
Environment=REDIS_URL=redis://localhost:6379
Environment=CACHE_DIR=/home/fichub/cache
ExecStart=/home/fichub/fichub
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Nginx Reverse Proxy

```nginx
server {
    listen 80;
    server_name fichub.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location /assets/ {
        proxy_pass http://127.0.0.1:3000;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    location /cache/ {
        proxy_pass http://127.0.0.1:3000;
        expires 30d;
        add_header Cache-Control "public";
    }
}
```

## TLS with Let's Encrypt

```bash
sudo certbot --nginx -d fichub.example.com
```

## Watch Out!

**Cross-compilation requires matching libraries!** FicHub uses `rustls` to avoid OpenSSL dependency issues.

**Test on the target device!** Always verify the binary works on the actual hardware.

## Summary

Cross-compilation enables deployment on ARM64 devices. Combined with rsync, systemd, and nginx, FicHub can run on anything from a Raspberry Pi to a cloud server.


---

# Part 7: Advanced Features

---

# Chapter 29: Search System

## Full-Text Search

FicHub's search system goes beyond simple string matching. It uses PostgreSQL's full-text search capabilities with weighted ranking, complex filters, and tag-based queries.

## The text_search Column

The `text_search` column is a generated tsvector:

```sql
ALTER TABLE fic_info ADD COLUMN IF NOT EXISTS text_search tsvector
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title,'')), 'A') ||
        setweight(to_tsvector('english', coalesce(description,'')), 'B')
    ) STORED;
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

**Weights:**
- A (title) — Higher relevance
- B (description) — Lower relevance

The `to_tsvector` function tokenizes text and applies English stemming ("running" → "run"). The GIN index makes queries fast.

## Search Parameters

```rust
pub struct SearchParams {
    pub q: Option<String>,              // Full-text search query
    pub include_tags: Vec<TagFilter>,   // ALL must match
    pub exclude_tags: Vec<TagFilter>,   // NONE must match
    pub include_any_tags: Vec<TagFilter>, // At least ONE
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,
    pub source: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
```

## The SearchQueryBuilder

```rust
pub struct SearchQueryBuilder {
    pub params: SearchParams,
    hidden_threshold: i16,
}
```

### Building the Count Query

```rust
pub fn build_count_query(&self) -> QueryBuilder<Postgres> {
    let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM fic_info fi WHERE 1=1");
    self.push_where_clauses(&mut qb);
    qb
}
```

### Building the Data Query

```rust
pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
    let has_q = self.params.q.is_some();
    let mut qb = QueryBuilder::new("SELECT fi.*, ");

    if let Some(ref q) = self.params.q {
        qb.push("ts_rank(fi.text_search, plainto_tsquery('english', ");
        qb.push_bind(q);
        qb.push(")) AS rank FROM fic_info fi WHERE 1=1");
    } else {
        qb.push("NULL::real AS rank FROM fic_info fi WHERE 1=1");
    }

    self.push_where_clauses(&mut qb);

    // ORDER BY
    let sort = self.params.sort.as_deref().unwrap_or(
        if has_q { "-relevance" } else { "-date" }
    );
    qb.push(" ORDER BY ");
    match sort {
        "-relevance" if has_q => qb.push("rank DESC"),
        "-date" => qb.push("fi.fic_updated DESC"),
        "-words" => qb.push("fi.words DESC"),
        "-chapters" => qb.push("fi.chapters DESC"),
        "-title" => qb.push("fi.title ASC"),
        _ => qb.push("fi.fic_updated DESC"),
    }

    // LIMIT / OFFSET
    let page = self.params.page.unwrap_or(1).max(1);
    let per_page = self.params.per_page.unwrap_or(20).max(1);
    let offset = (page - 1) * per_page;
    qb.push(" LIMIT ");
    qb.push_bind(per_page as i64);
    qb.push(" OFFSET ");
    qb.push_bind(offset as i64);

    qb
}
```

### WHERE Clauses

```rust
fn push_where_clauses(&self, qb: &mut QueryBuilder<Postgres>) {
    // Full-text search
    if let Some(ref q) = self.params.q {
        qb.push(" AND fi.text_search @@ plainto_tsquery('english', ");
        qb.push_bind(q);
        qb.push(")");
    }

    // include_tags — ALL must match
    for tag in &self.params.include_tags {
        qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND t.name = ");
        qb.push_bind(&tag.tag_name);
        qb.push(" AND t.tag_type_id = ");
        qb.push_bind(tag.tag_type_id);
        qb.push(" AND ft.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(")");
    }

    // exclude_tags — NONE must match
    for tag in &self.params.exclude_tags {
        qb.push(" AND NOT EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND t.name = ");
        qb.push_bind(&tag.tag_name);
        qb.push(" AND t.tag_type_id = ");
        qb.push_bind(tag.tag_type_id);
        qb.push(")");
    }

    // include_any_tags — at least ONE must match
    if !self.params.include_any_tags.is_empty() {
        qb.push(" AND EXISTS (SELECT 1 FROM fic_tags ft");
        qb.push(" JOIN tags t ON t.id = ft.tag_id");
        qb.push(" WHERE ft.url_id = fi.id");
        qb.push(" AND ft.score >= ");
        qb.push_bind(self.hidden_threshold);
        qb.push(" AND (");
        let mut first = true;
        for tag in &self.params.include_any_tags {
            if !first { qb.push(" OR"); }
            first = false;
            qb.push(" (t.name = ");
            qb.push_bind(&tag.tag_name);
            qb.push(" AND t.tag_type_id = ");
            qb.push_bind(tag.tag_type_id);
            qb.push(")");
        }
        qb.push("))");
    }

    // Word count range
    if let Some(min) = self.params.min_words {
        qb.push(" AND fi.words >= ");
        qb.push_bind(min);
    }
    if let Some(max) = self.params.max_words {
        qb.push(" AND fi.words <= ");
        qb.push_bind(max);
    }

    // Chapter count range
    if let Some(min) = self.params.min_chapters {
        qb.push(" AND fi.chapters >= ");
        qb.push_bind(min);
    }
    if let Some(max) = self.params.max_chapters {
        qb.push(" AND fi.chapters <= ");
        qb.push_bind(max);
    }

    // Completion status
    if let Some(true) = self.params.complete {
        qb.push(" AND fi.status = 'complete'");
    }
    if let Some(false) = self.params.complete {
        qb.push(" AND fi.status != 'complete'");
    }

    // Source filter
    if let Some(ref source) = self.params.source {
        qb.push(" AND fi.source = ");
        qb.push_bind(source);
    }

    // Date range
    if let Some(ref dt) = self.params.date_from {
        qb.push(" AND fi.fic_updated >= ");
        qb.push_bind(dt);
    }
    if let Some(ref dt) = self.params.date_to {
        qb.push(" AND fi.fic_updated <= ");
        qb.push_bind(dt);
    }
}
```

## Tag Filter Parsing

```rust
pub fn parse_tag_filters(input: &str) -> Result<Vec<TagFilter>, String> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let mut filters = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        let colon_pos = part.find(':').ok_or_else(|| {
            format!("Invalid format '{}': expected 'type_id:name'", part)
        })?;
        let type_id: i16 = part[..colon_pos].parse()
            .map_err(|e| format!("Invalid type_id in '{}': {}", part, e))?;
        let name = part[colon_pos + 1..].to_string();
        filters.push(TagFilter { tag_type_id: type_id, tag_name: name });
    }
    Ok(filters)
}
```

Format: `1:Harry Potter,2:Hermione Granger,4:Angst`

## The Search Handler

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let search_params = params.into_search_params()?;
    let per_page = search_params.per_page.unwrap_or(20)
        .min(state.config.search_max_per_page).max(1);

    let builder = SearchQueryBuilder::new(capped_params, state.config.tag_hidden_threshold);

    // Count query
    let mut count_query = builder.build_count_query();
    let total: (i64,) = count_query.build_query_as().fetch_one(&state.db).await?;

    // Data query
    let mut data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query.build_query_as().fetch_all(&state.db).await?;

    // Batch-fetch tags
    let url_ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let tag_rows = sqlx::query_as::<_, TagDbRow>(
        r#"SELECT ft.url_id, t.name, t.tag_type_id, tt.name AS type_name, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           JOIN tag_types tt ON tt.id = t.tag_type_id
           WHERE ft.url_id = ANY($1) AND ft.score >= $2
           ORDER BY ft.url_id, ft.score DESC"#,
    ).bind(&url_ids).bind(state.config.tag_hidden_threshold)
     .fetch_all(&state.db).await?;

    // Group tags by url_id
    let mut tags_by_fic: HashMap<String, Vec<TagDbRow>> = HashMap::new();
    for tag_row in tag_rows {
        tags_by_fic.entry(tag_row.url_id.clone()).or_default().push(tag_row);
    }

    // Build results
    let results: Vec<SearchResultData> = rows.into_iter().map(|row| {
        let fic_tags = tags_by_fic.remove(&row.id).unwrap_or_default();
        let response_tags: Vec<Value> = fic_tags.into_iter().map(|t| {
            json!({
                "name": t.name,
                "type": t.type_name,
                "type_id": t.tag_type_id,
                "score": t.score,
            })
        }).collect();
        SearchResultData {
            url_id: row.id,
            title: row.title,
            author: row.author,
            words: row.words,
            chapters: row.chapters,
            status: row.status,
            description: row.description,
            rank: row.rank,
            tags: response_tags,
            // ...
        }
    }).collect();

    Ok(Json(json!({
        "total": total,
        "page": page,
        "per_page": per_page,
        "results": results,
    })))
}
```

## Watch Out!

**Search is case-insensitive!** `plainto_tsquery` handles case automatically.

**Tag filters use the hidden threshold!** Tags below the threshold are excluded.

**Pagination is essential!** Without LIMIT/OFFSET, large result sets would be slow.

## Summary

FicHub's search uses PostgreSQL full-text search with weighted ranking, dynamic query building with tag filters, batch tag fetching, and proper pagination.

---

# Chapter 30: Tag System

## Tag Types

| ID | Name | Example |
|----|------|---------|
| 1 | fandom | Harry Potter |
| 2 | character | Hermione Granger |
| 3 | relationship | Harry/Hermione |
| 4 | freeform | Angst, Fluff |
| 5 | warning | Major Character Death |
| 6 | category | M/M, F/M |
| 7 | other | Alternate Universe |

## Tag Resolution

```rust
pub async fn resolve_tag(pool: &PgPool, tag_name: &str, tag_type_id: i16) -> AppResult<TagResolution> {
    // 1. Exact match
    if let Some(row) = lookup_tag(pool, tag_name).await? {
        return Ok(TagResolution { tag_id: row.0, tag_name: row.1, tag_type_id: row.2, is_new: false });
    }

    // 2. Alias match
    if let Some(canonical_id) = lookup_alias(pool, tag_name).await? {
        let tag = lookup_tag_by_id(pool, canonical_id).await?;
        return Ok(TagResolution { tag_id: canonical_id, tag_name: tag.1, tag_type_id: tag.2, is_new: false });
    }

    // 3. Create new
    let new_id = create_tag(pool, tag_name, tag_type_id).await?;
    Ok(TagResolution { tag_id: new_id, tag_name: tag_name.to_string(), tag_type_id, is_new: true })
}
```

## Score Management via Triggers

```sql
CREATE OR REPLACE FUNCTION update_fic_tag_score()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE fic_tags SET score = score + NEW.value WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' AND NEW.value <> OLD.value THEN
        UPDATE fic_tags SET score = score - OLD.value + NEW.value WHERE url_id = NEW.url_id AND tag_id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE fic_tags SET score = score - OLD.value WHERE url_id = OLD.url_id AND tag_id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

Three triggers fire on INSERT, UPDATE, and DELETE on `fic_tag_votes`.

## Visibility

```rust
pub fn is_hidden(score: i16, threshold: i16) -> bool {
    score <= threshold
}
```

Default threshold: -3. Tags need 3+ net downvotes to be hidden.

## Curator Tools

### Create Alias

```rust
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateAliasBody>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    sqlx::query(
        "INSERT INTO tag_aliases (alias_name, canonical_tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    ).bind(&body.alias_name).bind(body.canonical_tag_id)
     .execute(&state.db).await?;

    Ok(Json(json!({"err": 0, "msg": "alias created"})))
}
```

### Merge Tags

```rust
pub async fn merge_tags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<MergeTagsBody>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    if body.source_tag_id == body.target_tag_id {
        return Ok(Json(json!({"err": -1, "msg": "cannot merge tag into itself"})));
    }

    // Migrate fic_tags
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id) DO NOTHING"#,
    ).bind(body.source_tag_id).bind(body.target_tag_id)
     .execute(&state.db).await?;

    // Delete source
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(body.source_tag_id).execute(&state.db).await?;
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(body.source_tag_id).execute(&state.db).await?;

    Ok(Json(json!({"err": 0, "msg": "tags merged"})))
}
```

### Delete Tag

```rust
pub async fn delete_tag(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id).execute(&state.db).await?;

    if result.rows_affected() == 0 {
        return Ok(Json(json!({"err": -5, "msg": "tag not found"})));
    }

    Ok(Json(json!({"err": 0, "msg": "tag deleted"})))
}
```

### Flag Management

```rust
pub async fn list_flags(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<FlagListQuery>,
) -> Result<Json<Value>, AppError> {
    verify_curator_token(&headers, &state.config.curator_token)?;

    let resolved_filter = params.resolved.unwrap_or(false);

    let rows: Vec<FlagRow> = sqlx::query_as::<_, FlagRow>(
        r#"SELECT id, url_id, tag_id, flagged_by_ip::text, reason, resolved, created_at
           FROM tag_flags WHERE resolved = $1 ORDER BY created_at DESC"#,
    ).bind(resolved_filter).fetch_all(&state.db).await?;

    let flags: Vec<Value> = rows.into_iter().map(|r| {
        json!({
            "id": r.id,
            "url_id": r.url_id,
            "tag_id": r.tag_id,
            "reason": r.reason,
            "resolved": r.resolved,
        })
    }).collect();

    Ok(Json(json!({"err": 0, "flags": flags})))
}
```

## Watch Out!

**Tag names are case-sensitive!** `COLLATE "C"` means "Harry Potter" ≠ "harry potter".

**Merging is destructive!** Always double-check before merging.

**The trigger handles scores!** Don't manually update `fic_tags.score`.

## Summary

FicHub's tag system supports community-submitted tags with voting, visibility thresholds, curator tools, and PostgreSQL triggers for automatic score management.

---

# Chapter 31: OPDS Catalog

## What is OPDS?

OPDS (Open Publication Distribution System) is a standard for distributing e-books using Atom XML feeds. E-readers like Kindle, Kobo, and Calibre can subscribe to OPDS catalogs.

## The Root Catalog

```xml
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <id>urn:fichub:catalog</id>
  <title>FicHub</title>
  <updated>2024-01-15T10:30:00Z</updated>
  <entry>
    <title>Recent Fics</title>
    <link href="/opds/new" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:new</id>
    <summary>Recently added and updated fanfiction</summary>
  </entry>
  <entry>
    <title>Popular Fics</title>
    <link href="/opds/popular" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:popular</id>
  </entry>
  <entry>
    <title>Tags</title>
    <link href="/opds/tags" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
    <id>urn:fichub:catalog:tags</id>
  </entry>
  <entry>
    <title>Authors</title>
    <link href="/opds/authors" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
    <id>urn:fichub:catalog:authors</id>
  </entry>
  <entry>
    <title>Recommendations</title>
    <link href="/opds/recommendations/popular" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:recommendations</id>
  </entry>
  <entry>
    <title>Search</title>
    <link href="/opds/search?q=" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
    <id>urn:fichub:catalog:search</id>
  </entry>
</feed>
```

## Feed Types

**Navigation feeds** list categories:
```xml
<link href="/opds/new" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
```

**Acquisition feeds** list downloadable books with download links.

## Fic Entry

```rust
pub fn fic_entry(
    url_id: &str, title: &str, author: &str, summary: &str,
    updated: &str, words: i64, chapters: i32, status: &str,
) -> String {
    format!(
        r#"  <entry>
    <title>{title}</title>
    <author><name>{author}</name></author>
    <id>urn:fichub:fic:{url_id}</id>
    <updated>{updated}</updated>
    <summary>{summary}</summary>
    <dc:extent>{words}</dc:extent>
    <dc:format>application/epub+zip</dc:format>
    <category term="{status}" label="{status}"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/epub/{url_id}/{url_id}.epub" type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/pdf/{url_id}/{url_id}.pdf" type="application/pdf"/>
  </entry>"#,
        title = html_escape(title),
        author = html_escape(author),
        summary = html_escape(summary),
        // ...
    )
}
```

## Pagination

```rust
pub fn build_feed(
    title: &str, feed_id: &str, entries: &str, updated: &str,
    self_link: Option<&str>, feed_kind: FeedKind,
    pagination: Option<&PaginationInfo>,
) -> String {
    let mut links = String::new();

    if let Some(ref pagi) = pagination {
        let total_pages = (pagi.total as f64 / pagi.per_page as f64).ceil() as usize;
        if pagi.page > 1 {
            links.push_str(&format!(
                r#"  <link href="{base}?page={prev}" rel="previous"/>"#,
                base = pagi.base_path, prev = pagi.page - 1
            ));
        }
        if pagi.page < total_pages {
            links.push_str(&format!(
                r#"  <link href="{base}?page={next}" rel="next"/>"#,
                base = pagi.base_path, next = pagi.page + 1
            ));
        }
    }

    format!(
        r#"{header}<id>{id}</id>
  <title>{title}</title>
  <updated>{updated}</updated>
  <author><name>FicHub</name></author>
{links}{entries}</feed>"#,
        // ...
    )
}
```

## Shelf Authentication

```rust
pub async fn shelf_contents(
    State(state): State<Arc<AppState>>,
    Path(shelf_id): Path<i32>,
    Query(params): Query<ShelfQuery>,
) -> Result<impl IntoResponse, AppError> {
    let token = params.token.as_deref().unwrap_or("");
    if token != state.config.opds_shelf_token {
        return Err(AppError::BadRequest(-1, "invalid token".into()));
    }
    // ... return shelf contents ...
}
```

## Watch Out!

**Content-Type headers are critical!** Wrong headers confuse e-readers.

**XML escaping is mandatory!** All user content must be XML-escaped.

## Summary

FicHub's OPDS catalog provides a standard interface for e-readers with navigation feeds, acquisition feeds, pagination, and token-based shelf authentication.

---

# Chapter 32: API Documentation

## Self-Documenting API

FicHub provides API documentation at `/api/`:

```rust
pub async fn api_docs_handler() -> impl IntoResponse {
    Json(json!({
        "name": "fichub-rs API",
        "version": "0.1.0",
        "endpoints": {
            "/api/v0/epub": {
                "method": "GET",
                "params": { "q": "URL of the fanfiction" },
                "description": "Fetch metadata and download links"
            },
            // ...
        }
    }))
}
```

## Response Format

```json
{
    "err": 0,
    "msg": "...",
    "data": { ... }
}
```

| Code | Meaning |
|------|---------|
| 0 | Success |
| -1 | Bad request |
| -5 | Not found |
| -6 | Scraper error |
| -7 | Blacklisted |
| -10 | Automated blocked |
| -429 | Rate limited |

## Summary

FicHub's API documentation is self-hosted and follows a consistent format with error codes.

---

# Chapter 33: Performance

## Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(10))
    .connect(database_url)
    .await?;
```

## Caching Layers

1. **Database cache** — `export_log` table
2. **Disk cache** — EPUB/HTML files on disk
3. **Redis cache** — Rate limiter state
4. **Precomputed recommendations** — Cached results

## Profiling

```bash
cargo install flamegraph
cargo flamegraph -- -c ./fichub
```

## Database Query Performance

```sql
EXPLAIN ANALYZE SELECT ... FROM fic_info WHERE text_search @@ plainto_tsquery('english', 'harry potter');
```

## Async Performance

- **Connection reuse** — `reqwest::Client` reuses TCP connections
- **Semaphore control** — Prevents duplicate work
- **Redis multiplexing** — Concurrent Redis operations

## Watch Out!

**Don't block the async runtime!** Use `spawn_blocking` for sync operations.

**Monitor memory usage!** Use `valgrind` or `heaptrack` for leak detection.

**Connection pool exhaustion!** Keep database operations short.

## Summary

FicHub's performance relies on connection pooling, multi-layered caching, async I/O, and careful resource management.


---

# Part 8: Testing and Production

---

# Chapter 34: Backend Testing

## The Testing Pyramid

Testing is essential for maintaining a reliable backend. FicHub follows the testing pyramid: many unit tests, fewer integration tests, and a handful of end-to-end tests.

## Unit Tests with #[cfg(test)]

FicHub embeds unit tests directly in source files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_url_id_deterministic() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(1, "story_123");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_url_id_different_source_id() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(2, "story_123");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_url_id_length() {
        let id = generate_url_id(42, "abc123");
        assert_eq!(id.len(), 12);
    }

    #[test]
    fn test_generate_url_id_hex_chars() {
        let id = generate_url_id(7, "test_url");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
```

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test config::tests

# Specific test
cargo test test_from_env_defaults

# With output
cargo test -- --nocapture

# By name pattern
cargo test url_id
```

## Assert Macros

```rust
// Basic assertions
assert_eq!(result, expected);
assert_ne!(result, unexpected);
assert!(condition);

// With messages
assert_eq!(result, expected, "got {:?}", result);

// Expected panics
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

## Test Helpers

### EnvGuard for Environment Variables

```rust
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    keys: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self {
        EnvGuard { keys: Vec::new() }
    }

    fn set(&mut self, key: &str, val: &str) {
        self.keys.push(key.to_string());
        unsafe { std::env::set_var(key, val); }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for key in &self.keys {
            unsafe { std::env::remove_var(key); }
        }
    }
}
```

### clear_config_env

```rust
fn clear_config_env() {
    let keys = [
        "DATABASE_URL", "REDIS_URL", "CACHE_DIR", "SECONDARY_CACHE_DIR",
        "EXPORT_VERSION", "DYNAMIC_RATE_LIMIT", "NODE_NAME", "CALIBRE_CONTAINER",
        "TMP_DIR", "PORT", "FRONTEND_DIR", "TRUSTED_PROXIES", "IP_TAG_SOURCES",
        "REC_DEFAULT_DELAY_SECS", "REC_SITE_RATE_LIMITS", "REC_MAX_FAVOURITE_PAGES",
        "REC_MAX_USER_FAVOURITE_PAGES", "REC_MAX_RECOMMENDATIONS",
        "REC_MIN_FAVOURITERS_FOR_COLLAB", "REC_VOTING_BOOST_GAMMA",
        "REC_CACHE_TTL_HOURS", "REC_SUGGEST_LIMIT_PER_HOUR", "REC_VOTE_LIMIT_PER_HOUR",
        "REC_PRECOMPUTE_ENABLED", "REC_PRECOMPUTE_INTERVALHours", "REC_ENABLE_CROSS_SITE",
    ];
    for key in &keys {
        unsafe { std::env::remove_var(key); }
    }
}
```

## Testing Configuration

### Default Values

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
    assert_eq!(config.cache_dir, PathBuf::from("./cache"));
    assert!(config.secondary_cache_dir.is_none());
    assert_eq!(config.export_version, 1);
    assert!(config.dynamic_rate_limit);
    assert_eq!(config.node_name, "orion");
    assert_eq!(config.app_port, 3000);
    assert_eq!(config.frontend_dir, PathBuf::from("./frontend/build"));
    assert!(config.trusted_proxies.is_empty());
    assert_eq!(config.rec_default_delay_secs, 5);
    assert_eq!(config.rec_max_favourite_pages, 3);
    assert_eq!(config.rec_max_recommendations, 20);
    assert_eq!(config.rec_min_favouriters_for_collab, 5);
    assert!((config.rec_voting_boost_gamma - 0.2).abs() < f64::EPSILON);
    assert_eq!(config.rec_cache_ttl_hours, 12);
    assert!(config.rec_precompute_enabled);
    assert!(config.rec_enable_cross_site);
}
```

### Custom Values

```rust
#[test]
fn test_from_env_custom_values() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://custom/db");
    guard.set("REDIS_URL", "redis://custom");
    guard.set("CACHE_DIR", "/alt/cache");
    guard.set("PORT", "9090");
    guard.set("FRONTEND_DIR", "/alt/frontend");
    guard.set("TRUSTED_PROXIES", "10.0.0.1, 10.0.0.2");
    guard.set("REC_MAX_RECOMMENDATIONS", "100");
    guard.set("REC_VOTING_BOOST_GAMMA", "0.8");

    let config = Config::from_env();

    assert_eq!(config.database_url, "postgres://custom/db");
    assert_eq!(config.cache_dir, PathBuf::from("/alt/cache"));
    assert_eq!(config.app_port, 9090);
    assert_eq!(config.trusted_proxies, vec!["10.0.0.1".into(), "10.0.0.2".into()]);
    assert_eq!(config.rec_max_recommendations, 100);
    assert!((config.rec_voting_boost_gamma - 0.8).abs() < f64::EPSILON);
}
```

### Panics on Missing Required Config

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

#[test]
#[should_panic(expected = "REDIS_URL must be set")]
fn test_panics_without_redis_url() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test");
    let _ = Config::from_env();
}
```

### JSON Parsing

```rust
#[test]
fn test_rec_site_rate_limits_valid_json() {
    let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    clear_config_env();
    let mut guard = EnvGuard::new();
    guard.set("DATABASE_URL", "postgres://localhost/test");
    guard.set("REDIS_URL", "redis://localhost");
    guard.set("REC_SITE_RATE_LIMITS", r#"{"ao3": 10, "ffn": 5}"#);

    let config = Config::from_env();
    let mut expected = HashMap::new();
    expected.insert("ao3".to_string(), 10);
    expected.insert("ffn".to_string(), 5);
    assert_eq!(config.rec_site_rate_limits, expected);
}

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

## Testing Error Types

### Display Implementation

```rust
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
fn test_display_not_found() {
    let err = AppError::NotFound("story".into());
    let s = format!("{}", err);
    assert!(s.contains("NotFound"));
    assert!(s.contains("story"));
}

#[test]
fn test_display_internal() {
    let err = AppError::Internal("oops".into());
    let s = format!("{}", err);
    assert!(s.contains("Internal"));
    assert!(s.contains("oops"));
}

#[test]
fn test_display_scrape_error() {
    let err = AppError::ScrapeError("timeout".into());
    let s = format!("{}", err);
    assert!(s.contains("ScrapeError"));
    assert!(s.contains("timeout"));
}

#[test]
fn test_display_export_error() {
    let err = AppError::ExportError("epub fail".into());
    let s = format!("{}", err);
    assert!(s.contains("ExportError"));
    assert!(s.contains("epub fail"));
}

#[test]
fn test_display_database() {
    let err = AppError::Database("conn lost".into());
    let s = format!("{}", err);
    assert!(s.contains("Database"));
    assert!(s.contains("conn lost"));
}

#[test]
fn test_display_cache_error() {
    let err = AppError::CacheError("cache miss".into());
    let s = format!("{}", err);
    assert!(s.contains("CacheError"));
    assert!(s.contains("cache miss"));
}
```

### From Implementations

```rust
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
    let err = redis::RedisError::from((redis::ErrorKind::Io, "connection refused"));
    let app: AppError = err.into();
    match app {
        AppError::CacheError(msg) => assert!(msg.contains("connection refused")),
        _ => panic!("expected CacheError, got {:?}", app),
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
```

## Testing Token Buckets

FicHub has a pure-Rust token bucket implementation for testing:

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
            -1.0
        } else {
            (requested - new_tokens) / self.flow
        }
    }

    fn penalize(&mut self, now: f64) {
        self.request(1.5, now);
    }
}

#[test]
fn test_initial_bucket_fill() {
    let mut bucket = TokenBucket::new(100.0, 10.0, 0.0);
    assert!((bucket.value - 100.0).abs() < f64::EPSILON);
    let result = bucket.request(10.0, 0.0);
    assert!((result - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 90.0).abs() < f64::EPSILON);
}

#[test]
fn test_token_refill_over_time() {
    let mut bucket = TokenBucket::new(50.0, 10.0, 0.0);
    bucket.request(50.0, 0.0);  // Empty
    assert!((bucket.value - 0.0).abs() < f64::EPSILON);

    let r = bucket.request(20.0, 3.0);  // After 3s, 30 refilled
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);
}

#[test]
fn test_capacity_limits_refill() {
    let mut bucket = TokenBucket::new(50.0, 100.0, 0.0);
    bucket.request(50.0, 0.0);
    // After 10s: 1000 tokens, but capped at 50
    let wait = bucket.request(60.0, 10.0);
    assert!(wait > 0.0);
    let expected = (60.0 - 50.0) / 100.0;
    assert!((wait - expected).abs() < f64::EPSILON);
}

#[test]
fn test_wait_calculation() {
    let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
    bucket.request(10.0, 0.0);
    let wait = bucket.request(5.0, 0.0);
    let expected = 5.0 / 2.0;
    assert!((wait - expected).abs() < f64::EPSILON);
}

#[test]
fn test_multiple_requests_over_time() {
    let mut bucket = TokenBucket::new(20.0, 5.0, 0.0);

    let r = bucket.request(10.0, 0.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);

    let r = bucket.request(15.0, 2.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 5.0).abs() < f64::EPSILON);

    let r = bucket.request(20.0, 5.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 0.0).abs() < f64::EPSILON);

    let wait = bucket.request(10.0, 6.0);
    let expected = (10.0 - 5.0) / 5.0;
    assert!((wait - expected).abs() < f64::EPSILON);
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

## Testing Cache Paths

```rust
#[test]
fn test_cache_path_short_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Epub, "abc", "hash123");
    assert_eq!(path, Path::new("/cache/epub/abc/abc/hash123.epub"));
}

#[test]
fn test_cache_path_medium_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Html, "abcdef", "h");
    assert_eq!(path, Path::new("/cache/html/abc/def/abcdef/h.zip"));
}

#[test]
fn test_cache_path_long_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Mobi, "abcdefghijklm", "h1");
    assert_eq!(path, Path::new("/cache/mobi/abc/def/ghi/abcdefghijklm/h1.mobi"));
}

#[test]
fn test_cache_path_different_etypes() {
    let root = Path::new("/cache");
    let url_id = "abc";
    let hash = "h";
    assert_eq!(cache_path(root, &EType::Epub, url_id, hash), Path::new("/cache/epub/abc/abc/h.epub"));
    assert_eq!(cache_path(root, &EType::Html, url_id, hash), Path::new("/cache/html/abc/abc/h.zip"));
    assert_eq!(cache_path(root, &EType::Mobi, url_id, hash), Path::new("/cache/mobi/abc/abc/h.mobi"));
    assert_eq!(cache_path(root, &EType::Pdf, url_id, hash), Path::new("/cache/pdf/abc/abc/h.pdf"));
}
```

## Testing File MD5

```rust
#[test]
fn test_file_md5_known_content() {
    let dir = std::env::temp_dir().join("fichub_test_disk_md5");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("test.txt");
    std::fs::write(&file_path, b"hello world").unwrap();
    let result = file_md5(&file_path).unwrap();
    assert_eq!(result, "5eb63bbbe01eeed093cb22bb8f5acdc3");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_file_md5_empty_file() {
    let dir = std::env::temp_dir().join("fichub_test_disk_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("empty.txt");
    std::fs::write(&file_path, b"").unwrap();
    let result = file_md5(&file_path).unwrap();
    assert_eq!(result, "d41d8cd98f00b204e9800998ecf8427e");
    let _ = std::fs::remove_dir_all(&dir);
}
```

## Testing Export Helpers

```rust
fn make_test_meta() -> FicMetadata {
    FicMetadata {
        url_id: "test123".into(),
        title: "Test Fic".into(),
        author: "Test Author".into(),
        chapters: 10,
        words: 50000,
        desc: "<p>A great story</p>".into(),
        published: 1700000000000,
        updated: 1700000000000,
        status: "complete".into(),
        source: "https://archiveofourown.org/works/123456".into(),
        source_id: 1,
        author_id: 42,
        author_url: "https://archiveofourown.org/users/TestAuthor".into(),
        author_local_id: "123456".into(),
        content_hash: None,
        extra_meta: None,
        raw_extended_meta: None,
    }
}

#[test]
fn test_generate_slug_basic() {
    let slug = super::generate_slug("The Best Story", "abc123");
    assert!(slug.contains("The_Best_Story"));
    assert!(slug.contains("abc123"));
    assert!(!slug.starts_with('_'));
    assert!(!slug.ends_with('_'));
}

#[test]
fn test_generate_slug_special_chars() {
    let slug = super::generate_slug("Hello: World? (Part 1/2)", "xyz789");
    assert!(!slug.contains(':'));
    assert!(!slug.contains('?'));
    assert!(!slug.contains('('));
    assert!(slug.contains("Hello_World_Part_1_2"));
}

#[test]
fn test_build_info_string() {
    let meta = make_test_meta();
    let (info, notes) = super::build_info_string(&meta);
    assert!(info.contains("Test Fic"));
    assert!(info.contains("Test Author"));
    assert!(info.contains("50000"));
    assert!(info.contains("10"));
    assert!(info.contains("complete"));
    assert!(notes.is_empty());
}

#[test]
fn test_build_meta_json() {
    let meta = make_test_meta();
    let json = super::build_meta_json(&meta);
    assert_eq!(json["id"], "test123");
    assert_eq!(json["title"], "Test Fic");
    assert_eq!(json["author"], "Test Author");
    assert_eq!(json["chapters"], 10);
    assert_eq!(json["words"], 50000);
    assert_eq!(json["status"], "complete");
    assert_eq!(json["source_id"], 1);
    assert_eq!(json["author_id"], 42);
}

#[test]
fn test_build_metadata_response_greylisted() {
    let meta = make_test_meta();
    let resp = super::build_metadata_response(&meta, &[], &1, None, true);
    let json = resp.0;
    assert_eq!(json["err"], 0);
    assert!(json["notes"][0].as_str().unwrap_or("").contains("greylisted"));
    assert!(json["urls"].as_object().unwrap().is_empty());
}
```

## Testing EType

```rust
#[test]
fn test_etype_from_str() {
    assert_eq!("epub".parse::<EType>().unwrap(), EType::Epub);
    assert_eq!("html".parse::<EType>().unwrap(), EType::Html);
    assert_eq!("mobi".parse::<EType>().unwrap(), EType::Mobi);
    assert_eq!("pdf".parse::<EType>().unwrap(), EType::Pdf);
    assert!("invalid".parse::<EType>().is_err());
    assert_eq!("EPUB".parse::<EType>().unwrap(), EType::Epub);
}

#[test]
fn test_etype_as_str() {
    assert_eq!(EType::Epub.as_str(), "epub");
    assert_eq!(EType::Html.as_str(), "html");
    assert_eq!(EType::Mobi.as_str(), "mobi");
    assert_eq!(EType::Pdf.as_str(), "pdf");
}

#[test]
fn test_etype_suffix() {
    assert_eq!(EType::Epub.suffix(), ".epub");
    assert_eq!(EType::Html.suffix(), ".zip");
    assert_eq!(EType::Mobi.suffix(), ".mobi");
    assert_eq!(EType::Pdf.suffix(), ".pdf");
}

#[test]
fn test_etype_version() {
    assert_eq!(EType::Epub.version(), 1);
    assert_eq!(EType::Html.version(), 1);
    assert_eq!(EType::Mobi.version(), 0);
    assert_eq!(EType::Pdf.version(), 0);
}
```

## Testing Export Version Computation

```rust
#[test]
fn test_compute_version_all_zero() {
    assert_eq!(compute_version(0, 0, 0), 0);
}

#[test]
fn test_compute_version_all_one() {
    assert_eq!(compute_version(1, 1, 1), 3);
}

#[test]
fn test_compute_version_mixed() {
    assert_eq!(compute_version(5, 0, 3), 8);
    assert_eq!(compute_version(0, 5, 3), 8);
    assert_eq!(compute_version(3, 5, 0), 8);
}
```

## Testing Search Filters

```rust
#[test]
fn test_parse_tag_filters_empty() {
    let result = parse_tag_filters("").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_tag_filters_single() {
    let result = parse_tag_filters("1:Harry Potter").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].tag_type_id, 1);
    assert_eq!(result[0].tag_name, "Harry Potter");
}

#[test]
fn test_parse_tag_filters_multiple() {
    let result = parse_tag_filters("1:Harry Potter,2:Hermione Granger,4:Angst").unwrap();
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].tag_name, "Harry Potter");
    assert_eq!(result[1].tag_name, "Hermione Granger");
    assert_eq!(result[2].tag_name, "Angst");
}

#[test]
fn test_parse_tag_filters_invalid_format() {
    let result = parse_tag_filters("invalid");
    assert!(result.is_err());
}

#[test]
fn test_parse_tag_filters_trailing_comma() {
    let result = parse_tag_filters("1:test,").unwrap();
    assert_eq!(result.len(), 1);
}
```

## Testing Scraper Error Display

```rust
#[test]
fn test_scrape_error_display_not_found() {
    assert_eq!(format!("{}", ScrapeError::NotFound), "fic not found");
}

#[test]
fn test_scrape_error_display_blocked() {
    assert_eq!(format!("{}", ScrapeError::Blocked), "blocked by site");
}

#[test]
fn test_scrape_error_display_network() {
    let err = ScrapeError::Network("connection refused".into());
    assert_eq!(format!("{}", err), "network error: connection refused");
}

#[test]
fn test_scrape_error_display_parse_error() {
    let err = ScrapeError::ParseError("unexpected token".into());
    assert_eq!(format!("{}", err), "parse error: unexpected token");
}
```

## Testing Tag Resolution

```rust
#[test]
fn test_tag_resolution_struct_sizes() {
    assert_eq!(std::mem::size_of::<TagResolution>(), std::mem::size_of::<(i32, String, i16, bool)>());
}

#[test]
fn test_tag_resolution_new_tag() {
    let tag = TagResolution {
        tag_id: 1,
        tag_name: "Angst".into(),
        tag_type_id: 4,
        is_new: true,
    };
    assert_eq!(tag.tag_id, 1);
    assert_eq!(tag.tag_name, "Angst");
    assert!(tag.is_new);
}

#[test]
fn test_tag_resolution_existing_tag() {
    let tag = TagResolution {
        tag_id: 42,
        tag_name: "Fluff".into(),
        tag_type_id: 4,
        is_new: false,
    };
    assert!(!tag.is_new);
}
```

## Testing Vote Visibility

```rust
#[test]
fn test_is_hidden_above_threshold() {
    assert!(!is_hidden(0, -3));
    assert!(!is_hidden(-2, -3));
    assert!(!is_hidden(10, -3));
}

#[test]
fn test_is_hidden_at_or_below_threshold() {
    assert!(is_hidden(-3, -3));
    assert!(is_hidden(-5, -3));
    assert!(is_hidden(-10, -3));
    assert!(is_hidden(0, 0));
}

#[test]
fn test_compute_visibility() {
    let scores = vec![(1, 5), (2, 0), (3, -3), (4, -10)];
    let result = compute_visibility(&scores, -3);
    assert_eq!(result.len(), 4);
    assert!(!result[0].1); // score=5 > -3
    assert!(!result[1].1); // score=0 > -3
    assert!(result[2].1);  // score=-3 == -3
    assert!(result[3].1);  // score=-10 < -3
}
```

## Watch Out!

**Tests must be deterministic!** Avoid current time, random numbers, or external services.

**Parallel test execution!** Rust runs tests in parallel. Use mutexes for shared resources.

**Clean up test data!** Tests that modify the database should clean up.

## Summary

FicHub has comprehensive unit tests covering configuration, error handling, cache paths, token buckets, search filters, tag resolution, vote visibility, and export helpers. The `EnvGuard` helper ensures clean environment variable management.


---

# Supplementary Content: Complete API Reference and Operations Guide

---

# Complete API Reference

## Authentication

FicHub uses two authentication mechanisms:

### Bearer Token (Curator Endpoints)

Curator endpoints require a Bearer token in the Authorization header:

```http
POST /api/v0/curator/alias
Authorization: Bearer my-secret-token
Content-Type: application/json

{
    "alias_name": "HP",
    "canonical_tag_id": 42
}
```

The token is configured via the `CURATOR_TOKEN` environment variable. If not configured, curator endpoints return an error.

### IP-Based Identification (Tags and Recommendations)

Tag submissions, votes, and recommendations use the client's IP address for identification. This is extracted via the `ConnectInfo` extractor:

```rust
async fn handler(
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    let ip = remote.ip();
    // Use ip for rate limiting and vote tracking
}
```

## Endpoint Reference

### GET /api/

Returns API documentation. No parameters required.

**Response:**
```json
{
    "name": "fichub-rs API",
    "version": "0.1.0",
    "endpoints": {
        "/api/v0/epub": {
            "method": "GET",
            "params": { "q": "URL of the fanfiction" },
            "description": "Fetch metadata and download links"
        }
    }
}
```

### GET /api/v0/epub

The main export endpoint. Scrapes a fanfiction URL, generates exports, and returns download links.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | URL of the fanfiction |
| `automated` | string | No | Set to "true" to block automated requests |
| `format` | string | No | Specific format to generate (epub, html, mobi, pdf) |

**Example Request:**
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345678
```

**Success Response (200):**
```json
{
    "err": 0,
    "q": "https://archiveofourown.org/works/12345678",
    "url_id": "a1b2c3d4e5f6",
    "slug": "My_Amazing_Fic-a1b2c3d4e5f6",
    "meta": {
        "id": "a1b2c3d4e5f6",
        "title": "My Amazing Fanfiction",
        "author": "SomeAuthor",
        "chapters": 15,
        "words": 125000,
        "description": "A story about things...",
        "status": "complete",
        "source": "https://archiveofourown.org/works/12345678",
        "created": "2023-01-15T00:00:00Z",
        "updated": "2023-06-20T00:00:00Z"
    },
    "hashes": {
        "epub": "abc123def456",
        "html": "789xyz012abc"
    },
    "urls": {
        "epub": "/cache/epub/a1b2c3d4e5f6?h=abc123def456",
        "html": "/cache/html/a1b2c3d4e5f6?h=789xyz012abc"
    },
    "epub_url": "/cache/epub/a1b2c3d4e5f6?h=abc123def456",
    "html_url": "/cache/html/a1b2c3d4e5f6?h=789xyz012abc",
    "info": "My Amazing Fanfiction by SomeAuthor\n125000 words in 15 chapters\nStatus: complete\n",
    "notes": []
}
```

**Error Responses:**

| err | HTTP Status | Meaning |
|-----|-------------|---------|
| -1 | 400 | Missing query parameter |
| -5 | 400 | Unsupported URL or story not found |
| -7 | 400 | Fic or author is blacklisted |
| -10 | 400 | Automated request blocked |
| -429 | 429 | Rate limited |
| -6 | 502 | Scraper error (site down, blocked, etc.) |
| -1 | 500 | Internal server error |

### GET /api/v0/meta

Returns metadata without generating exports. Same parameters as `/api/v0/epub` but faster.

**Example Request:**
```
GET /api/v0/meta?q=https://archiveofourown.org/works/12345678
```

**Response:** Same as `/api/v0/epub` but with empty hashes and URLs.

### GET /api/v0/remote

Returns the client's IP address.

**Response:**
```json
{
    "ip": "192.168.1.100",
    "port": 54321,
    "is_automated": false
}
```

### GET /cache/{etype}/{url_id}/{fname}

Downloads a cached export file with hash validation.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `etype` | string | Export type: epub, html, mobi, pdf |
| `url_id` | string | Fic's unique ID |
| `fname` | string | Filename (contains the hash) |
| `h` | string | MD5 hash for validation (query param) |

**Example:**
```
GET /cache/epub/a1b2c3d4e5f6/my_fic.epub?h=abc123def456
```

**Response:** Binary file download with appropriate Content-Type header.

### GET /cache/{etype}/{url_id}

Downloads a cached export or redirects to trigger generation.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `etype` | string | Export type |
| `url_id` | string | Fic's unique ID |
| `h` | string | MD5 hash (query param, optional) |

If `h` is provided and the file exists, it's served directly. Otherwise, redirects to the frontend to trigger generation.

### GET /api/v0/recommendations

Returns recommendations for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | One of q/url_id | Fanfiction URL |
| `url_id` | string | One of q/url_id | Fic's unique ID |
| `n` | integer | No | Number of recommendations (default: 20, max: 100) |
| `site_domain` | string | No | Filter by site domain |

**Example:**
```
GET /api/v0/recommendations?url_id=a1b2c3d4e5f6&n=10
```

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "recommendations": [
        {
            "url_id": "x9y8z7w6v5u4",
            "title": "Similar Story",
            "author": "Another Author",
            "words": 85000,
            "chapters": 20,
            "status": "complete",
            "score": 0.75,
            "community_score": 3,
            "download_urls": {}
        }
    ],
    "generated_at": "2024-01-15T10:30:00Z"
}
```

### POST /api/v0/recommendations/suggest

Submit a community suggestion linking two fics.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "suggested_url": "https://archiveofourown.org/works/87654321",
    "comment": "Both feature time travel to the Marauders era"
}
```

**Response:**
```json
{
    "err": 0,
    "suggestion_id": 42
}
```

### POST /api/v0/recommendations/vote

Vote on a community suggestion.

**Request Body:**
```json
{
    "suggestion_id": 42,
    "vote": 1
}
```

**Response:**
```json
{
    "err": 0,
    "new_score": 5
}
```

### GET /api/v0/recommendations/votes

List suggestions and votes for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Fic's unique ID |

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "suggestions": [
        {
            "id": 42,
            "suggested_url_id": "x9y8z7w6v5u4",
            "comment": "Similar time travel premise",
            "net_votes": 5,
            "created": "2024-01-15T10:30:00Z"
        }
    ]
}
```

### POST /api/v0/tags/submit

Submit a tag on a fic.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_name": "Angst",
    "tag_type_id": 4
}
```

**Response:**
```json
{
    "err": 0,
    "tag_id": 123,
    "tag_name": "Angst",
    "tag_type_id": 4,
    "is_new": false
}
```

### POST /api/v0/tags/vote

Vote on a tag.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_id": 123,
    "value": 1
}
```

**Response:**
```json
{
    "err": 0,
    "new_score": 3,
    "hidden": false
}
```

### POST /api/v0/tags/flag

Flag a tag for curator review.

**Request Body:**
```json
{
    "url_id": "a1b2c3d4e5f6",
    "tag_id": 123,
    "reason": "Tag is incorrect for this story"
}
```

**Response:**
```json
{
    "err": 0,
    "msg": "flag submitted"
}
```

### GET /api/v0/tags

Get tags for a fic.

**Parameters:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url_id` | string | Yes | Fic's unique ID |

**Response:**
```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "tags": [
        {
            "id": 1,
            "name": "Harry Potter",
            "tag_type_id": 1,
            "score": 5,
            "hidden": false
        },
        {
            "id": 42,
            "name": "Angst",
            "tag_type_id": 4,
            "score": -4,
            "hidden": true
        }
    ]
}
```

### POST /api/v0/curator/alias

Create a tag alias (requires Bearer token).

**Request Body:**
```json
{
    "alias_name": "HP",
    "canonical_tag_id": 1
}
```

### POST /api/v0/curator/merge

Merge two tags (requires Bearer token).

**Request Body:**
```json
{
    "source_tag_id": 10,
    "target_tag_id": 1
}
```

### DELETE /api/v0/curator/tags/{id}

Delete a tag (requires Bearer token).

### GET /api/v0/curator/flags

List unresolved flags (requires Bearer token).

### POST /api/v0/curator/flags/{id}/resolve

Resolve a flag (requires Bearer token).

### GET /api/v0/search

Full-text search with filters.

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `q` | string | Full-text search query |
| `include_tags` | string | Comma-separated "type_id:name" (ALL must match) |
| `exclude_tags` | string | Comma-separated "type_id:name" (NONE must match) |
| `include_any_tags` | string | Comma-separated "type_id:name" (at least ONE) |
| `min_words` | integer | Minimum word count |
| `max_words` | integer | Maximum word count |
| `min_chapters` | integer | Minimum chapter count |
| `max_chapters` | integer | Maximum chapter count |
| `complete` | boolean | Filter by completion status |
| `source` | string | Filter by source URL |
| `date_from` | string | ISO 8601 date |
| `date_to` | string | ISO 8601 date |
| `sort` | string | Sort: -relevance, -date, -words, -chapters, -title |
| `page` | integer | Page number (default: 1) |
| `per_page` | integer | Results per page (default: 20, max: 50) |

**Example:**
```
GET /api/v0/search?q=harry+potter&include_tags=1:Harry+Potter&complete=true&sort=-words
```

**Response:**
```json
{
    "total": 1234,
    "page": 1,
    "per_page": 20,
    "results": [
        {
            "url_id": "a1b2c3d4e5f6",
            "title": "My Amazing Harry Potter Fic",
            "author": "SomeAuthor",
            "source": "https://archiveofourown.org/works/12345678",
            "words": 125000,
            "chapters": 15,
            "status": "complete",
            "description": "A Harry Potter story...",
            "rank": 0.85,
            "tags": [
                {"name": "Harry Potter", "type": "fandom", "type_id": 1, "score": 10}
            ],
            "total_freeform": 5
        }
    ]
}
```

---

# Operations Guide

## Monitoring

### Health Check

Add a health check endpoint:

```rust
async fn health_check() -> &'static str {
    "OK"
}

// In the router:
.route("/health", get(health_check))
```

### Key Metrics to Monitor

1. **Request rate** — Requests per second
2. **Response time** — p50, p95, p99 latencies
3. **Error rate** — Percentage of 4xx and 5xx responses
4. **Database connections** — Active vs pool size
5. **Cache hit rate** — Percentage served from cache
6. **Disk usage** — Cache directory size
7. **Redis memory** — Used memory and peak memory

### Log Analysis

FicHub logs every request with timing:

```
2024-01-15T10:30:00.123Z INFO request{method=GET uri=/api/v0/epub}: 200 response_time=1.234s
```

Use `grep` or `awk` to analyze:

```bash
# Requests per minute
grep "INFO request" fichub.log | awk '{print $1}' | cut -d: -f1-2 | sort | uniq -c

# Slow requests (> 5 seconds)
grep "response_time=[5-9]" fichub.log

# Error rates
grep -c "response_time=.*5[0-9][0-9]" fichub.log
```

## Backups

### Database Backup

```bash
# Daily backup
pg_dump -U fichub fichub | gzip > backup_$(date +%Y%m%d).sql.gz

# Restore
gunzip -c backup_20240115.sql.gz | psql -U fichub fichub

# Automated (add to crontab)
0 2 * * * pg_dump -U fichub fichub | gzip > /backups/fichub_$(date +\%Y\%m\%d).sql.gz
```

### Cache Backup

```bash
# Backup cache directory
rsync -avz /data/cache/ backup:/backups/fichub-cache/

# Automated
0 3 * * * rsync -avz /data/cache/ backup:/backups/fichub-cache/
```

### Full Backup Script

```bash
#!/bin/bash
set -e

BACKUP_DIR="/backups/fichub/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Database
pg_dump -U fichub fichub | gzip > "$BACKUP_DIR/database.sql.gz"

# Cache
rsync -avz /data/cache/ "$BACKUP_DIR/cache/"

# Config
cp /home/fichub/.env "$BACKUP_DIR/"

# Cleanup old backups (keep 30 days)
find /backups/fichub -maxdepth 1 -type d -mtime +30 -exec rm -rf {} +

echo "Backup complete: $BACKUP_DIR"
```

## Troubleshooting

### Common Issues

**"DATABASE_URL must be set"**
- Check that the `.env` file exists and has the correct value
- Verify the environment variable is exported

**"Failed to connect to database"**
- Verify PostgreSQL is running: `systemctl status postgresql`
- Check the connection string: `psql "postgres://fichub:fichub@localhost/fichub"`
- Check firewall rules if connecting remotely

**"Failed to connect to Redis"**
- Verify Redis is running: `systemctl status redis`
- Test connection: `redis-cli ping`

**"No scraper found for URL"**
- The URL format might be wrong
- Check supported sites: AO3, FF.net, XenForo, FictionPress, AFF, HP FanFic

**"Rate limited"**
- Too many requests from the same IP
- Wait for the retry-after period
- Check rate limiter configuration

**"Fic is blacklisted"**
- The fic or author is on the blacklist
- Check the blacklist tables in the database

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug,fichub=debug ./fichub
```

### Database Debugging

```sql
-- Check table sizes
SELECT pg_size_pretty(pg_total_relation_size('fic_info')) AS size;

-- Check index usage
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;

-- Check slow queries
SELECT query, calls, mean_exec_time, total_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

## Performance Tuning

### Connection Pool

If you see "connection pool exhausted" errors:

```rust
// Increase pool size
PgPoolOptions::new()
    .max_connections(50)  // Default is 20
    .connect(url).await?;
```

### Cache Size

Monitor cache directory size:

```bash
du -sh /data/cache/
```

If it's growing too large, set up automatic cleanup:

```bash
# Delete cache files older than 30 days
find /data/cache -type f -mtime +30 -delete
```

### Rate Limiting

Adjust rate limits based on your traffic:

```bash
# More lenient for internal use
REC_DEFAULT_DELAY_SECS=2

# Stricter for public-facing
REC_DEFAULT_DELAY_SECS=10
```

## Summary

This operations guide covers monitoring, backups, troubleshooting, and performance tuning. A well-maintained FicHub instance requires regular monitoring, automated backups, and proactive performance optimization.


---

# Supplementary Content: Deep Dives and Extended Explanations

---

# Extended Chapter: Understanding Axum's Architecture

## How Axum Processes Requests

When a request arrives at FicHub's server, it goes through several layers before reaching your handler code. Understanding this pipeline helps you write better handlers and debug issues.

### The Tower Service Stack

Axum is built on Tower, a library for building modular middleware. Every Axum application is a Tower service — a function that takes a request and returns a response. Middleware layers wrap this service, adding behavior before and after the handler runs.

When you write:

```rust
let app = Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(state);
```

You're building a stack that looks like this:

1. **CorsLayer** — Handles CORS headers (outermost)
2. **TraceLayer** — Logs the request and response
3. **Router** — Matches the URL to a handler
4. **epub_handler** — Your handler code (innermost)

When a request arrives, it flows from the outermost layer inward. Each layer can:
- Inspect or modify the request
- Pass it to the next layer
- Short-circuit and return a response immediately

### Request Flow in Detail

Here's what happens when you send `GET /api/v0/epub?q=https://ao3.org/works/123`:

1. **TCP connection** — The OS accepts the connection on port 3000
2. **HTTP parsing** — hyper (Axum's HTTP library) parses the request bytes into a structured request
3. **CorsLayer** — Checks if the request origin is allowed; adds CORS headers to the response
4. **TraceLayer** — Records the start time, method, and URI; will log the response when it comes back
5. **Router** — Matches `/api/v0/epub` to `epub_handler`
6. **Extractor resolution** — Axum looks at the handler's parameters and extracts them from the request:
   - `State(state): State<Arc<AppState>>` — Clones the Arc from the router's state
   - `Query(params): Query<ExportQuery>` — Parses the query string into `ExportQuery`
7. **Handler execution** — Your async function runs
8. **Response serialization** — The return value is converted to an HTTP response
9. **TraceLayer** — Logs the response status and timing
10. **CorsLayer** — Adds CORS headers if needed
11. **TCP send** — The response bytes are sent back to the client

### Why This Matters

Understanding the pipeline helps you:
- **Debug middleware issues** — If a request returns unexpected headers, check the middleware layers
- **Write efficient handlers** — Know what work the framework does vs. what you do
- **Understand error propagation** — Errors flow back through the same layers
- **Optimize performance** — Know where time is spent (parsing, extraction, handler, serialization)

## Extractors in Depth

Extractors are one of Axum's most powerful features. They let you declare what data you need from the request, and Axum figures out how to get it.

### How Extractors Work

When you write:

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
    Query(params): Query<ExportQuery>,
    Json(body): Json<SubmitBody>,
) -> Json<Value> {
    // ...
}
```

Axum resolves extractors in left-to-right order. For each parameter, it calls the corresponding `FromRequest` implementation, which either:
- Succeeds and provides the extracted data
- Fails and returns an error response (the handler is never called)

### Custom Extractors

You can create your own extractors by implementing `FromRequest`:

```rust
struct ClientIp(IpAddr);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let ConnectInfo(addr) = ConnectInfo::<SocketAddr>::from_request_parts(parts, _state).await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(ClientIp(addr.ip()))
    }
}
```

Now you can use `ClientIp(ip): ClientIp` in your handlers.

### extractor Combinations

Some extractors can be combined:

```rust
// Both work — Axum resolves them independently
async fn handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
) -> Json<Value> {
    // Access both state and client IP
}
```

But some extractors consume the request body, so they can't be combined:

```rust
// This WON'T work — both try to read the body
async fn handler(
    Json(body1): Json<FirstType>,
    Json(body2): Json<SecondType>,  // Error! Body already consumed
) -> Json<Value> {
    unreachable!()
}
```

## Router Patterns

### Nested Routes

Group related routes under a common prefix:

```rust
let api_routes = Router::new()
    .route("/epub", get(epub_handler))
    .route("/meta", get(meta_handler))
    .route("/search", get(search_handler));

let app = Router::new()
    .nest("/api/v0", api_routes)
    .nest("/opds", opds_routes);
```

### Method Routing

Chain multiple HTTP methods on the same path:

```rust
Router::new()
    .route("/items", get(list_items).post(create_item))
    .route("/items/{id}", get(get_item).put(update_item).delete(delete_item))
```

### Route Merging

Combine routers from different modules:

```rust
let app = Router::new()
    .merge(core_routes)
    .merge(recommender_routes)
    .merge(tag_routes)
    .merge(opds_routes);
```

## Error Handling Patterns

### The Result Return Type

Handlers return `Result<T, E>` where `E: IntoResponse`:

```rust
async fn handler() -> Result<Json<Value>, AppError> {
    let data = fetch_data().await?;  // ? converts errors
    Ok(Json(json!({ "data": data })))
}
```

### The ? Operator

The `?` operator is Rust's error propagation shorthand. When you write `fetch_data().await?`, it either:
- Returns the value on success
- Converts the error via `From` and returns it from the function

FicHub's `AppError` implements `From` for all common error types:

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}
```

### Custom Error Responses

The `IntoResponse` implementation on `AppError` controls the HTTP response:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            // ... other variants ...
        };
        (status, Json(body)).into_response()
    }
}
```

## Summary

Axum's architecture is built on Tower services and middleware layers. Understanding the request pipeline, extractor system, and error handling patterns is essential for building effective handlers.

---

# Extended Chapter: PostgreSQL Deep Dive

## Connection Pool Internals

When you create a `PgPool`, you're creating a pool of connections to PostgreSQL. Here's how it works:

### Pool Lifecycle

1. **Creation** — `PgPoolOptions::new().connect(url).await` creates the pool with zero connections
2. **Lazy connection** — Connections are created on-demand as handlers request them
3. **Maximum** — Up to `max_connections` (default 20) connections are created
4. **Idle** — Unused connections are kept alive with periodic pings
5. **Timeout** — If all connections are busy, new requests wait up to `acquire_timeout`
6. **Release** — After a query completes, the connection is returned to the pool

### Pool Configuration

```rust
PgPoolOptions::new()
    .max_connections(20)          // Maximum concurrent connections
    .min_connections(5)           // Minimum idle connections
    .max_lifetime(Duration::from_secs(1800))  // Recycle connections after 30 min
    .idle_timeout(Duration::from_secs(600))    // Close idle connections after 10 min
    .acquire_timeout(Duration::from_secs(10))  // Wait 10s for a connection
    .test_before_acquire(true)    // Test connections before handing them out
    .connect(url).await?
```

### Monitoring Pool Health

```sql
-- Active connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub' AND state = 'active';

-- Idle connections
SELECT count(*) FROM pg_stat_activity WHERE datname = 'fichub' AND state = 'idle';

-- Connection states
SELECT state, count(*) FROM pg_stat_activity WHERE datname = 'fichub' GROUP BY state;
```

If you see many idle connections, reduce `max_connections`. If you see many waiting requests, increase it.

## Query Performance

### Understanding EXPLAIN ANALYZE

```sql
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM fic_info WHERE text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY ts_rank(text_search, plainto_tsquery('english', 'harry potter')) DESC
LIMIT 20;
```

This shows:
- **Seq Scan vs Index Scan** — Whether PostgreSQL uses the GIN index
- **Cost** — Estimated and actual cost of each operation
- **Rows** — Estimated vs actual number of rows
- **Time** — Actual execution time

### Index Strategies

FicHub uses several index types:

**B-tree indexes** (default) — For equality and range queries:
```sql
CREATE INDEX idx_fic_info_id ON fic_info(id);
```

**GIN indexes** — For full-text search:
```sql
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

**Partial indexes** — For common filtered queries:
```sql
CREATE INDEX idx_request_log_date_export ON request_log(created)
WHERE export_file_name IS NOT NULL AND etype = 'epub';
```

**Composite indexes** — For multi-column queries:
```sql
CREATE INDEX idx_request_log_url_id_etype_created ON request_log(url_id, etype, created);
```

## JSON and JSONB

PostgreSQL supports JSON natively. FicHub stores some metadata as JSON text:

```sql
extra_meta TEXT  -- JSON text
raw_extended_meta TEXT  -- Raw JSON from the source
```

For complex JSON queries, you'd use JSONB:

```sql
-- Extract a field
SELECT extra_meta->>'rating' FROM fic_info WHERE id = 'abc123';

-- Filter by JSON field
SELECT * FROM fic_info WHERE (extra_meta::jsonb)->>'rating' = 'T';
```

## Transactions

For multi-step operations that must be atomic:

```rust
let mut tx = pool.begin().await?;

sqlx::query("UPDATE fic_tags SET score = score + 1 WHERE tag_id = $1")
    .bind(tag_id).execute(&mut *tx).await?;

sqlx::query("UPDATE fic_tag_votes SET value = value + 1 WHERE tag_id = $1")
    .bind(tag_id).execute(&mut *tx).await?;

tx.commit().await?;
```

If any step fails, the transaction is rolled back.

## Connection String Formats

```bash
# Simple
postgres://user:password@localhost/database

# With port
postgres://user:password@localhost:5432/database

# With options
postgres://user:password@localhost/database?sslmode=require

# Unix socket
postgres://user:password@/database?host=/var/run/postgresql
```

## Summary

Understanding connection pool internals, query performance, index strategies, and transaction handling helps you build reliable, fast database-backed applications.

---

# Extended Chapter: Async Rust Explained

## What is Async?

In synchronous code, operations execute one at a time. If your program needs to read a file, it blocks until the file is read. If it needs to make a network request, it blocks until the response arrives.

In asynchronous code, operations can be suspended while waiting for I/O. When a handler needs to wait for a database query, it yields control to the runtime, which can then handle other requests. When the query completes, the handler resumes.

## The async/await Pattern

```rust
async fn fetch_metadata(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;  // Suspends until response arrives
    let body = response.text().await?;         // Suspends until body is read
    Ok(body)
}
```

The `async` keyword marks a function as asynchronous. Inside, `.await` pauses execution until the future completes.

### What Happens at .await

When you write `reqwest::get(url).await`:

1. The function starts the HTTP request
2. It returns a future to the runtime
3. The runtime polls the future periodically
4. When the response arrives, the runtime resumes the function
5. The function continues with the response data

### Futures vs Promises

Rust futures are **lazy** — they don't do anything until polled. JavaScript promises are **eager** — they start executing immediately. This is a key difference.

```rust
// This does nothing — the future is just created
let future = async { 1 + 1 };

// This polls the future
let result = future.await;  // result = 2
```

## The Tokio Runtime

Tokio is the most popular async runtime for Rust. It provides:

- **Multi-threaded executor** — Runs tasks on a thread pool
- **Timer** — Sleeps and timeouts
- **I/O** — TCP, UDP, Unix sockets
- **Channels** — mpsc, oneshot, broadcast
- **Sync primitives** — Mutex, RwLock, Semaphore
- **Task spawning** — `tokio::spawn`

### The #[tokio::main] Macro

```rust
#[tokio::main]
async fn main() {
    // This runs on the Tokio runtime
    let result = fetch_metadata("https://example.com").await;
}
```

The macro expands to:

```rust
fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let result = fetch_metadata("https://example.com").await;
        });
}
```

## Common Async Patterns

### Join — Run Tasks Concurrently

```rust
let (epub, html) = tokio::join!(
    create_epub(&meta, &chapters, &tmp_dir),
    create_html_bundle(&meta, &chapters, &tmp_dir),
);
```

Both tasks run concurrently. `join!` waits for both to complete.

### Select — Race Multiple Tasks

```rust
tokio::select! {
    result = fetch_metadata(url) => { /* metadata arrived */ }
    _ = tokio::time::sleep(Duration::from_secs(30)) => { /* timeout */ }
}
```

Whichever future completes first wins. The other is cancelled.

### Spawn — Independent Tasks

```rust
tokio::spawn(async {
    // This runs independently
    process_fic(url).await;
});
// Main task continues without waiting
```

### Mutex — Shared Mutable State

```rust
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(HashMap::new()));

// In a handler
let mut map = data.lock().await;
map.insert("key".to_string(), "value".to_string());
// Lock is released when `map` goes out of scope
```

### Semaphore — Rate Limiting

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(10));  // Max 10 concurrent

let permit = semaphore.acquire().await?;
// Do work
drop(permit);  // Release the permit
```

## Send and Sync

Rust's ownership system extends to async code. Futures must be `Send` to be moved between threads:

```rust
// This future is Send — it can run on any thread
async fn handler() -> String {
    let data = fetch_data().await;
    process(data).await
}

// This future is NOT Send — it holds a non-Send type
async fn bad_handler() {
    let rc = std::rc::Rc::new(5);  // Rc is not Send
    // Error if this future needs to be Send
}
```

**Rule of thumb:** Don't hold `Rc`, `RefCell`, or raw pointers across `.await` points.

## Blocking Operations

Some operations can't be async (like file I/O or CPU-intensive work). Use `spawn_blocking`:

```rust
let result = tokio::task::spawn_blocking(move || {
    // This runs on a dedicated blocking thread pool
    std::fs::read_to_string(path)
}).await?;
```

## Watch Out!

**Don't block the runtime!** Long-running sync operations in async code block all other tasks.

**Avoid unwrap() in async code!** A panic in an async task kills only that task, but the error message is unhelpful. Use `?` or `unwrap_or_else`.

**Clone Arc, don't clone data!** `Arc::clone` is cheap (atomic increment). Cloning the data inside is expensive.

## Summary

Async Rust enables efficient I/O by suspending tasks while waiting. The Tokio runtime manages task scheduling, timers, and I/O. Understanding `Send`/`Sync`, `spawn_blocking`, and common patterns like `join!` and `select!` is essential for building concurrent applications.

---

# Extended Chapter: The Fanfiction Scraping Pipeline

## Complete Scraping Workflow

When FicHub processes a request for a fanfiction URL, it goes through a complex pipeline. Let's trace every step in detail.

### Phase 1: URL Resolution (Step 1-4)

**Step 1: HTTP Request Arrives**

A user sends:
```
GET /api/v0/epub?q=https://archiveofourown.org/works/12345678 HTTP/1.1
Host: fichub.example.com
```

The Axum router matches this to `epub_handler`.

**Step 2: Parameter Extraction**

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,
    pub automated: Option<String>,
    pub format: Option<String>,
}
```

Axum parses the query string into this struct. `q` is the fanfiction URL.

**Step 3: Validation**

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}
if params.automated.as_deref() == Some("true") {
    return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
}
```

**Step 4: Scraper Lookup**

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

The registry iterates through all scrapers and calls `can_handle` on each. For an AO3 URL, `Ao3Scraper::can_handle` returns true.

### Phase 2: Metadata Extraction (Step 5-6)

**Step 5: HTTP Request to AO3**

```rust
let fic_url = format!("{BASE_URL}/works/{work_id}?view_full_work=true");
let response = client
    .get(&fic_url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

This makes an HTTP GET request to AO3's servers. The `?view_full_work=true` parameter tells AO3 to render all chapters on one page.

**Step 6: HTML Parsing**

```rust
let html = response.text().await?;
let document = Html::parse_document(&html);

let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());
```

The `scraper` crate parses the HTML into a DOM tree, then CSS selectors extract specific elements.

### Phase 3: Database Operations (Step 6-8)

**Step 7: Upsert Fic Info**

```rust
sqlx::query(
    r#"INSERT INTO fic_info (id, title, author, words, chapters, status, source, ...)
       VALUES ($1, $2, $3, $4, $5, $6, $7, ...)
       ON CONFLICT (id) DO UPDATE SET
           title = EXCLUDED.title,
           words = EXCLUDED.words,
           ..."#,
)
.bind(&meta.url_id)
.bind(&meta.title)
// ... more binds ...
.execute(&state.db)
.await?;
```

The `ON CONFLICT DO UPDATE` clause ensures the record is created or updated.

**Step 8: Auto-populate Tags**

```rust
if let Ok(extracted_tags) = scraper.extract_tags(&state.http_client, query).await {
    for tag in &extracted_tags {
        if let Ok(resolution) = crate::tags::resolve::resolve_tag(
            &state.db, &tag.name, tag.tag_type_id,
        ).await {
            let _ = queries::upsert_fic_tag(
                &state.db, &meta.url_id, resolution.tag_id,
                &std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            ).await;
        }
    }
}
```

Tags extracted from AO3's HTML are automatically added to the database.

### Phase 4: Cache Check (Step 9-13)

**Step 9: Version Computation**

```rust
let version_bump = queries::get_fic_version_bump(&state.db, &meta.url_id).await?.unwrap_or(0);
let version = state.config.export_version + version_bump;
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
```

**Step 10-11: Cache Lookup**

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Cache hit — build response and return
    return Ok(build_response(&export_log, &meta));
}
```

The cache lookup checks if the same EPUB was already generated with the same input hash.

**Step 12: Semaphore Acquisition**

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

This prevents duplicate concurrent generation of the same EPUB.

**Step 13: Double-Check**

```rust
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Another request generated it while we were waiting
    return Ok(build_response(&export_log, &meta));
}
```

### Phase 5: Content Fetching (Step 14)

**Step 14: Fetch Chapters**

```rust
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

For AO3, this fetches the full work page and extracts each chapter's content.

For FF.net, this loops through each chapter URL individually:
```rust
for i in 1..=meta.chapters {
    let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
    let response = client.get(&url).send().await?;
    // ... extract content ...
}
```

### Phase 6: File Generation (Step 15-17)

**Step 15: EPUB Generation**

```rust
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

This creates a well-formatted EPUB file with metadata, CSS, introduction page, and chapters.

**Step 16: Move to Cache**

```rust
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

**Step 17: Generate HTML Bundle**

```rust
let (html_path, html_hash) = export::html_bundle::create_html_bundle(&meta, &chapters, &state.config.tmp_dir).await?;
let html_cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Html, &meta.url_id, &html_hash);
cache::disk::move_to_cache(&html_path, &html_cache_dest)?;
```

### Phase 7: Response Building

```rust
Ok(Json(json!({
    "err": 0,
    "q": query,
    "url_id": meta.url_id,
    "slug": generate_slug(&meta.title, &meta.url_id),
    "meta": build_meta_json(&meta),
    "hashes": {
        "epub": epub_hash,
        "html": html_hash,
    },
    "urls": {
        "epub": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
        "html": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
    },
    "epub_url": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash),
    "html_url": format!("/cache/html/{}?h={}", meta.url_id, html_hash),
    "info": build_info_string(&meta).0,
    "notes": Vec::<String>::new(),
})))
```

## Error Scenarios

### Network Error

If AO3 is unreachable:
```
ScrapeError::Network("connection refused") → AppError::ScrapeError → 502 Bad Gateway
```

### Parse Error

If AO3's HTML structure changed:
```
ScrapeError::ParseError("could not extract title") → AppError::ScrapeError → 502 Bad Gateway
```

### Blacklisted Fic

If the fic is on the blacklist:
```
AppError::BadRequest(-7, "fic is blacklisted") → 400 Bad Request
```

### Rate Limited

If the user has made too many requests:
```
AppError::RateLimited(3600) → 429 Too Many Requests
```

## Performance Characteristics

| Phase | Time | Notes |
|-------|------|-------|
| URL resolution | ~1ms | Regex + HashMap lookup |
| Metadata extraction | ~500ms-2s | HTTP request to fanfiction site |
| Database upsert | ~5ms | Single INSERT/UPDATE |
| Cache check | ~2ms | Single SELECT |
| Chapter fetching | ~2-30s | Depends on chapter count |
| EPUB generation | ~1-5s | Depends on story length |
| HTML generation | ~1-3s | Depends on story length |
| **Total (cache hit)** | **~10ms** | Database lookups only |
| **Total (cache miss)** | **~10-60s** | Full pipeline |

## Summary

The scraping pipeline is a carefully orchestrated sequence of network requests, HTML parsing, database operations, and file generation. Understanding each phase helps you optimize performance and debug issues.


---

# Supplementary Content: Practical Guides and Extended Concepts

---

# Extended Guide: Database Schema Design

## Why Schema Design Matters

The database schema is the foundation of any data-driven application. A well-designed schema ensures data integrity, query performance, and maintainability. A poorly designed schema leads to slow queries, data corruption, and frustrating development experiences.

FicHub's schema went through several iterations. Let's examine the design decisions behind each table.

## Core Tables

### fic_info — The Central Table

The `fic_info` table is the heart of FicHub. Every fic that's ever been scraped gets a row here:

```sql
CREATE TABLE fic_info (
    id VARCHAR(128) PRIMARY KEY,           -- Deterministic hash-based ID
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,                   -- Site-specific story ID
    chapters INT4 NOT NULL,
    words INT8 NOT NULL,
    description TEXT NOT NULL,
    fic_created TIMESTAMPTZ NOT NULL,       -- When the fic was published
    fic_updated TIMESTAMPTZ NOT NULL,       -- When the fic was last updated
    status TEXT NOT NULL,                   -- "ongoing", "complete", etc.
    source TEXT NOT NULL,                   -- Original URL
    extra_meta TEXT,                        -- Additional metadata (JSON)
    raw_extended_meta TEXT,                 -- Raw extended metadata
    source_id INT8,                         -- Site identifier (1=AO3, 2=FF.net)
    author_id INT8,                         -- Site-specific author ID
    content_hash VARCHAR(256)               -- Hash of story content
);
```

Design decisions:
1. **VARCHAR(128) for id** — The hash-based ID is 12 hex chars, but we use 128 to leave room for future changes
2. **TEXT for description** — Descriptions can be very long and are often HTML
3. **TIMESTAMPTZ** — Always store timestamps with timezone information
4. **content_hash** — Enables cache invalidation when stories are updated
5. **Separate created/updated from fic_created/fic_updated** — Our timestamps vs the story's timestamps

### export_log — The Cache Key Table

```sql
CREATE TABLE export_log (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    version INT NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(url_id, version, etype, input_hash)
);
```

The composite key `(url_id, version, etype, input_hash)` is the cache key:
- **url_id** — Which fic
- **version** — Export version (bumped on format changes)
- **etype** — Export type (epub, html, mobi, pdf)
- **input_hash** — Hash of the input data (changes when story is updated)

### Blacklist Tables

```sql
CREATE TABLE fic_blacklist (
    url_id VARCHAR(128) REFERENCES fic_info(id),
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(url_id, reason)
);

CREATE TABLE author_blacklist (
    source_id INT8 NOT NULL,
    author_id INT8 NOT NULL,
    reason INT NOT NULL DEFAULT 1,
    UNIQUE(source_id, author_id, reason)
);
```

Blacklists use numeric reasons:
- **5** — DMCA/legal takedown
- **6** — Greylisted (metadata shown, no download)
- **7** — Content policy violation
- **8** — Other

The greylist (reason 6) is a middle ground — users can see the metadata but can't download the file.

## Recommender Tables

### fic_works

```sql
CREATE TABLE fic_works (
    url_id VARCHAR(128) PRIMARY KEY REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    site_work_id VARCHAR(255) NOT NULL,
    favouriter_count INT4 NOT NULL DEFAULT 0,
    first_favourite_scraped TIMESTAMPTZ,
    last_favourite_scraped TIMESTAMPTZ,
    last_cooccur_update TIMESTAMPTZ,
    UNIQUE(site_domain, site_work_id)
);
```

This table extends `fic_info` with recommendation-specific data. The `ON DELETE CASCADE` ensures cleanup when a fic is deleted.

### fic_bookmarks

```sql
CREATE TABLE fic_bookmarks (
    user_hash VARCHAR(64) NOT NULL,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    first_seen TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_hash, url_id)
);
```

User hashes are SHA-256 of profile URLs — one-way, anonymous, but deterministic.

### fic_bookmark_cooccur

```sql
CREATE TABLE fic_bookmark_cooccur (
    work_a VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    work_b VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    site_domain VARCHAR(255) NOT NULL,
    cooccur_count INT4 NOT NULL DEFAULT 1,
    last_updated TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (work_a, work_b),
    CHECK (work_a < work_b)
);
```

The `CHECK (work_a < work_b)` constraint canonicalizes the pair order, preventing duplicates.

### precomputed_recommendations

```sql
CREATE TABLE precomputed_recommendations (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    recommended_url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    score REAL NOT NULL,
    rank SMALLINT NOT NULL,
    computed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, recommended_url_id)
);
```

This is a materialized cache — computed results stored for fast retrieval.

## Tagging Tables

### tags

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE COLLATE "C",
    tag_type_id SMALLINT NOT NULL REFERENCES tag_types(id),
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

The `COLLATE "C"` makes comparisons case-sensitive. This is intentional — "Harry Potter" and "harry potter" are different tags.

### fic_tags

```sql
CREATE TABLE fic_tags (
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    added_by_ip INET NOT NULL DEFAULT '0.0.0.0',
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id)
);
```

The junction table connecting fics to tags. The `score` column is automatically updated by a PostgreSQL trigger.

### fic_tag_votes

```sql
CREATE TABLE fic_tag_votes (
    url_id VARCHAR(128) NOT NULL,
    tag_id INTEGER NOT NULL,
    voter_ip INET NOT NULL,
    value SMALLINT NOT NULL CHECK (value IN (-1, 1)),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (url_id, tag_id, voter_ip),
    FOREIGN KEY (url_id, tag_id) REFERENCES fic_tags(url_id, tag_id) ON DELETE CASCADE
);
```

Each vote is either +1 (upvote) or -1 (downvote). The composite primary key ensures one vote per IP per tag per fic.

### The Score Trigger

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

Three triggers fire on INSERT, UPDATE, and DELETE. This ensures the score is always accurate without application-level logic.

## OPDS Tables

### opds_shelves

```sql
CREATE TABLE opds_shelves (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

### opds_shelf_items

```sql
CREATE TABLE opds_shelf_items (
    shelf_id INTEGER NOT NULL REFERENCES opds_shelves(id) ON DELETE CASCADE,
    url_id VARCHAR(128) NOT NULL REFERENCES fic_info(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, url_id)
);
```

## Schema Evolution

FicHub's schema evolved through four migrations:

1. **001_initial_schema** — Core tables (fic_info, request_log, export_log, blacklists)
2. **002_recommender** — Recommendation engine (fic_works, bookmarks, co-occurrence, suggestions, votes)
3. **003_tagging** — Tag system v3 (tags, aliases, fic_tags, votes, flags, full-text search)
4. **004_shelves** — OPDS shelves

Each migration is idempotent (`CREATE TABLE IF NOT EXISTS`) and can be run multiple times safely.

## Summary

FicHub's database schema is designed for:
- **Data integrity** — Foreign keys, unique constraints, check constraints
- **Performance** — Appropriate indexes, partial indexes, composite indexes
- **Maintainability** — Clear naming, consistent conventions, documented purpose
- **Evolution** — Idempotent migrations, backward-compatible changes

---

# Extended Guide: Error Handling Patterns

## Why Error Handling Matters

In a web server, errors are not exceptional — they're expected. Users send bad URLs, sites go down, databases lose connections. A well-designed error handling system ensures the server stays running and users get helpful error messages.

## FicHub's Error Architecture

### The Error Enum

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
```

Each variant represents a different class of error. The associated data varies:
- **BadRequest** — Error code + message (code is used by the frontend)
- **RateLimited** — Retry-after seconds
- **NotFound** — What wasn't found
- **Internal** — Generic server error
- **ScrapeError** — What went wrong during scraping
- **ExportError** — What went wrong during export
- **Database** — What went wrong with the database
- **CacheError** — What went wrong with Redis

### Display Implementation

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

### IntoResponse Implementation

This is where errors become HTTP responses:

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

Key design decisions:
1. **BadRequest returns the error code** — The frontend can handle specific error types
2. **RateLimited includes retry_after** — The client knows when to retry
3. **ScrapeError returns 502** — The error originated from an upstream site
4. **Internal, Database, CacheError, ExportError return 500** — The client gets a generic message, but the server logs the details
5. **NotFound returns 404** — Standard HTTP semantics

### From Implementations

The `?` operator works because `AppError` implements `From` for various error types:

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

## Error Handling in Handlers

### Simple Pattern

```rust
async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // ... more processing ...

    Ok(Json(json!({"err": 0, "data": result})))
}
```

### Pattern Matching Errors

```rust
match scraper.lookup(&state.http_client, query).await {
    Ok(meta) => {
        // Process metadata
    }
    Err(ScrapeError::NotFound) => {
        return Ok(Json(json!({"err": -5, "msg": "story not found"})));
    }
    Err(ScrapeError::Blocked) => {
        return Ok(Json(json!({"err": -6, "msg": "site is blocking requests"})));
    }
    Err(e) => {
        return Err(AppError::ScrapeError(e.to_string()));
    }
}
```

### Contextual Errors

Add context to errors for debugging:

```rust
let pool = db::init_pool(&config.database_url)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to connect to database: {}", e)))?;
```

## Error Logging

FicHub logs errors at different levels:

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);  // Always logged
    // ...
}
AppError::ScrapeError(msg) => {
    // NOT logged here — logged at the handler level if needed
    // ...
}
AppError::BadRequest(code, msg) => {
    // NOT logged — client errors are expected
    // ...
}
```

The principle: log server errors (500s), don't log client errors (400s).

## Summary

FicHub's error handling uses a centralized enum with variants for each error class, `From` implementations for automatic conversion, `IntoResponse` for HTTP responses, and appropriate logging levels.

---

# Extended Guide: Configuration Management

## Environment Variables

FicHub uses environment variables for all configuration. This follows the twelve-factor app methodology.

### Loading Configuration

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        // ... 40+ more settings ...
    }
}
```

### Required vs Optional Settings

**Required** — Server can't start without them:
- `DATABASE_URL`
- `REDIS_URL`

**Optional with defaults** — Sensible defaults provided:
- `CACHE_DIR` → `./cache`
- `PORT` → `3000`
- `RUST_LOG` → `info,fichub=debug`

**Optional without defaults** — Feature-specific:
- `SECONDARY_CACHE_DIR` → `None`
- `CALIBRE_CONTAINER` → `""`
- `CURATOR_TOKEN` → `None`

### The dotenvy Crate

FicHub loads a `.env` file if present:

```rust
dotenvy::dotenv().ok();
```

The `.ok()` silently ignores missing files. The `.env` file is loaded before reading environment variables, so its values serve as defaults.

### The .env File

```bash
# Required
DATABASE_URL=postgres://fichub:fichub@localhost/fichub
REDIS_URL=redis://localhost/0

# Optional with defaults
CACHE_DIR=./cache
TMP_DIR=./tmp
PORT=3000
FRONTEND_DIR=./frontend/build
RUST_LOG=info,fichub=debug

# Recommender settings
REC_DEFAULT_DELAY_SECS=5
REC_MAX_FAVOURITE_PAGES=3
REC_MAX_RECOMMENDATIONS=20
REC_VOTING_BOOST_GAMMA=0.2
REC_CACHE_TTL_HOURS=12

# Tagging settings
CURATOR_TOKEN=my-secret-token
TAG_HIDDEN_THRESHOLD=-3
TAG_SUBMIT_LIMIT_PER_HOUR=10
TAG_VOTE_LIMIT_PER_HOUR=20
SEARCH_MAX_PER_PAGE=50

# OPDS
OPDS_SHELF_TOKEN=fichub
```

### Parsing Configuration Values

Each setting is parsed from a string to its appropriate type:

```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

The pattern:
1. `std::env::var("PORT")` — Get the string value
2. `.unwrap_or_else(|_| "3000".to_string())` — Default if not set
3. `.parse()` — Convert to the target type
4. `.unwrap_or(3000)` — Default if parsing fails

### Complex Configuration

Some settings are more complex:

```rust
// JSON hash map
let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
    .unwrap_or_else(|_| "{}".to_string());
let rec_site_rate_limits: HashMap<String, u64> =
    serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();

// Comma-separated list
let trusted_proxies = std::env::var("TRUSTED_PROXIES")
    .unwrap_or_default()
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();

// Multi-line structured data
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
```

## Configuration Testing

FicHub has thorough configuration tests:

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
    assert_eq!(config.cache_dir, PathBuf::from("./cache"));
    assert_eq!(config.app_port, 3000);
    assert_eq!(config.rec_default_delay_secs, 5);
    assert_eq!(config.rec_voting_boost_gamma, 0.2);
    assert!(config.rec_precompute_enabled);
}
```

The `EnvGuard` struct automatically cleans up environment variables when it goes out of scope, preventing test pollution.

## Summary

FicHub's configuration system uses environment variables with sensible defaults, supports complex data types (JSON, comma-separated lists), and has comprehensive tests. The `dotenvy` crate provides a convenient `.env` file for development.


---

# Supplementary Content: Design Patterns and Architecture

---

# Design Patterns Used in FicHub

## The Strategy Pattern

FicHub uses the Strategy pattern extensively for its pluggable scraper system. The `SiteScraper` trait defines the interface, and each scraper provides a different implementation:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

This pattern makes it trivial to add new scrapers — just implement the trait and register it. The rest of the system doesn't need to change.

## The Registry Pattern

The `ScraperRegistry` is a classic Registry pattern — a central lookup table that maps identifiers (URLs) to implementations (scrapers):

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

impl ScraperRegistry {
    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }
}
```

This decouples the rest of the application from the specific scrapers. New scrapers are added to the registry without modifying any other code.

## The Repository Pattern

FicHub's database layer follows the Repository pattern. Each query function is a standalone function that takes a connection pool:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>("SELECT * FROM fic_info WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    sqlx::query("INSERT INTO fic_info ... ON CONFLICT ... DO UPDATE ...")
        .bind(&fic.id)
        // ...
        .execute(pool)
        .await?;
    Ok(())
}
```

This keeps database logic centralized and testable.

## The Builder Pattern

FicHub uses the Builder pattern for constructing complex objects:

### reqwest Client

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

### SQLx Query Builder

```rust
let mut qb = QueryBuilder::<Postgres>::new("SELECT fi.* FROM fic_info fi WHERE 1=1");

if let Some(ref q) = self.params.q {
    qb.push(" AND fi.text_search @@ plainto_tsquery('english', ");
    qb.push_bind(q);
    qb.push(")");
}

if let Some(min) = self.params.min_words {
    qb.push(" AND fi.words >= ");
    qb.push_bind(min);
}
```

### Axum Router

```rust
let app = Router::new()
    .route("/api/", get(api_docs_handler))
    .route("/api/v0/epub", get(epub_handler))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .with_state(state);
```

Each method call returns a modified builder, allowing fluent configuration.

## The Observer Pattern (via PostgreSQL Triggers)

FicHub uses PostgreSQL triggers as a form of the Observer pattern. When a vote is inserted, updated, or deleted, the trigger automatically updates the tag score:

```sql
CREATE TRIGGER trg_fic_tag_vote_insert
    AFTER INSERT ON fic_tag_votes
    FOR EACH ROW EXECUTE FUNCTION update_fic_tag_score();
```

The application doesn't need to manually update scores — the database handles it automatically.

## The Factory Pattern

The `ScraperRegistry::new()` method is a factory that creates all known scrapers:

```rust
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
```

## The Proxy Pattern

FicHub's caching system acts as a proxy — it intercepts requests and serves cached responses when available, only hitting the real data source (scraping) when the cache is empty:

```rust
// Check cache first
let cached = queries::find_export_log(&state.db, &url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_response(&export_log));
}

// Cache miss — do the real work
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
let (epub_path, hash) = export::epub::create_epub(&meta, &chapters, &tmp_dir).await?;
```

## The Chain of Responsibility Pattern

The middleware stack in Axum is a Chain of Responsibility — each layer can handle the request or pass it to the next:

```rust
Router::new()
    .route("/api/v0/epub", get(epub_handler))
    .layer(CorsLayer::permissive())    // Layer 1: Handle CORS
    .layer(TraceLayer::new_for_http()) // Layer 2: Log request
```

Request flows: CorsLayer → TraceLayer → Router → handler

## The Template Method Pattern

The EPUB and HTML generation follow a template method pattern — the overall structure is fixed, but the content varies:

```rust
// Fixed structure
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;
builder.metadata("title", &meta.title)?;
builder.stylesheet(css.as_bytes())?;
builder.add_content(introduction_page)?;

// Variable content
for chapter in chapters {
    builder.add_content(chapter_page(&chapter))?;
}

// Fixed finalization
builder.generate(file)?;
```

## The Double-Check Locking Pattern

FicHub uses double-check locking for cache validation:

```rust
// First check (no lock)
let cached = queries::find_export_log(&db, &url_id, version, "epub", &hash).await?;
if cached.is_some() { return Ok(cached); }

// Acquire lock
let sem = get_export_semaphore(&semaphores, &url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check (with lock)
let cached = queries::find_export_log(&db, &url_id, version, "epub", &hash).await?;
if cached.is_some() { return Ok(cached); }

// Do work
generate_epub().await?;
```

This pattern minimizes lock contention while preventing duplicate work.

## The Circuit Breaker Pattern

FicHub's rate limiter acts as a circuit breaker. When a site is blocking requests, the rate limiter returns `RateLimitResult::Blocked`:

```rust
if self.is_datacenter_ip(ip) {
    return RateLimitResult::Blocked;
}
```

This prevents the system from continuing to hammer a site that's rejecting requests.

## Summary

FicHub employs many classic design patterns: Strategy (scrapers), Registry (scraper lookup), Repository (database queries), Builder (configuration), Observer (triggers), Factory (registry creation), Proxy (caching), Chain of Responsibility (middleware), Template Method (export generation), Double-Check Locking (cache validation), and Circuit Breaker (rate limiting). Understanding these patterns makes the codebase easier to navigate and extend.

---

# Architectural Overview

## Layer Architecture

FicHub follows a layered architecture:

```
┌─────────────────────────────────────────┐
│           HTTP Layer (Axum)              │
│  Routes, Extractors, Middleware          │
├─────────────────────────────────────────┤
│         Application Layer               │
│  Export Flow, Recommendations, Search    │
├─────────────────────────────────────────┤
│          Domain Layer                   │
│  Scrapers, Exporters, Tag Resolution     │
├─────────────────────────────────────────┤
│         Data Access Layer               │
│  SQLx Queries, Redis Operations          │
├─────────────────────────────────────────┤
│        Infrastructure Layer             │
│  PostgreSQL, Redis, File System          │
└─────────────────────────────────────────┘
```

Each layer depends only on the layer below it. The HTTP layer doesn't know about PostgreSQL — it goes through the Application layer, which goes through the Data Access layer.

## Module Organization

```
src/
├── main.rs          → Entry point (infra layer)
├── server.rs        → HTTP setup (HTTP layer)
├── config.rs        → Configuration (infra layer)
├── error.rs         → Error types (cross-cutting)
├── db/              → Data access layer
│   ├── mod.rs       → Connection pool
│   ├── models.rs    → Data structures
│   └── queries.rs   → SQL queries
├── scrape/          → Domain layer (scraping)
│   ├── mod.rs       → Trait definitions
│   ├── registry.rs  → Scraper registry
│   └── sites/       → Site-specific scrapers
├── export/          → Domain layer (export)
│   ├── mod.rs       → Export types
│   ├── epub.rs      → EPUB generation
│   ├── html_bundle.rs → HTML generation
│   └── convert.rs   → Format conversion
├── cache/           → Domain layer (caching)
│   ├── mod.rs       → Cache types
│   └── disk.rs      → Disk operations
├── limiter/         → Domain layer (rate limiting)
│   ├── mod.rs       → Rate limiter trait
│   └── redis_bucket.rs → Token bucket impl
├── recommender/     → Application layer
│   ├── mod.rs       → Recommender types
│   ├── engine.rs    → Recommendation computation
│   ├── worker.rs    → Background collection
│   └── routes.rs    → API handlers
├── search/          → Application layer
│   ├── mod.rs       → Search modules
│   ├── builder.rs   → Query builder
│   └── routes.rs    → Search handler
├── tags/            → Application layer
│   ├── mod.rs       → Tag modules
│   ├── resolve.rs   → Tag resolution
│   ├── voting.rs    → Vote management
│   ├── routes.rs    → Tag handlers
│   └── curator.rs   → Curator tools
├── routes/          → HTTP layer
│   ├── mod.rs       → Route modules
│   ├── export.rs    → Export handler
│   ├── meta.rs      → Metadata handler
│   ├── cache_download.rs → Download handlers
│   ├── api_docs.rs  → API documentation
│   └── opds/        → OPDS catalog
└── frontend/        → Frontend module
    └── mod.rs       → Constants
```

## Data Flow

### Request Flow

```
HTTP Request
    ↓
Router (URL matching)
    ↓
Middleware (CORS, Logging)
    ↓
Handler (Parameter extraction)
    ↓
Domain Logic (Business rules)
    ↓
Data Access (Database/Redis)
    ↓
Response (JSON/XML)
```

### Scraping Flow

```
URL Input
    ↓
Scraper Selection (Registry lookup)
    ↓
HTTP Request (to fanfiction site)
    ↓
HTML Parsing (scraper crate)
    ↓
Data Extraction (CSS selectors)
    ↓
Metadata Assembly (FicMetadata)
    ↓
Database Storage (UPSERT)
```

### Export Flow

```
Metadata
    ↓
Cache Check (export_log table)
    ├── Hit → Return cached URLs
    └── Miss ↓
        ↓
Semaphore Acquisition (prevent duplicates)
        ↓
Double-Check Cache
        ├── Hit → Return cached URLs
        └── Miss ↓
            ↓
Chapter Fetching (from fanfiction site)
            ↓
EPUB Generation (epub-builder)
            ↓
HTML Generation (template)
            ↓
Cache Storage (move to cache dir)
            ↓
Database Recording (export_log)
            ↓
Response Building (JSON with URLs)
```

## Concurrency Model

FicHub's concurrency model is built on Tokio:

```
┌─────────────────────────────────────┐
│           Tokio Runtime             │
│  ┌─────────────────────────────┐    │
│  │     Worker Threads (N)      │    │
│  │  ┌───┐ ┌───┐ ┌───┐ ┌───┐  │    │
│  │  │ T │ │ T │ │ T │ │ T │  │    │
│  │  └───┘ └───┘ └───┘ └───┘  │    │
│  └─────────────────────────────┘    │
│                                     │
│  ┌─────────────────────────────┐    │
│  │     Blocking Thread Pool    │    │
│  │  ┌───┐ ┌───┐ ┌───┐        │    │
│  │  │ B │ │ B │ │ B │        │    │
│  │  └───┘ └───┘ └───┘        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

- **Async tasks** handle HTTP requests, database queries, and Redis operations
- **Blocking tasks** handle CPU-intensive work (EPUB generation, file I/O)
- **Shared state** is protected by `Arc` (read-only) or `Mutex`/`RwLock` (read-write)

## Summary

FicHub's architecture is clean, layered, and well-organized. Each module has a single responsibility, dependencies flow downward, and the concurrency model leverages Tokio's async runtime efficiently. Understanding this architecture makes it easy to navigate the codebase and add new features.


---

# Supplementary Content: Comprehensive Rust Tutorial

---

# Rust for FicHub Developers

## Variables and Binding

In Rust, `let` creates a binding. The binding is immutable by default:

```rust
let name = "FicHub";       // immutable string slice
let version = 1;            // immutable integer
let pi = 3.14;              // immutable float
```

To make a variable mutable, add `mut`:

```rust
let mut counter = 0;
counter += 1;               // OK — variable is mutable
```

Why immutable by default? Because immutability makes your code safer. When you see `let x = 5`, you know `x` will always be `5`. No function, no thread, no other code can change it. This eliminates an entire class of bugs.

Shadowing is when you re-declare a variable with the same name:

```rust
let x = 5;
let x = x + 1;     // x is now 6
let x = x * 2;     // x is now 12
```

Shadowing creates a new variable that happens to have the same name. This is useful for transforming values:

```rust
let name = "  FicHub  ";
let name = name.trim();    // "FicHub"
```

Type annotations are optional when Rust can infer the type:

```rust
let x = 5;              // Rust infers i32
let y = 5.0;            // Rust infers f64
let z: i64 = 5;         // Explicit annotation
let words: Vec<i32> = vec![1, 2, 3];  // Explicit annotation needed for collections
```

## Functions

Functions are defined with `fn`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b      // Last expression (no semicolon) is the return value
}
```

Key rules:
- Parameters must have type annotations
- Return type comes after `->`
- The last expression (without semicolon) is the return value
- You can use `return` explicitly, but idiomatic Rust omits it

Functions can be public (accessible from other modules) or private:

```rust
pub fn public_function() -> String { "visible everywhere".to_string() }
fn private_function() -> String { "only in this module".to_string() }
```

## Closures

Closures are anonymous functions:

```rust
let add_one = |x: i32| -> i32 { x + 1 };
let result = add_one(5);   // 6

// If the body is a single expression, omit braces:
let add_one = |x| x + 1;

// Closures can capture variables from their environment:
let greeting = "Hello";
let greet = |name| format!("{}, {}!", greeting, name);
println!("{}", greet("World"));  // "Hello, World!"
```

Closures are used extensively with iterators:

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Map: transform each element
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// Filter: keep elements matching a predicate
let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

// Fold: accumulate a single value
let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);
```

## Structs

Structs create custom types with named fields:

```rust
struct Person {
    name: String,
    age: u32,
    email: String,
}

let person = Person {
    name: "Alice".to_string(),
    age: 30,
    email: "alice@example.com".to_string(),
};

// Access fields with dot notation
println!("{} is {} years old", person.name, person.age);

// Mutable struct fields
let mut person = person;
person.age = 31;
```

Struct methods are defined in `impl` blocks:

```rust
impl Person {
    // Associated function (like a constructor)
    fn new(name: &str, age: u32, email: &str) -> Self {
        Person {
            name: name.to_string(),
            age,
            email: email.to_string(),
        }
    }

    // Method (takes &self)
    fn introduction(&self) -> String {
        format!("Hi, I'm {} and I'm {} years old.", self.name, self.age)
    }

    // Mutable method (takes &mut self)
    fn have_birthday(&mut self) {
        self.age += 1;
    }
}

let mut person = Person::new("Alice", 30, "alice@example.com");
println!("{}", person.introduction());
person.have_birthday();
```

## Enums

Enums represent a value that can be one of several variants:

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

let dir = Direction::North;
```

Enums can carry data with each variant:

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    Color(u8, u8, u8),
}

let msg = Message::Write("hello".to_string());
let msg = Message::Color(255, 0, 0);
let msg = Message::Move { x: 10, y: 20 };
```

Pattern matching with `match`:

```rust
match msg {
    Message::Quit => println!("Quitting"),
    Message::Move { x, y } => println!("Moving to ({}, {})", x, y),
    Message::Write(text) => println!("Writing: {}", text),
    Message::Color(r, g, b) => println!("Color: #{:02x}{:02x}{:02x}", r, g, b),
}
```

`match` must be exhaustive — every variant must be handled. The `_` pattern catches anything not explicitly matched:

```rust
match msg {
    Message::Quit => println!("Quitting"),
    _ => println!("Something else"),
}
```

## Option and Result

Rust doesn't have null. Instead, it has `Option`:

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some("Alice".to_string())
    } else {
        None
    }
}

// Handling Option
match find_user(1) {
    Some(name) => println!("Found: {}", name),
    None => println!("User not found"),
}

// Shorthand
if let Some(name) = find_user(1) {
    println!("Found: {}", name);
}

// Chaining
let upper = find_user(1).map(|name| name.to_uppercase());
```

`Result` represents success or failure:

```rust
fn parse_number(s: &str) -> Result<i32, String> {
    s.parse::<i32>().map_err(|e| format!("Parse error: {}", e))
}

// Handling Result
match parse_number("42") {
    Ok(n) => println!("Parsed: {}", n),
    Err(e) => println!("Error: {}", e),
}

// The ? operator propagates errors
fn calculate() -> Result<i32, String> {
    let x = parse_number("42")?;
    let y = parse_number("10")?;
    Ok(x + y)
}
```

## Traits

Traits define shared behavior:

```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

// Trait objects for dynamic dispatch
fn print_summary(item: &dyn Summary) {
    println!("{}", item.summarize());
}

// Generic functions with trait bounds
fn summarize_all(items: &[&dyn Summary]) {
    for item in items {
        println!("{}", item.summarize());
    }
}
```

## Ownership and Borrowing

Every value has exactly one owner. When the owner goes out of scope, the value is dropped:

```rust
{
    let s1 = String::from("hello");
    let s2 = s1;      // Ownership moves to s2
    // s1 is no longer valid
    println!("{}", s2);
}   // s2 is dropped here
```

Borrowing lets you use a value without taking ownership:

```rust
fn print_length(s: &String) {    // Borrows s
    println!("Length: {}", s.len());
}

let s = String::from("hello");
print_length(&s);                 // Pass a reference
println!("{}", s);                // s is still valid
```

Two kinds of borrows:
- `&T` — Shared borrow (many readers)
- `&mut T` — Mutable borrow (one writer)

```rust
fn push_world(s: &mut String) {
    s.push_str(", world!");
}

let mut s = String::from("hello");
push_world(&mut s);
println!("{}", s);   // "hello, world!"
```

## Enums with Data — AppError

FicHub's error type demonstrates enums with data:

```rust
enum AppError {
    BadRequest(i32, String),    // Code + message
    RateLimited(u64),           // Retry-after seconds
    NotFound(String),           // What wasn't found
    Internal(String),           // Error message
    ScrapeError(String),        // Scraping error
    ExportError(String),        // Export error
    Database(String),           // Database error
    CacheError(String),         // Cache error
}
```

Each variant can have different data. `BadRequest` has a code and message. `RateLimited` has just a number. This is much more expressive than a generic "error code + message" approach.

## Iterators

Rust iterators are lazy — they don't do anything until consumed:

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Lazy — does nothing yet
let doubled = numbers.iter().map(|x| x * 2);

// Consuming — actually runs the iterator
let result: Vec<i32> = doubled.collect();
```

Common iterator methods:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// map: transform each element
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// filter: keep matching elements
let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

// fold: accumulate a value
let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);

// any: check if any element matches
let has_even = numbers.iter().any(|x| x % 2 == 0);

// find: find first matching element
let first_even = numbers.iter().find(|x| x % 2 == 0);

// count: count matching elements
let even_count = numbers.iter().filter(|x| x % 2 == 0).count();

// take and skip
let first_three: Vec<&i32> = numbers.iter().take(3).collect();
let skip_three: Vec<&i32> = numbers.iter().skip(3).collect();

// chain: combine two iterators
let a = vec![1, 2];
let b = vec![3, 4];
let combined: Vec<&i32> = a.iter().chain(b.iter()).collect();
```

## Concurrency with Threads

Rust's ownership system prevents data races at compile time:

```rust
use std::thread;

let mut handles = vec![];

for i in 0..10 {
    let handle = thread::spawn(move || {
        // `move` takes ownership of `i`
        println!("Thread {}: Hello!", i);
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}
```

Shared state with `Arc` (atomic reference counting):

```rust
use std::sync::Arc;
use std::thread;

let counter = Arc::new(std::sync::atomic::AtomicI64::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Counter: {}", counter.load(std::sync::atomic::Ordering::SeqCst));
```

## Error Handling Best Practices

1. **Use `?` for error propagation** — Don't unwrap in production code
2. **Create custom error types** — Use enums, not strings
3. **Implement `From` for automatic conversion** — Makes `?` work seamlessly
4. **Log errors at the handler level** — Don't duplicate logging
5. **Return user-friendly messages** — Don't expose internal details

```rust
// Bad: unwrap everywhere
let config = Config::from_env();  // Panics if env vars missing
let pool = db::init_pool(&url).await.unwrap();  // Panics on connection error

// Good: propagate errors
let config = Config::from_env();  // Panics are OK for required config
let pool = db::init_pool(&url).await?;  // Error propagated to caller
```

## Summary

This Rust tutorial covered the language features that FicHub uses most: variables, functions, structs, enums, Option, Result, traits, ownership, borrowing, iterators, and concurrency. Master these concepts and you'll be able to read and modify any part of the FicHub codebase.

---

# Extended Guide: The reqwest HTTP Client

## Building a Client

```rust
let client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")          // Identify ourselves
    .timeout(Duration::from_secs(30))         // Request timeout
    .connect_timeout(Duration::from_secs(10)) // Connection timeout
    .pool_max_idle_per_host(10)               // Connection pooling
    .build()?;
```

## Making Requests

```rust
// GET request
let response = client.get(url).send().await?;

// GET with query parameters
let response = client.get(url)
    .query(&[("key", "value"), ("page", "1")])
    .send().await?;

// POST with JSON body
let response = client.post(url)
    .json(&json!({"name": "FicHub", "version": 1}))
    .send().await?;

// POST with form data
let response = client.post(url)
    .form(&[("username", "user"), ("password", "pass")])
    .send().await?;

// Custom headers
let response = client.get(url)
    .header("User-Agent", "fichub.net/0.1.0")
    .header("Accept", "text/html")
    .send().await?;
```

## Handling Responses

```rust
// Check status
if response.status().is_success() {
    println!("Success!");
} else if response.status().is_client_error() {
    println!("Client error: {}", response.status());
} else if response.status().is_server_error() {
    println!("Server error: {}", response.status());
}

// Get response as text
let html = response.text().await?;

// Get response as JSON
let json: serde_json::Value = response.json().await?;

// Get response as bytes
let bytes = response.bytes().await?;

// Stream response (for large files)
let mut stream = response.bytes_stream();
while let Some(chunk) = stream.next().await {
    let chunk = chunk?;
    // Process chunk...
}
```

## Connection Pooling

`reqwest::Client` automatically manages a connection pool:

```rust
// Reuse the client for multiple requests
let client = reqwest::Client::new();

// These share the same connection pool
let r1 = client.get("https://example.com/page1").send().await?;
let r2 = client.get("https://example.com/page2").send().await?;
let r3 = client.get("https://example.com/page3").send().await?;
```

The pool reuses TCP connections, avoiding the overhead of creating new connections for each request.

## Error Handling

```rust
match client.get(url).send().await {
    Ok(response) => {
        // Handle successful response
    }
    Err(e) => {
        if e.is_timeout() {
            println!("Request timed out");
        } else if e.is_connect() {
            println!("Connection failed: {}", e);
        } else if e.is_redirect() {
            println!("Too many redirects");
        } else {
            println!("Other error: {}", e);
        }
    }
}
```

## Summary

`reqwest` provides a ergonomic API for HTTP requests with automatic connection pooling, timeout handling, and streaming support. FicHub uses it for all communication with fanfiction sites.

---

# Extended Guide: HTML Parsing with the Scraper Crate

## The Basics

The `scraper` crate parses HTML and lets you query it with CSS selectors:

```rust
use scraper::{Html, Selector};

let html = r#"
<html>
<body>
    <h1 class="title">Hello World</h1>
    <p id="intro">Welcome to FicHub</p>
    <div class="content">
        <p>First paragraph</p>
        <p>Second paragraph</p>
    </div>
</body>
</html>"#;

let document = Html::parse_document(html);
```

## CSS Selectors

```rust
// Element selector
let sel = Selector::parse("h1").unwrap();

// Class selector
let sel = Selector::parse("h1.title").unwrap();

// ID selector
let sel = Selector::parse("#intro").unwrap();

// Attribute selector
let sel = Selector::parse("a[href]").unwrap();
let sel = Selector::parse("a[href='https://example.com']").unwrap();
let sel = Selector::parse("a[rel='author']").unwrap();

// Descendant selector
let sel = Selector::parse("div p").unwrap();

// Child selector
let sel = Selector::parse("div > p").unwrap();

// Multiple selectors (comma-separated)
let sel = Selector::parse("h1, h2, h3").unwrap();
```

## Extracting Data

```rust
// Find first matching element
let title = document
    .select(&Selector::parse("h1.title").unwrap())
    .next()
    .map(|el| el.text().collect::<String>())
    .unwrap_or_default();

// Find all matching elements
let paragraphs: Vec<String> = document
    .select(&Selector::parse("p").unwrap())
    .map(|el| el.text().collect::<String>())
    .collect();

// Get attribute value
let href = document
    .select(&Selector::parse("a[href]").unwrap())
    .next()
    .and_then(|el| el.value().attr("href"));

// Get inner HTML (preserves tags)
let content = document
    .select(&Selector::parse("div.content").unwrap())
    .next()
    .map(|el| el.inner_html());
```

## Text vs Inner HTML

```rust
// .text() — Extracts only text content, strips HTML tags
let text: String = element.text().collect();
// Result: "Hello World" (no HTML tags)

// .inner_html() — Returns the raw HTML inside the element
let html = element.inner_html();
// Result: "<p>Hello</p> <p>World</p>" (preserves tags)
```

Use `.text()` for metadata (title, author, word count).
Use `.inner_html()` for content that needs HTML formatting (chapter text).

## Handling Malformed HTML

The `scraper` crate is tolerant of malformed HTML:

```rust
let html = r#"
<html>
<body>
    <h1>Unclosed tag
    <p>Paragraph without closing tag
    <div>Div with <b>bold <i>and italic</b></i></div>
</body>
</html>"#;

let document = Html::parse_document(html);
// Parser handles the errors gracefully
```

## Performance Tips

1. **Parse once, query many** — Parse the HTML once, then run multiple selectors
2. **Cache selectors** — If you use the same selector repeatedly, parse it once
3. **Use specific selectors** — ID selectors are faster than class selectors
4. **Limit results** — Use `.next()` instead of `.collect()` when you only need the first match

## Summary

The `scraper` crate provides a clean, Rust-idiomatic API for parsing HTML and extracting data with CSS selectors. It handles malformed HTML gracefully and is the foundation of FicHub's scraping system.


---

