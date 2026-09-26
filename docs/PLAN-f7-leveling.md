# FicNexus — implementation plan: F7 site-wide leveling

Companion to `docs/specs/f7-leveling.md`. Executable from this text alone.

**Steps 1-3 are unambiguous and are implemented here.** Steps 4-5 (the level
gates) are blocked on an owner decision about the level curve and are recorded
as findings, not implemented. See "Blocked" at the end.

## Step 1 — grant exp on topic create (R1)

**File:** `src/routes/forum.rs`, in the topic-create handler.

`update_reputation_and_promote` already takes a delta and a reason, and
`Config` already carries `forum_exp_topic_create`. Find the success path of
`POST /api/forum/topics` - immediately after the topic row is inserted and the
id returned - and add:

```rust
// F7 site-wide leveling: topic creation earns exp. Value comes from Config
// (FORUM_EXP_TOPIC_CREATE, default 2) so it is tunable without a rebuild.
if state.config.forum_exp_topic_create > 0 {
    let _ = crate::db::queries::update_reputation_and_promote(
        &state.db,
        user_id,
        state.config.forum_exp_topic_create,
        "topic_create",
    )
    .await;
}
```

`update_reputation_and_promote` is called with `let _ =` everywhere else in
this codebase - it is a best-effort side effect and must not fail a post.
Follow that convention.

**The OP post is the same row as the topic.** Do not also grant `post_create`
for it, or the first topic would be worth 4 exp and
`f7_level_exp_from_topic_and_post` would assert the wrong total.

## Step 2 — grant exp on post create (R2)

**File:** `src/routes/forum.rs`, in the post-create handler
(`POST /api/forum/topics/{id}/posts`).

Same shape, using `state.config.forum_exp_post_create` and reason
`"post_create"`. This one *is* separate from the topic, because a reply is its
own row.

## Step 3 — the mod_received daily cap (R4)

**File:** `src/routes/forum.rs:750`.

The hook is already there:

```rust
if delta > 0 {
    let _ = crate::db::queries::update_reputation_and_promote(
        &state.db, author_id, 1, "mod_received").await;
}
```

The comment claims "capped +3/day" and **no cap is implemented** - the call
happens on every positive moderation. Add the cap before the call, following
the same shape as `daily_mod_usage` at `src/routes/forum.rs:499`:

```sql
SELECT COUNT(*) FROM reputation_events
 WHERE user_id = $1 AND reason = 'mod_received'
   AND created_at >= date_trunc('day', NOW())
```

Skip the grant once that count is 3. Check the real table name first with
`\d` - do not assume `reputation_events`; `update_reputation_and_promote` in
`src/db/queries/reputation.rs:8` is the authority on what it writes.

**Run this step's verification before moving on.** `f7_level_mod_received_exp_capped`
currently expects 3 and gets 0, which does not obviously match "the hook exists"
- confirm the test's own setup actually reaches the hook before assuming the cap
is the only gap.

## Step 4 — level-up notification (R3): BLOCKED

Needs the level curve. Not implemented. See "Blocked".

## Step 5 — level gates (R5): BLOCKED

`f7_level_gate_admin` expects 403 and gets 200; `f7_level_gate_curator` expects
403 and gets 409. **Which level gates which capability is a product decision
with real consequences** and is not derivable from config defaults. Not
implemented. See "Blocked".

## Step 6 — verification

```bash
export CARGO_TARGET_DIR=/home/alvaro/.cargo-target/ficnexus
export FORUM_POST_DELAY_SECS=0
cargo build                                        # clean
cargo test --lib                                   # 935 passed
touch tests/*.rs && cargo test --no-run 2>&1 | grep -c 'could not compile'   # 0
cargo test --test forum_api f7_ -- --include-ignored --test-threads=1
```

**R6 proof - the values come from Config, not literals.** This is the assertion
that makes steps 1-2 real rather than hardcoded:

```bash
FORUM_EXP_TOPIC_CREATE=0 cargo test --test forum_api   f7_level_exp_from_topic_and_post -- --include-ignored --test-threads=1
```

It must now observe 0 exp instead of 2. If it still sees 2, the value is
hardcoded and the step is not done.

**Assert where the value lands, not that a count is right.** A test asserting
"exp went up" passes even if the grant is wrong. The suite reads the exact
number from `users.exp`, which is right - keep it.

## Step 7 — commit

- `feat(forum): grant site-wide exp for topic and post creation (F7)`
- `fix(forum): cap mod_received exp at +3/day (F7)`

Tag `f7-exp-sources-2026-09-26`. Sync, and commit the session note to
`~/secondbrain`.

## Blocked - needs an owner decision

**The level curve.** `f7_level_exp_from_topic_and_post` asserts a user with 2
exp stays at level 0. That bounds the first threshold from below but does not
determine the rest, and three of the failing tests depend on where the
thresholds are. Guessing here either locks out legitimate users or makes the
gates cosmetic.

**The gate assignments.** `f7_level_gate_admin` expects 403 at some level for
the admin route; `f7_level_gate_curator` expects 403 for the curator route and
currently gets 409. Neither the level nor the routes are inferable from the
code.

Until those are settled, these two tests stay red and `forum_api` will not be
fully green. That is the correct outcome, and it is recorded rather than
hidden.
