# Part 2: Hello, Axum!

We've got the basics under our belt — variables, functions, ownership, and the tools of the trade. Now it's time to do something exciting: build a real web server that listens for HTTP requests and sends back responses.

This is where FicHub starts to come alive.

---

## Chapter 4: Hello Axum

### What Is a Web Server?

Before we write any code, let's make sure we understand what we're building. A **web server** is a program that sits on your computer, waits for someone to send it a request over the internet, and sends back a response. That's it!

When you type `https://archiveofourown.org` into your browser, your browser sends an **HTTP request** to AO3's servers. The server looks at what you're asking for (the home page, a specific story, etc.) and sends back an **HTTP response** — usually a web page written in HTML.

Here's the conversation between your browser and a server:

```
Your browser:  "Hey server, give me the page at /stories/12345"
Server:        "Sure thing! Here's the HTML for that story."
```

The request has:
- A **method** (GET to read something, POST to send something)
- A **path** (the URL, like `/stories/12345`)
- **Headers** (extra info, like what browser you're using)

The response has:
- A **status code** (200 means "OK, here you go", 404 means "not found")
- **Headers** (extra info about the response)
- A **body** (the actual content — HTML, JSON, an image, etc.)

FicHub is going to be one of these servers. When someone requests an EPUB download, we'll scrape the story, build the file, and send it back. But before we get to all that, let's start with the simplest possible server.

### Choosing Axum

There are lots of web frameworks in Rust — Actix Web, Rocket, Warp, and more. We're going to use **Axum**. Why?

Axum is built by the same team behind Tokio (our async runtime). It's designed to work beautifully with Tokio's features, it's type-safe (which means the compiler helps you catch bugs), and it's modern and well-maintained. It's also really pleasant to use — which matters when you're building something big.

💡 **Key Concept**

**What's a Framework?** A web framework is a toolbox that gives you the building blocks for a web server. It handles the boring stuff — parsing HTTP requests, managing TCP connections, routing requests to the right handler function — so you can focus on writing the fun stuff: your application logic.

### Adding Dependencies

Open your `Cargo.toml` file and add two dependencies:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

That's it for now. Let's talk about what each one does:

- **axum** is the web framework. It provides routing, request parsing, response building, and all the web server goodies.
- **tokio** is the **async runtime**. In Rust, we use async code to handle many things at once (like serving thousands of requests). Tokio is the engine that makes that possible. The `features = ["full"]` part means "give me everything Tokio has to offer."

After adding these lines, run:

```bash
cargo build
```

Cargo will download axum, tokio, and all their sub-dependencies. The first build takes a few minutes — grab a snack!

### Your First Axum Server

Let's write the simplest possible web server. Replace the contents of `src/main.rs` with:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // Create a router with one route
    let app = Router::new()
        .route("/hello", get(hello_handler));

    // Bind to a port and start serving
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn hello_handler() -> &'static str {
    "Hello, world!"
}
```

There's a lot here, so let's break it down piece by piece.

### Breaking It Down

**The `#[tokio::main]` macro:**

```rust
#[tokio::main]
async fn main() {
```

Remember `fn main()` from Chapter 3? This is the same thing, but with two additions:

1. `#[tokio::main]` — This is an **attribute macro** that sets up the Tokio async runtime for us. It's like putting on a special pair of glasses that lets us see (and use) async code.
2. `async` — This keyword means "this function can pause and resume." We'll explain why in a moment.

**The Router:**

```rust
let app = Router::new()
    .route("/hello", get(hello_handler));
```

A `Router` maps URL paths to handler functions. This line says: "When someone requests the `/hello` path using a GET request, call `hello_handler`."

**The listener:**

```rust
let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
    .await
    .expect("Failed to bind to port 3000");
```

This creates a TCP listener — a socket that waits for incoming connections on port 3000. The `0.0.0.0` means "listen on all network interfaces" (so other computers on your network can reach it too).

**Starting the server:**

```rust
axum::serve(listener, app)
    .await
    .expect("Server error");
```

This starts the server and keeps it running forever. The `.await` means "wait here and keep serving requests."

**The handler function:**

```rust
async fn hello_handler() -> &'static str {
    "Hello, world!"
}
```

This is the function that runs when someone visits `/hello`. It returns a `&'static str` — a string that lives for the entire program (the `"Hello, world!"` literal). Axum knows how to turn this into an HTTP response with status 200 and the text as the body.

### Running the Server

Save your file and run:

```bash
cargo run
```

You should see:

```
Server running on http://localhost:3000
```

Your server is alive! Open another terminal and try:

```bash
curl http://localhost:3000/hello
```

You should see:

```
Hello, world!
```

🎉 Congratulations! You just built a web server! Someone sent an HTTP request to your server, and your server responded. That's the fundamental loop of every web application in the world.

🧪 **Try It Yourself**

1. Open your browser and go to `http://localhost:3000/hello`. You'll see the same response.
2. Try visiting `http://localhost:3000/` — you'll get a 404 error. That's because we only defined a route for `/hello`.
3. Try visiting `http://localhost:3000/hello` with a POST request: `curl -X POST http://localhost:3000/hello`. You'll also get a 404 — because we only registered a GET handler.

### Adding More Routes

Let's add a few more routes to our server. Update `src/main.rs`:

```rust
use axum::{routing::get, Router, Json};
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/hello", get(hello_handler))
        .route("/about", get(about_handler))
        .route("/api/info", get(api_info_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}

async fn hello_handler() -> &'static str {
    "Hello, world!"
}

async fn about_handler() -> &'static str {
    "FicHub is a fanfiction download server."
}

async fn api_info_handler() -> Json<serde_json::Value> {
    Json(json!({
        "name": "fichub",
        "version": "0.1.0",
        "status": "running"
    }))
}
```

Notice something new: the `Json` type and the `json!` macro. The `json!` macro (from the `serde_json` crate) lets us create JSON data easily, and `Json` wraps it so Axum knows to send it with the correct `Content-Type: application/json` header.

Test the new routes:

```bash
curl http://localhost:3000/about
curl http://localhost:3000/api/info
```

The second one returns beautiful JSON:

```json
{"name":"fichub","version":"0.1.0","status":"running"}
```

This is exactly the pattern FicHub uses for its API responses!

### Understanding async/await

You've been seeing `async` and `.await` everywhere, so let's take a moment to understand what they mean.

In regular (synchronous) code, when you call a function, your program waits for it to finish before moving on:

```rust
let data = read_file("story.txt");  // Program waits here
println!("{}", data);               // Only runs after file is read
```

But what if reading the file takes 2 seconds? What if 100 people are requesting files at the same time? You'd need 100 threads, each waiting for its own file.

**Async code** is different. When an async function needs to wait for something (like reading a file or waiting for a network request), it **pauses** and lets other tasks run:

```rust
let data = read_file("story.txt").await;  // Pauses here, but others can run
println!("{}", data);                       // Runs when the file is ready
```

The `async` keyword on a function means "this function might pause." The `.await` keyword means "pause here until this is ready, then continue."

This is incredibly powerful for a web server. When 100 people request EPUBs at the same time, our server can start processing each request, pause when it needs to wait (for a database query, a web scrape, a file write), and let other requests run in the meantime. All on a single thread!

💡 **Key Concept**

**Async is not parallelism.** Async means doing many things by switching between them quickly (like a chef cooking multiple dishes by working on whichever one needs attention). Parallelism means doing many things at the same time (like having multiple chefs). Tokio gives us both, but async/await is the switching part.

Here's a concrete example. Imagine a handler that needs to fetch a story from a website, save it to the database, and return the result:

```rust
async fn download_story(url: String) -> Result<Story, AppError> {
    let html = reqwest::get(&url).await?.text().await?;  // Network I/O — can pause
    let story = parse_story(&html)?;                       // CPU work — fast
    sqlx::query("INSERT INTO stories ...").execute(&pool).await?;  // Database I/O — can pause
    Ok(story)
}
```

Both `.await` points are I/O operations — waiting for a network response or a database query. While this handler is paused, Tokio is free to process other requests. That's how our server handles thousands of concurrent users with just a handful of threads.

⚠️ **Watch Out**

You can only call `.await` inside an `async` function (or a `tokio::spawn` block). If you try to use `.await` in a regular `fn`, the compiler will give you a confusing error. Always make sure your function is marked `async` if you want to use `.await` inside it.

### Where We're Headed

In the real FicHub `server.rs`, the router looks a lot more complex than our example — it has routes for EPUB export, metadata, recommendations, tags, search, OPDS catalogs, and more:

```rust
Router::new()
    .route("/api/", get(routes::api_docs::api_docs_handler))
    .route("/api/v0/epub", get(routes::export::epub_handler))
    .route("/api/v0/meta", get(routes::meta::meta_handler))
    .route("/api/v0/search", get(crate::search::routes::search_handler))
    // ... many more routes
```

But the pattern is exactly the same: create a router, add routes, bind to a port, and serve. Each route maps a path to a handler function. You now understand the fundamentals — everything else is just more routes.

### What We've Learned

In this chapter, you've learned:

- **HTTP requests and responses** — the conversation between browsers and servers
- **Axum** — Rust's modern web framework
- **Router** — mapping URL paths to handler functions
- **Handlers** — functions that process requests and return responses
- **Json responses** — sending JSON data with the `json!` macro
- **async/await** — how Rust handles many things at once
- **Running your server** — `cargo run`, then `curl` to test

In the next chapter, we'll make our server configurable — reading settings from environment variables so we can run it in different environments without changing the code.

---

## Chapter 5: Configuration

### Why Configuration Matters

Right now, our server is hardcoded to listen on port 3000. What if port 3000 is already in use? What if we want to run on a different port in production? What if we need to connect to a different database?

The answer is **configuration** — settings that live outside your code so you can change them without recompiling. FicHub reads all its settings from **environment variables**, which is a common pattern in production software.

Think of it like this: imagine you have a robot that makes sandwiches. Instead of hardcoding "use cheddar cheese," you give it a note card that says "use cheddar." If you want swiss cheese tomorrow, you just change the note card — you don't have to rebuild the robot.

Environment variables are those note cards.

### Loading .env Files

For development, we'll use the `dotenvy` crate to load settings from a `.env` file. Add it to your `Cargo.toml`:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
dotenvy = "0.15"
```

Now create a `.env` file in your project root (next to `Cargo.toml`):

```
DATABASE_URL=postgres://localhost/fichub
REDIS_URL=redis://localhost/0
PORT=3000
```

These are the three most important settings for FicHub. The `dotenvy` crate reads this file and puts all the values into environment variables before your program starts.

In `main.rs`, load the `.env` file at the very top:

```rust
#[tokio::main]
async fn main() {
    // Load .env if present
    dotenvy::dotenv().ok();

    // ... rest of your server code
}
```

The `.ok()` at the end is important — it means "try to load .env, but don't panic if it's missing." This way, your server works both in development (with a `.env` file) and in production (where environment variables are set directly).

### Creating a Config Struct

Rather than reading environment variables scattered throughout your code, FicHub groups them all into a `Config` struct. This is cleaner, easier to test, and catches typos early.

Here's a simplified version:

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub cache_dir: PathBuf,
    pub app_port: u16,
    pub frontend_dir: PathBuf,
}
```

The `#[derive(Debug, Clone)]` line tells Rust to automatically generate two things:

- **Debug** — lets you print the config for debugging with `println!("{:?}", config)`
- **Clone** — lets you make copies of the config (needed because Axum handlers might need their own copy)

### The `from_env()` Pattern

Here's how FicHub loads its config from environment variables. We'll use a method called `Config::from_env()`:

```rust
impl Config {
    pub fn from_env() -> Self {
        // Required variables — panic if missing
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let redis_url = std::env::var("REDIS_URL")
            .expect("REDIS_URL must be set");

        // Optional variables — use defaults
        let cache_dir = std::env::var("CACHE_DIR")
            .unwrap_or_else(|_| "./cache".to_string());

        let app_port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let frontend_dir = std::env::var("FRONTEND_DIR")
            .unwrap_or_else(|_| "./frontend/build".to_string());

        Config {
            database_url,
            redis_url,
            cache_dir: PathBuf::from(cache_dir),
            app_port,
            frontend_dir: PathBuf::from(frontend_dir),
        }
    }
}
```

Let's look at the patterns here:

**Required variables** use `.expect()`:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");
```

If `DATABASE_URL` isn't set, the program panics with a clear error message. This is intentional — if the database URL is missing, we can't do anything useful, so we fail fast and loudly.

**Optional variables with defaults** use `.unwrap_or_else()`:

```rust
let app_port = std::env::var("PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse()
    .unwrap_or(3000);
```

This says: "Try to read `PORT`. If it's not set, use `"3000"`. Then parse it as a number. If parsing fails, use `3000`." There are two layers of fallbacks here — one for the missing variable and one for the malformed value.

**Optional variables that might not exist** use `.ok()`:

```rust
let secondary_cache_dir = std::env::var("SECONDARY_CACHE_DIR")
    .ok()                                    // Convert Result to Option
    .filter(|s| !s.is_empty())              // Ignore empty strings
    .map(PathBuf::from);                     // Convert to PathBuf if present
```

This is how you handle "nice to have" settings. If the variable isn't set, `secondary_cache_dir` is just `None`.

### Using the Config

Now we can use the config in our server. Here's an updated `main.rs`:

```rust
mod config;

use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    println!("Starting server on port {}", config.app_port);

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, world!" }));

    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
```

Now we can change the port by editing `.env` or setting an environment variable:

```bash
PORT=8080 cargo run
```

Your server now listens on port 8080!

### The Real FicHub Config

The actual FicHub `Config` struct has many more fields — over 30 of them! Here are the important ones:

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string |
| `REDIS_URL` | ✅ | — | Redis connection string |
| `PORT` | ❌ | `3000` | Server listening port |
| `CACHE_DIR` | ❌ | `./cache` | Where to store cached EPUBs |
| `TMP_DIR` | ❌ | `./tmp` | Temporary files directory |
| `FRONTEND_DIR` | ❌ | `./frontend/build` | Where the frontend files live |
| `NODE_NAME` | ❌ | `orion` | Name of this server instance |
| `DYNAMIC_RATE_LIMIT` | ❌ | `true` | Enable dynamic rate limiting |

### Updating the .env File

Let's expand our `.env` file:

```
DATABASE_URL=postgres://localhost/fichub
REDIS_URL=redis://localhost/0
PORT=3000
CACHE_DIR=./cache
TMP_DIR=./tmp
FRONTEND_DIR=./frontend/build
NODE_NAME=dev-server
```

🧪 **Try It Yourself**

1. Change the `PORT` value in `.env` to `4000` and run `cargo run`. Your server now listens on port 4000.
2. Try setting `PORT=5000 cargo run` — the command-line environment variable overrides the `.env` file.
3. Delete the `DATABASE_URL` line from `.env` and run the server. It should panic with `"DATABASE_URL must be set"`. That's our safety net!

⚠️ **Watch Out**

**Never commit your `.env` file to Git.** It contains connection strings with passwords! Add it to `.gitignore`:

```bash
echo ".env" >> .gitignore
```

The `.env` file is just for your local development. In production, environment variables are set by your deployment tool (Docker, Kubernetes, systemd, etc.).

### What We've Learned

In this chapter, you've learned:

- **Environment variables** — settings that live outside your code
- **dotenvy** — loading settings from a `.env` file in development
- **Config struct** — grouping related settings into one type
- **`from_env()` pattern** — reading env vars with required/optional handling
- **Required vs optional** — using `.expect()` for must-haves and defaults for nice-to-haves
- **The key variables** — DATABASE_URL, REDIS_URL, PORT

In the next chapter, we'll tackle one of the most important topics in any real application: error handling. Things go wrong — servers crash, databases lose connections, files go missing. We'll learn how Rust helps us handle all of these gracefully.

---

## Chapter 6: Error Handling

### Why Error Handling Matters

Everything breaks. Databases lose connections. Files go missing. Websites we scrape go down. Users send bad requests. A well-built server doesn't just work when everything is perfect — it works gracefully when things go wrong.

In Rust, error handling is a first-class feature. The compiler *forces* you to think about what happens when things fail. This is different from languages like Python or JavaScript, where errors can silently propagate and crash your program at the worst possible moment.

FicHub handles dozens of different error types: database errors, Redis errors, scraping failures, export failures, bad requests, rate limiting, and more. Each one needs a different response. Let's learn how.

### The Result Type

In Rust, functions that can fail return a `Result` type. A `Result` is either `Ok(value)` (success!) or `Err(error)` (something went wrong):

```rust
use std::fs;

fn read_story(path: &str) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}
```

The `?` operator is the magic here. It means: "If this is an error, return it immediately. If it's OK, give me the value and keep going." Without `?`, you'd have to write ugly nested `match` statements for every operation.

Let's see the difference:

**Without `?`:**

```rust
fn read_story(path: &str) -> Result<String, std::io::Error> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => Err(e),
    }
}
```

**With `?`:**

```rust
fn read_story(path: &str) -> Result<String, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content)
}
```

Same thing, half the code. The `?` operator is one of Rust's best features.

💡 **Key Concept**

**The `?` operator** automatically converts between error types (when you've implemented the right traits) and returns early on failure. Think of it as saying "try this, and if it fails, bail out with the error."

### Creating an AppError Enum

FicHub has many different types of errors. Instead of returning a different error type for each function, we create one unified error type that covers everything:

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    /// Bad request with error code and message
    BadRequest(i32, String),
    /// Rate limited — wait N seconds
    RateLimited(u64),
    /// Resource not found
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Scraper error
    ScrapeError(String),
    /// Database error
    Database(String),
    /// Cache error
    CacheError(String),
}
```

Each variant represents a different category of failure. When something goes wrong, we wrap it in the appropriate variant.

### Implementing IntoResponse

The magic part: Axum knows how to turn an `AppError` into an HTTP response. We implement the `IntoResponse` trait:

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

Notice two important things:

1. **Different status codes for different errors.** A bad request returns 400, rate limiting returns 429, not found returns 404, and internal errors return 500. This is exactly what HTTP clients expect.

2. **Sensitive information is hidden.** When there's an internal error, the response says `"internal server error"` — it doesn't leak the actual error details to the user. But we do log it with `tracing::error!` so developers can see what went wrong.

### Converting Between Error Types

To use the `?` operator with our AppError, we need to tell Rust how to convert other error types into `AppError`. This is done with `From` implementations:

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
```

Now the `?` operator works seamlessly. If a database query fails, `?` automatically converts the `sqlx::Error` into an `AppError::Database` and returns it. You never have to think about the conversion — Rust handles it.

### The AppResult Type Alias

To keep things clean, FicHub defines a type alias:

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Now handler functions can return `AppResult<Json<Value>>` instead of `Result<Json<Value>, AppError>`. Shorter and clearer.

### Using AppError in Handlers

Here's a complete example of a handler that uses AppError:

```rust
use axum::extract::Path;
use axum::Json;
use serde_json::json;

async fn get_story(
    Path(story_id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate the ID
    if story_id <= 0 {
        return Err(AppError::BadRequest(
            1,
            "Story ID must be positive".to_string(),
        ));
    }

    // Try to find the story in the database
    let story = find_story_in_db(story_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Try to find the story
    match story {
        Some(s) => Ok(Json(json!({
            "id": s.id,
            "title": s.title,
            "author": s.author,
        }))),
        None => Err(AppError::NotFound(
            format!("Story {} not found", story_id)
        )),
    }
}
```

This handler can fail in three ways:
1. Bad input → `AppError::BadRequest` (400)
2. Database failure → `AppError::Database` (500)
3. Story not found → `AppError::NotFound` (404)

Each failure produces the correct HTTP status code and a JSON error body. And because we implemented `IntoResponse`, Axum handles the conversion automatically.

### The Pattern: Ok(Json(json!({...})))

In the real FicHub, you'll see this pattern everywhere:

```rust
Ok(Json(json!({
    "title": story.title,
    "author": story.author,
    "chapters": story.chapters,
})))
```

This is the success path: wrap your data in `json!{}`, wrap that in `Json()`, and wrap that in `Ok()`. It's a little nested, but it's clear and type-safe. Axum knows exactly what to do with it.

🧪 **Try It Yourself**

1. Create a handler that returns different errors based on an input parameter. For example, if the ID is 0, return `BadRequest`; if it's negative, return `NotFound`; if it's positive, return `Ok` with some JSON.
2. Try removing one of the `From` implementations and see what error the compiler gives you. (Hint: it'll say something about not being able to convert between types.)
3. Create an error with an empty message and see what the JSON response looks like.

### Why Not Just Panic?

You might wonder: why not just use `panic!()` or `.unwrap()` when something goes wrong? In a small program, that's fine. But in a web server, a panic means the entire server crashes. One bad request takes down every user.

With `AppError`, a bad request returns a clean error response and the server keeps running. The next request works perfectly. That's the difference between a toy and a production server.

⚠️ **Watch Out**

**Don't expose internal errors to users.** Notice how `AppError::Internal` returns `"internal server error"` instead of the actual error message. If you leak error details (like SQL queries or file paths), you're giving attackers a roadmap to your system. Log the details with `tracing::error!`, but send generic messages to the user.

### What We've Learned

In this chapter, you've learned:

- **Result type** — Rust's way of handling success and failure
- **The `?` operator** — automatic error propagation
- **AppError enum** — a unified error type for all failure modes
- **IntoResponse** — converting errors to HTTP responses
- **From implementations** — automatic error conversion with `?`
- **Error hiding** — keeping internal details away from users
- **The pattern** — `Ok(Json(json!({...})))` for success, `Err(AppError::*)` for failure

In the next chapter, we'll learn about shared state — how to give all our handlers access to the database, Redis, and configuration they need.

---

## Chapter 7: Shared State

### Why Shared State Matters

Our server can handle requests now, but there's a problem: what if a handler needs to query the database? Or check the cache in Redis? Or read the configuration?

Each handler is just a function — it doesn't automatically have access to anything outside itself. We need a way to *share* resources across all handlers.

In FicHub, every handler needs access to:
- The **database** pool (for PostgreSQL queries)
- The **Redis** connection (for caching)
- The **configuration** (for settings like port, paths, etc.)
- The **HTTP client** (for scraping stories from the web)
- The **scraper registry** (which scraper handles which site)

We need to pass all of these to every handler somehow. That's where shared state comes in.

### Arc\<T\>: Thread-Safe Sharing

Remember from Chapter 3 that Rust's ownership system means only one thing can own a value at a time. But our handlers all need to *share* the same database pool, the same Redis connection, the same config. We can't give each handler its own copy — that would waste memory and break connections.

The solution is `Arc<T>` — **A**tomically **R**eference-**C**ounted. `Arc` lets multiple parts of your program share ownership of the same value. It's like having a library book with multiple checkout cards — many people can read it, and when the last person returns it, the book is finally recycled.

```rust
use std::sync::Arc;

let config = Arc::new(some_value);
let config_clone = Arc::clone(&config);
// Both `config` and `config_clone` point to the same data
```

`Arc` is safe to use across threads (that's the "atomically" part). When the last `Arc` pointing to a value is dropped, the value is freed. No memory leaks, no dangling pointers.

💡 **Key Concept**

**Arc vs Rc:** Rust has two reference-counted types: `Arc` and `Rc`. `Arc` (Atomic Reference Count) is thread-safe — you can share it across async tasks and threads. `Rc` (Reference Count) is not thread-safe but slightly faster. In a web server, always use `Arc` — your handlers run on Tokio's thread pool, so you need thread safety.

### Creating AppState

FicHub groups all shared resources into a single `AppState` struct:

```rust
use std::sync::Arc;
use crate::config::Config;

pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
}
```

Each field is a resource that handlers need. The database pool (`sqlx::PgPool`) is already designed to be shared — it manages its own pool of connections internally. The Redis connection uses a multiplexed connection that's safe to share. The HTTP client is also safe to share.

### Building the State

Here's how FicHub builds its state in `server.rs`:

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

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
        http_client,
    });

    // Build router and start serving
    let app = build_router(state).await;
    // ... bind and serve
}
```

Notice the flow:
1. Connect to PostgreSQL (creates a connection pool)
2. Connect to Redis (creates a multiplexed connection)
3. Build an HTTP client
4. Wrap everything in `Arc<AppState>`
5. Pass the state to the router

⚠️ **Watch Out**

**Connect before you serve.** Notice that we connect to the database and Redis *before* starting the server. If the connections fail, we get a clear error message at startup instead of mysterious failures when the first request arrives. Failing fast is always better than failing mysteriously.

### Passing State to Handlers

Here's where the magic happens. Axum has a special extractor called `State` that injects shared state into your handlers:

```rust
use axum::{extract::State, Json};
use std::sync::Arc;

async fn health_check(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    // We can access state.db, state.redis, state.config, etc.
    Json(serde_json::json!({
        "status": "ok",
        "node": state.config.node_name,
    }))
}
```

The `State(state): State<Arc<AppState>>` part is a **destructuring pattern**. It says: "Extract the shared state, call it `state`, and make it an `Arc<AppState>`."

### Wiring It Together

The state is passed to the router with `.with_state()`:

```rust
async fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/v0/meta", get(meta_handler))
        // ... more routes
        .with_state(state)   // <-- This passes the state to all handlers
}
```

The `.with_state()` call at the end makes the state available to every handler through the `State` extractor. Handlers that don't need it simply don't include the `State` parameter — Axum is smart enough to know the difference.

### A Real Handler Example

Let's write a handler that queries the database using our shared state:

```rust
use axum::{extract::State, Json};
use std::sync::Arc;
use serde_json::json;

async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Query the database using the shared pool
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stories")
        .fetch_one(&state.db)
        .await?;

    Ok(Json(json!({
        "total_stories": row.0,
        "node": state.config.node_name,
    })))
}
```

This handler:
1. Receives the shared `AppState` through the `State` extractor
2. Uses `state.db` to run a SQL query
3. Uses `state.config.node_name` to include the server name in the response
4. Returns the result as JSON (or an `AppError` if something fails)

The `?` operator after `fetch_one` means: if the database query fails, convert the error to `AppError::Database` and return it immediately.

### Connecting to a Database Pool

The `sqlx::PgPool` is a connection pool — it maintains multiple database connections and hands them out as needed. This is much better than creating a new connection for every request:

```rust
pub async fn init_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)        // Up to 5 simultaneous queries
        .connect(database_url)
        .await
}
```

With a pool of 5 connections, FicHub can handle 5 database queries simultaneously. If a 6th request comes in, it waits until a connection becomes free. This prevents the database from being overwhelmed while still handling lots of concurrent requests.

### Connecting to Redis

Redis connections are even simpler:

```rust
let redis_client = redis::Client::open(config.redis_url.as_str())
    .expect("Invalid Redis URL");

let redis_conn = redis_client.get_multiplexed_async_connection()
    .await
    .expect("Failed to connect to Redis");
```

A **multiplexed connection** means multiple Redis commands can be in flight at the same time on a single connection. It's like having a two-way radio where you can send and receive messages simultaneously. This is more efficient than having separate connections for each operation.

### Using Redis in a Handler

Here's a handler that reads from the cache:

```rust
use axum::{extract::State, Json};
use redis::AsyncCommands;

async fn get_cached_story(
    State(state): State<Arc<AppState>>,
    Path(url_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Try to get the story from Redis cache
    let cached: Option<String> = state.redis
        .get(&url_id)
        .await
        .map_err(AppError::CacheError)?;

    match cached {
        Some(data) => {
            // Cache hit! Parse and return
            let story: serde_json::Value = serde_json::from_str(&data)?;
            Ok(Json(json!({
                "source": "cache",
                "story": story,
            })))
        }
        None => {
            // Cache miss — would normally scrape and store
            Err(AppError::NotFound("Story not cached".to_string()))
        }
    }
}
```

This is the caching pattern that FicHub uses: check Redis first, and if the story isn't there (a "cache miss"), scrape it from the source site and store it in the cache for next time.

### Putting It All Together

Here's our complete server with shared state, from startup to serving:

```rust
mod config;
mod error;

use std::sync::Arc;
use axum::{routing::get, Router};
use error::AppError;

pub struct AppState {
    pub config: config::Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = config::Config::from_env();

    // Connect to PostgreSQL
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Connect to Redis
    let redis_client = redis::Client::open(config.redis_url.as_str())
        .expect("Invalid Redis URL");
    let redis_conn = redis_client.get_multiplexed_async_connection()
        .await
        .expect("Failed to connect to Redis");

    // Build shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        db: db_pool,
        redis: redis_conn,
    });

    // Build router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/v0/meta", get(meta_handler))
        .with_state(state);

    // Start serving
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");

    println!("Listening on {}", addr);
    axum::serve(listener, app).await.expect("Server error");
}

async fn health_check(
    State(state): State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "node": state.config.node_name,
    }))
}

async fn meta_handler(
    State(state): State<Arc<AppState>>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM stories")
        .fetch_one(&state.db)
        .await?;

    Ok(axum::Json(serde_json::json!({
        "total_stories": row.0,
        "node": state.config.node_name,
    })))
}
```

This is very close to the real FicHub. The main difference is that the real version has many more handlers, more state fields (scraper registry, rate limiter, recommendation engine, etc.), and middleware layers. But the core pattern — `Arc<AppState>` passed via `State` extractor — is exactly the same.

### The Pattern

Here's the pattern to remember:

1. **Define `AppState`** with all the resources your handlers need
2. **Build the state** at startup (connect to databases, create clients)
3. **Wrap in `Arc`** for thread-safe sharing
4. **Pass to router** with `.with_state(state)`
5. **Extract in handlers** with `State(state): State<Arc<AppState>>`

Every web server in Rust follows this pattern. Once you understand it, you can read any Axum codebase and know exactly how the state flows.

🧪 **Try It Yourself**

1. Add a `name: String` field to `AppState` and have the health check endpoint return it.
2. Create a new route `/api/v0/config` that returns the full configuration as JSON (be careful not to expose sensitive values like database URLs in production!).
3. Try creating two handlers that both use `State` — verify that they both receive the same state instance.

### What We've Learned

In this chapter, you've learned:

- **Shared state** — giving all handlers access to the same resources
- **Arc\<T\>** — thread-safe reference counting for sharing data
- **AppState** — grouping all shared resources in one struct
- **State extractor** — Axum's mechanism for injecting state into handlers
- **Database pools** — sharing a pool of connections across requests
- **Redis connections** — multiplexed connections for caching
- **The pattern** — build state at startup, wrap in `Arc`, pass to router, extract in handlers

### The Full FicHub AppState

The real FicHub `AppState` is larger than our example. It includes everything the server needs:

```rust
pub struct AppState {
    pub config: Config,
    pub db: sqlx::PgPool,
    pub redis: redis::aio::MultiplexedConnection,
    pub http_client: reqwest::Client,
    pub scraper_registry: Arc<ScraperRegistry>,
    pub cache_semaphores: CacheSemaphores,
    pub rate_limiter: Box<dyn RateLimiter>,
    pub recommender_engine: RecommendationEngine,
    pub collection_worker: CollectionWorker,
}
```

Each field serves a purpose:

- **config** — all the settings from environment variables
- **db** — the PostgreSQL connection pool
- **redis** — the Redis connection for caching and rate limiting
- **http_client** — a pre-configured HTTP client for scraping stories from the web
- **scraper_registry** — knows which scraper to use for each website (AO3, FF.net, XenForo)
- **cache_semaphores** — prevents two requests from scraping the same story simultaneously
- **rate_limiter** — tracks request rates to avoid overwhelming source sites
- **recommender_engine** — suggests similar stories based on what you've read
- **collection_worker** — a background task that precomputes recommendations

Even though this looks like a lot, the pattern is the same: create everything at startup, wrap it in `Arc`, pass it to the router, and extract it in handlers.

### Middleware: The Invisible Helpers

There's one more piece of the puzzle. In the real FicHub, the router uses **middleware** — code that runs before or after every request:

```rust
Router::new()
    .route("/api/v0/epub", get(epub_handler))
    // ... more routes
    .layer(TraceLayer::new_for_http())   // Logs every request
    .layer(CorsLayer::permissive())      // Allows cross-origin requests
    .with_state(state)
```

The `.layer()` calls add middleware. `TraceLayer` logs every incoming request and response — useful for debugging. `CorsLayer` lets web browsers from other domains access your API — needed for the frontend.

Middleware is like a security guard at the door of a building. Every visitor (request) passes through the guard, who checks their ID (CORS headers), makes a note of their visit (logging), and then lets them through to the handler. You don't have to write this logic in every handler — the middleware handles it once for all routes.

### Looking Back: The Big Picture

Let's zoom out and see what we've built over these four chapters:

**Chapter 4: Hello Axum** — We learned what a web server is and built our first one. A router maps paths to handlers. Handlers return responses. The server runs forever, waiting for requests.

**Chapter 5: Configuration** — We moved hardcoded values into environment variables, grouped them in a Config struct, and learned the `from_env()` pattern. Our server is now configurable without code changes.

**Chapter 6: Error Handling** — We created an `AppError` enum that covers every failure mode, implemented `IntoResponse` so Axum can convert errors to HTTP responses, and learned to use the `?` operator for clean error propagation.

**Chapter 7: Shared State** — We created an `AppState` struct, wrapped it in `Arc` for thread-safe sharing, and learned how handlers access shared resources through the `State` extractor.

Together, these four pieces form the backbone of every Axum web application:

```
Config → AppState → Router → Handlers → Responses (or Errors)
```

This is the architecture of FicHub. Every chapter from here on adds new capabilities — database queries, scraping, EPUB generation, caching — but the foundation we've built is the same structure that powers it all.

We've come a long way from "Hello, world!" Our server can receive requests, look up shared resources, handle errors gracefully, and respond with structured JSON. It's not just a toy anymore — it's the skeleton of a real, production-ready web server.

In the next part, we'll dive into the database — creating tables, writing queries, and using SQLx to talk to PostgreSQL. The real data work begins. Let's keep building!
