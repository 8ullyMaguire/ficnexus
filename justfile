# FicHub — task runner (just)
# Usage: just <recipe>  (see `just --list`)

set shell := ["bash", "-c"]
set dotenv-load := false

# ── Build / dev ────────────────────────────────────────────────────────────

# fast compile check (no codegen)
check:
    cargo check --workspace

# build the release binary (deploy artifact)
build:
    cargo build --release

# run the dev server (default bin = fichub)
dev:
    cargo run

# run the frontend dev server (SvelteKit/Vite)
dev-frontend:
    cd frontend && npm run dev

# watch: cargo check on every .rs change
watch-check:
    cargo watch -x check --workspace

# watch: cargo test on every .rs change
watch-test:
    cargo watch -x "test --lib"

# ── Test ───────────────────────────────────────────────────────────────────

# unit tests (fast, no DB)
test:
    cargo test --lib

# all unit + doc tests
test-all:
    cargo test

# DB-gated integration tests (needs .env DB/Redis); run suites individually
test-db:
    cargo test -- --include-ignored --test-threads=1

# a single DB-gated suite by name, e.g. `just test-suite requests_api`
test-suite suite:
    cargo test --test {{suite}} -- --include-ignored --test-threads=1

# frontend unit tests
test-frontend:
    cd frontend && npm test

# frontend E2E (15/15 green baseline; add new routes to +layout.svelte routePages)
test-e2e:
    cd frontend && npm run test:e2e

# everything fast (unit + frontend unit)
test-quick:
    cargo test --lib
    cd frontend && npm test

# ── Lint / format / audit ──────────────────────────────────────────────────

# rustfmt check
fmt:
    cargo fmt --check

# fix formatting in place
fmt-fix:
    cargo fmt

# clippy with warnings as errors
clippy:
    cargo clippy --workspace -- -D warnings

# frontend TS/JS check
svelte-check:
    cd frontend && npx svelte-check

# dependency security audit
audit:
    cargo audit

# unused deps scan
udeps:
    cargo udeps

# full pre-commit gate (what pre-commit runs)
lint:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings
    cd frontend && npx svelte-check

# ── Deploy (target: thinkcentre via ssh) ──────────────────────────────────
# Binary lands in /opt/fichub/fichub (systemd service `fichub`); frontend
# build is rsynced to /personal/documents/code/rust/fichub/frontend/build/.
# sudo is passwordless on thinkcentre.

# build release + frontend build (what the deploy_* recipes consume)
deploy-build:
    CARGO_TARGET_DIR=target cargo build --release
    cd frontend && npm run build

# rsync the SvelteKit static build to thinkcentre
deploy-frontend: deploy-build
    rsync -avz --delete frontend/build/ thinkcentre:/personal/documents/code/rust/fichub/frontend/build/

# build, stage binary via /tmp, swap into place with a rollback copy, restart,
# health-check (rolls the old binary back if health fails)
deploy-backend: deploy-build
    scp target/release/fichub thinkcentre:/tmp/fichub_new
    ssh thinkcentre "sudo mv /tmp/fichub_new /tmp/fichub_stage && sudo chown root:root /tmp/fichub_stage && sudo chmod 755 /tmp/fichub_stage"
    ssh thinkcentre "sudo systemctl stop fichub && \
        sudo cp -a /opt/fichub/fichub /opt/fichub/fichub.bak && \
        sudo mv /tmp/fichub_stage /opt/fichub/fichub && \
        sudo systemctl start fichub"
    ssh thinkcentre "for i in 1 2 3 4 5; do sleep 1; curl -sf -o /dev/null http://localhost:8000/health && exit 0; done; echo 'health check FAILED — rolling back'; \
        sudo systemctl stop fichub && sudo mv /opt/fichub/fichub.bak /opt/fichub/fichub && sudo systemctl start fichub; exit 1"

# build everything, ship backend + frontend, verify health
deploy-full: deploy-frontend deploy-backend
    @ssh thinkcentre "curl -s -o /dev/null -w 'health: HTTP %{http_code}\n' http://localhost:8000/health && systemctl is-active fichub"

# migrate: apply a .sql migration as postgres user (e.g. `just migrate 001_x.sql`)
migrate file:
    sudo -u postgres psql -d fichub -f {{file}}

# service status + health check
status:
    systemctl is-active fichub
    curl -s -o /dev/null -w "HTTP %{http_code}\n" http://localhost:8000/ || true

# ── Repo hygiene ───────────────────────────────────────────────────────────

# stale worktree report
worktrees:
    ~/bin/worktree-hygiene

# cargo bloat (release binary size breakdown)
bloat:
    cargo bloat --release

# benchmark (criterion) — run once weekly via cron
bench:
    cargo bench

# test coverage (llvm-cov) — needs cargo llvm-cov
coverage:
    cargo llvm-cov --lib
