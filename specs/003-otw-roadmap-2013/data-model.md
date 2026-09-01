# Data Model: OTW Roadmap 2013 Gaps

**Feature**: `003-otw-roadmap-2013` | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Research**: [research.md](./research.md)

Inherits all entities from `002-otwarchive-parity`. New/changed entities for this feature only.

## 1. Work — media & fiction type (US1)

- **Table**: `works` — add columns
  - `media_type` TEXT CHECK (`text`,`image`,`audio`,`video`,`embed`,`other`) NOT NULL DEFAULT `text`
  - `fiction_type` TEXT CHECK (`fiction`,`nonfiction`,`other`) NOT NULL DEFAULT `fiction`
- **Derived**: `nonfiction` boolean virtual (for index/search) = `fiction_type='nonfiction'` OR tag-based (kept for compat).
- **Indexes**: `CREATE INDEX ON works(media_type)`, `CREATE INDEX ON works(fiction_type)`, composite with fandom/language if search needs it.
- **Validation**: Posting form requires one of each; scraper imports default to `text`/`fiction` unless source indicates otherwise.
- **Relations**: none.

## 2. Work relationships (US2)

- **Table**: `work_relationships`
  - `id` BIGSERIAL PK
  - `work_id` BIGINT FK `works.id` NOT NULL  — the owned work (B that points to A)
  - `parent_type` TEXT CHECK (`work`,`external`) NOT NULL DEFAULT `work`
  - `parent_work_id` BIGINT FK `works.id` NULL  — when internal
  - `parent_url` TEXT NULL, `parent_title` TEXT NULL, `parent_author` TEXT NULL, `parent_language_id` INT NULL — when external
  - `kind` TEXT CHECK (`translation`,`remix`,`inspired_by`,`gift`,`prequel`,`sequel`) NOT NULL
  - `created_at` TIMESTAMPTZ, `notified_at` TIMESTAMPTZ NULL
- **Constraints**: `CHECK ((parent_type='work' AND parent_work_id IS NOT NULL) OR (parent_type='external' AND parent_url IS NOT NULL))`; `UNIQUE(work_id, parent_work_id, kind)` for internal; `UNIQUE(work_id, parent_url, kind)` for external.
- **Read**: reciprocal via `SELECT * FROM work_relationships WHERE parent_work_id = ?` unioned with owner row; serializer adds `related_to` / `related_from` arrays and hides anon/unrevealed parents.

## 3. Subscriptions & History — extended (US3)

- **Table**: `subscriptions` — extend polymorphic `subscribable_type`
  - Allowed types: `work` | `series` | `collection` | `user` | `pseud` | `tag` | `fandom`
  - Columns: `id`, `user_id` FK, `subscribable_type` TEXT, `subscribable_id` BIGINT, `muted` BOOL DEFAULT false, `created_at`
  - `UNIQUE(user_id, subscribable_type, subscribable_id)`
- **Table**: `reading_history` (already `057`) — polish
  - `id`, `user_id`, `work_id`, `chapter_id` NULL, `viewed_at`, `major_version` INT — existing shape; add `CLEAR` endpoint that soft-deletes by `user_id`.
- **Indexes**: `(user_id, viewed_at DESC)` for history; `(subscribable_type, subscribable_id)` for fan-out.

## 4. Anonymous posting — collection flag (US4a)

- **Table**: `collection_preferences` — add `anonymous` BOOL DEFAULT false (+ `anonymous_updated_at`)
- **Table**: `works` — `anonymous` BOOL GENERATED or maintained trigger from `collection_items.anonymous` (mirrors `Collectible` concern: work is anonymous if any anon collection item points at it).
- **Serializer rule**: when `work.anonymous` true, `pseud_ids`, `user_ids`, `byline` are redacted for anon viewers (author still sees via owner check); same for `series` (`app/models/series.rb:243`).

## 5. Drafts (US4b)

- **Table**: `drafts`
  - `id` BIGSERIAL PK, `user_id` FK NOT NULL, `payload` JSONB NOT NULL (preface + chapter bodies + media/fiction fields), `updated_at` TIMESTAMPTZ, `created_at` TIMESTAMPTZ
  - `UNIQUE(user_id, id)` implicitly; optionally `UNIQUE(user_id)` if single draft per user, but spec keeps many drafts so no.
- **Lifecycle**: `PUT /api/drafts/:id` autosave (debounced 1s) + localStorage fallback; `GET /api/drafts` list; `POST /api/drafts/:id/publish` creates `work` + `chapters` in transaction then deletes draft; `DELETE` discards.

## 6. Chapters — prologue/epilogue (US4c)

- **Table**: `chapters` — add `position_kind` TEXT CHECK (`prologue`,`chapter`,`epilogue`) NOT NULL DEFAULT `chapter`
- **Derived**: first chapter marked `prologue` renders as `Prologue: <title>`; last marked `epilogue` as `Epilogue: <title>`; middle stays `Chapter N`. Nav/TOC consult `position_kind` before numeric position.
- **Validation**: at most one `prologue` at position 1 and one `epilogue` at max position per work (soft check in service).

## 7. Private messaging (US5)

- **Tables**:
  - `conversations` (`id`, `created_at`, `updated_at`)
  - `conversation_participants` (`conversation_id` FK, `user_id` FK, `last_read_message_id` NULL, `archived` BOOL, `blocked` BOOL, PK (`conversation_id`,`user_id`))
  - `messages` (`id`, `conversation_id` FK, `sender_id` FK, `body` TEXT, `created_at`)
  - `blocks` (`user_id` FK, `blocked_user_id` FK, PK both)
- **Invariants**: DM requires exactly 2 participants initially; `blocks` prevents creation and delivery; rate limit via `limiter` + Redis; inbox badge = `messages.id > last_read_message_id`.

## 8. Admin roles, tags wrangling, Open Doors / imports (US7)

- **Table**: `admin_roles` (`user_id` FK, `role` TEXT CHECK (`abuse`,`wrangler`,`open_doors`,`support`,`translator`,`admin`), `granted_at`, PK (`user_id`,`role`))
- **Table**: `tags` — add `canonical_tag_id` BIGINT FK NULL, `is_canonical` BOOL DEFAULT true, `wrangler_notes` TEXT
  - Merge = set `canonical_tag_id` on synonyms; filtered listings resolve through canonical.
- **Tables** for imports:
  - `import_batches` (`id`, `source_name` TEXT, `collection_id` FK NULL, `created_by` FK, `status` (`pending`,`running`,`done`,`failed`), `created_at`)
  - `imported_works` (`id`, `batch_id` FK, `source_url` TEXT UNIQUE, `source_author` TEXT, `checksum` TEXT, `work_id` FK NULL, `status`)
  - Idempotency: re-import with same `source_url`+`checksum` no-ops; different checksum updates existing work.

## 9. Public API keys (US8)

- **Table**: `api_keys` (`id`, `user_id` FK, `key_hash` TEXT UNIQUE, `scopes` TEXT[] (`works:read`,`bookmarks:read`,`subscriptions:read`,`history:read`), `rate_limit_per_min` INT, `created_at`, `revoked_at` NULL)
- **Behavior**: Bearer token `Authorization: Bearer <key>`; hashed with `crypto.rs` helpers; Redis counter for rate limit; versioned under `/api/v1`.

## 10. I18n / skins / footer (US6)

- No new tables. `src/lib/i18n/dictionaries/*.ts` gains keys for all new chrome (media, relationships, subscriptions/history, anon/drafts/chapters, messages, admin, API). Footer/header verified against OTW map — static data, no migration.

## State Transitions

- Draft → Published: `drafts.payload` → `works` + `chapters` transaction; draft deleted.
- Anonymous collection toggle: flipping `collection_preferences.anonymous` enqueues re-derive of `works.anonymous` for member works (async after commit).
- Tag merge: `tags.is_canonical=false`, `canonical_tag_id` set; previous index entries invalidated.
- Import batch: `pending` → `running` → `done|failed`; each `imported_works` row tracks per-work status.
