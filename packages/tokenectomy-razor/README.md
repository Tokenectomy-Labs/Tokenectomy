# tokenectomy-razor 🗡️

<p align="left">
  <a href="https://mcpservers.org/servers/tokenectomy-labs/tokenectomy"><img src="https://mcpservers.org/badge.svg" alt="Listed on mcpservers.org" /></a>
  <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen" alt="Official MCP Registry" /></a>
  <a href="https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy"><img src="https://img.shields.io/badge/Glama.ai-Tokenectomy--Razor-purple" alt="Tokenectomy Razor on Glama.ai" /></a>
  <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm" alt="npm version" /></a>
  <a href="https://crates.io/crates/tokenectomy"><img src="https://img.shields.io/crates/v/tokenectomy.svg?logo=rust" alt="crates.io version" /></a>
  <a href="https://github.com/Tokenectomy-Labs/Tokenectomy/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License" /></a>
</p>

Autonomous, machine-to-machine (M2M) Model Context Protocol (MCP) server & log surgery engine for AI coding agents (**Claude Desktop**, **Cursor**, **Cline**, **Roo Code**, **Windsurf**, and **Google Antigravity**).

- **Official MCP Registry Name:** `io.github.Tokenectomy-Labs/razor`
- **Glama TDQS Rating:** Grade A (100% Schema & Documentation Quality Score)

---

## 🚀 Instant Usage with npx

No Rust compiler, no manual build, and zero dependencies required! The package automatically provisions the verified native precompiled Rust binary for your operating system and architecture:

```bash
npx -y tokenectomy-razor --mcp
```

### Supported Native Architectures
- **Linux:** `x86_64` (glibc & musl), `aarch64` (ARM64)
- **macOS:** Apple Silicon (`arm64`), Intel (`x86_64`)
- **Windows:** `x86_64` (MSVC)

---

## 🛠️ Exposed MCP Tools

Tokenectomy Razor equips agents with 4 high-performance tools adhering strictly to MCP JSON-RPC 2.0 specifications:

| Tool Name | Capability Description |
|---|---|
| `get_error_context` | Performs deep polyglot trace surgery on error dumps, removes 90%+ framework noise (`node_modules`, `site-packages`, `.cargo/registry`, Spring Boot, Tomcat, Hibernate, ASan, Go idle goroutines), redacts credentials, and extracts local source context bounded to the workspace. |
| `analyze_code` | Performs static AST code analysis to detect resource leaks, unclosed handles, and security vulnerabilities with bounded execution limits and precise LSP UTF-16 coordinates. |
| `apply_code_patch` | Applies atomic file modifications with post-write language syntax verification (`cargo check`, `node --check`, `py_compile`) and automated zero-dirty-diff rollback on validation failure. |
| `search_stack_overflow` | Queries Stack Exchange API for relevant error signatures using sanitized search terms. |

---

## 🔌 MCP Client Configuration

### Claude Desktop
Add to your `claude_desktop_config.json`:

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

### Cursor (`.cursor/mcp.json`)
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

### Google Antigravity CLI
Install directly into your Antigravity agent configuration:

```bash
agy mcp add tokenectomy -- npx -y tokenectomy-razor --mcp
```

### Cline / Roo Code / Windsurf / VS Code
Add to your MCP settings (`cline_mcp_settings.json` or `settings.json`):

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

---

## ⚡ Verifiable Performance Benchmarks

| Metric | Measured Value | Standard / Guarantee |
|---|---|---|
| **Redaction Throughput** | **73.3 MB/sec** (749,652 lines/sec) | 250,000-line enterprise dump processed in 333 ms |
| **ReDoS Immunity** | **1.44 ms** | Linear $O(N)$ evaluation on 50K pathological backtracking patterns |
| **Concurrent Throughput** | **7,312 ops/sec** | 100 concurrent OS threads across simultaneous sanitization calls |
| **Memory Footprint** | **76.24 MB VmRSS** | Peak resident memory under continuous high-load stress |

---

## 📦 Global Install (Optional)

You can also install the binary globally for standalone terminal CLI piping:

```bash
npm install -g tokenectomy-razor

# Run standalone CLI
razor --help
tokenectomy-razor --help

# Scrub framework frames and redact credentials via pipe
npm test 2>&1 | razor --scrub > sanitized.log
```

---

## 🗺️ Roadmap & Ecosystem Milestones

- ✅ **v1.0.0 - v1.1.0:** Core polyglot log surgery, O(N) ReDoS-immune redaction, AI reverse proxy gateway (`--proxy`).
- ✅ **v1.1.3:** Multi-architecture Docker images (GHCR) & GitHub Actions Marketplace Action.
- ✅ **v1.1.5:** Static AST code analysis engine (`analyze_code`) & Glama.ai TDQS Grade A compliance.
- ✅ **v1.1.6:** Precompiled standalone native binaries (Linux, macOS Apple Silicon/Intel, Windows).
- ✅ **v1.1.7:** Canonical Anthropic MCP Registry validation (`io.github.Tokenectomy-Labs/razor`) & `mcpservers.org` directory listing.
- 🔄 **v1.2.0 (In Progress):** Java/Kotlin (Spring Boot 3) & C/C++ backtrace surgery, `awesome-mcp-servers` directory PR.
- 📋 **v1.3.0 (Planned):** User-defined custom redaction rules (`~/.tokenectomy.toml`) & local agent token savings metrics.

---

## 🔗 Official Links & Resources

- **GitHub Repository:** [https://github.com/Tokenectomy-Labs/Tokenectomy](https://github.com/Tokenectomy-Labs/Tokenectomy)
- **Official MCP Registry:** [registry.modelcontextprotocol.io](https://registry.modelcontextprotocol.io) (`io.github.Tokenectomy-Labs/razor`)
- **mcpservers.org Listing:** [https://mcpservers.org/servers/tokenectomy-labs/tokenectomy](https://mcpservers.org/servers/tokenectomy-labs/tokenectomy)
- **Glama.ai Listing:** [https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy](https://glama.ai/mcp/servers/Tokenectomy-Labs/Tokenectomy)
- **Crates.io Crate:** [https://crates.io/crates/tokenectomy](https://crates.io/crates/tokenectomy)

