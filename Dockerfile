# ============================================================
# Multi-stage Dockerfile with cargo-chef for cached Rust builds
# ============================================================
# Stage 1: Chef — install cargo-chef
FROM rust:1-slim AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner — analyze dependencies and create recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder — cook dependencies (cached), then build app
FROM chef AS builder

# Cook dependencies first — this layer is cached as long as
# Cargo.toml/Cargo.lock don't change. No full recompile!
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy source and build — only your code recompiles
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 4: Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tokenectomy /usr/local/bin/tokenectomy
COPY --from=builder /app/target/release/razor /usr/local/bin/razor
COPY --from=builder /app/target/release/tokenectomy-razor /usr/local/bin/tokenectomy-razor
RUN ln -s /usr/local/bin/razor /usr/local/bin/tkmy

ENTRYPOINT ["razor", "--mcp"]
