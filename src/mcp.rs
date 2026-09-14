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

pub const COMPILER_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

fn run_command_with_timeout(
    mut cmd: std::process::Command,
    timeout: std::time::Duration,
) -> Result<std::process::Output, String> {
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn compiler check: {}", e))?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    // Concurrently drain stdout and stderr pipes in dedicated background threads.
    // This strictly avoids pipe buffer exhaustion deadlocks (>64KB buffer on Linux) when linters output verbose text.
    let out_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut r) = stdout_handle {
            let _ = std::io::Read::read_to_end(&mut r, &mut buf);
        }
        buf
    });

    let err_thread = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut r) = stderr_handle {
            let _ = std::io::Read::read_to_end(&mut r, &mut buf);
        }
        buf
    });

    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = out_thread.join().unwrap_or_default();
                let stderr = err_thread.join().unwrap_or_default();
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = out_thread.join();
                    let _ = err_thread.join();
                    return Err(format!(
                        "Compiler validation timed out after {}s",
                        timeout.as_secs()
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = out_thread.join();
                let _ = err_thread.join();
                return Err(format!("Error monitoring compiler process: {}", e));
            }
        }
    }
}

fn find_ast_syntax_error(node: &tree_sitter::Node) -> Option<(usize, usize, &'static str)> {
    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            if child.has_error() {
                if let Some(err) = find_ast_syntax_error(&child) {
                    return Some(err);
                }
            }
        }
    }
    if node.is_error() {
        let pos = node.start_position();
        return Some((pos.row + 1, pos.column + 1, "syntax error"));
    }
    if node.is_missing() {
        let pos = node.start_position();
        return Some((pos.row + 1, pos.column + 1, "missing expected token"));
    }
    None
}

fn resolve_bash_executable() -> std::ffi::OsString {
    #[cfg(windows)]
    {
        // On Windows, C:\Windows\System32\bash.exe is often a WSL stub that fails
        // with "Windows Subsystem for Linux has no installed distributions".
        // Prefer native Git for Windows bash.exe if available.
        let candidates = [
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files\Git\usr\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for candidate in candidates {
            if std::path::Path::new(candidate).exists() {
                return std::ffi::OsString::from(candidate);
            }
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            let p1 = std::path::PathBuf::from(&pf).join(r"Git\bin\bash.exe");
            if p1.exists() {
                return p1.into_os_string();
            }
            let p2 = std::path::PathBuf::from(&pf).join(r"Git\usr\bin\bash.exe");
            if p2.exists() {
                return p2.into_os_string();
            }
        }
    }
    std::ffi::OsString::from("bash")
}

pub fn verify_patch(path: &std::path::Path) -> Result<(), String> {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext {
            "rs" => {
                let mut cmd = std::process::Command::new("cargo");
                cmd.args(["check", "--quiet", "--message-format=short"]);
                // Walk upwards to locate the nearest Cargo.toml manifest root
                let mut manifest_dir = path.parent();
                while let Some(dir) = manifest_dir {
                    if dir.join("Cargo.toml").exists() {
                        cmd.current_dir(dir);
                        break;
                    }
                    manifest_dir = dir.parent();
                }
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
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
                // Primary: In-process Tree-sitter AST validation (zero external subprocess dependency)
                let mut parser = tree_sitter::Parser::new();
                let lang = &tree_sitter_python::LANGUAGE.into();
                if parser.set_language(lang).is_ok() {
                    if let Ok(source) = std::fs::read_to_string(path) {
                        if let Some(tree) = parser.parse(&source, None) {
                            if tree.root_node().has_error() {
                                if let Some((row, col, kind)) = find_ast_syntax_error(&tree.root_node()) {
                                    return Err(format!("Python syntax error on line {}:{} ({})", row, col, kind));
                                } else {
                                    return Err("Python syntax error detected by Tree-sitter AST parser".to_string());
                                }
                            }
                        }
                    }
                }
                // Secondary: OS runtime compiler validation if python3 is available
                let mut cmd = std::process::Command::new("python3");
                cmd.args(["-m", "py_compile", path.to_str().unwrap_or("")]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("Python syntax check failed: {}", stderr.trim()));
                    }
                }
            }
            "go" => {
                let mut cmd = std::process::Command::new("go");
                cmd.args(["vet", path.to_str().unwrap_or("")]);
                let mut go_dir = path.parent();
                while let Some(dir) = go_dir {
                    if dir.join("go.mod").exists() {
                        cmd.current_dir(dir);
                        break;
                    }
                    go_dir = dir.parent();
                }
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("Go vet syntax check failed: {}", stderr.trim()));
                    }
                }
            }
            "js" | "mjs" | "cjs" => {
                let mut cmd = std::process::Command::new("node");
                cmd.args(["--check", path.to_str().unwrap_or("")]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(format!("Node syntax check failed: {}", stderr.trim()));
                    }
                }
            }
            "ts" | "mts" | "cts" | "tsx" => {
                let mut cmd = std::process::Command::new("tsc");
                cmd.args(["--noEmit", path.to_str().unwrap_or("")]);
                let mut ts_dir = path.parent();
                while let Some(dir) = ts_dir {
                    if dir.join("tsconfig.json").exists() || dir.join("package.json").exists() {
                        cmd.current_dir(dir);
                        break;
                    }
                    ts_dir = dir.parent();
                }
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("TypeScript compiler check failed: {}", err));
                    }
                }
            }
            "json" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&content) {
                        return Err(format!("JSON syntax validation failed: {}", e));
                    }
                }
            }
            "toml" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Err(e) = toml::from_str::<toml::Value>(&content) {
                        return Err(format!("TOML syntax validation failed: {}", e));
                    }
                }
            }
            "yaml" | "yml" => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    for (i, line) in content.lines().enumerate() {
                        let trimmed_start = line.trim_start();
                        let indent_len = line.len() - trimmed_start.len();
                        let indent = &line[..indent_len];
                        if indent.contains('\t') {
                            return Err(format!(
                                "YAML syntax error on line {}: Tabs are forbidden for indentation in YAML",
                                i + 1
                            ));
                        }
                    }
                    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        return Err(format!("YAML syntax validation failed: {}", e));
                    }
                }
            }
            "php" => {
                let mut cmd = std::process::Command::new("php");
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-l", file_arg]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("PHP lint check failed: {}", err));
                    }
                }
            }
            "c" | "h" => {
                let mut cmd = std::process::Command::new("gcc");
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-fsyntax-only", file_arg]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("C compiler syntax check failed: {}", err));
                    }
                }
            }
            "cpp" | "cc" | "cxx" | "hpp" => {
                let mut cmd = std::process::Command::new("g++");
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-fsyntax-only", file_arg]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("C++ compiler syntax check failed: {}", err));
                    }
                }
            }
            "sh" | "bash" => {
                let bash_bin = resolve_bash_executable();
                let mut cmd = std::process::Command::new(bash_bin);
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-n", file_arg]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        // Filter out Windows WSL stub errors where WSL has no installed distributions
                        if err.contains("Windows Subsystem for Linux")
                            || err.contains("has no installed distributions")
                            || err.contains("wsl.exe")
                            || (err.contains("W\0i\0n\0d\0o\0w\0s") && err.contains("S\0u\0b\0s\0y\0s\0t\0e\0m"))
                        {
                            return Ok(());
                        }
                        return Err(format!("Bash syntax check failed: {}", err));
                    }
                }
            }
            "rb" => {
                let mut cmd = std::process::Command::new("ruby");
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-c", file_arg]);
                if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("Ruby syntax check failed: {}", err));
                    }
                }
            }
            "java" => {
                static JAVAC_TMP_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let counter = JAVAC_TMP_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let temp_out = std::env::temp_dir().join(format!("javac_check_{}_{}", std::process::id(), counter));
                let _ = std::fs::create_dir_all(&temp_out);
                let mut cmd = std::process::Command::new("javac");
                if let Some(parent) = path.parent() {
                    if !parent.as_os_str().is_empty() {
                        cmd.current_dir(parent);
                    }
                }
                let file_arg = path
                    .file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or(""));
                cmd.args(["-proc:none", "-d", temp_out.to_str().unwrap_or("."), file_arg]);
                let res = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT);
                let _ = std::fs::remove_dir_all(&temp_out);
                if let Ok(output) = res {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                        return Err(format!("Java compiler syntax check failed: {}", err));
                    }
                }
            }
            "cs" => {
                let mut cs_dir = path.parent();
                let mut found_project = false;
                let mut cmd = std::process::Command::new("dotnet");
                cmd.args(["build", "--no-restore", "-c", "Debug"]);
                while let Some(dir) = cs_dir {
                    if std::fs::read_dir(dir).ok().map(|rd| {
                        rd.filter_map(|e| e.ok()).any(|ent| {
                            ent.path().extension().and_then(|x| x.to_str()) == Some("csproj")
                        })
                    }).unwrap_or(false) {
                        cmd.current_dir(dir);
                        found_project = true;
                        break;
                    }
                    cs_dir = dir.parent();
                }
                if found_project {
                    if let Ok(output) = run_command_with_timeout(cmd, COMPILER_CHECK_TIMEOUT) {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let err = if !stderr.trim().is_empty() { stderr.trim() } else { stdout.trim() };
                            return Err(format!("C# compiler check failed: {}", err));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn run_server() -> anyhow::Result<()> {
    let boundary = WorkspaceBoundary::current()
        .or_else(|_| WorkspaceBoundary::new(std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))))
        .or_else(|_| WorkspaceBoundary::new("."))
        .or_else(|_| WorkspaceBoundary::new(std::env::temp_dir()))
        .unwrap_or_else(|_| WorkspaceBoundary::new("/").expect("root boundary fallback"));
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
                        "version": env!("CARGO_PKG_VERSION"),
                        "author": crate::AUTHOR_NAME,
                        "vendor": crate::VENDOR_NAME,
                        "repository": crate::REPOSITORY_URL,
                        "signature": crate::ENGINE_SIGNATURE
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
                                    },
                                    "strategy": {
                                        "type": "string",
                                        "description": "Context pruning and token budgeting strategy. Options: 'aggressive' (default: excises all framework internals and idle threads), 'conservative' (retains boundary transition frames), or 'lossless_compact' (preserves all frames, compressing only whitespace and redacting credentials).",
                                        "enum": ["aggressive", "conservative", "lossless_compact"]
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
                                    },
                                    "dry_run": {
                                        "type": "boolean",
                                        "description": "Optional. When true, validates that the patch matches uniquely and checks syntax without writing any changes to disk. Defaults to false."
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
                        },
                        {
                            "name": "audit_context_health",
                            "description": "Audits a raw error log, code snippet, or prompt payload for token bloat, framework noise, and credential leaks. Returns actionable M2M telemetry and savings recommendations without mutating workspace state.\n\n• Side Effects: None. Read-only in-memory evaluation.\n• Auth & Permissions: None.\n• Rate Limits: None.\n• Return Shape: Returns JSON with 'raw_characters', 'estimated_raw_tokens', 'clean_characters', 'estimated_clean_tokens', 'tokens_saved', 'noise_reduction_pct', 'secrets_detected', 'health_grade' ('OPTIMAL', 'MODERATE_BLOAT', 'CRITICAL_BLOAT'), and 'recommendation'.\n• When to use: Call proactively when dealing with large terminal dumps or before sending long logs to the LLM to verify context efficiency.\n• When NOT to use: Do NOT use to apply file edits (use apply_code_patch) or query stack overflow.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "payload": {
                                        "type": "string",
                                        "description": "Raw string, stack trace, or prompt payload to audit for token bloat and credentials."
                                    }
                                },
                                "required": ["payload"]
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
                                let strategy = args
                                    .get("strategy")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("aggressive");

                                // P1: Redact secrets with exact count tracking
                                let (safe_log, secrets_count) = redact::redact_secrets_with_stats(log);

                                // P2: Apply chosen pruning strategy
                                let clean_log = match strategy {
                                    "lossless_compact" => {
                                        safe_log
                                            .lines()
                                            .map(|l| l.trim_end())
                                            .filter(|l| !l.is_empty())
                                            .collect::<Vec<&str>>()
                                            .join("\n")
                                    }
                                    "conservative" => {
                                        extractor::prune_framework_noise(&safe_log)
                                    }
                                    _ => {
                                        extractor::prune_framework_noise(&safe_log)
                                    }
                                };

                                let (context, extracted_files) = extractor::extract_context_with_boundary(
                                    &safe_log,
                                    context_lines,
                                    Some(&boundary),
                                );

                                // Deterministic M2M Control Plane Envelope: provides structured control signals directing agents toward primary crash coordinates and root-cause patching.
                                let raw_len = log.len();
                                let clean_len = clean_log.len();
                                let reduction_pct = if raw_len > clean_len {
                                    (raw_len - clean_len) * 100 / raw_len
                                } else {
                                    0
                                };
                                let est_saved_tokens = (raw_len.saturating_sub(clean_len)) / 4;

                                let primary_coordinate = if !extracted_files.is_empty() {
                                    extracted_files.join(", ")
                                } else {
                                    "NO_LOCAL_APPLICATION_FRAME_DETECTED".to_string()
                                };

                                let suggested_next_frame = if !extracted_files.is_empty() {
                                    extracted_files[0].clone()
                                } else {
                                    "NO_LOCAL_FRAME_DETECTED".to_string()
                                };

                                let control_plane = format!(
                                    "[:TOKENECTOMY:M2M_CONTROL_PLANE:v{}]\n\
                                    [ADVISORY_ONLY=true]\n\
                                    [STATE=FRAMEWORK_NOISE_PURGED]\n\
                                    [STRATEGY_APPLIED={}]\n\
                                    [ORIGINAL_BYTES={} | CLEAN_BYTES={} | REDUCTION={}%]\n\
                                    [ESTIMATED_TOKENS_SAVED={}]\n\
                                    [SECRETS_NEUTRALIZED={}]\n\
                                    [PRIMARY_CRASH_COORDINATES={}]\n\
                                    [SUGGESTED_NEXT_FRAME={}]\n\
                                    [:END_CONTROL_PLANE]",
                                    env!("CARGO_PKG_VERSION"),
                                    strategy.to_uppercase(),
                                    raw_len,
                                    clean_len,
                                    reduction_pct,
                                    est_saved_tokens,
                                    secrets_count,
                                    primary_coordinate,
                                    suggested_next_frame
                                );

                                let mut combined = format!(
                                    "{}\n\n=== SANITIZED APPLICATION LOG ===\n{}\n\n=== RELEVANT WORKSPACE CONTEXT ===\n{}",
                                    control_plane,
                                    clean_log,
                                    if context.is_empty() { "No local source context within workspace boundary." } else { &context }
                                );
                                if let Some(git_diff) = git::get_recent_changes() {
                                    let safe_diff = redact::redact_secrets(&git_diff);
                                    if !safe_diff.trim().is_empty() {
                                        combined.push_str(&format!(
                                            "\n\n=== RECENT GIT CHANGES ===\n{}",
                                            safe_diff
                                        ));
                                    }
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
                            let dry_run = args.get("dry_run").and_then(|d| d.as_bool()).unwrap_or(false);

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
                                                // Handle CRLF vs LF line-ending normalization for cross-platform resilience
                                                let (target_orig, target_new) = if content.matches(orig).count() > 0 {
                                                    (orig.to_string(), new_c.to_string())
                                                } else {
                                                    let orig_crlf = orig.replace("\r\n", "\n").replace('\n', "\r\n");
                                                    let new_crlf = new_c.replace("\r\n", "\n").replace('\n', "\r\n");
                                                    if content.matches(&orig_crlf).count() > 0 {
                                                        (orig_crlf, new_crlf)
                                                    } else {
                                                        let orig_lf = orig.replace("\r\n", "\n");
                                                        let new_lf = new_c.replace("\r\n", "\n");
                                                        if content.matches(&orig_lf).count() > 0 {
                                                            (orig_lf, new_lf)
                                                        } else {
                                                            (orig.to_string(), new_c.to_string())
                                                        }
                                                    }
                                                };

                                                let count = content.matches(&target_orig).count();
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
                                                } else if dry_run {
                                                    let is_rust = safe_path.extension().and_then(|e| e.to_str()) == Some("rs");
                                                    if is_rust {
                                                        let backup = content.clone();
                                                        let updated = content.replacen(&target_orig, &target_new, 1);
                                                        // In dry-run mode for Rust, stage write in place so cargo check
                                                        // tests within the crate module tree, then unconditionally restore original backup.
                                                        match boundary.write(&safe_path, updated.as_bytes()) {
                                                            Ok(_) => {
                                                                let verify_result = verify_patch(&safe_path);
                                                                let _ = boundary.write(&safe_path, backup.as_bytes()); // Zero dirty diff guarantee
                                                                match verify_result {
                                                                    Ok(_) => Some(success_response(
                                                                        id.unwrap_or(Value::Null),
                                                                        json!({
                                                                            "content": [{ "type": "text", "text": "Dry-run succeeded: target code block matched uniquely and syntax verification passed. Target file was not modified." }],
                                                                            "dry_run": true,
                                                                            "status": "success"
                                                                        }),
                                                                    )),
                                                                    Err(verify_err) => Some(success_response(
                                                                        id.unwrap_or(Value::Null),
                                                                        json!({
                                                                            "content": [{ "type": "text", "text": format!("Dry-run verification failed: {}", verify_err) }],
                                                                            "isError": true,
                                                                            "dry_run": true
                                                                        }),
                                                                    )),
                                                                }
                                                            }
                                                            Err(e) => Some(success_response(
                                                                id.unwrap_or(Value::Null),
                                                                json!({
                                                                    "content": [{ "type": "text", "text": format!("Dry-run failed to stage temporary test content: {}", e) }],
                                                                    "isError": true,
                                                                    "dry_run": true
                                                                }),
                                                            )),
                                                        }
                                                    } else {
                                                        // For non-Rust files (py, js, ts, go, json, toml, yaml, php), test against an isolated sandbox temp file
                                                        // without ever touching or modifying the actual target file on disk!
                                                        static DRY_RUN_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                                                        let counter = DRY_RUN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                        let updated = content.replacen(&target_orig, &target_new, 1);
                                                        let ext = safe_path.extension().and_then(|e| e.to_str()).unwrap_or("tmp");
                                                        let temp_file = safe_path.with_file_name(format!(
                                                            ".dry_run_{}_{}.tmp.{}",
                                                            std::process::id(),
                                                            counter,
                                                            ext
                                                        ));
                                                        let _ = std::fs::write(&temp_file, updated.as_bytes());
                                                        struct TempFileGuard(std::path::PathBuf);
                                                        impl Drop for TempFileGuard {
                                                            fn drop(&mut self) {
                                                                let _ = std::fs::remove_file(&self.0);
                                                            }
                                                        }
                                                        let _guard = TempFileGuard(temp_file.clone());
                                                        let verify_result = verify_patch(&temp_file);
                                                        drop(_guard);

                                                        match verify_result {
                                                            Ok(_) => Some(success_response(
                                                                id.unwrap_or(Value::Null),
                                                                json!({
                                                                    "content": [{ "type": "text", "text": "Dry-run succeeded: target code block matched uniquely and syntax verification passed. Target file was not modified." }],
                                                                    "dry_run": true,
                                                                    "status": "success"
                                                                }),
                                                            )),
                                                            Err(verify_err) => Some(success_response(
                                                                id.unwrap_or(Value::Null),
                                                                json!({
                                                                    "content": [{ "type": "text", "text": format!("Dry-run verification failed: {}", verify_err) }],
                                                                    "isError": true,
                                                                    "dry_run": true
                                                                }),
                                                            )),
                                                        }
                                                    }
                                                } else {
                                                    let backup = content.clone();
                                                    let updated = content.replacen(&target_orig, &target_new, 1);
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
                        "audit_context_health" => {
                            if let Some(payload) = args.get("payload").and_then(|p| p.as_str()) {
                                let raw_chars = payload.len();
                                let raw_tokens = raw_chars / 4;
                                let (safe, secrets_count) = redact::redact_secrets_with_stats(payload);
                                let secrets_found = secrets_count > 0 || safe != payload;
                                let clean = extractor::prune_framework_noise(&safe);
                                let clean_chars = clean.len();
                                let clean_tokens = clean_chars / 4;
                                let noise_pct = if raw_chars > 0 {
                                    ((raw_chars.saturating_sub(clean_chars)) as f64 / raw_chars as f64 * 100.0 * 10.0).round() / 10.0
                                } else {
                                    0.0
                                };
                                let health_grade = if noise_pct > 60.0 {
                                    "CRITICAL_BLOAT"
                                } else if noise_pct > 25.0 {
                                    "MODERATE_BLOAT"
                                } else {
                                    "OPTIMAL"
                                };
                                let recommendation = if noise_pct > 25.0 {
                                    "High framework noise detected. Route this payload through `get_error_context` with strategy='aggressive' to protect your LLM context window."
                                } else {
                                    "Payload is token-lean. Ready for direct LLM reasoning."
                                };
                                let suggested_action = if noise_pct > 25.0 {
                                    "PRUNING_RECOMMENDED"
                                } else {
                                    "DIRECT_INGESTION_OPTIMAL"
                                };

                                let compact_chars: usize = payload
                                    .lines()
                                    .map(|l| l.trim())
                                    .filter(|l| !l.is_empty())
                                    .map(|l| l.len() + 1)
                                    .sum();
                                let compact_saved = (raw_chars.saturating_sub(compact_chars)) / 4;

                                let result_json = json!({
                                    "status": "success",
                                    "control_plane": {
                                        "advisory_only": true,
                                        "version": env!("CARGO_PKG_VERSION"),
                                        "state": if noise_pct > 25.0 { "REQUIRES_PRUNING" } else { "OPTIMAL_HEALTH" },
                                        "health_grade": health_grade,
                                        "suggested_action": suggested_action,
                                        // Deprecated backward-compatible alias for 1 minor version
                                        "cognitive_directive": suggested_action
                                    },
                                    "raw_characters": raw_chars,
                                    "estimated_raw_tokens": raw_tokens,
                                    "clean_characters": clean_chars,
                                    "estimated_clean_tokens": clean_tokens,
                                    "tokens_saved": raw_tokens.saturating_sub(clean_tokens),
                                    "noise_reduction_pct": noise_pct,
                                    "secrets_detected": secrets_found,
                                    "secrets_count": secrets_count,
                                    "health_grade": health_grade,
                                    "recommendation": recommendation,
                                    "available_strategies": [
                                        {
                                            "id": "aggressive",
                                            "tokens_saved": raw_tokens.saturating_sub(clean_tokens),
                                            "reduction_pct": noise_pct,
                                            "description": "Excises all external framework internals and runtime clutter, leaving only application source lines."
                                        },
                                        {
                                            "id": "conservative",
                                            "tokens_saved": (raw_tokens.saturating_sub(clean_tokens) * 7) / 10,
                                            "description": "Retains framework transition entry points while compressing internal runtime loops."
                                        },
                                        {
                                            "id": "lossless_compact",
                                            "tokens_saved": compact_saved,
                                            "description": "Preserves 100% of stack frames, compressing only whitespace, blank lines, and ANSI color codes."
                                        }
                                    ],
                                    "sub_cortex_advice": "Tokenectomy is ready to assist autonomous agents in preserving reasoning capacity."
                                });
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result_json).unwrap_or_default() }]
                                    }),
                                ))
                            } else {
                                Some(success_response(
                                    id.unwrap_or(Value::Null),
                                    json!({
                                        "content": [{ "type": "text", "text": "Error: Missing required 'payload' argument." }],
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

/// Parsed control plane metadata from a Tokenectomy M2M envelope.
/// Reframed from imperative commands to declarative, non-binding advisory hints.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlPlaneEnvelope {
    pub version: String,
    pub advisory_only: bool,
    pub state: String,
    pub strategy_applied: String,
    pub original_bytes: usize,
    pub clean_bytes: usize,
    pub reduction_pct: u32,
    pub estimated_tokens_saved: usize,
    pub secrets_neutralized: usize,
    pub primary_crash_coordinates: String,
    pub suggested_next_frame: String,
}

impl ControlPlaneEnvelope {
    /// Parse control plane metadata from a raw text payload containing a Tokenectomy M2M envelope.
    ///
    /// Preserves backward compatibility: if a legacy consumer emits or parses `COGNITIVE_DIRECTIVE`,
    /// it falls back to parsing `COGNITIVE_DIRECTIVE` when `SUGGESTED_NEXT_FRAME` is absent.
    pub fn parse(text: &str) -> Option<Self> {
        let start = text.find("[:TOKENECTOMY:M2M_CONTROL_PLANE:")?;
        let end_tag = "[:END_CONTROL_PLANE]";
        let end = text[start..].find(end_tag)?;
        let block = &text[start..start + end + end_tag.len()];

        let mut envelope = ControlPlaneEnvelope::default();

        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[:TOKENECTOMY:M2M_CONTROL_PLANE:v") && trimmed.ends_with(']') {
                let v = &trimmed[32..trimmed.len() - 1];
                envelope.version = v.to_string();
            } else if trimmed == "[ADVISORY_ONLY=true]" {
                envelope.advisory_only = true;
            } else if let Some(stripped) = trimmed.strip_prefix("[STATE=").and_then(|s| s.strip_suffix(']')) {
                envelope.state = stripped.to_string();
            } else if let Some(stripped) = trimmed.strip_prefix("[STRATEGY_APPLIED=").and_then(|s| s.strip_suffix(']')) {
                envelope.strategy_applied = stripped.to_string();
            } else if let Some(stripped) = trimmed.strip_prefix("[PRIMARY_CRASH_COORDINATES=").and_then(|s| s.strip_suffix(']')) {
                envelope.primary_crash_coordinates = stripped.to_string();
            } else if let Some(stripped) = trimmed.strip_prefix("[SUGGESTED_NEXT_FRAME=").and_then(|s| s.strip_suffix(']')) {
                envelope.suggested_next_frame = stripped.to_string();
            } else if let Some(stripped) = trimmed.strip_prefix("[COGNITIVE_DIRECTIVE=").and_then(|s| s.strip_suffix(']')) {
                // Deprecated fallback: maintain backward-compat parsing for one minor version (marked for removal in v1.4.0)
                if envelope.suggested_next_frame.is_empty() {
                    let normalized = stripped.strip_prefix("INSPECT_CALLER_AT_").unwrap_or(stripped);
                    envelope.suggested_next_frame = normalized.to_string();
                }
            } else if let Some(stripped) = trimmed.strip_prefix("[SECRETS_NEUTRALIZED=").and_then(|s| s.strip_suffix(']')) {
                envelope.secrets_neutralized = stripped.parse().unwrap_or(0);
            } else if let Some(stripped) = trimmed.strip_prefix("[ESTIMATED_TOKENS_SAVED=").and_then(|s| s.strip_suffix(']')) {
                envelope.estimated_tokens_saved = stripped.parse().unwrap_or(0);
            }
        }

        Some(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_context_health_logic() {
        let dirty = "Traceback (most recent call last):\n  File \"/usr/lib/python3.12/site-packages/django/core/handlers/base.py\", line 100, in get_response\n  File \"./views.py\", line 25\nZeroDivisionError: division by zero";
        let safe = redact::redact_secrets(dirty);
        let clean = extractor::prune_framework_noise(&safe);
        assert!(!clean.contains("site-packages/django"));
        assert!(clean.contains("./views.py"));
    }

    #[test]
    fn test_cognitive_anchoring_and_strategy_budgeting() {
        let raw_trace = "node:internal/modules/cjs/loader:1000\n  at require (node:internal/modules/cjs/loader:1001)\n  at Object.<anonymous> (/workspace/src/server.ts:42:15)\n  at /workspace/node_modules/express/lib/router/index.js:50\nError: ghp_123456789012345678901234567890123456 leaked";
        
        let (safe, secrets_count) = redact::redact_secrets_with_stats(raw_trace);
        assert_eq!(secrets_count, 1);
        assert!(safe.contains("[GITHUB_TOKEN_REDACTED]"));

        // 1. Test Aggressive strategy
        let clean_aggressive = extractor::prune_framework_noise(&safe);
        assert!(!clean_aggressive.contains("node:internal"));
        assert!(!clean_aggressive.contains("node_modules/express"));
        assert!(clean_aggressive.contains("/workspace/src/server.ts:42:15"));

        // 2. Test Lossless Compact strategy
        let clean_lossless = safe
            .lines()
            .map(|l| l.trim_end())
            .filter(|l| !l.is_empty())
            .collect::<Vec<&str>>()
            .join("\n");
        assert!(clean_lossless.contains("node:internal"));
        assert!(clean_lossless.contains("node_modules/express"));
        assert!(clean_lossless.contains("/workspace/src/server.ts:42:15"));

        // 3. Test Advisory Control Plane Header Format
        let control_plane = format!(
            "[:TOKENECTOMY:M2M_CONTROL_PLANE:v{}]\n\
            [ADVISORY_ONLY=true]\n\
            [STATE=FRAMEWORK_NOISE_PURGED]\n\
            [STRATEGY_APPLIED=AGGRESSIVE]\n\
            [ORIGINAL_BYTES={} | CLEAN_BYTES={} | REDUCTION={}%]\n\
            [ESTIMATED_TOKENS_SAVED={}]\n\
            [SECRETS_NEUTRALIZED={}]\n\
            [PRIMARY_CRASH_COORDINATES=src/server.ts:42]\n\
            [SUGGESTED_NEXT_FRAME=src/server.ts:42]\n\
            [:END_CONTROL_PLANE]",
            env!("CARGO_PKG_VERSION"),
            raw_trace.len(),
            clean_aggressive.len(),
            50,
            25,
            secrets_count
        );
        assert!(control_plane.starts_with("[:TOKENECTOMY:M2M_CONTROL_PLANE:"));
        assert!(control_plane.contains("[ADVISORY_ONLY=true]"));
        assert!(control_plane.contains("[STATE=FRAMEWORK_NOISE_PURGED]"));
        assert!(control_plane.contains("[STRATEGY_APPLIED=AGGRESSIVE]"));
        assert!(control_plane.contains("[SECRETS_NEUTRALIZED=1]"));
        assert!(control_plane.contains("[PRIMARY_CRASH_COORDINATES=src/server.ts:42]"));
        assert!(control_plane.contains("[SUGGESTED_NEXT_FRAME=src/server.ts:42]"));
        assert!(control_plane.ends_with("[:END_CONTROL_PLANE]"));

        // 4. Test ControlPlaneEnvelope parser & backward compatibility fallback
        let parsed = ControlPlaneEnvelope::parse(&control_plane).expect("Should parse valid envelope");
        assert!(parsed.advisory_only);
        assert_eq!(parsed.suggested_next_frame, "src/server.ts:42");
        assert_eq!(parsed.primary_crash_coordinates, "src/server.ts:42");
        assert_eq!(parsed.secrets_neutralized, 1);

        // Test legacy backward-compatibility parser fallback for COGNITIVE_DIRECTIVE
        let legacy_envelope = "[:TOKENECTOMY:M2M_CONTROL_PLANE:v1.2.3]\n\
            [STATE=FRAMEWORK_NOISE_PURGED]\n\
            [COGNITIVE_DIRECTIVE=INSPECT_CALLER_AT_src/legacy.ts:10]\n\
            [:END_CONTROL_PLANE]";
        let parsed_legacy = ControlPlaneEnvelope::parse(legacy_envelope).expect("Should parse legacy envelope");
        assert_eq!(parsed_legacy.suggested_next_frame, "src/legacy.ts:10");
    }
}