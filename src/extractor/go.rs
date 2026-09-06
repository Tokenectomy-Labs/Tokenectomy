use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct GoTraceParser;

static GO_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: \t/path/to/main.go:42 +0x3f or /path/to/main.go:42
    Regex::new(r"([a-zA-Z0-9_\-/\.]+\.go):(\d+)").unwrap()
});

impl TraceParser for GoTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("panic:") || log.contains("goroutine ") || log.contains(".go:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in GO_LOC_REGEX.captures_iter(log) {
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
