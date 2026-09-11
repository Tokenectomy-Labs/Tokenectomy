---
title: GitHub Actions Integration — CI/CD Log Sanitization
description: Sanitize build failure logs and prevent credential leakage in automated CI/CD workflows with Tokenectomy Razor.
---

# GitHub Actions CI/CD Integration

Sanitize build failure logs and prevent credential leakage in automated workflows.

## Usage

```yaml
- name: Sanitize Build Failure Log
  if: failure()
  uses: Tokenectomy-Labs/Tokenectomy@v1
  with:
    log-file: 'build.log'
    output-file: 'sanitized.log'
```

## Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `log-file` | String | `''` | Path to raw error log file |
| `log-content` | String | `''` | Direct string content (if no file) |
| `output-file` | String | `tokenectomy-sanitized.log` | Path for scrubbed output |
| `version` | String | `v1.1.6` | Binary release target version |
