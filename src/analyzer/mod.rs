// src/analyzer/mod.rs — High-Performance M2M AST Code Analysis Engine for Tokenectomy Razor

pub mod python_open;
pub mod python_exec;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tree_sitter::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub column: usize, // 1-indexed UTF-16 column for LSP / agent alignment (#16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    pub max_ast_nodes: usize,
    pub max_depth: usize,
    pub max_source_bytes: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_ast_nodes: 50_000,
            max_depth: 128,
            max_source_bytes: 1024 * 1024, // 1 MB
        }
    }
}

pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn language(&self) -> &'static str;
    fn check(
        &self,
        root: &Node,
        source_bytes: &[u8],
        source_text: &str,
        config: &AnalysisConfig,
    ) -> Result<Vec<Finding>, String>;
}

/// AppState holding pre-allocated rules (#20) and global configuration.
#[derive(Clone)]
pub struct AppState {
    pub rules: Arc<Vec<Box<dyn Rule>>>,
    pub config: Arc<AnalysisConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();
        rules.push(Box::new(python_open::PythonUnclosedOpenRule));
        rules.push(Box::new(python_exec::PythonDangerousExecRule));

        Self {
            rules: Arc::new(rules),
            config: Arc::new(AnalysisConfig::default()),
        }
    }

    pub fn with_config(config: AnalysisConfig) -> Self {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();
        rules.push(Box::new(python_open::PythonUnclosedOpenRule));
        rules.push(Box::new(python_exec::PythonDangerousExecRule));

        Self {
            rules: Arc::new(rules),
            config: Arc::new(config),
        }
    }
}

/// Converts tree-sitter 0-indexed byte column to 1-indexed UTF-16 column (#16)
pub fn byte_col_to_utf16_col(source: &str, line_0_idx: usize, byte_col: usize) -> usize {
    if let Some(line) = source.lines().nth(line_0_idx) {
        let safe_byte_col = byte_col.min(line.len());
        let prefix = match std::str::from_utf8(&line.as_bytes()[..safe_byte_col]) {
            Ok(s) => s,
            Err(e) => &line[..e.valid_up_to()],
        };
        prefix.encode_utf16().count() + 1
    } else {
        byte_col + 1
    }
}

/// Safe AST traversal with bounded node count and recursion depth (#12)
pub fn walk_ast_bounded<F>(
    root: &Node,
    max_depth: usize,
    max_nodes: usize,
    mut visitor: F,
) -> Result<(), String>
where
    F: FnMut(&Node, usize) -> Result<(), String>,
{
    let mut cursor = root.walk();
    let mut depth = 0;
    let mut node_count = 0;

    loop {
        node_count += 1;
        if node_count > max_nodes {
            return Err(format!(
                "AST node count limit exceeded (visited > {} nodes)",
                max_nodes
            ));
        }

        let current = cursor.node();
        visitor(&current, depth)?;

        if cursor.goto_first_child() {
            depth += 1;
            if depth > max_depth {
                return Err(format!(
                    "AST nesting depth limit exceeded (depth > {})",
                    max_depth
                ));
            }
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() {
                return Ok(());
            }
            depth -= 1;
            if cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRequest {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeResponse {
    pub version: String,
    pub status: String,
    pub language: String,
    pub total_findings: usize,
    pub findings: Vec<Finding>,
    pub duration_ms: f64,
}

pub fn analyze_source(
    state: &AppState,
    language: &str,
    code: &str,
) -> Result<AnalyzeResponse, String> {
    let start_time = std::time::Instant::now();

    if code.len() > state.config.max_source_bytes {
        return Err(format!(
            "Payload too large: {} bytes (max: {} bytes)",
            code.len(),
            state.config.max_source_bytes
        ));
    }

    let mut parser = tree_sitter::Parser::new();
    let lang_lower = language.to_lowercase();

    match lang_lower.as_str() {
        "python" | "py" => {
            parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|e| format!("Failed to set tree-sitter python language: {}", e))?;
        }
        other => return Err(format!("Unsupported language: '{}'", other)),
    }

    let tree = parser
        .parse(code, None)
        .ok_or_else(|| "Failed to parse code into AST".to_string())?;
    let root = tree.root_node();
    let source_bytes = code.as_bytes();

    let mut all_findings = Vec::new();

    // Iterate over pre-registered rules in AppState without allocation (#20)
    for rule in state.rules.iter() {
        if rule.language().eq_ignore_ascii_case(&lang_lower)
            || (lang_lower == "py" && rule.language() == "python")
        {
            let findings = rule.check(&root, source_bytes, code, &state.config)?;
            all_findings.extend(findings);
        }
    }

    let duration = start_time.elapsed().as_secs_f64() * 1000.0;

    Ok(AnalyzeResponse {
        version: "v1".to_string(),
        status: "ok".to_string(),
        language: language.to_string(),
        total_findings: all_findings.len(),
        findings: all_findings,
        duration_ms: (duration * 100.0).round() / 100.0,
    })
}
