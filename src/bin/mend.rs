/// Mend — Autonomous Bug Remediation & Self-Healing CLI Agent
///
/// A full agentic REPL (like Claude Code) powered by Mend Core (Qwen 2.5
/// Coder 7B fine-tuned) and the Tokenectomy Rust Sub-Cortex.
///
/// Architecture:
///   User Input → Sub-Cortex (scrub + redact) → Mend Core (stream inference)
///   → Tool Call Parser → Tool Executor → Feed back → Loop until done
///
/// Backends:
///   1. Cloudflare Workers AI (default, streaming SSE)
///   2. Ollama local    (--backend ollama)
///   3. OpenAI-compat   (--backend openai, needs OPENAI_API_KEY or custom URL)
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

// ── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "mend",
    about = "⚡ Mend — Autonomous Bug Remediation & Self-Healing Agent",
    version = "1.0.0"
)]
struct Cli {
    /// Backend to use for inference
    #[arg(long, default_value = "cloudflare", value_parser = ["cloudflare", "ollama", "openai"])]
    backend: String,

    /// Cloudflare Worker endpoint URL (or set MEND_CF_URL env var)
    #[arg(long, env = "MEND_CF_URL")]
    cf_url: Option<String>,

    /// Ollama model name (default: mend or hf.co/NadevA23/Mend-GGUF:Q4_K_M)
    #[arg(long, default_value = "mend")]
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

    /// Auto-run: immediately run test suite and enter fix loop (non-interactive)
    #[arg(long)]
    fix: bool,

    /// Max autonomous tool-call loop iterations before pausing for confirmation
    #[arg(long, default_value = "8")]
    max_iterations: usize,

    /// Workspace directory (default: current dir)
    #[arg(long, default_value = ".")]
    workspace: String,
}

// ── Message types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

// ── Tool call extracted from model output ────────────────────────────────────

#[derive(Debug)]
struct ToolCall {
    name: String,
    args: Value,
}

// ── System prompt ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = r#"You are Mend, an autonomous software engineering and SRE agent natively equipped with the Tokenectomy M2M Sub-Cortex.

Your mission: Diagnose, patch, and close bugs with compiler certainty and zero hallucination.

Available tools — always respond with a valid JSON object when taking action:

1. run_command
   {"name": "run_command", "arguments": {"command": "<shell command>"}}
   Use to run tests (npm test, cargo test, pytest), build commands, or read git status.

2. view_file
   {"name": "view_file", "arguments": {"path": "<relative file path>", "start_line": <int>, "end_line": <int>}}
   Use to read a specific section of a source file before patching.

3. apply_patch
   {"name": "apply_patch", "arguments": {"path": "<relative file path>", "original": "<exact lines to replace>", "replacement": "<replacement lines>"}}
   Use ONLY for surgical, character-exact replacements. Never rewrite entire files.

4. git_action
   {"name": "git_action", "arguments": {"action": "branch|commit|push", "message": "<commit message>", "branch": "<branch name>"}}
   Use to create fix branches and commit verified patches.

Rules:
- ALWAYS call run_command first to observe the actual failing test output.
- NEVER guess: read the file with view_file before applying any patch.
- ONLY touch the exact broken lines. Zero dirty diffs.
- After patching, ALWAYS re-run tests to verify.
- If tests pass: call git_action to commit the fix.
- If tests still fail after 3 attempts: stop and explain what you found."#;

// ── Tool executor ────────────────────────────────────────────────────────────

fn execute_tool(tool: &ToolCall, workspace: &str) -> String {
    match tool.name.as_str() {
        "run_command" => {
            let cmd = tool.args["command"].as_str().unwrap_or("echo 'no command'");
            println!(
                "  {} {}",
                "▶ run_command:".cyan().bold(),
                cmd.white()
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
                    // Sub-Cortex: scrub secrets + truncate noise
                    let scrubbed = redact_secrets(&combined);
                    let lines: Vec<&str> = scrubbed.lines().collect();
                    // Keep only last 60 lines to prevent context bloat
                    let truncated = if lines.len() > 60 {
                        format!(
                            "[...{} lines scrubbed by Sub-Cortex...]\n{}",
                            lines.len() - 60,
                            lines[lines.len() - 60..].join("\n")
                        )
                    } else {
                        scrubbed.clone()
                    };
                    let exit_code = o.status.code().unwrap_or(-1);
                    format!("exit_code: {}\n{}", exit_code, truncated)
                }
                Err(e) => format!("error: {}", e),
            }
        }

        "view_file" => {
            let path = tool.args["path"].as_str().unwrap_or("");
            let start = tool.args["start_line"].as_u64().unwrap_or(1) as usize;
            let end = tool.args["end_line"].as_u64().unwrap_or(0) as usize;
            println!(
                "  {} {} (lines {}-{})",
                "📄 view_file:".cyan().bold(),
                path.white(),
                start,
                if end == 0 { "end".to_string() } else { end.to_string() }
            );
            let full_path = PathBuf::from(workspace).join(path);
            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    let from = start.saturating_sub(1);
                    let to = if end == 0 || end > lines.len() { lines.len() } else { end };
                    lines[from..to]
                        .iter()
                        .enumerate()
                        .map(|(i, l)| format!("{:4} | {}", from + i + 1, l))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Err(e) => format!("error reading {}: {}", path, e),
            }
        }

        "apply_patch" => {
            let path = tool.args["path"].as_str().unwrap_or("");
            let original = tool.args["original"].as_str().unwrap_or("");
            let replacement = tool.args["replacement"].as_str().unwrap_or("");
            println!(
                "  {} {}",
                "🩹 apply_patch:".yellow().bold(),
                path.white()
            );
            let full_path = PathBuf::from(workspace).join(path);
            match fs::read_to_string(&full_path) {
                Ok(content) => {
                    if !content.contains(original) {
                        return format!(
                            "error: Target content not found in {}. Check exact whitespace and indentation.",
                            path
                        );
                    }
                    // Only replace first occurrence (surgical atomic patch)
                    let new_content = content.replacen(original, replacement, 1);
                    match fs::write(&full_path, &new_content) {
                        Ok(_) => {
                            let added = replacement.lines().count();
                            let removed = original.lines().count();
                            format!(
                                "patch applied: {} (+{} lines, -{} lines)",
                                path, added, removed
                            )
                        }
                        Err(e) => format!("error writing {}: {}", path, e),
                    }
                }
                Err(e) => format!("error reading {}: {}", path, e),
            }
        }

        "git_action" => {
            let action = tool.args["action"].as_str().unwrap_or("status");
            let message = tool.args["message"].as_str().unwrap_or("mend: apply fix");
            let branch = tool.args["branch"].as_str().unwrap_or("mend/fix");
            println!(
                "  {} {} ({})",
                "🚀 git_action:".green().bold(),
                action.white(),
                branch.dimmed()
            );
            let cmd = match action {
                "branch" => format!("git checkout -b {}", branch),
                "commit" => format!("git add -A && git commit -m \"{}\"", message),
                "push" => format!("git push -u origin {}", branch),
                _ => "git status".to_string(),
            };
            let output = Command::new("sh")
                .arg("-c")
                .arg(&cmd)
                .current_dir(workspace)
                .output();
            match output {
                Ok(o) => {
                    let out = String::from_utf8_lossy(&o.stdout);
                    let err = String::from_utf8_lossy(&o.stderr);
                    format!("{}{}", out, err).trim().to_string()
                }
                Err(e) => format!("error: {}", e),
            }
        }

        unknown => format!("unknown tool: {}", unknown),
    }
}

// ── Tool call parser ─────────────────────────────────────────────────────────

fn extract_tool_call(text: &str) -> Option<ToolCall> {
    // Match first complete JSON object in the response
    let re = regex::Regex::new(r#"\{[^{}]*"name"\s*:[^{}]*"arguments"\s*:\s*\{[^{}]*\}[^{}]*\}"#).ok()?;
    if let Some(m) = re.find(text) {
        if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
            let name = v["name"].as_str()?.to_string();
            let args = v["arguments"].clone();
            return Some(ToolCall { name, args });
        }
    }
    // Fallback: find any JSON object that has a "name" key
    let re2 = regex::Regex::new(r#"\{[^{}]+\}"#).ok()?;
    for m in re2.find_iter(text) {
        if let Ok(v) = serde_json::from_str::<Value>(m.as_str()) {
            if let Some(name) = v["name"].as_str() {
                let args = v["arguments"].clone();

                return Some(ToolCall { name: name.to_string(), args });
            }
        }
    }
    None
}

// ── Inference backends ───────────────────────────────────────────────────────

async fn stream_cloudflare(
    client: &Client,
    url: &str,
    messages: &[Message],
) -> Result<String> {
    let payload = json!({
        "messages": messages,
        "stream": true,
        "max_tokens": 512,
        "temperature": 0.2,
    });

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Cloudflare Worker")?;

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    print!("{}", "\nMend: ".bright_green().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Stream read error")?;
        let raw = String::from_utf8_lossy(&chunk);

        // SSE parsing: lines starting with "data: "
        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    break;
                }
                if let Ok(v) = serde_json::from_str::<Value>(data) {
                    // Cloudflare Workers AI SSE format
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
        "options": {"temperature": 0.2, "num_predict": 512}
    });

    let response = client
        .post(format!("{}/api/chat", host))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama — is it running?")?;

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    print!("{}", "\nMend: ".bright_green().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Ollama stream error")?;
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
        "max_tokens": 512,
        "temperature": 0.2,
    });

    let response = client
        .post(format!("{}/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to OpenAI-compatible API")?;

    let mut full_text = String::new();
    let mut stream = response.bytes_stream();

    print!("{}", "\nMend: ".bright_green().bold());
    io::stdout().flush()?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("OpenAI stream error")?;
        let raw = String::from_utf8_lossy(&chunk);
        for line in raw.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" { break; }
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

// ── Main REPL loop ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace = cli.workspace.clone();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    // Canonical workspace path
    let workspace_abs = fs::canonicalize(&workspace)
        .unwrap_or_else(|_| PathBuf::from(&workspace));
    let workspace_str = workspace_abs.to_string_lossy().to_string();

    // Detect project type
    let project_type = detect_project_type(&workspace_str);

    // ── Banner ───────────────────────────────────────────────────────────────
    println!();
    println!("{}", "╭──────────────────────────────────────────────────────────╮".bright_black());
    println!("{}  {}  {}",
        "│".bright_black(),
        "⚡ Mend v1.0 — Autonomous Bug Remediation Agent".bright_white().bold(),
        "│".bright_black()
    );
    println!("{}  Workspace : {}  {}",
        "│".bright_black(),
        workspace_str.cyan(),
        "│".bright_black()
    );
    println!("{}  Project   : {}  {}",
        "│".bright_black(),
        project_type.yellow(),
        "│".bright_black()
    );
    println!("{}  Backend   : {}  {}",
        "│".bright_black(),
        cli.backend.green(),
        "│".bright_black()
    );
    println!("{}  Sub-Cortex: {}  {}",
        "│".bright_black(),
        "ACTIVE (zero-leak redaction ON)".bright_green(),
        "│".bright_black()
    );
    println!("{}", "╰──────────────────────────────────────────────────────────╯".bright_black());
    println!();
    println!("{}", "Commands:".dimmed());
    println!("  {}   chat freely — ask about code, explain errors", "Any text".white());
    println!("  {}   autonomous fix loop — detect failures & heal", "/fix".bright_yellow().bold());
    println!("  {}   clear conversation history", "/clear".white());
    println!("  {}   exit Mend", "/exit".white());
    println!();

    // ── Conversation history ─────────────────────────────────────────────────
    let mut history: Vec<Message> = vec![Message {
        role: "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    }];

    // ── If --fix flag, immediately trigger fix loop ───────────────────────────
    let mut rl = DefaultEditor::new()?;

    // Load history file from ~/.mend_history
    let history_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".mend_history");
    let _ = rl.load_history(&history_file);

    if cli.fix {
        let fix_prompt = format!(
            "Run the project tests for a {} project in workspace '{}', find all failures, and autonomously heal them.",
            project_type, workspace_str
        );
        process_turn(
            &client, &cli, &mut history,
            &fix_prompt, &workspace_str, cli.max_iterations
        ).await?;
    }

    // ── Main REPL loop ────────────────────────────────────────────────────────
    loop {
        let readline = rl.readline(&format!("{} ", "mend >".bright_cyan().bold()));
        match readline {
            Ok(line) => {
                let input = line.trim().to_string();
                if input.is_empty() { continue; }

                let _ = rl.add_history_entry(&input);

                match input.as_str() {
                    "/exit" | "/quit" | "exit" | "quit" => {
                        println!("{}", "Goodbye! Mend signing off.".bright_green());
                        break;
                    }
                    "/clear" => {
                        history.truncate(1); // Keep system prompt
                        println!("{}", "✓ Conversation history cleared.".dimmed());
                        continue;
                    }
                    "/fix" => {
                        let fix_prompt = format!(
                            "Run the project tests for a {} project, find all failures, and autonomously heal them.",
                            project_type
                        );
                        process_turn(
                            &client, &cli, &mut history,
                            &fix_prompt, &workspace_str, cli.max_iterations
                        ).await?;
                    }
                    _ => {
                        // Sub-Cortex: redact any secrets user accidentally pastes
                        let scrubbed_input = redact_secrets(&input);
                        process_turn(
                            &client, &cli, &mut history,
                            &scrubbed_input, &workspace_str, cli.max_iterations
                        ).await?;
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("{}", "\nMend signing off.".bright_green());
                break;
            }
            Err(e) => {
                eprintln!("{}: {}", "readline error".red(), e);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_file);
    Ok(())
}

// ── Agentic turn: send to model, parse tool calls, execute, loop ─────────────

async fn process_turn(
    client: &Client,
    cli: &Cli,
    history: &mut Vec<Message>,
    user_input: &str,
    workspace: &str,
    max_iter: usize,
) -> Result<()> {
    // Add user message to history
    history.push(Message {
        role: "user".to_string(),
        content: user_input.to_string(),
    });

    let mut iterations = 0;

    loop {
        if iterations >= max_iter {
            println!(
                "\n{}",
                format!("⚠ Reached {} iterations. Pausing for your review.", max_iter)
                    .yellow().bold()
            );
            break;
        }
        iterations += 1;

        // Stream inference
        let response_text = match cli.backend.as_str() {
            "cloudflare" => {
                let url = cli.cf_url.as_deref().unwrap_or("https://mend-gateway.your-worker.workers.dev");
                stream_cloudflare(client, url, history).await?
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

        // Add assistant response to history
        history.push(Message {
            role: "assistant".to_string(),
            content: response_text.clone(),
        });

        // Try to extract and execute a tool call
        if let Some(tool_call) = extract_tool_call(&response_text) {
            println!(
                "\n{}",
                format!("╭─ Mend Tool: {} ─────────────────────────", tool_call.name)
                    .bright_black()
            );

            let tool_result = execute_tool(&tool_call, workspace);

            println!(
                "{}",
                format!("╰─ Result ──────────────────────────────────")
                    .bright_black()
            );
            // Print first 20 lines of tool result
            for line in tool_result.lines().take(20) {
                println!("  {}", line.dimmed());
            }
            if tool_result.lines().count() > 20 {
                println!("  {}", format!("[...{} more lines...]", tool_result.lines().count() - 20).dimmed());
            }
            println!();

            // Feed tool result back into conversation
            history.push(Message {
                role: "user".to_string(),
                content: format!(
                    "Tool result for {}:\n{}",
                    tool_call.name, tool_result
                ),
            });

            // Check for explicit completion signals
            let done_signals = ["all tests passed", "tests passed", "PASSED", "fix complete",
                               "patch applied successfully", "pull request created"];
            if done_signals.iter().any(|s| tool_result.to_lowercase().contains(&s.to_lowercase())) {
                // One more turn to let the model summarize
                continue;
            }
        } else {
            // No tool call = model gave a text answer or declared done
            break;
        }
    }
    Ok(())
}

// ── Project type detector ────────────────────────────────────────────────────

fn detect_project_type(workspace: &str) -> String {
    let indicators = [
        ("package.json", "Node.js / TypeScript"),
        ("Cargo.toml",   "Rust"),
        ("pyproject.toml", "Python"),
        ("setup.py",     "Python"),
        ("go.mod",       "Go"),
        ("pom.xml",      "Java / Maven"),
        ("build.gradle", "Java / Gradle"),
    ];
    for (file, label) in &indicators {
        if PathBuf::from(workspace).join(file).exists() {
            return label.to_string();
        }
    }
    "Unknown".to_string()
}
