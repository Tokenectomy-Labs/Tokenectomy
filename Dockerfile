# Multi-stage Dockerfile for Glama, Smithery, and Containerized MCP Execution
FROM rust:1.85-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./

# Pre-cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build actual application
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tokenectomy /usr/local/bin/tokenectomy
RUN ln -s /usr/local/bin/tokenectomy /usr/local/bin/tkmy

ENTRYPOINT ["tokenectomy", "--mcp"]
