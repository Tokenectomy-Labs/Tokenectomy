# 📐 Spec-First Contract Example: Transaction Processor

---

## State Machine Specification
```
[Uncommitted] ──(prepare)──► [Prepared] ──(commit)──► [Committed]
      │                           │
      └──(abort)──┐               └──(abort)──┐
                  ▼                           ▼
              [Aborted]                   [Aborted]
```

## Contract Invariants
1. A transaction cannot be committed unless it has reached `Prepared` state.
2. An aborted transaction can never transition into `Committed`.
3. Once `Committed`, state is immutable and can never be re-executed.

## Test Synthesis (Written First)
```rust
#[test]
fn test_cannot_commit_unprepared_transaction() {
    let tx = Transaction::new();
    let result = tx.commit();
    assert!(matches!(result, Err(TxError::IllegalTransition)));
}

#[test]
fn test_valid_prepare_and_commit_lifecycle() {
    let tx = Transaction::new().prepare().expect("Preparation must succeed");
    let committed = tx.commit().expect("Commit must succeed");
    assert_eq!(committed.status(), TxStatus::Committed);
}
```
