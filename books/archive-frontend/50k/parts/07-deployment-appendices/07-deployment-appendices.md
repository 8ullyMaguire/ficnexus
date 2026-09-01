# Part 7 — Building, Deploying & Reference

👋 Welcome to the final part of this book! By now you've explored every piece of the Archive Frontend — from the two-row header down to the last tag chip in the search form. In this part, we're going to cover what happens after you write all that beautiful code: **how it gets built, how it gets served, and how it actually ends up in front of millions of readers.**

We'll also wrap up with some big-picture thinking about *why* we made the choices we did, where the project is heading next, and give you two handy appendices — a complete file reference and an API field mapping table — that you can keep open while you work.

Let's finish strong! 💪

---

## Chapter 29: Building & Deploying

In most web projects, the frontend is a living, breathing Node.js server — it listens for connections, manages sessions, maybe even does client-side rendering with JavaScript frameworks running at runtime. FicHub's archive frontend is… different. It's **entirely static**.

That means everything you build — every Svelte component, every TypeScript module, every CSS custom property — gets baked into plain old HTML, CSS, and JavaScript files. No server needed. Just files.

### The Build Command

To create these files, you run one command:

```bash
npm run build
```

This triggers Vite (the bundler powering your SvelteKit project) to do its magic: compile your `.svelte` files, bundle your TypeScript, optimize your CSS, generate sourcemaps, and spit out a clean `build/` directory containing everything the browser needs.

Here's what matters most: **Vite hashes every asset filename**. That means instead of producing files like `index.css`, you get something like `index.a1b2c3d4.css`. Every build produces unique hash values, so when new code goes live, the filenames change too. This isn't just cosmetic — it's a clever caching strategy we'll explore later.

### Where the Files Go

On FicHub's ThinkCentre machine, the build output lands in:

```
/personal/documents/code/rust/fichub/frontend/build/
```

This path sits on an **NFS mount** (a network filesystem shared across machines), which means the Rust backend — running on the same ThinkCentre — can read these files without any special transfer step. They're already there.

💡 **Key Concept: NFS Mounts**
Think of an NFS mount like a shared Google Drive folder for your servers. Both the frontend build process and the Rust backend "see" the same directory, even if they're technically separate programs. No copying files back and forth — just drop them in and go.

### How the Rust Backend Serves Static Files

The backend is a Rust service built with Axum. It has a single job in the static-file world: read files from disk and hand them to the browser. This logic lives in `src/static.rs`:

```rust
use axum::routing::serve_static;
// ...
let state = AppState {
    frontend_dir: env::var("FRONTEND_DIR").expect("FRONTEND_DIR not set"),
    // ...other fields...
};
```

The `FRONTEND_DIR` environment variable (set in `.env`) points to wherever your build output lives. When a user navigates to `/search` or `/bookmarks`, the backend finds the matching file in that directory and streams it over HTTP. That's it.

There's no Node.js, no Express, no webpack dev server doing anything. Just Rust reading HTML files off disk. Simple, fast, reliable.

### Deployment Steps

Deploying a frontend change to FicHub looks something like this:

1. **Pull the latest code:**
   ```bash
   git pull origin main
   ```

2. **Rebuild:**
   ```bash
   npm run build
   ```

3. **Restart the backend (usually not needed):**
   For purely frontend changes, the Rust service doesn't need restarting — it reads fresh files from the `build/` directory on every request. You'd only restart `fichub.service` if you changed backend code or environment variables:
   ```bash
   sudo systemctl restart fichub.service
   ```

⚠️ **Watch Out — Don't Over-Restart!** Restarting `fichub.service` causes a brief moment of downtime for everyone. Only do it when necessary. Frontend-only changes are automatically picked up because the backend serves files dynamically from disk — no reload required.

### 🔧 Try It Yourself: Build Locally

Want to see this in action? Open a terminal inside the frontend directory and run:

```bash
cd /home/alvaro/code/rust/fichub/frontend
npm run build
ls -la build/
```

You should see a collection of hashed files — `index.html`, `app.[hash].js`, `index.[hash].css`, and more. These are your production-ready files, ready to be deployed to the ThinkCentre.

### Cache Busting with Hashes

Remember that Vite hash thing I mentioned earlier (`index.a1b2c3d4.css`)? That's your **cache-busting superpower**.

Here's the problem it solves: imagine you fix a bug in the archive footer and push a new build. Your users visit `/` and their browser loads the new `index.html`, which references `index.f5e6d7c8.css`. Their browser thinks: "I already have a CSS file called index.css!" — but wait, the *filename is different*. Since it's a brand-new URL that never existed before, the browser fetches it from scratch. Old cached versions? Physically gone from the server. The old URL returns a 404. Clean break.

Meanwhile, unchanged assets like vendor bundles stay hashed the same way across builds (because their content didn't change), so the browser happily reuses its cache for those. Smart, right?

To make this work extra reliably, every file is served with:

```
Cache-Control: max-age=31536000
```

That's **one year** in seconds. The browser says "if this filename exists, it will always exist and always look exactly the same." Which, thanks to hashing, is true!

💡 **Key Concept: Immutable Caching**
Hashed filenames + long TTLs = blazing-fast repeat visits. Once the browser caches your files, it never even asks the server again unless the URL changes. This is why modern CDNs serve static assets so efficiently.

### Common Pitfall: Stale Browser Cache

Sometimes, even with perfect deployment, a user's browser still shows old content. Why? Because browsers aggressively cache things. If you've pushed a new build but a user sees the old version, tell them to do a **hard refresh**:

- **Chrome/Firefox:** `Ctrl+Shift+R` (or `Cmd+Shift+R` on Mac)
- **Safari:** `Cmd+Option+E` then reload
- Or hold `Ctrl` while clicking the reload button

If that doesn't work, clearing the cache entirely usually fixes it. This isn't a bug in FicHub — it's just how the web works. Even giants like Google battle this exact issue daily.

### Build Warnings: Ignore the Noise

When you run `npm run build`, you might see some warnings pop up:

- **Unused CSS selectors** — Pre-existing styles that aren't referenced by any component. Harmless, cosmetic-only.
- **Svelte `state_referenced_locally`** — Svelte's suggestion that certain `$state()` variables could potentially be removed. Also non-fatal.

These warnings are tracked as low-priority housekeeping items. They won't break anything. Fixing them requires careful review to make sure removing an "unused" selector doesn't silently break styling. Don't let them distract you from writing new features!

---

## Chapter 30: Architecture Decisions & Future Work

Congratulations on making it this far through 30 chapters! Before we close the book, let's take a breather and think about the *big picture*. Why did we build the archive interface the way we did? What problems were we trying to solve? And where are we headed next?

### Why Path B: Two Shells in One App?

When the archive redesign started, the team considered three approaches:

| Approach | Description | Trade-off |
|----------|-------------|-----------|
| **Path A: Separate Apps** | Ship the archive UI as an entirely different application | Doubles your deploy pipeline, doubles your database connections, doubles your maintenance |
| **Path B: Layout Switch** ✅ | ONE app that swaps between layouts | One codebase, one deploy, one pool — two experiences |
| **Route Split** | Put archive routes under `/archive/*` and modern routes under `/` | Fragmented sharing of components, confusing URL patterns |

We chose **Path B** — a layout-level switch inside a single application — and here's why it was the winning choice:

1. **One codebase**: All components share utilities, API clients, auth stores, and type definitions. No sync headaches.
2. **One deploy pipeline**: One `git push` deploys both interfaces. No coordinating two separate CI/CD flows.
3. **One database connection pool**: The Rust backend talks to Postgres once. Both UI modes drink from the same well.
4. **Two complete experiences**: Users don't feel like they're using a "lite" archive mode. Every feature available in modern mode is equally available in archive mode.

It's like having two bedrooms in one house, connected by the same hallway. You pick whichever room you want to hang out in tonight, but the kitchen and bathroom serve both.

### Why Is Archive the Default?

Modern mode got built first — it was our experimental playground. Yet when it came time to ship, **archive became the default** for every user. Here's why:

**Familiarity wins.** Most fanfiction readers learned the web through AO3 (Archive of Our Own). Its layout, its color scheme, its way of organizing tags — that's muscle memory. A reader visiting FicHub shouldn't have to learn a new interface before they can enjoy the content. By making archive the default, we reduce friction and meet readers where they are.

The modern shell stays fully accessible though — it's one click away in the settings page, footer links, or dropdown menu. But the homepage greets you with the familiar, comfortable archive design.

### The One Divergence from AO3: AND vs OR

Almost everything in the archive UI mimics AO3 exactly — until you try filtering by fandom.

At AO3, selecting multiple fandoms uses **OR logic**: show me stories tagged with "Marvel Cinematic Universe" OR "Star Wars." Pick either one, you'll get results.

At FicHub, selecting multiple fandoms uses **AND logic**: show me stories tagged with "Marvel Cinematic Universe" AND "Star Wars." Works must have *every single tag you select*.

This is a known difference. It's documented. It makes the filter tighter and more specific — which some users love and others find frustrating when they expected the AO3-style broadening behavior. We've marked it clearly in the UI tooltips so nobody is blindsided.

⚠️ **Watch Out for the AND Behavior:** If you're searching for stories in multiple fandoms and getting zero results, check whether AND logic might be the culprit. Try selecting fewer tags at once.

### The Parachute Honesty Principle: Inert Fields

Some fields on the archive search form are **inert** — they're displayed but don't actually filter results. The crossovers checkbox, the hits counter, the language dropdown, the rating badges. They look interactive, but clicking or changing them does nothing.

Why show them at all? We call it the **parachute honesty principle**.

When users land on the search page, they scan it quickly. If they don't see "Crossovers" or "Hits" or "Language," they assume these filters don't exist at FicHub — maybe the backend can't handle them, maybe the site is incomplete. By showing disabled options with explanatory tooltips, we communicate: *"Yes, we know you want to filter by language. Yes, we want that too. We're building it — come back soon!"*

Users see what exists, understand the backend's limits transparently, and trust us that the missing pieces are intentional, not accidental.

💡 **Key Concept: Transparency Builds Trust**
Telling users what you *can't* do yet is often better than pretending it doesn't exist. People forgive unfinished features; they punish hidden ones.

### The Secret Superpower: `?ui=` URL Parameters

Try visiting this URL in your browser:

```
https://fichub.example.com/search?q=harry?ui=archive
```

Notice the `?ui=archive` at the end? That tiny parameter overrides anyone's personal preference and forces the archive view. Without touching any settings, anyone can render *any* page in archive mode.

This is incredibly useful for:

- **Documentation**: Link to a page in the format you're describing
- **Tutorials**: Show screenshots that match the current reader's expectations
- **A/B testing**: Compare how features behave in each mode side by side
- **User switching**: Click the link, try the other UI, go back — no settings change required

It's a small addition with outsized usefulness. Under the hood, `applyUiParam()` checks for this query parameter early in the page lifecycle and short-circuits the normal preference lookup.

### Roadmap: What's Next?

The archive frontend is feature-complete, but that doesn't mean the work stops here. Here's what's on the horizon:

1. **Restyle the Reader**: Right now, the fic reading page (where you actually read stories) looks completely different from the archive UI. A planned restyle would bring the reader page into the archive aesthetic — maroon headings, Georgia serif fonts, compact spacing. Readers scroll through dozens of pages per story, so consistency matters.

2. **Admin Pages**: The admin dashboard has a modern-shell treatment that doesn't quite match the archive elsewhere. Bringing admin pages into archive mode would unify the experience for moderators.

3. **More Archive Conditional Coverage**: Several pages still lack archive implementations. Any route page with `{#if uiMode === 'archive'}` still stubbed out as TODO is fair game.

4. **Potential Third Theme: "Retro"**: Some users have suggested a pre-AO3 "dark web" aesthetic — deep blacks, neon accents, table-based layouts. It'd be a fun nostalgic trip and a great showcase exercise. Whether it ships depends on community interest.

Performance-wise, the system is solid. Being prerendered client-side SvelteKit means **zero framework overhead** — no hydration waterfall, no React reconciliation, no Angular zones. Archive components are lightweight (~17K total lines of code across all files), and skeleton loading prevents FOUC (Flash of Unstyled Content) during data fetching. The result: fast loads, smooth interactions, happy readers.

💡 **Key Concept: Zero Framework Overhead**
Because pages are prerendered at build time, the browser receives fully-formed HTML. There's no JavaScript framework bootstrapping itself on the client side. What you see is what you get — no waiting, no flickering, no waterfall. The browser renders instantly.

---

## Appendix A: File Reference

Every file in the archive frontend, listed with its line count and purpose. Use this as a quick lookup when you're trying to figure out "which file handles X?"

### Directory Structure

All archive code lives under:

```
frontend/src/lib/ui/archive/
```

### Complete File List

| File | Lines | Purpose |
|------|-------|---------|
| `ArchiveLayout.svelte` | 63 | Root wrapper — wraps header, main, footer. Re-hosts CommandPalette, HelpModal. Initializes theme/i18n/auth on mount. Uses Svelte 5 `{@render children()}` slot pattern. |
| `ArchiveHeader.svelte` | ~250 | Two-row AO3-parity header. Top row: logo + avatar + Post/Log Out. Red navbar: Fandoms, Browse, Search ▾, About ▾ (native `<details>` dropdowns). Keyboard-accessible. No emojis anywhere. |
| `ArchiveFooter.svelte` | ~130 | Four-column grid footer (Customize, About, Contact, Development). Accent-line background, white text. Responsive collapses to 2 cols at 600px, 1 col at 380px. Contains "Switch to modern interface" button with `setPref` + `location.reload()` pattern. |
| `ArchiveNavLink.svelte` | 49 | Nav link with active-state detection via `$page.url`. Uses `$derived` for reactive computation. 2px accent underline on active state. |
| `ArchiveButton.svelte` | 58 | Flat button component. Variants: primary (maroon) vs secondary (gray). AO3 aesthetic: no shadows, no gradients, square corners. |
| `ArchiveWorkSearchForm.svelte` | 853 | **LARGEST COMPONENT**. Two-mode search form with autocomplete chips, tag suggestions, sort controls, and pagination. Orchestrates searchForm.ts functions. |
| `ArchiveListPage.svelte` | 216 | Reusable list wrapper. Props: title, items, loading, error, subtitle. Includes loading skeleton animation (5 pulsing blurbs) and empty/error states. Used by bookmarks, authors, notifications pages. |
| `ArchiveHome.svelte` | 470 | Homepage component. Featured sections, trending works, category browsing. Uses `ArchiveListPage` internally. |
| `WorkBlurb.svelte` | 208 | Core fic card component. Renders: rating badge, title (maroon link), byline, TagSoup, snippet, StatsLine, action buttons (Download, Read, Bookmark, Kudos). Anonymous users see login prompt. |
| `TagSoup.svelte` | 117 | Labeled tag rows. Bold gray label + comma-separated maroon tag links. Tags truncate at 6 for Additional Tags category with "+N more" details disclosure. |
| `StatsLine.svelte` | 52 | Single-line stats display. Format: "Words: 1,204,116 · Chapters: 109/109 · Kudos: 412 · Updated: 3d ago". Muted color, middot separators. Simplest content component. |
| `rating.ts` | 118 | Pure TypeScript helpers. Rating resolution, display labels, normalization. Zero Svelte dependency — importable anywhere. |
| `searchForm.ts` | 480 | **SECOND LARGEST FILE**. Pure module. Functions: `parseRange()`, `buildSearchQuery()`, `formStateToUrl()`, `urlToFormState()`, `mergeChipIntoForm()`, `filterTagsForSuggestions()`. 47 unit tests in companion test file. |
| `searchForm.test.ts` | — | Unit tests covering searchForm.ts pure functions. Tests form-state round-trips, URL serialization, chip merging, range parsing. |

### Import Dependency Graph

```
Root Layout (+layout.svelte)
    │
    └─→ ArchiveLayout.svelte
           │
           ├─→ ArchiveHeader.svelte
           │      └─→ ArchiveNavLink.svelte
           │
           ├─→ ArchiveFooter.svelte
           │      └─→ ArchiveNavLink.svelte
           │
           ├─→ Content (child routes)
           │      │
           │      ├─→ Search Page → ArchiveWorkSearchForm → searchForm.ts → Search API
           │      │
           │      ├─→ ArchiveListPage.svelte → WorkBlurb → TagSoup + StatsLine
           │      │                                   ↓
           │      │                             WorkBlurb → rating.ts
           │      │
           │      └─→ ArchiveHome.svelte → ArchiveListPage → WorkBlurb
           │
           └─→ prefs.ts (getPref called throughout)
                  ↑
              All routes → getPref(prefs.ts) → uiMode check
```

💡 **Quick Tip**: When you need to understand a file's role, check how many other files import it. `prefs.ts`, `searchForm.ts`, and `ArchiveLayout.svelte` sit at the top of the graph — almost everything depends on them. The leaf nodes (`TagSoup`, `StatsLine`, `ArchiveNavLink`) are self-contained components with zero imports.

---

## Appendix B: API Field Mapping Table

This table maps every form field on the archive search page to its corresponding backend API parameter. Use this when building new filters, debugging unexpected results, or implementing inert-to-active transitions.

| Form Field | API Param | Status | Notes |
|------------|-----------|--------|-------|
| `title` | `title` | ✅ Supported | Exact-match title filter |
| `author` | `author` | ✅ Supported | Filters by author name |
| `chapters_num` | `chapters` | ✅ Supported | Number range (min/max) |
| `word_count` | `min_word_count` / `max_word_count` | ✅ Supported | Parsed via `parseRange()` syntax |
| `fandom` | `include_tags=fandom:name` | ✅ Supported | Added to `include_tags` array |
| `character` | `include_tags=character:name` | ✅ Supported | Same tag-pattern as fandom |
| `relationship` | `include_tags=relationship:name` | ✅ Supported | Same tag-pattern as fandom |
| `additional_tag` | `include_tags=additional_tag:name` | ✅ Supported | Same tag-pattern as fandom |
| `warning` | `include_tags=warning:name` | ✅ Supported | Same tag-pattern as fandom |
| `category` | `include_tags=category:name` | ✅ Supported | Same tag-pattern as fandom |
| `kudos` | `max_kudos` | ⚠️ Partial | Upper bound only — no minimum kudos support |
| `comments` | `min_comments` | ✅ Supported | Minimum comment count |
| `bookmarks` | `min_bookmarks` | ✅ Supported | Minimum bookmark count |
| `rating` | `include_tags=rating:*` | 🪂 Inert | Best-effort resolved internally; no-op at API level |
| `crossovers` | N/A | 🪂 Inert | Shown but ignored by backend |
| `hits` | N/A | 🪂 Inert | Display placeholder only |
| `language` | N/A | 🪂 Inert | Placeholder for future implementation |
| `sort_value` | `sort_column` / `sort_descending` | ✅ Supported | Combined into single sort param |

### Legend

- ✅ **Supported** — fully functional, actively filters results
- ⚠️ **Partial** — implemented but with limitations (check notes column)
- 🪂 **Inert** — visible in UI but has no effect; shown for transparency (parachute honesty)

---

*You did it.* Thirty chapters of svelte runes, CSS custom properties, search forms, tag chips, layout switches, and static file serving. If you can reason about how all of this fits together — how a user types "dragon" into a search box and watches results populate across a fully styled archive interface — you've earned your place in the FicHub codebase.

Keep building, keep learning, and remember: the best code is the kind someone else can pick up without calling you at 2 AM. Happy coding! ✨
