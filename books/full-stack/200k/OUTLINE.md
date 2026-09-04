# OUTLINE.md — Building FicHub: The Full‑Stack 200K Tutorial (2026‑08 edition)

> Build FicHub feature‑by‑feature: one backend route, then its frontend page, then peel back the real implementation. Covers every shipped module as of Aug 2026 plus the full roadmap — every feature idea with pros/cons/verdict. By the end you have touched every module in the repo and built the whole thing yourself.

**Project**: FicHub — self‑hosted fanfiction archive + community platform
**Stack**: Rust 2024, Axum 0.8, SQLx 0.9, PostgreSQL, Redis, Ollama (lfm2.5:8b), epub‑builder, Tera, SvelteKit 5, Vite, forum‑core crate
**Word target**: 200 000 words (no upper bound — cover every feature)
**Parts**: 47 parts, ~90+ chapters
**Mode**: Toy‑first, then real. Each feature: skeleton → real implementation → polish.

---

## Part 1 — Boot the Server (4 chapters)

1.1 The empty project: `cargo init`, `Cargo.toml`, what each dependency is for  
1.2 `src/main.rs`: Tokio runtime, dotenv, the `run()` call  
1.3 `src/server.rs`: `AppState` — every piece of shared state, why it exists  
1.4 `axum::serve`: binding port 8000, the fallback static‑file layer, CORS, TraceLayer  
**Try It Yourself**: start the server, curl `/health`, see the TraceLayer log line.

---

## Part 2 — Hello Route: `/health` and `/api/remote` (3 chapters)

2.1 Backend: write `health_handler` — return JSON `{"status":"ok"}`  
2.2 Backend: write `remote_handler` — extract `ConnectInfo`, return IP/port  
2.3 Frontend: none yet — open the API in curl and the browser  
**Try It Yourself**: curl both endpoints, add a third route `/api/ping` that echoes a query param.

---

## Part 3 — Database Foundations (5 chapters)

3.1 `src/db/config.rs`: `DATABASE_URL`, `PgPoolOptions`, connection multiplexing  
3.2 `migrations/001_initial.sql`: the core tables (works, fic_info, authors, users, ratings, reviews, bookmarks, follows)  
3.3 `src/db/models.rs`: SQLx `FromRow` structs for every core table  
3.4 `src/db/queries.rs`: the first query — `get_work_by_url_id`  
3.5 Try It Yourself: write a query that counts works by fandom  
**Watch Out**: SQLx compile‑time checking needs a live DB at build time.

---

## Part 4 — Get a Single Work (frontend + backend) (5 chapters)

4.1 Backend: `GET /api/works/:id` — `get_work_handler`  
4.2 Backend: SQL join work → author → tags → rating stats  
4.3 Frontend: `src/routes/work/[workId]/+page.svelte` — shell, fetch, error state  
4.4 Frontend: render title, author, synopsis, word count, rating  
4.5 Try It Yourself: add the fandom tag chips to the work page  
**Key Concept**: why the work page queries several endpoints and merges them in Svelte.

---

## Part 5 — Search Works (6 chapters)

5.1 Backend: `GET /api/search` — `search_handler`  
5.2 Backend: `src/search/parser.rs` — parse query strings into clauses  
5.3 Backend: `src/search/builder.rs` — build SQL from parsed clauses  
5.4 Backend: `src/search/suggest.rs` — autocomplete suggestions  
5.5 Frontend: `src/routes/search/+page.svelte` — search box + results  
5.6 Frontend: PowerSearch/GuidedSearch chips, history chips, zero‑state  
**Try It Yourself**: add a year filter to the search builder.

---

## Part 6 — Upload a Work (5 chapters)

6.1 Backend: `POST /api/upload` — multipart extraction  
6.2 Backend: call fanfic‑scrapers to parse HTML → metadata  
6.3 Backend: insert into `works` + `fic_info` tables, return url_id  
6.4 Backend: update and delete routes  
6.5 Frontend: drop zone, progress, result link  
**Watch Out**: untrusted HTML from the wild — sanitise before inserting into DB.

---

## Part 7 — EPUB Export (5 chapters)

7.1 Backend: `GET /api/epub` — `epub_handler`  
7.2 Backend: `epub‑builder` crate, chapters, cover  
7.3 Backend: `convert_format_handler` (MOBI/PDF/AZW3 via Calibre sidecar)  
7.4 Backend: `/cache/{etype}/{url_id}` serving + OPDS links `?h={export_hash}`  
7.5 Frontend: download buttons on the work page, format selector  
**Try It Yourself**: add a TXT export alongside EPUB.

---

## Part 8 — User Accounts (6 chapters)

8.1 Backend: `POST /api/auth/register` — bcrypt, insert user row  
8.2 Backend: `POST /api/auth/login` — JWT issuance  
8.3 Backend: `POST /api/auth/refresh` — refresh token rotation  
8.4 Backend: `GET /api/auth/me` — current user profile  
8.5 Backend: `PUT /api/auth/locale` — per‑user locale preference  
8.6 Frontend: login + register + locale selector, single auth store  
**Try It Yourself**: add a “forgot password” stub.

---

## Part 9 — Bookmarks (5 chapters)

9.1 Backend: `POST /api/bookmarks` — add bookmark  
9.2 Backend: `GET /api/bookmarks` — list bookmarks  
9.3 Backend: `DELETE /api/bookmarks/:work_id` — remove bookmark  
9.4 Backend: CSV import/export — `import_bookmarks_csv`, `export_bookmarks_csv`  
9.5 Frontend: bookmark toggle on work page, bookmark list page  
**Try It Yourself**: add a “bookmark all in series” bulk action.

---

## Part 10 — Ratings and Kudos (5 chapters)

10.1 Backend: `POST /api/ratings` — 5‑star rating  
10.2 Backend: `GET /api/ratings/:work_id` — average + distribution  
10.3 Backend: `POST/DELETE /api/kudos/:work_id` — one‑click anonymous appreciation  
10.4 Frontend: star rating widget on work page  
10.5 Frontend: kudos button + kudos count + kudos animation  
**Key Concept**: kudos ≠ ratings — design rationale and when to use each.
**Appendix C**: Kudos deep dive — the kudos feed endpoint, guest exclusion via partial unique index, notification integration (kudos_on_work), and the critical `min_kudos` ≠ kudos naming bug.

---

## Part 11 — Reviews and Comments (6 chapters)

11.1 Backend: `POST /api/reviews` — upsert review  
11.2 Backend: `GET /api/works/:id/reviews` — review list  
11.3 Backend: `DELETE /api/reviews/:id` — delete own review  
11.4 Backend: `POST /api/comments` — threaded comments  
11.5 Backend: `GET /api/works/:url_id/comments` — comment tree  
11.6 Frontend: review form + review list + comment thread  
**Try It Yourself**: add comment reply (nested threading).

---

## Part 12 — Reading Lists, Shelves, Collections (6 chapters)

12.1 Backend: `POST/GET /api/lists` — reading lists (bundles)  
12.2 Backend: `POST/GET/DELETE /api/shelves` — shelves/collections  
12.3 Backend: `POST /api/shelves/add` — add work to shelf  
12.4 Backend: `GET /api/collections` — AO3‑style collection browse  
12.5 Backend: collection item submissions + curator approval voting  
12.6 Frontend: shelf manager, collection view page  
**Try It Yourself**: add a “view all works in collection” paginated list.

---

## Part 13 — Follows and Feed (5 chapters)

13.1 Backend: `POST/GET /api/follows` — follow authors/works/series  
13.2 Backend: `DELETE /api/follows/:id` — unfollow  
13.3 Backend: `GET /api/feed` — personalised updates feed  
13.4 Backend: `GET /api/v1/updates` — updates list with seen tracking  
13.5 Frontend: follows page, updates feed page, exclusion management  
**Try It Yourself**: add “mark all as read”.

---

## Part 14 — Author Pages and Series (5 chapters)

14.1 Backend: `GET /api/authors/by-name/:name` — author bibliography  
14.2 Backend: `GET /api/authors/:id` — author profile with social links  
14.3 Backend: `PUT /api/authors/:id` — update author profile (self)  
14.4 Backend: `GET /api/series/:id` — series page with chapter order  
14.5 Frontend: author page, series page, follow‑author button  
**Try It Yourself**: add “similar authors” based on co‑followship.

---

## Part 15 — Tags, Voting, and Curation (7 chapters)

15.1 Backend: `POST /api/tags/submit` — user submits a tag  
15.2 Backend: `POST /api/tags/vote` — vote on a tag  
15.3 Backend: `GET /api/tags` — list tags, autocomplete  
15.4 Backend: `POST /api/tags/flag` — flag a bad tag  
15.5 Backend: curator routes — alias, merge, delete tags  
15.6 Backend: `src/tags/curator.rs` — curator tools  
15.7 Frontend: tag browser, tag detail page, tag voting UI  
**Try It Yourself**: add tag autocomplete in the search box.

---

## Part 16 — Forum — Threadlight Core (8 chapters)

16.1 Backend: `forum_core` crate — generic engine over actor/score types  
16.2 Backend: `GET/POST /api/forum/categories` — category CRUD  
16.3 Backend: `GET/POST /api/forum/topics` — topic CRUD + topic slugs  
16.4 Backend: `POST /api/forum/topics/:topicId/posts` — create post  
16.5 Backend: `PATCH/DELETE /api/forum/posts/:postId` — edit/delete posts  
16.6 Backend: forum moderation — queue, status, react, metamod  
16.7 Backend: forum admin — bans, lock, pin, hide  
16.8 Frontend: forum index, topic view, post composer, moderation queue  
**Try It Yourself**: add “quote post” button in the composer.

---

## Part 17 — Ask the Archive (5 chapters)

17.1 Backend: `POST /api/search/ask` — natural‑language search  
17.2 Backend: `src/search/ask.rs` — Ollama integration, prompt construction  
17.3 Backend: `src/search/ask_cache.rs` — cache ask results  
17.4 Frontend: Ask the Archive page — question box + answers  
17.5 Try It Yourself: add “copy answer” button.

---

## Part 18 — Bounties and Reputation (5 chapters)

18.1 Backend: `POST/GET /api/bounties` — create/list bounties  
18.2 Backend: `POST /api/bounties/:id/claim` — claim a bounty  
18.3 Backend: `POST /api/bounties/:id/resolve` — resolve bounty  
18.4 Backend: `src/routes/progression.rs` — XP, levels, ability tree  
18.5 Frontend: bounty board, claim dialog, progression page  
**Try It Yourself**: add bounty filter by status.

---

## Part 19 — Notifications (4 chapters)

19.1 Backend: `GET /api/notifications` — list notifications  
19.2 Backend: `POST /api/notifications/read-all` — mark all read  
19.3 Backend: notification preferences CRUD  
19.4 Frontend: notification bell, dropdown, preferences page  
**Try It Yourself**: add notification type icons.

---

## Part 20 — RSS and OPDS (5 chapters)

20.1 Backend: `GET /feed.xml` — new arrivals RSS  
20.2 Backend: `GET /feed/follows.xml` — follows feed  
20.3 Backend: `GET /feed/works/:url_id` — per‑fic feed  
20.4 Backend: OPDS catalog — `/opds`, `/opds/search`, `/opds/authors`, `/opds/tags`  
20.5 Backend: OPDS Readium Web Publication Manifest (`GET /opds/manifest`) + RWPM links in Atom feeds  
**Try It Yourself**: add an OPDS shelf feed.

---

## Part 21 — Saved Searches and Alerts (4 chapters)

21.1 Backend: `POST/GET /api/search/saved` — saved searches  
21.2 Backend: `DELETE /api/search/saved/:id` — delete saved search  
21.3 Backend: `PUT /api/search/saved/:id/alert` — alert toggle  
21.4 Frontend: saved search manager, alert indicators  
**Try It Yourself**: add “run now” button that triggers a saved search.

---

## Part 22 — Blind Date with a Fic (3 chapters)

22.1 Backend: `GET /api/blind-date` — random fic, hidden title/fandom  
22.2 Backend: `GET /api/blind-date/reveal` — reveal answer  
22.3 Frontend: Blind Date page — guess, reveal, share

---

## Part 23 — Fandoms and Trending (4 chapters)

23.1 Backend: `GET /api/fandoms` — list fandoms + landing page  
23.2 Backend: `GET /api/fandoms/:slug` — fandom detail  
23.3 Backend: `GET /api/trending` — trending fics  
23.4 Backend: `GET /api/trending/tags` — trending tags

---

## Part 24 — Work Proposals and Fic Suggestions (4 chapters)

24.1 Backend: `POST/GET /api/work-proposals` — propose a work for addition  
24.2 Backend: `POST /api/work-proposals/:id/vote` — vote on proposal  
24.3 Backend: `GET/POST /api/fic-suggestions` — community suggestions  
24.4 Backend: suggestion voting + removal  
24.5 Frontend: work proposals page, fic suggestions page

---

## Part 25 — Pseuds and Creatorships (4 chapters)

25.1 Backend: `GET/POST /api/pseuds` — pseud (pen name) management  
25.2 Backend: `PATCH/DELETE /api/pseuds/:id` — update/delete pseud  
25.3 Backend: creatorship invite/approve/reject flow  
25.4 Frontend: pseud manager, creatorship requests

---

## Part 26 — Skins and Customisation (5 chapters)

26.1 Backend: `GET/POST /api/skins` — skin CRUD  
26.2 Backend: `GET/PUT /api/me/theme` — user theme  
26.3 Backend: `GET /api/nav` — navigation config  
26.4 Backend: `GET /api/widgets` — widget config  
26.5 Frontend: skin browser, theme picker, layout editor

---

## Part 27 — Reading Quests and Stats (4 chapters)

27.1 Backend: `POST /api/reading/status` — update reading status  
27.2 Backend: `GET /api/reading/list` — reading list  
27.3 Backend: `POST /api/reading/record` — record a read  
27.4 Backend: `GET /api/users/:id/streak` — reading streak  
27.5 Frontend: reading stats page, quest progress

---

## Part 28 — Reading History (3 chapters)

28.1 Backend: `GET/POST /api/reading/history` — per‑visit history  
28.2 Backend: `DELETE /api/reading/history/:id` — delete entry  
28.3 Backend: `POST /api/reading/history/clear` — clear all

---

## Part 29 — Translations (5 chapters)

29.1 Backend: `GET /api/locales` — list locales  
29.2 Backend: `GET /api/translations/:locale_code` — UI translations  
29.3 Backend: `POST /api/works/:id/translations` — upsert work translation  
29.4 Backend: `GET /api/works/:id/translations/:locale_code` — get translation  
29.5 Backend: chapter translation upsert + diff workflow  
29.6 Frontend: translation editor, diff view, review queue  
**Try It Yourself**: add a “translation progress bar” on work pages.

---

## Part 30 — Send to Kindle (3 chapters)

30.1 Backend: `POST /api/send-to-kindle` — email a fic to Kindle  
30.2 Backend: `src/routes/kindle.rs` — Lettre SMTP, format selection  
30.3 Frontend: Send to Kindle button on work page

---

## Part 31 — Roadmap Consensus Engine (4 chapters)

31.1 Backend: `POST /api/roadmap/suggest` — suggest a feature  
31.2 Backend: `GET /api/roadmap/arena` — arena of suggestions  
31.3 Backend: `POST /api/roadmap/vote` — vote in arena  
31.4 Backend: `GET /api/roadmap/consensus` — consensus results  
31.5 Frontend: roadmap page, arena UI, vote buttons

---

## Part 32 — Ollama Embeddings and Recommender (6 chapters)

32.1 Backend: `src/services/ollama.rs` — Ollama client, embed + generate  
32.2 Backend: `GET /api/recommendations` — general recommendations  
32.3 Backend: `GET /api/recommendations/personal` — personalised recs  
32.4 Backend: `src/recommender/engine.rs` — recommendation engine  
32.5 Backend: strategy registry — cooccur, decay, embeddings, MF, hybrid, bandit, curator prior  
32.6 Frontend: recommendations panel on work page, dedicated rec page  
**Try It Yourself**: add “why this recommendation” tooltip.

---

## Part 33 — Recipe Builder (4 chapters)

33.1 Backend: `GET/POST /api/recipes` — list/create recipes  
33.2 Backend: `PUT/DELETE /api/recipes/:id` — update/delete recipe  
33.3 Backend: `POST /api/recipes/:id/activate` — activate recipe  
33.4 Backend: `POST /api/recipes/:id/install` — install recipe  
33.5 Backend: `POST /api/recipes/:id/publish` — publish to gallery  
33.6 Frontend: recipe builder UI, gallery browser

---

## Part 34 — Moderation and Admin (8 chapters)

34.1 Backend: `GET /api/modlog` — public moderation log  
34.2 Backend: admin user management — list, role, ban  
34.3 Backend: admin moderation queue — approve/reject uploads  
34.4 Backend: content scan — scan, review, trust  
34.5 Backend: admin reports — list, resolve  
34.6 Backend: auto‑tagger — trigger, queue, approve/dismiss  
34.7 Backend: admin blacklist — fic, author  
34.8 Frontend: admin dashboard, mod queue, report queue  
**Try It Yourself**: add a “bulk approve” button for the mod queue.

---

## Part 35 — Auth Deep Dive: JWT, Sessions, Refresh (4 chapters)

35.1 Backend: `jsonwebtoken` — token structure, claims  
35.2 Backend: refresh token rotation, revocation  
35.3 Backend: `axum‑extra` typed headers for auth  
35.4 Backend: shadowban detection via proof‑of‑work  
35.5 Frontend: auth interceptor, token refresh flow

---

## Part 36 — Rate Limiter and Redis (4 chapters)

36.1 Backend: `src/limiter/` — tiered rate limiter  
36.2 Backend: Redis bucket algorithm  
36.3 Backend: datacenter IP tagging  
36.4 Backend: shadowban via proof‑of‑work challenge  
36.5 Frontend: rate limit error UI, retry‑after

---

## Part 37 — Proof of Work (3 chapters)

37.1 Backend: `GET /api/pow/challenge` — issue challenge  
37.2 Backend: `POST /api/pow/solve` — verify solution  
37.3 Backend: shadowban flow — challenge → solve → allow

---

## Part 38 — Frontend Foundations (5 chapters)

38.1 `frontend/package.json` — SvelteKit 5, Vite, static adapter  
38.2 `frontend/src/lib/api/client.ts` — API client, base URL, error handling  
38.3 `frontend/src/lib/i18n/` — localisation dictionaries (en, de, es, fr, pt‑BR, zh)  
38.4 `frontend/src/lib/ui/` — shared components (ArchiveHeader, WorkBlurb)  
38.5 Frontend: SvelteKit routing, `+page.svelte`, `+page.test.ts`

---

## Part 39 — Frontend Search UI (4 chapters)

39.1 `SimpleSearch.svelte` — simple search box  
39.2 `PowerSearch.svelte` — power search with chips  
39.3 `GuidedSearch.svelte` — guided search form  
39.4 `src/lib/api/search.ts` — search API client  
39.5 Frontend: search results rendering, zero‑state, error state, history chips

---

## Part 40 — Frontend Work and Fic Pages (4 chapters)

40.1 `src/routes/fic/[urlId]/+page.svelte` — fic page  
40.2 `src/routes/work/[workId]/+page.svelte` — work page  
40.3 `src/lib/ui/archive/WorkBlurb.svelte` — work summary component  
40.4 Frontend: read status toggle, bookmark button, kudos button, symbol squares

---

## Part 41 — Frontend Dashboard and Profile (4 chapters)

41.1 `src/routes/dashboard/+page.svelte` — user dashboard  
41.2 `src/routes/authors/[name]/+page.svelte` — author page  
41.3 `src/routes/updates/+page.svelte` — updates feed  
41.4 Frontend: progression widget, recent activity

---

## Part 42 — Frontend Forum and Curator (5 chapters)

42.1 `src/routes/forum/+page.svelte` — forum index  
42.2 `src/routes/curator/+page.svelte` — curator dashboard  
42.3 `src/routes/curator/approvals/+page.svelte` — approval queue  
42.4 `src/routes/curator/flags/+page.svelte` — flag queue  
42.5 `src/routes/curator/consensus/+page.svelte` — consensus feed

---

## Part 43 — Frontend Social Features (4 chapters)

43.1 `src/routes/follows/+page.svelte` — follows management  
43.2 `src/routes/follows/Exclusions.svelte` — exclusion editor  
43.3 `src/routes/ask/+page.svelte` — ask the archive  
43.4 `src/routes/search/+page.svelte` — full search experience

---

## Part 44 — Frontend Collections and Shelves (3 chapters)

44.1 `src/routes/work-proposals/+page.svelte` — work proposals  
44.2 `src/routes/curator/work-deletions/+page.svelte` — deletion requests  
44.3 Frontend: collection view, shelf manager, list editor

---

## Part 45 — Testing (4 chapters)

45.1 Backend: `src/routes/mod.rs` — `api_contract_tests`  
45.2 Backend: `cargo test`, SQLx `prepare --check`  
45.3 Frontend: `src/routes/*/page.test.ts` — SvelteKit route tests  
45.4 Frontend: Vitest, Testing Library, mock API

---

## Part 46 — Deployment (4 chapters)

46.1 `Dockerfile` — multi‑stage Rust build  
46.2 `docker‑compose.yml` — PostgreSQL, Redis, Ollama, FicHub  
46.3 `deploy.sh` — production deploy script  
46.4 Nginx config — reverse proxy, static frontend (note: on prod the Rust server serves static files directly on :8000)

---

## Part 47 — Roadmap Deep‑Dive: What Could Come Next (NEW — 2026‑08 audit)

> This part does not teach you to build something that exists today. It walks through every feature idea in the repo’s roadmap — what’s already shipped, what’s a cheap win, what’s a big bet, and a clear verdict on each. Use it as the map for your own FicHub instance.

### 47.1 How to read this part
- Each entry: **what it is**, **what already exists**, **pros**, **cons**, **verdict**.
- Verdict values: `Ship now` (already shipped or ready), `Cheap win` (one query / small module), `Spec first` (big, needs design), `Defer` (needs user base / data first), `No` (violates house policy).

### 47.2 Shipped — already in the codebase
A quick tour of everything the previous 46 parts built, with the commit / migration pointers a junior dev can grep for.

### 47.3 P1 — Content bottleneck
- **Cookie ingestion for AO3/FFN** — what it is, why the host is blocked, the pragmatic flow, pros/cons, verdict.
- **Scriptable API‑surface e2e in CI** — route‑walk as a gate, pros/cons, verdict.
- **Main‑char / ship score fixing** — already removed; why that matters for search overhaul.

### 47.4 P3 — Make the rec platform earn its keep
- **Shadow‑run decay + embeddings** — both built, cheap, golden test, pros/cons, verdict.
- **Sequential Markov in Next Up** — wiring `rank_targets` as an additional Next Up source, pros/cons, verdict.
- **Personalised recs by default** — wire `/personal` when logged in, toggle in settings, pros/cons, verdict.

### 47.6 P4 — Differentiators
- **Full‑text search over fic bodies** — shipped, tsvector + GIN + `<mark>` snippets, why it matters, verdict.
- **Analytics feedback loop** — `usage_events` + search‑to‑export conversion, verdict.
- **Per‑endpoint usage breakdown** — one GROUP BY over `usage_events`, pros/cons, verdict.

### 47.7 P5 — Curator/admin UI backlog
- Auto‑tag review UI, manual fic approval UI, metadata correction, report handling, translation post‑edit, rating/warning verification — each with status, pros/cons, verdict.

### 47.8 P6 — Cheap UX wins
|- Loading skeletons (partial), ask‑the‑archive response caching, auth‑store consistency (done), search‑history chips (done), user skins / work skins (done + marketplace follow‑up), half / quarter star ratings (one `user_prefs` column, no schema change), kudos privacy toggle (`kudos_private` in `user_prefs`; anonymous kudos not allowed — already shipped).

### 47.9 P7 — Bigger bets
- **Invite the small cohort** — onboarding flow + seed content + new‑member landing, verdict.
- **Verify ask‑the‑archive live translation with lfm2.5:8b** — live test result, verdict.

### 47.10 P8 — 50‑feature fit audit (the big list)
A chapter per cluster: download & ingestion, search & discovery, reader & reading experience, recommendations & personalisation, community & engagement, admin dashboard & engagement metrics, OTW Archive fit audit, brainstorm review, fastest OTW wins, user‑facing priority batch (UF). Each feature: one paragraph, pros/cons, verdict.

### 47.11 P8.5 — Bot + CLI feature parity
`fanfic‑archivist` Discord bot + `fichub` CLI gap list, what’s shipped, what’s ops/security P1/P2/P3, verdict.

### 47.12 P9 — Extension platform (v3.1)
Plugin manifest, theme design tokens, recipe builder — shipped. Deferred items: multi‑tenant sidecars, WASM sandbox, full marketplace, nav/layout editor, layout presets, community rec leaderboard, recipe→precomputed promotion, XP from customisation reputation, seasonal event themes, full marketplace with curator review. Each with why deferred + verdict.

### 47.13 Wattpad‑style feature audit (Group A/B/C)
Inline paragraph commenting (Marginalia already covers), author‑facing analytics, personal reading‑stats dashboard, community writing contests, algorithmic infinite scroll, built‑in drafting + scheduled publishing, spendable virtual currency, private DMs, public downvoting, walled‑garden mobile app — each with FicHub‑policy verdict.

### 47.14 Cheapest wins cheat‑sheet
A one‑page table: the single‑query / existing‑infra items you can ship this weekend.

---

*Each chapter is a standalone tutorial: read the code, type the changes, verify with curl or the browser. Features are presented in dependency order — you build the foundations first, then the features that depend on them. Backend and frontend for each feature are shown together so you always see the full request/response cycle.*
