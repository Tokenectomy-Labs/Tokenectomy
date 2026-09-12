use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct CSharpTraceParser;

static CSHARP_LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches:
    // in /app/src/Services/OrderService.cs:line 48
    // in C:\Users\App\Controllers\OrderController.cs:line 23
    Regex::new(r#"(?:in\s+|\s+)([a-zA-Z0-9_\-/\.\\:]+\.cs):line\s+(\d+)"#).unwrap()
});

impl TraceParser for CSharpTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains(".cs:line ")
            || (log.contains("Exception:") && log.contains(" in ") && log.contains(".cs"))
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in CSHARP_LOC_REGEX.captures_iter(log) {
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
    fn test_csharp_trace_parser() {
        let parser = CSharpTraceParser;
        let trace = r#"
System.NullReferenceException: Object reference not set to an instance of an object.
   at EnterpriseApp.Services.PaymentService.Charge(Decimal amount) in /app/src/PaymentService.cs:line 55
   at EnterpriseApp.Controllers.CheckoutController.Post() in /app/src/CheckoutController.cs:line 28
   at Microsoft.AspNetCore.Mvc.Infrastructure.ActionMethodExecutor.SyncActionResultExecutor.Execute() in Microsoft.AspNetCore.Mvc.Core.dll:line 100
"#;
        assert!(parser.detect(trace));
        let locs = parser.extract_locations(trace);
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].file, "/app/src/PaymentService.cs");
        assert_eq!(locs[0].line, 55);
        assert_eq!(locs[1].file, "/app/src/CheckoutController.cs");
        assert_eq!(locs[1].line, 28);
    }
}
