---
title: Tokenectomy Razor — AI Log Surgery & Secret Redaction Engine
description: High-performance MCP server for AI coding agents. Purge 90%+ framework noise, redact secrets with O(N) ReDoS immunity, sub-millisecond latency. Written in safe Rust.
---

# Tokenectomy Razor

**High-Performance Log Surgery & Secret Redaction Engine for AI Coding Agents**

<div class="badge-row">
  <a href="https://mcpservers.org/servers/tokenectomy-labs/tokenectomy"><img src="https://mcpservers.org/badge.svg" alt="Listed on mcpservers.org" height="20" width="160" /></a>
  <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen" alt="Official MCP Registry" height="20" width="170" /></a>
  <a href="https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy"><img src="https://img.shields.io/badge/Glama.ai-Tokenectomy--Razor-purple" alt="Glama.ai" height="20" width="155" /></a>
  <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?logo=rust" alt="Crates.io" height="20" width="95" /></a>
  <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm" alt="npm" height="20" width="100" /></a>
  <a href="https://securityscorecards.dev/viewer/?uri=github.com/Tokenectomy-Labs/Tokenectomy"><img src="https://api.securityscorecards.dev/projects/github.com/Tokenectomy-Labs/Tokenectomy/badge" alt="OpenSSF Scorecard" height="20" /></a>
  <a href="https://www.bestpractices.dev/projects/14704"><img src="https://www.bestpractices.dev/projects/14704/badge" alt="OpenSSF Best Practices" height="20" /></a>
  <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml"><img src="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml/badge.svg" alt="CI" height="20" width="80" /></a>
  <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" height="20" width="90" /></a>
</div>

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
