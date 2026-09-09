# Threadlight Features — FicHub Fit Analysis & Implementation Queue

> Companion to `docs/SPEC-COMMUNITY-PLATFORM.md` (the forum-first community spec, **v2** —
> Slashdot-style moderation points + metamoderation + site-wide leveling).
> This doc inventories every feature claimed by the Threadlight project
> (`/personal/documents/code/rust/threadlight` — FEATURES_AND_IDEAS.md,
> threadlight-project-spec.md, and the actual migrations/router/services),
> verifies what FicHub already has, and decides what to implement — only the
> features that genuinely fit FicHub's forum-first, self-hosted, generic-crate
> direction. Each approved feature has a milestone reference (F1..F7, per the
> v2 milestone table in SPEC §8).

---

## 1. Method

Every Threadlight feature was checked three ways:
1. **Claimed** — what FEATURES_AND_IDEAS.md / threadlight-project-spec.md says.
2. **Implemented** — what actually exists in threadlight's `src/api/mod.rs`
   (only ~15 feature areas are wired into the router), `migrations/`, and
   `src/services/` (many services exist but are partial/stubbed; the spec's
   "~120 routes" claim is aspirational — the router wires ~40).
3. **FicHub state** — grep of FicHub `src/`, `migrations/`, `frontend/` for the
   equivalent capability.

**Decision rule:** implement in FicHub only features that (a) fit the
forum-first community platform, (b) don't duplicate something FicHub already
has, and (c) are honest about what exists vs what's aspirational. Features
that are part of Threadlight's *differentiating* social/economy stack but
contradict FicHub's product direction are listed as **excluded**.

---

## 2. Feature inventory (verified)

### 2.1 Core social

| # | Threadlight feature | Threadlight status (actual) | FicHub status | Decision |
|---|---|---|---|---|
| 1 | User auth (JWT+bcrypt) | ✅ router `/api/v1/auth/*` | ✅ FicHub has JWT+bcrypt auth, `AuthUser` extractor | **Already have** — reuse |
| 2 | Posts (CRUD, archive, mod remove) | ✅ service `post.rs` 358 lines | Forum topics/posts = the new crate | **F1-F3** (as forum topics/posts) |
| 3 | Comments (threaded, materialized path) | ✅ migration `20240725_comments`, service | FicHub has fic comments (`comments` table, threaded via `parent_id`); forum uses flat+quote | **Partially reuse pattern**; forum is flat in M1 |
| 4 | Post voting (likes) | ✅ `post_likes` table + `post_vote` service | FicHub has ratings/reviews; forum needs votes | **F3** (±1 votes on forum posts) |
| 5 | Comment voting | ✅ `comment_likes` | Not needed for forum M1 | Excluded for now |
| 6 | Private messages | ❌ service `private_message.rs` exists but NO router routes wired | FicHub has no PMs | **Excluded** (non-goal) |
| 7 | User profiles (@me, avatar/banner) | ✅ router users/@me | FicHub has settings page + profile | **Already have** |
| 8 | Community notes (fact-checking) | ❌ service exists, no router | Out of forum-first scope | Excluded |
| 9 | Content filters (regex) | ✅ service `filter.rs` | FicHub has honeypot + content-scan, no per-user regex filters | **F8-ish** — could be M2; see §4 |
| 10 | Registration applications | ✅ migration + service + router | FicHub has no invite/application flow (cohort invite is P7#24) | **F7** (done 2026-08-14) |
| 11 | Invites | ✅ migration `user_invites` + handler | No invite system | **F7** (done 2026-08-14) |
| 12 | Site info endpoint | ✅ handler `site.rs` | No `/api/site`; FicHub has `/api/health` | **F7** (done 2026-08-14) |
| 13 | User blocks | ✅ `blocked_users` migration + service | No block system | **F7** (done 2026-08-14) |

### 2.2 Communities & governance

| # | Feature | Threadlight | FicHub | Decision |
|---|---|---|---|---|
| 14 | Communities CRUD | ✅ service + migration | Forum categories = curated, admin-created | **F1/F2** (as categories) |
| 15 | Community join/leave | ✅ | Forum: membership = implicit (categories public) | Simplify — see §4 |
| 16 | Curator system (granular permissions) | ✅ moderation service | FicHub roles: 0 reader / 5 curator / 10 admin (migration 007) | **Already have** — reuse roles |
| 17 | Community forking | ❌ service stub | Not a fit | Excluded |
| 18 | Trust network | ❌ service stub (`trust.rs` 28 lines) | Not a fit | Excluded |
| 19 | Moderation actions + jury | ❌ moderation service exists but jury not wired | FicHub has modlog + curator fixes | **F5/F6** (modlog integration, no jury — done) |
| 20 | Mod log | ✅ migration + router `/api/v1/mod-log` | FicHub has modlog (migration 034) | **Already have** — reuse |
| 21 | Reports | ✅ migration `post_reports` + service | FicHub has `user_reports` (migration 022) + `/api/admin/reports` | **Already have** — target types extended in F5 |

### 2.3 Economy & gamification

| # | Feature | Threadlight | FicHub | Decision |
|---|---|---|---|---|
| 22 | Credit economy | ✅ credit service + leaderboard migration | FicHub has reputation (users.reputation) + leaderboards (curators weekly/monthly) | **Excluded** — FicHub is not gamified; no credits |
| 23 | Daily rewards | ✅ credit service | — | Excluded |
| 24 | Bounties | ✅ credit service + worker planned | — | Excluded |
| 25 | Quests | ✅ quest service exists, no router | FicHub has `/quests` (quests.rs) — but a different quest system | **Already have** — keep FicHub's |
| 26 | Achievements | ✅ achievement service, no router | FicHub has badges (badges.rs) | **Already have** — keep FicHub's badges |

### 2.4 Social organization

| # | Feature | Threadlight | FicHub | Decision |
|---|---|---|---|---|
| 27 | Circles | ✅ circle service + migration, no router | Not a fit | Excluded |
| 28 | Collections | ✅ collection service + migration | FicHub has shelves + reading lists | **Already have** — keep FicHub's |
| 29 | Custom feeds | ✅ feed service + migration | FicHub has follows feed | **Already have** — keep FicHub's |
| 30 | User lists | ✅ userlist service + migration | FicHub has reading lists | Already have |
| 31 | User affinity | ❌ affinity service stub | Out of scope | Excluded |
| 32 | Feed plugins | ❌ feed_plugin service stub | Out of scope | Excluded |

### 2.5 Moderation & safety

| # | Feature | Threadlight | FicHub | Decision |
|---|---|---|---|---|
| 33 | Blocks (user) | ✅ migration + service | No | **F7** (done 2026-08-14) |
| 34 | Blocklist (server-wide) | ✅ migration + handler | FicHub has honeypot/anti-bot | **Already have** — anti-bot covers this |
| 35 | Reports | ✅ | FicHub has reports | Already have |
| 36 | Content filters | ✅ | FicHub has content-scan + honeypot | M2 option (§4) |
| 37 | Warning system | ❌ not in router | FicHub has modlog + reports | **F5/F6** (modlog = the warning record, done) |

### 2.6 Infrastructure

| # | Feature | Threadlight | FicHub | Decision |
|---|---|---|---|---|
| 38 | Site info endpoint | ✅ | No | **F7** (done 2026-08-14) |
| 39 | Health check | ✅ `/health` `/ready` | FicHub has `/api/health` + `health_api` test | Already have |
| 40 | Paginated responses | ✅ | FicHub cursor pagination in most list routes | Already have (reuse pattern) |
| 41 | SQLx compile-time queries | ✅ | ✅ | Already have |
| 42 | Axum 0.8 | ✅ | ✅ | Already have |
| 43 | Redis rate limiting | ✅ | ✅ tiered limiter | Already have |
| 44 | JWT + OptionalAuth | ✅ | ✅ `AuthUser` extractor | Already have |
| 45 | FTS search | ✅ search service | ✅ `body_text_search` (migration 040) | **F4** (forum search, done 2026-08-14) |

---

## 3. Approved implementation queue (F1..F7, v2)

Each is a **commit-ready unit** — one worktree branch, one subagent, then main
agent merges + commits. This is the plan for the forum-core crate + FicHub
integration + SvelteKit UI, mirroring `fanfic-scrapers` as a standalone crate.

| Milestone | Feature | Crate | FicHub routes | Frontend | Tests |
|---|---|---|---|---|---|
| F1 | `forum-core` crate skeleton: model + `Store` trait + `postgres` feature + migrations + unit tests | `forum_core/` | — | — | crate `cargo test` |
| F2 | Categories + topics CRUD (read) + list by category | `topics` | `GET /api/forum/categories`, `GET /api/forum/topics?category=` | `/forum` category list, `/forum/[cat]` topic list | DB-gated `tests/forum_api.rs` + vitest |
| F3 | Posts + follows + notifications (NO ±1 votes — mod points are the only score system) | `posts`, `follows` | `POST/GET/PATCH/DELETE /api/forum/topics/{id}` + `/posts/{id}`, `POST /follow`, notification wiring | topic view, reply composer, follow toggle, bell | + e2e |
| F4 | Read state + search (FTS tsvector + GIN, snippets) | `readstate`, `search` | `POST /api/forum/topics/{id}/read`, `GET /api/forum/search?q=` | unread dots/badge, `/forum/search` | + integration |
| F5 | **Moderation points**: grants, reasons, queue, thresholds, reports-as-signal, curator fast-hide | `moderation` | `GET/POST /api/forum/moderation/*`, `/api/admin/forum/hide` | `/forum/moderate`, mod reason picker | + e2e |
| F6 | **Metamoderation**: queue, anonymized audit, rolling unfair-rate, cooldowns | `moderation` | `GET /api/forum/metamod/*`, `POST /api/forum/metamod/{action}/vote` | `/forum/metamod` | + integration |
| F7 | **Leveling (site-wide)**: exp + 0-100 levels replace role/reputation; invites + registration applications; blocks; site info | — | level gates sweep, `/api/invites`, `/api/registration-applications`, `/api/blocks`, `/api/site` | settings/profile level UI, invite + application pages, block list | + e2e |
| F8 | **Release & publish**: AGPL-3.0 license + docs + crates.io publish of `forum-core` (mirror the `fanfic-scrapers` release path) **+ publish to its own public repo at opencommit.eu (MagicZhang/forum-core) with description + topic tags** — only after all of F1-F7 complete + verified | `forum_core` | ✅ **DONE 2026-08-15**: crate v0.1.0 live on crates.io; repo public w/ 12 topics | — | release checklist + publish dry-run |

**F8 release checklist (prepared — fanfic-scrapers precedent):**
- ✅ LICENSE file (AGPL-3.0 full text) + `license = "AGPL-3.0-or-later"` in Cargo.toml — done `a67bba9`
- ✅ `repository` metadata → `https://opencommit.eu/MagicZhang/forum-core` — done `a67bba9`
- ✅ README current (Vote-model clarification) — done `a67bba9`
- ✅ `cargo publish --dry-run` passes clean (12 files, no warnings) — done
- ✅ **crates.io publish `forum-core` v0.1.0** — **LIVE** (index.crates.io confirmed, not yanked)
- ✅ **opencommit.eu repo `MagicZhang/forum-core`** — created public, `main` pushed (51KB), all 12 topic tags set, raw URL 200
- ⏳ nothing left — F8 complete pending the above verifications (all confirmed)

---

## 4. Explicit exclusions (don't build)

These are Threadlight features that contradict FicHub's forum-first direction
or duplicate existing systems:

- **Credit economy / daily rewards / bounties** — FicHub is deliberately not
  gamified. Reputation + badges already exist and are enough.
- **Circles / trust network / affinity / feed plugins** — out of scope for a
  forum; FicHub's rec platform (pgvector + strategies) is the recommendation
  system.
- **Private messages** — explicit non-goal of the community spec (M1).
- **Community forking / jury moderation** — heavy governance machinery with no
  cohort-sized need; modlog + reports are sufficient.
- **Community notes (fact-checking)** — FicHub's curator/content-scan covers
  content quality.
- **Content filters (user regex)** — M2 candidate at most; FicHub's
  honeypot + content-scan + anti-bot cover abuse. Revisit after M1 cohort.
- **Post content types (link/image/video/mood)** — forum posts are markdown +
  opaque link-card payload (SPEC §3.1). No media pipeline in M1.
- **Scheduled posts / content calendar** — no fit.

## 5. Notes & hazards (learned while verifying)

- **Threadlight's router is the truth, not its spec tables.** FEATURES_AND_IDEAS
  claims ~40 features as "✅"; the actual `src/api/mod.rs` wires ~15 areas.
  Services for the rest exist but are partial or stub. We implement against the
  FicHub spec, not Threadlight's marketing.
- **Threadlight uses `BIGSERIAL` ids + `i64`; FicHub uses `users.id INT4`/`i32`**
  and `BIGSERIAL` for most other pk's. The forum crate must stay generic over
  the actor id type (`i32` in FicHub) — that's exactly what the generic crate
  boundary buys.
- **`user_reports` CHECK constraint** (migration 022) currently only allows
  `('comment','work','user')`. F7 adds `forum_topic`/`forum_post` via ALTER.
- **Notification types are free text** (migration 003) — `forum_reply` /
  `forum_mention` need no migration.
- **Roles** (migration 007): 0=reader, 5=curator, 10=admin. Forum moderation
  gates: `role >= 5` for mod actions, `role >= 10` for category admin.
- **`+layout.svelte` `routePages`** must include new route prefixes or the SPA
  renders the home dashboard instead (AGENTS.md).
- **Route test files must be `page.test.ts`**, not `+page.test.ts`.
- **DB-gated suites each own a mutex**; run `cargo test --test forum_api
  -- --test-threads=1` for determinism (cross-suite pollution gotcha).

---

## 6. Status

- [x] F1 crate skeleton — **DONE 2026-08-14** (ff6ee77, merged 48eb3df; wired into root Cargo.toml 3285a5d)
- [x] F2 categories/topics read — **DONE 2026-08-14** (ac95ceb, merged df53eee)
- [x] F3 posts + follows + notifications — done (merged 2026-08-14)
- [x] F4 read state + search — done (merged 2026-08-14)
- [x] F5 moderation points + scoped bans/timeouts — done (merged 2026-08-14)
- [x] F6 metamoderation — done (merged 2026-08-14)
- [x] F7 leveling (site-wide) + invites/applications + blocks + site info — done (merged 2026-08-14: backend c87fed4+2a427e5, frontend a36c9f5)
- [ ] F8 release & publish — AGPL-3.0 + docs + crates.io (only after F1-F7 verified) — pending
