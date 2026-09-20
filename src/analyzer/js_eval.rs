// src/analyzer/js_eval.rs — AST Rule for detecting dangerous dynamic code execution in JS/TS

use super::{byte_col_to_utf16_col, walk_ast_bounded, AnalysisConfig, Finding, Rule, Severity};
use tree_sitter::Node;

pub struct JsDangerousEvalRule;

impl Rule for JsDangerousEvalRule {
    fn name(&self) -> &'static str {
        "javascript/dangerous-eval"
    }

    fn description(&self) -> &'static str {
        "Detects calls to eval() or the Function constructor which introduce dynamic code execution hazards (CWE-95)"
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
            let kind = node.kind();
            if kind == "call_expression" {
                if let Some(func) = node.child_by_field_name("function") {
                    if func.kind() == "identifier" {
                        if let Ok(name) = func.utf8_text(source_bytes) {
                            if name == "eval" || name == "Function" {
                                let start_pos = node.start_position();
                                let line_1 = start_pos.row + 1;
                                let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);
                                let snippet = node
                                    .utf8_text(source_bytes)
                                    .ok()
                                    .map(|s| s.lines().next().unwrap_or(s).to_string());

                                findings.push(Finding {
                                    rule: "javascript/dangerous-eval".to_string(),
                                    severity: Severity::Error,
                                    message: format!(
                                        "Security Hazard (CWE-95): Unsafe dynamic execution via '{}()'",
                                        name
                                    ),
                                    line: line_1,
                                    column: col_1,
                                    snippet,
                                });
                            }
                        }
                    }
                }
            } else if kind == "new_expression" {
                if let Some(constructor) = node.child_by_field_name("constructor") {
                    if constructor.kind() == "identifier" {
                        if let Ok(name) = constructor.utf8_text(source_bytes) {
                            if name == "Function" {
                                let start_pos = node.start_position();
                                let line_1 = start_pos.row + 1;
                                let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);
                                let snippet = node
                                    .utf8_text(source_bytes)
                                    .ok()
                                    .map(|s| s.lines().next().unwrap_or(s).to_string());

                                findings.push(Finding {
                                    rule: "javascript/dangerous-eval".to_string(),
                                    severity: Severity::Error,
                                    message: "Security Hazard (CWE-95): Unsafe dynamic execution via 'new Function()' constructor".to_string(),
                                    line: line_1,
                                    column: col_1,
                                    snippet,
                                });
                            }
                        }
                    }
                }
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
    fn test_flag_eval() {
        let code = "const result = eval(userCode);\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = JsDangerousEvalRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 1);
        assert!(findings[0].message.contains("eval"));
    }

    #[test]
    fn test_flag_new_function() {
        let code = "const fn = new Function('a', 'b', 'return a + b');\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = JsDangerousEvalRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 1);
        assert!(findings[0].message.contains("Function"));
    }
}
