# AO3 Feature Gap Analysis

## Status: P0 Reading History ✅ done | P1+ prioritized backlog

## Overview

This document catalogs every user-facing AO3 feature that is missing from FicHub, organized by the left sidebar sections in AO3's user dropdown menu. Each section lists what AO3 provides, whether FicHub has it, and the priority for implementation.

Priority: P0 = essential reading experience, P1 = social/core feature, P2 = nice-to-have, P3 = rarely used / cosmetic

## Lists/Shelves vs Collections — DECIDED: Different Concepts

AO3 uses a single "Collections" concept for everything. FicHub splits this into two tables with different purposes:

- **`shelves`** (migration 004): Reading-status buckets — "Want to Read", "Currently Reading", "Finished", "Dropped". These map to AO3's "Mark for Later" (shelf) + reading-status checkboxes on work pages. The migration comment says "User-created shelves/collections."
- **`reading_lists`** (migration 019): Ordered, shareable lists with per-item blurbs (private by default, shareable via /api/lists/). These map to AO3's "Collections" — curated reading lists.

**Decision**: Do NOT rename either table. They already model AO3's two-layer system:
  - AO3 "Mark for Later" / reading status → FicHub `shelves` (status)
  - AO3 "Collections" (curated lists) → FicHub `reading_lists`
- The `reading_history` table (migration 057) is new and maps to AO3's per-visit history log, a third distinct concept.

A future "Collections" UI should surface `reading_lists` as AO3-style collections, with the UI label "Collections" while the backend table remains `reading_lists`.

## P0 — Implemented: Reading History

### `reading_history` table (migration 057)
- Columns: `id`, `user_id`, `work_id`, `chapter_num`, `visited_at`, `created_at`
- Records every visit to a work (reader open or work page view)
- `visited_at` timestamp per-visit (AO3 tracks every visit, not just last-read)
- `chapter_num` optional — captures which chapter was open
- Primary key on `(user_id, work_id, chapter_num, visited_at)` allows multiple visits

### API endpoints (registered in server.rs)
- `GET /api/reading/history?limit=50&offset=0` — paginated list of history entries
  Returns: `{ err, history: [{id, work_id, url_id, title, author, chapter_num, visited_at}], total, limit, offset }`
- `POST /api/reading/history` — record a visit `{ work_id, chapter_num? }`
- `DELETE /api/reading/history/{id}` — delete a single entry
- `POST /api/reading/history/clear` — clear all history

### Frontend
- Route `/history/+page.svelte` with archive-mode UI (AO3-style dl/lists) + modern fallback
- Navbar "History" link added to Search dropdown
- Reader page (`/read/[urlId]`) records a visit via `recordReadHistory()` on load
- API client functions: `getReadingHistory()`, `recordReadHistory()`, `deleteReadHistory()`, `clearReadHistory()`
- i18n keys added to all 6 locales (en, de, es, fr, pt-BR, zh)
- Tests: 7/7 passing (`src/routes/history/page.test.ts`)

### Known limitation
- No deduplication yet (AO3 collapses visits to the same work within a session). Current implementation records every reader open. A future enhancement could add a 1-hour dedupe window.

---

## Bounties for Trope-Based Writing (NEW — P8 UF)

**Motivation:** Most archives rely on user-generated content but lack an incentive layer for writing. A bounty system turns reading history + taste profiles into a content-creation flywheel: popular underserved tropes get staked as bounties, writers claim them, and the community rewards good output.

**Design:**
- `bounties` table: `(id, creator_user_id, title, description, tag_filters JSONB, xp_pool, status ENUM('open','claimed','completed','expired'), expires_at, created_at)`
- `bounty_claims` table: `(id, bounty_id, claimant_user_id, work_id, status ENUM('pending','approved','rejected'), curator_id, reviewed_at)`
- **Create:** User stakes XP (or gold/fichub-coin if that currency exists) into a bounty specifying a trope combo + optional setting. Example: "Dark Harry × Draco in a Coffee Shop AU — 40 XP pool."
- **Claim:** Writer creates a work matching the tag criteria, links it to the bounty. System auto-checks tag match; curators verify (positive-only — no public shaming of rejected claims).
- **Win:** First approved claim wins the pool. Unclaimed bounties can accumulate more stakes over time.
- **Variants:**
  - Author-posted bounties ("I'll write X trope if someone stakes 50 XP on it")
  - Time-limited bounties (expire after 30 days)
  - Fandom-scoped vs site-wide bounties
  - Community-voted bounty promotion (top-voted bounties appear on the homepage)
- **Pairs with:** Reading History (#34), Taste Profile (#29), Kudos (#52 in P8), Tag Wikis (#78) — all low-lift features that feed the same engagement engine.
- **Data needs:** `bounties` + `bounty_claims` tables, two API endpoints (`GET /api/bounties`, `POST /api/bounties/claim`), `/bounties` browse page, and a "Claim Bounty" button on the work submission/edit flow.

---|

## Sidebar Sections

### Dashboard
**AO3:** Personal dashboard with quick filters (Unread / By Date / By Kudos), reading progress, recently updated bookmarks, collections activity.
**Fichub:** None
**Priority:** P1
**Notes:** Fichub has `/reading` which partially covers this but lacks the dashboard layout with quick filter buttons and recent activity summary.

### Profile
**AO3:** Public profile showing avatar, bio, badges, favorite tags, fandoms, works, bookmarks, collections, history (configurable visibility).
**Fichub:** `/authors/[name]` shows author profile but lacks bio/edit fields, badges display, favorites, collections list.
**Priority:** P1

### Preferences
**AO3:** Full preferences page — email notifications, comment threads, kudos, bookmarks, subscriptions, history, search history, profile display, blacklist, skins, time zone.
**Fichub:** `/settings` exists but minimal — default format, theme, language. No notification prefs, no blacklist management, no skin selection.
**Priority:** P1

### Skins
**AO3:** User-created CSS skins — public skins, site skins, personal skin, skin creator with editor.
**Fichub:** `/settings/theme` has basic theme selection but no custom CSS skin system.
**Priority:** P2

### Pitch (Works / Drafts / Series / Bookmarks / Collections)
**AO3:** Sub-navigation linking to your own works, drafts, series, bookmarks, collections.
**Fichub:**
- Works: No user works dashboard (no upload/creation flow yet) — P2
- Drafts: No drafts system — P2
- Series: `/series/[id]` exists but no series management UI (create/edit/reorder) — P1
- Bookmarks: `/bookmarks` exists — ✓
- Collections: No collections system — P1

### Catch (Inbox / Statistics / History / Subscriptions)
**AO3:**
- **Inbox:** Notification inbox for comments on your works/ replies to your comments/ kudos/ bookmarks.
- **Statistics:** Per-work stats dashboard (hits, kudos, comments, bookmarks, top referrers).
- **History:** Reading history with visit timestamps.
- **Subscriptions:** List of works/users/collections you're following, with email notification toggle.

**Fichub:**
- Inbox: `/notifications` exists — covers comment/kudos/bookmark notifications — ✓
- Statistics: `/stats` exists but is global site stats, not per-work user stats — P1
- **History: Missing entirely** — P0
- Subscriptions: `/follows` exists (follows users/collections) but lacks work-level follow + email toggle — P1

### Switch (Sign-ups / Assignments / Claims / Related Works / Gifts)
**AO3:** Fic exchange / gift-athon system.
- Sign-ups: List of exchange signups you've joined.
- Assignments: Your assigned recipients.
- Claims: Works claimed for a request.
- Related Works: Works related to a gift-athon (your entries).
- Gifts: Works gifted to others.

**Fichub:** `/work-proposals` and `/requests` partially cover the "requests" concept but no full gift-athon workflow.
**Priority:** P3 (unless Fichub plans its own exchange system)

---

## Fandom Categories

**AO3:** Fandoms are categorized — Movies, Television, Movies & TV (Crossovers), Literature, Anime & Manga, Cartoons & Comics, Music, Theater, Mythology, Video Games, etc.

**Fichub:** `/fandoms` exists as flat alphabetical listing but without category grouping.
**Priority:** P2 (would help browseability)

---

## Top Navigation Bar (non-sidebar)

**AO3 top nav:**
- Home / Fandoms / Tags / Characters / Relationships / Genres / Collections / People / Works / Series / Challenges / Requests / Search / Random / Help

**Fichub top nav (current):**
- Archive (dropdown: Works, Fandoms, People, Tags, Series, Characters, Collections, Challenges, Requests, Authors, Lists) / Download / Read / Search / Recommendations / Forum / Roadmap / Stats / Leaderboard / Badges / Updates / Settings

### Missing top-nav-level pages:

1. **Challenges**
   - AO3: Listing of all challenge events (gift-athons, exchange, fest, etc.) with status (open, in progress, ended).
   - Fichub: No challenges page — P3

2. **Collections listing (site-wide)**
   - AO3: `/collections` — browse all public collections on the site.
   - Fichub: No site-wide collections browse — P2

3. **Characters browse**
   - AO3: `/people`? No — `/characters` — alphabetical listing of all characters, grouped by fandom.
   - Fichub: No `/characters` browse page — P2

4. **Relationships browse**
   - AO3: `/relationships` — alphabetical listing of all romantic/platonic pairing tags.
   - Fichub: No `/relationships` browse page — P2

5. **Genres browse**
   - AO3: Genres are part of the tag system, browsed via `/tags/genres`.
   - Fichub: Tags page exists but genres are mixed into all tag types — P2

---

## Work Page (/works/[id]) — Missing Components

**AO3 work page has these sections Fichub is missing or has differently:**

1. **Chapter navigation** (`/works/[id]/chapters/`)
   - AO3: Chapters listed as numbered list with "Entire Work" / "Chapter by Chapter" view selector at the top.
   - Fichub: `/read/[urlId]` has chapter selector — partially covered.

2. **Work meta block** (`<dl class="work meta">`)
   - AO3: Tags grouped by category (Fandoms, Characters, Relationships, Additional Tags, Warnings), Language, Series, Collections, Published date, Stats.
   - Fichub: ArchiveWork has partial meta — completed in Phase 3.

3. **Bookmark status** (on your own bookmarks)
   - AO3: When you've bookmarked a work, the page shows your bookmark tags/notes/count/chapter with a "Cancel Bookmark" button.
   - Fichub: No bookmark indicator on work pages — P1

4. **Mark for Later** (in a dropdown under Bookmark)
   - AO3: Bookmark → Mark for Later (separate list at /users/[you]/bookmarks/mark_for_later).
   - Fichub: No "Mark for Later" — P1

5. **Collections this work belongs to**
   - AO3: Shows list of collections that include this work.
   - Fichub: No — P2

6. **Co-bookmarked / co-signup suggestions**
   - AO3: "Readers also bookmarked" and "Recent comments" modules.
   - Fichub: No co-bookmark suggestions on work pages — P2

7. **Chapter comments**
   - AO3: Each chapter can have its own comment thread.
   - Fichub: Comments are work-level only — P2

---

## Search Advanced (/search) — Missing Facets

**AO3 search has these facet groups Fichub is missing:**

1. **Work History / Bookmarks / Collections** search (search within your own stuff) — P2
2. **Challenge search** (filter by gift-athon) — P3
3. **Sort by:** At least, Bookmarks, Comments, Hits, Kudos, Relevant — Fichub uses Hits + Random + Top, missing Bookmarks/Comments/Kudos sort — P1
4. **Completion status** filter (Completed / In-Progress / Not Updated recently) — P2
5. **Word count range** slider — P1
6. **Chapters count range** — P1
7. **Language** filter — P2
8. **Category** filter (F/F, M/M, etc.) — P1

---

## Reading History

**AO3:** `/users/[you]/history` — list of every work you've visited, with timestamps. Can delete individual entries or clear all. Shows "last visited" on work pages.

**Fichub:** **No reading history at all.**
**Priority:** P0
**Impact:** High — this is a core feature users expect. The user specifically called this out.

### Implementation needs:
- DB schema: `read_history` table (user_id, work_id, url_id, last_read_at, created_at)
- API endpoint: `GET /api/history` with pagination
- Frontend route: `/history` (or `/reading/history`)
- On work page / reader: POST `visited_at` timestamp
- Display last-visited in work listings
- Ability to delete/clear history entries

---

## Subscriptions

**AO3:** `/users/[you]/subscriptions` — lists all works/users/collections you're subscribed to. Each has email notification toggle. Can subscribe from work pages / user profiles / collection pages.

**Fichub:** `/follows` partially covers user-following. No work-level follows. No email toggle. No subscriptions management page.
**Priority:** P1

---

## Collections

**AO3 collections system:**
- Create collection (name, description, icon, visibility: public/private/anonymous, item selection: moderated/restricted)
- Add/remove works to collection
- Bookmark a collection
- View collection as: list of works, bookmarks with notes, reading list, random, series, etc.
- Collection challenges / gift-athons
- Collection members / maintainers / approvers

**Fichub:** `/lists` has lists (similar concept) but no collection-specific features (visibility, item selection modes, bookmarks-in-collection view).
**Priority:** P1

---

## Skin System

**AO3 full skin system:**
- Site skins (admin-approved, site-wide)
- Public skins (user-created, shared)
- Personal skins (private)
- Skin editor with CSS editor + preview
- Apply skins to site

**Fichub:** Basic theme system only (light/dark + accent color).
**Priority:** P2

---

## Comment System Enhancements

**AO3:**
- Comment threading (replies to replies)
- Comment editing (within 24h window)
- Comment deletion (with confirmation)
- Quote/block/ignore on commenters
- Email notification for replies to your comments
- Comment count shown on work listing thumbnails

**Fichub:**
- Flat comments only (no threading) — P1
- No comment editing/deletion — P1
- No per-user ignore — P2
- No email notifications for comment replies — P1

---

## Draft System

**AO3:**
- Save unpublished drafts of works
- Edit draft metadata/tags before publishing
- Delete drafts

**Fichub:** No works-creation flow at all (Fichub is currently a reader/scraper platform, not a creator platform).
**Priority:** P3 (depends on whether creation is in scope)

---

## Author Dashboard

**AO3:**
- `/works` — your works list with stats, edit links, delete
- `/works/[id]/edit` — full work editor (title, summary, tags, associations, chapters, series)
- `/works/[id]/delete` — delete with reason
- Series management: create / rename / reorder chapters
- Bulk upload via chapter import

**Fichub:** No author dashboard — P3 (creation not in current scope, but author profile improvements are P1)

---

## User Profile Customization

**AO3 profile features:**
- Avatar, bio (with HTML), badges display
- Favorite tags / characters / relationships display
- Lists: works, bookmarks, collections, series
- Guestbook/comments on profile
- Time zone setting (affects timestamp display)
- Profile visibility settings per-section

**Fichub:** `/authors/[name]` shows works list, no bio/badges/favorites.
**Priority:** P1

---

## Notification Preferences

**AO3 notification settings:**
- Email me when: someone comments on my work / replies to my comment / kudos my work / bookmarks my work / subscribes to my work / mentions me / collection invites / exchange assignments / works from followed subscriptions
- Digest instead of per-email
- Auto-follow works I comment on

**Fichub:** `/settings` has no notification preferences.
**Priority:** P1

---

## Blacklist / Hidden Filters

**AO3:**
- Hidden filters (block fandoms/tags/characters/relationships/warnings)
- Site-wide blocked tags
- User blocking (can't see their comments)

**Fichub:** `/settings` mentions blacklist but no UI for it.
**Priority:** P1

---

## Search History

**AO3:** `/users/[you]/search_history` — list of your recent searches with the same search form pre-filled with that query.

**Fichub:** No search history.
**Priority:** P2

---

## Site-wide Browse Pages

Missing browse-by-tag pages:
- `/tags/[id]` — tag detail (works, series, bookmarks, characters, relationships for this tag)
- `/characters` — alphabetical characters listing
- `/relationships` — alphabetical relationships listing
- `/collections` — site-wide collections browser
- `/series` — site-wide series browser (Fichub doesn't have this)

**Priority:** P2

---

## Reading Interface

**AO3 chapter reader (/works/[id]/chapters/[n]):**
- Chapter title in header
- Navigation: Previous / Next chapter / Table of Contents / Entire Work
- Bookmark/Mark for Later button in header
- Kudos button (with "thank" text box on first kudos)
- Comment count + scroll-to-comments
- Hit counter
- "Readers also bookmarked" suggestions at bottom
- "Series: ← → Next in series" links

**Fichub:** `/read/[urlId]` has chapter navigation but no kudos/bookmark-in-reader/comments-in-reader/suggestions.
**Priority:** P1

---

## Mobile Responsiveness

AO3's mobile experience:
- Collapsible navigation
- Mobile-friendly forms
- Touch-friendly buttons

**Fichub:** Modern mode is responsive; archive mode needs verification.
**Priority:** P2

---

## Priority Summary

| Priority | Feature | Route Needed |
|----------|---------|-------------|
| **P0** | Reading History | `/history` |
| **P0** | Last-visited on works | (inline) |
| **P1** | User Dashboard | `/dashboard` |
| **P1** | Bookmark status on work page | (inline) |
| **P1** | Mark for Later | `/bookmarks/mark-for-later` |
| **P1** | Work-level subscriptions | (inline) |
| **P1** | Per-work stats dashboard | `/works/[id]/stats` or inline |
| **P1** | Notification preferences | `/settings#notifications` |
| **P1** | Comment threading | (inline) |
| **P1** | Comment edit/delete | (inline) |
| **P1** | Profile customization | `/users/[name]/edit` |
| **P1** | Collections system | `/collections` |
| **P2** | Characters browse | `/characters` |
| **P2** | Relationships browse | `/relationships` |
| **P2** | Skins system | `/skins` |
| **P2** | Search history | `/search/history` |
| **P2** | Site-wide collections browse | `/collections` |
| **P2** | Fandom categories | `/fandoms` (categorize) |
| **P2** | Gift-athon system | `/challenges` |
| **P3** | Drafts | `/works/drafts` |
| **P3** | Author dashboard | `/works` |
| **P3** | Works creation/edit | `/works/[id]/edit` |
