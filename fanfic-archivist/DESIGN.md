# FicHub Archivist Bot — Final Design Context (for chatbot Q&A)

Comprehensive design context about the fanfic-archivist Discord bot. Paste to any
chatbot to ask follow-ups like "how to generalize this to Telegram/Matrix/etc."

## What it is

fanfic-archivist = Discord companion bot for FicHub, a self-hosted fanfiction
archive (Rust/Axum + PostgreSQL 16 + pgvector, SvelteKit frontend, 107-site
scraper parity via the fanfic-scrapers crate, LLM "Ask the Archive" via Ollama).

The bot is a **thin REST client** over the FicHub API. It:
- NEVER touches FicHub's Postgres directly — HTTP to `/api/*` only (exactly
  like the web frontend's `api/client.ts`).
- Shares FicHub's Redis instance for transient state, namespaced `archivist:*`.
- Is a standalone crate (NOT part of the FicHub cargo workspace), AGPL-3.0.
- Repo: opencommit.eu/MagicZhang/fanfic-archivist. Version 0.1.0.
- Deployed as a second systemd unit (`fanfic-archivist.service`) on the same
  host as FicHub (thinkcentre), binary at /opt/fanfic-archivist/.
- ~4,050 LOC Rust: poise 0.6 / serenity 0.12, reqwest 0.12 (rustls),
  redis 0.27 (aio + connection-manager), tokio 1, serde, sha2, uuid, chrono,
  thiserror, tracing, url, urlencoding, dotenvy, async-trait.

## Command surface (the feature set any port must reproduce)

Socrates-parity text commands (`!` prefix, both prefix + slash in most cases):
`!recs` `!fresh` `!gems` `!roll` `!page X` `!next` `!prev` `!complete` `!dead`
`!words <min> [max]` `!fandom <name>` `!xfandom <name>` `!xfic <url>`
`!liked`

Slash commands: `/recs` `/fresh` `/gems` `/roll` `/search` `/ask` `/quote`
`/body` `/download` `/bookmark` `/rate` `/updates` `/block` `/unblock`
`/kudos` `/request` `/roadmap` `/status` `/link` `/unlink` `/whoami`
`/link-code` + context menu "Get Fic Metadata".

UX niceties:
- Auto-URL detection: paste any supported fanfiction URL → ephemeral metadata
  embed with Bookmark + Download EPUB buttons (only the author sees it).
- Message context menu "Get Fic Metadata".
- Pagination button row (⏮ ⬅ ➡ ⏭) on lists, backed by Redis session cache.
- Library mutations (bookmark/rate/block) are ephemeral (private to user).
- Search/ask failure fallback: `/search` fails → falls back to `/ask`
  (natural-language search), logs to Redis, and optionally asks local Ollama
  to diagnose the root cause and echoes a short hint.

## Architecture / module layout

```
fanfic-archivist/
├── Cargo.toml          # standalone crate (not in FicHub workspace)
├── ROADMAP.md          # backlog: level-gating, RAM-aware LLM, Flipper parity, ops
└── src/
    ├── main.rs         # poise framework bootstrap: commands, prefix "!", event
    │                   # handler (Message + InteractionCreate), global slash
    │                   # registration, serenity client wiring
    ├── lib.rs
    ├── api.rs          # FichubClient: typed reqwest wrapper (get/post/delete,
    │                   # Bearer auth, err-field checking) + SearchParams query builder
    ├── model.rs        # API response models mirroring frontend/src/lib/api/types.ts
    ├── cache.rs        # PageCache: Redis pagination lists + ask/search response cache
    ├── store.rs        # TokenStore (platform user id → FicHub JWT, TTL 30d) +
    │                   # PendingLinkStore (one-time 8-char link codes)
    ├── config.rs       # env config (FANFIC_ARCHIVIST_*, DISCORD_TOKEN)
    ├── debug.rs        # auto LLM diagnosis of failed API calls via Ollama
    ├── error.rs        # BotError (HTTP/API/JSON/Redis/Discord/NotLinked/...)
    ├── util.rs         # supported-host URL detection, normalize, format_words, truncate
    └── commands/
        ├── recs.rs     # recs/fresh/gems/roll + pagination (!page/!next/!prev)
        │               # + filtering (!complete/!dead/!words/!fandom/!xfandom/!xfic/!liked)
        ├── search.rs   # /search /ask /quote /body + render + searchlog
        ├── library.rs  # /download /bookmark /rate /updates /block /unblock /kudos
        ├── community.rs# /request (list/create) /roadmap (top/controversial)
        ├── link.rs     # /link /unlink /whoami /link-code
        ├── events.rs   # auto-URL detection, context menu, component handler
        └── admin.rs    # /status health embed
```

## Design decisions (the important ones for generalization)

1. **Thin-client / API-first**: the entire bot is a typed HTTP wrapper around
   the FicHub REST API. All platform-specific logic (Discord) is confined to
   the command layer + main.rs. The API client, models, cache, and token store
   are platform-agnostic. This is the seam any Telegram/Matrix port hooks into.

2. **Auth = platform user id → FicHub JWT mapping in Redis** (key
   `archivist:token:<platform_user_id>`, TTL 30d, refreshed on every authed
   call via `touch`). `/link <username> <password>` exchanges credentials ONCE
   via `POST /api/auth/login` and stores only the JWT (passwords never
   persisted). `/link-code` generates a one-time 8-char code (unambiguous
   alphabet, no I/O/0/1) for a future web-side confirmation flow. The platform
   identity is just the key — Telegram chat id / Matrix user id would slot in
   the same way.

3. **Redis shared with the host app, `archivist:*` namespace**:
   - Pagination: `archivist:session:<uuid>` → whole JSON list (PageList),
     `archivist:user:<id>:latest` → current session pointer,
     `archivist:page:<session>` → current page. Full list cached so
     !next/!prev/!page never re-hit the API.
   - Response cache (protects the public bot from hammering LLM/API):
     `archivist:askcache:<sha256(trim+lowercase(q))>` → AskResponse, TTL 24h;
     `archivist:searchcache:<sha256(full_query_string)>` → SearchResponse,
     TTL 5 min. Best-effort by design — Redis failure = miss, never breaks the
     command.
   - Search analytics log: `archivist:searchlog:<YYYY-MM-DD>` Redis list,
     capped 500 entries/day, 7-day TTL. Used to debug queries.

4. **Ephemeral vs public**: library mutations (bookmark/rate/block) and
   `/link` replies are ephemeral (private to the invoking user); lists and
   metadata embeds are public channel messages. Telegram/Matrix equivalents:
   Telegram reply-to-sender / privacy-mode bots; Matrix private notes.

5. **Failure fallback ladder** (robustness pattern worth copying):
   `/search` API failure → try `/ask` (NL search, more robust, server-side
   falls back to plain search) → if that also fails, log + optionally call
   local Ollama (`FANFIC_ARCHIVIST_OLLAMA_URL` default localhost:11434,
   model `ornith:9b` default, `FANFIC_ARCHIVIST_LLM_DEBUG=0` disables) to
   auto-diagnose the root cause; cache the ask fallback result under askcache
   so repeated failing queries don't re-run the LLM.

6. **Command duality**: every command is registered both as slash command and
   `!` prefix command (Socrates parity) via poise attributes. Poise handles
   argument parsing for both. On another platform you'd keep the same command
   bodies but re-parse args (Telegram: bot commands + text args; Matrix:
   `!cmd` in message text).

7. **Pagination UX abstraction**: list commands store the full result list in
   Redis (session-scoped), render page 1 as embeds with a button row
   `⏮ ⬅ ➡ ⏭` (custom ids `<session>:first|prev|next|last`), and the component
   handler mutates the message in place. Text `!page/!next/!prev` read the
   same session state. Porting = same session store, different button renderer
   (Telegram inline keyboards; Matrix reactions or command-prefix nav).

8. **Hardcoded FicHub web URL** (a wart to abstract when generalizing):
   embeds link to `https://fichub.example.com/fic/<url_id>` — placeholder
   domain baked into commands/mod.rs + events.rs. A config
   `FANFIC_ARCHIVIST_WEB_URL` would fix it.

9. **No guild/server concept yet**: ROADMAP P1 wants level-gated access
   (FicHub user `level` 0-100 stored at link time, per-command min-level) +
   guild allowlist/denylist. Relevant for Telegram/Matrix (chat allowlists).

10. **Upload cap**: `/download` can upload EPUB to Discord when < 25 MiB
    (`FANFIC_ARCHIVIST_ALLOW_UPLOAD`, `MAX_UPLOAD_BYTES` default 26214400);
    otherwise returns link. Telegram: 50 MB bots; Matrix: 100 MB+ media repo.

## API endpoints the bot consumes (the full contract)

Auth: `POST /api/auth/login`, `GET /api/auth/me`
Export: `GET /api/epub?q=<url>`, `GET /api/meta?q=<url>`,
`GET /api/epub/convert?q=<url>&format=mobi|pdf|azw3` (lazy Calibre)
Recommendations: `GET /api/recommendations?q=&n=`,
`GET /api/recommendations/personal` (auth), `GET /api/recommendations/strategies`
Search: `GET /api/search?<params>`, `POST /api/search/ask`,
`GET /api/search/body?q=&page=&per_page=`
Social (auth): `GET /api/v1/feed?page=`, `POST /api/bookmarks`,
`DELETE /api/bookmarks/{work_id}`, `POST /api/ratings`,
`POST /api/blocks`, `DELETE /api/blocks/{url_id}`, `GET /api/kudos/{work_id}`
Community: `GET /api/requests?status=&page=`, `POST /api/requests` (auth),
`GET /api/roadmap/consensus`
Response envelope convention: `{ "err": 0, ... }` — err 0 = ok; bot's
`parse_response` checks HTTP status then the `err` field and surfaces
`err`/`msg` as BotError::Api.

Search params (SearchParams.to_query): q, page, per_page, min_words,
max_words, complete, source, fandom, exclude_fandom, main_char_attr,
min_kudos, sort, include_tags, exclude_tags.

## Config surface (env vars — ALL platform-specific in one place)

| var | default | purpose |
|-----|---------|---------|
| DISCORD_TOKEN | (required) | platform token |
| FANFIC_ARCHIVIST_BASE_URL | http://localhost:8000 | FicHub API root |
| FANFIC_ARCHIVIST_REDIS_URL | redis://localhost:6379 | shared Redis |
| FANFIC_ARCHIVIST_PAGE_SIZE | 5 | embeds per page |
| FANFIC_ARCHIVIST_API_PAGE_SIZE | 20 | API rows per fetch |
| FANFIC_ARCHIVIST_LINK_TTL | 600 | link-code TTL (sec) |
| FANFIC_ARCHIVIST_ALLOW_UPLOAD | true | direct /download upload |
| FANFIC_ARCHIVIST_MAX_UPLOAD_BYTES | 26214400 (25 MiB) | Discord cap |
| FANFIC_ARCHIVIST_OLLAMA_URL | http://localhost:11434 | debug LLM |
| FANFIC_ARCHIVIST_LLM_MODEL | ornith:9b | debug LLM model |
| FANFIC_ARCHIVIST_LLM_DEBUG | (unset = on) | set 0 to disable |

## Generalization seams (how to port to Telegram / Matrix / etc.)

The design was built with platform decoupling in mind; these are the seams:

1. **Adapter boundary = `commands/` + `main.rs` only.** `api.rs`, `model.rs`,
   `cache.rs`, `store.rs`, `config.rs`, `error.rs`, `util.rs` are
   platform-agnostic already. A port keeps those untouched and writes a new
   front-end: command registry, arg parsing, message/event loop, reply
   rendering (embed → Telegram HTML/Matrix formatted body), button row →
   Telegram inline keyboard / Matrix reaction or `/nav` command.

2. **Rendering layer**: extract embed builders (`rec_embed`, `search_embed`,
   metadata embed) behind a trait or a `Renderable` struct → render per
   platform. Text fallback already exists in commands like `/updates` /
   `/request` which use plain `ctx.say(string)` — those port trivially.

3. **Session/pagination state** is already transport-agnostic (Redis JSON
   lists keyed by uuid; per-user "latest" pointer keyed by platform user id).
   A Telegram port keys sessions by `telegram:<chat_id>:<user_id>` and offers
   `⬅️ ➡️` inline buttons or `!next/!prev` text commands — no backend change.

4. **Identity/linking**: token store keyed by opaque platform user id string.
   Telegram = `user.id`; Matrix = `@user:server` full MXID. The `/link`
   password flow and `/link-code` one-time-code flow work unchanged.

5. **Multi-platform = multiple front-ends, ONE core.** The cleanest
   generalization: split into `fichub-bot-core` crate (client + models + cache
   + store + command logic + a `Platform` trait: `send_embed`, `reply_ephemeral`,
   `buttons`, `user_id`, `on_message`) and thin platform binaries
   (`bot-discord`, `bot-telegram`, `bot-matrix`). This mirrors how
   `fanfic-scrapers` and `forum-core` were already extracted as standalone
   publishable crates.

6. **Event-driven additions** (ROADMAP): `/notify` webhook subscriptions,
   fic-update notifications — Telegram bots can push via getUpdates long
   polling; Matrix needs an appservice or a bot room; Discord needs webhooks.
   The core API (FicHub notifications endpoint) is the same.

7. **Platform-specific limits to plan for**:
   - Discord: 2000-char message cap (embeds 4096), 25 MiB upload, ephemeral
     replies, slash commands need global registration (up to 1h propagation),
     MESSAGE_CONTENT privileged intent required for `!` commands.
   - Telegram: 4096-char message cap, 50 MiB upload via Bot API, inline
     keyboards, HTML/markdown parse modes, bot privacy mode blocks auto-URL
     detection unless disabled via BotFather, no ephemeral (reply-to only).
   - Matrix: no native slash commands — convention is `!cmd` in message text;
     formatted messages via `m.room.message` format=org.matrix.custom.html;
     media via mxc:// uploads; bots are regular accounts (appservice or
     autojoin room); pagination UX = reactions or text nav.
   - Slack: 4000-char blocks, 40+ interactive components, slash commands need
     request URL verification, sockets mode for events.

## Deployment facts (for ops follow-ups)

- Build: `cd /home/alvaro/.cache/fanfic-archivist-repo && unset
  GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES && CARGO_TARGET_DIR=/
  media/alvaro/code-worktrees/.fichub-main-target cargo build --release
  --bin fanfic-archivist` (~1m37s).
- Deploy: scp binary → thinkcentre:/tmp → stop service, mv to
  /opt/fanfic-archivist/, chmod 755, start. Systemd unit mirrors fichub.service
  (Type=simple, User=alvaro, EnvironmentFile=/opt/fanfic-archivist/.env,
  After=fichub.service redis-server.service, Restart=on-failure).
- Binary is ~15-20 MB RSS (well under top-15 of process list).
- Redis keys used: archivist:session:*, archivist:user:<id>:latest,
  archivist:page:*, archivist:token:<id>, archivist:link:<code>,
  archivist:askcache:*, archivist:searchcache:*, archivist:searchlog:<date>.
- Git: remote `opencommit` (not origin); push `git push opencommit main`;
  credentials via `store --file ~/.config/git/credentials`.
- Poise gotchas (from the port notes): CreateReply.embed() singular no
  .embeds(); CreateEmbed::footer takes value not closure; CreateActionRow is
  an enum (CreateActionRow::Buttons(vec![])); multi-codepoint emoji '⏮️'
  FAILS — use single codepoint '⏮'; FrameworkBuilder has no token()/intents()
  — wire serenity::Client::builder().framework(framework) yourself (client
  must be mut); context menus are regular commands with
  context_menu_command attr; event_handler gets data as 4th arg not via ctx;
  redis 0.27 needs `connection-manager` feature for ConnectionManager;
  slash command description = first doc-comment line, max 100 chars.

## Roadmap / backlog (what a generalizer would keep in mind)

- P1: level-gated access (per-command min FicHub level + guild allowlist),
  API-token link flow, web-side link confirmation endpoint, RAM-aware LLM
  gating (check /api/health/resource before spawning LLM calls; degrade /ask
  to plain search).
- P1: Flipper parity — tracked-message state machines in Redis
  (archivist:msg:<channel>:<message>), !sus suspicious-author detection,
  !stats profile stats, !filters, !fresh recency, !cutoff/!wordcount/!year
  range tuning, /authors, /mood.
- P2: filter-chip select menus, /notify webhooks, OPDS/PWA deep-links,
  /curator leaderboard, /suggest, /vote, presence nickname, i18n.
- P3: systemd/Dockerfile, prometheus metrics, Redis Streams for large lists.
