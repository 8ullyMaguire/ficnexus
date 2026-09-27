# The last six suites: two product bugs, one config trap, three stale tests

Date: 2026-09-27 · Status: diagnosed, fixing
Sweep before: 52 pass, 6 fail, 58 suites

| suite | test | cause |
|---|---|---|
| `admin_api` | `admin_users_search_role_ban` | **product** — `/role` route reads the wrong column |
| `curator_fix_api` | `fix_applies_after_quorum_of_other_curators` | **config trap** — unwritable `BODY_CACHE_DIR` |
| `tags_api` | `search_autocomplete_resolve` | stale test — wrong response key |
| `bookmark_csv_api` | `export_returns_csv_with_header_and_rows` | stale test — pre-rename filename |
| `user_export_api` | `user_export_contains_all_expected_data` | stale test — pre-rename filename |
| `scheduled_topics_api` | `publish_scheduled_flips_and_notifies` | stale test — wrong bound column |

## 1. `admin_api` — the admin role editor is broken in the web UI (product)

`PUT /api/admin/users/{id}/role` returns **400**, because the handler reads a
field the client does not send and writes a column the client does not mean:

    // src/routes/admin.rs — set_user_role
    let new_role: i16 = payload
        .get("trust_level")                       // ← the client sends `role`
        .and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::BadRequest("trust_level field required (0-6)".to_string()))?;
    sqlx::query("UPDATE users SET trust_level = $1 WHERE id = $2")

The frontend sends the other name:

    // frontend/src/routes/admin/users/+page.svelte:26
    adminFetch(`/api/admin/users/${userId}/role`, {
      method: 'PUT', …
      body: JSON.stringify({ role }),
    })

So **the admin user-role editor 400s for every real admin**. The route is named
`/role`, the payload field is `role`, the frontend sends `role` — only the
handler disagrees, and it disagrees twice: wrong field in, wrong column out.

`users.role` is a live column, not a vestige — `src/routes/subsystems.rs:526`
writes it (`UPDATE users SET role = 1 WHERE id = $1 AND role = 0`) and
`subsystems.rs:32` documents gating on it. The test's assertion
(`SELECT role FROM users WHERE id = $1`) is the correct expectation.

**Decision: fix the handler** to read `role` and write `role`.

One consideration, recorded rather than assumed: the `role` and `trust_level`
columns are separate and both live, and the trust gates read `trust_level`
(50 sites) while `db/queries/proposals.rs:93` and `subsystems.rs:475` read
`role`. The handler's own comment justifies its choice at length, so this is not
a stray edit. But its comment argues the *access* decision (use `is_admin`, not a
trust level) and then silently changes the *write* target too — two different
decisions fused in one function. The fix keeps the access decision exactly as
documented and corrects only the write target, so the comment stays true.

The response `msg` stays "Role updated", which is what the test asserts.

## 2. `curator_fix_api` — the default body-cache dir is root-only (config trap)

    DIAG http=400 Bad Request body={"err":-1,"msg":"failed to apply body: Permission denied (os error 13)"}

The vote logic is correct — it reaches `VOTE_QUORUM`, computes `net >= 1`, and
fails only at the filesystem write. `config.rs:617`:

    let body_cache_dir = std::env::var("BODY_CACHE_DIR")
        .unwrap_or_else(|_| "/public/literature/fichub/bodies".to_string());

That default is a path only root can create, and the suite runner does not set
`BODY_CACHE_DIR`. So the test — correctly — got as far as writing a real body
blob, into a directory it may not touch.

`tests/body_search_api.rs` already solves this: it allocates a per-run temp dir
and assigns `config.body_cache_dir = body_dir`. This suite has no such
`test_config` override, so it inherits the production default.

**Decision: give the suite its own temp body dir**, following the
`body_search_api.rs` pattern, including the `remove_dir_all` on the way in. The
alternative — exporting `BODY_CACHE_DIR` from the runner — would be one line and
would fix every future suite at once, but it changes what the canonical sweep
runs, so it belongs in its own change with its own verification rather than
riding along here.

This one is worth naming: the test is not wrong, and the product is not wrong.
The default is a legitimate deployment path (a mounted volume on a server), and
a test that exercises a real code path will hit it.

## 3. `tags_api` — the test reads a key the API has never returned

    let results = b["results"].as_array().unwrap();     // panics: no such key

`search_tags` returns `tags`:

    Ok(Json(json!({ "err": 0, "total": …, "page": …, "limit": …, "tags": tags })))

and the frontend's own type agrees:

    // frontend/src/lib/api/tags.ts
    export interface TagSearchAdvancedResponse { err: number; msg?: string; tags?: TagItem[]; total?: number; }

**Decision: fix the test** to read `tags`. Two independent sources — handler and
frontend type — agree on the key, and one test disagrees.

**But not everywhere.** `/api/tags/search` (`search_tags`) returns `tags`, while
`/api/tags/autocomplete` (`tag_autocomplete`) returns `results` — different
handlers, different shapes, and the frontend has a separate type for each. A
blanket find-and-replace across the file was wrong on the two autocomplete
assertions and produced a *new* failure (`tags_api` then panicked on a short-query
assertion at line 497 that had previously passed). Fixed by reverting precisely
those two lines and recording the distinction in a comment, because "the tags
key" is not a repo-wide fact.

Both shapes are self-consistent and neither is a bug; the confusion is only in
having two similarly-named endpoints with different response envelopes.

**Adjacent defect, not fixed here:** the frontend sends `name=` and
`sort_by=`/`sort_direction=`/`wrangle_status=` while the handler's
`TagSearchParams` reads `q`/`sort`/`canonical`/`tag_type`. So
`searchTagsAdvanced`'s `q` is silently dropped and the advanced tag search does
not filter. That is a real product/frontend mismatch, but it is a third
behavioural change in a batch that already has two; recorded here so it is not
lost, and it gets its own change.

## 4 & 5. `bookmark_csv_api`, `user_export_api` — more pre-rename filenames

The 2026-08-29 rename (see `docs/sessions/2026-08-29-session-summary.md`)
included:

    - `src/routes/user_export.rs`: download filename `fichub-user-data-*.zip` → `ficnexus-user-data-*.zip`
    - `src/routes/social.rs`: bookmarks CSV filename `fichub-bookmarks.csv` → `ficnexus-bookmarks.csv`

and the source does emit the new names (`src/routes/user_export.rs:381`,
`src/routes/social.rs:396`). The two assertions were left behind — the same
residue as the `urn:fichub:` URNs in `v50`, and the third and fourth instance of
this pattern.

**Decision: fix the assertions.** A download filename is a public artefact a
browser shows to users; the rename was intentional.

## 6. `scheduled_topics_api` — the fixture binds the wrong column

    .bind(cat_slug.as_str())   // → category_id  (an i64 column)
    .expect("seed scheduled topic")

`seed_category` returns `(id, slug)` and the caller unpacks `let (_cat_id,
cat_slug) = …` — the id is discarded with a leading underscore and the *slug*
is bound to `category_id`. The insert fails at `.expect`.

An earlier failure in this same test had the fixture naming a nonexistent
`slug` column and binding the same value twice; that was fixed by renaming to
`topic_slug` and the comment records it. The `category_id` binding was missed in
the same repair.

**Decision: bind `cat_id`** and stop discarding it.

## 7. `scheduled_topics_api` — a 500 on every brand-new topic (product)

Once the fixture seeded successfully, the listing returned:

    DIAG after=[] topic_id=1 list={"err":-1,"msg":"database error"}
    DIAG2 recent status=500 Internal Server Error body={"err":-1,"msg":"database error"}

A 500, not a wrong result — so the diagnosis was the decode type, not the query
shape. `forum_topics.last_post_id` is `bigint NULL`:

    id|bigint|NO ... last_post_id|bigint|YES

and every one of the five sort branches in `list_topics` selected it bare while
the row tuple declared it `i64`:

    Vec<(i64, String, Option<String>, i32, String, i64, i64, i64, i64, …)>
                                     ↑ t.view_count, ↑ t.last_post_id  ← NULL

Proven on the live database, not reasoned about:

    INSERT INTO forum_topics (category_id, author_id, title, body, topic_slug) …
    SELECT id, (last_post_id IS NULL) FROM forum_topics WHERE topic_slug='zs';
    →  1|t

**So a newly created topic 500s the entire forum listing and the recent feed** —
not just its own row. `last_post_id` is NULL until the first reply, which is
every topic's state between creation and its first response.

Fixed with `COALESCE(t.last_post_id, 0)` in all five branches, which is what the
adjacent `unread` expression already did (`t.last_post_id IS NOT NULL AND …`) —
that neighbouring NULL guard is precisely why the bare column looked safe.

**Scope was larger than the first failure suggested.** Fixing `list_topics` moved
the failure to the *next* assertion in the same test, which pointed at a second
handler. A repo-wide sweep for the same pattern found the bare column decoded
into a non-optional `i64` in **five more places across three more handlers**:

    unread_topics()   1 occurrence
    recent_topics()   1
    popular_topics()  3  (one per sort branch)

All ten occurrences in the crate are now guarded. Fixing only the handler the
first failing test happened to reach would have left the 500 reachable through
three other public endpoints, so the fix was driven by the pattern, not by the
symptom.

## 8. `user_export_api` — a GDPR export that silently dropped data (product)

`collect_progress` hardcoded its quest payload:

    Ok(json!({ "login_streak": …, "quests": null }))

`user_daily_progress` and `daily_quests` both exist and hold the user's quest
state, and this endpoint's own test seeds them. So `/api/user/export` shipped a
data-export feature that omitted a category of user data and reported success.

Fixed with a real join over the user's progress rows. An export that claims
completeness and is not is worse than one that documents what it leaves out,
which is the only reading of `"quests": null` that is defensible.

## 9. `scheduled_topics_api` — the test asserted against a copy of cron's SQL

After the listing 500 was fixed, the publish test failed with
`author should have a notification` — while its own visibility assertion had
started passing. The cause: the test pasted the flip SQL inline instead of
running what cron runs.

    // Run the same SQL the publish-scheduled binary executes.
    let flipped: Vec<(i64, i32, String, Option<String>)> = sqlx::query_as(
        "UPDATE forum_topics SET scheduled_at = NULL … RETURNING id, author_id, title, topic_slug",
    ) …

That copy flips rows and returns them, and **never notifies anyone**. The
`publish-scheduled` binary does both. So the notification assertion was checking
for a row that nothing in the test could ever create — it could only ever fail,
and no edit to the *production* code could have satisfied it.

Three defects in one shape: a duplicated query that can drift from its
executor, a test that cannot fail for the right reason, and a binary whose
logic was unreachable from a test at all.

Fixed by moving the logic to `db::queries::social::publish_due_topics`, which
both the binary and the test now call. The binary is reduced to connect → call →
report, so the test exercises the code cron actually executes.

And the test's own assertion was broken independently of all of that:

    SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND type = 'topic_published'
    … .await.unwrap_or(0)

There is no `type` column — it is `notification_type` — so the assertion query
itself was a SQL error, and `.unwrap_or(0)` converted that error into a
confident `0`. The test therefore reported "the author got no notification" when
the truth was "your assertion is invalid". A count query whose failure mode is
indistinguishable from its expected result is not a test. It now uses
`notification_type` and `.expect(...)`, and carries the reason inline.

A repo-wide scan for that same masked pattern
(`query_scalar(…).await.unwrap_or(0)`) found this was the only instance.

One behaviour decision, kept deliberately: a failed notification does **not**
abort the run. The row is already flipped and un-claimed; aborting would leave
it published-but-unnotified with no retry, and re-claiming it would republish.

## 10. `curator_fix_api` — the readback used a different directory than the writer

After the write path was fixed, the test failed on `load_body(...)` returning
`None`, because the assertion built a **fresh** `Config::from_env()` and so got
the root-only default back while the handler had written to the temp dir.

Both sides now go through one `test_body_dir()` helper. It uses
`create_dir_all` **without** a preceding wipe: the readback calls it *after* the
handler has written, and clearing there would delete the blob the assertion is
about. An earlier version of this helper did wipe, which would have produced a
second, more confusing failure.

## Order of work

1. `admin_api` handler fix (product) — 400 → 200, then the persisted `role`
   assertion proves the write went to the right column.
2. `curator_fix_api` temp body dir — reuses `body_search_api.rs`'s pattern.
3. The four test-side fixes, all mechanical.
4. Full sweep, verified by set difference against the previous 6.
