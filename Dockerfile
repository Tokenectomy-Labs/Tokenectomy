# Multi-stage Dockerfile for Glama, Smithery, and Containerized MCP Execution
FROM rust:1.85-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./

# Pre-cache dependencies
RUN mkdir -p src/bin && \
    touch src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/razor.rs && \
    echo "fn main() {}" > src/bin/tokenectomy-razor.rs && \
    cargo build --release && \
    rm -rf src

# Build actual application
COPY src ./src
RUN touch src/lib.rs src/main.rs src/bin/razor.rs src/bin/tokenectomy-razor.rs && cargo build --release

# Minimal runtime image
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
