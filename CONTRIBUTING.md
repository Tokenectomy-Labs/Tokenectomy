# 🤝 Contributing to Tokenectomy Razor

Thank you for your interest in contributing! Tokenectomy Razor is built by developers, for developers, and designed specifically as an agent-native background infrastructure for AI coding agents. We welcome contributions of all kinds.

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Ways to Contribute](#ways-to-contribute)
- [Development Workflow](#development-workflow)
- [Testing & Quality](#testing--quality)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

---

## 📖 Code of Conduct

We are committed to providing a welcoming, respectful, and inclusive community for all contributors and maintainers.

### Core Principles
- 🤗 **Respectful & Inclusive**: Welcome developers of all experience levels.
- 🎯 **Community-Driven**: Prioritize maintainability, deterministic reliability, and agent-native ergonomics.
- 💬 **Constructive Collaboration**: Offer actionable, kind, and specific feedback.
- ⚡ **Attribution**: Acknowledge others' ideas and contributions.

---

## 🚀 Getting Started

### Prerequisites

- **Rust** (stable): [rustup.rs](https://rustup.rs/) (1.80+)
- **Cargo**: Bundled with Rust
- **Git**: For version control

### Local Setup

```bash
# Clone the repository
git clone https://github.com/Tokenectomy-Labs/Tokenectomy.git
cd Tokenectomy

# Build in debug mode
cargo build

# Run full test suite
cargo test

# Run benchmarks
cargo bench

# Verify linting & formatting
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

---

## 💡 Ways to Contribute

### 🐛 Report Bugs
Found a bug, unexpected panic, or regex mismatch? [Open an issue](https://github.com/Tokenectomy-Labs/Tokenectomy/issues) with:
- Clear, descriptive title.
- Minimal reproducible example or raw log snippet.
- Expected vs. actual sanitized output.
- Rust version & OS details (`rustc --version`, `uname -a`).
- Label: `bug`

### ✨ Suggest Features & Parsers
Have an idea for improved trace extraction, a new language parser, or agent integrations? [Open an issue](https://github.com/Tokenectomy-Labs/Tokenectomy/issues):
- Clear description of the use case.
- Sample stack traces and expected pruned output.
- Label: `enhancement`

### 📚 Improve Documentation
- Fix typos or clarify confusing sections in `README.md`.
- Add integration examples for specific AI agents (Cursor, Claude Desktop, Antigravity, Cline).
- Translate docs into other languages.
- Label: `documentation`

### 🌐 Add Language & Framework Support
Tokenectomy Razor supports polyglot stack trace parsing (Node.js/TypeScript, Python, Rust, Go, Java, Ruby, PHP). Want to add or improve framework filtering?
1. Add trace parser patterns in `src/extractor/` (or update language filters in `src/engine/`).
2. Add comprehensive unit tests with real-world trace fixtures in `tests/`.
3. Verify $O(N)$ linear time regex evaluation (zero catastrophic backtracking).
4. Label: `language-support`

---

## 🔧 Development Workflow

### 1. Fork & Branch
```bash
git checkout -b feat/your-feature-name
# or for bug fixes:
git checkout -b fix/issue-description
```

Branch naming convention:
- `feat/short-description` — new features or parsers
- `fix/short-description` — bug fixes
- `docs/short-description` — documentation updates
- `perf/short-description` — performance improvements

### 2. Make Changes
- Follow idiomatic Rust practices.
- Zero unnecessary heap allocations in inner loops.
- Format all code with `cargo fmt`.
- Write clear, atomic commit messages following [Conventional Commits](https://www.conventionalcommits.org/).

### 3. Keep Synced with Main
```bash
git fetch origin
git rebase origin/main
```

---

## 🧪 Testing & Quality

Every contribution must maintain hardware-grounded truth and 100% test pass rate:

```bash
# Run unit & integration tests
cargo test

# Run release test suite (performance & stress tests)
cargo test --release

# Format code
cargo fmt --check

# Clippy linter
cargo clippy --all-targets -- -D warnings

# Security audit on dependencies
cargo audit
```

---

## 📝 Submitting Changes

### Pre-Submission Checklist
- [ ] Code passes all tests: `cargo test --release`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] No credentials or secrets committed in git history
- [ ] Tests added for new logic or fixed bug
- [ ] Documentation updated if public interfaces changed

### Opening a Pull Request
1. Push your branch to your fork.
2. Open a Pull Request against `Tokenectomy-Labs/Tokenectomy:main`.
3. Provide a clear summary of changes and reference any related issues (`Closes #123`).

---

## 🎯 Project Roadmap

- **Phase 1: Foundation (Completed — v1.0 - v1.1)**
  - ✅ Polyglot trace parsing & framework frame stripping (Rust, Python, TS/JS, Go)
  - ✅ Deterministic regex-based secret redaction with $O(N)$ ReDoS immunity
  - ✅ AI Gateway Reverse Proxy (`--proxy`) & SHA-256 idempotency cache
  - ✅ JSON-RPC 2.0 stdio MCP server implementation
  - ✅ Crates.io crate (`tokenectomy`) & npm package (`tokenectomy-razor`)
  - ✅ GitHub Marketplace Action & GHCR Multi-Arch Docker containers
- **Phase 2: Static Analysis & Canonical Ecosystem Recognition (Completed — v1.1.5 - v1.1.7)**
  - ✅ Static AST code analysis engine (`analyze_code`) & LSP UTF-16 coordinates
  - ✅ Glama.ai Tool Definition Quality Score (TDQS Grade A) & Verified Maintainer
  - ✅ Precompiled multi-architecture release binaries (Linux x86_64/arm64/musl, macOS, Windows)
  - ✅ Official Anthropic MCP Registry publication (`io.github.Tokenectomy-Labs/razor`)
  - ✅ `mcpservers.org` directory listing and badge integration
- **Phase 3: Deep Polyglot Expansion & Integrations (Current — v1.2.0)**
  - 🔄 Java / Kotlin (Spring Boot 3, Gradle) framework stack trace extractors
  - 🔄 C / C++ AddressSanitizer (ASan) & GDB/LLDB backtrace sanitization
  - 🔄 Go goroutine dump compression & panic trace heuristics
  - 🔄 `awesome-mcp-servers` community catalog PR
- **Phase 4: Ecosystem Extensibility (Planned — v1.3.0+)**
  - 📋 User-defined custom redaction rules via `~/.tokenectomy.toml`
  - 📋 Local token savings & cost reduction metrics dashboard
  - 📋 Native IDE companion extensions (VS Code, JetBrains)
  - 📋 Server-Sent Events (SSE) remote MCP transport

---

## 🆘 Need Help?
- 🐛 Issues, questions & feature requests: [GitHub Issues](https://github.com/Tokenectomy-Labs/Tokenectomy/issues)
- 🔒 Security concerns: Please refer to [SECURITY.md](SECURITY.md)

Thank you for helping make Tokenectomy Razor the fastest, leanest context surgery sub-cortex for AI coding agents! 🗡️
