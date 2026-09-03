> STATUS: **SHIPPED (2026-08-27)** — implemented as a 7-level system (this
> plan sketched 5; the build extended it to Discourse-parity 0–6 with two
> staff tiers). See `src/services/trust.rs` (levels, weights, promoter),
> `src/routes/trust.rs` (`/api/me/trust`, `/api/admin/trust*`,
> `/api/admin/digest`), `src/bin/trust_promote.rs`, and the trust-gated
> write surfaces in `src/routes/reports.rs`, `recipes.rs`, `skins.rs`.
> Frontend: `/settings/trust`. Migration: `013_trust_levels.sql`.

# Trust Levels Rework — Discourse-inspired

> Canonical plan: `docs/brainstorm-02-roadmap-status.md` current focus + the
> existing `users` / `exp_events` / `award_exp` tables (F7 site-wide leveling
> shipped 2026-08-17, migrations 052/053). This doc is the step-by-step to rework
> the trust tier into a Discourse-style 5-level system layered *on top of* the
> XP engine.

## Context

FicHub currently has a 100-level / 10-rank progression system (XP → level →
rank, gates features by `rank`). What it does NOT have is a *safety* trust tier
— new users can immediately post topics, send PMs, flag, etc.

Discourse's model: TL0 (New) sandboxed → TL1 (Basic) = full read+core write →
TL2 (Member) = citizenship → TL3 (Regular) = community helpers → TL4 (Leader) =
near-moderator. Trust is earned by *behavior over time*, separate from "XP kudos."

We add trust as a **second axis** alongside rank. Rank = status/badges/feature
gates. Trust = safety/sandbox permissions. A user can be high-rank (posted a lot)
but TL0 (just created) → sandboxed until they read/visit consistently.

## Current state

- `users.exp` (legacy, used by old `update_reputation_and_promote`)
- `users.xp` / `users.level` / `users.rank` (v3 engine, live — see
  `docs/plans/reputation-xp-rewards-v3.md`)
- `exp_events` audit trail + `award_exp` on posts (F7)
- `users.role` (0=user, 10=admin) — used for admin gates, NOT for sandbox

## Target schema

Add to `users` (new migration `011`):

```sql
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS trust_level smallint DEFAULT 0 NOT NULL CHECK (trust_level BETWEEN 0 AND 4),
  ADD COLUMN IF NOT EXISTS tl0_until timestamptz,          -- null = no sandbox
  ADD COLUMN IF NOT EXISTS tl_metrics jsonb;               -- cached read counters
CREATE INDEX IF NOT EXISTS idx_users_trust_level ON users (trust_level);
```

- `trust_level`: 0–4, Discourse-matching names.
- `tl0_until`: nullable end-of-sandbox. When > now(), TL0 sandbox applies regardless of metrics (manual override / new-user grace).
- `tl_metrics`: cached counters so promotion is O(1) per visit, not a 100-day rollup on every request.

Seed `xp_source_defs` with read/track events (these already exist as page hits but aren't awarded XP — tie them to trust):

| event_type | xp | trust-relevant? | daily_cap |
|---|---|---|---|
| `read_topic` | 0 | counts toward TL1 (5 topics entered) | — |
| `read_post` | 0 | counts toward TL1 (30 posts) + TL2 (100/60min) | — |
| `send_like` | 1 | TL2 (1 like cast) | 60 |
| `receive_like` | 0 | TL2 (1 received) | — |
| `daily_visit` | 2 | TL2 (15 days), TL3 (50% of 100d) | 2 |

> XP on likes/visits feeds the XP axis; the *counts* (not the XP) feed trust.
> Keep XP small so trust is distinct from "farmer who spams reads for XP."

## Promoter job

New `src/cron/trust_promote.rs` (or extend `compute_leaderboards`), runs hourly:

```
for each user where trust_level < 4 and (tl0_until IS NULL OR tl0_until < now()):
  load tl_metrics
  if tl < 4 and meets_criteria(user, tl=4):         # manual, skip (staff-only)
     continue
  if tl < 3 and meets_criteria(user, tl=3):         # strict: visit%>=50, reply>=10, view>=25%(cap 500), read>=25%(cap 20k), likes given>=30 received>=20, <5 spam flags last 100d, no suspension last 180d
     set trust_level=3; log event
  else if tl < 2 and meets_criteria(user, tl=2):     # visited 15d (any last 30), gave 1 like, got 1 like, replied 3 topics, entered 20 topics, read 100 posts, 60 min read
     set trust_level=2; log event
  else if tl < 1 and meets_criteria(user, tl=1):     # entered 5 topics, read 30 posts, 10 min read
     set trust_level=1; emit welcome TL1 notice
```

**Demotion:** TL3 is revocable — if the last-100-day rollup dips below threshold,
demote TL3→TL2 after a 14-day grace. TL0/1/2 are monotonic (never lose them once
earned, matching Discourse).

**TL0 grace:** new accounts start `tl0_until = created_at + 14 days` so the very
first two weeks sandbox everything (honeypot + PoW already in place).

## Trust-permission gates (the sandbox)

Replace the `user.role < 10` / `role >= 10` checks for *safety* features:

| Action | Gate |
|---|---|
| Send PM | `trust_level >= 1` (not TL0) |
| Flag posts | `trust_level >= 1` |
| Post >2 hyperlinks | `trust_level >= 1` |
| Post image/attachment | `trust_level >= 1` |
| Reply to topics | `trust_level >= 1` (TL0 can comment on own work only) |
| Create new topic | `trust_level >= 1` (TL0: redirect to `/requests` only) |
| "Reply as new topic" | `trust_level >= 1` |
| Daily likes (limits) | scaled by trust (TL1: 6, TL2: 9, TL3: 12, TL4: 15) |
| Curator tools (hide/flag queue) | `role >= 10` still (admin/curator) — trust doesn't grant moderation |
| Access to TL3-only lounge category | `trust_level >= 3` |

`role` (staff) stays the authority for *moderation*; `trust_level` is the
authority for *participation safety*. A TL4 Leader is trusted but can't ban —
only `role >= 10` (curator/admin) can.

## Admin UX

- `/admin/users/{id}` shows `trust_level` + `tl_metrics` + one-click
  `trust_level=` override (logs to modlog).
- `/forum/lounge` category: readable only by TL3+ (the "Regulars" space).
- Welcome PMs / in-app notifications on TL1/TL2/TL3 promotion (reuse the
  existing notification service).

## Migration strategy (zero-downtime)

1. `011_trust_levels.sql` adds columns + backfills: TL0 for brand-new users (<14d),
   TL1 for anyone who already has `exp_events` showing read activity (they're
   clearly past onboarding).
2. Promoter backfills `tl_metrics` from `usage_events` history (30-day window)
   on first run.
3. Gate checks added gradually behind a `features.trust_levels` toggle (rank 1) —
   ship TL0 sandbox first, then TL1/2 gates, then TL3 lounge.

## Open questions (decide before build)

1. **TL3 like-quotas**: Discourse requires likes across ≥1/5 distinct users and
   ≥1/4 distinct days. Do we enforce the cross-user spread, or just raw counts?
   *Recommendation: raw counts + a separate "spam flags" signal — the cross-user
   spread is a TF/TL3-only nicety; we don't have a like-volume problem yet.*
2. **TL0 content**: should TL0 users still *consume* (read fics) while sandboxed?
   *Recommendation: yes — read is the single fundamental action per Discourse;
   sandbox write + social, not consumption.*
3. **Backfill baseline**: users who've read 100+ posts but just created their account
   — TL1 immediately or grace period? *Recommendation: immediate TL1 (they've
   proven engagement) but keep the 14-day read-only-if-suspended rule.*
