use clap::{Parser, Subcommand, ValueEnum};
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

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(
        about = "Wrap an agent or test command: runs background AI gateway, injects environment variables, and executes the target process",
        after_help = "EXAMPLES:\n  $ razor wrap -- claude\n  $ razor wrap -- aider --model deepseek/deepseek-chat\n  $ razor wrap npm test\n"
    )]
    Wrap(WrapArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct WrapArgs {
    #[arg(long, default_value = "127.0.0.1:8080", help = "Bind address for the background gateway proxy")]
    pub proxy_bind: Option<String>,

    #[arg(long, default_value = "auto", help = "Upstream LLM base URL (default: 'auto')")]
    pub upstream_url: Option<String>,

    #[arg(long, env = "TOKENECTOMY_MAX_HOURLY_TOKENS", help = "Safety circuit breaker limit")]
    pub max_hourly_tokens: Option<u64>,

    #[arg(long, default_value_t = 3, help = "Maximum retries when upstream returns HTTP 429 rate limit")]
    pub max_retries: usize,

    #[arg(long, default_value_t = true, action = clap::ArgAction::Set, help = "Automatically mitigate upstream HTTP 429 rate limits with exponential backoff")]
    pub auto_retry_429: bool,

    #[arg(trailing_var_arg = true, required = true, help = "Command and arguments to execute with Tokenectomy AI Gateway")]
    pub cmd: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(
    name = "tokenectomy-razor",
    alias = "razor",
    version,
    about = "Tokenectomy Razor 🗡️ — Zero-Waste Token Slicer (Community OSS)",
    after_help = "EXAMPLES:\n  $ python3 app.py 2>&1 | razor\n  $ razor --file error.log\n  $ razor --proxy\n  $ razor wrap -- claude\n  $ razor --mcp\n"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, help = "File to read error log from (reads from stdin if not provided)")]
    pub file: Option<PathBuf>,

    #[arg(short, long, help = "Context lines to extract above/below the error line")]
    pub context_lines: Option<usize>,

    #[arg(long, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(long, help = "Run as an MCP Server (JSON-RPC over stdio)")]
    pub mcp: bool,

    #[arg(long, alias = "sanitize", help = "Surgically scrub framework frames and redact credentials, printing clean log to stdout")]
    pub scrub: bool,

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

    #[arg(long, default_value = "auto", help = "Upstream LLM base URL (default: 'auto' routes Anthropic to api.anthropic.com, OpenAI to api.openai.com, Ollama to localhost:11434)")]
    pub upstream_url: String,

    #[arg(long, env = "TOKENECTOMY_MAX_HOURLY_TOKENS", help = "Safety circuit breaker: maximum raw tokens allowed per rolling hour (prevents runaway agent loops)")]
    pub max_hourly_tokens: Option<u64>,

    #[arg(long, default_value_t = 3, help = "Maximum retries when upstream returns HTTP 429 rate limit")]
    pub max_retries: usize,

    #[arg(long, default_value_t = true, action = clap::ArgAction::Set, help = "Automatically mitigate upstream HTTP 429 rate limits with exponential backoff")]
    pub auto_retry_429: bool,

    #[arg(long, help = "Allow proxy to bind to non-loopback addresses (requires --proxy-token)")]
    pub allow_remote: bool,

    #[arg(long, env = "TOKENECTOMY_PROXY_TOKEN", help = "Authentication token for proxy (required when using --allow-remote)")]
    pub proxy_token: Option<String>,

    #[arg(long, help = "Retrieve and verify a raw dump from cache by its SHA-256 hash or inspect dropped frames")]
    pub diff_verify: Option<String>,
}
