---
name: spec-first-architect
description: >-
  Use this skill when designing a new feature, implementing complex state transitions, building new modules from scratch, or preventing AI hallucinations and scope creep through rigorous Test-Driven Development (TDD).
---

# 📐 Spec-First Architect (Bulletproof TDD & Contract Synthesis)

A specialized engineering skill that enforces strict Test-Driven Development (TDD) and State Machine Invariant Contracts. It prevents AI models from hallucinating unneeded dependencies, writing sloppy code, or drifting from user requirements.

---

## 🎯 Architecture Phases

### Phase 1: Invariant Contract & State Modeling
1. Deconstruct the user's raw prompt into a formal State Machine:
   - **Explicit States**: e.g., `Idle`, `Processing`, `Success`, `Failed(Reason)`.
   - **Allowed Transitions**: Disallow illegal state jumps.
   - **Invariants**: Guarantees that must hold true before and after every operation.
2. Draft the contract specifications using [TDD State Machine Guide](references/tdd_state_machine_guide.md).
3. Confirm the contract types (structs, enums, error types) before writing business logic.

### Phase 2: Red Stage — Failing Test Synthesis
1. Write the test suite FIRST in dedicated test files.
2. The tests must cover:
   - Standard happy-path execution
   - State transition boundaries
   - Expected error conditions and error variants
   - Idempotency (repeating an operation returns consistent results)
3. **Mandatory Step**: Run the test suite and verify that it FAILS (Compilation error or assertion failure).
   - If tests pass before code is written, the test is invalid.

### Phase 3: Green Stage — Minimalist Implementation
1. Write strictly the minimum viable code necessary to make the failing tests pass.
2. **Anti-Scope-Creep Rule**:
   - Do NOT introduce unrequested helper libraries.
   - Do NOT add speculative "future-proofing" features.
   - Do NOT modify unrelated modules.
3. Run the test command:
   - Rust: `cargo test`
   - Python: `pytest`
   - Node: `npm test`
4. Iterate until all tests are green (100% passing).

### Phase 4: Refactor & Clean Code Optimization
1. Clean up duplicate code while keeping tests green at every step.
2. Verify AST integrity and eliminate dead code or compiler warnings.
3. Run the contract validator:
   ```bash
   bash scripts/contract_validator.sh <project-dir>
   ```

### Phase 5: Verification & Deliverable Documentation
1. Provide a concise summary showing:
   - The State Machine diagram / contract
   - The test assertions and proof of pass
   - The clean, minimalist implementation files
