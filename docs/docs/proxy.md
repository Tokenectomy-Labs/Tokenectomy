---
title: AI Gateway Proxy — Transparent LLM Reverse Proxy
description: Use Tokenectomy Razor as a transparent HTTP reverse proxy for real-time token reduction and secret scrubbing between clients and LLM providers.
---

# AI Gateway Reverse Proxy

Tokenectomy Razor can operate as a transparent local HTTP reverse proxy between client applications and upstream LLM providers (OpenAI, Anthropic, Ollama, OpenRouter).

## Basic Usage

```bash
# Forward to OpenAI
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url https://api.openai.com/v1

# Forward to local Ollama
razor --proxy --proxy-bind 127.0.0.1:8080 --upstream-url http://127.0.0.1:11434/v1
```

Point any SDK or IDE client to the local proxy:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8080/v1"
```

## Production Hardening

Binding to external interfaces requires explicit token authorization:

```bash
razor --proxy --proxy-bind 0.0.0.0:8080 \
  --upstream-url https://api.openai.com/v1 \
  --allow-remote \
  --proxy-token "YOUR_SECURE_TOKEN"
```

### Resource Limits

| Parameter | Default |
|---|---|
| Max Header Size | 64 KB |
| Max Body Size | 10 MB |
| Client Timeout | 30s |
| Upstream Timeout | 60s |
| Concurrency Cap | 128 connections |
