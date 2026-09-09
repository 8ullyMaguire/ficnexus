# FicHub Greenfield Rebuild — "Maximum Popularity" Specification

> **Date**: 2026-09-03
> **Status**: Proposal — a from-scratch design. Nothing here is implemented; this is the answer to *"if we deleted the repo and rebuilt FicHub with the single goal of maximum adoption, what would we build, in what order, and how?"*
> **Audience**: A junior developer with basic Rust + TypeScript. Every section is either directly implementable or points at existing code in this repo that should be **ported** (see Appendix A — Porting Map).
> **Reads alongside**: `SPECIFICATION.md` (what exists today), `docs/SITE-PREMISE.md`, `FICHUB_DESIGN.md`.

---

## Table of Contents

0. [Executive Summary](#0-executive-summary)
1. [What "Maximum Popularity" Means](#1-what-maximum-popularity-means)
2. [Diagnosis: Why the Current Build Doesn't Grow](#2-diagnosis)
3. [Product Thesis & Core Loop](#3-product-thesis--core-loop)
4. [Deliberately Cut Features](#4-deliberately-cut-features)
5. [Users & Personas](#5-users--personas)
6. [Architecture Overview](#6-architecture-overview)
7. [Data Model (DDL)](#7-data-model)
8. [API Specification](#8-api-specification)
9. [Backend Subsystems](#9-backend-subsystems)
10. [Frontend & SEO](#10-frontend--seo)
11. [Reader, Library & Download UX](#11-reader-library--download-ux)
12. [Discovery & Recommendations v1](#12-discovery--recommendations-v1)
13. [Accounts, Moderation & Trust](#13-accounts-moderation--trust)
14. [Legal & Platform Risk](#14-legal--platform-risk)
15. [Anti-Bot & Abuse](#15-anti-bot--abuse)
16. [Observability, QA & Testing](#16-observability-qa--testing)
17. [Deployment & Scaling](#17-deployment--scaling)
18. [Performance Budgets](#18-performance-budgets)
19. [Milestones M0–M5](#19-milestones)
20. [Junior-Dev Implementation Guide](#20-junior-dev-implementation-guide)
21. [Appendix A — Porting Map from the Current Repo](#21-appendix-a--porting-map)
22. [Appendix B — Environment Variables](#22-appendix-b--environment-variables)

---

## 0. Executive Summary

The current build (2026-09) is an impressive engineering artifact: 107 source-site adapters in `scrapers/`, a pluggable recommendation platform (~12 strategies), a 100-level progression system, an extension marketplace, a 7-level trust ladder, a forum, recipes, bounties, and a MaxDiff consensus arena — all behind one 1,169-line router (`src/server.rs`), ~70 route modules, and a static-SPA frontend on a single box.

From a growth standpoint this is inverted:

1. **~80% of the code serves ~5% of the value.** Gamification, marketplaces, and consensus arenas are retention *multipliers* for an audience that does not exist yet. They add zero acquisition.
2. **The frontend is invisible to search engines.** `adapter-static` + SPA fallback means every work page is JS-gated. For a content site, organic search is *the* acquisition channel; this is the single biggest popularity bug in the codebase.
3. **The identity ("download other people's fics") is legally fragile and content-poor.** It depends on outbound reachability to sites that actively block it (the docs note the AO3/FFN Cloudflare block), and it produces no native content to rank for.

The rebuild inverts three things:

- **Acquisition = SEO + share loops.** Server-rendered pages for every work/author/fandom/tag, sitemaps, structured data, OG share cards, bookmarklet, "sent from FicHub" EPUB metadata.
- **Retention = a personal library that follows updates.** Paste a URL once → the fic lives in your library; you get pinged on new chapters; you read it in a great reader. No XP required.
- **Legitimacy = native hosting + attribution + fast takedowns.** Original-fiction upload from day one, canonical attribution for imported works, one-click DMCA flow. This is what lets the import side survive at all.

**Everything else is deferred.** Section 4 lists exactly what is cut and the metric gate that must be met before each feature can return.

**Stack** (deliberately close to today's so code can be ported): Rust + Axum 0.8 + SQLx + PostgreSQL 16 + Redis; SvelteKit 5 with `adapter-node` (**SSR, not static**); S3-compatible object storage for generated ebooks; a Postgres-backed job queue; Cloudflare in front. Ollama becomes an **optional, flag-gated** dependency the site must run without.

---

## 1. What "Maximum Popularity" Means

Popularity is not feature count. It is the intersection of five layers:

| Layer | Question | Owner in this spec |
|---|---|---|
| **Acquisition** | Can a stranger find a specific fic via Google in one search? | §10 SEO, §9.6 sitemaps |
| **First-run value** | Does the URL→EPUB hook deliver value in <10 seconds, before signup? | §9.3 ingest, §11 |
| **Retention** | Does the library pull the user back when a followed fic updates? | §11.2 follows/notifications |
| **Sharing** | Does using the site produce artifacts others see? | §10.5 OG cards, bookmarklet, OPDS |
| **Trust** | Would a fic *author* — not just a reader — tolerate this site? | §14 legal, §13 moderation |

### 1.1 North star and guardrail metrics

One north star: **Weekly Active Readers (WAR)** — distinct users (or anonymous devices) who read or download at least one work in 7 days.

All metrics come from a single zero-PII `analytics_events` table (§7), preserving the current build's analytics principle.

| Metric | Definition | Target @ 6 months | Gate rule |
|---|---|---|---|
| WAR | weekly active readers | 5,000 | — |
| Hook conversion | paste-URL → completed download | ≥ 55% | < 40% blocks all non-SEO work until fixed |
| Signup conversion | completed download → account created | ≥ 25% | gates every M5 feature |
| D7 retention | new account returns within 7 days | ≥ 20% | < 10% freezes ALL new features; fix retention only |
| Organic share of traffic | search-engine referrals / total | ≥ 40% | < 20% forces SEO audit before any new feature |
| Update-notification CTR | clicks / delivered update notifications | ≥ 15% | tunes notification copy and medium |
| Takedown SLA | DMCA notice → content removed | < 24h | hard requirement (§14) |

**Process rule that prevents feature sprawl from re-emerging:** every feature PR must name the metric it moves and the gate it must beat. A feature that moves no metric is rejected.

---
## 2. Diagnosis: Why the Current Build Doesn't Grow

Concrete, code-grounded issues — each mapped to the section of this spec that fixes it:

| # | Problem in current build | Evidence | Fix in this spec |
|---|---|---|---|
| P1 | Work pages are JS-only: no SSR, no per-page meta tags, no sitemaps | `frontend/svelte.config.js` uses `adapter-static`; server falls back to `index.html` | §10: `adapter-node` SSR, sitemaps, JSON-LD |
| P2 | Value requires an account deep into the flow | bookmarks, follows, reader-position save are all auth-gated | §11.1: anonymous library via signed device cookie |
| P3 | Feature sprawl dilutes the core loop; huge onboarding cost for new devs | ~70 route modules; progression, marketplace, recipes, bounties, quests, trust all live simultaneously | §4: explicit cut list with return gates |
| P4 | Single-box deployment; one machine is SPOF and capacity ceiling | ThinkCentre + systemd, binary built on another host via NFS | §17: stateless containers, managed-Postgres path |
| P5 | Identity is "download from other sites", which source sites litigate or block | docs note the AO3/FFN Cloudflare block as an open gap | §14: native hosting + attribution-first import |
| P6 | Full-text body search over third-party content is a legal + storage liability | `src/search/body.rs`, `src/body_cache.rs` | §9.5: metadata search public; body search only over copies in *your own* library |
| P7 | Signup gated behind anti-bot puzzles (PoW) before the user has seen any value | `src/routes/pow.rs`; registration applications/invites | §13.1: open registration; PoW only on abuse signals |
| P8 | Recommendations need Ollama embeddings to be interesting | `OLLAMA_URL` is a hard dep of recs and Ask-the-Archive | §12: recs v1 needs zero ML; LLM features flag-gated |
| P9 | No mobile story beyond the desktop SPA; PWA listed as future work | `docs/plans/pwa-progressive-web-app.md` not started | §10.4: mobile-first layout + installable PWA in M2 |
| P10 | Social features (forum, bounties, requests) need a community that does not exist yet; empty rooms drive new users away | forum/requests pages live but content is seed-only | §13: community features unlock behind activity thresholds |

---

## 3. Product Thesis & Core Loop

### 3.1 One-sentence pitch

**"Paste any fanfic URL — read it here, keep it forever, get told when it updates."**

Everything on the site must serve that sentence. The homepage is a single input box. That's the whole above-the-fold.

### 3.2 The core loop (this is the product)

```
ACQUIRE                ACTIVATE                RETAIN                 SHARE
────────────────────   ─────────────────────   ────────────────────   ────────────────────
Google: "[fic name]    Paste URL (no account)  Fic follows updates →  Read-later card →
epub download"  ────►  ──► read in reader or   ──► push/email/RSS      friend pastes their
SSR work page ranks    download EPUB in <10s   ping → return visit    own URL in
```

The loop must close **without an account** for the first two steps, and the account is only demanded at the moment the user wants to *keep* something (bookmark, follow, sync position). This "paywall the saving, not the reading" rule is the single highest-impact conversion decision in the spec.

### 3.3 What the product is NOT (v1)

- Not a forum, not a social network, not a game.
- Not a scraper marketplace or a plugin ecosystem.
- Not an AI product (one optional LLM feature, flag-gated, §12.4).
- Not a replacement for AO3's community features — it is a *reader's* tool that AO3/FFN/etc. never built.

### 3.4 The three pillars, ranked by build priority

1. **Ingest & convert** (works for any of 107 supported hosts) — port `scrapers/` wholesale. This is the moat; nobody else supports this many sites.
2. **SSR + SEO** — every ingested work becomes a fast, indexable, shareable page. This is the acquisition engine.
3. **Library & notifications** — bookmarks, follows, update detection, email/RSS/push. This is the retention engine.

---
## 4. Deliberately Cut Features

Each cut feature names the gate that must be met before it can be reconsidered. This list is as important as the build list — it is the direct antidote to the current codebase's sprawl.

| Cut feature (exists today) | Why cut | Return gate |
|---|---|---|
| 100-level progression + ability tree (`src/progression.rs`) | pure retention cosmetics; zero acquisition; heavy UI | D7 retention ≥ 20% for 8 consecutive weeks |
| Extension marketplace (themes/skins/recipes/layouts) | supply-side feature with no suppliers; dead inventory for new users | ≥ 2,000 WAU AND ≥ 50 active creators |
| Recipe builder / pluggable rec strategies (~12 engines) | v1 recs (§12) achieve the same perceived quality with 3 deterministic strategies and no ML infra | rec CTR < 5% for 4 weeks after v1 ships |
| Forum + bounties + quests | communities need density; an empty forum actively repels | ≥ 1,000 DAU AND ≥ 20 daily organic posts elsewhere (comments) |
| Trust ladder TL0–TL6 as a user-facing system | replace with a simple 3-tier role model (§13.3); auto-promotion complexity is unneeded below 10k users | ≥ 10k registered users or first moderation crisis |
| Roadmap consensus / MaxDiff arena | process feature, not user feature | never in v1; keep as admin survey tool |
| Fic Requests prompt board | needs writers + requesters simultaneously (two-sided cold start) | ≥ 500 DAU |
| Collections, shelves, work-proposal voting, tag-election systems | collapsed into plain bookmarks + lists (§11.3) | ≥ 1,000 MAU with list-creation behavior observed |
| Ask-the-Archive (LLM NL search) | good feature, wrong time; keep flag-gated | organic ≥ 40% AND recs v1 shipped |
| Send-to-Kindle via SMTP | fragile, per-user config burden | ≥ 1,000 MAU requesting it (analytics counter) |
| Blind Date, idle-tick XP, leaderboards | gimmicks | never in v1 |
| ActivityPub federation (`src/activitypub/`) | high effort, near-zero audience overlap with fic readers | ≥ 25k MAU |
| Body full-text search across all ingested works | legal exposure (§14) + huge storage | only with legal review; default = own-library scope |
| OPDS catalog | keep! tiny cost, outsized e-reader niche value | stays (§11.5) |
| Multi-language UI (en/fr/es/de/pt-BR) | i18n maintenance burden before product-market fit | ≥ 5k MAU from non-English locales (analytics) |

**Porting rule**: cut ≠ delete from the repo. Keep the code on a branch (`archive/v1-feature-parity`) so features can be resurrected behind their gates without rewriting history.

---

## 5. Users & Personas

| Persona | Share (target) | Need | Where the spec serves them |
|---|---|---|---|
| **The Binger** (mobile reader) | 45% | reads 100k+ words/week on a phone; hates ads and app stores | §11 reader (offline PWA, position sync, dark/sepia) |
| **The Archivist** (hoarder) | 20% | downloads EPUBs, organizes, fears link-rot | §9.3 conversion, §11.3 lists, OPDS, per-chapter EPUB diffing |
| **The Following Fan** (serial reader) | 25% | reads WIPs as they update across 5 sites | §11.2 follows + update notifications (email/RSS/push) |
| **The Author** (original or fic writer) | 8% | wants readers and a stable permalink for their work | §14: native upload, canonical profile, `rel=canonical` attribution |
| **The Curator** (rec-list maker) | 2% | builds lists that others follow; high influence | §11.3 public lists with SSR pages + OG cards (free SEO content) |

Note the absent persona: *the gamification enjoyer*. That audience only exists after the five above are served.

---
## 6. Architecture Overview

### 6.1 System diagram

```
                    ┌──────────────────────────────────────────────────┐
                    │                   Cloudflare (CDN, WAF)          │
                    └───────────────────────┬──────────────────────────┘
                                            │
                    ┌───────────────────────▼──────────────────────────┐
                    │            SvelteKit 5 — adapter-node            │
                    │  SSR pages (works/authors/tags/fandoms/lists)    │
                    │  + /api proxy to Rust backend (same origin)      │
                    └───────────────────────┬──────────────────────────┘
                                            │ internal HTTP / :8000
 ┌──────────────┐   ┌───────────────────────▼──────────────────────────┐
 │ S3-compatible│◄──┤                Axum 0.8 API server (stateless)   │
 │ object store │   │  ┌────────────┐ ┌──────────┐ ┌────────────────┐  │
 │ (epubs,      │   │  │ ingest     │ │ library  │ │ notifications  │  │
 │  covers,     │   │  │ (scrapers/ │ │ + reader │ │ (email/RSS/    │  │
 │  exports)    │   │  │  crate,    │ │ API      │ │  web push)     │  │
 └──────────────┘   │  │  ported)   │ └──────────┘ └────────────────┘  │
                    │  └────────────┘ ┌──────────┐ ┌────────────────┐  │
                    │                 │ search   │ │ admin/mod      │  │
                    │                 │ (PG FTS) │ │ + analytics    │  │
                    │                 └──────────┘ └────────────────┘  │
                    └──────┬──────────────────┬───────────────┬────────┘
                           │                  │               │
                 ┌─────────▼───────┐  ┌───────▼──────┐  ┌─────▼──────────────┐
                 │ PostgreSQL 16   │  │ Redis        │  │ Worker pool (same  │
                 │ (data + FTS +   │  │ (rate limit, │  │ binary, --worker   │
                 │  job queue)     │  │  cache,      │  │ mode): scrape,     │
                 └─────────────────┘  │  sessions)   │  │ convert, notify,   │
                                      └──────────────┘  │ sitemap, digests   │
                                                        └────────────────────┘
```

### 6.2 Key architectural decisions (and why they differ from today)

| Decision | Today | Greenfield | Rationale |
|---|---|---|---|
| Frontend rendering | `adapter-static` SPA | `adapter-node` SSR + hydration | SEO is the acquisition engine (P1). Every work page must be crawlable HTML with meta/OG/JSON-LD. |
| API ownership | Rust serves SPA + API from one binary | SvelteKit node server (SSR) proxies `/api/*` to Rust | Keeps SSR session handling and edge caching sane; Rust stays a pure JSON API |
| Ebook storage | filesystem cache dir | S3-compatible store (MinIO self-host / R2) | Statelessness (P4), CDN-able downloads, cheap replication |
| Job processing | ad-hoc tokio tasks + CLI bins | Postgres-backed queue (`pgmq`-style table with `SELECT ... FOR UPDATE SKIP LOCKED`) | One less daemon; visibility, retries, dead-letter — critical for scraping reliability |
| Ollama | hard dep for recs + Ask | optional, `FEATURE_LLM=off` default | Site must be fully useful with zero ML (P8) |
| Deployment | single box, systemd | 2 containers (web, worker) + managed or dockerized PG/Redis/S3 | Horizontal scaling path from day one (P4) |
| Migrations | consolidated 001 + patches | numbered incremental `NNN_*.sql`, never edited after merge | Standard practice; the current convention already does this — keep it |

### 6.3 Repository layout

```
fichub-next/
├── Cargo.toml                  # workspace
├── crates/
│   ├── web/                    # axum binary: API routes, auth, rate limits
│   ├── worker/                 # same lib, --worker mode: job consumers
│   ├── ingest/                 # ported scrapers/ + conversion pipeline
│   └── notifications/          # email/RSS/push senders
├── frontend/                   # SvelteKit 5, adapter-node, SSR
├── migrations/                 # NNN_*.sql, append-only
├── docs/plans/                 # this document lives here
└── deploy/                     # docker-compose.yml, systemd units, Caddyfile
```

### 6.4 Conventions for the junior dev

- **Every route module = one file** in `crates/web/src/routes/`, mirroring today's layout (it works).
- **Errors**: one `AppError` enum with `IntoResponse`, `thiserror`-derived, JSON body `{ error: { code, message } }`. Port `src/error.rs`.
- **All timestamps `TIMESTAMPTZ`**, all ids: `works.id BIGINT` (public-facing `url_id TEXT` unique, 10-char base62 — keep today's scheme, it's good for URLs).
- **Never block the request thread on scraping.** Scrape = enqueue job + return `202 Accepted` with a status URL (§9.3). Only exception: the synchronous URL→EPUB hook path with a 25s hard timeout.
- **Config = env vars only** (`dotenvy` in dev), one `Config` struct, port `src/config.rs` and prune it.

---
## 7. Data Model

PostgreSQL 16. Naming: plural snake_case tables, `TIMESTAMPTZ` everywhere, soft-delete only for works (`deleted_at`), everything else hard-delete (GDPR). Migrations are append-only `NNN_*.sql`.

### 7.1 Core: works, sources, chapters

```sql
-- ============ WORKS ============
-- One logical fic across all mirrors/sites. The #1 SEO + canonical entity.
CREATE TABLE works (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    url_id         TEXT UNIQUE NOT NULL,          -- 10-char base62, public (port from current build)
    slug           TEXT NOT NULL,                 -- "title-slug"; /fic/{url_id} 302s to /fic/{url_id}-{slug}
    title          TEXT NOT NULL,
    sort_title     TEXT GENERATED ALWAYS AS (NULLIF(regexp_replace(lower(title), '^[a-z]{0,3}\\s+', ''), '')) STORED,
    -- author is the *display* author string from the primary source; see work_authors for identity resolution
    author_name    TEXT NOT NULL,
    description    TEXT NOT NULL DEFAULT '',
    language       TEXT NOT NULL DEFAULT 'en',
    is_complete    BOOLEAN,
    n_chapters     INT,
    word_count     INT,
    published_at   TIMESTAMPTZ,
    updated_at_src TIMESTAMPTZ,   -- what the source site claims
    content_freshness TIMESTAMPTZ, -- last verified ingest (drives sitemap priority, §9.6)
    primary_kind   TEXT NOT NULL DEFAULT 'fanfic' CHECK (primary_kind IN ('fanfic','original')),
    status         TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','unlisted','removed')),
    deleted_at     TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX works_url_id_uq ON works(url_id);
CREATE INDEX works_slug_idx ON works(slug);
CREATE INDEX works_freshness_idx ON works(status, content_freshness DESC);

-- ============ SOURCES (mirror links) ============
-- Each place a work exists on the open web. Multi-source = the core of
-- metadata dedup + link-rot resilience.
CREATE TABLE work_sources (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    work_id      BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    site         TEXT NOT NULL,            -- 'ao3' | 'ffnet' | 'royalroad' | ... (scraper keys)
    source_url   TEXT NOT NULL,
    url_hash     TEXT NOT NULL,            -- sha256 of normalized URL, dedup key
    is_primary   BOOLEAN NOT NULL DEFAULT false,
    last_seen_at TIMESTAMPTZ,             -- last successful full check (§9.4)
    last_status  TEXT,                     -- 'ok' | 'rate_limited' | 'gone' | 'blocked' | ...
    last_error   TEXT,
    UNIQUE (url_hash)
);
CREATE INDEX work_sources_work_idx ON work_sources(work_id);

-- ============ CHAPTERS ============
CREATE TABLE chapters (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    work_id       BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    n             INT NOT NULL,             -- 1-based chapter number
    title         TEXT NOT NULL DEFAULT '',
    word_count    INT,
    published_at  TIMESTAMPTZ,
    body_path     TEXT,                     -- S3 key of cleaned HTML; NULL until fetched
    body_sha256   TEXT,
    UNIQUE (work_id, n)
);
CREATE INDEX chapters_work_idx ON chapters(work_id, n);

-- ============ TAGS ============
-- Port the current tag canon: tags + tag_aliases + fic_tags with role_confidence.
CREATE TABLE tags (
    id       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name     TEXT UNIQUE NOT NULL,          -- display name
    slug     TEXT UNIQUE NOT NULL,          -- URL: /tags/{slug}
    kind     TEXT NOT NULL DEFAULT 'freeform',
          -- freeform | fandom | character | relationship | warning | rating | category
    fandom_slug TEXT,                       -- when kind='fandom'
    usage_count INT NOT NULL DEFAULT 0,
    status   TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','pending','hidden')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE tag_aliases (
    alias TEXT PRIMARY KEY,
    tag_id BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE
);
CREATE TABLE work_tags (
    work_id BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    tag_id  BIGINT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    role_confidence REAL NOT NULL DEFAULT 1.0,  -- port of migration 004
    PRIMARY KEY (work_id, tag_id)
);
CREATE INDEX work_tags_tag_idx ON work_tags(tag_id);

-- ============ FANDOMS ============
CREATE TABLE fandoms (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name       TEXT NOT NULL,
    slug       TEXT UNIQUE NOT NULL,
    media_type TEXT NOT NULL DEFAULT 'other' CHECK (media_type IN ('anime','book','cartoon','comic','game','movie','tv','other')),
    work_count INT NOT NULL DEFAULT 0,
    description TEXT NOT NULL DEFAULT ''
);
CREATE TABLE work_fandoms (
    work_id   BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    fandom_id BIGINT NOT NULL REFERENCES fandoms(id) ON DELETE CASCADE,
    is_cross  BOOLEAN NOT NULL DEFAULT false,   -- crossover flag (§9.5)
    PRIMARY KEY (work_id, fandom_id)
);
```

### 7.2 Users, library, social

```sql
-- ============ USERS ============
-- One auth identity = one row (email+password or OAuth). Pseudonyms are
-- display-only (port pseuds idea, simplified).
CREATE TABLE users (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email         TEXT UNIQUE,               -- NULL for device-only accounts
    password_hash TEXT,                      -- bcrypt; NULL for OAuth-only
    display_name  TEXT NOT NULL,
    handle        TEXT UNIQUE NOT NULL,      -- @handle, used in public list URLs
    role          TEXT NOT NULL DEFAULT 'user' CHECK (role IN ('user','curator','moderator','admin')),
    trust_tier    SMALLINT NOT NULL DEFAULT 0 CHECK (trust_tier <= 2),  -- §13.3: 0=new, 1=established, 2=trusted
    email_verified_at TIMESTAMPTZ,
    is_banned     BOOLEAN NOT NULL DEFAULT false,
    ban_reason    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============ ANONYMOUS LIBRARY ============
-- The conversion mechanism (§3.2): anyone can bookmark/follow before signup.
-- A signed, 400-day device cookie identifies the row. On signup, merge.
CREATE TABLE devices (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    token_hash  TEXT UNIQUE NOT NULL,        -- sha256 of cookie value
    user_id     BIGINT REFERENCES users(id), -- set when claimed at signup
    user_agent  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE bookmarks (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id     BIGINT REFERENCES users(id) ON DELETE CASCADE,
    device_id   BIGINT REFERENCES devices(id) ON DELETE CASCADE,
    work_id     BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'reading' CHECK (status IN ('reading','read','later','dNF')),
    note        TEXT,
    is_private  BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (num_nonnulls(user_id, device_id) = 1)  -- exactly one owner
);
CREATE UNIQUE INDEX bookmarks_owner_work_uq ON bookmarks(COALESCE(user_id,0), COALESCE(device_id,0), work_id);

CREATE TABLE follows (
    id        BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id   BIGINT REFERENCES users(id) ON DELETE CASCADE,
    device_id BIGINT REFERENCES devices(id) ON DELETE CASCADE,
    work_id   BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    channel   TEXT NOT NULL DEFAULT 'inbox' CHECK (channel IN ('inbox','email','push','rss')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (num_nonnulls(user_id, device_id) = 1)
);
CREATE UNIQUE INDEX follows_owner_work_uq ON follows(COALESCE(user_id,0), COALESCE(device_id,0), work_id);

-- Reading position (sync across devices once claimed by a user)
CREATE TABLE reading_progress (
    owner_type   TEXT NOT NULL CHECK (owner_type IN ('user','device')),
    owner_id     BIGINT NOT NULL,
    work_id      BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    chapter_n    INT NOT NULL,
    scroll_pct   REAL NOT NULL DEFAULT 0,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (owner_type, owner_id, work_id)
);

-- ============ LISTS (curated rec lists — the curator persona's SEO engine) ============
CREATE TABLE lists (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    owner_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    slug        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_public   BOOLEAN NOT NULL DEFAULT false,
    work_count  INT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (owner_id, slug)
);
CREATE TABLE list_items (
    list_id  BIGINT NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    work_id  BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    blurb    TEXT,                          -- one-line "why you should read this"
    position INT NOT NULL,
    PRIMARY KEY (list_id, work_id)
);

-- ============ REVIEWS / RATINGS (minimal social proof) ============
CREATE TABLE reviews (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    work_id    BIGINT NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating     SMALLINT CHECK (rating BETWEEN 1 AND 5),
    body       TEXT NOT NULL DEFAULT '',
    status     TEXT NOT NULL DEFAULT 'visible' CHECK (status IN ('visible','hidden','pending')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (work_id, user_id)
);
CREATE INDEX reviews_work_idx ON reviews(work_id) WHERE status = 'visible';

-- ============ UPLOADS (native / original works — the legal pillar, §14) ============
CREATE TABLE uploads (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    work_id     BIGINT UNIQUE NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    uploader_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    license     TEXT NOT NULL DEFAULT 'all-rights-reserved',
          -- all-rights-reserved | cc-by | cc-by-nc | cc-by-nc-sa | cc0
    author_signed_attribution BOOLEAN NOT NULL DEFAULT true,  -- "I am the author"
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============ JOBS (Postgres-backed queue — no extra daemon) ============
CREATE TABLE jobs (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    kind        TEXT NOT NULL,
         -- 'ingest' | 'refresh' | 'convert' | 'notify' | 'sitemap' | 'health_check' | 'digest'
    payload     JSONB NOT NULL,
    status      TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued','running','done','failed','dead')),
    priority    INT NOT NULL DEFAULT 100,       -- lower runs first; user-triggered = 10
    run_after   TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked_by   TEXT,
    locked_at   TIMESTAMPTZ,
    attempts    INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    last_error  TEXT,
    dedupe_key  TEXT UNIQUE,                    -- e.g. 'refresh:src:1234' prevents stampedes
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);
CREATE INDEX jobs_pick_idx ON jobs(status, priority, run_after) WHERE status IN ('queued','running');

-- ============ ANALYTICS (zero PII — port the current principle) ============
CREATE TABLE analytics_events (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    day        DATE NOT NULL,                   -- pre-aggregated daily grain; no raw rows kept > 30d
    event      TEXT NOT NULL,
        -- 'page_view_work' | 'download' | 'paste_ingest' | 'signup' | 'return_visit'
        -- | 'follow_created' | 'notify_sent' | 'notify_click' | 'list_view' | 'rec_click'
    count      BIGINT NOT NULL DEFAULT 1,
    dim1       TEXT,                            -- first dimension, e.g. source site, referrer class
    dim2       TEXT,
    PRIMARY KEY (day, event, dim1, dim2)
);
```

> **Deferred tables** (not in v1, see §4): forums, collections, shelves, bounties, quests, recipes, marketplace, progression, modlog, reports/trust beyond `trust_tier`, DMCA notices (added in M4 — see §14.3), push_subscriptions (added with web push in M2).

---

## 8. API Specification

All endpoints JSON, prefix `/api/v1/` (the current build's unversioned `/api/*` is a migration liability — version from day one). Auth: `Authorization: Bearer <jwt>` (access 15 min + refresh 30 d, port `jsonwebtoken` + bcrypt setup) **or** the device cookie (`fh_dev`, HttpOnly, Signed, 400 d) for the anonymous library. Rate limits: Redis token bucket (port `src/limiter/`), anonymous 30 req/min, authed 120 req/min, ingest 10/min.

### 8.1 Ingest & works

| Method | Path | Purpose | Notes |
|---|---|---|---|
| POST | `/api/v1/ingest` | paste a URL → work | Body `{ url }`. If URL known → `200 { work }`. Else `202 { job_id, status_url }`; poll status URL. Sync path (bookmarklet/hook) uses `?sync=1` with 25 s cap |
| GET | `/api/v1/works/{url_id}` | full work detail | Includes sources, tags, fandoms, stats, `my_bookmark`, `my_follow` (owner-scoped) |
| GET | `/api/v1/works/{url_id}/chapters` | chapter list | Body text NOT included (fetched per chapter, §9.5) |
| GET | `/api/v1/works/{url_id}/chapters/{n}` | chapter content | Cleaned HTML + prev/next; records progress if `?progress=1` |
| POST | `/api/v1/works/{url_id}/refresh` | re-check source | Enqueues `refresh` job (dedupe 10 min); `202` |
| GET | `/api/v1/search` | search | `q` + `sort=popular\|new\|updated\|words` + `page`; PG FTS + trigram (§9.5) |
| GET | `/api/v1/works/{url_id}/related` | related works | v1 = tag-overlap score (§12); cached 1 h |
| GET | `/api/v1/blind` | random pick | Powers Blind Date; cheap and fun, keep it |

### 8.2 Library & reader

| Method | Path | Purpose | Notes |
|---|---|---|---|
| GET/POST | `/api/v1/me/bookmarks` | list / add | Owner = user JWT **or** device cookie. POST body `{ url_id, status? }` |
| DELETE | `/api/v1/me/bookmarks/{work_id}` | remove | — |
| GET/POST/DELETE | `/api/v1/me/follows` | follow mgmt | POST body `{ url_id, channel? }` |
| PUT | `/api/v1/me/progress/{url_id}` | save position | `{ chapter_n, scroll_pct }`; fire-and-forget from reader |
| GET | `/api/v1/me/updates` | inbox feed | Followed works with `last_chapter_seen < chapters.n_chapters`, newest first |
| GET/POST/DELETE | `/api/v1/lists` (+`/{id}/items`) | lists | Public lists get SSR pages (§10) |
| GET | `/opds` | OPDS 1.2 catalog | Port `src/routes/opds/`; e-reader integration for free |

### 8.3 Notifications

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/v1/me/notifications` | inbox list (unread first) |
| POST | `/api/v1/me/notifications/read` | mark read `{ ids }` |
| PUT | `/api/v1/me/settings/notifications` | per-channel toggles + digest frequency |
| GET | `/feeds/updates/{token}.xml` | per-user RSS/Atom of followed-work updates (token = revocable) |

### 8.4 Auth & account

| Method | Path | Purpose |
|---|---|---|
| POST | `/api/v1/auth/register` | email + password (or OAuth code). **Claim device**: if `fh_dev` cookie present, merge its bookmarks/follows/progress into the new user in one transaction (§13.1) |
| POST | `/api/v1/auth/login` / `/refresh` / `GET /me` | standard JWT pair |
| POST | `/api/v1/auth/oauth/{provider}` | Google + Discord (§13.1) |
| PUT | `/api/v1/me/profile` | display name, handle, notification prefs |
| POST | `/api/v1/me/export` | GDPR export: enqueue job → zip of bookmarks/follows/lists/reviews (port `user_export.rs` idea) |
| DELETE | `/api/v1/me` | GDPR delete: hard-delete user + anonymize reviews |

### 8.5 Errors

```json
{ "error": { "code": "scraper_unsupported", "message": "No adapter for this site yet. Request it: /suggest-site" } }
```

Standard codes: `invalid_url`, `scraper_unsupported`, `scraper_timeout`, `rate_limited`, `unauthorized`, `forbidden`, `not_found`, `gone` (source dead, work still cached), `validation`. Every 5xx gets a `trace_id` for support.

---
## 9. Backend Subsystems

### 9.1 The scraper crate (PORT — do not rewrite)

The `scrapers/` crate (107 site adapters, FanFicFare parity, `SiteScraper` trait with `lookup` / `fetch_chapters` / `lookup_from_html` seams, `Registry` with specific-vs-fallback dispatch) is the project's crown jewel. **Port it as `crates/ingest/` with these changes only:**

1. Keep the trait and registry as-is (the `lookup_from_html` seam added for Wayback is exactly right — keep it).
2. Replace the FanFicFare *subprocess* catch-all with an optional sidecar container (still no Python CLI dependency for the top-20 native sites; the catch-all runs in the sidecar so a hung subprocess can't take down the API).
3. Add a per-site **politeness profile** table (env or DB): `{ min_delay_ms, max_concurrency, robots_respected }`. Default 1 req / 2 s / concurrency 2 per domain. This is non-negotiable (§14.2).
4. Keep domain-health telemetry (port `src/heal/` ideas): every failure increments `work_sources.last_status`; a circuit breaker skips domains failing > 80% over the last hour.

### 9.2 Ingest pipeline (new code, straightforward)

```
POST /api/v1/ingest { url }
  ├─ normalize URL (strip tracking params, lowercase host) → url_hash
  ├─ SELECT work via url_hash → hit? return 200 { work }
  ├─ miss? enqueue job 'ingest' { url_hash, url } (priority 10, dedupe_key = url_hash)
  └─ return 202 { job_id, status_url: /api/v1/jobs/{id} }

worker: 'ingest'
  ├─ registry.find_specific_or_fallback(url) → scrape metadata (+ chapters for WIP follow)
  ├─ fetch + clean chapter bodies (§9.5 cleaning rules) → S3
  ├─ find_or_create_work():
  │    exact title+author (case-insens) AND word_count within ±5% → merge into existing work
  │    else create work + source row
  ├─ extract tags/fandoms (port src/tags/ extraction + auto-tag review flow, simplified: curator queue only)
  └─ emit events: 'paste_ingest', analytics; enqueue 'convert' (§9.3)
```

`GET /api/v1/jobs/{id}` returns `{ status, work_url_id? }` — the frontend polls every 2 s with exponential backoff and renders a progress state ("talking to ao3… fetching 34 chapters… building EPUB").

### 9.3 Conversion & ebook generation (PORT + keep)

- EPUB: port `epub-builder` pipeline as-is; **add `calibre:`-style metadata `FicHub: url_id + download URL` on every generated file** (viral loop, §10.5).
- TXT/MD/HTML: keep current pure-Rust paths.
- MOBI/PDF/AZW3: Calibre sidecar container, invoked as a job (never in-request). Cache output in S3 keyed by `(work_id, format, content_hash)`.
- Filenames: `Title - Author [fichub-urlid].epub` (port `sanitize-filename` usage).

### 9.4 Update detection & refresh (the retention engine)

The single most valuable recurring job. Runs as `refresh` jobs scheduled by a scheduler loop (worker tick every minute scans `work_sources`): 

- **Cadence by heat**: works followed by ≥1 user → every 6 h; popular (≥100 follows) → hourly; unfollowed → weekly; `last_status = 'gone'` → monthly. Store next-due in `work_sources.run_after` (reuse the jobs table's `run_after`).
- **Detection**: compare `n_chapters` + `updated_at_src` + latest chapter `word_count`. New chapter → clean body → S3 → update `works.updated_at_src`, bump `content_freshness`.
- **Fan-out**: enqueue `notify` jobs per follower (batched: one digest per user per 6 h by default, instant for `channel='push'` opt-ins). `notify` renders email/RSS/push and writes `analytics_events('notify_sent')`.
- **Never notify twice**: `notifications` table keyed `(follow_id, chapter_id)` unique.

### 9.5 Search & content cleaning

- **Metadata search** (public): PG `tsvector` on title+description+author, `pg_trgm` for typo tolerance, filters: fandom, tag (AND/OR/NOT), rating, complete, word-count range, language, source site. **Do not port the full boolean v2 grammar for v1** — ship `q` + facets first; port `src/search/parser.rs` when power users ask (it's good code, it's just not day-one).
- **Body cleaning** (on ingest, port + tighten): strip scripts/styles/iframes, rewrite relative→absolute URLs, keep only an allowlist of inline tags (`p, em, strong, br, hr, img, blockquote, ul/ol/li, h1-h6, a[rel=nofollow]`), sanitize with an allowlist parser, store cleaned HTML in S3. Store `body_sha256` for dedup.
- **Body search**: only over works in the requesting user's own library (their cached copies). Full-corpus body search is §4-cut pending legal review.

### 9.6 Sitemaps & robots

Worker job `sitemap` (daily, or when new works > threshold):
- `/sitemaps/works-{shard}.xml` — sharded at 40k URLs, from `works(status='active')` ordered by `content_freshness DESC` (fresh WIPs get higher priority).
- `/sitemaps/tags-{shard}.xml`, `/sitemaps/fandoms.xml`, `/sitemaps/lists.xml` (public lists only).
- `/sitemap.xml` — index file. Register in robots.txt + Search Console.
- `robots.txt`: allow everything public, disallow `/api/`, `/me/`, `/opds/`.

### 9.7 Notifications subsystem (new `crates/notifications/`)

| Channel | Transport | Notes |
|---|---|---|
| Inbox | DB rows + `/api/v1/me/updates` | default for everyone |
| Email | `lettre` + SMTP provider | verify address first; RFC-compliant `List-Unsubscribe` headers (CAN-SPAM/GDPR requirement) |
| RSS/Atom | per-user token feed (§8.3) | revocable, no auth needed on feed URL |
| Web push | VAPID + `web-push` crate | M2; prompt only after user follows ≥ 2 works (don't burn the permission) |

Digest batching: a `digest` job per user merges all pending notifications into one email per chosen frequency (instant / daily / weekly). Default **daily** — the single biggest unsubscribe-prevention lever.

---
## 10. Frontend & SEO

SvelteKit 5, `adapter-node`, TypeScript. **SSR for all public pages; islands-free hydration is fine.** No CSS framework mandate; use the existing design tokens approach (`docs/theme/`, CSS custom properties) with 3 presets: dark (default), light, sepia — accessibility presets ship in M4.

### 10.1 Route map

| Route | Rendering | Purpose |
|---|---|---|
| `/` | SSR | The input box + trending works (cache 5 min). If a URL is pasted, server-side redirect to the ingest flow |
| `/fic/{url_id}-{slug}` | **SSR + caching** | THE money page. 301 from `/fic/{url_id}` slugless |
| `/fic/{url_id}/read` | CSR (client reader) | Reader is an app, not a page — but see §11.1 for the no-JS fallback |
| `/fic/{url_id}/download` | SSR | Format chooser + QR code (phone→desktop handoff) + embed code for the share card |
| `/author/{name-slug}` | SSR | Author page: works, series, source links (all mirrors) |
| `/fandom/{slug}` | SSR | Fandom landing: top works, popular tags, recent updates |
| `/tags/{slug}` | SSR | Tag pages = long-tail SEO gold ("harry potter time travel fanfic") |
| `/lists/{handle}/{slug}` | SSR | Public curated lists (curator persona) |
| `/search` | SSR (results) | Server-rendered results, shareable query URLs |
| `/library`, `/follows`, `/updates` | CSR | Private app pages behind auth/device cookie |
| `/suggest-site` | SSR form | Capture demand for unsupported hosts — feed the roadmap with real data |

### 10.2 The work page — SEO checklist (the money page)

Every `/fic/*` render MUST include, server-side:

- [ ] `<title>{title} - {author} | FicHub</title>` (≤ 60 chars where possible)
- [ ] `meta[name=description]` = first 160 chars of description
- [ ] `link[rel=canonical]` → `https://fichub.example/fic/{url_id}-{slug}`
- [ ] OG + Twitter cards (`og:title`, `og:description`, `og:image` = generated share card, §10.5)
- [ ] JSON-LD `BookArticle`/`Article` schema (title, author, wordCount, datePublished, inLanguage, genre from tags)
- [ ] Server-rendered: title, author, word count, chapters, status, tags (first 10), description, 3-chapter preview, download buttons, related works
- [ ] Internal links: author page, fandom page, top 5 tags, 5 related works — every work page links to ≥ 5 other indexable pages
- [ ] Download buttons render as real `<a href>` links (crawler-visible) to `/cache/{format}/{url_id}/{fname}` (port current cache route design)

### 10.3 Sitemap/robots (see §9.6) and Search Console verification in M1.

### 10.4 PWA & mobile

- Mobile-first CSS from M0 — 60% of fic reading is mobile (persona §5).
- Web app manifest + service worker (port `/sw.js` idea from SPECIFICATION.md): precache app shell, cache-first for chapter bodies (installed users get offline reading), stale-while-revalidate for metadata.
- Installable (A2HS prompt after 2nd visit), iOS-safe (apple-touch-icon, safe-area insets).

### 10.5 Share loops (the free growth channels)

1. **Generated share cards**: worker job renders an SVG→PNG OG image per work (title, author, word count, FicHub URL, cover if source provides one). Served at `/og/{url_id}.png`, cached forever, used in og:image.
2. **EPUB metadata stamp**: every generated EPUB embeds `FicHub` as contributor + the work URL in `dc:source` (§9.3). Files travel; each is an ad.
3. **Bookmarklet + browser extension**: current build's bookmarklet idea, keep: right-click → "Save to FicHub" on any supported fic page. Extension = same ingest API + auth token.
4. **List embeds**: `<iframe src="/embed/list/{id}">` widget for curators' blogs/Discord servers — server-rendered, no JS needed.
5. **r/FanFiction / Discord integrations**: read-only flair + `/suggest-site`; outbound announcements only after M2 (see §19, M3 launch plan).

---
## 11. Reader, Library & Download UX

### 11.1 The reader (`/fic/{url_id}/read`)

Client-rendered app; the state machine is:

```
load chapter n → render sanitized HTML → hydrate reader chrome
  ├─ typography panel: font (serif/sans/dyslexic), size, line height, width, theme (dark/light/sepia)
  ├─ position save: PUT /me/progress/{url_id} debounce 2s + on chapter change + on visibilitychange
  ├─ next/prev: prefetch chapter n+1 body when scroll_pct > 70%
  ├─ end-of-chapter: "Next chapter" → on last chapter: "Follow this fic to get notified" (conversion point!)
  └─ keyboard: ←/→ chapters, j/k scroll, f focus mode
```

**No-JS fallback**: `/fic/{url_id}/read?static=1` serves plain server-rendered chapter HTML with a link to the next chapter. Crawlers and e-reader browsers get content; JS users get the app.

Reader preferences persist per device pre-signup, per user post-signup (merge at claim, §13.1).

### 11.2 Follows & the updates inbox (retention engine)

- Follow button on work page + end-of-chapter + search results (hover card).
- Follow with `channel` chosen at follow-time: inbox (default), email, push, RSS.
- `/updates` page = reverse-chron inbox of new chapters across all followed works, with "resume where you left off" deep links (`/fic/x/read?ch=12&from=notif`).
- Update emails are digests by default (§9.7); each email shows 2–3 other updated followed fics + one rec (retention compounding).

### 11.3 Library, lists, downloads

- `/library`: bookmarked works grouped by status (reading / read / later), each row shows progress ("ch 14/39"), last-update, follow state. Filter + sort. This is the home screen for returning users.
- **Downloads**: format chips (EPUB default), "include images" toggle, per-work or batch (zip of a list). Every download event → analytics. Downloads never require auth (§3.2 rule).
- **Lists** (§7.2): public lists get SSR pages + OG cards + embeds. Curators get follower counts on lists in M3.

### 11.4 The paste flow (first-run magic moment)

1. User pastes `https://archiveofourown.org/works/123456` into the homepage box.
2. Instant client-side feedback: known-site badge appears while `POST /ingest` resolves.
3. Known work → redirect to `/fic/{url_id}-{slug}` (≤ 1.5 s target).
4. New work → progress panel with per-stage messages (§9.2), typically ≤ 25 s for a 40-chapter fic; then the work page.
5. Success state ends with two big buttons: **Read now** / **Download EPUB** — and a subtle "Save to your library" (device cookie, no signup wall).

### 11.5 OPDS (keep, cheap)

Port `src/routes/opds/` as-is: `/opds` root catalog → by-fandom → by-tag → per-work with acquisition links to cached files. This makes FicHub a first-class source in KOReader/Moon+/Calibre — an outsized niche channel for near-zero code.

---

## 12. Discovery & Recommendations v1

**Zero ML in v1.** Three deterministic strategies, each simple enough to debug with `EXPLAIN ANALYZE`:

1. **Tag-overlap related** (`/works/{id}/related`): score = Σ(tag weights of shared tags) normalized by sqrt(|tags_a|·|tags_b|), same-fandom boost ×1.5, exclude same author. Cache 1 h in Redis. This is the current build's `legacy_cooccur` reduced to its core — port `src/recommender/legacy_cooccur.rs` and strip the decay machinery.
2. **Also-bookmarked**: users who bookmarked X also bookmarked Y (co-occurrence over bookmarks, min 3 co-occurrences). Port the `work_feedback_signals` idea without the ML plumbing.
3. **Trending**: downloads + reads + follows in last 7 d with exponential time decay, per-fandom normalization. Port `src/routes/trending.rs` logic.

Presentation: one "Readers also enjoyed" row on the work page, one "Trending in {fandom}" on fandom pages, one rec slot in update digests. Every rec click → `analytics_events('rec_click', dim1=strategy)` — the data that later justifies (or kills) the fancy engine (§4 gate).

### 12.4 Ask-the-Archive (flag-gated, M4+)

Port `src/search/ask.rs` behind `FEATURE_LLM=1`: natural-language → the v1 filter set → normal search. The site never depends on Ollama being up (graceful fallback = plain search, current build already handles this — keep that behavior).

---
## 13. Accounts, Moderation & Trust

### 13.1 Auth (open, low-friction)

- Email+password (bcrypt) **and** OAuth (Google, Discord — where fic readers already are). No invite codes, no applications, no PoW at the door (P7 fix). PoW only fires on abuse signals (§15).
- **Device claim**: registration, when a `fh_dev` cookie exists, merges that device's bookmarks, follows, progress, and reader prefs into the new account — in one transaction. This is the single highest-leverage conversion code in the system; write a test for it before anything else in auth.
- Email verification required only before enabling `email` notifications (not before general use).

### 13.2 Moderation (small and boring beats large and clever)

- `reports` table + `POST /api/v1/reports` on works/reviews/users (port the current reports shape, drop trust-weighting). 
- `role='moderator'` sees a queue; one-click: dismiss / hide / ban. Every action → modlog row (public modlog ported from current build — it's a differentiator and costs little).
- Auto-triage, simplified from the current build: ≥ 3 independent reports → auto-hide pending review. No LLM moderation in v1.
- DMCA flow: §14.3.

### 13.3 Trust (3 tiers, not 7)

| Tier | Granted when | Powers |
|---|---|---|
| 0 — New | signup | read, bookmark, follow, review (rate-limited) |
| 1 — Established | 7 days old AND 5+ distinct-day activity | public lists, embed widgets |
| 2 — Trusted | manual grant by moderator | report weight ×3, review without rate limit, candidate for moderator |

Everything else from the current 7-level system (auto-promotions, weekly digest, participation scoring) is cut (§4). Revisit at 10k users.

---

## 14. Legal & Platform Risk

> **Read this section before writing any ingest code.** Popularity is capped instantly if the site gets a hosting provider takedown or is hated by the fic-writing community.

### 14.1 Positioning: reader's companion, not a pirate archive

- Import = caching + formatting for personal reading, with **full attribution and canonical source links always visible** on every work page ("Source: AO3 — original work by {author}").
- **Native hosting for original work from M1** (`uploads` table, §7.2): authors can post directly, choose a license, and get a canonical profile page. This changes the legal story from "we mirror your content" to "we host + index, with takedown respect" — and creates rankable original content.
- Never strip author names from exports (enforce in the EPUB builder — test it).

### 14.2 Politeness as policy

- Per-domain politeness profiles (§9.1), robots.txt respected by default, identifiable User-Agent `FicHubBot/1.0 (+https://fichub.example/bot)` with an opt-out URL per site that we honor.
- Rate-limit circuit breakers per domain; a domain that blocks us gets marked `blocked` and we stop hammering (the current build's self-healing telemetry becomes the enforcement mechanism).
- Published policy page `/bot` describing all of the above.

### 14.3 DMCA / takedown (build in M1, not "later")

- `/copyright` page (port current) with a web form → `dmca_notices` table (port migration 068 shape).
- SLA < 24 h: notice → set `works.status='removed'` (tombstone: "removed due to rights holder request"), purge S3 bodies, log the action. Repeat-infringer account policy in ToS.
- Counter-notice flow documented; keep it manual at this scale.

### 14.4 GDPR basics from day one

- `user_export` (zip) + account self-delete (§8.4).
- Zero-PII analytics (§7.2); no third-party trackers; email addresses encrypted at rest is optional, hashed device tokens mandatory.
- Privacy policy page in plain language; cookie banner = none (one functional cookie + one analytics-free local pref store = no consent requirement under most readings; get the policy right).

### 14.5 ToS / community rules

Short ToS: personal use of downloads, no bulk scraping of FicHub itself, attribution preserved, takedown rights reserved. Port the current `docs/src` legal pages as a starting point.

---
## 15. Anti-Bot & Abuse

Keep the current build's good ideas, in escalating-tiers form:

| Tier | Trigger | Mechanism |
|---|---|---|
| 0 — always | every request | Redis token bucket (port `src/limiter/`), Cloudflare bot fight mode, honeypot fields on all public forms (port `routes/honeypot.rs`) |
| 1 — suspicion | > N ingests/min/IP, 404 farms | PoW challenge (port `routes/pow.rs` — it's good, just not at the front door), progressive delays |
| 2 — confirmed abuse | PoW failures, scraper-farm fingerprints | shadowban (port Redis shadowban idea: 200 OK with garbage/empty results), IP-range block |
| 3 — scraped-at-scale | bulk download farms | per-device download quota (e.g. 200/day anonymous), CAPTCHA on the specific endpoint, not the whole site |

The hourly bot-scorer (port `bot_scorer` bin idea) runs as a worker job; shadowban list lives in Redis with TTL. **Key principle: friction scales with abuse, never with newness.**

---

## 16. Observability, QA & Testing

### 16.1 Observability

- `tracing` JSON logs with `trace_id` on every request (port current setup).
- Prometheus via `axum-prometheus` (port); Grafana dashboard: p95 latency per route, ingest success rate per domain, queue depth, refresh staleness p95, notification SLA.
- **Alerts that matter**: ingest success < 70% over 1 h for any top-10 domain; queue depth > 1,000; p95 work-page render > 800 ms; notify job failures.

### 16.2 The QA harness (PORT)

Port `qa/run.js` (Playwright + deterministic checks, zero-LLM) as the pre-deploy gate: API walk, work-page SSR content assertions (title/tag/JSON-LD present), reader nav, download round-trip, OPDS validity, RSS validity.

### 16.3 Test strategy (what a junior dev writes)

| Layer | Tool | What to cover |
|---|---|---|
| Scraper fixtures | `cargo test` | replay HTML fixtures per site (`lookup_from_html` makes this trivial — that's why it exists); golden FicMetadata JSON |
| API | `axum-test` | every route: 200/401/403/404 + auth matrix incl. device-cookie flows |
| Unit | `cargo test` | ingest dedup, merge heuristics, cleaning allowlist, politeness governor |
| Frontend | vitest + Testing Library | reader state machine, library UI, SEO tags in SSR output (`+page.ts` load + render) |
| E2E | Playwright | paste→read, paste→download, follow→notify (fake clock), signup device-merge |
| Contract | port `api_contract_tests.rs` | JSON shapes stay stable |

CI gate: `cargo test && npm test && node qa/run.js` green before deploy. Coverage floors: routes 70%+, ingest 80%+.

---

## 17. Deployment & Scaling

### 17.1 Topology

```
Cloudflare (DNS, CDN, WAF, bot fight)
  └─ Caddy/nginx → web container (adapter-node) → API container (axum)
                                       └→ worker container (same binary, --worker)
PostgreSQL 16 (docker, then managed)  ·  Redis (docker)  ·  MinIO (docker, then R2)
```

Everything reproducible: `deploy/docker-compose.yml` for dev + small prod; systemd units for the ThinkCentre-style single box if preferred (port `scripts/install_systemd_timers.sh` idea → worker replaces timers).

### 17.2 Scaling order (do not skip ahead)

1. **Vertical**: current single box handles ~10k MAU comfortably at these budgets (§18).
2. **Read replicas**: PG read replica for search + public pages when p95 > 500 ms.
3. **Object store off-box**: S3/MinIO → R2 when egress > 100 GB/mo (Cloudflare R2 = no egress fees — downloads are the bandwidth hog).
4. **Workers split**: convert jobs (CPU) and notify jobs (I/O) into separate worker deployments when queue depth alerts fire.
5. **PGTM**: Postgres continues to do FTS + queue + analytics through ~100k MAU; beyond that, port search to a dedicated engine (current build's pgvector/TSV stack is a good base).

### 17.3 Backups & DR

- nightly `pg_dump` → S3, 30-day retention; weekly restore drill (documented, timed).
- S3 versioning on bucket (protects chapter bodies); infra as code (compose files in git).

---

## 18. Performance Budgets

Enforced in CI (Lighthouse CI + `cargo test` timing assertions):

| Path | Budget (p75 mobile) | Notes |
|---|---|---|
| `/fic/{id}-{slug}` SSR | TTFB < 400 ms, LCP < 2.0 s, CLS < 0.1 | cache 5 min; `Cache-Control: s-maxage=300, stale-while-revalidate=3600` |
| `/fic/{id}/read` | TTI < 2.5 s | chapter body prefetch at 70% scroll |
| URL→EPUB (sync hook) | < 10 s p75, 25 s cap | timeout → async fallback + email-when-ready |
| Ingest (async) | < 30 s p75 for 40-chapter fic | politeness delays dominate; show progress |
| Download | first byte < 300 ms from CDN | S3 → Cloudflare in M2 |
| Search | < 300 ms p95 | PG FTS + trigram, `LIMIT 20` + keyset pagination |

Budget violations fail CI the same way test failures do.

---
## 19. Milestones

Each milestone has an acceptance gate. **Do not start the next milestone until the gate is met** — this rule is the whole point of the rebuild.

### M0 — Skeleton (week 1)
Workspace + containers + SSR frontend hello-world + migrations runner + CI (test/QA/build) + deploy script to one box.
**Gate**: `deploy` produces a reachable site; CI green; a fresh clone → running site in ≤ 30 min.

### M1 — The hook (weeks 2–5) — *value without an account*
Port scrapers crate; ingest pipeline + job queue + worker; work/chapter/search pages SSR'd with full §10.2 SEO checklist; downloads (EPUB/TXT/HTML first, Calibre sidecar after); sitemaps + Search Console; paste flow magic moment (§11.4); DMCA + /copyright + /bot pages (§14); analytics_events.
**Gate**: paste-URL → EPUB in < 25 s p75; work pages indexed (submit sitemap); qa/run.js passes; hook conversion ≥ 40% in dogfood.

### M2 — The library (weeks 6–9) — *reasons to return*
Device cookies + anonymous library; auth (email + OAuth) + device-claim merge; reader (typography, progress sync, prefetch); follows + update detection cadence (§9.4) + notifications (inbox/email/RSS); PWA + offline chapters; bookmarks/lists (private); OPDS port.
**Gate**: D7 retention ≥ 15% in dogfood; notify end-to-end test (fake chapter bump → email arrives) green; Lighthouse mobile ≥ 90 on work + reader.

### M3 — The loops (weeks 10–13) — *reasons to share*
Public lists + embeds; OG share cards (§10.5); recs v1 (§12); trending + fandom/tag landing pages; suggest-site flow; native uploads (§14.1); moderation queue + reports; share-card EPUB stamps.
**Gate**: organic share of traffic ≥ 20%; ≥ 5 public lists created by dogfooders; zero P0 moderation incidents in the first 2 weeks.

### M4 — Reach (ongoing after launch)
LLM Ask behind flag; accessibility presets (dyslexia/high-contrast); i18n re-evaluation; rec CTR review against §12 gate; performance budget hardening; web push for follows; Calibre formats (MOBI/AZW3/PDF) if demand data supports it.
**Gate**: WAR ≥ 1,000 before any M4 item starts.

### M5 — Re-admitting the cut features (only behind §4 gates)
Progression, marketplace, forum, requests, trust ladder v2 — each with its own §4 gate, its own metric, and a rollout plan. Default answer: no.

---

## 20. Junior-Dev Implementation Guide

### 20.1 Order of implementation (do not reorder)

1. **Migrations 001–004**: works/sources/chapters/tags/fandoms (§7.1). Seed 3 fandoms + 10 tags.
2. **Config + error + tracing** (port `src/config.rs`, `src/error.rs`, prune to §22 list).
3. **`GET /works/{url_id}` + `GET /search`** backed by fixtures (fake `FicMetadata` JSON) — frontend SSR pages can build in parallel against a mocked API.
4. **Jobs table + worker loop** (`FOR UPDATE SKIP LOCKED`, §7.2). Test: enqueue 100 ingest jobs → all complete, no double-run.
5. **Port the scrapers crate** (§9.1). Start with AO3, FFN, RoyalRoad, Wattpad (biggest demand), then the rest of the top 20; the FanFicFare sidecar covers the tail.
6. **Ingest pipeline** (§9.2) with cleaning (§9.5) + S3 bodies.
7. **Downloads** EPUB/TXT/HTML + cache routes (§9.3).
8. **SSR SEO** (§10.2 checklist — write it as an automated test, not a code review item).
9. **Sitemaps + robots** (§9.6).
10. **Devices + bookmarks + follows + progress** (§8.2) — anonymous-first.
11. **Auth + device-claim merge** (§13.1) — with the merge test written first.
12. **Reader** (§11.1) + PWA (§10.4).
13. **Update detection + notifications** (§9.4, §9.7).
14. **Recs v1 + trending** (§12).
15. **Lists + OG cards + embeds** (§10.5).
16. **Moderation + reports + DMCA form** (§13.2, §14.3).
17. **OPDS port** (§11.5).
18. **QA harness port + Lighthouse CI budgets** (§16).

### 20.2 Rules of engagement

- One PR = one route or one migration group; PR template includes the §1.1 metric question ("n/a" allowed in M0–M1).
- No feature is "done" without: tests (§16.3 row), an analytics event if user-visible, and a docs line in `docs/plans/` (house convention).
- Never edit a merged migration; append a new one.
- Never put scraping, conversion, or email sending inside an HTTP request handler. If it takes > 100 ms, it's a job.
- Secrets only via env; add every new var to §22 and `.env.example` in the same PR.
- When stuck > 45 min: write a failing test that isolates the confusion; if the test reveals a spec gap, fix this document (PR to `docs/plans/`).

### 20.3 First-week warmup tasks (real, shippable)

| # | Task | Teaches |
|---|---|---|
| 1 | Add `analytics_events` write + `/api/v1/admin/analytics/summary` | sqlx, routes, the event schema |
| 2 | Implement `GET /api/v1/blind` (random active work, cache 60 s) | queries, Redis cache |
| 3 | Add the `/suggest-site` form + table + admin list | forms, moderation basics |
| 4 | Write the §10.2 SEO checker as a test against a fixture work page | SSR internals, test infra |
| 5 | Add `List-Unsubscribe` headers + digest batching test to notifications | §9.7, email compliance |

---
## 21. Appendix A — Porting Map from the Current Repo

Port (adapt lightly, keep tests): these are assets, not rewrites.

| Current location | Becomes | Effort | Notes |
|---|---|---|---|
| `scrapers/` (107 adapters, trait + registry, `lookup_from_html` seam) | `crates/ingest/` | low | the crown jewel; §9.1 changes only |
| `epub-builder` pipeline, TXT/MD/HTML exports | `crates/ingest/convert` | low | add FicHub metadata stamp (§10.5) |
| `src/limiter/` + Redis token bucket | same | low | |
| `src/routes/pow.rs`, `honeypot.rs`, shadowban | anti-abuse tiers (§15) | low | moved behind signals |
| `src/error.rs`, `src/config.rs` (pruned) | same | low | prune §22 |
| `src/routes/opds/` | same | low | keep as-is |
| `src/routes/trending.rs` logic | recs v1 (§12) | low | strip decay config sprawl |
| `src/recommender/legacy_cooccur.rs` | related works (§12) | low | |
| `src/tags/` extraction + aliases | same tables (§7.1) | med | keep curator queue, drop elections |
| `src/routes/user_export.rs` idea | GDPR export (§8.4) | med | |
| `src/heal/` telemetry ideas | domain health + circuit breaker (§9.1) | med | |
| `qa/run.js` | same | low | update route list |
| `src/search/parser.rs` + `builder.rs` | deferred to M4 | med | good code, not day-one |
| `src/routes/kindle.rs`, lettre email | notifications crate | med | keep SMTP shape |
| `docs/` legal pages, `/copyright` | same | low | review + shorten |
| Svelte theme tokens (`docs/theme/`) | frontend design tokens | low | 3 presets only |

Do **not** port: progression/XP/quests/abilities, marketplace/recipes/skins, forum (`forum_core`), bounties, work-proposal voting, collections/shelves, trust ladder (7-level), ActivityPub, roadmap consensus, Ask-the-Archive (defer, flag-gated), multi-locale UI, blind-date (keep the tiny `/blind` endpoint only).

## 22. Appendix B — Environment Variables

Pruned from the current 100+ vars (see `src/config.rs`) to what the greenfield system actually needs:

```ini
# core
DATABASE_URL=postgres://fichub:...@localhost/fichub
REDIS_URL=redis://localhost:6379
S3_ENDPOINT=http://localhost:9000
S3_BUCKET=fichub
S3_ACCESS_KEY=...
S3_SECRET_KEY=...
PORT=8000
PUBLIC_BASE_URL=https://fichub.example

# auth
JWT_SECRET=...
DEVICE_COOKIE_SECRET=...
OAUTH_GOOGLE_CLIENT_ID/SECRET=...
OAUTH_DISCORD_CLIENT_ID/SECRET=...

# email
SMTP_HOST/PORT/USER/PASS/FROM=...

# web push (M2)
VAPID_PUBLIC_KEY=...
VAPID_PRIVATE_KEY=...

# workers
WORKER_CONCURRENCY=4
INGEST_SYNC_TIMEOUT_SECS=25
POLITENESS_DEFAULT_DELAY_MS=2000

# feature flags (default off)
FEATURE_LLM=0
OLLAMA_URL=http://localhost:11434   # only read when FEATURE_LLM=1

# conversion sidecar
CALIBRE_SIDECAR_URL=http://localhost:8081   # optional; EPUB/TXT/HTML need no sidecar

# abuse
ANON_RATE_LIMIT_PER_MIN=30
ANON_DOWNLOAD_QUOTA_PER_DAY=200
```

---

*End of specification. Questions, gaps, or disagreements → open a PR against this file; the spec is the source of truth and is expected to evolve with the metrics in §1.1.*
