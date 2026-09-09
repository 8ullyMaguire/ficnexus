# PWA Plan — Make FicNexus a Full Progressive Web App (Browse-Every-Link Verified)

## Goal
Turn `ficnexus` (Rust/Axum + SvelteKit SPA + Postgres, adapter-static fallback `index.html`) into an installable, offline-capable PWA where every link the app renders is usable — online fresh, offline from cache or with a graceful stale/offline page. No stale-shell deploys, no cached private data, no broken navigation.

## Current state (audited 2026-08-31)

- **Manifest**: `frontend/static/manifest.webmanifest` is valid — `name/short_name`, `start_url "/"`, `scope "/"`, `display "standalone"`, `background_color/theme_color`, 192+512 (+ maskable) icons. Since `04719fe` it also has a `shortcuts` entry for `Forum → /forum`.
- **SW**: `frontend/static/sw.js` (fichub-v6) is hand-rolled (no Workbox). Strategies live in both `sw.js:decideStrategy` and `src/lib/pwa/strategies.ts` + `strategies.test.ts`. Correct: `/` is **network-first** (prevents stale-shell after deploy), `/_app/` + icons + manifest + `sw.js` are `precache`, `/read/` + `/works/` are `cache-first` (offline reading), public fic-content APIs (`/api/reader/`, `/api/epub`, `/api/meta`, `/api/search/similar/`) are `network-first`/`stale-while-revalidate`; everything else (`/api/auth`, bookmarks, admin, votes, comments, etc.) returns `none` and is never cached. Install discovers hashed `/_app/` assets by fetching `/` and parsing them. `src/routes/+layout.ts` calls `registerServiceWorker()` (guards: browser only, secure context, production only). `src/app.html` links the manifest and `theme-color`. `OfflineIndicator.svelte` shows a bottom banner when `navigator.onLine === false`.
- **Build**: `svelte.config.js` is `adapter-static` with `fallback: 'index.html'` (SPA shell served by Rust for every unknown route). `vite.config.ts` proxies `/api` to `localhost:8004` in dev. `build/` is rsynced to ThinkCentre `/var/www/ficnexus` (Rust `FRONTEND_DIR`).
- **Routes** (from `find src/routes -name +page.svelte`): ~70 pages including `/`, `/search` (+ `/search/body`, `/search/tags`, `/search/syntax`), `/read/[urlId]`, `/works/[urlId]`, `/work/[workId]` (+ `stats`, `translate`), `/forum` (+ board/category/search/new/moderate/metamod/invites/apply/blocks), `/roadmap` (+ consensus/board/changelog), `/trending`, `/fandoms`, `/leaderboard`, `/bookmarks`, `/lists`, `/shelves`, `/follows`, `/updates`, `/feed`, `/notifications`, `/messages`, `/users/[id]`, `/authors`, `/series`, `/collections`, `/tags`, `/stats`, `/history`, `/reading`, `/dashboard`, `/download`, `/upload`, `/docs`, `/curator/*`, `/admin/*`, `/settings/*`, `/badges`, `/requests`, `/work-proposals`, `/recommendations`, `/marketplace`, `/blind-date`, etc. `+layout.svelte:routePages` drives the SPA shell. Forum has a mobile bottom nav (`ForumBottomNav.svelte`, <768px on `/forum*`, polling `/api/forum/notifications` for the inbox badge) since `04719fe`.
- **Gaps this plan closes**: (1) Not every link has an offline/retry affordance — search/browse/leaderboard hit private or uncached APIs and show spinners offline. (2) No update prompt when a new SW version is waiting (activate is `skipWaiting` + `clients.claim` — good — but the UI never tells the user). (3) No `apple-touch-icon`, no `screenshots`/`categories` in manifest, no `shortcuts` beyond Forum. (4) No offline fallback page for uncached navigations (`/search?q=…`, `/bookmarks`, etc.) — they just fail. (5) `sw.js` `VERSION` is a manual string (fichub-v6) — a forgotten bump ships stale caches. (6) No navigation preload / no background sync for queued writes (forum drafts). (7) `forum` caching is currently `none` except via fiction-content paths — the NodeBB offline requirement (categories + last-read topics + pending drafts) is not yet cached. (8) Browser verification was blocked by the `chromewebdata` proxy error during this pass — needs a real dev-server run (`npm run dev -- --host 0.0.0.0` on `gamingpc` or ThinkCentre tunnel) to click every link.

## Browse-every-link inventory (what we checked and how to finish)

This plan requires actually opening the app and clicking every visible link. Do it from a dev server:

```bash
# on gamingpc (the SvelteKit dev host)
cd ~/code/rust/ficnexus/frontend
npm run dev -- --host 0.0.0.0 --port 5173
# if backend is on ThinkCentre, tunnel it:
ssh -L 8004:localhost:8000 thinkcentre
# then in the browser tool
#   new_tab('http://localhost:5173/')
#   click through: header nav (Discover/Community/Account dropdowns),
#   Forum → category → topic thread → reply composer,
#   Search → tag pages → fandom/series/author pages,
#   Read/Works, Bookmarks/Lists/Shelves, Notifications, Settings,
#   Admin/Curator, Roadmap (board/consensus/changelog), Trending/Leaderboard
# For each page, assert: (a) no console errors, (b) correct back button,
#   (c) bottom nav visible on <768px in /forum*, (d) install prompt appears.
```

Inventory checklist (tick each by actually visiting it online, then offline with DevTools → Application → Service Workers → Offline):

- [ ] Landing + header (Discover/Community/Account dropdowns, command palette, locale switch, archive/modern toggle)
- [ ] `/search` + `/search/body` + `/search/tags` + `/search/syntax` + saved-search + tag chip flows
- [ ] `/read/[urlId]` + `/works/[urlId]` + `/work/[workId]` (+ stats/translate) — offline reading
- [ ] `/forum` (grid) → `/forum/[categorySlug]` (topic list, sort/prev/next) → `/forum/board/[topicSlug].[topicId]` (thread, quote, mod drawer, reactions, composer) + `/forum/search` + `/forum/new`
- [ ] `/trending`, `/leaderboard`, `/fandoms`, `/fandom/[slug]`, `/tags`, `/people`
- [ ] `/bookmarks`, `/lists`, `/shelves`, `/follows`, `/updates`, `/feed`, `/notifications`, `/messages`
- [ ] `/roadmap` (kanban) + `/roadmap/consensus` (arena) + `/roadmap/changelog`
- [ ] `/settings` (+ account/theme/recipes/trust) + `/admin/*` + `/curator/*` + `/modlog` + `/docs`
- [ ] 404/empty/error states for each of the above

## Architecture

Keep the hand-rolled `sw.js` (don't add Workbox — current `sw.js` is small, testable, and the strategies pure function is already unit-tested). Changes are additive.

### Manifest (additive, no breaking `start_url`/`scope`)
Update `frontend/static/manifest.webmanifest`:
- `theme_color`/`background_color` already correct for dark AO3 shell — keep.
- Add `screenshots` (1280×720 desktop + 390×844 mobile of `/` and `/forum`) for richer install UI.
- Add `categories: ["books","social","entertainment"]`.
- Add `shortcuts` for Search, Bookmarks, Roadmap, Forum (Forum already exists — add the other three).
- Add `icons` `purpose: "any"` is missing — set the 192/512 as `"any maskable"` or split into two entries (`any` + `maskable`) for Play Store TWA compat.
- Ensure `icons` include the actual `favicon.png` size if Lighthouse asks.

`src/app.html`: add `<link rel="apple-touch-icon" href="/icon-192.png">` and `<meta name="apple-mobile-web-app-capable" content="yes">` for iOS, keep the existing `manifest` + `theme-color`.

### Service worker (keep current strategies, add three things)
`frontend/static/sw.js` + mirrored `src/lib/pwa/strategies.ts` (and `strategies.test.ts`):
1. **Forum offline shell**: add `FIC_API_PREFIXES` or a separate `FORUM_API_PREFIXES = ['/api/forum/categories','/api/forum/topics']`? No — forum topic lists include per-user read state; don't cache them. Instead cache **navigation shells**: for any `GET` same-origin navigation (request `mode === 'navigate'` or `Accept` includes `text/html`), return `network-first` and cache the response in `PRECACHE` for offline fallback. This already happens for `/` — extend it to all `page` navigations so `/forum`, `/search`, etc. are usable offline as shells (data may be stale; the UI will retry fetches). Add an `offlineFallback(request)` that serves a small `offline.html` (see below) when both network and cache miss for a navigation.
2. **Version stamping**: replace `const VERSION = 'fichub-v6'` with a build-time stamp. Simplest: write `frontend/static/sw.js` at build via a `vite` plugin or a `scripts/stamp-sw-version.js` run in `npm run build` (`prebuild` hook) that rewrites `VERSION` to `fichub-${git rev-parse --short HEAD}-${Date.now()}` or to the `build/_app/immutable` hash fingerprint. Keep `clearOldCaches()` as-is — the `VERSION` prefix does the invalidation.
3. **Lifecycle prompts**: add an `updatefound` + `controllerchange` listener in `src/lib/pwa/register.ts` that dispatches a Svelte store (`updateAvailable`) and show a non-blocking toast (reuse `OfflineIndicator` styling) with a `Reload` button. `sw.js` already calls `skipWaiting()` + `clients.claim()` — the prompt is the missing UI. Also enable `navigationPreload` in `activate` when available for faster navigations.

Add `frontend/static/offline.html` (tiny, no SvelteKit, lists cached `/read/` + `/works/` entries by reading the `RUNTIME_FIC` cache via `caches.keys()` — optional).

### Registration + stores
- `src/lib/pwa/register.ts`: keep guards, add `registration.waiting` → store signal, and `navigator.serviceWorker.addEventListener('controllerchange')` → reload once.
- New `src/lib/pwa/install.svelte.ts` store: listen for `beforeinstallprompt`, stash the event, expose `canInstall` + `promptInstall()`. A dismissible `InstallPrompt.svelte` banner (desktop + mobile) calls it. No auto-popup; Lighthouse allows a manual install button.
- `src/lib/components/OfflineIndicator.svelte`: keep; add a second state `updateAvailable` toast next to it (don't replace the offline banner).
- `src/routes/+layout.svelte`: import `OfflineIndicator` (already present) + conditional `InstallPrompt` + update toast. Don't add ForumBottomNav-style bottom nav outside `/forum` — keep the forum PWA nav scoped to `/forum*`.

### Offline UX per page (so every link is usable)
- **Reader (`/read/`, `/works/`)**: already `cache-first` — add an offline banner like `"You're offline — reading from cache"` at the top of the reader when `navigator.onLine === false` but the fic is cached. The `RUNTIME_FIC` cache is key.
- **Forum**: online works as today. Offline: the category grid + topic shells render from the cached navigation fallback; topic lists will fetch and fail → show per-section offline empty states: `"Offline — categories/topics will update when you're back online. Drafts are saved locally."` Persist forum drafts in `localStorage` (already used for forum drafts) so offline replies queue and a `sync` on `online` event retries `createPost`/`updatePost` (add `src/lib/pwa/forumDraftQueue.ts` with a simple IndexedDB or localStorage queue + `navigator.onLine` + `visibilitychange` retry).
- **Search/bookmarks/lists/shelves/notifications**: these are private or must stay `none` in the SW, so offline they must degrade gracefully: show cached shell + an inline empty state that says `"You're offline — search/bookmarks need a connection"` rather than an infinite spinner. No caching of private data to make them offline-readable.
- **Docs (`/docs`)**: static; add `docs` prefix to `PRECACHE` or to navigation preload so docs pages are offline-readable.

### Push (optional, not in this milestone)
The forum-nodebb spec calls for VAPID push (FR-032). Keep it out of this plan's scope — it's an `admin` VAPID key + `PushSubscription` table + `/api/push/subscribe` endpoint. Card it as follow-up P3; the PWA is installable/offline-capable without it. If push is wanted later, track it as `docs/plans/pwa-push-vapid.md`.

## Implementation sequence (small, reviewable patches)

1. **Manifest + app.html** — add `screenshots`, `categories`, extra `shortcuts`, `apple-touch-icon`, split icon purposes. Verify with Lighthouse → Manifest section.
2. **Offline shells + fallback page** — extend `decideStrategy`/`sw.js` for all navigations network-first + cache, add `offline.html`, add tests in `strategies.test.ts` for `Accept: text/html` / `mode: navigate` branch. Bump `VERSION` stamping (script or vite plugin).
3. **Registration update prompt + install prompt** — `register.ts` store + `InstallPrompt.svelte` + wiring in `+layout.svelte`. Unit tests in `register.test.ts` + manual `beforeinstallprompt` test.
4. **Per-page offline empty states** — reader offline banner, forum draft queue + offline empty states, search/bookmarks graceful offline. Add `forumDraftQueue.test.ts` and component tests for the empty states.
5. **Browse-every-link pass** — run the full inventory in a real browser (online + offline toggle), fix what the crawl finds, record results back in this doc's checklist + in the PR description. Requires a running `npm run dev` (host `0.0.0.0`) + a backend (local `cargo run` or ThinkCentre tunnel) — the previous attempt hit `chromewebdata` proxy error because no dev server was running.

## Testing

- Unit: `npm test -- --run src/lib/pwa` (register + strategies). Extend `strategies.test.ts` for navigation/offline branches and for `FIC_API_PREFIXES` edge. Cover `OfflineIndicator` + `InstallPrompt` + `forumDraftQueue`.
- Build: `npm run build` must stay green — `sw.js` stays in `static/` (served at `/sw.js`), not `src/`; the stamp script must not break `adapter-static`.
- Manual: Lighthouse PWA audit (Chrome DevTools → Lighthouse → Progressive Web App) — every category must be green except optional push. `Application → Manifest`, `Service Workers` (registered, activated, offline tick).
- Brower-use crawl: `new_tab('http://localhost:5173/')` + click every header/footer/nav/link pattern, toggle `Offline` in `Application → Service Workers`, revisit every route from the inventory and assert shell + empty state.

## Rollout

- Land behind no feature flag — changes are additive and backward-compatible. The `VERSION` stamp invalidates old caches automatically. `skipWaiting` + `controllerchange` reload ensures users see the new shell after one refresh.
- Deploy is the same SPA rsync: `npm run build` + `rsync --delete build/ thinkcentre:/var/www/ficnexus/` (or `FRONTEND_DIR` as per `.env`). No backend migration.

## Verification

After deploy: hard refresh → `Application` shows `/sw.js` active, `Manifest` ok, `Install` button appears (or prompt banner), `Offline` tick shows cached shells render, reader pages open offline, forum categories render offline shells with draft queue.

## Out of scope

- Workbox integration, VAPID push, TWA packaging, background sync for non-forum writes, full search offline. Those are follow-upside P3+.
