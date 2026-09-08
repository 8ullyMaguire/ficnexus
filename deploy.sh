#!/usr/bin/env bash
# deploy.sh — build on gamingpc (fast compile host), deploy on thinkcentre.
#
# Layout: repo is NFS-shared between machines (/personal on thinkcentre,
# mounted read-write on gamingpc). The release binary built on gamingpc lands
# in the SAME shared tree, then is copied to /opt/fichub (LOCAL disk on the
# ThinkCentre) — the service runs from /opt since 2026-08-14 so a wedged NFS
# mount cannot SIGBUS the process (see docs/AGENTS.md "Deployment layout").
# "deploy" = copy binary to /opt + run migrations + restart + verify.
#
# Usage:
#   ./deploy.sh          build on gamingpc + sync /opt + restart + health check
#   ./deploy.sh --skip-build   just sync /opt + restart + health check
#
# Requirements: ssh gamingpc must work passwordless; .env sourced from repo.

set -euo pipefail
cd "$(dirname "$0")"

GAMINGPC="${GAMINGPC:-gamingpc}"
SERVICE=fichub
BIN=target/release/fichub
BIN_DIR=/opt/fichub
FRONTEND=frontend/build

echo "==> $(date) FicHub deploy"

if [ "${1:-}" != "--skip-build" ]; then
  echo "==> Building release on $GAMINGPC (shared NFS tree)..."
  ssh "$GAMINGPC" "cd /personal/documents/code/rust/fichub && unset CARGO_TARGET_DIR && cargo build --release --bin fichub --bin migrate --bin publish-scheduled" || {
    echo "ERROR: build on $GAMINGPC failed"; exit 1
  }
fi

echo "==> Verifying binary fresh..."
stat -c '%y %n' "$BIN"

echo "==> Copying binary to $BIN_DIR (local disk — service runs from here)..."
sudo mkdir -p "$BIN_DIR"
# Service must be stopped to overwrite a running binary.
sudo systemctl stop "$SERVICE" || true
sudo cp "$BIN" "$BIN_DIR/fichub"
sudo chmod +x "$BIN_DIR/fichub"

echo "==> Running database migrations (explicit deploy step)..."
sudo systemctl start "$SERVICE"
sleep 2
./target/release/migrate || { echo "ERROR: migrate failed"; exit 1; }

echo "==> Copying publish-scheduled binary..."
cp target/release/publish-scheduled "$BIN_DIR/publish-scheduled" 2>/dev/null || true
chmod +x "$BIN_DIR/publish-scheduled" 2>/dev/null || true

# Install systemd timer if unit files exist
if [ -f deploy/systemd/publish-scheduled.service ] && [ -f deploy/systemd/publish-scheduled.timer ]; then
  sudo cp deploy/systemd/publish-scheduled.service /etc/systemd/system/
  sudo cp deploy/systemd/publish-scheduled.timer /etc/systemd/system/
  sudo systemctl daemon-reload
  sudo systemctl enable --now publish-scheduled.timer
  echo "publish-scheduled timer enabled (runs every minute)"
fi

echo "==> Restarting $SERVICE..."
sudo systemctl restart "$SERVICE"
sleep 3
systemctl is-active --quiet "$SERVICE" || { echo "ERROR: $SERVICE not active"; exit 1; }

echo "==> Health check..."
curl -sf -o /dev/null -w 'root: %{http_code}\n' http://localhost:8000/ || { echo "ERROR: root not serving"; exit 1; }
curl -sf -o /dev/null -w 'sw.js: %{http_code}\n' http://localhost:8000/sw.js || true

echo "==> Running QA (post-deploy)..."
./qa.sh 2>&1 | tail -20 || echo "WARN: QA exited non-zero (check qa/bugs.db)"

echo "==> Deploy OK"
