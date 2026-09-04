---
name: adversary-bug-hunter
description: >-
  Use this skill when the user asks to stress-test, fuzz, red-team, find security vulnerabilities, or break a codebase with adversarial edge cases, followed by hardening and self-healing.
---

# 🩸 Adversary Bug Hunter (Red-Team Fuzzing & Code Hardener)

A specialized skill that shifts the agent from a polite assistant into a ruthless adversary. It systematically analyzes target code, generates adversarial tests, fuzzes edge cases, forces panics/crashes, and then engineers bulletproof remediation patches.

---

## 🎯 Workflow Phases

### Phase 1: Attack Surface Reconnaissance
1. Identify all public functions, API boundaries, CLI arguments, and unvalidated inputs.
2. Run the static hazard scanner:
   ```bash
   bash scripts/fuzz_scanner.sh <target-directory>
   ```
3. Look specifically for:
   - Unchecked unwraps (`.unwrap()`, `.expect()`, `panic!()`)
   - ReDoS patterns (unanchored or nested quantifiers in regular expressions)
   - Integer overflow/underflow in numeric conversions
   - Concurrency race conditions and deadlocks
   - Unhandled edge inputs: empty strings, null bytes (`\0`), 10MB+ buffers, NaN/Infinity, circular structures

### Phase 2: Threat Vector & Exploit Construction
1. Create dedicated adversarial test files named `adversarial_test.*` or property-based tests.
2. Formulate test vectors designed to break assumptions:
   - Extreme boundary values (`usize::MAX`, negative zero, off-by-one indices)
   - Malformed payloads (broken JSON, unterminated strings, invalid UTF-8 sequences)
   - Concurrency torture (spawn 100 threads hammering shared mutable state simultaneously)
3. Refer to [Vulnerability Patterns Reference](references/vulnerability_patterns.md) for language-specific exploit patterns.

### Phase 3: Exploit Execution & Crash Verification
1. Run the adversarial tests and capture the failure output:
   - For Rust: `cargo test --test adversarial_test -- --nocapture`
   - For Node: `npx jest adversarial_test.js`
   - For Python: `pytest tests/adversarial_test.py -v`
2. **Rule**: You MUST witness the code fail or crash. Do not hypothesize a bug without verifying it with executable proof.
3. Document the exact panic stack trace and root vulnerability.

### Phase 4: Hardening & Remediation
1. Apply targeted patches that eliminate the root hazard without breaking existing APIs:
   - Replace unwraps with safe error propagation (`?` or `Result/Option` handling).
   - Constrain regexes with linear-time engines or length pre-checks.
   - Enforce bounded buffers and safe arithmetic (`saturating_add`, `checked_mul`).
   - Add input validation shields at the outermost entry points.

### Phase 5: Immunity Proof & Regression Verification
1. Re-run the adversarial exploit suite: verify that all malicious payloads are now gracefully rejected with clean errors instead of crashing.
2. Run the project's original test suite to ensure zero regressions on the happy path.
3. Generate a structured [Adversarial Incident Report](examples/adversarial_report.md) for the user.
