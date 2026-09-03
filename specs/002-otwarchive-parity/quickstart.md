# Quickstart: OTW Archive Parity

## Prerequisites

- OTW reference checkout at `/personal/documents/code/ruby/otwarchive` (or `~/code/rust/otwarchive` symlink).
- FicHub running locally (`cargo run` + `npm run dev`).

## Slice-by-slice manual QA (do one, verify, then next)

### Slice 1 — Browse / filter / blurb (P1 MVP)

1. Open OTW `app/views/works/index.html.erb` + `_filters.html.erb` side-by-side with FicHub `/works`.
2. Apply Rating, Warning, Category, Fandom, Complete, Language + sort by Kudos. Verify URL query persists, reload keeps filters.
3. Compare one blurb: title · byline pseud · fandom · 4-square symbols · stats line (words/chapters/kudos/bookmarks/hits/updated) — screenshot both skins vs AO3.
4. Click blurb → work page, verify header order.

**Pass**: Filters/sorts/blurb visually match AO3; no layout shift.

### Slice 2 — Read / chapter / download (P1)

1. Open OTW `works/show.html.erb` + `chapters/show.html.erb`; compare FicHub work/chapter order (header → meta → summary → notes → TOC → body).
2. Test chapter dropdown + prev/next + Entire Work.
3. Download EPUB/HTML/MOBI/PDF/TXT, open files, check filename parity.

**Pass**: Section order + nav + downloads match AO3.

### Slice 3 — Kudos / bookmarks / subscriptions / history (P2)

1. Kudos as logged-in, as guest, verify one-per-user and counts.
2. Bookmark with tags/private/rec, check dashboard blurb.
3. Subscribe to work/series/user, check subscriptions list + inbox/history.

### Slice 4 — Comments / inbox (P2)

1. Post comment + nested reply (logged-in + guest where allowed).
2. Inbox: reply inline, delete, mark read.
3. Compare thread indentation + byline placement to OTW `_comment.html.erb`.

### Slice 5 — Pseuds / co-creators / series / tags (P2)

1. Create pseud, invite co-creator, approve; byline shows both, dashboards list work.
2. Create series, order works, test prev/next.
3. Open tag page `/tags/{tag}`; verify canonical/synonyms and filtered listing.

### Slice 6 — Collections / skins (P3)

1. Create collection, add work, test moderated queue.
2. Create/apply user skin + work skin, preview sanitized CSS.

## Regression guard

After each slice, smoke-test FicHub extras: forum, requests, Ask, recs, progression — must remain accessible (additive placement, not displaced).

## References

- Spec: [spec.md](./spec.md)
- Data model: [data-model.md](./data-model.md)
- Contracts: [contracts/api.md](./contracts/api.md)
- OTW views: `otwarchive/app/views/works/*`, `chapters/*`, `bookmarks/*`, `comments/*`, `collections/*`, `skins/*`

