use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct JavaTraceParser;

static JAVA_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: at com.example.MyClass.method(MyFile.java:42) or (MyFile.kt:15)
    Regex::new(r"\(([a-zA-Z0-9_\-/\.]+\.(?:java|kt)):(\d+)\)").unwrap()
});

impl TraceParser for JavaTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Exception") || log.contains("\tat ") || log.contains("Caused by:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in JAVA_LOC_REGEX.captures_iter(log) {
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
