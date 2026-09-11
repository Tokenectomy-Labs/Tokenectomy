# Tokenectomy Razor

> **Fast, deterministic log surgery and secret redaction for AI coding agents — purge 90%+ framework noise, redact credentials with O(N) ReDoS immunity, sub-millisecond latency. Written in safe Rust.**

*Tokenectomy (noun): **token** + **-ectomy** (surgical removal) — the precise excision of wasteful tokens from LLM context windows.*

### High-Performance Log Surgery & Secret Redaction Engine for AI Coding Agents

<p align="left">
  <a href="https://tokenectomy-web.vercel.app"><img src="https://img.shields.io/badge/Website-tokenectomy--web.vercel.app-000000?style=flat&logo=vercel" alt="Tokenectomy Razor official website" /></a>
  <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?logo=rust" alt="Tokenectomy Razor crate version on crates.io" /></a>
  <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm" alt="Tokenectomy Razor npm package version" /></a>
  <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml"><img src="https://github.com/Tokenectomy-Labs/Tokenectomy/actions/workflows/ci.yml/badge.svg" alt="Tokenectomy Razor CI build status" /></a>
  <a href="SECURITY.md"><img src="https://img.shields.io/badge/Security-Audited%20(RustSec)-2ea44f?logo=rust" alt="Tokenectomy Razor RustSec security audit status" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="Tokenectomy Razor MIT License" /></a>
  <a href="https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy"><img src="https://img.shields.io/badge/Glama.ai-Tokenectomy--Razor-purple" alt="Tokenectomy Razor on Glama.ai" /></a>
  <a href="https://mcpservers.org/servers/tokenectomy-labs/tokenectomy"><img src="https://mcpservers.org/badge.svg" alt="Listed on mcpservers.org" /></a>
  <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen" alt="Official MCP Registry" /></a>
  <a href="https://github.com/marketplace/actions/tokenectomy-razor"><img src="https://img.shields.io/badge/GitHub%20Marketplace-Tokenectomy%20Razor-blue?logo=githubactions" alt="Tokenectomy Razor GitHub Actions Marketplace" /></a>
  <a href="https://tokenectomy-labs.github.io/Tokenectomy"><img src="https://img.shields.io/badge/Docs-GitHub%20Pages-blue?logo=googledocs" alt="Tokenectomy Razor documentation site" /></a>
</p>

- **Official MCP Registry:** `mcp-name: io.github.Tokenectomy-Labs/razor`

---

## 📋 Table of Contents

- [What It Does](#what-it-does)
- [Key Features](#technical-highlights)
- [Benchmarks](#verifiable-benchmarks)
- [Installation](#installation)
- [MCP Integration](#model-context-protocol-mcp-integration)
- [Usage by Use Case](#quick-start)
- [Advanced Features](#advanced-usage)
- [Comparison](#how-tokenectomy-compares)
- [FAQ](#frequently-asked-questions)
- [Roadmap](#roadmap)
- [Security](#security--reliability-invariants)
- [Contributing](#contributing)

---

## What It Does

Tokenectomy Razor is an autonomous, machine-to-machine (M2M) Model Context Protocol (MCP) server and stream processing engine written in safe Rust. It intercepts error logs from AI agents, strips 90%+ of framework noise, automatically redacts secrets (JWTs, API keys, database credentials), and caches sanitized contexts with a 24-hour TTL—all without sending raw data to external services.

### In 30 Seconds

**The Problem:**
- AI agents waste tokens on framework noise (`node_modules`, `site-packages`, `.cargo/registry`)
- Sensitive credentials accidentally leak into LLM logs (AWS keys, database URLs, API tokens)
- Repeated identical errors cost money for every retry

**The Solution:**
```
Raw Error Log (38K tokens + secrets)
    ↓
[Redact secrets locally] → [Filter framework frames] → [Extract user code]
    ↓
Sanitized Context (2K tokens, no secrets) → Safe to send to LLM
```

### Real Example

**Before:**
```bash
$ cat error.log | head -20
Error in /home/user/.cargo/registry/src-xxx/tokio-1.35/src/runtime/mod.rs:12345
  at /home/user/.cargo/registry/src-yyy/serde/src/lib.rs:456
  Database connection failed: postgresql://admin:secretpass@db.example.com:5432/mydb
  JWT Auth token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0...
  [... 500+ more framework frames ...]
```

**After:**
```bash
$ cat error.log | razor --scrub
Error in /home/user/src/main.rs:42
  at /home/user/src/utils.rs:18
  Database connection failed: [CONNECTION_STRING_REDACTED]
  JWT Auth token: [JWT_REDACTED]
```

**Benefits:**
- ✅ 95% smaller context (2K vs 38K tokens) → Save money on LLM API calls
- ✅ Zero secrets in logs → Sleep better at night
- ✅ Identical errors cached → Second retry costs $0

---

## Technical Highlights

```
                  ┌──────────────────────────────────────────────┐
   Agent Error     │              TOKENECTOMY RAZOR               │    Sanitized Context
   Dump (38K toks) │  - Polyglot Stack Frame Filter               │ ──►  (2K toks) ──► LLM
  ────────────────►│  - Deterministic Secret Redactor (O(N))      │
                   │  - SHA-256 Idempotency Cache (24h TTL)       │
                   └──────────────────────────────────────────────┘
```

- **Deep Polyglot Trace Surgery**: In-memory parsing across Rust, Python, TypeScript/JavaScript, Go, Java/Kotlin (Spring Boot 3, Tomcat, Hibernate, Netty, Undertow, HikariCP), C/C++ (AddressSanitizer, GDB, glibc), and PHP. Surgically filters noisy framework internals and runtime boilerplate while isolating genuine user application code frames.
- **Static AST Code Analysis Engine (`analyze_code`)**: High-throughput static AST analysis detecting unclosed handles, resource leaks, and security vulnerabilities with bounded execution limits (50K AST nodes, 10 MB file limit) and precise LSP UTF-16 coordinates.
- **Glama Grade A TDQS Compliance**: 100% Tool Definition Quality Score with explicit schema boundaries, runtime preconditions, and full disclosure across all MCP tools.
- **AI Gateway Reverse Proxy (`--proxy`)**: Transparently intercepts prompt streams on `127.0.0.1:8080`, performing real-time token excision and credential sanitization before upstream forwarding to OpenAI, Anthropic, or Ollama.
- **Zero-Knowledge Secret Redaction**: Linear-time deterministic regex engine strips JWTs, API tokens, cloud access keys, connection strings, and private keys prior to network transmission. All processing happens locally.
- **SHA-256 Idempotency Cache**: Stores deterministic responses with a 24-hour TTL. Repeated CI/CD or agent loop failures incur zero upstream API cost.
- **Path Traversal Containment**: All MCP filesystem access is canonicalized and locked to the workspace root boundary (`CWD`). No `../` escapes or symlink breakouts.
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

### Understanding the Benchmarks

| Metric | Why It Matters | What To Expect |
|--------|---|---|
| **73.3 MB/sec redaction throughput** | Most logs are <5MB; you'll redact them in milliseconds | <10ms for typical CI logs |
| **1.44ms ReDoS immunity** | Prevents malicious log payloads from DoS'ing your system | Safe to use in production with untrusted input |
| **76.24 MB peak memory** | Suitable for constrained CI/CD runners (GitHub Actions, GitLab) | Fits within 256MB limits comfortably |
| **7,312 ops/sec concurrent** | Multiple AI agents querying simultaneously | 100 concurrent requests handled safely |

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

### Method 3: Precompiled Native Binaries (GitHub Releases)

Download zero-dependency, precompiled standalone binaries directly from [GitHub Releases](https://github.com/Tokenectomy-Labs/Tokenectomy/releases):

- **Linux:** `tokenectomy-linux-x86_64` (glibc), `tokenectomy-linux-x86_64-musl`, `tokenectomy-linux-aarch64`
- **macOS:** `tokenectomy-darwin-arm64` (Apple Silicon M1/M2/M3/M4), `tokenectomy-darwin-x86_64` (Intel)
- **Windows:** `tokenectomy-windows-x86_64.exe`

### Method 4: Build from Source

```bash
git clone https://github.com/Tokenectomy-Labs/Tokenectomy.git
cd Tokenectomy
cargo build --release
sudo cp target/release/razor /usr/local/bin/razor
```

### Method 5: Multi-Arch Container (GHCR)

```bash
docker pull ghcr.io/tokenectomy-labs/razor:latest
docker run -it ghcr.io/tokenectomy-labs/razor:latest --help
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

### VS Code (Native MCP / GitHub Copilot Agent / Continue)

For VS Code with native MCP support, create or edit `.vscode/mcp.json` in your workspace:

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

*Or use the ultra-low latency native binary (if installed via `cargo install tokenectomy`):*

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "razor",
      "args": ["--mcp"]
    }
  }
}
```

### VS Code + Cline

Open Cline Settings in VS Code (or edit `cline_mcp_settings.json`):
- **macOS:** `~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`
- **Linux:** `~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json`
- **Windows:** `%APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json`

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"],
      "disabled": false,
      "autoApprove": [
        "get_error_context",
        "search_stack_overflow",
        "analyze_code"
      ]
    }
  }
}
```

### VS Code + Roo Code

In Roo Code Settings (or edit `cline_mcp_settings.json` in Roo storage):
- **Linux:** `~/.config/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json`
- **macOS:** `~/Library/Application Support/Code/User/globalStorage/rooveterinaryinc.roo-cline/settings/cline_mcp_settings.json`

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

### Windsurf (Codeium)

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
| `analyze_code` | Performs static AST code analysis to detect resource leaks, security vulnerabilities, and code defects with bounded execution limits and precise LSP UTF-16 coordinates. |

---

## Quick Start

### I use Claude Desktop

```bash
# 1. Add to claude_desktop_config.json (see MCP Integration section above)
# 2. When Claude encounters errors, it automatically uses "get_error_context" tool
# 3. Errors stay sanitized without any additional setup
```

### I use GitHub Actions

```yaml
# Add to your workflow (.github/workflows/build.yml)
- name: Sanitize Build Failure Log
  if: failure()
  uses: daffa2555/tokenectomy-action@v1
  with:
    log-file: 'build.log'
    output-file: 'sanitized.log'

# Now you can safely share sanitized.log without leak concerns
```

Parameters:
| Parameter | Type | Default | Description |
|---|---|---|---|
| `log-file` | String | `''` | Path to raw error log file to process |
| `log-content` | String | `''` | Direct string content if file is not specified |
| `output-file` | String | `tokenectomy-sanitized.log` | Path for scrubbed output file |
| `version` | String | `v1.2.1` | Binary release target version |

### I want max privacy (air-gapped environment)

```bash
# All redaction happens locally—no network calls except to your LLM
cargo install tokenectomy

# Process logs without any cloud services
echo $ERROR_LOG | razor --scrub --local-only

# Or from a file:
razor --scrub --file /var/log/app/error.log > sanitized.log
```

---

## Advanced Usage

### Standalone CLI

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

### AI Gateway Reverse Proxy Mode

Tokenectomy Razor can operate as a transparent local HTTP reverse proxy. It sits between client applications and upstream LLM providers (OpenAI, Anthropic, Ollama, OpenRouter), performing real-time token excision and credential sanitization before upstream forwarding.

**Local Development (Default Loopback):**

```bash
# Forward to OpenAI
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.openai.com/v1

# Forward to local Ollama instance
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url http://127.0.0.1:11434/v1
```

Point any standard SDK or IDE client to the local proxy:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
# Now all API calls are automatically sanitized
```

**Production Proxy Hardening**

Binding to external interfaces (`0.0.0.0`) requires explicit token authorization:

```bash
razor --proxy --proxy-bind 0.0.0.0:8080 --upstream-url https://api.openai.com/v1 --allow-remote --proxy-token "YOUR_SECURE_TOKEN"
```

Resource limits enforced: `MAX_HEADER_SIZE` (64 KB), `MAX_BODY_SIZE` (10 MB), client/upstream timeouts (30s / 60s), and a 128-connection concurrency cap.

### Configuration

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

**Note:** Currently shipped with robust extractors for **Rust, Python, TypeScript/JavaScript, and Go**. Java/Kotlin, C/C++, and PHP support is coming in v1.2. See [#1](https://github.com/Tokenectomy-Labs/Tokenectomy/issues) for progress tracking.

---

## How Tokenectomy Compares

| Feature | Tokenectomy | Splunk Log Obfuscation | Datadog Logs | git-secrets |
|---------|---|---|---|---|
| Instant setup (no agent install) | ✅ | ❌ | ❌ | ✅ |
| Works with AI agents (MCP) | ✅ | ❌ | ❌ | ❌ |
| Local-only processing | ✅ | ❌ | ❌ | ✅ |
| Polyglot stack traces | ✅ | ✅ | ✅ | ❌ |
| Redaction caching (cost savings) | ✅ | ❌ | ✅ | ❌ |
| Open source (MIT) | ✅ | ❌ | ❌ | ✅ |
| **Price** | **Free OSS** | **$$$ /mo** | **$$$ /mo** | **Free** |

**When to Use Tokenectomy:**
- ✅ You use AI coding agents (Claude, Cursor, Cline, etc.)
- ✅ You care about privacy & local-first processing
- ✅ You want to reduce LLM token costs
- ✅ You're worried about secret leakage in logs

**When to Use Something Else:**
- ❌ You only need static secret scanning → use `truffleHog`, `detect-secrets`
- ❌ You need real-time monitoring dashboards → use `Datadog`, `New Relic`, `Splunk`
- ❌ Your error logs are naturally <100 tokens → overhead not worth it
- ❌ You're fully air-gapped → Actually Tokenectomy is perfect! (100% local processing)

---

## Frequently Asked Questions

**Q: Does Tokenectomy send my logs to external servers?**

A: No. All redaction, parsing, and filtering happens locally on your machine. The only network call is to your chosen LLM (OpenAI, Anthropic, Ollama) **after** sanitization is complete. See [SECURITY.md](SECURITY.md) for the zero-knowledge guarantee.

---

**Q: What secrets does Tokenectomy redact?**

A: GitHub PATs, AWS keys, OpenAI/Anthropic API keys, JWTs, database connection strings (PostgreSQL, MySQL, MongoDB, Redis), private SSH keys, Slack/Discord webhooks, and more. Full list in [src/redact.rs](src/redact.rs).

---

**Q: What if my secret doesn't match the redaction patterns?**

A: File an issue with an example (sanitized). We'll add the pattern. For now, you can add custom patterns in `~/.tokenectomy.toml` (feature coming in v1.3).

---

**Q: Is Tokenectomy safe for production?**

A: Yes. Written in **safe Rust** (zero unsafe code in security paths), **ReDoS-immune**, and **audited via RustSec**. See [SECURITY.md](SECURITY.md) for full details.

---

**Q: Can I use Tokenectomy offline?**

A: Yes—except Stack Overflow search. Use `--local-only` flag to disable all network access (except your LLM).

---

**Q: How do I remove Tokenectomy?**

A: Simply uninstall:
```bash
npm uninstall -g tokenectomy-razor
# OR
cargo uninstall tokenectomy
```
Zero config cleanup needed—no files left behind.

---

**Q: Can I use Tokenectomy in my CI/CD pipeline?**

A: Yes! Use the GitHub Marketplace action (see Quick Start section) or the Docker container. Works with GitHub Actions, GitLab CI, Jenkins, etc.

---

**Q: What's the difference between Razor (OSS) and Sentinel (Commercial)?**

A: Razor is the free, community version with all essential features. Sentinel adds advanced capabilities like tree-sitter AST healing, anti-hallucination guards, and time-machine undo. See [Edition Comparison](#edition-comparison) below.

---

## Edition Comparison

| Capability | Razor (Community OSS) | Sentinel (Commercial Tier) |
|---|:---:|:---:|
| Framework Log Filtering | Yes | Yes |
| Polyglot Trace Extraction (4 Languages) | Yes | Yes (7 Languages) |
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

**Interested in Sentinel?** [View pricing & features](https://tokenectomy.gumroad.com/l/kiznsu)

---

## Roadmap

| Milestone / Capability | Status | Target Version |
|---|:---:|:---:|
| Core Polyglot Log Surgery & $O(N)$ ReDoS-Immune Secret Redaction | ✅ Complete | v1.0.0 |
| AI Gateway Reverse Proxy (`--proxy`) & SHA-256 Idempotency Cache | ✅ Complete | v1.1.0 |
| Multi-arch Docker (GHCR) & GitHub Actions Marketplace Action | ✅ Complete | v1.1.3 |
| Static AST Code Analysis Engine (`analyze_code`) & UTF-16 LSP Offsets | ✅ Complete | v1.1.5 |
| Glama.ai Tool Definition Quality Score (TDQS Grade A) | ✅ Complete | v1.1.5 |
| Standalone Multi-Arch Precompiled Binaries (Linux, macOS, Windows) | ✅ Complete | v1.1.6 |
| Official Anthropic MCP Registry Listing (`io.github.Tokenectomy-Labs/razor`) | ✅ Complete | v1.1.7 |
| `mcpservers.org` Official Directory Synchronization & Badge | ✅ Complete | v1.1.7 |
| Java / Kotlin (Spring Boot 3, Gradle) framework stack trace extractors | ✅ Complete | v1.2.0 |
| C / C++ (AddressSanitizer & GDB/LLDB) backtrace cleaner | ✅ Complete | v1.2.0 |
| Go goroutine panic & dump compression heuristics | ✅ Complete | v1.2.0 |
| `awesome-mcp-servers` Community Directory Catalog Listing | 🔄 In Progress | v1.2.0 |
| User-defined custom redaction patterns via `~/.tokenectomy.toml` | 📋 Planned | v1.3.0 |
| Configurable noise thresholds & custom exclude patterns | 📋 Planned | v1.3.0 |
| Local agent token savings & cost reduction metrics dashboard | 📋 Planned | v1.3.0 |
| Native VS Code & JetBrains companion extensions | 📋 Planned | v1.4.0 |
| Server-Sent Events (SSE) remote MCP transport | 📋 Planned | v1.4.0 |
| Tree-sitter AST syntax healing & repair | ✅ Sentinel (Paid) | Available Now |
| Multi-file atomic transactions & Time-machine rollback (`--undo`) | ✅ Sentinel (Paid) | Available Now |

---

## Security & Reliability Invariants

- **Zero-Knowledge Processing**: All scanning and redaction occurs on local hardware before data leaves the system boundary.
- **ReDoS Immunity**: All pattern matchers utilize finite automaton evaluation with linear time guarantees. Verified in benchmarks.
- **Path Traversal Isolation**: File operations are strictly locked within workspace boundaries via `WorkspaceBoundary` security module.
- **Memory Safety**: Implemented in safe Rust with bounded stream readers (`.take()`) preventing resource exhaustion attacks.
- **Audit Verification**: Continuous dependency auditing maintained via RustSec advisory databases.

**Vulnerability Disclosure:** See [SECURITY.md](SECURITY.md) for responsible disclosure procedures.

---

## Contributing

Found a bug? Have a feature request? Want to add support for a new language?

1. **Issues:** [github.com/Tokenectomy-Labs/Tokenectomy/issues](https://github.com/Tokenectomy-Labs/Tokenectomy/issues)
2. **Pull Requests:** Fork, create a feature branch, and submit a PR with tests
3. **Security:** See [SECURITY.md](SECURITY.md) for private vulnerability disclosure

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed contribution guidelines.

---

## Resources

- 📖 **[Documentation](https://tokenectomy-labs.github.io/Tokenectomy)** — Full guides, API reference, and integration tutorials
- 📋 **[Changelog](CHANGELOG.md)** — Release history and notable changes
- 🤝 **[Contributing](CONTRIBUTING.md)** — How to contribute, development workflow, and testing
- 🔒 **[Security Policy](SECURITY.md)** — Vulnerability disclosure and audit details
- 🏗️ **[Architecture](ARCHITECTURE.md)** — Internal design and system architecture

---

## License

MIT License. See [LICENSE](LICENSE) for full terms.

---

**Made with ❤️ by [@daffa2555](https://github.com/daffa2555)**

Questions? Open an issue or start a discussion on [GitHub](https://github.com/Tokenectomy-Labs/Tokenectomy).
