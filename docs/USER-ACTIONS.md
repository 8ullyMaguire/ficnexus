# USER-ACTIONS.md

Runbook of user-actionable tasks the user has asked for but haven't been completed yet.
Each item is a self-contained task ready for a subagent.

> **Verification**: A full step-by-step checklist is in
> `docs/VERIFICATION-CHECKLIST.md`. Feed sections of that file (one area at a
> time) to another chatbot to ask about issues in isolated chunks.

## Backlog (pending)

1. **Add loading skeleton states to forum pages** — Currently shows plain
   "Loading..." text. AO3-style animated skeleton rows expected.
2. **Verify `body.skin-zerafina` scoped CSS** — The TopicThread.svelte uses
   `:global(.hidden-archive)` (now also in `zerafina-skin.css`). Confirm the
   modern mode `page-head` actually hides in archive mode when the component
   is loaded inside the TopicThread context.
3. **Forum tag display parity** — AO3 WorkBlurb uses `<ul class="tags commas">`
   (comma-joined tags) vs FicHub's `TagSoup` (labeled `<dt>`/`<dd>` rows).
   Decide whether to switch WorkBlurb tags or keep TagSoup for archive blurbs.
4. **`dl.work.meta` metadata block** — ArchiveWork.svelte should show tags by
   category, Language, Series, and Stats (Words/Chapters/Comments/Kudos/Bookmarks/Hits)
   like AO3, not the current `StatsLine` single-line format.
5. **Chapter navigation header** — Missing on work detail pages in archive mode.
6. **Word Count / Kudos sort** — Disabled in search dropdown (not supported by backend).
7. **`min_bookmarks`** — Listed as supported in INTERFACE-STYLES.md but may not be wired in backend.

## Recently completed (for reference)

- ✅ Forum pages rewritten with AO3 archive style
- ✅ Forum topic URLs fixed (`/forum/board/{slug}.{id}` format)
- ✅ TopicThread page-header hidden in archive mode
- ✅ Search dropdown fixed (Work/People/Tag/Bookmark search)
- ✅ `/search/tags` route created with AO3-style tag search form (+ `page.test.ts`, 5 tests)
- ✅ `/people` route exists with AO3-style people search
- ✅ Tags `switchMode` bug fixed (all 6 types)
- ✅ Search page modern mode hidden in archive
- ✅ Docs page rewritten
- ✅ Stats page fixed (broken hrefs, missing wrapper, missing `{/if}`)
- ✅ Roadmap feature ranking toggle button
- ✅ Broken hrefs fixed across ask, fandom, stats, search, authors pages
- ✅ Emoji removed from UI text across all pages
- ✅ EPUB codebase documentation created (`docs/src/epub-codebase.md` + `.epub`)
- ✅ All 29 routes return HTTP 200
- ✅ `.hidden-archive` CSS rule added to `zerafina-skin.css`
- ✅ **Follow Exclusions** — per-follow exclusion of works/series/fandoms from author-follow updates feed (`follow_exclusions` table, `/api/follows/*/exclusions`, feed anti-join, `Exclusions.svelte`, i18n ×6, branch 89822de)
- ✅ **Search v2 backend** — structured/fielded operators (`@field`, `~` fuzzy, wildcard, `with`/`not with`, `romship`/`platship` by polarity, ranges/comparisons + `50k` shorthand), `tags.rel_polarity` + `fic_tags.role_confidence` + `works` engagement counters (migrations 003–005)
- ✅ **Search v2 UI** — three-tier Simple/Guided/Power search on `/search` (interpreted-chips, live facet builder + match count, power textarea; commit 42024c2)
- ✅ **Saved searches + daily alerts** — save list/run/delete/alert-toggle, nightly watcher re-run + match diff, per-search Atom feed

<details><summary>Collection submissions + curator approvals</summary>
- ✅ **Unified curator Approvals queue** — `/curator/approvals` aggregating pending collection item requests + curator fix/metadata proposals + comment triage with type/status filter; **community voting** on collection submissions via `collection_submission_votes` (migration 006, commit 57d54d5)
</details>

## Recently completed — forum
- ✅ **Forum "New category" button fixed** — `forum.close` i18n key added ×6, `.new-category` form no longer hidden in archive mode, archive-form styling + ArchiveButton submit (commit b843c9b)
