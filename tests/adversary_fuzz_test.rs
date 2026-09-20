use tokenectomy::proxy::{
    decode_chunked_body, is_allowed_host, sanitize_prompt_payload, ProxyMetrics, resolve_upstream_target,
};
use serde_json::json;

#[test]
fn test_exploit_chunked_decoder_integer_overflow() {
    // Attack payload: A chunk header specifying usize::MAX as hex (e.g. FFFFFFFFFFFFFFFF)
    // In unhardened code, data_start + chunk_len overflows usize!
    let exploit_payload = b"FFFFFFFFFFFFFFFF\r\nmalicious_data\r\n";
    let res = decode_chunked_body(exploit_payload);
    assert!(res.is_err(), "Must cleanly reject overflow chunk size without panicking");
}

#[test]
fn test_exploit_ipv6_host_header_splitting() {
    // Browsers connecting to http://[::1]:8080 send Host: [::1]:8080
    // Unhardened split(':').next() yields "[" which causes 403 Forbidden!
    assert!(
        is_allowed_host("[::1]:8080", "127.0.0.1:8080"),
        "Must correctly identify [::1]:8080 as allowed loopback host"
    );
    assert!(
        is_allowed_host("[::1]", "127.0.0.1:8080"),
        "Must correctly identify [::1] as allowed loopback host"
    );
}

#[test]
fn test_exploit_circuit_breaker_integer_overflow_bypass() {
    let metrics = ProxyMetrics::new();
    let max_hourly = Some(1000);
    // Initial tokens = 10
    assert!(metrics.check_and_record_hourly_tokens(10, max_hourly));
    // Adversary payload requesting u64::MAX - 5 tokens
    // Unhardened current_hourly + new_tokens wraps to 4 in release (bypassing limit 1000!)
    // and panics with arithmetic overflow in debug!
    let tripped = !metrics.check_and_record_hourly_tokens(u64::MAX - 5, max_hourly);
    assert!(tripped, "Circuit breaker must trip and not overflow/wrap-around on huge token counts");
}

#[test]
fn test_exploit_leak_in_content_string_array() {
    let payload = json!({
        "messages": [
            {
                "role": "user",
                "content": [
                    "AWS key leaked: AKIAIOSFODNN7EXAMPLE",
                    "password=\"supersecret123\""
                ]
            }
        ]
    });

    let (sanitized, stats) = sanitize_prompt_payload(&payload);
    let rendered = serde_json::to_string(&sanitized).unwrap();
    assert!(
        !rendered.contains("AKIAIOSFODNN7EXAMPLE"),
        "String array in content must be redacted: {}", rendered
    );
    assert!(
        !rendered.contains("supersecret123"),
        "Passwords in string array content must be redacted: {}", rendered
    );
    assert!(stats.secrets_redacted >= 2, "Stats must count redacted secrets in string array");
}

#[test]
fn test_upstream_target_clean_v1_exact() {
    let target = resolve_upstream_target("http://127.0.0.1:11434/v1", "/v1", &[]);
    assert_eq!(target, "http://127.0.0.1:11434/v1", "Must not duplicate to /v1/v1");
}
