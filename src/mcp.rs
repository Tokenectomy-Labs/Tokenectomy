use crate::extractor;
use crate::git;
use crate::redact;
use crate::search;
use serde_json::{json, Value};
use std::io::{self, BufRead, Read, Write};

fn error_response(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

use crate::workspace::WorkspaceBoundary;

pub fn verify_patch(path: &std::path::Path) -> Result<(), String> {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "rs" => {
                if let Ok(output) = std::process::Command::new("cargo")
                    .args(["check", "--quiet", "--message-format=short"])
                    .output()
                {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err_msg = if !stderr.trim().is_empty() {
                            stderr.trim()
                        } else {
                            stdout.trim()
                        };
                        return Err(format!("Cargo check failed: {}", err_msg));
                    }
                }
            }
            "py" => {
                if let Ok(output) = std::process::Command::new("python3")
                    .args(["-m", "py_compile", path.to_str().unwrap_or("")])
                    .output()
                {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("Python syntax check failed: {}", stderr.trim()));
                    }
                }
            }
            "js" | "mjs" | "cjs" => {
                if let Ok(output) = std::process::Command::new("node")
                    .args(["--check", path.to_str().unwrap_or("")])
                    .output()
                {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("Node syntax check failed: {}", stderr.trim()));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn run_server() -> anyhow::Result<()> {
    let boundary = WorkspaceBoundary::current().unwrap_or_else(|_| {
        WorkspaceBoundary::new(std::env::current_dir().unwrap_or_default())
            .expect("workspace boundary initialization")
    });
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut handle = stdin.lock();
    loop {
        let mut line = String::new();
        // VULN-07: Limit read to 50MB to prevent MCP DoS
        let n = handle.by_ref().take(50 * 1024 * 1024).read_line(&mut line)?;
        if n == 0 {
            break; // EOF
        }
        
        if line.trim().is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                let err = error_response(None, -32700, "Parse error or line too long");
                writeln!(stdout, "{}", serde_json::to_string(&err)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = req.get("id").cloned();
        let method = match req.get("method").and_then(|v| v.as_str()) {
            Some(m) => m,
            None => {
                let err = error_response(id, -32600, "Invalid Request: missing method");
                writeln!(stdout, "{}", serde_json::to_string(&err)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = match method {
            "initialize" => {
                Some(success_response(id.unwrap_or(Value::Null), json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "tokenectomy",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })))
            }
            "notifications/initialized" => None,
            "ping" => Some(success_response(id.unwrap_or(Value::Null), json!({}))),
            "tools/list" => {
                Some(success_response(id.unwrap_or(Value::Null), json!({
                    "tools": [
                        {
                            "name": "get_error_context",
                            "description": "Extracts detailed source code context and git diff from an error log",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "log": { "type": "string" },
                                    "context_lines": { "type": "integer" }
                                },
                                "required": ["log"]
                            }
                        },
                        {
                            "name": "search_stack_overflow",
                            "description": "Search Stack Overflow for a specific error query",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string" }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "apply_code_patch",
                            "description": "Applies a code patch to a specific file by replacing original_code with new_code.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "file_path": { "type": "string", "description": "Absolute path to the file" },
                                    "original_code": { "type": "string", "description": "The exact code block to be replaced" },
                                    "new_code": { "type": "string", "description": "The new code block" }
                                },
                                "required": ["file_path", "original_code", "new_code"]
                            }
                        }
                    ]
                })))
            }
            "tools/call" => {
                let params = req.get("params");
                let name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str());
                let args = params.and_then(|p| p.get("arguments"));

                if name.is_none() || args.is_none() {
                    Some(error_response(id, -32602, "Invalid params"))
                } else {
                    let name = name.unwrap();
                    let args = args.unwrap();

                    match name {
                        "get_error_context" => {
                            if let Some(log) = args.get("log").and_then(|l| l.as_str()) {
                                let context_lines = args.get("context_lines").and_then(|c| c.as_u64()).unwrap_or(10) as usize;
                                // P1: Redact secrets BEFORE extracting context so raw credentials never enter AST or memory
                                let safe_log = redact::redact_secrets(log);
                                let (context, _) = extractor::extract_context_with_boundary(&safe_log, context_lines, Some(&boundary));
                                let mut combined = format!("Log:\n{}\nContext:\n{}", safe_log, context);
                                if let Some(git_diff) = git::get_recent_changes() {
                                    let safe_diff = redact::redact_secrets(&git_diff);
                                    combined.push_str(&format!("\n\nRecent Git Changes:\n{}", safe_diff));
                                }
                                Some(success_response(id.unwrap_or(Value::Null), json!({
                                    "content": [{ "type": "text", "text": combined }]
                                })))
                            } else {
                                Some(error_response(id, -32602, "Missing 'log' argument"))
                            }
                        }
                        "search_stack_overflow" => {
                            if let Some(query) = args.get("query").and_then(|q| q.as_str()) {
                                let results = search::search_stackoverflow(query).await;
                                let text = results.unwrap_or_else(|| "No results found".to_string());
                                Some(success_response(id.unwrap_or(Value::Null), json!({
                                    "content": [{ "type": "text", "text": text }]
                                })))
                            } else {
                                Some(error_response(id, -32602, "Missing 'query' argument"))
                            }
                        }
                        "apply_code_patch" => {
                            let file_path = args.get("file_path").and_then(|f| f.as_str());
                            let original = args.get("original_code").and_then(|o| o.as_str());
                            let new_code = args.get("new_code").and_then(|n| n.as_str());

                            if let (Some(fp), Some(orig), Some(new_c)) = (file_path, original, new_code) {
                                match boundary.resolve(fp) {
                                    Err(e) => {
                                        Some(error_response(id, -32602, &format!("Security Error: {}", e)))
                                    }
                                    Ok(safe_path) => {
                                        let result = match boundary.read(&safe_path) {
                                            Ok(content) => {
                                                let count = content.matches(orig).count();
                                                if count == 0 {
                                                    "Error: original_code not found in the file. Make sure it matches exactly.".to_string()
                                                } else if count > 1 {
                                                    format!("Error: original_code found {} times. Please provide a more specific code block.", count)
                                                } else {
                                                    let backup = content.clone();
                                                    let updated = content.replacen(orig, new_c, 1);
                                                    match boundary.write(&safe_path, updated.as_bytes()) {
                                                        Ok(_) => {
                                                            // P0-2: Verify patch and rollback on failure (0 dirty diff)
                                                            match verify_patch(&safe_path) {
                                                                Ok(_) => "Patch applied and verified successfully.".to_string(),
                                                                Err(verify_err) => {
                                                                    // Automatic rollback!
                                                                    let _ = boundary.write(&safe_path, backup.as_bytes());
                                                                    format!("Patch rejected and automatically rolled back: {}", verify_err)
                                                                }
                                                            }
                                                        }
                                                        Err(e) => format!("Failed to write file: {}", e),
                                                    }
                                                }
                                            }
                                            Err(e) => format!("Failed to read file: {}", e),
                                        };
                                        Some(success_response(id.unwrap_or(Value::Null), json!({
                                            "content": [{ "type": "text", "text": result }]
                                        })))
                                    }
                                }
                            } else {
                                Some(error_response(id, -32602, "Missing required arguments for apply_code_patch"))
                            }
                        }
                        _ => {
                            Some(error_response(id, -32601, &format!("Tool '{}' not found", name)))
                        }
                    }
                }
            }
            _ => {
                if id.is_some() {
                    Some(error_response(id, -32601, "Method not found"))
                } else {
                    None
                }
            }
        };

        if let Some(res) = response {
            writeln!(stdout, "{}", serde_json::to_string(&res)?)?;
            stdout.flush()?;
        }
    }
    Ok(())
}
