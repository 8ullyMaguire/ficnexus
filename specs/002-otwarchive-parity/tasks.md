# Tasks: OTW Archive Parity — Friction-Free AO3 Migration

**Input**: Design documents from `/specs/002-otwarchive-parity/`
**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/api.md](./contracts/api.md)
**Branch**: `002-otwarchive-parity`

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, branch, and reference baseline

- [ ] T001 Create feature branch `002-otwarchive-parity` from `main` and set `GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-fichub`
- [ ] T002 Audit OTW reference views in `/personal/documents/code/ruby/otwarchive/app/views/works/` and capture before screenshots for work index/show/chapter per `quickstart.md`
- [ ] T003 Verify existing test harness `cargo test --lib` and `vitest` with `page.test.ts` convention and `svelte-check` 0 errors baseline

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infra that MUST be complete before ANY user story can be implemented — DB migrations, pseud/creatorship plumbing, routing base

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Create migration `070_pseuds_creatorships.sql` for pseuds and creatorships per `data-model.md` in `migrations/070_pseuds_creatorships.sql`
- [ ] T005 Create migration `071_series_collections_skins.sql` for series, collection_items, skins tables in `migrations/071_series_collections_skins.sql`
- [ ] T006 Implement server route scaffolding in `src/server.rs` for new groups `/api/works`, `/api/pseuds`, `/api/series`, `/api/collections`, `/api/skins`, `/api/kudos`, `/api/bookmarks`, `/api/subscriptions`, `/api/comments`, `/api/inbox`
- [ ] T007 Implement auth helpers for pseud-aware byline in `src/routes/pseuds.rs` (pseud resolution + default pseud creation on migration)
- [ ] T008 Add i18n keys audit script and dictionary placeholders in `frontend/src/lib/i18n/dictionaries/en.ts` (and de/es/fr/pt-BR/zh parity)

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Browse, filter, and open works like AO3 (Priority: P1) 🎯 MVP

**Goal**: AO3-equivalent filters (Rating/Warnings/Category/Fandom/Relationship/Character/Freeform/Complete/Word Count/Date Updated/Language), sorts, and work blurb with 4-square symbols and stats line

**Independent Test**: As anon on `/works`, apply 3 filters + sort, verify URL persists + blurb matches AO3 section order/header; click through to `/works/{id}` without layout shift (see `quickstart.md` Slice 1)

### Tests for User Story 1

- [ ] T009 [P] [US1] Contract test for `GET /api/works` filter+sort+pagination in `src/routes/works.rs` (cargo test) and `frontend/src/routes/works/page.test.ts`
- [ ] T010 [P] [US1] Visual regression test for WorkBlurb AO3 header order in `frontend/src/lib/components/WorkBlurb.test.ts`

### Implementation for User Story 1

- [ ] T011 [P] [US1] Implement work filter query parsing (rating/warnings/category/fandom/relationship/character/freeform/complete/word_count/language/sort) in `src/routes/works.rs`
- [ ] T012 [P] [US1] Implement work blurb serializer with OTW `_meta`/`_blurb` section order and 4-square symbols in `src/routes/works.rs`
- [ ] T013 [US1] Build filter drawer/sidebar with AO3 facets and URL-persisted state in `frontend/src/routes/works/+page.svelte`
- [ ] T014 [US1] Polish WorkBlurb component to AO3 typography (archive vs modern skins) in `frontend/src/lib/components/WorkBlurb.svelte`
- [ ] T015 [US1] Add sort chips (updated/kudos/bookmarks/comments/hits/words) with active highlight in `frontend/src/routes/works/+page.svelte`
- [ ] T016 [US1] Wire empty-state and large-tag handling (truncate/expand like AO3) in `frontend/src/routes/works/+page.svelte`

**Checkpoint**: User Story 1 fully functional — filtered browse → blurb → work open works on both skins

---

## Phase 4: User Story 2 - Read, navigate, and download a work like AO3 (Priority: P1)

**Goal**: Work show in AO3 section order (header→meta→summary→notes→TOC→chapter body), chapter nav (dropdown/prev/next/entire-work), and downloads (EPUB/HTML/MOBI/PDF/TXT)

**Independent Test**: Open multi-chapter work, verify section order matches OTW `works/show.html.erb`, navigate via chapter dropdown/prev/next, download each format (see `quickstart.md` Slice 1b)

### Tests for User Story 2

- [ ] T017 [P] [US2] Contract test for `GET /api/works/{id}` and `GET /api/works/{id}/chapters` in `src/routes/chapters.rs`
- [ ] T018 [P] [US2] Integration test for download formats in `src/routes/export.rs` and `frontend/src/routes/read/[urlId]/page.test.ts`

### Implementation for User Story 2

- [ ] T019 [P] [US2] Implement work show serializer with AO3 section order (rating/warnings/category/fandoms/relationships/characters/tags → summary → notes → series) in `src/routes/works.rs`
- [ ] T020 [P] [US2] Implement chapter navigation (dropdown, prev/next, `?view_full_work`) in `src/routes/chapters.rs`
- [ ] T021 [US2] Build work show page with TOC and chapter body order in `frontend/src/routes/works/[workId]/+page.svelte`
- [ ] T022 [US2] Build chapter management UI with position editing in `frontend/src/routes/works/[workId]/chapters/+page.svelte`
- [ ] T023 [US2] Wire download menu (EPUB/HTML/MOBI/PDF/TXT) with OTW filename conventions in `frontend/src/lib/components/WorkActions.svelte` and `src/routes/export.rs` verification
- [ ] T024 [US2] Ensure archive skin typography and offline/PWA chapter caching parity in `frontend/src/lib/pwa/sw.ts` and `frontend/static/styles/zerafina-skin.css`

**Checkpoint**: User Stories 1 AND 2 work independently — browse→read→download loop complete

---

## Phase 5: User Story 3 - Kudos, bookmarks, subscriptions, and history like AO3 (Priority: P2)

**Goal**: Kudos (one per user, guest distinct, no undo), bookmarks with tags/notes/private/rec, subscriptions (work/series/pseud/user), and history

**Independent Test**: Logged-in + guest kudos, create/edit private bookmark with tags, subscribe to work+author, check history/bookmarks dashboards

### Tests for User Story 3

- [ ] T025 [P] [US3] Contract test for kudos (idempotency, guest dedup) in `src/routes/kudos.rs` and `frontend/src/lib/api/social.test.ts`
- [ ] T026 [P] [US3] Contract test for bookmarks (private/rec/tags) in `src/routes/bookmarks.rs`

### Implementation for User Story 3

- [ ] T027 [P] [US3] Implement kudos handler (one per user per work, guest distinct, no delete) in `src/routes/kudos.rs`
- [ ] T028 [P] [US3] Implement bookmarks CRUD with wrangled tag links, private/rec flags in `src/routes/bookmarks.rs`
- [ ] T029 [P] [US3] Implement subscriptions (work/series/pseud/user) with unique constraint in `src/routes/subscriptions.rs`
- [ ] T030 [US3] Implement history/readings endpoint with pagination in `src/routes/subscriptions.rs`
- [ ] T031 [US3] Build Kudos button with AO3 placement and disabled-after-kudos in `frontend/src/lib/components/KudosButton.svelte`
- [ ] T032 [US3] Build BookmarkForm with tags/notes/private/rec in `frontend/src/lib/components/BookmarkForm.svelte`
- [ ] T033 [US3] Build subscriptions and history pages under `frontend/src/routes/users/[userId]/subscriptions/+page.svelte` and `frontend/src/routes/history/+page.svelte`

**Checkpoint**: P2 engagement loop works without breaking P1 browse/read

---

## Phase 6: User Story 4 - Comments, threads, and inbox like AO3 (Priority: P2)

**Goal**: Threaded comments, guest comments where allowed, inbox (reply/delete/mark-read) with AO3 placement

**Independent Test**: Post top-level + nested reply, verify thread rendering; inbox shows new comment with reply/mark-read actions

### Tests for User Story 4

- [ ] T034 [P] [US4] Contract test for comment threads and guest flow in `src/routes/comments.rs`
- [ ] T035 [P] [US4] Contract test for inbox actions (reply/delete/mark-read) in `src/routes/comments.rs`

### Implementation for User Story 4

- [ ] T036 [P] [US4] Implement comment create/reply with nesting and guest fields in `src/routes/comments.rs`
- [ ] T037 [P] [US4] Implement inbox list and actions (reply inline, delete, mark-read) in `src/routes/comments.rs`
- [ ] T038 [US4] Build CommentThread component with AO3 indentation/byline/action links in `frontend/src/lib/components/CommentThread.svelte`
- [ ] T039 [US4] Build Inbox page with thread actions in `frontend/src/routes/inbox/+page.svelte`
- [ ] T040 [US4] Wire comment permissions (guest allowed where work allows) and archive vs modern skin parity in `frontend/src/lib/components/CommentThread.svelte`

**Checkpoint**: Social layer complete and independently testable

---

## Phase 7: User Story 5 - Create and edit works with pseuds, co-creators, series, and tags (Priority: P2)

**Goal**: Pseud picker, co-creatorship invites, series/collection assignment, tag wrangling, metadata/chapter edits

**Independent Test**: As author with 2 pseuds, `POST /works` under pseud B with co-creator C; verify both dashboards list work; series page shows ordered nav; tag page canonicalizes

### Tests for User Story 5

- [ ] T041 [P] [US5] Contract test for pseud CRUD and creatorship invite/approve in `src/routes/pseuds.rs`
- [ ] T042 [P] [US5] Contract test for series ordering and tag wrangling pages in `src/routes/series.rs`

### Implementation for User Story 5

- [ ] T043 [P] [US5] Implement pseud CRUD (name unique per user, one default) in `src/routes/pseuds.rs`
- [ ] T044 [P] [US5] Implement creatorship invite/approve flow and shared byline/dashboard in `src/routes/pseuds.rs`
- [ ] T045 [P] [US5] Implement series create/ordered works/prev-next nav in `src/routes/series.rs`
- [ ] T046 [P] [US5] Implement tag pages and wrangling (canonical/synonyms) lookup in `src/routes/tags.rs`
- [ ] T047 [US5] Build post/edit work form with pseud picker and co-creator field in `frontend/src/routes/works/new/+page.svelte`
- [ ] T048 [US5] Build series page with ordered works and navigation in `frontend/src/routes/series/[id]/+page.svelte`
- [ ] T049 [US5] Build tag page with type/canonical/filtered works in `frontend/src/routes/tags/[name]/+page.svelte`

**Checkpoint**: Authoring slice complete — migrating authors can publish with AO3 parity

---

## Phase 8: User Story 6 - Collections, challenges, gift exchanges, and skins (Priority: P3)

**Goal**: Collections (curated/moderated/challenge-linked), minimal challenge flow (offers/requests/potentials→assignment→claim), and user/work skins (sanitized CSS)

**Independent Test**: Create collection, add work via collect form, run minimal challenge signup→assignment→claim, apply site skin and work skin

### Tests for User Story 6

- [ ] T050 [P] [US6] Contract test for collection add and moderated queue in `src/routes/collections.rs`
- [ ] T051 [P] [US6] Contract test for skins (sanitized CSS, apply/assign) in `src/routes/skins.rs`

### Implementation for User Story 6

- [ ] T052 [P] [US6] Implement collections CRUD and collection_items with moderated queue in `src/routes/collections.rs`
- [ ] T053 [P] [US6] Implement challenge flow (signup/offers/requests, potentials, assignments, claims) in `src/routes/collections.rs`
- [ ] T054 [P] [US6] Implement skins CRUD with CSS sanitization (allowlist) and assignment in `src/routes/skins.rs`
- [ ] T055 [US6] Build collection page with AO3 blurb and moderation controls in `frontend/src/routes/collections/[id]/+page.svelte`
- [ ] T056 [US6] Build challenge signup/assignment/claim pages in `frontend/src/routes/collections/[id]/challenge/+page.svelte`
- [ ] T057 [US6] Build SkinPicker and work skin application with chrome tint in `frontend/src/lib/components/SkinPicker.svelte`
- [ ] T058 [US6] Wire site skin selection and preview in `frontend/src/routes/skins/+page.svelte`

**Checkpoint**: All 6 user stories independently functional

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories — docs, a11y, and regression gates

- [ ] T059 Audit WorkBlurb `page.test.ts` for AO3 parity and bump coverage in `frontend/src/lib/components/WorkBlurb.test.ts`
- [ ] T060 Run quickstart.md Slice 1–6 manual QA and capture before/after screenshots vs `otwarchive/app/views/` per slice
- [ ] T061 Verify zero regression to FicHub extras (forum/Ask/requests/recs/progression/translations/Marginalia/OPDS/PWA) per `quickstart.md`
- [ ] T062 Update `docs/brainstorm-02-roadmap-status.md` and `docs/NEXT-STEPS.md` to mark 002 slices shipped and cross-link to `specs/002-otwarchive-parity/`
- [ ] T063 Run `cargo check` and `svelte-check` with 0 errors and `cargo test --lib` green before each slice push
- [ ] T064 Ensure route tests use `page.test.ts` (not `+page.test.ts`) and add ArchiveButton verification where AO3 buttons appear

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational completion
  - P1 stories (US1, US2) → MVP first
  - P2 stories (US3, US4, US5) → after P1, can parallelize if staffed
  - P3 story (US6) → after P2
- **Polish (Phase 9)**: Depends on all desired stories being complete

### User Story Dependencies

- **US1 (P1) Browse/filter/blurb**: No dependencies on other stories; needs T004–T008
- **US2 (P1) Read/download**: Depends on US1 work serializer shape but independently testable once US1 filter infra lands
- **US3 (P2) Kudos/bookmarks/subs**: Independent of US4/US5/US6; touches same WorkBlurb placement as US1
- **US4 (P2) Comments/inbox**: Independent; shares pseud resolution with US5 but not blocked by it
- **US5 (P2) Pseuds/series/tags**: Independent; provides pseud infra that US3/US4 bylines consume
- **US6 (P3) Collections/challenges/skins**: Depends on US5 series/pseud shape for collection ownership; can mock if parallel

### Within Each User Story

- Contract/integration tests → models/migrations → services/handlers → frontend components → integration/manual QA
- Story complete before next priority unless explicitly parallelized

### Parallel Opportunities

- T009 ∥ T010 (US1 tests, different files)
- T011 ∥ T012 (US1 filters + blurb serializer, same file but different functions — sequence preferred; mark P but avoid same-line edits)
- T017 ∥ T018, T019 ∥ T020 (US2 tests and handlers)
- T025 ∥ T026, T027 ∥ T028 ∥ T029 (US3 kudos/bookmarks/subs)
- T034 ∥ T035, T036 ∥ T037 (US4 threads + inbox)
- T041 ∥ T042, T043 ∥ T045 ∥ T046 (US5 pseud/series/tags)
- T050 ∥ T051, T052 ∥ T054 (US6 collections vs skins)
- Once Foundational completes, US3/US4/US5 can be staffed in parallel by different developers

---

## Parallel Example: User Story 1

```bash
# Launch US1 tests together:
Task: "Contract test for GET /api/works filter+sort+pagination in src/routes/works.rs"
Task: "Visual regression test for WorkBlurb AO3 header order in frontend/src/lib/components/WorkBlurb.test.ts"

# Launch US1 implementation together:
Task: "Implement work filter query parsing in src/routes/works.rs"
Task: "Polish WorkBlurb component to AO3 typography in frontend/src/lib/components/WorkBlurb.svelte"
```

---

## Implementation Strategy

### MVP First (User Stories 1–2 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T008) — CRITICAL
3. Complete Phase 3: US1 Browse/filter/blurb (T009–T016)
4. **STOP and VALIDATE**: US1 independent test per quickstart.md — filters + blurb on both skins, URL persists
5. Complete Phase 4: US2 Read/download (T017–T024)
6. **STOP and VALIDATE**: US1+US2 loop — browse→open→read→download without regression
7. Deploy/demo if ready (P1 MVP)

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Deploy/Demo (browse MVP)
3. Add US2 → Test independently → Deploy/Demo (read MVP)
4. Add US3 → Test → Deploy (engagement)
5. Add US4 → Test → Deploy (social)
6. Add US5 → Test → Deploy (authoring)
7. Add US6 → Test → Deploy (collections/skins)
8. Polish → docs + regression + screenshot archive

### Parallel Team Strategy

With multiple developers after Foundational:

- Developer A: US1 → US2 (P1 track)
- Developer B: US3 + US4 (P2 engagement/social)
- Developer C: US5 + US6 (P2 authoring + P3 collections)
- Stories complete and integrate independently; rebase on `main` per slice

---

## Notes

- [P] tasks = different files, no dependencies — safe to parallelize
- [Story] label maps task to specific user story for traceability (US1..US6)
- Each user story is independently completable and testable per quickstart.md slice
- File paths are exact; adjust only if `plan.md` structure changes
- Commit after each task or logical group; push per slice after manual screenshot QA
- Avoid: vague tasks, same-file parallel edits without sequencing, cross-story hard dependencies
