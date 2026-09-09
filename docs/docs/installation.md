---
title: Installation Guide — Tokenectomy Razor
description: Install Tokenectomy Razor via npx, cargo, npm, source build, or Docker container.
---

# Installation

Tokenectomy Razor can be installed via multiple methods depending on your environment.

## Method 1: Instant via npx (Recommended)

No Rust toolchain, native compilation, or manual path setup required:

```bash
npx -y tokenectomy-razor --mcp
```

Or install globally via npm:

```bash
npm install -g tokenectomy-razor
```

## Method 2: Cargo (crates.io)

```bash
cargo install tokenectomy
```

## Method 3: Build from Source

```bash
git clone https://github.com/daffa2555/Tokenectomy.git
cd Tokenectomy
cargo build --release
sudo cp target/release/razor /usr/local/bin/razor
```

## Method 4: Multi-Arch Container (GHCR)

```bash
docker pull ghcr.io/daffa2555/razor:latest
```

## Next Steps

- [Configure MCP Integration →](mcp-integration.md)
- [Set up AI Gateway Proxy →](proxy.md)
- [Use in GitHub Actions →](github-actions.md)
