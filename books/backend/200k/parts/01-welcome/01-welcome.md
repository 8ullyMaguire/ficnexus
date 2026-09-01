# Part 1: Welcome to Rust Backend Development

---

# Chapter 1: What We're Building

## The World of Fanfiction

Have you ever watched a movie and thought, "I wish there were more stories about that side character"? Have you ever finished a book and wanted to spend just a little more time in that world? Have you ever read a TV show episode summary and thought, "What if the characters had met differently?" If any of that sounds familiar, you've already discovered the magic of fanfiction — and you're definitely not alone.

Fanfiction is one of the most vibrant creative communities on the internet. Millions of writers and readers come together on sites like Archive of Our Own (AO3), FanFiction.net, and countless XenForo forums to share stories inspired by the books, movies, and shows they love. We're talking about a community that produces more fiction annually than all published authors combined. It's enormous, it's passionate, and it's incredibly creative.

But here's the thing: reading fanfiction on these sites can be a pain. Ads cluttering the page. Clunky mobile interfaces that make scrolling a chore. Broken formatting when a story has special characters. Stories that get taken down without warning, vanishing from the internet forever. Wouldn't it be nice to download your favorite stories so you can read them offline, on your e-reader, or on a long flight without Wi-Fi? That's exactly what FicHub does.

The fanfiction community is also deeply creative in ways that go beyond writing. Readers curate reading lists, build tagging systems, create fan art, and develop tools to make the experience better. FicHub is part of that tradition — a tool built by fans, for fans, to make reading fanfiction more enjoyable. It's open source, meaning anyone can contribute, improve it, or run their own version on their own hardware.

What makes fanfiction particularly interesting from a technical standpoint is its diversity. There are stories on Archive of Our Own with over a million words. There are stories on FanFiction.net that have been around since 2001 — longer than some of you reading this book have been alive! There are niche communities on XenForo forums with their own custom themes and layouts. Each site has its own HTML structure, its own URL patterns, its own quirks. Building a tool that handles all of these is a genuine engineering challenge — and a rewarding one.

## What Is FicHub?

FicHub is a self-hosted fanfiction download server. Think of it as your personal librarian for the internet's creative writing. You give it a URL to a story on AO3, FanFiction.net, or any of the other supported sites, and it hands back a beautifully formatted EPUB file ready for your Kindle, Kobo, or any e-reader.

But FicHub isn't just a download button. It's a full-featured server with:

- **Scraping** — fetches story content from multiple fanfiction sites
- **Export** — generates EPUB, HTML, and other formats
- **Recommendations** — suggests similar stories based on community data
- **Search** — full-text search across your downloaded library
- **Tagging** — community-curated tags for better discovery
- **OPDS Catalog** — integrates with e-reader apps for seamless browsing

It's the kind of tool you run on your own hardware, share with friends, and never have to worry about some company shutting it down. Self-hosted means self-owned.

## The Big Picture: How FicHub Works

Let's zoom out and see how the whole system fits together. Here's the journey of a story through FicHub:

```
  Reader gives URL
       │
       ▼
  ┌─────────────┐     ┌──────────────┐     ┌───────────────┐
  │   Scraper   │────▶│    Parser    │────▶│   Exporter    │
  │  (fetches)  │     │  (extracts)  │     │  (generates)  │
  └─────────────┘     └──────────────┘     └───────────────┘
       │                    │                      │
       ▼                    ▼                      ▼
  ┌─────────────┐     ┌──────────────┐     ┌───────────────┐
  │  Database   │     │    Cache     │     │   Download    │
  │ (PostgreSQL)│     │   (Redis)    │     │  (EPUB/HTML)  │
  └─────────────┘     └──────────────┘     └───────────────┘
```

When someone pastes a URL into FicHub, the **scraper** (one of our site-specific scrapers) fetches the page. It figures out which site it's dealing with — AO3, FF.net, XenForo — and pulls the relevant HTML. Then the **parser** extracts the title, author, chapters, word count, tags, and all the story content. Finally, the **exporter** takes that structured data and generates a clean EPUB file with proper chapters, formatting, and metadata.

Along the way, we store the story in **PostgreSQL** so we don't have to scrape it again, and we use **Redis** for caching, rate limiting, and keeping things fast.

The database is the heart of FicHub. Every story we scrape gets stored with its full metadata — title, author, word count, tags, chapters, and the actual story content. This means the next time someone requests the same story, we don't need to hit AO3 again. We just pull it from our database and generate the EPUB instantly. The database also powers our search engine, our recommendation system, and our OPDS catalog.

Redis sits in front of the database for the hot path. When someone requests a story that's already been scraped, Redis serves the cached response in milliseconds instead of the database's hundreds of milliseconds. We also use Redis for rate limiting — making sure we don't hammer AO3 or FF.net with too many requests — and for background job queues that handle tasks like pre-generating EPUB files during off-peak hours.

The web server ties everything together using Axum, Rust's leading async web framework. Axum handles incoming HTTP requests, routes them to the right handler, and sends back responses. It's built on Tokio (the async runtime) and Tower (the middleware stack), giving us composable layers for things like CORS headers, request logging, and compression.

## Why Rust?

You might be wondering: why build this in Rust? Why not Python, Go, or even JavaScript?

Rust gives us three things we care about:

1. **Speed** — FicHub scrapes web pages, parses HTML, and generates EPUB files. Rust does all of this blazingly fast, even when handling thousands of requests. When you're fetching pages from AO3 and generating downloads, you want every millisecond to count.

2. **Safety** — Rust's compiler catches bugs before they ever reach production. No null pointer exceptions, no data races, no memory leaks. When you're running a server that handles user data and external network requests, this kind of safety isn't just nice — it's essential.

3. **Modern ecosystem** — Axum (our web framework) is built on Tokio (our async runtime) and Tower (our middleware stack). These libraries are production-proven and designed for exactly the kind of thing we're building. PostgreSQL integration with SQLx gives us compile-time checked queries. Redis support is first-class.

Plus, Rust's `async`/`await` syntax makes handling concurrent requests feel natural. When someone is downloading a 50-chapter story from AO3, we don't want that blocking other users from making requests. Rust's async model handles this beautifully.

Here's a concrete example of why this matters: imagine 50 users simultaneously request stories. With a traditional synchronous server, each request would block the thread while waiting for the network — the 50th user would have to wait for all 49 before them to finish. With Rust's async model, the server can start all 50 requests concurrently and process each response as it arrives. The difference isn't just theoretical — it's the difference between a server that can handle 100 requests per second and one that can handle 10,000.

Rust's approach to error handling is also worth mentioning. Instead of exceptions that can crash your program at runtime, Rust uses the `Result` and `Option` types to make errors explicit in the type system. If a function can fail, its return type says so. The compiler forces you to handle every possible failure before your code compiles. This is annoying at first, but it means FicHub rarely crashes in production. The bugs are caught before they ever reach your users.

💡 **Key Concept: Why "Self-Hosted" Matters**

Self-hosted means you run the software on your own computer or server, not on someone else's cloud service. You own the data, you control the uptime, and nobody can shut you down. For fanfiction — a community that has lost beloved sites to domain changes, funding problems, and policy shifts — self-hosting is a statement: *this tool belongs to us*.

## What You'll Learn

By the end of this book, you'll have built a complete fanfiction download server from scratch. Here's what we'll cover:

**The Rust Foundations** (Parts 1–2): We'll start with Rust basics — variables, functions, ownership, error handling — and then build our first web server with Axum. You'll understand how Rust's type system helps us write reliable code.

**The Database Layer** (Part 3): PostgreSQL is where we store everything. You'll learn SQL, database design, migrations, and how to use SQLx to talk to PostgreSQL from Rust with compile-time safety.

**Web Scraping** (Parts 4–5): This is where FicHub gets interesting. You'll build scrapers for AO3, FF.net, XenForo, and more. You'll learn HTML parsing, CSS selectors, and the strategy pattern for handling different site formats.

**The Export Engine** (Part 6): You'll generate EPUB files, HTML bundles, and other formats. You'll learn about file caching, semaphores to prevent duplicate work, and version hashing for cache invalidation.

**Redis and Rate Limiting** (Part 7): Redis powers caching and rate limiting. You'll implement a token bucket rate limiter and learn why protecting upstream sites matters.

**Search, Tags, and Recommendations** (Parts 8–10): These are the features that make FicHub more than just a download tool. Full-text search, community tagging, and a collaborative filtering recommendation engine.

**OPDS and Frontend** (Parts 11–12): OPDS lets e-reader apps browse your FicHub library directly. We'll also serve a SvelteKit frontend.

**Testing and Deployment** (Parts 13–15): We'll cover unit tests, integration tests, Docker, nginx, and monitoring. You'll learn to ship production-quality code.

Throughout the book, every concept connects back to FicHub. You won't learn Rust in the abstract — you'll learn it by building something real, something useful, something you can actually run and share with friends.

⚠️ **Watch Out: This Book Assumes Some Programming Experience**

We'll explain Rust concepts from scratch, but we won't explain what a variable, function, or HTTP request is. If you've never programmed before, consider learning Python or JavaScript first, then come back to Rust. If you've written code in any language, you'll be fine.

## The Tech Stack

Let's meet the tools we'll be using:

| Tool | What It Does | Why We Use It |
|------|-------------|---------------|
| **Rust** | Programming language | Fast, safe, modern |
| **Axum** | Web framework | Ergonomic, async, composable |
| **Tokio** | Async runtime | Powers Axum's async I/O |
| **PostgreSQL** | Database | Reliable, feature-rich, fast |
| **SQLx** | Database driver | Compile-time checked queries |
| **Redis** | Cache & queues | Fast in-memory data store |
| **reqwest** | HTTP client | For scraping fanfiction sites |
| **scraper** | HTML parser | CSS selector-based parsing |
| **epub-builder** | EPUB generator | Creates e-reader files |
| **tracing** | Logging | Structured, filterable logs |
| **Docker** | Containers | Consistent deployment |
| **nginx** | Reverse proxy | TLS, static files, load balancing |

Don't worry if you don't know what all of these do yet. We'll explain each one as we use it. By the end of the book, you'll understand why every tool on this list earned its spot.

A few quick notes on the choices:

- **Axum vs Actix**: Both are excellent Rust web frameworks. Axum is newer and built by the Tokio team, with a more composable design using Tower middleware. Actix is slightly faster in benchmarks, but Axum's ergonomics make it the better choice for most projects.

- **PostgreSQL vs SQLite vs MySQL**: PostgreSQL gives us full-text search (tsvector), JSONB columns, and mature extension support. SQLite would work for a single-user setup, but FicHub is designed for sharing with friends.

- **Redis vs Memcached**: Redis supports more data structures (lists, sets, sorted sets) and has pub/sub capabilities. We use sorted sets for rate limiting and strings for caching.

- **reqwest vs ureq**: Both are HTTP clients. reqwest is async (works with Tokio), while ureq is synchronous. Since we're building an async web server, reqwest is the natural choice.

These aren't arbitrary decisions — each tool was chosen because it solves a specific problem in the FicHub architecture.

## A Note for Experienced Developers

If you've built web servers before — in Python, Go, Node.js, or anything else — you'll notice that Rust does things differently. Where Python uses dynamic typing, Rust uses static types. Where Go has goroutines, Rust has async/await with the Tokio runtime. Where Node.js uses callbacks and promises, Rust uses `Future` traits that the compiler can optimize.

These differences aren't just syntactic sugar. Rust's approach to concurrency doesn't just give you async functions — it gives you **fearless concurrency**. The same ownership rules that prevent memory bugs also prevent data races at compile time. Two threads can't accidentally modify the same data because the compiler won't allow two mutable references to exist simultaneously.

If you're coming from a garbage-collected language, expect an adjustment period. Rust requires you to think about memory in ways you haven't had to before. But the payoff is real: programs that are as fast as C, as safe as Haskell, and as ergonomic as Python (once you learn the patterns). FicHub's scraper handles thousands of concurrent requests with sub-millisecond overhead — not because someone optimized it to death, but because Rust's default performance is that good.

## What's Ahead

In the next chapter, we'll set up your development environment — installing Rust, PostgreSQL, Redis, and everything else you need. Then we'll write our first Rust program and start exploring the language's unique features.

Ready? Let's build something amazing.

---

# Chapter 2: Setting Up Your Workshop

## Why Setup Matters

Every good builder starts with a clean workshop. Before we write a single line of Rust, we need to install the tools we'll be using. This chapter walks you through installing everything step by step. Don't skip this — a working setup means you can follow along with every code example in the book.

We'll install five things:

1. **Rust** — the programming language and its package manager, Cargo
2. **PostgreSQL** — the database where FicHub stores stories
3. **Redis** — the in-memory data store for caching and rate limiting
4. **Node.js** — needed for the SvelteKit frontend (we'll build the backend first, but it's good to have ready)
5. **A code editor** — we recommend VS Code with the rust-analyzer extension

Let's get started.

## Installing Rust with rustup

Rust has an official installer called `rustup`. It handles installing Rust, updating it, and managing different versions. It works on Linux, macOS, and Windows.

### On Linux (and macOS)

Open a terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This downloads the installer and runs it. When it asks you a question, just press `1` and Enter to proceed with the default installation.

After it finishes, restart your terminal (or run `source $HOME/.cargo/env`) and verify:

```bash
rustc --version
cargo --version
```

You should see something like:

```
rustc 1.82.0 (f6e511eec 2024-10-15)
cargo 1.82.0 (8f401398f 2024-09-30)
```

The exact version numbers don't matter — what matters is that they print something instead of "command not found."

### On Windows

Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs). You'll need the Visual Studio C++ Build Tools — the installer will guide you through this.

### What Did We Just Install?

Let's take a moment to understand what's in the toolbox:

- **`rustc`** — the Rust compiler. It takes your `.rs` files and turns them into executable programs.
- **`cargo`** — the package manager and build system. You'll use this for almost everything. `cargo build`, `cargo run`, `cargo test`, `cargo add` — Cargo is your best friend.
- **`rustup`** — the tool that installed everything. Use `rustup update` to get the latest Rust version.
- **`rust-analyzer`** — the language server that gives you autocompletion and error checking in your editor.

### Updating Rust

Rust releases a new stable version every six weeks. To stay current, run:

```bash
rustup update
```

This updates the compiler, Cargo, and all installed components. FicHub uses the 2024 edition of Rust, which is the latest. The edition system lets Rust evolve without breaking existing code — your old programs keep working even after you update.

You can check which editions and toolchains you have installed:

```bash
rustup show
```

This lists your default toolchain, host platform, and installed components. If you see `stable-x86_64-unknown-linux-gnu` as your default, you're good to go.

### Installing Useful Tools

While you're at it, install two tools that make Rust development much better:

```bash
rustup component add clippy
```

`clippy` is Rust's linter. It catches common mistakes and suggests more idiomatic ways to write your code. Think of it as a helpful senior developer looking over your shoulder:

```bash
cargo clippy
# Tells you about code that could be improved
```

The other tool is `rustfmt`, which formats your code consistently:

```bash
cargo fmt
# Reformats all your code to match the standard style
```

Set up your editor to run both of these automatically on save. If you're using VS Code, install the "rust-analyzer" extension — it gives you autocompletion, inline error messages, and code suggestions as you type. The experience is similar to what you get with TypeScript in VS Code, but even more powerful because Rust's type system is so rich.

💡 **Key Concept: Cargo Is Everything**

In the JavaScript world, you have npm, yarn, and package.json. In Python, you have pip and requirements.txt. In Rust, you have Cargo and Cargo.toml. Cargo handles building, testing, running, managing dependencies, and more. You'll spend most of your Rust life typing `cargo` commands.

## Installing PostgreSQL

PostgreSQL is the world's most advanced open-source relational database. It's what FicHub uses to store stories, tags, recommendations, and everything else.

### On Ubuntu/Debian

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
```

### On Arch Linux / Manjaro

```bash
sudo pacman -S postgresql
```

After installing, start PostgreSQL and create a database:

```bash
# On Ubuntu/Debian
sudo systemctl start postgresql
sudo systemctl enable postgresql

# On Arch Linux
sudo postgresql-setup --initdb
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

Now create a database for FicHub:

```bash
# Switch to the postgres user
sudo -u postgres psql

# Create a database and user
CREATE USER fichub WITH PASSWORD 'fichub';
CREATE DATABASE fichub OWNER fichub;
\q
```

Verify it works:

```bash
psql -U fichub -d fichub -c "SELECT 1;"
```

If you see `1`, PostgreSQL is ready.

⚠️ **Watch Out: Password Authentication**

On some systems, PostgreSQL uses "peer" authentication, which means you can only connect as the same user who's running the command. If you get an authentication error, you may need to edit `pg_hba.conf` (usually in `/etc/postgresql/*/main/` or `/var/lib/postgres/data/`) to allow password authentication:

```
# Before (peer authentication):
local   all   all   peer

# After (password authentication):
local   all   all   md5
```

Then restart PostgreSQL.

## Installing Redis

Redis is a fast, in-memory data store. FicHub uses it for caching (storing recently computed results so we don't have to recompute them) and rate limiting (making sure we don't send too many requests to fanfiction sites).

### On Ubuntu/Debian

```bash
sudo apt install redis-server
sudo systemctl start redis-server
sudo systemctl enable redis-server
```

### On Arch Linux / Manjaro

```bash
sudo pacman -S redis
sudo systemctl start redis
sudo systemctl enable redis
```

Verify Redis is running:

```bash
redis-cli ping
```

You should see `PONG`. That's Redis saying "I'm here and ready!"

You can also test setting and getting a value:

```bash
redis-cli SET test_key "hello"
redis-cli GET test_key
# Output: "hello"
redis-cli DEL test_key
```

If all three commands work, Redis is fully operational. Redis is an in-memory data store, meaning it keeps all its data in RAM. This makes it incredibly fast — reads and writes typically complete in under a millisecond. The tradeoff is that data doesn't persist across restarts by default. For FicHub, this is fine: cached data can be re-generated, and rate limiting counters reset periodically anyway.

Don't worry about the various Redis configuration options right now — the defaults work fine for development.

## Installing Node.js

Node.js is needed for building the SvelteKit frontend. We won't use it in the backend chapters, but it's good to have installed now so you're ready when we get to the frontend.

The easiest way to install Node.js is with `nvm` (Node Version Manager):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
```

Restart your terminal, then install Node.js:

```bash
nvm install 20
nvm use 20
node --version   # Should show v20.x.x
npm --version    # Should show 10.x.x
```

Alternatively, on Arch Linux:

```bash
sudo pacman -S nodejs npm
```

## Creating a New Rust Project

Now that we have our tools installed, let's create a Rust project. We'll start with a simple "hello world" and build from there.

Open a terminal and navigate to where you want your project to live:

```bash
mkdir -p ~/code/rust
cd ~/code/rust
```

Create a new Rust project:

```bash
cargo new fichub-book
cd fichub-book
```

This creates a new directory called `fichub-book` with a basic project structure:

```
fichub-book/
├── Cargo.toml      ← project configuration and dependencies
└── src/
    └── main.rs     ← your code goes here
```

Let's look at these files.

### Cargo.toml: Your Project's Recipe Book

`Cargo.toml` is where you tell Cargo about your project. Open it in your editor:

```toml
[package]
name = "fichub-book"
version = "0.1.0"
edition = "2021"

[dependencies]
```

Here's what each line means:

- **`[package]`** — this section describes your project
- **`name`** — the name of your project (used when publishing to crates.io)
- **`version`** — semantic versioning: major.minor.patch
- **`edition`** — which Rust edition to use (2021 is current; 2024 is available too)
- **`[dependencies]`** — libraries your project needs (currently empty)

As we build FicHub, this file will grow to include dozens of dependencies. Each one adds functionality we need — web serving, database access, HTML parsing, EPUB generation, and more.

### src/main.rs: Where the Magic Happens

`src/main.rs` is the entry point of your program. When you run your project, Rust compiles `main.rs` and executes it.

Let's look at what Cargo generated:

```rust
fn main() {
    println!("Hello, world!");
}
```

That's it. One function, one line of code. Let's run it.

## Running Your First Build

In your terminal (inside the `fichub-book` directory), run:

```bash
cargo run
```

You'll see output like this:

```
   Compiling fichub-book v0.1.0 (/home/user/code/rust/fichub-book)
    Finished dev [unoptimized + debuginfo] target(s) in 0.52s
     Running `target/debug/fichub-book`
Hello, world!
```

That's your first Rust program running! Let's break down what happened:

1. **`Compiling`** — Cargo called `rustc` to compile your code into machine code
2. **`Finished`** — Compilation succeeded (no errors!)
3. **`Running`** — Cargo ran the compiled program
4. **`Hello, world!`** — Your program's output

### What Happened Under the Hood

Let's slow down and understand the chain of events:

1. Cargo read your `Cargo.toml` to understand the project configuration
2. Cargo called `rustc` (the Rust compiler) with the appropriate flags
3. `rustc` compiled `src/main.rs` into machine code
4. The machine code was linked into a binary executable
5. The executable was placed at `target/debug/fichub-book`
6. Cargo ran the executable

The `target/` directory is where Cargo puts all its build artifacts. The `debug/` subdirectory contains debug builds. You can ignore this directory — it's like `.next/` in Next.js or `node_modules/` in Node.js. Cargo manages it for you.

Try running `cargo build` without `run` — it compiles but doesn't execute. Then run the binary manually. This two-step approach is useful when you want to compile once and run multiple times, or when you want to pass special command-line arguments.

The compiled binary lives at `target/debug/fichub-book`. You can run it directly:

```bash
./target/debug/fichub-book
# Output: Hello, world!
```

🧪 **Try It Yourself: Make It Personal**

Change `main.rs` to say hello to yourself:

```rust
fn main() {
    println!("Hello, Future Rust Developer!");
    println!("I'm going to build a fanfiction server today.");
}
```

Run `cargo run` again and see the new output. Try adding more `println!` lines. Can you make it print a little box?

```rust
fn main() {
    println!("┌─────────────────────┐");
    println!("│  Welcome to FicHub! │");
    println!("│  A Rust adventure.  │");
    println!("└─────────────────────┘");
}
```

## Understanding Cargo Commands

You'll use Cargo constantly while building FicHub. Here are the essential commands:

| Command | What It Does |
|---------|-------------|
| `cargo run` | Build and run your program |
| `cargo build` | Build without running |
| `cargo build --release` | Optimized build (faster, slower to compile) |
| `cargo check` | Check for errors without producing a binary |
| `cargo test` | Run all tests |
| `cargo add <crate>` | Add a dependency |
| `cargo fmt` | Format your code |
| `cargo clippy` | Lint your code (find common mistakes) |

### Adding Dependencies

When your project needs external libraries (called "crates" in Rust), you add them with `cargo add`:

```bash
cargo add serde              # Add the serde serialization library
cargo add serde_json         # Add JSON support
cargo add tokio -F full      # Add tokio with all features enabled
```

This automatically edits your `Cargo.toml` and downloads the crate. You can also add dependencies by editing `Cargo.toml` directly, which is what we'll do for most of this book.

When you first build after adding a new dependency, Cargo downloads it from crates.io (Rust's package registry) and compiles it. This can take a while for the first build. Subsequent builds are fast because Cargo caches compiled dependencies.

💡 **Key Concept: Debug vs Release Builds**

When you run `cargo build`, it creates a debug build. These compile fast but run slow — perfect during development. When you're ready for production, use `cargo build --release`. Release builds take longer to compile but run much faster (often 10x faster!). The binary goes to `target/release/` instead of `target/debug/`.

## A Quick Tour of Dependencies

Before we close this chapter, let's peek at what FicHub's `Cargo.toml` looks like. This is the real `Cargo.toml` from the FicHub project:

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
description = "Self-hosted fanfiction download server (fichub.net replacement)"

[dependencies]
# Web framework & runtime
axum = "0.8"
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.7", features = ["cors", "compression-gzip", "trace", "fs"] }

# Database
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# HTTP client
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

# HTML parsing
scraper = "0.27"

# EPUB generation
epub-builder = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

Don't try to understand all of this now — we'll explain every dependency as we use it. The point is to show you that a real project's `Cargo.toml` is a清单 of all the external libraries (called "crates") the project depends on. Each one brings specific functionality: `axum` gives us a web framework, `sqlx` lets us talk to PostgreSQL, `scraper` parses HTML, and so on.

The `{ version = "0.7", features = [...] }` syntax means "I want version 0.7 of this crate, and I need these specific features enabled." Crates often have optional features to keep download sizes small — you only pull in what you need.

## Your Development Environment

Let's make sure you have everything ready. Run these checks:

```bash
# Rust and Cargo
rustc --version     # Should show 1.70+ 
cargo --version     # Should show matching version

# PostgreSQL
psql --version      # Should show 14+
redis-cli --version # Should show 7+

# Node.js
node --version      # Should show 18+ or 20+
```

If all of these print version numbers, you're ready to start writing Rust code. If something's missing, go back to the relevant section and install it.

⚠️ **Watch Out: Don't Install Everything at Once**

If you're feeling overwhelmed by all the installation steps, take a break. You can always come back and install things later. The important thing is that you have Rust and Cargo working. PostgreSQL and Redis can wait until we actually need them (Part 3 for the database, Part 7 for Redis).

## What's Ahead

In the next chapter, we'll write our first real Rust program. We'll explore variables, functions, types, and the `println!` macro. By the end, you'll understand enough Rust to follow along with the more advanced chapters.

Let's keep building!

---

# Chapter 3: Your First Rust Program

## Hello, Rust!

In the last chapter, we created a Rust project and ran "Hello, world!" Now it's time to actually learn some Rust. This chapter covers the fundamentals: variables, types, functions, and how to print things. Think of this as learning the alphabet before writing sentences.

We'll build a tiny program that calculates word counts for fanfiction chapters. It's simple enough to follow, but real enough to be useful. Along the way, you'll learn the building blocks that everything else in this book is built on.

## Hello World, Again

Let's start with the classic example and break it down:

```rust
fn main() {
    println!("Hello, world!");
}
```

Every Rust program starts with a `main` function. This is the entry point — the first function that runs when your program starts. The `fn` keyword declares a function, `main` is its name, and `()` means it takes no parameters.

Inside `main`, we have one line: `println!("Hello, world!");`. Let's dissect this:

- `println!` — this is a **macro**, not a function. The `!` at the end tells you it's a macro. Macros in Rust look like function calls but have a `!` after their name. They do special things that regular functions can't do — like accepting a variable number of arguments.
- `("Hello, world!")` — the text we want to print, wrapped in parentheses and quotes.
- `;` — the semicolon ends the statement. In Rust, almost every statement ends with a semicolon. Forgetting it is a common beginner mistake.

### Why Macros?

You might wonder why `println!` is a macro instead of a regular function. The answer is that macros can accept a flexible number of arguments with different types. A regular function like `print(a: String, b: String)` always needs exactly two strings. But `println!` can take zero arguments (`println!()`), one argument (`println!("hi")`), or many arguments with different types (`println!("{} is {} years old", name, age)`). Macros give Rust this flexibility while still being checked at compile time.

Another common macro you'll see is `vec!`:

```rust
let chapters = vec![1, 2, 3, 4, 5];
```

This creates a `Vec<i32>` (a growable list) from a list of values. Without the macro, you'd need several lines of code to allocate the vector and push each value. The `vec!` macro handles the allocation and initialization for you.

Don't worry too much about how macros work internally. Just remember: if you see a `!` after a name, it's a macro. Macros are expanded (replaced with actual code) by the compiler before your program runs. The Rust reference manual has details on macro_rules! if you're curious, but for this book, you just need to know how to use them.

## Variables and Mutability

In Rust, you declare variables with `let`:

```rust
fn main() {
    let name = "Harry";
    let age = 17;
    let word_count = 45_000;

    println!("{} is {} years old and wrote {} words", name, age, word_count);
}
```

Running this gives:

```
Harry is 17 years old and wrote 45000 words
```

Notice a few things:

- **Inference** — we didn't say `let name: &str = "Harry";`. Rust figured out the type from the value. This is called **type inference**, and it means you don't have to write type annotations everywhere.
- **Underscores in numbers** — `45_000` is the same as `45000`. The underscores are just visual separators, like commas in `45,000`. They make large numbers easier to read.
- **String formatting** — `{}` is a placeholder. `println!` replaces it with the value of the variable. This is called **format string interpolation**.

### Immutability by Default

Here's something important: variables in Rust are **immutable** by default. That means once you assign a value, you can't change it:

```rust
fn main() {
    let word_count = 45_000;
    word_count = 50_000;  // ERROR! Can't reassign an immutable variable
}
```

This will produce a compiler error:

```
error[E0384]: cannot assign twice to immutable variable `word_count`
```

Rust is telling you: "Hey, you said this variable is immutable, and now you're trying to change it. That's not allowed."

If you need to change a variable, use `mut` (short for mutable):

```rust
fn main() {
    let mut word_count = 45_000;
    println!("Current word count: {}", word_count);

    word_count = 50_000;
    println!("Updated word count: {}", word_count);
}
```

This works and prints:

```
Current word count: 45000
Updated word count: 50000
```

💡 **Key Concept: Immutability Is the Default**

Rust defaults to immutability for a good reason: it makes code easier to reason about. When you see `let x = 5;`, you know `x` is always 5. No surprises. You only opt into mutability when you actually need it, which is less often than you think. Most variables don't need to change.

🧪 **Try It Yourself: Chapter Counter**

Create a program that counts chapters in a fanfiction story:

```rust
fn main() {
    let mut chapter = 1;
    let total_chapters = 25;

    println!("Reading chapter {} of {}", chapter, total_chapters);

    chapter = chapter + 1;
    println!("Now on chapter {} of {}", chapter, total_chapters);

    chapter = chapter + 1;
    println!("Now on chapter {} of {}", chapter, total_chapters);
}
```

What happens if you try to use `chapter` without `mut`? Try it and read the error message. Rust's error messages are famously helpful — they tell you exactly what's wrong and often suggest a fix.

## Basic Types

Rust is a **statically typed** language, meaning every variable has a type that's known at compile time. Let's look at the basic types:

### Integers

Rust has several integer types, depending on how many bytes you need:

```rust
fn main() {
    let a: i32 = 42;         // 32-bit signed integer (default)
    let b: i64 = 1_000_000;  // 64-bit signed integer
    let c: u32 = 42;         // 32-bit unsigned (no negatives)
    let d: i8 = -128;        // 8-bit signed (-128 to 127)
    let e: usize = 100;      // unsigned, size of a pointer (for array indices)

    println!("a={}, b={}, c={}, d={}, e={}", a, b, c, d, e);
}
```

In practice, you almost never need to specify the type — Rust defaults to `i32` for integer literals. Use `i64` or `u64` when you need bigger numbers (like word counts for long stories), and `usize` when you're indexing into arrays or vectors.

### Floating-Point Numbers

```rust
fn main() {
    let rating: f64 = 4.8;       // 64-bit float (default)
    let precise: f32 = 3.14;     // 32-bit float

    println!("Story rating: {}, Pi: {}", rating, precise);
}
```

Use `f64` for most floating-point math. It's more precise than `f32` and modern CPUs handle both at the same speed.

### Booleans

```rust
fn main() {
    let is_complete: bool = true;
    let has_warnings: bool = false;

    println!("Complete: {}, Has warnings: {}", is_complete, has_warnings);
}
```

Booleans are either `true` or `false`. They're used in conditions (if/else), comparisons, and as flags.

### Characters

```rust
fn main() {
    let first_letter: char = 'H';
    let emoji: char = '📚';
    let chinese: char = '书';

    println!("{}{}{}", first_letter, emoji, chinese);
}
```

Rust `char`s are 4 bytes and can represent any Unicode character — not just ASCII. This is important for fanfiction, which often contains characters from many languages and special Unicode symbols.

### Strings

Rust has two main string types, and the difference trips up every beginner:

```rust
fn main() {
    // String slice (&str) — a reference to string data
    let title: &str = "Harry Potter and the Methods of Rationality";

    // Owned String — owns its data, can be modified
    let mut author: String = String::from("Less Wrong");
    author.push_str(" (originally)");

    println!("{} by {}", title, author);
}
```

- **`&str`** — a string slice. It's a reference to string data stored somewhere else. You use it for text you don't need to change. Most function parameters should take `&str`.
- **`String`** — an owned, growable string. You use it when you need to modify the text, build it up, or store it somewhere that outlives the current function.

Don't worry if this distinction feels confusing now — we'll come back to it in Chapter 4 when we talk about ownership. For now, just know: `&str` for reading, `String` for building.

Here's a practical example that shows why both types exist:

```rust
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn main() {
    let title = String::from("The Lord of the Fanfics");

    // Pass a String to a &str parameter — Rust auto-converts
    println!("Words in title: {}", word_count(&title));

    // Pass a string literal directly — it's already a &str
    println!("Words in chapter: {}", word_count("Once upon a time..."));

    // Build a new String by modifying the original
    let mut new_title = title.clone();
    new_title.push_str(": The Fellowship of the Fic");
    println!("Extended title: {}", new_title);
}
```

Functions that only need to read text should take `&str` (not `&String`). This makes them more flexible — they can accept both `String` references and string literals without the caller needing to do anything special.

### Tuples and Arrays

```rust
fn main() {
    // Tuple — fixed-size collection of different types
    let chapter_info: (&str, i32, f64) = ("Chapter 1", 5000, 4.5);
    println!("{}: {} words, {} stars", chapter_info.0, chapter_info.1, chapter_info.2);

    // Array — fixed-size collection of the same type
    let ratings = [5, 4, 5, 4, 3];
    println!("Ratings: {:?}", ratings);
    println!("First rating: {}", ratings[0]);
}
```

Tuples hold a fixed number of values that can be different types. Arrays hold a fixed number of values that must be the same type. Both have their uses, but you'll see arrays more often (especially as `Vec<T>`, the growable version, which we'll learn soon).

## Functions

Functions in Rust are declared with `fn`. Let's write some:

```rust
fn calculate_words(chapter_count: i32, avg_words_per_chapter: i32) -> i32 {
    chapter_count * avg_words_per_chapter
}

fn format_word_count(count: i32) -> String {
    if count >= 1_000_000 {
        format!("{}M words", count / 1_000_000)
    } else if count >= 1_000 {
        format!("{}K words", count / 1_000)
    } else {
        format!("{} words", count)
    }
}

fn main() {
    let total_words = calculate_words(25, 3500);
    println!("Total: {}", total_words);         // 87500
    println!("Formatted: {}", format_word_count(total_words));  // 87K words
}
```

Let's break this down:

- **`fn calculate_words(...)`** — declares a function named `calculate_words`
- **`(chapter_count: i32, avg_words_per_chapter: i32)`** — two parameters, both `i32`. In Rust, you **must** specify the type of every parameter.
- **`-> i32`** — the return type. This function returns an `i32`.
- **`chapter_count * avg_words_per_chapter`** — the last expression in a function is the return value. Note: no `return` keyword, no semicolon. This is an **expression**, not a statement.

💡 **Key Concept: Statements vs Expressions**

This is a crucial Rust distinction:

- A **statement** does something and has a semicolon: `let x = 5;`
- An **expression** produces a value and has no semicolon: `5 + 3`

The last expression in a function body is the return value. If you add a semicolon, it becomes a statement and the function returns `()` (unit type, like "nothing"). Forgetting the semicolon on the return line is a common mistake — and Rust will tell you exactly what happened.

```rust
// This works:
fn add(a: i32, b: i32) -> i32 {
    a + b   // expression — no semicolon, returns the value
}

// This does NOT work as expected:
fn add_wrong(a: i32, b: i32) -> i32 {
    a + b;  // statement! Returns () which is not i32
}
```

### The println! Macro in Detail

We've been using `println!` a lot. Let's understand it better:

```rust
fn main() {
    let title = "My Fanfiction";
    let chapters = 10;
    let rating = 4.7;

    // Basic formatting with {}
    println!("Title: {}", title);
    println!("Chapters: {}, Rating: {}", chapters, rating);

    // Named placeholders
    println!("{title} has {chapters} chapters");

    // Debug formatting with {:?}
    println!("Debug: {:?}", (title, chapters));

    // Padding and alignment
    println!("{:<20} {:>5}", "Title", "Words");  // left-aligned, right-aligned
    println!("{:<20} {:>5}", "Chapter 1", "3500");
    println!("{:<20} {:>5}", "Chapter 2", "4200");

    // Number formatting
    println!("Total: {:.1} stars", rating);  // one decimal place
}
```

The `{}` format specifier is surprisingly powerful. You can:
- Use `{name}` to refer to a variable by name
- Use `{:?}` for debug formatting (works for any type)
- Use `{:<20}` for left-alignment within 20 characters
- Use `{:.1}` to format numbers with specific decimal places

## Comments and Documentation

Rust has several ways to leave notes in your code:

```rust
// This is a regular comment — it's ignored by the compiler

/// This is a documentation comment.
/// It appears in generated documentation (cargo doc).
/// You can use **markdown** in it!
fn documented_function() {
    // Comments can appear anywhere
    let x = 5; // even at the end of a line
}

/* This is a block comment.
   It can span multiple lines. */
```

Documentation comments (starting with `///`) are special. You can generate HTML documentation from them:

```rust
/// Calculates the total word count for a fanfiction story.
///
/// # Arguments
///
/// * `chapter_words` - A slice of word counts, one per chapter
///
/// # Returns
///
/// The total word count across all chapters.
///
/// # Examples
///
/// ```
/// let counts = vec![3000, 4500, 2800];
/// let total = word_count(&counts);
/// assert_eq!(total, 10300);
/// ```
fn word_count(chapter_words: &[i32]) -> i32 {
    chapter_words.iter().sum()
}
```

Try running `cargo doc --open` in your project. It generates beautiful HTML documentation from your `///` comments, complete with examples that get tested when you run `cargo test`.

⚠️ **Watch Out: Don't Forget the Semicolon**

The most common beginner mistake in Rust is putting a semicolon where it doesn't belong. Here's a quiz:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // ✅ Correct: expression, returns value
}

fn add(a: i32, b: i32) -> i32 {
    a + b;  // ❌ Wrong: statement, returns ()
}
```

The second version will give you an error like:

```
error: mismatched types
  expected `i32`, found `()`
```

If you see this error, you probably have a stray semicolon on your return expression.

## Putting It All Together

Let's write a small program that uses everything we've learned. We'll calculate statistics for a fanfiction story:

```rust
/// Calculate and display statistics for a fanfiction story.
fn main() {
    // Story metadata
    let title = "The Rationalist's Guide to Magic";
    let author = "LessWrong_Fan";
    let mut chapter_count = 0;

    // Chapter word counts (simulating data we'd get from a scraper)
    let chapter_words = [3_200, 4_500, 2_800, 5_100, 3_900, 4_200, 3_700];

    // Count chapters
    chapter_count = chapter_words.len() as i32;

    // Calculate statistics
    let total_words: i32 = chapter_words.iter().sum();
    let avg_words = total_words as f64 / chapter_count as f64;
    let min_words = chapter_words.iter().min().unwrap();
    let max_words = chapter_words.iter().max().unwrap();
    let is_long = total_words > 50_000;

    // Display results
    println!("╔══════════════════════════════════════╗");
    println!("║  Fanfiction Statistics Report         ║");
    println!("╠══════════════════════════════════════╣");
    println!("║  Title:    {:<26} ║", title);
    println!("║  Author:   {:<26} ║", author);
    println!("║  Chapters: {:<26} ║", chapter_count);
    println!("║  Total:    {:<26} ║", format_word_count(total_words));
    println!("║  Average:  {:<26} ║", format!("{:.0} words/chapter", avg_words));
    println!("║  Shortest: {:<26} ║", format_word_count(min_words));
    println!("║  Longest:  {:<26} ║", format_word_count(max_words));
    println!("║  Long?     {:<26} ║", is_long);
    println!("╚══════════════════════════════════════╝");
}

/// Format a word count into a human-readable string.
fn format_word_count(count: i32) -> String {
    if count >= 1_000_000 {
        format!("{}M words", count / 1_000_000)
    } else if count >= 1_000 {
        format!("{}K words", count / 1_000)
    } else {
        format!("{} words", count)
    }
}
```

Save this to `src/main.rs` and run `cargo run`. You should see a nicely formatted statistics report. This is a small example, but it uses real Rust features: variables, mutability, arrays, functions, formatting, and the `iter()` methods that are so powerful in Rust.

🧪 **Try It Yourself: Extend the Stats**

Add a function that finds the "best" chapter (the longest one) and prints its position:

```rust
fn longest_chapter(chapter_words: &[i32]) -> (usize, i32) {
    let mut max_idx = 0;
    let mut max_val = 0;
    for (i, &words) in chapter_words.iter().enumerate() {
        if words > max_val {
            max_val = words;
            max_idx = i;
        }
    }
    (max_idx, max_val)
}
```

Call it from `main` and display which chapter was longest. The `enumerate()` method gives you both the index and the value — a common Rust pattern.

## What's Ahead

You now know enough Rust to follow along with the rest of this book. Variables, functions, types, formatting, and comments — these are the building blocks. In the next chapter, we'll tackle Rust's most unique and powerful features: ownership, borrowing, lifetimes, and the `Option` and `Result` types. These are the concepts that make Rust different from every other language, and they're the reason Rust can guarantee memory safety without a garbage collector.

Don't worry if ownership feels strange at first. It felt strange for everyone. By the end of the next chapter, you'll understand why Rust works this way and how to think in Rust.

---

# Chapter 4: Thinking in Rust

## The Rust Mindset

Every programming language has its own way of thinking. In Python, you think in objects and dictionaries. In JavaScript, you think in callbacks and promises. In Rust, you think in **ownership**.

Ownership is Rust's most distinctive feature. It's the reason Rust can be both fast (no garbage collector slowing things down) and safe (no memory bugs, no data races). It's also the feature that trips up every beginner. But once it clicks, everything else in Rust makes sense.

In this chapter, we'll learn:

1. **Ownership** — who owns data and when it gets cleaned up
2. **Borrowing** — sharing data without transferring ownership
3. **Lifetimes** — how the compiler tracks how long references live
4. **Option and Result** — Rust's way of handling "maybe" and "error"
5. **Pattern matching** — making decisions based on data shapes
6. **The ? operator** — propagating errors gracefully

These concepts will come up in every chapter of this book. Let's take the time to understand them now.

## Ownership Deep Dive

### The Rule of Ownership

In Rust, every value has exactly one **owner**. When the owner goes out of scope (usually when the function returns), the value is dropped (memory is freed). This happens automatically — no garbage collector needed.

```rust
fn main() {
    let title = String::from("Harry Potter");  // `title` owns the String
    println!("{}", title);
}  // `title` goes out of scope here → String is dropped (memory freed)
```

That's simple enough. But what happens when you try to give the value to another variable?

### Moves

```rust
fn main() {
    let title1 = String::from("My Story");
    let title2 = title1;  // Ownership moves from title1 to title2

    // println!("{}", title1);  // ❌ ERROR: title1 no longer owns the value
    println!("{}", title2);     // ✅ Works: title2 owns the value
}
```

When you write `let title2 = title1;`, Rust doesn't copy the string data. Instead, it **moves** ownership from `title1` to `title2`. After the move, `title1` is no longer valid. If you try to use it, you get a compiler error:

```
error[E0382]: borrow of moved value: `title1`
```

That might seem annoying, but it's the key to Rust's memory safety. There's never a situation where two variables both think they own the same data. When the data goes out of scope, it's cleaned up exactly once.

In languages like C or C++, you could have two pointers to the same memory. One pointer frees the memory, and the other becomes a "dangling pointer" — it points to memory that no longer exists. Accessing it causes crashes, security vulnerabilities, or silent data corruption. Rust's ownership system eliminates this entire class of bugs at compile time.

In languages with garbage collection (like Java or Python), the runtime tracks which variables point to which objects and only frees memory when nothing references it anymore. This is safe but has overhead — the garbage collector runs periodically, pausing your program to clean up. Rust doesn't need this because ownership is explicit: when the owner goes out of scope, the value is dropped immediately. No scanning, no pausing, no surprises.

### Clones

What if you actually want two copies of the data? Use `clone()`:

```rust
fn main() {
    let title1 = String::from("My Story");
    let title2 = title1.clone();  // Deep copy — title1 still valid

    println!("Title 1: {}", title1);  // ✅ Works
    println!("Title 2: {}", title2);  // ✅ Works
}
```

Cloning makes an independent copy. Both variables own their own data. But cloning costs time and memory, so Rust makes you be explicit about it.

### Copies

Some types are so small and simple that copying them is essentially free. These types implement the `Copy` trait, and assignment copies them instead of moving:

```rust
fn main() {
    let chapter_number: i32 = 5;  // i32 implements Copy
    let chapter_copy = chapter_number;  // Copied, not moved

    println!("Original: {}", chapter_number);  // ✅ Works
    println!("Copy: {}", chapter_copy);         // ✅ Works
}
```

Types that implement `Copy`: `i32`, `f64`, `bool`, `char`, and tuples of copyable types. Types that don't implement `Copy`: `String`, `Vec<T>`, anything that owns heap memory.

💡 **Key Concept: Move vs Copy**

The rule is simple:
- If a type is cheap to copy (integers, booleans, etc.), it's `Copy`. Assignment copies the value.
- If a type is expensive to copy (strings, vectors, etc.), it's not `Copy`. Assignment moves the value.

This is why you can use `i32` variables freely but need to be careful with `String` variables.

### Ownership and Functions

Functions work the same way — passing a value to a function moves ownership:

```rust
fn print_title(title: String) {
    println!("Title: {}", title);
}  // `title` is dropped here

fn main() {
    let my_title = String::from("My Fanfic");
    print_title(my_title);  // Ownership moves into print_title
    // println!("{}", my_title);  // ❌ Can't use my_title — it was moved!
}
```

After calling `print_title(my_title)`, `my_title` is no longer valid. If you need to use it again, you have two options:

1. Return the value from the function
2. Use a reference (borrowing) — which we'll learn next

```rust
// Option 1: Return the value
fn print_title(title: String) -> String {
    println!("Title: {}", title);
    title  // Return ownership to the caller
}

fn main() {
    let my_title = String::from("My Fanfic");
    let my_title = print_title(my_title);  // Rebind to returned value
    println!("Still valid: {}", my_title);
}
```

This works, but it's verbose. There's a much better way.

## Borrowing

Instead of transferring ownership, you can **borrow** a value by creating a reference. A reference lets you look at a value without taking ownership of it.

### Immutable References (&)

```rust
fn print_title(title: &String) {  // Borrow, don't take ownership
    println!("Title: {}", title);
}  // Reference is dropped, but the original data is untouched

fn main() {
    let my_title = String::from("My Fanfic");
    print_title(&my_title);  // Pass a reference
    println!("Still valid: {}", my_title);  // ✅ Works!
}
```

The `&` in `&String` means "I'm borrowing this value." The function can look at it, but it can't move or modify it. When the function returns, the borrow ends, and the original owner still has the value.

You can have as many immutable references as you want:

```rust
fn main() {
    let title = String::from("My Story");
    let r1 = &title;
    let r2 = &title;
    let r3 = &title;

    println!("{}, {}, {}", r1, r2, r3);  // ✅ All three work
}
```

### Mutable References (&mut)

If you need to modify a borrowed value, use `&mut`:

```rust
fn add_chapter_count(count: &mut i32) {
    *count += 1;  // Dereference with * to modify the value
}

fn main() {
    let mut chapter_count = 0;  // Must be `mut`
    add_chapter_count(&mut chapter_count);  // Mutable borrow
    println!("Chapters: {}", chapter_count);  // 1
}
```

⚠️ **Watch Out: The Borrowing Rules**

Rust has two strict rules about borrowing:

1. You can have **either** any number of immutable references (`&T`) **or** exactly one mutable reference (`&mut T`) at a time. Never both.
2. References must always be valid (no dangling references).

This prevents data races — the situation where two threads try to modify the same data at the same time. The compiler simply won't let it compile.

```rust
fn main() {
    let mut title = String::from("My Story");

    let r1 = &title;
    let r2 = &title;
    // let r3 = &mut title;  // ❌ ERROR: can't have &mut while & exists

    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point

    let r3 = &mut title;  // ✅ OK: r1 and r2 are done
    r3.push_str(" - Extended");
    println!("{}", r3);
}
```

This works because Rust uses **non-lexical lifetimes** (NLL): references are only tracked until their last use. After `println!`, `r1` and `r2` are no longer used, so their borrows have ended. Then the mutable borrow is fine.

🧪 **Try It Yourself: Borrowing Experiments**

Try these exercises to build your intuition:

1. Write a function that takes a `&String` and returns its length (hint: use `.len()`).
2. Write a function that takes a `&mut String` and adds " - Downloaded" to the end.
3. Try to have two `&mut` references to the same variable at the same time. What error do you get?

## Lifetimes Basics

Lifetimes are Rust's way of tracking how long a reference is valid. In most everyday code, the compiler figures out lifetimes automatically. You only need to annotate them when the compiler can't figure it out on its own.

### When You Need Lifetime Annotations

```rust
// ❌ This doesn't compile — the compiler doesn't know the return type's lifetime
fn longest(a: &str, b: &str) -> &str {
    if a.len() > b.len() { a } else { b }
}

// ✅ This works — we tell the compiler the return type lives as long as `a` and `b`
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
```

The `'a` (pronounced "lifetime a") is a **lifetime parameter**. It says: "the return reference lives as long as both `a` and `b`." The compiler uses this to make sure you don't return a reference to data that has been dropped.

### Lifetime Elision

In many common patterns, Rust can infer lifetimes without you writing them. These rules are called **lifetime elision**:

```rust
// The compiler knows this function has one input and one output,
// so the output lifetime equals the input lifetime. No annotation needed.
fn first_word(s: &str) -> &str {
    // ...
    s
}
```

You'll need to write lifetime annotations in about 5% of cases. When you do, the error message will tell you exactly what's needed.

Here's the key insight about lifetimes: a reference can never outlive the data it points to. If you create a reference to a local variable and try to return it, the compiler catches it:

```rust
// ❌ This doesn't compile
fn broken_reference() -> &str {
    let s = String::from("hello");
    &s  // Can't return reference to local data!
}  // `s` is dropped here — the reference would be dangling

// ✅ This works — return the owned value instead
fn working() -> String {
    let s = String::from("hello");
    s  // Transfer ownership to the caller
}
```

The compiler tracks where each value lives and makes sure references never outlive the data. This is the "borrow checker" — Rust's most distinctive feature. It catches use-after-free bugs, dangling pointers, and data races at compile time, not in production.

Think of it like a library book: you can borrow it (read it), but you can't let someone else borrow the same copy while you have it checked out. And you definitely can't borrow it after it's been returned to the shelf.

## The Option Type

What happens when a value might not exist? In many languages, you use `null` or `None`. Rust doesn't have null. Instead, it has `Option<T>`:

```rust
enum Option<T> {
    Some(T),  // There is a value
    None,     // There is no value
}
```

Here's how you use it:

```rust
fn find_chapter_title(story: &str, chapter: i32) -> Option<String> {
    if chapter == 1 {
        Some(String::from("The Beginning"))
    } else if chapter == 2 {
        Some(String::from("The Journey"))
    } else {
        None  // No title for this chapter
    }
}

fn main() {
    let title = find_chapter_title("My Story", 1);
    match title {
        Some(t) => println!("Found: {}", t),
        None => println!("No title found"),
    }

    let missing = find_chapter_title("My Story", 99);
    match missing {
        Some(t) => println!("Found: {}", t),
        None => println!("No title for chapter 99"),
    }
}
```

`Option` is Rust's way of saying "this might not exist." The compiler forces you to handle both cases — you can't accidentally use a value that might be `None`. This prevents null pointer exceptions, one of the most common bugs in programming.

### Common Option Methods

```rust
fn main() {
    let some_number: Option<i32> = Some(42);
    let no_number: Option<i32> = None;

    // unwrap() — get the value or panic
    println!("{}", some_number.unwrap());  // 42
    // no_number.unwrap();  // ❌ Panics! Don't use unwrap in production.

    // unwrap_or() — get the value or use a default
    println!("{}", no_number.unwrap_or(0));  // 0

    // map() — transform the inner value if it exists
    let doubled = some_number.map(|n| n * 2);
    println!("{:?}", doubled);  // Some(84)

    // is_some() / is_none() — check existence
    println!("Has value: {}", some_number.is_some());  // true
}
```

Here's a practical example — finding a chapter in a story:

```rust
fn find_chapter(chapters: &[&str], number: usize) -> Option<&str> {
    chapters.get(number).copied()
}

fn main() {
    let chapters = vec!["The Beginning", "The Journey", "The End"];

    match find_chapter(&chapters, 1) {
        Some(title) => println!("Chapter 1: {}", title),
        None => println!("Chapter not found"),
    }

    // Use unwrap_or for a default
    let title = find_chapter(&chapters, 99).unwrap_or("Unknown");
    println!("Chapter 99: {}", title);

    // Use map to transform
    let upper = find_chapter(&chapters, 0).map(|t| t.to_uppercase());
    println!("Chapter 0: {:?}", upper);  // Some("THE BEGINNING")
}
```

The `get()` method on slices returns `Option` instead of panicking on out-of-bounds access. This is idiomatic Rust — always prefer safe methods that return `Option` over indexing that can panic.
```

💡 **Key Concept: Option Eliminates Null**

Tony Hoare, the inventor of null, called it his "billion-dollar mistake." Null causes crashes, security holes, and bugs that are hard to find. Rust's `Option` type solves this problem at the compiler level. If a value might be missing, the type system makes you handle that case. You literally cannot forget.

## The Result Type

`Result<T, E>` is like `Option` but for operations that can fail with an error:

```rust
enum Result<T, E> {
    Ok(T),   // Success with a value
    Err(E),  // Failure with an error
}
```

Here's a practical example:

```rust
use std::num::ParseIntError;

fn parse_chapter_count(input: &str) -> Result<i32, ParseIntError> {
    input.parse::<i32>()
}

fn main() {
    let good = parse_chapter_count("25");
    let bad = parse_chapter_count("twenty-five");

    match good {
        Ok(count) => println!("Chapters: {}", count),
        Err(e) => println!("Error: {}", e),
    }

    match bad {
        Ok(count) => println!("Chapters: {}", count),
        Err(e) => println!("Error: {}", e),
    }
}
```

Output:

```
Chapters: 25
Error: invalid digit found in string
```

The `parse()` method returns a `Result` because parsing can fail. The compiler forces you to handle the error case — you can't just assume the input is always a valid number.

### Common Result Methods

```rust
fn main() {
    let number: Result<i32, _> = "42".parse();

    // unwrap_or() — use a default on error
    let val = number.unwrap_or(0);  // 42

    // map() — transform the success value
    let doubled = number.map(|n| n * 2);  // Ok(84)

    // and_then() — chain operations that can fail
    let result = number
        .and_then(|n| {
            if n > 0 { Ok(n) } else { Err("not positive") }
        });

    println!("{:?}", result);  // Ok(42)
}
```

## Pattern Matching with match

The `match` expression is one of Rust's most powerful features. It lets you branch based on the shape of your data:

```rust
fn download_status(url: &str) -> &str {
    match url {
        "" => "No URL provided",
        url if url.starts_with("https://archiveofourown.org") => "AO3 link detected",
        url if url.starts_with("https://www.fanfiction.net") => "FF.net link detected",
        url if url.starts_with("https://forums.spacebattles.com") => "SpaceBattles link",
        _ => "Unknown source",
    }
}

fn main() {
    println!("{}", download_status(""));
    println!("{}", download_status("https://archiveofourown.org/works/12345"));
    println!("{}", download_status("https://www.fanfiction.net/s/12345/1/"));
    println!("{}", download_status("https://forums.spacebattles.com/threads/123"));
    println!("{}", download_status("https://some-random-site.com/fic/1"));
}
```

Output:

```
No URL provided
AO3 link detected
FF.net link detected
SpaceBattles link
Unknown source
```

`match` is exhaustive — you must handle every possible case. The `_` at the end is the "catch-all" pattern. If you forget it and don't handle all cases, the compiler will refuse to compile.

### Match with Enums

Match shines when working with enums like `Option` and `Result`:

```rust
fn chapter_info(chapter: i32) -> Option<(&'static str, i32)> {
    match chapter {
        1 => Some(("The Beginning", 3500)),
        2 => Some(("The Journey", 4200)),
        3 => Some(("The Climax", 5800)),
        4 => Some(("The End", 2900)),
        _ => None,
    }
}

fn main() {
    for ch in 1..=5 {
        match chapter_info(ch) {
            Some((title, words)) => {
                println!("Chapter {}: \"{}\" ({} words)", ch, title, words);
            }
            None => {
                println!("Chapter {}: not found", ch);
            }
        }
    }
}
```

### if let: When You Only Care About One Case

If you only need to handle one pattern, `if let` is more concise than `match`:

```rust
fn main() {
    let maybe_title: Option<&str> = Some("My Story");

    // Using match (verbose for single case)
    match maybe_title {
        Some(title) => println!("Title: {}", title),
        None => {},  // Do nothing
    }

    // Using if let (concise)
    if let Some(title) = maybe_title {
        println!("Title: {}", title);
    }
}
```

🧪 **Try It Yourself: URL Classifier**

Write a function that takes a URL string and returns an `Option<&str>` indicating which fanfiction site it belongs to:

```rust
fn classify_url(url: &str) -> Option<&str> {
    if url.contains("archiveofourown.org") {
        Some("ao3")
    } else if url.contains("fanfiction.net") || url.contains("fictionpress.com") {
        Some("ffnet")
    } else if url.contains("spacebattles.com") || url.contains("sufficientvelocity.com") {
        Some("xenforo")
    } else {
        None
    }
}
```

Then use `match` on the result to print a friendly message for each site.

## The ? Operator

One of the most elegant features in Rust is the `?` operator. It lets you propagate errors up the call stack without writing lots of boilerplate code.

Consider this without `?`:

```rust
use std::fs;

fn read_story_title(path: &str) -> Result<String, std::io::Error> {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(e),  // Early return on error
    };

    let title = match contents.lines().next() {
        Some(t) => t.to_string(),
        None => return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No title found"
        )),
    };

    Ok(title)
}
```

Now with `?`:

```rust
use std::fs;

fn read_story_title(path: &str) -> Result<String, std::io::Error> {
    let contents = fs::read_to_string(path)?;  // ? = return Err if error
    let title = contents.lines()
        .next()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No title found"
        ))?
        .to_string();

    Ok(title)
}
```

The `?` does this:
1. If the `Result` is `Ok`, unwrap it and continue
2. If the `Result` is `Err`, return early from the function with that error

It's like saying "do this, and if it fails, stop here and return the error."

⚠️ **Watch Out: ? Only Works in Functions That Return Result**

You can only use `?` inside a function that returns `Result` (or `Option`). If you try to use `?` in a function that returns `()`, you'll get a compiler error:

```rust
fn main() {
    let content = std::fs::read_to_string("file.txt")?;  // ❌ Can't use ? here
    // because main returns (), not Result
}
```

To use `?` in `main`, change its return type:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("file.txt")?;
    println!("{}", content);
    Ok(())
}
```

## Thinking in Rust: A Summary

Let's recap the Rust mindset:

1. **Ownership**: Every value has one owner. When the owner goes out of scope, the value is dropped. This prevents memory leaks without a garbage collector.

2. **Borrowing**: You can lend a value to a function without giving up ownership. Use `&T` for reading, `&mut T` for modifying. The compiler enforces that you can't have conflicting borrows.

3. **Option**: Values might not exist. `Option<T>` makes this explicit. The compiler forces you to handle the `None` case.

4. **Result**: Operations might fail. `Result<T, E>` makes this explicit. The compiler forces you to handle the error case.

5. **Pattern matching**: `match` lets you branch based on data shape. It's exhaustive — you must handle all cases.

6. **The ? operator**: Propagate errors gracefully without boilerplate.

These concepts are interconnected. Ownership determines when data is dropped. Borrowing lets you share data safely. Option and Result express uncertainty. Pattern matching lets you handle every case. The ? operator chains fallible operations together.

It's a system where the compiler is your co-pilot, catching bugs before they happen. It's strict, but it's right. After a while, you'll wonder how you ever programmed without it.

---

# Chapter 5: Error Handling in Rust

## Why Error Handling Matters

Every program can fail. Files might not exist. Networks might be down. User input might be garbage. A fanfiction site might change its HTML structure and break your scraper. If your program doesn't handle these failures gracefully, it crashes — and users don't like crashes.

In many languages, error handling is an afterthought. Exceptions can be thrown from anywhere and caught anywhere, leading to spaghetti code where you never know which function might throw. Rust takes a different approach: errors are **values** that flow through your program explicitly. The compiler makes sure you handle them.

This isn't just an academic difference. It means FicHub can:
- Return a friendly error message when a URL is invalid
- Retry scraping when a site is temporarily down
- Log useful debugging information when something goes wrong
- Continue serving other requests even when one request fails

In this chapter, we'll learn how Rust's error handling works in practice, from basic `Result` patterns to custom error types and structured logging.

## Result and Option Together

We introduced `Result` and `Option` in Chapter 4. Now let's see them work together, because that's what real code looks like.

### Converting Between Result and Option

Sometimes you have a `Result` but you only care about success/failure, not the specific error. Convert it to an `Option`:

```rust
fn main() {
    let parsed: Result<i32, _> = "42".parse();
    let as_option: Option<i32> = parsed.ok();  // Some(42), error is dropped

    let failed: Result<i32, _> = "not a number".parse();
    let as_option: Option<i32> = failed.ok();  // None

    println!("{:?}", as_option);  // None
}
```

And sometimes you have an `Option` and want to attach an error message:

```rust
fn main() {
    let maybe_title: Option<&str> = None;
    let as_result: Result<&str, &str> = maybe_title.ok_or("Title not found");

    match as_result {
        Ok(title) => println!("Title: {}", title),
        Err(e) => println!("Error: {}", e),
    }
}
```

### Combining Option and Result in Real Code

Here's a more realistic example — looking up a chapter title from a database:

```rust
fn find_chapter(story_id: i32, chapter: i32) -> Option<String> {
    // Simulating a database lookup
    let chapters: Vec<(i32, String)> = vec![
        (1, "The Beginning".to_string()),
        (2, "The Journey".to_string()),
        (3, "The Climax".to_string()),
    ];

    chapters.into_iter()
        .find(|(num, _)| *num == chapter)
        .map(|(_, title)| title)
}

fn main() {
    // Try to find chapter 2
    let title = find_chapter(1, 2)
        .unwrap_or_else(|| format!("Chapter {} not found", 2));
    println!("Found: {}", title);

    // Try to find chapter 99
    let missing = find_chapter(1, 99)
        .unwrap_or_else(|| format!("Chapter {} not found", 99));
    println!("Result: {}", missing);
}
```

💡 **Key Concept: Design for Failure**

Every function that can fail should return `Result` or `Option`. Don't use `unwrap()` in production code — it panics (crashes) on `None` or `Err`. Instead, use:
- `unwrap_or(default)` — provide a fallback value
- `unwrap_or_else(|| ...)` — compute a fallback lazily
- `ok_or(err)` — convert `Option` to `Result`
- `?` — propagate the error to the caller

The only acceptable place for `unwrap()` is in quick prototypes and tests.

## Custom Error Types with thiserror

As your program grows, you'll need custom error types. Instead of using generic strings or `Box<dyn Error>`, you can define structured error types that tell you exactly what went wrong.

The `thiserror` crate makes this easy:

```toml
# Add to Cargo.toml
[dependencies]
thiserror = "2"
```

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum FicHubError {
    #[error("Failed to scrape URL: {url}")]
    ScrapeFailed { url: String },

    #[error("Invalid chapter number: {0}")]
    InvalidChapter(i32),

    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

Let's break this down:

- `#[derive(Error, Debug)]` — generates the `Error` trait implementation
- `#[error("...")]` — defines the error message for each variant
- `#[from]` — automatically generates a `From` implementation for converting from another error type

Now you can use this error type in your functions:

```rust
fn scrape_story(url: &str) -> Result<String, FicHubError> {
    if url.is_empty() {
        return Err(FicHubError::ScrapeFailed {
            url: url.to_string(),
        });
    }

    if url.len() < 10 {
        return Err(FicHubError::ScrapeFailed {
            url: url.to_string(),
        });
    }

    // Simulate successful scraping
    Ok(format!("Content from {}", url))
}

fn parse_chapter(input: &str) -> Result<i32, FicHubError> {
    let num: i32 = input.parse()
        .map_err(|_| FicHubError::InvalidChapter(0))?;  // Map parse error to our type
    if num < 1 {
        return Err(FicHubError::InvalidChapter(num));
    }
    Ok(num)
}

fn main() {
    let url = "https://archiveofourown.org/works/12345";
    match scrape_story(url) {
        Ok(content) => println!("Got content: {}", content),
        Err(e) => println!("Scrape error: {}", e),
    }

    let chapter = parse_chapter("5");
    println!("Chapter: {:?}", chapter);

    let bad_chapter = parse_chapter("-1");
    println!("Bad chapter: {:?}", bad_chapter);
}
```

Output:

```
Got content: Content from https://archiveofourown.org/works/12345
Chapter: Ok(5)
Bad chapter: Err(InvalidChapter(-1))
```

### Using thiserror with Enum Variants

Here's a more complete error type for FicHub:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Scrape failed for {url}: {reason}")]
    ScrapeFailed { url: String, reason: String },

    #[error("Export failed: {0}")]
    Export(String),

    #[error("Cache miss: {0}")]
    CacheMiss(String),

    #[error("Rate limited: try again in {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("HTTP error: {0}")]
    Http(String),
}
```

Each variant has its own data, making it easy to understand what happened when you look at an error in your logs.

## The From Trait for Error Conversion

The `From` trait lets you convert one error type into another. This is useful when you're calling functions that return different error types and want to unify them into a single error type.

```rust
use std::fmt;

#[derive(Debug)]
struct AppError {
    message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AppError: {}", self.message)
    }
}

// Convert std::io::Error into AppError
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError {
            message: format!("IO error: {}", err),
        }
    }
}

// Convert a string parse error into AppError
impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError {
            message: format!("Parse error: {}", err),
        }
    }
}

fn read_and_parse(path: &str) -> Result<i32, AppError> {
    let content = std::fs::read_to_string(path)?;  // io::Error → AppError
    let number = content.trim().parse::<i32>()?;    // ParseIntError → AppError
    Ok(number)
}
```

The `?` operator uses `From` automatically. When `read_to_string` returns an `io::Error`, the `?` calls `From<io::Error>::from(err)` to convert it into an `AppError`. You don't have to do anything — it just works.

⚠️ **Watch Out: One Direction Only**

`From` works in one direction. You can convert `io::Error` → `AppError`, but not `AppError` → `io::Error`. This is intentional — you can always wrap a more specific error in a more general one, but you can't unwrap a general error into a specific one.

## Propagating Errors with ?

The `?` operator is the backbone of error handling in Rust. Let's see it in a realistic FicHub scenario.

Imagine building the scrape → export pipeline:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ExportError {
    #[error("Scrape failed: {0}")]
    Scrape(String),

    #[error("Export failed: {0}")]
    Export(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn scrape_fic(url: &str) -> Result<String, ExportError> {
    // Simulate scraping
    if url.is_empty() {
        return Err(ExportError::Scrape("Empty URL".to_string()));
    }
    Ok(format!("Story content from {}", url))
}

fn generate_epub(content: &str, title: &str) -> Result<Vec<u8>, ExportError> {
    if title.is_empty() {
        return Err(ExportError::Export("Empty title".to_string()));
    }
    // Simulate EPUB generation
    Ok(format!("EPUB for: {} ({})", title, content).into_bytes())
}

fn save_to_disk(data: &[u8], path: &str) -> Result<(), ExportError> {
    std::fs::write(path, data)?;  // io::Error → ExportError via From
    Ok(())
}

fn export_fic(url: &str, path: &str) -> Result<(), ExportError> {
    let content = scrape_fic(url)?;     // Scrape fails? Propagate error.
    let epub = generate_epub(&content, "My Story")?;  // Export fails? Propagate.
    save_to_disk(&epub, path)?;         // IO fails? Propagate.
    Ok(())
}

fn main() {
    match export_fic("https://example.com/fic/123", "/tmp/fic.epub") {
        Ok(()) => println!("Export successful!"),
        Err(e) => println!("Export failed: {}", e),
    }

    match export_fic("", "/tmp/fic.epub") {
        Ok(()) => println!("Export successful!"),
        Err(e) => println!("Export failed: {}", e),
    }
}
```

Output:

```
Export successful!
Export failed: Scrape failed: Empty URL
```

The `?` operator makes the happy path clean and linear. Without it, you'd need nested `match` statements or `if let` chains, which get messy fast.

### ? with Option

`?` also works with `Option` in functions that return `Option`:

```rust
fn get_first_chapter_title(chapters: &[(&str, i32)]) -> Option<&str> {
    let (_title, _words) = chapters.first()?;  // None if empty
    Some(chapters.first()?.0)
}

fn main() {
    let chapters = vec![
        ("The Beginning", 3500),
        ("The Journey", 4200),
    ];

    println!("{:?}", get_first_chapter_title(&chapters));  // Some("The Beginning")

    let empty: Vec<(&str, i32)> = vec![];
    println!("{:?}", get_first_chapter_title(&empty));     // None
}
```

## Debugging with tracing

When things go wrong in production, you need to understand what happened. `println!` debugging works for simple programs, but servers need structured logging that you can filter, search, and analyze.

The `tracing` crate provides this:

```toml
[dependencies]
tracing = "0.1"
```

### Basic Tracing

```rust
use tracing::{info, warn, error, debug};

fn scrape_url(url: &str) -> Result<String, String> {
    info!(url = %url, "Starting scrape");

    if url.is_empty() {
        error!(url = %url, "Empty URL provided");
        return Err("Empty URL".to_string());
    }

    if !url.starts_with("http") {
        warn!(url = %url, "URL doesn't start with http");
        return Err("Invalid URL".to_string());
    }

    debug!("Fetching page content");
    let content = format!("Content from {}", url);
    info!(url = %url, content_length = content.len(), "Scrape complete");
    Ok(content)
}

fn main() {
    // Initialize the subscriber
    tracing_subscriber::fmt::init();

    let _ = scrape_url("https://example.com/fic/123");
    let _ = scrape_url("");
    let _ = scrape_url("not-a-url");
}
```

The `info!`, `warn!`, `error!`, and `debug!` macros create structured log entries. The `url = %url` syntax attaches the URL as a structured field, which you can filter and search later.

### The tracing-subscriber Crate

`tracing` provides the logging macros, but you need `tracing-subscriber` to actually output them:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

Here's how FicHub initializes logging (from `main.rs`):

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing with environment-filtered output
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

This sets up logging with an **env filter** — you control verbosity with the `RUST_LOG` environment variable:

```bash
# Default: info level for everything, debug for fichub
cargo run

# Debug everything
RUST_LOG=debug cargo run

# Only see errors
RUST_LOG=error cargo run

# See fichub scraping in detail
RUST_LOG=fichub::scrape=trace cargo run
```

💡 **Key Concept: Structured vs Unstructured Logging**

Unstructured logging (like `println!("User {} requested URL {}", user, url)`) is hard to search and analyze. Structured logging (like `info!(user = %user, url = %url, "Request received")`) produces JSON output that tools like Grafana, Loki, and Elasticsearch can parse and query. When you have 10,000 log entries and need to find the one where scraping failed, structured logging saves you.

### Adding Spans

Tracing also supports **spans**, which group related log events. Spans are like "regions" of code that have a beginning and end. All log events within a span inherit the span's fields. This makes it easy to see which request a log entry belongs to — critical for debugging a web server handling many concurrent requests.

```rust
use tracing::{info_span, Instrument};

async fn scrape_story(url: &str) -> Result<String, String> {
    let span = info_span!("scrape", url = %url);
    let _guard = span.enter();

    info!("Starting scrape");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    info!("Fetch complete");

    Ok(format!("Content from {}", url))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let result = scrape_story("https://example.com").await;
    println!("{:?}", result);
}
```

Spans are like "regions" of code that have a beginning and end. All log events within a span inherit the span's fields. This makes it easy to see which request a log entry belongs to — critical for debugging a web server handling many concurrent requests.

## Putting It All Together: A FicHub Error Module

Let's build a complete error module like the one FicHub actually uses. First, look at the real `error.rs` from the FicHub project:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Scrape error: {0}")]
    Scrape(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Rate limited: {0}")]
    RateLimit(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),
}
```

This error type is used throughout FicHub. Each variant represents a different category of failure:

- `Scrape` — the scraper couldn't fetch or parse a page
- `Export` — EPUB/HTML generation failed
- `Database` — a SQL query failed
- `Cache` — Redis operations failed
- `RateLimit` — a client is making too many requests
- `NotFound` — the requested story doesn't exist
- `BadRequest` — the client sent invalid input

The beauty of this approach is that each error carries enough context to debug the problem. When you see `AppError::Scrape("AO3 returned 403 for https://...")` in your logs, you know exactly what happened and where to look.

🧪 **Try It Yourself: Build Your Own Error Type**

Create a small project that demonstrates error handling:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum FicError {
    #[error("Story not found: {url}")]
    NotFound { url: String },

    #[error("Chapter {chapter} out of range (1-{total})")]
    OutOfRange { chapter: i32, total: i32 },

    #[error("Network error: {0}")]
    Network(String),
}

fn get_chapter(url: &str, chapter: i32) -> Result<String, FicError> {
    let total_chapters = 10;  // Simulated

    if url.is_empty() {
        return Err(FicError::NotFound {
            url: url.to_string(),
        });
    }

    if chapter < 1 || chapter > total_chapters {
        return Err(FicError::OutOfRange { chapter, total: total_chapters });
    }

    Ok(format!("Chapter {} of {}", chapter, url))
}

fn main() {
    let tests = vec![
        ("https://example.com/fic", 5),
        ("", 1),
        ("https://example.com/fic", 15),
    ];

    for (url, chapter) in tests {
        match get_chapter(url, chapter) {
            Ok(content) => println!("✅ {}", content),
            Err(e) => println!("❌ {}", e),
        }
    }
}
```

Run it and see the different error messages. Then try adding a new error variant and handling it in `main`.

## What's Ahead

You now have a solid foundation in Rust fundamentals: variables, functions, ownership, borrowing, Option, Result, pattern matching, error handling, and logging. These concepts will appear in every chapter of this book.

In the next part, we'll build our first web server with Axum. We'll take everything we've learned and apply it to handling HTTP requests, routing URLs, and returning JSON responses. The pieces are starting to come together.

The foundation is laid. Time to build.

---

# Part 1 Summary

Congratulations! You've completed Part 1 of the FicHub Backend book. Here's what you learned:

- **Chapter 1**: What FicHub is, how it works (scrape → export → download), why Rust is the right choice, and what you'll build over the course of this book.

- **Chapter 2**: How to install Rust, PostgreSQL, Redis, and Node.js. How to create a new Rust project with `cargo new`, and how to build and run it.

- **Chapter 3**: Rust fundamentals — variables, mutability, basic types (integers, floats, booleans, chars, strings), functions, the `println!` macro, and comments.

- **Chapter 4**: Rust's unique features — ownership (moves, clones, copies), borrowing (`&` and `&mut` references), lifetimes, Option, Result, pattern matching with `match`, and the `?` operator.

- **Chapter 5**: Error handling in depth — combining Result and Option, custom error types with `thiserror`, the `From` trait for error conversion, propagating errors with `?`, and structured logging with `tracing` and `tracing-subscriber`.

You have the Rust fundamentals. In Part 2, we'll build our first web server with Axum — handling HTTP requests, routing URLs, parsing JSON, and building the foundation that the rest of FicHub runs on.

Ready for the next chapter? Let's keep building!

---

*End of Part 1: Welcome to Rust Backend Development*
