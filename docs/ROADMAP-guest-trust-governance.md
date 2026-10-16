# Guest access, trust levels and self-governance — ficnexus

Status: **proposal**, from the owner's request of 2026-10-16. It records intent in this repo's own
terms; nothing here is implemented. The estate-wide standard (items GS-1…GS-5, every setting
configurable, defaults and firewalls) is:

    secondbrain/10-Projects/2026-10-16-guest-access-trust-and-self-governance.md

This file is the site-specific half: what exists here today, what this site adopts, and the
firewalls that differ from the standard because this site is not the same shape as the others.

Stack: Rust (axum) + SvelteKit, frontend/ with pnpm.

## Where ficnexus stands today (measured 2026-10-16)

- **most of items 1 and 3 are already built**: `src/visitor/mod.rs` (signed `fh_vis` UUID cookie, documented as "anonymous users get trust-0 functionality"), `src/visitor/middleware.rs`, `frontend/src/lib/stores/consent.svelte.ts` and `frontend/src/lib/ui/consent/ConsentToast.svelte` (the two-state gate: `granted` cookie vs per-tab `rejected`), `src/routes/device_library.rs`
- `trust_level` is pervasive (440 occurrences) — item 4's substrate exists
- the fold is *stated* in the visitor module's header comment ("merged into their account on registration") but item 2's completeness check does not exist, and items 4's elasticity and all of item 5 do

## What this site adopts

- GS-2 (the fold, with the schema-driven completeness check) first — it is the gap between what is documented and what is enforced
- then GS-4's elasticity (the ladder exists; the fall-when-empty rule does not) and GS-5

## Config deltas (everything else is the standard's default)

- `consent.version` — the toast exists without a version stamp, so a text change would not re-ask
- `merge.grants_trust = false` (firewall §2)

## Firewalls specific to this site

- Nothing site-specific beyond the standard: the `fh_vis` cookie is signed and 30-day, and its trust-0 rule already matches `guest.parity_level = 1`.

## Order of work

GS-2 → GS-4 → GS-5 (GS-1/GS-3 are already in place). The standard's §9 explains the ordering:
consent first, because the guest identity *is* a cookie and the gate has to exist before the thing
it gates; the fold second, because offering guest access without it creates work to clean up later.

## If this is implemented

Every step needs a test that fails without it, and the numbers belong in this file when they
are real — the estate's rule is that a claim of done carries the command that proves it. The
two items in the standard that nothing in this repo has yet are the elastic trust rule (§4)
and the jury drawn from the eligible pool (§5): both are cheap to fake and therefore need a
test that would pass for the wrong reason to be written, and killed, on purpose.
