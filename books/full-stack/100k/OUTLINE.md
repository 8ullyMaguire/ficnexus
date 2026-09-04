# Building FicHub — Full-Stack Code-Along Book (100k words)

**Audience**: junior developers who want to FEEL like they built a real,
production-grade fanfiction platform themselves by following along with the
actual source code. Style: friendly, hands-on, "we're building this
together". Every part contains real code from the repo, 🧪 Try It Yourself,
⚠️ Watch Out, and 💡 Key Concept call-outs.

**Project**: FicHub — Rust/Axum backend + SvelteKit 5 SPA, PostgreSQL 16 +
pgvector, Redis, Ollama. Repo: `/personal/documents/code/rust/fichub`.

**Target**: 100,000 words. 13 parts, ~55 chapters, ~7,700 words per part.

**Status**: ✅ COMPLETE 2026-08-12 — all 13 parts written, assembled at
**213,219 words** (63 chapters) → `FicHub_FULL_STACK_100K.md` +
`FicHub_FULL_STACK_100K.epub`. Parts were written by sequential sub-agents
with SUMMARY/LAST_PASSAGE handoff (see parts/).

---

## PART 1 — Welcome to Your Fanfiction Platform (Part 1 of 13)
- Ch 1: What we're building — the big picture (read README.md, docs/src/what-is-fichub.md)
- Ch 2: Your toolkit — Rust, Axum, SvelteKit, PostgreSQL, Redis, Ollama (Cargo.toml, package.json)
- Ch 3: The repo tour — src/, frontend/src/, migrations/, tests/ (map every directory)
- Ch 4: Running it for the first time — .env, cargo run, npm run dev (config.rs)

## PART 2 — Laying the Rust Foundation
- Ch 5: The entry point — main.rs, lib.rs, how the server boots
- Ch 6: AppState — the dependency-injection heart (server.rs AppState)
- Ch 7: The router — every route registered (build_router in server.rs)
- Ch 8: Errors that don't crash — AppError + error.rs

## PART 3 — Configuration & the Database Layer
- Ch 9: Config from env — config.rs (every knob)
- Ch 10: PostgreSQL + SQLx — db/mod.rs, models.rs
- Ch 11: Migrations 1-34 — the schema story (migrations/*.sql)
- Ch 12: queries.rs — the giant query module (get_fic_info, upsert, search)
- Ch 13: Redis — rate limiter, shadowban, caching (limiter/, services/)

## PART 4 — The Scraper Subsystem
- Ch 14: How FicHub talks to other sites — scrape/mod.rs ScrapeError, ScraperRegistry
- Ch 15: AO3 scraper — ao3.rs (parse metadata, chapters)
- Ch 16: FanFicFare catch-all + FFN + RoyalRoad + XenForo (sites/)
- Ch 17: find_specific_or_fff — preferring native scrapers
- Ch 18: The body cache — every scraped fic saved to disk (body_cache.rs)

## PART 5 — Exports: From URL to EPUB
- Ch 19: The export pipeline — routes/export.rs (semaphore, double-check cache)
- Ch 20: The EPUB builder — export/ (pure Rust epub-builder)
- Ch 21: HTML/TXT/MD + Calibre sidecar for MOBI/PDF/AZW3
- Ch 22: Cache downloads + hashes — cache_download.rs, cache/disk.rs

## PART 6 — The API: Meta, Search, and Reader
- Ch 23: Metadata lookup — routes/meta.rs
- Ch 24: The search engine — search/parser.rs (boolean AND/OR/NOT, phrases, fields)
- Ch 25: main_char_attr — the "Dark Harry" semantics
- Ch 26: The web reader — routes/reader.rs + frontend /read/[urlId]
- Ch 27: The SPA frontend — +layout.svelte, +page.svelte, api client (client.ts)

## PART 7 — Authentication, Users & Social
- Ch 28: Auth — JWT, AuthUser extractor, create_token/verify_token (auth.rs)
- Ch 29: Users, roles, reputation — role tiers 0/1/5/10, admin/users
- Ch 30: Bookmarks, ratings, reviews — social.rs, work_ratings, reviews
- Ch 31: Comments & moderation — comments.rs, hide/delete, modlog
- Ch 32: Follows, updates feed, notifications — follows.rs, notifications.rs

## PART 8 — Community Features
- Ch 33: Fic Requests board — requests.rs (create, answer, vote, accept, candidates)
- Ch 34: Reading lists & shelves — lists.rs, shelves.rs
- Ch 35: Series & authors pages — series.rs, authors.rs
- Ch 36: RSS/Atom feeds — feed.rs, rss/
- Ch 37: Roadmap consensus — roadmap.rs, feature_clusters, Elo arena

## PART 9 — The Recommendation Platform
- Ch 38: The strategy registry — recommender/registry.rs (RecStrategy trait)
- Ch 39: The legacy co-occurrence engine — legacy_cooccur.rs + golden test
- Ch 40: More strategies — decay, embeddings, mf, hybrid, tag_graph, bandit, clusters
- Ch 41: rec_impressions + shadow mode — how to A/B strategies safely
- Ch 42: Personal recommendations — /api/recommendations/personal (bookmarks + downloads)

## PART 10 — Ask the Archive & AI Features
- Ch 43: Natural-language search — /api/search/ask (Ollama → filters)
- Ch 44: The auto-tagger — auto_tag.rs (ML tag suggestions, draft → approve)
- Ch 45: Translations — locales.rs (ML translation, post-edit workflow)
- Ch 46: Self-healing scraping — heal/ (classifier, snapshots, agent loop, migrations 29-30)

## PART 11 — Admin, Analytics & Transparency
- Ch 47: Admin endpoints — admin.rs (users, bans, stats, bots)
- Ch 48: Anti-bot defense — honeypot, rate limits, shadowban, PoW (pow.rs)
- Ch 49: Usage analytics — usage_events, /admin/analytics (views vs actions)
- Ch 50: The modlog — migration 34, /modlog page, every action recorded
- Ch 51: Curator content fixes — peer-voted body fixes (curator_content.rs)

## PART 12 — The Frontend Deep Dive
- Ch 52: SvelteKit 5 fundamentals in this repo — +layout, stores, i18n
- Ch 53: i18n — 6 language dictionaries, t() function
- Ch 54: PWA offline — service worker, precache, cache strategies
- Ch 55: Admin UI — the admin pages (stats, analytics, modlog, scrapers, users)
- Ch 56: Styling + component patterns — the .svelte files, page.test.ts pattern

## PART 13 — Production, Testing & Ship It
- Ch 57: Testing strategy — unit, DB-gated integration, frontend vitest
- Ch 58: The DB-gated test harness — Mutex, #[ignore], unique seeds
- Ch 59: Deployment — systemd, ThinkCentre, migrations on boot, the body-cache drive
- Ch 60: Git worktrees + the NFS quirk — the developer workflow
- Ch 61: The roadmap — what's next (docs/ROADMAP.md)
- Ch 62: Congratulations — you built FicHub!

---

## Writing conventions (for sub-agents)
- Write in Markdown, junior-friendly but not childish. "We" voice.
- Quote REAL code from the repo — read the actual file, paste actual
  snippets (trimmed to the relevant lines), explain line by line.
- Every chapter: at least one 🧪 Try It Yourself, one ⚠️ Watch Out, one
  💡 Key Concept.
- End each part file with a natural "continue reading" beat.
- At the very end of your response, return:
  SUMMARY: 2-3 sentences covering what you taught.
  LAST_PASSAGE: the last 1-2 sentences of your part (verbatim) so the next
  agent can continue seamlessly.
- Write the file to books/full-stack/100k/parts/NN-slug/NN-slug.md
