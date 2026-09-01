# Feature Specification: OTW Roadmap 2013 Gaps

**Feature Branch**: `003-otw-roadmap-2013`

**Created**: 2026-08-24

**Status**: Draft

**Input**: User description: "specification to build anything missing from the otwarchive roadmap" — source is the OTW AO3 News post *Archive Roadmap 2013* (published Wed, 27 Mar 2013), covering Interlude Rails 3, Version 0.8 (shipped), and planned Versions 0.9 / Interlude / 0.10 / 0.11 / 1.0-and-beyond.

Reference code: `/personal/documents/code/ruby/otwarchive` + `/home/alvaro/code/rust/otwarchive`

**Related**: `specs/002-otwarchive-parity` already scopes friction-free AO3 migration (browse/filter/blurb, read/download, kudos/bookmarks/subscriptions, comments/inbox, pseuds/co-creators/series/tags, collections/challenges/skins). This spec scopes **remaining roadmap items not covered by 002** — media-type posting, work relationships, subscriptions/history expansion, i18n, anon/drafts/chapters, admin roles, messaging, profiles, API/package — delivered slice-by-slice and verified against OTW reference views.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Post and find works by media type and fiction type (Priority: P1)

An author posts fanart, podfic, or a vid and marks it as that media type (not just a tag); a meta essay author marks their work as non-fiction. A reader filters the archive to "only Fanart" or "only Nonfiction/Meta" or "embeddable video works" and finds exactly those works.

This is roadmap §0.9: "specify a media type for your fanwork when posting (instead of relying on Additional Tags alone), and to filter by media type when browsing" plus "options for posting and searching for non-fiction fanworks (fannish meta and other non-fiction types)."

**Why this priority**: Roadmap calls it "the first major enhancement in this release cycle" for 0.9 and a prerequisite for true multimedia support. Without it, media-type browsing is tag-guesswork — core discovery friction.

**Independent Test**: As author, post one Fanart and one Meta work with the new posting field; as anonymous, use Work Search / browse to filter `media_type=art` and `fiction_type=nonfiction`; verify blurb, search URL, and filtered counts are correct on archive and modern skins.

**Acceptance Scenarios**:

1. **Given** author on Post New Work, **When** choosing Media Type (Text / Image / Audio / Video / Embed) and Fiction Type (Fiction / Non-fiction / Meta), **Then** the work saves with that type visible in the work header/meta and editable later.
2. **Given** mixed works, **When** filtering Media=Art or Fiction=Nonfiction on `/works`, **Then** only matching works appear and the filter state persists in the URL and survives reload.
3. **Given** a work with embeds (YouTube/podfic player), **When** viewing the work, **Then** embeds render safely (allowlisted) and the media type badge matches the posting choice.

---

### User Story 2 - Work relationships, remixes, translations, and inspiration (Priority: P1)

An author translates another work, remixes it, marks it "Inspired by," or gives a gift; the relationship shows on both works and is browsable, matching roadmap §0.9 "Improvements to the handling of work relationships (remixes, translations, gifts...) across the site."

Existing FicHub has chapter translations, but not OTW-style approved work-level translation/rewrite relationships with reciprocal links and search.

**Why this priority**: Roadmap lists it as a distinct 0.9 deliverable; without it, derived works are disconnected and discoverability breaks.

**Independent Test**: Create Work B "Translation of Work A" (with permission flow or explicit link), verify both work pages show reciprocal "Translation / Original" links and that a visit to either navigates to the other.

**Acceptance Scenarios**:

1. **Given** author with rights to relate works, **When** linking Work B to Work A as Translation / Remix / Inspired By / Gift, **Then** both works display the relationship in header/meta and link to each other.
2. **Given** related works, **When** browsing, **Then** a search can include or exclude translations/remixes (as a filter facet) where OTW exposes it.
3. **Given** a gifted work, **When** recipient views their gifts dashboard/inbox, **Then** the gift appears there as on OTW.

---

### User Story 3 - Expanded subscriptions, history, and better filtering (Priority: P1)

A reader subscribes exactly to what they want (author, work, series, collection, tag/fandom) and manages subscriptions; their reading History is improved and searchable, matching roadmap §0.9 "filtering for bookmarks... making personal bookmarks truly useful," §0.9 tag-filter improvements, §0.10 "More subscription options... Subscription management will be improved" and "Improvements to the History feature."

**Why this priority**: Roadmap promotes subscriptions/history from nice-to-have to core retention; current FicHub subscriptions/history are basic.

**Independent Test**: Subscribe to a tag and an author; manage (unsubscribe/mute) on a subscriptions page; check History lists works read with date and allows clearing.

**Acceptance Scenarios**:

1. **Given** logged-in reader, **When** subscribing to a work, series, author (pseud), collection, and a tag, **Then** each subscription type appears in one management page and can be removed/muted independently.
2. **Given** subscriptions, **When** a new chapter/collection item matching the subscription is posted, **Then** a notification/email entry (or feed) matches OTW behavior for that subscription type.
3. **Given** History, **When** reading several works, **Then** `/users/{me}/readings` shows them in reverse chronological order with a clear-history control matching OTW.

---

### User Story 4 - Anonymous posting, drafts, and chapter prologues/epilogues (Priority: P2)

A writer posts anonymously to an anonymous collection or uses anonymous posting where allowed; they manage drafts robustly (autosave/restore, multiple drafts); and they structure a multi-chapter work with explicit Prologue / Chapters / Epilogue markers, matching roadmap §0.10: "Anonymous posting," "Improved drafts management," "Better chapter management, including the ability to indicate prologues and epilogues."

**Why this priority**: Directly named as 0.10 deliverables; drafts/chapters affect every authoring session.

**Independent Test**: Create an anonymous-eligible collection, post anonymously, verify work hides byline until reveal; create draft, leave and return, verify draft restored; create 3-chapter work marking chapter 1 as Prologue and chapter 5 as Epilogue.

**Acceptance Scenarios**:

1. **Given** anonymous posting enabled (e.g., anonymous collection), **When** posting anonymously, **Then** work byline shows "Anonymous" to others, reveals only per collection/moderation rules, and book-keeping keeps true author private.
2. **Given** author mid-post, **When** saving a draft and returning later, **Then** draft restores verbatim and can be previewed, edited, or discarded without creating a work.
3. **Given** multi-chapter work, **When** chapters are marked Prologue/Epilogue, **Then** chapter nav and TOC show those labels (e.g., "Prologue: ...", "Epilogue: ...") matching OTW chapter show.

---

### User Story 5 - Private messaging (Priority: P2)

A user sends a private message to another user and manages their inbox, matching roadmap §0.10 "private messaging, one of the most-requested features."

Note: must be scoped to avoid abuse — rate-limited, blockable, and not a replacement for comments.

**Why this priority**: Roadmap singles it out as most-requested for 0.10; social layer beyond comments/inbox.

**Independent Test**: As Alice send a PM to Bob; as Bob verify inbox badge, read, reply, and block; verify blocked sender cannot resend.

**Acceptance Scenarios**:

1. **Given** logged-in Alice, **When** composing a PM to Bob from Bob's profile or inbox, **Then** Bob receives it in their messages inbox with unread badge.
2. **Given** PM thread, **When** Bob replies, **Then** messages thread in conversation order and both can delete/archive per OTW rules.
3. **Given** a blocked user, **When** blocked user attempts to PM, **Then** the action is rejected with an explanatory message and no notification is created.

---

### User Story 6 - Translatable UI and site skins documentation (Priority: P2)

A non-English speaker uses FicHub with interface strings in their language; a site admin/localization volunteer can contribute translations, matching roadmap §0.8/0.10: "site skins to customize appearance," "work skins to style works," §0.10 "Translation features for an Archive in many different languages... all site elements are translatable and complete the necessary translation tools for volunteers." This also covers roadmap §0.9 "A new header and footer layout, for easier site navigation" and footer (Site Map / TOS / Policies / Contact / Development links).

FicHub already has 6 locales (en/de/es/fr/pt-BR/zh) and site/work skins basics; this story completes coverage and reference layout.

**Why this priority**: Roadmap calls translatable interface "one of our main goals when we started"; accessibility promise.

**Independent Test**: Switch language to one of the 6 locales; verify header/nav/search/about, footers, and flash messages translate; verify header/footer order matches OTW site map.

**Acceptance Scenarios**:

1. **Given** locale set to `de` or `zh`, **When** browsing work index, work show, and profile pages, **Then** all chrome strings render in that locale with no raw i18n keys, matching translation files.
2. **Given** skins, **When** viewing site skin picker and work skin assignment, **Then** both list approved skins and apply as on OTW (site chrome vs work body).
3. **Given** footer, **When** viewing any page, **Then** footer exposes About / Site Map / TOS / Content Policy / Privacy / DMCA / Contact / Development links mirroring OTW footer (adapted for FicHub).

---

### User Story 7 - Admin roles, tag wrangling management, collections/challenges polish, and Open Doors (Priority: P3)

Admins operate with distinct roles (abuse, wranglers, open doors, support, translations) and tools; wranglers manage tag hierarchies; collections/challenges/prompt memes are bug-fixed and documented; Open Doors can import at-risk archives — matching roadmap §0.8/0.9/0.10: "tag wrangling management features," "review and rewrite ... to prepare for art/video/audio," "Refinements to the collection and challenge code... major bug fixes for gift exchanges, prompt memes and tag sets," "Improved and expanded tools for Open Doors," "more robust admin system... clearly defined roles."

**Why this priority**: Ops/admin story — lower than reader/author facing but required for roadmap completeness.

**Independent Test**: Verify at least two distinct admin roles exist with different permissions; wrangle a fandom tag as canonical; create an import batch (Open Doors style) and verify it creates works with attribution.

**Acceptance Scenarios**:

1. **Given** admin roles, **When** an abuse-only admin tries to access wrangling batch import, **Then** access is denied while wrangler role can access wrangling pages.
2. **Given** tag wrangling page, **When** marking a tag canonical and merging synonyms, **Then** tag pages canonicalize and filtered works merge.
3. **Given** an at-risk-archive import batch (Open Doors), **When** importing a small dump, **Then** works appear with original author attribution and import collection link.

---

### User Story 8 - Public API and accessible, test-covered package (Priority: P3)

A developer builds a reader app against FicHub's public API for works/bookmarks/reading; every page passes accessibility checks (functional → accessible → attractive) and automated tests mirror roadmap §0.11/1.0: roadmap §1.0 "Public API... API hooks for developers to build upon our codebase for posting/bookmarking/reading/etc. tools," §1.0 "A default interface that is functional, accessible, and attractive (in that order)..." and §0.11 "review our wide range of automated tests to ensure that all our features have complementary test suites."

FicHub has backend APIs but not a documented public API with keys/rate limits; accessibility/testing are ongoing.

**Why this priority**: Roadmap's definition of done for 1.0 — stable package + API + tests.

**Independent Test**: Obtain an API key, `GET /api/v1/works?filter[media]=art` and paginate; verify OpenAPI docs; run axe tests on work index/show.

**Acceptance Scenarios**:

1. **Given** API key, **When** calling `GET /api/v1/works`, `GET /api/v1/works/{id}`, `GET /api/v1/bookmarks` with filters/pagination, **Then** responses match documented JSON:API/OpenAPI and obey rate limits.
2. **Given** any major page (work index/show/chapter/post form), **When** running axe/WCAG 2.2 checks, **Then** zero critical accessibility violations.
3. **Given** CI, **When** checking coverage, **Then** each shipped roadmap slice has page/route/API tests matching its spec (no slice ships without tests).

---

### Edge Cases

- Imported vs native works must interoperate across all new features (media type, relationships, anon/drafts, API) — native flows must not break cache-backed imports which retain source attribution.
- Private messaging and anonymous posting must be abuse-resistant: rate limits, block/mute, report-to-abuse, and no email leakage; anon authorship must not be inferable from API/timing.
- Translation/relationship workflows must not allow uncredited appropriation — permission/claim steps required where OTW requires them.
- Draft autosave must not lose data on navigation or crash: debounce, localStorage backup, and server-confirmed save state.
- Prologue/Epilogue chapter markers must not break existing multi-chapter nav/history/subscriptions.
- I18n must fall back to `en` for missing keys and never render raw keys; skins must be sanitized (OTW allowlist) so injected CSS cannot exfiltrate.
- Admin role separation must be enforced server-side; frontend role checks are not authoritative.
- Large imports (Open Doors) must be batched and idempotent (re-import does not duplicate works).
- API must be versioned (`/api/v1`), paginated, and rate-limited; breaking changes require new version.
- Audio/video embeds and image uploads (if any future file hosting) must be sandboxed and content-type validated.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST allow authors to specify Media Type and Fiction Type when posting/editing a work and display it in work header/meta; readers MUST be able to filter/browse by Media Type and Fiction Type with URL-persisted query.
- **FR-002**: System MUST support work relationships: Translation, Remix, Inspired By / Prequel / Sequel, and Gift; relationships MUST be reciprocal (visible on both works), browsable, and permission-gated where OTW requires permission.
- **FR-003**: System MUST provide expanded subscriptions: subscribe to work, series, collection, pseud/author, and tag/fandom; MUST provide a single management page with per-subscription remove/mute and matching notification/feed delivery.
- **FR-004**: System MUST provide improved History: reverse-chronological listing of read works with date, pagination, and clear-history, matching OTW readings behavior and placement.
- **FR-005**: System MUST support anonymous posting via eligible collections/challenges: byline masked to "Anonymous," reveal only per collection rules, true author kept private and not inferable via API.
- **FR-006**: System MUST provide drafts management: create, autosave (debounced), restore, preview, and discard drafts without creating a work; drafts MUST survive navigation/reload.
- **FR-007**: System MUST support chapter-level Prologue/Epilogue markers; chapter nav, TOC, and subscription/history entries MUST surface those labels matching OTW chapter display.
- **FR-008**: System MUST provide private messaging between logged-in users with threaded conversations, unread badge, reply/archive/delete, block/mute, rate limiting, and abuse reporting; frontend controls MUST NOT be authoritative.
- **FR-009**: System MUST keep chrome fully translatable (header/nav/footer/flash messages) across supported locales with fallback to `en` and no raw keys; footer/header layout MUST mirror OTW site map (About / Site Map / Policies / Contact / Development) adapted for FicHub.
- **FR-010**: System MUST provide admin role separation (at minimum: abuse, wrangler, open-doors/import, support/translations) with role-gated routes enforced server-side; collection/challenge/prompt-meme flows MUST remain usable and bug-fixed.
- **FR-011**: System MUST provide tag-wrangling management (canonical, synonyms, hierarchy) and Open Doors-style archive import (batched, idempotent, with original attribution and import collection link).
- **FR-012**: System MUST expose a documented Public API (`/api/v1`) for works, bookmarks, subscriptions/history (read scope), with pagination, filtering, auth, and rate limits, plus OpenAPI spec and versioning.
- **FR-013**: System MUST preserve all existing FicHub features (forum, Ask, requests/recs, progression, search enhancements, chapter translations, Marginalia, OPDS, offline/PWA) — roadmap work MUST NOT regress them; where OTW and FicHub overlap, AO3 visual placement is canonical and FicHub extras remain accessible.
- **FR-014**: System MUST be accessible (WCAG 2.2 AA, functional → accessible → attractive) and ship each slice with manual QA checklist + screenshots against OTW reference views and automated page/route/API tests.

### Key Entities

- **Work**: Posting includes Media Type (Text / Image / Audio / Video / Embed) + Fiction Type (Fiction / Non-fiction / Meta) plus OTW header/meta (title, authors/pseuds, fandoms, rating/warnings/categories, ships/characters/tags, language, word count); relationships to other works via typed link.
- **Work Relationship**: Typed link between two works (Translation, Remix, Inspired By, Gift, etc.) with permission/status, visible reciprocally and browsable.
- **Subscription**: User→target link (work / series / collection / pseud / tag / fandom) with mute/unmute, feeding notifications/feeds.
- **History / Reading**: Timestamped record of works read/chapters visited by a user, paginated and clearable.
- **Draft**: Unsaved work/chapter content with autosave state, preview, and owner-only access, not yet a published work.
- **Chapter**: Ordered part of a work with optional Prologue/Epilogue marker; nav/TOC label accordingly.
- **Anonymous Posting**: Work whose public byline is "Anonymous" via an anonymous collection/challenge; true author link private until reveal.
- **Private Message / Conversation**: Direct message between users, threaded, with read state, block/rate-limit, and abuse report.
- **Locale / Translation**: Interface strings per locale with fallback; not to be confused with work-language tag or chapter translation feature.
- **Tag / Wrangling**: Fandom/relationship/character/freeform tag with canonical + synonyms + hierarchy; merging canonicalizes filtered listings.
- **Collection / Challenge / Prompt Meme**: Curated event/curated collection with moderation; challenge phases (signup/assignment/claims).
- **Import Batch (Open Doors)**: Batched import of at-risk archive works with source attribution and import collection; idempotent re-import.
- **Public API Key**: Credential for `/api/v1` with scopes, rate limits, and versioning.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An AO3-familiar tester can complete every P1–P2 journey (filter by media, relate works, subscribe/history, draft/anonymous, prologue/epilogue, PM) without guidance and report zero missing placement versus OTW reference screenshots (slice checklist 100% pass).
- **SC-002**: All Filter/Search/Browse controls that roadmap §0.9 names (media type, category, language, completion, new header/footer layout) are present and URL-persisted, verified by before/after screenshots per slice on both archive and modern skins.
- **SC-003**: No regression to existing FicHub features: forum/ask/requests/recs/progression/chapter-translations/Marginalia/OPDS/PWA still pass their prior tests and manual QA after each roadmap slice.
- **SC-004**: Roadmap work is shippable slice-by-slice: each user story can be deployed and manually verified independently (browse media → relationships → subscriptions/history → anon/drafts/chapters → messaging → i18n/skins → admin/wrangling/imports → API/accessibility) without breaking prior slices.
- **SC-005**: Accessibility and API done: zero axe critical violations on major pages; `GET /api/v1/works` with pagination/filter, authenticated bookmarks/history read, documented via OpenAPI, rate-limited and versioned.

## Assumptions

- Roadmap source is the March 27 2013 Archive Roadmap post quoted above; versions 0.8 are considered shipped (kudos, subscriptions/feeds, stats, site/work skins, embeds, drafts baseline) and not re-scoped except where polishing is needed.
- Reference implementation is `/personal/documents/code/ruby/otwarchive` (github.com/otwcode/otwarchive); views in `app/views/{works,chapters,collections,tags,users}` and models in `app/models` are the visual/behavioral spec, not a code port.
- FicHub retains Rust/Axum + SvelteKit + Postgres + pgvector + Redis + adapter-static; parity is behavioral/visual (templates, routes, controls), not Rails.
- FicHub extras are additive: where OTW and FicHub overlap, AO3 visual placement is canonical but FicHub extras (scraped imports, cache layer, explicit Ask/Requests, Recs, Progression, Marginalia, OPDS, PWA) remain accessible.
- Roles: anon collections control reveal; PM block/rate-limit/report are mandatory; all permission checks are backend-authoritative.
- I18n: the 6 existing locales (en/de/es/fr/pt-BR/zh) are the baseline; new strings are added to `src/lib/i18n/dictionaries/*.ts` and never render raw keys.
- Delivery: one polished vertical slice at a time with manual screenshot QA before next slice — same process as `002-otwarchive-parity`.

## Dependencies

- `002-otwarchive-parity` should ship its P1–P2 slices before 003 P1 where overlapping (browse/filter/read/download/kudos/bookmarks/subscriptions/comments/pseuds/series), to avoid duplicate filter/tag/series work.
- Data model migrations may build on 002 work (works/tags/series/collections) — prefer reusing tables over duplicating.
