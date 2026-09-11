---
title: Tokenectomy Razor — AI Log Surgery & Secret Redaction Engine
description: High-performance MCP server for AI coding agents. Purge 90%+ framework noise, redact secrets with O(N) ReDoS immunity, sub-millisecond latency. Written in safe Rust.
---

# Tokenectomy Razor

**High-Performance Log Surgery & Secret Redaction Engine for AI Coding Agents**

[![Listed on mcpservers.org](https://mcpservers.org/badge.svg)](https://mcpservers.org/servers/tokenectomy-labs/tokenectomy)
[![Official MCP Registry](https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen)](https://registry.modelcontextprotocol.io)
[![Glama.ai](https://img.shields.io/badge/Glama.ai-Tokenectomy--Razor-purple)](https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy)
[![Crates.io](https://img.shields.io/crates/v/tokenectomy.svg?logo=rust)](https://crates.io/crates/tokenectomy)
[![npm](https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm)](https://www.npmjs.com/package/tokenectomy-razor)
[![CI](https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml/badge.svg)](https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/Tokenectomy-Labs/Tokenectomy/blob/main/LICENSE)

Tokenectomy Razor is an autonomous, machine-to-machine (M2M) **Model Context Protocol (MCP)** server and stream processing engine written in safe Rust. It operates as a high-throughput background sub-cortex for AI coding assistants like **Claude Desktop**, **Cursor**, **Cline**, **Roo Code**, **Windsurf**, and **Google Antigravity**.

## What does it do?

- **Purges 90%+ of framework noise** from error traces (`node_modules`, `site-packages`, `.cargo/registry`)
- **Redacts secrets** (API keys, JWTs, connection strings) with O(N) ReDoS-immune regex
- **Static AST Code Analysis** (`analyze_code`) — detects unclosed handles, resource leaks, and vulnerabilities with zero-external-binary inspection
- **Sub-millisecond latency** — designed for real-time agent loops
- **SHA-256 idempotency cache** — zero repeated API cost for identical failures

```
                  ┌──────────────────────────────────────────────┐
  Agent Error     │              TOKENECTOMY RAZOR               │    Sanitized Context
  Dump (38K toks) │  - Polyglot Stack Frame Filter               │ ──►  (2K toks) ──► LLM
 ────────────────►│  - Deterministic Secret Redactor (O(N))      │
                  │  - SHA-256 Idempotency Cache (24h TTL)       │
                  └──────────────────────────────────────────────┘
```

## Quick Start

```bash
# Instant via npx (no Rust toolchain needed)
npx -y tokenectomy-razor --mcp

# Or install via cargo
cargo install tokenectomy

# Or via npm
npm install -g tokenectomy-razor
```

## Verified Performance

| Benchmark | Result |
|---|---|
| 250K lines (24 MB) redaction | **333 ms** (73 MB/s) |
| ReDoS resistance (50K chars) | **1.44 ms** (linear O(N)) |
| 100 concurrent threads | **27 ms** (7,312 ops/s) |
| Peak memory (250K lines) | **76 MB** VmRSS |

## Supported Languages

| Language | Frameworks |
|---|---|
| **Rust** | Tokio, Actix-web, Axum |
| **Python** | Django, FastAPI, Flask, PyTorch |
| **TypeScript / JavaScript** | Next.js, Express, NestJS, Vite |
| **Golang** | Gin, Fiber, Stdlib panics |
| **Java / Kotlin** | Spring Boot, Quarkus, Gradle |
| **C / C++** | GDB Backtraces, AddressSanitizer |
| **PHP** | Laravel, Symfony |

[Get started →](installation.md){ .md-button .md-button--primary }
[View on GitHub →](https://github.com/Tokenectomy-Labs/Tokenectomy){ .md-button }
