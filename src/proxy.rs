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

pub const MAX_HEADER_SIZE: usize = 64 * 1024; // 64 KB
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MB
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub const UPSTREAM_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
pub const MAX_CONCURRENT_CONNECTIONS: usize = 128;

/// Checks if a bind address is strictly local loopback (127.0.0.1, localhost, ::1).
pub fn is_loopback(bind_addr: &str) -> bool {
    let clean = bind_addr.trim();
    if clean == "localhost" || clean == "127.0.0.1" || clean == "::1" || clean == "[::1]" {
        return true;
    }
    if let Ok(ip) = clean.parse::<std::net::IpAddr>() {
        return ip.is_loopback();
    }
    if let Ok(addr) = clean.parse::<std::net::SocketAddr>() {
        return addr.ip().is_loopback();
    }
    if let Some((host, _)) = clean.rsplit_once(':') {
        let host_clean = host.trim_matches(|c| c == '[' || c == ']');
        if host_clean == "localhost" || host_clean == "127.0.0.1" || host_clean == "::1" {
            return true;
        }
        if let Ok(ip) = host_clean.parse::<std::net::IpAddr>() {
            return ip.is_loopback();
        }
    }
    false
}

/// Runs the AI Gateway Reverse Proxy on the specified bind address (default loopback).
pub async fn run_reverse_proxy(bind_addr: &str, upstream_url: &str) -> anyhow::Result<()> {
    run_reverse_proxy_configured(bind_addr, upstream_url, false, None).await
}

/// Runs the AI Gateway Reverse Proxy with explicit remote binding authorization and security limits.
pub async fn run_reverse_proxy_configured(
    bind_addr: &str,
    upstream_url: &str,
    allow_remote: bool,
    auth_token: Option<&str>,
) -> anyhow::Result<()> {
    let loopback = is_loopback(bind_addr);
    if !loopback {
        if !allow_remote {
            return Err(anyhow::anyhow!(
                "Security Violation: Binding to non-loopback address '{}' requires explicit '--allow-remote' flag.",
                bind_addr
            ));
        }
        if auth_token.is_none() || auth_token.map(|t| t.trim().is_empty()).unwrap_or(true) {
            return Err(anyhow::anyhow!(
                "Security Violation: Remote proxy mode on '{}' requires an authentication token via '--proxy-token' or 'TOKENECTOMY_PROXY_TOKEN'.",
                bind_addr
            ));
        }
    }

    let listener = TcpListener::bind(bind_addr).await?;
    let client = reqwest::Client::builder()
        .tcp_nodelay(true)
        .timeout(UPSTREAM_TIMEOUT)
        .build()?;
    let client = Arc::new(client);
    let upstream = Arc::new(upstream_url.trim_end_matches('/').to_string());
    let required_token = auth_token.map(|t| Arc::new(t.trim().to_string()));
    let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_CONNECTIONS));

    println!("⚡ Tokenectomy AI Gateway Proxy active on http://{}", bind_addr);
    println!("🔗 Forwarding to upstream: {}", upstream);
    if !loopback {
        println!("🔒 Remote mode ACTIVE (Protected with mandatory bearer authentication)");
    }
    println!("🛡️ Active filters: Zero-Leak Redaction + Polyglot Framework Surgery");

    loop {
        let (mut socket, peer_addr) = listener.accept().await?;
        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                log::warn!("Connection limit reached (max {}). Dropping {}", MAX_CONCURRENT_CONNECTIONS, peer_addr);
                let resp = "HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Proxy connection capacity saturated\",\"code\":503}}\r\n";
                let _ = socket.write_all(resp.as_bytes()).await;
                continue;
            }
        };

        let client_clone = Arc::clone(&client);
        let upstream_clone = Arc::clone(&upstream);
        let token_clone = required_token.clone();

        tokio::spawn(async move {
            let _permit = permit;
            let handler = async {
                let mut buf = vec![0u8; 16384];
                let mut total_read = 0;

                // Read HTTP request headers with MAX_HEADER_SIZE bound
                while total_read < MAX_HEADER_SIZE {
                    if buf.len() <= total_read {
                        buf.resize(buf.len() * 2, 0);
                    }
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
                let header_end = match req_str.find("\r\n\r\n") {
                    Some(idx) => idx,
                    None => {
                        let resp = "HTTP/1.1 431 Request Header Fields Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Headers exceed 64KB limit\",\"code\":431}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                };

                let raw_headers = &req_str[..header_end];
                let body_start = header_end + 4;
                let mut lines = raw_headers.split("\r\n");
                let req_line = lines.next().unwrap_or("");
                let parts: Vec<&str> = req_line.split_whitespace().collect();

                if parts.len() < 2 {
                    return;
                }

                let method = parts[0];
                let path = parts[1];

                // Health check endpoint
                if path == "/health" || path == "/v1/health" {
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"status\":\"ok\",\"service\":\"tokenectomy-gateway\",\"version\":\"1.1.2\"}\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // Extract Content-Length & Authorization
                let mut content_length = 0;
                let mut client_auth = None;
                for h in raw_headers.lines().skip(1) {
                    if let Some((k, v)) = h.split_once(':') {
                        let k_lower = k.trim().to_lowercase();
                        if k_lower == "content-length" {
                            content_length = v.trim().parse::<usize>().unwrap_or(0);
                        } else if k_lower == "authorization" {
                            client_auth = Some(v.trim().to_string());
                        } else if k_lower == "x-api-key" && client_auth.is_none() {
                            client_auth = Some(format!("Bearer {}", v.trim()));
                        }
                    }
                }

                // Enforce authentication for remote mode
                if let Some(expected_token) = &token_clone {
                    let is_authed = match &client_auth {
                        Some(auth) => {
                            let token_part = auth.strip_prefix("Bearer ").unwrap_or(auth.as_str()).trim();
                            token_part == expected_token.as_str()
                        }
                        None => false,
                    };
                    if !is_authed {
                        let resp = "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Unauthorized: Missing or invalid proxy authorization token\",\"code\":401}}\r\n";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }
                }

                // Enforce MAX_BODY_SIZE
                if content_length > MAX_BODY_SIZE {
                    let resp = "HTTP/1.1 413 Payload Too Large\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{\"error\":{\"message\":\"Payload exceeds 10MB limit\",\"code\":413}}\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                // Read remaining body
                let mut body_bytes = buf[body_start..total_read].to_vec();
                while body_bytes.len() < content_length {
                    let to_read = (content_length - body_bytes.len()).min(16384);
                    let mut temp = vec![0u8; to_read];
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
                    _ => client_clone.request(
                        reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                        &target_url,
                    ),
                };

                if let Some(auth) = client_auth {
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
            };

            if let Err(_) = tokio::time::timeout(REQUEST_TIMEOUT, handler).await {
                log::warn!("Proxy connection from {} timed out after {}s", peer_addr, REQUEST_TIMEOUT.as_secs());
            }
        });
    }
}
