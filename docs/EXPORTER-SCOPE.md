# ficnexus is an exporter

**Status: implemented.** This document is the record of the reduction: what
ficnexus is now, what was removed and which code carried it, what stayed and
why, and which database tables are now unused (they still exist — nothing was
dropped).

The one-line version: **give it a story URL, get a downloadable file.**
fichub.net-shaped. No account, no library, no forum, no feed.

---

## 1. What the service does

```
client ──GET /api/epub?q=<story url>──► scrape (fanfic-scrapers, 100 adapters)
                                      ► build EPUB (pure Rust) + HTML/TXT/MD
                                      ► snapshot body + export ledger (Postgres)
                                      ► JSON { meta, urls: {epub: /cache/…} }
client ──GET /cache/epub/<id>?h=<md5>─► the file, md5 re-validated
```

Everything a downloader needs and nothing else:

* 100 native site scrapers (`crates/scrapers`) — AO3, FFN, FictionPress,
  RoyalRoad, SpaceBattles, XenForo boards, eFiction family, WordPress-novel
  sites, adult-gated archives;
* document conversion: EPUB, HTML bundle, TXT, Markdown, FB2, KEPUB, DOCX in
  pure Rust; MOBI/PDF/AZW3 via the Calibre sidecar container;
* a fetch/export cache on disk plus a Postgres ledger (`fic_info`,
  `export_log`) so a repeat export is a cache hit, not a re-scrape;
* self-healing scrape telemetry and the Wayback Machine fallback;
* tiered Redis rate limiting, datacenter-ASN blocking and the proof-of-work
  gate for flagged clients;
* health checks.

Two processes: the HTTP server (`fichub`) and `migrate`. No workers, no cron
loops, no SPA build.

---

## 2. The route table

### 2.1 Before (baseline, commit `c3a67cb`)

486 route entries across 88 handler modules, mounted by 15 `chunk_*` builders
in `src/server.rs` (2,453 lines). Counting entries per handler module:

| routes | module | examples |
|---:|---|---|
| 49 | `routes::forum` | `/api/forum/topics`, `/api/forum/posts/{id}/moderate`, `/api/admin/forum/bans` |
| 39 | `routes::admin` | `/api/admin/moderation/queue`, `/api/admin/content-scan`, `/api/admin/bulk/*` |
| 22 | `routes::social` | `/api/auth/register`, `/api/auth/login`, `/api/auth/me`, `/api/bookmarks` |
| 21 | `routes::collections` | `/api/collections`, `/api/curator/approvals` |
| 17 | `recommender::*` | `/api/recommendations`, `/api/recommendations/train` |
| 13 | `routes::opds` | `/opds`, `/opds/popular` |
| 11 | `routes::authors` | `/api/authors/search`, `/api/authors/{id}` |
| 10 | `routes::requests` | `/api/requests`, `/api/requests/{id}/answers` |
| 9 each | `recipes`, `subsystems`, `quests`, `features` | `/api/recipes`, `/api/admin/invites`, `/api/reading/status`, `/api/features/{slug}/enable` |
| 8 each | `curator_content`, `follows`, `forum_groups`, `pseuds`, `skins`, `roadmap`, `tags::curator` | `/api/follows`, `/api/forum/groups`, `/api/roadmap/arena`, `/api/curator/alias` |
| 7 each | `lists`, `reports` | `/api/lists`, `/api/admin/reports` |
| 6 each | `device_library`, `saved_search`, `notifications`, `proposals`, `shelves`, `reader`, `locales`, `analytics` | `/api/reader/{url_id}`, `/api/notifications`, `/api/shelves` |
| 5 each | `search`, `bounties`, `user_preferences`, `forum_polls`, `comments`, `auto_tag`, `redirects`, `server` | `/api/search`, `/api/comment/{id}`, `/api/remote` |
| 4 each | `activitypub::actors`, `translate`, `badges`, `trust`, `heal` | `/actor`, `/api/translate/batch`, `/api/admin/trust` |
| 3 each | `download`, `reviews`, `user_export`, `user_credentials`, `forum_privileges`, `messages`, `updates`, `trending`, `upload`, `rss`, `bulk`, `customization`, `uploads` | `/api/download/author`, `/api/reviews`, `/feed.xml`, `/ws` |
| ≤2 each | `export`, `find_fic`, `cache_download`, `sitemap`, `pow`, `series`, `blind`, `reactions`, `leaderboard`, `drafts`, `docs`, `fandom`, `backfill`, `kindle`, `modlog`, `realtime`, `consensus`, `marginalia_admin`, `work_delete`, `api_docs`, `meta`, `health`, `user_formats` | `/api/epub`, `/api/meta`, `/api/health`, `/api/formats` |

### 2.2 After

`grep -n '\.route(' src/server.rs` — 11 entries, all in `chunk_export()`:

| method | path | why it stays |
|---|---|---|
| GET | `/` | the whole UI: URL box + format picker |
| GET | `/api/` | machine-readable description of the surface |
| GET | `/api/epub` | **the** export endpoint |
| GET | `/api/epub/convert` | export in a named format |
| GET | `/api/meta` | metadata preview before downloading |
| GET | `/api/health` | DB + Redis health |
| GET | `/api/formats` | supported output formats |
| GET | `/cache/{etype}/{url_id}/{fname}` | serve a built file (md5 in filename) |
| GET | `/cache/{etype}/{url_id}` | serve a built file (`?h=`), else explain |
| GET | `/api/pow/challenge` | anti-bot: challenge |
| POST | `/api/pow/solve` | anti-bot: solution |

Verified by `tests/exporter_api.rs`: `exporter_routes_are_mounted` asserts every
one of those answers non-404, and `community_routes_are_gone` asserts 21
representative old paths (`/api/auth/login`, `/api/bookmarks`, `/api/forum/topics`,
`/api/recommendations`, `/api/search`, `/feed.xml`, `/opds`, …) answer 404.

---

## 3. What was removed, and what carried it

Every item below duplicated darkhold. Removing it meant deleting the route
module, its service/support modules, its CLI binaries and its tests — not
hiding it behind a flag.

| subsystem | route module(s) | backing code deleted |
|---|---|---|
| accounts, sessions, JWT auth, roles | `routes::social` (22), `routes::user_export`, `routes::user_credentials`, `routes::user_preferences` | `services::trust`, `auth` extractor, `jsonwebtoken`/`bcrypt`/`axum-extra`/`aes-gcm` deps, `SiteCredRow` |
| bookmarks, ratings, kudos, reviews, comments, reactions | `routes::comments`, `routes::reviews`, `routes::reactions`, `routes::social` | `db/reviews.rs`, `db::queries::social`, `db::queries::prefs` |
| reading lists, shelves, collections, series/author pages | `routes::lists`, `routes::shelves`, `routes::collections`, `routes::series`, `routes::authors`, `routes::authors_fuzzy`, `routes::pseuds`, `routes::creatorships` | `db::queries::reputation` |
| follows + updates feed, notifications | `routes::follows`, `routes::updates`, `routes::notifications`, `routes::feed` | `follow_*` / `notify_*` query fns |
| fic requests (prompt board) | `routes::requests` | `db::queries::proposals` |
| forum (boards, posts, moderation points, metamoderation, leveling, polls, groups, privileges, DMs, drafts, uploads) | `routes::forum` (49), `forum_groups`, `forum_polls`, `forum_privileges`, `messages`, `drafts`, `uploads`, `reports`, `modlog` | `crates/forum-core`, `crates/forum-import`, `services::comment_triage` |
| progression: XP, 100 levels, ability tree, quests, badges, streaks, leaderboards | `routes::features`, `routes::quests`, `routes::badges`, `routes::leaderboard` | `services::leveling`, `services::achievements`, `leaderboard/`, `src/bin/{compute_stats,compute_leaderboards,backfill_scores,bot_scorer}.rs` |
| recommender platform (strategies, embeddings, bandit, recipes, entity recs) | `routes::recipes` (9), `recommender::*` (17), `routes::trending`, `routes::blind` | `src/recommender/`, `rec-engines/`, `scripts/ml/`, `services::{recipes,embedding_dedupe,search_mining}`, `src/bin/seed_roadmap.rs` |
| roadmap consensus / MaxDiff arena | `routes::roadmap`, `routes::consensus` | `crates/consensus`, `roadmap_seed.rs`, `services::ollama` embed use |
| marketplace: themes, skins, layouts, views, extensions, recipes gallery | `routes::skins`, `routes::extensions`, `routes::customization` | `services::{customization,extensions}` |
| search: advanced, body FTS, ask-the-archive, suggestions, saved searches + watcher | `routes::search`, `routes::saved_search`, `routes::fandom`, `routes::find_fic` | `src/search/` (12 files), `src/bin/{backfill_body_search,saved_search_watcher}.rs` |
| in-browser reader, PWA, OPDS, RSS, sitemaps, Send-to-Kindle, uploads, copyright/DMCA, translation, ActivityPub, realtime, usage analytics, admin UI, `/api/remote`, `/changes` | `routes::reader`, `opds/`, `rss/`, `sitemap`, `kindle`, `upload`, `uploads`, `copyright`, `translate`, `locales`, `docs`, `analytics`, `subsystems`, `admin`, `curator_content`, `proposals`, `work_delete`, `work_proposals`, `bounties`, `backfill`, `auto_tag`, `marginalia_admin`, `heal`, `bulk`, `redirects`, `blind` | `src/activitypub/`, `src/realtime/`, `src/visitor/`, `src/modlog.rs`, `src/meta_store.rs`, `src/fic_suggestions.rs`, `services::{translation,mailer,bookmark_import,content_scan,proposals,bounties,auto_tagger}`, `src/bin/{trust_promote,weekly_digest,publish_scheduled}.rs` |
| batch download (author/series streams) | `routes::download` | depended on per-user site credentials, i.e. on accounts |
| the SvelteKit SPA and its QA harness | — | `frontend/` (23 MB), `qa/`, `qa.sh`, CI frontend job |

Deleted outright: 74 route files, 16 of 18 services, 5 of 7 query modules,
5 whole module trees (`activitypub`, `realtime`, `recommender`, `visitor`,
`search`), 3 workspace crates, 10 CLI binaries, 61 integration test files,
the entire frontend.

Kept as data, not code: `migrations/` (45 files, untouched) and
`crates/forum-core/migrations/*.sql` — the record of how the tables were
created.

---

## 4. What stayed, and why

| kept | because |
|---|---|
| `crates/scrapers` (100 adapters) | the only way a URL becomes a story |
| `src/scrape/` (registry, wayback, quality classifier, author_link) | discovery, fallback, junk rejection |
| `src/export/` (epub, html_bundle, txt, md, fb2, kepub, docx, convert, fallback) | the file the user came for |
| `src/cache/` + `src/body_cache.rs` | instant repeat exports; the "site as cache" idea |
| `src/limiter/` (tiered Redis buckets, datacenter ASN blocking) | anti-bot defence |
| `src/services/pow.rs` + `routes/pow.rs` | proof-of-work gate for shadowbanned clients |
| `src/heal/` | scrape-failure telemetry that keeps the export path honest |
| `src/works/` + `db::queries::{core,tags,works}` | bookkeeping: which URL maps to which work, which file is cached |
| `src/tags/resolve.rs` | a scraped work's canonical tags are part of the metadata |
| `src/db/` (pool, migrations at start-up, models) | the export ledger and cache index |
| `routes::{export,meta,health,cache_download,formats,api_docs,landing,pow}` | the 11 routes above |
| `src/bin/migrate.rs` | applying the schema |

`AppState` shrank from 17 fields to 11 (`config`, `db`, `redis`,
`health_redis`, `http_client`, `scraper_registry`, `cache_semaphores`,
`rate_limiter`, `heal`, `wayback`, `ollama`). `run()` no longer spawns any
background loop.

---

## 5. Database: nothing dropped, 164 tables now unused

`migrations/` is untouched: 45 files, 180 tables. No `DROP TABLE`, no deleted
migration, no rewritten history. Deployed databases keep their data; they just
have 164 tables nothing reads any more.

Method: every SQL string literal in `src/` was parsed for
`FROM|INTO|UPDATE|JOIN <table>`; the 16 tables named are below. The other 164
are not named in any SQL the remaining code executes. (Row counts and FKs are
unchanged; a table can still be touched indirectly by a trigger defined in a
migration — that is why they are left in place rather than dropped.)

**Still used by the export path (16):**

`agent_runs`, `author_blacklist`, `auto_merge_log`, `export_log`, `fic_blacklist`,
`fic_info`, `fic_tags`, `fic_version_bump`, `heal_extractions`, `pending_exports`,
`request_log`, `request_source`, `scrape_failures`, `tag_aliases`, `tags`, `works`

**Unused (164).** Grouped by the subsystem that used them:

- **Forum (28):** `forum_topics`, `forum_posts`, `forum_messages`, `forum_drafts`,
  `forum_groups`, `forum_group_members`, `forum_polls`, `forum_poll_options`,
  `forum_poll_votes`, `forum_privileges`, `forum_tags`, `forum_topic_tags`,
  `forum_topic_views`, `forum_follows`, `forum_read_state`, `forum_rooms`,
  `forum_room_members`, `forum_bans`, `forum_mod_actions`, `forum_edit_proposals`,
  `forum_metamod_votes`, `forum_post_reactions`, `forum_notifications`,
  `forum_uploads`, `forum_user_blocks`, `forum_user_prefs`, `forum_categories`
- **Accounts, social, curation (42):** `users`, `bookmarks`, `fic_bookmarks`,
  `kudos`, `comments`, `comment_triage`, `reviews`, `work_ratings`,
  `work_rating_verifications`, `user_reports`, `follows`, `follow_exclusions`,
  `notifications`, `notification_preferences`, `reading_lists`,
  `reading_list_items`, `reading_history`, `reading_stats`, `shelves`,
  `work_shelves`, `series`, `series_works`, `pseuds`, `creatorships`,
  `blocked_users`, `user_consents`, `visitor_state`, `user_prefs`, `usage_events`,
  `reputation_events`, `trust_events`, `modlog`, `site_settings`, `proposals`,
  `proposal_votes`, `fic_tag_votes`, `tag_flags`, `tag_score_fixes`,
  `curator_quorum_proposals`, `content_scan`, `reactions`, `admin_daily_stats`
- **Progression (17):** `xp_events`, `xp_source_defs`, `xp_sources_streaks`,
  `exp_events`, `user_daily_progress`, `daily_quests`, `challenges`,
  `challenge_assignments`, `challenge_signups`, `login_streaks`,
  `badge_definitions`, `user_badges`, `leaderboard_weekly`,
  `leaderboard_monthly`, `user_features`, `user_layouts`, `user_views`
- **Recommender & roadmap (21):** `rec_author_graph`, `rec_bandit_arms`,
  `rec_embeddings`, `rec_impressions`, `rec_models`, `rec_training_runs`,
  `rec_transitions`, `rec_user_clusters`, `rec_user_curator_align`,
  `rec_user_signals`, `precomputed_recommendations`,
  `recommendation_suggestions`, `recommendation_votes`, `fic_bookmark_cooccur`,
  `tag_embeddings`, `feature_clusters`, `feature_suggestions`, `arena_votes`,
  `roadmap_changelog`, `ask_translation_cache`, `fic_works`
- **Requests, collections, marketplace, translations, OPDS (33):**
  `fic_requests`, `fic_request_answers`, `fic_request_answer_votes`,
  `fic_request_upvotes`, `collection_bookmarks`, `collection_item_requests`,
  `collection_submission_votes`, `extensions`, `extension_installs`,
  `extension_ratings`, `skins`, `user_skins`, `work_skins`, `user_recipes`,
  `user_invites`, `registration_applications`, `translations`,
  `translation_strings`, `chapter_translations`,
  `chapter_translation_versions`, `work_translations`, `locales`, `opds_shelves`,
  `opds_shelf_items`, `marginalia`, `doc_sections`, `copyright_notices`,
  `saved_searches`, `saved_search_matches`, `search_queries`, `bot_scores`,
  `bounty_pots`, `bounty_resolutions`
- **ActivityPub, author pages and leftovers (23):** `ap_keys`, `ap_inbox_log`,
  `ap_outbox`, `ap_remote_actors`, `author_profiles`, `author_profile_links`,
  `author_socials`, `author_social_proposals`, `author_social_votes`,
  `author_merge_proposals`, `curator_content_overrides`, `curator_fix_proposals`,
  `curator_fix_votes`, `curator_metadata_proposals`, `curator_metadata_votes`,
  `work_proposals`, `work_proposal_votes`, `work_delete_proposals`, `features`,
  `tag_types`, `user_favorite_tags`, `user_site_credentials`,
  `schema_migrations`

(`schema_migrations` is a legacy table; sqlx's own `_sqlx_migrations` is still
written by the migrator at start-up and is not counted in the 164.)

## 6. How this was verified

Everything below was executed in this repo, on this host, in this order — each
removal followed by a build, and the final state re-run end to end:

```text
cargo build                                  # clean, 0 warnings
cargo test                                   # 270 lib tests + 16 integration tests
                                             # (5 of them DB-gated, ignored)
cargo test --test exporter_api -- --include-ignored --test-threads=1
                                             # 16/16, incl. cached_export_is_served_as_a_file
                                             # and epub_is_built_from_chapters
```

`epub_is_built_from_chapters` builds a real EPUB from two chapters and asserts
it is a zip whose first entry is `mimetype` and whose package document exists.

Live server (Postgres 18 + Redis, scratch database built from `migrations/`):

```text
$ curl -s localhost:8099/api/health
{"status":"ok","db":true,"redis":true,"version":"0.3.0"}

$ curl -s 'localhost:8099/api/epub?q=deadexport01'
{"err":0,"url_id":"deadexport01",
 "epub_url":"/cache/epub/deadexport01?h=f459395b…",
 "hashes":{"epub":"f459395b…"},"info":"Live Curl Export by tester\n900 words…"}

$ curl -sD- -o out.epub 'localhost:8099/cache/epub/deadexport01?h=f459395b…'
HTTP/1.1 200 OK
content-type: application/epub+zip
content-disposition: attachment; filename="deadexport01.epub"
content-length: 46
# sha256-free proof: md5 of the served bytes == the hash in the URL

$ for p in /api/bookmarks /api/forum/topics /api/auth/me /api/recommendations \
           /api/search /feed.xml /opds /api/lists /api/recipes /api/requests; do …
/api/bookmarks 404   /api/forum/topics 404   /api/auth/me 404
/api/recommendations 404   /api/search 404   /feed.xml 404
/opds 404   /api/lists 404   /api/recipes 404   /api/requests 404
```

---

## 7. Left undone, and why

1. **A live scrape was not exercised.** This host has no outbound internet
   (TLS to `example.com` and `archiveofourown.org` is reset by the network), so
   `GET /api/epub?q=https://…` could not reach a real site. The scrape leg is
   therefore unverified here; the build leg and the serve leg are both covered
   by tests. Everything after the fetch is exercised by
   `cached_export_is_served_as_a_file`.
2. **Config still carries dead knobs.** 176 `Config` fields, of which ~130
   belong to removed subsystems (`REC_*`, `FORUM_*`, `TRANSLATE_*`,
   `PROPOSAL_*`, `SMTP_*`, …). They are inert — nothing reads them — but
   removing them means rewriting `Config::from_env` and its 7 test blocks. Only
   `FRONTEND_DIR` was removed in this change. This is the largest piece of
   cosmetic debt left.
3. **`src/db/models.rs` still defines community structs** (`Bookmark`,
   `ForumPost`, `XpEvent`, …). They are unused types over unused tables; they
   compile and cost nothing at runtime.
4. **`docs/` is historical.** The 2.4 MB of plan/spec files under `docs/` and
   `specs/` describe subsystems that no longer exist here (forum spec, roadmap,
   trust levels, the 56-suite test matrix). They are left as history; this file
   supersedes them for scope.
5. **No `DROP TABLE`, by design.** The 164 unused tables keep their data in
   deployed databases. If that data is ever worth reclaiming, it is a separate,
   deliberate migration — not part of turning a service off.
6. **darkhold is untouched.** No file outside this repository was modified.
