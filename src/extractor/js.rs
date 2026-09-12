use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct JsTraceParser;

static JS_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // at Object.<anonymous> (/path/to/app.ts:10:15)
    // at /path/to/app.tsx:10:15
    // at file:///path/to/app.js:10:15
    // at webpack:///./src/components/Button.tsx:25:3
    Regex::new(r#"(?:\(|at\s+)(?:file://|webpack:///)?([a-zA-Z0-9_@\-/\.]+\.(?:m?js|cjs|ts|tsx|jsx|mts|cts)):(\d+)"#).unwrap()
});

impl TraceParser for JsTraceParser {
    fn detect(&self, log: &str) -> bool {
        (log.contains("Error:")
            || log.contains("TypeError:")
            || log.contains("ReferenceError:")
            || log.contains("SyntaxError:")
            || log.contains("UnhandledPromiseRejection"))
            && (log.contains("at ") || log.contains(".js:") || log.contains(".ts:") || log.contains(".tsx:") || log.contains(".jsx:"))
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in JS_LOC_REGEX.captures_iter(log) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoped_package_and_webpack_trace() {
        let parser = JsTraceParser;
        let trace = r#"
TypeError: Cannot read properties of null (reading 'render')
    at renderButton (webpack:///./packages/@org/ui/src/Button.tsx:45:12)
    at Object.<anonymous> (node_modules/react-dom/cjs/react-dom.production.min.js:12:34)
    at Module._compile (/app/dist/bundle.js:100:5)
"#;
        assert!(parser.detect(trace));
        let locs = parser.extract_locations(trace);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].file, "./packages/@org/ui/src/Button.tsx");
        assert_eq!(locs[0].line, 45);
        assert_eq!(locs[1].file, "/app/dist/bundle.js");
        assert_eq!(locs[1].line, 100);
    }
}
