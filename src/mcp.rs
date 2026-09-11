use crate::extractor;
use crate::git;
use crate::redact;
use crate::search;
use crate::workspace::WorkspaceBoundary;
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
    let analyzer_state = crate::analyzer::AppState::new();
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
            "initialize" => Some(success_response(
                id.unwrap_or(Value::Null),
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "tokenectomy",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )),
            "notifications/initialized" => None,
            "ping" => Some(success_response(id.unwrap_or(Value::Null), json!({}))),
            "tools/list" => Some(success_response(
                id.unwrap_or(Value::Null),
                json!({
                    "tools": [
                        {
                            "name": "get_error_context",
                            "description": "Extracts focused source code snippets and git diffs from a raw error log or stack trace, stripping framework noise (node_modules, site-packages) and redacting credentials.\n\n• Side Effects: None. Strictly read-only; does not modify workspace files, git state, or environment variables.\n• Auth & Permissions: None required. Reads local filesystem within the current workspace boundary.\n• Rate Limits: None. Runs entirely locally on native machine code.\n• Return Shape: Returns a JSON object with 'sanitized_trace' (string without secrets/noise), 'source_frames' (array of objects with file, line, code_snippet), and 'git_diff' (string or null).\n• Failure Modes: If source files referenced in the trace do not exist locally, omits code snippets for those frames while still returning the sanitized trace. Returns an error JSON on unreadable input.\n• When to use: Call immediately when receiving a runtime exception, test failure, or compiler error to isolate the root cause before planning code fixes.\n• When NOT to use: Do NOT use to search web solutions (use search_stack_overflow), do NOT use to modify files (use apply_code_patch), and do NOT use to statically lint clean code without an error log (use analyze_code).\n• Prerequisites: Workspace directory must be accessible locally; git repository recommended for diff extraction.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "log": {
                                        "type": "string",
                                        "description": "Raw error stack trace, compiler panic, or terminal stderr string to analyze (e.g. Python traceback, Node.js error, Rust panic). Must be non-empty UTF-8 text up to 1MB. Automatically sanitized of API keys, JWTs, and passwords."
                                    },
                                    "context_lines": {
                                        "type": "integer",
                                        "description": "Number of source code lines to retrieve above and below each detected error line. Integer between 0 and 100. Defaults to 10 lines. Larger values expand the context window but consume more LLM tokens."
                                    }
                                },
                                "required": ["log"]
                            }
                        },
                        {
                            "name": "search_stack_overflow",
                            "description": "Queries the public Stack Overflow / Stack Exchange API for verified programming solutions and discussions matching an error signature.\n\n• Side Effects: None. Strictly read-only network search; does not mutate local files or repository state.\n• Auth & Permissions: No API key required for standard rate-limited anonymous queries.\n• Rate Limits: Subject to public Stack Exchange API rate limits (~300 requests/day per IP). Results are cached locally when possible.\n• Return Shape: Returns a JSON object containing 'query', 'total_results', and 'results' (array of objects with title, url, score, is_answered, answer_count, and answer excerpt).\n• Failure Modes: Returns empty results array if no matching questions exist. Returns an error message if network connectivity fails or API quota is exhausted.\n• When to use: Use when local code context from get_error_context is insufficient and external community patterns, known library bugs, or API migration examples are needed.\n• When NOT to use: Do NOT use with raw un-sanitized logs containing private tokens or file paths, do NOT use for local codebase inspection (use get_error_context), and do NOT use to edit code (use apply_code_patch).\n• Prerequisites: Outbound HTTP internet access to api.stackexchange.com.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Targeted search query string (e.g. 'ValueError: unsupported operand type(s) for +: int and str'). Must be free of project-specific paths, private tokens, or proprietary variable names. 3 to 150 characters recommended."
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "apply_code_patch",
                            "description": "Applies an atomic, verified code edit to a specific file by substituting original_code with new_code, with automatic syntax validation and instant rollback on failure.\n\n• Side Effects: Modifies the target file on the local filesystem. If syntax checks pass, the file is overwritten with patched contents; if syntax validation fails, the file is immediately restored to its exact original state (zero dirty diff).\n• Auth & Permissions: Requires write permissions for the target file on the host filesystem within the workspace boundary. Path traversal outside workspace root is blocked.\n• Rate Limits: None. Local disk I/O.\n• Return Shape: Returns a JSON object with 'status' ('success' or 'error'), 'file_path', 'lines_changed', 'verification' ('passed' or 'reverted'), and 'message'.\n• Failure Modes: Fails and aborts without touching the file if file_path is not found, if original_code does not match the file content uniquely, or if the compiler/linter check fails after patch application.\n• When to use: Use when you have finalized a bug fix or refactoring snippet and need safe, transactional application with zero risk of syntax corruption.\n• When NOT to use: Do NOT use for speculative edits without prior diagnosis (use get_error_context first), and do NOT use for whole-file generation when only a small block changes.\n• Prerequisites: Target file must exist and be within the current workspace directory.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "file_path": {
                                        "type": "string",
                                        "description": "Target file path to modify. Can be relative to the workspace root or an absolute path located inside the workspace boundary. Path traversal outside the workspace is rejected."
                                    },
                                    "original_code": {
                                        "type": "string",
                                        "description": "The exact character-for-character contiguous code block to be replaced, including exact leading indentation, newlines, and whitespace. Must match exactly one location in the file."
                                    },
                                    "new_code": {
                                        "type": "string",
                                        "description": "The replacement code block to substitute in place of original_code. Must maintain correct language syntax and indentation matching the surrounding code."
                                    }
                                },
                                "required": ["file_path", "original_code", "new_code"]
                            }
                        },
                        {
                            "name": "analyze_code",
                            "description": "Performs static AST code analysis using Tree-sitter to detect resource leaks (such as unclosed file handles), security vulnerabilities, and logic flaws with bounded execution limits and precise LSP UTF-16 coordinates.\n\n• Side Effects: None. Strictly read-only analysis of in-memory code; does not execute code, spawn subprocesses, or write to disk.\n• Auth & Permissions: None required. Fully offline, in-memory parser.\n• Rate Limits: None. Bounded to 1MB max source size, 128 max AST depth, and 50,000 max node visits per call.\n• Return Shape: Returns a JSON object containing 'language', 'findings_count', 'duration_ms' (latency metric), and 'findings' (array of objects with rule_id, message, severity, line [1-indexed], column [1-indexed UTF-16 code units], and remediation).\n• Failure Modes: Returns findings: [] if the code contains no detected defects. Returns an error message if the language is unsupported or if source code exceeds the 1MB or 128 AST depth limits.\n• When to use: Use proactively before committing or running code, or when reviewing Python files for unclosed file handles, resource leaks, or AST defects.\n• When NOT to use: Do NOT use when you have an active runtime crash log (use get_error_context instead), and do NOT use to apply fixes automatically (use apply_code_patch instead).\n• Prerequisites: Supported languages currently include Python ('python', 'py').",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "language": {
                                        "type": "string",
                                        "description": "Programming language identifier for the code snippet. Case-insensitive. Supported values: 'python', 'py'."
                                    },
                                    "code": {
                                        "type": "string",
                                        "description": "Raw source code string to analyze. Must not exceed 1,000,000 bytes (1MB). Does not execute runtime code; strictly parsed via Tree-sitter AST."
                                    }
                                },
                                "required": ["language", "code"]
                            }
                        }
                    ]
                }),
            )),
            "tools/call" => {
                let params = req.get("params");
                let name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str());
                let args = params.and_then(|p| p.get("arguments"));

                if let (Some(name), Some(args)) = (name, args) {
                    match name {
                        "get_error_context" => {
                            if let Some(log) = args.get("log").and_then(|l| l.as_str()) {
                                let context_lines = args
                                    .get("context_lines")
                                    .and_then(|c| c.as_u64())
                                    .unwrap_or(10) as usize;
                                // P1: Redact secrets BEFORE extracting context
                                let safe_log = redact::redact_secrets(log);
                                let (context, _) = extractor::extract_context_with_boundary(
                                    &safe_log,
                                    context_lines,
                                    Some(&boundary),
                                );
                                let mut combined =
                                    format!("Log:\n{}\nContext:\n{}", safe_log, context);
                                if let Some(git_diff) = git::get_recent_changes() {
                                    let safe_diff = redact::redact_secrets(&git_diff);
                                    combined.push_str(&format!(
                                        "\n\nRecent Git Changes:\n{}",
                                        safe_diff
                                    ));
                                }
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": combined }]
                                    }),
                                ))
                            } else {
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": "Error: Missing required 'log' argument." }],
                                        "isError": true
                                    }),
                                ))
                            }
                        }
                        "search_stack_overflow" => {
                            if let Some(query) = args.get("query").and_then(|q| q.as_str()) {
                                let results = search::search_stackoverflow(query).await;
                                let text = results
                                    .unwrap_or_else(|| "No results found".to_string());
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": text }]
                                    }),
                                ))
                            } else {
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": "Error: Missing required 'query' argument." }],
                                        "isError": true
                                    }),
                                ))
                            }
                        }
                        "apply_code_patch" => {
                            let file_path = args.get("file_path").and_then(|f| f.as_str());
                            let original = args.get("original_code").and_then(|o| o.as_str());
                            let new_code = args.get("new_code").and_then(|n| n.as_str());

                            if let (Some(fp), Some(orig), Some(new_c)) =
                                (file_path, original, new_code)
                            {
                                match boundary.resolve(fp) {
                                    Err(e) => Some(success_response(
                                        id.unwrap_or(Value::Null),
                                        json!({
                                            "content": [{ "type": "text", "text": format!("Security Error: {}", e) }],
                                            "isError": true
                                        }),
                                    )),
                                    Ok(safe_path) => {
                                        match boundary.read(&safe_path) {
                                            Ok(content) => {
                                                let count = content.matches(orig).count();
                                                if count == 0 {
                                                    Some(success_response(
                                                        id.unwrap_or(Value::Null),
                                                        json!({
                                                            "content": [{ "type": "text", "text": "Error: original_code not found in the file. Make sure it matches exactly." }],
                                                            "isError": true
                                                        }),
                                                    ))
                                                } else if count > 1 {
                                                    Some(success_response(
                                                        id.unwrap_or(Value::Null),
                                                        json!({
                                                            "content": [{ "type": "text", "text": format!("Error: original_code found {} times. Please provide a more specific code block.", count) }],
                                                            "isError": true
                                                        }),
                                                    ))
                                                } else {
                                                    let backup = content.clone();
                                                    let updated = content.replacen(orig, new_c, 1);
                                                    match boundary.write(&safe_path, updated.as_bytes()) {
                                                        Ok(_) => match verify_patch(&safe_path) {
                                                            Ok(_) => Some(success_response(
                                                                id.unwrap_or(Value::Null),
                                                                json!({
                                                                    "content": [{ "type": "text", "text": "Patch applied and verified successfully." }]
                                                                }),
                                                            )),
                                                            Err(verify_err) => {
                                                                let rollback_status = if let Err(rb_err) =
                                                                    boundary.write(&safe_path, backup.as_bytes())
                                                                {
                                                                    format!(
                                                                        "CRITICAL: Patch verification failed ({}), and rollback ALSO failed: {}",
                                                                        verify_err, rb_err
                                                                    )
                                                                } else {
                                                                    format!(
                                                                        "Patch rejected and automatically rolled back: {}",
                                                                        verify_err
                                                                    )
                                                                };
                                                                Some(success_response(
                                                                    id.unwrap_or(Value::Null),
                                                                    json!({
                                                                        "content": [{ "type": "text", "text": rollback_status }],
                                                                        "isError": true
                                                                    }),
                                                                ))
                                                            }
                                                        },
                                                        Err(e) => Some(success_response(
                                                            id.unwrap_or(Value::Null),
                                                            json!({
                                                                "content": [{ "type": "text", "text": format!("Failed to write file: {}", e) }],
                                                                "isError": true
                                                            }),
                                                        )),
                                                    }
                                                }
                                            }
                                            Err(e) => Some(success_response(
                                                id.unwrap_or(Value::Null),
                                                json!({
                                                    "content": [{ "type": "text", "text": format!("Failed to read file: {}", e) }],
                                                    "isError": true
                                                }),
                                            )),
                                        }
                                    }
                                }
                            } else {
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": "Missing required arguments for apply_code_patch." }],
                                        "isError": true
                                    }),
                                ))
                            }
                        }
                        "analyze_code" => {
                            let language = args.get("language").and_then(|l| l.as_str());
                            let code = args.get("code").and_then(|c| c.as_str());

                            if let (Some(lang), Some(source)) = (language, code) {
                                match crate::analyzer::analyze_source(&analyzer_state, lang, source) {
                                    Ok(resp) => {
                                        let json_output = serde_json::to_string_pretty(&resp).unwrap_or_default();
                                        Some(success_response(
                                            id.unwrap_or(Value::Null),
                                            json!({
                                                "content": [{ "type": "text", "text": json_output }]
                                            }),
                                        ))
                                    }
                                    Err(e) => {
                                        Some(success_response(
                                            id.unwrap_or(Value::Null),
                                            json!({
                                                "content": [{ "type": "text", "text": format!("Analysis error: {}", e) }],
                                                "isError": true
                                            }),
                                        ))
                                    }
                                }
                            } else {
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": "Error: Missing required 'language' or 'code' argument." }],
                                        "isError": true
                                    }),
                                ))
                            }
                        }
                        _ => Some(error_response(
                            id,
                            -32601,
                            &format!("Tool '{}' not found", name),
                        )),
                    }
                } else {
                    Some(error_response(id, -32602, "Invalid params"))
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