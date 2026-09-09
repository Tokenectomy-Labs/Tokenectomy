---
title: CLI Reference — Tokenectomy Razor Commands
description: Command-line usage reference for Tokenectomy Razor including scrub, proxy, and diagnostic modes.
---

# CLI Reference

In addition to M2M agent mode, Razor provides CLI commands for terminal piping and local shell scripting.

## Log Scrubbing

```bash
# Pipe stderr and scrub framework frames
npm test 2>&1 | razor --scrub > sanitized.log

# Sanitize a specific log file
razor --scrub --file /var/log/app/error.log > sanitized.log
```

## AI Provider Diagnosis

```bash
# With specific AI provider
razor --file error.log --provider openai
razor --file error.log --provider anthropic

# Local-only mode (no network)
razor --file error.log --local-only
```

## MCP Server Mode

```bash
# Start as MCP server (JSON-RPC 2.0 over stdio)
razor --mcp

# Or via npx
npx -y tokenectomy-razor --mcp
```

## Configuration

Configuration values can be set via `~/.tokenectomy.toml`:

```toml
default_provider = "openai"  # openai | anthropic | ollama | mock
openai_api_key = "sk-..."
anthropic_api_key = "sk-ant-..."
ollama_base_url = "http://localhost:11434"
context_lines = 10
max_context_chars = 10000
```
