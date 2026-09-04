# Building FicHub: The Full-Stack 200K Tutorial

> Every feature, one at a time. Backend route first, frontend page second, then peel back the real implementation. By the end you have touched every module in the repo and built the whole thing yourself.

**Project**: FicHub — self-hosted fanfiction archive + community platform
**Stack**: Rust 2024, Axum 0.8, SQLx 0.9, PostgreSQL, Redis, Ollama, epub-builder, Tera, SvelteKit 5, Vite
**Word target**: 200,000 words (no upper bound — cover every feature)
**Parts**: 49 parts, one per feature cluster
**Mode**: Toy-first, then real. Each feature: skeleton → real implementation → polish.

---

## Structure

The book has 6 Parts:

- **Part 1 — Welcome**: Project overview, architecture, reading guide, what we're building and why
- **Part 2 — Rust Backend**: Axum server, router, AppState, middleware, error handling, the full route table
- **Part 3 — Scraping**: Extracting fic metadata from external archives, scraper registry, site adapters
- **Part 4 — Export and Cache**: EPUB, PDF, MOBI, AZW3, FB2, TXT, HTML, DOCX, KEPUB generation; disk cache; rate limiting; download URLs
- **Part 5 — Recommendations**: Co-occurrence, TF-IDF, embeddings via Ollama, tag graphs, strategy registry, hybrid scoring
- **Part 6 — Production**: Systemd, nginx, monitoring, testing, deployment, ops

Each part within a Part follows the same pattern:

1. **Goal** — what we are building and why
2. **Real code** — the actual source files, with line numbers and excerpts
3. **Breakdown** — what the code does, piece by piece
4. **Try It Yourself** — exercises to make the feature your own
5. **Troubleshooting** — common mistakes and how to fix them

---

## Parts List

### Part 1 — Welcome
- 1.1 Project overview: what FicHub is, who it's for, what problem it solves
- 1.2 Architecture: the big picture (Rust backend, SvelteKit frontend, PostgreSQL, Redis, Ollama)
- 1.3 Reading guide: how to use this book, prerequisites, setup checklist
- 1.4 The journey ahead: map of all 49 parts and what each covers

### Part 2 — Rust Backend
- 2.1 The empty project: `cargo init`, `Cargo.toml`, what each dependency is for
- 2.2 `src/main.rs`: Tokio runtime, dotenv, the `run()` call
- 2.3 `src/server.rs`: `AppState` — every piece of shared state, why it exists
- 2.4 `axum::serve`: binding port 8000, the fallback static file layer, CORS, TraceLayer
- 2.5 The route table: every route in the system, organized by domain

### Part 3 — Scraping
- 3.1 Scraper registry: how FicHub knows which sites to scrape
- 3.2 HTML parsing: extracting title, author, chapters, word count from fic pages
- 3.3 Multi-site adapters: handling different site layouts
- 3.4 FicMetadata: the canonical struct that represents scraped data
- 3.5 Chapter extraction: getting the actual story content

### Part 4 — Export and Cache
- 4.1 EPUB generation: `epub-builder` crate, chapters, cover, Tera templates
- 4.2 Format conversion: MOBI, PDF, AZW3 via Calibre
- 4.3 Disk cache: the cache directory structure, hash-based lookup
- 4.4 Cache download routes: `/cache/{etype}/{url_id}` serving
- 4.5 Rate limiting: token bucket, concurrent export limits
- 4.6 Export flow: from scrape to download URL

### Part 5 — Recommendations
- 5.1 Co-occurrence: "users who read X also read Y"
- 5.2 TF-IDF: tag frequency scoring
- 5.3 Ollama embeddings: vector similarity via local LLM
- 5.4 Tag graphs: relationship scoring between tags
- 5.5 Strategy registry: cooccur, decay, embeddings, MF, hybrid
- 5.6 Recommendations routes: `/api/recommendations`, `/api/recommendations/personal`

### Part 6 — Production
- 6.1 Systemd service: `fichub.service`, user management, logging
- 6.2 Nginx: reverse proxy, static file serving
- 6.3 Monitoring: health checks, metrics, logging
- 6.4 Testing: Rust integration tests, TypeScript unit tests
- 6.5 Deployment: build, transfer, restart, verify

---

## How to Use This Book

Read the parts in order. Each part is self-contained but builds on earlier ones. If you're already familiar with a topic, skim the Goal and Real Code sections, then jump to Try It Yourself.

When you see a code block with line numbers, that's the actual source from the FicHub repository. The line numbers reference the real files — you can open them alongside the book and follow along.

**Try It Yourself** sections are hands-on. Do them. They're how you make the feature yours.

**Troubleshooting** sections cover the mistakes we made while building FicHub. Read them before you get stuck — they'll save you hours.

---

## Credits

This book documents the actual FicHub codebase. Every code excerpt is verbatim from the repository.

Written for junior developers who want to understand a real production full-stack application from the ground up.

---

*End of book metadata.*
