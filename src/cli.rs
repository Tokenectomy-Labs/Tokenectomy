use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, ValueEnum, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderChoice {
    Anthropic,
    Openai,
    Ollama,
    Mock,
}

#[derive(Parser, Debug)]
#[command(
    name = "tokenectomy-razor",
    alias = "razor",
    version,
    about = "Tokenectomy Razor 🗡️ — Zero-Waste Token Slicer (Community OSS)",
    after_help = "EXAMPLES:\n  $ python3 app.py 2>&1 | razor\n  $ razor --file error.log\n  $ razor --proxy\n  $ razor --mcp\n"
)]
pub struct Cli {
    #[arg(short, long, help = "File to read error log from (reads from stdin if not provided)")]
    pub file: Option<PathBuf>,

    #[arg(short, long, help = "Context lines to extract above/below the error line")]
    pub context_lines: Option<usize>,

    #[arg(long, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(long, help = "Run as an MCP Server (JSON-RPC over stdio)")]
    pub mcp: bool,

    #[arg(long, help = "Force using local provider (Ollama)")]
    pub local_only: bool,

    #[arg(short, long, help = "Skip security prompts (automatically approve)")]
    pub yes: bool,

    #[arg(long, help = "Maximum characters to extract for source code context")]
    pub max_context_chars: Option<usize>,

    #[arg(long, value_enum, help = "AI provider to use [possible values: anthropic, openai, ollama, mock]", hide_possible_values = true)]
    pub provider: Option<ProviderChoice>,

    #[arg(long, help = "Run as an AI Gateway Reverse Proxy (intercepts and compresses LLM prompts)")]
    pub proxy: bool,

    #[arg(long, default_value = "127.0.0.1:8080", help = "Bind address for the reverse proxy gateway")]
    pub proxy_bind: String,

    #[arg(long, default_value = "https://api.openai.com/v1", help = "Upstream LLM base URL to forward requests to")]
    pub upstream_url: String,
}
