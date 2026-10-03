# Admin/curator tier gates — the 37 count is wrong, and the real defect is a privilege bug

Spec: `docs/specs/admin-tier-separation.md` (written 2026-09-26, before any code change)
Status: **4 of 18 closed** (2026-10-03). The premise below was measured and found wrong.

## Correction (2026-10-03): measured, not assumed

This document said 37 inline gates in 16 files. Measured: **18** `trust_level < 5`
sites, and **70** calls to `require_admin_tier` already in the tree. The 26 `admin.rs`
copies it describes as outstanding were closed. So the count is stale by 19.

More importantly, **the 18 are not interchangeable**, and converting all of them to
`require_admin_tier` would have made two of them *worse*.

### Why a blind conversion is a privilege escalation

`require_admin_tier` reads `auth.is_admin`. `AuthUser` documents why that is a
different thing from `trust_level`:

    /// F7 trust level (0-6). Replaces legacy `role`.
    pub trust_level: i16,
    /// Administrator, from the token's `is_admin` claim. This is what
    /// `trust::require_admin_tier` reads — not `trust_level`, which is bounded
    /// to 0-6. See `docs/specs/admin-flag.md`.
    pub is_admin: bool,

`trust_level` is CHECK-constrained to 0-6, so `trust_level < 5` never meant "is an
admin" — it means "has reached rung 5", a *mid-tier* user. Three constants already
encode the tiers and the gates use none of them:

| constant | value | for |
|---|---|---|
| `ADMIN_MIN_TRUST` | 5 | admin tools |
| `RESOLVE_MIN_TRUST` | 5 | resolving disputes |
| `CURATOR_MIN_TRUST` | **3** | curator tools |

**`CURATOR_MIN_TRUST` was referenced nowhere outside its own definition.** Every
curator gate hardcoded `5`. That was the live bug: trust levels 3 and 4 were refused
curator tools they were entitled to.

### Closed in this pass (4)

- `src/recommender/routes.rs:540`, `:607` — pure admin gates, now `require_admin_tier`.
- `src/tags/curator.rs:24`, `src/routes/curator_content.rs:29` — curator gates, now a
  new `services::trust::require_curator` reading `CURATOR_MIN_TRUST`.

The curator two are the ones the old plan would have converted to the **admin**
helper, turning a too-strict check into a wrong-tier one.

`tests/curator_tier_boundary.rs` (6 tests) pins the threshold, that the two tiers stay
separate in both directions, and that no curator call site reintroduces a literal.
Mutation-verified: restoring the literal `5` fails with *"trust level 3 is the stated
curator minimum and must be admitted"*.

### Still open (14), and why each keeps its own comparison

Not gates — owner-or-staff, where a helper cannot express the second half (6):

    forum_groups.rs:505   non-admins fall through to an owner lookup
    extensions.rs:113     !is_public && author != uid && trust < 5
    skins.rs:260          owner != uid && trust < 5
    upload.rs:295,401     uploader-or-curator
    reports.rs:484        reporter-or-staff

Compound with a second threshold that conversion would drop (5):

    forum.rs:221          require_admin_at_level: admin AND auth.level < min_level
    forum.rs:1243         forum-scope bans
    forum.rs:2629         marginalia, level-gated
    forum_groups.rs:626   only admins may set role = owner
    forum_groups.rs:662   same, on a different handler

Not gates at all (2): `admin.rs:329` is the comment describing this bug;
`services/trust.rs:360` is a SQL predicate selecting banned users.

For the 5 compound sites the fix is not `require_admin_tier` but replacing the
literal `5` with the right named constant, so the tier is stated once.

---

# Original plan (kept for the history; its count was wrong)

# Replace the remaining 37 inline admin gates

Spec: `docs/specs/admin-tier-separation.md` (written 2026-09-26, before any code change)
Status: **not started** — follow-up to commit that closed 26 of them in `admin.rs`

## What is already done

`src/routes/admin.rs` no longer gates on trust level. Its 26 inline copies are
now:

    crate::services::trust::require_admin_tier(&user, &state)?;

which reads the `is_admin` claim. `require_admin_tier` already existed and was
already used in `auto_tag.rs`, `forum.rs` and `auth.rs` — `admin.rs` had 3
correct call sites and 26 wrong ones in the same file, 30 lines below a comment
explaining why the wrong ones were wrong.

## What is left: 37 copies in 16 files

    reports.rs 4   heal.rs 4   trust.rs 3   forum.rs 3   forum_groups.rs 3
    extensions.rs 3   copyright.rs 3   bulk.rs 3   upload.rs 2
    auto_tag.rs 2   analytics.rs 2
    skins.rs 1   docs.rs 1   curator_content.rs 1   bounties.rs 1

`admin.rs` still shows 1, which is the explanatory comment at
`src/routes/admin.rs:339`, not a gate.

`docs/specs/admin-tier-separation.md` counted 77 across 17 files. That count
included `admin.rs`'s 26, so **37 is the number that was always there behind
them.**

## The work, per file

For each handler carrying the inline block:

    -    if user.trust_level < 5 {
    -        return Err(AppError::Forbidden("Admin access required".into()));
    -    }
    +    crate::services::trust::require_admin_tier(&user, &state)?;

Variants to expect, all seen in `admin.rs`:

- the parameter is named `auth` rather than `user`
  (`admin.rs:492` `rep_award_handler`)
- the state extractor is present in every case checked so far; the build is the
  verification, not a read-through

## Why this is not cosmetic

Before the `admin.rs` change, any user at trust 5 could reach every
`/api/admin/*` route. Trust 5 is "staff" — a forum moderator. It is not
administrator, and the project's own decision (`docs/specs/admin-flag.md`,
commit `b497721`) is that admin is the `is_admin` flag, not a rung on the
ladder. `users.trust_level` is CHECK-constrained to 0-6, so no trust value can
express "administrator" in the first place.

The 37 remaining handlers have the identical exposure. This is a live
authorization gap, not a tidy-up.

## Why the spec's centralisation requirement still stands

`require_admin_tier` reads a flag, so calling it 37 more times centralises
*which* check is made but not a *threshold* — there is no threshold any more.
The spec's remaining concern is still valid in a weaker form: with 37 copies in
16 files, there is still no single place to audit whether any route is stricter
than its neighbours, and a future route can still be written with an inline
gate by mistake.

The durable fix is a middleware or extractor on the `/api/admin` router so
authorisation is applied by construction. That is a larger change than this
follow-up; worth doing once the 37 are gone and the pattern is uniform.

## Verification

Per file, after conversion:

    cargo build                                  # every handler needs `state` in scope
    cargo test --test <suite> -- --include-ignored --test-threads=1

Then the whole gate, because this touches authorization:

    bash scripts/run_db_suites.sh
    cargo test --lib

Suites that will need their `auth_header` helper given an `is_admin` parameter,
the same change made in `admin_api`, `content_scan_api`, `metadata_api`,
`analytics_api`, `comment_triage_api`, `curator_fix_api`, `heal_api` and
`reports_api`:

    `auth_header_for(user_id, trust_level, username, is_admin)`

A suite that was reaching an admin route on trust alone **will** start failing
when the gate is corrected. That failure is the gap being closed, not a
regression — do not "fix" it by lowering the trust level.
