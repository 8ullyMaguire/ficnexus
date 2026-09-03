# Lemmy — rust@programming.dev announcement (all tools)

Target: https://programming.dev/c/rust — single post covering FicHub + bot + CLI + core crates.

---

**Title:** I built a self-hosted fanfiction archive in Rust — and the bot core behind it is a zero-SDK platform-neutral library (Discord/Matrix/Telegram/Slack/IRC/fediverse/CLI)

**Body:**

I run a self-hosted fanfiction archive, [FicHub](https://fichub.polarisocial.xyz/), built entirely in Rust: an Axum/PostgreSQL server that scrapes and saves stories from 107+ sites (AO3, FanFiction.net, RoyalRoad, forums…) and exports them as EPUB/MOBI/PDF/TXT. (Note: the public fichub.net is a separate legacy Python codebase with a different API — the Rust rewrite is what everything below targets.)

**The server** ([source](https://opencommit.eu/MagicZhang/fichub))

- 107+ scraper adapters, full FanFicFare parity, site cache so fics survive the source disappearing
- Boolean + fielded search, main-character-attribute search ("dark Harry Potter starring Harry"), search inside fic bodies, natural-language "Ask the Archive"
- Bookmarking, 5-star ratings, reviews, threaded comments, reading lists, update feeds
- Personalized recommendations: co-occurrence, decay, embeddings, matrix factorization, hybrid, author/tag-graph, Markov, clusters, bandit — pluggable strategies
- A community roadmap with consensus voting (semantic clustering + Elo-style ranking)
- Self-healing scrape pipeline: structural failures get diagnosed by an on-the-fly LLM agent, validated, and used to complete the export; transient failures go into a pending-request queue that replays on success

**The bot workspace** ([source](https://opencommit.eu/MagicZhang/fanfic-archivist-bot))

The interesting engineering bit: instead of writing N bots, I wrote one platform-neutral core, [`archivist-core`](https://opencommit.eu/MagicZhang/archivist-core), and made every adapter a thin translation layer:

```
Discord ─┐
Matrix ──┤
Telegram─┼──► archivist-core ──► FicHub REST API
Slack ───┤        │                  + Redis (shared)
IRC ─────┤   do_* → PlatformMessage
Fediverse┘
CLI/TUI ─┘
```

- `archivist-core` has **zero platform SDKs** — no discord-rs, no teloxide. Every command is a `do_*` function returning a platform-neutral `PlatformMessage` IR (`Text`/`Rich`/`File`/`Ephemeral`).
- New commands land once in the core; every platform gets them for free. Adapters only translate platform events in and render the IR back out.
- Shared Redis for pagination cache, token store, cross-platform rate limiting.

**The CLI** ([source](https://opencommit.eu/MagicZhang/fanfic-archivist-cli)) — `fic-archivist`

- `search`, `quote`, `download` (batch: `--infile`, `--format epub,mobi,pdf,html`, `--out-dir`, `--force`), `body-search`, `bookmarks`, `recs`, `ask`, `link`/`updates`/`status`, forum subcommands
- Colored TTY or JSON output; a [ratatui](https://github.com/ratatui/ratatui) three-pane TUI (async worker over `tokio::sync::mpsc` — state machine testable without a terminal); rustyline REPL with intent classification

```sh
cargo install fic-archivist --version 0.2.0
```

**Testing** — 256 tests across 8 crates, all offline (no Redis/FicHub/Ollama needed to run the suite).

**Published crates** — `fic-archivist`, `fanfic-archivist` (Discord), `archivist-core`, `forum-core`, `fanfic-scrapers` on crates.io. Source on [Forgejo](https://opencommit.eu/MagicZhang/), AGPL-3.0-or-later.

The core + thin-adapters split is the piece I think is reusable beyond fanfiction — if you're building a multi-platform bot, it's worth stealing. Happy to answer questions about the adapter pattern, the scraper architecture, or the self-healing agent pipeline.
