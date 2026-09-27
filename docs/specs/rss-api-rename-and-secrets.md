# rss_api: a rename that missed two tests, and two different JWT secrets in one file

Date: 2026-09-27 · Status: diagnosed, fixing
Suite: `tests/rss_api.rs` — 3 passed, 4 failed

## 1. Two tests assert the pre-rename URN prefix (`urn:fichub:`)

`new_arrivals_feed_renders_atom` and `per_fic_feed_renders_single_entry` assert

    body.contains("urn:fichub:fic:rss-new-1")
    body.contains("urn:ficnexus:feed:work:rss-perfic-1")   // ← mixed

The handlers emit `urn:ficnexus:*` (`src/routes/rss/handlers.rs:59,91,119`), and
so do the OPDS feeds and the atom manifest. This was a **deliberate, documented
rename on 2026-08-29** — `docs/sessions/2026-08-29-session-summary.md` lists it
explicitly and names `src/routes/rss/*.rs` among the files updated:

    - `src/db/queries.rs` (bulk): all `urn:fichub:*` URN identifiers → `urn:ficnexus:*`
    - `src/routes/opds/*.rs`, `src/routes/rss/*.rs`, `src/routes/saved_search.rs`: …
    - `src/routes/subsystems.rs`: `SITE_NAME_DEFAULT` "FicHub" → "FicNexus"

The source is correct; **these assertions are stale**. `Cargo.toml` still says
`name = "fichub"` (the crate name was not part of the rename), which is exactly
the sort of leftover that makes the old prefix look canonical — worth not
"fixing" on its own, since the crate name is referenced from `fichub::` paths
throughout the test suite.

**Decision: update the assertions to `urn:ficnexus:`.** A URN prefix is a public
identifier surface — feed readers persist them, and the rename was intended.

Note the second test already asserts a `ficnexus` feed id one line apart from a
`fichub` entry id, which is how this survived a month: each assertion is
individually plausible.

## 2. Two tests get 401: the test file contains two different JWT secrets

`follows_feed_with_token_renders_followed_fic` and
`follows_feed_includes_followed_author_fics` both mint a token and pass it as
`?token=`. The handler (`follows_feed` → `authed_user_id` → `verify_token`)
rejects them, 401.

The cause is in the test file:

    // app()
    jwt_secret: "fichub-test-secret".into(),          // ← what the server verifies with

    // auth_token()
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "fichub-dev-secret".into());   // ← what it signs with

Two distinct literals, same file, no shared constant. The token is signed with
one and verified with the other. `JWT_SECRET` is unset in the suite runner
(`scripts/run_db_suites.sh`, and this session's `r1.sh` helper), so both
fallbacks are live code paths — not dead defaults.

The `feed/follows.xml?token=` mechanism itself is correct and is real product
behaviour: feed readers cannot send `Authorization` headers, so the query-param
path exists for exactly this. Only the test's secret wiring is wrong.

**Decision: introduce one constant** (`const TEST_JWT_SECRET`) used by both
`app()` and `auth_token()`, and have `auth_token` sign with it unconditionally
rather than reading the environment. An env lookup in a test that also
hard-codes a secret for the same purpose is the trap: it passes when the runner
happens to export the matching value and 401s when it does not, which is the
definition of a test that reports on its environment rather than on the code.
