// src/analyzer/python_exec.rs — AST Rule for detecting dangerous dynamic execution in Python

use super::{byte_col_to_utf16_col, walk_ast_bounded, AnalysisConfig, Finding, Rule, Severity};
use tree_sitter::Node;

pub struct PythonDangerousExecRule;

impl Rule for PythonDangerousExecRule {
    fn name(&self) -> &'static str {
        "python/dangerous-exec-eval"
    }

    fn description(&self) -> &'static str {
        "Detects calls to built-in eval() or exec() which present code injection hazards (CWE-95)"
    }

    fn language(&self) -> &'static str {
        "python"
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
            if node.kind() == "call" {
                if let Some(func_node) = node.child_by_field_name("function") {
                    // Must be a bare identifier, NOT a method call like model.eval()
                    if func_node.kind() == "identifier" {
                        if let Ok(name) = func_node.utf8_text(source_bytes) {
                            if name == "eval" || name == "exec" {
                                let start_pos = node.start_position();
                                let line_1 = start_pos.row + 1;
                                let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);

                                let snippet = node
                                    .utf8_text(source_bytes)
                                    .ok()
                                    .map(|s| s.lines().next().unwrap_or(s).to_string());

                                findings.push(Finding {
                                    rule: "python/dangerous-exec-eval".to_string(),
                                    severity: Severity::Error,
                                    message: format!(
                                        "Security Hazard (CWE-95): Unsafe dynamic execution via built-in '{}()'",
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
    fn test_flag_raw_eval() {
        let code = "result = eval(user_input)\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = PythonDangerousExecRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 1);
        assert!(findings[0].message.contains("eval"));
    }

    #[test]
    fn test_pytorch_model_eval_not_flagged() {
        let code = "model.eval()\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = PythonDangerousExecRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert!(findings.is_empty(), "model.eval() must NOT be flagged");
    }
}
