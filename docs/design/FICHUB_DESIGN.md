# FicHub v2 Architecture — Design Document

> **Status: ✅ IMPLEMENTED** (July 2026)
>
> The core of this design — the **unified works model** (Section 1 + Section 8) —
> has been fully implemented. The `works` table, auto-merge heuristics, curator
> proposals/voting, and social table migration are all live. Sections on
> translations (3), vector search (4), and gamification details (5) remain
> **aspirational / future work**.
>
> See `references/unified-works-schema.md` for the current canonical schema.

> **Goal**: Evolve FicHub from a download tool into a full community platform with normalized metadata, translations, gamification, vector search, and features that 70%+ of users expect — all running on an Orange Pi 5 (4GB RAM).

---

## Table of Contents
1. [Database Normalization](#1-database-normalization)
2. [Metadata Blob + Curated Normalization](#2-metadata-blob--curated-normalization)
3. [Translations](#3-translations)
4. [Vector Embedding Search](#4-vector-embedding-search)
5. [Gamification & Leaderboards](#5-gamification--leaderboards)
6. [Features for 70%+ Users](#6-features-for-70-users)
7. [Orange Pi Constraints](#7-orange-pi-constraints)
8. [Migration Path](#8-migration-path)

---

## 1. Database Normalization

### Current State (v1)
A single `fic_info` table with flat columns. Tags are in a separate normalized system (003_tagging.sql). Search uses PostgreSQL `tsvector` on title + description.

### Implemented State (v2) ✅
```
works                          ← Core normalized work
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │ ← SERIAL PK (not url_id — simplified from design)
│ canonical_title TEXT         │ normalized title
│ canonical_author TEXT        │ normalized author
│ description TEXT             │ curated description
│ default_source_id TEXT       │ FK to fic_info(id) — preferred source URL
│ created_at TIMESTAMPTZ      │
│ updated_at TIMESTAMPTZ      │
└─────────────────────────────┘

fic_info                        ← Source-level metadata (gained work_id FK)
┌─────────────────────────────┐
│ id (PK)                     │ url_id hash (12 hex chars)
│ title TEXT                   │ original scraped title
│ author TEXT                  │ original scraped author
│ description TEXT             │ original scraped description
│ ...
│ source TEXT                  │ full URL
│ work_id INTEGER FK→works(id) │ ← NEW: link to normalized work record
└─────────────────────────────┘

auto_merge_log                  ← Tracks automatic merge operations
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ source_url TEXT              │ URL that triggered the merge
│ matched_work_id INTEGER FK  │ work it was merged into
│ confidence FLOAT             │ merge confidence score
│ created_at TIMESTAMPTZ      │
└─────────────────────────────┘

work_proposals                  ← Curator merge/split proposals
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ proposer_id INTEGER FK      │ user who proposed
│ action_type TEXT             │ 'merge' | 'split'
│ source_work_id INTEGER FK   │ work to merge FROM
│ target_work_id INTEGER FK   │ work to merge INTO
│ work_id INTEGER FK           │ work being split (for splits)
│ details JSONB                │ proposal details
│ status TEXT                  │ 'pending' | 'accepted' | 'rejected'
│ created_at TIMESTAMPTZ      │
│ closed_at TIMESTAMPTZ       │
└─────────────────────────────┘

work_proposal_votes             ← Voting on proposals
┌─────────────────────────────┐
│ proposal_id INTEGER FK      │ → work_proposals(id)
│ user_id INTEGER FK           │ → users(id)
│ vote SMALLINT                │ 1=approve, 0=retract, -1=disapprove
│ voted_at TIMESTAMPTZ        │
│ PK (proposal_id, user_id)   │
└─────────────────────────────┘

reputation_events               ← Gamification foundation
┌─────────────────────────────┐
│ id SERIAL PRIMARY KEY       │
│ user_id INTEGER FK           │ → users(id)
│ event_type TEXT              │ e.g. 'curation_approve'
│ delta INTEGER                │ points (+/-)
│ created_at TIMESTAMPTZ      │
└─────────────────────────────┘
```

> **Design deviation**: The original design proposed `url_id` as the works PK
> and a `raw_json` JSONB column on fic_info. The implementation simplified this:
> `works` uses a SERIAL id (simpler joins, no hash deps), and `raw_json` was
> deferred (the scraper already stores structured data in fic_info columns).
> `curated_status` was also deferred — proposals/votes handle curation instead.

### Key Design Decisions

**1. Two-tier metadata (raw + curated) — IMPLEMENTED**
- `fic_info` keeps the ORIGINAL scraped data as-is (title, author, description, etc.)
- `works` table stores the CURATED, normalized version
- `fic_info.work_id` links each source record to its parent work
- All user-facing queries go through `works` (the normalized view)
- Curators propose changes via `work_proposals` + voting (not direct edits)

**2. Auto-merge on scrape — IMPLEMENTED**
- `find_or_create_work()` runs when a fic is scraped
- Exact title+author match (case-insensitive, trimmed) + word count within ±5%
- High confidence (≥0.9) → auto-merge immediately
- Medium confidence → create pending proposal
- New works are created automatically when no match is found

**3. Relationship to current tag system — PRESERVED**
- Current `tags`, `tag_types`, `fic_tags` tables remain unchanged
- Tags are associated with fic_info records (source-level), not works
- The existing tag voting (003_tagging.sql trigger) continues unchanged
- Future work could add work-level tag normalization via junction tables

**4. Curated status workflow — DEFERRED**
```
// Original design proposed this workflow:
0 = RAW:    Newly scraped, not touched by curator
1 = PARTIAL: Some fields normalized
2 = FULL:   All fields normalized and verified
3 = VERIFIED: Reviewed by trusted curator, locked

// Actual implementation: deferred — curation handled via
// work_proposals + voting instead. A curated_status column
// may be added in the future if editorial workflows are needed.
```

---

## 2. Metadata Blob + Curated Normalization

> **Status: DEFERRED** — `raw_json` column was not added to fic_info. The scraper
> already stores structured fields (title, author, description, words, chapters, etc.)
> in fic_info's existing columns. A JSONB blob may be added later if full scraper
> response archival becomes necessary.

### Raw JSON Blob (Planned, Not Implemented)

Add to `fic_info`:
```sql
ALTER TABLE fic_info ADD COLUMN raw_json JSONB;
```

Contains everything the scraper found:
```json
{
  "scraped_at": "2024-01-15T10:30:00Z",
  "url": "https://archiveofourown.org/works/12345678",
  "html_title": "The Story Title",
  "meta_tags": {"author": "AuthorName", "..."},
  "chapter_titles": ["Chapter 1", "Chapter 2"],
  "full_html": null,
  "scraper_version": "1.2.0"
}
```

`full_html` is NULL by default — only stored when a curator explicitly requests it for debugging.

### Curation Interface

| Tool | Description |
|------|-------------|
| **Field Editor** | Inline edit title, author, description, language, rating, status |
| **Tag Merger** | Search existing canonical tags, merge duplicates |
| **Tag Suggestion** | AI/algorithm suggests tags based on raw_json keywords |
| **Bulk Operations** | Apply changes across multiple works from same author/series |
| **Diff View** | See raw vs curated side-by-side before approving |

### Curation Permissions

| Role | Can edit own | Can approve edits | Can lock |
|------|-------------|-------------------|----------|
| New User | No | No | No |
| Reader | No | No | No |
| Curator | Yes (pending approval) | No | No |
| Senior Curator | Yes (instant) | Yes | No |
| Admin | Yes (instant) | Yes | Yes |

---

## 3. Translations

> **Status: FUTURE WORK** — Not yet implemented. Schema design retained below
> as a starting point when translations are prioritized.

### Database Schema

```sql
CREATE TABLE locales (
    id SERIAL PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,           -- "en", "es", "fr", "ja", "pt-BR"
    name TEXT NOT NULL,                  -- "English", "Español", ...
    is_rtl BOOLEAN DEFAULT FALSE
);

-- Static UI translations
CREATE TABLE translations (
    id BIGSERIAL PRIMARY KEY,
    locale_code TEXT REFERENCES locales(code),
    namespace TEXT NOT NULL,             -- "ui.download.title", "search.labels.fandom"
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(locale_code, namespace, key)
);

-- Dynamic content translations (fic metadata)
CREATE TABLE work_translations (
    id BIGSERIAL PRIMARY KEY,
    url_id VARCHAR(128) REFERENCES works(url_id) ON DELETE CASCADE,
    locale_code TEXT NOT NULL REFERENCES locales(code),
    title TEXT,
    summary TEXT,
    -- Translated by
    translated_by INT4 REFERENCES users(id),
    translated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(url_id, locale_code)
);
```

### Translation Workflow

1. User with translator role opens a work page
2. Clicks "Translate" button → form shows original text alongside editable fields
3. Submits translation → queued for review (or auto-approved for trusted translators)
4. Translated version becomes available when locale matches user preference

### Locale Detection

- User setting (stored in profile)
- Browser `Accept-Language` header fallback
- URL prefix: `/es/works/{url_id}` (optional, configurable)

---

## 4. Vector Embedding Search

> **Status: FUTURE WORK** — Not yet implemented. Design retained below as a
> starting point. Will require pgvector extension + embedding model on the Pi.

### Constraint: Orange Pi 5 (4GB RAM)

This rules out:
- Running a full LLM locally (would use 4GB+ just for the model)
- HuggingFace transformers (too heavy)
- ONNX Runtime with large models

### Feasible Options

**Option A — pgvector with small model (RECOMMENDED)**

Use `all-MiniLM-L6-v2` via `transformers.js` or Rust's `fastembed` crate.
- 384-dim float vectors → ~1.5KB per row
- IVFFlat index with 100 centroids → <100ms ANN search on 100k rows
- Quantize to halfvec (768 bytes per row) or binary (48 bytes per row)
- Total vector storage for 200k fics: ~150MB (float), ~10MB (binary)

```sql
-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE works ADD COLUMN embedding vector(384);
ALTER TABLE works ADD COLUMN embedding_updated_at TIMESTAMPTZ;

-- IVFFlat index for approximate nearest neighbor search
CREATE INDEX idx_works_embedding ON works 
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Or HNSW index (faster, more memory)
-- CREATE INDEX idx_works_embedding ON works USING hnsw (embedding vector_cosine_ops);
```

**Option B — Hybrid search (full-text + vector)**

Use PostgreSQL tsvector as primary search (already implemented in 003_tagging.sql) AND pgvector as a secondary ranker. Combine scores:

```sql
SELECT w.*, 
    ts_rank(w.text_search, to_tsquery('english', 'harry potter')) AS text_score,
    (1 - (w.embedding <=> query_embedding)) AS vector_score,
    (ts_rank(w.text_search, to_tsquery('english', 'harry potter')) * 0.6 + 
     (1 - (w.embedding <=> query_embedding)) * 0.4) AS combined_score
FROM works w
ORDER BY combined_score DESC
LIMIT 20;
```

**Option C — Embedding generation service (EDA APPROACH)**

Generate embeddings via a small standalone service:
- Tiny Rust binary using `fastembed` crate with `all-MiniLM-L6-v2`
- Model file: ~80MB on disk
- RAM usage: ~200MB while processing
- Processing time: ~200ms per fic (5 fics/sec)
- Batch processing: 100 fics in ~5 seconds
- Backfill 100k fics: ~5.5 hours (acceptable for a one-time job)

```bash
# Install model
curl -LO https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx

# Process embeddings in background (cron job, low priority)
./fichub embed --batch 100 --sleep-ms 50
```

### Search Architecture

```
User Query
    │
    ├── Text Query ──→ ts_rank(text_search, query)
    │                       │
    │                       ▼
    │               Combined Score ←── (0.6 × text + 0.4 × vector)
    │                       │
    │                       ▼
    │               Sorted Results
    │
    └── Syntax Query ──→ parseSearchQuery() → filter by tags/fields → sort
```

### Performance Budget

| Operation | Current | With pgvector | Notes |
|-----------|---------|---------------|-------|
| Full-text search | 5-15ms | 5-15ms | Unchanged |
| Vector search | N/A | 20-80ms | IVFFlat index |
| Hybrid search | N/A | 25-95ms | Combined |
| Embedding (batch) | N/A | 200ms per fic | Background job |
| RAM usage | ~1.7MB | +200MB | Model-only, not per-request |

---

## 5. Gamification & Leaderboards

> **Status: PARTIALLY IMPLEMENTED** — `reputation_events` table is live (foundation).
> The full badge system, daily quests, and leaderboard tables are future work.
> Proposal/voting system (Section 1) handles the curation gamification loop.

### Reputation System

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    email TEXT UNIQUE,
    role SMALLINT DEFAULT 0,             -- 0=reader, 1=curator, 2=senior, 3=admin
    reputation INT4 DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE reputation_events (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    event_type TEXT NOT NULL,             -- "curation_approve", "translation", "suggestion_accept"
    points INT4 NOT NULL,
    reference_type TEXT,                  -- "work", "tag", "translation", "suggestion"
    reference_id TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE user_badges (
    id BIGSERIAL PRIMARY KEY,
    user_id INT4 NOT NULL REFERENCES users(id),
    badge_type TEXT NOT NULL,             -- "curator_10", "translator_50", "voter_100"
    earned_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, badge_type)
);
```

### Reputation Point Values

| Action | Points | Badge Thresholds |
|--------|--------|------------------|
| Suggest a tag | +3 | 10 suggestions → "Tag Enthusiast" |
| Vote on tag | +1 | 100 votes → "Active Voter" |
| Curation approved | +25 | 10 approved → "Curator Apprentice" |
| Curation rejected | -5 | 50 approved → "Senior Curator" |
| Translation submitted | +15 | 10 translations → "Translator Novice" |
| Translation approved | +35 | 50 translations → "Polyglot" |
| Suggestion accepted | +20 | — |
| Daily login streak | +2/day | 7 days → "Streak Starter", 30 → "Dedicated" |
| Report bug | +10 | — |

### Leaderboards

| Leaderboard | Refresh | Scope |
|-------------|---------|-------|
| Top Curators (week) | Every hour | Weekly XP |
| Top Curators (all-time) | Daily | Total XP |
| Top Translators | Daily | Translation count |
| Most Active Voters | Hourly | Vote count |
| Rising Stars | Daily | New users with fastest rep growth |
| Works Most In Need | On-demand | Uncurated works by age/popularity |

### API Endpoints

```
GET /api/v1/leaderboard/curators?period=week&limit=20
GET /api/v1/leaderboard/translators?locale=es&period=all
GET /api/v1/users/{id}/reputation
GET /api/v1/users/{id}/badges
POST /api/v1/users/{id}/claim-badge/{badge_type}
```

### Daily Quests (Engagement)

| Quest | Reward |
|-------|--------|
| Curate 3 works today | +50 XP, "Curator" badge progress |
| Vote on 10 tags | +20 XP |
| Translate 1 summary | +40 XP |
| Suggest 5 tags | +15 XP |
| Login 7 consecutive days | +100 XP, streak badge |

---

## 6. Features for 70%+ Users

> **Status: IN PROGRESS** — Core social features (bookmarks, ratings, comments)
> and export system are live. Leaderboards, reading stats, and notifications
> are future work.

Based on typical fanfiction platform expectations + HN-style social features:

### Tier 1: Core (95%+ would want)
- ✅ **User accounts** — login/logout/profile
- ✅ **Bookmark/favorites** — save works to personal list
- ✅ **Search** — already exists, enhanced with vector
- ✅ **Download to EPUB/HTML/MOBI/PDF/AZW3/TXT/MD** — already exists
- ✅ **Filter by tags/status** — already partially exists
- ✅ **Dark mode** — already exists
- ✅ **Responsive mobile design** — already exists

### Tier 2: Expected (80%+ would want)
- ✅ **Reading history** — recently viewed works
- ✅ **Collections/lists** — create shared reading lists (004_shelves.sql exists)
- ✅ **RSS/OPDS** — subscribe to updates (already exists)
- ✅ **Related works** — recommendations (already exists)
- ✅ **Rating system** — like/dislike works
- ✅ **Comments** — discuss works
- ✅ **Author pages** — browse by author

### Tier 3: Addictive (70%+ would want)
- ✅ **Weekly/daily popular** — trending works this week
- ✅ **New works feed** — recently added works
- ✅ **Reading stats** — words read, fics completed
- ✅ **Custom recommendations** — based on reading history
- ✅ **Social features** — follows, notifications
- ✅ **Third-party API** — integrate with external tools
- ✅ **Random work button** — discovery
- ✅ **Work series tracking** — series with multiple parts

### Implementation Priority

| Phase | Features | Est. Backend | Est. Frontend | Risk |
|-------|----------|-------------|---------------|------|
| **P1** | User accounts, bookmarks, collections | 2 weeks | 1 week | Low |
| **P2** | Comments, ratings, reading history | 3 weeks | 1 week | Low |
| **P3** | Leaderboards, badge system, daily quests | 2 weeks | 2 weeks | Low |
| **P4** | Vector search, curation workflow | 3 weeks | 2 weeks | Medium |
| **P5** | Translations, author pages, reading stats | 2 weeks | 2 weeks | Medium |
| **P6** | Notifications, follows, series tracking | 2 weeks | 1 week | Medium |

---

## 7. Orange Pi Constraints

> **Status: REFERENCE** — Constraints unchanged. Budget estimates still valid.

### Memory Budget (4GB total)

| Component | RAM | Notes |
|-----------|-----|-------|
| OS + services | ~500 MB | Armbian, systemd basics |
| PostgreSQL | ~200 MB | Shared_buffers=256MB |
| Redis | ~50 MB | Default config |
| Rust backend (v2) | ~5 MB | Static binary, no heavy deps |
| Embedding model (runtime) | 0 MB | Only loaded during batch jobs |
| SvelteKit SPA | ~30 MB | Static files, served by Rust |
| **Available for features** | **~3.2 GB** | Growth room |

### What WON'T Work

- Running a full LLM (even llama 3B would consume >80% RAM)
- ONNX runtime with large models (>500MB)
- Real-time embedding at request time (batch only)
- ElasticSearch (too heavy, PostgreSQL full-text is sufficient)
- Image processing pipeline (thumbnails, OCR)

### What WILL Work

- pgvector with binary quantization (48 bytes per row)
- Lightweight ONNX model via `fastembed` or `ort` crate (200MB RAM, disposed after batch)
- PostgreSQL full-text search as primary index
- Background jobs for embedding generation (cron, low priority)
- Cached leaderboards (refresh every 5 min via cron)

### Storage Budget

| Data | Size Estimate |
|------|---------------|
| 200k works metadata | ~200 MB |
| 200k vectors (binary) | ~10 MB |
| Tag system (current) | ~5 MB |
| User accounts (10k) | ~10 MB |
| Bookmarks (500k) | ~50 MB |
| Comments (100k) | ~50 MB |
| Translations (50k) | ~10 MB |
| **Total** | **~335 MB** |

Well within the Orange Pi's available storage.

---

## 8. Migration Path

> **Status: Phase 0 IMPLEMENTED** — `002_unified_works.sql` has been applied.
> The remaining phases are future work.

### Phase 0: Schema Migration (v1 → v2 tables) ✅ DONE

Migration file: `migrations/002_unified_works.sql`

```sql
-- 1. Create works table with SERIAL PK (simplified from original url_id design)
CREATE TABLE works (
    id SERIAL PRIMARY KEY,
    canonical_title TEXT,
    canonical_author TEXT,
    description TEXT,
    default_source_id TEXT REFERENCES fic_info(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Add work_id FK to fic_info (links source → work)
ALTER TABLE fic_info ADD COLUMN work_id INTEGER REFERENCES works(id);
CREATE INDEX idx_fic_info_work_id ON fic_info(work_id);

-- 3. Bootstrap existing fic_info rows into works
--    Uses title+author matching to deduplicate:
--    - Exact match (case-insensitive, trimmed) → merge into existing work
--    - Word count within ±5% → high confidence auto-merge
--    - No match → create new work

-- 4. Create auto-merge log
CREATE TABLE auto_merge_log (
    id SERIAL PRIMARY KEY,
    source_url TEXT NOT NULL,
    matched_work_id INTEGER NOT NULL REFERENCES works(id),
    confidence FLOAT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 5. Create proposal/vote tables
CREATE TABLE work_proposals ( ... );
CREATE TABLE work_proposal_votes ( ... );

-- 6. Create reputation_events table
CREATE TABLE reputation_events ( ... );

-- 7. Migrate social tables to work_id
--    bookmarks, work_ratings, comments, opds_shelf_items
--    all gain work_id column, FK → works(id)
```

### What was NOT done (compared to original design)

- ❌ `raw_json JSONB` column on fic_info (deferred)
- ❌ `curated_status` column on works (deferred — proposals handle this)
- ❌ Junction tables (work_fandoms, work_characters, etc.) — deferred
- ❌ `source_version` column on works — not needed yet

### Phase 1: User System (non-breaking) ✅ DONE

- Users, auth, sessions — implemented
- Bookmarks — implemented (work_id-based)
- Collections — extend existing opds_shelves (implemented)

### Phase 2: Social & Gamification (partially done)

- Comments — ✅ implemented
- Ratings — ✅ implemented
- reputation_events — ✅ implemented (foundation)
- Leaderboards, badges, daily quests — ❌ future work

### Phase 3: Vector Search (future work) ❌

- pgvector extension — requires PostgreSQL restart
- Backfill embeddings via background job (cron, 5 fics/sec)
- Modify search to include vector score (behind feature flag)

### Phase 4: Translations & Curation (future work) ❌

- Locales, translations — new tables
- Work translations — new table
- Curation UI — new frontend pages
- Curation workflow — modifies works.curated_status

### Rollback Strategy

- All new tables are ADDITIONS — no existing tables are modified or dropped
- `fic_info` always retains original data, so rollback = just stop using new tables
- `works` duplicates data from `fic_info` — can be regenerated from `fic_info`
- Old search works unchanged — vector search is additive

---

## Summary Table

| Requirement | Approach | Status | Orange Pi Feasible? |
|-------------|----------|--------|-------------------|
| Normalized DB | works table + fic_info FK | ✅ Implemented | ✅ |
| Auto-merge | title+author matching + confidence | ✅ Implemented | ✅ |
| Curation proposals | work_proposals + voting | ✅ Implemented | ✅ |
| Metadata blob | raw_json JSONB on fic_info | ❌ Deferred | ✅ |
| Curated status workflow | curated_status + curator role | ❌ Deferred | ✅ |
| Social features | bookmarks, ratings, comments | ✅ Implemented | ✅ |
| Reputation foundation | reputation_events table | ✅ Implemented | ✅ |
| Leaderboards & badges | reputation + badge tables | ❌ Future | ✅ |
| Translations | work_translations table | ❌ Future | ✅ |
| Vector search | pgvector +all-MiniLM-L6-v2 | ❌ Future | ✅ (batch) |
| 70%+ features | 15 features across 3 tiers | 🟡 In progress | ✅ |
| Runs on Orange Pi 5 | 335MB storage, ~200MB added RAM | ✅ | — |

**Completed**: Unified works model, auto-merge, proposals/voting, social features (bookmarks, ratings, comments), export system (7 formats).
**Deferred**: raw_json blob, curated_status workflow, junction tables.
**Future**: Translations, vector search, full gamification (badges, leaderboards, daily quests).
