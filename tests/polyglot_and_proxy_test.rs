use tokenectomy::extractor::TraceParser;
use tokenectomy::extractor::{go::GoTraceParser, java::JavaTraceParser, cpp::CppTraceParser, php::PhpTraceParser, js::JsTraceParser};
use tokenectomy::proxy::sanitize_prompt_payload;

#[test]
fn test_go_trace_parser() {
    let parser = GoTraceParser;
    let go_panic = r#"
panic: runtime error: index out of range [3] with length 2

goroutine 1 [running]:
main.calculateTotal(0xc0000a4000, 0x3, 0x3)
	/app/src/billing/calc.go:42 +0x3f
main.main()
	/app/src/cmd/main.go:15 +0x2b
"#;
    assert!(parser.detect(go_panic));
    let locs = parser.extract_locations(go_panic);
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].file, "/app/src/billing/calc.go");
    assert_eq!(locs[0].line, 42);
    assert_eq!(locs[1].file, "/app/src/cmd/main.go");
    assert_eq!(locs[1].line, 15);
}

#[test]
fn test_java_trace_parser() {
    let parser = JavaTraceParser;
    let java_trace = r#"
Exception in thread "main" java.lang.NullPointerException: Cannot invoke method on null
	at com.example.service.PaymentService.processOrder(PaymentService.java:55)
	at com.example.controller.OrderController.handleCheckout(OrderController.kt:28)
	at org.springframework.web.servlet.DispatcherServlet.doDispatch(DispatcherServlet.java:1089)
"#;
    assert!(parser.detect(java_trace));
    let locs = parser.extract_locations(java_trace);
    assert!(locs.len() >= 2);
    assert_eq!(locs[0].file, "PaymentService.java");
    assert_eq!(locs[0].line, 55);
    assert_eq!(locs[1].file, "OrderController.kt");
    assert_eq!(locs[1].line, 28);
}

#[test]
fn test_cpp_trace_parser() {
    let parser = CppTraceParser;
    let cpp_trace = r#"
==12345==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x602000000014
READ of size 4 at 0x602000000014 thread T0
    #0 0x555555555149 in process_tensor(float const*, int) src/core/tensor.cpp:88
    #1 0x55555555518b in main src/main.cpp:24
"#;
    assert!(parser.detect(cpp_trace));
    let locs = parser.extract_locations(cpp_trace);
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].file, "src/core/tensor.cpp");
    assert_eq!(locs[0].line, 88);
    assert_eq!(locs[1].file, "src/main.cpp");
    assert_eq!(locs[1].line, 24);
}

#[test]
fn test_php_trace_parser() {
    let parser = PhpTraceParser;
    let php_trace = r#"
Fatal error: Uncaught TypeError: Argument 1 passed to App\Auth::login() must be of the type string, null given in /var/www/src/Auth.php:73
Stack trace:
#0 /var/www/src/Controller.php(32): App\Auth->login()
#1 /var/www/vendor/laravel/framework/src/Illuminate/Routing/Controller.php(54): App\Controller->handle()
"#;
    assert!(parser.detect(php_trace));
    let locs = parser.extract_locations(php_trace);
    assert_eq!(locs.len(), 3);
    assert_eq!(locs[0].file, "/var/www/src/Auth.php");
    assert_eq!(locs[0].line, 73);
    assert_eq!(locs[1].file, "/var/www/src/Controller.php");
    assert_eq!(locs[1].line, 32);
    assert!(tokenectomy::extractor::is_dependency_file(&locs[2].file));
    assert!(!tokenectomy::extractor::is_dependency_file(&locs[0].file));
}

#[test]
fn test_js_ts_extended_parser() {
    let parser = JsTraceParser;
    let ts_trace = r#"
TypeError: Cannot read properties of undefined (reading 'userId')
    at AuthHandler.verifyToken (/app/src/auth/handler.ts:42:18)
    at Object.<anonymous> (/app/src/components/App.tsx:99:12)
    at Module._compile (node_modules/ts-node/dist/index.js:85:10)
"#;
    assert!(parser.detect(ts_trace));
    let locs = parser.extract_locations(ts_trace);
    assert!(locs.iter().any(|l| l.file == "/app/src/auth/handler.ts" && l.line == 42));
    assert!(locs.iter().any(|l| l.file == "/app/src/components/App.tsx" && l.line == 99));
}

#[test]
fn test_proxy_sanitize_prompt_payload() {
    let inbound_json = serde_json::json!({
        "model": "gpt-4o",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful coding assistant."
            },
            {
                "role": "user",
                "content": "Here is the error from my Go server with API_KEY=sk-ant-api03-abcdef1234567890abcdef1234567890\n\npanic: runtime error: index out of range [3]\ngoroutine 1 [running]:\nmain.run()\n\t/usr/local/go/src/runtime/panic.go:40\n"
            }
        ]
    });

    let (sanitized_json, stats) = sanitize_prompt_payload(&inbound_json);
    
    let user_msg = sanitized_json["messages"][1]["content"].as_str().unwrap();
    assert!(!user_msg.contains("sk-ant-api03-abcdef1234567890abcdef1234567890"));
    assert!(user_msg.contains("[ANTHROPIC_API_KEY_REDACTED]") || user_msg.contains("REDACTED"));
    assert!(stats.secrets_redacted >= 1);
}

#[tokio::test]
async fn test_proxy_tcp_health_endpoint() {
    let bind_addr = "127.0.0.1:18095";
    tokio::spawn(async move {
        let _ = tokenectomy::proxy::run_reverse_proxy(bind_addr, "http://127.0.0.1:11434").await;
    });

    // Wait for server to bind
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let resp = client.get("http://127.0.0.1:18095/health").send().await.expect("Failed to connect to proxy");
    assert_eq!(resp.status(), 200);

    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "tokenectomy-gateway");
}

#[test]
fn test_scrub_prunes_dependency_frames_and_redacts_credentials() {
    let dirty_log = r#"
Error: Failed to connect to cluster
    at queryMaster (/app/src/db.ts:15:2)
    at node_modules/pg/lib/connection.js:84:11
    at node_modules/@prisma/client/runtime.js:200:5
    at site-packages/django/db/backends.py:40:1
Connection string: postgresql://admin:SuperSecretPass@cluster.internal:5432/main
AWS Key: AKIAIOSFODNN7EXAMPLE
    "#;

    let mut pruned = Vec::new();
    for line in dirty_log.lines() {
        if !tokenectomy::extractor::is_dependency_file(line) {
            pruned.push(line);
        }
    }
    let joined = pruned.join("\n");
    let safe = tokenectomy::redact::redact_secrets(&joined);

    assert!(!safe.contains("node_modules"));
    assert!(!safe.contains("site-packages"));
    assert!(!safe.contains("SuperSecretPass"));
    assert!(!safe.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(safe.contains("/app/src/db.ts:15:2"));
    assert!(safe.contains("[CONNECTION_STRING_REDACTED]"));
    assert!(safe.contains("[AWS_KEY_REDACTED]"));
}

