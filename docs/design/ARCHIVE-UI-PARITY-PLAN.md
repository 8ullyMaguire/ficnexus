# ARCHIVE-UI-PARITY-PLAN.md

## 1. Overview

This document is a gap analysis documenting discrepancies between the **OTW Archive (AO3)** reference behavior (observed in the cloned `~/code/ruby/otwarchive` repo) and **Fichub's archive frontend** (SvelteKit, served under the archive theme). It is organized as a parity checklist for bringing Fichub's archive UI closer to AO3's layout, markup structure, and data presentation conventions.

The goal is **UI/data parity**, not a full AO3 theme port. Each gap is classified by priority (High / Medium / Low) based on how visibly it diverges from the AO3 reference and how foundational the change is to the archive page structure.

## 2. High-priority gaps

These are foundational discrepancies that affect core navigation and the primary browse surfaces users hit first.

### 2.1 Navbar dropdown interaction (hover vs click)

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Interaction model | CSS-only `:hover` / `.open` on `#header`; mouseover opens, mouseaway closes | `<details>/<summary>` click-to-open; no hover | Fichub uses click-to-open `<details>` where AO3 uses pure CSS hover. Dropdowns behave opposite to user expectation set by AO3. |
| Markup | `<ul class="dropdown-menu"><li><a>…</li></ul>` inside `<li class="dropdown">` | `<details>`/`<summary>` accordion | Fichub's `ArchiveHeader.svelte` should switch to AO3-style `<li class="dropdown">` + `<ul class="dropdown-menu">` driven by `:hover`/`.open`. |
| Top nav items | About, Fandoms, Browse, Search, Help | Fandoms, Browse, Search ▾, About ▾, Requests, Forum, Ask, inline search | Fichub is missing Help; adds Requests/Forum/Ask inline. The inline search widget also differs. |
| User nav | Hi username, Dashboard, Post, Log Out (right side) | (not specified as parity target) | Fichub should ensure a parallel user nav block. |

**Action:** Rewrite `ArchiveHeader.svelte` to use CSS hover dropdowns (`<li class="dropdown">` + `<ul class="dropdown-menu">`), remove `<details>/<summary>`, match the AO3 top-nav labels (add Help, reconsider Requests/Forum/Ask placement), and mirror the user nav (Hi username / Dashboard / Post / Log Out).

### 2.2 Fandoms page layout

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Layout | Plain text alphabet index A–Z, grouped by media type (Anime, Books, etc.) | Modern card grid with fic count + word count, card shadows, images | Fichub shows cards + images; AO3 shows a flat alphabetical text list with letter anchors. |
| Sort controls | None (alphabetical index) | Sort buttons (by name, by size) present | Fichub adds sort UI AO3 does not have on this page. |
| Stats | No per-fandom stats on the index | Fic count + word count displayed per card | AO3 fandoms index shows no stats; Fichub shows fic/word counts inline. |
| Grouping | Media-type grouping, letter anchors | Card grid | Fichub needs letter anchors (`#a`, `#b`, …) and media-type groupings as plain text link lists. |

**Action:** Reimplement the Fandoms page as an alphabetical A–Z index with letter anchors, grouped by media type, using plain text links (no cards, no images, no per-fandom stats). Remove or relocate the sort-by-name/sort-by-size buttons and the card/shadow styling.

### 2.3 People / authors browse page

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Path / controller | `/people` — `PeopleController#search` and `#index` | `/authors` — search-only page (input + button) | Fichub path is `/authors` vs AO3 `/people`; no index/browse. |
| Index | Alphabetical browse of users + search form | No alphabetical browse; search only | AO3 has an index + search; Fichub has search-only. |
| PeopleBlurb | avatar, name, bio snippet | Not implemented | Fichub renders no avatar/name/bio people blurbs. |
| Fandoms browse | People browse supports browsing by name/fandom | Fichub has no browse-by-fandom for people | Missing the AO3-style browse facet. |

**Action:** Add an alphabetical People index (mirror AO3 `PeopleController#browse`), render `PeopleBlurb` (avatar + name + bio snippet), and add the search form with name/fandom facets. Align the path to AO3 conventions (`/people`).

## 3. Medium-priority gaps

These affect page structure and data presentation on secondary but important surfaces.

### 3.1 Tags page

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Path | `/tags` | `/tags` | (path matches) |
| Layout | "Most Popular" + "Random" tabs | Created in Phase 2 — no tabs | Fichub is missing the tab UI. |
| Tag cloud | `cloud1`–`cloud8` CSS classes for font-size weighting | Border/padding boxes per tag type, no cloud | No tag-cloud font weighting; uses box layout instead. |
| Usage counts | Not displayed | Displayed | AO3 hides counts; Fichumb shows them. |
| Category grouping | Flowing cloud, no borders | Per-tag-type border/padding boxes | Visual grouping differs structurally. |

**Action:** Replace the box-per-category layout with an AO3-style tag cloud using `cloud1`–`cloud8` classes, add "Most Popular" / "Random" tabs, and hide (or toggle) usage counts to match AO3.

### 3.2 Work detail page metadata

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Metadata block | `dl.work.meta` full metadata: tags by category, Language, Series, Stats | ArchiveWork.svelte: title, byline, StatsLine, summary, action buttons, chapters list, author notes | Missing `dl.work.meta` definition list, Language display, Series display. |
| Social metrics as meta | Hits/Kudos/Bookmarks/Comments/Collections shown as metadata | Social counts not surfaced as metadata | Fichub has a `StatsLine` (single line) vs AO3 `<dl class="work meta">`. |
| Chapter nav header | Work header navigation present | Download dropdown present but no work header nav | Missing chapter navigation header. |
| Preface/chapters/afterword | Preface, chapters, afterword sections | Chapters list + author notes | Afterword presentation differs. |

**Action:** Add a `dl.work.meta` block to `ArchiveWork.svelte` rendering tags by category, Language, Series, and the full Stats (Words, Chapters, Comments, Kudos, Bookmarks, Hits) plus Collections. Add a chapter navigation header and ensure preface/afterword sections are presented as in AO3.

### 3.3 Search page form structure

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Fieldset legend | `<legend>` + `<h3 class="landmark heading">` for each section | `<legend>` but NO `<h3 class="landmark heading">` | Fichumb missing the landmark headings. |
| Form sections | Work Info, Work Tags, Work Stats, Sort | (ArchiveWorkSearchForm exists) | Section headings need AO3-style `<h3 class="landmark heading">`. |
| Subnav | "Edit Your Search" link appears after results | NO "Edit Your Search" subnav link | Missing post-results subnav link. |
| Search buttons | Two "Search" buttons (top + bottom of form) | Two "Search" buttons present | (matches) |
| WorkBlurb tags | `<ul class="tags commas"><li>…</li></ul>` | Tags shown via TagSoup (labeled rows) | Fichumb uses labeled-row TagSoup, not comma-joined tag lists. |
| WorkBlurb meta | `<dl class="work meta">` with `<dt>` labels | StatsLine (single line) | Not `<dl>` based. |

**Action:** Add `<h3 class="landmark heading">` under each `<legend>` in `ArchiveWorkSearchForm`; add the "Edit Your Search" subnav link after results; reconsider TagSoup vs AO3 `<ul class="tags commas">` rendering for search-result blurbs.

### 3.4 WorkBlurb rendering differences

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Order | title → byline → fandoms line → required-tags symbol icons (rating/warning/category) → comma-joined tags → summary → series info → `dl.stats` (words, chapters, comments, kudos, bookmarks, hits) → collection count | rating badge → title → byline → TagSoup → snippet → StatsLine → action buttons | Missing fandoms line, required-tags symbol icons, series info, collection count. |
| Required-tags icons | Rating/warning/category symbols | Rating badge | Different representation; AO3 uses symbol icons, Fichumb a badge. |
| Stats | `dl.stats` | StatsLine (single line) | Structural difference. |
| Collection count | Displayed | Not displayed | Missing. |

**Action:** Reorder WorkBlurb to match AO3 sequence; add the fandoms line, required-tags symbol icons, series info, collection count; switch from TagSoup to comma-joined `<ul class="tags commas">` lists in archive-style blurbs; render stats as `dl.stats`.

## 4. Low-priority gaps

Polish and secondary surfaces that diverge but are less foundational.

### 4.1 Bookmarks page

| Aspect | AO3 (reference) | Fichub (current) | Gap |
|---|---|---|---|
| Index | Bookmarks index showing work blurbs for a user's bookmarks | Single list of user's bookmarks with fallback cards | Fichumb merges into one list. |
| Bookmark search | Separate Bookmark Search form at its own path with facets (bookmarker, work, etc.) | No Bookmark Search form variant | Missing the separate bookmark search form/page. |
| Bookmark blurbs | Show bookmark status icons + notes | (uses ArchiveWork style WorkBlurb) | Missing bookmark-specific status icons and notes display. |

**Action:** Add a dedicated Bookmark Search form/page with facets (bookmarker, work, etc.); render bookmark blurbs with bookmark status icons and user notes; keep the index page showing AO3-style work blurbs.

### 4.2 Home page — **NOT IN SCOPE**

The Fichub home/ dashboard page is **intentionally preserved as-is** (download bar + Recent Works + Trending sidebar). It is not subject to AO3 parity work. This plan does not touch the dashboard layout.

## 5. CSS / visual parity notes

Many gaps are structural; a few are pure visual/CSS. These can largely be addressed via the existing CSS variable layer without markup rewrites.

| Item | AO3 (reference) | Fichub (current) | Note |
|---|---|---|---|
| Base font | `Lucida Grande, Verdana, Helvetica, sans-serif` (sans-serif) | `Georgia/Times` serif | Fichumb intentionally uses serif for reading; consider a `--archive-body-font` toggle or accept as a deliberate brand choice. |
| Link color | `#900` (maroon) | `--archive-link #990000` (maroon) | Matches. |
| Text color | `#2a2a2a` | `--archive-text #2a2a2a` | Matches. |
| Breadcrumbs | `#666` text, `#900` links | Not specified | Fichumb should align breadcrumb colors. |
| `fieldset` | `border: 1px solid #ddd; padding: 10px;` | `border 1px solid --archive-border, padding 10px` | Matches if `--archive-border == #ddd`. |
| Tables | `border-collapse`, `#ddd` borders, `th { background: #eee }` | Not specified | Should ensure table styles match on search/results tables. |
| Tag links | maroon color, no background, hover underline | maroon color, no background | Matches AO3. |
| TagSoup `<dl>` | `dt { font-weight: bold; }`, `dd { margin: 0; }` | TagSoup uses `<dt>` (bold gray) + comma-separated links in `<dd>` | TagSoup already approximates the AO3 `<dl>` pattern, but AO3 WorkBlurb tags use `<ul class="tags commas">` — these are two different patterns and should not be conflated. |

**Note on the two tag patterns:** AO3 uses `<ul class="tags commas">` (comma-joined `<li>`) on WorkBlurbs and search results, but uses CSS classes `cloud1`–`cloud8` on the `/tags` cloud page. Fichumb's `TagSoup` component uses `<dt>`/`<dd>` labeled rows, which loosely mirrors AO3's `dl.work.meta` but is **not** the same as AO3's WorkBlurb tag rendering. Do not treat TagSoup as equivalent to AO3's tag lists.

## 6. Prioritized summary table

| Priority | Page/Section | AO3 reference | Fichub current | Parity action |
|---|---|---|---|---|
| High | Navbar | CSS hover dropdowns, mouseover open | `<details>/<summary>` click-to-open | Rewrite `ArchiveHeader.svelte` to CSS hover + `<ul.dropdown-menu>`; match nav labels; add user nav |
| High | Fandoms | Alphabetical A–Z text list, media-type groups, letter anchors | Card grid, images, fic/word counts, sort buttons | Reimplement as text index with letter anchors; remove cards/stats |
| High | People | `/people` index + search, PeopleBlurb (avatar/name/bio) | `/authors` search-only, no blurbs | Add index/browse + PeopleBlurb; align path to `/people` |
| Medium | Tags | `/tags` Most Popular/Random tabs, `cloud1`–`cloud8` weighting | Border boxes per category, usage counts, no tabs | Add tabs; switch to cloud weighting; hide usage counts |
| Medium | Work detail | `dl.work.meta` (tags/Language/Series/Stats), chapter nav header | StatsLine, missing meta/dl, no chapter nav | Add `dl.work.meta`, Language, Series, chapter nav header |
| Medium | Search form | `<legend>` + `<h3 class="landmark heading">`, "Edit Your Search" subnav | `<legend>` only, no subnav link | Add landmark headings; add "Edit Your Search" link |
| Medium | WorkBlurb | fandoms line, symbol icons, comma tags, `dl.stats`, series, collection count | rating badge, TagSoup, snippet, StatsLine | Reorder + add missing fields; switch to `<ul class="tags commas">`/`dl.stats` |
| Low | Bookmarks | Separate index + Bookmark Search form, status icons + notes | Single list, no search form variant | Add bookmark search form/page; render status icons + notes |
| Low | Home page | — | — | **NOT IN SCOPE** — dashboard preserved as-is (download + Recent + Trending)
| CSS | Fonts | sans-serif (Lucida Grande/Verdana/Helvetica) | serif (Georgia/Times) | Deliberate choice — flag, do not auto-change |
| CSS | Tag links | maroon, no bg, hover underline | maroon, no bg | Already matches |
| CSS | Tables / breadcrumbs | `border-collapse`, `#eee` th, `#666`/`#900` crumbs | Not specified / unspecified | Align table + breadcrumb styles |

## 7. Implementation notes (non-blocking, order suggestions)

1. **Non-blocking / incremental.** None of these changes block Fichub's existing functionality. The archive theme is a layer; parity improvements can be shipped incrementally.

2. **Suggested order** (highest leverage first):
   - **Step 1 — Navbar:** rewriting `ArchiveHeader.svelte` is self-contained and gives immediate AO3-like interaction. Low risk of backend coupling.
   - **Step 2 — Fandoms + People browse:** both are list-style pages with no card dependencies; implement together since they share the "plain text alphabetical index" pattern.
   - **Step 3 — WorkBlurb + WorkBlurb list:** central to Search, Bookmarks, and Work detail listings; doing this once and reusing the ArchiveWorkBlurb component pays off across pages.
   - **Step 4 — Work detail metadata:** add `dl.work.meta` to `ArchiveWork.svelte`; reuse the same `<dt>/<dd>` pattern from WorkBlurb for consistency.
   - **Step 5 — Tags cloud + Search form landmarks:** smaller, page-local changes.
   - **Step 6 — Bookmarks search form:** the largest new surface; defer until the shared blurb/meta components exist.
   - **Dashboard (NOT IN SCOPE):** the home/dashboard page is explicitly excluded. It keeps the download bar + Recent Works + Trending sidebar as-is.

3. **Component reuse.** AO3 renders WorkBlurb-like structures on Search, Bookmarks index, and Work detail. Fichumb should factor a shared `ArchiveWorkBlurb.svelte` (using `<ul class="tags commas">` + `dl.stats` + fandoms line + symbol icons + series + collection count) and reuse it in search results, bookmark listings, and related-work/series lists. Avoid the label-row `TagSoup` in archive-style contexts — reserve it for the meta `dl` (tags-by-category) block.

4. **CSS variables.** Most color/font values already match via `--archive-link`, `--archive-text`, `--archive-border`. The serif-vs-sans body font is a deliberate reading-comfort choice; do not normalize to AO3's sans-serif without a product decision. Keep a documented `--archive-body-font` override point if a future toggle is desired.

5. **Path conventions.** AO3 uses `/people`, `/fandoms`, `/tags`, `/search`. Fichumb uses `/authors` (vs `/people`). Renaming is a routing change; do it only if there is no downstream dependency on `/authors` (check robots, internal links, tests). If the rename is blocked, the behavioral parity (index + blurbs + search) is still achievable on the existing path.

6. **Avoid conflating TagSoup and AO3 tags.** TagSoup's `<dt>`/`<dd>` labeled rows approximate AO3's `dl.work.meta` metadata grouping, **not** AO3's WorkBlurb comma-joined `<ul class="tags commas">`. Implement them as separate rendering paths and do not let one substitute for the other during the parity work.
