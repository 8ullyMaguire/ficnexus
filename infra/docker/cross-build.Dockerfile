FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc

WORKDIR /app
COPY . .

RUN rustup target add aarch64-unknown-linux-gnu && \
    cargo build --release --target aarch64-unknown-linux-gnu

# Final stage: just the binary
FROM debian:bookworm-slim AS output
COPY --from=builder /app/target/aarch64-unknown-linux-gnu/release/fichub /fichub
