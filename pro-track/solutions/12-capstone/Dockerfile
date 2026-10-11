# syntax=docker/dockerfile:1
# Multi-stage build: compile in a big image with the Rust toolchain, then copy only the
# binary into a small runtime image. The result has no compiler, no source code and no
# build cache, so it's smaller and has less to attack.
#   docker build -t shortener .
#   docker run --rm -p 3000:3000 -v shortener-data:/data shortener

# ---- build stage ----
FROM rust:1-bookworm AS builder
WORKDIR /app

# Build the dependencies first, with placeholder sources. This layer is cached until
# Cargo.toml or Cargo.lock change, so editing your code doesn't rebuild every crate.
COPY Cargo.toml Cargo.lock build.rs ./
RUN mkdir src migrations \
    && echo 'fn main() {}' > src/main.rs \
    && touch src/lib.rs \
    && cargo build --release --locked \
    && rm -rf src

# Now the real code. `touch` makes sure Cargo sees it as newer than the placeholders.
COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# ---- runtime stage ----
FROM debian:bookworm-slim
# Run as an unprivileged user, never as root.
RUN useradd --system --uid 10001 --home-dir /data --create-home shortener
COPY --from=builder /app/target/release/shortener /usr/local/bin/shortener
USER shortener
WORKDIR /data
ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:///data/shortener.db \
    LOG_FORMAT=json
EXPOSE 3000
VOLUME ["/data"]
# docker stop sends SIGTERM: the service finishes its requests and exits.
CMD ["shortener"]
