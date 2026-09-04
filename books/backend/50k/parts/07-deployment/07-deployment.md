# Part 7: Deployment

## Chapter 23: Docker and Docker Compose

### From Code to Container

You've built a beautiful Rust web server. It scrapes stories, generates EPUBs, stores them in a database, caches them in Redis, and protects itself with rate limiting. It's fast, it's modular, and it's ready for the world.

But there's a problem.

When you run `cargo run` on your laptop, everything works because *your* laptop has Rust installed, *your* laptop has PostgreSQL running, and *your* laptop has Redis configured just right. But what happens when you try to run this on a different machine? A friend's server? A cloud provider? A tiny ARM computer on your desk?

"It works on my machine" is the most dangerous sentence in software development. Docker fixes this.

### What Is Docker? Containers Explained Simply

Imagine you have a toy box. Inside it, you have all your toys — blocks, cars, action figures. When you take this toy box to a friend's house, everything you need is right there. You don't need to borrow your friend's blocks or use their cars. Your toys travel with you.

Docker does the same thing for software. Instead of hoping that the destination machine has all the right dependencies installed, you package your application along with everything it needs — the Rust runtime, PostgreSQL, Redis, all of it — into a single unit called a **container**.

A container is like a mini-computer that runs inside your real computer. It has its own file system, its own processes, and its own dependencies. But unlike a virtual machine (which is a full computer-within-a-computer), a container shares the host's operating system kernel. This makes containers lightweight — they start in seconds, not minutes, and they use very little memory.

Here's the key insight: if your container runs on your laptop, it will run identically on a server in a datacenter, on an ARM board on your desk, or on someone else's machine across the world. Docker eliminates the "works on my machine" problem.

> 💡 **Key Concept: Container vs. Virtual Machine**
>
> A **virtual machine** is like building a whole new house inside your house. It has its own foundation, its own plumbing, its own electricity. It works, but it's heavy and slow to set up.
>
> A **container** is like putting a room in your house that you can lock off. It has its own furniture (dependencies), but it shares the foundation, plumbing, and electricity (the host OS kernel). It's light, fast, and efficient.
>
> This is why Docker containers start in milliseconds while virtual machines take minutes. For web servers, containers are almost always the better choice.

### Writing a Dockerfile for Rust

A Dockerfile is a recipe. It tells Docker exactly how to build your container, step by step. Let's look at FicHub's actual Dockerfile:

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release
```

Let's break this down line by line.

The `FROM` line tells Docker which base image to start with. We're using `rust:slim-bookworm` — a slim version of Debian Bookworm with Rust already installed. The `AS builder` part gives this build stage a name, which we'll use later.

The `RUN` line installs system packages that Rust needs. `pkg-config` helps find libraries, `libssl-dev` provides OpenSSL headers (needed by `reqwest` for HTTPS), and `libpq-dev` provides PostgreSQL client libraries (needed by `sqlx`). The `rm -rf /var/lib/apt/lists/*` at the end cleans up the package cache to keep the image small.

The `COPY` lines copy our source code into the container. We copy `Cargo.toml` and `Cargo.lock` first (for dependency resolution), then `src` (our Rust code) and `migrations` (our database schema).

The final `RUN` line compiles our code in release mode — optimized for speed.

### Multi-Stage Builds: The Secret to Small Containers

Here's the problem: the `rust:slim-bookworm` image is over 500MB, and after compiling, our binary is sitting in a directory full of compiled dependencies, build artifacts, and temporary files. If we shipped all of that, our container would be enormous.

The solution is a **multi-stage build**. We build the binary in one stage, then copy just the binary into a clean, minimal second stage:

```dockerfile
# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/fichub /app/fichub
COPY --from=builder /app/migrations /app/migrations
COPY .env.example /app/.env

EXPOSE 3000

CMD ["/app/fichub"]
```

This is the runtime stage. We start fresh with `debian:bookworm-slim` — a minimal Debian image without Rust, without build tools, without any of the compilation machinery. We only install what the running binary actually needs: `ca-certificates` (for HTTPS connections to AO3, FF.net, etc.) and `libpq-dev` (for PostgreSQL connections).

The magic line is `COPY --from=builder /app/target/release/fichub /app/fichub`. This reaches into the *builder* stage and grabs just our compiled binary. No source code, no dependencies, no build tools — just the single executable file.

We also copy the migrations (so we can set up the database) and the environment file. Then we expose port 3000 and tell Docker to run our binary.

> ⚠️ **Watch Out: The .env File**
>
> The Dockerfile copies `.env.example` as `.env`. In production, you should mount your real `.env` file as a Docker volume instead of baking it into the image. Secrets (database passwords, API keys) should never be baked into a Docker image — anyone who pulls your image can see them.
>
> We'll fix this with docker-compose volume mounts in a moment.

The result of multi-stage builds is dramatic. The builder stage might be 2GB with all the Rust toolchain and build artifacts. But the final runtime image? Just the binary, a minimal Debian base, and the libraries it needs — often under 100MB.

### docker-compose.yml: Orchestrating Services

FicHub doesn't run alone. It needs PostgreSQL for data storage and Redis for rate limiting and caching. Docker Compose lets you define and run all of these services together with a single command.

Here's FicHub's `docker-compose.yml`:

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: fichub
      POSTGRES_PASSWORD: fichub
      POSTGRES_USER: fichub
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U fichub"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
```

The first two services are PostgreSQL and Redis. We're using Alpine-based images because they're tiny — just a few megabytes each. The `healthcheck` blocks tell Docker to wait until each service is actually ready before starting FicHub. Without these, FicHub might try to connect to PostgreSQL before it's fully initialized and crash.

The `volumes` section creates a named volume called `pgdata`. This persists your database data even if you stop the containers. Without it, your data would vanish every time you ran `docker compose down`.

Now the main event:

```yaml
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgres://fichub:***@postgres:5432/fichub
      REDIS_URL: redis://redis:6379
      CACHE_DIR: /app/cache
      TMP_DIR: /app/tmp
      CALIBRE_CONTAINER: calibre
      PORT: 3000
      FRONTEND_DIR: /app/frontend
      RUST_LOG: info,fichub=debug
    volumes:
      - cache:/app/cache
      - tmp:/app/tmp
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
```

The `app` service builds from the Dockerfile in the current directory (`.`). It exposes port 3000, sets all the environment variables FicHub needs, and waits for PostgreSQL and Redis to be healthy before starting.

Notice the environment variables. `DATABASE_URL` points to `postgres:5432` — that hostname `postgres` comes from the service name in docker-compose. Docker creates a private network where services can find each other by name. FicHub doesn't need to know PostgreSQL's IP address; it just uses the name `postgres`.

> 💡 **Key Concept: Docker Networking**
>
> When Docker Compose starts multiple services, it creates a private network. Each service can reach the others by their service name. So `postgres` resolves to the PostgreSQL container, `redis` resolves to the Redis container, and `app` resolves to FicHub.
>
> This is why the `DATABASE_URL` uses `postgres` as the hostname instead of `localhost`. Inside the container, `localhost` refers to the container itself — not the PostgreSQL service. Service names are the way to talk between containers.

### Environment Variables in Docker

Environment variables are how you configure a container without changing the code. FicHub reads these at startup using the `dotenvy` crate:

```rust
let database_url = std::env::var("DATABASE_URL")
    .expect("DATABASE_URL must be set");

let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://localhost:6379".to_string());
```

In Docker, you can set environment variables three ways:

1. **In docker-compose.yml** — The `environment` block we saw above. Good for values that change per deployment but aren't secrets.

2. **In a `.env` file** — Create a `.env` file next to your `docker-compose.yml` and reference variables with `${VARIABLE_NAME}`. Good for secrets you don't want in version control.

3. **At runtime** — Pass `-e DATABASE_URL=...` to `docker run`. Good for one-off overrides.

For production, the best practice is to use a `.env` file for secrets and keep non-sensitive configuration in docker-compose.yml:

```yaml
  app:
    environment:
      DATABASE_URL: ${DATABASE_URL}
      REDIS_URL: ${REDIS_URL}
      RUST_LOG: info,fichub=debug
```

Then in your `.env` file (which you `.gitignore`):

```
DATABASE_URL=postgres://fichub:secretpassword@postgres:5432/fichub
REDIS_URL=redis://redis:6379
```

### Building and Running

With everything in place, starting FicHub is a single command:

```bash
docker compose up --build
```

The first time you run this, it takes a while. Docker pulls the base images, installs packages, downloads Rust dependencies, and compiles your code. On a decent machine, the first build might take 5-10 minutes. But here's the magic: on subsequent builds, Docker uses its layer cache. If you only changed a few lines of Rust code, Docker skips the dependency download and system package installation — it just recompiles your source. The second build might take 30 seconds.

Docker will:
1. Pull the PostgreSQL and Redis images
2. Build the FicHub image from the Dockerfile
3. Start PostgreSQL, wait for it to be healthy
4. Start Redis, wait for it to be healthy
5. Start FicHub

You'll see logs from all three services mixed together. When you see something like:

```
app-1  | 2024-01-15T10:30:00Z  INFO fichub: Listening on 0.0.0.0:3000
```

FicHub is ready. Open `http://localhost:3000` in your browser and you'll see the web interface.

To stop everything:

```bash
docker compose down
```

To stop and remove all data (database, cache, everything):

```bash
docker compose down -v
```

The `-v` flag removes the named volumes. Use this when you want a fresh start, but be careful — it deletes your database.

### Docker Build Cache Optimization

Building a Rust project in Docker can be painfully slow. Every time you change a line of code, Docker has to recompile everything — and Rust compilation is not fast. The trick is to leverage Docker's layer caching.

The FicHub Dockerfile copies `Cargo.toml` and `Cargo.lock` *before* copying the source code:

```dockerfile
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release
```

This order matters. Docker caches each layer. If only your `src/` files change (which happens most often), Docker reuses the cached `Cargo.toml` and `Cargo.lock` layers. But more importantly, it reuses the cached dependency download step. Without this optimization, every build would re-download and recompile all 70+ dependencies from scratch.

For even better caching, some developers add an intermediate step that creates dummy `src/main.rs`, runs `cargo build` to cache dependencies, then copies the real source and builds again. But for most projects, the simple approach works well enough.

> 💡 **Key Concept: Image Layering**
>
> Each instruction in a Dockerfile creates a **layer** — a snapshot of the file system at that point. Docker caches these layers. If you rebuild an image and a layer hasn't changed, Docker reuses the cached version instead of rebuilding it.
>
> This is why Dockerfiles put things that change infrequently (system packages, dependency lists) at the top, and things that change often (your source code) at the bottom. It's like packing a suitcase: put the stuff you won't need to access first at the bottom, so you don't have to unpack everything to get to your toothbrush.

### Volume Mounts for Persistence

Notice that the app service has two volume mounts:

```yaml
volumes:
  - cache:/app/cache
  - tmp:/app/tmp
```

These are named volumes — Docker manages them and persists data across container restarts. The `cache` volume stores generated EPUBs and scraped metadata. The `tmp` volume stores temporary files during EPUB generation.

Without these volumes, the cache would be wiped every time you restarted the container. With them, FicHub can serve previously-generated EPUBs instantly without re-scraping.

> 🧪 **Try It Yourself**
>
> 1. Create a directory for your FicHub deployment and add a `.env` file with your database credentials
> 2. Run `docker compose up --build` and watch the build process
> 3. Open `http://localhost:3000` and test a URL
> 4. Stop the containers with `docker compose down`
> 5. Start them again with `docker compose up` — notice that the cached EPUB from step 3 is still there
> 6. Try `docker compose down -v` and start fresh — the cache is gone

### What We Built

Docker packaging transforms FicHub from "works on my laptop" to "works everywhere." The multi-stage build compiles Rust in a heavy builder image and copies just the binary into a slim runtime image. Docker Compose orchestrates the three services — FicHub, PostgreSQL, and Redis — with health checks ensuring the right startup order. Environment variables configure the app without code changes. Named volumes preserve data across restarts. One command starts everything; one command stops it.

In the next chapter, we'll take this further: cross-compiling FicHub for ARM64 and deploying it to an Orange Pi single-board computer.


## Chapter 24: Cross-Compilation

### Building for Different CPUs

You built FicHub on your laptop — probably an x86_64 machine, the standard Intel or AMD processor that most computers use. The binary that `cargo build --release` produced runs on x86_64 machines. But what if you want to run FicHub on a different kind of computer?

Meet the Orange Pi. It's a single-board computer — think Raspberry Pi but running on an ARM64 processor. ARM chips are everywhere: your phone, your tablet, your smart TV, and increasingly, servers. They're power-efficient and cheap, which makes them perfect for running a self-hosted server like FicHub.

But here's the catch: an x86_64 binary won't run on an ARM64 processor. They speak different machine languages. You need to **cross-compile** — build your Rust code on your x86_64 laptop, but target an ARM64 processor.

> 💡 **Key Concept: What Is Cross-Compilation?**
>
> Cross-compilation is building software on one type of computer (the **host**) to run on a different type of computer (the **target**). Your laptop is the host (x86_64), and the Orange Pi is the target (ARM64).
>
> It's like writing a letter in English but having it translated into French before you send it. You wrote it on your English keyboard, but the recipient reads it in French. The content is the same; the delivery format is different.
>
> Rust has first-class cross-compilation support. Unlike C or C++, where cross-compilation is notoriously difficult, Rust makes it almost trivial — you just need to install the target and tell Cargo which one to use.

### Installing the ARM64 Target

Rust supports cross-compilation through **targets** — predefined configurations that tell the compiler what CPU architecture and operating system to build for. You install a target with `rustup`:

```bash
rustup target add aarch64-unknown-linux-gnu
```

That long string `aarch64-unknown-linux-gnu` is the target triple. It breaks down as:

- **aarch64** — The CPU architecture (ARM 64-bit)
- **unknown** — The vendor (not specified)
- **linux** — The operating system
- **gnu** — The C library (GNU libc, as opposed to musl)

After running this command, Rust can compile code for ARM64 Linux machines. But there's a catch: Rust can generate the ARM64 code, but it also needs ARM64 versions of any C libraries you link against. Remember that FicHub depends on `libpq` (PostgreSQL client) and OpenSSL — these are C libraries that need ARM64 versions.

### Using cross for Docker-Based Cross-Compilation

Manually installing ARM64 C libraries on your x86_64 laptop is messy. You'd need QEMU emulation, cross-compilation toolchains, and careful library path configuration. There's a much better tool: **cross**.

`cross` is a drop-in replacement for `cargo` that uses Docker to provide the exact build environment for any target. No manual library installation, no QEMU headaches — it just works.

First, install it:

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Then build for ARM64:

```bash
cross build --release --target aarch64-unknown-linux-gnu
```

That's it. `cross` will:
1. Pull a Docker image with the ARM64 toolchain
2. Mount your project inside the container
3. Build your code targeting ARM64
4. Copy the resulting binary back to your host machine

The binary lands in `target/aarch64-unknown-linux-gnu/release/fichub`. It looks identical to your regular build, but it's compiled for ARM64. You can verify this with `file`:

```bash
file target/aarch64-unknown-linux-gnu/release/fichub
# Output: ELF 64-bit LSB executable, ARM aarch64, ...
```

The `ARM aarch64` in the output confirms it's an ARM64 binary.

> ⚠️ **Watch Out: Build Time**
>
> Cross-compilation with `cross` is slower than a native build because it starts a Docker container and may need to pull the cross-compilation image on the first run. On subsequent builds, the image is cached and it's much faster. First-time builds can take 10-20 minutes depending on your internet speed and project size.
>
> Plan accordingly — don't start a cross-compilation build right before a demo.

### The cargo build --release --target Command

Let's compare the two approaches:

```bash
# Native build (for your current machine)
cargo build --release

# Cross-compilation (for ARM64)
cross build --release --target aarch64-unknown-linux-gnu
```

The `--release` flag is crucial. Without it, Cargo builds in debug mode — unoptimized, with debug symbols, often 10x slower. For deployment, you always want release mode.

The `--target` flag tells Cargo (or `cross`) which CPU architecture to target. Without it, Cargo builds for your current machine.

For a native build, you could also use:

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

This is identical to `cargo build --release` on an x86_64 machine — the `x86_64-unknown-linux-gnu` target is the default. But being explicit about targets is good practice, especially in CI/CD pipelines where you might be building for multiple architectures.

### Deploying to Orange Pi

With the ARM64 binary built, it's time to put it on the actual hardware. The Orange Pi is running a Debian-based Linux distribution, and we need to get our binary, migrations, and configuration over to it.

First, let's figure out the Orange Pi's IP address. If it's on your local network:

```bash
# On the Orange Pi, run:
hostname -I
# Output: 192.168.1.100
```

Now, copy the binary and migrations using `scp` (secure copy):

```bash
# Create a directory on the Orange Pi
ssh user@192.168.1.100 "mkdir -p ~/fichub/migrations"

# Copy the binary
scp target/aarch64-unknown-linux-gnu/release/fichub \
    user@192.168.1.100:~/fichub/fichub

# Copy migrations
scp -r migrations/* \
    user@192.168.1.100:~/fichub/migrations/

# Copy environment file
scp .env.production \
    user@192.168.1.100:~/fichub/.env
```

The `scp` command works like `cp` but over the network. It uses SSH for encryption, so your credentials and data are safe in transit.

### Running on the Target Machine

SSH into the Orange Pi:

```bash
ssh user@192.168.1.100
```

Set up the environment:

```bash
cd ~/fichub

# Make the binary executable
chmod +x fichub

# Create cache and tmp directories
mkdir -p cache tmp

# Verify it's the right architecture
file fichub
# Should say: ELF 64-bit LSB executable, ARM aarch64, ...
```

Now you need PostgreSQL and Redis running on the Orange Pi. The easiest approach is Docker:

```bash
# Start just PostgreSQL and Redis on the Orange Pi
docker compose -f docker-compose.prod.yml up -d postgres redis
```

Or you can install them directly on the system:

```bash
# On Debian/Ubuntu
sudo apt install postgresql redis-server

# Start them
sudo systemctl start postgresql
sudo systemctl start redis-server
```

With dependencies running, start FicHub:

```bash
cd ~/fichub

# Source the environment
source .env

# Run FicHub
./fichub
```

If everything works, you'll see:

```
INFO fichub: Listening on 0.0.0.0:3000
```

FicHub is now running on an ARM64 single-board computer, consuming a few watts of power, and serving fanfiction downloads. That's the beauty of Rust — one codebase, compiled to different architectures, runs anywhere.

> 🧪 **Try It Yourself**
>
> 1. Install a cross-compilation target: `rustup target add aarch64-unknown-linux-gnu`
> 2. Install cross: `cargo install cross`
> 3. Build for ARM64: `cross build --release --target aarch64-unknown-linux-gnu`
> 4. Check the binary: `file target/aarch64-unknown-linux-gnu/release/fichub`
> 5. If you don't have an ARM64 device, try cross-compiling for a different target you have access to
> 6. Compare the file sizes: `ls -lh target/release/fichub` vs `ls -lh target/aarch64-unknown-linux-gnu/release/fichub`

### Comparing Architectures

Different architectures have different strengths. Here's a quick comparison:

| Feature | x86_64 (Intel/AMD) | ARM64 (Orange Pi/RPi) |
|---|---|---|
| Raw performance | Higher | Lower |
| Power consumption | 50-200W | 5-15W |
| Cost | $500+ | $30-80 |
| Fan noise | Often needed | Fanless |
| Perfect for | Heavy workloads | Always-on servers |

For a personal fanfiction server that handles a few requests per minute, an ARM64 board is ideal. It sips power, generates no noise, and costs less than a fancy dinner out. The performance difference only matters when you're processing hundreds of concurrent requests — which FicHub's rate limiter is specifically designed to prevent.

> ⚠️ **Watch Out: Memory Limits**
>
> ARM64 single-board computers typically have 1-8GB of RAM. FicHub itself is memory-efficient (Rust programs usually are), but you're also running PostgreSQL and Redis. Make sure your board has at least 2GB of RAM. The Orange Pi 5 with 4GB or 8GB is a sweet spot — plenty of headroom for all three services.
>
> If you're on a 1GB board, consider running PostgreSQL on a separate machine or using SQLite instead.

### Making It a Systemd Service

Running `./fichub` in a terminal is fine for testing, but in production, you want FicHub to start automatically when the Orange Pi boots and restart if it crashes. That's what **systemd** does.

Create a service file:

```bash
sudo tee /etc/systemd/system/fichub.service << 'EOF'
[Unit]
Description=FicHub Fanfiction Download Server
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=fichub
WorkingDirectory=/home/fichub/fichub
ExecStart=/home/fichub/fichub/fichub
Restart=always
RestartSec=5
EnvironmentFile=/home/fichub/fichub/.env

[Install]
WantedBy=multi-user.target
EOF
```

Enable and start it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable fichub
sudo systemctl start fichub
sudo systemctl status fichub
```

Now FicHub starts on boot, restarts if it crashes, and logs to the system journal. You can check logs with `journalctl -u fichub -f`.

### What We Built

Cross-compilation lets you build FicHub on your laptop and deploy it to any architecture. The `cross` tool eliminates the pain of setting up cross-compilation toolchains by using Docker to provide clean build environments. Once built, `scp` gets the binary to the target machine, and systemd keeps it running.

In the next chapter, we'll put nginx in front of FicHub — a reverse proxy that handles TLS, serves static files, and protects our server from the public internet.


## Chapter 25: nginx Reverse Proxy

### The Front Door

FicHub is running. It listens on port 3000, serves EPUBs, and handles API requests. But there are a few things it can't do well on its own:

- **TLS/HTTPS** — Modern browsers expect HTTPS. FicHub doesn't handle TLS certificates.
- **Static files** — Serving CSS, JavaScript, and images is better left to a specialized tool.
- **Rate limiting at the edge** — Before requests even reach FicHub, we want to block bad actors.
- **Load balancing** — If you ever run multiple FicHub instances, you need something to distribute traffic.

That's where **nginx** comes in. nginx is a web server and reverse proxy that sits between the internet and your application. Think of it as a receptionist in an office lobby — visitors talk to the receptionist, who then routes them to the right desk.

> 💡 **Key Concept: Reverse Proxy**
>
> A **proxy** sits between a client and a server. A **forward proxy** hides the client (like a VPN — the server doesn't know who you are). A **reverse proxy** hides the server (like nginx — the client doesn't know which server processed their request).
>
> When you type `https://fichub.example.com` into your browser, your request goes to nginx first. nginx handles the TLS encryption, serves any static files, and forwards dynamic requests to FicHub on port 3000. The client never talks to FicHub directly — it only sees nginx.

### Installing nginx

On most Linux distributions, nginx is a package away:

```bash
# Debian/Ubuntu
sudo apt install nginx

# Arch Linux
sudo pacman -S nginx

# Fedora
sudo dnf install nginx
```

Start it and enable it on boot:

```bash
sudo systemctl start nginx
sudo systemctl enable nginx
```

Verify it's running:

```bash
curl http://localhost
# Should return nginx's default "Welcome to nginx!" page
```

### Configuring nginx to Proxy to FicHub

nginx's configuration lives in `/etc/nginx/sites-available/` on Debian-based systems. Create a new config file:

```bash
sudo tee /etc/nginx/sites-available/fichub << 'EOF'
server {
    listen 80;
    server_name fichub.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
EOF

# Enable the site
sudo ln -s /etc/nginx/sites-available/fichub /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default

# Test the configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

Let's break down this configuration.

The `server` block defines a virtual host — a website that nginx serves. The `listen 80` directive says "accept HTTP connections on port 80." The `server_name` directive says "only handle requests for this domain."

The `location /` block matches all requests and forwards them to FicHub:

- `proxy_pass http://127.0.0.1:3000` — Forward requests to FicHub on port 3000
- `proxy_set_header Host $host` — Pass the original hostname to FicHub
- `proxy_set_header X-Real-IP $remote_addr` — Tell FicHub the real client IP
- `proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for` — Pass the chain of IPs
- `proxy_set_header X-Forwarded-Proto $scheme` — Tell FicHub whether the original request was HTTP or HTTPS

Those proxy headers are crucial. Without them, FicHub would think every request comes from `127.0.0.1` (nginx's address), making rate limiting useless — all users would share one IP. With the headers, FicHub can see each user's real IP and rate-limit accordingly.

### Serving Frontend Static Files

If FicHub has a frontend (HTML, CSS, JavaScript), nginx can serve those files directly without bothering FicHub:

```nginx
server {
    listen 80;
    server_name fichub.example.com;

    # Serve static frontend files
    location / {
        root /var/www/fichub;
        try_files $uri $uri/ /index.html;

        # Cache static assets aggressively
        location ~* \.(css|js|png|jpg|gif|ico|woff2|svg)$ {
            expires 30d;
            add_header Cache-Control "public, immutable";
        }
    }

    # Proxy API requests to FicHub
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Now nginx handles two types of requests:

1. **Static files** (`/`, `*.css`, `*.js`, etc.) — Served directly from `/var/www/fichub` by nginx. No FicHub involvement. The `try_files` directive ensures that single-page app routes (like `/about`) serve `index.html` instead of returning 404.

2. **API requests** (`/api/`) — Forwarded to FicHub. Only the dynamic, server-side logic goes to the Rust binary.

The `expires 30d` and `Cache-Control` headers tell browsers to cache static assets for 30 days. This dramatically reduces load on the server and speeds up page loads for returning visitors.

> ⚠️ **Watch Out: CORS Headers**
>
> If your frontend runs on a different domain than your API, you need CORS headers. Since nginx serves both the frontend and proxies the API on the same domain, CORS isn't needed. But if you're developing locally with a separate frontend dev server, make sure FicHub's CORS middleware is configured to allow requests from `http://localhost:5173` (or wherever your dev server runs).

### TLS/HTTPS with Let's Encrypt

Modern browsers warn users about HTTP sites. Google Chrome shows "Not Secure" in the address bar. Firefox shows a similar warning. To get the padlock icon and "Secure" label, you need HTTPS.

**Let's Encrypt** provides free TLS certificates. The `certbot` tool automates the process:

```bash
# Install certbot
sudo apt install certbot python3-certbot-nginx

# Get a certificate
sudo certbot --nginx -d fichub.example.com

# Verify auto-renewal
sudo certbot renew --dry-run
```

Certbot will:
1. Verify you own the domain (by placing a file on your web server)
2. Obtain a TLS certificate from Let's Encrypt
3. Modify your nginx config to enable HTTPS
4. Set up auto-renewal (certificates expire every 90 days)

After running certbot, your nginx config becomes:

```nginx
server {
    listen 80;
    server_name fichub.example.com;
    # Redirect all HTTP to HTTPS
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name fichub.example.com;

    ssl_certificate /etc/letsencrypt/live/fichub.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/fichub.example.com/privkey.pem;

    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

The first server block redirects all HTTP traffic to HTTPS. The second handles HTTPS with the TLS certificate. The `ssl_certificate` and `ssl_certificate_key` lines point to the certificates certbot installed. Certbot also sets up a cron job that automatically renews certificates before they expire — you never have to think about it.

One more thing: always force HTTPS redirects. The `return 301 https://$host$request_uri;` line ensures that anyone who types `http://` gets redirected to `https://`. This prevents "mixed content" warnings where some resources load over HTTP and others over HTTPS.

> ⚠️ **Watch Out: Let's Encrypt Rate Limits**
>
> Let's Encrypt has rate limits to prevent abuse. You can only request 50 certificates per registered domain per week. This is plenty for normal use, but if you're testing and constantly creating new certificates, you might hit the limit. Use the staging environment for testing:
>
> ```bash
> sudo certbot --nginx --staging -d fichub.example.com
> ```
>
> This gives you fake (untrusted) certificates so you can test the flow without using up your real quota.

> 💡 **Key Concept: Why TLS Matters**
>
> Without TLS, data travels between the client and server in plain text. Anyone on the same Wi-Fi network — at a coffee shop, airport, or dorm — can read your traffic. They can see which stories you're downloading, steal your session cookies, or inject malicious content.
>
> With TLS, all traffic is encrypted. Even if someone intercepts the packets, they can't read them. It's the difference of sending a postcard (everyone can read it) versus sending a sealed letter (only the recipient can read it).

### Caching Headers for Static Assets

When a user visits FicHub, their browser downloads CSS files, JavaScript bundles, fonts, and images. Without caching, every page load re-downloads everything. With caching, the browser keeps local copies and only re-downloads when the files change.

nginx makes this easy:

```nginx
# Static assets — cache for 1 year (content-addressed filenames)
location ~* \.(css|js|woff2)$ {
    expires 1y;
    add_header Cache-Control "public, immutable";
}

# Images — cache for 30 days
location ~* \.(png|jpg|jpeg|gif|ico|svg|webp)$ {
    expires 30d;
    add_header Cache-Control "public";
}

# HTML — always revalidate (HTML references the hashed assets)
location ~* \.html$ {
    add_header Cache-Control "no-cache";
}
```

The trick is **immutable** caching. If your build process generates content-addressed filenames (like `app.a1b2c3d4.js`), the filename changes when the content changes. Browsers can safely cache these forever — the `immutable` directive tells them not to even check if the file has changed. When you deploy a new version, users automatically get the new files because they have different names.

For HTML files, we use `no-cache` because HTML is the entry point that references the hashed assets. The browser always checks for a new HTML file, which points to the new JS/CSS files.

### The Complete Deployment

Let's zoom out and see the full picture of how a request flows through the system:

```
Client (browser)
    │
    │ HTTPS request to fichub.example.com
    ▼
nginx (port 443)
    ├── TLS termination (decrypt the request)
    ├── Static file serving (CSS, JS, images)
    └── Reverse proxy to FicHub (API requests)
            │
            │ HTTP request to 127.0.0.1:3000
            ▼
        FicHub (Rust, port 3000)
            ├── Rate limiting (Redis)
            ├── Datacenter IP blocking
            ├── Scraper (fetch story metadata)
            ├── EPUB generation
            └── Database queries
                    │
                    │ SQL queries
                    ▼
                PostgreSQL (port 5432)
```

Every request passes through these layers:

1. **DNS** resolves `fichub.example.com` to your server's IP address
2. **nginx** receives the request, terminates TLS, and decides what to do
3. Static assets are served directly from disk — fast, no FicHub involvement
4. API requests are forwarded to FicHub with the real client IP
5. **FicHub** applies rate limiting, checks for abuse, and processes the request
6. **PostgreSQL** stores and retrieves data
7. **Redis** handles caching and rate limit counters

This layered architecture means each component does what it's best at. nginx is excellent at TLS and static files. FicHub is excellent at scraping and EPUB generation. PostgreSQL is excellent at data storage. Redis is excellent at fast lookups. No single component tries to do everything.

### Rate Limiting at the Edge

nginx can also do basic rate limiting before requests even reach FicHub. This adds another layer of defense:

```nginx
# Define rate limit zones
limit_req_zone $binary_remote_addr zone=fichub:10m rate=10r/s;

server {
    listen 443 ssl http2;
    server_name fichub.example.com;

    # Apply rate limit to API endpoints
    location /api/ {
        limit_req zone=fichub burst=20 nodelay;
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

The `limit_req_zone` directive defines a rate limit zone: 10 requests per second per IP address, stored in 10MB of memory. The `limit_req` directive applies this zone with a burst of 20 — allowing short spikes without blocking legitimate users.

This means nginx will reject obviously abusive traffic before it ever reaches FicHub. FicHub's own rate limiter then handles the more nuanced cases — like checking if an IP is a datacenter bot or applying per-tag limits. Two layers of protection, each catching different types of abuse.

### Health Checks and Monitoring

You need to know if any component goes down. nginx can proxy health checks to FicHub:

```nginx
# Health check endpoint
location /health {
    proxy_pass http://127.0.0.1:3000/api/v0/health;
    proxy_set_header Host $host;
}
```

For monitoring, you can set up a simple script that checks all services:

```bash
#!/bin/bash
# check-health.sh

# Check nginx
if ! systemctl is-active --quiet nginx; then
    echo "CRITICAL: nginx is down"
    sudo systemctl restart nginx
fi

# Check FicHub
if ! curl -sf http://localhost:3000/api/v0/health > /dev/null; then
    echo "WARNING: FicHub is not responding"
    sudo systemctl restart fichub
fi

# Check PostgreSQL
if ! pg_isready -q; then
    echo "CRITICAL: PostgreSQL is down"
    sudo systemctl restart postgresql
fi

# Check Redis
if ! redis-cli ping > /dev/null 2>&1; then
    echo "CRITICAL: Redis is down"
    sudo systemctl restart redis-server
fi

echo "All services healthy at $(date)"
```

Run this every minute with a cron job:

```bash
# Edit crontab
crontab -e

# Add this line
* * * * * /home/user/fichub/check-health.sh >> /var/log/fichub-health.log 2>&1
```

For production, you'd want a more sophisticated monitoring solution — Prometheus for metrics, Grafana for dashboards, or a service like UptimeRobot for external health checks. But the basic principle is the same: regularly check that each component is alive and restart it if it's not.

> 🧪 **Try It Yourself**
>
> 1. Install nginx on your machine
> 2. Start FicHub with `cargo run`
> 3. Write an nginx config that proxies port 80 to FicHub's port 3000
> 4. Visit `http://localhost` — you should see FicHub's interface
> 5. Check nginx's access logs: `tail -f /var/log/nginx/access.log`
> 6. Add caching headers for static assets
> 7. Set up a health check cron job
> 8. If you have a domain, try certbot for free HTTPS

### What We Built

nginx completes the deployment stack. As a reverse proxy, it handles TLS encryption with free Let's Encrypt certificates, serves static frontend files with aggressive caching, and forwards API requests to FicHub with the correct client IP headers. The health check script monitors all services and restarts them if they fail.

The full deployment chain is now: **DNS → nginx (TLS + static files + proxy) → FicHub (scraping + EPUB + rate limiting) → PostgreSQL + Redis**. Each layer does what it's best at. nginx handles the internet-facing security and performance. FicHub handles the application logic. PostgreSQL and Redis handle the data.

This is how real web applications are deployed. Not a single monolithic process trying to do everything, but a team of specialized components working together. Docker packages them, cross-compilation lets them run on any hardware, and nginx protects them from the public internet.

FicHub is no longer a project on your laptop. It's a production-ready server, deployed on real hardware, serving real users, protected by real security measures. The journey from "Hello World" to a fully deployed web server is complete — and you understood every step along the way.

This is what makes Rust special for this kind of project. You write the code once, and with cross-compilation, you can deploy it to a $40 ARM board, a $500 cloud server, or a $5,000 dedicated machine. The same binary, the same performance characteristics, the same safety guarantees. No other systems language makes this as easy.

And the deployment story isn't just about Rust. Docker is language-agnostic — you could build a FicHub equivalent in Go, Python, or Node.js and deploy it the same way. nginx is language-agnostic too. The patterns you learned here — multi-stage builds, reverse proxies, health checks, TLS termination — apply to any web application you'll ever deploy.

But now that FicHub is running in production, there's one more thing we should talk about: what happens when things go wrong? Monitoring, alerting, and maintaining a production system is a skill in itself. We'll cover that in the next part.

### What We Built — Part 7 Summary

In this part, we took FicHub from development to production deployment:

1. **Docker and Docker Compose** (Chapter 23) — Package FicHub into containers with multi-stage builds, orchestrate PostgreSQL and Redis with health checks, and manage configuration with environment variables.

2. **Cross-Compilation** (Chapter 24) — Build ARM64 binaries using `cross` and Docker, deploy to an Orange Pi single-board computer with `scp`, and manage the service with systemd.

3. **nginx Reverse Proxy** (Chapter 25) — Terminate TLS with Let's Encrypt, serve static files with caching headers, proxy API requests with real IP forwarding, and monitor all services with health checks.

The complete deployment architecture is:

```
Client → DNS → nginx (HTTPS) → FicHub (Rust) → PostgreSQL + Redis
```

Each component does what it's best at, and together they form a robust, secure, production-ready system. FicHub is no longer just code — it's a service, running on real hardware, available to real users, and built with tools and techniques used by professional software teams worldwide.
