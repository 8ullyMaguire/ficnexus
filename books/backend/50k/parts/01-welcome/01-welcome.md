# Part 1: Welcome to FicHub

## Chapter 1: What We're Building

### Welcome, Builder!

Hey there! 👋

If you're reading this, you're about to do something really cool: you're going to learn how to build a real, working web server from scratch. Not a toy example. Not a "Hello World" that you throw away. A *real* server that real people use every day.

The project is called **FicHub**, and it's a fanfiction download server. If you've ever wanted to read your favorite fanfiction on your phone, your e-reader, or just save a copy for yourself — FicHub is the tool that makes that happen.

But here's the best part: you're going to build it yourself. Every line of code, every feature, every little detail — you'll understand it because you wrote it.

Ready? Let's go.

### What Is FicHub?

FicHub is a self-hosted server that helps people download fanfiction from the internet. It does three main things:

1. **Scrapes** stories from sites like Archive of Our Own (AO3), FanFiction.net (FF.net), and XenForo-based forums
2. **Exports** them as beautiful EPUB files, HTML pages, or plain text
3. **Serves** them to you — download them, read them, keep them forever

Think of it like a librarian who knows exactly where to find every story on the internet, makes a neat copy of it, and hands it to you.

### Why Fanfiction?

Fanfiction is one of the biggest creative communities on the internet. Millions of people write stories about the books, movies, and shows they love. They take characters from Harry Potter, Marvel, Star Wars, and thousands of other worlds, and create new adventures.

The numbers are staggering. Archive of Our Own (AO3) alone has over 12 million works. FanFiction.net has been around since 1998 and still hosts millions of stories. There are XenForo-based communities for every fandom imaginable — from Doctor Who to BTS to Minecraft.

But here's the thing: reading fanfiction online has its downsides. Stories get deleted. Sites go down. Authors take their work offline. AO3 has had outages. FanFiction.net has lost entire archives. Having a personal download of your favorites means you never lose a story you love.

There's also the experience of reading itself. Reading a 200-chapter story on a phone screen is exhausting. The ads, the pop-ups, the tiny text — it's not ideal. But when you download that same story as an EPUB? It looks beautiful on your Kindle, your Kobo, or your phone's reading app. Chapters are properly formatted, images are included, and you can adjust the font size to whatever's comfortable.

That's where FicHub comes in. It lets people keep local copies of their favorite stories — formatted beautifully for reading on any device. And since FicHub is self-hosted, you control everything. Your reading habits are private, your downloads are instant, and you don't depend on anyone else's server staying online.

### The Journey: Scrape → Export → Download

Let's trace the path of a fanfiction story through FicHub. Imagine you just found an amazing story on AO3 and you want to read it on your Kindle.

```
You paste a URL → FicHub scrapes it → FicHub builds an EPUB → You download it
```

Here's what happens under the hood, step by step:

**Step 1: You paste a link.** You copy the URL of a story (like `https://archiveofourown.org/works/12345678`) and paste it into FicHub's web interface. That's all you need to do.

**Step 2: FicHub identifies the source.** FicHub looks at the URL and figures out which site it came from. AO3 URLs look different from FF.net URLs, and XenForo forums look different again. FicHub has a special piece of code called a **scraper registry** that knows how to handle each site.

**Step 3: The scraper gets to work.** The scraper for that site fetches the story page, parses the HTML, and pulls out everything: title, author, summary, tags, chapters, and all the text. It's like a robot reading the page and taking very careful notes.

**Step 4: The exporter builds the EPUB.** Now that FicHub has all the story data, it assembles it into an EPUB file — the same format your Kindle or Kobo uses. Chapters become chapters, the table of contents is generated automatically, and everything is formatted to look beautiful on any device.

**Step 5: Caching for speed.** The EPUB is saved in a cache (backed by Redis and the filesystem) so the next time someone requests the same story, it's served instantly — no re-scraping needed. FicHub can generate thousands of EPUBs, but it only needs to scrape each story once.

**Step 6: You download and enjoy.** You click the download button and the EPUB file appears on your device. Open it in your favorite reading app, adjust the font size, and settle in for a great read.

That's the core loop. But FicHub also does much more — it has a recommendation engine that suggests stories you might like, a tagging system that lets the community organize content, full-text search, and an OPDS catalog that e-readers can browse directly.

### Why Rust?

There are a lot of programming languages out there. So why did we pick Rust?

**Rust is fast.** Like, really fast. It runs as fast as C or C++, which means FicHub can handle lots of requests without breaking a sweat. When someone asks for a download, they get it instantly. FicHub needs to scrape websites, parse HTML, generate EPUB files, and serve them to users — all at the same time. Rust handles all of this without breaking a sweat.

**Rust is safe.** One of the biggest problems in programming is bugs — little mistakes in your code that cause crashes or, worse, security holes. Rust's compiler catches most of these *before your code ever runs*. It's like having a really smart friend who proofreads everything. If you try to use a piece of memory that's already been freed, Rust stops you. If you try to have two different parts of your code modifying the same data at the same time, Rust stops you. These kinds of bugs cause crashes, data corruption, and security vulnerabilities in other languages — but in Rust, they're compile-time errors.

**Rust is modern.** Rust was designed in the 2010s to solve real problems that programmers face today. It has amazing tooling (Cargo is the best package manager I've ever used), a fantastic community, and libraries for just about everything. Need to connect to a database? There's a crate for that. Need to parse HTML? There's a crate for that. Need to generate EPUB files? There's a crate for that too.

**Rust is fun.** Okay, that's subjective. But once you get the hang of it, there's something deeply satisfying about writing code that compiles and just *works*. No runtime crashes, no null pointer exceptions, no garbage collector pauses. Just fast, reliable code.

**Rust has a great community.** The Rust community is known for being welcoming and helpful. If you get stuck, there are forums, Discord servers, and Stack Overflow answers ready to help. The official Rust Book (often called "The Book") is one of the best programming tutorials ever written.

💡 **Key Concept**

**Why Not Python or JavaScript?**

You might be wondering: "Why not use Python or JavaScript? They're easier!" And you'd be right — they *are* easier to learn. But FicHub needs to handle many things at once: scraping, generating, caching, and serving. Python's Global Interpreter Lock (GIL) makes it hard to do many things at once. JavaScript can do it, but it's slower and less safe.

Rust gives you the best of both worlds: the performance of low-level languages like C, with the safety and ergonomics of high-level languages like Python. It's the perfect choice for a server that needs to be fast, reliable, and secure.

### What You'll Learn

By the time you finish this book, you'll know how to:

- **Axum** — A modern Rust web framework for building APIs and web servers
- **PostgreSQL** — A powerful relational database for storing all your data
- **Redis** — A lightning-fast in-memory cache and message broker
- **Docker** — Package your server so it runs anywhere
- **Async programming** — Write code that handles thousands of requests at once

You'll also learn the fundamentals of Rust — variables, functions, ownership, and all the building blocks you need to write real programs.

### The Three Main Features

Let's zoom in on the three big features that make FicHub special:

**1. Download Engine**

The heart of FicHub. It scrapes stories from multiple sites, handles all the formatting, and produces clean EPUB files. Each source site (AO3, FF.net, XenForo) has its own scraper — a piece of code that knows exactly how to extract story content from that particular site.

The download engine is more complex than it sounds. Different sites have different HTML structures, different ways of splitting chapters, different metadata formats. FicHub's scraper registry handles all of this automatically. You give it a URL, and it figures out the rest.

The engine also handles error cases gracefully. What if a story has been deleted? What if the site is temporarily down? What if the story is marked as "restricted" and requires login? FicHub has sensible defaults for all of these situations.

**2. Recommendation Engine**

Ever finished a great story and thought "I want more like this"? FicHub's recommendation engine analyzes what you've read and suggests similar stories. It looks at tags, authors, and community voting to find hidden gems you'll love.

The recommendation engine uses a technique called collaborative filtering. It looks at what stories are often liked together — if many people who enjoyed Story A also enjoyed Story B, then Story B is probably a good recommendation for someone who just finished Story A. The engine also weighs community votes and tagging patterns to improve its suggestions over time.

There's also a background worker that precomputes recommendations. Instead of making you wait while it crunches numbers, the recommendations are ready before you even ask. The worker runs periodically, updating its suggestions as new stories are scraped and new votes come in.

**3. Search and Discovery**

FicHub doesn't just download stories — it helps you find them. Full-text search lets you search by title, author, summary, or tags. The OPDS catalog means you can browse the collection right from your e-reader's browser.

OPDS stands for Open Publication Distribution System. It's a standard that e-readers like Kobo and some Kindle models understand. If you point your e-reader at FicHub's OPDS feed, you can browse the entire collection, see what's new, check out popular stories, and download them — all without leaving your e-reader's browser. It's like having a personal library that your Kindle can browse directly.

The tagging system is community-driven. Users can submit tags, vote on existing tags, and curate the collection. Tags are organized by type (genre, character, rating, etc.) and the system automatically hides or removes tags that the community doesn't find useful.

### What This Book Will Look Like

This book is written step-by-step. We'll start with the basics — setting up your tools, writing your first Rust program — and build up to the full FicHub server. Each chapter introduces new concepts and gives you working code you can run and experiment with.

We'll use a friendly, conversational tone because programming should be fun, not intimidating. We'll add exercises for you to try, warnings about common mistakes, and explanations of key concepts along the way.

Every chapter builds on the previous one. By the time you reach the end, you won't just have a working server — you'll understand *why* it works the way it does. You'll be able to modify it, extend it, and build your own projects from scratch.

Let's get started!

---

## Chapter 2: Setting Up Your Workshop

### Getting Your Tools Ready

Before we can build anything, we need the right tools. Think of this like setting up a workshop — you need your saw, your hammer, your measuring tape before you can build a bookshelf.

Here's what we need:

| Tool | What It Does | Why We Need It |
|------|-------------|----------------|
| **Rust** | Programming language | The language FicHub is written in |
| **PostgreSQL** | Database | Stores stories, tags, users, and more |
| **Redis** | Cache | Speeds things up with lightning-fast storage |
| **Node.js** | JavaScript runtime | For the frontend (later in the book) |
| **Cargo** | Rust's package manager | Comes with Rust — manages dependencies |

Don't worry if some of those terms are new. We'll explain everything as we go.

### Installing Rust

Rust is installed through a tool called **rustup**. It's like an app store for Rust — it installs the compiler, the standard library, and all the tools you need.

**On Linux or macOS**, open your terminal and run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This downloads the installer. When it asks you a question, just press **1** and then **Enter** to accept the default installation.

After it finishes, restart your terminal (or run `source $HOME/.cargo/env`), then check that everything worked:

```bash
rustc --version
cargo --version
```

You should see version numbers like `rustc 1.78.0` (or newer) and `cargo 1.78.0`. The exact numbers don't matter — as long as you see versions, you're good!

**On Windows**, download rustup-init.exe from [rustup.rs](https://rustup.rs) and run it. Same idea — accept the defaults.

🧪 **Try It Yourself**

Run these commands and verify you get version numbers:

```bash
rustc --version
cargo --version
```

If you get "command not found," try restarting your terminal. Still not working? The Rust community has your back at [users.rust-lang.org](https://users.rust-lang.org).

### Installing PostgreSQL

PostgreSQL (often just called "Postgres") is our database. It stores all the information about stories, tags, users, and everything else FicHub needs to remember.

**On macOS** (using Homebrew):

```bash
brew install postgresql@16
brew services start postgresql@16
```

**On Ubuntu/Debian:**

```bash
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**On Arch Linux:**

```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl start postgresql
```

Once Postgres is running, create a database for FicHub:

```bash
sudo -u postgres createdb fichub
```

PostgreSQL is a relational database, which means it stores data in tables with rows and columns — like a spreadsheet, but much more powerful. FicHub uses PostgreSQL to store:

- Story metadata (title, author, summary, word count, chapter count)
- Tags and categories
- User votes and recommendations
- Cache information

We'll set up the database schema in a later chapter. For now, just make sure Postgres is running and the `fichub` database exists.

⚠️ **Watch Out**

On some systems, Postgres uses "peer authentication," which means you need to be the `postgres` system user to create databases. If you get a permission error, try `sudo -u postgres psql` to connect as the postgres admin. You can also check the Postgres configuration file (usually at `/etc/postgresql/*/main/pg_hba.conf`) to adjust authentication settings.

### Installing Redis

Redis is a super-fast data store that we'll use for caching. When FicHub generates an EPUB, it saves a copy in Redis so the next request for the same story is instant — no re-scraping needed.

Redis stands for **Re**mote **Di**ctionary **S**erver. It stores data in memory (RAM) instead of on a hard drive, which makes it incredibly fast — we're talking microsecond response times. For FicHub, this means that when someone requests a story that's already been scraped, we can serve the EPUB in milliseconds.

Redis also handles rate limiting. FicHub needs to be polite to the sites it scrapes — it can't bombard AO3 with thousands of requests per second. Redis tracks how many requests each IP address has made and blocks those that go too fast.

**On macOS:**

```bash
brew install redis
brew services start redis
```

**On Ubuntu/Debian:**

```bash
sudo apt install redis-server
sudo systemctl start redis
```

**On Arch Linux:**

```bash
sudo pacman -S redis
sudo systemctl start redis
```

Verify it's running:

```bash
redis-cli ping
```

If you see `PONG`, Redis is happy and ready to go!

💡 **Key Concept**

**Why Redis Instead of Just a File?**

You might wonder why we need Redis at all. Couldn't we just save cached files on disk? We could, and FicHub does save EPUBs to disk too. But Redis gives us extra capabilities:

1. **Speed** — Redis is in-memory, so lookups take microseconds instead of milliseconds
2. **Rate limiting** — Redis's atomic counters let us track request rates without complex locking code
3. **Pub/Sub** — Redis can send messages between different parts of the system (useful for the recommendation worker)
4. **Expiry** — Redis can automatically delete old cache entries, so our cache doesn't grow forever

Redis is like having a really fast assistant who remembers everything and can handle thousands of tasks at once.

### Installing Node.js (For Later)

We won't use Node.js until much later in the book (when we build the frontend), but it's good to have it ready. We recommend using **nvm** (Node Version Manager):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20
nvm use 20
node --version
```

If you already have Node.js installed, that's fine — just make sure it's version 18 or newer.

### Creating Your First Rust Project

Now comes the fun part — writing code!

Cargo (Rust's package manager) can create a new project for you with a single command:

```bash
cargo new fichub
cd fichub
```

This creates a directory called `fichub` with this structure:

```
fichub/
├── Cargo.toml
└── src/
    └── main.rs
```

That's it! Two files. Simple, clean, and ready to go.

Let's peek inside `src/main.rs`:

```rust
fn main() {
    println!("Hello, world!");
}
```

This is the starting point of every Rust executable. The `main` function is where your program begins. Right now it just prints "Hello, world!" — but soon it'll launch an entire web server.

### What the Real FicHub Looks Like

Our toy project has two files. The real FicHub project is much bigger — but don't worry, we'll build up to it piece by piece. Here's a sneak peek at the actual project structure:

```
fichub/
├── Cargo.toml              # Dependencies and project info
├── .env                    # Environment variables (database URLs, ports)
├── src/
│   ├── main.rs             # Entry point — starts the server
│   ├── server.rs           # Axum web server setup
│   ├── config.rs           # Configuration from environment
│   ├── db.rs               # Database connection and queries
│   ├── cache.rs            # Redis caching layer
│   ├── scrape/             # Scrapers for different sites
│   │   ├── mod.rs
│   │   ├── registry.rs     # Which scraper handles which site
│   │   ├── ao3.rs          # AO3-specific scraping
│   │   └── ffn.rs          # FF.net-specific scraping
│   ├── export/             # EPUB/HTML generation
│   ├── recommender/        # Recommendation engine
│   ├── tags/               # Tagging system
│   ├── search/             # Full-text search
│   ├── routes/             # HTTP route handlers
│   └── limiter/            # Rate limiting
├── migrations/             # Database schema migrations
└── frontend/               # SvelteKit web interface
```

That's a lot of files! But here's the thing — each file has a clear purpose, and we'll understand every one of them by the time we're done. The beauty of Rust (and good software design) is that complex systems are built from simple, understandable pieces.

### Understanding Cargo.toml

Let's look at the `Cargo.toml` file. This is your project's birth certificate — it tells Cargo everything it needs to know about your project.

```toml
[package]
name = "fichub"
version = "0.1.0"
edition = "2024"

[dependencies]
```

The `[package]` section has:
- **name** — the name of your project (`fichub`)
- **version** — the version number (starting at `0.1.0`)
- **edition** — the Rust edition to use (2024 is the latest)

The `[dependencies]` section is where you'll list all the libraries your project needs. Right now it's empty, but soon it'll be full of awesome tools like Axum, SQLx, and Redis.

Let's look at what FicHub's real `Cargo.toml` will eventually contain:

```toml
[dependencies]
# Web framework
axum = "0.8"
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.9", features = ["postgres", "runtime-tokio"] }

# Redis
redis = { version = "1.4", features = ["aio", "tokio-comp"] }

# HTTP client (for scraping)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# HTML parsing
scraper = "0.27"

# EPUB generation
epub-builder = "0.8"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Don't worry about understanding all of this right now! We'll go through each dependency in detail in later chapters. For now, just know that each line adds a library to your project. Cargo will automatically download and compile all of them when you run `cargo build`.

The `features` part is interesting — it lets you turn on optional functionality. For example, `tokio = { version = "1", features = ["full"] }` means "give me Tokio with all its features enabled." Tokio is the async runtime that lets Rust handle thousands of connections at once.

💡 **Key Concept**

**What is a Crate?**

In Rust, a reusable library is called a **crate**. You might see references to "crates.io" — that's like an app store for Rust libraries. When you add a dependency to `Cargo.toml`, Cargo downloads that crate from crates.io. There are over 140,000 crates available, covering everything from web servers to game engines to math libraries.

### Running Your First Build

Let's make sure everything works:

```bash
cargo build
```

This compiles your project. The first time you run this, it might take a few minutes — Cargo is downloading and compiling all the standard libraries. Don't worry, this only happens once!

When it's done, you should see something like:

```
   Compiling fichub v0.1.0 (/path/to/fichub)
    Finished dev [unoptimized + debuginfo] target(s) in 0.5s
```

Now let's run it:

```bash
cargo run
```

You'll see:

```
Hello, world!
```

That's your first Rust program! We'll talk about what it does in the next chapter.

🧪 **Try It Yourself**

Try changing the text inside the quotes:

```rust
fn main() {
    println!("I am building FicHub!");
}
```

Run `cargo run` again and see what happens. Pretty cool, right?

### Other Useful Cargo Commands

Here are some Cargo commands you'll use a lot:

```bash
cargo build          # Compile your project
cargo run            # Compile and run it
cargo check          # Quick syntax check (faster than build)
cargo test           # Run all tests
cargo fmt            # Auto-format your code
cargo clippy         # Lint your code (find potential problems)
```

💡 **Key Concept**

**Cargo** is one of the best things about Rust. It handles downloading libraries, compiling your code, running tests, and even publishing your project. It's like npm (for JavaScript) or pip (for Python), but it also compiles your code. Think of it as your project's Swiss Army knife.

---

## Chapter 3: Your First Rust Program

### Hello, World!

Let's look at the code Cargo created for us. Open `src/main.rs`:

```rust
fn main() {
    println!("Hello, world!");
}
```

That's it. Six words of actual code. But there's a lot packed in here, so let's break it down.

**`fn main()`** — This defines a function called `main`. In Rust, every executable program needs a `main` function — it's where the computer starts running your code. The `fn` keyword means "function." The empty parentheses `()` mean this function takes no inputs.

**`println!("Hello, world!");`** — This prints text to the screen. The `!` after `println` means it's a **macro** (we'll explain that in a moment). The text inside the quotes is what gets printed. The semicolon `;` at the end marks the end of the statement.

💡 **Key Concept**

**Macros vs Functions**

You might notice that `println!` has an exclamation mark, but `main` doesn't. That's because `println!` is a **macro**, not a regular function. Macros in Rust are special — they're like functions that get expanded before your code runs. You can recognize them by the `!`. Don't worry too much about this right now — just remember that `println!` always needs that exclamation mark.

### Variables and Mutability

In Rust, you create variables with `let`:

```rust
fn main() {
    let name = "FicHub";
    let version = 1;
    let is_cool = true;

    println!("{} v{}", name, version);
    println!("Is it cool? {}", is_cool);
}
```

Here we have three variables:
- `name` is a **string** — a piece of text
- `version` is an **integer** — a whole number
- `is_cool` is a **boolean** — true or false

When you run this, you'll see:

```
FicHub v1
Is it cool? true
```

But here's something interesting about Rust: **variables are immutable by default**. That means once you set a value, you can't change it. Try this:

```rust
fn main() {
    let version = 1;
    version = 2;  // ERROR!
}
```

This won't compile. You'll get an error:

```
error[E0384]: cannot assign twice to immutable variable `version`
```

If you *want* to change a variable, you need to use `mut` (short for "mutable"):

```rust
fn main() {
    let mut version = 1;
    println!("Version: {}", version);

    version = 2;
    println!("Version: {}", version);
}
```

Now it compiles and runs:

```
Version: 1
Version: 2
```

⚠️ **Watch Out**

Making everything `mut` is a bad habit. Rust makes variables immutable by default for a reason — it helps prevent bugs. Only use `mut` when you actually need to change a value. If you see an error about "cannot assign twice," think about whether that variable really *should* be changing.

🧪 **Try It Yourself**

Create a new file called `variables.rs` with this code:

```rust
fn main() {
    let apples = 5;
    let mut bananas = 10;

    println!("I have {} apples", apples);
    println!("I have {} bananas", bananas);

    bananas = bananas + 5;
    println!("Now I have {} bananas", bananas);
}
```

Run it with `rustc variables.rs && ./variables`. What happens if you try to change `apples`?

### Basic Types

Rust has several basic types you'll use all the time:

**Integers** — whole numbers:

```rust
let age: i32 = 25;       // 32-bit signed integer
let big_number: i64 = 1000000;  // 64-bit signed integer
let count: u32 = 42;     // 32-bit UNSIGNED integer (no negatives)
```

**Floating-point numbers** — decimal numbers:

```rust
let temperature: f64 = 98.6;    // 64-bit float
let pi: f32 = 3.14159;          // 32-bit float
```

**Booleans** — true or false:

```rust
let is_active: bool = true;
let has_permission: bool = false;
```

**Strings** — text:

```rust
let greeting: &str = "Hello!";          // A string slice
let mut name = String::from("FicHub");  // An owned string
name.push_str(" Server");               // We can modify owned strings
```

💡 **Key Concept**

**`&str` vs `String`**

This confuses a lot of newcomers, so let's clear it up:

- `&str` (pronounced "string slice") is a reference to text that lives somewhere else. It's lightweight and fast to pass around. Think of it as pointing to a sign that says "Hello!" — you can read it, but you can't change it.

- `String` is an owned piece of text that your code controls. You can modify it, grow it, shrink it. Think of it as having your own whiteboard where you can write "Hello!" and erase it later.

In general, use `&str` when you just need to read text, and `String` when you need to build or modify text.

### Functions and Parameters

Functions are reusable blocks of code. Here's how you write one:

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    greet("Alice");
    greet("Bob");

    let result = add(3, 4);
    println!("3 + 4 = {}", result);
}
```

Let's break this down:

- **`fn greet(name: &str)`** — a function called `greet` that takes one parameter, `name`, which is a string slice.
- **`fn add(a: i32, b: i32) -> i32`** — a function that takes two integers and *returns* an integer. The `-> i32` part is the return type.
- In `add`, we didn't write `return a + b;` — in Rust, the last expression in a function is automatically returned. No semicolon needed!

Running this gives:

```
Hello, Alice!
Hello, Bob!
3 + 4 = 7
```

⚠️ **Watch Out**

Notice that `add` doesn't have a semicolon after `a + b`. In Rust, a **statement** (ends with `;`) doesn't return a value, but an **expression** (no `;`) does. If you accidentally add the semicolon, you'll get a confusing error. When a function is supposed to return something, don't put a semicolon on the last line!

### Ownership: Rust's Superpower

Okay, this is the part where Rust gets *really* interesting. Ownership is Rust's most unique feature, and it's what makes Rust so fast and safe.

Here's the basic idea: **every piece of data has exactly one owner**. When the owner goes out of scope (like when a function ends), the data is automatically cleaned up. No garbage collector needed!

```rust
fn main() {
    let name = String::from("FicHub");  // `name` owns the string

    let name2 = name;                     // Ownership MOVES to `name2`

    // println!("{}", name);  // ERROR! `name` no longer owns the string

    println!("{}", name2);    // This works! `name2` is the owner now
}
```

When we say `let name2 = name;`, the ownership of the string *moves* from `name` to `name2`. After that, `name` is no longer valid. This prevents bugs like double-free errors (where two parts of your code both try to clean up the same data).

Think of it like this: if you have a toy and you give it to your friend, you can't play with it anymore. You gave it away. That's exactly what ownership is — you give your data to another variable, and the original variable can't use it anymore.

But what if you want to *share* data? That's where **borrowing** comes in:

```rust
fn greet(name: &String) {
    println!("Hello, {}!", name);
}

fn main() {
    let name = String::from("FicHub");
    greet(&name);           // We BORROW `name` to greet
    println!("Still have: {}", name);  // We still own it!
}
```

By using `&name`, we're *lending* the string to the `greet` function without giving up ownership. The `greet` function can read it, but when it's done, the string comes back to us. This is called a **reference** or **borrow**.

Think of it like lending a book to a friend. They can read it, but it's still *your* book. When they're done, they give it back. That's borrowing.

⚠️ **Watch Out**

You might wonder: "If ownership moves, doesn't that make Rust annoying to use?" Not really! Once you get the hang of it, it feels natural. And the safety it provides is worth the small learning curve. Plus, borrowing (which we'll use all the time) lets you share data without moving it.

There are two kinds of borrowing:

- **Immutable borrowing** (`&T`): You can have as many readers as you want. Nobody can modify the data while anyone is reading it.
- **Mutable borrowing** (`&mut T`): You can have exactly one writer. Nobody else can read or write while the writer is active.

```rust
fn add_tag(tags: &mut Vec<String>, tag: &str) {
    tags.push(tag.to_string());
}

fn print_tags(tags: &Vec<String>) {
    for tag in tags {
        println!("Tag: {}", tag);
    }
}

fn main() {
    let mut tags = vec![
        "fantasy".to_string(),
        "romance".to_string(),
    ];

    print_tags(&tags);      // Immutable borrow — just reading
    add_tag(&mut tags, "slow-burn");  // Mutable borrow — modifying
    print_tags(&tags);      // Back to immutable borrow
}
```

💡 **Key Concept**

**Ownership Rules**

There are three simple rules:

1. Each value has exactly one owner
2. When the owner goes out of scope, the value is dropped (freed)
3. You can have either one mutable reference OR any number of immutable references — but never both at the same time

Rule 3 prevents data races. If multiple parts of your code can read data at the same time, that's fine. But if someone wants to *write* to the data, nobody else should be reading it at that moment. Rust enforces this at compile time!

In FicHub, this means:
- Multiple request handlers can read from the database at the same time (immutable borrows)
- Only one request handler can write to a particular piece of data at a time (mutable borrow)
- You can never accidentally free memory that someone else is still using

This is how Rust achieves both safety and speed — no runtime checks needed!

### The println! Macro in Detail

We've been using `println!` since chapter 1, but let's look at it more carefully. It's the most common way to print things in Rust:

```rust
fn main() {
    let name = "FicHub";
    let version = 2;
    let chapters = 42;

    // Basic printing
    println!("Hello, world!");

    // Printing variables with {}
    println!("Project: {}", name);

    // Multiple variables
    println!("{} v{} has {} chapters", name, version, chapters);

    // Named parameters (extra cool!)
    println!("{project} v{version} has {chapters} chapters",
             project = name, version = version, chapters = chapters);

    // Debug printing with {:?}
    let numbers = vec![1, 2, 3, 4, 5];
    println!("Numbers: {:?}", numbers);

    // Format options
    println!("{:>10}", "right");    // Right-align
    println!("{:<10}", "left");     // Left-align
    println!("{:#?}", (1, 2, 3));   // Pretty-print
}
```

The `{}` is a placeholder. Think of it like a blank line in a fill-in-the-blanks exercise:

- `{}` — print the value here
- `{name}` — print the variable called `name`
- `{:#?}` — pretty-print the Debug representation

### Comments

Comments are notes for humans. The computer ignores them. Rust has two kinds:

```rust
// This is a single-line comment

fn main() {
    // You can put comments anywhere
    let x = 5; // Even at the end of a line

    /*
     * This is a multi-line comment.
     * It's useful for longer explanations.
     * You can write as many lines as you want.
     */
    let y = 10;
}
```

For documentation, Rust has a special kind of comment using `///`:

```rust
/// Adds two numbers together.
///
/// # Arguments
///
/// * `a` - The first number
/// * `b` - The second number
///
/// # Examples
///
/// ```
/// let result = add(2, 3);
/// assert_eq!(result, 5);
/// ```
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

These `///` comments become documentation that can be turned into HTML with `cargo doc`. It's like having a user manual built right into your code!

🧪 **Try It Yourself**

Write a program that:
1. Creates variables for your name and age
2. Uses `println!` to print "My name is [name] and I am [age] years old"
3. Adds a comment explaining what the program does

Here's a starter:

```rust
// Introduction program
fn main() {
    let name = "Your Name";
    let age = 12;

    println!("My name is {} and I am {} years old!", name, age);
}
```

### Putting It All Together

Let's write a slightly bigger program that uses everything we've learned:

```rust
/// FicHub mini-demo: calculates story statistics
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn reading_time(word_count: usize) -> f64 {
    // Average reading speed: 250 words per minute
    word_count as f64 / 250.0
}

fn main() {
    let title = String::from("The Fellowship of the Fic");
    let author = "AnonymousAuthor123";
    let summary = "A story about heroes, friendship, and the power of \
                    community. Contains 5000 words of adventure and heart.";

    // Count words in the summary
    let words = word_count(summary);
    let time = reading_time(words);

    println!("=== FicHub Story Stats ===");
    println!("Title:   {}", title);
    println!("Author:  {}", author);
    println!("Words:   {}", words);
    println!("Est. reading time: {:.1} minutes", time);

    // Ownership demo: we can still use `title` because we used &str
    println!("Download: {}", title);
}

/// Counts the number of words in a text
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Calculates reading time in minutes (at 250 WPM)
fn reading_time(words: usize) -> f64 {
    words as f64 / 250.0
}
```

When you run this:

```
=== FicHub Story Stats ===
Title:   The Fellowship of the Fic
Author:  AnonymousAuthor123
Words:   21
Est. reading time: 0.1 minutes
Download: The Fellowship of the Fic
```

Notice how we used `&str` for `author` (just reading text) but `String` for `title` (we might want to modify it later). We also used functions to break our program into small, reusable pieces.

### What We've Learned

In this chapter, you've learned:

- **Variables** — how to store data with `let`, and when to use `mut`
- **Types** — integers, floats, booleans, and strings
- **Functions** — how to write reusable code with parameters and return values
- **Ownership** — Rust's unique system for managing memory safely
- **Printing** — how to format and display text with `println!`
- **Comments** — how to annotate your code for humans

These are the building blocks of every Rust program. Everything else we build in this book — the web server, the database queries, the EPUB generator — is made from these same pieces.

Let's recap the most important ideas:

**Variables are immutable by default.** This is unusual compared to other languages, but it's one of Rust's superpowers. When you see a variable without `mut`, you know it will never change. That's one less thing to worry about.

**Functions are declared with `fn`.** They take parameters (inputs) and can return values (outputs). The return type goes after an arrow `->`. Rust's last-expression-without-semicolon trick makes functions concise.

**Ownership prevents bugs.** Every value has one owner. When the owner goes out of scope, the value is freed. You can move ownership or borrow it (with `&`). This system eliminates entire categories of bugs at compile time.

**Rust is compiled, not interpreted.** When you run `cargo build`, Rust compiles your source code into a binary file. This binary runs directly on your computer — no interpreter needed. That's why Rust is so fast.

In the next chapter, we'll start building FicHub for real. We'll add our first dependencies, create our first web route, and see our server respond to its very first HTTP request. We'll take everything we've learned here and put it to work.

But for now, take a moment to celebrate. You've written your first Rust program. You understand variables, functions, and even ownership. You're officially a Rust programmer! 🎉

If you want to practice more before moving on, try these challenges:

1. Write a function that takes two strings and returns them concatenated together
2. Create a program that calculates the area of a rectangle given its width and height
3. Write a function that takes a number and returns true if it's even, false if it's odd

These are simple exercises, but they'll help solidify the concepts we've covered. The more you practice, the more natural Rust will feel.

Welcome to the workshop. Let's build something amazing.
