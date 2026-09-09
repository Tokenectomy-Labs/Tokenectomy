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

Please refer to the full [SECURITY.md](https://github.com/daffa2555/Tokenectomy/blob/main/SECURITY.md) for vulnerability disclosure procedures.
