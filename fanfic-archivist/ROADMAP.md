# fanfic-archivist — Crate Roadmap

Discord companion bot for FicHub. Thin client over the FicHub REST API
(`fichub-api-client` internal crate); shares FicHub's Redis for pagination.

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

## Next (backlog)

### P1 — auth + gating
- [x] **Level-gated access** (user requirement 2026-08-15): gate bot commands
      by FicHub user `level` (site-wide 0–100, F7) and/or Discord role. Shipped
      in bot v0.2.1 (canonical roadmap: `fanfic-archivist/ROADMAP.md` in the bot
      crate repo).
      - [x] **Level-gating enforcement** — `require_level()` + `require_level_for()`
        gate `bookmark`/`rate`/`updates`/`block`/`unblock`/`kudos` against
        `StoredToken.level` (kudos=2 reviewer+, others=1 linked member).
        `DiscordBotError::InsufficientLevel` reports required vs current level.
      - [ ] **Guild-level access** allowlist/denylist by server id.
- [ ] API-token flow (P8#88): user pastes a scoped token from FicHub profile →
      `/link token`; no password exchange. Fallback until then is the
      password flow.
- [ ] Web-side one-time link confirmation endpoint on FicHub:
      `POST /api/discord/link` consumes the `/link-code` code, ties Discord id
      → FicHub user server-side (requires bot token on the server).

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
