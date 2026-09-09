# FicHub — Next Steps Plan (2026-08-23, post pass-4)

> **Update 2026-08-27:** the 2026-08-27 pass shipped the trust system
> (7-level axis, trust-weighted reports + auto-triage, weekly digest,
> TL2/TL5 gates), the unified extension marketplace (`/marketplace`,
> migration 014), entity recommendations
> (`/api/v0/recommendations/entities`), and the trust/marketplace frontend
> (`/settings/trust`, `TrustBadge`). See the session log in
> `docs/brainstorm-02-roadmap-status.md`. The P1 polish-debt items below are
> still open.

> Supersedes `docs/PLAN-2026-08-23.md` and
> `.hermes/plans/2026-08-23_103420-site-quality-pass4.md` (both complete —
> every task landed, deployed to ThinkCentre, QA 257/0, pushed).

## Where we are

Site Quality Pass 4 shipped in full: OPDS hash-aware acquisition links +
RWPM manifest, curator consensus hub, typed fic-request answers
(work/LLM/search), ask↔requests unification shim, search-history chips,
AO3 symbol squares + guide modal, Marginalia (per-passage forum threads,
curator admin page), emoji reactions (anchored picker per design doc),
sequential-Markov Next Up, personalized-recs opt-in with settings toggle,
per-endpoint usage analytics, ask-response caching. Migrations through 066
applied on prod. Known follow-up: `/curator` hub card renders "Marginalia ✒️"
— emoji in UI text violates the house rule; strip it.

## Suggested next pass (priority order)

### P1 — Polish debt from pass 4 (half-day)
1. **Strip emoji from UI text**: `✒️` on the curator hub card; audit
   HomeDashboard (`🔥`, `✨`, `🧑‍🏫`, `🔍`, `⬇`) and reader toolbar glyphs —
   replace with text or gate behind CSS icons. House rule is no emoji in UI.
2. **Marginalia e2e click-test by hand**: create a passage discussion as a
   level-5 user, confirm icon count, tooltip preview, deep-link highlight,
   curator page excerpt. No automated coverage yet for the full loop.
3. **Reactions live click-test**: picker anchoring under scroll, wrap
   behavior with many reactions, archive-skin variant.
4. **QA harness portability**: `qa/run.js` defaults to `localhost:8000`
   but prod binds fine only from thinkcentre itself; document
   `QA_BASE=<tunnel URL>` in docs/DEPLOYMENT.md.

## P2 — Invite cohort launch
Skipped — no onboarding/wizard or seed content needed currently.

### P3 — Roadmap backlog picks (see §3 of roadmap)
1. **Search system overhaul** now unblocked (`main_char_attr` removed):
   faceted filters UI parity with backend capabilities.
2. **Reliable story updates & cache refresh** (P2 area of roadmap §ops):
   scheduled re-scrape for tracked works with stale caches.
3. **External uptime probe** (30 min, free) — UptimeRobot or similar on
   `/api/health`; alerts to a real channel.
4. **Body-cache backup** cron (20 min): rsync cache + EPUBs off-box.

### P4 — Bigger rocks (spec first, then delegate)
1. **Forum depth phase 2** per `docs/THREADLIGHT-FEATURES.md` remaining
   items (metamod UI polish, mod-power user tools).
2. **Extension platform v3.1** per `docs/v3-customization-spec.md` §11 —
   community-shared custom recommenders/themes.
3. **OPDS 2.0 full conformance**: test against Calibre + KOReader
   explicitly, cover navigation feed pagination edge cases.

### P5 — Wattpad-style feature audit (spec-first)

The user wants the Wattpad feature set evaluated for inclusion. Not all of
these match FicHub's archive-plus-community philosophy — each needs a spec
that reconciles it with FicHub's existing systems (levelling, modlog-style
curator consensus, no-negative-engagement policy, archive parity). Group A
= ship-as-is (aligns with current direction), Group B = adapt (keep the
value, reshape to FicHub norms), Group C = skip (violates house policy).

**A. Align as-is**
1. **Inline paragraph commenting in the web reader** — This is exactly
   **Marginalia** (already shipped). Wattpad calls it "inline comments" with
   highlight + reaction; FicHub has passage-hash anchored forum threads.
   Gap: Wattpad's UI floats a reaction toolbar on text selection. Consider
   adding a lightweight reaction shortcut to the existing highlight tooltip.
2. **Author-facing analytics dashboard** — FicHub's `/api/analytics` +
   `/api/admin/endpoint-usage` cover system usage; extend with per-work
   view/read/download/engagement metrics (read-count by chapter, avg time,
   completion rate). Curator-level visibility (role ≥ 5).
3. **Personalized reading stats dashboard** — FicHub already tracks
   `reading_history`/`reading/status` + bookmarks/ratings. Surface a
   `/user/{id}/stats` page: total words read (Σ chapter word counts),
   favorite genres (tag frequency), streak (days active), shelves
   breakdown. Leverages existing `usage_events` + `user_prefs`.
4. **Community writing contests / prompt challenges** — Map to forum +
   curator consensus: contest proposals → `/api/curator/consensus`
   proposal → community submissions via `fic_request_answers` or a new
   `contest_entries` table → voting via existing request votes. Spec out
   structured submission form (title, word count, required tags) reusing
   the upload pipeline.

**B. Adapt (keep value, reshape to FicHub norms)**
5. **Algorithmic infinite-scroll home feed** — FicHub deliberately uses a
   4-level fallback (personal → trending → popular → recently added), not
   a dopamine-maximizing TikTok feed. Adapt: add infinite scroll pagination
   to the existing HomeDashboard left column, driven by the same ranked
   query — continuous discovery without engagement-bait psychology.
6. **Built-in drafting + scheduled publishing** — Scope to a "write" area:
   a Markdown editor that persists drafts to a `user_work_drafts` table
   (reuse the existing upload/convert path) with a future `publish_at`
   timestamp. Publish creates a `fic_request` (community curation entry)
   or a curated `works` row (curator-only). Respect the archive skin.

**C. Skip (violates FicHub house policy)**
7. **Spendable virtual currency / paywalled premium chapters** — FicHub is
   donation-financed (BUY_ME_A_COFFEE link on `/donate`), not a walled
   marketplace. Monetizing reader attention or gating chapters behind pay
   contradicts the archive-everything ethos. Do not implement.
8. **Private direct messaging between users** — FicHub has public-by-default
   forums (moderated via modlog-style tools). Private DMs invite toxic
   behavior with no public record. Skip; point users to curator-flagged
   public forum threads instead.
9. **Public downvoting / negative engagement metrics on frontend** — FicHub
   uses positive-only karma (kudos/notifications are +1 style). Negative
   public voting enables brigading and drive-by trolling. Skip; if reaction
   feedback is needed, expand the emoji-reaction picker (Group A) instead.
10. **Walled-garden mobile app exclusivity** — FicHub is deliberately
    open-web (OPDS 2.0, PWA-installable, no app store gate). A mobile app
    that restricts web access contradicts this. Skip. If a mobile UX boost
    is needed, ship installable-PWA improvements instead.

All Group A/B items need individual specs before implementation. Group C is
explicitly out of scope pending user override.

## Working agreements (unchanged)
- Spec-first for big features; parallel subagent batches; orchestrator
  verifies + commits; never push until gates green.
- Gates: cargo check/tests, svelte-check 0 errors, vitest full suite,
  qa/run.js against deployed instance.
- Deploy pipeline: release build (CARGO_TARGET_DIR=worktree) → scp →
  md5 verify → migrations via psql → frontend /tmp build → rsync --delete
  → curl sweep → push.
