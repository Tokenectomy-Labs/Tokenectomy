pub mod python;
pub mod js;
pub mod rust;

pub struct CodeLocation {
    pub file: String,
    pub line: usize,
}

pub trait TraceParser {
    fn detect(&self, log: &str) -> bool;
    fn extract_locations(&self, log: &str) -> Vec<CodeLocation>;
}

fn is_dependency_file(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    
    let ignores = [
        "node_modules",     // JS/Node
        "site-packages",    // Python/Django
        "dist-packages",    // Python
        "venv",             // Python VirtualEnv
        ".venv",            // Python VirtualEnv
        "lib/python",       // Python built-ins
        ".cargo/registry",  // Rust
        ".rustup",          // Rust toolchain
        "vendor",           // PHP/Go/Ruby
        "gems",             // Ruby
    ];

    for ignore in ignores.iter() {
        if lower_path.contains(ignore) {
            return true;
        }
    }
    false
}

pub fn extract_context(log: &str, context_lines: usize, strict_cwd: bool) -> (String, Vec<String>) {
    let parsers: Vec<Box<dyn TraceParser>> = vec![
        Box::new(rust::RustTraceParser),
        Box::new(python::PythonTraceParser),
        Box::new(js::JsTraceParser),
    ];

    let mut context_output = String::new();
    let mut extracted_files = Vec::new();
    let current_dir = std::env::current_dir().unwrap_or_default();

    for parser in parsers {
        if parser.detect(log) {
            let locations = parser.extract_locations(log);
            for loc in locations {
                // Filter cerdas: Abaikan file internal framework/database
                if is_dependency_file(&loc.file) {
                    log::debug!("Ignoring framework/dependency file: {}", loc.file);
                    continue;
                }

                // Security: Strict CWD check for MCP mode
                if strict_cwd {
                    let path = std::path::Path::new(&loc.file);
                    let abs_path = if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        current_dir.join(path)
                    };
                    if !abs_path.starts_with(&current_dir) {
                        log::debug!("Security Block: File outside CWD ignored: {}", loc.file);
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(&loc.file) {
                    extracted_files.push(loc.file.clone());
                    
                    let lines: Vec<&str> = content.lines().collect();
                    let start = loc.line.saturating_sub(context_lines).saturating_sub(1);
                    let end = (loc.line + context_lines).min(lines.len());
                    
                    context_output.push_str(&format!("--- {} (Lines {}-{}) ---\n", loc.file, start + 1, end));
                    for (i, line) in lines.iter().enumerate().take(end).skip(start) {
                        context_output.push_str(&format!("{} | {}\n", i + 1, line));
                    }
                    context_output.push_str("\n");
                }
            }
            break;
        }
    }

    (context_output, extracted_files)
}
