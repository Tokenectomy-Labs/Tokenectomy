# Tokenectomy 🕵️‍♂️

[![smithery badge](https://smithery.ai/badge/tokenectomy)](https://smithery.ai/server/tokenectomy)
[![glama badge](https://glama.ai/badge/tokenectomy)](https://glama.ai/server/tokenectomy)

> 👑 **UPGRADE TO TOKENECTOMY PRO**  
> Want automatic patch applying (Auto-Fixer), AST Guard (Tree-sitter validation), True Ectomy (99% token reduction), and Jira Sync?  
> **[Get Tokenectomy Pro (Enterprise Edition) here ➡️](https://daffaanan.gumroad.com/l/kiznsu)**

**Tokenectomy** is an intelligent, Rust-based Command Line Interface (CLI) that explains application errors using AI. It goes beyond simple log analysis by functioning as a fully featured **Model Context Protocol (MCP) Server**, seamlessly integrating with AI IDEs and assistants like Claude Desktop and Cursor.

---

## ✨ Key Features

- **🔍 Smart Framework Filter**: Automatically bypasses thousands of lines of noisy internal logs (e.g., `node_modules`, `site-packages`) to instantly pinpoint the source code you actually wrote.
- **🌐 Automated Stack Overflow Search**: When encountering obscure errors, the CLI silently queries StackExchange APIs in the background and injects the top community solutions directly into the AI's context.
- **⚡ Blazing Fast Caching**: Error payloads are locally hashed and cached. Recurring errors are resolved instantly without consuming your OpenAI or Anthropic API quotas.
- **🛡️ Enterprise-Grade Security**: 
  - **Secret Redaction**: Advanced Regex engine sanitizes API Keys, AWS Secrets, and JWTs before transmission.
  - **Path Traversal Protection**: MCP edits and reads are strictly locked to your Current Working Directory (CWD).
- **🤖 MCP Server Mode**: Attach `Tokenectomy` to Claude Desktop or Cursor, enabling the AI to read your local logs, browse context, and apply code patches directly to your machine.
- **💅 Hermes-Style UI**: A beautifully crafted, color-graded terminal dashboard with REPL slash commands.

---

## 📦 Installation

### 🛍️ Pre-compiled Binaries (Recommended)
You can download ready-to-use binaries for Windows, macOS, and Linux from our **[Gumroad Store (Pay What You Want)](https://gumroad.com)**. No compilation required!

### ⚙️ Installing via Smithery (For Claude Desktop)
To install Tokenectomy for Claude Desktop automatically via [Smithery](https://smithery.ai/server/tokenectomy):

```bash
npx -y @smithery/cli install tokenectomy --client claude
```

### 🦀 Build from Source
This project is built with Rust for maximum performance and memory safety.

```bash
git clone https://github.com/daffa2555/Tokenectomy.git tokenectomy
cd tokenectomy
cargo build --release
sudo cp target/release/tokenectomy /usr/local/bin/tkmy
```

## 🚀 Usage (CLI Mode)

You can pipe error output directly from your application into `tokenectomy`, or read from an existing log file.

### Reading from a Pipe
```bash
python3 app.py 2>&1 | tokenectomy
```

### Reading from a File
```bash
tokenectomy --file /var/log/nginx/error.log
```

### Advanced Options
```bash
tokenectomy --local-only        # Force local execution via Ollama (100% offline)
tokenectomy --context-lines 20  # Extract 20 lines of context above and below the error
tokenectomy --yes               # Bypass interactive security prompts
```

## 🧠 Configuration (`.tokenectomy.toml`)

Store your API keys and default preferences globally in `~/.tokenectomy.toml` or locally within your project directory.

```toml
default_provider = "openai" # Options: openai, anthropic, ollama, mock
openai_api_key = "sk-..."
anthropic_api_key = "sk-ant-..."
ollama_base_url = "http://localhost:11434"
context_lines = 10
max_context_chars = 10000
```

---

## 🔌 MCP Server Integration

The true power of `Tokenectomy` lies in its Model Context Protocol (MCP) capabilities. Register it with **Claude Desktop** or **Cursor** to grant your AI assistant the ability to autonomously read error logs, fetch Stack Overflow references, and automatically apply code patches.

### Integrating with Claude Desktop
1. Open the Claude Desktop configuration file on Mac/Linux:
   `~/Library/Application Support/Claude/claude_desktop_config.json`
2. Append the following configuration:

```json
{
  "mcpServers": {
    "tokenectomy": {
      "command": "tokenectomy",
      "args": ["--mcp"]
    }
  }
}
```
3. Restart Claude Desktop and experience autonomous debugging!

---

*Built with 🦀 Rust for maximum performance, strict memory safety, and uncompromising security.*
