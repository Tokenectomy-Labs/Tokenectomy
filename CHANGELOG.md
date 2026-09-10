# Changelog

All notable changes to [Tokenectomy Razor](https://github.com/Tokenectomy-Labs/Tokenectomy) are documented here.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.3...HEAD
[1.1.3]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.1.0...v1.1.2
[1.1.0]: https://github.com/Tokenectomy-Labs/Tokenectomy/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/Tokenectomy-Labs/Tokenectomy/releases/tag/v1.0.0
