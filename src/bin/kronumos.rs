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
use reqwest::Client;
use rustyline::{DefaultEditor, error::ReadlineError};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, IsTerminal, Read, Write},
    path::PathBuf,
    time::Duration,
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

3. apply_patch
   {"name": "apply_patch", "arguments": {"path": "<relative file path>", "original": "<exact lines to replace>", "replacement": "<new lines>"}}
   Performs a surgical, character-exact replacement in the target file.

4. git_action
   {"name": "git_action", "arguments": {"action": "branch|commit|diff|status", "message": "<commit message>", "branch": "<branch name>"}}
   Manages git branches and commits verified changes.

Guidelines:
- Always run tests or view files first to gather grounded facts before proposing changes.
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
                    "  {} {}",
                    "▶ run_command:".cyan().bold(),
                    cmd.bright_white()
                );
            }

            let mut tokio_cmd = TokioCommand::new("sh");
            tokio_cmd
                .arg("-c")
                .arg(cmd)
                .current_dir(workspace);

            match timeout(Duration::from_secs(timeout_secs), tokio_cmd.output()).await {
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
                    format!("exit_code: {}\n{}", exit_code, truncated)
                }
                Ok(Err(e)) => format!("error executing command: {}", e),
                Err(_) => {
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
                    "  {} {} (lines {}-{})",
                    "📄 view_file:".cyan().bold(),
                    path.bright_white(),
                    start,
                    if end == 0 { "end".to_string() } else { end.to_string() }
                );
            }

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    let scrubbed = redact_secrets(&content);
                    let lines: Vec<&str> = scrubbed.lines().collect();
                    let from = start.saturating_sub(1);
                    let to = if end == 0 || end > lines.len() { lines.len() } else { end };
                    lines[from..to]
                        .iter()
                        .enumerate()
                        .map(|(i, l)| format!("{:4} │ {}", from + i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Err(e) => format!("error reading file {}: {}", path, e),
            }
        }

        "apply_patch" => {
            let path = tool.args["path"].as_str().unwrap_or("");
            let original = tool.args["original"].as_str().unwrap_or("");
            let replacement = tool.args["replacement"].as_str().unwrap_or("");

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
                    "  {} {}",
                    "🩹 apply_patch:".bright_yellow().bold(),
                    path.bright_white()
                );
            }

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    if !content.contains(original) {
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
                            format!(
                                "patch applied successfully: {} (+{} lines, -{} lines)",
                                path, added, removed
                            )
                        }
                        Err(e) => format!("error writing file {}: {}", path, e),
                    }
                }
                Err(e) => format!("error reading file {}: {}", path, e),
            }
        }

        "git_action" => {
            let action = tool.args["action"].as_str().unwrap_or("status");
            let message = tool.args["message"].as_str().unwrap_or("kronumos: patch remediation");
            let branch = tool.args["branch"].as_str().unwrap_or("kronumos/fix");

            if !quiet {
                println!(
                    "  {} {} ({})",
                    "🚀 git_action:".bright_green().bold(),
                    action.bright_white(),
                    branch.dimmed()
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
                    if res.is_empty() { "ok (no output)".to_string() } else { res }
                }
                Err(e) => format!("error executing git action: {}", e),
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

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json");

    if let Some(key) = api_key {
        if !key.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key.trim()));
        }
    }

    let response = req
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Cloudflare Worker gateway")?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Cloudflare Worker error ({}): {}", status, err_text);
    }

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    if !quiet {
        print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
        io::stdout().flush()?;
    }

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Stream read error")?;
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

    let response = client
        .post(format!("{}/api/chat", host.trim_end_matches('/')))
        .json(&payload)
        .send()
        .await
        .context(format!("Failed to connect to Ollama at {} — is Ollama running?", host))?;

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    if !quiet {
        print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
        io::stdout().flush()?;
    }

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ollama stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                let token = v["message"]["content"].as_str().unwrap_or("");
                if !token.is_empty() {
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

    let mut req = client
        .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
        .header("Content-Type", "application/json");

    if !api_key.trim().is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key.trim()));
    }

    let response = req
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to OpenAI-compatible inference server")?;

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    if !quiet {
        print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
        io::stdout().flush()?;
    }

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("OpenAI stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let token = v["choices"][0]["delta"]["content"].as_str().unwrap_or("");
                    if !token.is_empty() {
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
    if !quiet {
        println!();
    }
    Ok(full_text)
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

            if !quiet {
                println!(
                    "\n{}",
                    format!("╭─ ⚙️ tool: {} ─────────────────────────────────────", tool_call.name)
                        .bright_black()
                );
            }

            let tool_result = execute_tool(&tool_call, workspace, timeout_secs, quiet).await;

            if !quiet {
                println!(
                    "{}",
                    "╰─ 📋 tool result ────────────────────────────────────".bright_black()
                );
                for line in tool_result.lines().take(15) {
                    println!("  {}", line.dimmed());
                }
                if tool_result.lines().count() > 15 {
                    println!("  {}", format!("[...{} more lines...]", tool_result.lines().count() - 15).dimmed());
                }
            }

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
            if !quiet {
                println!(
                    "\n{}",
                    format!("╭─ ⚙️ tool: {} ─────────────────────────────────────", tool_call.name)
                        .bright_black()
                );
            }

            let tool_result = execute_tool(&tool_call, workspace, timeout_secs, quiet).await;

            if !quiet {
                println!(
                    "{}",
                    "╰─ 📋 tool result ────────────────────────────────────".bright_black()
                );
                for line in tool_result.lines().take(15) {
                    println!("  {}", line.dimmed());
                }
            }

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
    let cli = Cli::parse();
    let workspace = cli.workspace.clone();
    let client = Client::builder()
        .timeout(Duration::from_secs(180))
        .build()?;

    let workspace_abs = fs::canonicalize(&workspace)
        .unwrap_or_else(|_| PathBuf::from(&workspace));
    let workspace_str = workspace_abs.to_string_lossy().to_string();
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
        let status = run_autonomous_loop(
            &client, &cli, &workspace_str, &project_type,
            cli.max_iterations, cli.timeout, cli.quiet
        ).await?;

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
        println!();
        println!("{}", "    ██╗  ██╗██████╗  ██████╗ ███╗   ██╗██╗   ██╗███╗   ███╗ ██████╗ ███████╗".bright_cyan().bold());
        println!("{}", "    ██║ ██╔╝██╔══██╗██╔═══██╗████╗  ██║██║   ██║████╗ ████║██╔═══██╗██╔════╝".bright_cyan().bold());
        println!("{}", "    █████╔╝ ██████╔╝██║   ██║██╔██╗ ██║██║   ██║██╔████╔██║██║   ██║███████╗".bright_cyan().bold());
        println!("{}", "    ██╔═██╗ ██╔══██╗██║   ██║██║╚██╗██║██║   ██║██║╚██╔╝██║██║   ██║╚════██║".bright_cyan().bold());
        println!("{}", "    ██║  ██╗██║  ██║╚██████╔╝██║ ╚████║╚██████╔╝██║ ╚═╝ ██║╚██████╔╝███████║".bright_cyan().bold());
        println!("{}", "    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ ╚═╝     ╚═╝ ╚═════╝ ╚══════╝".bright_cyan().bold());
        println!();
        println!("{}", "                     · K R O N U M O S   K A I R O S ·".cyan().bold());
        println!("{}", "               Autonomous Code Remediation & SRE Agent • v1.0".bright_white().bold());
        println!("{}", "               Type /help for help, or ask anything to start.".dimmed());
        println!();
    }

    let mut history: Vec<Message> = vec![Message {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    let mut rl = DefaultEditor::new()?;
    let history_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kronumos_history");
    let _ = rl.load_history(&history_file);

    loop {
        let prompt_str = format!("{} ", "⚡ kronumos ❯".bright_cyan().bold());
        let readline = rl.readline(&prompt_str);

        match readline {
            Ok(line) => {
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
                                    println!("{}", "  Clean working tree (no uncommitted diffs).".dimmed());
                                } else {
                                    println!("{}", diff.bright_yellow());
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
                        println!("  {} Running `{}`...", "▶".cyan().bold(), test_cmd);
                        let dummy_tool = ToolCall {
                            name: "run_command".to_string(),
                            args: json!({ "command": test_cmd }),
                        };
                        let result = execute_tool(&dummy_tool, &workspace_str, cli.timeout, false).await;
                        println!("  {}", result.dimmed());
                        continue;
                    }
                    "/help" => {
                        println!("{}", "Commands:".bright_white().bold());
                        println!("  {}        Autonomous TDD test-and-repair loop", "/fix".bright_yellow());
                        println!("  {}       Autonomous loop alias", "/loop".bright_yellow());
                        println!("  {}       Show uncommitted git diff", "/diff".bright_cyan());
                        println!("  {}       Run detected project test runner", "/test".bright_green());
                        println!("  {}      Clear conversation context", "/clear".dimmed());
                        println!("  {}       Show this help message", "/help".dimmed());
                        println!("  {}       Exit session", "/exit".dimmed());
                        continue;
                    }
                    "/fix" | "/loop" => {
                        let _ = run_autonomous_loop(
                            &client, &cli, &workspace_str, &project_type,
                            cli.max_iterations, cli.timeout, false
                        ).await;
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
                println!("{}", "\n(Session interrupted. Type /exit to quit)".dimmed());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "\n✓ Kronumos session closed.".bright_cyan());
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
