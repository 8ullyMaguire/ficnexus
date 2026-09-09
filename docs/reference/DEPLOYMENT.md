# FicHub — Deployment Layout (2026-08-14)

**The service runs from LOCAL disk `/opt/fichub/fichub`, NOT from the NFS repo.**

## Why

Running the executable from `/personal` (fuse.mergerfs over 8 NFS mounts) made
the kernel **SIGBUS** the process whenever the NFS wedged — the code pages could
no longer be paged in from the stale mount. This killed the process mid-request:

- FicHub: recurring crash class (Aug 2026); the same class hit StreakForge on
  Aug 10/11/14, surfacing to users as **502 request failed on logging**.
- Moving the binary to local ext4 (`/opt`) makes the running process immune to
  NFS wedges. Data files (read once / recoverable) stay on NFS.

## Layout

| What | Where |
|------|-------|
| Binary | `/opt/fichub/fichub` (local NVMe ext4) |
| `.env` | `/opt/fichub/.env` (FRONTEND_DIR absolutized) |
| Frontend build (static) | `/personal/documents/code/rust/fichub/frontend/build` (NFS) |
| Caches / tmp / bodies | `/public/literature/...` (NFS) |
| Migrations | `/personal/documents/code/rust/fichub/migrations` (NFS, applied at deploy) |
| Repo (source) | `/personal/documents/code/rust/fichub` (NFS, shared with gamingpc) |

## Env var note

`/opt/fichub/.env` has `FRONTEND_DIR=/personal/documents/code/rust/fichub/frontend/build`
(absolute). The repo `.env` has it relative (`./frontend/build`) — **never copy
the repo .env over /opt's verbatim**; only the /opt copy works with
`WorkingDirectory=/opt/fichub`.

## Deploy

Always use `./deploy.sh` (or `./deploy.sh --skip-build`) from the repo root.
It builds on gamingpc, **stops fichub, copies the binary to `/opt/fichub/`,**
starts, runs migrations, restarts, and health-checks. Never `systemctl restart
fichub` alone after a build — the NFS binary is not what the service runs.

## QA from dev machine

ThinkCentre binds FicHub on localhost, so dev-machine QA needs an SSH tunnel:

```bash
ssh -L 18000:localhost:8000 -N -f thinkcentre
QA_BASE=http://localhost:18000 node qa/run.js
```

Use `QA_BASE` to target another reachable deployment. Do not use a direct
ThinkCentre LAN address when firewall policy blocks it; verify tunnel health
first with `curl http://localhost:18000/api/health`.

## Self-heal (NFS unwedge watchdog)

ThinkCentre runs these every 60s (systemd timers):

- `fichub-selfheal.service` / `.timer` → `/usr/local/sbin/fichub-selfheal.sh`
- `streakforge-selfheal.service` / `.timer` → `/usr/local/sbin/streakforge-selfheal.sh`

Each probes its service over localhost; on failure it recovers the `/personal`
mergerfs pool (`umount -l /personal` → restart `mergerfs-personal.service` →
restart `nfs-server` → `exportfs -r`) and restarts the service. Logs:
`/var/log/{fichub,streakforge}-selfheal.log`.

## sw.js fix (2026-08-14)

`serve_sw` now reads sw.js from `state.config.frontend_dir` (was hardcoded
`./frontend/build`), so it works regardless of WorkingDirectory. Test:
`cargo test --bin fichub server::tests::sw_served`.

## Migrations 052 + 053 (2026-08-17)

Applied automatically on service start (`sqlx::migrate!()`):

- **052** — Progressive customization: `user_features` (per-user feature activations), `user_levels` (XP/level tracking), `user_rank_progress` (rank history), `user_widgets` (dashboard layout), `user_views` (saved search views), `feature_gate_events` (activation audit). Feature gates seeded for 26 features.
- **053** — Extension platform: `user_recipes` (blend/filters/boost JSONB, curator_prior, is_active/is_public, installs counter), `extensions` (unified plugin manifest for recipes/themes/layouts/views), 3 recipe feature gates (`recipes.create`, `recipes.publish`, `recipes.gallery`).

Both migrations are idempotent (`CREATE TABLE IF NOT EXISTS`, `ON CONFLICT DO NOTHING`).
