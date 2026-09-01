## Part 6A — Deployment & Operations Deep Dive

Knowing how the product runs in production makes you a better developer of
it. This part covers the deploy pipeline, the ops gotchas, and what "done"
means here.

### Chapter 23A: The Deploy Pipeline

The production box is **ThinkCentre** (hostname `thinkcentre`), serving
`fichub.polarisocial.xyz`. The deploy path is deliberately simple:

1. **Build the backend release** on the dev machine (or the deploy box):
   ```bash
   cargo build --release --bin fichub
   ```
2. **Copy the binary** to the deploy box (or build there).
3. **Restart the service**:
   ```bash
   sudo systemctl restart fichub.service
   ```
4. **Migrations apply on boot** — the binary runs `sqlx::migrate!()` and
   applies any new migrations transactionally.
5. **Verify**: `curl https://fichub.polarisocial.xyz/api/health` shows
   `{"status":"ok","db":true,"redis":true}`, and the changed endpoint
   behaves.

The frontend: `cd frontend && npm run build` writes to `frontend/build`,
which the backend serves via ServeDir. Because the build dir is on shared
NFS, updating static files can be as simple as rebuilding — the running
service reads from disk. (A full restart picks up backend changes.)

**Key production config** (from the service env):

- `BODY_CACHE_DIR=/public/literature/fichub/bodies` — the body cache lives
  on the attached drive, NOT the DB.
- `REC_ENGINE_MODE=legacy` — the legacy rec engine is the live default.
- `AGENT_ENABLED=false` — the self-healing agent is off (diagnose-only).
- The Ollama default model is `lfm2.5:8b`.

> 🧪 **Try it:** on the deploy box, `systemctl status fichub.service`
> shows the service; `journalctl -u fichub.service -n 50` shows startup
> logs including migration output.
>
> ⚠️ **Watch out:** never build the backend on the deploy box's `/home`
> (NFS/space). Use the local ext4 target dir, as on dev.
>
> 💡 **Key concept:** deploy = build → copy → restart → migrations →
> verify. The binary carries its own schema.

### Chapter 23B: The Ops Gotchas (Learned the Hard Way)

These are documented because they bit the team — read them once and you'll
save yourself hours.

**1. The NFS git-objects quirk.** The repo lives on NFS. Git refs
repeatedly dangle after commits/merges. The recipe (from the workflow
chapter): set `GIT_OBJECT_DIRECTORY`, commit, copy objects into
`.git/objects`, unset, verify with `git cat-file -t <sha>`. If `git
cat-file` fails, the objects didn't land.

**2. The mergerfs wedge.** The deploy box's storage pool (mergerfs) can
wedge under heavy I/O. Recovery: `sudo umount -l /personal` +
`sudo systemctl restart mergerfs-personal.service` +
`sudo systemctl restart nfs-kernel-server`, then remount. If dev's
`/personal` goes EIO, this is the fix.

**3. The shared NFS cargo target.** `target/debug/incremental` on the
shared NFS target gets stale rlib fingerprints after mount wedges,
producing "required to be available in rlib format, but was not found"
errors. Fix: `rm -rf target/debug/incremental` and rebuild.

**4. The false "Redis down" report.** The admin dashboard once showed
Redis 🔴 when Redis was fine. Root cause: the health check PINGed the
shared `state.redis` connection, which the bookmark-import worker had
parked in an unbounded BRPOP — the PING timed out. Fix: dedicated
`health_redis` connection. **Rule: never PING the shared connection from a
health check.**

**5. The `.env` file.** Dev `.env` contains `DATABASE_URL`, `JWT_SECRET`,
`REDIS_URL`, etc. Worktrees ship only `.env.example` — copy the real one
for DB-gated tests. Secrets are in `.env`, never in test specs.

> 🧪 **Try it:** pick one gotcha and find the code/script that addresses
> it (e.g. the `health_redis` field in `AppState`).
>
> ⚠️ **Watch out:** if the DB-gated tests fail with weird rlib errors,
> it's the incremental cache (gotcha 3), not your code. Clear it first.
>
> 💡 **Key concept:** ops problems are deterministic once understood. The
> gotchas above have known recipes — follow them, don't improvise.

### Chapter 23C: What "Done" Means

A feature is done when it passes the house gates:

1. **Targeted tests pass** — the suite you touched, not the full suite
   (`cargo test --test <suite>` or `vitest run <file>`).
2. **The canonical verify passes** — `hermes verify --save` runs
   build → test → boot → readiness and records evidence. (On this setup,
   clear `target/debug/incremental` first if the shared target is stale.)
3. **Deployed + live-verified** — the binary is on the deploy box, the
   service restarted, migrations applied, and the changed endpoint checked
   via the public URL.
4. **Committed conventionally + pushed** — small meaningful commits,
   `feat:`/`fix:`/`test:`/`docs:`, mirror pushed to the opencommit.eu
   remote.

**The test conventions again, because they matter:**

- Never run the FULL cargo suite in normal dev (10 min).
- DB-gated suites: `#[ignore]` + `--include-ignored --test-threads=1`.
- Frontend: `page.test.ts` per route; `npx vitest run <file>`.

> 🧪 **Try it:** look at a recent commit message in `git log --oneline` and
> classify each by type. Notice how small the units are.
>
> ⚠️ **Watch out:** "works on my machine" isn't done. The verify + deploy
> steps are what make a feature real here.
>
> 💡 **Key concept:** done = tests + verify + deploy + commit. The bar is
> mechanical, not aspirational.

### Chapter 23D: The Live Systems You Can Inspect

The live site is a great learning tool. Things to try on
`fichub.polarisocial.xyz`:

- `/api/health` — the liveness check (db + redis booleans).
- `/roadmap` — the MaxDiff/Elo consensus arena (vote!).
- `/modlog` — the public moderation log (read it as any user).
- `/admin/analytics` — the zero-PII usage dashboard (admin only).
- `/docs/` — the user-facing docs (mdbook).
- `/ask` — Ask the Archive (natural-language search).
- `/requests` — the fic-request prompt board.
- `/read/<url_id>` — the web reader (offline-capable via PWA).

> 🧪 **Try it:** pick three of these and trace each back to its backend
> route in `build_router()`. You'll see the whole map again, this time
> live.
>
> ⚠️ **Watch out:** the live site is production — don't spam endpoints or
> create junk data while exploring. Use the dev server for experiments.
>
> 💡 **Key concept:** the live site is the product's truth. When in
> doubt about how something behaves, check production behavior — then
> find the code that produced it.

---
