## Part 7 — Testing, Working Here, and What's Next

### Chapter 22: The Testing Strategy

FicHub takes testing seriously — the repo convention is full test coverage
with frequent small commits. Three layers:

**1. Unit tests** — in-module `#[cfg(test)]` blocks. Fast, no DB. Run with:

```bash
cargo test --lib <name>        # e.g. cargo test --lib cache
```

**2. DB-gated integration tests** — `tests/*.rs` (e.g. `admin_api.rs`,
`modlog_api.rs`, `analytics_api.rs`, `heal_api.rs`). These boot a real
server against the real dev database. They're `#[ignore]`-marked by default
so a plain `cargo test` skips them, and they use a `Mutex` + unique seed
names + cleanup because the shared dev DB can't run them in parallel. Run
them with:

```bash
cargo test --test admin_api -- --include-ignored --test-threads=1
```

The pattern: each test seeds its own rows with unique names (e.g.
`test-client-%`), asserts, then cleans up. The `Mutex` guarantees one
DB-gated suite at a time.

**3. Frontend vitest** — `page.test.ts` next to each route. Run with:

```bash
npx vitest run src/routes/<x>/page.test.ts
```

**Repo convention: never run the FULL suite in normal dev.** Cargo takes
~10 minutes and DB-gated suites are slow. Run targeted suites only —
that's the house style.

The **canonical gate** is `hermes verify --save` (detect → build → test →
boot → readiness). It runs the real build and the full test pass and
records evidence. Known quirks: the shared NFS target's `incremental`
directory gets stale after mount wedges — `rm -rf target/debug/incremental`
before a canonical verify.

> 🧪 **Try it:** run one DB-gated suite end to end:
> `cargo test --test modlog_api -- --include-ignored --test-threads=1`
> (needs `.env` + dev DB). Watch it seed, assert, clean up.
>
> ⚠️ **Watch out:** if a DB-gated test fails with a duplicate-key error,
> it's usually a missing cleanup from a previous crashed run — the unique
> seed names are exactly what makes the cleanup tractable.
>
> 💡 **Key concept:** targeted suites, unique seeds, mutex-guarded DB
> access. The canonical verify is the final gate.

### Chapter 23: The Developer Workflow

Here's how work actually gets done on this repo. Follow this and you'll fit
right in.

**Branching.** Work on a feature branch, merge when verified. The repo
owner edits concurrently in another pane — don't squat on main. One git
worktree per feature when delegating (`/media/alvaro/code-worktrees/wt-<name>`).

**Builds.** Never build on `/home`. Use
`CARGO_TARGET_DIR=/media/alvaro/cargo-target-sh` (local ext4; the repo
lives on NFS). If you see NFS errors or disk-full, check the target dir.

**DB-gated tests need `.env`.** Worktrees ship only `.env.example`; copy
the real one:
`cp /personal/documents/code/rust/fichub/.env <worktree>/.env`.

**The NFS git-objects quirk.** After commit/merge, refs can dangle because
the repo lives on NFS. The recipe:

```bash
# per commit, when refs dangle:
export GIT_OBJECT_DIRECTORY=/home/alvaro/.cache/git-objects-<name>
git add ... && git commit ...
cp -rn /home/alvaro/.cache/git-objects-<name>/* .git/objects/ 2>/dev/null
unset GIT_OBJECT_DIRECTORY
git cat-file -t <sha>    # always verify
```

Also: `git config core.fsync none` first, and remove stale
`.git/refs/remotes/github/main.lock` before pushes.

**Conventional commits.** `feat:`, `fix:`, `test:`, `docs:`, `refactor:`.
Small, meaningful units.

**Deploy.** The live box is ThinkCentre (`fichub.polarisocial.xyz`).
Release build → copy binary → `sudo systemctl restart fichub.service` →
migrations apply on boot → verify health + the changed endpoint.

> 🧪 **Try it:** if you have a worktree, create a scratch branch, make a
> trivial commit, and walk the NFS object-copy recipe end to end. The
> `git cat-file -t` check is your safety net.
>
> ⚠️ **Watch out:** node_modules symlinks in worktrees point at a shared
> install — never commit them (mode 120000). The main frontend's
> node_modules is a real dir.
>
> 💡 **Key concept:** branch → build on local ext4 → targeted tests →
> conventional commit → verify → deploy. Boring on purpose.

### Chapter 24: Where to Start Contributing

The highest-leverage first tasks (from `docs/ROADMAP.md`) — all are real
gaps, sized for a junior dev, with the backend mostly or fully in place:

1. **Curator/admin UI backlog.** The APIs exist; the pages don't:
   - Auto-tag review UI (draft → approve ML tag suggestions)
   - Manual fic approval UI (`/api/admin/moderation/queue` exists)
   - Metadata correction (fix title/author/status/description on works)
   - Translation post-edit (draft → human)
2. **Search frontend control for `main_char_attr`.** The backend already
   supports "Dark Harry" semantics (main character + attribute). The UI
   pickers are missing — a clean frontend task.
3. **Fic Requests M3.** Notifications when a request gets answers/accepted,
   plus request upvotes. Backend plumbing mostly exists.
4. **Ops.** CI/CD, an external uptime probe, secrets hygiene (move DB
   password out of `.env` into a 600-perm systemd EnvironmentFile), and
   body-cache backups.

Pick one, branch, and follow the pattern: read the route file + its
`page.test.ts`, implement backend → frontend → test, verify with targeted
suites, commit conventionally, and get review.

**The pattern for a backend feature** (worth writing down):

1. `AppState` gets anything new it needs (or reuse existing).
2. Handler in `src/routes/<area>.rs` — `pub async fn`, `AppResult<T>`.
3. Register in `build_router()` (`{param}` syntax, Axum 0.8).
4. `modlog::record` if it's an admin/curator action.
5. DB-gated test in `tests/<area>_api.rs` (unique seeds + cleanup).
6. Frontend page + `page.test.ts`.
7. Targeted suites green → conventional commit.

> 🧪 **Try it:** pick ONE of the four starter tasks and spend an hour
> reading the files you'd touch. You don't have to finish — just map the
> change.
>
> ⚠️ **Watch out:** don't start with a huge architectural task. The
> starter list is sized to build confidence fast.
>
> 💡 **Key concept:** the API→test→frontend→test loop is the rhythm of
> this codebase. Every feature follows it.

### Chapter 24A: A Day in the Life — Troubleshooting Cheat-Sheet

When something breaks (and it will), here's the fast triage path. This is
the exact playbook the team uses.

**"The site is down / health check fails."**
1. `curl https://fichub.polarisocial.xyz/api/health` — which boolean is
   false?
2. `systemctl status fichub.service` + `journalctl -u fichub.service -n
   100` on the deploy box — what's the last error?
3. DB issue? `sudo -u postgres psql -d fichub -c "SELECT 1"`.
4. Redis issue? `redis-cli PING` — expect PONG. (Remember: a false
   redis:false used to happen from the shared-connection BRPOP; the fix is
   health_redis.)

**"A scrape fails for one URL."**
1. Check `scrape_failures` in the DB — what classification did the
   classifier assign?
2. Transient → retry. Blocked → likely a bot wall (cookies/proxy needed).
3. Structural → the site changed its HTML; the scraper needs a selector
   fix. This is a real bug, not an ops problem.

**"The search returns wrong results."**
1. Test the query in the UI, then check the parser's unit tests for the
   pattern.
2. Remember `main_char_attr` semantics — the attribute applies to the MAIN
   character.
3. Check `zero_result_queries` in admin search analytics — is this a known
   blind spot?

**"A DB-gated test fails mysteriously."**
1. Is it the incremental cache? (`rm -rf target/debug/incremental`)
2. Is it a leftover row from a crashed run? (unique seed names should
   isolate; clean up.)
3. Is it a parallel-suite collision? (run with `--test-threads=1`)

**"The frontend shows the wrong thing but the API is right."**
1. Check the API response first (`curl` the endpoint).
2. Check `client.ts` for the function the page calls — right URL? right
   token?
3. Check the `page.test.ts` — does the fixture match reality?

> 🧪 **Try it:** next time something breaks, follow the cheat-sheet
> top-to-bottom before diving into code. You'll find most problems in the
> first two steps.
>
> ⚠️ **Watch out:** the cheat-sheet assumes the health/telemetry systems
> are recording. If `scrape_failures` is empty when you expected a
> failure, check whether the route records it (a known telemetry gap for
> some paths).
>
> 💡 **Key concept:** triage by layers: health → logs → telemetry → code.
> Most issues resolve before you open an editor.

### Chapter 24B: The Docs You Should Read (and Keep Current)

FicHub takes documentation seriously — the house rule is "rich,
self-contained markdown fed to chatbots; features/decisions in markdown
not only code." The docs you'll use:

- **`docs/ROADMAP.md`** — the canonical plan: shipped, in-flight, and
  prioritized suggestions (P1-P7). Read this when deciding what to work
  on.
- **`docs/SPECIFICATION.md`** — the API spec + endpoint inventory. Check
  it before adding an endpoint.
- **`docs/STATUS.md` / `docs/TODO.md`** — session status + feature notes
  (superseded by ROADMAP for planning, still useful for history).
- **`docs/src/`** — the user-facing mdbook chapters (intro, searching,
  downloading, transparency, ...). Served at `/docs/`.
- **`docs/using-recommender-platform.md`** — how to use the rec platform.
- **The `fichub-development` skill** — the maintainer's playbook with
  references for every subsystem (in the Hermes profile, not the repo).

**The rule:** when you ship a feature, update the docs in the same commit.
ROADMAP gets the status change; SPECIFICATION gets the endpoint; the
mdbook gets a user-facing blurb if it's user-visible. Docs drift is a
review-blocking issue here.

> 🧪 **Try it:** pick a shipped feature (say, the modlog) and trace it
> through ROADMAP → SPECIFICATION → docs/src. See how the docs mirror the
> code.
>
> ⚠️ **Watch out:** don't add an endpoint without updating the
> SPECIFICATION's endpoint inventory. It's the contract.
>
> 💡 **Key concept:** docs are part of the deliverable. Ship code + docs
> together, or the review will bounce it.

---

### Chapter 25: Glossary of Terms You'll See in the Code

- **url_id** — the string key for a fic source (e.g. `ao3_21845264`,
  `xenforo_50062326`). Used everywhere as the primary identifier.
- **work** — the abstract story; a work has multiple *sources* (fic_info
  rows) if posted on several sites.
- **fic_info** — one source row: url_id, site_domain, title, author, etc.
- **main_char_attr** — a search filter with AO3 "Dark Harry" semantics:
  main character + attribute (e.g. `main_char_attr: dark harry potter`).
- **body cache** — the on-disk JSON blob store of scraped chapter bodies
  (`BODY_CACHE_DIR`).
- **modlog** — the transparent moderation log (migration 034).
- **usage_events** — the zero-PII analytics table (migration 033).
- **scrape_failures / agent_runs** — self-healing telemetry (migrations
  029-030).
- **health_redis** — the dedicated Redis connection for health checks
  (never share the worker's BRPOP connection).
- **CacheSemaphores** — bounded map of export semaphores (one concurrent
  export per url_id).
- **REC_ENGINE_MODE** — `legacy` (current cooccurrence engine) vs
  `pluggable` (strategy registry with decay/embeddings/mf/... strategies).
- **REC_SHADOW_MODE** — compute new strategies but return legacy output
  while logging impressions (safe A/B).
- **find_specific_or_fff** — registry helper preferring native scrapers
  over the FanFicFare catch-all.
- **X-Client-ID** — the anonymous browser UUID for usage analytics (never
  an IP).
- **AuthUser** — the JWT-backed auth extractor; `user_id != 0` means
  logged in, `role >= 10` means admin.
- **BODY_CACHE_DIR** — env knob for where scraped bodies live (default
  `/public/literature/fichub/bodies`).
- **hermes verify** — the canonical build+test+readiness gate that records
  verification evidence.

---

## Conclusion

You now know the shape of the whole system: one Axum binary serving both
API and SPA, a config struct that tames every env var, a query module that
owns the SQL, a scraper registry that prefers native adapters, a body
cache that makes the site a durable archive, and a social/governance layer
built on JWT auth, a transparent modlog, and zero-PII analytics.

The fastest way to learn the rest is to change something small. Pick a
chapter from "Where to Start Contributing," open the route file, add a
feature + a test, and run the targeted suite. The repo is well-commented,
the docs (`docs/ROADMAP.md`, `docs/SPECIFICATION.md`) map the territory,
and this book gives you the vocabulary.

Welcome aboard — the fics are waiting.
