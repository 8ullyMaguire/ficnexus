# fanfic-archivist

Discord companion bot for **FicHub** — a self-hosted fanfiction archive with
107-site scraper parity, pluggable recommendations, full-text body search,
LLM "Ask the Archive", a Fic Requests board, and a Roadmap consensus arena.

The bot is a **thin client** over the FicHub REST API: it never touches the
archive's database directly, and it shares FicHub's Redis for a transient
pagination cache so `!next`/`!prev` don't hammer the DB.

## Feature surface

**Socrates parity (text commands, `!` prefix):**
`!recs` · `!fresh` · `!gems` · `!roll` · `!page X` · `!next` · `!prev` ·
`!complete` · `!dead` · `!words <min> [max]` · `!fandom <name>` ·
`!xfandom <name>` · `!xfic <url>` · `!liked`

**Slash commands:**
`/recs` `/fresh` `/gems` `/roll` `/search` `/ask` `/quote` `/body`
`/download` `/bookmark` `/rate` `/updates` `/block` `/unblock` `/kudos`
`/request` `/roadmap` `/status` `/link` `/unlink` `/whoami` `/link-code`

**UX niceties:**
- Auto-URL detection — paste any supported fanfiction URL and get an ephemeral
  metadata embed with Bookmark + Download buttons (only you see it).
- Message context menu "Get Fic Metadata".
- Pagination buttons on lists (`⏮️ ⬅️ ➡️ ⏭️`) backed by the Redis cache.
- Library mutations (bookmark/rate/block) are ephemeral — personal, not public.

## Architecture

```
fanfic-archivist/
├── Cargo.toml        # standalone crate (not in the FicHub workspace)
├── ROADMAP.md        # crate backlog (level gating, RAM-aware LLM, web link, ...)
└── src/
    ├── main.rs       # poise framework: commands, events, context menus
    ├── lib.rs
    ├── api.rs        # fichub-api-client: typed reqwest wrapper for the REST API
    ├── model.rs      # API response models (mirrors frontend types.ts)
    ├── cache.rs      # Redis pagination cache (session-scoped lists)
    ├── store.rs      # token store (Discord id → FicHub JWT) + pending link codes
    ├── config.rs     # env config
    ├── util.rs       # URL detection, formatting
    └── commands/     # recs, search, library, community, link, events, admin
```

## Auth / linking

- `/link <username> <password>` — exchanges credentials **once** via
  `POST /api/auth/login`, stores only the JWT in Redis
  (`archivist:token:<discord_id>`, TTL 30d, refreshed on use). Passwords are
  never persisted.
- `/link-code` — one-time 8-char code for the future web-side confirmation
  flow (FicHub endpoint TBD).
- `/unlink`, `/whoami`.

## Environment

| Variable | Default | Purpose |
|---|---|---|
| `DISCORD_TOKEN` | — | Discord bot token (required) |
| `FANFIC_ARCHIVIST_BASE_URL` | `http://localhost:8000` | FicHub API root |
| `FANFIC_ARCHIVIST_REDIS_URL` | `redis://localhost:6379` | Shared Redis |
| `FANFIC_ARCHIVIST_PAGE_SIZE` | `5` | Embeds per page |
| `FANFIC_ARCHIVIST_API_PAGE_SIZE` | `20` | API rows per fetch |
| `FANFIC_ARCHIVIST_LINK_TTL` | `600` | Link-code TTL (sec) |
| `FANFIC_ARCHIVIST_ALLOW_UPLOAD` | `true` | Direct `/download` upload |
| `FANFIC_ARCHIVIST_MAX_UPLOAD_BYTES` | `26214400` | 25 MiB Discord cap |

## Build & run

```sh
cargo build --release
DISCORD_TOKEN=... cargo run --release
```

## License

AGPL-3.0-or-later (matches FicHub).
