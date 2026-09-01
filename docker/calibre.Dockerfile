FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    calibre \
    && rm -rf /var/lib/apt/lists/*

# Keep the container running
CMD ["tail", "-f", "/dev/null"]
