use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct RustTraceParser;

static RUST_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([a-zA-Z0-9_/\.\-]+\.rs):(\d+)").unwrap()
});

impl TraceParser for RustTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("panicked at") || log.contains(".rs:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in RUST_LOC_REGEX.captures_iter(log) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect() {
        let parser = RustTraceParser;
        assert!(parser.detect("thread 'main' panicked at src/main.rs:10:5"));
    }

    #[test]
    fn test_extract() {
        let parser = RustTraceParser;
        let log = "thread 'main' panicked at src/main.rs:10:5";
        let locs = parser.extract_locations(log);
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].file, "src/main.rs");
        assert_eq!(locs[0].line, 10);
    }
}
