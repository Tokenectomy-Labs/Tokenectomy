use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct PhpTraceParser;

static PHP_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: in /var/www/src/Auth.php:73 or #0 /var/www/src/Controller.php(32):
    Regex::new(r"(?:in\s+|\s+)([a-zA-Z0-9_\-/\.]+\.php)(?::|\()(\d+)").unwrap()
});

impl TraceParser for PhpTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Fatal error:")
            || log.contains("Uncaught ")
            || (log.contains("Stack trace:") && log.contains(".php"))
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in PHP_LOC_REGEX.captures_iter(log) {
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
