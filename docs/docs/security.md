---
title: Security Policy — Tokenectomy Razor
description: Security invariants, vulnerability disclosure, and audit information for Tokenectomy Razor.
---

# Security & Reliability

## Core Invariants

- **Zero-Knowledge Processing**: All scanning and redaction occurs on local hardware before data leaves the system boundary.
- **ReDoS Immunity**: All pattern matchers utilize finite automaton evaluation with linear time guarantees.
- **Path Traversal Isolation**: File operations are strictly locked within workspace boundaries.
- **Memory Safety**: Implemented in safe Rust with bounded stream readers (`.take()`) preventing resource exhaustion attacks.
- **Audit Verification**: Continuous dependency auditing via RustSec advisory databases.

## Vulnerability Disclosure

Please refer to the full [SECURITY.md](https://github.com/Tokenectomy-Labs/Tokenectomy/blob/main/SECURITY.md) for vulnerability disclosure procedures.

## Downstream Forks & Acceptable Use

Tokenectomy Razor is licensed under MIT exclusively for lawful and defensive purposes. Maintainers disclaim all liability for unlawful or malicious use by downstream forks, clones, or private deployments. See [DISCLAIMER.md](https://github.com/Tokenectomy-Labs/Tokenectomy/blob/main/DISCLAIMER.md).
