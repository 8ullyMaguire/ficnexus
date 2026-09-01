# Research: OTW Roadmap 2013 Gaps

**Feature**: `003-otw-roadmap-2013` | **Date**: 2026-08-24 | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md)

All decisions verified against `/personal/documents/code/ruby/otwarchive` reference (models/views/controllers) and current FicHub layout (`src/`, `frontend/`, `migrations/`).

## 1. Media Type & Fiction Type (US1, FR-001)

- **Decision**: Add two orthogonal columns on works — `media_type` (enum: `text|image|audio|video|embed|other`) and `fiction_type` (enum: `fiction|nonfiction|other`) — displayed in work header/meta and exposed as filter facets on `/works` and Work Search. Keep Additional Tags for everything else; do not overload tags with media.

- **Rationale**: OTW `app/views/works/_filters.html.erb` and `WorkIndexer` already index `nonfiction` as a derived filter; `app/models/work.rb#nonfiction` checks tag ids. But the roadmap explicitly says "instead of relying on Additional Tags alone" — so UI must offer a true posting field plus a filter facet that maps to a column/search token, not just tag intersection. This matches AO3's later "Media" facet behavior.

- **Alternatives considered**: Tag-only approach (cheaper) rejected — breaks roadmap promise and makes "only Fanart" queries unreliable. Separate `work_media_types` join table rejected — single enum covers 0.9 scope; multi-media works can use `other` + tags until 1.0 if needed.

- **Storage / indexing**: `work.media_type`, `work.fiction_type` with CHECK constraints; GIN or B-tree index; pgvector/scrape search in `src/search/` extended to include media filter. Not a tag mutation — preserves `scraped_tags` flow.

## 2. Work Relationships (US2, FR-002)

- **Decision**: Mirror OTW `RelatedWork` as `work_relationships` join: `id`, `work_id` (owner), `parent_type` (`work|external_work`), `parent_work_id`, `parent_url/title/author` for external, `kind` (`translation|remix|inspired_by|gift|prequel|sequel`), `approved_at`. Reciprocal read via reverse query (`WHERE parent_work_id = ?`) — no double-write.

- **Rationale**: `app/models/related_work.rb` is polymorphic (`parent: Work|ExternalWork`), validated for visibility/anon/orphan, and notifies parent owners. FicHub can start with internal-work parents only and add `ExternalWork` later for imports. Reciprocal display is a read pattern, matching OTW's work show sections.

- **Alternatives considered**: Double-row (forward+reverse) rejected — risks drift and needs transaction. ExternalWork kept minimal (url/title/author/language_id) to unblock Open Doors later.

## 3. Subscriptions & History (US3, FR-003/004)

- **Decision**: Extend existing subscriptions (currently work/series/author) to also target `tag`, `fandom`, `collection`. Store as `subscriptions(user_id, subscribable_type, subscribable_id)` with Redis pub for notifications/feeds emulation. History uses existing `reading_history` (`migrations/057_reading_history.sql`) polished to reverse-chronological listing with pagination and clear.

- **Rationale**: `app/models/subscription.rb` already subscribes to any polymorphic target; OTW sub types include user/pseud/tag/collection/series. Reusing the polymorphic shape avoids a second subscriptions table. History improvements are UI + index polish, not new storage.

- **Alternatives considered**: Per-type tables rejected — OTW polymorphic is simpler. Feed generation via background job deferred — notifications table + `/notifications` covers 0.9 scope.

## 4. Anonymous Posting, Drafts, Chapter Prologues/Epilogues (US4, FR-005/006/007)

- **Decision**:
  - Anon: `collection_preferences.anonymous` (boolean) + `works.anonymous` derived (via `Collectible` concern logic from `app/models/concerns/collectible.rb`). Anonymous works mask byline/pseud_ids in serializers when `anonymous?` or `unrevealed?`; API omits pseuds accordingly (`work.rb:1235`).
  - Drafts: new `drafts` table (id, user_id, payload JSONB, updated_at) with debounced PUT + localStorage fallback; preview via client render, discard deletes row — no work created until publish.
  - Chapters: add `chapters.position_kind` enum (`prologue|chapter|epilogue`) plus existing position; TOC/nav render "Prologue: Title" / "Epilogue: Title" as in `app/views/chapters/`.

- **Rationale**: All three are verbatim from `work.rb`, `chapter.rb`, `collectible.rb`, and `collection_preference.rb`. Drafts previously existed as `drafts.html.erb`; matching OTW's `post_work` flow keeps migration simple.

- **Alternatives considered**: Reusing `work_proposals` for drafts rejected — different lifecycle and permissions. Adding anon as a per-work toggle rejected — roadmap scopes it via anonymous collections/challenges.

## 5. Private Messaging (US5, FR-008)

- **Decision**: New `conversations` + `conversation_participants` + `messages` tables, inbox at `/messages`, block list via `blocks(user_id, blocked_user_id)`. Rate limit via `limiter` (existing crate) + Redis sliding window; abuse report hooks into existing moderation.

- **Rationale**: `app/models` has `Block`, `Comment.by_anonymous_creator?`, and DM-adjacent patterns but OTW DMs are a later addon — FicHub can follow modern DM design (threaded conversation, not single-table pm). Matches roadmap's "most-requested" framing while staying abuse-resistant.

- **Alternatives considered**: Reusing comments/inbox for DMs rejected — different visibility and block semantics.

## 6. Translatable UI + Skins/Footer Layout (US6, FR-009)

- **Decision**: All chrome strings go through `src/lib/i18n/dictionaries/*.ts` (6 locales already), fallback to `en`, never render raw keys. Footer mirrors OTW footer: Site Map / Diversity / TOS / Content Policy / Privacy / DMCA & TIDA / Site Status + Contact (Policy Questions & Abuse Reports / Technical Support & Feedback) + Development (version / Known Issues / GPL). Header/footer already AO3-parity from `003210d`/`78d2ab1` — this slice audits and fills gaps.

- **Rationale**: Roadmap calls translatable interface "one of our main goals"; FicHub already has `i18n` wiring — completing coverage is a string-audit task, not new infra. Skins already exist (site/work); documenting/selecting them suffices for 0.9.

- **Alternatives considered**: Introducing `rails-i18n`-style YAML rejected — keep TS dictionaries to avoid churn.

## 7. Admin Roles, Wrangling, Open Doors, Collections/Challenges Polish (US7, FR-010/011)

- **Decision**:
  - Admin roles: `admin_roles(user_id, role)` enum (`abuse|wrangler|open_doors|support|translator|admin`) + server-side `require_role!` guard on routes; frontend gates are cosmetic only.
  - Wrangling: `tags` gains `canonical_tag_id`, `is_canonical`, `wrangler_notes` — merge via canonical pointer (OTW `Tag` model).
  - Open Doors: `import_batches` + `imported_works` with source URL/author attribution + import collection link; re-import is idempotent by source URL + checksum.
  - Collections/challenges: bug-fix pass on existing `collections` (061) and challenge flow (use OTW `ChallengeSignup/Assignment/Claim` as reference for assignment/claim steps).

- **Rationale**: `app/models/admin.rb`, `tag.rb`, and `collection.rb` all expose these roles/fields; `story_parser.rb` plus `new_import.html.erb` guide the import flow. Idempotency is critical for at-risk archives.

- **Alternatives considered**: Single `is_admin` flag rejected — roadmap explicitly calls for "clearly defined roles" and modular admin.

## 8. Public API + Accessibility/Test Package (US8, FR-012/014)

- **Decision**: Versioned public API under `/api/v1` (works index/show, bookmarks read, subscriptions/history read) with `api_keys` table (hashed key, scopes, rate limits via `limiter`). Document via OpenAPI YAML under `contracts/openapi.yaml`. Accessibility: axe-core checks on work index/show/chapter/post-form; CI enforces per-slice `page.test.ts` + `cargo test`.

- **Rationale**: Roadmap §1.0 lists "Public API... API hooks for posting/bookmarking/reading" and "a wide range of automated tests" as 1.0 done-criteria. FicHub already has `/api/*` internal routes — versioning and scoping them is the delta.

- **Alternatives considered**: Reusing internal `/api/*` as public without versioning rejected — breaking-change risk.

## 9. Cross-cutting

- **NFS / deploy**: All new migrations stay small and sequential; use `/tmp/fichub-frontend-build` for frontend builds and `rsync --delete` to `/personal/.../frontend/build` → `FRONTEND_DIR`; verify with `curl /sw.js`.
- **No regressions**: Every slice includes a "FicHub extras still pass" check (forum/Ask/requests/recs/progression/Marginalia/OPDS/PWA) before merge — same gate as 002.
- **Slicing**: Order is the Phases list in plan.md — media (1) → relationships (2) → subs/history (3) → anon/drafts/chapters (4) → DMs (5) → i18n/footer (6) → admin/wrangling/imports (7) → API/a11y (8). Each slice is independently reviewable with screenshots.
