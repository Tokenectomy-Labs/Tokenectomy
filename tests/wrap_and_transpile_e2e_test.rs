use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokenectomy::proxy::{run_reverse_proxy_with_config, ProxyConfig};
use tokenectomy::wrap::find_available_port;

#[tokio::test]
async fn test_wrap_find_available_port() {
    let port = find_available_port(19450).await;
    assert!(port >= 19450);
}

#[tokio::test]
async fn test_wrap_command_execution() {
    let port = find_available_port(19500).await;
    let bind_addr = format!("127.0.0.1:{}", port);

    let exit_code = tokenectomy::wrap::run_wrap_command(
        vec!["echo".to_string(), "Tokenectomy wrap verified".to_string()],
        Some(bind_addr),
        Some("auto".to_string()),
        None,
        1,
        false,
    )
    .await
    .expect("Wrap execution failed");

    assert_eq!(exit_code, 0);
}

#[tokio::test]
async fn test_proxy_end_to_end_anthropic_to_openai_transpilation() {
    // 1. Spawn a mock upstream OpenAI server
    let mock_upstream_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_port = mock_upstream_listener.local_addr().unwrap().port();
    let upstream_url = format!("http://127.0.0.1:{}/v1", upstream_port);

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = mock_upstream_listener.accept().await {
            let mut buf = vec![0u8; 8192];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            let req_text = String::from_utf8_lossy(&buf[..n]);

            // Upstream must receive POST /v1/chat/completions (not /v1/messages)
            assert!(req_text.contains("POST /v1/chat/completions"));

            // Must contain OpenAI format messages
            assert!(req_text.contains("\"messages\""));

            let mock_openai_resp = json!({
                "id": "chatcmpl-mock-e2e",
                "object": "chat.completion",
                "created": 1700000000,
                "model": "gpt-4o",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Transpilation successfully verified by Tokenectomy!"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 15,
                    "completion_tokens": 8,
                    "total_tokens": 23
                }
            })
            .to_string();

            let http_resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                mock_openai_resp.len(),
                mock_openai_resp
            );
            let _ = socket.write_all(http_resp.as_bytes()).await;
        }
    });

    // 2. Spawn Tokenectomy AI Gateway configured with OpenAI upstream
    let proxy_port = find_available_port(19600).await;
    let proxy_bind = format!("127.0.0.1:{}", proxy_port);

    let config = ProxyConfig {
        bind_addr: proxy_bind.clone(),
        upstream_url,
        allow_remote: false,
        auth_token: None,
        max_hourly_tokens: None,
        max_retries: 1,
        auto_retry_429: false,
    };

    tokio::spawn(async move {
        let _ = run_reverse_proxy_with_config(config).await;
    });

    // Wait for gateway ready
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 3. Client sends Anthropic format request (/v1/messages) to Tokenectomy Gateway
    let client = reqwest::Client::new();
    let ant_request_body = json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1024,
        "system": "You are a test agent.",
        "messages": [
            {"role": "user", "content": "Hello from Claude Code!"}
        ]
    });

    let resp = client
        .post(format!("http://{}/v1/messages", proxy_bind))
        .header("x-api-key", "test-anthropic-key")
        .header("anthropic-version", "2023-06-01")
        .json(&ant_request_body)
        .send()
        .await
        .expect("Client send failed");

    assert_eq!(resp.status().as_u16(), 200);

    let ant_resp_json: serde_json::Value = resp.json().await.expect("Parse JSON response");

    // Client must receive Anthropic format response!
    assert_eq!(ant_resp_json["type"], "message");
    assert_eq!(ant_resp_json["role"], "assistant");
    assert_eq!(ant_resp_json["stop_reason"], "end_turn");
    assert_eq!(ant_resp_json["usage"]["input_tokens"], 15);
    assert_eq!(ant_resp_json["usage"]["output_tokens"], 8);

    let content = ant_resp_json["content"].as_array().unwrap();
    assert_eq!(
        content[0]["text"],
        "Transpilation successfully verified by Tokenectomy!"
    );
}
