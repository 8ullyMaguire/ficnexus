# Feature Specification: Site Statistics Page

**Feature Branch**: `004-site-stats`

**Created**: 2026-08-24

**Status**: Draft

**Input**: User description: "this doesn't seem to be implemented https://fichub.polarisocial.xyz/stats 'Site statistics are not yet available.'"

## User Scenarios & Testing *(mandatory)*

<!--
  User stories are PRIORITIZED user journeys. Each independently testable —
  implementing just ONE yields a viable MVP slice.
-->

### User Story 1 - View public site-wide statistics (Priority: P1)

A community member or prospective reader visits the `/stats` page to see how
active the archive is (works, users, kudos/bookmarks/reviews per period) so
they can judge whether the site has enough content to be worth joining.

**Why this priority**: Currently `/stats` renders only "Site statistics are not
yet available." — anonymous visitors get nothing and leave. AO3 and OTW
archives prominently surface trust metrics; hiding them breaks the AO3-migration
parity contract for this route.

**Independent Test**: As anonymous, GET `/stats` and `/api/site/stats`, verify
the page returns the site-wide statistics section (not the placeholder), showing
period rows and aggregate totals, in both archive and modern skins.

**Acceptance Scenarios**:

1. **Given** anonymous visitor on `/stats`, **When** the public API returns site
   statistics, **Then** the page renders "Site-wide statistics" with a rows
   table (Period, Views, Kudos, Bookmarks, Works read, Reviews) and a blockquote
   showing aggregate totals (Total works, Active users).
2. **Given** statistics are unavailable (API down / empty), **When** the page
   loads, **Then** it shows a graceful "Site statistics are not yet available."
   placeholder and does not crash.
3. **Given** the page is loaded in either skin, **When** switching
   uiMode (archive / modern), **Then** the statistics content renders in the
   skin-appropriate layout (archive typography + `archive-*` classes, modern
   `.stats-*` cards) without content loss.

---

### User Story 2 - View personal reading statistics (Priority: P2)

A logged-in reader opens `/stats` to review their own reading totals and login
streak so they can track progress and compare against the community.

**Why this priority**: Logged-in users expect personal totals; the page fetches
`/api/reading/analytics` and reading stats but only displays them when present.
This is the authenticated half of the route and the hook for gamification
(streaks) used elsewhere.

**Independent Test**: As logged-in user, GET `/stats`, verify "Your reading"
section shows words-read, works-read, and current/longest streak numbers.

**Acceptance Scenarios**:

1. **Given** authenticated user with reading history, **When** they open
   `/stats`, **Then** "Your reading" block quotes Words read, Works read, and
   login streak values derived from `/api/reading/analytics` + `/api/reading-stats`.
2. **Given** authenticated user with no reading yet, **When** opening `/stats`,
   **Then** "Your reading" shows a muted prompt to start reading and the site
   statistics section still renders.
3. **Given** unauthenticated visitor, **When** opening `/stats`, **Then** a
   "Log in to see your personal reading totals" prompt appears and anonymous
   site-wide statistics are still visible.

---

### User Story 3 - Discover statistics from any page via the footer (Priority: P2)

A reader browsing any work, chapter, or forum page wants to find site statistics
without navigating to a known URL, so they trust the archive's activity at a glance.

**Why this priority**: The `/stats` page is useless if readers can't discover it.
AO3/OEM archives surface community stats from the footer; FicHub's archive parity
contract includes this discoverability.

**Independent Test**: As anonymous, visit any content page, verify the footer
contains a "Statistics" link pointing to `/stats`.

**Acceptance Scenarios**:

1. **Given** any authenticated or anonymous page with a footer, **When** the
   footer renders, **Then** it contains a "Statistics" link navigating to
   `/stats` in both archive and modern skins.
2. **Given** a logged-in curator on a work page, **When** opening the
   navbar dropdown, **Then** an "About" entry is present and links to a public
   `/about` page describing FicHub/OEM.
3. **Given** an anonymous reader clicking the navbar dropdown, **Then** the
   same "About" entry is visible and accessible without authentication.

---

### Edge Cases

- What happens when **one period is missing a field** (e.g. `total_users` null)?
  Page renders `0` (zero) via null-coalescing and never throws.
- How does the system handle the **API returning `err !== 0`** or a non-`periods`
  payload? The page degrades to the "not yet available" placeholder.
- What if `siteStats.periods` is an **empty array**? Same placeholder path
  (guarded by `periods.length > 0`), preserving the personal-reading section.
- What happens when the **logged-in fetch for personal analytics fails**?
  `/reading/analytics` is fetched inside a Promise.all with `.catch(() => {})`;
  failure leaves personal stats null (the "start reading" prompt) but never
  breaks the page or site statistics.
- What if a **period label has no i18n key** (e.g. custom/edge period)?
  `fmtPeriod` falls back to the raw label so the table never renders blank.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST return site-wide statistics for periods (1d, 7d, 14d,
  30d, 90d, 365d, all) with views, kudos, bookmarks, works-read, and reviews
  counts via `GET /api/site/stats`, so the `/stats` page can render them.
- **FR-002**: System MUST expose aggregate totals (Total works, Active users)
  for the all-time period alongside per-period rows.
- **FR-003**: System MUST surface the anonymous visitor's public statistics on
  `/stats` without requiring authentication.
- **FR-004**: System MUST render personal reading totals (words read, works
  read, login streak) for authenticated users from
  `/api/reading/analytics` and reading-stats endpoints.
- **FR-005**: System MUST render the personal-reading section and "Log in to see
  personal totals" prompt appropriately based on auth state.
- **FR-006**: System MUST render statistics in archive skin (AO3-style
  typography, `archive-*` classes, `dl.stats` definition list for aggregates)
  and in modern skin (`.stats-*` cards + `.stats-grid`).
- **FR-007**: System MUST degrade gracefully — when `/api/site/stats` is absent,
  `err !== 0`, or returns an empty `periods` array, the page must show
  "Site statistics are not yet available." and never crash.
- **FR-008**: System MUST tolerate missing/null fields on period objects
  (render `0`, never undefined/NaN in the DOM).
- **FR-009**: System MUST expose a link to `/stats` in the site footer so readers
  can discover site statistics from any page.
- **FR-010**: System MUST expose an "About" entry in the navbar dropdown
  (Community / Browse / Curator / Admin / About) that links to a public page
  documenting the project, mirroring AO3's "About" top-level nav affordance.

### Key Entities *(data involved)*

- **PeriodStat**: a per-time-window aggregate (`period`, `work_views`, `kudos`,
  `bookmarks`, `works_read`, `reviews`, plus aggregate `total_works` /
  `total_users` on the all-time period). Represents community activity volume
  per period.
- **PersonalStats**: an authenticated reader's running totals (`total_words_read`,
  `total_works_read`, `login_streak`) used only for the "Your reading" block.
- **RecentRead**: an authenticated reader's recently-read works (`work_id`,
  `title`, `words_read`, `read_count`, `last_read_at`) shown as a list beneath
  the totals.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Anonymous `/stats` returns a page containing the
  "Site-wide statistics" heading and at least one period row within
  2 seconds of first render for 95% of page loads.
- **SC-002**: `/api/site/stats` returns `err:0` with a non-empty `periods` array
  containing at least 3 periods (7d, 30d, all) on the live production instance.
- **SC-003**: 100% of period-row cells render a numeric value (never `NaN` or
  raw `undefined`) when the API returns complete data.
- **SC-004**: Graceful-degradation path: when `/api/site/stats` returns
  `err != 0` or empty periods, the page still renders a 200 OK in both skins.
- **SC-005**: Authenticated `/stats` renders a "Your reading" section with
  words-read and works-read numbers within 2 seconds for 95% of page loads.
- **SC-006**: AO3-migration parity: the `/stats` route returns content (not the
  "not yet available" placeholder) for anonymous visitors, matching the
  archive-style statistics presence of OTW archives.

## Assumptions

- The backend already has the data to populate site statistics (reading history,
  kudos, bookmarks, reviews are tracked for other routes); FR-002/FR-003 require
  only an aggregation endpoint, not new tracking.
- Personal stats reuse the existing `/api/reading/analytics` and reading-stats
  endpoints already wired into the page; the gap is the *public* endpoint.
- "Active users" is approximated by a 30-day active count from reading history
  (reasonable default; exact definition is an admin concern).
- Statistics are read-only and cache-friendly (no per-user computation on the
  anonymous path), so a cache-control-friendly response is acceptable.

## Clarifications

### Session 2026-08-24

- Q: Should `/admin/stats` remain in scope for this feature, or be out of scope? → A: Option A — public `/stats` only; `/admin/stats` is pre-existing (role-guarded, working on production), explicitly out of scope, not modified. FR-003 scope confirmed: anonymous visitor only.

## Out of Scope

- `/admin/stats` and `/api/admin/stats` — already implemented behind `role < 10` guard with daily rollup + live totals; not touched by this feature.
- Admin-only statistics tooling (bot scoring, moderation analytics, export logs) — stays under the existing admin surface.
