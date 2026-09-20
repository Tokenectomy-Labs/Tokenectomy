// src/analyzer/js_debugger.rs — AST Rule for detecting leftover debugger statements in JS/TS

use super::{byte_col_to_utf16_col, walk_ast_bounded, AnalysisConfig, Finding, Rule, Severity};
use tree_sitter::Node;

pub struct JsNoDebuggerRule;

impl Rule for JsNoDebuggerRule {
    fn name(&self) -> &'static str {
        "javascript/no-debugger"
    }

    fn description(&self) -> &'static str {
        "Detects leftover 'debugger;' statements in JavaScript/TypeScript code (CWE-489)"
    }

    fn language(&self) -> &'static str {
        "javascript"
    }

    fn check(
        &self,
        root: &Node,
        source_bytes: &[u8],
        source_text: &str,
        config: &AnalysisConfig,
    ) -> Result<Vec<Finding>, String> {
        let mut findings = Vec::new();

        walk_ast_bounded(root, config.max_depth, config.max_ast_nodes, |node, _depth| {
            if node.kind() == "debugger_statement" {
                let start_pos = node.start_position();
                let line_1 = start_pos.row + 1;
                let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);
                let snippet = node
                    .utf8_text(source_bytes)
                    .ok()
                    .map(|s| s.lines().next().unwrap_or(s).to_string());

                findings.push(Finding {
                    rule: "javascript/no-debugger".to_string(),
                    severity: Severity::Warning,
                    message: "Code Quality Hazard (CWE-489): Active 'debugger;' breakpoint statement left in production code".to_string(),
                    line: line_1,
                    column: col_1,
                    snippet,
                });
            }
            Ok(())
        })?;

        Ok(findings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_debugger_statement() {
        let code = "function test() {\n    debugger;\n    return 42;\n}\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = JsNoDebuggerRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 2);
        assert!(findings[0].message.contains("debugger"));
    }
}
