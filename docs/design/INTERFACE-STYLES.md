# Interface Styles

FicHub offers two distinct interface modes to suit different reading preferences:

## Archive Mode (AO3-style)
- **Default for new users**
- Classic archive-style layout inspired by Archive of Our Own
- Features: sidebar navigation, work blurbs with metadata, tag soup display
- Compact, information-dense layout ideal for browsing large collections
- Traditional archive navigation patterns

## Modern Mode
- Clean, contemporary SvelteKit-based interface
- Card-based layouts with generous whitespace
- Designed for modern web users who prefer a more spacious, visual design
- Enhanced hover effects and transitions

## Switching Between Modes

### Settings Page
1. Navigate to **Settings** from the top navigation
2. Under **Interface Style**, select either **Archive** or **Modern**
3. Changes apply immediately

### Footer Toggle
- Look for the "Interface style" link in the site footer
- Click to toggle between Archive and Modern modes

### User Dropdown Menu
- Click your username/avatar in the top right
- Select **Interface Style** from the dropdown
- Choose your preferred mode

### URL Parameter
Add `?ui=archive` or `?ui=modern` to any URL to switch modes:
```
https://fichub.net/?ui=archive
https://fichub.net/search?q=harry+potter&ui=modern
```

## Search Form Field Matrix

The archive mode search form (`ArchiveWorkSearchForm`) maps AO3 Work Search
fields to FicHub API parameters. Fields marked **inert** are rendered in the
form but produce no filter effect — a footnote at the bottom of the form
explains which fields are not yet supported.

### Work Info

| AO3 Field | Widget | FicHub API Param | Status |
|---|---|---|---|
| Any Field | text input | `q` (free-text, may include field prefixes) | **works** |
| Title | text input | `q` + `title:(...)` via field-prefix parser | **works** |
| Creator | text input | `q` + `author:(...)` | **works** |
| Date | text input (YYYY-MM-DD) | `date_from` | **works** (one-sided) |
| Completion status | radio group (All / Complete / In Progress) | `complete=true` / `complete=false` / omitted | **works** |
| Crossovers | radio group (All / Crossovers / No Crossovers) | — | **inert** |
| Single Chapter | checkbox | `max_chapters=1` | **works** |
| Word Count | text input (AO3 range syntax) | `min_words` / `max_words` (parsed via `parseRange`) | **works** |
| Language | select (disabled) | — | **inert** |

### Work Tags

| AO3 Field | Widget | FicHub API Param | Status |
|---|---|---|---|
| Fandoms | text input | `q` + `1:<name>` (tag type 1 prefix) | **works** |
| Rating | select (Not Rated / General / Teen / Mature / Explicit) | `rating` | **inert** (best-effort) |
| Warnings | checkbox list (6 AO3 warning labels) | `q` + `5:<name>` per selected (tag type 5) | **works** |
| Categories | checkbox list (F/F, F/M, Gen, M/M, Multi, Other) | `q` + `6:<name>` per selected (tag type 6) | **works** |
| Characters | text input | `q` + `2:<name>` (tag type 2 prefix) | **works** |
| Relationships | text input | `q` + `3:<name>` (tag type 3 prefix) | **works** |
| Additional Tags | text input | `q` + `4:<name>` (tag type 4 prefix) | **works** |
| Other tags to include | text input | `tag_ids` (comma-separated numeric IDs) | **works** |

**Within-category AND vs AO3's OR divergence:** AO3 applies OR within
checkbox groups (e.g. selecting both F/F and M/M shows works with *either*).
FicHub v1 applies AND — selecting multiple warnings/categories requires
*all* to be present. This is a known limitation; fixing it requires
query-builder changes and is a follow-up task, not blocking.

### Work Stats

| AO3 Field | Widget | FicHub API Param | Status |
|---|---|---|---|
| Hits | range text input | — | **inert** (no hit counter) |
| Kudos | range text input | `min_kudos` (max ignored if unsupported) | **partial** |
| Comments | range text input | `min_comments` (max ignored if unsupported) | **partial** |
| Bookmarks | range text input | `min_bookmarks` (if backend added) | **partial** |

Range syntax supported: `1000`, `>1000`, `<5000`, `1000-5000`, `≥1000`, `≤5000`.
The `parseRange()` function in `searchForm.ts` handles all these formats.

### Sort Options

| AO3 Sort Option | Value | Supported |
|---|---|---|
| Best Match | (default) | **yes** |
| Creator | `creators` | **yes** |
| Title | `title_to_sort_on` | **yes** |
| Date Posted | `created_at` | **yes** |
| Date Updated | `updated_at` | **yes** |
| Word Count | `word_count` | **no** (disabled in dropdown) |
| Kudos | `kudos_count` | **no** (disabled in dropdown) |

Unsupported sort options are rendered in the dropdown but marked `disabled`
so the user sees them for parity but cannot select them — behavior is
honest where it is visible. Sort direction (Descending/Ascending) is
disabled when the active sort option is unsupported.

### Supported Parameter Matrix (canonical)

The frontend uses a `SUPPORTED` matrix object (`searchForm.ts`) to decide
which params to send to the API and which to skip with a footnote note:

| Param | Supported |
|---|---|
| `q` | yes |
| `sort` | yes |
| `min_words` | yes |
| `max_words` | yes |
| `complete` | yes |
| `max_chapters` | yes |
| `date_from` | yes |
| `min_kudos` | yes |
| `min_comments` | yes |
| `include_tags` | yes |
| `exclude_tags` | yes |
| `tag_ids` | yes |
| `no_warnings` | yes |
| `max_kudos` | **no** |
| `min_bookmarks` | **no** |
| `max_bookmarks` | **no** |
| `rating` | **no** |
| `language` | **no** |
| `dir` | **no** |

### Chips Feature

The search form displays up to 8 commonly added tags fetched from
`GET /api/search/suggest`. The chips row appears under the "Work Search"
heading in page mode and at the top of the results view.

- Each chip shows the tag name and carries a `tag_type_id` (1=fandom,
  2=character, 3=relationship, 4=freeform, 5=warning, 6=category)
- Clicking a chip merges the tag into the matching typed form field
  (via `mergeChipIntoForm`) and immediately submits the search
- For authenticated users, the endpoint returns personalized suggestions
  with `reason: "for_you"` based on bookmarked tag overlap; anonymous
  users see `reason: "popular"`
- The chip feature is the single intentional divergence from AO3's search
  page — AO3 has no equivalent "commonly added" row

### Two-Mode Form Component

`ArchiveWorkSearchForm` operates in two modes from a single component:

- **`page` mode**: Full form on its own page (Work Search page with
  fieldsets, fieldsets, chip row, both Search buttons, and the inert-field
  footnote)
- **`sidebar` mode**: Collapsible `<details>` sections in the right-hand
  filter sidebar on the results page. Sections are collapsed by default
  (except Sort). Topped by Sort by + Sort direction + a "Sort and Filter"
  submit, with a "Clear Filters" link at the bottom.

## Technical Details
- Preference stored in browser localStorage (`fichub_prefs_v1`)
- Default: `archive` for new users
- Root layout applies the `?ui=` parameter on every navigation
- CSS and components are conditionally rendered based on `uiMode` preference
