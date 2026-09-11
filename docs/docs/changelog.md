---
title: Changelog — Tokenectomy Razor Release History
description: Release history and notable changes for Tokenectomy Razor.
---

# Changelog

All notable changes to Tokenectomy Razor are documented here.

## [1.1.7] — 2026-09-12

### Changed
- Normalized official MCP registry namespace casing to exact org identifier: `io.github.Tokenectomy-Labs/razor`.
- Updated `tokenectomy-razor@1.1.7` package on npm with validated registry namespace.
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
- Multi-architecture Docker image (GHCR) for `linux/amd64` and `linux/arm64`.
- GitHub Marketplace Action for CI/CD log sanitization.
- Google Antigravity CLI integration guide.
- Official MCP Registry listing.

### Improved
- Secret redaction regex patterns expanded for GCP service account keys.
- Thread pool concurrency benchmark hardened.

### Fixed
- Memory optimization in 250K-line continuous stress workloads.

## [1.1.2] — 2026-08-15

### Improved
- README badge layout and documentation clarity.
- Minor performance tweaks in stack frame parser.

## [1.1.0] — 2026-07-01

### Added
- AI Gateway Reverse Proxy mode (`--proxy`).
- SHA-256 idempotency cache with 24h TTL.
- PHP (Laravel/Symfony) stack trace support.

## [1.0.0] — 2026-05-15

### Added
- Initial stable release.
- Polyglot trace surgery for Rust, Python, TypeScript/JavaScript, Go, Java/Kotlin, C/C++.
- Zero-knowledge secret redaction with O(N) ReDoS immunity.
- JSON-RPC 2.0 MCP server (stdio transport).
- npm package (`tokenectomy-razor`) and Cargo crate (`tokenectomy`).
