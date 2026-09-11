# tokenectomy-razor 🗡️

<p align="left">
  <a href="https://mcpservers.org/servers/tokenectomy-labs/tokenectomy"><img src="https://mcpservers.org/badge.svg" alt="Listed on mcpservers.org" /></a>
  <a href="https://registry.modelcontextprotocol.io"><img src="https://img.shields.io/badge/Official%20MCP%20Registry-io.github.Tokenectomy--Labs%2Frazor-brightgreen" alt="Official MCP Registry" /></a>
  <a href="https://www.npmjs.com/package/tokenectomy-razor"><img src="https://img.shields.io/npm/v/tokenectomy-razor.svg?logo=npm" alt="npm version" /></a>
</p>

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
[https://github.com/Tokenectomy-Labs/Tokenectomy](https://github.com/Tokenectomy-Labs/Tokenectomy)
