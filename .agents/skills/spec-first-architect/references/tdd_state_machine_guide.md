# TDD & State Machine Contract Guide

Guidelines for designing zero-bug, self-documenting software architectures.

---

## 1. Type-Driven State Machines (Making Illegal States Unrepresentable)
In languages with strong type systems like Rust, represent states as distinct types or enum variants rather than boolean flags:

```rust
// ❌ Bad: Boolean flag soup (Allows illegal states like is_loading && is_error)
struct DownloadManager {
    is_loading: bool,
    is_error: bool,
    data: Option<Vec<u8>>,
}

// ✅ Good: Exhaustive Enum State Machine
pub enum DownloadState {
    Idle,
    Downloading { bytes_received: usize, total: usize },
    Completed { data: Vec<u8> },
    Failed { error: DownloadError },
}
```

## 2. Invariant Contracts
For every module:
1. **Preconditions**: Requirements that must hold before calling a function (e.g. non-empty string, valid port number).
2. **Postconditions**: Guarantees made by the function upon returning `Ok` (e.g. buffer size matches returned length).
3. **Invariants**: Universal truths about the data structure that never change across transitions.

## 3. Red-Green-Refactor Cycle
- **Red**: Write a test verifying the new state transition. Run test. It MUST fail.
- **Green**: Implement the smallest possible code block. Run test. It MUST pass.
- **Refactor**: Simplify, improve naming, remove duplication. Run test. It MUST remain pass.
