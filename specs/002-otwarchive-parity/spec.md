# Feature Specification: OTW Archive Parity — Friction-Free AO3 Migration

**Feature Branch**: `002-otwarchive-parity`

**Created**: 2026-08-24

**Status**: Draft

**Input**: User description: "go through /personal/documents/code/ruby/otwarchive code and we are going to revise that our site has all the functionality from otwarchive polished so that someone coming from that archive doesn't feel any friction, the additional features should still work, I want to work on this bit by bit to manually test everything works and looks as similar as possible to original otwarchive"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse, filter, and open works like AO3 (Priority: P1)

An AO3 reader lands on FicHub and wants to find fics the same way they do on AO3: filter sidebar with Rating, Warning, Category, Fandom, Relationship, Character, Additional Tags, completion status, word count, date updated, language; sort by kudos/comments/bookmarks/hits/date; see work blurbs with AO3 header (title · author · fandom · rating/category/warnings squares · stats), and open a work.

**Why this priority**: Core reading flow — if browse/filter/blurb deviate, every AO3 user feels friction immediately.

**Independent Test**: As anonymous, open `/works` filter drawer, apply 3 filters + sort, verify URL persists, blurb shows AO3 squares/header parity, click through to `/works/{id}` and `/read/{id}` without layout shift.

**Acceptance Scenarios**:

1. **Given** anonymous on `/works`, **When** filters Rating=Mature, Category=F/M, Complete=Yes are applied, **Then** results, URL query, and blurb counts match filtered set and survive reload.
2. **Given** filtered list, **When** sorting by Kudos, **Then** order matches `kudos_count` and active sort chip is highlighted in both skins.
3. **Given** work blurb in list, **When** comparing to AO3 reference (title line, byline, fandom, 4-square symbols, stats line `words · chapters · kudos · bookmarks · hits · updated`), **Then** all elements present with AO3-like typography (archive vs modern both pass visual review).

---

### User Story 2 - Read, navigate, and download a work like AO3 (Priority: P1)

An AO3 reader opens a work, reads chapters, uses chapter nav, downloads EPUB/MOBI/PDF, and sees work meta (summary, notes, tags) with AO3 section order.

**Why this priority**: Second half of core reading — reading + download is the daily AO3 loop.

**Independent Test**: Open a multi-chapter work, verify header → meta → summary → notes → TOC → chapter body order matches AO3, navigate prev/next/chapter dropdown, download each format and open file.

**Acceptance Scenarios**:

1. **Given** work page, **When** viewing meta, **Then** sections appear in AO3 order (rating/warnings/category/fandoms/relationships/characters/tags → summary → notes → series) with archive typography.
2. **Given** multi-chapter work, **When** using chapter dropdown and prev/next, **Then** URL, scroll position, and progress track correctly and match AO3 nav affordances.
3. **Given** download menu, **When** requesting EPUB/HTML/MOBI/PDF/TXT, **Then** file downloads with AO3-equivalent content and filename convention.

---

### User Story 3 - Kudos, bookmarks, subscriptions, and history like AO3 (Priority: P2)

An AO3 user kudos a work (guest or logged-in, one per user), bookmarks with tags/notes (including private), subscribes to work/series/author, and reviews history — all where AO3 puts them.

**Why this priority**: Engagement loop — AO3 users expect these controls in the blurb/work footer and user menu.

**Independent Test**: Logged-in + guest kudos, create/edit private bookmark with tags, subscribe to work and author, check `/users/{me}/history` and `/users/{me}/bookmarks`.

**Acceptance Scenarios**:

1. **Given** logged-in on work page, **When** clicking Kudos, **Then** count increments once, button disables, undo not allowed (AO3 parity), guest kudos counted separately.
2. **Given** bookmark form, **When** saving with tags + private flag, **Then** bookmark appears in dashboard with AO3-like bookmark blurb and privacy respected.
3. **Given** subscriptions, **When** subscribing to work and author, **Then** entries appear in subscriptions list and notifications respect mute.

---

### User Story 4 - Comments, threads, and inbox like AO3 (Priority: P2)

An AO3 user posts a comment, replies in a thread, and manages replies in their inbox (reply, delete, mark read) — with guest commenting where allowed.

**Why this priority**: Social layer — comment threads are central to AO3 writer feedback.

**Independent Test**: Post top-level comment, reply nested, verify thread rendering, inbox shows new comment with actions.

**Acceptance Scenarios**:

1. **Given** work with comments enabled, **When** posting as logged-in and as guest (if allowed), **Then** comment appears in thread with correct byline and nesting.
2. **Given** inbox item, **When** replying inline, **Then** reply is threaded under parent and inbox marks read.
3. **Given** comment thread, **When** viewing in archive vs modern skin, **Then** indentation, byline, and action links match AO3 placement.

---

### User Story 5 - Create and edit works with pseuds, co-creators, series, and tags (Priority: P2)

An AO3 author creates a work (or imports one), picks pseud, adds co-creators, assigns series/collections, wrangles tags (fandom/relationship/character/freeform), and edits metadata/chapters.

**Why this priority**: Authoring — without pseud+co-creator+series parity, migrating authors cannot publish.

**Independent Test**: As author, `POST /works` with pseud, co-creator invite, series assignment; edit metadata and add chapter; verify wrangled tags link to tag pages.

**Acceptance Scenarios**:

1. **Given** author with 2 pseuds, **When** creating work under pseud B with co-creator C, **Then** byline shows `pseud B + C` and both dashboards list the work.
2. **Given** work in series, **When** viewing series page, **Then** works appear in AO3 series order with navigation prev/next.
3. **Given** tags with wrangling, **When** viewing tag page `/tags/{tag}`, **Then** tag type, canonical, and filtered works match AO3 behavior.

---

### User Story 6 - Collections, challenges, gift exchanges, and skins (Priority: P3)

AO3 power users use collections to curate, challenges/gift exchanges to run events, and skins to theme works/site — advanced but high AO3 identity.

**Why this priority**: Differentiator for fandom events; lower than core read/write but required for full parity.

**Independent Test**: Create collection, add work, run minimal challenge (signup → assignment → claim), apply user skin and work skin.

**Acceptance Scenarios**:

1. **Given** collection, **When** adding a work via work-collect form, **Then** work shows in collection with AO3 collection blurb and moderation queue if enabled.
2. **Given** challenge, **When** signing up and claiming, **Then** challenge flow matches OTW steps (offers/requests/potentials) and assignment visible.
3. **Given** user skin, **When** applying site skin, **Then** site chrome tints per skin with archive parity.

---

### Edge Cases

- Imported/scraped works vs natively created works: native authoring must not break cache-backed imports; imports retain source attribution.
- Guest vs logged-in gaps: guest kudos/comments/pseud flows degrade gracefully with login prompts matching AO3 copy.
- Scraped tags may be unwrangled: show raw tags with fallback search while wrangled tags link canonically.
- Large tag sets and long summaries: truncate/expand behavior matches AO3 (tag cloud, summary toggle).
- Collections/series with draft or hidden works: visibility respects draft/hidden state per OTW rules.
- Skins with unsafe CSS: sanitize like OTW (allowlist).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render work listings and work pages with AO3 section order, header, 4-square symbols, and stats line — matching OTWArchive reference views.
- **FR-002**: System MUST provide AO3-equivalent work filters (Rating, Warnings, Category, Fandom, Relationship, Character, Additional Tags, Completion, Word Count, Date Updated, Language) and sorts (updated, kudos, bookmarks, comments, hits, words) with URL-persisted query.
- **FR-003**: System MUST support chapter navigation (dropdown, prev/next, entire-work view) and downloads (EPUB/HTML/MOBI/PDF/TXT) with AO3-like placement and filenames.
- **FR-004**: System MUST support kudos (one per user per work, guest distinct), bookmarks (tags/notes/private/rec), and subscriptions (work/series/author) with AO3 placement and counts.
- **FR-005**: System MUST support threaded comments, guest comments (where allowed), and inbox management (reply/delete/mark-read) matching OTW comment/inbox flows.
- **FR-006**: System MUST support pseuds (multiple per user) and co-creatorship on works/series with shared bylines and dashboards.
- **FR-007**: System MUST support series and collections (curated, moderated, challenge-linked) with AO3 navigation and collection membership controls.
- **FR-008**: System MUST support tag wrangling (type, canonical, synonyms) and browsable tag pages with tag-filtered work listings.
- **FR-009**: System MUST support user skins and work skins (sanitized CSS) with AO3-like skin application.
- **FR-010**: System MUST keep existing FicHub features working (forum, requests, recs, progression, search enhancements, translations) — parity must not regress them; where OTW and FicHub overlap, AO3 placement wins visually but FicHub extensions remain accessible.
- **FR-011**: System MUST provide manual testability per slice — each phase ships with a checklist and screenshots against AO3 reference.

### Key Entities

- **Work**: Title, authors (pseuds + co-creators), fandoms, rating, warnings, categories, relationships, characters, freeforms, summary, notes, chapters, language, completion, word count, stats (kudos/bookmarks/hits/comments), series/collection links.
- **Pseud**: Alias per user, byline identity, co-creator target.
- **Series**: Ordered works, navigation, subscription target.
- **Collection / Challenge / Gift Exchange**: Curation and event container with moderation, signup/claims/assignments.
- **Bookmark**: User's saved work with tags/notes/private/rec flag.
- **Comment / Thread / Inbox**: Nested feedback with inbox actions.
- **Tag / Wrangling**: Type, canonical, synonyms, tag page.
- **Skin**: Site-wide or per-work CSS, sanitized and selectable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An AO3-familiar tester can complete all P1 flows (filter → open → read → download) on FicHub without guidance and report zero missing placement compared to AO3 reference screenshots (manual checklist 100% pass per phase).
- **SC-002**: 90% of AO3 reference views checked (work index, work show, chapter, kudos/bookmark/subscribe controls, comment thread, inbox, pseud picker, series, collection) match section order and control placement within one visual review cycle (before/after screenshots).
- **SC-003**: All P1+P2 user journeys have a manual test script that passes end-to-end on both archive and modern skins, with no regression to FicHub-native features (forum/requests/recs/progression).
- **SC-004**: Parity work is shippable slice-by-slice: each increment (browse, read/download, kudos/bookmarks/subscriptions, comments/inbox, pseuds/series) can be deployed and manually verified independently without breaking prior slices.

## Assumptions

- OTWArchive reference is `/personal/documents/code/ruby/otwarchive` (github.com/otwcode/otwarchive) — views/controllers/models examined slice-by-slice.
- FicHub retains its Rust/Axum + SvelteKit + Postgres stack; parity is visual/behavioral (templates, routes, controls), not Rails porting.
- Additional FicHub features (forum, requests, Ask, progression, Marginalia, OPDS) remain; where OTW and FicHub overlap, AO3 visual placement is canonical and FicHub extras are additive (secondary nav, not displacement).
- Work is incremental: each story is one polished vertical slice with manual QA before next slice.
- Tag wrangling can reuse existing tag tables; full OTW wrangling admin is phased.

