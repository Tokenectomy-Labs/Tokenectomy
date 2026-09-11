use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct PythonTraceParser;

static PYTHON_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: File "/path/to/file.py", line 42, in <module>
    Regex::new(r#"File\s+"([^"]+\.py)",\s+line\s+(\d+)"#).unwrap()
});

impl TraceParser for PythonTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Traceback (most recent call last):") || log.contains("File \"")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in PYTHON_LOC_REGEX.captures_iter(log) {
            if let (Some(file), Some(line_str)) = (cap.get(1), cap.get(2)) {
                if let Ok(line) = line_str.as_str().parse::<usize>() {
                    locations.push(CodeLocation {
                        file: file.as_str().to_string(),
                        line,
                    });
                }
            }
        }
        locations
    }
}
