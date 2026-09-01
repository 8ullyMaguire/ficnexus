# Part 1 — Welcome to the Project

---

# Chapter 1: What We're Building

## Meet FicHub

Imagine this: you're reading a fanfiction story online — maybe it's an epic *Harry Potter* reimagining, or a *Star Wars* character study that made you cry — and you think, "I want to read this on my phone, on the train, without needing Wi-Fi." So you open a tab, paste the URL, and within seconds you have an ebook file you can send to your Kindle, open on your phone, or save to your computer forever.

That's FicHub.

FicHub is a self-hosted web application that lets you download fanfiction from six major sites and save them as beautifully formatted ebooks. Paste a URL, click a button, get an EPUB. Simple on the surface, powerful underneath.

Why would you want this? Because fanfiction deserves better than being trapped in a browser tab. You might be reading on a phone with a spotty data connection. You might want to read offline on a flight. You might want to keep a personal archive of your favorite stories — because fanfiction sites have been known to go down, take stories offline, or change their formats without warning.

FicHub gives you a copy of the story that's yours forever. The EPUB file works on Kindles, Kobo e-readers, Apple Books, Google Play Books, and dozens of other reading apps. It's formatted with proper chapters, metadata, and (when available) cover art.

But FicHub isn't just about downloading. It's also a discovery engine. It can recommend stories you'll love based on what other fans are reading. It lets people suggest stories to the community, vote on those suggestions, and build a curated library of the best fanfiction out there.

And the coolest part? You can run the whole thing on your own computer. No one else's server. No tracking. No ads. Just you, your stories, and a tiny piece of the internet that belongs to you.

## The Two Halves

FicHub has two main parts, and we're going to build both of them from scratch.

**The Backend (Rust + Axum):** This is the engine room. It handles all the heavy lifting — fetching stories from fanfiction sites, converting them into ebook files, managing the database, running the recommendation engine, and serving up data to anyone who asks. We'll write this in Rust, a language that's fast, safe, and genuinely fun to use once you get the hang of it. We'll use a web framework called Axum, which is like a well-organized kitchen where every ingredient has its place.

**The Frontend (SvelteKit):** This is the face of FicHub — the part you see and interact with. It's a sleek, dark-themed interface with tabs, search bars, and beautiful cards showing story recommendations. We'll build this with SvelteKit, a framework that makes creating web interfaces feel less like programming and more like drawing a picture. SvelteKit handles the routing, the page loading, and all the plumbing so we can focus on making things look and feel great.

## The Full Architecture

Before we write a single line of code, let's zoom out and see how all the pieces fit together. Understanding the architecture now will make every chapter that follows feel like a natural step rather than a mysterious leap.

Here's the big picture:

```
┌─────────────────────────────────────────────────────────────┐
│                      YOUR BROWSER                           │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              SvelteKit Frontend                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐    │   │
│  │  │ Download │ │  Search  │ │  Recommendations  │    │   │
│  │  │   Tab    │ │   Tab    │ │      Tab          │    │   │
│  │  └────┬─────┘ └────┬─────┘ └────────┬─────────┘    │   │
│  │       │              │               │               │   │
│  │       └──────────────┼───────────────┘               │   │
│  │                      │                               │   │
│  │              API Client (fetch)                      │   │
│  └──────────────────────┼───────────────────────────────┘   │
│                         │                                    │
└─────────────────────────┼────────────────────────────────────┘
                          │ HTTP/JSON
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Web Server                           │
│                                                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │  Export  │ │  Search  │ │   Tags   │ │   OPDS   │     │
│  │ Handler  │ │ Handler  │ │ Handler  │ │  Feeds   │     │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘     │
│       │              │            │             │           │
│       ▼              ▼            ▼             ▼           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Core Services Layer                     │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐    │   │
│  │  │ Scraper  │ │ Exporter │ │  Recommendation   │    │   │
│  │  │ Registry │ │  Engine  │ │     Engine        │    │   │
│  │  └──────────┘ └──────────┘ └──────────────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
│                         │                                    │
└─────────────────────────┼────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌──────────┐   ┌──────────┐   ┌──────────┐
    │PostgreSQL│   │  Redis   │   │  Disk    │
    │ Database │   │  Cache   │   │  Cache   │
    └──────────┘   └──────────┘   └──────────┘
```

Let's trace a typical request to understand how these pieces work together.

**Scenario: You paste an AO3 URL and click "Download"**

1. **Browser** → The SvelteKit frontend receives your click, extracts the URL from the input field, and calls `fetchExport(url)` in the API client.

2. **API Client** → A simple `fetch()` call sends a GET request to `/api/v0/export?url=<encoded_url>`. The response is JSON.

3. **Axum Router** → The request hits the Axum server. Axum matches the URL pattern to the `export_handler` function. Before calling it, middleware runs: a `TraceLayer` logs the request, and a `CorsLayer` checks that the request came from an allowed origin.

4. **Export Handler** → This is the 17-step export flow (we'll build it in Part 7). It first checks the database cache — has this exact story been exported before? If yes, serve the cached file. If no, it's time to scrape.

5. **Scraper Registry** → The registry looks at the URL and finds the right scraper. AO3 URLs go to the AO3 scraper. FFN URLs go to the FFNet scraper. Each scraper knows how to fetch chapters and extract metadata from its specific site.

6. **AO3 Scraper** → Sends an HTTP request to AO3 (with a polite user-agent and rate limiting), parses the HTML response, extracts the story title, author, chapters, tags, and chapter content. Returns structured `FicMetadata`.

7. **Export Engine** → Takes the metadata and chapter content and builds an EPUB file. Proper HTML for each chapter, metadata for the title page, table of contents, and cover image (if available). Uses the `epub-builder` crate.

8. **Cache Layer** → The finished EPUB is saved to disk cache and the database gets a record of the export (so next time, we skip steps 5-8).

9. **Response** → The handler returns JSON with the download URL: `{"epub_url": "/cache/epub/abc123.epub"}`.

10. **Browser** → The frontend receives the JSON, updates the UI to show a download button, and redirects the browser to the EPUB URL. The file downloads.

That's the complete flow — from paste to download — in about 10 seconds for a story you've never downloaded before, and under 1 second for a cached one.

## What This Book Will Teach You

By the time you finish this book, you'll have built:

| Component | Technology | What You'll Learn |
|-----------|-----------|-------------------|
| Web server | Axum (Rust) | Routing, middleware, state management |
| Database | PostgreSQL | Schema design, migrations, queries |
| Cache | Redis + Disk | Rate limiting, export caching |
| Web scraping | reqwest + scraper | HTML parsing, multi-site adapters |
| Ebook generation | epub-builder | EPUB3 format, metadata, chapters |
| REST API | JSON endpoints | Error handling, versioning |
| Search | SQL + FTS | Dynamic queries, full-text search |
| Recommendations | Jaccard + voting | Collaborative filtering, scoring |
| Frontend | SvelteKit | SPA routing, API integration |
| Testing | cargo-test + Vitest | Unit, integration, E2E tests |
| Deployment | Docker + systemd | Containerization, remote deploy |

That's 11 major skills from a single project. Each one transfers to other projects. The Rust you learn here works for any backend. The SvelteKit you learn here works for any frontend. The PostgreSQL you learn here works for any database.

## Why Rust and SvelteKit?

### Rust for the Backend

Rust might seem like an unusual choice for a web application. Most web backends are written in Python, Node.js, Go, or Java. But Rust brings three superpowers that matter enormously for a project like FicHub:

**1. Zero-cost abstractions.** Rust's iterators, closures, and pattern matching feel as ergonomic as Python, but compile down to the same code you'd write by hand in C. When FicHub scrapes a story, parses HTML, and builds an EPUB, none of those operations add runtime overhead compared to a hand-tuned C program.

**2. Fearless concurrency.** FicHub needs to handle multiple downloads simultaneously — perhaps ten users requesting stories at the same time. In languages like Python, concurrent I/O requires careful management of threads, callbacks, or async/await. In Rust, the compiler *forces* you to handle shared state correctly. If you try to use a mutable reference from two threads, the code won't compile. That's not a limitation — it's a guarantee that your concurrent code is free of data races.

**3. No garbage collector pauses.** Python and Java have garbage collectors that occasionally pause all threads to reclaim memory. For a web server handling hundreds of requests per second, those pauses mean inconsistent response times. Rust uses a system called "ownership" that manages memory at compile time, so there are no GC pauses at all.

The tradeoff? Rust has a steeper learning curve. The compiler is strict, and you'll spend more time in the first week than you would with Python. But the payoff is a backend that's fast, correct, and ready for production on day one.

We use **Axum** as our web framework. Axum is built by the Tokio team (the same people who built the async runtime) and is designed for production use. It's modular, well-documented, and plays nicely with the Rust ecosystem.

### SvelteKit for the Frontend

SvelteKit is the meta-framework built on top of Svelte — similar to how Next.js is built on React, or Nuxt is built on Vue. But Svelte takes a fundamentally different approach:

**In React/Vue/Angular:** Your component code is JavaScript that runs in the browser. When the state changes, the framework re-executes your component function, diffs the virtual DOM, and updates the real DOM. This happens at runtime, in the user's browser.

**In Svelte:** Your component code is compiled into efficient imperative DOM manipulation. There's no virtual DOM, no diffing, no runtime framework overhead. The compiler turns `<p>{count}</p>` into `p.textContent = count` — direct DOM updates with no intermediary.

The result? Svelte applications are typically smaller, faster, and require less boilerplate than equivalent React or Vue applications. For a frontend like FicHub's — where we need smooth tab transitions, real-time search, and responsive UI — Svelte's directness is a perfect fit.

We use **SvelteKit** (not just plain Svelte) because it provides:
- **File-based routing**: Create a file in `src/routes/`, and it becomes a URL automatically
- **Server-side rendering**: Pages can load data on the server before sending HTML to the browser
- **Build optimization**: Automatic code splitting, tree shaking, and asset optimization
- **TypeScript support**: First-class TypeScript integration for safer code

## What You'll Need

Before we start building, you'll need a few things installed on your computer:

**Required:**
- A text editor (VS Code recommended — it has excellent Rust and Svelte support)
- Git (for version control)
- Node.js 18+ (for the SvelteKit frontend)
- PostgreSQL 14+ (for the database)
- Redis 7+ (for caching and rate limiting)

**Helpful but optional:**
- Docker and Docker Compose (for containerized deployment)
- Calibre (for EPUB conversion — we can use a Docker container instead)
- curl or HTTPie (for testing the API from the command line)
- A terminal emulator you're comfortable with

Don't worry if you don't have all of these yet — Chapter 3 walks you through installing and configuring everything step by step.

## The Journey Ahead

This book is structured as a progressive journey. We start simple — a "hello world" web server — and build up to a complete, tested, deployed application. Each chapter adds one new concept, and we build on everything that came before.

Here's the map:

```
Part 1: Welcome (you are here)
  → Ch 1-4: Project overview, web basics, setup, first programs

Part 2: Hello Route — /health and /api/remote
Part 3: Database Foundations — schema, migrations, sqlx
Part 4: Get a Single Work — fetcher, scraper registry
Part 5: Search Works — query builder, ranking
Part 6: Upload a Work — ingestion pipeline
Part 7: EPUB Export — 17-step export, caching
Part 8: User Accounts — auth, JWT, locale
Part 9: Bookmarks — save, read-later
Part 10: Ratings and Kudos — 5-star, one-click kudos
Part 11: Reviews and Comments — feedback system
Part 12: Reading Lists, Shelves, Collections
Part 13: Follows and Feed — social feed
Part 14: Author Pages and Series
Part 15: Tags, Voting, and Curation
Part 16: Forum — community discussions
Part 17: Ask the Archive — LLM Q&A
Part 18: Bounties and Reputation — XP v3 system
Part 19: Notifications — bell, list, preferences
Part 20: RSS and OPDS Feeds
Part 21: Saved Searches and Alerts
Part 22: Blind Date with a Fic
Part 23: Fandoms and Trending
Part 24: Work Proposals and Fic Suggestions
Part 25: Pseuds and Creatorships
Part 26: Skins and Customization
Part 27: Reading Quests and Stats
Part 28: Reading History
Part 29: Translations and Localization
Part 30: Send to Kindle
Part 31: Roadmap Consensus Engine
Part 32: Ollama Embeddings and Recommender
Part 33: Recipe Builder & v3 Customization
Part 34: Moderation and Admin
Part 35: Authentication Deep Dive
Part 36: Rate Limiter and Redis
Part 37: Proof of Work (anti-bot)
Part 38: Frontend Foundations — SvelteKit
Part 39: Frontend Search UI
Part 40: Work and Fic Pages
Part 41: Dashboard and Profile
Part 42: Forum and Curator Moderation
Part 43: Social Features
Part 44: Collections and Shelves
Part 45: Testing
Part 46: Deployment
Part 47: Roadmap Deep-Dive (what's next)
```

Don't skip ahead. Every chapter assumes you've read the ones before it. The code builds incrementally — by Part 7, you'll have a working export pipeline, but only because you built the database schema in Chapter 1 and the scraper in Chapter 4.

If you get stuck, the book has a few safety nets:
- **🧪 Try It Yourself** boxes with hands-on exercises
- **⚠️ Watch Out** warnings about common mistakes
- **Cross-references** linking related concepts across chapters
- **Code listings** with line numbers for easy reference

Ready? Let's build something amazing.

---

# Chapter 2: How the Web Works

## The Basics: Client and Server

Every web application has two halves: a **client** (usually your web browser) and a **server** (a computer somewhere on the internet that holds the data and runs the code). When you type a URL into your browser and press Enter, here's what actually happens:

1. Your browser looks up the server's IP address (using DNS)
2. It opens a TCP connection to that address
3. It sends an HTTP request: "Hey server, I want the page at `/export?url=...`"
4. The server processes the request and sends back an HTTP response: "Here's the data you asked for"
5. Your browser renders the response (HTML, JSON, images, etc.)

This seems simple, but there's a lot of detail hiding under the surface. Let's unpack the key concepts.

## HTTP: The Language of the Web

**HTTP** (Hypertext Transfer Protocol) is the language that clients and servers use to communicate. It's a text-based protocol — you can actually read the messages with your eyes.

Here's what a real HTTP request looks like:

```http
GET /api/v0/export?url=https://archiveofourown.org/works/12345 HTTP/1.1
Host: localhost:3000
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64)
Accept: application/json
Accept-Language: en-US,en;q=0.9
Connection: keep-alive
```

And here's what the server sends back:

```http
HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: 142
Cache-Control: no-cache

{
  "err": 0,
  "url_id": "ao3-12345",
  "meta": {
    "title": "My Favorite Story",
    "author": "SomeWriter",
    "chapters": 24
  }
}
```

Let's break this down:

**Request line:** `GET /api/v0/export?url=... HTTP/1.1`
- `GET` is the **HTTP method** — it means "I want to retrieve something"
- `/api/v0/export?url=...` is the **path** — which resource on the server you want
- `HTTP/1.1` is the protocol version

**Request headers:** These are key-value pairs that give the server extra information:
- `Host:` — which website you're requesting (important for virtual hosting)
- `User-Agent:` — identifies your browser (servers use this for logging)
- `Accept:` — what format you want the response in

**Response status:** `200 OK` means the request succeeded. Here are the status codes you'll see most often:

| Code | Meaning | When You'll See It |
|------|---------|-------------------|
| 200 | OK | Successful request |
| 301 | Moved Permanently | Redirect to a new URL |
| 304 | Not Modified | Cached response is still valid |
| 400 | Bad Request | Your request was malformed |
| 401 | Unauthorized | You need to log in |
| 403 | Forbidden | You're logged in but not allowed |
| 404 | Not Found | The resource doesn't exist |
| 429 | Too Many Requests | You've been rate limited |
| 500 | Internal Server Error | Something broke on the server |
| 502 | Bad Gateway | The upstream server is down |

## HTTP Methods

HTTP defines several methods (also called "verbs") that describe what you want to do:

- **GET** — Retrieve a resource (read-only, no side effects)
- **POST** — Create a new resource or submit data
- **PUT** — Replace a resource entirely
- **PATCH** — Partially update a resource
- **DELETE** — Remove a resource

FicHub's API primarily uses GET (for fetching metadata, searching, downloading) and POST (for submitting tags, voting). We don't use PUT, PATCH, or DELETE much — fanfiction is mostly read-only data.

## REST: The Architectural Style

Most modern web APIs follow **REST** (Representational State Transfer) principles:

1. **Resources are identified by URLs** — Each thing the API can give you has a unique URL
2. **Standard HTTP methods** — GET to read, POST to create, etc.
3. **Stateless** — Each request contains all the information the server needs; the server doesn't remember previous requests
4. **JSON responses** — Data is sent as JSON (JavaScript Object Notation)

FicHub follows REST:

```
GET  /api/v0/export?url=...     → Export a story
GET  /api/v0/meta?url=...       → Get story metadata
GET  /api/v0/search?q=...       → Search for stories
GET  /api/v0/tags               → List available tags
POST /api/v0/tags/submit        → Submit a new tag
POST /api/v0/tags/vote          → Vote on a tag
```

Notice the pattern: the URL tells you *what* you're getting, and the HTTP method tells you *what to do* with it.

## JSON: Data Serialization

**JSON** (JavaScript Object Notation) is the universal language for sending structured data over HTTP. It looks like this:

```json
{
  "err": 0,
  "url_id": "ao3-12345",
  "meta": {
    "title": "My Favorite Story",
    "author": "SomeWriter",
    "words": 125000,
    "chapters": 24,
    "complete": true,
    "tags": ["harry potter", "alternate universe", "romance"]
  }
}
```

JSON is:
- **Human-readable** — you can look at it and understand the data
- **Language-agnostic** — every programming language can parse it
- **Compact** — no unnecessary markup

In Rust, we use the `serde` crate (Serializer/Deserializer) to convert between Rust structs and JSON. In JavaScript, the `JSON.parse()` and `JSON.stringify()` functions handle it natively.

## How FicHub Uses HTTP

Let's trace the HTTP traffic for a complete user journey:

**Step 1: User opens FicHub**
```
Browser → GET / → Server returns HTML (the SvelteKit app)
Browser → GET /app.css → Server returns the stylesheet
Browser → GET /app.js → Server returns the JavaScript bundle
```

**Step 2: User pastes an AO3 URL and clicks Download**
```
Browser → GET /api/v0/export?url=https://archiveofourown.org/works/12345
Server → Scrapes AO3, generates EPUB, returns JSON:
  {"err": 0, "epub_url": "/cache/epub/abc123.epub"}
```

**Step 3: Browser downloads the EPUB**
```
Browser → GET /cache/epub/abc123.epub
Server → Returns the EPUB file (binary data)
```

**Step 4: User browses recommendations**
```
Browser → GET /api/v0/recommendations/ao3?limit=20
Server → Returns JSON with recommended stories:
  {"recommendations": [...]}
```

**Step 5: User searches for stories**
```
Browser → GET /api/v0/search?q=harry+potter+angst&min_words=50000&sort=score
Server → Queries database, returns JSON:
  {"total": 142, "page": 1, "results": [...]}
```

Each of these is a separate HTTP request-response cycle. The browser doesn't maintain a persistent connection to the server — it asks for something, gets the answer, and then asks for the next thing.

## TCP/IP: What's Underneath HTTP

HTTP runs on top of **TCP/IP** (Transmission Control Protocol / Internet Protocol). You don't need to understand TCP deeply to build web applications, but knowing the basics helps when things go wrong.

**TCP** guarantees that data arrives correctly and in order. When your browser sends an HTTP request, TCP breaks it into small packets, sends them over the network, and reassembles them at the other end. If a packet gets lost, TCP automatically retransmits it.

**IP** handles addressing and routing. Every device on the internet has an IP address (like `192.168.1.100` or `203.0.113.50`). IP routes packets from your computer to the server's computer, even if they're on opposite sides of the planet.

The key takeaway: **HTTP is a request-response protocol on top of TCP/IP**. When you build a web server with Axum, you're writing code that handles HTTP requests and sends HTTP responses. The TCP/IP layer handles the networking for you.

## DNS: The Internet's Phone Book

When you type `localhost:3000` in your browser, it needs to know the IP address. **DNS** (Domain Name System) translates human-readable names to IP addresses.

- `localhost` → `127.0.0.1` (your own computer)
- `fichub.example.com` → `203.0.113.50` (some server on the internet)

For development, you'll mostly use `localhost` (or `127.0.0.1`). When you deploy FicHub to a real server, you'll configure DNS to point your domain name to the server's IP address.

## Ports: Door Numbers on a Computer

A single computer can run many network services. **Ports** are like door numbers — they tell the operating system which program should receive incoming traffic.

- Port 80: HTTP (standard web traffic)
- Port 443: HTTPS (encrypted web traffic)
- Port 3000: FicHub (our default)
- Port 5432: PostgreSQL
- Port 6379: Redis

When you visit `localhost:3000`, your browser connects to port 3000 on your machine. The operating system delivers the data to whatever program is listening on that port — in our case, the Axum server.

## WebSocket: Persistent Connections

HTTP is request-response: the client asks, the server answers, the connection closes. But what if you need real-time communication? **WebSocket** maintains a persistent, bidirectional connection.

FicHub doesn't use WebSocket (we use simple HTTP for everything), but you might want to add it later for features like:
- Real-time progress bars during EPUB generation
- Live notifications when new stories are recommended
- Chat between users

Axum supports WebSocket through the `axum::extract::ws` module. We won't cover it in this book, but it's good to know it exists.

## HTTPS: Encrypted HTTP

Plain HTTP sends data in cleartext — anyone on the network can read it. **HTTPS** (HTTP Secure) encrypts the connection using TLS (Transport Layer Security), so only you and the server can read the data.

For development, HTTP on localhost is fine (there's no network to intercept). For production, you **must** use HTTPS. The typical setup:

1. Get an SSL/TLS certificate (Let's Encrypt provides free ones)
2. Configure nginx or Caddy as a reverse proxy
3. The reverse proxy handles HTTPS and forwards requests to your Axum server on localhost:3000

We'll cover this in detail in Part 46 (Deployment).

## Cookies and Sessions

HTTP is stateless — the server doesn't remember you between requests. But many websites need to track users (for logins, preferences, etc.). **Cookies** solve this: the server sends a small piece of data in the response, and the browser automatically includes it in every subsequent request.

FicHub doesn't use cookies for authentication (it's mostly a public application), but it does use cookies for:
- Remembering your theme preference (dark/light mode)
- Storing your search filters

If you were to add user accounts later, you'd use cookies (or JWT tokens) to track who's logged in.

## Content Types

When the server sends a response, it includes a `Content-Type` header that tells the browser what kind of data it is:

| Content-Type | What It Is | Example |
|-------------|-----------|---------|
| `text/html` | HTML page | The SvelteKit app |
| `application/json` | JSON data | API responses |
| `application/epub+zip` | EPUB ebook | Downloaded stories |
| `text/css` | CSS stylesheet | Styling |
| `application/javascript` | JavaScript | Client-side code |
| `image/png` | PNG image | Icons, covers |

The browser uses the content type to decide how to handle the data. A `text/html` response gets rendered as a web page. A `application/json` response gets passed to JavaScript. A `application/epub+zip` response triggers a file download.

## Caching

Caching is storing frequently-accessed data in a faster location. FicHub uses three levels of caching:

1. **Browser cache** — Your browser remembers files it's downloaded (CSS, JS, images) so it doesn't re-download them on every page load
2. **HTTP cache headers** — FicHub tells the browser how long to cache files (`Cache-Control: max-age=31536000` for immutable assets)
3. **Application cache** — FicHub caches generated EPUBs on disk and in the database, so the same story doesn't need to be re-scraped and re-generated

We'll implement all three levels in later chapters.

## Putting It All Together

Here's the complete picture of what happens when you use FicHub:

```
1. You open fichub.example.com in your browser
2. Browser: DNS lookup → fichub.example.com → 203.0.113.50
3. Browser: TCP connection to 203.0.113.50:443
4. Browser: TLS handshake (encryption setup)
5. Browser: GET / (HTTP/1.1, Host: fichub.example.com)
6. Nginx (reverse proxy): Terminates TLS, forwards to localhost:3000
7. Axum: Serves the SvelteKit HTML shell
8. Browser: Parses HTML, loads CSS and JS
9. Browser: GET /app.js → Axum serves the JavaScript bundle
10. SvelteKit: Runs in browser, mounts the SPA
11. You paste a URL and click Download
12. SvelteKit: fetchExport("https://ao3.org/works/12345")
13. Browser: GET /api/v0/export?url=...
14. Axum: export_handler → checks DB cache → scrapes AO3 → generates EPUB
15. Axum: Returns {"epub_url": "/cache/epub/abc123.epub"}
16. Browser: GET /cache/epub/abc123.epub
17. Browser: File downloads to your Downloads folder
18. You open the EPUB on your Kindle. Done!
```

That's the web in a nutshell. HTTP, TCP/IP, DNS, ports, JSON, caching — they all work together to make a seemingly simple paste-and-click experience happen.

---

# Chapter 3: Setting Up Your Workshop

## Installing Node.js

FicHub's frontend uses SvelteKit, which requires Node.js. Let's install it.

**On macOS (using Homebrew):**

```bash
# Install Homebrew if you don't have it
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Node.js
brew install node
```

**On Ubuntu/Debian:**

```bash
# Install Node.js 20 LTS
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

**On Arch Linux:**

```bash
sudo pacman -S nodejs npm
```

**Verify the installation:**

```bash
node --version
# Should show: v20.x.x or similar

npm --version
# Should show: 10.x.x or similar
```

## Installing Rust

Rust is installed through `rustup`, the official toolchain manager.

**On macOS/Linux:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts (press Enter for the default installation). After installation, restart your terminal or run:

```bash
source $HOME/.cargo/env
```

**Verify:**

```bash
rustc --version
# Should show: rustc 1.7x.0 (or newer)

cargo --version
# Should show: cargo 1.7x.0 (or newer)
```

**Useful extras:**

```bash
# Install clippy (linting) and rustfmt (formatting)
rustup component add clippy rustfmt

# Install cargo-watch for automatic recompilation
cargo install cargo-watch

# Install cargo-edit for cargo add/rm commands
cargo install cargo-edit
```

## Installing PostgreSQL

**On macOS:**

```bash
brew install postgresql@16
brew services start postgresql@16
```

**On Ubuntu/Debian:**

```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**On Arch Linux:**

```bash
sudo pacman -S postgresql
sudo -u postgres initdb -D /var/lib/postgres/data
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

**Create a database for FicHub:**

```bash
# On macOS (no sudo needed for the postgres user)
createuser fichub -P
# Enter a password when prompted (remember it for .env)

createdb fichub -O fichub
```

**On Linux:**

```bash
sudo -u postgres psql
```

Then in the PostgreSQL shell:

```sql
-- Create a user with a password
CREATE USER fichub WITH PASSWORD 'your_password_here';

-- Create the database owned by that user
CREATE DATABASE fichub OWNER fichub;

-- Grant all privileges
GRANT ALL PRIVILEGES ON DATABASE fichub TO fichub;

-- Exit
\q
```

**Find your PostgreSQL config file:**

```bash
# macOS
ls /opt/homebrew/var/postgresql@16/pg_hba.conf

# Linux
sudo find /etc/postgresql -name pg_hba.conf
```

You may need to edit the `pg_hba.conf` file to allow password authentication. Look for lines that say `peer` and change them to `md5` or `scram-sha-256`.

## Installing Redis

**On macOS:**

```bash
brew install redis
brew services start redis
```

**On Ubuntu/Debian:**

```bash
sudo apt-get install redis-server
sudo systemctl start redis
sudo systemctl enable redis
```

**On Arch Linux:**

```bash
sudo pacman -S redis
sudo systemctl start redis
sudo systemctl enable redis
```

**Verify:**

```bash
redis-cli ping
# Should respond: PONG
```

## Installing Docker (Optional)

Docker is optional for development but necessary for the deployment chapters.

**On macOS:** Install Docker Desktop from [docker.com](https://docker.com)

**On Linux:**

```bash
sudo apt-get install docker.io docker-compose
sudo usermod -aG docker $USER
# Log out and back in for the group change to take effect
```

**Verify:**

```bash
docker --version
docker-compose --version
```

## Creating the Project Structure

Let's set up the directory structure for FicHub:

```bash
# Create the main project directory
mkdir fichub
cd fichub

# Create the backend (Rust) project
cargo init --name fichub-rs

# Add our main dependencies to Cargo.toml
# (We'll explain each one later)

# Go back to the main directory
cd ..

# Create the frontend (SvelteKit) project
npm create svelte@latest frontend
# Choose: Skeleton project, TypeScript, ESLint, Prettier

# Create supporting directories
mkdir -p cache tmp migrations
```

## The .env File

FicHub is configured through environment variables. Create a `.env` file in the project root:

```bash
# Database
DATABASE_URL=postgres://fichub:your_password@localhost/fichub

# Redis
REDIS_URL=redis://127.0.0.1/

# Paths
CACHE_DIR=./cache
TMP_DIR=./tmp
FRONTEND_DIR=./frontend/build

# Server
PORT=3000
NODE_NAME=orion

# Feature flags
DYNAMIC_RATE_LIMIT=true
EXPORT_VERSION=1

# Calibre (for EPUB conversion, optional)
CALIBRE_CONTAINER=
```

⚠️ **Watch Out:** Never commit `.env` to git. Add it to `.gitignore`:

```bash
echo ".env" >> .gitignore
echo "cache/" >> .gitignore
echo "tmp/" >> .gitignore
```

## Setting Up Your Editor

**VS Code (recommended):**

Install these extensions:
- `rust-analyzer` — Rust language support (autocompletion, error checking, refactoring)
- `Svelte for VS Code` — Svelte syntax highlighting and IntelliSense
- `Tailwind CSS IntelliSense` — CSS class autocompletion (if using Tailwind)
- `Error Lens` — Shows errors inline in the code
- `Code Spell Checker` — Catches typos in strings and comments

**Useful VS Code settings for Rust:**

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.inlayHints.chainingHints.enable": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Verifying Everything Works

Let's run a quick smoke test to make sure all the pieces are in place:

```bash
# Check Node.js
node --version  # Should show v20+

# Check Rust
rustc --version  # Should show 1.7x+

# Check PostgreSQL
psql -U fichub -d fichub -c "SELECT version();"
# Should show PostgreSQL version info

# Check Redis
redis-cli ping  # Should show PONG

# Check that the project compiles
cd fichub
cargo check  # Should complete (may take a few minutes the first time)
```

If any of these fail, go back to the installation section for that tool and double-check.

🧪 **Try It Yourself:** Run `cargo check` and count how many warnings or errors you get. For a fresh project, you should see zero errors and a few "unused variable" warnings (which is normal — Rust warns about everything).

---

# Chapter 4: Your First Rust and SvelteKit Programs

## Hello, Rust!

Let's write the simplest possible Rust web server. Open `fichub/src/main.rs` and type:

```rust
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, FicHub! 🎉"
}
```

Now add the dependencies to `Cargo.toml`:

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Run it:

```bash
cargo run
```

Open `http://localhost:3000` in your browser. You should see "Hello, FicHub! 🎉"

**What just happened?**
- `#[tokio::main]` starts the Tokio async runtime (we need this for async operations)
- `Router::new().route("/", get(hello))` says "when someone hits `/`, call the `hello` function"
- `TcpListener::bind` starts listening for connections on port 3000
- `axum::serve` runs the server, processing requests as they come in
- `hello()` returns a simple string response

## Adding Dynamic Routes

Let's make it more interesting. Replace the `main.rs` content with:

```rust
use axum::{
    extract::Path,
    extract::Query,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/greet", get(greet))
        .route("/greet/{name}", get(greet_name))
        .route("/counter", get(counter));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> &'static str {
    "Hello, FicHub! Try /greet?name=Alice or /greet/Bob"
}

async fn greet(Query(params): Query<HashMap<String, String>>) -> String {
    let name = params.get("name")
        .map(|s| s.as_str())
        .unwrap_or("stranger");
    format!("Hello, {}! Welcome to FicHub.", name)
}

async fn greet_name(Path(name): Path<String>) -> String {
    format!("Hello, {}! You used a path parameter.", name)
}

static mut COUNTER: u64 = 0;

async fn counter() -> Json<HashMap<String, u64>> {
    unsafe { COUNTER += 1; }
    let mut response = HashMap::new();
    unsafe { response.insert("count".to_string(), COUNTER); }
    Json(response)
}
```

Try these URLs:
- `http://localhost:3000` → "Hello, FicHub! ..."
- `http://localhost:3000/greet?name=Alice` → "Hello, Alice! Welcome to FicHub."
- `http://localhost:3000/greet/Bob` → "Hello, Bob! You used a path parameter."
- `http://localhost:3000/counter` → `{"count": 1}` (increments each time)

**Key concepts:**
- `Path(name)` extracts a value from the URL path
- `Query(params)` extracts query string parameters
- `Json(response)` serializes a Rust value as JSON
- `{name}` in the route string is a path parameter placeholder

⚠️ **Watch Out:** The `static mut COUNTER` approach above is NOT how you'd handle shared state in production. It's unsafe and doesn't work with multiple threads. In Part 5, we'll learn the proper way using `Arc<AtomicU64>`.

## Your First SvelteKit Component

Now let's build something with SvelteKit. Create a new SvelteKit project:

```bash
cd fichub
npm create svelte@latest frontend
# Choose: Skeleton project, TypeScript
cd frontend
npm install
```

Edit `src/routes/+page.svelte`:

```svelte
<script lang="ts">
  let name = $state('FicHub');
  let count = $state(0);
  
  function increment() {
    count += 1;
  }
</script>

<main>
  <h1>Welcome to {name}!</h1>
  <p>You've clicked {count} {count === 1 ? 'time' : 'times'}.</p>
  <button onclick={increment}>
    Click me!
  </button>
</main>

<style>
  main {
    max-width: 600px;
    margin: 2rem auto;
    padding: 2rem;
    font-family: sans-serif;
    text-align: center;
  }
  
  h1 {
    color: #2563eb;
  }
  
  button {
    padding: 0.5rem 1.5rem;
    font-size: 1rem;
    background: #2563eb;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
  }
  
  button:hover {
    background: #1d4ed8;
  }
</style>
```

Run it:

```bash
cd frontend
npm run dev
```

Open `http://localhost:5173`. You'll see a welcome page with a clickable counter.

**What's happening?**
- `let name = $state('FicHub')` creates a reactive state variable (Svelte 5 runes)
- `let count = $state(0)` creates another reactive variable
- `{name}` and `{count}` in the HTML are reactive expressions — when the variable changes, the DOM updates automatically
- `onclick={increment}` calls the function when the button is clicked

The magic of Svelte: when you change `count`, only the text node displaying the count updates. No virtual DOM, no diffing, no framework overhead.

## Connecting Frontend to Backend

Now let's make them talk to each other. We'll set up a proxy so the SvelteKit dev server forwards API requests to our Rust server.

In `svelte.config.js`, add a proxy:

```javascript
import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter()
  }
};

export default config;
```

In `vite.config.ts`:

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': 'http://localhost:3000'
    }
  }
});
```

Now update `src/routes/+page.svelte`:

```svelte
<script lang="ts">
  let name = $state('FicHub');
  let count = $state(0);
  let serverTime = $state('');
  let loading = $state(false);
  
  async function fetchServerTime() {
    loading = true;
    try {
      const res = await fetch('/api/v0/remote');
      const data = await res.json();
      serverTime = JSON.stringify(data, null, 2);
    } catch (err) {
      serverTime = 'Error: Could not reach server';
    } finally {
      loading = false;
    }
  }
  
  function increment() {
    count += 1;
  }
</script>

<main>
  <h1>Welcome to {name}!</h1>
  <p>You've clicked {count} {count === 1 ? 'time' : 'times'}.</p>
  <button onclick={increment}>Click me!</button>
  
  <hr />
  
  <h2>Server Connection</h2>
  <button onclick={fetchServerTime} disabled={loading}>
    {loading ? 'Loading...' : 'Check Server'}
  </button>
  
  {#if serverTime}
    <pre>{serverTime}</pre>
  {/if}
</main>
```

Now when you click "Check Server", it fetches data from the Rust backend through the proxy. The frontend and backend are working together!

## Understanding the SvelteKit File Structure

```
frontend/
├── src/
│   ├── routes/          ← File-based routing
│   │   ├── +page.svelte    ← Homepage (/)
│   │   ├── +layout.svelte  ← Layout wrapper
│   │   └── +layout.ts      ← Layout data loader
│   ├── lib/             ← Shared code
│   │   ├── api/         ← API client code
│   │   ├── components/  ← Reusable components
│   │   └── util.ts      ← Utility functions
│   └── app.d.ts         ← TypeScript declarations
├── static/              ← Static assets (images, fonts)
├── svelte.config.js     ← SvelteKit configuration
├── vite.config.ts       ← Vite bundler configuration
└── package.json         ← Dependencies
```

The magic of SvelteKit routing:
- `src/routes/+page.svelte` → serves `/`
- `src/routes/search/+page.svelte` → serves `/search`
- `src/routes/[...slug]/+page.ts` → serves any URL (catch-all)

Each route can have:
- `+page.svelte` — the page component
- `+page.ts` or `+page.server.ts` — data loading
- `+layout.svelte` — shared layout
- `+error.svelte` — error page

## The TypeScript Types

In `src/lib/api/types.ts`, we define the shapes of data we expect from the API:

```typescript
export interface FicMeta {
  title: string;
  author: string;
  words: number;
  chapters: number;
  complete: boolean;
  tags: string[];
}

export interface ExportResult {
  err: number;
  url_id: string;
  epub_url?: string;
  meta?: FicMeta;
}

export interface SearchResult {
  total: number;
  page: number;
  per_page: number;
  results: FicMeta[];
}
```

TypeScript gives us confidence that our frontend code matches the API's data format. If we try to access `result.foo_bar` and it doesn't exist in the type, TypeScript will catch it at compile time.

## Practice Exercises

🧪 **Try It Yourself:** Before moving to Part 2, try these exercises:

1. **Add a new route** to the Rust server at `/health` that returns `{"status": "ok", "uptime": <seconds>}`. Hint: use `std::time::Instant` to track when the server started.

2. **Add a new page** to the SvelteKit app at `/about` with static content about FicHub.

3. **Create a form** in Svelte that sends a POST request to the Rust server with a JSON body. Hint: use `fetch('/api/endpoint', { method: 'POST', body: JSON.stringify(data), headers: { 'Content-Type': 'application/json' } })`.

4. **Add error handling** to the API client: what happens if the server is down? Display a user-friendly error message.

5. **Write a test** for the Rust server: send a GET request to `/` and verify the response is "Hello, FicHub!". Hint: use the `reqwest` crate.

These exercises reinforce the concepts from this chapter. Don't skip them — they'll make the later chapters much easier.

---

