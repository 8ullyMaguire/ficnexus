# Part 46 — Deployment

You've built every feature of FicHub. You added endpoints, wrote frontend pages, wired up the database and Redis, set up the recommendation engine, and got tests passing. But none of that matters unless you can **ship it to the real world** — where real users download real stories 24 hours a day.

This is Part 19. We pack your Rust binary into a Docker image, spin up PostgreSQL + Redis + Ollama with Docker Compose, write a deploy script that swaps in a new binary without downtime, and configure Nginx as a reverse proxy so the outside world can reach your site. By the end, you will understand the *entire* deployment pipeline from source code to production server.

---

## 46.1 Dockerfile — multi-stage Rust build

### Goal

Docker can't run Rust source code directly — it needs a compiled binary. But the Rust toolchain alone is over 2 GB. We use a **multi-stage build**: the first stage compiles the binary with the full Rust toolchain, and the second stage starts from a tiny `debian:bookworm-slim` image and copies in only the finished binary plus its runtime dependencies. The result is an image that's a fraction of the size.

Open the `Dockerfile`:

```dockerfile
# FicHub — Docker image with Rust binary + FanFicFare
# Multi-stage build for minimal image size

# Stage 1: Build the Rust binary
FROM rust:bookworm AS builder

WORKDIR /app
COPY . .

# Install cross-compilation tools for ARM64 (optional, for Pi builds)
RUN apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/*

# Build for the host architecture (amd64)
RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Install FanFicFare via pip in a virtual environment
RUN python3 -m venv /opt/fanficfare \
    && /opt/fanficfare/bin/pip install --no-cache-dir fanficfare \
    && ln -s /opt/fanficfare/bin/fanficfare /usr/local/bin/fanficfare

# Copy the built binary from builder
COPY --from=builder /app/target/release/fichub /usr/local/bin/fichub

# Copy migrations directory
COPY --from=builder /app/migrations /app/migrations

# Create cache and temp directories
RUN mkdir -p /public/literature/fanfiction/archive/fichub/cache \
    && mkdir -p /public/literature/fanfiction/archive/fichub/tmp \
    && mkdir -p /tmp/fichub_fff

# Environment variables
ENV DATABASE_URL=postgres://fichub:***@localhost:5432/fichub
ENV REDIS_URL=redis://localhost:6379
ENV CACHE_DIR=/public/literature/fanfiction/archive/fichub
ENV PORT=8004
ENV RUST_LOG=info

EXPOSE 8004

VOLUME ["/public/literature/fanfiction/archive/fichub"]

ENTRYPOINT ["fichub"]
```

### Breakdown

**Stage 1 — the builder**: `FROM rust:bookworm AS builder` pulls in the full Rust toolchain (rustc, cargo, std library). This image is large but that's fine — it's only used during the build. `COPY . .` copies the entire repository, and `RUN cargo build --release` compiles the `fichub` binary with optimizations.

**Why arm64 cross tools?** The `RUN apt-get install -y gcc-aarch64-linux-gnu` line installs a cross-compiler so the same Dockerfile can produce an ARM64 binary (for Raspberry Pi / Orange Pi deployments). A separate `docker/cross-build.Dockerfile` handles pure cross-compilation builds (it adds the `aarch64-unknown-linux-gnu` target and passes the linker).

**Stage 2 — the runtime**: `FROM debian:bookworm-slim` is a minimal Debian image — just the base system, no compilers, no Rust. This is what ships to production. We install only what the binary needs at runtime:

- `ca-certificates` — for HTTPS (TLS root certificates)
- `libssl3` — the OpenSSL shared library that `rustls` / `reqwest` may need
- `curl` — handy for health checks from inside the container
- `python3`, `python3-pip`, `python3-venv` — **FanFicFare**, the Python-based fanfiction scraper that acts as a fallback for sites without a native Rust scraper

**FanFicFare in a venv**: The virtual environment at `/opt/fanficfare` isolates FanFicFare's Python packages from the system Python. The `pip install --no-cache-dir` flag skips caching downloaded packages, keeping the image smaller. The `ln -s` symlink makes `fanficfare` callable from anywhere on `PATH`.

**Copying the binary**: `COPY --from=builder` takes *only* the compiled binary from the first stage — none of the Rust toolchain, none of the intermediate build artifacts.

**Environment variables**: The `ENV` lines set sensible defaults. In production, these are overridden by `docker-compose.yml` or your orchestration system (Kubernetes, systemd, etc.). Note `PORT=8004` — this matches the port in `deploy.sh`'s health check.

**Volume**: `VOLUME ["/public/literature/fanfiction/archive/fichub"]` declares that the cache directory should be a Docker volume, so cached fic bodies and exports persist across container restarts. Without this, every container restart would wipe your cache and force a full re-scrape.

### Try It Yourself: build the image locally

```bash
cd ~/code/rust/fichub
docker build -t fichub:latest .
```

Watch the two-stage output — you'll see the builder stage compile Rust, then the runtime stage install Python and copying the binary. Once it's done:

```bash
docker run --rm -e DATABASE_URL=postgres://fichub:***@host.docker.internal:5432/fichub \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -p 8004:8004 \
  fichub:latest
```

**Check**: `docker images fichub` should show an image much smaller than the raw `rust:bookworm` base. The binary alone is ~30 MB compressed.

### Troubleshooting

- **"cargo: command not found" during build**: You're in Stage 2 and trying to run `cargo`. Make sure you're in the builder stage — `FROM rust:bookworm AS builder` must be the first line.
- **FanFicFare fails to install**: The Python venv may not have `pip` available. Ensure `python3-venv` is installed (it's in the runtime deps). If the site you're scraping is behind Cloudflare, FanFicFare alone won't help — you may need a different scraper.
- **`error while loading shared library libssl.so.3`**: The `libssl3` package wasn't installed. Check that the `apt-get install` line in Stage 2 includes `libssl3`.

---

## 46.2 docker-compose.yml — PostgreSQL, Redis, Ollama, FicHub

### Goal

FicHub doesn't run alone — it needs PostgreSQL for data, Redis for rate limiting and caching, and Ollama for AI features (roadmap consensus, comment triage, embeddings). Docker Compose orchestrates all these containers so you can bring up the entire stack with one command.

Open `docker-compose.yml`:

```yaml
services:
  calibre:
    build:
      context: .
      dockerfile: docker/calibre.Dockerfile
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    network_mode: host

  app:
    build: .
    network_mode: host
    environment:
      DATABASE_URL: postgres://fichub:***@localhost:5432/fichub
      REDIS_URL: redis://localhost:6379
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      CALIBRE_CONTAINER: calibre
      PORT: 8004
      FRONTEND_DIR: /app/frontend
      RUST_LOG: info,fichub=debug
      JWT_SECRET: bbcc90f26b1985701d4e0e183d1013e747995ad7f9f06482fd7a9a0f72c105750ad4a5ec8ba610babd13f9031458064883574318477d92130a1764c19ce9a6cc
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      - calibre

volumes:
  cache:
  tmp:
```

### Breakdown

**`network_mode: host`**: Both services use host networking — they don't get their own IP addresses; they share the host's network stack. This means `DATABASE_URL` points to `localhost:5432` and `REDIS_URL` points to `localhost:6379`. Everything (PostgreSQL, Redis, Ollama) must be running directly on the host machine or in another host-networked container. This avoids the complexity of Docker bridge networks and DNS — the services talk to each other over `localhost`.

**Why host mode?** FicHub's scraper makes hundreds of outbound HTTP requests per second. Docker's bridge networking adds latency and connection-tracking overhead. Host mode gives bare-metal network performance. The tradeoff is that you must ensure ports 5432 (PostgreSQL), 6379 (Redis), 11434 (Ollama), and 8004 (FicHub) don't conflict with other services.

**`calibre` service**: This is a sidecar container that runs the Calibre ebook management software. FicHub uses it for format conversion (e.g., converting EPUB to MOBI for Kindle). It shares the `cache` and `tmp` volumes with the app so converted files are accessible. The `docker/calibre.Dockerfile` is minimal:

```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    calibre \
    && rm -rf /var/lib/apt/lists/*
CMD ["tail", "-f", "/dev/null"]
```

**`CALIBRE_CONTAINER: calibre`**: This environment variable tells the FicHub binary the container name so it can invoke `docker exec` into the Calibre container for conversions.

**The `cache` and `tmp` volumes**: These are Docker named volumes. `cache` holds downloaded fic files and export outputs. `tmp` holds temporary files during export. Using named volumes (instead of bind mounts) means Docker manages the storage lifecycle — you can `docker volume prune` to clean up unused volumes.

### What's missing from this compose file

The docker-compose.yml in the repo does **not** include PostgreSQL, Redis, or Ollama as services. In the development setup, these run separately on the host. Here's what a **complete** development docker-compose.yml would look like — with those services added:

```yaml
services:
  app:
    build: .
    ports:
      - "8004:8004"
    environment:
      DATABASE_URL: postgres://fichub:fichub@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      OLLAMA_URL: http://ollama:11434
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      PORT: 8004
      FRONTEND_DIR: /app/frontend
      RUST_LOG: info,fichub=debug
      JWT_SECRET: bbcc90f26b1985701d4e0e183d1013e747995ad7f9f06482fd7a9a0f72c105750ad4a5ec8ba610babd13f9031458064883574318477d92130a1764c19ce9a6cc
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
      ollama:
        condition: service_started

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: fichub
      POSTGRES_PASSWORD: fichub
      POSTGRES_DB: fichub
    volumes:
      - pg_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d/migrations
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama

volumes:
  cache:
  tmp:
  pg_data:
  redis_data:
  ollama_data:
```

**PostgreSQL**: Uses the official `postgres:16-alpine` image. The `healthcheck` runs `pg_isready` to confirm the server is accepting connections before FicHub starts. Migrations are mounted from the host `migrations/` directory.

**Redis**: Uses `redis:7-alpine`. The `--appendonly yes` flag enables AOF (Append Only File) persistence, so rate-limit counters and cached data survive restarts. Without persistence, every Redis restart would reset all rate-limit buckets.

**Ollama**: Provides the local AI backend for embeddings (Roadmap Consensus) and comment triage. The `nomic-embed-text` and `lfm2.5:8b` models (configured in `src/config.rs` via `OLLAMA_EMBED_MODEL` and `OLLAMA_CHAT_MODEL`) need to be pulled after the container starts:

```bash
docker compose exec ollama ollama pull nomic-embed-text
docker compose exec ollama ollama pull lfm2.5:8b
```

### The `.env.example` file

FicHub reads its configuration from environment variables. The `.env.example` file is the template:

```env
DATABASE_URL=postgres://fichub:***@localhost:5432/fichub
REDIS_URL=redis://localhost:6379
CACHE_DIR=./cache
SECONDARY_CACHE_DIR=
EXPORT_VERSION=1
DYNAMIC_RATE_LIMIT=true
NODE_NAME=orion
CALIBRE_CONTAINER=
PORT=3000
FRONTEND_DIR=./frontend/build
TMP_DIR=./tmp
# Optional: path to a MaxMind GeoLite2-ASN .mmdb file. When set, the rate
# limiter blocks requests from datacenter/hosting ASNs (1h retry-after).
# Unset or unreadable = datacenter blocking disabled (fail-open).
MAXMIND_DB=
RUST_LOG=info,fichub=debug
```

Copy it to `.env` and tweak:

```bash
cp .env.example .env
# Edit .env to set your DATABASE_URL with a real password, PORT, etc.
```

### Try It Yourself: bring up the stack

```bash
cd ~/code/rust/fichub
cp .env.example .env
# Edit .env: set DATABASE_URL to a real postgres password
docker compose up -d
```

Then watch FicHub boot:

```bash
docker compose logs -f app
```

You should see the tracing logs from `src/main.rs`:

```rust
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,fichub=debug")),
        )
        .init();
    let config = config::Config::from_env();
    tracing::info!("Starting fichub-rs server on port {}", config.app_port);
    server::run(config).await;
}
```

**Check**: Once the container is running, hit the health endpoint:

```bash
curl -s http://localhost:8004/api/health | python3 -m json.tool
```

Expected (200 OK):
```json
{
  "status": "ok",
  "db": true,
  "redis": true,
  "version": "0.2.0"
}
```

The health handler in `src/routes/health.rs` pings PostgreSQL with `SELECT 1` and Redis with `PING`, using a **dedicated** `health_redis` connection (not the shared one used by background workers, which can block on `BRPOP`). If either is down, it returns `degraded` or `error` with HTTP 503.

### Troubleshooting

- **`DATABASE_URL must be set`**: FicHub panics at startup if this is missing (see `src/config.rs` — `std::env::var("DATABASE_URL").expect(...)`). Make sure `.env` is in the working directory or the variable is set in the compose file.
- **`redis:false` in health**: The health check uses `state.health_redis` with a 2-second timeout. If Redis is under heavy load (lots of rate-limiting traffic), the 2s timeout may expire. Check `redis-cli ping` from inside the container.
- **Port 8004 already in use**: The `PORT` env var controls which port `src/main.rs` binds to. If you change it, update both the compose file and your Nginx config.
- **FanFicFare not found**: The Dockerfile installs it at `/usr/local/bin/fanficfare`. If you're running outside Docker, install it manually: `pip install fanficfare`. The config `CALIBRE_CONTAINER` must be empty (or the container name) depending on your setup.
- **Ollama model not pulled**: Comment triage and embeddings will fail silently if the model isn't available. The `OllamaClient` in `src/services/ollama.rs` retries and degrades gracefully — features just fall back to defaults.

---

## 46.3 deploy.sh — production deploy script

### Goal

In development we use `docker compose up`. In production, FicHub runs as a **systemd service** on a real server (the ThinkCentre), with its binary on local NVMe disk and data files on NFS. The `deploy.sh` script automates the entire release process: build, copy, migrate, restart, and health-check — all with rollback on failure.

Read the full `deploy.sh`:

```bash
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
  ssh "$GAMINGPC" "cd /personal/documents/code/rust/fichub && unset CARGO_TARGET_DIR && cargo build --release --bin fichub --bin migrate" || {
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
```

### Breakdown

**Why build on gamingpc?** The ThinkCentre is the production server, but the gaming PC has a faster CPU and NVMe cache. Both machines share the same NFS-mounted repo tree at `/personal`. The binary is compiled on the fast machine, lands in the shared tree, and gets copied to local disk on the ThinkCentre.

**The NFS problem**: Before 2026-08-14, the systemd service ran the binary directly from `/personal` (the NFS mount). If NFS wedged, the kernel would `SIGBUS` the process — the code pages could no longer be paged in from the stale mount. This killed FicHub mid-request, producing 502s. The fix: copy the binary to `/opt/fichub/fichub` on **local ext4 disk** before starting the service. See `docs/DEPLOYMENT.md` for the full layout.

**Key environment variables**:

- `GAMINGPC` — defaults to `"gamingpc"` (must resolve via SSH). Change with `GAMINGPC=other-host ./deploy.sh`.
- `BIN=target/release/fichub` — the release binary path. The `--bin fichub` flag ensures we only build the `fichub` binary (not the other `[[bin]]` targets like `assign-quests`, `compute-stats`, etc. from `Cargo.toml`).
- `BIN_DIR=/opt/fichub` — the local-disk destination. The service's `WorkingDirectory` is set to `/opt/fichub` in the systemd unit, so `FRONTEND_DIR` must be an absolute path there (see the note in `docs/DEPLOYMENT.md`).

**The deploy flow:**

1. **Build**: `cargo build --release --bin fichub --bin migrate` runs on gamingpc via SSH. The `--bin migrate` flag builds the migration runner binary too — deploy.sh runs it explicitly after the binary swap.
2. **Stop**: `sudo systemctl stop fichub` halts the running service so the binary file can be overwritten.
3. **Copy**: `sudo cp "$BIN" "$BIN_DIR/fichub"` copies the fresh binary from the NFS-shared tree to local disk. `chmod +x` ensures it's executable.
4. **Start + migrate**: The service is started, then `./target/release/migrate` runs pending SQL migrations from `migrations/`. SQLx's `migrate!()` macro also applies migrations automatically on service startup, but this explicit step catches migration failures before the health check.
5. **Restart**: `systemctl restart fichub` ensures the service picks up the new binary.
6. **Health check**: `curl -sf -o /dev/null http://localhost:8000/` verifies the server responds. If this fails, the script exits non-zero.
7. **QA**: `./qa.sh` runs the full QA harness (`qa/run.js`, `qa/api-walk.js`) and reports any regressions.

**The systemd service** (on the ThinkCentre) looks like:

```ini
[Unit]
Description=FicHub Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/opt/fichub
ExecStart=/opt/fichub/fichub
Restart=on-failure
RestartSec=5
EnvironmentFile=/opt/fichub/.env

[Install]
WantedBy=multi-user.target
```

Note: the `.env` in `/opt/fichub/` must have `FRONTEND_DIR` set to an **absolute path** (e.g., `/personal/documents/code/rust/fichub/frontend/build`), because the service's `WorkingDirectory` is `/opt/fichub`, not the NFS repo root. Never copy the repo `.env` verbatim — the relative `./frontend/build` path would resolve to `/opt/fichub/frontend/build`, which is empty.

**Rollback**: The `justfile` contains a more robust `deploy-backend` recipe that keeps a backup and rolls back on health-check failure:

```makefile
deploy-backend: deploy-build
    scp target/release/fichub thinkcentre:/tmp/fichub_new
    ssh thinkcentre "sudo mv /tmp/fichub_new /tmp/fichub_stage && \
        sudo chown root:root /tmp/fichub_stage && \
        sudo chmod 755 /tmp/fichub_stage"
    ssh thinkcentre "sudo systemctl stop fichub && \
        sudo cp -a /opt/fichub/fichub /opt/fichub/fichub.bak && \
        sudo mv /tmp/fichub_stage /opt/fichub/fichub && \
        sudo systemctl start fichub"
    ssh thinkcentre "for i in 1 2 3 4 5; do \
        sleep 1; curl -sf -o /dev/null http://localhost:8000/health && exit 0; \
    done; \
    echo 'health check FAILED — rolling back'; \
    sudo systemctl stop fichub && \
    sudo mv /opt/fichub/fichub.bak /opt/fichub/fichub && \
    sudo systemctl start fichub; exit 1"
```

This keeps `fichub.bak` as a rollback copy. If the health check fails 5 times (5 seconds), it swaps the backup back in.

### Self-heal watchdog

When NFS wedges, it's not enough to just restart FicHub — the NFS mount itself needs recovery. The ThinkCentre runs a systemd timer every 60 seconds (`fichub-selfheal.service` / `.timer`) that probes the service over localhost and, if down, performs an NFS recovery:

```bash
# /usr/local/sbin/fichub-selfheal.sh (simplified)
if ! curl -sf -o /dev/null http://localhost:8000/api/health; then
    echo "==> FicHub down, attempting NFS recovery..."
    umount -l /personal           # lazy-unmount the stale mergerfs pool
    systemctl restart mergerfs-personal.service
    systemctl restart nfs-server
    exportfs -r                  # re-export the shares
    systemctl restart fichub
fi
```

Logs go to `/var/log/fichub-selfheal.log`. The same pattern runs for StreakForge.

### Try It Yourself: simulate a deploy

On your dev machine, you can test the deploy flow locally:

```bash
cd ~/code/rust/fichub

# 1. Build the release binary
cargo build --release --bin fichub --bin migrate

# 2. Verify the binary exists
ls -la target/release/fichub target/release/migrate

# 3. Run migrations (if you have a local Postgres)
./target/release/migrate

# 4. Start the server locally to verify
PORT=8000 ./target/release/fichub &

# 5. Health check
sleep 2
curl -s http://localhost:8000/api/health | python3 -m json.tool

# 6. Stop
kill %1
```

**Check**: The health endpoint returns `{"status":"ok","db":true,"redis":true,"version":"0.2.0"}` with HTTP 200.

### Troubleshooting

- **`build on gamingpc failed`**: SSH to gamingpc must work passwordlessly. Test with `ssh gamingpc echo ok`. The gamingpc must have the Rust toolchain installed and the NFS repo mounted at `/personal/documents/code/rust/fichub`.
- **`migrate failed`**: Check that PostgreSQL is running and the `DATABASE_URL` in `/opt/fichub/.env` has the correct password. The migration binary reads `DATABASE_URL` from the environment — make sure the `.env` file is sourced (systemd does this via `EnvironmentFile`).
- **`SERVICE not active`**: Check `journalctl -u fichub -n 50` for the last 50 log lines. Common causes: missing `.env` file, wrong `FRONTEND_DIR`, port conflict (another process on 8004).
- **Binary is stale after deploy**: The `stat -c '%y %n' "$BIN"` line prints the binary's modification time. If it hasn't changed, the build step was skipped (used `--skip-build`) or the build didn't actually recompile.
- **QA failures after deploy**: `./qa.sh` runs the full QA suite. If it reports new bugs, check `qa/bugs.db` — the deploy script appends output to keep the latest QA results linked to the deploy.

---

## 46.4 Nginx config — reverse proxy, static frontend

### Goal

In production, you never expose the FicHub Rust server directly to the internet. Nginx sits in front of it as a **reverse proxy** — it terminates TLS, caches static assets, adds security headers, and forwards API requests to the Axum server. Nginx also serves the cached fic files from `/cache/{etype}/{url_id}` directly without involving the Rust binary, which is much faster for large downloads.

A typical production Nginx config for FicHub looks like this:

```nginx
# ── HTTP: redirect everything to HTTPS ──────────────────────────
server {
    listen 80;
    listen [::]:80;
    server_name fichub.example.com;

    # Security headers even on the redirect response
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-Frame-Options "SAMEORIGIN" always;

    return 301 https://$host$request_uri;
}

# ── HTTPS: the real server ──────────────────────────────────────
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name fichub.example.com;

    # Let's Encrypt certificates
    ssl_certificate     /etc/letsencrypt/live/fichub.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # Security headers (production baseline)
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # ── Static assets: cache forever ────────────────────────────
    # SvelteKit puts hashed JS/CSS/JS under /_app/immutable/.
    # The FicHub server (src/frontend/cache_headers.rs) also sets
    # Cache-Control: max-age=31536000, immutable on these paths,
    # but Nginx can serve them even faster since it skips the backend.
    location /_app/immutable/ {
        alias /opt/fichub/frontend/build/_app/immutable/;
        # Let Nginx handle the headers (no need to hit Rust)
        expires 1y;
        add_header Cache-Control "public, max-age=31536000, immutable";
        access_log off;
        add_header X-Content-Type-Options "nosniff" always;
    }

    # ── PWA service worker: never cache ─────────────────────────
    # src/server.rs serve_sw() sets Cache-Control: no-store on /sw.js.
    # Nginx enforces this too so a stale worker never lingers.
    location = /sw.js {
        alias /opt/fichub/frontend/build/sw.js;
        add_header Cache-Control "no-store" always;
        add_header X-Content-Type-Options "nosniff" always;
        access_log off;
    }

    # ── Cache downloads: serve directly from disk ───────────────
    # FicHub stores exports at /public/literature/fanfiction/archive/fichub/cache/
    # The route /cache/{etype}/{url_id} maps to cache files.
    # Nginx serves these without involving the Rust binary.
    location /cache/ {
        alias /public/literature/fanfiction/archive/fichub/cache/;
        # Let users download without forcing a new scrape
        expires 1y;
        add_header Cache-Control "public";
        access_log off;
    }

    # ── API routes: proxy to Axum ───────────────────────────────
    # Everything else goes to the Rust server on port 8004.
    location / {
        proxy_pass http://127.0.0.1:8004;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Long-running scrape requests can take minutes
        proxy_read_timeout 300s;
        proxy_connect_timeout 30s;
        proxy_send_timeout 300s;

        # WebSocket support (for future live features)
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Breakdown

**HTTP → HTTPS redirect**: The first `server` block listens on port 80 and returns a `301` redirect to HTTPS. This ensures all traffic is encrypted. The security headers (`X-Content-Type-Options`, `X-Frame-Options`) are added with `always` so they appear even on redirect responses.

**TLS configuration**: Uses Let's Encrypt certificates at `/etc/letsencrypt/live/...`. TLS 1.2 and 1.3 are enabled; TLS 1.0 and 1.1 are deliberately excluded (they're insecure). `ssl_session_cache` and `ssl_session_timeout` enable session resumption so repeat visitors don't pay the full TLS handshake cost.

**Static assets (`/_app/immutable/`)**: SvelteKit's `adapter-static` build puts all hashed JS, CSS, and image files under `/_app/immutable/`. Because the filenames contain content hashes (e.g., `app.a1b2c3.js`), they're effectively immutable — Nginx can cache them for a year with `expires 1y`. This takes the load off the Rust server for the most frequently requested files.

The `alias` directive maps the URL path to a filesystem path on the host. Here it points to `/opt/fichub/frontend/build/_app/immutable/` — the same `FRONTEND_DIR` that the Rust server uses (see `src/config.rs` — `FRONTEND_DIR` defaults to `./frontend/build` but in production must be an absolute path).

**Service worker (`/sw.js`)**: Served with `Cache-Control: no-store`. This is critical — if the browser caches a stale service worker, it will keep running an old offline-reading strategy and precache list for months. The Rust handler in `src/server.rs` (`serve_sw()`) enforces this header too, but Nginx acts as a second line of defense.

**Cache downloads (`/cache/`)**: FicHub stores user-exported fic files (EPUB, MOBI, HTML, TXT) as flat files on disk, not in the database. The route `/cache/{etype}/{url_id}/{fname}` (registered in `src/server.rs` via `ServeDir` fallback) maps to files in `CACHE_DIR`. By having Nginx serve these directly, the Rust binary is freed up to handle scraping requests.

**API proxy (`location /`)**: All other requests are forwarded to the Axum server on `127.0.0.1:8004`. The `proxy_set_header` lines ensure the Rust server sees the real client IP (`$remote_addr`), the original host, and knows the request came in over HTTPS (`$scheme` = "https"). This matters because `src/config.rs` reads `TRUSTED_PROXIES` to determine which X-Forwarded-For entries are trustworthy.

**Timeouts**: The default Nginx `proxy_read_timeout` is 60s, but FicHub's scrape operations (downloading a 200-chapter fic from AO3, generating an EPUB, converting to MOBI) can take several minutes. The `300s` (5-minute) timeouts accommodate long-running exports.

### Nginx + the health check

The deploy script's health check (`curl http://localhost:8000/`) hits the ThinkCentre directly on port 8000. But in a docker-compose or Nginx-fronted setup, you'd want to check through Nginx on port 443. The `src/routes/health.rs` handler supports query parameters:

```rust
pub async fn health_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HealthQuery>,
) -> impl IntoResponse {
    // ...
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();
    // ...
}
```

So you can probe health without triggering a full database connection:

```bash
# Through Nginx (HTTPS)
curl -sk -o /dev/null -w '%{http_code}' https://fichub.example.com/api/health
# Or skip DB checks for a fast liveness probe
curl -sk https://fichub.example.com/api/health?skip_db=true
```

### Try It Yourself: test the Nginx config

Save the config above to `/etc/nginx/sites-available/fichub` (or `/etc/nginx/conf.d/fichub.conf` depending on your distro), then test it:

```bash
# 1. Validate the config syntax
sudo nginx -t

# 2. Reload Nginx to apply
sudo systemctl reload nginx

# 3. Test HTTPS (replace with your real domain)
curl -sk https://fichub.example.com/api/health | python3 -m json.tool

# 4. Test that static assets are cached
curl -skI https://fichub.example.com/_app/immutable/assets/logo.abcd1234.png | grep Cache-Control

# 5. Test that the service worker is never cached
curl -skI https://fichub.example.com/sw.js | grep Cache-Control
```

**Check**: The health endpoint returns `200` with `{"status":"ok",...}`. The static asset returns `Cache-Control: public, max-age=31536000, immutable`. The service worker returns `Cache-Control: no-store`.

### Troubleshooting

- **`502 Bad Gateway`**: Nginx can't reach the Axum server. Check that `proxy_pass http://127.0.0.1:8004` matches the `PORT` environment variable the Rust server is listening on. Verify with `ss -tlnp | grep 8004`.
- **Static assets returning 404**: The `alias` path must point to the real frontend build directory. If `FRONTEND_DIR=/opt/fichub/frontend/build` in the systemd `.env`, the Nginx `alias` should match. Check with `ls /opt/fichub/frontend/build/_app/immutable/`.
- **Mixed content warnings**: If Nginx terminates TLS but the Rust server generates HTTP URLs (e.g., in export links), you'll get mixed-content errors. Nginx sets `X-Forwarded-Proto: $scheme` which the Rust server reads — check `src/config.rs` for `trusted_proxies` configuration. If your proxy IP isn't in the trusted list, the server won't trust the forwarded headers.
- **Let's Encrypt certificate renewal**: Certbot auto-renewal is typically set up via a cron job or systemd timer. Test with `sudo certbot renew --dry-run`. After renewal, reload Nginx: `sudo systemctl reload nginx`.
- **Cache download 404s after deploy**: If you moved the cache directory, update the `location /cache/` alias to point to the new `CACHE_DIR`. The default in `.env.example` is `./cache`, but `docs/DEPLOYMENT.md` notes production uses `/public/literature/fanfiction/archive/fichub` (see the `VOLUME` in the Dockerfile).

---

## 46.5 What you have now

- You understand the **Dockerfile**: multi-stage build (builder → slim runtime), why `python3-venv` + FanFicFare are installed, why the cache is a `VOLUME`, and how the `cross-build.Dockerfile` produces ARM64 binaries for Pi/Orange Pi.
- You understand **docker-compose.yml**: host networking for low-latency scraping, the Calibre sidecar for format conversion, and how to extend it with PostgreSQL, Redis, and Ollama services (with health checks and model pulls).
- You understand **deploy.sh**: the NFS-aware deploy flow (build on gamingpc → copy binary from NFS to local `/opt` disk → stop → copy → migrate → restart → health-check), why the binary must leave NFS, and the rollback pattern in the `justfile`'s `deploy-backend` recipe.
- You understand the **self-heal watchdog**: the 60-second systemd timer that recovers from NFS wedges by unmounting, restarting mergerfs, and restarting the service.
- You understand the **Nginx config**: HTTP→HTTPS redirect, TLS with Let's Encrypt, `expires 1y` on immutable assets and cache downloads, `no-store` on the service worker, and proxy headers for IP forwarding.
- You understand **`.env.example`** and the full config surface: every environment variable from `DATABASE_URL` to `OLLAMA_EMBED_MODEL` to the tiered rate-limiter and forum moderation settings in `src/config.rs`.

Next: Part 47 — Roadmap Deep-Dive: What Could Come Next. You will step back and see how every piece of FicHub connects: the data flow from scrape to cache to export, the auth system from JWT to role gates, the recommendation engine from signals to strategies, and where your learning journey goes from here.

---

*End of Part 46. On to [Part 47 — Roadmap Deep-Dive: What Could Come Next](./47-roadmap-deepdive/47-roadmap-deepdive.md).*
