# Part 1 — Welcome & Hello Server

Welcome! You are about to build FicHub from scratch. FicHub is a fanfiction archive and download server. People come here to read, download, rate, and discuss fanfiction. You will build it one feature at a time — each feature gets a real Rust backend route AND a real SvelteKit frontend page. No "build all the backend first" — every feature ships end-to-end before the next one starts.

This is Part 1 of 12. By the end you will have:

- A Rust Axum server running on port 8000
- A `/health` endpoint that says "OK"
- A `/ping` endpoint you wrote yourself
- A clear picture of the project skeleton

Grab a drink. Open your terminal. Let's go.

---

## 1.1 What is FicHub and why build it?

FicHub is a self-hosted fanfiction archive. Think of it as your own private version of AO3 or FanFiction.net — you scrape fanfiction from many sites, store it, let people search it, rate it, review it, download it as EPUB or MOBI, and talk about it in a forum.

Why build it yourself?

- **Learning**: you touch Rust, Axum, SQLx, PostgreSQL, Redis, SvelteKit, Vite, and EPUB generation — all in one project.
- **Control**: you decide what the archive looks like, what it downloads, and how the community works.
- **Fun**: fanfiction archives are genuinely useful. If you or your friends read fanfiction, a personal archive is a great tool.

FicHub's real stack (what you will build):

- **Backend**: Rust 2024 edition, Axum 0.8 (web framework), SQLx 0.9 (PostgreSQL queries), Redis (rate limiting), Ollama (AI features later), epub-builder (EPUB export), Tera (templates), fanfic-scrapers (downloading fic from other sites).
- **Frontend**: SvelteKit 5, Vite, static adapter (SPA), TypeScript, testing with Vitest.
- **Database**: PostgreSQL (works, authors, ratings, reviews, users, forum posts).
- **Cache / queue**: Redis (rate limiter, background jobs).

You do not need to know all of this yet. You will learn each piece as it appears.

---

## 1.2 Your first `cargo run`: the Axum server skeleton

Open a terminal and go to the project root:

```bash
cd ~/code/rust/fichub
```

Look at what is there. You should see a folder named `src/`. That is where the Rust code lives.

```bash
ls src/
# main.rs  server.rs  routes/  db/  services/  search/  recommender/
```

The entry point is `src/main.rs`. That is the first file Rust runs when you type `cargo run`.

Look at the top of `src/main.rs`:

```rust
// src/main.rs (excerpt)
use std::env;
use anyhow::Result;
use fichub::server::run;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    run().await?;
    Ok(())
}
```

Let's break this down:

- `#[tokio::main]` — this turns the `main` function into an async entry point. Tokio is Rust's async runtime. Axum needs async because it handles many connections at once without blocking.
- `dotenvy::dotenv().ok()` — loads a `.env` file if there is one. `.env` holds things like database URLs. The `.ok()` means "if there is no `.env`, that's fine, don't crash."
- `run().await?` — calls the real server startup function in `src/server.rs`. The `?` means "if this fails, propagate the error and exit."

So `main.rs` is just a thin wrapper. The real server lives in `src/server.rs`. Let's look at it.

Now try running it:

```bash
cargo run
```

The first time, this downloads and compiles every dependency. It can take a few minutes. Later runs are faster because of caching.

**Expected output**: the server starts and prints something like:

```
listening on http://0.0.0.0:8000
```

If you see that, the server is alive.

If it fails with a database error, that is expected — we have not set up PostgreSQL yet. For now, the server tries to connect and may panic. Don't worry; we will fix that in Part 2. For the rest of Part 1 we care about the shape of the code, not whether the database is running.

---

## 1.3 `src/main.rs` and `src/server.rs` line by line

Open `src/server.rs`. This is the heart of the backend. Let's read it section by section.

```rust
// src/server.rs (excerpt)
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use std::net::SocketAddr;

pub async fn run() -> anyhow::Result<()> {
    // 1. Build the router
    let app = Router::new()
        .route("/health", get(health))
        .layer(TraceLayer::new_for_http());

    // 2. Pick the address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    // 3. Start listening
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}
```

### Building the router

```rust
let app = Router::new()
    .route("/health", get(health))
    .layer(TraceLayer::new_for_http());
```

- `Router::new()` — creates an empty router. A router is a map from URL paths to handler functions.
- `.route("/health", get(health))` — attach the path `/health` to the `health` function. The `get` part means "only respond to HTTP GET requests." POST, PUT, DELETE to `/health` get a 404.
- `.layer(TraceLayer::new_for_http())` — a middleware that logs every request. You will see lines like `GET /health 200 OK` in your terminal.

### The address

```rust
let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
```

`[0, 0, 0, 0]` means "listen on all network interfaces." Port 8000. Together: `http://0.0.0.0:8000`.

### Listening and serving

```rust
let listener = tokio::net::TcpListener::bind(addr).await?;
axum::serve(listener, app).await?;
```

- `TcpListener::bind(addr)` — opens a TCP socket on that address. The `await` is needed because binding is an async operation.
- `axum::serve(listener, app)` — starts accepting HTTP connections and handing them to the router. This function runs forever (until you press Ctrl+C).

### The health handler

```rust
async fn health() -> &'static str {
    "OK"
}
```

- This is a handler. Axum calls it when someone GETs `/health`.
- It is async (because Axum handlers are async by default), but it does not actually do anything async here.
- It returns `&'static str` — a string that lives for the whole program lifetime. Axum knows how to turn that into an HTTP 200 response with `Content-Type: text/plain`.

That's the whole server. Small, right? That is the power of Axum — a working HTTP server in ~20 lines.

---

## 1.4 The `/health` endpoint: your first route

Now that you understand the code, let's actually use it.

Start the server (if it is not already running):

```bash
cargo run
```

In another terminal, ask the server if it is alive:

```bash
curl -s http://localhost:8000/health
```

**Expected output**:

```
OK
```

That's it. You just made an HTTP request to your own Rust server and got a response. This is the "Hello World" of web servers, but here it is a real production-style server with logging, address binding, and async — not a toy.

Try the long form with headers:

```bash
curl -v http://localhost:8000/health
```

You should see something like:

```
> GET /health HTTP/1.1
> Host: localhost:8000
>
< HTTP/1.1 200 OK
< content-type: text/plain; charset=utf-8
< content-length: 2
<
OK
```

The `200 OK` status code means "success." The `content-type: text/plain` tells your browser or curl that the body is plain text.

---

## 1.5 Try It Yourself: add a `/ping` route

Now it is your turn. You are going to add a new route to the server.

**Goal**: add a `GET /ping` endpoint that returns `"pong"`.

### Step 1 — Open `src/server.rs`

Find the line where `/health` is registered:

```rust
.route("/health", get(health))
```

### Step 2 — Add the `/ping` route

Right below the health route, add:

```rust
.route("/ping", get(ping))
```

### Step 3 — Add the `ping` handler

Below the `health` function, add:

```rust
async fn ping() -> &'static str {
    "pong"
}
```

### Step 4 — Restart the server

Press Ctrl+C in the terminal running `cargo run`, then start it again:

```bash
cargo run
```

### Step 5 — Test it

```bash
curl -s http://localhost:8000/ping
```

**Expected output**:

```
pong
```

### What you just did

- You added a new route to the Axum router.
- You wrote a handler function that returns a plain-text response.
- You restarted the server and verified the new route works.

That is the entire lifecycle of an Axum endpoint: register it in the router, write the handler, restart, curl.

### Watch Out

- **Forgot to restart the server?** Axum reads the router once at startup. If you change `server.rs` and do not restart, the old code keeps running. Always restart after changing Rust source files.
- **Port already in use?** If you see `address already in use`, another process is on port 8000. Either stop it or change the port in `server.rs` to something else like `8080`.

---

## What you have now

- A Rust project that compiles and runs.
- A `/health` endpoint that returns "OK".
- A `/ping` endpoint you added yourself.
- An understanding of `main.rs`, `server.rs`, router, handlers, and `axum::serve`.

Next: Part 2 — Database & Your First Model. You will set up PostgreSQL, write your first SQL migration, and create your first Rust `FromRow` struct.

---

*End of Part 1. On to [Part 2 — Database & Your First Model](./02-core-setup/02-core-setup.md).*
