# Session Summary — 2026-08-29

## FicHub → FicNexus Rename (epub books + user-facing strings)

### Backend (Rust)
- `src/export/kepub.rs`: epub dc:identifier `fichub-{url_id}` → `ficnexus-{url_id}`
- `src/db/queries.rs` (bulk): all `urn:fichub:*` URN identifiers → `urn:ficnexus:*`
- `src/routes/opds/*.rs`, `src/routes/rss/*.rs`, `src/routes/saved_search.rs`: OPDS/RSS feed titles, atom entry titles, saved search feed IDs renamed
- `src/routes/subsystems.rs`: `SITE_NAME_DEFAULT` "FicHub" → "FicNexus"
- `src/routes/kindle.rs`: email footer "Sent from FicHub." → "Sent from FicNexus."
- `src/services/mailer.rs`: email footer + test assertions updated
- `src/routes/user_export.rs`: download filename `fichub-user-data-*.zip` → `ficnexus-user-data-*.zip`
- `src/routes/social.rs`: bookmarks CSV filename `fichub-bookmarks.csv` → `ficnexus-bookmarks.csv`
- `src/main.rs`: startup log "Starting fichub-rs server" → "Starting ficnexus server"
- `src/routes/api_docs.rs`: API name "fichub-rs API" → "ficnexus API"
- `src/routes/work_proposals.rs`: comment "FicHub" → "FicNexus" (already was via sed)
- `src/search/ask.rs`, `src/search/parser.rs`: comments updated

### Frontend (SvelteKit)
- `src/lib/i18n/dictionaries/*.ts` (6 locales): brandAria, bookmarkHint, roadmap subtitle/suggestTitle
- `src/lib/ui/archive/*.ts` (6 locales): same keys
- `src/lib/ui/archive/ArchiveHeader.svelte`: brand name "FicHub" → "FicNexus", aria-label "Search FicHub" → "Search FicNexus"
- `src/lib/ui/archive/ArchiveHome.svelte`: empty state text
- `src/lib/ui/archive/ArchiveFooter.svelte`: GitHub link updated
- `src/routes/+layout.svelte`: title "Work Proposals — FicNexus", footer brand, Liberapay title, download filename
- `src/routes/work-proposals/+page.svelte`: title (already FicNexus via sed)
- `src/routes/bookmarks/+page.svelte`: CSV download filename
- `src/lib/components/ThemeEditor.svelte`: theme filename
- `src/lib/pwa/strategies.ts`: test fixture URL (comment only)
- `src/lib/i18n/index.svelte.ts`: comment only
- `src/lib/themes/apply.ts`: localStorage key `fichub-theme` — NOT renamed (internal data key)

### Preserved (internal/structural — not user-facing)
- `fichub_prefs_v1` — localStorage key (changing breaks user prefs)
- `fichub_locale`, `fichub_dl_url` — localStorage/sessionStorage keys
- `fichub:reader:state:*`, `fichub:reader:html:*` — reader storage keys
- `fichub-theme` — localStorage theme key
- `fichub-net` — external site name (fichub.net, different service)
- Forgejo repo URL `opencommit.eu/MagicZhang/fichub` (repo not renamed on Forgejo)
- `fichub-rs` crate name in Cargo.toml
- `fichub_mailer_test_`, `fichub-body-cache`, `fichub-pow:` — internal Redis key prefixes
- `fichub.local` — PWA test hostname
- Doc route slug `/docs/what-is-fichub.html` (internal URL path)

### Bug Fix
- `src/db/mod.rs`: Added `FICNEXUS_SKIP_MIGRATIONS` env var check alongside `FICHUB_SKIP_MIGRATIONS`. The systemd unit sets `FICNEXUS_SKIP_MIGRATIONS=1` but the code only checked `FICHUB_SKIP_MIGRATIONS`, causing production startup crash with `VersionMismatch(12)` (DB at v69, binary tried to migrate).

### Tests
- Backend: 760 passed, 0 failed
- Frontend: 721 passed, 5 pre-existing unhandled rejections (read/[urlId]/page.test.ts)

### Deployment
- Backend released binary → `/tmp/ficnexus.tmp` → `/opt/ficnexus/ficnexus` (stop/start service)
- Frontend `npm run build` → `rsync --delete` to `/var/www/ficnexus/`
- Site verified: `https://ficnexus.polarisocial.xyz/health` → 200
- Static docs/manifest/sw.js updated on prod server via sed
- Commits: `ea597a6` (rename), `1258edc` (static docs), `bcac029` (env var fix)
- Pushed to Forgejo mirror

### Note
- `FRONTEND_DIR=/personal/documents/code/rust/ficnexus/frontend/build` in `.env` points to dead NFS path — should be updated to `/var/www/ficnexus` or a `/media/alvaro/code-worktrees/` path. Left unchanged (not user-facing).
