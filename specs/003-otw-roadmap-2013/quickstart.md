# Quickstart: OTW Roadmap 2013 Gaps

**Feature**: `003-otw-roadmap-2013` | **Spec**: [spec.md](./spec.md) | **Plan**: [plan.md](./plan.md) | **Contracts**: [contracts/api.md](./contracts/api.md)

## Prerequisites

- OTW reference checkout: `/personal/documents/code/ruby/otwarchive` + `/home/alvaro/code/rust/otwarchive` (for screenshot comparison)
- Rust 1.85, Node 22, `cargo`, `npm`
- Postgres 16 + Redis running; `DATABASE_URL` and `REDIS_URL` from `/opt/fichub/.env` (or local dev `.env`)
- Branch: `003-otw-roadmap-2013`

## Setup

```bash
# backend
CARGO_TARGET_DIR=/media/alvaro/code-worktrees/fichub-target cargo check
cargo test --lib   # expect 663 passed, 1 known flaky export whitelist

# frontend (avoid NFS EISDIR)
rm -rf /tmp/fichub-frontend-build
cp -a frontend /tmp/fichub-frontend-build
rm -rf /tmp/fichub-frontend-build/.svelte-kit /tmp/fichub-frontend-build/build
npm --prefix /tmp/fichub-frontend-build run build
npx --prefix /tmp/fichub-frontend-build svelte-check

# run locally
cargo run &                # backend :8000
npm --prefix /tmp/fichub-frontend-build run dev  # frontend
```

## Validation — slice-by-slice (each slice must pass before next)

### Slice 1 — Media & fiction type
1. As author, Post New Work → set Media=Image, Fiction=Nonfiction → save. Verify work header shows badge.
2. As anon, `/works?media=image` and `/works?fiction=nonfiction` filter correctly; URL persists after reload; compare sidebar to `app/views/works/_filters.html.erb`.
3. Embed an external video URL; verify it renders sandboxed.

### Slice 2 — Work relationships
1. Create Work B linked to Work A as `translation` (`POST /api/works/:b/related_works`); verify both work pages show reciprocal link.
2. Try linking to an anon/unrevealed parent → expect 422.
3. Search with relationship facet if exposed.

### Slice 3 — Subscriptions & History
1. Subscribe to a tag and a pseud; check `GET /api/subscriptions` and management page; mute one and verify mute persists.
2. Read 3 works; check `GET /api/users/me/readings` reverse-chronological; `DELETE /api/users/me/readings` clears.
3. Trigger a chapter update on a subscribed work; verify notification appears.

### Slice 4 — Anonymous / Drafts / Prologue-Epilogue
1. Create anonymous collection (`preferences.anonymous=true`); post work to it; as anon viewer verify byline is "Anonymous".
2. Start a draft, type, wait for autosave, reload → draft restored; Publish → work appears, draft gone.
3. Create 3-chapter work with chapter 1 `prologue` and chapter 3 `epilogue`; verify TOC shows "Prologue: …" / "Epilogue: …".

### Slice 5 — Private messaging
1. Alice `POST /api/conversations {recipient_id: bob, body}`; Bob sees unread badge; reply threads correctly.
2. Bob blocks Alice; Alice's next PM fails 422; no notification created.
3. Verify rate limit (burst 10) returns 429 with `Retry-After`.

### Slice 6 — I18n + footer
1. Switch locale to `de` and `zh`; walk `/`, `/works`, `/works/:id`, `/messages` — no raw keys, all chrome translated.
2. Compare footer to OTW footer sections (Site Map / Policies / Contact / Development) — screenshot parity.

### Slice 7 — Admin / wrangling / imports
1. Grant `wrangler` role; as wrangler `POST /api/tags/:id/canonicalize`; verify filtered listings merge.
2. Try wrangling as `abuse`-only user → 403.
3. `POST /api/import_batches` then `POST /api/import_batches/:id/works` twice with same URL+checksum → second no-ops (idempotent).

### Slice 8 — Public API + a11y
1. Create API key `POST /api/admin/api_keys` (admin) with `works:read` scope; `curl -H "Authorization: Bearer <key>" /api/v1/works?filter[media]=text&page[size]=5` paginates and returns OpenAPI-shaped JSON.
2. Run `npm run test` and `cargo test` — per-slice `page.test.ts` + route tests green.
3. Run axe on `/`, `/works`, `/works/:id`, `/messages` — zero critical violations.

## Regression check (every slice)

After each slice, re-run:

```bash
cargo test --lib
npm --prefix /tmp/fichub-frontend-build run build
# manual: forum board loads, Ask/Requests/Recs open, chapter translations still approve, Marginalia still shows, OPDS still paginates
```

If any FicHub extra fails, the slice is not shippable.

## Reference screenshots

Per slice, capture before/after of the same OTW view (`app/views/{works,chapters,collections,tags}`) side-by-side with FicHub on archive skin; attach to the PR. Checklist in `spec.md` SC-001..005 must be 100% for the slice before merge.
