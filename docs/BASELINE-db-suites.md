# Real DB-suite baseline

First trustworthy measurement in this cycle, taken 2026-09-26 after
`provision_test_db.sh` was fixed (see `docs/specs/forum-api-isolation.md`).

## How to reproduce

    cd ~/code-local/rust/ficnexus
    export DATABASE_URL='postgres://…/ficnexus_test'
    export REDIS_URL='redis://127.0.0.1:6379' JWT_SECRET='…'
    export CARGO_TARGET_DIR=~/.cargo-target/ficnexus
    bash scripts/run_db_suites.sh

`run_db_suites.sh` now re-provisions before every suite and takes an flock on
`/tmp/ficnexus_test_db.lock`. Do not touch the test database while it runs.

## Totals

| | |
|---|---|
| suites | 57 |
| pass | 38 |
| fail | 19 |
| no-result | 0 |
| tests passing | 325 |
| tests failing | 43 |

Unit library (no database): **935 passed, 0 failed, 3 ignored**.

## Failing suites

| suite | pass | fail |
|---|---:|---:|
| forum_api | 41 | 8 |
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
| ask_archive_api | 6 | 1 |
| bookmark_csv_api | 4 | 1 |
| comment_triage_api | 2 | 1 |
| curator_fix_api | 2 | 1 |
| scheduled_topics_api | 2 | 1 |
| tags_api | 2 | 1 |
| user_export_api | 2 | 1 |

## Reading a number correctly

`forum_api` is **8 failures** under `run_db_suites.sh` and **12** when run by
hand. That is not instability — the runner exports
`FORUM_POST_DELAY_SECS=0` to disable the 10s flood-control window, and four
flood-control tests fail without it. Quote the runner's number, or quote the
hand-run number *and* say which one you used.

`admin_api` is 13/1: the one failure is `admin_users_search_role_ban`, the stale
fixture already documented in the admin-flag spec.

## What was wrong with the earlier numbers

Everything quoted for the DB suites before the provisioning fix (7, 11, 12, 23,
42/49, 24/31) came from a database that was 42 tables short. They are not
comparable to this table and should not be cited.
