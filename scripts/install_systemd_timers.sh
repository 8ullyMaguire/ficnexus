#!/usr/bin/env bash
#
# FicHub — one-shot install of all recurring background jobs as systemd
# oneshot services + timers. Run ON THE DEPLOY HOST (thinkcentre) as root.
#
#   sudo bash scripts/install_systemd_timers.sh
#
# What it wires up (mirrors the canonical scheduled jobs):
#   fichub-quests          Daily        00:05  assign-quests           (P3 gamification)
#   fichub-stats           Nightly      01:00  compute-stats           (admin_daily_stats)
#   fichub-leaderboards    Weekly        Sun   compute-leaderboards    (cached leaderboards)
#   fichub-bot-scorer      Hourly              bot-scorer              (bot score aggregation)
#   fichub-saved-search    Nightly      02:00  saved-search-watcher    (PN8 saved-search alerts)
#   fichub-db-backup       Nightly      03:30  pg_dump -> NFS          (ops)
#   fichub-selfheal        Every 60s           selfheal mount watch    (ops)
#
# Every bin is a sqlx CLI tool run against the same postgres DB (DATABASE_URL
# from the repo .env); selfheal + db-backup are small op shell scripts under
# /usr/local/sbin. Binaries must already be built: cargo build --release
# (target/release/assign-quests, compute-stats, compute-leaderboards,
# bot-scorer, saved-search-watcher).
#
# Install path of the repo differs between hosts (gamingpc dev=~/code,
# thinkcentre deploy=/personal/documents/code). Detect it, or override:
#   FICHUB_REPO=/path/to/fichub sudo -E bash scripts/install_systemd_timers.sh
#
set -euo pipefail

FICHUB_REPO="${FICHUB_REPO:-}"
if [[ -z "$FICHUB_REPO" ]]; then
    for cand in /personal/documents/code/rust/fichub /home/alvaro/code/rust/fichub; do
        if [[ -f "$cand/.env" && -x "$cand/target/release/compute-stats" ]]; then
            FICHUB_REPO="$cand"; break
        fi
    done
fi
[[ -n "$FICHUB_REPO" ]] || { echo "Cannot find fichub repo. Set FICHUB_REPO="; exit 1; }

RUN_USER="$(stat -c '%U' "$FICHUB_REPO/.env" 2>/dev/null || echo alvaro)"
BIN_DIR="$FICHUB_REPO/target/release"
# Prefer the deploy-host env (/opt/fichub/.env) when present — the repo .env
# on the deploy host can drift stale (DB password rotates). /opt/.env is the
# single canonical env the main fichub.service sources.
ENV_FILE="/opt/fichub/.env"
if [[ ! -f "$ENV_FILE" ]]; then ENV_FILE="$FICHUB_REPO/.env"; fi
echo "using env: $ENV_FILE"

write_unit() { # $1=unitname  $2=content
    printf '%s\n' "$2" > "/etc/systemd/system/$1"
    chmod 644 "/etc/systemd/system/$1"
}

mk_service() { # $1=name  $2=Description  $3=bin  $4=binName
    write_unit "$1.service" "\
[Unit]
Description=$2
After=postgresql.service
Wants=postgresql.service
[Service]
Type=oneshot
User=$RUN_USER
Group=$RUN_USER
WorkingDirectory=$FICHUB_REPO
EnvironmentFile=$ENV_FILE
ExecStart=$BIN_DIR/$4
StandardOutput=journal
StandardError=journal
"
}

mk_timer() { # $1=name  $2=Description  $3=OnCalendar
    write_unit "$1.timer" "\
[Unit]
Description=$2
[Timer]
OnCalendar=$3
Persistent=true
[Install]
WantedBy=timers.target
"
}

# --- assign quests (daily 00:05) ---
mk_service fichub-quests "FicHub Daily Quest Assignment"              assign-quests      assign-quests
mk_timer   fichub-quests "Run FicHub Quest Assignment Daily at 00:05" "*-*-* 00:05:00"

# --- stats (nightly 01:00) ---
mk_service fichub-stats "FicHub Daily Stats Computation"              compute-stats      compute-stats
mk_timer   fichub-stats "Run FicHub Stats Nightly at 01:00"           "*-*-* 01:00:00"

# --- leaderboards (weekly Sun 01:00) ---
mk_service fichub-leaderboards "FicHub Leaderboard Computation"         compute-leaderboards compute-leaderboards
mk_timer   fichub-leaderboards "Run FicHub Leaderboard Weekly Sun 01:00" "Sun *-*-* 01:00:00"

# --- bot scorer (hourly) ---
mk_service fichub-bot-scorer "FicHub Bot Score Aggregation (hourly)"  bot-scorer         bot-scorer
mk_timer   fichub-bot-scorer "Run FicHub Bot Score Aggregation Hourly" "hourly"

# --- saved-search alert watcher (nightly 02:00, after quests+stats) ---
mk_service fichub-saved-search "FicHub Saved-Search Alert Watcher"             saved-search-watcher saved-search-watcher
mk_timer   fichub-saved-search "Run FicHub Saved-Search Watcher Nightly 02:00" "*-*-* 02:00:00"

# --- self-heal (mount watchdog, op script) ---
write_unit fichub-selfheal.service "\
[Unit]
Description=FicHub self-heal: recover service when /personal mount drops
After=network-online.target fichub.service
[Service]
Type=oneshot
ExecStart=/usr/local/sbin/fichub-selfheal.sh
TimeoutStartSec=120
"
write_unit fichub-selfheal.timer "\
[Unit]
Description=Run FicHub self-heal check every 60s
[Timer]
OnBootSec=120
OnUnitActiveSec=60
AccuracySec=5
[Install]
WantedBy=timers.target
"

# --- db backup (nightly 03:30, op script) ---
write_unit fichub-db-backup.service "\
[Unit]
Description=FicHub database nightly backup (pg_dump to NFS pool)
After=postgresql.service
[Service]
Type=oneshot
ExecStart=/usr/local/sbin/fichub-db-backup.sh
StandardOutput=journal
StandardError=journal
"
write_unit fichub-db-backup.timer "\
[Unit]
Description=Run FicHub database backup nightly at 03:30
[Timer]
OnCalendar=*-*-* 03:30:00
Persistent=true
RandomizedDelaySec=300
[Install]
WantedBy=timers.target
"

systemctl daemon-reload
for t in fichub-quests fichub-stats fichub-leaderboards fichub-bot-scorer \
         fichub-saved-search fichub-db-backup fichub-selfheal; do
    systemctl enable --now "$t.timer"
done

echo "DONE. Active timers:"
systemctl list-timers --no-pager | grep fichub || true
