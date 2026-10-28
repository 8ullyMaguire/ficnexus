# FicHub — task runner (just)
# Usage: just <recipe>  (see `just --list`)
#
# ficnexus is an exporter: story URL in, file out. There is no frontend build,
# no worker service and no community platform to ship — see
# docs/EXPORTER-SCOPE.md.

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

# unit + integration (the exporter suite is #[ignore]d without a DB)
test-all:
    cargo test

# DB-gated exporter tests (needs .env DB/Redis)
test-db:
    set -a; . ./.env; set +a; cargo test --test exporter_api -- --include-ignored --test-threads=1

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

# dependency security audit
audit:
    cargo audit

# unused deps scan
udeps:
    cargo udeps

# full pre-commit gate
lint:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

# ── Deploy (target: thinkcentre via ssh) ──────────────────────────────────
# Binary lands in /opt/fichub/fichub (systemd service `fichub`); sudo is
# passwordless on thinkcentre.

deploy-build:
    CARGO_TARGET_DIR=target cargo build --release

# build, stage binary via /tmp, swap into place with a rollback copy, restart,
# health-check (rolls the old binary back if health fails)
deploy: deploy-build
    scp target/release/fichub thinkcentre:/tmp/fichub_new
    ssh thinkcentre "sudo mv /tmp/fichub_new /tmp/fichub_stage && sudo chown root:root /tmp/fichub_stage && sudo chmod 755 /tmp/fichub_stage"
    ssh thinkcentre "sudo systemctl stop fichub && \
        sudo cp -a /opt/fichub/fichub /opt/fichub/fichub.bak && \
        sudo mv /tmp/fichub_stage /opt/fichub/fichub && \
        sudo systemctl start fichub"
    ssh thinkcentre "for i in 1 2 3 4 5; do sleep 1; curl -sf -o /dev/null http://localhost:8000/api/health && exit 0; done; echo 'health check FAILED — rolling back'; \
        sudo systemctl stop fichub && sudo mv /opt/fichub/fichub.bak /opt/fichub/fichub && sudo systemctl start fichub; exit 1"

# migrate: apply a .sql migration as postgres user (e.g. `just migrate 001_x.sql`)
migrate file:
    sudo -u postgres psql -d fichub -f {{file}}

# service status + health check
status:
    systemctl is-active fichub
    curl -s http://localhost:8000/api/health; echo

# ── Repo hygiene ───────────────────────────────────────────────────────────

# stale worktree report
worktrees:
    ~/bin/worktree-hygiene

# test coverage (llvm-cov) — needs cargo llvm-cov
coverage:
    cargo llvm-cov --lib
