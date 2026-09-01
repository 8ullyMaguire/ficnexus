# Contracts: OTW Roadmap 2013 Gaps

Source: [spec.md](../spec.md) · [plan.md](../plan.md) · [data-model.md](../data-model.md)

Base URL: `http://localhost:8000` (prod `https://fichub.polarisocial.xyz`) — internal `/api/*` is cookie-auth; public `/api/v1/*` is Bearer-key auth where noted.

Common errors: `{ "error": "unauthorized" | "forbidden" | "not_found" | "unprocessable", "message": "..." }`

---

## 1. Work media & fiction type (Slice 1)

### `POST /api/works` / `PATCH /api/works/:id` — new fields
- **Body add**: `media_type: "text"|"image"|"audio"|"video"|"embed"|"other"` (default `text`), `fiction_type: "fiction"|"nonfiction"|"other"` (default `fiction`)
- **Response**: work JSON gains `media_type`, `fiction_type`, `nonfiction` (bool derived).
- **Auth**: logged-in; author or admin.

### `GET /api/works?media=text&fiction=nonfiction&...` / `GET /search?q=...&media=art`
- **Query add**: `media`, `fiction` (repeatable or single). Unknown values → 422.
- **Effect**: filters `works.media_type` / `works.fiction_type`; reflected in `?media=` URL so back/refresh persists.
- **Frontend**: posting fieldset `/_posting_fieldset.html.erb` parity; filters in `/_filters.html.erb` parity.

---

## 2. Work relationships (Slice 2)

### `POST /api/works/:id/related_works`
```json
{ "parent_type":"work", "parent_work_id": 123, "kind":"translation" }
-- or --
{ "parent_type":"external", "parent_url":"https://example.com/fic", "parent_title":"Title", "parent_author":"A", "kind":"inspired_by" }
```
- `kind` ∈ `translation|remix|inspired_by|gift|prequel|sequel`
- **Errors**: 404 if parent work not visible; 422 if anon/unrevealed/orphan-protected parent.
- **Side effect**: `UserMailer.related_work_notification` equivalent (notification to parent owners).

### `GET /api/works/:id/related_works`
- **Response**: `{ "related_to": [ … ], "related_from": [ … ] }` — reciprocal (owner + reverse query).

### `DELETE /api/works/:id/related_works/:relId`
- Owner or admin only.

### Mirror: `GET /api/works/:id` now includes `related_to` / `related_from` arrays and `gift_recipient` where applicable.

---

## 3. Subscriptions & History (Slice 3)

### `POST /api/subscriptions` — extended types
```json
{ "subscribable_type": "tag", "subscribable_id": 42 }
-- types: work|series|collection|user|pseud|tag|fandom
```
### `GET /api/subscriptions` — list own, with `muted`
### `DELETE /api/subscriptions/:id` + `PATCH /api/subscriptions/:id` `{ "muted": true }`

### `GET /api/users/:id/readings`
- Query: `?page&per_page` (default 20). Returns `{ readings: [{ work, viewed_at }], total, has_more }` reverse-chronological.
- `DELETE /api/users/:id/readings` — clear history (owner only).

---

## 4. Anonymous / Drafts / Chapters (Slice 4)

### `POST /api/collections/:id/preferences` — `anonymous`
- Body: `{ "anonymous": true }` — requires collection owner / admin role `open_doors` or `admin`.
- Effect: member works derive `works.anonymous` (async).

### Drafts
- `GET /api/drafts` — list own
- `POST /api/drafts` `{ "payload": { …preface/chapters/media… } }` → `{ draft }`
- `PUT /api/drafts/:id` `{ "payload": { … } }` — autosave (debounced)
- `POST /api/drafts/:id/publish` → `{ work }` (transaction; draft deleted)
- `DELETE /api/drafts/:id`

### Chapters — prologue/epilogue
- `POST /api/works/:id/chapters` / `PATCH /api/works/:id/chapters/:cid`
  - Body add: `position_kind: "prologue"|"chapter"|"epilogue"` (default `chapter`)
  - Validation: at most one `prologue` at position 1 and one `epilogue` at last per work.

---

## 5. Private messaging (Slice 5)

### `POST /api/conversations`
```json
{ "recipient_id": 99, "body": "hello" }
```
- Creates conversation (2 participants) + first message. 422 if recipient blocked sender or rate-limited.

### `GET /api/conversations` — list own (with `unread_count`)
### `GET /api/conversations/:id/messages` — `?before&limit` pagination
### `POST /api/conversations/:id/messages` `{ "body": "..." }`
### `POST /api/blocks` `{ "blocked_user_id": 99 }` / `DELETE /api/blocks/:id`

Inbox badge: `messages.id > conversation_participants.last_read_message_id`.

---

## 6. I18n / footer (Slice 6)

No API — string audit. Contract is: every user-visible chrome string is a key in `src/lib/i18n/dictionaries/{en,de,es,fr,pt-BR,zh}.ts` with fallback to `en`, no raw keys rendered. Footer mirrors OTW map (About / Site Map / Policies / Contact / Development) at `src/lib/ui/archive/ArchiveFooter.svelte` (or equivalent).

---

## 7. Admin roles / wrangling / imports (Slice 7)

### `GET /api/admin/roles` / `POST /api/admin/roles` `{ "user_id": 1, "role":"wrangler" }`
- Roles: `abuse|wrangler|open_doors|support|translator|admin`
- Guard: `require_role!(abuse|admin)` etc. per route — frontend gate not authoritative.

### `POST /api/tags/:id/canonicalize` `{ "canonical_tag_id": 10 }`
- Sets `canonical_tag_id`, `is_canonical=false`; filtered listings resolve through canonical.

### Imports (Open Doors)
- `POST /api/import_batches` `{ "source_name":"Smallville Slash Archive", "collection_id": 5 }` → `{ batch }`
- `POST /api/import_batches/:id/works` `{ "source_url":"https://...", "source_author":"A", "checksum":"...", "work_payload":{…} }`
- Idempotent by `source_url`+`checksum`.

---

## 8. Public API v1 (Slice 8)

All under `/api/v1` — Bearer `Authorization: Bearer <key>` via `api_keys`.

- `GET /api/v1/works?filter[media]=text&filter[fiction]=nonfiction&page[number]=1&page[size]=20`
- `GET /api/v1/works/:id`
- `GET /api/v1/bookmarks` (scoped `bookmarks:read`)
- `GET /api/v1/subscriptions` / `GET /api/v1/readings` (scoped per key)

**Pagination**: JSON:API `page[number]`/`page[size]` + `links: { self, next, prev }` and `meta: { total }`.
**Errors**: JSON:API error objects. Rate limit: `429` with `Retry-After`.
**Docs**: `contracts/openapi.yaml` generated from these routes.
