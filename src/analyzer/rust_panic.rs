// src/analyzer/rust_panic.rs — AST Rule for detecting explicit panics in Rust code

use super::{byte_col_to_utf16_col, walk_ast_bounded, AnalysisConfig, Finding, Rule, Severity};
use tree_sitter::Node;

pub struct RustExplicitPanicRule;

impl Rule for RustExplicitPanicRule {
    fn name(&self) -> &'static str {
        "rust/explicit-panic"
    }

    fn description(&self) -> &'static str {
        "Detects explicit invocations of panic!(), todo!(), or unimplemented!() macros in production code"
    }

    fn language(&self) -> &'static str {
        "rust"
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
            if node.kind() == "macro_invocation" {
                // In tree-sitter-rust, child(0) or field "macro" is the identifier or scoped_identifier
                let macro_name = node.child(0).and_then(|c| {
                    if c.kind() == "identifier" {
                        c.utf8_text(source_bytes).ok()
                    } else if c.kind() == "scoped_identifier" {
                        c.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source_bytes).ok())
                    } else {
                        None
                    }
                });

                if let Some(name) = macro_name {
                    if name == "panic" || name == "todo" || name == "unimplemented" {
                        let start_pos = node.start_position();
                        let line_1 = start_pos.row + 1;
                        let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);
                        let snippet = node
                            .utf8_text(source_bytes)
                            .ok()
                            .map(|s| s.lines().next().unwrap_or(s).to_string());

                        findings.push(Finding {
                            rule: "rust/explicit-panic".to_string(),
                            severity: Severity::Warning,
                            message: format!(
                                "Reliability Hazard: Explicit '{}!()' macro invocation in code",
                                name
                            ),
                            line: line_1,
                            column: col_1,
                            snippet,
                        });
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
    fn test_flag_rust_panic_and_todo() {
        let code = "fn run() {\n    panic!(\"crash\");\n    todo!();\n}\n";
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        let tree = parser.parse(code, None).unwrap();
        let config = AnalysisConfig::default();
        let rule = RustExplicitPanicRule;
        let findings = rule
            .check(&tree.root_node(), code.as_bytes(), code, &config)
            .unwrap();
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 2);
        assert!(findings[0].message.contains("panic"));
        assert_eq!(findings[1].line, 3);
        assert!(findings[1].message.contains("todo"));
    }
}
