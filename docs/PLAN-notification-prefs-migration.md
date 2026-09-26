# Add the six granular notification toggles (migration 060, which was never written)

Spec: `docs/specs/phantom-columns-500s.md` §6
Baseline: `41 pass, 16 fail, 0 config-missing, 57 suites` (commit `bd6e248`)

## The decision: add the migration, do not trim the code

`GET /api/notifications/preferences` returned
`{"err":-1,"msg":"database error"}` for every authenticated user, because the
code selects sixteen columns from `notification_preferences` and migration 001
created ten.

The six missing ones are `comments_on_work`, `replies_to_comments`,
`kudos_on_work`, `bookmarks_on_work`, `follows` and `mentions`.

**I first decided to trim the code, wrote the plan for it, and was wrong.**
The reversal is recorded below because the reasoning error is the reusable part.

## Why trimming looked right, and was not

The argument for trimming was that nothing reads the six fields, so nothing can
depend on them. The measurement behind it was sound and partial:

    src/db/queries/social.rs:839  notify_comment_reply   -> reads prefs.comment_reply
    src/db/queries/social.rs:876  notify_work_followers  -> reads prefs.work_update
    src/routes/notifications.rs:130   GET handler
    src/routes/notifications.rs:162   PUT handler

Both *delivery* paths gate on `comment_reply` and `work_update` — both of which
exist. No delivery path reads the six. "So the feature is inert; delete it."

Three things invalidate that, and none of them are visible from `src/`:

**1. The model names a migration that does not exist.**

    src/db/models.rs:195
    // ── Granular AO3-style toggles (migration 060) ─────────────────────

`ls migrations/` runs 001–094 with **60 through 63 missing**. 060 was never
written. So this is not a field that was cut — it is a migration that was lost,
and the struct comment is the surviving evidence of intent.

**2. The frontend ships UI for all six.**

    frontend/src/routes/settings/+page.svelte:880
    <dt>{t('settings.notifKudosOnWork')}</dt>
    <dd><input type="checkbox" bind:checked={notifPrefs.kudos_on_work}
               onchange={saveNotifPrefs} /></dd>

Translated labels, bound checkboxes, auto-save on change — six of them, in the
settings page. Users can see and toggle these today. The toggles cannot save
because the endpoint 500s, but the UI is real and the intent is explicit.

Trimming the backend would have removed the server side of a feature whose
client side is already deployed, turning a visible 500 into a settings page that
silently discards what the user toggles. That is strictly worse.

**3. The fields are read as a set, so they cannot be dropped piecemeal.**

`NotificationPreference` carries all thirteen, so removing six means touching
the model, the SELECT, the default-row INSERT, the UPDATE, the GET response
keys, the PUT body struct, the assigns, *and* the UI — a cross-cutting change
across two languages to remove a feature, on the strength of a grep.

## The lesson

I established "no code path reads these fields" and treated that as settling
the question. It settles a narrower question. The features that were genuinely
vestigial in this repo — `badge_text`, in the previous commit — were vestigial
in **every** layer at once: no model, no UI, no test, no consumer. The
distinguishing test is not "is it read" but "is it present in all the layers a
real feature would be present in". `badge_text` was absent from all of them;
these six were present in five of six, with only the migration missing.

Searching `src/` alone cannot tell those apart, because one of the layers a
real feature needs is a file under `migrations/` and another is a `.svelte`
file. The grep I ran was scoped to the directory where the answer was least
likely to be.

## What the migration does

`migrations/060_notification_prefs_granular.sql` adds the six columns as
`boolean DEFAULT true NOT NULL`, plus a `COMMENT ON COLUMN` for each carrying
the doc comment already on the struct field.

Defaults are `true`, matching every other toggle in this table, so an existing
user gains working toggles without opting in and with no behaviour change: the
delivery paths gate on `comment_reply` and `work_update`, which already
existed, so nothing starts or stops being sent. `ADD COLUMN IF NOT EXISTS`
makes it safe against a database that somehow already has them.

`updated_at` is deliberately **not** touched. A migration that changes no
preference value should not look like a preference edit in an audit trail.

## Applying it: 060 slots into a gap, it does not renumber anything

The sequence is 001–059, then 064–094 (plus missing 9 and 80). 060 fits the gap
exactly, so nothing is renumbered and no applied migration is touched.

`src/bin/migrate.rs` calls `Migrator::set_ignore_missing(true)` and then applies
whatever is pending, so a database that already reached 094 picks up 060 on the
next deploy without complaint. Verified rather than assumed: the test harness
replays `ls migrations/*.sql | sort`, and after the file was added
`information_schema` reports 16 columns for `notification_preferences` in the
right order — the six appended after `updated_at`, which is the order the
struct expects.

## One thing I did not fix, found on the way

`cargo run --bin migrate` panics on a fresh database:

    migration failed: ExecuteMigration(Database(PgDatabaseError { code: "42723",
    message: "function \"update_fic_tag_score\" already exists with same
    argument types" }))

That is a **pre-existing** conflict, unrelated to 060 — it fires on a migration
somewhere before it, and the test harness tolerates it because it applies files
individually with per-file error capture rather than through sqlx's migrator.
So the deploy path (`fichub migrate`, which `Deploy.sh` runs before restarting
the service) may be broken independently of everything in this plan. Not
diagnosed, not fixed, and it needs its own investigation — but it means
"migrations apply cleanly in production" is not currently a verified fact and
should not be assumed from the suite being green.

## Verification

- `information_schema` reports 16 columns for `notification_preferences`
  (mutation check: 9 without 060).
- `social_api` **9 passed, 0 failed** (was 8; +1 new test).
- New `granular_notification_toggles_round_trip` drives the real GET and PUT,
  turns three toggles off, reads them back, asserts the three absent from the
  PUT are untouched, and checks the row on disk. Mutation-checked: without 060
  it fails, with 060 it passes.
  - The existing `notifications_flow` asserted only the seven migration-001
    fields. The six had **no** coverage, which is how a 500 on this endpoint
    shipped — nothing failed until a suite happened to call GET /preferences.
  - The round trip matters more than the defaults: defaults come from the column
    DEFAULT and would pass even with the UPDATE broken.
- Sweep target: 42 pass, 16 fail, `0 config-missing`; `cargo test --lib` 935.
