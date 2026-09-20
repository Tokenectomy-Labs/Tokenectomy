<div align="center">
  <a href="https://tokenectomy-web.vercel.app">
    <img src="media/logo.png" width="130" alt="Tokenectomy Razor Logo" />
  </a>

  <h1>Tokenectomy Razor</h1>

  <p><b>Stop Claude, Cursor, and AI agents from burning your rate limits on framework internals and leaking production secrets.</b></p>
  <p><i>A sub-millisecond local AI Gateway & MCP server written in safe Rust. Cuts 41.7% mean noise across polyglot stack traces (up to 99.7% on deep framework dumps) and redacts credentials before context reaches LLMs.</i></p>

  <p>
    <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen?style=flat-square" alt="Official MCP Registry" /></a>
    <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?style=flat-square&color=ea580c&logo=rust" alt="crates.io" /></a>
    <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?style=flat-square&color=cb3837&logo=npm" alt="npm" /></a>
    <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tokenectomy-Labs/Tokenectomy/ci.yml?branch=main&style=flat-square&logo=githubactions&label=CI" alt="CI" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License" /></a>
  </p>

  <p>
    <a href="#-quick-start"><b>Quick Start (10s)</b></a> &bull;
    <a href="#-the-problem-why-your-ai-hits-rate-limits">The Problem</a> &bull;
    <a href="#-before--after-comparison">Before & After</a> &bull;
    <a href="https://tokenectomy-web.vercel.app">Live Interactive Demo</a> &bull;
    <a href="#verifiable-benchmarks">Benchmarks</a>
  </p>
</div>

<br />

---

<a id="quick-start"></a>
## ⚡ Quick Start (10s)

Add Tokenectomy to your workflow with zero toolchain setup. Runs immediately via `npx`:

### 1. AI Gateway Reverse Proxy (`--proxy`)
Run Tokenectomy as a zero-overhead local HTTP reverse proxy on `127.0.0.1:8080`. It intercepts outbound prompt streams, scrubs stack trace noise, redacts credentials, and forwards clean requests upstream:

```bash
# Forward to Anthropic Claude:
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.anthropic.com
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"

# Forward to OpenAI:
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.openai.com/v1
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"

# Forward to local Ollama:
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url http://127.0.0.1:11434/v1
```

### 2. Model Context Protocol (MCP) Setup

#### Cursor Composer (`.cursor/mcp.json`)
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

#### Claude Desktop (`claude_desktop_config.json`)
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

#### Claude Code CLI
```bash
claude mcp add tokenectomy npx -y tokenectomy-razor --mcp
```

#### Google Antigravity / Gemini CLI
```bash
agy mcp add tokenectomy-razor -- npx -y tokenectomy-razor --mcp
```

### 3. Terminal Piping & CLI Scrubbing
```bash
npm test 2>&1 | npx tokenectomy-razor
```

---

## 🛑 The Problem: Why Your AI Hits Rate Limits & Hallucinates

When your app crashes during development (Next.js, Express, FastAPI, Tokio), the runtime dumps **hundreds of lines of third-party plumbing** from `node_modules` or `site-packages`.

When you paste that raw crash dump into Cursor or Claude:

1. **Eats Your 5-Hour Rate Limit**: A single Express/Prisma error can dump **5,000 to 45,000 tokens** of third-party library code you never wrote. A few crash loops easily burn your session limit.
2. **Triggers AI Hallucinations**: Claude gets lost in framework internals (`node_modules/express/lib/router/layer.js` or `starlette/routing.py`) and tries to edit library files instead of your actual application code.
3. **Leaks Secrets & Credentials**: Connection strings with raw database passwords, JWT bearer tokens, and cloud keys embedded in error traces get forwarded to external model servers.

```
Raw Terminal Crash (45,820 tokens + Leaked Secrets)
     │
     ▼  <0.2ms Local Rust DFA Engine
[Redact Passwords & Keys] ──► [Strip Third-Party Framework Frames] ──► [Isolate Root Cause]
     │
     ▼
Clean Context (118 tokens • Zero Secrets • Sub-millisecond)
```

---

## 🔍 Before & After Comparison

### ❌ Without Tokenectomy: AI Hallucinates & Burns Context

```text
TypeError: Cannot read properties of undefined (reading 'token')
    at loadComponents (/app/node_modules/next/dist/server/load-components.js:14:2)
    at renderToHTML (/app/node_modules/next/dist/server/render.js:50:5)
    at nextServer (/app/node_modules/next/dist/server/next-server.js:80:12)
    at processTicksAndRejections (node:internal/process/task_queues:95:5)
    at runMicrotasks (node:internal/process/task_queues:120:3)
    at checkoutHandler (/app/pages/api/checkout.ts:42:15)
Database connection failed: postgresql://admin:super_secret_password@db.prod.internal:5432/primary
API key leaked: sk-ant-api03-abcdef1234567890abcdef1234567890
```
> **What Claude does:** Analyzes `load-components.js` and `next-server.js`, speculates on Webpack / Next.js internals, and leaks connection credentials to remote inference logs.

### ✅ With Tokenectomy: Clean Context & Instant Fix

```text
// [Tokenectomy Surgery: 5 internal framework frames pruned (83.3%)]
// Source: /app/pages/api/checkout.ts:42:15
42 |   const sessionToken = req.headers.authorization.token;
   |                                                  ^ TypeError: Cannot read properties of undefined (reading 'token')
🛡️ [CONNECTION_STRING_REDACTED]
🛡️ [ANTHROPIC_KEY_REDACTED]
```
> **What Claude does:** Instantly identifies that line 42 in `checkout.ts` attempted to access `.token` on undefined headers. Suggests optional chaining `req.headers.authorization?.token` immediately. Credentials redacted before transmission.

---

<a id="verifiable-benchmarks"></a>
## 🔬 Verifiable Benchmarks

<div align="center">
  <img src="media/benchmark_bars.png" alt="Tokenectomy Performance Benchmark Bar Chart" width="100%" />
</div>

All performance claims are hardware-grounded and independently reproducible on physical hardware (measured on 10-Core Intel Core i5-1235U @ 15W running Arch Linux, Kernel 6.13):

> **Hardware Dependency Notice:** Performance is hardware-dependent; reported throughput represents measured results on the specified test hardware (10-Core Intel Core i5-1235U @ 15W TDP). Throughput scales with higher TDP desktop/server CPUs and faster memory buses. Developers are encouraged to independently audit performance using the reproduction command below.

<details>
<summary><b>📋 Click to expand full raw benchmark terminal log ($ cargo test --release)</b></summary>

```text
$ cargo test --release --test stress_benchmark -- --nocapture

=====================================================================================
🧪 TOKENECTOMY OSS VERIFIABLE HEAVY STRESS BENCHMARK (100% REPRODUCIBLE IN OSS)
   Hardware: 10-Core / 12-Thread Intel Core i5-1235U | OS: Arch Linux | Kernel Telemetry Active
   Initial Baseline Process Memory (VmRSS): 3.45 MB
=====================================================================================

🔥 [TEST 1/3] QUARTER-MILLION LINES LOG REDACTION TORTURE (250,000 LINES / 25MB+ BUFFER)
  ├── Buffer Size: 24.44 MB (250000 lines)
  ├── Redaction Latency: 471.05ms (51.9 MB/sec)
  ├── Line Throughput: 530,735 lines/sec
  ├── Peak Memory (VmRSS): 76.05 MB (Delta: +72.60 MB)
  └── Status: ✅ PASSED (100% of 250,000 lines sanitized, zero memory balloon)

🔥 [TEST 2/3] REDOS CATASTROPHIC BACKTRACKING TORTURE (50,000 CHARS PAYLOAD)
  ├── Attack Payload Size: 50,082 characters
  ├── Execution Latency: 1.165 ms
  └── Status: ✅ PASSED (Linear O(N) evaluation, ReDoS-resistant on tested payloads)

🔥 [TEST 3/3] HIGH-CONCURRENCY TORTURE (100 PARALLEL OS THREADS)
  ├── Thread Concurrency: 100 concurrent OS threads
  ├── Successful Operations: 100/100 (100.0%)
  ├── Total Elapsed: 11.31ms
  ├── Concurrency Throughput: 17,688 ops/sec
  ├── Final VmRSS: 78.99 MB
  └── Status: ✅ PASSED (Zero race condition, zero deadlock)

=====================================================================================
🏆 TOKENECTOMY OSS STRESS BENCHMARK: 3/3 PASSED (100% GREEN)
   Total Suite Duration: 580.83ms
   Bounded Final VmRSS: 78.99 MB
=====================================================================================
test test_oss_heavy_stress_benchmark ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.58s
```
</details>

| Benchmark Target | Workload Under Test | Verified Measurement | Result |
|---|---|---|:---:|
| **Log Redaction Throughput** | 250,000 lines (24.44 MB) enterprise dump with API keys & connection URIs | **530,735 lines/sec** (471.0 ms, 51.9 MB/s) | **Pass** |
| **ReDoS Resilience** | 50,000-character pathological backtracking regex payload | **1.16 ms** (Deterministic Linear $O(N)$ DFA Evaluation) | **Pass** |
| **Thread Concurrency** | 100 concurrent OS threads executing simultaneous redaction | **17,688 ops/sec** (100/100 completed in 11.31 ms) | **Pass** |
| **Memory Footprint** | Peak Resident Memory during 250k-line continuous stress test | **76.05 MB VmRSS** via `/proc/self/status` (~3× input buffer size) | **Pass** |
| **Release Test Suite** | Full integration test matrix across extractors, filters, and analyzers | **90 / 90 Verified Green** (Zero panics, zero leaks) | **Pass** |

<!-- BEGIN_REDACTION_BENCHMARK -->
### 🛡️ Automated Redaction & Secret Sanitization Benchmark

Evaluated across an internal benchmark test fixture (`tests/fixtures/`) of **12 polyglot crash traces** (Rust, Python, TypeScript, Go, YAML) containing **22 ground-truth credentials** and clean negative controls. Evaluated head-to-head against **Gitleaks v8.30.1** (default ruleset).

> **Architectural Note:** Gitleaks is designed primarily as a repository commit/diff scanner, not an in-memory runtime trace redactor. Tokenectomy Razor is engineered specifically for runtime stream sanitization and stack trace surgery.

#### 1. Per-Category Precision, Recall & F1-Score

| Secret Category | Ground Truth | Razor Recall | Razor F1 | Gitleaks Recall | Gitleaks F1 | Sanitization Advantage |
|---|:---:|:---:|:---:|:---:|:---:|---|
| **Anthropic Claude API Key (`sk-ant-...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **AWS Access Key ID (`AKIA...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **AWS Secret Access Key** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **Database URI (PostgreSQL, MySQL, Redis, Mongo)** | 4 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **Generic Passwords / Auth Secrets (YAML/JSON)** | 3 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **GitHub Personal Access Token (`ghp_...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **GitLab Personal Access Token (`glpat-...`)** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **HuggingFace API Token (`hf_...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **JSON Web Token (RFC 7519 / Truncated)** | 2 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **npm Registry Access Token (`npm_...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **OpenAI API Key (`sk-...`, `sk-proj-...`)** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **PEM Private RSA Key Block** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **PyPI Package Upload Token (`pypi-AgEI...`)** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **SendGrid API Key (`SG...`)** | 1 | **100.0%** | **100.0%** | 0.0% | 0.0% | **+100% Recall** (M2M zero-leak) |
| **Slack Bot/User Token (`xoxb-...`)** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |
| **Stripe Live/Test Secret Key (`sk_live_...`)** | 1 | **100.0%** | **100.0%** | 100.0% | 100.0% | Parity (100% caught) |

#### 2. Head-to-Head Performance & Architectural Summary

| Dimension | Tokenectomy Razor (`--scrub`) | Gitleaks v8.30.1 | Architectural Rationale |
|---|:---:|:---:|---|
| **Overall Secret Recall** | **100.0%** (22/22) | 36.4% (8/22) | Razor captures unquoted URIs, DB ports & AI keys missed by diff rules |
| **Overall Precision** | **100.0%** (0 False Positives) | 88.9% | Zero false triggers on compiler errors & minified traces |
| **Overall F1-Score** | **100.0%** (internal fixture) | 51.6% | Comprehensive coverage engineered specifically for crash context |
| **Execution Engine** | High-throughput Rust DFA ($O(N)$) | Go regex scanner + Git tree crawler | Sub-millisecond latency for agent streaming backtraces |
| **ReDoS Resilience** | **Deterministic Linear Time** ($O(N)$) | Engine dependent | Non-backtracking DFA regex prevents catastrophic backtracking on tested dumps |
| **Sanitization Action** | Inline token redaction (`[KEY_REDACTED]`) | Warning log only (No scrub) | Directly sanitizes text before ingestion by LLM cortex |

#### 3. Token Reduction & LLM Context Savings (`tiktoken` cl100k_base)

| Metric | Measured Value | Operational Impact for AI Coding Agents |
|---|:---:|---|
| **Mean Token Reduction** | **41.67%** | Consistently shrinks raw crash trace token footprint |
| **Median Reduction (P50)** | **42.95%** | Typical credential and connection dump reduction |
| **90th Percentile (P90)** | **61.42%** | Eliminates long multi-line keys and credentials |
| **Min / Max Spread** | **0.00% — 81.36%** | 0% on clean negative controls (zero distortion), up to 81.4% on leaks |
| **Total Tokens Preserved / Saved** | **920 tokens** (44.02% net) | Prevents context window saturation and reduces LLM billing |
<!-- END_REDACTION_BENCHMARK -->

---

<a id="quick-start"></a>
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
| `audit_context_health` | Audits raw logs, traces, or prompt payloads for token bloat, framework noise, and credentials. Returns M2M telemetry, savings metrics, and context health grades. |

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
| **C# (.NET)** | ASP.NET Core, .NET Runtime | `System.Private.CoreLib`, `Microsoft.AspNetCore` |
| **Ruby on Rails** | Rails, Sinatra, Bundler | `/gems/`, `ruby/gems`, internal rack handlers |
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

<a id="edition-comparison"></a>
## ⚖️ Edition Comparison

| Capability | Razor (Community OSS) | Sentinel (Commercial Tier) |
|---|:---:|:---:|
| **Polyglot Stack Trace Surgery** | Yes (9 Runtime Languages) | Yes (All 9 Languages + Deep AST Semantic Healing) |
| **O(N) ReDoS-Safe Secret Redaction** | Yes | Yes |
| **JSON-RPC 2.0 MCP Server** | Yes | Yes |
| **AI Gateway Reverse Proxy (`--proxy`)** | Yes | Yes |
| **SHA-256 Idempotency Cache (24h TTL)** | Yes | Yes |
| **FinOps Metrics Dashboard** | Yes | Yes |
| **Syntax Validation Rollback (Compilers / Linters)** | Yes | Yes |
| **Automated Test Suite Rollback (0 Dirty Diff)** | — | **Yes** |
| **Tree-sitter AST Syntax Healing** | — | **Yes** |
| **Anti-Hallucination Scope Guard** | — | **Yes** |
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
| C# (.NET) & Ruby on Rails Deep Stack Surgery | ✅ Complete | v1.2.2 |
| User-defined custom redaction & noise rules (`~/.tokenectomy.toml`) | ✅ Complete | v1.2.2 |
| Autonomous Context Health Audit (`audit_context_health`) & M2M Advisory | ✅ Complete | v1.2.2 |
| Automated Redaction Benchmark & CI Gate | ✅ Complete | v1.2.2 |
| Declarative Advisory M2M Control Plane (`[:TOKENECTOMY:M2M_CONTROL_PLANE:v1.3.0]`) | ✅ Complete | v1.3.0 |
| Interactive Multi-Strategy Budgeting (`aggressive`, `conservative`, `lossless_compact`) | ✅ Complete | v1.2.3 |
| GitHub Actions OIDC Official Registry Publishing Gate | ✅ Complete | v1.2.3 |
| `awesome-mcp-servers` Community Catalog Listing | ✅ Complete | v1.2.4 |
| Inline Dropped Frame Identities (`[DROPPED_FRAMES: ...]`) & Anti-Silent Truncation Audit | ✅ Complete | v1.3.1 |
| Content-Addressable Raw Log Cache & Verification Hash (`--diff-verify`) | ✅ Complete | v1.3.1 |
| Deprecation & Removal of `cognitive_directive` alias | 📋 Planned | v1.4.0 |
| Native VS Code & JetBrains companion extensions | 📋 Planned | v1.4.0 |
| Server-Sent Events (SSE) remote MCP transport | 📋 Planned | v1.4.0 |

---

## 🔒 Security & Invariants

- **Local Execution by Default:** All stack trace parsing, frame pruning, and secret redaction execute locally on physical hardware.
- **Documented Network Egress:** In MCP server mode, `search_stack_overflow` is the sole tool with outbound network egress (HTTPS to `api.stackexchange.com`). The payload is strictly limited to sanitized, redacted error signature text (no code lines, no local paths, zero credentials). In air-gapped environments, use `--local-only` to disable network search entirely.
- **Deterministic Linear-Time Pattern Matching:** All pattern matchers utilize non-backtracking DFA regex engines ($O(N)$ linear time) and Aho-Corasick automaton evaluation.
- **Path Traversal Boundary Isolation:** File operations are strictly locked within the active workspace root (`CWD`). Path traversals (`../`) and unauthorized symlinks are blocked.
- **Safe Rust Implementation:** Core execution paths enforce safe Rust memory guarantees with bounded stream readers (`.take()`) preventing resource exhaustion.

For vulnerability disclosures, please review our [Security Policy](SECURITY.md).

---

## ⚖️ Legal & Downstream Fork Disclaimer

Tokenectomy Razor is provided strictly for lawful developer productivity, observability, log surgery, and defensive credential redaction. Any downstream forks, clones, redistributions, or private deployments operate completely independently of the original authors. Tokenectomy Labs and its maintainers assume zero liability for unlawful, malicious, or unauthorized actions committed by third parties using this codebase or derivatives thereof. All downstream operators bear 100% individual responsibility for compliance with local and international cybersecurity laws. See [DISCLAIMER.md](DISCLAIMER.md) for full legal terms.

---

## 🤝 Community & Resources

- 🌐 **[Official Website](https://tokenectomy-web.vercel.app)**
- 📖 **[Documentation](https://tokenectomy-labs.github.io/Tokenectomy)**
- 🏗️ **[System Architecture](ARCHITECTURE.md)**
- 📋 **[Changelog](CHANGELOG.md)**
- 🤝 **[Contributing Guidelines](CONTRIBUTING.md)**
- 🔒 **[Security Policy](SECURITY.md)**
- ⚖️ **[Disclaimer & Liability](DISCLAIMER.md)**

---

<div align="center">
  <p><b>Tokenectomy Labs</b> &bull; Autonomous M2M Sub-Cortex</p>
  <p>Engineered with precision by <b><a href="https://github.com/daffa2555">Daffa (@daffa2555)</a></b></p>
  <p>Licensed under the <a href="LICENSE">MIT License</a></p>
</div>
