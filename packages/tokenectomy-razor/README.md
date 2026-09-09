# tokenectomy-razor 🗡️

Autonomous M2M Model Context Protocol (MCP) server & log surgery engine for AI coding agents (Claude Desktop, Cursor, Cline, Antigravity, Windsurf).

## 🚀 Instant Usage with npx

No Rust compiler, no manual build, and zero dependencies required!

```bash
npx -y tokenectomy-razor --mcp
```

## 🔌 MCP Client Configuration

### Claude Desktop
Add to your `claude_desktop_config.json`:

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

### Cursor (.cursor/mcp.json)

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

### Antigravity (`agy mcp add`)

```bash
agy mcp add tokenectomy -- npx -y tokenectomy-razor --mcp
```

## 📦 Global Install (Optional)

```bash
npm install -g tokenectomy-razor

# Run standalone CLI
razor --help
tokenectomy-razor --help
```

## 🔗 Repository
[https://github.com/daffa2555/Tokenectomy](https://github.com/daffa2555/Tokenectomy)
