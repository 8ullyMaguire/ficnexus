# FicHub Advanced Search v2 — Comprehensive Plan

> **For Hermes:** Use the `fichub-development` + `subagent-driven-development` skills to implement this plan batch-by-batch. Canonical plan doc: `docs/brainstorm-02-roadmap-status.md` (this plan feeds its "Search & discovery" section; saved searches map to roadmap **P8#11**).
>
> **Schema-reality note (2026-08-25):** This plan was checked against the current repo (`migrations/001_initial.sql`, `src/search/*`, `src/services/ollama.rs`) and corrected where the draft referenced tables/columns that don't exist. See §1.1. Migration numbering in this doc is **illustrative** — the repo keeps a single consolidated `001_initial.sql` and prod registers distinct versions in `_sqlx_migrations`; new migrations continue from `001_initial.sql` as `002_*.sql` onward and must never reuse a version already on prod.
>
> **Implementation status (2026-08-25):** Backend operators (§2–§4) + polarity/role/counter schema shipped on `main` (migrations 003–005, commit 503d990); three-tier Simple/Guided/Power UI shipped (42024c2, merged 01c8eab); saved searches + nightly alert watcher + Atom feed shipped (007, watcher wired as `fichub-saved-search` systemd timer nightly via `scripts/install_systemd_timers.sh`). Companion features landed in the same batch: follow exclusions (002), unified curator approvals + collection-submission voting (006).

---

## 0. Status Quo (what's already shipped)

Anchored in the live codebase:

| Capability | Where it lives | Status |
|---|---|---|
| Boolean parser (AND/OR/NOT, phrases, parens) | `src/search/parser.rs` | ✅ shipped |
| Fielded search (`title:`, `author:`, `fandom:`) | parser + builder | ✅ shipped |
| `main_char_attr=Character\|Attribute` (AO3 "Dark Harry" semantics) | **REMOVED (b017681)** — search is now fully tag-based; main-char/attr = `score=10` on `fic_tags` | ✅ shipped, tag-based |
| `relationship_characters=X` (any ship containing X) | `src/search/builder.rs` | ✅ shipped |
| Typo tolerance | `pg_trgm` + `word_similarity` | ✅ shipped |
| Ask the Archive (NL → filter JSON via LLM) | `src/services/ollama.rs` + `/api/search/ask` | ✅ shipped |
| Tag canonicalization + synonyms | `tags` + `tag_aliases` | ✅ shipped |
| Personal recs from bookmarks | `/api/recommendations/personal` | ✅ shipped |
| Tag autocomplete with counts | `/api/search/suggest` | ✅ shipped |
| Search-history chips (top-8 per user) | migration 063 + `/api/search/history/chips` | ✅ shipped |
| Body full-text search | `body_text_search` tsvector + `/search/body` | ✅ shipped |
| Strict-gen / exclude-tag-types, ratings, kudos/bookmark/comments filters | `src/search/builder.rs` `SearchParams` | ✅ shipped |

**What's missing** (the gap this plan closes):
- Natural-language *in-band* search (NL is siloed in `/ask`, not the main search box)
- Romantic vs. platonic ship distinction
- Character-scoped attributes (`char:"X" with attr:"Y"`)
- Saved searches with alerts (roadmap **P8#11**, currently NEW)
- Crossover/primary-fandom control
- Full metadata filter surface (pub date, update date, kudos, beta, language)
- Relevance tuning / per-user ranking weights
- Three-tier UI (Simple / Guided / Power)

---

## 1. Data Model Extensions

### 1.1 Schema reality (verified against `migrations/001_initial.sql`)

The original draft put a polarity computed column on `fic_tags` keyed off `name`. Corrections required for real schema:

- `fic_tags` columns: `url_id` (varchar 128), `tag_id` (int, FK → `tags.id`), `score` (smallint, default 0), `added_by_ip`, `is_machine_suggested`, `reviewed_at`. **It has NO `name` column and NO `work_id`.** The tag name lives on `tags` (`id`, `name`, `tag_type_id`, `description`).
- Therefore **relationship polarity is a property of the relationship TAG itself**, not of the fic-tag link → the computed column belongs on **`tags`**, not `fic_tags`. Set once per canonical relationship name, shared by every fic using it.
- Tag types (from `src/search/tags.rs::get_type_name`): `1` fandom, `2` character, `3` relationship, `4` freeform/attribute, `5` warning, `6` category, `7` other/rating.
- Work identity: `fic_info.id` = `url_id` (varchar), plus `fic_info.work_id` int FK → **`works`** (canonical merged works: `id` int PK, `canonical_title`, `embedding vector(384)`, `is_visible`). `work_ratings`/`kudos`/`bookmarks` carry **both** `url_id` and nullable `work_id`.
- Counters: `works` has **no** kudos/comments/bookmarks/hit columns yet. `fic_works` has `favouriter_count` (per-site, bookmark-ish). Kudos live in `kudos`; ratings in `work_ratings`; hits logged in `request_log` (keyed by `url_id`).

### Migration 067 — Relationship polarity + primary-fandom convention (on `tags`)

````sql
-- Polarity is per canonical relationship tag (AO3 separator convention:
-- "/" = romantic (Drarry), "&" = platonic (Harry & Draco)).
ALTER TABLE tags
  ADD COLUMN rel_polarity SMALLINT GENERATED ALWAYS AS (
    CASE
      WHEN tag_type_id = 3 AND name LIKE '%/%' THEN 1   -- romantic
      WHEN tag_type_id = 3 AND name LIKE '%&%' THEN 2   -- platonic
      WHEN tag_type_id = 3 THEN 0                        -- unspecified
      ELSE NULL                                          -- not a relationship tag
    END
  ) STORED;

CREATE INDEX idx_tags_rel_polarity ON tags(tag_type_id, rel_polarity) WHERE tag_type_id = 3;

-- Primary fandom / main character: convention = score=10 on fic_tags
-- (already the codebase rule). Codify with a doc comment; backfill script
-- assigns score=10 to the first-listed fandom tag per fic where missing.
````

### Migration 068 — Role inference confidence (on `fic_tags`)

````sql
ALTER TABLE fic_tags
  ADD COLUMN role_confidence REAL DEFAULT 1.0;
-- 1.0 = explicit (author-tagged, score>=10 or reviewed)
-- 0.5 = inferred (e.g. character appears in >60% of chapters)
-- 0.0 = unspecified (legacy / couldn't tell)
--
-- Backfill:
--   score >= 10  -> 1.0
--   score  > 0   -> 0.5
--   score  = 0   -> 0.0
````

### Migration 069 — Saved searches + alerts

````sql
CREATE TABLE saved_searches (
  id          BIGSERIAL PRIMARY KEY,
  user_id     INT4 NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name        TEXT NOT NULL,
  query_text  TEXT NOT NULL,              -- canonicalised query string
  query_json  JSONB NOT NULL,             -- parsed AST (for fast re-exec)
  alert_mode  TEXT NOT NULL DEFAULT 'none'
                CHECK (alert_mode IN ('none','email','rss','both')),
  last_run_at TIMESTAMPTZ,
  last_match_count INT4,
  created_at  TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE (user_id, name)
);

CREATE TABLE saved_search_matches (
  search_id   BIGINT NOT NULL REFERENCES saved_searches(id) ON DELETE CASCADE,
  work_id     INT4 NOT NULL REFERENCES works(id) ON DELETE CASCADE,
  first_seen  TIMESTAMPTZ DEFAULT NOW(),
  notified_at TIMESTAMPTZ,
  PRIMARY KEY (search_id, work_id)
);

CREATE INDEX idx_saved_searches_user ON saved_searches(user_id);
CREATE INDEX idx_saved_searches_alerts ON saved_searches(alert_mode)
  WHERE alert_mode <> 'none';
````

### Migration 070 — Per-fic popularity/metadata counters (denormalised on `works`)

````sql
ALTER TABLE works
  ADD COLUMN kudos_count      INT4 NOT NULL DEFAULT 0,
  ADD COLUMN comments_count   INT4 NOT NULL DEFAULT 0,
  ADD COLUMN bookmarks_count  INT4 NOT NULL DEFAULT 0,
  ADD COLUMN hit_count        INT4 NOT NULL DEFAULT 0,
  ADD COLUMN first_published  TIMESTAMPTZ,
  ADD COLUMN last_updated     TIMESTAMPTZ,
  ADD COLUMN language_code    TEXT DEFAULT 'en',
  ADD COLUMN beta_status      TEXT DEFAULT 'unknown'
                CHECK (beta_status IN ('unknown','beta','unbeta'));

-- Maintained by triggers on kudos / work_ratings / comments / bookmarks
-- (both url_id and work_id flows must update the same row; guard against
-- NULL work_id by resolving via fic_info.url_id -> work_id).
-- Hits: increment from request_log rows referencing the work.
````

**Backfill:** populate `first_published`/`last_updated` from `fic_info.fic_created`/`fic_updated` and derive `language_code`/`beta_status` from `extra_meta` where those appear; compute count columns via `COUNT(*)` subqueries on the source tables.

---

## 2. Query Language v2 — Full Syntax

Extends `src/search/parser.rs` without breaking backward compatibility (every v1 query stays valid). Canonical reference for current parser grammar: the `boolean-query-parser-pitfalls` skill.

### Operators

| Syntax | Meaning | Example |
|---|---|---|
| `field:"value"` | Exact match | `char:"Harry Potter"` |
| `field~"value"` | Contains / fuzzy | `title~"dark lord"` |
| `-field:"value"` | Negation | `-attr:"AU"` |
| `field:"val*"` | Wildcard | `char:"Harry*"` |
| `@field:"value"` | Main/primary scope | `@char:"Harry"` |
| `field:"A" with field:"B"` | Relational (scoped on same fic) | `char:"Hermione" with attr:"BAMF"` / `not with attr:"OOC"` |
| `field:A-B` | Range | `words:50000-100000` |
| `field:>N`, `<N`, `>=N` | Comparison (+ `50k` suffix) | `words:>50k` |
| `A AND B`, `A OR B`, `NOT A` | Boolean | `(dark OR grey) AND harry` |
| `"exact phrase"` | Phrase | `"boy who lived"` |

**Pitfall (regex-lite):** no backreferences in `Regex::new` under regex-lite — wildcard/`with` parsing must use per-token patterns, not backreference-style capture, or the match silently no-ops.

### Field Reference

| Field | Type | Builder note |
|---|---|---|
| `title`, `author`, `description` | text | Full-text + trigram fallback (existing) |
| `fandom`, `primary_fandom` | tag (1) | `primary_fandom` = fandom with `score=10` |
| `char`, `@char` | tag (2) | `@char` = `score=10` (main character) |
| `ship`, `romship`, `platship` | tag (3) | `romship`/`platship` split via polarity |
| `attr`, `@attr` | tag (4, freeform) | `@attr` scoped to main char (upstream was `main_char_attr`; now tag-based) |
| `warning`, `category` | tag (5/6) | |
| `rating` | tag (7) | `general`/`teen`/`mature`/`explicit` resolve to canonical rating tags |
| `status` | enum | `complete`, `ongoing`, `hiatus`, `cancelled` ↔ `fic_info.status` |
| `words`, `chapters` | int | ranges + comparisons (exists) |
| `kudos`, `comments`, `bookmarks`, `hits` | int | popularity signals (new counters) |
| `published`, `updated` | date | `published:>2020-01-01` |
| `language` | text | ISO code |
| `beta` | enum | `beta`, `unbeta`, `unknown` |
| `source` | text | site domain (`archiveofourown.org`) |
| `crossover` | bool | `crossover:false` = exactly one fandom tag |
| `fandom_weight` | percent | `fandom_weight:"HP">50` (see open questions) |

### Examples

```
@char:"Harry Potter"                                 # protagonist, not cameo
@char:"Harry" @attr:"Dark"                           # dark protagonist
-romship:*"Malfoy"                                   # no romantic Malfoy, platonic OK

char:"Hermione" with attr:"BAMF" and not with attr:"OOC"

primary_fandom:"Harry Potter" crossover:false

words:>50k kudos:>100 updated:>2025-01-01 sort:kudos

@char:"Draco" @char:"Hermione" ship:"Draco/Hermione"
  published:>last_week() sort:published
```

---

## 3. Natural Language Layer (Simple Search)

The existing `/api/search/ask` (NL → filter JSON via Ollama, `src/services/ollama.rs`) is promoted to the **default mode** of the main search box.

### Flow

```
User types: "dark harry no malfoy ships over 50k completed"
        |
        v
[1] Try structured parse first (parser.rs)    <- fast path, no LLM
        | fails or partial
        v
[2] LLM translation (lfm2.5:8b, Ask-the-Archive logic)
    Prompt: current schema + user's recent searches + synonym dict
    Returns: { query_ast, filters }
        |
        v
[3] Render as "interpreted as:" editable chip row above results
```

### Key design choices

- **Fast path first.** 80% of queries parse without the LLM; NL is fallback, not default cost.
- **Transparent interpretation.** Show understood query as clickable chips the user can edit — no black box.
- **Cache per-query-hash.** Common phrasings cached 24h (reuse `ask_cache`).
- **Graceful degradation.** Ollama down → structured parse only (current behavior).
- **NL fallback mirror of structured:** ensure both `/api/search` and `/api/search/ask` share the same filter→SQL compilation so "interpreted as" chips re-execute deterministically.

### Synonym expansion (point 10)

`tag_aliases` exists (`src/search/tags.rs`). Extend `/api/search/suggest` to return canonical form + alias list. Parser transparently rewrites aliases (`badass` → `BAMF`) at compile time.

---

## 4. Three-Tier UI

Frontend: refactor `/search/+page.svelte` into three modes (Symple/Guided/Power components). Every route change updates the URL query string so views stay shareable/bookmarkable, and every tier keeps the `{#if uiMode==='archive'}` branch + ArchiveButton conventions (see `fichub-development` / laptop UI rules). No emoji in any UI text (i18n dictionaries `src/lib/i18n/dictionaries/*.ts`).

### Tier 1 — Simple Search (default)

Single input box accepting NL or structured syntax (auto-detected).

```
[ search: dark harry no malfoy ships over 50k            ]
  Interpreted as:
  [@char:Harry] [@attr:Dark] [-romship:*Malfoy] [words:>50k] [status:complete]
  [Edit] [Switch to Guided] [Switch to Power]
  ---- Results ----
```

### Tier 2 — Guided Search

Visual builder, live faceted counts (debounced 300ms, server-side count query). Checkboxes → URL query string. Facets: characters (main/any), attributes, ships (rom/plat), fandom, rating, status, word range, popularity band, updated-within.

### Tier 3 — Power Search

Raw query box with syntax highlighting, bracket matching, inline errors (see open questions on editor library). Autocomplete for fields/operators/tag names. `[Save as...] [Set alert...]` actions.

Tiers are freely interchangeable; URL always reflects the canonical query.

---

## 5. Saved Searches & Alerts

### Flow

1. From any results page: **Save this search**.
2. Name it; pick alert mode: **None / Email digest / RSS / Both**.
3. Nightly watcher (`src/bin/saved_search_watcher.rs`) re-runs each saved query, diffs against `saved_search_matches`, and:
   - inserts new matches
   - sends one batched email digest per user (all new matches, one email)
   - updates per-search RSS feed at `/u/{id}/saved/{search_id}.xml`

### Email digest

- One email per user per day (configurable), sent around user's local 9am.
- Contains: search name, top-10 new matches (title/author/blurb), "view all" link.
- Per-search unsubscribe link → sets `alert_mode='rss'`.
- Relay: `src/services/mailer.rs` / `kindle.rs` SMTP wiring (needs creds — USER-ACTIONS pattern).

### RSS

Stable per-search Atom feed; power-user path, zero email, full control.

---

## 6. Personalization & Relevance Tuning

### Default ranking

```
relevance = 0.40 * ts_rank
          + 0.25 * tag_match_score
          + 0.15 * popularity_norm   (log-scaled kudos + bookmarks)
          + 0.10 * recency           (updated, decayed)
          + 0.10 * personal_affinity (tag overlap w/ user bookmarks)
```

Implement as a composable `ORDER BY` expression built in `builder.rs`; weights become SQL parameters (never string-injected).

### User-tunable weights

`PUT /api/user/search-prefs`
```json
{ "weights": { "popularity": 0.3, "recency": 0.2, "personal": 0.2 },
  "author_blacklist": ["username1"],
  "default_sort": "relevance" }
```
Stored in `user_search_prefs` (JSONB; **create table** — not in current schema), applied at query compile time. Normalize weights so they sum to 1 inside a tunable band; clamp.

### Integration with rec platform

`rec_*` infrastructure (`rec_embeddings`, bookmark-affinity signals) feeds `personal_affinity`. No new model required — expose existing signals as a ranking input.

---

## 7. Implementation Roadmap

- **Phase 1 — Schema + parser (1 wk):** migrations 067–070, backfills, extend `parser.rs` (`with`, `@` any field, `romship`/`platship`, ranges, aliases). Unit tests per operator.
- **Phase 2 — Backend query compilation (1 wk):** extend `builder.rs` for new fields; `saved_searches` CRUD; `user_search_prefs` table + endpoint; `search-prefs` endpoint; DB-gated integration tests (extend `tests/search_api.rs`).
- **Phase 3 — NL layer promotion (3–5 d):** `/api/search` tries structured first → LLM fallback; synonym expansion in suggest; NL→AST cache; "interpreted as" chip row in response.
- **Phase 4 — Frontend three-tier UI (1.5 wk):** refactor `/search/+page.svelte` → `SimpleSearch.svelte` / `GuidedSearch.svelte` / `PowerSearch.svelte`; live facet counts (debounced); syntax-highlighted editor; Vitest + Playwright.
- **Phase 5 — Saved searches + alerts (1 wk):** save/search UI (modal + list page); nightly watcher bin; email digest; per-search RSS; tests.
- **Phase 6 — Personalization + polish (3–5 d):** search-prefs UI, weight sliders, author blacklist UI, A/B default vs tuned weights.

**Total:** ~5–6 wk one dev, ~3 wk two.

---

## 8. Testing Strategy

| Layer | Tool | Coverage target |
|---|---|---|
| Parser | `cargo test --lib search::parser` | 100% of operators |
| SQL builder | `cargo test --lib search::builder` | 100% of field types |
| Integration | `tests/search_api.rs` (DB-gated) | ≥25 cases incl. NL fallback |
| Saved searches | `tests/saved_searches_api.rs` | CRUD + watcher |
| Frontend | Vitest + Playwright | Three-tier flow, facet updates |
| NL layer | `tests/ask_archive.rs` | 10 canonical phrasings |
| Performance | `EXPLAIN ANALYZE` on 100k-row fixture | p95 < 200ms |

Canonical verify (`hermes verify --save`) must stay green throughout.

---

## 9. Open Questions

1. **Email relay.** `kindle.rs`/`mailer.rs` need SMTP creds (USER-ACTIONS pattern). Same relay serves digests.
2. **NL model choice.** Confirm lfm2.5:8b handles new operators (`with`, `romship`) in prompt tests before shipping.
3. **Primary-fandom heuristic.** First-listed score=10 is a good proxy but not perfect. Author override via a future "claim your work" flow (roadmap P8#36)?
4. **Crossover threshold.** `fandom_weight:"HP">50` needs per-fandom tag counts. Is tag-overlap proxy enough, or chapter-level sampling?
5. **Alert frequency.** Daily digest vs real-time. Daily cheaper/less spammy; real-time needs pub/sub. Recommend daily v1, real-time v2 on demand.
6. **Power editor library.** CodeMirror 6 is heavy (~200KB). Custom lightweight highlighter vs accept weight for power users.
7. **Counters migration scope.** `hits` from `request_log` is high-volume — confirm trigger cost/throughput acceptable before shipping live counting.

---

## 10. What This Gets Us

| AO3 limitation | FicHub v2 answer |
|---|---|
| No protagonist vs cameo | `@char` + role confidence |
| No ship exclusion without character exclusion | `-romship:*"X"` / `-platship:*"X"` |
| No scoped attributes | `@attr` + `with` operator |
| No saved-search alerts | Nightly watcher + email + RSS |
| No crossover control | `primary_fandom` + `fandom_weight` |
| Clunky word-count filters | Range syntax + all metadata first-class |
| No personalization | User-tunable weights + rec-platform integration |
| Strict tag matching | Synonym expansion + trigram fallback + NL layer |
| One-size-fits-all UI | Three tiers (Simple / Guided / Power) |

Result: the most capable search system in the fanfiction archive space while remaining approachable for someone who just types "dark harry no malfoy" and gets results.
