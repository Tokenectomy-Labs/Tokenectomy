# Tokenectomy ⚡ — Autonomous Context Surgery & Secret Shield for AI Agents (M2M MCP Server)

<p align="left">
  <a href="https://tokenectomy.vercel.app"><img src="https://img.shields.io/badge/Website-tokenectomy.vercel.app-000000?style=flat&logo=vercel" alt="Website" /></a>
  <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?logo=rust" alt="Crates.io" /></a>
  <a href="https://github.com/daffa2555/Tokenectomy/actions/workflows/ci.yml"><img src="https://github.com/daffa2555/Tokenectomy/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/rustsec/advisory-db"><img src="https://img.shields.io/badge/Security-Audited%20(RustSec)-2ea44f?logo=rust" alt="Security" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License" /></a>
  <a href="https://modelcontextprotocol.io"><img src="https://img.shields.io/badge/MCP-Compatible-purple" alt="MCP" /></a>
  <a href="https://github.com/daffa2555/tokenectomy-bechmark-history"><img src="https://img.shields.io/badge/Benchmarks-Verifiable%20History-blue?logo=github" alt="Benchmarks" /></a>
  <a href="https://github.com/daffa2555/Tokenectomy"><img src="https://img.shields.io/github/stars/daffa2555/Tokenectomy?style=social" alt="GitHub Stars" /></a>
</p>

> 🌐 **[Explore the Interactive Web Playground & Live Architecture →](https://tokenectomy.vercel.app)**  
> 🛡️ **Need Autonomous Auto-Fix, Tree-sitter AST Healing, & Reverse Proxy? [Discover Tokenectomy Sentinel (Pro Edition) →](https://daffa2555.gumroad.com/l/tokenectomy-sentinel)**  
> 🔀 **Pair with [Tokenectomy Git (OSS)](https://github.com/daffa2555/tokenectomy-git)** for autonomous Git fix branches & PR creation!  
> 📊 **Full Historical Telemetry Receipts (25K ➔ 1M Lines):** Check out the transparent **[Tokenectomy Benchmark History](https://github.com/daffa2555/tokenectomy-bechmark-history)** repository.


<p align="center">
  <img src="demo.gif" alt="Tokenectomy OSS Demo" width="100%" />
</p>

**Tokenectomy** is an agent-native **Machine-to-Machine (M2M) MCP server** built with Rust. Designed specifically as a background sidecar for autonomous coding agents (Claude Desktop, Cursor, Cline, Roo Code, Windsurf, Google Antigravity), it acts as an autonomous sub-cortex: surgically scrubbing 90%+ of internal framework noise (`node_modules`, `site-packages`, `.cargo/registry`) from error logs, auto-redacting sensitive credentials before cloud transmission, and enforcing AST syntax safety—**with zero human babysitting**.

```
                     ┌──────────────────┐
   Agent Error Dump  │   TOKENECTOMY    │      Clean Agent Context
   (38K tokens) ───► │   🕵️‍♂️ OSS (M2M)  │ ───►  (2K tokens)  ───► LLM Brain
                     │                  │
   node_modules/     │  🔍 Smart Filter │      Only YOUR code
   site-packages/    │  🛡️ Redact       │      + error message
   .cargo/registry/  │  💾 Cache        │      + StackOverflow refs
                     └──────────────────┘
```

---

## ✨ Key Features

- **🔍 Smart Framework Filter** — Strips thousands of lines of noisy internal stack frames (`node_modules`, `site-packages`, `.cargo/registry`, `__pycache__`) and keeps only the code *you* wrote.
- **🌐 Stack Overflow Search** — Silently queries StackExchange APIs and injects top community solutions into the AI's context.
- **⚡ SHA-256 Response Cache** — Identical errors hit local cache (24h TTL). Recurring CI/CD failures cost $0.00 in API calls.
- **🛡️ Secret Redaction** — Regex engine strips API keys, AWS secrets, JWTs, and database connection strings before any data leaves your machine (ReDoS-safe, linear-time).
- **🔒 Anti-Hardcode Secret Shield** — Automatically detects and blocks AI patches that attempt to hardcode raw API keys, passwords, or credentials into your source code.
- **📐 AST Syntax Validation** — In-memory Tree-sitter AST parser ensures AI patches never write broken syntax to your repository.
- **🔒 Path Traversal Protection** — MCP file operations are canonicalized and locked to your current working directory.
- **🤖 MCP Server Mode** — Full JSON-RPC 2.0 over stdio. Works with Claude Desktop, Cursor, VS Code, Google Antigravity, and any MCP-compatible client.
- **🔌 Multi-Provider** — Supports OpenAI, Anthropic, and Ollama (100% offline mode).

---

## 📊 Verifiable Real-World Performance & Benchmark (OSS vs Pro)

No buzzwords or artificial benchmarks. Every developer can verify the core functions on their own machine:

> 🔍 **100% Transparent Benchmark History & Industry Standards (ISO/IEC 25010 & OWASP):**  
> We track our complete scaling timeline (from 25K to 1 Million lines) and hardware telemetry logs in the **[tokenectomy-bechmark-history](https://github.com/daffa2555/tokenectomy-bechmark-history)** repository.

### 🆓 Tokenectomy OSS (Community Edition) — Verifiable Heavy Stress Benchmark


You don't need to buy anything to test this. Clone this repository right now and verify these heavy load benchmarks directly on your hardware:

| Feature Under Test | Tested Heavy Input | Real Measured Outcome | Status |
|---|---|---|:---:|
| **Quarter-Million Log Redaction** | **250,000 lines (24.44 MB)** enterprise dump with DB URLs, API keys, JWTs | **333.49 ms (73.3 MB/sec, 749,652 lines/sec)**. 100% sanitized. | ✅ Verified |
| **ReDoS Immunity** | 50,000-character malicious backtracking exploit string | **1.44 ms**. Linear $O(N)$ evaluation, 100% ReDoS immune. | ✅ Verified |
| **High Concurrency Torture** | 100 concurrent OS threads hammering redaction & extractor | **100/100 in 27.35 ms (7,312.7 ops/sec)**. Zero race conditions. | ✅ Verified |
| **Kernel Memory Footprint** | Peak Resident Memory during 250,000-line stress test | **76.24 MB VmRSS** via Linux `/proc/self/status`. Zero memory ballooning. | ✅ Verified |

> 💡 **"Skeptical about these numbers? Don't take our word for it."**  
> We hate marketing fluff and sweet talk as much as you do. You don't need to take our word for it or pay a single cent. Clone this repository, run the benchmark on your own machine, watch your CPU blaze through **250,000 lines** of logs in a third of a second, and verify the exact telemetry in your own terminal:
>
> ```bash
> cargo test --release --test stress_benchmark -- --nocapture
> ```

### 👑 Tokenectomy Pro — Industrial-Grade Heavy Production Torture Benchmark

To prove stability under enterprise workloads, we hammered **Tokenectomy Pro** with sustained multi-million token traffic, a 10MB Kubernetes crash avalanche, and 250 parallel OS threads on a 12-Core Intel i5-1235U with Arch Linux:

| Stress Vector | Tested Workload | Measured Kernel / CPU Telemetry | Status |
|---|---|---|:---:|
| **One Million Line Surgery** | **1,000,002 lines (82.99 MB / 19.5M tokens)** massive cluster dump | **19,500,032 raw tokens processed & scrubbed** (36,549 lines/sec). Zero crash, zero buffer overflow. | ✅ Passed |
| **Sustained Stream** | 100 consecutive microservice crash incidents (2.16 MB) | **604,490 ➔ 6,490 tokens (98.93% reduction)** at **41,752 tokens/sec**. | ✅ Passed |
| **K8s Crash Avalanche** | 6,506-line dump combining Spring Boot, PyTorch OOM, & Go | **Processed in 514 ms (1.6 MB/sec)**. User code preserved across 3 languages. | ✅ Passed |
| **Monorepo Parallel AST** | 100 multi-language files (Rust, TS, Python, Go) parsed simultaneously | **2.68 ms total (0.027 ms / file)** = **37,379 files/sec**. Zero memory leak. | ✅ Passed |
| **Extreme Concurrency** | 250 parallel OS threads hammering AST, Redact, & HaluGuard | **250/250 passed in 865 ms** (**867 ops/sec**). Zero deadlock or race condition. | ✅ Passed |
| **Memory Footprint (VmRSS)** | Full 600K-token & 250-thread torture test | **Peak RAM capped at 83.77 MB** via Linux `/proc/self/status`. Zero leak. | ✅ Controlled |


> 💡 **"Skeptical about these enterprise numbers? Don't take our word for it."**  
> Every purchaser of Tokenectomy Pro receives the full, standalone benchmark and torture test suite (`tests/heavy_production_torture.rs`) bundled in the package. You can run `cargo test --release --test heavy_production_torture -- --nocapture` on your own infrastructure to verify every metric before deployment.

👉 **[Get Tokenectomy Sentinel on Gumroad ($9) →](https://daffa2555.gumroad.com/l/tokenectomy-sentinel)** (Native binaries for Linux, macOS Apple Silicon, and Windows).

---

## 📦 Installation

### ⚡ Install via Cargo (crates.io)

```bash
cargo install tokenectomy
```

### 🦀 Build from Source

```bash
git clone https://github.com/daffa2555/Tokenectomy.git
cd Tokenectomy
cargo build --release
sudo cp target/release/tokenectomy /usr/local/bin/tkmy
```

### ⚙️ Install via Smithery (for Claude Desktop)

```bash
npx -y @smithery/cli install tokenectomy --client claude
```

---

## 🔌 M2M Agent Setup (1-Minute Integration)

Tokenectomy is architected to run silently between your AI Coding Agent and your repository over **JSON-RPC 2.0 stdio**. You configure it once, and your agent autonomously invokes Tokenectomy in the background during debugging and refactoring loops—**no manual copy-pasting or piping required**.

```bash
tkmy --mcp
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "tkmy",
      "args": ["--mcp"]
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
      "command": "tkmy",
      "args": ["--mcp"]
    }
  }
}
```

### Cline / Roo Code / Windsurf / VS Code

Add to your MCP settings (`settings.json` or `cline_mcp_settings.json`):

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "tkmy",
      "args": ["--mcp"]
    }
  }
}
```

### Google Antigravity CLI

```bash
agy mcp add tokenectomy -- tkmy --mcp
```

### 🤖 Available M2M MCP Tools

| Tool | Autonomous Agent Role |
|------|-------------|
| `get_error_context` | Performs deep log surgery: strips framework noise, redacts secrets, extracts source context and git diff |
| `search_stack_overflow` | Searches Stack Overflow for a specific error (query is auto-sanitized of secrets) |
| `apply_code_patch` | Applies a code patch to a file by search-and-replace |
 
### 🔀 Companion MCP Server: Tokenectomy Git

Close the autonomous loop from error diagnosis all the way to a published GitHub Pull Request! Pair Tokenectomy with our official companion MCP server: **[Tokenectomy Git](https://github.com/daffa2555/tokenectomy-git)**.

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "tkmy",
      "args": ["--mcp"]
    },
    "tokenectomy-git": {
      "command": "tkmy-git",
      "args": ["--mcp"]
    }
  }
}
```

Together, they enable your AI coding assistant to:
1. Scrub noisy error logs & redact credentials (`tokenectomy`)
2. Generate an accurate fix patch
3. Create a branch, commit files, and open a GitHub PR autonomously (`tokenectomy-git`)

---

## 🛠️ Standalone / Local CLI Mode (Optional)

While Tokenectomy is architected for autonomous machine-to-machine agent operation, it also provides a standalone CLI binary if you want to pipe logs in CI/CD pipelines, local shell scripts, or manual debugging:

### Pipe errors directly

```bash
python3 app.py 2>&1 | tkmy
cargo build 2>&1 | tkmy
node server.js 2>&1 | tkmy
```

### Read from a log file

```bash
tkmy --file /var/log/app/error.log
```

### Advanced options

```bash
tkmy --local-only            # 100% offline via Ollama ($0 cost)
tkmy --provider openai       # Use OpenAI GPT-4o
tkmy --provider anthropic    # Use Claude 3.5 Sonnet
tkmy --context-lines 20      # Extract 20 lines of surrounding context
tkmy --yes                   # Skip interactive prompts (CI/CD mode)
```

---

## 🧠 Configuration

Create `~/.tokenectomy.toml`:

```toml
default_provider = "openai"  # openai | anthropic | ollama | mock
openai_api_key = "sk-..."
anthropic_api_key = "sk-ant-..."
ollama_base_url = "http://localhost:11434"
context_lines = 10
max_context_chars = 10000
```

---

## 🔒 Security

- **No secrets leave your machine.** Regex engine redacts API keys, JWTs, AWS credentials, and database URLs before any data is sent to an LLM.
- **No path traversal.** MCP file operations are canonicalized and locked to CWD.
- **No stdin bombs.** Input is capped at 10MB (CLI) / 50MB (MCP) via `.take()`.
- **No weak hashing.** Cache uses `sha2::Sha256`, never `DefaultHasher`.

---

## 🏗️ Architecture

```
src/
├── main.rs          # Entry point, CLI args, REPL, pipeline orchestration
├── mcp.rs           # JSON-RPC 2.0 over stdio MCP server
├── extractor/       # Language-specific context extraction (Rust, Python, JS)
├── provider/        # AI backends (OpenAI, Anthropic, Ollama, Mock)
├── redact.rs        # Secret redaction (linear-time regex, ReDoS-safe)
├── cache.rs         # SHA-256 response cache (24h TTL, 0700 perms)
├── search.rs        # Stack Overflow API integration
└── git.rs           # Recent git diff extraction
```

---

## 🧠 Bundled Open Source Agent Skills

Tokenectomy OSS includes 2 native **Antigravity & Coding Agent Skills** in `.agents/skills/` to elevate your AI assistant's engineering discipline:

| Skill | Description | Location |
|---|---|---|
| **`adversary-bug-hunter`** | Red-team fuzzer that stress-tests edge cases, catches unhandled unwraps/ReDoS, and hardens code. | [`.agents/skills/adversary-bug-hunter/`](.agents/skills/adversary-bug-hunter/SKILL.md) |
| **`spec-first-architect`** | Enforces strict Test-Driven Development (TDD) & state machine invariants to eliminate AI hallucinations. | [`.agents/skills/spec-first-architect/`](.agents/skills/spec-first-architect/SKILL.md) |

---

## 🆓 vs 👑 — OSS vs Pro

| Feature | OSS (Free) | Pro ($9) |
|---|:---:|:---:|
| Smart Framework Filter | ✅ | ✅ |
| Stack Overflow Search | ✅ | ✅ |
| SHA-256 Response Cache | ✅ | ✅ |
| Secret Redaction (ReDoS-safe) | ✅ | ✅ |
| **Anti-Hardcode Secret Shield** | ✅ | ✅ |
| **AST Syntax Validation** (Tree-sitter) | ✅ | ✅ |
| MCP Server Mode | ✅ | ✅ |
| Multi-Provider (OpenAI, Anthropic, Ollama) | ✅ | ✅ |
| **Bundled Agent Skills** | 2 Skills | **4 Full Skills Suite** |
| **Auto-Fixer** (AI patch → auto-apply) | ❌ | ✅ |
| **AST Smart Healer** (auto-repair syntax errors) | ❌ | ✅ |
| **Code Integrity Guard** (Anti-Halu & Anti-Ngide) | ❌ | ✅ |
| **Test Verification Loop** (auto-rollback on test fail) | ❌ | ✅ |
| **Multi-File Atomic Transactions** | ❌ | ✅ |
| **Time Machine Undo Engine** (1-sec revert via `tkmy --undo`) | ❌ | ✅ |
| **True Ectomy Engine** (99% token reduction) | ❌ | ✅ |
| **DB Inspector** (real TCP port probe) | ❌ | ✅ |
| **Docker Diagnostics** (OOMKilled detection) | ❌ | ✅ |
| **`--benchmark` Mode** | ❌ | ✅ |
| **SRE Incident Commander Skill** (OOM 137 triage & 5-Whys RCA) | ❌ | **✅ Included** |
| **Refactor Sentinel Skill** (Zero-regression blast radius refactor) | ❌ | **✅ Included** |

👉 **[Get Tokenectomy Sentinel →](https://daffa2555.gumroad.com/l/tokenectomy-sentinel)**

---

## 📜 License

MIT — see [LICENSE](LICENSE) for details.

---

*Built with 🦀 Rust for maximum performance, strict memory safety, and uncompromising security.*
