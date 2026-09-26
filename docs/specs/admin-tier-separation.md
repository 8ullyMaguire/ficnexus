# FicNexus — specification: moderator (TL5) can reach admin-only (TL10) endpoints

Status: active. Written 2026-09-26, before any code change.
Plan: `docs/PLAN-admin-tier-separation.md`.

## 1. Problem

`admin_api` reports 8 failures. Seven of them are one defect, and it is an
**authorization gap, not a fixture problem**: every `/api/admin/*` route in
`src/routes/admin.rs` gates on `trust_level < 5`, while the tests hold six route
families to trust level 10.

A user at trust 5 — "staff" in the trust ladder, a forum moderator — can read
`/api/admin/stats`, list every bot client, shadowban a bot, read and act on the
comment moderation queue, drive the auto-tag queue, and read or write the fic and
author blacklists. Those are administrator actions.

## 2. Evidence

### 2.1 Every admin route uses the same threshold — and there are 77 of them

The threshold is copy-pasted inline rather than centralised, which is why none
of them is at 10. It is not confined to `admin.rs`:

    admin.rs 37,  auto_tag.rs 5,  heal.rs 4,  reports.rs 4,
    bulk.rs 3,  copyright.rs 3,  extensions.rs 3,  forum_groups.rs 3,
    forum.rs 3,  trust.rs 3,  analytics.rs 2,  upload.rs 2,
    bounties.rs 1,  curator_content.rs 1,  docs.rs 1,  skins.rs 1

**77 inline copies across 17 files.** This is why centralisation (R3) is a
requirement and not a tidy-up: the boundary between staff and admin is
expressed 77 times in 17 files, so there is no single place to change it and no
way to audit whether any given route is stricter than its neighbours.

### 2.2 The tests are unambiguous and consistent

Each failing test sends a trust-5 token and asserts 403, and separately sends a
trust-10 token and asserts 200. From `admin_blacklist_add_and_list`:

```rust
let admin = seed_admin_user(&db, "adminit_bl_admin", 10).await;
let low   = seed_admin_user(&db, "adminit_bl_low", 5).await;

// Role 5 -> 403 on every endpoint.
("GET",  "/api/admin/blacklist",         None),
("POST", "/api/admin/blacklist/fic",     Some(json!({ "url_id": fic_id, "reason": 5 }))),
("POST", "/api/admin/blacklist/author",  Some(json!({ "source_id": 4242, "author_id": 4243, "reason": 6 }))),
...
assert_eq!(s, StatusCode::FORBIDDEN, "{method} {uri} must require role 10");

// ... then, with the trust-10 token:
assert_eq!(s, StatusCode::OK, "blacklist fic: {body}");
```

Both halves. The tests are not asserting that admin is unreachable; they are
asserting the boundary is at 10, not 5.

### 2.3 The seven failures map to six route families

| test | route | failure |
|---|---|---|
| `admin_bots_requires_role_10` | `GET /api/admin/bots` | 200, want 403 |
| `admin_bot_shadowban_requires_role_10` | `POST /api/admin/bots/{id}/shadowban` | 200, want 403 |
| `admin_moderation_comments_queue` | `GET /api/admin/moderation/comments` | 200, want 403 |
| `admin_moderation_comment_actions` | `POST /api/admin/moderation/comments/{id}/hide` | 200, want 403 |
| `admin_stats_returns_daily_and_totals` | `GET /api/admin/stats` | 200, want 403 |
| `admin_auto_tag_queue_approve_dismiss` | `GET /api/admin/auto-tag/queue` | 200, want 403 |
| `admin_blacklist_add_and_list` | `GET/POST /api/admin/blacklist{,/fic,/author}` | 200, want 403 |

The eighth failure, `admin_users_search_role_ban`, is a different defect and is
out of scope for this spec — but it is the same *identifier* problem, in the
opposite direction, and section 3 explains it.

## 3. A third overloaded identifier, which is why this was invisible

`role` means three different things in this codebase, and the test suite uses
two of them at once:

| axis | where | meaning |
|---|---|---|
| `users.role` | database column | legacy "trusted account" flag, set to 1 by subsystem approval (`src/routes/subsystems.rs:475`) |
| `AuthUser.trust_level` | JWT claim | the trust ladder, 0-10 |
| `AuthUser.level` | JWT claim | forum leveling, separate from trust |

`seed_admin_user` at `tests/admin_api.rs:184` writes **`users.role`**:

```sql
INSERT INTO users (username, password_hash, role) VALUES ($1, 'x', $2) ...
```

while `auth_header(user_id, username, trust_level)` puts the value the routes
actually read into the **JWT**. So `seed_admin_user(..., 10)` sets a column no
admin route consults, and the routes read the third argument of `auth_header`.
The parameter is named `role` in the helper and is really a trust level.

The test passes anyway because both calls use consistent literals — but the
naming actively misleads. Anyone reading `seed_admin_user(db, name, 10)` would
reasonably conclude the route under test reads `users.role`. It does not.

This is the third time an overloaded `role`/`level` identifier has cost real
time in this codebase, after the JWT-vs-database `trust_level` split in
`uploads_api` and `reports`.

## 3.1 The same overload, in reverse: `set_user_role` is named after the wrong column

The eighth failure, `admin_users_search_role_ban`, is the mirror image of
section 3 and settles which side is wrong.

The test sends `{{"role": 5}}` and then asserts the **database** column:

```rust
Some(json!({ "role": 5 })),
assert_eq!(s, StatusCode::OK, "set role: {body}");
let role: i16 = sqlx::query_scalar("SELECT role FROM users WHERE id = $1") ...;
assert_eq!(role, 5, "role persisted");
```

The handler requires `trust_level` and writes `users.trust_level`:

```rust
let new_role: i16 = payload.get("trust_level") ... ;
sqlx::query("UPDATE users SET trust_level = $1 WHERE id = $2")
```

**The handler is right and the test is stale.** The deciding evidence is JWT
issuance, `src/routes/auth.rs:145-162`:

```sql
RETURNING id, username, trust_level, reputation, email, level, exp
...
trust_level: row.2,
```

The claim every admin route authorizes on is read from `users.trust_level`. A
handler that wrote `users.role` would have no effect on any authorization
decision in the product. So `PUT /api/admin/users/{id}/role` writing
`trust_level` is correct behaviour, and the test asserting `users.role`
persisted is asserting a column nothing reads.

The endpoint is **named** `role` and the variable is called `new_role`, which is
how the confusion started. The route name is a documentation problem, not a
behaviour one; renaming a public API path is a breaking change and is not in
scope.

## 4. Requirements

**R1.** The six route families in 2.3 reject `trust_level < 10` with 403.

**R2.** Trust level 10 continues to succeed on all of them. The tests assert
this half too, and a change that makes admin unreachable is not a fix.

**R3.** The threshold is **centralised**, not copy-pasted into six more places.
77 inline `trust_level < 5` checks across 17 files is why this drifted. One
helper, used by every route that gates on it, so the next threshold change is
one edit and the routes can be audited against each other.

**R4.** The distinction between staff (5) and admin (10) is expressed in
`Config`, not as a literal, so a deployment can widen or narrow it.

**R5.** No route is made *less* restrictive. The other 31 admin routes stay at
their current threshold unless the tests say otherwise. Widening the gate on
routes nobody asked about would lock out working moderators on deploy for no
stated reason.

**R6.** The naming in `seed_admin_user` is corrected so the parameter it writes
and the value the routes read are not both called `role`.

## 5. What is NOT being changed, and why

**The 31 other admin routes keep `trust_level < 5`.** Only the six families the
tests hold to 10 move. R5. Whether the remaining 31 *should* also be 10 is a
real question — `/api/admin/users`, `set_user_role`, `toggle_ban`,
`rep_award_handler` and `update_work_metadata` all sound like administrator
actions that a moderator should arguably not reach — but no test states an
intent for them, and changing 31 authorization boundaries on suspicion is how a
deploy locks out a moderator team.

**This is recorded as a finding for an owner, not acted on.** See section 6.

## 6. For the owner: the other 31

Routes that check `trust_level < 5` and, by name, look like they should require
10:

    admin_users, set_user_role, toggle_ban, rep_award_handler,
    update_work_metadata, approve_upload, reject_upload,
    run_embedding_dedupe, run_content_scan, search_mining,
    admin_backfill_tags, admin_backfill_bodies, admin_realtime,
    admin_search_analytics, admin_roadmap_consensus

`admin_users` and `set_user_role` in particular: a moderator being able to
promote users to admin is a privilege-escalation path, not just a data leak.
Nothing in the current test suite covers them, which is a test gap as much as a
code gap.

The centralised helper from R3 makes closing this a one-line change per route
once the decision is made, which is the main practical reason to do R3 first.

## 7. Non-goals

- The 31 routes in section 6.
- `admin_seed_user_upserts_and_lists`, a separate defect.
- Unifying `users.role` with `trust_level`. `users.role` is a real legacy column
  with a real consumer in `subsystems.rs`; it is not dead and removing it is a
  separate piece of work.

## 8. Acceptance

1. All seven tests pass.
2. For each of the six families, a trust-5 token gets 403 **and** a trust-10
   token gets 200 — both asserted, because a gate that returns 403 to everyone
   would satisfy the first half alone.
3. `cargo test --lib` still 935 passed; all 57 suites compile.
4. `grep -rc 'trust_level < 5' src/routes/*.rs` shows the moved routes replaced
   by helper calls, and the total count across all 17 files drops by exactly the
   number moved. Every remaining copy is untouched (R5).
