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
