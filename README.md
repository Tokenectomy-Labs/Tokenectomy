<div align="center">
  <a href="https://tokenectomy-web.vercel.app">
    <img src="media/logo.png" width="140" alt="Tokenectomy Razor Logo" />
  </a>

  <h1>Tokenectomy Razor</h1>

  <p><b>The autonomous M2M sub-cortex for AI coding agents</b></p>

  <p>
    <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?style=flat-square&color=ea580c&logo=rust" alt="crates.io" /></a>
    <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?style=flat-square&color=cb3837&logo=npm" alt="npm" /></a>
    <a href="https://tokenectomy-labs.github.io/Tokenectomy"><img src="https://img.shields.io/badge/docs-pages-2563eb?style=flat-square&logo=gitbook" alt="Documentation" /></a>
    <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tokenectomy-Labs/Tokenectomy/ci.yml?branch=main&style=flat-square&logo=githubactions" alt="CI" /></a>
    <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/mcp-registry-8b5cf6?style=flat-square" alt="Official MCP Registry" /></a>
    <a href="SECURITY.md"><img src="https://img.shields.io/badge/security-audited-2ea44f?style=flat-square&logo=rust" alt="Security Audit" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="License" /></a>
  </p>

  <p>
    Tokenectomy is a zero-allocation, machine-to-machine (M2M) sub-cortex written in safe Rust.<br />
    It sits between terminal execution and LLM context windows, excising 95%+ framework noise,<br />
    redacting credentials with <i>O(N)</i> ReDoS safety, and streaming sanitized context in sub-milliseconds.
  </p>

  <p>
    <b>Hardware-Grounded Truth &bull; &lt;0.2ms DFA Excision &bull; Zero Dirty Git Diff</b>
  </p>

  <p>
    <a href="https://tokenectomy-web.vercel.app">Website</a> &bull;
    <a href="https://tokenectomy-labs.github.io/Tokenectomy">Documentation</a> &bull;
    <a href="#quick-start">Quick Start</a> &bull;
    <a href="#verifiable-benchmarks">Benchmarks</a> &bull;
    <a href="ARCHITECTURE.md">Architecture</a> &bull;
    <a href="#edition-comparison">Sentinel Tier</a>
  </p>
</div>

<br />

```json
// Add to ~/.cursor/mcp.json or claude_desktop_config.json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

---

## ⚡ The 30-Second Surgery

When an autonomous coding agent runs a failing build or test, the terminal dumps tens of thousands of tokens of internal framework stack frames and potentially leaks production secrets directly into the LLM context window.

```
Raw Terminal Crash (45,820 tokens + Leaked Secrets)
     │
     ▼  <0.2ms Zero-Allocation Rust DFA Excision
[Redact Secrets Locally] ──► [Filter Framework Frames] ──► [Extract Source Context]
     │
     ▼
Sanitized Agent Context (118 tokens • Zero Secrets • Sub-millisecond)
```

### Before & After Comparison

#### ❌ Before: Raw Crash Dump (45,820 Tokens Ingested)

```text
TypeError: Cannot read properties of undefined (reading 'digest')
    at Object.<anon> (/node_modules/next/bundle5.js:142:31)
    at __webpack_require__ (/node_modules/next/bundle5.js:198:12)
    at Object.execute (/node_modules/next/dev-server.js:412:19)
    at processTicksAndRejections (task_queues:95:5)
    Database connection failed: postgresql://admin:super_secret_password@db.prod.internal:5432/primary
    API key leaked: sk-ant-api03-abcdef1234567890abcdef1234567890
    [... 480 internal dependency frames flooding LLM context ...]
```

#### ✅ After: Tokenectomy Razor (118 Tokens • &lt;0.2ms • Zero Secrets)

```text
src/components/Header.tsx:42:15 - SyntaxError
  42 |   const user = useSession( ;
     |                           ^ Expected ')'
🛡️ [CONNECTION_STRING_REDACTED]
🛡️ [REDACTED] ANTHROPIC_API_KEY=[REDACTED_SECRET_KEY]
```

> **Result:** 99.7% context token reduction, zero credential leakage, prompt cache preserved.

---

## 🔬 Verifiable Benchmarks

All performance claims are hardware-grounded and independently reproducible on physical hardware (measured on 10-Core Intel Core i5-1235U @ 15W running Arch Linux, Kernel 6.13):

| Benchmark Target | Workload Under Test | Verified Measurement | Result |
|---|---|---|:---:|
| **Log Redaction Throughput** | 250,000 lines (24.44 MB) enterprise dump with API keys & connection URIs | **531,002 lines/sec** (470.8 ms, 73.3 MB/s) | **Pass** |
| **ReDoS Immunity** | 50,000-character pathological backtracking regex payload | **1.09 ms** (Strict Linear $O(N)$ Evaluation) | **Pass** |
| **Thread Concurrency** | 100 concurrent OS threads executing simultaneous redaction | **7,312 ops/sec** (100/100 completed in 27.35 ms) | **Pass** |
| **Memory Footprint** | Peak Resident Memory during 250k-line continuous stress test | **76.24 MB VmRSS** via `/proc/self/status` | **Pass** |
| **Release Test Suite** | Full integration test matrix across extractors, filters, and analyzers | **57 / 57 Verified Green** (Zero panics, zero leaks) | **Pass** |

To reproduce locally on your physical machine:

```bash
cargo test --release --test stress_benchmark -- --nocapture
```

---

## 🚀 Quick Start

### 1. Model Context Protocol (MCP) Setup

Tokenectomy Razor operates natively over JSON-RPC 2.0 stdio, compliant with the official Model Context Protocol specification.

#### Cursor Composer
Add to `.cursor/mcp.json` in your workspace root:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

#### Claude Desktop
Add to `claude_desktop_config.json`:
- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux:** `~/.config/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

#### Windsurf (Codeium)
Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

#### VS Code (Cline / Roo Code)
In Cline or Roo Code settings (`cline_mcp_settings.json`):

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"],
      "disabled": false,
      "autoApprove": ["get_error_context", "analyze_code"]
    }
  }
}
```

#### Google Antigravity CLI
```bash
agy mcp add tokenectomy-razor -- npx -y tokenectomy-razor --mcp
```

---

### 2. Standalone CLI & Terminal Piping

When debugging or piping terminal output directly:

```bash
# Pipe terminal test failures through the surgical redactor:
npm test 2>&1 | razor --scrub

# Sanitize a specific raw log file:
razor --scrub --file /var/log/app/error.log > sanitized.log

# Offline local air-gapped mode (zero external network calls):
cat failure.log | razor --scrub --local-only
```

---

### 3. AI Gateway Reverse Proxy (`--proxy`)

Tokenectomy Razor can operate as a high-throughput local HTTP reverse proxy on `127.0.0.1:8080`. It intercepts outbound prompt streams, performs real-time token excision and credential sanitization, and forwards clean requests upstream to OpenAI, Anthropic, or Ollama.

```bash
# Start local gateway proxy forwarding to OpenAI:
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.openai.com/v1

# Start local gateway forwarding to Ollama:
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url http://127.0.0.1:11434/v1
```

Point any standard SDK client to the local proxy:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
```

#### FinOps Economics Dashboard
Open `http://127.0.0.1:8080/dashboard` in any browser to monitor real-time token savings, dollar savings (blended LLM pricing), total requests, and active security redactions.

---

## 🛠️ Exposed MCP Tools

Tokenectomy Razor complies with **Glama Grade A Tool Definition Quality Score (TDQS)** with explicit parameter boundaries:

| Tool Name | Capability Description |
|---|---|
| `get_error_context` | Performs trace surgery on error dumps, removes framework noise, redacts credentials, and extracts relevant local source context bounded to the workspace. |
| `analyze_code` | Performs static AST code analysis to detect resource leaks, unclosed handles, and syntax vulnerabilities with bounded execution limits and precise LSP UTF-16 coordinates. |
| `apply_code_patch` | Applies atomic file modifications with post-write language syntax verification (`cargo check`, `py_compile`, `node --check`) and automated rollback on failure. |
| `search_stack_overflow` | Queries Stack Exchange API for relevant error signatures using sanitized, redacted search terms. |

---

## 🌐 Supported Polyglot Ecosystems

| Language | Frameworks Supported | Excluded Framework Internals |
|---|---|---|
| **Rust** | Tokio, Actix-web, Axum | `.cargo/registry`, `.rustup`, `target/debug/build` |
| **TypeScript / JS** | Next.js, Express, NestJS, Vite | `node_modules`, `.next`, `dist`, webpack internals |
| **Python** | Django, FastAPI, Flask, PyTorch | `site-packages`, `dist-packages`, `venv`, `__pycache__` |
| **Golang** | Gin, Fiber, Stdlib Panics | `go/src` (stdlib), `go/pkg/mod`, `vendor` |
| **Java / Kotlin** | Spring Boot 3, Tomcat, Netty | `.m2/repository`, `.gradle/caches`, internal bytecode |
| **C / C++** | AddressSanitizer, GDB / LLDB | `/usr/include`, `/usr/lib`, `vcpkg_installed` |
| **PHP** | Laravel, Symfony | `vendor/composer`, `vendor/symfony` |

---

## 📦 Installation Options

### Method 1: Instant via npx (Zero Toolchain Setup)
```bash
npx -y tokenectomy-razor --mcp
```

### Method 2: Cargo (crates.io)
```bash
cargo install tokenectomy
```

### Method 3: Precompiled Standalone Binaries
Zero-dependency, standalone release binaries available on [GitHub Releases](https://github.com/Tokenectomy-Labs/Tokenectomy/releases):
- **Linux:** `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`
- **macOS:** `aarch64-apple-darwin` (Apple Silicon M1/M2/M3/M4), `x86_64-apple-darwin` (Intel)
- **Windows:** `x86_64-pc-windows-msvc.exe`

### Method 4: Multi-Arch Docker Container (GHCR)
```bash
docker pull ghcr.io/tokenectomy-labs/razor:latest
docker run -i ghcr.io/tokenectomy-labs/razor:latest --mcp
```

---

## ⚖️ Edition Comparison

| Capability | Razor (Community OSS) | Sentinel (Commercial Tier) |
|---|:---:|:---:|
| **Polyglot Stack Trace Surgery** | Yes (4 Languages) | Yes (All 7 Languages) |
| **O(N) ReDoS-Safe Secret Redaction** | Yes | Yes |
| **JSON-RPC 2.0 MCP Server** | Yes | Yes |
| **AI Gateway Reverse Proxy (`--proxy`)** | Yes | Yes |
| **SHA-256 Idempotency Cache (24h TTL)** | Yes | Yes |
| **FinOps Metrics Dashboard** | Yes | Yes |
| **Tree-sitter AST Syntax Healing** | — | **Yes** |
| **Anti-Hallucination Scope Guard** | — | **Yes** |
| **Automated Test Rollback (0 Dirty Diff)** | — | **Yes** |
| **Multi-File Atomic Transactions** | — | **Yes** |
| **Time Machine Undo Engine (`--undo`)** | — | **Yes** |
| **Autonomous Healing State Machine** | — | **Yes** |

> **Need Enterprise AST Self-Healing?** [Explore the Sentinel Tier](https://tokenectomy.gumroad.com/l/kiznsu)

---

## 🗺️ Roadmap & Milestones

| Milestone / Capability | Status | Target |
|---|:---:|:---:|
| Core Polyglot Log Surgery & $O(N)$ ReDoS Redaction | ✅ Complete | v1.0.0 |
| AI Gateway Reverse Proxy (`--proxy`) & Idempotency Cache | ✅ Complete | v1.1.0 |
| Multi-arch Docker & GitHub Actions Marketplace Action | ✅ Complete | v1.1.3 |
| Static AST Analysis Engine (`analyze_code`) & UTF-16 LSP | ✅ Complete | v1.1.5 |
| Glama.ai Tool Definition Quality Score (TDQS Grade A) | ✅ Complete | v1.1.5 |
| Precompiled Standalone Binaries (Linux, macOS, Windows) | ✅ Complete | v1.1.6 |
| Official MCP Registry Listing (`io.github.Tokenectomy-Labs/razor`) | ✅ Complete | v1.1.7 |
| `mcpservers.org` Directory Listing | ✅ Complete | v1.1.7 |
| Java/Kotlin (Spring Boot 3) & C/C++ (ASan) Extractors | ✅ Complete | v1.2.0 |
| `awesome-mcp-servers` Community Catalog Listing | 🔄 In Progress | v1.2.0 |
| User-defined custom redaction patterns (`~/.tokenectomy.toml`) | 📋 Planned | v1.3.0 |
| Native VS Code & JetBrains companion extensions | 📋 Planned | v1.4.0 |
| Server-Sent Events (SSE) remote MCP transport | 📋 Planned | v1.4.0 |

---

## 🔒 Security & Invariants

- **Zero-Knowledge Architecture:** All parsing, filtering, and secret redaction execute on physical local hardware. No logs are ever transmitted to third-party telemetry servers.
- **Strict Linear $O(N)$ ReDoS Immunity:** All pattern matchers utilize finite automaton evaluation with linear time guarantees, repelling catastrophic backtracking attacks.
- **Path Traversal Boundary Isolation:** File operations are strictly locked within the active workspace root (`CWD`). Path traversals (`../`) and unauthorized symlinks are blocked.
- **Safe Rust Implementation:** Core execution paths enforce safe Rust memory guarantees with bounded stream readers (`.take()`) preventing resource exhaustion.

For vulnerability disclosures, please review our [Security Policy](SECURITY.md).

---

## 🤝 Community & Resources

- 🌐 **[Official Website](https://tokenectomy-web.vercel.app)**
- 📖 **[Documentation](https://tokenectomy-labs.github.io/Tokenectomy)**
- 🏗️ **[System Architecture](ARCHITECTURE.md)**
- 📋 **[Changelog](CHANGELOG.md)**
- 🤝 **[Contributing Guidelines](CONTRIBUTING.md)**
- 🔒 **[Security Policy](SECURITY.md)**

---

<div align="center">
  <p><b>Tokenectomy Labs</b> &bull; Autonomous M2M Sub-Cortex</p>
  <p>Engineered with precision by <b><a href="https://github.com/daffa2555">Daffa (@daffa2555)</a></b></p>
  <p>Licensed under the <a href="LICENSE">MIT License</a></p>
</div>
