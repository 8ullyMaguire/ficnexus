# Research: OTW Archive Parity

## Decisions

### 1. Reference source: `otwarchive` app/views as visual/behavior spec, not code port

**Decision**: Use `otwarchive/app/views/works/*`, `chapters/*`, `bookmarks/*`, `comments/*`, `collections/*`, `skins/*` etc. as screenshot/order reference; implement in FicHub's SvelteKit components (e.g. `WorkBlurb`, `WorkMeta`, `CommentThread`) and Axum handlers. Keep stack Rust/Axum/Postgres/SvelteKit.

**Rationale**: User wants looks as similar as possible + manual testing slice-by-slice; porting Rails ERB would regress FicHub extras. Behavioral parity (section order, control placement, query params) is what AO3 users feel.

**Alternatives considered**: ERB-to-Svelte transpilation (fragile), full Rails parity branch (regresses forum/recs).

### 2. Filter/sort parity: mirror OTW `WorkSearchForm`/`WorkQuery`/`WorkIndexer`

**Decision**: Map OTW filter set (Rating, Warning, Category, Fandom, Relationship, Character, Additional Tags, Complete, Word Count, Date Updated, Language) and sorts (updated/revised, kudos, bookmarks, comments, hits, words) to FicHub's existing search. Where FicHub tags cover OTW tag types, reuse; where missing (e.g. warnings taxonomy), add taxonomy table + UI facets. Query stays URL-persisted.

**Rationale**: OTW `work_search_form.rb`, `work_query.rb`, `work_indexer.rb` define the exact facets AO3 users memorize. Reusing avoids duplicate search infra.

**Alternatives considered**: New search microservice (overkill for filter set).

### 3. Pseud / co-creator model: additive to existing user model

**Decision**: Add `pseuds` table (user_id, name, is_default) and `creatorships` join (work_id/series_id, pseud_id, approved). Byline renders `pseud (+ co-creators)`; dashboards list by pseud. Invite/approve flow mirrors `creatorships_controller.rb`. No breaking change to existing works (single pseud = default).

**Rationale**: OTW `pseud.rb`, `creatorship.rb` show pseuds are first-class; FicHub currently single username. Additive migration preserves existing data.

**Alternatives considered**: Repurpose username field (loses multi-pseud).

### 4. Series/collections incremental: lightweight first, challenges later

**Decision**: Phase 4 slice does Series + lightweight Collections (curated list, moderated flag, items). Challenge/gift-exchange (`challenge_models/`, assignments/claims/signups) deferred to Phase 5 after manual QA of earlier slices.

**Rationale**: User wants bit-by-bit polish; series/collection blurb+nav is daily AO3, challenges are event tooling (heavier).

### 5. Skins: sanitized CSS, site + work scope

**Decision**: `skins` + `work_skins` tables, allowlist CSS sanitizer (like OTW `skin.rb`/`work_skin.rb`), preview before apply. Archive skin stays default; user skins additive.

**Rationale**: OTW skins are self-hosted differentiator; minimal scope covers AO3 theming expectation without unsafe CSS.

### 6. Manual QA per slice: checklist + screenshots

**Decision**: `quickstart.md` holds a runnable checklist per increment (filters, blurb, read nav, kudos, etc.) with before/after AO3 screenshots. No automation gate required — user manually verifies visual parity before next slice, as requested.

**Rationale**: User explicitly wants "manually test everything works and looks as similar as possible" incrementally.

## Open items resolved

- All Technical Context unknowns resolved (language, deps, storage, testing, platform). No NEEDS CLARIFICATION remain.

