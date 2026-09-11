use tokenectomy::analyzer::{analyze_source, byte_col_to_utf16_col, AppState};

#[test]
fn test_syntax_error_resilience() {
    // Malformed Python code must not panic tree-sitter or the analyzer
    let state = AppState::new();
    let malformed_code = "def broken(:\n   open('incomplete.txt'\n   if while for\n";
    let res = analyze_source(&state, "python", malformed_code);
    assert!(res.is_ok(), "Malformed code must parse safely via tree-sitter error recovery nodes");
}

#[test]
fn test_empty_and_whitespace_input() {
    let state = AppState::new();
    assert_eq!(analyze_source(&state, "python", "").unwrap().total_findings, 0);
    assert_eq!(analyze_source(&state, "python", "   \n\n\t\r\n").unwrap().total_findings, 0);
}

#[test]
fn test_unsupported_language_clean_error() {
    let state = AppState::new();
    let res = analyze_source(&state, "brainfuck", "++++++++++[>+++++++>++++++++++>+++>+<<<<-]");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "Unsupported language: 'brainfuck'");
}

#[test]
fn test_payload_size_limit_rejection() {
    // Default max_source_bytes is 1MB. Input exceeding 1MB must be rejected with 422/error before AST parsing.
    let state = AppState::new();
    let large_code = "x = 1\n".repeat(200_000); // ~1.2 MB
    let res = analyze_source(&state, "python", &large_code);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Payload too large"));
}

#[test]
fn test_pathological_nesting_depth_cap() {
    // Generates 300 levels of nested parentheses
    let mut pathological = String::from("x = ");
    for _ in 0..300 {
        pathological.push('(');
    }
    pathological.push_str("1");
    for _ in 0..300 {
        pathological.push(')');
    }
    pathological.push('\n');

    let state = AppState::new(); // max_depth = 128
    let res = analyze_source(&state, "python", &pathological);
    assert!(res.is_err(), "Must reject depth exceeding 128");
    assert!(res.unwrap_err().contains("AST nesting depth limit exceeded"));
}

#[test]
fn test_method_call_named_open_is_ignored() {
    // obj.open() or Session().open() is NOT the Python builtin open()
    let state = AppState::new();
    let code = r#"
class Storage:
    def open(self, path):
        pass

s = Storage()
s.open("my_file.txt")
"#;
    let res = analyze_source(&state, "python", code).unwrap();
    assert_eq!(res.total_findings, 0, "Method invocation 's.open()' must NOT be flagged as builtin open");
}

#[test]
fn test_multiple_unclosed_calls_exact_coordinates() {
    let state = AppState::new();
    let code = "a = open('1.txt')\nb = 2\nc = open('2.txt')\n";
    let res = analyze_source(&state, "python", code).unwrap();
    assert_eq!(res.total_findings, 2);

    assert_eq!(res.findings[0].line, 1);
    assert_eq!(res.findings[0].column, 5); // 1-indexed

    assert_eq!(res.findings[1].line, 3);
    assert_eq!(res.findings[1].column, 5); // 1-indexed
}

#[test]
fn test_unicode_surrogate_and_multibyte_precision() {
    // "👋" (U+1F44B) is 4 bytes in UTF-8 and 2 code units in UTF-16
    // "こんにちは" is 15 bytes in UTF-8 and 5 code units in UTF-16
    let line = "# 👋 こんにちは f = open('test.txt')";
    let byte_offset = line.find("open").unwrap(); // Byte offset of 'open'

    // Calculate prefix length in UTF-16
    let prefix = &line[..byte_offset];
    let expected_utf16_col = prefix.encode_utf16().count() + 1;

    let calculated_col = byte_col_to_utf16_col(line, 0, byte_offset);
    assert_eq!(calculated_col, expected_utf16_col);
    assert_ne!(calculated_col, byte_offset + 1, "UTF-16 col must differ from raw byte offset on multibyte strings");
}

#[test]
fn test_nested_try_finally_complex_flow() {
    let state = AppState::new();
    let code = r#"
def process_two_files():
    f1 = open("file1.txt")
    try:
        f2 = open("file2.txt")
        try:
            return f1.read() + f2.read()
        finally:
            f2.close()
    finally:
        f1.close()
"#;
    let res = analyze_source(&state, "python", code).unwrap();
    assert_eq!(res.total_findings, 0, "Nested try/finally blocks must both resolve safely");
}

#[test]
fn test_unclosed_returned_open_is_flagged() {
    let state = AppState::new();
    let code = r#"
def get_file():
    return open("unclosed.txt")
"#;
    let res = analyze_source(&state, "python", code).unwrap();
    assert_eq!(res.total_findings, 1, "Unmanaged returned open() must be flagged");
    assert_eq!(res.findings[0].line, 3);
}

#[test]
fn test_mcp_analyze_code_tool_json_serialization() {
    let state = AppState::new();
    let code = "f = open('leak.txt')\n";
    let res = analyze_source(&state, "python", code).unwrap();
    let json_val = serde_json::to_value(&res).unwrap();

    assert_eq!(json_val["version"], "v1");
    assert_eq!(json_val["status"], "ok");
    assert_eq!(json_val["language"], "python");
    assert_eq!(json_val["total_findings"], 1);
    assert_eq!(json_val["findings"][0]["rule"], "python/unclosed-open");
    assert_eq!(json_val["findings"][0]["severity"], "warning");
    assert_eq!(json_val["findings"][0]["line"], 1);
    assert_eq!(json_val["findings"][0]["column"], 5);
    assert!(json_val["duration_ms"].is_number());
}
