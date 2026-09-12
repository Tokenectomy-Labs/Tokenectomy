use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct PythonTraceParser;

static PYTHON_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // 1. Standard Python traceback: File "/path/to/file.py", line 42
    // 2. Pytest / terminal direct location: /path/to/file.py:42
    Regex::new(r#"(?:File\s+"([^"]+\.(?:py|pyi|pyx))",\s+line\s+(\d+)|(?:^|\s+|at\s+)([a-zA-Z0-9_\-/\.]+\.(?:py|pyi|pyx)):(\d+))"#).unwrap()
});

impl TraceParser for PythonTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Traceback (most recent call last):")
            || log.contains("File \"")
            || log.contains(".py\", line ")
            || log.contains(".py:")
            || log.contains(".pyi:")
            || log.contains(".pyx:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in PYTHON_LOC_REGEX.captures_iter(log) {
            let (file_opt, line_opt) = if let (Some(f), Some(l)) = (cap.get(1), cap.get(2)) {
                (Some(f.as_str()), l.as_str().parse::<usize>().ok())
            } else if let (Some(f), Some(l)) = (cap.get(3), cap.get(4)) {
                (Some(f.as_str()), l.as_str().parse::<usize>().ok())
            } else {
                (None, None)
            };

            if let (Some(file), Some(line)) = (file_opt, line_opt) {
                if !super::is_framework_noise(file) {
                    locations.push(CodeLocation {
                        file: file.to_string(),
                        line,
                    });
                }
            }
        }
        locations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_traceback() {
        let parser = PythonTraceParser;
        let log = r#"Traceback (most recent call last):
  File "/app/services/auth.py", line 42, in login
    user = db.query()
  File "/app/env/lib/python3.11/site-packages/sqlalchemy/orm.py", line 150, in query
    return execute()
ZeroDivisionError: division by zero"#;

        assert!(parser.detect(log));
        let locs = parser.extract_locations(log);
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].file, "/app/services/auth.py");
        assert_eq!(locs[0].line, 42);
    }

    #[test]
    fn test_pytest_and_pyi_detection() {
        let parser = PythonTraceParser;
        let log = "FAILED tests/test_payment.pyx:88: AssertionError: Expected 200 got 500";
        assert!(parser.detect(log));
        let locs = parser.extract_locations(log);
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].file, "tests/test_payment.pyx");
        assert_eq!(locs[0].line, 88);
    }
}
