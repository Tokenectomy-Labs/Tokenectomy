---
title: MCP Integration — Configure Tokenectomy for AI Agents
description: Set up Tokenectomy Razor as an MCP server for Claude Desktop, Cursor, Cline, Roo Code, Windsurf, and Google Antigravity.
---

# MCP Integration

Configure Tokenectomy Razor as an autonomous background MCP server across major AI agent environments.

## Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

## Cursor

Add to `.cursor/mcp.json` in your project root:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

## Cline / Roo Code / Windsurf / VS Code

Add to your client configuration:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "npx",
      "args": ["-y", "tokenectomy-razor", "--mcp"]
    }
  }
}
```

## Google Antigravity CLI

```bash
agy mcp add tokenectomy-razor -- npx -y tokenectomy-razor --mcp
```

## Exposed MCP Tools

| Tool Name | Description |
|---|---|
| `get_error_context` | Trace surgery on error dumps, removes framework noise, redacts credentials |
| `search_stack_overflow` | Queries Stack Exchange API for relevant error signatures |
| `apply_code_patch` | Atomic file modifications with syntax verification and rollback |
