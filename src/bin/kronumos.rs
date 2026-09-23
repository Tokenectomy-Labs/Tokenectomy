/// Kronumos — Autonomous Bug Remediation & Self-Healing CLI Agent
///
/// A fast, transparent-friendly agentic REPL powered by Kronumos Core
/// and the Tokenectomy Rust Sub-Cortex.
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
    io::{self, Write},
    path::PathBuf,
    process::Command,
};
use tokenectomy::redact_secrets;

// ── CLI Configuration ────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "kronumos",
    about = "Kronumos Kairos",
    version = "1.0.0"
)]
struct Cli {
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

    /// Max autonomous tool-call iterations before pausing for user review
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

// ── Tool Executor ────────────────────────────────────────────────────────────

fn execute_tool(tool: &ToolCall, workspace: &str) -> String {
    match tool.name.as_str() {
        "run_command" => {
            let cmd = tool.args["command"].as_str().unwrap_or("echo 'no command provided'");
            
            // Security check: block destructive commands
            if let Err(err) = is_command_safe(cmd) {
                eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), err);
                return err;
            }

            println!(
                "  {} {}",
                "▶ run_command:".cyan().bold(),
                cmd.bright_white()
            );
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(workspace)
                .output();
            match output {
                Ok(o) => {
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
                Err(e) => format!("error executing command: {}", e),
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
                    eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    return e;
                }
            };

            println!(
                "  {} {} (lines {}-{})",
                "📄 view_file:".cyan().bold(),
                path.bright_white(),
                start,
                if end == 0 { "end".to_string() } else { end.to_string() }
            );

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    // Sub-Cortex: redact secrets in viewed file before returning
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
                    eprintln!("  {} {}", "🛡️ Security Block:".bright_red().bold(), e);
                    return e;
                }
            };

            println!(
                "  {} {}",
                "🩹 apply_patch:".bright_yellow().bold(),
                path.bright_white()
            );

            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    if !content.contains(original) {
                        return format!(
                            "error: Target content not found in {}. Ensure exact indentation and matching context.",
                            path
                        );
                    }
                    // Surgical atomic replacement of the first matching instance
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

            println!(
                "  {} {} ({})",
                "🚀 git_action:".bright_green().bold(),
                action.bright_white(),
                branch.dimmed()
            );

            // Immunity against shell injection: use direct Command args without sh -c
            let output = match action {
                "branch" => {
                    Command::new("git")
                        .args(["checkout", "-b", branch])
                        .current_dir(workspace)
                        .output()
                }
                "commit" => {
                    let _ = Command::new("git")
                        .args(["add", "-A"])
                        .current_dir(workspace)
                        .output();
                    Command::new("git")
                        .args(["commit", "-m", message])
                        .current_dir(workspace)
                        .output()
                }
                "diff" => {
                    Command::new("git")
                        .args(["diff"])
                        .current_dir(workspace)
                        .output()
                }
                "push" => {
                    Command::new("git")
                        .args(["push", "-u", "origin", branch])
                        .current_dir(workspace)
                        .output()
                }
                _ => {
                    Command::new("git")
                        .args(["status", "--short"])
                        .current_dir(workspace)
                        .output()
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
    // 1. Look for structured JSON block with "name" and "arguments"
    if let Ok(re) = regex::Regex::new(r#"\{[^{}]*"name"\s*:[^{}]*"arguments"\s*:\s*\{[^{}]*\}[^{}]*\}"#) {
        if let Some(m) = re.find(text) {
            if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
                let name = v["name"].as_str()?.to_string();
                let args = v["arguments"].clone();
                return Some(ToolCall { name, args });
            }
        }
    }

    // 2. Fallback: inspect any JSON-like object containing "name"
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

    print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);

        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    break;
                }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let token = v["response"]
                        .as_str()
                        .or_else(|| v["choices"][0]["delta"]["content"].as_str())
                        .unwrap_or("");
                    if !token.is_empty() {
                        print!("{}", token);
                        io::stdout().flush()?;
                        full_text.push_str(token);
                    }
                }
            }
        }
    }
    println!();
    Ok(full_text)
}

async fn stream_ollama(
    client: &Client,
    host: &str,
    model: &str,
    messages: &[Message],
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

    print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ollama stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                let token = v["message"]["content"].as_str().unwrap_or("");
                if !token.is_empty() {
                    print!("{}", token);
                    io::stdout().flush()?;
                    full_text.push_str(token);
                }
                if v["done"].as_bool().unwrap_or(false) {
                    break;
                }
            }
        }
    }
    println!();
    Ok(full_text)
}

async fn stream_openai_compat(
    client: &Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[Message],
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

    print!("{}", "◈ Kronumos ❯ ".bright_cyan().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("OpenAI stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" { break; }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    let token = v["choices"][0]["delta"]["content"].as_str().unwrap_or("");
                    if !token.is_empty() {
                        print!("{}", token);
                        io::stdout().flush()?;
                        full_text.push_str(token);
                    }
                }
            }
        }
    }
    println!();
    Ok(full_text)
}

// ── Agentic Turn Processing ──────────────────────────────────────────────────

async fn process_turn(
    client: &Client,
    cli: &Cli,
    history: &mut Vec<Message>,
    user_input: &str,
    workspace: &str,
    max_iter: usize,
) -> Result<()> {
    // Add user message to conversation history
    history.push(Message {
        role: "user".to_string(),
        content: user_input.to_string(),
    });

    let mut iterations = 0;

    loop {
        if iterations >= max_iter {
            println!(
                "\n{}",
                format!("⚠ Reached safety limit of {} tool iterations. Pausing for confirmation.", max_iter)
                    .bright_yellow().bold()
            );
            break;
        }
        iterations += 1;

        // Perform streaming inference across the selected backend
        let response_text = match cli.backend.as_str() {
            "cloudflare" => {
                let default_url = "https://kronumos-gateway.your-subdomain.workers.dev";
                let url = cli.cf_url.as_deref().unwrap_or(default_url);
                stream_cloudflare(client, url, cli.cf_key.as_deref(), history).await?
            }
            "ollama" => {
                stream_ollama(client, &cli.ollama_host, &cli.ollama_model, history).await?
            }
            "openai" => {
                let key = cli.openai_key.as_deref().unwrap_or("");
                stream_openai_compat(client, &cli.openai_url, key, &cli.openai_model, history).await?
            }
            _ => unreachable!(),
        };

        // Record assistant response in history
        history.push(Message {
            role: "assistant".to_string(),
            content: response_text.clone(),
        });

        // Parse tool invocation if emitted by model
        if let Some(tool_call) = extract_tool_call(&response_text) {
            println!(
                "\n{}",
                format!("╭─ ⚙️ tool: {} ─────────────────────────────────────", tool_call.name)
                    .bright_black()
            );

            let tool_result = execute_tool(&tool_call, workspace);

            println!(
                "{}",
                "╰─ 📋 tool result ────────────────────────────────────".bright_black()
            );
            
            let lines: Vec<&str> = tool_result.lines().collect();
            for line in lines.iter().take(20) {
                println!("  {}", line.dimmed());
            }
            if lines.len() > 20 {
                println!("  {}", format!("[...{} more lines...]", lines.len() - 20).dimmed());
            }
            println!();

            // Feed tool result back to agent
            history.push(Message {
                role: "user".to_string(),
                content: format!(
                    "Tool execution result for {}:\n{}",
                    tool_call.name, tool_result
                ),
            });

            // Termination heuristics: if fix was validated or tests passed, allow 1 final turn
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
            // Model returned a direct text explanation or finished
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
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let workspace_abs = fs::canonicalize(&workspace)
        .unwrap_or_else(|_| PathBuf::from(&workspace));
    let workspace_str = workspace_abs.to_string_lossy().to_string();
    let project_type = detect_project_type(&workspace_str);

    // ── Minimalist Transparent-Friendly Greeting ───────────────────────────
    println!();
    println!(
        "{}  {}",
        "╦╔═╦═╗╔═╗╔╗╔╦ ╦╔╦╗╔═╗╔═╗".bright_cyan().bold(),
        "Kronumos Kairos".bright_white().bold()
    );
    println!(
        "{}  {}",
        "╠╩╗╠╦╝║ ║║║║║ ║║║║║ ║╚═╗".bright_cyan().bold(),
        "v1.0".dimmed()
    );
    println!(
        "{}  {}",
        "╩ ╩╩╚═╚═╝╝╚╝╚═╝╩ ╩╚═╝╚═╝".bright_cyan().bold(),
        "Type /help for help, or ask anything to start.".dimmed()
    );
    println!();



    let mut history: Vec<Message> = vec![Message {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    let mut rl = DefaultEditor::new()?;
    let history_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".kronumos_history");
    let _ = rl.load_history(&history_file);

    // If --fix was specified on command line, immediately start autonomous fix
    if cli.fix {
        let fix_prompt = format!(
            "Run project tests for {} in '{}', diagnose any failures, and synthesize a surgical patch.",
            project_type, workspace_str
        );
        process_turn(&client, &cli, &mut history, &fix_prompt, &workspace_str, cli.max_iterations).await?;
    }

    // ── Interactive REPL Loop ────────────────────────────────────────────────
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
                        history.truncate(1); // Retain system prompt
                        println!("{}", "✓ Conversation buffer cleared.".dimmed());
                        continue;
                    }
                    "/diff" => {
                        let out = Command::new("git")
                            .arg("diff")
                            .current_dir(&workspace_str)
                            .output();
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
                        let result = execute_tool(&dummy_tool, &workspace_str);
                        println!("  {}", result.dimmed());
                        continue;
                    }
                    "/help" => {
                        println!("{}", "Commands:".bright_white().bold());
                        println!("  {}        Run test-and-repair loop", "/fix".bright_yellow());
                        println!("  {}       Show uncommitted git diff", "/diff".bright_cyan());
                        println!("  {}       Run detected project test runner", "/test".bright_green());
                        println!("  {}      Clear conversation context", "/clear".dimmed());
                        println!("  {}       Show this help message", "/help".dimmed());
                        println!("  {}       Exit session", "/exit".dimmed());
                        continue;
                    }
                    "/fix" => {
                        let fix_prompt = format!(
                            "Run the test suite for this {} workspace, diagnose any failing tests, view the source files, apply a minimal surgical patch, and verify tests pass.",
                            project_type
                        );
                        process_turn(
                            &client, &cli, &mut history,
                            &fix_prompt, &workspace_str, cli.max_iterations
                        ).await?;
                    }
                    _ => {
                        // Sub-Cortex: scrub credentials accidentally pasted into terminal
                        let scrubbed = redact_secrets(&input);
                        process_turn(
                            &client, &cli, &mut history,
                            &scrubbed, &workspace_str, cli.max_iterations
                        ).await?;
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
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
