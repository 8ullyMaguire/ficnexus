# Building the FicHub Backend

## A Complete Guide to Building a Fanfiction Download Server in Rust

**Version 1.0 — July 2026**

---

> *"Every great adventure begins with a single line of code."*

---

Welcome, young coder! 🎉

Have you ever wanted to download your favorite fanfiction stories so you can read them on your phone, tablet, or e-reader — even when you don't have internet? That's exactly what **FicHub** does! FicHub is a web server that takes a fanfiction URL from sites like Archive of Our Own (AO3) or FanFiction.net and turns it into a beautiful EPUB file you can read anywhere.

But here's the really cool part: FicHub doesn't just download stories. It remembers them, organizes them, suggests new ones you might like, lets people add tags to stories, and even creates a special book catalog called OPDS that your e-reader can browse!

In this book, we're going to build the **entire FicHub backend** from scratch using **Rust** — one of the most exciting programming languages in the world. We'll use **Axum** to handle web requests, **PostgreSQL** to store data, and **Redis** to make everything lightning fast.

### What You'll Learn

By the time you finish this book, you'll know how to:

- Write a web server in Rust using Axum
- Connect to PostgreSQL and run database queries
- Scrape fanfiction websites to extract story information
- Generate EPUB files that work on any e-reader
- Cache files on disk so you don't regenerate them
- Rate-limit requests so you don't overwhelm fanfiction sites
- Build a recommendation engine that finds similar stories
- Create OPDS catalogs for e-readers
- Deploy everything with Docker

### Who Is This Book For?

This book is for anyone who knows a little bit of programming and wants to learn Rust by building something real. We'll explain every concept as we go, so even if you've never used Rust before, you'll be able to follow along. We use a friendly, kid-friendly tone — because learning should be fun!

### The FicHub Source Code

The complete FicHub source code lives in a project with this structure:

```
fichub/
├── src/
│   ├── main.rs          # Where it all begins!
│   ├── lib.rs           # Library root — lists all modules
│   ├── config.rs        # Configuration from environment
│   ├── server.rs        # Axum server & router
│   ├── error.rs         # Error handling
│   ├── db/              # Database connection & queries
│   ├── scrape/          # Web scrapers for fanfiction sites
│   ├── export/          # EPUB & HTML generation
│   ├── cache/           # Disk caching system
│   ├── limiter/         # Rate limiting with Redis
│   ├── tags/            # Community tagging system
│   ├── search/          # Advanced search engine
│   ├── recommender/     # Recommendation engine
│   ├── routes/          # API route handlers
│   └── frontend/        # Frontend SPA serving
├── Cargo.toml           # Project dependencies
├── migrations/          # Database migrations
└── Dockerfile           # Container setup
```

Let's start our adventure!

---

# Chapter 1: Welcome to FicHub — The Big Picture

## What Is FicHub?

FicHub is a **self-hosted fanfiction download server**. That's a fancy way of saying it's a program you run on your own computer (or a server) that can grab fanfiction stories from the internet and turn them into files you can read offline.

Think of it like a librarian who lives inside your computer. You tell the librarian, "Hey, I want to read this story from AO3!" The librarian goes to AO3, reads the story, types it all up into a nice EPUB book, and hands it back to you. And the best part? The librarian remembers every story it's ever read, so if someone else asks for the same story, it can hand it over instantly without going back to AO3.

## Why Rust?

FicHub is written in **Rust**. Rust is a programming language made by Mozilla (the people who make Firefox). Here's why Rust is perfect for FicHub:

1. **It's fast** — Rust is as fast as C++, which means FicHub can handle lots of requests at once. When hundreds of people are trying to download stories at the same time, Rust doesn't break a sweat.

2. **It's safe** — Rust won't let you make the kinds of mistakes that cause programs to crash or have security holes. The Rust compiler catches bugs before your program even runs.

3. **It's modern** — Rust has wonderful tools for building web servers, databases, and more. The ecosystem is growing every day with new libraries (called "crates") that make common tasks easy.

4. **It's fun** — Once you get the hang of it, Rust's compiler is like a helpful friend who catches your mistakes before they become problems. Instead of finding bugs at 3 AM when your server crashes, Rust finds them while you're still typing!

## The Journey Ahead

In this book, we'll build FicHub piece by piece. Here's what each part does:

**Part 1 — Foundations (Chapters 1–4):**
We'll set up our Rust project, learn about configuration, and write our first program. You'll understand how FicHub starts up and loads its settings.

**Part 2 — The Web Server (Chapters 5–7):**
We'll build the Axum web server, create API endpoints, and learn how to handle errors gracefully. This is where FicHub starts accepting requests from the outside world.

**Part 3 — Data Storage (Chapter 8):**
We'll connect to PostgreSQL and learn how to store and retrieve fanfiction metadata. Every story FicHub has ever processed lives in this database.

**Part 4 — Web Scraping (Chapters 9–11):**
We'll learn how to visit fanfiction websites and extract story information from their HTML pages. This is the magic that turns a URL into a downloadable book.

**Part 5 — File Generation (Chapters 12–13):**
We'll generate EPUB files and HTML bundles that readers can enjoy. We'll learn how EPUB files are structured and how to create them from scratch.

**Part 6 — Performance (Chapters 14–15):**
We'll add caching and rate limiting to make FicHub fast and respectful. Caching means we don't regenerate files unnecessarily, and rate limiting means we don't overwhelm the sites we scrape.

**Part 7 — Community Features (Chapters 16–18):**
We'll build tagging, search, and recommendation systems. These features let the FicHub community help each other find great stories.

**Part 8 — Advanced Features (Chapters 19–20):**
We'll add OPDS catalog support and Docker deployment. OPDS lets e-readers browse FicHub's library, and Docker makes deployment a breeze.

Let's dive in!

---

> ### 🔧 Try It Yourself
> 
> Before we start coding, let's make sure you have everything set up:
> 
> 1. Install Rust by visiting [rustup.rs](https://rustup.rs) and following the instructions
> 2. Open a terminal and type `rustc --version` — you should see a version number like `rustc 1.78.0`
> 3. Type `cargo --version` — you should see another version number
> 4. Type `cargo new --help` to see what Cargo can do
> 
> If both commands work, you're ready to go! If not, re-read the installation instructions at rustup.rs.

# Chapter 2: Hello, Rust! — Your First Steps

## What Is a Programming Language?

A programming language is how we talk to computers. Just like you speak English or Spanish to talk to people, we use languages like Rust to tell computers what to do. When you write a Rust program, you're giving the computer a recipe — step-by-step instructions for what to build.

Computers are incredibly fast and powerful, but they're also incredibly literal. They do exactly what you tell them to do, nothing more and nothing less. That's why programming is both an art and a science — you need to be precise and creative at the same time.

## Your Very First Rust Program

Let's start with the simplest program possible. Open a terminal and create a new Rust project:

```bash
cargo new hello_fichub
cd hello_fichub
```

This creates a folder called `hello_fichub` with a file called `src/main.rs`. Open `src/main.rs` and you'll see:

```rust
fn main() {
    println!("Hello, world!");
}
```

Let's break this down:

- `fn main()` — This defines the **main function**. Every Rust program starts running from `main()`. Think of it as the "front door" of your program.
- `println!("Hello, world!")` — This prints text to the screen. The `!` means it's a **macro** (a special shortcut that does more than a regular function).
- The curly braces `{}` contain the code inside the function.
- The semicolon `;` at the end of the line tells Rust "this statement is done."

Run it with:

```bash
cargo run
```

You should see `Hello, world!` printed to your screen. Congratulations — you just wrote your first Rust program! 🎉

## Understanding Compilation

When you run `cargo build` or `cargo run`, Rust does something special called **compilation**. It translates your human-readable code into machine code that the computer can execute directly. This is different from languages like Python, which are interpreted (read line by line at runtime).

Compilation has a big advantage: Rust can catch errors before your program ever runs. If you make a typo or try to do something that doesn't make sense, the compiler will tell you about it immediately. This is like having a proofreader for your code!

## Variables and Types

In Rust, we use **variables** to store information. Here's how:

```rust
fn main() {
    let title = "Harry Potter";       // A string (text)
    let chapters = 17;                 // An integer (whole number)
    let words = 108_462;              // You can use underscores for big numbers!
    let is_complete = true;           // A boolean (true or false)

    println!("Title: {}", title);
    println!("Chapters: {}", chapters);
    println!("Words: {}", words);
    println!("Complete: {}", is_complete);
}
```

### Types of Variables

Rust has different **types** for different kinds of data:

| Type | What It Stores | Example |
|------|---------------|---------|
| `&str` | Text (string slice) | `"Hello"` |
| `String` | Text (owned, growable) | `String::from("Hello")` |
| `i32` | Integer (whole number) | `42` |
| `i64` | Large integer | `1_000_000` |
| `f64` | Floating point (decimal) | `3.14` |
| `bool` | True or false | `true` |
| `Option<T>` | Maybe something, maybe nothing | `Some(42)` or `None` |
| `Result<T, E>` | Success or failure | `Ok(42)` or `Err("oops")` |

### String vs &str

You might wonder why there are two string types. `&str` is a "string slice" — it's a view into text that already exists somewhere. `String` is an owned string that the variable controls. Think of `&str` like reading a page in a book (you can see the text, but you don't own the book), and `String` like having your own notebook (you can write in it and change it).

### Mutability

By default, variables in Rust can't be changed after they're created. This is called **immutability**. If you want to change a variable, you need to add `mut`:

```rust
fn main() {
    let mut word_count = 0;
    word_count = word_count + 500;  // This works because we used mut!
    println!("Word count: {}", word_count);
}
```

Why does Rust default to immutable? Because immutable code is safer and easier to reason about. When you see a variable without `mut`, you know it won't change — making it easier to understand what the code does.

> ### ⚠️ Watch Out!
> 
> If you try to change a variable without `mut`, Rust will give you an error. This is actually a good thing — it helps you catch mistakes early! The error message will be something like: "cannot assign twice to immutable variable `word_count`".

## Functions

Functions are reusable blocks of code. Here's how to write one:

```rust
fn calculate_read_time(word_count: i64) -> i64 {
    // Average reading speed is about 200 words per minute
    word_count / 200
}

fn main() {
    let words = 10_000;
    let minutes = calculate_read_time(words);
    println!("This story takes about {} minutes to read!", minutes);
}
```

Let's look at the function definition:

- `fn calculate_read_time(word_count: i64) -> i64` — This says "a function called `calculate_read_time` that takes one parameter called `word_count` of type `i64`, and returns an `i64`".
- Inside the function, `word_count / 200` is the last expression — in Rust, the last expression is automatically the return value! You don't need to write `return` (though you can if you want).

### Naming Conventions

Rust follows these naming conventions:
- **Functions** use `snake_case`: `calculate_read_time`, `get_fic_info`
- **Variables** use `snake_case`: `word_count`, `is_complete`
- **Types** use `PascalCase`: `FicMetadata`, `AppError`
- **Constants** use `SCREAMING_SNAKE_CASE`: `MAX_CONNECTIONS`, `DEFAULT_PORT`

### The FicHub Way

In FicHub, we have functions like `generate_url_id` that create unique identifiers for stories:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    // Use first 12 hex chars for a compact but unique ID
    hex::encode(&result[..6])
}
```

This function takes a `source_id` (like `1` for AO3) and a `story_id` (like `12345`) and creates a unique string like `"a1b2c3d4e5f6"`. It uses something called a **hash** — a mathematical trick that turns any input into a fixed-size output. The same input always produces the same output, which is important for consistency.

The `pub` keyword means this function is public — other parts of the code can use it. Without `pub`, the function would be private to its module.

## Enums and Pattern Matching

One of Rust's coolest features is **enums** — types that can be one of several different things:

```rust
enum ExportType {
    Epub,
    Html,
    Mobi,
    Pdf,
}

fn get_file_extension(export: &ExportType) -> &str {
    match export {
        ExportType::Epub => ".epub",
        ExportType::Html => ".zip",
        ExportType::Mobi => ".mobi",
        ExportType::Pdf => ".pdf",
    }
}
```

The `match` statement is like a switch — it checks which variant the enum is and runs the corresponding code. FicHub uses enums all over the place, like `AppError` (for different kinds of errors) and `RateLimitResult` (for rate-limiting outcomes).

### Enums with Data

Enums in Rust can also hold data:

```rust
enum RateLimitResult {
    Allowed,
    Wait(u64),    // How many seconds to wait
    Blocked,      // IP is blocked
}
```

Each variant can have different associated data. This is much more expressive than using separate variables or magic numbers.

### Why Enums Are Great

Enums prevent an entire class of bugs. If you use a string like `"epub"` to represent export types, you might accidentally type `"epu"` and not notice until runtime. With enums, the compiler catches this immediately. Plus, `match` on enums is exhaustive — if you forget to handle a case, the compiler tells you.

## Structs: Grouping Related Data

Structs let you bundle related data together:

```rust
struct Chapter {
    chapter_id: i32,
    title: String,
    content: String,
}

fn print_chapter(chapter: &Chapter) {
    println!("Chapter {}: {}", chapter.chapter_id, chapter.title);
}
```

FicHub uses structs everywhere:

```rust
pub struct FicMetadata {
    pub url_id: String,
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,
    pub status: String,
    pub source: String,
    // ... and more fields
}
```

### The `pub` Keyword

You'll notice `pub` before struct fields in FicHub. This makes the fields public — other modules can read and write them. Without `pub`, the fields would be private (only accessible within the same module).

## Error Handling with Result

Rust's `Result` type is how we handle things that might fail:

```rust
fn parse_chapter_count(text: &str) -> Result<i32, String> {
    match text.parse::<i32>() {
        Ok(num) => Ok(num),
        Err(_) => Err(format!("'{}' is not a valid number", text)),
    }
}

fn main() {
    match parse_chapter_count("10") {
        Ok(count) => println!("Found {} chapters", count),
        Err(e) => println!("Error: {}", e),
    }
}
```

The `?` operator makes error handling even cleaner:

```rust
fn do_something() -> Result<(), Box<dyn std::error::Error>> {
    let data = std::fs::read_to_string("config.json")?;  // Returns error if file not found
    let value: serde_json::Value = serde_json::from_str(&data)?;  // Returns error if invalid JSON
    Ok(())
}
```

If any step fails, `?` immediately returns the error. If it succeeds, you get the value. This makes error handling automatic and painless.

## Loops and Iterators

Rust has several ways to repeat things:

```rust
fn main() {
    // For loop
    for i in 1..=5 {
        println!("Chapter {}", i);
    }

    // While loop
    let mut count = 0;
    while count < 5 {
        count += 1;
        println!("Count: {}", count);
    }

    // Iterators (the Rust way)
    let chapters = vec!["Intro", "Chapter 1", "Chapter 2", "Conclusion"];
    for (i, title) in chapters.iter().enumerate() {
        println!("{}. {}", i + 1, title);
    }
}
```

Iterators are especially powerful in Rust. You can chain operations like `filter`, `map`, and `collect`:

```rust
let words = vec!["hello", "world", "rust", "is", "awesome"];
let long_words: Vec<&str> = words.iter()
    .filter(|w| w.len() > 3)
    .copied()
    .collect();
// long_words = ["hello", "world", "rust", "awesome"]
```

> ### 🔧 Try It Yourself
> 
> 1. Create a function that takes a word count and returns whether a story is a "short story" (under 7,500 words), a "novella" (7,500 to 40,000 words), or a "novel" (over 40,000 words). Use an enum for the return type!
> 2. Write a function that takes a title and removes all non-alphanumeric characters (replacing them with underscores).
> 3. What happens if you try to use a variable after moving it? (Try it with `String`!)

# Chapter 3: Cargo & Dependencies — Your Tool Belt

## What Is Cargo?

Cargo is Rust's package manager and build tool. Think of it like an app store for code libraries. When you want to use someone else's code (a "crate"), you tell Cargo about it, and it downloads and sets everything up for you.

Cargo handles many things for you:
- **Building** your project (compiling Rust code into a program)
- **Running** your project (starting the compiled program)
- **Testing** your project (running automated tests)
- **Managing dependencies** (downloading and updating libraries)
- **Documentation** (generating beautiful docs from your code comments)
- **Publishing** (sharing your crate with the world on crates.io)

When you type `cargo build`, Cargo:
1. Reads your `Cargo.toml` to see what dependencies you need
2. Downloads any dependencies you don't already have
3. Compiles your code and all dependencies
4. Puts the finished program in `target/debug/`

## The Cargo.toml File

Every Rust project has a file called `Cargo.toml`. This is like a recipe card that describes your project. Here's FicHub's:

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
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate", "tls-rustls-ring", "derive", "macros"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# HTTP client
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }

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
zip = { version = "8", package = "zip" }

# Async traits
async-trait = "0.1"

# Path handling
sanitize-filename = "0.5"

# Hashing/encoding
hex = "0.4"
regex-lite = "0.1"
rand = "0.8"
ipnetwork = "0.20"

[dev-dependencies]
# Testing
axum-test = "16"
```

Let's break this down:

### The [package] Section

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"
description = "Self-hosted fanfiction download server (fichub.net replacement)"
```

- `name` — What the project is called. This is used when you publish the crate.
- `version` — The version number. `0.1.0` means it's the very first version (0 = major, 1 = minor, 0 = patch).
- `edition` — Which version of Rust we're using. The 2024 edition has the latest features.
- `description` — A short description that appears on crates.io if you publish.

### The [dependencies] Section

This is where the magic happens. Each line adds an external library (crate) that FicHub uses:

```toml
axum = "0.8"
```

This simple line means "use version 0.8 of the axum crate." Cargo will download it and all its dependencies automatically.

## Our Key Dependencies

Let's look at the most important dependencies and what they do:

### Axum — Our Web Framework

```toml
axum = "0.8"
```

Axum is what handles all the incoming web requests. When someone visits `http://localhost:3000/api/v0/epub?q=https://ao3.org/works/12345`, Axum figures out which function should handle that request. It's built on top of Tower, which provides a powerful middleware system.

Axum was created by the team behind Tokio, so it integrates perfectly with Rust's async ecosystem. It's type-safe, which means if you make a mistake in your handler signatures, the compiler catches it.

### Tokio — Our Async Runtime

```toml
tokio = { version = "1", features = ["full"] }
```

Tokio is like the engine that makes everything run. Rust programs that do many things at once (like handling multiple web requests) need an "async runtime," and Tokio is the best one. The `features = ["full"]` enables all of Tokio's capabilities, including networking, timers, file I/O, and synchronization primitives.

Think of Tokos as a team of workers. When a request comes in, Tokio assigns a worker to handle it. While that worker is waiting for the database to respond, Tokio can use the same worker to handle another request. This is how FicHub can handle thousands of requests simultaneously.

### SQLx — Our Database Driver

```toml
sqlx = { version = "0.9", default-features = false, features = ["runtime-tokio", "postgres", "chrono", "uuid", "migrate", "tls-rustls-ring", "derive", "macros"] }
```

SQLx lets us talk to PostgreSQL. It's special because it checks your SQL queries at compile time — meaning it catches database mistakes before you even run the program! This feature is called "compile-time checked queries."

The features we enable:
- `runtime-tokio` — Use Tokio as the async runtime
- `postgres` — Support for PostgreSQL
- `chrono` — Date/time handling
- `uuid` — UUID support
- `migrate` — Automatic database migrations
- `tls-rustls-ring` — Secure database connections
- `derive` — Automatic struct derivation for database rows
- `macros` — Compile-time query checking

### Redis — Our Caching Helper

```toml
redis = { version = "1.4", features = ["aio", "tokio-comp"] }
```

Redis is a super-fast storage system that lives in memory. FicHub uses it for:
- Rate limiting (tracking how many requests each IP makes)
- Temporary data storage (like queuing background tasks)
- Session caching

The `aio` feature enables async I/O, and `tokio-comp` integrates with Tokio.

### Reqwest — Our HTTP Client

```toml
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
```

Reqwest is how FicHub talks to fanfiction websites. When we need to download a story from AO3, Reqwest makes the HTTP request. It supports:
- `json` — Automatic JSON serialization/deserialization
- `rustls-tls` — Secure HTTPS connections using Rustls (a pure-Rust TLS implementation)

### Scraper — Our HTML Parser

```toml
scraper = "0.27"
```

The `scraper` crate lets us parse HTML pages and find specific elements using CSS selectors. This is how we extract story titles, authors, and chapter content from fanfiction websites. It's like having a robot that can read web pages and pull out exactly the information we need.

### EPUB Builder — Our Book Maker

```toml
epub-builder = "0.8"
```

`epub-builder` creates EPUB files — the format used by most e-readers. It handles all the complicated EPUB structure (which is basically a ZIP file with specific XML files inside) so we just need to give it a title, author, and chapter content.

### Serde — Our Serialization Library

```toml
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Serde (Serialize/Deserialize) is one of Rust's most popular crates. It converts Rust structs to JSON and back. The `derive` feature lets you add automatic serialization to your structs with a single line:

```rust
#[derive(Serialize, Deserialize)]
struct Story {
    title: String,
    author: String,
    word_count: i64,
}
```

Now you can convert `Story` to JSON with `serde_json::to_string(&story)` or parse JSON into `Story` with `serde_json::from_str::<Story>(json)`.

### Tracing — Our Logging System

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
```

Tracing is a logging framework designed for async Rust. It lets you:
- Log messages at different levels (info, warn, error)
- Track how long operations take
- See which request triggered which log message
- Output logs in human-readable or JSON format

## Adding a New Dependency

To add a new crate to your project, run:

```bash
cargo add crate_name
```

Or you can edit `Cargo.toml` manually and then run:

```bash
cargo build
```

Cargo will download the new crate and all its dependencies automatically. It caches everything in the `target/` directory, so subsequent builds are fast.

## Dev Dependencies — Tools for Testing

At the bottom of `Cargo.toml`, you'll see:

```toml
[dev-dependencies]
axum-test = "16"
```

**Dev dependencies** are only used when you're testing, not when the program is running for real. `axum-test` lets us test our web server without actually starting it up. It creates an in-memory server that responds to requests, which is much faster than starting a real server.

## Understanding Features

You'll notice some dependencies have `features` listed. A "feature" is like an optional add-on:

```toml
tokio = { version = "1", features = ["full"] }
```

The `"full"` feature for Tokio enables everything Tokio can do. Without it, Tokio would only have basic features. You only enable what you need to keep your program small and compile times fast.

Features are defined in the crate's own `Cargo.toml`. When you enable a feature, you're telling Cargo "I want this crate with these optional capabilities enabled."

> ### ⚠️ Watch Out!
> 
> When you first build a new Rust project, Cargo downloads and compiles all dependencies. This can take a while the first time (sometimes 5-10 minutes for large projects)! Future builds will be much faster because Cargo caches everything. If you change dependencies, Cargo will only recompile what changed.

## The Cargo.lock File

After your first build, Cargo creates a `Cargo.lock` file. This file records the exact versions of all dependencies that were used. It ensures that everyone who builds your project uses the same versions, even if newer versions are available.

**Important**: Always commit `Cargo.lock` to version control! It ensures reproducible builds.

## Cargo Commands Cheat Sheet

Here are the most useful Cargo commands:

| Command | What It Does |
|---------|-------------|
| `cargo new name` | Create a new project |
| `cargo build` | Build the project (debug mode) |
| `cargo build --release` | Build with optimizations (for production) |
| `cargo run` | Build and run the project |
| `cargo test` | Run all tests |
| `cargo doc --open` | Generate and open documentation |
| `cargo add crate` | Add a dependency |
| `cargo update` | Update dependencies to latest compatible versions |
| `cargo clippy` | Run the linter for extra style checks |
| `cargo fmt` | Auto-format your code |

> ### 🔧 Try It Yourself
> 
> 1. Create a new Rust project: `cargo new my_fichub_tool`
> 2. Add `reqwest` as a dependency: `cargo add reqwest`
> 3. Add `serde` with the `derive` feature: `cargo add serde --features derive`
> 4. Add `tokio` with the `full` feature: `cargo add tokio --features full`
> 5. Look at your `Cargo.toml` — what changed?
> 6. Run `cargo build` — how long did it take?
> 7. Run `cargo build` again — much faster, right?


# Chapter 4: Configuration & main.rs — Setting Up Shop

## The Entry Point: main.rs

Every Rust program starts with `main.rs`. This is the "front door" — where the program begins running. When you type `cargo run`, Rust compiles your code and then calls the `main()` function. Everything happens from there.

Here's FicHub's `main.rs`:

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();

    // Load configuration
    let config = config::Config::from_env();

    tracing::info!("Starting fichub-rs server on port {}", config.app_port);

    // Run the server
    server::run(config).await;
}
```

Let's walk through each part in detail:

### The `pub mod` Lines

```rust
pub mod cache;
pub mod config;
pub mod db;
// ... and so on
```

Each `pub mod` line tells Rust "hey, there's a module (a piece of the project) with this name." This is like saying, "In my house, there's a kitchen, a bedroom, a living room..." Each module is a folder with its own Rust files.

The `pub` keyword means these modules are public — other parts of the code (and external users if this were a library) can access them. Without `pub`, the modules would be private.

FicHub has 13 modules:
- `cache` — Disk caching and semaphores
- `config` — Configuration from environment variables
- `db` — Database connection and queries
- `error` — Error types and conversions
- `export` — EPUB and HTML generation
- `frontend` — Frontend SPA serving
- `limiter` — Rate limiting with Redis
- `recommender` — Recommendation engine
- `routes` — API route handlers
- `scrape` — Web scrapers for fanfiction sites
- `search` — Advanced search engine
- `server` — Axum server setup and routing
- `tags` — Community tagging system

### The `#[tokio::main]` Attribute

```rust
#[tokio::main]
async fn main() {
```

This magic line tells Rust: "When this program starts, use Tokio as the async runtime." Without this, our `async` code wouldn't work. Think of it like plugging in the power cord — Tokio is the electricity that makes everything run.

The `async` keyword before `main` means this function is asynchronous. Async functions can "pause" while waiting for something (like a database query or network request) and "resume" when the result is ready. This lets the program handle many things at once without using multiple threads for every wait.

### Loading Environment Variables

```rust
dotenvy::dotenv().ok();
```

This line loads settings from a `.env` file. It's like a secret notebook where you write down your database password and other settings. The `.ok()` means "if the file doesn't exist, that's fine — don't panic."

The `.env` file might look like this:

```
DATABASE_URL=postgres://localhost/fichub
REDIS_URL=redis://localhost/
CACHE_DIR=./cache
PORT=3000
```

### Setting Up Logging

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
    )
    .init();
```

This sets up **logging** — a way for the program to write messages about what it's doing. When something goes wrong, you can read the logs to figure out what happened.

The `EnvFilter` lets you control which messages appear by setting the `RUST_LOG` environment variable. For example:
- `RUST_LOG=info` — Show info, warn, and error messages
- `RUST_LOG=debug` — Show debug messages too
- `RUST_LOG=fichub=debug` — Show debug messages only from FicHub

The default is `info,fichub=debug`, which shows info-level messages everywhere but debug-level messages from FicHub (because FicHub is the interesting part!).

### Loading Configuration

```rust
let config = config::Config::from_env();
```

This reads all the settings from environment variables and puts them into a `Config` struct. Let's look at what `Config` contains.

## The Config Struct

The Config struct holds all the settings FicHub needs:

```rust
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
    pub rec_max_recommendations: usize,
    pub rec_voting_boost_gamma: f64,
    pub rec_cache_ttl_hours: u32,
    // Tagging settings
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    pub search_max_per_page: usize,
    pub opds_shelf_token: String,
}
```

That's a lot of settings! Let's group them:

**Required settings** (crash if missing):
- `database_url` — Where PostgreSQL lives
- `redis_url` — Where Redis lives

**Optional settings with defaults:**
- `cache_dir` — Defaults to `./cache`
- `app_port` — Defaults to `3000`
- `node_name` — Defaults to `"orion"`
- `export_version` — Defaults to `1`
- `dynamic_rate_limit` — Defaults to `true`

**Recommender settings:**
- `rec_default_delay_secs` — Delay between scraping requests (default: 5)
- `rec_max_recommendations` — Max recommendations to return (default: 20)
- `rec_voting_boost_gamma` — How much community votes boost scores (default: 0.2)

### The `from_env` Method

```rust
impl Config {
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());

        let export_version = std::env::var("EXPORT_VERSION")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        // ... more settings ...

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            // ...
        }
    }
}
```

The `from_env` function reads each setting from an environment variable. Some settings are required (like `DATABASE_URL`), and if they're missing, the program will crash with a helpful error. Others have defaults (like `CACHE_DIR` defaulting to `"./cache"`).

Notice the pattern:
- `std::env::var("NAME")` — Try to read the variable
- `.expect("message")` — Crash with this message if it's missing
- `.unwrap_or_else(|_| "default".to_string())` — Use a default if it's missing
- `.parse().unwrap_or(default)` — Parse the string as a number, with fallback

### The .env File

Create a file called `.env` in your project root:

```env
DATABASE_URL=postgres://localhost/fichub
REDIS_URL=redis://localhost/
CACHE_DIR=./cache
PORT=3000
```

> ### ⚠️ Watch Out!
> 
> Never commit your `.env` file to version control (like git)! It might contain passwords. Add it to your `.gitignore` file. Here's what your `.gitignore` should include:
> 
> ```
> /target
> .env
> *.epub
> /cache
> /tmp
> ```

## The lib.rs File

There's also a file called `src/lib.rs`:

```rust
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod export;
pub mod frontend;
pub mod limiter;
pub mod recommender;
pub mod routes;
pub mod scrape;
pub mod search;
pub mod server;
pub mod tags;

/// Re-export key functions for integration testing.
pub use routes::export::{build_info_string, build_meta_json, generate_slug};
pub use scrape::ExtractedTag;
```

The `lib.rs` file is almost identical to the `pub mod` lines in `main.rs`. The difference is that `lib.rs` is used when other programs want to use FicHub as a library. The `pub use` lines at the bottom make specific functions available without having to navigate deep into the module tree.

### Why Both main.rs and lib.rs?

Having both files might seem redundant, but it serves a purpose:
- `main.rs` is the **binary crate** — it compiles to an executable program
- `lib.rs` is the **library crate** — it can be used by other Rust programs

This pattern lets you write tests that import FicHub's functions directly, which is much easier than testing through the web server.

## How Modules Map to Folders

Each `pub mod` line corresponds to a folder in `src/`:

```
src/
├── main.rs          # The binary entry point
├── lib.rs           # The library entry point
├── config.rs        # Config module (single file)
├── error.rs         # Error module (single file)
├── db/              # Database module (folder)
│   ├── mod.rs       # Module root
│   ├── models.rs    # Database structs
│   └── queries.rs   # SQL queries
├── scrape/          # Scraper module (folder)
│   ├── mod.rs       # Module root with SiteScraper trait
│   ├── registry.rs  # Scraper registry
│   └── sites/       # Individual site scrapers
│       ├── mod.rs
│       ├── ao3.rs
│       ├── ffnet.rs
│       └── ...
└── ...
```

When you write `pub mod db;`, Rust looks for either `src/db.rs` (a single file) or `src/db/mod.rs` (a folder with a mod.rs inside). FicHub uses both patterns — simple modules are single files, complex ones are folders.

> ### 🔧 Try It Yourself
> 
> 1. Create a `.env` file with `DATABASE_URL=postgres://localhost/test_db` and `REDIS_URL=redis://localhost/`
> 2. Run `cargo build` — does it compile successfully?
> 3. Try setting `PORT=8080` in your `.env` and running the program — does it listen on port 8080?
> 4. What happens if you set `DATABASE_URL` to an invalid value? (Try it!)
> 5. Look at the `Config` struct — how many fields does it have?

---

# Chapter 5: The Axum Web Server — Building the Engine

## What Is a Web Server?

A web server is a program that listens for requests from web browsers (or other programs) and sends back responses. When you type a URL into your browser, your browser sends a request to a web server, and the server sends back a web page.

FicHub is a special kind of web server — it's an **API server**. Instead of sending back web pages, it sends back **JSON** (a format that's easy for programs to understand). This means other programs (like mobile apps or web frontends) can talk to FicHub easily.

## What Is Axum?

Axum is a web framework for Rust. It's like a toolbox that makes it easy to build web servers. Axum handles:

- **Routing** — figuring out which function should handle each request
- **Extractors** — pulling data out of requests (like query parameters or JSON bodies)
- **Responses** — sending data back to the client
- **Middleware** — adding extra behavior (like logging or CORS)
- **State sharing** — letting different handlers share data

Axum is built on top of Tower, which provides a powerful middleware system. This means you can add features like rate limiting, compression, and tracing by stacking middleware layers.

## The AppState: Sharing Data

When your web server handles requests, it often needs to share data between different handlers. In Axum, we use a thing called **state**:

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

Think of `AppState` as a big toolbox that every handler can reach into. It contains:

- **`config`** — All the settings we loaded from environment variables
- **`db`** — A connection pool to PostgreSQL (many connections ready to use)
- **`redis`** — A connection to Redis
- **`http_client`** — A client for making HTTP requests to fanfiction sites
- **`scraper_registry`** — The list of scrapers for different websites
- **`cache_semaphores`** — Something to prevent two requests from generating the same file at the same time
- **`rate_limiter`** — Makes sure we don't overwhelm fanfiction sites
- **`recommender_engine`** — Finds similar stories
- **`collection_worker`** — Background worker for collecting recommendation data

### Why `Arc`?

You'll notice `Arc<ScraperRegistry>` instead of just `ScraperRegistry`. `Arc` stands for **Atomic Reference Counted**. It lets multiple parts of your program share the same data safely. Without `Arc`, Rust would only allow one owner at a time (this is called "ownership").

`Arc` works by keeping a count of how many parts of the program are using the data. When the count reaches zero (everyone is done), the data is automatically cleaned up. This is like a library book — multiple people can check it out, and when everyone returns it, it goes back on the shelf.

### `Box<dyn RateLimiter>`

The `Box<dyn RateLimiter>` is a **trait object**. It means "any type that implements the `RateLimiter` trait." This lets us swap out different rate limiter implementations without changing the rest of the code.

This is called "dynamic dispatch" — at runtime, the program figures out which implementation to call based on what's in the box. The alternative would be "static dispatch" with generics, but that wouldn't work here because we store the rate limiter in a struct field.

## Building the Router

The router is like a map that tells the server: "When someone visits THIS URL, call THIS function." Here's FicHub's router:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // API routes
        .route("/api/", get(routes::api_docs::api_docs_handler))
        .route("/api/v0/epub", get(routes::export::epub_handler))
        .route("/api/v0/meta", get(routes::meta::meta_handler))
        .route("/api/v0/remote", get(remote_handler))

        // Cache download routes
        .route("/cache/{etype}/{url_id}/{fname}",
            get(routes::cache_download::download_with_hash))
        .route("/cache/{etype}/{url_id}",
            get(routes::cache_download::download_or_export))

        // Recommender routes
        .route("/api/v0/recommendations",
            get(crate::recommender::routes::recommendations_handler))
        .route("/api/v0/recommendations/suggest",
            axum::routing::post(crate::recommender::routes::suggest_handler))
        .route("/api/v0/recommendations/vote",
            axum::routing::post(crate::recommender::routes::vote_handler))

        // Tag routes
        .route("/api/v0/tags/submit",
            axum::routing::post(crate::tags::routes::submit_tag))
        .route("/api/v0/tags/vote",
            axum::routing::post(crate::tags::routes::vote_tag))
        .route("/api/v0/tags/flag",
            axum::routing::post(crate::tags::routes::flag_tag))

        // Curator routes
        .route("/api/v0/curator/alias",
            axum::routing::post(crate::tags::curator::create_alias))
        .route("/api/v0/curator/merge",
            axum::routing::post(crate::tags::curator::merge_tags))

        // Search
        .route("/api/v0/search",
            get(crate::search::routes::search_handler))

        // OPDS catalog routes
        .route("/opds", get(crate::routes::opds::feeds::root_catalog))
        .route("/opds/new", get(crate::routes::opds::feeds::recent_feed))
        .route("/opds/popular", get(crate::routes::opds::feeds::popular_feed))
        .route("/opds/tags", get(crate::routes::opds::tags::tag_types))
        .route("/opds/search", get(crate::routes::opds::search::search_feed))

        // Legacy redirect routes
        .route("/legacy/epub_export", get(redirect_to_root))
        .route("/fic/{url_id}", get(redirect_to_root))

        // Static frontend
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
}
```

Each `.route()` line adds a new URL pattern. The first argument is the URL pattern, and the second is the handler function.

### GET vs POST

- **GET** — Used for fetching data. When you visit a URL in your browser, that's a GET request. FicHub uses GET for reading stories, getting metadata, searching, and browsing the OPDS catalog.
- **POST** — Used for sending data. When you fill out a form and click submit, that's a POST request. FicHub uses POST for submitting tags, voting on recommendations, and suggesting new links.

### URL Parameters

Some routes have parameters in curly braces:

```rust
.route("/cache/{etype}/{url_id}/{fname}", get(download_with_hash))
```

This means a URL like `/cache/epub/a1b2c3d4e5f6/myfile.epub` will pass `"epub"` as `etype`, `"a1b2c3d4e5f6"` as `url_id`, and `"myfile.epub"` as `fname` to the handler.

### Middleware

Middleware adds extra behavior to all requests:

```rust
.layer(TraceLayer::new_for_http())    // Logs every request
.layer(CorsLayer::permissive())        // Allows cross-origin requests
```

- **TraceLayer** — Logs information about every request (method, URL, status code, duration)
- **CorsLayer** — Handles Cross-Origin Resource Sharing, which allows web pages from different domains to access the API

### The Fallback Service

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

If no route matches the requested URL, the fallback service serves static files from the frontend directory. This is how FicHub serves its web frontend (a SvelteKit app). The `append_index_html_on_directories(true)` means visiting `/` serves `index.html`, and `fallback` serves `index.html` for any file that doesn't exist (important for single-page apps).

## Starting the Server

```rust
pub async fn run(config: Config) {
    // Connect to PostgreSQL
    let db_pool = db::init_pool(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Build HTTP client
    let http_client = reqwest::Client::builder()
        .user_agent("fichub.net/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    // Initialize scraper registry
    let scraper_registry = Arc::new(ScraperRegistry::new());
    tracing::info!("Registered {} scrapers", scraper_registry.scraper_count());

    // Initialize rate limiter
    let rate_limiter = limiter::redis_bucket::RedisBucketLimiter::new(
        redis_conn.clone(), config.dynamic_rate_limit,
    ).await.expect("Failed to initialize rate limiter");

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
        scraper_registry,
        cache_semaphores: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        rate_limiter: Box::new(rate_limiter),
        recommender_engine: RecommendationEngine::new(db_pool.clone()),
        collection_worker: CollectionWorker::new(/* ... */),
    });

    // Build router
    let app = build_router(state).await;

    // Bind and serve
    let addr = format!("0.0.0.0:{}", config.app_port);
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await
        .expect("Server error");
}
```

This function:
1. Connects to PostgreSQL (with 20 connection pool)
2. Connects to Redis
3. Creates the HTTP client (with 30-second timeout)
4. Sets up the scraper registry
5. Initializes the rate limiter
6. Creates all the shared state
7. Builds the router
8. Starts listening for requests on the configured port

The `0.0.0.0` means "listen on all network interfaces" — so anyone on your network can access it. If you only want localhost access, use `127.0.0.1`.

### The `into_make_service_with_connect_info` Call

```rust
axum::serve(listener, app.into_make_service_with_connect_info::<std::net::SocketAddr>())
```

This tells Axum to include the client's IP address in the request, which is important for rate limiting. Without this, FicHub wouldn't know which IP address made which request.

> ### ⚠️ Watch Out!
> 
> The server uses `.expect()` in several places. If PostgreSQL or Redis isn't running, the server will crash with a clear error message. Always make sure your databases are running before starting FicHub! You can check with:
> 
> ```bash
> # Check PostgreSQL
> pg_isready
> 
> # Check Redis
> redis-cli ping
> ```

> ### 🔧 Try It Yourself
> 
> 1. Look at the router — how many routes does FicHub have?
> 2. What happens if you visit `http://localhost:3000/api/`? (The API docs handler!)
> 3. What's the difference between `.route("/api/v0/epub", get(...))` and `.route("/api/v0/tags/submit", post(...))`?
> 4. Why does the HTTP client have a 30-second timeout?

---

# Chapter 6: Request Handling & JSON APIs — Talking to the World

## What Is an API?

An API (Application Programming Interface) is like a waiter in a restaurant. You tell the waiter what you want (your order), the waiter goes to the kitchen (the server), and brings back your food (the response). You don't need to know how the kitchen works — you just need to know what to order!

In FicHub, the API is a set of URLs that other programs can call to get information. For example:

- `GET /api/v0/epub?q=URL` — Get download links for a story
- `GET /api/v0/meta?q=URL` — Get just the metadata (no downloads)
- `GET /api/v0/search?q=query` — Search for stories
- `GET /opds` — Browse the OPDS catalog

The API returns **JSON** — a text format that's easy for programs to parse. Here's what a typical response looks like:

```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "meta": {
        "id": "a1b2c3d4e5f6",
        "title": "A Great Story",
        "author": "Someone Important",
        "chapters": 10,
        "words": 50000,
        "status": "complete"
    },
    "urls": {
        "epub": "/cache/epub/a1b2c3d4e5f6?h=abc123",
        "html": "/cache/html/a1b2c3d4e5f6?h=def456"
    }
}
```

## The Export Handler

Let's look at FicHub's main handler — the one that generates EPUB files. This is the most complex handler because it does many things:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ExportQuery>,
) -> Result<Json<Value>, AppError> {
    let start = std::time::Instant::now();

    // Step 1: Validate query parameter
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    // Step 2: Check for automated flag (block bots)
    if params.automated.as_deref() == Some("true") {
        return Ok(Json(json!({"err": -10, "msg": "automated requests blocked"})));
    }

    // Step 3: Find the appropriate scraper
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    // Step 4: Lookup metadata (scrape the fanfiction site)
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let info_request_ms = start.elapsed().as_millis() as i32;

    // Step 5: Store metadata in database
    let fic_info_row = FicInfo {
        id: meta.url_id.clone(),
        title: meta.title.clone(),
        author: meta.author.clone(),
        // ... more fields ...
    };
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // Step 6: Check blacklists
    let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
    if !fic_blacklist.is_empty() {
        if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7) {
            return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
        }
    }

    // Step 7: Check cache
    let version = state.config.export_version;
    let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;

    if let Some(export_log) = cached {
        // Cache hit! Return download links immediately
        let epub_hash = &export_log.export_hash;
        return Ok(Json(json!({
            "err": 0,
            "url_id": meta.url_id,
            "urls": { "epub": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash) },
        })));
    }

    // Step 8: Cache miss — generate EPUB
    let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
    let _permit = sem.acquire().await?;

    // Double-check cache (another request might have generated it)
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
    if let Some(export_log) = cached {
        return Ok(build_response(&meta, &export_log));
    }

    // Step 9: Fetch chapters from fanfiction site
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    // Step 10: Generate EPUB file
    let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;

    // Step 11: Move to cache directory
    let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
    cache::disk::move_to_cache(&epub_path, &cache_dest)?;

    // Step 12: Record in database
    queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;

    // Step 13: Log the request
    let export_ms = start.elapsed().as_millis() as i32;
    queries::insert_request_log(&state.db, source_id, "epub", query, info_request_ms,
        Some(&meta.url_id), fic_json.as_deref(), Some(export_ms), /* ... */).await?;

    // Step 14: Build and return response
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "urls": { "epub": format!("/cache/epub/{}?h={}", meta.url_id, epub_hash) },
    })))
}
```

### Extractors

Notice the parameters:

```rust
State(state): State<Arc<AppState>>
Query(params): Query<ExportQuery>
```

These are **extractors** — Axum's way of pulling data out of the request:

- **`State(state)`** — Gets the shared application state (the toolbox we talked about)
- **`Query(params)`** — Gets the URL query parameters (like `?q=https://...`)

### Query Parameters

```rust
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub q: Option<String>,           // The story URL
    pub automated: Option<String>,   // "true" if automated
    pub format: Option<String>,      // Desired format
}
```

This struct defines what query parameters the endpoint accepts. `Option<String>` means the parameter is optional — if it's not in the URL, it'll be `None`.

### The Response

The handler returns `Result<Json<Value>, AppError>`:

- **`Json<Value>`** — A successful response containing JSON data
- **`AppError`** — An error that might have occurred

The `json!()` macro creates JSON easily:

```rust
Json(json!({
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "meta": {
        "title": "A Great Story",
        "author": "Someone Important",
        "words": 50000,
    }
}))
```

### The Metadata Handler

FicHub also has a simpler endpoint that just returns metadata without generating files:

```rust
pub async fn meta_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MetaQuery>,
) -> Result<Json<Value>, AppError> {
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query", "q": ""})));
    }

    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;

    // Lookup metadata only (no chapters fetch — much faster!)
    let meta = scraper.lookup(&state.http_client, query).await
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;

    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "info": format!("{} by {} - {} words, {} chapters",
            meta.title, meta.author, meta.words, meta.chapters),
    })))
}
```

The difference? This handler only calls `lookup()` — it doesn't call `fetch_chapters()`. So it's much faster because it only needs to download one page from the fanfiction site instead of all the chapter pages.

## Helper Functions

FicHub has several helper functions that build parts of the response:

### generate_slug

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

This turns a title like "Harry Potter and the Cursed Child" into a URL-friendly slug like "Harry_Potter_and_the_Cursed_Child-a1b2c3d4e5f6". The slug is useful for generating human-readable download URLs.

### build_meta_json

```rust
pub fn build_meta_json(meta: &FicMetadata) -> Value {
    json!({
        "id": meta.url_id,
        "title": meta.title,
        "author": meta.author,
        "chapters": meta.chapters,
        "words": meta.words,
        "description": meta.desc,
        "status": meta.status,
        "source": meta.source,
        "created": chrono::DateTime::from_timestamp_millis(meta.published)
            .map(|d| d.to_rfc3339()).unwrap_or_default(),
        "updated": chrono::DateTime::from_timestamp_millis(meta.updated)
            .map(|d| d.to_rfc3339()).unwrap_or_default(),
        "extra_meta": meta.extra_meta,
        "raw_extended_meta": meta.raw_extended_meta,
        "author_url": meta.author_url,
        "author_local_id": meta.author_local_id,
        "source_id": meta.source_id,
        "author_id": meta.author_id,
    })
}
```

This takes a `FicMetadata` struct and turns it into a JSON value that can be included in API responses. Notice how it converts Unix timestamps (milliseconds since 1970) to RFC3339 format (like `"2024-01-15T10:30:00Z"`).

### build_info_string

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
        "{title} by {author}\n{words} words in {chapters} chapters\nStatus: {status}\nUpdated: {date} - {relative} ago\n",
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

This creates a human-readable summary like:
```
Harry Potter and the Methods of Rationality by Less Wrong
108462 words in 322 chapters
Status: complete
Updated: 2024-01-15 10:30:00 - 5 days ago
```

> ### 🔧 Try It Yourself
> 
> 1. Look at the `epub_handler` function — what happens when the query is empty?
> 2. What happens when an unsupported URL is provided?
> 3. Why does the handler check the cache before generating the EPUB?
> 4. What's the purpose of the `info_request_ms` timing?
> 5. How would you add a new query parameter to the export endpoint?


# Chapter 7: Error Handling — When Things Go Wrong

## Why Error Handling Matters

Imagine you're baking a cake. What happens if you run out of eggs? What if the oven breaks? What if you accidentally use salt instead of sugar? A good baker has a plan for each of these problems. In programming, we call these plans **error handling**.

Bad error handling leads to:
- Programs that crash without explanation
- Security vulnerabilities (hackers love error messages that leak information)
- Frustrated users who don't know what went wrong
- Data loss when operations fail silently

Good error handling leads to:
- Programs that recover gracefully from problems
- Clear, helpful error messages for users
- Secure systems that don't leak internal details
- Reliable systems that handle unexpected situations

## FicHub's Error Types

FicHub has a custom error enum called `AppError`. This is the central error type that all handlers use:

```rust
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
    /// Scraper error
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

Each variant represents a different kind of error:

- **`BadRequest(i32, String)`** — The client sent something wrong. The `i32` is an error code (like -5 for unsupported URL), and the `String` is a human-readable message.
- **`RateLimited(u64)`** — Too many requests, please wait this many seconds.
- **`NotFound(String)`** — The story doesn't exist on the fanfiction site.
- **`Internal(String)`** — Something went wrong inside the server.
- **`ScrapeError(String)`** — Problem fetching data from a fanfiction site.
- **`ExportError(String)`** — Problem generating the EPUB file.
- **`Database(String)`** — Problem talking to PostgreSQL.
- **`CacheError(String)`** — Problem with Redis.

## Converting Errors to HTTP Responses

The most important part of `AppError` is how it converts to an HTTP response. This is the `IntoResponse` implementation:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS,
                 json!({"err": -429, "msg": "rate limited", "retry_after": retry_after}))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "internal server error"}))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -5, "msg": "export failed"}))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "database error"}))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "cache error"}))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

Notice several important design decisions:

1. **Internal errors log the details but hide them from the client.** The server logs the full error message (so administrators can debug), but the client only sees "internal server error." This prevents leaking sensitive information like database connection strings or file paths.

2. **Each error type gets an appropriate HTTP status code.** `BadRequest` gets 400, `NotFound` gets 404, `RateLimited` gets 429, `ScrapeError` gets 502 (Bad Gateway — because the upstream site had an issue), and internal errors get 500.

3. **Every response includes an `err` code.** This makes it easy for clients to programmatically check what went wrong, rather than parsing error messages.

## Automatic Error Conversion

FicHub uses Rust's `From` trait to automatically convert errors from other libraries into `AppError`:

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

This means you can use the `?` operator with any of these error types, and Rust will automatically convert them:

```rust
let data = tokio::fs::read_to_string("config.json").await?;  // io::Error → AppError
let row = sqlx::query("SELECT...").fetch_one(&pool).await?;    // sqlx::Error → AppError
let value: Value = serde_json::from_str(&json)?;               // serde_json::Error → AppError
```

The `?` operator is Rust's magic error-propagation tool. If the operation succeeds, it gives you the value. If it fails, it automatically converts the error using the `From` implementation and returns it from the current function.

## The AppResult Type Alias

To make error handling even cleaner, FicHub defines a type alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

This means instead of writing `Result<T, AppError>` everywhere, you can just write `AppResult<T>`:

```rust
pub async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;  // sqlx::Error automatically becomes AppError
    Ok(row)
}
```

## Error Handling Patterns in FicHub

### Pattern 1: Validate, Then Process

```rust
// Validate first
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}

// Then process
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

### Pattern 2: Log and Hide Internal Errors

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);  // Log the details
    (StatusCode::INTERNAL_SERVER_ERROR,
     json!({"err": -1, "msg": "internal server error"}))  // Hide from client
}
```

### Pattern 3: Check Multiple Conditions

```rust
// Check fic blacklist
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        // Greylist — show metadata but no download
        return Ok(build_metadata_response(&meta, &[], &state.config.export_version, None, true));
    }
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        // Hard blacklist
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }
}
```

### Pattern 4: The `ok_or_else` Pattern

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

`find_scraper` returns `Option<&Box<dyn SiteScraper>>` — it's `Some(scraper)` if found, `None` if not. `ok_or_else` converts `None` into an `AppError`. The `||` means "create this error only if needed" (lazy evaluation).

### Pattern 5: The `.map_err` Pattern

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

`lookup` returns `Result<FicMetadata, ScrapeError>`. We use `.map_err` to convert the `ScrapeError` into an `AppError::ScrapeError`.

> ### ⚠️ Watch Out!
> 
> Don't use `.unwrap()` in production code! `.unwrap()` will crash your program if the value is `None` or `Err`. Always use `?` or `.unwrap_or_else()` with a helpful error message instead. The only acceptable uses of `.unwrap()` are in tests and quick prototyping.
>
> The FicHub codebase does use `.expect()` in the `run()` function (for database connections), but that's intentional — if the database isn't available, the server can't start, so crashing immediately with a clear message is the right behavior.

> ### 🔧 Try It Yourself
> 
> 1. Create a function that divides two numbers and returns an `AppError` if the divisor is zero
> 2. What HTTP status code would you use for a "division by zero" error?
> 3. Look at the `From<reqwest::Error>` implementation — why does it convert to `ScrapeError` instead of `Internal`?
> 4. What's the difference between `.ok_or()` and `.ok_or_else()`?

---

# Chapter 8: PostgreSQL — The FicHub Database

## What Is a Database?

A database is like a super-powered spreadsheet. It stores information in organized tables, and you can ask it questions like "Show me all stories by this author" or "How many stories have more than 100,000 words?"

FicHub uses **PostgreSQL** (often called "Postgres") — one of the most popular and powerful databases in the world. PostgreSQL is great because:

- It's free and open source
- It handles complex queries really well
- It has excellent support for full-text search
- It's incredibly reliable (people trust it with banking data!)
- It supports transactions (groups of operations that all succeed or all fail)

## Connecting to PostgreSQL

FicHub connects to PostgreSQL using **SQLx** — a Rust library that makes database work easy:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Run migrations from the migrations directory
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

- **`PgPoolOptions::new()`** — Starts configuring a connection pool
- **`.max_connections(20)`** — Allows up to 20 simultaneous database connections
- **`.acquire_timeout(Duration::from_secs(10))`** — Waits up to 10 seconds for a free connection
- **`.connect(database_url)`** — Actually connects to the database
- **Migrations** — Automatically applies any pending database schema changes

### What's a Connection Pool?

Instead of opening a new database connection for every request (which is slow), FicHub maintains a **pool** of 20 ready-to-use connections. When a handler needs to query the database, it borrows a connection from the pool, uses it, and returns it.

Think of it like a taxi stand. Instead of calling a taxi company every time someone needs a ride (slow), you keep 20 taxis waiting at the stand. When someone needs a ride, they just grab a taxi. When they're done, the taxi goes back to the stand.

### Migrations

Migrations are SQL files that define your database structure. They run automatically when the server starts. A typical migration might look like:

```sql
CREATE TABLE IF NOT EXISTS fic_info (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INTEGER NOT NULL DEFAULT 1,
    words BIGINT NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id BIGINT,
    author_id BIGINT,
    content_hash TEXT,
    created TIMESTAMPTZ,
    updated TIMESTAMPTZ,
    fic_created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    fic_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Each migration has a unique name (like `20240115_create_fic_info`) and runs only once. SQLx tracks which migrations have been applied in a special `_sqlx_migrations` table.

## Database Models

FicHub defines Rust structs that match the database tables:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FicInfo {
    pub id: String,              // Unique story identifier (like "a1b2c3d4e5f6")
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
    pub status: String,          // "complete", "ongoing", "hiatus", "cancelled"
    pub source: String,          // Original URL
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
    pub source_id: Option<i64>,
    pub author_id: Option<i64>,
    pub content_hash: Option<String>,
}
```

The `#[derive(FromRow)]` attribute tells SQLx "this struct maps to a database row." SQLx automatically converts database columns to struct fields! The other derives:
- `Debug` — Lets you print the struct for debugging
- `Clone` — Lets you make copies
- `Serialize` / `Deserialize` — Lets you convert to/from JSON

### Other Models

FicHub also has models for:

```rust
pub struct RequestSource {
    pub id: i64,
    pub created: Option<DateTime<Utc>>,
    pub is_automated: Option<bool>,
    pub route: Option<String>,
    pub description: Option<String>,
}

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

pub struct ExportLog {
    pub url_id: String,
    pub version: i32,
    pub etype: String,
    pub input_hash: String,
    pub export_hash: String,
    pub created: Option<DateTime<Utc>>,
}

pub struct FicBlacklist {
    pub url_id: String,
    pub created: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub reason: i32,
}
```

## Database Queries

Here's how FicHub stores a story in the database:

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

This is an **upsert** — it inserts a new record, or updates an existing one if the `id` already exists. The `ON CONFLICT (id) DO UPDATE` clause handles the update case. The `EXCLUDED` keyword refers to the values that were attempted to be inserted.

The `$1`, `$2`, etc. are **parameterized queries** — they prevent SQL injection attacks. Never concatenate user input directly into SQL strings!

### Reading from the Database

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

This function returns `Option<FicInfo>` — it's `Some(FicInfo)` if the story exists, or `None` if it doesn't. The `fetch_optional` method is perfect for lookups where the result might not exist.

### Cache Checking

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

This checks if we've already generated an export for this story — a key part of the caching system! If the export exists and the hash matches, we can serve the cached file instead of regenerating it.

### Blacklist Checking

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
```

This returns all blacklist entries for a story. Different reasons mean different things:
- Reason 5, 7, 8 — Hard blacklist (block completely)
- Reason 6 — Greylist (show metadata but no downloads)

### Similar Fic Search

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

The `ILIKE` operator is case-insensitive pattern matching. `%query%` means "contains the query anywhere in the text." This is used for "did you mean?" suggestions when a URL isn't recognized.

### Tag Queries

FicHub also has many tag-related queries:

```rust
pub async fn upsert_fic_tag(
    pool: &PgPool, url_id: &str, tag_id: i32, ip: &std::net::IpAddr,
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

This associates a tag with a story. The `3::inet` cast tells PostgreSQL to treat the string as an IP address type.

## Transaction Safety

PostgreSQL supports **transactions** — groups of operations that all succeed or all fail. For example, when merging tags:

```rust
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    // Reassign fic_tags from source to target
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

> ### ⚠️ Watch Out!
> 
> Never modify a migration file after it's been applied to a database! If you need to change the database structure, create a new migration file instead. Old migrations are like historical records — they show how the database evolved over time.
>
> Also, always use parameterized queries ($1, $2, etc.) instead of string formatting. SQL injection attacks are one of the most common security vulnerabilities, and parameterized queries prevent them completely.

> ### 🔧 Try It Yourself
> 
> 1. What does `fetch_optional` return vs `fetch_one`?
> 2. Why does FicHub use `$1`, `$2`, etc. in SQL queries instead of inserting values directly?
> 3. What would happen if two requests tried to `upsert_fic_info` for the same story at the same time?
> 4. How would you add a new query to find stories by word count range?

---

# Chapter 9: Web Scraping 101 — Reading the Web

## What Is Web Scraping?

Web scraping is like teaching a robot to read websites. When you visit a fanfiction story in your browser, you see a beautiful page with the title, author, and story text. But a computer sees raw HTML — a bunch of tags and text that looks like gibberish to humans.

A **scraper** is a program that reads the HTML and extracts the information we care about. It's like having a really fast reader who can pull out the important parts from any web page.

## Why Do We Scrape?

FicHub scrapes fanfiction sites because they don't provide APIs. An API is a structured way for programs to request data, but most fanfiction sites only have web pages. So we have to parse the HTML ourselves to extract:
- Story title and author
- Word count and chapter count
- Story description/summary
- Publication and update dates
- Story content (the actual text)
- Tags and categories

## The Scraper Trait

FicHub uses a **trait** (an interface) that all scrapers must implement:

```rust
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

Each scraper must answer four questions:
1. **`can_handle`** — "Can I handle this URL?" (e.g., does it belong to AO3?)
2. **`lookup`** — "Get me the metadata for this story"
3. **`fetch_chapters`** — "Download all the chapter content"
4. **`extract_tags`** — "What tags does this story have?" (optional — has a default implementation)

The `#[async_trait]` attribute is needed because async functions in traits require special handling in Rust. The `Send + Sync` bounds ensure the scraper can be shared across threads safely.

## FicMetadata: The Story Blueprint

Every scraper produces a `FicMetadata` struct:

```rust
pub struct FicMetadata {
    pub url_id: String,           // Unique ID (like "a1b2c3d4e5f6")
    pub title: String,
    pub author: String,
    pub chapters: i32,
    pub words: i64,
    pub desc: String,             // Description/summary
    pub published: i64,           // Unix timestamp (milliseconds)
    pub updated: i64,
    pub status: String,           // "complete", "ongoing", "hiatus"
    pub source: String,           // Original URL
    pub source_id: i64,           // Which website (1=AO3, 2=FF.net, etc.)
    pub author_id: i64,
    pub author_url: String,
    pub author_local_id: String,  // Site-specific author/story ID
    pub content_hash: Option<String>,
    pub extra_meta: Option<String>,
    pub raw_extended_meta: Option<String>,
}
```

This struct is the same regardless of which site the story comes from. Each scraper translates the site's HTML into this common format.

## The Scraper Registry

FicHub has a **registry** that knows about all available scrapers:

```rust
pub struct ScraperRegistry {
    scrapers: Vec<Box<dyn SiteScraper>>,
}

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

    pub fn find_scraper(&self, url: &str) -> Option<&Box<dyn SiteScraper>> {
        self.scrapers.iter().find(|s| s.can_handle(url))
    }

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
}
```

When a URL comes in, the registry asks each scraper "Can you handle this?" The first one that says "yes" gets to do the work.

### Why a Registry?

The registry pattern makes FicHub extensible. To add support for a new site, you just:
1. Create a new scraper struct
2. Implement the `SiteScraper` trait
3. Add it to the registry

No other code needs to change!

## Tags: Extracted Information

Scrapers can also extract structured tags from stories:

```rust
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

Tag types include:
- **Fandom** (1) — Harry Potter, Marvel, etc.
- **Character** (2) — Harry Potter, Hermione Granger, etc.
- **Relationship** (3) — Harry/Hermione, Steve/Bucky, etc.
- **Freeform** (4) — Angst, Fluff, Time Travel, etc.
- **Warning** (5) — Character Death, Major Character Death, etc.
- **Category** (6) — Gen, F/M, M/M, etc.

## URL IDs: Unique Story Identifiers

FicHub creates unique IDs for stories using a hash:

```rust
pub fn generate_url_id(source_id: i64, story_id: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(source_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(story_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..6])  // First 12 hex characters
}
```

For example, AO3 story `12345` would get `source_id = 1`, producing an ID like `"a1b2c3d4e5f6"`. The same story always gets the same ID — it's **deterministic**.

Why use a hash instead of just using the story ID? Because the same story ID might exist on different sites (AO3 story `12345` and FF.net story `12345` are different stories). By including the `source_id`, we ensure unique IDs across all sites.

## Chapter Content

Each chapter is a simple struct:

```rust
pub struct Chapter {
    pub chapter_id: i32,
    pub title: String,
    pub content: String,  // HTML content
}
```

The `content` field contains the chapter's HTML. This preserves formatting like paragraphs, emphasis, and block quotes from the original site.

## Error Types

Scrapers can fail in several ways:

```rust
pub enum ScrapeError {
    NotFound,              // The story doesn't exist
    Blocked,               // The site blocked our request
    Network(String),       // Network error (timeout, DNS, etc.)
    ParseError(String),    // Couldn't parse the HTML
}
```

These get converted to `AppError` automatically:

```rust
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ScrapeError(err.to_string())
    }
}
```

> ### 🔧 Try It Yourself
> 
> 1. Why does `generate_url_id` include the `source_id`? What would happen without it?
> 2. Why are only the first 12 hex characters used (6 bytes)?
> 3. Look at the `extract_tags` method — why does it have a default implementation that returns an empty vector?
> 4. What would happen if two scrapers both said `can_handle` for the same URL?

---

# Chapter 10: AO3 Scraper — Scraping Archive of Our Own

## What Is AO3?

Archive of Our Own (AO3) is one of the biggest fanfiction websites in the world, run by the Organization for Transformative Works. It has millions of stories across thousands of fandoms. FicHub's AO3 scraper can extract story metadata and content from AO3 pages.

## Understanding AO3 URLs

AO3 stories have URLs like:
- `https://archiveofourown.org/works/123456`
- `https://archiveofourown.org/works/123456/chapters/789012`

The scraper needs to extract the **work ID** (123456) from the URL:

```rust
fn extract_work_id(url: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"/works/(\d+)").ok()?;
    re.captures(url)?.get(1).map(|m| m.as_str().to_string())
}
```

The regex `/works/(\d+)` matches the pattern `/works/` followed by one or more digits, and captures the digits.

## Can This Scraper Handle the URL?

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("archiveofourown.org")
}
```

Simple! If the URL contains "archiveofourown.org", this scraper handles it. Note that this is a simple check — in production, you might want to be more specific to avoid matching URLs that just mention AO3 in a comment.

## Looking Up Metadata

The `lookup` method fetches the story page and extracts information:

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

    // Extract title
    let title = document
        .select(&Selector::parse("h2.title.heading").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Title".to_string());

    // Extract author
    let author = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .unwrap_or_else(|| "Unknown Author".to_string());

    // Extract author URL
    let author_url = document
        .select(&Selector::parse("a[rel='author']").unwrap())
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|h| format!("{BASE_URL}{h}"))
        .unwrap_or_default();

    // Extract description
    let description = document
        .select(&Selector::parse("blockquote.userstuff").unwrap())
        .next()
        .map(|el| el.inner_html())
        .unwrap_or_default();

    // Extract chapters (handle "10/15" format)
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

    // Extract word count
    let words = document
        .select(&Selector::parse("dd.words").unwrap())
        .next()
        .and_then(|el| {
            el.text().collect::<String>().replace(',', "").parse().ok()
        })
        .unwrap_or(0);

    // Extract status
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

### CSS Selectors

The scraper uses **CSS selectors** to find specific elements in the HTML:

| Selector | What It Finds |
|----------|---------------|
| `h2.title.heading` | The story title |
| `a[rel='author']` | The author link |
| `blockquote.userstuff` | The description |
| `dd.chapters` | Chapter count |
| `dd.words` | Word count |
| `dd.status` | Completion status |

### Why the `?view_full_work=true` Parameter?

AO3 normally shows one chapter at a time. Adding `?view_full_work=true` to the URL tells AO3 to show all chapters on one page. This makes scraping much easier because we only need to make one HTTP request instead of one per chapter.

### User-Agent Header

```rust
.header("User-Agent", "fichub.net/0.1.0")
```

Always set a descriptive User-Agent! Some websites block requests without one. FicHub identifies itself as "fichub.net/0.1.0" so the website administrators can contact us if needed.

## Fetching Chapters

The `fetch_chapters` method downloads the full story:

```rust
async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
    let url = format!("{BASE_URL}/works/{}?view_full_work=true", meta.author_local_id);
    let response = client.get(&url)
        .header("User-Agent", "fichub.net/0.1.0")
        .send().await
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

    // Fallback: if no chapter divs found, try reading full work content
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

The fallback at the end handles single-chapter stories — if there are no `div.chapter` elements, it looks for the main content div.

> ### ⚠️ Watch Out!
> 
> AO3 might change their HTML structure at any time! When that happens, the CSS selectors break and the scraper stops working. That's why FicHub has scrapers for multiple sites — if one breaks, the others still work.
>
> Also, AO3 has rate limits. If you make too many requests too quickly, they'll temporarily block your IP. That's why FicHub has rate limiting built in!

> ### 🔧 Try It Yourself
> 
> 1. Visit an AO3 story in your browser and view the page source (Ctrl+U)
> 2. Can you find the `h2.title.heading` element?
> 3. What CSS selector would you use to find the story's rating?
> 4. Why does the scraper use `inner_html()` instead of `text()` for the content?

---

# Chapter 11: FanFiction.net & Other Sites — Supporting More Sources

## FanFiction.net

FanFiction.net (FF.net) is one of the oldest fanfiction sites, founded in 1998. Its HTML structure is different from AO3, so it needs its own scraper.

### FF.net URLs

FF.net stories have URLs like:
- `https://www.fanfiction.net/s/12345/1/`
- `https://www.fanfiction.net/s/12345/15/` (chapter 15)

The story ID is the number after `/s/`.

### The FF.net Scraper

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
            .ok_or_else(|| ScrapeError::ParseError("could not extract story ID".into()))?;

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

        // FF.net uses different selectors than AO3
        let title = document
            .select(&Selector::parse("#profile_top b.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());

        let author = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Author".to_string());

        let author_url = document
            .select(&Selector::parse("#profile_top a.xcontrast_txt").unwrap())
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|h| format!("https://www.fanfiction.net{h}"))
            .unwrap_or_default();

        let description = document
            .select(&Selector::parse("#profile_top div.xcontrast_txt").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let words = document
            .select(&Selector::parse("#profile_top span[data-xutitle='word count']").unwrap())
            .next()
            .and_then(|el| {
                el.text().collect::<String>().replace(',', "").parse().ok()
            })
            .unwrap_or(0);

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

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        let mut chapters = Vec::new();
        let story_id = &meta.author_local_id;

        // FF.net shows one chapter at a time, so we loop through all chapters
        for i in 1..=meta.chapters {
            let url = format!("https://www.fanfiction.net/s/{story_id}/{i}/");
            let response = client.get(&url)
                .header("User-Agent", "fichub.net/0.1.0")
                .send().await
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
}
```

### Key Differences from AO3

1. **Selectors are different** — FF.net uses `#profile_top b.xcontrast_txt` for titles instead of `h2.title.heading`
2. **One chapter at a time** — FF.net doesn't have a `?view_full_work=true` option, so we must loop through all chapters
3. **Different URL structure** — FF.net uses `/s/ID/CHAPTER/` while AO3 uses `/works/ID`
4. **Source ID** — FF.net gets `source_id = 2` while AO3 gets `source_id = 1`

### FictionPress

FictionPress is actually the same company as FF.net, and the sites have identical HTML structure. FicHub cleverly reuses the FF.net scraper:

```rust
pub use FfNetScraper as FictionPressScraper;
```

That's it! One line of code. The `FictionPressScraper` is just the `FfNetScraper` under a different name. This is possible because the `can_handle` method already checks for both domains:

```rust
fn can_handle(&self, url: &str) -> bool {
    url.contains("fanfiction.net") || url.contains("fictionpress.com")
}
```

## Other Supported Sites

FicHub also supports:

- **XenForo** — Forum-based fanfiction sites (like Spacebattles, Sufficient Velocity)
- **AdultFanFiction** — Another fanfiction site
- **HPFanFic** — Harry Potter specific fanfiction site

Each has its own scraper file, but they all follow the same pattern: implement the `SiteScraper` trait with `can_handle`, `lookup`, and `fetch_chapters`.

### XenForo Scraper

XenForo is interesting because many fanfiction sites use the XenForo forum software. The scraper handles:
- Thread-based stories (each post is a chapter)
- Pagination (stories spanning multiple pages)
- Different XenForo themes

## Adding a New Scraper

Want to add support for a new fanfiction site? Here's the recipe:

1. Create a new file in `src/scrape/sites/` (like `my_new_site.rs`)
2. Define a struct (like `pub struct MyNewSiteScraper;`)
3. Implement `SiteScraper` for it
4. Add it to the `ScraperRegistry::new()` function

```rust
// src/scrape/sites/my_new_site.rs
pub struct MyNewSiteScraper;

#[async_trait]
impl SiteScraper for MyNewSiteScraper {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("my-new-site.com")
    }

    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError> {
        // 1. Extract the story ID from the URL
        // 2. Fetch the story page
        // 3. Parse the HTML
        // 4. Extract metadata
        // 5. Return FicMetadata
        todo!()
    }

    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError> {
        // 1. Fetch each chapter page
        // 2. Extract chapter content
        // 3. Return Vec<Chapter>
        todo!()
    }
}
```

Then register it:

```rust
// In src/scrape/registry.rs
impl ScraperRegistry {
    pub fn new() -> Self {
        let mut scrapers: Vec<Box<dyn SiteScraper>> = Vec::new();
        scrapers.push(Box::new(sites::ao3::Ao3Scraper));
        scrapers.push(Box::new(sites::ffnet::FfNetScraper));
        // ... existing scrapers ...
        scrapers.push(Box::new(sites::my_new_site::MyNewSiteScraper));
        ScraperRegistry { scrapers }
    }
}
```

> ### 🔧 Try It Yourself
> 
> 1. Why does FF.net need to loop through chapters individually while AO3 can get them all at once?
> 2. What's the benefit of reusing `FfNetScraper` for FictionPress?
> 3. If you were adding a scraper for a new site, what's the first thing you'd need to figure out?
> 4. Why does the FF.net scraper check for both `fanfiction.net` and `fictionpress.com`?


# Chapter 12: Building EPUB Files — Creating Digital Books

## What Is an EPUB?

EPUB stands for **Electronic Publication**. It's like a ZIP file that contains HTML pages, a table of contents, and metadata. Almost every e-reader — Kindle, Kobo, Nook, iPad — can read EPUB files.

Think of an EPUB as a tiny website packaged into a book. It has:
- An **index page** with the title, author, and description
- **Chapter pages** with the story content
- A **stylesheet** for nice formatting
- A **table of contents** for easy navigation
- A **container file** that tells e-readers where to find everything

EPUB is actually an open standard maintained by the W3C (the people who maintain HTML and CSS). This means anyone can create EPUB files without paying royalties.

## FicHub's EPUB Generator

FicHub uses the `epub-builder` crate to create EPUB files. Let's walk through the entire process:

### Step 1: Create a Work Directory

Each EPUB is generated in its own temporary directory (named after a UUID). This prevents conflicts when multiple requests are processed at the same time:

```rust
let uuid = Uuid::new_v4();
let work_dir = tmp_dir.join(uuid.to_string());
fs::create_dir_all(&work_dir)?;
```

A UUID (Universally Unique Identifier) is a 128-bit number that's practically guaranteed to be unique. For example: `550e8400-e29b-41d4-a716-446655440000`.

### Step 2: Set Up the Builder

```rust
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;
```

The builder manages all the EPUB's internal structure. We tell it the title, author, language, and description. These metadata fields appear in e-reader libraries.

### Step 3: Add a Stylesheet

```rust
let css = concat!(
    "body{font-family:serif;line-height:1.5;}",
    "h2{text-align:center;}",
    "p{margin:0.5em 0;}"
);
builder.stylesheet(css.as_bytes())?;
```

This CSS makes the EPUB look nice:
- **Serif font** — Traditional book-like appearance
- **Line height 1.5** — Comfortable reading spacing
- **Centered chapter titles** — Like a real book
- **Paragraph margins** — Clear separation between paragraphs

### Step 4: Add the Introduction Page

The introduction page shows the story's metadata — title, author, word count, and description. This is like the title page of a physical book:

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

### Step 5: Add Chapters

For each chapter, we create an XHTML page:

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

Notice that we use `escape_html` on the title but not on the content. The content is already HTML from the scraper, so we don't want to escape it again.

### Step 6: Generate the File

```rust
let epub_path = work_dir.join("output.epub");
let file = fs::File::create(&epub_path)?;
builder.generate(file)?;
```

This creates the actual EPUB file on disk.

### Step 7: Compute the Hash

```rust
let epub_data = fs::read(&epub_path)?;
let md5_hex = Md5::digest(&epub_data)
    .iter()
    .map(|b| format!("{:02x}", b))
    .collect::<String>();

Ok((epub_path, md5_hex))
```

The MD5 hash is used for caching. If the same story is requested again and the content hash matches, we can serve the cached file instead of regenerating it.

## Helper Functions

### escape_html

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace(''', "&#39;")
}
```

This prevents special HTML characters from breaking the EPUB structure. For example, if a title contains `<script>`, it becomes `&lt;script&gt;` — safe to display as text.

### format_timestamp

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

This converts Unix milliseconds to a readable date like "2024-01-15".

## The Export Module

FicHub also has an `ExportError` type for export-related problems:

```rust
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
            ExportError::IoError(e) => write!(f, "IO error: {e}"),
            ExportError::TemplateError(e) => write!(f, "template error: {e}"),
            ExportError::EpubError(e) => write!(f, "EPUB error: {e}"),
            ExportError::CalibreError(e) => write!(f, "Calibre error: {e}"),
            ExportError::ZipError(e) => write!(f, "zip error: {e}"),
        }
    }
}
```

And it supports multiple export types:

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

Version numbers help with cache invalidation — when the EPUB template changes, we bump the version and all cached files become stale.

> ### ⚠️ Watch Out!
> 
> The `epub-builder` crate expects XHTML, not regular HTML. That's why every chapter page starts with `<?xml version="1.0"?>` and uses proper closing tags like `</body>` instead of just `</body>`. XHTML is stricter about closing tags and attribute quoting.

> ### 🔧 Try It Yourself
> 
> 1. What happens if a chapter's content contains a `<script>` tag? How does `escape_html` handle it?
> 2. Why does the EPUB use XHTML instead of regular HTML?
> 3. What's the purpose of the MD5 hash?
> 4. How would you add a cover image to the EPUB?

---

# Chapter 13: HTML Bundles & ZIP Archives — Alternative Formats

## HTML Bundles

Not everyone has an e-reader! Some people prefer reading in their web browser. FicHub can create **HTML bundles** — single HTML files that contain the entire story, packaged in a ZIP archive.

## Why HTML Bundles?

HTML bundles have several advantages:
1. **No special software needed** — Anyone with a web browser can read them
2. **Works offline** — Download once, read anywhere
3. **Preserves formatting** — HTML supports bold, italic, images, and more
4. **Small file size** — ZIP compression reduces the file size significantly
5. **Universal compatibility** — Every device can open HTML files

## Creating the HTML Bundle

The HTML bundle generator creates a beautiful, responsive reading experience:

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
    // ... (assemble HTML, write to file, create ZIP, compute hash)
}
```

### Chapter Navigation

Each chapter gets an anchor link (`#ch1`, `#ch2`, etc.) so readers can jump between chapters using the navigation panel. The navigation uses a two-column layout for compactness:

```html
<div class="nav">
    <h3>Chapter Navigation</h3>
    <ul>
        <li><a href="#ch1">Chapter 1</a></li>
        <li><a href="#ch2">Chapter 2</a></li>
        <li><a href="#ch3">Chapter 3</a></li>
    </ul>
</div>
```

### Responsive CSS

The HTML bundle includes responsive CSS that works on any screen size:

```css
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
.nav ul { list-style: none; columns: 2; }
.content p { margin: 0.5em 0; text-indent: 1.5em; }
.content p:first-of-type { text-indent: 0; }
```

Key design decisions:
- **`max-width: 800px`** — Prevents lines from being too long (hard to read)
- **`margin: 0 auto`** — Centers the content
- **`line-height: 1.7`** — Generous spacing for comfortable reading
- **`columns: 2` for navigation** — Compact chapter list

### The Complete HTML Template

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>{title} — {author}</title>
    <style>/* ... CSS ... */</style>
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
    <div class="nav">
        <h3>Chapter Navigation</h3>
        <ul>{nav}</ul>
    </div>
    <hr/>
    <div class="content">{content}</div>
    <hr/>
    <div class="footer">
        <p>Generated by fICHub — {source}</p>
    </div>
</body>
</html>
```

## The ZIP Archive

The HTML is wrapped in a ZIP file:

```rust
let zip_path = work_dir.join("bundle.zip");
let zip_file = fs::File::create(&zip_path)?;
let mut zip = ZipWriter::new(zip_file);

let options = SimpleFileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .unix_permissions(0o644);

zip.start_file("index.html", options)?;
zip.write_all(html.as_bytes())?;
zip.finish()?;
```

### Why ZIP?

1. **Compression** — ZIP reduces file size by 50% or more (HTML is very compressible)
2. **Standard format** — Every operating system can open ZIP files
3. **Single file** — The entire story is in one file, easy to email or share
4. **Renamable** — Users can rename `.zip` to `.html` if they prefer

### Deflated Compression

The `CompressionMethod::Deflated` is the standard ZIP compression algorithm. It's fast and provides good compression for text content. There are other options (like BZIP2 or LZMA) but Deflated is the best balance of speed and compression.

## Serving Download Files

When someone requests a cached file, FicHub serves it with the correct MIME type:

```rust
let mime = match etype {
    EType::Epub => "application/epub+zip",
    EType::Html => "application/zip",
    EType::Mobi => "application/x-mobipocket-ebook",
    EType::Pdf => "application/pdf",
};

let filename = format!("{}{}", url_id, etype.suffix());
let headers = [
    ("Content-Type", mime),
    ("Content-Disposition", &format!("attachment; filename="{}"", filename)),
];
```

The `Content-Disposition: attachment` header tells the browser to download the file instead of trying to display it.

> ### 🔧 Try It Yourself
> 
> 1. Why does the HTML bundle use `columns: 2` for the chapter navigation?
> 2. What's the benefit of using `Deflated` compression in the ZIP?
> 3. How would you add a "Back to Top" link after each chapter?
> 4. What's the difference between the EPUB and HTML bundle approaches?

---

# Chapter 14: The Caching System — Remembering Everything

## Why Cache?

Imagine if every time someone asked for the same story, FicHub had to go back to AO3, download all the chapters, and generate the EPUB again. That would be:

1. **Slow** — Downloading from AO3 takes time (sometimes 10+ seconds)
2. **Rude** — We'd be hitting AO3's servers constantly, potentially getting blocked
3. **Wasteful** — We'd be generating the same file over and over
4. **Unreliable** — If AO3 is down, no one can download anything

Caching solves all these problems. When FicHub generates an EPUB, it saves it to disk. The next time someone asks for the same story, FicHub just serves the saved file — no downloading, no generating, instant response!

## Cache Architecture

FicHub's cache system has three layers:

### Layer 1: Database Cache Log

The `export_log` table in PostgreSQL tracks which files have been generated:

```sql
CREATE TABLE export_log (
    url_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);
```

When a request comes in, FicHub first checks this table:
- If a matching entry exists, the file is cached
- If not, the file needs to be generated

### Layer 2: Disk Storage

Cached files are stored on disk in a structured directory:

```
cache/
├── epub/
│   ├── a1b/
│   │   └── a1b2c3d4e5f6/
│   │       └── a1b2c3d4e5f6/
│   │           └── a1b2c3d4e5f6.epub
│   └── x9y/
│       └── x9y8z7w6v5u4/
│           └── x9y8z7w6v5u4/
│               └── x9y8z7w6v5u4.epub
├── html/
│   └── ...
├── mobi/
│   └── ...
└── pdf/
    └── ...
```

The 3-character prefix directories prevent having too many files in a single folder (which can slow things down on some filesystems).

### Layer 3: Concurrent Export Prevention

The `CacheSemaphores` system prevents two requests from generating the same EPUB at the same time:

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

A **semaphore** with 1 permit ensures only one task can generate a file at a time for each (url_id, etype) pair.

## The Double-Check Pattern

FicHub uses a clever trick called the **double-check pattern**:

```rust
// First check (fast — just a database query)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Cache hit! Return immediately
    return Ok(build_response(&meta, &export_log));
}

// Acquire semaphore (might wait if another request is generating the same file)
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check (another request might have generated it while we were waiting)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Another request generated it! Use the cached version
    return Ok(build_response(&meta, &export_log));
}

// Actually generate the EPUB
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;

// Move to cache directory
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;

// Record in database
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

### Why Two Checks?

Without the second check, this could happen:
1. Request A checks cache — miss
2. Request B checks cache — miss
3. Request A acquires semaphore, generates EPUB
4. Request B acquires semaphore, generates EPUB again (wasteful!)

With the double-check:
1. Request A checks cache — miss
2. Request B checks cache — miss
3. Request A acquires semaphore, checks cache again — miss, generates EPUB
4. Request B acquires semaphore, checks cache again — hit! Uses cached version

## Cache Path Computation

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

    path = path.join(url_id);
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

For a url_id like `"a1b2c3d4e5f6"`:
- First chunk: `"a1b"` (characters 0-2)
- Second chunk: `"c3d"` (characters 3-5)
- Third chunk: `"4e5"` (characters 6-8)
- Full directory: `"a1b2c3d4e5f6"`
- File: `"a1b2c3d4e5f6.epub"`

## Hash Validation

When serving cached files, FicHub validates the hash:

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

The hash ensures the file hasn't been corrupted or tampered with. If the hash doesn't match, FicHub returns an error instead of serving a potentially corrupt file.

## Cache Invalidation

Cache invalidation is one of the hardest problems in computer science! FicHub handles it through version numbers:

1. **`export_version`** — Bumped when the EPUB template changes
2. **`content_hash`** — Changes when the story content changes on the source site
3. **`fic_version_bump`** — Manual bump for specific stories

When any of these change, the cache key changes, and FicHub regenerates the file.

> ### ⚠️ Watch Out!
> 
> Cache invalidation is one of the hardest problems in computer science! If a story is updated on AO3, FicHub needs to know to regenerate the EPUB. That's what the `content_hash` and `export_version` are for — they detect when the source has changed.
>
> Also, cached files accumulate over time. You should periodically clean up old cached files to save disk space.

> ### 🔧 Try It Yourself
> 
> 1. What would happen without the double-check pattern? (Think about two concurrent requests)
> 2. Why are cached files named with their MD5 hash instead of just the story title?
> 3. What's the advantage of the 3-character prefix directories?
> 4. How would you implement a "cache size limit" that deletes old files when the cache gets too big?

---

# Chapter 15: Rate Limiting with Redis — Being a Good Citizen

## What Is Rate Limiting?

Rate limiting is like having a polite queue at a store. Instead of everyone rushing in at once, rate limiting says "only 10 people per minute can come in." This prevents:

1. **Overwhelming fanfiction sites** — If FicHub made 1000 requests per second to AO3, they'd block us
2. **Resource exhaustion** — Too many concurrent requests could crash our server
3. **Unfair usage** — One user shouldn't be able to hog all the resources
4. **DDoS protection** — Prevents malicious actors from flooding the server

## The Token Bucket Algorithm

FicHub uses a clever rate limiting algorithm called the **token bucket**. Imagine a bucket that:

- Has a maximum capacity (like 30 tokens)
- Refills at a steady rate (like 0.116 tokens per second)
- Each request costs 1 token
- If the bucket is empty, you have to wait until tokens refill

Here's how it works in Redis (a fast in-memory database):

```lua
local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

-- Get current bucket state
local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

-- Get current time
local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

-- Initialize if new bucket
if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

-- Calculate new tokens based on elapsed time
local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    -- Request allowed — consume tokens
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1  -- -1 means "allowed"
else
    -- Request denied — calculate wait time
    local wait = (requested - new_tokens) / flow
    return wait  -- Positive number = seconds to wait
end
```

This Lua script runs inside Redis and is **atomic** — it can't be interrupted by other requests. This is crucial for rate limiting — if two requests check the bucket at the same time, the atomicity ensures they don't both think they have enough tokens.

## The Rate Limiter Trait

```rust
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request is allowed for the given IP
    async fn check_ip(&self, ip: IpAddr) -> RateLimitResult;

    /// Report a failure (penalize)
    async fn report_failure(&self, ip: IpAddr);

    /// Check if IP is in a datacenter blocklist
    fn is_datacenter_ip(&self, ip: IpAddr) -> bool;
}

pub enum RateLimitResult {
    Allowed,
    Wait(u64),    // Wait this many seconds
    Blocked,      // Datacenter IP, etc.
}
```

## The Redis Bucket Implementation

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,        // SHA of loaded Lua script
    dynamic_rate_limit: bool,
    static_delay_base: f64, // seconds
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,

    // Configuration
    global_capacity: f64,   // 150 tokens
    global_flow: f64,       // 30 tokens/second
    ip_capacity: f64,       // 30 tokens
    ip_flow: f64,           // 0.116 tokens/second (~1/8.6 sec)
}
```

### Two Levels of Rate Limiting

FicHub rate-limits at two levels:

1. **Global** — Total requests across all users (150 tokens, refills at 30/sec)
2. **Per-IP** — Requests from each individual IP address (30 tokens, refills at 0.116/sec)

This means:
- A single user can make about 1 request every 8.6 seconds
- The total system can handle about 30 requests per second
- If either limit is exceeded, the user has to wait

### Why Two Levels?

- **Global limit** prevents one user from consuming all server resources
- **Per-IP limit** prevents individual users from overwhelming fanfiction sites
- Together, they ensure fair usage and site compatibility

### The Check Flow

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    if !self.dynamic_rate_limit {
        // Simple static delay mode
        let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
        tokio::time::sleep(Duration::from_secs_f64(delay)).await;
        return RateLimitResult::Allowed;
    }

    // Check datacenter IPs (block bots)
    if self.is_datacenter_ip(ip) {
        return RateLimitResult::Blocked;
    }

    // Check global bucket first (system-wide limit)
    let global_wait = self.check_bucket("rate:global", self.global_capacity, self.global_flow)
        .await.unwrap_or(-1.0);
    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Check per-IP bucket (individual user limit)
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, self.ip_capacity, self.ip_flow)
        .await.unwrap_or(-1.0);
    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

## Penalizing Failures

When a request fails (like a network error), FicHub penalizes the IP by consuming extra tokens:

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
        .arg(1.5)  // penalize with 1.5 tokens (more than a normal request)
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

The penalty uses 1.5 tokens instead of 1, making the IP wait longer before trying again. This discourages rapid retries that could overwhelm the system.

## Datacenter IP Blocking

FicHub can block requests from datacenter IPs (like AWS, Google Cloud, Azure) to prevent automated scraping:

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

Datacenter IPs are typically used by bots and scrapers, not real users. Blocking them reduces abuse while having minimal impact on legitimate users.

## Static vs Dynamic Rate Limiting

FicHub supports two modes:

1. **Dynamic** (default) — Uses the token bucket algorithm with Redis
2. **Static** — Adds a random delay (0.1 to 0.2 seconds) to every request

```rust
if !self.dynamic_rate_limit {
    let delay = self.static_delay_base + rand::random::<f64>() * self.static_delay_base;
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
    return RateLimitResult::Allowed;
}
```

Static mode is simpler but less efficient — it slows down all requests equally, even when the server isn't under load.

> ### ⚠️ Watch Out!
> 
> Rate limiting uses Redis, so if Redis goes down, rate limiting stops working! FicHub handles this gracefully by falling back to a simple static delay. This is called "degrading gracefully" — the system still works, just not as optimally.
>
> Also, rate limits should be configurable. What works for a small personal server might not work for a large public instance.

> ### 🔧 Try It Yourself
> 
> 1. Why does the global bucket have more capacity than the per-IP bucket?
> 2. What would happen if we removed the per-IP rate limit?
> 3. How does the token bucket handle bursts of traffic vs. sustained traffic?
> 4. Why is the Lua script important for correctness?


# Chapter 16: Community Tags — Letting Users Help

## What Are Tags?

Tags are labels that describe stories. For example, a Harry Potter story might have tags like:
- **Fandom:** Harry Potter
- **Characters:** Harry Potter, Hermione Granger
- **Relationships:** Harry/Hermione
- **Freeform:** Angst, Hurt/Comfort, Time Travel
- **Warnings:** Character Death
- **Category:** F/M

FicHub lets the **community** add and vote on tags. This is like a wiki where everyone can contribute! Tags help users find stories they'll enjoy and avoid stories they won't.

## Tag Types

FicHub has six types of tags:

| ID | Type | Example | Description |
|----|------|---------|-------------|
| 1 | Fandom | Harry Potter | What universe the story is set in |
| 2 | Character | Hermione Granger | Which characters appear |
| 3 | Relationship | Harry/Hermione | Romantic or platonic pairings |
| 4 | Freeform | Angst, Fluff | Any other descriptors |
| 5 | Warning | Character Death | Content warnings |
| 6 | Category | F/M, Gen | Target audience |

Each tag type serves a different purpose:
- **Fandom** tags help users find stories in their favorite universe
- **Character** tags help users find stories featuring their favorite characters
- **Relationship** tags help users find stories with their favorite pairings
- **Freeform** tags are the most flexible — they can be anything
- **Warning** tags help users avoid content they don't want to see
- **Category** tags describe the general audience for the story

## Tag Resolution

When someone submits a tag, FicHub needs to figure out if it's new or already exists. This is a three-step process:

```rust
pub async fn resolve_tag(
    pool: &PgPool,
    tag_name: &str,
    tag_type_id: i16,
) -> AppResult<TagResolution> {
    // Step 1: Look up by exact name (case-sensitive)
    if let Some(row) = lookup_tag(pool, tag_name).await? {
        return Ok(TagResolution {
            tag_id: row.0,
            tag_name: row.1,
            tag_type_id: row.2,
            is_new: false,
        });
    }

    // Step 2: Look up in aliases (e.g., "HP" → "Harry Potter")
    if let Some(canonical_id) = lookup_alias(pool, tag_name).await? {
        let tag = lookup_tag_by_id(pool, canonical_id).await?
            .ok_or_else(|| AppError::Internal(format!(
                "alias '{}' points to non-existent tag", tag_name
            )))?;
        return Ok(TagResolution {
            tag_id: tag.0,
            tag_name: tag.1,
            tag_type_id: tag.2,
            is_new: false,
        });
    }

    // Step 3: Create new tag
    let new_id = create_tag(pool, tag_name, tag_type_id).await?;
    Ok(TagResolution {
        tag_id: new_id,
        tag_name: tag_name.to_string(),
        tag_type_id,
        is_new: true,
    })
}
```

### Why Three Steps?

1. **Exact match** — Most common case. The tag already exists exactly as written.
2. **Alias lookup** — Handles abbreviations and alternate spellings. "HP" becomes "Harry Potter", "Drarry" becomes "Draco/Harry".
3. **Create new** — Only if the tag doesn't exist at all. This keeps the tag database clean.

### Case Sensitivity

Tag lookup is **case-sensitive** (using PostgreSQL's `COLLATE "C"`). This means "Angst" and "angst" are different tags. This prevents duplicate tags with different capitalization.

## Community Voting

Users can upvote (+1) or downvote (-1) tags. The score determines visibility:

```rust
pub async fn record_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: IpAddr,
    value: i16,
    hidden_threshold: i16,
) -> AppResult<VoteResult> {
    // Upsert the vote (INSERT or UPDATE if exists)
    sqlx::query(
        r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, voter_ip) DO UPDATE SET value = EXCLUDED.value"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .bind(value)
    .execute(pool)
    .await?;

    // Read back the updated score
    let row: (i16,) = sqlx::query_as(
        "SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;

    let new_score = row.0;
    let hidden = is_hidden(new_score, hidden_threshold);

    Ok(VoteResult { new_score, hidden })
}
```

### How Voting Works

1. User submits a vote (+1 or -1)
2. The vote is recorded in `fic_tag_votes` (one vote per IP per tag per story)
3. If the user already voted, their previous vote is updated
4. The tag's score in `fic_tags` is automatically updated (via a database trigger)
5. If the score drops below the threshold, the tag becomes hidden

### The Hidden Threshold

Tags with scores below a threshold are hidden:

```rust
pub fn is_hidden(score: i16, threshold: i16) -> bool {
    score <= threshold
}

pub fn compute_visibility(scores: &[(i32, i16)], threshold: i16) -> Vec<(i32, bool)> {
    scores
        .iter()
        .map(|(tag_id, score)| (*tag_id, is_hidden(*score, threshold)))
        .collect()
}
```

The default threshold is `-3` — so a tag needs at least 3 downvotes to be hidden. This prevents a single disgruntled user from hiding tags.

### Why Not Delete Low-Scored Tags?

Keeping low-scored tags (instead of deleting them) has advantages:
- The tag might be useful for other stories
- Voting patterns can change over time
- It preserves history and prevents abuse
- Hidden tags can be unhidden if the community changes its mind

## Curator Features

FicHub has **curators** — trusted users who can perform administrative actions:

```rust
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AliasBody>,
) -> Result<Json<Value>, AppError> {
    // Verify curator token
    let token = body.token.as_deref()
        .ok_or_else(|| AppError::BadRequest(-1, "token required".into()))?;

    if Some(token.to_string()) != state.config.curator_token {
        return Ok(Json(json!({"err": -3, "msg": "invalid curator token"})));
    }

    // Create the alias
    queries::create_tag_alias(&state.db, &body.alias_name, body.canonical_tag_id).await?;

    Ok(Json(json!({"err": 0, "msg": "alias created"})))
}
```

Curators can:
- **Create aliases** — Map alternate names to canonical tags
- **Merge tags** — Combine duplicate tags into one
- **Delete tags** — Remove inappropriate or duplicate tags
- **Resolve flags** — Review and act on user-reported tags

### The Merge Operation

Merging tags is a complex database operation:

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

When merging, if both tags exist on the same story, the higher score is kept (`GREATEST`).

## Rate Limiting for Tags

To prevent abuse, FicHub rate-limits tag operations:

```rust
pub async fn check_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
    max_per_hour: u32,
) -> AppResult<()> {
    let count: Option<u32> = redis.get(key).await.unwrap_or(None);
    if count.unwrap_or(0) >= max_per_hour {
        return Err(crate::error::AppError::RateLimited(3600));
    }
    let _: () = redis.incr(key, 1).await.unwrap_or_default();
    let _: () = redis.expire(key, 3600).await.unwrap_or_default();
    Ok(())
}
```

Default limits:
- Tag submissions: 10 per hour per IP
- Tag votes: 20 per hour per IP

> ### 🔧 Try It Yourself
> 
> 1. Why does tag resolution check aliases before creating new tags?
> 2. What's the benefit of having a hidden threshold instead of just deleting low-scored tags?
> 3. How would you prevent a user from submitting 1000 tags per minute?
> 4. What happens if a curator creates an alias that creates a loop (A → B → A)?

---

# Chapter 17: Smart Search — Finding Stories Fast

## What Is Full-Text Search?

Full-text search is like a super-powered search engine inside your database. Instead of just matching exact words, it can:

- Find words that are similar (like "running" matching "run")
- Rank results by relevance
- Handle multiple words in any order
- Ignore common words like "the" and "and"
- Handle different word forms (like "runs" matching "run")

PostgreSQL has built-in full-text search, and FicHub uses it to help users find stories!

## How PostgreSQL Full-Text Search Works

PostgreSQL converts text into a special `tsvector` type that's optimized for searching:

```sql
-- Convert text to tsvector
SELECT to_tsvector('english', 'Harry Potter ran quickly through the castle');
-- Returns: 'castle':9 'harry':1 'potter':2 'quickli':7 'ran':4

-- Search using tsquery
SELECT * FROM fic_info
WHERE text_search @@ plainto_tsquery('english', 'harry potter');
```

The `plainto_tsquery` function converts user input into a search query. The `@@` operator checks if a row matches the query.

### Ranking Results

PostgreSQL can rank results by relevance:

```sql
SELECT *, ts_rank(text_search, plainto_tsquery('english', 'harry potter')) AS rank
FROM fic_info
WHERE text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY rank DESC;
```

The `ts_rank` function returns a score from 0 to 1. Higher scores mean more relevant results.

## The Search Query Builder

FicHub has a sophisticated search system that supports many filters:

```rust
pub struct SearchParams {
    pub q: Option<String>,           // Full-text search query
    pub include_tags: Vec<TagFilter>, // ALL these tags must be present (AND)
    pub exclude_tags: Vec<TagFilter>, // NONE of these tags (AND NOT)
    pub include_any_tags: Vec<TagFilter>, // At least ONE of these tags (OR)
    pub min_words: Option<i64>,
    pub max_words: Option<i64>,
    pub min_chapters: Option<i32>,
    pub max_chapters: Option<i32>,
    pub complete: Option<bool>,       // Filter by completion status
    pub source: Option<String>,       // Filter by source site
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub sort: Option<String>,         // Sort field
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
```

## Building Dynamic SQL

The search builder constructs SQL queries dynamically based on the filters:

```rust
pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
    let has_q = self.params.q.is_some();

    let mut qb = QueryBuilder::<Postgres>::new("SELECT fi.*, ");

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
        if has_q { "-relevance" } else { "-date" },
    );
    qb.push(" ORDER BY ");
    match sort {
        "-relevance" => {
            if has_q { qb.push("rank DESC"); }
            else { qb.push("fi.fic_updated DESC"); }
        }
        "-date" => qb.push("fi.fic_updated DESC"),
        "-words" => qb.push("fi.words DESC"),
        "-chapters" => qb.push("fi.chapters DESC"),
        "-title" => qb.push("fi.title ASC"),
        _ => qb.push("fi.fic_updated DESC"),
    }

    // LIMIT / OFFSET for pagination
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

### The Query Builder Pattern

FicHub uses `sqlx::QueryBuilder` instead of raw SQL strings. This provides:
1. **SQL injection protection** — Parameters are automatically escaped
2. **Dynamic queries** — Add WHERE clauses based on which filters are provided
3. **Type safety** — Bind values are checked at compile time

### Tag Filters in SQL

Tag filters use SQL `EXISTS` subqueries:

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

    // Word count filters
    if let Some(min) = self.params.min_words {
        qb.push(" AND fi.words >= ");
        qb.push_bind(min);
    }
    if let Some(max) = self.params.max_words {
        qb.push(" AND fi.words <= ");
        qb.push_bind(max);
    }

    // Completion filter
    if let Some(true) = self.params.complete {
        qb.push(" AND fi.status = 'complete'");
    }
    if let Some(false) = self.params.complete {
        qb.push(" AND fi.status != 'complete'");
    }
}
```

### How Tag Filters Work

- **`include_tags`** — Uses `EXISTS` to check that ALL specified tags are present
- **`exclude_tags`** — Uses `NOT EXISTS` to ensure NONE of the specified tags are present
- **`include_any_tags`** — Uses `EXISTS` with `OR` to check that at least ONE tag is present

The `ft.score >= hidden_threshold` ensures hidden tags don't affect search results.

## Tag Filter Parsing

Tags are specified as "type_id:name" pairs in the URL:

```rust
pub fn parse_tag_filters(input: &str) -> Result<Vec<TagFilter>, String> {
    let mut filters = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }

        let colon_pos = part.find(':').ok_or_else(|| {
            format!("Invalid tag filter format '{}': expected 'type_id:name'", part)
        })?;
        let type_id: i16 = part[..colon_pos].parse()
            .map_err(|e| format!("Invalid tag type_id: {}", e))?;
        let name = part[colon_pos + 1..].to_string();
        filters.push(TagFilter { tag_type_id: type_id, tag_name: name });
    }
    Ok(filters)
}
```

Example URL: `?include_tags=1:Harry+Potter,2:Hermione+Granger`

This becomes two `TagFilter` entries:
- `{ tag_type_id: 1, tag_name: "Harry Potter" }`
- `{ tag_type_id: 2, tag_name: "Hermione Granger" }`

## Pagination

Search results are paginated to handle large result sets:

```rust
let page = self.params.page.unwrap_or(1).max(1);
let per_page = self.params.per_page.unwrap_or(20).max(1);
let offset = (page - 1) * per_page;
```

- `page` defaults to 1 (first page)
- `per_page` defaults to 20 results per page
- Both have minimum values to prevent weird behavior
- `search_max_per_page` config limits the maximum results per page (default: 50)

## The Search Handler

```rust
pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<Value>, AppError> {
    let search_params = SearchParams {
        q: params.q,
        include_tags: parse_tag_filters(&params.include_tags.unwrap_or_default())?,
        exclude_tags: parse_tag_filters(&params.exclude_tags.unwrap_or_default())?,
        min_words: params.min_words,
        max_words: params.max_words,
        complete: params.complete,
        source: params.source,
        sort: params.sort,
        page: params.page,
        per_page: params.per_page,
        ..Default::default()
    };

    let builder = SearchQueryBuilder::new(search_params, state.config.tag_hidden_threshold);

    // Get total count (for pagination)
    let count_query = builder.build_count_query();
    let total: (i64,) = count_query.fetch_one(&state.db).await?;

    // Get results
    let data_query = builder.build_data_query();
    let rows: Vec<FicSearchRow> = data_query.fetch_all(&state.db).await?;

    let results: Vec<SearchResult> = rows.into_iter().map(Into::into).collect();

    Ok(Json(json!({
        "err": 0,
        "total": total.0,
        "page": builder.page(),
        "results": results,
    })))
}
```

> ### ⚠️ Watch Out!
> 
> Full-text search can be slow on large databases! FicHub uses a GIN index on the `text_search` column to make searches fast. Without this index, searching would scan every row in the table (called a "sequential scan"), which could take seconds on large tables.
>
> Also, `ILIKE` queries (used in `search_similar_fics`) don't use indexes efficiently. For small datasets it's fine, but for large datasets you'd want to use full-text search instead.

> ### 🔧 Try It Yourself
> 
> 1. What's the difference between `include_tags` (AND) and `include_any_tags` (OR)?
> 2. Why does the sort default to "relevance" when there's a search query but "date" when there isn't?
> 3. How would you add a filter for "stories updated in the last 30 days"?
> 4. What happens if someone searches for a very common word like "the"?

---

# Chapter 18: The Recommendations Engine — Finding Similar Stories

## What Is Collaborative Filtering?

Collaborative filtering is the same technology Netflix and Amazon use to recommend movies and products. The idea is simple: **people who liked the same things you like probably share other tastes too.**

For FicHub, this means:
1. Find people who bookmarked the same story you're reading
2. Look at what OTHER stories those people bookmarked
3. Recommend those stories to you!

This is called "collaborative filtering" because users collaborate (indirectly) to filter recommendations. You don't need to explicitly tell FicHub what you like — your bookmarking behavior says it for you.

## How FicHub's Recommender Works

The recommendation engine has three main parts:

### Part 1: Data Collection (The Worker)

A background worker scrapes user favorites from fanfiction sites:

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiter>,
}
```

The worker runs in an infinite loop, processing items from a Redis queue:

```rust
pub async fn run(&self) {
    info!("Collection worker started — polling Redis queues");
    loop {
        for fetcher in &self.fetchers {
            let domain = fetcher.site_domain();
            let key = format!("collection_queue:{}", domain);

            let item_str: Option<String> = {
                let mut conn = self.redis.lock().await;
                redis::cmd("LPOP").arg(&key).query_async(&mut *conn).await.unwrap_or(None)
            };

            if let Some(item_str) = item_str {
                // Apply rate limit before making HTTP requests
                if let Some(rl) = self.rate_limiters.get(domain) {
                    rl.wait_if_needed().await;
                }

                if let Ok(item) = serde_json::from_str::<QueueItem>(&item_str) {
                    debug!("Processing {} from {}", item.url_id, domain);
                    if let Err(e) = self.process_work(item).await {
                        error!("Error processing work on {}: {}", domain, e);
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
```

### The Collection Process

When processing a work, the worker:

1. **Fetches favouriters** — Users who bookmarked this story
2. **For each new user** — Fetches their other bookmarks
3. **Updates co-occurrence** — Records which stories appear together
4. **Updates favouriter count** — How many people bookmarked this story

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = self.fetchers.iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or_else(|| format!("No fetcher for {}", item.site_domain))?;

    // Fetch favouriters
    let favouriters = fetcher.collect_favouriters(&self.client, &work_url, max_pages).await?;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // Skip if already recorded
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)",
        ).bind(&user_hash).bind(&item.url_id).fetch_one(&self.db).await.unwrap_or(false);

        if exists { continue; }

        // Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        ).bind(&user_hash).bind(&item.url_id).bind(&item.site_domain)
         .execute(&self.db).await?;

        // Fetch user's other favourites
        let user_favs = fetcher.collect_user_favourites(&self.client, user_url, max_pages).await?;

        // Update co-occurrence
        for fav_url_id in &user_favs {
            if fav_url_id != &item.url_id {
                sqlx::query(
                    r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, cooccur_count)
                       VALUES ($1, $2, 1)
                       ON CONFLICT (work_a, work_b)
                       DO UPDATE SET cooccur_count = fic_bookmark_cooccur.cooccur_count + 1"#,
                ).bind(&item.url_id).bind(fav_url_id).execute(&self.db).await?;
            }
        }
    }

    Ok(())
}
```

### Part 2: The Recommendation Engine

The engine computes recommendations using co-occurrence analysis:

```rust
async fn compute_live(db: &PgPool, query: &RecQuery, config: &Config, limit: usize)
    -> Result<Vec<RecResult>, AppError>
{
    // Get seed metadata
    let seed = sqlx::query_as::<_, (i32, String, String)>(
        r#"SELECT COALESCE(fw.favouriter_count, 0),
                  COALESCE(fi.title, ''), COALESCE(fi.author, '')
           FROM fic_works fw JOIN fic_info fi ON fi.id = fw.url_id
           WHERE fw.url_id = $1"#,
    ).bind(&query.url_id).fetch_optional(db).await?
        .ok_or_else(|| AppError::NotFound(format!("Work {} not found", query.url_id)))?;

    let (favouriter_count, seed_title, seed_author) = seed;

    // Get co-occurrence candidates
    let cooccur: Vec<CooccurRow> = sqlx::query_as::<_, CooccurRow>(
        r#"WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
            candidates AS (
              SELECT CASE WHEN work_a = $1 THEN work_b ELSE work_a END AS candidate_id,
                     cooccur_count
              FROM fic_bookmark_cooccur WHERE work_a = $1 OR work_b = $1
            )
            SELECT c.candidate_id, c.cooccur_count, fw.favouriter_count,
              (c.cooccur_count::float / (s.favouriter_count + fw.favouriter_count - c.cooccur_count))
              AS jaccard
            FROM candidates c JOIN fic_works fw ON fw.url_id = c.candidate_id
            CROSS JOIN seed s
            WHERE c.candidate_id != $1 ORDER BY jaccard DESC LIMIT $2"#,
    ).bind(&query.url_id).bind(limit as i64).fetch_all(db).await?;

    // Tag fallback for stories with few favouriters
    let min_collab = config.rec_min_favouriters_for_collab as i32;
    let use_tag_fallback = favouriter_count < min_collab;

    let tag_candidates = if use_tag_fallback {
        fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
    } else {
        Vec::new()
    };

    // Blend collaborative + tag scores
    let weight = (favouriter_count as f64 / 5.0).min(1.0);
    for c in &mut candidates {
        let collab = c.score;
        let tag = c.tag_score;
        c.score = collab * weight + tag * (1.0 - weight);
    }

    // Community vote boost
    let gamma = config.rec_voting_boost_gamma;
    for c in &mut candidates {
        let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
        let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
        c.score *= boost;
    }

    // Sort and take top N
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(candidates.into_iter().take(limit).collect())
}
```

### The Jaccard Similarity

The key metric is **Jaccard similarity**:

```
jaccard = cooccur_count / (favouriter_count_A + favouriter_count_B - cooccur_count)
```

This measures how much two stories' fan bases overlap. A Jaccard of 1.0 means the same people liked both stories. A Jaccard of 0.0 means no overlap.

### Tag Fallback

When a story has few favouriters (less than `rec_min_favouriters_for_collab`, default 5), collaborative filtering doesn't have enough data. In this case, FicHub falls back to tag-based recommendations:

```rust
async fn fetch_tag_candidates(
    db: &PgPool, url_id: &str, seed_title: &str, seed_author: &str, limit: usize,
) -> Result<Vec<(String, f64)>, AppError> {
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    seen.insert(url_id.to_string());

    // Same author (strong signal — score 0.8)
    if !seed_author.is_empty() {
        let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info WHERE id != $1 AND author ILIKE $2 ORDER BY words DESC LIMIT $3"#,
        ).bind(url_id).bind(format!("%{}%", seed_author)).bind(limit as i64)
         .fetch_all(db).await?;

        for row in rows {
            if seen.insert(row.id.clone()) {
                results.push((row.id, 0.8));
            }
        }
    }

    // Title keyword matches (weaker signal — score 0.5)
    let keywords: Vec<&str> = seed_title.split_whitespace().filter(|w| w.len() >= 3).collect();
    for kw in keywords {
        if results.len() >= limit { break; }
        let rows: Vec<FicInfoRow> = sqlx::query_as::<_, FicInfoRow>(
            r#"SELECT id, title, author, words, chapters, status, source, description
               FROM fic_info WHERE id != $1 AND title ILIKE $2 ORDER BY words DESC LIMIT $3"#,
        ).bind(url_id).bind(format!("%{}%", kw)).bind((limit - results.len()) as i64)
         .fetch_all(db).await?;

        for row in rows {
            if seen.insert(row.id.clone()) {
                results.push((row.id, 0.5));
            }
        }
    }

    Ok(results)
}
```

The tag fallback gives:
- **0.8 score** for stories by the same author
- **0.5 score** for stories with matching title keywords

### Community Vote Boost

Users can suggest and vote on recommendations:

```rust
let gamma = config.rec_voting_boost_gamma;  // Default: 0.2
for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The boost uses a logarithmic scale — 1 vote adds a small boost, 10 votes add more, but the boost diminishes with more votes. This prevents vote manipulation from completely dominating the algorithm.

## Per-Site Rate Limiting

The worker uses atomic CAS loops to enforce rate limits per site:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

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
            // CAS failed — retry
        } else {
            let remaining = delay_nanos - elapsed as u64;
            tokio::time::sleep(Duration::from_nanos(remaining)).await;
        }
    }
}
```

This ensures that even with many concurrent workers, requests to the same site are properly spaced out. The atomic CAS (Compare-And-Swap) loop prevents race conditions.

## The SiteFetcher Trait

Each fanfiction site needs its own fetcher for collecting user favorites:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    fn site_domain(&self) -> &str;

    async fn collect_favouriters(&self, client: &Client, work_url: &str, max_pages: u32)
        -> Result<Vec<String>, ScrapeError>;

    async fn collect_user_favourites(&self, client: &Client, user_url: &str, max_pages: u32)
        -> Result<Vec<String>, ScrapeError>;

    fn user_hash(&self, user_url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_url.to_lowercase().as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

User hashes are SHA-256 of their profile URL — this keeps user identities anonymous while still tracking favorites.

> ### 🔧 Try It Yourself
> 
> 1. Why does the engine use Jaccard similarity instead of just counting co-occurrences?
> 2. What's the purpose of the `rec_voting_boost_gamma` config value?
> 3. How would you add support for a new fanfiction site's favorites?
> 4. Why use a logarithmic boost for votes instead of linear?

---

# Chapter 19: OPDS Catalog — E-Reader Integration

## What Is OPDS?

OPDS stands for **Open Publication Distribution System**. It's like a library catalog that e-readers can browse. When you add an OPDS catalog to your e-reader, you can:

- Browse stories by genre, author, or tag
- Search for stories
- Download EPUB files directly to your e-reader
- See recently added and popular stories

OPDS feeds are XML files that follow a specific format (based on Atom feeds). Most e-readers (Kindle, Kobo, Nook, Calibre) support OPDS natively.

## The Root Catalog

When you connect your e-reader to FicHub's OPDS catalog, you see the root page with categories:

```rust
pub async fn root_catalog() -> impl IntoResponse {
    let entries = format!(
        r#"<entry>
    <title>New Stories</title>
    <id>urn:fichub:opds:new</id>
    <updated>{updated}</updated>
    <content type="text">Recently added fanfiction</content>
    <link href="/opds/new" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Popular Stories</title>
    <id>urn:fichub:opds:popular</id>
    <updated>{updated}</updated>
    <content type="text">Most popular fanfiction</content>
    <link href="/opds/popular" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Browse by Tag</title>
    <id>urn:fichub:opds:tags</id>
    <updated>{updated}</updated>
    <content type="text">Browse stories by tag</content>
    <link href="/opds/tags" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Authors</title>
    <id>urn:fichub:opds:authors</id>
    <updated>{updated}</updated>
    <content type="text">Browse stories by author</content>
    <link href="/opds/authors" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Search</title>
    <id>urn:fichub:opds:search</id>
    <updated>{updated}</updated>
    <content type="text">Search for stories</content>
    <link href="/opds/search" rel="subsection" type="application/atom+xml"/>
</entry>"#,
        updated = iso_now(),
    );

    let body = build_feed(
        "FicHub Catalog",
        "urn:fichub:opds:root",
        &entries,
        &iso_now(),
        Some("/opds"),
        FeedKind::Navigation,
        None,
    );

    opds_response(body, FeedKind::Navigation)
}
```

## OPDS Entry Format

Each story in the catalog is an **entry** with specific fields:

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
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <link rel="http://opds-spec.org/image" href="/cache/cover/{url_id}.jpg" type="image/jpeg"/>
    <link rel="http://opds-spec.org/image/thumbnail" href="/cache/cover/{url_id}_thumb.jpg" type="image/jpeg"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/epub/{url_id}/{url_id}.epub" type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition" href="/cache/pdf/{url_id}/{url_id}.pdf" type="application/pdf"/>
    <link rel="alternate" href="/fics/{url_id}" type="text/html"/>
  </entry>"#,
        title = html_escape(title),
        author = html_escape(author),
        summary = html_escape(summary),
        // ...
    )
}
```

The `link` elements tell the e-reader where to download files. The `rel="acquisition"` link is what the e-reader uses to download the EPUB.

## Navigation vs Acquisition Feeds

OPDS has two types of feeds:

### Navigation Feeds

Navigation feeds list categories (like a table of contents):

```rust
pub enum FeedKind {
    Navigation,
    Acquisition,
}

impl FeedKind {
    pub fn content_type(&self) -> &'static str {
        match self {
            FeedKind::Navigation => "application/atom+xml;profile=opds-catalog;kind=navigation",
            FeedKind::Acquisition => "application/atom+xml;profile=opds-catalog;kind=acquisition",
        }
    }
}
```

### Acquisition Feeds

Acquisition feeds list actual stories that can be downloaded. Here's the "recently updated" feed:

```rust
pub async fn recent_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();

    let fics: Vec<FicInfo> = sqlx::query_as(
        "SELECT * FROM fic_info ORDER BY fic_updated DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page as i64).bind(offset as i64)
    .fetch_all(&state.db).await.unwrap_or_default();

    let entries: String = fics.iter().map(|fic| {
        fic_entry(
            &fic.id, &fic.title, &fic.author, &fic.description,
            &fic.fic_updated.to_rfc3339(), fic.words, fic.chapters, &fic.status,
        )
    }).collect();

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db).await.unwrap_or(0);

    let body = build_feed(
        "Recently Updated",
        "urn:fichub:opds:new",
        &entries,
        &iso_now(),
        Some("/opds/new"),
        FeedKind::Acquisition,
        Some(&PaginationInfo {
            base_path: "/opds/new".to_string(),
            page, per_page, total,
        }),
    );

    opds_response(body, FeedKind::Acquisition)
}
```

## Pagination

OPDS feeds support pagination for large result sets:

```rust
pub struct PaginationInfo {
    pub base_path: String,
    pub page: usize,
    pub per_page: usize,
    pub total: i64,
}
```

The feed includes `<link rel="previous">` and `<link rel="next">` elements so the e-reader can navigate pages. The `PageParams` struct handles the query parameters:

```rust
pub struct PageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl PageParams {
    pub fn page(&self) -> usize { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> usize { self.per_page.unwrap_or(50).max(1).min(100) }
    pub fn offset(&self) -> usize { (self.page() - 1) * self.per_page() }
}
```

## The Feed Builder

```rust
pub fn build_feed(
    title: &str, feed_id: &str, entries: &str, updated: &str,
    self_link: Option<&str>, feed_kind: FeedKind,
    pagination: Option<&PaginationInfo>,
) -> String {
    let mut links = String::new();

    if let Some(link) = self_link {
        links.push_str(&format!(
            r#"  <link href="{link}" rel="self" type="{ct}"/>"#,
            link = html_escape(link), ct = feed_kind.content_type(),
        ));
    }

    links.push_str(&format!(
        r#"  <link href="/opds" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>"#,
    ));

    // Add pagination links
    if let Some(ref pagi) = pagination {
        let total_pages = (pagi.total as f64 / pagi.per_page as f64).ceil() as usize;
        if pagi.page > 1 {
            let prev_href = format!("{}?page={}&per_page={}",
                pagi.base_path, pagi.page - 1, pagi.per_page);
            links.push_str(&format!(
                r#"  <link href="{prev}" rel="previous" type="{ct}"/>"#,
                prev = html_escape(&prev_href), ct = feed_kind.content_type(),
            ));
        }
        if pagi.page < total_pages {
            let next_href = format!("{}?page={}&per_page={}",
                pagi.base_path, pagi.page + 1, pagi.per_page);
            links.push_str(&format!(
                r#"  <link href="{next}" rel="next" type="{ct}"/>"#,
                next = html_escape(&next_href), ct = feed_kind.content_type(),
            ));
        }
    }

    format!(
        r#"{header}<id>{id}</id>
  <title>{title}</title>
  <updated>{updated}</updated>
  <author><name>FicHub</name></author>
{links}{entries}</feed>"#,
        header = atom_xml_header(),
        id = html_escape(feed_id), title = html_escape(title),
        updated = updated, links = links, entries = entries,
    )
}
```

## Shelf Support

FicHub also supports OPDS shelves — authenticated collections of stories:

```rust
pub async fn shelf_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ShelfQuery>,
) -> impl IntoResponse {
    // Verify shelf token
    let token = params.token.as_deref().unwrap_or("");
    if token != state.config.opds_shelf_token {
        return opds_response(
            build_feed("Access Denied", "urn:fichub:opds:denied", "", &iso_now(), None, FeedKind::Navigation, None),
            FeedKind::Navigation,
        );
    }

    // Return shelf contents
    // ...
}
```

> ### 🔧 Try It Yourself
> 
> 1. Why does OPDS use XML instead of JSON?
> 2. What's the difference between a navigation feed and an acquisition feed?
> 3. How would you add a "Bookmarks" section to the OPDS catalog?
> 4. What security considerations are there for the shelf token?

---

# Chapter 20: Docker & Deployment — Shipping It Out

## What Is Docker?

Docker is like a magical box that packages your entire program with everything it needs to run. When you put FicHub in a Docker box, it works the same way on any computer — your laptop, a server in a data center, or a friend's computer.

Think of Docker like a shipping container. Inside the container, everything is organized and secured. The container can be shipped anywhere in the world, and when it arrives, everything inside works perfectly.

## Why Docker?

Without Docker, deploying FicHub would require:
1. Installing Rust on the server
2. Installing the correct version of Rust
3. Installing PostgreSQL
4. Installing Redis
5. Setting up all the right versions
6. Configuring everything to work together
7. Hoping nothing conflicts with other software on the server

With Docker, you just run one command and everything works!

## The Dockerfile

A Dockerfile is a recipe for building a Docker image:

```dockerfile
# Build stage — compile the Rust program
FROM rust:1.78 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# Runtime stage — minimal image with just the binary
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y     ca-certificates     && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/fichub /usr/local/bin/fichub
COPY --from=builder /app/migrations /usr/local/bin/migrations

# Copy frontend assets
COPY frontend/build /app/frontend/build

# Create directories for cache and temp files
RUN mkdir -p /app/cache /app/tmp

# Expose the port
EXPOSE 3000

# Start the server
CMD ["fichub"]
```

### Multi-Stage Build

This Dockerfile uses a **multi-stage build**:

1. **Build stage** — Uses a full Rust image (about 1.5GB) to compile the program
2. **Runtime stage** — Uses a minimal Debian image (about 80MB) with just the compiled binary

This makes the final Docker image much smaller (about 50MB instead of 500MB). The build tools aren't needed at runtime, so we leave them behind.

### How Each Line Works

- `FROM rust:1.78 as builder` — Start with the Rust image, name this stage "builder"
- `WORKDIR /app` — Set the working directory to /app
- `COPY Cargo.toml Cargo.lock ./` — Copy dependency files first (for caching)
- `COPY src ./src` — Copy source code
- `COPY migrations ./migrations` — Copy database migrations
- `RUN cargo build --release` — Compile with optimizations
- `FROM debian:bookworm-slim` — Start fresh with minimal Debian
- `COPY --from=builder ...` — Copy just the compiled binary from the builder stage
- `EXPOSE 3000` — Document that the container uses port 3000
- `CMD ["fichub"]` — Run the fichub binary when the container starts

## Docker Compose

For running FicHub with PostgreSQL and Redis, use Docker Compose:

```yaml
version: '3.8'
services:
  fichub:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://fichub:password@db:5432/fichub
      - REDIS_URL=redis://redis:6379/
      - CACHE_DIR=/app/cache
      - PORT=3000
      - NODE_NAME=production
    volumes:
      - fichub_cache:/app/cache
      - fichub_tmp:/app/tmp
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_started
    restart: unless-stopped

  db:
    image: postgres:16
    environment:
      - POSTGRES_USER=fichub
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=fichub
    volumes:
      - pg_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  pg_data:
  redis_data:
  fichub_cache:
  fichub_tmp:
```

### Services

- **fichub** — The FicHub server
- **db** — PostgreSQL database
- **redis** — Redis cache

### Volumes

Volumes persist data between container restarts:
- `pg_data` — PostgreSQL data (your stories database!)
- `redis_data` — Redis data (rate limiting state)
- `fichub_cache` — Cached EPUB files
- `fichub_tmp` — Temporary files during generation

### Health Checks

Health checks tell Docker when a service is ready:

```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U fichub"]
  interval: 5s
  timeout: 5s
  retries: 5
```

FicHub waits for PostgreSQL to be healthy before starting, preventing connection errors.

## Running FicHub

With Docker Compose, starting everything is one command:

```bash
docker compose up -d
```

This:
1. Builds the FicHub image (first time only)
2. Pulls PostgreSQL and Redis images (first time only)
3. Creates the volumes
4. Starts PostgreSQL, Redis, and FicHub
5. Runs database migrations
6. Starts listening on port 3000

### Useful Commands

```bash
# Start everything
docker compose up -d

# View logs
docker compose logs -f fichub

# Stop everything
docker compose down

# Rebuild after code changes
docker compose up -d --build

# Access PostgreSQL directly
docker compose exec db psql -U fichub

# Access Redis directly
docker compose exec redis redis-cli
```

## Environment Variables in Production

For production, you should use more secure settings:

```env
DATABASE_URL=postgres://fichub:very_secure_password@db:5432/fichub
REDIS_URL=redis://redis:6379/
CACHE_DIR=/app/cache
PORT=3000
NODE_NAME=production-server
TRUSTED_PROXIES=10.0.0.0/8,172.16.0.0/12
DYNAMIC_RATE_LIMIT=true
```

### Security Considerations

1. **Never hard-code passwords** in the Dockerfile or docker-compose.yml
2. **Use Docker secrets** or environment variables for sensitive data
3. **Don't expose Redis** to the outside world (it has no authentication by default)
4. **Use strong passwords** for PostgreSQL
5. **Enable TLS** for production deployments

## Scaling

For high traffic, you can run multiple FicHub instances:

```yaml
services:
  fichub:
    build: .
    deploy:
      replicas: 3
```

Since FicHub stores data in PostgreSQL and Redis (not in memory), multiple instances can share the same databases. This is called "horizontal scaling."

### Load Balancing

When running multiple instances, you need a load balancer to distribute requests:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - fichub1
      - fichub2
      - fichub3
```

## Monitoring

FicHub includes Prometheus metrics via the `axum-prometheus` crate:

```toml
axum-prometheus = "0.10"
```

This automatically tracks:
- Request count per endpoint
- Response times (histograms)
- Error rates
- Active connections

You can visualize these with Grafana for a beautiful monitoring dashboard.

## Backup Strategy

Regular backups are essential for production:

```bash
# Backup PostgreSQL
docker compose exec db pg_dump -U fichub fichub > backup_$(date +%Y%m%d).sql

# Backup cached files
tar -czf cache_backup_$(date +%Y%m%d).tar.gz -C /var/lib/docker/volumes/fichub_fichub_cache .

# Restore PostgreSQL
docker compose exec -T db psql -U fichub fichub < backup_20240115.sql
```

## Performance Tuning

### PostgreSQL Optimization

```sql
-- Add indexes for common queries
CREATE INDEX idx_fic_info_updated ON fic_info(fic_updated DESC);
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_source ON fic_info(source);
CREATE INDEX idx_fic_tags_url_id ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag_id ON fic_tags(tag_id);
```

### Redis Configuration

```conf
# Increase memory limit
maxmemory 256mb
maxmemory-policy allkeys-lru
```

### Rust Optimizations

```bash
# Build with maximum optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

> ### ⚠️ Watch Out!
> 
> Never hard-code passwords in your Dockerfile! Always use environment variables or Docker secrets. The docker-compose.yml file should also be excluded from version control if it contains secrets.

> ### 🔧 Try It Yourself
> 
> 1. Why does the Dockerfile use a multi-stage build?
> 2. What would happen if you didn't use volumes for PostgreSQL data?
> 3. How would you set up automatic backups for the database?
> 4. How would you monitor FicHub's performance in production?

---

# Appendix A: Quick Reference

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point — starts the server |
| `src/lib.rs` | Library root — lists all modules |
| `src/config.rs` | Configuration from environment variables |
| `src/server.rs` | Axum server setup and routing |
| `src/error.rs` | Error types and HTTP responses |
| `src/db/mod.rs` | Database connection pool |
| `src/db/models.rs` | Database table structs |
| `src/db/queries.rs` | SQL queries |
| `src/scrape/mod.rs` | Scraper trait and types |
| `src/scrape/registry.rs` | Scraper registry |
| `src/scrape/sites/*.rs` | Site-specific scrapers |
| `src/export/mod.rs` | Export types and error handling |
| `src/export/epub.rs` | EPUB generation |
| `src/export/html_bundle.rs` | HTML bundle generation |
| `src/cache/mod.rs` | Cache semaphores and EType |
| `src/cache/disk.rs` | Disk cache operations |
| `src/limiter/mod.rs` | Rate limiter trait |
| `src/limiter/redis_bucket.rs` | Redis token bucket |
| `src/tags/mod.rs` | Tagging system |
| `src/tags/resolve.rs` | Tag resolution |
| `src/tags/voting.rs` | Vote recording |
| `src/tags/routes.rs` | Tag API handlers |
| `src/tags/curator.rs` | Curator endpoints |
| `src/search/mod.rs` | Search types |
| `src/search/builder.rs` | Search query builder |
| `src/search/routes.rs` | Search API handler |
| `src/recommender/mod.rs` | Recommendation types |
| `src/recommender/engine.rs` | Recommendation engine |
| `src/recommender/worker.rs` | Background collection worker |
| `src/recommender/routes.rs` | Recommendation API handlers |
| `src/routes/mod.rs` | Route module declarations |
| `src/routes/export.rs` | Export API handler |
| `src/routes/meta.rs` | Metadata API handler |
| `src/routes/cache_download.rs` | Cache download handler |
| `src/routes/api_docs.rs` | API documentation |
| `src/routes/opds/*.rs` | OPDS catalog handlers |
| `src/frontend/mod.rs` | Frontend serving |

## Common Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Run the server
cargo run

# Build Docker image
docker build -t fichub .

# Run with Docker Compose
docker compose up -d

# View logs
docker compose logs -f fichub

# Access PostgreSQL
docker compose exec db psql -U fichub fichub

# Backup database
docker compose exec db pg_dump -U fichub fichub > backup.sql
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection URL |
| `REDIS_URL` | ✅ | — | Redis connection URL |
| `CACHE_DIR` | No | `./cache` | Cached file directory |
| `SECONDARY_CACHE_DIR` | No | `None` | Secondary cache location |
| `PORT` | No | `3000` | Server port |
| `NODE_NAME` | No | `"orion"` | Server instance name |
| `EXPORT_VERSION` | No | `1` | Export version number |
| `DYNAMIC_RATE_LIMIT` | No | `true` | Smart rate limiting |
| `CALIBRE_CONTAINER` | No | `""` | Calibre Docker container |
| `TMP_DIR` | No | `./tmp` | Temporary directory |
| `FRONTEND_DIR` | No | `./frontend/build` | Frontend build directory |
| `TRUSTED_PROXIES` | No | `""` | Trusted proxy IPs |
| `CURATOR_TOKEN` | No | `None` | Curator authentication token |
| `OPDS_SHELF_TOKEN` | No | `"fichub"` | OPDS shelf authentication |
| `REC_DEFAULT_DELAY_SECS` | No | `5` | Recommender delay |
| `REC_MAX_RECOMMENDATIONS` | No | `20` | Max recommendations |
| `REC_VOTING_BOOST_GAMMA` | No | `0.2` | Vote boost factor |
| `REC_CACHE_TTL_HOURS` | No | `12` | Recommendation cache TTL |
| `TAG_HIDDEN_THRESHOLD` | No | `-3` | Tag visibility threshold |
| `SEARCH_MAX_PER_PAGE` | No | `50` | Max search results per page |

---

# Appendix B: What's Next?

## Ideas for Extending FicHub

Now that you understand how FicHub works, here are some ideas for extending it:

1. **Add a new scraper** — Pick a fanfiction site that FicHub doesn't support yet and write a scraper for it. Follow the pattern in `src/scrape/sites/ao3.rs`.

2. **Improve the recommender** — Add more signals like word count similarity, genre matching, or reading time patterns.

3. **Build a mobile app** — Use FicHub's API to create an Android or iOS app for browsing and downloading stories.

4. **Add user accounts** — Let users create accounts and track their reading history, favorites, and reading lists.

5. **Implement reading lists** — Let users create and share curated lists of stories (like "Best Harry Potter fics of 2024").

6. **Add cover art** — Generate or fetch cover images for stories. You could use AI image generation or scrape cover art from the source sites.

7. **Build a Discord bot** — Let users search and download stories from Discord using slash commands.

8. **Add translation support** — Let users contribute translations of stories to other languages.

9. **Implement reading progress** — Track where users left off in each story and sync across devices.

10. **Add social features** — Let users follow each other, see what their friends are reading, and share recommendations.

## Resources for Learning More

- **Rust Book**: [doc.rust-lang.org/book](https://doc.rust-lang.org/book/) — The official Rust tutorial
- **Axum Examples**: [github.com/tokio-rs/axum](https://github.com/tokio-rs/axum/tree/main/examples) — Example Axum applications
- **SQLx Docs**: [docs.rs/sqlx](https://docs.rs/sqlx) — SQLx documentation
- **PostgreSQL Tutorial**: [postgresqltutorial.com](https://www.postgresqltutorial.com/) — Learn SQL
- **Docker Docs**: [docs.docker.com](https://docs.docker.com/) — Docker documentation
- **OPDS Specification**: [opds-spec.org](https://opds-spec.org/) — OPDS standard documentation

## Thank You!

Thank you for reading "Building the FicHub Backend"! We hope you enjoyed learning about Rust, web scraping, EPUB generation, and all the other cool technologies that make FicHub work.

Remember: every expert was once a beginner. Keep coding, keep learning, and most importantly — have fun! 🎉

The FicHub project is open source, so if you build something cool with it, share it with the community. And if you find a bug or have an idea for improvement, don't hesitate to contribute!

---

*Happy reading, and happy coding!*

*— The FicHub Team*


# Deep Dive: The Recommendation Engine

## How Collaborative Filtering Works

Collaborative filtering is based on a simple insight: **people who liked the same things you like probably share other tastes too.**

### The Basic Algorithm

1. **Find similar users** — People who bookmarked the same stories you did
2. **Find their other bookmarks** — Stories they liked that you haven't seen
3. **Rank by similarity** — Stories liked by more similar users rank higher
4. **Return top N** — Give you the best recommendations

### Mathematical Foundation

The core metric is **Jaccard similarity**:

```
Jaccard(A, B) = |A ∩ B| / |A ∪ B|
```

Where:
- A = set of users who bookmarked story A
- B = set of users who bookmarked story B
- A ∩ B = users who bookmarked both stories
- A ∪ B = users who bookmarked either story

For example:
- Story A has 100 favouriters
- Story B has 80 favouriters
- 50 users bookmarked both stories

```
Jaccard = 50 / (100 + 80 - 50) = 50 / 130 ≈ 0.385
```

A Jaccard of 0.385 means about 38.5% overlap between the fan bases.

### Why Jaccard Instead of Raw Count?

Raw co-occurrence count is misleading:
- Story A (1000 favouriters) and Story B (10 favouriters) with 5 co-occurrences
- Story C (50 favouriters) and Story D (50 favouriters) with 5 co-occurrences

Raw count says they're equally similar, but Jaccard correctly identifies that C and D are more similar (10% overlap vs 0.5% overlap).

## FicHub's Recommendation Algorithm

### Step 1: Seed Lookup

Find the story the user is asking about:

```rust
let seed = sqlx::query_as::<_, (i32, String, String)>(
    r#"SELECT COALESCE(fw.favouriter_count, 0),
              COALESCE(fi.title, ''), COALESCE(fi.author, '')
       FROM fic_works fw
       JOIN fic_info fi ON fi.id = fw.url_id
       WHERE fw.url_id = $1"#,
).bind(&query.url_id).fetch_optional(db).await?;
```

### Step 2: Co-occurrence Query

Find stories that appear together with the seed:

```rust
let cooccur: Vec<CooccurRow> = sqlx::query_as::<_, CooccurRow>(
    r#"WITH seed AS (SELECT favouriter_count FROM fic_works WHERE url_id = $1),
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
        LIMIT $2"#,
).bind(&query.url_id).bind(limit as i64).fetch_all(db).await?;
```

This SQL query:
1. Finds all stories co-bookmarked with the seed
2. Calculates Jaccard similarity for each
3. Returns the top N candidates

### Step 3: Tag Fallback

If the seed has few favouriters (< `rec_min_favouriters_for_collab`, default 5), collaborative filtering doesn't have enough data. FicHub falls back to tag-based recommendations:

```rust
let tag_candidates = if use_tag_fallback {
    fetch_tag_candidates(db, &query.url_id, &seed_title, &seed_author, limit).await?
} else {
    Vec::new()
};
```

The tag fallback uses:
- **Same author** — Score 0.8 (strong signal)
- **Title keywords** — Score 0.5 (weaker signal)

### Step 4: Score Blending

Combine collaborative and tag-based scores:

```rust
let weight = (favouriter_count as f64 / 5.0).min(1.0);

for c in &mut candidates {
    let collab = c.score;      // Jaccard similarity
    let tag = c.tag_score;     // Tag-based score
    c.score = collab * weight + tag * (1.0 - weight);
}
```

The `weight` is based on how many favouriters the seed has:
- 0 favouriters → weight = 0 (100% tag-based)
- 5+ favouriters → weight = 1 (100% collaborative)

This smoothly transitions from tag-based to collaborative filtering as more data becomes available.

### Step 5: Community Vote Boost

Apply boosts from community votes:

```rust
let gamma = config.rec_voting_boost_gamma;  // Default: 0.2

for c in &mut candidates {
    let nv = net_votes.get(&c.url_id).copied().unwrap_or(0);
    // Logarithmic boost: 1 vote adds small boost, more votes add diminishing returns
    let boost = 1.0 + gamma * (nv.max(0) as f64).ln_1p();
    c.score *= boost;
}
```

The logarithmic scale prevents vote manipulation:
- 0 votes → boost = 1.0 (no change)
- 1 vote → boost ≈ 1.14
- 5 votes → boost ≈ 1.32
- 10 votes → boost ≈ 1.46
- 100 votes → boost ≈ 1.92

### Step 6: Sort and Return

```rust
candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
let top: Vec<CandidateScore> = candidates.into_iter().take(limit).collect();
```

## The Collection Worker

### How Data Is Collected

The collection worker scrapes user favourites from fanfiction sites:

```rust
pub struct CollectionWorker {
    db: PgPool,
    redis: Mutex<MultiplexedConnection>,
    client: Client,
    config: Config,
    fetchers: Vec<Box<dyn SiteFetcher>>,
    rate_limiters: HashMap<String, PerSiteRateLimiter>,
}
```

### The SiteFetcher Trait

Each site needs its own fetcher:

```rust
#[async_trait]
pub trait SiteFetcher: Send + Sync {
    fn site_domain(&self) -> &str;

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

### The Collection Process

When a new story is requested, it's enqueued for collection:

```rust
pub async fn enqueue(
    &self, url_id: &str, site_domain: &str, site_work_id: &str,
) -> Result<(), redis::RedisError> {
    let item = QueueItem {
        url_id: url_id.to_string(),
        site_domain: site_domain.to_string(),
        site_work_id: site_work_id.to_string(),
    };
    let json = serde_json::to_string(&item)?;
    let key = format!("collection_queue:{}", site_domain);
    let mut conn = self.redis.lock().await;
    redis::cmd("LPUSH").arg(&[key.as_str(), json.as_str()])
        .query_async(&mut *conn).await
}
```

### Processing a Work

The worker processes each work in the queue:

```rust
async fn process_work(&self, item: QueueItem) -> Result<(), Box<dyn std::error::Error>> {
    let fetcher = self.fetchers.iter()
        .find(|f| f.site_domain() == item.site_domain)
        .ok_or_else(|| format!("No fetcher for {}", item.site_domain))?;

    // 1. Fetch favouriters
    let favouriters = fetcher.collect_favouriters(&self.client, &work_url, max_pages).await?;

    for user_url in &favouriters {
        let user_hash = fetcher.user_hash(user_url);

        // 2. Skip if already recorded
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM fic_bookmarks WHERE user_hash = $1 AND url_id = $2)"
        ).bind(&user_hash).bind(&item.url_id)
         .fetch_one(&self.db).await.unwrap_or(false);

        if exists { continue; }

        // 3. Record the bookmark
        sqlx::query(
            "INSERT INTO fic_bookmarks (user_hash, url_id, site_domain)
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
        ).bind(&user_hash).bind(&item.url_id).bind(&item.site_domain)
         .execute(&self.db).await?;

        // 4. Fetch user's other favourites
        let user_favs = fetcher.collect_user_favourites(&self.client, user_url, max_pages).await?;

        // 5. Update co-occurrence
        for fav_url_id in &user_favs {
            if fav_url_id != &item.url_id {
                sqlx::query(
                    r#"INSERT INTO fic_bookmark_cooccur (work_a, work_b, cooccur_count)
                       VALUES ($1, $2, 1)
                       ON CONFLICT (work_a, work_b)
                       DO UPDATE SET cooccur_count = fic_bookmark_cooccur.cooccur_count + 1"#,
                ).bind(&item.url_id).bind(fav_url_id).execute(&self.db).await?;
            }
        }
    }

    Ok(())
}
```

### Rate Limiting the Worker

The worker respects per-site rate limits:

```rust
pub struct PerSiteRateLimiter {
    last_request: AtomicI64,
    delay_secs: u64,
}

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
```

The atomic CAS loop ensures only one thread can claim a time slot, even with multiple concurrent workers.

## Community Suggestions

### Submitting Suggestions

Users can manually suggest recommendations:

```rust
pub async fn suggest_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SuggestBody>,
) -> Result<Json<Value>, AppError> {
    // Resolve the suggested URL to a url_id
    let scraper = state.scraper_registry.find_scraper(&body.suggested_url)
        .ok_or_else(|| AppError::BadRequest(-5, "unsupported URL".into()))?;
    let meta = scraper.lookup(&state.http_client, &body.suggested_url).await?;
    let suggested_url_id = meta.url_id;

    // Validate both stories exist
    let seed_exists = queries::get_fic_info(&state.db, &body.url_id).await?.is_some();
    let suggestion_exists = queries::get_fic_info(&state.db, &suggested_url_id).await?.is_some();

    if !seed_exists {
        state.collection_worker.enqueue(&body.url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "seed fic not found"})));
    }

    if !suggestion_exists {
        state.collection_worker.enqueue(&suggested_url_id, "unknown", "unknown").await?;
        return Ok(Json(json!({"err": -5, "msg": "suggested fic not collected yet"})));
    }

    // Submit the suggestion
    let suggestion_id = submit_suggestion(
        &state.db, &body.url_id, &suggested_url_id, "0.0.0.0", body.comment.as_deref(),
    ).await?;

    Ok(Json(json!({"err": 0, "suggestion_id": suggestion_id})))
}
```

### Voting on Suggestions

Users can upvote or downvote suggestions:

```rust
pub async fn vote_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteBody>,
) -> Result<Json<Value>, AppError> {
    if body.vote != 1 && body.vote != -1 {
        return Ok(Json(json!({"err": -1, "msg": "vote must be 1 or -1"})));
    }

    let new_score = cast_vote(
        &state.db, body.suggestion_id, "0.0.0.0", body.vote as i16,
    ).await?;

    Ok(Json(json!({"err": 0, "new_score": new_score})))
}
```

### Community Scores

The recommendation engine incorporates community votes:

```rust
async fn get_community_scores(db: &PgPool) -> Result<HashMap<String, i32>, AppError> {
    let rows: Vec<(String, Option<i32>)> = sqlx::query_as(
        r#"SELECT s.suggested_url_id, SUM(v.vote)::INT AS net_votes
           FROM recommendation_suggestions s
           JOIN recommendation_votes v ON v.suggestion_id = s.id
           GROUP BY s.suggested_url_id"#,
    ).fetch_all(db).await?;

    Ok(rows.into_iter()
        .filter_map(|(url_id, score)| score.map(|s| (url_id, s)))
        .collect())
}
```

## Precomputed Recommendations

For popular stories, FicHub precomputes recommendations:

```rust
pub async fn compute_and_cache(
    &self, url_id: &str, config: &Config,
) -> Result<(), AppError> {
    let query = RecQuery {
        url_id: url_id.to_string(),
        n: config.rec_max_recommendations,
        site_domain: None,
    };

    let results = compute_live(&self.db, &query, config, config.rec_max_recommendations).await?;

    // Clear old cache entries
    sqlx::query("DELETE FROM precomputed_recommendations WHERE url_id = $1")
        .bind(url_id).execute(&self.db).await?;

    // Insert new entries
    for (rank, result) in results.iter().enumerate() {
        sqlx::query(
            r#"INSERT INTO precomputed_recommendations
                   (url_id, recommended_url_id, score, rank, computed_at)
               VALUES ($1, $2, $3, $4, NOW())"#,
        ).bind(url_id).bind(&result.url_id).bind(result.score as f32)
         .bind(rank as i16).execute(&self.db).await?;
    }

    Ok(())
}
```

### Cache Lookup

When a recommendation request comes in, FicHub first checks the precomputed cache:

```rust
async fn check_cache(
    db: &PgPool, query: &RecQuery, config: &Config, limit: usize,
) -> Result<Vec<RecResult>, AppError> {
    let rows: Vec<CachedRow> = sqlx::query_as::<_, CachedRow>(
        r#"SELECT pr.recommended_url_id, pr.score,
                  fi.title, fi.author, fi.words, fi.chapters,
                  fi.status, fi.source, fi.description
           FROM precomputed_recommendations pr
           JOIN fic_info fi ON fi.id = pr.recommended_url_id
           WHERE pr.url_id = $1
             AND pr.computed_at > NOW() - ($2 * INTERVAL '1 hour')
           ORDER BY pr.rank ASC
           LIMIT $3"#,
    ).bind(&query.url_id).bind(config.rec_cache_ttl_hours as i32)
     .bind(limit as i64).fetch_all(db).await?;

    // Enrich with community scores
    let community = get_community_scores(db).await?;

    let results = rows.into_iter().map(|r| {
        let cs = community.get(&r.recommended_url_id).copied().unwrap_or(0);
        RecResult {
            url_id: r.recommended_url_id,
            title: r.title,
            author: r.author,
            words: r.words,
            chapters: r.chapters,
            status: r.status,
            site_domain: r.source,
            summary: r.description,
            score: r.score as f64,
            community_score: cs,
            download_urls: HashMap::new(),
        }
    }).collect();

    Ok(results)
}
```

### Cache TTL

Precomputed recommendations expire after `rec_cache_ttl_hours` (default 12 hours). This ensures recommendations stay fresh while avoiding constant recomputation.

## Configuration

The recommendation engine is highly configurable:

```rust
pub rec_default_delay_secs: u64,           // Delay between scraping (default: 5)
pub rec_site_rate_limits: HashMap<String, u64>,  // Per-site delays
pub rec_max_favourite_pages: u32,          // Max pages to scrape (default: 3)
pub rec_max_user_favourite_pages: u32,     // Max pages per user (default: 3)
pub rec_max_recommendations: usize,        // Max results (default: 20)
pub rec_min_favouriters_for_collab: u32,   // Min for collaborative (default: 5)
pub rec_voting_boost_gamma: f64,           // Vote boost factor (default: 0.2)
pub rec_cache_ttl_hours: u32,              // Cache TTL (default: 12)
pub rec_suggest_limit_per_hour: u32,       // Suggestion rate limit (default: 5)
pub rec_vote_limit_per_hour: u32,          // Vote rate limit (default: 10)
pub rec_precompute_enabled: bool,          // Enable precomputation (default: true)
pub rec_precompute_interval_hours: u32,    // Precompute interval (default: 6)
pub rec_enable_cross_site: bool,           // Cross-site recommendations (default: true)
```

## Performance Considerations

### Database Indexes

```sql
CREATE INDEX idx_fic_bookmarks_user ON fic_bookmarks(user_hash);
CREATE INDEX idx_fic_bookmarks_work ON fic_bookmarks(url_id);
CREATE INDEX idx_fic_cooccur_works ON fic_bookmark_cooccur(work_a, work_b);
CREATE INDEX idx_precomputed_url ON precomputed_recommendations(url_id, computed_at);
```

### Query Optimization

The co-occurrence query is the most expensive operation. Optimizations:
1. Limit the number of candidates (`LIMIT $2`)
2. Use CTEs for readability and potential optimization
3. Index on `(work_a, work_b)` for fast lookups

### Caching Strategy

1. **Precomputed cache** — For popular stories (checked first)
2. **Live computation** — For stories without cached recommendations
3. **Database cache** — Export logs prevent redundant generation

## Evaluation Metrics

To measure recommendation quality:

1. **Precision** — How many recommendations are relevant?
2. **Recall** — How many relevant items were recommended?
3. **Diversity** — Are recommendations from different sources?
4. **Novelty** — Are recommendations surprising (not obvious)?
5. **Coverage** — What percentage of items can be recommended?

FicHub doesn't currently compute these metrics, but they could be added for evaluation.


# Deep Dive: OPDS Catalog System

## Understanding OPDS

OPDS (Open Publication Distribution System) is a protocol that allows e-readers to browse and download books from online catalogs. It's based on Atom feeds (a web syndication format) with specific extensions for book metadata.

### Why OPDS?

OPDS solves several problems:
1. **Discovery** — Users can find books without knowing specific URLs
2. **Browsing** — Categories, tags, and search are built-in
3. **Download** — Direct EPUB download to the e-reader
4. **Sync** — Progress and bookmarks can be synced across devices
5. **Offline** — Books can be downloaded for offline reading

### OPDS vs Other Formats

| Feature | OPDS | RSS | JSON API |
|---------|------|-----|----------|
| E-reader support | Excellent | Poor | None |
| Browsing | Built-in | Limited | Custom |
| Download | Direct | Links | Custom |
| Standard | Yes | Yes | No |

## OPDS Feed Structure

### Atom Feed Base

Every OPDS feed is an Atom feed with OPDS extensions:

```xml
<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom"
      xmlns:dc="http://purl.org/dc/terms/"
      xmlns:opds="http://opds-spec.org/2010/catalog"
      xmlns:pse="http://opds-spec.org/2010/partial">
  <id>urn:fichub:opds:root</id>
  <title>FicHub Catalog</title>
  <updated>2024-01-15T10:30:00Z</updated>
  <author><name>FicHub</name></author>
  
  <!-- Links -->
  <link href="/opds" rel="self" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  <link href="/opds" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  
  <!-- Entries -->
  <entry>
    <title>New Stories</title>
    <id>urn:fichub:opds:new</id>
    <updated>2024-01-15T10:30:00Z</updated>
    <content type="text">Recently added fanfiction</content>
    <link href="/opds/new" rel="subsection" type="application/atom+xml"/>
  </entry>
</feed>
```

### Navigation Feeds

Navigation feeds list categories (like a table of contents):

```xml
<feed>
  <title>FicHub - New Stories</title>
  <link href="/opds/new" rel="self" type="application/atom+xml;profile=opds-catalog;kind=acquisition"/>
  <link href="/opds" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>
  
  <entry>
    <title>A Great Story</title>
    <author><name>Author Name</name></author>
    <id>urn:fichub:fic:abc123</id>
    <updated>2024-01-15T10:30:00Z</updated>
    <summary>A wonderful fanfiction about...</summary>
    <dc:extent>50000</dc:extent>
    <dc:format>application/epub+zip</dc:format>
    <category term="complete" label="Complete"/>
    <category term="chapters:10" label="10 chapters"/>
    
    <!-- Download links -->
    <link rel="http://opds-spec.org/acquisition"
          href="/cache/epub/abc123/abc123.epub"
          type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition"
          href="/cache/pdf/abc123/abc123.pdf"
          type="application/pdf"/>
    
    <!-- Cover image -->
    <link rel="http://opds-spec.org/image"
          href="/cache/cover/abc123.jpg"
          type="image/jpeg"/>
    <link rel="http://opds-spec.org/image/thumbnail"
          href="/cache/cover/abc123_thumb.jpg"
          type="image/jpeg"/>
    
    <!-- HTML link -->
    <link rel="alternate"
          href="/fics/abc123"
          type="text/html"/>
  </entry>
</feed>
```

### Key Elements

- **`<title>`** — Story title
- **`<author>`** — Author name
- **`<id>`** — Unique identifier (URN format)
- **`<updated>`** — Last update time (RFC3339 format)
- **`<summary>`** — Story description
- **`<dc:extent>`** — Word count
- **`<dc:format>`** — File format
- **`<category>`** — Tags/categories
- **`<link rel="acquisition">`** — Download links
- **`<link rel="image">`** — Cover images

## FicHub's OPDS Implementation

### The Root Catalog

```rust
pub async fn root_catalog() -> impl IntoResponse {
    let entries = format!(
        r#"<entry>
    <title>New Stories</title>
    <id>urn:fichub:opds:new</id>
    <updated>{updated}</updated>
    <content type="text">Recently added fanfiction</content>
    <link href="/opds/new" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Popular Stories</title>
    <id>urn:fichub:opds:popular</id>
    <updated>{updated}</updated>
    <content type="text">Most popular fanfiction</content>
    <link href="/opds/popular" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Browse by Tag</title>
    <id>urn:fichub:opds:tags</id>
    <updated>{updated}</updated>
    <content type="text">Browse stories by tag</content>
    <link href="/opds/tags" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Authors</title>
    <id>urn:fichub:opds:authors</id>
    <updated>{updated}</updated>
    <content type="text">Browse stories by author</content>
    <link href="/opds/authors" rel="subsection" type="application/atom+xml"/>
</entry>
<entry>
    <title>Search</title>
    <id>urn:fichub:opds:search</id>
    <updated>{updated}</updated>
    <content type="text">Search for stories</content>
    <link href="/opds/search" rel="subsection" type="application/atom+xml"/>
</entry>"#,
        updated = iso_now(),
    );

    let body = build_feed(
        "FicHub Catalog",
        "urn:fichub:opds:root",
        &entries,
        &iso_now(),
        Some("/opds"),
        FeedKind::Navigation,
        None,
    );

    opds_response(body, FeedKind::Navigation)
}
```

### Story Entry Builder

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
    <category term="chapters:{chapters}" label="{chapters} chapters"/>
    <link rel="http://opds-spec.org/image"
          href="/cache/cover/{url_id}.jpg"
          type="image/jpeg"/>
    <link rel="http://opds-spec.org/image/thumbnail"
          href="/cache/cover/{url_id}_thumb.jpg"
          type="image/jpeg"/>
    <link rel="http://opds-spec.org/acquisition"
          href="/cache/epub/{url_id}/{url_id}.epub"
          type="application/epub+zip"/>
    <link rel="http://opds-spec.org/acquisition"
          href="/cache/pdf/{url_id}/{url_id}.pdf"
          type="application/pdf"/>
    <link rel="alternate"
          href="/fics/{url_id}"
          type="text/html"/>
  </entry>"#,
        title = html_escape(title),
        author = html_escape(author),
        summary = html_escape(summary),
        url_id = url_id,
        updated = updated,
        words = words,
        chapters = chapters,
        status = status,
    )
}
```

### Feed Builder

```rust
pub fn build_feed(
    title: &str, feed_id: &str, entries: &str, updated: &str,
    self_link: Option<&str>, feed_kind: FeedKind,
    pagination: Option<&PaginationInfo>,
) -> String {
    let mut links = String::new();

    if let Some(link) = self_link {
        links.push_str(&format!(
            r#"  <link href="{link}" rel="self" type="{ct}"/>"#,
            link = html_escape(link),
            ct = feed_kind.content_type(),
        ));
    }

    links.push_str(&format!(
        r#"  <link href="/opds" rel="start" type="application/atom+xml;profile=opds-catalog;kind=navigation"/>"#,
    ));

    if let Some(ref pagi) = pagination {
        let total_pages = (pagi.total as f64 / pagi.per_page as f64).ceil() as usize;
        if pagi.page > 1 {
            let prev_href = format!("{}?page={}&per_page={}",
                pagi.base_path, pagi.page - 1, pagi.per_page);
            links.push_str(&format!(
                r#"  <link href="{prev}" rel="previous" type="{ct}"/>"#,
                prev = html_escape(&prev_href),
                ct = feed_kind.content_type(),
            ));
        }
        if pagi.page < total_pages {
            let next_href = format!("{}?page={}&per_page={}",
                pagi.base_path, pagi.page + 1, pagi.per_page);
            links.push_str(&format!(
                r#"  <link href="{next}" rel="next" type="{ct}"/>"#,
                next = html_escape(&next_href),
                ct = feed_kind.content_type(),
            ));
        }
    }

    format!(
        r#"{header}<id>{id}</id>
  <title>{title}</title>
  <updated>{updated}</updated>
  <author><name>FicHub</name></author>
{links}{entries}</feed>"#,
        header = atom_xml_header(),
        id = html_escape(feed_id),
        title = html_escape(title),
        updated = updated,
        links = links,
        entries = entries,
    )
}
```

## OPDS Features in FicHub

### Recently Updated Feed

```rust
pub async fn recent_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();

    let fics: Vec<FicInfo> = sqlx::query_as(
        "SELECT * FROM fic_info ORDER BY fic_updated DESC LIMIT $1 OFFSET $2"
    )
    .bind(per_page as i64).bind(offset as i64)
    .fetch_all(&state.db).await.unwrap_or_default();

    let entries: String = fics.iter().map(|fic| {
        fic_entry(
            &fic.id, &fic.title, &fic.author, &fic.description,
            &fic.fic_updated.to_rfc3339(), fic.words, fic.chapters, &fic.status,
        )
    }).collect();

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fic_info")
        .fetch_one(&state.db).await.unwrap_or(0);

    let body = build_feed(
        "Recently Updated",
        "urn:fichub:opds:new",
        &entries,
        &iso_now(),
        Some("/opds/new"),
        FeedKind::Acquisition,
        Some(&PaginationInfo {
            base_path: "/opds/new".to_string(),
            page, per_page, total,
        }),
    );

    opds_response(body, FeedKind::Acquisition)
}
```

### Popular Feed

```rust
pub async fn popular_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PageParams>,
) -> impl IntoResponse {
    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();

    // Get stories sorted by download count
    let fics: Vec<FicInfo> = sqlx::query_as(
        r#"SELECT fi.* FROM fic_info fi
           LEFT JOIN request_log rl ON rl.url_id = fi.id
           GROUP BY fi.id
           ORDER BY COUNT(rl.id) DESC
           LIMIT $1 OFFSET $2"#,
    )
    .bind(per_page as i64).bind(offset as i64)
    .fetch_all(&state.db).await.unwrap_or_default();

    // ... build feed ...
}
```

### Tag Browsing

```rust
pub async fn tag_types(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let types: Vec<(i16, String)> = sqlx::query_as(
        "SELECT DISTINCT tag_type_id, 
         CASE tag_type_id
           WHEN 1 THEN 'Fandom'
           WHEN 2 THEN 'Character'
           WHEN 3 THEN 'Relationship'
           WHEN 4 THEN 'Freeform'
           WHEN 5 THEN 'Warning'
           WHEN 6 THEN 'Category'
         END as type_name
         FROM tags ORDER BY tag_type_id"
    ).fetch_all(&state.db).await.unwrap_or_default();

    let entries: String = types.iter().map(|(id, name)| {
        format!(
            r#"<entry>
    <title>{name}</title>
    <id>urn:fichub:opds:tags:{id}</id>
    <updated>{updated}</updated>
    <content type="text">Browse {name} tags</content>
    <link href="/opds/tags/{id}" rel="subsection" type="application/atom+xml"/>
</entry>"#,
            name = html_escape(name),
            id = id,
            updated = iso_now(),
        )
    }).collect();

    let body = build_feed(
        "Browse by Tag Type",
        "urn:fichub:opds:tags",
        &entries,
        &iso_now(),
        Some("/opds/tags"),
        FeedKind::Navigation,
        None,
    );

    opds_response(body, FeedKind::Navigation)
}
```

### Search

```rust
pub async fn search_feed(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let query = params.q.unwrap_or_default();
    if query.is_empty() {
        return opds_response(
            build_feed("Search", "urn:fichub:opds:search", "", &iso_now(), None, FeedKind::Acquisition, None),
            FeedKind::Acquisition,
        );
    }

    let fics: Vec<FicInfo> = sqlx::query_as(
        r#"SELECT * FROM fic_info
           WHERE text_search @@ plainto_tsquery('english', $1)
           ORDER BY ts_rank(text_search, plainto_tsquery('english', $1)) DESC
           LIMIT 50"#,
    ).bind(&query).fetch_all(&state.db).await.unwrap_or_default();

    let entries: String = fics.iter().map(|fic| {
        fic_entry(/* ... */)
    }).collect();

    let body = build_feed(
        &format!("Search: {}", query),
        &format!("urn:fichub:opds:search:{}", query),
        &entries,
        &iso_now(),
        Some(&format!("/opds/search?q={}", urlencoding::encode(&query))),
        FeedKind::Acquisition,
        None,
    );

    opds_response(body, FeedKind::Acquisition)
}
```

## OPDS Authentication

### Shelf Authentication

FicHub supports authenticated shelves:

```rust
pub async fn shelf_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ShelfQuery>,
) -> impl IntoResponse {
    let token = params.token.as_deref().unwrap_or("");
    if token != state.config.opds_shelf_token {
        return opds_response(
            build_feed("Access Denied", "urn:fichub:opds:denied",
                "<entry><title>Invalid Token</title></entry>",
                &iso_now(), None, FeedKind::Navigation, None),
            FeedKind::Navigation,
        );
    }

    // Return shelf contents
    let fics: Vec<FicInfo> = sqlx::query_as(
        "SELECT * FROM fic_info ORDER BY title ASC LIMIT 100"
    ).fetch_all(&state.db).await.unwrap_or_default();

    // ... build feed ...
}
```

## OPDS Client Compatibility

### Supported Clients

FicHub's OPDS catalog works with:
- **Calibre** — Desktop e-book management
- **KOReader** — E-reader firmware
- **Apple Books** — iOS/macOS
- **FBReader** — Cross-platform reader
- **Mantano Reader** — Android

### Testing with Calibre

1. Open Calibre
2. Click "Connect/share" → "Start Content server"
3. Add a new OPDS catalog: `http://localhost:3000/opds`
4. Browse and download stories

### Testing with KOReader

1. Go to OPDS catalog settings
2. Add catalog URL: `http://your-server:3000/opds`
3. Browse and download

## Performance Considerations

### Pagination

OPDS feeds should be paginated to avoid large responses:

```rust
pub struct PageParams {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

impl PageParams {
    pub fn page(&self) -> usize { self.page.unwrap_or(1).max(1) }
    pub fn per_page(&self) -> usize { self.per_page.unwrap_or(50).max(1).min(100) }
    pub fn offset(&self) -> usize { (self.page() - 1) * self.per_page() }
}
```

### Caching

OPDS feeds can be cached:

```rust
// Add Cache-Control header
(
    [("Cache-Control", "public, max-age=300")],  // 5 minutes
    body,
)
```

### Compression

Enable gzip compression for OPDS feeds:

```rust
.layer(CompressionLayer::new().gzip(true))
```

## Advanced OPDS Features

### Partial Content

OPDS supports partial content for large books:

```xml
<link rel="http://opds-spec.org/indirect"
      href="/opds/partial/abc123"
      type="application/atom+xml"/>
```

### Acquisition by Price

OPDS can indicate pricing:

```xml
<link rel="http://opds-spec.org/acquisition"
      href="/buy/abc123"
      type="application/epub+zip">
  <opds:price currency="USD">0.00</opds:price>
</link>
```

### Open Search

OPDS supports OpenSearch for discoverable search:

```xml
<link rel="search"
      type="application/opensearchdescription+xml"
      href="/opds/search.xml"
      title="Search FicHub"/>
```

## Conclusion

OPDS provides a standardized way for e-readers to access FicHub's library. By implementing OPDS, FicHub integrates seamlessly with the existing e-reader ecosystem, allowing users to browse, search, and download stories directly to their devices.


# Deep Dive: Advanced Rust Patterns

## Error Handling Patterns

### The Builder Pattern with Error Handling

FicHub uses builders that can fail:

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

Each builder method returns `Self`, and `build()` returns `Result<Client, Error>`. The `.expect()` unwraps the result, panicking if it fails.

### Custom Error Conversion

FicHub implements `From` for automatic error conversion:

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
```

This enables the `?` operator to automatically convert errors:

```rust
let data = tokio::fs::read_to_string("config.json").await?;  // io::Error → AppError
let row = sqlx::query("SELECT...").fetch_one(&pool).await?;    // sqlx::Error → AppError
```

### Error Context

For better error messages, use `.context()`:

```rust
use anyhow::Context;

let config = std::fs::read_to_string("config.json")
    .context("Failed to read config file")?;
```

### The Map-Err Pattern

Transform errors while preserving the type:

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

## Concurrency Patterns

### The Semaphore Pattern

Prevent concurrent access to a resource:

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(1));  // Only 1 concurrent task

let permit = semaphore.acquire().await?;
// ... do work ...
drop(permit);  // Release the semaphore
```

FicHub uses this to prevent duplicate EPUB generation:

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;
```

### The Mutex Pattern

Protect shared mutable state:

```rust
use tokio::sync::Mutex;

let data = Arc::new(Mutex::new(HashMap::new()));

// In one task
let mut map = data.lock().await;
map.insert("key", "value");

// In another task
let mut map = data.lock().await;
let value = map.get("key");
```

### The RwLock Pattern

Allow concurrent reads, exclusive writes:

```rust
use tokio::sync::RwLock;

let data = Arc::new(RwLock::new(HashSet::new()));

// Multiple readers can access simultaneously
let ips = data.read().await;
if ips.contains(&ip) { /* ... */ }
drop(ips);

// Only one writer at a time
let mut ips = data.write().await;
ips.insert(new_ip);
```

### The Atomic CAS Pattern

Lock-free concurrent updates:

```rust
use std::sync::atomic::{AtomicI64, Ordering};

let counter = AtomicI64::new(0);

// Atomic increment
counter.fetch_add(1, Ordering::Relaxed);

// Compare-and-swap loop
loop {
    let current = counter.load(Ordering::Acquire);
    let new_value = current + 1;
    if counter.compare_exchange(
        current, new_value, Ordering::AcqRel, Ordering::Relaxed
    ).is_ok() {
        break;  // Successfully updated
    }
    // CAS failed — retry
}
```

FicHub uses this for rate limiting:

```rust
pub async fn wait_if_needed(&self) {
    loop {
        let now = current_time();
        let last = self.last_request.load(Ordering::Acquire);
        let elapsed = now - last;

        if elapsed >= self.delay_nanos {
            if self.last_request.compare_exchange(
                last, now, Ordering::AcqRel, Ordering::Relaxed
            ).is_ok() {
                return;  // Successfully claimed the slot
            }
            // CAS failed — retry
        } else {
            sleep(remaining_time).await;
        }
    }
}
```

## Async Patterns

### Join Multiple Futures

Run multiple async operations concurrently:

```rust
use futures::future::join_all;

let urls = vec!["url1", "url2", "url3"];
let results = join_all(urls.iter().map(|url| fetch(url))).await;
```

### Select Multiple Futures

Wait for the first future to complete:

```rust
use tokio::select;

tokio::select! {
    result = future1 => { /* future1 completed */ }
    result = future2 => { /* future2 completed */ }
}
```

### Spawn Background Tasks

Run tasks in the background:

```rust
tokio::spawn(async move {
    // Long-running task
    loop {
        process_queue_item().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
});
```

### Channel Communication

Send messages between tasks:

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(32);

// Sender task
tokio::spawn(async move {
    tx.send("hello").await.unwrap();
});

// Receiver task
while let Some(message) = rx.recv().await {
    println!("Received: {}", message);
}
```

## Ownership Patterns

### Clone on Write

Clone data only when you need to modify it:

```rust
fn process(config: Config) {
    // Use config directly (no clone)
    println!("{}", config.database_url);
}

fn process_and_modify(config: Config) {
    let mut config = config;  // Take ownership
    config.app_port = 8080;   // Modify
    // Use modified config
}
```

### Arc for Shared Ownership

Share data across tasks:

```rust
let state = Arc::new(AppState { /* ... */ });

for _ in 0..10 {
    let state = state.clone();  // Clone the Arc, not the data
    tokio::spawn(async move {
        // Use state
    });
}
```

### Ref for Borrowing

Borrow data without taking ownership:

```rust
fn process_fic(fic: &FicInfo) {
    println!("{}", fic.title);  // Borrow title
    // fic is still valid after this function
}
```

## Trait Patterns

### Default Implementations

Provide default behavior that can be overridden:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
    
    // Default implementation
    async fn extract_tags(&self, _client: &reqwest::Client, _url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
        Ok(Vec::new())
    }
}
```

### Trait Objects

Store different types in the same collection:

```rust
let scrapers: Vec<Box<dyn SiteScraper>> = vec![
    Box::new(Ao3Scraper),
    Box::new(FfNetScraper),
    Box::new(XenForoScraper),
];

for scraper in &scrapers {
    if scraper.can_handle(url) {
        return scraper.lookup(client, url).await;
    }
}
```

### Static Dispatch vs Dynamic Dispatch

```rust
// Static dispatch (monomorphized — one function per type)
fn process<T: SiteScraper>(scraper: &T, url: &str) {
    scraper.lookup(client, url).await;
}

// Dynamic dispatch (virtual dispatch — runtime lookup)
fn process(scraper: &dyn SiteScraper, url: &str) {
    scraper.lookup(client, url).await;
}
```

Static dispatch is faster (no runtime overhead) but generates more code. Dynamic dispatch is slower but more flexible.

## Memory Management

### Stack vs Heap

- **Stack** — Fast, automatic cleanup, limited size (usually 1-8MB)
- **Heap** — Slower, manual management, unlimited size

Most data lives on the stack. `Box`, `String`, `Vec` use heap allocation.

### Smart Pointers

| Pointer | Purpose | Overhead |
|---------|---------|----------|
| `Box<T>` | Heap allocation | Minimal |
| `Rc<T>` | Single-threaded shared ownership | Reference counting |
| `Arc<T>` | Multi-threaded shared ownership | Atomic reference counting |
| `Mutex<T>` | Interior mutability | Lock overhead |
| `RwLock<T>` | Multiple readers OR one writer | Lock overhead |

### Memory Leaks

Rust prevents most memory leaks, but some are possible:
- `Rc`/`Arc` cycles (reference counting loops)
- `mem::forget` (explicit leak)
- Threads that never terminate

FicHub avoids these by:
- Using `Arc` carefully (no cycles)
- Not using `mem::forget`
- Ensuring background tasks can be stopped

## Performance Optimization

### Zero-Cost Abstractions

Rust's abstractions compile down to the same code as manual implementations:

```rust
// This iterator chain
let result: Vec<String> = fics.iter()
    .filter(|fic| fic.words > 10_000)
    .map(|fic| fic.title.clone())
    .collect();

// Compiles to something like:
let mut result = Vec::new();
for fic in &fics {
    if fic.words > 10_000 {
        result.push(fic.title.clone());
    }
}
```

### Avoid Unnecessary Allocations

```rust
// Bad: Creates a new String every iteration
for i in 0..100 {
    let s = format!("item_{}", i);
    // ...
}

// Better: Reuse a buffer
let mut s = String::new();
for i in 0..100 {
    s.clear();
    write!(&mut s, "item_{}", i).unwrap();
    // ...
}
```

### Use References When Possible

```rust
// Bad: Clones the string
fn process(title: String) { /* ... */ }

// Better: Borrows the string
fn process(title: &str) { /* ... */ }
```

### Pre-allocate Collections

```rust
// Bad: Grows dynamically
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// Better: Pre-allocates
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

## Testing Patterns

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_url_id() {
        let id = generate_url_id(1, "story_123");
        assert_eq!(id.len(), 12);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_url_id_deterministic() {
        let id1 = generate_url_id(1, "story_123");
        let id2 = generate_url_id(1, "story_123");
        assert_eq!(id1, id2);
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let pool = db::init_pool("postgres://localhost/test_db").await.unwrap();
        let result = sqlx::query("SELECT 1").fetch_one(&pool).await;
        assert!(result.is_ok());
    }
}
```

### Test Fixtures

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
```

## Code Organization

### Module Structure

FicHub uses a clean module structure:

```
src/
├── main.rs          # Entry point
├── lib.rs           # Library root
├── config.rs        # Configuration
├── server.rs        # Server setup
├── error.rs         # Error types
├── db/              # Database
│   ├── mod.rs
│   ├── models.rs
│   └── queries.rs
├── scrape/          # Scrapers
│   ├── mod.rs
│   ├── registry.rs
│   └── sites/
│       ├── mod.rs
│       ├── ao3.rs
│       ├── ffnet.rs
│       └── ...
├── export/          # File generation
│   ├── mod.rs
│   ├── epub.rs
│   └── html_bundle.rs
├── cache/           # Caching
│   ├── mod.rs
│   └── disk.rs
├── limiter/         # Rate limiting
│   ├── mod.rs
│   └── redis_bucket.rs
├── tags/            # Tagging system
│   ├── mod.rs
│   ├── resolve.rs
│   ├── voting.rs
│   ├── routes.rs
│   └── curator.rs
├── search/          # Search engine
│   ├── mod.rs
│   ├── builder.rs
│   └── routes.rs
├── recommender/     # Recommendations
│   ├── mod.rs
│   ├── engine.rs
│   ├── worker.rs
│   └── routes.rs
└── routes/          # API handlers
    ├── mod.rs
    ├── export.rs
    ├── meta.rs
    ├── cache_download.rs
    ├── api_docs.rs
    └── opds/
        ├── mod.rs
        ├── feeds.rs
        ├── tags.rs
        ├── authors.rs
        ├── recommendations.rs
        ├── search.rs
        └── shelves.rs
```

### Separation of Concerns

Each module has a single responsibility:
- `db/` — Only database operations
- `scrape/` — Only web scraping
- `export/` — Only file generation
- `cache/` — Only caching logic
- `limiter/` — Only rate limiting
- `tags/` — Only tag operations
- `search/` — Only search functionality
- `recommender/` — Only recommendations
- `routes/` — Only HTTP handlers

This makes the code easier to understand, test, and modify.

## Conclusion

Rust's ownership system, pattern matching, and async/await provide powerful tools for building reliable, performant systems. FicHub demonstrates how these features work together to create a production-ready backend.


# Deep Dive: Security and Best Practices

## Security Considerations

### SQL Injection Prevention

FicHub uses parameterized queries everywhere:

```rust
// SAFE: Parameterized query
sqlx::query("SELECT * FROM fic_info WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;

// DANGEROUS: String interpolation (NEVER DO THIS!)
// sqlx::query(&format!("SELECT * FROM fic_info WHERE id = '{}'", id))
```

The `$1`, `$2` etc. are placeholders that PostgreSQL handles safely. User input is never interpolated into SQL strings.

### XSS Prevention

FicHub escapes HTML in responses:

```rust
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace(''', "&#39;")
}
```

This prevents cross-site scripting attacks where malicious HTML could execute JavaScript.

### Rate Limiting

Rate limiting prevents abuse:

```rust
async fn check_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
    max_per_hour: u32,
) -> AppResult<()> {
    let count: Option<u32> = redis.get(key).await.unwrap_or(None);
    if count.unwrap_or(0) >= max_per_hour {
        return Err(AppError::RateLimited(3600));
    }
    let _: () = redis.incr(key, 1).await.unwrap_or_default();
    let _: () = redis.expire(key, 3600).await.unwrap_or_default();
    Ok(())
}
```

### Input Validation

Validate all user input:

```rust
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}

if query.len() > 2048 {
    return Ok(Json(json!({"err": -1, "msg": "query too long"})));
}
```

### Authentication

FicHub uses token-based authentication for curator operations:

```rust
let token = body.token.as_deref()
    .ok_or_else(|| AppError::BadRequest(-1, "token required".into()))?;

if Some(token.to_string()) != state.config.curator_token {
    return Ok(Json(json!({"err": -3, "msg": "invalid curator token"})));
}
```

### Secrets Management

Never commit secrets to version control:

```gitignore
# .gitignore
.env
*.pem
*.key
secrets/
```

Use environment variables or Docker secrets for sensitive data.

## Performance Best Practices

### Connection Pooling

Use connection pools for databases:

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .acquire_timeout(Duration::from_secs(10))
    .connect(database_url)
    .await?;
```

### Caching

Cache expensive operations:

```rust
// Check cache first
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    return Ok(build_cached_response(&meta, &export_log));
}

// Generate only if not cached
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;
```

### Async Everything

Use async for I/O operations:

```rust
// BAD: Blocking I/O
let data = std::fs::read_to_string("file.txt").unwrap();

// GOOD: Async I/O
let data = tokio::fs::read_to_string("file.txt").await?;
```

### Avoid Unnecessary Allocations

```rust
// BAD: Creates new String every time
fn get_title(fic: &FicInfo) -> String {
    fic.title.clone()
}

// GOOD: Borrows existing string
fn get_title(fic: &FicInfo) -> &str {
    &fic.title
}
```

## Code Quality Best Practices

### Error Handling

Always handle errors explicitly:

```rust
// BAD: Panics on error
let data = std::fs::read_to_string("config.json").unwrap();

// GOOD: Handles error gracefully
let data = match std::fs::read_to_string("config.json") {
    Ok(data) => data,
    Err(e) => {
        tracing::warn!("Failed to read config: {}", e);
        return Err(AppError::Internal(e.to_string()));
    }
};
```

### Documentation

Document public APIs:

```rust
/// Upsert a fic_info record (INSERT ON CONFLICT UPDATE)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `fic` - FicInfo record to upsert
///
/// # Returns
/// * `AppResult<()>` - Ok(()) on success, Err on failure
pub async fn upsert_fic_info(pool: &PgPool, fic: &FicInfo) -> AppResult<()> {
    // ...
}
```

### Testing

Write tests for critical functionality:

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_url_id_different_source() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);
}
```

### Code Reviews

Review code for:
- Security vulnerabilities
- Performance issues
- Error handling gaps
- Naming consistency
- Documentation completeness

## Deployment Best Practices

### Docker

Use multi-stage builds for small images:

```dockerfile
FROM rust:1.78 AS builder
# ... build ...

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/fichub /usr/local/bin/fichub
```

### Health Checks

Always implement health checks:

```rust
async fn health_handler() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
```

### Monitoring

Add metrics and logging:

```rust
tracing::info!("Request: {} {} {}", method, uri, status);
```

### Backups

Automate database backups:

```bash
#!/bin/bash
docker compose exec -T db pg_dump -U fichub fichub | gzip > backup_$(date +%Y%m%d).sql.gz
```

### Graceful Shutdown

Handle shutdown signals:

```rust
tokio::signal::ctrl_c().await?;
tracing::info!("Shutting down...");
// Clean up resources
```

## Common Pitfalls

### Deadlocks

Avoid holding multiple locks:

```rust
// BAD: Potential deadlock
let lock1 = mutex1.lock().await;
let lock2 = mutex2.lock().await;  // Might wait forever

// GOOD: Always lock in same order
let lock1 = mutex1.lock().await;
let lock2 = mutex2.lock().await;
```

### Memory Leaks

Be careful with `Arc` cycles:

```rust
// BAD: Reference cycle
let a = Arc::new(Mutex::new(Some(b.clone())));
let b = Arc::new(Mutex::new(Some(a.clone())));

// GOOD: Use weak references when needed
let weak_a = Arc::downgrade(&a);
```

### Unbounded Growth

Limit collection sizes:

```rust
// BAD: Unbounded
let mut vec = Vec::new();
loop {
    vec.push(data);  // Grows forever!
}

// GOOD: Bounded
let mut vec = Vec::with_capacity(1000);
loop {
    if vec.len() >= 1000 {
        vec.remove(0);  // Remove oldest
    }
    vec.push(data);
}
```

### Blocking Async

Never block in async code:

```rust
// BAD: Blocks the executor
async fn bad_handler() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks!
}

// GOOD: Uses async sleep
async fn good_handler() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // Non-blocking
}
```

## Conclusion

Building production-ready software requires attention to security, performance, and code quality. FicHub demonstrates these principles through:
- Parameterized queries for SQL injection prevention
- HTML escaping for XSS prevention
- Rate limiting for abuse prevention
- Connection pooling for performance
- Caching for expensive operations
- Comprehensive testing for reliability
- Proper error handling for robustness

By following these best practices, FicHub provides a secure, performant, and maintainable backend for fanfiction downloads.


# Deep Dive: Search Engine Implementation

## Full-Text Search in PostgreSQL

### How PostgreSQL Search Works

PostgreSQL's full-text search converts text into a special `tsvector` type:

```sql
-- Convert text to tsvector
SELECT to_tsvector('english', 'Harry Potter ran quickly through the castle');
-- Returns: 'castle':9 'harry':1 'potter':2 'quickli':7 'ran':4

-- The numbers are positions in the original text
-- Words are stemmed (running -> run, quickly -> quickli)
```

### Search Queries

```sql
-- Simple search
SELECT * FROM fic_info
WHERE text_search @@ plainto_tsquery('english', 'harry potter');

-- Ranked search
SELECT *, ts_rank(text_search, plainto_tsquery('english', 'harry potter')) AS rank
FROM fic_info
WHERE text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY rank DESC;

-- Weighted search (title matches rank higher)
SELECT *,
    ts_rank_cd(
        setweight(to_tsvector('english', title), 'A') ||
        setweight(to_tsvector('english', author), 'B') ||
        setweight(to_tsvector('english', description), 'C'),
        plainto_tsquery('english', 'harry potter')
    ) AS rank
FROM fic_info
ORDER BY rank DESC;
```

### GIN Index

For fast searches, create a GIN index:

```sql
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);
```

Without this index, every search scans every row (sequential scan). With the GIN index, searches are much faster (index scan).

## FicHub's Search Implementation

### The SearchQueryBuilder

```rust
pub struct SearchQueryBuilder {
    pub params: SearchParams,
    hidden_threshold: i16,
}

impl SearchQueryBuilder {
    pub fn new(params: SearchParams, hidden_threshold: i16) -> Self {
        Self { params, hidden_threshold }
    }

    pub fn build_data_query(&self) -> QueryBuilder<Postgres> {
        let has_q = self.params.q.is_some();

        let mut qb = QueryBuilder::<Postgres>::new("SELECT fi.*, ");

        // Add rank column
        if let Some(ref q) = self.params.q {
            qb.push("ts_rank(fi.text_search, plainto_tsquery('english', ");
            qb.push_bind(q);
            qb.push(")) AS rank FROM fic_info fi WHERE 1=1");
        } else {
            qb.push("NULL::real AS rank FROM fic_info fi WHERE 1=1");
        }

        // Add WHERE clauses
        self.push_where_clauses(&mut qb);

        // Add ORDER BY
        let sort = self.params.sort.as_deref().unwrap_or(
            if has_q { "-relevance" } else { "-date" },
        );
        qb.push(" ORDER BY ");
        match sort {
            "-relevance" => {
                if has_q { qb.push("rank DESC"); }
                else { qb.push("fi.fic_updated DESC"); }
            }
            "-date" => qb.push("fi.fic_updated DESC"),
            "-words" => qb.push("fi.words DESC"),
            "-chapters" => qb.push("fi.chapters DESC"),
            "-title" => qb.push("fi.title ASC"),
            _ => qb.push("fi.fic_updated DESC"),
        }

        // Add LIMIT/OFFSET
        let page = self.params.page.unwrap_or(1).max(1);
        let per_page = self.params.per_page.unwrap_or(20).max(1);
        let offset = (page - 1) * per_page;
        qb.push(" LIMIT ");
        qb.push_bind(per_page as i64);
        qb.push(" OFFSET ");
        qb.push_bind(offset as i64);

        qb
    }

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
            qb.push(")");
        }

        // Word count filters
        if let Some(min) = self.params.min_words {
            qb.push(" AND fi.words >= ");
            qb.push_bind(min);
        }
        if let Some(max) = self.params.max_words {
            qb.push(" AND fi.words <= ");
            qb.push_bind(max);
        }

        // Completion filter
        if let Some(true) = self.params.complete {
            qb.push(" AND fi.status = 'complete'");
        }

        // Source filter
        if let Some(ref source) = self.params.source {
            qb.push(" AND fi.source = ");
            qb.push_bind(source);
        }
    }
}
```

### Why Use QueryBuilder?

`sqlx::QueryBuilder` provides:
1. **SQL injection protection** — Parameters are automatically escaped
2. **Dynamic queries** — Add WHERE clauses based on filters
3. **Type safety** — Bind values are checked at compile time
4. **Readability** — Code is clearer than string concatenation

### Tag Filter Parsing

```rust
pub fn parse_tag_filters(input: &str) -> Result<Vec<TagFilter>, String> {
    let mut filters = Vec::new();
    for part in input.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }

        let colon_pos = part.find(':').ok_or_else(|| {
            format!("Invalid format '{}': expected 'type_id:name'", part)
        })?;
        let type_id: i16 = part[..colon_pos].parse()
            .map_err(|e| format!("Invalid type_id: {}", e))?;
        let name = part[colon_pos + 1..].to_string();
        filters.push(TagFilter { tag_type_id: type_id, tag_name: name });
    }
    Ok(filters)
}
```

Example: `"1:Harry Potter,2:Hermione Granger"` becomes:
```rust
vec![
    TagFilter { tag_type_id: 1, tag_name: "Harry Potter" },
    TagFilter { tag_type_id: 2, tag_name: "Hermione Granger" },
]
```

## Search Features

### Full-Text Search

```sql
-- Find stories mentioning "harry potter"
WHERE text_search @@ plainto_tsquery('english', 'harry potter')

-- Find stories with exact phrase
WHERE text_search @@ phraseto_tsquery('english', 'harry potter')
```

### Tag Filtering

```rust
// AND: All tags must match
"AND EXISTS (SELECT 1 FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id
 WHERE ft.url_id = fi.id AND t.name = 'Harry Potter')"

// OR: At least one tag must match
"AND EXISTS (SELECT 1 FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id
 WHERE ft.url_id = fi.id AND (t.name = 'Angst' OR t.name = 'Fluff'))"

// NOT: Tag must not exist
"AND NOT EXISTS (SELECT 1 FROM fic_tags ft JOIN tags t ON t.id = ft.tag_id
 WHERE ft.url_id = fi.id AND t.name = 'Character Death')"
```

### Numeric Filters

```rust
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
```

### Date Filters

```rust
if let Some(ref dt) = self.params.date_from {
    qb.push(" AND fi.fic_updated >= ");
    qb.push_bind(dt);
}
if let Some(ref dt) = self.params.date_to {
    qb.push(" AND fi.fic_updated <= ");
    qb.push_bind(dt);
}
```

### Sorting

```rust
match sort {
    "-relevance" => {
        if has_q { qb.push("rank DESC"); }
        else { qb.push("fi.fic_updated DESC"); }
    }
    "-date" => qb.push("fi.fic_updated DESC"),
    "-words" => qb.push("fi.words DESC"),
    "-chapters" => qb.push("fi.chapters DESC"),
    "-title" => qb.push("fi.title ASC"),
    _ => qb.push("fi.fic_updated DESC"),
}
```

### Pagination

```rust
let page = self.params.page.unwrap_or(1).max(1);
let per_page = self.params.per_page.unwrap_or(20).max(1);
let offset = (page - 1) * per_page;
qb.push(" LIMIT ");
qb.push_bind(per_page as i64);
qb.push(" OFFSET ");
qb.push_bind(offset as i64);
```

## Search Performance

### Indexing Strategy

```sql
-- GIN index for full-text search (ESSENTIAL)
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);

-- B-tree indexes for common filters
CREATE INDEX idx_fic_info_updated ON fic_info(fic_updated DESC);
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_source ON fic_info(source);

-- Indexes for tag queries
CREATE INDEX idx_fic_tags_url_id ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag_id ON fic_tags(tag_id);
```

### Query Analysis

Use `EXPLAIN ANALYZE` to analyze query performance:

```sql
EXPLAIN ANALYZE
SELECT fi.*, ts_rank(fi.text_search, plainto_tsquery('english', 'harry potter')) AS rank
FROM fic_info fi
WHERE fi.text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY rank DESC
LIMIT 20;
```

### Caching Search Results

For frequently repeated searches, cache the results:

```rust
async fn search_with_cache(
    state: &AppState,
    params: &SearchParams,
) -> Result<Vec<SearchResult>, AppError> {
    // Generate cache key from params
    let cache_key = format!("search:{}", serde_json::to_string(params)?);
    
    // Check Redis cache
    if let Ok(Some(cached)) = redis::cmd("GET").arg(&cache_key)
        .query_async::<_, String>(&mut state.redis.clone()).await {
        return serde_json::from_str(&cached).map_err(|e| AppError::Internal(e.to_string()));
    }
    
    // Execute search
    let results = execute_search(state, params).await?;
    
    // Cache for 5 minutes
    let json = serde_json::to_string(&results)?;
    let _: () = redis::cmd("SET").arg(&cache_key).arg(&json)
        .arg("EX").arg(300)
        .query_async(&mut state.redis.clone()).await.unwrap_or_default();
    
    Ok(results)
}
```

## Search Analytics

Track search queries to understand user behavior:

```rust
pub async fn log_search(
    pool: &PgPool,
    query: &str,
    result_count: i64,
    response_time_ms: i32,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO search_log (query, result_count, response_time_ms)
         VALUES ($1, $2, $3)"
    )
    .bind(query)
    .bind(result_count)
    .bind(response_time_ms)
    .execute(pool)
    .await?;
    Ok(())
}
```

## Advanced Search Features

### Autocomplete

```sql
SELECT DISTINCT title FROM fic_info
WHERE title ILIKE $1
LIMIT 10
```

### Spell Correction

```sql
-- Find similar words using trigram similarity
SELECT word, similarity(word, $1) AS sim
FROM pg_trgm.words
WHERE similarity(word, $1) > 0.3
ORDER BY sim DESC
LIMIT 5
```

### Faceted Search

```sql
-- Get tag counts for current search results
SELECT t.name, COUNT(*) as count
FROM fic_tags ft
JOIN tags t ON t.id = ft.tag_id
WHERE ft.url_id IN (
    SELECT id FROM fic_info
    WHERE text_search @@ plainto_tsquery('english', $1)
)
GROUP BY t.name
ORDER BY count DESC
LIMIT 20
```

## Conclusion

FicHub's search system provides powerful, flexible search capabilities using PostgreSQL's full-text search. The key components are:
- GIN indexes for fast full-text search
- Dynamic query building with `sqlx::QueryBuilder`
- Tag filtering with EXISTS subqueries
- Pagination for large result sets
- Caching for repeated searches
- Analytics for understanding user behavior


# Deep Dive: Tagging System Architecture

## Tag System Overview

FicHub's tagging system (v3) allows the community to add, vote on, and moderate tags for stories. This creates a rich metadata layer that improves search, recommendations, and discovery.

### Tag Types

| ID | Type | Example | Purpose |
|----|------|---------|---------|
| 1 | Fandom | Harry Potter | What universe the story is set in |
| 2 | Character | Hermione Granger | Which characters appear |
| 3 | Relationship | Harry/Hermione | Romantic or platonic pairings |
| 4 | Freeform | Angst, Fluff | Any other descriptors |
| 5 | Warning | Character Death | Content warnings |
| 6 | Category | F/M, Gen | Target audience |

### Tag Lifecycle

1. **Submission** — User submits a tag for a story
2. **Resolution** — System checks if tag exists, creates if not
3. **Association** — Tag is linked to the story
4. **Voting** — Community upvotes/downvotes the tag
5. **Visibility** — Tags below threshold are hidden
6. **Moderation** — Curators can merge, alias, or delete tags

## Tag Resolution

### The Three-Step Process

```rust
pub async fn resolve_tag(
    pool: &PgPool,
    tag_name: &str,
    tag_type_id: i16,
) -> AppResult<TagResolution> {
    // Step 1: Look up by exact name (case-sensitive)
    if let Some(row) = lookup_tag(pool, tag_name).await? {
        return Ok(TagResolution {
            tag_id: row.0,
            tag_name: row.1,
            tag_type_id: row.2,
            is_new: false,
        });
    }

    // Step 2: Look up in aliases
    if let Some(canonical_id) = lookup_alias(pool, tag_name).await? {
        let tag = lookup_tag_by_id(pool, canonical_id).await?
            .ok_or_else(|| AppError::Internal(format!(
                "alias '{}' points to non-existent tag", tag_name
            )))?;
        return Ok(TagResolution {
            tag_id: tag.0,
            tag_name: tag.1,
            tag_type_id: tag.2,
            is_new: false,
        });
    }

    // Step 3: Create new tag
    let new_id = create_tag(pool, tag_name, tag_type_id).await?;
    Ok(TagResolution {
        tag_id: new_id,
        tag_name: tag_name.to_string(),
        tag_type_id,
        is_new: true,
    })
}
```

### Why Case-Sensitive?

Tag lookup uses PostgreSQL's `COLLATE "C"` for case-sensitive matching:

```sql
SELECT id, name, tag_type_id FROM tags WHERE name = $1
```

This means "Angst" and "angst" are different tags. This prevents:
- Duplicate tags with different capitalization
- Inconsistent tag naming
- Confusion about which tag to use

### Alias System

Aliases map alternate names to canonical tags:

```sql
-- Create alias
INSERT INTO tag_aliases (alias_name, canonical_tag_id)
VALUES ('HP', 1)  -- "HP" -> "Harry Potter" (tag_id=1)

-- Lookup alias
SELECT canonical_tag_id FROM tag_aliases WHERE alias_name = 'HP'
```

Common alias patterns:
- Abbreviations: "HP" → "Harry Potter"
- Alternate spellings: "Drarry" → "Draco/Harry"
- Nicknames: "The Boy Who Lived" → "Harry Potter"

## Voting System

### Recording Votes

```rust
pub async fn record_vote(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    voter_ip: IpAddr,
    value: i16,
    hidden_threshold: i16,
) -> AppResult<VoteResult> {
    // Upsert the vote
    sqlx::query(
        r#"INSERT INTO fic_tag_votes (url_id, tag_id, voter_ip, value)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, voter_ip)
           DO UPDATE SET value = EXCLUDED.value"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(voter_ip.to_string())
    .bind(value)
    .execute(pool)
    .await?;

    // Read back updated score
    let row: (i16,) = sqlx::query_as(
        "SELECT score FROM fic_tags WHERE url_id = $1 AND tag_id = $2",
    )
    .bind(url_id)
    .bind(tag_id)
    .fetch_one(pool)
    .await?;

    let new_score = row.0;
    let hidden = is_hidden(new_score, hidden_threshold);

    Ok(VoteResult { new_score, hidden })
}
```

### Vote Behavior

- Each IP can vote once per tag per story
- Changing a vote updates the existing record
- The `fic_tags.score` is automatically updated via a database trigger
- The score determines visibility

### Visibility Threshold

```rust
pub fn is_hidden(score: i16, threshold: i16) -> bool {
    score <= threshold
}
```

Default threshold: `-3`. A tag needs at least 3 downvotes to be hidden.

### Why Not Delete Low-Scored Tags?

Keeping hidden tags has advantages:
1. The tag might be useful for other stories
2. Voting patterns can change over time
3. It preserves history
4. Hidden tags can be unhidden if the community changes its mind
5. It prevents abuse (someone could downvote all tags to hide them)

## Auto-Moderation

### Hidden Tags

Tags below the threshold are hidden from normal queries:

```rust
pub async fn get_fic_tags(
    pool: &PgPool,
    url_id: &str,
    hidden_threshold: i16,
) -> AppResult<Vec<(i32, String, i16, i16, bool)>> {
    let rows = sqlx::query_as::<_, (i32, String, i16, i16)>((
        r#"SELECT t.id, t.name, t.tag_type_id, ft.score
           FROM fic_tags ft
           JOIN tags t ON t.id = ft.tag_id
           WHERE ft.url_id = $1
           ORDER BY t.tag_type_id, ft.score DESC"#,
    ))
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

### Auto-Delete Threshold

For tags that are extremely unpopular, FicHub can auto-delete them:

```rust
pub tag_auto_delete_threshold: Option<i16>,  // e.g., -5
```

When a tag's score drops below this threshold, it's automatically removed.

### Flagging System

Users can flag inappropriate tags:

```rust
pub async fn insert_tag_flag(
    pool: &PgPool,
    url_id: &str,
    tag_id: i32,
    flagged_by_ip: &std::net::IpAddr,
    reason: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO tag_flags (url_id, tag_id, flagged_by_ip, reason)
           VALUES ($1, $2, $3::inet, $4)
           ON CONFLICT (url_id, tag_id, flagged_by_ip) DO NOTHING"#,
    )
    .bind(url_id)
    .bind(tag_id)
    .bind(flagged_by_ip.to_string())
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}
```

## Curator Features

### Creating Aliases

```rust
pub async fn create_alias(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AliasBody>,
) -> Result<Json<Value>, AppError> {
    // Verify curator token
    let token = body.token.as_deref()
        .ok_or_else(|| AppError::BadRequest(-1, "token required".into()))?;

    if Some(token.to_string()) != state.config.curator_token {
        return Ok(Json(json!({"err": -3, "msg": "invalid curator token"})));
    }

    // Create the alias
    queries::create_tag_alias(&state.db, &body.alias_name, body.canonical_tag_id).await?;

    Ok(Json(json!({"err": 0, "msg": "alias created"})))
}
```

### Merging Tags

```rust
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    // Reassign fic_tags from source to target
    sqlx::query(
        r#"INSERT INTO fic_tags (url_id, tag_id, added_by_ip, score)
           SELECT ft.url_id, $2, ft.added_by_ip, ft.score
           FROM fic_tags ft
           WHERE ft.tag_id = $1
           ON CONFLICT (url_id, tag_id)
           DO UPDATE SET score = GREATEST(fic_tags.score, EXCLUDED.score)"#,
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

When merging, if both tags exist on the same story, the higher score is kept (`GREATEST`).

### Deleting Tags

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

    // Delete fic_tags
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    // Delete aliases
    sqlx::query("DELETE FROM tag_aliases WHERE canonical_tag_id = $1")
        .bind(tag_id).execute(pool).await?;

    // Delete tag
    sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id).execute(pool).await?;

    Ok(())
}
```

The `force` parameter allows deleting tags that are still in use.

### Resolving Flags

```rust
pub async fn resolve_flag(pool: &PgPool, flag_id: i64) -> AppResult<()> {
    sqlx::query("UPDATE tag_flags SET resolved = TRUE WHERE id = $1")
        .bind(flag_id)
        .execute(pool)
        .await?;
    Ok(())
}
```

## Rate Limiting

### Tag Submission Limits

```rust
pub async fn check_rate_limit(
    redis: &mut redis::aio::MultiplexedConnection,
    key: &str,
    max_per_hour: u32,
) -> AppResult<()> {
    let count: Option<u32> = redis.get(key).await.unwrap_or(None);
    if count.unwrap_or(0) >= max_per_hour {
        return Err(AppError::RateLimited(3600));
    }
    let _: () = redis.incr(key, 1).await.unwrap_or_default();
    let _: () = redis.expire(key, 3600).await.unwrap_or_default();
    Ok(())
}
```

Default limits:
- Tag submissions: 10 per hour per IP
- Tag votes: 20 per hour per IP

## Tag API Endpoints

### Submit Tag

```rust
pub async fn submit_tag(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SubmitTagBody>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("tag:submit:{}", body.ip);
    check_rate_limit(&mut state.redis.clone(), &key, state.config.tag_submit_limit_per_hour).await?;

    // Resolve tag
    let resolution = crate::tags::resolve::resolve_tag(
        &state.db, &body.tag_name, body.tag_type_id,
    ).await?;

    // Associate with story
    queries::upsert_fic_tag(
        &state.db, &body.url_id, resolution.tag_id, &body.ip,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "tag_id": resolution.tag_id,
        "tag_name": resolution.tag_name,
        "is_new": resolution.is_new,
    })))
}
```

### Vote on Tag

```rust
pub async fn vote_tag(
    State(state): State<Arc<AppState>>,
    Json(body): Json<VoteTagBody>,
) -> Result<Json<Value>, AppError> {
    // Rate limit check
    let key = format!("tag:vote:{}", body.ip);
    check_rate_limit(&mut state.redis.clone(), &key, state.config.tag_vote_limit_per_hour).await?;

    // Record vote
    let result = crate::tags::voting::record_vote(
        &state.db,
        &body.url_id,
        body.tag_id,
        body.ip,
        body.value,
        state.config.tag_hidden_threshold,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "new_score": result.new_score,
        "hidden": result.hidden,
    })))
}
```

### Get Tags

```rust
pub async fn get_tags(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GetTagsQuery>,
) -> Result<Json<Value>, AppError> {
    let tags = queries::get_fic_tags(
        &state.db,
        &params.url_id,
        state.config.tag_hidden_threshold,
    ).await?;

    Ok(Json(json!({
        "err": 0,
        "tags": tags.iter().map(|(id, name, type_id, score, hidden)| {
            json!({
                "id": id,
                "name": name,
                "type_id": type_id,
                "score": score,
                "hidden": hidden,
            })
        }).collect::<Vec<_>>(),
    })))
}
```

## Database Schema

### Tags Table

```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    tag_type_id SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Tag Aliases Table

```sql
CREATE TABLE tag_aliases (
    alias_name TEXT PRIMARY KEY,
    canonical_tag_id INTEGER NOT NULL REFERENCES tags(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Fic Tags Table

```sql
CREATE TABLE fic_tags (
    url_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL REFERENCES tags(id),
    added_by_ip INET,
    score SMALLINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id)
);
```

### Tag Votes Table

```sql
CREATE TABLE fic_tag_votes (
    url_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    voter_ip INET NOT NULL,
    value SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);
```

### Tag Flags Table

```sql
CREATE TABLE tag_flags (
    id BIGSERIAL PRIMARY KEY,
    url_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    flagged_by_ip INET NOT NULL,
    reason TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (url_id, tag_id, flagged_by_ip)
);
```

## Performance Considerations

### Indexes

```sql
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_type ON tags(tag_type_id);
CREATE INDEX idx_fic_tags_url ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag ON fic_tags(tag_id);
CREATE INDEX idx_fic_tags_score ON fic_tags(url_id, score DESC);
CREATE INDEX idx_tag_votes_url ON fic_tag_votes(url_id);
CREATE INDEX idx_tag_votes_tag ON fic_tag_votes(tag_id);
CREATE INDEX idx_tag_flags_resolved ON tag_flags(resolved) WHERE resolved = FALSE;
```

### Query Optimization

Tag queries can be expensive with many tags. Optimizations:
1. Use indexes on `fic_tags(url_id)` and `fic_tags(tag_id)`
2. Limit the number of tags returned
3. Cache popular tag lists
4. Use materialized views for tag statistics

## Conclusion

FicHub's tagging system provides a flexible, community-driven way to categorize stories. Key features include:
- Automatic tag resolution (existing → alias → new)
- Community voting with visibility thresholds
- Curator moderation (aliases, merges, deletions)
- Rate limiting to prevent abuse
- Flagging system for inappropriate content
- Database-backed persistence with proper indexing


# Deep Dive: Cache System Architecture

## Cache Architecture Overview

FicHub's cache system has three layers:

### Layer 1: Database Cache Log

The `export_log` table tracks which files have been generated:

```sql
CREATE TABLE export_log (
    url_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);
```

When a request comes in, FicHub first checks this table. If a matching entry exists, the file is cached on disk.

### Layer 2: Disk Storage

Cached files are stored in a structured directory:

```
cache/
├── epub/
│   ├── a1b/
│   │   └── a1b2c3d4e5f6/
│   │       └── a1b2c3d4e5f6/
│   │           └── a1b2c3d4e5f6.epub
│   └── x9y/
│       └── x9y8z7w6v5u4/
│           └── x9y8z7w6v5u4/
│               └── x9y8z7w6v5u4.epub
├── html/
│   └── ...
├── mobi/
│   └── ...
└── pdf/
    └── ...
```

The 3-character prefix directories prevent having too many files in a single folder.

### Layer 3: Concurrent Export Prevention

The `CacheSemaphores` system prevents two requests from generating the same file at the same time:

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

## Cache Path Computation

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

    // Actual file: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}
```

For a url_id like `"a1b2c3d4e5f6"`:
- First chunk: `"a1b"` (characters 0-2)
- Second chunk: `"c3d"` (characters 3-5)
- Third chunk: `"4e5"` (characters 6-8)
- Full directory: `"a1b2c3d4e5f6"`
- File: `"a1b2c3d4e5f6.epub"`

## The Double-Check Pattern

FicHub uses a clever trick called the **double-check pattern**:

```rust
// First check (fast — just a database query)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Cache hit! Return immediately
    return Ok(build_response(&meta, &export_log));
}

// Acquire semaphore (might wait if another request is generating the same file)
let sem = cache::get_export_semaphore(&state.cache_semaphores, &meta.url_id, &EType::Epub).await;
let _permit = sem.acquire().await?;

// Second check (another request might have generated it while we were waiting)
let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &input_hash).await?;
if let Some(export_log) = cached {
    // Another request generated it! Use the cached version
    return Ok(build_response(&meta, &export_log));
}

// Actually generate the EPUB
let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
let (epub_path, epub_hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;

// Move to cache directory
let cache_dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &epub_hash);
cache::disk::move_to_cache(&epub_path, &cache_dest)?;

// Record in database
queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &epub_hash).await?;
```

### Why Two Checks?

Without the second check, this could happen:
1. Request A checks cache — miss
2. Request B checks cache — miss
3. Request A acquires semaphore, generates EPUB
4. Request B acquires semaphore, generates EPUB again (wasteful!)

With the double-check:
1. Request A checks cache — miss
2. Request B checks cache — miss
3. Request A acquires semaphore, checks cache again — miss, generates EPUB
4. Request B acquires semaphore, checks cache again — hit! Uses cached version

## Hash Validation

When serving cached files, FicHub validates the hash:

```rust
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Md5, Digest};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}
```

The hash ensures the file hasn't been corrupted or tampered with.

### Serving Cached Files

```rust
pub async fn download_with_hash(
    State(state): State<Arc<AppState>>,
    Path((etype_str, url_id, fname)): Path<(String, String, String)>,
    Query(params): Query<DownloadQuery>,
) -> Response {
    let hash = match params.h {
        Some(h) => h,
        None => fname.trim_end_matches(etype.suffix()).to_string(),
    };

    let cache_path = crate::cache::disk::cache_path(
        &state.config.cache_dir, &etype, &url_id, &hash,
    );

    if !cache_path.exists() {
        return Json(json!({"err": -5, "msg": "file not found"})).into_response();
    }

    // Validate hash matches actual file content
    match crate::cache::disk::file_md5(&cache_path) {
        Ok(actual_hash) if actual_hash == hash => {
            // Hash matches — serve the file
            let data = tokio::fs::read(&cache_path).await.unwrap();
            let mime = match etype {
                EType::Epub => "application/epub+zip",
                EType::Html => "application/zip",
                EType::Mobi => "application/x-mobipocket-ebook",
                EType::Pdf => "application/pdf",
            };
            let filename = format!("{}{}", url_id, etype.suffix());
            let headers = [
                ("Content-Type", mime),
                ("Content-Disposition", &format!("attachment; filename="{}"", filename)),
            ];
            (headers, data).into_response()
        }
        _ => Json(json!({"err": -5, "msg": "hash mismatch"})).into_response(),
    }
}
```

## Cache Invalidation

### Version-Based Invalidation

FicHub uses version numbers for cache invalidation:

```rust
pub fn compute_version(export_version: i32, etype_version: i32, fic_version_bump: i32) -> i32 {
    export_version + etype_version + fic_version_bump
}
```

When any version changes, the cache key changes, and FicHub regenerates the file.

### Content Hash Invalidation

If the story content changes on the source site, the `content_hash` changes:

```rust
let input_hash = meta.content_hash.clone().unwrap_or_else(|| "upstream".to_string());
```

### Manual Invalidation

Curators can manually bump versions:

```sql
INSERT INTO fic_version_bump (id, value)
VALUES ('abc123', 1)
ON CONFLICT (id) DO UPDATE SET value = fic_version_bump.value + 1;
```

## Cache Cleanup

### Stale Cache Removal

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

### Disk Space Management

For production, implement disk space limits:

```rust
pub fn enforce_cache_limit(cache_root: &Path, max_size_bytes: u64) -> AppResult<()> {
    let mut total_size = 0u64;
    let mut files = Vec::new();

    for entry in fs::read_dir(cache_root)?.flatten() {
        if entry.path().is_file() {
            let metadata = entry.metadata()?;
            total_size += metadata.len();
            files.push((entry.path(), metadata.modified().ok()));
        }
    }

    if total_size > max_size_bytes {
        // Sort by modification time (oldest first)
        files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove oldest files until under limit
        for (path, _) in &files {
            if total_size <= max_size_bytes {
                break;
            }
            if let Ok(metadata) = fs::metadata(path) {
                total_size -= metadata.len();
                let _ = fs::remove_file(path);
            }
        }
    }

    Ok(())
}
```

## Cache Statistics

Track cache performance:

```rust
pub struct CacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub generated: AtomicU64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 { 0.0 } else { hits as f64 / total as f64 }
    }
}
```

## Cache Warming

Pre-generate popular files:

```rust
pub async fn warm_cache(state: &AppState) -> AppResult<()> {
    // Get most popular stories
    let popular: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT fi.id, COUNT(rl.id) as downloads
           FROM fic_info fi
           LEFT JOIN request_log rl ON rl.url_id = fi.id
           GROUP BY fi.id
           ORDER BY downloads DESC
           LIMIT 100"#,
    ).fetch_all(&state.db).await?;

    // Pre-generate EPUBs for popular stories
    for (url_id, _downloads) in popular {
        let cached = queries::find_export_log(
            &state.db, &url_id, state.config.export_version, "epub", "upstream"
        ).await?;

        if cached.is_none() {
            // Generate EPUB in background
            let state = state.clone();
            let url_id = url_id.clone();
            tokio::spawn(async move {
                if let Err(e) = generate_and_cache(&state, &url_id).await {
                    tracing::warn!("Failed to warm cache for {}: {}", url_id, e);
                }
            });
        }
    }

    Ok(())
}
```

## Cache Best Practices

### Use Content-Addressable Storage

Files are stored by their hash, not by their URL. This ensures:
- Deduplication (same content = same hash = same file)
- Integrity (hash mismatch = corruption detected)
- Immutability (hash never changes for same content)

### Validate Before Serving

Always validate the hash before serving a cached file:

```rust
match crate::cache::disk::file_md5(&cache_path) {
    Ok(actual_hash) if actual_hash == hash => {
        // Serve the file
    }
    _ => {
        // Hash mismatch — file is corrupted or tampered
        return Err(AppError::CacheError("hash mismatch".into()));
    }
}
```

### Use Semaphores for Concurrent Access

Prevent duplicate generation with semaphores:

```rust
let sem = cache::get_export_semaphore(&state.cache_semaphores, &url_id, &etype).await;
let _permit = sem.acquire().await?;
```

### Monitor Cache Hit Rate

Track and optimize cache performance:

```rust
let hit_rate = cache_stats.hit_rate();
tracing::info!("Cache hit rate: {:.1}%", hit_rate * 100.0);
```

## Conclusion

FicHub's cache system provides efficient, reliable caching through:
1. Database-backed cache logging
2. Content-addressable disk storage
3. Semaphore-based concurrency control
4. Double-check pattern for efficiency
5. Hash validation for integrity
6. Version-based invalidation
7. Automatic cleanup and warming

This ensures fast responses for repeated requests while maintaining data integrity.


# Deep Dive: Rate Limiting Deep Dive

## Token Bucket Algorithm

### How It Works

The token bucket algorithm is like a bucket that:
- Has a maximum capacity (e.g., 30 tokens)
- Refills at a steady rate (e.g., 0.116 tokens/second)
- Each request costs 1 token
- If the bucket is empty, you must wait

### Mathematical Model

```
new_tokens = min(capacity, value + elapsed * flow)
allowed = new_tokens - requested

if allowed >= 0:
    consume tokens
    return allowed
else:
    wait_time = (requested - new_tokens) / flow
    return wait_time
```

### Example

Bucket: capacity=10, flow=1 token/sec

| Time | Value | Request | Result | New Value |
|------|-------|---------|--------|-----------|
| 0.0 | 10 | 3 | Allowed | 7 |
| 0.5 | 7.5 | 5 | Allowed | 2.5 |
| 1.0 | 3.5 | 5 | Wait 2.5s | 2.5 |
| 3.5 | 5.0 | 5 | Allowed | 0 |
| 4.0 | 0.5 | 3 | Wait 2.5s | 0.5 |

### Why Token Bucket?

Compared to other algorithms:
- **Fixed window** — Allows bursts at window boundaries
- **Sliding window** — More complex, similar performance
- **Leaky bucket** — Constant rate, no bursts
- **Token bucket** — Allows controlled bursts, simple to implement

Token bucket is ideal for FicHub because:
1. Allows short bursts (good for bursty traffic)
2. Smooths out over time (prevents sustained overload)
3. Easy to implement in Redis with Lua scripts

## Redis Implementation

### The Lua Script

```lua
-- Input:
-- KEYS[1] = bucket key
-- ARGV[1] = tokens requested (usually 1)
-- ARGV[2] = bucket capacity
-- ARGV[3] = flow rate (tokens per second)

-- Output:
-- -1 if allowed
-- positive number = seconds to wait

local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

-- Get current state
local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

-- Get current time
local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

-- Initialize if new bucket
if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

-- Calculate new tokens
local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    -- Request allowed
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    -- Request denied — calculate wait time
    local wait = (requested - new_tokens) / flow
    return wait
end
```

### Why Lua Scripts?

Redis executes Lua scripts atomically — no other command can run while a script is executing. This is crucial for rate limiting because:
1. Reading and writing the token bucket must be atomic
2. Two concurrent requests can't both think they have enough tokens
3. The time calculation must be consistent

### Loading the Script

```rust
let lua_script = r#"..."#;
let mut conn = redis_conn.clone();
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;
```

Redis compiles the script and returns a SHA hash. Subsequent calls use `EVALSHA` with the hash.

### Executing the Script

```rust
let result: f64 = redis::cmd("EVALSHA")
    .arg(&self.lua_sha[..])
    .arg(1)           // Number of keys
    .arg(key)         // Key
    .arg(1.0)         // Requested tokens
    .arg(capacity)    // Bucket capacity
    .arg(flow)        // Flow rate
    .query_async(&mut conn)
    .await?;
```

## Two-Tier Rate Limiting

### Global Bucket

System-wide limit across all users:

```rust
let global_wait = self.check_bucket("rate:global", 150.0, 30.0).await?;
if global_wait > 0.0 {
    return RateLimitResult::Wait(global_wait.ceil() as u64);
}
```

- Capacity: 150 tokens
- Flow: 30 tokens/second
- Effective rate: 30 requests/second

### Per-IP Bucket

Individual user limit:

```rust
let ip_key = format!("rate:ip:{}", ip);
let ip_wait = self.check_bucket(&ip_key, 30.0, 0.116).await?;
if ip_wait > 0.0 {
    return RateLimitResult::Wait(ip_wait.ceil() as u64);
}
```

- Capacity: 30 tokens
- Flow: 0.116 tokens/second
- Effective rate: 1 request every ~8.6 seconds

### Why Two Tiers?

- **Global limit** prevents one user from consuming all server resources
- **Per-IP limit** prevents individual users from overwhelming fanfiction sites
- Together, they ensure fair usage and site compatibility

## Failure Penalization

When a request fails, consume extra tokens:

```rust
async fn penalize(&self, key: &str, capacity: f64, flow: f64) -> Result<(), redis::RedisError> {
    let mut conn = self.redis.clone();
    let _: f64 = redis::cmd("EVALSHA")
        .arg(&self.lua_sha[..])
        .arg(1)
        .arg(key)
        .arg(1.5)  // Penalize with 1.5 tokens
        .arg(capacity)
        .arg(flow)
        .query_async(&mut conn)
        .await?;
    Ok(())
}
```

The penalty uses 1.5 tokens instead of 1, making the IP wait longer before trying again. This discourages rapid retries that could overwhelm the system.

## Static vs Dynamic Rate Limiting

### Dynamic Mode

Uses token buckets with Redis:

```rust
if self.dynamic_rate_limit {
    // Check global bucket
    let global_wait = self.check_bucket("rate:global", 150.0, 30.0).await?;
    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Check per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, 30.0, 0.116).await?;
    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

Advantages:
- More efficient — only slows down when needed
- More accurate — accounts for actual usage patterns
- Allows bursts — good for bursty traffic

### Static Mode

Adds random delay to every request:

```rust
else {
    let delay = 0.1 + rand::random::<f64>() * 0.1;  // 100-200ms
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
    RateLimitResult::Allowed
}
```

Advantages:
- Simpler — no Redis needed
- Works offline — no network dependency

Disadvantages:
- Less efficient — slows down even when not needed
- Less accurate — doesn't account for usage patterns

## Datacenter IP Blocking

### Loading IP Ranges

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

### Checking IPs

```rust
fn is_datacenter_ip(&self, _ip: IpAddr) -> bool {
    // Synchronous check - best-effort
    // For production, use a proper prefix tree (ipnet crate)
    false
}
```

Note: The current implementation always returns `false`. For production, implement proper IP range checking using the `ipnetwork` crate.

## Rate Limiter Configuration

```rust
pub struct RedisBucketLimiter {
    redis: redis::aio::MultiplexedConnection,
    lua_sha: String,
    dynamic_rate_limit: bool,
    static_delay_base: f64,
    datacenter_ips: Arc<RwLock<HashSet<IpAddr>>>,
    global_capacity: f64,   // 150
    global_flow: f64,       // 30
    ip_capacity: f64,       // 30
    ip_flow: f64,           // 0.116
}
```

### Configuration via Environment Variables

```rust
let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
    .unwrap_or_else(|_| "true".to_string())
    .parse::<bool>()
    .unwrap_or(true);
```

## Performance Considerations

### Redis Overhead

Each rate limit check requires:
1. One `HMGET` to read bucket state
2. One `TIME` to get current time
3. One `HMSET` to update bucket state

Total: ~3 Redis commands per check. At 1000 requests/second, that's 3000 Redis commands/second.

### Connection Pooling

Use multiplexed connections to share a single TCP connection:

```rust
let redis_conn = redis_client.get_multiplexed_async_connection().await?;
```

### Caching Rate Limit Results

For very high throughput, cache rate limit results in memory:

```rust
struct RateLimitCache {
    cache: Arc<Mutex<HashMap<IpAddr, (Instant, RateLimitResult)>>>,
}

impl RateLimitCache {
    async fn check(&self, ip: IpAddr) -> Option<RateLimitResult> {
        let cache = self.cache.lock().await;
        if let Some((time, result)) = cache.get(&ip) {
            if time.elapsed() < Duration::from_secs(1) {
                return Some(result.clone());
            }
        }
        None
    }
}
```

## Testing Rate Limiting

### Unit Tests

```rust
#[test]
fn test_token_bucket_initial_fill() {
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
    let r = bucket.request(20.0, 3.0);  // After 3s, 30 tokens refilled
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);  // 30 - 20 = 10
}

#[test]
fn test_wait_calculation() {
    let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
    bucket.request(10.0, 0.0);  // Empty
    let wait = bucket.request(5.0, 0.0);  // Need 5 tokens
    let expected = 5.0 / 2.0;  // 2.5 seconds
    assert!((wait - expected).abs() < f64::EPSILON);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_rate_limiter() {
    let redis = create_test_redis().await;
    let limiter = RedisBucketLimiter::new(redis, true).await.unwrap();
    
    let ip = "192.168.1.1".parse().unwrap();
    
    // First request should be allowed
    let result = limiter.check_ip(ip).await;
    assert!(matches!(result, RateLimitResult::Allowed));
    
    // Many rapid requests should eventually be limited
    let mut limited = false;
    for _ in 0..100 {
        let result = limiter.check_ip(ip).await;
        if matches!(result, RateLimitResult::Wait(_)) {
            limited = true;
            break;
        }
    }
    assert!(limited);
}
```

## Conclusion

FicHub's rate limiting system provides:
1. Token bucket algorithm for smooth, burst-tolerant limiting
2. Two-tier limiting (global + per-IP) for fair usage
3. Redis-backed implementation for distributed systems
4. Lua scripts for atomic operations
5. Failure penalization to discourage abuse
6. Datacenter IP blocking for bot prevention
7. Configurable via environment variables
8. Comprehensive test coverage

This ensures FicHub remains responsive while respecting upstream fanfiction sites.


# Deep Dive: Error Handling Patterns

## FicHub's Error Types

### The AppError Enum

```rust
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
    /// Scraper error
    ScrapeError(String),
    /// Export/generation error
    ExportError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

Each variant provides enough context to:
1. Generate an appropriate HTTP status code
2. Log useful debug information
3. Return a safe error message to the client

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

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            AppError::RateLimited(retry_after) => {
                (StatusCode::TOO_MANY_REQUESTS,
                 json!({"err": -429, "msg": "rate limited", "retry_after": retry_after}))
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, json!({"err": -5, "msg": msg}))
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "internal server error"}))
            }
            AppError::ScrapeError(msg) => {
                (StatusCode::BAD_GATEWAY, json!({"err": -6, "msg": msg}))
            }
            AppError::ExportError(msg) => {
                tracing::error!("Export error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -5, "msg": "export failed"}))
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "database error"}))
            }
            AppError::CacheError(msg) => {
                tracing::error!("Cache error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR,
                 json!({"err": -1, "msg": "cache error"}))
            }
        };

        (status, Json(body)).into_response()
    }
}
```

### From Implementations

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

## The ? Operator

The `?` operator is syntactic sugar for error propagation:

```rust
// This:
let data = std::fs::read_to_string("config.json")?;

// Is equivalent to:
let data = match std::fs::read_to_string("config.json") {
    Ok(data) => data,
    Err(e) => return Err(e.into()),
};
```

### Chaining with ?

```rust
async fn get_fic_info(pool: &PgPool, id: &str) -> AppResult<Option<FicInfo>> {
    let row = sqlx::query_as::<_, FicInfo>(
        "SELECT * FROM fic_info WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;  // sqlx::Error → AppError
    Ok(row)
}
```

## Error Handling Patterns

### Pattern 1: Validate, Then Process

```rust
// Validate first
let query = params.q.as_deref().unwrap_or("");
if query.is_empty() {
    return Ok(Json(json!({"err": -1, "msg": "no query"})));
}

// Then process
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

### Pattern 2: Log and Hide Internal Errors

```rust
AppError::Internal(msg) => {
    tracing::error!("Internal error: {}", msg);  // Log the details
    (StatusCode::INTERNAL_SERVER_ERROR,
     json!({"err": -1, "msg": "internal server error"}))  // Hide from client
}
```

### Pattern 3: Check Multiple Conditions

```rust
let fic_blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
if !fic_blacklist.is_empty() {
    if fic_blacklist.iter().any(|b| b.reason == 6) {
        return Ok(build_metadata_response(&meta, &[], &state.config.export_version, None, true));
    }
    if fic_blacklist.iter().any(|b| b.reason == 5 || b.reason == 7 || b.reason == 8) {
        return Ok(Json(json!({"err": -7, "msg": "fic is blacklisted"})));
    }
}
```

### Pattern 4: The ok_or_else Pattern

```rust
let scraper = state.scraper_registry.find_scraper(query)
    .ok_or_else(|| AppError::BadRequest(-5, format!("unsupported URL: {}", query)))?;
```

`find_scraper` returns `Option<&Box<dyn SiteScraper>>`. `ok_or_else` converts `None` into an `AppError`.

### Pattern 5: The map_err Pattern

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

`lookup` returns `Result<FicMetadata, ScrapeError>`. We use `.map_err` to convert the `ScrapeError` into an `AppError::ScrapeError`.

## Error Best Practices

### Don't Use .unwrap() in Production

```rust
// BAD: Panics on error
let data = std::fs::read_to_string("config.json").unwrap();

// GOOD: Handles error gracefully
let data = match std::fs::read_to_string("config.json") {
    Ok(data) => data,
    Err(e) => {
        tracing::warn!("Failed to read config: {}", e);
        return Err(AppError::Internal(e.to_string()));
    }
};
```

### Provide Context

```rust
// BAD: Generic error message
return Err(AppError::Internal("failed".into()));

// GOOD: Specific error message
return Err(AppError::Internal(format!("failed to read config file: {}", e)));
```

### Log Before Returning

```rust
// BAD: Silent failure
return Err(AppError::Database(e.to_string()));

// GOOD: Log then return
tracing::error!("Database query failed: {}", e);
return Err(AppError::Database(e.to_string()));
```

### Use the Right Error Type

```rust
// BAD: Everything is Internal
return Err(AppError::Internal("scraper failed".into()));

// GOOD: Specific error types
return Err(AppError::ScrapeError("failed to fetch story".into()));
```

## Testing Error Handling

### Test Error Display

```rust
#[test]
fn test_display_bad_request() {
    let err = AppError::BadRequest(400, "invalid input".into());
    let s = format!("{}", err);
    assert!(s.contains("BadRequest"));
    assert!(s.contains("400"));
    assert!(s.contains("invalid input"));
}
```

### Test Error Conversion

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
```

### Test HTTP Status Codes

```rust
#[tokio::test]
async fn test_error_response_status() {
    let error = AppError::NotFound("story not found".into());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
```

## Conclusion

FicHub's error handling system provides:
1. Comprehensive error types for different failure modes
2. Automatic HTTP status code mapping
3. Safe error messages (no information leakage)
4. Detailed logging for debugging
5. Automatic error conversion via From trait
6. Clean propagation via ? operator
7. Comprehensive test coverage

This ensures FicHub handles failures gracefully while providing useful feedback to users and administrators.


# Deep Dive: Configuration Management

## Configuration Overview

FicHub's configuration is loaded from environment variables. This approach:
- Keeps secrets out of code
- Makes deployment flexible (different settings for dev/staging/prod)
- Follows Twelve-Factor App principles
- Works with Docker, Kubernetes, and traditional deployments

## The Config Struct

```rust
pub struct Config {
    // Required settings
    pub database_url: String,
    pub redis_url: String,
    
    // File paths
    pub cache_dir: PathBuf,
    pub secondary_cache_dir: Option<PathBuf>,
    pub tmp_dir: PathBuf,
    pub frontend_dir: PathBuf,
    
    // Server settings
    pub app_port: u16,
    pub node_name: String,
    
    // Export settings
    pub export_version: i32,
    pub dynamic_rate_limit: bool,
    pub calibre_container: String,
    
    // Network settings
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
    
    // Tagging settings
    pub curator_token: Option<String>,
    pub tag_hidden_threshold: i16,
    pub tag_auto_delete_threshold: Option<i16>,
    pub tag_submit_limit_per_hour: u32,
    pub tag_vote_limit_per_hour: u32,
    
    // Search settings
    pub search_max_per_page: usize,
    
    // OPDS settings
    pub opds_shelf_token: String,
}
```

## Loading Configuration

### The from_env Method

```rust
impl Config {
    pub fn from_env() -> Self {
        // Required settings
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        
        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");
        
        // Optional settings with defaults
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());
        
        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);
        
        // ... more settings ...
        
        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            // ...
        }
    }
}
```

### Pattern: Required Settings

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

Use `.expect()` for required settings. The program should crash immediately if they're missing.

### Pattern: Optional Settings with Defaults

```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());
```

Use `.unwrap_or_else()` for optional settings with string defaults.

### Pattern: Parsed Settings

```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

Chain `.parse()` for numeric settings. The double `.unwrap_or` handles both missing and invalid values.

### Pattern: Boolean Settings

```rust
let dynamic_rate_limit = std::env::var("DYNAMIC_RATE_LIMIT")
    .unwrap_or_else(|_| "true".to_string())
    .parse::<bool>()
    .unwrap_or(true);
```

Boolean settings accept "true" or "false" (case-insensitive).

### Pattern: Optional Settings

```rust
let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
    .ok()  // Convert Result to Option
    .filter(|s| !s.is_empty())  // Ignore empty strings
    .map(PathBuf::from);  // Convert to PathBuf
```

Use `.ok()` and `.filter()` for truly optional settings.

### Pattern: List Settings

```rust
let trusted_proxies = std::env::var("TRUSTED_PROXIES")
    .unwrap_or_default()
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();
```

Comma-separated lists are split and trimmed.

### Pattern: Complex Settings

```rust
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

Multi-line settings are parsed line by line.

### Pattern: JSON Settings

```rust
let rec_site_rate_limits_str = std::env::var("REC_SITE_RATE_LIMITS")
    .unwrap_or_else(|_| "{}".to_string());
let rec_site_rate_limits: HashMap<String, u64> =
    serde_json::from_str(&rec_site_rate_limits_str).unwrap_or_default();
```

Complex settings can be encoded as JSON.

## Environment Variables Reference

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | `postgres://user:pass@host/db` |
| `REDIS_URL` | Redis connection URL | `redis://localhost/` |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CACHE_DIR` | `./cache` | Cached file directory |
| `PORT` | `3000` | Server port |
| `NODE_NAME` | `orion` | Server instance name |
| `EXPORT_VERSION` | `1` | Export version number |
| `DYNAMIC_RATE_LIMIT` | `true` | Smart rate limiting |
| `TMP_DIR` | `./tmp` | Temporary directory |
| `FRONTEND_DIR` | `./frontend/build` | Frontend build directory |

### Recommender Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `REC_DEFAULT_DELAY_SECS` | `5` | Delay between scraping |
| `REC_MAX_RECOMMENDATIONS` | `20` | Max recommendations |
| `REC_VOTING_BOOST_GAMMA` | `0.2` | Vote boost factor |
| `REC_CACHE_TTL_HOURS` | `12` | Recommendation cache TTL |
| `REC_MIN_FAVOURITERS_FOR_COLLAB` | `5` | Min for collaborative filtering |

### Tagging Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CURATOR_TOKEN` | `None` | Curator authentication token |
| `TAG_HIDDEN_THRESHOLD` | `-3` | Tag visibility threshold |
| `TAG_SUBMIT_LIMIT_PER_HOUR` | `10` | Tag submission rate limit |
| `TAG_VOTE_LIMIT_PER_HOUR` | `20` | Tag vote rate limit |

### Search Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SEARCH_MAX_PER_PAGE` | `50` | Max search results per page |

### OPDS Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `OPDS_SHELF_TOKEN` | `fichub` | OPDS shelf authentication |

## Testing Configuration

### The EnvGuard Pattern

```rust
static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    keys: Vec<String>,
}

impl EnvGuard {
    fn new() -> Self { EnvGuard { keys: Vec::new() } }
    
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

### Test Examples

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
}

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

## Configuration Best Practices

### Use Defaults Liberally

Provide sensible defaults for optional settings:

```rust
let cache_dir = std::env::var("CACHE_DIR")
    .unwrap_or_else(|_| "./cache".to_string());
```

### Validate Early

Check configuration at startup:

```rust
let config = Config::from_env();

// Validate critical settings
assert!(!config.database_url.is_empty(), "DATABASE_URL must not be empty");
assert!(config.app_port > 0, "PORT must be positive");
```

### Document Settings

Add comments explaining each setting:

```rust
// Rate limiting: 5 seconds between requests to the same site
let rec_default_delay_secs = std::env::var("REC_DEFAULT_DELAY_SECS")
    .unwrap_or_else(|_| "5".to_string())
    .parse().unwrap_or(5);
```

### Use Type Safety

Parse settings into proper types:

```rust
// BAD: Store as string
let port: String = std::env::var("PORT").unwrap_or("3000".to_string());

// GOOD: Parse to integer
let port: u16 = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse().unwrap_or(3000);
```

## Conclusion

FicHub's configuration system provides:
1. Environment-variable-based configuration
2. Sensible defaults for optional settings
3. Type-safe parsing with error handling
4. Comprehensive test coverage
5. Clear documentation
6. Flexible deployment options

This follows the Twelve-Factor App methodology and works seamlessly with Docker, Kubernetes, and traditional deployments.


# Chapter 1 Expansion: Understanding the FicHub Architecture

## The Request Lifecycle

When someone visits `http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/12345`, here's what happens step by step:

1. **DNS Resolution** — The browser looks up the IP address of localhost
2. **TCP Connection** — The browser connects to port 3000
3. **HTTP Request** — The browser sends a GET request with the URL
4. **Axum Routing** — Axum matches the URL pattern `/api/v0/epub` to the `epub_handler`
5. **Extraction** — Axum extracts the query parameter `q` from the URL
6. **Handler Execution** — The `epub_handler` function runs
7. **Scraper Selection** — The handler finds the right scraper for AO3
8. **Metadata Lookup** — The scraper fetches the story page from AO3
9. **Database Storage** — The metadata is stored in PostgreSQL
10. **Cache Check** — The handler checks if we've already generated this EPUB
11. **EPUB Generation** (if needed) — The EPUB file is created
12. **Response** — The handler returns JSON with download links

This entire process typically takes 1-5 seconds for a new story, but only milliseconds for cached stories.

## Module Interactions

Here's how the different modules work together:

```
                    ┌─────────────┐
                    │   main.rs   │
                    └──────┬──────┘
                           │
                    ┌──────▼──────┐
                    │  server.rs  │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
     ┌──────▼──────┐ ┌────▼────┐ ┌───────▼──────┐
     │  routes/    │ │  db/    │ │   scrape/    │
     │  (handlers) │ │(database│ │  (scrapers)  │
     └──────┬──────┘ └────┬────┘ └───────┬──────┘
            │              │              │
     ┌──────▼──────┐ ┌────▼────┐ ┌───────▼──────┐
     │  export/    │ │ cache/  │ │  limiter/    │
     │(EPUB/HTML)  │ │ (disk)  │ │ (rate limit) │
     └─────────────┘ └─────────┘ └──────────────┘
```

When a request comes in:
1. `server.rs` routes it to the right handler in `routes/`
2. The handler might need data from `db/` (database queries)
3. The handler might need to scrape a website using `scrape/`
4. The handler might need to generate files using `export/`
5. The handler checks `cache/` to avoid redundant work
6. The handler respects `limiter/` to avoid overwhelming upstream sites

## The Data Flow

Let's trace the data flow for an EPUB request:

```
User Request
    │
    ▼
┌─────────────────┐
│  Query: q=URL   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Scraper Lookup  │ ──► Find scraper for this URL
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ HTTP GET to AO3 │ ──► Fetch story page
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Parse HTML      │ ──► Extract title, author, etc.
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Store in DB     │ ──► Upsert fic_info
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Check Cache     │ ──► Look up export_log
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
  Hit       Miss
    │         │
    ▼         ▼
┌────────┐ ┌─────────────────┐
│ Return │ │ Fetch Chapters  │
│ cached │ └────────┬────────┘
│ URLs   │          │
└────────┘          ▼
               ┌─────────────────┐
               │ Generate EPUB   │
               └────────┬────────┘
                        │
                        ▼
               ┌─────────────────┐
               │ Save to Cache   │
               └────────┬────────┘
                        │
                        ▼
               ┌─────────────────┐
               │ Return URLs     │
               └─────────────────┘
```

## Design Principles

FicHub follows several important design principles:

### 1. Separation of Concerns

Each module has a single responsibility:
- `scrape/` only scrapes websites
- `export/` only generates files
- `db/` only handles database operations
- `cache/` only manages caching

This makes the code easier to understand, test, and modify.

### 2. Trait-Based Abstraction

FicHub uses traits (interfaces) to define contracts:
- `SiteScraper` — How scrapers must behave
- `RateLimiter` — How rate limiters must behave
- `SiteFetcher` — How recommendation fetchers must behave

This allows swapping implementations without changing the rest of the code.

### 3. Graceful Degradation

If something fails, FicHub tries to continue:
- If Redis is down, rate limiting falls back to static delays
- If a scraper fails, it returns a clear error instead of crashing
- If the cache is corrupted, it regenerates the file

### 4. Defensive Programming

FicHub validates inputs and handles edge cases:
- Empty queries return helpful error messages
- Invalid URLs are rejected with clear errors
- Database errors are logged but hidden from users

## The Tech Stack

FicHub's technology choices:

| Component | Technology | Why |
|-----------|-----------|-----|
| Language | Rust | Performance, safety, modern ecosystem |
| Web Framework | Axum | Type-safe, async, great middleware |
| Database | PostgreSQL | Powerful, reliable, full-text search |
| Cache | Redis | Fast, in-memory, atomic operations |
| HTTP Client | Reqwest | Async, TLS, easy to use |
| HTML Parser | scraper | CSS selectors, fast |
| EPUB Generator | epub-builder | Simple API, well-tested |
| Logging | tracing | Async, structured, fast |
| Serialization | serde | De facto standard for Rust |
| Deployment | Docker | Reproducible, portable |

## Performance Characteristics

FicHub is designed for performance:

- **Concurrent requests** — Tokio handles thousands of simultaneous connections
- **Connection pooling** — 20 database connections ready to use
- **Disk caching** — Cached files served instantly
- **Rate limiting** — Prevents overload on upstream sites
- **Async I/O** — Non-blocking network operations

Typical response times:
- Cached EPUB request: ~10ms
- New EPUB request: ~3-10 seconds (depends on source site)
- Metadata-only request: ~1-2 seconds
- Search query: ~50-100ms
- OPDS feed: ~20-50ms


# Deep Dive: Rust Concepts Used in FicHub

## Ownership and Borrowing

Rust's most unique feature is its **ownership system**. Every piece of data has exactly one owner. When the owner goes out of scope, the data is automatically cleaned up. This prevents memory leaks without needing a garbage collector.

```rust
fn main() {
    let title = String::from("Harry Potter");  // title owns the String
    let reference = &title;                      // reference borrows title
    println!("{}", reference);                   // works fine
    // title is dropped here (memory freed)
}
```

### Borrowing Rules

1. You can have **many immutable references** (`&T`) OR **one mutable reference** (`&mut T`), but not both at the same time
2. References must always be valid (no dangling references)

These rules prevent data races at compile time. If two threads try to modify the same data, the compiler catches it before the program runs.

### How FicHub Uses Ownership

In FicHub, `AppState` is wrapped in `Arc` (Atomic Reference Counted):

```rust
let state = Arc::new(AppState { /* ... */ });
```

`Arc` allows multiple parts of the program to share the same data. Each clone of the `Arc` increments a reference count. When all references are dropped, the data is cleaned up.

## Traits and Dynamic Dispatch

Traits in Rust are like interfaces in Java or TypeScript. They define a set of methods that a type must implement:

```rust
#[async_trait]
pub trait SiteScraper: Send + Sync {
    fn can_handle(&self, url: &str) -> bool;
    async fn lookup(&self, client: &reqwest::Client, url: &str) -> Result<FicMetadata, ScrapeError>;
    async fn fetch_chapters(&self, client: &reqwest::Client, meta: &FicMetadata) -> Result<Vec<Chapter>, ScrapeError>;
}
```

### Static vs Dynamic Dispatch

Rust supports two ways to use traits:

**Static dispatch** (generics) — The compiler creates a separate function for each type:
```rust
fn process_scraper<T: SiteScraper>(scraper: &T, url: &str) {
    // Compiler generates code specific to T
}
```

**Dynamic dispatch** (trait objects) — The program looks up the method at runtime:
```rust
fn process_scraper(scraper: &dyn SiteScraper, url: &str) {
    // Runtime lookup via vtable
}
```

FicHub uses dynamic dispatch in the `ScraperRegistry`:
```rust
scrapers: Vec<Box<dyn SiteScraper>>
```

This allows storing different scraper types in the same vector.

### Send and Sync

The `Send` and `Sync` marker traits are crucial for async code:
- `Send` — The type can be moved to another thread
- `Sync` — The type can be shared between threads

Most types in Rust are `Send + Sync` by default, but some types (like `Rc` or raw pointers) are not.

## Error Handling Patterns

### The ? Operator

The `?` operator is syntactic sugar for error propagation:

```rust
// This:
let data = std::fs::read_to_string("config.json")?;

// Is equivalent to:
let data = match std::fs::read_to_string("config.json") {
    Ok(data) => data,
    Err(e) => return Err(e.into()),
};
```

### Custom Error Types

FicHub's `AppError` enum demonstrates good error design:

```rust
pub enum AppError {
    BadRequest(i32, String),    // Client error
    RateLimited(u64),           // Throttling
    NotFound(String),           // Missing resource
    Internal(String),           // Server error
    ScrapeError(String),        // Upstream error
    ExportError(String),        // Generation error
    Database(String),           // Storage error
    CacheError(String),         // Cache error
}
```

Each variant provides enough context to:
1. Generate an appropriate HTTP status code
2. Log useful debug information
3. Return a safe error message to the client

### The From Trait

The `From` trait enables automatic error conversion:

```rust
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}
```

This means `?` on a `sqlx::Result` automatically converts to `AppResult`.

## Async/Await in Rust

### How Async Works

Async functions in Rust are compiled into state machines. When you call an async function, it returns a `Future` that can be polled by the runtime:

```rust
async fn fetch_story(url: &str) -> Result<FicMetadata, ScrapeError> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    // ...
}
```

The `.await` points are where the function can suspend. Between these points, the runtime can execute other tasks.

### Tokio's Role

Tokio is the runtime that:
1. Manages the thread pool
2. Schedules tasks
3. Handles I/O events
4. Provides timers and synchronization primitives

Without Tokio, async functions in Rust wouldn't actually run. The `#[tokio::main]` attribute sets up the runtime automatically.

### Concurrent vs Parallel

- **Concurrent** — Multiple tasks making progress (interleaved execution)
- **Parallel** — Multiple tasks running simultaneously (on multiple CPU cores)

Tokio provides both. By default, it uses a thread pool with as many threads as CPU cores.

## Pattern Matching

Pattern matching in Rust is powerful and exhaustive:

```rust
match rate_limit_result {
    RateLimitResult::Allowed => {
        // Process the request
    }
    RateLimitResult::Wait(seconds) => {
        // Return 429 with retry-after
    }
    RateLimitResult::Blocked => {
        // Return 403
    }
}
```

### Exhaustiveness

The compiler ensures you handle every variant. If you add a new variant to `RateLimitResult` without updating the match, the program won't compile. This prevents bugs from forgotten cases.

### Destructuring

Pattern matching can destructure complex types:

```rust
match error {
    AppError::BadRequest(code, msg) => {
        // code and msg are extracted automatically
    }
    _ => {} // Catch-all for other variants
}
```

## Closures and Iterators

FicHub makes heavy use of closures and iterators:

```rust
// Closure: anonymous function
let is_complete = |status: &str| status == "complete";

// Iterator chain
let titles: Vec<String> = fics.iter()
    .filter(|fic| fic.words > 10_000)
    .map(|fic| fic.title.clone())
    .collect();
```

### Why Iterators?

Iterators in Rust are **zero-cost abstractions** — they compile down to the same code as manual loops. But they're much more readable and composable.

### Common Iterator Methods

| Method | What It Does |
|--------|-------------|
| `filter` | Keep only elements matching a condition |
| `map` | Transform each element |
| `collect` | Convert iterator to a collection |
| `enumerate` | Add index to each element |
| `take` | Take only N elements |
| `skip` | Skip N elements |
| `any` | Check if any element matches |
| `find` | Find first matching element |
| `fold` | Accumulate into a single value |

## Smart Pointers

### Box<T>

`Box<T>` is a heap-allocated pointer. It's used for:
- Trait objects: `Box<dyn SiteScraper>`
- Recursive types
- Large data that shouldn't be on the stack

### Arc<T>

`Arc<T>` (Atomic Reference Counted) allows shared ownership:
```rust
let state = Arc::new(AppState { /* ... */ });
let state_clone = state.clone();  // Increments reference count
```

### Mutex<T>

`Mutex<T>` provides interior mutability with thread safety:
```rust
let semaphores: Arc<Mutex<HashMap<...>>> = Arc::new(Mutex::new(HashMap::new()));
let mut map = semaphores.lock().await;
map.insert(key, value);
```

### RwLock<T>

`RwLock<T>` allows multiple readers OR one writer:
```rust
let datacenter_ips: Arc<RwLock<HashSet<IpAddr>>> = ...;
// Multiple readers can access simultaneously
let ips = datacenter_ips.read().await;
// Only one writer at a time
let mut ips = datacenter_ips.write().await;
```

## Concurrency Patterns

### The Semaphore Pattern

Used in FicHub to prevent duplicate exports:

```rust
let sem = Arc::new(Semaphore::new(1));  // Only 1 concurrent task
let permit = sem.acquire().await?;
// ... do work ...
drop(permit);  // Release the semaphore
```

### The Double-Check Pattern

Used in FicHub's export handler:

```rust
// Check 1: Before acquiring lock
if let Some(cached) = check_cache().await {
    return cached;
}

// Acquire lock
let _permit = semaphore.acquire().await?;

// Check 2: After acquiring lock
if let Some(cached) = check_cache().await {
    return cached;
}

// Generate the file
generate_and_cache().await
```

### The Atomic CAS Pattern

Used in FicHub's rate limiter:

```rust
loop {
    let last = self.last_request.load(Ordering::Acquire);
    let now = current_time();
    
    if now - last >= delay {
        if self.last_request.compare_exchange(
            last, now, Ordering::AcqRel, Ordering::Relaxed
        ).is_ok() {
            return;  // Successfully claimed the slot
        }
        // CAS failed — another thread claimed it, retry
    } else {
        sleep(remaining_time).await;
    }
}
```

This ensures only one thread can claim a time slot, even with multiple concurrent callers.


# Deep Dive: Testing FicHub

## Why Test?

Tests are like safety nets for your code. They catch bugs before they reach users. In FicHub, there are several types of tests:

1. **Unit tests** — Test individual functions in isolation
2. **Integration tests** — Test how modules work together
3. **HTTP tests** — Test API endpoints end-to-end

## Unit Testing in FicHub

### Testing Configuration

FicHub's config module has thorough tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Serializes all env-var-manipulating tests to prevent cross-pollution
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        keys: Vec<String>,
    }

    impl EnvGuard {
        fn new() -> Self { EnvGuard { keys: Vec::new() } }
        
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
        assert!(config.dynamic_rate_limit);
    }

    #[test]
    #[should_panic(expected = "DATABASE_URL must be set")]
    fn test_panics_without_database_url() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_config_env();
        let mut guard = EnvGuard::new();
        guard.set("REDIS_URL", "redis://localhost");
        let _ = Config::from_env();
    }
}
```

### Key Testing Patterns

1. **Mutex for env vars** — Tests that modify environment variables must be serialized
2. **EnvGuard for cleanup** — Automatically removes env vars after each test
3. **should_panic** — Tests that verify expected panics
4. **Assert macros** — `assert_eq!`, `assert!`, `assert_ne!` for checking results

### Testing URL ID Generation

```rust
#[test]
fn test_generate_url_id_deterministic() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(1, "story_123");
    assert_eq!(id1, id2);  // Same input = same output
}

#[test]
fn test_generate_url_id_different_source_id() {
    let id1 = generate_url_id(1, "story_123");
    let id2 = generate_url_id(2, "story_123");
    assert_ne!(id1, id2);  // Different source = different ID
}

#[test]
fn test_generate_url_id_length() {
    let id = generate_url_id(42, "abc123");
    assert_eq!(id.len(), 12);  // Always 12 hex chars
}

#[test]
fn test_generate_url_id_hex_chars() {
    let id = generate_url_id(7, "test_url");
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}
```

### Testing the Token Bucket

FicHub includes a pure-Rust token bucket implementation for testing:

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
            -1.0  // Allowed
        } else {
            (requested - new_tokens) / self.flow  // Wait time
        }
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
    bucket.request(50.0, 0.0);  // Empty the bucket
    assert!((bucket.value - 0.0).abs() < f64::EPSILON);

    // After 3s -> 30 tokens refilled (capped at 50)
    let r = bucket.request(20.0, 3.0);
    assert!((r - (-1.0)).abs() < f64::EPSILON);
    assert!((bucket.value - 10.0).abs() < f64::EPSILON);  // 30 - 20 = 10
}

#[test]
fn test_wait_calculation() {
    let mut bucket = TokenBucket::new(10.0, 2.0, 0.0);
    bucket.request(10.0, 0.0);  // Empty
    let wait = bucket.request(5.0, 0.0);  // Need 5 tokens, 0 available
    let expected = 5.0 / 2.0;  // 2.5 seconds
    assert!((wait - expected).abs() < f64::EPSILON);
}
```

### Testing Error Types

```rust
#[test]
fn test_display_bad_request() {
    let err = AppError::BadRequest(400, "invalid input".into());
    let s = format!("{}", err);
    assert!(s.contains("BadRequest"));
    assert!(s.contains("400"));
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
```

## Integration Testing

Integration tests verify that modules work together correctly.

### Testing Scraper Traits

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
```

### Testing Cache Paths

```rust
#[test]
fn test_cache_path_short_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Epub, "abc", "hash123");
    assert_eq!(path, Path::new("/cache/epub/abc/abc/hash123.epub"));
}

#[test]
fn test_cache_path_long_url_id() {
    let root = Path::new("/cache");
    let path = cache_path(root, &EType::Mobi, "abcdefghijklm", "h1");
    assert_eq!(path, Path::new("/cache/mobi/abc/def/ghi/abcdefghijklm/h1.mobi"));
}

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
```

## Testing HTTP Handlers

FicHub uses `axum-test` for testing HTTP handlers:

```rust
#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_api_docs_handler() {
        let response = api_docs_handler().await;
        // Verify the response contains expected fields
    }
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_generate_url_id

# Run tests in a specific module
cargo test db::queries::tests

# Run tests and show println! output
cargo test -- --show-output

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

## Test Coverage

While FicHub doesn't have formal coverage tracking, the test suite covers:
- Configuration loading and validation
- URL ID generation (determinism, length, format)
- Error type conversions
- Cache path computation
- Token bucket algorithm
- Tag resolution logic
- Search query building
- Export version computation
- EPUB generation (with mocked data)
- Rate limiting logic

## Writing Good Tests

### Test Naming Convention

```rust
#[test]
fn test_<what_is_being_tested>() {
    // Arrange: Set up test data
    // Act: Perform the operation
    // Assert: Verify the result
}
```

### Isolation

Each test should be independent:
- Don't depend on other tests running first
- Don't share state between tests
- Clean up after yourself (temp files, env vars)

### Edge Cases

Good tests cover:
- Normal cases (happy path)
- Edge cases (empty input, maximum values)
- Error cases (invalid input, missing resources)
- Concurrency (if applicable)

## Performance Testing

FicHub's performance can be tested with tools like `wrk` or `hey`:

```bash
# Install hey
cargo install hey

# Benchmark the API
hey -n 1000 -c 10 http://localhost:3000/api/v0/meta?q=https://archiveofourown.org/works/12345

# Benchmark with concurrent requests
hey -n 10000 -c 50 http://localhost:3000/api/v0/epub?q=https://archiveofourown.org/works/12345
```

## Common Testing Pitfalls

1. **Flaky tests** — Tests that sometimes pass and sometimes fail. Usually caused by timing issues or shared state.

2. **Slow tests** — Tests that take too long to run. FicHub mitigates this by using unit tests for most logic and only integration testing critical paths.

3. **Missing cleanup** — Tests that create files or modify env vars without cleaning up. The `EnvGuard` pattern helps with this.

4. **Over-mocking** — Testing implementation details instead of behavior. Focus on what the code does, not how it does it.


# Deep Dive: Database Design and SQL Patterns

## Database Schema Overview

FicHub's PostgreSQL database has several interconnected tables. Let's explore each one and understand how they relate to each other.

### Core Tables

#### fic_info — The Main Story Table

```sql
CREATE TABLE IF NOT EXISTS fic_info (
    id TEXT PRIMARY KEY,
    created TIMESTAMPTZ,
    updated TIMESTAMPTZ,
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    author_url TEXT,
    author_local_id TEXT,
    chapters INTEGER NOT NULL DEFAULT 1,
    words BIGINT NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT '',
    fic_created TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    fic_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status TEXT NOT NULL DEFAULT 'ongoing',
    source TEXT NOT NULL,
    extra_meta TEXT,
    raw_extended_meta TEXT,
    source_id BIGINT,
    author_id BIGINT,
    content_hash TEXT,
    -- Full-text search vector
    text_search tsvector GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(author, '')), 'B') ||
        setweight(to_tsvector('english', coalesce(description, '')), 'C')
    ) STORED
);
```

The `text_search` column is a **generated column** — PostgreSQL automatically updates it whenever the title, author, or description changes. The weights (A, B, C) mean title matches are ranked higher than author matches, which are ranked higher than description matches.

#### export_log — Cache Tracking

```sql
CREATE TABLE IF NOT EXISTS export_log (
    url_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    etype TEXT NOT NULL,
    input_hash TEXT NOT NULL,
    export_hash TEXT NOT NULL,
    created TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, version, etype, input_hash)
);
```

This table tracks which exports have been generated. The composite primary key ensures each unique combination of story, version, format, and input hash is stored once.

#### fic_tags — Tag Associations

```sql
CREATE TABLE IF NOT EXISTS fic_tags (
    url_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    added_by_ip INET,
    score SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (url_id, tag_id)
);
```

This is a junction table connecting stories to tags. The `score` field tracks the community vote score for this tag on this story.

#### fic_tag_votes — Individual Votes

```sql
CREATE TABLE IF NOT EXISTS fic_tag_votes (
    url_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    voter_ip INET NOT NULL,
    value SMALLINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (url_id, tag_id, voter_ip)
);
```

Each vote is recorded with the voter's IP address. One vote per IP per tag per story.

#### fic_bookmarks — User Bookmarks (for Recommendations)

```sql
CREATE TABLE IF NOT EXISTS fic_bookmarks (
    user_hash TEXT NOT NULL,
    url_id TEXT NOT NULL,
    site_domain TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (user_hash, url_id)
);
```

User identities are hashed for privacy. This table tracks which users have bookmarked which stories.

#### fic_bookmark_cooccur — Co-occurrence Matrix

```sql
CREATE TABLE IF NOT EXISTS fic_bookmark_cooccur (
    work_a TEXT NOT NULL,
    work_b TEXT NOT NULL,
    cooccur_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (work_a, work_b)
);
```

This is the heart of the recommendation engine. It tracks how many users have bookmarked both work_a and work_b. The Jaccard similarity is computed from these counts.

### Relationship Diagram

```
fic_info ──────────────┐
    │                   │
    ├── fic_tags ───────┤
    │       │           │
    │       └── tags    │
    │                   │
    ├── fic_tag_votes   │
    │                   │
    ├── export_log      │
    │                   │
    ├── fic_blacklist   │
    │                   │
    ├── fic_works ──────┤
    │       │           │
    │       └── fic_bookmark_cooccur
    │                   │
    └── fic_bookmarks   │
                        │
request_source ─────────┘
request_log
```

## Common Query Patterns

### UPSERT Pattern

Used throughout FicHub for inserting or updating records:

```sql
INSERT INTO fic_info (id, title, author, ...)
VALUES ($1, $2, $3, ...)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    author = EXCLUDED.author,
    updated = NOW()
```

The `EXCLUDED` keyword refers to the values that were attempted to be inserted. This is more efficient than checking if the row exists first.

### EXISTS Pattern

Used for checking if a relationship exists:

```sql
SELECT EXISTS(
    SELECT 1 FROM fic_tags ft
    JOIN tags t ON t.id = ft.tag_id
    WHERE ft.url_id = $1
    AND t.name = $2
)
```

`EXISTS` is faster than `COUNT(*)` when you only need to know if at least one row matches.

### CTE Pattern

Common Table Expressions make complex queries readable:

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
    (c.cooccur_count::float / (s.favouriter_count + fw.favouriter_count - c.cooccur_count)) AS jaccard
FROM candidates c
JOIN fic_works fw ON fw.url_id = c.candidate_id
CROSS JOIN seed s
WHERE c.candidate_id != $1
ORDER BY jaccard DESC
LIMIT $2
```

### Dynamic Query Building

FicHub's search uses `sqlx::QueryBuilder` for dynamic queries:

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

This builds SQL safely, preventing injection attacks while allowing dynamic filters.

## Indexing Strategy

Proper indexes are crucial for performance:

```sql
-- Primary key indexes (automatic)
-- fic_info(id)
-- export_log(url_id, version, etype, input_hash)

-- Full-text search index
CREATE INDEX idx_fic_info_text_search ON fic_info USING GIN(text_search);

-- Common query patterns
CREATE INDEX idx_fic_info_updated ON fic_info(fic_updated DESC);
CREATE INDEX idx_fic_info_words ON fic_info(words);
CREATE INDEX idx_fic_info_status ON fic_info(status);
CREATE INDEX idx_fic_info_source ON fic_info(source);

-- Tag queries
CREATE INDEX idx_fic_tags_url_id ON fic_tags(url_id);
CREATE INDEX idx_fic_tags_tag_id ON fic_tags(tag_id);
CREATE INDEX idx_fic_tags_score ON fic_tags(url_id, score DESC);

-- Recommendation queries
CREATE INDEX idx_fic_bookmarks_user ON fic_bookmarks(user_hash);
CREATE INDEX idx_fic_bookmarks_work ON fic_bookmarks(url_id);
CREATE INDEX idx_fic_cooccur_works ON fic_bookmark_cooccur(work_a, work_b);
```

### GIN Index for Full-Text Search

The GIN (Generalized Inverted Index) is essential for full-text search performance. Without it, every search would scan every row in the table.

### Partial Indexes

For frequently filtered queries:

```sql
-- Only index completed stories
CREATE INDEX idx_fic_info_complete ON fic_info(fic_updated DESC)
WHERE status = 'complete';

-- Only index recent stories
CREATE INDEX idx_fic_info_recent ON fic_info(fic_updated DESC)
WHERE fic_updated > NOW() - INTERVAL '30 days';
```

## Transaction Safety

FicHub uses transactions for operations that must be atomic:

```rust
pub async fn merge_tags(pool: &PgPool, source_tag_id: i32, target_tag_id: i32) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    // All these operations succeed or fail together
    sqlx::query("INSERT INTO fic_tags ...").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM fic_tags WHERE tag_id = $1").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM tags WHERE id = $1").execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}
```

Without transactions, a failure midway through could leave the database in an inconsistent state.

## Connection Pool Tuning

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)        // How many concurrent connections
    .acquire_timeout(Duration::from_secs(10))  // How long to wait for a connection
    .min_connections(5)         // Keep at least 5 connections ready
    .idle_timeout(Duration::from_secs(300))  // Close idle connections after 5 min
    .max_lifetime(Duration::from_secs(1800))  // Recycle connections after 30 min
    .connect(database_url)
    .await?;
```

### Choosing Pool Size

The optimal pool size depends on:
- Number of concurrent requests
- Average query execution time
- Available memory

Rule of thumb: `pool_size = (number_of_cpu_cores * 2) + number_of_disk_spindles`

For a typical server: `(4 * 2) + 1 = 9` connections. FicHub uses 20 for headroom.

## Backup and Recovery

### Automated Backups

```bash
#!/bin/bash
# backup.sh — Run daily via cron
BACKUP_DIR="/backups/fichub"
DATE=$(date +%Y%m%d_%H%M%S)

# PostgreSQL backup
docker compose exec -T db pg_dump -U fichub fichub | gzip > "$BACKUP_DIR/db_$DATE.sql.gz"

# Cache backup
tar -czf "$BACKUP_DIR/cache_$DATE.tar.gz" -C /var/lib/docker/volumes/fichub_fichub_cache .

# Keep only last 30 days
find "$BACKUP_DIR" -name "*.gz" -mtime +30 -delete
```

### Recovery

```bash
# Restore PostgreSQL
gunzip -c backup_20240115.sql.gz | docker compose exec -T db psql -U fichub fichub

# Restore cache
tar -xzf cache_20240115.tar.gz -C /var/lib/docker/volumes/fichub_fichub_cache
```

## Performance Monitoring

### Slow Query Logging

Enable in PostgreSQL:

```sql
ALTER SYSTEM SET log_min_duration_statement = 1000;  -- Log queries > 1 second
ALTER SYSTEM SET log_statement = 'none';
SELECT pg_reload_conf();
```

### Query Analysis

```sql
EXPLAIN ANALYZE
SELECT fi.*, ts_rank(fi.text_search, plainto_tsquery('english', 'harry potter')) AS rank
FROM fic_info fi
WHERE fi.text_search @@ plainto_tsquery('english', 'harry potter')
ORDER BY rank DESC
LIMIT 20;
```

This shows the query plan and actual execution time, helping identify bottlenecks.


# Deep Dive: Web Scraping Techniques

## Understanding HTML Structure

When FicHub scrapes a fanfiction website, it receives HTML like this:

```html
<html>
<head><title>My Story - FanFiction.net</title></head>
<body>
  <div id="profile_top">
    <b class="xcontrast_txt">My Amazing Story</b>
    <a class="xcontrast_txt" href="/u/12345/AuthorName">AuthorName</a>
    <div class="xcontrast_txt">A story about...</div>
    <span class="xgray">Rated: T | Reviews: 42 | Words: 50000 | Chapters: 10</span>
  </div>
  <div class="storytext">
    <p>Once upon a time...</p>
  </div>
</body>
</html>
```

The scraper needs to navigate this structure and extract specific pieces of information.

## CSS Selectors Explained

CSS selectors are patterns used to select HTML elements. Here are the most common ones:

### Basic Selectors

| Selector | Meaning | Example |
|----------|---------|---------|
| `tagname` | Element by tag | `div`, `p`, `a` |
| `.classname` | Element by class | `.storytext`, `.title` |
| `#idname` | Element by ID | `#profile_top` |
| `*` | Any element | `*` |

### Combinators

| Selector | Meaning | Example |
|----------|---------|---------|
| `A B` | B inside A | `div.storytext p` |
| `A > B` | B direct child of A | `div > p` |
| `A + B` | B immediately after A | `h2 + p` |
| `A ~ B` | B anywhere after A | `h2 ~ p` |

### Attribute Selectors

| Selector | Meaning | Example |
|----------|---------|---------|
| `[attr]` | Has attribute | `[href]` |
| `[attr=val]` | Attribute equals | `[class="title"]` |
| `[attr~=val]` | Attribute contains word | `[class~="title"]` |
| `[attr*=val]` | Attribute contains | `[href*="archiveofourown"]` |

### FicHub's CSS Selectors

Let's look at each scraper's selectors:

#### AO3 Selectors

```rust
// Title
Selector::parse("h2.title.heading")

// Author
Selector::parse("a[rel='author']")

// Description
Selector::parse("blockquote.userstuff")

// Chapter count
Selector::parse("dd.chapters")

// Word count
Selector::parse("dd.words")

// Story status
Selector::parse("dd.status")

// Chapter content
Selector::parse("div.chapter")

// Chapter title
Selector::parse("h3.title")

// Chapter body
Selector::parse("div.userstuff")
```

#### FF.net Selectors

```rust
// Title
Selector::parse("#profile_top b.xcontrast_txt")

// Author
Selector::parse("#profile_top a.xcontrast_txt")

// Description
Selector::parse("#profile_top div.xcontrast_txt")

// Word count
Selector::parse("#profile_top span[data-xutitle='word count']")

// Chapter count
Selector::parse("#profile_top span[data-xutitle='chapters']")

// Chapter content
Selector::parse("div.storytext")

// Chapter select dropdown
Selector::parse("select#chap_select option[selected]")
```

## Handling Different HTML Structures

### Text vs Inner HTML

There's an important difference between `.text()` and `.inner_html()`:

```rust
// For: <p>Hello <b>World</b></p>

element.text()         // Returns: "Hello World" (just the text)
element.inner_html()   // Returns: "Hello <b>World</b>" (includes HTML tags)
```

FicHub uses `inner_html()` for story content (to preserve formatting) and `text()` for metadata (like titles and author names).

### Handling Missing Elements

HTML pages might not always have the expected structure. FicHub handles this with `unwrap_or_else`:

```rust
let title = document
    .select(&Selector::parse("h2.title.heading").unwrap())
    .next()  // Returns Option<Element>
    .map(|el| el.text().collect::<String>().trim().to_string())
    .unwrap_or_else(|| "Unknown Title".to_string());  // Fallback
```

If the element isn't found, `next()` returns `None`, and we use the fallback value.

### Handling Malformed HTML

The `scraper` crate uses the `html5ever` parser, which is very forgiving. It handles:
- Unclosed tags
- Incorrect nesting
- Missing attributes
- Invalid characters

This is important because fanfiction sites often have slightly malformed HTML.

## Network Requests

### Setting User-Agent

Always set a descriptive User-Agent:

```rust
let response = client
    .get(&url)
    .header("User-Agent", "fichub.net/0.1.0")
    .send()
    .await
    .map_err(|e| ScrapeError::Network(e.to_string()))?;
```

Some sites block requests without a User-Agent or with suspicious ones (like "python-requests").

### Handling HTTP Status Codes

```rust
if !response.status().is_success() {
    return Err(ScrapeError::NotFound);
}
```

FicHub treats any non-2xx status as "not found." In production, you might want to distinguish between 404 (not found), 403 (forbidden), and 500 (server error).

### Timeout Configuration

```rust
let http_client = reqwest::Client::builder()
    .user_agent("fichub.net/0.1.0")
    .timeout(std::time::Duration::from_secs(30))
    .build()
    .expect("Failed to build HTTP client");
```

The 30-second timeout prevents hanging on slow responses. Individual requests can also have their own timeouts.

## Rate Limiting While Scraping

### Per-Site Delays

FicHub adds delays between requests to the same site:

```rust
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
```

The default delay is 5 seconds between requests to the same site. This can be configured per-site via `REC_SITE_RATE_LIMITS`.

### Respecting Robots.txt

In production, scrapers should check `robots.txt` to see which URLs are allowed:

```
User-agent: fichub.net
Disallow: /private/
Allow: /works/
```

FicHub currently doesn't check robots.txt, but this would be a good improvement.

## Extracting Tags from HTML

Some sites expose tags in their HTML. For example, AO3 shows tags like:

```html
<ul class="tags">
    <li><a href="/tags/Harry%20Potter">Harry Potter</a></li>
    <li><a href="/tags/Hermione%20Granger">Hermione Granger</a></li>
    <li><a href="/relationships/Harry%20Potter%20%7C%20Hermione%20Granger">Harry/Hermione</a></li>
</ul>
```

The `extract_tags` method parses these into `ExtractedTag` structs:

```rust
async fn extract_tags(&self, client: &reqwest::Client, url: &str) -> Result<Vec<ExtractedTag>, ScrapeError> {
    let html = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&html);
    
    let mut tags = Vec::new();
    
    // Extract fandom tags
    let fandom_sel = Selector::parse("ul.tags li a[href*='/tags/']").unwrap();
    for el in document.select(&fandom_sel) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::fandom(&name));
        }
    }
    
    // Extract relationship tags
    let rel_sel = Selector::parse("ul.tags li a[href*='/relationships/']").unwrap();
    for el in document.select(&rel_sel) {
        let name = el.text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            tags.push(ExtractedTag::relationship(&name));
        }
    }
    
    Ok(tags)
}
```

## Error Recovery

### Retry Logic

For transient errors (like network timeouts), FicHub could implement retry logic:

```rust
async fn fetch_with_retry(client: &Client, url: &str, max_retries: u32) -> Result<String, ScrapeError> {
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match client.get(url).send().await {
            Ok(response) if response.status().is_success() => {
                return response.text().await.map_err(|e| ScrapeError::Network(e.to_string()));
            }
            Ok(_) => {
                last_error = Some(ScrapeError::NotFound);
            }
            Err(e) => {
                last_error = Some(ScrapeError::Network(e.to_string()));
            }
        }
        
        if attempt < max_retries - 1 {
            tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
        }
    }
    
    Err(last_error.unwrap_or(ScrapeError::NotFound))
}
```

### Graceful Degradation

If a scraper fails, FicHub returns a clear error instead of crashing:

```rust
let meta = scraper.lookup(&state.http_client, query).await
    .map_err(|e| AppError::ScrapeError(e.to_string()))?;
```

The error message includes details about what went wrong, helping with debugging.

## Testing Scrapers

### Unit Testing Selectors

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use scraper::{Html, Selector};
    
    #[test]
    fn test_extract_title_from_ao3() {
        let html = r#"<html><body>
            <h2 class="title heading">Test Story</h2>
        </body></html>"#;
        
        let document = Html::parse_document(html);
        let title = document
            .select(&Selector::parse("h2.title.heading").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        
        assert_eq!(title, "Test Story");
    }
    
    #[test]
    fn test_extract_title_missing() {
        let html = r#"<html><body>
            <p>No title here</p>
        </body></html>"#;
        
        let document = Html::parse_document(html);
        let title = document
            .select(&Selector::parse("h2.title.heading").unwrap())
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        
        assert_eq!(title, "Unknown Title");
    }
}
```

## Performance Considerations

### Connection Reuse

Reqwest reuses TCP connections by default (connection pooling). This is much faster than creating a new connection for each request.

### HTML Parsing Performance

The `scraper` crate is fast because it uses `html5ever`, which is written in Rust and optimized for speed. For a typical fanfiction page (100KB HTML), parsing takes about 1-5 milliseconds.

### Memory Usage

Large fanfiction stories can have substantial HTML content. FicHub processes chapters one at a time to keep memory usage low:

```rust
for chapter in chapters {
    let chapter_html = format!(/* ... */, content = chapter.content);
    builder.add_content(EpubContent::new(/* ... */))?;
}
```

Each chapter is processed and added to the EPUB builder, then the chapter data can be dropped from memory.


# Deep Dive: The Axum Web Framework

## How Axum Works

Axum is a web framework built on top of Tower. It's designed for type safety and composability. Let's explore how it works in depth.

### The Request-Response Cycle

When a request arrives at FicHub:

1. **TCP Connection** — Tokio accepts the connection
2. **HTTP Parsing** — Hyper parses the HTTP request
3. **Middleware Chain** — TraceLayer, CorsLayer process the request
4. **Route Matching** — Axum finds the matching route
5. **Extraction** — Axum extracts data from the request (state, query params, etc.)
6. **Handler Execution** — The handler function runs
7. **Response Generation** — The handler returns a response
8. **Response Sending** — Hyper sends the HTTP response

### Extractors in Detail

Extractors are how Axum gets data from requests. They implement the `FromRequest` trait:

#### State Extractor

```rust
async fn handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    // state is the shared AppState
    let db_pool = &state.db;
    // ...
}
```

#### Query Extractor

```rust
#[derive(Deserialize)]
struct MyQuery {
    q: Option<String>,
    page: Option<u32>,
}

async fn handler(Query(params): Query<MyQuery>) -> Json<Value> {
    let query = params.q.unwrap_or_default();
    let page = params.page.unwrap_or(1);
    // ...
}
```

#### Path Extractor

```rust
async fn handler(Path((etype, url_id)): Path<(String, String)>) -> Json<Value> {
    // etype and url_id are extracted from the URL path
    // e.g., /cache/epub/abc123 -> etype="epub", url_id="abc123"
}
```

#### JSON Body Extractor

```rust
#[derive(Deserialize)]
struct SuggestBody {
    url_id: String,
    suggested_url: String,
    comment: Option<String>,
}

async fn handler(Json(body): Json<SuggestBody>) -> Json<Value> {
    // body is parsed from the request body as JSON
    let url_id = body.url_id;
    // ...
}
```

#### ConnectInfo Extractor

```rust
async fn handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>
) -> Json<Value> {
    // addr is the client's IP address
    let ip = addr.ip();
    // ...
}
```

### Multiple Extractors

A handler can have multiple extractors:

```rust
async fn my_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MyQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<Value>, AppError> {
    // All extractors are available
}
```

Extractors are processed in order. If one fails (e.g., invalid JSON), the error is returned immediately.

### Response Types

Handlers can return different response types:

```rust
// JSON response
async fn json_handler() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

// Plain text response
async fn text_handler() -> PlainText<String> {
    PlainText("Hello, world!".to_string())
}

// HTML response
async fn html_handler() -> Html<String> {
    Html("<h1>Hello, world!</h1>".to_string())
}

// Redirect
async fn redirect_handler() -> Redirect {
    Redirect::to("/new-location")
}

// Custom status code
async fn status_handler() -> StatusCode {
    StatusCode::NO_CONTENT
}

// Tuple response (status + headers + body)
async fn tuple_handler() -> (StatusCode, [(HeaderName, &'static str); 1], String) {
    (
        StatusCode::OK,
        [("X-Custom", "value")],
        "response body".to_string(),
    )
}
```

### The IntoResponse Trait

Any type that implements `IntoResponse` can be returned from a handler:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::BadRequest(code, msg) => {
                (StatusCode::BAD_REQUEST, json!({"err": code, "msg": msg}))
            }
            // ...
        };
        (status, Json(body)).into_response()
    }
}
```

### Middleware

Middleware adds behavior to all requests. Axum uses Tower middleware:

```rust
let app = Router::new()
    .route("/api", get(handler))
    .layer(TraceLayer::new_for_http())  // Logs requests
    .layer(CorsLayer::permissive())      // Allows CORS
    .layer(CompressionLayer::new());     // Compresses responses
```

#### TraceLayer

Logs information about every request:

```
2024-01-15T10:30:00Z INFO request{method=GET uri=/api/v0/epub status=200}: fichub::server
```

#### CorsLayer

Handles Cross-Origin Resource Sharing:

```rust
// Permissive (allows all origins)
.layer(CorsLayer::permissive())

// Custom
.layer(CorsLayer::new()
    .allow_origin("http://localhost:3000".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any)
)
```

#### CompressionLayer

Compresses response bodies:

```rust
.layer(CompressionLayer::new()
    .gzip(true)
    .br(true)
    .zstd(true)
)
```

### State Management

State is shared across all handlers using `Arc`:

```rust
// Create state
let state = Arc::new(AppState { /* ... */ });

// Use in router
let app = Router::new()
    .route("/api", get(handler))
    .with_state(state);

// Access in handler
async fn handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db = &state.db;
    // ...
}
```

### Nested Routers

Axum supports nested routers for organizing code:

```rust
let api_routes = Router::new()
    .route("/epub", get(epub_handler))
    .route("/meta", get(meta_handler))
    .route("/search", get(search_handler));

let opds_routes = Router::new()
    .route("/", get(root_catalog))
    .route("/new", get(recent_feed));

let app = Router::new()
    .nest("/api/v0", api_routes)
    .nest("/opds", opds_routes);
```

### Error Handling

Axum uses the `IntoResponse` trait for error handling:

```rust
// Handler returns Result<T, E> where E: IntoResponse
async fn handler() -> Result<Json<Value>, AppError> {
    let data = do_something().await?;  // ? uses From trait for conversion
    Ok(Json(json!({"data": data})))
}
```

### Testing Axum Handlers

FicHub uses `axum-test` for testing:

```rust
#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handler() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .with_state(create_test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### Performance Tips

1. **Use `Arc` for shared state** — Avoid cloning large structs
2. **Minimize middleware** — Each layer adds overhead
3. **Use connection pooling** — For database and HTTP connections
4. **Enable compression** — Reduces response size
5. **Use `fetch_optional` over `fetch_one`** — When the result might not exist
6. **Cache frequently accessed data** — In Redis or memory

## Tower: The Foundation

Tower is the library that powers Axum's middleware system. It provides:

- **Service trait** — The core abstraction for request handling
- **Layer trait** — For adding middleware
- **Buffer** — For backpressure
- **Timeout** — For request timeouts
- **Retry** — For automatic retries

### How Tower Layers Work

```
Request → Layer 1 → Layer 2 → Layer 3 → Handler → Layer 3 → Layer 2 → Layer 1 → Response
```

Each layer wraps the next, adding behavior on the way in and out.

### Custom Middleware

You can create custom middleware:

```rust
use tower::{Service, Layer};
use std::task::{Context, Poll};

#[derive(Clone)]
struct MyMiddleware<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for MyMiddleware<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            // Before handler
            println!("Request: {} {}", req.method(), req.uri());
            
            let response = inner.call(req).await?;
            
            // After handler
            println!("Response: {}", response.status());
            
            Ok(response)
        })
    }
}
```

## Real-World Examples

### The Export Handler Flow

Let's trace through the export handler step by step:

```rust
pub async fn epub_handler(
    State(state): State<Arc<AppState>>,  // Extract shared state
    Query(params): Query<ExportQuery>,    // Extract query parameters
) -> Result<Json<Value>, AppError> {
    // 1. Validate input
    let query = params.q.as_deref().unwrap_or("");
    if query.is_empty() {
        return Ok(Json(json!({"err": -1, "msg": "no query"})));
    }

    // 2. Find scraper
    let scraper = state.scraper_registry.find_scraper(query)
        .ok_or_else(|| AppError::BadRequest(-5, "unsupported URL".into()))?;

    // 3. Fetch metadata
    let meta = scraper.lookup(&state.http_client, query).await?;

    // 4. Store in database
    queries::upsert_fic_info(&state.db, &fic_info_row).await?;

    // 5. Check blacklists
    let blacklist = queries::check_fic_blacklist(&state.db, &meta.url_id).await?;
    if !blacklist.is_empty() {
        return Ok(Json(json!({"err": -7, "msg": "blacklisted"})));
    }

    // 6. Check cache
    let cached = queries::find_export_log(&state.db, &meta.url_id, version, "epub", &hash).await?;
    if let Some(log) = cached {
        return Ok(build_cached_response(&meta, &log));
    }

    // 7. Generate EPUB
    let chapters = scraper.fetch_chapters(&state.http_client, &meta).await?;
    let (path, hash) = export::epub::create_epub(&meta, &chapters, &state.config.tmp_dir).await?;

    // 8. Save to cache
    let dest = cache::disk::cache_path(&state.config.cache_dir, &EType::Epub, &meta.url_id, &hash);
    cache::disk::move_to_cache(&path, &dest)?;

    // 9. Record in database
    queries::insert_export_log(&state.db, &meta.url_id, version, "epub", &input_hash, &hash).await?;

    // 10. Return response
    Ok(Json(json!({
        "err": 0,
        "url_id": meta.url_id,
        "urls": { "epub": format!("/cache/epub/{}?h={}", meta.url_id, hash) },
    })))
}
```

Each step is designed to be idempotent — running it multiple times produces the same result. This is important for reliability.

## Conclusion

Axum provides a powerful, type-safe foundation for building web servers. Its key strengths are:
- **Type safety** — Catch errors at compile time
- **Composability** — Mix and match middleware
- **Performance** — Built on Tokio and Hyper
- **Flexibility** — Works with any async runtime
- **Community** — Backed by the Tokio team


# Deep Dive: EPUB File Format

## Understanding EPUB Structure

An EPUB file is actually a ZIP archive with a specific internal structure. Let's explore what's inside:

```
my_story.epub (ZIP file)
├── mimetype                    # Must be first, uncompressed
├── META-INF/
│   └── container.xml           # Points to the content file
└── OEBPS/
    ├── content.opf             # Package document (metadata + manifest)
    ├── toc.ncx                 # Table of contents (for older readers)
    ├── stylesheet.css          # Stylesheet
    ├── introduction.xhtml      # Title page
    ├── chapter_1.xhtml         # Chapter 1
    ├── chapter_2.xhtml         # Chapter 2
    └── ...
```

## The Mimetype File

The first file in the ZIP must be `mimetype` with exactly this content:

```
application/epub+zip
```

This file must be stored uncompressed (no deflation). This allows e-readers to identify the file as an EPUB by looking at the first few bytes.

## The Container File

`META-INF/container.xml` tells the e-reader where to find the package document:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
```

## The Package Document (content.opf)

This is the heart of the EPUB. It contains metadata, a manifest of all files, and the reading order:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>A Great Story</dc:title>
    <dc:creator>Author Name</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier id="bookid">urn:uuid:550e8400-e29b-41d4-a716-446655440000</dc:identifier>
    <dc:description>A wonderful fanfiction story</dc:description>
    <meta property="dcterms:modified">2024-01-15T10:30:00Z</meta>
  </metadata>
  
  <manifest>
    <item id="stylesheet" href="stylesheet.css" media-type="text/css"/>
    <item id="introduction" href="introduction.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="chapter_1" href="chapter_1.xhtml" media-type="application/xhtml+xml"/>
    <item id="chapter_2" href="chapter_2.xhtml" media-type="application/xhtml+xml"/>
    <item id="toc" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  
  <spine toc="toc">
    <itemref idref="introduction"/>
    <itemref idref="chapter_1"/>
    <itemref idref="chapter_2"/>
  </spine>
</package>
```

### Metadata Section

The metadata section contains:
- `dc:title` — Book title
- `dc:creator` — Author name
- `dc:language` — Language code (e.g., "en", "es", "fr")
- `dc:identifier` — Unique identifier (UUID, ISBN, etc.)
- `dc:description` — Book description
- `meta property="dcterms:modified"` — Last modification date

### Manifest Section

The manifest lists every file in the EPUB:
- `id` — Unique identifier for the file
- `href` — Relative path to the file
- `media-type` — MIME type of the file
- `properties` — Optional properties (like "nav" for the navigation document)

### Spine Section

The spine defines the reading order:
- `toc` — References the table of contents
- `itemref` — Each entry is a file from the manifest, in reading order

## The Table of Contents

EPUB 3 uses an XHTML navigation document:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Table of Contents</title></head>
<body>
  <nav epub:type="toc">
    <h1>Table of Contents</h1>
    <ol>
      <li><a href="introduction.xhtml">Introduction</a></li>
      <li><a href="chapter_1.xhtml">Chapter 1: The Beginning</a></li>
      <li><a href="chapter_2.xhtml">Chapter 2: The Journey</a></li>
    </ol>
  </nav>
</body>
</html>
```

EPUB 2 uses an NCX file (for backward compatibility):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <navMap>
    <navPoint id="intro" playOrder="1">
      <navLabel><text>Introduction</text></navLabel>
      <content src="introduction.xhtml"/>
    </navPoint>
    <navPoint id="ch1" playOrder="2">
      <navLabel><text>Chapter 1</text></navLabel>
      <content src="chapter_1.xhtml"/>
    </navPoint>
  </navMap>
</ncx>
```

## Chapter XHTML Format

Each chapter is an XHTML file:

```xml
<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>Chapter 1: The Beginning</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
    <h2>Chapter 1: The Beginning</h2>
    <p>Once upon a time, in a land far away, there lived a young wizard named Harry.</p>
    <p>He didn't know it yet, but his life was about to change forever.</p>
</body>
</html>
```

### Why XHTML?

EPUB requires XHTML (not regular HTML) because:
1. **Stricter parsing** — Ensures consistent rendering across devices
2. **XML compatibility** — Can be processed with XML tools
3. **Namespace support** — Allows EPUB-specific attributes

Key differences from HTML:
- All tags must be closed (`<br/>` instead of `<br>`)
- All attributes must be quoted (`class="text"` instead of `class=text`)
- Tags must be properly nested
- Case matters (XHTML is case-sensitive)

## CSS in EPUB

EPUB supports a subset of CSS. Here's what FicHub uses:

```css
body {
    font-family: serif;
    line-height: 1.5;
}

h2 {
    text-align: center;
}

p {
    margin: 0.5em 0;
}
```

### CSS Limitations in EPUB

Not all CSS features work in EPUB:
- No CSS Grid (limited support)
- No CSS Variables (limited support)
- No CSS Animations
- Limited Flexbox support
- No `position: fixed` or `position: sticky`

Stick to basic CSS for maximum compatibility.

## How FicHub Generates EPUBs

FicHub uses the `epub-builder` crate, which handles all the complexity:

```rust
// Create builder
let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

// Add metadata
builder.metadata("title", &meta.title)?;
builder.metadata("author", &meta.author)?;
builder.metadata("lang", "en")?;
builder.metadata("description", &meta.desc)?;

// Add stylesheet
let css = "body{font-family:serif;line-height:1.5;}";
builder.stylesheet(css.as_bytes())?;

// Add chapters
for chapter in chapters {
    let html = format!(/* chapter XHTML */);
    builder.add_content(
        EpubContent::new(&filename, html.as_bytes())
            .title(&chapter.title),
    )?;
}

// Generate file
let file = fs::File::create("output.epub")?;
builder.generate(file)?;
```

The `epub-builder` crate automatically:
1. Creates the `mimetype` file
2. Generates `container.xml`
3. Creates `content.opf` with metadata and manifest
4. Generates the navigation document
5. Packages everything into a ZIP

## EPUB Validation

EPUB files should be validated using the official EPUBCheck tool:

```bash
# Install EPUBCheck
wget https://github.com/w3c/epubcheck/releases/download/v5.1.0/epubcheck-5.1.0.zip
unzip epubcheck-5.1.0.zip

# Validate an EPUB
java -jar epubcheck.jar my_story.epub
```

Common validation errors:
- Missing `mimetype` file
- Incorrect MIME type in `mimetype`
- Broken internal links
- Missing required metadata
- Invalid XHTML

## EPUB Compatibility

Different e-readers have different EPUB support:

| Reader | EPUB 2 | EPUB 3 | CSS Support |
|--------|--------|--------|-------------|
| Kindle | Limited | No | Basic |
| Kobo | Yes | Yes | Good |
| Nook | Yes | Limited | Basic |
| Apple Books | Yes | Yes | Excellent |
| Calibre | Yes | Yes | Excellent |

For maximum compatibility, FicHub targets EPUB 2 with basic CSS.

## Advanced EPUB Features

### Cover Images

To add a cover image:

```rust
let cover_bytes = fs::read("cover.jpg")?;
builder.add_cover_image("cover.jpg", &*cover_bytes, "image/jpeg")?;
```

### Footnotes

EPUB supports footnotes using the `epub:type` attribute:

```html
<p>This is a statement.<a epub:type="noteref" href="#fn1">[1]</a></p>
<aside epub:type="footnote" id="fn1">
    <p>This is the footnote content.</p>
</aside>
```

### MathML

For scientific content, EPUB supports MathML:

```html
<math xmlns="http://www.w3.org/1998/Math/MathML">
    <msup>
        <mi>x</mi>
        <mn>2</mn>
    </msup>
</math>
```

### SVG Images

EPUB supports SVG for vector graphics:

```html
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
    <circle cx="50" cy="50" r="40" fill="blue"/>
</svg>
```

## Debugging EPUB Issues

### Common Problems

1. **Blank pages** — Usually caused by empty `<div>` elements or margin/padding issues
2. **Text overflow** — Use `overflow: hidden` or `word-wrap: break-word`
3. **Images not displaying** — Check the `href` path and MIME type
4. **Styles not applying** — Verify the stylesheet link in `<head>`
5. **Table of contents missing** — Ensure the `nav` element has `epub:type="toc"`

### Debugging Tools

- **Calibre** — Open the EPUB and inspect the structure
- **Sigil** — EPUB editor with syntax highlighting
- **epubcheck** — Official validation tool
- **Readium** — Browser-based EPUB reader for testing

## Performance Considerations

### File Size

EPUB files should be reasonable in size:
- Text-only: 100KB - 1MB
- With images: 1MB - 50MB
- With high-res images: 50MB - 500MB

FicHub's EPUBs are typically 100KB - 5MB (text-only).

### Generation Speed

Generating an EPUB involves:
1. Creating the work directory: ~1ms
2. Building the EPUB: ~10-100ms (depends on chapter count)
3. Computing the MD5 hash: ~1-10ms
4. Moving to cache: ~1ms

Total: ~15-115ms for EPUB generation.

### Memory Usage

The `epub-builder` crate keeps the entire EPUB in memory before writing. For very large stories (1000+ chapters), this could use significant memory. FicHub handles this by processing chapters one at a time.


# Deep Dive: Redis and Caching Architecture

## What Is Redis?

Redis (Remote Dictionary Server) is an in-memory data structure store. It's incredibly fast because it keeps all data in RAM instead of on disk. FicHub uses Redis for:

1. **Rate limiting** — Token bucket implementation
2. **Temporary data** — Collection queue for recommendations
3. **Caching** — Fast lookups for frequently accessed data
4. **Counting** — Rate limit counters

## Redis Data Structures

Redis supports several data structures:

### Strings

The simplest type — key-value pairs:

```redis
SET rate:ip:192.168.1.1 '{"value": 30, "last_drain": 1705312200.5}'
GET rate:ip:192.168.1.1
```

### Hashes

Maps of field-value pairs (like a mini-database):

```redis
HMSET bucket:global value 150 last_drain 1705312200.5
HMGET bucket:global value last_drain
```

FicHub uses hashes for token buckets:

```lua
-- Lua script for token bucket
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

### Lists

Ordered collections (used as queues):

```redis
LPUSH collection_queue:archiveofourown.org '{"url_id":"abc123","site_domain":"archiveofourown.org"}'
RPOP collection_queue:archiveofourown.org
```

FicHub uses lists for the recommendation collection queue.

### Sets

Unordered collections of unique strings:

```redis
SADD datacenter_ips "10.0.0.1" "10.0.0.2" "172.16.0.1"
SISMEMBER datacenter_ips "10.0.0.1"  -- Returns 1 (true)
```

### Sorted Sets

Ordered collections with scores (like a leaderboard):

```redis
ZADD leaderboard 100 "user:abc" 200 "user:def"
ZRANGEBYSCORE leaderboard 0 150  -- Users with score 0-150
```

## Redis Configuration

### Basic Configuration

```conf
# /etc/redis/redis.conf

# Network
bind 127.0.0.1
port 6379
timeout 0

# Memory
maxmemory 256mb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
save 60 10000

# Logging
loglevel notice
logfile /var/log/redis/redis-server.log
```

### Memory Management

Redis can use several eviction policies when memory is full:

| Policy | Description |
|--------|-------------|
| `noeviction` | Return errors when memory limit reached |
| `allkeys-lru` | Evict least recently used keys |
| `volatile-lru` | Evict least recently used keys with TTL |
| `allkeys-random` | Evict random keys |
| `volatile-random` | Evict random keys with TTL |
| `allkeys-ttl` | Evict keys with shortest TTL |
| `volatile-ttl` | Evict keys with shortest TTL |
| `noeviction` | Don't evict, return errors |

FicHub uses `allkeys-lru` — when Redis is full, it evicts the least recently used keys. This is appropriate because rate limit buckets are less important than other data when memory is tight.

### Persistence

Redis can save data to disk in two ways:

1. **RDB snapshots** — Periodic snapshots of the dataset
2. **AOF (Append-Only File)** — Logs every write operation

For FicHub's use case, persistence isn't critical (rate limits can be rebuilt), so RDB snapshots with long intervals are fine.

## Redis Connection Management

### Multiplexed Connections

FicHub uses multiplexed connections:

```rust
let redis_client = redis::Client::open(config.redis_url.as_str())?;
let redis_conn = redis_client.get_multiplexed_async_connection().await?;
```

A multiplexed connection sends multiple commands over a single TCP connection, reducing overhead.

### Connection Pooling

For high-throughput scenarios, you might want connection pooling:

```rust
let pool = redis::aio::ConnectionManager::new(client).await?;
```

`ConnectionManager` automatically reconnects if the connection drops.

### Error Handling

Redis operations can fail:

```rust
match redis::cmd("GET").arg(&key).query_async(&mut conn).await {
    Ok(value) => { /* use value */ }
    Err(e) => {
        tracing::warn!("Redis error: {}", e);
        // Fall back to default behavior
    }
}
```

FicHub handles Redis errors gracefully — if Redis is unavailable, rate limiting falls back to static delays.

## Lua Scripting in Redis

### Why Lua?

Redis executes Lua scripts atomically — no other command can run while a script is executing. This is crucial for rate limiting because:

1. Reading and writing the token bucket must be atomic
2. Two concurrent requests can't both think they have enough tokens
3. The time calculation must be consistent

### FicHub's Token Bucket Script

```lua
-- Input:
-- KEYS[1] = bucket key
-- ARGV[1] = tokens requested (usually 1)
-- ARGV[2] = bucket capacity
-- ARGV[3] = flow rate (tokens per second)

-- Output:
-- -1 if allowed
-- positive number = seconds to wait

local key = KEYS[1]
local requested = tonumber(ARGV[1])
local capacity = tonumber(ARGV[2])
local flow = tonumber(ARGV[3])

-- Get current state
local bucket = redis.call('HMGET', key, 'value', 'last_drain')
local value = tonumber(bucket[1])
local last_drain = tonumber(bucket[2])

-- Get current time
local now = redis.call('TIME')
local now_sec = tonumber(now[1]) + tonumber(now[2])/1000000

-- Initialize if new bucket
if value == nil then
    value = capacity
    last_drain = now_sec
    redis.call('HMSET', key, 'value', value, 'last_drain', last_drain)
end

-- Calculate new tokens
local elapsed = now_sec - last_drain
local new_tokens = math.min(capacity, value + elapsed * flow)
local allowed = new_tokens - requested

if allowed >= 0 then
    -- Request allowed
    redis.call('HMSET', key, 'value', allowed, 'last_drain', now_sec)
    return -1
else
    -- Request denied — calculate wait time
    local wait = (requested - new_tokens) / flow
    return wait
end
```

### Loading Lua Scripts

```rust
let lua_script = r#"..."#;  // The Lua script above
let mut conn = redis_conn.clone();
let lua_sha: String = redis::cmd("SCRIPT")
    .arg("LOAD")
    .arg(lua_script)
    .query_async(&mut conn)
    .await?;
```

Redis compiles the script and returns a SHA hash. Subsequent calls use `EVALSHA` with the hash instead of the full script.

### Executing Lua Scripts

```rust
let result: f64 = redis::cmd("EVALSHA")
    .arg(&self.lua_sha[..])
    .arg(1)           // Number of keys
    .arg(key)         // Key
    .arg(1.0)         // Requested tokens
    .arg(capacity)    // Bucket capacity
    .arg(flow)        // Flow rate
    .query_async(&mut conn)
    .await?;
```

## Rate Limiting in Production

### Two-Tier Rate Limiting

FicHub implements two tiers:

1. **Global bucket** — System-wide limit (150 tokens, 30/sec refill)
2. **Per-IP bucket** — Individual user limit (30 tokens, 0.116/sec refill)

```rust
async fn check_ip(&self, ip: IpAddr) -> RateLimitResult {
    // Check global bucket
    let global_wait = self.check_bucket("rate:global", 150.0, 30.0).await?;
    if global_wait > 0.0 {
        return RateLimitResult::Wait(global_wait.ceil() as u64);
    }

    // Check per-IP bucket
    let ip_key = format!("rate:ip:{}", ip);
    let ip_wait = self.check_bucket(&ip_key, 30.0, 0.116).await?;
    if ip_wait > 0.0 {
        return RateLimitResult::Wait(ip_wait.ceil() as u64);
    }

    RateLimitResult::Allowed
}
```

### Failure Penalization

When a request fails, FicHub penalizes the IP:

```rust
async fn report_failure(&self, ip: IpAddr) {
    // Consume extra tokens on failure
    let _ = self.penalize("rate:global", 150.0, 30.0).await;
    let ip_key = format!("rate:ip:{}", ip);
    let _ = self.penalize(&ip_key, 30.0, 0.116).await;
}
```

The penalty uses 1.5 tokens instead of 1, making the IP wait longer.

### Dynamic vs Static Rate Limiting

FicHub supports two modes:

**Dynamic** (default) — Uses token buckets with Redis:
- More efficient — only slows down when needed
- More complex — requires Redis
- More accurate — accounts for actual usage patterns

**Static** — Adds random delay to every request:
- Simpler — no Redis needed
- Less efficient — slows down even when not needed
- Less accurate — doesn't account for usage patterns

```rust
if !self.dynamic_rate_limit {
    let delay = 0.1 + rand::random::<f64>() * 0.1;  // 100-200ms
    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
    return RateLimitResult::Allowed;
}
```

## Monitoring Redis

### Redis CLI Commands

```bash
# Check connection
redis-cli ping

# View all keys
redis-cli KEYS "*"

# View rate limit bucket
redis-cli HGETALL rate:global

# Monitor commands in real-time
redis-cli MONITOR

# View server info
redis-cli INFO

# Check memory usage
redis-cli INFO memory
```

### Redis Metrics

Key metrics to monitor:
- `connected_clients` — Number of connected clients
- `used_memory` — Memory usage
- `evicted_keys` — Keys evicted due to memory limit
- `instantaneous_ops_per_sec` — Operations per second
- `keyspace_hits` / `keyspace_misses` — Cache hit rate

## Redis Best Practices

### Key Naming Convention

Use descriptive key names:

```
rate:global                    # Global rate limit bucket
rate:ip:192.168.1.1           # Per-IP rate limit bucket
collection_queue:ao3           # AO3 collection queue
datacenter_ips                 # Set of datacenter IPs
```

### TTL (Time-To-Live)

Set TTLs on temporary data:

```redis
# Rate limit counters expire after 1 hour
SET rate:ip:192.168.1.1 1 EX 3600

# Collection queue items expire after 24 hours
LPUSH collection_queue:ao3 '{"url_id":"abc123"}'
EXPIRE collection_queue:ao3 86400
```

### Pipeline Commands

For better performance, batch multiple commands:

```rust
let mut pipe = redis::pipe();
pipe.cmd("SET").arg("key1").arg("value1");
pipe.cmd("SET").arg("key2").arg("value2");
pipe.query_async(&mut conn).await?;
```

### Avoid Large Keys

Redis is optimized for small values. Avoid storing large strings (like entire HTML pages). Instead:
- Store references or hashes
- Use compression for large values
- Split large data across multiple keys

## Redis Alternatives

If Redis isn't available, FicHub could use:

1. **In-memory HashMap** — Simple but lost on restart
2. **SQLite** — Persistent but slower
3. **Memcached** — Similar to Redis but simpler
4. **PostgreSQL** — Can store rate limit data, but slower

FicHub's design allows swapping the rate limiter implementation via the `RateLimiter` trait.


# Deep Dive: Docker Production Deployment

## Building a Production Docker Image

### Optimizing the Build

The multi-stage build is crucial for small images:

```dockerfile
# Stage 1: Build
FROM rust:1.78 AS builder
WORKDIR /app

# Copy dependency files first (for layer caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release
RUN rm -rf src

# Copy actual source and rebuild
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y     ca-certificates     && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fichub /usr/local/bin/fichub
COPY --from=builder /app/migrations /usr/local/bin/migrations
COPY frontend/build /app/frontend/build

RUN mkdir -p /app/cache /app/tmp /app/logs

EXPOSE 3000

# Run as non-root user
RUN useradd -r -s /bin/false fichub
USER fichub

CMD ["fichub"]
```

### Build Optimization Tricks

1. **Layer caching** — Copy `Cargo.toml` before source code
2. **Dependency caching** — Build dependencies first, then source
3. **Minimal base** — Use `debian:bookworm-slim` instead of full Debian
4. **Non-root user** — Run as unprivileged user for security
5. **Clean up** — Remove apt cache and build artifacts

### Docker BuildKit

Enable BuildKit for faster builds:

```bash
DOCKER_BUILDKIT=1 docker build -t fichub .
```

BuildKit provides:
- Better caching
- Parallel stage execution
- Build secrets
- SSH forwarding

## Docker Compose Production Setup

### Production docker-compose.yml

```yaml
version: '3.8'

services:
  fichub:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "127.0.0.1:3000:3000"  # Only bind to localhost
    environment:
      - DATABASE_URL=postgres://fichub:${DB_PASSWORD}@db:5432/fichub
      - REDIS_URL=redis://redis:6379/
      - CACHE_DIR=/app/cache
      - PORT=3000
      - NODE_NAME=production
      - RUST_LOG=info,fichub=debug
    volumes:
      - fichub_cache:/app/cache
      - fichub_tmp:/app/tmp
      - fichub_logs:/app/logs
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  db:
    image: postgres:16-alpine
    environment:
      - POSTGRES_USER=fichub
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=fichub
      - POSTGRES_INITDB_ARGS=--auth-host=scram-sha-256
    volumes:
      - pg_data:/var/lib/postgresql/data
      - ./init.sql:/docker-entrypoint-initdb.d/init.sql:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub -d fichub"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 30s
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1G
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    command: redis-server --requirepass ${REDIS_PASSWORD} --maxmemory 256mb --maxmemory-policy allkeys-lru
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "-a", "${REDIS_PASSWORD}", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
    depends_on:
      - fichub
    restart: unless-stopped

volumes:
  pg_data:
  redis_data:
  fichub_cache:
  fichub_tmp:
  fichub_logs:
```

### Environment Variables

Create a `.env` file (never commit this!):

```env
DB_PASSWORD=your_secure_password_here
REDIS_PASSWORD=your_redis_password_here
```

### Using Docker Secrets

For even better security, use Docker secrets:

```yaml
services:
  fichub:
    secrets:
      - db_password
      - redis_password
    environment:
      - DATABASE_URL=postgres://fichub@db:5432/fichub

secrets:
  db_password:
    file: ./secrets/db_password.txt
  redis_password:
    file: ./secrets/redis_password.txt
```

## Nginx Configuration

### Reverse Proxy Setup

```nginx
upstream fichub {
    server fichub:3000;
}

server {
    listen 80;
    server_name fichub.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fichub.example.com;

    ssl_certificate /etc/nginx/certs/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml;
    gzip_min_length 1000;

    location / {
        proxy_pass http://fichub;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
        proxy_pass http://fichub;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Don't cache API responses
    location /api/ {
        proxy_pass http://fichub;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        add_header Cache-Control "no-store, no-cache, must-revalidate";
    }
}
```

## SSL/TLS Setup

### Let's Encrypt with Certbot

```bash
# Install certbot
apt install certbot python3-certbot-nginx

# Get certificate
certbot --nginx -d fichub.example.com

# Auto-renewal
certbot renew --dry-run
```

### Docker-based Certbot

```yaml
services:
  certbot:
    image: certbot/certbot
    volumes:
      - ./certs:/etc/letsencrypt
      - ./www:/var/www/certbot
    entrypoint: "/bin/sh -c 'trap exit TERM; while :; do certbot renew; sleep 12h & wait $${!}; done;'"
```

## Monitoring and Logging

### Prometheus Metrics

FicHub exposes metrics at `/metrics`:

```yaml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana
    ports:
      - "3001:3000"
    volumes:
      - grafana_data:/var/lib/grafana
```

### Prometheus Configuration

```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'fichub'
    static_configs:
      - targets: ['fichub:3000']
    metrics_path: '/metrics'
```

### Log Aggregation

For production, use structured logging:

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
    )
    .json()  // JSON format for log aggregation
    .init();
```

### ELK Stack

For advanced log analysis:

```yaml
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:8.11.0
    environment:
      - discovery.type=single-node
      - xpack.security.enabled=false
    volumes:
      - es_data:/usr/share/elasticsearch/data

  logstash:
    image: docker.elastic.co/logstash/logstash:8.11.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf

  kibana:
    image: docker.elastic.co/kibana/kibana:8.11.0
    ports:
      - "5601:5601"
```

## Backup Strategies

### Automated Backups

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/fichub/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# PostgreSQL backup
docker compose exec -T db pg_dump -U fichub fichub |     gzip > "$BACKUP_DIR/fichub_$(date +%H%M%S).sql.gz"

# Redis backup
docker compose exec -T redis redis-cli -a "$REDIS_PASSWORD" BGSAVE
docker compose cp redis:/data/dump.rdb "$BACKUP_DIR/redis_$(date +%H%M%S).rdb"

# Cache backup
tar -czf "$BACKUP_DIR/cache_$(date +%H%M%S).tar.gz"     -C /var/lib/docker/volumes/fichub_fichub_cache .

# Cleanup old backups (keep 30 days)
find /backups/fichub -type d -mtime +30 -exec rm -rf {} +

# Upload to remote storage (optional)
# rclone copy "$BACKUP_DIR" remote:fichub-backups/
```

### Cron Job

```bash
# Add to crontab
0 2 * * * /opt/fichub/backup.sh >> /var/log/fichub-backup.log 2>&1
```

### Recovery Procedure

```bash
# Stop FicHub
docker compose down

# Restore PostgreSQL
gunzip -c backup.sql.gz | docker compose exec -T db psql -U fichub fichub

# Restore Redis
docker compose cp backup/redis.rdb redis:/data/dump.rdb
docker compose restart redis

# Restore cache
tar -xzf backup/cache.tar.gz -C /var/lib/docker/volumes/fichub_fichub_cache

# Start FicHub
docker compose up -d
```

## Performance Tuning

### PostgreSQL Tuning

```sql
-- Increase shared buffers (25% of RAM)
ALTER SYSTEM SET shared_buffers = '1GB';

-- Increase work memory
ALTER SYSTEM SET work_mem = '16MB';

-- Increase maintenance work memory
ALTER SYSTEM SET maintenance_work_mem = '256MB';

-- Enable parallel queries
ALTER SYSTEM SET max_parallel_workers_per_gather = 4;

-- Tune checkpoint
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '64MB';

-- Reload configuration
SELECT pg_reload_conf();
```

### Redis Tuning

```conf
# Increase maxmemory
maxmemory 512mb

# Use allkeys-lru for rate limiting
maxmemory-policy allkeys-lru

# Disable persistence for rate limiting (data is ephemeral)
save ""
appendonly no

# Increase TCP backlog
tcp-backlog 511
```

### Rust Optimizations

```bash
# Build with maximum optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Use link-time optimization
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
```

## Scaling Strategies

### Horizontal Scaling

Run multiple FicHub instances behind a load balancer:

```yaml
services:
  fichub:
    deploy:
      replicas: 3
    # ... other config
```

### Database Scaling

For high traffic:

1. **Read replicas** — Separate read and write databases
2. **Connection pooling** — Use PgBouncer
3. **Partitioning** — Split large tables by date or hash
4. **Archiving** — Move old data to cold storage

### Cache Scaling

For high traffic:

1. **Redis Cluster** — Distribute data across multiple Redis instances
2. **Redis Sentinel** — High availability with automatic failover
3. **Local caching** — Add in-memory caching for hot data

## Security Hardening

### Network Security

```yaml
services:
  fichub:
    networks:
      - internal
    # Don't expose ports directly

  nginx:
    networks:
      - internal
      - external
    ports:
      - "443:443"

networks:
  internal:
    driver: bridge
  external:
    driver: bridge
```

### Container Security

```dockerfile
# Run as non-root user
RUN useradd -r -s /bin/false fichub
USER fichub

# Read-only filesystem
docker run --read-only --tmpfs /tmp fichub

# Drop capabilities
docker run --cap-drop=ALL --cap-add=NET_BIND_SERVICE fichub
```

### Secrets Management

```bash
# Use Docker secrets
echo "my_secret_password" | docker secret create db_password -

# Or use environment variables from a file
docker compose --env-file .env.prod up -d
```

## Troubleshooting

### Common Issues

1. **Container won't start** — Check logs: `docker compose logs fichub`
2. **Database connection refused** — Ensure PostgreSQL is healthy: `docker compose ps`
3. **Redis connection refused** — Check Redis password and network
4. **Out of memory** — Increase container memory limits
5. **Disk full** — Clean up old cached files

### Debugging Commands

```bash
# View container logs
docker compose logs -f fichub

# Execute command in container
docker compose exec fichub sh

# Check container resource usage
docker stats

# Inspect container
docker inspect fichub_fichub_1

# View container filesystem
docker compose exec fichub ls -la /app/cache
```

### Health Checks

```bash
# Check API health
curl http://localhost:3000/api/

# Check database
docker compose exec db pg_isready -U fichub

# Check Redis
docker compose exec redis redis-cli ping

# Check disk space
docker system df
```

## Conclusion

Deploying FicHub in production requires careful attention to:
1. **Security** — Use secrets, non-root users, network isolation
2. **Reliability** — Health checks, restart policies, backups
3. **Performance** — Resource limits, database tuning, caching
4. **Monitoring** — Logs, metrics, alerts
5. **Scalability** — Horizontal scaling, load balancing

With these practices, FicHub can serve thousands of users reliably.


# Deep Dive: Frontend Integration

## Serving the Frontend

FicHub serves a SvelteKit frontend as static files. The frontend is built separately and served by Axum's `ServeDir` middleware.

### Static File Serving

```rust
.fallback_service(
    ServeDir::new(&frontend_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(frontend_dir.join("index.html"))),
)
```

This configuration:
1. Serves static files from `frontend_dir`
2. Appends `index.html` when visiting directories
3. Falls back to `index.html` for any file that doesn't exist (SPA routing)

### Why SPA Routing?

Single-Page Applications (SPAs) handle routing in the browser. When you visit `/stories/abc123`, the browser loads `index.html`, and JavaScript handles the routing. The `fallback` ensures all routes return `index.html`.

## API Communication

The frontend communicates with FicHub's API:

### Fetching Metadata

```javascript
const response = await fetch(`/api/v0/meta?q=${encodeURIComponent(url)}`);
const data = await response.json();
```

### Downloading EPUBs

```javascript
const response = await fetch(`/api/v0/epub?q=${encodeURIComponent(url)}`);
const data = await response.json();
if (data.err === 0) {
    window.location.href = data.epub_url;
}
```

### Searching

```javascript
const response = await fetch(`/api/v0/search?q=${encodeURIComponent(query)}&page=1`);
const data = await response.json();
```

## CORS Configuration

FicHub uses permissive CORS for development:

```rust
.layer(CorsLayer::permissive())
```

In production, restrict CORS:

```rust
.layer(CorsLayer::new()
    .allow_origin("https://fichub.example.com".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers(Any)
)
```

## Frontend Build Process

### SvelteKit Build

```bash
cd frontend
npm install
npm run build
```

This creates static files in `frontend/build/`.

### Docker Integration

```dockerfile
# Copy frontend assets
COPY frontend/build /app/frontend/build
```

### Development Mode

For development, run the frontend separately:

```bash
# Terminal 1: Backend
cargo run

# Terminal 2: Frontend
cd frontend
npm run dev
```

The SvelteKit dev server proxies API requests to the backend.

## API Response Format

All FicHub API responses follow a consistent format:

```json
{
    "err": 0,
    "message": "success",
    "data": { ... }
}
```

### Error Responses

```json
{
    "err": -5,
    "msg": "unsupported URL"
}
```

### Success Responses

```json
{
    "err": 0,
    "url_id": "a1b2c3d4e5f6",
    "meta": {
        "title": "A Great Story",
        "author": "Author Name",
        "words": 50000,
        "chapters": 10
    },
    "urls": {
        "epub": "/cache/epub/a1b2c3d4e5f6?h=abc123",
        "html": "/cache/html/a1b2c3d4e5f6?h=def456"
    }
}
```

## Frontend Features

### Story Browser

The frontend provides a story browser with:
- Search functionality
- Tag filtering
- Sorting options
- Pagination

### Download Manager

Users can:
- Request EPUB downloads
- View download progress
- Access previously downloaded files

### OPDS Setup Guide

The frontend includes instructions for setting up OPDS on various e-readers.

## Performance Optimization

### Static Asset Caching

```nginx
location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}
```

### API Response Caching

```rust
// Add Cache-Control header
(
    [("Cache-Control", "public, max-age=300")],  // 5 minutes
    body,
)
```

### Compression

Enable gzip compression:

```rust
.layer(CompressionLayer::new().gzip(true))
```

## Security Considerations

### Content Security Policy

```nginx
add_header Content-Security-Policy "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'";
```

### XSS Prevention

The frontend escapes user input:

```javascript
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
```

### CSRF Protection

For POST requests, include a CSRF token:

```javascript
const response = await fetch('/api/v0/tags/submit', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json',
        'X-CSRF-Token': csrfToken,
    },
    body: JSON.stringify({ url_id, tag_name }),
});
```

## Conclusion

FicHub's frontend integration provides:
1. Static file serving for SvelteKit SPA
2. Consistent API response format
3. CORS configuration for cross-origin requests
4. Performance optimization via caching and compression
5. Security headers for XSS and CSRF protection
6. OPDS setup guidance for e-readers

This creates a seamless user experience for browsing, searching, and downloading fanfiction.
