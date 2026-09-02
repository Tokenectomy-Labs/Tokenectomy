use super::{CodeLocation, TraceParser};
use regex::Regex;

pub struct PythonTraceParser;

impl TraceParser for PythonTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Traceback (most recent call last):") || log.contains("File \"")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        // File "/path/to/file.py", line 42, in <module>
        let re = Regex::new(r#"File\s+"([^"]+\.py)",\s+line\s+(\d+)"#).unwrap();
        
        for cap in re.captures_iter(log) {
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
