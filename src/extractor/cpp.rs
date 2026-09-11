use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct CppTraceParser;

static CPP_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // #0 0x555555555149 in func() src/core/tensor.cpp:88
    // or src/main.cpp:24:15: error:
    Regex::new(r"(?:at\s+|\s+)([a-zA-Z0-9_\-/\.]+\.(?:cpp|cc|cxx|c|hpp|h)):(\d+)").unwrap()
});

impl TraceParser for CppTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("AddressSanitizer")
            || log.contains("Segmentation fault")
            || log.contains("core dumped")
            || log.contains(".cpp:")
            || log.contains(".cc:")
            || log.contains(".cxx:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in CPP_LOC_REGEX.captures_iter(log) {
            if let (Some(file), Some(line_str)) = (cap.get(1), cap.get(2)) {
                let file_str = file.as_str();
                if super::is_framework_noise(file_str) {
                    continue;
                }

                if let Ok(line) = line_str.as_str().parse::<usize>() {
                    locations.push(CodeLocation {
                        file: file_str.to_string(),
                        line,
                    });
                }
            }
        }
        locations
    }
}
