## 📝 Description

Briefly describe the intent and impact of this Pull Request.

Fixes #(issue) <!-- if applicable -->

## 🛠️ Type of Change

- [ ] 🐛 Bug fix (non-breaking change which fixes an issue)
- [ ] ✨ New feature / language parser
- [ ] ⚡ Performance optimization (latency, memory, or token reduction)
- [ ] 📚 Documentation update
- [ ] 🧪 Tests / Benchmarks

## ✅ Verification & Quality Checklist

Before submitting, please ensure your changes meet our standards:

- [ ] `cargo test` passes locally without failures
- [ ] `cargo clippy --all-targets -- -D warnings` passes with 0 warnings
- [ ] `cargo fmt --check` passes
- [ ] Linear-time regex / ReDoS safety verified (if modifying redaction or parser regexes)
- [ ] No secrets, keys, or personal tokens committed
