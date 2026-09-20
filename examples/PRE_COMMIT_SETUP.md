# 🪝 Tokenectomy Git Pre-commit Hook Setup

Stop leaking credentials, database connection strings, and massive framework crash traces into your Git commit history.

---

## ⚡ 10-Second Setup

Add Tokenectomy to your `.pre-commit-config.yaml` in your project repository:

```yaml
repos:
  - repo: https://github.com/Tokenectomy-Labs/Tokenectomy
    rev: v1.3.3 # or main
    hooks:
      - id: tokenectomy-scrub
```

Then install the hook with pre-commit:

```bash
pre-commit install
```

---

## 🛡️ What It Does

When running `git commit`, Tokenectomy automatically:
1. Inspects staged `.log`, `.txt`, `.json`, `.trace`, and `.diff` files.
2. Scans for leaked API keys (OpenAI, Anthropic, AWS, GitHub), JWTs, and database URLs.
3. Surgically strips third-party framework frames (`node_modules`, `site-packages`, `.cargo/registry`).
4. Blocks commits that violate zero-leak privacy invariants.
