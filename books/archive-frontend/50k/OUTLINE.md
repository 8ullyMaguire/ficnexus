# FicHub Archive Frontend — 50K Tutorial Book Outline

**Subject:** Archive Frontend (AO3-style UI)
**Target:** 50,000 words
**Audience:** Junior developers onboarding to the FicHub codebase
**Prerequisites:** Basic Svelte 5, TypeScript, CSS knowledge

---

## Part 1: Welcome & Architecture ( chapters 1–4 )

### Chapter 1: What Is the Archive Interface?
- FicHub has two UI modes: Modern (card-based) and Archive (AO3-style)
- Archive mode is the default for all users
- Why we built it: text-dense, tag-heavy, familiar to fanfiction readers
- The AO3 inspiration: what we copied, what we diverged on
- How users switch between modes (Settings, footer, dropdown, ?ui= URL param)

### Chapter 2: Project Structure
- Where archive code lives: `frontend/src/lib/ui/archive/`
- 17 files total: 14 Svelte components + 2 TypeScript modules + 1 test file
- How archive components relate to route pages (conditional rendering)
- The shared layer: API clients, auth store, prefs store
- File-by-file overview with line counts

### Chapter 3: The Dual-Shell Architecture
- Root layout (`+layout.svelte`) picks ArchiveLayout vs Shell
- `getPref('uiMode')` drives the switch
- `applyUiParam()` for URL override (`?ui=archive` or `?ui=modern`)
- How the modern shell stays 100% intact
- The prefs store: `src/lib/prefs.ts` — UserPrefs interface, localStorage persistence

### Chapter 4: Design Tokens & Theme System
- CSS custom properties: `--archive-bg`, `--archive-link`, `--archive-border`, etc.
- Two presets: Archive Classic (light) + Archive Noir (dark)
- Georgia serif headings, maroon accent (#990000), 2px radius
- How themes are defined in `src/lib/themes/presets.ts`
- The ThemeTokens interface and how it maps to CSS vars

---

## Part 2: Shell Components ( chapters 5–8 )

### Chapter 5: ArchiveLayout — The Wrapper
- `ArchiveLayout.svelte` (63 lines) — the simplest component
- Wraps header + main container + footer
- Re-hosts CommandPalette and HelpModal
- Initializes theme, i18n, auth on mount
- The `{@render children()` pattern (Svelte 5 slots)

### Chapter 6: ArchiveHeader — The AO3 Parity Header
- Two-row layout: logo row + red navbar
- Top row: "FicHub" Georgia maroon + avatar box + Post/Log Out
- Red navbar: Fandoms, Browse, Search ▾, About ▾ (dropdowns via `<details>/<summary>`)
- Right cluster: Requests, Forum, Ask, inline search input
- Keyboard accessible dropdowns (native HTML details element)
- No emojis anywhere (AO3 rule)

### Chapter 7: ArchiveFooter — The Red Footer
- `--archive-accent-line` background, white text
- 4-column grid: Customize, About, Contact, Development
- "Switch to modern interface" button
- Responsive: 2 columns at 600px, 1 column at 380px
- setPref + window.location.reload() pattern

### Chapter 8: ArchiveNavLink & ArchiveButton
- `ArchiveNavLink.svelte` (49 lines) — active state via `$page.url`
- `$derived` for reactive active detection
- 2px accent underline on active link
- `ArchiveButton.svelte` (58 lines) — flat bg-raised, 1px border, 0-radius
- Variant prop: primary (maroon) vs secondary (gray)
- The AO3 button aesthetic: no shadows, no gradients, no rounded corners

---

## Part 3: Content Components ( chapters 9–13 )

### Chapter 9: WorkBlurb — The Fic Card
- `WorkBlurb.svelte` (208 lines) — the core display component
- Props: `fic` (search result object with all fields)
- Renders: rating badge, title (maroon link), byline, TagSoup, snippet, StatsLine, action buttons
- Action buttons: Download, Read, Bookmark, Kudos
- Anonymous users see "Log In to Bookmark or Give Kudos"
- Box styling: 1px border, 0.75em padding, archive CSS variables

### Chapter 10: TagSoup — Tag Display
- `TagSoup.svelte` (117 lines) — labeled tag rows
- Props: `tags` (grouped by category), `showAll`
- Each row: bold gray label + comma-separated maroon tag links
- Tag links: `/search?include_tags={type}:{name}`
- Truncation: Additional Tags (category 4) truncated at 6 with "+N more" `<details>`

### Chapter 11: StatsLine — Single-Line Stats
- `StatsLine.svelte` (52 lines) — the simplest content component
- Props: words, chapters, status, kudos, updated
- Format: "Words: 1,204,116 · Chapters: 109/109 · Kudos: 412 · Updated: 3d ago"
- Muted color, 0.85em font, middot separators

### Chapter 12: rating.ts — Pure Helpers
- `rating.ts` (118 lines) — zero Svelte, pure TypeScript
- `mapRating()`: rating code → AO3 display name
- `groupTags()`: flat tag array → grouped by category (1–6)
- `formatWords()`: comma-formats numbers
- `chaptersDisplay()`: "n/n" if complete, "n/?" otherwise
- `formatUpdated()`: relative time ("3d ago")

### Chapter 13: ArchiveWork — Fic Detail Page
- `ArchiveWork.svelte` (407 lines) — the largest content component
- Props: `fic` (full ExportResponse), `isBookmarked`, `onToggleBookmark`
- Layout: title, byline, date, StatsLine, summary, action buttons, chapters list, notes
- Download dropdown with format selection
- Uses mapRating() and groupTags() from rating.ts

---

## Part 4: Search System ( chapters 14–18 )

### Chapter 14: searchForm.ts — The Pure Search Module
- `searchForm.ts` (480 lines) — the second-largest file, zero Svelte
- `parseRange()`: AO3 word count syntax ("1000", ">1000", "<5000", "1000-5000")
- `buildSearchQuery()`: FormState → URLSearchParams (drops inert fields)
- `formStateToUrl()` / `urlToFormState()`: URL round-trip for back button
- `mergeChipIntoForm()`: chip click → tag added to form field
- The SUPPORTED matrix: which params the backend accepts
- Why pure functions matter: 47 unit tests, easy to reason about

### Chapter 15: searchForm.test.ts — Testing the Module
- 47 tests covering every mapping row
- Range syntax edge cases
- URL round-trip fidelity
- Chip merge behavior
- How to run: `npx vitest run src/lib/ui/archive/searchForm.test.ts`
- Test structure: describe/it blocks, assertion patterns

### Chapter 16: ArchiveWorkSearchForm — The Two-Mode Component
- `ArchiveWorkSearchForm.svelte` (853 lines) — the largest file in the archive
- Two modes: `page` (full form) and `sidebar` (collapsed filters)
- PAGE MODE: three fieldsets (Work Info, Work Tags, Work Stats)
- Fieldset anatomy: `<fieldset>` + `<legend>` + `<dl>` rows
- SIDEBAR MODE: collapsed `<details>` sections with sort at top
- Chips row from `/api/search/suggest`
- Disabled fields: Crossovers, Hits, Language (inert with footnote)
- The form state lifecycle: onChange → formState update → onSubmit → buildSearchQuery → navigate

### Chapter 17: Search Page Integration
- `search/+page.svelte` — the route page that orchestrates everything
- No query → ArchiveWorkSearchForm in page mode
- Has query → two-column results (WorkBlurb list + sidebar form)
- Form state management: URL ↔ FormState ↔ API filters
- Pagination: AO3-style "← Previous · 1 2 3 … 10 · Next →"
- The search API call: `search(filters)` from `$lib/api/search`

### Chapter 18: Search API & Backend Connection
- `frontend/src/lib/api/search.ts` — the API client
- SearchFilters interface, SearchResult interface, SearchResponse
- How form fields map to API params (the field→API table)
- Rating as tag filter (not a column)
- Faceted results: fandoms, characters, relationships, warnings
- The suggest endpoint: `/api/search/suggest` for chips

---

## Part 5: Page-by-Page Restyle ( chapters 19–24 )

### Chapter 19: Home Page — ArchiveHome
- `ArchiveHome.svelte` (470 lines) — the archive landing page
- Compact download input at top
- Two-column layout: main (Recent Works) + sidebar (Trending)
- Personalized recs section (logged in + enough_data)
- Parallel data loading: `Promise.allSettled` with skeleton states

### Chapter 20: Tags Page — AO3 Tag Index
- `/tags` route (NEW — didn't exist before)
- Fetches all 6 tag types in parallel
- Organized by category: Fandoms, Characters, Relationships, Additional Tags, Warnings, Categories
- Each tag links to `/search?include_tags={type}:{name}`
- The `frontend/src/lib/api/tags.ts` module

### Chapter 21: Bookmarks, Authors, Notifications
- `/bookmarks` — archive conditional, WorkBlurb per bookmark, fallback cards for deleted works
- `/authors` — minimal People Search (input + author link list)
- `/notifications` — plain rows with "Mark All Read" ArchiveButton
- Pattern: every page uses `getPref('uiMode')` + `{#if uiMode === 'archive'}`

### Chapter 22: Requests & Forum
- `/requests` — list (title, status, upvotes), detail (fieldset), new (form)
- `/forum` — category index table, topic list table, post boxes, new topic form
- The bordered table pattern: `--archive-border`, compact rows, maroon links
- Forum detail: posts as bordered boxes with author header row

### Chapter 23: Ask, Settings, Fic Detail
- `/ask` — "Ask the Archive" textarea + WorkBlurb results
- `/settings` — AO3 fieldset Interface Style radio section
- `/fic/[urlId]` — ArchiveWork component with tag table, summary, chapters

### Chapter 24: ArchiveListPage — The Shared List Component
- `ArchiveListPage.svelte` (216 lines) — reusable list wrapper
- Props: title, items, loading, error, subtitle
- Loading skeleton with pulsing animation (5 skeleton blurbs)
- Empty state and error state
- Used by multiple pages for consistent list rendering

---

## Part 6: Patterns & Best Practices ( chapters 25–28 )

### Chapter 25: The Archive Conditional Pattern
- Every route page uses the same pattern:
  ```svelte
  {#if uiMode === 'archive'}
    <!-- archive view -->
  {:else}
    <!-- modern view (unchanged) -->
  {/if}
  ```
- Why this works: additive, no refactoring, modern shell untouched
- How to add archive support to a new page (step-by-step)
- Common mistakes: forgetting the `:else` branch, not importing getPref

### Chapter 26: CSS Architecture in Archive Mode
- All archive CSS uses `--archive-*` custom properties
- Never hardcode hex values in components (always use vars)
- The two-preset system: light and dark themes
- Scoped `<style>` blocks (no Tailwind, no global CSS)
- The AO3 aesthetic: thin borders, square corners, Georgia serif, compact spacing

### Chapter 27: Svelte 5 Runes in Practice
- `$props()` replacing `export let`
- `$state()` for local reactive state
- `$derived()` for computed values (uiMode, active link)
- `$effect()` for side effects (theme init, outside click handlers)
- `{#snippet}` and `{@render}` replacing slots
- Real examples from every archive component

### Chapter 28: Testing Strategy
- Unit tests: `searchForm.test.ts` (47 tests, pure functions)
- E2E tests: `e2e/archive-ui.spec.ts` (9 Playwright tests)
- What to test: form state round-trip, URL params, chip merge
- What NOT to test: styling, layout (visual regression is separate)
- Coverage gates: 85/78/76 on `src/lib/**`

---

## Part 7: Deployment & Operations ( chapters 29–30 )

### Chapter 29: Building & Deploying
- Frontend is static: `npm run build` → `build/` directory
- Deployed to ThinkCentre via NFS
- No server restart needed for frontend changes
- The `FRONTEND_DIR` env var points to the build output
- Cache busting: immutable hashed assets, 1-year cache

### Chapter 30: Architecture Decisions & Future Work
- Why Path-B (two shells in one app) over separate apps
- Why archive is default (familiarity for fanfiction readers)
- The one divergence from AO3: within-category AND vs OR
- Inert fields: why we show disabled options (parity + honesty)
- Future: restyle reader, admin, more pages into archive mode
- The `?ui=` URL param: how it enables sharing specific views

---

## Appendices

### Appendix A: File Reference
- Complete file listing with line counts and purposes
- Import dependency graph

### Appendix B: API Field Mapping
- Full table: AO3 field → FicHub param → Status (works/partial/inert)

### Appendix C: CSS Token Reference
- All `--archive-*` variables with light/dark values
- Font stack, border radius, spacing conventions
