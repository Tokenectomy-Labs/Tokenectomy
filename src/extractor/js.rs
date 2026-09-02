use super::{CodeLocation, TraceParser};
use regex::Regex;

pub struct JsTraceParser;

impl TraceParser for JsTraceParser {
    fn detect(&self, log: &str) -> bool {
        // Node.js or browser trace
        log.contains("Error:") && (log.contains("at ") || log.contains(".js:"))
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        //    at Object.<anonymous> (/path/to/app.js:10:15)
        // or at /path/to/app.js:10:15
        let re = Regex::new(r"(?:\(|at\s+)([a-zA-Z0-9_/\.\-]+\.m?js):(\d+)").unwrap();
        
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
