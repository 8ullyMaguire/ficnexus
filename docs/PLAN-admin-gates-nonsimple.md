# The 15 admin gates that are not mechanical swaps

Spec: `docs/specs/admin-tier-separation.md`
Plan executed: `docs/PLAN-admin-gates-followup.md`
Status: **not started** — the 25 unconditional gates in that plan are done; these are the rest

## Why the plan's premise was wrong

`docs/PLAN-admin-gates-followup.md` said:

> For each handler carrying the inline block:
>     -    if user.trust_level < 5 {
>     -        return Err(AppError::Forbidden("Admin access required".into()));
>     -    }
>     +    crate::services::trust::require_admin_tier(&user, &state)?;

and counted 37 sites as if they were that block. They are not. A mechanical
substitution was applied to 25 of them and **hit a helper function that had no
`state` parameter and whose own docstring argued for the opposite**:

    /// `is_admin` is deliberately not an alternative path here. This is the forum
    /// gate, governed by forum participation; the `/api/admin/*` tier is a separate
    /// surface guarded by `require_admin_tier` in `admin.rs`. Letting the flag in
    /// here would make "who can create a category" depend on a flag unrelated to
    /// the forum.
    fn require_admin_at_level(auth: &AuthUser, min_level: i16) -> Result<i32, AppError> {
        let uid = require_user(auth)?;
        if auth.trust_level < 5 { ... }

That one was reverted. The compiler caught it — `state` is not in scope — but the
docstring is the real reason it was wrong, and a mechanical edit would have
removed it.

**The count was also wrong: 40 sites, not 37.** 25 are unconditional admin
gates (one of which is the `forum.rs` helper, which stays as written — so 24 were
replaced). The other 15 fall into four shapes, and three of the four are correct
as written.

## Group B — moderator or verified-account tiers, not admin (4 sites, all correct)

    forum.rs:398      if auth.trust_level < 3   "Moderator access required"
    forum.rs:1030     if auth.trust_level < 3   "Moderator access required"
    skins.rs:215      if owner != Some(uid) && auth.trust_level < 3
    upload.rs:30      if trust_level < 1        "Uploads require a verified account"

A trust level **is** the right gate for these. Moderation and verified-account
are rungs on the community ladder, not the administrator flag. `docs/specs/
admin-flag.md` says admin is `is_admin`; it does not say trust stops meaning
"moderator". Leave all four.

## Group C — owner-or-admin bypass (5 sites, correct, but worth a test)

    reports.rs:486    if row.0 != user_id && auth.trust_level < 5
    extensions.rs:115 if !ext.is_public && auth.user_id != Some(ext.author_id) && auth.trust_level < 5
    upload.rs:295     if user.trust_level < 5 && work.uploader_id != Some(user_id)
    upload.rs:401     if user.trust_level < 5 && work.uploader_id != Some(user_id)
    skins.rs:260      if owner != Some(uid) && auth.trust_level < 5

These are "the owner may act, or an admin". The trust test here is the
**admin** half, so `is_admin` is the correct replacement — but the compound
condition means the rewrite is `owner || is_admin`, and `is_admin` must not
become a *second* way for a non-owner to act when the owner branch already
allowed it. Note `skins.rs:215` (trust 3) and `skins.rs:260` (trust 5) are the
same route's two branches; do not unify them by accident.

Lowest-risk change in this group, but the one most likely to be done
mechanically and wrongly.

## Group D — conditional admin bypass on a named scope (4 sites, need product intent)

    forum.rs:1243     if scope == "forum" && auth.trust_level < 5   "Forum-scope bans require admin"
    forum.rs:2599     if is_marginalia && auth.trust_level < 5      "Level 5 required for marginalia"
    forum_groups.rs:626  if role == "owner" && auth.trust_level < 5
    forum_groups.rs:662  if body.role == "owner" && auth.trust_level < 5

Each grants admins an escape hatch over a domain-specific rule. The question per
site is the same and needs an answer before code: **is the flag the right
escalation path here, or is this ladder level doing domain work?**

`forum.rs:2599` is the clearest: the message says "Level 5 required for
marginalia", which reads as *the domain demands level 5*, with an admin
bypass bolted on. If the bypass is intentional, say so; if not, the admin gets a
power the ladder never granted anyone.

## Group E — fallthrough with a follow-up query (1 site, cannot be a swap)

    forum_groups.rs:505
        if auth.trust_level < 5 {
            // Non-admins must be owner
            let role: Option<String> = sqlx::query_scalar(...)

Not a guard that returns — it conditionally queries. `require_admin_tier` returns
`Result`, so the rewrite has to preserve the fallthrough:

    let is_admin = crate::services::trust::require_admin_tier(&auth, &state).is_ok();
    if !is_admin { /* the owner check */ }

Worth reading that whole handler before touching it.

## Group F — curator tier (1 site, separate ladder)

    curator_content.rs:29   if user.trust_level < 5   "Curator access required"

Distinct from admin by name and message. There is a
`docs/specs/f7-level-gate.md` about the trust/level split; check whether curator
is meant to be a trust rung or a flag before changing this one. **Not a
mechanical swap and not obviously in scope for admin-tier work.**

## Summary

| Group | Sites | Action |
|---|---|---|
| A unconditional admin | 25 | **24 done** — replaced with `require_admin_tier`; `forum.rs`'s helper stays as written (documented) |
| B moderator / verified | 4 | leave — trust is correct here |
| C owner-or-admin | 5 | rewrite as `owner \|\| is_admin`; add a test per site |
| D conditional bypass | 4 | needs an owner answer per site |
| E fallthrough | 1 | hand rewrite, preserve the fallthrough |
| F curator tier | 1 | check `docs/specs/f7-level-gate.md` first |

**C, D and E are the remaining authorization work: 10 sites.** They are not 10
copies of one mistake, and treating them as copies is what produced the
`forum.rs` helper regression above.

## The durable fix is still middleware

After A–F, `/api/admin/*` and every escalated surface is guarded by hand, in
40+ places. A middleware or extractor on the admin router applies the check by
construction, and then these decisions become *exceptions to document* rather
than *checks to remember*. That is the change that stops this recurring; the 25
mechanical replacements do not.
