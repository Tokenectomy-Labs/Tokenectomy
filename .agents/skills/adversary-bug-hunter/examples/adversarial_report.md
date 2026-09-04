# 🩸 Adversarial Security & Stress Test Report

**Target**: `src/parser.rs`  
**Status**: 🔴 Vulnerability Confirmed ➔ 🟢 Hardened & Verified  

---

## 💥 Confirmed Attack Vector
- **Vulnerability Type**: Uncontrolled Recursion Stack Overflow (CWE-674)
- **Attack Payload**: Deeply nested JSON object (Depth: 12,000 layers).
- **Observed Failure**: Process crashed with `SIGSEGV: stack overflow` (Exit Code 139).

## 🛠️ Hardening Patch Applied
- Replaced recursive parser with an iterative parser utilizing an explicit heap stack.
- Introduced a hard `MAX_RECURSION_DEPTH = 256` guard at the ingress tokenizer.

## 🛡️ Immunity Proof
- Re-ran adversarial test `adversarial_depth_bomb`: Gracefully returned `Err(ParseError::DepthLimitExceeded)`.
- Existing regression test suite: 42/42 tests passed (0 regressions).
