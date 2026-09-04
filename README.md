
# Tokenectomy 🕵️‍♂️ — Smart Log Surgery & Context Reducer for LLMs (MCP Server & CLI)

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLXdpZHRoPSIyIi8+PC9zdmc+)](https://modelcontextprotocol.io)
[![Glama](https://img.shields.io/badge/Glama-MCP%20Server-blueviolet)](https://glama.ai/mcp/servers)
[![GitHub Stars](https://img.shields.io/github/stars/daffa2555/Tokenectomy?style=social)](https://github.com/daffa2555/Tokenectomy)

> 👑 **Looking for Auto-Fixer, AST Smart Healer, Anti-Halu, and Time Machine Undo?**
> **[Get Tokenectomy Pro (Enterprise Edition) →](https://tokenectomy.gumroad.com/l/kiznsu)**
>
> 🔀 **Pair with [Tokenectomy Git (OSS)](https://github.com/daffa2555/tokenectomy-git)** for autonomous Git branch, commit, and Pull Request creation!

<p align="center">
  <img src="demo.gif" alt="Tokenectomy OSS Demo" width="100%" />
</p>

**Tokenectomy** is a high-performance Rust CLI and MCP server that scrubs 90%+ of framework noise from error logs before they reach your AI assistant's context window. It strips `node_modules`, `site-packages`, and vendor stack frames, redacts secrets, injects StackOverflow solutions, and caches responses locally — so your AI spends tokens on *your* code, not framework internals.

```
                     ┌──────────────────┐
  Raw Error Log      │   TOKENECTOMY    │      Clean Context
  (38K tokens)  ───► │   🕵️‍♂️ OSS       │ ───►  (2K tokens)  ───► LLM
                     │                  │
  node_modules/      │  🔍 Smart Filter │      Only YOUR code
  site-packages/     │  🛡️ Redact       │      + error message
  .cargo/registry/   │  💾 Cache        │      + StackOverflow refs
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

### 🆓 Tokenectomy OSS (Community Edition)
| Feature Under Test | Tested Input | Real Measured Outcome | Status |
|---|---|---|:---:|
| **Secret Redaction** | PostgreSQL URL, OpenAI `sk-proj`, AWS keys, JWT | 100% scrubbed locally before cloud transmission | ✅ Verified |
| **Stack Trace Extractor** | Rust compiler error & panic traces | File & line isolated; framework noise stripped | ✅ Verified |
| **Sub-ms Test Suite** | Local regex & parser pipeline | **0.13s execution time** for all 9 unit tests | ✅ Verified |

*Run verification on OSS:*
```bash
cargo test
```

### 👑 Need Deep Log Surgery & Auto-Rollback? (Tokenectomy Pro)
| Enterprise Feature | Real-World Benchmark Outcome | Edition |
|---|---|:---:|
| **True Ectomy Log Surgery** | **98.60% Token Reduction** (5,355 ➔ 75 tokens on Express/Prisma) | **Pro Exclusive** 👑 |
| **AST Smart Healer** | **1.23 ms** offline syntax repair via Tree-sitter | **Pro Exclusive** 👑 |
| **HaluGuard Protection** | Blocks `// ... existing code ...` lazy code deletion in **42 ms** | **Pro Exclusive** 👑 |
| **Test-Fail Auto-Rollback** | **100% clean rollback** (**0 bytes dirty diff**) when tests fail | **Pro Exclusive** 👑 |
| **Atomic Multi-File Tx** | Aborts multi-file patch atomically if any file has syntax error | **Pro Exclusive** 👑 |

👉 **[Get Tokenectomy Pro on Gumroad ($9) →](https://tokenectomy.gumroad.com/l/kiznsu)**

---

## 📦 Installation

### 🦀 Build from Source (Recommended)

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

## 🚀 Usage

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

## 🔌 MCP Server Integration

Register Tokenectomy with your AI editor to grant it autonomous debugging capabilities.

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

### Google Antigravity CLI

```bash
agy mcp add tokenectomy -- tkmy --mcp
```

### VS Code / Windsurf

Add to your `settings.json`:

```json
{
  "mcp.servers": {
    "tokenectomy": {
      "command": "tkmy",
      "args": ["--mcp"]
    }
  }
}
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `get_error_context` | Performs log surgery: strips framework noise, redacts secrets, extracts source context and git diff |
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

👉 **[Get Tokenectomy Pro →](https://tokenectomy.gumroad.com/l/kiznsu)**

---

## 📜 License

MIT — see [LICENSE](LICENSE) for details.

---

*Built with 🦀 Rust for maximum performance, strict memory safety, and uncompromising security.*
