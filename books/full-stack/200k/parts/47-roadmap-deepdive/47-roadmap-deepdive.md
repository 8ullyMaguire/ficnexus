# Part 47 — Roadmap Deep-Dive: What Could Come Next

You've built FicHub from `cargo init` to a full archive platform. Parts 1–46 taught you every shipped feature. This part is different: it doesn't teach you to build something. It walks through every feature idea the project considered — what shipped, what's a cheap win, what's a big bet, and why. Think of it as sitting in on a product-planning meeting, except the meeting already happened and you have the minutes.

The source of truth is `docs/brainstorm-02-roadmap-status.md` — the consolidated project scratch file everyone edits. This part distills it for you.

---

## 47.1 Shipped — the recap tour

Before we look forward, here is everything the previous 46 parts built — with the commit hashes and migration numbers you'd grep for to find the code yourself.

| Area | What shipped (Aug 2026) | Where in the code |
|------|------------------------|-------------------|
| Multi-format export | EPUB, HTML, MOBI, PDF, AZW3, TXT, MD — pure-Rust epub-builder, no external tools | Part 7; `src/export/`, `epub-builder` crate |
| Scraper platform | fanfic-scrapers crate — **107/107 real FanFicFare adapters** as native Rust scrapers; login, cookie-store, is_adult gate, cookie re-ingest; Python-CLI removed | Parts 4–6; `fanfic-scrapers` crate, `src/scrapers/`, `src/export/fallback.rs` |
| Self-healing | M1: scrape-failure classifier (transient/blocked/structural/systemic), HTML snapshots, `POST /api/admin/heal` diagnose-only. M2: on-the-fly scraper creation by agent on structural failure, safety-gated. | Migrations 029–031, 050–051; `src/heal/`, `AGENT_ENABLED=false`, `AGENT_USE_ON_FLY=true` |
| Site-as-cache | Every scraped fic body persisted as sharded versioned files under BODY_CACHE_DIR; exports reuse cache; curator fixes peer-voted (migration 032: propose → vote → apply at quorum ≥2) | `src/cache/`, migration 032 |
| v3 progression system | **100 levels, 10 ranks**, XP engine, ability tree UI (`/features`), widget-composed home dashboard (rank 5+), theme/layout customization (rank 1+/3+), custom saved views (rank 7+), admin feature editor | Migration 052; `src/routes/progression.rs`, `src/services/progression.rs` |
| Trust system | Discourse-style **7 levels** (TL0 New → TL6 Near-admin), separate from XP/level/rank. TL0 sandboxes new accounts; TL2+ gates publishing to shared galleries; TL5+ Community Moderators resolve reports; role ≥10 sole authority for hide/ban/admin | Migration 013; `src/bin/trust_promote.rs`, `frontend/src/routes/settings/trust/` |
| Reputation/XP rewards v3 | `POST /api/xp/idle-tick` deduplicates per-user hourly buckets (max 24 awards/day = 48 XP); `POST /api/admin/reputation/award` live for role ≥10, admin XP sources; all grants modlogged | Migration 011; `docs/plans/reputation-xp-rewards-v3.md` |
| Badge-wiring fix | `POST /api/admin/reputation/award` now calls `check_and_award_badges`; tier-1 dead code removed; daily-quest gamification loop scrapped; `/api/reading/*`, `/api/users/{id}/streak`, `/api/users/{id}/reading-stats` kept | Commit 591776c |

| Forum depth phase 2 | F1 read-state badges, F2 pin/lock affordances (locked 403, pinned ordering), F3 metamod queue polish (filterable unreviewed/reviewed, pagination next_cursor, 409 single verdict, anonymized grant detail, unfair_rate/cooldown_until), F4 per-author mod drawer | Commit cd61b1a (2026-08-24); forum-core crate `0.1.0`, repo opencommit.eu/MagicZhang/forum-core |
| Extension marketplace | Unified `/marketplace` gallery for themes, skins, recipes, layouts, views; trust-gated publishing (TL2+); one-click idempotent install; 1–5 star ratings with running average; remix (copy public as private draft) | Commit 63e6c66 (2026-08-27), migration 014; `src/routes/extensions.rs`, `services/extensions.rs`, `docs/v3-customization-spec.md` |
| v3 customization platform | Recipe Builder (user-composed rec blends), Plugin Manifest (unified shareable objects table), Theme Design Tokens (CSS custom properties + 5 presets + import/export + live preview) | Migrations 052 + 053; `src/routes/recipes.rs`, `services/recipes.rs` |
| Site quality pass 4 | OPDS hash-aware acquisition links + RWPM manifest (GET /opds/manifest, 6 sections, root Atom feed rel=alternate type=application/webpubmanifest+json); curator consensus hub (public-read for logged-in users, filter chips + search + status/sort deep-linking); typed fic-request answers (AnswerRow.svelte, LLM auto-answer attributed "FicHub Archivist", graceful fallback); ask↔requests unification; search-history chips (migration 063); AO3 symbol squares + guide modal; Marginalia; emoji reactions (anchored picker); sequential-Markov Next Up; personalized-recs opt-in; per-endpoint usage analytics; ask-response caching | Various commits; migrations 033, 063 |
| lfm2.5:8b model | Canonical Ollama model — benchmark winner 21.3–21.7 tok/s on ThinkCentre; supersedes qwen3.5:9b; keep_alive 30m | `src/services/ollama.rs` |
| FFF 107/107 parity | Every FanFicFare adapter site as native Rust scraper; login support; cookie re-ingest; is_adult gate (7 adult archives); Python-CLI removed; native-scraper routing fix (RoyalRoad ~62s→~280ms); markup-drift fix; user-supplied credentials (AES-256-GCM encrypted, 30-day expiry, passwords never returned) | `fanfic-scrapers` crate v0.6.0→v0.10.0 |

| Anti-bot | Honeypot traps, tiered rate limits, Redis shadowban, PoW challenge, hourly bot-scorer, `/admin/bots` | Migrations + `src/bin/bot_scorer.rs` |
| Bulk admin actions | Atomic-at-request bulk operations | Commit b017681 |
| main_char_attr | **Removed** (commit b017681) — search is now fully tag-based. This unlocks a search system overhaul. |
| Fandom landing pages | `/fandoms` + `/api/fandoms` | Commit 21a0440 (2026-08-17) |
| FFN metadata fallback | `fichub_net.rs` async client for `GET fichub.net/api/v0/epub`, wired into `ffnet.rs` for CF challenges and words==0 parses | Commit 99a088a |
| Saved searches + daily alerts | Save/list/run/delete/alert-toggle; nightly re-run + match diff; per-search Atom feed | Migration 007; `scripts/install_systemd_timers.sh` (fichub-saved-search, nightly 02:00) |
| Follow Exclusions | Exclude works/series/fandoms from author follow; feed anti-join + Exclusions.svelte | Migration 002 |
| Collection submissions + unified curator approvals | `collection_submission_votes` + single `/curator/approvals` queue aggregating collection + proposal + triage items with type/status filter | Migration 006 |
| Docs hub fix | `/docs` no longer links to Forgejo HTML; left column lists in-app mdbook entries, right pane renders markdown | Commit 581c751; `build_docs_map.py` |
| Consensus auth fix | `/api/curator/consensus` readable by any logged-in user (was curator-only 403); write/vote paths unchanged | Commit 66ac13e |
| Registration honeypot fix | Frontend sends `form_opened_at` + `website:''` | Commit b038705 |
| Progression endpoint SQL fix | `COALESCE(smallint, 0)` type mismatch | Commit 59454cf |
| Forum "New category" fix | i18n key x6; .new-category form un-hidden in archive mode with AO3-styled submit | Commit |
| Modlog | Migration 034; every admin/curator action recorded; `GET /api/modlog` + `/modlog` page readable by **any logged-in user** | `src/modlog.rs` |
| Usage analytics | `usage_events` (migration 033) + middleware recording views vs actions (X-Client-ID, zero-PII); `/admin/analytics` shows unique daily/weekly/monthly visitors, active vs view-only users, action timeline, search→export conversion | `src/routes/admin/analytics.rs` |
| API route-walk | `qa/api-walk.js` — scriptable API-surface audit, parses every route from `src/server.rs`, hits each endpoint, asserts gates + flags 5xx/stubs; caught + fixed two real bugs (reading-stats 500 on unknown users; analytics 500 on empty request_log) | `qa/api-walk.js` |

> **💡 Key Concept**: This is the difference between a stale tutorial and the real codebase. Every line above has a commit hash or migration number. If you hear "Part 49 claims X shipped but git says otherwise," you can check the hash.

---

## 47.2 P1 — Content bottleneck (the hard problem)

### Cookie ingestion for AO3/FFN

**What it is**: The server can't reach AO3 (404+challenge) or FFN (403 Cloudflare) outbound. Users want their fics from those sites. The fix isn't the server scraping — it's the user feeding the server a cookie they already have.

**What already exists**: `src/export/fallback.rs` calls `https://fichub.net/api/v0/epub?q=<url>` as a scraping fallback for AO3/FFN when the native adapter can't reach the host. Body cache + curator peer-voted fixes make failed scrapes recoverable. `fanfic-scrapers` has `SiteScraper::login`, `SiteCredentials`, `FANFICSCRAPER_<DOMAIN>_USER/_PASS`, `cookie_store`, `lookup_authed`. `is_adult` gate exists.

**Pros**: Unlocks the two biggest archives. Pragmatic — doesn't require the server to solve Cloudflare.

**Cons**: Security review needed (cookies are credentials). User has to open the blocked page in their own browser once and paste the cookie. Not automatic.

**Verdict**: *Product decision* (not a technical commit). Stated as P1#1. USER-ACTIONS.md §4 has the cookie-import flow. Cookie-ingest is NOT shipped — the host is still blocked outbound.

> **⚠️ Watch Out**: Don't describe cookie ingestion as "we handle AO3/FFN." We handle them via the `fichub.net` fallback, not native outbound scraping. The distinction matters.

### Scriptable API-surface e2e in CI

**What it is**: `qa/api-walk.js` already exists — it parses every route from `src/server.rs`, hits each endpoint, asserts gates + flags 5xx/stubs. It caught two real bugs (reading-stats 500 on unknown users; analytics 500 on empty request_log). The idea: run it as a CI gate.

**Pros**: Catches breakages before deploy. Zero cost to keep running — it's already written.

**Cons**: Needs a running dev server to hit. Slower than unit tests.

**Verdict**: *Cheap win.* Add the script as a CI step. One config edit. Do it.

### Main-character / ship score fixing

**What it is**: The old `main_char_attr` field used to let users pick a main character, which fed a "ship score" for recommendations. It was removed (commit b017681) because it created schema complexity for weak signal.

**What already exists**: It's gone. Search is fully tag-based now. The recommender uses tag/category/fandom signals, not character picks.

**Pros**: Cleanup. Unblocks a search overhaul (tag-based is simpler).

**Cons**: Nothing — the field is removed, not pending.

**Verdict**: *Ship now* — it's already removed. Don't describe `main_char_attr` picker as a pending feature. It's gone.

---

## 47.4 P3 — Make the rec platform earn its keep

### Shadow-run decay + embeddings

**What it is**: Two recommendation strategies already exist — co-occurrence decay (`decay`) and embeddings (`embeddings` via lfm2.5:8b). The idea: run them side-by-side and let engagement data pick the winner.

**What already exists**: `REC_STRATEGIES` config supports multiple strategies. `rec_impressions` tracks what users engage with. Shadow-run mode (`REC_SHADOW_MODE=true`) runs alternatives silently.

**Pros**: Data-driven. Cheap — both strategies exist. Golden test possible (fixed seed).

**Cons**: Need engagement data to accumulate before conclusions are meaningful.

**Verdict**: *Cheap win.* Set `REC_SHADOW_MODE=true` + `REC_STRATEGIES=cooccur:1.0,decay:0.0,embeddings:0.0`. Read `rec_impressions` after a week; promote the winner by flipping weights.

### Sequential Markov in Next Up

**What it is**: The current Next Up panel uses a fixed strategy blend. Sequential Markov (`rank_targets`) would order recs as a sequence, not a bag.

**Pros**: Better "streak" feel. Currently wired but underused.

**Cons**: Needs a settings toggle (`recs.personalized` pref in `user_prefs`).

**Verdict**: *Cheap win.* Hook `rank_targets` into the Next Up panel query.

### Personalized recs by default

**What it is**: When a user is logged in, use their personal recs instead of the generic blend.

**Pros**: Better first impression for logged-in users. Currently opt-in.

**Cons**: Cold-start problem for new users with no history. Need a settings toggle.

**Verdict**: *Spec first.* Needs the `recs.personalized` pref + a cold-start fallback (new users get trending until they bookmark something).

> **💡 Key Concept**: Personalization without cold-start handling is a bad first impression. Always have a fallback for users with no signal.

---

## 47.5 P4 — Differentiators (why FicHub is special)

### Full-text search over fic bodies

**What it is**: Search inside fic text, not just metadata.

**What already exists**: Shipped. `body_text_search` with tsvector + GIN index + `<mark>` snippets. `/search/body`.

**Pros**: OTW doesn't have this (AO3 searches synopsis, not body). Differentiator.

**Cons**: Index size. Body cache growth.

**Verdict**: *Ship now* — already done. This is why the body cache exists.

### Analytics feedback loop

**What it is**: Connect search queries to export conversions. If a search returns 0 results and the user exports nothing, that's a content gap.

**Pros**: Turns usage data into action. Already tracking both.

**Cons**: Need a dashboard to surface it. Currently raw data only.

**Verdict**: *Cheap win.* Add a "top zero-result queries" panel to the admin analytics page. One GROUP BY.

### Per-endpoint usage breakdown

**What it is**: `usage_events` records every request path. A GROUP BY over paths tells you which endpoints are hot.

**Pros**: Cheap. Data exists. Helps prioritize perf work.

**Cons**: Need to aggregate. Currently raw.

**Verdict**: *Cheap win.* One GROUP BY over `usage_events`.

---

## 47.6 P5 — Curator/admin UI backlog (APIs mostly exist)

| Feature | Status | Verdict |
|---------|--------|---------|
| Auto-tag review UI | API exists; no dedicated review page | *Cheap win* (wire the route) |
| Manual fic approval UI | `/api/admin/moderation/queue` exists; no UI page | *Cheap win* (just wire the route) |
| Metadata correction | `/admin/metadata` page + API shipped (P8) | *Ship now* |
| Report handling | `POST /api/reports` exists; no admin UI | *Spec first* |
| Translation post-edit | Translation review exists; post-edit is part of it | *Ship now* (covered) |
| Rating/warning verification | `work_rating_verifications` + UPSERT shipped | *Ship now* |

---

## 47.7 P6 — Cheap UX wins

| Feature | Status | Verdict |
|---------|--------|---------|
| Loading skeletons | Partial — shipped on `/my-works`, `/collections`, `/collections/[id]`, `/authors` | *Cheap win* (extend to remaining pages) |
| Ask-the-Archive caching | keep-warm cron needed (cold ~17.6s, `keep_alive: 30m` can lapse) | *Cheap win* |
| Search-history chips | **DONE** — migration 063, `GET /api/search/history/chips` | *Ship now* |
| User skins / work skins | `skins` + `user_skins` tables shipped (001_initial.sql); marketplace skin kind in v3.1 | *Cheap win* (work-level picker) |
| Half / quarter star ratings | Ratings store integer 1–5; frontend display toggle | *Cheap win* (one `user_prefs` column, no DB change) |
| Kudos privacy toggle | `kudos_private` in `user_prefs` (default public); anonymous kudos not allowed (already requires login) | *Ship now* (anon lock shipped) |

**Settings for ratings**: Allow user to set star display granularity — whole, half, or quarter stars. The backend stores integer 1–5; the frontend toggles the visual grain. One `user_prefs` row (`rating_granularity: 1|2|4`), one star-widget function. Cheap because no schema change required — integer storage stays, display changes.

**Kudos privacy and authentication**:
- **Anonymous kudos not allowed** — `POST /api/kudos/{work_id}` already requires `auth` (signed-in user). The kudos table stores `user_id`; guests get no kudos row. This is already enforced by the route extractor requiring a logged-in user. _Default state: this is the shipped behavior._
- **Privacy toggle** — kudos are public by default (shown in the feed on work pages). Add a per-user setting to make their kudos **private**: a `kudos_private: bool` in `user_prefs` (default `false`). When `true`, that user's kudos do **not** appear in the `/api/kudos/{work_id}/recent` feed (the feed already filters by `k.user_id IS NOT NULL` — add an additional `AND NOT (SELECT kudos_private FROM user_prefs WHERE user_id = k.user_id)`). The work's kudos **count** still includes theirs (aggregate).
- **Clicking kudos shows who kudosed** — the kudos feed endpoint (`GET /api/kudos/{work_id}/recent`) already returns the list of users who kudosed a work, paginated. This is the "who's kudosing" list shown on the work page sidebar. No additional work needed — it's shipped. The "clicking kudos shows users" UX is already wired: the WorkBlurb component fetches the recent kudos list and displays usernames.

**Verdict**: *Ship now* for the anonymous-lock (already done). *Cheap win* for the privacy toggle (one column, one WHERE clause change). Already shipped for "clicking kudos shows who kudosed."

---

## 47.8 P7 — Bigger bets

### Invite the small cohort

**What it is**: Invite codes + registration applications (F7) ship with the leveling milestone. Onboarding flow + seed content + "new member" landing remain.

**What already exists**: The full governance layer — modlog, consensus, transparency, trust system — is ready. The invite system is shipped but dormant.

**Pros**: Controlled growth. No open registration means no spam flood.

**Cons**: Needs onboarding copy + seed content selection.

**Verdict**: *Spec first.* Needs onboarding copy + seed content selection.

### Verify Ask-the-Archive live translation with lfm2.5:8b

**What it is**: The previous live test (with qwen3.5:9b) showed `main_char_attr` null. With `lfm2.5:8b` as the default model now, re-verify.

**Pros**: Quick test. Confirms the model upgrade fixed the earlier null issue.

**Cons**: One-off test, not ongoing monitoring.

**Verdict**: *Ship now.* Quick live test: `curl /api/search/ask` with a Chinese query, check `translated: true`.

---

## 47.9 P8 — 50‑feature fit audit (the big list)

> 2026-08-15: a 50-item feature list was reviewed against the roadmap. Every item is captured below. Items already tracked keep their P-number; genuinely new items get a P-number here. Priority leans on the user's "cheapest wins" note.
>
> 2026-08-15: **OTW Archive fit audit** — surveyed github.com/otwcode/otwarchive and added missing features that fit FicHub #52–#72. **Brainstorm review** added 22 more as #73–#94.

### Download & ingestion

| # | Feature | What exists | Pros | Cons | Verdict |
|---|---------|-------------|------|------|---------|
| 1 | Batch series/author download (one zip of EPUBs) | Multi-select UI pattern; `zip` crate available | One click, huge value | Multi-URL scraper coordination | *Spec first* |
| 2 | Download queue with progress (cancel/resume) | No queue infra | Big batch UX | Complex state mgmt | *Spec first* |
| 3 | Customizable EPUB (cover, font, notes, toggles) | epub-builder available | High personal value | UI complexity | *Spec first* |
| 4 | Cookie ingestion for AO3/FFN | P1#1 — see above | Unlocks AO3/FFN | Security review needed | *Product decision* |
| 5 | Wayback Machine fallback | FAQ mentions it; no impl | Resilient to source death | Wayback fidelity varies | *Spec first* |
| 6 | Update watcher (cron re-check ongoing fics → notify) | `follows` + `updates` exist | Automatic freshness | Cron + notification complexity | *Spec first* |
| 7 | Cover image API (`/api` covers) | No impl | Lets userscripts / external frontends show covers | Trivial | *Cheap win* |
| 8 | CORS on API | Off by default | Enables bookmarklets/userscripts | Security review (open CORS) | *Spec first* |
| 9 | Instance import tool | No impl | Migration from fichub.net / other instances | Data mapping complexity | *Big bet* |

### Search & discovery

| # | Feature | What exists | Pros | Cons | Verdict |
|---|---------|-------------|------|------|---------|
| 10 | Search-history chips | **DONE** (063) | Personal discovery | None | *Ship now* |
| 11 | Saved searches (notify on new match) | **SHIPPED** (007) | Retention hook | Timer infra | *Ship now* |
| 12 | Main-character/attribute picker UI | **N/A** — main_char_attr removed | — | — | *No* |
| 13 | Public community shelves page | `reading_lists` exists; no public browse | Social proof | Privacy policy | *Cheap win* |
| 14 | Trope-combo builder (intersect two attributes on /tropes) | No impl | Discovery depth | Query complexity | *Spec first* |
| 15 | Quote of the day (from body FTS index) | `body_text_search` shipped | Daily engagement hook | Cache invalidation | *Cheap win* |
| 16 | Cross-fandom bridge (co-bookmark movement) | No impl | Cross-fandom discovery | Complex query | *Spec first* |
| 17 | Mood preset chips (angsty/fluffy/slow-burn facets) | No impl | Emotional discovery | Tag taxonomy needed | *Spec first* |
| 18 | Tag-wrangling assistant (admin synonym/merge suggestions) | No impl | Metadata quality | Admin tool | *Cheap win* |
| 19 | "Also bookmarked" anchors in search results | No impl | Social proof in results | Aggregation cost | *Cheap win* |

### Reader & reading experience

| # | Feature | What exists | Pros | Cons | Verdict |
|---|---------|-------------|------|------|---------|
| 20 | Chapter highlights/annotations (private quotes & notes) | Marginalia (passage-based forum threads) exists | Personal reader layer | Private vs public distinction | *Spec first* |
| 21 | Reading stats dashboard (words, finished, shelf %, monthly recap) | `reading_history` + `reading_status` exists | "Spotify Wrapped" for reading | Aggregation | *Cheap win* |
| 22 | Estimated reading time on cards + reader | No impl | "12 min read" on every card | Per-chapter word count | *Cheap win* |
| 23 | Text-to-speech mode in web reader | No impl | Accessibility + hands-free reading | TTS engine choice | *Spec first* |
| 24 | Full app-shell offline PWA | Partial shipped | Offline reading | Service worker complexity | *Spec first* |
| 25 | Chapter-release countdown (cadence of followed in-progress) | No impl | Anticipation hook | Cron + notification | *Spec first* |
| 26 | Reader keyboard shortcuts (j/k nav, font keys, bookmark) | No impl | Power-user UX | Keybinding system | *Cheap win* |
| 132 | Bookmark/history export (CSV/JSON) | `bookmarks`, `user_ratings`, `reading_history` exist | Reduces lock-in anxiety | One SELECT + serialize | *Ship now* |
| 133 | Emoji reactions redesign | ReactionsPicker + 051 shipped; newer picker design supersedes | Polish | Design refresh | *Cheap win* |

### Recommendations & personalization

| # | Feature | What exists | Pros | Cons | Verdict |
|---|---------|-------------|------|------|---------|
| 27 | Recommendation explainability ("because you bookmarked X") | In recommender; not surfaced | Trust in recs | Explanation generation | *Cheap win* |
| 28 | Rec feedback loop (👍/👎 → bandit weights) | No impl | Improves recs over time | Bandit infra | *Spec first* |
| 29 | Taste profile page (tag affinities + cluster) | No impl | User identity | Data aggregation | *Cheap win* |
| 30 | "People like you also loved" (clusters strategy on home) | No impl | Social discovery | Clustering | *Spec first* |
| 31 | Mood slot ("give me something dark tonight") | No impl | Intentional discovery | Mood taxonomy | *Spec first* |
| 32 | Cold-start onboarding picker (3 picks to seed recs) | No impl | Solves cold start | Picker design | *Spec first* |
| 33 | Strategy shadow-run dashboard (per-strategy engagement) | Shadow-run tracked; dashboard half P8 | Data-driven decisions | Dashboard UI | *Ship now* (half — data exists) |

### Community & engagement

| # | Feature | What exists | Pros | Cons | Verdict |
|---|---------|-------------|------|------|---------|
| 34 | Reading challenges (monthly goals → badges) | Progression system exists | Engagement loop | Challenge design | *Spec first* |
| 35 | Comment reactions — anchored emoji picker with search bar; positive-only policy | Spec refined 2026-08-23; `docs/REACTION-EMOJI-DESIGN.md`; 051_forum_reactions.sql | Engagement on comments | Design refresh | *Cheap win* |
| 36 | Author claim & verification (badge + metadata fixes) | Pseuds exist | Metadata quality | Verification flow | *Spec first* |
| 37 | Fic gifting (rec + personal note to a user) | No impl | Social acquisition | Gift UI | *Spec first* |
| 38 | Reading clubs (private groups + shelf + thread) | Forum exists | Community depth | Grouping model | *Spec first* |
| 39 | Weekly digest (followed fandoms/tags, in-app or email) | No impl | #1 retention tool | Email delivery | *Spec first* |
| 40 | Engagement showcase (kudos/comments/bookmark counts, positive-only) | Kudos/ratings exist | Social proof | Aggregation | *Cheap win* |
| 51 | Ghost/absorbed accounts — free usernames of inactive users (FORUM-BRAINSTORM B16) | No impl | Username recycling | Trust implications | *Spec first* |
| 131 | Cross-site source links on fic page — each fic_info source renders as clickable badge linking to original fic page on that site; each scraper defines `source_url(fic_info_id) -> Option<String>` | `fic_info` sources exist; no display | Social proof at a glance | Scraper method per site | *Cheap win* |
| 41 | Tropes bounties — users stake XP or bounty pool to incentivize fics for specific trope combinations/settings underserved; first fic satisfying criteria wins pool; pairs with P8#34 reading challenges + P8#29 taste profile | No impl | Gamifies creation | Bounty verification | *Spec first* |
| 124 | Reputation + daily login streaks + community bounties — daily login streaks (XP for consecutive daily visits), forum posting reputation (XP for posts/replies receiving positive feedback), community bounties on Fic Requests (users stake XP to incentivize authors; winner takes pot). Pairs with P8#41 tropes bounties + P8#117 forum level gate + P8#118 daily quests/streaks. Existing progression system (100 levels, exp_events, award_exp) provides foundation — this adds reputation currency layer + bounty mechanics | Progression system exists | Retention + creation incentive | Bounty escrow model | *Spec first* |

### OTW Archive fit audit — genuinely missing (see #52–72)

These were surveyed from `otwarchive` and are genuinely missing from FicHub:

| # | Feature | What exists | Verdict |
|---|---------|-------------|---------|
| 52 | Kudos — one-click anonymous appreciation (no text, one per user per work; guest counts separate). Positive-only, zero effort, cheapest engagement win OTW proves | **SHIPPED** 2026-08-15 (migration 048, `/api/kudos/{work_id}`, work-page button, search min_kudos + sort repointed to real kudos) | *Ship now* |
| 53 | Pseuds (multiple identities per user) — each with own author page + bookmarks; all share one account | **SHIPPED** (`pseuds` table in 001_initial.sql + `src/routes/pseuds.rs`) | *Ship now* |
| 54 | Orphaning — user-initiated "orphan my work/pseud/series" → content moves to site `orphan_account`, username freed, no data deleted; voluntary version of B16 forced absorption | No impl; ghost-account machinery from B16 | *Spec first* |
| 55 | Subscriptions — per-work/per-author/per-series subscribe (distinct from follow: opt-in email/RSS digest). FicHub has follows but no email/RSS channel | No impl | *Spec first* |
| 56 | Mark-for-later — "Want to Read" shelf exists (P8#24 has toread via `want_to_read`); OTW's dedicated "Mark for Later" home page is same data, separate surface | Covered by `want_to_read` shelf | *Ship now* (covered) |
| 57 | Gift exchanges / fests / challenges — structured prompt-meme + gift-exchange: signup with offers/requests, tag-set restrictions, matching, anonymous reveals; richest community feature OTW has; FicHub's consensus/roadmap machinery is adjacent but distinct | No impl | *Big bet* |
| 58 | User skins + work skins (custom CSS) — OTW lets users theme whole site and authors theme individual works | `skins` + `user_skins` shipped (001); `/settings/theme` for app design tokens; skin-kind in v3.1 extension marketplace | *Ship now* |
| 59 | User-level mute/block site-wide — OTW has global block.rb + mute.rb (hide user's content site-wide, not just forum). FicHub's user blocks are forum-only today | `blocked_users` table + POST/DELETE/GET `/api/blocks` shipped | *Cheap win* |
| 60 | Username/email history (admin) — track past usernames + emails per user for identity forensics (ban evasion, sockpuppet rings); cheap: `user_past_usernames` table | No impl; cheap data model | *Cheap win* |
| 61 | Reading history + stats — per-user reading history (what/when you read), with aggregate reading stats dashboard | P8#21 (reading stats dashboard) adjacent | *Cheap win* |
| 62 | Abuse report queue UI — OTW's abuse reports + admin queue; FicHub has moderation/flagging but no dedicated abuse-report admin surface | `POST /api/reports` exists; no admin UI | *Spec first* |
| 63 | Moderated works / manual approval — OTW lets admins gate works behind review; FicHub has manual fic approval (P5#15) + translation review | P5#15 exists | *Cheap win* |
| 64 | Fannish next-of-kin — designate trusted user who can take over/delete your works if you die/disappear; distinctively OTW; cheap data model, meaningful trust feature for community archive | No impl | *Cheap win* |
| 65 | Gifts (fic gifted to a user/pseud) — dedicated "gifted to X" link + gift inbox; already partially in P8#37 (fic gifting); OTW's version adds "gift exchange" reveal mechanic | Covered by P8#37 | *Ship now* (covered) |
| 66 | Tag sets / tag-set nominations — structured collections of tags for challenge signups + community-organized "nominate your tags" drives; fits tag-wrangling direction (P8#18) | No impl | *Cheap win* |
| 67 | Site-wide admin announcement banner — one dismissible banner (e.g. outage, migration, event), cached, "banner seen" per-user; FicHub has no announcement surface today | No impl | *Cheap win* |
| 68 | Per-fic hit/view counter — public view counts on fic cards + reader; OTW's stat_counter/redis_hit_counter prove pattern; FicHub tracks usage_events but has no public per-fic view count (forum topics have view_count, fics don't) | `usage_events` exists | *Cheap win* |
| 69 | Comment-reply inbox — unified "replies to your comments" inbox with unread counts + in-reply-to threading; FicHub has notifications/read state; dedicated comment-reply inbox (OTW inbox_comment) is distinct | Notifications/read state exist | *Spec first* |
| 70 | Known-issues / status page — admin-curated "known issues" list with public status view (title, content, severity/status); cheap, high trust value for self-hosted community | No impl | *Cheap win* |
| 71 | External works (record a fic hosted elsewhere) — bookmark/record off-site works (dead link, other archive, original fiction) with author, language, tags; no scraping; fits archive direction | No impl | *Cheap win* |
| 72 | Per-fic language metadata — OTW's language.rb (default English, per-work language, sortable); FicHub has UI i18n (028_i18n_seed.sql) but no per-fic language field | No impl | *Cheap win* |

**Fastest OTW wins (cheap, high fit)**: #52 kudos (one-click), #60 username history (admin table), #64 fannish next-of-kin (data model), #59 user-level mute (extends existing blocks), #61 reading history (extends P8#21), #67 announcement banner, #68 per-fic hit counter, #72 per-fic language metadata.

### Brainstorm review (2026-08-15) — re-read FicHub Idea Brainstorm (115 ideas)

Skipped as already covered: cookie ingestion → P1#1, wayback → P8#5, main-char picker → P1#4, semantic-cold-start → P8#32, annotations → P8#20, batch export + queue → P8#1–2, e-reader → IDEAS §1, EPUB styling → P8#3, OPDS → shipped, endpoint dashboard → P4#13, backup → USER-ACTIONS #3, blocks → P8#59, status page → P8#46/#70, secrets/uptime → USER-ACTIONS #1–2, taste profile → P8#29.

Skipped as wild/niche: OCR screenshot ingest, email-to-FicHub, ingestion quests, voice search, parallel view, spice slider (part of #76), reading-buddy matcher, fic debates, native writing mode, beta-reader matching, co-authoring, Anki export, federation, most of §11 delight.

| # | Feature | What exists | Verdict |
|---|---------|-------------|---------|
| 73 | Browser-extension capture (starred) — user opens blocked fic in own browser, extension POSTs HTML to FicHub, existing adapters parse it; sidesteps AO3/FFN host block entirely; complements P1#1 cookie flow | No impl | *Spec first* |
| 74 | "Paste the HTML" manual ingest — textarea for raw page, parsed server-side; zero-dependency version of #73; no extension install | No impl | *Cheap win* |
| 75 | Semantic/meaning search (starred) — NL query embedded against fic vectors (nomic-embed-text already resident for rec platform); distinct from Ask-the-Archive (LLM → boolean filters); this is embedding-to-embedding over pgvector | No impl | *Spec first* |
| 76 | Content shield + spice controls — collapse explicit/gory passages behind tap (warnings + LLM triage); global cap on explicitness in recs | No impl | *Spec first* |
| 77 | Quote cards (starred) — render favorite line as shareable image; builds on shipped body FTS (`/search/body` + `<mark>` snippets); natural viral surface | No impl | *Cheap win* |
| 78 | Tag wiki — community definitions of tropes linked from every tag chip; complements admin tag-wrangling (#18), curator-approve entries | No impl | *Spec first* |
| 79 | Fandom landing pages — auto-generated per-fandom hub: stats, top fics, trending ships, recent activity | **SHIPPED** 2026-08-17 (commit 21a0440) | *Ship now* |
| 80 | Obsidian/Markdown export with frontmatter — tags, dates, metadata embedded; one fic → vault-ready .md | No impl | *Cheap win* |
| 81 | Print-ready book PDF — real book layout (covers, margins), not converted EPUB | No impl | *Spec first* |
| 82 | Public archive stats page — growth curves, top fandoms, exports/day; zero-PII over usage_events; distinct from admin endpoint dashboard (P4#13) | No impl | *Cheap win* |
| 83 | Trope popularity over time + fandom health index — trend graphs per tag; which fandoms grow/die by ingest + read rates | No impl | *Spec first* |
| 84 | Metadata open dataset — monthly dump of metadata (not bodies) for researchers/tooling | No impl | *Cheap win* |
| 85 | Self-host one-click compose (starred) — Docker compose with sane defaults; no Docker on current host, but adoption lever (fichub.net replacement = instances) | No Docker on host | *Big bet* |
| 86 | Dead-dove detector — LLM scan (Ollama + body cache) for content contradicting tags | No impl | *Spec first* |
| 87 | Age gate for is_adult content — scrapers already carry flag; enforce in UI | Flag exists | *Cheap win* |
| 88 | API tokens for third parties — scoped keys so external tools build on FicHub (complements P8#8 CORS) | No impl | *Spec first* |
| 89 | CLI — `fichub get <url> --format epub` for power users | No impl | *Cheap win* |
| 90 | Chat bots (Discord/Telegram) + webhooks — search, lookup, "send me this fic"; fic-updated events to n8n | No impl | *Spec first* |
| 91 | i18n key system (starred) — 6-locale i18n shipped (en/de/es/fr/pt-BR/zh) via frontend/src/lib/i18n with per-locale dictionaries (~45K entries each), LocaleSelector component in layout + archive footer, localStorage persistence (fichub_locale), browser-language detection, backend PUT /api/auth/locale flow; UI text live-translated across routed pages; remaining work is key-coverage completion and per-fic language metadata | Shipped i18n subsystem | *Cheap win* (coverage completion) |
| 92 | WCAG audit + RTL end-to-end — keyboard nav, screen-reader labels, contrast; settings store already has useRtl | No impl | *Spec first* |
| 93 | Fic-to-podcast RSS — per-fic audio feed where each chapter is episode (TTS) | No impl | *Spec first* |
| 94 | Prompt calendar + flash-fic challenges — daily/weekly prompts feeding Fic Requests; weekly contests with arena voting | No impl | *Spec first* |
| 95 | RAM-aware concurrency governor — track resident memory (RSS) + Ollama load; decide whether another LLM (chat, embeddings, dead-dove scans) can run in parallel or must degrade gracefully; Ollama models resource-heavy (lfm2.5:8b resident; nomic-embed-text); multi-user Discord bots / rec engines can OOM box; probe: cgroup v2 memory.current + memory.max (or /proc/meminfo), plus Ollama /api/ps for loaded models; expose `GET /api/health/resource` so Discord bot can pre-check before spawning LLM call | No impl | *Cheap win* |
| 96 | Fic-level moods (starred) — classify each work's tone/mood on small axis (Neutral/Funny/Shocky/Flirty/Dramatic/Hurty/Bondy), not author-level; computed from content signals: genre ratios, description tone, review/kudos sentiment, tag mix; unlike author-moods (which describe author's average style), fic-moods describe story itself; feeds rec engine (mood-similarity) + search facets + per-fic "mood" chip | No impl | *Spec first* |
| 97 | Author recommendations (starred) — recommend authors similar to ones I like, or whose works fit my taste; today website recommends fics only; no "authors like X" surface; build: author feature vectors from works (genre mix, mood distribution, fandom spread, popularity band) → author-similarity ranking; plus "authors who wrote fic I've kudos'd/bookmarked" as implicit taste signal; deliver: `/api/recommendations/authors?q=<author-or-fic>` + "Authors you might like" panel + Discord /authors command | No impl | *Spec first* |
| 98 | Rarity-tier weighted rec strategy — per-author overlap ratio + matches + sigma, then weight authors by rarity tier (unique 0.2×matches, rare 0.05×, uncommon 0.005×, common 1); rare overlap with big-list author scores higher than popular author with one common match; explainable ("you share 5 faves with this author whose list is 90% rare fic") + powers !sus-style list-manipulation detection; add as pluggable weighted strategy beside mf/embeddings | No impl | *Spec first* |
| 99 | Audience-genre inference (recommend-by-who-likes-it) — compute each fic's genre profile of people who recommend it (kudos/bookmark graph): "this fic is funny to humor-lovers", not just "declared genre"; cheap SQL over kudos/bookmarks; no ML; powers "recommend me funny fic" + per-fic "audience tastes" signal + genre facets | No impl | *Cheap win* |
| 100 | Explorer / size / popularity bands as filters — first-class search/rec filters: popularity band (barely-known / relatively-unknown / popular), size class (small <=20k / medium <=100k / big <=400k / huge), plus per-list ratios (explorer ratio, crossover ratio, unfinished ratio, mature ratio, fandom-diversity); unlocks "obscure gems" (!gems), "short completed fic", etc | No impl | *Cheap win* |
| 101 | Tracked-message bot state machines — Zeks/flipper Discord bot persists state machine per message (tracked_recommendation_list, tracked_roll, tracked_review, tracked_fic_details, tracked_similarity_list, tracked_help_page, tracked_delete_confirmation), so commands mutate that message in place (re-roll same list, next/prev by message reference, delete-confirm); more capable than current button rows; port to fanfic-archivist | No impl | *Spec first* |
| 102 | Missing bot commands from Flipper — !sus (suspicious-author detection / list-bombing), !stats (per-user profile stats), !fresh (recency-gated), !filters (filter rec stream by liked authors / forced params / reset), !full-favourites (browse whole list), !cutoff/!wordcount/!year (range tuning); add to fanfic-archivist | No impl | *Spec first* |
| 103 | Author list-manipulation / abuse detection (!sus) — flag authors whose fav-list looks gamed (anomalous overlap, ratio outliers, suspicious vote patterns); surface on author pages + Discord; pairs with rarity-tier weighting | No impl | *Spec first* |
| 104 | Per-author profile stats — aggregate per-author: wordcount/size distribution, fandom diversity, crossover ratio, mood uniformity, popularity band, "explorer" tendency; powers author pages + !stats + author-recs signal | No impl | *Cheap win* |
| 105 | Circuit breaker for dead user sidecars (fallback to cooccur) | No impl | *Cheap win* |
| 106 | Per-surface rec recipes (home vs fic-page vs Next-Up each use different blends) | No impl | *Cheap win* |
| 107 | Rec subscription (follow another user's recipe, auto-propagate updates) | No impl | *Cheap win* |
| 108 | Theme remixing (fork shared presets, tweak, re-share) | No impl | *Cheap win* |
| 109 | Reduced-motion accessibility theme preset | No impl | *Cheap win* |
| 110 | Seasonal event themes (unlock temporarily, collectible achievements) | No impl | *Spec first* |
| 111 | Custom shelf colors and icons (visual library organization) | No impl | *Cheap win* |
| 112 | Command palette for customization (universal launcher for saved views, themes, recipes) | No impl | *Cheap win* |
| 113 | User-remappable keyboard shortcuts (exportable as shareable profiles) | No impl | *Cheap win* |
| 114 | Skip locked widgets when installing shared layouts | No impl | *Cheap win* |
| 115 | Reset-to-default layout button | No impl | *Cheap win* |
| 116 | Blind-date as unlockable widget (surfaces random hidden gems) | No impl | *Cheap win* |
| 117 | Forum posting level gate (anti-spam for fresh accounts) — configurable level threshold, default L3 | No impl | *Cheap win* |
| 118 | Daily quests + reading streaks for XP (gamified engagement loop) — note: daily-quest gamification loop scrapped in badge-wiring fix; reading-history + streak system kept; this is the recommitment of streaks as XP loop | Progression system exists | *Spec first* |
| 119 | Rec explainability (strategy name + reason on every recommendation card) | In recommender; not surfaced | *Cheap win* |
| 120 | People-like-you row (taste clusters → explain by group) | No impl | *Spec first* |
| 121 | Curator prior personal slider (user-adjustable taste shaping) | No impl | *Cheap win* |
| 122 | Customization profile export (one portable JSON backup of all prefs) | No impl | *Cheap win* |
| 123 | Feature toggle API (power users script setup via API tokens) | No impl | *Cheap win* |
| 124 | Curator-verified badge in rec marketplace — curator review → verified flag on extensions | No impl | *Cheap win* |
| 125 | Level gate + curator review for publishing themes/layouts — extends recipe publish gate to themes/layouts | No impl | *Cheap win* |
| 126 | Widget marketplace with live previews (rendered from layout JSON) | No impl | *Spec first* |
| 127 | Rec shown-to-engaged ratio as social proof per shared recipe — impressions → engagement ratio from rec_impressions | No impl | *Cheap win* |
| 128 | Per-feature notification granularity (mute what you disabled) | No impl | *Cheap win* |

### Fastest OTW wins (cheap, high fit): #52 kudos (one-click), #60 username history (admin table), #64 fannish next-of-kin (data model), #59 user-level mute (extends existing blocks), #61 reading history (extends #21), #67 announcement banner, #68 per-fic hit counter, #72 per-fic language metadata.

### Fastest to ship (user-flagged cheapest wins): #41, #43, #45, #50 (single queries over existing tables), #27 (already in recommender), #4 (stated P1).

---

## 47.10 Admin dashboard & engagement metrics

| # | Feature | What exists | Verdict |
|---|---------|-------------|---------|
| 41 | Per-endpoint usage breakdown — P4#13 (one GROUP BY over usage_events) | `usage_events` exists | *Cheap win* |
| 42 | Conversion funnel chart (visit → search → read → bookmark → export) | No impl | *Spec first* |
| 43 | Zero-result query report (top failing queries → ingest) — P4#12 data exists; P8 — surface the report | `search_queries` exists | *Cheap win* |
| 44 | Cohort retention (weekly signup cohorts, return rate) | No impl | *Spec first* |
| 45 | Bot vs human traffic split (time series from bot_scores) | `bot_scores` table exists | *Cheap win* |
| 46 | Scraper health board (failures by domain × class + agent run log) | `scrape_failures`/`agent_runs` exist | *Cheap win* |
| 47 | Content coverage report (fics by site/fandom/word-count bucket) | No impl | *Spec first* |
| 48 | Search→export conversion trend (extend view into weekly trend) | P4#12 data exists | *Cheap win* |
| 49 | Modlog diff view (before/after values on metadata edits) | `modlog` exists | *Cheap win* |

---

## 47.11 P9 — Extension platform (v3.1) deferred

| # | Feature | Why deferred |
|---|---------|-------------|
| 51 | Multi-tenant sidecars (user-hosted rec engines) — needs trust system + outbound HTTP safety; compute on user's server | Trust + outbound HTTP not solved |
| 52 | WASM sandbox for user scoring functions — heavy dependency (wasmtime); fuel+memory metering needed | Heavy dependency |
| 53 | Full marketplace with ratings/reviews + engagement leaderboard — needs user base first; shadow-run engagement as gate | Needs user base |
| 54 | Nav/layout editor (drag-drop reorder) — partially built; iterate on existing widget dashboard | Already partially done |
| 55 | Layout presets (mobile vs desktop variants) — nice but low priority; users can save multiple layouts via user_views | Low priority |
| 56 | Community recommender leaderboard (Elo from rec_impressions) — needs rec_impressions data to accumulate first | Needs data |
| 57 | Recipe → precomputed promotion (batch compute popular shared recipes) — only when shared recipes get popular enough to warrant batch compute | Needs popularity |
| 58 | XP from customization reputation ("your theme installed 50 times") — makes customization itself a progression loop; needs marketplace first | Needs marketplace |
| 129 | Seasonal event themes with time-gated unlocks + collectible achievements — needs progression loop proven first; engagement hook for later | Needs proven loop |
| 130 | Full marketplace with curator review pipeline + verified badges — needs user base + rec_impressions data; curator review is heavy | Needs user base + data |

> **💡 Key Concept**: P9 isn't "stuff we can't build." It's "stuff we shouldn't build yet because the foundation isn't there." The extension marketplace (P7) is the foundation for most of these. Don't try P9 items before P7 is stable.

---

## 47.12 Wattpad-style feature audit (Group A/B/C)

### A. Align as-is

1. **Inline paragraph commenting in web reader** — Marginalia already covers this (passage-hash anchored forum threads); gap: Wattpad UI floats reaction toolbar on text selection; consider adding lightweight reaction shortcut to existing highlight tooltip. *Cheap win*.

2. **Author-facing analytics dashboard** — `/api/analytics` + `/api/admin/endpoint-usage` cover system usage; extend with per-work view/read/download/engagement metrics (read-count by chapter, avg time, completion rate); curator-level visibility (role ≥ 5). *Spec first*.

3. **Personalized reading stats dashboard** — FicHub already tracks `reading_history`/`reading_status` + bookmarks/ratings; surface `/user/{id}/stats` page: total words read (sum chapter word counts), favorite genres (tag frequency), streak (days active), shelves breakdown; leverages existing `usage_events` + `user_prefs`. *Cheap win*.

4. **Community writing contests / prompt challenges** — map to forum + curator consensus: contest proposals → `/api/curator/consensus` proposal → community submissions via fic_request_answers or new contest_entries table → voting via existing request votes; spec structured submission form (title, word count, required tags) reusing upload pipeline. *Spec first*.

### B. Adapt (keep value, reshape to FicHub norms)

5. **Algorithmic infinite-scroll home feed** — FicHub deliberately uses 4-level fallback (personal → trending → popular → recently added), not dopamine-maximizing TikTok feed; adapt: add infinite scroll pagination to existing HomeDashboard left column, driven by same ranked query — continuous discovery without engagement-bait psychology. *Cheap win*.

6. **Built-in drafting + scheduled publishing** — scope to "write" area: Markdown editor persisting drafts to `user_work_drafts` table (reuse existing upload/convert path) with future `publish_at` timestamp; publish creates fic_request (community curation entry) or curated works row (curator-only); respect archive skin. *Spec first*.

### C. Skip (violates FicHub house policy)

7. **Spendable virtual currency / paywalled premium chapters** — FicHub is donation-financed (BUY_ME_A_COFFEE link on `/donate`), not walled marketplace; monetizing reader attention or gating chapters behind pay contradicts archive-everything ethos; do not implement. **No**.

8. **Private direct messaging between users** — FicHub has public-by-default forums (moderated via modlog-style tools); private DMs invite toxic behavior with no public record; skip; point users to curator-flagged public forum threads instead. **No**.

9. **Public downvoting / negative engagement metrics on frontend** — FicHub uses positive-only karma (kudos/notifications are +1 style); negative public voting enables brigading and drive-by trolling; skip; if reaction feedback needed, expand emoji-reaction picker (Group A) instead. **No**.

10. **Walled-garden mobile app exclusivity** — FicHub is deliberately open-web (OPDS 2.0, PWA-installable, no app store gate); mobile app restricting web access contradicts this; skip; if mobile UX boost needed, ship installable-PWA improvements instead. **No**.

---

## 47.13 Cheapest wins cheat-sheet (one page at end of Part 49)

- **#41** per-endpoint usage breakdown — one GROUP BY over `usage_events`
- **#43** zero-result query report — surface existing P4#12 data
- **#45** bot vs human traffic split — time series from `bot_scores`
- **#50** feature adoption tracker — path-prefix over `usage_events`
- **#27** rec explainability — already in recommender
- **#4** cookie ingestion — stated P1 (product decision)
- **#74** paste HTML — zero-dependency manual ingest
- **#82** public stats page — one GROUP BY over `usage_events`
- **#87** age gate — flag already carried
- **#89** CLI — thin wrapper over existing endpoints
- **#91** i18n key extraction — mechanical, unlocks #92
- **#60** username history — admin table
- **#64** fannish next-of-kin — data model
- **#67** announcement banner — cheap
- **#68** per-fic hit counter — social proof; cheap (Redis counter)
- **#72** per-fic language metadata — cheap

> **💡 Key Concept**: These are single-query or zero-dep items. If you want to ship something this weekend, pick one of these. They're the "do this now, think about the rest later" list.

---

## 47.14 What you'd build next — the prioritised backlog

Looking at the full table above, here is how you'd sequence the next 5 features if you were shipping tomorrow:

1. **#43 zero-result query report** — surface existing data. One panel on the admin page. Tells you what to ingest next.
2. **#68 per-fic hit counter** — Redis counter. Social proof on fic cards. Makes FicHub feel alive.
3. **#67 announcement banner** — dismissible, cached. Lets you communicate outages or events.
4. **#64 fannish next-of-kin** — one data model, one settings page. Meaningful trust feature for a community archive.
5. **#60 username history** — admin table. Identity forensics for ban evasion.

These 5 form a narrative: surface what's broken (#1), show engagement (#2), communicate (#3), deepen trust (#4), harden security (#5). Ship them in that order and you've addressed the most visible gaps without building anything big.

> **⚠️ Watch Out**: Don't chase feature count. A platform with 10 well-chosen features feels more complete than one with 50 half-built ones. FicHub already has 46 shipped parts. The backlog isn't a to-do list — it's a map. Pick one cheap win, ship it, then re-evaluate.

---

End of Part 47 — Roadmap Deep-Dive. The codebase you built in Parts 1–46 is the foundation. The roadmap is the frontier. Pick a cheap win first — you've earned it.