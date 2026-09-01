# Contracts: OTW Archive Parity

## Work filters & blurb (P1)

- `GET /api/works?rating=&warnings=&category=&fandom=&relationship=&character=&freeform=&complete=&word_count=&language=&sort=&page=` — mirrors OTW `WorkSearch` facets; query persists in URL.
- Response: `works[]` with `blurb` (title, byline pseuds, fandoms, warnings, rating, categories, stats, symbols) matching OTW `_meta`/`_blurb` order.
- Frontend: `WorkBlurb.svelte` + filter drawer — archive parity typography.

## Chapter / download (P1)

- `GET /api/works/{id}` + `GET /api/works/{id}/chapters` / `/read/{id}` — header → meta → summary → notes → TOC → body order like OTW `works/show.html.erb`.
- `GET /api/works/{id}/download?format=epub|html|mobi|pdf|txt` — filename like OTW `downloads_controller`.

## Kudos / bookmarks / subscriptions (P2)

- `POST /api/works/{id}/kudos` — one per user, guest counted separately; `DELETE` not allowed (OTW parity).
- `POST /api/works/{id}/bookmarks` `{tags, notes, private, rec}` — wrangled tag links.
- `POST /api/subscriptions` `{type: work|series|user, id}` — inbox/history hooks.

## Comments / inbox (P2)

- `POST /api/works/{id}/comments` + `POST /api/comments/{id}/replies` — threaded; guest fields where allowed.
- `GET /api/inbox` + `POST /api/inbox/{id}/reply|delete|read` — mirrors `inbox_controller`.

## Pseuds / series / tags (P2)

- `GET/POST /api/pseuds`, `POST /api/works/{id}/creatorships` invite/approve.
- `GET /api/series/{id}` ordered works, prev/next nav.
- `GET /tags/{name}` + `GET /api/tags/{type}` wrangled.

## Collections / skins (P3)

- `POST /api/collections/{id}/items` add work, moderated queue.
- `GET/POST /api/skins` + `POST /api/works/{id}/skin` — sanitized CSS.
