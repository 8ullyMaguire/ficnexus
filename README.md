# ficnexus — a fanfiction exporter

Paste a story URL, get a downloadable file. That is the whole service.

ficnexus scrapes a work from any of ~100 supported fanfiction archives and
builds **EPUB, HTML, TXT, Markdown, FB2, KEPUB, DOCX** in pure Rust, plus
**MOBI, PDF, AZW3** through a Calibre sidecar. It keeps a cache of fetched
works and built files so a repeat download is instant.

There are no accounts, no bookmarks, no reader, no forum, no feed,
no recommendations. Those live in **darkhold**, a separate service. This repo
is only the exporter; the reduction that produced it is written down in
[`docs/EXPORTER-SCOPE.md`](./docs/EXPORTER-SCOPE.md), including the before/after
route table and the list of tables that are now unused but deliberately not
dropped.

**Stack**: Rust (edition 2024) · Axum 0.8 · SQLx 0.9 · PostgreSQL · Redis ·
optional Ollama (author-link resolution) · optional Calibre container.

---

## Quick start

Prerequisites: PostgreSQL (with `pgvector` + `pg_trgm`), Redis, and a Rust
toolchain. Calibre-in-Docker is optional and only needed for MOBI/PDF/AZW3.

```bash
cp .env.example .env      # DATABASE_URL, REDIS_URL are the only required vars
cargo run                 # http://localhost:3000  (PORT)
```

Schema migrations in `migrations/` are applied by sqlx **on start-up**; the
`migrate` binary does the same thing one-shot:

```bash
cargo run --bin migrate
```

## Using it

Web: open `/`, paste a URL, pick a format. API:

```bash
# 1. ask for an export — returns metadata + a download URL
curl 'http://localhost:3000/api/epub?q=https://archiveofourown.org/works/12345'
# {"err":0,"url_id":"…","hashes":{"epub":"…"},
#  "epub_url":"/cache/epub/…?h=…","meta":{…},"tags":[…],"sources":[…]}

# 2. fetch the file
curl -OJ 'http://localhost:3000/cache/epub/<url_id>?h=<hash>'

# a specific format directly
curl 'http://localhost:3000/api/epub/convert?format=mobi&q=<url>'

# metadata only
curl 'http://localhost:3000/api/meta?q=<url>'
```

## Routes

Eleven entries. `grep -n '\.route(' src/server.rs` is the authoritative list.

| Method | Path | Purpose |
|---|---|---|
| GET | `/` | the UI: URL box, format picker, download links |
| GET | `/api/` | machine-readable description of this surface |
| GET | `/api/epub?q=<url>[&format=]` | scrape, build, return metadata + file URLs |
| GET | `/api/epub/convert?q=<url>&format=` | build one specific format |
| GET | `/api/meta?q=<url>` | metadata only |
| GET | `/api/health` | database + Redis ping |
| GET | `/api/formats` | supported formats (and which need Calibre) |
| GET | `/cache/{etype}/{url_id}/{fname}` | serve a built file, md5 in the filename |
| GET | `/cache/{etype}/{url_id}?h=<hash>` | serve a built file, or explain why not |
| GET | `/api/pow/challenge` | proof-of-work challenge (flagged clients) |
| POST | `/api/pow/solve` | submit a solved challenge |

Everything else answers 404. `tests/exporter_api.rs` asserts both halves of
that sentence.

## Output formats

| format | engine |
|---|---|
| `epub` | `epub-builder` (default) |
| `html` | zipped HTML bundle |
| `txt`, `md`, `fb2`, `kepub`, `docx` | pure Rust builders in `src/export/` |
| `mobi`, `pdf`, `azw3` | Calibre `ebook-convert` in the container named by `CALIBRE_CONTAINER` |

Without a Calibre container those three return a clear error instead of failing
at download time.

## Supported sites

`crates/scrapers` ships native adapters for ~100 archives — AO3 and the OTW
siblings, FanFiction.net, FictionPress, RoyalRoad, ScribbleHub, FimFiction,
Literotica, Wattpad, the XenForo boards (SpaceBattles, SufficientVelocity,
QuestionableQuesting, fiction.live), the eFiction families, WordPress-novel
sites, and the adult-gated archives (fanfics.me, fictionhunt, inkbunny, sofurry,
dokuga, …). `ScraperRegistry::new()` registers them at boot and logs the count.

Some sites block datacenter IPs; the export path falls back to Internet
Archive snapshots when `WAYBACK_FALLBACK_ENABLED=true`.

## Architecture

```
client ─► Axum :3000
            │
            ├─ scrape/ ───── fanfic-scrapers (100 adapters), Wayback fallback,
            │                quality classifier, self-heal telemetry
            ├─ export/ ───── EPUB / HTML / TXT / MD / FB2 / KEPUB / DOCX builders
            │                + Calibre sidecar for MOBI / PDF / AZW3
            ├─ cache/ ────── hash-keyed file tree (CACHE_DIR) + body snapshots
            ├─ db ────────── fic_info / export_log ledger, tag + work bookkeeping
            ├─ Redis ─────── rate-limit buckets, proof-of-work state
            └─ limiter/ ──── tiered per-IP/per-client buckets, datacenter ASNs
```

No background workers: nothing is spawned at start-up beyond the HTTP listener.

## Anti-bot

* tiered Redis token buckets per IP and per `x-client-id` (download tier for
  `/api/epub`, stricter for flagged clients);
* datacenter-ASN blocking when `MAXMIND_DB` points at a GeoLite2-ASN database;
* proof-of-work challenge for shadowbanned clients (429 with a challenge, then
  `/api/pow/solve`);
* `automated=true` requests are refused outright.

## Database

The schema is 45 migration files / 180 tables and **none of it was dropped**.
The export path names 16 of those tables in SQL; the other 164 are unused and
left in place, with their data. The full list, grouped by the subsystem that
used them, is in [`docs/EXPORTER-SCOPE.md`](./docs/EXPORTER-SCOPE.md#5-database-nothing-dropped-164-tables-now-unused).

New migrations must be idempotent (`CREATE TABLE IF NOT EXISTS`,
`ON CONFLICT DO NOTHING`) — sqlx re-applies a file whose recorded checksum was
deleted, and the service refuses to start if a checksum mismatches.

## Testing

```bash
cargo test                 # 270 unit tests + 16 integration tests
                           # (5 integration tests are DB-gated and #[ignore]d)

set -a; . ./.env; set +a
cargo test --test exporter_api -- --include-ignored --test-threads=1
                           # all 16, against a real Postgres + Redis
```

`tests/exporter_api.rs` covers three things:

* the export helpers (slug, info string, meta JSON, format list);
* `epub_is_built_from_chapters` — metadata + chapters in, a real EPUB (zip with
  a `mimetype` entry and a package document) out;
* the shipped router — every kept route is mounted, 21 old community paths
  answer 404, `/api/health` reports `db:true redis:true`, and a seeded cached
  export is served byte-for-byte with its md5 re-validated.

A scratch database can be built from `migrations/` alone:

```bash
bash scripts/provision_test_db.sh      # idempotent; prints DATABASE_URL
```

## Deployment

```bash
just deploy        # release build, stage to thinkcentre, swap binary, restart,
                   # roll back if /api/health fails
```

The release binary runs from `/opt/fichub/fichub` (local disk, not the NFS
repo) under the `fichub.service` unit. See `infra/docker/` for a container
layout (app + Calibre).

## Stack and language policy

What this repo is: a Rust (edition 2024) service — axum 0.8 on tokio for the
HTTP surface, sqlx 0.9 against PostgreSQL for the cache of fetched works and
built files, format writers in this workspace and a Calibre sidecar for MOBI,
PDF and AZW3. No accounts, no reader, no recommendations: those are darkhold's
job.

The estate default for new work is an axum + Postgres core, a SvelteKit front
end, the shared design tokens for anything that draws a UI, and contracts
between services — work-manifest v1, OpenAPI, RSS/ActivityPub — instead of a
shared database. Services read each other's published contracts; they do not
read each other's tables.

This repo is the default, on its server side. Its only UI is a small
server-rendered page around the paste-a-URL form, so the SvelteKit and design
token parts of the default do not apply; the contract part does, and the
cache is the only thing it owns.

The rules that apply to every repo in the estate are in
`10-Projects/estate-master-prompt.md` in the secondbrain vault.

## License

AGPL-3.0-only.

## Credits

Inspired by [fichub.net](https://fichub.net) and
[FanFicFare](https://github.com/JimmXinu/FanFicFare) — the site adapters in
`crates/scrapers` follow FanFicFare's site coverage.
