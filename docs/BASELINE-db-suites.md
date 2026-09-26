# Baseline — DB integration suites

Current as of 2026-09-26, after the F7 level gate, the F3 edit-window trim, and
two harness fixes (`provision_test_db.sh` drop + Redis flush).

## How to reproduce

    cd ~/code-local/rust/ficnexus
    export DATABASE_URL='postgres://…/ficnexus_test'
    export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='…'
    export CARGO_TARGET_DIR=~/.cargo-target/ficnexus
    bash scripts/run_db_suites.sh

The script re-provisions the database **and flushes Redis** before every suite,
and holds an flock on `/tmp/ficnexus_test_db.lock`. Do not touch either store
while it runs — a concurrent `provision_test_db.sh` in another shell drops the
database out from under the run, which happened once and invalidated a full
sweep.

## Totals

| | before fixes | now |
|---|---:|---:|
| suites | 57 | 57 |
| pass | 38 | **39** |
| fail | 19 | **18** |
| no-result | 0 | 0 |
| tests passing | 325 | **329** |
| tests failing | 43 | **39** |

Unit library (no external state): **935 passed, 0 failed, 3 ignored**.

## Failing suites

| suite | pass | fail |
|---|---:|---:|
| forum_api | 44 | 5 |
| requests_api | 6 | 5 |
| search_api | 37 | 4 |
| rss_api | 4 | 3 |
| search_analytics_api | 0 | 3 |
| auto_tag | 0 | 2 |
| leaderboard_api | 1 | 2 |
| series_api | 2 | 2 |
| social_api | 6 | 2 |
| translation_review_api | 5 | 2 |
| work_proposals_api | 0 | 2 |
| admin_api | 13 | 1 |
| bookmark_csv_api | 4 | 1 |
| comment_triage_api | 2 | 1 |
| curator_fix_api | 2 | 1 |
| scheduled_topics_api | 2 | 1 |
| tags_api | 2 | 1 |
| user_export_api | 2 | 1 |

## The five remaining `forum_api` failures

```
admin_creates_category_and_list_returns_it
anonymous_gets_401_on_all_f3_routes
topic_list_returns_seeded_topic_with_counts
trust_grant_detail_stays_readable
trust_metamod_vote_gone
```

Both F7 gate tests and both F3 edit tests are green. What is left is a mix of
global-count assertions that see rows other suites left behind, and the 401-sweep
test.

## Reading a number correctly

`forum_api` is **5 failures** under `run_db_suites.sh` and **more** when run by
hand: the runner exports `FORUM_POST_DELAY_SECS=0` to disable the 10s
flood-control window, and those tests fail without it. Quote the runner's
number, or say which one you used.

`admin_api` is 13/1: the one failure is `admin_users_search_role_ban`, the stale
fixture already documented in the admin-flag spec.

## Numbers before 2026-09-26 are void

Every earlier figure — 7, 11, 12, 23, 24/31, 42/49, 38/19, 39/18 — was measured
against a **42-table-short database** (a silently-failing `DROP DATABASE`) and/or
with a **warm Redis** whose cached responses short-circuited the handlers under
test. Do not compare against them; see `docs/specs/forum-api-isolation.md`.

## What the two harness bugs were

1. `psql -c "DROP ..." -c "CREATE ..."` reports only the last statement's status,
   so the drop failed silently whenever a session was still connected, the
   script re-applied migrations onto the old schema, and it still printed
   `SCHEMA OK`. Measured: 138 tables against a claimed 180.
2. Redis survived the database rebuild, and `ask.rs:284` returns a cached
   response *before* `log_ask_analytics()`. Five stale keys made
   `ask_archive_api` report 0/7, then 6/1, then 6/1 on identical runs; deleting
   them gave 7/0 with no code change.
