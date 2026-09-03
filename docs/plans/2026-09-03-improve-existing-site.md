# Plan: Improve the Existing FicHub Site

Incremental improvements to the current codebase (no rewrite), ordered by impact. Each item is a separate commit.

## Items

1. **Meta tags on public pages** — per-fic `<svelte:head>` (title, description, canonical, OG, Twitter) on `works/[urlId]`; titles on other key public pages.
2. **Sitemap** — `/sitemaps/works-{shard}.xml` + index from the Rust backend (nightly or on-demand), plus `robots.txt` in `frontend/static/`.
3. **Cookie consent toast** — required by item 4. Approve/reject toast on first visit; state stored in localStorage; links to `/privacy`.
4. **Anonymous library (device cookie)** — bookmarks/follows work before signup via signed `fh_dev` cookie; merged into the account on register/login. Only set when consent given.
5. **Homepage focus** — paste-URL box + trending above the dashboard widgets.

## Status

- [ ] 1. Meta tags
- [ ] 2. Sitemap + robots.txt
- [ ] 3. Cookie consent toast
- [ ] 4. Anonymous library device cookie
- [ ] 5. Homepage focus
