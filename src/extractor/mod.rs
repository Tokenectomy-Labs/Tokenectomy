pub mod python;
pub mod js;
pub mod rust;
pub mod go;
pub mod java;
pub mod cpp;
pub mod php;
pub mod csharp;
pub mod ruby;

pub struct CodeLocation {
    pub file: String,
    pub line: usize,
}

pub trait TraceParser: Send + Sync {
    fn detect(&self, log: &str) -> bool;
    fn extract_locations(&self, log: &str) -> Vec<CodeLocation>;
}

/// Checks whether `component` appears in `text` enclosed by path or word boundaries
/// (e.g. '/', '\', whitespace, quotes, parentheses, colon).
/// Prevents false positives like 'vendor_portal' matching 'vendor', or 'chicago/src' matching 'go/src'.
pub fn has_path_component(text: &str, component: &str) -> bool {
    let bytes = text.as_bytes();
    let comp_bytes = component.as_bytes();
    if comp_bytes.is_empty() || bytes.len() < comp_bytes.len() {
        return false;
    }

    let is_boundary = |b: u8| -> bool {
        b == b'/'
            || b == b'\\'
            || b == b' '
            || b == b'\t'
            || b == b'"'
            || b == b'\''
            || b == b'('
            || b == b')'
            || b == b'['
            || b == b']'
            || b == b':'
            || b == b'\r'
            || b == b'\n'
    };

    let mut start = 0;
    while let Some(pos) = text[start..].find(component) {
        let idx = start + pos;
        let before_ok = if idx == 0 {
            true
        } else {
            is_boundary(bytes[idx - 1])
        };

        let after_idx = idx + comp_bytes.len();
        let after_ok = if after_idx == bytes.len() {
            true
        } else {
            is_boundary(bytes[after_idx])
        };

        if before_ok && after_ok {
            return true;
        }

        start = idx + 1;
    }

    false
}

/// Determines whether a file path or stack trace line represents framework, runtime, or dependency noise.
pub fn is_framework_noise(line_or_path: &str) -> bool {
    let lower = line_or_path.to_lowercase();
    let trimmed = line_or_path.trim_start();
    let path_norm = lower.replace('\\', "/");

    // 1. Exact path directory component matches (prevents false positives on 'vendor_portal', 'chicago/src', 'inventory')
    let component_ignores = [
        "node_modules",        // JS/Node/TS
        "site-packages",       // Python
        "dist-packages",       // Python
        "venv",                // Python VirtualEnv
        ".venv",               // Python VirtualEnv
        "vendor",              // PHP/Go/Ruby dependencies
        "gems",                // Ruby gems
        "__pycache__",          // Python compiled bytecode
        "vcpkg_installed",     // C++ vcpkg
        ".gradle",             // Java Gradle cache
        ".cargo/registry",     // Rust
        ".rustup",             // Rust toolchain
        "pkg/mod",             // Go modules cache
        "go/src",              // Go standard library
        ".m2/repository",      // Java Maven repo
        "usr/include",         // C/C++ system headers
        "usr/lib",             // C/C++ system libraries
        "target/debug/build",  // Rust build scripts
        "lib/python",          // Python built-ins
        "internal/modules",    // Node.js internal CJS/ESM loader
        "rustc",               // Rust standard library / compiler frames
    ];

    for ignore in component_ignores.iter() {
        if has_path_component(&path_norm, ignore) {
            return true;
        }
    }

    // 2. Specific runtime protocol prefixes and markers
    let runtime_markers = [
        "node:internal/",      // Node.js internal runtime
        "<frozen ",            // Python internal frozen modules & importlib
        "asyncio/base_events", // Python asyncio internals
        "asyncio/events.py",   // Python asyncio internals
        "starlette/routing",   // Starlette / FastAPI routing frames
        "uvicorn/protocols/",  // Uvicorn server frames
        "gunicorn/workers/",   // Gunicorn worker frames
        "build/glibc-",        // Glibc internals
        "system.private.corelib", // .NET CoreLib
        "microsoft.aspnetcore.",  // ASP.NET Core
    ];

    for marker in runtime_markers.iter() {
        if path_norm.contains(marker) {
            return true;
        }
    }

    // 2. Java / Kotlin enterprise framework stack frames (Spring Boot, Tomcat, Hibernate, Netty, Undertow, JDK)
    let java_framework_prefixes = [
        "at org.springframework.",
        "at org.apache.catalina.",
        "at org.apache.tomcat.",
        "at org.apache.coyote.",
        "at org.hibernate.",
        "at org.eclipse.jetty.",
        "at jakarta.servlet.",
        "at javax.servlet.",
        "at io.netty.",
        "at io.undertow.",
        "at com.zaxxer.hikari.",
        "at java.base/",
        "at java.lang.reflect.",
        "at jdk.internal.",
        "at sun.reflect.",
        "at kotlinx.coroutines.",
        "at org.junit.",
        "at System.",
        "at Microsoft.AspNetCore.",
    ];
    for prefix in java_framework_prefixes.iter() {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    if trimmed.starts_with("... ") && trimmed.ends_with("common frames omitted") {
        return true;
    }

    // 3. C / C++ ASan, GDB, glibc, libstdc++ runtime frames
    let cpp_runtime_markers = [
        "__libc_start_main",
        "__libc_start_call_main",
        "libc-start.c",
        "libc_start_call_main.h",
        "/lib/x86_64-linux-gnu/libc.so",
        "/lib/x86_64-linux-gnu/libasan.so",
        "/usr/lib/x86_64-linux-gnu/libasan.so",
        "/usr/lib/x86_64-linux-gnu/libstdc++.so",
        "libasan.so",
        "libstdc++.so",
        "__sanitizer::",
        "__asan::",
        "__asan_",
        "(/lib/x86_64-linux-gnu/",
        "(/usr/lib/x86_64-linux-gnu/",
        "sysdeps/nptl/",
    ];
    for marker in cpp_runtime_markers.iter() {
        if line_or_path.contains(marker) {
            return true;
        }
    }
    if trimmed.contains(" in _start (") || trimmed.ends_with(" in _start") {
        return true;
    }

    // 4. Go runtime goroutine idle states & internal scheduler frames
    let go_idle_markers = [
        "[force gc (idle)]",
        "[GC sweep wait]",
        "[GC scavenge wait]",
        "[finalizer wait]",
        "[scavenge wait]",
        "[select (no cases)]",
    ];
    for marker in go_idle_markers.iter() {
        if line_or_path.contains(marker) {
            return true;
        }
    }

    let go_runtime_frames = [
        "runtime.gopark(",
        "runtime.forcegchelper(",
        "runtime.goexit(",
        "runtime.gcBgMarkWorker(",
        "runtime.bgsweep(",
        "runtime.bgscavenge(",
        "runtime.runfinq(",
    ];
    for frame in go_runtime_frames.iter() {
        if trimmed.starts_with(frame) {
            return true;
        }
    }

    false
}

/// Backward-compatible parallel-bridge shim for `is_dependency_file`.
/// Retains 100% compatibility with existing callers while delegating to `is_framework_noise`.
#[inline]
pub fn is_dependency_file(path: &str) -> bool {
    is_framework_noise(path)
}

/// Checks if a line or path is framework noise, incorporating user-defined custom patterns.
pub fn is_framework_noise_with_custom(line_or_path: &str, custom_noise: &[String]) -> bool {
    if is_framework_noise(line_or_path) {
        return true;
    }
    let lower = line_or_path.to_lowercase();
    for pat in custom_noise {
        if !pat.is_empty() && lower.contains(&pat.to_lowercase()) {
            return true;
        }
    }
    false
}

use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DroppedFramesSummary {
    pub total_dropped: usize,
    pub categories: BTreeMap<String, usize>,
}

impl DroppedFramesSummary {
    pub fn empty() -> Self {
        Self {
            total_dropped: 0,
            categories: BTreeMap::new(),
        }
    }

    /// Formats the dropped frames summary into a compact inline string, e.g.:
    /// "17 frames (node_modules/next: 14, node:internal: 3)"
    /// or "0 frames (lossless)"
    pub fn to_inline_summary(&self) -> String {
        if self.total_dropped == 0 {
            return "0 frames (lossless)".to_string();
        }
        let mut parts = Vec::new();
        for (cat, count) in &self.categories {
            parts.push(format!("{}: {}", cat, count));
        }
        format!("{} frames ({})", self.total_dropped, parts.join(", "))
    }
}

/// Classifies a framework/runtime noise line into its package or runtime origin identity.
/// Strictly respects `is_framework_noise_with_custom`: returns None if not noise.
pub fn classify_dropped_frame(line_or_path: &str, custom_noise: &[String]) -> Option<String> {
    if !is_framework_noise_with_custom(line_or_path, custom_noise) {
        return None;
    }

    let lower = line_or_path.to_lowercase();
    for pat in custom_noise {
        if !pat.is_empty() && lower.contains(&pat.to_lowercase()) {
            return Some(format!("custom:{}", pat));
        }
    }

    let trimmed = line_or_path.trim_start();
    let path_norm = lower.replace('\\', "/");

    // 1. Node modules package extraction (e.g. node_modules/next, node_modules/@prisma/client)
    if let Some(pos) = path_norm.find("node_modules/") {
        let after = &path_norm[pos + "node_modules/".len()..];
        let mut segments = after.split('/');
        if let Some(first) = segments.next() {
            if first.starts_with('@') {
                if let Some(second) = segments.next() {
                    return Some(format!("node_modules/{}/{}", first, second));
                }
            }
            return Some(format!("node_modules/{}", first));
        }
    }

    // 2. Python site-packages / dist-packages
    for prefix in &["site-packages/", "dist-packages/"] {
        if let Some(pos) = path_norm.find(prefix) {
            let after = &path_norm[pos + prefix.len()..];
            if let Some(first) = after.split('/').next() {
                let clean_pkg = first.trim_end_matches(".py");
                return Some(format!("site-packages/{}", clean_pkg));
            }
        }
    }

    // 3. Runtime markers
    if path_norm.contains("node:internal/") || path_norm.contains("internal/modules") {
        return Some("node:internal".to_string());
    }
    if path_norm.contains("<frozen ") {
        return Some("python:frozen".to_string());
    }
    if path_norm.contains("asyncio/") {
        return Some("python:asyncio".to_string());
    }
    if path_norm.contains("starlette/") {
        return Some("site-packages/starlette".to_string());
    }
    if path_norm.contains("uvicorn/") {
        return Some("site-packages/uvicorn".to_string());
    }
    if path_norm.contains("gunicorn/") {
        return Some("site-packages/gunicorn".to_string());
    }
    if path_norm.contains("lib/python") {
        return Some("python:stdlib".to_string());
    }
    if has_path_component(&path_norm, "venv") || has_path_component(&path_norm, ".venv") {
        return Some("python:venv".to_string());
    }

    // 4. Java / Kotlin
    if trimmed.starts_with("at org.springframework.") {
        return Some("org.springframework".to_string());
    }
    if trimmed.starts_with("at org.apache.") {
        return Some("org.apache".to_string());
    }
    if trimmed.starts_with("at org.hibernate.") {
        return Some("org.hibernate".to_string());
    }
    if trimmed.starts_with("at jakarta.") || trimmed.starts_with("at javax.") {
        return Some("java:servlet".to_string());
    }
    if trimmed.starts_with("at io.netty.") {
        return Some("io.netty".to_string());
    }
    if trimmed.starts_with("at java.base/") {
        return Some("java.base".to_string());
    }
    if trimmed.starts_with("at java.lang.reflect.")
        || trimmed.starts_with("at jdk.internal.")
        || trimmed.starts_with("at sun.reflect.")
    {
        return Some("java:internal".to_string());
    }
    if trimmed.starts_with("at com.zaxxer.hikari.") {
        return Some("hikari_cp".to_string());
    }
    if trimmed.starts_with("... ") && trimmed.ends_with("common frames omitted") {
        return Some("java:common_omitted".to_string());
    }

    // 5. .NET
    if path_norm.contains("system.private.corelib") || trimmed.starts_with("at System.") {
        return Some("dotnet:corelib".to_string());
    }
    if path_norm.contains("microsoft.aspnetcore.")
        || trimmed.starts_with("at Microsoft.AspNetCore.")
    {
        return Some("dotnet:aspnetcore".to_string());
    }

    // 6. C/C++
    if path_norm.contains("__sanitizer")
        || path_norm.contains("libasan")
        || path_norm.contains("__asan")
    {
        return Some("cpp:asan".to_string());
    }
    if path_norm.contains("libc") || path_norm.contains("glibc") {
        return Some("cpp:libc".to_string());
    }
    if path_norm.contains("libstdc++") {
        return Some("cpp:libstdc++".to_string());
    }

    // 7. Rust
    if let Some(pos) = path_norm.find(".cargo/registry/src/") {
        let after = &path_norm[pos + ".cargo/registry/src/".len()..];
        let parts: Vec<&str> = after.split('/').collect();
        if parts.len() >= 2 {
            let crate_dir = parts[1];
            let crate_name = crate_dir.split('-').next().unwrap_or(crate_dir);
            return Some(format!("cargo:{}", crate_name));
        }
        return Some("cargo:registry".to_string());
    }
    if path_norm.contains(".rustup") || path_norm.contains("rustc") {
        return Some("rust:std".to_string());
    }

    // 8. Go
    if path_norm.contains("go/src") {
        return Some("go:stdlib".to_string());
    }
    if let Some(pos) = path_norm.find("pkg/mod/") {
        let after = &path_norm[pos + "pkg/mod/".len()..];
        let mut segments = after.split('/');
        if let (Some(s1), Some(s2)) = (segments.next(), segments.next()) {
            return Some(format!("go:{}/{}", s1, s2));
        }
        return Some("go:pkg_mod".to_string());
    }
    if trimmed.starts_with("runtime.") {
        return Some("go:runtime".to_string());
    }

    // 9. Ruby
    if let Some(pos) = path_norm.find("gems/") {
        let after = &path_norm[pos + "gems/".len()..];
        if let Some(first) = after.split('/').next() {
            let gem_name = first.split('-').next().unwrap_or(first);
            return Some(format!("ruby:{}", gem_name));
        }
    }

    // 10. Fallback matching component_ignores
    let component_ignores = [
        "node_modules", "site-packages", "dist-packages", "venv", ".venv",
        "vendor", "gems", "__pycache__", "vcpkg_installed", ".gradle",
        ".cargo/registry", ".rustup", "pkg/mod", "go/src", ".m2/repository",
        "usr/include", "usr/lib", "target/debug/build", "lib/python",
        "internal/modules", "rustc"
    ];
    for ignore in component_ignores.iter() {
        if has_path_component(&path_norm, ignore) {
            return Some((*ignore).to_string());
        }
    }

    Some("framework_noise".to_string())
}

/// Surgically prunes framework noise with detailed statistics tracking dropped frame counts and identities.
pub fn prune_framework_noise_with_stats(
    raw: &str,
    custom_noise: &[String],
) -> (String, DroppedFramesSummary) {
    let mut cleaned_lines = Vec::new();
    let mut in_idle_goroutine = false;
    let mut total_dropped = 0;
    let mut categories: BTreeMap<String, usize> = BTreeMap::new();

    for line in raw.lines() {
        let trimmed = line.trim();

        // Detect Go goroutine block headers
        if trimmed.starts_with("goroutine ") {
            if trimmed.contains("[force gc (idle)]")
                || trimmed.contains("[GC sweep wait]")
                || trimmed.contains("[GC scavenge wait]")
                || trimmed.contains("[finalizer wait]")
                || trimmed.contains("[scavenge wait]")
                || trimmed.contains("[select (no cases)]")
            {
                in_idle_goroutine = true;
                total_dropped += 1;
                *categories.entry("go:idle_goroutines".to_string()).or_insert(0) += 1;
                continue;
            } else {
                in_idle_goroutine = false;
            }
        }

        // If inside an idle goroutine, skip lines until next non-indented block or empty line
        if in_idle_goroutine {
            if line.is_empty() {
                in_idle_goroutine = false;
            } else {
                total_dropped += 1;
                *categories.entry("go:idle_goroutines".to_string()).or_insert(0) += 1;
                continue;
            }
        }

        // Check and classify if the individual line is framework noise
        if let Some(category) = classify_dropped_frame(line, custom_noise) {
            total_dropped += 1;
            *categories.entry(category).or_insert(0) += 1;
            continue;
        }

        cleaned_lines.push(line);
    }

    (
        cleaned_lines.join("\n"),
        DroppedFramesSummary {
            total_dropped,
            categories,
        },
    )
}

/// Surgically prunes framework noise, internal runtime stack lines, and idle Go goroutines from a raw log.
pub fn prune_framework_noise(raw: &str) -> String {
    prune_framework_noise_with_stats(raw, &[]).0
}

/// Surgically prunes framework noise including user-defined custom noise patterns.
pub fn prune_framework_noise_with_custom(raw: &str, custom_noise: &[String]) -> String {
    prune_framework_noise_with_stats(raw, custom_noise).0
}

pub fn extract_context(log: &str, context_lines: usize, strict_cwd: bool) -> (String, Vec<String>) {
    let boundary = if strict_cwd {
        crate::workspace::WorkspaceBoundary::current().ok()
    } else {
        None
    };
    extract_context_with_boundary(log, context_lines, boundary.as_ref())
}

pub fn extract_context_with_boundary(
    log: &str,
    context_lines: usize,
    boundary: Option<&crate::workspace::WorkspaceBoundary>,
) -> (String, Vec<String>) {
    let parsers: Vec<Box<dyn TraceParser>> = vec![
        Box::new(rust::RustTraceParser),
        Box::new(python::PythonTraceParser),
        Box::new(js::JsTraceParser),
        Box::new(go::GoTraceParser),
        Box::new(java::JavaTraceParser),
        Box::new(cpp::CppTraceParser),
        Box::new(php::PhpTraceParser),
        Box::new(csharp::CSharpTraceParser),
        Box::new(ruby::RubyTraceParser),
    ];

    let mut context_output = String::new();
    let mut extracted_files = Vec::new();
    let mut seen_locations: std::collections::HashSet<(String, usize)> = std::collections::HashSet::new();

    for parser in parsers {
        if parser.detect(log) {
            let locations = parser.extract_locations(log);
            for loc in locations {
                // Deduplicate across polyglot parsers and repetitive trace frames
                if !seen_locations.insert((loc.file.clone(), loc.line)) {
                    continue;
                }

                // Filter cerdas: Abaikan file internal framework/library/dependency
                if is_dependency_file(&loc.file) {
                    log::debug!("Ignoring framework/dependency file: {}", loc.file);
                    continue;
                }

                // Security: Strict boundary check
                if let Some(b) = boundary {
                    if !b.is_safe(&loc.file) {
                        log::debug!("Security Block: File outside workspace boundary ignored: {}", loc.file);
                        continue;
                    }
                }

                let content_opt = if let Some(b) = boundary {
                    b.read(&loc.file).ok()
                } else {
                    std::fs::read_to_string(&loc.file).ok()
                };

                if let Some(content) = content_opt {
                    if !extracted_files.contains(&loc.file) {
                        extracted_files.push(loc.file.clone());
                    }
                    
                    let lines: Vec<&str> = content.lines().collect();
                    let start = loc.line.saturating_sub(context_lines).saturating_sub(1);
                    let end = (loc.line + context_lines).min(lines.len());
                    
                    context_output.push_str(&format!("--- {} (Lines {}-{}) ---\n", loc.file, start + 1, end));
                    for (i, line) in lines.iter().enumerate().take(end).skip(start) {
                        context_output.push_str(&format!("{} | {}\n", i + 1, line));
                    }
                    context_output.push_str("\n");
                }
            }
        }
    }

    (context_output, extracted_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropped_frames_classification_and_summary() {
        assert_eq!(
            classify_dropped_frame("    at loadComponents (/app/node_modules/next/dist/server/load-components.js:14:2)", &[]),
            Some("node_modules/next".to_string())
        );
        assert_eq!(
            classify_dropped_frame("    at client (/app/node_modules/@prisma/client/runtime/index.js:5:10)", &[]),
            Some("node_modules/@prisma/client".to_string())
        );
        assert_eq!(
            classify_dropped_frame("  File \"/app/.venv/lib/python3.11/site-packages/starlette/routing.py\", line 123, in app", &[]),
            Some("site-packages/starlette".to_string())
        );
        assert_eq!(
            classify_dropped_frame("    at processTicksAndRejections (node:internal/process/task_queues:95:5)", &[]),
            Some("node:internal".to_string())
        );
        assert_eq!(
            classify_dropped_frame("    at org.springframework.web.servlet.DispatcherServlet.doDispatch(DispatcherServlet.java:1062)", &[]),
            Some("org.springframework".to_string())
        );
        assert_eq!(
            classify_dropped_frame("    at checkoutHandler (/app/pages/api/checkout.ts:42:15)", &[]),
            None
        );

        let trace = "Error: Boom\n    at loadComponents (/app/node_modules/next/dist/server/load-components.js:14:2)\n    at render (/app/node_modules/next/dist/server/render.js:50:5)\n    at processTicksAndRejections (node:internal/process/task_queues:95:5)\n    at checkoutHandler (/app/pages/api/checkout.ts:42:15)";
        let (clean, summary) = prune_framework_noise_with_stats(trace, &[]);
        assert_eq!(summary.total_dropped, 3);
        assert_eq!(summary.categories.get("node_modules/next"), Some(&2));
        assert_eq!(summary.categories.get("node:internal"), Some(&1));
        assert_eq!(summary.to_inline_summary(), "3 frames (node:internal: 1, node_modules/next: 2)");
        assert!(clean.contains("checkoutHandler"));
        assert!(!clean.contains("node_modules/next"));

        let empty_summary = DroppedFramesSummary::empty();
        assert_eq!(empty_summary.to_inline_summary(), "0 frames (lossless)");
    }
}
