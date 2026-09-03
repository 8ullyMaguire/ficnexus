# Plan: Improve the Existing FicHub Site

Incremental improvements to the current codebase (no rewrite), ordered by impact. Each item is a separate commit.

## Items

1. **Meta tags on public pages** — per-fic `<svelte:head>` (title, description, canonical, OG, Twitter) on `works/[urlId]`; titles on other key public pages.
2. **Sitemap** — `/sitemaps/works-{shard}.xml` + index from the Rust backend (nightly or on-demand), plus `robots.txt` in `frontend/static/`.
3. **Cookie consent toast** — required by item 4. Approve/reject toast on first visit; state stored in localStorage; links to `/privacy`.
4. **Anonymous library (device cookie)** — bookmarks/follows work before signup via signed `fh_dev` cookie; merged into the account on register/login. Only set when consent given.
5. **Homepage focus** — paste-URL box + trending above the dashboard widgets.

## Status

- [x] 1. Meta tags — commit 90020f1 (`works/[urlId]` full OG/Twitter/canonical set; homepage, search, fandoms, trending, tags, authors titles)
- [x] 2. Sitemap + robots.txt — commit 9a292a9 (`/sitemaps/index.xml`, `/sitemaps/works-{shard}.xml` × 36, `PUBLIC_BASE_URL` config, `robots.txt` in `frontend/static/`)
- [x] 3. Cookie consent toast — commit 9fa5086 (`consent.svelte.ts` store + `ConsentToast.svelte` in ArchiveLayout; accept → 1-year `fh_consent` cookie, reject → sessionStorage only, links `/privacy`)
- [ ] 4. Anonymous library device cookie — gate `fh_dev` on `consentGranted()`; merge into account on register/login
- [ ] 5. Homepage focus — paste-URL box + trending above dashboard widgets

## Notes

- Repo had zero git history; baseline commit 56e3bee created first (also fixed `.gitignore`: `frontend/.svelte-kit/`, `frontend/build/`, `tmp/`, `books/`, `qa/`, `cache/`).
- Backend is the Axum server (`src/server.rs` `build_router`) — sitemaps are served there, not the SvelteKit SPA (SPA can't emit per-fic URLs for crawlers).
- Stray root-level `+page.svelte` / `ProposalDiff.svelte` files (copies of curator consensus page + diff component) were swept into the baseline; candidate for cleanup.
- `reader-lib.test.ts` (18 tests) fails with `localStorage.clear()` undefined under this sandbox's jsdom — pre-existing, unrelated to these changes.
