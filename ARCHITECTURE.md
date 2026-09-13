# Tokenectomy Razor: System Architecture & Technical Specifications

> **The Autonomous Machine-to-Machine (M2M) Sub-Cortex for AI Coding Agents**

---

## 1. Architectural Philosophy & Core Invariants

Tokenectomy Razor is designed primarily as an **Autonomous Machine-to-Machine (M2M) Sub-Cortex** operating via the official **Model Context Protocol (MCP)** over standard JSON-RPC 2.0 stdio. It intercepts terminal crash logs and code modifications between execution environments and Large Language Model (LLM) context windows.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             AGENT ENVIRONMENT                               │
│              (Cursor Composer, Claude Desktop, Windsurf, Cline)             │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ JSON-RPC 2.0 stdio / HTTP
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             TOKENECTOMY RAZOR                               │
│                                                                             │
│   ┌───────────────────────────┐         ┌───────────────────────────────┐   │
│   │ Tier 1: Protocol & Ingest │         │ Tier 2: Polyglot Trace Engine │   │
│   │ - MCP Server (JSON-RPC)   │ ──────► │ - Polyglot Stack Slicers      │   │
│   │ - AI Gateway Reverse Proxy│         │ - Framework Noise Stripper    │   │
│   │ - FinOps Metrics Exporter │         │ - Local Source Context Extr.  │   │
│   └───────────────────────────┘         └──────────────┬────────────────┘   │
│                                                        │                    │
│                                                        ▼                    │
│   ┌───────────────────────────┐         ┌───────────────────────────────┐   │
│   │ Tier 4: AST & Integrity   │         │ Tier 3: Security & Redaction  │   │
│   │ - Static AST Code Analysis│ ◄────── │ - Linear O(N) ReDoS Safe DFA  │   │
│   │ - Atomic Patch Validation │         │ - Zero-Knowledge Redaction    │   │
│   │ - Auto Git Rollback Loop  │         │ - Workspace Boundary Guard    │   │
│   └───────────────────────────┘         └───────────────────────────────┘   │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Sanitized Context (<0.2ms)
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          UPSTREAM LLM CONTEXT WINDOW                        │
│                   (Claude 3.5 Sonnet, GPT-4o, Local Ollama)                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Core Invariants

1. **Machine-to-Machine (M2M) Invariant**: Autonomous AI agents invoke Tokenectomy in background loops without human intervention. The CLI is auxiliary, reserved for shell piping and reproducible audit benchmarks.
2. **Zero Dirty Git Diff**: Any agent code patch applied via `apply_code_patch` must pass post-write language syntax verification (`cargo check`, `node --check`, `py_compile`). On validation failure, state is automatically rolled back to 0 dirty git diff.
3. **Linear $O(N)$ ReDoS Immunity**: Secret redaction and log scanning run through deterministic finite automata (DFA). Pathological backtracking inputs are repelled in constant linear time ($O(N)$), preventing denial-of-service stalls.
4. **Sub-Millisecond Latency**: Written in safe, zero-allocation Rust. Stack parsing and noise filtering execute in $<0.2\text{ ms}$, ensuring zero agent latency overhead.
5. **Zero-Knowledge Air-Gap**: All log surgery, AST inspection, and credential masking execute locally on physical hardware. No log data or telemetry is transmitted to third-party endpoints.

---

## 2. Multi-Tier Architecture

### Tier 1: Protocol & Ingestion Layer

- **Model Context Protocol (`src/mcp.rs`)**:
  - Implements the official Model Context Protocol (JSON-RPC 2.0 stdio).
  - Complies with **Glama Grade A Tool Definition Quality Score (TDQS)** with strict JSON schema definitions, boundary validation, and zero ambient hallucination.
  - Exposes tools: `get_error_context`, `analyze_code`, `apply_code_patch`, and `search_stack_overflow`.

- **AI Gateway Reverse Proxy (`src/proxy.rs`)**:
  - High-throughput asynchronous HTTP reverse proxy running on `127.0.0.1:8080`.
  - Transparently intercepts streaming prompt requests before forwarding upstream to OpenAI, Anthropic, or local Ollama.
  - Enforces resource boundaries: `MAX_HEADER_SIZE` (64 KB), `MAX_BODY_SIZE` (10 MB), request timeouts (30s / 60s), and a 128-connection concurrency limit.

- **FinOps Economics Engine (`src/dashboard.rs`)**:
  - Embedded zero-dependency HTTP dashboard and Prometheus metrics exporter (`/v1/metrics`, `/dashboard`).
  - Tracks live token reductions, blended LLM dollar savings, active credential redactions, and request throughput.

---

### Tier 2: Polyglot Trace Excision Engine (`src/extractor/`)

The context engine utilizes modular trait implementations per language runtime rather than a fragile single universal regular expression:

```rust
pub trait TraceParser: Send + Sync {
    fn detect(&self, log: &str) -> bool;
    fn extract_locations(&self, log: &str) -> Vec<CodeLocation>;
}
```

#### Supported Language Extractors

| Runtime Module | Primary Target Frameworks | Filtered Framework Noise |
|---|---|---|
| `rust.rs` | Tokio, Actix, Axum, Stdlib panics | `.cargo/registry`, `.rustup`, `target/debug/build` |
| `js.rs` | Next.js, Vite, Express, NestJS, Webpack | `node_modules`, `.next`, `dist`, runtime bundles |
| `python.rs` | Django, FastAPI, Flask, PyTorch | `site-packages`, `dist-packages`, `venv`, `__pycache__` |
| `go.rs` | Gin, Fiber, Stdlib Goroutine panics | `go/src` (stdlib), `go/pkg/mod`, `vendor` |
| `java.rs` | Spring Boot 3, Tomcat, Netty, Hibernate | `.m2/repository`, `.gradle/caches`, internal bytecode |
| `cpp.rs` | GDB / LLDB backtraces, AddressSanitizer | `/usr/include`, `/usr/lib`, `vcpkg_installed` |
| `csharp.rs` | .NET Core, ASP.NET Runtime | `bin/Debug`, `obj/`, NuGet packages |
| `php.rs` | Laravel, Symfony, Composer | `vendor/composer`, `vendor/symfony` |
| `ruby.rs` | Ruby on Rails, Rack | `vendor/bundle`, gem paths |

#### Context Bounding
For each identified source location, the engine reads a configurable source window (`[line - N, line + N]`, default 10 lines) directly from disk, stripping hundreds of framework frames while delivering exact application context to the LLM.

---

### Tier 3: Security & Deterministic Redaction Engine (`src/redact.rs`, `src/workspace.rs`)

- **Deterministic Secret Redaction**:
  - Employs DFA-based linear-time pattern matching.
  - Automatically identifies and masks high-entropy credentials:
    - Anthropic API keys (`sk-ant-api03-...`)
    - OpenAI API keys (`sk-...`)
    - GitHub Personal Access Tokens (`ghp_...`)
    - AWS Access Keys & Secret Access Keys (`AKIA...`)
    - JSON Web Tokens (`eyJ...`)
    - Database Connection Strings (`postgresql://`, `mysql://`, `mongodb://`, `redis://`)
    - Private SSH / TLS Keys (`-----BEGIN RSA PRIVATE KEY-----`)
    - Slack / Discord Webhooks

- **Workspace Boundary Containment (`src/workspace.rs`)**:
  - All file reads and writes are canonicalized and verified against the workspace root (`CWD`).
  - Strict containment rejects directory traversal payloads (`../`), absolute paths outside workspace boundaries, and circular symlink breakouts.

- **SHA-256 Idempotency Cache (`src/cache.rs`)**:
  - Deterministic in-memory response cache with a 24-hour TTL.
  - Hashes sanitized log signatures to prevent repeated agent invocation costs on identical terminal failures.

---

### Tier 4: AST Analysis & Atomic Patching (`src/analyzer/`, `src/git.rs`)

- **Static AST Code Analysis (`src/analyzer/`)**:
  - Detects syntax anomalies, resource leaks, unclosed handles, and unbounded loops.
  - Enforces bounded processing limits: maximum 50,000 AST nodes and 10 MB file cap.
  - Emits diagnostic findings with precise LSP-compliant UTF-16 line and character offsets.

- **Atomic Code Patching with Auto-Rollback (`src/git.rs`)**:
  - Applies patches atomically.
  - Triggers native language syntax gates (`cargo check`, `node --check`, `py_compile`).
  - If the syntax check fails, `git.rs` triggers an immediate rollback to the pre-patch commit state, guaranteeing **zero dirty git diffs** in automated agent loops.

---

## 3. Directory Layout

```
Tokenectomy/
├── Cargo.toml                  # Workspace dependencies & compiler optimizations
├── ARCHITECTURE.md             # System architecture & specification
├── README.md                   # Public documentation & Tokio-grade hero
├── SECURITY.md                 # Security policy & vulnerability reporting
├── media/                      # Official brand assets (logo.png, logo.jpg)
├── src/
│   ├── main.rs                 # Binary CLI entry point
│   ├── lib.rs                  # Core library exports
│   ├── cli.rs                  # Clap command-line interface arguments
│   ├── mcp.rs                  # JSON-RPC 2.0 stdio MCP server implementation
│   ├── proxy.rs                # AI Gateway HTTP reverse proxy (127.0.0.1:8080)
│   ├── dashboard.rs            # FinOps telemetry & metrics dashboard UI
│   ├── redact.rs               # O(N) ReDoS-immune regex secret redactor
│   ├── workspace.rs            # Workspace boundary guard & traversal isolator
│   ├── cache.rs                # SHA-256 idempotency cache (24h TTL)
│   ├── search.rs               # Sanitized Stack Exchange error search
│   ├── git.rs                  # Git rollback & dirty diff verification
│   ├── analyzer/               # Static AST code analysis engine
│   │   └── mod.rs              # AST node walker & LSP diagnostic generator
│   ├── extractor/              # Polyglot stack trace surgery
│   │   ├── mod.rs              # Extractor dispatcher & source window loader
│   │   ├── rust.rs             # Rust panic & backtrace parser
│   │   ├── js.rs               # Node.js, V8, TypeScript trace parser
│   │   ├── python.rs           # Python traceback parser
│   │   ├── go.rs               # Go runtime panic parser
│   │   ├── java.rs             # JVM & Spring Boot stack parser
│   │   ├── cpp.rs              # AddressSanitizer & GDB parser
│   │   ├── csharp.rs           # .NET Core trace parser
│   │   ├── php.rs              # PHP & Laravel parser
│   │   └── ruby.rs             # Ruby on Rails parser
│   └── provider/               # Direct model providers (CLI standalone mode)
│       ├── mod.rs              # AiProvider trait definition
│       ├── anthropic.rs        # Anthropic Messages API client
│       ├── openai.rs           # OpenAI Chat Completions client
│       ├── ollama.rs           # Local Ollama HTTP client
│       └── mock.rs             # Test fixture mock provider
└── tests/
    └── stress_benchmark.rs     # Standalone hardware stress test suite
```

---

## 4. Benchmark & Hardware Truth

Tokenectomy adheres to **Hardware-Grounded Truth**. Performance measurements are verified on bare-metal physical hardware:

- **Redaction Throughput**: 531,002 lines/sec (470.8 ms for 250,000 lines).
- **ReDoS Resistance**: 1.09 ms on 50,000-character malicious backtracking payloads.
- **Thread Concurrency**: 7,312 ops/sec across 100 simultaneous OS threads.
- **Memory Footprint**: 76.24 MB VmRSS peak during sustained 250k-line ingestion.

Reproducible audit command:
```bash
cargo test --release --test stress_benchmark -- --nocapture
```

---

## 5. Security & Isolation Model

1. **Air-Gapped Local Execution**: All token excision and regex masking occur on the host machine. No external cloud round-trips occur without explicit agent direction.
2. **Deterministic Regex Engine**: Backtracking regular expressions are banned from the codebase. All redaction rules compile to linear state machines.
3. **Workspace Path Sandboxing**: Paths referenced in error traces must resolve strictly within `current_dir()`. Symlinks resolving outside the root directory trigger an immediate access denial.
4. **Bounded Stream Reading**: File ingestion applies `.take(MAX_BODY_SIZE)` to prevent memory exhaustion from runaway log files.

---

<div align="center">
  <p><b>Tokenectomy Labs</b> &bull; Autonomous M2M Sub-Cortex</p>
  <p>Maintained by <b><a href="https://github.com/daffa2555">Daffa (@daffa2555)</a></b></p>
  <p>Licensed under the <a href="LICENSE">MIT License</a></p>
</div>
