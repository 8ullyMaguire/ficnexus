# FicNexus — the F3 topic/post edit window: no window exists, and none is wanted

Status: **decided and implemented 2026-09-26.** Owner decision: the author
edit-expiry window is not wanted.

## 1. The decision

`f3_edit_topic_author_window_and_mod` asserted that an author gets 403 once
their topic is older than 15 minutes. There is no such rule in the code, and
there has not been one. The owner confirmed the window is not a wanted feature,
so the test's expectation was wrong, not the handler.

## 2. What `update_topic` actually does

`src/routes/forum.rs:2805`:

```rust
let user_id = require_user(&auth)?;
let (_, author_id, category_id, _, _) = load_live_topic(&state.db, topic_id).await?;
if is_banned(&state, user_id, category_id).await? { … }

let (is_author, is_mod) = (author_id == user_id, is_mod(&auth));
if !is_author && !is_mod {
    return Err(AppError::Forbidden("Not the topic author".to_string()));
}

let cur: (String, String, String) =
    sqlx::query_as("SELECT title, body, created_at::text FROM forum_topics WHERE id = $1")
```

`created_at` is selected and then **never compared to anything**. There is no
window, no 15-minute constant, and no config field for one. The author branch
runs unconditionally:

```rust
if !is_author {
    // insert into forum_edit_proposals; return {status: "pending"}
}
```

So the real rules are:

| actor | outcome |
|---|---|
| author | edit applied directly, at any age |
| moderator (trust ≥ 3), not the author | edit **proposed**, not applied — `forum_edit_proposals` row, `status: "pending"` |
| anyone else | 403 "Not the topic author" |

`is_mod` is `trust_level >= 3` (`forum.rs:392`), which is a lower bar than
`require_admin`'s 5. A trust-5 moderator is therefore a moderator but not an
admin here.

## 3. Two defects the test was hiding

The test had been red, and its second half had **never run** — the first
assertion aborted before reaching it. That dead half was also wrong.

**D1 — the author case asserted a feature that does not exist.** It aged the
topic an hour and expected 403. Replaced with an assertion that pins the real
rule: the author gets 200 and the title changes, at any age. The topic is still
aged deliberately, so the test proves age is irrelevant rather than merely
unused.

**D2 — the moderator case asserted a direct edit.** It patched the topic with a
trust-5 token and read the row back expecting `"Mod rename"`. But a moderator is
not the author, so `update_topic` takes the `!is_author` branch and files a
proposal. The title never changes. The test comment said "Mod can edit any time"
and the code has never allowed that.

Rewritten to pin the proposal path:

- response is 200 with `err: 0` and `status: "pending"`;
- `proposal_id` in the response equals the `forum_edit_proposals` row id;
- the topic row is **unchanged** — a proposal must not edit the topic.

The third assertion is the one worth having: it is what makes this a test rather
than a restatement of the handler.

## 4. Naming

`f3_edit_topic_author_window_and_mod` → `f3_edit_topic_author_any_time_and_mod_proposes`.

The old name asserted the behaviour that was removed. Leaving it would invite
the next reader to "fix" the handler by adding the window back.

`f3_edit_post_author_window_and_mod` was already green and is left alone. Its
name is also inaccurate, but it passes and renaming it is not part of this
decision; noted in §5.

## 5. Follow-up not taken here

- `f3_edit_post_author_window_and_mod` — same misleading name, same apparent
  topic. Green, so untouched. Worth renaming for the same reason.
- The 15-minute figure appears in comments in both tests. Both are updated or
  removed where the trim touched them; the post test still carries one.

## 6. Verification

Both edit tests green together:

    cargo test --test forum_api f3_edit -- --include-ignored --test-threads=1
    test result: ok. 2 passed; 0 failed

`forum_api` 43/6 → 44/5, and the full 57-suite sweep recorded in
`docs/BASELINE-db-suites.md`.
