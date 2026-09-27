# requests_api: five failures from one asynchronous product feature

Date: 2026-09-27 · Status: diagnosed, fixing
Suite: `tests/requests_api.rs` — 6 passed, 5 failed

## Also explains a sweep anomaly

This suite was `7 passed / 4 failed` in the last sweep and is `6 / 5` now, with
no code change to it. That is not noise: **Ollama came up** (I pulled
`nomic-embed-text` earlier this session for the `auto_tag` suite), and the
failing behaviour depends on a live Ollama.

Worth recording because it means **these results are not reproducible on a
machine without Ollama** — the suite has an undeclared external dependency.

## Root cause: the Archivist answer is a real feature the tests don't know about

`create_request` spawns a background task that writes an automatic answer:

    // ── Archivist LLM answer (best-effort, never blocks creation) ──
    tokio::spawn(async move {
        let v2 = crate::search::ask::translate_for_request(&state2, &nl).await;
        sqlx::query(
            r#"INSERT INTO fic_request_answers (request_id, user_id, work_id, pitch, source, answer_kind, search_query)
               SELECT $1, r.user_id, NULL, $2, 'archivist', 'llm', $3 FROM fic_requests r WHERE r.id = $1"#
        )
        …
    });

So a freshly created request can gain an `archivist`-sourced answer a few
milliseconds later, asynchronously, with no gate and no config flag. Every
failing assertion counts answers in absolute terms:

| test | asserts | actual |
|---|---|---|
| `create_list_detail_request` | `answers.len() == 0` | 1 (archivist) |
| `answer_and_vote_flow` | `answers.len() == 1` | 2 |
| `answer_via_url_id_ask_flow` | `answers.len() == 1` | 2 |
| `delete_answer_flow` | `answers.is_empty()` | 1 |
| `duplicate_answer_and_cap` | 3 answers all HTTP 200 | 400 on one |

Diagnostic that settled it — asked the database, not the SQL:

    DIAG answers=2 kinds=[String("archivist"), String("user")]

## Why the product is right and the tests are wrong

The Archivist is deliberate, documented in the handler, and useful: posting a
request gets you a live search immediately instead of an empty answer list. It
is best-effort by design — Ollama down means no auto-answer, and the test suite
on a machine without Ollama would pass these assertions.

The tests are not wrong for being strict; they are wrong for asserting a
**global** count in a system that has an **asynchronous** contributor to that
count. That is a latent flake regardless of Ollama: even with Ollama present, the
race is real, and on a slow machine the archivist insert can land *after* the
assertion, making the same run pass or fail on timing.

Two things are being fixed, and only one of them is the count.

### The genuine race

`delete_answer_flow` asserts `answers.is_empty()` after deleting the user's
answer. If the Archivist task has not yet inserted, the list is empty and it
passes; if it has, the list has the archivist row and it fails. **The outcome
depends on how fast the machine is.** The fix is not "assert 1" — it is to
assert on *which* answers exist, excluding the archivist explicitly.

### The cap test, which is different

`duplicate_answer_and_cap` expects three answers from one user to succeed and a
fourth to be rejected. It got 400 on one of the first three. The per-user cap
is `MAX_ANSWERS_PER_REQUEST` (requests.rs:22) and it counts **all** answers on
the request — including the archivist's, which is attributed to `r.user_id`, the
requester. So the requester already occupies one slot before posting anything.

That is arguably a **product bug**: a system-generated answer consuming a quota
that exists to limit *user* answers. The requester is penalised for a row they
did not write.

## Decisions

- **Counts**: tests filter `source == "user"` (or exclude `"archivist"`) rather
  than asserting absolute lengths. This removes the race instead of papering
  over it with a sleep.
- **The cap**: counting an auto-answer against a per-user answer cap is wrong,
  and I am fixing the **product** here, not the test. The cap should count
  `source = 'user'` answers only. This is the one place in this batch where the
  test was right and the product was wrong.

Not doing: a `tokio::time::sleep` to let the Archivist land, or a
`--test-threads` fudge. Both would hide the race rather than remove it, and the
first would make the suite slower on every run.

## Result

    requests_api: test result: ok. 11 passed; 0 failed; 0 ignored   (74.5s)

The 74 seconds is the Archivist calling Ollama once per created request. The
suite is slower than most because it does real LLM work, and it is *correct* for
the first time on a machine with Ollama running.

## Related, found and deliberately not changed

`list_requests` reports `answer_count` from an unfiltered count:

    (SELECT COUNT(*) FROM fic_request_answers a
      WHERE a.request_id = r.id AND a.deleted_at IS NULL)

so it **includes the Archivist's automatic answer**. That is arguably the right
product choice — the count is a "how much help did this get" signal and the
Archivist is real help — and unlike the per-user cap it does not penalise the
requester. **No test asserts `answer_count` at all**, so changing it would be an
untested behaviour change, and I am not making one. Recorded here so the next
person to see an off-by-one `answer_count` knows it was a decision, not an
oversight.

Scanned all suites for the same race: `requests_api` was the only one asserting
absolute answer counts on `/api/requests`.
