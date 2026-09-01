# Tasks: Forum Depth Phase 2

**Input**: Design documents from `specs/001-forum-depth-phase2/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1..US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify existing infra, no new crates/tables

- [ ] T001 Verify `forum_read_state` + `forum_topics.status` schema in `migrations/041_forum_core.sql` and live DB
- [ ] T002 Confirm existing `POST /api/forum/topics/{id}/read` handler in `src/routes/forum.rs` and `frontend/src/lib/api/forum.ts`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: API helpers and shared types used by all stories

- [ ] T003 Add `unread`/`last_read_post_id` computation helper for topic list/detail in `src/routes/forum.rs`
- [ ] T004 Add `status` ordering helper `(status='pinned') DESC` for topic list in `src/routes/forum.rs`
- [ ] T005 Update `frontend/src/lib/api/forum.ts` types to include `status`, `unread`, `last_read_post_id` in topic interfaces
- [ ] T006 Create shared badge CSS/labels for archive parity in `frontend/src/lib/components/forum/TopicThread.svelte` (text labels, no emoji)

**Checkpoint**: Foundation ready — US1..US4 can start in parallel (if staffed) else P1→P2→P3 order

---

## Phase 3: User Story 1 - Read-state badges (Priority: P1) 🎯 MVP

**Goal**: Logged-in users see unread indicators on forum lists; opening topic marks read.

**Independent Test**: Create topic as A, as B verify `unread:true` on list, open T + `POST .../read`, verify `unread:false` on next list. Anonymous sees no badge. See quickstart.md §1.

### Tests for User Story 1

- [ ] T007 [P] [US1] Contract test for `GET /api/forum/topics` unread field in `frontend/src/lib/api/forum.test.ts`
- [ ] T008 [P] [US1] Contract test for `POST /api/forum/topics/{id}/read` idempotent upsert in `frontend/src/lib/api/forum.test.ts`
- [ ] T009 [US1] Page test for unread badge on list in `frontend/src/routes/forum/[categorySlug]/page.test.ts`

### Implementation for User Story 1

- [ ] T010 [US1] LEFT JOIN `forum_read_state` and return `unread`/`last_read_post_id` in `GET /api/forum/topics` in `src/routes/forum.rs`
- [ ] T011 [US1] Return `unread`/`last_read_post_id` in `GET /api/forum/topics/{id}` in `src/routes/forum.rs`
- [ ] T012 [US1] Implement idempotent `POST /api/forum/topics/{id}/read` upsert (if not already complete) in `src/routes/forum.rs`
- [ ] T013 [US1] Add `markTopicRead(topicId, lastReadPostId)` client in `frontend/src/lib/api/forum.ts`
- [ ] T014 [US1] Render unread badge in category list `frontend/src/routes/forum/[categorySlug]/+page.svelte`
- [ ] T015 [US1] Trigger mark-read on topic open in `frontend/src/routes/forum/board/[topicSlug].[topicId]/+page.svelte` and `frontend/src/lib/components/forum/TopicThread.svelte`
- [ ] T016 [US1] Verify archive vs modern skin parity for unread badge (no emoji, CSS square)

**Checkpoint**: US1 fully functional and testable independently

---

## Phase 4: User Story 2 - Pin / Lock affordances (Priority: P1)

**Goal**: Curators pin/lock topics; pinned sorts first; locked disables replies.

**Independent Test**: As curator pin → top sort; lock → non-curator 403; unlock → reply works. See quickstart.md §2.

### Tests for User Story 2

- [ ] T017 [P] [US2] Contract test for `POST /api/admin/forum/topics/{id}/pin` in `frontend/src/lib/api/forum.test.ts`
- [ ] T018 [P] [US2] Contract test for `POST /api/admin/forum/topics/{id}/lock` + locked post rejection in `frontend/src/lib/api/forum.test.ts`
- [ ] T019 [US2] Page test for pinned/locked badges and disabled reply in `frontend/src/routes/forum/board/[topicSlug].[topicId]/page.test.ts`

### Implementation for User Story 2

- [ ] T020 [US2] Implement `POST /api/admin/forum/topics/{id}/pin` handler with curator gate in `src/routes/forum.rs`
- [ ] T021 [US2] Implement `POST /api/admin/forum/topics/{id}/lock` handler with curator gate in `src/routes/forum.rs`
- [ ] T022 [US2] Enforce locked check in `POST /api/forum/topics/{id}/posts` in `src/routes/forum.rs` (403 topic_locked for non-curator)
- [ ] T023 [US2] Expose `status` ordering in list query `(status='pinned') DESC` in `src/routes/forum.rs`
- [ ] T024 [P] [US2] Add pin/lock client methods in `frontend/src/lib/api/forum.ts`
- [ ] T025 [P] [US2] Render pin/locked badges in list `frontend/src/routes/forum/[categorySlug]/+page.svelte` and detail `frontend/src/lib/components/forum/TopicThread.svelte`
- [ ] T026 [US2] Disable/hide reply form when `status='locked'` for non-curators in `frontend/src/lib/components/forum/TopicThread.svelte`
- [ ] T027 [US2] Add curator pin/lock toggle controls in `frontend/src/lib/components/forum/TopicThread.svelte` (curator only, level≥50)

**Checkpoint**: US1+US2 both work independently

---

## Phase 5: User Story 3 - Metamod queue polish (Priority: P2)

**Goal**: Filterable metamod queue, single verdict, unfair-rate surfacing.

**Independent Test**: 3 grants, vote one fair, verify filter hides reviewed, duplicate 409. See quickstart.md §3.

### Tests for User Story 3

- [ ] T028 [P] [US3] Contract test for `GET /api/forum/metamod/queue?filter=` pagination in `frontend/src/lib/api/forum.test.ts`
- [ ] T029 [P] [US3] Contract test for duplicate verdict 409 in `frontend/src/lib/api/forum.test.ts`

### Implementation for User Story 3

- [ ] T030 [US3] Add `filter`/`verdict`/`cursor` handling to `GET /api/forum/metamod/queue` in `src/routes/forum.rs`
- [ ] T031 [US3] Enforce single verdict per (grant_id, reviewer_id) with 409 in `POST /api/forum/metamod/grants/{id}/verdict` in `src/routes/forum.rs`
- [ ] T032 [US3] Ensure `GET /api/forum/metamod/grants/{id}` returns anonymized context in `src/routes/forum.rs`
- [ ] T033 [US3] Add metamod queue client with filters in `frontend/src/lib/api/forum.ts`
- [ ] T034 [US3] Build filterable queue UI (unreviewed/reviewed, fair/unfair) in `frontend/src/routes/forum/metamod/+page.svelte`
- [ ] T035 [US3] Surface rolling unfair-rate/cooldown in moderate drawer `frontend/src/routes/forum/moderate/+page.svelte`

**Checkpoint**: US1..US3 all independently functional

---

## Phase 6: User Story 4 - Mod-power user tools polish (Priority: P3)

**Goal**: Per-user mod context drawer from topic/post actions.

**Independent Test**: Open mod drawer for user with 2 grants, verify reasons listed.

- [ ] T036 [US4] Expose recent grants API for user `GET /api/forum/moderation/user/{id}/grants` if not exists in `src/routes/forum.rs`
- [ ] T037 [US4] Implement mod drawer UI with recent grants/reasons in `frontend/src/lib/components/forum/TopicThread.svelte` or `frontend/src/routes/forum/moderate/+page.svelte`
- [ ] T038 [US4] Add page test for drawer in `frontend/src/routes/forum/moderate/page.test.ts`

**Checkpoint**: All 4 stories independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Gates, docs, cleanup

- [ ] T039 Run `cargo check` + `cargo test --lib` and fix forum tests in repo root
- [ ] T040 [P] Run `npm run build` + `svelte-check` (0 errors) + `npm test` forum suite in `frontend/`
- [ ] T041 [P] Verify archive skin parity for all new badges (pin/lock/unread) — manual QA both skins
- [ ] T042 Verify no emoji in new UI text (grep `✒️|🔥|✨|🧑‍🏫|🔍|⬇` in frontend)
- [ ] T043 Run `quickstart.md` full validation (read-state, pin/lock, metamod) against local + ThinkCentre
- [ ] T044 Update `docs/brainstorm-02-roadmap-status.md` P4 status and `specs/001-forum-depth-phase2/` links
- [ ] T045 Deploy: `CARGO_TARGET_DIR=/media/... cargo build --release` → scp → direct DB psql if needed → `/tmp` frontend build → `rsync --delete` to `FRONTEND_DIR` → `systemctl restart fichub` → curl sweep → push

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational
  - US1 and US2 are both P1 — can run in parallel (different files) or sequentially P1→P1
  - US3 (P2) after P1 stories
  - US4 (P3) after US3
- **Polish (Phase 7)**: Depends on desired stories complete

### User Story Dependencies

- **US1 (P1) Read-state**: No deps on other stories
- **US2 (P1) Pin/Lock**: No deps on US1 (independent), but shares helpers from Phase 2
- **US3 (P2) Metamod**: Independent of US1/US2
- **US4 (P3) Mod drawer**: May reuse US3 queue data but independently testable

### Within Each User Story

- Contract tests before implementation (write FAIL, then implement)
- Handlers before clients before UI
- Core logic before integration

### Parallel Opportunities

- T007∥T008∥T009 (all US1 tests, different expectations)
- T010→T011→T012 sequential (same file `forum.rs`), then T013∥T014∥T015 (different frontend files)
- T017∥T018∥T019 (US2 tests)
- T020 sequential in `forum.rs`, then T024∥T025∥T026 parallel (different files)
- Once Phase 2 done, US1 and US2 can be staffed in parallel by two devs

---

## Parallel Example: User Story 1

```bash
# Tests in parallel:
Task T007: forum.test.ts unread field
Task T008: forum.test.ts read upsert
# Handlers sequential (same file):
Task T010: GET /api/forum/topics unread
Task T011: GET /api/forum/topics/{id} unread
# Then UI in parallel:
Task T014: [categorySlug]/+page.svelte badge
Task T015: board/[topicSlug].[topicId]/+page.svelte mark-read
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T006)
3. Complete Phase 3: US1 Read-state (T007-T016)
4. **STOP and VALIDATE**: quickstart.md §1 — list badge + mark-read
5. Deploy/demo MVP

### Incremental Delivery

1. Setup+Foundational → foundation ready
2. Add US1 → Test → Deploy (MVP)
3. Add US2 → Test → Deploy
4. Add US3 → Test → Deploy
5. Add US4 → Test → Deploy
6. Polish → final sweep

---

## Notes

- Tasks map 1:1 to FR-001..013 and contracts; no new tables/migrations expected (reuse 041/043/044)
- If migration 070 needed for index, add as T003b and note in plan.md
- Keep archive parity and no-emoji gate on every UI task
- Commit after each task group; use `page.test.ts` not `+page.test.ts`

