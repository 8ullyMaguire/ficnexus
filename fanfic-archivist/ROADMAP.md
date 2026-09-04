# fanfic-archivist — Crate Roadmap

Multi-platform companion bot for FicHub. Thin client over the FicHub REST API
(`fichub-api-client` internal crate); shares FicHub's Redis for pagination.
Business logic lives in `archivist-core` as platform-neutral `do_*` functions
returning `PlatformMessage`. Each adapter parses platform-specific input, calls
the matching `do_*`, and renders the IR natively.

## Architecture (2026-09-03)

The bot has been re-architected from a Discord-only bot into a multi-platform
system with 9 adapters:

| Adapter | Crate | Platform | Status |
|---------|-------|----------|--------|
| Discord | `fanfic-archivist-discord` | Discord (poise/serenity) | ✅ full parity |
| Telegram | `fanfic-archivist-telegram` | Telegram (teloxide) | ✅ most commands |
| IRC | `fanfic-archivist-irc` | IRC (irc crate) | ✅ core commands |
| Matrix | `fanfic-archivist-matrix` | Matrix (matrix-sdk) | ✅ core commands |
| Slack | `fanfic-archivist-slack` | Slack (slack-rs) | ✅ most commands |
| Fediverse | `fanfic-archivist-fediverse` | Mastodon + Bluesky | ✅ core commands |
| CLI/REPL | `fanfic-archivist-cli` | Terminal (ratatui) | ✅ most commands |
| Webhook | `fanfic-archivist-webhook` | HTTP (axum) | ✅ events only |
| TUI | `fanfic-archivist-tui` | Terminal UI (ratatui) | ✅ most commands |

All adapters live in `~/code/rust/fanfic-archivist-bot/crates/`.

## Command Coverage Matrix

| Command | Discord | Telegram | IRC | Matrix | Slack | Fediverse | CLI | Webhook |
|---------|---------|----------|-----|--------|-------|-----------|-----|---------|
| recs/fresh/gems/roll | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| search/ask/quote/body | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| download/download_direct | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| bookmark/rate/kudos | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| block/unblock | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| updates | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| exclude | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| link/unlink/whoami/linkcode | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | — |
| request/roadmap | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| fandoms/fandom | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| work | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| daily | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| suggest | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| notify | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| curator | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| vote | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| authors | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| stats | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| filters/reset-filters | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| track | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| sus | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| link_token | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| opds | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| status_presence | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| full-favourites | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| cutoff/wordcount/year | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| blind/reveal | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| trending | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| comments | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| also | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| notifications | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| follow | ✅ | ✅ | ❌ | ✅ | ❌ | ❌ | ✅ | — |
| forum | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ | — |
| help/status/commands | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| webhook events | — | — | — | — | — | — | — | ✅ |
| work-proposals | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| bounties | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| favorite-tags | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| trust-level | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| extensions | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| gdpr-export | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |
| work-translations | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | — |

## New FicHub Features (migrations 008-074)

| Migration | Feature | API Endpoint | Bot Command | Status |
|-----------|---------|-------------|-------------|--------|
| 008 | Work deletion proposals | `/api/work-proposals`, `/{id}/vote` | `/work-delete` | ✅ core done |
| 010 | Reputation bounties | `/api/bounties/*` | `/bounty` | ✅ core done |
| 012 | User favorite tags | `/api/users/me/favorite-tags` | `/favorite-tags` | ✅ core done |
| 013 | Trust levels | `/api/users/me/trust` | `/trust` | ✅ core done |
| 014 | Extensions marketplace | `/api/extensions/*` | `/extensions` | ✅ core done |
| 064 | Roadmap kanban | `/api/roadmap/kanban` | `/roadmap-kanban` | ❌ |
| 067 | GDPR compliance | `/api/me/gdpr-export` | `/gdpr-export` | ✅ core done |
| 070 | Auto-moderation | `/api/admin/moderation` | `/mod` | ❌ |
| 072 | ActivityPub | `/api/federation/*` | `/federation` | ❌ |
| 073 | Forum parity | `/api/forum/*` | `/forum` | ✅ partial |
| 074 | Translation | `/api/works/{id}/translations` | `/translate` | ✅ core done |

## Shipped (v0.1.0 — scaffold)

- Crate skeleton: `fichub-api-client` (typed reqwest wrapper) + `fanfic-archivist`
  binary (poise/serenity), edition 2024, AGPL-3.0.
- Typed API models matching `frontend/src/lib/api/types.ts`; endpoints for
  export, convert, meta, recommendations, search/ask/body, feed, bookmarks,
  ratings, kudos, blocks, requests, roadmap consensus, auth login/me.
- Redis pagination cache (`PageCache`): session-scoped lists for
  `!next`/`!prev`/`!page X`; per-user "latest session" pointer.
- Token store (`TokenStore`): Discord id → FicHub JWT in Redis, TTL 30d,
  refreshed on use.
- `/link` password flow (exchanges creds once, stores only JWT), `/unlink`,
  `/whoami`, `/link-code` (one-time code for web-side confirmation).
- Socrates-parity text commands: `!recs` `!fresh` `!gems` `!roll` `!page`
  `!next` `!prev` `!complete` `!dead` `!words` `!fandom` `!xfandom` `!xfic`
  `!liked` + slash equivalents.
- FicHub-exclusive commands: `/search` `/ask` `/quote` `/body` `/download`
  `/bookmark` `/rate` `/updates` `/block` `/unblock` `/kudos` `/request`
  `/roadmap` `/status`.
- Auto-URL detection (ephemeral metadata embed + Bookmark/Download buttons),
  message context menu "Get Fic Metadata", pagination component buttons.

## Shipped (v0.2.0 — multi-adapter re-architecture)

- `archivist-core` crate: all business logic as `do_*` functions returning
  `PlatformMessage`. Poise command wrappers (Discord) and the intent dispatcher
  (free-form mentions) call the SAME functions.
- 9 adapter crates: Discord, Telegram, IRC, Matrix, Slack, Fediverse
  (Mastodon + Bluesky), CLI/REPL, Webhook, TUI.
- Level-gated access: gate bot commands by FicHub user `level` (site-wide
  0–100, F7) and/or Discord role.
- RAM-aware LLM gating: `GET /api/resource` polled before LLM classification;
  degrades to heuristic classifier when free RAM below threshold.
- Prometheus metrics: `/metrics` on `9090` via `std::sync::atomic` counters.
- Follow exclusions: `!exclude` wired across all 9 adapters.
- Forum parity: `!forum` commands (categories, topics, create, reply, follow,
  mark-read, search) wired across adapters.
- New FicHub features (migrations 008-074): work deletion proposals, reputation
  bounties, user favorite tags, trust levels, extensions marketplace, GDPR
  export, work translations — all implemented in `archivist-core` dispatch.
- Matrix adapter parity (2026-09-04): all missing commands wired — daily, suggest,
  curator, vote, authors, stats, filters, reset-filters, track, sus, link_token,
  opds, status_presence, exclude, full-favourites, cutoff, wordcount, year,
  blind/reveal, download_direct, block, unblock, updates, link/unlink/whoami,
  linkcode, request, roadmap, commands, status, help, quote, body, rate, also,
  trending, comments, notifications, follow. 22 parse tests passing.

## Backlog

### P1 — Adapter parity (bring all 9 adapters to Discord parity)
- [ ] **Telegram parity** — add missing commands: block, unblock, updates,
      exclude, request, roadmap, daily, suggest, notify, curator, vote,
      authors, stats, filters, track, sus, link_token, opds, status_presence,
      full-favourites, cutoff, wordcount, year, blind/reveal, trending,
      comments, also, notifications, follow, forum, help, work-proposals,
      bounties, favorite-tags, trust-level, extensions, gdpr-export,
      work-translations.
- [ ] **IRC parity** — add missing commands: block, unblock, updates, exclude,
      request, roadmap, daily, suggest, notify, curator, vote, authors,
      stats, filters, track, sus, link_token, opds, status_presence,
      full-favourites, cutoff, wordcount, year, blind/reveal, trending,
      comments, also, notifications, follow, forum, work-proposals, bounties,
      favorite-tags, trust-level, extensions, gdpr-export, work-translations.
- [x] **Matrix parity** — add missing commands: daily, suggest, curator, vote,
      authors, stats, filters, reset-filters, track, sus, link_token, opds,
      status_presence, exclude, full-favourites, cutoff, wordcount, year,
      blind_date_reveal, download_direct, block, unblock, updates, link,
      unlink, whoami, linkcode, request, roadmap, commands, status, help,
      quote, body, rate, also, trending, comments, notifications, follow.
      ✅ COMPLETE (2026-09-04): all variants + parse + dispatch wired, 22 tests
      passing.
- [ ] **Slack parity** — add missing commands: block, unblock, updates, exclude,
      request, roadmap, daily, suggest, notify, curator, vote, authors,
      stats, filters, track, sus, link_token, opds, status_presence,
      full-favourites, cutoff, wordcount, year, blind/reveal, trending,
      comments, also, notifications, follow, forum, help, work-proposals,
      bounties, favorite-tags, trust-level, extensions, gdpr-export,
      work-translations.
- [ ] **Fediverse parity** — add missing commands: block, unblock, updates,
      exclude, request, roadmap, daily, suggest, notify, curator, vote,
      authors, stats, filters, track, sus, link_token, opds, status_presence,
      full-favourites, cutoff, wordcount, year, blind/reveal, trending,
      comments, also, notifications, follow, forum, work-proposals, bounties,
      favorite-tags, trust-level, extensions, gdpr-export, work-translations.
- [ ] **CLI parity** — add missing commands: work-proposals, bounties,
      favorite-tags, trust-level, extensions, gdpr-export, work-translations.
- [ ] **TUI parity** — add missing commands: work-proposals, bounties,
      favorite-tags, trust-level, extensions, gdpr-export, work-translations.

### P1 — New FicHub features (migrations 008-074) — adapter wiring
- [ ] **Work deletion proposals** — wire `/work-delete` to Telegram, IRC,
      Matrix, Slack, Fediverse, CLI, TUI adapters.
- [ ] **Reputation bounties** — wire `/bounty` to Telegram, IRC, Matrix,
      Slack, Fediverse, CLI, TUI adapters.
- [ ] **User favorite tags** — wire `/favorite-tags` to Telegram, IRC,
      Matrix, Slack, Fediverse, CLI, TUI adapters.
- [ ] **Trust levels** — wire `/trust` to Telegram, IRC, Matrix, Slack,
      Fediverse, CLI, TUI adapters.
- [ ] **Extensions marketplace** — wire `/extensions` to Telegram, IRC,
      Matrix, Slack, Fediverse, CLI, TUI adapters.
- [ ] **GDPR export** — wire `/gdpr-export` to Telegram, IRC, Matrix, Slack,
      Fediverse, CLI, TUI adapters.
- [ ] **Work translations** — wire `/translate` to Telegram, IRC, Matrix,
      Slack, Fediverse, CLI, TUI adapters.

### P1 — RAM/LLM concurrency
- [x] **RAM-aware LLM gating**: `GET /api/resource` (JSON mem/disk/uptime — the
  canonical endpoint, not `/api/health/resource` which is HTML) polled by bot
  `ram_gated()` before LLM classification; degrades to heuristic classifier
  when `mem.avail_mb` < `FANFIC_ARCHIVIST_RAM_GATE_THRESHOLD_MB` (0 = disabled).
  Verified live: with threshold=999999 MB the bot logged
  `skipping LLM classification — free RAM below 999999 MB threshold`
  and classified `random fic about space travel` → `roll` via heuristic.
  Shipped in bot v0.2.1 (canonical: `fanfic-archivist/ROADMAP.md`).

### P1 — Flipper parity (from Zeks/flipper audit 2026-08-15)
- [ ] **Tracked-message state machines** — persist a state machine *per
      message* (like Zeks/flipper `src/discord/tracked-messages/`:
      `tracked_recommendation_list`, `tracked_roll`, `tracked_review`,
      `tracked_fic_details`, `tracked_similarity_list`, `tracked_help_page`,
      `tracked_delete_confirmation`). Commands mutate that message in place:
      re-roll the same rec list, next/prev by message reference, delete
      confirmation. More capable than current ephemeral button rows.
      - Store state in Redis under `archivist:msg:<channel>:<message>`.
      - Wire into `component_handler` for custom-id dispatch.
- [ ] **`!sus` — suspicious-author detection** — flag authors whose fav-list
      looks gamed: anomalous overlap, ratio outliers, suspicious vote
      patterns. Reuse FicHub rarity-tier weighting (roadmap P8#98) + report.
- [x] **`!stats` — per-user profile stats** — wordcount/size distribution,
      fandom diversity, crossover ratio, popularity band,
      "explorer" tendency (FicHub per-author stats, roadmap P8#104).
- [ ] **`!filters` / `!reset-filters`** — filter the rec stream by liked
      authors, forced params, or reset (Zeks/flipper
      `FilterLikedAuthors`/`ForceListParams`/`ResetFilters`).
- [x] **`!fresh` recency-gated** — `do_recs(... mode:"decay")` shipped (uses
      `updated_at` decay; live on all 7 adapters + CLI).
      `!full-favourites` (whole list browse) still TODO.
- [ ] **Range tuning `!cutoff` / `!wordcount` / `!year`** — tweak rec
      thresholds from Discord.
- [x] **`/authors` — author bibliography** — when FicHub ships
      author-recs (roadmap P8#97): "authors like X" or "authors who wrote
      fic you kudos'd". Bot-side `/authors <name>` command exists (shows
      author's works via `GET /api/authors/by-name/{name}`).
- [x] **`/mood` — fic-level mood filter** — once FicHub ships
      fic-level moods (roadmap P8#96): rec me something dramatic / funny /
      fluffy; add mood facet to `/recs`. Removed from backlog — no server
      endpoint, low priority.

### P2 — UX + parity
- [x] Filter-chip select menus: after a list is shown, offer a
      `SelectMenu` of filters (completed/dead, word-count buckets, fandom)
      that apply server-side filters and re-store the session.
- [x] `/download` direct Discord upload when < 25 MiB (EPUB from FicHub
      cache); fallback to link.
- [x] `/opds` — OPDS link + PWA offline reader deep-link to FicHub web.
- [ ] `/notify` — subscribe to fic-update webhooks (needs FicHub
      notifications API).
- [x] `/curator` — leaderboard of curators (top contributors).
- [x] `/suggest` — suggest a fic as a recommendation (POST
      `/api/recommendations/suggest`).
- [x] `/vote` — roadmap arena voting from Discord (POST
      `/api/roadmap/vote`).
- [ ] Nickname/status integration: show linked username in presence.
- [ ] i18n for bot strings (once FicHub P8#91 key system lands).

### P2 — FicHub features shipped 2026-08-25 (batch: follow-excl / saved-search / search-v2 / collection-sub)
These ship on FicHub `main` but the bot API client predates them. Wire the
thinnest covers first (P1-worthy: follow exclusions + saved-search alerts);
the richer ones (search v2 operators, curated collection voting) are P2+.
Status: follow-exclusions now wired into the bot (see `!exclude` above);
saved-search alerts and collection-vote/curator-approvals still pending bot
client methods.
- [x] **`!exclude` / exclusions per follow** — client methods
      (`add_follow_exclusion` / `list_follow_exclusions` /
      `delete_follow_exclusion`) + `Intent::Exclude` + dispatch arms across
      all 9 adapter crates (Discord /exclude slash command, Telegram, IRC,
      Matrix, Slack, Fediverse Masto+Bsky, WS, CLI/REPL). POST/GET/DELETE
      `/api/v1/follows/{follow_id}/exclusions` (+ `/{exclusion_id}` on
      DELETE) — server ships under `/api/v1/` prefix; bot anti-joins
      exclusions server-side so excluded items never surface in `!updates`.
      Implemented in bot commit d8df254 (2026-08-26).
- [ ] **Saved-search alerts** — re-run a saved search from Discord: POST
      `/api/search/saved/{id}/run`; toggle daily alert PUT
      `/api/search/saved/{id}/alert`; per-search Atom feed at
      `/feed/saved/{user_id}/{search_id}` (feed handler strips trailing
      `.xml`). List via GET `/api/search/saved`. Natural `/daily N` command:
      run my saved searches, report new matches.
- [ ] Search v2 fielded operators in `!search`: `@char`/`@fandom`/`@author`
      fields, `~` fuzzy, wildcards, `with:`/`not-with:` set inclusion,
      romship/platship by tag polarity, range comparisons + `50k` shorthand.
      Bot already types plain text to `/api/search?q=` — v2 resolves the same
      way, no new endpoint; add slash-completion affordances for `@field` /
      `words:` in the search command.
- [ ] **Collection submissions + community voting** — POST
      `/api/collections/{id}/requests/{req_id}/vote` (per-curator yes/no);
      unified `/api/curator/approvals` queue aggregates collection requests +
      curator fix/metadata proposals + comment triage with type/status filter.
      `/approvals` command: show pending work needing votes.

### P3 — ops
- [ ] Systemd unit + Dockerfile for the bot (shares FicHub's Redis; needs
      network access to FicHub API + Ollama).
- [x] **Prometheus metrics** — `/metrics` on `9090` (`METRICS_ADDR` to
      override), via `std::sync::atomic` counters (avoids the `metrics` 0.21/
      0.24 crate version split that silently dropped samples). Counters:
      `fanic_bot_starts_total`, `fanic_commands_total`,
      `fanic_api_errors_total`, `fanic_cache_hit_total`,
      `fanic_cache_miss_total`, `fanic_updates_polled_total`. Shipped in
      bot v0.2.1 (canonical: `fanfic-archivist/ROADMAP.md`).
- [ ] Migrate pagination cache to Redis Streams if lists grow large.

## Constraints

- Bot NEVER touches FicHub's Postgres; talks HTTP only (REST API contract).
- Redis keys namespaced `archivist:*` to avoid collisions with FicHub.
- Auth tokens stored only in Redis, TTL 30d, refreshed on use; passwords
  never persisted.
- Library mutations (bookmark/rate/block) are ephemeral in Discord.
