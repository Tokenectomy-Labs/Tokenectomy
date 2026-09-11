use super::{CodeLocation, TraceParser};
use regex::Regex;
use std::sync::LazyLock;

pub struct JavaTraceParser;

static JAVA_FRAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Matches: at com.example.MyClass.method(MyFile.java:42) or (MyFile.kt:15)
    Regex::new(r"(?:at\s+([a-zA-Z0-9_\-/\.$]+)\s*)?\(([a-zA-Z0-9_\-/\.]+\.(?:java|kt)):(\d+)\)").unwrap()
});

fn is_framework_class(class_path: &str) -> bool {
    let prefixes = [
        "org.springframework.",
        "org.apache.",
        "org.hibernate.",
        "org.eclipse.jetty.",
        "jakarta.",
        "javax.",
        "io.netty.",
        "io.undertow.",
        "com.zaxxer.hikari.",
        "java.base/",
        "java.lang.reflect.",
        "jdk.internal.",
        "sun.reflect.",
        "sun.misc.",
        "kotlinx.coroutines.",
        "org.junit.",
        "org.testng.",
        "org.mockito.",
    ];
    for p in prefixes {
        if class_path.starts_with(p) {
            return true;
        }
    }
    false
}

impl TraceParser for JavaTraceParser {
    fn detect(&self, log: &str) -> bool {
        log.contains("Exception") || log.contains("\tat ") || log.contains("Caused by:")
    }

    fn extract_locations(&self, log: &str) -> Vec<CodeLocation> {
        let mut locations = Vec::new();
        for cap in JAVA_FRAME_REGEX.captures_iter(log) {
            if let Some(class_method) = cap.get(1) {
                if is_framework_class(class_method.as_str()) {
                    continue;
                }
            }

            if let (Some(file), Some(line_str)) = (cap.get(2), cap.get(3)) {
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
