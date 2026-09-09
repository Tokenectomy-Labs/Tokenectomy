---
title: Changelog — Tokenectomy Razor Release History
description: Release history and notable changes for Tokenectomy Razor.
---

# Changelog

All notable changes to Tokenectomy Razor are documented here.

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
