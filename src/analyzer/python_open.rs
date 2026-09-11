// src/analyzer/python_open.rs — AST Rule for detecting unclosed open() calls in Python for Razor

use super::{byte_col_to_utf16_col, walk_ast_bounded, AnalysisConfig, Finding, Rule, Severity};
use std::collections::HashSet;
use tree_sitter::Node;

pub struct PythonUnclosedOpenRule;

impl Rule for PythonUnclosedOpenRule {
    fn name(&self) -> &'static str {
        "python/unclosed-open"
    }

    fn description(&self) -> &'static str {
        "Detects calls to open() that are not managed by a with statement, contextlib.closing, or try/finally: close()"
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

        // #14: Scope resolution / Name binding check for shadowed `open`
        let shadowed_open_scopes = collect_shadowed_open_scopes(root, source_bytes, config)?;

        // #12: Traverse AST with depth & node limits
        walk_ast_bounded(root, config.max_depth, config.max_ast_nodes, |node, _depth| {
            if node.kind() == "call" {
                if is_builtin_open_call(node, source_bytes, &shadowed_open_scopes) {
                    // #15: Verify if properly managed by with, closing(), or try/finally
                    if !is_properly_managed(node, source_bytes) {
                        let start_pos = node.start_position();
                        let line_1 = start_pos.row + 1;
                        // #16: Convert tree-sitter byte column to UTF-16 code units (LSP standard)
                        let col_1 = byte_col_to_utf16_col(source_text, start_pos.row, start_pos.column);

                        let snippet = node
                            .utf8_text(source_bytes)
                            .ok()
                            .map(|s| s.lines().next().unwrap_or(s).to_string());

                        findings.push(Finding {
                            rule: "python/unclosed-open".to_string(),
                            severity: Severity::Warning,
                            message: "Resource leak hazard: 'open()' called without 'with' statement, 'contextlib.closing', or 'try ... finally: close()'".to_string(),
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

/// #14: Detects scopes where `open` has been shadowed (e.g. `def open(...)` or `open = ...`)
fn collect_shadowed_open_scopes(
    root: &Node,
    source: &[u8],
    config: &AnalysisConfig,
) -> Result<HashSet<usize>, String> {
    let mut shadowed_scopes = HashSet::new();

    walk_ast_bounded(root, config.max_depth, config.max_ast_nodes, |node, _depth| {
        // Check `def open(...)`
        if node.kind() == "function_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = name_node.utf8_text(source) {
                    if name == "open" {
                        if let Some(parent) = node.parent() {
                            shadowed_scopes.insert(parent.id());
                        }
                    }
                }
            }
        }
        // Check `open = ...`
        if node.kind() == "assignment" {
            if let Some(left) = node.child_by_field_name("left") {
                if left.kind() == "identifier" {
                    if let Ok(name) = left.utf8_text(source) {
                        if name == "open" {
                            if let Some(parent) = node.parent() {
                                shadowed_scopes.insert(parent.id());
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(shadowed_scopes)
}

/// Determines if a call node is specifically targeting the builtin `open()`
fn is_builtin_open_call(
    call_node: &Node,
    source: &[u8],
    shadowed_scopes: &HashSet<usize>,
) -> bool {
    let func = match call_node.child_by_field_name("function") {
        Some(f) => f,
        None => return false,
    };

    if func.kind() != "identifier" {
        return false;
    }

    if let Ok(name) = func.utf8_text(source) {
        if name != "open" {
            return false;
        }
    } else {
        return false;
    }

    // #14: Check if any ancestor scope shadows `open`
    let mut curr = call_node.parent();
    while let Some(parent) = curr {
        if shadowed_scopes.contains(&parent.id()) {
            return false;
        }
        curr = parent.parent();
    }

    true
}

/// #15: Verifies whether the open() call is properly managed:
/// - In a `with` statement
/// - Wrapped in `contextlib.closing(...)`
/// - Assigned to variable and closed in `try ... finally: var.close()`
fn is_properly_managed(call_node: &Node, source: &[u8]) -> bool {
    // 1. Is it enclosed directly in `with_statement` / `with_item`?
    let mut probe = *call_node;
    while let Some(parent) = probe.parent() {
        if parent.kind() == "with_item" || parent.kind() == "with_statement" {
            return true;
        }
        probe = parent;
    }

    // 2. Is it wrapped in `contextlib.closing(open(...))` or `closing(open(...))`?
    if let Some(parent) = call_node.parent() {
        if parent.kind() == "argument_list" {
            if let Some(outer_call) = parent.parent() {
                if outer_call.kind() == "call" {
                    if let Some(func) = outer_call.child_by_field_name("function") {
                        if let Ok(func_name) = func.utf8_text(source) {
                            if func_name.ends_with("closing") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Is it assigned to a variable and closed in `try ... finally`?
    let mut bound_var: Option<String> = None;
    let mut curr = *call_node;

    while let Some(parent) = curr.parent() {
        if parent.kind() == "assignment" {
            if let Some(left) = parent.child_by_field_name("left") {
                if left.kind() == "identifier" {
                    if let Ok(name) = left.utf8_text(source) {
                        bound_var = Some(name.to_string());
                    }
                }
            }
            break;
        }
        if parent.kind() == "function_definition" || parent.kind() == "module" {
            break;
        }
        curr = parent;
    }

    if let Some(ref var_name) = bound_var {
        let mut scope_probe = call_node.parent();
        while let Some(parent) = scope_probe {
            if parent.kind() == "try_statement" {
                if has_matching_finally_close(&parent, var_name, source) {
                    return true;
                }
            }
            if parent.kind() == "block" || parent.kind() == "module" {
                if check_subsequent_try_finally(&parent, call_node, var_name, source) {
                    return true;
                }
            }
            scope_probe = parent.parent();
        }
    }

    false
}

fn has_matching_finally_close(try_node: &Node, var_name: &str, source: &[u8]) -> bool {
    let mut cursor = try_node.walk();
    for child in try_node.children(&mut cursor) {
        if child.kind() == "finally_clause" {
            return contains_close_call(&child, var_name, source);
        }
    }
    false
}

fn check_subsequent_try_finally(
    block_node: &Node,
    call_node: &Node,
    var_name: &str,
    source: &[u8],
) -> bool {
    let mut cursor = block_node.walk();
    let mut found_call = false;

    for child in block_node.children(&mut cursor) {
        if !found_call {
            if child.start_byte() <= call_node.start_byte() && child.end_byte() >= call_node.end_byte() {
                found_call = true;
            }
            continue;
        }

        if child.kind() == "try_statement" {
            if has_matching_finally_close(&child, var_name, source) {
                return true;
            }
        }
    }

    false
}

fn contains_close_call(node: &Node, var_name: &str, source: &[u8]) -> bool {
    contains_close_call_bounded(node, var_name, source, 0, 32)
}

fn contains_close_call_bounded(
    node: &Node,
    var_name: &str,
    source: &[u8],
    current_depth: usize,
    max_depth: usize,
) -> bool {
    if current_depth > max_depth {
        return false;
    }
    let expected = format!("{}.close", var_name);
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "call" {
            if let Some(func) = child.child_by_field_name("function") {
                if let Ok(text) = func.utf8_text(source) {
                    if text == expected {
                        return true;
                    }
                }
            }
        }
        if contains_close_call_bounded(&child, var_name, source, current_depth + 1, max_depth) {
            return true;
        }
    }

    false
}
