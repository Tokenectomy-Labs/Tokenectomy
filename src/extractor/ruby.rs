use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct RubyTraceParser;

static RUBY_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // app/controllers/orders_controller.rb:42:in `create'
    // /app/services/payment_service.rb:18:in 'charge'
    Regex::new(r#"(?:^|\s+|\()([a-zA-Z0-9_\-/\.]+\.rb):(\d+)(?::in|:|\))"#).unwrap()
});

impl TraceParser for RubyTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains(".rb:") && (log.contains(":in `") || log.contains(":in '") || log.contains("gems/") || log.contains("Error") || log.contains("Exception"))
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in RUBY_LOC_REGEX.captures_iter(log) {
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
    fn test_ruby_trace_parser() {
        let parser = RubyTraceParser;
        let trace = r#"
NoMethodError: undefined method `charge' for nil:NilClass
  app/controllers/orders_controller.rb:42:in `create'
  app/models/order.rb:15:in `process!'
  gems/activerecord-7.0.0/lib/active_record/persistence.rb:50:in `save'
"#;
        assert!(parser.detect(trace));
        let locs = parser.extract_locations(trace);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].file, "app/controllers/orders_controller.rb");
        assert_eq!(locs[0].line, 42);
        assert_eq!(locs[1].file, "app/models/order.rb");
        assert_eq!(locs[1].line, 15);
    }
}
