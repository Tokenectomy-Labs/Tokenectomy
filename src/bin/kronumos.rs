/// Kronumos — Autonomous Bug Remediation & Self-Healing CLI Agent
///
/// A fast, transparent-friendly agentic REPL and autonomous self-healing loop
/// powered by Kronumos Core and the Tokenectomy Rust Sub-Cortex.
///
/// Backends:
///   1. Cloudflare Workers AI (serverless edge, zero cost, streaming SSE)
///   2. Ollama local          (--backend ollama, GGUF offline inference)
///   3. OpenAI-compatible     (--backend openai, Groq, OpenRouter, vLLM)

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use rustyline::{
    completion::{Completer, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::Hinter,
    history::DefaultHistory,
    validate::Validator,
    CompletionType, Config, Editor, Helper,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokenectomy::redact_secrets;
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

// ── CLI Configuration ────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "kronumos",
    about = "Kronumos Kairos — Autonomous Bug Remediation & SRE Agent",
    version = "1.0.0"
)]
struct Cli {
    /// Optional one-shot prompt or command to execute without entering REPL
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    /// Inference backend: "cloudflare", "ollama", or "openai"
    #[arg(long, default_value = "cloudflare", value_parser = ["cloudflare", "ollama", "openai"])]
    backend: String,

    /// Cloudflare Worker gateway URL (or set KRONUMOS_CF_URL env var)
    #[arg(long, env = "KRONUMOS_CF_URL")]
    cf_url: Option<String>,

    /// Cloudflare Worker API secret (optional, or set KRONUMOS_CF_KEY env var)
    #[arg(long, env = "KRONUMOS_CF_KEY")]
    cf_key: Option<String>,

    /// Ollama model name (default: hf.co/NadevA23/Kronumos-GGUF:Q4_K_M or kronumos)
    #[arg(long, default_value = "hf.co/NadevA23/Kronumos-GGUF:Q4_K_M")]
    ollama_model: String,

    /// Ollama host (default: http://localhost:11434)
    #[arg(long, default_value = "http://localhost:11434")]
    ollama_host: String,

    /// OpenAI-compatible API base URL
    #[arg(long, env = "OPENAI_BASE_URL", default_value = "https://api.openai.com/v1")]
    openai_url: String,

    /// OpenAI-compatible API Key
    #[arg(long, env = "OPENAI_API_KEY")]
    openai_key: Option<String>,

    /// Model name for OpenAI-compatible backend
    #[arg(long, default_value = "gpt-4o-mini")]
    openai_model: String,

    /// Non-interactive auto-fix: immediately run test suite and enter autonomous fix loop
    #[arg(long)]
    fix: bool,

    /// Autonomous self-healing loop alias (test-diagnose-patch-verify until green)
    #[arg(long, short = 'l')]
    r#loop: bool,

    /// Suppress banner, progress spinners, and colors (clean piping for unix pipelines)
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Command execution timeout in seconds for tests and tools (default: 120s)
    #[arg(long, default_value = "120")]
    timeout: u64,

    /// Max autonomous tool-call iterations before pausing or reporting failure
    #[arg(long, default_value = "10")]
    max_iterations: usize,

    /// Target workspace directory (default: current directory)
    #[arg(long, default_value = ".")]
    workspace: String,

    /// Automatically rollback unsuccessful patches if tests remain failing (zero dirty diff)
    #[arg(long)]
    auto_rollback: bool,

    /// Automatically commit verified patch to git once tests pass
    #[arg(long)]
    commit: bool,

    /// Automatically create and switch to a new branch for the fix
    #[arg(long)]
    branch: Option<String>,

    /// Output results in structured JSON format (machine-to-machine / CI/CD)
    #[arg(long)]
    json: bool,
}

// ── Persistent Configuration File (.kronumos.toml & ~/.config/kronumos/config.toml) ──

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    backend: Option<String>,
    cf_url: Option<String>,
    cf_key: Option<String>,
    ollama_model: Option<String>,
    ollama_host: Option<String>,
    openai_url: Option<String>,
    openai_key: Option<String>,
    openai_model: Option<String>,
    timeout: Option<u64>,
    max_iterations: Option<usize>,
    auto_rollback: Option<bool>,
    commit: Option<bool>,
    branch: Option<String>,
}

fn load_config(workspace: &str) -> ConfigFile {
    let mut config = ConfigFile::default();

    // 1. Try global user config ~/.config/kronumos/config.toml
    if let Some(config_dir) = dirs::config_dir() {
        let global_path = config_dir.join("kronumos").join("config.toml");
        if global_path.exists() {
            if let Ok(content) = fs::read_to_string(&global_path) {
                if let Ok(parsed) = toml::from_str::<ConfigFile>(&content) {
                    config = parsed;
                }
            }
        }
    }

    // 2. Try project-level config .kronumos.toml (takes precedence over global config)
    let project_path = PathBuf::from(workspace).join(".kronumos.toml");
    if project_path.exists() {
        if let Ok(content) = fs::read_to_string(&project_path) {
            if let Ok(parsed) = toml::from_str::<ConfigFile>(&content) {
                if parsed.backend.is_some() { config.backend = parsed.backend; }
                if parsed.cf_url.is_some() { config.cf_url = parsed.cf_url; }
                if parsed.cf_key.is_some() { config.cf_key = parsed.cf_key; }
                if parsed.ollama_model.is_some() { config.ollama_model = parsed.ollama_model; }
                if parsed.ollama_host.is_some() { config.ollama_host = parsed.ollama_host; }
                if parsed.openai_url.is_some() { config.openai_url = parsed.openai_url; }
                if parsed.openai_key.is_some() { config.openai_key = parsed.openai_key; }
                if parsed.openai_model.is_some() { config.openai_model = parsed.openai_model; }
                if parsed.timeout.is_some() { config.timeout = parsed.timeout; }
                if parsed.max_iterations.is_some() { config.max_iterations = parsed.max_iterations; }
                if parsed.auto_rollback.is_some() { config.auto_rollback = parsed.auto_rollback; }
                if parsed.commit.is_some() { config.commit = parsed.commit; }
                if parsed.branch.is_some() { config.branch = parsed.branch; }
            }
        }
    }

    config
}

impl Cli {
    fn merge_with_config(&mut self, config: ConfigFile) {
        if self.backend == "cloudflare" && config.backend.is_some() {
            self.backend = config.backend.unwrap();
        }
        if self.cf_url.is_none() {
            self.cf_url = config.cf_url;
        }
        if self.cf_key.is_none() {
            self.cf_key = config.cf_key;
        }
        if let Some(m) = config.ollama_model {
            if self.ollama_model == "hf.co/NadevA23/Kronumos-GGUF:Q4_K_M" {
                self.ollama_model = m;
            }
        }
        if let Some(h) = config.ollama_host {
            if self.ollama_host == "http://localhost:11434" {
                self.ollama_host = h;
            }
        }
        if self.openai_key.is_none() {
            self.openai_key = config.openai_key;
        }
        if let Some(u) = config.openai_url {
            if self.openai_url == "https://api.openai.com/v1" {
                self.openai_url = u;
            }
        }
        if let Some(m) = config.openai_model {
            if self.openai_model == "gpt-4o-mini" {
                self.openai_model = m;
            }
        }
        if let Some(t) = config.timeout {
            if self.timeout == 120 {
                self.timeout = t;
            }
        }
        if let Some(mi) = config.max_iterations {
            if self.max_iterations == 10 {
                self.max_iterations = mi;
            }
        }
        if !self.auto_rollback && config.auto_rollback.unwrap_or(false) {
            self.auto_rollback = true;
        }
        if !self.commit && config.commit.unwrap_or(false) {
            self.commit = true;
        }
        if self.branch.is_none() {
            self.branch = config.branch;
        }
    }
}

// ── Message Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

// ── Tool Call Data ───────────────────────────────────────────────────────────

#[derive(Debug)]
struct ToolCall {
    name: String,
    args: Value,
}

// ── Autonomous Loop Result ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixStatus {
    AlreadyPassing,
    Resolved {
        iterations: usize,
        diff_stat: String,
    },
    Unresolved {
        iterations: usize,
        last_error: String,
    },
}

// ── System Prompt ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = r#"You are Kronumos, an elite autonomous bug remediation and SRE agent equipped with the Tokenectomy M2M Sub-Cortex.

Your mission: Diagnose failures, inspect code, synthesize minimal atomic patches, and verify fixes with compiler certainty.

Available Tools — when you decide to take an action, output ONLY a JSON object:

1. run_command
   {"name": "run_command", "arguments": {"command": "<shell command>"}}
   Runs tests (e.g. pytest, cargo test, npm test), build commands, or git checks.

2. view_file
   {"name": "view_file", "arguments": {"path": "<relative file path>", "start_line": <int>, "end_line": <int>}}
   Inspects the exact source lines of a file before attempting any modification.

3. search_code
   {"name": "search_code", "arguments": {"pattern": "<text or symbol to search>", "path": "<optional subpath>"}}
   Searches the codebase for a text pattern or symbol definition across all project files.

4. list_files
   {"name": "list_files", "arguments": {"path": "<optional subpath>", "max_depth": <int>}}
   Lists the project file structure and directories up to max_depth (default: 2).

5. apply_patch
   {"name": "apply_patch", "arguments": {"path": "<relative file path>", "original": "<exact lines to replace>", "replacement": "<new lines>"}}
   Performs a surgical, character-exact replacement in the target file.

6. git_action
   {"name": "git_action", "arguments": {"action": "branch|commit|diff|status", "message": "<commit message>", "branch": "<branch name>"}}
   Manages git branches and commits verified changes.

Guidelines:
- Always run tests or view files first to gather grounded facts before proposing changes.
- Use search_code and list_files to explore unfamiliar projects and locate symbol definitions.
- Never guess code contents: inspect using view_file before editing.
- Ensure patches are minimal, surgical, and maintain zero dirty diffs in unrelated code.
- Re-run tests immediately after patching to verify regression-free status.
- Once verified, commit the fix cleanly or explain findings to the user."#;

// ── Security Guardrails ──────────────────────────────────────────────────────

fn is_safe_workspace_path(path: &str, workspace: &str) -> Result<PathBuf, String> {
    // 1. Block obvious sensitive files
    let lower = path.to_lowercase();
    let sensitive_patterns = [
        ".env", "id_rsa", "id_ed25519", "id_ecdsa", "credentials",
        ".aws/credentials", ".ssh/", ".gnupg/", "passwd", "shadow",
        ".bash_history", ".zsh_history", ".git/config",
    ];
    for pattern in &sensitive_patterns {
        if lower.contains(pattern) {
            return Err(format!("Security error: Access to sensitive file '{}' is prohibited.", path));
        }
    }

    // 2. Prevent path traversal outside workspace
    let ws_path = fs::canonicalize(workspace).map_err(|e| e.to_string())?;
    let target = ws_path.join(path);

    // If file exists, canonicalize and verify workspace prefix
    if target.exists() {
        let canon_target = fs::canonicalize(&target).map_err(|e| e.to_string())?;
        if !canon_target.starts_with(&ws_path) {
            return Err(format!("Security error: Path traversal outside workspace: '{}'", path));
        }
        Ok(canon_target)
    } else {
        // If file doesn't exist yet (new file creation), check parent
        let mut check_dir = target.clone();
        check_dir.pop();
        if let Ok(canon_dir) = fs::canonicalize(&check_dir) {
            if !canon_dir.starts_with(&ws_path) {
                return Err(format!("Security error: Target parent outside workspace: '{}'", path));
            }
        }
        Ok(target)
    }
}

fn is_command_safe(cmd: &str) -> Result<(), String> {
    let trimmed = cmd.trim().to_lowercase();
    let dangerous_patterns = [
        "rm -rf /", "rm -rf ~", "rm -rf $home", "mkfs", "dd if=",
        ":(){ :|:& };:", "> /dev/sda", "chmod -r 777 /", "chown -r",
        "curl | sh", "curl | bash", "wget | sh", "wget | bash",
        "shutdown", "reboot", "init 0",
    ];
    for pattern in &dangerous_patterns {
        if trimmed.contains(pattern) {
            return Err(format!("Security error: Destructive command blocked: '{}'", pattern));
        }
    }
    Ok(())
}

// ── Search & File Exploration Helpers ────────────────────────────────────────

fn search_code_in_dir(
    dir: &Path,
    pattern: &str,
    max_results: usize,
    results: &mut Vec<String>,
    ws_path: &Path,
) {
    if results.len() >= max_results {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let ignored_names = [
        ".git", "node_modules", "target", ".cargo", "vendor", "dist",
        "build", "__pycache__", ".venv", "venv", ".idea", ".vscode",
    ];

    for entry in entries.flatten() {
        if results.len() >= max_results {
            break;
        }
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if ignored_names.iter().any(|&ign| ign == file_name) {
            continue;
        }

        if path.is_dir() {
            search_code_in_dir(&path, pattern, max_results, results, ws_path);
        } else if path.is_file() {
            if let Ok(meta) = path.metadata() {
                if meta.len() > 500_000 {
                    continue;
                }
            }
            if let Ok(content) = fs::read_to_string(&path) {
                let rel_path = path.strip_prefix(ws_path).unwrap_or(&path).to_string_lossy();
                let lower_pat = pattern.to_lowercase();
                for (idx, line) in content.lines().enumerate() {
                    if line.to_lowercase().contains(&lower_pat) {
                        results.push(format!("{}:{}: {}", rel_path, idx + 1, line.trim()));
                        if results.len() >= max_results {
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn list_files_in_dir(
    dir: &Path,
    current_depth: usize,
    max_depth: usize,
    lines: &mut Vec<String>,
    ws_path: &Path,
) {
    if current_depth > max_depth || lines.len() >= 100 {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let ignored_names = [
        ".git", "node_modules", "target", ".cargo", "vendor", "dist",
        "build", "__pycache__", ".venv", "venv", ".idea", ".vscode",
    ];

    let mut dir_entries = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ignored_names.iter().any(|&ign| ign == name) {
            continue;
        }
        dir_entries.push(entry);
    }
    dir_entries.sort_by_key(|e| e.file_name());

    for entry in dir_entries {
        if lines.len() >= 100 {
            break;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let indent = "  ".repeat(current_depth);

        if path.is_dir() {
            lines.push(format!("{}[DIR]  {}/", indent, name));
            list_files_in_dir(&path, current_depth + 1, max_depth, lines, ws_path);
        } else {
            lines.push(format!("{}[FILE] {}", indent, name));
        }
    }
}

// ── Visual Cockpit & Spinner Helpers ─────────────────────────────────────────

fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.cyan.bold} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner())
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

fn print_kronumos_hud(workspace: &str, project_type: &str, _backend: &str) {
    println!();
    println!("{}", "    ██╗  ██╗██████╗  ██████╗ ███╗   ██╗██╗   ██╗███╗   ███╗ ██████╗ ███████╗".bright_cyan().bold());
    println!("{}", "    ██║ ██╔╝██╔══██╗██╔═══██╗████╗  ██║██║   ██║████╗ ████║██╔═══██╗██╔════╝".bright_cyan().bold());
    println!("{}", "    █████╔╝ ██████╔╝██║   ██║██╔██╗ ██║██║   ██║██╔████╔██║██║   ██║███████╗".bright_cyan().bold());
    println!("{}", "    ██╔═██╗ ██╔══██╗██║   ██║██║╚██╗██║██║   ██║██║╚██╔╝██║██║   ██║╚════██║".bright_cyan().bold());
    println!("{}", "    ██║  ██╗██║  ██║╚██████╔╝██║ ╚████║╚██████╔╝██║ ╚═╝ ██║╚██████╔╝███████║".bright_cyan().bold());
    println!("{}", "    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝ ╚═════╝ ╚══════╝".bright_cyan().bold());
    println!();
    println!("{}", "                     · K R O N U M O S   K A I R O S ·".cyan().bold());
    println!("{}", "               Autonomous Bug Remediation & SRE Agent • v1.0".bright_white().bold());
    println!();
    let ws_basename = Path::new(workspace)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| workspace.to_string());
    let short_project = match project_type {
        p if p.contains("Rust") => "Rust",
        p if p.contains("Python") => "Python",
        p if p.contains("Node") => "Node",
        p if p.contains("Go") => "Go",
        _ => "Generic",
    };
    let ws_info = format!("{}/ ({})", ws_basename, short_project);
    let col1_plain = if ws_info.len() > 28 {
        format!("{:.28}", ws_info)
    } else {
        format!("{:<28}", ws_info)
    };
    let col2_plain = format!("{:<24}", "Token Surgery: Active");
    let col3_plain = format!("{:<19}", "Kairos v1.0");

    println!("{}", "╭─ 🧭 WORKSPACE ────────────────── 🧠 SUB-CORTEX ──────────── ⚡ ENGINE ───────────╮".bright_black());
    println!(
        "│  {} │  {} │  {} │",
        col1_plain.bright_white(),
        col2_plain.bright_green().bold(),
        col3_plain.bright_cyan().bold()
    );
    println!("{}", "╰──────────────────────────────────────────────────────────────────────────────────╯".bright_black());
    println!("{}", "  Type / (or press Tab) for command menu, or describe any bug to start autonomous healing.".dimmed());
    println!();
}

fn print_command_cockpit() {
    println!("{}", "╭─ 💡 Command Cockpit ─────────────────────────────────────╮".bright_cyan().bold());
    println!("│  {}  {:<45} │", format!("{:<8}", "/fix").bright_yellow().bold(), "Autonomous TDD test-and-repair loop");
    println!("│  {}  {:<45} │", format!("{:<8}", "/undo").bright_yellow().bold(), "Revert uncommitted patches (zero dirty diff)");
    println!("│  {}  {:<45} │", format!("{:<8}", "/diff").bright_cyan().bold(), "Inspect current uncommitted git diff");
    println!("│  {}  {:<45} │", format!("{:<8}", "/test").bright_green().bold(), "Run project test runner on physical hardware");
    println!("│  {}  {:<45} │", format!("{:<8}", "/clear").dimmed(), "Clear conversation context & buffer");
    println!("│  {}  {:<45} │", format!("{:<8}", "/help").dimmed(), "Show this command cockpit reference");
    println!("│  {}  {:<45} │", format!("{:<8}", "/exit").dimmed(), "Exit interactive session (or double Ctrl+C)");
    println!("{}", "╰──────────────────────────────────────────────────────────╯".bright_cyan().bold());
}

struct KronumosHelper {
    commands: Vec<(&'static str, &'static str)>,
}

impl KronumosHelper {
    fn new() -> Self {
        Self {
            commands: vec![
                ("/fix", "Autonomous TDD test-and-repair loop"),
                ("/undo", "Revert uncommitted patches (zero dirty diff)"),
                ("/diff", "Inspect current uncommitted git diff"),
                ("/test", "Run project test runner on physical hardware"),
                ("/clear", "Clear conversation context & buffer"),
                ("/help", "Show command cockpit reference"),
                ("/exit", "Exit interactive session (or double Ctrl+C)"),
                ("/loop", "Autonomous loop alias"),
            ],
        }
    }
}

impl Completer for KronumosHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        if prefix.starts_with('/') {
            let matches: Vec<Pair> = self
                .commands
                .iter()
                .filter(|(cmd, _)| cmd.starts_with(prefix))
                .map(|(cmd, desc)| Pair {
                    display: format!("{:<8} {}", cmd, desc),
                    replacement: cmd.to_string(),
                })
                .collect();
            return Ok((0, matches));
        }
        Ok((0, Vec::new()))
    }
}

impl Hinter for KronumosHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }
        if line == "/" {
            return Some(" (press Tab for completions, Enter for menu)".dimmed().to_string());
        }
        if line.starts_with('/') {
            for (cmd, desc) in &self.commands {
                if cmd.starts_with(line) && cmd.len() > line.len() {
                    return Some(format!("{} ({})", &cmd[line.len()..], desc).dimmed().to_string());
                }
            }
        }
        None
    }
}

impl Highlighter for KronumosHelper {}
impl Validator for KronumosHelper {}
impl Helper for KronumosHelper {}

// ── Async Tool Executor with Timeout Guard ───────────────────────────────────

async fn execute_tool(tool: &ToolCall, workspace: &str, timeout_secs: u64, quiet: bool) -> String {
    match tool.name.as_str() {
        "run_command" => {
            let cmd = tool.args["command"].as_str().unwrap_or("echo 'no command provided'");
            
            // Security check: block destructive commands
            if let Err(err) = is_command_safe(cmd) {
                if !quiet {
                    eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), err);
                }
                return err;
            }

            if !quiet {
                println!(
                    "{}",
                    format!("╭─ ⚙️  tool: run_command ─────────────────────────────────────────").bright_cyan().bold()
                );
                println!("│  ▶ {}", cmd.bright_white().bold());
            }

            let pb = if !quiet {
                Some(create_spinner(&format!("Executing `{}` on hardware gate...", cmd)))
            } else {
                None
            };

            let mut tokio_cmd = TokioCommand::new("sh");
            tokio_cmd
                .arg("-c")
                .arg(cmd)
                .current_dir(workspace);

            let res = timeout(Duration::from_secs(timeout_secs), tokio_cmd.output()).await;

            if let Some(ref sp) = pb {
                sp.finish_and_clear();
            }

            match res {
                Ok(Ok(o)) => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let combined = format!("{}{}", stdout, stderr);
                    
                    // Sub-Cortex: redact secrets and strip repetitive framework noise
                    let scrubbed = redact_secrets(&combined);
                    let lines: Vec<&str> = scrubbed.lines().collect();
                    let truncated = if lines.len() > 60 {
                        format!(
                            "[...{} lines pruned by Sub-Cortex...]\n{}",
                            lines.len() - 60,
                            lines[lines.len() - 60..].join("\n")
                        )
                    } else {
                        scrubbed
                    };
                    let exit_code = o.status.code().unwrap_or(-1);
                    if !quiet {
                        let status_badge = if o.status.success() {
                            "[✓ exit 0]".bright_green().bold()
                        } else {
                            format!("[✗ exit {}]", exit_code).bright_red().bold()
                        };
                        println!(
                            "{}",
                            format!("╰─ 📋 result {} ─────────────────────────────────────────────", status_badge).bright_cyan().bold()
                        );
                    }
                    format!("exit_code: {}\n{}", exit_code, truncated)
                }
                Ok(Err(e)) => {
                    if !quiet {
                        println!("{}", "╰─ ✗ execution failed ──────────────────────────────────────────".bright_red().bold());
                    }
                    format!("error executing command: {}", e)
                }
                Err(_) => {
                    if !quiet {
                        println!("{}", format!("╰─ ✗ timed out after {}s ────────────────────────────────────────", timeout_secs).bright_red().bold());
                    }
                    format!("error: command timed out after {} seconds.", timeout_secs)
                }
            }
        }

        "view_file" => {
            let path = tool.args["path"].as_str().unwrap_or("");
            let start = tool.args["start_line"].as_u64().unwrap_or(1) as usize;
            let end = tool.args["end_line"].as_u64().unwrap_or(0) as usize;

            // Security check: sandbox within workspace & block credentials
            let full_path = match is_safe_workspace_path(path, workspace) {
                Ok(p) => p,
                Err(e) => {
                    if !quiet {
                        eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    }
                    return e;
                }
            };

            if !quiet {
                println!(
                    "{}",
                    format!("╭─ 📄 view_file: {} (lines {}-{}) ───────────────────────", path, start, if end == 0 { "end".to_string() } else { end.to_string() }).bright_cyan().bold()
                );
            }

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    let scrubbed = redact_secrets(&content);
                    let lines: Vec<&str> = scrubbed.lines().collect();
                    let from = start.saturating_sub(1);
                    let to = if end == 0 || end > lines.len() { lines.len() } else { end };
                    if !quiet {
                        for (i, l) in lines[from..to].iter().take(15).enumerate() {
                            println!("│ {:4} │ {}", from + i + 1, l.dimmed());
                        }
                        if to - from > 15 {
                            println!("│      [...{} more lines...]", to - from - 15);
                        }
                        println!("{}", "╰────────────────────────────────────────────────────────────────".bright_cyan().bold());
                    }
                    lines[from..to]
                        .iter()
                        .enumerate()
                        .map(|(i, l)| format!("{:4} │ {}", from + i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Err(e) => {
                    if !quiet {
                        println!("{}", "╰─ ✗ error reading file ─────────────────────────────────────────".bright_red().bold());
                    }
                    format!("error reading file {}: {}", path, e)
                }
            }
        }

        "search_code" => {
            let pattern = tool.args["pattern"].as_str().unwrap_or("");
            let subpath = tool.args["path"].as_str().unwrap_or(".");
            if pattern.is_empty() {
                return "error: 'pattern' argument is required for search_code".to_string();
            }

            let search_dir = match is_safe_workspace_path(subpath, workspace) {
                Ok(p) => p,
                Err(e) => {
                    if !quiet {
                        eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    }
                    return e;
                }
            };

            if !quiet {
                println!(
                    "{}",
                    format!("╭─ 🔍 search_code: '{}' in {} ───────────────────────────────", pattern, subpath).bright_cyan().bold()
                );
            }

            let ws_path = match fs::canonicalize(workspace) {
                Ok(p) => p,
                Err(e) => return format!("error resolving workspace: {}", e),
            };

            let mut matches = Vec::new();
            search_code_in_dir(&search_dir, pattern, 25, &mut matches, &ws_path);

            if !quiet {
                for m in matches.iter().take(8) {
                    println!("│  {}", m.bright_white());
                }
                if matches.len() > 8 {
                    println!("│  [...{} more occurrences found...]", matches.len() - 8);
                }
                println!("{}", format!("╰─ Found {} matches ───────────────────────────────────────────", matches.len()).bright_cyan().bold());
            }

            if matches.is_empty() {
                format!("No occurrences of '{}' found in '{}'", pattern, subpath)
            } else {
                format!("Found {} matches for '{}':\n{}", matches.len(), pattern, matches.join("\n"))
            }
        }

        "list_files" => {
            let subpath = tool.args["path"].as_str().unwrap_or(".");
            let max_depth = tool.args["max_depth"].as_u64().unwrap_or(2).min(4) as usize;

            let target_dir = match is_safe_workspace_path(subpath, workspace) {
                Ok(p) => p,
                Err(e) => {
                    if !quiet {
                        eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    }
                    return e;
                }
            };

            if !quiet {
                println!(
                    "{}",
                    format!("╭─ 📂 list_files: {} (max_depth: {}) ──────────────────────────", subpath, max_depth).bright_cyan().bold()
                );
            }

            let ws_path = match fs::canonicalize(workspace) {
                Ok(p) => p,
                Err(e) => return format!("error resolving workspace: {}", e),
            };

            let mut lines = Vec::new();
            list_files_in_dir(&target_dir, 0, max_depth, &mut lines, &ws_path);

            if !quiet {
                for l in lines.iter().take(15) {
                    println!("│  {}", l.dimmed());
                }
                if lines.len() > 15 {
                    println!("│  [...{} more items...]", lines.len() - 15);
                }
                println!("{}", format!("╰─ {} items indexed ─────────────────────────────────────────────", lines.len()).bright_cyan().bold());
            }

            if lines.is_empty() {
                format!("Directory '{}' is empty or contained only ignored files.", subpath)
            } else {
                format!("Directory structure of '{}':\n{}", subpath, lines.join("\n"))
            }
        }

        "apply_patch" => {
            let path = tool.args["path"].as_str().unwrap_or("");
            let original = tool.args["original"].as_str().unwrap_or("");
            let replacement = tool.args["replacement"].as_str().unwrap_or("");

            let full_path = match is_safe_workspace_path(path, workspace) {
                Ok(p) => p,
                Err(e) => {
                    if !quiet {
                        eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    }
                    return e;
                }
            };

            if !quiet {
                println!("{}", format!("╭─ 🩹 apply_patch: {} ─────────────────────────────────────────", path).bright_yellow().bold());
                for line in original.lines().take(6) {
                    println!("│ {}", format!("- {}", line).bright_red());
                }
                if original.lines().count() > 6 {
                    println!("│ {}", format!("  [...{} lines removed...]", original.lines().count() - 6).dimmed());
                }
                for line in replacement.lines().take(6) {
                    println!("│ {}", format!("+ {}", line).bright_green());
                }
                if replacement.lines().count() > 6 {
                    println!("│ {}", format!("  [...{} lines added...]", replacement.lines().count() - 6).dimmed());
                }
            }

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    if !content.contains(original) {
                        if !quiet {
                            println!("{}", "╰─ ✗ original hunk context not found in file ──────────────────".bright_red().bold());
                        }
                        return format!(
                            "error: Target content not found in {}. Ensure exact indentation and matching context.",
                            path
                        );
                    }
                    let new_content = content.replacen(original, replacement, 1);
                    match fs::write(&full_path, &new_content) {
                        Ok(_) => {
                            let added = replacement.lines().count();
                            let removed = original.lines().count();
                            if !quiet {
                                println!("{}", format!("╰─ ✓ patch applied (+{} lines, -{} lines) ──────────────────────────", added, removed).bright_green().bold());
                            }
                            format!(
                                "patch applied successfully: {} (+{} lines, -{} lines)",
                                path, added, removed
                            )
                        }
                        Err(e) => {
                            if !quiet {
                                println!("{}", "╰─ ✗ write error ───────────────────────────────────────────────".bright_red().bold());
                            }
                            format!("error writing file {}: {}", path, e)
                        }
                    }
                }
                Err(e) => {
                    if !quiet {
                        println!("{}", "╰─ ✗ read error ────────────────────────────────────────────────".bright_red().bold());
                    }
                    format!("error reading file {}: {}", path, e)
                }
            }
        }

        "git_action" => {
            let action = tool.args["action"].as_str().unwrap_or("status");
            let message = tool.args["message"].as_str().unwrap_or("kronumos: patch remediation");
            let branch = tool.args["branch"].as_str().unwrap_or("kronumos/fix");

            if !quiet {
                println!(
                    "{}",
                    format!("╭─ 🚀 git_action: {} ({}) ───────────────────────────────────────", action, branch).bright_green().bold()
                );
            }

            let output = match action {
                "branch" => {
                    TokioCommand::new("git")
                        .args(["checkout", "-b", branch])
                        .current_dir(workspace)
                        .output()
                        .await
                }
                "commit" => {
                    let _ = TokioCommand::new("git")
                        .args(["add", "-A"])
                        .current_dir(workspace)
                        .output()
                        .await;
                    TokioCommand::new("git")
                        .args(["commit", "-m", message])
                        .current_dir(workspace)
                        .output()
                        .await
                }
                "diff" => {
                    TokioCommand::new("git")
                        .args(["diff"])
                        .current_dir(workspace)
                        .output()
                        .await
                }
                "push" => {
                    TokioCommand::new("git")
                        .args(["push", "-u", "origin", branch])
                        .current_dir(workspace)
                        .output()
                        .await
                }
                _ => {
                    TokioCommand::new("git")
                        .args(["status", "--short"])
                        .current_dir(workspace)
                        .output()
                        .await
                }
            };

            match output {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout);
                    let err = String::from_utf8_lossy(&o.stderr);
                    let res = format!("{}{}", out, err).trim().to_string();
                    if !quiet {
                        if !res.is_empty() {
                            for line in res.lines().take(10) {
                                println!("│  {}", line.dimmed());
                            }
                        }
                        println!("{}", "╰─────────────────────────────────────────────────────────────────".bright_green().bold());
                    }
                    if res.is_empty() { "ok (no output)".to_string() } else { res }
                }
                Err(e) => {
                    if !quiet {
                        println!("{}", "╰─ ✗ git error ──────────────────────────────────────────────────".bright_red().bold());
                    }
                    format!("error executing git action: {}", e)
                }
            }
        }

        unknown => format!("unknown tool: {}", unknown),
    }
}

// ── Tool Call Parser ─────────────────────────────────────────────────────────

fn extract_tool_call(text: &str) -> Option<ToolCall> {
    if let Ok(re) = regex::Regex::new(r#"\{[^{}]*"name"\s*:[^{}]*"arguments"\s*:\s*\{[^{}]*\}[^{}]*\}"#) {
        if let Some(m) = re.find(text) {
            if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
                let name = v["name"].as_str()?.to_string();
                let args = v["arguments"].clone();
                return Some(ToolCall { name, args });
            }
        }
    }

    if let Ok(re2) = regex::Regex::new(r#"\{[^{}]+\}"#) {
        for m in re2.find_iter(text) {
            if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
                if let Some(name) = v["name"].as_str() {
                    let args = v.get("arguments").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
                    return Some(ToolCall { name: name.to_string(), args });
                }
            }
        }
    }
    None
}

// ── Inference Streamers ──────────────────────────────────────────────────────

async fn stream_cloudflare(
    client: &Client,
    url: &str,
    api_key: Option<&str>,
    messages: &[Message],
    quiet: bool,
) -> Result<String> {
    let payload = json!({
        "messages": messages,
        "stream": true,
        "max_tokens": 768,
        "temperature": 0.2,
    });

    let pb = if !quiet {
        Some(create_spinner("Consulting Sub-Cortex & synthesizing response..."))
    } else {
        None
    };

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json");

    if let Some(key) = api_key {
        if !key.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key.trim()));
        }
    }

    let response = match req.json(&payload).send().await {
        Ok(r) => r,
        Err(e) => {
            if let Some(ref sp) = pb { sp.finish_and_clear(); }
            return Err(e).context("Failed to connect to Cloudflare Worker gateway");
        }
    };

    if !response.status().is_success() {
        if let Some(ref sp) = pb { sp.finish_and_clear(); }
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Cloudflare Worker error ({}): {}", status, err_text);
    }

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();
    let mut first_token = true;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                if let Some(ref sp) = pb { sp.finish_and_clear(); }
                return Err(e).context("Stream read error");
            }
        };
        let raw = String::from_utf8_lossy(&chunk);

        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let token = v["response"]
                        .as_str()
                        .or_else(|| v["choices"][0]["delta"]["content"].as_str())
                        .unwrap_or("");
                    if !token.is_empty() {
                        if first_token {
                            first_token = false;
                            if let Some(ref sp) = pb {
                                sp.finish_and_clear();
                            }
                            if !quiet {
                                print!("{}", "◈ Kronumos Kairos ❯ ".bright_cyan().bold());
                                io::stdout().flush()?;
                            }
                        }
                        if !quiet {
                            print!("{}", token);
                            io::stdout().flush()?;
                        }
                        full_text.push_str(token);
                    }
                }
            }
        }
    }
    if let Some(ref sp) = pb {
        sp.finish_and_clear();
    }
    if !quiet {
        println!();
    }
    Ok(full_text)
}

async fn stream_ollama(
    client: &Client,
    host: &str,
    model: &str,
    messages: &[Message],
    quiet: bool,
) -> Result<String> {
    let payload = json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "options": { "temperature": 0.2, "num_predict": 768 }
    });

    let pb = if !quiet {
        Some(create_spinner(&format!("Querying local Ollama model `{}`...", model)))
    } else {
        None
    };

    let response = match client
        .post(format!("{}/api/chat", host.trim_end_matches('/')))
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            if let Some(ref sp) = pb { sp.finish_and_clear(); }
            return Err(e).context(format!("Failed to connect to Ollama at {} — is Ollama running?", host));
        }
    };

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();
    let mut first_token = true;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                if let Some(ref sp) = pb { sp.finish_and_clear(); }
                return Err(e).context("Ollama stream read error");
            }
        };
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                let token = v["message"]["content"].as_str().unwrap_or("");
                if !token.is_empty() {
                    if first_token {
                        first_token = false;
                        if let Some(ref sp) = pb {
                            sp.finish_and_clear();
                        }
                        if !quiet {
                            print!("{}", "◈ Kronumos Kairos ❯ ".bright_cyan().bold());
                            io::stdout().flush()?;
                        }
                    }
                    if !quiet {
                        print!("{}", token);
                        io::stdout().flush()?;
                    }
                    full_text.push_str(token);
                }
                if v["done"].as_bool().unwrap_or(false) {
                    break;
                }
            }
        }
    }
    if let Some(ref sp) = pb {
        sp.finish_and_clear();
    }
    if !quiet {
        println!();
    }
    Ok(full_text)
}

async fn stream_openai_compat(
    client: &Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[Message],
    quiet: bool,
) -> Result<String> {
    let payload = json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "max_tokens": 768,
        "temperature": 0.2,
    });

    let pb = if !quiet {
        Some(create_spinner(&format!("Consulting inference backend `{}`...", model)))
    } else {
        None
    };

    let mut req = client
        .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
        .header("Content-Type", "application/json");

    if !api_key.trim().is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key.trim()));
    }

    let response = match req.json(&payload).send().await {
        Ok(r) => r,
        Err(e) => {
            if let Some(ref sp) = pb { sp.finish_and_clear(); }
            return Err(e).context("Failed to connect to OpenAI-compatible inference server");
        }
    };

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();
    let mut first_token = true;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                if let Some(ref sp) = pb { sp.finish_and_clear(); }
                return Err(e).context("OpenAI stream read error");
            }
        };
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let token = v["choices"][0]["delta"]["content"].as_str().unwrap_or("");
                    if !token.is_empty() {
                        if first_token {
                            first_token = false;
                            if let Some(ref sp) = pb {
                                sp.finish_and_clear();
                            }
                            if !quiet {
                                print!("{}", "◈ Kronumos Kairos ❯ ".bright_cyan().bold());
                                io::stdout().flush()?;
                            }
                        }
                        if !quiet {
                            print!("{}", token);
                            io::stdout().flush()?;
                        }
                        full_text.push_str(token);
                    }
                }
            }
        }
    }
    if let Some(ref sp) = pb {
        sp.finish_and_clear();
    }
    if !quiet {
        println!();
    }
    Ok(full_text)
}

// ── Context Sliding Window & Token Budgeting ─────────────────────────────────

fn compact_history(history: &mut Vec<Message>, max_messages: usize) {
    if history.len() > max_messages {
        let preserve_recent = 6;
        if history.len() > 2 + preserve_recent {
            let prune_count = history.len() - 2 - preserve_recent;
            history.drain(2..2 + prune_count);
            history.insert(2, Message {
                role: "system".to_string(),
                content: format!("[Context Compaction: {} intermediate debugging steps pruned to preserve token limits]", prune_count),
            });
        }
    }
}

// ── Autonomous Self-Healing Closed Loop Engine ───────────────────────────────

async fn run_autonomous_loop(
    client: &Client,
    cli: &Cli,
    workspace: &str,
    project_type: &str,
    max_iter: usize,
    timeout_secs: u64,
    quiet: bool,
) -> Result<FixStatus> {
    let test_cmd = match project_type {
        p if p.contains("Rust") => "cargo test",
        p if p.contains("Python") => "pytest",
        p if p.contains("Node") => "npm test",
        p if p.contains("Go") => "go test ./...",
        _ => "make test",
    };

    if !quiet {
        println!("{}", "╭─ 🔄 Autonomous TDD Healing Loop Engaged ───────────────".bright_cyan().bold());
        println!("  {} Project Type: {}", "📦".cyan(), project_type.bright_white());
        println!("  {} Test Runner:  `{}`", "🧪".cyan(), test_cmd.bright_yellow());
        println!("  {} Max Rounds:   {}", "⏱️".cyan(), max_iter.to_string().bright_white());
        println!("{}", "╰──────────────────────────────────────────────────────────".bright_cyan().bold());
        println!();
        println!("  {} Probing workspace test baseline...", "▶".cyan().bold());
    }

    // Step 1: Baseline test probe
    let baseline_tool = ToolCall {
        name: "run_command".to_string(),
        args: json!({ "command": test_cmd }),
    };
    let baseline_res = execute_tool(&baseline_tool, workspace, timeout_secs, quiet).await;

    // Check if tests are already green
    let baseline_passed = baseline_res.starts_with("exit_code: 0");
    if baseline_passed {
        if !quiet {
            println!("  {}", "✓ Baseline check: All tests already passing. No code defects detected.".bright_green().bold());
        }
        return Ok(FixStatus::AlreadyPassing);
    }

    if !quiet {
        println!("  {}", "✗ Tests failing. Engaging closed-loop remediation...".bright_red().bold());
    }

    // Step 2: Initialize conversation with baseline diagnostics
    let mut history: Vec<Message> = vec![
        Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        },
        Message {
            role: "user".to_string(),
            content: format!(
                "The test runner `{}` failed in workspace '{}' ({}):\n\n{}\n\nDiagnose the root cause, view the relevant source files with `view_file`, synthesize a minimal surgical patch with `apply_patch`, and verify that all tests pass.",
                test_cmd, workspace, project_type, baseline_res
            ),
        },
    ];

    let mut iteration = 0;
    let mut last_error = baseline_res.clone();

    while iteration < max_iter {
        compact_history(&mut history, 12);
        iteration += 1;
        if !quiet {
            println!(
                "{}",
                format!("\n─── [Healing Loop Round {}/{}] Synthesizing Fix ───", iteration, max_iter)
                    .bright_yellow().bold()
            );
        }

        // Stream model inference
        let response_text = match cli.backend.as_str() {
            "cloudflare" => {
                let default_url = "https://kronumos-gateway.your-subdomain.workers.dev";
                let url = cli.cf_url.as_deref().unwrap_or(default_url);
                if url.contains("your-subdomain") {
                    if !quiet {
                        eprintln!("\n{} Please configure your Cloudflare Worker URL or choose another backend:", "⚠️ Gateway Not Configured:".bright_yellow().bold());
                        eprintln!("  1. Set environment variable: export KRONUMOS_CF_URL=\"https://your-worker.workers.dev\"");
                        eprintln!("  2. Or run with local Ollama: kronumos --fix --backend ollama");
                        eprintln!("  3. Or run with OpenAI/Groq:  export OPENAI_API_KEY=\"...\" && kronumos --fix --backend openai\n");
                    }
                    return Ok(FixStatus::Unresolved {
                        iterations: 0,
                        last_error: "Cloudflare Worker URL not configured (placeholder detected)".to_string(),
                    });
                }
                match stream_cloudflare(client, url, cli.cf_key.as_deref(), &history, quiet).await {
                    Ok(t) => t,
                    Err(e) => {
                        if !quiet {
                            eprintln!("  {} Backend connection failed: {}", "✗".bright_red(), e);
                        }
                        return Ok(FixStatus::Unresolved {
                            iterations: iteration,
                            last_error: format!("Backend connection error: {}", e),
                        });
                    }
                }
            }
            "ollama" => {
                match stream_ollama(client, &cli.ollama_host, &cli.ollama_model, &history, quiet).await {
                    Ok(t) => t,
                    Err(e) => {
                        if !quiet {
                            eprintln!("  {} Ollama connection failed: {}", "✗".bright_red(), e);
                        }
                        return Ok(FixStatus::Unresolved {
                            iterations: iteration,
                            last_error: format!("Ollama connection error: {}", e),
                        });
                    }
                }
            }
            "openai" => {
                let key = cli.openai_key.as_deref().unwrap_or("");
                match stream_openai_compat(client, &cli.openai_url, key, &cli.openai_model, &history, quiet).await {
                    Ok(t) => t,
                    Err(e) => {
                        if !quiet {
                            eprintln!("  {} OpenAI backend connection failed: {}", "✗".bright_red(), e);
                        }
                        return Ok(FixStatus::Unresolved {
                            iterations: iteration,
                            last_error: format!("OpenAI connection error: {}", e),
                        });
                    }
                }
            }
            _ => unreachable!(),
        };

        history.push(Message {
            role: "assistant".to_string(),
            content: response_text.clone(),
        });

        if let Some(tool_call) = extract_tool_call(&response_text) {
            let is_patch = tool_call.name == "apply_patch";
            let tool_result = execute_tool(&tool_call, workspace, timeout_secs, quiet).await;

            history.push(Message {
                role: "user".to_string(),
                content: format!("Tool execution result for {}:\n{}", tool_call.name, tool_result),
            });

            // Verification Gate: if a patch was applied, IMMEDIATELY re-run the test suite!
            if is_patch && tool_result.contains("patch applied successfully") {
                if !quiet {
                    println!("\n  {} Verifying patch with `{}`...", "🧪".bright_cyan().bold(), test_cmd);
                }

                let verify_tool = ToolCall {
                    name: "run_command".to_string(),
                    args: json!({ "command": test_cmd }),
                };
                let verify_res = execute_tool(&verify_tool, workspace, timeout_secs, quiet).await;
                let tests_pass = verify_res.starts_with("exit_code: 0");

                if tests_pass {
                    // Extract git diff stat for transparent verification summary
                    let diff_stat = match TokioCommand::new("git")
                        .args(["diff", "--stat"])
                        .current_dir(workspace)
                        .output()
                        .await
                    {
                        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                        Err(_) => String::new(),
                    };

                    // Auto Branch Delivery
                    if let Some(ref b) = cli.branch {
                        let _ = TokioCommand::new("git")
                            .args(["checkout", "-b", b])
                            .current_dir(workspace)
                            .output()
                            .await;
                        if !quiet {
                            println!("  {} Switched to branch `{}`", "🌿".bright_green().bold(), b.bright_white());
                        }
                    }

                    // Auto Commit Delivery
                    if cli.commit {
                        let _ = TokioCommand::new("git")
                            .args(["add", "-A"])
                            .current_dir(workspace)
                            .output()
                            .await;
                        let commit_msg = format!("fix(remediation): verified tests green via Kronumos (round {})", iteration);
                        let _ = TokioCommand::new("git")
                            .args(["commit", "-m", &commit_msg])
                            .current_dir(workspace)
                            .output()
                            .await;
                        if !quiet {
                            println!("  {} Fix committed cleanly: `{}`", "💾".bright_green().bold(), commit_msg.bright_white());
                        }
                    }

                    if !quiet {
                        println!("\n{}", "🎉 ────────────────────────────────────────────────────────────".bright_green().bold());
                        println!("{}", format!("✓ Fix Verified! All tests passed in round {}.", iteration).bright_green().bold());
                        if !diff_stat.is_empty() {
                            println!("\n  {} Verified Changes:\n  {}", "📊".cyan(), diff_stat.bright_white());
                        }
                        println!("{}", "──────────────────────────────────────────────────────────────".bright_green().bold());
                    }

                    return Ok(FixStatus::Resolved {
                        iterations: iteration,
                        diff_stat,
                    });
                } else {
                    last_error = verify_res.clone();
                    if !quiet {
                        println!("  {}", "✗ Tests still failing after patch. Feeding compiler diagnostics back to agent...".bright_red().bold());
                    }
                    history.push(Message {
                        role: "user".to_string(),
                        content: format!(
                            "Verification test failed after applying your patch:\n{}\nPlease analyze why this patch was insufficient, view additional context if needed, and synthesize an updated patch.",
                            verify_res
                        ),
                    });
                }
            }
        } else {
            // Model did not output a tool JSON. Run verification check to see if issue was resolved
            let verify_tool = ToolCall {
                name: "run_command".to_string(),
                args: json!({ "command": test_cmd }),
            };
            let verify_res = execute_tool(&verify_tool, workspace, timeout_secs, quiet).await;
            if verify_res.starts_with("exit_code: 0") {
                let diff_stat = match TokioCommand::new("git")
                    .args(["diff", "--stat"])
                    .current_dir(workspace)
                    .output()
                    .await
                {
                    Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                    Err(_) => String::new(),
                };

                if let Some(ref b) = cli.branch {
                    let _ = TokioCommand::new("git")
                        .args(["checkout", "-b", b])
                        .current_dir(workspace)
                        .output()
                        .await;
                }
                if cli.commit {
                    let _ = TokioCommand::new("git")
                        .args(["add", "-A"])
                        .current_dir(workspace)
                        .output()
                        .await;
                    let commit_msg = format!("fix(remediation): verified tests green via Kronumos (round {})", iteration);
                    let _ = TokioCommand::new("git")
                        .args(["commit", "-m", &commit_msg])
                        .current_dir(workspace)
                        .output()
                        .await;
                }

                return Ok(FixStatus::Resolved {
                    iterations: iteration,
                    diff_stat,
                });
            }
        }
    }

    if !quiet {
        eprintln!(
            "\n{}",
            format!("⚠ Autonomous loop completed {} iterations without resolving all failing tests.", max_iter)
                .bright_yellow().bold()
        );
    }

    if cli.auto_rollback {
        let _ = TokioCommand::new("git")
            .args(["restore", "."])
            .current_dir(workspace)
            .output()
            .await;
        if !quiet {
            eprintln!("  {}", "🛡️ Auto-rollback: Unsuccessful patches reverted. Zero dirty diff preserved.".bright_yellow().bold());
        }
    }

    Ok(FixStatus::Unresolved {
        iterations: max_iter,
        last_error,
    })
}

// ── Agentic Turn Processing (General Chat / Commands) ─────────────────────────

async fn process_turn(
    client: &Client,
    cli: &Cli,
    history: &mut Vec<Message>,
    user_input: &str,
    workspace: &str,
    max_iter: usize,
    timeout_secs: u64,
    quiet: bool,
) -> Result<()> {
    history.push(Message {
        role: "user".to_string(),
        content: user_input.to_string(),
    });

    let mut iterations = 0;

    loop {
        compact_history(history, 12);
        if iterations >= max_iter {
            if !quiet {
                println!(
                    "\n{}",
                    format!("⚠ Reached safety limit of {} tool iterations. Pausing for confirmation.", max_iter)
                        .bright_yellow().bold()
                );
            }
            break;
        }
        iterations += 1;

        let response_text = match cli.backend.as_str() {
            "cloudflare" => {
                let default_url = "https://kronumos-gateway.your-subdomain.workers.dev";
                let url = cli.cf_url.as_deref().unwrap_or(default_url);
                if url.contains("your-subdomain") {
                    eprintln!("\n{} Please configure your Cloudflare Worker URL or choose another backend:", "⚠️ Gateway Not Configured:".bright_yellow().bold());
                    eprintln!("  1. Set environment variable: export KRONUMOS_CF_URL=\"https://your-worker.workers.dev\"");
                    eprintln!("  2. Or run with local Ollama: kronumos --backend ollama");
                    eprintln!("  3. Or run with OpenAI/Groq:  export OPENAI_API_KEY=\"...\" && kronumos --backend openai\n");
                    anyhow::bail!("Cloudflare Worker URL not configured (placeholder detected)");
                }
                stream_cloudflare(client, url, cli.cf_key.as_deref(), history, quiet).await?
            }
            "ollama" => {
                stream_ollama(client, &cli.ollama_host, &cli.ollama_model, history, quiet).await?
            }
            "openai" => {
                let key = cli.openai_key.as_deref().unwrap_or("");
                stream_openai_compat(client, &cli.openai_url, key, &cli.openai_model, history, quiet).await?
            }
            _ => unreachable!(),
        };

        history.push(Message {
            role: "assistant".to_string(),
            content: response_text.clone(),
        });

        if let Some(tool_call) = extract_tool_call(&response_text) {
            let tool_result = execute_tool(&tool_call, workspace, timeout_secs, quiet).await;

            history.push(Message {
                role: "user".to_string(),
                content: format!(
                    "Tool execution result for {}:\n{}",
                    tool_call.name, tool_result
                ),
            });

            let completion_signals = [
                "all tests passed",
                "tests passed",
                "test result: ok",
                "fix verified",
                "committed fix",
            ];
            if completion_signals.iter().any(|s| tool_result.to_lowercase().contains(s)) {
                continue;
            }
        } else {
            break;
        }
    }
    Ok(())
}

// ── Workspace Helper ─────────────────────────────────────────────────────────

fn detect_project_type(workspace: &str) -> String {
    let indicators = [
        ("Cargo.toml",      "Rust (Cargo)"),
        ("pyproject.toml",  "Python (PEP 517 / Poetry / Hatch)"),
        ("setup.py",        "Python (Setuptools)"),
        ("package.json",    "Node.js / TypeScript"),
        ("go.mod",          "Go Module"),
        ("pom.xml",         "Java (Maven)"),
        ("build.gradle",    "Java / Kotlin (Gradle)"),
        ("Makefile",        "Make Project"),
    ];
    for (file, label) in &indicators {
        if PathBuf::from(workspace).join(file).exists() {
            return label.to_string();
        }
    }
    "Generic Workspace".to_string()
}

// ── Entry Point & REPL ───────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();
    let workspace = cli.workspace.clone();
    let workspace_abs = fs::canonicalize(&workspace)
        .unwrap_or_else(|_| PathBuf::from(&workspace));
    let workspace_str = workspace_abs.to_string_lossy().to_string();

    // Load persistent config and merge with CLI arguments
    let config = load_config(&workspace_str);
    cli.merge_with_config(config);

    if cli.json {
        cli.quiet = true;
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;

    let project_type = detect_project_type(&workspace_str);

    // ── Mode 1: Piped Stdin Execution (cat error.log | kronumos) ────────────
    if !io::stdin().is_terminal() {
        let mut piped_input = String::new();
        io::stdin().read_to_string(&mut piped_input)?;
        if !piped_input.trim().is_empty() {
            let combined = if let Some(ref p) = cli.prompt {
                format!("{}\n\nContext:\n{}", p, piped_input)
            } else {
                piped_input
            };
            let mut history = vec![Message {
                role: "system".to_string(),
                content: SYSTEM_PROMPT.to_string(),
            }];
            let scrubbed = redact_secrets(&combined);
            process_turn(
                &client, &cli, &mut history, &scrubbed, &workspace_str,
                cli.max_iterations, cli.timeout, cli.quiet
            ).await?;
            return Ok(());
        }
    }

    // ── Mode 2: Positional One-Shot Execution (kronumos "fix test") ──────────
    if let Some(ref prompt) = cli.prompt {
        let mut history = vec![Message {
            role: "system".to_string(),
            content: SYSTEM_PROMPT.to_string(),
        }];
        let scrubbed = redact_secrets(prompt);
        process_turn(
            &client, &cli, &mut history, &scrubbed, &workspace_str,
            cli.max_iterations, cli.timeout, cli.quiet
        ).await?;
        return Ok(());
    }

    // ── Mode 3: Headless Autonomous Loop (--fix or --loop) ───────────────────
    if cli.fix || cli.r#loop {
        let start_time = Instant::now();
        let status = run_autonomous_loop(
            &client, &cli, &workspace_str, &project_type,
            cli.max_iterations, cli.timeout, cli.quiet
        ).await?;
        let duration_secs = (start_time.elapsed().as_millis() as f64) / 1000.0;

        if cli.json {
            let json_val = match &status {
                FixStatus::AlreadyPassing => json!({
                    "status": "already_passing",
                    "project_type": project_type,
                    "iterations": 0,
                    "tests_passed": true,
                    "duration_seconds": duration_secs,
                }),
                FixStatus::Resolved { iterations, diff_stat } => json!({
                    "status": "resolved",
                    "project_type": project_type,
                    "iterations": iterations,
                    "diff_stat": diff_stat,
                    "tests_passed": true,
                    "duration_seconds": duration_secs,
                }),
                FixStatus::Unresolved { iterations, last_error } => json!({
                    "status": "unresolved",
                    "project_type": project_type,
                    "iterations": iterations,
                    "last_error": last_error,
                    "tests_passed": false,
                    "duration_seconds": duration_secs,
                }),
            };
            println!("{}", serde_json::to_string_pretty(&json_val)?);
        }

        match status {
            FixStatus::AlreadyPassing | FixStatus::Resolved { .. } => {
                std::process::exit(0);
            }
            FixStatus::Unresolved { .. } => {
                std::process::exit(1);
            }
        }
    }

    // ── Mode 4: Interactive REPL ─────────────────────────────────────────────
    if !cli.quiet {
        print_kronumos_hud(&workspace_str, &project_type, &cli.backend);
    }

    let mut history: Vec<Message> = vec![Message {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::<KronumosHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(KronumosHelper::new()));
    rl.bind_sequence(rustyline::KeyEvent::ctrl('c'), rustyline::Cmd::Interrupt);
    rl.bind_sequence(rustyline::KeyEvent::new('\x03', rustyline::Modifiers::NONE), rustyline::Cmd::Interrupt);
    let history_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kronumos_history");
    let _ = rl.load_history(&history_file);

    let last_ctrl_c = Arc::new(Mutex::new(None::<Instant>));
    let last_ctrl_c_bg = last_ctrl_c.clone();

    tokio::spawn(async move {
        loop {
            if tokio::signal::ctrl_c().await.is_ok() {
                let now = Instant::now();
                let mut guard = last_ctrl_c_bg.lock().unwrap();
                if let Some(prev) = *guard {
                    if now.duration_since(prev) < Duration::from_millis(2000) {
                        println!("{}", "\n✓ Session closed.".dimmed());
                        std::process::exit(0);
                    }
                }
                *guard = Some(now);
                println!("{}", "\n(Press Ctrl+C again to exit)".dimmed());
            }
        }
    });

    loop {
        let prompt_str = format!("{} ", "◈ kronumos ❯".bright_cyan().bold());
        let readline = rl.readline(&prompt_str);

        match readline {
            Ok(line) => {
                *last_ctrl_c.lock().unwrap() = None;
                let input = line.trim().to_string();
                if input.is_empty() { continue; }

                let _ = rl.add_history_entry(&input);

                match input.as_str() {
                    "/exit" | "/quit" | "exit" | "quit" => {
                        println!("{}", "✓ Session closed.".dimmed());
                        break;
                    }
                    "/clear" => {
                        history.truncate(1);
                        println!("{}", "✓ Conversation buffer cleared.".dimmed());
                        continue;
                    }
                    "/undo" => {
                        let out = TokioCommand::new("git")
                            .args(["restore", "."])
                            .current_dir(&workspace_str)
                            .output()
                            .await;
                        match out {
                            Ok(_) => println!("{}", "✓ Last uncommitted patches reverted. Working tree restored.".bright_yellow()),
                            Err(e) => eprintln!("  error reverting changes: {}", e),
                        }
                        continue;
                    }
                    "/diff" => {
                        let out = TokioCommand::new("git")
                            .arg("diff")
                            .current_dir(&workspace_str)
                            .output()
                            .await;
                        match out {
                            Ok(o) => {
                                let diff = String::from_utf8_lossy(&o.stdout);
                                if diff.trim().is_empty() {
                                    println!("  {}", "Clean working tree (no uncommitted diffs).".dimmed());
                                } else {
                                    println!("{}", "╭─ 🔍 git diff (uncommitted changes) ──────────────────────────".bright_yellow().bold());
                                    for line in diff.lines() {
                                        if line.starts_with('+') && !line.starts_with("+++") {
                                            println!("│ {}", line.bright_green());
                                        } else if line.starts_with('-') && !line.starts_with("---") {
                                            println!("│ {}", line.bright_red());
                                        } else if line.starts_with("@@") {
                                            println!("│ {}", line.bright_cyan());
                                        } else {
                                            println!("│ {}", line.dimmed());
                                        }
                                    }
                                    println!("{}", "╰──────────────────────────────────────────────────────────────".bright_yellow().bold());
                                }
                            }
                            Err(e) => eprintln!("  error running git diff: {}", e),
                        }
                        continue;
                    }
                    "/test" => {
                        let test_cmd = match project_type.as_str() {
                            p if p.contains("Rust") => "cargo test",
                            p if p.contains("Python") => "pytest",
                            p if p.contains("Node") => "npm test",
                            p if p.contains("Go") => "go test ./...",
                            _ => "make test",
                        };
                        let dummy_tool = ToolCall {
                            name: "run_command".to_string(),
                            args: json!({ "command": test_cmd }),
                        };
                        let _ = execute_tool(&dummy_tool, &workspace_str, cli.timeout, false).await;
                        continue;
                    }
                    "/" | "/?" | "/menu" | "/help" => {
                        print_command_cockpit();
                        continue;
                    }
                    "/fix" | "/loop" => {
                        let _ = run_autonomous_loop(
                            &client, &cli, &workspace_str, &project_type,
                            cli.max_iterations, cli.timeout, false
                        ).await;
                    }
                    cmd if cmd.starts_with('/') => {
                        println!("  {}", format!("Unknown command `{}`. Available commands:", cmd).bright_red());
                        print_command_cockpit();
                        continue;
                    }
                    _ => {
                        let scrubbed = redact_secrets(&input);
                        process_turn(
                            &client, &cli, &mut history,
                            &scrubbed, &workspace_str, cli.max_iterations, cli.timeout, false
                        ).await?;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                let now = Instant::now();
                let mut guard = last_ctrl_c.lock().unwrap();
                if let Some(prev) = *guard {
                    if now.duration_since(prev) < Duration::from_millis(2000) {
                        println!("{}", "\n✓ Session closed.".dimmed());
                        break;
                    }
                }
                *guard = Some(now);
                println!("{}", "\n(Press Ctrl+C again to exit)".dimmed());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "\n✓ Session closed.".dimmed());
                break;
            }
            Err(e) => {
                eprintln!("{}: {}", "readline error".bright_red(), e);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_file);
    Ok(())
}
