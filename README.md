# Tokenectomy Razor

### High-Performance Log Surgery & Secret Redaction Engine for AI Coding Agents

<p align="left">
  <a href="https://tokenectomy-web.vercel.app"><img src="https://img.shields.io/badge/Website-tokenectomy--web.vercel.app-000000?style=flat&logo=vercel" alt="Website" /></a>
  <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?logo=rust" alt="Crates.io" /></a>
  <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm" alt="npm" /></a>
  <a href="https://github.com/daffa2555/Tokenectomy/actions/workflows/ci.yml"><img src="https://github.com/daffa2555/Tokenectomy/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="SECURITY.md"><img src="https://img.shields.io/badge/Security-Audited%20(RustSec)-2ea44f?logo=rust" alt="Security" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License" /></a>
  <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.daffa2555%2Frazor-brightgreen" alt="Official MCP Registry" /></a>
  <a href="https://github.com/marketplace/actions/tokenectomy-razor"><img src="https://img.shields.io/badge/GitHub%20Marketplace-Tokenectomy%20Razor-blue?logo=githubactions" alt="GitHub Marketplace" /></a>
</p>

Tokenectomy Razor is an autonomous, machine-to-machine (M2M) Model Context Protocol (MCP) server and stream processing engine written in safe Rust. Engineered as a high-throughput background sub-cortex for AI coding assistants (Claude Desktop, Cursor, Cline, Roo Code, Windsurf, Google Antigravity), Razor purges 90%+ of redundant framework dependency frames (`node_modules`, `site-packages`, `.cargo/registry`) from execution traces, enforces zero-knowledge credential redaction ($O(N)$ ReDoS-immune), and operates with sub-millisecond latency.

```
                  ┌──────────────────────────────────────────────┐
  Agent Error     │              TOKENECTOMY RAZOR               │    Sanitized Context
  Dump (38K toks) │  - Polyglot Stack Frame Filter               │ ──►  (2K toks) ──► LLM
 ────────────────►│  - Deterministic Secret Redactor (O(N))      │
                  │  - SHA-256 Idempotency Cache (24h TTL)       │
                  └──────────────────────────────────────────────┘
```

---

## Technical Highlights

- **Polyglot Trace Surgery**: In-memory parsing across Rust, Python, TypeScript/JavaScript, Go, Java/Kotlin, C/C++ (ASan & GDB), and PHP (Laravel/Symfony). Filters noisy dependency frames and isolates first-party source code.
- **AI Gateway Reverse Proxy (`--proxy`)**: Transparently intercepts prompt streams on `127.0.0.1:8080`, performing real-time token excision and credential sanitization before upstream forwarding.
- **Zero-Knowledge Secret Redaction**: Linear-time deterministic regex engine strips JWTs, API tokens, cloud access keys, and connection strings prior to network transmission.
- **SHA-256 Idempotency Cache**: Stores deterministic responses with a 24-hour TTL. Repeated CI/CD or agent loop failures incur zero upstream API cost.
- **Path Traversal Containment**: All MCP filesystem access is canonicalized and locked to the workspace root boundary (`CWD`).
- **M2M Protocol Compliance**: Native JSON-RPC 2.0 stdio server compliant with the official Model Context Protocol specification.

---

## Verifiable Benchmarks

Performance metrics are hardware-grounded and reproducible via standalone benchmark suites:

| Benchmark Target | Workload Under Test | Verified Measurement | Result |
|---|---|---|:---:|
| **High-Volume Log Redaction** | 250,000 lines (24.44 MB) enterprise dump containing API keys and connection URIs | **333.49 ms (73.3 MB/sec, 749,652 lines/sec)** | Pass |
| **ReDoS Resistance** | 50,000-character pathological backtracking string | **1.44 ms** (Linear $O(N)$ evaluation) | Pass |
| **Thread Concurrency** | 100 concurrent OS threads executing simultaneous redaction and extraction | **100/100 completed in 27.35 ms (7,312 ops/sec)** | Pass |
| **Kernel Memory Footprint** | Peak Resident Memory during 250,000-line continuous stress test | **76.24 MB VmRSS** via `/proc/self/status` | Pass |

> **Reproduce locally:**
> ```bash
> cargo test --release --test stress_benchmark -- --nocapture
> ```

---

## Installation

### Method 1: Instant via npx (Recommended for MCP Clients)

No Rust toolchain, native compilation, or manual path setup required:

```bash
npx -y tokenectomy-razor --mcp
```

Or install globally via npm:

```bash
npm install -g tokenectomy-razor
```

### Method 2: Cargo (crates.io)

```bash
cargo install tokenectomy
```

### Method 3: Build from Source

```bash
git clone https://github.com/daffa2555/Tokenectomy.git
cd Tokenectomy
cargo build --release
sudo cp target/release/razor /usr/local/bin/razor
```

### Method 4: Multi-Arch Container (GHCR)

```bash
docker pull ghcr.io/daffa2555/razor:latest
```

---

## Model Context Protocol (MCP) Integration

Configure Tokenectomy Razor as an autonomous background server across major AI agent environments:

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

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

### Cursor

Add to `.cursor/mcp.json` in your project root:

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

### Cline / Roo Code / Windsurf / VS Code

Add to your client configuration (`cline_mcp_settings.json` or `settings.json`):

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

### Google Antigravity CLI

```bash
agy mcp add tokenectomy-razor -- npx -y tokenectomy-razor --mcp
```

### Exposed MCP Tools

| Tool Name | Capability Description |
|---|---|
| `get_error_context` | Performs trace surgery on error dumps, removes framework noise, redacts credentials, and extracts relevant local source context bounded to the workspace. |
| `search_stack_overflow` | Queries Stack Exchange API for relevant error signatures using sanitized search terms. |
| `apply_code_patch` | Applies atomic file modifications with post-write language syntax verification (`cargo check`, `py_compile`, `node --check`) and automated rollback on validation failure. |

---

## AI Gateway Reverse Proxy Mode

Tokenectomy Razor can operate as a transparent local HTTP reverse proxy. It sits between client applications and upstream LLM providers (OpenAI, Anthropic, Ollama, OpenRouter), performing real-time token reduction and secret scrubbing without requiring MCP tool configurations.

```bash
# Forward to OpenAI
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.openai.com/v1

# Forward to local Ollama instance
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url http://127.0.0.1:11434/v1
```

Point any standard SDK or IDE client to the local proxy:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
```

### Production Proxy Hardening

Binding to external interfaces (`0.0.0.0`) requires explicit token authorization:

```bash
razor --proxy --proxy-bind 0.0.0.0:8080 --upstream-url https://api.openai.com/v1 --allow-remote --proxy-token "YOUR_SECURE_TOKEN"
```

Resource limits enforced: `MAX_HEADER_SIZE` (64 KB), `MAX_BODY_SIZE` (10 MB), client/upstream timeouts (30s / 60s), and a 128-connection concurrency cap.

---

## GitHub Actions CI/CD Integration

Sanitize build failure logs and prevent credential leakage in automated workflows:

```yaml
- name: Sanitize Build Failure Log
  if: failure()
  uses: daffa2555/Tokenectomy@v1
  with:
    log-file: 'build.log'
    output-file: 'sanitized.log'
```

| Parameter | Type | Default | Description |
|---|---|---|---|
| `log-file` | String | `''` | Path to raw error log file to process |
| `log-content` | String | `''` | Direct string content if file is not specified |
| `output-file` | String | `tokenectomy-sanitized.log` | Path for scrubbed output file |
| `version` | String | `v1.1.3` | Binary release target version |

---

## Supported Ecosystems

| Language | Primary Frameworks | Excluded Framework Paths |
|---|---|---|
| **Rust** | Tokio, Actix-web, Axum | `.cargo/registry`, `.rustup`, `target/debug/build` |
| **Python** | Django, FastAPI, Flask, PyTorch | `site-packages`, `dist-packages`, `venv`, `__pycache__` |
| **TypeScript / JavaScript** | Next.js, Express, NestJS, Vite | `node_modules`, `.next`, `dist`, webpack internals |
| **Golang** | Gin, Fiber, Stdlib panics | `go/src` (stdlib), `go/pkg/mod`, `vendor` |
| **Java / Kotlin** | Spring Boot, Quarkus, Gradle | `.m2/repository`, `.gradle/caches`, framework internals |
| **C / C++** | GDB Backtraces, AddressSanitizer | `/usr/include`, `/usr/lib`, `vcpkg_installed` |
| **PHP** | Laravel, Symfony | `vendor/composer`, `vendor/symfony`, `vendor/laravel` |

---

## Standalone CLI Usage

In addition to M2M agent mode, Razor provides CLI commands for terminal piping and local shell scripting:

```bash
# Scrub framework frames and output clean log
npm test 2>&1 | razor --scrub > sanitized.log

# Sanitize a specific log file
razor --scrub --file /var/log/app/error.log > sanitized.log

# CLI diagnosis with specific AI provider
razor --file error.log --provider openai
razor --file error.log --provider anthropic
razor --file error.log --local-only
```

---

## Configuration

Configuration values can be set via `~/.tokenectomy.toml`:

```toml
default_provider = "openai"  # openai | anthropic | ollama | mock
openai_api_key = "sk-..."
anthropic_api_key = "sk-ant-..."
ollama_base_url = "http://localhost:11434"
context_lines = 10
max_context_chars = 10000
```

---

## Edition Comparison

| Capability | Razor (Community OSS) | Sentinel (Commercial Tier) |
|---|:---:|:---:|
| Framework Log Filtering | Yes | Yes |
| Polyglot Trace Extraction (7 Languages) | Yes | Yes |
| AI Reverse Proxy Gateway (`--proxy`) | Yes | Yes |
| Stack Overflow Integration | Yes | Yes |
| SHA-256 Idempotency Cache | Yes | Yes |
| ReDoS-Safe Secret Redaction | Yes | Yes |
| MCP Protocol Server (JSON-RPC) | Yes | Yes |
| Bundled Agent Skills | 2 Skills (Spec TDD & Fuzzer) | Full 4 Skills Suite |
| Tree-sitter AST Syntax Healing | No | Yes |
| Anti-Hallucination Scope Guard | No | Yes |
| Automated Test Rollback Loop | No | Yes |
| Multi-File Atomic Transactions | No | Yes |
| Time Machine Undo Engine (`--undo`) | No | Yes |
| True Ectomy Deep Surgery Engine | No | Yes |
| Live DB Port & Docker Diagnostics | No | Yes |

Commercial licenses and enterprise capabilities: **[Tokenectomy Sentinel](https://tokenectomy.gumroad.com/l/kiznsu)**.

---

## Security & Reliability Invariants

- **Zero-Knowledge Processing**: All scanning and redaction occurs on local hardware before data leaves the system boundary.
- **ReDoS Immunity**: All pattern matchers utilize finite automaton evaluation with linear time guarantees.
- **Path Traversal Isolation**: File operations are strictly locked within workspace boundaries.
- **Memory Safety**: Implemented in safe Rust with bounded stream readers (`.take()`) preventing resource exhaustion attacks.
- **Audit Verification**: Continuous dependency auditing maintained via RustSec advisory databases.

Refer to [`SECURITY.md`](SECURITY.md) for vulnerability disclosure procedures.

---

## License

MIT License. See [`LICENSE`](LICENSE) for terms.
