// tests/ast_analyzer_test.rs — Verification of AST Analyzer for Tokenectomy Razor

use tokenectomy::analyzer::{analyze_source, byte_col_to_utf16_col, AnalysisConfig, AppState};

#[test]
fn test_unclosed_open_flagged() {
    let state = AppState::new();
    let code = r#"
def read_data():
    f = open("leak.txt", "r")
    return f.read()
"#;
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 1, "Unclosed open() should be flagged");
    assert_eq!(res.findings[0].rule, "python/unclosed-open");
    assert_eq!(res.findings[0].line, 3);
}

#[test]
fn test_with_statement_not_flagged() {
    let state = AppState::new();
    let code = r#"
def read_data():
    with open("clean.txt", "r") as f:
        return f.read()
"#;
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 0, "open() inside 'with' must not be flagged");
}

#[test]
fn test_contextlib_closing_not_flagged() {
    // #15: contextlib.closing(open(...))
    let state = AppState::new();
    let code = r#"
from contextlib import closing

def read_data():
    with closing(open("clean.txt", "r")) as f:
        return f.read()
"#;
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 0, "'contextlib.closing(open())' must not be flagged");
}

#[test]
fn test_try_finally_close_not_flagged() {
    // #15: try ... finally: f.close()
    let state = AppState::new();
    let code = r#"
def read_data():
    f = open("safe.txt", "r")
    try:
        data = f.read()
    finally:
        f.close()
    return data
"#;
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 0, "try/finally with f.close() must not be flagged");
}

#[test]
fn test_open_shadowing_not_flagged() {
    // #14: Custom `def open(...)` should not be falsely treated as builtin open()
    let state = AppState::new();
    let code = r#"
def open(custom_arg):
    return f"custom: {custom_arg}"

def worker():
    res = open("something")
    return res
"#;
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 0, "Locally shadowed open() must not be flagged");
}

#[test]
fn test_multibyte_utf16_column_alignment() {
    // #16: UTF-16 code units vs tree-sitter byte column
    let code = "# 🇮🇩 Halo dunia\nf = open('test.txt')\n";
    let state = AppState::new();
    let res = analyze_source(&state, "python", code).expect("Analysis should succeed");
    assert_eq!(res.total_findings, 1);

    // Line 2: "f = open('test.txt')"
    // Column 5 in 1-indexed (f is 1, ' ' is 2, '=' is 3, ' ' is 4, 'o' is 5)
    assert_eq!(res.findings[0].line, 2);
    assert_eq!(res.findings[0].column, 5);

    // Test inline multibyte prefix on the same line:
    let line_with_emoji = "/* 🇮🇩 */ f = open('test.txt')";
    let col = byte_col_to_utf16_col(line_with_emoji, 0, 14);
    assert_eq!(col, 11, "Column in UTF-16 should be 11 (10 prefix units + 1)");
}

#[test]
fn test_bounded_ast_traversal_limit() {
    // #12: Extremely deep nested AST exceeds configured limit
    let strict_config = AnalysisConfig {
        max_ast_nodes: 50,
        max_depth: 5,
        max_source_bytes: 1024 * 1024,
    };
    let state = AppState::with_config(strict_config);

    // Create nested expression: [[[[[[1]]]]]]
    let deep_code = "x = [[[[[[[[[[[[[[[[[[[[1]]]]]]]]]]]]]]]]]]]]\n";
    let res = analyze_source(&state, "python", deep_code);
    assert!(res.is_err(), "Deeply nested code should be rejected by depth guard");
    assert!(res.unwrap_err().contains("AST nesting depth limit exceeded"));
}

#[tokio::test]
async fn test_proxy_v1_analyze_endpoint() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();
    drop(listener);

    tokio::spawn(async move {
        let _ = tokenectomy::proxy::run_reverse_proxy(&format!("127.0.0.1:{}", port), "https://api.openai.com").await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .expect("Should connect to proxy");

    let payload = serde_json::json!({
        "language": "python",
        "code": "def run():\n    f = open('leak.log')\n"
    }).to_string();

    let req = format!(
        "POST /v1/analyze HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port,
        payload.len(),
        payload
    );

    stream.write_all(req.as_bytes()).await.unwrap();
    let mut resp_buf = Vec::new();
    stream.read_to_end(&mut resp_buf).await.unwrap();
    let resp_str = String::from_utf8_lossy(&resp_buf);

    assert!(resp_str.contains("HTTP/1.1 200 OK"), "Expected 200 OK, got: {}", resp_str);
    assert!(resp_str.contains(r#""version":"v1""#));
    assert!(resp_str.contains(r#""total_findings":1"#));
    assert!(resp_str.contains("python/unclosed-open"));
}
