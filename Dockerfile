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
ENV DATABASE_URL=postgres://fichub:password@localhost:5432/fichub
ENV REDIS_URL=redis://localhost:6379
ENV CACHE_DIR=/public/literature/fanfiction/archive/fichub
ENV PORT=8004
ENV RUST_LOG=info

EXPOSE 8004

VOLUME ["/public/literature/fanfiction/archive/fichub"]

ENTRYPOINT ["fichub"]
