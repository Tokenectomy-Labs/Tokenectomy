use crate::redact::redact_secrets;
use crate::extractor::is_dependency_file;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Default)]
pub struct ProxySanitizeStats {
    pub raw_chars: usize,
    pub sanitized_chars: usize,
    pub secrets_redacted: usize,
}

/// Surgically cleans prompt payload: redacts secrets and strips framework dependency noise.
pub fn sanitize_prompt_payload(payload: &Value) -> (Value, ProxySanitizeStats) {
    let mut stats = ProxySanitizeStats::default();
    let mut modified = payload.clone();

    if let Some(messages) = modified.get_mut("messages").and_then(|m| m.as_array_mut()) {
        for msg in messages {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                stats.raw_chars += content.len();
                
                // 1. Redact credentials
                let redacted = redact_secrets(content);
                if redacted != content {
                    stats.secrets_redacted += 1;
                }

                // 2. Strip noisy framework stack traces if detected
                let mut cleaned_lines = Vec::new();
                for line in redacted.lines() {
                    let is_noise = line.trim_start().starts_with("at ")
                        || line.trim_start().starts_with("File \"")
                        || line.trim_start().starts_with("#")
                        || line.trim_start().starts_with("goroutine ");

                    if is_noise && is_dependency_file(line) {
                        // Skip internal runtime / vendor frames
                        continue;
                    }
                    cleaned_lines.push(line);
                }

                let final_content = cleaned_lines.join("\n");
                stats.sanitized_chars += final_content.len();

                if let Some(c_field) = msg.get_mut("content") {
                    *c_field = Value::String(final_content);
                }
            }
        }
    }

    (modified, stats)
}

/// Runs the AI Gateway Reverse Proxy on the specified bind address.
pub async fn run_reverse_proxy(bind_addr: &str, upstream_url: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind_addr).await?;
    let client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .build()?;
    let client = Arc::new(client);
    let upstream = Arc::new(upstream_url.trim_end_matches('/').to_string());

    println!("⚡ Tokenectomy AI Gateway Proxy active on http://{}", bind_addr);
    println!("🔗 Forwarding to upstream: {}", upstream);
    println!("🛡️ Active filters: Zero-Leak Redaction + Polyglot Framework Surgery");

    loop {
        let (mut socket, peer_addr) = listener.accept().await?;
        let client_clone = Arc::clone(&client);
        let upstream_clone = Arc::clone(&upstream);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            let mut total_read = 0;

            // Read HTTP request headers & initial body chunk
            while total_read < buf.len() {
                match socket.read(&mut buf[total_read..]).await {
                    Ok(0) => break,
                    Ok(n) => {
                        total_read += n;
                        if buf[..total_read].windows(4).any(|w| w == b"\r\n\r\n") {
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Socket read error from {}: {}", peer_addr, e);
                        return;
                    }
                }
            }

            if total_read == 0 {
                return;
            }

            let req_str = String::from_utf8_lossy(&buf[..total_read]);
            let mut lines = req_str.split("\r\n");
            let req_line = lines.next().unwrap_or("");
            let parts: Vec<&str> = req_line.split_whitespace().collect();

            if parts.len() < 2 {
                return;
            }

            let method = parts[0];
            let path = parts[1];

            // Health check endpoint
            if path == "/health" || path == "/v1/health" {
                let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"status\":\"ok\",\"service\":\"tokenectomy-gateway\",\"version\":\"1.0.0\"}\r\n";
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            // Extract HTTP headers & body
            let header_end = req_str.find("\r\n\r\n").unwrap_or(total_read);
            let raw_headers = &req_str[..header_end];
            let body_start = header_end + 4;
            
            // Determine Content-Length
            let mut content_length = 0;
            let mut auth_header = None;
            for h in raw_headers.lines().skip(1) {
                if let Some((k, v)) = h.split_once(':') {
                    let k_lower = k.trim().to_lowercase();
                    if k_lower == "content-length" {
                        content_length = v.trim().parse::<usize>().unwrap_or(0);
                    } else if k_lower == "authorization" {
                        auth_header = Some(v.trim().to_string());
                    }
                }
            }

            // Read remaining body if needed
            let mut body_bytes = buf[body_start..total_read].to_vec();
            while body_bytes.len() < content_length {
                let mut temp = vec![0u8; 16384];
                match socket.read(&mut temp).await {
                    Ok(0) => break,
                    Ok(n) => body_bytes.extend_from_slice(&temp[..n]),
                    Err(_) => break,
                }
            }

            // Process and sanitize body if JSON
            let mut final_body = body_bytes;
            if let Ok(json) = serde_json::from_slice::<Value>(&final_body) {
                let (sanitized, stats) = sanitize_prompt_payload(&json);
                if stats.secrets_redacted > 0 || stats.raw_chars > stats.sanitized_chars {
                    let saved_pct = if stats.raw_chars > 0 {
                        (stats.raw_chars.saturating_sub(stats.sanitized_chars) as f64 / stats.raw_chars as f64) * 100.0
                    } else {
                        0.0
                    };
                    eprintln!(
                        "⚡ [Proxy] Sanitized request from {}: -{:.1}% noise cut, {} secrets redacted",
                        peer_addr, saved_pct, stats.secrets_redacted
                    );
                }
                if let Ok(new_bytes) = serde_json::to_vec(&sanitized) {
                    final_body = new_bytes;
                }
            }

            // Forward to upstream
            let target_url = format!("{}{}", upstream_clone, path);
            let mut req_builder = match method {
                "POST" => client_clone.post(&target_url),
                "GET" => client_clone.get(&target_url),
                _ => client_clone.request(reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET), &target_url),
            };

            if let Some(auth) = auth_header {
                req_builder = req_builder.header("Authorization", auth);
            }
            req_builder = req_builder
                .header("Content-Type", "application/json")
                .body(final_body);

            match req_builder.send().await {
                Ok(mut upstream_resp) => {
                    let status = upstream_resp.status();
                    let mut head = format!("HTTP/1.1 {} {}\r\n", status.as_u16(), status.canonical_reason().unwrap_or(""));
                    for (k, v) in upstream_resp.headers() {
                        head.push_str(&format!("{}: {}\r\n", k.as_str(), v.to_str().unwrap_or("")));
                    }
                    head.push_str("\r\n");
                    let _ = socket.write_all(head.as_bytes()).await;

                    while let Ok(Some(chunk)) = upstream_resp.chunk().await {
                        if socket.write_all(&chunk).await.is_err() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    let err_json = serde_json::json!({
                        "error": {
                            "message": format!("Tokenectomy Gateway: Failed to reach upstream: {}", e),
                            "type": "bad_gateway",
                            "code": 502
                        }
                    });
                    let resp = format!(
                        "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}\r\n",
                        err_json
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                }
            }
        });
    }
}
