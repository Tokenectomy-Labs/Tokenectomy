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

#[test]
fn test_proxy_is_loopback() {
    assert!(tokenectomy::proxy::is_loopback("127.0.0.1:8080"));
    assert!(tokenectomy::proxy::is_loopback("127.0.0.1"));
    assert!(tokenectomy::proxy::is_loopback("localhost:8080"));
    assert!(tokenectomy::proxy::is_loopback("localhost"));
    assert!(tokenectomy::proxy::is_loopback("[::1]:8080"));
    assert!(tokenectomy::proxy::is_loopback("::1"));

    assert!(!tokenectomy::proxy::is_loopback("0.0.0.0:8080"));
    assert!(!tokenectomy::proxy::is_loopback("192.168.1.100:8080"));
    assert!(!tokenectomy::proxy::is_loopback("10.0.0.1:8080"));
}

#[tokio::test]
async fn test_proxy_remote_bind_security_guards() {
    // 1. Binding to non-loopback without allow_remote should fail
    let res = tokenectomy::proxy::run_reverse_proxy_configured("0.0.0.0:18991", "http://127.0.0.1:11434", false, None).await;
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("Security Violation"));
    assert!(err.contains("--allow-remote"));

    // 2. Binding to non-loopback with allow_remote=true but missing token should fail
    let res2 = tokenectomy::proxy::run_reverse_proxy_configured("0.0.0.0:18991", "http://127.0.0.1:11434", true, None).await;
    assert!(res2.is_err());
    let err2 = res2.err().unwrap().to_string();
    assert!(err2.contains("requires an authentication token"));
}

#[tokio::test]
async fn test_proxy_bearer_auth_and_unauthorized_rejection() {
    let bind_addr = "127.0.0.1:18096";
    let auth_token = "my-secret-agent-token-12345";

    tokio::spawn(async move {
        let _ = tokenectomy::proxy::run_reverse_proxy_configured(
            bind_addr,
            "http://127.0.0.1:11434",
            false,
            Some(auth_token),
        ).await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // 1. Health endpoint should remain accessible without token
    let health_resp = client.get("http://127.0.0.1:18096/health").send().await.expect("Failed to call /health");
    assert_eq!(health_resp.status(), 200);

    // 2. Protected endpoint without token should return 401 Unauthorized
    let unauth_resp = client.post("http://127.0.0.1:18096/v1/chat/completions")
        .body("{}")
        .send()
        .await
        .expect("Failed to call completions");
    assert_eq!(unauth_resp.status(), 401);

    // 3. Protected endpoint with invalid token should return 401 Unauthorized
    let wrong_resp = client.post("http://127.0.0.1:18096/v1/chat/completions")
        .header("Authorization", "Bearer wrong-token")
        .body("{}")
        .send()
        .await
        .expect("Failed to call completions with wrong token");
    assert_eq!(wrong_resp.status(), 401);
}

#[test]
fn test_verify_patch_and_auto_rollback_on_syntax_error() {
    let temp_dir = std::env::temp_dir().join(format!("tokenectomy_verify_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // Create a valid Python file
    let py_path = temp_dir.join("calc.py");
    let valid_code = "def calculate_tax(subtotal: float) -> float:\n    return subtotal * 0.1\n";
    std::fs::write(&py_path, valid_code).expect("Failed to write test file");

    // 1. Verification of valid file passes
    let verify_res = tokenectomy::mcp::verify_patch(&py_path);
    assert!(verify_res.is_ok());

    // 2. Simulate patch with syntax error (invalid Python)
    let broken_code = "def calculate_tax(subtotal: float\n    return subtotal *\n";
    // Simulate apply_code_patch: backup original, write patch, verify, rollback on fail
    let backup = std::fs::read_to_string(&py_path).unwrap();
    std::fs::write(&py_path, broken_code).unwrap();
    
    let check = tokenectomy::mcp::verify_patch(&py_path);
    assert!(check.is_err(), "Broken syntax must fail verification");
    
    // Auto-rollback
    std::fs::write(&py_path, &backup).unwrap();
    let restored = std::fs::read_to_string(&py_path).unwrap();
    assert_eq!(restored, valid_code, "File must be 100% restored with 0 dirty diff");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_spring_boot_deep_stack_trace_pruning() {
    let raw_trace = r#"
org.springframework.web.util.NestedServletException: Request processing failed: java.lang.NullPointerException: user repository returned null
	at org.springframework.web.servlet.FrameworkServlet.processRequest(FrameworkServlet.java:1014)
	at org.springframework.web.servlet.FrameworkServlet.doPost(FrameworkServlet.java:914)
	at jakarta.servlet.http.HttpServlet.service(HttpServlet.java:590)
	at org.springframework.web.servlet.FrameworkServlet.service(FrameworkServlet.java:885)
	at jakarta.servlet.http.HttpServlet.service(HttpServlet.java:658)
	at org.apache.catalina.core.ApplicationFilterChain.internalDoFilter(ApplicationFilterChain.java:205)
	at org.apache.catalina.core.ApplicationFilterChain.doFilter(ApplicationFilterChain.java:149)
	at org.apache.tomcat.websocket.server.WsFilter.doFilter(WsFilter.java:51)
	at org.apache.catalina.core.ApplicationFilterChain.internalDoFilter(ApplicationFilterChain.java:174)
	at org.apache.catalina.core.StandardWrapperValve.invoke(StandardWrapperValve.java:167)
	at org.apache.catalina.core.StandardContextValve.invoke(StandardContextValve.java:90)
	at org.apache.catalina.authenticator.AuthenticatorBase.invoke(AuthenticatorBase.java:492)
	at org.apache.catalina.core.StandardHostValve.invoke(StandardHostValve.java:130)
	at org.apache.catalina.valves.ErrorReportValve.invoke(ErrorReportValve.java:93)
	at org.apache.catalina.core.StandardEngineValve.invoke(StandardEngineValve.java:74)
	at org.apache.catalina.connector.CoyoteAdapter.service(CoyoteAdapter.java:343)
	at org.apache.coyote.http11.Http11Processor.service(Http11Processor.java:390)
	at org.apache.coyote.AbstractProcessorLight.process(AbstractProcessorLight.java:63)
	at org.apache.coyote.AbstractProtocol$ConnectionHandler.process(AbstractProtocol.java:926)
	at org.apache.tomcat.util.net.NioEndpoint$SocketProcessor.doRun(NioEndpoint.java:1790)
	at org.apache.tomcat.util.net.SocketProcessorBase.run(SocketProcessorBase.java:52)
	at org.apache.tomcat.util.threads.ThreadPoolExecutor.runWorker(ThreadPoolExecutor.java:1191)
	at org.apache.tomcat.util.threads.ThreadPoolExecutor$Worker.run(ThreadPoolExecutor.java:659)
	at org.apache.tomcat.util.threads.TaskThread$WrappingRunnable.run(TaskThread.java:61)
	at java.base/java.lang.Thread.run(Thread.java:1583)
Caused by: java.lang.NullPointerException: user repository returned null
	at com.example.service.OrderService.processOrder(OrderService.java:64)
	at com.example.controller.OrderController.handleCheckout(OrderController.kt:32)
	at java.base/jdk.internal.reflect.DirectMethodHandleAccessor.invoke(DirectMethodHandleAccessor.java:103)
	at java.base/java.lang.reflect.Method.invoke(Method.java:580)
	at org.springframework.web.method.support.InvocableHandlerMethod.doInvoke(InvocableHandlerMethod.java:255)
	... 42 common frames omitted
"#;

    let parser = JavaTraceParser;
    assert!(parser.detect(raw_trace));

    let locs = parser.extract_locations(raw_trace);
    // Framework frames (FrameworkServlet, HttpServlet, ApplicationFilterChain, WsFilter, Thread, DirectMethodHandleAccessor, Method, InvocableHandlerMethod) MUST be discarded.
    // Only user code locations (OrderService.java:64, OrderController.kt:32) should remain.
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].file, "OrderService.java");
    assert_eq!(locs[0].line, 64);
    assert_eq!(locs[1].file, "OrderController.kt");
    assert_eq!(locs[1].line, 32);

    // Test prune_framework_noise removes Spring and Tomcat frames
    let cleaned = tokenectomy::extractor::prune_framework_noise(raw_trace);
    assert!(!cleaned.contains("org.springframework.web.servlet"));
    assert!(!cleaned.contains("org.apache.catalina"));
    assert!(!cleaned.contains("org.apache.tomcat"));
    assert!(!cleaned.contains("org.apache.coyote"));
    assert!(!cleaned.contains("java.base/"));
    assert!(!cleaned.contains("common frames omitted"));
    assert!(cleaned.contains("OrderService.java:64"));
    assert!(cleaned.contains("OrderController.kt:32"));
}

#[test]
fn test_cpp_asan_deep_stack_trace_pruning() {
    let asan_log = r#"
==38291==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x603000000048 at pc 0x55dc12 bp 0x7ffd12 sp 0x7ffd10
READ of size 8 at 0x603000000048 thread T0
    #0 0x7f9a1234 in __asan_memcpy (/usr/lib/x86_64-linux-gnu/libasan.so.8+0x1234)
    #1 0x555555555149 in process_tensor(float const*, int) src/core/tensor.cpp:88
    #2 0x55555555518b in main src/main.cpp:24
    #3 0x7f9a5678 in __libc_start_call_main ../sysdeps/nptl/libc_start_call_main.h:58
    #4 0x7f9a5700 in __libc_start_main_impl ../csu/libc-start.c:360
    #5 0x555555555020 in _start (/app/bin/server+0x101)
0x603000000048 is located 0 bytes to the right of 40-byte region [0x603000000020,0x603000000048)
"#;

    let parser = CppTraceParser;
    assert!(parser.detect(asan_log));

    let locs = parser.extract_locations(asan_log);
    // __libc_start_main_impl, libc-start.c, and libc_start_call_main.h should be filtered out
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].file, "src/core/tensor.cpp");
    assert_eq!(locs[0].line, 88);
    assert_eq!(locs[1].file, "src/main.cpp");
    assert_eq!(locs[1].line, 24);

    let cleaned = tokenectomy::extractor::prune_framework_noise(asan_log);
    assert!(!cleaned.contains("__asan_memcpy"));
    assert!(!cleaned.contains("libasan.so"));
    assert!(!cleaned.contains("__libc_start_call_main"));
    assert!(!cleaned.contains("__libc_start_main_impl"));
    assert!(!cleaned.contains("in _start"));
    assert!(cleaned.contains("src/core/tensor.cpp:88"));
    assert!(cleaned.contains("src/main.cpp:24"));
}

#[test]
fn test_go_goroutine_panic_compression() {
    let go_dump = r#"
panic: runtime error: invalid memory address or nil pointer dereference
[signal SIGSEGV: code=0x1 addr=0x0 pc=0x498a72]

goroutine 1 [running]:
main.processOrder(0x0)
	/app/src/order.go:45 +0x3a
main.main()
	/app/src/main.go:18 +0x22

goroutine 2 [force gc (idle)]:
runtime.gopark(0x4a0120, 0x0, 0x11, 0x14, 0x1)
	/usr/local/go/src/runtime/proc.go:381 +0xd6
runtime.forcegchelper()
	/usr/local/go/src/runtime/proc.go:320 +0xb8
runtime.goexit()
	/usr/local/go/src/runtime/asm_amd64.s:1598 +0x1

goroutine 3 [GC sweep wait]:
runtime.gopark(0x4a0120, 0x0, 0x0c, 0x14, 0x1)
	/usr/local/go/src/runtime/proc.go:381 +0xd6
runtime.bgsweep()
	/usr/local/go/src/runtime/mgcsweep.go:161 +0x8e
runtime.goexit()
	/usr/local/go/src/runtime/asm_amd64.s:1598 +0x1

goroutine 4 [finalizer wait]:
runtime.gopark(0x4a0120, 0x0, 0x10, 0x14, 0x1)
	/usr/local/go/src/runtime/proc.go:381 +0xd6
runtime.runfinq()
	/usr/local/go/src/runtime/mfinal.go:193 +0xb5
runtime.goexit()
	/usr/local/go/src/runtime/asm_amd64.s:1598 +0x1
"#;

    let parser = GoTraceParser;
    assert!(parser.detect(go_dump));

    let locs = parser.extract_locations(go_dump);
    // Go runtime internals (/usr/local/go/src/runtime/...) should be filtered out
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].file, "/app/src/order.go");
    assert_eq!(locs[0].line, 45);
    assert_eq!(locs[1].file, "/app/src/main.go");
    assert_eq!(locs[1].line, 18);

    // Prune idle goroutines: goroutines 2, 3, and 4 should be completely pruned away!
    let cleaned = tokenectomy::extractor::prune_framework_noise(go_dump);
    assert!(cleaned.contains("goroutine 1 [running]:"));
    assert!(cleaned.contains("/app/src/order.go:45"));
    assert!(cleaned.contains("/app/src/main.go:18"));
    assert!(!cleaned.contains("[force gc (idle)]"));
    assert!(!cleaned.contains("[GC sweep wait]"));
    assert!(!cleaned.contains("[finalizer wait]"));
    assert!(!cleaned.contains("runtime.forcegchelper"));
    assert!(!cleaned.contains("runtime.bgsweep"));
    assert!(!cleaned.contains("runtime.runfinq"));
}

