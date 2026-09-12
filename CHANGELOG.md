# Changelog

All notable changes to [Tokenectomy Razor](https://github.com/Tokenectomy-Labs/Tokenectomy) are documented here.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **Polyglot Multi-Trace Analysis**: Full continuous detection across all matching language parsers with cross-parser coordinate deduplication (supporting mixed stacks e.g. Node + Python, Rust + C FFI).
- **Extended Language & Framework Support**:
  - Added native **C# (.NET)** stack trace parser (`extractor::csharp::CSharpTraceParser`) extracting `.cs:line` coordinates.
  - Added native **Ruby on Rails** stack trace parser (`extractor::ruby::RubyTraceParser`) extracting `.rb:line` locations and filtering gem frames.
  - Python parser enhanced with `.pyi` / `.pyx` extensions and pytest assertion failure format detection.
  - JavaScript / TypeScript parser enhanced with webpack paths (`webpack:///`), URIs (`file:///`), and npm scoped packages (`@org/pkg`).
  - Added runtime noise filters for Python AsyncIO (`asyncio/base_events`), Starlette / FastAPI routing frames (`starlette/routing`), Uvicorn server frames (`uvicorn/protocols`), Gunicorn workers (`gunicorn/workers`), and .NET Core runtime frames (`System.Private.CoreLib`, `Microsoft.AspNetCore`).
- **User-Defined Custom Redaction & Noise Configuration**:
  - Dynamic `custom_redact_rules` and `custom_noise_patterns` support via project-level (`.tokenectomy.toml`) or user-level (`~/.tokenectomy.toml`) configuration without recompilation.
- **Gateway HTTP Path Normalization**:
  - Robust query parameter (`/health?format=json`) and trailing slash (`/dashboard/`) normalization for internal proxy endpoints, while preserving full query strings when forwarding requests to upstream LLMs.
- **YAML Syntax Validation & Dry-Run MCP Patching**:
  - Added pure-Rust native YAML syntax validation via `serde_yaml` and strict tab indentation rejection (`.yaml` / `.yml`) in `verify_patch` to prevent syntax corruption in CI/CD and container manifests without relying on external python runtimes.
  - Introduced `"dry_run": true` mode in MCP `apply_code_patch` tool allowing autonomous agents to simulate patch matching and syntax verification with zero disk mutation.
- **Expanded Secret Redaction**: High-precision, linear-time zero-backtracking redaction for HuggingFace tokens (`hf_...`), npm access tokens (`npm_...`), PyPI upload tokens (`pypi-AgEI...`), Stripe API keys (`sk_live_...`, `rk_live_...`), GitLab personal access tokens (`glpat-...`), and SendGrid API keys (`SG....`).
- **Universal Multi-Format Prompt Payload Sanitization**: Tokenectomy AI Gateway Proxy (`sanitize_prompt_payload`) now seamlessly handles Anthropic / OpenAI multi-part message content arrays (`[{"type": "text", ...}, {"type": "tool_result", ...}]`), top-level system prompts (string or block arrays), legacy completion prompts, and embedding inputs.
- **Client Header Forwarding**: AI Gateway Proxy now securely extracts and forwards essential client headers (`x-api-key`, `anthropic-version`, `anthropic-beta`, `openai-organization`, `openai-project`, `user-agent`) to upstream LLM APIs, while strictly stripping RFC 7230 §6.1 hop-by-hop headers.
- **Transactional Config Validation**: Added in-process syntax verification for `json` (via `serde_json`) and `toml` (via `toml`), plus TypeScript compilation checks (`ts`/`tsx` via `tsc`) in `apply_code_patch` with automatic zero-dirty-diff rollback.
- **MCP Token Economy**: `get_error_context` now applies `prune_framework_noise` directly to the returned log, slashing prompt token usage by 90%+ for calling AI coding agents.

## [1.2.0] — 2026-09-12

### Added
- **Deep Polyglot Surgery Engine**:
  - **Java/Kotlin Spring Boot 3 & Enterprise Framework Filtering**: Surgically detects and prunes enterprise stack frame noise from Spring Boot (`org.springframework.*`), Tomcat (`org.apache.catalina.*`, `org.apache.tomcat.*`), Hibernate (`org.hibernate.*`), Netty (`io.netty.*`), Undertow (`io.undertow.*`), HikariCP (`com.zaxxer.hikari.*`), Coroutines (`kotlinx.coroutines.*`), and JDK internals (`jdk.internal.*`, `java.base/Thread`), isolating strictly user application code frames.
  - **C/C++ AddressSanitizer (ASan) & GDB Surgery**: Automatically discards sanitizer runtime internals (`__asan_memcpy`, `libasan.so`, `__sanitizer::`), glibc startup wrappers (`__libc_start_call_main`, `libc-start.c`, `sysdeps/`), and binary entry points (`_start`), extracting genuine application source coordinates (`.cpp`, `.cc`, `.c`, `.hpp`).
  - **Go Goroutine Dump Compression**: Detects and prunes idle runtime goroutines (`[force gc (idle)]`, `[GC sweep wait]`, `[finalizer wait]`, `runtime.gopark`, `runtime.forcegchelper`), preserving crashing and active user goroutines.
- **Surgical Pruning Function**: Introduced `extractor::prune_framework_noise` for comprehensive multi-language runtime log compression.
- **Parallel-Bridge Pattern**: Retained 100% backward compatibility for `extractor::is_dependency_file` via an inline zero-overhead delegation shim to `is_framework_noise`.
- **Adversarial Hardening**: Eliminated unchecked unwraps across MCP parameters in `src/mcp.rs`.

## [1.1.7] — 2026-09-12

### Changed
- Normalized official MCP registry namespace casing to exact org identifier: `io.github.Tokenectomy-Labs/razor`.
- Published `tokenectomy-razor@1.1.7` package to npm with validated registry namespace.
- Added official `mcpservers.org` directory listing badge.

## [1.1.6] — 2026-09-11

### Added
- Multi-target precompiled native release binaries for Linux (`x86_64`, `musl`, `aarch64`), macOS (`arm64`, `x86_64`), and Windows (`x86_64`).
- Published `tokenectomy v1.1.6` to crates.io with official registry verification.
- Glama.ai verified maintainer status with 100% Tool Definition Quality Score (TDQS Grade A).

## [1.1.5] — 2026-09-11

### Added
- Static AST code analysis engine (`analyze_code` MCP tool and `/v1/analyze` HTTP reverse proxy endpoint).
- Preemptive code defect detection (unclosed handles, resource leaks, security flaws) with zero external subprocess requirements.
- LSP UTF-16 code offset calculations and bounded analysis limits (50,000 AST nodes, 10 MB payload cap).
- Upgraded all JSON-RPC MCP tool schemas (`get_error_context`, `search_stack_overflow`, `apply_code_patch`, `analyze_code`) with complete TDQS Grade A schema disclosures.
- Adversarial test suite for AST parser boundary conditions and stress limits.

## [1.1.4] — 2026-09-10

### Changed
- Migrated all repository and package URLs to `Tokenectomy-Labs` organization.
- Added Google Search Console verification and docs site SEO optimization.

## [1.1.3] — 2026-09-09

### Added
- Multi-architecture Docker image published to GHCR (`linux/amd64`, `linux/arm64`).
- GitHub Marketplace Action for CI/CD build failure log sanitization.
- Google Antigravity CLI integration guide and `agy mcp add` support.
- Official MCP Registry listing (`io.github.tokenectomy-labs/razor`).

### Improved
- Secret redaction regex patterns expanded (GCP service account keys, Vault tokens).
- Thread pool concurrency benchmark hardened to 100 concurrent OS threads.

### Fixed
- Memory optimization in 250K-line continuous stress workloads (76 MB VmRSS peak).

## [1.1.2] — 2026-08-15

### Improved
- README badge layout and documentation clarity.
- Minor performance tweaks in stack frame parser hot path.

## [1.1.0] — 2026-07-01

### Added
- AI Gateway Reverse Proxy mode (`--proxy`) for transparent LLM request interception.
- SHA-256 idempotency cache with configurable 24-hour TTL.
- PHP (Laravel/Symfony) stack trace parsing and framework frame filtering.
- Production proxy hardening (`--allow-remote`, `--proxy-token`).

## [1.0.0] — 2026-05-15

### Added
- Initial stable release of Tokenectomy Razor.
- Polyglot trace surgery across 7 languages: Rust, Python, TypeScript/JavaScript, Go, Java/Kotlin, C/C++.
- Zero-knowledge deterministic secret redaction with O(N) linear-time ReDoS immunity.
- JSON-RPC 2.0 stdio MCP server compliant with Model Context Protocol specification.
- Three MCP tools: `get_error_context`, `search_stack_overflow`, `apply_code_patch`.
- npm package (`tokenectomy-razor`) for instant `npx` usage.
- Cargo crate (`tokenectomy`) published to crates.io.
- Path traversal containment locked to workspace root (CWD).
- Standalone CLI with `--scrub`, `--file`, `--provider`, `--local-only` modes.

[Unreleased]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.7...HEAD
[1.1.7]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.6...v1.1.7
[1.1.6]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.5...v1.1.6
[1.1.5]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.4...v1.1.5
[1.1.4]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.3...v1.1.4
[1.1.3]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.0...v1.1.2
[1.1.0]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/Tokenectomy-Labs/Tokenectomy/releases/tag/v1.0.0
