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

fn is_path_safe(file_path: &str) -> bool {
    if let Ok(current_dir) = std::env::current_dir() {
        let current_dir = current_dir.canonicalize().unwrap_or(current_dir);
        let path = std::path::Path::new(file_path);
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            current_dir.join(path)
        };
        // Canonicalize untuk resolve symlink
        if let Ok(resolved) = abs_path.canonicalize() {
            resolved.starts_with(&current_dir)
        } else {
            false // File tidak ada = tolak
        }
    } else {
        false
    }
}

pub async fn run_server() -> anyhow::Result<()> {
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
                                let (context, _) = extractor::extract_context(log, context_lines, true);
                                let mut combined = format!("Log:\n{}\nContext:\n{}", log, context);
                                if let Some(git_diff) = git::get_recent_changes() {
                                    combined.push_str(&format!("\n\nRecent Git Changes:\n{}", git_diff));
                                }
                                // Sensor API key, password, JWT sebelum dikirim
                                let safe_combined = redact::redact_secrets(&combined);
                                Some(success_response(id.unwrap_or(Value::Null), json!({
                                    "content": [{ "type": "text", "text": safe_combined }]
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
                                if !is_path_safe(fp) {
                                    Some(error_response(id, -32602, "Security Error: Cannot modify files outside the current working directory"))
                                } else {
                                    let result = match std::fs::read_to_string(fp) {
                                        Ok(content) => {
                                            let count = content.matches(orig).count();
                                            if count == 0 {
                                                "Error: original_code not found in the file. Make sure it matches exactly.".to_string()
                                            } else if count > 1 {
                                                format!("Error: original_code found {} times. Please provide a more specific code block.", count)
                                            } else {
                                                let updated = content.replacen(orig, new_c, 1);
                                                match std::fs::write(fp, updated) {
                                                    Ok(_) => "Patch applied successfully.".to_string(),
                                                    Err(e) => format!("Failed to write file: {}", e)
                                                }
                                            }
                                        }
                                        Err(e) => format!("Failed to read file: {}", e)
                                    };
                                    Some(success_response(id.unwrap_or(Value::Null), json!({
                                        "content": [{ "type": "text", "text": result }]
                                    })))
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
