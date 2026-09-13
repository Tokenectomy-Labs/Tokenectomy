<div align="center">
  <a href="https://tokenectomy-web.vercel.app">
    <img src="media/logo.png" width="140" alt="Tokenectomy Razor Logo" />
  </a>

  <h1>Tokenectomy Razor</h1>

  <p><b>The autonomous M2M sub-cortex for AI coding agents</b></p>

  <p>
    <a href="https://tokenectomy-web.vercel.app"><img src="https://img.shields.io/badge/Website-tokenectomy--web.vercel.app-000000?style=flat-square&logo=vercel" alt="Tokenectomy Razor official website" /></a>
    <a href="https://tokenectomy-labs.github.io/Tokenectomy"><img src="https://img.shields.io/badge/Documentation-GitHub%20Pages-2563eb?style=flat-square&logo=googledocs" alt="Documentation" /></a>
    <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen?style=flat-square" alt="Official MCP Registry" /></a>
    <br />
    <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?style=flat-square&color=ea580c&logo=rust" alt="crates.io" /></a>
    <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?style=flat-square&color=cb3837&logo=npm" alt="npm" /></a>
    <a href="SECURITY.md"><img src="https://img.shields.io/badge/Security%20Audit-RustSec%20Audited-2ea44f?style=flat-square&logo=rust" alt="Security Audit" /></a>
    <a href="https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy"><img src="https://img.shields.io/badge/Glama.ai-Grade%20A%20TDQS-purple?style=flat-square" alt="Glama.ai" /></a>
    <br />
    <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/Tokenectomy-Labs/Tokenectomy/ci.yml?branch=main&style=flat-square&logo=githubactions&label=CI" alt="CI" /></a>
    <a href="https://github.com/marketplace/actions/tokenectomy-razor"><img src="https://img.shields.io/badge/GitHub%20Marketplace-Action%20v1-blue?style=flat-square&logo=githubactions" alt="GitHub Marketplace" /></a>
    <a href="https://mcpservers.org/servers/tokenectomy-labs/tokenectomy"><img src="https://img.shields.io/badge/mcpservers.org-Listed-blueviolet?style=flat-square" alt="mcpservers.org" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License" /></a>
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

<a id="verifiable-benchmarks"></a>
## 🔬 Verifiable Benchmarks

<div align="center">
  <img src="media/benchmark_bars.png" alt="Tokenectomy Performance Benchmark Bar Chart" width="100%" />
</div>

All performance claims are hardware-grounded and independently reproducible on physical hardware (measured on 10-Core Intel Core i5-1235U @ 15W running Arch Linux, Kernel 6.13):

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
  └── Status: ✅ PASSED (Linear O(N) evaluation, 100% ReDoS Immune)

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

| Benchmark Target | Workload Under Test | Verified Measurement | Result |
|---|---|---|:---:|
| **Log Redaction Throughput** | 250,000 lines (24.44 MB) enterprise dump with API keys & connection URIs | **530,735 lines/sec** (471.0 ms, 51.9 MB/s) | **Pass** |
| **ReDoS Immunity** | 50,000-character pathological backtracking regex payload | **1.16 ms** (Strict Linear $O(N)$ Evaluation) | **Pass** |
| **Thread Concurrency** | 100 concurrent OS threads executing simultaneous redaction | **17,688 ops/sec** (100/100 completed in 11.31 ms) | **Pass** |
| **Memory Footprint** | Peak Resident Memory during 250k-line continuous stress test | **76.05 MB VmRSS** via `/proc/self/status` | **Pass** |
| **Release Test Suite** | Full integration test matrix across extractors, filters, and analyzers | **57 / 57 Verified Green** (Zero panics, zero leaks) | **Pass** |

<!-- BEGIN_REDACTION_BENCHMARK -->
### 🛡️ Automated Redaction & Secret Sanitization Benchmark

Automated evaluation across **12 polyglot crash traces** (Rust, Python, TypeScript, Go, YAML) containing **22 ground-truth credentials** and clean negative controls. Evaluated head-to-head against **Gitleaks v8.30.1**.

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
| **Overall F1-Score** | **100.0%** | 51.6% | Comprehensive coverage engineered specifically for crash context |
| **Execution Engine** | Zero-allocation Rust DFA ($O(N)$) | Go regex scanner + Git tree crawler | Sub-millisecond latency for agent streaming backtraces |
| **ReDoS Immunity** | **Guaranteed Linear Time** ($O(N)$) | Engine dependent | Immune to catastrophic backtracking on massive dumps |
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
| C# (.NET) & Ruby on Rails Deep Stack Surgery | ✅ Complete | v1.2.2 |
| User-defined custom redaction & noise rules (`~/.tokenectomy.toml`) | ✅ Complete | v1.2.2 |
| Autonomous Context Health Audit (`audit_context_health`) & M2M Advisory | ✅ Complete | v1.2.2 |
| Automated Redaction Benchmark & CI Gate | ✅ Complete | v1.2.2 |
| M2M Cognitive Anchoring Control Plane (`[:TOKENECTOMY:M2M_CONTROL_PLANE:v1.2.3]`) | ✅ Complete | v1.2.3 |
| Interactive Multi-Strategy Budgeting (`aggressive`, `conservative`, `lossless_compact`) | ✅ Complete | v1.2.3 |
| GitHub Actions OIDC Official Registry Publishing Gate | ✅ Complete | v1.2.3 |
| `awesome-mcp-servers` Community Catalog Listing | 🔄 In Progress | v1.2.4 |
| Native VS Code & JetBrains companion extensions | 📋 Planned | v1.3.0 |
| Server-Sent Events (SSE) remote MCP transport | 📋 Planned | v1.3.0 |

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
